---
name: git-workflow
description: Use before any git operation in the horismos repo — branching, committing, stashing, worktrees, PRs, pushing. Covers the never-implement-on-main rule, prp-NNN-slug branch naming, the .claude/worktrees/ layout and its shared-stash hazard, commit granularity and message form, the make verify precondition, the board-in-the-same-commit rule, and gh-axi PR conventions.
---

# Git workflow

## Branches and worktrees

- **Never implement on `main`.** If you find yourself on main with changes, branch
  first, then commit.
- Branch name = the PRP slug, lowercased: `prp-001-ontology-core`,
  `prp-002-reasoning-substrate`. Non-PRP work: `fix-<slug>` / `docs-<slug>`.
- Isolated work happens in a worktree at `.claude/worktrees/<branch-name>/` — the
  directory name matches the branch exactly. `.claude/worktrees/` is git-ignored.
- **Worktrees share one stash stack with the main checkout.** Never use bare
  `git stash` here — another session's worktree can pop your entry. Prefer a
  **WIP commit** on your own branch (`git commit -m "WIP: <what>"`, amend or fixup
  later). If you must stash, use a named `git stash push -m` and pop it yourself in
  the same turn.
- `git status` reporting clean is **not** proof the tree is clean at the moment you
  start: a prior interrupted session in the same worktree may have left uncommitted
  work (this happened in the PRP-001 U6 batch). Re-check `git status` and
  `git diff --stat` at the start of every session and reconcile before writing code.

## Committing

| Rule | Value |
|---|---|
| Granularity | one unit of work, or one coherent batch step, per commit |
| Message | imperative summary that **names the unit**: `Add the World instance store and its five-check validation gate (U6)` |
| Precondition for "done" commits | `make verify` green — check-boundary, lint (fmt, clippy `-D warnings`, ruff, mypy), tests (Rust + pytest), no-model-gate |
| Trailer | `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` |
| Independence | each unit commit must compile and pass tests **on its own**, not just at branch head (verify by checking the SHA out into a throwaway worktree if in doubt) |

Body: what changed and why, not a file list. If a commit resolves a decision or
records a ruling, say which D-item — but the decision itself lives in the PRP, never
only in the commit message.

## Never weaken the gates

`make check-boundary` and `make no-model-gate` are the anti-wrapper claim in
mechanical form. Never edit the grep in `check-boundary`, never add a default feature
that smuggles a networked dep into `horismos-core`, never `--no-verify`. If a
dependency breaks a gate, that is a finding to park or a `[DECIDE]`, not a Makefile
edit. (Precedent: oxrdf's transitive `rand`/`getrandom` broke the wasm32 build; the
ruling was "wasm-gate is PRP-006's CI job", not a loosened target.)

Adding any dependency: root `[workspace.dependencies]` first, then
`workspace = true` in the crate. Core deps must be pure — no network, no provider SDK.

## Pull requests

- Use **`gh-axi`** (the agent-ergonomic GitHub CLI wrapper) in preference to `gh`.
- Open a **draft PR early** — after the first unit lands, not at the end. Title = the
  PRP title; body = the U-item checklist copied from `02-tasks-and-gaps.md`, ticked as
  units land, plus the DoD.
- PR bodies end with:
  `🤖 Generated with [Claude Code](https://claude.com/claude-code)`
- Mark ready for review only after the final whole-branch review and `make verify`.

## Board updates ride along

Any commit that changes a PRP's status must also carry, in the **same commit**:
`PRPs/BOARD.md` (Current state, Next goal, Board row, `_Last updated:_`), the PRP's
own `**Status**:` line, and `planning/board.html` — republished to the **existing**
artifact URL recorded in BOARD.md's header. Never mint a new artifact URL.

## Pushing

Push only when the user asks. Same for tags, releases, force-pushes, and merges to
main. Committing locally is normal; publishing is a request-gated act.
