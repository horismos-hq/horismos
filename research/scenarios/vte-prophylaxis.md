# Scenario: VTE prophylaxis risk-stratified ordering

## Claim

**#1 recommended next scenario.** The closest available match to triage's existing
shape — bounded scoring (Caprini/Padua) + contraindication check, almost entirely
structured EHR facts, well-typed order actions, and a real RCT showing the mechanism
class works.

## Evidence

**[measured]** Kucher et al., NEJM (2005): an RCT of 1,255 vs. 1,251 patients found a
computer alert markedly reduced DVT/PE rates
([nejm.org](https://www.nejm.org/doi/full/10.1056/NEJMoa041533)).

**[inference]** This scenario is reusable almost entirely via horismos's already-built
"structured-input bypass" (facts asserted directly with `provenance: api`, no Bridge
configured at all), which lets it be built and formally verified as a fully symbolic
deployment first, adding an optional NL perception step later. It diversifies the
domain vocabulary (CHEST guideline terms, Caprini/Padua) away from SNOMED symptom
subsumption, and Kucher et al.'s RCT gives horismos a real external validation of the
*mechanism class* to point to, decoupled from any vendor's black-box implementation.

## Source

- Kucher et al., "Electronic Alerts to Prevent Venous Thromboembolism among
  Hospitalized Patients," NEJM (2005) —
  https://www.nejm.org/doi/full/10.1056/NEJMoa041533

## Confidence

mixed — the RCT result is [measured]; the fit-to-horismos-architecture reasoning and
recommendation ranking are [inference].

## Last-verified

2026-08-24
