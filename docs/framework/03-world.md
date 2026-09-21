# 03 · The World

The World is the typed instance store — an **admitted projection**, bound to one decision
package and hydration, not the deployment's enterprise memory of record (pivot adopted
2026-08-24: snapshot-scoped guarantees). It holds
facts as instances of ontology classes, and **every mutation Horismos mediates passes the
validation gate**. There is no back door *inside an evaluation or commit Horismos
mediates* — that qualifier matters: Horismos cannot see, let alone gate, a write another
client makes directly against the external system of record. Everything in this document
below — the gate, truth maintenance, snapshots, transactions — is unchanged and exact
*for one admitted projection*; what's new is stating plainly what stays true only inside
that boundary, and what an external system's own concurrent writers can invalidate.

## World as a transaction projection

For a standalone deployment (structured facts asserted directly via the API, the shipped
triage pattern), a World *is* effectively the whole record for that session — nothing
below changes for that case. For a deployment overlaying an external system of record
(FHIR, a repository), a World is instead the local, closed evaluation workspace for **one
decision epoch**:

- **Hydration** (`horismos-host::ExternalSnapshot` plus an adapter): an external read materializes a declared,
  versioned closure into a canonical `ExternalSnapshot` *before* any World transaction —
  no adapter performs a network read from inside matching, chaining, verification, or a
  transaction in progress (doc 14's anti-wrapper boundary, unchanged and enforced here
  too).
- **Admission**: the `ExternalSnapshot` is admitted as base facts through the same
  five-check gate below, all-or-nothing — a partially-admitted bundle is never observable,
  matching the existing atomic-transaction guarantee.
- **Rejection/quarantine**: a bundle that fails admission (schema mismatch, declared
  closure incomplete, unsupported external construct) is rejected in domain terms and
  never partially lands — quarantine for inspection is an adapter-level concern, not a
  World state.
- **Closure**: the projection's declared input closure states exactly which
  facts/resources were treated as complete for this decision (PRP-005's assumption
  manifest references this). Absent-because-not-fetched and absent-because-verified-not-
  present are distinct states at the adapter layer, feeding Horismos's existing pragmatic
  closed-world reading (doc 04) rather than silently collapsing into it.
- **Freshness / external-conflict invalidation**: the projection is valid only as of its
  source-version vector. A conflicting external update since hydration does not
  retroactively corrupt the local World (its gate/TMS/proof stay exact for the facts it
  was given) — it invalidates the projection's *external* validity: a proposed commit
  against a drifted source fails closed, and the projection must be refreshed (re-
  hydrated, re-chained) before a new commit is attempted.
- **Conditional commit**: a `ProposedEffectSet` produced from the World's fixed point
  commits back to the system of record only if the source still matches the recorded
  version vector (a transaction Bundle / conditional update where the external system
  supports it). The trace distinguishes "decision produced," "commit attempted," "commit
  accepted," and "commit failed/conflicted" as separate, separately observable states —
  there is no cross-system atomicity claim (doc 09, doc 15).

None of this changes the mechanics below — they are the mechanism a `Decision Package`'s
base facts pass through, whether asserted directly via the API or hydrated by an adapter.

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
inference *invalidates* what no longer holds). The captain's ruling and PRP-002's
decision record make it concrete: **recompute-on-base-mutation**, not
derivations-as-immutable-history. A
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

Tier 2's grammar is deliberately small (decided 2026-08-18, PRP-003): one subject
class per query, subsumption-aware; conjunctive property = value equality filters
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
