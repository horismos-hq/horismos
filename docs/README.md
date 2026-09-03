# Horismos developer docs (site sources)

This directory is the source tree for the developer-facing MkDocs Material site.
`docs/framework/` is the shipped-voice framework definition (docs 00–15) and is
mounted into the site unchanged as the "Framework reference" section — edit those
files under their own conventions, never for the site's sake.

This file is excluded from the build (`exclude_docs` in `mkdocs.yml`).

## Serve and build

From the repository root:

```sh
uvx --with mkdocs-material mkdocs serve            # http://127.0.0.1:8000, live reload
uvx --with mkdocs-material mkdocs build --strict   # the gate: warnings are errors
```

`mkdocs.yml` lives at the repo root; the build output goes to `site/` (gitignored).

## Conventions

- **Runnable code is tested.** Every fenced block written exactly as ```` ```python ````
  on the pages listed in `python/tests/test_devdocs_snippets.py` is extracted and
  executed verbatim, in order, from the repo root (`make test-py`, or
  `uv run pytest python/tests/test_devdocs_snippets.py`). A docs edit that breaks a
  sample breaks the build.
- **Illustrative code is labelled.** Anything that does not run against the installed
  package today — a LangGraph sketch, an API that lands with a later PRP — is fenced
  with a title, e.g. ```` ```python title="Illustrative — lands with PRP-008" ````, which
  the test deliberately does not match and which the reader sees as a header.
- **Turtle is pulled from the real files.** Ontology listings use
  `--8<-- "examples/…"` snippets so the page and the file cannot drift
  (`check_paths: true` fails the build if a file moves).
- **Claims are sourced.** Numbers on the landing page and use-case pages come from
  `research/` (each tagged `[measured]` there) or from `docs/framework/`; link the
  source inline. Do not invent claims.
- **Status is truthful.** `PRPs/BOARD.md` is the authority on what is merged, in build,
  or planned; the Roadmap page links to it rather than duplicating it.

## Layout

```
mkdocs.yml                     site config (repo root)
docs/
  index.md                     landing page — the pitch
  quickstart.md                10 minutes, zero API keys
  coming-from-langgraph.md     honest side-by-side
  governing-a-coding-agent.md  the change-gate story (examples/change-gate/)
  use-cases/                   one page per domain, same template (examples/gallery/)
  guarantees.md                what is claimed, what is not
  architecture.md              the bounded projection kernel in one diagram
  roadmap.md                   merged / in build / planned
  framework/                   the framework definition (unchanged, mounted)
  stylesheets/extra.css        small additions over Material defaults
  assets/screenshots/          PR-body screenshots
```
