---
name: impl-prp
description: Use when implementing an Active Horismos PRP — "start PRP-00N", "build the units", "continue the PRP-001 run", or resuming an interrupted implementation session. Covers the activation gate (all D-items resolved first), worktree/branch setup, the progress ledger, unit batching, per-batch briefs and reviews, TDD RED/GREEN evidence, make verify per commit, parked-findings rulings, gap recording, and the finish sequence (final review, PR, board update).
---

# Implementing a PRP

Plan authority: `PRPs/PRP-NNN-slug/02-tasks-and-gaps.md`. Lifecycle:
`PRPs/CONVENTIONS.md`. Git rules: `.claude/skills/git-workflow`. Pre-report checks:
`.claude/skills/self-review`. Reviewer: the `senior-reviewer` agent
(`.claude/agents/senior-reviewer.md`).

This is distilled from the PRP-001 run.

## Preconditions (do not start a unit until all are true)

1. PRP status is **Active** and it is in folder form.
2. **Every D-item is checked** with a dated resolution line. An unresolved D-item
   blocking your unit stops the run — resolve it via `.claude/skills/brainstorming`
   and record it in the PRP. Decisions are never made in code comments, commit
   messages, or reports.
3. `make verify` is green on the baseline commit. Record that commit SHA.
4. You are on a `prp-NNN-slug` branch in `.claude/worktrees/prp-NNN-slug/`, not main.

## The working record — the PRP's own `02-tasks-and-gaps.md`, plus local scratch

The **durable** record is the owning PRP's `02-tasks-and-gaps.md` (D-item
resolutions, U-item check-offs, rulings, gap notes) and the shipped-voice text left
where each `[DECIDE]` marker was struck — never a parallel tracker (the
`.superpowers/` process directory is gitignored scratch: use it for in-flight notes
if helpful, but nothing lives only there). In-flight state must still survive
context loss — a fresh session must be able to read the PRP folder plus the branch
and continue. Track during a run:

- header: plan path, branch, worktree, baseline SHA;
- **controller resolutions** — interfaces pinned across units (dep choices, type
  shapes, which crate owns what), so unit N+1 doesn't re-decide unit N's API;
- one line per unit: `Task U6: complete (commits <base>..<head>, review clean)`;
- `Task UN: minor (deferred): …` for parked findings;
- `Task UN: ruling: …` for judgement calls that are not plan changes;
- `Task UN: parked — <finding> — ruling: <where it is handled instead>`;
- pacing changes, with who approved them.

Briefs and implementer reports are local scratch files (same naming, don't invent
new ones): `task-U6-U7-brief.md`, `task-U6-U7-report.md` — anything in them worth
keeping (rulings, deferred minors, gaps) is copied into `02-tasks-and-gaps.md`.

## Batching

Run adjacent units as batches once the shape of the code is established (PRP-001 ran
U1 solo, then B1=U3+U4, B2=U5, B3=U6+U7, B4=U8+U9, B5=U10+U11). Per batch: one brief,
one implementer pass, **one reviewer**, `make verify` at the end — but **separate
commits per unit**, each independently green. Batching is a pacing decision; record it
in the ledger with who approved it.

## The brief (write before any code)

One file per batch, `task-U<N>[-U<M>]-brief.md`, containing:

1. **Task text verbatim** from the plan, quoted — never paraphrased.
2. **Governing decisions verbatim** (the D-items this batch is bound by).
3. **Controller pins** — cross-unit interface decisions made outside the plan, with
   the reason ("use `imbl` NOW in U6 so U8's `snapshot()` is O(1), not a retrofit").
4. **Required reading before writing code** — the relevant `.claude/skills/*`, the
   governing `docs/framework/NN-*.md` ("read it word by word"), and the existing
   source files this batch extends.
5. **Numbered requirements** per unit, each testable, including the exact domain-text
   strings the tests must assert.
6. **Process**: TDD per unit, separate commits, `make verify` at batch end, the
   commit trailer, and any new dependency (name + why it is boundary-safe).
7. **Out of scope** — the neighbouring units and PRPs, explicitly, so the implementer
   doesn't overbuild.
8. **Carried-forward concerns** — reviewer concerns and deferred minors from earlier
   batches that this batch must address ("U5 MUST address serialize() panicking on a
   hand-built Document — carried from Batch 1 minors").

## TDD, per unit

RED then GREEN, with **pasted evidence** in the report — a compile error naming the
missing module, or named failing tests. A compile-only RED is weak: when the type
exists but the behaviour doesn't, add the variant first and show the *behavioural*
failure (PRP-001's U7 did exactly this). Clippy counts: `-D warnings` RED → GREEN is
worth recording.

Tests assert **real behaviour and exact domain text** — run against the real
`examples/triage/triage.ttl` wherever the fixture supports it; use small inline
ontologies only for constraints triage.ttl doesn't exercise. Every rejection test
asserts both the machine-readable structure and the rendered sentence.

## Commit rhythm

One unit (or one coherent step of it) per commit; message = imperative summary naming
the unit: `Add the World instance store and its five-check validation gate (U6)`.
`make verify` must be green before any commit that claims a unit done. Details in
`.claude/skills/git-workflow`.

## Review, per batch

Hand the `senior-reviewer` agent: the brief, the implementer report, and a diff file
(a `git diff <base>..<head>` file in local scratch, regenerable — never committed) or the
range. The report must **name its risks** — the places the implementer is least sure
of — and the reviewer checks each one specifically. Severity calibration and output
format live in the agent definition.

Then:

- **Critical/Important** → fix in this batch, before the batch is called complete.
- **Minor** → park it: one ledger line `Task UN: minor (deferred): …`. Parked findings
  that a later unit must handle get **carried into that unit's brief**.
- **Disagreement** → a ruling line in the ledger stating what was decided and why; if
  the disagreement is really an open design question, it is a `[DECIDE]` for a PRP,
  not a ruling.

## Gaps and discoveries

Anything the plan missed goes into the PRP's `02-tasks-and-gaps.md` under **Gaps
discovered during implementation** as a dated note — appended, never absorbed into a
unit silently. If it changes scope, edit `01-objective.md` explicitly in the same
commit. If it is an unowned design question (PRP-001's "may an immutable fact be
retracted and re-asserted?"), add a `[DECIDE: … — PRP-NNN]` marker to the governing
doc and a D-item to the owning PRP. Never decide it in code.

## Finishing

1. All U-items complete, `make verify` green, DoD unit run against triage.
2. **Final whole-branch review** — `senior-reviewer` over `main..HEAD`, not just the
   last batch.
3. Open/finalize the PR with `gh-axi` (draft PR should already exist from unit 1;
   see `.claude/skills/git-workflow`), body = the DoD with evidence.
4. **Status surfaces, all in one commit**: check the U-item boxes in
   `02-tasks-and-gaps.md`, flip the PRP `**Status**:` line to Done with the date,
   update `PRPs/BOARD.md` (Current state, Next goal, Board row, `_Last updated:_`),
   regenerate `planning/board.html` and republish it to the **existing** artifact URL.
5. Leave the ledger, briefs, and reports in place — they are the record of how the
   phase was built.
