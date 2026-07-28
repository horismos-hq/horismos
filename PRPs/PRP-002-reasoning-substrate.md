# PRP-002 — Reasoning Substrate

**Status**: Draft
**Depends on**: PRP-001

## Intent

Deterministic inference over the instance graph: forward chaining to fixed point,
backward chaining with proof trees, subsumption, constraint checking — with traces
designed as a training corpus from day one.

## Scope

- **Subsumption**: class-hierarchy reasoning (`CrushingChestPain ⊑ ChestPain ⊑ Symptom`),
  including imported hierarchies; property domain/range inference.
- **Forward chaining**: rules fire over the graph to a fixed point; priority ordering,
  fired-rule tracking (no re-fire), iteration cap — port the semantics proven in
  sigil-lang's evaluator and Rust VM `rules.rs`; adapt the Rete-lite matcher to
  ontology instances (alpha nodes keyed on class + property predicates).
- **Backward chaining**: `prove(goal)` with cycle detection and depth cap; `explain(fact)`
  returns the proof tree — port sigil-lang `backward.py` semantics to Rust.
- **Constraint checking** as continuous invariant: violations are impossible by
  construction (gate) or surfaced as diagnostics when rules would derive them.
- **Traces** (the flywheel obligation): every inference step recorded in a stable,
  versioned schema — typed steps, fired rules, proof trees, timing; `loss_mask` and
  outcome-label fields reserved per the TIR/CIR literature (see sigil-lang
  `docs/research/INTERLEAVED_SYMBOLIC_EXECUTION.md`). JSON export, compact and pretty.

## Non-goals

- No workflow execution, no LLM calls, no BDD verification (PRP-004/005).
- No probabilistic/fuzzy inference — determinism is the product.

## Key decisions to make

1. Rete-lite vs. naive match-loop first (sigil-lang data: marshaling dominated at small
   scale; here everything is in-process Rust, so Rete pays off earlier — benchmark).
2. Proof-tree granularity: per-condition (sigil-lang precedent) vs. per-rule.
3. Trace schema versioning policy (traces outlive framework versions once used as
   training data).
4. Negation semantics: `isNot`/absence under open-world RDF vs. Sigil's pragmatic
   closed-world `_UNBOUND` precedent — pick, document, test (sigil-lang's answer:
   comparisons with unbound are false, `is not` true; re-validate for the graph world).

## Definition of done

Triage scenario runs end-to-end symbolically: structured facts in → subsumption +
forward chaining → `acuity` derived → `explain` renders the r1 proof; traces serialize
to the versioned schema; parity tests against hand-computed fixed points; criterion
benchmarks vs. naive matching.
