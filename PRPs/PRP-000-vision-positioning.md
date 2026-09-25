# PRP-000 — Vision & Positioning

**Status**: Draft · standing document
**Depends on**: —

## Intent

Fix what Horismos claims, to whom, and — as important — what it deliberately does not claim,
so every later PRP inherits a stable story.

## The pitch

**Pivot adopted 2026-08-24** (captain ruling, `bounded projection kernel`;
`horismos-thin-pivot-s1/report.md` §"PRP-by-PRP change map", row PRP-000): Horismos is an
ontology-first neuro-symbolic **decision and capability kernel**. A subject-matter expert
authors one artifact — a small RDFS-plus overlay declaring the rules, constraints, and
decision/workflow fragment the deployment needs — and the runtime **evaluates it as a
verified projection over an existing system of record**, not as the system of record itself.
Horismos does not own durable state, the enterprise workflow, provider clients, or standard
terminology; an adapter hydrates a versioned, explicitly scoped bundle from the external
authority, the unchanged gate/reasoning/truth-maintenance/proof/trace core runs over that
local snapshot, and the result is a typed proposed effect set an adapter conditionally
commits back — failing closed if the source versions drifted. LLMs still exist only at two
declared transducer boundaries (perception: NL → typed instances; narration: proof → NL).
Every conclusion carries a proof tree; every rule set is formally verified before it runs.
Delete the LLM and the framework still runs.

**Lead story amended 2026-09-25** (PRP-009 Tier 1; captain's decision 3, "Front Door",
2026-09-23): the docs home page and the nav lead with **Physical AI**. The kernel is pitched
as the layer between a policy and what it acts on — a planner and an actuator interface first,
an agent and its tools second — and the first example a reader meets on the home page is the
motion-command shield (`examples/gallery/safety-envelope.ttl`): a learned policy proposes a
command, the rules decide with a proof whether it may pass, a missing sensor reading is
refused rather than read as safe, and no model is involved at all. The quickstart still
opens with triage; the Tier 2 PR extends the shield to it when it lands. The rest of the
pitch above is unchanged; the
shield exercises the same gate, chaining, truth maintenance and verification with none of the
LLM boundaries. What it does not claim is fixed by PRP-009's safety-boundary statement (the
wording is the captain's, decision 1): Horismos is a supervisory decision layer, not a
safety-rated component, a safety PLC or a control loop, and makes no claim against ISO 10218,
ISO/TS 15066 or IEC 61508.

Superseded framing, kept here for the record: "one artifact owns domain + rules + workflows"
described Horismos as the system of record. The evidence (`research/formalization/`,
`horismos-thin-pivot-s1/report.md`) is that the historically successful CIG-family systems
were thin layers over an existing record (FHIR/CDS-Hooks/CQL), not replacements for it, and
that "own everything" repeats the highest-friction side of a 20-year adoption failure. See
PRP-008 for the standards/host-integration work this pivot requires.

**Tagline candidates** (settle here): "The symbolic spine for LLM agents" ·
"Agents that live inside an ontology" · "Neural proposes. Symbolic decides." ·
"A verified decision layer that overlays what you already run."

## Comparison: thin decision layer vs. adjacent standards and harnesses

Horismos does not compete with a system of record; it overlays one. The honest comparison
set now includes the clinical-standards stack and coding-agent harnesses, not just other
agent frameworks (see the full comparison table in `## Honest placement` below for the
neuro-symbolic set):

| System | What it owns | What Horismos adds if layered on top |
|---|---|---|
| **FHIR** | The resource model and wire format for a system of record (patients, observations, plans). | Horismos does not replace FHIR resources with its own World-as-record; it hydrates a versioned, declared-closure bundle *from* FHIR into a local projection, reasons over it, and conditionally commits proposed effects back — FHIR keeps owning durable state. |
| **CQL (Clinical Quality Language)** | Expression evaluation over clinical data models — the industry-standard way to compute derived clinical facts. | Horismos composes with a CQL engine (consumes its typed results as inputs) rather than rebuilding one; native CQL compilation is an explicit non-goal until there is evidence it is needed (PRP-008 D7). |
| **CDS Hooks** | The service-invocation contract (a hook fires, a card comes back) that lets an EHR call an external decision service at a workflow point. | Horismos can sit behind a CDS-Hooks-style service surface as the decision service — the host still owns the hook lifecycle and the card UI; Horismos supplies the proof-backed decision. |
| **PlanDefinition (FHIR)** | A standard, portable representation of a care/order plan or protocol. | Horismos can import and execute a *deliberately bounded subset* of PlanDefinition, failing closed (a domain-term load error, never a silent approximation) on unsupported constructs — it does not attempt full PlanDefinition coverage. |
| **Coding-agent harnesses (Claude Code skills, Pi, DeepSeek Harness)** | Extensibility, typed tool-call schemas, session logs, hooks/approvals, sandboxes — real engineering, already built. | None of the surveyed harnesses declare a semantic domain graph with verified cross-tool preconditions/effects or maintain a fixed point over repository state. A Horismos **Capability Fragment** (PRP-007/008) is a small ontology overlay attached to an *existing* skill/tool that proves when a capability is eligible, constrains its observable effect, and emits a proof — without replacing the harness's own state, sandbox, or approval machinery. |

## Snapshot-scoped guarantees (the honest claim under the thin model)

The pre-pivot text implied a global, always-current guarantee ("every mutation passes the
gate," full stop). Under the bounded-projection model the guarantee is real but scoped —
stated here verbatim from the adopted design report so it is not re-derived or loosened ad
hoc downstream:

> Given a verified decision package, a declared input closure, a source-version vector, and
> an admitted input bundle, Horismos deterministically produces a fixed-point decision
> snapshot, proof, trace, and proposed effect set. A commit succeeds only if the external
> resources still match that source-version vector. Mutations outside Horismos are outside
> the guarantee and invalidate the projection until it is refreshed.

And the one-sentence positioning statement every later PRP inherits:

> Horismos is a deterministic, ontology-first decision and capability kernel that evaluates a
> versioned projection of an existing system, proves why an effect is allowed, and
> conditionally commits through standards-native adapters.

What stays true unconditionally (local, inside one evaluated projection): the gate has no
back door, truth maintenance re-chains to a fixed point, determinism holds over materialized
inputs, and every proof/trace is exact for the recorded snapshot. What becomes explicitly
scoped: freshness (only as current as the last hydration + version check), completeness
(bounded by a declared input closure, not implicit), and atomicity (local transactions are
atomic; cross-system commits are conditional, not distributed-atomic). See
`docs/framework/03-world.md` and `docs/framework/15-security.md` for the full mechanism.

## Personas

1. **Researchers** building neuro-symbolic systems who today glue Scallop/Prolog/ad-hoc
   rules to LLM calls by hand.
2. **Agent builders** in regulated or correctness-critical domains (health, finance,
   compliance) who cannot ship "the model felt confident."
3. **SMEs** (clinicians, adjusters, compliance officers) — not direct users of the API,
   but the authors of the ontology via the authoring agent (PRP-007).
4. **Robotics and physical-AI teams** (added 2026-09-25, PRP-009) shipping a learned policy —
   a VLA model, an RL controller, an LLM planner — that proposes motion near people, who
   need a deterministic shield with a proof between the policy and the actuator interface,
   above the safety-rated stops and interlocks the cell already has and never in their place.

## Honest placement

Kautz **Type 2–3**: deterministic symbolic orchestrator, neural perception at the edges.
We say so plainly. The comparison set and one-line differentiation:

- **Scallop** — differentiable Datalog (Type 5), research-oriented; no agent loop, no
  workflow model, no LLM boundary contracts. Horismos is the productionized Type 2–3 layer.
- **Apollo-1 (AUI)** — closed, typed-state commercial agent runtime; validates the market.
  Horismos is open, ontology-standard (RDFS-plus), and verifiable.
- **LangGraph / agent SDKs** — orchestration where the LLM decides; no typed world, no
  proofs, no verification. Horismos inverts control.
- **GoRules/Drools** — rules engines without an agent model, ontology, or neural boundary.

## Research appendix (not a phase)

The differentiable extension (see plan artifact, Vision tab): trace fine-tuning
(TIR/CIR-style SFT+GRPO on Horismos trajectories — Type 2 cloned into Type 3 weights),
Scallop-style provenance semirings over the Datalog-shaped rule subset (genuine Type 5),
and GNN guidance over the instance graph (open question). Obligation on v1: traces are
training-corpus-grade from day one (PRP-002).

## Decisions (resolved 2026-07-28)

- **Proving ground — both-track**: τ-Bench airline supplies the public benchmark
  number (direct Apollo-1 comparison, existing scoreboard); clinical triage stays the
  docs narrative and SNOMED interop showcase.
  *Amended 2026-09-25* (PRP-009 Tier 1, captain's decision 3 "Front Door", 2026-09-23):
  the docs home page and the nav now lead with the motion-command shield; the Tier 2 PR
  extends that to the quickstart when it lands. Clinical triage stays the running example
  of the framework reference (docs 01–13), the flagship fixture and the SNOMED interop
  showcase, and
  `PRPs/CONVENTIONS.md`'s rule that every PRP's DoD is expressed against triage is
  unchanged; PRP-009's DoD is the recorded exception.
- **Naming — Horismos** (rebrand from "Sigil"): Aristotle's ὁρισμός, *definition* —
  root of "horizon", matching both the ontology-as-definitions thesis and the
  declared-LLM-boundaries thesis. Basis: `sigil` was taken on crates.io and PyPI, and
  an unrelated "Sigil programming language" (Daemoniorum LLC) actively publishes
  crates. Verified free at decision time: crates `horismos`, `horismos-core`,
  `horismos-bridges`, `horismos-py`; PyPI `horismos`; domain `horismos.dev`
  (**register promptly** — PRP-006 registers the package names). Vocabulary prefix:
  `horis:`; namespace IRI: `https://w3id.org/horismos#` (w3id PR filed at PRP-001
  activation — deliberately decoupled from any marketing domain).
- **License — MIT OR Apache-2.0 dual** (Rust convention): `LICENSE-MIT` +
  `LICENSE-APACHE` at repo root; all package metadata says `MIT OR Apache-2.0`.
- **Turtle parser — oxigraph** (owned by PRP-001, recorded here for visibility):
  human call, spike waived; validated empirically in PRP-001 U2.

## Definition of done

A `README.md` + docs-site landing page that states the pitch, personas, comparison table,
and honest taxonomy — reviewed and stable enough that PRP-001–007 never re-litigate it.
