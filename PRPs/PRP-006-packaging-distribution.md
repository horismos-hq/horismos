# PRP-006 — Packaging, Distribution & Docs

**Status**: Draft
**Depends on**: PRP-003, PRP-004, PRP-005

## Intent

Make Sigil installable, documented, and credible as a product: crates.io + PyPI +
docs site + examples, with CI that enforces the anti-wrapper guarantee.

## Scope

- **Crates**: `sigil-core` (no-LLM symbolic core), `sigil-bridges` (provider SDKs,
  feature-gated), `sigil-py` (PyO3 layer). Feature flags follow sigil-lang's proven
  `pyo3-ext`/`wasm` isolation pattern.
- **PyPI**: `sigil` wheels via maturin-action; abi3; full OS × Python matrix.
- **Docs site**: `docs/framework/` (already authored as the definition) becomes the
  published documentation — mdBook or MkDocs Material; API reference generated
  (rustdoc + mkdocstrings); the triage walkthrough as the tutorial.
- **Examples**: triage (flagship), one no-LLM example (structured facts only — the
  anti-wrapper demo), one MCP-tools example, one τ-Bench harness scaffold.
- **CI gates**: fmt/clippy/ruff/mypy; full test suite with mock Bridge only (zero API
  keys in CI — the "symbolic core runs with no model configured" claim enforced
  mechanically); wheel builds; docs build; benchmark regression tracking.
- Versioning & release: semver pre-1.0 discipline, CHANGELOG, tagged releases.

## Non-goals

- No hosted service, no telemetry, no marketing site beyond the docs landing page
  (PRP-000 owns positioning copy).

## Key decisions to make

1. Repo layout: single workspace repo (lean) vs. split repos.
2. Docs tool (mdBook vs. MkDocs Material) — pick for versioned docs + Python API refs.
3. Name availability on crates.io/PyPI for `sigil` (likely taken — decide fallback:
   `sigil-core`/`sigil-ai`/`sigilframe`) — feeds back into PRP-000 naming.

## Definition of done

`cargo add sigil-core` and `pip install <name>` work from public registries;
docs site deploys from `docs/framework/`; all examples run in CI with the mock Bridge;
a newcomer reproduces the triage quickstart from docs alone.
