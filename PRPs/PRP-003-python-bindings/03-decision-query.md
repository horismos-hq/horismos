# D2 — Query surface: decision record

**Decided 2026-08-18**: v1 ships a **class-scoped conjunctive pattern
query**, evaluated in core (`World::query_pattern`, the B6-mandated named
core unit); the **textual SPARQL subset is deferred beyond v1** with the
dated gap note in `02-tasks-and-gaps.md`. Owns and strikes doc 03's
`[DECIDE: exact subset grammar — PRP-003]`.

## The question, falsifiably

"The triage quickstart (and PRP-004's runtime on top of it) needs a
textual SPARQL SELECT/ASK evaluator in v1" — falsified: the quickstart
needs exactly one query shape, *instances of a class filtered by
property-value equalities* ("patients whose acuity is Immediate"), and
doc 05's workflow goals/conditions consume the rule-condition evaluator
(PRP-002), not SPARQL. Nothing shipped or scheduled for v1 consumes a
SPARQL string.

## Options weighed

1. **SPARQL SELECT/ASK subset (string) in core** — a grammar, parser,
   binding-set evaluator, and its error surface, for zero v1 consumers.
   Highest blow-up risk in this PRP; rejected for v1.
2. **Pythonic pattern query compiling to a core evaluator** — one core
   method (subsumption-aware class scan + typed value-equality filters,
   deterministic order), marshaled like every other call. Ships.
3. **No query surface** — falsifies doc 11's `World.query` row and leaves
   the doc-03 marker unresolved. Rejected.

A textual SPARQL front-end, if ever wanted, *compiles to* the shipped
pattern tier — the deferral names its target rather than an unowned wish.

## The exact shipped grammar (what the doc-03 marker asked for)

`query(class, {property: value, ...})` — conjunction only:

- `class`: subsumption-aware (the `subClassOf` closure, anchored imports
  included), exactly like `instances()`.
- each `(property, value)`: satisfied when the instance holds a value for
  `property` equal to `value` (any value of a multi-valued property);
  equality is core's typed `Value` equality (node refs compare by IRI).
- result: the matching `Ref`s in the store's deterministic order.
- no disjunction, no negation, no joins across subjects, no ORDER
  BY/LIMIT, no variables — one subject class per query, by design.

Everything outside that grammar is the deferral.
