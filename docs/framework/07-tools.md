# 07 · Tools

Tools are how a Sigil agent affects the outside world — and the sharpest place where
Sigil inverts the conventional agent stack. **The LLM never chooses a tool, never
constructs arguments, never interprets results.** Tools are declared contracts,
fired by the runtime when workflow preconditions hold.

## Declaring a tool

```turtle
:ehr.assign_queue a sigil:Tool ;
    sigil:takes [
        sigil:arg [ sigil:name "patient" ; sigil:type :Patient ] ;
        sigil:arg [ sigil:name "queue"   ; sigil:type :AcuityLevel ]
    ] ;
    sigil:effect [
        sigil:asserts [ sigil:property :queuedAt ; sigil:value sigil:now ] ;
        sigil:asserts [ sigil:property :queueRef ; sigil:type xsd:string ]
    ] ;
    sigil:binding "python:ehr_client.assign_queue" ;
    sigil:timeout "5s" ;
    sigil:idempotent false .
```

- **`sigil:takes`** — argument names and ontology types. At invocation, the runtime
  *reads argument values from the graph* (the ActionStep names the bindings). No text
  parsing is involved anywhere.
- **`sigil:effect`** — the declared shape of what the tool may write back. This is a
  *contract*, not documentation: the return value is validated against it, through
  the World gate (provenance: `tool`), before any fact lands. An EHR API returning
  garbage is a rejected transaction and the ActionStep's `sigil:onError` edge fires.
- **`sigil:idempotent`** — drives replay semantics (doc 09): non-idempotent tools are
  never re-executed on replay; their recorded effects are substituted.

## Bindings

| Binding scheme | Example | Notes |
|----------------|---------|-------|
| `python:` | `python:ehr_client.assign_queue` | Dotted path or registered callable (`tools={...}` at Agent construction) |
| `rust:` | `rust:assign_queue` | Registered `Tool` trait impl |
| `mcp://` | `mcp://ehr-server/assign_queue` | MCP server tool; Sigil consumes the MCP schema and *narrows* it to the declared contract |
| `http:` / `https:` | `https://api…/assign` | JSON POST; response validated like any effect |

MCP deserves emphasis: the ecosystem already exposes domain tools over MCP. Sigil
does not rebuild them — it wraps them in a typed contract. Where the MCP tool's own
schema is looser than the `sigil:takes`/`sigil:effect` declaration, the declaration
wins: out-of-contract calls can't be made, out-of-contract results can't land.

## Invocation semantics

An `ActionStep` fires its tool when:

1. The step is current, and
2. every `sigil:requires` pattern holds in the graph.

Then: arguments resolved from the graph → binding invoked with `sigil:timeout` →
result validated against `sigil:effect` → effects asserted in the step's transaction →
`sigil:next`. On timeout or validation failure: `sigil:onError`. Retries are explicit
(`sigil:retries`, default 0 for non-idempotent tools).

## ProposalStep: the constrained escape hatch

Some choices are genuinely open — three legal rebooking options, which does the
customer prefer articulated in prose? That is a perception-like judgment, and it gets
a perception-like contract:

```turtle
:PickOption a sigil:ProposalStep ;
    sigil:candidates [ sigil:query "…legal options pattern…" ] ;   # graph-derived
    sigil:rankBy "customer preference expressed in the request" ;
    sigil:model "claude:claude-fable-5" ;
    sigil:confidence 0.8 ;
    sigil:fallback :AskHuman ;
    sigil:next :ExecuteOption .
```

Hard properties:

- Candidates come from a **graph query** — every candidate already satisfies its
  preconditions. The LLM ranks; it cannot add options.
- The selection is recorded with confidence, and the *execution* of the chosen option
  is a normal ActionStep with normal contracts.
- Low confidence falls back — typically to a human (`WaitStep`).

If you find yourself reaching for ProposalStep to pick *whether* something is allowed,
stop: that is a rule, and it belongs in the ontology.

## Failure classes, compared

| Failure | Conventional stack | Sigil |
|---------|-------------------|-------|
| Wrong tool called | LLM misjudged | Impossible — no neural tool choice |
| Malformed/hallucinated args | Parse-and-pray | Impossible — args read from typed graph |
| Tool called before its preconditions | Prompted convention | Impossible — precondition gate |
| Bad data returned by tool | Enters context silently | Rejected at effect validation, `onError` |
| Tool has side effects on retry/replay | Unmodeled | `sigil:idempotent` drives policy |

The remaining honest failure class — the tool itself misbehaves in the real world in
ways its contract can't see — is logged with full trace context, which is exactly the
context an incident review needs.
