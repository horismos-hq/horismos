# Implementer report — U1 (rule model), U2 (evaluation), U3 (naive matcher)

Commits: `68c5fb4` (U1), `b3c98b3` (U2), `0471656` (U3). `make verify` green at
each head (U3's head re-verified after a fmt amend; the intermediate U3 commit
briefly carried fmt drift, folded into the amend).

## What was built

- `reason::rule` — typed `Rule`/`ConditionTree`/`Condition`/`Op`/`Conclusion`;
  parsed at `Ontology::assemble` from all graphs; load-time domain-term errors
  (undeclared property/class, mixed subjects, missing parts, operator arity,
  multiple operators, priority type, rule-side unit checks per PRP-001 D6);
  `Ontology::rules()` pre-sorted (priority desc, IRI lex on resolved strings).
  19 tests incl. triage.ttl r1 shape parity.
- `Schema::class_declared` + `Schema::canonical_iri` — the D-7 sameAs-canonical
  class/individual lookups.
- `reason::eval` — the 11 operators + 3 combinators over `&World`; doc 04 edge
  table row-by-row tests; D-4 resolved as ∀-isNot (markers struck in docs
  02/04 in the U2 commit); property test pins isNot ≡ ¬is, isEmpty ≡
  ¬isAssigned. 14 tests.
- `reason::matcher` — `Matcher` trait, `FiredSet`, `Candidate`, `NaiveMatcher`
  (the oracle). Deterministic order: rules pre-sorted, subjects in first-intern
  token order. 4 tests incl. Example-1 pass-1 parity.

## TDD evidence

U1's RED was compile-level (`Ontology::rules()` and `reason::rule` did not
exist — the 19 tests could not build). U2/U3 tests were written against the
already-shaped modules and passed on first full run; the semantics they pin
were hand-derived from the PRP's worked examples before implementation
(Example 1 pass 1 candidates, doc-04 table rows). Weakness acknowledged: no
captured behavioural-RED run for U2/U3.

## Named risks (check these specifically)

1. **`ref_matches` shape coverage** (`eval.rs`): value-as-class vs
   value-as-instance vs enum-member identity — is the sameAs-canonical
   fallback for individuals correct, and can it accidentally make two
   *unrelated* IRIs equal? (`canonical_iri` returns `iri` when never in a
   sameAs triple, so equality should require an actual anchor.)
2. **Mixed-subject rejection** uses `class_declared`-based canonical equality
   (`canonically_equal` in rule.rs) — check it can't wrongly reject a rule
   whose conditions name the same class through an anchor twin.
3. **Operator/list parsing** in `rule.rs` walks blank-node lists by
   first/rest maps; malformed lists (from hand-written Turtle) break the walk
   silently (walk stops) — is any silent truncation observable as wrong rule
   semantics rather than an error? (Profile validation U3 owns list
   well-formedness; confirm that assumption holds for condition lists.)
4. **`contains` semantics** — substring for string/string, else `is`-match;
   this is an interpretation (documented in eval.rs) not spelled out in doc
   04. Flag if you think it needs a doc edit or a [DECIDE].
5. **Priority is `i64` with default 0** — negative priorities are legal by
   construction; nothing in the docs forbids them. OK?

## Out of scope (deliberately)

Forward chaining/transactions, proofs, traces, Rete — later units.
