# Examples

The example set — the planned entries come from docs/framework/14-architecture.md; the
docs-site overlays ship today:

| Example | Purpose | Arrives with |
|---------|---------|--------------|
| `triage/` | Flagship: clinical triage — ttl + runs + tests, exercises every feature | seeded now; grows with every PRP |
| `no-llm-monitoring/` | The anti-wrapper demo: full run with structured input, zero LLM | PRP-004 |
| `mcp-tools/` | Tool bindings over MCP | PRP-004 |
| `taubench-harness/` | τ-Bench airline benchmark harness | PRP-000 decision, then PRP-004+ |
| `change-gate/` | The coding-agent change gate: a 23-term capability fragment (rules only; runs today) — see the docs site's "Governing a coding agent" | docs site (2026-09) |
| `gallery/` | Small illustrative overlays behind the docs site's use-case gallery (VTE prophylaxis, claims adjudication, safety envelope, rebooking, CI triage); rules only, every one exercised by `python/tests/test_devdocs_snippets.py` | docs site (2026-09) |
