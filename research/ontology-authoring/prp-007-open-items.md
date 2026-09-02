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
   `verification-of-llm-drafted-artifacts.md` for the "vacuous
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
   See `verification-of-llm-drafted-artifacts.md`.

3. **`search_vocab`'s output format should support polyhierarchy and provenance
   metadata**, not just an `owl:sameAs` anchor — see
   `../terminology/snomed-ct.md` for the full finding (the current
   `examples/triage/snomed-ct-subset.ttl` fixture already shows single-parent
   structure with no version provenance).

4. **[inference]** **Restate the sign-off gate's rationale from accuracy to safety, and treat the
   verifier's consistency pass as necessary but not sufficient.** (Sweep #2,
   2026-09-02.) Ontology-refinement loops with no human gate now report benchmark gains
   — OaK, LLM judge plus HermiT ([arXiv:2608.22974](https://arxiv.org/abs/2608.22974));
   ASKS ([arXiv:2608.29612](https://arxiv.org/abs/2608.29612)) — so "the gate is what
   makes it accurate" is no longer a safe argument; and "single-axiom OWL edits achieve
   93.3% SIV success with 100% Pellet-consistency stealth"
   ([arXiv:2606.17223](https://arxiv.org/abs/2606.17223)), so a reasoner-consistency
   check alone cannot be the gate. Implied: SME review of the *semantic diff*, not only
   the rendered summary; non-vacuity (item 1); per-fact provenance on every derived
   conclusion. See `autonomous-agents-failure-modes.md` (sweep #2 delta) and
   `../neurosymbolic-landscape/pivot-alignment.md` (X2, A10).

5. **[inference]** **Capability-fragment compiler mode: name the export surfaces and widen the effect
   model.** (Sweep #2.) Claude Code `SKILL.md` `hooks` + `metadata`; Gemini CLI
   `toolAnnotations`; MCP `tools/resolve` (draft SEP-1862); DeepSeek Harness's Claude
   Code/Codex hook bridge; effect constraints must cover credentials and endpoints, not
   only paths. For the envelope D-item shared with PRP-008: wrap Codex V4A `apply_patch`
   verbatim and put the base-hash check in the policy stage, not the editing tool. See
   `../neurosymbolic-landscape/coding-agent-harnesses.md` and
   `../neurosymbolic-landscape/pivot-alignment.md` (A11, A12).

## Context: why the sign-off gate should stay hard, not loosen

**[measured]** The rice trait ontology curation study — LLM-as-fallback + mandatory
expert sign-off, kappa 0.719 — is the closest structural sibling to PRP-007's design
found in the literature
([PMC12670841](https://pmc.ncbi.nlm.nih.gov/articles/PMC12670841/)). The
axiom-identification benchmark (F1 0.218–0.642 across 12 LLMs, "not high enough to
fully automate axiom identification,"
[arXiv:2512.05594](https://arxiv.org/abs/2512.05594)) is the strongest evidence-based
argument *for* keeping PRP-007's gate hard rather than lightening it — see
`00-overview.md`.

## Source

See `../reading-log.md`, Q6 section, and the pages linked above.

## Confidence

mixed — the underlying literature findings are [measured]; the specific
recommendations for PRP-007 are [inference] and are captain-held, not applied here.

## Last-verified

2026-09-02 (items 4–5 added by sweep #2; items 1–3 unchanged)
