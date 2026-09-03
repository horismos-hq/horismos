# 09 · Traces

A trace is the typed trajectory of a run. It serves three consumers with one object:
**auditors** (why did the agent do that?), **operators** (replay and debug), and —
deliberately, from day one — **training pipelines** (the Vision flywheel: Horismos
trajectories are TIR/CIR-grade data).

## The trajectory schema (v1)

```jsonc
{
  "schema": "horismos.trace/1",                 // versioned FOREVER — traces outlive releases
  "run_id": "run_01J…",
  "ontology": { "iri": "…/triage", "version": "1.2.0", "hash": "sha256:…" },
  "workflow": "triage:Triage",
  "started_at": "2026-07-28T09:14:03Z",
  "input": { "kind": "nl", "text": "58M, crushing chest pain 20 min, HR 128…" },

  "steps": [
    {
      "step": "triage:Intake", "kind": "perception",
      "model": "claude:claude-fable-5",
      "confidence": 0.94, "threshold": 0.85, "accepted": true,
      "latency_ms": 640, "tokens": { "in": 412, "out": 96 },
      "asserted": [
        { "s": "_:p7", "p": ":heartRate", "o": 128, "conf": 0.94 },
        { "s": "_:p7", "p": ":hasSymptom", "o": ":CrushingChestPain", "conf": 0.91 }
      ],
      "loss_mask": false                      // model-generated → trainable
    },
    {
      "step": "triage:Assess", "kind": "reasoning",
      "fired": [
        { "rule": ":r1", "bindings": { "?p": "_:p7" },
          "derived": [{ "s": "_:p7", "p": ":acuity", "o": ":Immediate" }],
          "proof": { /* ProofNode, doc 04 */ } }
      ],
      "suppressed": [],                        // conflict-policy losers, if any
      "goal_reached": true,
      "loss_mask": true                        // executor output → masked in training
    },
    {
      "step": "triage:Route", "kind": "action",
      "tool": ":ehr.assign_queue", "binding": "mcp://ehr-server/assign_queue",
      "args": { "patient": "_:p7", "queue": ":Immediate" },
      "preconditions": [{ "pattern": "acuity isAssigned", "held": true }],
      "effects_asserted": [{ "s": "_:p7", "p": ":queuedAt", "o": "2026-07-28T09:14:05Z" }],
      "idempotent": false, "latency_ms": 120,
      "loss_mask": true
    }
  ],

  "outcome": {
    "terminal": "triage:Done", "goal_reached": true,
    "decision": { "property": ":acuity", "value": ":Immediate" },
    "labels": { "workflow_completed": true, "fallback_taken": false }   // reward-ready
  },
  "snapshots": { "s0": "…", "s1": "…" },       // per step boundary, binary refs
  "totals": { "duration_ms": 1930, "bridge_calls": 1, "cost_usd": 0.0041 }
}
```

## Design rules

1. **Schema versioning is sacred.** `horismos.trace/1` never changes meaning; evolution
   adds `horismos.trace/2` with a converter. Traces used as training data outlive every
   framework release.
2. **`loss_mask` is precomputed.** The TIR/CIR literature is unanimous: train on
   model-generated tokens, mask executor observations. Horismos traces mark this per
   step at emission — the training exporter doesn't guess.
3. **Outcome labels are deterministic.** `goal_reached`, `fallback_taken`,
   verification-passing effects — computed by the runtime, not judged by a model.
   This is the "reward for free" property.
4. **Provenance everywhere.** Every asserted fact in a trace carries its provenance
   and (for perception) confidence — the neuro-symbolic seam stays visible at the
   per-fact level.

## Replay

```python
from horismos import replay
run2 = replay(trace, ontology_dir="…")
```

- Deterministic steps re-execute and must reproduce recorded results (drift =
  hard error — catches ontology/runtime version skew).
- Bridge calls are substituted from the trace (never re-billed, never re-sampled).
- Non-idempotent tools are substituted from recorded effects; idempotent tools may
  re-execute with `--live-tools`.
- Snapshot refs make partial replay possible: start at any step boundary.

## Training export

```python
from horismos.export import to_training_jsonl
to_training_jsonl(traces, out="corpus.jsonl",
                  format="tagged",          # <perceive>/<reason>/<act>/<narrate>, executor output in <observe> (decided, PRP-002 — trace::migrate::EXPORT_TAGS)
                  min_outcome={"workflow_completed": True})
```

Export filters on outcome labels (the deterministic reward), renders steps in a
stable tagged format with masks applied, and emits dataset cards documenting the
ontology version and filter criteria. The NL→Turtle authoring corpus (doc 13) uses
the same exporter with a different template.

## Standards-integration trace envelope

The bounded projection adds adapter evidence that is not a new meaning for a core reasoning
step. `horismos.integration-trace/1` therefore wraps an unchanged core trace and carries the
snapshot closure, resource version vector, profile/mapping/package/terminology identities,
proof reference, proposed effects, and remote receipts. Commit state remains distinct from
decision completion: `decision-produced`, `commit-attempted`, `commit-accepted`,
`commit-conflicted`, and `commit-failed` are separately observable. A later incompatible
change to this envelope requires its own explicit migration; it never silently changes
`horismos.trace/1` or `/2`. FHIR payloads and coding-repository contents remain subject to
the deployment's retention and redaction policy in addition to the core trace rules below.

## Storage & privacy

Traces are JSONL on disk by default; a store interface (append, query by run/outcome/
time) supports production backends. Perception steps can be marked
`horis:redactInput true` for domains where the raw NL must not persist (the typed
facts and hashes remain — auditability without payload retention).
**[DECIDE: redaction defaults per domain pack — PRP-006]**
