//! Proof trees (U4/U5): the recorded derivation of a conclusion — goal at
//! the root, one node per condition (D-2: per-condition granularity, with
//! subsumption hops explicit), provenance on every leaf.
//!
//! One structure serves both directions (doc 04): forward chaining records a
//! [`ProofNode`] per firing (consumed by `World::explain`), and backward
//! chaining's `prove` builds the same shape. `render` produces doc 04's
//! human form, used by traces, narration (doc 06), and authoring (doc 13).
//!
//! Nodes hold interned tokens, not strings — they are world-relative, like
//! every other in-memory structure; rendering and trace serialization
//! resolve them against the world at the edge.
//!
//! Spec: `docs/framework/04-reasoning.md` · Owner: PRP-002

use crate::World;
use crate::ontology::term::{Iri, Ref, Value};
use crate::reason::eval;
use crate::reason::rule::{Condition, ConditionTree, Op, Rule, Term};
use crate::world::Provenance;

/// One node of a proof tree.
#[derive(Debug, Clone, PartialEq)]
pub struct ProofNode {
    pub kind: ProofKind,
    pub children: Vec<ProofNode>,
}

/// What a [`ProofNode`] asserts.
#[derive(Debug, Clone, PartialEq)]
pub enum ProofKind {
    /// The root of a derivation: `subject.property = value`, via `rule`.
    Conclusion {
        subject: Ref,
        property: Iri,
        value: Value,
        rule: Iri,
        explain: Option<String>,
    },
    /// A combinator node (`horis:any` / `horis:none` / nested `horis:all`),
    /// kept so the recorded structure mirrors the authored rule.
    Group { combinator: Combinator, held: bool },
    /// One leaf condition evaluation.
    Condition {
        subject: Ref,
        condition: Condition,
        held: bool,
        via: ConditionVia,
    },
}

/// The three combinators, as they appear in proofs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    All,
    Any,
    NoneOf,
}

/// How a leaf condition was established.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionVia {
    /// Held against an asserted value, matched directly.
    Asserted {
        value: Value,
        provenance: Option<Provenance>,
    },
    /// Held against an asserted node value through the class hierarchy:
    /// `value ⊑ target` (doc 04's explicit `via: subsumption(…)` step).
    Subsumption {
        value: Iri,
        target: Iri,
        provenance: Option<Provenance>,
    },
    /// Held with no witness value — the closed-world rows of doc 04's table
    /// (`isNot`/`isEmpty` over an unassigned property) — or did not hold.
    ClosedWorld,
    /// Established hypothetically by a nested rule derivation during
    /// backward chaining (U6); the nested proof is in `children`.
    Derived,
}

/// Builds the recorded proof for one firing: the conclusion at the root,
/// per-condition children mirroring the rule's `horis:when` tree.
pub fn firing_proof(world: &World, subject: Ref, rule: &Rule, value: &Value) -> ProofNode {
    ProofNode {
        kind: ProofKind::Conclusion {
            subject,
            property: rule.then.property,
            value: value.clone(),
            rule: rule.iri,
            explain: rule.explain.clone(),
        },
        children: tree_children(world, subject, &rule.when),
    }
}

/// The proof children for one `horis:when` node. A top-level `horis:all` is
/// flattened into the conclusion's direct children (doc 04's rendered form);
/// `any`/`none`/nested combinators keep an explicit [`ProofKind::Group`]
/// node.
fn tree_children(world: &World, subject: Ref, tree: &ConditionTree) -> Vec<ProofNode> {
    match tree {
        ConditionTree::All(children) => children
            .iter()
            .map(|child| tree_proof(world, subject, child))
            .collect(),
        other => vec![tree_proof(world, subject, other)],
    }
}

/// The proof node for one `horis:when` subtree.
pub fn tree_proof(world: &World, subject: Ref, tree: &ConditionTree) -> ProofNode {
    match tree {
        ConditionTree::Leaf(condition) => condition_proof(world, subject, condition),
        ConditionTree::All(children) => group_proof(world, subject, Combinator::All, children),
        ConditionTree::Any(children) => group_proof(world, subject, Combinator::Any, children),
        ConditionTree::NoneOf(children) => {
            group_proof(world, subject, Combinator::NoneOf, children)
        }
    }
}

fn group_proof(
    world: &World,
    subject: Ref,
    combinator: Combinator,
    children: &[ConditionTree],
) -> ProofNode {
    let tree = match combinator {
        Combinator::All => ConditionTree::All(children.to_vec()),
        Combinator::Any => ConditionTree::Any(children.to_vec()),
        Combinator::NoneOf => ConditionTree::NoneOf(children.to_vec()),
    };
    ProofNode {
        kind: ProofKind::Group {
            combinator,
            held: eval::holds(world, subject, &tree),
        },
        children: children
            .iter()
            .map(|child| tree_proof(world, subject, child))
            .collect(),
    }
}

/// The proof node for one leaf condition: its truth value plus *how* — the
/// witness value and whether the match went through the hierarchy.
pub fn condition_proof(world: &World, subject: Ref, condition: &Condition) -> ProofNode {
    let held = eval::condition_holds(world, subject, condition);
    let via = if held {
        witness_via(world, subject, condition)
    } else {
        ConditionVia::ClosedWorld
    };
    ProofNode {
        kind: ProofKind::Condition {
            subject,
            condition: condition.clone(),
            held,
            via,
        },
        children: Vec::new(),
    }
}

/// For a condition that held: which asserted value witnessed it, and was the
/// match direct or a subsumption hop.
fn witness_via(world: &World, subject: Ref, condition: &Condition) -> ConditionVia {
    // The first value that satisfies the operator on its own — the ∃
    // witness. The ∀/absence operators (isNot, isEmpty) yield no witness by
    // construction and fall through to ClosedWorld.
    let witness = world
        .values(subject, condition.property)
        .find(|value| eval::value_satisfies(world, value, &condition.op))
        .cloned();

    let Some(value) = witness else {
        // isNot/isEmpty (∀/absence) hold without a witness value; so does an
        // unassigned-property edge case.
        return ConditionVia::ClosedWorld;
    };

    let provenance = world.fact_provenance(subject, condition.property, &value);
    if let (Value::Ref(r), Some(target)) = (&value, condition.op.node_target()) {
        let schema = world.ontology().schema();
        let direct = schema.canonical_iri(r.iri()) == schema.canonical_iri(target);
        if !direct {
            return ConditionVia::Subsumption {
                value: r.iri(),
                target,
                provenance,
            };
        }
    }
    ConditionVia::Asserted { value, provenance }
}

impl Op {
    /// The IRI a node value would be matched against, when this operator
    /// compares against a node term: `is`/`contains` directly; for `inList`
    /// the first listed IRI the world's value matches is reported by the
    /// witness logic instead (`None` here keeps it conservative).
    fn node_target(&self) -> Option<Iri> {
        match self {
            Op::Is(Term::Iri(t)) | Op::Contains(Term::Iri(t)) => Some(*t),
            _ => None,
        }
    }
}

// -- Rendering (doc 04's human form) ----------------------------------------

impl ProofNode {
    /// Renders the doc 04 tree:
    ///
    /// ```text
    /// :acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"
    ///   ├─ heartRate 128 exceeds 120            (asserted by the api)
    ///   └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (subsumption)
    /// ```
    pub fn render(&self, world: &World) -> String {
        let mut out = String::new();
        self.render_line(world, &mut out);
        out.push('\n');
        render_children(&self.children, world, "  ", &mut out);
        out
    }

    fn render_line(&self, world: &World, out: &mut String) {
        let onto = world.ontology();
        match &self.kind {
            ProofKind::Conclusion {
                property,
                value,
                rule,
                explain,
                ..
            } => {
                out.push_str(&format!(
                    "{} = {}   via {}",
                    onto.compact(world.resolve(*property)),
                    render_value(world, value),
                    onto.local_name(world.resolve(*rule)),
                ));
                if let Some(text) = explain {
                    out.push_str(&format!(" — \"{text}\""));
                }
            }
            ProofKind::Group { combinator, held } => {
                let name = match combinator {
                    Combinator::All => "all",
                    Combinator::Any => "any",
                    Combinator::NoneOf => "none",
                };
                out.push_str(&format!(
                    "{name} of:{}",
                    if *held { "" } else { "   (not held)" }
                ));
            }
            ProofKind::Condition {
                condition,
                held,
                via,
                ..
            } => {
                let property = onto.local_name(world.resolve(condition.property));
                match via {
                    ConditionVia::Subsumption { value, target, .. } => {
                        out.push_str(&format!(
                            "{property} {} ⊑ {}   (subsumption)",
                            onto.compact(world.resolve(*value)),
                            onto.compact(world.resolve(*target)),
                        ));
                    }
                    ConditionVia::Asserted { value, provenance } => {
                        out.push_str(&format!(
                            "{property} {} {}   (asserted{})",
                            render_value(world, value),
                            render_op(world, &condition.op),
                            provenance
                                .map(|p| format!(" by {}", p.as_str()))
                                .unwrap_or_default(),
                        ));
                    }
                    ConditionVia::Derived => {
                        out.push_str(&format!(
                            "{property} {}   (derived)",
                            render_op(world, &condition.op)
                        ));
                    }
                    ConditionVia::ClosedWorld => {
                        out.push_str(&format!(
                            "{property} {}   ({})",
                            render_op(world, &condition.op),
                            if *held { "closed world" } else { "not held" },
                        ));
                    }
                }
            }
        }
    }
}

fn render_children(children: &[ProofNode], world: &World, indent: &str, out: &mut String) {
    for (i, child) in children.iter().enumerate() {
        let last = i + 1 == children.len();
        out.push_str(indent);
        out.push_str(if last { "└─ " } else { "├─ " });
        child.render_line(world, out);
        out.push('\n');
        let deeper = format!("{indent}{}", if last { "   " } else { "│  " });
        render_children(&child.children, world, &deeper, out);
    }
}

/// A value in proof text: literals in lexical form, nodes in compact form.
fn render_value(world: &World, value: &Value) -> String {
    match value {
        Value::Ref(r) => world.ontology().compact(world.resolve(r.iri())),
        other => other.lexical_form().unwrap_or_default(),
    }
}

/// The operator with its operand(s), in doc 04's phrasing ("exceeds 120",
/// "is :ChestPain").
pub(crate) fn render_op(world: &World, op: &Op) -> String {
    let term = |t: &Term| match t {
        Term::Iri(i) => world.ontology().compact(world.resolve(*i)),
        Term::Literal(v) => v.lexical_form().unwrap_or_default(),
    };
    let value = |v: &Value| v.lexical_form().unwrap_or_default();
    match op {
        Op::Exceeds(v) => format!("exceeds {}", value(v)),
        Op::Below(v) => format!("below {}", value(v)),
        Op::AtLeast(v) => format!("atLeast {}", value(v)),
        Op::AtMost(v) => format!("atMost {}", value(v)),
        Op::Between(lo, hi) => format!("between {} and {}", value(lo), value(hi)),
        Op::Is(t) => format!("is {}", term(t)),
        Op::IsNot(t) => format!("isNot {}", term(t)),
        Op::IsAssigned => "isAssigned".to_owned(),
        Op::IsEmpty => "isEmpty".to_owned(),
        Op::Contains(t) => format!("contains {}", term(t)),
        Op::InList(terms) => format!(
            "inList ({})",
            terms.iter().map(term).collect::<Vec<_>>().join(" ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reason::testing::example_world;

    fn t(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/t#{local}"))
    }

    /// PRP Example 2's `explain` half: after Example 1's transaction
    /// committed, the recorded r1 derivation renders in doc 04's exact
    /// shape — conclusion via rule with its explain string, one line per
    /// condition, the subsumption hop explicit.
    #[test]
    fn explain_renders_the_recorded_r1_proof_in_doc_04_shape() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        let acuity = t(&mut world, "acuity");

        let p7 = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(128)),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("accepted");

        let proof = world
            .explain(p7, acuity)
            .expect("acuity was rule-derived, so it has a recorded derivation");
        assert_eq!(
            proof.render(&world),
            ":acuity = :Immediate   via r1 — \"Tachycardia with chest pain — immediate track\"\n\
             \x20 ├─ heartRate 128 exceeds 120   (asserted by the api)\n\
             \x20 └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (subsumption)\n"
        );
    }

    #[test]
    fn explain_is_none_for_base_facts_and_unset_properties() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let acuity = t(&mut world, "acuity");

        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(96))])
            .expect("accepted");

        assert!(
            world.explain(p, heart_rate).is_none(),
            "a base fact has no rule derivation to explain"
        );
        assert!(
            world.explain(p, acuity).is_none(),
            "an unset property has nothing to explain"
        );
    }

    /// The proof's leaves carry the provenance of the facts they rest on —
    /// doc 04's visible neuro-symbolic seam ("asserted by perception").
    #[test]
    fn proof_leaves_carry_the_asserting_provenance() {
        use crate::world::Provenance;

        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        let acuity = t(&mut world, "acuity");

        let p7 = world
            .assert_instance_as(
                Provenance::Perception,
                None,
                patient,
                [
                    (heart_rate, Value::Integer(128)),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("accepted");

        let proof = world.explain(p7, acuity).expect("derived");
        let rendered = proof.render(&world);
        assert!(
            rendered.contains("(asserted by perception)"),
            "the neural seam must be visible: {rendered}"
        );

        let ProofKind::Condition { via, .. } = &proof.children[1].kind else {
            panic!("second child is the symptom condition");
        };
        let ConditionVia::Subsumption {
            provenance: Some(Provenance::Perception),
            ..
        } = via
        else {
            panic!("the subsumption leaf must carry perception provenance, got {via:?}");
        };
    }

    /// After a late higher-priority override (U4), explain answers with the
    /// *standing* derivation, not the replaced one.
    #[test]
    fn explain_follows_a_conflict_override() {
        use crate::Ontology;
        use crate::reason::testing::PRELUDE;

        let source = format!(
            "{PRELUDE}\
             :Patient a rdfs:Class .\n\
             :Symptom a rdfs:Class .\n\
             :ChestPain rdfs:subClassOf :Symptom .\n\
             :AcuityLevel a rdfs:Class ; horis:oneOf ( :Immediate :Urgent :Standard ) .\n\
             :hasSymptom a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Symptom ;\n\
             \x20   horis:cardinality \"0..*\" .\n\
             :acuity a rdf:Property ; rdfs:domain :Patient ; rdfs:range :AcuityLevel ;\n\
             \x20   horis:cardinality \"0..1\" ; horis:assignedBy horis:RulesOnly .\n\
             :needsCardiacMonitor a rdf:Property ; rdfs:domain :Patient ;\n\
             \x20   rdfs:range xsd:boolean ; horis:cardinality \"0..1\" ;\n\
             \x20   horis:assignedBy horis:RulesOnly .\n\
             :rA a horis:Rule ; horis:priority 9 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] .\n\
             :rC a horis:Rule ; horis:priority 8 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ] ) ] ;\n\
             \x20 horis:then [ horis:property :needsCardiacMonitor ; horis:value true ] .\n\
             :rB a horis:Rule ; horis:priority 20 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :needsCardiacMonitor ; horis:is true ] ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Immediate ] .\n"
        );
        let onto = Ontology::load_from_str(&source, |_| None).expect("valid");
        let mut world = World::new(&onto);
        let patient = t(&mut world, "Patient");
        let has_symptom = t(&mut world, "hasSymptom");
        let chest_pain = t(&mut world, "ChestPain");
        let acuity = t(&mut world, "acuity");

        let p = world
            .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(chest_pain)))])
            .expect("accepted");

        let proof = world.explain(p, acuity).expect("derived");
        let ProofKind::Conclusion { rule, .. } = &proof.kind else {
            panic!("a derivation roots at a Conclusion");
        };
        assert_eq!(
            world.resolve(*rule),
            "https://example.org/t#rB",
            "explain must answer with the standing (overriding) derivation"
        );
    }
}
