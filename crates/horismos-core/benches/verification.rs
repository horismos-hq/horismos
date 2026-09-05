//! PRP-005's interactive-budget benchmark: canonical triage load includes
//! parsing, schema/rule/workflow construction, and default-hard verification.

use criterion::{Criterion, criterion_group, criterion_main};
use horismos_core::Ontology;

const TRIAGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/triage/triage.ttl"
);

fn load_and_verify_triage(c: &mut Criterion) {
    c.bench_function("triage_load_plus_verification", |b| {
        b.iter(|| Ontology::load(TRIAGE).expect("canonical triage must verify"));
    });
}

criterion_group!(benches, load_and_verify_triage);
criterion_main!(benches);
