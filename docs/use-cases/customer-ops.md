# Customer operations

## The scene

A customer writes in: *"I booked yesterday but my plans changed, can I move my flight to next month?"* An airline support agent has to answer with the fare rules, not with sympathy. This is the shape of the τ-bench airline tasks, and the measured record is sobering: in τ²-bench airline, 78% of observed failures were **silent wrong-state failures** — the agent changed or refunded something it should not have, with no tool error to flag it. Deterministic, read-only pre-execution gates that inspect the proposed call and current state before allowing a write raised task success from 29.6% to 42.0% ([arXiv:2607.07405](https://arxiv.org/abs/2607.07405)).

That paper states Horismos's scope in its own words: such gates "do not guarantee task success, but they can deterministically prevent a known class of silent policy-violating writes."

## The overlay

The model reads the message and proposes typed facts — fare class, hours since booking, hours to departure, segments flown. The rules decide what the fare allows. A `ProposalStep` can then let the model rank the legal rebooking options by what the customer asked for; it cannot invent one ([Tools](../framework/07-tools.md)).

```turtle
--8<-- "examples/gallery/rebooking.ttl"
```

## The decision, with proof

```python
from horismos import HorismosValidationError, Ontology, World
from horismos.examples import path

onto = Ontology.load(path("gallery/rebooking.ttl"))
world = World(onto)

def request(subject: str, **facts: object) -> None:
    ref = world.assert_(":ChangeRequest", subject=subject, **facts)
    proof = world.explain(ref, ":eligibility")
    print(proof.render() if proof else f"{subject}: no ruling — hand to an agent")

request(":req-1", fare_class=":BasicEconomy", hours_since_booking=6,
        hours_to_departure=400, segments_flown=0)
request(":req-2", fare_class=":BasicEconomy", hours_since_booking=72,
        hours_to_departure=400, segments_flown=0)
request(":req-3", fare_class=":Economy", hours_since_booking=200,
        hours_to_departure=30, segments_flown=0)
```

```text
:eligibility = :FreeChange   via freeWithin24h — "Booked within the last 24 hours and departure is at least 7 days out — free change"
  ├─ none of:
  │  └─ segmentsFlown exceeds 0   (not held)
  ├─ hoursSinceBooking 6 atMost 24   (asserted by the api)
  └─ hoursToDeparture 400 atLeast 168   (asserted by the api)

:eligibility = :NotEligible   via basicLocked — "Basic economy fares cannot be changed after the 24-hour window"
  ├─ any of:
  │  ├─ none of:
  │  │  └─ hoursSinceBooking atMost 24   (not held)
  │  └─ none of:   (not held)
  │     └─ hoursToDeparture 400 atLeast 168   (asserted by the api)
  └─ fareClass :BasicEconomy is :BasicEconomy   (asserted by the api)

:eligibility = :ChangeWithFee   via economyFee — "Economy fare, more than 24 hours before departure — change permitted with fee"
  ├─ none of:
  │  └─ segmentsFlown exceeds 0   (not held)
  ├─ any of:
  │  ├─ none of:
  │  │  └─ hoursSinceBooking atMost 24   (not held)
  │  └─ none of:
  │     └─ hoursToDeparture atLeast 168   (not held)
  ├─ fareClass :Economy is :Economy   (asserted by the api)
  └─ hoursToDeparture 30 atLeast 24   (asserted by the api)
```

The first customer gets a free change because the 24-hour rule outranks the basic-economy lock — and the proof says exactly that, which is what the agent reads back. The narration step renders it in the customer's language; the ruling never changes in translation because the prose is generated only from the proof.

A ticket that has been flown cannot be changed whatever the fare, inside 24 hours economy is locked, and no matter how persuasive the message, the model cannot assert eligibility:

```python
request(":req-4", fare_class=":Economy", hours_since_booking=200,
        hours_to_departure=10, segments_flown=0)
request(":req-5", fare_class=":Business", hours_since_booking=500,
        hours_to_departure=2, segments_flown=1)
try:
    world.assert_(":ChangeRequest", subject=":req-6",
                  fare_class=":Economy", eligibility=":FreeChange")
except HorismosValidationError as e:
    print(f"[{e.constraint}] {e}")
```

```text
:eligibility = :NotEligible   via economyLate — "Economy fare inside 24 hours of departure — no changes"
  ├─ fareClass :Economy is :Economy   (asserted by the api)
  └─ hoursToDeparture 10 below 24   (asserted by the api)

:eligibility = :NotEligible   via alreadyFlown — "Travel has begun — the ticket can no longer be changed or refunded"
  └─ segmentsFlown 1 exceeds 0   (asserted by the api)

[provenance_refused] eligibility may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from the api
```

This is the prompt-injection story in miniature. A message engineered to make the model "decide" the customer is eligible has nowhere to land: the decision property is omitted from the perception schema entirely and refused at the gate if it arrives anyway. The worst an adversarial message can do is propose a wrong-but-schema-valid *fact* (a false booking time), which is a data-integrity risk bounded by the confidence threshold and the fallback — stated as such in [Security & Trust](../framework/15-security.md).

## Why verification matters here

Fare rules are the textbook case for conflict-freedom: `freeWithin24h` and `basicLocked` overlap by design and priority resolves them, but `economyFee` and `economyLate` must be disjoint, and a rule set with hundreds of fare conditions accumulates overlaps nobody intended. The check named five overlapping pairs in these seven rules — `alreadyFlown` against three of them, `freeWithin24h` against two more — each with its region and the rule that wins. All five were already resolved correctly by priority, and all five are now stated as conditions: the losing rules carry explicit `segmentsFlown` and 24-hour-window guards, so the rule set reads the way it behaves. Completeness reported the customer no rule covered, and it was the one this page had guessed at: a business fare that has been flown is covered — by `alreadyFlown` — but an economy fare with `hoursToDeparture` unassigned was not, because `economyFee` needs `atLeast 24` and `economyLate` needs `below 24`. The default is now `:ChangeWithFee`, via `departureTimeUnknownEconomy`. That is a policy choice the check forced into the open rather than one it made: the only condition that would block the change is the one that is unknown, the late cutoff is the exception rather than the general rule, and denying a passenger because the carrier failed to record its own departure time is the wrong way to resolve a gap in the carrier's own data. Termination is trivial here and proven anyway.

## Status and honest edges

- Runs today with structured input. The perception step that turns the customer's message into these facts is the Rust agent loop's `PerceptionStep` (see [the clinical quickstart's mock-bridge run](../quickstart-clinical.md#9-run-the-whole-agent-loop-mock-bridge-no-network-no-key)); the `ProposalStep` that ranks legal options is parsed and traced today but its execution is deferred in the current PRP-004 build.
- τ-bench airline is the framework's chosen public benchmark (PRP-000). No Horismos number exists yet; the harness scaffold is a PRP-006 deliverable. The +12.4 pp figure above is the cited paper's, for its own deterministic gates — evidence for the mechanism class, not a Horismos result.
- Fare rules here are illustrative and much simpler than any carrier's.
