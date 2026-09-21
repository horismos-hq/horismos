# Contributing to Horismos

Working conventions for agentic development sessions live in `AGENTS.md` (`CLAUDE.md` is
a compatibility symlink). The framework definition is `docs/framework/` (docs 00–15),
written in shipped voice; `PRPs/` holds the phase plans (PRP-000 vision → PRP-008
standards & host integration) and `PRPs/BOARD.md` is the at-a-glance current, next, and
shipped state.

## Layout

| Path | What |
|------|------|
| `crates/horismos-core` | The symbolic engine: ontology, world, reasoner, workflows, traces, verification — zero LLM deps |
| `crates/horismos-host` | Format-neutral host SPI (`ExternalSnapshot`, `ProposedEffectSet`, `HostAdapter`) and the candidate-artifact runtime policy |
| `crates/horismos-fhir` | Bounded FHIR R4 adapter: hydration, mapping, conditional commit, CDS Hooks / PlanDefinition subset |
| `crates/horismos-harness` | Coding-harness adapter: recorded repository projection, candidate-artifact handoff |
| `crates/horismos-bridges` | Bridge implementations: provider clients (feature-gated), mock |
| `crates/horismos-py` | PyO3 extension module — marshaling only, no logic |
| `python/horismos` | The `horismos` Python package: pure-Python surface over the native module |
| `examples/triage` | The running example: clinical triage (`triage.ttl`) |
| `examples/vte` | The standards-anchored vertical slice: VTE prophylaxis over a recorded FHIR bundle (PRP-008) |

## Development

```sh
make build     # cargo build --workspace
make develop   # build + install the Python package into .venv (uv + maturin)
make test      # Rust tests + pytest
make lint      # fmt check, clippy, ruff, mypy
make verify    # everything above + the anti-wrapper no-model-gate
```

## Documentation site

The developer-facing docs (landing page, zero-key quickstart, "coming from LangGraph",
the coding-agent change gate, a use-case gallery, and the framework reference) are an
MkDocs Material site with sources in `docs/` and config at the repo root, published at
<https://docs.horismos.ai/> (deployed from `main` by `.github/workflows/docs-pages.yml`):

```sh
uvx --with mkdocs-material mkdocs serve            # http://127.0.0.1:8000
uvx --with mkdocs-material mkdocs build --strict   # the docs gate
```

Every runnable snippet on the site's own pages is executed by
`python/tests/test_devdocs_snippets.py` (the framework reference's Python API page is exercised
by `python/tests/test_quickstart.py`); conventions live in `docs/README.md`.

## License

MIT OR Apache-2.0 (final license decision tracked in PRP-000).
