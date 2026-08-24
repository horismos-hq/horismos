# Regulatory shape (FDA SaMD / CDS-exemption / PCCP)

## Claim

The FDA's non-device CDS exemption is structurally favorable to horismos's proof-tree
transparency, but does not guarantee automatic qualification, especially for
time-critical scenarios. The FDA's Predetermined Change Control Plan (PCCP) guidance,
finalized December 2024, maps unusually well onto horismos's ontology-as-versioned-
artifact model — a genuinely strong story to have ready for a regulator much later,
not now.

## Evidence

**[measured]** The FDA's non-device CDS exemption (21st Century Cures Act
§520(o)(1)(E)) requires software to "enable a health care professional to
independently review the basis for the recommendation"
([Federal Register, 2022-09-28](https://www.federalregister.gov/documents/2022/09/28/2022-20993/clinical-decision-support-software-guidance-for-industry-and-food-and-drug-administration-staff)).
Horismos's proof-tree transparency is structurally favorable to that language in a way
a black-box classifier is not — but a time-critical acuity assignment (like the
current triage scenario) plausibly fails the exemption's "not primarily relying on the
software" prong regardless of transparency. **[inference]**

**[measured]** The FDA's PCCP guidance, finalized December 2024, authorizes
pre-authorized bounded future AI-device modification envelopes
([fda.gov](https://www.fda.gov/regulatory-information/search-fda-guidance-documents/marketing-submission-recommendations-predetermined-change-control-plan-artificial-intelligence)).
**[inference]** A PCCP for horismos would plausibly describe bounded *ontology* edits
(new rules, threshold tweaks) verified pre-deployment and diffed against BDD
witnesses — a genuinely strong, evidence-backed story to have ready for a regulator
much later, not now.

**[measured]** Related regulatory landscape: the FDA/Health Canada/MHRA joint Good
Machine Learning Practice (GMLP) guiding principles (10 principles, 2021-10,
[fda.gov](https://www.fda.gov/media/153486/download)); the FDA's public AI/ML-Enabled
Medical Devices registry
([fda.gov](https://www.fda.gov/medical-devices/software-medical-device-samd/artificial-intelligence-and-machine-learning-aiml-enabled-medical-devices));
a 27-year retrospective finding ~5–6% of AI/ML devices ever recalled, mostly software
bugs ([PMC12274014](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC12274014/)).

## Source

See `../reading-log.md`, Q4 section.

## Confidence

mixed — the regulatory text and guidance content are [measured]; how they'd apply to
horismos specifically is [inference], since no horismos-specific regulatory filing
exists yet.

## Last-verified

2026-08-24
