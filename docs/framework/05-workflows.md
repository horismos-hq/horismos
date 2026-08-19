# 05 · Workflows

A workflow is the agent loop, declared as ontology data. The Agent (runtime) is a
generic interpreter; **all** domain behavior — what to extract, when to reason, what
may execute — comes from the ontology. Changing the loop means editing `.ttl`, not code.

## Step types

| Step class | Purpose | Neural? |
|------------|---------|---------|
| `horis:PerceptionStep` | NL → typed instances via a Bridge (doc 06) | **Yes** — the main one |
| `horis:ReasoningStep` | Run/await inference until a goal pattern holds | No |
| `horis:ActionStep` | Invoke a declared Tool when preconditions hold (doc 07) | No |
| `horis:NarrationStep` | Proof tree → NL for humans (doc 06) | Yes (prose only) |
| `horis:ProposalStep` | LLM ranks precondition-legal candidates; runtime executes | Constrained |
| `horis:WaitStep` | Await external input (structured facts or human decision) | No |

## The triage workflow, complete

```turtle
:Triage a horis:Workflow ;
    horis:startsAt :Intake ;
    horis:version "1.2.0" .

:Intake a horis:PerceptionStep ;
    horis:extracts :Patient, :Symptom ;
    horis:model "claude:claude-fable-5" ;
    horis:confidence 0.85 ;
    horis:fallback :EscalateToNurse ;
    horis:next :Assess .

:Assess a horis:ReasoningStep ;
    horis:goal [ horis:property :acuity ; horis:isAssigned true ] ;
    horis:onUnreachable :EscalateToNurse ;      # goal can't hold after fixed point
    horis:next :Route .

:Route a horis:ActionStep ;
    horis:tool :ehr.assign_queue ;
    horis:requires [ horis:property :acuity ; horis:isAssigned true ] ;
    horis:next :Explain .

:Explain a horis:NarrationStep ;
    horis:narrates [ horis:property :acuity ] ;
    horis:audience "charge-nurse" ;
    horis:next :Done .

:Done a horis:Step ; horis:terminal true .

:EscalateToNurse a horis:ActionStep ;
    horis:tool :paging.notify_nurse ;
    horis:terminal true .
```

## Execution semantics

The Agent maintains exactly one current step. Per step:

1. **Snapshot** the World (trace boundary).
2. **Check `horis:requires`** (preconditions) against the graph. Unsatisfied →
   the step *cannot fire*; if it stays unsatisfiable after the current transaction's
   fixed point, follow `horis:onUnsatisfied` (or fail the run if absent —
   verification proves this can't happen for well-formed workflows, doc 10).
3. **Execute** the step body (per its type).
4. **Commit** the resulting transaction (gate → chain → fixed point, doc 03).
5. **Advance** along `horis:next` (or the fallback edge the step's contract selected).
6. **Terminal step** → the run ends; the run's `outcome` is computed (below).

Key property: between steps there is **no implicit state** — everything the next step
can see is in the World. This is what makes runs replayable and workflows composable.

## Branching

Deterministic branching is data:

```turtle
:AfterAssess a horis:ChoiceStep ;
    horis:branches (
        [ horis:when [ horis:property :acuity ; horis:is :Immediate ] ; horis:next :FastTrack ]
        [ horis:when [ horis:property :acuity ; horis:is :Urgent ]    ; horis:next :Route ]
    ) ;
    horis:otherwise :Route .
```

Branch conditions use the same 11 operators as rules; evaluation order is document
order; first match wins. For choices that are *not* decidable from the graph — genuine
preference among legal options — use a `ProposalStep` (doc 07), never a ChoiceStep
with fake conditions.

## Fallbacks and failure

Every step type has a mandatory failure edge where failure is possible:

| Failure | Edge | Notes |
|---------|------|-------|
| Perception below confidence / invalid after retries | `horis:fallback` | **Required** on every PerceptionStep (load error if absent) |
| Reasoning goal unreachable at fixed point | `horis:onUnreachable` | Required |
| Tool error / effect validation failure | `horis:onError` | Required on ActionSteps |
| Precondition never satisfiable | `horis:onUnsatisfied` | Optional if verification proves satisfiability |

There are no exceptions escaping a run: every run ends at a terminal step, and
verification proves that statically (doc 10).

## Run outcomes

A finished run reports:

```python
result.decision      # convention: the property named by workflow horis:decides, if set
result.terminal      # which terminal step ended the run (:Done vs :EscalateToNurse)
result.goal_reached  # all ReasoningStep goals satisfied en route
result.proof         # proof tree(s) for the decision property
result.trace         # the full trajectory (doc 09)
```

`horis:decides` names exactly one decision property per workflow
(`:Triage horis:decides :acuity`). `result.decision` is `{property, value}` for
that property when the run ends holding a value for it, and absent otherwise —
matching doc 09's `outcome.decision`, which is one pair, not a list.

## Composition

Workflows may invoke workflows: `horis:SubflowStep` with `horis:workflow :OtherFlow`.
The subflow runs in the same World (same transaction discipline), its trace nests, and
its terminal outcome selects the parent's next edge. One level of nesting is supported
in v1: a workflow reachable through a `horis:SubflowStep` may not itself declare a
`horis:SubflowStep` — depth is capped at one, checked at load, never a runtime
surprise.
