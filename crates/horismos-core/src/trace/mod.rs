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

pub use emit::{
    AssertedEffect, PerceivedFact, action_step, narration_step, perception_step, proposal_step,
    reasoning_step,
};
pub use schema::{Step, Trace};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use crate::ontology::term::{Iri, Ref, Value};
    use crate::reason::testing::example_world;
    use crate::world::Provenance;

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

    // -- U9: round-trip tests for every one of the eight declared step
    // kinds (Perception, Reasoning, Action, Narration, Proposal, Wait,
    // Choice, Subflow — captain decision 2026-08-19). Reasoning's own
    // round trip is `json_round_trips_in_both_forms` above; the four v1
    // kinds here use their `trace::emit` builders, the three v2-only kinds
    // that have no builder yet (Choice/Subflow/Wait — U12 defers their
    // execution, so there is no typed input to build one from) use a
    // hand-built `Step` fixture, matching D13's own resolution note.

    /// `to_json`/`from_json` in both forms reproduce the trace exactly —
    /// the same assertion `json_round_trips_in_both_forms` makes, factored
    /// out so every kind below can reuse it.
    fn assert_round_trips(trace: &Trace) {
        assert_eq!(
            Trace::from_json(&trace.to_json()).expect("compact parses"),
            *trace
        );
        assert_eq!(
            Trace::from_json(&trace.to_json_pretty()).expect("pretty parses"),
            *trace
        );
    }

    #[test]
    fn perception_step_round_trips() {
        let mut world = example_world();
        let subject = Ref::new(t(&mut world, "p7"));
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        let step = perception_step(
            &world,
            Some(":Intake"),
            Some("claude:claude-fable-5".to_owned()),
            Some(0.94),
            Some(0.85),
            &[
                PerceivedFact {
                    subject,
                    property: heart_rate,
                    value: Value::Integer(128),
                    confidence: Some(0.94),
                },
                PerceivedFact {
                    subject,
                    property: has_symptom,
                    value: Value::Ref(Ref::new(crushing)),
                    confidence: Some(0.91),
                },
            ],
        );
        let Step::Perception {
            accepted, asserted, ..
        } = &step
        else {
            panic!("a perception step");
        };
        assert_eq!(*accepted, Some(true), "0.94 >= 0.85");
        assert_eq!(asserted.len(), 2);
        assert_eq!(asserted[0].provenance, Some(Provenance::Perception));

        let mut trace = Trace::new("run_perception");
        trace.steps.push(step);
        assert_round_trips(&trace);
    }

    #[test]
    fn action_step_round_trips() {
        let mut world = example_world();
        let subject = Ref::new(t(&mut world, "p7"));
        let queued_at = t(&mut world, "queuedAt");
        let step = action_step(
            &world,
            Some(":Route"),
            Some(":ehr.assign_queue"),
            &[AssertedEffect {
                subject,
                property: queued_at,
                value: Value::String("2026-07-28T09:14:05Z".to_owned()),
            }],
        );
        let Step::Action {
            effects_asserted, ..
        } = &step
        else {
            panic!("an action step");
        };
        assert_eq!(effects_asserted[0].provenance, Some(Provenance::Tool));

        let mut trace = Trace::new("run_action");
        trace.steps.push(step);
        assert_round_trips(&trace);
    }

    #[test]
    fn narration_step_round_trips() {
        let step = narration_step(
            Some(":Explain"),
            Some("charge-nurse".to_owned()),
            Some("Acuity is immediate: heart rate 128 with crushing chest pain.".to_owned()),
        );
        let mut trace = Trace::new("run_narration");
        trace.steps.push(step);
        assert_round_trips(&trace);
    }

    #[test]
    fn proposal_step_round_trips() {
        let mut world = example_world();
        let option_a = Ref::new(t(&mut world, "OptionA"));
        let option_b = Ref::new(t(&mut world, "OptionB"));
        let step = proposal_step(
            &world,
            Some(":Recommend"),
            Some("claude:claude-fable-5".to_owned()),
            &[option_a, option_b],
            Some(0.81),
            Some(0.7),
            Some(option_a),
        );
        let Step::Proposal {
            candidates, chosen, ..
        } = &step
        else {
            panic!("a proposal step");
        };
        assert_eq!(
            candidates,
            &vec![":OptionA".to_owned(), ":OptionB".to_owned()]
        );
        assert_eq!(chosen.as_deref(), Some(":OptionA"));

        // Proposal is v2-only (D13) — a real `Trace::new_v2` run, not `new`.
        let mut trace = Trace::new_v2("run_proposal");
        trace.steps.push(step);
        assert_round_trips(&trace);
    }

    #[test]
    fn choice_step_round_trips() {
        // Choice execution is deferred (U12); no builder exists yet, so this
        // is a hand-built fixture (D13's own resolution note) exercising
        // only the schema and the byte round trip.
        let step = Step::Choice {
            step: Some(":RouteBySeverity".to_owned()),
            branch_taken: Some(":UrgentPath".to_owned()),
            otherwise: false,
            latency_ms: None,
            loss_mask: true,
        };
        let mut trace = Trace::new_v2("run_choice");
        trace.steps.push(step);
        assert_round_trips(&trace);
    }

    #[test]
    fn subflow_step_round_trips() {
        let nested = Step::Reasoning {
            step: Some(":NestedAssess".to_owned()),
            fired: vec![],
            suppressed: vec![],
            diagnostics: vec![],
            passes: 1,
            goal_reached: Some(true),
            latency_ms: None,
            loss_mask: true,
        };
        let step = Step::Subflow {
            step: Some(":Delegate".to_owned()),
            workflow: ":SubTriage".to_owned(),
            steps: vec![nested],
            terminal: ":SubDone".to_owned(),
            latency_ms: None,
            loss_mask: true,
        };
        let mut trace = Trace::new_v2("run_subflow");
        trace.steps.push(step);
        assert_round_trips(&trace);
    }

    #[test]
    fn wait_step_round_trips() {
        // D3: WaitStep execution is deferred to v1.1 — hand-built fixture
        // since no code path produces one yet (matches `trace::migrate`'s
        // own note on this kind).
        let step = Step::Wait {
            step: Some(":AwaitApproval".to_owned()),
            reason: Some("awaiting human sign-off".to_owned()),
            loss_mask: true,
        };
        let mut trace = Trace::new_v2("run_wait");
        trace.steps.push(step);
        assert_round_trips(&trace);
    }

    /// Determinism (CLAUDE.md), extended to the new v2 kinds: two
    /// identical multi-kind traces serialize byte-identically.
    #[test]
    fn v2_traces_with_the_new_kinds_serialize_byte_identically() {
        fn build() -> Trace {
            let mut trace = Trace::new_v2("run_det");
            trace.steps.push(Step::Choice {
                step: Some(":RouteBySeverity".to_owned()),
                branch_taken: Some(":UrgentPath".to_owned()),
                otherwise: false,
                latency_ms: None,
                loss_mask: true,
            });
            trace.steps.push(Step::Wait {
                step: Some(":AwaitApproval".to_owned()),
                reason: Some("awaiting human sign-off".to_owned()),
                loss_mask: true,
            });
            trace
        }
        assert_eq!(build().to_json(), build().to_json());
    }
}
