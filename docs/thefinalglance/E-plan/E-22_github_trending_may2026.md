# E-22: SKILL.md export + microsandbox boot + deferred MCP loading (May 2026 trending steals)
Objective: Export ELAI's `.claude/skills/` as SKILL.md-compliant manifests for cross-vendor portability, replace any docker/process-based code-execution sandbox with microsandbox/libkrun microVM boot (<200 ms), add deferred MCP tool schema loading per Anthropic's Tool-Search pattern, and benchmark `elai.map` against `zilliztech/claude-context` so the user can quantify the moat.
Estimated effort: 3 sessions, ~10-12 h
Pre-requisites: B-18 PLANNED (Tier C Sandbox Activation), C-11 PLANNED (Sandbox Completeness), C-35 PLANNED (Tool Attention Dynamic Gating + Lazy MCP Schema Loading), B-37 SHIPPED (PerRoleRuntime)
Parallel-safe with: E-21 (peer-systems steals — different surface), E-23 (positioning — copy not code), E-24 (MCP harden — depends on E-22 deferred loading), E-25 (compaction — different ladder)
Source report: workflow/improvement-sweep-2026-05-19/reports/22-github-trending-may2026.md

## Context

The May 2026 trending snapshot identifies four converged patterns ELAI must adopt to keep its window open. (1) SKILL.md as cross-vendor open standard: Anthropic spec Dec 2025, OpenAI Codex adopted developers.openai.com/codex/skills, five of top-20 fastest-growing repos are "skills" repos (addyosmani/agent-skills 43.3k +26k/mo, mattpocock/skills 72k, multica-ai/andrej-karpathy-skills 80k+). ELAI's `.claude/skills/` is already on this pattern but does not export manifests. (2) MicroVM sandboxing: superradcompany/microsandbox (YC X26, libkrun-based, sub-200ms boot, hardware isolation) is the new default; Docker boots >1s and shares the kernel; northflank.com/blog/how-to-sandbox-ai-agents documents the practical case. (3) MCP Streamable HTTP replaces SSE since Anthropic spec 2025-11-25 (modelcontextprotocol.io/specification/2025-11-25/basic/transports); deferred tool-description loading saves 15-80k tokens per turn (mindstudio.ai/blog/claude-code-mcp-server-token-overhead). (4) Code-context MCP servers proliferate (zilliztech/claude-context 11.4k MCP, Understand-Anything 15k) — the report's Open Question 4 asks whether elai.map is redundant; only a benchmark answers.

The report also surfaces three competitive threats: Archon (21.6k, MIT, deterministic YAML workflow + AI nodes), ruflo (52.8k, multi-agent swarm + federation + AgentDB+HNSW), and Microsoft Conductor (May 14 2026 announcement, deterministic orchestration for multi-agent AI workflows). None ship Rust-native FSM + grounded reasoning — that combination is still ELAI's window — but the SKILL.md export is the cheapest way to plant a flag in the cross-vendor skill marketplaces (agentskill.sh 110k+, Vercel skills.sh 87k skills, Agensi marketplace per kdnuggets top-5).

Anti-patterns explicitly flagged: hype-trained skill repos with no validation (Imbad0202/academic-research-skills 11.6k = README copy-paste), self-evolving claims without measurable benchmarks (NousResearch/hermes-agent 156k stars, no SWE-bench number), Docker for code-exec, agent sprawl without composition (ruflo 100+ agents). ELAI must steal the pattern (SKILL.md, microVM) without inheriting the anti-pattern (no quality gate).

## Existing state (B/C/D coverage)

- B-18 PLANNED (Tier C Sandbox Activation): per B-PLAN-SUMMARY line 331-340, Windows default `enable_sandbox=true` at dashboard/src/lib.rs:1183, task_runner.rs:2477, agent_pool.rs:153; wire self.sandbox.launch_agent(). REPLACES the planned Tier C implementation with microsandbox/libkrun-backed launch — the spec stayed implementation-agnostic, microsandbox satisfies the contract.
- B-19 PLANNED (Hook System Unification) and B-20 PLANNED (Tool Registry Unification): per B-PLAN-SUMMARY line 349-367, sequential after B-18. EXTENSION: E-22 deferred MCP schema loading wires into the same IpcToolRegistry that B-20 stabilises; no replacement of B-19/B-20, just a flag (`lazy_schemas=true`) on the registry.
- C-11 PLANNED (Sandbox Completeness — Tier C Isolation): per C-SUMMARY_INDEX line 137. REPLACES the abstract "Tier C isolation" with concrete microsandbox/libkrun on Linux; document Windows fallback (Hyper-V isolated container or WSL2 with microsandbox) explicitly because microsandbox is Linux-only per github.com/superradcompany/microsandbox README — verify via WebSearch.
- C-35 PLANNED (Tool Attention Dynamic Gating + Lazy MCP Schema Loading): per C-SUMMARY_INDEX line 598-601, cites arXiv:2604.21816 with 95% measured MCP-tax reduction. EXTENSION: E-22 adds the SKILL.md export surface and the elai.map vs claude-context benchmark; the lazy-schema mechanism itself is C-35's responsibility — E-22 only wires the consumer-side feature flag.
- C-39 PLANNED (MCP Bridge CLI Surface Extension): adds MCPClient + `--via-mcp` CLI flag. EXTENSION: E-22 verifies the MCP bridge speaks Streamable HTTP (Mar 2025 spec deprecation of SSE) and adds a regression test.
- B-37 SHIPPED: PerRoleRuntime + 11 backends. E-22 does not modify runtime layer; sandbox is upstream of runtime selection.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\22-github-trending-may2026.md` (full)
- `F:\.ELAI_workflow\.claude\skills\` (each SKILL.md — directory listing only via Glob; do not full-read all skills in main LLM context per anti-read-big)
- `F:\.ELAI_workflow\.elai_cc\crates\plugins\src\mcp_bridge.rs` (existing MCP bridge — Streamable HTTP support audit)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\lib.rs` line 1183 area (B-18 sandbox attach point)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs` line 2477 area (sandbox launch site)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\agent_pool.rs` line 153 area (sandbox launch site)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 290-470, 569-720 (B-15..B-23 harness + B-29..B-35 knowledge engine context)
- `F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-SUMMARY_INDEX.md` lines 130-180, 595-635 (C-11 + C-35 + C-39 detail)
- Anthropic SKILL.md spec — code.claude.com/docs/en/skills (WebFetch)
- microsandbox README — github.com/superradcompany/microsandbox (WebFetch; verify license, Windows support, version)
- zilliztech/claude-context README — github.com/zilliztech/claude-context (WebFetch; benchmark target)
- modelcontextprotocol.io/specification/2025-11-25/basic/transports (Streamable HTTP spec)

## Tasks

1. SKILL.md export shim. New module `crates/skills/src/export.rs` (verify the `skills` crate exists; if not, the existing skill loader lives in `crates/tools/src/lib.rs` per the elai.map hits at lines 3330, 6190 — extend that instead, do NOT create a new crate without user confirmation per HARD RULE). Walk `.claude/skills/` and for each subdirectory containing `SKILL.md`, normalize the YAML frontmatter to the Anthropic public spec (name, description, version, allowed_tools optional, model optional). Write the normalized form to `dist/skills/{name}/SKILL.md` for cross-vendor consumption. Idempotent. Detect drift between source and dist via blake3 content-hash; emit `skill_export_drift` SSB row if dist is stale. Reference: code.claude.com/docs/en/skills, developers.openai.com/codex/skills.

2. Microsandbox / libkrun integration. Add a `Sandbox` enum to `crates/core/src/sandbox.rs` (NEW module) with variants `None`, `Process`, `MicroVm`. On Linux, the `MicroVm` variant shells out to the microsandbox CLI (verify the CLI exists and is on PATH at startup; failure mode = degrade to `Process` with a SSB warning `sandbox_microvm_unavailable`). On Windows, `MicroVm` falls back to `Process` with a one-line `tracing::warn!` because libkrun is Linux-only per the project README (verify via WebSearch and add a `# Anti-stale-training` comment with verification date). The sandbox launch sites at `dashboard/src/lib.rs:1183`, `task_runner.rs:2477`, `agent_pool.rs:153` already exist per B-18 spec — wire them to call `Sandbox::launch(cmd, args, opts)`. Anti-slopsquatting: confirm the crate name and binary name match the public README before adding any dep.

3. Deferred MCP tool schema loading (consumer of C-35's mechanism). Add a feature flag `lazy_schemas` to `crates/plugins/src/mcp_bridge.rs::McpBridge`. When enabled, on `tools/list` the bridge returns only `{name, description}` pairs (one-line descriptions of ≤140 chars). Full input_schema fetched on demand via a new `tools/get_schema` JSON-RPC method we add (compatible with MCP Tool-Search per anthropic.com/engineering/code-execution-with-mcp). Measure token savings: before and after, on a `local-rag + pod-lifecycle` enumeration, log `mcp_tools_list_bytes_pre / mcp_tools_list_bytes_post`. Emit SSB `mcp_lazy_schema_savings`. The C-35 phase ships the same mechanism — coordinate with the user to avoid double work; if C-35 is in flight, E-22 only adds the regression test and the SSB row.

4. Streamable HTTP transport for MCP bridge. Per modelcontextprotocol.io/specification/2025-11-25/basic/transports, SSE is deprecated since Mar 2025. Audit `crates/plugins/src/mcp_bridge.rs` for the transport implementation; if SSE-only, add a `transport: streamable_http` config option that uses a single POST endpoint with optional Server-Sent-Events RESPONSE (one direction only — request body is JSON-RPC over POST). Emit `mcp_transport_negotiated` SSB row. Reference: blog.fka.dev "Why MCP Deprecated SSE And Go With Streamable HTTP" Jun 2025.

5. elai.map vs claude-context benchmark. Run a like-for-like comparison on the .elai_cc tree itself: (a) elai.map symbol-lookup latency (`grep <symbol> .claude/tools/elai.map`) and token-cost-to-resolve-context, vs (b) zilliztech/claude-context MCP server's same queries. Output `workflow/audits/elai-map-vs-claude-context-2026-05-19.md` with per-query results and a verdict. Spawn the claude-context MCP server in a temporary microVM (Task 2) to keep the comparison clean. Reference: github.com/zilliztech/claude-context. This is a benchmark, not a refactor — the verdict can be "keep both" or "deprecate one" but the answer must be data-grounded.

6. 12-factor agents audit. Cross-reference humanlayer/12-factor-agents (20.5k stars, github.com/humanlayer/12-factor-agents) against ELAI's harness. Output `workflow/audits/12-factor-audit-2026-05-19.md` with one row per factor and a PASS/PARTIAL/FAIL grade. Soft enforcement — no fixer queue; the audit is a positioning artifact for the user to decide which factors to harden.

7. SKILL.md publish path. Add `elai skills publish [--target claude|codex|cursor|vercel|agensi]` CLI subcommand that takes the Task 1 export output and packages it for the target marketplace's intake format (deferred manifest spec for each — Claude Code is identity-mapping the SKILL.md; Codex/Cursor/Vercel formats need WebFetch verification before committing format). Default target is `claude` because the spec is identity. Other targets emit a clear `not_yet_supported` error with a link to the marketplace docs.

## WIRING_CONTRACT

produces_symbols:
  - skills::export::export_all(src: &Path, dst: &Path) -> Result<ExportReport>
  - skills::export::ExportReport { exported: usize, drift: Vec<String> }
  - core::sandbox::Sandbox { None, Process, MicroVm }
  - core::sandbox::Sandbox::launch(cmd, args, opts) -> Result<Child>
  - plugins::mcp_bridge::McpBridge::lazy_schemas (config flag)
  - plugins::mcp_bridge::tools/get_schema (new JSON-RPC method)
  - cli::skills::publish_target (new subcommand)
consumes_symbols:
  - crates/dashboard-v2/src-tauri/src/lib.rs:1183 (Sandbox::launch attach)
  - crates/orchestrator/src/task_runner.rs:2477 (Sandbox::launch attach)
  - crates/orchestrator/src/agent_pool.rs:153 (Sandbox::launch attach)
  - .claude/skills/*/SKILL.md (read for export)
  - crates/plugins/src/mcp_bridge.rs::list_tools (lazy_schemas branch)
reachable_from:
  - crates/cli/src/main.rs::main (new `skills export`, `skills publish`, `bench claude-context` subcommands)
  - crates/orchestrator/src/task_runner.rs::drive_task (sandbox launches via core::sandbox)
  - crates/plugins/src/mcp_bridge.rs::McpBridge::handle_list_tools (lazy_schemas branch)
stub_ban:
  - no Docker-shelling-out fallback in sandbox.rs (microsandbox-only on Linux, process on Windows)
  - no full schema returned from tools/list when lazy_schemas=true (verify in test)
  - no SSE-only path remaining in mcp_bridge if Task 4 lands
  - no .unwrap() in sandbox.rs (this is production code; use ? / map_err)
frontend_pair: crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (skill-publish status; or new SkillsPanel.svelte under B-26 dashboard completion umbrella)
observable_effect:
  trigger: "elai skills export --src .claude/skills --dst dist/skills" on a clean tree
  expect_ssb: "skill_export_completed" with exported>0, drift=[]
  expect_artifact: dist/skills/{name}/SKILL.md exists for each .claude/skills/{name}/SKILL.md, with normalized YAML frontmatter and identical body content

## DETERMINISTIC_CHECK

```bash
# 1. SKILL.md export module exists and respects spec
rg "fn export_all" .elai_cc/crates --type rust
# Expected: 1 hit

# 2. Sandbox enum + launch surface
rg "pub enum Sandbox" .elai_cc/crates/core --type rust
rg "Sandbox::launch" .elai_cc/crates --type rust
# Expected: 1 enum def + ≥3 call sites (dashboard/task_runner/agent_pool)

# 3. lazy_schemas branch present
rg "lazy_schemas" .elai_cc/crates/plugins/src/mcp_bridge.rs
# Expected: ≥2 hits (field decl + branch in handle_list_tools)

# 4. Streamable HTTP transport audited (file exists OR test confirms presence)
rg "streamable_http|StreamableHttp" .elai_cc/crates/plugins/src/mcp_bridge.rs
# Expected: ≥1 hit (if absent, contract test in tests/contracts/mcp_streamable_http.rs must explicitly assert SSE remains, with a TODO[E-22-followup] reason)

# 5. Audit artifacts present
test -f workflow/audits/elai-map-vs-claude-context-2026-05-19.md
test -f workflow/audits/12-factor-audit-2026-05-19.md
# Expected: both files exist

# 6. Smoke: microVM boot time under 500ms on Linux (CI gate, skip on Windows)
.elai_cc/target/release/elai-sandbox-bench --target microvm --iterations 5
# Expected on Linux: p50 boot <500ms, p95 <1000ms (microsandbox README claims <200ms; loosen acceptance to 500ms for first-pass)
```

## ACCEPTANCE

- [ ] `dist/skills/` contains every `.claude/skills/{name}/SKILL.md` with normalized frontmatter; blake3 hash check passes
- [ ] `Sandbox::MicroVm::launch` succeeds on Linux with a "hello world" payload; falls back to `Process` on Windows with `sandbox_microvm_unavailable` SSB row
- [ ] `lazy_schemas=true` reduces `tools/list` response size by ≥80% on the local-rag + pod-lifecycle combined catalog; SSB row records absolute before/after byte counts
- [ ] Streamable HTTP transport supported by `mcp_bridge` OR an explicit contract test documents the SSE-remaining state with a follow-up TODO referencing E-22
- [ ] elai.map vs claude-context benchmark artifact exists with ≥10 query rows and a written verdict (keep both / deprecate / hybrid)
- [ ] 12-factor audit artifact exists with one row per factor and a grade
- [ ] `elai skills publish --target claude` succeeds round-trip on the export output
- [ ] No new dependencies added without user confirmation (microsandbox is shelled-out, not a Rust crate dep; verify)

## Tests to add

- `crates/skills/tests/export_round_trip.rs`: export then re-import preserves body byte-for-byte
- `crates/core/tests/sandbox_linux_microvm.rs`: gated `#[cfg(target_os = "linux")]`, microsandbox bin present check
- `crates/core/tests/sandbox_windows_fallback.rs`: gated `#[cfg(windows)]`, asserts `Sandbox::MicroVm::launch` returns `Sandbox::Process` payload + warning
- `crates/plugins/tests/mcp_lazy_schemas.rs`: `tools/list` with lazy_schemas=true returns no `input_schema` field; `tools/get_schema` returns it on demand
- `crates/plugins/tests/mcp_streamable_http_smoke.rs`: round-trip a JSON-RPC request via Streamable HTTP
- `crates/cli/tests/skills_publish_dry_run.rs`: `--target claude --dry-run` lists files to publish
- `tests/contracts/mcp_transport_capability.rs`: regression test for transport negotiation

## Files touched

- CREATE: `crates/skills/src/export.rs` OR `crates/tools/src/skills_export.rs` (~200 LOC) — pick based on existing crate layout per Reads task
- CREATE: `crates/core/src/sandbox.rs` (~250 LOC including Windows fallback path)
- CREATE: `crates/cli/src/skills_cmd.rs` (~150 LOC) for `skills export|publish`
- CREATE: `workflow/audits/elai-map-vs-claude-context-2026-05-19.md`
- CREATE: `workflow/audits/12-factor-audit-2026-05-19.md`
- MODIFY: `crates/plugins/src/mcp_bridge.rs` (+~80 LOC for lazy_schemas + tools/get_schema + Streamable HTTP audit)
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs:1183` (Sandbox attach)
- MODIFY: `crates/orchestrator/src/task_runner.rs:2477` (Sandbox attach)
- MODIFY: `crates/orchestrator/src/agent_pool.rs:153` (Sandbox attach)
- MODIFY: `crates/cli/src/main.rs` (+~25 LOC for new subcommands)
- MODIFY: `crates/core/src/lib.rs` (+1 LOC mod declaration)
- CREATE: `workflow/reasoning/E-22-decisions.md`
- CREATE: `dist/skills/` (output directory, .gitignored)

## Citations / Grounding

- code.claude.com/docs/en/skills (Anthropic SKILL.md spec) — grounds Task 1 format
- developers.openai.com/codex/skills (OpenAI Codex adopts SKILL.md) — grounds Task 7 cross-vendor target
- github.com/superradcompany/microsandbox (libkrun microVM, YC X26) — grounds Task 2; CONFIRM Windows support absent and license via WebFetch before any dep
- northflank.com/blog/how-to-sandbox-ai-agents — practical case for microVM > Docker — grounds Task 2 rationale
- modelcontextprotocol.io/specification/2025-11-25/basic/transports — Streamable HTTP — grounds Task 4
- mindstudio.ai/blog/claude-code-mcp-server-token-overhead (55,000-token GitHub MCP overhead) — grounds Task 3
- anthropic.com/engineering/code-execution-with-mcp (Tool Search + code-execution pattern) — grounds Task 3 mechanism
- github.com/zilliztech/claude-context (11.4k MCP server, code-context competitor) — grounds Task 5 benchmark target
- github.com/humanlayer/12-factor-agents (20.5k stars) — grounds Task 6 reference list
- arXiv:2604.21816 (Tool Attention, 95% MCP-tax reduction) — cited by C-35; grounds the lazy-schema effect-size claim — VERIFY arXiv ID per memory feedback_paper_citation_verify (Cite was provided in C-plan; if 2604 prefix invalid, mark Ungrounded and substitute)

## Open questions / risks

- microsandbox is Linux-only per project README; user runs Windows 11 per env block. The Windows path falls back to `Sandbox::Process`. Open question: should the Linux microsandbox path be gated to remote-sidecar host (which is Ubuntu 24.04 WSL2 per CLAUDE.md remote-sidecar block), running sandboxed agent calls only on the remote? This trades local latency for hardware isolation.
- arXiv:2604.21816 citation needs WebSearch verification per memory feedback_paper_citation_verify (2 bad cites already in B-40/B-42 plans). If the ID is invalid, the Tool Attention claim becomes Ungrounded and Task 3 must justify on token-cost data alone (mindstudio + onlycli benchmarks are sufficient).
- SKILL.md frontmatter normalization: Anthropic spec evolved Dec 2025; Codex adopted but may have format drift; Cursor/Vercel/Agensi formats unconfirmed. Task 7 publishing to non-Claude targets is INGEST-classified per grounding sweep — block on WebFetch confirmation before adding any target.
- elai.map vs claude-context benchmark: requires running the claude-context MCP server which is TypeScript + Bun + vector DB. The microVM (Task 2) is a clean way to host this; if microsandbox is unavailable on the dev host the benchmark stalls. Open question: is benchmarking on the remote Linux sidecar acceptable?
- `dist/skills/` path: confirm with the user whether this is gitignored at workflow root (likely) and whether it should be checked-in inside .elai_cc/ (likely not — output artifact).

## Next

E-24 consumes Task 3's deferred-schema mechanism for its zero-trust MCP harden (C-42 territory) — coordinate the lazy_schemas flag semantics. E-25 consumes the SKILL.md export as a candidate transport for "skills as portable state across compaction" — if E-25's compaction ladder serializes skill state, it should consume the dist/skills/ format. After E-22, B-18 (Tier C Sandbox Activation) can ship by wiring the existing Tier C harness to `Sandbox::MicroVm::launch` rather than building its own sandbox primitive — this collapses two PLANNED B-N specs into one shipped foundation.
