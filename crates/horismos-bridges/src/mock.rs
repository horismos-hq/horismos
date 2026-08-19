//! MockBridge: scripted perceive/narrate/rank responses with configurable
//! confidence and latency — the backend every test and CI run uses.
//!
//! Doc 08's own illustration (Python, § The mock backend):
//!
//! ```python
//! bridge = MockBridge(script={
//!     "58M, crushing chest pain": PerceiveResult(instances=[...], confidence=0.94),
//!     "*": PerceiveResult.low_confidence(),
//! })
//! ```
//!
//! This module is the Rust shape of the same contract: scriptable by exact
//! input-pattern match, an optional `"*"` wildcard default, deterministic
//! (no randomness, no clock read), and every call recorded so a test can
//! assert what the runtime actually invoked. Crate-free and network-free —
//! `horismos-bridges` itself may depend on provider SDKs behind feature
//! flags, but `mock` never does, so it stays buildable and runnable
//! anywhere `horismos-core` is (the anti-wrapper guarantee's enforcement
//! mechanism, doc 14: a symbolic-core test that "needs" a real key is a
//! build failure by policy, and `mock:` is how every test avoids that).
//!
//! Unscripted input never panics or errors: it falls through to the
//! documented low-confidence default (D9 — a `PerceiveResult` with no
//! confidence fails safe into the runtime's fallback edge, never a false
//! accept). `NarrateResult`/`RankResult` have no comparable "confidence"
//! concept to fail safe on, so their defaults are simply their `Default`
//! impls, kept for API symmetry with `perceive`.
//!
//! Spec: `docs/framework/08-bridge.md` (§ mock backend) · Owner: PRP-004

use std::collections::BTreeMap;
use std::sync::Mutex;

use async_trait::async_trait;
use horismos_core::bridge::{
    Bridge, BridgeError, NarrateRequest, NarrateResult, PerceiveRequest, PerceiveResult,
    RankRequest, RankResult,
};

const WILDCARD: &str = "*";

/// One recorded call, in invocation order — what a test asserts against to
/// check the runtime called the bridge the way it meant to (doc 08: "records
/// invocations").
#[derive(Debug, Clone)]
pub enum Invocation {
    Perceive(PerceiveRequest),
    Narrate(NarrateRequest),
    Rank(RankRequest),
}

/// A scripted, deterministic `Bridge` — pattern (exact match on the call's
/// input text) → response, with an optional `"*"` wildcard default. Build
/// with [`MockBridge::new`] and the `script_*` methods; every call is
/// recorded and retrievable via [`MockBridge::invocations`].
pub struct MockBridge {
    id: String,
    perceive_script: BTreeMap<String, PerceiveResult>,
    narrate_script: BTreeMap<String, NarrateResult>,
    rank_script: BTreeMap<String, RankResult>,
    invocations: Mutex<Vec<Invocation>>,
}

impl MockBridge {
    /// A bridge with no scripted responses — every call falls through to
    /// its type's default (perceive: the documented low-confidence
    /// default).
    pub fn new() -> Self {
        Self::with_id("mock:default")
    }

    /// A bridge whose [`Bridge::id`] is `id` — for tests that pin the
    /// trace's `model` field (D8) to something other than the generic
    /// default.
    pub fn with_id(id: impl Into<String>) -> Self {
        MockBridge {
            id: id.into(),
            perceive_script: BTreeMap::new(),
            narrate_script: BTreeMap::new(),
            rank_script: BTreeMap::new(),
            invocations: Mutex::new(Vec::new()),
        }
    }

    /// Scripts `result` for a `perceive` call whose `PerceiveRequest::text`
    /// exactly equals `pattern` (or for any unscripted text, when
    /// `pattern` is `"*"`).
    pub fn script_perceive(mut self, pattern: impl Into<String>, result: PerceiveResult) -> Self {
        self.perceive_script.insert(pattern.into(), result);
        self
    }

    /// Scripts `result` for a `narrate` call whose `NarrateRequest::proof_text`
    /// exactly equals `pattern` (or `"*"` for any unscripted proof text).
    pub fn script_narrate(mut self, pattern: impl Into<String>, result: NarrateResult) -> Self {
        self.narrate_script.insert(pattern.into(), result);
        self
    }

    /// Scripts `result` for a `rank` call whose `RankRequest::context`
    /// exactly equals `pattern` (or `"*"` for any unscripted context).
    pub fn script_rank(mut self, pattern: impl Into<String>, result: RankResult) -> Self {
        self.rank_script.insert(pattern.into(), result);
        self
    }

    /// Every call this bridge has served, in invocation order.
    pub fn invocations(&self) -> Vec<Invocation> {
        self.invocations
            .lock()
            .expect("MockBridge invocation log is never held across a panic")
            .clone()
    }

    fn record(&self, invocation: Invocation) {
        self.invocations
            .lock()
            .expect("MockBridge invocation log is never held across a panic")
            .push(invocation);
    }
}

impl Default for MockBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Bridge for MockBridge {
    async fn perceive(&self, req: PerceiveRequest) -> Result<PerceiveResult, BridgeError> {
        let response = self
            .perceive_script
            .get(&req.text)
            .or_else(|| self.perceive_script.get(WILDCARD))
            .cloned()
            .unwrap_or_default();
        self.record(Invocation::Perceive(req));
        Ok(response)
    }

    async fn narrate(&self, req: NarrateRequest) -> Result<NarrateResult, BridgeError> {
        let response = self
            .narrate_script
            .get(&req.proof_text)
            .or_else(|| self.narrate_script.get(WILDCARD))
            .cloned()
            .unwrap_or_default();
        self.record(Invocation::Narrate(req));
        Ok(response)
    }

    async fn rank(&self, req: RankRequest) -> Result<RankResult, BridgeError> {
        let response = self
            .rank_script
            .get(&req.context)
            .or_else(|| self.rank_script.get(WILDCARD))
            .cloned()
            .unwrap_or_default();
        self.record(Invocation::Rank(req));
        Ok(response)
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap as Map;

    /// U6 test target (first half): scripted responses round-trip exactly.
    #[tokio::test]
    async fn a_scripted_perceive_response_round_trips_exactly() {
        let bridge = MockBridge::new().script_perceive(
            "58M, crushing chest pain",
            PerceiveResult {
                instances: vec![serde_json::json!({"heartRate": 128})],
                confidence: 0.94,
                fact_confidence: Map::new(),
                accounting: Default::default(),
            },
        );

        let result = bridge
            .perceive(PerceiveRequest {
                text: "58M, crushing chest pain".to_string(),
                schema: serde_json::json!({"type": "object"}),
            })
            .await
            .expect("scripted perceive must succeed");

        assert_eq!(result.confidence, 0.94);
        assert_eq!(
            result.instances,
            vec![serde_json::json!({"heartRate": 128})]
        );
    }

    /// U6 test target (second half): unscripted input yields the documented
    /// low-confidence default (D9 — omitted confidence fails safe to 0.0).
    #[tokio::test]
    async fn unscripted_perceive_input_yields_the_low_confidence_default() {
        let bridge = MockBridge::new().script_perceive(
            "58M, crushing chest pain",
            PerceiveResult {
                instances: vec![serde_json::json!({"heartRate": 128})],
                confidence: 0.94,
                fact_confidence: Map::new(),
                accounting: Default::default(),
            },
        );

        let result = bridge
            .perceive(PerceiveRequest {
                text: "an input nothing scripted".to_string(),
                schema: serde_json::json!({"type": "object"}),
            })
            .await
            .expect("an unscripted call still succeeds — it just gets the default");

        assert_eq!(result.confidence, 0.0);
        assert!(result.instances.is_empty());
    }

    #[tokio::test]
    async fn a_wildcard_script_overrides_the_built_in_default() {
        let bridge = MockBridge::new().script_perceive(
            "*",
            PerceiveResult {
                instances: vec![],
                confidence: 0.1,
                fact_confidence: Map::new(),
                accounting: Default::default(),
            },
        );

        let result = bridge
            .perceive(PerceiveRequest {
                text: "anything at all".to_string(),
                schema: serde_json::json!({}),
            })
            .await
            .expect("wildcard-scripted call succeeds");

        assert_eq!(result.confidence, 0.1);
    }

    #[tokio::test]
    async fn every_call_is_recorded_in_invocation_order() {
        let bridge = MockBridge::new();

        bridge
            .perceive(PerceiveRequest {
                text: "first".to_string(),
                schema: serde_json::json!({}),
            })
            .await
            .expect("unscripted perceive still succeeds");
        bridge
            .narrate(NarrateRequest {
                proof_text: "acuity := Immediate".to_string(),
                audience: "charge-nurse".to_string(),
            })
            .await
            .expect("unscripted narrate still succeeds");

        let log = bridge.invocations();
        assert_eq!(log.len(), 2);
        assert!(matches!(&log[0], Invocation::Perceive(r) if r.text == "first"));
        assert!(matches!(&log[1], Invocation::Narrate(r) if r.audience == "charge-nurse"));
    }

    #[tokio::test]
    async fn rank_falls_back_to_the_default_result_when_unscripted() {
        let bridge = MockBridge::new();
        let result = bridge
            .rank(RankRequest {
                context: "unscripted".to_string(),
                candidates: vec![serde_json::json!({"a": 1}), serde_json::json!({"b": 2})],
            })
            .await
            .expect("an unscripted rank call still succeeds");
        assert_eq!(result.chosen, 0);
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn id_defaults_to_mock_default_and_is_overridable() {
        assert_eq!(MockBridge::new().id(), "mock:default");
        assert_eq!(MockBridge::with_id("mock:triage").id(), "mock:triage");
    }
}
