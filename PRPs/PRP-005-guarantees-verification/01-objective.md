# PRP-005 — Guarantees & Verification

**Status**: Active (2026-09-02)
**Depends on**: PRP-001, PRP-002 (usable without PRP-004)

## Intent

Load-time formal verification of rule sets and workflows with human-readable
counterexamples — the feature that separates "framework with guarantees" from
"framework with tests." This is a port-and-adapt of sigil-lang's BDD-verifier
semantics to Horismos's ontology model. The sibling `../sigil-lang` checkout was not
available in the activation worktree, so implementation claims are grounded in this
PRP and the framework contracts rather than source-level prior-art comparisons.

## Background

A binary decision diagram (BDD) represents a Boolean function canonically for a fixed
variable order. Horismos turns each constrained input property into a finite symbolic
partition: numeric thresholds become intervals, closed enumerations become value atoms,
and class-valued conditions include subclasses. A rule's conditions then denote a BDD
region. Set operations answer the useful questions exactly: uncovered input is
`reachable AND NOT(any rule)`, and a conflict is the intersection of two rules that
assign different values.

The guarantee is deliberately bounded. Verification ranges only over the declared
input closure and external assumptions carried by the report; it says nothing about an
unfetched record, a live external server, or whether a formalized policy matches SME
intent. See docs 03, 04, and 10.

Python receives one whole `VerificationReport` from Rust. It does not reproduce BDD or
workflow logic in the binding layer.

## Scope

- Four rule-set checks: completeness, conflict-freedom, dead-rule detection, and
  chaining-cycle detection.
- Ontology-aware discretization: property constraints bound domains, class hierarchy
  widens applicability, and units are exact labels with no conversion.
- Workflow reachability, termination, `horis:requires` satisfiability, fallbacks, and
  choice totality.
- Domain-term witnesses with machine-readable regions.
- Rust and Python `Ontology.verify()` surfaces, default-hard load verification with an
  explicit `allow_incomplete` escape, and CI-friendly exit codes.
- A five-field assumption manifest on every report.
- Rule and constraint non-vacuity checks as first-class witnesses.

## Assumption manifest

Every `VerificationReport` carries `report.assumptions` with exactly five fields:
input closure, external profile/version, mapping version, supported subset, and
unverified external effects. PRP-005 uses the provisional core-native shape decided in
D4; PRP-008 may replace field value types with `ExternalSnapshot`-linked types without
removing or weakening any field.

`report.ok` means only that the rule and workflow checks hold under those assumptions.
It does not claim domain correctness, external completeness, terminology-server
correctness, or real-world tool behavior.

## Non-vacuity

- A satisfiable rule is vacuous when every region where it could fire is already
  assigned by a higher-priority rule, so its conclusion is inert.
- A declared constraint is vacuous when it excludes no value or cardinality from the
  otherwise-typed space.
- Both findings use ordinary domain-term witnesses and machine-readable regions.

## Non-goals

- No probabilistic verification and no verification of LLM behavior.
- No SMT solver dependency. The implementation uses `biodivine-lib-bdd`, a pure-Rust
  BDD library, behind the existing `verify` feature.
- No claim about external record completeness, terminology correctness, or host/tool
  implementation behavior.
- No incremental edit graph in v1; a newly loaded immutable ontology is verified as a
  whole within the interactive budget (D2).
- No unit conversion or dimensional analysis (D3).

## Worked triage example

The triage rule partition uses the authored thresholds `100` and `120`, the
`heartRate` bounds `[0, 300]`, and the `Symptom` class hierarchy. Removing the rule
covering moderate heart rate with no symptom leaves one symbolic region:

> no acuity assigned when heartRate ∈ (100, 120] and no Symptom present

The witness also carries that interval and absence predicate structurally, so an SME
sees the same finding the authoring agent consumes.

## Definition of done

- The triage ontology verifies with a seeded completeness gap producing the exact
  witness above (domain terms plus a machine-readable region).
- Workflow reachability, termination, requires, and fallback checks pass and fail on
  crafted fixtures.
- `onto.verify()` works in Rust and Python with CI-friendly exit codes; stubs match.
- A benchmark proves load plus verification is under one second at triage scale.
- A satisfiable-but-inert rule and a vacuous constraint are each caught with a
  domain-term witness.
- Every report carries the explicit five-field assumption manifest.
- No SMT dependency; `make verify`, `make check-boundary`, and `make no-model-gate`
  are green.

## Glossary

- **BDD**: a canonical graph representation of a Boolean function under a fixed
  variable order.
- **Discretization**: replacing an infinite typed domain with finitely many regions
  that preserve every authored condition boundary.
- **Reachable region**: an input combination allowed by the schema, class hierarchy,
  and declared closure.
- **Witness**: a concrete symbolic region demonstrating why a check failed.
- **Vacuous**: internally consistent but excluding or changing nothing.
- **Assumption manifest**: the five explicit bounds under which `report.ok` applies.
