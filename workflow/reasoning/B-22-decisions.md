B-22 Phase 4G Autonomous Mode — DECISIONS
Date: 2026-06-05 (backfill)
Source: workflow/sessions/s-plan-audit-2026-06-04/plan-status/B/B-22_phase_4G_autonomous_mode.md

SHIPPED
- autonomous_gate.rs: AutonomousGate (wraps InteractiveGate), AutoAdvanceEvaluator (6 evidence
  checks), TreeAutoAdvancePolicy, SplitTaskPlan, SubstituteModel.
- Production wire: runtime_setup.rs:499-512 constructs AutonomousGate::new() at the single
  production chokepoint and threads it into TaskRunnerConfig.autonomous_gate; task_runner.rs:6951-7278
  calls autonomous_gate.as_ref() in the live loop; emit_autonomous_decision (task_runner.rs:14164-
  14232) writes GateDecision + EscalationFired SSB rows.
- Dashboard: set_gate_mode + get_auto_advance_stats (commands.rs:943/973, registered lib.rs:547-548);
  MonitorWindow.svelte AutoAdvanceStats; CommandPalette.svelte:83 + SettingsWindow.svelte:357 call
  set_gate_mode.

KEY DECISIONS
- SSB action_type naming divergence (the load-bearing decision): the plan declared THREE discrete
  variants auto_advance_decision / autonomy_escalation / split_task_initiated. The implementation does
  NOT add discrete variants; instead it encodes the decision as a string LABEL inside the
  GateDecision payload (.decision field) with values "auto_advance" / "split_task" / "escalate" /
  "substitute_model". model_swap IS a real discrete variant (SsbActionType::ModelSwap, ssb.rs:8811).
  RESOLUTION: keep the GateDecision-label encoding as shipped; the contract is updated to match the
  payload-label form rather than back-filling three thin variants. The dashboard reads decisions off
  the GateDecision stream, so the label encoding is what the consumer actually needs.
- /mode live-CLI flip is delegated to the dashboard set_gate_mode IPC; headless flips happen by
  setting ELAI_GATE_MODE before launching the runner (repl_command.rs:742-759). The CLI does not
  mutate gate mode mid-session.

DEFERRED / NOT DONE
- B-22b (4-pattern stagnation detector + GenerationWatchdog) is a separate follow-up, explicitly NOT
  in original B-22 scope.

R9 / WIRE-DARK
- Two env flags were unregistered (the SHIPPED_WIRE_DARK cause), now in wire-dark-allowlist.toml:
  ELAI_DISABLE_AUTONOMOUS_GATE (kill_switch, smoke_test autonomous_gate_contract.rs::
  contract_kill_switch_requests_human) and ELAI_GATE_MODE (kill_switch, smoke_test
  c9_autonomous_infra_contract.rs::c9_contract_gate_mode_from_cli_args). Both promotion_plan_id=B-22.

NOTES
- wiring_contract_check hits=0 on AutonomousGate/AutoAdvanceEvaluator was a tool false negative; both
  exist in autonomous_gate.rs and are wired via runtime_setup.rs.
