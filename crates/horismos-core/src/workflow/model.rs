//! The typed workflow model (U2): `horis:Workflow` and the eight
//! `horis:*Step` kinds, parsed from the ontology's graphs into data the
//! [`crate::workflow::agent::Agent`] interprets — a workflow is a graph of
//! steps, not a function, the same posture `reason::rule` (PRP-002) already
//! established for rules.
//!
//! Parsing happens once, at [`crate::Ontology`] assembly, after
//! [`crate::workflow::tool::parse_tools`] (U3) — `ActionStep`'s
//! `horis:tool` and `SubflowStep`'s `horis:workflow` are cross-validated
//! against what was actually declared, so a dangling edge is a *load*
//! error, never a runtime surprise.
//!
//! Every step kind parses (the ontology stays a generic graph of typed step
//! data); execution is a separate question the interpreter answers — see
//! `workflow::agent` and `PRP-004-agent-loop/02-tasks-and-gaps.md` D3 for
//! `WaitStep`, which parses here but is not executed in v1.
//!
//! Spec: `docs/framework/05-workflows.md`, `06-llm-boundaries.md` ·
//! Owner: PRP-004

use std::collections::{BTreeMap, BTreeSet};

use crate::error::OntologyError;
use crate::ontology::schema::Schema;
use crate::ontology::term::{Interner, Iri};
use crate::ontology::turtle::{Document, Literal, Node, Object, Triple};
use crate::reason::rule::{
    Condition, ConditionTree, MAX_CONDITION_DEPTH, Op, SubjectGraph, node_iri, object_text,
    term_operand,
};
use crate::workflow::tool::Tool;

// -- Vocabulary this module reads directly -------------------------------

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const HORIS_WORKFLOW: &str = "https://w3id.org/horismos#Workflow";
const HORIS_STARTS_AT: &str = "https://w3id.org/horismos#startsAt";
const HORIS_VERSION: &str = "https://w3id.org/horismos#version";
const HORIS_DECIDES: &str = "https://w3id.org/horismos#decides";

const HORIS_PERCEPTION_STEP: &str = "https://w3id.org/horismos#PerceptionStep";
const HORIS_REASONING_STEP: &str = "https://w3id.org/horismos#ReasoningStep";
const HORIS_ACTION_STEP: &str = "https://w3id.org/horismos#ActionStep";
const HORIS_NARRATION_STEP: &str = "https://w3id.org/horismos#NarrationStep";
const HORIS_PROPOSAL_STEP: &str = "https://w3id.org/horismos#ProposalStep";
const HORIS_WAIT_STEP: &str = "https://w3id.org/horismos#WaitStep";
const HORIS_CHOICE_STEP: &str = "https://w3id.org/horismos#ChoiceStep";
const HORIS_SUBFLOW_STEP: &str = "https://w3id.org/horismos#SubflowStep";
const HORIS_STEP: &str = "https://w3id.org/horismos#Step";

const HORIS_NEXT: &str = "https://w3id.org/horismos#next";
const HORIS_GOAL: &str = "https://w3id.org/horismos#goal";
const HORIS_REQUIRES: &str = "https://w3id.org/horismos#requires";
const HORIS_FALLBACK: &str = "https://w3id.org/horismos#fallback";
const HORIS_TERMINAL: &str = "https://w3id.org/horismos#terminal";
const HORIS_ON_UNREACHABLE: &str = "https://w3id.org/horismos#onUnreachable";
const HORIS_ON_ERROR: &str = "https://w3id.org/horismos#onError";
const HORIS_ON_UNSATISFIED: &str = "https://w3id.org/horismos#onUnsatisfied";
const HORIS_EXTRACTS: &str = "https://w3id.org/horismos#extracts";
const HORIS_MODEL: &str = "https://w3id.org/horismos#model";
const HORIS_CONFIDENCE: &str = "https://w3id.org/horismos#confidence";
const HORIS_TOOL: &str = "https://w3id.org/horismos#tool";
const HORIS_AUDIENCE: &str = "https://w3id.org/horismos#audience";
const HORIS_NARRATES: &str = "https://w3id.org/horismos#narrates";
const HORIS_REQUIRED: &str = "https://w3id.org/horismos#required";
const HORIS_BRANCHES: &str = "https://w3id.org/horismos#branches";
const HORIS_OTHERWISE: &str = "https://w3id.org/horismos#otherwise";
const HORIS_WHEN: &str = "https://w3id.org/horismos#when";
const HORIS_WORKFLOW_PROP: &str = "https://w3id.org/horismos#workflow";
const HORIS_CANDIDATES: &str = "https://w3id.org/horismos#candidates";
const HORIS_RANK_BY: &str = "https://w3id.org/horismos#rankBy";
const HORIS_RETRIES: &str = "https://w3id.org/horismos#retries";
const HORIS_PROPERTY: &str = "https://w3id.org/horismos#property";
const HORIS_SUBJECT: &str = "https://w3id.org/horismos#subject";

const HORIS_ALL: &str = "https://w3id.org/horismos#all";
const HORIS_ANY: &str = "https://w3id.org/horismos#any";
const HORIS_NONE: &str = "https://w3id.org/horismos#none";
const HORIS_EXCEEDS: &str = "https://w3id.org/horismos#exceeds";
const HORIS_BELOW: &str = "https://w3id.org/horismos#below";
const HORIS_AT_LEAST: &str = "https://w3id.org/horismos#atLeast";
const HORIS_AT_MOST: &str = "https://w3id.org/horismos#atMost";
const HORIS_BETWEEN: &str = "https://w3id.org/horismos#between";
const HORIS_IS: &str = "https://w3id.org/horismos#is";
const HORIS_IS_NOT: &str = "https://w3id.org/horismos#isNot";
const HORIS_IS_ASSIGNED: &str = "https://w3id.org/horismos#isAssigned";
const HORIS_IS_EMPTY: &str = "https://w3id.org/horismos#isEmpty";
const HORIS_CONTAINS: &str = "https://w3id.org/horismos#contains";
const HORIS_IN_LIST: &str = "https://w3id.org/horismos#inList";

// -- Public model ------------------------------------------------------------

/// One parsed `horis:Workflow`.
#[derive(Debug, Clone)]
pub struct Workflow {
    pub iri: Iri,
    pub starts_at: Iri,
    pub version: Option<String>,
    /// D5 (decided 2026-08-18): the single decision property, if declared.
    pub decides: Option<Iri>,
    /// Every step this workflow declares, keyed by step IRI.
    pub steps: BTreeMap<Iri, Step>,
}

impl Workflow {
    pub fn step(&self, iri: Iri) -> Option<&Step> {
        self.steps.get(&iri)
    }
}

/// One parsed step: the fields every kind shares, plus its kind-specific
/// body.
#[derive(Debug, Clone)]
pub struct Step {
    pub iri: Iri,
    pub terminal: bool,
    pub body: StepBody,
}

/// The eight step kinds (doc 05), plus the untyped bare `horis:Step`
/// terminal marker (`:Done a horis:Step ; horis:terminal true .`).
#[derive(Debug, Clone)]
pub enum StepBody {
    Bare,
    Perception(PerceptionStep),
    Reasoning(ReasoningStep),
    Action(ActionStep),
    Narration(NarrationStep),
    Proposal(ProposalStep),
    Choice(ChoiceStep),
    Wait(WaitStep),
    Subflow(SubflowStep),
}

#[derive(Debug, Clone)]
pub struct PerceptionStep {
    pub extracts: Vec<Iri>,
    pub model: Option<String>,
    pub confidence: f64,
    pub fallback: Option<Iri>,
    pub next: Iri,
    pub retries: u32,
}

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub goal: ConditionTree,
    pub on_unreachable: Option<Iri>,
    pub next: Iri,
}

#[derive(Debug, Clone)]
pub struct ActionStep {
    pub tool: Iri,
    pub requires: Option<ConditionTree>,
    pub on_error: Option<Iri>,
    pub on_unsatisfied: Option<Iri>,
    /// `None` when this step is terminal (doc 05's `:EscalateToNurse`).
    pub next: Option<Iri>,
}

#[derive(Debug, Clone)]
pub struct NarrationStep {
    pub narrates: Iri,
    pub audience: Option<String>,
    pub model: Option<String>,
    /// D7 (decided 2026-08-18): non-fatal by default; `true` makes a
    /// narration failure propagate instead of falling back to
    /// `proof.render()`.
    pub required: bool,
    pub next: Iri,
}

#[derive(Debug, Clone)]
pub struct ProposalStep {
    /// The candidate individuals ranked over, in declaration order. Doc 07
    /// illustrates a graph-derived `horis:query`; this build takes a
    /// simpler, decided shape — a plain declared list — pending a concrete
    /// query syntax (recorded as an implementation note, not a `[DECIDE]`:
    /// no framework marker asks for one).
    pub candidates: Vec<Iri>,
    pub rank_by: Option<String>,
    pub model: Option<String>,
    pub confidence: f64,
    pub fallback: Option<Iri>,
    pub next: Iri,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub when: ConditionTree,
    pub next: Iri,
}

#[derive(Debug, Clone)]
pub struct ChoiceStep {
    pub branches: Vec<Branch>,
    pub otherwise: Option<Iri>,
}

#[derive(Debug, Clone)]
pub struct WaitStep {
    /// Parsed for profile-completeness; execution is deferred (D3,
    /// `PRP-004-agent-loop/02-tasks-and-gaps.md`) — `workflow::agent` never
    /// produces a run that reaches one.
    pub next: Option<Iri>,
}

#[derive(Debug, Clone)]
pub struct SubflowStep {
    pub workflow: Iri,
    /// The parent's continuation once the subflow reaches any terminal
    /// (D6's simplest reading — doc 05 does not show a worked example).
    pub next: Iri,
}

// -- Parsing ------------------------------------------------------------

/// Parses every `horis:Workflow` across `documents`. `tools` is U3's
/// already-parsed `horis:Tool` set — `ActionStep.tool` and
/// `SubflowStep.workflow` (against the *other* parsed workflows, checked in
/// a second pass below) are cross-validated against it.
pub fn parse_workflows(
    documents: &[&Document],
    schema: &Schema,
    tools: &BTreeMap<Iri, Tool>,
    interner: &Interner,
) -> Result<(BTreeMap<Iri, Workflow>, Vec<Iri>), OntologyError> {
    let graph = SubjectGraph::collect(documents.iter().copied(), interner);

    let mut workflow_ids = Vec::new();
    let mut step_ids: BTreeMap<Iri, Vec<&'static str>> = BTreeMap::new();
    let step_types: &[(&str, &'static str)] = &[
        (HORIS_PERCEPTION_STEP, "PerceptionStep"),
        (HORIS_REASONING_STEP, "ReasoningStep"),
        (HORIS_ACTION_STEP, "ActionStep"),
        (HORIS_NARRATION_STEP, "NarrationStep"),
        (HORIS_PROPOSAL_STEP, "ProposalStep"),
        (HORIS_WAIT_STEP, "WaitStep"),
        (HORIS_CHOICE_STEP, "ChoiceStep"),
        (HORIS_SUBFLOW_STEP, "SubflowStep"),
        (HORIS_STEP, "Step"),
    ];

    for document in documents {
        for Triple {
            subject,
            predicate,
            object,
        } in &document.triples
        {
            if interner.resolve(*predicate) != RDF_TYPE {
                continue;
            }
            let Object::Node(type_node) = object else {
                continue;
            };
            let type_str = interner.resolve(node_iri(*type_node));
            if type_str == HORIS_WORKFLOW {
                if matches!(subject, Node::Blank(_)) {
                    return Err(OntologyError::WorkflowAnonymous {
                        kind: "horis:Workflow".to_owned(),
                    });
                }
                let iri = node_iri(*subject);
                if !workflow_ids.contains(&iri) {
                    workflow_ids.push(iri);
                }
            }
            if let Some((_, kind)) = step_types.iter().find(|(t, _)| *t == type_str) {
                if matches!(subject, Node::Blank(_)) {
                    return Err(OntologyError::WorkflowAnonymous {
                        kind: format!("horis:{kind}"),
                    });
                }
                let iri = node_iri(*subject);
                let kinds = step_ids.entry(iri).or_default();
                if !kinds.contains(kind) {
                    kinds.push(kind);
                }
            }
        }
    }
    workflow_ids.sort_by_key(|&iri| interner.resolve(iri).to_owned());

    for (&iri, kinds) in &step_ids {
        // `horis:Step` co-declared with a concrete kind is fine (every
        // typed step *is* a `horis:Step`); two concrete kinds on one
        // subject is not.
        let concrete: Vec<&&str> = kinds.iter().filter(|k| **k != "Step").collect();
        if concrete.len() > 1 {
            return Err(OntologyError::StepAmbiguousKind {
                step: interner.resolve(iri).to_owned(),
                a: concrete[0].to_string(),
                b: concrete[1].to_string(),
            });
        }
    }

    let mut workflows = BTreeMap::new();
    for &wf_iri in &workflow_ids {
        let workflow = parse_one_workflow(wf_iri, &graph, &step_ids, schema, tools, interner)?;
        workflows.insert(wf_iri, workflow);
    }

    let claimed: BTreeSet<Iri> = workflows
        .values()
        .flat_map(|workflow| workflow.steps.keys().copied())
        .collect();
    let orphaned: Vec<Iri> = step_ids
        .keys()
        .filter(|step| !claimed.contains(step))
        .copied()
        .collect();
    // Second pass: SubflowStep.workflow must name a *different* declared
    // workflow whose own steps contain no SubflowStep (D6 — depth capped at
    // one, checked here since it needs every workflow parsed first).
    for workflow in workflows.values() {
        for step in workflow.steps.values() {
            let StepBody::Subflow(subflow) = &step.body else {
                continue;
            };
            let Some(target) = workflows.get(&subflow.workflow) else {
                return Err(OntologyError::WorkflowUndeclaredStep {
                    subject: interner.resolve(step.iri).to_owned(),
                    edge: "horis:workflow".to_owned(),
                    target: interner.resolve(subflow.workflow).to_owned(),
                });
            };
            let nests_further = target
                .steps
                .values()
                .any(|s| matches!(s.body, StepBody::Subflow(_)));
            if nests_further {
                return Err(OntologyError::SubflowNestingTooDeep {
                    step: interner.resolve(step.iri).to_owned(),
                    target: interner.resolve(subflow.workflow).to_owned(),
                });
            }
        }
    }

    Ok((workflows, orphaned))
}

fn parse_one_workflow(
    wf_iri: Iri,
    graph: &SubjectGraph,
    step_ids: &BTreeMap<Iri, Vec<&'static str>>,
    schema: &Schema,
    tools: &BTreeMap<Iri, Tool>,
    interner: &Interner,
) -> Result<Workflow, OntologyError> {
    let name = || interner.resolve(wf_iri).to_owned();

    let starts_at = match graph.object(wf_iri, HORIS_STARTS_AT, interner) {
        Some(Object::Node(n)) => node_iri(*n),
        _ => {
            return Err(OntologyError::WorkflowMissingPart {
                subject: name(),
                part: "horis:startsAt".to_owned(),
            });
        }
    };

    let version = match graph.object(wf_iri, HORIS_VERSION, interner) {
        Some(Object::Literal(Literal::Typed(crate::ontology::term::Value::String(s)))) => {
            Some(s.clone())
        }
        _ => None,
    };

    let decides = match graph.object(wf_iri, HORIS_DECIDES, interner) {
        Some(Object::Node(n)) => {
            let property = node_iri(*n);
            if schema.property(property).is_none() {
                return Err(OntologyError::StepUndeclaredProperty {
                    subject: format!("{} horis:decides", name()),
                    property: interner.resolve(property).to_owned(),
                });
            }
            Some(property)
        }
        _ => None,
    };

    let mut steps = BTreeMap::new();
    for &step_iri in step_ids.keys() {
        let step = parse_step(step_iri, graph, step_ids, schema, tools, interner)?;
        steps.insert(step_iri, step);
    }

    // Every edge that names a step must resolve within this workflow's
    // declared steps — a dangling edge is a load error, never a runtime
    // surprise. Validated against every step declared in the document
    // (`step_ids` is shared across all workflows here), so a stray
    // unreachable step with its own dangling edge is still caught.
    check_step_ref(starts_at, &steps, &name(), "horis:startsAt", interner)?;
    for step in steps.values() {
        check_step_edges(step, &steps, interner)?;
    }

    // `steps` above is scratch space keyed by every step *declared in the
    // document*, not this workflow's own — a document naming more than one
    // `horis:Workflow` would otherwise leave every workflow holding the
    // union of all of them (bug: this previously made D6's "does the
    // subflow target nest further" check fire on *any* `SubflowStep`
    // anywhere in a multi-workflow document, since the target's polluted
    // map always contained one). Scope the returned `Workflow.steps` to
    // what `starts_at` actually reaches, matching this struct's own doc
    // ("every step *this workflow* declares").
    let reachable = reachable_steps(starts_at, &steps_snapshot_edges(&steps));
    steps.retain(|iri, _| reachable.contains(iri));

    Ok(Workflow {
        iri: wf_iri,
        starts_at,
        version,
        decides,
        steps,
    })
}

/// The step IRIs `starts_at` can reach by following the same edges
/// [`check_step_edges`] validates (`next`/`fallback`/`onError`/
/// `onUnreachable`/`onUnsatisfied`/branch `next`/`otherwise`).
fn reachable_steps(starts_at: Iri, edges: &BTreeMap<Iri, Vec<Iri>>) -> BTreeSet<Iri> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![starts_at];
    while let Some(iri) = stack.pop() {
        if !seen.insert(iri) {
            continue;
        }
        if let Some(targets) = edges.get(&iri) {
            stack.extend(targets.iter().copied());
        }
    }
    seen
}

/// Each step's outgoing edges, for [`reachable_steps`]'s BFS — the same
/// edge set [`check_step_edges`] walks, just collected rather than
/// validated.
fn steps_snapshot_edges(steps: &BTreeMap<Iri, Step>) -> BTreeMap<Iri, Vec<Iri>> {
    steps
        .iter()
        .map(|(&iri, step)| {
            let targets = match &step.body {
                StepBody::Bare => vec![],
                StepBody::Perception(p) => p.fallback.into_iter().chain([p.next]).collect(),
                StepBody::Reasoning(r) => r.on_unreachable.into_iter().chain([r.next]).collect(),
                StepBody::Action(a) => [a.on_error, a.on_unsatisfied, a.next]
                    .into_iter()
                    .flatten()
                    .collect(),
                StepBody::Narration(n) => vec![n.next],
                StepBody::Proposal(p) => p.fallback.into_iter().chain([p.next]).collect(),
                StepBody::Choice(c) => c
                    .branches
                    .iter()
                    .map(|b| b.next)
                    .chain(c.otherwise)
                    .collect(),
                StepBody::Wait(w) => w.next.into_iter().collect(),
                StepBody::Subflow(s) => vec![s.next],
            };
            (iri, targets)
        })
        .collect()
}

fn check_step_ref(
    target: Iri,
    steps: &BTreeMap<Iri, Step>,
    subject: &str,
    edge: &str,
    interner: &Interner,
) -> Result<(), OntologyError> {
    if steps.contains_key(&target) {
        Ok(())
    } else {
        Err(OntologyError::WorkflowUndeclaredStep {
            subject: subject.to_owned(),
            edge: edge.to_owned(),
            target: interner.resolve(target).to_owned(),
        })
    }
}

fn check_step_edges(
    step: &Step,
    steps: &BTreeMap<Iri, Step>,
    interner: &Interner,
) -> Result<(), OntologyError> {
    let subject = interner.resolve(step.iri).to_owned();
    let check = |target: Iri, edge: &str| check_step_ref(target, steps, &subject, edge, interner);
    match &step.body {
        StepBody::Bare => {}
        StepBody::Perception(p) => {
            if let Some(fallback) = p.fallback {
                check(fallback, "horis:fallback")?;
            }
            check(p.next, "horis:next")?;
        }
        StepBody::Reasoning(r) => {
            if let Some(on_unreachable) = r.on_unreachable {
                check(on_unreachable, "horis:onUnreachable")?;
            }
            check(r.next, "horis:next")?;
        }
        StepBody::Action(a) => {
            if let Some(on_error) = a.on_error {
                check(on_error, "horis:onError")?;
            }
            if let Some(u) = a.on_unsatisfied {
                check(u, "horis:onUnsatisfied")?;
            }
            if let Some(n) = a.next {
                check(n, "horis:next")?;
            }
        }
        StepBody::Narration(n) => {
            check(n.next, "horis:next")?;
        }
        StepBody::Proposal(p) => {
            if let Some(fallback) = p.fallback {
                check(fallback, "horis:fallback")?;
            }
            check(p.next, "horis:next")?;
        }
        StepBody::Choice(c) => {
            for b in &c.branches {
                check(b.next, "horis:next (branch)")?;
            }
            if let Some(o) = c.otherwise {
                check(o, "horis:otherwise")?;
            }
        }
        StepBody::Wait(w) => {
            if let Some(n) = w.next {
                check(n, "horis:next")?;
            }
        }
        StepBody::Subflow(s) => {
            check(s.next, "horis:next")?;
        }
    }
    Ok(())
}

fn parse_step(
    iri: Iri,
    graph: &SubjectGraph,
    step_ids: &BTreeMap<Iri, Vec<&'static str>>,
    schema: &Schema,
    tools: &BTreeMap<Iri, Tool>,
    interner: &Interner,
) -> Result<Step, OntologyError> {
    let name = || interner.resolve(iri).to_owned();
    let terminal = matches!(
        graph.object(iri, HORIS_TERMINAL, interner),
        Some(Object::Literal(Literal::Typed(
            crate::ontology::term::Value::Boolean(true)
        )))
    );

    let kinds = step_ids.get(&iri).cloned().unwrap_or_default();
    let concrete = kinds.iter().find(|k| **k != "Step").copied();

    let body = match concrete {
        None => StepBody::Bare,
        Some("PerceptionStep") => {
            StepBody::Perception(parse_perception(iri, graph, schema, interner)?)
        }
        Some("ReasoningStep") => {
            StepBody::Reasoning(parse_reasoning(iri, graph, schema, interner)?)
        }
        Some("ActionStep") => StepBody::Action(parse_action(iri, graph, schema, tools, interner)?),
        Some("NarrationStep") => {
            StepBody::Narration(parse_narration(iri, graph, schema, interner)?)
        }
        Some("ProposalStep") => StepBody::Proposal(parse_proposal(iri, graph, interner)?),
        Some("ChoiceStep") => StepBody::Choice(parse_choice(iri, graph, schema, interner)?),
        Some("WaitStep") => StepBody::Wait(parse_wait(iri, graph, interner)),
        Some("SubflowStep") => StepBody::Subflow(parse_subflow(iri, graph, interner)?),
        Some(other) => unreachable!("step_types only ever records the eight kinds; got {other}"),
    };

    if terminal && !matches!(body, StepBody::Bare | StepBody::Action(_)) {
        return Err(OntologyError::WorkflowMissingPart {
            subject: name(),
            part: "horis:next (only horis:Step and horis:ActionStep may be terminal in v1)"
                .to_owned(),
        });
    }

    Ok(Step {
        iri,
        terminal,
        body,
    })
}

fn required_iri(
    iri: Iri,
    graph: &SubjectGraph,
    predicate: &str,
    part: &str,
    interner: &Interner,
) -> Result<Iri, OntologyError> {
    match graph.object(iri, predicate, interner) {
        Some(Object::Node(n)) => Ok(node_iri(*n)),
        _ => Err(OntologyError::WorkflowMissingPart {
            subject: interner.resolve(iri).to_owned(),
            part: part.to_owned(),
        }),
    }
}

fn optional_iri(
    iri: Iri,
    graph: &SubjectGraph,
    predicate: &str,
    interner: &Interner,
) -> Option<Iri> {
    match graph.object(iri, predicate, interner) {
        Some(Object::Node(n)) => Some(node_iri(*n)),
        _ => None,
    }
}

fn optional_string(
    iri: Iri,
    graph: &SubjectGraph,
    predicate: &str,
    interner: &Interner,
) -> Option<String> {
    match graph.object(iri, predicate, interner) {
        Some(Object::Literal(Literal::Typed(crate::ontology::term::Value::String(s)))) => {
            Some(s.clone())
        }
        _ => None,
    }
}

fn required_confidence(
    iri: Iri,
    graph: &SubjectGraph,
    interner: &Interner,
) -> Result<f64, OntologyError> {
    match graph.object(iri, HORIS_CONFIDENCE, interner) {
        Some(object) => {
            let text = object_text(object, interner);
            let value: f64 = text
                .parse()
                .map_err(|_| OntologyError::StepInvalidConfidence {
                    step: interner.resolve(iri).to_owned(),
                    value: text.clone(),
                })?;
            if !(0.0..=1.0).contains(&value) {
                return Err(OntologyError::StepInvalidConfidence {
                    step: interner.resolve(iri).to_owned(),
                    value: text,
                });
            }
            Ok(value)
        }
        None => Err(OntologyError::WorkflowMissingPart {
            subject: interner.resolve(iri).to_owned(),
            part: "horis:confidence".to_owned(),
        }),
    }
}

fn parse_perception(
    iri: Iri,
    graph: &SubjectGraph,
    schema: &Schema,
    interner: &Interner,
) -> Result<PerceptionStep, OntologyError> {
    let mut extracts = Vec::new();
    for (predicate, object) in graph.pairs(iri) {
        if interner.resolve(*predicate) != HORIS_EXTRACTS {
            continue;
        }
        let Object::Node(n) = object else { continue };
        let class = node_iri(*n);
        if schema.class_declared(class).is_none() {
            return Err(OntologyError::StepUndeclaredClass {
                step: interner.resolve(iri).to_owned(),
                class: interner.resolve(class).to_owned(),
            });
        }
        extracts.push(class);
    }
    if extracts.is_empty() {
        return Err(OntologyError::WorkflowMissingPart {
            subject: interner.resolve(iri).to_owned(),
            part: "horis:extracts".to_owned(),
        });
    }

    let retries = match graph.object(iri, HORIS_RETRIES, interner) {
        Some(Object::Literal(Literal::Typed(crate::ontology::term::Value::Integer(n)))) => {
            u32::try_from(*n).unwrap_or(0)
        }
        _ => 2, // doc 06's documented default
    };

    Ok(PerceptionStep {
        extracts,
        model: optional_string(iri, graph, HORIS_MODEL, interner),
        confidence: required_confidence(iri, graph, interner)?,
        fallback: optional_iri(iri, graph, HORIS_FALLBACK, interner),
        next: required_iri(iri, graph, HORIS_NEXT, "horis:next", interner)?,
        retries,
    })
}

fn parse_reasoning(
    iri: Iri,
    graph: &SubjectGraph,
    schema: &Schema,
    interner: &Interner,
) -> Result<ReasoningStep, OntologyError> {
    let goal_node = match graph.object(iri, HORIS_GOAL, interner) {
        Some(Object::Node(n)) => *n,
        _ => {
            return Err(OntologyError::WorkflowMissingPart {
                subject: interner.resolve(iri).to_owned(),
                part: "horis:goal".to_owned(),
            });
        }
    };
    let goal = parse_condition_tree(
        goal_node,
        graph,
        schema,
        interner,
        &format!("{} horis:goal", interner.resolve(iri)),
        &mut BTreeSet::new(),
        0,
    )?;
    Ok(ReasoningStep {
        goal,
        on_unreachable: optional_iri(iri, graph, HORIS_ON_UNREACHABLE, interner),
        next: required_iri(iri, graph, HORIS_NEXT, "horis:next", interner)?,
    })
}

fn parse_action(
    iri: Iri,
    graph: &SubjectGraph,
    schema: &Schema,
    tools: &BTreeMap<Iri, Tool>,
    interner: &Interner,
) -> Result<ActionStep, OntologyError> {
    let tool = required_iri(iri, graph, HORIS_TOOL, "horis:tool", interner)?;
    if !tools.contains_key(&tool) {
        return Err(OntologyError::WorkflowUndeclaredStep {
            subject: interner.resolve(iri).to_owned(),
            edge: "horis:tool".to_owned(),
            target: interner.resolve(tool).to_owned(),
        });
    }

    let requires = match graph.object(iri, HORIS_REQUIRES, interner) {
        Some(Object::Node(n)) => Some(parse_condition_tree(
            *n,
            graph,
            schema,
            interner,
            &format!("{} horis:requires", interner.resolve(iri)),
            &mut BTreeSet::new(),
            0,
        )?),
        _ => None,
    };

    Ok(ActionStep {
        tool,
        requires,
        on_error: optional_iri(iri, graph, HORIS_ON_ERROR, interner),
        on_unsatisfied: optional_iri(iri, graph, HORIS_ON_UNSATISFIED, interner),
        next: optional_iri(iri, graph, HORIS_NEXT, interner),
    })
}

fn parse_narration(
    iri: Iri,
    graph: &SubjectGraph,
    schema: &Schema,
    interner: &Interner,
) -> Result<NarrationStep, OntologyError> {
    let narrates_node = match graph.object(iri, HORIS_NARRATES, interner) {
        Some(Object::Node(n)) => *n,
        _ => {
            return Err(OntologyError::WorkflowMissingPart {
                subject: interner.resolve(iri).to_owned(),
                part: "horis:narrates".to_owned(),
            });
        }
    };
    let narrates = match graph.object(node_iri(narrates_node), HORIS_PROPERTY, interner) {
        Some(Object::Node(n)) => node_iri(*n),
        _ => {
            return Err(OntologyError::WorkflowMissingPart {
                subject: interner.resolve(iri).to_owned(),
                part: "horis:property on its horis:narrates".to_owned(),
            });
        }
    };
    if schema.property(narrates).is_none() {
        return Err(OntologyError::StepUndeclaredProperty {
            subject: format!("{} horis:narrates", interner.resolve(iri)),
            property: interner.resolve(narrates).to_owned(),
        });
    }

    let required = matches!(
        graph.object(iri, HORIS_REQUIRED, interner),
        Some(Object::Literal(Literal::Typed(
            crate::ontology::term::Value::Boolean(true)
        )))
    );

    Ok(NarrationStep {
        narrates,
        audience: optional_string(iri, graph, HORIS_AUDIENCE, interner),
        model: optional_string(iri, graph, HORIS_MODEL, interner),
        required,
        next: required_iri(iri, graph, HORIS_NEXT, "horis:next", interner)?,
    })
}

fn parse_proposal(
    iri: Iri,
    graph: &SubjectGraph,
    interner: &Interner,
) -> Result<ProposalStep, OntologyError> {
    let candidates = match graph.object(iri, HORIS_CANDIDATES, interner) {
        Some(Object::Node(head)) => graph
            .list(*head, interner)
            .into_iter()
            .map(|o| match o {
                Object::Node(n) => Ok(node_iri(n)),
                other => Err(OntologyError::StepInvalidOperand {
                    subject: format!("{} horis:candidates", interner.resolve(iri)),
                    operator: "horis:candidates".to_owned(),
                    value: object_text(&other, interner),
                    expected: "a list of individual IRIs".to_owned(),
                }),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => {
            return Err(OntologyError::WorkflowMissingPart {
                subject: interner.resolve(iri).to_owned(),
                part: "horis:candidates".to_owned(),
            });
        }
    };
    if candidates.is_empty() {
        return Err(OntologyError::WorkflowMissingPart {
            subject: interner.resolve(iri).to_owned(),
            part: "at least one horis:candidates member".to_owned(),
        });
    }

    Ok(ProposalStep {
        candidates,
        rank_by: optional_string(iri, graph, HORIS_RANK_BY, interner),
        model: optional_string(iri, graph, HORIS_MODEL, interner),
        confidence: required_confidence(iri, graph, interner)?,
        fallback: optional_iri(iri, graph, HORIS_FALLBACK, interner),
        next: required_iri(iri, graph, HORIS_NEXT, "horis:next", interner)?,
    })
}

fn parse_choice(
    iri: Iri,
    graph: &SubjectGraph,
    schema: &Schema,
    interner: &Interner,
) -> Result<ChoiceStep, OntologyError> {
    let branches = match graph.object(iri, HORIS_BRANCHES, interner) {
        Some(Object::Node(head)) => {
            let mut out = Vec::new();
            for item in graph.list(*head, interner) {
                let Object::Node(branch_node) = item else {
                    return Err(OntologyError::WorkflowMissingPart {
                        subject: interner.resolve(iri).to_owned(),
                        part: "a horis:branches list member to be a condition node".to_owned(),
                    });
                };
                let branch_iri = node_iri(branch_node);
                let when_node = match graph.object(branch_iri, HORIS_WHEN, interner) {
                    Some(Object::Node(n)) => *n,
                    _ => {
                        return Err(OntologyError::WorkflowMissingPart {
                            subject: interner.resolve(iri).to_owned(),
                            part: "horis:when on a horis:branches member".to_owned(),
                        });
                    }
                };
                let when = parse_condition_tree(
                    when_node,
                    graph,
                    schema,
                    interner,
                    &format!("{} horis:branches", interner.resolve(iri)),
                    &mut BTreeSet::new(),
                    0,
                )?;
                let next = required_iri(branch_iri, graph, HORIS_NEXT, "horis:next", interner)?;
                out.push(Branch { when, next });
            }
            out
        }
        _ => {
            return Err(OntologyError::WorkflowMissingPart {
                subject: interner.resolve(iri).to_owned(),
                part: "horis:branches".to_owned(),
            });
        }
    };

    Ok(ChoiceStep {
        branches,
        otherwise: optional_iri(iri, graph, HORIS_OTHERWISE, interner),
    })
}

fn parse_wait(iri: Iri, graph: &SubjectGraph, interner: &Interner) -> WaitStep {
    WaitStep {
        next: optional_iri(iri, graph, HORIS_NEXT, interner),
    }
}

fn parse_subflow(
    iri: Iri,
    graph: &SubjectGraph,
    interner: &Interner,
) -> Result<SubflowStep, OntologyError> {
    Ok(SubflowStep {
        workflow: required_iri(iri, graph, HORIS_WORKFLOW_PROP, "horis:workflow", interner)?,
        next: required_iri(iri, graph, HORIS_NEXT, "horis:next", interner)?,
    })
}

// -- Condition parsing (goal/requires/branch `horis:when`) -------------------
//
// Reuses `reason::rule`'s `Condition`/`ConditionTree`/`Op`/`Term` types (they
// carry no rule-specific state — a rule's *subject class* lives on `Rule`,
// not on `Condition`) with a fresh, standalone parser: workflow conditions
// carry no `horis:subject` (they evaluate against the run's current case
// instance, resolved by `workflow::agent` at run time, not against a class
// pattern-matched across the whole world the way rule conditions are).

fn parse_condition_tree(
    node: Node,
    graph: &SubjectGraph,
    schema: &Schema,
    interner: &Interner,
    subject_text: &str,
    on_path: &mut BTreeSet<Iri>,
    depth: usize,
) -> Result<ConditionTree, OntologyError> {
    if depth > MAX_CONDITION_DEPTH {
        return Err(OntologyError::StepNestingTooDeep {
            subject: subject_text.to_owned(),
            max: MAX_CONDITION_DEPTH,
        });
    }
    let subject = node_iri(node);
    if !on_path.insert(subject) {
        return Err(OntologyError::WorkflowMissingPart {
            subject: subject_text.to_owned(),
            part: "an acyclic condition body (a condition node contains itself)".to_owned(),
        });
    }

    type CombinatorMatch = (&'static str, fn(Vec<ConditionTree>) -> ConditionTree, Node);
    let mut found: Option<CombinatorMatch> = None;
    for (combinator, name, build) in [
        (HORIS_ALL, "all", ConditionTree::All as fn(_) -> _),
        (HORIS_ANY, "any", ConditionTree::Any as fn(_) -> _),
        (HORIS_NONE, "none", ConditionTree::NoneOf as fn(_) -> _),
    ] {
        if let Some(Object::Node(head)) = graph.object(subject, combinator, interner) {
            if let Some((existing, _, _)) = found {
                return Err(OntologyError::StepMultipleCombinators {
                    subject: subject_text.to_owned(),
                    a: existing.to_owned(),
                    b: name.to_owned(),
                });
            }
            found = Some((name, build, *head));
        }
    }

    let result = if let Some((_, build, head)) = found {
        let mut children = Vec::new();
        for item in graph.list(head, interner) {
            let Object::Node(child) = item else {
                return Err(OntologyError::WorkflowMissingPart {
                    subject: subject_text.to_owned(),
                    part: "a condition node inside its combinator list".to_owned(),
                });
            };
            children.push(parse_condition_tree(
                child,
                graph,
                schema,
                interner,
                subject_text,
                on_path,
                depth + 1,
            )?);
        }
        Ok(build(children))
    } else {
        Ok(ConditionTree::Leaf(parse_leaf_condition(
            node,
            graph,
            schema,
            interner,
            subject_text,
        )?))
    };
    on_path.remove(&subject);
    result
}

fn parse_leaf_condition(
    node: Node,
    graph: &SubjectGraph,
    schema: &Schema,
    interner: &Interner,
    subject_text: &str,
) -> Result<Condition, OntologyError> {
    let subject = node_iri(node);

    let property = match graph.object(subject, HORIS_PROPERTY, interner) {
        Some(Object::Node(p)) => node_iri(*p),
        _ => {
            return Err(OntologyError::WorkflowMissingPart {
                subject: subject_text.to_owned(),
                part: "horis:property on a condition".to_owned(),
            });
        }
    };
    if schema.property(property).is_none() {
        return Err(OntologyError::StepUndeclaredProperty {
            subject: subject_text.to_owned(),
            property: interner.resolve(property).to_owned(),
        });
    }

    let op = parse_operator(subject, graph, interner, property, subject_text)?;
    Ok(Condition {
        property,
        op,
        unit: None,
    })
}

fn parse_operator(
    subject: Iri,
    graph: &SubjectGraph,
    interner: &Interner,
    property: Iri,
    subject_text: &str,
) -> Result<Op, OntologyError> {
    let literal_operand =
        |object: &Object, operator: &str| -> Result<crate::ontology::term::Value, OntologyError> {
            match object {
                Object::Literal(Literal::Typed(v)) => Ok(v.clone()),
                other => Err(OntologyError::StepInvalidOperand {
                    subject: subject_text.to_owned(),
                    operator: operator.to_owned(),
                    value: object_text(other, interner),
                    expected: "a typed literal".to_owned(),
                }),
            }
        };

    let mut found: Option<Op> = None;
    for (predicate, object) in graph.pairs(subject) {
        let predicate = interner.resolve(*predicate);
        let op = match predicate {
            HORIS_EXCEEDS => Some(Op::Exceeds(literal_operand(object, "horis:exceeds")?)),
            HORIS_BELOW => Some(Op::Below(literal_operand(object, "horis:below")?)),
            HORIS_AT_LEAST => Some(Op::AtLeast(literal_operand(object, "horis:atLeast")?)),
            HORIS_AT_MOST => Some(Op::AtMost(literal_operand(object, "horis:atMost")?)),
            HORIS_BETWEEN => {
                let Object::Node(head) = object else {
                    return Err(OntologyError::StepBetweenArity {
                        subject: subject_text.to_owned(),
                        found: 0,
                    });
                };
                let items = graph.list(*head, interner);
                let mut values = Vec::new();
                for item in &items {
                    match item {
                        Object::Literal(Literal::Typed(v)) => values.push(v.clone()),
                        other => {
                            return Err(OntologyError::StepInvalidOperand {
                                subject: subject_text.to_owned(),
                                operator: "horis:between".to_owned(),
                                value: object_text(other, interner),
                                expected: "a typed literal list member".to_owned(),
                            });
                        }
                    }
                }
                if values.len() != 2 {
                    return Err(OntologyError::StepBetweenArity {
                        subject: subject_text.to_owned(),
                        found: values.len(),
                    });
                }
                Some(Op::Between(values[0].clone(), values[1].clone()))
            }
            HORIS_IS => Some(Op::Is(term_operand(object))),
            HORIS_IS_NOT => Some(Op::IsNot(term_operand(object))),
            HORIS_IS_ASSIGNED => match object {
                Object::Literal(Literal::Typed(crate::ontology::term::Value::Boolean(true))) => {
                    Some(Op::IsAssigned)
                }
                other => {
                    return Err(OntologyError::StepInvalidOperand {
                        subject: subject_text.to_owned(),
                        operator: "horis:isAssigned".to_owned(),
                        value: object_text(other, interner),
                        expected: "the literal true".to_owned(),
                    });
                }
            },
            HORIS_IS_EMPTY => match object {
                Object::Literal(Literal::Typed(crate::ontology::term::Value::Boolean(true))) => {
                    Some(Op::IsEmpty)
                }
                other => {
                    return Err(OntologyError::StepInvalidOperand {
                        subject: subject_text.to_owned(),
                        operator: "horis:isEmpty".to_owned(),
                        value: object_text(other, interner),
                        expected: "the literal true".to_owned(),
                    });
                }
            },
            HORIS_CONTAINS => Some(Op::Contains(term_operand(object))),
            HORIS_IN_LIST => {
                let Object::Node(head) = object else {
                    return Err(OntologyError::StepInvalidOperand {
                        subject: subject_text.to_owned(),
                        operator: "horis:inList".to_owned(),
                        value: object_text(object, interner),
                        expected: "a list".to_owned(),
                    });
                };
                let items = graph
                    .list(*head, interner)
                    .iter()
                    .map(term_operand)
                    .collect();
                Some(Op::InList(items))
            }
            _ => None,
        };
        if let Some(op) = op {
            if let Some(existing) = &found {
                return Err(OntologyError::StepMultipleOperators {
                    subject: subject_text.to_owned(),
                    property: interner.resolve(property).to_owned(),
                    a: existing.name().to_owned(),
                    b: op.name().to_owned(),
                });
            }
            found = Some(op);
        }
    }
    // `horis:subject` is legal on a workflow condition (harmless — nothing
    // reads it here); it is simply never required, unlike a rule condition.
    let _ = HORIS_SUBJECT;
    found.ok_or_else(|| OntologyError::StepMissingOperator {
        subject: subject_text.to_owned(),
        property: interner.resolve(property).to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use crate::ontology::facade::Ontology;
    use crate::reason::rule::Op;

    use super::*;

    fn triage() -> Ontology {
        Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl must load, validate, and build a schema")
    }

    fn iri(onto: &Ontology, local: &str) -> Iri {
        onto.iri(&format!("https://example.org/triage#{local}"))
            .unwrap_or_else(|| panic!("{local} must be named by triage.ttl"))
    }

    /// U2 test target: `:Triage` parses into the exact documented step
    /// graph — every kind, every edge, `:PagingFailed`'s fallback chain
    /// included.
    #[test]
    fn triage_workflow_parses_into_the_documented_step_graph() {
        let onto = triage();
        let wf_iri = iri(&onto, "Triage");
        let wf = onto.workflow(wf_iri).expect(":Triage must parse");

        assert_eq!(wf.starts_at, iri(&onto, "Intake"));
        assert_eq!(wf.version.as_deref(), Some("1.2.0"));
        assert_eq!(wf.decides, Some(iri(&onto, "acuity")));
        assert_eq!(wf.steps.len(), 7);

        let intake = match &wf.step(iri(&onto, "Intake")).expect(":Intake").body {
            StepBody::Perception(p) => p,
            other => panic!("expected PerceptionStep, got {other:?}"),
        };
        assert_eq!(
            intake.extracts,
            vec![iri(&onto, "Patient"), iri(&onto, "Symptom")]
        );
        assert_eq!(intake.model.as_deref(), Some("claude:claude-fable-5"));
        assert_eq!(intake.confidence, 0.85);
        assert_eq!(intake.fallback, Some(iri(&onto, "EscalateToNurse")));
        assert_eq!(intake.next, iri(&onto, "Assess"));

        let assess = match &wf.step(iri(&onto, "Assess")).expect(":Assess").body {
            StepBody::Reasoning(r) => r,
            other => panic!("expected ReasoningStep, got {other:?}"),
        };
        assert_eq!(
            assess.goal,
            ConditionTree::Leaf(Condition {
                property: iri(&onto, "acuity"),
                op: Op::IsAssigned,
                unit: None,
            })
        );
        assert_eq!(assess.on_unreachable, Some(iri(&onto, "EscalateToNurse")));
        assert_eq!(assess.next, iri(&onto, "Route"));

        let route = match &wf.step(iri(&onto, "Route")).expect(":Route").body {
            StepBody::Action(a) => a,
            other => panic!("expected ActionStep, got {other:?}"),
        };
        assert_eq!(route.tool, iri(&onto, "ehr.assign_queue"));
        assert!(route.requires.is_some());
        assert_eq!(route.on_error, Some(iri(&onto, "EscalateToNurse")));
        assert_eq!(route.next, Some(iri(&onto, "Explain")));

        let explain = match &wf.step(iri(&onto, "Explain")).expect(":Explain").body {
            StepBody::Narration(n) => n,
            other => panic!("expected NarrationStep, got {other:?}"),
        };
        assert_eq!(explain.narrates, iri(&onto, "acuity"));
        assert_eq!(explain.audience.as_deref(), Some("charge-nurse"));
        assert!(!explain.required);
        assert_eq!(explain.next, iri(&onto, "Done"));

        let done = wf.step(iri(&onto, "Done")).expect(":Done");
        assert!(done.terminal);
        assert!(matches!(done.body, StepBody::Bare));

        let escalate = wf
            .step(iri(&onto, "EscalateToNurse"))
            .expect(":EscalateToNurse");
        assert!(escalate.terminal);
        let escalate_body = match &escalate.body {
            StepBody::Action(a) => a,
            other => panic!("expected ActionStep, got {other:?}"),
        };
        assert_eq!(escalate_body.tool, iri(&onto, "paging.notify_nurse"));
        assert_eq!(escalate_body.on_error, Some(iri(&onto, "PagingFailed")));
        assert_eq!(escalate_body.next, None);

        let paging_failed = wf.step(iri(&onto, "PagingFailed")).expect(":PagingFailed");
        assert!(paging_failed.terminal);
        assert!(matches!(paging_failed.body, StepBody::Bare));
    }

    #[test]
    fn a_dangling_next_edge_is_a_domain_term_error() {
        let err = Ontology::load_from_str_allow_incomplete(
            "@prefix : <https://example.org/t#> .\n\
             @prefix horis: <https://w3id.org/horismos#> .\n\
             @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             :Patient a rdfs:Class .\n\
             :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean .\n\
             :Wf a horis:Workflow ; horis:startsAt :A .\n\
             :A a horis:ReasoningStep ;\n\
                 horis:goal [ horis:property :flag ; horis:isAssigned true ] ;\n\
                 horis:onUnreachable :A ;\n\
                 horis:next :Nowhere .\n",
            |_| None,
        )
        .expect_err("a next edge to an undeclared step must be rejected at load");
        assert!(err.domain_text().contains("Nowhere"), "{err:?}");
    }

    const DEFERRED_PRELUDE: &str = "@prefix : <https://example.org/t#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    /// Regression for the reachability fix below (`reachable_steps`,
    /// `steps_snapshot_edges`): a document naming two workflows must not
    /// leave either one holding the other's steps — `Workflow.steps` is
    /// scoped to what its own `starts_at` actually reaches, matching the
    /// struct's doc comment. Before the fix, `:Main`'s `:A` (a
    /// `SubflowStep`, present in *every* workflow's steps map because
    /// `step_ids` was never scoped) made `:Sub` incorrectly look like it
    /// nested a subflow of its own, unconditionally rejecting this
    /// otherwise-valid one-level SubflowStep.
    #[test]
    fn a_workflows_steps_are_scoped_to_what_it_reaches_not_the_whole_document() {
        let source = format!(
            "{DEFERRED_PRELUDE}\
             :Sub a horis:Workflow ; horis:startsAt :SubStart .\n\
             :SubStart a horis:Step ; horis:terminal true .\n\
             :Main a horis:Workflow ; horis:startsAt :A .\n\
             :A a horis:SubflowStep ; horis:workflow :Sub ; horis:next :B .\n\
             :B a horis:Step ; horis:terminal true .\n"
        );
        let onto = Ontology::load_from_str_allow_incomplete(&source, |_| None)
            .expect("a genuine one-level subflow must load cleanly");

        let sub_iri = onto
            .iri("https://example.org/t#Sub")
            .expect("Sub is declared");
        let sub = onto.workflow(sub_iri).expect(":Sub must parse");
        assert_eq!(
            sub.steps.len(),
            1,
            ":Sub reaches only :SubStart, never :Main's :A/:B: {:?}",
            sub.steps.keys().collect::<Vec<_>>()
        );

        let main_iri = onto
            .iri("https://example.org/t#Main")
            .expect("Main is declared");
        let main = onto.workflow(main_iri).expect(":Main must parse");
        assert_eq!(main.steps.len(), 2, ":Main reaches :A and :B");
    }

    /// D6's depth-1 cap must still reject genuine nesting after the
    /// reachability fix above — a target workflow that itself contains a
    /// (now correctly-scoped) `SubflowStep` is still too deep.
    #[test]
    fn genuine_two_level_subflow_nesting_is_still_rejected() {
        let source = format!(
            "{DEFERRED_PRELUDE}\
             :Third a horis:Workflow ; horis:startsAt :ThirdStart .\n\
             :ThirdStart a horis:Step ; horis:terminal true .\n\
             :Sub a horis:Workflow ; horis:startsAt :C .\n\
             :C a horis:SubflowStep ; horis:workflow :Third ; horis:next :SubEnd .\n\
             :SubEnd a horis:Step ; horis:terminal true .\n\
             :Main a horis:Workflow ; horis:startsAt :A .\n\
             :A a horis:SubflowStep ; horis:workflow :Sub ; horis:next :B .\n\
             :B a horis:Step ; horis:terminal true .\n"
        );
        let err = Ontology::load_from_str_allow_incomplete(&source, |_| None)
            .expect_err("two levels of subflow nesting must be rejected (D6)");
        assert!(
            matches!(err, OntologyError::SubflowNestingTooDeep { .. }),
            "{err:?}"
        );
    }

    // -- B1: workflow condition nesting is bounded like rule bodies ---------

    /// `levels` nested single-element `horis:all` combinators over one leaf
    /// condition, built programmatically for the same reason as the rule
    /// side's `nested_when`.
    fn nested_condition(levels: usize) -> String {
        let mut body = "[ horis:property :flag ; horis:isAssigned true ]".to_owned();
        for _ in 0..levels {
            body = format!("[ horis:all ( {body} ) ]");
        }
        body
    }

    /// One workflow per entry point: a `ReasoningStep` goal, an
    /// `ActionStep`'s `horis:requires`, and a `ChoiceStep` branch
    /// `horis:when`, each carrying `levels` of nesting.
    fn nested_workflow_sources(levels: usize) -> [(&'static str, String); 3] {
        let body = nested_condition(levels);
        let head = format!(
            "{DEFERRED_PRELUDE}\
             :Patient a rdfs:Class .\n\
             :flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean .\n\
             :Wf a horis:Workflow ; horis:startsAt :A .\n\
             :End a horis:Step ; horis:terminal true .\n"
        );
        [
            (
                "horis:goal",
                format!(
                    "{head}:A a horis:ReasoningStep ; horis:goal {body} ;\n\
                     \x20 horis:onUnreachable :End ; horis:next :End .\n"
                ),
            ),
            (
                "horis:requires",
                format!(
                    "{head}:t a horis:Tool ; horis:binding \"rust:t\" ;\n\
                     \x20 horis:takes [ horis:arg [ horis:name \"p\" ; horis:type :Patient ] ] ;\n\
                     \x20 horis:effect [ horis:asserts [ horis:property :flag ; horis:type xsd:boolean ] ] .\n\
                     :A a horis:ActionStep ; horis:tool :t ; horis:requires {body} ;\n\
                     \x20 horis:onError :End ; horis:next :End .\n"
                ),
            ),
            (
                "horis:branches",
                format!(
                    "{head}:A a horis:ChoiceStep ;\n\
                     \x20 horis:branches ( [ horis:when {body} ; horis:next :End ] ) ;\n\
                     \x20 horis:otherwise :End .\n"
                ),
            ),
        ]
    }

    /// The defect: `parse_condition_tree` guarded cycles but not depth, so a
    /// workflow condition a few thousand levels deep overflowed the stack
    /// on every load, exactly as a rule body did.
    #[test]
    fn a_workflow_condition_nested_deeper_than_the_cap_is_refused() {
        for (entry, source) in nested_workflow_sources(MAX_CONDITION_DEPTH + 1) {
            let err = Ontology::load_from_str_allow_incomplete(&source, |_| None)
                .expect_err("nesting past the cap must be a load error, not a crash");
            assert!(
                matches!(
                    &err,
                    OntologyError::StepNestingTooDeep { max, .. } if *max == MAX_CONDITION_DEPTH
                ),
                "{entry}: {err:?}"
            );
            assert!(
                err.domain_text().contains(entry),
                "{entry}: the message names where the nesting is: {err}"
            );
        }
    }

    /// Companion: each entry point still loads at the cap, so the refusal
    /// above is about depth and nothing else.
    #[test]
    fn a_workflow_condition_nested_to_the_cap_still_loads() {
        for (entry, source) in nested_workflow_sources(MAX_CONDITION_DEPTH) {
            let onto = Ontology::load_from_str_allow_incomplete(&source, |_| None)
                .unwrap_or_else(|e| panic!("{entry}: nesting at the cap must still load: {e:?}"));
            let wf_iri = onto
                .iri("https://example.org/t#Wf")
                .expect("Wf is declared");
            let a_iri = onto.iri("https://example.org/t#A").expect("A is declared");
            let step = onto
                .workflow(wf_iri)
                .and_then(|wf| wf.step(a_iri))
                .expect(":A must parse");
            let mut node = match &step.body {
                StepBody::Reasoning(r) => &r.goal,
                StepBody::Action(a) => a.requires.as_ref().expect("requires is present"),
                StepBody::Choice(c) => &c.branches[0].when,
                other => panic!("{entry}: unexpected step body {other:?}"),
            };
            let mut depth = 0;
            while let ConditionTree::All(children) = node {
                depth += 1;
                node = &children[0];
            }
            assert_eq!(depth, MAX_CONDITION_DEPTH, "{entry}");
            assert!(matches!(node, ConditionTree::Leaf(_)), "{entry}: {node:?}");
        }
    }
}
