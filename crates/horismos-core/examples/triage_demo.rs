//! A guided tour of what `horismos-core` does after PRP-001 — run with
//! `cargo run -p horismos-core --example triage_demo`.
//!
//! Loads the real triage ontology plus the SNOMED subset and walks through:
//! gate rejections in domain terms, provenance policy, sameAs-bridged
//! subsumption queries, snapshots/diff, and the tamper-proof binary form.

use horismos_core::{Ontology, Provenance, Ref, Value, World};
use rust_decimal::Decimal;

fn main() {
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/triage");

    println!("== 1 · Load triage.ttl + snomed-ct-subset.ttl ==");
    let onto = Ontology::load(format!("{base}/triage.ttl"))
        .expect("triage.ttl loads")
        .with_import(format!("{base}/snomed-ct-subset.ttl"))
        .expect("SNOMED subset imports");
    println!("loaded: root + 1 import, profile-valid, schema built\n");

    let mut world = World::new(&onto);
    let patient = world.intern("https://example.org/triage#Patient");
    let heart_rate = world.intern("https://example.org/triage#heartRate");
    let has_symptom = world.intern("https://example.org/triage#hasSymptom");
    let acuity = world.intern("https://example.org/triage#acuity");
    let immediate = world.intern("https://example.org/triage#Immediate");
    let symptom = world.intern("https://example.org/triage#Symptom");
    let crushing = world.intern("http://snomed.info/id/71884009");
    let diaphoresis = world.intern("http://snomed.info/id/52613005");

    println!("== 2 · The gate rejects, in domain terms ==");
    let err = world
        .assert_instance(patient, [(heart_rate, Value::Decimal(Decimal::from(1280)))])
        .unwrap_err();
    println!("heartRate 1280   → {}", err.domain_text());

    let err = world
        .assert_instance_as(
            Provenance::Perception,
            None,
            patient,
            [(acuity, Value::Ref(Ref::new(immediate)))],
        )
        .unwrap_err();
    println!("LLM sets acuity  → {}", err.domain_text());

    let err = world
        .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(diaphoresis)))])
        .unwrap_err();
    println!("diaphoresis as a :Symptom (not under the ChestPain anchor)");
    println!("                 → {}\n", err.domain_text());

    println!("== 3 · Valid facts pass; queries see the SNOMED hierarchy ==");
    let p1 = world.intern("https://example.org/triage#p1");
    world
        .assert_named_instance(
            p1,
            patient,
            [
                (heart_rate, Value::Decimal(Decimal::from(128))),
                (has_symptom, Value::Ref(Ref::new(crushing))),
            ],
        )
        .expect("valid patient admitted");
    println!("admitted :p1 — heartRate 128, symptom sct:71884009 (crushing chest pain)");

    let case = world
        .assert_instance(crushing, [])
        .expect("an instance of an imported concept");
    let found = world.instances(symptom).any(|r| r == case);
    println!(
        "instances(:Symptom) finds the sct:71884009 case: {found} \
         (sct:71884009 ⊑ sct:29857009 = :ChestPain ⊑ :Symptom)\n"
    );

    println!("== 4 · Snapshots, diff, and the tamper-proof binary form ==");
    let before = world.snapshot();
    let pleuritic = world.intern("http://snomed.info/id/2237002");
    world
        .assert_instance_as(
            Provenance::Api,
            Some(p1),
            patient,
            [(has_symptom, Value::Ref(Ref::new(pleuritic)))],
        )
        .expect("second symptom admitted");
    let diff = world.diff(&before);
    println!(
        "after adding pleuritic chest pain: diff = {} added, {} removed",
        diff.added().len(),
        diff.removed().len()
    );

    let bytes = world.snapshot().to_bytes();
    println!(
        "snapshot serialized: {} bytes (postcard, versioned header)",
        bytes.len()
    );

    let mut tampered = bytes.clone();
    let n = tampered.len();
    tampered[n / 2] ^= 0xFF;
    match horismos_core::Snapshot::from_bytes(&onto, &tampered) {
        Err(e) => println!("tampered bytes    → {}", e.domain_text()),
        Ok(_) => println!("tampered bytes    → this flip survived decoding AND re-gating"),
    }

    println!("\n(no LLM was consulted in the making of this world)");
}
