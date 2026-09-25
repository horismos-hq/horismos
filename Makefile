# Horismos dev entry points. Requires: rustup (stable), uv. Everything Python
# runs through uv; maturin is a dev dependency, never a global install.

.PHONY: build develop test test-rust test-py test-workflows lint fmt notices no-model-gate check-boundary verify clean demo demo-live demo-jev demo-jev-live demo-jev-py demo-jev-py-live demo-shield

build:
	cargo build --workspace

# Build the extension module and install the horismos package editable into .venv
develop:
	uv sync
	uv run maturin develop --uv

test: test-rust test-py test-workflows

test-rust:
	cargo test --workspace

test-py:
	uv run pytest

# The release workflow's tag guard, run verbatim from release.yml
test-workflows:
	uv run python .github/scripts/test_release_guard.py

lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	# The provider bridges are feature-gated, so the workspace pass above never
	# compiles them; lint them as the published wheel builds them.
	cargo clippy -p horismos-py --all-targets --features bridges-jev,bridges-openrouter -- -D warnings
	uv run ruff check python scripts examples/mujoco
	uv run ruff format --check python scripts examples/mujoco
	uv run mypy

fmt:
	cargo fmt --all
	uv run ruff format python scripts examples/mujoco
	uv run ruff check --fix python scripts examples/mujoco

# Regenerate THIRD_PARTY_NOTICES.txt (shipped in the wheel and sdist) from the
# release closure. Run after any change to Cargo.lock; `make test` fails while
# it is stale.
notices:
	uv run python scripts/generate_third_party_notices.py

# The anti-wrapper gate: the symbolic core must build and test with no
# default features, no bridges, no network (docs/framework/14-architecture.md)
no-model-gate:
	cargo build -p horismos-core --no-default-features --features verify
	cargo test  -p horismos-core --no-default-features --features verify

# horismos-core must never gain a dependency edge to horismos-bridges or a provider SDK
check-boundary:
	@! grep -qE 'horismos-bridges|reqwest|anthropic|openai' crates/horismos-core/Cargo.toml \
		|| (echo "BOUNDARY VIOLATION: model-related dependency in horismos-core" && exit 1)
	@echo "anti-wrapper boundary: OK"

verify: check-boundary lint test no-model-gate

# PRP-004 U14: the triage-workflow demo against MockBridge — deterministic,
# no network, no key.
demo:
	cargo run -p horismos-bridges --example triage_agent_demo

# Same demo, built with the OpenRouter provider client; uses OpenRouterBridge
# when OPENROUTER_API_KEY is set, else runs MockBridge silently — the first output
# line ("… — mock" vs "… — live (OpenRouter)") says which bridge ran.
demo-live:
	cargo run -p horismos-bridges --example triage_agent_demo --features bridges-openrouter

# The text->decisions demo: a clinical note perceived into examples/vte/vte.ttl's
# four booleans, with the recommendation derived by the rules and printed as a
# proof tree. MockBridge — deterministic, no network, no key.
demo-jev:
	cargo run -p horismos-bridges --example jev_vte_demo

# Same demo, built with the Jev provider client; uses JevBridge when
# OPENROUTER_API_KEY (or TYPESAFE_API_KEY) is set, else MockBridge — the first
# output line ("... - mock" vs "... - live (Jev)") says which bridge ran.
demo-jev-live:
	cargo run -p horismos-bridges --example jev_vte_demo --features bridges-jev

# The same demo from Python, exactly as an installed package runs it
# (`python -m horismos demo`, the packaged vte/perception.ttl), so the step's
# horis:confidence gate is in the loop. Always MockBridge: a key in the
# environment is not consent to spend it.
demo-jev-py: develop
	uv run python -m horismos demo

# The live model, asked for explicitly; needs OPENROUTER_API_KEY or
# TYPESAFE_API_KEY (or a _FILE variant).
demo-jev-py-live: develop
	HORISMOS_DEMO_LIVE=1 uv run python -m horismos demo

# PRP-009 tier 3: the MuJoCo shield demo, headless, fixed seed. MuJoCo is the
# `mujoco` extra, never a core dependency — this target installs it into .venv
# (the next `make develop` removes it again) and asserts the outcome, never
# pixels: the shield-off run touches the human, the shield-on run does not, its
# verdict sequence equals examples/mujoco/expected-verdicts.json, and a second
# run repeats it. Exit code is the assertion; nothing here skips.
demo-shield:
	uv sync --extra mujoco
	uv run maturin develop --uv
	uv run python examples/mujoco/demo.py --check

clean:
	cargo clean
	rm -rf .venv dist
