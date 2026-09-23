//! The Bridge contract: a model client behind a uniform interface.
//!
//! This module holds the **pure trait only** (perceive / narrate / rank,
//! confidence and timeout semantics). Implementations live in `horismos-bridges`
//! — `horismos-core` must never depend on a provider SDK. This inversion is the
//! enforced anti-wrapper boundary.
//!
//! `Bridge` is a client, not another orchestrator (doc 08): three methods,
//! nothing else. Thresholds, retries, fallbacks, and timeout enforcement are
//! the runtime's job (`horismos-core::workflow`), never a `Bridge`
//! implementation's.
//!
//! Spec: `docs/framework/08-bridge.md` · Owner: PRP-004

use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A model client: `perceive`/`narrate`/`rank`, and nothing else (doc 08).
/// The runtime is the loop; implementations of this trait are dumb clients.
#[async_trait]
pub trait Bridge: Send + Sync {
    /// One `PerceptionStep` call (doc 06): input text in, a result forced
    /// into the caller-supplied schema out.
    async fn perceive(&self, req: PerceiveRequest) -> Result<PerceiveResult, BridgeError>;

    /// One `NarrationStep` call (doc 06): a rendered proof tree in, prose
    /// out. Never asserted into the World.
    async fn narrate(&self, req: NarrateRequest) -> Result<NarrateResult, BridgeError>;

    /// One `ProposalStep` call (doc 07): the model ranks precondition-legal
    /// candidates the runtime already computed; it cannot add options.
    async fn rank(&self, req: RankRequest) -> Result<RankResult, BridgeError>;

    /// The identifier recorded in the trace's `model` field (D8) — the
    /// model **actually invoked**, which may differ from the ontology's
    /// declared `horis:model` string when deployment overrides it.
    fn id(&self) -> &str;
}

/// A shared handle to a bridge is a bridge. `AgentBuilder::bridge` takes
/// ownership of what it is given, so a deployment that must keep its own
/// reference — a binding layer reading a mock's recorded invocations after a
/// run, or one configured client serving two prefixes — registers an `Arc`.
/// Pure delegation: the runtime still owns every threshold and fallback.
#[async_trait]
impl<B: Bridge + ?Sized> Bridge for std::sync::Arc<B> {
    async fn perceive(&self, req: PerceiveRequest) -> Result<PerceiveResult, BridgeError> {
        (**self).perceive(req).await
    }

    async fn narrate(&self, req: NarrateRequest) -> Result<NarrateResult, BridgeError> {
        (**self).narrate(req).await
    }

    async fn rank(&self, req: RankRequest) -> Result<RankResult, BridgeError> {
        (**self).rank(req).await
    }

    fn id(&self) -> &str {
        (**self).id()
    }
}

/// One `PerceptionStep` call: the input text plus the JSON Schema derived
/// from `horis:extracts` (doc 06's mapping table). The schema is the
/// provider's typed-output channel, used backwards (doc 08) — not to let
/// the model choose actions, but to force conformance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceiveRequest {
    /// The natural-language input to extract instances from.
    pub text: String,
    /// The JSON Schema derived from the workflow step's `horis:extracts`
    /// classes; the model's output must conform to it.
    pub schema: serde_json::Value,
}

/// One proposed instance: shape matches the derived JSON Schema's `object`.
/// Never trusted blindly — every instance still passes the World gate
/// (doc 06 step 5) before assertion.
pub type ProposedInstance = serde_json::Value;

/// The result of one `perceive` call.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerceiveResult {
    /// Proposed instances, one JSON object per extracted instance.
    pub instances: Vec<ProposedInstance>,
    /// Call confidence (D9): the `assert_facts` schema's self-reported
    /// `confidence` field, normalized by the `Bridge` implementation. A
    /// self-report the wire omitted is normalized to `0.0` here — fails
    /// safe into the runtime's fallback edge, never a false accept.
    pub confidence: f64,
    /// Per-fact confidence, keyed by index into `instances`, when the
    /// provider supports it (doc 06 § Confidence). An index absent from
    /// this map inherits `confidence`.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub fact_confidence: BTreeMap<usize, f64>,
    /// Per-call accounting (latency, tokens, cost, audit metadata).
    #[serde(default)]
    pub accounting: CallAccounting,
}

/// One `NarrationStep` call: the rendered proof tree (doc 04), never the
/// raw world or the original input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrateRequest {
    /// The proof tree, already rendered to text by `proof.render()`.
    pub proof_text: String,
    /// The declared `horis:audience` (e.g. `"charge-nurse"`).
    pub audience: String,
}

/// The result of one `narrate` call.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NarrateResult {
    /// The rendered prose. Attached to the trace as `narration`; never
    /// asserted into the World (doc 06).
    pub text: String,
    /// Per-call accounting (latency, tokens, cost, audit metadata).
    #[serde(default)]
    pub accounting: CallAccounting,
}

/// One `ProposalStep` call: candidates come from a graph query (doc 07) —
/// every candidate already satisfies its preconditions. The model ranks;
/// it cannot add options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankRequest {
    /// The ranking context (e.g. `horis:rankBy`'s stated criterion, plus
    /// the original input where relevant).
    pub context: String,
    /// Precondition-legal candidates, one JSON object per candidate.
    pub candidates: Vec<serde_json::Value>,
}

/// The result of one `rank` call.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RankResult {
    /// Index into `RankRequest::candidates` the model selected.
    pub chosen: usize,
    /// Call confidence, same semantics as `PerceiveResult::confidence`.
    pub confidence: f64,
    /// Per-call accounting (latency, tokens, cost, audit metadata).
    #[serde(default)]
    pub accounting: CallAccounting,
}

/// Per-call accounting the runtime records into the trace (doc 08 §
/// Operational semantics: "every call records model, latency, token
/// counts, cost estimate"). `model`/`finish_reason`/`raw_metadata` are an
/// audit escape hatch for provider-specific fields (e.g. an OpenRouter
/// deployment's actually-routed upstream model) that do not yet have a
/// dedicated trace field — carried through so trace emission can decide
/// what to surface. `Bridge::id()` remains the trace's authoritative
/// `model` field (D8); `model` here is a supplementary audit value.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CallAccounting {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prompt_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub completion_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cost_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub finish_reason: Option<String>,
    /// Provider-specific fields not yet promoted to a dedicated field
    /// (e.g. OpenRouter's `provider` routing metadata).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub raw_metadata: Option<serde_json::Value>,
}

/// Errors from a `Bridge` call (doc 08 § Operational semantics: "Fallback
/// on any Bridge error ... identical to low confidence"). The runtime
/// treats every variant the same way — the step's `horis:fallback` edge —
/// so these exist for trace/log fidelity, not differentiated handling.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BridgeError {
    /// The call did not complete within its enforced timeout.
    #[error("bridge {bridge} timed out")]
    Timeout { bridge: String },
    /// The provider declined to answer (safety refusal, content policy).
    #[error("bridge {bridge} refused: {reason}")]
    Refused { bridge: String, reason: String },
    /// The provider's output could not be interpreted against the
    /// requested contract (schema, tool-call shape).
    #[error("bridge {bridge} returned malformed output: {reason}")]
    MalformedOutput { bridge: String, reason: String },
    /// The call itself failed (network, auth, rate limit, non-2xx).
    #[error("bridge {bridge} request failed: {reason}")]
    RequestFailed { bridge: String, reason: String },
}

impl BridgeError {
    /// The failing condition, stated in domain vocabulary.
    pub fn domain_text(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    /// Drives a `Future` to completion without an async runtime — the
    /// bridge trait's request/result types and a trivial implementor must
    /// compile and run under `--no-default-features --features verify`
    /// (no-model-gate), which has no async executor available.
    fn block_on<F: Future>(fut: F) -> F::Output {
        let mut cx = Context::from_waker(Waker::noop());
        let mut fut = pin!(fut);
        loop {
            if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
                return out;
            }
        }
    }

    /// A trivial in-crate `Bridge` test double — no network, no keys.
    struct TestDouble;

    #[async_trait]
    impl Bridge for TestDouble {
        async fn perceive(&self, req: PerceiveRequest) -> Result<PerceiveResult, BridgeError> {
            Ok(PerceiveResult {
                instances: vec![serde_json::json!({"echo": req.text})],
                confidence: 0.99,
                fact_confidence: BTreeMap::new(),
                accounting: CallAccounting::default(),
            })
        }

        async fn narrate(&self, req: NarrateRequest) -> Result<NarrateResult, BridgeError> {
            Ok(NarrateResult {
                text: format!("narrated: {}", req.proof_text),
                accounting: CallAccounting::default(),
            })
        }

        async fn rank(&self, req: RankRequest) -> Result<RankResult, BridgeError> {
            if req.candidates.is_empty() {
                return Err(BridgeError::MalformedOutput {
                    bridge: self.id().to_string(),
                    reason: "no candidates".to_string(),
                });
            }
            Ok(RankResult {
                chosen: 0,
                confidence: 0.9,
                accounting: CallAccounting::default(),
            })
        }

        fn id(&self) -> &str {
            "test:double"
        }
    }

    #[test]
    fn test_double_perceive_round_trips() {
        let bridge: Box<dyn Bridge> = Box::new(TestDouble);
        let result = block_on(bridge.perceive(PerceiveRequest {
            text: "58M, crushing chest pain".to_string(),
            schema: serde_json::json!({"type": "object"}),
        }))
        .expect("perceive should succeed");
        assert_eq!(result.instances.len(), 1);
        assert_eq!(result.confidence, 0.99);
    }

    #[test]
    fn test_double_narrate_round_trips() {
        let bridge: Box<dyn Bridge> = Box::new(TestDouble);
        let result = block_on(bridge.narrate(NarrateRequest {
            proof_text: "acuity := Immediate".to_string(),
            audience: "charge-nurse".to_string(),
        }))
        .expect("narrate should succeed");
        assert_eq!(result.text, "narrated: acuity := Immediate");
    }

    #[test]
    fn test_double_rank_rejects_empty_candidates() {
        let bridge: Box<dyn Bridge> = Box::new(TestDouble);
        let err = block_on(bridge.rank(RankRequest {
            context: "customer preference".to_string(),
            candidates: vec![],
        }))
        .expect_err("rank with no candidates should fail");
        assert_eq!(
            err,
            BridgeError::MalformedOutput {
                bridge: "test:double".to_string(),
                reason: "no candidates".to_string(),
            }
        );
    }

    #[test]
    fn a_shared_handle_delegates_to_the_bridge_it_shares() {
        let shared: std::sync::Arc<dyn Bridge> = std::sync::Arc::new(TestDouble);
        let registered: Box<dyn Bridge> = Box::new(std::sync::Arc::clone(&shared));
        let result = block_on(registered.perceive(PerceiveRequest {
            text: "58M".to_string(),
            schema: serde_json::json!({"type": "object"}),
        }))
        .expect("perceive should succeed");
        assert_eq!(result.instances, vec![serde_json::json!({"echo": "58M"})]);
        let narrated = block_on(registered.narrate(NarrateRequest {
            proof_text: "r1 fired".to_string(),
            audience: "charge-nurse".to_string(),
        }))
        .expect("narrate should succeed");
        assert_eq!(narrated.text, "narrated: r1 fired");
        let ranked = block_on(registered.rank(RankRequest {
            context: "acuity".to_string(),
            candidates: vec![serde_json::json!({"queue": "resus"})],
        }))
        .expect("rank should succeed");
        assert_eq!(ranked.chosen, 0);
        assert_eq!(registered.id(), shared.id());
    }

    #[test]
    fn bridge_id_is_the_trace_model_field() {
        let bridge: Box<dyn Bridge> = Box::new(TestDouble);
        assert_eq!(bridge.id(), "test:double");
    }

    #[test]
    fn bridge_error_domain_text_matches_display() {
        let err = BridgeError::Timeout {
            bridge: "test:double".to_string(),
        };
        assert_eq!(err.domain_text(), err.to_string());
    }
}
