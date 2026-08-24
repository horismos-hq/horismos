# Verification of LLM-drafted artifacts: vacuous verification, the formalization gap, CoT faithfulness

## Claim

Models can satisfy a formal verifier with a trivial specification that technically
passes but constrains nothing — "vacuous verification," a named failure mode with an
exact structural analog in horismos: an LLM-authoring agent could draft a rule that
never fires, or a constraint that is vacuously satisfiable, internally consistent,
BDD-verifiable, and useless or wrong.

## Evidence

**[measured]** "Vacuous verification"/"vacuous assertions" is named and explicitly
designed against in multiple 2025–2026 papers: LASA
([arXiv:2506.17865](https://arxiv.org/abs/2506.17865)), CHARGE (reports non-vacuity
rate as a headline metric,
[arXiv:2607.27776](https://arxiv.org/abs/2607.27776)), and "From Natural Language to
Verified Code" ([arXiv:2604.22601](https://arxiv.org/abs/2604.22601)).

**[measured]** Lahiri (2024) states the structural ceiling directly: "there's no
algorithmic guarantee that formal specifications accurately capture user intent"
([arXiv:2406.09757](https://arxiv.org/abs/2406.09757)) — the verifier checks internal
consistency against a spec, not whether the spec matches what the SME meant. ReForm
(2025/26) names the identical gap in autoformalization: the "formalization gap"
between a statement being provable/verifiable and being faithful to what was
informally meant ([arXiv:2510.24592](https://arxiv.org/abs/2510.24592)).

**[measured]** Composition makes this worse, not just per-artifact: DafnyCOMP found
per-function verification success drops sharply under composition
([arXiv:2509.23061](https://arxiv.org/abs/2509.23061)); VeriContest found 92%
plain-coding success versus only 5.29% end-to-end success on formally-verified
variants ([arXiv:2605.08553](https://arxiv.org/abs/2605.08553)). A repair-loop study
("Is Three the Magic Number?") found diminishing returns after 3–4 repair iterations
([arXiv:2607.05197](https://arxiv.org/abs/2607.05197)).

**[measured, directly undercuts an implicit assumption in PRP-007's design]** PRP-007's
whole review surface rests on the SME seeing only a rendered NL summary plus
scenario-probe proof trees, never raw Turtle — betting that narration quality is what
keeps this safe. Chain-of-thought faithfulness research says this bet should not be
made lightly: Turpin et al. (2023) showed CoT explanations "can systematically
misrepresent the true reason for a model's prediction" while remaining fluent and
plausible ([arXiv:2305.04388](https://arxiv.org/abs/2305.04388)) — and, more
pointedly, Lanham et al. (2023) found larger, more capable models produce *less*
faithful reasoning, not more ([arXiv:2307.13702](https://arxiv.org/abs/2307.13702)), a
finding a 2025–2026 cluster (FaithCoT-Bench, C2-Faith) confirms remains open and
unresolved. No paper in this sweep specifically studies faithfulness of NL renderings
of *separately-verified formal artifacts* (as opposed to general CoT reasoning
traces) — this is a genuine, currently untested gap in exactly the mechanism PRP-007
depends on. **[inference]** The recommended mitigation is empirical, not
architectural: extend PRP-007's own planned DoD test (seeded incompleteness caught by
the verifier) to also seed subtly-wrong-but-internally-consistent axioms, and measure
whether SMEs using the rendered summary + scenario probes actually catch them — treat
this as an assumption to test, not a solved problem.

**[measured]** Precedent for witness/violation NL rendering: xpSHACL renders SHACL
violations in NL, a precedent for horismos's own witness-rendering approach
([vldb.org](https://www.vldb.org/2025/Workshops/VLDB-Workshops-2025/LLM+Graph/LLMGraph-6.pdf)).
ARPaCCino has an LLM draft Rego/OPA policy, iteratively refined against
infrastructure-as-code compliance checks — a related draft-then-gate pattern outside
ontologies ([arXiv:2507.10584](https://arxiv.org/abs/2507.10584)).

## Source

See `../reading-log.md`, Q6 section.

## Confidence

measured (the named failure modes and cited findings), with the mitigation
recommendation flagged [inference].

## Last-verified

2026-08-24
