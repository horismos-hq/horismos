# 08 · The Bridge

The Bridge is Sigil's only connection to model providers: a deliberately thin client
layer beneath the LLM boundary steps (doc 06). It is a port of the neural-bridge
design proven in sigil-lang (`ModelBackend` protocol + `ModelRegistry`, 76 tests),
with the same philosophy: **Sigil's runtime is the loop; the Bridge is a model
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

The ontology names a model (`sigil:model "claude:claude-fable-5"`); deployment maps
names to configured Bridge instances. Swapping providers is a config change; the
ontology is provider-agnostic. **[DECIDE: allow deployment-level model override of
ontology-declared models (lean yes, with trace annotation) — PRP-004]**

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
output modes use those instead; the Bridge hides the difference. Confidence derivation
is provider-specific (logprobs where available, calibrated self-report otherwise) and
documented per backend. **[DECIDE: confidence calibration strategy per provider —
PRP-004; the mock backend makes tests independent of this decision]**

## Operational semantics

- **Timeouts**: per-call, from the ontology (`sigil:timeout` on the step) or Bridge
  default; enforced by the runtime (sigil-lang precedent: thread/task-based, portable).
- **Fallback on any Bridge error** (timeout, rate limit, refusal, malformed output):
  identical to low confidence — the step's `sigil:fallback` edge. There is no
  "retry until it works" mode.
- **Accounting**: every call records model, latency, token counts, cost estimate in
  the trace. Per-run and per-deployment budgets are queryable; a `sigil:budget` on
  the workflow makes exceeding it a fallback-triggering condition.
  **[DECIDE: budget semantics v1 or defer — PRP-004]**
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
