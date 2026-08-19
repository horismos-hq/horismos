# 08 · The Bridge

The Bridge is Horismos's only connection to model providers: a deliberately thin client
layer beneath the LLM boundary steps (doc 06). It is a port of the neural-bridge
design proven in sigil-lang (`ModelBackend` protocol + `ModelRegistry`, 76 tests),
with the same philosophy: **Horismos's runtime is the loop; the Bridge is a model
client, not another orchestrator.** No LangChain, no delegated agent SDKs.

## The contract

```rust
// Rust
#[async_trait]
pub trait Bridge: Send + Sync {
    async fn perceive(&self, req: PerceiveRequest) -> Result<PerceiveResult, BridgeError>;
    async fn narrate(&self, req: NarrateRequest) -> Result<NarrateResult, BridgeError>;
    async fn rank(&self, req: RankRequest) -> Result<RankResult, BridgeError>;  // ProposalStep
    fn id(&self) -> &str;
}
```

```python
# Python — structural protocol, same three capabilities
class Bridge(Protocol):
    def perceive(self, req: PerceiveRequest) -> PerceiveResult: ...
    def narrate(self, req: NarrateRequest) -> NarrateResult: ...
    def rank(self, req: RankRequest) -> RankResult: ...
```

`PerceiveResult` carries: proposed instances (JSON matching the derived schema),
call confidence, optional per-fact confidences, token/latency accounting.
The runtime — not the Bridge — owns thresholds, retries, fallbacks, validation.
Bridges stay dumb on purpose.

## Registry and routing

Model identifiers in the ontology are prefix-routed, exactly as in sigil-lang:

| Prefix | Backend | Notes |
|--------|---------|-------|
| `claude:` | Anthropic Messages API | default recommendation |
| `openai:` | OpenAI Responses/Chat | |
| `hf:` | Local HuggingFace pipeline | classification-style perception without a frontier model |
| `onnx:` | ONNX Runtime | edge/embedded |
| `python:` | Any Python callable | escape hatch, custom models |
| `mock:` | Deterministic mock | **tests and CI run on this — zero API keys** |

```python
agent = Agent(world, workflow="triage:Triage",
              bridges={"claude:claude-fable-5": AnthropicBridge.from_env()})
```

The ontology names a model (`horis:model "claude:claude-fable-5"`); deployment maps
names to configured Bridge instances. Swapping providers is a config change; the
ontology is provider-agnostic. Deployment may override an ontology-declared model
with a different configured Bridge for the same prefix (swap in a cheaper `claude:`
model, or `mock:` in tests); every trace step's `model` field records the Bridge
actually invoked (`Bridge::id()`), never merely the ontology's declared string, so
an override stays visible after the fact without a dedicated trace field.

## Provider mechanics (perception)

One call per PerceptionStep. For Anthropic:

```
messages.create(
  model    = "claude-fable-5",
  tools    = [{ name: "assert_facts", input_schema: <derived from ontology> }],
  tool_choice = { type: "tool", name: "assert_facts" },     # forced
  messages = [{ role: "user", content: <input text> }])
```

The provider's tool-use machinery is used *backwards* — not to let the model choose
actions, but as a constrained typed-output channel. Providers with native structured
output modes use those instead; the Bridge hides the difference. v1 ships one
calibration strategy for every provider: the forced tool call's schema carries a
`confidence` number field the model must self-report, and a result that omits it is
treated as `0.0` — failing safe into the fallback edge rather than false-accepting.
Logprob-derived confidence, where a provider exposes it, is a future per-backend
enhancement layered on the same field; the mock backend's scripted confidence makes
every test independent of either.

## Operational semantics

- **Timeouts**: per-call, from the ontology (`horis:timeout` on the step) or Bridge
  default; enforced by the runtime (sigil-lang precedent: thread/task-based, portable).
- **Fallback on any Bridge error** (timeout, rate limit, refusal, malformed output):
  identical to low confidence — the step's `horis:fallback` edge. There is no
  "retry until it works" mode.
- **Accounting**: every call records model, latency, token counts, cost estimate in
  the trace — this ships in v1. `horis:budget` as a workflow-level threshold that
  turns exceeding it into a fallback-triggering condition is deferred past v1: the
  accounting the feature would consume already exists, so adding the threshold
  check later is additive, not a redesign.
- **No streaming** in v1: perception outputs are atomic typed payloads; narration is
  short. Revisit only with evidence.

## The mock backend

`mock:` is a first-class citizen, not a test hack:

```python
bridge = MockBridge(script={
    "58M, crushing chest pain": PerceiveResult(
        instances=[...], confidence=0.94),
    "*": PerceiveResult.low_confidence(),
})
```

Scriptable by input pattern, deterministic, records invocations. The entire test
suite, all examples in CI, and the τ-Bench harness scaffold run against `mock:` —
which is also the enforcement mechanism for the anti-wrapper guarantee: if any
symbolic-core test *needs* a real key, that is a build failure by policy (doc 14).
