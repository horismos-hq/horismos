---
name: learn-codebase
description: Use when someone new to the horismos codebase wants a guided tour, onboarding walkthrough, code walkthrough, or visual learning material — "walk me through the code", "onboard a new engineer", "explain the codebase", "learning curriculum". Produces a published HTML artifact.
---

# Learn-codebase: the Horismos curriculum artifact

Produces ONE self-contained HTML artifact: a ByteByteGo-style, diagram-led,
unit-by-unit curriculum teaching this codebase to a newcomer. Diagrams carry the
teaching; prose supports them.

## Workflow (in this order)

1. **Read sources of truth before writing anything**: `docs/framework/00-index.md`,
   `CLAUDE.md`, `PRPs/README.md`, plus the *actual current source* of every file you
   will quote. Code snippets are verbatim from files read this session — never from
   memory, never invented. Cite the repo path on every code card.
2. **Load the `artifact-design` skill** before writing the page.
3. **Write the HTML to the scratchpad in sections** — Write the shell + Unit 0, then
   append units with Edit. Never emit the whole page in one Write (it will be
   thousands of lines; single-shot writes truncate or degrade).
4. **Publish with the Artifact tool**: favicon `🎓`, title `Learning Horismos` — both
   stable across every redeploy. Updates re-publish the same file path so the URL
   survives. `<title>` in the HTML itself.
5. Default scope: full curriculum (units follow the docs 00–14 reading order, roughly
   11 units, ending with "your first contribution"). If the user names a focus area,
   produce a single-unit deep dive with the same unit skeleton.

## The unit skeleton (every unit, same order, no variations)

1. Eyebrow (`Unit N · Theme`) + title + 2–3 sentence lede
2. "After this unit you can" — 2–4 objectives
3. One or two diagrams with the color legend strip
4. 2–3 code walkthrough cards: title bar = repo path + crate-colored dot; circled
   number markers on load-bearing lines keying into a numbered "trace notes" list
   below the card
5. "Enforced, not promised" callout where a hard rule applies (show the `make`
   command that enforces it)
6. "Gate check" self-quiz: 2–3 `<details>` questions with answers inside
7. "Field guide" table: exact repo paths to read next, with one-line reasons
8. Prev/next unit nav

## The visual system (fixed, established in Unit 0, never varied)

- **Four-color boundary semantics** as CSS custom properties, used by every diagram,
  chip, and dot: `--symbolic` indigo `#3B5BDB` (deterministic machinery), `--neural`
  amber `#C87B1B` (probabilistic), `--gate` teal `#17877B` (gates/verification/
  language boundary), `--forbidden` red `#B4382E` (forbidden edges, rejections).
- Theme-aware: tokens redefined under `@media (prefers-color-scheme: dark)` AND
  `:root[data-theme="dark"]` / `:root[data-theme="light"]` overrides.
- Two diagram kinds, divided by job: **mermaid** (`<pre class="mermaid">`, native —
  no libraries) for anything with edges, using mid-tone `classDef` fills legible on
  both themes; **hand-built HTML/CSS grid "anatomy" cards** for boxed labeled
  layouts mermaid can't do.
- Code highlighting: hand-authored `<span class="tk-kw|tk-com|tk-str|tk-meta|tk-fn">`
  tokens. No highlight.js, no CDN, no external requests of any kind (artifact CSP
  blocks them).
- Sticky left rail with unit list + `localStorage` progress checkboxes; prose at
  ~68ch; wide diagrams/code in breakout containers with `overflow-x: auto`.

## Honesty rules

- Modules that are stubs are taught as stubs: state the owning PRP and spec doc
  ("specified in doc 04, lands with PRP-002"), never describe unimplemented
  behavior as existing.
- The curriculum teaches the architecture *through* the current code — when the
  codebase grows, regenerate from current source, don't patch stale snippets.
