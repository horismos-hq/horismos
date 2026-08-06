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
    use crate::ontology::term::Value;
    use crate::reason::testing::example_world;

    fn t(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/t#{local}"))
    }

    fn rule_iri<'a>(world: &'a World, candidate: &Candidate) -> &'a str {
        let rule = &world.ontology().rules()[candidate.rule_index];
        world.ontology().resolve(rule.iri)
    }

    /// The highest-priority rule whose conditions hold wins: r1 (priority
    /// 10) over r2 (5) and r3 (8). (The assertion itself already chained to
    /// fixed point — U4 — so this exercises candidate *selection* on the
    /// settled world, where a fresh transaction would re-offer every rule.)
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

        // The assertion chained to fixed point (U4), so acuity is already
        // :Immediate here — r3's condition holds, and at priority 8 it
        // outranks r2's 5.
        let candidate = matcher
            .next_candidate(&world, &rules, &fired)
            .expect("r3 remains fireable");
        assert_eq!(rule_iri(&world, &candidate), "https://example.org/t#r3");

        let r3 = t(&mut world, "r3");
        fired.insert((r3, p7.iri()));
        let candidate = matcher
            .next_candidate(&world, &rules, &fired)
            .expect("r2 is last");
        assert_eq!(rule_iri(&world, &candidate), "https://example.org/t#r2");
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
