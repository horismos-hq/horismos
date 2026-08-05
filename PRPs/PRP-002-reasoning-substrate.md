# PRP-002 — Reasoning Substrate

**Status**: Draft
**Depends on**: PRP-001

This PRP is **teaching-grade** (see `CONVENTIONS.md` § Teaching-grade PRPs): the five
spec sections — Intent, Scope, Non-goals, Key decisions, Definition of done — are the
phase contract, unchanged in meaning from the original one-pager. Around them sit
background, worked examples, Python-parallel notes, and a glossary, written for a
reader who knows Python well but is new to symbolic reasoning and to Rust. Teaching
sections cite `docs/framework/` (especially [doc 04 · Reasoning](../docs/framework/04-reasoning.md))
rather than restating it; where they summarize, the doc is the authority.

## Intent

Deterministic inference over the instance graph: forward chaining to fixed point,
backward chaining with proof trees, subsumption, constraint checking — with traces
designed as a training corpus from day one.

## What PRP-001 already provides

Everything in this PRP builds on the ontology core that is already Done
([PRP-001](PRP-001-ontology-core/01-objective.md)):

- An `Ontology` loaded from Turtle (`examples/triage/triage.ttl` plus a ~50-concept
  SNOMED chest-pain subset), validated against the RDFS-plus profile, with a
  queryable schema: classes, the `rdfs:subClassOf` closure, property domain/range,
  and the constraint vocabulary (doc 02).
- A `World` of typed instances where **every mutation passes the five-check
  validation gate** — class exists, domain/range, constraints, cardinality,
  provenance policy — rejecting atomically with domain-term witnesses like
  "heartRate 1280 exceeds horis:max 300 (bpm) on :Patient" (doc 03).
- O(1) snapshots with structural diff, tier-1 pattern queries, and a proptest suite
  pinning gate invariants.

PRP-002 adds the **Reasoner**: the component that derives new facts from rules,
proves goals on demand, and records how — without an LLM anywhere in the loop.

## Background — the ideas this PRP assumes

### Rules over a typed graph

A Horismos rule is data in the ontology, not code. `triage.ttl` ships one:

```turtle
:r1 a horis:Rule ;
    horis:priority 10 ;
    horis:when [
        horis:all (
            [ horis:subject :Patient ; horis:property :heartRate ; horis:exceeds 120 ]
            [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ]
        )
    ] ;
    horis:then [ horis:property :acuity ; horis:value :Immediate ] ;
    horis:explain "Tachycardia with chest pain — immediate track" .
```

Read it as: *for any `:Patient` whose `heartRate` exceeds 120 and who has a symptom
that is a `:ChestPain`, set `acuity` to `:Immediate`.* Conditions use the 11
operators and 3 combinators defined in doc 02 — nothing else. The conclusion is not
executed as a side effect; it is **asserted through the same validation gate as every
other mutation**, with provenance `rule`. That is principle 1 of the framework: the
symbolic runtime decides, and even its own conclusions get no back door.

> **Python parallel.** A rule is roughly a frozen dataclass
> `Rule(priority, when: list[Condition], then: Conclusion, explain: str)` that an
> engine interprets — like a row in a rules table — not a `def` that runs. In Rust
> the conditions become an enum with one variant per operator
> (`Condition::Exceeds { property, value }`, …); a Rust `enum` is a *tagged union*:
> like a Python `Union[Exceeds, Below, …]` of dataclasses, except the compiler
> forces every `match` to handle every variant, so "forgot the `isEmpty` case" is a
> compile error, not a 3 a.m. page.

### Forward chaining and the fixed point

Forward chaining is data-driven inference: start from the facts, fire every rule
whose conditions hold, which may derive new facts, which may enable more rules —
repeat until nothing new can be derived. That resting state is the **fixed point**:
running the rules once more would change nothing.

Doc 04 fixes the loop precisely; the semantics that matter:

- **One firing per pass**: each pass picks the highest-priority candidate whose
  conditions hold (ties broken by rule IRI, lexicographic — never load order), fires
  it, and re-evaluates. Derived facts can enable rules mid-run.
- **No re-fire**: a (rule, bindings) pair fires at most once per transaction —
  `r1` cannot fire twice for the same patient.
- **Conflict policy**: two rules concluding different values for the same
  single-cardinality property is a conflict; higher priority wins, and the losing
  conclusion is recorded in the trace as *suppressed*, not silently dropped.
- **Iteration cap** (default 100) as a tripwire; termination is separately verified
  in PRP-005 (doc 10).

Forward chaining runs *inside every World transaction* (doc 03): gate → assert base
facts → chain to fixed point → constraint re-check → commit or roll back entirely.
A caller never observes a world that is half-chained.

> **Python parallel.** sigil-lang's evaluator and its Rust VM
> (`../sigil-lang/src/sigil_core/src/vm/rules.rs`) are the proving ground: a
> priority-sorted loop over rules with a `fired_rules: HashSet` and a
> `MAX_ITERATIONS` cap — the same shape as a Python
> `while True: … if not any_fired: break` over a `set()` of fired names. The port
> keeps those semantics and re-keys them to graph patterns (class + property)
> instead of flat variable names.

### Backward chaining and proof trees

Backward chaining is goal-driven: instead of running everything forward and looking
at what fell out, start from a goal — "*could* `:p7` have `acuity :Immediate`?" —
find the rules that could conclude it, and recursively try to establish their
conditions from facts or from other rules. The result is a **proof tree**: the goal
at the root, one child per condition, leaves at asserted facts. Cycle detection and
a depth cap (50) keep mutually-recursive rules from looping forever.

The API surfaces this twice (doc 04): `prove(goal)` searches for a derivation that
*would* establish the goal; `explain(fact)` returns the recorded derivation of a
fact that *is* established, straight from the forward-chaining trace. Both return
the same `ProofNode` structure, so "why?" has one answer format everywhere —
traces, narration (doc 06), and the authoring loop (doc 13).

> **Python parallel.** The Python-side anchor is sigil-lang's
> `src/sigil/interpreter/backward.py`, ported to Rust with graph goals: its
> `ProofNode` dataclass (`goal, rule, subgoals, result, bindings`) becomes a Rust
> struct; its `_UNBOUND` sentinel becomes `Option<Value>` (`None` instead of a
> magic marker object); its `self._proving: set[str]` cycle guard and
> `MAX_DEPTH = 50` carry over as-is; its rule index (`dict[str, list[RuleDecl]]`
> mapping each settable variable to the rules that set it) becomes a map from
> property IRI to concluding rules. Where the Python code signals failure by
> returning a node with `result=False` or raising `SigilRuntimeError`, the Rust
> port returns `Result`/`Option` — Rust has no exceptions; fallibility is in the
> return type, and the compiler makes ignoring it a visible choice (`?` propagates,
> like a `raise` that the function signature admits to).

### Subsumption — the quiet superpower

`triage.ttl` declares a small hierarchy, anchored into SNOMED CT:

```
:CrushingChestPain ⊑ :ChestPain ⊑ :Symptom        (⊑ = rdfs:subClassOf, "is a kind of")
:ChestPain owl:sameAs sct:29857009
```

Subsumption is class-hierarchy reasoning, and it is always on: the condition
`horis:is :ChestPain` matches an instance of **any subclass**, including subclasses
that arrived via imports. So `r1` fires for a patient with `:CrushingChestPain`
although no rule mentions crushing chest pain — the SNOMED subset supplied hierarchy
nobody had to write a rule for. Every such match is recorded as an explicit
`via: subsumption(…)` step in the proof tree, so the hierarchy hop is visible, not
magic. Property inference stays limited to `rdfs:domain`/`rdfs:range` typing; there
is no OWL-style complex-class inference (doc 04).

PRP-001 built the `subClassOf` closure with `owl:sameAs`-canonical lookup; this PRP
makes the reasoner consume it — and inherits PRP-001's recorded gap that anchored
external IRIs are second-class in exact-IRI lookups (see *Other open questions*).

### Matching, and why Rete exists

The naive way to find candidate rules each pass is to re-evaluate every condition of
every rule against the whole graph — O(rules × facts) per pass, re-doing almost all
of the work each time, since one firing changes only a sliver of the world. The
classic fix is the **Rete** algorithm: compile the rules into a network that
remembers partial matches, so when a fact arrives only the affected rules get
re-checked. A full Rete has an *alpha network* (per-condition filters with memories:
"all facts where heartRate exceeds 120") feeding a *beta network* (joins across
conditions with their own memories).

**Rete-lite**, sigil-lang's variant (`../sigil-lang/src/sigil_core/src/matcher.rs`),
keeps the alpha level — rules compiled to condition lists, per-condition evaluation,
priority-sorted match results — without the full beta-join machinery. This PRP
adapts it to ontology instances: alpha nodes keyed on **class + property**
predicates, so asserting `:p7 :heartRate 128` touches only the conditions that
mention `:Patient` × `:heartRate`. Whether the network is worth building *first* is
Key decision 1 — the naive loop is the correctness oracle either way.

> **Python parallel.** Naive vs. Rete-lite is a strategy choice behind one
> interface. In Python you would write a `Matcher` `Protocol` (or ABC) with
> `candidates(world) -> list[Match]` and two implementations; in Rust that is a
> `trait Matcher` with two implementing types. Same idea, one difference in
> flavor: Rust resolves the implementation at compile time (generics / static
> dispatch) unless you explicitly ask for runtime dispatch (`dyn Matcher`, the
> analogue of passing any duck-typed object).

### Negation: closed world vs. open world

RDF is **open-world** by default: a triple you don't have is merely *unknown* —
maybe true, maybe false, nobody said. That is the right stance for the open web and
the wrong one for operational state: a triage runtime that answers "unknown" to
"is the heart rate above 120?" for a patient with no recorded heart rate cannot
route anybody. Horismos therefore takes a documented, deliberate **pragmatic
closed-world** reading (doc 04): the world *is* the record.

The edge semantics, inherited from sigil-lang's `_UNBOUND` precedent (comparisons
with an unbound variable are false; `is not` is true) and re-stated for the graph in
doc 04's table:

| Situation | Result |
|-----------|--------|
| Property unassigned, any comparison (`exceeds`, `is`, …) | `false` |
| Property unassigned, `horis:isNot` | `true` |
| Property unassigned, `horis:isAssigned` | `false` |
| Multi-valued property, `horis:is X` | `true` if **any** value matches (∃) |

Re-validating this stance for the graph world — including the still-open ∃ vs. ∀
question for multi-valued `horis:isNot` — is Key decision 4. Nothing here is
resolved by this document.

### Traces are output, not logging

Principle 2: the audit log and the training corpus are the same object. Every
inference step this PRP performs — fired rules with bindings, derived facts,
suppressed conclusions, proof trees, subsumption hops — is recorded as typed steps
in the versioned trajectory schema of doc 09 (`horismos.trace/1`), with `loss_mask`
and outcome-label fields reserved per the TIR/CIR literature (sigil-lang
`docs/research/INTERLEAVED_SYMBOLIC_EXECUTION.md`). The reasoner does not *also*
write a log; the trace **is** the log, and it is what a training exporter reads.

> **Python parallel.** A typed trace step is a Rust enum
> (`Step::Reasoning { fired, suppressed, goal_reached }`,
> `Step::Perception { … }`, …) serialized with `serde` — the `"kind"` field in the
> doc-09 JSON is the union tag. The Python analogue is a `Union` of frozen
> dataclasses with a discriminator field, `json.dumps`'d by a schema-aware
> encoder. Rust's `#[derive(Serialize)]` plays the role of `dataclasses.asdict`
> plus a version-pinned schema.

## Scope

- **Subsumption**: class-hierarchy reasoning (`CrushingChestPain ⊑ ChestPain ⊑ Symptom`),
  including imported hierarchies; property domain/range inference.
  (Background above; worked into every example below.)
- **Forward chaining**: rules fire over the graph to a fixed point; priority ordering,
  fired-rule tracking (no re-fire), iteration cap — port the semantics proven in
  sigil-lang's evaluator and Rust VM `rules.rs`; adapt the Rete-lite matcher to
  ontology instances (alpha nodes keyed on class + property predicates).
  (Worked example 1.)
- **Backward chaining**: `prove(goal)` with cycle detection and depth cap; `explain(fact)`
  returns the proof tree — port sigil-lang `backward.py` semantics to Rust.
  (Worked example 2.)
- **Constraint checking** as continuous invariant: violations are impossible by
  construction (gate) or surfaced as diagnostics when rules would derive them.
  (Worked example 3.)
- **Traces** (the flywheel obligation): every inference step recorded in a stable,
  versioned schema — typed steps, fired rules, proof trees, timing; `loss_mask` and
  outcome-label fields reserved per the TIR/CIR literature (see sigil-lang
  `docs/research/INTERLEAVED_SYMBOLIC_EXECUTION.md`). JSON export, compact and pretty.

## Worked examples

All three examples use the shipped fixture vocabulary
(`examples/triage/triage.ttl`): `:Patient`, `:heartRate` (xsd:decimal, bpm,
0–300, cardinality 0..1), `:hasSymptom` (0..*), `:acuity` (one of `:Immediate`
`:Urgent` `:Standard`, cardinality 0..1, `horis:assignedBy horis:RulesOnly`), the
hierarchy `:CrushingChestPain ⊑ :ChestPain ⊑ :Symptom`, and rule `:r1`. Rules
`:r2`, `:r3`, `:r9` and the property `:needsCardiacMonitor` are **illustrative
only** — declared inline here so every step is hand-checkable, not part of the
fixture. Every construct they use is declared doc-02 vocabulary.

### Example 1 — forward chaining to fixed point

Illustrative additions to the rule set:

```turtle
:r2 a horis:Rule ;                       # chest pain alone is at least urgent
    horis:priority 5 ;
    horis:when [ horis:all (
        [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ] ) ] ;
    horis:then [ horis:property :acuity ; horis:value :Urgent ] ;
    horis:explain "Chest pain — at least urgent" .

:needsCardiacMonitor a rdf:Property ;    # illustrative property, not in the fixture
    rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:cardinality "0..1" .

:r3 a horis:Rule ;                       # immediate-track patients get a monitor
    horis:priority 8 ;
    horis:when [ horis:all (
        [ horis:subject :Patient ; horis:property :acuity ; horis:is :Immediate ] ) ] ;
    horis:then [ horis:property :needsCardiacMonitor ; horis:value true ] ;
    horis:explain "Immediate track — continuous cardiac monitoring" .
```

A transaction asserts patient `:p7` with `heartRate 128` and
`hasSymptom :CrushingChestPain` (both pass the gate), then chains:

**Pass 1.** Evaluate all rules against the graph.
- `:r1` (priority 10): `128 exceeds 120` ✓; `hasSymptom` has value
  `:CrushingChestPain`, and `:CrushingChestPain ⊑ :ChestPain` ✓ (subsumption).
  Candidate.
- `:r2` (priority 5): same subsumption match ✓. Candidate.
- `:r3` (priority 8): `acuity` is unassigned → any comparison over an unassigned
  property is `false` (doc 04 table). Not a candidate.

Highest priority candidate is `:r1` → fire. `(:p7 :acuity :Immediate)` goes
**through the gate** with provenance `rule`: class ✓, range `:AcuityLevel` ✓,
`oneOf` ✓, first value under cardinality 0..1 ✓, and `horis:assignedBy
horis:RulesOnly` admits provenance `rule` ✓. Derived. `(r1, {?p→:p7})` enters the
fired set.

**Pass 2.** Re-evaluate — the derived fact changed the graph.
- `:r1`: conditions hold, but `(r1, {?p→:p7})` already fired → excluded (no re-fire).
- `:r3` (priority 8): `acuity horis:is :Immediate` now ✓. Candidate.
- `:r2` (priority 5): still a candidate.

Fire `:r3` (8 > 5). `(:p7 :needsCardiacMonitor true)` passes the gate. Derived.

**Pass 3.** Remaining candidate: `:r2`. Fire it. Its conclusion
`(:p7 :acuity :Urgent)` conflicts with the existing `:Immediate` — different value
for the same single-cardinality property. Conflict policy: the higher-priority
rule's value stands; `:r2`'s conclusion is recorded in the trace as **suppressed**.
`(r2, {?p→:p7})` is fired either way.

**Pass 4.** No candidates remain → **fixed point** in 4 passes, well under the cap
of 100. Final state: `acuity = :Immediate`, `needsCardiacMonitor = true`; trace
records three firings, one suppression, and per-firing proofs. Note the pass-2
lesson: `:r3` was unfireable at pass 1 and became fireable *because of* a derived
fact — that is why the loop re-evaluates instead of scanning rules once. (Statically,
verification will flag the r1/r2 conflict pair at load time — doc 10, PRP-005;
runtime suppression is the last resort, not the design.)

### Example 2 — `prove` and `explain`: a proof tree by hand

Same world, *before* any chaining: `:p7` has `heartRate 128` and
`hasSymptom :CrushingChestPain`; `acuity` unassigned.

`world.prove(":acuity :Immediate", subject=:p7)` works backwards:

1. Goal `(:p7 :acuity :Immediate)`. Not an existing fact — so find rules whose
   conclusion could establish it. The rule index (property → concluding rules) gives
   `:r1` (and only `:r1`: `:r2` concludes `:Urgent`, a different value).
2. Try `:r1`. Prove each condition:
   - `heartRate exceeds 120`: the fact `heartRate 128` is asserted → leaf,
     `via: asserted`, `128 > 120` ✓.
   - `hasSymptom is :ChestPain`: the fact `hasSymptom :CrushingChestPain` is
     asserted; the hierarchy gives `:CrushingChestPain ⊑ :ChestPain` → leaf,
     `via: subsumption(:CrushingChestPain)` ✓.
3. All conditions proved → the goal is provable via `:r1`.

Rendered (doc 04's format — this exact shape feeds narration and authoring):

```
:acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"
  ├─ heartRate 128 exceeds 120            (asserted by perception, conf 0.94)
  └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (SNOMED subsumption)
```

Every leaf carries provenance down to perception confidence — an auditor sees that
a symbolic conclusion rests on a neural extraction, and how confident it was.

The failing case, checked the same way: patient `:p9` with `heartRate 96` and
`hasSymptom :ChestPain`. Goal `(:p9 :acuity :Immediate)`: only `:r1` concludes
`:Immediate`; its first condition `96 exceeds 120` is false; no other rule can
establish the goal → `prove` returns no proof (`None`). Cycle detection (the
in-progress goal set) and the depth cap of 50 bound the search when rules reference
each other — semantics ported unchanged from `backward.py`.

`explain` differs only in its source: after Example 1's transaction committed,
`world.explain(:p7, "acuity")` replays the **recorded** derivation from the
forward-chaining trace — same `ProofNode` shape, answering "why *is* this
established?" rather than "*could* it be?".

### Example 3 — a constraint violation surfaced as a diagnostic

An authoring mistake, illustrative:

```turtle
:r9 a horis:Rule ;
    horis:priority 1 ;
    horis:when [ horis:all (
        [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :Symptom ] ) ] ;
    horis:then [ horis:property :acuity ; horis:value :Discharged ] .   # not an AcuityLevel
```

`:Discharged` is not in `:AcuityLevel`'s closed enumeration
(`horis:oneOf ( :Immediate :Urgent :Standard )`). The gate has no back door for
rules: when `:r9` fires, its conclusion is checked like any assertion and rejected
with a domain-term witness in the doc-03 template:

```
acuity :Discharged not in horis:oneOf (:Immediate :Urgent :Standard) on :Patient
```

plus the machine-readable structure (constraint IRI, subject, offending value). This
is the scope bullet "constraint checking as continuous invariant" in action:
violations are impossible by construction — a rule cannot push the world into an
invalid state; it can only produce a diagnostic naming exactly which declaration it
would have violated. Transactions stay atomic per doc 03 (commit or roll back
entirely; partial states are never observable). The *earlier* net — flagging `:r9`
at load time so the runtime rejection never happens — is verification, PRP-005
(doc 10); the same static pass that PRP-001's D6 gap assigns the rule-side **unit**
check to at rule load (a condition comparing `heartRate` against a value in
anything but `"bpm"` is a load-time error, never a runtime surprise).

> **Python parallel.** The gate returning
> `Result<Derived, ValidationRejected { witness }>` is the `try/except
> HorismosValidationError` of the Python surface (doc 03) — one error object
> carrying both `domain_text()` (the sentence above) and the structured fields, the
> same duality as a Python exception with a rich `__str__` and typed attributes.
> Errors are `thiserror`-derived enums: think "exception class hierarchy, but as a
> tagged union the compiler checks exhaustively."

## A note on ownership (the store and the reasoner)

The reasoner *reads* the world during matching and *writes* only through gate-checked
transactions. Rust makes that split mechanical: matching borrows the world immutably
(`&World` — any number of simultaneous readers), while a transaction takes the
exclusive mutable borrow (`&mut World` — one writer, no readers, enforced at compile
time). Doc 12's "single-writer by design" is not a convention the team promises to
follow; the borrow checker rejects code that violates it.

> **Python parallel.** In Python this discipline exists too — "don't mutate the
> world while iterating a query over it" — but it lives in code review and runtime
> `RuntimeError: dict changed size during iteration` surprises. Rust moves that
> class of bug to compile time. Two adjacent facts worth knowing: interned IRIs
> (`Iri` as a `u32` newtype into an arena, PRP-001's D2) are the systems version of
> `sys.intern` — compare integers, not strings; and determinism forbids iterating
> a `HashMap` where order can leak into results (Rust's `HashMap` order is
> unspecified, like pre-3.7 dicts) — ordered structures (`BTreeMap`) or explicit
> sorts are used wherever iteration order could reach a trace.

## Non-goals

- No workflow execution, no LLM calls, no BDD verification (PRP-004/005).
- No probabilistic/fuzzy inference — determinism is the product.

## Key decisions to make

All four remain **open**. They become D-items at activation (`CONVENTIONS.md`) and
are resolved via the repo brainstorming protocol — never ad hoc, and not in this
document. Context below states the options and what evidence would settle each.

1. **Rete-lite vs. naive match-loop first** (sigil-lang data: marshaling dominated at
   small scale; here everything is in-process Rust, so Rete pays off earlier —
   benchmark).
   *Context*: the naive loop re-evaluates every rule against the graph each pass —
   trivially correct, easy to read, and the natural parity oracle; Rete-lite keeps
   per-condition alpha memories keyed on class + property so a single assertion
   re-checks only the rules that mention it. In sigil-lang the PyO3 marshaling cost
   swamped matching at small scale, so the network never paid for itself; with the
   store and matcher both in-process Rust that excuse is gone, but triage-scale
   rule counts may still be too small to notice. What settles it: criterion
   benchmarks (naive vs. Rete-lite) on the triage fixture and a larger synthetic
   ontology — the DoD's benchmark requirement exists precisely to feed this
   decision, and whichever loses stays in-tree as the differential-testing oracle.

2. **Proof-tree granularity**: per-condition (sigil-lang precedent) vs. per-rule.
   *Context*: per-condition means every condition evaluation — including each
   subsumption hop — is its own `ProofNode`, which is what doc 04's rendered tree
   shows and what `backward.py` produces via per-condition subgoals; per-rule
   collapses a firing into one node with condition results as attributes, making
   traces smaller but narration (doc 06) and the authoring loop (doc 13) coarser.
   What settles it: measured trace sizes on realistic runs weighed against what the
   downstream consumers (narration, verification witnesses, training export)
   actually need to reference; if per-rule loses information those consumers use,
   it loses.

3. **Trace schema versioning policy** (traces outlive framework versions once used as
   training data).
   *Context*: doc 09 already fixes the ground rule — `horismos.trace/1` never
   changes meaning; evolution adds `horismos.trace/2` with a converter — so the
   open question is the policy detail: what counts as a breaking change (new
   optional field? new step kind? new suppression reason?), where converters live
   and how they are tested, and how the reserved `loss_mask`/outcome-label fields
   may evolve without invalidating corpora already trained on (the TIR/CIR
   motivation, per sigil-lang `docs/research/INTERLEAVED_SYMBOLIC_EXECUTION.md`).
   What settles it: a written compatibility policy exercised by a dry-run v1→v2
   migration test in this PRP, so the first real schema change follows a rehearsed
   path. The doc-09 training-export tag set (`[DECIDE: exact tag set — PRP-002]`)
   is resolved alongside.

4. **Negation semantics**: `isNot`/absence under open-world RDF vs. Horismos's
   pragmatic closed-world `_UNBOUND` precedent — pick, document, test (sigil-lang's
   answer: comparisons with unbound are false, `is not` true; re-validate for the
   graph world).
   *Context*: the closed-world stance itself is documented and inherited (doc 04's
   edge-semantics table, restated in Background above); what the graph world adds
   is multi-valued properties, where sigil-lang's flat variables offer no
   precedent. For `:p7` with `hasSymptom = {:CrushingChestPain, :Nausea}`, does
   `horis:isNot :ChestPain` mean "some value is not a ChestPain" (∃ — true here,
   via `:Nausea`) or "no value is a ChestPain" (∀ — false here)? The ∀ reading
   makes `isNot` the negation of `is`; the ∃ reading is compositional with the
   ∃-semantics `is` already has. The choice must also square with the
   `horis:none` combinator and the unassigned-property row (`isNot` true). What
   settles it: enumerating the doc-04 table over multi-valued cases, picking the
   reading that keeps the operator algebra coherent, and pinning it with
   property tests; doc 04's `[DECIDE: confirm ∃ vs. ∀ for multi-valued isNot]`
   marker is deleted in the same commit.

### Other open questions this PRP owns

Owned `[DECIDE]` markers and inherited PRP-001 gaps, listed here so activation
finds them (each becomes a D- or U-item; none are resolved here):

- **Retraction semantics for rule-derived facts** — cascade vs. forbid; lean forbid
  (doc 03).
- **Retraction vs. `horis:immutable`** — retract-then-assert currently defeats
  immutability (doc 02; `world/gate.rs`; PRP-001 gap 2026-07-29).
- **sameAs-canonical class lookup** — anchored external IRIs are second-class as
  values; spurious rejects only (PRP-001 gap).
- **Stored fact provenance** — provenance is gate-checked but not stored, so
  `RulesOnly` facts don't survive the wire form; traces record provenance anyway
  (PRP-001 gap).
- **Rule-side unit check at rule load** — D6's strict-units follow-through
  (PRP-001 gap).
- **Training-export tag set** — folded into Key decision 3 (doc 09).

## Definition of done

Triage scenario runs end-to-end symbolically: structured facts in → subsumption +
forward chaining → `acuity` derived → `explain` renders the r1 proof; traces serialize
to the versioned schema; parity tests against hand-computed fixed points; criterion
benchmarks vs. naive matching.

(Concretely: Example 1 and Example 2 above *are* hand-computed expected outputs —
the parity tests assert the engine reproduces them, and trace equality is a valid
assertion because inference is deterministic, doc 04.)

## Glossary

| Term | Plain-English meaning |
|------|----------------------|
| IRI | A global identifier (URL-shaped) for a class, property, or instance — `:heartRate` expands to one. |
| Triple | One fact: subject–property–value, e.g. `(:p7, :heartRate, 128)`. |
| Instance | An individual of a class — `:p7` is an instance of `:Patient`. |
| Subsumption (⊑) | "Is a kind of" reasoning over the class hierarchy; `:CrushingChestPain ⊑ :ChestPain`. |
| Forward chaining | Data-driven inference: fire rules on facts, derive, repeat. |
| Backward chaining | Goal-driven inference: start from a claim, search for rules and facts that establish it. |
| Fixed point | The state where re-running the rules derives nothing new — chaining's stopping condition. |
| Proof tree | The recorded derivation of a conclusion: goal at the root, conditions as children, facts at the leaves. |
| Witness | The concrete evidence naming *why* — a failing constraint in domain terms, or a proof leaf. |
| Bindings | The variable-to-value map for one rule match, e.g. `{?p → :p7}`. |
| Alpha node | A per-condition filter in a Rete network, with a memory of currently matching facts. |
| Rete(-lite) | An incremental matching network that avoids re-evaluating all rules on every change; -lite = alpha level only, no full join network. |
| Closed-world assumption | What is not recorded is treated as false — Horismos's documented stance for operational state. |
| Open-world assumption | What is not recorded is merely unknown — standard RDF/web semantics. |
| Skolem IRI | The stable IRI a blank (unnamed) instance receives at assertion. |
| Interning | Storing each IRI string once and passing around a small integer handle (`u32`). |
| Gate | The five-check validation every World mutation passes — no back door, including for rules. |
| Transaction | One atomic unit: gate → assert → chain to fixed point → re-check → commit or roll back. |
| Provenance | Who asserted a fact: `perception \| rule \| tool \| api` — policy-checked at the gate. |
| Trace | The typed trajectory of a run — audit log and training corpus, one object (`horismos.trace/1`). |
| Loss mask | Per-step flag marking what a trainer should learn from (model output) vs. mask (executor output). |
| Fired set | The (rule, bindings) pairs that already fired this transaction — the no-re-fire guard. |
| Suppressed | A conclusion recorded but not asserted because a higher-priority rule won the conflict. |
