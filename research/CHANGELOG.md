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
  snippet markers, a `reports/` line in its structure block, and a corrected closing
  section (the founding report is now tracked). No claim text was rewritten or retagged.
