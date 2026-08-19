# 03 · The World

The World is the typed instance store — the agent's memory of record. It holds facts
as instances of ontology classes, and **every mutation passes the validation gate**.
There is no back door.

## Creating a world

```python
from horismos import Ontology, World

onto  = Ontology.load("triage.ttl", imports=["snomed-ct-subset.ttl"])
world = World(onto)
```

A World is always bound to exactly one Ontology. Facts are triples over interned IRIs;
blank-node instances get stable skolem IRIs at assertion.

## Assertion and the validation gate

```python
p = world.assert_(Patient(heart_rate=128,
                          systolic=92, diastolic=60,
                          symptoms=[CrushingChestPain, Diaphoresis]))
```

The gate checks, in order — and rejects the whole assertion atomically on the first
failure:

1. **Class exists** in the ontology (or an imported graph).
2. **Property domains/ranges**: `heartRate` on a `Patient`, value is `xsd:decimal`.
3. **Constraints**: `128 ∈ [0, 300]` ✓; unit consistency; `horis:pattern`.
4. **Cardinality**: a second `heartRate` on the same patient replaces only if the
   property isn't `horis:immutable`; violations reject.
5. **Provenance policy**: `horis:assignedBy` — an assertion arriving from a
   PerceptionStep may not set `horis:RulesOnly` properties (the gate knows each
   assertion's provenance: `perception | rule | tool | api`).

Rejections raise `HorismosValidationError` with the failing constraint **in domain terms**
("heartRate 1280 exceeds horis:max 300 (bpm) on :Patient") and a machine-readable
structure (used by the perception retry loop, doc 06).

`world.retract(fact)` exists and passes the same policy checks, plus two retraction
rulings (decided, PRP-002): a `horis:immutable` fact refuses retraction outright,
and a **rule-derived** fact refuses retraction from perception/tool/api — the error
names the deriving rule; the remedy is to change the *base* facts the derivation
rests on (see Truth maintenance, next) — the rule path itself remains admitted (a
rule may retract what a rule asserted).

## Truth maintenance

**Derived facts are recomputed, never left stale.** Every external mutation —
`assert_`, a base-fact replacement, or `retract` — ends with the world at the
forward-chained fixed point of its *final* base facts, exactly as if that fact set
had been asserted into a fresh world. A derived `:Immediate` acuity that rests on
`heartRate` 128 cannot survive that `heartRate` being replaced by 96 or retracted:
the transaction that changes the support also invalidates and, where the new base
facts still warrant it, re-derives the conclusion — through however many rules the
support was chained (a fact derived from a derived fact is invalidated too).

This closes a decision doc 03 previously left ambiguous ("derived facts change only
by re-running inference after base-fact changes" did not specify that re-running
inference *invalidates* what no longer holds). The captain's ruling
(`tm-derived-facts.md`) and PRP-002's decision record (`03-decisions.md`) make it
concrete: **recompute-on-base-mutation**, not derivations-as-immutable-history. A
derivation is a claim about the *current* world, not a logged event — the trace is
where history lives (doc 09); the world is where the current fixed point lives.

Implementation note (v1): the current approach resets every standing derived fact
and re-chains to the new fixed point on every external transaction, rather than
tracking per-fact dependency edges and invalidating only what a change actually
touches. This is simple and provably correct — the property test suite pins
equivalence with fresh-world reconstruction — at the cost of doing full-graph work
on every transaction, not just ones near the change. Incremental truth maintenance
(a real TMS) is a possible future optimization, not built here; see the PRP-002
decision record for the measured cost on the triage fixture.

One modeling ruling made at the gate in PRP-001 and recorded here: **instances are
single-typed** — a subject asserted as a second `rdf:type` is refused
(`ClassConflict`). Multi-classification is modeled through the class hierarchy
(`rdfs:subClassOf`), not through multiple types on one instance.

## Queries

Three tiers, all read-only:

```python
# 1. Pattern lookup (fast path)
world.instances(":Patient")
world.value(p, "heart_rate")                     # 128

# 2. Pattern query (class scan + conjunctive value filters)
world.query(":Patient", acuity=":Immediate")

# 3. Reasoning queries (delegate to the Reasoner, doc 04)
world.prove(p, ":acuity", ":Immediate")          # goal-directed, proof or None
world.explain(p, ":acuity")                      # proof tree for an existing fact
```

Tier 2's grammar is deliberately small (decided 2026-08-18, PRP-003 —
`PRPs/PRP-003-python-bindings/03-decision-query.md`): one subject class per
query, subsumption-aware; conjunctive property = value equality filters
(any value of a multi-valued property); results in the store's
deterministic order; an undeclared class or property is a domain-term
error, never a silently empty result. A textual SPARQL front-end
(SELECT/ASK, FILTER, OPTIONAL, ORDER BY/LIMIT) is deferred beyond v1 and,
if built, compiles to this tier. No property paths, no federation, no
updates — permanently.

## Snapshots

```python
s = world.snapshot()          # cheap: persistent structure sharing
...mutations...
world2 = World.from_snapshot(s)
world.diff(s)                 # added/removed facts since s
```

Snapshots are immutable values. Every workflow step boundary automatically snapshots
(doc 05), which is what makes traces replayable (doc 09). Serialization: Turtle for
interchange; a compact binary form for replay (postcard-serialized, versioned
header — decided, PRP-001).

## Subscriptions

```python
world.subscribe("?p :acuity :Immediate",
                lambda binding: page_charge_nurse(binding["p"]))
```

Callbacks fire after the transaction that made the pattern true commits — post-gate,
post-inference-fixed-point, never mid-chaining. Subscriptions are how external systems
observe the world without polling; they cannot mutate it re-entrantly (mutations from
a callback are queued as new transactions).

## Transactions

Every externally-triggered mutation (perception batch, tool effect, API `assert_`,
`retract`) is a transaction: gate → change the requested base fact(s) → truth
maintenance re-chains to the fixed point of the resulting base-fact set (doc 04) →
constraint re-check on derived facts → commit (or roll back entirely) → notify
subscribers → append to trace. Partial states are never observable. A retraction
transaction chains too — it is not merely "remove one fact" — so it is subject to
the same iteration-cap tripwire as an assertion, and rolls back the whole
transaction, requested retraction included, if that cap trips.
