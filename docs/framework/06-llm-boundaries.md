# 06 · LLM Boundaries

The entire neural surface of a Horismos deployment is two step types, both declared in
the ontology, both with typed contracts, both fully traced. This document is the
contract for each.

## PerceptionStep: NL → typed instances

### What the runtime does

```
1. Collect target classes from horis:extracts        (:Patient, :Symptom)
2. Derive a JSON Schema from those classes           (see mapping below)
3. ONE Bridge call: input text + schema, output forced into the schema
4. Confidence check:   result.confidence ≥ horis:confidence ?
       no  → follow horis:fallback edge. The LLM never guesses.
5. Validation gate:    every proposed instance through the World gate
       violations → structured feedback → bounded retry (horis:retries, default 2)
       still failing → horis:fallback
6. Assert (provenance: perception, confidence attached per fact)
```

### Schema derivation (ontology → JSON Schema)

| Ontology | JSON Schema |
|----------|-------------|
| Class with properties | `object` with `properties`; `required` from cardinality `1..` |
| `xsd:decimal` + `horis:min/max` | `number` with `minimum`/`maximum` |
| `horis:unit "bpm"` | `description` carries the unit ("value in bpm") |
| `horis:oneOf ( … )` | `enum` of IRIs (local names) |
| Subclass tree under an extract target | `enum` of the tree's local names — the LLM picks from the *closed* symptom vocabulary, it cannot invent `:Heartburn-ish` |
| `horis:assignedBy horis:RulesOnly` | **omitted from the schema entirely** — unrepresentable in the LLM's output |
| `horis:pattern` | `pattern` |

The schema is derived fresh from the loaded ontology at load time and cached; it is
never hand-written and cannot drift from the ontology.

### Confidence

The Bridge reports one confidence per call (provider-specific derivation, doc 08;
the mock backend makes it scriptable for tests). Per-fact confidences are attached
when the provider supports them, else the call confidence is inherited by each fact.
Confidence is **metadata, not truth**: it gates acceptance and is recorded in trace
and proof provenance; it never weights inference. There is no fuzzy logic in Horismos.

### What perception cannot do

- Set `horis:RulesOnly` properties (schema omission + gate enforcement, two layers).
- Create classes, properties, or rules.
- Trigger tools or transitions — it returns facts; the workflow decides what's next.
- Retry unboundedly or "try a different phrasing" — bounded retries, then fallback.

### The structured-input bypass

Perception is for language. Structured sources skip it entirely:

```python
world.assert_(Patient(heart_rate=128, ...))      # provenance: api
```

Same gate, same downstream behavior. A deployment fed only by monitors and EHR feeds
runs with **no Bridge configured at all** — CI enforces this configuration works
(the anti-wrapper gate, doc 14).

## NarrationStep: proof → NL

The inverse transducer: renders a deterministic result for a human audience.

```turtle
:Explain a horis:NarrationStep ;
    horis:narrates [ horis:property :acuity ] ;
    horis:audience "charge-nurse" ;
    horis:model "claude:claude-fable-5" ;
    horis:next :Done .
```

Contract:

- **Input** to the model: the *rendered proof tree* (doc 04) + audience + style rules.
  Not the raw world, not the conversation, not the NL input.
- **Output**: prose. It is attached to the trace as `narration`, clearly marked as
  generated, and **never** asserted into the World.
- The proof remains the authoritative explanation; narration is its presentation.
  Auditors read the proof; humans in the loop read the narration; they must agree
  because one is generated from the other — and *only* from the other.
- Narration failure (timeout, refusal) is non-fatal by default: the step falls back
  to the mechanical `proof.render()` text. `horis:required true` makes it fatal.
  **[DECIDE: default non-fatal confirmed? — PRP-004]**

## Where the neural surface ends

Everything not listed in this document is symbolic. In particular, these are **not**
LLM operations in Horismos: rule evaluation, workflow transitions, tool selection, tool
argument construction, precondition checks, conflict resolution, verification,
retraction, queries. If a future feature proposes adding a neural step type, it must
come with this document's level of contract: typed input, typed output, gate,
fallback, trace visibility.
