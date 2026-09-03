# Verdict history

<!-- Site page: consolidates the standing verdicts and the decision-shaped items they
     raised, with links into the pages that own the evidence. Verdict text is quoted,
     never paraphrased; nothing here decides anything. -->

**[inference — an index, not a new verdict]** Horismos's research corpus issues
verdicts in one vocabulary — *aligned* / *aligned-with-adjustments* / *misaligned* —
against a named set of commitments, and re-issues rather than edits them. This page
puts every verdict-shaped statement the corpus has made on one timeline, quotes each
one verbatim, and indexes the captain-held items each raised. The owning page of each
verdict is the source of truth; when a sweep re-verdicts, add a row to the
[standing page's own table](neurosymbolic-landscape/pivot-alignment.md#verdict-history)
first and mirror it here.

<div class="verdict" markdown>
<div><div class="verdict-word">aligned-with-adjustments</div><div class="verdict-meta">current · pivot alignment, sweep #2 (2026-09-02) · RL axis, deep dive (2026-09-02)</div></div>
<div markdown>
Two standing verdicts, both issued 2026-09-02, both on the bounded-projection pivot
adopted 2026-08-24: the pivot itself is confirmed by the field's post-August-2026
motion and needs only shape-preserving adjustments; along the RL axis the field's
motion "*confirms the pivot's shape and the two principles; it challenges the
completeness of what Horismos has built to back the 'traces are the corpus' claim*".
</div>
</div>

## Timeline

<ul class="timeline" markdown>
<li class="neural" markdown>
<span class="when">2026-08-19 · a164db4</span><br>
**Repository state the corpus started from.** PRP-001–003 done, PRP-004 (agent loop)
landing; one hand-authored scenario (ER chest-pain triage); positioning: "one artifact
owns domain, rules *and* workflows". **[measured — local repo]**
</li>
<li markdown>
<span class="when">2026-08-24 · task horismos-research-s1</span><br>
**Founding reality-check sweep — the diagnosis.** Six research questions, six parallel
research agents. Its load-bearing judgment, flagged as such by the researching agent and
endorsed by the report: Horismos's core bet "*structurally resembles the **losing side**
of a 20-year natural experiment the field has already run, even though its internal
engineering … is more rigorous than any predecessor's*". Three mechanisms with no CIG
precedent (the perception boundary, the uniform gate, fixed-point truth maintenance);
five named risks; the validation ladder; the three recommended scenarios; the
vacuous-verification finding for PRP-007.
→ [Report](reports/2026-08-24-founding-sweep.md) · [what it changed](reports/2026-08-24-founding-sweep.md#5-what-this-changes-concretely)
</li>
<li markdown>
<span class="when">2026-08-24 · task horismos-thin-pivot-s1</span><br>
**Thin-pivot second opinion — the verdict on the thesis.** "*Verdict: partial on the
thesis, but proceed with the pivot.*" A literal "World as a live view over FHIR" is
rejected (Option B); staying vertically integrated is rejected (Option A); the
**bounded projection kernel** (Option C) is recommended: hydrate a declared, versioned
closure; reason to a fixed point unchanged; propose typed effects; commit only if the
source versions still match. Includes the guarantee-boundary table (which mechanism
survives thinness, and at what honest limit) and the PRP-by-PRP / doc-by-doc change map.
→ [Report](reports/2026-08-24-thin-pivot-second-opinion.md) · [guarantee boundary](reports/2026-08-24-thin-pivot-second-opinion.md#the-guarantee-boundary) · [change map](reports/2026-08-24-thin-pivot-second-opinion.md#prp-by-prp-change-map)
</li>
<li class="gate" markdown>
<span class="when">2026-08-24 → 2026-09-02 · PR #9, PR #10 · 1c7ce1c, 5c45c0a</span><br>
**The wiki lands; the pivot is adopted.** PR #9 distributes the founding sweep into
`research/`. The captain rules for the bounded projection kernel; PR #10 applies the
second opinion's change maps across PRP-000–PRP-008 and framework docs 00–15 (the
pivot's statements now live in PRP-000 §"Snapshot-scoped guarantees", PRP-008, and docs
03/06/07/10/13). **[measured — local repo]**
</li>
<li class="gate" markdown>
<span class="when">2026-09-02 · sweep #2 · PR #11</span><br>
**Pivot alignment — the standing verdict.** "*aligned-with-adjustments.* Every
load-bearing commitment of the bounded-projection pivot … now has independent,
post-August-2026 confirmation from at least two of the database, formal-methods,
agent-safety, and HL7 standards communities. … The adjustments are real but
shape-preserving … None requires changing the pivot's shape." Eight commitments C1–C8
assessed; ten challenge lines X1–X10; fourteen adjustments A1–A14, each with its owner.
→ [Standing page](neurosymbolic-landscape/pivot-alignment.md) · [C1–C8](neurosymbolic-landscape/pivot-alignment.md#the-commitments-assessed) · [confirms](neurosymbolic-landscape/pivot-alignment.md#what-the-new-literature-confirms) · [challenges](neurosymbolic-landscape/pivot-alignment.md#what-it-challenges-or-where-the-field-moved-differently) · [adjustments](neurosymbolic-landscape/pivot-alignment.md#adjustments-worth-considering)
</li>
<li class="gate" markdown>
<span class="when">2026-09-02 · task horismos-research-rl-s1</span><br>
**RL-axis deep dive — the scoped verdict.** "*aligned-with-adjustments — scoped to the
learnable-ontologies × KG × RL axis, extending (not re-litigating) sweep #2's verdict on
the pivot itself.*" The field's most-wanted object in 2026 RL is a deterministic symbolic
step-checker usable as a process reward — which Horismos's gate, proof trees and outcome
labels are, for any loaded ontology, and nobody in the surveyed literature uses an
RDFS/OWL/SHACL/Datalog kernel that way. But the "traces are the corpus" claim is half
true today: the representation half is built, the signal half is spec-only. Five ranked
moves; a list of what *not* to build; four open decisions.
→ [Report](reports/2026-09-02-rl-deep-dive.md) · [§4.1 verdict table](reports/2026-09-02-rl-deep-dive.md#41-alignment-verdict-on-the-rl-axis) · [§4.4 ranked moves](reports/2026-09-02-rl-deep-dive.md#44-usefulness-ranking-what-would-make-horismos-most-useful-to-real-builders-at-this-intersection) · [§5 open decisions](reports/2026-09-02-rl-deep-dive.md#5-open-decisions-captain-shaped-flagged-not-answered-here)
</li>
</ul>

## The verdicts, verbatim

| Date | Scope | Verdict | Owning page | One-line reason (quoted) |
|---|---|---|---|---|
| 2026-08-24 | The "thin layer" thesis | **partial on the thesis, but proceed with the pivot** | [Thin-pivot second opinion](reports/2026-08-24-thin-pivot-second-opinion.md#executive-verdict) | "The strategic diagnosis is right … The literal implementation 'make `World` a live view over a FHIR server' would be wrong. The defensible design is a **bounded projection kernel**." |
| 2026-09-02 | The bounded-projection pivot, commitments C1–C8 | **aligned-with-adjustments** | [Pivot alignment](neurosymbolic-landscape/pivot-alignment.md#verdict) | "Core commitments independently confirmed post-2026-08-24; divergences run neural-ward at the line Horismos draws; adjustments are shape-preserving (A1–A14)." |
| 2026-09-02 | The RL axis (learnable ontologies × KG × RL) | **aligned-with-adjustments** (scoped) | [RL deep dive §4.1](reports/2026-09-02-rl-deep-dive.md#41-alignment-verdict-on-the-rl-axis) | "The field's motion *confirms* the pivot's shape and the two principles; it *challenges* the completeness of what Horismos has built to back the 'traces are the corpus' claim and the granularity/format of the planned export; and it *bypasses* Horismos only where frontier-lab practice is internal and unpublished." |

## What "aligned-with-adjustments" commits the corpus to

**[inference]** The vocabulary is deliberate. *Aligned* would mean the field's motion
confirms the commitments and nothing is owed. *Misaligned* would mean a load-bearing
commitment has independent evidence against it and the pivot's shape is in question.
*Aligned-with-adjustments* means the shape is confirmed and specific, owner-assigned
items are owed — each rests on measured facts listed with it, none is applied by the
wiki, and each is closed only in its owning PRP. The corpus keeps score:

| Item family | Count | Where they live | Owners named |
|---|---|---|---|
| Challenge lines X1–X10 (where the field moved differently) | 10 | [pivot-alignment](neurosymbolic-landscape/pivot-alignment.md#what-it-challenges-or-where-the-field-moved-differently) | positioning (PRP-000), PRP-007, PRP-008, doc 06 |
| Adjustments A1–A14 (shape-preserving, each with its measured basis) | 14 | [pivot-alignment](neurosymbolic-landscape/pivot-alignment.md#adjustments-worth-considering) | PRP-008 D1–D5, D8–D10; PRP-005; PRP-007; PRP-000; doc 06 / doc 15 |
| RL-axis ranked moves 1–5 and the "do not build" list | 5 + 7 | [RL deep dive §4.4](reports/2026-09-02-rl-deep-dive.md#44-usefulness-ranking-what-would-make-horismos-most-useful-to-real-builders-at-this-intersection) | PRP-004, PRP-005, PRP-007, two "owner open" |
| RL-axis open decisions OD-1–OD-4 | 4 | [RL deep dive §5](reports/2026-09-02-rl-deep-dive.md#5-open-decisions-captain-shaped-flagged-not-answered-here) | captain (OD-1, OD-2), PRP-002 text (OD-3), PRP-007 (OD-4) |
| PRP-007 open items from the founding sweep (+1 from sweep #2) | 4 | [prp-007-open-items](ontology-authoring/prp-007-open-items.md) | PRP-007 |
| Required PRP-008 D-items proposed by the second opinion | 10 | [second opinion](reports/2026-08-24-thin-pivot-second-opinion.md#required-prp-008-decisions) | PRP-008 (now its D1–D10) |
| Founding-sweep follow-ups | 5 | [founding sweep §5](reports/2026-08-24-founding-sweep.md#5-what-this-changes-concretely) | PRP-000, PRP-007, doc 02, scenario choice |

## Decision-shaped questions still open, by owner

**[measured — as recorded in the owning pages on 2026-09-02; none decided here]**

- **Captain.** Which scenario — VTE / NEWS2 / sepsis-compliance — is PRP-008's first
  vertical slice ([pivot-alignment](neurosymbolic-landscape/pivot-alignment.md#open-questions-decision-shaped-flagged-not-answered-here);
  the second opinion [recommends VTE](reports/2026-08-24-thin-pivot-second-opinion.md#first-pivot-proof)).
  Owner and sequencing of an environment + grader surface (OD-1); whether to publish a
  Horismos RL environment externally (OD-2).
- **PRP-000.** Whether "kernel" survives OaK's use of the word (X10, A13); restating the
  differentiator against Palantir ("same guarantees without owning state; a proof, not a
  receipt").
- **PRP-005.** Absent-vs-not-fetched as a stated differentiator and tested manifest
  assumptions (A14); the reward-oracle acceptance items (gate error rates ρ₀/ρ₁,
  isomorphic-relabeling invariance, a formal non-gameability statement, a permissiveness
  number).
- **PRP-007.** Restating the gate rationale from accuracy to safety (X2, A10); the
  staged-admission question for low-risk additive deltas (OD-4); the four tracked open
  items.
- **PRP-008.** D1–D5, D8, D9, D10, each narrowed by evidence in X7 and A1–A7.
- **Doc 06 / PRP-004 / PRP-007.** Whether narration gains a deterministic conclusion
  monitor (A8).
- **Doc 09 / PRP-002.** Restating design rule 2 ("the literature is unanimous" on loss
  masking is not true) and adding a neutral export rendering (OD-3).
