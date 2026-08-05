# Batch 0 — U1: rule model + load-time parse

## Task text (from the run ledger's derived unit plan, tracking the PRP Scope)

> U1. Rule model + load-time parse: `reason::rule::{Rule, ConditionTree, Condition,
> Conclusion}`; parsed from ontology documents at `Ontology::assemble`; load-time
> validation in domain terms (undeclared property/class in conditions or conclusion,
> invalid operator arity/type, rule-side `horis:unit` mismatch — PRP-001 D6
> follow-through); `Ontology::rules()` accessor.

PRP Scope bullet this serves: "Forward chaining: rules fire over the graph …" —
rules must first exist as typed data. Doc 02 § Rule vocabulary is the grammar;
doc 04 fixes the semantics the model must be able to express.

## Governing decisions

- PRP-001 D6 (units, strict): "units are opaque labels, exact match required between
  rule conditions and property declarations; mismatch is a load-time verification
  error in domain terms." The PRP-001 gap note hands the *rule-side* check to
  PRP-002: a condition may state `horis:unit`; if stated it must equal the
  property's declared unit exactly; a condition on a property with a declared unit
  and no unit in the condition inherits the declaration (no error).
- Controller pin: rules parse at `Ontology::assemble`; a rule error is a load error.
- Controller pin: v1 rules bind one subject class; mixed-subject rules are a
  load-time domain-term error.
- No new vocabulary terms: only doc-02's 11 operators, 3 combinators,
  `horis:Rule/priority/when/then/explain/subject/property/value/unit`.

## Required reading

- `docs/framework/02-ontology.md` § Rule vocabulary (the 11 operators + 3 combinators)
- `docs/framework/04-reasoning.md` (semantics the model must carry: priority,
  explain string, conclusion property+value)
- `crates/horismos-core/src/ontology/schema.rs` (RawTriples pattern, rdf list
  walking, error style), `turtle.rs` (`Document`, `Object`, `Literal`, `Node`)
- `.claude/skills/ontology-modeling-conventions`

## Requirements

1. `reason::rule` module: `Rule { iri, priority: i64 (default 0), when: ConditionTree,
   then: Conclusion, explain: Option<String>, subject_class: Iri }`,
   `ConditionTree { All(Vec<..>) | Any(Vec<..>) | None(Vec<..>) | Leaf(Condition) }`,
   `Condition { property: Iri, op: Op, unit: Option<String> }`,
   `Op` = 11 variants: Exceeds(Value) Below(Value) AtLeast(Value) AtMost(Value)
   Between(Value, Value) Is(TermOrValue) IsNot(TermOrValue) IsAssigned IsEmpty
   Contains(Value) InList(Vec<TermOrValue>), `Conclusion { property: Iri,
   value: TermOrValue }`.
2. Parse from the ontology's `Document`s (all graphs) every subject with
   `a horis:Rule`. Blank-node condition/conclusion bodies; `horis:all/any/none`
   over RDF lists; nested combinators allowed.
3. Load-time domain-term errors (new `OntologyError` variants), each naming the rule:
   - condition/conclusion property not declared → "rule :r9 names pulse, which is
     not a declared property in the ontology"
   - `horis:subject` names an undeclared class
   - conditions naming two different subject classes
   - missing `horis:when`/`horis:then`
   - `horis:between` without exactly two values; operator/value type mismatches are
     NOT checked here (evaluation semantics + PRP-005 verification own that) except
     unit: rule-side `horis:unit` ≠ declared unit → "rule :rX heartRate condition
     states unit \"mmHg\" but heartRate is declared in \"bpm\"" (exact strings in
     tests may differ; tests assert the load fails and the text names rule,
     property, both units).
4. `Ontology::rules()` → `&[Rule]`, sorted (priority desc, rule IRI lex) at build.
5. `triage.ttl` loads with exactly one rule, r1: priority 10, 2-condition All,
   conclusion acuity=:Immediate, explain string intact.
6. TDD: RED = behavioural failures (tests naming the missing module/API compile
   failure is acceptable for the very first test; the validation tests must show
   behavioural RED once the type exists).

## Out of scope

Condition *evaluation* (U2), matching (U3/U8), firing (U4), proof trees (U5/U6),
traces (U7), workflow vocabulary (PRP-004). No `.ttl` fixture changes.

## Carried-forward concerns

None (first batch).
