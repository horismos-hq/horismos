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

## Sweep #2 delta (2026-09-02): the formalization gap gets numbers; the narration boundary gets an attack

**[measured]** ShadowBench (EMNLP 2026) checks autoformalized Lean 4 statements
against "shadow" statements by compilation plus forward and backward implication:
Claude Opus 4.8 reaches 61.8% compile rate but 11.2% semantic-alignment pass on 178
research-level problems, with 98.8% concordance to expert judgment
([arXiv:2608.29270](https://arxiv.org/abs/2608.29270)). A PRISMA survey of 38
agent-safety studies finds NL→formal translation "achieves only 24% to 35% semantic
correctness" and that "no existing approach simultaneously achieves soundness,
scalability, semantic correctness, and task-level safety preservation"
([arXiv:2608.14590](https://arxiv.org/abs/2608.14590)). Legal-reasoning
autoformalization shows "scope laundering (reporting conclusions without executing
formal logic), implicit constraint blindness, and program synthesis failures," with
scope laundering persisting "across all models"
([arXiv:2606.16118](https://arxiv.org/abs/2606.16118)); another study finds models
"mistranslat[e] premises during formalization, producing internally consistent outputs
that evade detection entirely" at 87–99% compile rates
([arXiv:2604.19459](https://arxiv.org/abs/2604.19459)). Detection is improving: a
faithfulness probe catches 89.6% of drifted Lean formalizations at 3.0% false-positive
rate versus 41.2% for type-checking alone
([arXiv:2606.16541](https://arxiv.org/abs/2606.16541)); round-trip formalize →
back-translate → re-formalize → equivalence-check flags failing rules with 1.4–2.5x
more NLI drift ([arXiv:2604.25031](https://arxiv.org/abs/2604.25031)).

**[measured]** Vacuity: no paper dated after 2026-08-24. The most recent work: KaPilot
appends `#[kani::ensures(|_| false)]` and calls a spec non-vacuous only if verification
fails after and succeeds before — no vacuity rate reported
([arXiv:2607.21957](https://arxiv.org/abs/2607.21957)); an RL study found verified
reward rising 2.2% → 58.1% was "largely specification hacking," dropping to 9.7% → 31.1%
after filtering weak specs ([arXiv:2605.30914](https://arxiv.org/abs/2605.30914));
mutation-guided rejection of vacuous LLM-written RTL assertions
([arXiv:2606.21451](https://arxiv.org/abs/2606.21451)); and "vacuity failure" of risk
bounds — a never-acting selector drives risk to zero — excluded by construction in
actionability certificates ([arXiv:2608.12444](https://arxiv.org/abs/2608.12444)), a
different sense of vacuity relevant to stating what a passed check rules out.

**[measured]** The narration boundary: in an LLM–solver loop the certificate "ties to a
property of a formula, not to the sentence that [the] LLM model outputs." Under prompt
injection, five open models inverted the verified conclusion in 61% of 120 trials; a
hardened prompt cut this to 14% but the stealthy share (verdict reported correctly,
conclusion flipped) moved only from 0.57 to 0.53 and adaptive attacks recovered to 43%.
Theorem 3.1: a conclusion monitor comparing narrated answer to verified verdict is sound
and complete; "the only defense that drives their rate to zero is enforcement, which
reports ρ(v) in place of the model conclusion"
([arXiv:2606.19588](https://arxiv.org/abs/2606.19588)).

**[measured]** Verifier-before-run peers: FORGE runs four static analyses over Datalog
agent policies before deployment — contradiction, redundancy, subsumption, conditional
reachability — and states its correctness theorem only "when the environment contract
holds" (τ²-bench compliance 58% → 98%)
([arXiv:2602.16708](https://arxiv.org/abs/2602.16708)); proof-carrying certificates for
LLM pipelines partition declared assumptions by tier and turn abstention into a
"maximum-weight certifiable residue with audit-logged dropped claims"
([arXiv:2605.16407](https://arxiv.org/abs/2605.16407)).

**[inference]** Bearing on the recommendations above and on PRP-005: (a) PRP-007's
rendered-summary review surface should be complemented by a deterministic conclusion
monitor (narrated conclusion vs proof verdict) — the narration paper's own remedy; (b)
the seeded-wrong-axiom DoD test now has measured baselines to beat — a five-to-one
compile-vs-meaning gap and 41.2% typecheck-only detection; (c) FORGE's four analyses are
a direct peer for PRP-005's rule-set checks, and its closed-world stance ("Any fact
absent … may be treated as false") is the contrast that makes Horismos's
absent-vs-not-fetched distinction a stated differentiator. Flagged in
[`../neurosymbolic-landscape/pivot-alignment.md`](../neurosymbolic-landscape/pivot-alignment.md) (A8, A14); owners unchanged.

## Source

See [`../reading-log.md`](../reading-log.md), Q6 section.

## Confidence

measured (the named failure modes and cited findings), with the mitigation
recommendation flagged [inference].

## Last-verified

2026-09-02 (sweep #2 delta appended; founding claims unchanged)
