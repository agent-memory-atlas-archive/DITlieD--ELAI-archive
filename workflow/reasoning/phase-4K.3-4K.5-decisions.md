phase 4K.3 + 4K.4 + 4K.5 decisions (bundled)

What
Bundle implementation of phase 4K.3 (project selector + task input), 4K.4 (workspace shell + plan tree + activity feed), and 4K.5 (diff viewer + intervention response + preview link) in a single execution pass under the 200K context budget.

Locked architectural decisions

Decision 1 — SSB single-writer invariant vs dashboard write needs
Adopted: option (c) second Ssb writer instance opened in the dashboard process pointing at the same db_path.
Reason: dashboard runs in a separate OS process (cli/src/main.rs spawns it detached). No in-process IPC channel exists between dashboard and orchestrator. SQLite WAL permits multi-process writers; per-process the single-writer invariant holds (each Ssb owns one mpsc UnboundedSender). Dashboard write cadence is user-driven (intervention responses, task submissions) — contention with orchestrator is statistically rare; SQLITE_BUSY retried by busy_timeout=5000ms.
Tradeoff: documented as DASHBOARD_SECOND_SSB_WRITER_MVP_TRADEOFF in gotchas.md.

Decision 2 — submit_task delivery mechanism
Adopted: NEW SsbActionType::TaskSubmitted, schema bump v8 -> v9.
Reason: direct orchestrator channel injection across OS processes is impossible in 4K scope. SSB is the canonical cross-process substrate. Orchestrator pickup of TaskSubmitted is explicitly DEFERRED to a later phase (DASHBOARD_TASK_SUBMITTED_PICKUP_DEFERRED). 4K.3 only writes the action; consumption is post-MVP.
9-step protocol applied: (1) variant in SsbActionType, (2) match arms in as_sql_str + ssb_action_type_from_str + ssb_action_type_from_str_public, (3) variant in SsbPayload, (4) action_type() match arm, (5) CHECK constraint includes 'task_submitted', (6) SCHEMA_VERSION 8->9, (7) idempotent run_migration (existing pattern: drop+recreate when version != SCHEMA_VERSION), (8) all_test_payloads + COUNT bumped 31->32, (9) proptest coverage updated.
TaskSubmittedPayload: { task: String, project_id: String, submitted_at_ms: i64 }.
Timestamp source: ClockProvider via Ssb (avoid raw std::time per entropy quarantine). Adds Ssb::now_ms() helper that calls clock.system_now() and converts to ms-since-epoch.

Decision 3 — step_progress(phase_id) addition to SsbViewer
Signature: pub fn step_progress(&self, phase_id: &str) -> Result<Vec<StepStatus>, SsbViewerError>.
StepStatus: { step_id, status, started_at: Option<i64>, completed_at: Option<i64>, agent_id: Option<String> }.
Implementation: scan ssb rows where action_type IN ('phase_transition','phase_complete','reasoning_trace'); rmp_serde decode payload to extract step_id (msgpack, NOT JSON); group in HashMap by step_id; status = phase_complete > phase_transition > reasoning_trace; started_at = MIN(ts), completed_at = MAX(ts) when complete; agent_id = MAX(agent_id).
Validation: phase_id matches ^[A-Za-z0-9._-]{1,64}$; reject empty / dot-traversal.
Companion: pub fn get_action_detail(&self, seq: i64) -> Result<SsbActionDetail, SsbViewerError>. SsbActionDetail = { seq, ts, action_type, agent_id, payload_json: serde_json::Value, files: Vec<String> }. payload_json built via rmp_serde decode then serde_json::to_value of the SsbPayload.

Supporting decisions

Projects store
Path: $USERPROFILE/.elai_cc/projects.json (Windows) or $HOME/.elai_cc/projects.json (Unix). Resolved via std::env::var to avoid adding the `dirs` dependency.
Atomic write: write to projects.json.tmp + std::fs::rename.
Schema: Vec<ProjectEntry> with { project_id, name, root_path, created_at_ms, last_opened_ms }.
Validation: project_id regex ^[A-Za-z0-9._-]{1,64}$; reject "..", reject absolute path in name.

Tauri shell strategy (preview link open)
Use std::process::Command directly. No tauri-plugin-shell dependency.
Windows: cmd /c start <url>. macOS: open <url>. Linux: xdg-open <url>.
URL whitelist: must start with http://localhost or http://127.0.0.1 — reject any other scheme/host (security boundary).

Frontend dependencies
LOCKED: svelte 4.2.19 + vite 5.4.11 + @tauri-apps/api 2.1.1. Do NOT bump.
No new npm packages.

Cargo dependencies
ADD to elai-dashboard/Cargo.toml: scaffold = { path = "../scaffold" } (verify exact crate name = "scaffold").
elai-orchestrator already present; Ssb is exported via crate root re-export.
Do NOT add: dirs (use std::env::var), chrono (use Ssb::now_ms wrapping ClockProvider), tauri-plugin-shell.

Test scope discipline
Scoped tests during implementation: cargo test -p elai_cc_contracts --test dashboard_4k3_4k4_4k5_contract; cargo test -p elai-dashboard.
Full workspace tests reserved for the auditor.
cargo check --workspace required to PASS before audit.

Single audit constraint
Auditor receives ONE invocation. Verdict scored independently per sub-phase (4K.3, 4K.4, 4K.5). Every BLOCK + every WARN fixed in ONE corrective pass. NO re-audit. Architectural BLOCKs that need redesign escalate to user; do NOT silently downgrade.

Memory-bank
Updates after build/test in MEMORIZING via memory-writer agent.
New gotchas: DASHBOARD_SECOND_SSB_WRITER_MVP_TRADEOFF, DASHBOARD_TASK_SUBMITTED_PICKUP_DEFERRED.

FSM target trajectory
PLANNING (current) -> TESTING_FIRST -> EXECUTING -> AUDITING -> MEMORIZING -> COMPLETE -> IDLE.

Commit policy
Prefix: "4K.3-4K.5: ...". Body includes pass/fail counts, deterministic check status. NO Co-Authored-By trailer.
