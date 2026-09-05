B-3 STUB COMMAND ERADICATION — DECISIONS DOC
Plan: docs/thefinalglance/B-plan/B-3_stub_command_eradication.md
Session: s-25120 (completion-drive recheck closure)
Date: 2026-05-22
Status: SHIPPED (executor pass, awaiting auditor)

1. SCOPE REBASELINE (V1 -> V2)
   The plan's original 30-item DEAD UI INVENTORY was built against the V1
   dashboard at crates/dashboard/. V2 reorganized the surface so the cited
   line numbers (commands.rs:4054 trigger_reindex, ActivityEntry.svelte,
   ssb_viewer.rs:2196 get_pagerank_top, Settings.svelte:112-117,
   ChatWindow.svelte:344-376, WindowRegistry.ts:27, intentRouter.ts:145-156)
   are all stale. Per the V2 REBASELINE preface plus the prior re-check
   report at workflow/sessions/s-25120/cross-check-recheck/B-3*-report.md,
   the items resolve as follows:

   Item     V1-claim                                                         V2-status
   ------   -----------------------------------------------------            -----------
   #1       task_runner.rs:2818 languages stub                              FIXED — task_runner.rs:5976 now passes summary.registry_status.loaded
   #2       ssb_viewer.rs:2196 get_pagerank_top                              SHIPPED on V2 — pagerank_top at ssb_viewer.rs:2915 (table-exists guard); Tauri wrapper at commands.rs:1764 registered at lib.rs:608
   #3       ssb_viewer.rs:2205 get_query_tier_stats                          FIXED — viewer method existed at ssb_viewer.rs:2981; added Tauri wrapper + invoke_handler registration this session
   #4       ssb_viewer.rs:2219 get_freshness_state                           SHIPPED on V2 — wrapper at commands.rs:1764, viewer at ssb_viewer.rs:3067
   #5       cli/main.rs:3721 NoopSsbNotifier replacement                     PARTIAL — production reranker.rs has zero NoopSsbNotifier refs (C-1 fix); cascade.evaluate has no production caller (B-4 scope); added SsbDegradationNotifier production adapter in live_cli.rs for B-4 to wire
   #6       dep_audit_completed SSB action_type                              FIXED — added SsbActionType::DepAuditCompleted + payload (SCHEMA_VERSION 98)
   #7       baseline_run_completed SSB action_type                           FIXED — added SsbActionType::BaselineRunCompleted + payload
   #8       deterministic_audit_run SSB action_type                          FIXED — added SsbActionType::DeterministicAuditRun + payload
   #9       emit_reasoning GRC budgets                                       SHIPPED on V2 — pipeline_grc threaded through emit_reasoning per C-1
   #10      "reasoning · N topics" label                                    SHIPPED on V2 — depends on #9
   #11      RulesDrawer 1s polling + rule_transition feed                    FIXED (revised) — 10s polling preserved (1s would be a regression per integration-explorer drift note); added rulesData.lastTransitions $state + get_rule_transitions Tauri command + SsbActionType::RuleTransition variant + "LAST 5 RULE TRANSITIONS" RulesDrawer section
   #12      PlanWindow registered                                            SHIPPED on V2 — WindowHost.svelte:7,64 mounts PlanWindow
   #13-#15  get_decomposition_tree / get_phase_detail / get_baby_step_detail SHIPPED on V2 — commands.rs:3804/11021/11045
   #16      ts-rs binding generation pipeline                                CLASS-C REFRAME — workspace policy is hand-authored TS mirrors (empire/types.rs:11 documents canonical decision); added SessionHandoffEntry interface in types.ts mirroring Rust shape {seq, ts_ms, winner, abandoned, trunk_snap_id, from_snap_count}; ts-rs dep NOT added per "no new dependencies without confirmation"
   #17      Reindex button                                                   SHIPPED on V2 — reindex_now at commands.rs:6752 (renamed from V1 trigger_reindex); contract test at dashboard_tier4_settings_contract.rs:36 already migrated to reindex_now
   #18      branch_merge case                                                SHIPPED on V2 — session_dag.rs:1327 emits SsbActionType::BranchMerged (supersede pattern); ssb_viewer queries both BranchMerge + BranchMerged at ssb_viewer.rs:3137-3138
   #19      isolation_metric renderer                                        FIXED — V2 has no ActivityEntry.svelte (CLASS B met by V2 rebaseline); added explicit get_isolation_metric_status() defer-flag in stub.ts citing B-18 as the producer phase
   #20      /continue handler                                                SHIPPED on V2 — CommandWindow.svelte:269 invokes load_handoff
   #21-#24  Status bar FSM badge / state colour / EXECUTING pulse / Monitor  SHIPPED on V2 — depend on B-2; component-level wires present
   #25      /session fork                                                    SHIPPED on V2 — CommandPalette.svelte:97-110 + fork_session_branch command at commands.rs:2830
   #26      merge/close branch operations                                    PARTIAL — palette entries exist; underlying Tauri commands (merge_session_branch, close_session_branch) registered at lib.rs:706-707; B-9 SessionDag activation owns the deep producer wire
   #27      AutoAdvanceStats feature-gate                                    FIXED — added `autonomous-mode` cargo feature; #[cfg(feature="autonomous-mode")] gates real get_auto_advance_stats body; non-feature build returns zeroed shape; frontend feature_flags.svelte hides the metric row entirely
   #28      ProposalsPanel feature-gate                                      FIXED — added `self-modification` cargo feature; #[cfg]-gates real get_pending_proposals/get_applied_proposals bodies; MonitorWindow conditionally mounts <ProposalsPanel /> on featureFlags.self_modification
   #29      MONITOR window                                                   SHIPPED on V2 — MonitorWindow.svelte present + mounted via AppShell routing
   #30      EXPLORE window                                                   SHIPPED on V2 — ExploreWindow.svelte present

2. KEY DECISIONS

   DECISION-A: ts-rs is NOT added.
   Rationale: orchestrator/src/empire/types.rs:11 documents that the workspace
   intentionally does NOT use ts-rs / tauri-specta. Cross-language types are
   hand-authored mirrors in dashboard-v2/src/lib/api/types.ts. Adding ts-rs
   would violate the "no new dependencies without confirmation" rule + the
   established workspace pattern. The SessionHandoffEntry schema mismatch
   (item #16) is fixed by hand-authoring the corrected interface in types.ts.

   DECISION-B: 1s polling for RulesDrawer is DECLINED.
   Rationale: integration-explorer noted that RulesDrawer.svelte already
   polls at 5s (FSM) + 10s (rulesData) via the makeRefCounted ref-counted
   timer at state/drawers.svelte.ts:70-87. Tightening to 1s would be a
   measurable performance regression on a panel that is rarely the user's
   focus. The plan task collapses to "add rule_transition SSB feed + show
   last 5 inline" — implemented as a 6th drawer section reading
   rulesData.lastTransitions on the same 10s tick.

   DECISION-C: SsbActionType::RuleTransition is a sibling-stream of the
   persistent `rule_transitions` SQLite table, not a replacement.
   Rationale: the rules crate already persists transitions to
   rule_transitions (schema.rs:101, lifecycle.rs:188) for the rules engine
   itself. The SSB row is a thin mirror so the dashboard can subscribe to
   the live stream without cross-database polling. The producer wire
   (rules::lifecycle::write_transition_log emitting an SsbPayload row
   alongside the SQLite insert) is left to the rules crate to land when
   the lifecycle module gains an SSB writer handle.

   DECISION-D: knowledge_degradation production wire is a B-4 deferral.
   Rationale: cascade.evaluate(..., &dyn SsbNotifier) has no production
   caller today — startup_readiness_check only invokes cascade.set_mode()
   (no notifier). The production NoopSsbNotifier ban was already met by
   C-1's reranker.rs refactor (test-only impls behind #[cfg(any(test,
   feature="test-helpers"))]). B-3's contribution is the production
   adapter SsbDegradationNotifier in live_cli.rs (mirroring the
   SsbRerankerNotifier pattern) so B-4 has a real constructor to wire
   into the cascade-evaluate loop when it lands.

   DECISION-E: ELAI_DASHBOARD_OFFLINE is wired both runtime + compile-time.
   Rationale: V2 rebaseline preface prescribed an env-var gate; zero
   matches in code before this session. Wire chosen:
     - `get_feature_flags()` Tauri command reads
       std::env::var("ELAI_DASHBOARD_OFFLINE")` and surfaces it in the
       returned shape.
     - stub.ts's tryInvoke() respects either window.__ELAI_DASHBOARD_OFFLINE__
       (set on bootstrap via featureFlags hydration) OR
       import.meta.env.VITE_ELAI_DASHBOARD_OFFLINE (Vite compile-time).
     - The `get_feature_flags` command itself is exempt from the short-
       circuit so the shell can always hydrate the runtime signal.

3. CROSS-LANGUAGE WIRING TOUCHED

   Rust (.elai_cc/crates/):
     orchestrator/src/ssb.rs                     +4 SsbActionType variants (SCHEMA_VERSION 97 -> 98), +4 Payload structs, SQL allowlist, as_sql_str, from_str, action_type, all_test_payloads
     orchestrator/src/task_runner.rs:5976        replaced Vec::new() languages stub with summary.registry_status.loaded
     dashboard-v2/src-tauri/src/commands.rs      +get_query_tier_stats, +get_rule_transitions, +get_feature_flags, dual-cfg get_auto_advance_stats + get_pending_proposals + get_applied_proposals
     dashboard-v2/src-tauri/src/lib.rs           register 3 new commands in invoke_handler!
     dashboard-v2/src-tauri/Cargo.toml           +autonomous-mode +self-modification cargo features
     cli/src/live_cli.rs                         +SsbDegradationNotifier production adapter (mirroring SsbRerankerNotifier C-1 pattern)

   Frontend (.elai_cc/crates/dashboard-v2/src/lib/):
     api/types.ts                                +FeatureFlags +QueryTierStats +RuleTransitionRow +SessionHandoffEntry
     api/stub.ts                                 ELAI_DASHBOARD_OFFLINE handling in tryInvoke; +get_rule_transitions +get_query_tier_stats +get_feature_flags +get_isolation_metric_status
     state/feature_flags.svelte.ts               NEW — bootstrap-once feature-flag store
     state/drawers.svelte.ts                     +lastTransitions $state in createRules() + 5-row poll
     drawers/RulesDrawer.svelte                  +LAST 5 RULE TRANSITIONS section with transitionTone color map
     windows/MonitorWindow.svelte                imports featureFlags, hydrates on $effect mount, gates <ProposalsPanel /> + the AutoAdvance metric row with deferred-state fallback

4. NO COMMIT — auditor reads first.
   Per CLAUDE.md rule + .claude/rules/commit-cadence.md HARD RULE,
   commits happen only after the full ticket passes audit. Memory-bank
   writes (memory-writer agent) follow audit.

5. RESIDUAL DEFERRALS (documented, intentional)
   - B-4 owns: cascade.evaluate(&SsbDegradationNotifier) periodic-loop wire-up;
     emission of BaselineRunCompleted payloads from the baseline harness;
     producer for QueryTierStats (per-query tier decisions).
   - B-9 owns: SessionDag full BranchMerged production producer (#26
     merge/close branch ops reachable end-to-end).
   - B-15 owns: DeterministicAuditRun payload producer in
     deterministic_audit_gate.
   - B-18 owns: isolation_metric Tier-C IPC interception producer +
     orbit caption renderer.
   - B-22 owns: real autonomous-mode producer (toggles
     cfg(feature="autonomous-mode") into a meaningful runtime).
   - B-23 owns: DepAuditCompleted payload producer + self-modification
     producer (toggles cfg(feature="self-modification") into a
     meaningful runtime).
   - rules crate owns: SsbPayload::RuleTransition emission alongside
     existing write_transition_log INSERT.
