# Sigil dev entry points. Requires: rustup (stable), uv. Everything Python
# runs through uv; maturin is a dev dependency, never a global install.

.PHONY: build develop test test-rust test-py lint fmt no-model-gate check-boundary verify clean

build:
	cargo build --workspace

# Build the extension module and install the sigil package editable into .venv
develop:
	uv sync
	uv run maturin develop --uv

test: test-rust test-py

test-rust:
	cargo test --workspace

test-py:
	uv run pytest

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
	cargo build -p sigil-core --no-default-features --features verify
	cargo test  -p sigil-core --no-default-features --features verify

# sigil-core must never gain a dependency edge to sigil-bridges or a provider SDK
check-boundary:
	@! grep -qE 'sigil-bridges|reqwest|anthropic|openai' crates/sigil-core/Cargo.toml \
		|| (echo "BOUNDARY VIOLATION: model-related dependency in sigil-core" && exit 1)
	@echo "anti-wrapper boundary: OK"

verify: check-boundary lint test no-model-gate

clean:
	cargo clean
	rm -rf .venv dist
