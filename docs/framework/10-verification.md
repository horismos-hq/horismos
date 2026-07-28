# 10 · Verification

Verification is what separates "framework with tests" from "framework with
guarantees": before an ontology runs, Sigil proves properties of its rule set and
workflows, and renders violations as counterexamples an SME can read. The engine is
a port-and-adapt of sigil-lang's BDD verifier (`biodivine-lib-bdd`; pure Rust,
WASM-safe, 50 tests of prior art; the Z3 rejection stands — no 200MB C++ dependency).

## When it runs

- **Load time**: `Ontology.load` runs verification by default; failures block
  deployment unless explicitly overridden (`allow_incomplete=True` — annotated in
  every subsequent trace). **[DECIDE: confirm default-hard policy — PRP-005]**
- **Authoring time**: every draft the authoring agent produces (doc 13) — target
  <1s interactive on triage-sized ontologies.
- **CI**: `sigil verify path/` exit codes; JSON output for tooling.

## Rule-set checks

For each rules-assigned property (e.g. `:acuity`), conditions are discretized into
threshold predicates and encoded as BDDs. The ontology makes the encoding *tighter*
than sigil-lang's: `sigil:min/max` bound variable domains, `sigil:oneOf` closes value
sets, and the class hierarchy widens rule applicability through subsumption
(a rule on `:ChestPain` covers the `:CrushingChestPain` region).

| Check | Question | Counterexample rendering |
|-------|----------|--------------------------|
| **Completeness** | Does some rule assign the property for every reachable input region? | "No acuity assigned when heartRate ∈ (100, 120] and no Symptom present" |
| **Conflict-freedom** | Can two rules derive different values for a single-cardinality property on an overlapping region? | "r1 → :Immediate but r4 → :Urgent when heartRate ∈ (120, 140] ∧ ChestPain — r1 wins by priority; confirm intended" |
| **Dead rules** | Is any rule's condition unsatisfiable given constraints and higher-priority shadowing? | "r7 can never fire: requires heartRate > 350, but sigil:max is 300" |
| **Termination** | Can chaining cycle (rule A derives what re-enables rule B and vice versa)? | The cycle, listed rule by rule |

Completeness is relative to declared reachability: constraints define the input space,
`sigil:assignedBy` defines which properties are inputs vs. derived.

## Workflow checks (new in Sigil, beyond the sigil-lang port)

| Check | Question |
|-------|----------|
| **Reachability** | Is every step reachable from `sigil:startsAt`? |
| **Termination** | Does every path reach a `sigil:terminal` step? (No implicit dead-ends.) |
| **Fallback totality** | Does every fallible step declare its failure edge? (Perception: `fallback`; Reasoning: `onUnreachable`; Action: `onError`.) |
| **Precondition satisfiability** | Can each `sigil:requires` ever hold, given the rules that could establish it? (An ActionStep requiring `acuity isAssigned` is checked against the rule set's completeness for `:acuity`.) |
| **Choice totality** | Do ChoiceStep branches + `otherwise` cover all cases? |

The precondition check is the composition point: workflow verification *uses* the
rule-set BDDs. A workflow is proven to never stall only because the rules are proven
to always assign what its steps require.

## The report

```python
report = onto.verify()
report.ok                     # bool: all checks pass
report.complete               # per-property completeness
report.conflicts              # list[Conflict] with regions + priority resolution
report.dead_rules             # list[DeadRule]
report.witnesses              # every finding as a Witness:
                              #   .domain_text  — the SME-readable sentence
                              #   .region       — machine-readable bounds
                              #   .suggest      — repair hint where derivable
report.to_json()
```

Witnesses are the shared currency of three consumers: humans (rendered text), the
authoring agent (structured region → generates a repair draft), and CI (JSON diff —
"this PR introduces a new completeness gap").

## What verification does not claim

- Nothing about LLM behavior — by construction, perception is behind the gate and
  the fallback edge; verification proves the *symbolic* system is total and sound
  around whatever perception does.
- Nothing about tool implementations — only that their declared contracts are
  respected at the boundaries.
- Nothing probabilistic. Properties are proven over the discretized symbolic space,
  exactly, or reported with a witness. There are no "95% verified" states.
