# GLIF / GLIF3

## Claim

GLIF (GuideLine Interchange Format) is a founding Task-Network Model formalism whose
own pilot study documented the exact "encoding-variability" and "vocabulary problem"
that defeated cross-site interchange — and whose authors explicitly flagged natural-
language eligibility-criteria parsing as an unsolved problem in the 1990s, a gap
horismos's `PerceptionStep` directly targets three decades later.

## Evidence

**[measured]** Ohno-Machado et al.'s founding GLIF paper documents that two encoders
of the same guideline, even within one collaboration, produced non-interchangeable
representations
([PMC61313](https://pmc.ncbi.nlm.nih.gov/articles/PMC61313/)). The same paper leaves
eligibility criteria as unstructured narrative text, stating verbatim: "we do not have
natural-language processors that can parse and make sense out of such clauses."

**[measured]** GLIF3's evolution paper (2000) describes a 3-level architecture:
flowchart / verifiable computable specification / implementable specification
([pubmed.ncbi.nlm.nih.gov/11079963](https://pubmed.ncbi.nlm.nih.gov/11079963/)).

**[inference]** Horismos's confidence-gated, JSON-Schema-forced, bounded-retry,
mandatory-fallback `PerceptionStep` is a direct, appropriately distrustful answer to
the exact gap GLIF's authors named and left open for a generation.

## Source

- Ohno-Machado et al., "The GuideLine Interchange Format" (~1998) —
  https://pmc.ncbi.nlm.nih.gov/articles/PMC61313/
- GLIF3 evolution paper (2000) — https://pubmed.ncbi.nlm.nih.gov/11079963/

## Confidence

measured

## Last-verified

2026-08-24
