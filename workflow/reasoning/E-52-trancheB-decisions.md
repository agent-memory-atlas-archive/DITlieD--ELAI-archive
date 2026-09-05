E-52-trancheB-decisions.md

Session: s-22600 Tranche B ITEM B.3 (E-52-S3 FSM driver outcome thread)
Date: 2026-05-31
Status: PARTIAL -- data thread shipped; gate-firing call site deferred

Scope landed

The per-file `DiagnosticProbeOutcome` reducer and the return-type change on `run_validate_dispatch` are SHIPPED. The TODO[E-52-S3] marker at task_runner.rs:11969 has been removed. The remaining work -- constructing a `TransitionContext { diagnostic_probe_outcome: Some(_), ..default }` and calling `evidence::check_transition(FsmState::Executing, FsmState::Auditing, &ctx)` at the right architectural boundary -- is deferred to a separate followup. This doc captures why.

What shipped

1. `diagnostic_probe::rank_worst(a, b) -> &Out` helper (diagnostic_probe.rs). Pure function. Ranks per-file outcomes by their would-BLOCK posture against `DiagnosticProbeGate.check` (evidence.rs:666-697). Tie-breaks by error_count so the loudest probe wins among same-rank pairs.
   - Rank 2: `is_authoritative() && !is_clean()` -- would BLOCK the gate
   - Rank 1: `!is_authoritative()` -- `TimedOut` / `ToolMissing`, soft-pass
   - Rank 0: `is_authoritative() && is_clean()` -- clean Completed
2. `run_validate_dispatch` signature change: `async fn ... -> ()` becomes `async fn ... -> Option<DiagnosticProbeOutcome>`. Ok branch folds per-file outcomes via `rank_worst` into a single `Option`; Err branch returns `None` (no probe data available); the env-gate-OFF path also returns `None`.
3. Caller at task_runner.rs:8275 captures the returned Option into `_diagnostic_probe_outcome`. Underscore-prefix binding documents the future hookup site without triggering unused-variable warnings.

What was DELIBERATELY NOT shipped (per prompt scope)

Per Tranche B prompt:
> DO NOT also wire the (Executing, Auditing) TransitionContext build site (gate-firing is a separate followup -- note it in workflow/reasoning/E-52-trancheB-decisions.md)

The gate-firing call site was deliberately left out. The architectural reason: `run_validate_dispatch` does not own the FSM transition context -- the orchestrator FSM driver does. Adding a `check_transition(Executing, Auditing, &ctx)` call at the `drive_task` post-validate boundary would introduce a NEW gate-enforcement site, potentially flipping the FSM driver semantics in ways the explorer flagged as MED-HIGH risk:

- drive_task does NOT today call check_transition(EXECUTING, AUDITING). New gate-enforcement site is structural change.
- FSM driver may own this transition exclusively; the right surface might be fsm.rs receiving a per-task-stash from drive_task via a setter (P2 from context-package), NOT drive_task building the ctx in-band (P1).
- This decision needs verification of the EXECUTING -> AUDITING transition site in fsm.rs and an explicit choice between P1 (drive_task owns ctx-build + check) and P2 (drive_task writes to a stash; FSM driver reads + builds ctx + checks).

Followup work registered

Open question: which architectural surface owns the FSM gate firing for EXECUTING -> AUDITING?

Two candidate paths from the context-package:
  P1 (drive_task owns): change drive_task to build TransitionContext at the post-validate boundary and call evidence::check_transition. Block on Err -> emit BLOCK Finding + abort or branch. Cleanest data flow; introduces a new gate-enforcement site outside fsm.rs.
  P2 (fsm.rs owns + stash channel): add ResolvedConfig.last_diagnostic_probe_outcome: Mutex<Option<DiagnosticProbeOutcome>>; run_validate_dispatch / drive_task writes; FSM driver reads + builds ctx + checks. Coupling-via-mutex; preserves single-owner FSM-driver semantics; rejected by P1 cleanliness arg.

Recommended next step: grep fsm.rs for the EXECUTING -> AUDITING transition site BEFORE picking. If fsm.rs has a clear transition method that already accepts a per-task context, P2 is the obvious extension. If fsm.rs's transition is a thin state-machine flip with no per-task context, P1 wins.

DiagnosticProbeBlock beacon emission also stays deferred (per E-52-tranche-B3-decisions.md NOT-DONE list -- a 3-line addition that will flip once a real subprocess landing happens AND the gate fires).

Grounding

- workflow/sessions/s-22600/exploration/trancheB-integration.md SUB-3 (load-bearing call-site question).
- workflow/sessions/s-22600/context-package-trancheB.md ITEM B.3 (P1 vs P2 decision matrix).
- workflow/reasoning/E-52-tranche-B3-decisions.md section (5) (canonical guidance: "run_validate_dispatch does not own the FSM transition context -- the orchestrator FSM driver does").
- .elai_cc/crates/orchestrator/src/evidence.rs:655-697 (DiagnosticProbeGate::check soft-pass semantics).
- .elai_cc/crates/knowledge/src/diagnostic.rs:168 (`is_authoritative()` definition).
- .elai_cc/crates/orchestrator/src/diagnostic_probe.rs:364 (`synthetic_error_outcome` precedent for the constructor / helper section).
