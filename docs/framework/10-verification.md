# 10 · Verification

Verification is what separates "framework with tests" from "framework with
guarantees": before an ontology runs, Horismos proves properties of its rule set and
workflows, and renders violations as counterexamples an SME can read. The engine adapts
the sigil-lang BDD-verifier design described by PRP-005 using
`biodivine-lib-bdd`: the implementation is pure Rust and WASM-safe, and deliberately
has no SMT/Z3 dependency.

## When it runs

- **Load time**: `Ontology.load` runs verification by default; failures block
  deployment unless explicitly overridden (`allow_incomplete=True` — annotated in
  every subsequent trace). The default-hard policy is fixed; `allow_incomplete` is
  the explicit authoring/debug escape (decided 2026-09-02, PRP-005 D1).
- **Authoring time**: every draft the authoring agent produces (doc 13) — target
  <1s interactive on triage-sized ontologies.
- **CI**: `python -m horismos verify path/to/ontology.ttl` emits deterministic JSON
  and exits `0` only when every check passes. Repeat `--import path` for explicit
  undeclared imports.

## Rule-set checks

For each rules-assigned property (e.g. `:acuity`), conditions are discretized into
threshold predicates and encoded as BDDs. The ontology makes the encoding *tighter*
than sigil-lang's: `horis:min/max` bound variable domains, `horis:oneOf` closes value
sets, and the class hierarchy widens rule applicability through subsumption
(a rule on `:ChestPain` covers the `:CrushingChestPain` region).

Every authored constant or ordered boundary in rule conditions, workflow reasoning
goals, action `horis:requires`, and choice branches participates in that projection.
A value used only by a workflow is therefore still represented exactly rather than
being mistaken for an impossible input. Ordered comparisons follow the runtime's
`compare`: numbers cross-type, `xsd:dateTime` by instant, and strings
lexicographically, each in its own opaque order with no conversion; a comparison the
runtime cannot order (a boolean, a duration, a mismatched datatype) never holds.
`xsd:boolean` properties are closed over `true`/`false`, so an `is true` / `is false`
pair is a complete partition. A rule may conclude on a property perception also sets;
downstream conditions then see the input value *or* any standing rule conclusion.

| Check | Question | Counterexample rendering |
|-------|----------|--------------------------|
| **Completeness** | Does some rule assign the property for every reachable input region? | "No acuity assigned when heartRate ∈ (100, 120] and no Symptom present" |
| **Conflict-freedom** | Can two rules derive different values for a single-cardinality property on an overlapping region? | "r1 → :Immediate but r4 → :Urgent when heartRate ∈ (120, 140] ∧ ChestPain — r1 wins by priority; confirm intended" |
| **Dead rules** | Is any rule's condition unsatisfiable given constraints and higher-priority shadowing? | "r7 can never fire: requires heartRate > 350, but horis:max is 300" |
| **Termination** | Can chaining cycle (rule A derives what re-enables rule B and vice versa)? | The cycle, listed rule by rule |

Completeness is relative to declared reachability: constraints define the input space,
`horis:assignedBy` defines which properties are inputs vs. derived.

## Workflow checks (new in Horismos, beyond the sigil-lang port)

| Check | Question |
|-------|----------|
| **Reachability** | Is every step reachable from `horis:startsAt`? |
| **Termination** | Does every path reach a `horis:terminal` step? (No implicit dead-ends.) |
| **Fallback totality** | Does every fallible step declare its failure edge? (Perception: `fallback`; Reasoning: `onUnreachable`; Action: `onError`.) |
| **Precondition satisfiability** | Can each `horis:requires` ever hold, given the rules that could establish it? (An ActionStep requiring `acuity isAssigned` is checked against the rule set's completeness for `:acuity`.) |
| **Choice totality** | Do ChoiceStep branches + `otherwise` cover all cases? |

The precondition check is the composition point: workflow verification *uses* the
rule-set BDDs. Facts established by rules are computed with the runtime's exact
priority order and restart-to-highest-priority fixed point, including derived-from-derived
support. A workflow is proven to never stall only because those rules establish what
its steps require.

## The report

```python
report = onto.verify()
report.ok                     # bool: all checks pass
report.complete               # per-property completeness
report.conflicts              # list[Conflict]: both rules, values, priorities,
                              #   the region, and the priority `winner`
report.dead_rules             # tuple[str, ...] of rule CURIEs
report.terminating            # bool: rule chaining terminates
report.workflows              # per-workflow termination/requires/fallback/choice
                              #   totality; unreachable steps are global witnesses
report.witnesses              # every finding as a Witness:
                              #   .severity     — "error" fails the report,
                              #                   "info" is advisory only
                              #   .domain_text  — the SME-readable sentence
                              #   .region       — machine-readable bounds
                              #   .suggestion   — repair hint where derivable
report.assumptions            # always-present five-field manifest
report.exit_code              # 0 when ok, otherwise 1
report.to_json()
```

Witnesses are the shared currency of three consumers: humans (rendered text), the
authoring agent (structured region → generates a repair draft), and CI (JSON diff —
"this PR introduces a new completeness gap").

## The assumption manifest (added 2026-08-24, bounded-projection pivot)

Every `VerificationReport` carries an explicit `assumptions` manifest alongside
`report.ok`: `input_closure`, `external_profile_versions`, `mapping_version`,
`supported_subset`, and `unverified_external_effects`. Each field is populated or
explicitly empty, so scope is machine-readable rather than implied by silence. The
container is the provisional PRP-005 shape; PRP-008 may replace its value types with
`ExternalSnapshot`-native types, but the five fields are fixed.
Full spec: `PRPs/PRP-005-guarantees-verification/01-objective.md`
§"Assumption manifest". This section
exists precisely so the "does not claim" list below is enforced structurally, not just
stated in prose.

## Non-vacuity (added 2026-08-24)

A rule or constraint can be internally consistent and BDD-verifiable while binding
nothing — the LASA/CHARGE "vacuous verification" failure mode
(`horismos-research-s1/report.md` §1.7). The distinct non-vacuity pass flags a
satisfiable condition whose conclusion adds no new fact and a constraint that excludes
no value in the declared domain. Both findings carry a domain-term witness and a
machine-readable region. A bound that can never reject a value of the property's own
type — `horis:min`/`horis:max` on a non-numeric property, `horis:pattern` on a
non-string property, or a universal pattern — is an error. An explicit
`horis:cardinality "0..*"` is a documented authored value (doc 02) that restates the
default, so it is reported at `info` severity: the note appears in `report.witnesses`
but does not make `report.ok` false or block the default-hard load. Full spec:
`PRPs/PRP-005-guarantees-verification/01-objective.md`
§"Non-vacuity checks".

## Imported-artifact and capability checks (declared-future, PRP-008/PRP-007)

Where an ontology overlay imports a standard artifact (a PlanDefinition subset) or
compiles a capability fragment from an existing tool/skill, verification extends to:
mapping loss/totality reporting (did the import lose information, and is that loss
bounded), supported-subset conformance (does the imported artifact stay within the
declared supported construct list, failing closed at load otherwise), and capability
composition checks (do two capability fragments' declared effects conflict). These checks
compose with, and do not replace, the rule-set and workflow checks above.

## What verification does not claim

- Nothing about LLM behavior — by construction, perception is behind the gate and
  the fallback edge; verification proves the *symbolic* system is total and sound
  around whatever perception does.
- Nothing about tool implementations — only that their declared contracts are
  respected at the boundaries.
- Nothing probabilistic. Properties are proven over the discretized symbolic space,
  exactly, or reported with a witness. There are no "95% verified" states.
- **Nothing about external record completeness or freshness** (added, pivot) — a
  verification pass is proven over the declared input closure in the assumption
  manifest, never over an unbounded or currently-live external record.
- **Nothing about terminology-server correctness** — an anchored mapping is checked for
  internal consistency, not for whether the terminology server's own content is correct.
- **Nothing about tool or host implementation behavior** beyond declared-contract
  compliance (doc 07's correction on contract narrowing applies here too) — and,
  declared-future, nothing about the semantic correctness of a candidate-artifact
  proposal (docs 06/07): the candidate-artifact boundary is a policy/sandbox gate, not a
  verification theorem.
