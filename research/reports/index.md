# Deep-dive reports

<!-- Index of the full reports imported into the corpus. Each report file carries its
     own provenance header immediately under its title. -->

The wiki's topic pages are the living, incrementally-updated version of the research;
these are the full reports that produced it, imported verbatim so the original framing,
methodology notes, tables and negatives are never lost. Each carries a provenance
header (task, date, repository commit it was grounded against, what has happened
since) and links its repo-file references to that exact commit — a report's
`file:line` citations are only true at the state it read.

| Report | Task · date | Grounded against | What it is | Where its findings went |
|---|---|---|---|---|
| [Founding reality-check sweep](2026-08-24-founding-sweep.md) | `horismos-research-s1` · 2026-08-24 | [`a164db4`](https://github.com/billaram/horismos/blob/a164db4) | Six research questions against what Horismos actually specifies; the executive reality-check (§1), scenario catalog (§2), proposed wiki structure and intake convention (§3), 120-row reading log (§4). 17 measured / 8 inference / 3 unverified top-level tags; 40 citations. | Distributed into every topic directory by PR #9; §1.1's diagnosis triggered the second opinion below. |
| [Thin-pivot second opinion](2026-08-24-thin-pivot-second-opinion.md) | `horismos-thin-pivot-s1` · 2026-08-24 | [`1c7ce1c`](https://github.com/billaram/horismos/blob/1c7ce1c) | An independent assessment of the "thin layer" thesis: three options adversarially compared; the guarantee-boundary table; the PRP-by-PRP and doc-by-doc change maps; the ten PRP-008 D-items; the capability-fragment mechanism for coding agents; risks of pivoting and of not pivoting; the recommended sequence. | The captain adopted Option C; PR #10 applied the change maps across PRP-000–008 and docs 00–15. Its verdict is quoted on the [verdict history](../verdict-history.md). |
| [RL-axis deep dive](2026-09-02-rl-deep-dive.md) | `horismos-research-rl-s1` · 2026-09-02 | [`5c45c0a`](https://github.com/billaram/horismos/blob/5c45c0a) | Learnable ontologies × knowledge graphs × reinforcement learning: the claimed-vs-built audit of the trace surface (§2), a six-part landscape (§3), the alignment table, flywheel assessment, gated ontology-learning sandbox and ranked moves (§4), four open decisions (§5), a 100-row source log (§6). 51 measured / 31 inference / 10 unverified tags; 168 citations. | Not yet distributed into topic pages or the reading log — a follow-up sweep should append its sources under a Q8 section and file its §4.2/§4.3 findings on the ontology-authoring pages. |
| [TypeSafe Jev deep dive](2026-09-21-typesafe-jev-deep-dive.md) | `horismos-research-jev-s1` · 2026-09-21 | [`adfc27a`](https://github.com/billaram/horismos/blob/adfc27a) | The non-generative typed-decision-model axis: what Jev actually is from primary sources, with a fourteen-row corrections table (§3); the calibration, routing, constrained-decoding and parallel-decoding literatures (§4); six days of adoption and independent evidence (§5); the position against Horismos — threat/complement/validation, Jev-as-proposer, the auditability wedge, calibration vs proof, single-pass evaluation, and the re-issued standing verdict (§6); nine ranked moves and a do-not-build list (§7); negatives (§8); eight open decisions (§9). | Distributed the same day into [`typed-decision-models.md`](../neurosymbolic-landscape/typed-decision-models.md) (new page) plus dated deltas on the Q3 overview, lineage assessment, coding-agent harnesses, validation ladder and regulatory pages, and ~145 reading-log rows under Q9. |

## Reading them together

**[inference]** The three reports are one argument in three movements. The founding
sweep establishes the historical fact the whole design must answer — the formalisms
that owned domain, rules and workflow together stayed academic, and the thin,
composable layers over FHIR are the ones that shipped — while naming what Horismos has
that none of them had (the uniform gate, fixed-point truth maintenance, a
confidence-gated perception boundary). The second opinion turns that fact into a
design: not "become a FHIR wrapper", not "delete World", but a bounded projection that
keeps every novel guarantee inside a versioned snapshot and makes the honest claim
narrower. The RL deep dive then asks what the kernel is *for* beyond clinical decision
support, and finds the field converging on exactly the object Horismos already
produces as a side effect — a deterministic, per-step, proof-carrying verdict — while
showing that the code has not yet caught up with the claim. The
[standing verdict](../neurosymbolic-landscape/pivot-alignment.md) sits between the
second and third movements and is re-issued every sweep.

**[inference]** The fourth report is a coda to the first three, and the first one written
against a *commercial* rather than an academic neighbour. Where the founding sweep asked
whether Horismos resembles the losing side of a past natural experiment, the Jev deep
dive asks what happens when a funded company concedes the premise — decisions should be
typed, not generated — and ships only that half. Its answer is that the premise stopped
being a differentiator on 2026-09-15, that two sharper ones are available and
evidence-backed (cross-answer consistency, and epistemic rather than merely structural
auditability), and that the weakest link in Horismos's own chain is the unmeasured,
self-reported number its perception threshold compares against.

## Importing a new report

When a future sweep or deep dive produces a full report:

1. Copy it verbatim to `research/reports/YYYY-MM-DD-<slug>.md`; keep every tag and
   citation; replace any workspace-local absolute path with a repo-relative reference.
2. Insert a `> **Provenance.**` blockquote directly under the title: task id, date,
   scope, the commit it was grounded against (link it), what has happened since, and
   what was changed on import (ideally nothing but links).
3. Turn repo-file references into links to GitHub **at that commit**, and
   `research/…` references into site-relative links.
4. Add it to this table and to `mkdocs-research.yml`'s nav under the theme it belongs
   to; add a changelog line; verify with `mkdocs build --strict`.
