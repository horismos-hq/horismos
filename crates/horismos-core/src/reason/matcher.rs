//! Candidate matching (U3/U8): which (rule, subject) may fire next.
//!
//! Two implementations behind one trait (PRP-002 Key decision 1): the
//! [`NaiveMatcher`], which re-evaluates every rule against the world each
//! pass — trivially correct, the differential-testing oracle — and the
//! Rete-lite network (U8), which keeps per-condition alpha memories keyed on
//! class + property so one assertion re-checks only the rules that mention
//! it. Both produce the *identical* candidate sequence; the property tests
//! pin that.
//!
//! Candidate order is doc 04's, exactly: priority descending, ties by rule
//! IRI lexicographic (the rules slice arrives pre-sorted that way from
//! [`crate::Ontology::rules`]), and — when one rule matches several subjects
//! — subject token order, which is first-intern order and therefore
//! deterministic (CLAUDE.md: never hash-iteration order, never load order
//! beyond the document itself).
//!
//! Spec: `docs/framework/04-reasoning.md` · Owner: PRP-002

use std::collections::BTreeSet;

use crate::World;
use crate::ontology::term::{Iri, Ref};
use crate::reason::eval::holds;
use crate::reason::rule::Rule;

/// The (rule IRI, subject IRI) pairs that already fired this transaction —
/// doc 04's no-re-fire guard. A pair enters the set when the rule fires,
/// whether its conclusion was derived, suppressed, or rejected.
pub type FiredSet = BTreeSet<(Iri, Iri)>;

/// One fireable match: an index into the pre-sorted rules slice, and the
/// bound subject instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate {
    pub rule_index: usize,
    pub subject: Ref,
}

/// The matching strategy behind forward chaining. Implementations must
/// return byte-identical candidate sequences for identical inputs — the
/// strategy is a performance choice, never a semantics choice.
pub trait Matcher {
    /// The highest-priority (rule, subject) whose conditions hold in
    /// `world` and which is not in `fired`, in the documented deterministic
    /// order; `None` at fixed point.
    fn next_candidate(
        &mut self,
        world: &World,
        rules: &[Rule],
        fired: &FiredSet,
    ) -> Option<Candidate>;

    /// A fact landed on `subject` (a fired conclusion committed). Incremental
    /// matchers refresh their memories here; the naive matcher ignores it.
    fn notify_asserted(&mut self, world: &World, rules: &[Rule], subject: Ref, property: Iri) {
        let _ = (world, rules, subject, property);
    }
}

/// The oracle: full re-evaluation, no state. O(rules × instances) per call
/// and proudly so — every incremental matcher is tested against it.
#[derive(Debug, Default)]
pub struct NaiveMatcher;

impl NaiveMatcher {
    pub fn new() -> Self {
        Self
    }
}

impl Matcher for NaiveMatcher {
    fn next_candidate(
        &mut self,
        world: &World,
        rules: &[Rule],
        fired: &FiredSet,
    ) -> Option<Candidate> {
        for (rule_index, rule) in rules.iter().enumerate() {
            for subject in world.instances(rule.subject_class) {
                if fired.contains(&(rule.iri, subject.iri())) {
                    continue;
                }
                if holds(world, subject, &rule.when) {
                    return Some(Candidate {
                        rule_index,
                        subject,
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;
    use crate::ontology::term::Value;

    const PRELUDE: &str = "@prefix : <https://example.org/t#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    /// PRP Example 1's illustrative r2/r3 alongside the r1 shape, on an
    /// inline patient schema (r2/r3 are illustrative only — not fixture
    /// changes).
    const EXAMPLE_1: &str = ":Patient a rdfs:Class .\n\
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
         :r1 a horis:Rule ; horis:priority 10 ;\n\
         \x20 horis:when [ horis:all (\n\
         \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ]\n\
         \x20   [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ]\n\
         \x20 ) ] ;\n\
         \x20 horis:then [ horis:property :acuity ; horis:value :Immediate ] ;\n\
         \x20 horis:explain \"Tachycardia with chest pain — immediate track\" .\n\
         :r2 a horis:Rule ; horis:priority 5 ;\n\
         \x20 horis:when [ horis:all (\n\
         \x20   [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ]\n\
         \x20 ) ] ;\n\
         \x20 horis:then [ horis:property :acuity ; horis:value :Urgent ] ;\n\
         \x20 horis:explain \"Chest pain — at least urgent\" .\n\
         :r3 a horis:Rule ; horis:priority 8 ;\n\
         \x20 horis:when [ horis:all (\n\
         \x20   [ horis:subject :Patient ; horis:property :acuity ; horis:is :Immediate ]\n\
         \x20 ) ] ;\n\
         \x20 horis:then [ horis:property :needsCardiacMonitor ; horis:value true ] ;\n\
         \x20 horis:explain \"Immediate track — continuous cardiac monitoring\" .\n";

    fn example_world() -> World {
        let onto = Ontology::load_from_str(&format!("{PRELUDE}{EXAMPLE_1}"), |_| None)
            .expect("the Example 1 ontology must load");
        World::new(&onto)
    }

    fn t(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/t#{local}"))
    }

    fn rule_iri<'a>(world: &'a World, candidate: &Candidate) -> &'a str {
        let rule = &world.ontology().rules()[candidate.rule_index];
        world.ontology().resolve(rule.iri)
    }

    /// PRP Example 1, pass 1: r1 and r2 are candidates (subsumption match),
    /// r3 is not (acuity unassigned → comparison false). Highest priority
    /// candidate is r1.
    #[test]
    fn pass_one_picks_r1_over_r2_and_excludes_r3() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        let p7 = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(128)),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("accepted");

        let rules = world.ontology().rules().to_vec();
        let mut matcher = NaiveMatcher::new();
        let fired = FiredSet::new();

        let candidate = matcher
            .next_candidate(&world, &rules, &fired)
            .expect("r1 must be fireable");
        assert_eq!(rule_iri(&world, &candidate), "https://example.org/t#r1");
        assert_eq!(candidate.subject, p7);
    }

    #[test]
    fn a_fired_pair_is_excluded_and_the_next_priority_takes_over() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        let r1 = t(&mut world, "r1");
        let p7 = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(128)),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("accepted");

        let rules = world.ontology().rules().to_vec();
        let mut matcher = NaiveMatcher::new();
        let mut fired = FiredSet::new();
        fired.insert((r1, p7.iri()));

        let candidate = matcher
            .next_candidate(&world, &rules, &fired)
            .expect("r2 remains fireable");
        assert_eq!(
            rule_iri(&world, &candidate),
            "https://example.org/t#r2",
            "with (r1, p7) fired, priority 5 r2 is the best remaining candidate (r3 needs acuity)"
        );
    }

    #[test]
    fn no_candidates_is_the_fixed_point() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(96))])
            .expect("accepted");
        let _ = p;

        let rules = world.ontology().rules().to_vec();
        let mut matcher = NaiveMatcher::new();
        let fired = FiredSet::new();
        assert_eq!(
            matcher.next_candidate(&world, &rules, &fired),
            None,
            "96 bpm and no symptoms fires nothing"
        );
    }

    #[test]
    fn one_rule_matching_two_subjects_yields_them_in_intern_order() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let has_symptom = t(&mut world, "hasSymptom");
        let chest_pain = t(&mut world, "ChestPain");
        let r1 = t(&mut world, "r1");
        let symptom_value = Value::Ref(Ref::new(chest_pain));

        let first = world
            .assert_instance(patient, [(has_symptom, symptom_value.clone())])
            .expect("accepted");
        let second = world
            .assert_instance(patient, [(has_symptom, symptom_value)])
            .expect("accepted");

        let rules = world.ontology().rules().to_vec();
        let mut matcher = NaiveMatcher::new();
        let mut fired = FiredSet::new();
        // Neither patient is tachycardic, so r2 (priority 5) is the top rule.
        let a = matcher
            .next_candidate(&world, &rules, &fired)
            .expect("candidate");
        assert_eq!(rule_iri(&world, &a), "https://example.org/t#r2");
        assert_eq!(a.subject, first, "first-interned subject first");

        let r2 = t(&mut world, "r2");
        let _ = r1;
        fired.insert((r2, first.iri()));
        let b = matcher
            .next_candidate(&world, &rules, &fired)
            .expect("candidate");
        assert_eq!(b.subject, second);
    }
}
