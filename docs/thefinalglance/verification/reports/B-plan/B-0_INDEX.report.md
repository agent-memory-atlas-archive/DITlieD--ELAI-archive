# Verification Report: B-0_INDEX.md
Source: docs/thefinalglance/B-plan/B-0_INDEX.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary

B-0_INDEX.md is the master index for the B-plan sequence. It records per-session
shipping status for B-1 through B-39C plus the oracle-gate patch. The document
covers Part I-VII plans and embeds live status notes from three main sessions
(s-59964, s-26716, s-30584, s-34648). Core wiring claims in the session status
blocks are well-supported by code evidence. The INDEX is, however, stale relative
to the current codebase: B-37, B-40, B-41, B-42 shipped after the index was last
updated (SCHEMA_VERSION 46 in ssb.rs; index last mentions v26 as "confirmed"). The
B-36 final status flip (PARTIAL -> DONE) is supported by code evidence for
run_planning_task and run_research existing in commands.rs. All plan-section
entries (Part III-VII) remain aspirational descriptions of unshipped work.

## Claims verified DONE

- B-1 path_unification contract test file exists — evidence: [path_unification.rs](.elai_cc/crates/contracts/tests/path_unification.rs)
- B-2 set_ssb_writer wired in fsm; OnceLock<Mutex<Option<Arc<Ssb>>>> — evidence: [live_cli.rs:743](.elai_cc/crates/cli/src/live_cli.rs#L743) and [startup.rs:97](.elai_cc/crates/orchestrator/src/startup.rs#L97)
- B-2 PhaseTransitionPayload struct defined — evidence: [ssb.rs:1450](.elai_cc/crates/orchestrator/src/ssb.rs#L1450)
- B-4 CircuitBreakerStateChangePayload struct defined — evidence: [ssb.rs:1730](.elai_cc/crates/orchestrator/src/ssb.rs#L1730)
- B-6 EvidenceGateEvaluatedPayload struct defined — evidence: [ssb.rs:1805](.elai_cc/crates/orchestrator/src/ssb.rs#L1805)
- B-6 SupervisionEventPayload struct defined — evidence: [ssb.rs:1818](.elai_cc/crates/orchestrator/src/ssb.rs#L1818)
- B-6 BspSuperstep emitted in bsp/engine.rs — evidence: [bsp/engine.rs:454](.elai_cc/crates/orchestrator/src/bsp/engine.rs#L454)
- B-6 AgentLifecycle emitted at multiple sites in bsp/engine.rs — evidence: [bsp/engine.rs:353,509,580,599](.elai_cc/crates/orchestrator/src/bsp/engine.rs#L353)
- B-6 AntiSpiralHit variant present in ssb.rs — evidence: ssb.rs SCIP_REF row confirmed in elai.map
- B-7 BubbleUpTyped, IntegrationCheckpointSuccessPayload, DagRepairRoundPayload, ConceptDecompositionNodePayload — evidence: [ssb.rs:1769,1791](.elai_cc/crates/orchestrator/src/ssb.rs#L1769) and [pipeline_builder.rs:17](.elai_cc/crates/orchestrator/src/pipeline_builder.rs#L17)
- B-7b (follow-up) WebResearchStepEmitter wired at SsbWebResearchStepEmitter — evidence: [live_cli.rs:2312](.elai_cc/crates/cli/src/live_cli.rs#L2312)
- B-7b get_bubble_up_events command implemented — evidence: [commands.rs:2034](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L2034)
- B-8 PipelineGrc::new() and pipeline_grc_wired_contract.rs — evidence: [pipeline_grc_wired_contract.rs:38](.elai_cc/crates/contracts/tests/pipeline_grc_wired_contract.rs#L38) and [grc.rs:239](.elai_cc/crates/orchestrator/src/grc.rs#L239)
- B-9 BranchTree.svelte exists with statusColor/relativeTime fns — evidence: [BranchTree.svelte:19](.elai_cc/crates/dashboard-v2/src/lib/ui/BranchTree.svelte#L19)
- B-10 context_budget_computed_snapshot() in ssb_viewer + get_context_budget wired in commands.rs — evidence: [ssb_viewer.rs:2741](.elai_cc/crates/orchestrator/src/ssb_viewer.rs#L2741) and [commands.rs:1716](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L1716)
- B-13 BubbleUpStrip.svelte, OrbitShootingStarOverlay.svelte, captions.ts — evidence: [BubbleUpStrip.svelte](.elai_cc/crates/dashboard-v2/src/lib/orbit/BubbleUpStrip.svelte), [OrbitShootingStarOverlay.svelte](.elai_cc/crates/dashboard-v2/src/lib/orbit/OrbitShootingStarOverlay.svelte), [captions.ts:97](.elai_cc/crates/dashboard-v2/src/lib/orbit/captions.ts#L97)
- B-14 start_chat_stream / cancel_chat_stream commands + active_streams AbortHandle store — evidence: [commands.rs:3987](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L3987) and [state.rs:467](.elai_cc/crates/dashboard-v2/src-tauri/src/state.rs#L467)
- B-15 cli/src/main.rs decomposed (252 -> 255 LOC confirmed), live_cli.rs exists as separate module (4198 LOC) — evidence: [main.rs](.elai_cc/crates/cli/src/main.rs) (255 lines), [live_cli.rs](.elai_cc/crates/cli/src/live_cli.rs) (4198 lines)
- B-22 phase_4g_extended_contract.rs exists, derive_autonomous_evidence/AutoAdvanceStats/get_auto_advance_stats wired — evidence: [phase_4g_extended_contract.rs](.elai_cc/crates/contracts/tests/phase_4g_extended_contract.rs), [ssb_viewer.rs:3380](.elai_cc/crates/orchestrator/src/ssb_viewer.rs#L3380), [commands.rs:591](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L591)
- B-22 patch-4G.1 pendingResearchChip in commands.svelte.ts, acceptResearchChip/rejectResearchChip — evidence: [commands.svelte.ts:70](.elai_cc/crates/dashboard-v2/src/lib/state/commands.svelte.ts#L70) and [commands.svelte.ts:370](.elai_cc/crates/dashboard-v2/src/lib/state/commands.svelte.ts#L370)
- B-24 MediaValidationPanel.svelte, wire_media_validator_for_spec, SsbMediaSink — evidence: [MediaValidationPanel.svelte](.elai_cc/crates/dashboard-v2/src/lib/components/MediaValidationPanel.svelte), [media_validator_e2e_cloud_mode.rs:275](.elai_cc/crates/orchestrator/tests/media_validator_e2e_cloud_mode.rs#L275)
- B-24 media_validator test files (8 files in orchestrator/tests/) — evidence: confirmed via Glob
- B-26 NotificationManager.svelte, set_webhook_url, get_supervision_log — evidence: [NotificationManager.svelte:71](.elai_cc/crates/dashboard-v2/src/lib/notifications/NotificationManager.svelte#L71), [commands.rs:3328](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L3328), [commands.rs:1672](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L1672)
- B-36 (follow-up DONE) run_planning_task and run_research implemented in commands.rs (D.2/D.3) — evidence: [commands.rs:4237](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L4237) and [commands.rs:4354](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L4354)
- B-36 TauriWebResearchStepEmitter implemented — evidence: [commands.rs:4320](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L4320)
- B-36 set/get_sidecar_settings commands — evidence: [commands.rs:5021](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L5021) and [commands.rs:4999](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs#L4999)
- B-37 (PART VII plan) RuntimeAdapter trait implemented by 9+ adapters (AnthropicApiRuntime, ClaudeCodeRuntime, CodexCliRuntime, CopilotCliRuntime, GeminiCliRuntime, HermesRuntime, KiroCliRuntime, OllamaRuntime, OpenCodeRuntime) — evidence: [runtime_orchestrator.rs](.elai_cc/crates/orchestrator/src/runtime_orchestrator.rs), IMPL_TRAIT rows in elai.map
- B-37 PerRoleRuntime struct/default in per_role_runtime.rs — evidence: [per_role_runtime.rs:169](.elai_cc/crates/orchestrator/src/per_role_runtime.rs#L169)
- B-37 RuntimeOrchestrator in state.rs — evidence: [state.rs:488](.elai_cc/crates/dashboard-v2/src-tauri/src/state.rs#L488)
- B-39 chat_reflect.rs exists + chat_reflect_contract.rs — evidence: [chat_reflect.rs](.elai_cc/crates/reasoning/src/chat_reflect.rs), [chat_reflect_contract.rs](.elai_cc/crates/contracts/tests/chat_reflect_contract.rs)
- B-39C substitute path wired in live_cli.rs (~L1707) — evidence: [live_cli.rs:1685-1751 range](.elai_cc/crates/cli/src/live_cli.rs#L1685)
- Oracle-gate patch intent.ts, intent.test-cases.ts, acceptResearchChip/rejectResearchChip — evidence: [intent.ts:89](.elai_cc/crates/dashboard-v2/src/lib/utils/intent.ts#L89), [intent.test-cases.ts](.elai_cc/crates/dashboard-v2/src/lib/utils/intent.test-cases.ts)
- Net-new feature: remote_sidecar.rs exists + RemoteSidecarBootOverlay.svelte — evidence: [remote_sidecar.rs:87](.elai_cc/crates/cli/src/remote_sidecar.rs#L87), [RemoteSidecarBootOverlay.svelte:11](.elai_cc/crates/dashboard-v2/src/lib/screens/RemoteSidecarBootOverlay.svelte#L11)
- Auto-remote-sidecar poll_until_healthy implemented — evidence: [remote_sidecar.rs:217](.elai_cc/crates/cli/src/remote_sidecar.rs#L217)

## Claims PARTIAL

- SCHEMA_VERSION claim: INDEX says "SCHEMA_VERSION=26 confirmed" (B-6 session), but actual ssb.rs SCHEMA_VERSION = 46 at time of this report.
  - exists: [ssb.rs:184](.elai_cc/crates/orchestrator/src/ssb.rs#L184) — const SCHEMA_VERSION: i64 = 46
  - gap: INDEX is out of date — B-37, B-38, B-40, B-41, B-42, B-37C have bumped it to 46. INDEX records no status for these phases.

- B-7 status: INDEX first records B-7 PARTIAL, then updates to DONE after B-7b follow-up in session followup round.
  - exists: ConceptDecompositionNodePayload, DagRepairRound, BubbleUpTyped all confirmed
  - gap: ssb_viewer.decomposition_tree() stub is mentioned as pending but not explicitly verified complete; the PARTIAL->DONE flip is in the followup section and not in the canonical "Status of B-N items vs plan" list.

- B-15 status: INDEX records "PARTIAL — main.rs decomposition DONE 2026-05-08; other B-15 items remain pending."
  - exists: main.rs = 255 LOC (decomposition done); live_cli.rs, args.rs, auth.rs etc. all present
  - gap: HashMap ban, entropy leaks, dead-code strip, hooks.rs Windows fix explicitly noted as still pending. No code evidence these are done.

- B-36 status: INDEX followup says DONE but adds "(D.4 obscura needs web_search dep wired — currently subprocess fallback)".
  - exists: run_planning_task, run_research, set/get_sidecar_settings all confirmed in commands.rs
  - gap: obscura/web_search dep not verified wired; stub.ts still shows run_planning_task as a stub export meaning Svelte frontend may not call the Tauri command directly.

## Claims MISSING

- B-37, B-38, B-40, B-41, B-42 shipped phases are NOT recorded in the INDEX at all — searched: "B-40", "B-41", "B-42", "B-38", "SHIPPED"; the INDEX only describes B-37/B-38 as future planned work (PART VII). MEMORY.md confirms they shipped, code confirms SCHEMA_VERSION 46 and B-40/B-41/B-42 comments in ssb.rs, but the INDEX has no status block for them.

- B-5 per-agent budget wiring to emit_reasoning: INDEX says "per-agent budget wiring documented as TODO in emit_reasoning" — searched: "emit_reasoning" in elai.map. No evidence it was closed beyond a TODO comment.

- Oracle-gate "No commit yet" note: INDEX explicitly states "No commit yet — work staged for user review." Not verifiable from code state vs git, but the code symbols exist (intent.ts, pendingResearchChip, acceptResearchChip). Whether this was committed is UNVERIFIABLE from code alone.

## Claims UNVERIFIABLE

- Session audit reports and fixer reports (workflow/sessions/s-59964/FINAL-AUDIT.md, FINAL-FIXER-REPORT.md, etc.) — reason: workflow/ is gitignored; no code equivalent to verify against.
- Memory-bank changelog entries (changelog.md, gotchas.md, etc.) — reason: memory-bank/ is gitignored; cannot verify update claims.
- "Release builds: cargo build --release -p elai-cli PASS, cargo build --release -p elai-dashboard-v2 PASS" — reason: this is a runtime verification; cannot verify from static code analysis.
- "Live smoke vs friend's RTX 3090: 6/7 sidecar endpoints PASS" — reason: runtime test result, not a code artifact.
- B-9 dag-write-in-dashboard deferred — reason: SessionDag integration into AppState would need runtime test to confirm JSONL replacement; code presence of SessionDag is confirmed but replacement of JSONL is not statically verifiable.
- B-5 Task 3 partial (ContextCompiler/MediaValidator required_steps) — reason: agent_grc_config_for_role is confirmed in agent_pool.rs:837; partial status of required_steps fix not deterministically verifiable without running tests.
- Commit hashes (04f68b8, 6dd4af5, 6adb454, fd5c2e5) — reason: git log not checked; could verify but outside scope of static analysis.

## What must be done (action list)

- [ ] Update B-0_INDEX.md with status blocks for B-37 (DONE — SCHEMA_VERSION 46, RuntimeAdapter 9+ impls, commit fd5c2e5), B-38 (status unknown from INDEX), B-40 (DONE per MEMORY), B-41 (DONE per MEMORY), B-42 (DONE per MEMORY).
- [ ] Add current SCHEMA_VERSION (46) to the INDEX or replace "SCHEMA_VERSION=26 confirmed" with a pointer to ssb.rs.
- [ ] Resolve B-15 remaining items (HashMap ban, entropy leaks, hooks.rs Windows fix) — add explicit DONE or OPEN entries in INDEX.
- [ ] Verify B-5 emit_reasoning per-agent budget wiring is still a TODO or close it; update INDEX status.
- [ ] Confirm oracle-gate patch was committed (oracle-gate.1 has "No commit yet" note in INDEX); update status.
- [ ] Clarify B-36 obscura/web_search dep status — stub.ts run_planning_task still shows stub path, verify Tauri invoke is registered in invoke_handler!.

## Notes

The INDEX is a living document that was accurately updated through the oracle-gate patch (2026-05-11) but has not been updated since. The codebase has advanced by at least 4 more B-phases (B-37, B-38, B-40, B-41, B-42 per ssb.rs comments and MEMORY.md). The structural plan sections (Part I-VII descriptions) are aspirational roadmap text, not shipping claims, and are therefore not subject to DONE/MISSING classification. The overall architecture described (dashboard-v2, Tauri commands, SSB, PerRoleRuntime, chat REFLECT) is verified present in code.
