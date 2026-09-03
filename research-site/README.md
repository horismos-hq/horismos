# Horismos research site

A second MkDocs Material site — separate from the developer docs — that renders the
research corpus under `research/` with navigation by theme, the evidence-tag
convention explained up front, the standing verdicts made prominent, and the three
full deep-dive reports imported beside the wiki they produced. Audience: the team and
research-minded readers. Not dev-facing, not deployed: local serve only.

## Serve and build

```sh
# live preview (port 8001 so it never collides with the developer site on 8000)
uvx --with mkdocs-material mkdocs serve -f mkdocs-research.yml -a 127.0.0.1:8001

# the check that must pass before merging: warnings are errors
uvx --with mkdocs-material mkdocs build -f mkdocs-research.yml --strict
```

Output goes to `site-research/` (gitignored). No Pages or deploy workflow references
this config; publishing it is a separate captain decision.

## Layout

| Path | Role |
|---|---|
| `mkdocs-research.yml` (repo root) | The site config: `docs_dir: research`, `site_dir: site-research`, theme, extensions, and the by-theme `nav`. |
| `research/` | The canonical corpus. The site points at it; nothing is duplicated. |
| `research/index.md`, `research/verdict-history.md`, `research/sweeps.md`, `research/reports/index.md` | The site's own pages (home, verdict timeline, process, report index). They synthesize and link; they do not restate wiki claims. |
| `research/reports/` | The three full reports, imported verbatim with provenance headers. |
| `research-site/overrides/` | Material `custom_dir`: the stylesheet and the evidence-tag script under `assets/`. Non-template files here are copied into the built site. |

## Conventions the config relies on

- **Single-sourcing over forking.** `research/README.md` remains the wiki's canonical
  statement of purpose, tag convention and cadence. The home page includes those
  sections through `pymdownx.snippets` section markers (`<!-- --8<-- [start:name] -->`
  … `[end:name]`, invisible on GitHub). Because MkDocs refuses `README.md` and
  `index.md` side by side, `README.md` is listed in `exclude_docs` and any wiki link to
  `../README.md` points at `index.md` instead — the same content, served once.
- **`_template/page.md` is excluded** as a page (its `<angle-bracket>` placeholders would
  vanish as HTML) and shown verbatim as a code block on "How sweeps work".
- **Evidence tags are literal text in the markdown.** `assets/javascripts/evidence-tags.js`
  wraps `[measured]`, `[inference]`, `[unverified]` and `[DECIDE]` in styled chips at
  view time, skipping code and links. The source is never rewritten.
- **Links are real links.** Backtick path references across the wiki were converted to
  markdown links to the same targets (visible text unchanged). References to repo files
  outside `research/` link to GitHub: at `main` from wiki pages, and at the pinned
  commit named in each report's provenance header from the reports (a report's
  `file:line` numbers are only true at the state it read).
- **`--strict` proves coverage.** `validation.nav.omitted_files: warn` means every page
  under `research/` must be in the nav (or in `exclude_docs`); `links.anchors: warn`
  means every `#fragment` link must resolve. A new page is not "on the site" until the
  strict build is green.

## Adding a page

1. Write it under `research/<topic>/` from `research/_template/page.md`, tagging every
   claim per `research/README.md`.
2. Add it once to the `nav` in `mkdocs-research.yml`, under the theme a research-minded
   reader would look in (the nav is by theme, not by directory).
3. Reference other pages with relative markdown links, not bare backtick paths.
4. Run the strict build.

## Coordination

The developer site owns `mkdocs.yml`, `docs/`, `site/`, port 8000 and
`.github/workflows/docs-pages.yml`; this site owns `mkdocs-research.yml`, `research/`,
`site-research/` and port 8001. Neither config references the other's tree.
