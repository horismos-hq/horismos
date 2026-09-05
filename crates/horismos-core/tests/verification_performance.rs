//! Executable guard for doc 10's interactive verification budget.

use std::time::{Duration, Instant};

use horismos_core::Ontology;

const TRIAGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/triage/triage.ttl"
);

#[test]
fn triage_load_plus_verification_is_under_one_second() {
    Ontology::load(TRIAGE).expect("warm canonical triage load must verify");
    let started = Instant::now();
    let ontology = Ontology::load(TRIAGE).expect("measured canonical triage load must verify");
    let elapsed = started.elapsed();

    assert!(ontology.verify().ok());
    assert!(
        elapsed < Duration::from_secs(1),
        "triage load plus verification took {elapsed:?}, exceeding the one-second budget"
    );
}
