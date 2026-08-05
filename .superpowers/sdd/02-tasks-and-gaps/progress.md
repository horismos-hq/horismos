# PRP-002 Reasoning Substrate — run ledger

- Plan: `PRPs/PRP-002-reasoning-substrate.md` (teaching-grade, single-file on main).
  PR #3 converts it to folder form (`PRPs/PRP-002-reasoning-substrate/02-tasks-and-gaps.md`);
  per the launch brief this run derives its own unit plan until that merges, then
  rebases and adopts the folder's ledger surface.
- Branch: `fm/horismos-prp002-impl-a1` (firstmate-managed disposable worktree —
  supersedes the `.claude/worktrees/prp-NNN-slug` convention for this run).
- Worktree: `/Users/billaram/.treehouse/horismos-86ae5f/3/horismos`
- Baseline: `eee0ddf` (main; `make verify` green checked at run start — see below).

## Unit plan (derived from the PRP's Scope + DoD; PRP-001 folder shape)

- U1. Rule model + load-time parse: `reason::rule::{Rule, ConditionTree, Condition,
  Conclusion}`; parsed from ontology documents at `Ontology::assemble`; load-time
  validation in domain terms (undeclared property/class in conditions or conclusion,
  invalid operator arity/type, rule-side `horis:unit` mismatch — PRP-001 D6
  follow-through); `Ontology::rules()` accessor.
- U2. Condition evaluation over `&World`: 11 operators + 3 combinators, closed-world
  edge semantics per doc 04's table with resolved ∀-`isNot` (D-4 below);
  subsumption-aware `is`/`isNot`/`inList` via `Schema::is_subclass_of`.
- U3. Naive matcher (the correctness oracle): candidates = all (rule, subject)
  pairs whose conditions hold, minus fired; ordered priority desc, then rule IRI
  lexicographic.
- U4. Forward chaining to fixed point inside the World transaction: one firing per
  pass, no re-fire, conflict suppression (higher priority wins), iteration cap 100,
  conclusions through the gate with `Provenance::Rule`, gate rejects surfaced as
  diagnostics (world stays valid, firing recorded), whole-transaction rollback on
  cap exceeded; per-firing proof + derivation records; stored fact provenance
  (in-memory side map — PRP-001 gap adoption, see rulings).
- U5. `ProofNode` + `render()` (doc 04 format) + `World::explain(subject, property)`
  from the recorded forward-chaining derivations.
- U6. Backward chaining `prove(world, goal)`: rule index (property → concluding
  rules), cycle detection via in-progress goal set, depth cap 50, hypothesis overlay
  (prove never mutates the world), subsumption leaves — `backward.py` semantics
  re-keyed to graph goals.
- U7. Trace schema `horismos.trace/1`: typed steps (`reasoning` now; `perception`/
  `action`/`narration` shapes reserved), fired/suppressed/diagnostics, `loss_mask`,
  outcome labels; serde JSON compact + pretty; byte-identical serialization pinned;
  written versioning policy + v1→v2 dry-run migration test (D-3).
- U8. Rete-lite matcher (alpha memories keyed class + property); differential
  property tests vs the naive oracle; criterion benchmarks (triage fixture + larger
  synthetic ontology); D-1 resolved from the numbers; loser stays in-tree as oracle.
- U9. Retraction rulings wired into the gate: derived-fact retraction forbidden
  (D-5), `horis:immutable` binds retraction (D-6), sameAs-canonical class lookup
  ruling (D-7); doc 02/03/04 [DECIDE] markers struck in the same commits.
- U10. DoD run: Examples 1–3 from the PRP as executable parity tests against
  hand-computed fixed points; trace equality assertions; proptest suite for
  inference invariants (fixed-point convergence, no re-fire, caps, determinism).

## Controller resolutions (interfaces pinned across units)

- The reasoner reads the world as `&World` only; every conclusion is asserted via
  `World::assert_instance_as(Provenance::Rule, ..)`-equivalent internal path — the
  gate is the only write path (no back door).
- Rules parse at `Ontology::assemble` into `Inner.rules: Vec<Rule>` — a rule error is
  a *load* error (same posture as `horis:pattern` compilation). `Ontology::rules()`
  returns `&[Rule]` sorted by (priority desc, IRI lex) at build time.
- v1 rule shape: every condition in a rule names the same `horis:subject` class; the
  conclusion applies to that bound instance. Mixed-subject rules are a load-time
  domain-term error (recorded as a gap, not silently supported wrong).
- Bindings render as `?<initial>` of the subject class local name (`?p` for
  `:Patient`) — matches doc 09's example JSON and the PRP's `{?p→:p7}`.
- `Matcher` is a trait with `NaiveMatcher` and `ReteLite` implementations; forward
  chaining is generic over it; the default is set by D-1's benchmark.
- Proof granularity is per-condition (D-2): one `ProofNode` child per condition,
  subsumption hops explicit (`via: subsumption(...)`).
- Timing fields in traces are `Option` and omitted from JSON when absent, so trace
  equality remains byte-exact in deterministic tests.

## Decisions (brainstorming protocol; to be transferred into the PRP folder's
## 02-tasks-and-gaps.md when PR #3 merges)

- [ ] D-1. **Matcher: build both, benchmark decides the default** (resolution *path*
  fixed 2026-08-05; checkbox closes when U8's numbers exist). Statement: "Forward
  chaining's default matcher at triage scale is chosen by criterion benchmark; naive
  and Rete-lite both ship, the loser as differential-testing oracle." Evidence: PRP
  Key decision 1 prescribes exactly this; doc 14 pins CI-tracked criterion benchmarks
  from PRP-002 onward; sigil-lang's marshaling-dominated result is void in-process.
  Fallback: if the benchmark is a wash at both scales, naive is the default (simpler,
  easier to audit) and Rete-lite stays behind the trait. RESULT: *pending U8* — to be
  recorded here with the measured numbers.
- [x] D-2. **Proof-tree granularity: per-condition** (decided 2026-08-05).
  Statement: "Every condition evaluation — including each subsumption hop — is its
  own `ProofNode`; a firing is the parent node." Evidence: doc 04's `ProofNode`
  literally shows per-condition children and explicit `via: subsumption(...)` nodes
  ("hierarchy steps are explicit proof nodes"); `backward.py` produces per-condition
  subgoals; the PRP says per-rule loses if downstream consumers use the detail —
  narration (doc 06) and the authoring loop (doc 13) both render the per-condition
  tree. Measured cost: *pending U10* — record the triage Example-1 compact trace size
  here; if it stays in the low KBs the per-rule size argument does not materialize.
  Fallback: if realistic traces blow up, collapse *rendering*, not the recorded
  structure.
- [x] D-3. **Trace schema versioning policy** (decided 2026-08-05). Statement:
  "`horismos.trace/1` never changes meaning. Additive-optional fields (serde
  `skip_serializing_if` + `default`) are non-breaking within v1; anything that
  changes the meaning or type of an existing field, adds a required field, adds a
  new `kind`/step variant consumers must understand, or repurposes
  `loss_mask`/outcome labels is breaking and mints `horismos.trace/2` plus a pure
  `convert_v1_to_v2` function in `trace::migrate`, exercised by a dry-run test.
  Reserved fields (`loss_mask`, `outcome.labels`) may gain new *keys* without a
  version bump; existing keys never change meaning — corpora already trained on
  stay valid (TIR/CIR: sigil-lang INTERLEAVED_SYMBOLIC_EXECUTION.md §Design
  Recommendations)." Training-export tag set (folded in): XML-style per-step-kind
  tags `<perceive>`/`<reason>`/`<act>`/`<narrate>` with `<observe>` for masked
  executor output — the NVIDIA `<llm-code>`/`<llm-code-output>` family, the largest
  existing TIR-corpus precedent (OpenMathInstruct-1/NeMo-Skills, structurally
  identical per the research doc's comparison table); recorded as the doc-09 tag
  set, consumed when the exporter lands (PRP-003+). Fallback: tags are exporter
  template data, not trace data — swapping them never touches recorded traces.
- [x] D-4. **Negation semantics: ∀ for multi-valued `isNot`** (decided 2026-08-05).
  Statement: "`horis:isNot X` holds iff *no* value of the property `is X`
  (subsumption-aware) — `isNot` ≡ ¬`is` in every world state." Evidence: the
  doc-04 edge table pins unassigned→`isNot` true; the ∃ reading contradicts it (an
  unassigned property has no witness value, so ∃ would yield false) — only ∀ keeps
  the documented table coherent; sigil-lang's `_UNBOUND` precedent ("comparisons
  false, `is not` true") is exactly the ¬ reading; `horis:none` (¬∨) then composes:
  `none(is X)` ≡ `isNot X` on a single condition. Pinned by property test
  `is_not_is_the_negation_of_is`. Doc 04 marker struck in the same commit (U2);
  doc 02's inherited-and-re-validated note rewritten.
- [x] D-5. **Retraction of rule-derived facts: forbid** (decided 2026-08-05).
  Statement: "A fact recorded as rule-derived cannot be retracted directly, by any
  provenance; the error names the deriving rule. Derived facts change only by
  re-running inference after base-fact changes." Evidence: doc 03's own lean
  ("lean forbid — derived facts are retracted only by re-running inference");
  cascade requires truth-maintenance machinery PRP-002's scope excludes; allowing
  silent retraction would let the world contradict its recorded proofs — trace and
  world would disagree, violating principle 2. Fallback: cascade can be added later
  behind the same gate check without changing the forbid default.
- [x] D-6. **`horis:immutable` binds retraction** (decided 2026-08-05). Statement:
  "Retracting a value of a `horis:immutable` property is refused at the gate, same
  witness shape as re-assertion; retract-then-assert can no longer replace an
  immutable value." Evidence: doc 02 defines immutable as "once set, cannot be
  re-asserted with a different value" — a two-step bypass makes the declaration
  vacuous; the PRP-001 gap note (2026-07-29) flagged exactly this hole and handed
  it here. Fallback: a future `horis:retractable` term could opt back in per
  property; none is needed now. Doc 02 marker struck in the same commit (U9).
- [x] D-7. **sameAs-canonical class lookup** (decided 2026-08-05). Statement:
  "Everywhere the reasoner or gate answers a *class* question, IRIs are first
  mapped through the `owl:sameAs` canonical map — `Schema::class()` gains a
  canonical-aware sibling used by gate range/oneOf checks and by rule-condition
  class references, so an anchored external IRI (`sct:29857009`) is first-class
  wherever its local twin is." Evidence: PRP-001 gap (spurious rejects only);
  subsumption is already canonical — the asymmetry is the bug. Scope note: value
  *identity* (`Value::Ref` equality) stays token-exact; only class-position lookups
  canonicalize. Fallback: none needed — strictly fewer spurious rejects.

## Unit log

(one line per unit as it completes)

## Gaps discovered during implementation

(dated notes; transfer to the PRP folder's 02-tasks-and-gaps.md on rebase)
