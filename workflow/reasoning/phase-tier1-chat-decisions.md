Phase: Tier 1 CHAT WINDOW (dashboard-wiring-plan.txt section 5)
Session: s-147708
Date: 2026-04-22

WHAT

Wire the KIMI-inspired CHAT window into the dashboard shell built in Tier 0.
CHAT replaces the inline /mode + /model command strip in Workspace.svelte with
a message-list + card-based surface. SSB grows two new action_type rows
(chat_message, chat_response) and the dashboard gains four read/write Tauri
commands (classify_intent, submit_chat_message, get_chat_history,
get_recent_tasks).

KEY DECISIONS

1. SlashPalette ownership = Shell.svelte overlay slot.
   The slash palette is a cross-window command surface (Ctrl+. opens it over
   the whole shell). Shell.svelte already owns `layout.overlay === "slash-palette"`
   (lines 223-233, currently a placeholder "arrives in Tier 1"). The component
   renders INSIDE that conditional block, NOT inside ChatWindow. ChatWindow
   can trigger the overlay via openOverlay() but does not render it.

2. intentRouter = pure client-side deterministic first, backend fallback on
   AMBIGUOUS. intentRouter.ts exports routeClientSide(text, context) which
   returns IntentClassification | null. null means "client-side could not
   classify", at which point the caller (ChatWindow) invokes classify_intent
   on the backend. The backend command is a thin shell for now — it echoes
   AMBIGUOUS with empty alternatives. Full LLM-backed classification is
   deferred until an executor/classifier role is chosen (Tier 1 ships the
   wiring; deterministic prefixes cover 100% of today's slash commands).

3. /mode + /model regression preserved via intent-router passthrough.
   routeClientSide recognises `/mode <arg>` and `/model <arg>` as MODE / MODEL
   intents. ChatWindow dispatches those directly to set_gate_mode /
   set_model_routing (existing tauri commands, signatures unchanged).
   submit_chat_message still writes a chat_message row for history, but the
   side-effect of mode-change flows through the existing commands — no new
   code path on the orchestrator side.

4. InterventionCardV2 = composition, not duplication.
   InterventionCardV2.svelte imports InterventionCard.svelte and renders it
   inside a message-card chrome (header, timestamp, footer). The
   `confirm_irreversible:` branch + submit_intervention_response +
   respond_to_confirmation flow stays in the inner InterventionCard —
   V2 does not re-implement it. This locks in the gotchas
   INTERVENTION_OPTION_UNTAGGED_VARIANT_ORDER safety: the untagged enum's
   Retry-before-Keyword order (ssb.rs:802) remains untouched because we do
   not touch the type or the parse_chosen function.

5. SSB schema bump = SCHEMA_VERSION 19 -> 20.
   Real current value confirmed via grep of ssb.rs:26 (context-compiler's
   "SCHEMA_VERSION is 8" claim was stale by ~11 versions). Migration branch
   uses the existing idempotent DROP+recreate pattern at ssb.rs:2082
   (`conn.execute_batch(CREATE_SSB_TABLE)`), which re-issues CREATE TABLE IF
   NOT EXISTS — safe to run twice.

6. SsbActionType count update = real count, not hardcoded +2.
   The `/// All 32 action types` doc at ssb.rs:93 is stale (real variant
   count = 71 before this change, verified by counting as_sql_str arms
   lines 282-352). After adding ChatMessage + ChatResponse the count is 73.
   Doc comment becomes `/// All 73 action types`. The `all_test_payloads`
   vector at ssb.rs:3667 currently contains only 32 of the 71 variants
   (the post-TaskSubmitted additions were never backfilled into the test
   vector). We add 2 entries there (chat_message + chat_response) and bump
   the assert_eq! at line 3905 from 32 -> 34 — we do NOT backfill the
   missing 39, that is out of scope for Tier 1.

7. CHAT_MESSAGE + CHAT_RESPONSE are dashboard-originated, no orchestrator
   consumer yet.
   Same deferral pattern as DASHBOARD_TASK_SUBMITTED_PICKUP_DEFERRED gotcha.
   Rows are written for history and dashboard poll; no task_runner side-
   effect for now. The variant doc comments note this.

8. project_id scoping follows the `TaskSubmitted` precedent.
   project_id is carried INSIDE ChatMessagePayload (like TaskSubmittedPayload),
   not relied upon from the SSB row column. SsbViewer::chat_history reads the
   payload msgpack to filter client-side. Reasoning: the dashboard writer has
   a single fixed row-level project_id (bound at Ssb::new time), but the user
   can be acting on any project. Follows the existing precedent rather than
   introducing a new convention.

9. Tauri command surface: 43 -> 47 entries in invoke_handler! (lib.rs:1305).
   Context-compiler said 35 -> 39; real counts are 43 -> 47 (grep
   `#[tauri::command]` in lib.rs:1305-1360 returns 43 names between the
   macro delimiters). tier0-shell.mjs INVOKE_BASELINE constant (= 35) is
   frozen; tier1-chat.mjs gets its own INVOKE_EXTENDED constant (>= 39 new
   frontend invocations, actual count verified at test time).

10. std::sync::Mutex<Connection> (Windows correctness).
    SsbViewer already uses std::sync::Mutex<Connection> per
    RUSQLITE_CONNECTION_NOT_SEND_WINDOWS. New chat_history + recent_tasks
    methods follow the same pattern (no .await held across the lock).

11. submit_chat_message = async fn (follows submit_task precedent).
    Ssb::write_payload is async (ssb.rs:2003), so the tauri command is
    async. The existing submit_task at commands.rs:291 is the reference
    pattern — mirror it exactly.

12. Named constants only.
    CHAT_TEXT_MAX_LEN = 10_000, HISTORY_RING_SIZE = 20, LIMIT_MIN = 1,
    LIMIT_MAX = 500 — all named consts in commands.rs / intentRouter.ts.
    No magic numbers.

EXPECTED TESTS

Contract tests RED first, GREEN after implementation:
  crates/contracts/tests/dashboard_tier1_chat_contract.rs
    - assert SsbActionType::ChatMessage + ChatResponse exist
    - assert as_sql_str roundtrip for both ("chat_message", "chat_response")
    - assert ssb_action_type_from_str_public accepts both strings
    - assert SsbPayload::action_type() returns correct variant for both
    - assert SCHEMA_VERSION bumped by 1 (19 -> 20)
    - assert ChatMessagePayload / ChatResponsePayload serde round-trip
    - assert writing a ChatMessage row through the SSB writer succeeds
      against a fresh in-memory DB (exercises the CHECK constraint)
    - assert SsbViewer::chat_history and recent_tasks return expected rows
      after writing fixtures

  crates/dashboard/tests/contracts/tier1-chat.mjs
    - REQUIRED_CHAT_FILES = 11 files under src-frontend/src/chat/ exist
    - 4 new command names are present in invoke_handler! in lib.rs
    - intentRouter.ts exports routeClientSide and handles >= 10 prefix
      mappings (/task /fix /explore /config /status /continue /research
      /ask /mode /model)
    - WindowRegistry.ts LIVE map contains [WindowType.CHAT]: ChatWindow
    - Shell.svelte overlay block at layout.overlay === "slash-palette"
      renders the SlashPalette import (not the placeholder text)
    - tier0-shell.mjs baseline (INVOKE_BASELINE = 35) re-runs GREEN
      after Tier 1 changes

Scoped build (auditor runs the full suite):
  cd .elai_cc && cargo check -p elai_cc_orchestrator -p elai_cc_contracts
  cd .elai_cc && cargo check -p elai_cc_dashboard
  cd .elai_cc/crates/dashboard/src-frontend && npm run build
  node .elai_cc/crates/dashboard/tests/contracts/tier0-shell.mjs
  node .elai_cc/crates/dashboard/tests/contracts/tier1-chat.mjs

UNKNOWNS

- classify_intent stub behaviour: today it returns AMBIGUOUS with empty
  alternatives for anything deterministic routing could not handle. Longer-
  term this becomes a sidecar-backed call; Tier 1 does not ship that.
- MessageList does not virtualise yet. Server-side limit (HISTORY_RING_SIZE
  = 20 via get_chat_history) is the cap. Virtualisation is a Tier 2
  concern if chat volume grows.
- CSS style pass (teal borders for StatusCard, fail border for ErrorCard,
  etc.) matches the dashboardRules #14 "no dynamic class names" rule.
  Static class names only — if design calls for more variation, that is
  Tier 3 drawer styling territory.

REGRESSION PATHS (explicitly verified)

- /mode <arg> -> intentRouter MODE -> ChatWindow calls set_gate_mode ->
  Shell.svelte's 2s gate poll reflects the change.
- /model <arg> -> intentRouter MODEL -> ChatWindow calls set_model_routing.
- InterventionCardV2 -> mounts InterventionCard -> confirm_irreversible:
  branch intact -> submit_intervention_response / respond_to_confirmation
  paths unchanged.
- Workspace.svelte still renders PlanTree + ActivityFeed + InterventionCard
  + ProposalsPanel as the PLAN window (only the /mode /model command strip
  and the commandLog are removed).

CONSTRAINTS

- No commits.
- No new Rust crates or npm packages.
- No memory-bank writes by main model (memory-writer agent only).
- No .unwrap() / .expect() / panic!() in library Rust.
- No tokio::sync::Mutex for rusqlite::Connection.
- No reading of memory-bank / plan files directly (already satisfied; plan
  was user-authorised exception, context-compiler ran for everything else).

FSM JOURNEY

IDLE -> PLANNING (done) -> TESTING_FIRST -> EXECUTING -> AUDITING ->
MEMORIZING -> COMPLETE -> IDLE. Auditor and memory-writer agents spawned,
not skipped. mcp_session_release at end.

OUT-OF-SCOPE FINDING (logged here for the next patch, NOT fixed in Tier 1)

The ssb.rs CHECK constraint in CREATE_SSB_TABLE (lines 40-67 pre-Tier-1)
omits three mcp_* variants that the enum + as_sql_str + from_str all
reference: mcp_server_connected, mcp_tool_called, mcp_server_disconnected.
Source: batch-ful-d (commit bc43953) added enum variants but forgot the
CHECK list. Effect: fresh SSB databases silently reject those rows with
SQLITE_CONSTRAINT; existing dev DBs skip the problem because CREATE TABLE
IF NOT EXISTS never rewrites an existing CHECK constraint on a schema
version bump. Tier 1 intentionally does NOT fix this to keep the diff
narrow — captured here so a future patch-N.X can restore parity.
