# Autonomous agents' failure modes: NELL, Wikidata governance, AutoSchemaKG, KARMA

## Claim

The field's longest-running real example of largely-autonomous knowledge extension
(NELL) shows concrete, documented schema drift; the largest real continuously-evolving
public knowledge graph (Wikidata) achieves reliability by the opposite mechanism —
mandatory human discussion before any schema-level change.

## Evidence

**[measured]** NELL (CMU, running continuously since 2010) needed human correction
"every few weeks," and Stuart Russell reports confidence in only 3% of its own
beliefs. NELL's own documented drift chain is the clearest concrete evidence of
unchecked schema drift found anywhere in this sweep: internet *cookies* miscategorized
as *baked goods*, then further deduced into *computer files* by pattern-matching
sentence structure
([en.wikipedia.org/wiki/Never-Ending_Language_Learning](https://en.wikipedia.org/wiki/Never-Ending_Language_Learning)).

**[measured]** Wikidata, the largest real, continuously-evolving public knowledge
graph, achieves reliability by the opposite mechanism: mandatory human community
discussion before any schema-level change, with only instance-level data left to bots
([en.wikipedia.org/wiki/Wikidata](https://en.wikipedia.org/wiki/Wikidata)).

**[measured]** AutoSchemaKG (2025, ACL 2026) performs fully autonomous web-scale
schema induction with no dedup mechanism, and admits residual inconsistency risk
([arXiv:2505.23628](https://arxiv.org/abs/2505.23628)). KARMA (2025) is a 9-agent
autonomous KG enrichment system that is confidence-gated but not systematically
human-reviewed ([arXiv:2502.06472](https://arxiv.org/abs/2502.06472)).

**[measured]** A genuine reasoner-in-loop precedent exists (a HermiT-in-loop
neuro-symbolic integration paper), but it was withdrawn by its own authors for
inaccuracies ([arXiv:2504.07640](https://arxiv.org/abs/2504.07640)). GLaMoR (2025)
replaces a symbolic OWL reasoner with a learned approximator — counter to horismos's
anti-wrapper stance ([arXiv:2504.19023](https://arxiv.org/abs/2504.19023)).

**[inference]** Horismos's existing design already has a structural answer to both
risks these examples raise: traces are one-directional output (never an input that
mutates the ontology), and no action ever fires off a raw prediction — only off
symbolically-checked preconditions. Where horismos is genuinely *behind* is the
learning axis: v1 has no path at all from accumulated execution traces back into
candidate ontology edits. See `../intake/filed/2026-08-24-markovic-learnable-
ontologies.md` for the fuller version of this argument, worked through against a
captain-supplied source.

## Source

See `../reading-log.md`, Q6 section.

## Confidence

mixed — case facts are [measured]; the "horismos already has a structural answer"
framing is [inference].

## Last-verified

2026-08-24
