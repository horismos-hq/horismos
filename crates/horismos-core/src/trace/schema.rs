//! The `horismos.trace/1` trajectory schema (U7) — doc 09, as serde types.
//!
//! One object serves auditors, operators, and training pipelines
//! (principle 2: the audit log and the training corpus are the same
//! object). The schema is **versioned forever**: `horismos.trace/1` never
//! changes meaning; see [`crate::trace::migrate`] for the compatibility
//! policy and the rehearsed v1→v2 path.
//!
//! All four step kinds of doc 09 are part of v1 — `perception`, `reasoning`,
//! `action`, and `narration` — so the phases that emit the other three
//! (PRP-004) extend nothing; PRP-002 emits `reasoning` steps. Every field
//! that a phase does not yet populate is optional and *omitted* from the
//! JSON (never `null`), which is also what keeps trace equality a valid,
//! byte-exact test assertion for the deterministic subset.
//!
//! Everything here is resolved strings — compacted IRIs, lexical values —
//! never interned tokens, which are meaningless outside their world. The
//! conversion happens once, at emission ([`crate::trace::emit`]).
//!
//! Spec: `docs/framework/09-traces.md` · Owner: PRP-002

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::world::Provenance;

/// The schema tag every v1 trace carries.
pub const SCHEMA_V1: &str = "horismos.trace/1";

/// A full trajectory (doc 09). PRP-002 populates `steps` with reasoning
/// steps; run-level fields it cannot know (workflow, input, model timing)
/// stay `None` until the agent loop (PRP-004) owns emission.
///
/// `run_id` is caller-supplied: the core mints no identifiers — no
/// randomness, no wall-clock (CLAUDE.md determinism).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    /// Always [`SCHEMA_V1`] for traces this build writes.
    pub schema: String,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ontology: Option<OntologyRef>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub workflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub input: Option<Input>,
    pub steps: Vec<Step>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub outcome: Option<Outcome>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub totals: Option<Totals>,
}

impl Trace {
    /// A new, empty v1 trace under a caller-chosen run id.
    pub fn new(run_id: impl Into<String>) -> Trace {
        Trace {
            schema: SCHEMA_V1.to_owned(),
            run_id: run_id.into(),
            ontology: None,
            workflow: None,
            started_at: None,
            input: None,
            steps: Vec::new(),
            outcome: None,
            totals: None,
        }
    }

    /// Compact JSON — the JSONL/storage form (doc 09).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("a Trace is plain data; serialization cannot fail")
    }

    /// Pretty JSON — the human-inspection form.
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self)
            .expect("a Trace is plain data; serialization cannot fail")
    }

    /// Parses a v1 trace back from JSON (compact or pretty).
    pub fn from_json(json: &str) -> Result<Trace, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// The ontology a run executed against (doc 09).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OntologyRef {
    pub iri: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hash: Option<String>,
}

/// The run's input (doc 09) — natural language or structured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub text: Option<String>,
}

/// One step of the trajectory, tagged by `kind` exactly as doc 09's JSON
/// shows. `loss_mask` is precomputed at emission (design rule 2): model
/// output is trainable (`false` = learn), executor output is masked
/// (`true` = mask).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Step {
    /// A transducer extraction (doc 06) — emitted by PRP-004.
    Perception {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        step: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        model: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        confidence: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        threshold: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        accepted: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        latency_ms: Option<u64>,
        asserted: Vec<TraceFact>,
        loss_mask: bool,
    },
    /// A forward-chaining pass to fixed point — PRP-002's step.
    Reasoning {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        step: Option<String>,
        fired: Vec<FiredEntry>,
        suppressed: Vec<SuppressedEntry>,
        /// Gate-refused conclusions — the constraint-diagnostic channel
        /// (PRP Example 3). Empty on a healthy run.
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        diagnostics: Vec<DiagnosticEntry>,
        /// Passes to the fixed point, the final empty pass included.
        passes: usize,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        goal_reached: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        latency_ms: Option<u64>,
        loss_mask: bool,
    },
    /// A tool invocation (doc 07) — emitted by PRP-004.
    Action {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        step: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        tool: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        latency_ms: Option<u64>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        effects_asserted: Vec<TraceFact>,
        loss_mask: bool,
    },
    /// A narration rendering (doc 06) — emitted by PRP-004.
    Narration {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        step: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        audience: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        text: Option<String>,
        loss_mask: bool,
    },
}

/// One fact in trace form: subject / property / object as compacted IRIs
/// and lexical values, plus per-fact provenance (design rule 4) and, for
/// perception, confidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceFact {
    pub s: String,
    pub p: String,
    pub o: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provenance: Option<Provenance>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub conf: Option<f64>,
}

/// One rule firing whose conclusion stands (doc 09 `fired`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FiredEntry {
    pub rule: String,
    pub bindings: BTreeMap<String, String>,
    pub derived: Vec<TraceFact>,
    pub proof: ProofJson,
}

/// A conflict-policy loser (doc 09 `suppressed`) — recorded, never silently
/// dropped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuppressedEntry {
    pub rule: String,
    pub bindings: BTreeMap<String, String>,
    pub conclusion: TraceFact,
    pub winner: String,
}

/// A gate-refused conclusion: the diagnostic in gate wording plus the
/// machine-readable pieces the witness carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticEntry {
    pub rule: String,
    pub bindings: BTreeMap<String, String>,
    pub conclusion: TraceFact,
    pub witness: String,
}

/// A proof tree in trace form (doc 04's ProofNode, resolved): `via` is the
/// rule IRI, `"asserted"`, `"subsumption(<class>)"`, `"derived"`, or
/// `"closed-world"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofJson {
    pub goal: String,
    pub via: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub explain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provenance: Option<Provenance>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub conditions: Vec<ProofJson>,
}

/// The run outcome (doc 09): deterministic labels, computed by the runtime
/// — "reward for free". PRP-002 reserves the shape; PRP-004 populates it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub terminal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub goal_reached: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub decision: Option<TraceFact>,
    /// Reward-ready labels. Keys may be *added* within v1 (see
    /// `trace::migrate`'s policy); existing keys never change meaning.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub labels: BTreeMap<String, serde_json::Value>,
}

/// Run totals (doc 09) — populated by the runtime (PRP-004).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Totals {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bridge_calls: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cost_usd: Option<f64>,
}
