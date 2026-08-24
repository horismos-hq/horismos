# Scenario: sepsis bundle (SSC) compliance-tracking — explicitly not prediction

## Claim

**#3 recommended next scenario.** The rhetorically and technically sharpest pick for
directly rebutting "one hand-authored scenario," because sepsis is simultaneously the
field's highest-profile CDS *failure* (Epic Sepsis Model) and has a credible positive
counter-example (TREWS). Scope must be strictly bundle timing/checklist compliance —
never sepsis prediction.

## Evidence

**[measured]** The Epic Sepsis Model was deployed to hundreds of hospitals with no
published external validation before that scale; independent retrospective validation
at Michigan Medicine found AUC 0.63, sensitivity 33%, PPV 12% — clinicians needed to
evaluate 109 alerts to find one true, previously-untreated case
([Wong et al., JAMA Internal Medicine 2021](https://jamanetwork.com/journals/jamainternalmedicine/fullarticle/2781307)).
A 2024 NEJM AI reanalysis found much of its apparent accuracy was label leakage —
AUROC collapsed to 0.47 (coin-flip) once post-suspicion data were excluded
([ai.nejm.org](https://ai.nejm.org/doi/full/10.1056/AIoa2300032)). See
`../validation/case-studies.md` for the full case history including the 2026 v2 update.

**[measured]** TREWS (Adams et al., Nature Medicine 2022), a prospective outcome-level
sepsis-alert evaluation across 590,736 patients and 5 hospitals, found confirmed
alerts associated with ~18% relative mortality reduction
([nature.com](https://www.nature.com/articles/s41591-022-01894-0)).

**[measured]** A 2024 sepsis-alert systematic review/meta-analysis finds alert systems
generally associated with lower mortality and better SSC-bundle adherence, with an
alert-fatigue caveat
([PMC11265133](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11265133/)).

**[inference]** Horismos's mandatory pre-deployment verification and proof trees are
architecturally the opposite of what went wrong with the Epic model (deployed at scale
with no external validation, and later found to leak the label). Scope the build to
SSC bundle timing/checklist compliance — lactate drawn, cultures before antibiotics,
antibiotics within window, reassessment documented — never to sepsis prediction, to
stay on the positive-evidence side of this history rather than repeating it.

## Source

- Wong et al., JAMA Internal Medicine (2021) —
  https://jamanetwork.com/journals/jamainternalmedicine/fullarticle/2781307
- "Evaluation of Sepsis Prediction Models before Onset of Treatment," NEJM AI (2024) —
  https://ai.nejm.org/doi/full/10.1056/AIoa2300032
- Adams et al., Nature Medicine (TREWS) (2022) —
  https://www.nature.com/articles/s41591-022-01894-0
- Sepsis alert systematic review/meta-analysis (2024) —
  https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11265133/

## Confidence

mixed — case outcomes are [measured]; the scoping recommendation is [inference].

## Last-verified

2026-08-24
