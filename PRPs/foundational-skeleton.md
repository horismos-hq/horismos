# PRP — Foundational Skeleton

**Status**: Done (2026-07-28) · standing infrastructure document
**Depends on**: PRP-000 (positioning), docs/framework/14 (layout authority)

Unnumbered because it is not a phase: it is the substrate every phase assumes —
repo layout, build tooling, agentic-session conventions (AGENTS.md), and the
Claude skills roster. Revised whenever the substrate changes.

## Intent

PRP-001 starts from `make verify` green, not from an empty directory. The skeleton
encodes the architecture's non-negotiables (anti-wrapper boundary, crate ownership,
determinism posture) as *mechanical checks* before any logic exists, so they are
enforced from commit one rather than retrofitted.

## Layout (as built — authority: docs/framework/14-architecture.md)

```
horismos/
├── Cargo.toml                 # [workspace] resolver 3, edition 2024, shared lints
├── rust-toolchain.toml        # stable + rustfmt/clippy + wasm32 target
├── Makefile                   # THE dev entry point (make ships with macOS; just isn't installed)
├── pyproject.toml             # maturin backend, uv dep groups, ruff/mypy/pytest config
├── AGENTS.md                  # session conventions; points here and at docs/ (CLAUDE.md = compat symlink)
├── crates/
│   ├── horismos-core/            # ontology/ world/ reason/ workflow/ verify/ trace/ bridge/ error wasm
│   ├── horismos-bridges/         # mock + feature-gated anthropic/openai/onnx stubs
│   ├── horismos-py/              # PyO3 cdylib `_horismos` — marshaling only
│   └── horismos-fhir/            # [declared-future, PRP-008] host adapter crate — same
│                                   #   dependency posture as horismos-bridges (depends on
│                                   #   horismos-core, never the reverse; free to depend on
│                                   #   an HTTP client); reserved here, built there
├── python/horismos/              # pure-Python surface, py.typed, _horismos.pyi
├── python/tests/              # pytest (smoke: native roundtrip)
├── examples/triage/           # triage.ttl seeded from docs 02–05; grows with every PRP
├── docs/framework/  PRPs/  planning/
└── .claude/skills/            # project-local skills (roster below)
```

**Pivot note (2026-08-24)**: the `horismos-fhir` crate above is a layout reservation only
— no code, no `Cargo.toml` member, no gate. Building it, and the "no network inside
evaluation" fixtures/gates that would prove the anti-wrapper boundary still holds once it
exists, are `PRPs/PRP-008-standards-host-integration.md`'s implementation work, not this
skeleton's. See `docs/framework/14-architecture.md` §"Two-boundary data flow" for the
mechanism this crate will implement.

Crate stubs are **docs-only** (module trees + ownership doc-comments, no premature
API): each module states its spec doc and owning PRP. The only real code is the
version roundtrip (`horismos_core::VERSION` → `horismos.__version__`) proving the
PyO3/maturin pipeline works.

## Tooling decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Task runner | Makefile | zero-install on macOS; `just` not present on dev machine |
| Rust | stable (pinned via rust-toolchain.toml), edition 2024, MSRV stable−2 | docs/framework/12 policy |
| Lints | `unsafe_code = deny`, clippy `-D warnings`, `unwrap_used = warn` | workspace-level, inherited by all crates |
| Python env | uv (`uv sync`, dep group `dev`) | fast, lockfile, builds the wheel via maturin backend |
| Wheel | maturin, abi3-py311, `python-source = python`, module `horismos._horismos` | PRP-003 pattern proven in sigil-lang |
| Py QA | ruff (lint+format), mypy `--strict`, pytest, hypothesis | property-test parity with proptest |
| Boundary enforcement | `make check-boundary` (grep gate) + `make no-model-gate` (build/test core with no default features) | doc 14's CI jobs, runnable locally from day one |
| CI | **deferred to PRP-006** | skeleton stays local-tooling only |

Open items recorded here, decided elsewhere: PyPI name `horismos` availability
(PRP-006), license files MIT+Apache-2.0 dual (PRP-000 confirm), `biodivine-lib-bdd`
dependency lands when `verify` grows real code (PRP-002/005).

## Claude skills roster

Research findings (2026-07-28): Anthropic's official skills repo has **no** Rust or
Python development skills — only `skill-creator` (meta) and `claude-api` matter.
PyO3-specific community skills are thin; boundary conventions must be authored
locally. Strong external collections, referenced not vendored:

- [leonardomso/rust-skills](https://github.com/leonardomso/rust-skills) — 265 rules,
  exemplary progressive-disclosure structure (also the template our skills imitate)
- [apollographql/skills](https://github.com/apollographql/skills) `rust-best-practices`
- [wdm0006/python-skills](https://github.com/wdm0006/python-skills) — uv/ruff/pytest/
  Hypothesis/API-design, matches our stack exactly
- [wshobson/agents](https://github.com/wshobson/agents) — `python-type-safety`,
  `python-project-structure`
- obra/superpowers — already installed globally (TDD, systematic debugging,
  verification-before-completion); cited from AGENTS.md, not duplicated

**Project-local skills** (`.claude/skills/`), per official authoring guidance
(description = discovery trigger; SKILL.md < 500 lines as a table of contents;
scripts for fragile sequences):

| Skill | Purpose | When |
|-------|---------|------|
| `cargo-workspace-map` | crate/module ownership + dependency rules; prevents code landing in the wrong crate | ✅ shipped |
| `ontology-modeling-conventions` | `horis:` vocabulary, IRI scheme, 11 operators, fixture style | ✅ shipped |
| `pyo3-boundary` | error mapping, GIL rules, stub regeneration, marshal-at-boundary conventions | ✅ shipped |
| `learn-codebase` | generates the "Learning Horismos" onboarding artifact: unit-curriculum HTML with diagrams + code walkthroughs | ✅ shipped |
| `generate-prp` | drafting a new PRP: lifecycle, D/U-item template, [DECIDE] two-way links, board discipline | ✅ shipped (2026-07-29) |
| `impl-prp` | executing an Active PRP: worktree, ledger, unit batching, briefs, reviews, finish sequence | ✅ shipped (2026-07-29) |
| `git-workflow` | branch naming, worktrees, stash hazard, commit granularity, gate rules, PR flow | ✅ shipped (2026-07-29) |
| `self-review` | pre-report audits: determinism, boundary, errors, gate, tests, Rust/Python specifics | ✅ shipped (2026-07-29) |
| `brainstorming` | research-aware decision protocol for [DECIDE] questions: evidence sweep → options → adversarial pass → D-item record | ✅ shipped (2026-07-29) |
| `rdf-turtle-fixtures` | fixture authoring + validator script loop | superseded — `ontology-modeling-conventions` + the U2 parser tests cover it; revive only if fixture authoring gets scripted |
| `proptest-strategies` | shared strategies, proptest↔Hypothesis parity rules | PRP-003 (parity needs the Python surface; Rust-side strategies live in `tests/gate_properties.rs`) |
| `release-and-stubs` | scripted version-bump → wheel → stub-check sequence | PRP-006 |

**Project-local agents** (`.claude/agents/`):

| Agent | Purpose | When |
|-------|---------|------|
| `senior-reviewer` | batch/branch diff review: spec compliance, named risks, repo-calibrated severity (gate bypass / boundary / nondeterminism = Critical) | ✅ shipped (2026-07-29) |

## Definition of done

`git clone` → `make verify` green with no network beyond crates.io/PyPI: fmt, clippy
`-D warnings`, ruff, mypy strict, Rust tests, pytest native-roundtrip, anti-wrapper
boundary + no-model-gate. AGENTS.md and the three "now" skills exist and agree with
docs/framework. **Met 2026-07-28.**
