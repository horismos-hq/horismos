# Lineage assessment: what lineage is horismos really in?

## Claim

If forced to pick one lineage for the whole system: horismos's symbolic core is
classical rule-based CDS / expert-system lineage (MYCIN, Arden Syntax), substantially
modernized. Its neural-boundary engineering is genuinely closer to where 2025–2026
formal-agent-verification research is heading than to anything MYCIN, Arden, or any
current clinical AI product ships today. Calling the whole thing "neurosymbolic AI"
unqualified invites a direct, informed comparison to Scallop/LNN/DeepProbLog that
horismos will lose on its own terms (no differentiability, no gradient training).
Revisit this page each time positioning language changes.

## Evidence

**[measured]** Horismos's reasoning core — discrete forward/backward chaining, zero
probability in inference, proof traces — is architecturally continuous with the
classical expert-system lineage: MYCIN's backward chaining and explanation-from-trace
([Wikipedia, MYCIN](https://en.wikipedia.org/wiki/Mycin)), and Arden Syntax's Medical
Logic Modules (still an active HL7 standard, v3.0 published October 2023;
[Wikipedia, Arden Syntax](https://en.wikipedia.org/wiki/Arden_syntax)) — even though
the field's own Kautz Type 1–6 neurosymbolic taxonomy, as reflected in Wikipedia's
overview, does not mention that lineage at all
([en.wikipedia.org/wiki/Neuro-symbolic_AI](https://en.wikipedia.org/wiki/Neuro-symbolic_AI)).

**[measured]** On the neural-boundary-engineering half: Lean4Agent uses Lean4
dependent types to verify agent workflow structure before execution
([arXiv:2606.06523](https://arxiv.org/abs/2606.06523), reporting ~12% better
SWE-Bench-Verified outcomes for verified vs. unverified workflows) — a 2025–2026
research direction closer to horismos's ontology-derived schema + hard gate + bounded
retry + mandatory fallback than to anything MYCIN, Arden, or any current clinical AI
product or agent framework ships today. Related hard-gate precedents outside clinical
AI: AlphaGeometry (LLM proposes, symbolic deduction engine verifies,
[Nature 2024](https://www.nature.com/articles/s41586-023-06747-5)), Logic-LM (LLM→FOL,
Z3/Prover9 decides, [arXiv:2305.12295](https://arxiv.org/abs/2305.12295)), SATLM
(NL→SAT, Z3 solves,
[NeurIPS 2023](https://proceedings.neurips.cc/paper_files/paper/2023/file/8e9c7d4a48bdac81a58f983a64aaf42b-Paper-Conference.pdf)),
VeriPlan (LLM planner → formal property translation → model checker,
[CHI 2025](https://doi.org/10.1145/3706598.3714113)), and Sigloch & Benzmüller's
formal+embedding hybrid gate reporting 83%/72% hallucination detection in the
medical-device domain ([arXiv:2605.26942](https://arxiv.org/abs/2605.26942)).

**[inference]** The more defensible claim, and a more differentiated one: name both
lineages explicitly — formally-verified, expert-system-style symbolic reasoning,
gated at its only two neural boundaries by a mechanism closer to where formal
agent-verification research is heading than to anything currently shipping. Notably,
horismos's own stated future research direction — "provenance semirings... a nod to
Scallop-style differentiable provenance" — is, concretely, "become more like Scallop."
That would be a real architectural fork away from the current hard-separation design,
not an incremental add-on, and is worth naming explicitly the next time it comes up in
a roadmap conversation.

## Source

See `research/reading-log.md`, Q3 section.

## Confidence

mixed — lineage/precedent facts are [measured]; the positioning recommendation is
[inference].

## Last-verified

2026-08-24
