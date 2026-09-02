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
    Ontology::load_allow_incomplete(TRIAGE)
        .expect("triage.ttl must load")
        .with_import(SNOMED)
        .expect("the SNOMED subset must import")
}

/// triage.ttl (plus its SNOMED import) with one added spoiler rule that
/// outranks r1 for `acuity` — appended in memory, kept local to this test:
/// the shipped fixture stays untouched.
///
/// Truth maintenance (doc 03 § Truth maintenance) means a world built
/// through the public API is always at its forward-chained fixed point, so
/// section (f) below — proving r1's `:Immediate` by rule search rather than
/// reading the standing value — needs a real world where r1's own
/// conditions hold but its conclusion does not actually stand. The old
/// "assert, then manually retract-as-Rule the derived acuity" technique no
/// longer produces that (retraction re-chains and simply re-derives it);
/// the spoiler does.
fn load_with_spoiler() -> Ontology {
    let triage_source = std::fs::read_to_string(TRIAGE).expect("triage.ttl reads");
    let spoiler = "\n:r0 a horis:Rule ;\n\
         \x20   horis:priority 20 ;\n\
         \x20   horis:when [ horis:all (\n\
         \x20       [ horis:subject :Patient ; horis:property :heartRate ; horis:isAssigned true ]\n\
         \x20   ) ] ;\n\
         \x20   horis:then [ horis:property :acuity ; horis:value :Standard ] ;\n\
         \x20   horis:explain \"Baseline — local to this test only, outranks r1\" .\n";
    Ontology::load_from_str_allow_incomplete(&format!("{triage_source}{spoiler}"), |_| None)
        .expect("triage.ttl plus the local spoiler rule must load")
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

    // (f) prove finds the derivation for a goal that is genuinely provable
    // but not the real fixed point's standing value (PRP Example 2's
    // provable case) — a spoiler rule local to this test (see
    // `load_with_spoiler`) outranks r1 for `acuity`, so r1's own conditions
    // hold for real without its conclusion ever landing for real; truth
    // maintenance is exactly what makes any other technique for reopening
    // an already-derived goal (e.g. retracting it) immediately re-derive it
    // instead.
    let spoiled = load_with_spoiler();
    let mut world2 = World::new(&spoiled);
    let patient2 = triage(&mut world2, "Patient");
    let heart_rate2 = triage(&mut world2, "heartRate");
    let has_symptom2 = triage(&mut world2, "hasSymptom");
    let acuity2 = triage(&mut world2, "acuity");
    let immediate2 = triage(&mut world2, "Immediate");
    let standard2 = triage(&mut world2, "Standard");
    let unstable_angina2 = world2.intern("http://snomed.info/id/4557003");
    let p10 = world2
        .assert_instance(
            patient2,
            [
                (heart_rate2, Value::Integer(128)),
                (has_symptom2, Value::Ref(Ref::new(unstable_angina2))),
            ],
        )
        .expect("the base facts pass the gate");
    assert_eq!(
        world2.value(p10, acuity2),
        Some(&Value::Ref(Ref::new(standard2))),
        "sanity: the local spoiler must win the real acuity slot, not r1"
    );

    let proof = world2
        .prove(p10, acuity2, &Value::Ref(Ref::new(immediate2)))
        .expect("r1 establishes the goal hypothetically");
    let rendered = proof.render(&world2);
    assert!(rendered.contains("via r1"), "{rendered}");
    assert_eq!(
        world2.value(p10, acuity2),
        Some(&Value::Ref(Ref::new(standard2))),
        "prove is read-only — the real world keeps the spoiler's standing value"
    );

    // (g) Zero LLM: see the module doc — `make no-model-gate` builds and
    // tests this whole surface with no model, no network, no keys.
}

/// Anchors the exact proof text the Python quickstart pins
/// (`python/tests/test_reasoning.py`, PRP-003 B4): same scenario with the
/// local `:CrushingChestPain` leaf instead of the imported SNOMED one, so
/// cross-language byte-parity is asserted against a Rust-side expectation,
/// not just a shared code path.
#[test]
fn the_python_quickstart_proof_renders_identically() {
    let onto = load();
    let mut world = World::new(&onto);
    let patient = triage(&mut world, "Patient");
    let heart_rate = triage(&mut world, "heartRate");
    let has_symptom = triage(&mut world, "hasSymptom");
    let crushing = triage(&mut world, "CrushingChestPain");
    let acuity = triage(&mut world, "acuity");

    let p = world
        .assert_instance(
            patient,
            [
                (heart_rate, Value::Integer(128)),
                (has_symptom, Value::Ref(Ref::new(crushing))),
            ],
        )
        .expect("the base facts pass the gate");

    let proof = world.explain(p, acuity).expect("r1 derived the acuity");
    assert_eq!(
        proof.render(&world),
        ":acuity = :Immediate   via r1 \u{2014} \"Tachycardia with chest pain \u{2014} immediate track\"\n\
         \x20 \u{251c}\u{2500} heartRate 128 exceeds 120   (asserted by the api)\n\
         \x20 \u{2514}\u{2500} hasSymptom :CrushingChestPain \u{2291} :ChestPain   (subsumption)\n"
    );
}
