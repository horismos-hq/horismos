# Horismos dev entry points. Requires: rustup (stable), uv. Everything Python
# runs through uv; maturin is a dev dependency, never a global install.

.PHONY: build develop test test-rust test-py test-workflows lint fmt no-model-gate check-boundary verify clean demo demo-live demo-jev demo-jev-live

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
	uv run ruff check python
	uv run ruff format --check python
	uv run mypy

fmt:
	cargo fmt --all
	uv run ruff format python
	uv run ruff check --fix python

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

clean:
	cargo clean
	rm -rf .venv dist
