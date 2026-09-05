E-25 Tranche C decisions
Source session: s-22600 Tranche C (continuation chain: s-night-2026-05-30 -> s-21636 -> s-22300 -> s-22600)
Date: 2026-05-31

Scope: ship PRODUCER bodies on top of the SHIPPED 9-arm substrate
(ClaudeCodeArchitectureEventKind + ClaudeCodeArchPrimitive incl. all 3
guards + EditCheckpointSnapshotted). ZERO schema bump expected, ZERO
new SsbActionType. Confirmed by post-tranche cargo check --workspace
--tests (GREEN).

DECISION-T-C-1 (C.1 anti-pattern guards):
SHIP three guards as sibling modules under
crates/orchestrator/src/architecture/:
  - bash_complexity_guard.rs (~280 LOC incl tests)
  - approval_fatigue_guard.rs (~310 LOC incl tests)
  - sandbox_template_guard.rs (~210 LOC incl tests)
Each guard mirrors the FreeTierGuard shape at
cost/free_tier_guard.rs:103: per-process struct + check + build_payload.
Thresholds are NAMED CONSTANTS (no magic numbers):
  - BASH_TOKEN_COUNT_THRESHOLD = 80
  - BASH_PIPE_COUNT_THRESHOLD = 4
  - BASH_METACHAR_COUNT_THRESHOLD = 6
  - APPROVAL_WINDOW_SIZE = 50
  - AUTO_APPROVE_THRESHOLD_PCT = 80
  - SKILL_TEMPLATE_JAIL_PATH = ".elai_cc/.elai/state/skill_template_jail"
Three separate env gates (NOT one shared) so each guard can flip
independently in field testing:
  - ELAI_E25_BASH_COMPLEXITY_GUARD_ENABLE
  - ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE
  - ELAI_E25_SANDBOX_TEMPLATE_GUARD_ENABLE
SSB cascade: REUSE the SHIPPED Pattern-2 single-variant
SsbActionType::ClaudeCodeArchitectureEvent + the 3 already-shipped Kind
arms (BashComplexityBlocked / ApprovalFatigueDetected /
SandboxTemplateWarned). ZERO new SsbActionType. ZERO schema bump.
Contract test: crates/contracts/tests/e25_guards_contract.rs (3 tests,
one per guard).

DECISION-T-C-2 (C.2 prompt-handler):
MINT NEW 3-arm enum at
architecture::hook_prompt::PromptDecision { Allow, Deny(reason),
Ask(reason) }. Per Tranche-C explorer GAP-1: extending
HookVerdictKind {Allow, Warn, Block} would cascade through 9+ existing
match-arm sites at hook_dispatcher.rs:179/185/221/746/758/781/797/801/
828/...; explicitly rejected. The NEW enum lives in
architecture/hook_prompt.rs alongside the SHIPPED
ClaudeCodeArchitectureEventKind::PromptDecisionMade arm reuse path.
Default-off wire-dark gate ELAI_E25_PROMPT_HANDLER_ENABLE.
5s timeout (PROMPT_HANDLER_TIMEOUT = Duration::from_secs(5)) via
tokio::time::timeout per E-25 plan L60.
Parse-failure semantic LOCKED: NO graceful fallback to Allow; returns
PromptHandlerError::InvalidDecisionFormat / UnknownVariant
(thiserror; native async fn; no async-trait; no Box<dyn Error>).
Module surface intentionally NOT wired into hook_dispatcher.rs this
tranche -- the 9+ HookVerdictKind use sites would cascade; the
follow-up tranche (likely E-24 Task 5 pre-MCP-call guard) consumes
hook_prompt::prompt_decision directly.

DECISION-T-C-3 (C.3 cache_edits):
DOC-ONLY DEFERRAL. No producer module. See companion file
workflow/reasoning/E-25-trancheC-cache-edits-deferral.md.
Matches B-4 D14 disposition unchanged.

DECISION-T-C-4 (C.4 edit_checkpoints):
SHIP at crates/orchestrator/src/architecture/edit_checkpoints.rs
(~500 LOC incl tests). On-disk content-addressed storage at
.elai_cc/.session-checkpoints/{session_id}/{seq:06}/{blake3_prefix:8}/{path}.
blake3 ONLY (HARD RULE Anti-Stale-Training; DETERMINISTIC_CHECK #9 --
zero blake2/sha2 hits). Workspace dep already cleared at Cargo.toml:56
+ orchestrator/Cargo.toml:82 + Cargo.lock:265 (no slopsquat
re-verify needed per explorer).
Public surface: snapshot_pre_edit / list_session_checkpoints /
rewind_to / evict_oldest / evict_session + build_snapshot_payload.
Eviction policy: caller responsibility -- helpers expose evict_oldest
(default last-N=10 per session) + evict_session (full evict at session
END). Git-commit-hook eviction (HookPoint::PostCommit) DEFERRED to
follow-up patch-N (verification of the hook point + integration with
session_dag tear-down lives in a future tranche).
Trigger point: HookPoint::PreToolUse (hook_dispatcher.rs:1603 --
verified to exist) filtered to tool_name in {"Edit", "Write",
"NotebookEdit"} by the caller. The module itself is a pure storage +
query surface; the dispatcher wiring is a follow-up patch-N to keep
this tranche minimum-diff (hook_dispatcher.rs cascade risk).
Rewind emission per GAP-3: reuse the SHIPPED
EditCheckpointSnapshotted Kind arm with details_json.rewind_event =
true marker. 9-arm Kind enum LOCKED per session-4 DECISION-5 -- NO
10th EditCheckpointRewound arm.
CLI surface: `elai checkpoint list [--session-id ID] [--json]` +
`elai checkpoint rewind --steps N [--session-id ID] [--dry-run]`
at crates/cli/src/checkpoint_cmd.rs (+ args.rs CheckpointSub +
parse_checkpoint_subcommand_args + main.rs CliAction::CheckpointSub
dispatch).
Env gate: ELAI_E25_EDIT_CHECKPOINTS_ENABLE for the SSB emit cadence;
the CLI inspection paths work regardless of the gate so the operator
can audit existing snapshots without flipping.
Contract test: crates/contracts/tests/e25_edit_checkpoints_contract.rs
(4 tests: snapshot creates file at expected path / rewind restores
file content / rewind emits SSB row with rewind_event=true marker /
unknown seq returns error not panic).

DECISION-T-C-5 (C.5 dashboard panels):
DOC-ONLY DEFERRAL. See companion file
workflow/reasoning/E-25-trancheC-dashboard-panels-deferral.md.
CompactionLadderPanel BLOCKED-ON-PRODUCER (zero callers of
run_compaction_ladder outside contract test). McpSchemaPanel DESCOPED
(blocks on E-22 Task 3 / C-35). SessionForkPanel REDUNDANT (existing
SessionsDrawer + BranchTree covers it).

Verification cadence:
- cargo check --workspace --tests after EACH item (per user constraint
  "no cargo test unless permitted"). Auditor still owns the test
  runner; main LLM did compile-only verification.
- Pre-existing warnings only. ZERO new warnings introduced by Tranche
  C code.

Constraints honored (per user prompt):
- Anti-stale-training: tracing not log; thiserror not Box<dyn Error>;
  blake3 not blake2/sha2; .try_into() narrowing where needed; named
  constants; no .unwrap() in production code paths (test-only).
- No new dependencies (blake3 1.8 already workspace dep -- no
  anti-slopsquatting re-verify required).
- Stay in .elai_cc/ (HARD project boundary).
- No auto-commit; no cargo test; max 3 parallel agents (executor only).

Followups (logged as future patch-N candidates):
1. Wire BashComplexityGuard / ApprovalFatigueGuard / SandboxTemplateGuard
   into permissions.rs::evaluate (pre-tool-dispatch) +
   skill_templating.rs::expand_skill_template (microsandbox absent
   branch). Currently the guard modules ship the substrate but no
   in-process emitter wires them into the production hook dispatch.
2. Wire hook_prompt::prompt_decision into the E-24 Task 5
   pre-MCP-call guard once that lands. Default runtime per CLAUDE.md
   remote-sidecar block = Qwen2.5-Coder-3B abliterated at
   127.0.0.1:8100 (PerRoleRuntime::assistant_decision role).
3. Wire edit_checkpoints::snapshot_pre_edit into hook_dispatcher.rs
   PreToolUse path filtered to tool_name in {Edit, Write,
   NotebookEdit}. Plus session_dag tear-down -> evict_session.
4. After (3) lands, ship CompactionLadderPanel.svelte + Tauri
   list_compaction_events command (currently DEFERRED).
5. Verify HookPoint::PostCommit existence and wire git-commit-hook
   eviction (out-of-scope for this tranche per explorer GAP-4).
