# 07 · Tools

Tools are how a Horismos agent affects the outside world — and the sharpest place where
Horismos inverts the conventional agent stack. **The LLM never chooses a tool, never
constructs arguments, never interprets results.** Tools are declared contracts,
fired by the runtime when workflow preconditions hold.

## Declaring a tool

```turtle
:ehr.assign_queue a horis:Tool ;
    horis:takes [
        horis:arg [ horis:name "patient" ; horis:type :Patient ] ;
        horis:arg [ horis:name "queue"   ; horis:type :AcuityLevel ]
    ] ;
    horis:effect [
        horis:asserts [ horis:property :queuedAt ; horis:value horis:now ] ;
        horis:asserts [ horis:property :queueRef ; horis:type xsd:string ]
    ] ;
    horis:binding "python:ehr_client.assign_queue" ;
    horis:timeout "5s" ;
    horis:idempotent false .
```

- **`horis:takes`** — argument names and ontology types. At invocation, the runtime
  *reads argument values from the graph* (the ActionStep names the bindings). No text
  parsing is involved anywhere.
- **`horis:effect`** — the declared shape of what the tool may write back. This is a
  *contract*, not documentation: the return value is validated against it, through
  the World gate (provenance: `tool`), before any fact lands. An EHR API returning
  garbage is a rejected transaction and the ActionStep's `horis:onError` edge fires.
- **`horis:idempotent`** — drives replay semantics (doc 09): non-idempotent tools are
  never re-executed on replay; their recorded effects are substituted.

## Bindings

| Binding scheme | Example | Notes |
|----------------|---------|-------|
| `python:` | `python:ehr_client.assign_queue` | Dotted path or registered callable (`tools={...}` at Agent construction) |
| `rust:` | `rust:assign_queue` | Registered `Tool` trait impl |
| `mcp://` | `mcp://ehr-server/assign_queue` | MCP server tool; Horismos consumes the MCP schema and *narrows* it to the declared contract |
| `http:` / `https:` | `https://api…/assign` | JSON POST; response validated like any effect |

MCP deserves emphasis: the ecosystem already exposes domain tools over MCP. Horismos
does not rebuild them — it wraps them in a typed contract. Where the MCP tool's own
schema is looser than the `horis:takes`/`horis:effect` declaration, the declaration
wins: out-of-contract calls can't be made, out-of-contract results can't land.

## Adapter-mediated external commits (declared-future, PRP-008)

Under the bounded-projection pivot (`PRPs/PRP-000-vision-positioning.md`), a tool effect
that reaches an external system of record commits through an Adapter, not a bare
synchronous call, and the contract above extends with:

- **Expected source versions**: an adapter-mediated commit carries the source-version
  vector the World's projection was hydrated against; the external system rejects (or the
  adapter refuses to attempt) a commit against a resource that has since changed —
  the same fail-closed discipline as doc 03's conditional-commit semantics.
- **Remote receipt/effect reconciliation**: a commit's outcome (accepted, conflicted,
  failed) is a distinct trace state from "decision produced" — the tool's declared
  `horis:effect` describes what Horismos proposes to assert, not a guarantee the remote
  system accepted it; reconciliation records which actually happened.
- **Authorization scopes**: an adapter-mediated tool binding carries the authorization
  scope (e.g. a SMART-on-FHIR scope) it commits under, checked before the call, not
  inferred from a successful response.
- **Distributed-failure semantics**: no cross-system atomicity claim (doc 03, doc 15) —
  a multi-effect commit may partially succeed; the trace names exactly which effects
  committed and which didn't, rather than reporting one aggregate success/failure.
- Where useful, an adapter-mediated tool aligns with an existing operation shape (e.g.
  FHIR ActivityDefinition/operations) rather than inventing a parallel one — but this is
  not forced: a tool that has no natural standards analog stays a plain `horis:Tool`.

None of this changes tools without an external commit (the shipped `rust:`/`python:`
in-process bindings) — they keep the synchronous contract above unmodified.

## The candidate-artifact boundary (declared-future, coding-agent use case)

See `docs/framework/06-llm-boundaries.md` §"Candidate-artifact authoring" for the full
statement. In tools terms: a model-generated patch/file/command is never an ordinary
`horis:Tool` argument built from graph-read values — it is a typed proposal envelope that
passes through deterministic policy validation, sandboxed test execution, and a symbolic
authorization step before an `ActionStep`-like effect can land. This is captain-adopted
design (`decisions/captain-pivot-ruling.md` item 3), owned jointly by `PRPs/PRP-007-
authoring-agent.md` and `PRPs/PRP-008-standards-host-integration.md`, and **not built in
v1** — declared here so it has a named shape rather than being invented ad hoc against a
future coding-agent deadline.

## Invocation semantics

An `ActionStep` fires its tool when:

1. The step is current, and
2. every `horis:requires` pattern holds in the graph.

Then: arguments resolved from the graph → binding invoked with `horis:timeout` →
result validated against `horis:effect` → effects asserted in the step's transaction →
`horis:next`. On timeout or validation failure: `horis:onError`. Retries are explicit
(`horis:retries`, default 0 for non-idempotent tools).

## ProposalStep: the constrained escape hatch

Some choices are genuinely open — three legal rebooking options, which does the
customer prefer articulated in prose? That is a perception-like judgment, and it gets
a perception-like contract:

```turtle
:PickOption a horis:ProposalStep ;
    horis:candidates [ horis:query "…legal options pattern…" ] ;   # graph-derived
    horis:rankBy "customer preference expressed in the request" ;
    horis:model "claude:claude-fable-5" ;
    horis:confidence 0.8 ;
    horis:fallback :AskHuman ;
    horis:next :ExecuteOption .
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

| Failure | Conventional stack | Horismos |
|---------|-------------------|-------|
| Wrong tool called | LLM misjudged | Impossible — no neural tool choice |
| Malformed/hallucinated args | Parse-and-pray | Impossible — args read from typed graph |
| Tool called before its preconditions | Prompted convention | Impossible — precondition gate |
| Bad data returned by tool | Enters context silently | Rejected at effect validation, `onError` |
| Tool has side effects on retry/replay | Unmodeled | `horis:idempotent` drives policy |

The remaining honest failure class — the tool itself misbehaves in the real world in
ways its contract can't see — is logged with full trace context, which is exactly the
context an incident review needs.

**Correction (2026-08-24, bounded-projection pivot review)**: "impossible" in the table
above describes the *visible, contract-checked* surface — which tool fires, what arguments
it receives, when it's callable — not the tool implementation's full behavior. Contract
narrowing constrains what an `ActionStep` can *ask for* and what result can *land as a
fact*; it does not and cannot prove the bound implementation has no hidden side effect
outside its declared `horis:effect` (a `python:`/`http:` binding that also writes to an
unrelated system, for instance). That residual is the "tool itself misbehaves" row above,
stated precisely rather than implied away by the table's absolutist framing.
