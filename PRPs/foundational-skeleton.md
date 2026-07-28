# PRP — Foundational Skeleton

**Status**: Done (2026-07-28) · standing infrastructure document
**Depends on**: PRP-000 (positioning), docs/framework/14 (layout authority)

Unnumbered because it is not a phase: it is the substrate every phase assumes —
repo layout, build tooling, agentic-session conventions (CLAUDE.md), and the
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
├── CLAUDE.md                  # session conventions; points here and at docs/
├── crates/
│   ├── horismos-core/            # ontology/ world/ reason/ workflow/ verify/ trace/ bridge/ error wasm
│   ├── horismos-bridges/         # mock + feature-gated anthropic/openai/onnx stubs
│   └── horismos-py/              # PyO3 cdylib `_horismos` — marshaling only
├── python/horismos/              # pure-Python surface, py.typed, _horismos.pyi
├── python/tests/              # pytest (smoke: native roundtrip)
├── examples/triage/           # triage.ttl seeded from docs 02–05; grows with every PRP
├── docs/framework/  PRPs/  planning/
└── .claude/skills/            # project-local skills (roster below)
```

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
  verification-before-completion); cited from CLAUDE.md, not duplicated

**Project-local skills** (`.claude/skills/`), per official authoring guidance
(description = discovery trigger; SKILL.md < 500 lines as a table of contents;
scripts for fragile sequences):

| Skill | Purpose | When |
|-------|---------|------|
| `cargo-workspace-map` | crate/module ownership + dependency rules; prevents code landing in the wrong crate | now |
| `ontology-modeling-conventions` | `horis:` vocabulary, IRI scheme, 11 operators, fixture style | now |
| `pyo3-boundary` | error mapping, GIL rules, stub regeneration, marshal-at-boundary conventions | now |
| `learn-codebase` | generates the "Learning Horismos" onboarding artifact: unit-curriculum HTML with diagrams + code walkthroughs | now |
| `rdf-turtle-fixtures` | fixture authoring + validator script loop | PRP-001 (needs the parser) |
| `proptest-strategies` | shared strategies, proptest↔Hypothesis parity rules | PRP-001 |
| `release-and-stubs` | scripted version-bump → wheel → stub-check sequence | PRP-006 |

## Definition of done

`git clone` → `make verify` green with no network beyond crates.io/PyPI: fmt, clippy
`-D warnings`, ruff, mypy strict, Rust tests, pytest native-roundtrip, anti-wrapper
boundary + no-model-gate. CLAUDE.md and the three "now" skills exist and agree with
docs/framework. **Met 2026-07-28.**
