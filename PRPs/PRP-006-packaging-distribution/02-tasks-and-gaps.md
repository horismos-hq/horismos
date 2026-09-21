# PRP-006 — Tasks & Gaps

## Landed (2026-09-21, branch `fm/horismos-pypi-a1`)

The PyPI release path. Nothing else in the plan was touched.

- **One version, authored once.** The workspace `version` in the root `Cargo.toml` is
  the single source: Cargo SemVer `0.1.0-alpha.1`, which maturin normalises to the
  PEP 440 distribution version `0.1.0a1`. `pyproject.toml` keeps `dynamic = ["version"]`
  rather than repeating the number; `horismos.__version__` reads the installed
  distribution metadata, so it reports the PEP 440 spelling.
- **`.github/workflows/release.yml`**, on Trusted Publishing (OIDC): `id-token: write`
  scoped to the publish job alone, no API token in the workflow or in repository
  secrets. It supersedes the build-only `wheels.yml`, which is deleted — its matrix and
  smoke test are folded in, so they live in one place.
- **Wheel matrix**: Linux x86_64, Linux aarch64, macOS universal2, Windows x86_64 — one
  abi3 wheel per platform for every CPython from 3.11. musl is deliberately skipped for
  this release.
- **Source archive, gated.** The sdist is built and then installed from and exercised by
  a Linux job running the same doc-11 symbolic quickstart the wheel gate runs; publication
  depends on it. This is a multi-crate workspace with path dependencies, so an archive
  missing a crate is the realistic failure and it is now caught before upload.
- **The publish guard.** A PyPI filename can never be reused, so every route to the real
  index — the `v*` tag push and the manual dispatch alike — must run from a tag ref whose
  version matches the root `Cargo.toml`, compared in Cargo's SemVer spelling on both
  sides. A mismatch or a non-tag ref fails with both values named. The manual dispatch
  route to `pypi` stays available. TestPyPI carries no tag requirement.
- **Both indexes strict, no `skip-existing`.** A duplicate filename is answered with
  HTTP 400 on TestPyPI exactly as on PyPI. Each rehearsal therefore needs a fresh
  version — bump the alpha suffix (`a1`, `a2`, `a3`); TestPyPI version numbers are free.
  A rehearsal that silently uploaded nothing and reported success would destroy the
  evidence the manual checks below depend on.
- **A build-and-verify route that touches no index.** The dispatch input takes a third
  value, `none`, which runs the same wheel matrix, sdist and quickstart gate and then
  skips the publish job. The default stays `testpypi`.

## Decisions to resolve (before implementation)

Carried over unresolved; none of them blocked the release path this branch landed, and
this branch resolved none of them. Each is stated where it already lives — nothing new
is recorded here.

- [ ] D1. **Default redaction posture** (opt-in vs. opt-out) for `horis:redactInput`,
      per domain pack. `[DECIDE, doc 15 §4 / doc 09]`, key decision 4 in
      `01-objective.md`; blocks the redaction item of the security acceptance surface.
- [ ] D2. **Encryption-at-rest and access-control hooks** for the default trace store —
      a minimal `TraceStore` wrapper interface vs. fully deployer-owned infra.
      `[DECIDE, doc 15 §4]`, key decision 5 in `01-objective.md`; blocks the trace-store
      privacy item of the security acceptance surface.
- [ ] D3. **SBOM format** (CycloneDX vs. SPDX) and whether the dependency-audit gate
      blocks a release or only warns. `[DECIDE, doc 15 §4]`, key decision 6 in
      `01-objective.md`; blocks the dependency-audit CI gate.
- [ ] D4. **crates.io name availability** for `horismos`, with the fallbacks named in
      key decision 3 of `01-objective.md` (`horismos-core`/`horismos-ai`/
      `horismosframe`) — the open half of that decision, whose PyPI half was decided
      2026-09-21. No crates.io check has been run, so nothing further is recorded here.

## Deliberately not covered

- **The linux-aarch64 wheel is built but never executed.** No aarch64 Linux runner is
  available to this repository, so the smoke-test matrix covers the other three
  platforms only — each on CPython 3.11 through 3.14. This is a deliberate gap, not an
  oversight: it means the aarch64 artefact's first real exercise would be a user's
  install unless it is checked by hand after a TestPyPI rehearsal.

## Open, and not ours to close

- The `pypi` and `testpypi` GitHub environments do not exist yet, and neither Trusted
  Publisher is registered. Both need the captain's PyPI/TestPyPI login. Until they
  exist, nothing can publish.
- The distribution name `horismos` was checked on 2026-09-21: PyPI and TestPyPI both
  returned 404 for it. It is unclaimed, not held — nothing has been uploaded to either
  index.

## Still open from the plan

- **crates.io.** This branch did not touch it; D4 above and `cargo add horismos-core`
  in the DoD remain open.
