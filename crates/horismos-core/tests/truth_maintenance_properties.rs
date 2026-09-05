//! Truth maintenance — the determinism property (captain's ruling
//! `tm-derived-facts.md` (d)): after any sequence of mutations, a world is
//! byte-identical to a fresh world built from that world's *final* base
//! facts alone. This is the cleanest correctness statement for the fix:
//! the resulting world never carries a stale derivation forward, whatever
//! path of replace/retract got there.
//!
//! The script only ever touches `heartRate`, a single-valued property, on
//! one named subject; `hasSymptom` is asserted once up front and never
//! touched again. That sidesteps a second, orthogonal question — assertion
//! order within a multi-valued property across an incremental history vs.
//! a single-shot reconstruction — which truth maintenance does not claim
//! to answer and this property is not testing.
//!
//! Spec: `docs/framework/03-world.md` § Truth maintenance · Owner: PRP-002

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
    Ontology::load_from_str_allow_incomplete(EXAMPLE_1, |_| None).expect("Example 1 loads")
}

fn t(world: &mut World, local: &str) -> Iri {
    world.intern(&format!("https://example.org/t#{local}"))
}

/// One step: set `heartRate` to a value, or clear it.
#[derive(Debug, Clone, Copy)]
enum Step {
    Set(i64),
    Clear,
}

fn step_strategy() -> impl Strategy<Value = Step> {
    prop_oneof![(60i64..200).prop_map(Step::Set), Just(Step::Clear),]
}

/// Builds `p1` with `hasSymptom :ChestPain` asserted once, then applies
/// `script` one step at a time, tracking the surviving `heartRate` base
/// fact. Returns the final world alongside that ground truth.
fn replay(onto: &Ontology, script: &[Step]) -> (World, Option<i64>) {
    let mut world = World::new(onto);
    let patient = t(&mut world, "Patient");
    let heart_rate = t(&mut world, "heartRate");
    let has_symptom = t(&mut world, "hasSymptom");
    let chest_pain = t(&mut world, "ChestPain");
    let p1 = t(&mut world, "p1");

    world
        .assert_named_instance(
            p1,
            patient,
            [(has_symptom, Value::Ref(Ref::new(chest_pain)))],
        )
        .expect("hasSymptom is gate-valid and never touched again");

    let mut current: Option<i64> = None;
    for step in script {
        match *step {
            Step::Set(v) => {
                world
                    .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(v))])
                    .expect("script values are gate-valid by construction");
                current = Some(v);
            }
            Step::Clear => {
                if let Some(v) = current {
                    world
                        .retract(Ref::new(p1), heart_rate, &Value::Integer(v))
                        .expect("heartRate is a plain, non-immutable base fact");
                    current = None;
                }
            }
        }
    }
    (world, current)
}

/// Builds a fresh world asserting `p1`'s final surviving base facts
/// (`hasSymptom` plus `heartRate` if any) in one shot.
fn fresh_reconstruction(onto: &Ontology, final_heart_rate: Option<i64>) -> World {
    let mut world = World::new(onto);
    let patient = t(&mut world, "Patient");
    let heart_rate = t(&mut world, "heartRate");
    let has_symptom = t(&mut world, "hasSymptom");
    let chest_pain = t(&mut world, "ChestPain");
    let p1 = t(&mut world, "p1");

    let mut properties = vec![(has_symptom, Value::Ref(Ref::new(chest_pain)))];
    if let Some(v) = final_heart_rate {
        properties.push((heart_rate, Value::Integer(v)));
    }
    world
        .assert_named_instance(p1, patient, properties)
        .expect("the reconstructed base facts are gate-valid by construction");
    world
}

proptest! {
    /// Equivalence with fresh-world reconstruction (the determinism
    /// property): a world that lived through an arbitrary sequence of
    /// replace/retract mutations on a supporting base fact is
    /// byte-identical, as a snapshot, to a fresh world built from exactly
    /// that world's final base facts.
    #[test]
    fn incremental_mutation_matches_fresh_reconstruction(
        script in proptest::collection::vec(step_strategy(), 0..12)
    ) {
        let onto = example_ontology();
        let (incremental, final_heart_rate) = replay(&onto, &script);
        let fresh = fresh_reconstruction(&onto, final_heart_rate);

        prop_assert_eq!(
            incremental.snapshot().to_bytes(),
            fresh.snapshot().to_bytes(),
            "script {:?} left a world that disagrees with fresh reconstruction",
            script
        );
    }
}
