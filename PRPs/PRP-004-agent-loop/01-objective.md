# PRP-004 — Ontology-First Agent Loop

**Status**: Active (2026-08-18)
**Depends on**: PRP-001, PRP-002, PRP-003 (claim-only — see *Coordination with PRP-003* below)

This PRP is **teaching-grade** (`CONVENTIONS.md` § Teaching-grade PRPs): the five spec
sections below are the phase contract; around them sit background for a reader who
knows Python but is new to Rust trait objects and to the neuro-symbolic split, worked
examples against `examples/triage/triage.ttl`, Python-parallel notes, and a glossary.
Teaching sections cite `docs/framework/05-workflows.md` through `08-bridge.md` rather
than restating them.

## Pivot note (2026-08-24) — bounded projection kernel

Captain-adopted pivot (`horismos-thin-pivot-s1/report.md`, PRP-004 row; PRP-000 §"Snapshot-
scoped guarantees"): this PRP's current branch, scope, and DoD below are **unchanged and
finish first** — the report explicitly recommends landing the current scenario/runtime work
without dragging standards integration into the active branch. The durable description
drops one claim going forward: Horismos does **not** require that "the ontology must own the
outer loop." The workflow/tool/Bridge kernel this PRP ships is reusable as a **decision
microflow** a host invokes (e.g. via a CDS-Hooks-style call, or one PlanDefinition-subset
fragment) rather than the framework owning a deployment's entire longitudinal workflow.
Host-invoked/stateless execution, adapter bindings for external commits, and aligning
`ActionStep` effects with conditional external commits are **owned by PRP-008** (activated
after this PRP and PRP-005) — not built here, not retrofitted mid-branch.

## Intent

Execute workflows declared in the ontology. Perception (NL → typed instances) and
narration (proof → NL) become the only two places a model touches a Horismos run;
every step transition, precondition check, tool invocation, and conflict resolution
stays symbolic. After this PRP, `docs/framework/14-architecture.md`'s "one perception
transaction" data-flow diagram is real code, and "the loop is ontology data" stops
being an aspiration.

## Background — the ideas this PRP assumes

### What a transducer boundary actually is

A *transducer* converts between representations without adding judgment: a JPEG
decoder is a transducer (pixels in, pixels out — no decision about what the image
*means*). Horismos declares exactly two neural transducer boundaries, and nowhere
else does a model touch a run (doc 06):

- **Perception**: natural language → typed instances. The model's job is narrowly
  "extract these classes from this text," not "extract, then decide what matters" —
  the runtime, not the model, decides what happens with the extracted facts.
- **Narration**: a proof tree → prose. The model's job is "render this for a
  charge nurse," not "explain what you think happened" — it renders a symbolic
  artifact it did not produce and cannot alter.

> **Python parallel.** This is the same discipline as keeping a template-rendering
> function pure: `render(context) -> html` must never *decide* what's in `context`,
> only how it looks. Perception and narration are Horismos's two blessed render
> functions, one in each direction, and the framework is designed so nothing else
> needs one.

### Why the runtime fires tools, never the model

Conventional agent frameworks hand the model a list of tools and a system prompt and
let it choose. Horismos inverts this (doc 07): a `horis:Tool` is a **contract**
(`horis:takes`/`horis:effect`/`horis:binding`), and an `ActionStep` fires it exactly
when the current step is `:Route`-like and its `horis:requires` pattern already holds
in the graph — a plain boolean check over facts that are *already there*, most of
which chaining derived without any model in the loop. The model never sees the tool
list, never constructs an argument, never decides "now would be a good time to page
the nurse." This closes an entire class of conventional-agent failure modes (doc 07's
table: wrong tool called, hallucinated args, tool fired before its preconditions —
all become **impossible**, not merely unlikely, because there is no step where a model
makes any of those choices) at the cost of authoring more of the domain up front, in
the ontology, rather than in a prompt.

> **Python parallel.** Think of the difference between `eval(model_output)` (the
> model's text becomes code that runs) and a `match` statement over a closed enum the
> model can only *select from a pre-computed, already-safe list*. Horismos's
> `ActionStep` is the second shape, always: candidates and preconditions are computed
> symbolically first, and the neural step (when there is one at all — `ProposalStep`,
> below) only ranks what already passed.

### The Bridge: a client, not another orchestrator

`docs/framework/08-bridge.md` is explicit that Horismos does not want a LangChain
inside itself: the `Bridge` trait is three methods (`perceive`/`narrate`/`rank`) and
nothing else — no memory, no planning, no retries-until-it-works. Retries, thresholds,
fallbacks, and validation are the **runtime's** job, sitting outside every `Bridge`
implementation, so swapping providers is a client swap, never an architecture change.

> **Python parallel.** In Rust, "a pure interface with several implementations
> chosen at runtime" is `dyn Bridge` behind a `Box` or `Arc` — the same shape as a
> Python `Protocol` (or ABC) with several concrete classes, selected by whatever
> constructs the caller. `#[async_trait]` exists because Rust's native `async fn` in
> traits doesn't yet support the dynamic dispatch (`dyn Bridge`) a runtime registry
> needs; it desugars each async method into one returning a boxed future — mechanical
> plumbing, not a different design.

### Anti-wrapper, mechanically

`horismos-core` (where the workflow interpreter and the `Bridge` *trait* live) must
never depend on `horismos-bridges`, a provider SDK, or the network — this is a
`make check-boundary` grep, not a promise. Every symbolic behavior this PRP adds
(step transitions, precondition gating, tool-effect validation, trace emission) must
compile and pass its tests under `make no-model-gate`
(`cargo test -p horismos-core --no-default-features --features verify`) with zero
network and zero keys. The `Bridge` *implementations* — mock, OpenRouter — live in
`horismos-bridges`, which is allowed to depend on an HTTP client.

### Reading a workflow as data, once more

Doc 05's triage workflow (reproduced in `examples/triage/triage.ttl`) is the worked
example this whole PRP targets:

```turtle
:Triage a horis:Workflow ; horis:startsAt :Intake ; horis:version "1.2.0" .

:Intake a horis:PerceptionStep ;
    horis:extracts :Patient, :Symptom ;
    horis:model "claude:claude-fable-5" ; horis:confidence 0.85 ;
    horis:fallback :EscalateToNurse ; horis:next :Assess .

:Assess a horis:ReasoningStep ;
    horis:goal [ horis:property :acuity ; horis:isAssigned true ] ;
    horis:onUnreachable :EscalateToNurse ; horis:next :Route .

:Route a horis:ActionStep ;
    horis:tool :ehr.assign_queue ;
    horis:requires [ horis:property :acuity ; horis:isAssigned true ] ;
    horis:next :Explain .

:Explain a horis:NarrationStep ;
    horis:narrates [ horis:property :acuity ] ; horis:audience "charge-nurse" ;
    horis:next :Done .

:Done a horis:Step ; horis:terminal true .
:EscalateToNurse a horis:ActionStep ; horis:tool :paging.notify_nurse ; horis:terminal true .
```

Every arrow in this graph (`horis:next`, `horis:fallback`, `horis:onUnreachable`) is a
data edge the interpreter walks; nothing here is Rust code. The interpreter this PRP
ships is generic over *any* workflow shaped like this — triage is the fixture, not a
special case.

## Scope

- **Workflow engine** (`horismos-core::workflow`): parses `horis:Workflow` at
  `Ontology::assemble` time (mirroring PRP-002's `reason::rule` pattern — a load-time
  model, never a runtime surprise), and interprets it: step transitions, `horis:goal`
  satisfaction, `horis:requires` gating, fallback/`onError`/`onUnreachable` edges,
  terminal states. Ships execution for `PerceptionStep`, `ReasoningStep`,
  `ActionStep`, `NarrationStep`. `ChoiceStep`/`ProposalStep`/`WaitStep`/`SubflowStep`
  parse (profile-valid ontologies referencing them load) but execution is an explicit,
  gap-noted deferral — see *Key decisions*, D3/D6, and the open D13 below.
- **`horis:Tool` contracts** (doc 07): `horis:takes`/`horis:effect`/`horis:binding`
  parsed alongside workflows; a Rust-fn binding registry (`Agent::builder().tool(...)`);
  declared-effect validation of tool outputs through the same five-check gate,
  `Provenance::Tool`. Python-callable/HTTP/MCP bindings are scoped out (see
  *Coordination with PRP-003*).
- **`Bridge` finalization** (`horismos-core::bridge`): the pure trait
  (`perceive`/`narrate`/`rank`, `id()`), request/result types, `BridgeError` — no
  network, enforced by `make check-boundary`. Implementations in `horismos-bridges`:
  a deterministic `MockBridge` (CI, zero keys) and an `OpenRouterBridge` (OpenAI-
  compatible chat completions, forced `tool_choice`, feature-gated).
- **`PerceptionStep`**: JSON Schema derived from `horis:extracts` classes (doc 06's
  mapping table); one `Bridge::perceive` call with a forced `assert_facts` tool
  choice; confidence threshold; validation-gate retry loop (`horis:retries`, default
  2); mandatory fallback.
- **`NarrationStep`**: proof tree → NL via `Bridge::narrate`; non-fatal failure
  (mechanical `proof.render()` fallback) unless `horis:required true`.
- **Traces**: perception/action/narration steps extend PRP-002's already-shipped
  `horismos.trace/1` schema (`trace::emit`); full-run trajectories serialize and
  round-trip. Choice/Proposal/Wait/Subflow trace representation is explicitly **not**
  decided here — D13.

## Non-goals

- **`ProposalStep` candidate ranking, `ChoiceStep`, `SubflowStep`, `WaitStep`
  execution** — parsed, not executed, in this PRP; deferred with dated gap notes
  (D3, D6, D13). No multi-agent coordination, no streaming UI, no memory beyond the
  World (PRP-004's own stated non-goals, unchanged).
- **LLM-driven planning** — workflows are authored, not synthesized at runtime
  (PRP-007).
- **Python-callable / HTTP / MCP tool bindings** — the Rust-fn binding ships;
  the other three schemes are gap-noted, owned once PRP-003's Python surface merges
  (`docs/framework/07-tools.md`).
- **Truth maintenance for derived facts across re-assertion/retraction** —
  a defect in the already-merged PRP-002 reasoner (World does not invalidate or
  recompute a rule-derived conclusion when the base fact that supported it is later
  replaced or retracted — see gap note in `02-tasks-and-gaps.md`, citing the
  independent review at `data/horismos-prp-review-s1/report.md` top finding 2). This
  PRP does not attempt a fix; it is PRP-002's defect to own and resolve. PRP-004's
  own perception/tool-effect paths are written so the shipped demo and DoD do not
  exercise it (no property both feeds a rule condition and is corrected mid-run), and
  the exposure for workflows that *do* re-assert such a property is documented, not
  papered over.
- **Verified totality guarantees** ("every run ends at a terminal step") — doc 05's
  language describes the *intended* guarantee; it is proved statically by PRP-005,
  not by this PRP. PRP-004's own tests exercise dynamic failure behavior (fallback
  edges actually fire) without claiming the static proof.
- **Budget enforcement, a multi-world manager utility, async tool bindings** —
  D10/D12/D11 below; all deferred past v1 with recorded rationale.

## Coordination with PRP-003

PRP-003 (Python bindings) is in flight on a sibling branch and unmerged. This PRP
does not build against it: everything ships against `horismos-core` directly. The
Rust-native live entry point is `examples/triage_agent_demo.rs` (`cargo run --example
triage_agent_demo`, or `make demo` / `make demo-live`). Wiring `agent.run(...)` from
Python is recorded as a final, explicit, dated follow-up unit (`02-tasks-and-gaps.md`
U-Python) rather than attempted here — if PRP-003 has merged by the time a future
session picks this up, that unit wires it; if not, the Rust binary remains the
demo-of-record.

## Key decisions to make

Twelve decisions are resolved at activation (`02-tasks-and-gaps.md` D1–D12, each dated
2026-08-18); one — D13, trace representation for Choice/Proposal/Wait/Subflow — is
raised to the captain and stays **open**, per the independent review
(`data/horismos-prp-review-s1/report.md`, top finding 3) and interim orders on this
branch. D1–D4 are the PRP's own originally-listed decisions (loop residence,
integration-adapter priority, human-in-the-loop timing, structured-output mechanics);
D5–D12 are the eight `[DECIDE]` markers this PRP owns in `docs/framework/` (see the
marker table in `02-tasks-and-gaps.md` — the independent review's finding D1 is that
the PRP's original four-item list omitted all eight of these; this activation closes
that gap).

## Definition of done

`agent.run("58M, crushing chest pain, heart rate 128")` (via the Rust demo binary;
Python wiring is a follow-up, see above) executes the triage workflow exactly as
doc 05/06/07 dissect it:

- **Immediate track**: one `Bridge::perceive` call (mock in CI; `OpenRouterBridge`
  behind `--features bridges-openrouter` with a live key) asserts `heartRate 128` and
  `hasSymptom :CrushingChestPain` through the gate; `:Assess` observes `:r1` already
  fired during that same transaction's chaining (`acuity :Immediate`); `:Route` fires
  `:ehr.assign_queue`, whose declared effects (`:queuedAt` = the transaction-stamped
  `horis:now`, `:queueRef` type-checked as `xsd:string`) pass gate validation with
  `Provenance::Tool`; `:Explain` narrates (mock or real) with a non-fatal fallback to
  `proof.render()`; the run ends at `:Done`.
- **Fallback track**: a scripted low-confidence `MockBridge` response sends the run
  down `:Intake`'s `horis:fallback` edge to `:EscalateToNurse`, which fires
  `:paging.notify_nurse`; the run ends there, `outcome.labels.fallback_taken == true`.
  A `MockBridge` script that fails the gate on a valid-schema-but-out-of-constraint
  payload exercises the bounded-retry loop before falling back.
  All non-mock-bridge tests are property/unit tests against `MockBridge`; `make
  verify` (including `make no-model-gate` and `make check-boundary`) is green
  throughout — the symbolic core still builds and passes with every bridge excluded.
- **Tool-effect validation**: a tool binding that returns an out-of-contract effect
  (wrong type, or asserting an undeclared property) is rejected at the gate with a
  domain-term witness, and the `ActionStep`'s `horis:onError` edge fires.
- **Trajectory**: the full run's `horismos.trace/1` trajectory serializes
  (`Trace::to_json`) and round-trips (`Trace::from_json` reproduces it byte-for-byte);
  two identical mock runs against fresh `World`s produce byte-identical trajectories
  (determinism, CLAUDE.md).

## Security acceptance surface (doc 15)

PRP-004 owns the perception, tool-binding, and Bridge/credential trust boundaries.
Its DoD is not met until it also proves, alongside the functional DoD above:

- **Perception injection containment** (doc 15 §1): an adversarial NL fixture set
  showing injected instructions never fire a tool or transition, never populate a
  `RulesOnly` property, and never leak into narration; a fixture exercising
  ProposalStep's `rankBy` the same way; a fixture demonstrating the residual
  wrong-but-schema-valid-fact risk is measured, not merely assumed bounded.
- **Closed binding-scheme allowlist**: an ontology naming an unrecognized
  `horis:binding` scheme fails at load time with a domain-term error, not at
  invocation.
- **`python:` bindings resolve only via a deployer-supplied callable registry** —
  no dotted-path import of an ontology string, ever. A test asserting a binding
  absent from the registry fails at `Agent` construction.
- **`http:`/`https:` bindings default-deny**: no HTTP tool is constructible without
  an explicit deployer allowlist; a test asserting requests to private/loopback/
  link-local address ranges are rejected even for an allowlisted hostname
  (DNS-rebinding case included).
- **Secret non-serialization**: a Bridge constructed with a real-looking credential
  never has that credential appear in any emitted trace, snapshot, or log line, on
  both success *and* error paths.

Deployer-owned and stated in the DoD's docs, not silently assumed: OS-level
sandboxing of tool-execution and MCP-server processes (doc 15 §2).

This PRP's shipped DoD implements only `rust:`-fn tool bindings (see Scope/
Non-goals above); the `python:`/`http:`/`https:`/`mcp:` binding schemes, their
closed-allowlist load-time check, and the adversarial perception-injection fixture
set are NOT built in this PRP and remain open — tracked as D14/D15 in
`02-tasks-and-gaps.md` (mechanism choices) plus an unresolved-scope gap note (the
fixture set and allowlist enforcement themselves), not silently claimed met.

## Glossary

| Term | Plain-English meaning |
|------|----------------------|
| Transducer boundary | A place a model converts between representations without adding judgment — perception (NL→facts) and narration (proof→NL) are Horismos's only two. |
| `Bridge` | The trait a model client implements: `perceive`/`narrate`/`rank`, nothing else. |
| `Agent` | The generic workflow interpreter — one per (ontology, bindings); `run` borrows a `World`. |
| Forced tool choice | Using a provider's tool-calling machinery backwards: not to let the model pick an action, but as a typed-output channel the provider is required to fill. |
| `ActionStep` | A workflow step that fires a declared `horis:Tool` once its `horis:requires` precondition holds in the graph. |
| `horis:effect` | The declared *shape* of what a tool may write back — validated like any other gate check, never trusted blindly. |
| Fallback edge | The mandatory failure path every step type with a possible failure must declare (`horis:fallback`/`onUnreachable`/`onError`/`onUnsatisfied`). |
| Trajectory / trace | The full typed record of one run — audit log and training corpus, one object (`horismos.trace/1`). |
| Loss mask | Per-step flag marking model-generated output (trainable) vs. executor output (masked). |
| `horis:now` | The transaction-stamped timestamp a tool effect may reference — injected once per transaction by the runtime, never a live wall-clock read. |
| Truth maintenance | Keeping derived facts consistent as their supporting base facts change — the PRP-002 gap this PRP inherits but does not fix (see Non-goals). |
