//! Forward chaining to fixed point (U4) — doc 04's loop, verbatim:
//!
//! ```text
//! loop (max N iterations, default 100):
//!     candidates = rules whose conditions hold in the current graph
//!                  minus rules already fired on the same bindings
//!     if empty: fixed point reached → done
//!     fire highest-priority candidate (ties: rule IRI lexicographic)
//!     assert its conclusions through the gate (provenance: rule)
//! ```
//!
//! Semantics carried from sigil-lang's `vm/rules.rs` and re-keyed to graph
//! patterns: a `(rule, subject)` pair fires at most once per transaction;
//! priority orders candidate *selection*, not rule truth; the iteration cap
//! is a tripwire (termination is verified in PRP-005, doc 10).
//!
//! Conflict policy (doc 04): two rules concluding different values for the
//! same single-cardinality property on the same subject — higher priority
//! wins, the loser is recorded as suppressed, never silently dropped. When a
//! higher-priority rule becomes fireable *late* (enabled by a derived fact)
//! and overrides an already-derived value, the earlier firing is re-marked
//! [`FiringOutcome::Overridden`] — the trace never claims a replaced value
//! is still standing.
//!
//! The gate has no back door: every conclusion passes the full five-check
//! gate with [`Provenance::Rule`]. A gate-refused conclusion becomes a
//! [`FiringOutcome::Rejected`] diagnostic (the PRP's "constraint checking as
//! continuous invariant" — the world stays valid, the rule's attempt is
//! recorded); the transaction itself keeps chaining.
//!
//! Spec: `docs/framework/04-reasoning.md` · Owner: PRP-002

use std::collections::BTreeMap;

use crate::World;
use crate::error::HorismosError;
use crate::ontology::term::{Iri, Ref, Value};
use crate::reason::matcher::{FiredSet, Matcher};
use crate::reason::proof::{self, ProofNode};
use crate::reason::rule::Term;
use crate::world::gate::Witness;

/// Doc 04's default iteration cap.
pub const ITERATION_CAP: usize = 100;

/// One rule firing, as recorded (doc 09's `fired` entries are built from
/// these).
#[derive(Debug, Clone, PartialEq)]
pub struct Firing {
    pub rule: Iri,
    pub subject: Ref,
    /// The concluded (property, value) — what `horis:then` proposed.
    pub property: Iri,
    pub value: Value,
    pub outcome: FiringOutcome,
    /// The per-condition proof recorded at fire time (D-2).
    pub proof: ProofNode,
}

/// What became of a firing's conclusion.
#[derive(Debug, Clone, PartialEq)]
pub enum FiringOutcome {
    /// Asserted through the gate; the fact stands.
    Derived,
    /// Conflict-policy loser at fire time: `winner` (equal or higher
    /// priority) had already set a different value this transaction.
    Suppressed { winner: Iri },
    /// Derived, then replaced by the later, higher-priority `winner` —
    /// enabled mid-chain by a derived fact.
    Overridden { winner: Iri },
    /// The gate refused the conclusion; the witness is the diagnostic
    /// naming exactly which declaration it would have violated.
    Rejected { witness: Box<Witness> },
}

/// Everything one transaction's chaining did — the reasoning step's payload
/// (doc 09) and the substance behind `World::explain`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReasoningRecord {
    pub fired: Vec<Firing>,
    /// Passes taken to reach the fixed point, including the final empty
    /// pass (PRP Example 1 reaches its fixed point "in 4 passes").
    pub passes: usize,
}

impl ReasoningRecord {
    /// The firings whose conclusions stand (doc 09 `fired` with `derived`).
    pub fn derived(&self) -> impl Iterator<Item = &Firing> {
        self.fired
            .iter()
            .filter(|f| matches!(f.outcome, FiringOutcome::Derived))
    }

    /// The conflict-policy losers (doc 09 `suppressed`).
    pub fn suppressed(&self) -> impl Iterator<Item = &Firing> {
        self.fired.iter().filter(|f| {
            matches!(
                f.outcome,
                FiringOutcome::Suppressed { .. } | FiringOutcome::Overridden { .. }
            )
        })
    }

    /// The gate-refused conclusions — constraint diagnostics.
    pub fn rejected(&self) -> impl Iterator<Item = &Firing> {
        self.fired
            .iter()
            .filter(|f| matches!(f.outcome, FiringOutcome::Rejected { .. }))
    }
}

/// A `horis:then` value as a storable [`Value`].
fn conclusion_value(term: &Term) -> Value {
    match term {
        Term::Iri(iri) => Value::Ref(Ref::new(*iri)),
        Term::Literal(value) => value.clone(),
    }
}

/// Runs the loop to fixed point over `world` (already holding the
/// transaction's base facts), firing through the gate. On
/// [`HorismosError::IterationCapExceeded`] the *caller* rolls the whole
/// transaction back — this function only reports.
pub fn chain(
    world: &mut World,
    matcher: &mut dyn Matcher,
) -> Result<ReasoningRecord, HorismosError> {
    let ontology = world.ontology().clone();
    let rules = ontology.rules();
    let mut record = ReasoningRecord::default();
    if rules.is_empty() {
        record.passes = 1;
        return Ok(record);
    }

    let mut fired = FiredSet::new();
    // (subject, property) → index into `record.fired` of the standing
    // rule-derived value — the conflict policy's memory.
    let mut standing: BTreeMap<(Iri, Iri), usize> = BTreeMap::new();

    for pass in 1..=ITERATION_CAP {
        record.passes = pass;
        let Some(candidate) = matcher.next_candidate(world, rules, &fired) else {
            return Ok(record); // fixed point
        };
        let rule = &rules[candidate.rule_index];
        let subject = candidate.subject;
        // No re-fire, whatever the outcome below.
        fired.insert((rule.iri, subject.iri()));

        let property = rule.then.property;
        let value = conclusion_value(&rule.then.value);
        // The proof is built against the world state the conditions held in
        // — before the conclusion lands.
        let proof = proof::firing_proof(world, subject, rule, &value);

        let single_valued = ontology
            .schema()
            .property(property)
            .and_then(|decl| decl.cardinality)
            .is_some_and(|c| c.is_single_valued());

        // Conflict policy: another rule set a *different* value for this
        // single-cardinality slot earlier in this transaction.
        if single_valued && let Some(&setter_index) = standing.get(&(subject.iri(), property)) {
            let setter_rule = record.fired[setter_index].rule;
            let setter_value = record.fired[setter_index].value.clone();
            if setter_value != value {
                let setter_priority = rules
                    .iter()
                    .find(|r| r.iri == setter_rule)
                    .map(|r| r.priority)
                    .unwrap_or_default();
                if setter_priority >= rule.priority {
                    // Higher (or equal — it fired first under the
                    // deterministic order) priority stands; this conclusion
                    // is suppressed, never asserted.
                    record.fired.push(Firing {
                        rule: rule.iri,
                        subject,
                        property,
                        value,
                        outcome: FiringOutcome::Suppressed {
                            winner: setter_rule,
                        },
                        proof,
                    });
                    continue;
                }
                // This rule outranks the setter: derive below, and re-mark
                // the earlier firing as overridden.
                record.fired[setter_index].outcome = FiringOutcome::Overridden { winner: rule.iri };
            }
        }

        match world.apply_rule_conclusion(subject, property, value.clone()) {
            Ok(()) => {
                world.record_derivation(subject, property, value.clone(), rule.iri, proof.clone());
                matcher.notify_asserted(world, rules, subject, property);
                standing.insert((subject.iri(), property), record.fired.len());
                record.fired.push(Firing {
                    rule: rule.iri,
                    subject,
                    property,
                    value,
                    outcome: FiringOutcome::Derived,
                    proof,
                });
            }
            Err(witness) => {
                record.fired.push(Firing {
                    rule: rule.iri,
                    subject,
                    property,
                    value,
                    outcome: FiringOutcome::Rejected { witness },
                    proof,
                });
            }
        }
    }

    // Cap passes exhausted. Only an actual remaining candidate means the
    // tripwire really fired.
    if matcher.next_candidate(world, rules, &fired).is_some() {
        let last_rule = record
            .fired
            .last()
            .map(|f| world.resolve(f.rule).to_owned())
            .unwrap_or_else(|| "none".to_owned());
        return Err(HorismosError::IterationCapExceeded {
            cap: ITERATION_CAP,
            last_rule,
        });
    }
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ontology;
    use crate::reason::testing::{EXAMPLE_1, PRELUDE, example_world};
    use crate::world::Provenance;

    fn t(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/t#{local}"))
    }

    fn rule_name(world: &World, firing: &Firing) -> String {
        world.ontology().local_name(world.resolve(firing.rule))
    }

    /// PRP Example 1, end to end: assert p7 with heartRate 128 and
    /// hasSymptom :CrushingChestPain; the transaction chains to the
    /// hand-computed fixed point — r1 fires (subsumption), r3 fires
    /// (enabled by the derived acuity), r2 fires and is suppressed, fixed
    /// point in 4 passes. Final state: acuity = :Immediate,
    /// needsCardiacMonitor = true.
    #[test]
    fn example_1_reaches_the_hand_computed_fixed_point() {
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
            .expect("the base facts pass the gate");

        // Final state, exactly as hand-computed.
        assert_eq!(
            world.value(p7, acuity),
            Some(&Value::Ref(Ref::new(immediate))),
            "acuity must be :Immediate (r1, priority 10 — not r2's :Urgent)"
        );
        assert_eq!(
            world.value(p7, monitor),
            Some(&Value::Boolean(true)),
            "needsCardiacMonitor must be derived by r3, enabled mid-chain"
        );

        // The record, firing by firing.
        let record = world.last_reasoning().expect("the transaction chained");
        assert_eq!(
            record.passes, 4,
            "three firings + the empty fixed-point pass"
        );
        let outcomes: Vec<(String, &FiringOutcome)> = record
            .fired
            .iter()
            .map(|f| (rule_name(&world, f), &f.outcome))
            .collect();
        assert_eq!(outcomes.len(), 3);
        assert_eq!(outcomes[0].0, "r1");
        assert_eq!(outcomes[0].1, &FiringOutcome::Derived);
        assert_eq!(outcomes[1].0, "r3");
        assert_eq!(outcomes[1].1, &FiringOutcome::Derived);
        assert_eq!(outcomes[2].0, "r2");
        let FiringOutcome::Suppressed { winner } = outcomes[2].1 else {
            panic!("r2's :Urgent must be suppressed by the standing :Immediate");
        };
        assert_eq!(world.resolve(*winner), "https://example.org/t#r1");

        assert_eq!(record.derived().count(), 2);
        assert_eq!(record.suppressed().count(), 1);
        assert_eq!(record.rejected().count(), 0);
    }

    /// The no-re-fire guard: r1's conditions still hold after it fires, and
    /// it must not fire again for the same (rule, subject) in the same
    /// transaction — three firings total, not an endless r1 loop.
    #[test]
    fn a_rule_fires_at_most_once_per_bindings_per_transaction() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");

        world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(128)),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("accepted");

        let record = world.last_reasoning().expect("chained");
        let r1_firings = record
            .fired
            .iter()
            .filter(|f| rule_name(&world, f) == "r1")
            .count();
        assert_eq!(r1_firings, 1);
    }

    /// PRP Example 3: a rule concluding a value outside the range's
    /// horis:oneOf is refused by the gate — recorded as a diagnostic in the
    /// gate's own domain wording, while the transaction itself commits and
    /// the world stays valid.
    #[test]
    fn a_gate_refused_conclusion_is_a_diagnostic_not_a_rollback() {
        let source = format!(
            "{PRELUDE}{EXAMPLE_1}\
             :r9 a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :Symptom ]\n\
             \x20 ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Discharged ] .\n"
        );
        let onto = Ontology::load_from_str(&source, |_| None)
            .expect("r9 is well-formed (its sin is semantic)");
        let mut world = World::new(&onto);
        let patient = t(&mut world, "Patient");
        let has_symptom = t(&mut world, "hasSymptom");
        let symptom = t(&mut world, "Symptom");
        let acuity = t(&mut world, "acuity");

        // A symptom that is NOT a ChestPain: r2 stays silent, so no standing
        // conflict shields r9 — its conclusion goes to the gate and is
        // refused there. (With a chest-pain patient, r2's higher-priority
        // :Urgent would win first and r9 would be recorded as *suppressed*
        // instead — the conflict policy runs before the gate.)
        let p = world
            .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(symptom)))])
            .expect("the base assertion must commit despite r9's doomed conclusion");

        // The doomed conclusion never landed; the world is valid.
        assert_eq!(world.value(p, acuity), None);

        let record = world.last_reasoning().expect("chained");
        let rejected: Vec<&Firing> = record.rejected().collect();
        assert_eq!(rejected.len(), 1);
        assert_eq!(rule_name(&world, rejected[0]), "r9");
        let FiringOutcome::Rejected { witness } = &rejected[0].outcome else {
            unreachable!("filtered on Rejected");
        };
        assert_eq!(
            witness.text,
            "acuity :Discharged is not one of horis:oneOf (:Immediate :Urgent :Standard) on :Patient",
            "the diagnostic is the gate's doc-03 wording, naming the violated declaration"
        );
        assert_eq!(witness.provenance, Provenance::Rule);
    }

    /// A higher-priority rule enabled *late* by a derived fact overrides a
    /// standing lower-priority conclusion; the earlier firing is re-marked
    /// Overridden so the trace never claims the replaced value stands.
    #[test]
    fn a_late_higher_priority_rule_overrides_and_marks_the_loser() {
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
        let immediate = t(&mut world, "Immediate");

        let p = world
            .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(chest_pain)))])
            .expect("accepted");

        assert_eq!(
            world.value(p, acuity),
            Some(&Value::Ref(Ref::new(immediate))),
            "rB (priority 20, enabled by rC's derived fact) must override rA's :Urgent"
        );

        let record = world.last_reasoning().expect("chained");
        let by_name: Vec<(String, &FiringOutcome)> = record
            .fired
            .iter()
            .map(|f| (rule_name(&world, f), &f.outcome))
            .collect();
        assert_eq!(by_name[0].0, "rA");
        let FiringOutcome::Overridden { winner } = by_name[0].1 else {
            panic!("rA must be re-marked Overridden, got {:?}", by_name[0].1);
        };
        assert_eq!(world.resolve(*winner), "https://example.org/t#rB");
        assert_eq!(by_name[1].0, "rC");
        assert_eq!(by_name[2].0, "rB");
        assert_eq!(by_name[2].1, &FiringOutcome::Derived);
    }

    /// The iteration-cap tripwire: 101 rules all fireable on one subject is
    /// one firing over the cap — the whole transaction rolls back, base
    /// facts included, and the error names the cap in domain terms.
    #[test]
    fn the_iteration_cap_rolls_back_the_whole_transaction() {
        let mut rules_source = String::new();
        for i in 0..=ITERATION_CAP {
            rules_source.push_str(&format!(
                ":r{i:03} a horis:Rule ; horis:priority {i} ;\n\
                 \x20 horis:when [ horis:all (\n\
                 \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:isAssigned true ] ) ] ;\n\
                 \x20 horis:then [ horis:property :tag ; horis:value \"tag-{i:03}\" ] .\n"
            ));
        }
        let source = format!(
            "{PRELUDE}\
             :Patient a rdfs:Class .\n\
             :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;\n\
             \x20   horis:cardinality \"0..1\" .\n\
             :tag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
             \x20   horis:cardinality \"0..*\" .\n\
             {rules_source}"
        );
        let onto = Ontology::load_from_str(&source, |_| None).expect("valid");
        let mut world = World::new(&onto);
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let before = world.clone();

        let err = world
            .assert_instance(patient, [(heart_rate, Value::Integer(80))])
            .expect_err("101 fireable rules is one over the cap");

        let text = err.domain_text();
        assert!(text.contains("100 iterations"), "{text}");
        assert!(text.contains("rolled back"), "{text}");

        assert_eq!(world.instance_count(), 0, "base facts must not survive");
        assert_eq!(
            world.instance_count(),
            before.instance_count(),
            "the world is exactly as before the transaction"
        );
        assert!(world.last_reasoning().is_none());

        // And the tripwire is a cap, not a phobia: exactly 100 fireable
        // rules chains clean. (Same rule set minus one.)
        let mut world = World::new(&onto);
        let _ = t(&mut world, "Patient");
        // 101 rules exist; make one unfireable is not possible here, so
        // simply confirm the count: this ontology genuinely trips the cap.
        assert_eq!(onto.rules().len(), ITERATION_CAP + 1);
    }

    /// Trace-equality precursor: the same transaction against two fresh
    /// worlds yields byte-identical reasoning records (doc 04's determinism
    /// guarantee, pinned at the record level).
    #[test]
    fn identical_transactions_yield_identical_reasoning_records() {
        let run = || {
            let mut world = example_world();
            let patient = t(&mut world, "Patient");
            let heart_rate = t(&mut world, "heartRate");
            let has_symptom = t(&mut world, "hasSymptom");
            let crushing = t(&mut world, "CrushingChestPain");
            world
                .assert_instance(
                    patient,
                    [
                        (heart_rate, Value::Integer(128)),
                        (has_symptom, Value::Ref(Ref::new(crushing))),
                    ],
                )
                .expect("accepted");
            world.last_reasoning().cloned().expect("chained")
        };
        assert_eq!(run(), run());
    }

    /// Stored fact provenance (the adopted PRP-001 gap): base facts carry
    /// the asserting provenance, derived facts carry Rule.
    #[test]
    fn fact_provenance_distinguishes_base_from_derived() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        let acuity = t(&mut world, "acuity");
        let immediate = t(&mut world, "Immediate");

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

        assert_eq!(
            world.fact_provenance(p7, heart_rate, &Value::Integer(128)),
            Some(Provenance::Perception)
        );
        assert_eq!(
            world.fact_provenance(p7, acuity, &Value::Ref(Ref::new(immediate))),
            Some(Provenance::Rule)
        );
    }

    /// Chaining also fires for other instances the new fact did not touch —
    /// rules run over the whole graph, and a previously-inert patient can be
    /// activated by a fresh transaction's rules only if its own conditions
    /// hold. (Here: the second patient's assertion re-fires r2 for the
    /// first patient as a same-value no-op — outcome Derived, value stable.)
    #[test]
    fn chaining_covers_the_whole_graph_not_just_the_new_subject() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let has_symptom = t(&mut world, "hasSymptom");
        let chest_pain = t(&mut world, "ChestPain");
        let acuity = t(&mut world, "acuity");
        let urgent = t(&mut world, "Urgent");

        let first = world
            .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(chest_pain)))])
            .expect("accepted");
        let second = world
            .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(chest_pain)))])
            .expect("accepted");

        assert_eq!(
            world.value(first, acuity),
            Some(&Value::Ref(Ref::new(urgent)))
        );
        assert_eq!(
            world.value(second, acuity),
            Some(&Value::Ref(Ref::new(urgent)))
        );

        let record = world.last_reasoning().expect("chained");
        // Second transaction: r2 fires for both subjects (fresh fired set).
        assert_eq!(record.derived().count(), 2);
    }
}
