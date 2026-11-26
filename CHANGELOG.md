# Changelog

All notable changes to Horismos are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) in its 0.x form: within 0.1.x a
release does not break the documented Python surface (`horismos.__all__`) or the public
Rust API; a breaking change moves to 0.2.0 and is listed here.

## [0.1.0] - 2026-11-26

The first stable release: `pip install horismos` resolves to it without `--pre`. It ships
abi3 wheels for CPython 3.11 and newer on Linux x86_64 (glibc 2.17 or newer), Linux
aarch64 (glibc 2.28 or newer), macOS universal2 and Windows x86_64, plus an sdist.

### Included

- Ontology loading with fail-closed BDD verification: completeness, conflicts, dead
  rules, termination and workflow checks, each finding carrying a witness in domain terms.
- A typed World behind the five-check validation gate.
- Forward and backward chaining with truth maintenance and proof trees.
- `horismos.trace/2` traces.
- The workflow engine and the agent loop, with the mock, Jev and OpenRouter bridges
  compiled into the wheel.
- Packaged example ontologies (`horismos.examples`) and `python -m horismos demo`.

### New since 0.1.0a4

- The `horismos[mujoco]` extra (pins `mujoco==3.14.0`) for the repository's MuJoCo
  shield demo (#37).
- The verifier names an unassigned numeric input as "no X assigned" (#43).
- The documentation leads with the Physical AI shield (#39, #40, #41).
- Shield decision-latency benchmarks with recorded runs (#38).
- Pull-request CI runs `make verify` and the strict docs build (#42).

### Not included / known limits

- Not published on crates.io.
- Not a safety-rated component.
- Open security decisions: the redaction default, trace-store encryption hooks, and an
  audit/SBOM release gate.
- CPython 3.15 is untested.

[0.1.0]: https://github.com/horismos-hq/horismos/releases/tag/v0.1.0
