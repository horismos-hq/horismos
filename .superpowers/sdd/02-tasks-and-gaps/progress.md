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

- [x] D-1. **Matcher: Rete-lite default, naive stays as the oracle** (decided
  2026-08-07 from U8's criterion run). Statement: "Forward chaining's default
  matcher is chosen by criterion benchmark; naive and Rete-lite both ship, the
  loser as differential-testing oracle." Evidence: PRP Key decision 1 prescribes
  exactly this; doc 14 pins CI-tracked criterion benchmarks from PRP-002 onward.
  RESULT (2026-08-07, `cargo bench -p horismos-core --bench matching`, Apple
  Silicon dev machine): triage_transaction naive 5.30 µs vs rete 5.47 µs per
  full transaction — a wash, both ~200× under the 1 ms perf posture;
  chain60_transaction (60-rule sequential chain, 31 instances) naive 1.60 ms —
  OVER the 1 ms posture — vs rete 126.9 µs, a 12.6× win exactly where doc 14
  predicted incremental matching pays. Default = `MatcherKind::ReteLite`;
  `NaiveMatcher` stays selectable via `World::use_matcher` as the
  differential-testing oracle (pinned by the `differential_tests` proptest:
  byte-identical traces and stores under both). Fallback: `use_matcher` is the
  escape valve if numbers change at an unmeasured scale.
- [x] D-2. **Proof-tree granularity: per-condition** (decided 2026-08-05).
  Statement: "Every condition evaluation — including each subsumption hop — is its
  own `ProofNode`; a firing is the parent node." Evidence: doc 04's `ProofNode`
  literally shows per-condition children and explicit `via: subsumption(...)` nodes
  ("hierarchy steps are explicit proof nodes"); `backward.py` produces per-condition
  subgoals; the PRP says per-rule loses if downstream consumers use the detail —
  narration (doc 06) and the authoring loop (doc 13) both render the per-condition
  tree. Measured cost (2026-08-07, U10 DoD run over triage+SNOMED, one reasoning
  step, full per-condition proof with the subsumption hop): **778 bytes compact,
  1,299 bytes pretty** — the per-rule size argument does not materialize at
  profile scale (bounded < 4 KB by the DoD test). Fallback: if realistic traces
  blow up, collapse *rendering*, not the recorded structure.
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
- [x] D-5. **Retraction of rule-derived facts: forbid, except from the rule path
  itself** (decided 2026-08-05, refined 2026-08-06). Statement: "A fact recorded as
  rule-derived refuses retraction from perception/tool/api provenance; the error
  names the deriving rule and points at re-running inference. `Provenance::Rule`
  retraction remains permitted — it is the inference side's own mechanism, and
  PRP-001 U7 already pinned 'a rule may retract what a rule asserted'
  (`retraction_passes_the_same_provenance_policy`)." Evidence: doc 03's lean
  ("lean forbid — derived facts are retracted only by re-running inference");
  cascade requires truth-maintenance machinery PRP-002's scope excludes; allowing
  caller-side retraction would let the world contradict its recorded proofs —
  trace and world disagreeing violates principle 2. Fallback: cascade can be added
  later behind the same gate check without changing the forbid default.
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

- Task U1: complete (commit 68c5fb4, verify green). Rule model + load-time parse
  incl. D6 rule-side unit check and D-7 canonical class lookup.
- Task U2: complete (commit b3c98b3). Evaluation + D-4 resolved (∀-isNot), doc
  02/04 markers struck, property test pins isNot ≡ ¬is.
- Task U3: complete (commit 0471656, amended for fmt — the pre-amend commit
  briefly carried fmt drift, noted). Matcher trait + naive oracle.
- Task U4: complete (commit c0efc2f). Forward chaining in the transaction,
  conflict policy incl. late-override re-marking, cap rollback, per-fact
  provenance store, gate-diagnostic rejections; ALSO closed the PRP-001
  wire-form provenance gap (snapshot wire v2) because chaining made every
  triage world hold a RulesOnly fact — snapshot round-trip required it.
- Task U5: complete (commit fd8f2fa). explain() + doc-04 render pinned
  byte-exactly; leaves carry provenance.
- Task U6: complete (commit 88f21a7). prove(goal) via cloned hypothesis world
  through the real gate; cycle set + depth cap 50 ported as-is.
- Task U7: complete (commit 080ea1a). horismos.trace/1 schema (all four step
  kinds are v1; reasoning emitted), byte-exact serialization pinned, D-3
  resolved: policy in trace::migrate module doc, v1→v2 dry-run test, export
  tag set pinned by test, doc 09 marker struck.
- Task U8: complete (commit 9af7089). Rete-lite + differential proptest vs
  the naive oracle + criterion benchmarks; D-1 resolved (ReteLite default,
  12.6× at chain-60 scale; see 03-benchmarks.md).
- Task U9: complete (commit d8cf777, amended once for a clippy manual-map).
  Immutable-binds-retraction (D-6), derived-facts-forbid with rule-path
  exception (D-5), sameAs-canonical oneOf/range at the gate (D-7); doc 02/03
  markers struck; single-typed ruling recorded in doc 03.
- Ruling: matcher unit tests re-baselined after U4 integration (assertion now
  chains, so post-assert world states include derived facts — candidate-
  selection expectations updated accordingly; candidate ORDER semantics
  unchanged).
- Ruling (U4): a conclusion that is both a conflict loser and gate-invalid is
  recorded as *suppressed* (conflict policy runs before the gate — it decides
  whether the conclusion is even proposed); Example 3's diagnostic surfaces
  when no standing conflict shields the rule, which is the PRP's own setup.
- Ruling (U6): negated combinators are evaluated, never derived toward, in
  prove — non-monotonicity under closed world; documented in backward.rs.

## Inline self-review of U1–U6 (2026-08-07; firstmate-authorized fallback —
## senior-reviewer infra down, see status log)

Ran the `.claude/skills/self-review` checklist over the committed range:
- Determinism: no HashMap/HashSet/wall-clock/RNG in any reason module; all
  chaining state is BTree/imbl-ordered; record-level trace equality pinned by
  test. PASS.
- Boundary: no new deps in U1–U6 (serde_json arrives with U7 — pure,
  wasm-safe, workspace-declared). PASS.
- Gate: apply_rule_conclusion and the wire-v2 restore path both route through
  gate::check_assertion*; prove mutates only a cloned hypothesis
  (copy-on-write verified by the prove-is-read-only test). PASS.
- Errors: rule load errors are thiserror variants with full-IRI rendering —
  matches PRP-001's load-error style (SubclassCycle, InvalidPattern);
  compacted domain-term form remains the gate-witness convention. Ruling:
  consistent, keep.
- FIXED during review: `concludes_goal` literal match was exact `==`; now
  uses the evaluator's numeric cross-typed `value_eq` (U4–U6 named risk 5).
- FIXED during review: U1 unit-mismatch test upgraded to exact-sentence
  assertion.
- Task U1–U6 minors (deferred): (a) malformed condition rdf-lists stop the
  walk silently if hand-authored triples evade oxttl's desugaring — profile
  validation owns list well-formedness, revisit in PRP-005 verification;
  (b) `ConditionTree::leaves()` currently test-only — U8's alpha-memory
  build is its intended consumer.
- Named risks re-checked mechanically: prove aliasing (risk 4) — clone-on-
  write confirmed by test; wire-v2 forgery (risk 6) — bounded, pinned by
  a_forged_provenance_cannot_smuggle_an_invalid_value.

## Inline self-review of U7–U10 (2026-08-07; option-b fallback, second batch)

- Task U10: complete (commit 3ccb657, after a clippy type-complexity fix
  caught by the gated verify). DoD run + property suite; D-2 size evidence
  recorded (778 B compact).
- Determinism: trace modules have no HashMap/wall-clock; 38 optionals are
  `skip_serializing_if`; bindings/labels are BTreeMaps; emission is a pure
  function of (world, record) — pinned by byte-equality tests + proptest.
- Boundary: serde_json (pure) and criterion (dev-only) — check-boundary and
  no-model-gate green throughout.
- FIXED during review: the `Provenance` serde doc claimed "never a variant
  index" — false for the postcard snapshot wire form, which encodes variant
  order; the comment now states the real contract (reordering variants =
  snapshot-format break, bump SNAPSHOT_VERSION).
- YAGNI check: all four step kinds in trace v1 is deliberate (adding a kind
  later would be a breaking v2 per the D-3 policy — the shapes are doc 09's
  own); `use_matcher`/`MatcherKind` has three real consumers (benches,
  differential tests, the D-1 escape valve); `EXPORT_TAGS` is D-3's pinned
  resolution awaiting the PRP-003 exporter.
- Deviation to name in the PR: status surfaces (U-item boxes, PRP Status
  flip, BOARD.md + board artifact) are DEFERRED until PR #3's folder
  conversion merges — the launch brief forbids restructuring PRP docs in
  this branch; the ledger carries the record meanwhile.

## Gaps discovered during implementation

(dated notes; transfer to the PRP folder's 02-tasks-and-gaps.md on rebase)

- 2026-08-06 — **retraction does not re-chain**: doc 03's transaction stages
  are implemented for assertion; a retraction commits without forward
  chaining, so a rule enabled by an absence (`isEmpty`/`isNot`) does not fire
  until the next assertion-bearing transaction. No TMS in v1 (per D-5's
  forbid). Owner: PRP-004's transaction/workflow pass, alongside
  subscriptions.
- 2026-08-06 — **derivation records don't survive the wire form**: fact
  provenance now travels (wire v2) but the proofs behind `explain` are
  rebuilt only by re-running inference (doc 09 replay); a byte-restored world
  answers `explain` with None until then. Documented on `from_snapshot`.
- 2026-08-06 — **snapshot wire v1 refused, not converted**: SNAPSHOT_VERSION
  bumped to 2 for per-fact provenance; no pre-release v1 corpus exists, so
  from_bytes refuses v1 with its version message rather than converting.

## Gaps discovered during implementation

(dated notes; transfer to the PRP folder's 02-tasks-and-gaps.md on rebase)
