//! `JevBridge` (`bridges-jev` feature): a **text → decisions** client for
//! TypeSafe's Jev, over either OpenRouter's Decisions endpoint or TypeSafe's
//! own System One endpoint.
//!
//! ## Why this is not `OpenRouterBridge` with a different model id
//!
//! [`crate::openrouter::OpenRouterBridge`] posts to `/chat/completions` and
//! reads text (a forced tool call whose `arguments` are a JSON string) back
//! out. Jev has no text output channel at all: it takes a free-text `state`
//! plus a set of typed *questions* and returns a typed *answer* per question
//! — a probability, an option key, or an ordinal level. Its
//! `supported_parameters` is empty. Different endpoint, different request
//! body, different response body; sharing a module would be sharing nothing
//! but the word "HTTP".
//!
//! ## The wire protocol
//!
//! Verified live on 2026-09-21 against both transports (see
//! [`JevTransport`]). Request:
//!
//! ```json
//! { "model": "...", "state": "<free text>",
//!   "questions": { "<id>": { "type": "noul|choice|score",
//!                            "instructions": "...", "criteria": { } } } }
//! ```
//!
//! Response:
//!
//! ```json
//! { "model": "...",
//!   "answers": { "<id>": { "type": "...", "noul": 0.96, "choice": "<key>",
//!                          "probabilities": {}, "confidence": 0.81,
//!                          "score": 1.05, "legend": {} } },
//!   "usage": { "input_tokens": 619, "output_tokens": 79, "cost": 1.6e-05 } }
//! ```
//!
//! On the OpenRouter transport a `noul` question's `criteria` must carry
//! **both** `"true"` and `"false"` keys; `build_question` always emits
//! both, so one code path serves both transports.
//!
//! ### What a `score` answer looks like — measured
//!
//! The populated `legend` shape was measured live on 2026-09-21 against
//! the OpenRouter transport (routed model `typesafe/jev-1.13-20260917`). A
//! three-level question answered:
//!
//! ```json
//! { "score": 1.99, "legend": {"0": "Mild", "1": "Moderate", "2": "Severe"},
//!   "probabilities": {"0": 0, "1": 0.01, "2": 0.99}, "confidence": 0.98 }
//! ```
//!
//! and a four-level one answered `"score": 1.99` with `"legend": {"0":
//! "Independent", "1": "NeedsAssistance", "2": "BedBound", "3":
//! "Immobile"}` and `"confidence": 0.97`. So, as measured: the legend maps
//! **numeric level → option key**, the scale is **0-based**, there is no
//! off-by-one against the declared levels, and `probabilities` is keyed by
//! the same level strings as the legend.
//!
//! This bridge therefore reads the legend as the authority for which level
//! an answer names, and rounds the level to the nearest declared index only
//! when a response carries no legend, or carries one whose entries do not
//! *all* read as `{numeric: string}` — a partly readable legend is not
//! trusted for the part that did parse, because resolving from a surviving
//! subset would name a level the answer did not. Either fallback is
//! recorded in `CallAccounting::raw_metadata` rather than failing the call,
//! and resolves the level exactly as the index path does, ties included.
//! The inverse orientation is deliberately not guessed at: accepting a
//! flipped legend would emit plausible-but-wrong ordinal levels under
//! perception provenance.
//!
//! The same `usage` block reports the call's cost directly — measured
//! `"usage": {"input_tokens": 383, "output_tokens": 32, "cost":
//! 1.6086e-05}` — so [`CallAccounting::cost_usd`] is the wire's own number,
//! with `PROMPT_USD_PER_MTOK` only as the fallback for a response that
//! omits it.
//!
//! ## The schema → questions mapping
//!
//! [`PerceiveRequest::schema`] is the JSON Schema the runtime derives from a
//! `PerceptionStep`'s `horis:extracts` (doc 06's mapping table;
//! `horismos_core::workflow::agent::extract_schema`). Each `oneOf` branch
//! under `instances.items` is one class, and each of its properties becomes
//! one Jev question:
//!
//! | JSON Schema shape | Jev primitive | Notes |
//! |---|---|---|
//! | `"type": "boolean"` | `noul` | `criteria.true`/`criteria.false` from the property's description or a [`PropertyHint`] |
//! | `"enum": [...]` (a `horis:oneOf` or closed subclass vocabulary) | `choice` | option keys are the enum members' local names, ≤ 255 |
//! | an `enum` marked ordinal by a [`PropertyHint`] | `score` | the enum members in declared order, 2–10 levels |
//! | `"type": "number"`/`"integer"`, open `"string"`, `date-time`, duration | **refused** | see below |
//!
//! ## The refusal is a feature
//!
//! Jev cannot produce a number or a free string — its own guidance is that
//! it is not a calculator. So when a derived schema asks for one, this
//! bridge fails the call with a [`BridgeError::Refused`] naming the
//! offending property, rather than approximating it. A numeric property
//! keeps coming through the external-snapshot path (doc 03, doc 08 § "Not a
//! Bridge") with real provenance. A bucketed `score` standing in for a heart
//! rate would be a fabricated measurement wearing a perception provenance;
//! an honest refusal routes the step to its `horis:fallback` edge instead,
//! which is what the runtime's error handling is for (doc 08 § Operational
//! semantics: every `BridgeError` is treated identically to low confidence).
//!
//! ## One class per call
//!
//! Jev answers every question it is asked. A perception schema carrying
//! more than one `oneOf` branch would therefore come back with one instance
//! of *every* class — including a class the text never mentions — and that
//! instance would enter the World wearing a perception provenance, which is
//! the same fabrication the numeric refusal exists to prevent. So a
//! multi-class schema is refused by name, and a `PerceptionStep` that wants
//! two classes runs as two steps.
//!
//! Inside the one class that is allowed, every declared property is asked
//! and every question is answered: this bridge cannot yet say "the text
//! does not state this" for an optional (`horis:cardinality "0..1"`)
//! property — it proposes a value for it like any other. Whether perception
//! grows a presence contract is an open design question owned elsewhere,
//! not something this bridge decides on its own.
//!
//! ## Narration is impossible here
//!
//! [`Bridge::narrate`] returns [`BridgeError::Refused`] without making a
//! call. Jev has no text output channel, so there is nothing to render a
//! proof into. Stitching prose out of answer keys would be this bridge
//! inventing narration the model never produced. A deployment that needs
//! narration registers a second, text-capable bridge for the narration step.
//!
//! ## Ordinality is declared at deployment, not in the ontology
//!
//! The RDFS-plus profile (doc 02) has no vocabulary for "this enum is
//! ordered", and inventing one ad hoc is forbidden (CLAUDE.md). So a
//! deployment that wants an enum sent as an ordered `score` rather than an
//! unordered `choice` says so with [`PropertyHint::ordinal`], alongside the
//! per-property question wording the ontology likewise cannot carry. Absent
//! a hint, every enum is a `choice` — the conservative reading.
//!
//! ## Feature gating
//!
//! `bridges-jev` is **default-OFF**, exactly like `bridges-openrouter`: with
//! both off, `cargo tree -p horismos-py` shows no `reqwest`, `rustls`,
//! `aws-lc`, `ring` or `openssl` at all, so nothing reaches a provider unless a
//! build asks. The published wheel does ask — `[tool.maturin] features` in
//! `pyproject.toml` compiles both HTTP bridges in — which is why its aarch64
//! build runs in the manylinux_2_28 image: `ring` needs a C compiler that
//! defines `__ARM_ARCH`, and the 2014 image's GCC 4.8.5 does not. Forcing the
//! macro there compiles, but yields a `ring` that rejects valid RSA and P-384
//! signatures (measured 2026-09-23), so never do that.
//!
//! Spec: `docs/framework/08-bridge.md`

use std::collections::BTreeMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use horismos_core::bridge::{
    Bridge, BridgeError, CallAccounting, NarrateRequest, NarrateResult, PerceiveRequest,
    PerceiveResult, RankRequest, RankResult,
};
use serde::Deserialize;
use serde_json::{Map, Value};

/// Jev's `choice` primitive accepts at most this many options.
const MAX_CHOICE_OPTIONS: usize = 255;
/// Jev's `score` primitive accepts this many ordered levels.
const SCORE_LEVELS: std::ops::RangeInclusive<usize> = 2..=10;
/// The **fallback** price for a Decisions response that carries no
/// `usage.cost`: OpenRouter's observed list price for `typesafe/jev-1.13`
/// on 2026-09-21, prompt tokens at $0.042 per million with completion
/// tokens billed at zero. The wire normally reports the cost itself
/// (measured 2026-09-21: `"usage": {"input_tokens": 383, "output_tokens":
/// 32, "cost": 1.6086e-05}`, exactly 383 × 0.042 / 1e6) and that number is
/// preferred — it is per-transport-correct by construction and cannot go
/// stale when a price changes. Either way a trace's `cost_usd` is what doc
/// 08 § Accounting calls it, a "cost estimate", and
/// `raw_metadata.price_basis` says which of the two sources it came from.
const PROMPT_USD_PER_MTOK: f64 = 0.042;
const HTTP_TIMEOUT: Duration = Duration::from_secs(60);

/// The two gateways that speak the Decisions protocol. Same request and
/// response shape; different URL, auth environment variable, and model alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JevTransport {
    /// `POST https://openrouter.ai/api/alpha/decisions`, `OPENROUTER_API_KEY`.
    OpenRouter,
    /// `POST https://api.typesafe.ai/v1/systemone`, `TYPESAFE_API_KEY`.
    TypeSafe,
}

impl JevTransport {
    /// The endpoint this transport posts to.
    pub fn url(self) -> &'static str {
        match self {
            JevTransport::OpenRouter => "https://openrouter.ai/api/alpha/decisions",
            JevTransport::TypeSafe => "https://api.typesafe.ai/v1/systemone",
        }
    }

    /// The environment variable holding this transport's bearer token. A
    /// `_FILE`-suffixed variant names a file to read it from instead.
    pub fn key_env(self) -> &'static str {
        match self {
            JevTransport::OpenRouter => "OPENROUTER_API_KEY",
            JevTransport::TypeSafe => "TYPESAFE_API_KEY",
        }
    }

    /// The model alias this transport routes under.
    pub fn default_model(self) -> &'static str {
        match self {
            JevTransport::OpenRouter => "typesafe/jev-1.13",
            JevTransport::TypeSafe => "jev-1.13",
        }
    }

    /// Whether this transport's bearer token is configured at all: its
    /// environment variable, **or** the `_FILE` variant naming a file to
    /// read it from. [`JevBridge::from_env`] selects on this rather than on
    /// the bare variable, so a file-only deployment is not routed to a
    /// transport it never configured and then told at call time that some
    /// other variable is unset.
    pub fn key_configured(self) -> bool {
        self.key_configured_in(|name| std::env::var(name).ok())
    }

    /// [`Self::key_configured`] against an injected environment, so the
    /// precedence is testable without mutating the process's own
    /// (`unsafe_code = "deny"` makes `std::env::set_var` a hard error).
    fn key_configured_in(self, lookup: impl Fn(&str) -> Option<String>) -> bool {
        let env = self.key_env();
        lookup(env).is_some() || lookup(&format!("{env}_FILE")).is_some()
    }

    /// The `raw_metadata.price_basis` token for a response that carries no
    /// `usage.cost` of its own, so a cost-per-case can be grouped by where
    /// its number came from. [`PROMPT_USD_PER_MTOK`] is OpenRouter's
    /// observed price, so on the TypeSafe transport the fallback is a
    /// resale price rather than this transport's own, and says so in its
    /// own token; what that means is in the constant's documentation, not
    /// repeated into every trace.
    fn fallback_price_basis(self) -> &'static str {
        match self {
            JevTransport::OpenRouter => "openrouter-list-price-estimate",
            JevTransport::TypeSafe => "openrouter-list-price-estimate-on-typesafe-transport",
        }
    }

    /// The short name recorded in `CallAccounting::raw_metadata`.
    fn name(self) -> &'static str {
        match self {
            JevTransport::OpenRouter => "openrouter",
            JevTransport::TypeSafe => "typesafe",
        }
    }
}

/// Deployment-side wording and ordinality for one property, keyed by its
/// local name. The ontology has no vocabulary for either (doc 02), so they
/// are supplied here rather than invented as `horis:` terms.
#[derive(Debug, Clone, Default)]
pub struct PropertyHint {
    /// The question put to the model. Defaults to the property schema's
    /// `description`, then to a generated sentence naming the property.
    pub instructions: Option<String>,
    /// `criteria.true` for a boolean property.
    pub when_true: Option<String>,
    /// `criteria.false` for a boolean property.
    pub when_false: Option<String>,
    /// Send this property's `enum` as an ordered `score` (2–10 levels, in
    /// declared order) rather than an unordered `choice`.
    pub ordinal: bool,
}

impl PropertyHint {
    /// A `noul` question's three strings, the shape the probe measured as
    /// producing well-separated probabilities.
    pub fn boolean(
        instructions: impl Into<String>,
        when_true: impl Into<String>,
        when_false: impl Into<String>,
    ) -> Self {
        PropertyHint {
            instructions: Some(instructions.into()),
            when_true: Some(when_true.into()),
            when_false: Some(when_false.into()),
            ordinal: false,
        }
    }
}

/// A `text → decisions` [`Bridge`] over TypeSafe's Jev.
pub struct JevBridge {
    transport: JevTransport,
    model: String,
    configured_id: String,
    endpoint: String,
    /// A key supplied directly by the deployment, taking precedence over the
    /// environment (the same direct-key / env-var / file precedence
    /// `OpenRouterBridge` inherits from `litellm-rust`).
    api_key: Option<String>,
    hints: BTreeMap<String, PropertyHint>,
    client: reqwest::Client,
    /// The model the wire actually reported, latched on the first successful
    /// response — see [`Bridge::id`]'s note.
    observed_id: OnceLock<String>,
}

impl JevBridge {
    /// A bridge over `transport`, using that transport's default model alias.
    pub fn new(transport: JevTransport) -> Self {
        let model = transport.default_model().to_string();
        Self::with_model(transport, model)
    }

    /// A bridge over `transport` pinned to a specific model id.
    pub fn with_model(transport: JevTransport, model: impl Into<String>) -> Self {
        let model = model.into();
        let client = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .expect("reqwest client construction with only a timeout cannot fail");
        JevBridge {
            configured_id: format!("jev:{model}"),
            endpoint: transport.url().to_string(),
            transport,
            model,
            api_key: None,
            hints: BTreeMap::new(),
            client,
            observed_id: OnceLock::new(),
        }
    }

    /// Picks a transport from the environment: OpenRouter when its key is
    /// configured — `OPENROUTER_API_KEY`, or `OPENROUTER_API_KEY_FILE`
    /// naming a file to read it from — otherwise TypeSafe. `JEV_MODEL`
    /// overrides the chosen transport's default model alias.
    pub fn from_env() -> Self {
        Self::from_env_in(|name| std::env::var(name).ok())
    }

    /// [`Self::from_env`] against an injected environment, so the routing
    /// decision itself is testable without mutating the process's own
    /// (`unsafe_code = "deny"` makes `std::env::set_var` a hard error).
    fn from_env_in(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let transport = if JevTransport::OpenRouter.key_configured_in(&lookup) {
            JevTransport::OpenRouter
        } else {
            JevTransport::TypeSafe
        };
        match lookup("JEV_MODEL") {
            Some(model) => Self::with_model(transport, model),
            None => Self::new(transport),
        }
    }

    /// Supplies the bearer token directly, in preference to the transport's
    /// environment variable — for a deployment that holds its secrets
    /// somewhere other than the process environment. Never logged, never
    /// placed in a trace.
    #[must_use]
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Registers deployment-side wording/ordinality for the property whose
    /// local name is `property` (e.g. `"reducedMobility"`).
    #[must_use]
    pub fn with_hint(mut self, property: impl Into<String>, hint: PropertyHint) -> Self {
        self.hints.insert(property.into(), hint);
        self
    }

    fn refused(&self, reason: impl Into<String>) -> BridgeError {
        BridgeError::Refused {
            bridge: self.configured_id.clone(),
            reason: reason.into(),
        }
    }

    fn malformed(&self, reason: impl Into<String>) -> BridgeError {
        BridgeError::MalformedOutput {
            bridge: self.configured_id.clone(),
            reason: reason.into(),
        }
    }

    /// Resolves the bearer token: a directly supplied key first, then the
    /// transport's environment variable, then a local file named by its
    /// `_FILE`-suffixed variant (the
    /// same precedence `OpenRouterBridge` uses). Never logged, never placed
    /// in a trace or `CallAccounting::raw_metadata`.
    fn resolve_key(&self) -> Result<String, BridgeError> {
        if let Some(key) = &self.api_key {
            return Ok(key.clone());
        }
        let env = self.transport.key_env();
        if let Ok(key) = std::env::var(env) {
            return Ok(key);
        }
        let file_env = format!("{env}_FILE");
        let Ok(path) = std::env::var(&file_env) else {
            return Err(BridgeError::RequestFailed {
                bridge: self.configured_id.clone(),
                reason: format!("{env} is unset and {file_env} names no fallback"),
            });
        };
        std::fs::read_to_string(&path)
            .map(|s| s.trim().to_string())
            .map_err(|e| BridgeError::RequestFailed {
                bridge: self.configured_id.clone(),
                reason: format!("{file_env} {path:?}: {e}"),
            })
    }

    /// One Decisions call: `{model, state, questions}` in, a parsed
    /// [`JevResponse`] plus the observed latency out.
    async fn ask(
        &self,
        state: &str,
        questions: Map<String, Value>,
    ) -> Result<(JevResponse, Duration), BridgeError> {
        let key = self.resolve_key()?;
        let body = serde_json::json!({
            "model": self.model,
            "state": state,
            "questions": Value::Object(questions),
        });
        let started = Instant::now();
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(key)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    BridgeError::Timeout {
                        bridge: self.configured_id.clone(),
                    }
                } else {
                    BridgeError::RequestFailed {
                        bridge: self.configured_id.clone(),
                        reason: e.to_string(),
                    }
                }
            })?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| BridgeError::RequestFailed {
                bridge: self.configured_id.clone(),
                reason: format!("could not read the response body: {e}"),
            })?;
        let elapsed = started.elapsed();
        if !status.is_success() {
            let snippet: String = text.chars().take(500).collect();
            return Err(BridgeError::RequestFailed {
                bridge: self.configured_id.clone(),
                reason: format!("HTTP {status} from {}: {snippet}", self.endpoint),
            });
        }
        let parsed: JevResponse = serde_json::from_str(&text)
            .map_err(|e| self.malformed(format!("response was not a Decisions payload: {e}")))?;
        if let Some(model) = &parsed.model {
            // Latch the routed model id: `Bridge::id` hands out a `&str`, so
            // it needs somewhere stable to borrow from.
            let _ = self.observed_id.set(format!("jev:{model}"));
        }
        Ok((parsed, elapsed))
    }

    fn accounting(
        &self,
        response: &JevResponse,
        elapsed: Duration,
        extra: Map<String, Value>,
    ) -> CallAccounting {
        let prompt_tokens = response.usage.as_ref().and_then(|u| u.input_tokens);
        let completion_tokens = response.usage.as_ref().and_then(|u| u.output_tokens);
        // The endpoint reports the call's cost itself; the constant is only
        // what a response that omits it falls back to. `price_basis` says
        // which of the two this number is, so a cost-per-case published as
        // a benchmark metric names where it came from.
        let (cost_usd, price_basis) = match response.usage.as_ref().and_then(|u| u.cost) {
            Some(cost) => (Some(cost), "wire-usage-cost"),
            None => (
                prompt_tokens.map(|t| t as f64 * PROMPT_USD_PER_MTOK / 1_000_000.0),
                self.transport.fallback_price_basis(),
            ),
        };
        let mut raw = Map::new();
        raw.insert(
            "transport".to_string(),
            Value::String(self.transport.name().to_string()),
        );
        raw.insert(
            "price_basis".to_string(),
            Value::String(price_basis.to_string()),
        );
        raw.extend(extra);
        CallAccounting {
            latency_ms: Some(elapsed.as_millis() as u64),
            prompt_tokens,
            completion_tokens,
            cost_usd,
            model: response.model.clone(),
            // Jev answers every question it is asked or fails the call; there
            // is no partial-completion concept for it to report.
            finish_reason: None,
            raw_metadata: Some(Value::Object(raw)),
        }
    }
}

// -- The wire response -----------------------------------------------------

#[derive(Debug, Deserialize)]
struct JevResponse {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    answers: BTreeMap<String, JevAnswer>,
    #[serde(default)]
    usage: Option<JevUsage>,
}

#[derive(Debug, Deserialize)]
struct JevAnswer {
    #[serde(default)]
    noul: Option<f64>,
    #[serde(default)]
    choice: Option<String>,
    #[serde(default)]
    probabilities: Option<BTreeMap<String, f64>>,
    #[serde(default)]
    confidence: Option<f64>,
    #[serde(default)]
    score: Option<f64>,
    /// The level → option-key mapping the endpoint reports alongside a
    /// `score`. Authoritative when present: it is what says which scale the
    /// level is on, which the level alone cannot.
    #[serde(default)]
    legend: Option<Map<String, Value>>,
}

#[derive(Debug, Deserialize)]
struct JevUsage {
    #[serde(default)]
    input_tokens: Option<u64>,
    #[serde(default)]
    output_tokens: Option<u64>,
    /// The call's cost in USD, reported by the endpoint itself (measured
    /// 2026-09-21). Preferred over `PROMPT_USD_PER_MTOK`.
    #[serde(default)]
    cost: Option<f64>,
}

// -- Schema → questions ----------------------------------------------------

/// What one schema property was translated into.
#[derive(Debug, Clone, PartialEq)]
enum PropKind {
    Noul,
    /// `(wire option key, the schema enum member it stands for)`, in schema
    /// order. The wire key is the member's local name — a safe identifier —
    /// so the answer has to be mapped back through this list.
    Choice(Vec<(String, String)>),
    /// The same pairs, but asked as an ordered `score`.
    Score(Vec<(String, String)>),
}

#[derive(Debug, Clone)]
struct PropPlan {
    question_id: String,
    name: String,
    kind: PropKind,
    /// The derived schema wrapped this property in an array (a multi-valued
    /// `horis:cardinality`); the single answer is wrapped back into one.
    multi: bool,
}

#[derive(Debug, Clone)]
struct InstancePlan {
    /// The `class` const the derived schema pins this branch to.
    class: Option<String>,
    props: Vec<PropPlan>,
}

/// The translation of one [`PerceiveRequest::schema`]: the wire `questions`
/// object plus everything needed to put the answers back into instances.
#[derive(Debug, Clone)]
struct QuestionPlan {
    instances: Vec<InstancePlan>,
    questions: Map<String, Value>,
}

/// The `oneOf` branches under `instances.items`, or the schema itself when
/// it is already a bare class object.
fn class_schemas(schema: &Value) -> Vec<&Value> {
    let Some(items) = schema.pointer("/properties/instances/items") else {
        return vec![schema];
    };
    match items.get("oneOf").and_then(Value::as_array) {
        Some(branches) => branches.iter().collect(),
        None => vec![items],
    }
}

/// Reads an `enum` of strings off a property schema.
fn enum_members(schema: &Value) -> Option<Vec<String>> {
    let members = schema.get("enum")?.as_array()?;
    members
        .iter()
        .map(|m| m.as_str().map(str::to_string))
        .collect()
}

/// The last path segment of a compact IRI (`":HoldProphylaxis"` →
/// `"HoldProphylaxis"`) — a wire-safe option key.
fn local_name(member: &str) -> String {
    member
        .rsplit([':', '/', '#'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(member)
        .to_string()
}

/// Assigns each enum member a unique wire key, de-duplicating local names
/// that collide across namespaces by falling back to the full member string.
fn option_keys(members: &[String]) -> Vec<(String, String)> {
    let mut keys: Vec<(String, String)> = Vec::with_capacity(members.len());
    for member in members {
        let mut key = local_name(member);
        if keys.iter().any(|(existing, _)| *existing == key) {
            key = member.clone();
        }
        keys.push((key, member.clone()));
    }
    keys
}

impl JevBridge {
    /// Translates a derived perception schema into Jev questions, or refuses
    /// with a [`BridgeError::Refused`] naming the property Jev cannot answer.
    fn plan_questions(&self, schema: &Value) -> Result<QuestionPlan, BridgeError> {
        let branches = class_schemas(schema);
        if branches.len() > 1 {
            let classes: Vec<String> = branches
                .iter()
                .enumerate()
                .map(|(index, branch)| {
                    branch
                        .pointer("/properties/class/const")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .unwrap_or_else(|| format!("branch {index}"))
                })
                .collect();
            return Err(self.refused(format!(
                "the perception schema extracts {} classes ({}), and Jev answers every question \
                 it is asked — it would propose an instance of each one even for a class the \
                 text never mentions, which is a fabricated fact wearing a perception \
                 provenance. Extract one class per perception step",
                classes.len(),
                classes.join(", ")
            )));
        }
        let mut instances = Vec::with_capacity(branches.len());
        let mut questions = Map::new();

        for (index, branch) in branches.iter().enumerate() {
            let class = branch
                .pointer("/properties/class/const")
                .and_then(Value::as_str)
                .map(str::to_string);
            let subject = class.as_deref().unwrap_or("case");
            let Some(properties) = branch.get("properties").and_then(Value::as_object) else {
                return Err(self.refused(format!(
                    "the perception schema's branch {index} declares no properties to extract"
                )));
            };

            let mut props = Vec::new();
            for (name, declared) in properties {
                if name == "class" || declared.get("const").is_some() {
                    continue;
                }
                // A multi-valued property arrives as an array wrapper; the
                // element schema is what Jev is actually asked about.
                let (element, multi) = match declared.get("type").and_then(Value::as_str) {
                    Some("array") => match declared.get("items") {
                        Some(items) => (items, true),
                        None => {
                            return Err(self.refused(format!(
                                "property `{name}` on `{subject}` is an array with no declared \
                                 item schema, so there is nothing for Jev to answer"
                            )));
                        }
                    },
                    _ => (declared, false),
                };
                let hint = self.hints.get(name.as_str());
                let kind = self.classify(subject, name, element, hint)?;
                let question_id = format!("i{index}_{name}");
                questions.insert(
                    question_id.clone(),
                    build_question(subject, name, element, &kind, hint),
                );
                props.push(PropPlan {
                    question_id,
                    name: name.clone(),
                    kind,
                    multi,
                });
            }
            instances.push(InstancePlan { class, props });
        }

        if questions.is_empty() {
            return Err(self.refused(
                "the perception schema asks for no answerable property — Jev has nothing to decide"
                    .to_string(),
            ));
        }
        Ok(QuestionPlan {
            instances,
            questions,
        })
    }

    /// The mapping table in the module doc, including its refusals.
    fn classify(
        &self,
        subject: &str,
        name: &str,
        element: &Value,
        hint: Option<&PropertyHint>,
    ) -> Result<PropKind, BridgeError> {
        if let Some(members) = enum_members(element) {
            if members.is_empty() {
                return Err(self.refused(format!(
                    "property `{name}` on `{subject}` declares an empty enum, so there is no \
                     option for Jev to choose"
                )));
            }
            let options = option_keys(&members);
            if hint.is_some_and(|h| h.ordinal) {
                if !SCORE_LEVELS.contains(&options.len()) {
                    return Err(self.refused(format!(
                        "property `{name}` on `{subject}` is declared ordinal but has {} levels; \
                         Jev's score primitive takes {}-{} ordered levels",
                        options.len(),
                        SCORE_LEVELS.start(),
                        SCORE_LEVELS.end()
                    )));
                }
                return Ok(PropKind::Score(options));
            }
            if options.len() > MAX_CHOICE_OPTIONS {
                return Err(self.refused(format!(
                    "property `{name}` on `{subject}` has {} options; Jev's choice primitive takes \
                     at most {MAX_CHOICE_OPTIONS}",
                    options.len()
                )));
            }
            return Ok(PropKind::Choice(options));
        }

        match element.get("type").and_then(Value::as_str) {
            Some("boolean") => Ok(PropKind::Noul),
            other => {
                let declared = other.unwrap_or("unconstrained");
                let format = element
                    .get("format")
                    .and_then(Value::as_str)
                    .map(|f| format!(" (format `{f}`)"))
                    .unwrap_or_default();
                Err(self.refused(format!(
                    "property `{name}` on `{subject}` has JSON Schema type `{declared}`{format}, \
                     and Jev answers only noul/choice/score — it cannot produce a number or a \
                     free string. Approximating it as a score would fabricate a measurement \
                     under perception provenance; supply `{name}` through the external-snapshot \
                     path instead, or narrow its range to a closed vocabulary"
                )))
            }
        }
    }
}

/// One wire question. `criteria` always carries both `noul` keys, which the
/// OpenRouter transport requires and the TypeSafe transport accepts.
fn build_question(
    subject: &str,
    name: &str,
    element: &Value,
    kind: &PropKind,
    hint: Option<&PropertyHint>,
) -> Value {
    let description = element.get("description").and_then(Value::as_str);
    let instructions = hint
        .and_then(|h| h.instructions.clone())
        .or_else(|| description.map(str::to_string))
        .unwrap_or_else(|| match kind {
            PropKind::Noul => format!("Does `{name}` hold for this `{subject}`?"),
            _ => format!("Which `{name}` does the text support for this `{subject}`?"),
        });
    match kind {
        PropKind::Noul => {
            let when_true = hint
                .and_then(|h| h.when_true.clone())
                .unwrap_or_else(|| format!("The text supports `{name}` for this `{subject}`."));
            let when_false = hint.and_then(|h| h.when_false.clone()).unwrap_or_else(|| {
                format!("The text does not support `{name}`, or states the contrary.")
            });
            serde_json::json!({
                "type": "noul",
                "instructions": instructions,
                "criteria": {"true": when_true, "false": when_false},
            })
        }
        PropKind::Choice(options) => {
            // The ontology carries no per-member description vocabulary
            // (doc 02), so the member itself — the compact IRI the schema's
            // enum declares — is the whole of what the model is told about
            // an option. That is also the value shape `rank` sends for the
            // same `choice` primitive: one wire type, one criteria shape.
            let criteria: Map<String, Value> = options
                .iter()
                .map(|(key, member)| (key.clone(), Value::String(member.clone())))
                .collect();
            serde_json::json!({
                "type": "choice",
                "instructions": instructions,
                "criteria": criteria,
            })
        }
        PropKind::Score(levels) => {
            let criteria: Vec<Value> = levels
                .iter()
                .map(|(key, _)| Value::String(key.clone()))
                .collect();
            serde_json::json!({
                "type": "score",
                "instructions": instructions,
                "criteria": criteria,
            })
        }
    }
}

/// The nearest level in a `{numeric level: option key}` legend to `raw`, as
/// `(level key, option key)` — the level key being what `probabilities` is
/// keyed by. `None` unless *every* entry reads that way, which is the
/// caller's signal to fall back to the declared index: a legend resolved
/// from only the entries that happened to parse could name a level the
/// answer did not. The inverse orientation is deliberately not guessed at.
fn level_from_legend(legend: &Map<String, Value>, raw: f64) -> Option<(String, String)> {
    let mut nearest: Option<(f64, f64, &String, &str)> = None;
    for (level, option) in legend {
        let (Ok(parsed), Some(option)) = (level.trim().parse::<f64>(), option.as_str()) else {
            return None;
        };
        let distance = (parsed - raw).abs();
        let closer = nearest.is_none_or(|(best, best_level, _, _)| {
            // Equidistant levels go to the higher one, the same round-half-up
            // the index path's `f64::round` performs, so the two paths cannot
            // disagree about a midpoint.
            distance < best || (distance == best && parsed > best_level)
        });
        if closer {
            nearest = Some((distance, parsed, level, option));
        }
    }
    nearest.map(|(_, _, level, option)| (level.clone(), option.to_string()))
}

/// One answered property: the JSON value to put into the instance, and how
/// sure the model was **of that value** (not of `true`).
#[derive(Debug)]
struct Answered {
    value: Value,
    confidence: f64,
    /// The answer carried a `legend` this bridge could not read as
    /// `{numeric level: option key}`, so the level was resolved by the
    /// index fallback instead. Surfaced in `raw_metadata`.
    legend_unreadable: bool,
}

impl JevBridge {
    fn read_answer(&self, plan: &PropPlan, answer: &JevAnswer) -> Result<Answered, BridgeError> {
        match &plan.kind {
            PropKind::Noul => {
                let p = answer.noul.ok_or_else(|| {
                    self.malformed(format!(
                        "question {:?} was asked as a noul but the answer carries no `noul` \
                         probability",
                        plan.question_id
                    ))
                })?;
                Ok(Answered {
                    value: Value::Bool(p >= 0.5),
                    // Certainty in the value actually asserted: a `noul` of
                    // 0.04 is a confident `false`, not an unconfident one.
                    confidence: p.max(1.0 - p),
                    legend_unreadable: false,
                })
            }
            PropKind::Choice(options) => {
                let key = answer.choice.as_deref().ok_or_else(|| {
                    self.malformed(format!(
                        "question {:?} was asked as a choice but the answer carries no `choice` \
                         key",
                        plan.question_id
                    ))
                })?;
                let member = options
                    .iter()
                    .find(|(option, _)| option == key)
                    .map(|(_, member)| member.clone())
                    .ok_or_else(|| {
                        self.malformed(format!(
                            "question {:?} was answered with option {key:?}, which is not one of \
                             the options offered",
                            plan.question_id
                        ))
                    })?;
                Ok(Answered {
                    value: Value::String(member),
                    confidence: answer.confidence.unwrap_or_else(|| {
                        answer
                            .probabilities
                            .as_ref()
                            .and_then(|p| p.get(key).copied())
                            // D9: a confidence the wire omitted normalizes
                            // to 0.0 — fails safe into the fallback edge.
                            .unwrap_or(0.0)
                    }),
                    legend_unreadable: false,
                })
            }
            PropKind::Score(levels) => {
                let raw = answer.score.ok_or_else(|| {
                    self.malformed(format!(
                        "question {:?} was asked as a score but the answer carries no `score` \
                         level",
                        plan.question_id
                    ))
                })?;
                if levels.is_empty() {
                    return Err(self.malformed(format!(
                        "question {:?} was asked as a score over no levels",
                        plan.question_id
                    )));
                }
                // Jev reports a continuous level (e.g. 1.99), and the
                // response's own `legend` says which declared level that
                // names — measured as `{numeric level: option key}` on a
                // 0-based scale, with `probabilities` keyed by the same
                // level strings. A response without a readable legend still
                // has to be answerable, so the level rounds to the nearest
                // declared index instead; nothing is invented either way.
                let legend = answer.legend.as_ref().filter(|l| !l.is_empty());
                let resolved = legend.and_then(|legend| level_from_legend(legend, raw));
                let legend_unreadable = legend.is_some() && resolved.is_none();
                let (level_key, key) = resolved.unwrap_or_else(|| {
                    let index = (raw.round().max(0.0) as usize).min(levels.len() - 1);
                    (index.to_string(), levels[index].0.clone())
                });
                let member = levels
                    .iter()
                    .find(|(option, _)| *option == key)
                    .map(|(_, member)| member.clone())
                    .ok_or_else(|| {
                        self.malformed(format!(
                            "question {:?} was answered at level {key:?}, which is not one of \
                             the levels offered",
                            plan.question_id
                        ))
                    })?;
                Ok(Answered {
                    value: Value::String(member),
                    confidence: answer.confidence.unwrap_or_else(|| {
                        answer
                            .probabilities
                            .as_ref()
                            .and_then(|p| p.get(&level_key).copied())
                            // D9, as on the choice path: a confidence the
                            // wire omitted normalizes to 0.0.
                            .unwrap_or(0.0)
                    }),
                    legend_unreadable,
                })
            }
        }
    }
}

#[async_trait::async_trait]
impl Bridge for JevBridge {
    async fn perceive(&self, req: PerceiveRequest) -> Result<PerceiveResult, BridgeError> {
        let plan = self.plan_questions(&req.schema)?;
        let (response, elapsed) = self.ask(&req.text, plan.questions.clone()).await?;

        let mut instances = Vec::with_capacity(plan.instances.len());
        let mut fact_confidence = BTreeMap::new();
        let mut by_property = Vec::with_capacity(plan.instances.len());
        let mut unreadable_legends: Vec<Value> = Vec::new();

        for (index, instance) in plan.instances.iter().enumerate() {
            let mut object = Map::new();
            if let Some(class) = &instance.class {
                object.insert("class".to_string(), Value::String(class.clone()));
            }
            let mut confidences = Map::new();
            let mut weakest: Option<f64> = None;
            for prop in &instance.props {
                let answer = response.answers.get(&prop.question_id).ok_or_else(|| {
                    self.malformed(format!(
                        "question {:?} was asked but the response has no answer for it",
                        prop.question_id
                    ))
                })?;
                let answered = self.read_answer(prop, answer)?;
                if answered.legend_unreadable {
                    unreadable_legends.push(Value::String(prop.question_id.clone()));
                }
                let value = if prop.multi {
                    Value::Array(vec![answered.value])
                } else {
                    answered.value
                };
                object.insert(prop.name.clone(), value);
                confidences.insert(
                    prop.name.clone(),
                    serde_json::json!(round6(answered.confidence)),
                );
                weakest =
                    Some(weakest.map_or(answered.confidence, |w: f64| w.min(answered.confidence)));
            }
            if let Some(weakest) = weakest {
                // Per-instance confidence is the weakest property in it: an
                // instance is only as trustworthy as its shakiest fact, and
                // averaging would let one coin-flip hide behind three
                // certainties.
                fact_confidence.insert(index, weakest);
            }
            instances.push(Value::Object(object));
            by_property.push(Value::Object(confidences));
        }

        // Call confidence — what the runtime compares against the step's
        // `horis:threshold` — is the weakest answer in the whole call, for
        // the same reason, and 0.0 when there is nothing to be confident
        // about (D9's fail-safe normalization).
        let confidence = fact_confidence.values().copied().fold(f64::NAN, f64::min);
        let confidence = if confidence.is_nan() { 0.0 } else { confidence };

        // Jev's per-question probabilities have no dedicated trace field and
        // `fact_confidence` is keyed by instance, so the per-property detail
        // rides the documented audit escape hatch rather than being dropped.
        let mut extra = Map::new();
        extra.insert(
            "fact_confidence_by_property".to_string(),
            Value::Array(by_property),
        );
        if !unreadable_legends.is_empty() {
            // The level came from the declared index because the legend it
            // should have come from did not read as `{numeric: string}`.
            // Visible in the trace rather than failing the perception.
            extra.insert(
                "unreadable_score_legends".to_string(),
                Value::Array(unreadable_legends),
            );
        }
        let accounting = self.accounting(&response, elapsed, extra);
        Ok(PerceiveResult {
            instances,
            confidence,
            fact_confidence,
            accounting,
        })
    }

    /// Always refused: Jev has no text output channel. See the module doc.
    async fn narrate(&self, _req: NarrateRequest) -> Result<NarrateResult, BridgeError> {
        Err(self.refused(
            "Jev is a text→decisions model with no text output channel, so it cannot render a \
             proof into prose. Register a text-capable bridge for the narration step; this \
             bridge will not stitch prose out of answer keys."
                .to_string(),
        ))
    }

    async fn rank(&self, req: RankRequest) -> Result<RankResult, BridgeError> {
        if req.candidates.is_empty() {
            return Err(self.malformed("rank called with no candidates".to_string()));
        }
        if req.candidates.len() > MAX_CHOICE_OPTIONS {
            return Err(self.refused(format!(
                "rank was given {} candidates; Jev's choice primitive takes at most \
                 {MAX_CHOICE_OPTIONS}",
                req.candidates.len()
            )));
        }
        // The runtime has already computed the precondition-legal candidates
        // (doc 07), so this is exactly Jev's shape: one closed choice the
        // model cannot add to.
        let keys: Vec<String> = (0..req.candidates.len()).map(|i| format!("c{i}")).collect();
        let criteria: Map<String, Value> = keys
            .iter()
            .zip(&req.candidates)
            .map(|(key, candidate)| (key.clone(), Value::String(candidate.to_string())))
            .collect();
        let question = serde_json::json!({
            "type": "choice",
            "instructions": "Select the single candidate that best satisfies the stated context. \
                             Each option's description is the candidate itself, as JSON.",
            "criteria": criteria,
        });
        let mut questions = Map::new();
        questions.insert("selection".to_string(), question);

        let (response, elapsed) = self.ask(&req.context, questions).await?;
        let answer = response.answers.get("selection").ok_or_else(|| {
            self.malformed("the response has no answer for the `selection` question".to_string())
        })?;
        let key = answer.choice.as_deref().ok_or_else(|| {
            self.malformed("the `selection` answer carries no `choice` key".to_string())
        })?;
        let chosen = keys.iter().position(|k| k == key).ok_or_else(|| {
            self.malformed(format!(
                "the model answered with option {key:?}, which is not one of the {} candidates \
                 offered",
                req.candidates.len()
            ))
        })?;
        let confidence = answer.confidence.unwrap_or_else(|| {
            answer
                .probabilities
                .as_ref()
                .and_then(|p| p.get(key).copied())
                .unwrap_or(0.0)
        });
        Ok(RankResult {
            chosen,
            confidence,
            accounting: self.accounting(&response, elapsed, Map::new()),
        })
    }

    /// The model **actually invoked**: the `model` the Decisions response
    /// reported (`typesafe/jev-1.13-20260917` for a call that requested the
    /// `typesafe/jev-1.13` alias), latched on the first successful call.
    /// Before any call — and after one that never returned a `model` — this
    /// is the configured alias.
    fn id(&self) -> &str {
        self.observed_id
            .get()
            .map(String::as_str)
            .unwrap_or(&self.configured_id)
    }
}

/// Keeps `raw_metadata` a stable, diffable audit value rather than a float
/// with 17 digits of noise.
fn round6(value: f64) -> f64 {
    (value * 1e6).round() / 1e6
}

#[cfg(test)]
mod tests {
    use super::*;
    use horismos_core::reason::ProofKind;
    use horismos_core::workflow::agent::extract_schema;
    // `Value` is `serde_json::Value` throughout this module, so the typed one
    // comes in under a name of its own.
    use horismos_core::{Ontology, Provenance, Ref, Value as TypedValue, World};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    const VTE_TTL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/vte/vte.ttl");

    fn vte_ontology() -> Ontology {
        Ontology::load(VTE_TTL).expect("vte.ttl must load and verify")
    }

    /// The schema the runtime really derives for `examples/vte/vte.ttl`'s
    /// `:VteCase` — four booleans, and no `:recommendation` (it is
    /// `horis:assignedBy horis:RulesOnly`, so the derivation never shows it
    /// to a model). Derived rather than hand-copied, so every claim these
    /// tests make about "what the model is shown" is a claim about the
    /// shipped path instead of about a fixture.
    fn vte_schema() -> Value {
        let onto = vte_ontology();
        let vte_case = onto.node_ref(":VteCase").expect(":VteCase is declared");
        extract_schema(&onto, &[vte_case])
    }

    fn bridge() -> JevBridge {
        JevBridge::new(JevTransport::OpenRouter)
    }

    // -- Schema → questions -------------------------------------------------

    #[test]
    fn booleans_become_noul_questions_with_both_criteria_keys() {
        let plan = bridge()
            .plan_questions(&vte_schema())
            .expect("four booleans are answerable");
        assert_eq!(plan.questions.len(), 4);
        let q = plan
            .questions
            .get("i0_reducedMobility")
            .expect("the reducedMobility question is planned");
        assert_eq!(q.get("type"), Some(&Value::String("noul".to_string())));
        // The OpenRouter transport rejects a noul whose criteria omit either
        // key, so both are always emitted.
        assert!(
            q.pointer("/criteria/true").is_some(),
            "missing criteria.true: {q}"
        );
        assert!(
            q.pointer("/criteria/false").is_some(),
            "missing criteria.false: {q}"
        );
        assert_eq!(plan.instances.len(), 1);
        assert_eq!(plan.instances[0].class.as_deref(), Some(":VteCase"));
    }

    #[test]
    fn a_property_hint_supplies_the_question_wording() {
        let bridge = bridge().with_hint(
            "activeCancer",
            PropertyHint::boolean(
                "Does the patient have active malignancy?",
                "Malignancy currently present or under active treatment.",
                "No malignancy, or a historical malignancy in remission.",
            ),
        );
        let plan = bridge
            .plan_questions(&vte_schema())
            .expect("the schema is answerable");
        let q = &plan.questions["i0_activeCancer"];
        assert_eq!(
            q.get("instructions").and_then(Value::as_str),
            Some("Does the patient have active malignancy?")
        );
        assert_eq!(
            q.pointer("/criteria/true").and_then(Value::as_str),
            Some("Malignancy currently present or under active treatment.")
        );
    }

    #[test]
    fn an_enum_becomes_a_choice_keyed_by_local_name() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"instances": {"type": "array", "items": {"oneOf": [{
                "type": "object",
                "properties": {
                    "class": {"const": ":Case"},
                    "recommendation": {"type": "string", "enum": [
                        ":PharmacologicProphylaxis", ":HoldProphylaxis", ":NoProphylaxisIndicated"
                    ]},
                },
            }]}}},
        });
        let plan = bridge()
            .plan_questions(&schema)
            .expect("a closed vocabulary is answerable");
        let q = &plan.questions["i0_recommendation"];
        assert_eq!(q.get("type"), Some(&Value::String("choice".to_string())));
        let criteria = q
            .get("criteria")
            .and_then(Value::as_object)
            .expect("choice criteria are an object");
        assert_eq!(criteria.len(), 3);
        assert_eq!(
            criteria.get("HoldProphylaxis"),
            Some(&Value::String(":HoldProphylaxis".to_string())),
            "each option is described by the member it stands for — the same criteria value \
             shape `rank` sends for this primitive, rather than a null: {criteria:?}"
        );
        assert!(
            criteria.values().all(Value::is_string),
            "no option may be described as null: {criteria:?}"
        );
    }

    #[test]
    fn an_ordinal_hint_turns_an_enum_into_a_score() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"instances": {"type": "array", "items": {"oneOf": [{
                "type": "object",
                "properties": {
                    "class": {"const": ":Case"},
                    "severity": {"type": "string", "enum": [":Mild", ":Moderate", ":Severe"]},
                },
            }]}}},
        });
        let bridge = bridge().with_hint(
            "severity",
            PropertyHint {
                ordinal: true,
                ..PropertyHint::default()
            },
        );
        let plan = bridge
            .plan_questions(&schema)
            .expect("3 levels is in range");
        let q = &plan.questions["i0_severity"];
        assert_eq!(q.get("type"), Some(&Value::String("score".to_string())));
        assert_eq!(
            q.get("criteria"),
            Some(&serde_json::json!(["Mild", "Moderate", "Severe"])),
            "score levels must stay in declared order: {q}"
        );
    }

    #[test]
    fn a_multi_valued_property_is_asked_about_its_element_schema() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"instances": {"type": "array", "items": {"oneOf": [{
                "type": "object",
                "properties": {
                    "class": {"const": ":Case"},
                    "flags": {"type": "array", "items": {"type": "boolean"}},
                },
            }]}}},
        });
        let plan = bridge()
            .plan_questions(&schema)
            .expect("booleans are answerable");
        assert_eq!(
            plan.questions["i0_flags"]["type"],
            Value::String("noul".into())
        );
        assert!(plan.instances[0].props[0].multi);
    }

    // -- The refusal, which is the point -----------------------------------

    #[test]
    fn a_numeric_property_is_refused_by_name_and_never_approximated() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"instances": {"type": "array", "items": {"oneOf": [{
                "type": "object",
                "properties": {
                    "class": {"const": ":Patient"},
                    "heartRate": {"type": "number", "minimum": 0, "maximum": 300},
                },
            }]}}},
        });
        let err = bridge()
            .plan_questions(&schema)
            .expect_err("Jev cannot produce a number; the bridge must refuse");
        let BridgeError::Refused { reason, .. } = &err else {
            panic!("expected a refusal naming the property, got {err:?}");
        };
        assert!(
            reason.contains("heartRate"),
            "the refusal must name the offending property: {reason}"
        );
        assert!(
            reason.contains("number"),
            "the refusal must say what was asked for: {reason}"
        );
        assert!(
            reason.contains("external-snapshot"),
            "the refusal must point at the path that can supply it: {reason}"
        );
        // And nothing is silently substituted: an ordinal hint cannot unlock
        // a numeric property either, because there are no declared levels to
        // bucket it into. Approximating a heart rate as a 1-5 score would be
        // a fabricated measurement wearing perception provenance.
        let hinted = bridge().with_hint(
            "heartRate",
            PropertyHint {
                ordinal: true,
                ..PropertyHint::default()
            },
        );
        let err = hinted
            .plan_questions(&schema)
            .expect_err("an ordinal hint must not turn a number into a score");
        assert!(matches!(err, BridgeError::Refused { .. }), "{err:?}");
        assert!(err.domain_text().contains("heartRate"), "{err:?}");
    }

    #[test]
    fn an_integer_property_is_refused() {
        let schema = serde_json::json!({"type": "object", "properties": {"instances": {
            "type": "array", "items": {"oneOf": [{"type": "object", "properties": {
                "class": {"const": ":Patient"}, "ageYears": {"type": "integer"}}}]}}}});
        let err = bridge()
            .plan_questions(&schema)
            .expect_err("integers are refused");
        assert!(matches!(err, BridgeError::Refused { .. }), "got {err:?}");
        assert!(err.domain_text().contains("ageYears"), "{err:?}");
    }

    #[test]
    fn an_open_string_property_is_refused() {
        let schema = serde_json::json!({"type": "object", "properties": {"instances": {
            "type": "array", "items": {"oneOf": [{"type": "object", "properties": {
                "class": {"const": ":Patient"}, "chiefComplaint": {"type": "string"}}}]}}}});
        let err = bridge()
            .plan_questions(&schema)
            .expect_err("open strings are refused");
        assert!(err.domain_text().contains("chiefComplaint"), "{err:?}");
    }

    #[test]
    fn a_date_time_property_is_refused_and_says_so() {
        let schema = serde_json::json!({"type": "object", "properties": {"instances": {
            "type": "array", "items": {"oneOf": [{"type": "object", "properties": {
                "class": {"const": ":Patient"},
                "admittedAt": {"type": "string", "format": "date-time"}}}]}}}});
        let err = bridge()
            .plan_questions(&schema)
            .expect_err("date-times are refused");
        assert!(err.domain_text().contains("date-time"), "{err:?}");
    }

    #[test]
    fn an_ordinal_enum_outside_the_level_range_is_refused() {
        let members: Vec<String> = (0..12).map(|i| format!(":L{i}")).collect();
        let schema = serde_json::json!({"type": "object", "properties": {"instances": {
            "type": "array", "items": {"oneOf": [{"type": "object", "properties": {
                "class": {"const": ":Case"},
                "grade": {"type": "string", "enum": members}}}]}}}});
        let bridge = bridge().with_hint(
            "grade",
            PropertyHint {
                ordinal: true,
                ..PropertyHint::default()
            },
        );
        let err = bridge
            .plan_questions(&schema)
            .expect_err("12 levels exceeds Jev's 10");
        assert!(err.domain_text().contains("grade"), "{err:?}");
        assert!(err.domain_text().contains("10"), "{err:?}");
    }

    #[test]
    fn a_rules_only_property_never_reaches_the_bridge() {
        // Not this module's check — `extract_schema` drops
        // `horis:assignedBy horis:RulesOnly` properties before a Bridge
        // sees them. Pinned here because the VTE example's whole point is
        // that `:recommendation` is absent from what the model is shown,
        // not merely discouraged — so the check runs over the real
        // derivation, and first establishes that the property is there to
        // be dropped at all.
        let onto = vte_ontology();
        assert!(
            onto.node_ref(":recommendation").is_ok(),
            "vte.ttl must declare :recommendation for this test to mean anything"
        );
        let vte_case = onto.node_ref(":VteCase").expect(":VteCase is declared");
        let plan = bridge()
            .plan_questions(&extract_schema(&onto, &[vte_case]))
            .expect("planned");
        assert_eq!(plan.questions.len(), 4, "the four extractable booleans");
        assert!(
            !plan.questions.keys().any(|k| k.contains("recommendation")),
            "questions: {:?}",
            plan.questions.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_multi_class_schema_is_refused_by_name() {
        // Jev answers every question it is asked, so two class branches
        // would come back as one instance of each — including one for a
        // class the note never mentions, entering the World under
        // perception provenance.
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"instances": {"type": "array", "items": {"oneOf": [
                {"type": "object", "properties": {
                    "class": {"const": ":VteCase"},
                    "priorVte": {"type": "boolean"}}},
                {"type": "object", "properties": {
                    "class": {"const": ":Ward"},
                    "isolationRequired": {"type": "boolean"}}},
            ]}}},
        });
        let err = bridge()
            .plan_questions(&schema)
            .expect_err("a multi-class schema must be refused, not answered class by class");
        let BridgeError::Refused { reason, .. } = &err else {
            panic!("expected a refusal naming the classes, got {err:?}");
        };
        assert!(reason.contains(":VteCase"), "{reason}");
        assert!(reason.contains(":Ward"), "{reason}");
        assert!(
            reason.contains("one class per perception step"),
            "the refusal must point at the fix: {reason}"
        );
    }

    #[test]
    fn a_file_only_key_still_configures_its_transport() {
        // A deployment holding its key in a gitignored file sets only the
        // `_FILE` variant; selecting on the bare variable would route it to
        // the other transport and then fail naming a variable it never set.
        let file_only = |name: &str| {
            (name == "OPENROUTER_API_KEY_FILE").then(|| "/run/secrets/openrouter".to_string())
        };
        assert!(JevTransport::OpenRouter.key_configured_in(file_only));
        assert!(
            !JevTransport::TypeSafe.key_configured_in(file_only),
            "nothing names a TypeSafe key"
        );
        assert!(
            JevTransport::TypeSafe
                .key_configured_in(|name| (name == "TYPESAFE_API_KEY").then(|| "k".to_string()))
        );
        assert!(!JevTransport::OpenRouter.key_configured_in(|_| None));
    }

    // -- Answers → results --------------------------------------------------

    #[test]
    fn a_confident_false_is_a_confident_answer() {
        let plan = PropPlan {
            question_id: "i0_priorVte".to_string(),
            name: "priorVte".to_string(),
            kind: PropKind::Noul,
            multi: false,
        };
        let answered = bridge()
            .read_answer(
                &plan,
                &JevAnswer {
                    noul: Some(0.04),
                    choice: None,
                    probabilities: None,
                    confidence: None,
                    score: None,
                    legend: None,
                },
            )
            .expect("a noul answer is readable");
        assert_eq!(answered.value, Value::Bool(false));
        assert!(
            (answered.confidence - 0.96).abs() < 1e-9,
            "p=0.04 is a confident false, got {}",
            answered.confidence
        );
    }

    #[test]
    fn a_score_snaps_to_the_nearest_declared_level() {
        let levels = vec![
            ("Mild".to_string(), ":Mild".to_string()),
            ("Moderate".to_string(), ":Moderate".to_string()),
            ("Severe".to_string(), ":Severe".to_string()),
        ];
        let plan = PropPlan {
            question_id: "i0_severity".to_string(),
            name: "severity".to_string(),
            kind: PropKind::Score(levels),
            multi: false,
        };
        let answered = bridge()
            .read_answer(
                &plan,
                &JevAnswer {
                    noul: None,
                    choice: None,
                    probabilities: None,
                    confidence: Some(0.81),
                    score: Some(1.05),
                    legend: None,
                },
            )
            .expect("a score answer is readable");
        assert_eq!(answered.value, Value::String(":Moderate".to_string()));
        assert_eq!(answered.confidence, 0.81);
    }

    #[test]
    fn an_option_the_bridge_never_offered_is_malformed_output() {
        let plan = PropPlan {
            question_id: "i0_recommendation".to_string(),
            name: "recommendation".to_string(),
            kind: PropKind::Choice(vec![("Hold".to_string(), ":Hold".to_string())]),
            multi: false,
        };
        let err = bridge()
            .read_answer(
                &plan,
                &JevAnswer {
                    noul: None,
                    choice: Some("SomethingElse".to_string()),
                    probabilities: None,
                    confidence: None,
                    score: None,
                    legend: None,
                },
            )
            .expect_err("an invented option must not be accepted");
        assert!(
            matches!(err, BridgeError::MalformedOutput { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn colliding_local_names_fall_back_to_the_full_member() {
        let members = vec![":Acute".to_string(), "other:Acute".to_string()];
        let keys = option_keys(&members);
        assert_eq!(keys[0].0, "Acute");
        assert_eq!(
            keys[1].0, "other:Acute",
            "a collision must not shadow: {keys:?}"
        );
    }

    fn price_basis_of(accounting: &CallAccounting) -> String {
        accounting
            .raw_metadata
            .as_ref()
            .and_then(|m| m.pointer("/price_basis"))
            .and_then(Value::as_str)
            .expect("every call records where its cost came from")
            .to_string()
    }

    /// Cost per case is grouped by `price_basis`, so the three real sources
    /// — the wire's own number, the fallback constant, and that same
    /// constant applied on the transport it was not priced for — each get
    /// their own stable token.
    #[test]
    fn the_price_basis_names_the_source_the_cost_came_from() {
        let response = |cost: Option<f64>| JevResponse {
            model: None,
            answers: BTreeMap::new(),
            usage: Some(JevUsage {
                input_tokens: Some(1_000_000),
                output_tokens: Some(0),
                cost,
            }),
        };
        let elapsed = Duration::from_millis(1);

        let openrouter = JevBridge::new(JevTransport::OpenRouter).accounting(
            &response(None),
            elapsed,
            Map::new(),
        );
        assert_eq!(openrouter.cost_usd, Some(PROMPT_USD_PER_MTOK));
        assert_eq!(
            price_basis_of(&openrouter),
            "openrouter-list-price-estimate"
        );

        let typesafe =
            JevBridge::new(JevTransport::TypeSafe).accounting(&response(None), elapsed, Map::new());
        assert_eq!(typesafe.cost_usd, Some(PROMPT_USD_PER_MTOK));
        assert_eq!(
            price_basis_of(&typesafe),
            "openrouter-list-price-estimate-on-typesafe-transport",
            "the estimate a TypeSafe call carries is not that transport's own price"
        );

        let from_wire = JevBridge::new(JevTransport::TypeSafe).accounting(
            &response(Some(1.6086e-05)),
            elapsed,
            Map::new(),
        );
        assert_eq!(from_wire.cost_usd, Some(1.6086e-05));
        assert_eq!(price_basis_of(&from_wire), "wire-usage-cost");
    }

    /// The transport `from_env` actually selects, driven through the same
    /// injected-environment seam: a key held only in a gitignored file must
    /// route to the transport that file names, not to the other one that
    /// would then fail naming a variable nobody set.
    #[test]
    fn from_env_routes_on_the_file_variant_too() {
        let file_only = |name: &str| {
            (name == "OPENROUTER_API_KEY_FILE").then(|| "/run/secrets/openrouter".to_string())
        };
        let bridge = JevBridge::from_env_in(file_only);
        assert_eq!(bridge.endpoint, JevTransport::OpenRouter.url());
        assert_eq!(bridge.id(), "jev:typesafe/jev-1.13");

        let typesafe_only = |name: &str| (name == "TYPESAFE_API_KEY").then(|| "k".to_string());
        let bridge = JevBridge::from_env_in(typesafe_only);
        assert_eq!(bridge.endpoint, JevTransport::TypeSafe.url());
        assert_eq!(bridge.id(), "jev:jev-1.13");

        let unconfigured = JevBridge::from_env_in(|_| None);
        assert_eq!(
            unconfigured.endpoint,
            JevTransport::TypeSafe.url(),
            "TypeSafe stays the default when nothing is configured"
        );

        let pinned = JevBridge::from_env_in(|name| match name {
            "OPENROUTER_API_KEY" => Some("k".to_string()),
            "JEV_MODEL" => Some("typesafe/jev-1.13-20260917".to_string()),
            _ => None,
        });
        assert_eq!(pinned.id(), "jev:typesafe/jev-1.13-20260917");
        assert_eq!(pinned.endpoint, JevTransport::OpenRouter.url());
    }

    #[test]
    fn id_is_the_configured_alias_before_any_call() {
        assert_eq!(bridge().id(), "jev:typesafe/jev-1.13");
        assert_eq!(
            JevBridge::new(JevTransport::TypeSafe).id(),
            "jev:jev-1.13",
            "the TypeSafe transport routes under the bare alias"
        );
    }

    // -- Wire level ---------------------------------------------------------

    /// One-shot local HTTP listener: reads one whole request, hands its raw
    /// bytes to `assert_request`, and always writes `response_body` back
    /// first so a failed assertion surfaces as an assertion rather than as
    /// a connection reset. Returns the server task's handle as well as its
    /// address — [`join_server`] must be awaited, or every request-shape
    /// assertion below is dead: a panic inside a dropped `JoinHandle` is
    /// swallowed by the runtime and the test passes on the scripted
    /// response alone.
    async fn one_shot_server(
        response_body: &'static str,
        assert_request: impl FnOnce(String) + Send + 'static,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binding a local test listener should not fail");
        let addr = listener
            .local_addr()
            .expect("a bound listener should have a local address");
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener
                .accept()
                .await
                .expect("accepting the one expected connection should not fail");
            let request_text = read_request(&mut stream).await;
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
        (format!("http://{addr}"), handle)
    }

    /// Awaits the server task so an assertion that failed inside it fails
    /// *this* test, with its own message.
    async fn join_server(handle: tokio::task::JoinHandle<()>) {
        if let Err(err) = handle.await {
            match err.try_into_panic() {
                Ok(panic) => std::panic::resume_unwind(panic),
                Err(err) => panic!("the mock server task did not finish: {err}"),
            }
        }
    }

    /// Reads a whole HTTP request: headers, then exactly `Content-Length`
    /// body bytes. A single `read` would truncate a body split across TCP
    /// segments and turn a real request into unparseable JSON.
    async fn read_request(stream: &mut tokio::net::TcpStream) -> String {
        let mut raw: Vec<u8> = Vec::new();
        let mut chunk = vec![0_u8; 4096];
        loop {
            let n = stream
                .read(&mut chunk)
                .await
                .expect("reading the request should not fail");
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&chunk[..n]);
            if request_len(&raw).is_some_and(|total| raw.len() >= total) {
                break;
            }
        }
        String::from_utf8_lossy(&raw).to_string()
    }

    /// The full byte length of the request in `raw`, once its headers have
    /// arrived; `None` while they have not.
    fn request_len(raw: &[u8]) -> Option<usize> {
        let head_end = raw.windows(4).position(|w| w == b"\r\n\r\n")? + 4;
        let head = String::from_utf8_lossy(&raw[..head_end]);
        let body_len = head
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
            .and_then(|(_, value)| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        Some(head_end + body_len)
    }

    fn request_body(request_text: &str) -> Value {
        let body = request_text.split("\r\n\r\n").nth(1).unwrap_or_default();
        serde_json::from_str(body).expect("request body should be valid JSON")
    }

    /// A bridge pointed at a local mock listener, with a directly supplied
    /// key — no environment mutation (`unsafe_code = "deny"`, and a shared
    /// process environment is not safe to mutate from parallel tests).
    fn bridge_at(base_url: String) -> JevBridge {
        let mut bridge = JevBridge::new(JevTransport::OpenRouter).with_api_key("test-key");
        bridge.endpoint = base_url;
        bridge
    }

    /// One ordinal property over three declared levels, with the
    /// deployment-side hint that makes it a `score` rather than a `choice`.
    fn severity_schema() -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {"instances": {"type": "array", "items": {"oneOf": [{
                "type": "object",
                "properties": {
                    "class": {"const": ":Case"},
                    "severity": {"type": "string", "enum": [":Mild", ":Moderate", ":Severe"]},
                },
            }]}}},
        })
    }

    fn ordinal_bridge_at(base_url: String) -> JevBridge {
        bridge_at(base_url).with_hint(
            "severity",
            PropertyHint {
                ordinal: true,
                ..PropertyHint::default()
            },
        )
    }

    const SCRIPTED_VTE_RESPONSE: &str = r#"{
        "model": "typesafe/jev-1.13-20260917",
        "answers": {
            "i0_reducedMobility": {"type": "noul", "noul": 0.96},
            "i0_activeCancer":    {"type": "noul", "noul": 0.12},
            "i0_priorVte":        {"type": "noul", "noul": 0.04},
            "i0_bleedingRisk":    {"type": "noul", "noul": 0.07}
        },
        "usage": {"input_tokens": 619, "output_tokens": 79}
    }"#;

    #[tokio::test]
    async fn perceive_posts_the_decisions_body_and_maps_the_answers_back() {
        let (base_url, server) = one_shot_server(SCRIPTED_VTE_RESPONSE, |request_text| {
            let body = request_body(&request_text);
            assert_eq!(
                body.get("model"),
                Some(&Value::String("typesafe/jev-1.13".to_string())),
                "request did not carry the model alias:\n{body}"
            );
            assert!(
                body.get("state")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.contains("fall")),
                "request did not carry the clinical note as `state`:\n{body}"
            );
            let questions = body
                .get("questions")
                .and_then(Value::as_object)
                .expect("request carries a questions object");
            assert_eq!(questions.len(), 4, "expected four questions:\n{body}");
            for (id, q) in questions {
                assert_eq!(
                    q.get("type"),
                    Some(&Value::String("noul".to_string())),
                    "{id}"
                );
                assert!(
                    q.pointer("/criteria/true").is_some(),
                    "{id} lacks criteria.true"
                );
                assert!(
                    q.pointer("/criteria/false").is_some(),
                    "{id} lacks criteria.false"
                );
            }
            let head = request_text
                .split("\r\n\r\n")
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            assert!(
                head.contains("authorization: bearer test-key"),
                "the supplied bearer token must reach the wire:\n{request_text}"
            );
            assert!(
                !request_text.contains("chat/completions"),
                "this bridge must post to the Decisions endpoint:\n{request_text}"
            );
        })
        .await;

        let bridge = bridge_at(base_url);
        let result = bridge
            .perceive(PerceiveRequest {
                text: "72-year-old admitted following a fall, mobilising only with assistance."
                    .to_string(),
                schema: vte_schema(),
            })
            .await
            .expect("perceive should succeed against the scripted response");
        join_server(server).await;

        assert_eq!(
            result.instances,
            vec![serde_json::json!({
                "class": ":VteCase",
                "activeCancer": false,
                "bleedingRisk": false,
                "priorVte": false,
                "reducedMobility": true,
            })]
        );
        // The weakest answer in the instance (activeCancer, p=0.12 → a
        // false held at 0.88) is what both confidences report.
        assert_eq!(result.fact_confidence.get(&0).copied(), Some(0.88));
        assert!(
            (result.confidence - 0.88).abs() < 1e-9,
            "{}",
            result.confidence
        );

        assert_eq!(result.accounting.prompt_tokens, Some(619));
        assert_eq!(result.accounting.completion_tokens, Some(79));
        assert_eq!(
            result.accounting.model.as_deref(),
            Some("typesafe/jev-1.13-20260917")
        );
        assert!(result.accounting.latency_ms.is_some());
        // This scripted `usage` carries no `cost`, so the estimate falls
        // back to the constant — and says so.
        let cost = result
            .accounting
            .cost_usd
            .expect("cost is estimated from usage");
        assert!((cost - 619.0 * 0.042 / 1e6).abs() < 1e-12, "{cost}");
        assert!(
            result
                .accounting
                .raw_metadata
                .as_ref()
                .and_then(|m| m.pointer("/price_basis"))
                .and_then(Value::as_str)
                == Some("openrouter-list-price-estimate"),
            "a fallback estimate must name itself as one: {:?}",
            result.accounting.raw_metadata
        );

        // Per-question probabilities survive into the audit escape hatch.
        let per_property = result
            .accounting
            .raw_metadata
            .as_ref()
            .and_then(|m| m.pointer("/fact_confidence_by_property/0"))
            .cloned()
            .expect("per-property confidences are recorded");
        assert_eq!(
            per_property
                .pointer("/reducedMobility")
                .and_then(Value::as_f64),
            Some(0.96)
        );
        assert_eq!(
            per_property.pointer("/priorVte").and_then(Value::as_f64),
            Some(0.96),
            "a p=0.04 noul is a 0.96-confident false: {per_property}"
        );

        // `id()` reports the model actually invoked, not the alias asked for.
        assert_eq!(bridge.id(), "jev:typesafe/jev-1.13-20260917");
    }

    #[tokio::test]
    async fn rank_asks_one_closed_choice_over_the_runtime_s_candidates() {
        let response = r#"{
            "model": "typesafe/jev-1.13-20260917",
            "answers": {"selection": {"type": "choice", "choice": "c1", "confidence": 0.81,
                                      "probabilities": {"c0": 0.19, "c1": 0.81}}},
            "usage": {"input_tokens": 88, "output_tokens": 4}
        }"#;
        let (base_url, server) = one_shot_server(response, |request_text| {
            let body = request_body(&request_text);
            let criteria = body
                .pointer("/questions/selection/criteria")
                .and_then(Value::as_object)
                .expect("rank asks one choice question");
            assert_eq!(criteria.len(), 2, "one option per candidate:\n{body}");
            assert_eq!(
                body.pointer("/questions/selection/type"),
                Some(&Value::String("choice".to_string()))
            );
        })
        .await;
        let bridge = bridge_at(base_url);
        let result = bridge
            .rank(RankRequest {
                context: "cheapest bed that still meets the isolation requirement".to_string(),
                candidates: vec![
                    serde_json::json!({"bed": "A1"}),
                    serde_json::json!({"bed": "B2"}),
                ],
            })
            .await
            .expect("rank should succeed");
        join_server(server).await;
        assert_eq!(result.chosen, 1);
        assert_eq!(result.confidence, 0.81);
        assert!(
            result
                .accounting
                .raw_metadata
                .as_ref()
                .and_then(|m| m.pointer("/price_basis"))
                .and_then(Value::as_str)
                == Some("openrouter-list-price-estimate"),
            "a cost the wire did not report must name the list it came from: {:?}",
            result.accounting.raw_metadata
        );
    }

    /// The legend, not the index, decides which declared level a continuous
    /// score names. The measured wire is 0-based, on which both paths agree
    /// — so this case is deliberately offset (a 1-based legend) to tell
    /// them apart: an index read would answer `:Moderate` here.
    #[tokio::test]
    async fn a_score_is_resolved_through_the_response_s_legend() {
        let response = r#"{
            "model": "typesafe/jev-1.13-20260917",
            "answers": {"i0_severity": {"type": "score", "score": 1.05, "confidence": 0.81,
                                        "legend": {"1": "Mild", "2": "Moderate", "3": "Severe"}}},
            "usage": {"input_tokens": 42, "output_tokens": 3}
        }"#;
        let (base_url, server) = one_shot_server(response, |request_text| {
            let body = request_body(&request_text);
            assert_eq!(
                body.pointer("/questions/i0_severity/type"),
                Some(&Value::String("score".to_string())),
                "an ordinal-hinted enum is asked as a score:\n{body}"
            );
            assert_eq!(
                body.pointer("/questions/i0_severity/criteria"),
                Some(&serde_json::json!(["Mild", "Moderate", "Severe"])),
                "score levels must reach the wire in declared order:\n{body}"
            );
        })
        .await;
        let result = ordinal_bridge_at(base_url)
            .perceive(PerceiveRequest {
                text: "mild tenderness only".to_string(),
                schema: severity_schema(),
            })
            .await
            .expect("a score answer is readable");
        join_server(server).await;
        assert_eq!(
            result.instances,
            vec![serde_json::json!({"class": ":Case", "severity": ":Mild"})],
            "the legend puts 1.05 at its own level 1, which is `:Mild`"
        );
        assert_eq!(result.confidence, 0.81);
    }

    /// Without a legend there is nothing to read the scale off, so the
    /// level rounds to the nearest declared index — the documented
    /// fallback, and the reason the legend is preferred when present.
    #[tokio::test]
    async fn a_score_without_a_legend_falls_back_to_the_declared_index() {
        let response = r#"{
            "model": "typesafe/jev-1.13-20260917",
            "answers": {"i0_severity": {"type": "score", "score": 1.05, "confidence": 0.81}},
            "usage": {"input_tokens": 42, "output_tokens": 3}
        }"#;
        let (base_url, server) = one_shot_server(response, |_| {}).await;
        let result = ordinal_bridge_at(base_url)
            .perceive(PerceiveRequest {
                text: "moderate pain".to_string(),
                schema: severity_schema(),
            })
            .await
            .expect("a score answer is readable without a legend");
        join_server(server).await;
        assert_eq!(
            result.instances,
            vec![serde_json::json!({"class": ":Case", "severity": ":Moderate"})]
        );
    }

    /// The shape the live probe actually measured on 2026-09-21: a 0-based
    /// legend, `probabilities` keyed by the same level strings, no
    /// `confidence` field of its own, and a `usage.cost` the endpoint
    /// reports directly. The confidence comes from the answered level's
    /// probability rather than normalizing to 0.0, exactly as the `choice`
    /// path does, and the cost is the wire's number, not the constant.
    #[tokio::test]
    async fn a_measured_score_answer_reads_its_probability_and_the_wire_s_cost() {
        let response = r#"{
            "model": "typesafe/jev-1.13-20260917",
            "answers": {"i0_severity": {"type": "score", "score": 1.99,
                                        "legend": {"0": "Mild", "1": "Moderate", "2": "Severe"},
                                        "probabilities": {"0": 0, "1": 0.01, "2": 0.99}}},
            "usage": {"input_tokens": 383, "output_tokens": 32, "cost": 1.6086e-05}
        }"#;
        let (base_url, server) = one_shot_server(response, |_| {}).await;
        let result = ordinal_bridge_at(base_url)
            .perceive(PerceiveRequest {
                text: "severe, unremitting".to_string(),
                schema: severity_schema(),
            })
            .await
            .expect("a score answer is readable");
        join_server(server).await;
        assert_eq!(
            result.instances,
            vec![serde_json::json!({"class": ":Case", "severity": ":Severe"})],
            "1.99 on the measured 0-based legend is its level 2"
        );
        assert_eq!(
            result.confidence, 0.99,
            "an omitted confidence falls back to the answered level's probability"
        );
        assert_eq!(result.accounting.cost_usd, Some(1.6086e-05));
        assert_eq!(
            result
                .accounting
                .raw_metadata
                .as_ref()
                .and_then(|m| m.pointer("/price_basis"))
                .and_then(Value::as_str),
            Some("wire-usage-cost")
        );
    }

    /// A legend this bridge cannot read as `{numeric level: option key}` —
    /// here the inverse orientation, which is deliberately not guessed at —
    /// must not fail an otherwise answerable perception: the level falls
    /// back to the declared index, and the trace records that it did.
    #[tokio::test]
    async fn an_unreadable_legend_falls_back_to_the_index_and_records_it() {
        let response = r#"{
            "model": "typesafe/jev-1.13-20260917",
            "answers": {"i0_severity": {"type": "score", "score": 1.05, "confidence": 0.7,
                                        "legend": {"Mild": 1, "Moderate": 2, "Severe": 3}}},
            "usage": {"input_tokens": 42, "output_tokens": 3}
        }"#;
        let (base_url, server) = one_shot_server(response, |_| {}).await;
        let result = ordinal_bridge_at(base_url)
            .perceive(PerceiveRequest {
                text: "moderate pain".to_string(),
                schema: severity_schema(),
            })
            .await
            .expect("an unreadable legend must not fail the whole perception");
        join_server(server).await;
        assert_eq!(
            result.instances,
            vec![serde_json::json!({"class": ":Case", "severity": ":Moderate"})],
            "the index fallback answers 1.05 as the declared level 1"
        );
        assert_eq!(
            result
                .accounting
                .raw_metadata
                .as_ref()
                .and_then(|m| m.pointer("/unreadable_score_legends"))
                .cloned(),
            Some(serde_json::json!(["i0_severity"])),
            "the trace must show the level did not come from the legend: {:?}",
            result.accounting.raw_metadata
        );
    }

    /// A legend only *partly* shaped `{numeric: string}` is not trusted for
    /// the entries that did parse: resolving from the survivors would name
    /// a level the answer did not (here level 1 drops out, leaving 0 and 2
    /// equidistant from a score of 1.0). The whole legend is set aside for
    /// the index, and the trace says so.
    #[tokio::test]
    async fn a_partly_readable_legend_is_not_trusted_at_all() {
        let response = r#"{
            "model": "typesafe/jev-1.13-20260917",
            "answers": {"i0_severity": {"type": "score", "score": 1.0, "confidence": 0.7,
                                        "legend": {"0": "Mild", "1": {"label": "Moderate"},
                                                   "2": "Severe"}}},
            "usage": {"input_tokens": 42, "output_tokens": 3}
        }"#;
        let (base_url, server) = one_shot_server(response, |_| {}).await;
        let result = ordinal_bridge_at(base_url)
            .perceive(PerceiveRequest {
                text: "moderate pain".to_string(),
                schema: severity_schema(),
            })
            .await
            .expect("a partly readable legend must not fail the whole perception");
        join_server(server).await;
        assert_eq!(
            result.instances,
            vec![serde_json::json!({"class": ":Case", "severity": ":Moderate"})],
            "the index answers 1.0 as the declared level 1, which the legend subset would not"
        );
        assert_eq!(
            result
                .accounting
                .raw_metadata
                .as_ref()
                .and_then(|m| m.pointer("/unreadable_score_legends"))
                .cloned(),
            Some(serde_json::json!(["i0_severity"])),
            "a legend set aside must be visible in the trace: {:?}",
            result.accounting.raw_metadata
        );
    }

    /// The two paths must not disagree about a midpoint: `f64::round` takes
    /// 1.5 up to level 2, so the legend's equidistant tie goes the same way.
    #[tokio::test]
    async fn the_legend_and_the_index_agree_at_an_exact_midpoint() {
        let with_legend = r#"{
            "model": "typesafe/jev-1.13-20260917",
            "answers": {"i0_severity": {"type": "score", "score": 1.5, "confidence": 0.6,
                                        "legend": {"0": "Mild", "1": "Moderate",
                                                   "2": "Severe"}}}
        }"#;
        let (base_url, server) = one_shot_server(with_legend, |_| {}).await;
        let from_legend = ordinal_bridge_at(base_url)
            .perceive(PerceiveRequest {
                text: "borderline".to_string(),
                schema: severity_schema(),
            })
            .await
            .expect("a midpoint score is readable");
        join_server(server).await;

        let without_legend = r#"{
            "model": "typesafe/jev-1.13-20260917",
            "answers": {"i0_severity": {"type": "score", "score": 1.5, "confidence": 0.6}}
        }"#;
        let (base_url, server) = one_shot_server(without_legend, |_| {}).await;
        let from_index = ordinal_bridge_at(base_url)
            .perceive(PerceiveRequest {
                text: "borderline".to_string(),
                schema: severity_schema(),
            })
            .await
            .expect("a midpoint score is readable");
        join_server(server).await;

        assert_eq!(
            from_legend.instances, from_index.instances,
            "the legend and the index must resolve a midpoint the same way"
        );
        assert_eq!(
            from_index.instances,
            vec![serde_json::json!({"class": ":Case", "severity": ":Severe"})],
            "1.5 rounds up, as `f64::round` does"
        );
    }

    /// A legend naming an option the bridge never offered is malformed
    /// output — the same treatment an invented `choice` key gets, rather
    /// than a guess at which level was meant.
    #[tokio::test]
    async fn a_legend_naming_an_unoffered_level_is_malformed_output() {
        let response = r#"{
            "model": "typesafe/jev-1.13-20260917",
            "answers": {"i0_severity": {"type": "score", "score": 1.05,
                                        "legend": {"1": "Trivial", "2": "Catastrophic"}}}
        }"#;
        let (base_url, server) = one_shot_server(response, |_| {}).await;
        let err = ordinal_bridge_at(base_url)
            .perceive(PerceiveRequest {
                text: "mild tenderness only".to_string(),
                schema: severity_schema(),
            })
            .await
            .expect_err("a level the bridge never offered must not be accepted");
        join_server(server).await;
        assert!(
            matches!(err, BridgeError::MalformedOutput { .. }),
            "{err:?}"
        );
        assert!(err.domain_text().contains("Trivial"), "{err:?}");
    }

    #[tokio::test]
    async fn narrate_refuses_without_making_a_call() {
        // No listener is started: a call would fail with a connection error
        // rather than a refusal, so reaching `Refused` proves nothing was
        // sent.
        let bridge = JevBridge::new(JevTransport::OpenRouter);
        let err = bridge
            .narrate(NarrateRequest {
                proof_text: ":recommendation := :PharmacologicProphylaxis".to_string(),
                audience: "charge-nurse".to_string(),
            })
            .await
            .expect_err("Jev has no text output channel");
        let BridgeError::Refused { reason, .. } = &err else {
            panic!("expected a refusal, got {err:?}");
        };
        assert!(
            reason.contains("no text output channel"),
            "the refusal must explain why: {reason}"
        );
        assert!(
            reason.contains("narration step"),
            "the refusal must point at the fix: {reason}"
        );
    }

    #[tokio::test]
    async fn a_non_2xx_response_is_a_request_failure_carrying_the_body() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binding a local test listener should not fail");
        let addr = listener
            .local_addr()
            .expect("bound listener has an address");
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("one connection");
            let mut buf = vec![0_u8; 65536];
            let _ = stream.read(&mut buf).await.expect("read the request");
            let body = r#"{"error":{"message":"unknown question type"}}"#;
            let response = format!(
                "HTTP/1.1 400 Bad Request\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await.expect("write");
            stream.flush().await.expect("flush");
        });
        let bridge = bridge_at(format!("http://{addr}"));
        let err = bridge
            .perceive(PerceiveRequest {
                text: "note".to_string(),
                schema: vte_schema(),
            })
            .await
            .expect_err("a 400 must not be read as answers");
        let BridgeError::RequestFailed { reason, .. } = &err else {
            panic!("expected a request failure, got {err:?}");
        };
        assert!(reason.contains("400"), "{reason}");
        assert!(reason.contains("unknown question type"), "{reason}");
    }

    #[tokio::test]
    async fn an_unanswered_question_is_malformed_output() {
        let response = r#"{"model": "typesafe/jev-1.13", "answers": {
            "i0_reducedMobility": {"type": "noul", "noul": 0.9}}}"#;
        let (base_url, server) = one_shot_server(response, |_| {}).await;
        let bridge = bridge_at(base_url);
        let err = bridge
            .perceive(PerceiveRequest {
                text: "note".to_string(),
                schema: vte_schema(),
            })
            .await
            .expect_err("three of four questions went unanswered");
        join_server(server).await;
        assert!(
            matches!(err, BridgeError::MalformedOutput { .. }),
            "{err:?}"
        );
    }

    /// The whole claim, end to end over the HTTP path a live deployment
    /// takes: a clinical note is posted to the Decisions endpoint, Jev's
    /// typed answers come back, they clear the World's validation gate as
    /// `Provenance::Perception` — and `:recommendation` is derived by the
    /// **rules**. The model is never shown that property (it is
    /// `horis:assignedBy horis:RulesOnly`), so it cannot propose one; the
    /// two notes below differ only in the facts Jev proposed, and fire two
    /// different rules on them.
    #[tokio::test]
    async fn perceived_facts_clear_the_gate_and_the_rules_decide() {
        // Same shape as `SCRIPTED_VTE_RESPONSE`, for a note that says the
        // patient is already anticoagulated: `bleedingRisk` comes back true
        // and the higher-priority rule wins.
        const BLEEDING_RESPONSE: &str = r#"{
            "model": "typesafe/jev-1.13-20260917",
            "answers": {
                "i0_reducedMobility": {"type": "noul", "noul": 0.94},
                "i0_activeCancer":    {"type": "noul", "noul": 0.09},
                "i0_priorVte":        {"type": "noul", "noul": 0.28},
                "i0_bleedingRisk":    {"type": "noul", "noul": 0.61}
            },
            "usage": {"input_tokens": 58, "output_tokens": 79}
        }"#;

        let onto = vte_ontology();
        let vte_case = onto.node_ref(":VteCase").expect(":VteCase is declared");
        let recommendation = onto
            .node_ref(":recommendation")
            .expect(":recommendation is declared");
        let schema = extract_schema(&onto, &[vte_case]);

        for (note, response, expected_rule, expected_decision) in [
            (
                "72-year-old admitted following a fall, mobilising only with assistance, \
                 not anticoagulated.",
                SCRIPTED_VTE_RESPONSE,
                ":risk-without-contraindication",
                ":PharmacologicProphylaxis",
            ),
            (
                "78yo, post-op day 2, on warfarin, bed-bound",
                BLEEDING_RESPONSE,
                ":bleeding-contraindication",
                ":HoldProphylaxis",
            ),
        ] {
            let (base_url, server) = one_shot_server(response, |_| {}).await;
            let result = bridge_at(base_url)
                .perceive(PerceiveRequest {
                    text: note.to_string(),
                    schema: schema.clone(),
                })
                .await
                .expect("perceive should succeed against the scripted response");
            join_server(server).await;

            let instance = result
                .instances
                .first()
                .expect("the schema declares one class, so one instance comes back");
            assert!(
                instance.get("recommendation").is_none(),
                "a RulesOnly property must never come back from the model: {instance}"
            );

            let mut properties = Vec::new();
            for (name, value) in instance
                .as_object()
                .expect("a proposed instance is an object")
            {
                if name == "class" {
                    continue;
                }
                let property = onto
                    .node_ref(&format!(":{name}"))
                    .expect("vte.ttl declares every proposed property");
                let held = value.as_bool().expect("vte.ttl declares them all boolean");
                properties.push((property, TypedValue::Boolean(held)));
            }
            let mut world = World::new(&onto);
            let case = world
                .assert_instance_as(Provenance::Perception, None, vte_case, properties)
                .expect("the proposed facts must clear the validation gate");

            let decided = TypedValue::Ref(Ref::new(
                onto.node_ref(expected_decision)
                    .expect("a declared :VteRecommendation member"),
            ));
            assert_eq!(
                world.value(case, recommendation),
                Some(&decided),
                "note {note:?} should decide {expected_decision}"
            );
            assert_eq!(
                world.fact_provenance(case, recommendation, &decided),
                Some(Provenance::Rule),
                "the decision is the rules', not the model's"
            );

            let proof = world
                .explain(case, recommendation)
                .expect("a derived fact carries its proof");
            match &proof.kind {
                ProofKind::Conclusion { rule, .. } => assert_eq!(
                    *rule,
                    onto.node_ref(expected_rule).expect("a declared rule"),
                    "note {note:?} fired the wrong rule"
                ),
                other => panic!("a derivation's root is a conclusion, got {other:?}"),
            }
            assert!(
                proof.render(&world).contains(expected_decision),
                "the proof must read as the decision it made:\n{}",
                proof.render(&world)
            );
        }
    }

    /// Run by hand against the live endpoint:
    /// `OPENROUTER_API_KEY=sk-or-... cargo test -p horismos-bridges --features
    /// bridges-jev -- --ignored live_perceive_smoke`.
    #[tokio::test]
    #[ignore = "needs a live OPENROUTER_API_KEY; not run in CI"]
    async fn live_perceive_smoke() {
        let bridge = JevBridge::from_env();
        let result = bridge
            .perceive(PerceiveRequest {
                text: "Bed-bound since admission. No previous clots. Not anticoagulated."
                    .to_string(),
                schema: vte_schema(),
            })
            .await
            .expect("live perceive should succeed with a real key");
        assert_eq!(result.instances.len(), 1);
        assert!(result.accounting.model.is_some());
    }
}
