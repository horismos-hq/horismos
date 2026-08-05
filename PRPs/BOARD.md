# Horismos Board

At-a-glance project state. Updated in the same commit as any PRP status change
(`CONVENTIONS.md`). Visual version:
[Horismos Board artifact](https://claude.ai/code/artifact/dc7ee691-4c9f-40a7-b235-0760de8ff17d)
(source: `planning/board.html`, republished to the same URL on updates).

_Last updated: 2026-07-29_

## Current state

**The ontology core is real: PRP-001 is Done.** `horismos-core` loads an RDFS-plus
ontology (Turtle via oxigraph's parser crates), validates it against the profile,
builds a queryable schema with sameAs-anchored subsumption over imported
vocabularies (a ~50-concept SNOMED chest-pain subset ships as the fixture), and
holds a typed World whose **every mutation passes the five-check validation gate** —
atomic rejection, domain-term witnesses ("heartRate 1280 exceeds horis:max 300
(bpm) on :Patient"), provenance policy (`horis:assignedBy`) enforced. O(1)
snapshots with structural diff and a gate-revalidated postcard binary form;
tier-1 pattern queries; a proptest suite pinning gate invariants (validity,
atomicity, rejection naming, byte-identical determinism); the DoD run is
`crates/horismos-core/tests/dod.rs`. ~160 tests, `make verify` green, zero LLM.
Branch: `prp-001-ontology-core` (PR #1). Six PRP-workflow skills + a
senior-reviewer agent now live in `.claude/`.

## Next goal

**PRP-002 (reasoning substrate) is next**: forward chaining (Rete-lite),
backward `prove`/`explain`, proof trees, training-grade traces. It inherits
dated gap notes from PRP-001 (see its `02-tasks-and-gaps.md`): retraction vs
`horis:immutable` [DECIDE], sameAs-canonical class lookup, stored fact
provenance, plus D6's rule-side unit check at rule load.

Human side quests: register `horismos.dev`; file the w3id.org/horismos PR; reserve
crates.io/PyPI names early; merge PR #1.

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
| Engine | [PRP-002 reasoning](PRP-002-reasoning-substrate/01-objective.md) | ⬜ Draft — **next up** | Chaining, subsumption, proof trees, training-grade traces |
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
- **Engine (PRP-001)**: `horismos-core::ontology` (terms/interning, Turtle,
  profile validation, schema + subsumption, imports/anchoring) and
  `horismos-core::world` (store, five-check gate, provenance, snapshots/diff/
  binary form, tier-1 queries), property-tested; fixtures
  `examples/triage/{triage,snomed-ct-subset}.ttl`.
- **Skeleton**: `horismos-bridges` (mock + gated provider stubs), `horismos-py`
  (version roundtrip), `python/horismos`, Makefile — all green.
- **Conventions**: CLAUDE.md hard rules; PRP lifecycle in
  [CONVENTIONS.md](CONVENTIONS.md).
- **Skills**: cargo-workspace-map, ontology-modeling-conventions, pyo3-boundary,
  learn-codebase, generate-prp, impl-prp, git-workflow, self-review,
  brainstorming + the senior-reviewer agent (+ global superpowers).
- **Prior art**: `../sigil-lang` — Rete-lite, BDD verifier, constraint solver, trace
  design, neural bridge, authoring loop (~1,270 tests to port from).
