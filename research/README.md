# Horismos research wiki

## Purpose

<!-- --8<-- [start:purpose] -->
Horismos's design has, until now, been checked against exactly one hand-authored
scenario (the ER chest-pain triage example) and the team's own judgment. This wiki
exists to reality-check that design against what researchers and industry actually do,
worldwide, across the fields horismos draws on: computer-interpretable clinical
guideline formalisms, neuro-symbolic AI, clinical AI/CDS validation practice,
terminology engineering (SNOMED CT/LOINC/RxNorm), and agent-authored/learnable
ontologies. It is a knowledge base, not a decision log — it informs PRPs and
`[DECIDE]` resolutions, it doesn't replace them. Captain-held decisions referenced
here (e.g. the PRP-007 recommendations below) stay open questions in this wiki; they
are resolved, if at all, in their owning PRP.

The wiki was founded from a single research sweep,
`horismos-research-s1` (2026-08-24) — see every page below for its findings. That
sweep's own full report is the founding snapshot; this wiki is the living,
incrementally-updated version of it.
Weekly sweeps since then append dated "Sweep #N delta" sections to existing pages
rather than rewriting them (first: sweep #2, 2026-09-02 — see [`CHANGELOG.md`](CHANGELOG.md)).
<!-- --8<-- [end:purpose] -->

## The evidence-labeling convention

<!-- --8<-- [start:convention] -->
Every claim in this wiki is tagged **[measured]** or **[inference]**, kept visibly
separate, always:

- **[measured]** — backed by a cited paper, spec, standard, or a fact directly
  verified against this repo. The citation (URL, or `local repo` + file path) always
  accompanies the tag.
- **[inference]** — synthesis or judgment: a researcher's or an agent's reasoning
  connecting measured facts, not itself an independently citable fact.
- **[unverified]** — a qualifier, not a third label: the source was relayed only via a
  search snippet because the primary-text fetch failed. Mark it explicitly rather than
  silently upgrading it to a confirmed [measured] citation.

This isn't decoration. Horismos's own design principle — neural proposes, symbolic
decides — has a research-wiki analog: don't let a plausible-sounding synthesis pass as
a documented fact. When you add or edit a page, tag every claim you add. If you can't
tell which label a claim deserves, that's a sign it needs a source or needs rewording
as an explicit inference.
<!-- --8<-- [end:convention] -->

## Structure

```
research/
  README.md                    — this file
  CHANGELOG.md                 — append-only, one line per merged PR
  reading-log.md               — canonical cross-topic source log
  _template/page.md            — standard page skeleton
  reports/                     — the full deep-dive reports, imported verbatim with provenance headers
  index.md, verdict-history.md, sweeps.md, reports/index.md
                               — site-only pages of the MkDocs rendering (see research-site/README.md)

  intake/                      — the URL-drop convention (see intake/README.md)
  formalization/                — Q1: guideline representation languages & standards
  scenarios/                    — Q2: the scenario catalog beyond triage
  neurosymbolic-landscape/       — Q3: academic + industry neuro-symbolic AI landscape
    pivot-alignment.md           — standing page: is the adopted pivot aligned with the
                                   field? re-verdicted every sweep (verdict-history table on the page)
  validation/                    — Q4: clinical AI validation practice
  terminology/                   — Q5: SNOMED CT / LOINC / RxNorm licensing & scale
  ontology-authoring/            — Q6: agent-authored and learnable ontologies (PRP-007)
```

Each topic directory has a `00-overview.md` (or `00-catalog.md` / `00-ladder.md`) as
its entry point, plus one page per sub-topic. Follow the internal links from any
overview page to go deeper.

## Update cadence

<!-- --8<-- [start:cadence] -->
Three paths keep this wiki current:

1. **Weekly research sweep PRs.** A recurring pass that revisits open questions,
   checks for new publications relevant to a page's claims, and appends deltas.
2. **Captain-directed deep dives.** A written captain brief scopes one axis; the dive
   runs as its own research task, lands a full report under `research/reports/`, and
   distributes dated delta sections into the topic pages it touches. The reports and
   where each one's findings went are inventoried in [`reports/index.md`](reports/index.md);
   the dives themselves are listed alongside the sweeps in the
   [sweeps-and-dives table](sweeps.md#sweeps-so-far).
3. **Ad-hoc captain URL intakes.** The captain (or anyone) can drop a single URL at
   any time via `research/intake/` — see [`intake/README.md`](intake/README.md) for the exact mechanism.
   This does not wait for the weekly cadence.

## The delta-PR convention

Every update to this wiki — weekly sweep, captain-directed deep dive or ad-hoc intake —
lands as a **small, reviewable PR**: one URL, one finding, or one topic-page update per PR, never a bulk
rewrite of multiple pages at once. This mirrors the intake mechanism's own shape
(fetch → summarize → position → file, §see [`intake/README.md`](intake/README.md)) and keeps every change
independently reviewable against its cited source. If a change touches more than a
handful of files, it's probably several PRs, not one. The exceptions are a weekly sweep
and a captain-directed deep dive, each of which lands as a single PR: their edits to
existing pages are confined to dated delta sections (see above), so each page's delta
stays reviewable on its own.
<!-- --8<-- [end:cadence] -->

## Where the founding sweep's report lives

The full `horismos-research-s1` report's content has been distributed into the pages
below by topic; the original, with its framing and the researching agents' own
methodology notes, is tracked verbatim as
[`reports/2026-08-24-founding-sweep.md`](reports/2026-08-24-founding-sweep.md).
The other deep-dive reports live beside it and are indexed, with their provenance and
where each one's findings went, in [`reports/index.md`](reports/index.md). The whole corpus is also
browsable as a MkDocs site: `research-site/README.md` at the repo root explains how to
serve it locally.
