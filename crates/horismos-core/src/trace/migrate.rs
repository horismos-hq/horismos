//! Schema-versioning policy and migration (U7, resolving PRP-002 Key
//! decision 3). Traces outlive framework releases — once used as training
//! data, a corpus must stay meaningful forever — so:
//!
//! **The policy** (`docs/framework/09-traces.md` design rule 1, made
//! precise here):
//!
//! - `horismos.trace/1` never changes meaning. Within v1, the ONLY
//!   permitted evolution is additive-optional: a new field that serializes
//!   with `skip_serializing_if` and deserializes with `default`, whose
//!   absence means exactly what v1 traces already mean. Old readers ignore
//!   it; old traces parse unchanged.
//! - **Breaking** — and therefore minting `horismos.trace/2` plus a pure
//!   converter here — is any of: changing the meaning or type of an
//!   existing field; adding a *required* field; adding a new step `kind`
//!   (consumers must understand every kind they replay); renaming or
//!   repurposing `loss_mask` or any existing `outcome.labels` key.
//! - The reserved training fields evolve without a version bump only by
//!   *adding keys*: new `outcome.labels` entries and new per-step optional
//!   annotations. Existing keys never change meaning — corpora already
//!   trained on stay valid (the TIR/CIR requirement, sigil-lang
//!   `docs/research/INTERLEAVED_SYMBOLIC_EXECUTION.md`).
//! - Converters are pure `serde_json::Value → Value` functions living in
//!   this module, named `convert_vN_to_vM`, each with a round-trip test.
//!   Conversion never guesses: a field v2 requires that v1 cannot supply is
//!   filled with an explicitly-marked migration default, never inferred.
//!
//! The training-export tag set (folded into the same decision): XML-style
//! per-step-kind tags — `<perceive>`, `<reason>`, `<act>`, `<narrate>`,
//! with `<observe>` wrapping masked executor output — the
//! `<llm-code>`/`<llm-code-output>` family (NVIDIA NeMo-Skills /
//! OpenMathInstruct-1), the largest existing TIR-corpus precedent. Tags are
//! exporter *template* data (doc 09's `format="tagged"`), not trace data:
//! the exporter (PRP-003+) consumes this constant; recorded traces never
//! contain them, so revising tags never touches a stored trace.
//!
//! The v1→v2 dry run demanded by the decision lives in this module's tests:
//! a hypothetical v2 (adds a required `engine` object) is migrated to, so
//! the first *real* schema change follows a rehearsed path.
//!
//! Spec: `docs/framework/09-traces.md` · Owner: PRP-002

/// The doc-09 training-export tag set (decided, PRP-002 — see module doc):
/// one tag per step kind, in doc-09 kind order, plus the executor-output
/// wrapper.
pub const EXPORT_TAGS: [(&str, &str); 5] = [
    ("perception", "perceive"),
    ("reasoning", "reason"),
    ("action", "act"),
    ("narration", "narrate"),
    ("executor-output", "observe"),
];

/// Parses the `N` out of a trace's `"schema": "horismos.trace/N"` tag.
/// `None` for a non-trace or malformed value — callers refuse, never guess.
pub fn schema_version(trace: &serde_json::Value) -> Option<u32> {
    trace
        .get("schema")?
        .as_str()?
        .strip_prefix("horismos.trace/")?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn schema_version_parses_v1_and_refuses_garbage() {
        assert_eq!(
            schema_version(&json!({"schema": "horismos.trace/1"})),
            Some(1)
        );
        assert_eq!(
            schema_version(&json!({"schema": "horismos.trace/2"})),
            Some(2)
        );
        assert_eq!(schema_version(&json!({"schema": "sigil.trace/1"})), None);
        assert_eq!(schema_version(&json!({"schema": "horismos.trace/"})), None);
        assert_eq!(schema_version(&json!({"run_id": "x"})), None);
    }

    /// The rehearsed v1→v2 migration (the decision's dry run): v2
    /// hypothetically requires an `engine` object v1 never carried. The
    /// converter is what the policy prescribes — pure, explicit-default,
    /// version-tag-rewriting — and the shape a real `convert_v1_to_v2`
    /// will take when v2 actually exists.
    fn convert_v1_to_v2_dry_run(mut trace: Value) -> Value {
        assert_eq!(
            schema_version(&trace),
            Some(1),
            "converters check before they touch"
        );
        let obj = trace.as_object_mut().expect("a trace is an object");
        obj.insert("schema".into(), Value::from("horismos.trace/2"));
        // A required v2 field v1 cannot supply: an explicitly-marked
        // migration default, never an inference.
        obj.insert(
            "engine".into(),
            json!({"name": "unknown", "note": "migrated from horismos.trace/1"}),
        );
        trace
    }

    #[test]
    fn the_v1_to_v2_dry_run_migrates_without_touching_v1_meaning() {
        let v1 = json!({
            "schema": "horismos.trace/1",
            "run_id": "run_test",
            "steps": [{
                "kind": "reasoning",
                "fired": [], "suppressed": [], "passes": 1, "loss_mask": true
            }]
        });

        let v2 = convert_v1_to_v2_dry_run(v1.clone());
        assert_eq!(schema_version(&v2), Some(2));
        assert_eq!(
            v2["engine"]["name"], "unknown",
            "explicit migration default"
        );
        // Every v1 field survives, meaning intact.
        assert_eq!(v2["run_id"], v1["run_id"]);
        assert_eq!(v2["steps"], v1["steps"]);
        // And the source object was not consumed into something lossy: the
        // original still parses as v1.
        assert_eq!(schema_version(&v1), Some(1));
    }

    #[test]
    fn the_export_tag_set_is_the_decided_vocabulary() {
        // Pins D-3's tag-set resolution so a drive-by rename shows up as a
        // failing test, not a silent corpus break.
        assert_eq!(
            EXPORT_TAGS,
            [
                ("perception", "perceive"),
                ("reasoning", "reason"),
                ("action", "act"),
                ("narration", "narrate"),
                ("executor-output", "observe"),
            ]
        );
    }
}
