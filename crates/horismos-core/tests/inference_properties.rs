//! U10 — property tests pinning the inference invariants (PRP-002
//! acceptance: fixed-point convergence, no re-fire, deterministic
//! byte-identical traces), proptest-style over randomized assertion
//! scripts, through the public API only.
//!
//! The fixture is the PRP's worked Example 1 — triage.ttl's domain plus the
//! illustrative `:r2`/`:r3`/`:needsCardiacMonitor` — declared inline
//! (integration tests cannot reach the crate-private test fixture, and the
//! PRP marks those rules "illustrative only", so they stay out of the
//! shipped `.ttl`).
//!
//! Spec: `docs/framework/04-reasoning.md` § Determinism guarantees ·
//! Owner: PRP-002

use std::collections::BTreeSet;

use horismos_core::reason::forward::ITERATION_CAP;
use horismos_core::trace::reasoning_step;
use horismos_core::{Iri, Ontology, Ref, Value, World};
use proptest::prelude::*;

const EXAMPLE_1: &str = "@prefix : <https://example.org/t#> .\n\
     @prefix horis: <https://w3id.org/horismos#> .\n\
     @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
     @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
     @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
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

fn example_ontology() -> Ontology {
    Ontology::load_from_str_allow_incomplete(EXAMPLE_1, |_| None)
        .expect("the Example 1 ontology loads")
}

fn t(world: &mut World, local: &str) -> Iri {
    world.intern(&format!("https://example.org/t#{local}"))
}

/// One transaction's observable outcome: the trace JSON, the pass count,
/// and the (rule, subject) firing list.
type TransactionOutcome = (String, usize, Vec<(String, String)>);

/// Replays one script; returns each transaction's outcome.
fn replay(onto: &Ontology, script: &[(i64, usize)]) -> Vec<TransactionOutcome> {
    let mut world = World::new(onto);
    let patient = t(&mut world, "Patient");
    let heart_rate = t(&mut world, "heartRate");
    let has_symptom = t(&mut world, "hasSymptom");
    let symptoms = [
        t(&mut world, "CrushingChestPain"),
        t(&mut world, "ChestPain"),
        t(&mut world, "Symptom"),
    ];

    let mut out = Vec::new();
    for &(rate, pick) in script {
        let mut properties = vec![(heart_rate, Value::Integer(rate))];
        if pick < symptoms.len() {
            properties.push((has_symptom, Value::Ref(Ref::new(symptoms[pick]))));
        }
        world
            .assert_instance(patient, properties)
            .expect("script values are gate-valid by construction");
        let record = world.last_reasoning().cloned().expect("chained");
        let fired_pairs = record
            .fired
            .iter()
            .map(|f| {
                (
                    world.resolve(f.rule).to_owned(),
                    world.resolve(f.subject.iri()).to_owned(),
                )
            })
            .collect();
        let step = reasoning_step(&world, None, &record);
        out.push((
            serde_json::to_string(&step).expect("serializes"),
            record.passes,
            fired_pairs,
        ));
    }
    out
}

proptest! {
    /// Fixed-point convergence: every transaction reaches its fixed point
    /// well under the iteration cap, whatever the script.
    #[test]
    fn chaining_converges_under_the_cap(
        script in proptest::collection::vec((60i64..200, 0usize..4), 1..6)
    ) {
        let onto = example_ontology();
        for (_, passes, _) in replay(&onto, &script) {
            prop_assert!(passes <= ITERATION_CAP);
        }
    }

    /// No re-fire: within one transaction, a (rule, subject) pair fires at
    /// most once — sigil-lang's fired-set semantics, re-keyed to the graph.
    #[test]
    fn no_rule_fires_twice_for_the_same_bindings(
        script in proptest::collection::vec((60i64..200, 0usize..4), 1..6)
    ) {
        let onto = example_ontology();
        for (_, _, fired) in replay(&onto, &script) {
            let unique: BTreeSet<_> = fired.iter().collect();
            prop_assert_eq!(unique.len(), fired.len(), "duplicate firing: {:?}", fired);
        }
    }

    /// Determinism, end to end: the same script against two fresh worlds
    /// yields byte-identical trace JSON for every transaction (doc 04's
    /// same-inputs-same-trace guarantee — trace equality as a first-class
    /// assertion).
    #[test]
    fn identical_scripts_yield_byte_identical_traces(
        script in proptest::collection::vec((60i64..200, 0usize..4), 1..6)
    ) {
        let onto = example_ontology();
        prop_assert_eq!(replay(&onto, &script), replay(&onto, &script));
        // And across two independently loaded ontologies (fresh interners):
        let again = example_ontology();
        prop_assert_eq!(replay(&onto, &script), replay(&again, &script));
    }
}
