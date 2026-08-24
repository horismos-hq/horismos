# Scenario: NEWS2 deterioration + rapid-response escalation

## Claim

**#2 recommended next scenario.** A cleaner fit than triage's acuity subsumption in
one respect — the scoring is a literal published table rather than a rule set
horismos has to author from domain judgment — and it targets a documented failure
mode (good score, poor outcomes, because escalation protocols aren't followed) that
horismos's workflow-level formalization, not just a risk-score calculator, directly
addresses.

## Evidence

**[measured]** NEWS2 prognostic accuracy studies report AUROC ~0.67–0.69 for
mortality, but poor performance for predicting cardiac arrest specifically
([PMC8718668](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8718668/)).

**[measured]** A 2026 NEWS/NEWS2 systematic review finds outcome improvement is
inconsistent, and attributes this to escalation-protocol non-adherence rather than
scoring inaccuracy
([springer.com](https://link.springer.com/article/10.1186/s13643-026-03088-y)).

**[inference]** This is precisely what horismos's *workflow-level* formalization
targets: encode the full escalation contract (page RRT, timing, acknowledgment), not
just the lookup table. The resulting proof trace becomes audit-grade evidence of
protocol adherence, not just prediction accuracy — a structurally different claim than
"the score is accurate."

## Source

- NEWS2 prognostic accuracy study —
  https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8718668/
- NEWS/NEWS2 systematic review (2026) —
  https://link.springer.com/article/10.1186/s13643-026-03088-y

## Confidence

mixed — outcome findings are [measured]; the fit-to-horismos-architecture reasoning is
[inference].

## Last-verified

2026-08-24
