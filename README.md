# Horismos

**Ontology-first neuro-symbolic decision and capability kernel. Rust core, Python bindings.**

A subject-matter expert authors one artifact — a small RDFS-plus overlay declaring the
rules, constraints, and decision/workflow fragment a deployment needs — and the runtime
evaluates it as a verified, versioned projection over an existing system of record (see
`PRPs/PRP-000-vision-positioning.md`). LLMs exist only at two declared transducer
boundaries (perception: NL → typed instances; narration: proof → NL). Every conclusion
carries a proof tree; every rule set is formally verified before it runs. Delete the LLM
and the kernel still runs.

**Neural proposes. Symbolic decides.**

## Status

Pre-alpha: the symbolic core (PRP-001/002), the Python bindings (PRP-003) and the Rust
agent loop (PRP-004) are merged; verification (PRP-005) and standards/host integration
(PRP-008) are in build on branches — see `PRPs/BOARD.md`.

- `docs/framework/` — the framework definition (docs 00–15), written in shipped voice
- `PRPs/` — phase plans (PRP-000 vision → PRP-008 standards & host integration)
- `AGENTS.md` — working conventions for agentic development sessions (`CLAUDE.md` is
  a compatibility symlink)

## Layout

| Path | What |
|------|------|
| `crates/horismos-core` | The symbolic engine: ontology, world, reasoner, workflows, traces, verification — zero LLM deps |
| `crates/horismos-bridges` | Bridge implementations: provider clients (feature-gated), mock |
| `crates/horismos-py` | PyO3 extension module — marshaling only, no logic |
| `python/horismos` | The `horismos` Python package: pure-Python surface over the native module |
| `examples/triage` | The running example: clinical triage (`triage.ttl`) |

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
MkDocs Material site with sources in `docs/` and config at the repo root:

```sh
uvx --with mkdocs-material mkdocs serve            # http://127.0.0.1:8000
uvx --with mkdocs-material mkdocs build --strict   # the docs gate
```

Every runnable snippet on the site's own pages is executed by
`python/tests/test_devdocs_snippets.py` (the framework reference's Python API page is exercised
by `python/tests/test_quickstart.py`); conventions live in `docs/README.md`.

## License

MIT OR Apache-2.0 (final license decision tracked in PRP-000).
