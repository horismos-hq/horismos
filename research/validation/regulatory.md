# Regulatory shape (FDA SaMD / CDS-exemption / PCCP)

## Claim

The FDA's non-device CDS exemption is structurally favorable to horismos's proof-tree
transparency, but does not guarantee automatic qualification, especially for
time-critical scenarios. The FDA's Predetermined Change Control Plan (PCCP) guidance,
finalized December 2024 *[corrected 2026-09-21 — the PCCP guidance is dated August 2025;
see the Jev deep dive delta below]*, maps unusually well onto horismos's
ontology-as-versioned-artifact model — a genuinely strong story to have ready for a
regulator much later, not now.

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

## Jev deep dive delta (2026-09-21): three markets called "auditability", moving in different directions

**[measured — primary]** Two corrections to the text above, both found by reading the FDA
documents themselves rather than analyses of them:

1. **The PCCP guidance is dated AUGUST 2025**, current as of 2025-08-18 — not December
   2024. The Evidence section's reasoning is unaffected; the date is wrong
   ([fda.gov](https://www.fda.gov/regulatory-information/search-fda-guidance-documents/marketing-submission-recommendations-predetermined-change-control-plan-artificial-intelligence)).
2. **The revised CDS guidance's cover date is JANUARY 2026**, docket FDA-2017-D-6569 — the
   sweep #2 delta could only reach it through a secondary analysis and marked the month
   `[unverified]`. The 27-page PDF confirms it
   ([fda.gov/media/109618/download](https://www.fda.gov/media/109618/download)). Its
   silence on AI is confirmed at token level: `artificial intelligence` **0**,
   `machine learning` **0**, `generative` **0**, `calibrat` **0**, `uncertaint` **0**,
   against `independently review` **17**, `basis for` **19**, `time-critical` **16**.

**[measured — primary]** Criterion 4's operative language reads like a specification for a
proof tree, and is worth having verbatim on this page:

> "(d) The software output provides the HCP user with relevant patient-specific information
> and **other knowns/unknowns for consideration** (e.g., missing, corrupted, or unexpected
> input data values) that will enable the HCP to independently review the basis… Beyond the
> input medical information relied upon, **being able to clearly understand how the logic
> was applied for the patient (e.g., matching of patient-level data to criteria in practice
> guidelines, comparison to a reference/normal database, development and use of a
> prioritized schema for recommendation generation) will aid an HCP in understanding the
> basis of the recommendations.**"

It also requires the basis be "presented **in a manner that promotes usability and avoids
information overload, including prioritizing the most decision-relevant information**", and
FDA's own compliant example asks for output that "indicates whether any expected input
medical information from the medical record was missing". **[inference]** That maps onto
forward chaining over declared rules, `horis:priority` with its `Suppressed`/`Overridden`
outcomes, and the gate's `Witness` respectively. The time-critical exclusion still bites —
"FDA does not consider software functions intended for a critical, time-sensitive task or
decision to meet Criterion 4" — so the founding claim above stands: the triage scenario
likely fails Criterion 4 on timing regardless of proof quality. The enforcement-discretion
example is sharper than recorded: a cardiovascular-risk predictor qualifies, but one that
"predicts risk of a cardiovascular event **in the next 24 hours** … is a device."

**[measured — primary]** **New since sweep #2: FDA CDRH, "Considerations for the Regulation
of Generative AI-Enabled Medical Devices: Discussion Paper and Request for Feedback"**, page
current 2026-08-18, docket **FDA-2026-N-7874**, comments due **2026-10-19**, 31 pp,
expressly "does not represent draft or final guidance"
([fda.gov/media/194242/download](https://www.fda.gov/media/194242/download)). Term counts
across the full document: `explain`/`explainab` **0**, `interpretab` **0**, `transparen`
**1**, `calibrat` **2**, `uncertaint` **9**, `traceab` **2**. It proposes calibration as a
premarket benchmarking element — "**S.3. Calibration, uncertainty communication, and
clinical deferral** … Presenting uncertain, outdated, or contested information with false
confidence is treated as a safety failure" — *and* promotes reviewability to a risk driver:
functions "may nonetheless be of higher risk because **the user typically cannot
independently evaluate the basis for the output (and therefore the function may migrate
higher on the consequences axis)**", with footnote 13 tying that to "the independent-review
criterion of the clinical decision support exclusion under section 520(o)(1)(E)". Discussion
Question 1 asks whether "the traceability of the output (i.e., to primary source materials)"
should be a risk dimension.

**[inference]** **FDA's position, stated precisely: calibration is a safety floor;
inability to show the basis is a risk multiplier. They stack; they do not compete.** A
calibrated proposer does not buy a product out of a reason, and a reason does not buy it out
of calibration.

**[measured]** **HTI-5 is no longer merely proposed.** The OIRA "Executive Order Submissions
Under Review" page, dated 2026-09-21, lists `HHS-ONC | RIN 0955-AA09 | Status: Pending
Review | STAGE: Final Rule | RECEIVED DATE: 07/28/2026`
([reginfo.gov](https://www.reginfo.gov/public/do/eoPackageMain)); the Unified Agenda's
projected `Final Action 08/00/2026` has slipped and the Federal Register API returns zero
2026 ASTP/ONC rules. **[inference]** Publication is plausible October–November 2026. "No
final rule as of 2026-09-02" above remains true and is misleading if read as "stalled". The
NPRM preamble (90 FR 60970) would strike **all** of §170.315(b)(11)(iv)–(vi) and remove and
reserve §170.315(a)(9), leaving no certification requirement that a decision-support tool
cite its source at all, with no replacement proposed. Its rationale is the strongest
evidence against the transparency thesis anywhere in the corpus: "we have no publicly
available evidence indicating that **a single doctor, nurse, or administrator has accessed,
recorded, or modified a single source attribute** … the underlying assumption … **was
incorrect**", retracting a projected $3 billion benefit. AHA, WEDI and MGMA opposed
(6,400+ comments); the party cited in support is the EHR Association.

**[inference]** The finding this page should carry forward is that **"auditability" is three
different markets**:

| | What it is | 2026 direction |
|---|---|---|
| System-level transparency | Model cards, source attributes — metadata beside the product | **Dismantled**, with a regulator's evidentiary finding that nobody used it |
| Per-decision reason-giving to the professional | FDA Criterion 4's "how the logic was applied for the patient" | **Strengthening**, and promoted to a risk driver in the GenAI paper |
| Per-decision reason-giving to the affected person | GDPR Art. 15(1)(h), ECOA § 1002.9(b)(2), AI Act Art. 86, CA HSC § 1367.01(k) | **Holding** — and holding where the duty is *old* and privately enforceable |

**[measured — primary]** Beyond FDA, the teeth are sharper than this page previously
recorded. CJEU **C-203/22 (*CK v Dun & Bradstreet Austria*, 2025-02-27)** ¶ 59: the Art.
15(1)(h) duty "**cannot be satisfied either by the mere communication of a complex
mathematical formula, such as an algorithm, or by the detailed description of all the steps
in automated decision-making**" — a score plus a weighting statement was held insufficient,
and so, note, would a raw derivation dump be. **C-634/21 (*SCHUFA*, 2023-12-07)**: the
probability value *itself* is the Art. 22 decision where a third party draws strongly on it.
**12 CFR § 1002.9(b)(2)**: statements that an applicant "failed to achieve a qualifying
score … are insufficient", with a private right of action under 15 U.S.C. § 1691e.
**California SB 1120 / HSC § 1367.01(k)** (in force 2025-01-01): a utilization-review tool
must base its determination on individual clinical history and "[i]ndividual clinical
circumstances as presented by the requesting provider", not "solely on a group dataset", and
medical necessity is reserved to a licensed clinician. **EU AI Act**: Regulation (EU)
2026/1744 (in force 2026-07-27) deferred Chapter III §§1–3 to 2027-12-02 (Annex III) and
2028-08-02 (Annex I), but **left Art. 86 — the right to explanation — undeferred**, and
Annex III point 5(d) expressly lists "emergency healthcare patient triage systems".
**[unverified]** Whether Art. 86 bites while the classification article it depends on is
deferred is genuinely unsettled and **no practitioner source addresses it**.

**[measured]** And where a calibrated number suffices, recorded so the claim stays bounded:
NYC Local Law 144 is an audit regime with no individual-explanation duty; Fed **SR 26-2**
and **OCC Bulletin 2026-13** (both 2026-04-17) superseded SR 11-7 and carve generative and
agentic AI out of scope entirely; Texas TRAIGA, Illinois HB 3773, Utah AIPA and California's
FEHA ADS regulations require notice and non-discrimination but no reason; and Colorado's
"principal reason or reasons" duty was delayed, enjoined (*X.AI LLC v. Weiser*, D. Colo.,
stay 2026-04-27) and then repealed and reenacted without that phrase by **SB 26-189** (signed
2026-05-14, effective 2027-01-01).

**[inference]** The reframing this page should adopt: the defensible claim is not "buyers
want auditability" — the AMA's 2026 physician survey (n=1,692) ranks validation (88%) and
privacy (86%) first and never mentions explainability — but **"non-reviewability is a cost
multiplier"**: it moves a product from non-device to device at FDA, up the consequences axis
in the GenAI framework, into Art. 86 exposure in the EU, and into per-se insufficiency under
§ 1002.9(b)(2) and § 1367.01(k). That argument is addressed to general counsel and
regulatory affairs, not to the clinician. Full treatment, including the clinical
human-factors evidence that cuts the other way, in the
[Jev deep dive §6.3](../reports/2026-09-21-typesafe-jev-deep-dive.md#63-the-auditability-wedge).

## Source

See [`../reading-log.md`](../reading-log.md), Q4 section, and the Q9 section's
regulation/auditability group added by the Jev deep dive.

## Confidence

mixed — the regulatory text and guidance content are [measured]; how they'd apply to
horismos specifically is [inference], since no horismos-specific regulatory filing
exists yet.

## Last-verified

2026-09-21 (Jev deep dive delta appended: two dates corrected — the PCCP date, which appears
in both the Claim and Evidence, and the revised CDS guidance's date in Evidence; the January 2026
CDS guidance and the August 2026 GenAI discussion paper read as primary sources, non-FDA
regimes added; sweep #2 delta unchanged, last checked 2026-09-02)
