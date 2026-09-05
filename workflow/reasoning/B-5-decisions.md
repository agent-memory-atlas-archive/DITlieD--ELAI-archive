B-5 Reasoning Engine Producer Wiring — DECISIONS
Date: 2026-06-05 (backfill)
Source: workflow/sessions/s-plan-audit-2026-06-04/plan-status/B/B-5_reasoning_producer_wiring.md

SHIPPED
- SSB action types (4 of 5 planned), all in ssb.rs: GrcStepEmitted (9807), GroundingSweepResult
  (9808), CsqEvaluated (9809), AuditorRubberStampRejected (9810), with payload structs at 16175/
  16195/16214/16230.
- Tauri commands (5, registered in lib.rs): get_grc_budgets (2709), get_grounding_sweep (2760),
  get_csq_log (2878), get_feasibility_log (2925), get_agent_grc_config (2997).
- emit_reasoning refactored (task_runner.rs:12187-12249) to pull real agent_id + budget_remaining
  from the PipelineGrc handle; named constants for grounding_level/confidence (no magic literals).
- ReasoningDrawer.svelte: per-agent GRC budget gauges, grounding-sweep panel, CSQ-pending panel;
  drawers.svelte.ts:328-329 mark groundingSweep/csqPending live (unwired=false).
- Production reach: emit_reasoning (task_runner.rs:2405-2414) passes resolved.pipeline_grc.as_ref()
  so GrcStepEmitted fires on live runs; emit_csq_evaluated at task_runner.rs:7819/7837/7850.

KEY DECISIONS
- feasibility_check renamed: the planned `feasibility_check` SSB type does not exist; it was
  superseded by FeasibilityReport + FeasibilityEscalated, which are B-21 (Phase 4F) scope, not B-5.
  B-5 does not own a discrete feasibility_check variant.

DEFERRED / NOT DONE
- (none — all originally-deferred items are now WIRED, see UPDATE 2026-07-02 below)

UPDATE 2026-07-02 (B-5 executor S5 — GAP-1 + GAP-2 + GAP-3 close)
- CsqBlockCard.svelte (Task 6): WIRED. The component exists (199 lines) at
  crates/dashboard-v2/src/lib/components/CsqBlockCard.svelte, filters get_csq_log for
  Critical + guard_present=false rows, Decline -> submit_task replan string. Mounted in
  ReasoningDrawer.svelte at the top of the CSQ section. The documented honest in-scope
  wiring uses submit_task (replan request string), not a typed ReplanRequest Tauri command
  — no FSM-mutation command exists; this is acceptable per the original decisions rationale.
- get_agent_grc_config -> Svelte consumer (Task 3): WIRED. The ReasoningDrawer imports
  get_agent_grc_config + get_feasibility_log and renders the per-role GRC config table
  (8+ roles) plus the feasibility log panel. No longer dead-code.
- GAP-1 (anti-rubber-stamp counter): CLOSED. New Tauri command
  `get_auditor_rubber_stamp_rejections` (commands.rs, registered in lib.rs) reads
  `auditor_rubber_stamp_rejected` SSB rows via SsbViewer::recent_rows_of_type and reshapes
  them into AuditorRubberStampRejectionRow JSON. New stub.ts wrapper
  `getAuditorRubberStampRejections(limit?)`, new types.ts row
  `AuditorRubberStampRejectionRow`, new reasoningData store field
  `auditorRubberStampsRejected`, new ReasoningDrawer counter UI ("Auditor: N rubber stamps
  rejected this session") with a red Badge when count > 0 and a per-row breakdown list.
  Empty array = steady state (counter shows 0); no NaN / undefined path.
- GAP-2 (emit_reasoning real values): CLOSED. On the grc=Some branch, emit_reasoning now
  pulls grounding_level from `engine.get_agent_config().grounding_level` (real per-agent
  enum value serialized as "Uncertain" / "High" / "Verified"), derives confidence via
  confidence_from_grounding_level (Uncertain=0.5, High=0.8, Verified=1.0), and measures
  elapsed_ms from a std::time::Instant captured at fn entry. The hardcoded literals
  "medium" / 0.9 / 0 are NO LONGER used on the grc=Some path. The grc=None fallback path
  retains the named constants (documented intentional synthetic branch). Two unit tests
  added in task_runner.rs tests mod: b5_emit_reasoning_grc_some_pulls_real_grounding_values
  (asserts Verified/1.0/real agent_id) + b5_emit_reasoning_grc_none_uses_fallback_constants
  (regression guard for the synthetic branch). BasicGrcEngine does not expose the last
  ThinkResult's grounding_level (topics field is private, no accessor on the GrcEngine
  trait); the per-agent required minimum grounding level is the real, non-synthetic value
  available without an out-of-scope edit to engine.rs. When agent_config is None (default
  engine, first emit), the grc=Some path falls back to the named constants — documented.

NOTES
- No ELAI_*_ENABLE flag; all four SSB types are default-ON (no allowlist row). wiring_contract_check
  hits=0 was a false negative — all planned symbol names verified present.
- B-5 GAP-1 read command holds no lock across .await (SsbViewer::recent_rows_of_type is
  sync; the Tauri command is sync). No .unwrap()/.expect() in the new command or emit path.
- B-5 GAP-2: the Mutex guard is dropped before the async SSB write (codeRules #8). The
  three cloned values (role, budget, grounding_level_str) are owned and independent of
  the guard. elapsed_ms is computed from emit_start.elapsed() AFTER the guard is dropped,
  so it captures the full lock+extract duration.
