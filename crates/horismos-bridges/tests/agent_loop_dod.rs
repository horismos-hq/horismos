//! PRP-004 U15 — the Agent loop Definition-of-Done run: the immediate
//! track, the fallback track, and tool-effect validation, all against the
//! real [`MockBridge`] (not a hand-rolled stub) driving the real
//! [`Agent`]. Lives in `horismos-bridges` rather than
//! `crates/horismos-core/tests/` because `horismos-core` may not depend on
//! `horismos-bridges` even as a dev-dependency (the anti-wrapper boundary,
//! CLAUDE.md) — a test exercising `Agent` + the real `MockBridge` together
//! has nowhere else to live. Independent of `bridges-openrouter`: builds
//! and runs with default features, matching `make no-model-gate`'s sibling
//! guarantee for `horismos-core` itself.
//!
//! DoD track 4 (trajectory serialize+replay) is
//! `the_trajectory_serializes_and_replays_deterministically`, below.
//!
//! Spec: `PRPs/PRP-004-agent-loop/01-objective.md` § Definition of done ·
//! Owner: PRP-004

use std::collections::BTreeMap;

use horismos_bridges::mock::MockBridge;
use horismos_core::bridge::{NarrateResult, PerceiveResult};
use horismos_core::workflow::agent::Tool;
use horismos_core::workflow::{Agent, RunInput};
use horismos_core::{Iri, Ontology, Ref, Value, World};

const TRIAGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/triage/triage.ttl"
);
const SNOMED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/triage/snomed-ct-subset.ttl"
);
const IMMEDIATE_INPUT: &str = "58M, crushing chest pain, heart rate 128";

fn load_ontology() -> Ontology {
    Ontology::load(TRIAGE)
        .expect("triage.ttl must load")
        .with_import(SNOMED)
        .expect("the SNOMED subset must import")
}

fn fixed_now() -> chrono::DateTime<chrono::FixedOffset> {
    chrono::DateTime::parse_from_rfc3339("2026-08-19T00:00:00Z")
        .expect("a fixed RFC3339 literal parses")
}

/// A `Tool` that fills every property it is given with a fixed string —
/// enough to satisfy any `horis:effect` `TypedPlaceholder` in this fixture
/// without a real EHR/paging backend.
struct StubTool(Vec<Iri>);
impl Tool for StubTool {
    fn call(&self, _args: &BTreeMap<String, Value>) -> Result<BTreeMap<Iri, Value>, String> {
        Ok(self
            .0
            .iter()
            .map(|&p| (p, Value::String("stub".to_owned())))
            .collect())
    }
}

/// A `Tool` that returns a wrong-typed value (an integer where the
/// declared effect is `xsd:string`) for every property it is given —
/// exercises the gate-rejection branch of tool-effect validation.
struct WrongTypeTool(Vec<Iri>);
impl Tool for WrongTypeTool {
    fn call(&self, _args: &BTreeMap<String, Value>) -> Result<BTreeMap<Iri, Value>, String> {
        Ok(self.0.iter().map(|&p| (p, Value::Integer(42))).collect())
    }
}

/// DoD track 1 (immediate): a high-confidence perceive matching PRP-002
/// Example 1 drives the full `:Intake -> :Assess -> :Route -> :Explain ->
/// :Done` path, `acuity` resolving to `:Immediate` exactly as the
/// hand-computed proof predicts.
#[test]
fn the_immediate_track_reaches_done_with_acuity_immediate() {
    let onto = load_ontology();

    let bridge = MockBridge::new()
        .script_perceive(
            IMMEDIATE_INPUT,
            PerceiveResult {
                instances: vec![serde_json::json!({
                    "class": ":Patient",
                    "heartRate": 128,
                    "hasSymptom": ":CrushingChestPain"
                })],
                confidence: 0.94,
                fact_confidence: BTreeMap::new(),
                accounting: Default::default(),
            },
        )
        .script_narrate(
            "*",
            NarrateResult {
                text: "immediate acuity".to_owned(),
                accounting: Default::default(),
            },
        );

    let agent = Agent::builder(&onto)
        .workflow(":Triage")
        .expect("Triage is declared")
        .bridge("claude", bridge)
        .tool(
            "assign_queue",
            StubTool(vec![
                onto.node_ref(":queueRef").expect(":queueRef is declared"),
            ]),
        )
        .tool(
            "notify_nurse",
            StubTool(vec![
                onto.node_ref(":pagedTo").expect(":pagedTo is declared"),
            ]),
        )
        .build()
        .expect("every binding is registered");

    let mut world = World::new(&onto);
    let outcome = tokio_test_block_on(agent.run(
        &mut world,
        RunInput::new("run_dod_immediate", IMMEDIATE_INPUT, fixed_now()),
    ))
    .expect("the immediate track must complete");

    assert_eq!(
        onto.resolve(outcome.terminal),
        "https://example.org/triage#Done"
    );
    assert!(!outcome.fallback_taken);
    let case = outcome.case.expect("perception resolved a case subject");
    let acuity = onto.node_ref(":acuity").expect(":acuity is declared");
    assert_eq!(
        world.value(case, acuity),
        Some(&Value::Ref(Ref::new(
            onto.node_ref(":Immediate").expect(":Immediate is declared")
        )))
    );
    assert_eq!(outcome.trace.steps.len(), 4, "{:?}", outcome.trace.steps);
    let narrations = outcome
        .trace
        .steps
        .iter()
        .filter(|s| matches!(s, horismos_core::trace::Step::Narration { .. }))
        .count();
    assert_eq!(narrations, 1);
}

/// DoD track 2 (fallback): a below-threshold `PerceiveResult` (`:Intake`
/// declares `horis:confidence 0.85`) takes `:Intake`'s `horis:fallback`
/// edge to `:EscalateToNurse`. Doc06's algorithm ("4. Confidence check ...
/// no -> follow horis:fallback edge. The LLM never guesses") runs the
/// confidence gate *before* asserting anything (step 6), so no case
/// subject exists at all when `:EscalateToNurse` is reached this way.
///
/// **Gap found while writing this test** (dated below, and in
/// `02-tasks-and-gaps.md`): `:paging.notify_nurse`'s effects
/// (`:notifiedAt`/`:pagedTo`) are `rdfs:domain :Patient`, so invoking it
/// needs *some* `:Patient` instance to attach them to — but this edge
/// never creates one. `Agent::run` correctly refuses
/// (`WorkflowError::NoCaseSubject`, its own doc: "a malformed workflow
/// verification (PRP-005) should catch statically; this is the runtime
/// backstop") rather than silently skipping the tool or guessing a
/// subject. This fixture's `:EscalateToNurse` is therefore only actually
/// reachable-and-satisfiable via `:Route`'s `horis:onError` edge (see
/// `tool_effect_validation_rejects_a_wrong_typed_return_and_follows_on_error`,
/// where a case already exists) — reaching it via `:Intake`'s
/// low-confidence fallback specifically needs either a case-independent
/// escalation tool or runtime-minted-instance semantics that don't exist
/// yet; recorded as a gap, not fixed here.
#[test]
fn the_fallback_track_on_low_confidence_perception_refuses_rather_than_guessing() {
    let onto = load_ontology();
    const VAGUE_INPUT: &str = "patient seems unwell, hard to say more";

    let bridge = MockBridge::new().script_perceive(
        VAGUE_INPUT,
        PerceiveResult {
            instances: vec![],
            confidence: 0.4,
            fact_confidence: BTreeMap::new(),
            accounting: Default::default(),
        },
    );

    let agent = Agent::builder(&onto)
        .workflow(":Triage")
        .expect("Triage is declared")
        .bridge("claude", bridge)
        .tool(
            "assign_queue",
            StubTool(vec![
                onto.node_ref(":queueRef").expect(":queueRef is declared"),
            ]),
        )
        .tool(
            "notify_nurse",
            StubTool(vec![
                onto.node_ref(":pagedTo").expect(":pagedTo is declared"),
            ]),
        )
        .build()
        .expect("every binding is registered");

    let mut world = World::new(&onto);
    let err = tokio_test_block_on(agent.run(
        &mut world,
        RunInput::new("run_dod_fallback", VAGUE_INPUT, fixed_now()),
    ))
    .expect_err("no case subject exists to page a nurse about");

    assert!(matches!(
        err,
        horismos_core::workflow::WorkflowError::NoCaseSubject
    ));
}

/// DoD track 3 (tool-effect validation): an out-of-contract tool return
/// (an integer where `:queueRef` declares `xsd:string`) is refused at the
/// validation gate, so `:Route` follows `horis:onError` to
/// `:EscalateToNurse`, whose own well-formed `notify_nurse` tool still
/// fires.
#[test]
fn tool_effect_validation_rejects_a_wrong_typed_return_and_follows_on_error() {
    let onto = load_ontology();

    let bridge = MockBridge::new()
        .script_perceive(
            IMMEDIATE_INPUT,
            PerceiveResult {
                instances: vec![serde_json::json!({
                    "class": ":Patient",
                    "heartRate": 128,
                    "hasSymptom": ":CrushingChestPain"
                })],
                confidence: 0.94,
                fact_confidence: BTreeMap::new(),
                accounting: Default::default(),
            },
        )
        .script_narrate(
            "*",
            NarrateResult {
                text: "immediate acuity".to_owned(),
                accounting: Default::default(),
            },
        );

    let agent = Agent::builder(&onto)
        .workflow(":Triage")
        .expect("Triage is declared")
        .bridge("claude", bridge)
        .tool(
            "assign_queue",
            WrongTypeTool(vec![
                onto.node_ref(":queueRef").expect(":queueRef is declared"),
            ]),
        )
        .tool(
            "notify_nurse",
            StubTool(vec![
                onto.node_ref(":pagedTo").expect(":pagedTo is declared"),
            ]),
        )
        .build()
        .expect("every binding is registered");

    let mut world = World::new(&onto);
    let outcome = tokio_test_block_on(agent.run(
        &mut world,
        RunInput::new(
            "run_dod_tool_effect_validation",
            IMMEDIATE_INPUT,
            fixed_now(),
        ),
    ))
    .expect("the run completes via the onError edge, not a WorkflowError");

    assert_eq!(
        onto.resolve(outcome.terminal),
        "https://example.org/triage#EscalateToNurse"
    );
    assert!(outcome.fallback_taken);
    let case = outcome.case.expect("perception resolved a case subject");
    let queue_ref = onto.node_ref(":queueRef").expect(":queueRef is declared");
    assert_eq!(
        world.value(case, queue_ref),
        None,
        "the rejected assign_queue effect must never reach the world"
    );
    let paged_to = onto.node_ref(":pagedTo").expect(":pagedTo is declared");
    assert_eq!(
        world.value(case, paged_to),
        Some(&Value::String("stub".to_owned())),
        "the terminal ActionStep's own tool must still have fired"
    );
}

/// DoD track 4 (trajectory serialize+replay): a full run's trajectory
/// round-trips through JSON exactly (`Trace::from_json(t.to_json()) == t`,
/// pretty form too), and two independent runs of the same workflow against
/// fresh `World`s, given the same `run_id`/input/`now`, produce
/// byte-identical `to_json()` output — determinism (CLAUDE.md) extended to
/// the full trajectory, not just the final `World` state.
#[test]
fn the_trajectory_serializes_and_replays_deterministically() {
    fn run() -> horismos_core::trace::Trace {
        let onto = load_ontology();
        let bridge = MockBridge::new()
            .script_perceive(
                IMMEDIATE_INPUT,
                PerceiveResult {
                    instances: vec![serde_json::json!({
                        "class": ":Patient",
                        "heartRate": 128,
                        "hasSymptom": ":CrushingChestPain"
                    })],
                    confidence: 0.94,
                    fact_confidence: BTreeMap::new(),
                    accounting: Default::default(),
                },
            )
            .script_narrate(
                "*",
                NarrateResult {
                    text: "immediate acuity".to_owned(),
                    accounting: Default::default(),
                },
            );
        let agent = Agent::builder(&onto)
            .workflow(":Triage")
            .expect("Triage is declared")
            .bridge("claude", bridge)
            .tool(
                "assign_queue",
                StubTool(vec![
                    onto.node_ref(":queueRef").expect(":queueRef is declared"),
                ]),
            )
            .tool(
                "notify_nurse",
                StubTool(vec![
                    onto.node_ref(":pagedTo").expect(":pagedTo is declared"),
                ]),
            )
            .build()
            .expect("every binding is registered");
        let mut world = World::new(&onto);
        tokio_test_block_on(agent.run(
            &mut world,
            RunInput::new("run_dod_replay", IMMEDIATE_INPUT, fixed_now()),
        ))
        .expect("the immediate track must complete")
        .trace
    }

    let first = run();
    let second = run();
    assert_eq!(first.to_json(), second.to_json());
    assert_eq!(first.to_json_pretty(), second.to_json_pretty());

    let round_tripped =
        horismos_core::trace::Trace::from_json(&first.to_json()).expect("compact form parses");
    assert_eq!(round_tripped, first);
    let round_tripped_pretty = horismos_core::trace::Trace::from_json(&first.to_json_pretty())
        .expect("pretty form parses");
    assert_eq!(round_tripped_pretty, first);
}

/// A hand-rolled single-threaded executor — `Agent::run`'s `Bridge`/`Tool`
/// calls never actually pend against `MockBridge`, so no real reactor is
/// needed (matches `workflow::agent`'s own test-only `block_on`).
fn tokio_test_block_on<F: std::future::Future>(fut: F) -> F::Output {
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = pin!(fut);
    loop {
        if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
            return out;
        }
    }
}
