# PRP-006 — Packaging, Distribution & Docs

**Status**: Draft
**Depends on**: PRP-003, PRP-004, PRP-005

## Intent

Make Horismos installable, documented, and credible as a product: crates.io + PyPI +
docs site + examples, with CI that enforces the anti-wrapper guarantee.

## Scope

- **Crates**: `horismos-core` (no-LLM symbolic core), `horismos-bridges` (provider SDKs,
  feature-gated), `horismos-py` (PyO3 layer). Feature flags follow sigil-lang's proven
  `pyo3-ext`/`wasm` isolation pattern.
- **PyPI**: `horismos` wheels via maturin-action; abi3; full OS × Python matrix.
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
3. Name availability on crates.io/PyPI for `horismos` (likely taken — decide fallback:
   `horismos-core`/`horismos-ai`/`horismosframe`) — feeds back into PRP-000 naming.
4. **[DECIDE, doc 15 §4 / doc 09]** Default redaction posture (opt-in vs. opt-out)
   for `horis:redactInput`, per domain pack.
5. **[DECIDE, doc 15 §4]** Encryption-at-rest and access-control hooks for the
   default trace store — a minimal `TraceStore` wrapper interface vs. fully
   deployer-owned infra.
6. **[DECIDE, doc 15 §4]** SBOM format (CycloneDX vs. SPDX) and whether the
   dependency-audit gate blocks a release or only warns.

## Definition of done

`cargo add horismos-core` and `pip install <name>` work from public registries;
docs site deploys from `docs/framework/`; all examples run in CI with the mock Bridge;
a newcomer reproduces the triage quickstart from docs alone.

## Security acceptance surface (doc 15)

PRP-006 owns the trace-retention/privacy and supply-chain trust boundaries. Its DoD
is not met until it also proves:

- **Redaction is enforced at emission, not scrubbed after the fact**: a run with
  `horis:redactInput true` produces a trace containing no raw NL anywhere in the
  document — including nested per-step perception input, not just the top-level
  `input.text` field.
- **Trace store privacy posture is documented, not implicit**: the shipped JSONL
  default store's lack of built-in encryption-at-rest/access-control is stated in the
  docs site as deployer-owned (doc 15 §4), and whatever hook is decided by D5 above
  ships with a test.
- **Dependency audit runs as a CI gate**: `cargo audit`/`cargo deny` and a Python
  equivalent (e.g. `pip-audit`) fail the build on a known-vulnerable dependency per
  D6's chosen strictness; an SBOM is generated per release.
- **Anti-wrapper boundary doubles as a supply-chain control**: CI proves
  `horismos-core` has zero dependency edge into `horismos-bridges` or any provider
  SDK (already enforced by `make check-boundary`/`no-model-gate` — this DoD item is
  that no *new* release process or packaging change weakens it).
