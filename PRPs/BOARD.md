# Horismos Board

At-a-glance project state. Updated in the same commit as any PRP status change
(`CONVENTIONS.md`). Visual version:
[Horismos Board artifact](https://claude.ai/code/artifact/dc7ee691-4c9f-40a7-b235-0760de8ff17d)
(source: `planning/board.html`, republished to the same URL on updates).

_Last updated: 2026-08-18_

## Current state

**The Python front door is real: PRP-003 is Done (2026-08-18).** `pip install`
of the locally built abi3 wheel runs the doc-11 symbolic triage quickstart
verbatim, zero LLM: `Ontology.load` (+ SNOMED import), `World` with
whole-transaction `assert_` (gate + chaining inside one crossing, GIL
released, lock acquired GIL-safely), snake_case kwargs construction (D1:
dict-like generic form shipped; proxies prototyped and rejected on
evidence), the tier-2 pattern query (D2: core `World::query_pattern`;
textual SPARQL deferred), post-commit subscriptions (003/004 split pinned),
`HorismosError` hierarchy where `str(exc)` is `domain_text()` and the
structured attributes are the machine contract, proof trees rendering
byte-identically to the Rust suite, snapshots/diffs/wire form, stubs
`mypy --strict` clean, Hypothesis boundary invariants, CI wheel-matrix
config (execution is PRP-006's). 69 pytest + ~248 Rust tests, `make verify`
green. Branch: `fm/horismos-prp003-a1`. PRP-002 was recorded Done here
2026-08-18 (merged 2026-08-05 as PR #4; folder record in PR #3, open).
**Open captain hold**: derived-fact truth maintenance (plan review, finding
2) — held above the PRPs; PRP-003 binds current behavior without endorsing
it.

## Next goal

**PRP-004 (agent loop) is next** — workflows, perception/narration
boundaries, tools, the Bridge implementations — with **PRP-005
(verification) startable in parallel** (it needs only 001+002). PRP-004
inherits the pinned 003/004 subscription split (core transactional
notification stage), the `Ontology.version`/`hash` deferral (trace
stamping is its consumer), and doc 11's Agent surface.

Human side quests: register `horismos.dev`; file the w3id.org/horismos PR;
reserve crates.io/PyPI names early; merge PR #3 (PRP-002 folder record);
decide the truth-maintenance hold.

Also closed 2026-07-28 (in PRP-000): rebrand **Sigil → Horismos**, proving ground
both-track (τ-Bench number, triage narrative), license MIT OR Apache-2.0 dual.

## End goal

The chained definitions of done, PRP-000 → 007:

> `pip install horismos` / `cargo add horismos-core` from public registries. A clinician
> produces a formally verified `triage.ttl` **through conversation alone** — the
> authoring agent drafts Turtle, the BDD verifier gatekeeps with domain-term
> witnesses, the SME signs off on rendered rules, never syntax. Then
> `agent.run("58M, crushing chest pain…")` executes that workflow: one perception
> call at a declared boundary, deterministic reasoning with proof trees, tools fired
> by the runtime, a replayable training-grade trace — and the whole thing still runs
> with the LLM deleted. Docs site live, mock-bridge CI with zero keys, benchmark
> number public.

Beyond v1 (research appendix, not a phase): trace fine-tuning (Type 2 → Type 3),
provenance semirings over the Datalog-shaped subset (Type 5), GNN guidance.

## Board

| Track | PRP | Status | One-liner |
|-------|-----|--------|-----------|
| Standing | [foundational-skeleton](foundational-skeleton.md) | ✅ Done (standing) | Repo layout, tooling, skills — `make verify` green |
| Standing | [PRP-000 vision](PRP-000-vision-positioning.md) | ✅ Decisions closed | Pitch, personas, honest Kautz 2–3 placement; naming/license/benchmark resolved |
| Engine | [PRP-001 ontology core](PRP-001-ontology-core/01-objective.md) | ✅ Done (2026-07-29) | Load/validate ontology, typed World, five-check gate, snapshots, queries |
| Engine | [PRP-002 reasoning](PRP-002-reasoning-substrate/01-objective.md) | ✅ Done (2026-08-05, PR #4; recorded 2026-08-18) | Chaining, subsumption, proof trees, training-grade traces |
| Engine | [PRP-005 verification](PRP-005-guarantees-verification.md) | ⬜ Draft | BDD checks + domain-term witnesses (needs only 001+002) |
| Surface | [PRP-003 python bindings](PRP-003-python-bindings/01-objective.md) | ✅ Done (2026-08-18) | `pip install horismos`; the Pythonic front door |
| Runtime | [PRP-004 agent loop](PRP-004-agent-loop.md) | ⬜ Draft — **next up** | Workflow engine, perception/narration boundaries, tools, Bridge |
| Ship & Author | [PRP-006 packaging](PRP-006-packaging-distribution.md) | ⬜ Draft | crates.io + PyPI + docs site + CI gates |
| Ship & Author | [PRP-007 authoring agent](PRP-007-authoring-agent.md) | ⬜ Draft | SMEs converse; Claude drafts Turtle; verifier gatekeeps |

Build order: 001 → 002 → 003 → 004 → 005 → 006 → 007, with 005 startable right
after 002 (it deliberately doesn't need the agent loop).

## Quick recap — what exists today

- **Spec**: docs/framework 00–14 — concepts, ontology profile, world/gate, reasoning,
  workflows, LLM boundaries, tools, bridge, traces, verification, both APIs,
  authoring, architecture.
- **Engine (PRP-001 + PRP-002)**: `horismos-core::ontology` (terms/interning,
  Turtle, profile validation, schema + subsumption, imports/anchoring),
  `horismos-core::world` (store, five-check gate, provenance, snapshots/diff/
  binary form, tier-1 queries), `horismos-core::reason` (rule model, 11
  operators, naive + Rete-lite matchers, forward/backward chaining, proof
  trees) and `horismos-core::trace` (`horismos.trace/1`), property-tested;
  fixtures `examples/triage/{triage,snomed-ct-subset}.ttl`.
- **Skeleton**: `horismos-bridges` (mock + gated provider stubs), `horismos-py`
  (version roundtrip), `python/horismos`, Makefile — all green.
- **Conventions**: CLAUDE.md hard rules; PRP lifecycle in
  [CONVENTIONS.md](CONVENTIONS.md).
- **Skills**: cargo-workspace-map, ontology-modeling-conventions, pyo3-boundary,
  learn-codebase, generate-prp, impl-prp, git-workflow, self-review,
  brainstorming + the senior-reviewer agent (+ global superpowers).
- **Prior art**: `../sigil-lang` — Rete-lite, BDD verifier, constraint solver, trace
  design, neural bridge, authoring loop (~1,270 tests to port from).
