# Proposed validation ladder

## Claim

Given where horismos actually is (pre-alpha, one hand-built scenario, deterministic
replay, and formal rule verification as core, already-built capabilities), a credible
5-rung validation ladder runs from adversarial synthetic testing through a scoped
prospective/RCT comparison. This is a living/revised document as rungs are actually
attempted.

## Evidence

**[measured]** Formal verification does not prove the medicine is right. Horismos's
own verification documentation ([`docs/framework/10-verification.md`](https://github.com/billaram/horismos/blob/main/docs/framework/10-verification.md)) already states
the limit: verification proves the reasoner is total and sound *around whatever
perception does*, and claims nothing about tool implementations or anything
probabilistic. **[inference]** The clinically load-bearing corollary: verification
proves the authored rule set is complete, conflict-free, and terminating relative to
itself — it does not and cannot prove the rules encode correct medicine. A perfectly
verified ontology can exhaustively, consistently, and provably encode a wrong
chest-pain risk-stratification threshold.

**[measured — the proposed ladder]**

| Rung | Activity | What it buys | What it explicitly does not buy |
|---|---|---|---|
| 0 | Adversarial synthetic case suites + property-based testing, using the BDD verifier's own completeness witnesses as a test-case generator | Regression testing via byte-identical trace equality — cheap, hard pass/fail, no statistical noise | Any evidence the rules are clinically correct |
| 1 | Expert clinician red-team of the ontology's rules + verification witnesses (2–3 clinicians, different institutions) | Directly targets the IBM-Watson-style narrow/idiosyncratic-encoding failure; proof trees and domain-text witnesses are far cheaper for a clinician to review than a black-box model's SHAP values — this operationalizes the FDA's CDS-exemption language ("independently review the basis") as a design property | Nothing about statistical generalization |
| 2 | Retrospective replay against a public de-identified dataset slice matching the scenario (e.g. MIMIC-IV's ED module, not eICU-CRD which is ICU-only) — explicitly re-run the exact label-leakage check that broke the Epic Sepsis Model | Internal-validation confidence; a concrete, checkable leakage test | Calibration/generalization across sites — internal-only validation systematically overstates real-world reliability per the cross-institutional MIMIC-IV↔eICU literature |
| 3 | Silent-mode/shadow run alongside a real (non-critical or non-acting) workflow, clinicians blinded, full trace capture | Determinism gives a real edge here that black-box shadow evaluation lacks: no random-seed/sampling confound, so any discrepancy is fully attributable to an exact rule via exact replay | Alert-fatigue economics and label leakage in real data — properties of deployment, not of symbolic-vs-statistical reasoning |
| 4 | Prospective/RCT, scoped honestly as a process-outcome comparison first (e.g. time-to-review), not a mortality trial | Only 0.2% of even FDA-cleared AI devices ever reach this rung ([PLOS Digital Health 2025 evidence census](https://journals.plos.org/digitalhealth/article?id=10.1371%2Fjournal.pdig.0001597)); a null RCT result (e.g. the well-powered DDX-BRO cluster RCT, [Lancet Digital Health](https://www.thelancet.com/journals/landig/article/PIIS2589-7500(24)00250-4/fulltext)) is a real, common, non-failure outcome to plan for | — |

**[measured]** The "silent trial" methodology paper documents a real hazard at rung 3:
one hydronephrosis case's retrospective AUROC of 0.90 fell to a live shadow AUROC of
0.50 (chance) — the rung must actually be attempted, not skipped as redundant with
retrospective validation
([PMC9424628](https://pmc.ncbi.nlm.nih.gov/articles/PMC9424628/)).

**[measured]** On rung 4 outcomes generally: eRegQual (Lancet Digital Health, 2021) is
a positive CDS RCT counter-example (structured antenatal care); a 2012 Annals of
Internal Medicine review found CDS systems broadly produce process-of-care gains with
little/no mortality effect
([acpjournals.org](https://www.acpjournals.org/doi/10.7326/0003-4819-157-1-201207030-00450)).
A 2026 medRxiv preprint on calibration drift under cross-institutional deployment is
flagged `[unverified — preprint]`: internal calibration can look fine while external
calibration silently fails, and AUROC alone is blind to it
([medrxiv.org](https://www.medrxiv.org/content/10.64898/2026.05.03.26352335v1.full)).

## Jev deep dive delta (2026-09-21): the ladder measures the rules, not the threshold

**[inference]** The five rungs above all ask variants of one question — *are the encoded
rules right?* None of them asks the other question a Horismos deployment depends on:
*is the acceptance threshold at the perception boundary right?* That boundary is
`accepted = result.confidence >= horis:confidence`
(`crates/horismos-core/src/workflow/agent.rs`:492, **[measured — local repo]**), and the
number it compares against is, today, self-reported by the model and never measured
(`docs/framework/08-bridge.md`: "the forced tool call's schema carries a `confidence`
number field the model must self-report"). A deployment could pass every rung on this
ladder and still route the wrong third of its cases to a human — or fail to route the
right ones.

**[measured]** The literature the Jev deep dive surveyed gives this a name and a set of
instruments. The mechanism is Chow's reject option (C. K. Chow, *IEEE Trans. Inf. Theory*
IT-16(1):41–46, 1970) and the modern selective-prediction line (Geifman & El-Yaniv,
NeurIPS 2017; Mozannar & Sontag, ICML 2020) **[unverified — confirmed
bibliographically, primary PDFs not opened]**. The measurement is a reliability diagram
plus an expected calibration error. The *guarantee* is distribution-free: split conformal
prediction and conformal risk control (Angelopoulos & Bates,
[arXiv:2107.07511](https://arxiv.org/abs/2107.07511);
[arXiv:2208.02814](https://arxiv.org/abs/2208.02814)).

**[measured]** Three 2026 results bear directly on this page's existing rung-2 caution
that "internal-only validation systematically overstates real-world reliability":

- Kotte, "When Can Conformal Risk Control Certify LLM Outputs?"
  ([arXiv:2606.29054](https://arxiv.org/abs/2606.29054), Jun 2026), covering
  schema-validated structured generation and classification: heuristic abstention
  policies "violate user-specified risk targets on **7.5–12.5% of settings**, with no
  per-deployment guarantee"; and "under cross-dataset shift the target is violated on
  **14 of 16 transfers** by static CRC and every tested adaptive-conformal-inference step
  size."
- ConfidenceBench ([arXiv:2607.20526](https://arxiv.org/abs/2607.20526), Jul 2026):
  across 15 frontier models, "accuracy and calibration diverge substantially across model
  families: the most accurate model is not the best-calibrated," with several models
  below a calibrated-random baseline.
- Kim & Kang, "Same Answer, Different Confidence"
  ([arXiv:2605.27752](https://arxiv.org/abs/2605.27752), May 2026): the measurement
  protocol "leave[s] the sign of the ECE comparison undetermined in 6 of 12 settings",
  and **5 of 12 audited studies never state which protocol they used**.

**[inference]** Three consequences for this ladder, none of which changes a rung:

1. **Rung 0 gains an instrument, not a rung.** The adversarial synthetic suites already
   produce labelled perception outcomes; computing a reliability diagram and an ECE over
   them is nearly free, and it is the cheapest possible check that `horis:confidence` is
   doing anything at all.
2. **Rung 2's calibration caution now has a dated, quantified citation** rather than a
   single `[unverified]` preprint — and it generalizes: the drift being measured is not
   only in the clinical model but in the *proposer's* confidence.
3. **Any calibration number this project publishes must state its protocol.** Given that
   the sign of an ECE comparison is protocol-undetermined in half of settings, an
   unqualified ECE is not evidence. This is a reporting rule, and it should be adopted
   before the first measurement rather than after.

**[inference]** The deep dive's ranked move R2 proposes going further — choosing the
threshold by split-conformal calibration against a *declared target risk* and publishing
the implied abstention rate, so the number becomes a checkable statement rather than an
author's choice. That is an open decision (OD-3, owner PRP-005), not a ladder change, and
it is recorded here only because this is the page a reader would look at to ask "how
would we know?".

**[measured]** A second pass of the same dive, aimed at the regulatory and clinical-AI
record, returned three further results that bear on this ladder. They are recorded here
because each one is an instrument or a caution, and none of them is a rung.

- **Abstention is not a null action, and its cost is now measured in clinicians.** Jabbour
  et al., "Selective Prediction Reduces the Negative Effects of Automation Bias Overall but
  Increases False Negatives" ([arXiv:2508.07617](https://arxiv.org/abs/2508.07617), v2,
  2026-08-11), 259 clinicians: accuracy fell from 66% to 56% under inaccurate AI and
  recovered to 64% under selective prediction, but "when informed the AI abstains,
  clinicians **underdiagnose (18% increase in missed diagnoses) and undertreat (35% increase
  in missed treatments) compared to no AI input at all**." **[inference]** `horis:fallback`
  is a reject option, so this is a measured cost on Horismos's own failure path — and the
  cost is larger on treatment than on diagnosis. Two consequences for this ladder: **rung 1
  should red-team the fallback branch's message, not only the rules**, since what the
  clinician is told on abstention is now known to bias them toward inaction; and **rung 3's
  shadow run should record the abstention rate and its downstream disposition**, not only
  the accepted cases, because the harm this result measures is invisible if you only score
  what the system answered.
- **A conformal guarantee is marginal, not per-case or per-subgroup.** Mehrtens, Bucher &
  Brinker ([arXiv:2506.18162](https://arxiv.org/abs/2506.18162), 2025-06-22): "conformal
  predictions are unreliable under distributional shifts … [and are] **not reliable for
  subsets of the data**", because "**marginal coverage is a property over the distribution
  and does not give guarantees for singular prediction sets**" — with a worked example of a
  90% marginal guarantee delivering 92% coverage to one patient subgroup and **62%** to
  another. **[inference]** This is a direct rider on the R2/OD-3 proposal below: any
  published coverage number must be calibrated per deployment, reported by clinically
  meaningful subgroup, and stated together with its calibration distribution. Otherwise the
  ladder would be certifying a sentence that does not mean what a reader will take it to
  mean — the same failure mode as rung 2's internal-validation caution, one level up.
- **Calibration drifts while discrimination holds, so an AUROC-based monitor never fires.**
  Davis et al., *JAMIA* 2017;24(6):1052–61 followed seven acute-kidney-injury models over
  nine years and 2,003 VA admissions: "**[d]iscrimination was maintained for all models.
  Calibration declined as all models increasingly overpredicted risk.**" Their
  recommendation is that updating be "tailored … rather than be limited to regularly
  scheduled annual or biannual intervals." **[inference]** Whatever rung 0 measures under
  the reliability-diagram proposal above must be **re-measured on a trigger, not on a
  calendar** — and the trigger must be a calibration statistic, because the accuracy
  statistic will not move. The current-generation version of the same warning: the Epic
  Sepsis Model v2, evaluated prospectively across four health systems and 227,091
  encounters, scored AUROC **0.82 to 0.92 depending on the site**, with PPV **0.13 to 0.26**
  (Wong et al., *JAMA Network Open* 2026;9(2):e260181). A vendor's calibration is a property
  of model × site × period, not of the model.

**[inference]** And one addition that would be a genuine new instrument rather than a
refinement, recorded here because this is where it would live: the distinction the corpus
now rests on — that a derivation is a different object from a post-hoc explanation — has
almost no human-factors evidence behind it. The one positive signal is a single small
eye-tracking study (Han et al., *J Eye Mov Res* 2026;19(3):55) in which "verifiable
explanations", defined as pointers back to checkable primary evidence rather than
rationalisations, raised AI↔image gaze switches from 6.04 to 8.52 **on incorrect-AI trials
specifically**. Everything else in the literature measures the *other* artifact and finds
it does not help, including for intrinsically interpretable models (Gomez et al., *npj
Digital Medicine* 2025;8:706, which isolates intrinsic from post-hoc in 87 optometrists and
finds anchoring bias in both arms). A clinician study in which the artifact under test is a
derivation over declared facts is the Jev dive's ranked move **R9**, and it would sit on
**rung 1** — expert review of the rules extended to expert review of the *rendered proof*,
measured rather than assumed.

## Source

See [`../reading-log.md`](../reading-log.md), Q4 section, and the Q9 section added by the
Jev deep dive.

## Confidence

mixed — case evidence per rung is [measured]; the ladder structure itself and its
"what it buys / doesn't buy" framing are [inference]. The delta's 2026 citations are
[measured]; the selective-prediction lineage is [unverified] (bibliographic only); every
consequence drawn for the ladder is [inference].

## Last-verified

2026-09-21 (Jev deep dive delta appended; founding ladder unchanged, last checked 2026-08-24)
