# Q6: agent-authored and learnable ontologies — overview

## Claim

The formal-code-verification literature has a named failure mode with no
ontology-specific precedent yet but an exact structural analog in horismos: "vacuous
verification." No system anywhere in the surveyed literature was found autonomously
extending its own ontology with zero human review in real deployment — PRP-007's hard
sign-off gate is, if anything, ahead of the field's own best practice. This is the
single most concrete, actionable finding of the founding sweep; see
`prp-007-open-items.md` for the specific gaps it implies.

## Evidence

**[measured]** The field's own systematic literature review (Li, Garijo &
Poveda-Villalón, *Semantic Web Journal*, 2026, screening 15,688 papers down to 36)
found no standardized benchmarks, inconsistent metrics, poor reproducibility, and
minimal human-in-the-loop integration across LLM-based ontology engineering generally
([semantic-web-journal.net](https://www.semantic-web-journal.net/content/large-language-models-ontology-engineering-systematic-literature-review)) —
PRP-007's hard verifier-plus-sign-off design is stricter than most of the field's own
practice, not an outlier of excess caution.

**[measured]** The axiom/rule layer specifically — exactly where PRP-007's authoring
agent operates — is documented as the field's hardest, weakest layer: a 2025/2026
axiom-identification benchmark across 12 LLMs and 2,771 axioms found F1 scores ranging
0.218 to 0.642 by domain, concluding performance is "not high enough to fully automate
axiom identification"
([arXiv:2512.05594](https://arxiv.org/abs/2512.05594)) — the strongest evidence-based
argument *for* keeping PRP-007's gate hard rather than lightening it.

**[measured]** The closest external architectural analog found: open-ontologies
(Tesseract Academy) — a deterministic Rust OWL2-DL+SHACL engine that gates LLM tool
calls, self-reporting 9th/13 on an OAEI benchmark
([github.com/fabio-rovai/open-ontologies](https://github.com/fabio-rovai/open-ontologies)).

See `ontology-learning-benchmarks.md`, `autonomous-agents-failure-modes.md`,
`verification-of-llm-drafted-artifacts.md`, and `prp-007-open-items.md` for the
detailed findings this overview summarizes.

## Source

See `../reading-log.md`, Q6 section.

## Confidence

measured

## Last-verified

2026-08-24
