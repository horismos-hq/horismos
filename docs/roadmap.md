# Roadmap and status

This page is the published status of record. It is reconciled against the project board — the internal single source of truth, updated in the same commit as any change it reports — so that what you read here is what is actually merged, in build, or planned. Where the two ever drift, the board is authoritative.

## At a glance

| Track | Phase plan | Status | What it delivers |
|---|---|---|---|
| Standing | PRP-000 vision & positioning | <span class="horis-status-merged">Decisions closed</span> | The pitch, personas, honest placement (Kautz Type 2–3), naming, dual license, τ-bench airline as the public benchmark; bounded-projection pivot adopted 2026-08-24 |
| Engine | PRP-001 ontology core | <span class="horis-status-merged">Merged</span> (2026-07-29) | Turtle load, RDFS-plus profile, typed World, five-check gate, snapshots, queries |
| Engine | PRP-002 reasoning substrate | <span class="horis-status-merged">Merged</span> (2026-08-05; truth-maintenance fix 2026-08-19) | Forward/backward chaining, subsumption, proof trees, `horismos.trace/1` |
| Surface | PRP-003 Python bindings | <span class="horis-status-merged">Merged</span> (2026-08-18) | The `horismos` package: `Ontology`, `World`, snapshots, subscriptions, the error hierarchy, `mypy --strict` stubs |
| Runtime | PRP-004 agent loop | <span class="horis-status-merged">Critical path merged</span> (PR #8) — follow-ups open | Workflow interpreter, tool contracts and effect validation, `MockBridge`, `OpenRouterBridge`, `horismos.trace/2`, the live triage demo |
| Engine | PRP-005 guarantees & verification | <span class="horis-status-merged">Merged</span> (2026-09-03) | BDD checks with domain-term witnesses, assumption manifest, non-vacuity, `verify()` in both APIs; loads fail closed on any error-severity finding |
| Integration | PRP-008 standards & host integration | <span class="horis-status-merged">Merged</span> (2026-09-03, PR #15) | FHIR hydration and conditional commit, terminology provenance, CDS-Hooks-style surface, bounded PlanDefinition import, the candidate-artifact boundary, the generic host SPI and a coding-harness adapter |
| Ship | PRP-006 packaging & distribution | <span class="horis-status-planned">Planned</span> (after 008) | crates.io and PyPI releases, CI gates (including this docs site's `--strict` build and the mock-bridge zero-key suite), wheel matrix, SBOM |
| Author | PRP-007 authoring agent | <span class="horis-status-planned">Planned</span> (after 008) | Import/align/overlay-first authoring with the verifier as gatekeeper; the capability-fragment compiler mode |

Build order is a dependency DAG, not the numbering: 001 → 002 → 003 → 004 → 005 → **008** → 006 → 007.

## What you can call today

| Surface | Available now | Notes |
|---|---|---|
| Python: `Ontology.load`, `World.assert_` / `retract` / `query` / `value(s)` / `instances`, `explain`, `prove`, `snapshot` / `diff` / wire form, `subscribe`, the `HorismosError` hierarchy | **Yes** — every runnable snippet on this site's guide pages runs against it | `Ontology.verify()` returns a full report; `Ontology.load` refuses an ontology with any error-severity witness unless `allow_incomplete=True` |
| Rust: the same kernel plus `workflow::Agent` (perception → reasoning → action → narration), `Tool` trait, `Bridge` trait, `MockBridge`, `OpenRouterBridge` | **Yes** — `make demo`, `make demo-live`, `crates/horismos-bridges/tests/agent_loop_dod.rs` | `ChoiceStep`, `ProposalStep`, `WaitStep`, `SubflowStep` parse and are traced but execution is deferred (`StepDeferred`) |
| Python `Agent.run(...)` | Not yet | PRP-004 follow-up (U-Python) now that PRP-003 is merged |
| `python:`, `http(s):`, `mcp://` tool bindings | Not yet | Only `rust:` bindings ship; URL and MCP allowlist mechanics are open decisions (PRP-004 D14/D15) |
| Live perception with multi-valued properties | **Yes** — a real `make demo-live` run reaches `:Done` with the `r1` proof ([Go live in 2 minutes](go-live.md)) | Array-valued perception answers were dropped until 2026-09-03; regression tests in `crates/horismos-core/src/workflow/agent.rs` |
| Verification witnesses, assumption manifest | **Yes** | PRP-005 |
| FHIR / repository hydration, conditional commit, CDS Hooks surface | **Yes** — `crates/horismos-fhir` (`FhirHydrationRequest`, the conditional-commit types, the CDS Hooks card module) over the generic host SPI in `crates/horismos-host`; `examples/vte` runs the VTE slice over a recorded bundle | PRP-008 (PR #15). Structured-input adapters against recorded fixtures — no live EHR connection ships |
| `pip install horismos` / `cargo add horismos-core` from public registries | Not yet | PRP-006; today: `make develop` from a clone |
| This docs site published at a public URL | **Yes** — <https://docs.horismos.ai/>, deployed from `main` by `.github/workflows/docs-pages.yml` | Deploy-on-merge only; PR-triggered CI (this site's `--strict` build as a check) is still PRP-006 |

## The end state

The chained definitions of done, quoted from the board:

> `pip install horismos` / `cargo add horismos-core` from public registries. A clinician produces a formally verified `triage.ttl` **through conversation alone** — the authoring agent drafts Turtle, the BDD verifier gatekeeps with domain-term witnesses, the SME signs off on rendered rules, never syntax. Then `agent.run("58M, crushing chest pain…")` executes that workflow: one perception call at a declared boundary, deterministic reasoning with proof trees, tools fired by the runtime, a replayable training-grade trace — and the whole thing still runs with the LLM deleted. Docs site live, mock-bridge CI with zero keys, benchmark number public.

Beyond v1, as research rather than a phase: trace fine-tuning, provenance semirings over the Datalog-shaped rule subset, GNN guidance over the instance graph.

## Follow along

- This page is reconciled against the board on every status change — the date under the
  title tells you how fresh it is.
- The [Framework reference](framework/00-index.md) is the specification the phase plans
  build against; each phase plan owns a slice of it, and the tracks above say which.
- Design questions still open are marked **[DECIDE]** in the framework docs. Each is
  owned by exactly one phase plan and resolved there, never ad hoc — so a **[DECIDE]**
  you find in the reference is a live question, not an oversight.
- Every design commitment is re-assessed against new literature on a weekly research
  sweep; the standing verdict on the bounded-projection design is summarised on
  [Guarantees](guarantees.md) § "Why we think the shape is right".
