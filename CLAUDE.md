# Sigil — working conventions

Sigil is an **ontology-first neuro-symbolic agent framework**: Rust core, Python
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
  the [Sigil Board artifact](https://claude.ai/code/artifact/95d5a1ed-95cf-41b2-8b38-f51404f79b0b)
  (regenerate from `planning/board.html`, republish to the same URL on status changes).
- `docs/framework/00-index.md` → docs 01–14: the framework definition, written in
  shipped voice. **Read the relevant doc before implementing anything.**
- `PRPs/README.md`: phase plans. Open design questions are marked **[DECIDE]** in the
  docs, each owned by exactly one PRP — resolve them in the owning PRP, never ad hoc.
- `PRPs/foundational-skeleton.md`: this skeleton — layout, tooling, skills roster.
- Prior art: `../sigil-lang` (sibling repo) — the proving ground this framework
  inherits from; PRPs cite what to port.

## Hard rules

- **Anti-wrapper boundary**: `sigil-core` never depends on `sigil-bridges`, provider
  SDKs, or the network. The `Bridge` trait lives in `sigil-core::bridge` as a pure
  interface; implementations live in `sigil-bridges`. `make check-boundary` and
  `make no-model-gate` enforce this — never weaken them.
- **No logic in `sigil-py`**: marshaling only. Whole inputs in, whole outcomes out —
  no cross-language chatter inside a transaction.
- **Determinism**: no wall-clock, randomness, or iteration-order dependence anywhere
  in inference. `sigil:now` is transaction-stamped. Trace equality is a valid test
  assertion.
- **Errors speak domain terms**: every error carries `domain_text()` in ontology
  vocabulary ("heartRate 1280 exceeds sigil:max 300 (bpm) on :Patient"), plus a
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
make no-model-gate # sigil-core --no-default-features --features verify, build+test
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
- The `sigil:` vocabulary, the 11 condition operators, and the RDFS-plus profile are
  defined in `docs/framework/02-ontology.md` — never invent vocabulary terms ad hoc.

## Project skills (.claude/skills/)

- `cargo-workspace-map` — which crate/module owns what; consult before placing code.
- `ontology-modeling-conventions` — the `sigil:` vocabulary, IRI scheme, operators;
  consult before writing or editing any `.ttl`.
- `pyo3-boundary` — Rust↔Python boundary conventions; consult before touching
  `sigil-py` or `python/sigil`.
