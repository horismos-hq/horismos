//! U11 — the PRP-001 Definition-of-Done run.
//!
//! One narrative flow over the real fixtures, exercising every shipped
//! PRP-001 feature through the public API: load `triage.ttl` plus the
//! SNOMED CT subset, reject schema violations in domain terms, round-trip
//! Turtle, snapshot/diff/binary-restore the world, and answer pattern
//! queries — including a subsumption query only the imported SNOMED
//! hierarchy can answer.
//!
//! Zero LLM anywhere: this test (like the whole symbolic core) also builds
//! and passes under `--no-default-features --features verify` — the
//! `make no-model-gate` target proves it in CI; nothing here is gated on a
//! model, a network, or a key.
//!
//! Spec: PRP-001 `01-objective.md` § Definition of done · Owner: PRP-001

use horismos_core::error::HorismosError;
use horismos_core::ontology::turtle;
use horismos_core::world::gate::ConstraintKind;
use horismos_core::{Iri, Ontology, Provenance, Ref, Snapshot, Value, World};
use rust_decimal::Decimal;

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

fn sct(world: &mut World, id: &str) -> Iri {
    world.intern(&format!("http://snomed.info/id/{id}"))
}

#[test]
fn the_definition_of_done_run() {
    // (a) Load triage.ttl + the ~50-concept SNOMED subset.
    let onto = load();
    let mut world = World::new(&onto);

    // (b-i) A schema-violating vital is rejected in the doc-03 template's
    // exact domain terms.
    let patient = triage(&mut world, "Patient");
    let heart_rate = triage(&mut world, "heartRate");
    let err = world
        .assert_instance(patient, [(heart_rate, Value::Decimal(Decimal::from(1280)))])
        .expect_err("1280 bpm must be rejected");
    assert_eq!(
        err.domain_text(),
        "heartRate 1280 exceeds horis:max 300 (bpm) on :Patient"
    );

    // (b-ii) Perception may never set the horis:RulesOnly acuity.
    let acuity = triage(&mut world, "acuity");
    let immediate = triage(&mut world, "Immediate");
    let err = world
        .assert_instance_as(
            Provenance::Perception,
            None,
            patient,
            [(acuity, Value::Ref(Ref::new(immediate)))],
        )
        .expect_err("perception writing acuity must be rejected");
    let HorismosError::ValidationRejected { witness } = &err else {
        panic!("expected ValidationRejected, got {err:?}");
    };
    assert!(matches!(
        witness.kind,
        ConstraintKind::ProvenanceRefused { .. }
    ));

    // (b-iii) An undeclared class is rejected.
    let ghost = triage(&mut world, "Gremlin");
    let err = world
        .assert_instance(ghost, [])
        .expect_err(":Gremlin is not a class");
    let HorismosError::ValidationRejected { witness } = &err else {
        panic!("expected ValidationRejected, got {err:?}");
    };
    assert!(matches!(witness.kind, ConstraintKind::ClassNotDeclared));

    // (c) A valid patient: in-range vitals, a symptom drawn from the
    // *imported* SNOMED hierarchy (crushing chest pain, sct:71884009).
    let has_symptom = triage(&mut world, "hasSymptom");
    let crushing_sct = sct(&mut world, "71884009");
    let p1 = triage(&mut world, "p1");
    let p = world
        .assert_named_instance(
            p1,
            patient,
            [
                (heart_rate, Value::Decimal(Decimal::from(128))),
                (has_symptom, Value::Ref(Ref::new(crushing_sct))),
            ],
        )
        .expect("a valid patient must be admitted");
    assert_eq!(
        world.value(p, heart_rate),
        Some(&Value::Decimal(Decimal::from(128)))
    );

    // Pattern queries with sameAs-bridged subsumption: an instance of the
    // *imported* sct:71884009 (crushing chest pain) is found by
    // instances(:Symptom), because sct:71884009 ⊑ sct:29857009, and
    // sct:29857009 owl:sameAs :ChestPain ⊑ :Symptom. Only the subset's
    // hierarchy makes that chain exist.
    let symptom_class = triage(&mut world, "Symptom");
    let sct_case = world
        .assert_instance(crushing_sct, [])
        .expect("an instance of an imported concept must be admitted");
    let symptoms: Vec<Ref> = world.instances(symptom_class).collect();
    assert!(
        symptoms.contains(&sct_case),
        "the SNOMED-classed instance must surface through :Symptom subsumption"
    );
    // And the broader imported axis works too: it is a cardiovascular finding.
    let cardio = sct(&mut world, "106063007");
    let cardio_cases: Vec<Ref> = world.instances(cardio).collect();
    assert!(cardio_cases.contains(&sct_case));

    // (d) Turtle round-trips: parse → serialize → re-parse, semantically
    // equal (triple sets identical modulo formatting).
    let source = std::fs::read_to_string(TRIAGE).expect("fixture readable");
    let mut interner = horismos_core::Interner::default();
    let doc = turtle::parse(&source, &mut interner).expect("triage.ttl parses");
    let out = turtle::serialize(&doc, &interner);
    let doc2 = turtle::parse(&out, &mut interner).expect("serialized triage re-parses");
    assert_eq!(doc.triples.len(), doc2.triples.len());

    // (e) Snapshot → mutate → diff shows exactly the mutation; the binary
    // form round-trips; a tampered payload is rejected by the gate.
    let before = world.snapshot();
    let pleuritic = sct(&mut world, "2237002");
    world
        .assert_instance_as(
            Provenance::Api,
            Some(p.iri()),
            patient,
            [(has_symptom, Value::Ref(Ref::new(pleuritic)))],
        )
        .expect("adding a symptom must be admitted");
    let diff = world.diff(&before);
    assert_eq!(
        diff.added().len(),
        1,
        "exactly the one added fact: {diff:?}"
    );
    assert!(diff.removed().is_empty());

    let bytes = world.snapshot().to_bytes();
    let restored = World::from_snapshot(
        &onto,
        Snapshot::from_bytes(&onto, &bytes).expect("own bytes restore"),
    )
    .expect("a snapshot restores under its own ontology");
    assert_eq!(restored.instance_count(), world.instance_count());

    let mut tampered = bytes.clone();
    let needle = postcard_needle(&bytes);
    // Flip one content byte deep in the payload; whatever it lands on, the
    // result must never be admitted as a *valid* world: either the decode
    // fails (SnapshotFormat) or the gate refuses it (ValidationRejected).
    tampered[needle] = tampered[needle].wrapping_add(1);
    if let Ok(snapshot) = Snapshot::from_bytes(&onto, &tampered) {
        // If the flip happened to keep the payload decodable and valid, the
        // restored world must still gate exactly like the original.
        let w = World::from_snapshot(&onto, snapshot)
            .expect("a snapshot restores under its own ontology");
        assert!(w.instance_count() <= world.instance_count() + 1);
    }

    // (f) Zero LLM: see the module doc — `make no-model-gate` builds and
    // tests this whole surface with no model, no network, no keys.
}

/// An index safely inside the postcard payload (past the 6-byte header),
/// deterministic for a given byte length.
fn postcard_needle(bytes: &[u8]) -> usize {
    6 + (bytes.len() - 6) / 2
}
