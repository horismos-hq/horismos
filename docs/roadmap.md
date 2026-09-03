# Roadmap and status

Truthful to `PRPs/BOARD.md`, which is the single source of truth for status and is updated in the same commit as any change. This page summarizes; when they disagree, the board wins. Last reconciled: 2026-09-03.

## At a glance

| Track | Phase plan | Status | What it delivers |
|---|---|---|---|
| Standing | PRP-000 vision & positioning | <span class="horis-status-merged">Decisions closed</span> | The pitch, personas, honest placement (Kautz Type 2–3), naming, dual license, τ-bench airline as the public benchmark; bounded-projection pivot adopted 2026-08-24 |
| Engine | PRP-001 ontology core | <span class="horis-status-merged">Merged</span> (2026-07-29) | Turtle load, RDFS-plus profile, typed World, five-check gate, snapshots, queries |
| Engine | PRP-002 reasoning substrate | <span class="horis-status-merged">Merged</span> (2026-08-05; truth-maintenance fix 2026-08-19) | Forward/backward chaining, subsumption, proof trees, `horismos.trace/1` |
| Surface | PRP-003 Python bindings | <span class="horis-status-merged">Merged</span> (2026-08-18) | The `horismos` package: `Ontology`, `World`, snapshots, subscriptions, the error hierarchy, `mypy --strict` stubs |
| Runtime | PRP-004 agent loop | <span class="horis-status-merged">Critical path merged</span> (PR #8) — follow-ups open | Workflow interpreter, tool contracts and effect validation, `MockBridge`, `OpenRouterBridge`, `horismos.trace/2`, the live triage demo |
| Engine | PRP-005 guarantees & verification | <span class="horis-status-building">Draft on the board; in build on branches</span> | BDD checks with domain-term witnesses, assumption manifest, non-vacuity, `verify()` in both APIs |
| Integration | PRP-008 standards & host integration | <span class="horis-status-building">Draft on the board; in build on branches</span> (drafted 2026-08-24; sequenced after 005) | FHIR hydration and conditional commit, terminology provenance, CDS-Hooks-style surface, bounded PlanDefinition import, the candidate-artifact boundary, the generic host SPI and a coding-harness adapter |
| Ship | PRP-006 packaging & distribution | <span class="horis-status-planned">Planned</span> (after 008) | crates.io and PyPI releases, CI gates (including this docs site's `--strict` build and the mock-bridge zero-key suite), wheel matrix, SBOM |
| Author | PRP-007 authoring agent | <span class="horis-status-planned">Planned</span> (after 008) | Import/align/overlay-first authoring with the verifier as gatekeeper; the capability-fragment compiler mode |

Build order is a dependency DAG, not the numbering: 001 → 002 → 003 → 004 → 005 → **008** → 006 → 007.

## What you can call today

| Surface | Available now | Notes |
|---|---|---|
| Python: `Ontology.load`, `World.assert_` / `retract` / `query` / `value(s)` / `instances`, `explain`, `prove`, `snapshot` / `diff` / wire form, `subscribe`, the `HorismosError` hierarchy | **Yes** — every snippet on this site runs against it | `Ontology.verify()` returns a stub report until PRP-005 |
| Rust: the same kernel plus `workflow::Agent` (perception → reasoning → action → narration), `Tool` trait, `Bridge` trait, `MockBridge`, `OpenRouterBridge` | **Yes** — `make demo`, `make demo-live`, `crates/horismos-bridges/tests/agent_loop_dod.rs` | `ChoiceStep`, `ProposalStep`, `WaitStep`, `SubflowStep` parse and are traced but execution is deferred (`StepDeferred`) |
| Python `Agent.run(...)` | Not yet | PRP-004 follow-up (U-Python) now that PRP-003 is merged |
| `python:`, `http(s):`, `mcp://` tool bindings | Not yet | Only `rust:` bindings ship; URL and MCP allowlist mechanics are open decisions (PRP-004 D14/D15) |
| Live perception with multi-valued properties | Partial | See the quickstart's live-path note; the mock path is the reference behaviour |
| Verification witnesses, assumption manifest | Not yet | PRP-005 |
| FHIR / repository hydration, conditional commit, CDS Hooks surface | Not yet | PRP-008 |
| `pip install horismos` / `cargo add horismos-core` from public registries | Not yet | PRP-006; today: `make develop` from a clone |
| This docs site published at a public URL | Not yet | PRP-006 wires the CI build; today: `uvx --with mkdocs-material mkdocs serve` |

## The end state

The chained definitions of done, quoted from the board:

> `pip install horismos` / `cargo add horismos-core` from public registries. A clinician produces a formally verified `triage.ttl` **through conversation alone** — the authoring agent drafts Turtle, the BDD verifier gatekeeps with domain-term witnesses, the SME signs off on rendered rules, never syntax. Then `agent.run("58M, crushing chest pain…")` executes that workflow: one perception call at a declared boundary, deterministic reasoning with proof trees, tools fired by the runtime, a replayable training-grade trace — and the whole thing still runs with the LLM deleted. Docs site live, mock-bridge CI with zero keys, benchmark number public.

Beyond v1, as research rather than a phase: trace fine-tuning, provenance semirings over the Datalog-shaped rule subset, GNN guidance over the instance graph.

## Follow along

- `PRPs/BOARD.md` — the board, current state, next goal, human side quests.
- `PRPs/README.md` — reading order and the dependency-DAG rule.
- `research/README.md` — the research wiki; `research/neurosymbolic-landscape/pivot-alignment.md` is re-verdicted every weekly sweep.
- Open decisions are marked **[DECIDE]** in the framework docs, each owned by exactly one PRP and resolved there, never ad hoc.
