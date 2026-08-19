//! `OpenRouterBridge` (`bridges-openrouter` feature): an OpenAI-compatible
//! chat-completions client against `https://openrouter.ai/api/v1`, built on
//! the `litellm-rust` SDK crate (`avivsinai/litellm-rust`, exact-pinned —
//! see the dependency comment in the workspace `Cargo.toml`).
//!
//! ## Containment
//!
//! `litellm-rust` and `reqwest` live **only** behind this module's
//! `bridges-openrouter` feature, in `horismos-bridges`. `horismos-core`
//! never sees either — `make check-boundary` enforces this mechanically.
//!
//! ## Why this module bypasses `litellm_rust::LiteLLM::completion()`
//!
//! U13's containment note requires verifying, against the crate's own
//! types/source (0.3.0), whether its response adapter surfaces
//! `model`/`finish_reason`/OpenRouter routing metadata — traces need these
//! as audit fields (`Bridge::CallAccounting`). Empirically, it does not:
//!
//! - `litellm_rust::providers::openai_compat::chat()` — the function
//!   `LiteLLM::completion()` dispatches every `ProviderKind::OpenAICompatible`
//!   call through (OpenRouter is registered as this kind, `router.rs`'s
//!   `default_provider_config("openrouter")`) — deserializes the upstream
//!   JSON into a private `OpenAIChatResponse` struct that has no `model`
//!   field and no `finish_reason` field, then always constructs the
//!   returned `ChatResponse` with `raw: None` (`providers/openai_compat.rs`,
//!   the `chat()` function's final `Ok(ChatResponse { .. raw: None })`).
//!   `ChatResponse::raw` exists as a field but the OpenAI-compatible path
//!   never populates it, so it is not the "raw response access" escape
//!   hatch it looks like from the type alone.
//! - The crate does not surface HTTP response headers to `LiteLLM::completion()`
//!   callers either (only one derived value, `x-litellm-response-cost`, makes
//!   it into `ChatResponse::header_cost`), so an OpenRouter routing header
//!   (e.g. an echoed `X-OpenRouter-Metadata` acknowledgement) is unreachable
//!   through the high-level API too.
//!
//! What the crate *does* expose, and what this module uses instead: forced
//! `tool_choice` is expressible (`types::ChatRequest::tool_choice: Option<Value>`,
//! forwarded verbatim into the request body by
//! `providers::openai_compat::build_chat_body`) — confirmed empirically, not
//! assumed, satisfying the other half of U13's containment note — and the
//! transport primitive underneath every provider call,
//! `litellm_rust::http::send_json_once`, is `pub` and returns
//! `(T, reqwest::HeaderMap)` for any `T: DeserializeOwned`, headers included.
//! This module calls that primitive directly with `T = serde_json::Value`,
//! recovering `model`/`finish_reason`/OpenRouter's `provider` field from the
//! same response the high-level API would have silently narrowed — the
//! "thin supplement" the containment note pre-authorized, kept inside
//! `litellm-rust`'s own transport rather than a second raw-`reqwest` call
//! (which would cost a duplicate, possibly-different network round trip).
//! `litellm_rust::config::ProviderConfig` and
//! `litellm_rust::providers::resolve_api_key` are used as designed (env-var
//! / direct-key / no-auth precedence, extra headers).
//!
//! Request bodies are built by hand rather than via
//! `serde_json::to_value(ChatRequest)`: `ChatRequest`'s `Option<T>` fields
//! have no `skip_serializing_if`, so a naive `Serialize` would send
//! `"temperature": null, ...` for every unset field — `build_chat_body`
//! avoids this by conditional insertion, but it is a private function. This
//! module only ever sets `model`/`messages`/`tools`/`tool_choice`, so
//! replicating that conditional-insertion shape for those four fields is a
//! few lines, not a fork.
//!
//! ## `OPENROUTER_API_KEY`
//!
//! Read from the `OPENROUTER_API_KEY` environment variable, or — when unset
//! — from a local file named by `OPENROUTER_API_KEY_FILE` (a gitignored
//! path the deployer chooses). Never logged, never committed, never placed
//! in a trace or `CallAccounting::raw_metadata`.
//!
//! ## D4 (forced `tool_choice`, live verification)
//!
//! `OPENROUTER_API_KEY` was not present in this environment when this
//! module was written (checked — no env var, no `OPENROUTER_API_KEY_FILE`).
//! The request-shape tests below verify the forced-`tool_choice` body
//! against a local mock listener (no network, no key); `live_perceive_smoke`
//! is `#[ignore]`d and only runs with a real key, per D4's existing gap
//! note (`PRPs/PRP-004-agent-loop/02-tasks-and-gaps.md`).
//!
//! Spec: `docs/framework/08-bridge.md` · Owner: PRP-004

use std::time::Instant;

use horismos_core::bridge::{
    Bridge, BridgeError, CallAccounting, NarrateRequest, NarrateResult, PerceiveRequest,
    PerceiveResult, RankRequest, RankResult,
};
use litellm_rust::config::ProviderConfig;
use litellm_rust::error::LiteLLMError;
use litellm_rust::providers::resolve_api_key;
use reqwest::header::HeaderMap;
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";
/// An inexpensive default (spec: "model from `OPENROUTER_MODEL` env
/// (default an inexpensive model)"); any OpenRouter-routed model id works.
const DEFAULT_MODEL: &str = "openai/gpt-4o-mini";
const ASSERT_FACTS_TOOL: &str = "assert_facts";
const SELECT_CANDIDATE_TOOL: &str = "select_candidate";

/// An OpenAI-compatible chat-completions `Bridge` against OpenRouter, built
/// on `litellm-rust`. See the module doc for the containment/verification
/// story behind its design.
pub struct OpenRouterBridge {
    id: String,
    model: String,
    base_url: String,
    provider_config: ProviderConfig,
    client: reqwest::Client,
}

impl OpenRouterBridge {
    /// Builds a bridge from environment configuration:
    /// `OPENROUTER_MODEL` (default [`DEFAULT_MODEL`]) and
    /// `OPENROUTER_API_KEY` / `OPENROUTER_API_KEY_FILE` (checked lazily, at
    /// call time — construction never fails for a missing key, matching
    /// `litellm-rust`'s own `resolve_api_key` posture).
    pub fn from_env() -> Self {
        let model = std::env::var("OPENROUTER_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        Self::new(model)
    }

    /// Builds a bridge for a specific OpenRouter-routed model id (e.g.
    /// `"anthropic/claude-3.5-haiku"`).
    pub fn new(model: impl Into<String>) -> Self {
        let model = model.into();
        let id = format!("openrouter:{model}");
        let provider_config = ProviderConfig::default()
            .with_base_url(DEFAULT_BASE_URL)
            .with_api_key_env("OPENROUTER_API_KEY")
            // OpenRouter-specific: ask for routing/provider metadata on the
            // response (D9/U13 containment note); not live-verified — see
            // the module doc and D4's gap note.
            .with_header("X-OpenRouter-Metadata", "enabled");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client construction with only a timeout cannot fail");
        Self {
            id,
            model,
            base_url: DEFAULT_BASE_URL.to_string(),
            provider_config,
            client,
        }
    }

    /// Resolves the API key: `OPENROUTER_API_KEY` first, then a local file
    /// named by `OPENROUTER_API_KEY_FILE` (both gitignored paths, per
    /// `docs/framework/08-bridge.md`'s "never logged, never committed").
    fn resolve_key(&self) -> Result<Option<String>, BridgeError> {
        match resolve_api_key(&self.provider_config) {
            Ok(key) => Ok(key),
            Err(LiteLLMError::MissingApiKey(_)) => {
                let Ok(path) = std::env::var("OPENROUTER_API_KEY_FILE") else {
                    return Err(BridgeError::RequestFailed {
                        bridge: self.id.clone(),
                        reason: "OPENROUTER_API_KEY is unset and OPENROUTER_API_KEY_FILE names no fallback".to_string(),
                    });
                };
                std::fs::read_to_string(&path)
                    .map(|s| Some(s.trim().to_string()))
                    .map_err(|e| BridgeError::RequestFailed {
                        bridge: self.id.clone(),
                        reason: format!("OPENROUTER_API_KEY_FILE {path:?}: {e}"),
                    })
            }
            Err(e) => Err(self.map_litellm_error(e)),
        }
    }

    fn map_litellm_error(&self, err: LiteLLMError) -> BridgeError {
        match &err {
            LiteLLMError::Http { message, .. } if message.contains("timed out") => {
                BridgeError::Timeout {
                    bridge: self.id.clone(),
                }
            }
            LiteLLMError::Parse(_) => BridgeError::MalformedOutput {
                bridge: self.id.clone(),
                reason: err.to_string(),
            },
            LiteLLMError::Refusal { text, .. } => BridgeError::Refused {
                bridge: self.id.clone(),
                reason: text.clone(),
            },
            LiteLLMError::Truncated { text, .. } => BridgeError::MalformedOutput {
                bridge: self.id.clone(),
                reason: format!("response truncated: {text}"),
            },
            _ => BridgeError::RequestFailed {
                bridge: self.id.clone(),
                reason: err.to_string(),
            },
        }
    }

    /// Sends one chat-completions request (built by the caller — `model`
    /// is filled in here) and returns the raw response JSON plus headers.
    /// Uses `litellm_rust::http::send_json_once` as the transport (see the
    /// module doc for why this bypasses `LiteLLM::completion()`).
    async fn call(&self, mut body: Value) -> Result<(Value, HeaderMap), BridgeError> {
        body["model"] = Value::String(self.model.clone());
        let key = self.resolve_key()?;
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut builder = self.client.post(url).json(&body);
        if let Some(key) = key {
            builder = builder.bearer_auth(key);
        }
        for (k, v) in &self.provider_config.extra_headers {
            builder = builder.header(k, v);
        }
        litellm_rust::http::send_json_once::<Value>(builder)
            .await
            .map_err(|e| self.map_litellm_error(e))
    }

    /// Extracts the forced tool call's parsed `arguments` object, the
    /// response's `finish_reason`, and the accounting this response
    /// carries (doc 08 § Operational semantics; see the module doc for the
    /// audit-field recovery this performs).
    fn extract_tool_call(&self, raw: &Value, tool_name: &str) -> Result<Value, BridgeError> {
        let malformed = |reason: String| BridgeError::MalformedOutput {
            bridge: self.id.clone(),
            reason,
        };
        let choice = raw
            .get("choices")
            .and_then(|c| c.get(0))
            .ok_or_else(|| malformed("response has no choices".to_string()))?;
        let message = choice
            .get("message")
            .ok_or_else(|| malformed("choice has no message".to_string()))?;
        let tool_calls = message
            .get("tool_calls")
            .and_then(Value::as_array)
            .filter(|calls| !calls.is_empty())
            .ok_or_else(|| {
                malformed(format!(
                    "model did not call the forced tool {tool_name:?} (no tool_calls)"
                ))
            })?;
        let call = &tool_calls[0];
        let name = call.pointer("/function/name").and_then(Value::as_str);
        if name != Some(tool_name) {
            return Err(malformed(format!(
                "forced tool_choice named {tool_name:?} but the model called {name:?}"
            )));
        }
        let args_str = call
            .pointer("/function/arguments")
            .and_then(Value::as_str)
            .ok_or_else(|| malformed("tool call has no string `function.arguments`".to_string()))?;
        serde_json::from_str(args_str)
            .map_err(|e| malformed(format!("tool call arguments are not valid JSON: {e}")))
    }

    fn accounting(&self, raw: &Value, elapsed: std::time::Duration) -> CallAccounting {
        let usage = raw.get("usage");
        let prompt_tokens = usage
            .and_then(|u| u.get("prompt_tokens"))
            .and_then(Value::as_u64);
        let completion_tokens = usage
            .and_then(|u| u.get("completion_tokens"))
            .and_then(Value::as_u64);
        let cost_usd = usage.and_then(|u| u.get("cost")).and_then(Value::as_f64);
        let model = raw.get("model").and_then(Value::as_str).map(String::from);
        let finish_reason = raw
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("finish_reason"))
            .and_then(Value::as_str)
            .map(String::from);
        // OpenRouter-specific routing metadata not yet promoted to a
        // dedicated field (D9/U13 containment note) — carried through raw.
        let mut extra = serde_json::Map::new();
        if let Some(provider) = raw.get("provider") {
            extra.insert("provider".to_string(), provider.clone());
        }
        if let Some(id) = raw.get("id") {
            extra.insert("id".to_string(), id.clone());
        }
        let raw_metadata = (!extra.is_empty()).then_some(Value::Object(extra));
        CallAccounting {
            latency_ms: Some(elapsed.as_millis() as u64),
            prompt_tokens,
            completion_tokens,
            cost_usd,
            model,
            finish_reason,
            raw_metadata,
        }
    }
}

#[async_trait::async_trait]
impl Bridge for OpenRouterBridge {
    async fn perceive(&self, req: PerceiveRequest) -> Result<PerceiveResult, BridgeError> {
        let tool = serde_json::json!({
            "type": "function",
            "function": {
                "name": ASSERT_FACTS_TOOL,
                "description": "Extract typed instances from the input text, per the supplied schema.",
                "parameters": req.schema,
            }
        });
        let tool_choice =
            serde_json::json!({"type": "function", "function": {"name": ASSERT_FACTS_TOOL}});
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": req.text}],
            "tools": [tool],
            "tool_choice": tool_choice,
        });
        let started = Instant::now();
        let (raw, _headers) = self.call(body).await?;
        let accounting = self.accounting(&raw, started.elapsed());
        let args = self.extract_tool_call(&raw, ASSERT_FACTS_TOOL)?;
        let instances = args
            .get("instances")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        // D9: a self-reported confidence the wire omitted normalizes to
        // 0.0 — fails safe into the runtime's fallback edge.
        let confidence = args
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        Ok(PerceiveResult {
            instances,
            confidence,
            fact_confidence: Default::default(),
            accounting,
        })
    }

    async fn narrate(&self, req: NarrateRequest) -> Result<NarrateResult, BridgeError> {
        let body = serde_json::json!({
            "messages": [
                {
                    "role": "system",
                    "content": format!(
                        "Render the following proof for the audience \"{}\". Prose only, no new claims.",
                        req.audience
                    ),
                },
                {"role": "user", "content": req.proof_text},
            ],
        });
        let started = Instant::now();
        let (raw, _headers) = self.call(body).await?;
        let accounting = self.accounting(&raw, started.elapsed());
        let text = raw
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .ok_or_else(|| BridgeError::MalformedOutput {
                bridge: self.id.clone(),
                reason: "response has no choices/0/message/content".to_string(),
            })?
            .to_string();
        Ok(NarrateResult { text, accounting })
    }

    async fn rank(&self, req: RankRequest) -> Result<RankResult, BridgeError> {
        if req.candidates.is_empty() {
            return Err(BridgeError::MalformedOutput {
                bridge: self.id.clone(),
                reason: "rank called with no candidates".to_string(),
            });
        }
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "chosen": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": req.candidates.len() - 1,
                    "description": "Index into the candidates list.",
                },
                "confidence": {"type": "number", "minimum": 0, "maximum": 1},
            },
            "required": ["chosen", "confidence"],
        });
        let tool = serde_json::json!({
            "type": "function",
            "function": {
                "name": SELECT_CANDIDATE_TOOL,
                "description": "Select the best candidate for the stated ranking context.",
                "parameters": schema,
            }
        });
        let tool_choice =
            serde_json::json!({"type": "function", "function": {"name": SELECT_CANDIDATE_TOOL}});
        let candidates_json =
            serde_json::to_string(&req.candidates).map_err(|e| BridgeError::MalformedOutput {
                bridge: self.id.clone(),
                reason: format!("candidates did not serialize to JSON: {e}"),
            })?;
        let body = serde_json::json!({
            "messages": [{
                "role": "user",
                "content": format!("{}\n\nCandidates (choose by index):\n{}", req.context, candidates_json),
            }],
            "tools": [tool],
            "tool_choice": tool_choice,
        });
        let started = Instant::now();
        let (raw, _headers) = self.call(body).await?;
        let accounting = self.accounting(&raw, started.elapsed());
        let args = self.extract_tool_call(&raw, SELECT_CANDIDATE_TOOL)?;
        let chosen = args.get("chosen").and_then(Value::as_u64).ok_or_else(|| {
            BridgeError::MalformedOutput {
                bridge: self.id.clone(),
                reason: "tool call arguments have no integer `chosen`".to_string(),
            }
        })? as usize;
        if chosen >= req.candidates.len() {
            return Err(BridgeError::MalformedOutput {
                bridge: self.id.clone(),
                reason: format!(
                    "chosen index {chosen} is out of range for {} candidates",
                    req.candidates.len()
                ),
            });
        }
        let confidence = args
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        Ok(RankResult {
            chosen,
            confidence,
            accounting,
        })
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Starts a one-shot local HTTP server: reads one request, hands its
    /// raw bytes to `assert_request`, writes back `response_body` as a
    /// `200 application/json` response regardless of what `assert_request`
    /// finds (so a failed assertion reports cleanly on the test's own
    /// thread instead of a downstream connection-reset panic in the
    /// spawned task). No network, no key — mirrors `litellm-rust`'s own
    /// `http.rs` test pattern.
    async fn one_shot_server(
        response_body: &'static str,
        assert_request: impl FnOnce(String) + Send + 'static,
    ) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binding a local test listener should not fail");
        let addr = listener
            .local_addr()
            .expect("a bound listener should have a local address");
        tokio::spawn(async move {
            let (mut stream, _) = listener
                .accept()
                .await
                .expect("accepting the one expected connection should not fail");
            let mut buf = vec![0_u8; 65536];
            let n = stream
                .read(&mut buf)
                .await
                .expect("reading the request should not fail");
            let request_text = String::from_utf8_lossy(&buf[..n]).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("writing the scripted response should not fail");
            stream
                .flush()
                .await
                .expect("flushing the scripted response should not fail");
            assert_request(request_text);
        });
        format!("http://{addr}")
    }

    /// Parses the JSON body out of a raw HTTP request's text (the part
    /// after the blank line separating headers from body).
    fn request_body(request_text: &str) -> Value {
        let body = request_text.split("\r\n\r\n").nth(1).unwrap_or_default();
        serde_json::from_str(body).expect("request body should be valid JSON")
    }

    fn bridge_at(base_url: String) -> OpenRouterBridge {
        // no_auth: the mock listener needs no bearer token; a real
        // OPENROUTER_API_KEY (or OPENROUTER_API_KEY_FILE) is only resolved
        // for a real OpenRouter call.
        let mut bridge = OpenRouterBridge::new("openai/gpt-4o-mini");
        bridge.base_url = base_url;
        bridge.provider_config = bridge.provider_config.clone().with_no_auth(true);
        bridge
    }

    const SCRIPTED_ASSERT_FACTS_RESPONSE: &str = r#"{
        "id": "gen-abc123",
        "model": "openai/gpt-4o-mini-2024-07-18",
        "provider": "OpenAI",
        "choices": [{
            "finish_reason": "tool_calls",
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {
                        "name": "assert_facts",
                        "arguments": "{\"instances\":[{\"heartRate\":128}],\"confidence\":0.94}"
                    }
                }]
            }
        }],
        "usage": {"prompt_tokens": 42, "completion_tokens": 11, "cost": 0.0003}
    }"#;

    #[tokio::test]
    async fn perceive_forces_assert_facts_tool_choice_and_parses_result() {
        let base_url = one_shot_server(SCRIPTED_ASSERT_FACTS_RESPONSE, |request_text| {
            let body = request_body(&request_text);
            assert_eq!(
                body.get("tool_choice"),
                Some(
                    &serde_json::json!({"type": "function", "function": {"name": "assert_facts"}})
                ),
                "request body did not force assert_facts tool_choice:\n{body}"
            );
            assert_eq!(
                body.pointer("/tools/0/function/name"),
                Some(&Value::String("assert_facts".to_string())),
                "request body did not declare the assert_facts tool:\n{body}"
            );
            assert!(
                !request_text.contains("\"temperature\":null"),
                "request body leaked a null optional field:\n{request_text}"
            );
        })
        .await;

        let bridge = bridge_at(base_url);
        let result = bridge
            .perceive(PerceiveRequest {
                text: "58M, crushing chest pain, heart rate 128".to_string(),
                schema: serde_json::json!({"type": "object", "properties": {
                    "instances": {"type": "array"},
                    "confidence": {"type": "number"},
                }}),
            })
            .await
            .expect("perceive should succeed against the scripted response");

        assert_eq!(
            result.instances,
            vec![serde_json::json!({"heartRate": 128})]
        );
        assert_eq!(result.confidence, 0.94);

        // The audit-metadata recovery this module exists for (module doc):
        // model/finish_reason/OpenRouter `provider` all survive, though
        // `LiteLLM::completion()` would have dropped them.
        assert_eq!(
            result.accounting.model.as_deref(),
            Some("openai/gpt-4o-mini-2024-07-18")
        );
        assert_eq!(
            result.accounting.finish_reason.as_deref(),
            Some("tool_calls")
        );
        assert_eq!(result.accounting.prompt_tokens, Some(42));
        assert_eq!(result.accounting.completion_tokens, Some(11));
        assert_eq!(result.accounting.cost_usd, Some(0.0003));
        assert_eq!(
            result
                .accounting
                .raw_metadata
                .as_ref()
                .and_then(|m| m.get("provider"))
                .and_then(Value::as_str),
            Some("OpenAI")
        );
    }

    #[tokio::test]
    async fn perceive_missing_confidence_normalizes_to_zero() {
        let response = r#"{
            "model": "openai/gpt-4o-mini",
            "choices": [{
                "finish_reason": "tool_calls",
                "message": {"tool_calls": [{
                    "function": {"name": "assert_facts", "arguments": "{\"instances\":[]}"}
                }]}
            }]
        }"#;
        let base_url = one_shot_server(response, |_| {}).await;
        let bridge = bridge_at(base_url);
        let result = bridge
            .perceive(PerceiveRequest {
                text: "ambiguous input".to_string(),
                schema: serde_json::json!({"type": "object"}),
            })
            .await
            .expect("perceive should succeed even with confidence omitted");
        assert_eq!(result.confidence, 0.0);
    }

    #[tokio::test]
    async fn perceive_rejects_a_reply_that_never_calls_the_forced_tool() {
        let response = r#"{
            "model": "openai/gpt-4o-mini",
            "choices": [{"finish_reason": "stop", "message": {"content": "no tool call here"}}]
        }"#;
        let base_url = one_shot_server(response, |_| {}).await;
        let bridge = bridge_at(base_url);
        let err = bridge
            .perceive(PerceiveRequest {
                text: "text".to_string(),
                schema: serde_json::json!({"type": "object"}),
            })
            .await
            .expect_err("a reply with no tool call must be a malformed-output error, not a panic");
        assert!(matches!(err, BridgeError::MalformedOutput { .. }));
    }

    #[tokio::test]
    async fn narrate_extracts_message_content() {
        let response = r#"{
            "model": "openai/gpt-4o-mini",
            "choices": [{"finish_reason": "stop", "message": {"content": "Acuity is Immediate."}}]
        }"#;
        let base_url = one_shot_server(response, |request_text| {
            assert!(
                request_text.contains("charge-nurse"),
                "request body did not carry the audience:\n{request_text}"
            );
        })
        .await;
        let bridge = bridge_at(base_url);
        let result = bridge
            .narrate(NarrateRequest {
                proof_text: "acuity := Immediate, via :r1".to_string(),
                audience: "charge-nurse".to_string(),
            })
            .await
            .expect("narrate should succeed");
        assert_eq!(result.text, "Acuity is Immediate.");
    }

    #[tokio::test]
    async fn rank_forces_select_candidate_tool_choice() {
        let response = r#"{
            "model": "openai/gpt-4o-mini",
            "choices": [{
                "finish_reason": "tool_calls",
                "message": {"tool_calls": [{
                    "function": {"name": "select_candidate", "arguments": "{\"chosen\":1,\"confidence\":0.8}"}
                }]}
            }]
        }"#;
        let base_url = one_shot_server(response, |request_text| {
            let body = request_body(&request_text);
            assert_eq!(
                body.get("tool_choice"),
                Some(&serde_json::json!({"type": "function", "function": {"name": "select_candidate"}})),
                "request body did not force select_candidate tool_choice:\n{body}"
            );
        })
        .await;
        let bridge = bridge_at(base_url);
        let result = bridge
            .rank(RankRequest {
                context: "customer preference".to_string(),
                candidates: vec![
                    serde_json::json!({"option": "A"}),
                    serde_json::json!({"option": "B"}),
                ],
            })
            .await
            .expect("rank should succeed");
        assert_eq!(result.chosen, 1);
        assert_eq!(result.confidence, 0.8);
    }

    #[test]
    fn rank_rejects_empty_candidates_without_a_call() {
        let bridge = OpenRouterBridge::new("openai/gpt-4o-mini");
        let fut = bridge.rank(RankRequest {
            context: "x".to_string(),
            candidates: vec![],
        });
        // Poll once: with no candidates the error path returns before any
        // await point, so a single poll is sufficient without a runtime.
        use std::future::Future;
        use std::pin::pin;
        use std::task::{Context, Poll, Waker};
        let mut cx = Context::from_waker(Waker::noop());
        match pin!(fut).poll(&mut cx) {
            Poll::Ready(Err(BridgeError::MalformedOutput { .. })) => {}
            other => panic!("expected an immediate MalformedOutput error, got {other:?}"),
        }
    }

    #[test]
    fn id_reports_the_openrouter_prefixed_model() {
        let bridge = OpenRouterBridge::new("anthropic/claude-3.5-haiku");
        assert_eq!(bridge.id(), "openrouter:anthropic/claude-3.5-haiku");
    }

    /// D4's live-verification gap: run by hand with a real key —
    /// `OPENROUTER_API_KEY=sk-... cargo test -p horismos-bridges --features
    /// bridges-openrouter -- --ignored live_perceive_smoke`.
    #[tokio::test]
    #[ignore = "needs a live OPENROUTER_API_KEY; D4 gap, not run in CI"]
    async fn live_perceive_smoke() {
        let bridge = OpenRouterBridge::from_env();
        let result = bridge
            .perceive(PerceiveRequest {
                text: "58M, crushing chest pain, heart rate 128".to_string(),
                schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "instances": {"type": "array"},
                        "confidence": {"type": "number"},
                    },
                    "required": ["instances", "confidence"],
                }),
            })
            .await
            .expect("live perceive call should succeed with a real key");
        assert!(
            result.accounting.model.is_some(),
            "expected the live response to report a model"
        );
    }
}
