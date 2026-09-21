//! The **text → decisions** demo: a clinical note goes in, TypeSafe's Jev
//! proposes four typed booleans with a probability each, and
//! `examples/vte/vte.ttl`'s three rules decide the recommendation — with a
//! proof tree saying exactly which rule fired and on what.
//!
//! The point is not that an API was called. It is that the model proposed
//! and the rules decided, and you can see why. `:recommendation` carries
//! `horis:assignedBy horis:RulesOnly`, so the schema the model is shown does
//! not contain it: the model has no way to express the decision even if it
//! wanted to. Not discouraged — absent.
//!
//! ```
//! make demo-jev        # MockBridge: deterministic, no network, no key
//! make demo-jev-live   # JevBridge over OpenRouter when OPENROUTER_API_KEY is set
//! ```
//!
//! The two notes and the per-property question wording are lifted from the
//! wire-protocol probe this bridge was written against, so the example and
//! the probe agree on what was measured.
//!
//! Spec: `docs/framework/08-bridge.md` · Owner: PRP-004

use std::collections::BTreeMap;

use horismos_core::bridge::{Bridge, PerceiveRequest, PerceiveResult};
use horismos_core::workflow::agent::extract_schema;
use horismos_core::{Iri, Ontology, Provenance, Value, World};

#[cfg(feature = "bridges-jev")]
use horismos_bridges::jev::{JevBridge, JevTransport, PropertyHint};

const VTE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/vte/vte.ttl");

/// The probe's default note: reduced mobility is plain, the malignancy is
/// historical, and nothing bleeds.
const NOTE_CLEAR: &str = "\
72-year-old admitted yesterday following a fall at home with a fractured \
right neck of femur, awaiting theatre. Mobilising only with assistance and \
largely confined to bed since admission. Past history of breast carcinoma \
treated eight years ago, in remission, no current therapy. No previous clots \
documented. Haemoglobin stable, platelets 210, not anticoagulated, no \
bleeding noted on the ward round.";

/// The probe's borderline note. "On warfarin" is the ambiguity: already
/// anticoagulated reads as a contraindication to *adding* pharmacologic
/// prophylaxis, but the note never says so outright.
const NOTE_UNCERTAIN: &str = "78yo, post-op day 2, on warfarin, bed-bound";

/// The four properties, in the order the demo prints them.
const FACTS: [&str; 4] = [
    "reducedMobility",
    "activeCancer",
    "priorVte",
    "bleedingRisk",
];

fn main() {
    let onto = Ontology::load(VTE).expect("vte.ttl must load and verify");
    let vte_case = onto.node_ref(":VteCase").expect(":VteCase is declared");
    let recommendation = onto
        .node_ref(":recommendation")
        .expect(":recommendation is declared");

    // The same derivation `Agent::run` performs for a PerceptionStep's
    // `horis:extracts` — not a hand-written stand-in.
    let schema = extract_schema(&onto, &[vte_case]);

    let (bridge, mode) = build_bridge();
    println!("Horismos VTE text→decisions demo — {mode}");
    println!(
        "ontology: examples/vte/vte.ttl  ·  model requested: {}",
        bridge.id()
    );
    println!(
        "\n:recommendation is `horis:assignedBy horis:RulesOnly`, so it is absent from the\n\
         schema the model is shown. The model proposes facts; the rules decide.\n\
         schema asks the model for: {}",
        schema_properties(&schema).join(", ")
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a current-thread runtime must build");

    for (label, note) in [("1", NOTE_CLEAR), ("2", NOTE_UNCERTAIN)] {
        rule(&format!("Note {label}"));
        println!("{}\n", wrap(note, "  "));

        let result = match runtime.block_on(bridge.perceive(PerceiveRequest {
            text: note.to_string(),
            schema: schema.clone(),
        })) {
            Ok(result) => result,
            Err(err) => {
                // A Bridge error is not a crash: the runtime treats it
                // exactly as low confidence and takes the step's
                // `horis:fallback` edge (doc 08). Printed in full because
                // the refusals this bridge makes are the interesting part.
                println!(
                    "  bridge error (would take the fallback edge):\n    {}",
                    err.domain_text()
                );
                continue;
            }
        };

        println!("  Jev proposed these typed facts, one parallel pass:");
        let per_property = result
            .accounting
            .raw_metadata
            .as_ref()
            .and_then(|m| m.pointer("/fact_confidence_by_property/0"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let instance = result
            .instances
            .first()
            .expect("the schema declares one class, so one instance comes back");
        for name in FACTS {
            let value = instance
                .get(name)
                .and_then(serde_json::Value::as_bool)
                .expect("every vte.ttl property is a required boolean");
            let p = per_property
                .get(name)
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(f64::NAN);
            println!("    {name:<18} {value:<5}  p={p:.3}  {}", bar(p));
        }
        println!(
            "\n  call confidence {:.3} (the weakest fact in the call — nothing averages away)",
            result.confidence
        );
        if let Some(weakest) = result.fact_confidence.get(&0) {
            println!("  instance[0] confidence {weakest:.3}");
        }

        // Every proposed fact still passes the validation gate on the way in
        // (doc 03) — perception provenance buys no exemption.
        let mut world = World::new(&onto);
        let case = world
            .assert_instance_as(
                Provenance::Perception,
                None,
                vte_case,
                typed_properties(&onto, instance),
            )
            .expect("the proposed facts must clear the validation gate");

        println!("\n  The rules decided. The model had no vote:");
        match world.explain(case, recommendation) {
            Some(proof) => {
                for line in proof.render(&world).lines() {
                    println!("    {line}");
                }
            }
            None => println!("    (no rule derived :recommendation for these facts)"),
        }

        print_accounting(&result);
    }

    rule("Why this is the whole point");
    // `Bridge::id()` reports the model *actually invoked* — the alias asked
    // for was `typesafe/jev-1.13`; the wire answered with a dated pin.
    println!(
        "  model actually invoked (Bridge::id, the trace's `model` field): {}",
        bridge.id()
    );
    println!("  Two notes, two different fact sets, two different rules firing — and in both");
    println!("  cases the recommendation is a derivation you can read, not a model's opinion.");
    println!("  Delete the bridge and the rules still run; delete the rules and there is no");
    println!("  decision at all.");
}

/// The property names the derived schema actually asks the model for —
/// `:recommendation` is not among them, which is the claim this demo makes.
fn schema_properties(schema: &serde_json::Value) -> Vec<String> {
    schema
        .pointer("/properties/instances/items/oneOf/0/properties")
        .and_then(serde_json::Value::as_object)
        .map(|props| {
            props
                .keys()
                .filter(|k| *k != "class")
                .map(String::clone)
                .collect()
        })
        .unwrap_or_default()
}

/// Turns one proposed JSON instance into the typed property values the
/// World gate takes. `vte.ttl` is all booleans; anything else is a bug in
/// this demo's fixture, not a case to guess at.
fn typed_properties(onto: &Ontology, instance: &serde_json::Value) -> Vec<(Iri, Value)> {
    let mut out = Vec::new();
    let object = instance
        .as_object()
        .expect("a proposed instance is a JSON object");
    for (name, value) in object {
        if name == "class" {
            continue;
        }
        let property = onto
            .node_ref(&format!(":{name}"))
            .unwrap_or_else(|e| panic!("the model proposed an undeclared property {name:?}: {e}"));
        let boolean = value
            .as_bool()
            .unwrap_or_else(|| panic!("vte.ttl declares {name:?} xsd:boolean, got {value}"));
        out.push((property, Value::Boolean(boolean)));
    }
    out
}

fn print_accounting(result: &PerceiveResult) {
    let a = &result.accounting;
    let cost = a
        .cost_usd
        .map(|c| format!("${c:.6}"))
        .unwrap_or_else(|| "—".to_string());
    println!(
        "\n  {} ms  ·  {} in / {} out tokens  ·  {cost}  ·  model {}",
        a.latency_ms
            .map(|m| m.to_string())
            .unwrap_or_else(|| "—".into()),
        a.prompt_tokens
            .map(|t| t.to_string())
            .unwrap_or_else(|| "—".into()),
        a.completion_tokens
            .map(|t| t.to_string())
            .unwrap_or_else(|| "—".into()),
        a.model.as_deref().unwrap_or("—"),
    );
}

// -- Bridge selection ------------------------------------------------------

/// The four questions, worded as the probe worded them. The ontology has no
/// vocabulary for question text (doc 02), so it is supplied at deployment.
#[cfg(feature = "bridges-jev")]
fn hinted_bridge() -> JevBridge {
    JevBridge::from_env()
        .with_hint(
            "reducedMobility",
            PropertyHint::boolean(
                "Is the patient's mobility reduced relative to their usual baseline?",
                "Bed-bound, immobile, confined to a chair, or mobilising only with assistance.",
                "Independently mobile at their usual baseline.",
            ),
        )
        .with_hint(
            "activeCancer",
            PropertyHint::boolean(
                "Does the patient have active malignancy?",
                "Malignancy currently present or under active treatment.",
                "No malignancy, or a historical malignancy in remission with no current treatment.",
            ),
        )
        .with_hint(
            "priorVte",
            PropertyHint::boolean(
                "Has the patient previously had a venous thromboembolism?",
                "A documented previous DVT or pulmonary embolism.",
                "No previous venous thromboembolism documented.",
            ),
        )
        .with_hint(
            "bleedingRisk",
            PropertyHint::boolean(
                "Is there a contraindication to pharmacologic prophylaxis on bleeding grounds?",
                "Active bleeding, a recent major bleed, severe thrombocytopenia, or full \
                 anticoagulation already.",
                "No bleeding contraindication identified.",
            ),
        )
}

/// Whether a live key is configured, by the same rule `JevBridge` selects
/// its transport with: the bearer-token variable, **or** the `_FILE`
/// variant naming a gitignored file to read it from. Checking only the bare
/// variable would run the mock while printing "no API key set" for a
/// deployment that had configured one.
#[cfg(feature = "bridges-jev")]
fn live_key_configured() -> bool {
    JevTransport::OpenRouter.key_configured() || JevTransport::TypeSafe.key_configured()
}

#[cfg(not(feature = "bridges-jev"))]
fn live_key_configured() -> bool {
    ["OPENROUTER_API_KEY", "TYPESAFE_API_KEY"]
        .iter()
        .any(|var| std::env::var(var).is_ok() || std::env::var(format!("{var}_FILE")).is_ok())
}

fn build_bridge() -> (Box<dyn Bridge>, &'static str) {
    let live = live_key_configured();

    #[cfg(feature = "bridges-jev")]
    {
        if live {
            return (Box::new(hinted_bridge()), "live (Jev)");
        }
        return (Box::new(mock_bridge()), "mock (no API key set)");
    }
    #[cfg(not(feature = "bridges-jev"))]
    {
        if live {
            eprintln!(
                "An API key is set, but this binary wasn't built with --features bridges-jev — \
                 falling back to the mock bridge. Use `make demo-jev-live` for the live path."
            );
        }
        (Box::new(mock_bridge()), "mock")
    }
}

/// The mock path replays probabilities in the shape a live `JevBridge`
/// returns them — same `fact_confidence`, same `raw_metadata` — so the
/// printing above has exactly one code path.
fn mock_bridge() -> horismos_bridges::mock::MockBridge {
    horismos_bridges::mock::MockBridge::with_id("mock:jev-1.13")
        .script_perceive(
            NOTE_CLEAR,
            scripted(
                [
                    ("reducedMobility", true, 0.96),
                    ("activeCancer", false, 0.88),
                    ("priorVte", false, 0.96),
                    ("bleedingRisk", false, 0.93),
                ],
                619,
                79,
            ),
        )
        .script_perceive(
            NOTE_UNCERTAIN,
            scripted(
                [
                    ("reducedMobility", true, 0.94),
                    ("activeCancer", false, 0.91),
                    ("priorVte", false, 0.72),
                    ("bleedingRisk", true, 0.61),
                ],
                58,
                79,
            ),
        )
}

fn scripted(
    facts: [(&str, bool, f64); 4],
    input_tokens: u64,
    output_tokens: u64,
) -> PerceiveResult {
    let mut instance = serde_json::Map::new();
    instance.insert("class".into(), serde_json::json!(":VteCase"));
    let mut confidences = serde_json::Map::new();
    let mut weakest = f64::INFINITY;
    for (name, value, p) in facts {
        instance.insert(name.to_string(), serde_json::json!(value));
        confidences.insert(name.to_string(), serde_json::json!(p));
        weakest = weakest.min(p);
    }
    PerceiveResult {
        instances: vec![serde_json::Value::Object(instance)],
        confidence: weakest,
        fact_confidence: BTreeMap::from([(0, weakest)]),
        accounting: horismos_core::bridge::CallAccounting {
            latency_ms: Some(0),
            prompt_tokens: Some(input_tokens),
            completion_tokens: Some(output_tokens),
            cost_usd: Some(input_tokens as f64 * 0.042 / 1_000_000.0),
            model: Some("mock:jev-1.13".into()),
            finish_reason: None,
            raw_metadata: Some(serde_json::json!({
                "transport": "mock",
                "fact_confidence_by_property": [serde_json::Value::Object(confidences)],
            })),
        },
    }
}

// -- Presentation ----------------------------------------------------------

fn rule(title: &str) {
    println!("\n{}", "─".repeat(78));
    println!("{title}");
    println!("{}", "─".repeat(78));
}

fn bar(p: f64) -> String {
    if p.is_nan() {
        return String::new();
    }
    let width = 24;
    let filled = (p * width as f64).round().clamp(0.0, width as f64) as usize;
    format!("{}{}", "█".repeat(filled), "·".repeat(width - filled))
}

fn wrap(text: &str, indent: &str) -> String {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.chars().count() + word.chars().count() + 1 > 76 {
            lines.push(format!("{indent}{current}"));
            current.clear();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(format!("{indent}{current}"));
    }
    lines.join("\n")
}
