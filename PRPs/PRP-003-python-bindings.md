# PRP-003 — Python Bindings & API

**Status**: Draft
**Depends on**: PRP-001, PRP-002

## Intent

The developer front door: `pip install horismos`, a Pythonic API over the Rust core via
PyO3/maturin — the packaging pattern already proven in sigil-lang.

## Scope

- **Package** `horismos`: `Ontology`, `World`, `Rule` handles, query results as Python
  objects; zero-copy where PyO3 allows, marshal at the boundary otherwise (sigil-lang's
  `HorismosValue::from_python/to_python` experience applies directly).
- **API design** (per `docs/framework/11-python-api.md`):
  - `Ontology.load(path, imports=[...])`, `.verify()` (stub until PRP-005), `.namespaces`
  - `World(onto)`: `.assert_()`, `.retract()`, `.query(sparql)`, `.explain(pattern)`,
    `.prove(goal)`, `.snapshot()`, `.subscribe(pattern, callback)`
  - Typed instance construction: `Patient(heart_rate=128, symptoms=[ChestPain])` —
    generated proxy classes from the loaded ontology (the "feels native in Python" bet).
  - Errors as a `HorismosError` hierarchy with domain-term messages and fix suggestions
    (sigil-lang's error-experience conventions carry over).
- **Wheels**: maturin, abi3, CI matrix (Linux/macOS/Windows × 3.11–3.14).
- Type stubs (`.pyi`) and mypy-clean surface.

## Non-goals

- No agent/workflow surface yet (PRP-004 extends this package).
- No async API in v1 (revisit with the agent loop).

## Key decisions to make

1. Generated proxy classes vs. dict-like generic instances — ergonomics vs. magic;
   prototype both against the triage example.
2. SPARQL subset exposure vs. a Pythonic query builder (or both, builder compiling to
   the subset).
3. GIL strategy for long inference runs (`py.allow_threads` around chaining).

## Definition of done

`pip install horismos` (local wheel) → the docs' Python quickstart for triage runs verbatim
with structured input (no LLM); pytest suite exercises every public symbol; stubs pass
mypy; wheels build in CI for the full matrix.
