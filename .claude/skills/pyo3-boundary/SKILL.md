---
name: pyo3-boundary
description: Use before touching crates/sigil-py, python/sigil, .pyi stubs, or pyproject/maturin config — covers the marshal-at-the-boundary rule, no-logic-in-bindings rule, error mapping to the Python SigilError hierarchy, GIL release policy, abi3 constraints, and the maturin/uv build loop.
---

# PyO3 boundary conventions

Spec: `docs/framework/11-python-api.md` (surface), `14-architecture.md` (boundary
posture). Proven pattern inherited from sigil-lang's `pyo3-ext`.

## The three boundary rules

1. **No logic in `sigil-py`.** It marshals; `sigil-core` decides. If you are writing
   an `if` about domain behavior in binding code, it belongs in core.
2. **Whole values across, once.** Hand a whole input over, get a whole outcome back —
   never chatty per-triple round-trips inside a transaction (the sigil-lang benchmark
   lesson: marshaling dominated small workloads).
3. **The pure-Python package is the public surface.** `python/sigil/` re-exports and
   pythonizes; `sigil._sigil` (the native module) is private. Users never import
   `_sigil` directly.

## Build loop

```sh
make develop    # uv sync + maturin develop --uv  → editable install in .venv
make test-py    # pytest against the freshly built module
```

The extension is abi3-py311 (`pyo3 = { features = ["abi3-py311"] }`): one wheel per
platform, Python ≥ 3.11. Don't use APIs outside the stable ABI.

## Conventions

- **Errors**: Rust `SigilError` variants map 1:1 to the Python `SigilError`
  hierarchy (`OntologyError`, `VerificationFailed`, `ValidationRejected`,
  `WorkflowError`, `BridgeError`, `ToolError`). Every raised error carries the
  domain-term message (`domain_text()`) as `str(exc)` and the machine-readable
  structure as exception attributes. Never raise bare `RuntimeError`/`ValueError`
  from binding code.
- **GIL**: release it (`py.detach` / allow-threads) around any core call that can
  run long — chaining, verification, load. Never call back into Python while
  detached.
- **Stubs**: `python/sigil/_sigil.pyi` is updated in the *same change* as any native
  surface change; `make lint` (mypy --strict) is the gate. `py.typed` ships in the
  package.
- **Naming**: Rust `snake_case` methods surface as Python `snake_case`; reserved
  words get a trailing underscore (`assert_`, matching the documented API).

## Checklist before finishing

`make develop && make test-py && make lint` — all green, stubs in sync, no logic
added to `sigil-py`.
