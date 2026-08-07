//! Emission (U7): converting a transaction's [`ReasoningRecord`] into a
//! `horismos.trace/1` reasoning [`Step`] — the one place interned tokens
//! become trace strings (compacted IRIs, lexical values).
//!
//! Emission is a pure function of (world, record): no wall-clock, no ids
//! minted, no iteration over unordered state — so two identical
//! transactions emit byte-identical steps, and trace equality stays a valid
//! test assertion (CLAUDE.md). Timing fields exist in the schema but are
//! stamped by the runtime (PRP-004), never here.
//!
//! Spec: `docs/framework/09-traces.md` · Owner: PRP-002

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
