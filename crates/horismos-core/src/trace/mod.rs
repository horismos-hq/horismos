//! Traces: the trajectory of a run — audit log and training corpus, one object.
//!
//! The `horismos.trace/1` schema as serde types ([`schema`]), emission from
//! reasoning records ([`emit`]), and the versioning policy + migration path
//! ([`migrate`]). Trace equality is a meaningful test assertion: emission is
//! a pure function of (world, record), timing is runtime-stamped or absent,
//! and absent optionals are omitted from the JSON, never `null`. Replay is
//! deterministic re-execution (doc 09) and arrives with the runtime
//! (PRP-004).
//!
//! Spec: `docs/framework/09-traces.md` · Owner: PRP-002

pub mod emit;
pub mod migrate;
pub mod schema;

pub use emit::reasoning_step;
pub use schema::{Step, Trace};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use crate::ontology::term::{Iri, Ref, Value};
    use crate::reason::testing::example_world;

    fn t(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/t#{local}"))
    }

    /// Runs PRP Example 1 and wraps its reasoning record in a v1 trace.
    fn example_trace() -> (World, Trace) {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(128)),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("accepted");

        let record = world.last_reasoning().cloned().expect("chained");
        let mut trace = Trace::new("run_test");
        let step = reasoning_step(&world, Some(":Assess"), &record);
        trace.steps.push(step);
        (world, trace)
    }

    /// Determinism, at the serialization edge: two identical transactions
    /// against fresh worlds serialize to byte-identical JSON — compact and
    /// pretty (doc 04's guarantee, CLAUDE.md's "trace equality is a valid
    /// test assertion").
    #[test]
    fn identical_runs_serialize_byte_identically() {
        let (_, a) = example_trace();
        let (_, b) = example_trace();
        assert_eq!(a.to_json(), b.to_json());
        assert_eq!(a.to_json_pretty(), b.to_json_pretty());
    }

    /// The Example-1 reasoning step carries doc 09's shape: schema tag,
    /// kind, fired entries with bindings and proofs (subsumption explicit),
    /// the suppressed r2 with its winner, the pass count, and the
    /// precomputed loss mask.
    #[test]
    fn the_example_1_step_matches_doc_09s_shape() {
        let (_, trace) = example_trace();
        let json = trace.to_json();

        for expected in [
            "\"schema\":\"horismos.trace/1\"",
            "\"run_id\":\"run_test\"",
            "\"kind\":\"reasoning\"",
            "\"step\":\":Assess\"",
            "\"rule\":\":r1\"",
            "\"?p\"",
            "\"winner\":\":r1\"",
            "\"via\":\"subsumption(:CrushingChestPain)\"",
            "\"passes\":4",
            "\"loss_mask\":true",
            "\"provenance\":\"rule\"",
        ] {
            assert!(json.contains(expected), "missing {expected} in {json}");
        }

        let Step::Reasoning {
            fired, suppressed, ..
        } = &trace.steps[0]
        else {
            panic!("a reasoning step");
        };
        assert_eq!(fired.len(), 2, "r1 and r3 stand");
        assert_eq!(fired[0].rule, ":r1");
        assert_eq!(fired[1].rule, ":r3");
        assert_eq!(suppressed.len(), 1, "r2 lost to r1");
        assert_eq!(suppressed[0].rule, ":r2");
        assert_eq!(suppressed[0].winner, ":r1");
    }

    /// Compact and pretty forms parse back to the same Trace, equal to the
    /// original (the storage round trip).
    #[test]
    fn json_round_trips_in_both_forms() {
        let (_, trace) = example_trace();
        assert_eq!(
            Trace::from_json(&trace.to_json()).expect("compact parses"),
            trace
        );
        assert_eq!(
            Trace::from_json(&trace.to_json_pretty()).expect("pretty parses"),
            trace
        );
    }

    /// A gate-refused conclusion (PRP Example 3) reaches the trace as a
    /// diagnostic carrying the gate's own domain wording.
    #[test]
    fn gate_diagnostics_reach_the_trace_in_gate_wording() {
        use crate::Ontology;
        use crate::reason::testing::{EXAMPLE_1, PRELUDE};

        let source = format!(
            "{PRELUDE}{EXAMPLE_1}\
             :r9 a horis:Rule ; horis:priority 1 ;\n\
             \x20 horis:when [ horis:all (\n\
             \x20   [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :Symptom ]\n\
             \x20 ) ] ;\n\
             \x20 horis:then [ horis:property :acuity ; horis:value :Discharged ] .\n"
        );
        let onto = Ontology::load_from_str(&source, |_| None).expect("loads");
        let mut world = World::new(&onto);
        let patient = t(&mut world, "Patient");
        let has_symptom = t(&mut world, "hasSymptom");
        let symptom = t(&mut world, "Symptom");
        world
            .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(symptom)))])
            .expect("commits despite r9");

        let record = world.last_reasoning().cloned().expect("chained");
        let step = reasoning_step(&world, None, &record);
        let Step::Reasoning { diagnostics, .. } = &step else {
            panic!("reasoning step");
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, ":r9");
        assert_eq!(
            diagnostics[0].witness,
            "acuity :Discharged is not one of horis:oneOf (:Immediate :Urgent :Standard) on :Patient"
        );
    }

    /// Absent optionals are omitted, not null — the byte-stability rule.
    #[test]
    fn absent_optionals_are_omitted_from_the_json() {
        let (_, trace) = example_trace();
        let json = trace.to_json();
        assert!(!json.contains("null"), "no nulls, ever: {json}");
        assert!(!json.contains("latency_ms"), "unstamped timing is absent");
        assert!(
            !json.contains("\"outcome\""),
            "no outcome until a runtime computes one"
        );
    }
}
