# DEEP-C-002 — exhaustive feature and behavior discovery

## Scope, evidence model, and method

This ledger is a source-faithful deep read of exactly the ten originals assigned to `DEEP-C-002`. `03-C-SUMMARY_INDEX.discovery.md` was used only to orient the comparison in **Summary additions and corrections**; it is not treated as implementation evidence. Physical line anchors use the form `[C-1:L56-L72]` and refer to the verified files in the manifest table below.

Evidence classes are deliberately separate:

- **E (explicit):** directly required, described, prohibited, accepted, deferred, or reported as shipped by a source.
- **I (inferred):** a consequence or integration implication not directly promised by a source.
- **A (ambiguous):** underspecified enough that more than one implementation can satisfy the prose.
- **C (conflicting):** two explicit clauses cannot both be the current source of truth without ordering, amendment, or reconciliation.

An **atomic record** is one independently checkable requirement or source assertion. Repeated appearances of the same requirement in Tasks, `WIRING_CONTRACT`, deterministic checks, acceptance, and file lists are merged into one record with all useful anchors. Closed identifier sets and field schemas remain together when splitting them would destroy the stated atomic schema/event-family invariant.

## Exact processed manifest and mechanical verification

Manifest section: `DEEP-SOURCE-MANIFEST.md` lines 369-384, heading `### DEEP-C-002`. The section declares family C, 10 files, 2,552 logical lines, 135,174 bytes, within the 1,800-2,800-line target. Every value below was independently recomputed with `wc -l -c` and `sha256sum`; all match.

| # | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---:|---|---:|---:|---|---|---|
| 1 | numbered/index original | 288 | 15,390 | `61485f3c2bb18d40ff70effd99c6e7ce2b26b389c4cfc5e084f9922808579efd` | `docs/thefinalglance/C-plan/C-1_ssb_emission_knowledge_reasoning.md` | exact |
| 2 | numbered/index original | 298 | 16,413 | `1efaf96308cd77b7e490167a6a0b87dde71cba2c98edea0ee4b11ba26368f6b1` | `docs/thefinalglance/C-plan/C-4_tauri_commands_orchestration_cloud.md` | exact |
| 3 | numbered/index original | 345 | 18,521 | `611982c5119fa992ebe5b757b0eb088cde483a6a69b9f3dd825b4d43d4f3d893` | `docs/thefinalglance/C-plan/C-5_cloud_mode_hardening.md` | exact |
| 4 | numbered/index original | 256 | 13,443 | `fb080a4d27fd46d49ef888411df890ef3d066773d1249a0e52008acc07efdc1c` | `docs/thefinalglance/C-plan/C-12_per_level_auditor_navigator_wiring.md` | exact |
| 5 | numbered/index original | 328 | 19,927 | `020c243e16a9aa80fbe67f1a293376ee6d6ab4851767df416adbd5e020a7ee6d` | `docs/thefinalglance/C-plan/C-19_test_infra_backfill_adversarial.md` | exact |
| 6 | numbered/index original | 316 | 17,598 | `4e5fb5d72a96926c4ed261632d01dbc0a179c467fd30ab37aef911cf39b73d31` | `docs/thefinalglance/C-plan/C-27_testing_trust_t0_wiring_contract.md` | exact |
| 7 | numbered/index original | 219 | 9,793 | `a59524220b15f09e0213a359f79ccbe7c64550261cf240281ee8debcb14b1ea8` | `docs/thefinalglance/C-plan/C-42_semantic_gateway_zero_trust_fuzzing.md` | exact |
| 8 | numbered/index original | 203 | 9,957 | `e07ece1e15c5dece851eee4cdb95575e6052fa80ebb5bc0bdbf0814db546cb12` | `docs/thefinalglance/C-plan/C-43_phase_4h_scope_freeze_ratification.md` | exact |
| 9 | numbered/index original | 142 | 6,626 | `605bb7df86670c939852dd2fb4626632c91065b360f05c9d839061813b3f8763` | `docs/thefinalglance/C-plan/C-50_contract_coding_test_scaffolding.md` | exact |
| 10 | numbered/index original | 157 | 7,506 | `275a37cc2eb5cd37a919e43163c26ec7717d9a6806523fa02c044f9a73ef230d` | `docs/thefinalglance/C-plan/C-57_openhands_index_swe_cycle_swe_atlas.md` | exact |
| **Total** |  | **2,552** | **135,174** |  | **10 files** | **exact** |

## Explicit per-file feature ledgers

### C-1 — SSB Emission: Knowledge + Reasoning Producers

- **C1-E01 — [status/capability]** Objective: a live cloud-Sonnet task must persist real, non-null SSB payload rows for exactly seven action names: `grc_step_emitted`, `grounding_sweep_result`, `csq_evaluated`, `auditor_rubber_stamp_rejected`, `hyde_query`, `vector_cache_hit_rate`, and `query_trace`; estimated effort is one 6-8-hour session. [C-1:L1-L7]
- **C1-E02 — [dependencies/concurrency]** Prerequisites are shipped B-4, B-5, and B-8; the work is declared parallel-safe with C-2, C-3, and C-4 because the latter are described as dashboard-side and not changing `ssb.rs`/`task_runner.rs` emit logic. [C-1:L8-L13]
- **C1-E03 — [baseline/status]** The source records the starting gaps: five reasoning variants absent; `emit_reasoning` uses synthetic `agent_id="task_runner"` and zero budget/elapsed values; HyDE/cache events do not exist; `pipeline.rs` only traces `query_trace`; the reranker defaults to `NoopSsbNotifier`; CSQ and rubber-stamp paths do not write SSB; three dashboard commands/flags are stubs. [C-1:L15-L37]
- **C1-E04 — [schema/lifecycle]** Add `SsbActionType::{GrcStepEmitted,GroundingSweepResult,CsqEvaluated,AuditorRubberStampRejected,HydeQuery,VectorCacheHitRate,QueryTrace}`, all `as_sql_str`/`from_str` arms, payload arms, SQL serialization, and exactly one atomic `SCHEMA_VERSION` increment for the family. [C-1:L56-L72; C-1:L210-L222]
- **C1-E05 — [schema]** `GrcStepEmittedPayload` fields are `agent_id`, `topic`, `step_type`, `grounding_level`, `confidence:f64`, `budget_remaining:u32`, and `elapsed_ms:u64`. [C-1:L61-L64]
- **C1-E06 — [schema]** `GroundingSweepResultPayload` fields are `ungrounded:u32`, `verify_needed:u32`, `grounded:u32`, and `top_claim:Option<String>`. [C-1:L63-L65]
- **C1-E07 — [schema]** `CsqEvaluatedPayload` fields are `severity:String`, `guard_present:bool`, `blocked:bool`, and `check_count:u32`. [C-1:L65-L66]
- **C1-E08 — [schema]** `AuditorRubberStampRejectedPayload` fields are `role:String`, `finding_count:u32`, and `expected_min:u32`. [C-1:L66-L67]
- **C1-E09 — [schema/privacy]** `HydeQueryPayload` fields are `query_hash:String`, `hypothesis_len:u32`, `cache_hit:bool`, and `latency_ms:u64`; the query itself is represented by `blake3_hex(query)`, not raw text. [C-1:L67-L68; C-1:L93-L96]
- **C1-E10 — [schema]** `VectorCacheHitRatePayload` fields are `session_id:String`, `hits:u32`, `misses:u32`, and `hit_rate_pct:f64`. [C-1:L69-L70]
- **C1-E11 — [schema]** `QueryTracePayload` fields are `query_id:String`, `tier:String`, `stage_count:u32`, `result_count:u32`, and `total_latency_ms:u64`. [C-1:L70-L72]
- **C1-E12 — [workflow/integration]** Refactor `emit_reasoning` to accept `grc: Option<&Arc<AsyncMutex<PipelineGrc>>>`; when present, extract real budget, agent, topic, and elapsed time, emit `GrcStepEmitted` in addition to the existing `ReasoningTrace`, and update all 11 call sites to pass `resolved.pipeline_grc.as_ref()`. [C-1:L74-L78]
- **C1-E13 — [event/workflow]** At the former `TODO(B-future)` around `task_runner.rs:5119`, emit `CsqEvaluated` from `severity`, `guard_present`, `hook_pre_blocked`, and `check_count` via `writer.write_payload()`. [C-1:L80-L82]
- **C1-E14 — [event/workflow]** Immediately after `ground_plan`, emit `GroundingSweepResult` populated from `ungrounded.len()`, `grounded.len()`, `verify.len()`, and `ungrounded.first().cloned()`. [C-1:L84-L86]
- **C1-E15 — [event/failure]** In the `GrcError::InsufficientFindings` branch, emit `AuditorRubberStampRejected`; inject an optional real SSB writer into `BasicGrcEngine::advance`, preferring an existing side-channel/channel if available. [C-1:L88-L91]
- **C1-E16 — [event/observability]** `HydeStage::run` records cache hit and latency and emits one `HydeQuery` via an injected `Arc<dyn SsbNotifier>`; the contract requires the second identical query to report `cache_hit=true`. [C-1:L93-L96; C-1:L265-L267]
- **C1-E17 — [data/lifecycle]** `VectorCache` gains hit/miss counters and `flush_hit_rate_ssb(session_id, writer)`, which emits the rate and resets counters; `QueryPipeline::run_query` invokes it after query completion, and tests accept `0.01` error from `hits/(hits+misses)*100`. [C-1:L98-L101; C-1:L268-L270]
- **C1-E18 — [event/integration]** Preserve `tracing::debug` and add a `QueryTrace` SSB write after it; `QueryPipeline` gains optional notifier state threaded from the live CLI construction path. Its contract expects exactly one trace per minimal query, including nonnegative result count and latency. [C-1:L103-L107; C-1:L271-L273]
- **C1-E19 — [integration/security]** Replace the production `NoopSsbNotifier` default in `CrossEncoderReranker::new` with the real live-CLI SSB handle; test-only/no-comment occurrences are the only tolerated remnants. [C-1:L109-L110; C-1:L150-L152; C-1:L197-L199]
- **C1-E20 — [surface/data]** `get_grounding_sweep` must read real `GroundingSweepResult` rows through `SsbViewer`, remove hardcoded zero output, and set `unwired.groundingSweep=false`. [C-1:L112-L114; C-1:L201-L204]
- **C1-E21 — [surface/data]** `get_query_trace_recent` must return real `query_trace` rows rather than `json!([])` and set `unwired.recentTraces=false`. [C-1:L116-L117; C-1:L193-L195]
- **C1-E22 — [surface/data]** `get_csq_pending` must return `csq_evaluated` rows filtered to `blocked=true` and set `unwired.csqPending=false`. [C-1:L119-L120]
- **C1-E23 — [documentation/evidence]** `workflow/reasoning/C-1-decisions.md` records the notifier injection choice for HyDE/cache, the engine writer path, and final schema version. [C-1:L122-L124]
- **C1-E24 — [reachability/observability]** The end-to-end entrypoint is `elai /task --mode cloud`; an SQL query over `.elai/sessions/{id}/ssb.db` must show at least five distinct new types, at least one row per returned type, and non-null payloads. `grc_step_emitted` and `query_trace` are mandatory every run; rubber-stamp rejection is conditional. [C-1:L148-L161; C-1:L228-L254]
- **C1-E25 — [tests/done gate]** Deterministic gates require ≥21 enum/string hits, one `emit_reasoning` GRC signature, CSQ and grounding emits, engine/HyDE/query-trace writes, no production `NoopSsbNotifier`, no zero grounding stub, clean workspace check, and schema ≥47. [C-1:L164-L213]
- **C1-E26 — [tests/done gate]** Required test changes are five new contract files (`ssb_new_variants_contract.rs`, `emit_reasoning_grc_contract.rs`, `hyde_ssb_emit_contract.rs`, `vector_cache_hit_rate_contract.rs`, `query_trace_ssb_contract.rs`) plus extensions to `reasoning/tests/agent_config.rs` and `production_agent_rpc_contract.rs`; package tests for orchestrator, knowledge, and reasoning and the workspace check must pass. [C-1:L215-L222; C-1:L257-L278]
- **C1-E27 — [surface/done gate]** UI acceptance requires a non-`task_runner` agent in ReasoningDrawer, at least one recent trace in KnowledgeDrawer, and all three relevant `unwired` flags false. [C-1:L223-L227]
- **C1-E28 — [constraints/non-goals]** Explicitly out of scope: C-2 HyDE preview and `CsqBlockCard`; C-3 PageRank/freshness/canary commands; C-4 recall metrics, DEEP-vs-STANDARD MRR, and stage latency; major replanning on blocked feasibility; `verify_claim` prefetch wiring; and cross-encoder TOML documentation. [C-1:L280-L288]

### C-4 — Tauri Commands: Orchestration + Cloud Surface Completion

- **C4-E01 — [status/capability]** Objective: expose existing orchestration and cloud-spend state through real Tauri/dashboard surfaces, remove hardcoded stubs, and add contract tests; estimated effort is one session. [C-4:L1-L6]
- **C4-E02 — [dependencies/concurrency]** C-1 is said to provide agent-lifecycle emitters and C-2 BabyStepSpec persistence; the work is parallel-safe with C-3 due to non-overlapping commands. [C-4:L7-L10]
- **C4-E03 — [baseline/status]** Recorded gaps include an eight-role always-IDLE `get_agents`, missing V2 phase/baby-step commands, an empty executable pipeline, absent research history, four implemented-but-unregistered viewer methods, no project list/create service, three frontend runtime stubs despite real backend commands, and zero-valued cloud spend. [C-4:L12-L54]
- **C4-E04 — [data/schema]** Add `SsbViewer::recent_agent_lifecycle_rows(limit:u32) -> Vec<AgentLifecyclePayload>` using `SELECT action_type, payload FROM ssb WHERE action_type='agent_lifecycle' ORDER BY seq DESC LIMIT ?`; empty storage returns an empty vector. [C-4:L75-L79; C-4:L151-L154]
- **C4-E05 — [workflow/fallback]** `get_agents` reads the latest 200 lifecycle rows, groups by role, takes maximum sequence per role, maps state and payload PID/heartbeat, and falls back to IDLE only for roles with no SSB row before any task. [C-4:L81-L87]
- **C4-E06 — [surface/failure]** Add and register `get_phase_detail(phase_id:String)`, forwarding to the real viewer; unknown IDs return structured `PhaseNotFound`, not panic. [C-4:L89-L92; C-4:L247-L248]
- **C4-E07 — [surface/failure]** Add/register `get_baby_step_detail(baby_step_id:String)` and replace unconditional NotFound by querying `baby_step_ready` or `concept_decomposition_node` by ID. If C-2 is absent, return `BabyStepNotPersisted`, not panic or stub success. [C-4:L94-L101; C-4:L249-L250]
- **C4-E08 — [surface/data]** Replace `get_executable_pipeline(session_id:String)` empty JSON with ordered phase → step → baby-step leaves reshaped from `decomposition_tree`/`concept_decomposition_node`; no rows produce an empty vector, not panic. [C-4:L103-L108; C-4:L251-L252]
- **C4-E09 — [surface/fallback]** Add/register `get_research_node_history(session_id:String,node_id:String)`, filter `research_node_stage`, deserialize `ResearchNodeHistoryEntry` matching the TS shape, and return an empty vector when B-7's emitter is absent. [C-4:L110-L115; C-4:L253-L254]
- **C4-E10 — [surface/integration]** Expose/register thin wrappers for `get_phase_progress`, `get_recent_actions`, `get_file_changes`, and `get_action_detail`; export the latter from `stub.ts`. [C-4:L117-L122]
- **C4-E11 — [surface/data/fallback]** `list_projects` parses `<project_root>/projects.json` into `ProjectEntry {name,path,last_opened_ms}`; an absent file returns an empty vector. [C-4:L124-L127]
- **C4-E12 — [surface/lifecycle/idempotence]** `create_project(name,parent_path)` creates `parent_path/name`, writes a stub `elai_cc.toml`, appends to `projects.json`, and uses temp-file-plus-rename atomic replacement. [C-4:L127-L130]
- **C4-E13 — [surface/integration]** Replace frontend implementations of `list_runtimes`, `test_runtime`, and `set_role_runtime` with `invoke(...)` calls using `{backend}` / `{role,backend}` payloads; mount `RuntimePanel.svelte` in Settings if absent. [C-4:L132-L138]
- **C4-E14 — [data/schema]** Add `SsbViewer::cloud_spend_summary()`: latest `cloud_spend.cost_usd` as task spend, today's summed cost as day spend, today's cache-creation/read token totals, and redaction count from `RedactionStat` when defined or zero otherwise. `get_cloud_spend` must use AppState/viewer; empty storage yields zero totals. [C-4:L140-L149; C-4:L154-L155]
- **C4-E15 — [capabilities/surfaces]** The exact ten newly exposed Tauri commands are `get_phase_detail`, `get_baby_step_detail`, `get_executable_pipeline`, `get_research_node_history`, `get_phase_progress`, `get_recent_actions`, `get_file_changes`, `get_action_detail`, `list_projects`, and `create_project`; all must be in `invoke_handler`. [C-4:L161-L170; C-4:L220-L226]
- **C4-E16 — [failure/stub bans]** `get_agents` cannot return a hardcoded literal; `get_cloud_spend` cannot return hardcoded zero fields; `get_executable_pipeline` cannot return `Ok(json!([]))`; and spend aggregation must query SSB rather than `Default::default()`. [C-4:L186-L190]
- **C4-E17 — [observability]** Cloud todo-CRUD smoke requires: ≥1 non-IDLE agent during execution, ≥2 decomposition phases, positive task spend after the Sonnet call, `ToolUse` in recent actions, and `src/todo.rs`/`FileWrite` in file changes. [C-4:L195-L201]
- **C4-E18 — [tests/done gate]** `c4_tauri_commands_contract.rs` covers six empty/absent-state behaviors: lifecycle empty, spend zeros, phase error, baby-step error, pipeline empty, and research history empty; the scoped contract test and clean workspace check must pass. [C-4:L151-L159; C-4:L203-L239]
- **C4-E19 — [acceptance/lifecycle]** Acceptance additionally requires known phase/baby-step data after producers exist, project directory/index persistence, live runtime panel data, nonzero MonitorWindow gauge, frontend-callability of all wrappers, and the five manual smoke assertions. [C-4:L241-L273]
- **C4-E20 — [files/evidence]** Writes span `ssb_viewer.rs`, dashboard commands/registration/API/Settings, one new contract file, and `workflow/reasoning/C-4-decisions.md`. [C-4:L275-L293]

### C-5 — Cloud Mode Hardening

- **C5-E01 — [status]** The post-rebaseline status is explicitly **SHIPPED** across two sessions, with all ten task groups landed. [C-5:L14-L15]
- **C5-E02 — [schema/status]** Shipped schema state is `SCHEMA_VERSION=103`, an atomic three-variant bump from 102 for `RouteBlocked`, `WarnLimitReached`, and `CostEstimate`; recorded bijection is 242→245 and `action_type_lock TOTAL_VARIANTS=306`. [C-5:L16-L18; C-5:L28-L30]
- **C5-E03 — [config/status]** The shipped CLI selector is `--provider local|cloud|hybrid`, not `--mode`; `--mode` is a separate gate-mode axis. `resolve_backend_mode` gives the CLI override precedence over TOML `[cloud].default_mode`, and absent CLI input uses that default. [C-5:L19-L22; C-5:L110-L119]
- **C5-E04 — [config/workflow]** Provider mappings are `cloud→BackendMode::Api`, `hybrid|auto→BackendMode::Hybrid`, and `local→BackendMode::Local`. [C-5:L113-L116]
- **C5-E05 — [surface/status]** The shipped component path is `dashboard-v2/src/lib/screens/CloudPanel.svelte`, not `lib/components`, mounted under Settings' Cloud section. [C-5:L23-L25; C-5:L165-L188]
- **C5-E06 — [event/status]** Redaction is the distinct `SsbActionType::RedactionApplied`, not a `REDACTION_APPLIED` command subtype; `RedactingBabyStepExecutor` writes those rows. [C-5:L26-L27; C-5:L182-L184]
- **C5-E07 — [surface/status/deferred]** `get_cloud_route_status`, `CloudRouteStatus`, and three CloudPanel banners shipped. A dedicated last-five `RedactionApplied` timeline is explicitly deferred pending `get_recent_redactions` or a spend-summary extension. [C-5:L31-L37; C-5:L176-L186]
- **C5-E08 — [documentation/status]** `workflow/reasoning/C-5-decisions.md` records Q1-Q6, DOUBLE-EMIT Option A, the `UNGROUNDED_HEURISTIC` rationale, and drift corrections. [C-5:L38-L40]
- **C5-E09 — [dependencies/concurrency]** C-5 closes B-27 gaps, depends on C-1 and C-4 live SSB/spend infrastructure, is parallel-safe with C-6/C-7, and adopts their design tokens when available. Patch-R3 ToolUse translation is non-blocking; text-only smoke tasks are the fallback if absent. [C-5:L1-L12; C-5:L64-L66]
- **C5-E10 — [failure/security]** `CloudModeRouter::route_cloud` reads accumulated live spend before routing. At `spent_day_usd >= spend_cap_daily_usd` it returns typed `DailySpendCapExceeded`; the analogous monthly comparison returns `MonthlySpendCapExceeded`; a `route_blocked` row records the refusal. [C-5:L90-L99]
- **C5-E11 — [config/default/idempotence]** `CloudConfig.warn_at_percent:Option<u8>` defaults to `Some(80)`. Warning evaluation occurs before the hard-cap check and may emit only once per calendar day when the percentage threshold is reached. [C-5:L101-L108]
- **C5-E12 — [failure/fallback]** If the live spend read errors, the router logs a warning and skips the cap check rather than blocking all cloud calls. [C-5:L230-L233]
- **C5-E13 — [retry/recovery]** In hybrid/auto mode, exhaustion of local baby-step retries emits `EscalateToCloud` and reruns once through `ExecMode::Cloud`; a second exhaustion halts and cannot escalate again. [C-5:L120-L126; C-5:L195-L198]
- **C5-E14 — [config/schema]** Exact sizing constants are `BABY_STEP_LOC_LOCAL_3B=25`, `BABY_STEP_LOC_LOCAL_7B=40`, `BABY_STEP_LOC_CLOUD_SONNET=300`, and `BABY_STEP_LOC_CLOUD_OPUS=500`. [C-5:L128-L133]
- **C5-E15 — [workflow]** `baby_step_loc_for_mode(mode,model)->u16` dispatches by execution mode plus the substrings `3b`, `7b`, `sonnet`, and `opus`, and feeds PipelineBuilder's step-size selection. [C-5:L134-L136]
- **C5-E16 — [cost/data]** `estimate_task_cost_usd(plan_skeleton,model)->(low,high)` uses `step_count * avg_tokens * price_per_token`, with `avg_tokens=baby_step_loc_for_mode*4.0`; the token/LOC rule is explicitly labeled `UNGROUNDED_HEURISTIC`. [C-5:L138-L144]
- **C5-E17 — [workflow/surface]** Cost estimation occurs before the first cloud call. The autonomous path does not block; the interactive path surfaces the estimate to CloudPanel. The shipped status says the event is the atomic `CostEstimate` variant. [C-5:L145-L148; C-5:L28-L31]
- **C5-E18 — [config/security/privacy]** `CloudConfig.custom_endpoint:Option<String>` overrides the Anthropic base URL; TOML wires `[cloud] custom_endpoint`. It is rejected in air-gapped mode, first use logs an info message, and the endpoint value must never enter SSB to avoid internal-URL leakage. [C-5:L150-L156; C-5:L234-L235]
- **C5-E19 — [schema/event]** `RouteBlockedPayload` is `{reason:String, spent_usd:f64, cap_usd:f64, cap_period:String}` and the action SQL string is `route_blocked`; it participates in the action-type bijection test. [C-5:L158-L163]
- **C5-E20 — [surface/state]** CloudPanel spend UI shows daily spend/cap, warn-at-80 and blocked badges, and a monthly-cap row when configured. [C-5:L168-L172]
- **C5-E21 — [surface/permissions]** The provider selector is a local/hybrid/cloud segmented control calling `set_execution_mode`; cloud selection is disabled when `airGapped=true`. [C-5:L173-L174]
- **C5-E22 — [surface/privacy]** CloudPanel shows today's aggregate redaction count; the detailed recent-event feed remains deferred. [C-5:L175-L186]
- **C5-E23 — [surface/state]** Banner precedence is explicit: `RouteBlocked` is latest-row driven; `WarnLimitReached` is hidden when a route-block supersedes it; `CostEstimate` displays the heuristic disclaimer inline. [C-5:L176-L181]
- **C5-E24 — [tests]** `cloud_ceiling_contract.rs` checks cap error, 80% warning, and `RouteBlocked` round-trip; `hybrid_escalation_contract.rs` checks escalation emission, cloud success, and one-retry halt; `cloud_loc_constants_contract.rs` checks all values and four dispatch combinations. [C-5:L190-L202]
- **C5-E25 — [observability]** The primary observable sets daily cap `$0.05`, triggers at `$0.04`, expects one warning then route block at cap, and expects the gauge/mode selector to reflect live state. [C-5:L237-L242]
- **C5-E26 — [done gates]** Deterministic checks cover live cap errors, event registration, constants, config fields, `--provider`, corrected component path, endpoint support, all three contract suites, and an orchestrator hybrid-integration test. [C-5:L244-L284]
- **C5-E27 — [acceptance]** Acceptance enumerates daily/monthly errors, route-block and warning evidence, provider bridge, one escalation, constants/helper, bounded cost estimate, endpoint guard, panel mounting, tests, three smoke scenarios, and clean handling rather than panic. [C-5:L286-L318]
- **C5-E28 — [files/status]** Production changes cover cloud router, SSB schema, task runner, CLI/config, dashboard, and contracts; however, the later Files list still contains pre-amendment names/paths and is not authoritative over the status block. [C-5:L320-L339]

### C-12 — Per-Level Auditor Navigator Wiring

- **C12-E01 — [capability]** Objective: connect the existing per-level dispatcher at five navigator tiers, ensure six mode-specific prompt blocks, pass `FramedIdea` to Concepts, emit started/verdict/cascade rows, and make AuditTimeline consume live verdicts. [C-12:L1-L10]
- **C12-E02 — [dependencies/status]** Requires C-1/C-2 and shipped B-41 types/scaffold; is parallel-safe with C-11, C-13, C-14, and C-15. B-41 is described as MOSTLY_DONE with navigator integration as the remaining gap. [C-12:L11-L24]
- **C12-E03 — [workflow/state]** Five completion mappings are explicit: concept phases resolved→`Concepts`, phase steps resolved→`Phases`, step baby-steps resolved→`Steps`, authored baby-step batch→`BabySteps`, and authored detail→`BabyStepDetail`. [C-12:L38-L44]
- **C12-E04 — [prompt/config]** `.claude/agents/auditor.md` must contain six substantive (~80 LOC) REVIEW blocks named Concepts, Phase, Step, BabyStep, BabyStepDetail, and Cosmos; missing/placeholders are added, and Concepts accepts `framed_idea`. [C-12:L68-L72; C-12:L208-L212]
- **C12-E05 — [data/fallback]** Verify `FramedIdea` is available at the Concepts call and forward it; acceptance adds that an unframed task warns and falls back rather than hard-aborting. [C-12:L74-L78; C-12:L213-L214]
- **C12-E06 — [event/failure]** Before each of five auditor spawns, call `per_level_audit_dispatch::emit_started(&ssb,agent_id,superstep,mode,tier_node_id)` using the existing typed node ID. Emit failures are warnings and cannot abort the pipeline. [C-12:L80-L85; C-12:L215-L216]
- **C12-E07 — [event/workflow]** After each auditor returns `LevelAuditVerdict`, call `record_verdict(&ssb,agent_id,superstep,verdict)` at all five tiers. [C-12:L87-L90; C-12:L217-L218]
- **C12-E08 — [state]** `CascadeOutcome::Pass` continues unchanged. [C-12:L90-L92]
- **C12-E09 — [retry/idempotence]** `CascadeOutcome::RespawnOnce` respawns the same auditor with the updated verdict through existing bubble-up logic; a monotonic guard must prevent double respawn. [C-12:L92-L94; C-12:L219-L220]
- **C12-E10 — [recovery/state]** `CascadeOutcome::Cascade` bubbles to the parent tier decomposer through the existing all-resolved path. [C-12:L94-L95; C-12:L221-L221]
- **C12-E11 — [recovery/state]** `CascadeOutcome::EscalateToIdea` routes to the B-40 FRAMING retry and emits an ESCALATE log. [C-12:L95-L96; C-12:L222-L222]
- **C12-E12 — [failure]** `record_verdict` errors must surface as ESCALATE rather than being silently swallowed; no wildcard cascade arm is permitted. [C-12:L147-L150; C-12:L217-L218]
- **C12-E13 — [surface/data]** `get_per_level_verdicts` gains AppState, scans SSB for the task ID and action types `per_level_audit_verdict` plus `per_level_audit_cascade`, deserializes `Vec<LevelAuditVerdictJson>`, and replaces `Ok(Vec::new())`. [C-12:L98-L102]
- **C12-E14 — [surface/state]** Existing `AuditTimelineWindow.svelte` binding/props remain; live output renders mode-colored markers and arrows where `cascade_to_parent=true`. [C-12:L104-L107; C-12:L225-L227]
- **C12-E15 — [test topology]** `navigator_audit_wiring_contract.rs` builds 3 concepts × 1 phase × 2 steps × 2 baby steps, runs all five completions, asserts five distinct mode-correct verdicts, and enforces started-before-verdict sequence order. [C-12:L109-L117]
- **C12-E16 — [test topology]** `audit_timeline_command_contract.rs` seeds five verdict and two cascade rows, expects exactly five JSON entries, and expects two `cascade_to_parent` mappings. [C-12:L119-L125]
- **C12-E17 — [observability]** Cloud smoke builds a three-concept Rust project and requires five distinct, correctly-modeled verdict rows, started-before-verdict order, a command result of at least five entries, five UI markers, and conditional cascade arrows. [C-12:L152-L163; C-12:L233-L237]
- **C12-E18 — [done gate]** Mechanical checks require import presence, all five enum modes, exactly five `emit_started` and five `record_verdict` references, all four outcome arms, removal of the empty stub, both contract files, clean scoped compilation, and both passing test filters. [C-12:L165-L206]
- **C12-E19 — [files]** Writes are limited to navigator integration, the Tauri command, conditional prompt completion, two contract tests, and a C-12 decision document. [C-12:L239-L251]

### C-19 — Test Infrastructure Backfill

- **C19-E01 — [scope/status]** Objective: fill missing/partial adversarial tests, Criterion benchmarks, quantified metrics, and regression gates for phases 4A/4C/4D/4F and B-9; estimated 2-3 sessions with no production-code changes. [C-19:L1-L5]
- **C19-E02 — [dependencies/deferred]** Requires C-11 sandbox symbols and conditionally C-22 feasibility types; tests may be compile-gated under `stub_result_v2` until C-22 lands. Parallel-safe with C-18/C-20 due to non-overlapping test/bench/metrics scope. [C-19:L6-L11; C-19:L227-L230]
- **C19-E03 — [security/test]** `test_4a_adv_ipc_u32_max_length_rejected` sends `u32::MAX` frame length and requires `PayloadTooLarge` before a 4GB allocation. [C-19:L62-L66]
- **C19-E04 — [security/test]** `test_4a_adv_ipc_so_peercred_mismatch` connects a second process after accept and requires `SO_PEERCRED` rejection. [C-19:L67-L68]
- **C19-E05 — [resilience/test]** `test_4a_adv_ipc_think_step_burst_10k` sends 10,000 `ThinkStep`s, drops excess under budget, avoids OOM, and preserves orchestrator responsiveness. [C-19:L69-L70]
- **C19-E06 — [security/test]** `test_4a_adv_selfmod_path_traversal` supplies `../../../metrics.rs` and requires canonical-realpath rejection. [C-19:L71-L72]
- **C19-E07 — [security/test]** `test_4a_adv_selfmod_bash_write_locked` blocks a Bash write to a locked path with no mutation. [C-19:L73-L74]
- **C19-E08 — [security/test]** `test_4a_adv_selfmod_git_commit_locked` rejects a commit targeting a locked path before issuing it. [C-19:L75-L76]
- **C19-E09 — [security/test]** `test_4a_adv_selfmod_symlink_resolved` resolves a symlink and applies the lock to its canonical target. [C-19:L77-L78]
- **C19-E10 — [workflow/test]** `test_4a_adv_replan_minor_fast_track` requires MINOR-with-evidence auditor fast-track without an INTERACTIVE gate. [C-19:L79-L80]
- **C19-E11 — [permissions/test]** `test_4a_adv_replan_major_human_gate` requires a MAJOR request to raise the INTERACTIVE gate and not auto-proceed. [C-19:L81-L82]
- **C19-E12 — [idempotence/test]** `test_4a_adv_replan_max_one_per_phase` rejects a second request from the same agent in the same phase. [C-19:L83-L84]
- **C19-E13 — [validation/test]** `test_4a_adv_replan_no_evidence_rejected` rejects `evidence=None`. [C-19:L85-L86]
- **C19-E14 — [lifecycle/test]** `test_4a_adv_discovery_proposed_gate` requires `PROPOSED` before any `CANDIDATE` transition. [C-19:L87-L88]
- **C19-E15 — [resource/test]** `test_4a_adv_discovery_inflated_patterns_capped` deduplicates/caps 1,000 repeated violation hashes. [C-19:L89-L90]
- **C19-E16 — [ordering/test]** `test_4a_adv_crash_ssb_before_toolresult` requires synchronous SSB confirmation before sending `ToolResult`. [C-19:L91-L92]
- **C19-E17 — [recovery/test]** `test_4a_adv_crash_partial_frame_discard` SIGKILLs mid-frame and requires partial-buffer discard on close without hang. [C-19:L93-L94]
- **C19-E18 — [test compliance]** Every one of the 15 phase-4A additions carries a `// Covers audit BLOCK-N / WARN-N` comment. [C-19:L62-L64; C-19:L288-L290]
- **C19-E19 — [performance]** Add Criterion 0.5 only after lockfile verification, a `[[bench]]`, and four groups in `tiered_analysis_bench.rs`: T1 with 1,000 AST rules/p50<5ms; T2 at 4,999/5,000/5,001 lines/p50<50ms; T3 WARN path/p50<500ms; and total `PostToolUse` latency with Ruff simulation. HTML goes under `target/criterion/`. [C-19:L96-L107]
- **C19-E20 — [observability/data]** Add nine exact TOML metric sections: `t1_latency`, `t2_latency`, `t3_latency`, `hook_overhead`, `post_tool_use_latency`, `compile_time_delta`, `test_count`, `loc`, and `coverage`, with the stated fields; measured values are preferred, while pending zero placeholders may be CI-replaced. [C-19:L108-L114]
- **C19-E21 — [done gate]** `benchmark_eval_improved.rs` asserts `delta_pct >= -5.0`, blocking regressions worse than five percent. [C-19:L116-L119]
- **C19-E22 — [security/test]** `adversarial_filesystem_escape` attempts `/etc/passwd` and requires IPC-layer BLOCK with no content. [C-19:L123-L127]
- **C19-E23 — [security/test]** `adversarial_direct_network_access` requires seccomp to block a direct TCP socket, preserving IPC-only networking. [C-19:L128-L129]
- **C19-E24 — [recovery/test]** `adversarial_grc_budget_exhaustion` uses budget 1 and requires forced convergence/status rather than loop or hang. [C-19:L130-L131]
- **C19-E25 — [safety/test]** `adversarial_all_training_high_risk` marks `ALL-TRAINING` as `HIGH_RISK` and excludes the node from the final report. [C-19:L132-L133]
- **C19-E26 — [security/test]** Feasibility A1 attempts a stub filesystem escape and requires IPC BLOCK/no content. [C-19:L137-L141]
- **C19-E27 — [security/test]** A2 attempts direct TCP, requires seccomp block, and populates `StubResult.error`. [C-19:L141-L142]
- **C19-E28 — [failure/test]** A3 sends malformed IPC and requires `StubTimeout` escalation. [C-19:L142-L143]
- **C19-E29 — [concurrency/test]** A4 concurrently claims GPU from two stubs and forbids VRAM over-allocation. [C-19:L143-L144]
- **C19-E30 — [fallback/test]** A5 models zero GPUs and requires `ProceedWithConstraints` or `Escalate`, never panic. [C-19:L144-L145]
- **C19-E31 — [boundary/test]** A6 sends exactly `MAX_FRAME_SIZE` bytes and requires acceptance. [C-19:L146-L146]
- **C19-E32 — [race/recovery/test]** A7 drops VRAM between probe and launch; recheck must catch it and return `Escalate`. [C-19:L147-L148]
- **C19-E33 — [resource/test]** A8 enforces `MAX_ALTERNATIVES_PER_STEP=3` by rejecting a fourth alternative. [C-19:L149-L150]
- **C19-E34 — [timeout/test]** Every feasibility adversarial fixture, including an infinite loop, must be caught within a 30-second harness. [C-19:L150-L150; C-19:L231-L232]
- **C19-E35 — [lifecycle/test]** T9 creates one SSB branch per stub alternative tagged `branch:stub_{step_id}_{n}`; three alternatives produce three rows. [C-19:L152-L155]
- **C19-E36 — [race/test]** T13 calls `ModelPool.available_vram()` before each launch and escalates if the mock reports insufficient VRAM. [C-19:L156-L157]
- **C19-E37 — [determinism/recovery/test]** T14 models graceful ACK by 25s versus hard kill at 30s using paused Tokio time, returning `StubResult` or `StubTimeout` respectively. [C-19:L158-L159]
- **C19-E38 — [fallback/test]** T15 returns `ProceedWithConstraints` when one of two GPUs exists and records `constraint_result.passed=false` without escalating. [C-19:L160-L161]
- **C19-E39 — [failure/test]** T16 turns malformed IPC into a non-panicking `StubResult` with populated error and `alternative_index=0`. [C-19:L162-L163]
- **C19-E40 — [workflow/test]** `baby_step_feasibility_skip_when_margins_large` short-circuits with a skip reason when all margins are ≥20%. [C-19:L165-L168]
- **C19-E41 — [observability/test]** `baby_step_feasibility_resource_delta_tracked` requires nonzero before/after VRAM delta. [C-19:L169-L170]
- **C19-E42 — [resource/test]** `baby_step_feasibility_max_alternatives_cap` rejects a fourth alternative. [C-19:L171-L172]
- **C19-E43 — [recovery/test]** `baby_step_feasibility_escalation_propagates` surfaces the child escalation in the parent report list. [C-19:L173-L174]
- **C19-E44 — [CLI/test]** `test_session_merge_slash_command` parses `/session merge branch-abc` into the Session merge action and exact branch ID. [C-19:L178-L182]
- **C19-E45 — [CLI/test]** `test_session_close_slash_command` does the analogous close parse. [C-19:L183-L184]
- **C19-E46 — [surface/test]** `test_branch_tree_onfork_prop_fires` simulates the prop event and requires the callback to receive the branch name. [C-19:L185-L187]
- **C19-E47 — [surface/test]** `test_branch_tree_onmerge_onclose_props_present` compile-checks both props against Tauri command signatures rather than rendering Svelte. [C-19:L188-L192]
- **C19-E48 — [constraints]** Tests must compile against existing production APIs; missing C-22 fields may be feature-gated and documented, and adversarial processes must use bounded timeout/join so no test runner can hang. [C-19:L227-L232]
- **C19-E49 — [observability]** Explicit executions cover adversarial IPC/research/session suites, feasibility suites, and plugin benches; smoke expects traversal blocking with no `SANDBOX_VIOLATION` row, T1/T2/T3 timing output, and A1-A8 within 30s. [C-19:L220-L242]
- **C19-E50 — [done gate]** Mechanical acceptance requires ≥15 new 4A functions, benchmark file, ≥9 metric sections, regression assertion, exactly four research tests, eight A-tests, five T-tests, ≥6 baby-step tests total, four B-9 tests, and clean workspace compilation. [C-19:L244-L310]

### C-27 — Testing-Trust T(-1) and T-0

- **C27-E01 — [capability/status]** Objective combines determinism tier T(-1) and wiring-contract tier T-0: hardware/seed/cache/fingerprint determinism, drift evidence, contract parsing/auditing, CLI, and CI; estimated 2-3 sessions. [C-27:L1-L9]
- **C27-E02 — [dependencies/concurrency]** Requires C-15 clock/entropy and C-17 DKG resilience; declared parallel-safe with C-21 through C-26 except the sole shared serialization point in `ssb.rs`. [C-27:L10-L14]
- **C27-E03 — [schema]** Bump `SCHEMA_VERSION` 46→47 and add `WiringEvidence`, `WiringViolation`, and `DeterminismDrift` action/payload variants with Debug/Clone/Serialize/Deserialize. [C-27:L61-L71]
- **C27-E04 — [schema]** `WiringEvidence` contains `entrypoint:String`, `produces_symbols_verified:Vec<String>`, `reachable_count:u32`, and `audit_id:Uuid`. [C-27:L63-L64]
- **C27-E05 — [schema]** `WiringViolation` contains `violation_kind:WiringViolationKind`, `file:String`, `line:u32`, `pattern:String`, and `audit_id:Uuid`; kinds are `StubPattern`, `UnreachableSymbol`, and `BrokenObservable`. [C-27:L65-L69]
- **C27-E06 — [schema]** `DeterminismDrift` contains `seed:u64`, `run_a_hash:String`, `run_b_hash:String`, `first_divergence_index:u32`, and `audit_id:Uuid`. [C-27:L67-L68]
- **C27-E07 — [config/default]** `DeterminismConfig` fields are `hardware_class:HardwareClass`, `seed:u64`, `blake3_cache_prefix:[u8;8]`, and `system_fingerprint_pin:Option<String>`; `HardwareClass={Cpu,Gpu(GpuSku),Unknown}`, with `GpuSku` a string newtype. [C-27:L73-L76]
- **C27-E08 — [config/default]** `from_env` reads `ELAI_SEED` defaulting to 0 and `ELAI_HARDWARE_CLASS` defaulting to Unknown. `matches_fingerprint` returns true when no pin is configured. [C-27:L77-L82]
- **C27-E09 — [determinism/security]** The eight-byte prefix is the first eight bytes of `blake3::hash(seed.to_le_bytes())`; `prefix_cache_key` pads it to 32 bytes and uses `blake3::keyed_hash`. SHA-256/MD5 are forbidden for this cache key. [C-27:L78-L81; C-27:L189-L191]
- **C27-E10 — [failure]** `DeterminismError` is a `thiserror` enum with `EnvParseError(String)` and `HashKeyError`; determinism code may not unwrap. [C-27:L81-L82]
- **C27-E11 — [determinism/event]** `DriftComparator {seed,ssb_tx,audit_id}` compares `ActionTypeSequence(Vec<String>)`, emits one drift row at the first differing zipped index with BLAKE3 hashes of joined sequences, emits zero rows for matches, and uses an unbounded SSB channel without unwrap. [C-27:L84-L88]
- **C27-E12 — [schema]** `WiringContract` fields are `bn_id`, `produces_symbols`, `consumes_symbols`, `reachable_from`, `stub_ban`, optional `frontend_pair`, and `observable_effects`; `StubBanEntry` has pattern/description and `ObservableEffect` has trigger, optional SSB/artifact expectations, and timeout seconds. [C-27:L90-L97]
- **C27-E13 — [failure]** `WiringContractError` variants are parse, I/O-from-`io::Error`, and regex errors; all core contract types derive Debug/Clone/Serialize/Deserialize. [C-27:L96-L97]
- **C27-E14 — [config/fallback]** `EntrypointSetLoader {config_path}` reads TOML `[wiring.entrypoints] symbols`; a missing section returns an empty set with `tracing::warn!`, never panic. A commented template is added to `elai_cc.toml`. [C-27:L99-L104]
- **C27-E15 — [security/audit]** `StubBanAuditor` compiles regexes and emits `WiringViolation(StubPattern)` with file/line. Production walks exclude `/tests/`, `_test.rs`, and `_contract.rs`; default patterns cover `todo!()`, `unimplemented!()`, `deferred`, `stub`, and `Ok(Default::default())`. [C-27:L106-L113]
- **C27-E16 — [reachability/evidence]** `DkgReachabilityAuditor` reuses `SymbolGraph` and breadth-first searches from every entrypoint, short-circuiting on first hit; reachable produced symbols emit `WiringEvidence`, unreachable ones emit `WiringViolation(UnreachableSymbol)`. [C-27:L115-L119]
- **C27-E17 — [observability/retry]** `ObservableEffectRunner` polls the SQLite SSB DB every second under `tokio::time::timeout`; missing expected SSB or artifact produces `BrokenObservable` plus `Err`, never panic. [C-27:L121-L126]
- **C27-E18 — [CLI/workflow]** `WiringAuditCmd {config_path,contracts_dir,ssb_db_path,seed}` loads entrypoints, globs Markdown, parses `WIRING_CONTRACT` blocks, runs all three auditors, aggregates `WiringAuditReport {evidence,violations,audit_id}`, prints PASS/FAIL with file:line, and exits 0/1. [C-27:L128-L136]
- **C27-E19 — [surface]** Expose `elai wiring audit [--config PATH] [--contracts PATH]` and add `SlashCommand::WiringAudit`. [C-27:L135-L136]
- **C27-E20 — [CI/compliance]** A Windows `wiring-audit.yml` builds release CLI and audits configured contracts on push and pull request to main, returns nonzero on violations, and skips `[ci skip]` commits. [C-27:L138-L142]
- **C27-E21 — [tests]** Unit coverage requires stable same-seed keys, drift/non-drift behavior, stub detection and test exclusion, empty missing-entrypoint behavior, and broken-observable timeout emission. [C-27:L144-L148]
- **C27-E22 — [contract/done gate]** Tier-0 contract requires the three SSB variants, public schema 47, environment seed 42 success, and zero drift for equal sequences. [C-27:L150-L154]
- **C27-E23 — [documentation]** Decision record mandates BLAKE3 rather than SHA-256, BFS rather than DFS for bounded large-graph traversal, and one-second SQLite polling because there is no NOTIFY and polling is testable. [C-27:L156-L160]
- **C27-E24 — [stub bans]** Explicit bans: no alternate digest; no scanning test/contract paths; no drift row on equality; no timeout panic; no panic on missing config; and exact CLI exit codes. [C-27:L189-L195]
- **C27-E25 — [observability]** Evidence scenarios cover clean entrypoints, a manually injected default-return stub with file/line violation, two seed-42 cloud runs with zero drift, and a nonexistent event producing `BrokenObservable` within timeout+5s. [C-27:L196-L205]
- **C27-E26 — [lifecycle/migration]** Acceptance uniquely requires an old schema-46 DB to emit a migration warning when public schema is 47. [C-27:L253-L255]
- **C27-E27 — [done gate]** Acceptance requires payload round-trips, exact violation kinds, correct first divergence, reachable/unreachable evidence, CLI behavior, four smoke scenarios, Windows CI triggers, clean workspace, ≥10 scoped unit tests, tier-0 contract pass, and the three-decision document. [C-27:L253-L286]
- **C27-E28 — [files]** Adds determinism/wiring modules, CLI dispatch, CI workflow, contract tests, decision doc, and modifies SSB, CLI commands, and TOML. [C-27:L288-L310]

### C-42 — Semantic Gateway Zero-Trust MCP Fuzzing

- **C42-E01 — [security/capability]** Objective: deterministically discover 100% of unauthorized state transitions in ELAI's synthetic MCP fixtures before C-40/C-44 add consumers; target after fixes is zero false negatives/zero CRITICAL findings on that corpus. [C-42:L1-L7; C-42:L36-L45]
- **C42-E02 — [dependencies/status]** Phase 4K, C-35, and C-39 must be shipped. It is orthogonal to C-36/37/38/41/43-53 but conflicts with C-35's MCP surface. [C-42:L9-L12]
- **C42-E03 — [threat model]** External consumers include frameworks, IDEs, CLI, future Composer 2, and GitHub PR routing; an LLM-controlled prompt is treated as potentially malicious and cannot rely on dashboard trust. [C-42:L21-L34]
- **C42-E04 — [security/workflow]** The fuzzer enumerates FSM state × tool × argument-shape tuples, compares C-35 gate plus per-tool policy, and reports any unauthorized-but-authorized outcome. [C-42:L36-L41]
- **C42-E05 — [evidence/dependency]** Paper and source ingestion for arXiv:2604.25555 produces `workflow/papers/semantic-gateway-fuzzer.md`. [C-42:L14-L19; C-42:L56-L58]
- **C42-E06 — [permissions/schema]** Every tool gets `#[authz(states="PLANNING,EXECUTING",actors="fsm,user")]` or equivalent TOML; `AuthzPolicy::check(tool_name,fsm_state,actor)` returns `Authorized` or `Rejected{reason}`. [C-42:L60-L63]
- **C42-E07 — [constraints/security]** New `crates/mcp_fuzzer` is guarded by feature `mcp_fuzzer`, integration-test-only, and excluded from production binaries. [C-42:L65-L68]
- **C42-E08 — [determinism]** `FuzzCorpus`, `FuzzRunner`, and `FuzzReport` enumerate reproducibly through seeded RNG. [C-42:L65-L68]
- **C42-E09 — [test data]** Corpus covers every registered tool × every FSM state × five shapes (`valid`, `oversized`, `undersized`, `malformed`, `escape-attempting`) with at least 200 tuples. [C-42:L70-L73]
- **C42-E10 — [integration/data]** Runner binds a localhost MCP bridge, invokes every tuple, and records `(authorized,expected_authorized,response_shape)`. [C-42:L75-L76]
- **C42-E11 — [failure/severity]** Report categories are authorized-when-reject=`CRITICAL`, rejected-when-authorize=`LATENT BUGS/P1`, and non-structured rejection=`response-shape violation/BLOCK`. [C-42:L77-L82]
- **C42-E12 — [CI/compliance]** Every PR runs the fuzzer, posts a findings table as a PR comment, preserves a report JSON artifact, and any CRITICAL result blocks merge. [C-42:L84-L85; C-42:L135-L140]
- **C42-E13 — [events/ordering]** Add `mcp_authz_check`, `mcp_authz_reject`, and `fuzzer_finding`; the check row must precede tool execution, rejection emits when denied, and fuzzer findings emit from integration tests. [C-42:L87-L89; C-42:L124-L138]
- **C42-E14 — [security/fallback]** Missing tool declarations default to DENY-ALL without failing registration and emit a structured startup warning. Rejection evidence must include `tool_name`, `fsm_state`, `actor`, and `denied_reason`; generic `unauthorized` is insufficient. [C-42:L99-L103; C-42:L127-L133]
- **C42-E15 — [TDD]** Named tests 4K.X.3-.7 cover deterministic corpus, real bridge round-trip, per-state/tool policy, discovery of one deliberate known bug, and a post-fix zero-CRITICAL rerun; tests must precede wiring. [C-42:L91-L97; C-42:L183-L185]
- **C42-E16 — [audit]** Sweep every currently registered tool and add explicit authz annotations. [C-42:L99-L103]
- **C42-E17 — [surface/status]** `AuthzPanel.svelte` is explicitly **PLANNED** to show real-time check/reject rates and click-through reason histograms; acceptance nevertheless asks it to render. [C-42:L105-L107; C-42:L186-L186]
- **C42-E18 — [contract]** `mcp_authz_contract.rs` verifies interface shape and fail-closed default behavior. [C-42:L109-L110]
- **C42-E19 — [done gate]** Deterministic gates require authz module, fuzzer crate, ≥200 corpus entries, all three events, annotation cross-check for every registration, feature-enabled tests, CI job, and contract pass. [C-42:L142-L169]
- **C42-E20 — [release gate]** C-40 and C-44 progress/merge is gated until the synthetic corpus reports zero CRITICAL; findings become individual P0 tickets. [C-42:L213-L219]

### C-43 — Phase 4H Scope-Freeze Ratification

- **C43-E01 — [policy/status]** Objective freezes Phase 4H at its current scope and explicitly defers continuous self-reward, evolutionary prompt search, and online-RL fine-tuning until measurement justifies expansion. [C-43:L1-L8]
- **C43-E02 — [dependencies/concurrency]** Requires shipped Phase 4H and is declared parallel-safe with every C plan as a policy/documentation patch plus defensive guardrails. [C-43:L10-L12]
- **C43-E03 — [current lifecycle]** Context describes current behavior as layer 1 mistake-driven prompt modification, layer 2 auditor approval, layer 3 MSI regression-triggered auto-revert, plus a Cosmos tier for non-code artifacts. [C-43:L25-L33]
- **C43-E04 — [status/evidence]** MSI auto-revert is the only narrow path labeled HIGH-confidence; isolated value of layers 1-2 is unmeasured. AEL anti-evidence says mechanisms beyond memory+reflection degrade open-ended performance. [C-43:L14-L21; C-43:L35-L42]
- **C43-E05 — [evidence]** Paper ingestion produces `ael-ablation-interface.md` with the per-mechanism ablation table verbatim. [C-43:L61-L63]
- **C43-E06 — [policy/compliance]** `phase-4h-scope-policy.md` cites AEL, defines current scope, and admits a fourth layer only if it improves at least two published post-C-54 benchmarks, wins a side-by-side same-fixture ablation by strictly ≥+3 percentage points, and has per-project explicit opt-in (never default). [C-43:L65-L73]
- **C43-E07 — [guardrail/permissions]** A new self-mod path requires `#[allow(self_mod_experimental, justify="...")]`; the guard emits a compile warning unless the feature is enabled, and missing annotation fails CI clippy-as-error. [C-43:L75-L81]
- **C43-E08 — [test/evidence]** `ablation_harness.rs` accepts baseline/candidate `AbstractAgent` configurations, runs N matched tasks, and reports per-task and aggregate delta for appending to policy. [C-43:L83-L87]
- **C43-E09 — [events]** Add `self_mod_layer_proposed` for attempted improperly-gated registration and `self_mod_ablation_run` whenever the harness runs; the latter must emit before returning. [C-43:L89-L91; C-43:L126-L129]
- **C43-E10 — [determinism/test]** 4H.X.3 requires deterministic matched-pair delta on a synthetic two-config fixture. [C-43:L93-L95]
- **C43-E11 — [guardrail/test]** 4H.X.4 requires a compile warning for a new path lacking the annotation. [C-43:L96-L97]
- **C43-E12 — [compatibility/test]** 4H.X.5 grandfathers the current three layers through a baseline allowlist and forbids guardrail noise for them. [C-43:L98-L99]
- **C43-E13 — [anti-rubber-stamp/test]** 4H.X.6 requires a synthetic DGM-like candidate to show a negative delta on at least one adversarial fixture. [C-43:L100-L102]
- **C43-E14 — [documentation]** Append the exact scope-expansion lesson to the active `mistakes.md` and add a dated C-43 scope-freeze/benchmark-gate footnote to `DETAILED_SUMMARY_01` Phase 4H. [C-43:L104-L110]
- **C43-E15 — [constraints]** The policy must cite AEL verbatim; the guard cannot silently allow; the harness cannot return before writing its event; there is no frontend surface. [C-43:L126-L132]
- **C43-E16 — [CI/done gate]** A PR adding an ungated path must surface a clippy warning and cannot merge without explicit annotation plus an ablation result attached to policy. [C-43:L133-L136]
- **C43-E17 — [done gate]** Acceptance requires paper/policy artifacts, all three decision criteria, macro, allowlist, deterministic harness, 4H.X.3-.6, negative fixture, summary/mistakes edits, and lint rejection. [C-43:L164-L177]
- **C43-E18 — [non-goal]** No expansion mechanism is adopted by this plan; any later self-improvement loop must first produce the harness's markdown evidence table. [C-43:L197-L203]

### C-50 — Contract-Coding Test Scaffolding

- **C50-E01 — [capability]** A fresh project emits per-function preconditions/postconditions before implementation, extending FILE_PLAN and skeleton-first completeness. [C-50:L1-L7; C-50:L18-L32]
- **C50-E02 — [dependencies/concurrency]** Requires shipped Phase 4F completeness and is parallel-safe with C-35-C-49 and C-51-C-53. [C-50:L9-L11]
- **C50-E03 — [evidence]** Paper/source ingestion targets arXiv:2604.13100 and its open implementation; cited benchmark evidence is 47% functional and near-perfect structural integrity. [C-50:L13-L16; C-50:L43-L44]
- **C50-E04 — [schema/language]** `ContractSpec` holds `preconditions:Vec<Predicate>`, `postconditions:Vec<Predicate>`, and `contract_language:ContractLang`; supported expressions are Rust `assert!`, Python `assert`, and TypeScript predicates. [C-50:L46-L50]
- **C50-E05 — [validation/security]** Predicates are target-language source strings and must parse through tree-sitter; parse failure rejects the contract and cannot fall back to permissive validation. [C-50:L47-L50; C-50:L95-L99]
- **C50-E06 — [workflow]** `ContractEmitter` runs after FILE_PLAN and before skeleton, prompts once per planned function for PRE/POST predicates, validates syntax, and stores `.elai/contracts/{file_path}.toml`. [C-50:L52-L55]
- **C50-E07 — [workflow/analysis]** `ContractViolationCheck` runs at body completion, parses the body AST, checks preconditions on entry paths and postconditions on return paths, and extends MiniCritic. [C-50:L57-L60]
- **C50-E08 — [retry/event]** Violations emit SSB before entering `ContractViolationRetry`; a corrected body can resolve the retry through the existing structural chain. [C-50:L57-L60; C-50:L95-L97]
- **C50-E09 — [config/default]** `elai new project` enables contracts by default; existing projects opt in through `[code_completeness.contracts.enabled]`. [C-50:L62-L64]
- **C50-E10 — [events]** Add `contract_emitted`, `contract_violation_detected`, and `contract_retry_resolved`. [C-50:L66-L67]
- **C50-E11 — [test]** 4F.X.3 verifies TOML round-trip of `ContractSpec`. [C-50:L69-L70]
- **C50-E12 — [test]** 4F.X.4 validates predicates through target-language tree-sitter. [C-50:L70-L71]
- **C50-E13 — [test]** 4F.X.5 detects a deliberately violated body. [C-50:L71-L72]
- **C50-E14 — [test/recovery]** 4F.X.6 verifies retry resolution with a corrected body. [C-50:L72-L73]
- **C50-E15 — [benchmark/test]** 4F.X.7's task text targets 47% functional and ≥95% structural success on Greenfield-5; final acceptance uses conservative gates of ≥40% and ≥90%. [C-50:L74-L75; C-50:L120-L122]
- **C50-E16 — [contract/done gate]** Add `contract_emitter_contract.rs`; deterministic checks require module/symbol/event/fixture existence plus passing code-completeness contract tests. [C-50:L77-L77; C-50:L101-L109]
- **C50-E17 — [composition]** The intended three-layer pipeline is FILE_PLAN declarations → behavioral contracts → skeleton bodies constrained by contracts. [C-50:L138-L142]

### C-57 — OpenHands Index + SWE-Cycle + SWE Atlas

- **C57-E01 — [capability/status]** Run three public benchmarks in one RunPod session to amortize cost: OpenHands Index (issue/greenfield/frontend/testing), SWE-Cycle (FullCycle), and SWE Atlas (Q&A/test writing/refactoring). The plan is explicitly RunPod-dependent. [C-57:L1-L6]
- **C57-E02 — [dependencies/concurrency]** C-54's primary SWE-bench Verified number must exist first; the run is described as parallel-safe with C-54/55/56/58 but sequential on the same RunPod. [C-57:L8-L10]
- **C57-E03 — [benchmark data]** Recorded reference axes include OpenHands' 72% internal SWE-bench baseline, SWE-Cycle's isolated→FullCycle solve-rate drop, and SWE Atlas' 124 Q&A, 90 test-writing, and 70 refactoring tasks. [C-57:L12-L21]
- **C57-E04 — [evaluation intent]** SWE-Cycle tests the full PLANNING→EXECUTING→AUDITING→MEMORIZING FSM across its own iterative outputs, including MSI/bubble-up; SWE Atlas measures local-vs-frontier gap and hybrid closure; OpenHands approximates developer workflows. [C-57:L23-L34]
- **C57-E05 — [config/cost]** Default is one hybrid config per benchmark—Sonnet 4.6 designer, Composer 2 executor, local auditor. A local-only arm is added only if a useful published comparator exists; exhaustive 3×3 runs are rejected as over-budget. [C-57:L36-L39; C-57:L59-L62]
- **C57-E06 — [evidence]** Ingest SWE-Cycle/SWE Atlas papers and freshly fetch/document OpenHands methodology. [C-57:L41-L50]
- **C57-E07 — [integration]** Add `openhands_index_adapter.rs`, `swe_cycle_adapter.rs`, and `swe_atlas_adapter.rs`; each loads tasks, presents them to ELAI, collects patches/answers, and scores with the benchmark's official rubric. [C-57:L52-L57]
- **C57-E08 — [methodology/determinism]** Run discipline is single-attempt pass@1, no cherry-picking, and one scorecard JSON per benchmark. [C-57:L64-L65]
- **C57-E09 — [scoring]** OpenHands follows published per-axis scoring; SWE-Cycle reports the solve-rate curve by cycle; SWE Atlas uses rubric scoring via Sonnet 4.6 or the paper's judge if released. [C-57:L67-L71]
- **C57-E10 — [artifacts]** Produce one combined methodology/results treatment for the three runs plus three separate scorecards, using one RunPod instance. [C-57:L73-L74; C-57:L89-L94]
- **C57-E11 — [evidence/questions]** Required findings compare ELAI's FullCycle drop with the paper, ELAI hybrid with same-class Llama/Qwen SWE Atlas baselines, and ELAI's OpenHands axes with published axes. [C-57:L76-L82]
- **C57-E12 — [events]** Add `multi_benchmark_run_started`, per-axis/per-benchmark `multi_benchmark_axis_completed`, and `multi_benchmark_scorecard_emitted`. [C-57:L84-L85]
- **C57-E13 — [dependencies]** Harnesses consume all three backend tiers after C-44 and the SSB bus. [C-57:L95-L97]
- **C57-E14 — [surface]** CLI entrypoints are `elai benchmark openhands-index|swe-cycle|swe-atlas --runpod-id <id>` and suite mode `elai benchmark suite --runpod-id <id>`. [C-57:L98-L100]
- **C57-E15 — [constraints/compliance]** Per-axis—not aggregate-only—scores are mandatory; SWE-Cycle must expose each cycle; all external comparisons cite version and date. [C-57:L101-L105]
- **C57-E16 — [done gate]** Acceptance requires three completed runs, three events, per-axis/curve/rubric outputs, honest combined results, all Task-7 findings, and total RunPod cost below three independent-run costs. [C-57:L120-L132]
- **C57-E17 — [artifacts/lifecycle]** Vendored corpora are added for all three (SWE Atlas only if released); post-run artifacts live under the active session, while public results live at `.elai_cc/docs/multi_benchmark_results.md`. [C-57:L134-L150]
- **C57-E18 — [status/dependency]** Closure expands evidence beyond C-54 and feeds C-59's aggregate comparison. [C-57:L152-L157]

## Inferred interactions and consequences (not direct commitments)

- **I01 — Schema sequencing.** C-1, C-5, C-27, C-42, C-43, C-50, and C-57 all mutate the SSB event family. Their changes therefore need a single migration/order strategy even where individual sources claim parallel safety; fixed version literals cannot be applied independently. This follows from the shared schema surfaces, but no unified migration plan is stated.
- **I02 — Spend enforcement race.** C-5's read-before-route cap check is not described as a reservation/transaction. Concurrent cloud calls could all read a below-cap value and cross the ceiling together unless the implementation adds serialization or an atomic spend reservation.
- **I03 — Daily-warning idempotency.** “No prior row this calendar day” implies a stable day boundary and concurrency-safe deduplication key/transaction; neither is specified.
- **I04 — Cache-counter concurrency.** C-1's hit/miss mutation plus flush-and-reset needs synchronization if `VectorCache` is shared, but counter atomicity/locking is not stated.
- **I05 — Project-index concurrency.** C-4's temp-file/rename makes each replacement crash-resistant, but without locking two concurrent `create_project` calls can still lose an append.
- **I06 — Audit correlation.** C-12's started-before-verdict contract implies rows can be correlated by mode/tier/task/agent and sequence, although no explicit correlation ID is introduced.
- **I07 — Zero-trust coverage.** C-42's “100%” result applies only to the enumerated synthetic corpus and expected-policy oracle, not to unmodeled states, tools, or semantic attacks.
- **I08 — Scope guard evidence.** C-43's ≥+3pp/two-benchmark gate makes C-54/C-57 outputs governance inputs for future self-mod features even though the plans do not define an automated link.
- **I09 — Contract checking limits.** C-50's AST path checks establish static structural evidence; they do not by themselves prove arbitrary semantic predicates. Runtime/property-test interpretation is not specified.
- **I10 — Benchmark reproducibility.** C-57's pass@1/no-cherry-picking discipline plus version/date citations implies immutable corpus/config/run metadata should accompany scorecards, but the exact provenance schema is not specified.

## Ambiguities requiring a decision

- **A01 — C-1 engine injection.** The source permits `Option<Arc<Ssb>>`, a channel, or an existing side-channel for `BasicGrcEngine::advance`; the owning API and failure semantics are intentionally undecided. [C-1:L88-L91]
- **A02 — C-1 current schema baseline.** “Bump by exactly 1” and deterministic “≥47” do not identify the branch's actual pre-bump value. [C-1:L56-L60; C-1:L210-L222]
- **A03 — C-1 trace/cache identities.** Generation and uniqueness rules for `query_id`, cache `session_id`, and the meaning of `tier`/`stage_count` are not defined. [C-1:L69-L71; C-1:L103-L107]
- **A04 — C-4 baby-step source.** The reader may query `baby_step_ready` **or** `concept_decomposition_node`; precedence and result-shape reconciliation are not stated. [C-4:L94-L101]
- **A05 — C-4 project root/month boundary.** Resolution of `<project_root>`, validation of names/parent paths, duplicate project behavior, and the timezone defining “today” are unspecified. [C-4:L124-L149]
- **A06 — C-4 research emitter.** Empty output is accepted because B-7 emission is missing, so a “wired” command can remain permanently empty; producer ownership is deferred outside this plan. [C-4:L110-L115]
- **A07 — C-5 cap units/precedence details.** Warning is checked before the cap, but the exact monthly warning behavior, floating-point rounding, calendar timezone, and daily/monthly block selection when both are exceeded are absent. [C-5:L90-L108]
- **A08 — C-5 cost bounds.** The source gives one formula but does not define how it becomes distinct low/high values or how model prices are sourced/versioned. [C-5:L138-L148]
- **A09 — C-12 prompt naming.** Prompt blocks are singular (`Phase`, `Step`, `BabyStep`) while enum variants are plural (`Phases`, `Steps`, `BabySteps`); mapping is apparent but not normatively declared. [C-12:L68-L72; C-12:L138-L140]
- **A10 — C-12 cascade decoding.** Seven seeded rows (five verdict + two cascade) become exactly five JSON entries with two flags, but merge/correlation rules are not specified. [C-12:L98-L102; C-12:L119-L125]
- **A11 — C-19 Criterion assertions.** Criterion ordinarily measures rather than directly asserts p50; the source does not define the machine-readable extraction/gate implementation or benchmark hardware class. [C-19:L98-L107]
- **A12 — C-19 placeholder metrics.** Task text permits numeric zero placeholders labeled `pending_bench`, while acceptance requires numeric values; it does not say whether zero placeholders pass DONE. [C-19:L108-L114; C-19:L295-L298]
- **A13 — C-27 fingerprint pin.** Source/loading of `system_fingerprint_pin` and behavior on a mismatch are not defined; only the `None→true` case is. [C-27:L73-L82]
- **A14 — C-27 drift length mismatch.** Comparator uses zipped sequences and says “first differing index”; behavior when one sequence is a strict prefix of the other is absent. [C-27:L84-L88]
- **A15 — C-27 contract grammar.** Markdown parsing is named, but escaping, multiline values, multiple blocks, malformed-block recovery, and exact `WiringAuditError` schema are unspecified. [C-27:L128-L136]
- **A16 — C-42 policy ontology.** Only an example state/actor annotation is shown; the complete FSM-state and actor vocabularies and state normalization are not defined. [C-42:L60-L63]
- **A17 — C-42 corpus count check.** The deterministic command counts files, while the requirement counts tuples; a file may contain one or many tuples. [C-42:L70-L73; C-42:L151-L152]
- **A18 — C-43 guard implementation.** The proposed `#[allow(...)]` spelling resembles Rust's built-in lint attribute, while the Files section calls it a proc macro. How the system detects a path with no attribute at all is not described. [C-43:L75-L81; C-43:L181-L188]
- **A19 — C-43 harness statistics.** N, benchmark sampling, confidence/significance, seed handling, and the aggregate-delta formula are not defined. [C-43:L83-L87]
- **A20 — C-50 contract semantics.** “Checks that conditions hold on paths” lacks a solver/runtime model, supported predicate subset, side-effect policy, and retry limit/exhaustion outcome. [C-50:L46-L60]
- **A21 — C-50 artifact path.** `{file_path}` may contain separators or absolute paths; escaping/collision and project-root containment are not specified. [C-50:L52-L55]
- **A22 — C-57 official scoring fallback.** OpenHands version/task snapshot, SWE-Cycle adapter protocol, and precise behavior when SWE Atlas's official judge/corpus is unavailable remain open. [C-57:L41-L57; C-57:L67-L71]
- **A23 — C-57 cost proof.** “Less than three independent run cost” has no comparator measurement method, pricing snapshot, or treatment of failed/retried tasks. [C-57:L120-L132]

## Explicit conflicts and stale clauses

| ID | Conflict | Evidence and disposition |
|---|---|---|
| C01 | C-4 says C-1 supplies `AgentLifecycle` emitters, but C-1 defines only seven knowledge/reasoning events and never lists `AgentLifecycle`. | [C-4:L7-L9] versus [C-1:L3-L6; C-1:L56-L72]. Dependency claim needs another producer or a corrected prerequisite. |
| C02 | C-5 shipped provider flag versus stale plan clauses. | Status and amended Task 3 say `--provider`; `WIRING_CONTRACT`, acceptance, Files, and smoke still say `--mode cloud|hybrid|local`. The status block explicitly says do not re-implement the stale spelling, so `--provider` is canonical. [C-5:L19-L22; C-5:L110-L119] versus [C-5:L217-L229; C-5:L293-L294; C-5:L308-L308; C-5:L329-L329]. |
| C03 | C-5 shipped atomic warning/cost events versus stale generic command payloads. | Status says `WarnLimitReached` and `CostEstimate` are atomic variants; Tasks 2/6, WIRING observable, and acceptance still call them `COMMAND` rows. Status explicitly overrides. [C-5:L28-L30] versus [C-5:L101-L108; C-5:L145-L148; C-5:L239-L240; C-5:L291-L300]. |
| C04 | C-5 component path and privacy timeline are stale later in the same file. | Canonical shipped path is `lib/screens`, not `lib/components`; timeline is deferred. Acceptance and Files still require a redaction feed and `lib/components`. [C-5:L23-L37; C-5:L165-L188] versus [C-5:L216-L217; C-5:L302-L303; C-5:L317-L318; C-5:L332-L334]. |
| C05 | Provider mode used by multiple smoke commands conflicts with C-5's corrected CLI axis. | C-1/C-4/C-12/C-19/C-27 use `elai /task --mode cloud`, while C-5 says `--mode` is gate mode and provider is selected by `--provider`. [C-1:L228-L232; C-4:L195-L196; C-12:L152-L155; C-19:L236-L238; C-27:L201-L202] versus [C-5:L19-L22; C-5:L110-L119]. |
| C06 | SSB schema baselines cannot all be current. | C-27 mandates 46→47, C-1 says one bump and ≥47, while C-5 reports shipped schema 103. Integration must rebase to the actual current version rather than set 47. [C-27:L61-L71; C-27:L210-L211] versus [C-1:L56-L60; C-1:L210-L222] versus [C-5:L16-L18]. |
| C07 | C-12 describes `get_per_level_verdicts` both as a live reader with no producer rows and as a literal `Vec::new()` stub. | [C-12:L26-L31] versus [C-12:L51-L55; C-12:L98-L102]. Task 5/acceptance clearly require replacing the literal stub. |
| C08 | C-19 test command/file mismatch. | New phase-4A tests are placed in `adversarial_ipc.rs`, but observable/acceptance invoke `--test sandbox_tests` and expect the 15 additions there. [C-19:L62-L64; C-19:L220-L235] versus [C-19:L305-L307]. |
| C09 | C-27 CLI type/file ownership is inconsistent. | Task 9 locates `WiringAuditCmd` in `crates/cli/src/wiring_audit.rs`; the Files list says `wiring_contract.rs` also contains `WiringAuditCmd/Report/Error`. [C-27:L128-L136] versus [C-27:L293-L298]. |
| C10 | C-42 dashboard status conflicts with its acceptance. | Task/WIRING/Files label `AuthzPanel.svelte` PLANNED, but acceptance requires it to render. [C-42:L105-L107; C-42:L114-L119; C-42:L189-L200] versus [C-42:L186-L187]. |
| C11 | C-42 explicit-authz completion has two gates. | Task 9/deterministic check require every tool to have an annotation; acceptance permits annotation **or** DENY-ALL fallback. [C-42:L99-L103; C-42:L157-L160] versus [C-42:L173-L176]. Runtime fail-closed behavior is compatible, but DONE criteria differ. |
| C12 | C-43's “three layers” are defined differently. | Objective's parenthetical names auditor approval + MSI + Cosmos, while Context names mistake-driven modification + auditor + MSI, with Cosmos additional. [C-43:L3-L5] versus [C-43:L25-L33]. |
| C13 | C-43 says no behavioral runtime change but adds runtime event behavior. | [C-43:L10-L12] versus `self_mod_layer_proposed` runtime warning and new SSB events [C-43:L89-L91]. |
| C14 | C-43 missing-attribute enforcement is internally inconsistent. | The macro allegedly emits a warning, yet a path lacking the macro cannot execute it; separate clippy-as-error detection is required but unspecified. [C-43:L75-L81; C-43:L93-L99]. |
| C15 | C-50 benchmark target differs between TDD task and final acceptance. | Task 7 says reproduce 47% functional and 95%+ structural; acceptance relaxes to ≥40%/≥90%. [C-50:L69-L75] versus [C-50:L120-L122]. Final acceptance is the stated done gate, but the TDD test wording remains stronger. |
| C16 | C-57 prerequisite and parallel statement conflict operationally. | C-54 must be shipped and its number exist first, yet C-57 is called parallel-safe with C-54 while runs are sequential on the same RunPod. [C-57:L8-L10]. Planning can be parallel; the actual benchmark run cannot satisfy “C-54 first” concurrently. |
| C17 | C-4 “Next” misidentifies C-5. | It calls C-5 a ReviewWindow diff-wiring plan, but the processed C-5 is Cloud Mode Hardening. [C-4:L295-L298] versus [C-5:L1-L5]. |
| C18 | C-43 source-grounding asks for verbatim ablation material but does not give a quotation limit/provenance format. | This is a source-internal compliance risk, not a feature change. [C-43:L61-L73; C-43:L126-L131]. |

## Cross-source interaction map

1. **Evidence production and consumption:** C-1 creates reasoning/knowledge SSB evidence and live commands; C-4 exposes orchestration/spend readers; C-5 consumes C-4 spend data to decide routing and adds hardening events/UI; C-12 adds auditor lifecycle rows and a live timeline. Their observable loop is producers → SSB → viewer/Tauri → dashboard, with CLI smoke as the trigger. [C-1:L35-L37; C-4:L140-L149; C-5:L90-L99; C-12:L98-L107]
2. **Testing-trust overlay:** C-19 supplies adversarial/performance evidence. C-27 then makes every plan's `WIRING_CONTRACT` machine-auditable through stub, reachability, and observable-effect gates; its SSB rows become audit evidence rather than product output. [C-19:L194-L242; C-27:L128-L160]
3. **Security chain:** C-19 verifies IPC/sandbox/authz-adjacent boundaries; C-42 adds fail-closed, state-aware authorization at the MCP gateway and gates later C-40/C-44 consumers. [C-19:L62-L94; C-42:L99-L103; C-42:L213-L219]
4. **Self-modification evidence loop:** C-43 freezes Phase 4H and requires ablation evidence; C-57's SWE-Cycle specifically exercises MSI/bubble-up across repeated outputs and therefore can contribute relevant evidence, though the plans do not automate this handoff. [C-43:L65-L87; C-57:L28-L31]
5. **Completeness and feasibility:** C-50 layers behavioral contracts on Phase 4F/C-31/C-32; C-19 separately backfills Phase-4F feasibility adversarial/contract tests. The former changes generated-code gates, while the latter validates feasibility/sandbox machinery. [C-50:L18-L32; C-19:L135-L175]
6. **Benchmark dependency chain:** C-54 creates the primary number, C-44 supplies C-57's three-tier backend, C-57 creates supplementary multi-axis evidence, and C-59 later aggregates it. [C-57:L8-L10; C-57:L95-L97; C-57:L152-L157]
7. **Shared serialization hotspot:** Every new event set must be merged through the same SSB enum/string/payload/schema registry and tested for bijection/round-trip; this makes claimed plan-level parallelism conditional on serial schema integration. [C-1:L56-L72; C-5:L16-L18; C-27:L61-L71; C-42:L87-L89; C-43:L89-L91; C-50:L66-L67; C-57:L84-L85]

## Summary additions and corrections

These deltas are against `03-C-SUMMARY_INDEX.discovery.md` only; source anchors below remain the authority.

### C-1 deltas

- **Addition:** The index names the seven events but omits all seven exact payload schemas, SQL serialization/bijection work, optional `PipelineGrc` signature, 11 caller updates, dual `ReasoningTrace` emission, cache counter reset, query hashing, and conditional engine-writer choice. [C-1:L56-L110]
- **Addition:** The live-query semantics include `blocked=true` for CSQ, real viewer rows for trace/grounding, and three exact `unwired` flags. [C-1:L112-L120]
- **Correction/precision:** “Five contract tests” means five newly added contract files; the source also requires two existing-suite extensions, for seven named test changes. [C-1:L257-L278]
- **Addition:** Smoke distinguishes two always-required events from a conditional rubber-stamp event and requires real positive GRC budget; the index's ≥5/non-null shorthand omits this. [C-1:L228-L254]
- **Addition:** Seven explicit non-goals prevent mistakenly absorbing C-2/C-3/C-4 and feasibility/documentation work. [C-1:L280-L288]

### C-4 deltas

- **Correction:** The index claims closure of B-26 and B-36. The source names B-6, B-7, B-27, phase-4K, and phase-4M as shipped/deferred surface origins; B-36 appears only as an AppState pattern, and B-26 is absent. [C-4:L14-L54]
- **Addition:** The index omits C-2 as the BabyStepSpec prerequisite, structured `PhaseNotFound`/`BabyStepNotPersisted`, research-history empty fallback, ordered pipeline shape, and project temp-file/rename behavior. [C-4:L7-L10; C-4:L89-L130]
- **Addition:** Cloud spend is not merely “live”: it has latest-task, daily sum, cache token totals, and redaction-count fallback semantics. [C-4:L140-L149]
- **Addition:** The source's smoke has five observable assertions, not only non-IDLE agent and nonzero spend. [C-4:L195-L201]
- **Correction:** C-4's own “Next: C-5 ReviewWindow” does not match the actual C-5 source and should not orient sequencing. [C-4:L295-L298; C-5:L1-L5]

### C-5 deltas

- **Critical correction:** The index says CLI `--mode cloud|hybrid|local`; the amended shipped source says `--provider`, with `--mode` reserved for gate mode. [C-5:L19-L22; C-5:L110-L119]
- **Addition:** C-5 is explicitly shipped, with schema 103, three atomic variants, bijection counts, and a decision log. [C-5:L14-L40]
- **Correction:** Warning and estimate are shipped `WarnLimitReached`/`CostEstimate` variants, not generic COMMAND payloads; the index omits both event types. [C-5:L28-L31]
- **Addition:** Exact LOC semantics are local 3B=25, local 7B=40, cloud Sonnet=300, cloud Opus=500; the index explicitly lacked these labels. [C-5:L128-L136]
- **Addition:** The cost formula, ungrounded heuristic label, fail-open-on-spend-read-error behavior, endpoint privacy rule, and warning-before-cap/once-per-day semantics are absent from the index. [C-5:L101-L108; C-5:L138-L156; C-5:L230-L235]
- **Correction:** CloudPanel lives under `lib/screens`; its detailed last-five redaction timeline is deferred. The index's unqualified panel/redaction-feed wording overstates completion. [C-5:L23-L37; C-5:L165-L188]
- **Addition:** Banner precedence, `get_cloud_route_status`, one-escalation halt behavior, and `airGapped` selector disabling are explicit source behaviors. [C-5:L31-L37; C-5:L120-L126; C-5:L173-L181]

### C-12 deltas

- **Addition:** The index omits the sixth Cosmos prompt block, singular-prompt/plural-enum naming, typed tier IDs, non-aborting emit failure, ESCALATE-on-record-error, monotonic no-double-respawn guard, and unframed warning/fallback. [C-12:L68-L96; C-12:L147-L150; C-12:L208-L222]
- **Addition:** The command merges verdict and cascade action types into five JSON entries; the two exact contract fixtures and started-before-verdict ordering are stronger than the summary. [C-12:L98-L125]
- **Correction/ambiguity:** The index says the command changes from empty vector to live data, which matches Tasks, but the source Context also incorrectly describes it as already reading live rows. Treat the literal-stub replacement as canonical. [C-12:L26-L31; C-12:L51-L55; C-12:L98-L102]

### C-19 deltas

- **Addition:** The index collapses the 15 IPC cases; the source provides exact function names and distinct allocation, peer-credential, budget, canonicalization, gating, dedup, ordering, and partial-frame assertions. [C-19:L60-L94]
- **Addition:** The benchmark contract includes Criterion 0.5 lockfile verification, four groups, exact T2 boundary sizes, Ruff simulation, and HTML artifacts. [C-19:L96-L107]
- **Addition:** All nine metrics and their fields, plus the conditional `pending_bench` placeholder path, are explicit. [C-19:L108-L114]
- **Addition:** The source specifies all A1-A8, T9/T13-T16, four baby-step, four research, and four SessionDag/BranchTree behaviors, including paused-time and 30-second anti-hang constraints. [C-19:L121-L192]
- **Correction:** The index/plan shorthand “15 sandbox tests” masks a command mismatch: additions live in `adversarial_ipc.rs`, while later smoke invokes `sandbox_tests`. [C-19:L62-L64; C-19:L234-L235; C-19:L305-L307]

### C-27 deltas

- **Addition:** Exact payload fields, error variants, environment defaults, keyed-hash construction, equality/no-row rule, first-divergence behavior, and test-path exclusions are missing from the index. [C-27:L61-L119]
- **Addition:** Wiring contracts have a concrete field schema; CLI parses Markdown blocks, prints exact PASS/FAIL forms and exit codes; CI uses Windows and `[ci skip]`; observables poll at one-second cadence and fail with typed evidence. [C-27:L90-L148]
- **Addition:** Acceptance uniquely adds a schema-46 migration warning and ≥10 new unit functions. [C-27:L253-L285]
- **Correction/context:** The index faithfully reports 46→47 from this source, but that literal is stale against C-5's shipped schema 103 and must be rebased during integration. [C-27:L61-L71; C-5:L14-L18]

### C-42 deltas

- **Addition:** The index omits the explicit threat model, feature-gated/test-only crate constraint, complete five-shape tuple model, localhost triple record, structured rejection fields, PR-comment artifact, and C-40/C-44 P0 release gate. [C-42:L21-L45; C-42:L65-L85; C-42:L127-L140; C-42:L213-L219]
- **Addition:** Severity semantics are CRITICAL for unauthorized authorization, P1 latent bug for false rejection, and BLOCK for rejection-shape failure. [C-42:L77-L82]
- **Correction:** `AuthzPanel.svelte` is PLANNED, not unconditionally added; acceptance and summary overstate its current status. [C-42:L105-L119; C-42:L186-L200]
- **Precision:** Fail-closed registration does not fail startup; it emits a structured warning and denies use. [C-42:L99-L103; C-42:L127-L133]

### C-43 deltas

- **Addition:** The index omits exact adoption gates: two published benchmarks, strict ≥+3pp same-fixture gain, and per-project non-default opt-in. [C-43:L65-L73]
- **Addition:** Current scope includes mistake-driven prompt mutation and a Cosmos tier, with only MSI labeled narrow/high-confidence; the phrase “three-layer” is inconsistent inside the source and should not be flattened. [C-43:L25-L42]
- **Addition:** Tests require baseline grandfathering, a negative DGM fixture, pre-return SSB, mistakes entry, CI lint failure, and a concrete evidence table before expansion. [C-43:L93-L110; C-43:L126-L177; C-43:L197-L203]
- **Correction/precision:** The index calls this proc-macro enforcement; the source's proposed `#[allow(...)]` mechanism and detection of missing attributes are underspecified and internally tensioned. [C-43:L75-L81; C-43:L181-L188]

### C-50 deltas

- **Addition:** The index omits the exact `ContractSpec`/`Predicate`/`ContractLang` schema, supported Rust/Python/TS encodings, tree-sitter parse-or-reject rule, TOML artifact path, AST entry/return-path check, and existing-project opt-in. [C-50:L46-L64]
- **Addition:** The event-before-retry ordering and no-permissive-fallback bans are explicit. [C-50:L95-L99]
- **Correction/precision:** Summary's ≥40%/≥90% acceptance is correct, but the TDD task itself says 47%/95%+. Both thresholds must be preserved until the intended test gate is resolved. [C-50:L69-L75; C-50:L120-L122]

### C-57 deltas

- **Addition:** The index omits exact default composition (Sonnet 4.6/Composer 2/local auditor), optional local-only arm, pass@1/no-cherry-picking discipline, CLI forms, external-baseline citation rule, and RunPod cost gate. [C-57:L36-L39; C-57:L59-L65; C-57:L98-L105; C-57:L120-L132]
- **Addition:** Each adapter performs load/present/collect/official-score, and outputs must expose OpenHands axes, SWE-Cycle's curve, and SWE Atlas rubrics rather than aggregates. [C-57:L52-L82]
- **Addition:** C-44's three backend tiers are an additional consumed dependency; C-59 is the downstream aggregator. [C-57:L95-L97; C-57:L152-L157]
- **Correction/precision:** “Parallel-safe with C-54” applies at most to preparation; the benchmark run requires C-54's number first and shares one sequential RunPod. [C-57:L8-L10]

## Zero-omission and zero-duplicate audit

### Coverage accounting

| Source | Verified lines read | Task coverage | Explicit atomic records | Ambiguity/conflict links |
|---|---:|---|---:|---|
| C-1 | 1-288 (288) | Tasks 1-14; WIRING, 11 deterministic checks, acceptance, 7 named test changes, all out-of-scope bullets | 28 | A01-A03; C01, C05-C06 |
| C-4 | 1-298 (298) | Tasks 1-11; all ten commands, fallbacks, WIRING, 9 deterministic checks, acceptance/files/Next | 20 | A04-A06; C01, C07, C17 |
| C-5 | 1-345 (345) | Status amendment + Tasks 1-10; WIRING, 9 checks, acceptance, all deferred/drift clauses | 28 | A07-A08; C02-C06 |
| C-12 | 1-256 (256) | Tasks 1-8; five tiers/four outcomes/six prompts, WIRING, 10 checks, acceptance/files | 19 | A09-A10; C05, C07 |
| C-19 | 1-328 (328) | Tasks 1-9; every one of 15+4+8+5+4+4 named tests, 4 benches, 9 metrics, WIRING/checks/acceptance | 50 | A11-A12; C08 |
| C-27 | 1-316 (316) | Tasks 1-13; all types/fields/defaults/auditors/CLI/CI/tests/decisions, WIRING/checks/acceptance | 28 | A13-A15; C06, C09 |
| C-42 | 1-219 (219) | Tasks 1-11; policy/corpus/severities/events/TDD/CI/panel/contract/release gate | 20 | A16-A17; C10-C11 |
| C-43 | 1-203 (203) | Tasks 1-8; policy criteria/guard/harness/events/TDD/docs/CI/non-goals | 18 | A18-A19; C12-C14, C18 |
| C-50 | 1-142 (142) | Tasks 1-8; schemas/languages/emitter/critic/retry/config/events/TDD/thresholds | 17 | A20-A21; C15 |
| C-57 | 1-157 (157) | Tasks 1-8; adapters/config/run discipline/scoring/artifacts/events/CLI/cost/status | 18 | A22-A23; C16 |
| **Total** | **2,552** | **All numbered tasks and all terminal sections** | **246 explicit records** | **10 inferred, 23 ambiguous, 18 conflicts** |

### Omission audit

- All ten manifest paths were read from line 1 through their verified final line; no other original C-plan chunk was opened.
- Every numbered Task block is represented above. Every closed set is exhaustively enumerated: C-1's 7 actions/7 payloads/7 test changes; C-4's 10 commands/6 absence tests/5 smoke observations; C-5's 3 shipped variants/4 LOC constants/3 suites; C-12's 6 prompt modes/5 tiers/4 outcomes/2 contracts; C-19's 15+4+8+5+4+4 tests, 4 benchmarks, and 9 metrics; C-27's schemas/auditors/CLI/CI/tests; C-42's 5 tuple shapes/3 severities/3 events/5 TDD cases; C-43's 3 decision gates/4 TDD cases/2 events; C-50's 3 languages/3 events/5 TDD cases; C-57's 3 adapters/3 scorecards/3 events/3 scoring families.
- Every source's dependency/status/deferred, `WIRING_CONTRACT`, observable effect, deterministic check, acceptance, file-impact, and non-goal/Next material was either merged into an explicit record or captured in A/C/delta sections.

### Duplicate audit

- Task, WIRING, deterministic-check, acceptance, and Files repetitions are intentionally represented once per semantic requirement with multiple anchors.
- Similar but non-identical requirements remain separate: C-1 trace emission vs trace Tauri read; C-5 warning vs route-block vs estimate; C-12 emit-start vs record-verdict; C-19 boundary/race/fallback variants; C-27 evidence vs violation vs drift; C-42 runtime authz events vs offline finding; C-50 emit/check/retry; C-57 axis completion vs scorecard completion.
- Summary-index statements are not counted as features. They appear only as additions/corrections, preventing derived-summary duplication from inflating the **246** explicit-record count.

