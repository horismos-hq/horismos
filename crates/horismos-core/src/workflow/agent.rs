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
//! ## Trace emission (U9)
//!
//! Every executed Perception/Reasoning/Action/Narration step builds its
//! `trace::emit` `Step` inline (see each `execute_*` method) and `run`
//! accumulates them into `RunOutcome.trace`, a `horismos.trace/2` `Trace`
//! with `ontology`/`workflow`/`input`/`outcome` populated. Timing fields
//! (`latency_ms`, `Totals`) stay `None` — `horismos-core` has no clock
//! feature by design (CLAUDE.md), so a caller with one stamps them post-hoc.
//! A run touching a deferred kind (`Choice`/`Proposal`/`Wait`/`Subflow`)
//! ends in `Err` before a `Step` for it could exist.
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
use crate::workflow::deferred;
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
    /// This run's trace `run_id` (doc 09) — caller-supplied for the same
    /// reason `now` is: `horismos-core` mints neither randomness nor a
    /// clock reading by design (CLAUDE.md), so a unique id is the caller's
    /// job (a demo binary, `horismos-py`), and a fixed string in tests
    /// keeps trajectory-determinism assertions meaningful.
    pub run_id: String,
    /// The natural-language input the run's `PerceptionStep`s consume.
    pub text: String,
    /// The transaction-stamped `horis:now` (U11) every `ActionStep`'s
    /// `horis:now`-sentinel tool effect uses this run — read by the caller
    /// exactly once, at transaction open, never re-read inside chaining
    /// (CLAUDE.md's "no wall-clock ... in inference" rule, made concrete for
    /// tool effects). `horismos-core` has no clock feature to read one
    /// itself by design; a caller with one (a demo binary, `horismos-py`)
    /// supplies `chrono::Utc::now().into()` or a fixed instant in tests.
    /// Also stamps the trace's `started_at` (doc 09).
    pub now: chrono::DateTime<chrono::FixedOffset>,
}

impl RunInput {
    pub fn new(
        run_id: impl Into<String>,
        text: impl Into<String>,
        now: chrono::DateTime<chrono::FixedOffset>,
    ) -> Self {
        RunInput {
            run_id: run_id.into(),
            text: text.into(),
            now,
        }
    }
}

/// One `agent.run(...)` call's result: the terminal step reached, the
/// computed decision (D5), whether any step's fallback/error edge fired,
/// and the full trajectory — everything the caller needs without
/// re-deriving it from the final `World` state.
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
    /// The full trajectory (doc 09, U9): one `horismos.trace/2` `Trace`,
    /// one step per executed Perception/Reasoning/Action/Narration
    /// (`Choice`/`Proposal`/`Wait`/`Subflow` never execute in v1, so a
    /// run touching one ends in `Err` before a `Step` for it could exist).
    /// `trace.to_json()`/`to_json_pretty()` serialize it; the audit log and
    /// the training corpus are the same object (CLAUDE.md).
    pub trace: crate::trace::Trace,
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
        let mut goal_reached = true;

        let mut trace = crate::trace::Trace::new_v2(input.run_id.clone());
        trace.ontology = Some(crate::trace::schema::OntologyRef {
            iri: ontology_namespace(self.onto, self.workflow.iri),
            version: self.workflow.version.clone(),
            hash: None,
            allow_incomplete: self.onto.allows_incomplete(),
        });
        trace.workflow = Some(self.onto.compact(self.onto.resolve(self.workflow.iri)));
        trace.started_at = Some(input.now.to_rfc3339());
        trace.input = Some(crate::trace::schema::Input {
            kind: "nl".to_owned(),
            text: Some(input.text.clone()),
        });

        loop {
            // A bare `horis:Step` (`:Done`, `:PagingFailed`) is a pure
            // endpoint marker with nothing to execute — load-time validation
            // (`workflow::model`) guarantees it is also always terminal. A
            // terminal `ActionStep` (`:EscalateToNurse`) is the other
            // allowed terminal body (model.rs: "only horis:Step and
            // horis:ActionStep may be terminal in v1") specifically so its
            // action still fires — paging on the way out — before the run
            // ends, so it is dispatched below like any other step; the
            // `current.terminal` check after dispatch is what stops the run
            // from following its `horis:onError`/`horis:next` edge further.
            if matches!(current.body, StepBody::Bare) {
                return Ok(self.finish(
                    current.iri,
                    case,
                    fallback_taken,
                    goal_reached,
                    trace,
                    world,
                ));
            }

            let outcome = self.execute_step(world, &current, case, &input).await?;
            if outcome.is_fallback {
                fallback_taken = true;
                if matches!(current.body, StepBody::Reasoning(_)) {
                    goal_reached = false;
                }
            }
            if let Some(resolved_case) = outcome.case {
                case = Some(resolved_case);
            }
            if let Some(step) = outcome.trace_step {
                trace.steps.push(step);
            }
            if current.terminal {
                return Ok(self.finish(
                    current.iri,
                    case,
                    fallback_taken,
                    goal_reached,
                    trace,
                    world,
                ));
            }
            current = self
                .workflow
                .step(outcome.next)
                .expect("edge targets are validated at load time")
                .clone();
        }
    }

    /// Builds the `RunOutcome` for reaching terminal step `terminal` (D5's
    /// `horis:decides` lookup against the final case-subject state), and
    /// closes out `trace.outcome` (doc 09) with the same values.
    fn finish(
        &self,
        terminal: Iri,
        case: Option<Ref>,
        fallback_taken: bool,
        goal_reached: bool,
        mut trace: crate::trace::Trace,
        world: &World,
    ) -> RunOutcome {
        let decision = self.workflow.decides.and_then(|property| {
            case.and_then(|c| world.value(c, property))
                .cloned()
                .map(|v| (property, v))
        });
        let decision_fact = decision.as_ref().zip(case).map(|((property, value), c)| {
            crate::trace::schema::TraceFact {
                // `c` (the case subject) is very likely a runtime-minted
                // skolem instance, never in `self.onto`'s load-time-only
                // interner — resolve it through `world`'s (a strict
                // superset: it starts as `self.onto`'s and only ever grows).
                // `property` is safe through either, being load-time
                // declared, but `world` keeps both call sites uniform.
                s: world.ontology().compact(world.resolve(c.iri())),
                p: world.ontology().compact(world.resolve(*property)),
                o: crate::trace::emit::value_json(world, value),
                provenance: None,
                conf: None,
            }
        });
        trace.outcome = Some(crate::trace::schema::Outcome {
            terminal: Some(self.onto.compact(self.onto.resolve(terminal))),
            goal_reached: Some(goal_reached),
            decision: decision_fact,
            labels: BTreeMap::from([(
                "fallback_taken".to_owned(),
                serde_json::Value::Bool(fallback_taken),
            )]),
        });
        RunOutcome {
            terminal,
            decision,
            fallback_taken,
            case,
            trace,
        }
    }

    async fn execute_step(
        &self,
        world: &mut World,
        step: &Step,
        case: Option<Ref>,
        input: &RunInput,
    ) -> Result<StepOutcome, WorkflowError> {
        // Compacted CURIE — matches `workflow::deferred`'s convention
        // (domain-term errors and trace step names read the same way
        // whether a step executed or deferred).
        let step_name = self.onto.compact(self.onto.resolve(step.iri));
        match &step.body {
            StepBody::Perception(p) => self.execute_perception(world, &step_name, p, input).await,
            StepBody::Reasoning(r) => self.execute_reasoning(world, &step_name, case, r),
            StepBody::Action(a) => self.execute_action(world, &step_name, case, a, input.now),
            StepBody::Narration(n) => self.execute_narration(world, &step_name, case, n).await,
            // Never actually reached: `run()`'s loop short-circuits on a
            // Bare body before ever dispatching here (see its own doc
            // comment). Kept as an explicit, still-domain-term arm rather
            // than `unreachable!()` so a direct `execute_step` call (a
            // future test, say) fails safely instead of panicking.
            StepBody::Bare => Err(WorkflowError::StepDeferred {
                step: step_name,
                kind: "horis:Step (non-terminal, no body)".to_owned(),
            }),
            StepBody::Choice(_) => Err(deferred::choice_deferred(world, step.iri)),
            StepBody::Proposal(_) => Err(deferred::proposal_deferred(world, step.iri)),
            StepBody::Wait(_) => Err(deferred::wait_deferred(world, step.iri)),
            StepBody::Subflow(_) => Err(deferred::subflow_deferred(world, step.iri)),
        }
    }

    async fn execute_perception(
        &self,
        world: &mut World,
        step_name: &str,
        p: &PerceptionStep,
        input: &RunInput,
    ) -> Result<StepOutcome, WorkflowError> {
        let fallback = p
            .fallback
            .ok_or_else(|| WorkflowError::MissingFailureEdge {
                step: step_name.to_owned(),
                edge: "horis:fallback".to_owned(),
            })?;
        let bridge = self.bridge_for(p.model.as_deref())?;
        let schema = derive_extract_schema(self.onto, world.ontology().schema(), &p.extracts);
        let model = Some(bridge.id().to_owned());

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
                // A transport-level failure carries no confidence/instance
                // data worth recording — the fallback edge is the whole
                // trace-visible signal for this case.
                Err(_) => return Ok(self.fallback_outcome(fallback)),
            };
            let accepted = result.confidence >= p.confidence;
            if !accepted {
                let mut outcome = self.fallback_outcome(fallback);
                outcome.trace_step = Some(crate::trace::perception_step(
                    world,
                    Some(step_name),
                    model.clone(),
                    Some(result.confidence),
                    Some(p.confidence),
                    &[],
                ));
                return Ok(outcome);
            }
            match self.assert_perceived(world, &result) {
                Ok((primary, asserted)) => {
                    let mut outcome = StepOutcome::new(p.next, false, primary);
                    outcome.trace_step = Some(crate::trace::perception_step(
                        world,
                        Some(step_name),
                        model,
                        Some(result.confidence),
                        Some(p.confidence),
                        &asserted,
                    ));
                    return Ok(outcome);
                }
                Err(_) if attempt < p.retries => attempt += 1,
                Err(_) => return Ok(self.fallback_outcome(fallback)),
            }
        }
    }

    /// Asserts every proposed instance (provenance `perception`); returns
    /// the case subject (the first-extracted class's asserted instance, if
    /// any — see the module doc) plus every fact asserted, in interned form,
    /// for [`crate::trace::perception_step`].
    ///
    /// One `PerceiveResult` may carry several instances, each asserted
    /// through its own gate-checked `assert_instance_as` transaction. If a
    /// later instance is rejected, the whole batch must still read as one
    /// attempt to `execute_perception`'s retry loop: an earlier instance's
    /// commit is not left standing for a retry to pile onto or duplicate.
    /// Staging onto a cloned `World` (an O(1) structural-sharing clone, not
    /// a deep copy) and swapping it in only once every instance clears the
    /// gate gives that all-or-nothing behavior without adding new `World`
    /// retraction surface.
    fn assert_perceived(
        &self,
        world: &mut World,
        result: &PerceiveResult,
    ) -> Result<(Option<Ref>, Vec<crate::trace::PerceivedFact>), WorkflowError> {
        let mut staged = world.clone();
        let mut primary = None;
        let mut asserted_facts = Vec::new();
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
            let schema = staged.ontology().schema();
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
                // `derive_class_schema` asks for a JSON array wherever the
                // property is multi-valued, so a schema-conforming answer
                // carries one element per proposed value. Each element is
                // one candidate fact; cardinality stays the gate's call
                // (`assert_instance_as` below), never decided here.
                let candidates: Vec<&serde_json::Value> = match json_value {
                    serde_json::Value::Array(items) => items.iter().collect(),
                    scalar => vec![scalar],
                };
                for candidate in candidates {
                    let value = match decl.range {
                        Some(PropertyRange::Class(_)) => candidate
                            .as_str()
                            .and_then(|name| self.onto.node_ref(name).ok())
                            .map(|iri| Value::Ref(Ref::new(iri))),
                        _ => json_to_value(candidate, decl),
                    };
                    if let Some(value) = value {
                        properties.push((property, value));
                    }
                }
            }
            let fact_confidence = result
                .fact_confidence
                .get(&index)
                .copied()
                .unwrap_or(result.confidence);
            let asserted = staged
                .assert_instance_as(Provenance::Perception, None, class, properties.clone())
                .map_err(|e| WorkflowError::PerceptionRejected {
                    reason: e.domain_text(),
                })?;
            for (property, value) in properties {
                asserted_facts.push(crate::trace::PerceivedFact {
                    subject: asserted,
                    property,
                    value,
                    confidence: Some(fact_confidence),
                });
            }
            if primary.is_none() {
                primary = Some(asserted);
            }
        }
        *world = staged;
        Ok((primary, asserted_facts))
    }

    fn execute_reasoning(
        &self,
        world: &World,
        step_name: &str,
        case: Option<Ref>,
        r: &ReasoningStep,
    ) -> Result<StepOutcome, WorkflowError> {
        let case = case.ok_or(WorkflowError::NoCaseSubject)?;
        let reached = holds(world, case, &r.goal);
        // The goal check itself mutates nothing (doc 05: a ReasoningStep
        // asserts nothing), so there is no fresh `ReasoningRecord` of its
        // own — `world.last_reasoning()` is whatever the most recent
        // transaction (typically the preceding PerceptionStep's assert)
        // chained, which is exactly what made this goal reachable or not.
        // `goal_reached` isn't part of that record (it belongs to *this*
        // step, not the chaining pass), so it's stamped after the call —
        // `trace::emit`'s own documented convention for runtime-derived
        // fields.
        let trace_step = world.last_reasoning().map(|record| {
            let mut step = crate::trace::reasoning_step(world, Some(step_name), record);
            if let crate::trace::Step::Reasoning { goal_reached, .. } = &mut step {
                *goal_reached = Some(reached);
            }
            step
        });
        let mut outcome = if reached {
            StepOutcome::new(r.next, false, Some(case))
        } else {
            StepOutcome::new(
                r.on_unreachable
                    .ok_or_else(|| WorkflowError::MissingFailureEdge {
                        step: step_name.to_owned(),
                        edge: "horis:onUnreachable".to_owned(),
                    })?,
                true,
                Some(case),
            )
        };
        outcome.trace_step = trace_step;
        Ok(outcome)
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
                Some(edge) => Ok(StepOutcome::new(edge, true, Some(case))),
                None => Err(WorkflowError::RequiresUnsatisfiable {
                    step: step_name.to_owned(),
                }),
            };
        }

        let outcome = self.invoke_tool(world, case, a.tool, now);
        match outcome {
            Ok(effects) => {
                let next =
                    a.next
                        .or(a.on_error)
                        .ok_or_else(|| WorkflowError::MissingFailureEdge {
                            step: step_name.to_owned(),
                            edge: "horis:onError".to_owned(),
                        })?;
                let mut outcome = StepOutcome::new(next, false, Some(case));
                let effects: Vec<crate::trace::AssertedEffect> = effects
                    .into_iter()
                    .map(|(property, value)| crate::trace::AssertedEffect {
                        subject: case,
                        property,
                        value,
                    })
                    .collect();
                outcome.trace_step = Some(crate::trace::action_step(
                    world,
                    Some(step_name),
                    Some(&self.onto.compact(self.onto.resolve(a.tool))),
                    &effects,
                ));
                Ok(outcome)
            }
            // A rejected/failed invocation has no confirmed effect to
            // record — `horis:onError` is the whole trace-visible signal,
            // same posture as perception's transport-failure branch.
            Err(_) => Ok(StepOutcome::new(
                a.on_error
                    .ok_or_else(|| WorkflowError::MissingFailureEdge {
                        step: step_name.to_owned(),
                        edge: "horis:onError".to_owned(),
                    })?,
                true,
                Some(case),
            )),
        }
    }

    fn invoke_tool(
        &self,
        world: &mut World,
        case: Ref,
        tool_iri: Iri,
        now: chrono::DateTime<chrono::FixedOffset>,
    ) -> Result<Vec<(Iri, Value)>, ToolError> {
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
            .assert_on(Provenance::Tool, case, properties.clone())
            .map_err(|e| ToolError::InvocationFailed {
                tool: self.onto.resolve(tool_iri).to_owned(),
                reason: e.domain_text(),
            })?;
        Ok(properties)
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
        let text = match result {
            Ok(r) => r.text,
            Err(e) if n.required => {
                return Err(WorkflowError::NarrationFailed {
                    step: step_name.to_owned(),
                    reason: e.domain_text(),
                });
            }
            // D7: non-fatal by default — the mechanical proof.render()
            // fallback stands in, and it is still this step's recorded
            // output (emit.rs's own doc: "this builder only ever sees text
            // that exists, mechanical or modeled").
            Err(_) => proof_text,
        };
        let mut outcome = StepOutcome::new(n.next, false, Some(case));
        outcome.trace_step = Some(crate::trace::narration_step(
            Some(step_name),
            n.audience.clone(),
            Some(text),
        ));
        Ok(outcome)
    }

    fn fallback_outcome(&self, fallback: Iri) -> StepOutcome {
        StepOutcome::new(fallback, true, None)
    }
}

struct StepOutcome {
    next: Iri,
    is_fallback: bool,
    case: Option<Ref>,
    /// `Some(step)` for every step kind that actually executed (U9) —
    /// `None` for a fallback edge with no call worth recording (a
    /// transport-level Bridge error, a rejected tool invocation) and for
    /// the deferred/Bare kinds, which never reach a `StepOutcome` at all.
    trace_step: Option<crate::trace::Step>,
}

impl StepOutcome {
    fn new(next: Iri, is_fallback: bool, case: Option<Ref>) -> Self {
        StepOutcome {
            next,
            is_fallback,
            case,
            trace_step: None,
        }
    }
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

/// The workflow's own namespace (the part of its IRI before the last `#`
/// or `/`) — used as `trace.ontology.iri` (doc 09: `"…/triage"`).
/// `Ontology::load` takes a filesystem path, not a document IRI, so there
/// is no dedicated "ontology IRI" to read; the workflow's namespace is the
/// nearest available proxy (every triage.ttl term, workflow included,
/// shares one `@prefix : <...#>`).
fn ontology_namespace(onto: &Ontology, workflow_iri: Iri) -> String {
    let resolved = onto.resolve(workflow_iri);
    match resolved.rfind(['#', '/']) {
        Some(cut) => resolved[..cut].to_owned(),
        None => resolved.to_owned(),
    }
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
        let onto = Ontology::load_allow_incomplete(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("the explicit authoring posture still loads verified triage");

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
            RunInput::new(
                "run_immediate",
                "58M, crushing chest pain, heart rate 128",
                now,
            ),
        ))
        .expect("the immediate track must complete");

        assert_eq!(
            onto.resolve(outcome.terminal),
            "https://example.org/triage#Done"
        );
        assert!(!outcome.fallback_taken);
        let case = outcome.case.expect("perception resolved a case subject");
        let acuity = onto.node_ref(":acuity").expect(":acuity is declared");
        let immediate = onto.node_ref(":Immediate").expect(":Immediate is declared");
        assert_eq!(
            world.value(case, acuity),
            Some(&Value::Ref(Ref::new(immediate)))
        );
        // D5 runtime target: `:Triage`'s `horis:decides :acuity` resolves to
        // a concrete `RunOutcome.decision`, not just a value reachable on
        // the case subject — this is what was silently `None` before
        // `triage.ttl` gained the `horis:decides` declaration.
        assert_eq!(
            outcome.decision,
            Some((acuity, Value::Ref(Ref::new(immediate))))
        );
        // U10 test target (well-formed branch): `:Route`'s tool effect
        // (`:queueRef`, asserted by `assign_queue`) is still visible on the
        // case subject after later steps (`:Explain`, `:Done`) ran.
        let queue_ref = onto.node_ref(":queueRef").expect(":queueRef is declared");
        assert_eq!(
            world.value(case, queue_ref),
            Some(&Value::String("stub".to_owned()))
        );
        // U9 test target: the trajectory records one step per executed
        // Perception/Reasoning/Action/Narration, and round-trips exactly.
        assert_eq!(outcome.trace.steps.len(), 4, "{:?}", outcome.trace.steps);
        let narration = outcome
            .trace
            .steps
            .iter()
            .find(|s| matches!(s, crate::trace::Step::Narration { .. }))
            .expect("the trace records :Explain's narration");
        assert!(matches!(
            narration,
            crate::trace::Step::Narration { text: Some(t), .. } if t == "narrated"
        ));
        assert_eq!(outcome.trace.run_id, "run_immediate");
        let ontology_ref = outcome
            .trace
            .ontology
            .as_ref()
            .expect("agent traces identify their ontology");
        assert!(ontology_ref.allow_incomplete);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&outcome.trace.to_json())
                .expect("trace JSON is valid")["ontology"]["allow_incomplete"],
            true,
            "the explicit escape is durable trace evidence"
        );
        assert_eq!(
            crate::trace::Trace::from_json(&outcome.trace.to_json()).expect("trace round-trips"),
            outcome.trace
        );
    }

    /// A stub `Tool` that returns a wrong-typed value (an integer where the
    /// declared effect is `xsd:string`) for every property it is given —
    /// exercises U10's gate-rejection branch.
    struct WrongTypeTool(Vec<Iri>);
    impl Tool for WrongTypeTool {
        fn call(&self, _args: &BTreeMap<String, Value>) -> Result<BTreeMap<Iri, Value>, String> {
            Ok(self.0.iter().map(|&p| (p, Value::Integer(42))).collect())
        }
    }

    /// U10 test target (gate-rejected branch): an out-of-contract tool
    /// return — here, an integer where `:queueRef` declares `xsd:string` —
    /// is refused at the validation gate, so `:Route` follows its
    /// `horis:onError` edge to `:EscalateToNurse`. `:EscalateToNurse` is
    /// itself a terminal `ActionStep`: reaching it still fires
    /// `:paging.notify_nurse` (see `Agent::run`'s module doc) before the run
    /// ends, so the paging effect is visible too.
    #[test]
    fn a_gate_rejected_tool_effect_follows_on_error_and_still_pages_the_nurse() {
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
                WrongTypeTool(vec![
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
            RunInput::new(
                "run_rejected_effect",
                "58M, crushing chest pain, heart rate 128",
                now,
            ),
        ))
        .expect("the run completes via the onError edge, not a WorkflowError");

        assert_eq!(
            onto.resolve(outcome.terminal),
            "https://example.org/triage#EscalateToNurse"
        );
        assert!(outcome.fallback_taken);
        let case = outcome.case.expect("perception resolved a case subject");
        let queue_ref = onto.node_ref(":queueRef").expect(":queueRef is declared");
        assert_eq!(
            world.value(case, queue_ref),
            None,
            "the rejected assign_queue effect must never reach the world"
        );
        let paged_to = onto.node_ref(":pagedTo").expect(":pagedTo is declared");
        assert_eq!(
            world.value(case, paged_to),
            Some(&Value::String("stub".to_owned())),
            "the terminal ActionStep's own tool must still have fired"
        );
    }

    /// U10 test target (witness content): the rejected tool return's
    /// domain-term witness names the property and the datatype mismatch —
    /// calls `Agent::invoke_tool` directly (private, same-crate access) so
    /// the witness text isn't lost behind `execute_action`'s onError
    /// routing.
    #[test]
    fn a_gate_rejected_tool_effect_carries_a_domain_term_witness() {
        let onto = triage();
        let bridge = StubBridge {
            id: "mock:triage".to_owned(),
            confidence: 0.9,
            instances: vec![],
        };
        let agent = Agent::builder(&onto)
            .workflow(":Triage")
            .expect("Triage is declared")
            .bridge("claude", bridge)
            .tool(
                "assign_queue",
                WrongTypeTool(vec![
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
        // `:acuity` is `horis:assignedBy horis:RulesOnly` — set it the only
        // legitimate way (the tachycardia-with-chest-pain rule chaining off
        // an ordinary Api-provenance assertion), matching how a real run
        // reaches `:Route` rather than asserting it directly.
        let patient = onto.node_ref(":Patient").expect(":Patient is declared");
        let heart_rate = onto.node_ref(":heartRate").expect(":heartRate is declared");
        let has_symptom = onto
            .node_ref(":hasSymptom")
            .expect(":hasSymptom is declared");
        let crushing = onto
            .node_ref(":CrushingChestPain")
            .expect(":CrushingChestPain is declared");
        let case = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Decimal(rust_decimal::Decimal::from(128))),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("a well-formed Patient assertion succeeds, chaining acuity := Immediate");

        let now = chrono::DateTime::parse_from_rfc3339("2026-08-19T00:00:00Z")
            .expect("a fixed RFC3339 literal parses");
        let tool_iri = onto
            .node_ref(":ehr.assign_queue")
            .expect(":ehr.assign_queue is declared");
        let err = agent
            .invoke_tool(&mut world, case, tool_iri, now)
            .expect_err("an integer queueRef must be rejected by the gate");

        let text = err.domain_text();
        assert!(
            text.contains("queueRef"),
            "witness must name the offending property, got: {text}"
        );
    }

    /// Regression target: `assert_perceived` must treat one perception
    /// attempt's instances as a single all-or-nothing transaction. A second
    /// instance whose `heartRate` (1280) exceeds `:heartRate`'s
    /// `horis:max` (300) is gate-rejected — the first, well-formed
    /// instance's commit must not be left standing in `world` for a retry
    /// to compound on or duplicate.
    #[test]
    fn a_gate_rejected_instance_leaves_no_partial_perception_commit() {
        let onto = triage();
        let bridge = StubBridge {
            id: "mock:triage".to_owned(),
            confidence: 0.9,
            instances: vec![],
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
        let patient = onto.node_ref(":Patient").expect(":Patient is declared");
        let result = PerceiveResult {
            instances: vec![
                serde_json::json!({
                    "class": ":Patient",
                    "heartRate": 128,
                    "hasSymptom": ":CrushingChestPain"
                }),
                serde_json::json!({
                    "class": ":Patient",
                    "heartRate": 1280,
                    "hasSymptom": ":CrushingChestPain"
                }),
            ],
            confidence: 0.9,
            fact_confidence: BTreeMap::new(),
            accounting: Default::default(),
        };

        agent
            .assert_perceived(&mut world, &result)
            .expect_err("the second instance's out-of-range heartRate must be gate-rejected");
        assert_eq!(
            world.instances(patient).count(),
            0,
            "the first instance's commit must be rolled back when a later instance in the \
             same perception attempt is gate-rejected"
        );
    }

    /// Regression target: `derive_class_schema` asks the bridge for a JSON
    /// array wherever a property is multi-valued (`:hasSymptom`, `0..*`),
    /// so a schema-conforming answer is `"hasSymptom": [":CrushingChestPain"]`.
    /// `assert_perceived` used to resolve class-ranged values with
    /// `as_str()` only, silently dropping the array — `:Assess` was then
    /// unreachable on every live run and the loop took the `onUnreachable`
    /// escalation. The array shape must reach `:Done` exactly like the
    /// bare-string shape the mock bridge sends.
    #[test]
    fn the_immediate_track_reaches_done_when_perception_returns_an_array_of_symptoms() {
        let onto = triage();
        let bridge = StubBridge {
            id: "mock:triage".to_owned(),
            confidence: 0.9,
            instances: vec![serde_json::json!({
                "class": ":Patient",
                "heartRate": 128,
                "hasSymptom": [":CrushingChestPain"]
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
            RunInput::new(
                "run_immediate_array",
                "58M, crushing chest pain, heart rate 128",
                now,
            ),
        ))
        .expect("the immediate track must complete");

        assert_eq!(
            onto.resolve(outcome.terminal),
            "https://example.org/triage#Done",
            "an array-valued symptom must not leave :Assess unreachable"
        );
        assert!(!outcome.fallback_taken);
        let acuity = onto.node_ref(":acuity").expect(":acuity is declared");
        let immediate = onto.node_ref(":Immediate").expect(":Immediate is declared");
        assert_eq!(
            outcome.decision,
            Some((acuity, Value::Ref(Ref::new(immediate))))
        );
    }

    /// Every element of an array-valued perception answer is one proposed
    /// fact: all of them land on the subject and all of them are recorded
    /// as perceived facts in the trace. Cardinality is still the gate's
    /// call — nothing here bypasses it.
    #[test]
    fn an_array_valued_perception_asserts_every_element() {
        let onto = triage();
        let bridge = StubBridge {
            id: "mock:triage".to_owned(),
            confidence: 0.9,
            instances: vec![],
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
        let result = PerceiveResult {
            instances: vec![serde_json::json!({
                "class": ":Patient",
                "heartRate": 128,
                "hasSymptom": [":CrushingChestPain", ":ChestPain"]
            })],
            confidence: 0.9,
            fact_confidence: BTreeMap::new(),
            accounting: Default::default(),
        };

        let (primary, facts) = agent
            .assert_perceived(&mut world, &result)
            .expect("two declared symptoms in an array are a well-formed instance");
        let subject = primary.expect("the instance is the primary subject");
        let has_symptom = onto
            .node_ref(":hasSymptom")
            .expect(":hasSymptom is declared");
        let symptoms: Vec<&Value> = world.values(subject, has_symptom).collect();
        let expected = [
            Value::Ref(Ref::new(
                onto.node_ref(":ChestPain").expect(":ChestPain is declared"),
            )),
            Value::Ref(Ref::new(
                onto.node_ref(":CrushingChestPain")
                    .expect(":CrushingChestPain is declared"),
            )),
        ];
        assert_eq!(symptoms.len(), 2, "both symptoms must stand: {symptoms:?}");
        for value in &expected {
            assert!(
                symptoms.contains(&value),
                "{value:?} missing from {symptoms:?}"
            );
        }
        assert_eq!(
            facts.iter().filter(|f| f.property == has_symptom).count(),
            2,
            "both array elements must be recorded as perceived facts: {facts:?}"
        );
        assert_eq!(facts.len(), 3, "heartRate plus two symptoms: {facts:?}");
    }
}
