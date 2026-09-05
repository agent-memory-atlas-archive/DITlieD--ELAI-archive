B-21 Phase 4F Feasibility Probe — DECISIONS
Date: 2026-06-05 (backfill)
Source: workflow/sessions/s-plan-audit-2026-06-04/plan-status/B/B-21_phase_4F_feasibility_probe.md

SHIPPED (all deliverables wired)
- feasibility_probe.rs (crates/orchestrator/src/feasibility_probe.rs, 1500+ lines) + system_probe.rs.
- Types: SystemProbeResult, FeasibilityReport, StubResult, BabyStepFeasibilityReport,
  BabyStepStubResult, FeasibilityProber trait, FeasibilityEscalation, ResourceMargins,
  TaskFeasibility.
- SSB action types (ssb.rs:8849-9841): system_probe, feasibility_report, novel_step_detected,
  stub_run, alternative_explored, feasibility_escalated.
- FeasibilityPanel.svelte; get_feasibility_report Tauri command (commands.rs:721, lib.rs:541);
  FsmState::FeasibilityProbe wired in evidence.rs:1712.
- Production reach: task_runner.rs::drive_task calls emit_feasibility_probe_for_pipeline (6357) and
  emit_baby_step_feasibility_probe (6635); TaskRunnerConfig.feasibility_prober (2662) injects the
  prober. Not test-only.

KEY DECISIONS
- C-22 SSB rename (WIRING_DIVERGENCE_RESOLVED): feasibility_probe_started -> system_probe;
  feasibility_probe_complete -> feasibility_report. The original plan names were superseded.
- ELAI_DISABLE_FEASIBILITY_PROBE is a kill-switch (set truthy disables the probe), NOT a default-OFF
  enable. Default-OFF here means the probe RUNS (the safe default). It already has its own row in
  wire-dark-allowlist.toml under the base-phase-4F reconciliation (promotion_plan_id=C-22); no new
  B-21 allowlist row is required.

DEFERRED / NOT DONE
- None functional. The only original gap was this decisions doc (now backfilled).

NOTES
- wiring_contract_check returned all FAIL/hits=0 due to prose/multi-line symbol descriptions the
  grep checker cannot match; codebase grep + elai.map confirm full implementation. Plan ACCEPTANCE
  checkboxes were unchecked, but the code is ground truth.
