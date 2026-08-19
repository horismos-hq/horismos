//! Truth maintenance for derived facts — regressions for the captain's
//! ruling (`tm-derived-facts.md`) and the independent review's top finding
//! 2: derived facts MUST be recomputed/invalidated when their supporting
//! base facts change or are retracted. Before this fix, `retract_as` never
//! re-chained at all, and `assert_instance_as` re-chained without first
//! clearing stale derivations, so a derived `:Immediate` from `heartRate`
//! 128 survived the heart rate being replaced by 96 or retracted outright.
//!
//! The fixture is the PRP's worked Example 1 — `:r1` derives `:acuity
//! :Immediate` from `heartRate > 120`, `:r3` derives `needsCardiacMonitor
//! true` from `acuity :Immediate` (derived-from-derived, for the cascade
//! regression). Declared inline: integration tests cannot reach the
//! crate-private `reason::testing` fixture, and the PRP marks `:r2`/`:r3`/
//! `:needsCardiacMonitor` "illustrative only", so they stay out of the
//! shipped `.ttl`.
//!
//! Spec: `docs/framework/03-world.md` § Truth maintenance · Owner: PRP-002

use horismos_core::error::HorismosError;
use horismos_core::reason::forward::ITERATION_CAP;
use horismos_core::{Iri, Ontology, Ref, Value, World};

const EXAMPLE_1: &str = "@prefix : <https://example.org/t#> .\n\
     @prefix horis: <https://w3id.org/horismos#> .\n\
     @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
     @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
     @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
     :Patient a rdfs:Class .\n\
     :AcuityLevel a rdfs:Class ; horis:oneOf ( :Immediate :Urgent :Standard ) .\n\
     :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;\n\
     \x20   horis:unit \"bpm\" ; horis:min 0 ; horis:max 300 ; horis:cardinality \"0..1\" .\n\
     :acuity a rdf:Property ; rdfs:domain :Patient ; rdfs:range :AcuityLevel ;\n\
     \x20   horis:cardinality \"0..1\" ; horis:assignedBy horis:RulesOnly .\n\
     :needsCardiacMonitor a rdf:Property ; rdfs:domain :Patient ;\n\
     \x20   rdfs:range xsd:boolean ; horis:cardinality \"0..1\" ;\n\
     \x20   horis:assignedBy horis:RulesOnly .\n\
     :r1 a horis:Rule ; horis:priority 10 ;\n\
     \x20 horis:when [ horis:all (\n\
     \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ]\n\
     \x20 ) ] ;\n\
     \x20 horis:then [ horis:property :acuity ; horis:value :Immediate ] ;\n\
     \x20 horis:explain \"Tachycardia — immediate track\" .\n\
     :r3 a horis:Rule ; horis:priority 8 ;\n\
     \x20 horis:when [ horis:all (\n\
     \x20   [ horis:subject :Patient ; horis:property :acuity ; horis:is :Immediate ]\n\
     \x20 ) ] ;\n\
     \x20 horis:then [ horis:property :needsCardiacMonitor ; horis:value true ] ;\n\
     \x20 horis:explain \"Immediate track — continuous cardiac monitoring\" .\n";

fn example_world() -> World {
    let onto = Ontology::load_from_str(EXAMPLE_1, |_| None).expect("Example 1 loads");
    World::new(&onto)
}

fn t(world: &mut World, local: &str) -> Iri {
    world.intern(&format!("https://example.org/t#{local}"))
}

/// (a) A derived `:Immediate` must not survive its supporting `heartRate`
/// being replaced by a value that no longer satisfies `r1`'s condition —
/// the world must re-derive to the *correct new* fixed point (nothing
/// stands, since no rule fires for `heartRate` 96), not just stop updating.
#[test]
fn replacing_the_supporting_heart_rate_invalidates_the_stale_derivation() {
    let mut world = example_world();
    let patient = t(&mut world, "Patient");
    let heart_rate = t(&mut world, "heartRate");
    let acuity = t(&mut world, "acuity");
    let immediate = t(&mut world, "Immediate");
    let p1 = t(&mut world, "p1");

    world
        .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(128))])
        .expect("128 exceeds 120: r1 fires");
    assert_eq!(
        world.value(Ref::new(p1), acuity),
        Some(&Value::Ref(Ref::new(immediate))),
        "sanity: :Immediate must be derived from heartRate 128 first"
    );

    world
        .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(96))])
        .expect("96 is gate-valid (within 0..300)");

    assert_eq!(
        world.value(Ref::new(p1), acuity),
        None,
        ":Immediate must not survive heartRate being replaced by 96 — no rule fires at 96"
    );
    assert!(
        world.explain(Ref::new(p1), acuity).is_none(),
        "a stale derivation record must not survive either"
    );
}

/// (b) Retracting the supporting `heartRate` outright removes the
/// derivation it alone supported.
#[test]
fn retracting_the_supporting_heart_rate_removes_the_unsupported_derivation() {
    let mut world = example_world();
    let patient = t(&mut world, "Patient");
    let heart_rate = t(&mut world, "heartRate");
    let acuity = t(&mut world, "acuity");
    let immediate = t(&mut world, "Immediate");
    let p1 = t(&mut world, "p1");

    world
        .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(128))])
        .expect("128 exceeds 120: r1 fires");
    assert_eq!(
        world.value(Ref::new(p1), acuity),
        Some(&Value::Ref(Ref::new(immediate)))
    );

    world
        .retract(Ref::new(p1), heart_rate, &Value::Integer(128))
        .expect("heartRate is a plain base fact, not immutable or derived");

    assert_eq!(
        world.value(Ref::new(p1), heart_rate),
        None,
        "the retracted base fact itself must be gone"
    );
    assert_eq!(
        world.value(Ref::new(p1), acuity),
        None,
        ":Immediate must not survive heartRate being retracted — nothing supports it any more"
    );
    assert!(world.explain(Ref::new(p1), acuity).is_none());
}

/// (c) Chained support: `needsCardiacMonitor` is derived from `:acuity
/// :Immediate` (derived-from-derived). Invalidating the root support must
/// cascade through the whole derivation chain, not just its immediate
/// conclusion.
#[test]
fn invalidating_a_root_support_cascades_through_derived_from_derived_facts() {
    let mut world = example_world();
    let patient = t(&mut world, "Patient");
    let heart_rate = t(&mut world, "heartRate");
    let acuity = t(&mut world, "acuity");
    let monitor = t(&mut world, "needsCardiacMonitor");
    let immediate = t(&mut world, "Immediate");
    let p1 = t(&mut world, "p1");

    world
        .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(128))])
        .expect("128 exceeds 120: r1 fires, then r3 fires off the derived acuity");
    assert_eq!(
        world.value(Ref::new(p1), acuity),
        Some(&Value::Ref(Ref::new(immediate)))
    );
    assert_eq!(
        world.value(Ref::new(p1), monitor),
        Some(&Value::Boolean(true)),
        "sanity: the chained conclusion must derive first"
    );

    world
        .retract(Ref::new(p1), heart_rate, &Value::Integer(128))
        .expect("plain base fact retraction");

    assert_eq!(
        world.value(Ref::new(p1), acuity),
        None,
        "the root derivation must be gone"
    );
    assert_eq!(
        world.value(Ref::new(p1), monitor),
        None,
        "needsCardiacMonitor was derived only from the now-invalidated acuity — it must cascade away too"
    );
}

/// Atomicity: `retract_as` now re-chains, so a retraction can trip the
/// iteration cap exactly as an assertion can — and must roll back the
/// *whole* transaction, the requested retraction included, leaving the
/// world byte-identical to before the attempt (doc 03 § Transactions).
///
/// 101 rules watch `heartRate isEmpty` — inert while `heartRate` is
/// present, so the first transaction (asserting it) is safe; retracting it
/// makes all 101 fireable at once on the re-chain, one over the cap.
#[test]
fn a_retraction_that_trips_the_iteration_cap_rolls_back_the_whole_transaction() {
    let mut rules_source = String::new();
    for i in 0..=ITERATION_CAP {
        rules_source.push_str(&format!(
            ":r{i:03} a horis:Rule ; horis:priority {i} ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :heartRate ; horis:isEmpty true ] ) ] ;\n\
             \x20 horis:then [ horis:property :tag ; horis:value \"tag-{i:03}\" ] .\n"
        ));
    }
    let source = format!(
        "@prefix : <https://example.org/t#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         :Patient a rdfs:Class .\n\
         :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;\n\
         \x20   horis:cardinality \"0..1\" .\n\
         :tag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
         \x20   horis:cardinality \"0..*\" .\n\
         {rules_source}"
    );
    let onto = Ontology::load_from_str(&source, |_| None).expect("valid");
    let mut world = World::new(&onto);
    let patient = t(&mut world, "Patient");
    let heart_rate = t(&mut world, "heartRate");
    let p1 = t(&mut world, "p1");

    world
        .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(80))])
        .expect("heartRate present: isEmpty is false, nothing fires, safe first transaction");
    assert_eq!(world.instance_count(), 1);
    let before = world.clone();

    let err = world
        .retract(Ref::new(p1), heart_rate, &Value::Integer(80))
        .expect_err("101 fireable rules on the re-chain is one over the cap");
    let HorismosError::IterationCapExceeded { cap, .. } = err else {
        panic!("expected IterationCapExceeded, got {err:?}");
    };
    assert_eq!(cap, ITERATION_CAP);

    assert_eq!(
        world.value(Ref::new(p1), heart_rate),
        Some(&Value::Integer(80)),
        "the requested retraction itself must roll back too, not just the derived tags"
    );
    assert_eq!(world.instance(Ref::new(p1)), before.instance(Ref::new(p1)));
    assert!(
        world.last_reasoning().is_some(),
        "sanity: last_reasoning is untouched by the rolled-back attempt (still the prior transaction's)"
    );
}
