# Horismos — working conventions

Horismos is an **ontology-first neuro-symbolic agent framework**: Rust core, Python
bindings. One authored artifact — an RDFS-plus ontology declaring the domain, its
rules, and its workflows — is loaded, formally verified, and executed. LLMs exist only
at declared transducer boundaries (perception: NL → typed instances; narration:
proof → NL). Delete the LLM and the framework still runs.

**The two principles** (they resolve most design arguments):
1. Neural proposes, symbolic decides — no rule firing, workflow transition, or tool
   call is ever made by a model.
2. Traces are output, not logging — the audit log and the training corpus are the
   same object.

## Sources of truth

- `PRPs/BOARD.md`: at-a-glance state — current / next / end goal. Visual version:
  the [Horismos Board artifact](https://claude.ai/code/artifact/dc7ee691-4c9f-40a7-b235-0760de8ff17d)
  (regenerate from `planning/board.html`, republish to the same URL on status changes).
- `docs/framework/00-index.md` → docs 01–14: the framework definition, written in
  shipped voice. **Read the relevant doc before implementing anything.**
- `PRPs/README.md`: phase plans. Open design questions are marked **[DECIDE]** in the
  docs, each owned by exactly one PRP — resolve them in the owning PRP, never ad hoc.
- `PRPs/foundational-skeleton.md`: this skeleton — layout, tooling, skills roster.
- Prior art: `../sigil-lang` (sibling repo) — the proving ground this framework
  inherits from; PRPs cite what to port.

## Hard rules

- **Anti-wrapper boundary**: `horismos-core` never depends on `horismos-bridges`, provider
  SDKs, or the network. The `Bridge` trait lives in `horismos-core::bridge` as a pure
  interface; implementations live in `horismos-bridges`. `make check-boundary` and
  `make no-model-gate` enforce this — never weaken them.
- **No logic in `horismos-py`**: marshaling only. Whole inputs in, whole outcomes out —
  no cross-language chatter inside a transaction.
- **Determinism**: no wall-clock, randomness, or iteration-order dependence anywhere
  in inference. `horis:now` is transaction-stamped. Trace equality is a valid test
  assertion.
- **Errors speak domain terms**: every error carries `domain_text()` in ontology
  vocabulary ("heartRate 1280 exceeds horis:max 300 (bpm) on :Patient"), plus a
  machine-readable structure.
- **The validation gate has no back door**: every World mutation passes it,
  regardless of provenance (perception | rule | tool | api).

## Commands

Always use the Makefile (identical for humans, CI, and agent sessions):

```
make build         # cargo build --workspace
make develop       # uv sync + maturin develop (builds the extension into .venv)
make test          # Rust tests + pytest
make lint          # cargo fmt --check, clippy -D warnings, ruff, mypy --strict
make fmt           # auto-format both languages
make no-model-gate # horismos-core --no-default-features --features verify, build+test
make check-boundary
make verify        # all of the above — run before claiming work done
```

`make verify` must be green before any commit that claims a task complete.

## Conventions

- Rust: edition 2024, MSRV = latest stable − 2, `unsafe_code = "deny"`,
  clippy warnings are errors in CI. Errors are `thiserror`-derived.
- Python: 3.11+, `mypy --strict` clean surface, ruff for lint+format, abi3 wheels
  via maturin. Type stubs (`.pyi`) updated in the same change as the native surface.
- Testing: property tests (proptest on the Rust side, Hypothesis on the Python side)
  for gate/inference invariants; the triage example (`examples/triage/`) must
  exercise every shipped feature.
- Docs are written in shipped voice; open questions are marked **[DECIDE]** with the
  owning PRP referenced. Don't silently decide them in code.
- The `horis:` vocabulary, the 11 condition operators, and the RDFS-plus profile are
  defined in `docs/framework/02-ontology.md` — never invent vocabulary terms ad hoc.

## Project skills (.claude/skills/) and agents (.claude/agents/)

Domain skills — consult before touching the matching surface:

- `cargo-workspace-map` — which crate/module owns what; consult before placing code.
- `ontology-modeling-conventions` — the `horis:` vocabulary, IRI scheme, operators;
  consult before writing or editing any `.ttl`.
- `pyo3-boundary` — Rust↔Python boundary conventions; consult before touching
  `horismos-py` or `python/horismos`.
- `learn-codebase` — guided-tour/onboarding artifact for newcomers.

Process skills — the PRP workflow itself:

- `generate-prp` — drafting a new PRP (lifecycle, D/U-items, [DECIDE] links, board).
- `impl-prp` — executing an Active PRP (worktree, ledger, batching, briefs,
  reviews, finish sequence). Use this to start or resume any PRP build.
- `git-workflow` — branches, worktrees, commit rules, PR flow for this repo.
- `self-review` — pre-report audit checklist (determinism, boundary, errors, gate).
- `brainstorming` — research-aware protocol for resolving [DECIDE] questions;
  never decide them ad hoc.

Agents:

- `senior-reviewer` — diff/batch reviewer with this repo's severity calibration;
  used per batch and for final whole-branch reviews in `impl-prp`.

## Maintaining this file

Keep this file for knowledge useful to almost every future agent session in this project.
Do not repeat what the codebase already shows; point to the authoritative file or command instead.
Prefer rewriting or pruning existing entries over appending new ones.
When updating this file, preserve this bar for all agents and keep entries concise.
