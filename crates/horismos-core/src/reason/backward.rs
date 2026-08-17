//! Backward chaining (U6): `prove(goal)` — goal-driven search for a
//! derivation that *would* establish a fact, ported from sigil-lang's
//! `backward.py` and re-keyed to graph goals.
//!
//! What carries over unchanged: the rule index (here: rules whose
//! conclusion matches the goal), the in-progress goal set for cycle
//! detection (`_proving`), the depth cap of 50, and the shape of the search
//! — find rules that could conclude the goal, recursively try to establish
//! their conditions from facts or from other rules. What the graph adds:
//! goals are (subject, property, value) triples, condition truth is U2's
//! closed-world evaluation, and hypothetical derivations are asserted into
//! a **cloned hypothesis world through the real gate** (O(1) clone,
//! persistent structure) — so even a hypothesis cannot violate a
//! constraint, and `prove` never mutates the world it was asked about.
//!
//! `prove` answers "*could* this be established?"; `World::explain` (U5)
//! answers "why *is* this established?" from the forward-chaining record.
//! Both return the same [`ProofNode`] shape (doc 04).
//!
//! Negated combinators (`horis:none`) and `isNot`/`isEmpty` are *evaluated*
//! against the hypothesis, never established by derivation — proving a
//! negation by deriving more facts would be non-monotonic under the
//! closed-world reading.
//!
//! Spec: `docs/framework/04-reasoning.md` · Owner: PRP-002

use std::collections::BTreeSet;

use crate::World;
use crate::ontology::term::{Iri, Ref, Value};
use crate::reason::eval;
use crate::reason::proof::{self, ConditionVia, ProofKind, ProofNode};
use crate::reason::rule::{Condition, ConditionTree, Op, Rule, Term};

/// sigil-lang `BackwardChainer.MAX_DEPTH`, carried over as-is (doc 04).
pub const MAX_DEPTH: usize = 50;

/// Searches for a derivation of `(subject, property, value)` — an existing
/// fact answers immediately (a leaf proof); otherwise rules whose
/// conclusions match the goal are tried depth-first, highest priority
/// first. `None` means the goal is not establishable from the current
/// facts and rules within the caps.
pub fn prove(world: &World, subject: Ref, property: Iri, value: &Value) -> Option<ProofNode> {
    // The hypothesis world: hypothetical conclusions land here, through the
    // gate; the caller's world is untouched (prove borrows it immutably).
    let mut hypothesis = world.clone();
    let mut proving = BTreeSet::new();
    prove_goal(&mut hypothesis, subject, property, value, &mut proving, 0)
}

/// One goal in the search. `proving` holds the (subject, property) pairs on
/// the current path — sigil-lang's `_proving` set, the cycle guard.
fn prove_goal(
    hypothesis: &mut World,
    subject: Ref,
    property: Iri,
    value: &Value,
    proving: &mut BTreeSet<(Iri, Iri)>,
    depth: usize,
) -> Option<ProofNode> {
    let goal_term = value_as_term(value);
    let goal_condition = Condition {
        property,
        op: Op::Is(goal_term),
        unit: None,
    };

    // Base case: the fact already holds (directly or via subsumption) — a
    // leaf proof carrying how, and with what provenance.
    if eval::condition_holds(hypothesis, subject, &goal_condition) {
        return Some(proof::condition_proof(hypothesis, subject, &goal_condition));
    }

    // Depth cap and cycle detection, exactly backward.py's order.
    if depth >= MAX_DEPTH {
        return None;
    }
    if !proving.insert((subject.iri(), property)) {
        return None; // already proving this goal — a rule cycle
    }

    let ontology = hypothesis.ontology().clone();
    let mut result = None;
    for rule in ontology.rules() {
        if !concludes_goal(hypothesis, rule, subject, property, value) {
            continue;
        }
        if let Some(proof) = try_rule(hypothesis, rule, subject, proving, depth) {
            result = Some(proof);
            break;
        }
    }

    proving.remove(&(subject.iri(), property));
    result
}

/// The rule index: does `rule`'s conclusion match the goal, and does the
/// rule bind `subject`'s class? (PRP Example 2 step 1: only `:r1` — `:r2`
/// concludes `:Urgent`, a different value.)
fn concludes_goal(world: &World, rule: &Rule, subject: Ref, property: Iri, value: &Value) -> bool {
    if rule.then.property != property {
        return false;
    }
    let schema = world.ontology().schema();
    let subject_bound = world
        .instance(subject)
        .is_some_and(|i| schema.is_subclass_of(i.class(), rule.subject_class));
    if !subject_bound {
        return false;
    }
    match (&rule.then.value, value) {
        (Term::Iri(concluded), Value::Ref(goal)) => {
            schema.canonical_iri(*concluded) == schema.canonical_iri(goal.iri())
        }
        // Numeric cross-typing (an authored `1` matches a goal `1.0`), same
        // relation the evaluator uses everywhere else.
        (Term::Literal(concluded), goal) => eval::value_eq(concluded, goal),
        (Term::Iri(_), _) => false,
    }
}

/// Tries to establish every condition of `rule` for `subject` — from facts,
/// or recursively from other rules — and, on success, lands the conclusion
/// in the hypothesis world through the gate.
///
/// **Abandoned branches roll back.** A failed attempt restores the
/// hypothesis to its entry state (an O(1) persistent-structure clone), so
/// facts derived along a path that ultimately failed can neither falsify
/// later `isNot`/`isEmpty` evaluations nor prop up a sibling branch's
/// proof — the search explores alternatives, it does not accumulate them.
fn try_rule(
    hypothesis: &mut World,
    rule: &Rule,
    subject: Ref,
    proving: &mut BTreeSet<(Iri, Iri)>,
    depth: usize,
) -> Option<ProofNode> {
    let checkpoint = hypothesis.clone();
    let attempt = (|| {
        let children = prove_tree(hypothesis, subject, &rule.when, proving, depth)?;

        let value = match &rule.then.value {
            Term::Iri(iri) => Value::Ref(Ref::new(*iri)),
            Term::Literal(v) => v.clone(),
        };
        // Even a hypothesis passes the gate: a rule whose conclusion the gate
        // refuses (Example 3's :r9) can establish nothing.
        hypothesis
            .apply_rule_conclusion(subject, rule.then.property, value.clone())
            .ok()?;

        Some(ProofNode {
            kind: ProofKind::Conclusion {
                subject,
                property: rule.then.property,
                value,
                rule: rule.iri,
                explain: rule.explain.clone(),
            },
            children,
        })
    })();
    if attempt.is_none() {
        *hypothesis = checkpoint;
    }
    attempt
}

/// Establishes one `horis:when` subtree, returning the proof children in
/// doc 04's flattened top-level-`all` shape (mirroring the forward side's
/// recorded proofs).
fn prove_tree(
    hypothesis: &mut World,
    subject: Ref,
    tree: &ConditionTree,
    proving: &mut BTreeSet<(Iri, Iri)>,
    depth: usize,
) -> Option<Vec<ProofNode>> {
    match tree {
        ConditionTree::All(children) => {
            let mut proofs = Vec::new();
            for child in children {
                proofs.extend(prove_tree(hypothesis, subject, child, proving, depth)?);
            }
            Some(proofs)
        }
        ConditionTree::Any(children) => children.iter().find_map(|child| {
            // Each failed alternative rolls its derivations back before the
            // next is tried (see `try_rule`'s abandoned-branches contract).
            let checkpoint = hypothesis.clone();
            let attempt = prove_tree(hypothesis, subject, child, proving, depth);
            if attempt.is_none() {
                *hypothesis = checkpoint;
            }
            attempt
        }),
        // Negation is evaluated, never derived toward (module doc).
        ConditionTree::NoneOf(_) => eval::holds(hypothesis, subject, tree)
            .then(|| vec![proof::tree_proof(hypothesis, subject, tree)]),
        ConditionTree::Leaf(condition) => {
            let leaf = prove_condition(hypothesis, subject, condition, proving, depth)?;
            Some(vec![leaf])
        }
    }
}

/// Establishes one leaf condition: from the (hypothesis) facts if it
/// already holds, else by recursively proving a rule-concluded value for
/// its property and re-evaluating.
fn prove_condition(
    hypothesis: &mut World,
    subject: Ref,
    condition: &Condition,
    proving: &mut BTreeSet<(Iri, Iri)>,
    depth: usize,
) -> Option<ProofNode> {
    if eval::condition_holds(hypothesis, subject, condition) {
        return Some(proof::condition_proof(hypothesis, subject, condition));
    }

    // backward.py's "prove unbound variables first": each rule concluding
    // this property offers a candidate value; prove one, then re-evaluate
    // the condition against the enriched hypothesis (the derived value may
    // still fail the operator — roll the attempt back and try the next
    // rule, per the abandoned-branches contract on `try_rule`).
    let ontology = hypothesis.ontology().clone();
    for rule in ontology.rules() {
        if rule.then.property != condition.property {
            continue;
        }
        let candidate = match &rule.then.value {
            Term::Iri(iri) => Value::Ref(Ref::new(*iri)),
            Term::Literal(v) => v.clone(),
        };
        let checkpoint = hypothesis.clone();
        let Some(subproof) = prove_goal(
            hypothesis,
            subject,
            condition.property,
            &candidate,
            proving,
            depth + 1,
        ) else {
            *hypothesis = checkpoint;
            continue;
        };
        if eval::condition_holds(hypothesis, subject, condition) {
            return Some(ProofNode {
                kind: ProofKind::Condition {
                    subject,
                    condition: condition.clone(),
                    held: true,
                    via: ConditionVia::Derived,
                },
                children: vec![subproof],
            });
        }
        *hypothesis = checkpoint;
    }
    None
}

fn value_as_term(value: &Value) -> Term {
    match value {
        Value::Ref(r) => Term::Iri(r.iri()),
        other => Term::Literal(other.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;
    use crate::reason::testing::{PRELUDE, example_world};

    fn t(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/t#{local}"))
    }

    /// PRP Example 2, the provable case: before any acuity exists, could
    /// `:p7 :acuity :Immediate` be established? Yes, via r1 — with the
    /// subsumption hop explicit, and the world left unmutated.
    #[test]
    fn prove_finds_the_r1_derivation_without_mutating_the_world() {
        // Build the world *without* triggering r1: no rules fire for a
        // patient with neither tachycardia nor symptoms recorded... but
        // Example 2 wants exactly p7's facts. Rules fire on assert (U4), so
        // the closest faithful setup is a world whose acuity was derived
        // and then retracted — instead, prove is exercised on a fresh goal
        // r1 does establish: the *hypothetical* question for a patient
        // whose facts arrived without chaining is PRP-004's perception
        // stage; here we retract the derived acuity first (Rule provenance
        // may retract what a rule asserted — PRP-001 U7 semantics).
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        let acuity = t(&mut world, "acuity");
        let immediate = t(&mut world, "Immediate");
        let monitor = t(&mut world, "needsCardiacMonitor");

        let p7 = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(128)),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("accepted");
        // Strip the chained conclusions so the goal is genuinely open.
        world
            .retract_as(
                crate::world::Provenance::Rule,
                p7,
                acuity,
                &Value::Ref(Ref::new(immediate)),
            )
            .expect("a rule may retract a rule-derived fact");
        world
            .retract_as(
                crate::world::Provenance::Rule,
                p7,
                monitor,
                &Value::Boolean(true),
            )
            .expect("retract the monitor too");
        assert_eq!(world.value(p7, acuity), None, "the goal must be open");

        let before = world.clone();
        let proof = prove(&world, p7, acuity, &Value::Ref(Ref::new(immediate)))
            .expect("r1 establishes the goal");

        // The world was not touched.
        assert_eq!(world.value(p7, acuity), None);
        assert_eq!(
            world.instance(p7),
            before.instance(p7),
            "prove is read-only"
        );

        let ProofKind::Conclusion { rule, .. } = &proof.kind else {
            panic!("the goal roots at r1's conclusion");
        };
        assert_eq!(world.resolve(*rule), "https://example.org/t#r1");
        let rendered = proof.render(&world);
        assert!(rendered.contains("via r1"), "{rendered}");
        assert!(
            rendered.contains(":CrushingChestPain ⊑ :ChestPain"),
            "the subsumption hop must be explicit: {rendered}"
        );
    }

    /// PRP Example 2, the failing case: p9 with heartRate 96 — only r1
    /// concludes :Immediate, its first condition is false, nothing else can
    /// establish it → None.
    #[test]
    fn prove_returns_none_when_no_rule_can_establish_the_goal() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let chest_pain = t(&mut world, "ChestPain");
        let acuity = t(&mut world, "acuity");
        let immediate = t(&mut world, "Immediate");

        let p9 = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(96)),
                    (has_symptom, Value::Ref(Ref::new(chest_pain))),
                ],
            )
            .expect("accepted");
        // r2 derived :Urgent during the assert; the :Immediate goal stays
        // unestablishable (r1 needs 96 > 120, false; conflict aside, no
        // other rule concludes :Immediate... except r3-enabled paths that
        // do not exist).
        assert!(
            prove(&world, p9, acuity, &Value::Ref(Ref::new(immediate))).is_none(),
            "96 bpm cannot reach :Immediate"
        );
    }

    /// An already-established fact proves immediately as a leaf — the base
    /// case, answering with how it is asserted.
    #[test]
    fn prove_answers_an_existing_fact_with_a_leaf() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(96))])
            .expect("accepted");

        let proof = prove(&world, p, heart_rate, &Value::Integer(96)).expect("asserted fact");
        let ProofKind::Condition { held, via, .. } = &proof.kind else {
            panic!("an existing fact proves as a leaf");
        };
        assert!(*held);
        assert!(matches!(via, ConditionVia::Asserted { .. }));
    }

    /// A chained goal: proving r3's conclusion requires first proving r1's —
    /// the nested derivation appears as a Derived condition with the
    /// sub-proof as its child.
    #[test]
    fn prove_chains_through_intermediate_rules() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        let acuity = t(&mut world, "acuity");
        let immediate = t(&mut world, "Immediate");
        let monitor = t(&mut world, "needsCardiacMonitor");

        let p7 = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(128)),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("accepted");
        // Open both goals again.
        world
            .retract_as(
                crate::world::Provenance::Rule,
                p7,
                monitor,
                &Value::Boolean(true),
            )
            .expect("retract monitor");
        world
            .retract_as(
                crate::world::Provenance::Rule,
                p7,
                acuity,
                &Value::Ref(Ref::new(immediate)),
            )
            .expect("retract acuity");

        let proof =
            prove(&world, p7, monitor, &Value::Boolean(true)).expect("r3 via r1 establishes it");
        let ProofKind::Conclusion { rule, .. } = &proof.kind else {
            panic!("roots at r3");
        };
        assert_eq!(world.resolve(*rule), "https://example.org/t#r3");

        // Its one condition (acuity is :Immediate) was derived, not asserted.
        let ProofKind::Condition { via, .. } = &proof.children[0].kind else {
            panic!("condition child");
        };
        assert!(matches!(via, ConditionVia::Derived));
        let ProofKind::Conclusion { rule, .. } = &proof.children[0].children[0].kind else {
            panic!("the nested derivation roots at r1");
        };
        assert_eq!(world.resolve(*rule), "https://example.org/t#r1");
    }

    /// Mutually recursive rules terminate via the in-progress goal set —
    /// backward.py's `_proving`, keyed here on (subject, property).
    #[test]
    fn cyclic_rules_terminate_and_fail_cleanly() {
        let source = format!(
            "{PRELUDE}\
             :Patient a rdfs:Class .\n\
             :flagA a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;\n\
             \x20   horis:cardinality \"0..1\" .\n\
             :flagB a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;\n\
             \x20   horis:cardinality \"0..1\" .\n\
             :rA a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :flagB ; horis:is true ] ) ] ;\n\
             \x20 horis:then [ horis:property :flagA ; horis:value true ] .\n\
             :rB a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :flagA ; horis:is true ] ) ] ;\n\
             \x20 horis:then [ horis:property :flagB ; horis:value true ] .\n"
        );
        let onto = Ontology::load_from_str(&source, |_| None).expect("valid");
        let mut world = World::new(&onto);
        let patient = t(&mut world, "Patient");
        let flag_a = t(&mut world, "flagA");

        let p = world.assert_instance(patient, []).expect("accepted");
        assert!(
            prove(&world, p, flag_a, &Value::Boolean(true)).is_none(),
            "the cycle guard must terminate the search unproven"
        );
    }

    /// A rule ladder deeper than MAX_DEPTH is cut off — the depth cap,
    /// carried over from backward.py at 50.
    #[test]
    fn the_depth_cap_bounds_the_search() {
        let mut decls = String::new();
        let mut rules = String::new();
        let levels = MAX_DEPTH + 5;
        for i in 0..=levels {
            decls.push_str(&format!(
                ":lvl{i} a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;\n\
                 \x20   horis:cardinality \"0..1\" .\n"
            ));
        }
        for i in 0..levels {
            let next = i + 1;
            rules.push_str(&format!(
                ":r{i:03} a horis:Rule ; horis:priority 1 ;\n\
                 \x20 horis:when [ horis:all (\n\
                 \x20   [ horis:subject :Patient ; horis:property :lvl{next} ; horis:is true ] ) ] ;\n\
                 \x20 horis:then [ horis:property :lvl{i} ; horis:value true ] .\n"
            ));
        }
        let source = format!("{PRELUDE}:Patient a rdfs:Class .\n{decls}{rules}");
        let onto = Ontology::load_from_str(&source, |_| None).expect("valid");
        let mut world = World::new(&onto);
        let patient = t(&mut world, "Patient");
        let lvl0 = t(&mut world, "lvl0");

        let p = world.assert_instance(patient, []).expect("accepted");
        assert!(
            prove(&world, p, lvl0, &Value::Boolean(true)).is_none(),
            "nothing asserts the ladder's top, and the cap cuts the search"
        );
        // (The positive control — a derivation *within* the cap — is
        // `prove_chains_through_intermediate_rules` above.)
    }
}

#[cfg(test)]
mod branch_rollback_tests {
    use super::*;
    use crate::Ontology;
    use crate::reason::testing::PRELUDE;

    fn t(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/t#{local}"))
    }

    /// The abandoned-branches contract (PRP-002 review finding): a rule
    /// attempt that fails after deriving intermediate facts must roll them
    /// back — here, rA's derivation would otherwise falsify rB's `isEmpty`
    /// condition, making a provable goal unprovable.
    #[test]
    fn a_failed_branch_rolls_back_and_cannot_poison_a_negation() {
        // Goal: :verdict = :Ok. Two deriving rules, tried in priority order:
        //   rA (priority 9): needs :marker = true (derivable via rM) AND
        //       heartRate exceeds 999 (false — the branch fails late,
        //       after rM's marker fact landed in the hypothesis).
        //   rB (priority 5): needs :marker isEmpty — true in the real
        //       world, and true again only if rA's attempt rolled back.
        let source = format!(
            "{PRELUDE}\
             :Patient a rdfs:Class .\n\
             :Verdict a rdfs:Class ; horis:oneOf ( :Ok ) .\n\
             :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;\n\
             \x20   horis:cardinality \"0..1\" .\n\
             :marker a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;\n\
             \x20   horis:cardinality \"0..1\" ; horis:assignedBy horis:RulesOnly .\n\
             :verdict a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Verdict ;\n\
             \x20   horis:cardinality \"0..1\" ; horis:assignedBy horis:RulesOnly .\n\
             :rM a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:isAssigned true ] ) ] ;\n\
             \x20 horis:then [ horis:property :marker ; horis:value true ] .\n\
             :rA a horis:Rule ; horis:priority 9 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :marker ; horis:is true ]\n\
             \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 999 ] ) ] ;\n\
             \x20 horis:then [ horis:property :verdict ; horis:value :Ok ] .\n\
             :rB a horis:Rule ; horis:priority 5 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :marker ; horis:isEmpty true ] ) ] ;\n\
             \x20 horis:then [ horis:property :verdict ; horis:value :Ok ] .\n"
        );
        let onto = Ontology::load_from_str(&source, |_| None).expect("valid");
        let mut world = World::new(&onto);
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let verdict = t(&mut world, "verdict");
        let ok = t(&mut world, "Ok");
        let marker = t(&mut world, "marker");

        // The assertion chains: rB fires first (marker still empty at pass
        // 1) deriving the verdict, then rM sets the marker. Unwind both
        // through the rule path so the world is clean and the goal
        // genuinely open.
        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(80))])
            .expect("accepted");
        world
            .retract_as(
                crate::world::Provenance::Rule,
                p,
                verdict,
                &Value::Ref(Ref::new(ok)),
            )
            .expect("unwind the chained verdict");
        world
            .retract_as(
                crate::world::Provenance::Rule,
                p,
                marker,
                &Value::Boolean(true),
            )
            .expect("unwind the chained marker");
        assert_eq!(world.value(p, marker), None, "marker must be open");
        assert_eq!(world.value(p, verdict), None, "the goal must be open");

        // prove tries rA first (priority 9): rM re-derives marker into the
        // hypothesis, then `exceeds 999` fails. Without rollback, that
        // leftover marker falsifies rB's isEmpty and the goal is wrongly
        // unprovable; with rollback, rB proves it.
        let proof = prove(&world, p, verdict, &Value::Ref(Ref::new(ok)))
            .expect("rB must prove the goal after rA's branch rolls back");
        let ProofKind::Conclusion { rule, .. } = &proof.kind else {
            panic!("roots at a conclusion");
        };
        assert_eq!(
            world.resolve(*rule),
            "https://example.org/t#rB",
            "the proof must come from rB, on a hypothesis rA did not pollute"
        );
    }
}
