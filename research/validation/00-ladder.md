# Proposed validation ladder

## Claim

Given where horismos actually is (pre-alpha, one hand-built scenario, deterministic
replay, and formal rule verification as core, already-built capabilities), a credible
5-rung validation ladder runs from adversarial synthetic testing through a scoped
prospective/RCT comparison. This is a living/revised document as rungs are actually
attempted.

## Evidence

**[measured]** Formal verification does not prove the medicine is right. Horismos's
own verification documentation (`docs/framework/10-verification.md`) already states
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

## Source

See `research/reading-log.md`, Q4 section.

## Confidence

mixed — case evidence per rung is [measured]; the ladder structure itself and its
"what it buys / doesn't buy" framing are [inference].

## Last-verified

2026-08-24
