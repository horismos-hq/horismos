# How sweeps work

<!-- Site page for the Intake & process section. The cadence and delta-PR text is
     single-sourced from README.md via snippet markers; the page template is shown
     verbatim from _template/page.md. -->

The corpus grows by three mechanisms and one shape. All three mechanisms are below
(quoted from the wiki's own README, which remains the canonical statement); the
shape — a page with five fixed sections and a tag on every claim — is shown at the end.

--8<-- "README.md:cadence"

## What a sweep produces

A sweep is a research task with a written brief, run in an isolated worktree, that
reads the repository first (so every comparison is against what Horismos actually
specifies) and then surveys the field with parallel research agents. Its outputs land
in three places:

| Output | Where | Convention |
|---|---|---|
| A full report | [`reports/`](reports/index.md) | Imported verbatim, with a provenance header naming the task, date, repo commit it was grounded against, and where its findings went. The report is the snapshot; the wiki is the living version. |
| Deltas on topic pages | the `research/<topic>/` pages | A dated section appended to each page it touches, named for its origin — "Sweep #N delta (YYYY-MM-DD)" for a weekly sweep, "<dive> deep dive delta (YYYY-MM-DD)" for a captain-directed dive; founding text is never rewritten (supersessions are recorded in the delta, e.g. [regulatory](validation/regulatory.md#sweep-2-delta-2026-09-02-the-2022-cds-guidance-is-superseded-dsi-transparency-mandates-are-being-withdrawn)). |
| Rows in the reading log, a line in the changelog | [`reading-log.md`](reading-log.md), [`CHANGELOG.md`](CHANGELOG.md) | Append-only. The reading log is grouped by research question; `[unverified]` rows are marked, never silently upgraded. |

A standing page — currently only [pivot alignment](neurosymbolic-landscape/pivot-alignment.md)
— is re-verdicted rather than edited: the verdict line is re-issued and a row is added
to the page's verdict-history table, mirrored on the [verdict history](verdict-history.md).

## Sweeps so far

| # | Date | Task | What it did | Report |
|---|---|---|---|---|
| 1 (founding) | 2026-08-24 | `horismos-research-s1` | Six research questions (formalization, scenarios, neuro-symbolic landscape, validation, terminology, ontology authoring), one captain URL intake run live as the worked example; proposed this wiki's structure. Landed as PR #9. | [Founding sweep](reports/2026-08-24-founding-sweep.md) |
| — (second opinion) | 2026-08-24 | `horismos-thin-pivot-s1` | Independent assessment of the "thin layer" thesis the founding sweep raised; produced the bounded projection kernel and the change maps PR #10 applied. Not a sweep — a decision-support report. | [Second opinion](reports/2026-08-24-thin-pivot-second-opinion.md) |
| 2 | 2026-09-02 | `horismos-research-sweep-w2` | Literature since 2026-08-24 across Q1/Q3/Q4/Q5/Q6 and a new Q7 (coding-agent harnesses); issued the standing pivot-alignment verdict; two new topic pages; deltas on nine pages; fixed the terminology page's missing tags. Landed as PR #11. | Distributed into pages (no separate report file); see the [changelog](CHANGELOG.md) |
| — (deep dive) | 2026-09-02 | `horismos-research-rl-s1` | Captain-directed deep research on the RL axis, taking sweep #2's verdict as given; six parallel sub-sweeps; verdict scoped to its axis; ranked moves and open decisions. | [RL deep dive](reports/2026-09-02-rl-deep-dive.md) |
| — (consolidation) | 2026-09-03 | `horismos-researchdocs-a1` | This site: imported the three reports, added the theme navigation, verdict history and these process pages; existing pages changed only by link fixes. | — |
| — (deep dive) | 2026-09-21 | `horismos-research-jev-s1` | Captain-directed deep research on TypeSafe's **Jev** and the non-generative typed-decision-model axis, taking sweep #2's and the RL dive's verdicts as given; five parallel sub-agents; a fourteen-row corrections table against the intake facts; one new topic page, dated deltas on five existing pages, ~148 reading-log rows under Q9; nine ranked moves and eight open decisions; standing verdict re-issued. One FDA date on the regulatory page corrected and a second confirmed first-hand, and one claim drafted from a paywalled abstract corrected in place. | [Jev deep dive](reports/2026-09-21-typesafe-jev-deep-dive.md) |

## How an intake works

The ad-hoc path is the [intake convention](intake/README.md): drop a URL in the
[queue](intake/queue.md); the next agent session fetches and reads it in full, names the
author and affiliation, does one shallow follow-up pass, **positions it against
Horismos's actual design**, and files it — a raw note under `intake/filed/`, a paragraph
on the relevant topic page, a reading-log row, and the queue row marked `filed`. The
[Markovic intake](intake/filed/2026-08-24-markovic-learnable-ontologies.md) is the worked
example, and it is what later grew into the RL-axis deep dive's central question.

## The page template

Every topic page follows this skeleton (`research/_template/page.md`, shown verbatim).
The five sections are what let a reader audit a page in under a minute: what it claims,
on what tagged evidence, from which sources, at what confidence, checked when.

```markdown
--8<-- "_template/page.md"
```

## Honesty conventions worth knowing

- **Failures are recorded, not hidden.** The founding sweep records that one research
  agent returned a non-answer and had to be resumed; the RL deep dive records that its
  web-search budget ran out before two sub-sweeps finished and tags everything from
  them accordingly; sweep #2 removed a CQL-standardization claim that could not be
  verified rather than soften it.
- **Negatives are first-class.** "Searched but not found" sections list what the sweep
  looked for and did not find — often the strongest evidence that a square is open.
- **Recommendations are not decisions.** Captain-held decisions referenced anywhere in
  the corpus stay open here; they are resolved, if at all, in their owning PRP through
  the brainstorming protocol.
