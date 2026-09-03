# Horismos — working conventions

Horismos is an **ontology-first neuro-symbolic decision and capability kernel**: Rust
core, Python bindings. A small RDFS-plus overlay declares the rules, constraints, and
decision/workflow fragment a deployment needs; the runtime loads it, formally verifies
it, and evaluates it as a versioned projection over an existing system of record (not
as the system of record itself — see `PRPs/PRP-000-vision-positioning.md`). LLMs exist
only at declared transducer boundaries (perception: NL → typed instances; narration:
proof → NL). Delete the LLM and the kernel still runs.

**The two principles** (they resolve most design arguments):
1. Neural proposes, symbolic decides — no rule firing, workflow transition, or tool
   call is ever made by a model.
2. Traces are output, not logging — the audit log and the training corpus are the
   same object.

## Sources of truth

- `PRPs/BOARD.md`: at-a-glance state — current / next / end goal. Visual version:
  the [Horismos Board artifact](https://claude.ai/code/artifact/dc7ee691-4c9f-40a7-b235-0760de8ff17d)
  (regenerate from `planning/board.html`, republish to the same URL on status changes).
- `docs/framework/00-index.md` → docs 01–15: the framework definition, written in
  shipped voice (15 is the v1 security/trust threat model). **Read the relevant doc
  before implementing anything.**
- `PRPs/README.md`: phase plans. Open design questions are marked **[DECIDE]** in the
  docs, each owned by exactly one PRP — resolve them in the owning PRP, never ad hoc.
- `PRPs/foundational-skeleton.md`: this skeleton — layout, tooling, skills roster.
- `research/README.md`: the research wiki — reality-checks the design against the field,
  every claim tagged `[measured]`/`[inference]`; consult before resolving a `[DECIDE]`.
  `research/neurosymbolic-landscape/pivot-alignment.md` is the standing verdict on the
  bounded-projection pivot, re-issued each weekly sweep.
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
- **Loads fail closed**: `Ontology::load` (Rust) and `Ontology.load` (Python) verify
  and refuse an ontology with any error-severity finding. `LoadOptions { allow_incomplete }`
  / the `*_allow_incomplete` helpers / `allow_incomplete=True` are the only escape; it is
  stamped into every agent trace. Tests that build deliberately partial ontologies must
  use it. Spec: `docs/framework/10-verification.md`.
- **Truth maintenance**: derived facts are recomputed, never left stale — every
  external transaction (`assert_`, replace, `retract`) resets all standing
  derivations and re-chains to the fixed point of the resulting base facts, even
  through chained (derived-from-derived) support. A world built through the public
  API is therefore *always* at its fixed point — there is no way to construct one
  where a rule's conditions hold but its conclusion doesn't also stand. This
  means the old "derive a fact, then manually retract it with `Provenance::Rule`"
  trick for building an open goal to test `prove`/backward chaining no longer works
  (retraction re-derives it right back); use a higher-priority "spoiler" rule for
  the same property instead (see `reason/backward.rs`'s `spoiled_world` for the
  pattern). Spec: `docs/framework/03-world.md` § Truth maintenance; decision
  record: `PRPs/PRP-002-reasoning-substrate/03-decisions.md`.

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
- Two wire contracts are versioned forever — evolve them only per their written
  policies: the trace schema (`horismos-core::trace::migrate` module doc) and the
  snapshot binary form (`world/snapshot.rs` header consts; `Provenance` variant
  order is part of it).
- Process artifacts (ledgers, briefs, review diffs) are untracked scratch
  (`.superpowers/` is gitignored); the durable record is the owning PRP's
  `02-tasks-and-gaps.md` and the shipped-voice text where `[DECIDE]` markers were
  struck.

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
