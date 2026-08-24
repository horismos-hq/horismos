# PRP-007 open items (tracked)

These are recommendations from the `horismos-research-s1` sweep, filed here as open
questions for the captain to act on, defer, or decline. **They are not applied to
PRP-007 by this wiki** — PRP-007's actual scope is a captain-held decision, resolved
only in the owning PRP. This page tracks the items until closed or explicitly
deferred.

## Items

1. **Add an explicit non-vacuity verification pass, separate from consistency-
   checking.** PRP-007 as currently scoped has no explicit check for whether a
   drafted rule/constraint actually binds under some reachable scenario, as distinct
   from being internally consistent. See
   `ontology-authoring/verification-of-llm-drafted-artifacts.md` for the "vacuous
   verification" literature this responds to
   ([LASA](https://arxiv.org/abs/2506.17865),
   [CHARGE](https://arxiv.org/abs/2607.27776),
   [arXiv:2604.22601](https://arxiv.org/abs/2604.22601)).

2. **Extend the planned DoD test to also seed subtly-wrong-but-internally-consistent
   axioms**, not just seeded incompleteness, and measure whether SMEs using the
   rendered NL summary + scenario probes actually catch them. This responds to the
   CoT-faithfulness literature's finding that more capable models can produce *less*
   faithful reasoning (Turpin et al. 2023,
   [arXiv:2305.04388](https://arxiv.org/abs/2305.04388); Lanham et al. 2023,
   [arXiv:2307.13702](https://arxiv.org/abs/2307.13702)) — no paper studies
   faithfulness of NL renderings of *separately-verified formal artifacts*
   specifically, so this is an assumption to test empirically, not a solved problem.
   See `ontology-authoring/verification-of-llm-drafted-artifacts.md`.

3. **`search_vocab`'s output format should support polyhierarchy and provenance
   metadata**, not just an `owl:sameAs` anchor — see
   `terminology/snomed-ct.md` for the full finding (the current
   `examples/triage/snomed-ct-subset.ttl` fixture already shows single-parent
   structure with no version provenance).

## Context: why the sign-off gate should stay hard, not loosen

**[measured]** The rice trait ontology curation study — LLM-as-fallback + mandatory
expert sign-off, kappa 0.719 — is the closest structural sibling to PRP-007's design
found in the literature
([PMC12670841](https://pmc.ncbi.nlm.nih.gov/articles/PMC12670841/)). The
axiom-identification benchmark (F1 0.218–0.642 across 12 LLMs, "not high enough to
fully automate axiom identification,"
[arXiv:2512.05594](https://arxiv.org/abs/2512.05594)) is the strongest evidence-based
argument *for* keeping PRP-007's gate hard rather than lightening it — see
`ontology-authoring/00-overview.md`.

## Source

See `research/reading-log.md`, Q6 section, and the pages linked above.

## Confidence

mixed — the underlying literature findings are [measured]; the specific
recommendations for PRP-007 are [inference] and are captain-held, not applied here.

## Last-verified

2026-08-24
