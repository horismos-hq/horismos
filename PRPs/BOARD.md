# Horismos Board

At-a-glance project state. Updated in the same commit as any PRP status change
(`CONVENTIONS.md`). Visual version:
[Horismos Board artifact](https://claude.ai/code/artifact/dc7ee691-4c9f-40a7-b235-0760de8ff17d)
(source: `planning/board.html`, republished to the same URL on updates).

_Last updated: 2026-09-03 (PRP-008 completed on its isolated branch; VTE prophylaxis,
the generic host SPI, candidate-artifact pipeline, and FHIR/harness adapters pass the
full local verification gate). Previously: 2026-09-02 (PRP-008 activated in parallel
with PRP-005; VTE prophylaxis confirmed and D1–D10 resolved). Previously: 2026-08-24
(bounded-projection-kernel pivot adopted; new PRP-008 drafted). Previously: 2026-08-19 (PRP-004's
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
built or verified). `PRPs/PRP-008-standards-host-integration/` is Done on its isolated
branch: VTE prophylaxis now exercises the bounded projection end to end, with the
five-field PRP-005 assumption coupling isolated for reconciliation when that parallel
verifier lane lands. PRP-008 remains sequenced before PRP-006/007.

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
config (execution is PRP-006's). 70 pytest + ~248 Rust tests, `make verify`
green. Branch: `fm/horismos-prp003-a1`. PRP-002 was recorded Done here
2026-08-18 (merged 2026-08-05 as PR #4; folder record in PR #3, open).
**Former captain hold, now resolved**: derived-fact truth maintenance (plan
review, finding 2) — fixed on `main` (PR #7, 2026-08-19, recompute/invalidate
on retraction); PRP-003 already bound current behavior without endorsing it,
so nothing there needed to change.

## Next goal

**PRP-008 (standards & host integration) is Done** (2026-09-03, branch
`fm/horismos-prp008-a1`). The generic host SPI, candidate-artifact runtime boundary,
FHIR adapter, CDS Hooks/PlanDefinition subset, VTE slice, and coding-harness proof all
ship together. PRP-005 coupling remains isolated to its provisional five-field
`report.assumptions` shape for reconciliation when that parallel lane lands; PRP-006 is
the next phase unlocked by this integration surface.

**PRP-004 (agent loop) is Active** (2026-08-18, branch `fm/horismos-prp004-a1`),
implementing the workflow engine, Bridge, and tools against merged `main`
directly (not against PRP-003's now-Done Python surface, per its own
coordination note) — with **PRP-005 (verification) startable in parallel**
(it needs only 001+002). PRP-004 inherits the pinned 003/004 subscription
split (core transactional notification stage), the `Ontology.version`/`hash`
deferral (trace stamping is its consumer), and doc 11's Agent surface. An
independent review (`data/horismos-prp-review-s1/report.md`) found the
planning surface a phase behind merged code at the time and flagged the
truth-maintenance defect above (since fixed) and an open trace-schema
question for workflow step kinds beyond perception/reasoning/action/
narration (since resolved as D13 in PRP-004's `02-tasks-and-gaps.md` — mint
`horismos.trace/2`) — both tracked there rather than fixed ad hoc.

**Update 2026-08-19**: the U7–U15 critical path (Agent interpreter, trace/2
emission wiring, tool-effect validation, the live triage demo, the DoD test
module) is implemented, the U9/U12 contractor branch is integrated, and
[PR #8](https://github.com/billaram/horismos/pull/8) is open against `main`,
pending human review — `make verify` green throughout. Two open decisions
(D14/D15: URL allowlist and MCP-server allowlist mechanics for future
`http:`/`https:`/`mcp:` tool bindings) surfaced from the doc-15 security
workstream landing on `main` mid-branch and are gap-noted in
`02-tasks-and-gaps.md`, not blocking — this PRP ships only `rust:` bindings.
Also recorded there, not silently absorbed: the `:EscalateToNurse`
case-subject gap (a low-confidence perception fallback has no case to page
against — a future design decision, not a bug in the runtime's refusal to
guess), and that this repo has no CI wired to run on pull requests yet
(`.github/workflows/wheels.yml` is `workflow_dispatch`-only by design,
PRP-006's to wire up) — the PR's validation bar is review + `make verify`
until then.

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
| Engine | [PRP-005 verification](PRP-005-guarantees-verification.md) | ⬜ Draft | BDD checks + domain-term witnesses (needs only 001+002) |
| Surface | [PRP-003 python bindings](PRP-003-python-bindings/01-objective.md) | ✅ Done (2026-08-18) | `pip install horismos`; the Pythonic front door |
| Runtime | [PRP-004 agent loop](PRP-004-agent-loop/01-objective.md) | 🟦 Active (2026-08-18) | Workflow engine, perception/narration boundaries, tools, Bridge |
| Integration | [PRP-008 standards & host integration](PRP-008-standards-host-integration/01-objective.md) | ✅ Done (2026-09-03) | VTE slice; FHIR hydration/commit, CDS-Hooks/PlanDefinition/CQL composition, candidate-artifact boundary, generic host SPI |
| Ship & Author | [PRP-006 packaging](PRP-006-packaging-distribution.md) | ⬜ Draft | crates.io + PyPI + docs site + CI gates — activates after PRP-008 |
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
- **Skeleton**: `horismos-bridges` (mock + gated provider stubs), `horismos-py`
  (version roundtrip), `python/horismos`, Makefile — all green.
- **Conventions**: CLAUDE.md hard rules; PRP lifecycle in
  [CONVENTIONS.md](CONVENTIONS.md).
- **Skills**: cargo-workspace-map, ontology-modeling-conventions, pyo3-boundary,
  learn-codebase, generate-prp, impl-prp, git-workflow, self-review,
  brainstorming + the senior-reviewer agent (+ global superpowers).
- **Prior art**: `../sigil-lang` — Rete-lite, BDD verifier, constraint solver, trace
  design, neural bridge, authoring loop (~1,270 tests to port from).
