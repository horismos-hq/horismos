//! U10 — the property-test suite against the validation gate (PRP-001).
//!
//! Four invariants over generated operation sequences, run through the
//! public API against the real `examples/triage/triage.ttl`:
//!
//! 1. no sequence of valid operations produces an invalid world;
//! 2. every rejection names its constraint, in domain terms;
//! 3. a failing assertion is atomic — the world is exactly what it would
//!    have been without it;
//! 4. inference-free determinism — the same operations produce
//!    byte-identical worlds.
//!
//! Spec: `docs/framework/03-world.md` · Owner: PRP-001

use horismos_core::error::HorismosError;
use horismos_core::world::gate::ConstraintKind;
use horismos_core::{Ontology, Provenance, Ref, Value, World};
use proptest::prelude::*;
use rust_decimal::Decimal;

const TRIAGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/triage/triage.ttl"
);
const NS: &str = "https://example.org/triage#";

fn ontology() -> Ontology {
    Ontology::load(TRIAGE).expect("triage.ttl must load")
}

fn iri(world: &mut World, local: &str) -> horismos_core::Iri {
    world.intern(&format!("{NS}{local}"))
}

/// One generated operation. `Valid` variants can never be refused by the
/// gate; `Invalid` variants can never be accepted. That partition is what
/// the properties lean on.
#[derive(Debug, Clone)]
enum Op {
    // -- valid ------------------------------------------------------------
    /// Anonymous patient: in-range heart rate, both symptom levels.
    AssertPatient { heart_rate: i32 },
    /// Retract one symptom fact from the `index`-th asserted patient (no-op
    /// when it was already retracted or no patient exists yet — the driver
    /// keeps the operation valid by construction).
    RetractChestPain { index: usize },
    // -- invalid ----------------------------------------------------------
    /// `heartRate` above `horis:max 300`.
    HeartRateAboveMax { heart_rate: i32 },
    /// `heartRate` given a string — an `rdfs:range` violation.
    HeartRateWrongDatatype,
    /// An undeclared class.
    UnknownClass,
    /// An undeclared property on a declared class.
    UnknownProperty,
    /// Perception writing the `horis:RulesOnly` property `acuity`.
    PerceptionSetsAcuity,
    /// A symptom reference nothing declares.
    UnknownSymptomRef,
    /// A caller-named subject squatting the reserved skolem namespace.
    SkolemSquat,
}

fn valid_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0..=300).prop_map(|heart_rate| Op::AssertPatient { heart_rate }),
        (0usize..8).prop_map(|index| Op::RetractChestPain { index }),
    ]
}

fn invalid_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (301..=5000).prop_map(|heart_rate| Op::HeartRateAboveMax { heart_rate }),
        Just(Op::HeartRateWrongDatatype),
        Just(Op::UnknownClass),
        Just(Op::UnknownProperty),
        Just(Op::PerceptionSetsAcuity),
        Just(Op::UnknownSymptomRef),
        Just(Op::SkolemSquat),
    ]
}

fn any_op() -> impl Strategy<Value = Op> {
    prop_oneof![valid_op(), invalid_op()]
}

/// Applies `op` to `world`, tracking asserted patients (and whether their
/// `:ChestPain` fact is still held) so `RetractChestPain` stays valid by
/// construction. Returns the gate's verdict for the invalid variants.
fn apply(world: &mut World, patients: &mut Vec<(Ref, bool)>, op: &Op) -> Result<(), HorismosError> {
    let patient = iri(world, "Patient");
    let heart_rate = iri(world, "heartRate");
    let has_symptom = iri(world, "hasSymptom");
    let chest_pain = iri(world, "ChestPain");
    let crushing = iri(world, "CrushingChestPain");
    match op {
        Op::AssertPatient { heart_rate: hr } => {
            let r = world.assert_instance(
                patient,
                [
                    (heart_rate, Value::Decimal(Decimal::from(*hr))),
                    (has_symptom, Value::Ref(Ref::new(chest_pain))),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )?;
            patients.push((r, true));
            Ok(())
        }
        Op::RetractChestPain { index } => {
            if patients.is_empty() {
                return Ok(());
            }
            let index = index % patients.len();
            let (subject, held) = patients[index];
            if !held {
                return Ok(());
            }
            world.retract(subject, has_symptom, &Value::Ref(Ref::new(chest_pain)))?;
            patients[index].1 = false;
            Ok(())
        }
        Op::HeartRateAboveMax { heart_rate: hr } => world
            .assert_instance(patient, [(heart_rate, Value::Decimal(Decimal::from(*hr)))])
            .map(drop),
        Op::HeartRateWrongDatatype => world
            .assert_instance(patient, [(heart_rate, Value::String("fast".to_owned()))])
            .map(drop),
        Op::UnknownClass => {
            let ghost = iri(world, "Ghost");
            world.assert_instance(ghost, []).map(drop)
        }
        Op::UnknownProperty => {
            let ghost = iri(world, "ghostProperty");
            world
                .assert_instance(patient, [(ghost, Value::Integer(1))])
                .map(drop)
        }
        Op::PerceptionSetsAcuity => {
            let acuity = iri(world, "acuity");
            let immediate = iri(world, "Immediate");
            world
                .assert_instance_as(
                    Provenance::Perception,
                    None,
                    patient,
                    [(acuity, Value::Ref(Ref::new(immediate)))],
                )
                .map(drop)
        }
        Op::UnknownSymptomRef => {
            let nowhere = iri(world, "NoSuchSymptom");
            world
                .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(nowhere)))])
                .map(drop)
        }
        Op::SkolemSquat => {
            let squatted = world.intern("https://w3id.org/horismos/.well-known/genid/0");
            world.assert_named_instance(squatted, patient, []).map(drop)
        }
    }
}

/// Re-checks a finished world against the schema it was built under, using
/// only public accessors: every `Patient`'s `heartRate` is a decimal within
/// the declared `[0, 300]`, and at most one value is held (cardinality
/// `0..1`). This is the "invalid world" detector property 1 relies on — it
/// must never fire if the gate did its job.
fn assert_world_valid(world: &mut World) {
    let patient = iri(world, "Patient");
    let heart_rate = iri(world, "heartRate");
    for subject in world.instances(patient).collect::<Vec<_>>() {
        let values: Vec<Value> = world.values(subject, heart_rate).cloned().collect();
        assert!(
            values.len() <= 1,
            "heartRate is 0..1, held {}",
            values.len()
        );
        for v in values {
            match v {
                Value::Decimal(d) => {
                    assert!(
                        d >= Decimal::ZERO && d <= Decimal::from(300),
                        "heartRate {d} escaped [0, 300]"
                    )
                }
                other => panic!("heartRate held a non-decimal: {other:?}"),
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Invariant 1: no sequence of valid operations produces an invalid
    /// world — and the world it does produce survives the full binary
    /// round-trip (`to_bytes` → `from_bytes` re-gates every fact).
    #[test]
    fn valid_operations_never_produce_an_invalid_world(ops in prop::collection::vec(valid_op(), 0..24)) {
        let onto = ontology();
        let mut world = World::new(&onto);
        let mut patients = Vec::new();
        for op in &ops {
            apply(&mut world, &mut patients, op).expect("a valid op was refused by the gate");
        }
        assert_world_valid(&mut world);

        let bytes = world.snapshot().to_bytes();
        let restored = horismos_core::Snapshot::from_bytes(&onto, &bytes)
            .expect("a gate-built world must re-validate from its own bytes");
        let restored = World::from_snapshot(&onto, restored);
        prop_assert_eq!(world.instance_count(), restored.instance_count());
    }

    /// Invariant 2: every rejection names its constraint — a structured
    /// witness plus domain text that speaks ontology vocabulary, never
    /// internal tokens.
    #[test]
    fn every_rejection_names_its_constraint(ops in prop::collection::vec(invalid_op(), 1..12)) {
        let onto = ontology();
        let mut world = World::new(&onto);
        let mut patients = Vec::new();
        for op in &ops {
            let err = apply(&mut world, &mut patients, op)
                .expect_err("an invalid op was admitted by the gate");
            let HorismosError::ValidationRejected { witness } = &err else {
                panic!("expected ValidationRejected, got {err:?}");
            };
            // The machine-readable half names the constraint the op tripped...
            let kind_matches = match op {
                Op::HeartRateAboveMax { .. } => matches!(witness.kind, ConstraintKind::Max { .. }),
                Op::HeartRateWrongDatatype => matches!(witness.kind, ConstraintKind::Range { .. }),
                Op::UnknownClass => matches!(witness.kind, ConstraintKind::ClassNotDeclared),
                Op::UnknownProperty => matches!(witness.kind, ConstraintKind::PropertyNotDeclared),
                Op::PerceptionSetsAcuity => {
                    matches!(witness.kind, ConstraintKind::ProvenanceRefused { .. })
                }
                Op::UnknownSymptomRef => matches!(witness.kind, ConstraintKind::UnknownReference),
                Op::SkolemSquat => matches!(witness.kind, ConstraintKind::ReservedSubject { .. }),
                valid => panic!("valid op {valid:?} in the invalid generator"),
            };
            prop_assert!(kind_matches, "{:?} produced the wrong witness kind: {:?}", op, witness.kind);
            prop_assert!(witness.property.is_some() || witness.subject.starts_with("http"),
                "witness addresses neither a property nor a subject: {witness:?}");
            // ...and the human half is in domain terms.
            let text = err.domain_text();
            prop_assert!(!text.is_empty());
            prop_assert!(!text.contains("Iri("), "internal token leaked: {text}");
            prop_assert_eq!(&text, &witness.text);
        }
        prop_assert_eq!(world.instance_count(), 0, "an invalid op mutated the world");
    }

    /// Invariant 3: a failing assertion is atomic — interleave one invalid
    /// op into a valid sequence and the world's bytes are identical to the
    /// run without it.
    #[test]
    fn a_rejected_operation_leaves_no_trace(
        prefix in prop::collection::vec(valid_op(), 0..8),
        bad in invalid_op(),
        suffix in prop::collection::vec(valid_op(), 0..8),
    ) {
        let onto = ontology();

        let mut with = World::new(&onto);
        let mut with_patients = Vec::new();
        let mut with_ops: Vec<&Op> = prefix.iter().collect();
        with_ops.push(&bad);
        with_ops.extend(suffix.iter());
        for op in with_ops {
            let _ = apply(&mut with, &mut with_patients, op);
        }

        let mut without = World::new(&onto);
        let mut without_patients = Vec::new();
        for op in prefix.iter().chain(suffix.iter()) {
            apply(&mut without, &mut without_patients, op).expect("valid op refused");
        }

        prop_assert_eq!(with.snapshot().to_bytes(), without.snapshot().to_bytes());
    }

    /// Invariant 4: determinism — the same operations against two fresh
    /// worlds yield byte-identical snapshots, identical skolem IRIs, and
    /// identical query iteration order.
    #[test]
    fn identical_operations_build_identical_worlds(ops in prop::collection::vec(any_op(), 0..24)) {
        let onto = ontology();
        let mut a = World::new(&onto);
        let mut b = World::new(&onto);
        let mut ap = Vec::new();
        let mut bp = Vec::new();
        for op in &ops {
            let ra = apply(&mut a, &mut ap, op);
            let rb = apply(&mut b, &mut bp, op);
            prop_assert_eq!(ra.is_ok(), rb.is_ok(), "verdicts diverged on {:?}", op);
        }
        prop_assert_eq!(a.snapshot().to_bytes(), b.snapshot().to_bytes());

        let patient_a = iri(&mut a, "Patient");
        let patient_b = iri(&mut b, "Patient");
        let order_a: Vec<String> =
            a.instances(patient_a).map(|r| a.resolve(r.iri()).to_owned()).collect();
        let order_b: Vec<String> =
            b.instances(patient_b).map(|r| b.resolve(r.iri()).to_owned()).collect();
        prop_assert_eq!(order_a, order_b);
    }
}
