# Batch 1 — U2 condition evaluation + U3 naive matcher

## Task text (ledger unit plan)

> U2. Condition evaluation over `&World`: 11 operators + 3 combinators,
> closed-world edge semantics per doc 04's table with resolved ∀-`isNot` (D-4);
> subsumption-aware `is`/`isNot`/`inList` via `Schema::is_subclass_of`.
>
> U3. Naive matcher (the correctness oracle): candidates = all (rule, subject)
> pairs whose conditions hold, minus fired; ordered priority desc, then rule IRI
> lexicographic.

## Governing decisions

- D-4 (ledger): `isNot X` ≡ ¬(`is X`) — ∀ over multi-valued; unassigned → true.
  Strike doc 04's `[DECIDE]` marker and rewrite doc 02's re-validation note in
  the same commit. Pin with property test `is_not_is_the_negation_of_is`.
- Doc 04 edge table verbatim: unassigned + comparison → false; unassigned +
  `isAssigned` → false; multi-valued `is` → ∃.
- Controller pin: `Matcher` trait; naive returns the single next candidate per
  doc-04 order (priority desc, rule IRI lex, then subject token asc for
  multi-subject ties — token order = first-intern order, deterministic).

## Semantics detail (from doc 02/03/04 + gate precedent)

- `is X` on a `Ref` value r: true iff r ≡ X sameAs-canonically, or r is a
  declared class with r ⊑ X, or r names a world instance whose class ⊑ X —
  mirroring the gate's own range acceptance (class-as-value and
  instance-as-value both occur in PRP-001's tests).
- Numeric comparisons cross-type over Integer/Decimal; DateTime ordered;
  String lexicographic (sigil `compare_values` precedent); Duration/Boolean
  unordered → false.
- `between` is inclusive on both bounds (sigil `Between` precedent).
- `contains`: string value + string term → substring; otherwise ∃-membership
  with the same match relation as `is`.
- `inList`: ∃ value, ∃ listed term with the `is` match relation.
- Combinators: All ∧, Any ∨ (empty Any → false; empty All → true — standard
  identities), NoneOf ≡ ¬∨.

## Required reading

docs 02 § Rule vocabulary, 04 § Condition evaluation; `reason/rule.rs` (U1);
`world/mod.rs` queries; `.claude/skills/ontology-modeling-conventions`.

## Requirements

1. `reason::eval`: `holds(world, subject, &ConditionTree) -> bool` and
   `condition_holds(world, subject, &Condition) -> bool`; pure, `&World` only.
2. `reason::matcher`: `FiredSet`, `Candidate { rule: usize, subject: Ref }`,
   `trait Matcher { fn next_candidate(&mut self, world, rules, fired) -> Option<Candidate>; fn invalidate(&mut self, ...) }`
   (shape may be refined in U8 — naive must stay the oracle), `NaiveMatcher`.
3. Tests: r1 candidate cases from PRP Example 1 pass 1 (p7 matches r1 via
   subsumption; r3-style rule not a candidate while acuity unassigned);
   fired exclusion; deterministic ordering; doc-04 edge table row by row;
   proptest: `isNot ≡ ¬is`, `isEmpty ≡ ¬isAssigned` over arbitrary worlds.
4. TDD: behavioural RED runs recorded for the new modules (module stub with
   `todo!()` or missing fn compile error for the first test; assertion
   failures for semantics tests where feasible).
5. `make verify` green; separate commits U2 then U3.

## Out of scope

Firing/transactions (U4), proofs (U5), Rete (U8), trace (U7).

## Carried-forward concerns

- U1 note: `Op::Contains`/`InList` accept any `Term`; evaluation defines their
  runtime meaning — keep it exactly as specified above so PRP-005 can verify
  against the same table.
