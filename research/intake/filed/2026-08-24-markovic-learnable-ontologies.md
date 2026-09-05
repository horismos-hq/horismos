# Filed: Markovic, "How do you go from learnable ontologies to world models?"

- **Source**: Vasilije Markovic (founder, Cognee — an open-source AI agent memory
  platform), LinkedIn, Aug 2026.
- **URL**: https://www.linkedin.com/pulse/how-do-you-go-from-learnable-ontologies-world-models-markovic-flqtf/
- **Dropped by**: captain, mid-sweep, `horismos-research-s1` (2026-08-18 per the
  article's own date, filed 2026-08-24).
- **Provenance flags**: not paywalled; vendor-founder-authored (Markovic is Cognee's
  founder, so the piece doubles as product framing); not withdrawn.

## Summary

**[measured]** Markovic argues agent memory can evolve toward a world model via a
continuous, self-reinforcing loop: ingest traces → learn an ontology from them → let
that ontology shape future ingestion → enrich → repeat, with regularities over state
transitions eventually approximating predictive dynamics. The article's own commenters
flag the obvious risk — unsupervised drift in a self-reinforcing loop, and a blurred
line between prediction and authorization-to-act — which the article does not fully
resolve; it explicitly hedges that this isn't yet real predictive dynamics.

## Follow-up pass

**[measured]** Two related sources were traced to place the piece in context:

- Markovic, Obradovic, Hajdu, Pavlovic, "Optimizing the Interface Between Knowledge
  Graphs and LLMs for Complex Reasoning" (2025) — Cognee's own KG-LLM
  hyperparameter-tuning study across HotPotQA/2WikiMultiHop/MuSiQue; gains were
  inconsistent across datasets. https://arxiv.org/abs/2505.24478
- Ken Huang, "Why Ontology Matters for Agentic AI in 2026: From World Models to
  Governable Decisions" (2026) — a thematically adjacent piece framing ontology as a
  "context compiler"/operational interface, citing Palantir Ontology, W3C PROV, NIST
  AI RMF, and the EU AI Act. https://kenhuangus.substack.com/p/why-ontology-matters-for-agentic
- Cognee itself (product/GitHub): open-source AI agent memory platform, MIT-licensed,
  $7.5M seed. https://github.com/topoteretes/cognee

## Positioned against horismos's design

**[inference]** Horismos's existing design already has a structural answer to both
risks the Markovic piece's commenters raised: traces are one-directional output (the
audit log and training corpus, never an input that mutates the ontology), and no
action ever fires off a raw prediction — only off symbolically-checked preconditions.
Where horismos is genuinely *behind* Cognee's ambition is the learning axis: v1 has no
path at all from accumulated execution traces back into candidate ontology edits.

The recommended synthesis for PRP-007 (a captain-held open question, not applied here
— see [`../../ontology-authoring/prp-007-open-items.md`](../../ontology-authoring/prp-007-open-items.md)): don't imitate Cognee's
auto-reinforcing loop. Treat trace-mining as a new **proposal source** feeding the
*existing* gated authoring loop — verifier + SME sign-off stays the only path any
ontology change takes, applying horismos's own "neural proposes, symbolic decides"
principle one level up, at the authoring layer instead of just the runtime layer.

See [`../../ontology-authoring/00-overview.md`](../../ontology-authoring/00-overview.md) and
[`../../ontology-authoring/autonomous-agents-failure-modes.md`](../../ontology-authoring/autonomous-agents-failure-modes.md) for the broader
evidence (NELL's drift, Wikidata's governance-by-human-discussion) this positioning
draws on.
