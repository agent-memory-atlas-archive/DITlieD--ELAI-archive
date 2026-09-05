# Verification Report: B-36_dashboard_v2_src_tauri_wiring.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-36_dashboard_v2_src_tauri_wiring.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The core B-36 implementation is substantially complete and well beyond the plan's baseline description. The lib.rs is 777 lines (not 28) with 145 tauri::command handlers registered; all 13 D-commands and all 15 net-new surfaces are implemented in commands.rs. Cargo.toml has engine crates (elai-orchestrator, elai-providers, elai-core) and the binary name constant is already fixed to "elai-dashboard-v2". The main gaps are: stub.ts uses a tryInvoke+mock-fallback pattern (not the planned offline-flag toggle), the three Session-1 analysis documents are absent, and contract tests were never added.

## Claims verified DONE

- lib.rs registers 2 baseline commands (greet + get_grounding_cache_stats) — evidence: crates/dashboard-v2/src-tauri/src/lib.rs:61,189
- lib.rs is now ~600+ lines after wiring — evidence: lib.rs is 777 lines (confirmed wc -l)
- state.rs AppState struct with Arc<Mutex<...>> handles exists — evidence: crates/dashboard-v2/src-tauri/src/state.rs:212-317
- AppState registered via .manage() in Tauri builder — evidence: lib.rs:59
- tauri_plugin_dialog::init() plugin attached — evidence: lib.rs:58
- Cargo.toml has elai-orchestrator, elai-core, elai-providers deps — evidence: src-tauri/Cargo.toml:36,41,42
- Cargo.toml has tokio and reqwest deps — evidence: src-tauri/Cargo.toml:30,31
- Binary name is "elai-dashboard-v2" — evidence: src-tauri/Cargo.toml:15 [[bin]] name
- DASHBOARD_BINARY_NAME fixed to "elai-dashboard-v2" — evidence: crates/cli/src/dashboard.rs:10
- ELAI_DASHBOARD_BINARY env-var override also present — evidence: crates/cli/src/dashboard.rs:102
- D.1 get_sidecar_health() — GET /health probe returning vram/model/dola shape — evidence: commands.rs:3513
- D.2 run_planning_task() — SSB-actuated, emits "planning-progress" event — evidence: commands.rs:4237,4268
- D.3 run_research() — drives DefaultWebResearchRunner, emits "research-done" — evidence: commands.rs:4354
- D.4 fetch_settings_get(), fetch_settings_set(), obscura_test_fetch() — evidence: commands.rs:4496,4511,4604
- D.5 run_d5_live_contract() — evidence: commands.rs:4733
- D.6 reindex_now() — evidence: commands.rs:4013
- D.7 one_shot_prompt() — spawns elai.exe via tokio::process::Command — evidence: commands.rs:4871
- D.8 get_tool_visibility() — mirrors MODEL_HIDDEN_TOOL_NAMES const locally — evidence: commands.rs:4942,4915
- D.9 set_local_max_context(), get_local_budget() — evidence: commands.rs:2862,2870
- D.10 set_local_min_prompt(), get_local_min_prompt() — evidence: commands.rs:2880,2888
- D.11 sidecar_search() — evidence: commands.rs:5096
- D.12 get_grounding_cache_stats() — evidence: commands.rs:3482
- D.13 get_sidecar_settings(), set_sidecar_settings(), restart_sidecar() — evidence: commands.rs:4999,5021,5047
- Net-new: recompute_steering_vector — evidence: commands.rs:751
- Net-new: get_steering_status — evidence: commands.rs:740
- Net-new: get_mcp_status — evidence: commands.rs:793
- Net-new: get_gg_signals — evidence: commands.rs:1583
- Net-new: get_dkg_stats — evidence: commands.rs:878
- Net-new: get_sidecar_status — evidence: commands.rs:2207
- Net-new: get_bubble_up_events — evidence: commands.rs:2034
- Net-new: get_hook_executions — evidence: commands.rs:2661
- Net-new: get_shortcoder_stats — evidence: commands.rs:2684
- Net-new: run_explore_query — evidence: commands.rs:2799
- Net-new: continue_task — evidence: commands.rs:2850
- Net-new: get_grc_budgets — evidence: commands.rs:1515
- Net-new: get_evidence_gates — evidence: commands.rs:1616
- Net-new: get_context_budget — evidence: commands.rs:1695
- Net-new: get_search_mode — evidence: commands.rs:831
- elai-cli NOT added as a dep; MODEL_HIDDEN_TOOL_NAMES mirrored locally (no-new-dep rule respected) — evidence: commands.rs:4912-4915, Cargo.toml has no elai-cli entry
- commands/ subdirectory not created (lib.rs thin, commands.rs carries handlers) — evidence: ls shows no commands/ dir, lib.rs 777 lines delegates to commands module

## Claims PARTIAL

- stub.ts replaces mock returns with invoke() for wired commands
  - exists: stub.ts uses tryInvoke() wrapper (ensureInvoke() -> tauri invoke) for wired commands; stub.ts:183-198, stub.ts:222
  - gap: pattern is tryInvoke-with-mock-fallback, NOT the planned ELAI_DASHBOARD_OFFLINE=1 flag-gated offline mode. No ELAI_DASHBOARD_OFFLINE env var found in stub.ts. Fallback always active on error, not behind explicit flag toggle.

- D.7 one_shot_prompt(text, backend, port, allowed_tools) signature
  - exists: one_shot_prompt implemented — evidence: commands.rs:4871
  - gap: implementation takes only `text: String`; no backend/port/allowed_tools params; does not set ANTHROPIC_BASE_URL/ELAI_LOCAL_MIN_PROMPT env vars on spawn; uses plain `["prompt", &text]` args not --backend flag.

- D.2 run_planning_task streams web_research_step rows via Window::emit("web_research_step", ...)
  - exists: run_planning_task emits "planning-progress" event (not "web_research_step") — evidence: commands.rs:4268
  - gap: plan specified event name "web_research_step"; actual event is "planning-progress". D.3 run_research emits "research-done", not a streaming step-by-step "web_research_step" feed.

- AppState holds Arc<dyn SsbWriter> and Arc<ResearchCache>
  - exists: AppState has writer: Option<Arc<Ssb>> (not Arc<dyn SsbWriter> trait object) — evidence: state.rs:220
  - gap: SsbWriter trait abstraction not present; ResearchCache not a distinct named field; plan specified typed Arc<ResearchCache> for Track A.6/D.7. No ResearchCache field found in AppState.

## Claims MISSING

- docs/specs/track-d-v2-spec.md (Session 1 deliverable A, >=6 sections >=13 entries) — searched: Glob docs/specs/track-d*, file not found
- docs/thefinalglance/B-plan-delta-2026-05-07.md (Session 1 deliverable B) — searched: Glob docs/thefinalglance/B-plan-delta*, file not found
- Contract tests in crates/contracts/tests/dashboard_v2_*_contract.rs — searched: Glob crates/contracts/tests/dashboard_v2*, no files found
- ELAI_LIVE_SIDECAR-gated end-to-end test for D.1+D.11 — searched: grep ELAI_LIVE_SIDECAR in contracts/tests; not found in any dashboard_v2 test file

## Claims UNVERIFIABLE

- workflow/sessions/<sid>/HANDOFF-track-d-v2-impl.md (Session 1 deliverable C) — reason: session ID is a runtime value; memory-bank refs are UNVERIFIABLE per auditor rules
- Visual smoke screenshots (workflow/sessions/<sid>/screenshots/D-<item>.png) — reason: session-scoped artifact, not checkable without specific sid
- pnpm tauri:build passes — reason: requires live build execution; no CI artifact in codebase
- ELAI_LIVE_SIDECAR live sidecar test passing — reason: requires remote sidecar; not verifiable statically

## What must be done (action list)

- [ ] Add ELAI_DASHBOARD_OFFLINE=1 flag gating to stub.ts so offline mode is explicit (not always-on error fallback). Target: crates/dashboard-v2/src/lib/api/stub.ts
- [ ] Expand one_shot_prompt signature to accept backend, port, and allowed_tools; set ANTHROPIC_BASE_URL/ELAI_LOCAL_MIN_PROMPT/ELAI_LOCAL_MAX_CONTEXT on the spawned process. Target: commands.rs:4871
- [ ] Rename the planning-progress Tauri emit to web_research_step (or document deviation from spec). Target: commands.rs:4268
- [ ] Add contract tests: one per D-command in crates/contracts/tests/dashboard_v2_<cmd>_contract.rs using Tauri MockRuntime
- [ ] Add ELAI_LIVE_SIDECAR-gated end-to-end test for D.1 (get_sidecar_health) + D.11 (sidecar_search)
- [ ] Create docs/specs/track-d-v2-spec.md (Session 1 analysis document A, >=6 sections, >=13 D-row entries)
- [ ] Create docs/thefinalglance/B-plan-delta-2026-05-07.md (Session 1 analysis document B, >=10 B-plan files reviewed)
- [ ] Create workflow/sessions/<sid>/HANDOFF-track-d-v2-impl.md with phase-cluster table (Session 1 deliverable C)

## Notes

The plan baseline description ("EXACTLY TWO commands: greet and get_grounding_cache_stats", "28-line state") is stale relative to the actual codebase — the dashboard-v2 backend was heavily extended before/during B-36 execution, now at 777 lines with 145 handlers. The elai-cli dep was explicitly NOT added (MODEL_HIDDEN_TOOL_NAMES mirrored locally). The commands/ subdirectory was not needed as commands.rs handles all handlers. Tauri command count (145) exceeds the plan's >=28 minimum by a wide margin. The three Session-1 analysis documents are the only substantial process gaps.

REPORT_WRITTEN=F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-36_dashboard_v2_src_tauri_wiring.report.md | status=MOSTLY_DONE | done=38 | partial=4 | missing=4 | unverifiable=4
