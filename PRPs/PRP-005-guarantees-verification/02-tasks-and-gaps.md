# PRP-005 — Tasks & Gaps

Plan: `PRPs/PRP-005-guarantees-verification/01-objective.md`

Branch: `fm/horismos-prp005-a1`

Worktree: `/Users/billaram/.treehouse/horismos-86ae5f/1/horismos`

Baseline: `5c45c0a3ac4c31ffa8d5d5ec90214d7d8b740e64` (`make verify` green,
2026-09-02)

## Decisions to resolve (before implementation)

- [x] D1. **Verification at load** (decided 2026-09-02): every ordinary load runs
  verification and fails closed by default. `allow_incomplete` is the explicit
  authoring/debug escape and the resulting ontology remains inspectable through
  `verify()`; disabling verification entirely is not a public v1 path. This follows
  doc 10's deployment posture and keeps an invalid rule set from becoming an
  executable default. If repeated full checks breach the one-second budget, cache the
  immutable report inside `Ontology`; do not weaken the default.
- [x] D2. **Incremental re-verification** (decided 2026-09-02): v1 performs a full,
  deterministic verification of each newly loaded immutable ontology. There is no
  ontology-edit API or stable dependency graph to invalidate incrementally, and doc
  14 budgets a full load plus verify at under one second. If the benchmark later
  crosses that budget, the fallback is dependency-indexed recompilation inside
  `verify`, preserving identical reports and witness ordering.
- [x] D3. **Unit-aware numeric discretization** (decided 2026-09-02): thresholds are
  interpreted only in the property's one declared opaque unit. Rule-side labels must
  match exactly (already a load error); unitless rules on a unitful property inherit
  that property's unit. No conversions or dimensional algebra occur. This is already
  the shipped contract in docs 02 and 04; if conversion is ever required, it needs new
  authored mapping vocabulary in its own PRP rather than an implicit verifier table.
- [x] D4. **Assumption-manifest shape** (decided 2026-09-02, provisional pending
  PRP-008): use a dedicated `VerificationReport.assumptions: AssumptionManifest`, not
  witness categories. It carries exactly five fields: `input_closure`,
  `external_profile_versions`, `mapping_version`, `supported_subset`, and
  `unverified_external_effects`; standalone verification supplies explicit empty
  values. PRP-008 may replace their provisional string/list value types with
  `ExternalSnapshot`-linked types, but may not remove a field or make absence
  implicit.

## Controller resolutions

- BDD ownership is `horismos-core::verify`, behind feature `verify`; PyO3 marshals one
  complete report and contains no verification logic.
- Variable names, properties, rules, workflows, witnesses, and JSON arrays are emitted
  in ontology/token or lexicographic order. Witness selection uses the first satisfying
  valuation, never a random valuation.
- `biodivine-lib-bdd` is a workspace dependency with default features disabled. Its
  random-sampling API is never called; determinism is pinned by report equality tests.
- Verification findings are report data. Default-hard loaders translate a non-OK
  report into `VerificationFailed`; `allow_incomplete` returns the ontology and the
  same report remains available.
- Batching approved by the task's required PRP workflow: B1=U1, B2=U2+U3,
  B3=U4+U5, B4=U6+U7. One senior-reviewer pass follows each batch.

## Units of work

- [x] U1. **Verification model and dependency** — add the pure-Rust BDD dependency,
  `AssumptionManifest`, structured regions/witnesses, deterministic JSON, report
  categories, and `exit_code`. Tests pin all five manifest fields when empty and
  populated, witness ordering, and JSON round-trip shape. **Done 2026-09-02**:
  private `ReportBuilder` finalization makes inconsistent `ok`/finding state
  unconstructible; complete JSON shape and permutation tests pin recursive ordering;
  the BDD dependency is `verify`-scoped and a direct wasm32 build is green. Batch review
  approved with no findings after two Important determinism/construction issues were
  corrected (commits `f833a99..HEAD`).
- [x] U2. **Rule-set BDD verification** — ontology-aware numeric/class/value
  discretization plus completeness, conflict, dead-rule, termination, and rule
  non-vacuity checks. Test target: seeded triage gap renders exactly “no acuity
  assigned when heartRate ∈ (100, 120] and no Symptom present” and carries the
  interval/absence region; crafted overlaps/dead/cycles fail. **Done 2026-09-02**:
  exact datatype-aware BDD partitions now cover decimal/integer boundaries, class
  subsumption, exact values, substring predicates, and multi-valued conclusion sets;
  completeness/conflict/dead/shadowing/enablement-cycle witnesses are deterministic.
  The seeded sentence and structured region are pinned exactly. Batch review approved
  after numeric, semantic-equality, regex, and compound/multi-value termination
  soundness findings were corrected.
- [x] U3. **Constraint non-vacuity** — detect declared constraints that exclude no
  otherwise-typed value/cardinality and emit first-class witnesses. Test target:
  `horis:pattern ".*"` is caught in domain terms while a restricting pattern passes.
  **Done 2026-09-02**: explicit `0..*` and semantics-preserving universal-pattern
  forms emit structured domain witnesses; restrictive and whitespace-significant
  patterns remain unflagged.
- [x] U4. **Workflow verification** — reachability, every-path termination,
  fallible-edge totality, requires satisfiability, and choice totality over parsed
  workflow graphs. Crafted fixtures pass and fail each required acceptance check.
  **Done 2026-09-02**: orphan declarations remain available as verification
  evidence without entering executable workflow maps; graph cycles/dead ends,
  missing failure edges, unsatisfiable action preconditions, and uncovered choices
  each produce their own deterministic finding. Optional parsed failure edges have
  a domain-error runtime backstop for explicitly incomplete ontologies. Batch review
  approved after global-orphan attribution and rule-established/chained precondition
  semantics were corrected to mirror the runtime's priority-and-restart fixed point.
  Final review additionally pinned gate-rejected conclusions, mutable and immutable
  standing-winner semantics, distinct multi-valued conclusions, and workflow-only
  string/numeric discretization against real runtime outcomes.
- Task U5: minor (deferred): add direct regressions for `allow_incomplete` propagation
  through `with_import` and execution of an incomplete workflow's missing-edge runtime
  backstop; the paths were inspected and approved during Batch 3 review.
- [x] U5. **Rust load and error surface** — `Ontology::verify`, default-hard load,
  explicit load options/`allow_incomplete`, `VerificationFailed { report }`, and
  public report exports. Existing callers are migrated without weakening production
  defaults. **Done 2026-09-02**: ordinary filesystem and string loads run the full
  verifier and retain the boxed machine report on failure; explicit authoring helpers
  still verify and expose the same deterministic report. Partial internal fixtures use
  the named escape while production loaders remain fail-closed. `make verify` is green.
- [x] U6. **Python and CI surface** — native whole-report marshaling, public Python
  dataclasses, load flags, error report attachment, stubs, and `python -m horismos
  verify` exit `0`/`1` with JSON output. **Done 2026-09-02**: the PyO3 layer returns
  one canonical JSON payload plus the core exit code; frozen Python dataclasses marshal
  every report field, including the five-field manifest. Default-hard failures raise
  `HorismosVerificationError` with the typed report attached. The CLI's failing-report
  response and exit 1 are exercised through a real subprocess. Explicit imports are
  assembled before the single verification decision. `make verify` is green (291 Rust
  unit tests plus integration suites and 74 Python tests). Batch review additionally
  tightened the native exception stub to `VerificationReport` and pinned successful
  and failing CLI output byte-for-byte across repeated subprocess invocations.
- [x] U7. **Triage, benchmark, docs, and DoD** — extend the canonical triage rules to
  a conflict-free complete partition, seed the exact test gap by removing its one
  covering rule, add a Criterion benchmark asserting/measuring the under-one-second
  budget, and update shipped API/verification docs. **Done 2026-09-02**: canonical
  triage now verifies under ordinary fail-closed loading, while the gap regression
  removes the marked moderate/no-symptom rule from that same artifact and pins the
  exact sentence plus structured numeric and absence predicates. A test enforces the
  one-second ceiling and Criterion measures load plus verification below 1 ms on this
  worktree. Docs 09–12 cover the shipped report, CLI, load policy,
  assumption manifest, fixed-point workflow composition, and durable trace annotation
  for the explicit incomplete escape. `make verify` is green (296 Rust unit tests plus
  integration/property suites and 74 Python tests).
- [x] U8. **Finish sequence** — `make verify`, `make check-boundary`, and
  `make no-model-gate`; final whole-branch senior review; mark PRP Done and update
  BOARD plus `planning/board.html`; commit the complete branch. **Done 2026-09-03**:
  final review approved with no findings after runtime-alignment regressions; all
  required Makefile gates, the wasm build, and the triage benchmark are green. The
  repo boards mark PRP-005 Done and also correct PRP-004 to Done after PR #8.

## Post-review corrections (2026-09-03)

- Ordered comparisons now discretize `xsd:dateTime` and string constants as
  thresholds in their own opaque orders, mirroring the runtime's `compare`; an
  incomparable operand (boolean, duration, mismatched datatype) never holds instead
  of silently being classified as a dead rule.
- `xsd:boolean` properties are closed over `true`/`false`, so `is true` / `is false`
  partitions and choice branches verify as total.
- Rule conclusions on perception-settable properties compose into downstream rule and
  workflow conditions (input value or any standing conclusion).
- Explicit `horis:cardinality "0..*"` is an info-severity note (`Witness.severity`);
  hard `ConstraintVacuity` remains for universal patterns and for `horis:min`/`max`/
  `pattern` that cannot reject a value of the property's type. Canonical triage
  declares `0..*` on `:hasSymptom` again and still loads.
- `WorkflowVerification.reachable` was removed (it could never be false); unreachable
  steps remain global `WorkflowReachability` witnesses.
- `Conflict` carries `first_priority`, `second_priority`, and `winner`; its witness
  reads "r1 → v1 but r2 → v2 for p when … — r1 wins by priority; confirm intended".
  When both rules share a priority, `winner` is `None` and the witness states that
  the standing value is order-dependent and asks the author to raise one priority.
- Second review round: a standing conclusion on a mutable single-valued
  perception property replaces the input value in downstream conditions (the gate's
  own replace semantics) rather than being unioned with it; thresholds of different
  kinds on a range-less property never share a cell.
- Third review round: the priority-ordered, restart-on-growth fixed point already
  gives a reader that outranks the writer its pre-replacement view (candidates only
  grow), so the swapped-priority scenario reports its genuine conflict and the
  `isEmpty` reader is live; both are pinned as behavioral regressions against a
  World run rather than by restructuring the read model.
- PRP-004's own status record now reads Done (PR #8, 2026-08-25).

## Gaps discovered during implementation

- 2026-09-02: the specified sibling prior-art checkout `../sigil-lang` is not present
  in this disposable worktree. The implementation follows the PRP's stated semantics
  and the public `biodivine-lib-bdd` API; no source-level port claim will be made.
