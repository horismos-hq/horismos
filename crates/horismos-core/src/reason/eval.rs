//! Condition evaluation (U2): the 11 operators and 3 combinators evaluated
//! over the instance graph, with the documented pragmatic closed-world edge
//! semantics (doc 04's table):
//!
//! | Situation | Result |
//! |-----------|--------|
//! | Property unassigned, any comparison (`exceeds`, `is`, …) | `false` |
//! | Property unassigned, `horis:isNot` | `true` |
//! | Property unassigned, `horis:isAssigned` | `false` |
//! | Multi-valued property, `horis:is X` | `true` if **any** value matches (∃) |
//! | Multi-valued property, `horis:isNot X` | `true` iff **no** value matches (∀) |
//!
//! The ∀ reading of `isNot` is D-4's resolution: `isNot X` ≡ ¬(`is X`) in
//! every world state — the only reading consistent with the unassigned row
//! (an unassigned property has no witness value, so ∃ would contradict
//! `isNot` = true) and with sigil-lang's `_UNBOUND` precedent.
//!
//! `is X` is subsumption-aware, `owl:sameAs`-canonically (doc 04's "matches
//! any subclass", D-7): a `Ref` value matches `X` when it *is* `X`, when it
//! names a declared class that is a subclass of `X`, or when it names a world
//! instance whose class is a subclass of `X` — the same three shapes the
//! gate's own range check admits.
//!
//! Spec: `docs/framework/04-reasoning.md` · Owner: PRP-002

use std::cmp::Ordering;

use rust_decimal::Decimal;

use crate::World;
use crate::ontology::term::{Iri, Ref, Value};
use crate::reason::rule::{Condition, ConditionTree, Op, Term};

/// Evaluates a whole `horis:when` tree for `subject`. Combinator identities:
/// `All([])` is `true`, `Any([])` is `false`, `NoneOf([])` is `true` (¬∨ of
/// nothing).
pub fn holds(world: &World, subject: Ref, tree: &ConditionTree) -> bool {
    match tree {
        ConditionTree::Leaf(condition) => condition_holds(world, subject, condition),
        ConditionTree::All(children) => children.iter().all(|c| holds(world, subject, c)),
        ConditionTree::Any(children) => children.iter().any(|c| holds(world, subject, c)),
        ConditionTree::NoneOf(children) => !children.iter().any(|c| holds(world, subject, c)),
    }
}

/// Evaluates one leaf condition for `subject`, per the module-level table.
pub fn condition_holds(world: &World, subject: Ref, condition: &Condition) -> bool {
    let values: Vec<&Value> = world.values(subject, condition.property).collect();

    match &condition.op {
        // isNot ≡ ¬is (D-4): the ∀ negation of the ∃ per-value relation —
        // structural, not re-implemented; the property test pins it.
        Op::IsNot(term) => !values.iter().any(|v| term_matches(world, v, term)),
        Op::IsEmpty => values.is_empty(),
        // Every remaining operator is ∃ over values (unassigned → false —
        // there is nothing to witness it), including `isAssigned`, which any
        // value at all witnesses.
        op => values.iter().any(|v| value_satisfies(world, v, op)),
    }
}

/// Does this single stored value witness `op`? The per-value ∃ relation
/// `condition_holds` quantifies over — also the proof builder's witness
/// finder (U4). The ∀/absence-shaped operators (`isNot`, `isEmpty`) have no
/// per-value witness and answer `false`.
pub(crate) fn value_satisfies(world: &World, value: &Value, op: &Op) -> bool {
    match op {
        Op::IsNot(_) | Op::IsEmpty => false,
        Op::IsAssigned => true,
        Op::Is(term) => term_matches(world, value, term),
        Op::Exceeds(bound) => compare(value, bound) == Some(Ordering::Greater),
        Op::Below(bound) => compare(value, bound) == Some(Ordering::Less),
        Op::AtLeast(bound) => {
            matches!(
                compare(value, bound),
                Some(Ordering::Greater | Ordering::Equal)
            )
        }
        Op::AtMost(bound) => {
            matches!(
                compare(value, bound),
                Some(Ordering::Less | Ordering::Equal)
            )
        }
        Op::Between(low, high) => {
            matches!(
                compare(value, low),
                Some(Ordering::Greater | Ordering::Equal)
            ) && matches!(compare(value, high), Some(Ordering::Less | Ordering::Equal))
        }
        Op::Contains(term) => contains_matches(world, value, term),
        Op::InList(terms) => terms.iter().any(|t| term_matches(world, value, t)),
    }
}

/// The `is` match relation between one stored value and one condition term.
fn term_matches(world: &World, value: &Value, term: &Term) -> bool {
    match (value, term) {
        (Value::Ref(r), Term::Iri(target)) => ref_matches(world, *r, *target),
        (value, Term::Literal(expected)) => value_eq(value, expected),
        // A literal value never matches an IRI term.
        (_, Term::Iri(_)) => false,
    }
}

/// `horis:contains`: substring on string/string, else the `is` relation on
/// any value (∃ membership).
fn contains_matches(world: &World, value: &Value, term: &Term) -> bool {
    if let (Value::String(s), Term::Literal(Value::String(needle))) = (value, term) {
        return s.contains(needle.as_str());
    }
    term_matches(world, value, term)
}

/// Does the node `r` count as `target`? Three shapes, all
/// `owl:sameAs`-canonical (D-7), mirroring the gate's range acceptance:
/// exact/canonical identity; a declared class that subsumes into `target`;
/// a world instance whose class subsumes into `target`.
fn ref_matches(world: &World, r: Ref, target: Iri) -> bool {
    let schema = world.ontology().schema();
    if r.iri() == target {
        return true;
    }
    if let (Some(a), Some(b)) = (
        schema.class_declared(r.iri()),
        schema.class_declared(target),
    ) && schema.is_subclass_of(a, b)
    {
        return true;
    }
    if let Some(instance) = world.instance(r)
        && schema.is_subclass_of(instance.class(), target)
    {
        return true;
    }
    // Non-class, non-instance IRIs (enum members like :Immediate) still
    // deserve canonical identity: sameAs twins are the same individual.
    schema.canonical_iri(r.iri()) == schema.canonical_iri(target)
}

/// Value equality with numeric cross-typing: `Integer 5` equals
/// `Decimal 5.0` (an integer *is* a decimal in XSD's derivation hierarchy).
fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Integer(x), Value::Decimal(y)) | (Value::Decimal(y), Value::Integer(x)) => {
            Decimal::from(*x) == *y
        }
        _ => a == b,
    }
}

/// Ordering for the comparison operators: numeric cross-type
/// (Integer/Decimal), `xsd:dateTime`, and lexicographic strings (sigil-lang
/// `compare_values` precedent). Booleans, durations (partially ordered), and
/// node references are incomparable → `None` → the comparison is `false`.
fn compare(value: &Value, bound: &Value) -> Option<Ordering> {
    match (value, bound) {
        (Value::Integer(a), Value::Integer(b)) => Some(a.cmp(b)),
        (Value::Decimal(a), Value::Decimal(b)) => Some(a.cmp(b)),
        (Value::Integer(a), Value::Decimal(b)) => Some(Decimal::from(*a).cmp(b)),
        (Value::Decimal(a), Value::Integer(b)) => Some(a.cmp(&Decimal::from(*b))),
        (Value::DateTime(a), Value::DateTime(b)) => Some(a.cmp(b)),
        (Value::String(a), Value::String(b)) => Some(a.as_str().cmp(b.as_str())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;
    use crate::ontology::term::Iri;
    use crate::reason::rule::{Condition, Op, Term};
    use proptest::prelude::*;
    use rust_decimal::Decimal;

    fn triage_world() -> World {
        let onto = Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl must load");
        World::new(&onto)
    }

    fn tri(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/triage#{local}"))
    }

    fn cond(property: Iri, op: Op) -> Condition {
        Condition {
            property,
            op,
            unit: None,
        }
    }

    /// PRP Example 1 pass 1, first condition: `heartRate exceeds 120` holds
    /// for p7 (128) and not for a 96-bpm patient.
    #[test]
    fn exceeds_holds_for_128_over_120_and_not_for_96() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p7 = world
            .assert_instance(patient, [(heart_rate, Value::Integer(128))])
            .expect("accepted");
        let p9 = world
            .assert_instance(patient, [(heart_rate, Value::Integer(96))])
            .expect("accepted");

        let c = cond(heart_rate, Op::Exceeds(Value::Integer(120)));
        assert!(condition_holds(&world, p7, &c));
        assert!(!condition_holds(&world, p9, &c));
    }

    /// PRP Example 1's subsumption hop: `hasSymptom is :ChestPain` holds for
    /// a patient whose symptom is :CrushingChestPain — a class no rule
    /// mentions.
    #[test]
    fn is_matches_a_subclass_value_via_subsumption() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let has_symptom = tri(&mut world, "hasSymptom");
        let crushing = tri(&mut world, "CrushingChestPain");
        let chest_pain = tri(&mut world, "ChestPain");
        let symptom = tri(&mut world, "Symptom");
        let p = world
            .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(crushing)))])
            .expect("accepted");

        assert!(condition_holds(
            &world,
            p,
            &cond(has_symptom, Op::Is(Term::Iri(chest_pain)))
        ));
        assert!(condition_holds(
            &world,
            p,
            &cond(has_symptom, Op::Is(Term::Iri(symptom))),
        ));
        assert!(condition_holds(
            &world,
            p,
            &cond(has_symptom, Op::Is(Term::Iri(crushing)))
        ));
    }

    #[test]
    fn is_matches_an_instance_value_through_its_class() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let has_symptom = tri(&mut world, "hasSymptom");
        let crushing = tri(&mut world, "CrushingChestPain");
        let chest_pain = tri(&mut world, "ChestPain");

        let symptom_instance = world
            .assert_instance(crushing, [])
            .expect("a CrushingChestPain instance");
        let p = world
            .assert_instance(patient, [(has_symptom, Value::Ref(symptom_instance))])
            .expect("accepted");

        assert!(condition_holds(
            &world,
            p,
            &cond(has_symptom, Op::Is(Term::Iri(chest_pain)))
        ));
    }

    #[test]
    fn is_does_not_match_a_sibling_or_superclass_downward() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let has_symptom = tri(&mut world, "hasSymptom");
        let chest_pain = tri(&mut world, "ChestPain");
        let crushing = tri(&mut world, "CrushingChestPain");
        let p = world
            .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(chest_pain)))])
            .expect("accepted");

        assert!(
            !condition_holds(&world, p, &cond(has_symptom, Op::Is(Term::Iri(crushing)))),
            "subsumption must not hold downward: a ChestPain is not a CrushingChestPain"
        );
    }

    // -- Doc 04's edge-semantics table, row by row --------------------------

    #[test]
    fn an_unassigned_property_fails_every_comparison() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let chest_pain = tri(&mut world, "ChestPain");
        let has_symptom = tri(&mut world, "hasSymptom");
        let p = world.assert_instance(patient, []).expect("accepted");

        for op in [
            Op::Exceeds(Value::Integer(0)),
            Op::Below(Value::Integer(1000)),
            Op::AtLeast(Value::Integer(0)),
            Op::AtMost(Value::Integer(1000)),
            Op::Between(Value::Integer(0), Value::Integer(1000)),
        ] {
            assert!(
                !condition_holds(&world, p, &cond(heart_rate, op.clone())),
                "unassigned heartRate must fail {}",
                op.name()
            );
        }
        assert!(!condition_holds(
            &world,
            p,
            &cond(has_symptom, Op::Is(Term::Iri(chest_pain)))
        ));
        assert!(!condition_holds(
            &world,
            p,
            &cond(has_symptom, Op::InList(vec![Term::Iri(chest_pain)]))
        ));
        assert!(!condition_holds(
            &world,
            p,
            &cond(has_symptom, Op::Contains(Term::Iri(chest_pain)))
        ));
    }

    #[test]
    fn an_unassigned_property_satisfies_is_not() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let has_symptom = tri(&mut world, "hasSymptom");
        let chest_pain = tri(&mut world, "ChestPain");
        let p = world.assert_instance(patient, []).expect("accepted");

        assert!(condition_holds(
            &world,
            p,
            &cond(has_symptom, Op::IsNot(Term::Iri(chest_pain)))
        ));
    }

    #[test]
    fn is_assigned_and_is_empty_are_complements() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let empty = world.assert_instance(patient, []).expect("accepted");
        let full = world
            .assert_instance(patient, [(heart_rate, Value::Integer(80))])
            .expect("accepted");

        assert!(!condition_holds(
            &world,
            empty,
            &cond(heart_rate, Op::IsAssigned)
        ));
        assert!(condition_holds(
            &world,
            empty,
            &cond(heart_rate, Op::IsEmpty)
        ));
        assert!(condition_holds(
            &world,
            full,
            &cond(heart_rate, Op::IsAssigned)
        ));
        assert!(!condition_holds(
            &world,
            full,
            &cond(heart_rate, Op::IsEmpty)
        ));
    }

    /// D-4's worked case: hasSymptom = {CrushingChestPain, Nausea-shaped
    /// sibling}. `isNot :ChestPain` is ∀ — false, because one value IS a
    /// ChestPain.
    #[test]
    fn multi_valued_is_not_is_the_forall_reading() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let has_symptom = tri(&mut world, "hasSymptom");
        let crushing = tri(&mut world, "CrushingChestPain");
        let chest_pain = tri(&mut world, "ChestPain");
        let symptom = tri(&mut world, "Symptom");

        // :Symptom itself is a declared class, usable as a non-ChestPain value.
        let p = world
            .assert_instance(
                patient,
                [
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                    (has_symptom, Value::Ref(Ref::new(symptom))),
                ],
            )
            .expect("accepted");

        assert!(
            !condition_holds(
                &world,
                p,
                &cond(has_symptom, Op::IsNot(Term::Iri(chest_pain)))
            ),
            "∀ reading: some value IS a ChestPain, so isNot is false"
        );
        assert!(
            condition_holds(&world, p, &cond(has_symptom, Op::Is(Term::Iri(chest_pain)))),
            "∃ reading of is: some value matches"
        );
    }

    // -- Value semantics -----------------------------------------------------

    #[test]
    fn numeric_comparison_is_cross_typed_between_integer_and_decimal() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p = world
            .assert_instance(
                patient,
                [(heart_rate, Value::Decimal(Decimal::new(1285, 1)))], // 128.5
            )
            .expect("accepted");

        assert!(condition_holds(
            &world,
            p,
            &cond(heart_rate, Op::Exceeds(Value::Integer(128)))
        ));
        assert!(condition_holds(
            &world,
            p,
            &cond(
                heart_rate,
                Op::Between(Value::Integer(100), Value::Integer(129))
            )
        ));
        assert!(condition_holds(
            &world,
            p,
            &cond(
                heart_rate,
                Op::AtMost(Value::Decimal(Decimal::new(1285, 1)))
            )
        ));
    }

    #[test]
    fn between_is_inclusive_on_both_bounds() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(120))])
            .expect("accepted");

        assert!(condition_holds(
            &world,
            p,
            &cond(
                heart_rate,
                Op::Between(Value::Integer(120), Value::Integer(140))
            )
        ));
        assert!(condition_holds(
            &world,
            p,
            &cond(
                heart_rate,
                Op::Between(Value::Integer(100), Value::Integer(120))
            )
        ));
        assert!(!condition_holds(
            &world,
            p,
            &cond(
                heart_rate,
                Op::Between(Value::Integer(121), Value::Integer(140))
            )
        ));
    }

    #[test]
    fn in_list_matches_membership_with_subsumption() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let has_symptom = tri(&mut world, "hasSymptom");
        let crushing = tri(&mut world, "CrushingChestPain");
        let chest_pain = tri(&mut world, "ChestPain");
        let p = world
            .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(crushing)))])
            .expect("accepted");

        assert!(condition_holds(
            &world,
            p,
            &cond(has_symptom, Op::InList(vec![Term::Iri(chest_pain)]))
        ));
    }

    #[test]
    fn combinators_compose_with_documented_identities() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(128))])
            .expect("accepted");

        let above = ConditionTree::Leaf(cond(heart_rate, Op::Exceeds(Value::Integer(120))));
        let below = ConditionTree::Leaf(cond(heart_rate, Op::Below(Value::Integer(100))));

        assert!(holds(&world, p, &ConditionTree::All(vec![above.clone()])));
        assert!(!holds(
            &world,
            p,
            &ConditionTree::All(vec![above.clone(), below.clone()])
        ));
        assert!(holds(
            &world,
            p,
            &ConditionTree::Any(vec![below.clone(), above.clone()])
        ));
        assert!(holds(
            &world,
            p,
            &ConditionTree::NoneOf(vec![below.clone()])
        ));
        assert!(!holds(
            &world,
            p,
            &ConditionTree::NoneOf(vec![above.clone()])
        ));

        // Identities: All([]) = true, Any([]) = false, NoneOf([]) = true.
        assert!(holds(&world, p, &ConditionTree::All(vec![])));
        assert!(!holds(&world, p, &ConditionTree::Any(vec![])));
        assert!(holds(&world, p, &ConditionTree::NoneOf(vec![])));
    }

    #[test]
    fn contains_does_substring_on_strings() {
        let mut world = World::new(
            &Ontology::load_from_str(
                "@prefix : <https://example.org/t#> .\n\
                 @prefix horis: <https://w3id.org/horismos#> .\n\
                 @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
                 @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                 @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
                 :Note a rdfs:Class .\n\
                 :text a rdf:Property ; rdfs:domain :Note ; rdfs:range xsd:string .\n",
                |_| None,
            )
            .expect("valid"),
        );
        let note = world.intern("https://example.org/t#Note");
        let text = world.intern("https://example.org/t#text");
        let n = world
            .assert_instance(note, [(text, Value::String("crushing chest pain".into()))])
            .expect("accepted");

        assert!(condition_holds(
            &world,
            n,
            &cond(
                text,
                Op::Contains(Term::Literal(Value::String("chest".into())))
            )
        ));
        assert!(!condition_holds(
            &world,
            n,
            &cond(
                text,
                Op::Contains(Term::Literal(Value::String("fracture".into())))
            )
        ));
    }

    // -- D-4 pinned property-style: isNot ≡ ¬is, isEmpty ≡ ¬isAssigned -----

    proptest! {
        /// D-4's algebra pin: for arbitrary small symptom sets (including
        /// empty), `isNot X` answers exactly `!is X`, and `isEmpty` answers
        /// exactly `!isAssigned`.
        #[test]
        fn is_not_is_the_negation_of_is(symptom_picks in proptest::collection::vec(0usize..3, 0..3)) {
            let mut world = triage_world();
            let patient = tri(&mut world, "Patient");
            let has_symptom = tri(&mut world, "hasSymptom");
            let classes = [
                tri(&mut world, "CrushingChestPain"),
                tri(&mut world, "ChestPain"),
                tri(&mut world, "Symptom"),
            ];
            let chest_pain = classes[1];

            let properties: Vec<_> = symptom_picks
                .iter()
                .map(|&i| (has_symptom, Value::Ref(Ref::new(classes[i]))))
                .collect();
            let p = world
                .assert_instance(patient, properties)
                .expect("all three classes are valid Symptom values");

            let is = condition_holds(&world, p, &cond(has_symptom, Op::Is(Term::Iri(chest_pain))));
            let is_not = condition_holds(&world, p, &cond(has_symptom, Op::IsNot(Term::Iri(chest_pain))));
            prop_assert_eq!(is_not, !is, "isNot must be the ∀ negation of is");

            let assigned = condition_holds(&world, p, &cond(has_symptom, Op::IsAssigned));
            let empty = condition_holds(&world, p, &cond(has_symptom, Op::IsEmpty));
            prop_assert_eq!(empty, !assigned);
        }
    }
}
