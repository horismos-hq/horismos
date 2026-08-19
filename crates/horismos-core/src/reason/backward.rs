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

    /// A variant of Example 1 with an added spoiler rule `:r0` (priority 20,
    /// outranking `:r1`'s 10): whenever `heartRate` is asserted at all,
    /// `:r0` claims the single-valued `acuity` slot with `:Standard` before
    /// the conflict policy ever gets to `:r1`, so `:r1`'s `:Immediate`
    /// conclusion — though its own conditions genuinely hold — never
    /// actually stands in a *real*, forward-chained world.
    ///
    /// Truth maintenance (doc 03 § Truth maintenance) means a world built
    /// through the public API is always already at its fixed point, so a
    /// goal that *did* stand can only be re-proved as a same-value leaf,
    /// never walked through [`ProofKind::Conclusion`] — the "derive then
    /// manually retract the conclusion" setup these tests used before is no
    /// longer reachable (retraction now re-chains too). Spoiling the real
    /// winner is what keeps `:r1`/`:r3` individually provable by backward
    /// search while their conclusions stay genuinely absent from the real
    /// world.
    fn spoiled_world() -> World {
        let source = format!(
            "{PRELUDE}\
             :Patient a rdfs:Class .\n\
             :Symptom a rdfs:Class .\n\
             :ChestPain rdfs:subClassOf :Symptom .\n\
             :CrushingChestPain rdfs:subClassOf :ChestPain .\n\
             :AcuityLevel a rdfs:Class ; horis:oneOf ( :Immediate :Urgent :Standard ) .\n\
             :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;\n\
             \x20   horis:unit \"bpm\" ; horis:min 0 ; horis:max 300 ; horis:cardinality \"0..1\" .\n\
             :hasSymptom a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Symptom ;\n\
             \x20   horis:cardinality \"0..*\" .\n\
             :acuity a rdf:Property ; rdfs:domain :Patient ; rdfs:range :AcuityLevel ;\n\
             \x20   horis:cardinality \"0..1\" ; horis:assignedBy horis:RulesOnly .\n\
             :needsCardiacMonitor a rdf:Property ; rdfs:domain :Patient ;\n\
             \x20   rdfs:range xsd:boolean ; horis:cardinality \"0..1\" ;\n\
             \x20   horis:assignedBy horis:RulesOnly .\n\
             :r0 a horis:Rule ; horis:priority 20 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:isAssigned true ]\n\
             \x20 ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Standard ] ;\n\
             \x20 horis:explain \"Baseline — outranks r1 so the real world never settles on :Immediate\" .\n\
             :r1 a horis:Rule ; horis:priority 10 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ]\n\
             \x20   [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ]\n\
             \x20 ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Immediate ] ;\n\
             \x20 horis:explain \"Tachycardia with chest pain — immediate track\" .\n\
             :r3 a horis:Rule ; horis:priority 8 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :acuity ; horis:is :Immediate ]\n\
             \x20 ) ] ;\n\
             \x20 horis:then [ horis:property :needsCardiacMonitor ; horis:value true ] ;\n\
             \x20 horis:explain \"Immediate track — continuous cardiac monitoring\" .\n"
        );
        let onto = Ontology::load_from_str(&source, |_| None).expect("valid");
        World::new(&onto)
    }

    /// PRP Example 2, the provable case: could `:p7 :acuity :Immediate` be
    /// established? Yes, via r1 — with the subsumption hop explicit, and
    /// the world left unmutated — even though the *real* world settled on
    /// r0's `:Standard` (see [`spoiled_world`]).
    #[test]
    fn prove_finds_the_r1_derivation_without_mutating_the_world() {
        let mut world = spoiled_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        let acuity = t(&mut world, "acuity");
        let immediate = t(&mut world, "Immediate");
        let standard = t(&mut world, "Standard");

        let p7 = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(128)),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("accepted");
        assert_eq!(
            world.value(p7, acuity),
            Some(&Value::Ref(Ref::new(standard))),
            "sanity: r0 must win the real slot, not r1"
        );

        let before = world.clone();
        let proof = prove(&world, p7, acuity, &Value::Ref(Ref::new(immediate)))
            .expect("r1 establishes the goal hypothetically");

        // The world was not touched.
        assert_eq!(
            world.value(p7, acuity),
            Some(&Value::Ref(Ref::new(standard)))
        );
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
    /// sub-proof as its child. Both stay off the *real* fixed point (r0
    /// spoils `acuity`, see [`spoiled_world`]), so this also exercises
    /// derived-from-derived truth maintenance: neither goal can be answered
    /// by the top-level leaf shortcut.
    #[test]
    fn prove_chains_through_intermediate_rules() {
        let mut world = spoiled_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
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
        assert_eq!(
            world.value(p7, monitor),
            None,
            "sanity: r3 never fires for real — acuity is r0's :Standard, not :Immediate"
        );

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

        // prove is read-only: the real world never saw either conclusion.
        assert_eq!(world.value(p7, monitor), None);
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
    /// back — here, rA's derivation would otherwise falsify rB's `isNot`
    /// condition, making a provable goal unprovable.
    ///
    /// Under truth maintenance (doc 03 § Truth maintenance) the "assert,
    /// then manually retract-as-Rule the chained conclusions" setup this
    /// test used before no longer produces an open goal — retraction now
    /// re-chains and simply re-derives them. Two spoilers keep the *real*
    /// world settled somewhere `prove`'s rule search must still walk past:
    /// `rSpoiler` outranks both `rA`/`rB` for `verdict`, and `rSuppress`
    /// outranks `rM` for `marker`, so the real world settles on
    /// `marker = false`, never `:marker`'s absence — which is exactly what
    /// lets `rB`'s `isNot true` (true whenever unset *or* assigned a
    /// different value) stand in for the old `isEmpty` check.
    #[test]
    fn a_failed_branch_rolls_back_and_cannot_poison_a_negation() {
        // Goal: :verdict = :Ok. Two deriving rules, tried in priority order:
        //   rA (priority 9): needs :marker = true (derivable via rM) AND
        //       heartRate exceeds 999 (false — the branch fails late,
        //       after rM's marker fact landed in the hypothesis).
        //   rB (priority 5): needs :marker isNot true — true for real
        //       (rSuppress won it as false), and true again only if rA's
        //       attempt rolled back its hypothetical marker = true.
        let source = format!(
            "{PRELUDE}\
             :Patient a rdfs:Class .\n\
             :Verdict a rdfs:Class ; horis:oneOf ( :Ok :Other ) .\n\
             :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;\n\
             \x20   horis:cardinality \"0..1\" .\n\
             :marker a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;\n\
             \x20   horis:cardinality \"0..1\" ; horis:assignedBy horis:RulesOnly .\n\
             :verdict a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Verdict ;\n\
             \x20   horis:cardinality \"0..1\" ; horis:assignedBy horis:RulesOnly .\n\
             :rSpoiler a horis:Rule ; horis:priority 99 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:isAssigned true ] ) ] ;\n\
             \x20 horis:then [ horis:property :verdict ; horis:value :Other ] .\n\
             :rSuppress a horis:Rule ; horis:priority 50 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:isAssigned true ] ) ] ;\n\
             \x20 horis:then [ horis:property :marker ; horis:value false ] .\n\
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
             \x20   [ horis:subject :Patient ; horis:property :marker ; horis:isNot true ] ) ] ;\n\
             \x20 horis:then [ horis:property :verdict ; horis:value :Ok ] .\n"
        );
        let onto = Ontology::load_from_str(&source, |_| None).expect("valid");
        let mut world = World::new(&onto);
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let verdict = t(&mut world, "verdict");
        let ok = t(&mut world, "Ok");
        let other = t(&mut world, "Other");
        let marker = t(&mut world, "marker");

        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(80))])
            .expect("accepted");
        // Sanity: the real fixed point never reaches :Ok — rSpoiler and
        // rSuppress win their slots, so rA/rB/rM are suppressed for real.
        assert_eq!(
            world.value(p, verdict),
            Some(&Value::Ref(Ref::new(other))),
            "rSpoiler must win the real verdict slot"
        );
        assert_eq!(
            world.value(p, marker),
            Some(&Value::Boolean(false)),
            "rSuppress must win the real marker slot"
        );

        // prove tries rA first (priority 9): rM re-derives marker into the
        // hypothesis, then `exceeds 999` fails. Without rollback, that
        // leftover marker falsifies rB's `isNot true` and the goal is
        // wrongly unprovable; with rollback, rB proves it.
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

        // prove is read-only: the real world's suppressed slots are untouched.
        assert_eq!(world.value(p, verdict), Some(&Value::Ref(Ref::new(other))));
        assert_eq!(world.value(p, marker), Some(&Value::Boolean(false)));
    }
}
