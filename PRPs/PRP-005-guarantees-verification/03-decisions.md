# PRP-005 — Verification decisions (2026-09-02)

This record is the evidence trail for D1–D4. The concise resolutions and fallbacks are
in `02-tasks-and-gaps.md`.

## Evidence sweep

- Docs 02 and 04 already make units opaque exact-match labels and reject mismatches at
  load. Doc 10 already leans to default-hard load verification. Doc 14 budgets full
  load plus verification at under one second and defines `Ontology` as immutable.
- The prior-art sibling `../sigil-lang` was unavailable in this worktree. No option is
  justified by claims about uninspected source.
- `biodivine-lib-bdd`'s official Rustdoc documents owned, thread-shareable BDD values,
  deterministic Boolean operations, and first/last satisfying valuations:
  <https://docs.rs/biodivine-lib-bdd/latest/biodivine_lib_bdd/>. Its upstream repository
  describes the same owned representation and notes that tiny repeated modifications
  are not its strongest performance case:
  <https://github.com/sybila/biodivine-lib-bdd>.
- PRP-008's `ExternalSnapshot` types do not exist. The task fixes the manifest's five
  semantic fields now and explicitly requires a dedicated provisional struct.

## D1 — default-hard verification

Falsifiable statement: an ordinary ontology load always verifies and cannot return an
executable incomplete ontology unless its caller explicitly requests
`allow_incomplete`.

Blast radius: core ontology/error/verify, PyO3/Python load signatures, every fixture
load, docs 10–12, U5–U7, and PRP-008's package admission. Gates: no-model and wasm.

Options: (A) default hard with override; (B) opt-in `verify()`; (C) parse now and fail
only when a World/Agent is constructed. A makes the documented guarantee structural
and still supports authoring. B permits accidental deployment of unchecked rules. C
duplicates state checks at every consumer and allows misleading ontology introspection.

Adversarial pass: A makes deliberately broken fixtures more explicit and costs a full
verify per load; the authoring override and <1s budget bound both costs. B/C violate the
load-time contract. Ranked criteria: house guarantee > no bypass > authoring usability >
cost. Resolution: A.

## D2 — edit-time re-verification

Falsifiable statement: full verification of each newly loaded immutable ontology is
interactive at triage scale, so v1 needs no incremental invalidation graph.

Blast radius: verify compilation/cache design, authoring PRP-007, benchmark U7; no
Python or gate semantics change.

Options: (A) full recomputation; (B) cache compiled BDDs per immutable ontology; (C)
introduce a rule/property dependency graph for incremental edits. A has the smallest
state surface and identical behavior on every call. B helps repeated `verify()` but
adds cache synchronization. C benefits an edit API that does not exist and risks stale
findings.

Adversarial pass: A could miss the time target; the mandatory benchmark is its kill
criterion. B/C add state and invalidation correctness risks. Ranked criteria: identical
deterministic reports > under-one-second target > downstream flexibility > complexity.
Resolution: A, with dependency-indexed recompilation as the recorded fallback.

## D3 — numeric units

Falsifiable statement: verification partitions numeric values only in the property's
declared opaque unit; it performs no conversion.

Blast radius: discretization and witness rendering (U2), ontology load behavior already
implemented, docs 02/04; future mapping work belongs to PRP-008.

Options: (A) exact-match opaque labels; (B) a built-in unit conversion table; (C)
dimensioned quantities and conversion vocabulary. A exactly preserves shipped rule
semantics. B silently invents conversions without provenance. C needs vocabulary and a
far larger type system.

Adversarial pass: A makes authors normalize units before load, which is already the
documented boundary. B/C introduce hidden floating conversion and vocabulary outside
this PRP. Ranked criteria: shipped semantics > determinism > witness clarity >
ergonomics. Resolution: A.

## D4 — assumption manifest

Falsifiable statement: a dedicated five-field struct is clearer and harder to omit than
encoding assumptions as witness categories.

Blast radius: report/JSON/Python types (U1/U6) and PRP-008's future adapter types.

Options: (A) dedicated provisional struct; (B) witness categories; (C) defer the
machine shape. A makes every report structurally complete. B conflates verification
scope with failures and disappears when there are no witnesses. C violates the pivot's
acceptance criterion.

Adversarial pass: A's string/list types will evolve once PRP-008 lands, but the field
names and semantics remain fixed; this is cheaper than a false stable dependency on
types that do not exist. Ranked criteria: never silently absent > exact five-field
contract > PRP-008 migration path > short-term convenience. Resolution: A.
