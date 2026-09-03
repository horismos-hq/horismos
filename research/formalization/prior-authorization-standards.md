# Prior-authorization standards (Da Vinci CRD/DTR/PAS)

## Claim

The Da Vinci Prior Authorization Support IG has the strongest FHIR prior art of any
CIG-adjacent standard surveyed, but adoption is slow (only 35% of PA transactions
fully electronic as of 2023) and the workflow itself is documented as a source of
severe, litigated harm when automated opaquely — which is precisely why the scenario
catalog rules it out as a first horismos build (see [`../scenarios/ruled-out.md`](../scenarios/ruled-out.md)).

## Evidence

**[measured]** The Da Vinci Prior Authorization Support IG defines the CRD/DTR/PAS
architecture; X12-membership friction is a noted adoption barrier
([hl7.org/fhir/us/davinci-pas](https://hl7.org/fhir/us/davinci-pas/)).

**[measured]** CMS's Interoperability and Prior Authorization Final Rule (CMS-0057-F,
2024-01-17) requires FHIR PA APIs
([cms.gov](https://www.cms.gov/newsroom/fact-sheets/cms-interoperability-and-prior-authorization-final-rule-cms-0057-f)).

**[measured]** The CAQH 2023 Index Report found only 35% of PA transactions fully
electronic, up from 28% the prior year
([caqh.org](https://caqh.org/hubfs/43908627/drupal/2024-01/2023_CAQH_Index_Report.pdf)).

See [`../scenarios/ruled-out.md`](../scenarios/ruled-out.md) for the documented harms (UnitedHealth/naviHealth,
Cigna PXDX litigation) that make this a bad *first* horismos scenario despite the
strong prior art.

## Source

- Da Vinci Prior Authorization Support IG — https://hl7.org/fhir/us/davinci-pas/
- CMS Interoperability and Prior Authorization Final Rule —
  https://www.cms.gov/newsroom/fact-sheets/cms-interoperability-and-prior-authorization-final-rule-cms-0057-f
- CAQH 2023 Index Report —
  https://caqh.org/hubfs/43908627/drupal/2024-01/2023_CAQH_Index_Report.pdf

## Confidence

measured

## Last-verified

2026-08-24
