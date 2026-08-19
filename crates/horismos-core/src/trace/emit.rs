//! Emission (U7/U9): converting a transaction's typed results into trace
//! [`Step`]s — the one place interned tokens become trace strings (compacted
//! IRIs, lexical values). `reasoning_step` is PRP-002's; `perception_step`/
//! `action_step`/`narration_step`/`proposal_step` are PRP-004's (U9),
//! mirroring its pattern for the four kinds a real run can actually
//! produce today. `Choice`/`Subflow`/`Wait` (v2-only, doc 09/`trace::schema`)
//! have no builder yet — nothing in this PRP executes those kinds (U12), so
//! there is no typed input to build one from; `trace::schema::Step`'s own
//! variants are the schema's complete word on their shape in the meantime.
//!
//! Emission is a pure function of (world, typed inputs): no wall-clock, no
//! ids minted, no iteration over unordered state — so two identical calls
//! emit byte-identical steps, and trace equality stays a valid test
//! assertion (CLAUDE.md). Timing and step-specific derived flags
//! (`latency_ms`; `reasoning_step`'s `goal_reached`) exist in the schema but
//! are stamped by the runtime (PRP-004) after the call, via direct field
//! assignment on the returned `Step` — never computed here, since that
//! needs context (elapsed time; the workflow's own `horis:goal`) this
//! module does not have.
//!
//! Spec: `docs/framework/09-traces.md` · Owner: PRP-002 (reasoning_step),
//! PRP-004 (perception_step/action_step/narration_step/proposal_step, U9)

use std::collections::BTreeMap;

use crate::World;
use crate::ontology::term::{Iri, Ref, Value};
use crate::reason::forward::{Firing, FiringOutcome, ReasoningRecord};
use crate::reason::proof::{ConditionVia, ProofKind, ProofNode};
use crate::trace::schema::{
    DiagnosticEntry, FiredEntry, ProofJson, Step, SuppressedEntry, TraceFact,
};
use crate::world::Provenance;

/// Builds the reasoning [`Step`] for one committed transaction.
/// `step_name` is the workflow step IRI when a workflow is running
/// (PRP-004); `None` for a bare transaction.
pub fn reasoning_step(world: &World, step_name: Option<&str>, record: &ReasoningRecord) -> Step {
    let mut fired = Vec::new();
    let mut suppressed = Vec::new();
    let mut diagnostics = Vec::new();

    for firing in &record.fired {
        match &firing.outcome {
            FiringOutcome::Derived => fired.push(fired_entry(world, firing)),
            FiringOutcome::Suppressed { winner } | FiringOutcome::Overridden { winner } => {
                suppressed.push(SuppressedEntry {
                    rule: compact(world, firing.rule),
                    bindings: bindings(world, firing),
                    conclusion: fact(world, firing, Some(Provenance::Rule)),
                    winner: compact(world, *winner),
                });
            }
            FiringOutcome::Rejected { witness } => diagnostics.push(DiagnosticEntry {
                rule: compact(world, firing.rule),
                bindings: bindings(world, firing),
                conclusion: fact(world, firing, Some(Provenance::Rule)),
                witness: witness.text.clone(),
            }),
        }
    }

    Step::Reasoning {
        step: step_name.map(str::to_owned),
        fired,
        suppressed,
        diagnostics,
        passes: record.passes,
        goal_reached: None,
        latency_ms: None,
        // Design rule 2 (doc 09): executor output is masked in training.
        loss_mask: true,
    }
}

/// One fact a `PerceptionStep` asserted into the World, in interned form —
/// [`perception_step`]'s input. Per-fact `confidence` mirrors doc 09's
/// `asserted[].conf` (`PerceiveResult::fact_confidence`, falling back to the
/// call's overall confidence when the provider does not support it, doc 06
/// § Confidence).
#[derive(Debug, Clone)]
pub struct PerceivedFact {
    pub subject: Ref,
    pub property: Iri,
    pub value: Value,
    pub confidence: Option<f64>,
}

/// One effect a `Tool` asserted back into the World, in interned form —
/// [`action_step`]'s input (doc 07: only what passed `horis:effect`
/// validation through the gate is ever in this list).
#[derive(Debug, Clone)]
pub struct AssertedEffect {
    pub subject: Ref,
    pub property: Iri,
    pub value: Value,
}

/// Builds the [`Step::Perception`] for one transducer call (doc 06, U9).
/// `accepted` is the confidence gate's own verdict — `confidence >=
/// threshold` — computed here whenever both are known, since (unlike
/// `reasoning_step`'s `goal_reached`) it needs no context beyond the two
/// values already passed in.
pub fn perception_step(
    world: &World,
    step_name: Option<&str>,
    model: Option<String>,
    confidence: Option<f64>,
    threshold: Option<f64>,
    asserted: &[PerceivedFact],
) -> Step {
    let accepted = match (confidence, threshold) {
        (Some(c), Some(t)) => Some(c >= t),
        _ => None,
    };
    Step::Perception {
        step: step_name.map(str::to_owned),
        model,
        confidence,
        threshold,
        accepted,
        latency_ms: None,
        asserted: asserted
            .iter()
            .map(|f| {
                trace_fact(
                    world,
                    f.subject,
                    f.property,
                    &f.value,
                    Provenance::Perception,
                    f.confidence,
                )
            })
            .collect(),
        // Design rule 2 (doc 09): a transducer's output is model-generated,
        // trainable.
        loss_mask: false,
    }
}

/// Builds the [`Step::Action`] for one tool invocation (doc 07, U9).
pub fn action_step(
    world: &World,
    step_name: Option<&str>,
    tool: Option<&str>,
    effects_asserted: &[AssertedEffect],
) -> Step {
    Step::Action {
        step: step_name.map(str::to_owned),
        tool: tool.map(str::to_owned),
        latency_ms: None,
        effects_asserted: effects_asserted
            .iter()
            .map(|f| {
                trace_fact(
                    world,
                    f.subject,
                    f.property,
                    &f.value,
                    Provenance::Tool,
                    None,
                )
            })
            .collect(),
        // Design rule 2: a tool's effect is executor output, masked.
        loss_mask: true,
    }
}

/// Builds the [`Step::Narration`] for one `Bridge::narrate` call (doc 06,
/// U9). No `World` needed: narration renders a proof tree already resolved
/// to text (`proof.render()`'s output, or the model's rendering of it) and
/// asserts nothing.
pub fn narration_step(
    step_name: Option<&str>,
    audience: Option<String>,
    text: Option<String>,
) -> Step {
    Step::Narration {
        step: step_name.map(str::to_owned),
        audience,
        text,
        // Design rule 2: the rendered prose is model-generated, trainable
        // (the mechanical `proof.render()` fallback is also captured here —
        // it is still the step's recorded output — but D7's non-fatal
        // narration failure means this builder only ever sees text that
        // exists, mechanical or modeled).
        loss_mask: false,
    }
}

/// Builds the [`Step::Proposal`] for one `Bridge::rank` call (doc 07, U9).
/// `candidates` is the precondition-satisfied set the *runtime* computed —
/// never more than that, per doc 07 ("the LLM ranks; it cannot add
/// options") — in the order presented to the model.
pub fn proposal_step(
    world: &World,
    step_name: Option<&str>,
    model: Option<String>,
    candidates: &[Ref],
    confidence: Option<f64>,
    threshold: Option<f64>,
    chosen: Option<Ref>,
) -> Step {
    let accepted = match (confidence, threshold) {
        (Some(c), Some(t)) => Some(c >= t),
        _ => None,
    };
    Step::Proposal {
        step: step_name.map(str::to_owned),
        model,
        candidates: candidates.iter().map(|r| compact(world, r.iri())).collect(),
        confidence,
        threshold,
        accepted,
        chosen: chosen.map(|r| compact(world, r.iri())),
        latency_ms: None,
        // Design rule 2: the ranking is model-generated, trainable — same
        // posture as perception (schema doc comment on `Step::Proposal`).
        loss_mask: false,
    }
}

fn trace_fact(
    world: &World,
    subject: Ref,
    property: Iri,
    value: &Value,
    provenance: Provenance,
    conf: Option<f64>,
) -> TraceFact {
    TraceFact {
        s: compact(world, subject.iri()),
        p: compact(world, property),
        o: value_json(world, value),
        provenance: Some(provenance),
        conf,
    }
}

fn fired_entry(world: &World, firing: &Firing) -> FiredEntry {
    FiredEntry {
        rule: compact(world, firing.rule),
        bindings: bindings(world, firing),
        derived: vec![fact(world, firing, Some(Provenance::Rule))],
        proof: proof_json(world, &firing.proof),
    }
}

/// Doc 09's binding map: the rule's one subject variable, named after its
/// class (`?p` for `:Patient` — the controller pin matching the doc's own
/// `{"?p": "_:p7"}` example), bound to the subject's IRI.
fn bindings(world: &World, firing: &Firing) -> BTreeMap<String, String> {
    let class_initial = world
        .ontology()
        .rules()
        .iter()
        .find(|r| r.iri == firing.rule)
        .map(|r| world.ontology().local_name(world.resolve(r.subject_class)))
        .and_then(|local| local.chars().next().map(|c| c.to_ascii_lowercase()))
        .unwrap_or('s');
    BTreeMap::from([(
        format!("?{class_initial}"),
        compact(world, firing.subject.iri()),
    )])
}

fn fact(world: &World, firing: &Firing, provenance: Option<Provenance>) -> TraceFact {
    TraceFact {
        s: compact(world, firing.subject.iri()),
        p: compact(world, firing.property),
        o: value_json(world, &firing.value),
        provenance,
        conf: None,
    }
}

/// The trace form of one proof node (doc 04's ProofNode, resolved).
pub fn proof_json(world: &World, node: &ProofNode) -> ProofJson {
    match &node.kind {
        ProofKind::Conclusion {
            subject,
            property,
            value,
            rule,
            explain,
        } => ProofJson {
            goal: triple_text(world, *subject, *property, value),
            via: compact(world, *rule),
            explain: explain.clone(),
            provenance: None,
            conditions: node
                .children
                .iter()
                .map(|child| proof_json(world, child))
                .collect(),
        },
        ProofKind::Group { combinator, held } => ProofJson {
            goal: format!(
                "{} of {} condition(s){}",
                match combinator {
                    crate::reason::proof::Combinator::All => "all",
                    crate::reason::proof::Combinator::Any => "any",
                    crate::reason::proof::Combinator::NoneOf => "none",
                },
                node.children.len(),
                if *held { "" } else { " (not held)" },
            ),
            via: "group".to_owned(),
            explain: None,
            provenance: None,
            conditions: node
                .children
                .iter()
                .map(|child| proof_json(world, child))
                .collect(),
        },
        ProofKind::Condition {
            subject,
            condition,
            via,
            ..
        } => {
            let (via_text, provenance) = match via {
                ConditionVia::Asserted { provenance, .. } => ("asserted".to_owned(), *provenance),
                ConditionVia::Subsumption {
                    value, provenance, ..
                } => (
                    format!("subsumption({})", compact(world, *value)),
                    *provenance,
                ),
                ConditionVia::Derived => ("derived".to_owned(), None),
                ConditionVia::ClosedWorld => ("closed-world".to_owned(), None),
            };
            ProofJson {
                goal: format!(
                    "{} {} {}",
                    compact(world, subject.iri()),
                    world
                        .ontology()
                        .local_name(world.resolve(condition.property)),
                    crate::reason::proof::render_op(world, &condition.op),
                ),
                via: via_text,
                explain: None,
                provenance,
                conditions: node
                    .children
                    .iter()
                    .map(|child| proof_json(world, child))
                    .collect(),
            }
        }
    }
}

fn triple_text(world: &World, subject: Ref, property: Iri, value: &Value) -> String {
    format!(
        "{} {} {}",
        compact(world, subject.iri()),
        compact(world, property),
        match value {
            Value::Ref(r) => compact(world, r.iri()),
            other => other.lexical_form().unwrap_or_default(),
        }
    )
}

fn compact(world: &World, iri: Iri) -> String {
    world.ontology().compact(world.resolve(iri))
}

/// A stored value in trace form: integers and booleans as JSON natives,
/// everything else (decimals included — JSON numbers are lossy floats) in
/// its lexical form; node references as compacted IRIs.
fn value_json(world: &World, value: &Value) -> serde_json::Value {
    match value {
        Value::Integer(i) => serde_json::Value::from(*i),
        Value::Boolean(b) => serde_json::Value::from(*b),
        Value::Ref(r) => serde_json::Value::from(compact(world, r.iri())),
        other => serde_json::Value::from(other.lexical_form().unwrap_or_default()),
    }
}
