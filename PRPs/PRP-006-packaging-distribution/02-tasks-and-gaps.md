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
  index — the `v*` tag push and the manual dispatch alike — must run from a `v`-prefixed
  tag ref whose version matches the root `Cargo.toml`, compared in Cargo's SemVer
  spelling on both sides. A mismatch or a non-tag ref fails with both values named. The manual dispatch
  route to `pypi` stays available. TestPyPI carries no tag requirement.
- **Both indexes strict, no `skip-existing`.** A duplicate filename is answered with
  HTTP 400 on TestPyPI exactly as on PyPI. Each rehearsal therefore needs a fresh
  version — bump the alpha suffix (`a1`, `a2`, `a3`); TestPyPI version numbers are free.
  A rehearsal that silently uploaded nothing and reported success would destroy the
  evidence the manual checks below depend on.
- **A build-and-verify route that publishes nowhere.** The dispatch input takes a third
  value, `none`, which runs the same wheel matrix, sdist and quickstart gate and then
  skips the publish job. It is not offline: the sdist test's source build still fetches
  its build backend from PyPI. The default stays `testpypi`.

## Landed (2026-09-23, branch `fm/horismos-pybridges-a1`)

The model bridges in the wheel, and the aarch64 wheel gated. Captain's decisions of
2026-09-23: one wheel with everything; the aarch64 build moves to a modern toolchain image;
the aarch64 wheel gets its own test job.

- **Bridges compiled in.** `[tool.maturin] features` adds `bridges-jev` and
  `bridges-openrouter`, so `pip install horismos` reaches both through `horismos.bridges`
  and `horismos.Agent`. Every crate's own default stays empty; that list is the only
  opt-in, and `scripts/generate_third_party_notices.py` now reads it rather than
  repeating it, so the notices closure (87 more crates, every one publishing its own
  licence files) cannot drift from the wheel.
- **aarch64 on manylinux_2_28.** With `ring` (via `reqwest`/`rustls`) in the tree, the
  2014 cross image fails: its GCC 4.8.5 defines no `__ARM_ARCH`, which every `ring` from
  0.17.8 to 0.17.14 requires. Forcing `-D__ARM_ARCH=8` builds a wheel that installs and
  passes the quickstart on native arm64, and a `ring` that rejects valid RSA-2048 and
  ECDSA P-384 signatures and fails real TLS with `BadSignature` — measured against the
  same code built with GCC 12, which passes. Rejected. The `manylinux_2_28` cross image
  (GCC 7.5, `__ARM_ARCH 8` predefined) builds a `ring` that passes the same checks and
  real TLS natively at glibc 2.28 and 2.36. Cost, accepted: ARM Linux needs glibc 2.28
  or newer for the prebuilt wheel. The other three targets are unchanged.
- **`test-aarch64`.** The aarch64 wheel now gets the install-and-quickstart gate on
  CPython 3.11–3.14, under QEMU on an x86_64 runner, inside the pinned
  `manylinux_2_28_aarch64` image, i.e. at its glibc floor. The publish job was left
  untouched, so the gate reaches it through `test`, which waits for `test-aarch64`.

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

- **No release gate exercises the bridges' TLS stack.** Every platform's gate runs the
  symbolic quickstart, which never calls a bridge, and the linker drops code nothing
  references — so a `ring` that builds but computes wrong answers (the aarch64 failure
  above) passes it. The fix for that failure was the toolchain, measured by hand; a gate
  that performs a real handshake from the installed wheel needs a provider call with a
  key, so it is not in the release path. (Closed 2026-09-23: the aarch64 wheel, built
  but never executed before, now has `test-aarch64`.)

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

## Deferred from review (2026-09-21, branch `fm/horismos-pypi-a1`)

Open and unresolved, deliberately left for a follow-up: the review that raised them
judged each documentation accuracy or test coverage rather than a correctness or safety
defect. `README.md` becomes the permanent PyPI long description at the first upload, not
at merge, and nothing can upload while the environments and Trusted Publishers above do
not exist — so there is an editing window after this branch lands.

- **The README's fail-closed message may be wrong** (`README.md`). The README quotes the
  message the verifier emits when `:r3` is dropped from the example ontology. The review
  reads `render_region_conditions` (`crates/horismos-core/src/verify/rules.rs:2123-2125`)
  and its wrapper (`rules.rs:1178-1181`) as producing "no acuity assigned when no
  heartRate assigned" — the uncovered region being the *absent* heart rate,
  `RegionPredicate::Absent { value: None }`, the shape asserted at
  `crates/horismos-core/src/verify/mod.rs:603` — and reads "heartRate < 0" as
  unreachable. The fix round that wrote the sentence reports having measured the quoted
  message by deleting `:r3` and running it. The two accounts conflict and one of them is
  wrong. Settling it means re-running the deletion against the current build and quoting
  what is actually emitted, or cutting the quote and stating the behaviour without
  quoting output.
- **No gate executes the README example** (`README.md`). The example carries a fresh
  inline ontology and pins exact proof-tree output, but nothing runs it: `mkdocs.yml`
  sets `docs_dir: docs` and lists `README.md` under `exclude_docs`, and
  `python/tests/test_devdocs_snippets.py` walks only its hardcoded `PAGES` map under
  `docs/`. This repository has no PR-triggered CI either. A later change to
  `explain().render()` spacing, the `via <rule>` prefix, the `(asserted by the api)`
  provenance label, or the `Ontology.load` / `onto.ref` signatures would leave the
  published block silently stale. Either bring the README example under the existing
  executable-snippet harness so drift breaks `make test`, or drop the pinned output so
  there is nothing to go stale.
- **Two brief constraints were superseded by ruling** (`README.md`, `CONTRIBUTING.md`).
  The original brief for this branch said `docs.horismos.ai` must not be used because the
  domain was not live, and scoped the README change to one short paragraph naming the
  prebuilt platforms. Both were deliberately overridden later: the domain was re-verified
  live (`docs.horismos.ai` returns 200; `horismos-hq.github.io/horismos/` 301-redirects
  to it), and the README was rewritten end to end for the package audience with the
  contributor material moved verbatim into a new `CONTRIBUTING.md`. Recorded so the
  divergence is traceable; it is bookkeeping and needs no change.
