# Scenarios explicitly ruled out

## Claim

Every scenario the founding sweep explicitly ruled out, with reasons, so the reasoning
isn't silently lost on a future revisit.

## Evidence

**[measured + inference, per scenario]**

- **Prior authorization automation.** Has the strongest FHIR prior art of anything
  surveyed (Da Vinci CRD/DTR/PAS is literally the use case it was built for) and is
  precisely why it's a bad *first* choice — the documented harms came from opaque
  systems serving an adversarial payer incentive, and deploying a pre-alpha framework
  into a litigated, regulator-scrutinized workflow is a liability/maturity mismatch,
  not a technical one. **[measured]** UnitedHealth/naviHealth nH Predict litigation
  alleges a "known 90% error rate"
  ([CBS News](https://www.cbsnews.com/news/unitedhealth-lawsuit-ai-deny-claims-medicare-advantage-health-insurance-denials/));
  ProPublica's Cigna PXDX investigation found 300,000 claims denied in 2 months at
  ~1.2 sec/claim
  ([propublica.org](https://www.propublica.org/article/cigna-pxdx-medical-health-insurance-rejection-claims));
  active 2026 Senate/OIG inquiry into Medicare Advantage AI-driven denials
  ([STAT News](https://www.statnews.com/2026/07/15/medicare-advantage-ai-care-denials-probe-blumenthal-hawley/);
  [STAT News OIG coverage](https://www.statnews.com/2026/06/11/medicare-advantage-oig-report-rehab-care-deny-appeal-reverse/)).
  It's a strong *future* direction once horismos has an established trust record.

- **Imaging appropriateness (ACR AUC).** Sound decision logic (a literal lookup
  table) but a politically dead federal program: CMS paused/rescinded the AUC program
  in CY2024, not for clinical reasons but because "the claims system cannot perform
  this function effectively"
  ([ACC](https://www.acc.org/Latest-in-Cardiology/Articles/2023/12/14/15/16/CMS-Pauses-AUC-Program-For-Advanced-diagnostic-Imaging-Rescinds-Current-Regulations)).
  Deprioritize, not rule out permanently.

- **Chronic-care pathway (diabetes/HTN/HF).** Doesn't match the architecture's
  bounded, terminal-step shape — "recommend" isn't a typed tool effect. A 76-study
  JAMIA meta-analysis found small, heterogeneous effects and no CDS design feature
  reliably predicted success
  ([academic.oup.com](https://academic.oup.com/jamia/article/29/10/1757/6639847)).

- **ED disposition.** Unbounded gestalt judgment, not rule-representable; social/
  logistic factors aren't perceivable facts. Existing tools like STRATIFY are
  statistical carve-outs, not rules
  ([springer.com](https://link.springer.com/article/10.1186/s43058-025-00779-w)).
  Narrow low-risk subsets show promise; general disposition doesn't.

- **ICU ventilator weaning.** The worst architectural fit surveyed despite the
  strongest trial evidence (iCareWean cut wean time 4 days → 2 days,
  [AJRCCM](https://www.atsjournals.org/doi/10.1164/rccm.200511-1780OC)) — closed-loop
  continuous titration has no discrete decision point to formalize; horismos's
  discrete-step model doesn't fit continuous streaming perception.

- **Medication reconciliation.** Good NL fit for perception, but needs a huge external
  drug database (RxNorm, not ontology-native) and "flag discrepancy" isn't a
  world-mutating typed effect. Positive evidence exists (AHRQ MATCH Toolkit,
  https://www.ahrq.gov/patient-safety/settings/hospital/match/index.html), but the
  mechanism doesn't match horismos's reasoning model.

- **Fall-risk / delirium screening.** Mostly statistical scoring with a weak evidence
  base; thin/early-stage evidence overall. Ruled out for now, not permanently.

- **Antimicrobial stewardship.** Not ruled out — a plausible 4th choice, not top 3.
  Restriction-tier logic is well-typed and structured, but a 30-trial meta-analysis
  found median only 3% improvement, "raising questions as to clinical significance"
  ([researchgate.net](https://www.researchgate.net/publication/331401165); RCT
  context: [Clinical Infectious Diseases WISCA trial](https://academic.oup.com/cid/article/72/9/e265/5876588)).

## Source

See per-item citations above; all also listed in [`../reading-log.md`](../reading-log.md) Q2
section.

## Confidence

mixed — outcome/evidence claims are [measured]; the rule-out reasoning per scenario is
[inference].

## Last-verified

2026-08-24
