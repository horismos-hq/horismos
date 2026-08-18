# PRP-004 — Ontology-First Agent Loop

**Status**: Draft
**Depends on**: PRP-001, PRP-002, PRP-003

## Intent

Execute workflows declared in the ontology: perception and narration as the only LLM
boundaries, deterministic step transitions, tools invoked by the runtime (never chosen
by the model). This is the phase where "the loop is ontology data" becomes real.

## Scope

- **Workflow engine**: interpret `horis:Workflow` — step transitions, `horis:goal`
  satisfaction checks, `horis:requires` precondition gating, fallback edges, terminal
  states. The engine is generic; all domain behavior comes from the ontology.
- **PerceptionStep**: derive a JSON schema from the ontology classes in `horis:extracts`;
  one Bridge call with forced tool_choice (`assert_facts`); confidence threshold;
  mandatory fallback step; validation gate before assertion.
- **NarrationStep**: proof tree → NL rendering contract (generates prose, never
  conclusions).
- **ProposalStep**: LLM ranks only precondition-satisfied candidates; runtime executes.
- **Tools**: `horis:Tool` contracts (`takes`/`effect`/`binding`); bindings for Python
  callables, Rust fns, HTTP, MCP; declared-effect validation of tool outputs.
- **Bridge**: thin trait/protocol over provider SDKs — Anthropic, OpenAI, local
  ONNX/HF, deterministic mock. Prefix routing, per-call timeouts, fallbacks — a direct
  port of sigil-lang's `ModelBackend`/`ModelRegistry` design (76 tests of prior art).
- **Run trajectories**: agent runs emit the PRP-002 trace schema end-to-end (perception
  confidence, fired rules, tool calls, outcome labels) — the training corpus starts here.

## Non-goals

- No multi-agent coordination, no streaming UI, no memory beyond the World.
- No LLM-driven planning: workflows are authored, not synthesized at runtime
  (synthesis belongs to the authoring agent, PRP-007).

## Key decisions to make

1. Loop residence: Rust (WASM-capable, embeddable) with Python callbacks for tools, vs.
   Python loop over Rust primitives. Lean Rust-resident; validate against Python-tool
   ergonomics early.
2. Batteries-included loop AND/OR integration adapters (LangGraph node, Claude Agent SDK
   tool) — priority order.
3. Interruption/human-in-the-loop step type in v1 or deferred.
4. Structured-output mechanics per provider (forced tool_choice vs. native JSON modes) —
   hide behind the Bridge.
5. **[DECIDE, doc 15 §2]** URL allowlist configuration surface for `http:`/`https:`
   tool bindings — per-tool vs. per-Agent, glob vs. exact host match.
6. **[DECIDE, doc 15 §2]** Whether v1 adds an MCP-server allowlist analogous to the
   HTTP one, once there's a concrete MCP threat scenario to design against.

## Definition of done

`agent.run("58M, crushing chest pain…")` executes the triage workflow exactly as
dissected in the plan's Technical Outline: mock Bridge in CI (zero keys), real Anthropic
Bridge behind a flag; fallback path tested (low confidence → `:EscalateToNurse`);
tool-effect validation tested; full-run trajectory serializes and replays.

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
- **`python:` bindings resolve only via a deployer-supplied callable registry** — no
  dotted-path import of an ontology string, ever. A test asserting a binding absent
  from the registry fails at `Agent` construction.
- **`http:`/`https:` bindings default-deny**: no HTTP tool is constructible without an
  explicit deployer allowlist; a test asserting requests to private/loopback/
  link-local address ranges are rejected even for an allowlisted hostname (DNS-
  rebinding case included).
- **Secret non-serialization**: a Bridge constructed with a real-looking credential
  never has that credential appear in any emitted trace, snapshot, or log line, on
  both success *and* error paths.

Deployer-owned and stated in the DoD's docs, not silently assumed: OS-level
sandboxing of tool-execution and MCP-server processes (doc 15 §2).
