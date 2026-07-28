# 05 · Workflows

A workflow is the agent loop, declared as ontology data. The Agent (runtime) is a
generic interpreter; **all** domain behavior — what to extract, when to reason, what
may execute — comes from the ontology. Changing the loop means editing `.ttl`, not code.

## Step types

| Step class | Purpose | Neural? |
|------------|---------|---------|
| `sigil:PerceptionStep` | NL → typed instances via a Bridge (doc 06) | **Yes** — the main one |
| `sigil:ReasoningStep` | Run/await inference until a goal pattern holds | No |
| `sigil:ActionStep` | Invoke a declared Tool when preconditions hold (doc 07) | No |
| `sigil:NarrationStep` | Proof tree → NL for humans (doc 06) | Yes (prose only) |
| `sigil:ProposalStep` | LLM ranks precondition-legal candidates; runtime executes | Constrained |
| `sigil:WaitStep` | Await external input (structured facts or human decision) | No |

## The triage workflow, complete

```turtle
:Triage a sigil:Workflow ;
    sigil:startsAt :Intake ;
    sigil:version "1.2.0" .

:Intake a sigil:PerceptionStep ;
    sigil:extracts :Patient, :Symptom ;
    sigil:model "claude:claude-fable-5" ;
    sigil:confidence 0.85 ;
    sigil:fallback :EscalateToNurse ;
    sigil:next :Assess .

:Assess a sigil:ReasoningStep ;
    sigil:goal [ sigil:property :acuity ; sigil:isAssigned true ] ;
    sigil:onUnreachable :EscalateToNurse ;      # goal can't hold after fixed point
    sigil:next :Route .

:Route a sigil:ActionStep ;
    sigil:tool :ehr.assign_queue ;
    sigil:requires [ sigil:property :acuity ; sigil:isAssigned true ] ;
    sigil:next :Explain .

:Explain a sigil:NarrationStep ;
    sigil:narrates [ sigil:property :acuity ] ;
    sigil:audience "charge-nurse" ;
    sigil:next :Done .

:Done a sigil:Step ; sigil:terminal true .

:EscalateToNurse a sigil:ActionStep ;
    sigil:tool :paging.notify_nurse ;
    sigil:terminal true .
```

## Execution semantics

The Agent maintains exactly one current step. Per step:

1. **Snapshot** the World (trace boundary).
2. **Check `sigil:requires`** (preconditions) against the graph. Unsatisfied →
   the step *cannot fire*; if it stays unsatisfiable after the current transaction's
   fixed point, follow `sigil:onUnsatisfied` (or fail the run if absent —
   verification proves this can't happen for well-formed workflows, doc 10).
3. **Execute** the step body (per its type).
4. **Commit** the resulting transaction (gate → chain → fixed point, doc 03).
5. **Advance** along `sigil:next` (or the fallback edge the step's contract selected).
6. **Terminal step** → the run ends; the run's `outcome` is computed (below).

Key property: between steps there is **no implicit state** — everything the next step
can see is in the World. This is what makes runs replayable and workflows composable.

## Branching

Deterministic branching is data:

```turtle
:AfterAssess a sigil:ChoiceStep ;
    sigil:branches (
        [ sigil:when [ sigil:property :acuity ; sigil:is :Immediate ] ; sigil:next :FastTrack ]
        [ sigil:when [ sigil:property :acuity ; sigil:is :Urgent ]    ; sigil:next :Route ]
    ) ;
    sigil:otherwise :Route .
```

Branch conditions use the same 11 operators as rules; evaluation order is document
order; first match wins. For choices that are *not* decidable from the graph — genuine
preference among legal options — use a `ProposalStep` (doc 07), never a ChoiceStep
with fake conditions.

## Fallbacks and failure

Every step type has a mandatory failure edge where failure is possible:

| Failure | Edge | Notes |
|---------|------|-------|
| Perception below confidence / invalid after retries | `sigil:fallback` | **Required** on every PerceptionStep (load error if absent) |
| Reasoning goal unreachable at fixed point | `sigil:onUnreachable` | Required |
| Tool error / effect validation failure | `sigil:onError` | Required on ActionSteps |
| Precondition never satisfiable | `sigil:onUnsatisfied` | Optional if verification proves satisfiability |

There are no exceptions escaping a run: every run ends at a terminal step, and
verification proves that statically (doc 10).

## Run outcomes

A finished run reports:

```python
result.decision      # convention: the property named by workflow sigil:decides, if set
result.terminal      # which terminal step ended the run (:Done vs :EscalateToNurse)
result.goal_reached  # all ReasoningStep goals satisfied en route
result.proof         # proof tree(s) for the decision property
result.trace         # the full trajectory (doc 09)
```

**[DECIDE: `sigil:decides` — single decision property per workflow vs. explicit
outcome mapping per terminal step — PRP-004]**

## Composition

Workflows may invoke workflows: `sigil:SubflowStep` with `sigil:workflow :OtherFlow`.
The subflow runs in the same World (same transaction discipline), its trace nests, and
its terminal outcome selects the parent's next edge. One level of nesting is supported
in v1. **[DECIDE: recursion ban vs. depth cap — PRP-004]**
