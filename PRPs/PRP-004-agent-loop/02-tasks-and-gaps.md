# PRP-004 — Tasks & Gaps

## [DECIDE] marker inventory (independent-review finding D1: the PRP's original
four-item list omitted all eight framework markers this PRP owns — closed by this
table)

| # | Marker | Location | Blocks | Resolved at |
|---|--------|----------|--------|-------------|
| D5 | `horis:decides` modeling | `docs/framework/05-workflows.md:115-116` | U2 (outcome computation) | this doc, below |
| D6 | Subflow recursion policy | `docs/framework/05-workflows.md:120-123` | U12 (deferred) | this doc, below |
| D7 | Narration failure default | `docs/framework/06-llm-boundaries.md:86-88` | U-Narration | this doc, below |
| D8 | Deployment-level model override | `docs/framework/08-bridge.md:53-56` | U7 (Agent builder) | this doc, below |
| D9 | Confidence calibration strategy | `docs/framework/08-bridge.md:70-75` | U4/U13 (Bridge, OpenRouter) | this doc, below |
| D10 | Budget semantics | `docs/framework/08-bridge.md:84-87` | (none — deferred) | this doc, below |
| D11 | Sync/async tool story | `docs/framework/12-rust-api.md:41-45` | U3/U10 (Tool trait) | this doc, below |
| D12 | Multi-world manager utility | `docs/framework/12-rust-api.md:83-88` | (none — deferred) | this doc, below |
| D14 | URL allowlist configuration surface for http:/https: bindings | `docs/framework/15-security.md:133-134` | a future http:/https: tool-binding unit (not yet built in this PRP) | this doc, below (open) |
| D15 | MCP-server allowlist policy | `docs/framework/15-security.md:139-140` | a future mcp: tool-binding unit (not yet built in this PRP) | this doc, below (open) |

All eight markers above are struck from their docs (rewritten in shipped voice) in
the same commit as this file.

## Decisions to resolve (before implementation)

- [x] D1. **Loop residence** (decided 2026-08-18): Rust-resident. The `Agent`
      interpreter lives in `horismos-core::workflow` as a generic Rust step
      interpreter (doc 12's `Agent::builder(...)` API), matching the PRP's original
      lean. Risk considered: Python-tool ergonomics. Resolution: a `python:`-bound
      tool is still one Rust `Tool` impl that marshals in/out at invocation only — no
      cross-language chatter inside a transaction (doc 14) — so residence in Rust does
      not force awkward Python tool authoring; it only defers *building* the
      `python:` binding scheme itself (see D-Python-Bindings gap, below), which is a
      separate question from where the loop lives. No full spike was run; the
      architectural read above is the evidence, and the risk is bounded because the
      `Tool` trait boundary was designed to absorb it either way.
- [x] D2. **Integration adapters priority** (decided 2026-08-18): batteries-included
      loop ships first; a LangGraph node / Claude Agent SDK tool adapter is deferred
      past v1 (gap, dated below). No adapter code in this PRP.
- [x] D3. **Human-in-the-loop step (`horis:WaitStep`)** (decided 2026-08-18):
      deferred to v1.1, not shipped as trivially cheap. Pausing a run and resuming it
      later — across a process boundary, in the general case — needs persisted
      in-flight run state that nothing in the engine yet provides; that is not a
      cheap add. `horis:WaitStep` parses (already-allowed profile vocabulary);
      executing one returns `WorkflowError::StepDeferred { step: "WaitStep" }` naming
      the step in domain terms. Gap, dated below.
- [x] D4. **Structured-output mechanics per provider** (decided 2026-08-18): hidden
      behind `Bridge`. `PerceiveRequest` carries a JSON Schema (derived from
      `horis:extracts`, doc 06's mapping table) plus the input text; each `Bridge`
      decides how to force conformance — `MockBridge` matches scripted responses by
      input pattern; `OpenRouterBridge` forces `tool_choice` on an OpenAI-compatible
      `assert_facts` tool (OpenRouter's own API reference documents
      `tool_choice: {"type":"function","function":{"name":...}}` as OpenAI-compatible
      forcing). **Not live-verified**: `OPENROUTER_API_KEY` was not present in this
      environment (checked — no env var, no local key file), so this is verified
      against OpenRouter's published API reference, not a live call. The captain can
      verify live: `export OPENROUTER_API_KEY=sk-... && make demo-live`. Dated gap:
      confirm live tool_choice forcing across at least one real OpenRouter-proxied
      model once a key is available; if a specific model ignores forcing, the
      documented behavior here needs a per-model note.
- [x] D5. **`horis:decides` modeling** (decided 2026-08-18, closes
      `docs/framework/05-workflows.md:115-116`): single decision property per
      workflow, not a per-terminal-step outcome map. `horis:decides` names exactly
      one property (`:Triage horis:decides :acuity`); `result.decision` is
      `Some({property, value})` when the run ends holding a value for that property,
      `None` otherwise. This matches the trace schema already shipped in PRP-002
      (`outcome.decision` is one `{"property", "value"}` pair, not a list,
      `docs/framework/09-traces.md:53-56`) — the simpler reading is also the one
      already committed to on the wire.
- [x] D6. **Subflow recursion policy** (decided 2026-08-18, closes
      `docs/framework/05-workflows.md:120-123`): depth cap of exactly one, matching
      the doc's own "one level of nesting… in v1" — a workflow reachable via a
      `horis:SubflowStep` may not itself declare a `horis:SubflowStep`; checked at
      `Ontology::assemble` (a load error, never a runtime surprise, same posture as
      PRP-002's rule-load checks). Execution itself is deferred (D13/U12) — the
      *policy* is decided independently of whether the runtime executes it yet.
- [x] D7. **Narration failure default** (decided 2026-08-18, closes
      `docs/framework/06-llm-boundaries.md:86-88`): confirmed non-fatal by default —
      the doc's own preceding sentences already stated this; the marker was only
      asking for sign-off. `NarrationStep` failure (Bridge error, timeout, refusal)
      falls back to the mechanical `proof.render()` text and the run continues;
      `horis:required true` makes it fatal (`WorkflowError`).
- [x] D8. **Deployment-level model override** (decided 2026-08-18, closes
      `docs/framework/08-bridge.md:53-56`): yes, with trace annotation, per the
      doc's own lean. `Agent::builder().bridge(prefix, bridge_instance)` may bind a
      different concrete model than the ontology's `horis:model` string names for
      the same prefix (a cheaper model, or `mock:` in tests); every perception/
      narration/proposal trace step's `model` field records `Bridge::id()` — the
      model **actually invoked** — not merely the ontology's declared string, so an
      override is visible after the fact without a new trace field.
- [x] D9. **Confidence calibration strategy** (decided 2026-08-18, closes
      `docs/framework/08-bridge.md:70-75`): one strategy for every provider in v1 —
      calibrated self-report. The `assert_facts` tool's JSON Schema carries a
      `confidence` number field the model must fill in; a `PerceiveResult` that omits
      it is treated as confidence `0.0` (fails safe into the fallback edge rather
      than false-accepting). Logprob-derived confidence, where a provider exposes it,
      is a future per-backend enhancement layered on top of the same
      `PerceiveResult.confidence` field — not required for any backend to ship in v1.
      `MockBridge`'s scripted confidence makes every test independent of either.
- [x] D10. **Budget semantics** (decided 2026-08-18, closes
      `docs/framework/08-bridge.md:84-87`): defer past v1. Per-call accounting
      (model, latency, tokens, cost estimate) ships in v1 and lands in every trace
      step and in `Totals` — `horis:budget` as a workflow-level fallback-triggering
      threshold is not implemented; the accounting the feature would consume already
      exists, so adding the threshold check later is additive, not a redesign.
- [x] D11. **Sync/async tool story** (decided 2026-08-18, closes
      `docs/framework/12-rust-api.md:41-45`): v1 ships `Tool::invoke` as a plain sync
      call, per doc 12's primary declaration. An async variant behind a future
      `async-tools` feature is deferred — nothing in this PRP needs it (Rust-fn
      bindings are fast and synchronous by construction; a tool binding that blocks
      the executor is the deployer's concern, `docs/framework/14-architecture.md`'s
      existing security-and-trust posture).
- [x] D12. **Multi-world manager utility** (decided 2026-08-18, closes
      `docs/framework/12-rust-api.md:83-88`): defer past v1. One `Agent` + one
      `World` per case is the documented pattern (doc 12); a registry/manager utility
      (spawn/reap worlds keyed by case id) is straightforward for a caller to write
      directly and is deferred until a real deployment needs one built in.
- [x] D13. **Trace representation of `ChoiceStep`/`ProposalStep`/`WaitStep`/
      `SubflowStep`** (decided 2026-08-19, captain decision, option (b) of the two
      below). Per the independent review (`data/horismos-prp-review-s1/report.md`, top
      finding 3) and interim orders on this branch: `horismos.trace/1`'s `Step` enum
      had exactly four kinds (perception/reasoning/action/narration,
      `crates/horismos-core/src/trace/schema.rs:108-172`); the compatibility policy
      (`trace/migrate.rs`) requires a new step kind to be `horismos.trace/2`, not an
      additive v1 change. The two options considered:
      (a) map each of the four remaining step kinds losslessly into the existing four
      trace kinds via annotated fields (e.g. a `ChoiceStep` firing recorded as a
      `reasoning`-kind step with a `branch_taken` field) — cheap, no version bump, but
      needs a specification of the lossless mapping for each kind before it is
      trustworthy training data; or
      (b) **mint `horismos.trace/2` now**, before any real corpus depends on v1's exact
      four kinds — chosen. `trace::schema` now declares `SCHEMA_V2` and four new `Step`
      variants (`Choice`, `Proposal`, `Subflow`, `Wait`) alongside the original four,
      each documented with its `loss_mask` semantics; `Trace::new` still emits v1
      unchanged (byte-for-byte, every PRP-002 test untouched) and a parallel
      `Trace::new_v2` emits v2 for the Agent's code path. `trace::migrate` gained
      `convert_v1_to_v2` (pure, purely additive — rewrites only the `"schema"` tag,
      since a v1 trace can only ever contain the original four kinds by construction)
      with round-trip tests per kind, including a hand-built `Step::Wait` fixture since
      execution never produces one (D3). **resolved: trace/2 minted and absorbed** —
      `trace::schema`/`trace::migrate` land in this commit; wiring `trace::emit`
      builders for the four new kinds and the Agent's use of `Trace::new_v2` is U9/U7,
      tracked below, not blocked by this decision any longer.
- [ ] D14. **URL allowlist configuration surface for `http:`/`https:` tool
      bindings** (open, closes `docs/framework/15-security.md:133-134`): per-tool
      vs. per-Agent scoping, glob vs. exact host match — undecided. Does not block
      anything already shipped: this PRP ships only `rust:`-fn tool bindings (see
      Scope); no `http:`/`https:` binding scheme exists yet for this decision to
      gate. Blocks whichever future unit adds one.
- [ ] D15. **MCP-server allowlist policy** (open, closes
      `docs/framework/15-security.md:139-140`): whether v1 adds an MCP-server
      allowlist analogous to the HTTP one, once there is a concrete MCP threat
      scenario to design against — undecided. Same scoping rationale as D14: no
      `mcp:` binding scheme exists yet in this PRP.

## Units of work

- [x] U1. **Ontology vocabulary extension for Tool contracts** — extend
      `ontology::profile::ALLOWED_HORIS` with the doc-07 Tool vocabulary (`Tool`,
      `takes`, `arg`, `name`, `type`, `effect`, `asserts`, `binding`, `timeout`,
      `idempotent`, `retries`, `now`) and the doc-07 `ProposalStep` vocabulary
      (`candidates`, `rankBy`) plus `decides` for D5 (already present) and
      `StepDeferred`-relevant terms already in the list (`WaitStep`, `ChoiceStep`,
      `SubflowStep` — already allowed). Test target:
      `examples/triage/triage.ttl`, extended with `:ehr.assign_queue` and
      `:paging.notify_nurse` `horis:Tool` declarations (doc 07's own example plus a
      matching second tool — both are currently *referenced* by the workflow's
      `ActionStep`s but never *declared*, a fixture gap this PRP closes), still
      passes `ontology::profile::validate`.
- [x] U2. **Workflow + Step model parsing** (`workflow::model`) — parse
      `horis:Workflow`/`horis:*Step` into typed Rust structs at `Ontology::assemble`,
      mirroring `reason::rule`'s pattern (load-time errors, never runtime surprises).
      `Ontology::workflow(name)` accessor. `horis:decides` per D5. Depth-1 check for
      `SubflowStep` per D6. Test target: `triage.ttl`'s `:Triage` workflow parses into
      the exact documented step graph (`:Intake` → `:Assess` → `:Route` → `:Explain`
      → `:Done`, `:EscalateToNurse` terminal), with every edge (`next`/`fallback`/
      `onUnreachable`/`onError`) resolved to the right step. **Done 2026-08-19**:
      `workflow::model::tests::triage_workflow_parses_into_the_documented_step_graph`
      green; the fixture also needed `:Route`/`:EscalateToNurse` `horis:onError` edges
      and a new terminal `:PagingFailed` step (a fixture gap, not a model gap — noted
      below).
- [x] U3. **Tool model parsing** (`workflow::tool`) — `horis:Tool`
      (`takes`/`effect`/`binding`) parsed at load, sync-only per D11. Test target:
      `:ehr.assign_queue` parses with two args (`patient: :Patient`, `queue:
      :AcuityLevel`) and two declared effects (`:queuedAt` = `horis:now` sentinel,
      `:queueRef`: `xsd:string`-typed placeholder). **Done 2026-08-19**:
      `workflow::tool::tests::ehr_assign_queue_parses_with_two_args_and_two_effects`
      green; the fixture was missing the four effect properties'
      (`:queuedAt`/`:queueRef`/`:notifiedAt`/`:pagedTo`) schema declarations
      (`horis:assignedBy horis:ToolsOnly`), also closed here.
- [ ] U4. **`Bridge` trait finalization** (`horismos-core::bridge`) — `perceive`/
      `narrate`/`rank`/`id()`, request/result types, `BridgeError`. Confidence field
      on `PerceiveResult` per D9. No network dependency. Test target: a trivial
      in-crate test double implements `Bridge` and the crate still builds/tests under
      `--no-default-features --features verify` (no-model-gate). In progress on the
      contractor branch `fm/horismos-prp004-u4-u13`; not yet integrated — the
      `horismos-core::bridge` module today is still the PRP-000-era doc-only stub.
- [ ] U5. **`HorismosError` extension** — `WorkflowError`, `ToolError`,
      `BridgeError` variants (the three the error module's own comment already
      names as pending), each `domain_text()`'d. In progress alongside U4 on
      `fm/horismos-prp004-u4-u13`. (Not to be confused with the `OntologyError`
      load-time variants U2/U3 already added — `StepMissingOperator` and friends —
      which are a different enum for a different failure class: malformed ontology
      vs. a failed run.)
- [x] U6. **`MockBridge`** (`horismos-bridges::mock`) — scripted perceive/narrate/
      rank by input pattern, deterministic, records invocations, default
      low-confidence response for unscripted input. Test target: scripted responses
      round-trip exactly; unscripted input yields the documented low-confidence
      default. **Done 2026-08-19**: exact-match scripting per call kind
      (`perceive`/`narrate`/`rank`, each keyed by its own input text —
      `PerceiveRequest::text`/`NarrateRequest::proof_text`/`RankRequest::context`)
      with a `"*"` wildcard, matching doc 08's own Python illustration; unscripted
      calls fall through to each result type's `Default` (perceive: confidence
      `0.0`, empty instances — D9's fail-safe). `MockBridge::invocations()` returns
      every call in order for a test to assert against. 6 tests green
      (`mock::tests::*`).
- [ ] U7. **Agent interpreter** (`workflow::agent`) — `Agent::builder(&onto)
      .workflow(name).bridge(prefix, bridge).tool(name, impl).build()`;
      `agent.run(&mut world, RunInput)` executing Perception → Reasoning → Action →
      Narration, `horis:requires` gating, fallback/`onUnreachable`/`onError` edges,
      `horis:now` transaction-stamping (once per `run`, never re-read). Deployment
      model override per D8. Test target: a mock-bridge run of the full triage
      workflow reaches `:Done` with `acuity :Immediate`, matching PRP-002's Example 1
      hand-computed proof exactly. Unblocked now that U4/U6 exist; mine to drive next
      (`workflow::mod`'s `agent` submodule is deliberately not yet declared — see its
      module doc).
- [ ] U8. **Fallback path** — `MockBridge` scripted to a low-confidence response;
      test target: the run takes `:Intake`'s `horis:fallback` edge to
      `:EscalateToNurse`, fires `:paging.notify_nurse`, and
      `outcome.labels.fallback_taken == true`. A second script returns a
      schema-valid-but-gate-rejected payload to exercise the bounded-retry loop
      (`horis:retries`, default 2) before falling back.
- [ ] U9. **Trace emission extension** (`trace::emit`) — `perception_step`/
      `action_step`/`narration_step` builders mirroring `reasoning_step`; run-level
      `Trace` fields (`ontology`, `workflow`, `input`, `outcome`, `totals`)
      populated by the Agent. Test target: a full run's trajectory matches doc 09's
      documented shape field-for-field; `Trace::from_json(t.to_json()) == t`; two
      identical mock runs against fresh `World`s produce byte-identical trajectories.
      **Partially done 2026-08-19** (D13's prerequisite): `trace::schema` ships
      `SCHEMA_V2`/`Trace::new_v2` and the four new `Step` variants; `trace::migrate`
      ships `convert_v1_to_v2` with per-kind round-trip tests. The `trace::emit`
      builder functions and the Agent's population of run-level fields remain —
      blocked on U7.
- [ ] U10. **Tool-effect validation** — `ActionStep` resolves args from the graph,
      invokes the bound `Tool`, validates the returned effect against
      `horis:effect` (type/class match for typed placeholders; `horis:now` resolved
      to the transaction stamp) through the gate with `Provenance::Tool`;
      `horis:onError` on gate rejection, invocation error, or timeout. Test target:
      an out-of-contract tool return (wrong type; an undeclared property) is
      rejected with a domain-term witness and the step follows `onError`; a
      well-formed return's effects are visible to the next step.
- [ ] U11. **`horis:now` transaction stamping** — `Agent::run` (or `Tool`'s
      invocation context) carries one timestamp, supplied by the caller (defaults to
      the host clock read exactly once, at transaction open — never inside chaining),
      never re-read; deterministic tests pass a fixed instant. This is the "no
      wall-clock in inference" rule (CLAUDE.md) made concrete for tool effects.
- [ ] U12. **Deferred step kinds** (`ChoiceStep`/`ProposalStep`/`WaitStep`/
      `SubflowStep`) — parse (already-allowed profile vocabulary; `SubflowStep`'s
      depth-1 check per D6), execute as `WorkflowError::StepDeferred { step }` naming
      the concrete step and kind in domain terms. Explicitly does **not** touch trace
      emission — D13 is resolved (trace/2 exists), but wiring these four kinds'
      *execution* traces is still future work past this PRP's DoD; parsing (U2) is
      already done. Test target: a workflow using one of these step kinds loads
      cleanly and fails at `run()` with the named, domain-term error — not a panic,
      not a silent no-op.
- [ ] U13. **`OpenRouterBridge`** (`horismos-bridges::openrouter`, feature
      `bridges-openrouter`) — OpenAI-compatible chat completions client, base URL
      `https://openrouter.ai/api/v1`, model from `OPENROUTER_MODEL` env (default an
      inexpensive model), forced `tool_choice` on `assert_facts`/`narrate`/
      `rank`-shaped calls. Reads `OPENROUTER_API_KEY` from env or a gitignored local
      file only — never logged, never committed, never in a trace. Test target:
      request-shape unit tests against a mocked HTTP layer (no network, no key, still
      runs in CI) verifying the forced-tool-choice request body; a `#[ignore]`d live
      test gated on the key's presence for the captain to run by hand (D4's
      live-verification gap). **Superseding note (2026-08-19, captain directive)**:
      the text above describes the original thin-`reqwest` shape; the captain has
      since directed this to use the `litellm-rust` SDK crate as the provider layer
      instead, under strict containment — exact-pinned version, lives only inside
      `horismos-bridges` behind `Bridge` (nothing above the trait may know it exists),
      its response adapter's discarding of `model`/`finish_reason`/OpenRouter routing
      metadata must be verified against the pinned version and those trace-audit
      fields recovered if so, forced `tool_choice` for `assert_facts` must work
      through it (empirically verified, `needs-decision` if it cannot), and
      `MockBridge` (U6) stays crate-free. In progress on `fm/horismos-prp004-u4-u13`;
      reconcile this entry's text with whatever that branch actually ships when
      integrated.
- [ ] U14. **Live demo** (`examples/triage_agent_demo.rs` + `make demo` /
      `make demo-live`) — loads `triage.ttl` + the SNOMED subset, runs "58M, crushing
      chest pain, heart rate 128" through `MockBridge` (CI/`make demo`) or
      `OpenRouterBridge` (`make demo-live`, needs `OPENROUTER_API_KEY`), prints the
      proof tree and narration, writes the full trajectory JSON. Test target: the
      mock run's trajectory matches a committed golden JSON exactly (byte-for-byte,
      determinism).
- [ ] U15. **The DoD run** (`crates/horismos-core/tests/agent_loop_dod.rs`) — the
      final unit: one test module reproducing the whole Definition of Done
      (immediate track, fallback track, tool-effect validation, trajectory
      serialize+replay) against `MockBridge`, independent of `bridges-openrouter`.
- [ ] U-Python. **Python `agent.run(...)` wiring** — deferred: PRP-003 is
      unmerged on a sibling branch (see `01-objective.md` § Coordination with
      PRP-003). Dated 2026-08-18: pick this up once PRP-003 merges; until then the
      Rust demo binary (U14) is the entry point of record.

## Gaps discovered during implementation

- **2026-08-18 — inherited defect, not fixed here: PRP-002 truth maintenance.**
  Independent review `data/horismos-prp-review-s1/report.md` top finding 2
  (CONFIRMED · Critical): `World::assert_instance_as` re-chains on every assertion,
  but neither it nor `World::retract_as` invalidates or recomputes a standing
  rule-derived conclusion when the base fact that supported it is later replaced or
  retracted (`crates/horismos-core/src/world/mod.rs:340-419,603-632`). Concretely: if
  a patient's `heartRate` is asserted at 128 (deriving `acuity :Immediate` via `:r1`),
  then later re-asserted at 96 or retracted, `acuity` stays `:Immediate` — stale,
  silently. This is a PRP-002 defect (truth-maintenance policy is unresolved there,
  not a PRP-004 question), and this PRP does not attempt a fix — that decision
  belongs to PRP-002's own record per the review's recommended repair order. Exposure
  in the agent loop: PRP-004's own perception/tool-effect assertion paths were
  written so the shipped demo and DoD never re-assert a rule-supporting property
  mid-run (the triage happy path asserts `heartRate`/`hasSymptom` once, and the
  `:Route` tool's effects — `:queuedAt`, `:queueRef` — never overlap with any rule
  condition), so the demo does not visibly exhibit the bug. Any future workflow that
  *does* correct a rule-supporting base fact mid-run (a corrected vital sign, a
  retracted symptom) inherits this defect until PRP-002 resolves it. Flagged here,
  not papered over, per interim orders on this branch.
- **2026-08-18 — D13 above**: trace representation for Choice/Proposal/Wait/Subflow
  is an open captain decision; see the D-item.
- **2026-08-19 — security acceptance surface ported forward from PR #6.** PR #6
  (security threat model, `docs/framework/15-security.md`) added a Security
  acceptance surface subsection and two `[DECIDE]` markers (D14/D15 above) to the
  old single-file `PRP-004-agent-loop.md` draft, after this branch had already
  converted that file to folder form at activation. This rebase ports that content
  forward rather than losing it on conflict. This PRP's already-shipped DoD
  (`rust:`-only tool bindings, U1–U15) does not claim to meet the
  `http:`/`https:`/`python:`/`mcp:`-binding-specific bullets of the security
  acceptance surface, since those binding schemes are not built here (see
  Non-goals) — D14/D15 and the adversarial-fixture-set requirement stay open for
  whichever future PRP/unit adds those binding schemes.
