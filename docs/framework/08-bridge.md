# 08 · The Bridge

The Bridge is Horismos's only connection to model providers: a deliberately thin client
layer beneath the LLM boundary steps (doc 06). It is a port of the neural-bridge
design proven in sigil-lang (`ModelBackend` protocol + `ModelRegistry`, 76 tests),
with the same philosophy: **Horismos's runtime is the loop; the Bridge is a model
client, not another orchestrator.** No LangChain, no delegated agent SDKs.

## The contract

```rust
// Rust
#[async_trait]
pub trait Bridge: Send + Sync {
    async fn perceive(&self, req: PerceiveRequest) -> Result<PerceiveResult, BridgeError>;
    async fn narrate(&self, req: NarrateRequest) -> Result<NarrateResult, BridgeError>;
    async fn rank(&self, req: RankRequest) -> Result<RankResult, BridgeError>;  // ProposalStep
    fn id(&self) -> &str;
}
```

```python
# Python — structural protocol, same three capabilities
class Bridge(Protocol):
    def perceive(self, req: PerceiveRequest) -> PerceiveResult: ...
    def narrate(self, req: NarrateRequest) -> NarrateResult: ...
    def rank(self, req: RankRequest) -> RankResult: ...
```

`PerceiveResult` carries: proposed instances (JSON matching the derived schema),
call confidence, optional per-fact confidences, token/latency accounting.
The runtime — not the Bridge — owns thresholds, retries, fallbacks, validation.
Bridges stay dumb on purpose.

## Registry and routing

Model identifiers in the ontology are prefix-routed, exactly as in sigil-lang:

| Prefix | Backend | Notes |
|--------|---------|-------|
| `claude:` | Anthropic Messages API | default recommendation |
| `openai:` | OpenAI Responses/Chat | |
| `hf:` | Local HuggingFace pipeline | classification-style perception without a frontier model |
| `onnx:` | ONNX Runtime | edge/embedded |
| `python:` | Any Python callable | escape hatch, custom models |
| `jev:` | TypeSafe Jev, `text → decisions` | typed answers only — no text output, so perception and proposal but never narration |
| `mock:` | Deterministic mock | **tests and CI run on this — zero API keys** |

```python
agent = Agent(world, workflow="triage:Triage",
              bridges={"claude:claude-fable-5": AnthropicBridge.from_env()})
```

The ontology names a model (`horis:model "claude:claude-fable-5"`); deployment maps
names to configured Bridge instances. Swapping providers is a config change; the
ontology is provider-agnostic. Deployment may override an ontology-declared model
with a different configured Bridge for the same prefix (swap in a cheaper `claude:`
model, or `mock:` in tests); every trace step's `model` field records the Bridge
actually invoked (`Bridge::id()`), never merely the ontology's declared string, so
an override stays visible after the fact without a dedicated trace field.

## Provider mechanics (perception)

One call per PerceptionStep. For Anthropic:

```
messages.create(
  model    = "claude-fable-5",
  tools    = [{ name: "assert_facts", input_schema: <derived from ontology> }],
  tool_choice = { type: "tool", name: "assert_facts" },     # forced
  messages = [{ role: "user", content: <input text> }])
```

**Not a Bridge**: an Adapter that hydrates an `ExternalSnapshot`
from a FHIR server or repository is *not* a Bridge, and gains no `Bridge` implementation —
a Bridge supplies a neural *proposal* that must clear the gate; an Adapter supplies
authoritative structured state that's admitted the same way any other `api`-provenance
assertion is (doc 03, doc 06). Model providers stay outside core exactly as today; an
Adapter adds no inward relationship to `horismos-core`'s dependency graph beyond what doc
14 draws: `horismos-host`, `horismos-fhir`, and `horismos-harness` depend toward core,
never the reverse.

The provider's tool-use machinery is used *backwards* — not to let the model choose
actions, but as a constrained typed-output channel. Providers with native structured
output modes use those instead; the Bridge hides the difference. v1 ships one
calibration strategy for every provider: the forced tool call's schema carries a
`confidence` number field the model must self-report, and a result that omits it is
treated as `0.0` — failing safe into the fallback edge rather than false-accepting.
Logprob-derived confidence, where a provider exposes it, is a future per-backend
enhancement layered on the same field; the mock backend's scripted confidence makes
every test independent of either.

### A decision model: `jev:`

Not every provider behind a Bridge speaks text. TypeSafe's Jev is a
`text → decisions` model: it takes free text plus a set of typed *questions* and
returns a typed *answer* per question — a probability (`noul`), an option key
(`choice`), or an ordinal level (`score`). It has no text output channel at all.
`JevBridge` (`bridges-jev`, default-off like every TLS-carrying bridge) translates
the derived perception schema into those three primitives:

| Derived schema shape | Jev primitive |
|---|---|
| `"type": "boolean"` | `noul`, `criteria.true`/`criteria.false` from the property's description |
| `"enum": [...]` — a `horis:oneOf` or closed subclass vocabulary | `choice`, one option per member |
| an enum a deployment declares ordinal | `score`, the members in declared order |
| `"type": "number"`/`"integer"`, an open string, a date-time | **refused** |

**The refusal is the design, not a gap.** Jev cannot produce a number or a free
string, so when a derived schema asks for one the call fails with a
`BridgeError` naming the offending property, and the step takes its
`horis:fallback` edge like any other Bridge error. It does not bucket the number
into a `score` and it does not drop the property. A bucketed heart rate would be
a fabricated measurement wearing perception provenance; a numeric property
belongs on the external-snapshot path, with the real reading and the real
source. An honest refusal is worth more than a plausible wrong number.

**One class per step.** A `jev:` PerceptionStep extracts exactly one class. A
schema carrying more than one is refused by name, and a step that wants two runs
as two steps. Jev answers every question it is asked, so a two-class schema would
come back with an instance of *both* classes — including one for a class the text
never mentioned, a fabricated fact wearing perception provenance, which is the
same thing the numeric refusal exists to prevent. The `horis:extracts :Patient,
:Symptom` step used as the canonical example elsewhere in this spec is right for a
text-capable bridge; routed to `jev:` it is two steps.

**Narration is impossible here** and says so: `narrate` returns a `BridgeError`
explaining that the model has no text output channel and that narration needs a
different bridge. Nothing is stitched out of answer keys, and no empty success is
returned. Routing is per step, so a workflow whose PerceptionStep declares
`horis:model "jev:jev-1.13"` and whose NarrationStep declares a `claude:` model
reaches two different Bridges in the same run — no step ever asks Jev for prose.

Jev returns one probability per question, which is more resolution than the
single call-level `confidence` carries, so `JevBridge` fills `fact_confidence`
per proposed instance and records the per-property detail in the call's audit
metadata. Both the per-instance and the call-level number are the *weakest*
answer they cover, not an average: one coin-flip must not hide behind three
certainties when the runtime compares against `horis:confidence`.

The RDFS-plus profile has no vocabulary for "this enum is ordered" or for
question wording, and neither is invented for this bridge. A deployment supplies
both alongside the Bridge instance.

Worked end to end in `crates/horismos-bridges/examples/jev_vte_demo.rs`
(`make demo-jev`, `make demo-jev-live`): a clinical note becomes four typed
booleans with a probability each, and `examples/vte/vte.ttl`'s rules derive
`:recommendation` — which is `horis:assignedBy horis:RulesOnly`, so it never
appears in the schema the model is shown.

#### The ontology it runs on

```turtle title="examples/vte/vte.ttl — pulled from the file, in full"
--8<-- "examples/vte/vte.ttl"
```

Four `xsd:boolean` properties on `:VteCase`, three rules, and a
`:recommendation` carrying `horis:assignedBy horis:RulesOnly`. That last
declaration is what makes the demo's claim checkable, because schema
derivation drops a rules-only property before any Bridge is handed the
schema. Derived from the file above, what the model is shown is:

```json title="horismos_core::workflow::agent::extract_schema(&onto, &[:VteCase]), serialised"
{
  "properties": {
    "confidence": {
      "maximum": 1,
      "minimum": 0,
      "type": "number"
    },
    "instances": {
      "items": {
        "oneOf": [
          {
            "properties": {
              "activeCancer": { "type": "boolean" },
              "bleedingRisk": { "type": "boolean" },
              "class": { "const": ":VteCase" },
              "priorVte": { "type": "boolean" },
              "reducedMobility": { "type": "boolean" }
            },
            "required": [
              "class",
              "reducedMobility",
              "activeCancer",
              "priorVte",
              "bleedingRisk"
            ],
            "type": "object"
          }
        ]
      },
      "type": "array"
    }
  },
  "required": [
    "instances",
    "confidence"
  ],
  "type": "object"
}
```

`:recommendation` is not in it. Not marked read-only, not discouraged in the
instructions — absent. There is no field in which the model could return a
recommendation, so the one thing the rules decide is the one thing perception
cannot propose.

#### Routing a step to it

`vte.ttl` declares no workflow of its own; the demo calls `Bridge::perceive`
directly with the schema above. A workflow that wanted the same perception
routed to Jev declares the model the way any step declares one — the prefix
before the colon is what selects the Bridge, and a `jev:` step extracts a
single class. Routing is per step, so the mixed run described above is just
two declarations in the one workflow:

```turtle title="Illustrative — examples/vte/ ships no workflow; two steps, two prefixes, one run"
:ReadNote a horis:PerceptionStep ;          # → the jev: Bridge
    horis:extracts :VteCase ;
    horis:model "jev:jev-1.13" ;
    horis:confidence 0.85 ;
    horis:fallback :EscalateToClinician ;
    horis:next :Decide .

:Explain a horis:NarrationStep ;            # → a text-capable Bridge
    horis:narrates [ horis:property :recommendation ] ;
    horis:audience "charge-nurse" ;
    horis:model "claude:claude-fable-5" ;
    horis:next :Done .
```

Each prefix is bound to a Bridge instance at deployment, and the binding is
checked when the agent is built rather than when the step runs: with more than
one Bridge bound, a step naming a prefix none of them covers fails there. A
deployment that binds exactly one Bridge routes every step to it whatever
prefix the step declares — which is how a single `mock:` bridge can stand in
for both of the steps above.

#### What each primitive becomes

**A boolean becomes a `noul`.** Each of `:VteCase`'s four properties is one
question. With no deployment hint the wording is generated from the property
and class names:

```json title="One of the four questions derived from vte.ttl, unhinted"
"i0_reducedMobility": {
  "type": "noul",
  "instructions": "Does `reducedMobility` hold for this `:VteCase`?",
  "criteria": {
    "true":  "The text supports `reducedMobility` for this `:VteCase`.",
    "false": "The text does not support `reducedMobility`, or states the contrary."
  }
}
```

A `PropertyHint` replaces all three strings — the ontology has no vocabulary
for question wording, so the deployment supplies it alongside the Bridge:

```json title="The same question, with the demo's hint wording"
"i0_reducedMobility": {
  "type": "noul",
  "instructions": "Is the patient's mobility reduced relative to their usual baseline?",
  "criteria": {
    "true":  "Bed-bound, immobile, confined to a chair, or mobilising only with assistance.",
    "false": "Independently mobile at their usual baseline."
  }
}
```

The answer is a probability, read back as a typed boolean plus a confidence:
a `noul` of 0.04 is `false` at confidence 0.96.

**A closed vocabulary becomes a `choice`.** The same three-member vocabulary
`:VteRecommendation` declares, on a class where the property is not rules-only
and so does reach the bridge:

```json title="A three-member horis:oneOf, as a choice question"
"i0_recommendation": {
  "type": "choice",
  "instructions": "Which `recommendation` does the text support for this `:Case`?",
  "criteria": {
    "PharmacologicProphylaxis": ":PharmacologicProphylaxis",
    "HoldProphylaxis":          ":HoldProphylaxis",
    "NoProphylaxisIndicated":   ":NoProphylaxisIndicated"
  }
}
```

The wire key is each member's local name — a safe identifier — and the
answer is mapped back to the full member before it becomes a proposed fact.
An answer naming an option the bridge never offered is not coerced to the
nearest one; it is malformed output, and the step takes its fallback edge.
When two members' local names collide, the later one falls back to its full
member string rather than shadowing the first.

**An ordinal enumeration becomes a `score`.** Ordinality is a deployment
hint, not ontology vocabulary, and absent the hint an enum stays an unordered
`choice`. With it, the members are sent as levels in declared order:

```json title="The same shape of enum, declared ordinal"
"i0_severity": {
  "type": "score",
  "instructions": "Which `severity` does the text support for this `:Case`?",
  "criteria": ["Mild", "Moderate", "Severe"]
}
```

A `score` of 1.05 over those three levels resolves to `:Moderate`, carrying
the answer's own confidence. Where the response reports a legend mapping
level to option key, that legend decides which level was named; only a
response carrying no legend, or one that does not read as a whole, falls back
to rounding to the nearest declared index — and that fallback is recorded in
the call's audit metadata rather than failing the call. The primitive takes
2–10 levels, so a twelve-member enum declared ordinal is refused rather than
collapsed.

#### The refusals, in their own words

Every refusal below is raised while the schema is being translated, before
any request is sent. The runtime treats each one exactly as it treats low
confidence: the step takes its `horis:fallback` edge. The
`jev:typesafe/jev-1.13` each message opens with is `Bridge::id()` — the `jev:`
prefix and the model alias the bridge was configured with.

```text title="BridgeError::Refused — domain_text(), one per offending schema"
bridge jev:typesafe/jev-1.13 refused: property `heartRate` on `:Patient` has JSON Schema type `number`, and Jev answers only noul/choice/score — it cannot produce a number or a free string. Approximating it as a score would fabricate a measurement under perception provenance; supply `heartRate` through the external-snapshot path instead, or narrow its range to a closed vocabulary

bridge jev:typesafe/jev-1.13 refused: property `chiefComplaint` on `:Patient` has JSON Schema type `string`, and Jev answers only noul/choice/score — it cannot produce a number or a free string. Approximating it as a score would fabricate a measurement under perception provenance; supply `chiefComplaint` through the external-snapshot path instead, or narrow its range to a closed vocabulary

bridge jev:typesafe/jev-1.13 refused: property `admittedAt` on `:Patient` has JSON Schema type `string` (format `date-time`), and Jev answers only noul/choice/score — it cannot produce a number or a free string. Approximating it as a score would fabricate a measurement under perception provenance; supply `admittedAt` through the external-snapshot path instead, or narrow its range to a closed vocabulary

bridge jev:typesafe/jev-1.13 refused: property `grade` on `:Case` is declared ordinal but has 12 levels; Jev's score primitive takes 2-10 ordered levels
```

Declaring `heartRate` ordinal does not unlock it. There are no declared
levels to bucket a number into, and inventing some would be the fabricated
measurement the refusal exists to prevent, so the hinted schema is refused by
the same message.

Narration is refused the same way, without making a call at all:

```text title="BridgeError::Refused from narrate()"
bridge jev:typesafe/jev-1.13 refused: Jev is a text→decisions model with no text output channel, so it cannot render a proof into prose. Register a text-capable bridge for the narration step; this bridge will not stitch prose out of answer keys.
```

#### One class per step, as a failure

A schema whose `instances.items` carries two `oneOf` branches:

```json title="Refused: two classes in one perception step"
{"oneOf": [
  {"type": "object", "properties": {
      "class":     {"const": ":VteCase"},
      "priorVte":  {"type": "boolean"}}},
  {"type": "object", "properties": {
      "class":             {"const": ":Ward"},
      "isolationRequired": {"type": "boolean"}}}
]}
```

```text title="BridgeError::Refused — the classes are named"
bridge jev:typesafe/jev-1.13 refused: the perception schema extracts 2 classes (:VteCase, :Ward), and Jev answers every question it is asked — it would propose an instance of each one even for a class the text never mentions, which is a fabricated fact wearing a perception provenance. Extract one class per perception step
```

The working version is two steps, each with one `horis:extracts` class,
chained by `horis:next`: the note is read twice, and a `:Ward` instance only
exists if a step that asked about wards produced one.

#### The demo, printed

`make demo-jev` builds without `bridges-jev` and runs against `MockBridge`:
no key, no network, same output every time. The mock replays probabilities in
the shape a live `JevBridge` returns them, so the printing has one code path
either way.

```text title="make demo-jev, no provider key in the environment — verbatim, less cargo's own lines and the latency and cost columns of the two accounting lines"
Horismos VTE text→decisions demo — mock
ontology: examples/vte/vte.ttl  ·  model requested: mock:jev-1.13

:recommendation is `horis:assignedBy horis:RulesOnly`, so it is absent from the
schema the model is shown. The model proposes facts; the rules decide.
schema asks the model for: activeCancer, bleedingRisk, priorVte, reducedMobility

──────────────────────────────────────────────────────────────────────────────
Note 1
──────────────────────────────────────────────────────────────────────────────
  72-year-old admitted yesterday following a fall at home with a fractured
  right neck of femur, awaiting theatre. Mobilising only with assistance and
  largely confined to bed since admission. Past history of breast carcinoma
  treated eight years ago, in remission, no current therapy. No previous clots
  documented. Haemoglobin stable, platelets 210, not anticoagulated, no
  bleeding noted on the ward round.

  Jev proposed these typed facts, one parallel pass:
    reducedMobility    true   p=0.960  ███████████████████████·
    activeCancer       false  p=0.880  █████████████████████···
    priorVte           false  p=0.960  ███████████████████████·
    bleedingRisk       false  p=0.930  ██████████████████████··

  call confidence 0.880 (the weakest fact in the call — nothing averages away)
  instance[0] confidence 0.880

  The rules decided. The model had no vote:
    :recommendation = :PharmacologicProphylaxis   via risk-without-contraindication — "VTE risk factor without bleeding contraindication"
      ├─ any of:
      │  ├─ reducedMobility true is true   (asserted by perception)
      │  ├─ activeCancer is true   (not held)
      │  └─ priorVte is true   (not held)
      └─ bleedingRisk false is false   (asserted by perception)

  619 in / 79 out tokens  ·  model mock:jev-1.13

──────────────────────────────────────────────────────────────────────────────
Note 2
──────────────────────────────────────────────────────────────────────────────
  78yo, post-op day 2, on warfarin, bed-bound

  Jev proposed these typed facts, one parallel pass:
    reducedMobility    true   p=0.940  ███████████████████████·
    activeCancer       false  p=0.910  ██████████████████████··
    priorVte           false  p=0.720  █████████████████·······
    bleedingRisk       true   p=0.610  ███████████████·········

  call confidence 0.610 (the weakest fact in the call — nothing averages away)
  instance[0] confidence 0.610

  The rules decided. The model had no vote:
    :recommendation = :HoldProphylaxis   via bleeding-contraindication — "Bleeding contraindication — hold pharmacologic prophylaxis"
      └─ bleedingRisk true is true   (asserted by perception)

  58 in / 79 out tokens  ·  model mock:jev-1.13

──────────────────────────────────────────────────────────────────────────────
Why this is the whole point
──────────────────────────────────────────────────────────────────────────────
  model actually invoked (Bridge::id, the trace's `model` field): mock:jev-1.13
  Two notes, two different fact sets, two different rules firing — and in both
  cases the recommendation is a derivation you can read, not a model's opinion.
  Delete the bridge and the rules still run; delete the rules and there is no
  decision at all.
```

Note 1 and note 2 differ in one fact — `bleedingRisk` — and that one fact
sends the two runs down two different rules. Note 2's weakest answer is 0.610,
which is what the runtime would compare against the step's `horis:confidence`
threshold: the weakest fact in the call, not the average of the four. The
recommendation in both runs is a derivation with a proof, and in neither run
was it a field the model could have filled in.

## Operational semantics

- **Timeouts**: per-call, from the ontology (`horis:timeout` on the step) or Bridge
  default; enforced by the runtime (sigil-lang precedent: thread/task-based, portable).
- **Fallback on any Bridge error** (timeout, rate limit, refusal, malformed output):
  identical to low confidence — the step's `horis:fallback` edge. There is no
  "retry until it works" mode.
- **Accounting**: every call records model, latency, token counts, cost estimate in
  the trace — this ships in v1. `horis:budget` as a workflow-level threshold that
  turns exceeding it into a fallback-triggering condition is deferred past v1: the
  accounting the feature would consume already exists, so adding the threshold
  check later is additive, not a redesign.
- **No streaming** in v1: perception outputs are atomic typed payloads; narration is
  short. Revisit only with evidence.

## The mock backend

`mock:` is a first-class citizen, not a test hack:

```python
bridge = MockBridge(script={
    "58M, crushing chest pain": PerceiveResult(
        instances=[...], confidence=0.94),
    "*": PerceiveResult.low_confidence(),
})
```

Scriptable by input pattern, deterministic, records invocations. The entire test
suite, all examples in CI, and the τ-Bench harness scaffold run against `mock:` —
which is also the enforcement mechanism for the anti-wrapper guarantee: if any
symbolic-core test *needs* a real key, that is a build failure by policy (doc 14).
