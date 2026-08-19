//! The Agent interpreter (U7): a generic step-graph walker. All domain
//! behavior comes from the loaded [`crate::Ontology`] — this module owns
//! only the mechanics doc 05 § Execution semantics describes: one current
//! step, precondition checks, one transaction per step, edge selection,
//! termination.
//!
//! ## The case subject
//!
//! Every condition a workflow evaluates (`ReasoningStep.goal`,
//! `ActionStep.requires`) is written with no `horis:subject` — doc 05/06
//! describe it as evaluated "against a runtime-resolved current case
//! instance," but neither doc names the resolution rule. This build's
//! reading (an implementation note, not a `[DECIDE]`: no framework marker
//! asks for one — the same posture as `workflow::model`'s `ProposalStep`
//! candidate-list simplification): the case subject is the instance a
//! [`PerceptionStep`] asserts of the **first** class named in its
//! `horis:extracts` list. Every step after `Intake` in the triage workflow
//! evaluates conditions, resolves `ActionStep` tool arguments, and looks up
//! a `NarrationStep`'s `horis:narrates` property against that one `Ref` for
//! the rest of the run — matching "one Agent + one World per case" (D12).
//!
//! ## Tool argument resolution
//!
//! Doc 07: "arguments resolved from the graph" and "the ActionStep names
//! the bindings" — but no worked example or vocabulary term shows how; the
//! `:Route` step in doc 05's own worked example declares no argument
//! binding at all. This build resolves each `horis:arg` **by type**,
//! against the case subject's own data — no text parsing, matching doc 07's
//! promise:
//!
//! - An arg whose declared type is a class the case subject's own class
//!   *is* (subsumption-aware) resolves to the case subject itself
//!   (`:ehr.assign_queue`'s `patient` arg, typed `:Patient`, against a
//!   `:Patient` case).
//! - Otherwise, the case subject's own asserted properties
//!   (`Instance::properties`) are searched for exactly one whose declared
//!   `rdfs:range` matches the arg's type (subsumption-aware for a class
//!   range, exact for a datatype range); its value is taken
//!   (`:ehr.assign_queue`'s `queue` arg, typed `:AcuityLevel`, resolves via
//!   `:acuity` — the case's only `:AcuityLevel`-ranged property).
//! - Zero or more than one match is a runtime [`ToolError::InvocationFailed`]
//!   naming the ambiguity — the binding is never invoked with a guessed
//!   argument.
//!
//! ## Deferred step kinds (U12)
//!
//! `ChoiceStep`/`ProposalStep`/`WaitStep`/`SubflowStep` parse (U2) but do
//! not execute in v1: reaching one returns
//! [`WorkflowError::StepDeferred`] naming the concrete step and kind —
//! never a panic, never a silent no-op (D3, D6).
//!
//! ## What this unit does not yet do
//!
//! Trace emission (U9: `trace::emit` builders for the six new step kinds,
//! wired into `run`) is a tracked follow-up, not part of this unit —
//! `RunOutcome` exposes the final `World` state, the terminal step, the
//! computed `horis:decides` value, and whether a fallback edge fired, which
//! is everything the DoD's world-state assertions need.
//!
//! Spec: `docs/framework/05-workflows.md`, `06-llm-boundaries.md`,
//! `07-tools.md`, `12-rust-api.md` · Owner: PRP-004

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::bridge::{Bridge, NarrateRequest, PerceiveRequest, PerceiveResult};
use crate::ontology::facade::Ontology;
use crate::ontology::schema::{Cardinality, PropertyDecl, PropertyRange, Schema};
use crate::ontology::term::{Datatype, Iri, Ref, Value};
use crate::reason::eval::holds;
use crate::workflow::WorkflowError;
use crate::workflow::model::{
    ActionStep, NarrationStep, PerceptionStep, ReasoningStep, Step, StepBody, Workflow,
};
use crate::workflow::tool::ToolError;
use crate::workflow::tool::{ArgType, Binding, EffectValue, Tool as ToolDecl};
use crate::world::{Provenance, World};

/// A Rust implementation of one `horis:Tool`'s `rust:` binding — registered
/// under the binding's name via [`AgentBuilder::tool`]. Returns the values
/// for the declared effect properties whose [`EffectValue`] is a
/// [`EffectValue::TypedPlaceholder`] (a `horis:value horis:now` effect is
/// filled in by the runtime, never by the tool). A returned value not
/// covered by `horis:effect`, or of the wrong shape, is rejected at the
/// validation gate (provenance `tool`) — the tool never bypasses it.
pub trait Tool: Send + Sync {
    fn call(&self, args: &BTreeMap<String, Value>) -> Result<BTreeMap<Iri, Value>, String>;
}

/// One `agent.run(...)` call's input (doc 12).
#[derive(Debug, Clone)]
pub struct RunInput {
    /// The natural-language input the run's `PerceptionStep`s consume.
    pub text: String,
    /// The transaction-stamped `horis:now` (U11) every `ActionStep`'s
    /// `horis:now`-sentinel tool effect uses this run — read by the caller
    /// exactly once, at transaction open, never re-read inside chaining
    /// (CLAUDE.md's "no wall-clock ... in inference" rule, made concrete for
    /// tool effects). `horismos-core` has no clock feature to read one
    /// itself by design; a caller with one (a demo binary, `horismos-py`)
    /// supplies `chrono::Utc::now().into()` or a fixed instant in tests.
    pub now: chrono::DateTime<chrono::FixedOffset>,
}

impl RunInput {
    pub fn new(text: impl Into<String>, now: chrono::DateTime<chrono::FixedOffset>) -> Self {
        RunInput {
            text: text.into(),
            now,
        }
    }
}

/// One `agent.run(...)` call's result: the terminal step reached, the
/// computed decision (D5), and whether any step's fallback/error edge
/// fired — everything the caller needs without re-deriving it from the
/// final `World` state.
#[derive(Debug, Clone)]
pub struct RunOutcome {
    pub terminal: Iri,
    /// `Some((property, value))` when the workflow's `horis:decides`
    /// property holds a value on the case subject at the terminal step
    /// (D5); `None` if the workflow declares no `horis:decides` or the run
    /// never gave the property a value.
    pub decision: Option<(Iri, Value)>,
    /// True iff any step in this run followed a fallback/error edge
    /// (`horis:fallback`/`horis:onError`/`horis:onUnsatisfied`) rather than
    /// its primary `horis:next`.
    pub fallback_taken: bool,
    /// The case subject this run resolved (see the module doc), if
    /// perception ever ran. `None` for a workflow whose `startsAt` step is
    /// not a `PerceptionStep` and never reaches one.
    pub case: Option<Ref>,
}

/// Builds an [`Agent`]: the ontology, which of its workflows to run, and
/// the `Bridge`/`Tool` implementations bound to the `horis:model` prefixes
/// and `rust:` binding names the workflow declares.
pub struct AgentBuilder<'a> {
    onto: &'a Ontology,
    workflow: Option<&'a Workflow>,
    bridges: BTreeMap<String, Arc<dyn Bridge>>,
    tools: BTreeMap<String, Arc<dyn Tool>>,
}

impl<'a> AgentBuilder<'a> {
    fn new(onto: &'a Ontology) -> Self {
        AgentBuilder {
            onto,
            workflow: None,
            bridges: BTreeMap::new(),
            tools: BTreeMap::new(),
        }
    }

    /// Selects the `horis:Workflow` to run, by CURIE or full IRI (e.g.
    /// `":Triage"` against `triage.ttl`'s own `@prefix : <...#>`).
    pub fn workflow(mut self, name: &str) -> Result<Self, WorkflowError> {
        let iri = self
            .onto
            .node_ref(name)
            .map_err(|e| WorkflowError::UnknownWorkflow {
                name: name.to_owned(),
                reason: e.domain_text(),
            })?;
        let workflow = self
            .onto
            .workflow(iri)
            .ok_or_else(|| WorkflowError::UnknownWorkflow {
                name: name.to_owned(),
                reason: "not a declared horis:Workflow".to_owned(),
            })?;
        self.workflow = Some(workflow);
        Ok(self)
    }

    /// Binds a `Bridge` implementation under `prefix` — every step whose
    /// `horis:model` string starts with `"{prefix}:"` (or, for a step
    /// declaring no `horis:model`, the sole registered bridge) uses it. A
    /// deployment may bind a different concrete model than the ontology's
    /// declared string names for the same prefix (D8) — the trace records
    /// `Bridge::id()`, the model actually invoked.
    pub fn bridge(mut self, prefix: impl Into<String>, bridge: impl Bridge + 'static) -> Self {
        self.bridges.insert(prefix.into(), Arc::new(bridge));
        self
    }

    /// Registers a [`Tool`] implementation under `name` — the runtime
    /// resolves an `ActionStep`'s `rust:{name}` binding to it.
    pub fn tool(mut self, name: impl Into<String>, tool: impl Tool + 'static) -> Self {
        self.tools.insert(name.into(), Arc::new(tool));
        self
    }

    /// Validates every binding the selected workflow needs is registered,
    /// then builds the [`Agent`]. A workflow naming a `rust:` tool binding
    /// with no registered implementation, or a `horis:model` prefix with no
    /// registered bridge, fails here — never mid-run.
    pub fn build(self) -> Result<Agent<'a>, WorkflowError> {
        let workflow = self.workflow.ok_or(WorkflowError::NoWorkflowSelected)?;

        for step in workflow.steps.values() {
            if let StepBody::Action(action) = &step.body {
                let tool_decl =
                    self.onto
                        .tool(action.tool)
                        .ok_or_else(|| WorkflowError::UnboundTool {
                            tool: self.onto.resolve(action.tool).to_owned(),
                        })?;
                if let Binding::Rust(name) = &tool_decl.binding
                    && !self.tools.contains_key(name)
                {
                    return Err(WorkflowError::UnboundTool { tool: name.clone() });
                }
            }
            let model = match &step.body {
                StepBody::Perception(p) => p.model.as_deref(),
                StepBody::Narration(n) => n.model.as_deref(),
                StepBody::Proposal(p) => p.model.as_deref(),
                _ => None,
            };
            if let Some(model) = model {
                let prefix = model.split(':').next().unwrap_or(model);
                if !self.bridges.contains_key(prefix) && self.bridges.len() != 1 {
                    return Err(WorkflowError::UnboundBridge {
                        prefix: prefix.to_owned(),
                    });
                }
            }
        }

        Ok(Agent {
            onto: self.onto,
            workflow,
            bridges: self.bridges,
            tools: self.tools,
        })
    }
}

/// A workflow bound to concrete `Bridge`/`Tool` implementations, ready to
/// run against a `World`.
pub struct Agent<'a> {
    onto: &'a Ontology,
    workflow: &'a Workflow,
    bridges: BTreeMap<String, Arc<dyn Bridge>>,
    tools: BTreeMap<String, Arc<dyn Tool>>,
}

impl<'a> Agent<'a> {
    /// Starts building an `Agent` for `onto`.
    pub fn builder(onto: &'a Ontology) -> AgentBuilder<'a> {
        AgentBuilder::new(onto)
    }

    fn bridge_for(&self, model: Option<&str>) -> Result<&Arc<dyn Bridge>, WorkflowError> {
        let prefix = model.and_then(|m| m.split(':').next());
        if let Some(prefix) = prefix
            && let Some(bridge) = self.bridges.get(prefix)
        {
            return Ok(bridge);
        }
        if self.bridges.len() == 1 {
            return Ok(self.bridges.values().next().expect("len checked above"));
        }
        Err(WorkflowError::UnboundBridge {
            prefix: prefix.unwrap_or("(none declared)").to_owned(),
        })
    }

    /// Runs the workflow from its `horis:startsAt` step to a terminal step,
    /// against `world`, once per call — one `World` mutation per step, per
    /// doc 05 § Execution semantics.
    pub async fn run(
        &self,
        world: &mut World,
        input: RunInput,
    ) -> Result<RunOutcome, WorkflowError> {
        let mut current = self
            .workflow
            .step(self.workflow.starts_at)
            .expect("parse-time validation guarantees startsAt names a declared step")
            .clone();
        let mut case: Option<Ref> = None;
        let mut fallback_taken = false;

        loop {
            if current.terminal {
                let decision = self.workflow.decides.and_then(|property| {
                    case.and_then(|c| world.value(c, property))
                        .cloned()
                        .map(|v| (property, v))
                });
                return Ok(RunOutcome {
                    terminal: current.iri,
                    decision,
                    fallback_taken,
                    case,
                });
            }

            let outcome = self.execute_step(world, &current, case, &input).await?;
            if outcome.is_fallback {
                fallback_taken = true;
            }
            if let Some(resolved_case) = outcome.case {
                case = Some(resolved_case);
            }
            current = self
                .workflow
                .step(outcome.next)
                .expect("edge targets are validated at load time")
                .clone();
        }
    }

    async fn execute_step(
        &self,
        world: &mut World,
        step: &Step,
        case: Option<Ref>,
        input: &RunInput,
    ) -> Result<StepOutcome, WorkflowError> {
        let step_name = self.onto.resolve(step.iri).to_owned();
        match &step.body {
            StepBody::Perception(p) => self.execute_perception(world, p, input).await,
            StepBody::Reasoning(r) => self.execute_reasoning(world, case, r),
            StepBody::Action(a) => self.execute_action(world, &step_name, case, a, input.now),
            StepBody::Narration(n) => self.execute_narration(world, &step_name, case, n).await,
            StepBody::Bare => Err(WorkflowError::StepDeferred {
                step: step_name,
                kind: "horis:Step (non-terminal, no body)".to_owned(),
            }),
            StepBody::Choice(_) => Err(WorkflowError::StepDeferred {
                step: step_name,
                kind: "horis:ChoiceStep".to_owned(),
            }),
            StepBody::Proposal(_) => Err(WorkflowError::StepDeferred {
                step: step_name,
                kind: "horis:ProposalStep".to_owned(),
            }),
            StepBody::Wait(_) => Err(WorkflowError::StepDeferred {
                step: step_name,
                kind: "horis:WaitStep".to_owned(),
            }),
            StepBody::Subflow(_) => Err(WorkflowError::StepDeferred {
                step: step_name,
                kind: "horis:SubflowStep".to_owned(),
            }),
        }
    }

    async fn execute_perception(
        &self,
        world: &mut World,
        p: &PerceptionStep,
        input: &RunInput,
    ) -> Result<StepOutcome, WorkflowError> {
        let bridge = self.bridge_for(p.model.as_deref())?;
        let schema = derive_extract_schema(self.onto, world.ontology().schema(), &p.extracts);

        let mut attempt = 0;
        loop {
            let result = bridge
                .perceive(PerceiveRequest {
                    text: input.text.clone(),
                    schema: schema.clone(),
                })
                .await;
            let result = match result {
                Ok(r) => r,
                Err(_) => return Ok(self.fallback_outcome(p.fallback)),
            };
            if result.confidence < p.confidence {
                return Ok(self.fallback_outcome(p.fallback));
            }
            match self.assert_perceived(world, &result) {
                Ok(primary) => {
                    return Ok(StepOutcome {
                        next: p.next,
                        is_fallback: false,
                        case: primary,
                    });
                }
                Err(_) if attempt < p.retries => attempt += 1,
                Err(_) => return Ok(self.fallback_outcome(p.fallback)),
            }
        }
    }

    /// Asserts every proposed instance (provenance `perception`); returns
    /// the case subject (the first-extracted class's asserted instance, if
    /// any — see the module doc).
    fn assert_perceived(
        &self,
        world: &mut World,
        result: &PerceiveResult,
    ) -> Result<Option<Ref>, WorkflowError> {
        let mut primary = None;
        for (index, instance) in result.instances.iter().enumerate() {
            let obj = instance
                .as_object()
                .ok_or_else(|| WorkflowError::PerceptionMalformed {
                    reason: format!("instance {index} is not a JSON object"),
                })?;
            let class_name = obj.get("class").and_then(|v| v.as_str()).ok_or_else(|| {
                WorkflowError::PerceptionMalformed {
                    reason: format!("instance {index} carries no \"class\" field"),
                }
            })?;
            let class =
                self.onto
                    .node_ref(class_name)
                    .map_err(|e| WorkflowError::PerceptionMalformed {
                        reason: e.domain_text(),
                    })?;
            let schema = world.ontology().schema();
            let mut properties = Vec::new();
            for (key, json_value) in obj {
                if key == "class" {
                    continue;
                }
                let Some(property) = term_in_namespace_of(self.onto, class, key) else {
                    continue;
                };
                let Some(decl) = schema.property(property) else {
                    continue;
                };
                let value = match decl.range {
                    Some(PropertyRange::Class(_)) => json_value
                        .as_str()
                        .and_then(|name| self.onto.node_ref(name).ok())
                        .map(|iri| Value::Ref(Ref::new(iri))),
                    _ => json_to_value(json_value, decl),
                };
                if let Some(value) = value {
                    properties.push((property, value));
                }
            }
            let asserted = world
                .assert_instance_as(Provenance::Perception, None, class, properties)
                .map_err(|e| WorkflowError::PerceptionRejected {
                    reason: e.domain_text(),
                })?;
            if primary.is_none() {
                primary = Some(asserted);
            }
        }
        Ok(primary)
    }

    fn execute_reasoning(
        &self,
        world: &World,
        case: Option<Ref>,
        r: &ReasoningStep,
    ) -> Result<StepOutcome, WorkflowError> {
        let case = case.ok_or(WorkflowError::NoCaseSubject)?;
        if holds(world, case, &r.goal) {
            Ok(StepOutcome {
                next: r.next,
                is_fallback: false,
                case: Some(case),
            })
        } else {
            Ok(StepOutcome {
                next: r.on_unreachable,
                is_fallback: true,
                case: Some(case),
            })
        }
    }

    fn execute_action(
        &self,
        world: &mut World,
        step_name: &str,
        case: Option<Ref>,
        a: &ActionStep,
        now: chrono::DateTime<chrono::FixedOffset>,
    ) -> Result<StepOutcome, WorkflowError> {
        let case = case.ok_or(WorkflowError::NoCaseSubject)?;

        if let Some(requires) = &a.requires
            && !holds(world, case, requires)
        {
            return match a.on_unsatisfied {
                Some(edge) => Ok(StepOutcome {
                    next: edge,
                    is_fallback: true,
                    case: Some(case),
                }),
                None => Err(WorkflowError::RequiresUnsatisfiable {
                    step: step_name.to_owned(),
                }),
            };
        }

        let outcome = self.invoke_tool(world, case, a.tool, now);
        match outcome {
            Ok(()) => Ok(StepOutcome {
                next: a.next.unwrap_or(a.on_error),
                is_fallback: false,
                case: Some(case),
            }),
            Err(_) => Ok(StepOutcome {
                next: a.on_error,
                is_fallback: true,
                case: Some(case),
            }),
        }
    }

    fn invoke_tool(
        &self,
        world: &mut World,
        case: Ref,
        tool_iri: Iri,
        now: chrono::DateTime<chrono::FixedOffset>,
    ) -> Result<(), ToolError> {
        let tool_decl = self
            .onto
            .tool(tool_iri)
            .expect("build() validated every ActionStep tool is declared");
        let name = match &tool_decl.binding {
            Binding::Rust(name) => name,
            Binding::Unsupported { scheme, .. } => {
                return Err(ToolError::InvocationFailed {
                    tool: self.onto.resolve(tool_iri).to_owned(),
                    reason: format!("binding scheme {scheme}: is not invocable in this build"),
                });
            }
        };
        let implementation = self
            .tools
            .get(name)
            .expect("build() validated every rust: binding has a registered Tool");

        let args = resolve_tool_args(world, world.ontology().schema(), case, tool_decl).map_err(
            |reason| ToolError::InvocationFailed {
                tool: self.onto.resolve(tool_iri).to_owned(),
                reason,
            },
        )?;

        let returned =
            implementation
                .call(&args)
                .map_err(|reason| ToolError::InvocationFailed {
                    tool: self.onto.resolve(tool_iri).to_owned(),
                    reason,
                })?;

        let now = Value::DateTime(now);
        let mut properties = Vec::new();
        for effect in &tool_decl.effect {
            let value = match &effect.value {
                EffectValue::Now => now.clone(),
                EffectValue::TypedPlaceholder(_) => returned
                    .get(&effect.property)
                    .cloned()
                    .ok_or_else(|| ToolError::InvocationFailed {
                        tool: self.onto.resolve(tool_iri).to_owned(),
                        reason: format!(
                            "tool did not return a value for {}",
                            self.onto.resolve(effect.property)
                        ),
                    })?,
            };
            properties.push((effect.property, value));
        }

        world
            .assert_on(Provenance::Tool, case, properties)
            .map_err(|e| ToolError::InvocationFailed {
                tool: self.onto.resolve(tool_iri).to_owned(),
                reason: e.domain_text(),
            })?;
        Ok(())
    }

    async fn execute_narration(
        &self,
        world: &World,
        step_name: &str,
        case: Option<Ref>,
        n: &NarrationStep,
    ) -> Result<StepOutcome, WorkflowError> {
        let case = case.ok_or(WorkflowError::NoCaseSubject)?;
        let proof_text = render_proof_or_fact(world, case, n.narrates);
        let bridge = self.bridge_for(n.model.as_deref())?;
        let result = bridge
            .narrate(NarrateRequest {
                proof_text: proof_text.clone(),
                audience: n.audience.clone().unwrap_or_default(),
            })
            .await;
        match result {
            Ok(_) => Ok(StepOutcome {
                next: n.next,
                is_fallback: false,
                case: Some(case),
            }),
            Err(e) if n.required => Err(WorkflowError::NarrationFailed {
                step: step_name.to_owned(),
                reason: e.domain_text(),
            }),
            Err(_) => Ok(StepOutcome {
                next: n.next,
                is_fallback: false,
                case: Some(case),
            }),
        }
    }

    fn fallback_outcome(&self, fallback: Iri) -> StepOutcome {
        StepOutcome {
            next: fallback,
            is_fallback: true,
            case: None,
        }
    }
}

struct StepOutcome {
    next: Iri,
    is_fallback: bool,
    case: Option<Ref>,
}

/// Resolves every `horis:arg` the tool declares against `case`'s own data —
/// see the module doc's "Tool argument resolution" section.
fn resolve_tool_args(
    world: &World,
    schema: &Schema,
    case: Ref,
    tool_decl: &ToolDecl,
) -> Result<BTreeMap<String, Value>, String> {
    let case_class = world
        .instance(case)
        .expect("case is always an asserted instance")
        .class();
    let mut args = BTreeMap::new();
    for arg in &tool_decl.takes {
        let value = match arg.ty {
            ArgType::Class(target)
                if case_class == target || schema.is_subclass_of(case_class, target) =>
            {
                Value::Ref(case)
            }
            ty => {
                let matches: Vec<Iri> = world
                    .instance(case)
                    .expect("case exists")
                    .properties()
                    .filter(|&p| {
                        schema
                            .property(p)
                            .and_then(|decl| decl.range)
                            .is_some_and(|range| range_matches(schema, range, ty))
                    })
                    .collect();
                match matches.as_slice() {
                    [only] => world
                        .value(case, *only)
                        .cloned()
                        .ok_or_else(|| format!("no value for {}", world.resolve(*only)))?,
                    [] => {
                        return Err(format!(
                            "no property on the case subject matches arg \"{}\"'s declared type",
                            arg.name
                        ));
                    }
                    _ => {
                        return Err(format!(
                            "arg \"{}\"'s declared type matches more than one property on the case subject",
                            arg.name
                        ));
                    }
                }
            }
        };
        args.insert(arg.name.clone(), value);
    }
    Ok(args)
}

fn range_matches(schema: &Schema, range: PropertyRange, ty: ArgType) -> bool {
    match (range, ty) {
        (PropertyRange::Datatype(a), ArgType::Datatype(b)) => a == b,
        (PropertyRange::Class(a), ArgType::Class(b)) => a == b || schema.is_subclass_of(a, b),
        _ => false,
    }
}

/// Renders "the proof" a `NarrationStep` narrates (doc 06: "the rendered
/// proof tree ... not the raw world"): the standing rule-derivation record
/// for `property` on `case`, if one exists, else a plain fact sentence — a
/// property perception or a tool asserted directly has no proof tree to
/// render, and narration must still produce *something* deterministic.
fn render_proof_or_fact(world: &World, case: Ref, property: Iri) -> String {
    if let Some(proof) = world.explain(case, property) {
        return proof.render(world);
    }
    match world.value(case, property) {
        Some(value) => format!(
            "{} {} = {}",
            world.resolve(case.iri()),
            world.resolve(property),
            value_text(world, value)
        ),
        None => format!(
            "{} {} is unassigned",
            world.resolve(case.iri()),
            world.resolve(property)
        ),
    }
}

fn value_text(world: &World, value: &Value) -> String {
    match value {
        Value::Ref(r) => world.resolve(r.iri()).to_owned(),
        other => other.lexical_form().unwrap_or_default(),
    }
}

/// Resolves a bare property local name (a perceived instance's JSON key,
/// e.g. `"heartRate"` — properties are named bare, matching CLAUDE.md's own
/// domain-term convention: "properties bare, classes prefixed") against
/// `reference`'s own namespace (everything up to and including its last `#`
/// or `/`) — the only namespace a perceived property can plausibly be in,
/// since [`derive_class_schema`] only ever emits property keys the schema
/// itself declared under that same class's namespace. `None` if nothing
/// declares the resulting IRI.
fn term_in_namespace_of(onto: &Ontology, reference: Iri, local_name: &str) -> Option<Iri> {
    let reference_text = onto.resolve(reference);
    let cut = reference_text.rfind(['#', '/'])? + 1;
    let candidate = format!("{}{}", &reference_text[..cut], local_name);
    onto.iri(&candidate)
}

/// Turns one JSON scalar from a perceived instance into a typed [`Value`],
/// per `property`'s declared range — `None` for a shape that doesn't parse
/// (silently dropped, matching perception's "never trusted blindly" posture
/// — the World gate is the real backstop, this is just type coercion).
fn json_to_value(json: &serde_json::Value, property: &PropertyDecl) -> Option<Value> {
    match property.range? {
        PropertyRange::Datatype(dt) => {
            let lexical = match json {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => return None,
            };
            Value::parse_typed(&lexical, dt).ok()
        }
        PropertyRange::Class(_) => {
            // A class-ranged property's proposed value is the local name
            // of a declared class/individual (doc06's enum-of-local-names
            // shape) — the caller resolves it to an Iri, not here (this
            // function has no Ontology handle); handled by the caller.
            None
        }
    }
}

/// Derives the JSON Schema a `PerceptionStep`'s `Bridge` call is forced
/// into, from `horis:extracts`, per doc 06's mapping table. The top-level
/// shape (`instances`/`confidence`) matches what `OpenRouterBridge` (U13)
/// already reads out of its forced tool call.
fn derive_extract_schema(onto: &Ontology, schema: &Schema, extracts: &[Iri]) -> serde_json::Value {
    let class_schemas: Vec<serde_json::Value> = extracts
        .iter()
        .map(|&class| derive_class_schema(onto, schema, class))
        .collect();
    serde_json::json!({
        "type": "object",
        "properties": {
            "instances": {
                "type": "array",
                "items": { "oneOf": class_schemas }
            },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
        },
        "required": ["instances", "confidence"]
    })
}

fn derive_class_schema(onto: &Ontology, schema: &Schema, class: Iri) -> serde_json::Value {
    let mut properties = serde_json::Map::new();
    let mut required = vec!["class".to_string()];
    properties.insert(
        "class".to_string(),
        serde_json::json!({ "const": onto.compact(onto.resolve(class)) }),
    );
    for decl in schema.properties() {
        if decl.domain != Some(class) {
            continue;
        }
        if decl.assigned_by == crate::ontology::schema::AssignedBy::RulesOnly {
            continue;
        }
        let single = derive_property_schema(onto, schema, decl);
        let multi_valued = decl.cardinality.is_some_and(|c| !c.is_single_valued());
        let final_schema = if multi_valued {
            serde_json::json!({ "type": "array", "items": single })
        } else {
            single
        };
        if decl.cardinality.is_some_and(Cardinality::requires_a_value) {
            required.push(onto.local_name(onto.resolve(decl.iri)));
        }
        properties.insert(onto.local_name(onto.resolve(decl.iri)), final_schema);
    }
    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

fn derive_property_schema(
    onto: &Ontology,
    schema: &Schema,
    decl: &PropertyDecl,
) -> serde_json::Value {
    let mut out = match decl.range {
        Some(PropertyRange::Datatype(Datatype::Decimal | Datatype::Integer)) => {
            let mut n = serde_json::json!({ "type": "number" });
            if let (Some(obj), Some(Value::Decimal(min))) = (n.as_object_mut(), &decl.min) {
                obj.insert(
                    "minimum".to_string(),
                    serde_json::json!(min.to_string().parse::<f64>().unwrap_or(0.0)),
                );
            } else if let (Some(obj), Some(Value::Integer(min))) = (n.as_object_mut(), &decl.min) {
                obj.insert("minimum".to_string(), serde_json::json!(min));
            }
            if let (Some(obj), Some(Value::Decimal(max))) = (n.as_object_mut(), &decl.max) {
                obj.insert(
                    "maximum".to_string(),
                    serde_json::json!(max.to_string().parse::<f64>().unwrap_or(0.0)),
                );
            } else if let (Some(obj), Some(Value::Integer(max))) = (n.as_object_mut(), &decl.max) {
                obj.insert("maximum".to_string(), serde_json::json!(max));
            }
            n
        }
        Some(PropertyRange::Datatype(Datatype::String)) => serde_json::json!({ "type": "string" }),
        Some(PropertyRange::Datatype(Datatype::Boolean)) => {
            serde_json::json!({ "type": "boolean" })
        }
        Some(PropertyRange::Datatype(Datatype::DateTime)) => {
            serde_json::json!({ "type": "string", "format": "date-time" })
        }
        Some(PropertyRange::Datatype(Datatype::Duration)) => {
            serde_json::json!({ "type": "string" })
        }
        Some(PropertyRange::Class(class)) => {
            if let Some(one_of) = schema.class(class).and_then(|c| c.one_of.as_ref()) {
                let names: Vec<String> = one_of
                    .iter()
                    .map(|&iri| onto.compact(onto.resolve(iri)))
                    .collect();
                serde_json::json!({ "type": "string", "enum": names })
            } else {
                let names: Vec<String> = schema
                    .subclasses(class)
                    .map(|iri| onto.compact(onto.resolve(iri)))
                    .collect();
                if names.is_empty() {
                    serde_json::json!({ "type": "string" })
                } else {
                    serde_json::json!({ "type": "string", "enum": names })
                }
            }
        }
        None => serde_json::json!({ "type": "string" }),
    };
    if let (Some(obj), Some(pattern)) = (out.as_object_mut(), &decl.pattern) {
        obj.insert("pattern".to_string(), serde_json::json!(pattern));
    }
    if let (Some(obj), Some(unit)) = (out.as_object_mut(), &decl.unit) {
        obj.insert(
            "description".to_string(),
            serde_json::json!(format!("value in {unit}")),
        );
    }
    out
}

/// Drives a `Future` to completion without an async runtime — matching
/// `bridge::mod`'s own test-double driver. Callers that need real async I/O
/// (`OpenRouterBridge`'s `reqwest` calls) drive `Agent::run` from their own
/// runtime instead (the demo binary, U14); this helper is for tests whose
/// `Bridge`/`Tool` never actually pend.
#[cfg(test)]
fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = pin!(fut);
    loop {
        if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
            return out;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::{BridgeError, NarrateResult, RankRequest, RankResult};

    struct StubBridge {
        id: String,
        confidence: f64,
        instances: Vec<serde_json::Value>,
    }

    #[async_trait::async_trait]
    impl Bridge for StubBridge {
        async fn perceive(&self, _req: PerceiveRequest) -> Result<PerceiveResult, BridgeError> {
            Ok(PerceiveResult {
                instances: self.instances.clone(),
                confidence: self.confidence,
                fact_confidence: BTreeMap::new(),
                accounting: Default::default(),
            })
        }
        async fn narrate(&self, _req: NarrateRequest) -> Result<NarrateResult, BridgeError> {
            Ok(NarrateResult {
                text: "narrated".to_owned(),
                accounting: Default::default(),
            })
        }
        async fn rank(&self, _req: RankRequest) -> Result<RankResult, BridgeError> {
            unimplemented!("not exercised by the DoD path")
        }
        fn id(&self) -> &str {
            &self.id
        }
    }

    /// A stub `Tool` that fills every `TypedPlaceholder` effect with a
    /// fixed string, keyed by property IRI — enough to satisfy any tool's
    /// declared contract without a real backend.
    struct StubTool(Vec<Iri>);
    impl Tool for StubTool {
        fn call(&self, _args: &BTreeMap<String, Value>) -> Result<BTreeMap<Iri, Value>, String> {
            Ok(self
                .0
                .iter()
                .map(|&p| (p, Value::String("stub".to_owned())))
                .collect())
        }
    }

    fn triage() -> Ontology {
        Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl must load")
    }

    /// U7 test target: a mock-bridge run of the full triage workflow
    /// reaches `:Done` with `acuity :Immediate`.
    #[test]
    fn the_immediate_track_reaches_done_with_acuity_immediate() {
        let onto = triage();

        let bridge = StubBridge {
            id: "mock:triage".to_owned(),
            confidence: 0.9,
            instances: vec![serde_json::json!({
                "class": ":Patient",
                "heartRate": 128,
                "hasSymptom": ":CrushingChestPain"
            })],
        };

        let agent = Agent::builder(&onto)
            .workflow(":Triage")
            .expect("Triage is declared")
            .bridge("claude", bridge)
            .tool(
                "assign_queue",
                StubTool(vec![
                    onto.node_ref(":queueRef").expect(":queueRef is declared"),
                ]),
            )
            .tool(
                "notify_nurse",
                StubTool(vec![
                    onto.node_ref(":pagedTo").expect(":pagedTo is declared"),
                ]),
            )
            .build()
            .expect("every binding is registered");

        let mut world = World::new(&onto);
        let now = chrono::DateTime::parse_from_rfc3339("2026-08-19T00:00:00Z")
            .expect("a fixed RFC3339 literal parses");
        let outcome = block_on(agent.run(
            &mut world,
            RunInput::new("58M, crushing chest pain, heart rate 128", now),
        ))
        .expect("the immediate track must complete");

        assert_eq!(
            onto.resolve(outcome.terminal),
            "https://example.org/triage#Done"
        );
        assert!(!outcome.fallback_taken);
        let case = outcome.case.expect("perception resolved a case subject");
        let acuity = onto.node_ref(":acuity").expect(":acuity is declared");
        assert_eq!(
            world.value(case, acuity),
            Some(&Value::Ref(Ref::new(
                onto.node_ref(":Immediate").expect(":Immediate is declared")
            )))
        );
    }
}
