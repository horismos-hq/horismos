# The Horismos research corpus

<!-- Site home page. Canonical prose about the wiki's purpose, tag convention and
     update cadence is single-sourced from README.md through the snippet markers
     there (see research-site/README.md); do not restate it here. -->

This is the evidence base behind Horismos's design decisions: the founding
reality-check sweep, the independent second opinion that produced the
bounded-projection pivot, the standing verdict on whether that pivot stays aligned
with the field, the reinforcement-learning deep dive, and the living wiki that all of
them feed. It is written for the team and for research-minded readers, depth over
marketing. It is **not** the developer documentation.

<div class="verdict" markdown>
<div><div class="verdict-word">aligned-with-adjustments</div><div class="verdict-meta">standing verdict · sweep #2 · 2026-09-02 · re-issued every sweep · last re-issued 2026-09-21 (typed-decision axis)</div></div>
<div markdown>
The bounded-projection pivot adopted on 2026-08-24 — snapshot-scoped guarantees over a
versioned hydration, a conditional commit that fails closed on source-version drift, a
thin overlay on CDS Hooks / FHIR / CQL, a gate that emits proofs — has independent
post-August-2026 confirmation from the database, formal-methods, agent-safety and HL7
standards communities. Where the field moved differently it moved neural-ward at the
exact line Horismos draws, and the same papers supply the numbers for why Horismos
should not follow. Fourteen shape-preserving adjustments (A1–A14) and ten challenge
lines (X1–X10) are recorded, all captain-held. The RL-axis deep dive of the same day
extends the verdict along its axis, and the Jev deep dive (2026-09-21) re-issues it
along a third — the non-generative typed-decision-model axis — unchanged in shape.
**[Read the standing page →](neurosymbolic-landscape/pivot-alignment.md)** ·
[Verdict history](verdict-history.md) · [RL-axis verdict](reports/2026-09-02-rl-deep-dive.md#41-alignment-verdict-on-the-rl-axis) · [Typed-decision verdict](reports/2026-09-21-typesafe-jev-deep-dive.md#66-q-d6-the-standing-verdict-re-issued)
</div>
</div>

## How to read this corpus

Every page carries the same discipline. Learn the three tags once and every table,
verdict and adjustment in the corpus reads the same way — the tags below are rendered
as chips throughout the site (hover one for its definition); in the markdown source
they are literal text.

<div class="tag-legend">
<span class="ev ev-measured">[measured]</span>
<span>A cited paper, spec, standard, or a fact directly verified against this repository. The citation always accompanies the tag; repo facts cite <code>local repo</code> plus a path.</span>
<span class="ev ev-inference">[inference]</span>
<span>Synthesis or judgment — a researcher's or an agent's reasoning connecting measured facts. Every verdict, every "bearing" and every adjustment is an inference, by construction.</span>
<span class="ev ev-unverified">[unverified]</span>
<span>A qualifier, not a third label: the source was only seen through a search snippet because the primary fetch failed. Marked rather than silently upgraded.</span>
<span class="ev ev-decide">[DECIDE]</span>
<span>An open design question owned by exactly one PRP. The corpus informs it; it is resolved only in the owning PRP, never here.</span>
</div>

--8<-- "README.md:convention"

## Why the corpus exists

--8<-- "README.md:purpose"

## The corpus by theme

The navigation is by theme, not by directory: each wiki page is listed exactly once,
where a research-minded reader would look for it. The `research/` markdown stays
canonical — this site only points at it.

<div class="corpus-grid" markdown>

<div class="corpus-card" markdown>
<p class="q">Flagship · verdict history</p>
<h3><a href="verdict-history/">The Pivot</a></h3>
<p>From the founding sweep's diagnosis ("the losing side of a 20-year natural experiment") through the <a href="reports/2026-08-24-thin-pivot-second-opinion/">second opinion</a> that shaped the bounded projection kernel, to the <a href="neurosymbolic-landscape/pivot-alignment/">standing alignment verdict</a> and the <a href="neurosymbolic-landscape/lineage-assessment/">lineage question</a>.</p>
</div>

<div class="corpus-card" markdown>
<p class="q">Q3 · Q7</p>
<h3><a href="neurosymbolic-landscape/00-overview/">Landscape</a></h3>
<p>Who owns the term "neurosymbolic" (Scallop, LNN, DeepProbLog) and why Horismos's hard gate is a different animal; what clinical AI vendors actually ship; how <a href="neurosymbolic-landscape/coding-agent-harnesses/">coding-agent harnesses</a> structure capabilities, permissions and policy.</p>
</div>

<div class="corpus-card" markdown>
<p class="q">Deep dive · 2026-09-02</p>
<h3><a href="reports/2026-09-02-rl-deep-dive/">RL axis</a></h3>
<p>Learnable ontologies × knowledge graphs × reinforcement learning: whether Horismos's proofs, gate verdicts and deterministic traces can serve as verifiable rewards, what "traces are the corpus" would take to make true, and a gated ontology-learning sandbox. Plus the <a href="ontology-authoring/kg-maintenance-and-agent-memory/">KG-maintenance and agent-memory</a> convergence.</p>
</div>

<div class="corpus-card" markdown>
<p class="q">Deep dive · 2026-09-21</p>
<h3><a href="neurosymbolic-landscape/typed-decision-models/">Typed decisions</a></h3>
<p>The non-generative "System One model" category that appeared on 2026-09-15: what <a href="reports/2026-09-21-typesafe-jev-deep-dive/">TypeSafe's Jev</a> actually is from primary sources, which literature already owns each piece, how it is really doing six days in, and why the durable differentiators are cross-answer consistency and epistemic — not merely structural — auditability.</p>
</div>

<div class="corpus-card" markdown>
<p class="q">Q2 · Q4 · Q5</p>
<h3><a href="scenarios/00-catalog/">Clinical ground</a></h3>
<p>The <a href="scenarios/00-catalog/">scenario catalog</a> beyond triage (VTE, NEWS2, sepsis-bundle compliance, and what was ruled out), the <a href="validation/00-ladder/">five-rung validation ladder</a> with its cautionary case studies, and <a href="terminology/00-overview/">terminology</a> licensing and subsumption at scale.</p>
</div>

<div class="corpus-card" markdown>
<p class="q">Q1</p>
<h3><a href="formalization/00-overview/">Formalization</a></h3>
<p>Thirty years of computer-interpretable guideline formalisms — Arden, GLIF, PROforma, the Task-Network-Model family — and why CQL, CDS Hooks and FHIR PlanDefinition were the ones that reached national scale.</p>
</div>

<div class="corpus-card" markdown>
<p class="q">Q6 · PRP-007</p>
<h3><a href="ontology-authoring/00-overview/">Ontology authoring</a></h3>
<p>Agent-authored and learnable ontologies: benchmarks, the documented failure modes of autonomous knowledge extension, vacuous verification and the formalization gap, and the <a href="ontology-authoring/prp-007-open-items/">open items</a> tracked for PRP-007.</p>
</div>

<div class="corpus-card" markdown>
<p class="q">Process</p>
<h3><a href="sweeps/">Intake &amp; process</a></h3>
<p>How weekly sweeps and ad-hoc URL intakes work, the delta-PR convention, the page template, the <a href="reports/">imported deep-dive reports</a>, the canonical <a href="reading-log/">reading log</a> and the <a href="CHANGELOG/">changelog</a>.</p>
</div>

</div>

## Three ways in

- **New to the project.** Read the founding sweep's
  [executive reality-check](reports/2026-08-24-founding-sweep.md#1-executive-reality-check)
  (§1.1–1.7), then the [pivot's guarantee-boundary table](reports/2026-08-24-thin-pivot-second-opinion.md#the-guarantee-boundary),
  then the [standing verdict](neurosymbolic-landscape/pivot-alignment.md). Forty minutes; you will
  know why the design is shaped the way it is and what it does not claim.
- **Owning a PRP `[DECIDE]`.** Start from the [verdict history](verdict-history.md), which
  indexes every captain-held adjustment (A1–A14), challenge (X1–X10) and open decision
  (OD-1–OD-4 on the RL axis, OD-1–OD-8 on the typed-decision axis) by owner; follow the links into the evidence; resolve the question in your
  PRP through the brainstorming protocol, not here.
- **Skeptical.** Go straight to what the field does *differently*:
  [X1–X10](neurosymbolic-landscape/pivot-alignment.md#what-it-challenges-or-where-the-field-moved-differently),
  the [risks of pivoting](reports/2026-08-24-thin-pivot-second-opinion.md#risks-of-pivoting),
  the ["searched but not found"](reports/2026-09-02-rl-deep-dive.md#7-searched-but-not-found-consolidated)
  negatives, and the [ruled-out scenarios](scenarios/ruled-out.md). The corpus records
  its own misses (a research agent that returned a non-answer and was resumed; a
  search budget exhausted mid-sweep; a removed claim that could not be verified) rather
  than papering over them.

## The corpus in numbers

| | |
|---|---|
| Wiki topic pages (six research questions + Q7) | 31, each with Claim / Evidence / Source / Confidence / Last-verified |
| Full deep-dive reports, imported verbatim | 4 — [index](reports/index.md) |
| Sources in the [reading log](reading-log.md) | 438 rows across eight sections (sweeps #1 and #2, plus Q9 from the Jev deep dive); the RL deep dive's 100-odd further sources are still only in its own [§6 source log](reports/2026-09-02-rl-deep-dive.md#6-source-log) |
| Standing verdicts | 3 — pivot alignment (sweep #2), the RL axis and the typed-decision axis (deep dives) — see the [verdict history](verdict-history.md) |
| Captain-held items indexed | 14 adjustments, 10 challenge lines, 12 open decisions, 14 ranked moves, 5 PRP-007 open items |

## How it stays current

Two paths — weekly sweep PRs and ad-hoc URL intakes — each landing as a small,
reviewable delta. The full convention is on [How sweeps work](sweeps.md); the
[changelog](CHANGELOG.md) is the append-only record of what landed when.
