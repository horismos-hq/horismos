# Research wiki changelog

Append-only. One line per merged PR: date, PR link, summary.

- 2026-08-24 — (this PR) — Founded `research/` from the `horismos-research-s1` sweep
  report: reality-check findings (§1), scenario catalog (§2), and reading log (§4)
  distributed into the structure the report proposed (§3), including the Markovic
  learnable-ontologies intake as the worked example.
- 2026-09-02 — (this PR) — Sweep #2 delta (`horismos-research-sweep-w2`): added the
  standing [`neurosymbolic-landscape/pivot-alignment.md`](neurosymbolic-landscape/pivot-alignment.md) (verdict:
  **aligned-with-adjustments** for the 2026-08-24 bounded-projection pivot; 14
  adjustments A1–A14 and 10 challenge lines X1–X10, all captain-held), plus two new
  topic pages ([`neurosymbolic-landscape/coding-agent-harnesses.md`](neurosymbolic-landscape/coding-agent-harnesses.md),
  [`ontology-authoring/kg-maintenance-and-agent-memory.md`](ontology-authoring/kg-maintenance-and-agent-memory.md)) and dated "Sweep #2 delta"
  sections on nine existing pages. Fixed the evidence-tag gap in
  [`terminology/00-overview.md`](terminology/00-overview.md) deferred from PR #9 (the page carried no
  [measured]/[inference] tags; now tagged, repo facts re-verified at HEAD). Reading log:
  +~140 rows across Q1/Q3/Q4/Q5/Q6 and a new Q7 (coding-agent harnesses).
  **Supersessions noted:** [`validation/regulatory.md`](validation/regulatory.md) — the FDA 2022 CDS guidance cited
  in its Evidence is superseded by the January 2026 revision (delta records the change;
  founding text left intact); [`formalization/cql-cds-hooks-fhir.md`](formalization/cql-cds-hooks-fhir.md) — "normative ballot
  planned for 2025" for CDS Hooks is now a 3.0.0 R3 Normative Ballot in reconciliation,
  2.0.1 still current; PRP-000's positioning-table claim about harnesses re-checked and
  still holds, with four nuances recorded. No sweep #1 page text was rewritten.
- 2026-09-03 — (this PR) — Research site (`horismos-researchdocs-a1`): consolidated the
  corpus into a second MkDocs site (`mkdocs-research.yml` at the repo root, docs_dir =
  `research/`, local serve only — no deploy workflow). Imported the three full deep-dive
  reports verbatim under `reports/` with provenance headers (founding sweep 2026-08-24,
  thin-pivot second opinion 2026-08-24, RL-axis deep dive 2026-09-02); added the site's
  own pages (`index.md` home, `verdict-history.md`, `sweeps.md`, `reports/index.md`).
  Existing wiki pages changed only by link fixes: backtick path references became real
  links to the same targets (repo files link to GitHub), `README.md` gained invisible
  snippet markers, `reports/` and site-page lines in its structure block, and a corrected closing
  section (the founding report is now tracked). No claim text was rewritten or retagged.
- 2026-09-21 — (this PR) — Jev deep dive (`horismos-research-jev-s1`): captain-directed
  deep research on TypeSafe's **Jev** and the non-generative typed-decision-model axis,
  landed as the fourth full report
  ([`reports/2026-09-21-typesafe-jev-deep-dive.md`](reports/2026-09-21-typesafe-jev-deep-dive.md))
  plus one new topic page
  ([`neurosymbolic-landscape/typed-decision-models.md`](neurosymbolic-landscape/typed-decision-models.md))
  and dated "Jev deep dive delta (2026-09-21)" sections on five existing pages
  ([`neurosymbolic-landscape/00-overview.md`](neurosymbolic-landscape/00-overview.md),
  [`neurosymbolic-landscape/lineage-assessment.md`](neurosymbolic-landscape/lineage-assessment.md),
  [`neurosymbolic-landscape/coding-agent-harnesses.md`](neurosymbolic-landscape/coding-agent-harnesses.md),
  [`validation/00-ladder.md`](validation/00-ladder.md),
  [`validation/regulatory.md`](validation/regulatory.md)). Reading log: ~145 rows under a
  new Q9 section (Q8 stays reserved for the RL dive's undistributed sources). The
  standing pivot-alignment verdict was re-issued, scoped to this axis, and mirrored on
  [`verdict-history.md`](verdict-history.md). **Corrections recorded:** fourteen intake
  facts were checked against primary sources and four were wrong or misleading — the
  benchmark's reference answers are two frontier models rather than humans, the "0%
  structured-output error" figure is stated by its own author to be "not empirical", the
  interpretability critique attributed to TypeSafe is a third party's, and there are
  three co-founders rather than one. Two dates in
  [`validation/regulatory.md`](validation/regulatory.md) were corrected against the FDA
  documents themselves (the PCCP guidance is AUGUST 2025, not December 2024; the revised
  CDS guidance's cover date is JANUARY 2026, confirming what sweep #2 could only mark
  `[unverified]`), and one claim drafted in this report from a paywalled abstract was
  wrong and is corrected in place with the error recorded in its §10 (Ghassemi et al.
  attacks *inherently* explainable models too, not only post-hoc ones). Eight open
  decisions (OD-1–OD-8) recorded, all captain-held; no PRP text edited and no `crates/`
  change.
