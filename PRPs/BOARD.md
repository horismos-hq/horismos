# Horismos Board

At-a-glance project state. Updated in the same commit as any PRP status change
(`CONVENTIONS.md`). Visual version:
[Horismos Board artifact](https://claude.ai/code/artifact/dc7ee691-4c9f-40a7-b235-0760de8ff17d)
(source: `planning/board.html`, republished to the same URL on updates).

_Last updated: 2026-07-28_

## Current state

**Definition phase complete; zero engine code.** The framework is fully specified
(docs/framework 00–14, shipped voice, every open question a **[DECIDE]** owned by one
PRP). The workspace skeleton is green under `make verify`: three crates + Python
package, PyO3 roundtrip proven, anti-wrapper boundary mechanically enforced
(`check-boundary`, `no-model-gate`). Five project skills, CLAUDE.md, triage fixture
seeded.

## Next goal

**PRP-001 is Active** — all decisions D1–D6 resolved 2026-07-28 (oxigraph; interned
IRIs + persistent adjacency store; documented `horis:` constraints; postcard
snapshots; pre-extracted vocabulary subsets; strict units). Build units U1–U11
test-first, ending at the DoD: `triage.ttl` + SNOMED subset loads, schema violations
rejected in domain terms, Turtle round-trips, snapshots, pattern queries — no LLM.

Human side quests: register `horismos.dev`; file the w3id.org/horismos PR; reserve
crates.io/PyPI names early.

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
| Engine | [PRP-001 ontology core](PRP-001-ontology-core/01-objective.md) | 🟢 **Active** — D1–D6 done, building U1–U11 | Load/validate ontology, typed World, validation gate |
| Engine | [PRP-002 reasoning](PRP-002-reasoning-substrate.md) | ⬜ Draft | Chaining, subsumption, proof trees, training-grade traces |
| Engine | [PRP-005 verification](PRP-005-guarantees-verification.md) | ⬜ Draft | BDD checks + domain-term witnesses (needs only 001+002) |
| Surface | [PRP-003 python bindings](PRP-003-python-bindings.md) | ⬜ Draft | `pip install horismos`; the Pythonic front door |
| Runtime | [PRP-004 agent loop](PRP-004-agent-loop.md) | ⬜ Draft | Workflow engine, perception/narration boundaries, tools, Bridge |
| Ship & Author | [PRP-006 packaging](PRP-006-packaging-distribution.md) | ⬜ Draft | crates.io + PyPI + docs site + CI gates |
| Ship & Author | [PRP-007 authoring agent](PRP-007-authoring-agent.md) | ⬜ Draft | SMEs converse; Claude drafts Turtle; verifier gatekeeps |

Build order: 001 → 002 → 003 → 004 → 005 → 006 → 007, with 005 startable right
after 002 (it deliberately doesn't need the agent loop).

## Quick recap — what exists today

- **Spec**: docs/framework 00–14 — concepts, ontology profile, world/gate, reasoning,
  workflows, LLM boundaries, tools, bridge, traces, verification, both APIs,
  authoring, architecture.
- **Skeleton**: `horismos-core` (module tree = the seven nouns, docs-only stubs),
  `horismos-bridges` (mock + gated provider stubs), `horismos-py` (version roundtrip),
  `python/horismos`, `examples/triage/triage.ttl`, Makefile, all green.
- **Conventions**: CLAUDE.md hard rules; PRP lifecycle in
  [CONVENTIONS.md](CONVENTIONS.md).
- **Skills**: cargo-workspace-map, ontology-modeling-conventions, pyo3-boundary,
  learn-codebase (+ global superpowers).
- **Prior art**: `../sigil-lang` — Rete-lite, BDD verifier, constraint solver, trace
  design, neural bridge, authoring loop (~1,270 tests to port from).
