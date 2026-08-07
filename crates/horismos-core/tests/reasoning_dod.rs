//! U10 — the PRP-002 Definition-of-Done run.
//!
//! The triage scenario, end to end, symbolically (PRP-002 § Definition of
//! done): structured facts in → subsumption + forward chaining → `acuity`
//! derived → `explain` renders the r1 proof; traces serialize to the
//! versioned schema; trace equality asserted across identical runs. The
//! subsumption leg runs through the **imported** SNOMED hierarchy
//! (sct:4557003 "Unstable angina" ⊑ sct:194828000 "Angina" ⊑ sct:29857009,
//! `owl:sameAs`-anchored to `:ChestPain`) — hierarchy nobody wrote a rule
//! for.
//!
//! Zero LLM anywhere: this test also builds and passes under
//! `--no-default-features --features verify` (`make no-model-gate`).
//!
//! Spec: `PRPs/PRP-002-reasoning-substrate.md` § Definition of done ·
//! Owner: PRP-002

use horismos_core::trace::{Trace, reasoning_step};
use horismos_core::{Iri, Ontology, Ref, Value, World};

const TRIAGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/triage/triage.ttl"
);
const SNOMED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/triage/snomed-ct-subset.ttl"
);

fn load() -> Ontology {
    Ontology::load(TRIAGE)
        .expect("triage.ttl must load")
        .with_import(SNOMED)
        .expect("the SNOMED subset must import")
}

fn triage(world: &mut World, local: &str) -> Iri {
    world.intern(&format!("https://example.org/triage#{local}"))
}

/// One end-to-end run: returns the world, the patient, and the trace.
fn run() -> (World, Ref, Trace) {
    let onto = load();
    let mut world = World::new(&onto);
    let patient = triage(&mut world, "Patient");
    let heart_rate = triage(&mut world, "heartRate");
    let has_symptom = triage(&mut world, "hasSymptom");
    let unstable_angina = world.intern("http://snomed.info/id/4557003");

    // Structured facts in (perception's output shape, no LLM in sight).
    let p7 = world
        .assert_instance(
            patient,
            [
                (heart_rate, Value::Integer(128)),
                (has_symptom, Value::Ref(Ref::new(unstable_angina))),
            ],
        )
        .expect("the base facts pass the gate");

    let record = world
        .last_reasoning()
        .cloned()
        .expect("the transaction chained");
    let mut trace = Trace::new("run_dod");
    trace
        .steps
        .push(reasoning_step(&world, Some(":Assess"), &record));
    (world, p7, trace)
}

#[test]
fn the_definition_of_done_run() {
    let (mut world, p7, trace) = run();
    let acuity = triage(&mut world, "acuity");
    let immediate = triage(&mut world, "Immediate");

    // (a) Forward chaining + imported subsumption derived the acuity: r1
    // fired for "unstable angina" although no rule mentions it — two
    // imported hops and a sameAs anchor away from :ChestPain.
    assert_eq!(
        world.value(p7, acuity),
        Some(&Value::Ref(Ref::new(immediate))),
        "acuity must be derived :Immediate via r1"
    );

    // (b) explain renders the recorded r1 proof, doc 04's exact shape —
    // the subsumption hop visible, the leaf provenance visible.
    let proof = world
        .explain(p7, acuity)
        .expect("the derivation is recorded");
    assert_eq!(
        proof.render(&world),
        ":acuity = :Immediate   via r1 — \"Tachycardia with chest pain — immediate track\"\n\
         \x20 ├─ heartRate 128 exceeds 120   (asserted by the api)\n\
         \x20 └─ hasSymptom sct:4557003 ⊑ :ChestPain   (subsumption)\n"
    );

    // (c) The trace serializes to the versioned schema, both forms, and
    // parses back.
    let compact = trace.to_json();
    assert!(compact.starts_with("{\"schema\":\"horismos.trace/1\""));
    assert_eq!(Trace::from_json(&compact).expect("parses"), trace);
    assert_eq!(
        Trace::from_json(&trace.to_json_pretty()).expect("parses"),
        trace
    );
    // D-2's size evidence: per-condition granularity keeps the whole
    // Example-1-shaped step in the low kilobytes at profile scale.
    assert!(
        compact.len() < 4096,
        "the compact trace should stay small; got {} bytes",
        compact.len()
    );

    // (d) Trace equality is a valid test assertion: a second identical run
    // serializes byte-identically (deterministic inference, doc 04).
    let (_, _, again) = run();
    assert_eq!(compact, again.to_json());

    // (e) prove answers the failing case with None (PRP Example 2): 96 bpm
    // cannot reach :Immediate, and nothing else concludes it.
    let patient = triage(&mut world, "Patient");
    let heart_rate = triage(&mut world, "heartRate");
    let has_symptom = triage(&mut world, "hasSymptom");
    let chest_pain = triage(&mut world, "ChestPain");
    let p9 = world
        .assert_instance(
            patient,
            [
                (heart_rate, Value::Integer(96)),
                (has_symptom, Value::Ref(Ref::new(chest_pain))),
            ],
        )
        .expect("accepted");
    assert!(
        world
            .prove(p9, acuity, &Value::Ref(Ref::new(immediate)))
            .is_none(),
        "prove must refuse: r1's heartRate condition is false and nothing else concludes :Immediate"
    );

    // (f) prove finds the derivation for an open goal (PRP Example 2's
    // provable case): unwind the derived facts through the rule path (the
    // one retraction D-5 admits), then ask again.
    use horismos_core::Provenance;
    world
        .retract_as(
            Provenance::Rule,
            p7,
            acuity,
            &Value::Ref(Ref::new(immediate)),
        )
        .expect("the rule path may unwind its own derivation");
    let proof = world
        .prove(p7, acuity, &Value::Ref(Ref::new(immediate)))
        .expect("r1 establishes the goal hypothetically");
    let rendered = proof.render(&world);
    assert!(rendered.contains("via r1"), "{rendered}");
    assert!(
        world.value(p7, acuity).is_none(),
        "prove is read-only — the goal stays open in the real world"
    );

    // (g) Zero LLM: see the module doc — `make no-model-gate` builds and
    // tests this whole surface with no model, no network, no keys.
}
