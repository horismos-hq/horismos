---
title: Horismos
hide:
  - navigation
  - toc
---

<div class="horis-hero" markdown>

# Neural proposes. Symbolic decides.

<p class="horis-tagline">Horismos is an ontology-first decision kernel that sits <em>around</em> the agent you already run. Your rules make the call. The model translates language into typed facts. Every conclusion ships with a proof — and the kernel still runs with the model deleted.</p>

[Quickstart — 10 minutes, zero API keys](quickstart.md){ .md-button .md-button--primary }
[Coming from LangGraph](coming-from-langgraph.md){ .md-button }
[Governing a coding agent](governing-a-coding-agent.md){ .md-button }

<div class="horis-chips" markdown="span">
<span>Day one: an opt-in bridge for [TypeSafe AI's Jev](#when-the-model-is-the-boundary) — typed answers, never prose</span>
<span>Rust core · Python bindings</span>
<span>Standard Turtle, RDFS-plus</span>
<span>MIT OR Apache-2.0</span>
<span>Pre-alpha — see the [roadmap](roadmap.md)</span>
</div>

<div class="horis-flowcard">
<div class="horis-flowcap"><b>Where the model is allowed to act</b> <span>Two declared boundaries. Nothing else calls a model.</span></div>
<svg class="horis-flow-defs" width="0" height="0" aria-hidden="true" focusable="false">
  <style>
    /* Every var() carries a literal fallback read from docs/stylesheets/extra.css, and
     * every rule is declared twice — once for the default scheme, once again under
     * [data-md-color-scheme="slate"]. `fill` and `stroke` take their *initial* value
     * when a custom property is invalid at computed-value time — black and none
     * respectively — so an unguarded var() turns this diagram into solid black boxes
     * on any hiccup that keeps extra.css from applying. extra.css is the one unhashed
     * stylesheet on the site, so it is the one that can go missing while Material's
     * content-hashed bundles are still served from cache. Those cached bundles keep
     * defining and applying the slate scheme, so a dark-mode reader reaches these
     * fallbacks on a dark page: one scheme's literals are not enough. The attribute
     * selector, not a prefers-color-scheme media query, is what also tracks Material's
     * manual light/dark toggle. */
    .hf-in   { fill: var(--horis-paper-2, #fafbfc); stroke: var(--horis-line, #e4e7ec); }
    .hf-neu  { fill: var(--horis-neural-soft, #fdf2e2); stroke: var(--horis-neural, #96590a); stroke-opacity: .5; }
    .hf-gate { fill: var(--horis-gate-soft, #dff0ed); stroke: var(--horis-gate, #12736a); stroke-opacity: .5; }
    .hf-sym  { fill: var(--horis-symbolic-soft, #eaeefc); stroke: var(--horis-symbolic, #3b5bdb); stroke-opacity: .5; }
    .hf-out  { fill: var(--md-default-bg-color, #ffffff); stroke: var(--horis-gate, #12736a); stroke-width: 1.8; }
    .hf-k    { font-family: var(--horis-mono, ui-monospace, "SF Mono", Menlo, Consolas, monospace);
               font-size: 9.5px; font-weight: 650; letter-spacing: 1px; }
    .hf-k-neu  { fill: var(--horis-neural, #96590a); }
    .hf-k-gate { fill: var(--horis-gate, #12736a); }
    .hf-k-sym  { fill: var(--horis-symbolic, #3b5bdb); }
    .hf-t    { font-family: inherit; font-size: 14px; font-weight: 650; fill: var(--horis-ink, #15171c); }
    .hf-s    { font-family: inherit; font-size: 11.5px; fill: var(--horis-ink-3, #5b6472); }
    .hf-lead { font-family: inherit; font-size: 12.5px; font-weight: 600; fill: var(--horis-ink-2, #3d434f); }
    .hf-a    { stroke: var(--horis-ink-3, #5b6472); stroke-width: 1.6; fill: none; marker-end: url(#horisFlowArrow); }
    .hf-r    { stroke: var(--horis-refuse, #b4382e); stroke-width: 1.5; fill: none; stroke-dasharray: 4 3;
               marker-end: url(#horisFlowArrowRefuse); }
    .hf-rt   { font-family: inherit; font-size: 11.5px; fill: var(--horis-refuse, #b4382e); }
    .hf-mk   { fill: var(--horis-ink-3, #5b6472); }
    .hf-mkr  { fill: var(--horis-refuse, #b4382e); }

    [data-md-color-scheme="slate"] .hf-in   { fill: var(--horis-paper-2, #171a21); stroke: var(--horis-line, #2e3542); }
    [data-md-color-scheme="slate"] .hf-neu  { fill: var(--horis-neural-soft, #33291a); stroke: var(--horis-neural, #e0a458); }
    [data-md-color-scheme="slate"] .hf-gate { fill: var(--horis-gate-soft, #152f2c); stroke: var(--horis-gate, #4fbfaf); }
    [data-md-color-scheme="slate"] .hf-sym  { fill: var(--horis-symbolic-soft, #1f2640); stroke: var(--horis-symbolic, #8ba3f5); }
    [data-md-color-scheme="slate"] .hf-out  { fill: var(--md-default-bg-color, #13161c); stroke: var(--horis-gate, #4fbfaf); }
    [data-md-color-scheme="slate"] .hf-k-neu  { fill: var(--horis-neural, #e0a458); }
    [data-md-color-scheme="slate"] .hf-k-gate { fill: var(--horis-gate, #4fbfaf); }
    [data-md-color-scheme="slate"] .hf-k-sym  { fill: var(--horis-symbolic, #8ba3f5); }
    [data-md-color-scheme="slate"] .hf-t    { fill: var(--horis-ink, #e7eaf1); }
    [data-md-color-scheme="slate"] .hf-s    { fill: var(--horis-ink-3, #8d95a8); }
    [data-md-color-scheme="slate"] .hf-lead { fill: var(--horis-ink-2, #b9c0cf); }
    [data-md-color-scheme="slate"] .hf-a    { stroke: var(--horis-ink-3, #8d95a8); }
    [data-md-color-scheme="slate"] .hf-r    { stroke: var(--horis-refuse, #e28d83); }
    [data-md-color-scheme="slate"] .hf-rt   { fill: var(--horis-refuse, #e28d83); }
    [data-md-color-scheme="slate"] .hf-mk   { fill: var(--horis-ink-3, #8d95a8); }
    [data-md-color-scheme="slate"] .hf-mkr  { fill: var(--horis-refuse, #e28d83); }
  </style>
  <defs>
    <marker id="horisFlowArrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
      <path class="hf-mk" d="M0 0 10 5 0 10z"/>
    </marker>
    <marker id="horisFlowArrowRefuse" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
      <path class="hf-mkr" d="M0 0 10 5 0 10z"/>
    </marker>
  </defs>
</svg>
<svg class="horis-flow-wide" viewBox="0 0 900 200" role="img" aria-label="Flow diagram: language, documents and system state enter perception, the one neural boundary, which proposes typed facts; the five-check gate admits or refuses them in domain terms; rules decide to a fixed point; the kernel emits a decision, a proof and a replayable trace.">

  <rect class="hf-in" x="8" y="46" width="132" height="52" rx="8"/>
  <text class="hf-lead" x="74" y="69" text-anchor="middle">Language, docs,</text>
  <text class="hf-lead" x="74" y="85" text-anchor="middle">system state</text>

  <rect class="hf-neu" x="172" y="34" width="150" height="76" rx="9"/>
  <text class="hf-k hf-k-neu" x="247" y="57" text-anchor="middle" fill="var(--horis-neural, #96590a)">NEURAL · BOUNDARY 1</text>
  <text class="hf-t" x="247" y="77" text-anchor="middle">Perception</text>
  <text class="hf-s" x="247" y="95" text-anchor="middle">proposes typed facts</text>

  <rect class="hf-gate" x="354" y="34" width="150" height="76" rx="9"/>
  <text class="hf-k hf-k-gate" x="429" y="57" text-anchor="middle" fill="var(--horis-gate, #12736a)">NO BACK DOOR</text>
  <text class="hf-t" x="429" y="77" text-anchor="middle">Five-check gate</text>
  <text class="hf-s" x="429" y="95" text-anchor="middle">class · range · constraint</text>

  <rect class="hf-sym" x="536" y="34" width="150" height="76" rx="9"/>
  <text class="hf-k hf-k-sym" x="611" y="57" text-anchor="middle" fill="var(--horis-symbolic, #3b5bdb)">SYMBOLIC</text>
  <text class="hf-t" x="611" y="77" text-anchor="middle">Rules decide</text>
  <text class="hf-s" x="611" y="95" text-anchor="middle">to a fixed point</text>

  <rect class="hf-out" x="718" y="34" width="174" height="76" rx="9"/>
  <text class="hf-k hf-k-gate" x="805" y="57" text-anchor="middle" fill="var(--horis-gate, #12736a)">THE PRODUCT</text>
  <text class="hf-t" x="805" y="77" text-anchor="middle">Decision + proof</text>
  <text class="hf-s" x="805" y="95" text-anchor="middle">+ replayable trace</text>

  <path class="hf-a" d="M142 72 L164 72"/>
  <path class="hf-a" d="M324 72 L346 72"/>
  <path class="hf-a" d="M506 72 L528 72"/>
  <path class="hf-a" d="M688 72 L710 72"/>

  <path class="hf-r" d="M429 112 L429 150 L247 150 L247 114"/>
  <text class="hf-rt" x="444" y="154">refused atomically, in domain terms —</text>
  <text class="hf-rt" x="444" y="172">“heartRate 1280 exceeds horis:max 300 (bpm) on :Patient”</text>
</svg>
<svg class="horis-flow-narrow" viewBox="0 0 320 480" role="img" aria-label="Flow diagram: language, documents and system state enter perception, the one neural boundary, which proposes typed facts; the five-check gate admits or refuses them in domain terms; rules decide to a fixed point; the kernel emits a decision, a proof and a replayable trace.">
  <rect class="hf-in" x="30" y="6" width="200" height="46" rx="8"/>
  <text class="hf-lead" x="130" y="26" text-anchor="middle">Language, docs,</text>
  <text class="hf-lead" x="130" y="42" text-anchor="middle">system state</text>

  <rect class="hf-neu" x="20" y="80" width="220" height="72" rx="9"/>
  <text class="hf-k hf-k-neu" x="130" y="102" text-anchor="middle" fill="var(--horis-neural, #96590a)">NEURAL · BOUNDARY 1</text>
  <text class="hf-t" x="130" y="122" text-anchor="middle">Perception</text>
  <text class="hf-s" x="130" y="140" text-anchor="middle">proposes typed facts</text>

  <rect class="hf-gate" x="20" y="180" width="220" height="72" rx="9"/>
  <text class="hf-k hf-k-gate" x="130" y="202" text-anchor="middle" fill="var(--horis-gate, #12736a)">NO BACK DOOR</text>
  <text class="hf-t" x="130" y="222" text-anchor="middle">Five-check gate</text>
  <text class="hf-s" x="130" y="240" text-anchor="middle">class · range · constraint</text>

  <rect class="hf-sym" x="20" y="280" width="220" height="72" rx="9"/>
  <text class="hf-k hf-k-sym" x="130" y="302" text-anchor="middle" fill="var(--horis-symbolic, #3b5bdb)">SYMBOLIC</text>
  <text class="hf-t" x="130" y="322" text-anchor="middle">Rules decide</text>
  <text class="hf-s" x="130" y="340" text-anchor="middle">to a fixed point</text>

  <rect class="hf-out" x="20" y="380" width="220" height="48" rx="9"/>
  <text class="hf-t" x="130" y="400" text-anchor="middle">Decision + proof</text>
  <text class="hf-s" x="130" y="418" text-anchor="middle">+ replayable trace</text>

  <path class="hf-a" d="M130 52 L130 76"/>
  <path class="hf-a" d="M130 152 L130 176"/>
  <path class="hf-a" d="M130 252 L130 276"/>
  <path class="hf-a" d="M130 352 L130 376"/>

  <path class="hf-r" d="M240 216 L282 216 L282 116 L246 116"/>
  <text class="hf-rt" x="130" y="458" text-anchor="middle">refused atomically, in domain terms</text>
</svg>
</div>

</div>

## The whole idea in ten lines

You have shipped an agent. Somewhere in a prompt it says *"patients with a heart rate over 120 and chest pain are immediate acuity"*, and somewhere in production the model skipped that line. In Horismos the sentence is a rule in a small ontology file, the model can only propose typed facts, and the rule — not the model — assigns the decision:

<div class="horis-proof" markdown>
<p class="horis-panel-label">The proof, not a transcript</p>

```python
from horismos import Ontology, World, HorismosValidationError

onto = Ontology.load("examples/triage/triage.ttl",
                     imports=["examples/triage/snomed-ct-subset.ttl"])
world = World(onto)

p = world.assert_(":Patient", subject=":p7",
                  heart_rate=128, has_symptom=[":CrushingChestPain"])

print(world.value(p, ":acuity"))              # derived by rule r1 inside the same transaction
print(world.explain(p, ":acuity").render())   # the proof, not a transcript
```

```text
Ref(:Immediate)
:acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"
  ├─ heartRate 128 exceeds 120   (asserted by the api)
  └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (subsumption)
```

</div>

No rule mentions *crushing* chest pain. The imported SNOMED subset says it is a kind of chest pain, and the proof shows that step. Now try to set the decision directly — the way a model, a tool, or a careless caller would:

<div class="horis-refusal" markdown>
<p class="horis-panel-label">Refused — every path into the world, no back door</p>

```python
try:
    world.assert_(":Patient", subject=":p8", acuity=":Immediate")
except HorismosValidationError as e:
    print(e)
```

```text
acuity may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from the api
```

</div>

That refusal is the product. It applies to every path into the world — perception, rule, tool, API — with no back door. The snippets above run verbatim against this repository; the [quickstart](quickstart.md) walks through them with nothing to configure.

## What breaks in prompt-governed agents

These are the failure classes working agent teams keep meeting. Every number below is measured in the primary source cited beside it, not by us.

<div class="horis-cmp" markdown>

<div class="horis-cmpcard" markdown>
### Silent wrong-state failures — the run "succeeds", the state is wrong, no tool errored

<p class="horis-meas" markdown><span class="horis-lbl">What was measured</span>In τ²-bench airline, 78% of observed failures were silent wrong-state failures with no tool error; deterministic read-only pre-execution gates raised task success from 29.6% to 42.0% ([arXiv:2607.07405](https://arxiv.org/abs/2607.07405))</p>

<p class="horis-does" markdown><span class="horis-lbl">What Horismos does instead</span>Every write passes a five-check gate in domain terms; rule-derived facts are recomputed, never left stale ([The World](framework/03-world.md))</p>
</div>

<div class="horis-cmpcard" markdown>
### Stale approvals — correct when checked, stale when acted on

<p class="horis-meas" markdown><span class="horis-lbl">What was measured</span>Approval verdicts changed in 5.3–48.4% of cases after an eight-step replay shift ([arXiv:2608.26306](https://arxiv.org/abs/2608.26306)); models produced stale-aligned decisions 74.7–77.3% of the time, while version-stamp checks were 100% deterministic ([arXiv:2608.25553](https://arxiv.org/abs/2608.25553), [arXiv:2609.00243](https://arxiv.org/abs/2609.00243))</p>

<p class="horis-does" markdown><span class="horis-lbl">What Horismos does instead</span>Decisions are scoped to a versioned snapshot; a commit against drifted source state fails closed ([Guarantees](guarantees.md))</p>
</div>

<div class="horis-cmpcard" markdown>
### Prompt injection flipping the conclusion

<p class="horis-meas" markdown><span class="horis-lbl">What was measured</span>Under injection, five open models inverted a verified conclusion in 61% of trials; a hardened prompt cut that to 14%; only enforcement drove it to zero ([arXiv:2606.19588](https://arxiv.org/abs/2606.19588)). Six LLMs repeated a single planted fake lab value in 50–83% of clinical vignettes ([Communications Medicine, 2025](https://www.nature.com/articles/s43856-025-01021-3))</p>

<p class="horis-does" markdown><span class="horis-lbl">What Horismos does instead</span>Perception can only propose schema-forced facts; decision properties are unrepresentable in its output; narration renders only from the proof ([LLM Boundaries](framework/06-llm-boundaries.md))</p>
</div>

<div class="horis-cmpcard" markdown>
### Policies that cannot see state

<p class="horis-meas" markdown><span class="horis-lbl">What was measured</span>Codex rules "evaluate command prefixes only — they cannot reference repository state"; after PocketOS lost production data to a blanket-scope token, the post-mortem read "guardrails must be implemented at the API gateway and token-permission level, not in advisory text" ([Codex rules documentation](https://learn.chatgpt.com/docs/agent-configuration/rules); [AIID 7311](https://incidentdatabase.ai/reports/7311))</p>

<p class="horis-does" markdown><span class="horis-lbl">What Horismos does instead</span>Preconditions are declared over typed world state and checked by the runtime before any tool fires ([Tools](framework/07-tools.md))</p>
</div>

</div>

## What you get

- **A typed world, not a scratchpad.** Facts are instances of ontology classes; ranges, units, cardinality, closed enumerations and provenance policy are enforced at the gate. Rejections are sentences an auditor can read: `heartRate 1280 exceeds horis:max 300 (bpm) on :Patient`.
- **Rules that decide, with proofs.** Deterministic forward and backward chaining, subsumption over your class hierarchy and imported vocabularies, a proof tree per conclusion, and truth maintenance that re-derives when base facts change.
- **LLMs at two declared boundaries only.** Perception (text → typed instances, schema-forced, confidence-gated, with a mandatory fallback edge) and narration (proof → prose, never written back). Structured input skips the model entirely.
- **Tools the runtime fires.** Declared contracts: typed arguments read from the graph, preconditions checked, effects validated through the same gate. The model never picks a tool or builds its arguments.
- **Traces as the product.** Every run emits a typed, replayable trajectory — the audit log and the training corpus are the same object, with loss masks precomputed.
- **Verification before deployment.** Completeness, conflict-freedom, dead rules, termination and workflow totality, with counterexamples rendered in domain terms. `Ontology.load` refuses an ontology that fails them (see [Guarantees](guarantees.md) for exactly what is claimed).

## When the model *is* the boundary

Everything above is a discipline the kernel imposes on a model that would happily do more. TypeSafe AI's **Jev** is the other case. It is a `text → decisions` model: it takes free text plus a set of typed *questions* and returns one typed *answer* each — a probability, an option key, or an ordinal level. It has no text output channel at all. Not instructed not to write prose. **Cannot.**

Routed through the perception boundary, Jev proposes and only proposes. `JevBridge` translates the perception schema the runtime derives from your ontology into those three primitives: a boolean property becomes a probability, a closed vocabulary becomes a choice, an enumeration a deployment declares ordinal becomes a score. An ontology names it the same way it names any other model — `horis:model "jev:jev-1.13"` — and routing is per step, so a workflow can send perception to Jev and narration to a text-capable bridge in the same run. Ask Jev itself to narrate and it refuses without making a call: the model has no text output channel, and the bridge will not stitch prose out of answer keys.

Ask it for something it cannot answer — a number, an open string, a date-time — and it refuses there too. It does not bucket the number into a score and it does not quietly drop the property. The call fails, naming the property, and the step takes its `horis:fallback` edge like any other bridge error, because *approximating it as a score would fabricate a measurement under perception provenance*. A heart rate belongs on the structured-input path, with the real reading and the real source. **The refusal is the design, not a gap.**

Here is a whole decision. One line of a ward note, four typed booleans with a probability each, and the three rules in `examples/vte/vte.ttl` reaching a recommendation that carries `horis:assignedBy horis:RulesOnly` — so it is absent from the schema the model is shown. The model has no way to express the decision. Not discouraged: absent.

<div class="horis-proof" markdown>
<p class="horis-panel-label">Typed answers in, a derivation out</p>

```text title="make demo-jev — second note, token and cost lines trimmed"
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
```

</div>

*On warfarin* is the ambiguity — already anticoagulated reads as a reason to hold pharmacologic prophylaxis, but the note never says so outright — and the answer set says so, at `p=0.610`. Both the per-instance and the call-level confidence are the *weakest* answer they cover, never an average, so one coin-flip cannot hide behind three certainties when the runtime compares against `horis:confidence`. Every proposed fact still clears the same five-check gate on the way in; perception provenance buys no exemption.

`make demo-jev` prints that transcript deterministically against the mock — no key, no network. `make demo-jev-live` builds in the real client; `bridges-jev` is default-off, like every bridge that carries TLS. Either way it is the same `Bridge` trait an ordinary chat model implements, and the kernel takes both unchanged. The full treatment — the schema-shape-to-primitive table, why a `jev:` step extracts exactly one class, and what the bridge deliberately refuses to guess — is in [The Bridge](framework/08-bridge.md).

## It surrounds your stack

Horismos does not replace LangGraph, your agent SDK, your coding harness, or your system of record. It gates one decision inside them.

See [Architecture](architecture.md) for the bounded projection kernel in one diagram, and [Coming from LangGraph](coming-from-langgraph.md) for the honest split of who does what.

## What Horismos does not claim

Stating the edges plainly is part of the offer.

- **It does not verify the LLM.** Verification proves the symbolic system is total and sound around whatever perception does; the mitigation for a bad extraction is the gate, the confidence threshold and the fallback edge — not a promise about the model.
- **It does not prove generated code is correct.** For coding agents, the planned candidate-artifact boundary proves a proposal was policy-legal and passed its declared checks. Semantic correctness stays a review and test claim.
- **Its guarantees are snapshot-scoped.** Inside one evaluated projection the gate has no back door and determinism holds. Freshness and completeness are only as good as the last hydration and the declared input closure.
- **It is not a system of record, an orchestrator, or OWL-DL.** It overlays your records, runs one bounded decision, and takes RDFS plus a small executable vocabulary — full description-logic inference is out of scope forever.
- **It is pre-alpha.** The symbolic core, the Python bindings, the Rust agent loop, verification (PRP-005) and standards/host integration (PRP-008) are all merged; packaging and distribution (PRP-006) is not. The [roadmap](roadmap.md) says exactly what runs today.

## Where next

<div class="grid cards" markdown>

- **[Quickstart](quickstart.md)** — `make develop`, load the triage ontology, watch the gate reject bad data, print a proof, run the full agent loop on the mock bridge, then go live with one environment variable.
- **[Go live in 2 minutes](go-live.md)** — the same demo against a real model through OpenRouter: one environment variable, a real captured trace, and a decision and proof identical to the mock run.
- **[Coming from LangGraph](coming-from-langgraph.md)** — the same decision written both ways, and the ten lines that let a graph node call the kernel.
- **[Governing a coding agent](governing-a-coding-agent.md)** — a 28-term change gate where the bot cannot grant itself merge permission.
- **[Use cases](use-cases/index.md)** — health, coding, insurance, physical AI, customer operations: one template, five runnable overlays.
- **[The guarantees, honestly](guarantees.md)** — what `verify()` proves, what is explicitly not claimed.
- **[Framework reference](framework/00-index.md)** — the shipped-voice definition, docs 00–15.

</div>
