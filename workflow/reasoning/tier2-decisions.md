Tier 2 Dashboard Wiring — Decision Doc
Session: s-342352
Date: 2026-04-22

What
  Wire all 7 visual windows from dashboard-wiring-plan.txt §6 (Tier 2). Two parallel tracks with one audit cycle each.

Track A (frontend-only):
  ActivityWindow, ReviewWindow, SettingsWindow placeholder, FSM-keyed routing in layoutState, sticky-override, WindowRegistry mapping for these 3.

Track B (backend-extending):
  PlanWindow + 3 SsbViewer methods, MonitorWindow + get_monitor_snapshot aggregator, ExploreWindow re-routed to /search, PreviewWindow + Tauri-managed DevServerHandle registry serving both get_preview_state and stop_dev_server, contract-test invoke_handler count update 48->56.

Key decisions
  D1. Preview state: runtime join with web_validator::DevServerStatus + Tauri Arc<RwLock<HashMap>> registry. Rejected SSB v9->v10 schema bump (cascade churn through ssb.rs round-trip tests + downstream readers; plan §12 enumerates zero new SSB action types this tier).
  D2. Explore: re-route to existing GET /search (server.py:2343) with 5-min TTL HashMap cache keyed by blake3 explore_id. Defers /v1/explore work; honors plan §13 grounding rule (verify before calling).
  D3. WindowRegistry mapping for PLAN/MONITOR/EXPLORE/PREVIEW deferred to Tier 2.5. Reason: Track A owns WindowRegistry.ts for parallel-file-safety; merging both tracks' edits mid-parallel violates the no-shared-files rule. Window files + contract tests are still delivered this session.
  D4. Contract test homes: crates/dashboard/tests/contracts/*.mjs (Node, mirror tier1-chat.mjs) + crates/contracts/tests/dashboard_tier2_windows_contract.rs (Rust, mirror dashboard_tier1_chat_contract.rs). The .elai_cc/src-tauri/tests/contracts/ path I initially named does not exist.
  D5. BabyStepSpec source: Track B greps the orchestrator crate at runtime. If unreachable, fields are Optional with a doc-comment limitation rather than fabricated.

Expected test outcome (per audit)
  Track A: cargo check workspace pass, contract tests for ActivityWindow/ReviewWindow/SettingsWindow mount + routing-table FSM mapping pass, no regression in existing PlanTree/ActivityFeed/DiffViewer mounts inside PanelHost.
  Track B: cargo check workspace pass, 8 new tauri commands return well-typed payloads against fixture SSB, /search re-route returns valid ExploreResult, preview registry round-trip (start handle -> get_preview_state shows it -> stop_dev_server clears it).

Unknowns
  U1. BabyStepSpec accessibility from orchestrator (Track B will resolve at runtime).
  U2. Whether the existing dashboard build picks up new src-frontend/src/windows/ tree without vite.config.ts edits — likely yes since Tier 1 added src-frontend/src/chat/ without manifest changes.

Constraints
  C1. 1 audit cycle per track. block_count=0 on AUDITING->MEMORIZING transition.
  C2. No commits this session.
  C3. No new dependencies.
  C4. Project root = F:\.ELAI_workflow\.elai_cc\ — no writes outside.
