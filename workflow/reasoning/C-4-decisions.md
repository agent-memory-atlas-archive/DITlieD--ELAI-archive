C-4 Decisions — Tauri Commands Orchestration + Cloud Spend Surface
==================================================================

Session: s-32952
Plan: docs/thefinalglance/C-plan/C-4_tauri_commands_orchestration_cloud.md
Context package: workflow/sessions/s-32952/context-package-C4.md
Status: SHIPPED (cargo check + svelte-check + 8/8 C-4 contract tests pass)


QUESTIONS RESOLVED (orchestrator decisions)
-------------------------------------------
Q1 SKIP plan Task 1 + Task 2 (recent_agent_lifecycle_rows + get_agents
   rewrite) — C-2 already shipped the equivalent
   recent_agent_status_changes (ssb_viewer.rs:3474) and live get_agents
   (commands.rs:2134). No duplicate viewer reader added. Cold-start
   fallback inside the live-read path is preserved per stub_ban
   interpretation locked in C-2-decisions.md.

Q2 baby_step_detail partial-populate. The viewer body was rewritten to
   scan recent BabyStepReady SSB rows (C-2 emits these) and match
   baby_step_id by stringified u64. On match, the decomposition tree
   (decomposition_tree()) is walked to derive concept/phase/step names
   from the ancestry ids; task = payload.label; tdd / acceptance /
   estimated_tokens remain None / 0 because no SSB row carries them.
   Returns NotFound when no matching BabyStepReady row exists. The
   Tauri command maps NotFound to `{known: false, baby_step_id}`.

Q3 RuntimeInfo DTO drift (DRIFT-9) — FIXED in C-4. The Rust
   `RuntimeInfo` (commands.rs:7321) now emits:
     - `backend: String`           (display label = canonical)
     - `canonical_name: String`    (added in C-4; matches TS
                                    RuntimeBackendId)
     - `roles: Vec<String>`
     - `health: RuntimeHealth`     (typed, serializes as
                                    `{status, reason?}` — matches
                                    TS RuntimeHealthJson)
     - `is_stub: bool`             (renamed from `stub` via
                                    `#[serde(rename = "is_stub")]`)
     - `latency_ms: Option<u32>`
   `TestReport` was also aligned: now emits `backend` + typed
   `health: RuntimeHealth` alongside the existing ok/latency_ms/error.
   Frontend (types.ts:1818-1849) interface was already correct; the
   Rust side now produces matching JSON.

Q4 projects.json path. Per orchestrator decision: `<app_data_dir>/projects.json`
   resolved via Tauri 2's `AppHandle::path().app_data_dir()`. When the
   resolver fails (non-Tauri context, unlikely), a fallback path under
   `<project_root>/.elai_cc/projects.json` is used so the command does
   not error on degraded environments. `list_projects` returns
   `{known: true, projects: []}` when the file is missing / empty.

Q5 redaction_count source. Returned as 0 today. The orchestrator's
   instruction said to "pull from `redacted` counter in CloudSpend
   payloads (already in ssb.rs:2050)" — verified the actual
   CloudSpendPayload has no `redacted` field (9 fields: provider, model,
   input_tokens, output_tokens, cache_creation_tokens,
   cache_read_tokens, cost_usd, task_id, ts_ms). The wire field is kept
   on `CloudSpendSummary` for forward compatibility; a future
   `redaction_event` SSB action type can populate it without breaking
   the frontend. Documented in ssb_viewer.rs:CloudSpendSummary and in
   cloud_spend_summary's body comment.

DRIFTs from context-package-C4.md
---------------------------------
DRIFT-6 research_node_stage — Task 6 implemented as empty-Vec
   `Ok(Value::Array(Vec::new()))` returning command. C-5 deferral
   documented in the command body comment.

DRIFT-5 / Q2 — baby_step_detail body rewritten (see Q2 resolution).

DRIFT-9 / Q3 — RuntimeInfo + TestReport widened (see Q3 resolution).

Task 6 research_node_stage: deferred per orchestrator instructions
("not yet defined"). get_phase_detail returns a subset (PhaseDetail
DTO from C-2) without it; the wire shape is stable.


CHANGES SHIPPED
---------------
crates/orchestrator/src/ssb_viewer.rs
- ADDED `CloudSpendSummary` DTO (task_usd, day_usd, cache_creation_tokens,
  cache_read_tokens, redaction_count).
- ADDED `SsbViewer::cloud_spend_summary(now_ms: i64) -> Result<CloudSpendSummary>`
  — single SQL pass over recent CloudSpend rows; latest cost_usd =>
  task_usd; today's rows sum into day_usd + token counters.
  `created_at` is stored in *seconds* (per ssb::execute_write), so the
  caller-supplied `now_ms` is converted to a seconds-based day boundary
  internally.
- REWROTE `SsbViewer::baby_step_detail` body — scans BabyStepReady SSB
  rows for the requested id, walks decomposition_tree() for ancestry
  names, returns partial-populated BabyStepDetail. NotFound preserved
  for the "no matching row" path.
- ADDED `BabyStepReadyPayload` import.

crates/orchestrator/src/lib.rs
- Re-exported `CloudSpendSummary` from ssb_viewer.

crates/dashboard-v2/src-tauri/src/commands.rs
- FIXED RuntimeInfo wire-format drift (DRIFT-9): emits canonical_name,
  typed `health: RuntimeHealth`, renamed stub -> is_stub.
- FIXED TestReport: emits backend + typed health alongside ok/error/
  latency_ms.
- REPLACED get_executable_pipeline stub (line ~2566) with live
  decomposition_tree() walk that flattens leaves into ordered
  baby-step rows with concept/phase/step ancestry.
- REPLACED get_cloud_spend stub (line ~3825) with
  viewer.cloud_spend_summary(now_ms). State arg renamed from `_state`
  to `state` per context-package-C4.md DRIFT-3.
- ADDED 9 new Tauri commands at file tail:
    get_phase_detail(phase_id)
    get_baby_step_detail(baby_step_id)
    get_phase_progress()
    get_recent_actions(limit?, offset?)
    get_file_changes(session_id)
    get_action_detail(seq)
    get_research_node_history(session_id, node_id)   # empty-Vec stub
    list_projects(app)
    create_project(app, name, parent_path)
  Each NotFound-bearing command maps to `{known: false, <id>}` so
  the frontend never branches on a thrown error.
- ADDED ProjectEntry DTO + projects_json_path helper.

crates/dashboard-v2/src-tauri/src/lib.rs
- Registered all 9 new commands in invoke_handler under a new
  `// C-4 — ORCHESTRATION + CLOUD SPEND SURFACE` block.

crates/contracts/tests/orchestration_cloud_surface_contract.rs (NEW)
- 8 tests: SCHEMA floor, cloud_spend_summary empty + aggregate,
  baby_step_detail NotFound + partial-populate, phase_detail NotFound,
  empty-Vec views (phase_progress / recent_actions / file_changes /
  decomposition_tree), get_action_detail Validation error.
- All 8 PASS.

NOT TOUCHED (already shipped per DRIFT-1 / DRIFT-8)
----------------------------------------------------
- get_agents — live in C-2 (commands.rs:2134).
- stub.ts list_runtimes/test_runtime/set_role_runtime — already calls
  tryInvoke()/inv() to the Tauri commands.
- SettingsWindow.svelte RuntimePanel mount — already at line 728.
- get_executable_pipeline registration in lib.rs — already present.


COMMAND COUNT
-------------
- 9 NEW Tauri commands registered.
- 2 EXISTING Tauri commands rewritten (get_executable_pipeline +
  get_cloud_spend; both already registered before C-4).
- 1 EXISTING SsbViewer method rewritten (baby_step_detail).
- 1 NEW SsbViewer method (cloud_spend_summary).
- 2 EXISTING DTOs widened (RuntimeInfo + TestReport).
- 1 NEW DTO (CloudSpendSummary).
- 1 NEW DTO inside commands.rs (ProjectEntry).


TEST + CHECK RESULTS
--------------------
- cargo check -p elai-orchestrator: PASS (0 errors)
- cargo check -p elai-dashboard-v2: PASS (0 errors)
- cargo check --workspace: PASS (0 errors)
- cargo test -p elai_cc_contracts --test orchestration_cloud_surface_contract:
  8/8 PASS (33ms total)
- cargo test -p elai_cc_contracts --test c2_ssb_new_variants_contract:
  14/14 PASS (regression check — schema-floor unchanged at 48)
- cargo test -p elai-orchestrator --lib ssb_viewer: 4/4 PASS
- svelte-check (dashboard-v2): 4323 files, 0 ERRORS, 13 WARNINGS
  (all warnings pre-existing, unrelated to C-4 changes)


RESIDUAL CONCERNS
-----------------
1. research_node_stage emitter still absent (DRIFT-6); get_research_node_history
   command is wired-but-empty. Deferred to C-5 per orchestrator
   instruction.
2. redaction_count is always 0 today (Q5 resolution). Frontend MUST
   render this as "—" or hide when zero; do not display "0 redactions"
   misleadingly.
3. baby_step_detail's `task` field uses the truncated BabyStepReady
   payload.label, not the full BabyStepSpec.task. When a true
   BabyStepSpec persistence path ships in C-5+, the partial-populate
   path can promote to full-populate with no DTO change.
4. PhaseDetail semantic fields (description / expected_outcome /
   dependencies / estimated_steps_remaining) remain None because the
   SSB has no PhaseSpec row type. Same C-5+ deferral as baby steps.
5. Day-window boundary for cloud_spend_summary uses *UTC* day, not the
   operator's local day. This is documented behaviour — within +/- 1
   day for any timezone. A future patch can thread the operator's
   IANA tz through if the dashboard surface demands strict local-day
   accuracy.
6. RuntimeInfo wire-format change is a BREAKING CHANGE to any
   downstream consumer that already polled the old `health: String`
   shape. The frontend's TS interface was always declaring the typed
   form (types.ts:1818), so this aligns Rust to TS and unbreaks the
   panel; no Svelte consumer code needed to change.


NEXT
----
C-5: ReviewWindow live diff wire + accept_change/decline_change.
Optional: re-enable a redaction_event SSB action type so
redaction_count populates from real data.
