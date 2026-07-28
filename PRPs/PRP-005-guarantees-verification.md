# PRP-005 — Guarantees & Verification

**Status**: Draft
**Depends on**: PRP-001, PRP-002 (usable without PRP-004)

## Intent

Load-time formal verification of rule sets and workflows with human-readable
counterexamples — the feature that separates "framework with guarantees" from
"framework with tests." Direct port-and-adapt of sigil-lang's proven BDD verifier
(`biodivine-lib-bdd`, 50 tests of prior art).

## Scope

- **Four rule-set checks** (ported semantics): completeness (every reachable input
  region assigns the target), conflict-freedom (same-target rules never derive
  contradictory values on overlapping regions), dead-rule detection, termination
  (chaining cycle detection).
- **Adaptation to ontology semantics** (the new work): discretization must respect the
  class hierarchy (subsumption widens rule applicability) and constraint shapes
  (min/max bound the variable domains — tighter BDDs than sigil-lang had).
- **Workflow checks** (new): every step reachable; every path reaches a terminal step;
  every `sigil:requires` satisfiable given the rules; every PerceptionStep has a
  fallback edge.
- **Witnesses**: BDD valuations decoded into domain terms — "no acuity assigned when
  heartRate ∈ (100, 120] and no Symptom present" — rendered for SMEs, machine-readable
  for the authoring agent (PRP-007 consumes these).
- **Surface**: `onto.verify()` in both APIs; `report.complete / .conflicts / .witnesses`;
  CI-friendly exit codes.

## Non-goals

- No probabilistic verification, no verification of LLM behavior (out of scope by
  definition — the LLM is behind validated boundaries).
- No SMT solver dependency (the Z3 rejection from sigil-lang ADRs stands: BDD is pure
  Rust and WASM-safe).

## Key decisions to make

1. Verification at load: mandatory-with-override vs. opt-in (`assert_complete()` style).
   Lean: always run, fail deploys by default, explicit `allow_incomplete` escape.
2. Incremental re-verification on ontology edit (authoring loop performance target:
   interactive, <1s for triage-sized ontologies).
3. Scope of numeric discretization with units (`sigil:unit` interplay).

## Definition of done

The triage ontology verifies with a seeded completeness gap that produces the exact
witness above; workflow reachability checks pass/fail on crafted fixtures; verification
runs in both Python and Rust APIs and in CI; interactive-speed benchmark met.
