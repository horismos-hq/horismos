# Horismos Board

At-a-glance project state. Updated in the same commit as any PRP status change
(`CONVENTIONS.md`). Visual version:
[Horismos Board artifact](https://claude.ai/code/artifact/dc7ee691-4c9f-40a7-b235-0760de8ff17d)
(source: `planning/board.html`, republished externally after status changes merge).

_Last updated: 2026-09-21 (PRP-006 activated: the PyPI release path — packaging
metadata, single-source version, `.github/workflows/release.yml` — landed; nothing
published yet). Previously: 2026-09-05 (PRP-008 merged into `main`'s state: the branch's host SPI,
candidate-artifact pipeline, FHIR/harness adapters and VTE slice now sit on top of the
landed PRP-005 verifier and both documentation sites). Previously: 2026-09-03 (PRP-005
Done; PRP-004 corrected to Done after PR #8 merged 2026-08-25; PRP-008 completed on its
isolated branch). Previously: 2026-09-02 (PRP-008 activated in parallel with PRP-005;
VTE prophylaxis confirmed and D1–D10 resolved). Previously: 2026-08-24
(bounded-projection-kernel pivot adopted; new PRP-008 drafted;
PRP-000/001/002/003/004/005/007 and framework docs 00–15 amended for the new positioning —
no PRP-001–004 DoD rewritten, no runtime code changed). Previously: 2026-08-19 (PRP-004's
U7–U15 critical path implemented, PR #8 open pending review; PRP-002 truth-maintenance
defect fixed on `main` (PR #7) — no longer an open captain hold)._

## Pivot: bounded projection kernel (2026-08-24)

Captain-adopted (`data/horismos-thin-pivot-s1/report.md`,
`data/horismos-thin-pivot-s1/decisions/captain-pivot-ruling.md`). Horismos's positioning
moves from "one artifact owns domain + rules + workflows" to a **thin, standards-
compatible decision/capability layer**: adapters hydrate versioned, explicitly scoped
snapshots from an existing system of record (FHIR/CIS, or a coding-harness repository);
the unchanged gate/reasoning/truth-maintenance/proof/trace core runs over that local
projection; typed proposed effects commit back conditionally, failing closed on
source-version drift. Full statement: `PRPs/PRP-000-vision-positioning.md` §"Snapshot-
scoped guarantees". This is a **documentation/spec-only** change in this pass — no
`horismos-core`/`horismos-py` code changed, no PRP-001–004 Done/Active DoD rewritten
(each gained a non-normative "Pivot note" reinterpreting scope, not altering what was
built or verified). `PRPs/PRP-008-standards-host-integration/` is Done: VTE prophylaxis
now exercises the bounded projection end to end, with the five-field PRP-005 assumption
coupling isolated in `horismos-host::assumptions` — the parallel verifier lane has since
landed, so that isolation is now a conversion seam to converge rather than a placeholder.
PRP-008 remains sequenced before PRP-006/007.

## Current state

**PRP-005 (guarantees & verification) is Done (2026-09-03).** The pure-Rust,
WASM-safe BDD verifier checks rule completeness/conflicts/deadness/termination,
workflow reachability/termination/requires/fallbacks/choices, and rule/constraint
non-vacuity. Every deterministic report carries the five-field assumption manifest and
domain-term machine witnesses. Ordinary Rust and Python loads verify and fail closed;
the explicit `allow_incomplete` authoring escape remains visible in every subsequent
agent trace. Canonical triage is complete, its seeded gap has the exact required
witness, and load plus verification benchmarks below 1 ms here.

**PRP-004 (agent loop) is Done.** PR #8 merged on 2026-08-25, shipping the workflow
engine, Bridge/tool boundaries, trace/2 emission, and the triage DoD run. This corrects
the board's stale Active status; its future-binding security decisions remain owned by
the later PRP that adds those binding schemes.

The Python front door remains Done under PRP-003, and the derived-fact truth-maintenance
fix remains landed on `main` through PR #7.

## Next goal

**PRP-008 (standards & host integration) is Done** (2026-09-03, branch
`fm/horismos-prp008-a1`; merged with `main` 2026-09-05). It supplies the
bounded-projection adapter types and the first standards-anchored vertical slice before
PRP-006/007, per the captain-adopted dependency DAG: the generic host SPI,
candidate-artifact runtime boundary, FHIR adapter, CDS Hooks/PlanDefinition subset, VTE
slice, and coding-harness proof all ship together. PRP-005 has since landed on `main`,
so the one remaining coupling is bookkeeping: `horismos-host::assumptions` still carries
its own five-field `AssumptionManifest` (same five fields, adapter-native types)
alongside the shipped `horismos-core::verify::AssumptionManifest`; converging the two is
gap-noted in PRP-008's `02-tasks-and-gaps.md`, not a behaviour gap.

**PRP-006 (packaging & distribution) is Active (2026-09-21)**, unlocked by that
integration surface. Its first slice has landed: `pyproject.toml` carries the PyPI
metadata, the workspace `version` in the root `Cargo.toml` is the single source
maturin normalises into the PEP 440 distribution version, and
`.github/workflows/release.yml` builds the four wheel platforms plus an sdist, gates
both on the doc-11 quickstart, and uploads through Trusted Publishing with no token in
the repo. PyPI still has nothing. The first TestPyPI rehearsal (run 35682837533)
uploaded four wheels and was then refused on the sdist, whose `License-File:` metadata
named licence files the tarball did not contain; `pyproject.toml` now declares them
under PEP 639. Those wheels were `0.1.0a1`, so that version is spent on TestPyPI and
cannot be re-uploaded — hence the bump to `0.1.0-alpha.2`. A later rehearsal then
uploaded all four wheels and the sdist at `0.1.0a2`, spending that version too; the
publish step keeps no `skip-existing`, so the workspace is now `0.1.0-alpha.3` to give
the next rehearsal a fresh version. The rest of the phase —
crates.io, the generated API reference, the dependency-audit/SBOM gates, CHANGELOG and
tagged releases — is unstarted.

Human side quests: register `horismos.dev`; file the w3id.org/horismos PR;
reserve crates.io/PyPI names early; merge PR #3 (PRP-002 folder record).

Also closed 2026-07-28 (in PRP-000): rebrand **Sigil → Horismos**, proving ground
both-track (τ-Bench number, triage narrative), license MIT OR Apache-2.0 dual.

**Security/trust workstream authorized 2026-08-18** (captain decision, per
independent-review findings R2/D4: security posture existed only as a disclaimer,
and several v1 trust-boundary decisions had no PRP owner). Not a new PRP number —
cross-cutting: `docs/framework/15-security.md` records the v1 threat model (assets,
trust boundaries, threats, mitigations vs. explicitly deployer-owned concerns); each
of PRP-004, PRP-006, PRP-007 gained a **Security acceptance surface** subsection
naming what its DoD must prove, plus new owned **[DECIDE]** markers for the
genuinely open mechanism choices (URL allowlist config, MCP-server isolation, trace
retention/encryption hooks, redaction default, SBOM policy, authoring resource
limits). No existing hard rule weakened; no decision owned by another PRP resolved
ad hoc.

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
| Engine | [PRP-005 verification](PRP-005-guarantees-verification/01-objective.md) | ✅ Done (2026-09-03) | BDD checks + domain-term witnesses, five-field assumptions, non-vacuity |
| Surface | [PRP-003 python bindings](PRP-003-python-bindings/01-objective.md) | ✅ Done (2026-08-18) | `pip install horismos`; the Pythonic front door |
| Runtime | [PRP-004 agent loop](PRP-004-agent-loop/01-objective.md) | ✅ Done (2026-08-25, PR #8) | Workflow engine, perception/narration boundaries, tools, Bridge |
| Integration | [PRP-008 standards & host integration](PRP-008-standards-host-integration/01-objective.md) | ✅ Done (2026-09-03, PR #15) | VTE slice; FHIR hydration/commit, terminology provenance, CDS-Hooks/PlanDefinition/CQL composition, candidate-artifact boundary, generic host SPI |
| Ship & Author | [PRP-006 packaging](PRP-006-packaging-distribution/01-objective.md) | 🟦 Active (2026-09-21) | crates.io + PyPI + docs site + CI gates — PyPI release path landed first |
| Ship & Author | [PRP-007 authoring agent](PRP-007-authoring-agent.md) | ⬜ Draft (amended 2026-08-24) | Import/align/overlay-first; SMEs converse; Claude drafts Turtle; verifier gatekeeps |

Build order (dependency DAG, not strict number order — `PRPs/CONVENTIONS.md`):
001 → 002 → 003 → 004 → 005 → **008** → 006 → 007, with 005 startable right
after 002 (it deliberately doesn't need the agent loop), and 008 sequenced after 005
and before 006/007 despite its higher number.

## Quick recap — what exists today

- **Spec**: docs/framework 00–15 — concepts, ontology profile, world/gate, reasoning,
  workflows, LLM boundaries, tools, bridge, traces, verification, both APIs,
  authoring, architecture, security & trust.
- **Engine (PRP-001 + PRP-002)**: `horismos-core::ontology` (terms/interning,
  Turtle, profile validation, schema + subsumption, imports/anchoring),
  `horismos-core::world` (store, five-check gate, provenance, snapshots/diff/
  binary form, tier-1 queries), `horismos-core::reason` (rule model, 11
  operators, naive + Rete-lite matchers, forward/backward chaining, proof
  trees) and `horismos-core::trace` (`horismos.trace/1`), property-tested;
  fixtures `examples/triage/{triage,snomed-ct-subset}.ttl`.
- **Verification (PRP-005)**: `horismos-core::verify` — BDD rule/workflow checks with
  domain-term witnesses, the five-field assumption manifest, fail-closed loads in both
  APIs and the stamped `allow_incomplete` escape.
- **Integration (PRP-008)**: `horismos-host` (host SPI, candidate-artifact policy,
  assumptions coupling), `horismos-fhir` (FHIR R4 hydration/commit, CDS Hooks /
  PlanDefinition subset, the VTE slice), `horismos-harness` (coding-harness adapter).
- **Skeleton**: `horismos-bridges` (mock + gated provider stubs), `horismos-py`
  (version roundtrip), `python/horismos`, Makefile — all green.
- **Docs sites**: the developer site (`mkdocs.yml` + `docs/`, published to GitHub Pages
  by `.github/workflows/docs-pages.yml`) and the research site (`mkdocs-research.yml` +
  `research/`, `research-site/`).
- **Conventions**: CLAUDE.md hard rules; PRP lifecycle in
  [CONVENTIONS.md](CONVENTIONS.md).
- **Skills**: cargo-workspace-map, ontology-modeling-conventions, pyo3-boundary,
  learn-codebase, generate-prp, impl-prp, git-workflow, self-review,
  brainstorming + the senior-reviewer agent (+ global superpowers).
- **Prior art**: `../sigil-lang` — Rete-lite, BDD verifier, constraint solver, trace
  design, neural bridge, authoring loop (~1,270 tests to port from).
