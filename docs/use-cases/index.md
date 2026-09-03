# Use cases

One template, five domains, every overlay runnable against the shipped engine. Each page follows the same four beats so you can compare them, and each ends with an honest status line.

| Beat | What you will find |
|---|---|
| **The scene** | The decision an agent is being asked to make, and what goes wrong when a prompt makes it |
| **The overlay** | A small Turtle file: classes, constraints, a `RulesOnly` decision property, and the rules — pulled verbatim from `examples/` |
| **The decision, with proof** | Facts in, a derived decision and its proof tree out; the refusal when something tries to set the decision directly; truth maintenance when a fact changes |
| **Why verification matters here** | What PRP-005's completeness, conflict, dead-rule and non-vacuity checks would tell you about *this* rule set, and what stays out of scope |

All thresholds in the gallery overlays are illustrative teaching values, not clinical, regulatory, or safety guidance. The triage example (`examples/triage/`) is the framework's flagship fixture; the gallery overlays live in `examples/gallery/` and exist to show the shape.

<div class="grid cards" markdown>

- **[Health](health.md)** — VTE prophylaxis (the recommended first standards-anchored vertical slice), the triage flagship, and why prior authorization is a *later* build.
- **[Coding](coding.md)** — CI failure triage, and merge/release gating through the change gate.
- **[Insurance](insurance.md)** — first-pass claims adjudication where every denial is an appealable proof tree.
- **[Physical AI](physical-ai.md)** — a shield between a learned policy and an actuator: typed preconditions gate motion commands.
- **[Customer operations](customer-ops.md)** — airline change and refund eligibility, the shape of the τ-bench airline tasks.

</div>

## What every page shares

- A decision property marked `horis:assignedBy horis:RulesOnly`. No model, tool, or API caller can set it; only a rule firing can.
- A refusal, shown, when something tries.
- Proof trees whose leaves name the provenance of every fact they rest on.
- A fail-closed default: when the facts a rule needs are missing, there is no decision at all — never a silent default.
- Rules written against concepts, so coverage grows with the vocabulary (see the quickstart's SNOMED step) rather than with the prompt.
