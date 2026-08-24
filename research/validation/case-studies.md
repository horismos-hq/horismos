# Case studies: Epic Sepsis Model, TREWS, Sepsis Watch, Watson Oncology, DDX-BRO

## Claim

IBM Watson for Oncology is the load-bearing cautionary case for horismos's *current*
risk profile — a hand-authored-knowledge failure, not a generalization failure — more
than any statistical-ML case study. Independent external validation, when it happens,
tends to be unflattering relative to internal/vendor numbers.

## Evidence

### IBM Watson for Oncology

**[measured]** STAT News's 2018 reporting on internal IBM slide decks found Watson had
produced "unsafe and incorrect" treatment recommendations, traced to a root cause that
is not a generalization failure but a hand-authored-knowledge failure: the system was
trained on a small number of synthetic (hypothetical) cases rather than real patient
data, and encoded the personal treatment preferences of a handful of Memorial Sloan
Kettering specialists rather than guideline consensus
([STAT News, 2018-07-25](https://www.statnews.com/2018/07/25/ibm-watson-recommended-unsafe-incorrect-treatments/);
earlier investigation: [STAT News, 2017-09-05](https://www.statnews.com/2017/09/05/watson-ibm-cancer/)).

**[inference]** That is mechanically horismos's exact current shape: one
hand-authored ontology, built from one team's understanding of one scenario, with no
external clinician review yet.

### Epic Sepsis Model

**[measured]** Deployed to hundreds of US hospitals with no published external
validation before that scale. Independent retrospective validation at Michigan
Medicine found AUC 0.63, sensitivity 33%, PPV 12% — clinicians needed to evaluate 109
alerts to find one true, previously-untreated case
([Wong et al., JAMA Internal Medicine 2021](https://jamanetwork.com/journals/jamainternalmedicine/fullarticle/2781307);
STAT News called it "little better than a coin flip,"
[statnews.com](https://www.statnews.com/2021/06/21/epic-sepsis-prediction-tool/); see
also the editorial at
[pubmed.ncbi.nlm.nih.gov/34152360](https://pubmed.ncbi.nlm.nih.gov/34152360/)). A 2024
NEJM AI reanalysis found much of its apparent accuracy was label leakage — encoding "a
clinician already suspects sepsis" rather than physiology — with AUROC collapsing to
0.47 (coin-flip) once post-suspicion data were excluded
([ai.nejm.org](https://ai.nejm.org/doi/full/10.1056/AIoa2300032)). A 2026 JAMA Network
Open study of a silently-updated v2 (Dec 2023) found AUROC 0.82–0.92 but PPV still
only 0.13–0.26
([jamanetwork.com](https://jamanetwork.com/journals/jamanetworkopen/fullarticle/2845595)).

### TREWS and Sepsis Watch (positive counter-examples)

**[measured]** TREWS (Adams et al., Nature Medicine 2022): 590,736 patients, 5
hospitals; confirmed alerts associated with ~18% relative mortality reduction
([nature.com](https://www.nature.com/articles/s41591-022-01894-0)). Sepsis Watch:
Duke's implementation study, live since Nov 2018
([JMIR Med Inform 2020](https://medinform.jmir.org/2020/7/e15182/)); multisite
external validation transferred to 4 new EDs with no retraining, though per-site
threshold tuning was needed
([npj Digital Medicine 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12159134/)).

### DeepMind Streams (NHS)

**[measured]** Royal Free London's own audit disclosure of Streams
([royalfree.nhs.uk](https://www.royalfree.nhs.uk/news/royal-free-london-publishes-audit-streams-app));
a peer review found ~£2,000/patient cost savings, reduced AKI admission costs
([digitalhealth.net, 2019-07](https://www.digitalhealth.net/2019/07/deepminds-streams-saves-2000-peer-review/)).

### DDX-BRO (null RCT, a real non-failure outcome)

**[measured]** A well-powered cluster RCT with a null result on its diagnostic-
quality-risk outcome
([Lancet Digital Health](https://www.thelancet.com/journals/landig/article/PIIS2589-7500(24)00250-4/fulltext)) —
cited in `00-ladder.md` as the kind of common, non-failure outcome to plan
for at rung 4.

## Source

See `../reading-log.md`, Q2 and Q4 sections.

## Confidence

mixed — case outcomes are [measured]; the framing of Watson as horismos's most
directly analogous risk is [inference].

## Last-verified

2026-08-24
