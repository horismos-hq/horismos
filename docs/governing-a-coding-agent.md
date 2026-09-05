# Governing a coding agent

Your coding agent — Claude Code, Codex, pi, a harness of your own — proposes a patch and asks to merge it. Today the answer comes from a permission prompt, a prefix rule, or increasingly a model classifier. None of those can see the repository's state, and none of them produces a reason you could show a release manager. This page builds the alternative: a **25-term change gate** the harness consults before it acts, where the decision is derived by rules over typed facts and the bot cannot grant itself merge permission.

## Why prompt-level policy is not enough

The research wiki's survey of harnesses (`research/neurosymbolic-landscape/coding-agent-harnesses.md`, 2026-09-02) finds the same shape everywhere:

- **Policies cannot see state.** Codex rules "evaluate command prefixes only — they cannot reference repository state, file contents, or environmental conditions." Gemini CLI's policy engine predicates over tool name, argument patterns and annotations only.
- **The approval step is going neural.** Codex `auto_review`, Claude Code `auto` mode, and Cursor Auto-review put a classifier inside the decision; Cursor calls its run modes "best-effort guardrails rather than a hard security boundary."
- **Patch application has no base identity.** No core harness checks a base hash before applying an edit; the one tool-local attempt "silently returns success when anchors are stale," and Claude Code shipped a fix for file tools following a symlink swapped *after* the permission check — a time-of-check/time-of-use gap between authorization and effect.
- **The failures happen below the prompt.** After PocketOS lost production data and its backups to a blanket-scope token, the post-mortem read: "Guardrails must be implemented at the API gateway and token-permission level, not in advisory text."

The same survey finds that no harness "declares a semantic domain graph with verified cross-tool preconditions/effects or maintains a fixed point over repository state" — and that every harness already exposes the slot such a thing would plug into: a typed, ordered permission decision with a reason string (`PreToolUse` hooks in Claude Code, `BeforeTool` in Gemini CLI, `beforeShellExecution` in Cursor, an MCP proxy for everything else).

## The 25-term change gate

Eight classes and enumeration members, nine properties, eight rules. The harness asserts what it observed about a proposal; the rules — and only the rules — assign `:gateDecision`. It runs today against the shipped engine.

```turtle
--8<-- "examples/change-gate/change-gate.ttl"
```

Two lines carry most of the weight. `horis:assignedBy horis:RulesOnly` on `:gateDecision` means the gate refuses that property from every provenance except a rule firing. And `horis:cardinality "1"` on `:proposedBy` means a proposal with no declared proposer is not a proposal.

## Run it

```python
from horismos import HorismosValidationError, Ontology, World

onto = Ontology.load("examples/change-gate/change-gate.ttl")
world = World(onto)

def gate(subject: str, **observed: object) -> None:
    ref = world.assert_(":ChangeProposal", subject=subject, **observed)
    proof = world.explain(ref, ":gateDecision")
    print(proof.render() if proof else f"{subject}: no rule fired — no decision")

# A routine bot change: fresh base, green checks, small diff, no protected paths.
gate(":pr-41", proposed_by=":Bot", target_branch="main",
     base_matches_head=True, tests_passed=True, lint_passed=True,
     touches_protected_path=False, lines_changed=112, human_approvals=0)
```

```text
:gateDecision = :Approved   via approveRoutineChange — "Fresh base, green checks, no protected paths, small diff — routine change"
  ├─ baseMatchesHead true is true   (asserted by the api)
  ├─ testsPassed true is true   (asserted by the api)
  ├─ lintPassed true is true   (asserted by the api)
  ├─ touchesProtectedPath false is false   (asserted by the api)
  └─ linesChanged 112 atMost 400   (asserted by the api)
```

That proof is the reason string your harness hook returns. Now the cases that matter.

**A protected path with no human approval** pends rather than merges:

```python
gate(":pr-42", proposed_by=":Bot",
     base_matches_head=True, tests_passed=True, lint_passed=True,
     touches_protected_path=True, lines_changed=30, human_approvals=0)
```

```text
:gateDecision = :NeedsHuman   via unreviewedChangeNeedsHuman — "Green checks, but no human approval and the change is outside the routine envelope — a reviewer is required"
  ├─ baseMatchesHead true is true   (asserted by the api)
  ├─ testsPassed true is true   (asserted by the api)
  ├─ lintPassed true is true   (asserted by the api)
  ├─ none of:
  │  └─ humanApprovals atLeast 1   (not held)
  └─ any of:
     ├─ none of:
     │  └─ touchesProtectedPath is false   (not held)
     └─ none of:   (not held)
        └─ linesChanged 30 atMost 400   (asserted by the api)
```

**A stale base is rejected** even with everything else green — the highest-priority rule wins, and the proof names the clause:

```python
gate(":pr-43", proposed_by=":Bot",
     base_matches_head=False, tests_passed=True, lint_passed=True,
     touches_protected_path=False, lines_changed=10, human_approvals=0)
```

```text
:gateDecision = :Rejected   via rejectStaleBase — "Base hash no longer matches the branch head — the proposal was authored against stale state"
  └─ baseMatchesHead false is false   (asserted by the api)
```

**The bot cannot approve itself.** A model, a tool, or any API caller trying to write the decision is refused at the gate, in domain terms:

```python
try:
    world.assert_(":ChangeProposal", subject=":pr-44",
                  proposed_by=":Bot", gate_decision=":Approved")
except HorismosValidationError as e:
    print(f"[{e.constraint}] {e}")
```

```text
[provenance_refused] gateDecision may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from the api
```

**Decisions track the facts.** A large bot diff needs a human; when a human approves, the decision is re-derived; when the base drifts under it, the approval is withdrawn — all in the transaction that changed the fact, never a stale approval left standing:

```python
gate(":pr-45", proposed_by=":Bot",
     base_matches_head=True, tests_passed=True, lint_passed=True,
     touches_protected_path=False, lines_changed=900, human_approvals=0)
gate(":pr-45", human_approvals=1)             # a reviewer approves
gate(":pr-45", base_matches_head=False)       # main moved; the hash no longer matches
```

```text
:gateDecision = :NeedsHuman   via unreviewedChangeNeedsHuman — "Green checks, but no human approval and the change is outside the routine envelope — a reviewer is required"
  ├─ baseMatchesHead true is true   (asserted by the api)
  ├─ testsPassed true is true   (asserted by the api)
  ├─ lintPassed true is true   (asserted by the api)
  ├─ none of:
  │  └─ humanApprovals atLeast 1   (not held)
  └─ any of:
     ├─ none of:   (not held)
     │  └─ touchesProtectedPath false is false   (asserted by the api)
     └─ none of:
        └─ linesChanged atMost 400   (not held)

:gateDecision = :Approved   via approveReviewedChange — "Fresh base, green checks, and a human approved it"
  ├─ baseMatchesHead true is true   (asserted by the api)
  ├─ testsPassed true is true   (asserted by the api)
  ├─ lintPassed true is true   (asserted by the api)
  └─ humanApprovals 1 atLeast 1   (asserted by the api)

:gateDecision = :Rejected   via rejectStaleBase — "Base hash no longer matches the branch head — the proposal was authored against stale state"
  └─ baseMatchesHead false is false   (asserted by the api)
```

**Missing facts never produce a default approval** — and they no longer produce silence either. An earlier draft of this gate left a proposal the harness could not fully observe with no `:gateDecision` at all, and relied on the caller to read that absence as *stop*. `verify()`'s completeness check reported the region, and closing it required a rule: `:unrecordedCheckNeedsHuman` derives an explicit `:NeedsHuman` whose proof names the checks that never reported, while `:Approved` stays unreachable from missing evidence. A workflow routes that decision to a person on a stated reason, not on a value that failed to arrive:

```python
gate(":pr-46", proposed_by=":Human")
print(world.query(":ChangeProposal", gate_decision=":Approved"))
```

```text
:gateDecision = :NeedsHuman   via unrecordedCheckNeedsHuman — "A required check never reported — the gate cannot approve on evidence it does not have"
  ├─ baseMatchesHead isNot false   (closed world)
  ├─ testsPassed isNot false   (closed world)
  ├─ lintPassed isNot false   (closed world)
  └─ any of:
     ├─ baseMatchesHead isEmpty   (closed world)
     ├─ testsPassed isEmpty   (closed world)
     └─ lintPassed isEmpty   (closed world)

[Ref(:pr-41)]
```

Only `:pr-41` is approved. `:pr-45` left the approved set in the transaction that recorded the base drift — there was no window in which a stale approval stood. (The test suite pins exactly this sequence: `python/tests/test_devdocs_snippets.py`.)

!!! tip "Read the proof, not the transcript"
    Every one of these outcomes is a proof tree with the rule's own `horis:explain` sentence at the root and the observed facts at the leaves. That is the artifact you attach to the merge, the artifact compliance asks for, and the artifact a later verification pass reasons about.

## Where it plugs into your harness

The gate is a pure function of observed facts, so it sits wherever your harness lets deterministic code run *before* a model or a human sees the request:

| Harness | Attach point (from the harness's own docs, per the research survey) | The fragment's job there |
|---|---|---|
| Claude Code | A `PreToolUse` hook (command or `http` handler) returning `permissionDecision` + `permissionDecisionReason`; declared per skill in `SKILL.md` frontmatter `hooks` | Materialize the proposal's facts, run the gate, return the proof as the reason; run **before** `auto` mode's classifier, not instead of it |
| Gemini CLI | A `BeforeTool` hook; policy rules can match `toolAnnotations` | Same — the policy engine keeps its precedence, the fragment supplies the state-aware decision |
| Cursor | `beforeShellExecution` / `preToolUse` hooks with `permission: allow \| deny \| ask` | Same; set `failClosed: true` — Cursor's hooks fail open by default |
| Codex, pi, anything over MCP | An MCP proxy or `pi.on("tool_call", …)` extension | The deterministic control the MCP maintainers themselves recommend over trusting tool annotations |

A sketch of the Claude Code shape, so the mechanics are concrete:

```python title="A PreToolUse hook — sketch, not shipped code; check your harness's hook contract"
import json, sys
from horismos import Ontology, World

event = json.load(sys.stdin)                          # the tool call the harness is about to make
facts = observe(event)                                # YOUR deterministic code: base hash, test status, paths…

world = World(Ontology.load("change-gate.ttl"))
ref = world.assert_(":ChangeProposal", subject=":this", **facts)
decision = world.value(ref, ":gateDecision")
proof = world.explain(ref, ":gateDecision")

verdict = {None: "ask", ":Approved": "allow", ":NeedsHuman": "ask", ":Rejected": "deny"}
print(json.dumps({"hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": verdict[decision.curie if decision else None],
    "permissionDecisionReason": proof.render() if proof else "no rule fired — needs a human",
}}))
```

`observe()` is deliberately yours: computing `baseMatchesHead` is a hash comparison, `testsPassed` is a sandbox result, `touchesProtectedPath` is a glob match. Horismos does not do arithmetic or run your tests; it decides what the results imply and proves it. Turning those observations into a versioned, all-or-nothing snapshot with a conditional commit is what the coding-harness adapter in PRP-008 formalizes.

## Skills as capability fragments

A `SKILL.md` today declares *when to use* a skill and *which tools it may call*. What it cannot declare is a precondition over world state, a bound on observable effect, or a proof. The framework's name for that missing piece is a **Capability Fragment**: a small ontology overlay attached to an existing skill or tool that proves when the capability is eligible, constrains its effect, and emits a proof — without replacing the harness's own state, sandbox, or approval machinery.

The change gate above *is* a capability fragment written by hand. The authoring agent's compiler mode (PRP-007) is meant to derive one from an existing `SKILL.md`, tool JSON Schema, or `AGENTS.md`: lift the machine-readable structure deterministically, let a model propose only the prose-derived conditions, verify, and emit a **formalization coverage report** that says which clauses became checkable constraints, which are test-backed, and which remain prose. See [Authoring](framework/13-authoring.md).

## The candidate-artifact boundary

A coding agent necessarily generates novel text — patches, files, commands. That does not fit an ordinary `ActionStep`, whose arguments are read from the graph, and pretending it does would weaken *neural proposes, symbolic decides* by stealth. The framework's honest extension is the **candidate-artifact boundary** ([LLM Boundaries](framework/06-llm-boundaries.md), [Tools](framework/07-tools.md)):

1. Model output is a typed **proposal envelope** — patch, file, or command — never a tool argument.
2. **Deterministic policy** validates target paths, base hashes, syntax, and permissions.
3. A **sandbox** executes the declared checks.
4. Only then does a **symbolic authorization step** — the change gate above — permit application.

What the boundary proves: the proposal was policy-legal and passed its declared checks. What it does not and cannot prove: that the code is semantically correct. That stays a review and test claim, and the framework says so wherever the boundary is described.

## Built today, landing next — honestly

| Piece | Status |
|---|---|
| Typed proposal facts, RulesOnly decision, proofs, truth maintenance, the gate refusing self-approval | **Runs today** — everything on this page executed against the shipped engine |
| Workflow around it (`ReasoningStep` with `onUnreachable` → human, `ActionStep` firing a merge tool with a validated effect) | Declared vocabulary and Rust interpreter exist today (`make demo` runs the triage workflow); a change-gate workflow is a Turtle exercise, not new engine work |
| Base-hash check, sandbox execution, and the proposal envelope as a first-class typed boundary | **Landing in PRP-008** (host-adapter/runtime side) — the envelope's `horis:` vocabulary is minted jointly with PRP-007 |
| A coding-harness adapter that materializes repository state (branch, dirty files, test status, path scopes) as a versioned projection with conditional commit | **Landing in PRP-008** — the second-domain proof that the host SPI is format-neutral, not FHIR-shaped |
| Compiling a fragment from an existing `SKILL.md` / tool schema, with a coverage report | **Planned in PRP-007** (after PRP-008) |
| Completeness / conflict / dead-rule verification of the eight rules, with domain-term witnesses | **Yes** — `verify()` proves `:gateDecision` total and conflict-free; `Ontology.load` refuses the overlay otherwise |

Track the live status on the [roadmap](roadmap.md) and in `PRPs/BOARD.md`.
