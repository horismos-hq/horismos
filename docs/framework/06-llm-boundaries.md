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

### The structured-input bypass — the primary path for standards-hydrated input

Perception is for language. Structured sources skip it entirely:

```python
world.assert_(Patient(heart_rate=128, ...))      # provenance: api
```

Same gate, same downstream behavior. A deployment fed only by monitors and EHR feeds
runs with **no Bridge configured at all** — CI enforces this configuration works
(the anti-wrapper gate, doc 14).

**Under the bounded-projection pivot**, this
bypass is promoted to the **primary structured-input path** for a standards-hydrated
deployment: an Adapter's `ExternalSnapshot` (FHIR resources, repository/tool
observations — doc 14 § "Two-boundary data flow") is admitted exactly this
way, with provenance `api`, never through perception. Mapping a FHIR resource or repo
observation into ontology facts is **deterministic adapter code, checked before runtime —
never a perception shortcut**. An LLM never infers what a structured standard resource
means; that would smuggle judgment into what is supposed to be a mechanical conversion,
exactly the discipline this document already holds perception itself to.

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

## Where the neural surface ends

Everything not listed in this document is symbolic. In particular, these are **not**
LLM operations in Horismos: rule evaluation, workflow transitions, tool selection, tool
argument construction, precondition checks, conflict resolution, verification,
retraction, queries. If a future feature proposes adding a neural step type, it must
come with this document's level of contract: typed input, typed output, gate,
fallback, trace visibility.

## Candidate-artifact authoring

A useful coding agent necessarily generates novel source text, patches, commands, and
sometimes filenames — content an ordinary `ActionStep` cannot fit, because the current
contract says the model never constructs tool arguments (doc 07). Pretending free-form
code generation is a normal ActionStep would weaken the neural-proposes/symbolic-decides
principle by stealth. The honest extension, captain-adopted alongside the bounded-
projection pivot and owned jointly by PRP-007 (vocabulary) and PRP-008 (runtime):
a **candidate-artifact boundary**. Model output is a typed
patch/file/command *proposal envelope*, not a tool argument; deterministic policy
validates target paths, base hashes, syntax, and permissions; a sandbox executes required
tests/checks; only then does a symbolic authorization step permit application. Semantic
correctness of the generated content remains a review/test claim — the boundary proves the
proposal was policy-legal and passed its declared checks, never that the code is right.
`horismos-host::artifact` implements the runtime half of this boundary; PRP-007 retains the authoring-side
envelope vocabulary and generation semantics. The runtime consumes the envelope as opaque
typed data rather than inventing an `eval`-shaped escape hatch.
