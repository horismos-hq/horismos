---
title: Go live in 2 minutes
---

# Go live in 2 minutes

[Step 9 of the clinical quickstart](quickstart-clinical.md#9-run-the-whole-agent-loop-mock-bridge-no-network-no-key) ran the full triage loop against the deterministic mock bridge: no network, no key. This page runs the **same binary, the same Turtle file, the same workflow and the same tools** against a real model through OpenRouter. One environment variable is the only difference, and everything quoted below is the real output of one such run, captured on 2026-09-03 with the default model and committed unchanged.

## 1. Put the key in your environment

```sh
export OPENROUTER_API_KEY=...                       # your existing key, from your shell or secret manager
# optional: export OPENROUTER_MODEL=anthropic/claude-sonnet-4.5   # default: openai/gpt-4o-mini
```

The bridge reads the key from the environment when it starts (`OpenRouterBridge::from_env()`). It never logs it, never writes it into a trace, and nothing in this repository contains it — the trajectory file at the end of this page was checked for that before it was committed. The demo binary picks live over mock on exactly one signal: whether `OPENROUTER_API_KEY` is set.

## 2. Run the same demo

```sh
make demo-live
```

Real output, verbatim apart from the shortened temp-file path. The narration is the model's own prose, quoted as it came back:

```text
Horismos triage agent demo — live (OpenRouter)
input: "58M, crushing chest pain, heart rate 128"

terminal: :Done
fallback taken: false
decision: :acuity = :Immediate

proof (:acuity):
:acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"
  ├─ heartRate 128 exceeds 120   (asserted by perception)
  └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (subsumption)

narration:
  [:Explain] In our current situation, we are dealing with a patient exhibiting significant signs that necessitate prompt attention. The acuity level assigned here is categorized as "Immediate," which highlights the urgency of the case.

The reasoning can be traced through the established reference point r1, referred to as "Tachycardia with chest pain — immediate track." This indicates that the clinical parameters warrant immediate action. To break it down, we see that the heart rate of the patient is at 128 beats per minute. This figure is important because it notably exceeds the threshold of 120 beats per minute. The fact that this heart rate was asserted by perception gives us a reliable foundation for our assessment, reinforcing the critical nature of the situation.

Additionally, the patient is reported to have a symptom of crushing chest pain. This symptom is classified under the broader category of chest pain. This classification, understood through the principle of subsumption, further underscores the severity of the patient's condition.

Together, these observations and classifications firmly establish the need for immediate intervention for the patient in question.

trajectory (4 steps) written to /var/folders/…/horismos_triage_demo_trajectory.json
```

Put it next to the mock run in step 9: `terminal`, `fallback taken`, `decision` and the proof tree are identical, character for character. Only the first line and the narration changed. That is the design working: the model translated language into typed facts, and later rendered a proof into prose. It never touched the decision.

## 3. What actually happened, step by step

The demo writes the whole run as `horismos.trace/2`, the schema described in [Traces](framework/09-traces.md). The four steps below are quoted from the file this run wrote; the complete file is listed at the end of the page and committed as `examples/triage/runs/live-openrouter-2026-09-03.trajectory.json`.

### Perception — the model proposes typed facts

`:Intake` is a `PerceptionStep` declared with `horis:extracts :Patient, :Symptom` and `horis:confidence 0.85`. At load time the runtime derived a JSON Schema from those two classes — `heartRate` as a bounded number in bpm, the symptom tree as a closed enum of declared local names, and `:acuity` left out entirely because it is `horis:assignedBy horis:RulesOnly`. The step made one bridge call: the input text plus that schema, the answer forced into it.

```json
{
  "kind": "perception",
  "step": ":Intake",
  "model": "openrouter:openai/gpt-4o-mini",
  "confidence": 0.9,
  "threshold": 0.85,
  "accepted": true,
  "asserted": [
    {
      "s": "<https://w3id.org/horismos/.well-known/genid/0>",
      "p": ":hasSymptom",
      "o": ":CrushingChestPain",
      "provenance": "perception",
      "conf": 0.9
    },
    {
      "s": "<https://w3id.org/horismos/.well-known/genid/0>",
      "p": ":heartRate",
      "o": "128",
      "provenance": "perception",
      "conf": 0.9
    }
  ],
  "loss_mask": false
}
```

Three things to read off this step:

- **`model` names the bridge that really ran** (`openrouter:openai/gpt-4o-mini`), not the `claude:` prefix the ontology declares. Deployment maps prefix to bridge; the trace makes an override visible after the fact.
- **`confidence` 0.9 cleared `threshold` 0.85**, so the proposal was accepted. Below the threshold nothing is asserted and the workflow takes `:Intake`'s `horis:fallback` edge — the model never guesses a patient into existence.
- **Every `asserted` fact passed the validation gate before it landed**, with `provenance: perception` and its confidence attached. The gate checked range (`:CrushingChestPain` is a declared `:Symptom`), cardinality, `horis:min`/`horis:max` and RulesOnly. Had the model returned a heart rate of 1280, the instance would have been refused with `heartRate 1280 exceeds horis:max 300 (bpm) on :Patient` and no partial commit would remain — but unlike a below-threshold answer, a gate refusal does not go straight to the fallback. The step re-calls the bridge up to two more times (`:Intake` declares no `horis:retries`, so doc 06's default of 2 applies), each a paid model call, and only once those are exhausted does the workflow take `:Intake`'s `horis:fallback` edge.

### Reasoning — the rule decides

```json
{
  "kind": "reasoning",
  "step": ":Assess",
  "fired": [
    {
      "rule": ":r1",
      "bindings": {
        "?p": "<https://w3id.org/horismos/.well-known/genid/0>"
      },
      "derived": [
        {
          "s": "<https://w3id.org/horismos/.well-known/genid/0>",
          "p": ":acuity",
          "o": ":Immediate",
          "provenance": "rule"
        }
      ],
      "proof": {
        "goal": "<https://w3id.org/horismos/.well-known/genid/0> :acuity :Immediate",
        "via": ":r1",
        "explain": "Tachycardia with chest pain — immediate track",
        "conditions": [
          {
            "goal": "<https://w3id.org/horismos/.well-known/genid/0> heartRate exceeds 120",
            "via": "asserted",
            "provenance": "perception"
          },
          {
            "goal": "<https://w3id.org/horismos/.well-known/genid/0> hasSymptom is :ChestPain",
            "via": "subsumption(:CrushingChestPain)",
            "provenance": "perception"
          }
        ]
      }
    }
  ],
  "suppressed": [],
  "passes": 2,
  "goal_reached": true,
  "loss_mask": true
}
```

`:r1` fired on the perceived facts and derived `:acuity :Immediate` with `provenance: rule`. The proof records both conditions: the comparison of the perceived heart rate against 120, and the subsumption step `:CrushingChestPain ⊑ :ChestPain` that satisfied the `hasSymptom is :ChestPain` condition. `goal_reached: true` is what let the workflow continue. Had `:acuity` stayed unassigned, the declared `horis:onUnreachable` edge would have sent the run to `:EscalateToNurse` — a page to a person, not a guessed acuity.

### Action — a tool fires and its effects are validated

```json
{
  "kind": "action",
  "step": ":Route",
  "tool": ":ehr.assign_queue",
  "effects_asserted": [
    {
      "s": "<https://w3id.org/horismos/.well-known/genid/0>",
      "p": ":queuedAt",
      "o": "2026-08-19T00:00:00+00:00",
      "provenance": "tool"
    },
    {
      "s": "<https://w3id.org/horismos/.well-known/genid/0>",
      "p": ":queueRef",
      "o": "ED-QUEUE-IMMEDIATE-7",
      "provenance": "tool"
    }
  ],
  "loss_mask": true
}
```

`:Route` requires `:acuity` to be assigned (a precondition the runtime checks, not a hint in a prompt) and invoked `:ehr.assign_queue`, a Rust stub in the demo. Its declared effects — `:queuedAt` stamped with the transaction's `horis:now`, `:queueRef` from the tool — were asserted with `provenance: tool` through the same gate as everything else. The demo pins the run's `horis:now` to a fixed instant (`2026-08-19T00:00:00Z`, passed as the `RunInput` timestamp) so the mock and live trajectories diff byte for byte, which is why `started_at` and `:queuedAt` in the committed trajectory predate the 2026-09-03 capture.

### Narration — the model renders the proof

```json
{
  "kind": "narration",
  "step": ":Explain",
  "audience": "charge-nurse",
  "text": "In our current situation, we are dealing with a patient exhibiting significant signs that necessitate prompt attention. The acuity level assigned here is categorized as \"Immediate,\" which highlights the urgency of the case. \n\nThe reasoning can be traced through the established reference point r1, referred to as \"Tachycardia with chest pain — immediate track.\" This indicates that the clinical parameters warrant immediate action. To break it down, we see that the heart rate of the patient is at 128 beats per minute. This figure is important because it notably exceeds the threshold of 120 beats per minute. The fact that this heart rate was asserted by perception gives us a reliable foundation for our assessment, reinforcing the critical nature of the situation.\n\nAdditionally, the patient is reported to have a symptom of crushing chest pain. This symptom is classified under the broader category of chest pain. This classification, understood through the principle of subsumption, further underscores the severity of the patient's condition. \n\nTogether, these observations and classifications firmly establish the need for immediate intervention for the patient in question.",
  "loss_mask": false
}
```

`:Explain` made the second and last model call: the proof in, prose for the `charge-nurse` audience out. The text is quoted, not trusted. Its `loss_mask: false` marks it — like the perception step — as model-generated and therefore trainable, while the reasoning and action steps carry `loss_mask: true`: executor output a fine-tuning run must never learn to imitate. One file, two roles: the audit log and the training corpus are the same object.

## Same code, one variable

| | `make demo` | `make demo-live` with `OPENROUTER_API_KEY` set |
|---|---|---|
| Bridge behind the `claude:` prefix | `MockBridge`, scripted | `OpenRouterBridge::from_env()` |
| Network | none | two HTTPS calls to OpenRouter: perceive, narrate |
| Perception step in the trace | `model: mock:default`, confidence 0.94 | `model: openrouter:openai/gpt-4o-mini`, confidence 0.9 |
| Reasoning step, action step, outcome | identical | identical — the two trajectory files were diffed while writing this page |
| Narration step | the scripted sentence | the model's prose |

Everything else — the Turtle file, the workflow, the tool bindings, the gate, the proof, the trace schema — is one code path. Unset the variable and the kernel still runs.

## Honest edges

- **The tools are stubs.** `assign_queue` returns a fixed queue reference; a deployment binds a real EHR call under the same declared contract, and the gate validates its effects the same way.
- **The narration is prose from a model.** It is recorded and labelled, not verified. The proof is the thing to read when you want to know why.
- **A live model is not deterministic.** Your narration will differ from the one above. The decision and the proof will not, as long as the perception step asserts the same facts — and if the model misses the symptom, the proof cannot be built and the run escalates rather than guesses.
- **This is the first live run of the demo to reach `:Done`.** Until 2026-09-03 the runtime dropped array-valued perception answers: the derived schema asks for an array for the multi-valued `:hasSymptom`, the model answered with one, and the value was lost, so every live run found `:Assess` unreachable and paged the nurse with `fallback taken: true`. The fail-closed edge did exactly what the workflow declared; the defect is fixed in `crates/horismos-core/src/workflow/agent.rs` with regression tests that failed before the fix.
- **`make demo` stays on the mock bridge even with the key set.** It is built without the OpenRouter client, and says so on stderr. Only `make demo-live` switches bridges.
- **This demo runs from Rust.** Its workflow fires tools, and tool bindings are Rust functions. `horismos.Agent` runs workflows of perception and reasoning steps from Python — see [the same loop from Python](quickstart-clinical.md#the-same-loop-from-python).

## The full trajectory

The file exactly as the run wrote it (pretty-printed):

```json
--8<-- "examples/triage/runs/live-openrouter-2026-09-03.trajectory.json"
```
