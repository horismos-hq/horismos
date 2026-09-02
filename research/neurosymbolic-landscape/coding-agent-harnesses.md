# Coding-agent harnesses: how capabilities, permissions and policies are structured (sweep #2)

## Claim

**[inference — summary of the tagged evidence below]** As of 2026-09-02 the positioning
claim in `PRPs/PRP-000-vision-positioning.md`
§"Comparison" — that no surveyed harness "declare[s] a semantic domain graph with
verified cross-tool preconditions/effects or maintain[s] a fixed point over repository
state" — still holds, with four nuances that put some harnesses closer than the table
implies: a Claude Code skill can now attach enforcement hooks to itself in `SKILL.md`
frontmatter; Gemini CLI's policy engine consumes declared tool annotations as policy
input; base-hash-guarded patch application exists only as third-party tool overrides
that core harnesses declined; and the approval step in Codex, Claude Code `auto` mode,
and Cursor is moving toward a model classifier — the opposite direction from "symbolic
decides." Both the capability-fragment and candidate-artifact designs are confirmed as
gaps no harness fills; the incident record says the failures they target happen below
the prompt.

## Evidence

### Policy languages cannot see repository state

**[measured]** OpenAI Codex CLI rules (v0.147–0.149, Aug 2026) are Starlark
`prefix_rule(pattern=[...], decision="allow"|"prompt"|"forbidden", ...)` entries; the
documentation states rules "evaluate command prefixes only — they cannot reference
repository state, file contents, or environmental conditions." Sandbox modes are
`read-only` / `workspace-write` (default; `.git`, `.agents`, `.codex` read-only,
`network_access = false`) / `danger-full-access` via Seatbelt or bwrap+seccomp
([rules](https://learn.chatgpt.com/docs/agent-configuration/rules);
[approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security)).
v0.149.0 removed `approval_policy = "untrusted"` (a deterministic "known-safe command
set" tier) with no deprecation period; the security page still lists it and issue
#39973 (2026-08-21) is open without maintainer response
([openai/codex #39973](https://github.com/openai/codex/issues/39973)). Attribution of
individual features to v0.147.0 rests on a third-party release write-up → [unverified].

**[measured]** Gemini CLI's policy engine is the most declarative surveyed: TOML rules
over `toolName`, `decision` (`allow|deny|ask_user`; `ask_user` "treated as deny in
headless mode"), `priority`, `modes`, `argsPattern` (regex over args-as-JSON),
`mcpName`, `commandPrefix`, `commandRegex`, `subagent`, `toolAnnotations` (e.g.
`{ readOnlyHint = true }`), with tiered priorities; hooks `BeforeTool`/`AfterTool`/
`BeforeModel`/`BeforeToolSelection` etc. Issue #20469 documents static tool exclusions
being applied *before* the policy engine so `allow` rules were silently ignored in
non-interactive modes ([policy-engine.md](https://github.com/google-gemini/gemini-cli/blob/main/docs/reference/policy-engine.md);
[hooks reference](https://github.com/google-gemini/gemini-cli/blob/main/docs/hooks/reference.md);
[#20469](https://github.com/google-gemini/gemini-cli/issues/20469)).

**[measured]** DeepSeek Harness v0.1 (MIT, developer preview, ~2026-08-13/20 per
secondary sources → [unverified]) is "everything is a plugin": `packages/interaction/`
for approval/permission/ask-user, `packages/hooks/` providing "Claude Code/Codex hook
bridges," sandbox policy `read-only`/`workspace-write`/`full-access`, and the invariant
"Model-visible ⟺ logged: anything that reaches a model request must be reconstructable
from the session log" ([AGENTS.md](https://github.com/deepseek-ai/deepseek-harness/blob/master/AGENTS.md);
[README](https://raw.githubusercontent.com/deepseek-ai/deepseek-harness/master/README.md)).
pi ships four core tools (`read`, `write`, `edit`, `bash`), `pi.registerTool`,
`pi.on("tool_call", ...)`, `SKILL.md` skills, and "No permission popups. Run in a
container, or build your own confirmation flow with extensions inline"
([pi README](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/README.md)).
Neither declares preconditions or effects, nor tracks repository state.

### Skills: an attach point exists, a precondition vocabulary does not

**[measured]** Claude Code `SKILL.md` frontmatter fields: `name, description,
when_to_use, argument-hint, arguments, disable-model-invocation, user-invocable,
allowed-tools, disallowed-tools, model, effort, context, agent, background, hooks,
paths, shell, metadata, license, compatibility`. `allowed-tools` is a *pre-approval*
("It does not restrict which tools are available"; "Workspace trust doesn't gate this
field"); `disallowed-tools` removes tools from the pool; `hooks` are "registered when
the skill is invoked and kept running for the rest of the session," e.g. a `PreToolUse`
matcher `"Bash"` running a script; `paths` is an activation glob; `metadata` is a map
Claude Code "doesn't act on" ([skills](https://code.claude.com/docs/en/skills);
[hooks](https://code.claude.com/docs/en/hooks)). The Agent Skills open specification
(agentskills.io) defines `name`, `description`, `license`, `compatibility` ("Max 500
characters … environment requirements"), `metadata`, and `allowed-tools` ("pre-approved
tools … (Experimental)"), plus `scripts/`, `references/`, `assets/`; its validator
checks frontmatter only ([specification](https://agentskills.io/specification)). The MCP
Skills-over-MCP working group's direction is a resources-based extension (SEP-2640)
([charter](https://modelcontextprotocol.io/community/working-groups/skills-over-mcp)).

**[inference]** A Horismos Capability Fragment fits as a `PreToolUse`/`PermissionRequest`
command or `http` hook declared in `SKILL.md`, with `metadata` carrying the fragment IRI
and `scripts/` shipping the deterministic check — attached to, not replacing, the skill.
What no harness offers is the fragment's content: a declarative precondition over
world/repository state, a bound on observable effect, and a proof.

### Permission pipelines: ordered, typed, with reason strings — and a classifier

**[measured]** Claude Code's evaluation order is Hooks → Deny rules → Ask rules →
Permission mode → Allow rules → `canUseTool`; "A hook that returns `allow` does not skip
the deny and ask rules." `PreToolUse` returns `permissionDecision` (`allow|deny|ask`),
`permissionDecisionReason`, `updatedInput`; handler types are `command`, `http`,
`mcp_tool`, `prompt`, and `agent` ("spawn a subagent … to verify conditions before
returning a decision … experimental"). Critical paths are a "circuit breaker" that "never
lets a `permissions.allow` rule or a `PreToolUse` hook that returns `allow` approve." An
MCP server can set `_meta: {"anthropic/requiresUserInteraction": true}` to force a
prompt on every call. Modes: `default, dontAsk, acceptEdits, bypassPermissions, plan,
auto` (model classifier)
([Agent SDK permissions](https://code.claude.com/docs/en/agent-sdk/permissions);
[permission modes](https://code.claude.com/docs/en/permission-modes);
[MCP approval](https://code.claude.com/docs/en/mcp#require-approval-for-a-specific-tool)).
Cursor's hooks include `beforeShellExecution, afterShellExecution, beforeMCPExecution,
beforeReadFile, afterFileEdit, preToolUse, postToolUse`; blocking hooks return
`permission: "allow"|"deny"|"ask"`, and hook failures are **fail-open** unless
`failClosed: true`; Cursor describes its run modes as "best-effort guardrails rather
than a hard security boundary" ([hooks](https://cursor.com/docs/hooks);
[security](https://cursor.com/docs/agent/security)).

**[inference]** The harness already exposes the slots a fragment needs (a typed decision
chain with reason strings, precedence rules, server-declared per-tool constraints it
will honour). The challenge is direction of travel: `prompt`/`agent` hooks, `auto` mode,
Codex `auto_review`, and Cursor Auto-review put a model inside the decision step. A
fragment must be positioned as the deterministic hook that runs before those.

### Containment hardening: per-bug validators, one TOCTOU

**[measured]** Claude Code v2.1.238–2.1.257 (2026-08-20 → 2026-09-01) added
`--restricted` / `CLAUDE_CODE_RESTRICTED=1`, `permissions.blockReadsOutsideWorkingDirectories`,
a "Containment Escape rule" in `auto` mode, and fixed: file tools "following a symlink
swapped inside the working directory after the permission check," Bash deny rules not
applying to `< file` redirects and `tac`/`egrep`, plugin paths escaping via symlink, and
worktree sessions refusing `$VAR` reads/heredocs as "too complex to verify that it
stays inside the worktree" ([changelog](https://code.claude.com/docs/en/changelog)).
Worktree isolation applies four checks — file edits, command working directory, git
redirects, and "command shape" ("blocks a Bash or Monitor command it can't verify stays
inside the worktree … You can't turn this check off"), surfacing refusals as tool
errors with guidance ([worktrees](https://code.claude.com/docs/en/worktrees)).

**[inference]** The symlink-swap bug is a TOCTOU between authorization and effect —
exactly what a base identity check on the target at apply time closes — and the
"command shape" check is a shipped, non-disableable, fail-closed static validator over
proposal *targets*: the same posture as "policy-legal, not semantically correct," and
evidence users accept refusals of unverifiable proposals when the error says how to
proceed.

### Patch envelopes: typed, but no base identity, no policy stage

**[measured]** Codex `apply_patch` (V4A) emits `apply_patch_call` items with operations
`create_file | update_file | delete_file`, `path`, and a diff with `@@` context anchors;
the harness applies it and returns `status: "completed" | "failed"` with outputs such as
"Invalid Context" — no base hash, file version, or expected-content field is defined
([OpenAI docs](https://developers.openai.com/api/docs/guides/tools-apply-patch)).
Claude Code issue #25775 (hashline addressing, "hash mismatch rejects edits before file
corruption") was closed "not planned"; OpenCode #15424 reports its experimental hashline
edit "silently returns success when LINE#ID anchors are stale," also closed; the working
implementations are `pi-hashline-edit` (v0.8.3, 2026-07-14) and a DeepSeek Harness
extension (snippet only → [unverified])
([#25775](https://github.com/anthropics/claude-code/issues/25775);
[#15424](https://github.com/anomalyco/opencode/issues/15424);
[pi-hashline-edit](https://pi.dev/packages/pi-hashline-edit)). ATM
([arXiv:2607.00041](https://arxiv.org/abs/2607.00041)) independently describes a CID
broker that "records a base hash at admission time," re-reads repository state before
applying, and takes "a controlled successor path" on mismatch — single author,
synthetic evaluation, no harness adoption.

**[inference]** The candidate-artifact envelope can wrap V4A verbatim and add the base
hash and declared checks; the hash check belongs in the deterministic policy stage, not
in the editing tool — OpenCode's tool-local check is the documented fail-open case.

### MCP: no typed preconditions/effects; enforcement pushed to clients

**[measured]** The 2026-07-28 MCP revision makes the core stateless, adds multi-round-
trip requests (`resultType: "input_required"`), moves Tasks to an extension, and leaves
tool annotations unchanged (`readOnlyHint, destructiveHint, idempotentHint,
openWorldHint, title`); the maintainers write that annotations "are not guaranteed to
faithfully describe tool behavior, and clients must treat them as untrusted unless they
come from a trusted server" and should "implement deterministic controls rather than
relying solely on hints." SEP-1862 (`tools/resolve(name, args)` returning a per-call
refined `Tool`) is still draft; SEP-1984 (`reversibleHint`, `privilegedAccessHint`, …)
has had minimal engagement since January 2026
([2026-07-28 blog](https://blog.modelcontextprotocol.io/posts/2026-07-28/);
[annotations blog](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/);
[SEP-1862](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/1862);
[SEP-1984](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/1984)).

### The academic convergence and the incident record

**[measured]** Five 2026 groups converge on deterministic pre-execution gates over
declared constraints and current state: SMT-LIB policies checked by Z3
([arXiv:2603.20449](https://arxiv.org/abs/2603.20449)); "Reason Less, Verify More"
(+12.4 pp on τ²-bench airline; 78% of failures silent wrong-state,
[arXiv:2607.07405](https://arxiv.org/abs/2607.07405)); AgentLTL
([arXiv:2607.02599](https://arxiv.org/abs/2607.02599)); ActPlane, in-kernel eBPF because
"tool-call guardrails cannot intercept system actions that bypass the tool layer"
([arXiv:2606.25189](https://arxiv.org/abs/2606.25189)); "Prompts Don't Protect," ABAC at
an MCP proxy so unauthorized tools never reach the model
([arXiv:2605.18414](https://arxiv.org/abs/2605.18414)). Ontology-grounded project memory for coding agents (MOOSEDev, NeSy 2026 industry
track) — decisions, constraints and rationales as a KG over MCP with "lifecycle status,
provenance, and supersession links" — answered supersession, set-completeness and
negation questions at 0.98–1.00 where a production vector-memory tool surfaced 6–27%
([arXiv:2608.13662](https://arxiv.org/abs/2608.13662)). Contract2Tool *infers* tool
contracts (`preconditions, effects, risk level, cost`) from schemas, docs and traces,
reaching 0.980 vs 0.990 gold downstream success
([arXiv:2606.07904](https://arxiv.org/abs/2606.07904)). Incidents: PocketOS
(2026-04-25) — Opus 4.6 via Cursor issued a Railway `volumeDelete(...)` with a
blanket-scope token, destroying production data and its volume backups; the report
states neither "Destructive Guardrails" nor Plan Mode intervened and that "Guardrails
must be implemented at the API gateway and token-permission level, not in advisory
text" ([AIID 7311](https://incidentdatabase.ai/reports/7311)); a Replit-freeze
recreation enforces the constraint "below the prompt" at the tool-call boundary
([agenticcontrolplane.com](https://agenticcontrolplane.com/blog/recreated-replit-database-deletion));
a nine-incident compilation attributes most to constraints delivered as prompt text or
shell-expansion gaps after approval ([adversa.ai](https://adversa.ai/blog/ai-coding-agent-incidents/);
per-row primary sources not individually opened → [unverified]).

**[inference]** Confirms the capability fragment strongly, with two corrections to its
current framing: learned contracts (Contract2Tool) should be positioned as perception-
side *proposals* into a verified fragment, and effect constraints must cover the
credentials and endpoints a tool can reach, not only file paths — several incidents are
out-of-band network effects no path validator models (the sandbox stays necessary, as
`docs/framework/15-security.md` already assumes).

## Searched but not found

**[measured — negative results of this sweep's queries, 2026-09-02]**

- Any core harness whose edit/patch application checks a base content hash or version.
- Any manifest field declaring typed preconditions or effects (nearest: `paths`,
  `compatibility`, MCP hints, draft SEP-1862).
- Any harness maintaining derived facts or a fixed point over repository state.
- A sandbox-*pass*-then-symbolic-authorize pipeline; what exists is sandbox-fail-then-ask
  (Codex), sandbox-then-classifier (Cursor), or sandbox-then-human-PR-review (Copilot
  cloud agent).
- Published quantitative data on how often `auto`/`auto_review`/Auto-review approve
  destructive actions.

## Source

Per-claim citations inline; all rows in `../reading-log.md` § "Q7 — Coding-agent
harness capability/policy structure (sweep #2)".

## Confidence

mixed — harness documentation, changelog entries, issue states and paper findings are
[measured] (items marked [unverified] rest on secondary write-ups or snippets); the
fit-to-design conclusions are [inference].

## Last-verified

2026-09-02
