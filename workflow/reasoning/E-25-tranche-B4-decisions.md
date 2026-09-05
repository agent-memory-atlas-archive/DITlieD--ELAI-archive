E-25 Tranche B-4 decisions

Session: s-22300
Sub-phase: B-4 (compaction_ladder + skill_templating + opusplan FsmState overlay + session_fork SSB emit)
Author: orchestrate executor
SCHEMA_VERSION unchanged at 172 (verified — SCHEMA_VERSION_PUBLIC reuses 172).

Shipped (D1..D6)

D1 compaction_ladder.rs module — SHIPPED
  Location: crates/orchestrator/src/compaction_ladder.rs
  Public surface: CompactionLadder { tiers, target_budget_tokens }, LadderRunResult, LadderEventEmitter trait, NullEmitter, run_compaction_ladder(), compaction_ladder_enabled().
  Sync function; per-tier estimate; cheapest-first traversal; stops at first tier dropping budget at-or-below target. Emits one ClaudeCodeArchitectureEvent (CompactionLadderTierApplied) per applied tier via the caller-supplied LadderEventEmitter.
  Env-gate: ELAI_E25_COMPACTION_LADDER_ENABLE (wire-dark default OFF).
  Async wraps for adaptive_reduce / invoke_telescope / compact_session are NOT inlined — those are async + need a Session/sidecar context that does not flow through this sync runner. The runner ships the traversal/budget-stop/emit cadence; downstream caller (task_runner / session_dag::create_handoff) composes the async wraps. Decision recorded under "concrete impl decisions D2" in exploration notes.

D2 skill_templating runtime expansion — SHIPPED
  Location: crates/orchestrator/src/skill_templating.rs
  Public surface: SkillTemplate, expand_skill_template(), skill_templating_enabled(), SkillTemplateError (4 variants), SkillTemplateEmitter trait, NullEmitter.
  Runtime expansion of !`cmd` blocks inside SKILL.md bodies. Allowlisted commands only (git/ls/pwd/date) on first ship; growth requires user confirm. Shell metacharacters (; & | $ < > ` ) rejected outright. 2s timeout per command. Commands dispatched via std::process::Command (no shell layer).
  Env-gate: ELAI_E25_SKILL_TEMPLATING_ENABLE (wire-dark default OFF).
  Live-reload via notify crate DEFERRED (see D7).

D3 opusplan FsmState overlay — SHIPPED
  Location: crates/orchestrator/src/fsm.rs (additive FsmInner field + Fsm setter/getter + env-gate helper).
  Surface: Fsm::set_opus_plan_active(bool) async, Fsm::opus_plan_active() async, opus_plan_overlay_enabled() helper, ELAI_E25_OPUSPLAN_OVERLAY_ENABLE_VAR const.
  Decision: NOT minted as FsmSubstate::OpusPlan (would collide with shipped Framing/SleepCompute/Tdd/DarwinianGenerationEvaluating/Adversary). Keyed on FsmState (top-level Planning/Executing already exist). Per exploration GAP-4 + context-package OPEN-QUESTION Q2.
  Persistence: opus_plan_active is process-local (NOT persisted to PersistedFsmState). Restarts default OFF. Rationale: planner/executor tier engagement is a runtime overlay; durable persistence would re-enable lookups against an ephemeral runtime label.
  Emission is the caller's responsibility — Ssb::write_payload is async + the setter is sync; the caller checks opus_plan_overlay_enabled() and writes via the existing write_payload pattern when the flag flips.

D4 session_fork SSB emit at 3 sites — SHIPPED
  Sites wired:
    - crates/cli/src/handoff.rs:495 (DAG path Ok branch): emit_session_forked(... "session_forked_dag" ...)
    - crates/cli/src/handoff.rs:fallback (ConversationRuntime::fork_session path): emit_session_forked(... "session_forked_fallback" ...)
    - crates/dashboard-v2/src-tauri/src/commands.rs:fork_session_branch: emit_session_forked_dashboard(... "session_forked_branch" ...) async
  Both helpers carry details_json = { fork_event, parent_session, child_session, branch_point }.
  Env-gate: ELAI_E25_SESSION_FORK_EMIT_ENABLE (wire-dark default OFF). Failures swallowed (best-effort telemetry).
  Pattern: cli sync helper spins a short-lived current_thread runtime to await write_payload (mirrors /session merge L585-588). Dashboard helper is already async.

D5 forked_from metadata on SessionFork — SHIPPED
  Location: crates/orchestrator/src/session.rs (SessionFork struct + fork() constructor + to_json / from_json).
  Field: forked_from: Option<String> (defaults None on direct forks; reserved for chain-fork root lineage in future).
  Back-compat: from_json tolerates older handoff JSON without the field (returns None).
  Note: SessionHandoff struct does NOT exist in this codebase as the spec implied; SessionFork is the canonical Rust fork-provenance struct. SessionHandoff is the .md output file. The spec intent (record forked_from in handoff metadata) is satisfied by SessionFork.forked_from being serialized to JSON via to_json -> handoff.

D6 contract tests — SHIPPED
  Location: crates/contracts/tests/e25_tranche_b4_contract.rs
  Tests:
    - no_parallel_compaction_ladder_module (banned _v2 symbols + single-file assert)
    - no_parallel_skill_template_loader (banned _v2 symbols)
    - no_parallel_opusplan_overlay (banned _v2 symbols)
    - compaction_ladder_tier_round_trip_msgpack (round-trip every LOCKED tier)
    - session_fork_event_details_json_shape (details_json scoped to 3 valid fork_event labels)
    - opusplan_overlay_default_off (env-gate unset -> false)
    - compaction_ladder_default_off (gate unset -> zero-effect LadderRunResult)
    - skill_templating_default_off (gate unset -> body verbatim)
    - workspace_walker_finds_at_least_one_rs_file (walker sanity)

Deferred (D7..D14)

D7 notify crate add — DEFERRED
  Required by skill_templating live-reload. Cargo.lock grep returned zero hits for ^name = "notify". Adding the dep requires explicit user confirmation per CLAUDE.md HARD RULE (no new deps without confirm) + anti-slopsquatting verification (cargo search notify; owner=notify-rs/notify; latest >= 6.x or 7.x; <24mo; MIT/Apache).
  Until landed: skill_templating ships as sync expander invoked on each manual SKILL.md load. No filesystem watcher.

D8 dashboard CompactionLadderPanel.svelte + Tauri list_compaction_events command — DEFERRED
  Cross-tranche dep + path-correction (drawers/ not components/ per DRIFT-9 from session-3 decisions). Ship as follow-up after first CompactionLadderTierApplied rows land via task_runner wiring.

D9 dashboard McpSchemaPanel.svelte — DEFERRED (DESCOPED)
  Blocks on E-22 Task 3 / C-35 lazy_schemas mechanism (unshipped). Per E-25-decisions DECISION-3.

D10 dashboard SessionForkPanel.svelte — DEFERRED (REDUNDANT)
  Existing SessionsDrawer.svelte + BranchTree UX already covers fork visualization. Extend SessionsDrawer if needed.

D11 edit_checkpoints.rs (Primitive 6b) — DEFERRED
  Out of tranche-B4 scope per task description (compaction + templating + opusplan + fork only — 6a NOT 6b). Plan task-6b covers checkpoints; ship as separate tranche.

D12 BashComplexityGuard / ApprovalFatigueGuard / SandboxTemplateGuard — DEFERRED
  Primitives 7b/7c/7-sandbox-warn. Out of tranche-B4 scope. Substrate catalog already covers the 3-arm SSB discriminator; concrete impls ship as separate tranches.

D13 prompt-handler primitive — DEFERRED
  Primitive 4. Out of tranche-B4 scope.

D14 cache_edits passthrough — DEFERRED
  api.anthropic.com server-side only; local sidecar cannot honor. Cloud-Sonnet path is B-37 follow-up roadmap. Documented in compaction_ladder.rs module-level doc.

Wire-dark posture summary

All four shipped surfaces are default-OFF:
  ELAI_E25_COMPACTION_LADDER_ENABLE     -> D1 reduction + SSB emit
  ELAI_E25_SKILL_TEMPLATING_ENABLE      -> D2 expansion + SSB emit
  ELAI_E25_OPUSPLAN_OVERLAY_ENABLE      -> D3 emit guard (helper)
  ELAI_E25_SESSION_FORK_EMIT_ENABLE     -> D4 SSB emit at 3 fork sites

SCHEMA_VERSION unchanged at 172. Pattern-2 cascade respected — zero new SsbActionType variants; all new fire-points ride ClaudeCodeArchitectureEvent details_json.

Validation notes

Compile-only check pending auditor run (per CLAUDE.md feedback_no_cargo_test_unless_permitted memory + main-llm-role rule). Auditor scope:
  - cargo check -p elai-orchestrator
  - cargo check -p elai_cc_contracts
  - cargo check --workspace (wider scan; ensure no breakage from SessionFork additive field)
END E-25-tranche-B4-decisions.md
