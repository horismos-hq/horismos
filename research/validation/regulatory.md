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

## Sweep #2 delta (2026-09-02): the 2022 CDS guidance is superseded; DSI transparency mandates are being withdrawn

**[measured]** FDA issued a revised Clinical Decision Support Software final guidance
in January 2026, superseding the 2022 guidance cited above (search snippets give the
issue date as January 6 or January 29; fda.gov returned 404 to every fetch → the exact
date is [unverified]; the Covington & Burling analysis was read in full). Criterion 4
keeps "enabling the HCP to independently review the basis for the recommendations … so
that it is not the intent that the HCP rely primarily on any of such recommendations";
FDA now "exercises enforcement discretion for singular recommendations when clinically
appropriate" where the 2022 text required option lists; the time-critical exclusion
moved from Criterion 3 to Criterion 4; transparency expectations for inputs and logic
are maintained; the guidance "remains silent on AI-enabled tools, generative AI
applications, and medical chatbots"
([Covington](https://www.cov.com/en/news-and-insights/insights/2026/01/5-key-takeaways-from-fdas-revised-clinical-decision-support-cds-software-guidance);
[FDA guidance page](https://www.fda.gov/regulatory-information/search-fda-guidance-documents/clinical-decision-support-software)).

**[measured]** HTI-5 (ASTP/ONC, proposed 2025-12-29, comments closed 2026-02-27, no
final rule as of 2026-09-02) would remove legacy CDS §170.315(a)(9) and revise DSI
§170.315(b)(11) to "fully remove the artificial intelligence (AI) 'model card'
requirements," aiming to "establish a new foundation on which to build FHIR-based API
requirements in the future"
([fact sheet](https://www.healthit.gov/topic/laws-regulation-and-policy/hti-5-proposed-rule-fact-sheet);
[overview deck](https://healthit.gov/wp-content/uploads/2026/02/HTI-5-Proposed-Rule-Overview.pdf);
[status](https://healthit.gov/regulations/hti-rules/hti-5-proposed-rule/)).

**[inference]** The founding claim above stands and sharpens: Criterion 4 now
explicitly tolerates a single recommendation, so a card whose narration cites the proof
is the exemption artifact, while an authorized write with no HCP review step would sit
on the device side of the line — bears on PRP-008 D5 (output posture), not decided
here. Trace transparency is becoming a differentiator rather than a certification
checkbox.

## Source

See [`../reading-log.md`](../reading-log.md), Q4 section.

## Confidence

mixed — the regulatory text and guidance content are [measured]; how they'd apply to
horismos specifically is [inference], since no horismos-specific regulatory filing
exists yet.

## Last-verified

2026-09-02 (sweep #2 delta appended; the 2022 FDA CDS guidance cited in Evidence is superseded — see delta)
