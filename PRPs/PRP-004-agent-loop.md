# PRP-004 — Ontology-First Agent Loop

**Status**: Draft
**Depends on**: PRP-001, PRP-002, PRP-003

## Intent

Execute workflows declared in the ontology: perception and narration as the only LLM
boundaries, deterministic step transitions, tools invoked by the runtime (never chosen
by the model). This is the phase where "the loop is ontology data" becomes real.

## Scope

- **Workflow engine**: interpret `sigil:Workflow` — step transitions, `sigil:goal`
  satisfaction checks, `sigil:requires` precondition gating, fallback edges, terminal
  states. The engine is generic; all domain behavior comes from the ontology.
- **PerceptionStep**: derive a JSON schema from the ontology classes in `sigil:extracts`;
  one Bridge call with forced tool_choice (`assert_facts`); confidence threshold;
  mandatory fallback step; validation gate before assertion.
- **NarrationStep**: proof tree → NL rendering contract (generates prose, never
  conclusions).
- **ProposalStep**: LLM ranks only precondition-satisfied candidates; runtime executes.
- **Tools**: `sigil:Tool` contracts (`takes`/`effect`/`binding`); bindings for Python
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

## Definition of done

`agent.run("58M, crushing chest pain…")` executes the triage workflow exactly as
dissected in the plan's Technical Outline: mock Bridge in CI (zero keys), real Anthropic
Bridge behind a flag; fallback path tested (low confidence → `:EscalateToNurse`);
tool-effect validation tested; full-run trajectory serializes and replays.
