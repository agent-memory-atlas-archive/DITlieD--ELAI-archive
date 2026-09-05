# C-4: Tauri Commands — Orchestration + Cloud Surface Completion
Objective: wire the missing/stubbed Tauri commands that expose orchestration
state and cloud-spend data to the dashboard. Every targeted command already
has its SSB viewer query method or equivalent SSB payload defined; this plan
adds the Tauri surface, removes hardcoded stubs, and adds contract tests.
Estimated effort: 1 session.
Pre-requisites: C-1 (SSB agent-lifecycle emitters ship first, provides the
AgentLifecycle rows that get_agents reads); C-2 (BabyStepSpec persistence
ships first, provides rows that get_baby_step_detail reads).
Parallel-safe with: C-3 (different commands, no shared symbols).

## Context

B-6, B-7, B-27, phase-4K, and phase-4M all shipped backend infrastructure
but deferred the Tauri surface layer. The resulting dashboard gaps are:

- get_agents() (commands.rs:1655) returns a hardcoded 8-role IDLE list.
  SSB already emits AgentLifecycle rows from bsp/engine.rs (242, 264, 353,
  509, 580) but ssb_viewer.rs has no query method for them. B-6 report
  confirms: "ACCEPTANCE criterion 'No always IDLE agent statuses' is not met."

- get_phase_detail / get_baby_step_detail exist only in the archived V1
  crate (.bkp_old/); ssb_viewer.rs has phase_detail() (line 1872) and
  baby_step_detail() (line 1916) but V2 commands.rs exposes neither.
  baby_step_detail() returns NotFound for every call.

- get_executable_pipeline (commands.rs:2046) returns Ok(json!([])). The
  SSB viewer has decomposition_tree() wired; executable pipeline rows
  require reading concept_decomposition_node + phase_node rows in order.

- get_research_node_history is entirely absent from V2 commands.rs.
  ResearchNodeStage type exists in types.ts frontend (line 809) but no
  Tauri command feeds it.

- get_phase_progress / get_recent_actions / get_file_changes / get_action_detail
  have full ssb_viewer.rs implementations (lines 990, 1019, 1353, 1511) but
  are not registered as Tauri commands in commands.rs or lib.rs. phase-4K
  report confirms all four are PARTIAL for this reason.

- list_projects / create_project (phase-4K.3) — only open_project_from_folder
  covers the "open existing" path. No server-side project list or creation.

- list_runtimes / test_runtime / set_role_runtime (commands.rs:6778, 6863,
  6922) are REAL implementations (B-37 shipped them). phase-4M report confirms
  they are wired to stub.ts in the Svelte frontend, not real invoke() calls.
  RuntimePanel.svelte exists (167 LOC) but polls stub data.

- get_cloud_spend (commands.rs:3305) returns hardcoded zeros; comment at
  line 3306 documents the deferred SsbViewer::cloud_spend_summary(). The
  CLOUD_SPEND SSB action type and payload are fully defined (ssb.rs:694,
  1880). B-27 report confirms: "SsbViewer::cloud_spend_summary() missing."

- get_active_agents naming (B-6) — plan said get_active_agents; shipped
  name is get_agents. No rename needed; WIRING_CONTRACT note resolves this.

## Reads

- .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs (stubs at lines
  1655, 2046, 3305)
- .elai_cc/crates/orchestrator/src/ssb_viewer.rs (phase_detail:1872,
  baby_step_detail:1916, phase_progress:990, recent_actions:1019,
  file_changes:1353, get_action_detail:1511, decomposition_tree:1758)
- .elai_cc/crates/orchestrator/src/ssb.rs (AgentLifecycle:689,
  CLOUD_SPEND:694, AgentLifecyclePayload:1844, CloudSpendPayload:1880)
- .elai_cc/crates/orchestrator/src/bsp/engine.rs (AgentLifecycle emit
  sites: 242, 264, 353, 509, 580)
- .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (invoke_handler)
- .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts (frontend stubs for
  list_runtimes/test_runtime/set_role_runtime)
- .elai_cc/crates/dashboard-v2/src/lib/components/RuntimePanel.svelte
- reports: B-6, B-7, B-27, phase-4K, phase-4M (docs/thefinalglance/verification/reports/)

## Tasks

1. **SsbViewer::recent_agent_lifecycle_rows(limit: u32)**
   Add to ssb_viewer.rs. Query: SELECT action_type, payload FROM ssb WHERE
   action_type = 'agent_lifecycle' ORDER BY seq DESC LIMIT ?. Deserialize
   each row into AgentLifecyclePayload. Return Vec<AgentLifecyclePayload>.
   This is the sole missing reader that unblocks get_agents live data.

2. **get_agents() — replace hardcoded IDLE list**
   In commands.rs:1655, call viewer.recent_agent_lifecycle_rows(200). Group
   by role; pick the latest row per role (max seq). Map AgentLifecycle state
   field to lifecycle string. Populate pid, last_heartbeat_ms from payload
   fields. Fall back to IDLE only when SSB has no rows for that role (empty
   state before any task runs). This satisfies B-6 ACCEPTANCE: "No always
   IDLE agent statuses."

3. **get_phase_detail(phase_id: String) Tauri command**
   Add pub fn get_phase_detail to commands.rs forwarding to
   viewer.phase_detail(&phase_id). Register in lib.rs invoke_handler.
   phase_detail() at ssb_viewer.rs:1872 already performs a real SQL read.

4. **get_baby_step_detail(baby_step_id: String) Tauri command + fix NotFound**
   Two parts: (a) add the Tauri command in commands.rs forwarding to
   viewer.baby_step_detail(&baby_step_id); (b) fix baby_step_detail() in
   ssb_viewer.rs to query the SSB for baby_step_ready or
   concept_decomposition_node rows filtered by baby_step_id, rather than
   returning NotFound unconditionally. Pre-requisite C-2 provides the write
   side. If C-2 is not yet shipped, the command returns a structured
   BabyStepNotPersisted error (not a panic, not a stub Ok).

5. **get_executable_pipeline(session_id: String) Tauri command — replace stub**
   commands.rs:2046 returns Ok(json!([])). Replace with a call to
   viewer.decomposition_tree() filtered to action-type concept_decomposition_node,
   then re-shaped as an ordered list of executable steps (phase -> step ->
   baby-step). The viewer method already exists (ssb_viewer.rs:1758); the
   command just needs to serialize the tree leaves as an ordered Vec<Value>.

6. **get_research_node_history(session_id: String, node_id: String) Tauri command**
   Add to commands.rs. Query SSB for research_node_stage rows (action_type
   'research_node_stage') filtered by session_id and node_id. Deserialize
   to Vec<ResearchNodeHistoryEntry> matching the types.ts ResearchNodeStage
   shape. Register in lib.rs. Note: if research_node_stage SSB rows are
   absent (B-7 confirms emitter is missing), return empty Vec — no panic.

7. **get_phase_progress() / get_recent_actions() / get_file_changes() /
   get_action_detail() — expose as Tauri commands**
   Four thin wrappers, each forwarding to the matching ssb_viewer.rs method
   (990, 1019, 1353, 1511 respectively). Add #[tauri::command] pub fn
   declarations in commands.rs. Register all four in lib.rs invoke_handler.
   Export get_action_detail from stub.ts (phase-4K gap item).

8. **list_projects() / create_project() Tauri commands**
   list_projects: read <project_root>/projects.json if it exists; parse as
   Vec<ProjectEntry> {name, path, last_opened_ms}; return it. If the file is
   absent return empty Vec. create_project(name: String, parent_path: String):
   create_dir_all(parent_path/name); write stub elai_cc.toml; append entry to
   projects.json (atomic write via temp-file + rename matching runtime_cmd.rs
   pattern). Register both in lib.rs.

9. **Wire list_runtimes / test_runtime / set_role_runtime in frontend**
   In stub.ts, replace the three stub function bodies with real tauri invoke()
   calls: invoke('list_runtimes'), invoke('test_runtime', {backend}),
   invoke('set_role_runtime', {role, backend}). Mount RuntimePanel.svelte in
   SettingsWindow.svelte if not already present (phase-4M gap: "runtime_panel_added
   = false"). The Tauri commands themselves are already real (commands.rs:6778,
   6863, 6922 verified done by phase-4M report).

10. **SsbViewer::cloud_spend_summary() + get_cloud_spend() real implementation**
    Add cloud_spend_summary() to ssb_viewer.rs. Query:
    (a) task_usd — SELECT payload FROM ssb WHERE action_type = 'cloud_spend'
        ORDER BY seq DESC LIMIT 1, extract cost_usd.
    (b) day_usd — SUM json_extract(payload,'$.cost_usd') for today's rows.
    (c) Aggregate cache_creation_tokens, cache_read_tokens across today's rows.
    (d) redaction_count — query RedactionStat rows if defined, else 0.
    In commands.rs:3305 replace _state with state; call
    state.viewer.cloud_spend_summary() and serialize. Satisfies B-27 action
    item: "implement SsbViewer::cloud_spend_summary()."

11. **Contract tests**
    Add tests/c4_tauri_commands_contract.rs covering:
    - recent_agent_lifecycle_rows returns empty Vec on empty SSB (not panic).
    - cloud_spend_summary returns zero totals on empty SSB (not panic).
    - get_phase_detail returns PhaseNotFound error (not panic) when phase absent.
    - get_baby_step_detail returns BabyStepNotPersisted error when absent.
    - get_executable_pipeline returns empty Vec when no decomposition rows.
    - get_research_node_history returns empty Vec when no stage rows.
    Run: cargo test -p elai_cc_contracts -- c4_tauri 2>&1.

## WIRING_CONTRACT

produces_symbols:
  - SsbViewer::recent_agent_lifecycle_rows(limit: u32) -> Vec<AgentLifecyclePayload>
  - SsbViewer::cloud_spend_summary() -> CloudSpendSummary
  - Tauri commands: get_phase_detail, get_baby_step_detail, get_executable_pipeline,
    get_research_node_history, get_phase_progress, get_recent_actions,
    get_file_changes, get_action_detail, list_projects, create_project
  - get_agents() — live AgentLifecycle rows (replaces hardcoded IDLE list)
  - get_cloud_spend() — real SSB aggregation (replaces hardcoded-zero stub)
consumes_symbols:
  - SsbViewer (ssb_viewer.rs) — existing phase_detail, baby_step_detail,
    phase_progress, recent_actions, file_changes, get_action_detail,
    decomposition_tree methods
  - AgentLifecycle SSB action type + AgentLifecyclePayload (ssb.rs:689,1844)
  - CLOUD_SPEND SSB action type + CloudSpendPayload (ssb.rs:694,1880)
  - RuntimeOrchestrator (commands.rs:6778,6863,6922 — already real; frontend
    stub.ts is the only gap)
  - C-1 AgentLifecycle emitters (pre-req); C-2 BabyStepSpec persistence (pre-req)
reachable_from:
  - dashboard-v2 frontend via tauri invoke() calls
  - RuntimePanel.svelte (list_runtimes/test_runtime/set_role_runtime)
  - MonitorWindow.svelte (get_cloud_spend)
  - OrchestrationDrawer.svelte (get_agents, get_phase_progress, get_recent_actions)
  - PlanWindow.svelte (get_phase_detail, get_baby_step_detail, get_executable_pipeline)
stub_ban:
  - get_agents MUST NOT return a hardcoded JSON literal after Task 2 lands
  - get_cloud_spend MUST NOT return hardcoded zero fields after Task 10 lands
  - get_executable_pipeline MUST NOT return Ok(json!([])) after Task 5 lands
  - cloud_spend_summary MUST query SSB, not return Default::default()
frontend_pair:
  - RuntimePanel.svelte (list_runtimes/test_runtime/set_role_runtime invoke wiring)
  - MonitorWindow.svelte cloud spend gauges
  - OrchestrationDrawer.svelte agents panel
observable_effect:
  trigger: elai /task --mode cloud "implement a todo CRUD module in src/todo.rs with 4 functions"
  expect: get_agents returns >=1 entry with lifecycle != IDLE during execution
  expect: get_phase_progress returns multi-phase decomposition (>=2 phases)
  expect: get_cloud_spend returns task_usd > 0.0 after Sonnet 4.6 call completes
  expect: get_recent_actions shows ToolUse rows in response body
  expect: get_file_changes shows src/todo.rs with action=FileWrite

## DETERMINISTIC_CHECK

```bash
# 1. recent_agent_lifecycle_rows exists in ssb_viewer
rg "fn recent_agent_lifecycle_rows" .elai_cc/crates/orchestrator/src/ssb_viewer.rs

# 2. cloud_spend_summary exists in ssb_viewer
rg "fn cloud_spend_summary" .elai_cc/crates/orchestrator/src/ssb_viewer.rs

# 3. get_agents no longer returns hardcoded IDLE literal
rg "lifecycle.*IDLE.*hardcoded\|always IDLE" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
# Expected: zero matches

# 4. get_cloud_spend no longer returns hardcoded zeros
rg "task_usd.*0\.0.*hardcoded\|TODO.*B-27\.5" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
# Expected: zero matches

# 5. New Tauri commands present in commands.rs
rg "pub fn get_phase_detail\|pub fn get_baby_step_detail\|pub fn get_executable_pipeline\|pub fn get_research_node_history\|pub fn get_phase_progress\|pub fn get_recent_actions\|pub fn get_file_changes\|pub fn get_action_detail\|pub fn list_projects\|pub fn create_project" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs | wc -l
# Expected: 10

# 6. All 10 new commands registered in invoke_handler
rg "get_phase_detail\|get_baby_step_detail\|get_executable_pipeline\|get_research_node_history\|get_phase_progress\|get_recent_actions\|get_file_changes\|get_action_detail\|list_projects\|create_project" .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs | wc -l
# Expected: >=10

# 7. RuntimePanel frontend stubs replaced with real invoke
rg "invoke.*list_runtimes\|invoke.*test_runtime\|invoke.*set_role_runtime" .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts
# Expected: 3 matches

# 8. Contract tests compile and pass
cargo test -p elai_cc_contracts -- c4_tauri 2>&1 | grep "test result"
# Expected: test result: ok

# 9. Cargo check clean
cargo check --workspace 2>&1 | grep "^error" | wc -l
# Expected: 0
```

## ACCEPTANCE

- [ ] SsbViewer::recent_agent_lifecycle_rows compiles; returns empty Vec on
      empty SSB without panic (contract test passes)
- [ ] get_agents returns live AgentLifecycle rows during a cloud task; at
      least one entry has lifecycle != IDLE while task is running
- [ ] get_phase_detail command registered; returns phase data for a known
      phase_id; returns structured PhaseNotFound (not panic) for unknown id
- [ ] get_baby_step_detail command registered; returns BabyStepNotPersisted
      (not panic) when C-2 is not yet shipped; returns real data post-C-2
- [ ] get_executable_pipeline returns ordered step list from SSB; returns
      empty Vec (not panic) when no decomposition rows
- [ ] get_research_node_history registered; returns empty Vec when no stage
      rows; not panic
- [ ] get_phase_progress / get_recent_actions / get_file_changes /
      get_action_detail all registered in lib.rs invoke_handler and callable
      from frontend
- [ ] list_projects returns Vec (possibly empty) from projects.json; no panic
      when file absent
- [ ] create_project creates directory and appends to projects.json (atomic
      write); cargo check passes
- [ ] list_runtimes / test_runtime / set_role_runtime invoke() calls in
      stub.ts replace the stub bodies; RuntimePanel.svelte renders live data
- [ ] SsbViewer::cloud_spend_summary() compiles; returns zero totals on empty
      SSB (contract test passes)
- [ ] get_cloud_spend returns task_usd > 0.0 after a cloud Sonnet 4.6 task;
      MonitorWindow cloud gauge renders non-zero
- [ ] Cloud smoke test passes:
      elai /task --mode cloud "implement a todo CRUD module in src/todo.rs
      with 4 functions" -- then verify all 5 observable_effect assertions
      manually against live dashboard data
- [ ] cargo check --workspace: 0 errors
- [ ] cargo test -p elai_cc_contracts -- c4_tauri: all pass

## Files touched

- MODIFY: .elai_cc/crates/orchestrator/src/ssb_viewer.rs
    (ADD recent_agent_lifecycle_rows, ADD cloud_spend_summary)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
    (REPLACE get_agents stub, REPLACE get_cloud_spend stub,
     REPLACE get_executable_pipeline stub,
     ADD get_phase_detail, get_baby_step_detail, get_research_node_history,
     get_phase_progress, get_recent_actions, get_file_changes,
     get_action_detail, list_projects, create_project)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
    (register all 10 new commands in invoke_handler)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts
    (replace list_runtimes / test_runtime / set_role_runtime stub bodies
     with real tauri invoke() calls)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte
    (mount RuntimePanel if not already present)
- ADD: .elai_cc/crates/contracts/tests/c4_tauri_commands_contract.rs
- CREATE: workflow/reasoning/C-4-decisions.md

## Next

C-5 (ReviewWindow live diff wire — removes UnwiredNote banner, adds
accept_change/decline_change commands) or C-3 if not yet shipped.
