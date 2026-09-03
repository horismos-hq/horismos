# Horismos Reality-Check Research Sweep

> **Provenance.** Produced by scout task `horismos-research-s1` on **2026-08-24** (scope: knowledge only — no implementation, no PR). This is the report that founded the wiki: PR #9 distributed its §1 findings, §2 catalog and §4 reading log into the topic pages; §3 became the wiki's own structure and the intake convention. Repository facts in it were verified against the tree as of [`a164db4`](https://github.com/billaram/horismos/blob/a164db4) (2026-08-19, the last commit before the wiki landed), and repo-file references below link to that commit. Imported verbatim on 2026-09-03 by task `horismos-researchdocs-a1`: every [measured] / [inference] / [unverified] tag and citation is intact; the only edits are this header and path references turned into links. **Where it stands now:** the §1.1 diagnosis triggered the [thin-pivot second opinion](2026-08-24-thin-pivot-second-opinion.md) the same day and, through it, the bounded-projection pivot adopted in PR #10; the [standing verdict](../neurosymbolic-landscape/pivot-alignment.md) tracks whether that pivot stays aligned with the field.

**Scout task**: `horismos-research-s1` · **Date**: 2026-08-24 · **Scope**: knowledge only — no implementation, no PR.

Every design discussion in this repo has orbited one hand-authored scenario (ER chest-pain
triage). This sweep grounds horismos's architecture — the ontology-first workflow model, the
two declared LLM boundaries, the validation gate, truth maintenance, formal verification,
and the SNOMED-anchored subsumption pattern — against what researchers and industry actually
do, worldwide, in clinical decision support, neuro-symbolic AI, clinical AI validation,
terminology engineering, and ontology-authoring/learning. It also covers a 6th question the
captain added mid-sweep (agent-authored and learnable ontologies, since horismos wants that
path eventually via PRP-007) and files one captain-supplied URL as a worked example of a
proposed intake convention.

**Method.** I read `README.md`, [`PRPs/README.md`](https://github.com/billaram/horismos/blob/a164db4/PRPs/README.md), [`PRPs/BOARD.md`](https://github.com/billaram/horismos/blob/a164db4/PRPs/BOARD.md), `PRP-000`, `PRP-007`,
and [`docs/framework/00-index.md`](https://github.com/billaram/horismos/blob/a164db4/docs/framework/00-index.md)/`05-workflows.md`/`06-llm-boundaries.md` first, so every
comparison below is against what horismos *actually* specifies, not a generic description.
I then ran six parallel research agents (one per question below, each independently briefed
on horismos's real architecture) with extensive WebSearch/WebFetch, plus one resumed run and
one nested sub-agent where a first attempt stalled. Every claim below is tagged
**[measured]** (a cited paper, spec, or documented case) or **[inference]** (synthesis/
judgment, mine or the researching agent's) — kept visibly separate throughout. One research
agent (Q3) initially returned a non-answer and had to be resumed with an explicit correction;
that failure and recovery is recorded honestly rather than papered over, because it's the
kind of process fact a report claiming "every claim sourced" should not hide.

---

## 1. Executive reality-check

### 1.1 The workflow model: closer to the losing side of a 20-year natural experiment

**[measured]** Horismos's step graph (`PerceptionStep`/`ReasoningStep`/`ActionStep`/
`NarrationStep`/`ProposalStep`/`ChoiceStep`/`WaitStep`/`SubflowStep`) is, structurally, a
**Task-Network Model** — the same family as GLIF/GLIF3 (Action/Conditional/Branch/
Synchronization steps), PROforma (Action/Decision/Enquiry/Plan), Asbru, and EON
([PMC6172935](https://pmc.ncbi.nlm.nih.gov/articles/PMC6172935/)). This isn't a novel shape;
it's the field's converged-upon answer to "a guideline is a graph of typed steps," reached
independently by at least five research groups from 1995–2010.

**[measured]** None of those TNM formalisms reached sustained production, multi-site use.
GLIF's own founding pilot study found that even *within one collaboration*, two encoders of
the same guideline produced non-interchangeable representations, because of an
**"encoding-variability" and "vocabulary problem"** its own authors named explicitly
([PMC61313](https://pmc.ncbi.nlm.nih.gov/articles/PMC61313/)). EON's clinical instantiation
(ATHENA-HTN, a real multi-site VA RCT) found decision support alone produced **no significant
difference** in blood-pressure control versus a reminder-only control arm
([PMC2243943](https://pmc.ncbi.nlm.nih.gov/articles/PMC2243943/)). SAGE, a 5-year,
NIST-funded, six-institution consortium explicitly built to solve what GLIF and Arden
couldn't (vendor-CIS integration + standard terminology), has no evidence of any
continuation after its funding ended around 2006 — it appears to have quietly stopped rather
than been documented as a failure. GLARE, a 7-year Turin academic collaboration, is
self-described by its own authors as a "domain-independent prototypical system" — research
language, not production language.

**[measured]** By contrast, the formalisms that *did* reach real, regulatory-mandated,
national scale — CQL (CMS-mandated quality reporting), CDS Hooks (broad EHR-vendor support:
Epic, Cerner, Allscripts, athenahealth), and FHIR PlanDefinition/ActivityDefinition (the
substrate for HL7's Da Vinci prior-authorization implementation guides) — won specifically by
being **thin, composable layers on top of FHIR and the already-dominant vendor CIS**, not by
trying to own the whole domain+rules+workflow stack as one artifact. GLIF, Asbru, EON, and
SAGE tried to be that single owning artifact, and stayed academic.

**[inference — the single most important comparative judgment in this sweep, flagged as such
by the researching agent, and endorsed here]** Horismos's core bet — one ontology owning
domain, rules, *and* workflow, with its own World/gate/trace substrate rather than
delegating to FHIR resources and vendor CIS integration — structurally resembles the
**losing side** of a 20-year natural experiment the field has already run, even though its
internal engineering (truth maintenance, a uniform validation gate, formal pre-execution
verification) is more rigorous than any predecessor's. This is not a reason to abandon the
design — see §1.2 for what's genuinely new — but it is a reason the captain should expect
"how does this integrate with an existing EHR/CIS" to be the first hard question any
credible reviewer asks, and to have an answer ready before it's asked.

### 1.2 What's genuinely novel, and why it matters

Three horismos mechanisms have **no precedent** in the CIG formalism literature, and each
directly targets a documented, named historical failure:

- **`PerceptionStep` answers a gap GLIF's own authors explicitly flagged as unsolved in the
  1990s.** Their paper left eligibility criteria as unstructured narrative text because,
  verbatim, *"we do not have natural-language processors that can parse and make sense out
  of such clauses"* ([PMC61313](https://pmc.ncbi.nlm.nih.gov/articles/PMC61313/)). Horismos's
  perception boundary — confidence-gated, JSON-Schema-forced, bounded-retry, mandatory
  fallback — is a three-decades-later, appropriately distrustful answer to a gap the field
  named and then left open for a generation.
- **The uniform validation gate (every mutation — perception, rule, tool, API — passes the
  same gate) has no analog anywhere surveyed.** Arden Syntax's curly braces are the literal
  *opposite*: an unguarded, per-institution bypass around standard rule logic, and that
  bypass is exactly what defeated Arden's portability
  ([Samwald et al. 2012](https://pubmed.ncbi.nlm.nih.gov/22342733/)).
- **Truth maintenance with fixed-point recomputation on every transaction has no CIG
  precedent at all** — it's closer to classical AI truth-maintenance-system theory than to
  anything in Arden/GLIF/PROforma/Asbru/EON/SAGE. This is a direct, structural answer to the
  single best-documented empirical CDS failure mode in the whole sweep: Wright et al.'s
  taxonomy of 68 alert malfunctions across 14 US sites, "widespread and often persist[ing]
  for long periods" (JAMIA, [doi:10.1093/jamia/ocx106](https://dx.doi.org/10.1093/jamia/ocx106)),
  and a companion single-site OHSU/Epic study finding 4 confirmed malfunctions **none of
  which had ever been detected before the retrospective study**, all caused by *unrelated*
  system changes silently breaking a rule
  ([PMC6220702](https://pmc.ncbi.nlm.nih.gov/articles/PMC6220702/)) — the field's own rule of
  thumb is 1–2 silent malfunctions per year for every 200+ alerts running.
- Separately, formal (BDD-style) pre-execution verification of a rule set revives a research
  line — model checking of Asbru guidelines with a purpose-built temporal logic
  ([spinroot.com](https://spinroot.com/spin/symposia/ws06/027.pdf)) — that stayed in small
  academic case studies and never reached production tooling anywhere. If horismos
  operationalizes this at real rule-set scale, that is a genuine, not-yet-industrialized
  contribution, but it inherits that research line's open, unanswered question: model
  checking approaches to guideline verification have not been shown to scale past small
  case studies, and state-space explosion is the generic risk as rule-set size grows.

### 1.3 Five named risks the field's history predicts, in order of how directly they threaten horismos

**[inference, synthesized from converging evidence across the Q1 and Q6 sweeps]**

1. **Authoring/knowledge-acquisition burden at real breadth.** Every CIG project surveyed —
   PROforma's own evaluators, GLIF's pilot, a 2024 multimorbidity project that needed 296
   hand-authored CDS rules across just 25 source guidelines
   ([PMC11163046](https://pmc.ncbi.nlm.nih.gov/articles/PMC11163046/)) — reports that turning
   guidelines into formal representation is slow and expert-intensive even at narrow scope.
   Horismos has exactly one hand-authored workflow. The jump from "1 flagship scenario proves
   the model" to "N scenarios at real institutional breadth" is precisely where SAGE, GLIF,
   and Asbru stalled.
2. **Maintenance/versioning as source guidelines evolve.** The LASSIE case study found only
   133 of 515 sentences in one guideline update were inheritable unchanged — the rest needed
   re-formalization ([PMC2859225](https://pmc.ncbi.nlm.nih.gov/articles/PMC2859225/)).
   Horismos's "formally reverify before running" model has no described incremental/diffing
   reverification workflow yet; naive full reverification on every ontology edit risks
   scaling maintenance cost with ontology size, not with the size of the actual change.
3. **Escalation/fallback fatigue is a separate design problem formal correctness does not
   solve.** The triage example's "escalate to nurse on low confidence" fallback is
   structurally an alert. Systematic reviews find drug-drug-interaction alert override rates
   near 90%, with 46–96% override rates across alert types
   ([journals.sagepub.com/10.1177/14604582241263242](https://journals.sagepub.com/doi/10.1177/14604582241263242)),
   and Kawamoto et al.'s foundational 70-RCT review found only four features predict CDS
   success — none of them "logical correctness"
   ([Kawamoto 2005](https://pubmed.ncbi.nlm.nih.gov/15767266/)). Correctness of the reasoning
   does not, by itself, prevent alert volume from swamping clinician trust.
4. **No demonstrated EHR/vendor-CIS integration story** — see §1.1. This is the single
   biggest structural gap between horismos's current design and how the field's winners
   actually won.
5. **Trust/clinician-acceptance barriers are largely orthogonal to representation quality.**
   Multiple systematic reviews find physician attitude and workflow-disruption risk dominate
   adoption independent of how well-engineered the underlying logic is. No amount of formal
   verification addresses this class of barrier.

### 1.4 "Neurosymbolic AI" is honest but risks a specific, answerable objection

**[measured]** The academic field that currently owns the term "neurosymbolic AI" — Scallop
(UPenn, [arXiv:2304.04812](https://arxiv.org/abs/2304.04812)), Logical Neural Networks
(MIT-IBM, [arXiv:2006.13155](https://arxiv.org/abs/2006.13155)), and DeepProbLog/ProbLog
(KU Leuven, actively funded through an ERC Advanced Grant awarded 2023) — is unified by
making symbolic structure **differentiable and probabilistically weighted**, not by hard-
gating a trained model's output before it acts. Horismos deliberately does the opposite:
confidence gates *acceptance*, never *weights inference*, and there is no fuzzy logic
anywhere in the reasoner (per [`docs/framework/06-llm-boundaries.md`](https://github.com/billaram/horismos/blob/a164db4/docs/framework/06-llm-boundaries.md) and confirmed by every
research agent independently). By the standard the field's own most-cited reference systems
set, a hard, non-differentiable wall between neural and symbolic components is a **looser**
claim to the "neurosymbolic" label than what Scallop/LNN/DeepProbLog actually do — and,
tellingly, Wikipedia's own overview of neuro-symbolic AI contains **no mention of MYCIN,
Arden Syntax, or classical expert systems** at all
([en.wikipedia.org/wiki/Neuro-symbolic_AI](https://en.wikipedia.org/wiki/Neuro-symbolic_AI))
— the field does not see itself as continuous with that lineage, and horismos's reasoning
core (discrete forward/backward chaining, zero probability in inference, proof traces) is
architecturally continuous with exactly that lineage: MYCIN's backward chaining and
explanation-from-trace, and Arden Syntax's Medical Logic Modules (still an active HL7
standard, v3.0 published October 2023).

**[inference]** If forced to pick one lineage for the whole system: horismos's **symbolic
core** is classical rule-based CDS / expert-system lineage, substantially modernized (real
formal verification and truth maintenance where MYCIN had certainty-factor arithmetic and
Arden had no cross-MLM consistency guarantee). Horismos's **neural-boundary engineering**
(ontology-derived schema + hard gate + bounded retry + mandatory fallback, applied
uniformly to every mutation) is genuinely closer to where 2025–2026 formal-agent-
verification research is heading — Lean4Agent uses Lean4 dependent types to verify agent
workflow structure before execution ([arXiv:2606.06523](https://arxiv.org/abs/2606.06523),
reporting ~12% better SWE-Bench-Verified outcomes for verified vs. unverified workflows) —
than to anything MYCIN, Arden, or any current clinical AI product or agent framework ships
today. That half of the design earns "novel." Calling the whole thing "neurosymbolic AI"
unqualified invites a direct, informed comparison to Scallop/LNN/DeepProbLog that horismos
will lose on its own terms (no differentiability, no gradient training). The more defensible
claim, and a more differentiated one: **name both lineages explicitly** — formally-verified,
expert-system-style symbolic reasoning, gated at its only two neural boundaries by a
mechanism closer to where formal agent-verification research is heading than to anything
currently shipping. Notably, horismos's own stated future research direction — "provenance
semirings... a nod to Scallop-style differentiable provenance" — is, concretely, "become
more like Scallop." That would be a real architectural fork away from the current
hard-separation design, not an incremental add-on, and is worth naming explicitly the next
time it comes up in a roadmap conversation.

**[measured]** Concrete evidence for why the hard-gate design matters, even though no
horismos-specific number exists yet: a Mount Sinai adversarial study planted a single fake
lab value, sign, or diagnosis into 300 clinical vignettes; six leading LLMs repeated or
elaborated the planted error in **50–83% of cases**, and prompt engineering reduced but did
not eliminate this (*Communications Medicine*, Aug 2025,
[nature.com/articles/s43856-025-01021-3](https://www.nature.com/articles/s43856-025-01021-3)).
Separately, one **genuine, deployed, quantified precedent for symbolic gating beating an
ungated LLM baseline** exists: an oncology clinical-trial-matching system where LLM-extracted
patient features must pass explicit graph-enforced numeric/Boolean/temporal consistency
checks before a match surfaces, reporting F1 0.82 versus 0.47 (GPT-4 zero-shot) / 0.67
(GPT-4 chain-of-thought) on a prospective 3,804-patient evaluation
([PMC13091143](https://pmc.ncbi.nlm.nih.gov/articles/PMC13091143/)). **No head-to-head
"hard-gated ontology system vs. RAG-grounded LLM" clinical trial exists anywhere in the
literature surveyed** — horismos's safety argument is currently architectural, not yet
measured, for itself specifically.

**[inference]** Industry offers no ready-made competitor to imitate or fear here: none of the
major clinical AI scribe/agent vendors surveyed (Abridge, Ambience Healthcare, Nuance DAX,
Epic's own AI features) publicly document a formal or symbolic verification layer — all rely
on retrieval/traceability plus human sign-off as the actual safety mechanism. No company
surveyed brands itself "neurosymbolic clinical AI" or "ontology-grounded clinical agent" as
a core identity (IMO Health uses that language in blog marketing attached to a terminology
product, not a shipped gated-agent architecture). Among generic LLM agent frameworks
(LangGraph, Google ADK, OpenAI Agents SDK, Microsoft Agent Framework, CrewAI), only Google
ADK draws a genuine hard line — "Workflow Agents" execute without consulting a model — but
only over fixed orchestration scaffolding, never over an LLM node's semantic content; OpenAI's
own docs confirm its Guardrails are soft and post-hoc (tools "may have already... executed"
before a tripwire cancels anything). Apollo-1/AUI (cited in horismos's own PRP-000 comparison
set) markets itself as "the first neuro-symbolic model" with GA "scheduled for Q2 2026," but
no published technical specification, benchmark, or proof of an actual hard-gate mechanism
was found for it — a genuine example, in a system horismos already names as a comparator, of
"neurosymbolic" used as positioning language without documentation. **This gap — genuine
pre-execution formal gating of LLM output, applied uniformly, in a shipping product — is
real and currently open.** Horismos is not competing against an incumbent here; it's racing
absence.

### 1.5 SNOMED CT: the pattern is right, the current fixture already shows the failure mode

**[measured]** Horismos's approach — a hand-picked SNOMED subset compiled offline into a
`.ttl` import, reasoned over by horismos's own Rete-lite engine — is architecturally
*correct*, not an anti-pattern, and is arguably the **only** pattern compatible with
horismos's own hard rules: `horismos-core` is deliberately I/O-free during inference, and
"trace equality is a valid test assertion" is flatly incompatible with a live network call to
a terminology server (Snowstorm, Ontoserver) during reasoning. Compiling the closure once,
offline, per release, and serving it as static versioned data is exactly what production
terminology servers do internally — SNOMED International's own vendor guidance calls
infrequent-update custom subsets *"a consistent weakness of existing systems not using a
terminology server"* while explicitly recommending a terminology server + ECL/ValueSets over
hand-rolled subsets
([docs.snomed.org](https://docs.snomed.org/implementation-guides/implementation-fact-sheets/implementation-dilemmas/do-i-need-an-external-interface-terminology-to-implement-snomed-ct)).
Horismos's pattern of compiling *before* the runtime, not *during* it, is structurally
analogous to what that guidance recommends — it just needs the compilation step built.

**[measured — I verified this directly against the repo, not from general description]**
[`examples/triage/snomed-ct-subset.ttl`](https://github.com/billaram/horismos/blob/a164db4/examples/triage/snomed-ct-subset.ttl) today ships ~50 hand-picked concepts using
**single-parent `rdfs:subClassOf` only** — real SNOMED CT is a true polyhierarchy (most
concepts have multiple `is-a` parents by design; ~368,000 active concepts as of the June 2024
figure in SNOMED's own guidance) and the shipped fixture represents none of that structure.
Its own header comment admits this candidly: *"the fixture's point is the structure, not
registry fidelity."* That's a defensible statement about a demo fixture, but it means the
exact risk this research question asked about — "polyhierarchy edge cases the ontology
author's hand-picked subset might not represent correctly" — is not hypothetical. It is
visible in the shipped example today, and reasoning that depends on real subsumption reaching
the right ancestor will silently encode whichever single parent an author happened to pick.
There is also no version/release provenance recorded anywhere in the fixture, and SNOMED CT
retires/replaces roughly a quarter of its concept space on an ongoing basis
([confluence.ihtsdotools.org](https://confluence.ihtsdotools.org/display/DOCEG/Concept+Inactivation)),
so a hand-copied subset silently drifts stale with no mechanism to detect it.

**[measured]** A directly on-point study quantifies exactly how much a hand-curated
("extensional") subset loses relative to a proper subsumption-based ("intensional") one:
extensional value sets captured a **median of only 65%** of clinically relevant SNOMED terms
that an equivalent ECL-based hierarchy definition would capture — **as low as 2% for
hypertension specifically** — while being far less concise (78 vs. 3 concepts, median) and
slower to author (37 vs. 5 minutes, median)
([Chu et al., JMIR Medical Informatics, Jan 2019, PMC6351992](https://pmc.ncbi.nlm.nih.gov/articles/PMC6351992/)).
This is, concretely, the risk horismos's own reasoning engine is exposed to *today* through
its shipped fixture, on a scenario horismos itself uses as the flagship example.

**[inference]** Recommendation: don't change the offline-compiled-subset architecture — it's
correct. Do treat extraction as a build-time, versioned, reproducible pipeline (an ECL
expression plus a pinned SNOMED edition/version as the actual source of truth, ideally
backed by a self-hostable terminology server such as Snowstorm), preserve full multi-parent
polyhierarchy in the generated output rather than collapsing to one parent, and stamp
provenance (source edition/version + generating ECL expression) into the subset file so
staleness is mechanically detectable. This is exactly `search_vocab`/PRP-007's stated scope
— the finding is that its output format needs to support polyhierarchy and provenance
metadata, not just an `owl:sameAs` anchor. Separately, and independent of the technical
pattern: whoever ships a horismos ontology bundle **containing real SNOMED CT content** to
third parties is themselves distributing SNOMED CT and needs their own SNOMED International
Affiliate status with sub-licensing obligations to *their* downstream users
([docs.snomed.org licensing](https://docs.snomed.org/snomed-ct-practical-guides/vendor-introduction-to-snomed-ct/7-licensing))
— that's a deploying-org obligation horismos-the-framework cannot discharge on anyone's
behalf, worth a line in [`docs/framework/02-ontology.md`](https://github.com/billaram/horismos/blob/a164db4/docs/framework/02-ontology.md) or an ops guide before this becomes
someone's surprise.

### 1.6 Validation: horismos is at rung −1, and formal verification does not prove the medicine is right

**[measured]** IBM Watson for Oncology is the load-bearing cautionary case for horismos's
*current* risk profile, more than any statistical-ML case study. STAT News's 2018 reporting
on internal IBM slide decks found Watson had produced "unsafe and incorrect" treatment
recommendations, traced to a root cause that is not a generalization failure but a
**hand-authored-knowledge failure**: the system was trained on a small number of *synthetic*
(hypothetical) cases rather than real patient data, and encoded the personal treatment
preferences of a handful of Memorial Sloan Kettering specialists rather than guideline
consensus ([STAT News, 2018-07-25](https://www.statnews.com/2018/07/25/ibm-watson-recommended-unsafe-incorrect-treatments/)).
That is mechanically horismos's exact current shape: one hand-authored ontology, built from
one team's understanding of one scenario, with no external clinician review yet. Horismos's
own verification documentation ([`docs/framework/10-verification.md`](https://github.com/billaram/horismos/blob/a164db4/docs/framework/10-verification.md)) already states the
limit plainly: verification proves the reasoner is total and sound *around whatever
perception does*, and claims nothing about tool implementations or anything probabilistic.
This report adds the clinically load-bearing corollary explicitly, because it is easy to
elide in practice: **verification proves the authored rule set is complete, conflict-free,
and terminating relative to itself — it does not and cannot prove the rules encode correct
medicine.** A perfectly verified ontology can exhaustively, consistently, and provably encode
a wrong chest-pain risk-stratification threshold. Property tests built purely against the
verifier's own witnesses would pass just as cleanly on a clinically wrong ontology as a
correct one.

**[measured]** Independent external validation, when it happens, tends to be unflattering
relative to internal/vendor numbers: the Epic Sepsis Model, deployed to hundreds of US
hospitals with **no published external validation before that scale**, was independently
retrospectively validated at Michigan Medicine and found to have AUC 0.63, sensitivity 33%,
PPV 12% — clinicians needed to evaluate 109 alerts to find one true, previously-untreated
case
([Wong et al., JAMA Internal Medicine, 2021](https://jamanetwork.com/journals/jamainternalmedicine/fullarticle/2781307)).
A 2024 NEJM AI reanalysis found much of its apparent accuracy was label leakage — encoding
"a clinician already suspects sepsis" rather than physiology — with AUROC collapsing to
0.47 (coin-flip) once post-suspicion data were excluded
([NEJM AI 2024](https://ai.nejm.org/doi/full/10.1056/AIoa2300032)).

**[inference — proposed validation ladder]** Given where horismos actually is (pre-alpha,
one hand-built scenario, deterministic replay, and formal rule verification as core, already-
built capabilities), a credible ladder runs:

| Rung | Activity | What it buys | What it explicitly does not buy |
|---|---|---|---|
| 0 | Adversarial synthetic case suites + property-based testing, using the BDD verifier's own completeness witnesses as a test-case generator | Regression testing via byte-identical trace equality — cheap, hard pass/fail, no statistical noise | Any evidence the rules are clinically correct |
| 1 | Expert clinician red-team of the ontology's rules + verification witnesses (2–3 clinicians, different institutions) | Directly targets the IBM-Watson-style narrow/idiosyncratic-encoding failure; proof trees and domain-text witnesses are far cheaper for a clinician to review than a black-box model's SHAP values — this operationalizes the FDA's CDS-exemption language ("independently review the basis") as a design property | Nothing about statistical generalization |
| 2 | Retrospective replay against a public de-identified dataset slice matching the scenario (e.g. MIMIC-IV's ED module, not eICU-CRD which is ICU-only) — explicitly re-run the exact label-leakage check that broke the Epic Sepsis Model | Internal-validation confidence; a concrete, checkable leakage test | Calibration/generalization across sites — internal-only validation systematically overstates real-world reliability per the cross-institutional MIMIC-IV↔eICU literature |
| 3 | Silent-mode/shadow run alongside a real (non-critical or non-acting) workflow, clinicians blinded, full trace capture | Determinism gives a real edge here that black-box shadow evaluation lacks: no random-seed/sampling confound, so any discrepancy is fully attributable to an exact rule via exact replay | Alert-fatigue economics and label leakage in real data — properties of deployment, not of symbolic-vs-statistical reasoning |
| 4 | Prospective/RCT, scoped honestly as a process-outcome comparison first (e.g. time-to-review), not a mortality trial | Only 0.2% of even FDA-cleared AI devices ever reach this rung ([PLOS Digital Health 2025 evidence census](https://journals.plos.org/digitalhealth/article?id=10.1371%2Fjournal.pdig.0001597)); a null RCT result (e.g. the well-powered DDX-BRO cluster RCT, [Lancet Digital Health](https://www.thelancet.com/journals/landig/article/PIIS2589-7500(24)00250-4/fulltext)) is a real, common, non-failure outcome to plan for | — |

**[measured]** On regulatory shape: the FDA's non-device CDS exemption (21st Century Cures
Act §520(o)(1)(E)) requires software to "enable a health care professional to independently
review the basis for the recommendation" — horismos's proof-tree transparency is structurally
favorable to that language in a way a black-box classifier is not, but this does not mean
automatic qualification: a time-critical acuity assignment (like the current triage scenario)
plausibly fails the exemption's "not primarily relying on the software" prong regardless of
transparency. Separately, the FDA's Predetermined Change Control Plan guidance, finalized
December 2024, maps unusually well onto horismos's own ontology-as-versioned-artifact model —
a PCCP for horismos would plausibly describe bounded *ontology* edits (new rules, threshold
tweaks) verified pre-deployment and diffed against BDD witnesses, a genuinely strong, evidence-
backed story to have ready for a regulator much later, not now.

### 1.7 Agent-authored ontologies (PRP-007): the one finding sharp enough to change the plan

**[measured — the single most concrete, actionable finding in the whole sweep]** The formal-
code-verification literature has a named failure mode with no ontology-specific precedent yet
but an exact structural analog in horismos: **"vacuous verification."** Models can satisfy a
formal verifier with a trivial specification that technically passes but constrains nothing —
named and explicitly designed against in multiple 2025–2026 papers (LASA
[arXiv:2506.17865](https://arxiv.org/abs/2506.17865); CHARGE
[arXiv:2607.27776](https://arxiv.org/abs/2607.27776); "From Natural Language to Verified Code"
[arXiv:2604.22601](https://arxiv.org/abs/2604.22601)). This maps exactly onto a horismos rule
an LLM-authoring agent drafts that never fires, or a constraint that is vacuously satisfiable
— internally consistent, BDD-verifiable, and useless or wrong. PRP-007 as currently scoped has
**no explicit non-vacuity check separate from consistency-checking** — the recommendation is
to add one: does each drafted rule/constraint actually bind under some reachable scenario, as
a distinct verification pass.

**[measured]** Lahiri (2024) states the structural ceiling directly: *"there's no algorithmic
guarantee that formal specifications accurately capture user intent"*
([arXiv:2406.09757](https://arxiv.org/abs/2406.09757)) — the verifier checks internal
consistency against a spec, not whether the spec matches what the SME meant. The
autoformalization literature has its own name for the identical gap: the **"formalization
gap"** between a statement being provable/verifiable and being faithful to what was informally
meant ([ReForm, arXiv:2510.24592](https://arxiv.org/abs/2510.24592)). PRP-007's design
already, correctly, treats SME sign-off as a separate, non-bypassable gate from verification —
per this literature that's the right response to a structural limit, not redundant caution.

**[measured, and directly undercuts an implicit assumption in PRP-007's design]** PRP-007's
whole review surface rests on the SME seeing only a rendered NL summary plus scenario-probe
proof trees, never raw Turtle — betting that narration quality is what keeps this safe.
Chain-of-thought faithfulness research says this bet should not be made lightly: Turpin et al.
(2023) showed CoT explanations "can systematically misrepresent the true reason for a model's
prediction" while remaining fluent and plausible
([arXiv:2305.04388](https://arxiv.org/abs/2305.04388)), and — more pointedly — Lanham et al.
(2023) found **larger, more capable models produce *less* faithful reasoning, not more**
([arXiv:2307.13702](https://arxiv.org/abs/2307.13702)), a finding a 2025–2026 cluster
(FaithCoT-Bench, C2-Faith) confirms remains open and unresolved. Upgrading the narration model
would not automatically make PRP-007's review surface safer, and no paper in this sweep
specifically studies faithfulness of NL renderings of *separately-verified formal artifacts*
(as opposed to general CoT reasoning traces) — this is a genuine, currently untested gap in
exactly the mechanism PRP-007 depends on. The recommended mitigation is empirical, not
architectural: extend PRP-007's own planned DoD test (seeded incompleteness caught by the
verifier) to also seed *subtly-wrong-but-internally-consistent* axioms, and measure whether
SMEs using the rendered summary + scenario probes actually catch them — treat this as an
assumption to test, not a solved problem.

**[measured, on the "learnable ontologies → world models" framing the captain flagged]** A
captain-supplied source (Vasilije Markovic/Cognee,
[LinkedIn](https://www.linkedin.com/pulse/how-do-you-go-from-learnable-ontologies-world-models-markovic-flqtf/),
Aug 2026 — read and filed as the intake convention's worked example, §3) argues agent memory
can evolve toward a world model via a continuous, self-reinforcing loop: ingest traces → learn
ontology → let that ontology shape future ingestion → enrich → repeat, with regularities over
state transitions eventually approximating predictive dynamics. The article's own commenters
flag the obvious risk — unsupervised drift in a self-reinforcing loop, and a blurred line
between prediction and authorization-to-act. That risk is not hypothetical: the field's
longest-running real example of largely-autonomous knowledge extension, NELL (CMU, running
continuously since 2010), needed human correction "every few weeks" and Stuart Russell reports
it has confidence in only 3% of its own beliefs; NELL's own documented drift chain — internet
*cookies* miscategorized as *baked goods*, then further deduced into *computer files* by
pattern-matching sentence structure — is the clearest concrete evidence of unchecked schema
drift found anywhere in this sweep
([en.wikipedia.org/wiki/Never-Ending_Language_Learning](https://en.wikipedia.org/wiki/Never-Ending_Language_Learning)).
Wikidata, the largest real, continuously-evolving public knowledge graph, achieves reliability
by the opposite mechanism: mandatory human community discussion before any schema-level
change, with only instance-level data left to bots.

**[inference]** Horismos's existing design already has a structural answer to both risks the
Markovic piece's commenters raised, and it's worth stating plainly: traces are one-directional
output (the audit log and training corpus, never an input that mutates the ontology), and no
action ever fires off a raw prediction — only off symbolically-checked preconditions. Where
horismos is genuinely *behind* Cognee's ambition is the learning axis: v1 has no path at all
from accumulated execution traces back into candidate ontology edits. The right synthesis for
PRP-007, and the concrete way to "map the path to agent-authored ontologies" the captain
asked for: don't imitate Cognee's auto-reinforcing loop. Treat trace-mining as a new
**proposal source** feeding the *existing* gated authoring loop — verifier + SME sign-off
stays the only path any ontology change takes, exactly matching horismos's own "neural
proposes, symbolic decides" principle applied one level up, at the authoring layer instead of
just the runtime layer.

**[measured]** Elsewhere in the landscape: the field's own systematic literature review (Li,
Garijo & Poveda-Villalón, *Semantic Web Journal*, 2026, screening 15,688 papers down to 36)
found **no standardized benchmarks, inconsistent metrics, poor reproducibility, and minimal
human-in-the-loop integration** across LLM-based ontology engineering generally — PRP-007's
hard verifier-plus-sign-off design is stricter than most of the field's own practice, not an
outlier of excess caution. And the axiom/rule layer specifically — exactly where PRP-007's
authoring agent operates — is documented as the field's hardest, weakest layer: a 2025/2026
axiom-identification benchmark across 12 LLMs and 2,771 axioms found F1 scores ranging 0.218
to 0.642 by domain, concluding performance is **"not high enough to fully automate axiom
identification"** ([arXiv:2512.05594](https://arxiv.org/abs/2512.05594)) — the strongest
evidence-based argument *for* keeping PRP-007's gate hard rather than lightening it. No system
anywhere in the literature was found autonomously extending its own ontology with zero human
review in real deployment; PRP-007's hard sign-off gate is, if anything, ahead of the field's
own best practice on that axis.

---

## 2. Scenario catalog: what's beyond triage, and the 3 recommended next scenarios

A comparison across the field's most-studied clinical decision workflows, scored against what
horismos's architecture specifically needs: (a) bounded, rule-representable logic; (b) inputs
realistic for structured facts or a short-NL perception step; (c) well-typed tool-effect
actions; (d) stakes worth formally verifying; (e) prior art (FHIR PlanDefinition/CQL/published
protocol) to ground the ontology's vocabulary.

| Scenario | (a) Bounded? | (b) Perception-realistic? | (c) Well-typed actions? | (d) Stakes? | (e) Prior art? | Automation evidence | Verdict |
|---|---|---|---|---|---|---|---|
| **VTE prophylaxis ordering** | Yes — Caprini/Padua scoring + contraindication check | Yes — almost entirely structured EHR facts | Yes — order enoxaparin/SCDs | Yes — leading preventable cause of hospital death | Strong (CHEST/ACCP guidelines) | **Strong positive**: [Kucher et al., NEJM 2005](https://www.nejm.org/doi/full/10.1056/NEJMoa041533) RCT (1,255 vs. 1,251 patients) — computer alert markedly reduced DVT/PE | **#1 recommend** |
| **NEWS2 deterioration + RRT escalation** | Yes — literal published points table | Yes — periodic vitals, structured | Yes — page RRT (same shape as triage's nurse page) | Yes — "failure to rescue" is a named safety crisis | Strong (NHS/RCP-published spec) | Mixed on outcomes, but root cause is documented as **protocol non-adherence**, not scoring failure ([systematic review](https://link.springer.com/article/10.1186/s13643-026-03088-y)) | **#2 recommend** |
| **Sepsis bundle (SSC) compliance-tracking** — reframed away from prediction | Yes, scoped to bundle timing/checklist, not prediction | Yes — order/lab/med timestamps | Yes — reassessment/escalation actions | Yes — the field's highest-profile CDS failure *and* a positive counter-example | Strong (SSC is a published, timed checklist) | Predictive variant (Epic Sepsis Model) failed badly and at scale; compliance-tracking has separate positive evidence and a positive prospective precedent ([TREWS, Nature Medicine 2022](https://www.nature.com/articles/s41591-022-01894-0), ~18% relative mortality reduction on confirmed alerts) | **#3 recommend** |
| Prior authorization automation | Yes structurally (Da Vinci CRD/DTR/PAS) | Yes | Yes | High-stakes, adversarial not clinical | Very strong — this is literally the Da Vinci use case | **Severe documented harm** from opaque automation: UnitedHealth/naviHealth nH Predict litigation (alleged "known 90% error rate"), Cigna PXDX (ProPublica: 1.2 sec/claim, batch-denied), active 2026 Senate/OIG inquiry | **Rule out for now** — see below |
| Imaging appropriateness (ACR AUC) | Yes — literal lookup table | Yes | Yes | Moderate–high | Very strong — the FHIR CRD/DTR use case | Positive where deployed, but CMS **paused/rescinded** the federal AUC program in CY2024 — not for clinical reasons but because "the claims system cannot perform this function effectively" | Sound logic, bad timing — deprioritize |
| Chronic-care pathway (diabetes/HTN/HF) | Partial — order-set level only | Mixed, longitudinal | Weak — "recommend" isn't a typed effect | Moderate | Strong (CQL/CPG IG exists) | Weak: 76-study JAMIA meta-analysis found small, heterogeneous effects and **no CDS design feature reliably predicted success** | Rule out |
| ED disposition | No — unbounded gestalt | Weak — social/logistic factors aren't perceivable facts | Weak | High-stakes | Weak — existing tools (STRATIFY) are statistical carve-outs, not rules | Narrow low-risk subsets show promise; general disposition doesn't | Rule out |
| ICU ventilator weaning | No — needs continuous titration | **Poor** — continuous streaming, not discrete perception | Ambiguous | High | Some | Strong positive effect (wean time 4d→2d) | **Rule out** — wrong shape entirely for horismos's discrete-step model |
| Medication reconciliation | Partial — comparison problem | Good NL fit, but needs a huge external drug DB | Weak — "flag discrepancy" isn't world-mutating | High | Moderate (RxNorm, not ontology-native) | Positive (AHRQ), but the mechanism doesn't match horismos's reasoning model | Rule out |
| Antimicrobial stewardship | Partial — restriction-tier logic only | Structured | Yes | Moderate | Some | Modest: 30-trial meta-analysis found median only 3% improvement, "raising questions as to clinical significance" | Plausible 4th choice, not top 3 |
| Fall-risk / delirium screening | Weak — mostly statistical scoring | Weak evidence base | Weak | Moderate | Weak | Thin/early-stage evidence | Rule out for now |

**Recommendation, in order:**

1. **VTE prophylaxis risk-stratified ordering** — the closest available match to triage's
   existing shape, reusable almost entirely via horismos's already-built "structured-input
   bypass" (facts asserted directly with `provenance: api`, no Bridge configured at all),
   which lets this be built and formally verified as a fully symbolic deployment first,
   adding an optional NL perception step later. Diversifies the domain vocabulary (CHEST
   guideline terms, Caprini/Padua) away from SNOMED symptom subsumption, and Kucher et al.'s
   RCT gives horismos a real external validation of the *mechanism class* to point to,
   decoupled from any vendor's black-box implementation.
2. **NEWS2 deterioration + rapid-response escalation** — if anything a cleaner fit than
   triage's acuity subsumption, because the scoring is a literal published table rather than
   a rule set horismos has to author from domain judgment. The literature's own documented
   failure mode (good score, poor outcomes, because escalation protocols aren't followed) is
   precisely what horismos's *workflow-level* formalization — not just a risk-score
   calculator — targets: encode the full escalation contract, not just the lookup table, and
   the resulting proof trace becomes audit-grade evidence of protocol adherence, not just
   prediction accuracy.
3. **Sepsis bundle compliance-tracking, explicitly not prediction** — the rhetorically and
   technically sharpest pick for directly rebutting "one hand-authored scenario," because
   sepsis is simultaneously the field's highest-profile CDS *failure* (Epic Sepsis Model:
   deployed to hundreds of hospitals with no external validation before scale) and has a
   credible positive counter-example (TREWS). Horismos's mandatory pre-deployment
   verification and proof trees are architecturally the *opposite* of what went wrong with
   the Epic model. Scope the build to SSC bundle timing/checklist compliance — lactate drawn,
   cultures before antibiotics, antibiotics within window, reassessment documented — never to
   sepsis prediction.

**Explicitly ruled out, with reasons:** prior authorization has the strongest FHIR prior art
of anything surveyed and is precisely why it's a bad *first* choice — the documented harms
came from opaque systems serving an adversarial payer incentive, and deploying a pre-alpha
framework into a litigated, regulator-scrutinized workflow is a liability/maturity mismatch,
not a technical one; it's a strong *future* direction once horismos has an established trust
record. Imaging AUC has solved decision logic but a politically dead federal program. Chronic
care and ED disposition don't match the architecture's bounded, terminal-step shape. ICU
ventilator weaning is the worst architectural fit surveyed despite the strongest trial
evidence — closed-loop continuous titration has no discrete decision point to formalize.

---

## 3. Proposed research wiki: `research/` in the horismos repo

Designed so a weekly research pass appends deltas as small, reviewable PRs, and so the
captain can drop a URL at any time and have it read, positioned, and filed without
interrupting other work — the intake mechanism was used live in this sweep (§3.2) as its own
first worked example.

### 3.1 Directory layout

```
research/
  README.md                    — what this wiki is, how entries are structured, how weekly
                                  passes work, link to this report as the founding snapshot
  CHANGELOG.md                 — append-only, one line per merged PR: date, PR link, summary
  reading-log.md               — the single canonical cross-topic log: Source | Date | Topic
                                  | One-line takeaway | URL | filed-under (path). Append-only.
  _template/
    page.md                    — standard page skeleton: Claim / Evidence / Source /
                                  Confidence (measured|inference) / Last-verified date

  intake/
    README.md                  — the URL-drop convention (§3.2)
    queue.md                   — append-only: {url, dropped_by, date, status}. One row per
                                  drop; status starts "unfiled".
    filed/
      YYYY-MM-DD-<slug>.md     — raw fetch note per processed URL: full summary, author/
                                  affiliation, what it argues, references it points to,
                                  provenance flags (paywalled? vendor-authored? withdrawn?)

  formalization/                — Q1: guideline representation languages & standards
    00-overview.md              — the comparison table + "what failed and why" summary
    arden-syntax.md
    glif.md
    proforma-deontics.md
    tnm-family.md                — Asbru/EON/SAGE/GLARE grouped (shared TNM lineage)
    cql-cds-hooks-fhir.md
    bpm-plus-health.md
    prior-authorization-standards.md

  scenarios/                    — Q2: the scenario catalog
    00-catalog.md                — the comparison table (§2), living document, updated as
                                    new scenarios are evaluated or built
    vte-prophylaxis.md
    news2-deterioration.md
    sepsis-bundle-compliance.md
    ruled-out.md                  — every scenario explicitly ruled out + why, so the
                                     reasoning isn't silently lost on a future revisit

  neurosymbolic-landscape/       — Q3
    00-overview.md                — academic (LNN/Scallop/DeepProbLog/Kautz taxonomy),
                                     industry (vendors, agent frameworks), benchmarks
    lineage-assessment.md         — the "what lineage is horismos really in" analysis (§1.4),
                                     revisited each time positioning language changes

  validation/                    — Q4
    00-ladder.md                  — the proposed validation ladder (§1.6), living/revised as
                                     rungs are actually attempted
    regulatory.md                 — FDA SaMD/CDS-exemption/PCCP tracking
    case-studies.md                — Epic Sepsis Model, TREWS, Sepsis Watch, Watson Oncology,
                                      DDX-BRO, etc. — one section per case, updated as new
                                      external validations are published

  terminology/                   — Q5
    00-overview.md
    snomed-ct.md                   — licensing + subsumption-at-scale + the extraction-
                                      tooling recommendation (§1.5)
    loinc.md
    rxnorm.md

  ontology-authoring/             — Q6
    00-overview.md
    ontology-learning-benchmarks.md
    autonomous-agents-failure-modes.md   — NELL, Wikidata governance, AutoSchemaKG, KARMA
    verification-of-llm-drafted-artifacts.md  — vacuous verification, the formalization gap,
                                                 CoT faithfulness — PRP-007's risk register
    prp-007-open-items.md           — the concrete, actionable gaps (§1.7), tracked until
                                       closed or explicitly deferred
```

### 3.2 The intake convention

A person (most often the captain) can drop a URL into [`research/intake/queue.md`](../intake/queue.md) at any
time, or hand it to an agent directly in conversation. Either way, the next agent session
that notices an `unfiled` row — or receives the URL directly — does:

1. **Fetch and read** the source in full (not just a search snippet).
2. **Summarize** its argument, name the author/affiliation, and — where the source cites or
   obviously implies further sources (references, the author's other work, a company/lab
   behind it) — do one shallow follow-up pass to place it in context, rather than filing it
   as an isolated fact.
3. **Position** it against horismos's actual design (not a generic take) — this is the step
   that makes an intake entry worth more than a bookmark.
4. **File**: write `research/intake/filed/YYYY-MM-DD-<slug>.md` (the raw note), update or
   create the relevant `research/<topic>/*.md` page with a short paragraph incorporating the
   finding, append one row to [`research/reading-log.md`](../reading-log.md), and mark the `queue.md` row `filed`
   with a pointer to the filed note.
5. This is a small PR by construction — one URL in, a handful of file touches out — matching
   the brief's requirement that weekly passes append deltas as small, reviewable PRs.

**Worked example, run live in this sweep:** the captain dropped
`https://www.linkedin.com/pulse/how-do-you-go-from-learnable-ontologies-world-models-markovic-flqtf/`
mid-task. It was fetched and read in full (not just searched); the author (Vasilije Markovic,
founder of Cognee, an open-source AI agent memory platform) and the piece's argument
(traces → learned ontology → reshapes future ingestion → approximates a world model, with an
honest caveat that this isn't yet real predictive dynamics) were identified; one follow-up
pass traced Markovic's own related academic work (arXiv:2505.24478, Cognee's KG-LLM
hyperparameter-tuning paper) and a thematically adjacent piece (Ken Huang, "Why Ontology
Matters for Agentic AI in 2026," framing ontology as a "context compiler"); it was positioned
against horismos's design (§1.7 above — the sharp contrast between Cognee's continuous
self-reinforcing loop and horismos's one-directional, gated design); and it is filed in the
reading log below as the first row under "captain intake." Under the proposed structure, this
would have produced [`research/intake/filed/2026-08-24-markovic-learnable-ontologies.md`](../intake/filed/2026-08-24-markovic-learnable-ontologies.md), an
updated paragraph in [`research/ontology-authoring/00-overview.md`](../ontology-authoring/00-overview.md), and the corresponding
reading-log row — exactly the shape a real weekly-pass PR would take.

---

## 4. Reading log

Every source cited by any research agent in this sweep, or by me directly. Grouped by
research question for scan-ability; `[unverified]` flags a source relayed only via a search
snippet where direct primary-text fetch failed, noted honestly rather than silently upgraded
to a confirmed citation.

### Captain intake (worked example, §3.2)

| Source | Date | Takeaway | URL |
|---|---|---|---|
| Vasilije Markovic, "How do you go from learnable ontologies to world models?" | 2026-08-18 | Cognee founder argues agent memory can approach a world model via a continuous trace↔ontology feedback loop; explicitly hedges this isn't yet real predictive dynamics | https://www.linkedin.com/pulse/how-do-you-go-from-learnable-ontologies-world-models-markovic-flqtf/ |
| Markovic, Obradovic, Hajdu, Pavlovic, "Optimizing the Interface Between Knowledge Graphs and LLMs for Complex Reasoning" | 2025 | Cognee's own KG-LLM hyperparameter-tuning study across HotPotQA/2WikiMultiHop/MuSiQue; gains inconsistent across datasets | https://arxiv.org/abs/2505.24478 |
| Ken Huang, "Why Ontology Matters for Agentic AI in 2026: From World Models to Governable Decisions" | 2026 | Frames ontology as a "context compiler"/operational interface; cites Palantir Ontology, W3C PROV, NIST AI RMF, EU AI Act | https://kenhuangus.substack.com/p/why-ontology-matters-for-agentic |
| Cognee (product/GitHub) | ongoing | Open-source AI agent memory platform, MIT-licensed, $7.5M seed | https://github.com/topoteretes/cognee |

### Q1 — Clinical guideline/workflow formalization

| Source | Date | Takeaway | URL |
|---|---|---|---|
| Wikipedia, Arden Syntax | accessed 2026 | History, real deployments (Regenstrief CARE, LDS HELP, CHICA), curly-braces criticism | https://en.wikipedia.org/wiki/Arden_Syntax |
| Samwald et al., "The Arden Syntax standard for clinical decision support: Experiences and directions" | 2012 | Works for standalone rules; major technical-integration barriers in heterogeneous HIS | https://pubmed.ncbi.nlm.nih.gov/22342733/ |
| "Sharing MLM's: An Experiment between Columbia-Presbyterian and LDS Hospital" | archival | Direct documentation of the curly-braces portability problem | https://pdf.core.ac.uk/corefilesystem/pdf/11926/276277185.pdf |
| Ohno-Machado et al., "The GuideLine Interchange Format" | ~1998 | GLIF's own pilot: encoding variability + vocabulary problem defeated interchange; no NLP existed for eligibility criteria | https://pmc.ncbi.nlm.nih.gov/articles/PMC61313/ |
| GLIF3 evolution paper | 2000 | 3-level architecture: flowchart / verifiable computable spec / implementable spec | https://pubmed.ncbi.nlm.nih.gov/11079963/ |
| "Evaluation of PROforma as a language for implementing medical guidelines" | 2006 | 6-pharmacy hypertension trial; documents authoring-burden limitations | https://pmc.ncbi.nlm.nih.gov/articles/PMC1459125/ |
| Deontics company site | accessed 2026 | PROforma's commercial descendant | https://deontics.com/about/ |
| "Pioneering NHS trust using AI to help doctors make right treatment decisions" | 2023 | Deontics/Guy's & St Thomas' prostate pathway, 33% triage figure (vendor-reported) | https://hospitalhealthcare.com/news/pioneering-nhs-trust-using-ai-to-help-doctors-make-right-treatment-decisions/ |
| Peleg, "Computer-interpretable clinical guidelines: a methodological review" | 2013 | Full lifecycle review across CIG formalisms | https://www.sciencedirect.com/science/article/pii/S1532046413000841 |
| Review of CIGs for multimorbidity management | 2018 | TNM family confirmed; most cited works proof-of-concept, not operational | https://pmc.ncbi.nlm.nih.gov/articles/PMC6172935/ |
| "Versioning computer-interpretable guidelines... LASSIE" | 2010 | 133/515 sentences inheritable on one guideline update | https://pmc.ncbi.nlm.nih.gov/articles/PMC2859225/ |
| "Identifying Barriers to Hypertension Guideline Adherence... ATHENA" | ~2007 | ATHENA-HTN VA trial clinician-feedback analysis | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC1839712/ |
| "Implementing clinical practice guidelines... ATHENA DSS" | ~2008 | No significant BP-control difference from DSS alone vs. control | https://pmc.ncbi.nlm.nih.gov/articles/PMC2243943/ |
| ATHENA-HTN trial registry | — | Multi-site VA RCT record | https://classic.clinicaltrials.gov/ct2/show/results/NCT00374452 |
| SAGE Guideline Model overview | 2007 | 5-year NIST-funded consortium; no post-2006 continuation found | https://www.clinfowiki.org/wiki/index.php/The_SAGE_guideline_model |
| Terenziani et al., "The GLARE Approach to Clinical Guidelines" | ~2004 | 7-year Turin collaboration; self-framed as prototypical | https://people.unipmn.it/alessio/doc/terenziani_CPG.pdf |
| "GEM at 10" | ~2011 | Decade retrospective still asking what limits adoption | https://www.researchgate.net/publication/51921221 |
| CAREPATH study, Frontiers in Medicine | 2024 | 296 CDS rules/326 CDS Hooks cards from 25 guidelines; bespoke co-creation methodology needed | https://pmc.ncbi.nlm.nih.gov/articles/PMC11163046/ |
| "Assessment of 3 standards-based CDS tools... CQL, CDS Hooks, FHIR" | 2025 | 96–100% accuracy; real barrier was EHR data-documentation gaps, not the language | https://pmc.ncbi.nlm.nih.gov/articles/PMC12309839/ |
| CDS Hooks specification home | 2025 | v2.0.1 STU2; normative ballot planned 2025 | https://cds-hooks.hl7.org/ |
| FHIR Clinical Reasoning Module spec | current | PlanDefinition/ActivityDefinition; targeting FHIR R6 normative | https://hl7.org/fhir/clinicalreasoning-module.html |
| BPM+ Health / HL7 BPM Community of Practice | 2024–2025 | OMG-to-HL7 transition; generic BPMN/DMN/CMMN reuse strategy | https://www.bpm-plus.org/ |
| Da Vinci Prior Authorization Support IG | current | CRD/DTR/PAS architecture; X12-membership friction noted | https://hl7.org/fhir/us/davinci-pas/ |
| CAQH 2023 Index Report | 2024 | Only 35% of PA transactions fully electronic (up from 28%) | https://caqh.org/hubfs/43908627/drupal/2024-01/2023_CAQH_Index_Report.pdf |
| Override rate of DDI alerts, systematic review/meta-analysis | 2024 | ~90% DDI override rate; 46.2–96.2% range across 23 studies | https://journals.sagepub.com/doi/10.1177/14604582241263242 |
| Kawamoto, Houlihan, Balas, Lobach, BMJ | 2005 | 70-RCT review; 4 features predict CDS success | https://pubmed.ncbi.nlm.nih.gov/15767266/ |
| Wright et al., CDS alert malfunction taxonomy, JAMIA | 2018 | 68 malfunctions, 14 sites, persist for long periods | https://dx.doi.org/10.1093/jamia/ocx106 |
| "CDS malfunctions in a commercial EHR" (OHSU/Epic) | 2018 | 4 malfunctions, all silent/undetected, all from unrelated system changes | https://pmc.ncbi.nlm.nih.gov/articles/PMC6220702/ |
| "Verification of Medical Guidelines by Model Checking" (Asbru) | ~2005 | Model-checking verification research line; stayed small-scale/academic | https://spinroot.com/spin/symposia/ws06/027.pdf |
| MobiGuide evaluation | 2017 | Small-scale patient trial (10 AF + 19 GDM patients) | https://link.springer.com/article/10.1007/s11257-017-9190-5 |
| CDS Hooks implementer list | — | Vendor support breadth | https://www.trisotech.com/cds-hooks/ |

### Q2 — Scenario landscape beyond triage

| Source | Date | Takeaway | URL |
|---|---|---|---|
| Wong et al., JAMA Internal Medicine | 2021 | Epic Sepsis Model: AUC 0.63, sensitivity 33%, 109 alerts/true case | https://jamanetwork.com/journals/jamainternalmedicine/fullarticle/2781307 |
| STAT News, Epic sepsis coverage | 2021-06-21 | "Little better than a coin flip"; no prior external validation | https://www.statnews.com/2021/06/21/epic-sepsis-prediction-tool/ |
| Habib et al. editorial, JAMA IM | 2021 | "The Epic Sepsis Model Falls Short — The Importance of External Validation" | https://pubmed.ncbi.nlm.nih.gov/34152360/ |
| Sepsis alert systematic review/meta-analysis | 2024 | Alert systems associated with lower mortality/better SSC-bundle adherence, alert-fatigue caveat | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11265133/ |
| Adams et al., Nature Medicine (TREWS) | 2022 | 590,736 patients, 5 hospitals; confirmed alerts → ~18% relative mortality reduction | https://www.nature.com/articles/s41591-022-01894-0 |
| NEWS2 prognostic accuracy study | — | AUROC ~0.67–0.69 for mortality, poor for cardiac arrest | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8718668/ |
| NEWS/NEWS2 systematic review | 2026 | Outcome improvement inconsistent; attributed to escalation non-adherence | https://link.springer.com/article/10.1186/s13643-026-03088-y |
| Kucher et al., NEJM | 2005 | RCT (1,255 vs. 1,251): computer alert reduced DVT/PE rates | https://www.nejm.org/doi/full/10.1056/NEJMoa041533 |
| AHRQ MATCH Toolkit | — | Medication-reconciliation-at-transitions toolkit | https://www.ahrq.gov/patient-safety/settings/hospital/match/index.html |
| CMS Interoperability and Prior Authorization Final Rule | 2024-01-17 | CMS-0057-F requires FHIR PA APIs (Da Vinci CRD/DTR/PAS) | https://www.cms.gov/newsroom/fact-sheets/cms-interoperability-and-prior-authorization-final-rule-cms-0057-f |
| CBS News, UnitedHealth/naviHealth lawsuit coverage | 2023–2024 | Lawsuit alleges nH Predict "known 90% error rate" | https://www.cbsnews.com/news/unitedhealth-lawsuit-ai-deny-claims-medicare-advantage-health-insurance-denials/ |
| ProPublica, Cigna PXDX investigation | 2023-03-25 | 300,000 claims denied in 2 months, 1.2 sec/claim average | https://www.propublica.org/article/cigna-pxdx-medical-health-insurance-rejection-claims |
| STAT News, Medicare Advantage AI probe | 2026-07-15 | Senate inquiry into UnitedHealth/Humana/CVS AI-driven denials | https://www.statnews.com/2026/07/15/medicare-advantage-ai-care-denials-probe-blumenthal-hawley/ |
| STAT News, OIG report coverage | 2026-06-11 | HHS OIG: post-hospital care denial rates 51–80% at 3 major insurers | https://www.statnews.com/2026/06/11/medicare-advantage-oig-report-rehab-care-deny-appeal-reverse/ |
| Clinical Infectious Diseases (WISCA antimicrobial CDS trial) | 2021 | RCT, 6,849 patients; no overall benefit except cellulitis LOS | https://academic.oup.com/cid/article/72/9/e265/5876588 |
| Antibiotic CDSS meta-analysis (30 trials) | — | Median 3% improvement; questions clinical significance | https://www.researchgate.net/publication/331401165 |
| JAMIA chronic-disease CDS meta-analysis | 2022 | 76 studies; small/heterogeneous effects, no reliable design predictor | https://academic.oup.com/jamia/article/29/10/1757/6639847 |
| STRATIFY ED disposition tool | 2025 | 13-factor low-risk HF-discharge score | https://link.springer.com/article/10.1186/s43058-025-00779-w |
| CMS AUC Program pause, ACC | 2023-12-14 | Program paused for claims-infrastructure reasons, not clinical ones | https://www.acc.org/Latest-in-Cardiology/Articles/2023/12/14/15/16/CMS-Pauses-AUC-Program-For-Advanced-diagnostic-Imaging-Rescinds-Current-Regulations |
| iCareWean, AJRCCM | 2006 | Closed-loop weaning cut wean time 4 days → 2 days | https://www.atsjournals.org/doi/10.1164/rccm.200511-1780OC |
| Alert-override systematic review | 2020 | DDI override prevalence ~90% (95% CI 85–95%) | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7400042/ |

### Q3 — Neuro-symbolic / ontology-grounded clinical AI

| Source | Date | Takeaway | URL |
|---|---|---|---|
| Li, Huang, Naik, "Scallop: A Language for Neurosymbolic Programming" | 2023 | Datalog + differentiable provenance semirings; formal but built for gradient training | https://arxiv.org/abs/2304.04812 |
| Riegel et al., "Logical Neural Networks" | 2020 | Weighted real-valued logic, differentiable, bound-propagation | https://arxiv.org/abs/2006.13155 |
| IBM/LNN GitHub | ongoing | Reference implementation, mature/stable | https://github.com/IBM/LNN |
| Manhaeve et al., "DeepProbLog" | 2018 | Neural predicates + ProbLog, exact/approx weighted model counting | https://arxiv.org/abs/1805.10872 |
| Marra, Dumančić, Manhaeve, De Raedt, survey, Artificial Intelligence 328 | 2024 | Unifying statistical-relational + neurosymbolic AI survey | https://www.sciencedirect.com/science/article/pii/S0004370223002084 |
| Wikipedia, Neuro-symbolic AI | current | Kautz Type 1–6 taxonomy; **no mention of MYCIN/Arden/expert systems** | https://en.wikipedia.org/wiki/Neuro-symbolic_AI |
| Wikipedia, Arden Syntax | current | Still-active HL7 standard, v3.0 Oct 2023 | https://en.wikipedia.org/wiki/Arden_syntax |
| Wikipedia, MYCIN | current | Backward chaining, certainty factors, trace-as-explanation | https://en.wikipedia.org/wiki/Mycin |
| Trinh, Wu, Le, He, Luong, "AlphaGeometry," Nature | 2024 | LLM proposes, symbolic deduction engine verifies — genuine hard gate | https://www.nature.com/articles/s41586-023-06747-5 |
| Pan et al., "Logic-LM" | 2023 | LLM→FOL, Z3/Prover9 decides — hard gate | https://arxiv.org/abs/2305.12295 |
| Ye et al., "SATLM" | 2023 | NL→SAT, Z3 solves — hard gate | https://proceedings.neurips.cc/paper_files/paper/2023/file/8e9c7d4a48bdac81a58f983a64aaf42b-Paper-Conference.pdf |
| Loaiza-Bonilla et al., oncology trial-matching neurosymbolic system | 2026 | Graph-enforced consistency checks; F1 0.82 vs. 0.47/0.67 ungated GPT-4 | https://pmc.ncbi.nlm.nih.gov/articles/PMC13091143/ |
| Gao et al., DR.KNOWS, JMIR AI | 2025 | UMLS graph-path RAG for diagnosis — retrieval, not gating | https://ai.jmir.org/2025/1/e58670 |
| Chang & Sung, SNOMED CT in LLMs scoping review | 2024 | Field lacks standardized KG-integration methods | https://medinform.jmir.org/2024/1/e62924 |
| JAMIA RAG-in-biomedicine meta-analysis | 2025 | Pooled OR 1.35 favoring RAG; hallucination reduction not consistently quantified | https://academic.oup.com/jamia/article/32/4/605/7954485 |
| Singhal et al., Med-PaLM 2, Nature Medicine | 2025 | 86.5% MedQA; pure scaling, no symbolic component | https://www.nature.com/articles/s41591-024-03423-7 |
| Adversarial hallucination study, Communications Medicine | 2025 | 6 LLMs repeated planted clinical errors in 50–83% of cases | https://www.nature.com/articles/s43856-025-01021-3 |
| "Representation Before Retrieval," medRxiv | 2026 | RAG raised unsupported claims 5.0%→43.6% in one setting | https://www.medrxiv.org/content/10.64898/2026.02.13.26346256v1.full |
| Abridge product page | 2026 | Transcript-linked notes; human-verified, no symbolic layer claimed | https://www.abridge.com/ai |
| Epic AI agents launch, Healthcare IT News | 2025 | Physician sign-off is the safety gate for Epic's Art/Emmie/Penny | https://www.healthcareitnews.com/news/epic-unveils-ai-agents-showcases-new-foundational-models |
| OpenAI Agents SDK, Guardrails docs | current | Guardrails confirmed soft/post-hoc; tools may fire before a tripwire cancels | https://openai.github.io/openai-agents-python/guardrails/ |
| Google ADK, Workflow Agents docs | current | Genuine hard gate, but only over fixed orchestration, not LLM node content | https://adk.dev/agents/workflow-agents/ |
| Apache KIE (Drools) homepage | current | Plain rules engine, no LLM/agent/formal-verification content | https://kie.apache.org/ |
| AUI / Apollo-1 | GA claimed Q2 2026 | Markets "first neuro-symbolic model"; no published technical proof found | https://www.aui.io |
| Lean4Agent | 2026 | Lean4 verifies agent workflow structure pre-execution; ~12% SWE-Bench gain | https://arxiv.org/abs/2606.06523 |
| VeriPlan, CHI 2025 | 2025 | LLM planner → formal property translation → model checker | https://doi.org/10.1145/3706598.3714113 |
| Hekmatnejad, Simms, Fainekos, "Model Checking CDS Using SMT" | 2019 | Classical CDS formal-verification antecedent, no LLM connection | https://arxiv.org/abs/1901.04545 |
| Sigloch & Benzmüller, "Neuro-Symbolic Verification of LLM Outputs" | 2026 | Formal+embedding hybrid gate; 83%/72% hallucination detection (medical-device domain) | https://arxiv.org/abs/2605.26942 |

### Q4 — Clinical AI validation practice

| Source | Date | Takeaway | URL |
|---|---|---|---|
| Wong et al., JAMA Internal Medicine | 2021 | Epic Sepsis Model independent validation: AUC 0.63, PPV 12% | https://jamanetwork.com/journals/jamainternalmedicine/fullarticle/2781307 |
| "Evaluation of Sepsis Prediction Models before Onset of Treatment," NEJM AI | 2024 | Label leakage: AUROC fell to 0.47 excluding post-suspicion data | https://ai.nejm.org/doi/full/10.1056/AIoa2300032 |
| "Multicenter Prospective Validation of an Updated Proprietary Sepsis Prediction Model," JAMA Network Open | 2026 | Epic Sepsis Model v2 (silent Dec 2023): AUROC 0.82–0.92, PPV still 0.13–0.26 | https://jamanetwork.com/journals/jamanetworkopen/fullarticle/2845595 |
| Adams et al., Nature Medicine (TREWS) | 2022 | Prospective outcome-level sepsis-alert evaluation | https://www.nature.com/articles/s41591-022-01894-0 |
| Sendak et al., "Sepsis Watch integration," JMIR Med Inform | 2020 | Duke's implementation study; live since Nov 2018 | https://medinform.jmir.org/2020/7/e15182/ |
| "Sepsis Watch multisite external validation," npj Digital Medicine | 2025 | Transferred to 4 new EDs, no retraining, per-site threshold tuning needed | https://pmc.ncbi.nlm.nih.gov/articles/PMC12159134/ |
| STAT News, "IBM's Watson supercomputer recommended 'unsafe and incorrect' cancer treatments" | 2018-07-25 | Trained on synthetic cases + a few specialists' preferences, not guidelines | https://www.statnews.com/2018/07/25/ibm-watson-recommended-unsafe-incorrect-treatments/ |
| STAT News, "IBM pitched Watson as a revolution... nowhere close" | 2017-09-05 | Earlier investigation into Watson for Oncology underperformance | https://www.statnews.com/2017/09/05/watson-ibm-cancer/ |
| Royal Free London, Streams audit | n.d. | NHS trust's own DeepMind Streams evaluation disclosure | https://www.royalfree.nhs.uk/news/royal-free-london-publishes-audit-streams-app |
| Digital Health News, Streams cost-savings peer review | 2019-07 | ~£2,000/patient cost savings, reduced AKI admission costs | https://www.digitalhealth.net/2019/07/deepminds-streams-saves-2000-peer-review/ |
| "The silent trial" methodology paper | 2022 | Hydronephrosis case: retrospective AUROC 0.90 → live shadow AUROC 0.50 (chance) | https://pmc.ncbi.nlm.nih.gov/articles/PMC9424628/ |
| DDX-BRO trial, Lancet Digital Health | 2024/2025 | Well-powered cluster RCT, null result on diagnostic-quality-risk outcome | https://www.thelancet.com/journals/landig/article/PIIS2589-7500(24)00250-4/fulltext |
| eRegQual trial, Lancet Digital Health | 2021 | Positive CDS RCT counter-example (structured antenatal care) | https://www.thelancet.com/journals/landig/article/PIIS2589-7500(21)00269-7/fulltext |
| "Effect of Clinical Decision-Support Systems," Annals of Internal Medicine | 2012 | Process-of-care gains, little/no mortality effect | https://www.acpjournals.org/doi/10.7326/0003-4819-157-1-201207030-00450 |
| "1,357 AI medical devices cleared, 3 tested on patient outcomes," PLOS Digital Health | 2025 | Only 0.2% of FDA-cleared AI devices ever evaluated against a patient-centered outcome | https://journals.plos.org/digitalhealth/article?id=10.1371%2Fjournal.pdig.0001597 |
| FDA, AI/ML-Enabled Medical Devices list | ongoing | Public registry of cleared AI/ML devices | https://www.fda.gov/medical-devices/software-medical-device-samd/artificial-intelligence-and-machine-learning-aiml-enabled-medical-devices |
| FDA, PCCP final guidance | 2024-12-03 | Pre-authorized bounded future AI-device modification envelopes | https://www.fda.gov/regulatory-information/search-fda-guidance-documents/marketing-submission-recommendations-predetermined-change-control-plan-artificial-intelligence |
| FDA/Health Canada/MHRA, GMLP guiding principles | 2021-10 | 10 joint guiding principles for AI/ML medical device development | https://www.fda.gov/media/153486/download |
| Federal Register, CDS software guidance | 2022-09-28 | Final interpretation of the 21st Century Cures Act's non-device CDS exemption | https://www.federalregister.gov/documents/2022/09/28/2022-20993/clinical-decision-support-software-guidance-for-industry-and-food-and-drug-administration-staff |
| "27 Years of AI/ML Medical Device Recalls" | n.d. | ~5–6% of AI/ML devices ever recalled, mostly software bugs | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC12274014/ |
| [unverified — preprint] "Calibration Drift Under Cross-Institutional Deployment," medRxiv | 2026 | Internal calibration fine, external calibration can silently fail; AUROC blind to it | https://www.medrxiv.org/content/10.64898/2026.05.03.26352335v1.full |

### Q5 — Terminology licensing and subsumption at scale

| Source | Date | Takeaway | URL |
|---|---|---|---|
| SNOMED International, "Get SNOMED CT" | accessed 2026 | Free for Member countries; fees by territory outside membership | https://www.snomed.org/get-snomed |
| SNOMED International, vendor licensing guide | accessed 2026 | Distributing SNOMED-enabled products requires Affiliate status + sub-licensing | https://docs.snomed.org/snomed-ct-practical-guides/vendor-introduction-to-snomed-ct/7-licensing |
| NLM, SNOMED CT FAQs | accessed 2026 | NLM distributes SNOMED CT free to US/international users via UMLS license | https://www.nlm.nih.gov/healthit/snomedct/faq.html |
| SNOMED International, "Do I need an external interface terminology?" | accessed 2026 | ~368,000 active concepts; explicitly warns against hand-rolled custom subsets | https://docs.snomed.org/implementation-guides/implementation-fact-sheets/implementation-dilemmas/do-i-need-an-external-interface-terminology-to-implement-snomed-ct |
| SNOMED International, Concept Inactivation | accessed 2026 | ~25% of concept space inactive at any time | https://confluence.ihtsdotools.org/display/DOCEG/Concept+Inactivation |
| SNOMED International, ECL introduction | accessed 2026 | Formal language for intensional subset definitions | https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/introduction/1-introduction |
| SNOMED International, ICD-10 Map Overview | accessed 2026 | SNOMED→ICD-10 maps documented as one-directional and lossy | https://docs.snomed.org/snomed-ct-specifications/snomed-ct-to-icd-10-map-specification/introduction/2-icd-10-map-overview |
| GitHub, IHTSDO/snowstorm | accessed 2026 | SNOMED International's own open-source terminology server | https://github.com/IHTSDO/snowstorm |
| CSIRO, Ontoserver FAQs | accessed 2026 | Decouples implementers from raw RF2; enables frequent version updates | https://ontoserver-wp.csiro.au/faqs/ |
| Metke-Jimenez et al., "Ontoserver: a syndicated terminology server" | — | Peer-reviewed description of CSIRO's FHIR terminology server | https://pubmed.ncbi.nlm.nih.gov/30223897/ |
| [unverified — secondary summary only] Dentler et al., "Comparison of reasoners for large ontologies in OWL 2 EL" | — | ELK classifies full SNOMED CT in ~5s; CEL ~20 min | https://www.semantic-web-journal.net/sites/default/files/swj120_2.pdf |
| Chu et al., JMIR Medical Informatics | 2019 | Extensional value sets capture median 65% (as low as 2%) of terms an intensional definition would | https://pmc.ncbi.nlm.nih.gov/articles/PMC6351992/ |
| Daniel Vreeman, "Regenstrief's approach to licensing LOINC" | — | Free for commercial/non-commercial use; cannot fork/modify core content | https://danielvreeman.com/blog/2016/11/24/regenstriefs-approach-to-licensing-loinc/ |
| NLM, RxNorm Terms of Service | accessed 2026 | Core RXCUI/names are US public domain; full releases need free UMLS license | https://www.nlm.nih.gov/research/umls/rxnorm/docs/termsofservice.html |
| NLM, RxNav Data Sources | accessed 2026 | RxTerms/MED-RT ecosystem relationships | https://lhncbc.nlm.nih.gov/RxNav/information/DataSources.html |
| Repo file: [`examples/triage/snomed-ct-subset.ttl`](https://github.com/billaram/horismos/blob/a164db4/examples/triage/snomed-ct-subset.ttl) | current HEAD | Shipped fixture: ~50 concepts, single-parent only, no polyhierarchy | local repo |
| Repo file: [`docs/framework/02-ontology.md`](https://github.com/billaram/horismos/blob/a164db4/docs/framework/02-ontology.md) (~line 132) | current HEAD | Resolved decision D5/PRP-001: vocabularies consumed as pre-extracted subset imports | local repo |
| Repo file: [`docs/framework/13-authoring.md`](https://github.com/billaram/horismos/blob/a164db4/docs/framework/13-authoring.md) | current HEAD | Planned (unbuilt) `search_vocab` SNOMED-anchoring tool, PRP-007 scope | local repo |

### Q6 — Agent-authored and learnable ontologies

| Source | Date | Takeaway | URL |
|---|---|---|---|
| OntoGPT, Monarch Initiative | ongoing | Most mature tool found; structured extraction against an existing vocabulary, not from-scratch axiom authoring | https://github.com/monarch-initiative/ontogpt |
| open-ontologies (Tesseract Academy) | 2026 | Closest external architectural analog: deterministic Rust OWL2-DL+SHACL engine gates LLM tool calls; self-reports 9th/13 on an OAEI benchmark | https://github.com/fabio-rovai/open-ontologies |
| LLMs4OL 2024 Overview | 2024 | Term typing F1 0.9938 vs. relation extraction F1 0.0783 | https://arxiv.org/html/2409.10146v1 |
| "Ontology Learning with LLMs: Axiom Identification Benchmark" (OntoAxiom) | 2025/26 | F1 0.218–0.642 by domain; "not high enough to fully automate axiom identification" | https://arxiv.org/abs/2512.05594 |
| Li, Garijo & Poveda-Villalón, SLR, Semantic Web Journal | 2026 | 15,688 papers screened to 36; no standardized benchmarks, poor reproducibility field-wide | https://www.semantic-web-journal.net/content/large-language-models-ontology-engineering-systematic-literature-review |
| Manda, "LLMs in Bio-Ontology Research: A Review" | 2025 | Maturity tiers; documents "workload shift rather than reduction" as a real failure mode | https://pmc.ncbi.nlm.nih.gov/articles/PMC12649945/ |
| Rice trait ontology curation study | 2025 | LLM-as-fallback + mandatory expert sign-off; kappa 0.719 — closest structural sibling to PRP-007 found | https://pmc.ncbi.nlm.nih.gov/articles/PMC12670841/ |
| AutoSchemaKG | 2025 (ACL 2026) | Fully autonomous web-scale schema induction; no dedup mechanism, admits residual inconsistency risk | https://arxiv.org/abs/2505.23628 |
| KARMA | 2025 | 9-agent autonomous KG enrichment; confidence-gated, not systematic human review | https://arxiv.org/abs/2502.06472 |
| Wikipedia, Never-Ending Language Learning (NELL) | accessed 2026 | Cookies→baked-goods→computer-files drift chain; 3% belief confidence | https://en.wikipedia.org/wiki/Never-Ending_Language_Learning |
| Wikipedia, Wikidata (governance) | accessed 2026 | Schema changes require mandatory human community discussion | https://en.wikipedia.org/wiki/Wikidata |
| "Neuro-Symbolic Integration + Ontological Reasoning" (HermiT-in-loop paper) | 2025 | Genuine reasoner-in-loop precedent — **withdrawn by its own authors** for inaccuracies | https://arxiv.org/abs/2504.07640 |
| GLaMoR | 2025 | Replaces symbolic OWL reasoner with a learned approximator — counter to horismos's anti-wrapper stance | https://arxiv.org/abs/2504.19023 |
| "Is Three the Magic Number?" repair-loop study | 2026 | Diminishing returns after 3–4 repair iterations | https://arxiv.org/abs/2607.05197 |
| LASA | 2025 | Names "vacuous assertions" as a known LLM-verification failure mode | https://arxiv.org/abs/2506.17865 |
| CHARGE | 2026 | Reports non-vacuity rate as a headline verification metric | https://arxiv.org/abs/2607.27776 |
| "From Natural Language to Verified Code" | 2026 | Names and designs against "vacuous verification" explicitly | https://arxiv.org/abs/2604.22601 |
| Lahiri, "Evaluating LLM-driven User-Intent Formalization" | 2024 | No algorithmic guarantee a formal spec captures user intent | https://arxiv.org/abs/2406.09757 |
| DafnyCOMP | 2025 | Per-function verification success drops sharply under composition | https://arxiv.org/abs/2509.23061 |
| VeriContest | 2026 | 92% plain-coding success vs. 5.29% end-to-end on formally-verified variants | https://arxiv.org/abs/2605.08553 |
| ReForm | 2025/26 | Names the "formalization gap" between provable and faithful | https://arxiv.org/abs/2510.24592 |
| Turpin et al., "LMs Don't Always Say What They Think" | 2023 | CoT explanations can systematically misrepresent the true reason for a prediction | https://arxiv.org/abs/2305.04388 |
| Lanham et al., "Measuring Faithfulness in Chain-of-Thought Reasoning" | 2023 | Larger, more capable models produce *less* faithful reasoning | https://arxiv.org/abs/2307.13702 |
| xpSHACL | 2025 | Renders SHACL violations in NL — precedent for horismos's witness-rendering approach | https://www.vldb.org/2025/Workshops/VLDB-Workshops-2025/LLM+Graph/LLMGraph-6.pdf |
| ARPaCCino | 2025 | LLM drafts Rego/OPA policy, iteratively refined against IaC compliance checks | https://arxiv.org/abs/2507.10584 |

---

## 5. What this changes, concretely

Nothing here rises to "reproduced a bug with an obvious fix" — this is a research sweep, and
per the brief's scope it produces knowledge, not a PR. The items below are what I'd want
tracked as follow-up if the captain wants to act on this:

- **Positioning**: consider naming both lineages explicitly (expert-system-style symbolic
  core + state-of-the-art neural-boundary engineering) rather than an unqualified
  "neurosymbolic AI" claim — see §1.4.
- **Scenario choice**: VTE prophylaxis, NEWS2 deterioration, and reframed sepsis-bundle
  compliance are the recommended next three, in that order — see §2.
- **PRP-007 (unbuilt, Draft)**: add an explicit non-vacuity verification pass, and extend the
  planned DoD's seeded-incompleteness test to also cover seeded subtly-wrong-but-consistent
  axioms — see §1.7. Both are additions to an already-Draft PRP, not a change to shipped code.
- **SNOMED fixture**: [`examples/triage/snomed-ct-subset.ttl`](https://github.com/billaram/horismos/blob/a164db4/examples/triage/snomed-ct-subset.ttl)'s single-parent structure and
  missing version provenance are visible today, not hypothetical — worth a note in
  [`docs/framework/02-ontology.md`](https://github.com/billaram/horismos/blob/a164db4/docs/framework/02-ontology.md) and consideration when `search_vocab`/PRP-007's output
  format is designed — see §1.5.
- **`research/`**: the directory structure and intake convention in §3 is ready to stand up
  as an actual first PR whenever the captain wants it — I did not create it in the repo,
  per the brief's "no implementation" scope, but the layout above is complete enough to
  create directly.

No unresolved decision requiring the captain's binary choice was generated by this sweep —
every finding above is a recommendation the captain can act on, defer, or decline at their
own pace, not a blocking fork.
