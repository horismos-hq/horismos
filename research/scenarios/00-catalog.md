# Scenario catalog: what's beyond triage

## Claim

A comparison across the field's most-studied clinical decision workflows, scored
against what horismos's architecture specifically needs: (a) bounded,
rule-representable logic; (b) inputs realistic for structured facts or a short-NL
perception step; (c) well-typed tool-effect actions; (d) stakes worth formally
verifying; (e) prior art (FHIR PlanDefinition/CQL/published protocol) to ground the
ontology's vocabulary. Three scenarios — VTE prophylaxis ordering, NEWS2 deterioration
+ RRT escalation, and sepsis bundle compliance-tracking — are recommended as the next
three builds, in that order. See per-scenario pages for detail, and
[`ruled-out.md`](ruled-out.md) for what was excluded and why.

## Evidence

**[measured — table reproduced from the founding sweep]**

| Scenario | (a) Bounded? | (b) Perception-realistic? | (c) Well-typed actions? | (d) Stakes? | (e) Prior art? | Automation evidence | Verdict |
|---|---|---|---|---|---|---|---|
| **VTE prophylaxis ordering** | Yes — Caprini/Padua scoring + contraindication check | Yes — almost entirely structured EHR facts | Yes — order enoxaparin/SCDs | Yes — leading preventable cause of hospital death | Strong (CHEST/ACCP guidelines) | **Strong positive**: [Kucher et al., NEJM 2005](https://www.nejm.org/doi/full/10.1056/NEJMoa041533) RCT (1,255 vs. 1,251 patients) — computer alert markedly reduced DVT/PE | **#1 recommend** |
| **NEWS2 deterioration + RRT escalation** | Yes — literal published points table | Yes — periodic vitals, structured | Yes — page RRT (same shape as triage's nurse page) | Yes — "failure to rescue" is a named safety crisis | Strong (NHS/RCP-published spec) | Mixed on outcomes, but root cause is documented as **protocol non-adherence**, not scoring failure ([systematic review](https://link.springer.com/article/10.1186/s13643-026-03088-y)) | **#2 recommend** |
| **Sepsis bundle (SSC) compliance-tracking** — reframed away from prediction | Yes, scoped to bundle timing/checklist, not prediction | Yes — order/lab/med timestamps | Yes — reassessment/escalation actions | Yes — the field's highest-profile CDS failure *and* a positive counter-example | Strong (SSC is a published, timed checklist) | Predictive variant (Epic Sepsis Model) failed badly and at scale; compliance-tracking has separate positive evidence and a positive prospective precedent ([TREWS, Nature Medicine 2022](https://www.nature.com/articles/s41591-022-01894-0), ~18% relative mortality reduction on confirmed alerts) | **#3 recommend** |
| Prior authorization automation | Yes structurally (Da Vinci CRD/DTR/PAS) | Yes | Yes | High-stakes, adversarial not clinical | Very strong — this is literally the Da Vinci use case | **Severe documented harm** from opaque automation: UnitedHealth/naviHealth nH Predict litigation (alleged "known 90% error rate"), Cigna PXDX (ProPublica: 1.2 sec/claim, batch-denied), active 2026 Senate/OIG inquiry | **Rule out for now** |
| Imaging appropriateness (ACR AUC) | Yes — literal lookup table | Yes | Yes | Moderate–high | Very strong — the FHIR CRD/DTR use case | Positive where deployed, but CMS **paused/rescinded** the federal AUC program in CY2024 — not for clinical reasons but because "the claims system cannot perform this function effectively" | Sound logic, bad timing — deprioritize |
| Chronic-care pathway (diabetes/HTN/HF) | Partial — order-set level only | Mixed, longitudinal | Weak — "recommend" isn't a typed effect | Moderate | Strong (CQL/CPG IG exists) | Weak: 76-study JAMIA meta-analysis found small, heterogeneous effects and **no CDS design feature reliably predicted success** | Rule out |
| ED disposition | No — unbounded gestalt | Weak — social/logistic factors aren't perceivable facts | Weak | High-stakes | Weak — existing tools (STRATIFY) are statistical carve-outs, not rules | Narrow low-risk subsets show promise; general disposition doesn't | Rule out |
| ICU ventilator weaning | No — needs continuous titration | **Poor** — continuous streaming, not discrete perception | Ambiguous | High | Some | Strong positive effect (wean time 4d→2d) | **Rule out** — wrong shape entirely for horismos's discrete-step model |
| Medication reconciliation | Partial — comparison problem | Good NL fit, but needs a huge external drug DB | Weak — "flag discrepancy" isn't world-mutating | High | Moderate (RxNorm, not ontology-native) | Positive (AHRQ), but the mechanism doesn't match horismos's reasoning model | Rule out |
| Antimicrobial stewardship | Partial — restriction-tier logic only | Structured | Yes | Moderate | Some | Modest: 30-trial meta-analysis found median only 3% improvement, "raising questions as to clinical significance" | Plausible 4th choice, not top 3 |
| Fall-risk / delirium screening | Weak — mostly statistical scoring | Weak evidence base | Weak | Moderate | Weak | Thin/early-stage evidence | Rule out for now |

This is a living document — update it as new scenarios are evaluated or built.

## Source

Per-cell citations are in the table; full sources in [`../reading-log.md`](../reading-log.md) Q2
section. Detail pages: [`vte-prophylaxis.md`](vte-prophylaxis.md), [`news2-deterioration.md`](news2-deterioration.md),
[`sepsis-bundle-compliance.md`](sepsis-bundle-compliance.md), [`ruled-out.md`](ruled-out.md).

## Confidence

measured (each row's cited evidence), with the (a)-(e) scoring and overall verdict
column being [inference] applying that evidence against horismos's architecture.

## Last-verified

2026-08-24
