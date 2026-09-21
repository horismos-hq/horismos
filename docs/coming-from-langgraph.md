# Coming from LangGraph

You already ship agents. LangGraph (or the OpenAI and Anthropic agent SDKs, or a coding harness) gives you orchestration, streaming, checkpoints, retries, and a huge ecosystem. Keep all of it. This page shows the same small decision written both ways, says plainly where LangGraph is the right tool, and shows the ten lines that let a graph node call the kernel.

## The same decision, twice

The decision: *a patient with a heart rate over 120 and any kind of chest pain is immediate acuity.*

=== "LangGraph, as most teams write it"

    ```python title="Illustrative — langgraph is not a dependency of this repository"
    from typing import TypedDict
    from langgraph.graph import StateGraph, END

    class TriageState(TypedDict):
        note: str
        heart_rate: int | None
        symptoms: list[str]
        acuity: str | None

    def extract(state: TriageState) -> TriageState:
        # LLM call: free text -> fields. Structured output helps; nothing checks
        # that heart_rate is a heart rate, or that "chest tightness" is chest pain.
        ...

    def decide(state: TriageState) -> TriageState:
        # Usually a second LLM call with the policy in the prompt; sometimes an
        # if-chain. Either way "why did it pick Immediate?" is a transcript.
        ...

    def route(state: TriageState) -> TriageState:
        # The model emits the tool call and its arguments.
        ...

    g = StateGraph(TriageState)
    g.add_node("extract", extract); g.add_node("decide", decide); g.add_node("route", route)
    g.set_entry_point("extract")
    g.add_edge("extract", "decide"); g.add_edge("decide", "route"); g.add_edge("route", END)
    app = g.compile()
    ```

    The graph is fine. The problem is that the policy lives in `decide`'s prompt, the state is whatever the last node returned, and the audit trail is the transcript.

=== "Horismos"

    The policy is data, in a file your domain expert can review:

    ```turtle
    :acuity a rdf:Property ;
        rdfs:domain :Patient ; rdfs:range :AcuityLevel ;
        horis:cardinality "0..1" ;
        horis:assignedBy horis:RulesOnly .          # no model, tool, or caller sets this

    :r1 a horis:Rule ;
        horis:priority 10 ;
        horis:when [ horis:all (
            [ horis:subject :Patient ; horis:property :heartRate  ; horis:exceeds 120 ]
            [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ] ) ] ;
        horis:then [ horis:property :acuity ; horis:value :Immediate ] ;
        horis:explain "Tachycardia with chest pain — immediate track" .
    ```

    And the decision is one gated transaction with a proof:

    ```python
    from horismos import Ontology, World

    onto = Ontology.load("examples/triage/triage.ttl",
                         imports=["examples/triage/snomed-ct-subset.ttl"])

    def decide_acuity(*, heart_rate: int, symptoms: list[str]) -> tuple[str | None, str | None]:
        """One decision per call: a fresh World, one gated transaction, a proof or None."""
        world = World(onto)
        p = world.assert_(":Patient", heart_rate=heart_rate, has_symptom=symptoms)
        acuity = world.value(p, ":acuity")
        proof = world.explain(p, ":acuity")
        return (acuity.curie if acuity else None, proof.render() if proof else None)

    print(decide_acuity(heart_rate=128, symptoms=[":CrushingChestPain"]))
    print(decide_acuity(heart_rate=96, symptoms=[":CrushingChestPain"]))
    ```

    ```text
    (':Immediate', ':acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"\n  ├─ heartRate 128 exceeds 120   (asserted by the api)\n  └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (subsumption)\n')
    (':Standard', ':acuity = :Standard   via r3 — "Heart rate at most 100 — standard track"\n  └─ heartRate 96 atMost 100   (asserted by the api)\n')
    ```

    Both calls return a value *and* the clause that produced it. The second one is the
    interesting case: a heart rate of 96 with chest pain is not an emergency, and the
    rule that says so is named in the proof rather than left implicit.

    The signature still returns `str | None`, and that is not dead code. `verify()`
    proves this rule set assigns `:acuity` for every input in its declared closure, so
    `None` is unreachable *here* — but a rule set that has not been verified, or one
    loaded with `allow_incomplete=True`, can still leave the property unassigned. When
    that happens it is a visible branch, not a confident-sounding guess.

## Wrap, don't replace

`decide_acuity` is a plain Python function. Drop it into your graph as the decision node and let the graph keep doing what it is good at:

```python title="Your graph, with the decision node swapped — illustrative"
def decide(state: TriageState) -> TriageState:
    acuity, proof = decide_acuity(heart_rate=state["heart_rate"], symptoms=state["symptoms"])
    return {**state, "acuity": acuity, "proof": proof}

def after_decide(state: TriageState) -> str:
    return "route" if state["acuity"] else "escalate"   # None means no rule fired

g.add_node("decide", decide)
g.add_conditional_edges("decide", after_decide, {"route": "route", "escalate": "escalate"})
```

What changed: the model's extraction now has to clear a typed gate before it counts as a fact; the decision is made by a rule you can diff and `verify()` for completeness and conflicts before you deploy it; the proof travels in state to whoever needs to explain the call; and "no rule fired" is a visible branch rather than a confident-sounding guess. What did not change: your graph, your checkpointer, your streaming, your tools.

!!! note "No adapter package yet"
    A LangGraph node / agent-SDK tool adapter was deliberately deferred past v1 — see the [roadmap](roadmap.md). Today the glue is the ten lines above, and you own it. That is also why it works with any orchestrator: the kernel is whole-input-in, whole-outcome-out.

## Concept mapping

| You have in LangGraph | Horismos equivalent | What is different |
|---|---|---|
| `State` (a `TypedDict`) | `World` over an `Ontology` | Every write is schema-, constraint-, cardinality- and provenance-checked; rejections are domain sentences |
| A node | A step (`PerceptionStep`, `ReasoningStep`, `ActionStep`, `NarrationStep`, …) | Only perception and narration may call a model; the rest is deterministic |
| Conditional edges | `ChoiceStep` branches, or simply rules | Conditions use the same 11 operators as rules; first match wins, deterministically |
| A tool node | `horis:Tool` contract + `ActionStep` | The runtime fires the tool when preconditions hold; arguments are read from the graph; the result is validated against the declared effect |
| Checkpoints | `Snapshot` | O(1), immutable, versioned binary form; every step boundary snapshots automatically |
| `interrupt()` / human-in-the-loop | `WaitStep`, and every fallback edge | `WaitStep` execution is a v1.1 item; the fallback edges (`fallback`, `onUnreachable`, `onError`) are mandatory today |
| LangSmith trace | `horismos.trace/2` | Typed and replayable: model id, confidence, threshold, every asserted fact, every rule fired with its proof, `loss_mask` per step |

## Where each tool is right

| Keep LangGraph (or your SDK) for | Add Horismos for |
|---|---|
| Orchestration: fan-out, retries, cycles, streaming tokens to a UI | The decision itself: typed facts in, rule-derived conclusion and proof out |
| Fast iteration on prompts and flows | A policy artifact a domain expert can read, diff, sign off, and verify for completeness and conflicts |
| Persistence across processes, long-running sessions | Snapshot-scoped determinism: same facts, same ontology, same proof, byte for byte |
| Provider breadth and the model ecosystem | Two declared model boundaries with a mandatory fallback; the kernel runs with the model deleted |
| Human-in-the-loop UX, approvals UI | The fact that "no rule fired" is a first-class outcome the UI can route |

Horismos does not stream, does not persist runs across processes today, has no UI, and ships one provider bridge (OpenRouter) plus a mock. If your risk is orchestration, it will not help you. If your risk is *"the model felt confident"*, it is the missing piece.

## Where the model still lives

Two places, both declared in the ontology, both traced:

- **Perception** — text → typed instances. One schema-forced call per `PerceptionStep`; the schema is derived from the ontology, so decision properties do not even have a slot in the model's output; a confidence threshold and a mandatory `horis:fallback` edge decide what happens when the model is unsure.
- **Narration** — proof → prose for a named audience. Never written back into the world. Auditors read the proof; humans read the narration; they agree because one is generated only from the other.

A third, constrained slot exists for genuine preferences among *legal* options: a `ProposalStep` lets the model rank candidates a graph query produced. It cannot add one. See [Tools](framework/07-tools.md).

## Where next

- [Quickstart](quickstart.md) for the full symbolic walk-through and the mock-bridge agent run.
- [Governing a coding agent](governing-a-coding-agent.md) if your agent writes code.
- [Concepts](framework/01-concepts.md) for the comparison table in the framework's own words.
