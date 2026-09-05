# E-24: MCP bridge hardening (schema-hash pinning, list-time sanitization, Streamable HTTP, no gateway)
Objective: Harden the existing MCP bridge per OWASP MCP03:2025 + Trail of Bits "line jumping" + ReversingLabs rug-pull findings. Add tool-schema hash pinning, on-list description sanitization, per-tool token-budget enforcement, and migrate transport from SSE to Streamable HTTP. Explicitly REJECT any LLM gateway (LiteLLM / OpenRouter / Portkey) in front of Anthropic.
Estimated effort: 2-3 sessions, ~8-10 h
Pre-requisites: B-37 SHIPPED (PerRoleRuntime, Anthropic /v1/messages sidecar surface), C-35 PLANNED (Tool Attention + Lazy Schema), C-39 PLANNED (MCP Bridge CLI Surface), C-42 PLANNED (Semantic Gateway Zero-Trust MCP Fuzzing), B-19 PLANNED (Hook System Unification), B-20 PLANNED (Tool Registry Unification)
Parallel-safe with: E-21 (different surface), E-22 (E-22 ships lazy_schemas → E-24 consumes it), E-23 (positioning), E-25 (compaction)
Source report: workflow/improvement-sweep-2026-05-19/reports/24-agent-protocols-mcp.md

## Context

The report's verdict is precise: ELAI's existing MCP bridge (HTTP JSON-RPC, deny-list + sanitization, dashboard-consumed) is already on the right track — Option B "MCP for external tools only" is the recommended path. But four hardening gaps remain: (1) tool-schema hash pinning per OWASP MCP03:2025 to prevent rug-pulls like postmark-mcp Sept 17 2025 where v1.0.16 silently added a 1-line BCC backdoor (thehackernews.com/2025/09/first-malicious-mcp-server-found.html, 1,643 downloads, ~300 orgs compromised); (2) tool-description sanitization on EVERY list response per Trail of Bits "line jumping" PoC Apr 21 2025 (blog.trailofbits.com/2025/04/21/jumping-the-line-how-mcp-servers-can-attack-you-before-you-ever-use-them/) — descriptions ARE prompt-injection vectors that exfiltrate chat history and plant `chmod -R 0666 ~;` prefixes; (3) per-tool token-budget enforcement to defuse the 55,000-token GitHub MCP overhead (mindstudio.ai/blog/claude-code-mcp-server-token-overhead) — 5 servers × 20 tools ≈ 80k tokens before any work; (4) transport migration from SSE to Streamable HTTP since Anthropic spec 2025-11-25 deprecated SSE (modelcontextprotocol.io/specification/2025-11-25/basic/transports) and 28% of registered servers already speak Streamable HTTP (zylos.ai/research/2026-03-08).

The report also rules out two design temptations. (a) Full MCP adoption for internal IPC: REJECT — postcard at 60ns serialize / 180ns deserialize / ~70% bincode size is unbeatable for the orchestrator-agent hot path (users.rust-lang.org/t/is-it-better-to-use-bincode-or-postcard/88740). The 92% exploit probability across enterprise MCP stacks (venturebeat.com/security/mcp-stacks-have-a-92-exploit-probability) is reason enough to keep the internal trust boundary outside the MCP surface. (b) LLM gateway in front of Anthropic: REJECT — LiteLLM adds 10-20ms + breaks past 300-500 RPS, OpenRouter +<50ms, all break Anthropic's prompt-cache layer (decodethefuture.org/en/anthropic-blocks-third-party-tools/ April 2026 policy). ELAI's local-sidecar Anthropic-protocol choice is strategically correct for 2026.

MCP also lacks agent semantics (Scola, medium.com/@thomas.scola/mcp-is-not-enough-the-missing-gaps-in-open-agent-standards-3bc31e7b4e59): no identity, no manifests, no federated discovery, no policy-as-primitive. The Nov 2025 spec added Tasks (async/cancel) and Sampling-with-Tools (SEP-1577) but ELAI's FSM + AgentPool already do what Tasks does. The report's Open Question 3 asks whether A2A is worth implementing — answer for E-24: no, because ELAI has no consumer that speaks A2A (S&P Global, ServiceNow Fabric do; ELAI's user base — none confirmed). Defer A2A to a future E-N if any external A2A consumer materialises.

## Existing state (B/C/D coverage)

- B-37 SHIPPED: PerRoleRuntime + Anthropic-protocol sidecar surface (commit fd5c2e5). NO CHANGE — E-24 explicitly REJECTS adding a gateway layer in front.
- C-35 PLANNED (Tool Attention Dynamic Gating + Lazy MCP Schema Loading): per C-SUMMARY_INDEX line 598. EXTENSION: E-24 consumes C-35's lazy-schema mechanism (also shipped by E-22 Task 3) and adds the per-tool token-budget enforcement layer on top.
- C-39 PLANNED (MCP Bridge CLI Surface Extension — OpenHands Parity): adds MCPClient + `--via-mcp` CLI flag + Dispatch trait. EXTENSION: E-24 hardens the same `crates/plugins/src/mcp_bridge.rs` but on the bridge SERVER side (sanitization + hash pin) and on the CLIENT side (Streamable HTTP).
- C-42 PLANNED (Semantic Gateway Zero-Trust MCP Fuzzing): per C-SUMMARY_INDEX line 647. EXTENSION: E-24 lands the OWASP MCP03:2025 schema-hash + sanitization layer FIRST so C-42's fuzz corpus can exercise it. C-42 then becomes a regression suite for E-24's surface.
- B-19 PLANNED (Hook System Unification) and B-20 PLANNED (Tool Registry Unification): per B-PLAN-SUMMARY lines 349-367. DELTA: E-24 modifies the same tool registry surface; coordinate ordering — B-20 should land before E-24 OR E-24's modifications must be additive (new methods, no breaks). Recommend additive approach to allow parallel execution.
- B-3 SHIPPED (Stub Command Eradication): per B-PLAN-SUMMARY line 126 — MCP bridge contract test `crates/contracts/tests/fuld_mcp_bridge_contract.rs` already exists per elai.map lines 208-247. E-24 ADDS contract tests, does not modify existing ones.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\24-agent-protocols-mcp.md` (full)
- `F:\.ELAI_workflow\.elai_cc\crates\plugins\src\mcp_bridge.rs` (existing bridge; per elai.map lines 1190-1196 — verify current sanitization + deny-list)
- `F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\fuld_mcp_bridge_contract.rs` (existing contract — extend, do not replace)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 290-470 (B-15..B-23 harness context, B-19/B-20 in particular)
- `F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-SUMMARY_INDEX.md` lines 595-700 (C-35..C-42 detail)
- OWASP MCP Top 10 2025 #3 (owasp.org/www-project-mcp-top-10/2025/MCP03-2025-Tool-Poisoning) — WebFetch
- Trail of Bits "Jumping the Line" Apr 21 2025 (blog.trailofbits.com) — WebFetch
- MCP spec 2025-11-25 transports section (modelcontextprotocol.io/specification/2025-11-25/basic/transports) — WebFetch
- thehackernews.com/2025/09/first-malicious-mcp-server-found.html (postmark-mcp postmortem) — WebFetch

## Tasks

1. Tool-schema hash pinning. New module `crates/plugins/src/mcp_schema_pin.rs` (~250 LOC). On first connection to an MCP server, fetch `tools/list`, compute `blake3` (NOT blake2 / sha2 per CLAUDE.md anti-stale-training) over the canonicalized JSON-serialized full tool list (sorted by tool name; serde_json with consistent map serialization). Store the pin in `.elai_cc/.elai/mcp/pins/{server_url}.toml` with `pinned_hash`, `pinned_at`, `pinned_version`. On subsequent connections, recompute and compare. On mismatch: BLOCK by default with `mcp_schema_pin_violation` SSB row; require explicit `elai mcp pin update <server>` CLI to re-pin. Surfaces postmark-mcp class of rug-pulls. Reference OWASP MCP03:2025.

2. Tool-description sanitization on EVERY list. The bridge already has deny-list + sanitization per elai.map line 1193 (`McpError::SanitizationFailed`). Audit current sanitization rules — what is stripped? Extend to cover the Trail of Bits "line jumping" patterns: (a) markdown-code-fenced commands inside descriptions; (b) `<system>` / `<assistant>` / `<user>` role tags inside descriptions; (c) prompt-leak strings like "ignore previous instructions", "you are now"; (d) shell command prefixes like `chmod`, `rm`, `curl`, `wget`, `eval`, `bash -c`; (e) URL exfil patterns like `?data=` query params on external domains. Each rule emits a `mcp_sanitization_rule_applied` SSB row with `rule_id` + `replacement_count`. Reference: Trail of Bits Apr 21 2025.

3. Per-tool token-budget enforcement. New `crates/plugins/src/mcp_budget.rs` (~180 LOC). Configurable budget per server (default 5,000 tokens for `tools/list` response, configurable in `elai_cc.toml` under `[mcp.budgets]`). On `tools/list` response, estimate token cost via tiktoken-rs (verify the crate name and current version via WebSearch; anti-slopsquatting) OR a simple chars/4 heuristic if no tokenizer dep is approved. When over budget: drop tools by lowest priority first (priority = explicit allowlist position; default tail of allowlist drops first). Emit `mcp_budget_enforced` SSB row with `dropped_tools: [...]`. Composes with E-22 Task 3 lazy_schemas: if lazy_schemas reduces base response below budget, no drops needed.

4. Streamable HTTP migration. Audit `mcp_bridge.rs` transport — verify per E-22 Task 4 audit (or run the same audit if E-22 hasn't shipped). Implement Streamable HTTP transport: single POST endpoint for client→server requests, optional Server-Sent-Events for server→client streaming responses. Reference: modelcontextprotocol.io/specification/2025-11-25/basic/transports. SSE-only mode kept behind a `transport: sse_legacy` config flag for backward compatibility with non-migrated MCP servers, with a startup warning `mcp_legacy_sse_transport` SSB row.

5. Per-tool input-validation hook (PreToolUse on MCP calls). Add an LLM-as-decision hook handler type per Claude Code's `prompt` hook (code.claude.com/docs/en/hooks). The hook receives the tool name + sanitized parameters and prompts a fast model (default the local Qwen 3B sidecar — uses existing ANTHROPIC_BASE_URL) with: "Is this MCP call safe to execute? Output JSON {decision: allow/deny/ask, reason: ...}." On `deny`, the call is blocked + `mcp_llm_guard_blocked` SSB row. On `ask`, the call requires user confirmation via dashboard. Reference E-25 for related `prompt`-hook infrastructure — coordinate so the LLM-decision hook is a shared primitive across E-24 / E-25.

6. Explicit gateway-rejection guard. New startup check in `crates/cli/src/main.rs` (or wherever `ANTHROPIC_BASE_URL` is read) that verifies the URL is one of: the local sidecar (`http://localhost:8100` or `http://127.0.0.1:8100`), `https://api.anthropic.com`, or an explicitly-confirmed dev override. Any other URL — particularly LiteLLM / OpenRouter / Portkey hostnames — emits a startup warning `anthropic_gateway_detected` SSB row referencing decodethefuture.org/en/anthropic-blocks-third-party-tools/ and the 10-50ms latency + prompt-cache-break cost. Hard-block only if the user explicitly opts in via `--strict-no-gateway` flag.

7. Contract tests for the four hardening surfaces. Extend `crates/contracts/tests/fuld_mcp_bridge_contract.rs` (existing) and create `crates/contracts/tests/mcp_schema_pin_contract.rs` + `crates/contracts/tests/mcp_sanitization_line_jumping_contract.rs` + `crates/contracts/tests/mcp_budget_enforcement_contract.rs` + `crates/contracts/tests/mcp_transport_streamable_http_contract.rs`. Each test exercises one of Tasks 1-4 against a synthetic MCP server fixture and asserts the SSB row emission + behavior. The line-jumping fixture must include each of the 5 Trail-of-Bits patterns and assert each is stripped.

## WIRING_CONTRACT

produces_symbols:
  - plugins::mcp_schema_pin::pin_or_verify(server_url: &str, tools: &[ToolDef]) -> Result<PinStatus>
  - plugins::mcp_schema_pin::PinStatus { Pinned, Verified, Violation }
  - plugins::mcp_bridge::sanitize_description(desc: &str, rules: &SanitizeRules) -> SanitizeResult
  - plugins::mcp_bridge::SanitizeResult { sanitized: String, rules_applied: Vec<RuleId>, replacement_count: usize }
  - plugins::mcp_budget::enforce(response: &mut ToolsListResponse, budget: TokenBudget) -> EnforceReport
  - plugins::mcp_budget::EnforceReport { dropped: Vec<String>, kept_tokens: usize }
  - hooks::llm_guard::pre_mcp_call_decision(tool: &str, params: &Value) -> GuardDecision
  - hooks::llm_guard::GuardDecision { Allow, Deny(reason), Ask(reason) }
  - cli::mcp_pin::pin_subcommand (new `elai mcp pin <server>` CLI surface)
consumes_symbols:
  - crates/plugins/src/mcp_bridge.rs::McpBridge (existing — extended, not replaced)
  - crates/contracts/tests/fuld_mcp_bridge_contract.rs (existing — extended)
  - crates/orchestrator/src/runtime_orchestrator.rs::PerRoleRuntime (LLM-guard uses sidecar)
  - blake3 crate (already in deps per CLAUDE.md anti-stale-training rule)
  - .elai_cc/.elai/mcp/pins/{server_url}.toml (pin persistence)
reachable_from:
  - crates/cli/src/main.rs (gateway-rejection check at startup; `elai mcp pin` subcommand)
  - crates/plugins/src/mcp_bridge.rs::handle_list_tools (sanitization + budget enforcement)
  - crates/plugins/src/mcp_bridge.rs::call_tool (LLM-guard hook)
  - crates/plugins/src/mcp_bridge.rs::initialize (schema-pin verification)
stub_ban:
  - no blake2 / sha2 for the pin hash (HARD RULE: blake3 only)
  - no LLM gateway URL accepted at startup without explicit override flag
  - no SSE-only transport without `transport: sse_legacy` opt-in
  - no full-schema returned from `tools/list` when budget exceeded
  - no .unwrap() in production sanitization paths
  - no manual JSON parsing of MCP responses (use serde_json with strict schemas)
frontend_pair: crates/dashboard-v2/src/lib/components/McpStatusPanel.svelte (new — surfaces pin status + budget enforcement + last-LLM-guard decision; alternatively extend existing dashboard MCP panel if present)
observable_effect:
  trigger: connect to a fixture MCP server (postmark-mcp-style schema with sanitization-bait + budget overflow), THEN connect a second time with mutated schema
  expect_ssb: first connect emits "mcp_schema_pinned" + "mcp_sanitization_rule_applied" rows; second connect emits "mcp_schema_pin_violation" and BLOCKS the call
  expect_artifact: ".elai_cc/.elai/mcp/pins/{server_url}.toml" exists after first connect; second connect's call_tool returns McpError::SchemaPinViolation

## DETERMINISTIC_CHECK

```bash
# 1. blake3 used for pin (NOT blake2/sha2)
rg "blake3::hash" .elai_cc/crates/plugins/src/mcp_schema_pin.rs
# Expected: ≥1 hit
rg "blake2|sha2" .elai_cc/crates/plugins/src/mcp_schema_pin.rs
# Expected: zero hits

# 2. All 4 hardening modules present
test -f .elai_cc/crates/plugins/src/mcp_schema_pin.rs
test -f .elai_cc/crates/plugins/src/mcp_budget.rs
rg "sanitize_description" .elai_cc/crates/plugins/src/mcp_bridge.rs
rg "transport.*streamable_http|StreamableHttp" .elai_cc/crates/plugins/src/mcp_bridge.rs
# Expected: all 4 succeed

# 3. LLM-guard hook surface
rg "pre_mcp_call_decision|GuardDecision" .elai_cc/crates --type rust
# Expected: ≥2 hits (def + call site)

# 4. Gateway-rejection guard at startup
rg "anthropic_gateway_detected" .elai_cc/crates/cli/src/main.rs
# Expected: ≥1 hit

# 5. Contract tests added
test -f .elai_cc/crates/contracts/tests/mcp_schema_pin_contract.rs
test -f .elai_cc/crates/contracts/tests/mcp_sanitization_line_jumping_contract.rs
test -f .elai_cc/crates/contracts/tests/mcp_budget_enforcement_contract.rs
test -f .elai_cc/crates/contracts/tests/mcp_transport_streamable_http_contract.rs
# Expected: all 4 exist

# 6. Rug-pull regression: postmark-mcp-style fixture
cargo test -p contracts --test mcp_schema_pin_contract postmark_style_rug_pull_blocked
# Expected: PASS

# 7. No new gateway dependencies
grep -E "litellm|openrouter|portkey" .elai_cc/Cargo.lock
# Expected: zero matches
```

## ACCEPTANCE

- [ ] Schema-pin stored on first connect; mismatch on second connect blocks the call and emits SSB row
- [ ] All 5 Trail-of-Bits line-jumping patterns stripped by sanitization with per-rule SSB rows
- [ ] Per-tool token-budget drops lowest-priority tools when `tools/list` exceeds budget
- [ ] Streamable HTTP transport works round-trip; SSE-legacy mode kept behind opt-in flag
- [ ] LLM-guard hook intercepts MCP calls and produces Allow/Deny/Ask decisions
- [ ] Gateway-rejection startup warning fires on any non-local non-anthropic.com URL
- [ ] `elai mcp pin` and `elai mcp pin update` CLI subcommands work
- [ ] 4 new contract tests added and pass
- [ ] No new LLM gateway dependencies in Cargo.lock
- [ ] No blake2 / sha2 in the new code (blake3-only per anti-stale-training)

## Tests to add

- `crates/contracts/tests/mcp_schema_pin_contract.rs`: postmark-style mutation = block; first-connect = pinned
- `crates/contracts/tests/mcp_sanitization_line_jumping_contract.rs`: 5 patterns × per-rule assertion (markdown-fenced cmd, role tags, ignore-prev-instructions, shell prefixes, URL-exfil query params)
- `crates/contracts/tests/mcp_budget_enforcement_contract.rs`: 20-tool fixture with budget=2000 tokens, asserts the dropped set is the lowest-priority tail
- `crates/contracts/tests/mcp_transport_streamable_http_contract.rs`: round-trip a JSON-RPC request via Streamable HTTP; assert no SSE-only path used
- `crates/contracts/tests/llm_guard_decision_contract.rs`: synthetic fixture returns `deny` for a `rm -rf /` payload; bridge blocks; SSB row emitted
- `crates/contracts/tests/gateway_rejection_startup_warning.rs`: ANTHROPIC_BASE_URL = `https://openrouter.ai/...` triggers warning row
- Extend `crates/contracts/tests/fuld_mcp_bridge_contract.rs` (do not rewrite — only add new functions)

## Files touched

- CREATE: `crates/plugins/src/mcp_schema_pin.rs` (~250 LOC)
- CREATE: `crates/plugins/src/mcp_budget.rs` (~180 LOC)
- CREATE: `crates/plugins/src/llm_guard.rs` (~200 LOC) — or under `crates/hooks/src/llm_guard.rs` if a hooks crate exists; verify via Glob
- MODIFY: `crates/plugins/src/mcp_bridge.rs` (+~150 LOC for sanitization rule expansion + budget integration + Streamable HTTP transport + pin verification)
- MODIFY: `crates/plugins/src/lib.rs` (+3 LOC mod declarations)
- MODIFY: `crates/cli/src/main.rs` (+~40 LOC for `mcp pin` subcommand + gateway-rejection guard)
- CREATE: `crates/contracts/tests/mcp_schema_pin_contract.rs`
- CREATE: `crates/contracts/tests/mcp_sanitization_line_jumping_contract.rs`
- CREATE: `crates/contracts/tests/mcp_budget_enforcement_contract.rs`
- CREATE: `crates/contracts/tests/mcp_transport_streamable_http_contract.rs`
- CREATE: `crates/contracts/tests/llm_guard_decision_contract.rs`
- CREATE: `crates/contracts/tests/gateway_rejection_startup_warning.rs`
- MODIFY: `crates/contracts/tests/fuld_mcp_bridge_contract.rs` (+~50 LOC for hardening regression checks)
- CREATE: `crates/dashboard-v2/src/lib/components/McpStatusPanel.svelte` (~120 LOC) OR extend existing if present
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~20 LOC for `get_mcp_status` Tauri command)
- CREATE: `workflow/reasoning/E-24-decisions.md`

## Citations / Grounding

- owasp.org/www-project-mcp-top-10/2025/MCP03-2025-Tool-Poisoning — grounds Task 1 (schema-hash pinning + immutable registry + signed schemas)
- blog.trailofbits.com/2025/04/21/jumping-the-line-how-mcp-servers-can-attack-you-before-you-ever-use-them — grounds Task 2 (line-jumping PoC; description-as-prompt-injection)
- thehackernews.com/2025/09/first-malicious-mcp-server-found.html — postmark-mcp Sept 17 2025 rug-pull, 1,643 downloads, ~300 orgs compromised — grounds Task 1 in-wild case
- modelcontextprotocol.io/specification/2025-11-25/basic/transports — Streamable HTTP transport spec — grounds Task 4
- mindstudio.ai/blog/claude-code-mcp-server-token-overhead — 55,000-token GitHub MCP, 5 servers × 20 tools ≈ 80k tokens — grounds Task 3 budget enforcement
- onlycli.github.io/OnlyCLI/blog/mcp-token-cost-benchmark — MCP 35x token cost vs CLI — grounds Task 3 rationale
- decodethefuture.org/en/anthropic-blocks-third-party-tools — Anthropic Apr 2026 third-party tool block + prompt-cache-break — grounds Task 6 gateway rejection
- venturebeat.com/security/mcp-stacks-have-a-92-exploit-probability — 92% exploit rate — grounds entire hardening posture
- users.rust-lang.org/t/is-it-better-to-use-bincode-or-postcard/88740 — postcard 60ns / ~70% bincode size — grounds REJECT-full-MCP for internal IPC
- workos.com/blog/mcp-2025-11-25-spec-update — Nov 2025 spec additions (Tasks, Sampling-with-Tools, URL-mode elicitation) — grounds the "FSM already does what Tasks does" argument
- reversinglabs.com/blog/mcp-rug-pull-attack-worries — OX Security 9-of-11 registries poisoned, Clawdbot exposure 2,000+ instances — grounds Task 1 urgency
- practical-devsecops.com/mcp-tool-poisoning — Queen's University study, 7.2% of 1,899 OSS MCP servers vulnerable — grounds the base-rate threat
- CVE-2025-49596 (MCP Inspector RCE) — grounds the "no auto-approve" stance

## Open questions / risks

- LLM-guard hook (Task 5) uses the local sidecar; if the sidecar is offline, every MCP call must either fail-closed or fall back to deterministic deny-list only. Open question for the user: which fail-mode is correct? Default proposed: fail-closed when LLM-guard configured but sidecar unavailable, with a clear error and a `--no-llm-guard` opt-out.
- Budget enforcement priority (Task 3): drop-by-lowest-priority requires per-tool priority metadata. If MCP servers don't expose priority, ELAI must maintain its own per-server allowlist with rank. Open question: where does priority live — in `elai_cc.toml` per-tool, or in a separate `.elai_cc/.elai/mcp/priority.toml`?
- Pin update flow: `elai mcp pin update` is a manual override. A malicious server can social-engineer the user into updating. Mitigation: require `--diff` output before commit, showing exactly what changed in the schema. Add this to the CLI behavior.
- Streamable HTTP migration may break third-party MCP servers ELAI consumes that still only speak SSE. The `transport: sse_legacy` flag is the escape hatch but adds maintenance burden. Open question: deprecation timeline? Recommendation: 90-day window from E-24 ship date before SSE-legacy removal.
- C-42 (Zero-Trust MCP Fuzzing) is PLANNED and depends on E-24 surface landing first. Confirm ordering with the user — if C-42 ships before E-24, the fuzz corpus will exercise the un-hardened surface and produce a noisy report.
- Anti-slopsquatting on tiktoken-rs (Task 3): VERIFY the crate name, current version, and last-publish date via `cargo search` or crates.io before any addition. The chars/4 heuristic is acceptable as a no-dep fallback and may be the safer choice for first ship.
- Gateway-rejection guard (Task 6) interaction with the remote sidecar setup: ANTHROPIC_BASE_URL=http://localhost:8100 routes through SSH tunnel to remote 127.0.0.1:8100 per CLAUDE.md remote-sidecar block. The check sees localhost and allows it. Confirm the SSH tunnel is transparent to this check.

## Next

E-22 Task 3 (deferred MCP loading) and E-24 Task 3 (per-tool budget) compose: E-22 reduces base response size, E-24 enforces a hard cap. Both ship together for the full token-economy story. E-25 (compaction ladder) and E-24 Task 5 (LLM-guard) share the `prompt`-type hook primitive — coordinate so the hook handler is a shared crate-level primitive consumed by both. C-42 fuzzing becomes the regression suite for E-24's hardening once both land. After E-24, the README "no surprise pricing" / "no hidden third-party calls" claim from E-23 is fully backed by code: gateway-rejection guard at startup, sanitization at every list, pin verification at every connect.
