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
  every `horis:requires` satisfiable given the rules; every PerceptionStep has a
  fallback edge.
- **Witnesses**: BDD valuations decoded into domain terms — "no acuity assigned when
  heartRate ∈ (100, 120] and no Symptom present" — rendered for SMEs, machine-readable
  for the authoring agent (PRP-007 consumes these).
- **Surface**: `onto.verify()` in both APIs; `report.complete / .conflicts / .witnesses`;
  CI-friendly exit codes.

## Assumption manifest (added 2026-08-24, bounded-projection pivot)

Captain-adopted (`horismos-thin-pivot-s1/report.md` §"Framework doc-by-doc change map" row
10; `horismos-research-s1/report.md` §1.7). A verification report must declare its own
scope, not imply a global guarantee it cannot make. Every `VerificationReport` states an
explicit **assumption manifest** alongside `report.ok`:

- **Input closure**: exactly which facts/resources this verification treated as the complete
  input space (doc 03's declared closure, PRP-008) — verification proves totality/soundness
  *over that declared closure*, never over an unbounded or implicit "everything."
- **External profile/version**: which external vocabulary/profile versions (SNOMED edition,
  FHIR profile, PlanDefinition version) the checked rules/mappings assume.
- **Mapping version**: which standard-vocabulary mapping version anchored terms used in
  discretization.
- **Supported subset**: which subset of any imported artifact (e.g. a PlanDefinition) was
  formalized and checked vs. left prose-only or out of scope.
- **Unverified external effects**: which tool/adapter effects are declared-shape-checked
  only (contract compliance) with no claim about the real-world correctness of the
  implementation behind them.

`report.ok` must never be read as "the domain is correct" or "the external data is
complete" — those claims are explicitly out of scope and the manifest exists so a reader
never has to infer that from silence. This does not change the four rule-set checks or the
workflow checks below; it makes their scope machine-readable and SME-readable at once.

## Non-vacuity checks (added 2026-08-24)

Captain-adopted per `horismos-research-s1/report.md` §1.7 (the LASA/CHARGE "vacuous
verification" failure mode — a rule or constraint can be internally consistent, BDD-
verifiable, and bind nothing). Add a verification pass **distinct from** consistency-
checking:

- **Rule non-vacuity**: for each rule, does its `horis:when` condition set actually hold
  under some reachable input region (i.e. is the rule ever a candidate)? A rule whose
  conditions are unsatisfiable is already caught by dead-rule detection (above); non-vacuity
  additionally flags rules whose conditions *are* satisfiable but whose conclusion never
  differs from what a higher-priority rule or default already assigns — internally
  consistent, verifiable, and inert.
- **Constraint non-vacuity**: for each declared constraint (`horis:min/max`, `horis:oneOf`,
  `horis:pattern`, …), does it actually exclude some region of the otherwise-typed value
  space? A constraint that is vacuously satisfied by every value it could ever see (e.g. a
  `horis:max` set above any value the property's own type/domain could produce) is flagged,
  not silently accepted as "passing."
- Witnesses render the same way as completeness/conflict witnesses (domain-term sentence +
  machine-readable region) so non-vacuity findings are first-class in `report.witnesses`,
  not a separate, easy-to-ignore channel.

This does not certify that a rule set means what the SME intended (Lahiri 2024's structural
ceiling: no algorithmic guarantee a formal spec captures intent) — it only closes the gap
where "verifiable" was silently read as "does something."

## Non-goals

- No probabilistic verification, no verification of LLM behavior (out of scope by
  definition — the LLM is behind validated boundaries).
- No SMT solver dependency (the Z3 rejection from sigil-lang ADRs stands: BDD is pure
  Rust and WASM-safe).
- No claim that verification proves external record completeness, terminology-server
  correctness, or tool/host implementation behavior — the assumption manifest above states
  this explicitly rather than leaving it to be inferred (see also doc 10 §"What
  verification does not claim").

## Key decisions to make

1. Verification at load: mandatory-with-override vs. opt-in (`assert_complete()` style).
   Lean: always run, fail deploys by default, explicit `allow_incomplete` escape.
2. Incremental re-verification on ontology edit (authoring loop performance target:
   interactive, <1s for triage-sized ontologies).
3. Scope of numeric discretization with units (`horis:unit` interplay).
4. **[DECIDE, PRP-008 coordination]** Exact machine-readable shape of the assumption
   manifest (a `report.assumptions` struct vs. reusing `report.witnesses` with a new
   category) — resolved when PRP-008's `ExternalSnapshot`/input-closure types exist to
   reference; the manifest's five required fields above are fixed regardless of shape.

## Definition of done

The triage ontology verifies with a seeded completeness gap that produces the exact
witness above; workflow reachability checks pass/fail on crafted fixtures; verification
runs in both Python and Rust APIs and in CI; interactive-speed benchmark met; a seeded
vacuous rule (satisfiable conditions, inert conclusion) and a seeded vacuous constraint are
each caught by the non-vacuity pass with a domain-term witness; every `VerificationReport`
carries the five-field assumption manifest, populated (closure/versions declared) or
explicitly empty (standalone ontology with no external hydration), never silently absent.
