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

## Published site

The site is published at <https://docs.horismos.ai/> by
`.github/workflows/docs-pages.yml`: on every push to `main` that touches `mkdocs.yml`,
`docs/` or `examples/` (and on manual dispatch) it runs the same strict build and
deploys `site/` to GitHub Pages. It is a deploy-on-merge workflow, not a pull-request
check — PRP-006 owns PR CI. The repository's Pages setting must be "GitHub Actions"
for the deploy job to succeed. `site_url` in `mkdocs.yml` is that address.

## Conventions

- **Runnable code is tested.** Every fenced block written exactly as ```` ```python ````
  on the pages listed in `python/tests/test_devdocs_snippets.py` is extracted and
  executed verbatim, in order, from the repo root (`make test-py`, or
  `uv run pytest python/tests/test_devdocs_snippets.py`). A docs edit that breaks a
  sample breaks the build.
- **Shown output is pinned.** A ```` ```text ```` fence that immediately follows a
  runnable ```` ```python ```` fence (same indent, only blank lines between) is the
  output the page shows for that sample, and the same test asserts it appears verbatim
  in the page's real stdout. Regenerate the block from the sample's stdout rather
  than hand-editing it; a diagram or shell transcript elsewhere on the page is not
  matched. Styling markup (a `<div markdown>` wrapper, for instance) has to go
  around the whole pair — anything between the two fences unpins the output.
- **Illustrative code is labelled.** Anything that does not run against the installed
  package today — a LangGraph sketch, an API that lands with a later PRP — is fenced
  with a title, e.g. ```` ```python title="Illustrative — lands with PRP-008" ````, which
  the test deliberately does not match and which the reader sees as a header.
- **Turtle is pulled from the real files.** Ontology listings use
  `--8<-- "examples/…"` snippets so the page and the file cannot drift
  (`check_paths: true` fails the build if a file moves).
- **Claims are sourced, but the citation must be public.** Numbers on the landing page
  and use-case pages are researched against the internal wiki (each claim tagged
  `[measured]` there) or against `docs/framework/`. Do not invent claims — and do not
  publish the internal path either. **The repository is private and this site is
  public**, so a reader cannot open `research/…`, `PRPs/…`, `decisions/…`, or any
  internal sweep-report name; a path like that renders as a dead pointer that also
  advertises what it cannot show. Instead: link a page on this site that covers the
  same ground ([Verification](framework/10-verification.md), [Guarantees](guarantees.md),
  [Roadmap](roadmap.md), …), cite the external primary source the wiki itself cites
  (paper, standard, post-mortem) where there is one, or state what was measured in the
  sentence. Never weaken a claim just to avoid citing a private source — say the
  finding in prose instead. Repository paths that describe the shape of the code a
  reader will get (`crates/…`, `examples/…`, `python/…`, `make` targets) are fine to
  keep as orientation.
  **A pointer is a claim.** Never replace a private path with a link you have not
  opened and confirmed covers the claim; a link that lands on a page not discussing the
  topic reads as confusion, where a private path at least reads as missing access. If no
  public page carries it, state the fact in prose and cite nothing.
  **Check it by enumeration, not by a prefix grep.** Grepping for known prefixes only
  finds the names someone already thought of; it misses bare filenames
  (`03-decisions.md`) and working-artefact names that carry no prefix at all. List
  every inline code span and every link target under `docs/`, then classify each one —
  an unresolvable file, directory, document or document *section* is in scope whatever
  its prefix. And follow every pointer to its destination: replacing a private path
  with a link to a page that still carries one only moves the dead end.
- **The visual layer is one file.** `docs/stylesheets/extra.css` holds every colour
  as a `--horis-*` token with a light and a slate value; `mkdocs.yml` sets
  `primary: custom` / `accent: custom` so the theme ships no colour of its own. The
  palette carries meaning and matches the research site: blue is symbolic, amber is
  neural, teal is the gate/proof, red is a refusal. Light is the designed default,
  and the slate scheme is kept working — a new token needs both values. There is no
  `theme.custom_dir`; `theme.font: false` and "no external assets" (no webfonts, no
  CDN, no images) are deliberate.
- **Status is truthful.** The internal board is the authority on what is merged, in
  build, or planned; the Roadmap page restates its current state in the page's own
  words and carries the date it was last reconciled, rather than pointing readers at a
  file they cannot open.

## Layout

```
mkdocs.yml                     site config (repo root)
docs/
  index.md                     landing page — the pitch
  quickstart.md                10 minutes, zero API keys
  go-live.md                   the same demo on a real model — one env var, a real captured trace
  coming-from-langgraph.md     honest side-by-side
  governing-a-coding-agent.md  the change-gate story (examples/change-gate/)
  use-cases/                   one page per domain, same template (examples/gallery/)
  guarantees.md                what is claimed, what is not
  architecture.md              the bounded projection kernel in one diagram
  roadmap.md                   merged / in build / planned
  framework/                   the framework definition (unchanged, mounted)
  stylesheets/extra.css        the whole visual layer — see the convention above
  assets/screenshots/          PR-body screenshots
.github/workflows/docs-pages.yml   deploys the strict build to GitHub Pages on merge to main
examples/triage/runs/          real trajectories quoted by the site (horismos.trace/2)
```
