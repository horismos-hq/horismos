//! Naive vs Rete-lite matching benchmarks (U8) — the evidence PRP-002 Key
//! decision 1 prescribes: "criterion benchmarks (naive vs. Rete-lite) on
//! the triage fixture and a larger synthetic ontology". Results are
//! recorded in the run ledger and the PRP; doc 14 CI-tracks these from
//! PRP-002 onward.
//!
//! Two scales:
//! - `triage_transaction`: the flagship fixture (1 rule, one patient) —
//!   the perf-posture row "gate + chain transaction < 1 ms".
//! - `chain60_transaction`: a synthetic 60-rule sequential chain (rule i
//!   turns on the flag rule i+1 watches) over a world padded with 30 idle
//!   patients — the shape where incremental matching should pay: each
//!   firing changes one (subject, property), so Rete-lite re-checks one
//!   rule while the naive loop re-scans all 60 against every instance,
//!   every pass.
//!
//! `*_replace_transaction` (PRP-002 truth-maintenance decision record,
//! `03-decisions.md`): the cost the "retract-derived + rechain" fix adds —
//! a *second* transaction (replacing `heartRate`) against a world already
//! at its fixed point, which now pays for resetting every standing
//! derivation plus a full matcher rebuild and re-chain, on top of the same
//! per-transaction cost `triage_transaction`/`chain60_transaction` already
//! measure. The first-ever transaction is unaffected (the reset scans an
//! empty `derivations` map — nothing to do).

use criterion::{Criterion, criterion_group, criterion_main};
use horismos_core::reason::matcher::MatcherKind;
use horismos_core::{Ontology, Ref, Value, World};

fn triage_ontology() -> Ontology {
    Ontology::load_allow_incomplete(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/triage/triage.ttl"
    ))
    .expect("triage.ttl loads")
}

/// The synthetic chain: `:f0 … :f60` boolean flags on `:Patient`;
/// rule i (priority descending so the chain fires in order) watches `f{i}`
/// and sets `f{i+1}`.
fn chain_ontology(rules: usize) -> Ontology {
    let mut source = String::from(
        "@prefix : <https://example.org/bench#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         :Patient a rdfs:Class .\n",
    );
    for i in 0..=rules {
        source.push_str(&format!(
            ":f{i} a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;\n\
             \x20   horis:cardinality \"0..1\" .\n"
        ));
    }
    for i in 0..rules {
        let next = i + 1;
        let priority = rules - i;
        source.push_str(&format!(
            ":r{i:03} a horis:Rule ; horis:priority {priority} ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :f{i} ; horis:is true ] ) ] ;\n\
             \x20 horis:then [ horis:property :f{next} ; horis:value true ] .\n"
        ));
    }
    Ontology::load_from_str_allow_incomplete(&source, |_| None).expect("the synthetic chain loads")
}

fn triage_transaction(c: &mut Criterion) {
    let onto = triage_ontology();
    let mut group = c.benchmark_group("triage_transaction");
    for (name, kind) in [
        ("naive", MatcherKind::Naive),
        ("rete", MatcherKind::ReteLite),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut world = World::new(&onto);
                world.use_matcher(kind);
                let patient = world.intern("https://example.org/triage#Patient");
                let heart_rate = world.intern("https://example.org/triage#heartRate");
                let has_symptom = world.intern("https://example.org/triage#hasSymptom");
                let crushing = world.intern("https://example.org/triage#CrushingChestPain");
                world
                    .assert_instance(
                        patient,
                        [
                            (heart_rate, Value::Integer(128)),
                            (has_symptom, Value::Ref(Ref::new(crushing))),
                        ],
                    )
                    .expect("accepted")
            })
        });
    }
    group.finish();
}

fn chain60_transaction(c: &mut Criterion) {
    let onto = chain_ontology(60);
    let mut group = c.benchmark_group("chain60_transaction");
    group.sample_size(30);
    for (name, kind) in [
        ("naive", MatcherKind::Naive),
        ("rete", MatcherKind::ReteLite),
    ] {
        group.bench_function(name, |b| {
            b.iter_batched(
                || {
                    // 30 idle patients pad the subject axis; the 31st gets
                    // the fuse lit inside the measured section.
                    let mut world = World::new(&onto);
                    world.use_matcher(kind);
                    let patient = world.intern("https://example.org/bench#Patient");
                    for _ in 0..30 {
                        world.assert_instance(patient, []).expect("idle patient");
                    }
                    let f0 = world.intern("https://example.org/bench#f0");
                    (world, patient, f0)
                },
                |(mut world, patient, f0)| {
                    world
                        .assert_instance(patient, [(f0, Value::Boolean(true))])
                        .expect("lights the 60-rule chain")
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn triage_replace_transaction(c: &mut Criterion) {
    let onto = triage_ontology();
    let mut group = c.benchmark_group("triage_replace_transaction");
    for (name, kind) in [
        ("naive", MatcherKind::Naive),
        ("rete", MatcherKind::ReteLite),
    ] {
        group.bench_function(name, |b| {
            b.iter_batched(
                || {
                    let mut world = World::new(&onto);
                    world.use_matcher(kind);
                    let patient = world.intern("https://example.org/triage#Patient");
                    let heart_rate = world.intern("https://example.org/triage#heartRate");
                    let has_symptom = world.intern("https://example.org/triage#hasSymptom");
                    let crushing = world.intern("https://example.org/triage#CrushingChestPain");
                    let p1 = world.intern("https://example.org/triage#p1");
                    world
                        .assert_named_instance(
                            p1,
                            patient,
                            [
                                (heart_rate, Value::Integer(128)),
                                (has_symptom, Value::Ref(Ref::new(crushing))),
                            ],
                        )
                        .expect("accepted");
                    (world, p1, patient, heart_rate)
                },
                |(mut world, p1, patient, heart_rate)| {
                    world
                        .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(96))])
                        .expect("96 is gate-valid; this now resets and re-chains derived facts")
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn chain60_replace_transaction(c: &mut Criterion) {
    let onto = chain_ontology(60);
    let mut group = c.benchmark_group("chain60_replace_transaction");
    group.sample_size(30);
    for (name, kind) in [
        ("naive", MatcherKind::Naive),
        ("rete", MatcherKind::ReteLite),
    ] {
        group.bench_function(name, |b| {
            b.iter_batched(
                || {
                    let mut world = World::new(&onto);
                    world.use_matcher(kind);
                    let patient = world.intern("https://example.org/bench#Patient");
                    for _ in 0..30 {
                        world.assert_instance(patient, []).expect("idle patient");
                    }
                    let f0 = world.intern("https://example.org/bench#f0");
                    let lit = world.intern("https://example.org/bench#lit");
                    world
                        .assert_named_instance(lit, patient, [(f0, Value::Boolean(true))])
                        .expect("lights the 60-rule chain");
                    (world, lit, patient, f0)
                },
                |(mut world, lit, patient, f0)| {
                    // Re-asserting the same value is a no-op on the base
                    // fact, but truth maintenance still resets and
                    // re-chains all 60 derived flags on this transaction.
                    world
                        .assert_named_instance(lit, patient, [(f0, Value::Boolean(true))])
                        .expect("accepted")
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    triage_transaction,
    chain60_transaction,
    triage_replace_transaction,
    chain60_replace_transaction
);
criterion_main!(benches);
