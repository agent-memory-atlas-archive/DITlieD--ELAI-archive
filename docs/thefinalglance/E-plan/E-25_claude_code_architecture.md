# E-25: Claude Code architecture steals (6-tier compaction, deferred MCP, skill templating, LLM-decision hook, session fork)
Objective: Port the 6 high-leverage Claude Code harness primitives reverse-engineered by karanprasad / VILA-Lab / agiflow / Pragmatic Engineer into ELAI: 6-tier compaction ladder, deferred MCP tool descriptions, `!\`cmd\`` skill templating with live reload, `prompt`-type LLM-decision hook, opusplan-style plan/execute auto-switch, session forking. Skip Claude Code anti-patterns (pre-trust execution window, per-subcommand parsing event-loop starvation, approval-fatigue masking).
Estimated effort: 3 sessions, ~12 h
Pre-requisites: B-37 SHIPPED (PerRoleRuntime — plan/execute auto-switch attach point), B-22 SHIPPED (autonomous mode + auto-advance + StagnationDetector), Phase 3H (SESSION_HANDOFF wiring — extend, not replace), B-26 DONE-PARTIAL (V2 dashboard polish — fork-session UX surface)
Parallel-safe with: E-21 (different surface — peer-grounded gaps), E-22 (E-22 ships skills-export; E-25 adds dynamic templating on top), E-23 (positioning + UX — E-25 powers some of the UX guards via mechanisms), E-24 (E-24 LLM-guard shares `prompt`-hook primitive with E-25 Task 4)
Source report: workflow/improvement-sweep-2026-05-19/reports/25-claude-code-architecture.md

## Context

The reverse-engineering of Claude Code's ~512K LOC by karanprasad.com established that the agent loop itself is ~40 lines and 98.4% of Claude Code is deterministic harness infrastructure. The InfoQ May 2026 postmortem (infoq.com/news/2026/05/anthropic-claude-code-postmortem/) documented that three independent product changes (effort default Mar 4, cache-clear bug Mar 26, Haiku over-delegation) ALL degraded quality without telemetry catching it — validating ELAI's existing auditor + b-plan-runner gate discipline. The report identifies 6 high-leverage primitives that exist in Claude Code and do NOT yet exist in ELAI.

Primitive 1 (compaction ladder): Claude Code uses a 6-tier ladder (Budget Reduction → Snip → Microcompact → Context Collapse → Auto-Compact → narrative-summary) and the Anthropic API's `cache_edits` parameter to surgically remove tool results WITHOUT busting the prompt cache, retaining ~76% cache benefit per karanprasad.com. ELAI's PreCompact hook writes SESSION_HANDOFF.md and then truncates — naive, single-step, no cache awareness. Even without `cache_edits` (which is an api.anthropic.com server-side feature ELAI's local sidecar can't access), the tier ordering alone saves tokens.

Primitive 2 (deferred MCP descriptions): code.claude.com/docs/en/mcp documents that MCP tool descriptions are deferred — only names load at session start, full schemas fetched on demand via tool-search. mindstudio.ai/blog/claude-code-mcp-server-token-overhead measured 55,000 tokens for 93-tool GitHub MCP; 5 servers × 20 tools ≈ 80,000 tokens schema overhead per turn. ELAI loads every `mcp__local-rag__*` and `mcp__pod-lifecycle__*` schema at session start — same problem. This overlaps with E-22 Task 3 and E-24 Task 3 — E-25 takes the consumer-side configuration and the dashboard surface.

Primitive 3 (skill templating + live reload): code.claude.com/docs/en/skills documents `!\`cmd\`` shell injection BEFORE skill content reaches the model, plus filesystem-watched live reload. ELAI's skill loader at `crates/tools/src/lib.rs` reads static markdown only (per elai.map line 3330). Strict-win, low risk.

Primitive 4 (LLM-decision hook): code.claude.com/docs/en/hooks documents `"type": "prompt"` hook handler that calls a fast model with a custom yes/no prompt — decision returns block / allow / ask. ELAI hooks are command/bash only per CLAUDE.md hook-enforcement block. Highest leverage of the 6 because it composes with every other primitive.

Primitive 5 (opusplan plan/execute auto-switch): code.claude.com/docs/en/model-config documents the `opusplan` alias — Opus for plan, Sonnet for execute. ELAI's RuntimeOrchestrator (B-37) routes per-role, not per-FSM-substate. The extension is small (substate-overlay on PerRoleRuntime) and unlocks cheap eval with no behavior loss.

Primitive 6 (session fork): code.claude.com/docs/en/how-claude-code-works documents `--fork-session` and `/branch` — copies transcript to new session ID, leaves original untouched, useful for "try another approach without losing progress." ELAI's SESSION_HANDOFF is linear. Combined with pre-edit checkpoints (also from Claude Code; code.claude.com/docs/en/how-claude-code-works mentions snapshot-before-edit + Esc-Esc rewind), the user gets cheap "what if I tried this differently" exploration.

Anti-patterns explicitly skipped: pre-trust extension execution window (source of 2 CC CVEs per VILA-Lab analysis), per-subcommand parsing >50 BYPASSES security analysis (VILA-Lab documented safety bypass), 93% approval-rate move-the-boundary decision (VILA-Lab — approval fatigue masking), MCP silent-empty-file-read (substratia.io / nimbalyst.com 2026 reviews), Bash permission patterns that constrain command arguments are fragile (code.claude.com/docs/en/permissions explicit warning).

## Existing state (B/C/D coverage)

- B-22 SHIPPED (autonomous mode): provides auto_advance_decision + autonomy_escalation SSB rows. EXTENSION: E-25 Primitive 5 (opusplan) consumes the same FSM substate signal — no rewrite, just overlay.
- B-37 SHIPPED (Multi-Runtime Backend Abstraction Phase 4M): RuntimeAdapter + PerRoleRuntime + RuntimeOrchestrator + 4 SSB action types + /runtime CLI + RuntimePanel. EXTENSION: E-25 Primitive 5 adds a substate-overlay on PerRoleRuntime; backward-compatible additive.
- Phase 3H (SESSION_HANDOFF wiring per CLAUDE.md PreCompact hook): existing. EXTENSION: E-25 Primitive 1 implements the 6-tier ladder ABOVE the existing PreCompact mechanism — the ladder runs first; if all tiers fail to meet budget, fall through to existing SESSION_HANDOFF write. No replacement.
- B-26 DONE-PARTIAL (V2 dashboard polish): EXTENSION: E-25 Primitive 6 (session fork) adds a small FSM panel surface for the fork UX.
- E-22 Task 3 (deferred MCP loading) and E-24 Task 3 (per-tool token-budget): OVERLAP with E-25 Primitive 2. E-25 owns the dashboard-side surface and the configuration semantics; the underlying lazy-schema mechanism is shared. Coordinate landing order — Primitive 2 lands AFTER E-22 Task 3 OR is rolled into E-22.
- E-24 Task 5 (LLM-guard hook): OVERLAP with E-25 Primitive 4. E-25 owns the generic `prompt`-type hook handler; E-24 consumes it for the pre-MCP-call guard. Pull the handler into a shared crate-level primitive (`crates/hooks/src/prompt_handler.rs` or `crates/hooks/src/llm_decision.rs`).
- B-39 SHIPPED (Chat REFLECT pipeline): provides the precedent for "LLM as decision-maker" in ELAI — the REFLECT triage step calls the sidecar with a structured prompt and acts on the response. E-25 Primitive 4 reuses this pattern for hooks.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\25-claude-code-architecture.md` (full)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\session_dag.rs` (existing create_handoff path; per elai.map line 3125)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_orchestrator.rs` (B-37 PerRoleRuntime; per elai.map line 870)
- `F:\.ELAI_workflow\.elai_cc\crates\tools\src\lib.rs` lines 3320-3340 area (existing skill loader; per elai.map line 3330)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\hook_dispatcher.rs` (per elai.map line 41694 — existing hook dispatcher)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\resume.rs` (existing resume path — fork attaches here)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\live_cli.rs` lines 1420-1435 (handoff entrypoint per elai.map line 45)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 290-400 (Phase 3H / B-15..B-23 harness context)
- code.claude.com/docs/en/how-claude-code-works (WebFetch — 6-tier compaction + fork-session reference)
- code.claude.com/docs/en/skills (WebFetch — dynamic context injection + live reload)
- code.claude.com/docs/en/hooks (WebFetch — `prompt` handler type)
- karanprasad.com/blog/how-claude-code-actually-works-reverse-engineering-512k-lines (WebFetch — 6-tier ladder detail + cache_edits)
- infoq.com/news/2026/05/anthropic-claude-code-postmortem (WebFetch — what NOT to do)

## Tasks

1. 6-tier compaction ladder. New module `crates/orchestrator/src/compaction_ladder.rs` (~400 LOC). Implements the 6 tiers as a sequence of `CompactionTier` strategies executed cheapest-first, each producing a `(new_messages, freed_tokens, tier_id)` triple. Tiers per karanprasad.com: (T1) Budget Reduction — strip whitespace, dedupe identical tool calls; (T2) Snip — drop oldest tool-result payloads >2KB while keeping their summaries; (T3) Microcompact — summarize 3 oldest user/assistant pairs into one bullet list per pair; (T4) Context Collapse — fold all >5-turn-old tool results into a single "older work" summary; (T5) Auto-Compact — invoke sidecar with a "summarize this transcript preserving X" prompt; (T6) Narrative-summary — last-resort full re-narrate by sidecar. Ladder stops at the first tier that drops below the target budget. Each tier emits `compaction_tier_applied` SSB row with `tier_id`, `freed_tokens`, `wall_ms`. The existing PreCompact SESSION_HANDOFF write runs AFTER the ladder if budget still not met (NOT replaced). Cache-aware ordering: the ladder preserves the first N tokens of the prompt (system + CLAUDE.md + first user turn) to maximise the chance of remote sidecar's prompt-cache (if any) hitting — emit a `compaction_cache_preserved` SSB row when the cache-preservation bytes are unchanged. CACHE_EDITS NOTE: api.anthropic.com `cache_edits` is server-side and unavailable through the local sidecar — document this in the module-level doc comment.

2. Deferred MCP tool description loading (E-25 surface). The lazy-schema mechanism is shipped by E-22 Task 3 (or C-35). E-25 adds: (a) a `--lazy-mcp-schemas` CLI flag and `[mcp.lazy_schemas]` `elai_cc.toml` block; (b) a dashboard `McpSchemaPanel.svelte` showing for each MCP server: tool count, schema-loaded count, bytes saved; (c) an SSB row `mcp_schema_lazy_fetch` per on-demand fetch. If E-22 / C-35 has not landed, this task is BLOCKED — do not duplicate the underlying mechanism.

3. `!\`cmd\`` skill templating + live reload. Extend `crates/tools/src/lib.rs` skill loader (per elai.map line 3330) to (a) scan SKILL.md body for `` !` ... ` `` blocks AND for triple-backtick `!` fenced blocks; (b) execute each in a constrained shell (default-deny network; default-deny FS writes outside the skill dir; 5s timeout; per CLAUDE.md zero-lazy-code rule, the shell is `microsandbox` if available per E-22 Task 2, otherwise process-isolated with a `tracing::warn!` `skill_template_unsandboxed`); (c) replace the placeholder block with stdout before the skill content is handed to the model. Live reload via `notify` crate (verify on crates.io / WebSearch first per anti-slopsquatting; current ELAI may already use it — Grep). When a `.claude/skills/{name}/SKILL.md` changes, the loader updates its in-memory entry and emits `skill_hot_reloaded` SSB row. Reference: code.claude.com/docs/en/skills "dynamic context injection" + "live change detection."

4. `prompt`-type LLM-decision hook handler (shared primitive). New `crates/hooks/src/prompt_handler.rs` (~250 LOC) OR if no `hooks` crate exists, place under `crates/orchestrator/src/hook_prompt.rs`. Verifies crate layout via Glob before deciding. The handler signature: `pub async fn prompt_decision(prompt: &str, schema: PromptSchema, runtime: &dyn RuntimeAdapter) -> Result<PromptDecision>`. PromptDecision = Allow | Deny(reason) | Ask(reason). Calls the runtime with a fixed system prompt: "You are a deterministic decision-maker. Read the question. Output STRICT JSON {\"decision\": \"allow|deny|ask\", \"reason\": \"...\"}. No prose." Times out at 5s. JSON-parses the response; on parse failure, returns `Err(PromptHandlerError::InvalidDecisionFormat)`. Used by E-24 Task 5 (pre-MCP-call guard) AND by any future hook that wants LLM-decision semantics. Per CLAUDE.md anti-stale-training, the runtime defaults to the local Qwen 3B sidecar via PerRoleRuntime's `assistant_decision` role.

5. opusplan-style plan/execute runtime overlay. Extend `RuntimeOrchestrator` (B-37) with a `substate_overlay: HashMap<FsmSubstate, RoleRuntimeOverride>` field. When the FSM transitions into substate `Framing` / `Planning` / `Scoping` / `Decomposing`, the override routes `assistant_primary` role to the configured planner-runtime (default: cloud Sonnet, configurable). When transitioning into `Executing` / `Fixing`, route back to the configured executor-runtime (default: local Qwen 3B sidecar). Emit `runtime_substate_overlay_active` SSB row. Reference: code.claude.com/docs/en/model-config opusplan alias. Backward compatible: empty overlay = current B-37 behavior.

6. Session forking + pre-edit checkpoints. (6a) New `crates/cli/src/fork.rs` (~180 LOC). `elai resume --fork <new_session_id>` clones the current SESSION_HANDOFF + memory-bank + FSM state into a new session directory, leaves the original session intact, marks the new session's lineage via `forked_from: <original_session_id>` in its handoff metadata. (6b) New `crates/orchestrator/src/edit_checkpoints.rs` (~200 LOC). PreToolUse hook for Edit / Write captures a snapshot of the target file at `.elai_cc/.session-checkpoints/{session_id}/{seq}/{path}` BEFORE the write proceeds. Snapshots evicted on session END or on git commit (verify the commit-hook hook point exists). `elai checkpoint rewind --steps N` restores the previous snapshot. Reference: code.claude.com/docs/en/how-claude-code-works "checkpoints snapshot files before edits" + `--fork-session` / `/branch`.

7. Anti-pattern guards. Explicit guards against the 7 Claude Code anti-patterns the report flagged: (a) NO pre-trust extension execution — MCP / pod-lifecycle / local-rag startup MUST be gated on the gateway-rejection guard from E-24 Task 6, no side effects before that check; (b) NO per-subcommand parsing bypass at >50 — if a Bash command has >50 subcommands, BLOCK with `bash_too_complex` SSB row (do NOT inherit the silent bypass); (c) NO approval-fatigue boundary widening — if auto-approval rate exceeds 80% over a rolling 50-decision window, emit `approval_fatigue_detected` SSB row and require explicit user reconfirmation on the next prompt (per CLAUDE.md anti-sycophancy rule); (d) NO silent-empty-file-read for mcp_search_context — explicitly distinguish "no results" vs "RAG corpus empty" vs "topic not registered" in the return shape; (e) keep `model=sonnet` global default per CLAUDE.md and surface any per-agent override in the status line so Haiku-over-delegation cannot happen invisibly; (f) NO Bash regex argument matching — defer to E-24 Task 5 LLM-guard; (g) telemetry every quality regression (auditor pass-rate, ungrounded-claim count, revert rate) per E-23 trust-telemetry plane, so the InfoQ-style 6-week-undetected regression cannot happen in ELAI without an SSB row firing.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::compaction_ladder::CompactionLadder { tiers: Vec<Box<dyn CompactionTier>> }
  - orchestrator::compaction_ladder::CompactionLadder::run(msgs: Vec<Message>, target_budget: usize) -> CompactionResult
  - orchestrator::compaction_ladder::CompactionResult { messages, freed_tokens, tiers_applied, cache_preserved_bytes }
  - tools::skill_loader::ShellInjectionResult (new struct in existing skill loader)
  - tools::skill_loader::watch_skill_dirs() -> JoinHandle (live reload)
  - hooks::prompt_handler::prompt_decision(prompt, schema, runtime) -> PromptDecision
  - hooks::prompt_handler::PromptDecision { Allow, Deny(reason), Ask(reason) }
  - orchestrator::runtime_orchestrator::RuntimeOrchestrator::with_substate_overlay (extension)
  - cli::fork::fork_session(orig: &SessionId, target: &SessionId) -> Result<()>
  - orchestrator::edit_checkpoints::EditCheckpoints { dir: PathBuf }
  - orchestrator::edit_checkpoints::EditCheckpoints::snapshot_before_edit(session_id, seq, path) -> Result<()>
  - orchestrator::edit_checkpoints::EditCheckpoints::rewind(session_id, steps) -> Result<()>
consumes_symbols:
  - crates/orchestrator/src/session_dag.rs::create_handoff (E-25 ladder runs first)
  - crates/orchestrator/src/runtime_orchestrator.rs::PerRoleRuntime (substate overlay)
  - crates/tools/src/lib.rs::skill_loader (extended with shell injection + live reload)
  - crates/orchestrator/src/hook_dispatcher.rs::HookPoint (prompt-handler invocation site)
  - crates/cli/src/resume.rs (fork-resume entry)
  - blake3 crate (already present; used for checkpoint integrity)
  - notify crate (verify presence via Cargo.lock Grep; CONFIRM with user before adding if absent)
reachable_from:
  - crates/cli/src/main.rs (`elai resume --fork`, `elai checkpoint rewind`, `--lazy-mcp-schemas` flag)
  - crates/orchestrator/src/session_dag.rs::on_pre_compact (ladder entrypoint)
  - crates/orchestrator/src/hook_dispatcher.rs::dispatch (prompt-handler dispatch path)
  - crates/orchestrator/src/task_runner.rs (substate overlay consumption via RuntimeOrchestrator)
  - crates/tools/src/lib.rs::load_skill_from_dir (shell injection extension)
stub_ban:
  - no `// TODO: implement tier N` — all 6 tiers required (lower 3 can be naive but must be non-stub)
  - no `unimplemented!()` in compaction_ladder.rs
  - no .unwrap() in production paths
  - no synchronous file IO in compaction ladder (must be async because sidecar invocation is async)
  - no silent fallback to no-op when sidecar unavailable in T5 / T6 — must skip with `compaction_tier_skipped` SSB row
  - no live-reload race: file modification during in-flight skill invocation must use a generation counter; old in-flight invocations complete with old content, new invocations get new content
  - no shell-injection block executed without sandbox flag check — if microsandbox unavailable on the host, emit `skill_template_unsandboxed` warning and apply the same `.elai_cc/.elai/state/` jail
frontend_pair: crates/dashboard-v2/src/lib/components/CompactionLadderPanel.svelte (new, ~140 LOC — visualises tier application across recent compactions); crates/dashboard-v2/src/lib/components/McpSchemaPanel.svelte (new, ~120 LOC — Primitive 2 dashboard surface); crates/dashboard-v2/src/lib/components/SessionForkPanel.svelte (new, ~100 LOC — fork UX)
observable_effect:
  trigger: synthetic 30K-token transcript fed through compaction_ladder.rs with target=20K
  expect_ssb: ≥1 "compaction_tier_applied" SSB row per applied tier, with non-zero freed_tokens; ≥1 "compaction_cache_preserved" row
  expect_artifact: result.messages token count under 20K; tiers_applied list is non-empty and stops at first tier that meets budget

## DETERMINISTIC_CHECK

```bash
# 1. Compaction ladder has 6 distinct tiers
rg "struct.*Tier|impl CompactionTier for" .elai_cc/crates/orchestrator/src/compaction_ladder.rs
# Expected: ≥6 hits (one per tier implementation)

# 2. Skill loader shell injection wired
rg '!`|backtick_bang|ShellInjection' .elai_cc/crates/tools/src/lib.rs
# Expected: ≥1 hit

# 3. prompt_handler primitive exists and is consumed by E-24 surface
rg "prompt_decision" .elai_cc/crates --type rust
# Expected: ≥2 hits (def + at least one call site)

# 4. RuntimeOrchestrator substate overlay
rg "substate_overlay|with_substate_overlay" .elai_cc/crates/orchestrator/src --type rust
# Expected: ≥2 hits

# 5. Fork and checkpoint subcommands wired
rg "fn fork_session|fn rewind" .elai_cc/crates --type rust
# Expected: ≥2 hits

# 6. Anti-pattern guards
rg "bash_too_complex|approval_fatigue_detected|skill_template_unsandboxed" .elai_cc/crates --type rust
# Expected: ≥3 hits (one per guard SSB row emit site)

# 7. Smoke: 30K-token compaction
cargo test -p orchestrator compaction_ladder_30k_to_20k
# Expected: PASS

# 8. Live skill reload smoke
cargo test -p tools skill_live_reload_picks_up_edit
# Expected: PASS

# 9. blake3 (NOT blake2/sha2) for checkpoint integrity
rg "blake3" .elai_cc/crates/orchestrator/src/edit_checkpoints.rs
# Expected: ≥1 hit
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/edit_checkpoints.rs
# Expected: zero hits
```

## ACCEPTANCE

- [ ] All 6 compaction tiers implemented and individually testable
- [ ] Ladder stops at first tier meeting target budget; emits tier-applied SSB rows
- [ ] Cache-preserved bytes recorded per compaction
- [ ] `!\`cmd\`` blocks in SKILL.md execute in sandbox and stdout inlined before model sees content
- [ ] Live skill reload picks up file changes within 1s without restart; in-flight invocations complete with old content
- [ ] `prompt_decision` returns Allow / Deny / Ask in <5s with strict JSON
- [ ] E-24 Task 5 (LLM-guard for MCP calls) consumes the shared prompt_handler primitive (cross-check by Grep)
- [ ] Substate overlay routes Planning → planner-runtime, Executing → executor-runtime
- [ ] `elai resume --fork <id>` clones session state; original untouched; forked_from metadata recorded
- [ ] Pre-edit checkpoints written to `.elai_cc/.session-checkpoints/` and rewindable via CLI
- [ ] 7 anti-pattern guards present and tested
- [ ] Three new dashboard panels render with live data
- [ ] No regression in existing PreCompact SESSION_HANDOFF behavior (it still runs as fall-through)

## Tests to add

- `crates/orchestrator/tests/compaction_ladder_each_tier.rs`: 6 sub-tests, one per tier, asserting freed_tokens > 0 on a tier-targeted fixture
- `crates/orchestrator/tests/compaction_ladder_30k_to_20k.rs`: end-to-end ladder reduces 30K → ≤20K
- `crates/orchestrator/tests/compaction_cache_preserved_bytes.rs`: cache-preserved bytes unchanged across the ladder
- `crates/tools/tests/skill_shell_injection.rs`: `!`echo hello`` in SKILL.md body, asserts "hello" in pre-model content
- `crates/tools/tests/skill_live_reload_picks_up_edit.rs`: write SKILL.md, observe loaded content, modify, observe new content within 1s
- `crates/hooks/tests/prompt_decision_strict_json.rs`: malformed response returns Err; valid JSON returns Allow/Deny/Ask
- `crates/orchestrator/tests/runtime_substate_overlay.rs`: FSM Planning → planner-runtime, Executing → executor-runtime
- `crates/cli/tests/fork_session_clones_state.rs`: fork preserves memory-bank + FSM; original session unmodified
- `crates/orchestrator/tests/edit_checkpoints_rewind.rs`: edit file, rewind, file contents match snapshot
- `crates/orchestrator/tests/anti_pattern_guards.rs`: 7 sub-tests for each guard SSB row firing

## Files touched

- CREATE: `crates/orchestrator/src/compaction_ladder.rs` (~400 LOC)
- CREATE: `crates/hooks/src/prompt_handler.rs` (~250 LOC) — verify hooks crate exists via Glob; if absent, place at `crates/orchestrator/src/hook_prompt.rs`
- CREATE: `crates/orchestrator/src/edit_checkpoints.rs` (~200 LOC)
- CREATE: `crates/cli/src/fork.rs` (~180 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/CompactionLadderPanel.svelte` (~140 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/McpSchemaPanel.svelte` (~120 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/SessionForkPanel.svelte` (~100 LOC)
- MODIFY: `crates/orchestrator/src/session_dag.rs` (+~30 LOC for ladder invocation in PreCompact path)
- MODIFY: `crates/orchestrator/src/runtime_orchestrator.rs` (+~80 LOC for substate_overlay field + apply logic)
- MODIFY: `crates/orchestrator/src/hook_dispatcher.rs` (+~40 LOC for prompt-handler dispatch type)
- MODIFY: `crates/tools/src/lib.rs` (+~150 LOC for shell injection + live reload — verify approximate insertion point at the existing skill_loader area per elai.map line 3330)
- MODIFY: `crates/cli/src/main.rs` (+~50 LOC for `resume --fork`, `checkpoint rewind`, `--lazy-mcp-schemas` flag)
- MODIFY: `crates/cli/src/resume.rs` (+~30 LOC for fork delegation)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~40 LOC for 3 new Tauri commands)
- CREATE: `workflow/reasoning/E-25-decisions.md`

## Citations / Grounding

- karanprasad.com/blog/how-claude-code-actually-works-reverse-engineering-512k-lines — 6-tier compaction + cache_edits + 98.4% deterministic infra split — grounds Task 1 architecture
- code.claude.com/docs/en/how-claude-code-works — official compaction reference + checkpoints + fork-session — grounds Tasks 1, 6
- code.claude.com/docs/en/skills — dynamic context injection + live change detection — grounds Task 3
- code.claude.com/docs/en/hooks — `"type": "prompt"` handler — grounds Task 4
- code.claude.com/docs/en/model-config — opusplan alias — grounds Task 5
- code.claude.com/docs/en/mcp — deferred tool description loading — grounds Task 2
- mindstudio.ai/blog/claude-code-mcp-server-token-overhead — 55,000-token GitHub MCP overhead — grounds Task 2 token-economy claim
- agiflow.io/blog/claude-code-internals-reverse-engineering-prompt-augmentation — 6 injection layers — grounds the compaction-vs-prompt-augmentation boundary
- vrungta.substack.com/p/claude-code-architecture-reverse — runtime-is-dumb-model-is-CEO + primitive-tools-beat-custom-integrations — grounds the choice to NOT add MCP gateway in front (E-24 cross-reference)
- newsletter.pragmaticengineer.com/p/building-claude-code-with-boris-cherny — glob+grep driven by model beats custom integrations; rejected vector DB — grounds the static elai.map + SCIP positioning per E-22 Task 5 cross-reference
- github.com/VILA-Lab/Dive-into-Claude-Code — pre-trust execution window weakness + per-subcommand >50 bypass + approval-fatigue masking — grounds Task 7 anti-pattern guards
- infoq.com/news/2026/05/anthropic-claude-code-postmortem — 6-week-undetected quality regression — grounds Task 7 telemetry guard rationale
- substratia.io/blog/best-mcp-servers-claude-code + nimbalyst.com/blog/best-claude-code-mcp-servers — MCP silent-empty-file-read antipattern — grounds Task 7 guard (d)
- CLAUDE.md anti-stale-training block — grounds blake3-only + tokio::sync::Mutex / native async-trait choices throughout

## Open questions / risks

- Anthropic `cache_edits` API parameter is server-side at api.anthropic.com. The local sidecar (per CLAUDE.md remote-sidecar block) is custom Python that does not implement cache_edits semantics — the cache-preservation gains are theoretical for local-only runs. Open question: when ELAI is configured for cloud Sonnet (B-37 runtime selection), should the compaction ladder pass `cache_edits` parameter through? Verify the local sidecar's protocol passthrough behavior before committing this path.
- Live skill reload (Task 3) on Windows: `notify` crate has platform differences for file-system events. CONFIRM Windows file-watch behavior via WebSearch; the elai.bat boot path uses Windows file paths. Risk: silent missed updates on Windows network drives.
- Substate overlay (Task 5): if the planner-runtime is cloud-Sonnet and the user is offline, planning fails. Open question: graceful fallback to executor-runtime when planner-runtime unavailable? Default proposed: yes, with `substate_overlay_fallback` SSB row.
- Fork sessions (Task 6a) cloning memory-bank: the memory-bank/ directory is large; cloning is wasteful for try-this-different exploration. Open question: shallow-clone via hardlink + copy-on-write, or full duplication? Hardlinks work on Linux + NTFS but not on all FS — needs verification.
- Edit checkpoints (Task 6b) eviction policy: snapshots on every Edit + Write tool call could balloon `.session-checkpoints/` to GB-scale on long sessions. Open question: keep last 10? Keep until session END? Compress with zstd?
- Approval-fatigue guard (Task 7c) threshold: 80% over 50 decisions is a guess. Open question for the user: is this calibrated to your usage, or should it be configurable in `elai_cc.toml`?
- Sandbox in skill templating (Task 3) depends on E-22 Task 2 (microsandbox); if E-22 hasn't landed, Task 3's sandbox falls back to "process-isolated + warn" which is materially weaker. Acceptable for first ship; harden in follow-up.
- Anti-pattern guard (Task 7b) >50 subcommands BLOCK behavior: ELAI users may legitimately run compound commands. Open question: is 50 the right ceiling, or should it be configurable with a documented escape hatch (`--allow-complex-bash`)?

## Next

E-25 unblocks the largest set of subsequent improvements: E-21's repo-as-state-store benefits from session forking; E-22's lazy_schemas finds its dashboard surface in Task 2; E-24's LLM-guard consumes Task 4's prompt_handler; E-23's context-state panel and pricing badge are powered by ladder telemetry + substate overlay. After E-25, ELAI matches Claude Code on the 6 highest-leverage harness primitives AND has structural guards against the 7 documented Claude Code anti-patterns. The 1% AI / 99% deterministic-infra ratio that karanprasad.com measured in Claude Code becomes a defensible feature claim for ELAI rather than aspirational copy.
