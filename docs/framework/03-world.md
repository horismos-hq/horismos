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
names the deriving rule; derived facts change only by re-running inference after
base-fact changes (rule provenance, the inference side's own mechanism, remains
admitted). There is no cascade in v1.

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

Every externally-triggered mutation (perception batch, tool effect, API `assert_`)
is a transaction: gate → assert base facts → forward-chain to fixed point (doc 04) →
constraint re-check on derived facts → commit (or roll back entirely) → notify
subscribers → append to trace. Partial states are never observable.
