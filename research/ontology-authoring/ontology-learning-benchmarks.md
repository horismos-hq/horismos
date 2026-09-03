# Ontology-learning benchmarks

## Claim

LLM-based ontology learning is strong at term typing/entity extraction and weak at
relation/axiom identification — precisely the layer PRP-007's authoring agent would
need to be strong at.

## Evidence

**[measured]** The LLMs4OL 2024 Overview reports term typing F1 0.9938 versus relation
extraction F1 0.0783 ([arXiv:2409.10146v1](https://arxiv.org/html/2409.10146v1)) — an
enormous gap between the two task types.

**[measured]** "Ontology Learning with LLMs: Axiom Identification Benchmark"
(OntoAxiom, 2025/26) found F1 scores 0.218–0.642 by domain across 12 LLMs and 2,771
axioms, concluding performance is "not high enough to fully automate axiom
identification" ([arXiv:2512.05594](https://arxiv.org/abs/2512.05594)).

**[measured]** OntoGPT (Monarch Initiative) is the most mature tool found for
structured extraction against an existing vocabulary — but it does structured
extraction, not from-scratch axiom authoring
([github.com/monarch-initiative/ontogpt](https://github.com/monarch-initiative/ontogpt)).

**[measured]** Manda's review of LLMs in bio-ontology research documents a maturity
tiering and names "workload shift rather than reduction" as a real failure mode — LLM
assistance moves effort around rather than eliminating it
([PMC12649945](https://pmc.ncbi.nlm.nih.gov/articles/PMC12649945/)).

**[measured]** A rice trait ontology curation study using LLM-as-fallback with
mandatory expert sign-off achieved kappa 0.719 — the closest structural sibling to
PRP-007's design found in the literature
([PMC12670841](https://pmc.ncbi.nlm.nih.gov/articles/PMC12670841/)); see
[`prp-007-open-items.md`](prp-007-open-items.md).

## Source

See [`../reading-log.md`](../reading-log.md), Q6 section.

## Confidence

measured

## Last-verified

2026-08-24
