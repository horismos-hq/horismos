//! PRP-004 U14 — the live triage-workflow demo: loads `triage.ttl` plus the
//! SNOMED CT subset, runs "58M, crushing chest pain, heart rate 128" through
//! the [`horismos_core::workflow::Agent`], and prints the proof tree and
//! narration the run produced.
//!
//! `make demo` runs this against [`MockBridge`] (deterministic, no network,
//! no key) — the same path `agent_loop_dod.rs` exercises. `make demo-live`
//! builds with `--features bridges-openrouter` and swaps in
//! `OpenRouterBridge::from_env()` when `OPENROUTER_API_KEY` is set (D4's gap
//! note: this is the one live-key path that can confirm forced
//! `tool_choice` against a real OpenRouter-proxied model).
//!
//! Trajectory JSON output (byte-for-byte golden comparison) is U9's
//! `trace::emit` — not wired into `Agent::run` yet (see
//! `PRPs/PRP-004-agent-loop/02-tasks-and-gaps.md`'s U9 entry). This demo
//! prints what `RunOutcome` already exposes: the terminal step, the
//! decision, the case subject's proof tree, and every narration.
//!
//! Spec: `docs/framework/05-workflows.md` § worked example · Owner: PRP-004

use std::collections::BTreeMap;

use horismos_bridges::mock::MockBridge;
use horismos_core::bridge::PerceiveResult;
use horismos_core::workflow::agent::Tool;
use horismos_core::workflow::{Agent, RunInput};
use horismos_core::{Iri, Ontology, Value, World};

#[cfg(feature = "bridges-openrouter")]
use horismos_bridges::openrouter::OpenRouterBridge;

const TRIAGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/triage/triage.ttl"
);
const SNOMED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/triage/snomed-ct-subset.ttl"
);
const INPUT: &str = "58M, crushing chest pain, heart rate 128";

/// `:ehr.assign_queue`'s `rust:assign_queue` binding — deterministic (no
/// clock, no randomness: CLAUDE.md), so a mock run's trajectory is
/// reproducible byte-for-byte once U9 wires trace emission through.
struct AssignQueue {
    queue_ref: Iri,
}
impl Tool for AssignQueue {
    fn call(&self, _args: &BTreeMap<String, Value>) -> Result<BTreeMap<Iri, Value>, String> {
        Ok(BTreeMap::from([(
            self.queue_ref,
            Value::String("ED-QUEUE-IMMEDIATE-7".to_owned()),
        )]))
    }
}

/// `:paging.notify_nurse`'s `rust:notify_nurse` binding.
struct NotifyNurse {
    paged_to: Iri,
}
impl Tool for NotifyNurse {
    fn call(&self, _args: &BTreeMap<String, Value>) -> Result<BTreeMap<Iri, Value>, String> {
        Ok(BTreeMap::from([(
            self.paged_to,
            Value::String("charge-nurse-oncall".to_owned()),
        )]))
    }
}

fn load_ontology() -> Ontology {
    Ontology::load(TRIAGE)
        .expect("triage.ttl must load")
        .with_import(SNOMED)
        .expect("the SNOMED subset must import")
}

fn mock_bridge() -> MockBridge {
    MockBridge::new()
        .script_perceive(
            INPUT,
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
            horismos_core::bridge::NarrateResult {
                text: "Patient triaged as immediate acuity: heart rate 128 bpm with \
                       crushing chest pain meets the tachycardia-plus-chest-pain rule."
                    .to_owned(),
                accounting: Default::default(),
            },
        )
}

#[tokio::main]
async fn main() {
    let onto = load_ontology();
    let queue_ref = onto.node_ref(":queueRef").expect(":queueRef is declared");
    let paged_to = onto.node_ref(":pagedTo").expect(":pagedTo is declared");
    let acuity = onto.node_ref(":acuity").expect(":acuity is declared");

    let live = std::env::var("OPENROUTER_API_KEY").is_ok();
    let mode = if live { "live (OpenRouter)" } else { "mock" };
    println!("Horismos triage agent demo — {mode}\ninput: {INPUT:?}\n");

    let mut builder = Agent::builder(&onto)
        .workflow(":Triage")
        .expect("Triage is declared")
        .tool("assign_queue", AssignQueue { queue_ref })
        .tool("notify_nurse", NotifyNurse { paged_to });

    #[cfg(feature = "bridges-openrouter")]
    {
        if live {
            builder = builder.bridge("claude", OpenRouterBridge::from_env());
        } else {
            builder = builder.bridge("claude", mock_bridge());
        }
    }
    #[cfg(not(feature = "bridges-openrouter"))]
    {
        if live {
            eprintln!(
                "OPENROUTER_API_KEY is set, but this binary wasn't built with \
                 --features bridges-openrouter — falling back to the mock bridge. \
                 Use `make demo-live` for the live path."
            );
        }
        builder = builder.bridge("claude", mock_bridge());
    }

    let agent = builder
        .build()
        .expect("every tool/model binding is registered");

    let mut world = World::new(&onto);
    let now = chrono::DateTime::parse_from_rfc3339("2026-08-19T00:00:00Z")
        .expect("a fixed RFC3339 literal parses");
    let outcome = agent
        .run(&mut world, RunInput::new("triage_demo_run", INPUT, now))
        .await
        .expect("the demo's fixed input reaches a terminal step");

    println!("terminal: {}", onto.compact(onto.resolve(outcome.terminal)));
    println!("fallback taken: {}", outcome.fallback_taken);
    if let Some((property, value)) = &outcome.decision {
        println!(
            "decision: {} = {}",
            onto.compact(onto.resolve(*property)),
            render_value(&onto, value)
        );
    }

    if let Some(case) = outcome.case {
        println!("\nproof (:acuity):");
        match world.explain(case, acuity) {
            Some(proof) => print!("{}", proof.render(&world)),
            None => println!("  (no rule-derived acuity — the run took a fallback edge)"),
        }
    }

    let narrations: Vec<_> = outcome
        .trace
        .steps
        .iter()
        .filter_map(|s| match s {
            horismos_core::trace::Step::Narration {
                step,
                text: Some(t),
                ..
            } => Some((step.clone(), t.clone())),
            _ => None,
        })
        .collect();
    if !narrations.is_empty() {
        println!("\nnarration:");
        for (step, text) in &narrations {
            println!("  [{}] {text}", step.as_deref().unwrap_or("(unnamed step)"));
        }
    }

    let trajectory_path = std::env::temp_dir().join("horismos_triage_demo_trajectory.json");
    std::fs::write(&trajectory_path, outcome.trace.to_json_pretty())
        .expect("the trajectory file must be writable");
    println!(
        "\ntrajectory ({} steps) written to {}",
        outcome.trace.steps.len(),
        trajectory_path.display()
    );
}

fn render_value(onto: &Ontology, value: &Value) -> String {
    match value {
        Value::Ref(r) => onto.compact(onto.resolve(r.iri())),
        other => other.lexical_form().unwrap_or_default(),
    }
}
