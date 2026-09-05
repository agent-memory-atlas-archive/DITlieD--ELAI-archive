# Verification Report: phase-4F.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-4F.txt
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
Phase 4F (FEASIBILITY_PROBE) is substantially implemented with a working FeasibilityProber trait, skip logic, constraint checker, decision logic, stub line-limit enforcement, dual-timeout harness, msgpack serialization, baby-step extension types, and 23 passing contract tests. However, several spec-mandated type shapes diverge from the implementation (SystemProbeResult vs SystemProbeReport, GpuInfo missing uuid/driver_version as bytes, ResourceDelta absent, StubResult missing alternative_index/stdout_hash/stderr_summary), the 8 adversarial tests (4F.4) are entirely absent, SSB branch-per-stub-alternative is not wired, production construction paths still pass feasibility_prober: None, and 6 of 9 spec metrics in phase_4F.toml are marked unmeasured.

## Deliverables verified DONE

- FeasibilityReport struct (with task_feasibility, resource_margins, novel_steps, stub_results, escalations, skip_reason) — evidence: crates/orchestrator/src/feasibility_probe.rs:158
- FeasibilityEscalation enum (NoFeasibleAlternative, ResourceMarginBreached, StubTimeout) — evidence: crates/orchestrator/src/feasibility_probe.rs:150
- FeasibilityDecision enum (Proceed, ProceedWithConstraints, Escalate, Replan) — evidence: crates/orchestrator/src/feasibility_probe.rs:215
- ConstraintResult struct (resource, required_bytes, available_bytes, margin, passed) — evidence: crates/orchestrator/src/feasibility_probe.rs:201
- SystemConstraintsRequired struct (min_vram_bytes, min_ram_bytes, min_disk_bytes, required_gpu_count) — evidence: crates/orchestrator/src/feasibility_probe.rs:301
- FeasibilityExecutionMode enum (Interactive, Autonomous) — evidence: crates/orchestrator/src/feasibility_probe.rs:231
- FeasibilityProber trait (dyn-safe, probe + probe_baby_step, no async-trait) — evidence: crates/orchestrator/src/feasibility_probe.rs:403
- StubExecutor trait (dyn-safe, run_stub) — evidence: crates/orchestrator/src/feasibility_probe.rs:437
- SyntheticStubExecutor (test default) — evidence: crates/orchestrator/src/feasibility_probe.rs:454
- Tier3StubExecutor (real elai_cc_agent spawn, FEASIBILITY tool filter, dual 25s/30s timeout, IPC) — evidence: crates/orchestrator/src/feasibility_probe.rs:498
- enforce_stub_line_limit (strict <50 lines) — evidence: crates/orchestrator/src/feasibility_probe.rs:378
- MAX_ALTERNATIVES_PER_STEP = 3 — evidence: crates/orchestrator/src/feasibility_probe.rs:391
- alternative_within_cap pure fn — evidence: crates/orchestrator/src/feasibility_probe.rs:396
- decide_with_mode free fn (Proceed / ProceedWithConstraints / Escalate / Replan per mode) — evidence: crates/orchestrator/src/feasibility_probe.rs:331
- check_constraints free fn (pure, no I/O, per-resource margins) — evidence: crates/orchestrator/src/feasibility_probe.rs:241
- Skip criteria (novel_steps empty AND all margins >= 20%) — evidence: crates/orchestrator/src/feasibility_probe.rs:6-10, contract test:133
- GpuInfo struct (name, vram_total_mb, vram_free_mb, tier, driver_version) — evidence: crates/orchestrator/src/system_probe.rs:72
- SystemProbeReport struct (gpu, ram_total_mb, ram_free_mb, disk_total_mb, disk_free_mb, detection_method) — evidence: crates/orchestrator/src/system_probe.rs:112
- stub_results.msgpack written via rmp-serde (best-effort, non-fatal) — evidence: crates/orchestrator/src/feasibility_probe.rs:918
- BabyStepFeasibilityReport, BabyStepStubResult, BabyStepEscalation types (4F.7-4F.8) — evidence: crates/orchestrator/src/feasibility_probe.rs:169-188
- FeasibilityProber::probe_baby_step implemented (4F.8) — evidence: crates/orchestrator/src/feasibility_probe.rs:1054
- task_runner.rs emit_feasibility_probe_for_pipeline helper — evidence: crates/orchestrator/src/task_runner.rs:7703
- task_runner.rs emit_baby_step_feasibility_probe helper — evidence: crates/orchestrator/src/task_runner.rs:7782
- FeasibilityProbe FSM state registered with tool filter (FsmState::FeasibilityProbe) — evidence: crates/orchestrator/src/tool_filter.rs:479-484
- PLANNING->FEASIBILITY_PROBE transition gate — evidence: crates/orchestrator/src/gate.rs:829
- DefaultFeasibilityProber wired into dashboard AppState — evidence: crates/dashboard-v2/src-tauri/src/state.rs:440
- 23 passing contract tests in feasibility_probe_contract.rs (covers skip criteria, resource margins, novel step stubs, baby step, msgpack, env kill-switch, dyn-safety, decide modes, check_constraints, size limit, Tier3 rejection) — evidence: crates/contracts/tests/feasibility_probe_contract.rs:120-936
- phase_4F.toml metrics file present — evidence: .elai_cc/metrics/phase_4F.toml:1
- rmp-serde and blake3 in orchestrator Cargo.toml — evidence: crates/orchestrator/Cargo.toml:52,64
- nvml-wrapper present as optional dep (vlm-integration feature) — evidence: crates/orchestrator/Cargo.toml:51

## Deliverables PARTIAL

- StubResult struct
  - exists: crates/orchestrator/src/feasibility_probe.rs:141 (step_id, stub_passed, elapsed_ms, error)
  - gap: spec fields missing: alternative_index (u32), stdout_hash ([u8;32] blake3), stderr_summary (first 512 bytes), resource_delta (ResourceDelta)

- GpuInfo struct
  - exists: crates/orchestrator/src/system_probe.rs:72 (name, vram_total_mb, vram_free_mb, tier, driver_version Option)
  - gap: spec requires uuid (String, stable cross-reboot identity from nvml device.uuid()); spec names GpuInfo.total_bytes/free_bytes in bytes (u64); impl uses MB (u32); driver_version is Option<String> not String

- SystemProbeResult (spec name)
  - exists: crates/orchestrator/src/system_probe.rs:112 as SystemProbeReport
  - gap: renamed; spec fields gpu_count (u32), gpu_info (Vec<GpuInfo>), ram_total_bytes/ram_free_bytes/disk_free_bytes (u64, bytes), os (String), is_supported (bool) not present; impl uses ram_total_mb, disk_total_mb (MB units) and single Option<GpuInfo> not Vec

- SystemConstraints (spec name)
  - exists: crates/orchestrator/src/feasibility_probe.rs:301 as SystemConstraintsRequired
  - gap: renamed; functionally equivalent but name mismatch against spec DETERMINISTIC_CHECK greps

- FeasibilityProbe trait (spec name)
  - exists: crates/orchestrator/src/feasibility_probe.rs:403 as FeasibilityProber
  - gap: spec DETERMINISTIC_CHECK greps "pub trait FeasibilityProbe" (no 'r'); impl is FeasibilityProber; spec methods probe_system and run_stub are absent (probe_system is DefaultSystemProber::probe; run_stub is StubExecutor trait)

- Spec 4F.1 DETERMINISTIC_CHECK: feasibility/mod.rs layout
  - exists: all types in feasibility_probe.rs (not split into feasibility/types.rs + feasibility/mod.rs)
  - gap: spec greps target crates/orchestrator/src/feasibility/types.rs and feasibility/mod.rs; actual layout is single flat file feasibility_probe.rs + system_probe.rs

- phase_4F.toml metrics (9 required)
  - exists: .elai_cc/metrics/phase_4F.toml (file present, test_count=23, test_pass_count=23)
  - gap: system_probe_latency_ms, stub_execution_p50_ms, stub_execution_p99_ms, constraint_check_latency_us, report_serialization_bytes, skip_criteria_evaluation_us all marked unmeasured=true; compile_time_delta_s absent; spec requires numeric values

- baby_step_feasibility tests (4F.7 DETERMINISTIC_CHECK: >= 6 tests)
  - exists: 2 tests in feasibility_probe_contract.rs (contract_baby_step_probe_well_formed:208, contract_baby_step_escalation_on_failed_stub:440)
  - gap: spec requires >= 6 tests via `cargo test -p orchestrator --lib -- baby_step_feasibility`; only 2 found covering the baby-step path

- SSB branch per stub alternative (4F.5 I4)
  - exists: metrics confirm ssb_branch_per_alternative = false (phase_4F.toml:70)
  - gap: before-each-stub SSB snapshot with tag="branch:stub_{step_id}_{n}" is not implemented; winning branch merge not wired

## Deliverables MISSING

- ResourceDelta struct — searched: "ResourceDelta", "resource_delta", "vram_before_bytes", "vram_after_bytes"; no evidence in any source file
- StubResult.stdout_hash ([u8;32] blake3) — searched: "stdout_hash"; no evidence
- StubResult.stderr_summary — searched: "stderr_summary"; no evidence
- StubResult.alternative_index — searched: "alternative_index"; no evidence
- GpuInfo.uuid field — searched: "uuid", "device.uuid"; no evidence (metrics confirm absent)
- 4F.4 adversarial tests (A1-A8: filesystem escape, network access, malformed IPC, concurrent GPU, no-GPU probe, MAX_FRAME_SIZE, VRAM race, branch limit) — searched: "feasibility_adversarial", "A1", "A2", "fs_escape", "seccomp blocks"; only A6 (10MB size check) and A8 (branch cap) are partially covered by existing contract tests; A1-A5, A7 absent
- T9 SSB branch creation per stub alternative test — searched: "T9", "branch:stub_", "ssb.*branch"; no evidence in test files; metrics confirm not wired
- T13 VRAM re-check before stub launch (ModelPool.available_vram()) — searched: "available_vram", "vram_dropped", "T13"; no evidence
- T14 dual timeout test (graceful_ack, 25s/30s window) — searched: "graceful_ack", "T14", "dual timeout"; no evidence as full test; only stub_timeout_graceful constant test (contract:499) checks constant ordering not actual timing
- T15 ProceedWithConstraints single-GPU fallback test — searched: "T15", "single_gpu_fallback"; no evidence
- T16 malformed IPC frame synthesized StubResult test — searched: "T16", "malformed_ipc"; no evidence
- Production construction paths (cli/main.rs, runtime_setup.rs passing feasibility_prober: None) — searched: "feasibility_prober: None"; confirmed in metrics (production_construction_paths_wired = false)

## Deliverables UNVERIFIABLE

- cargo test results (24/24 pass per exit criteria) — reason: auditor role only; test execution not performed; metrics show 23/23 contract tests pass, 0/8 adversarial
- Stub timeout test completing in <35s (T7 infinite loop, hard kill at 30s) — reason: timing-sensitive real process spawn; synthetic executor does not exercise this path; Tier3StubExecutor contract tests only confirm agent-not-found error path

## What must be done (action list)

- [ ] Rename SystemProbeReport -> SystemProbeResult (or add type alias) to satisfy spec DETERMINISTIC_CHECK grep: crates/orchestrator/src/system_probe.rs
- [ ] Add GpuInfo.uuid field (String) and expose it via shell-out parsing: crates/orchestrator/src/system_probe.rs:72
- [ ] Add ResourceDelta struct (vram_before_bytes, vram_after_bytes, ram_before_bytes, ram_after_bytes) and wire into StubResult: crates/orchestrator/src/feasibility_probe.rs
- [ ] Add StubResult.alternative_index (u32), stdout_hash ([u8;32]), stderr_summary (String): crates/orchestrator/src/feasibility_probe.rs:141
- [ ] Add feasibility/mod.rs + feasibility/types.rs module layout OR update spec DETERMINISTIC_CHECK greps to match flat layout
- [ ] Implement SSB branch snapshot per stub alternative with tag="branch:stub_{step_id}_{n}": crates/orchestrator/src/feasibility_probe.rs (I4 item)
- [ ] Wire production construction paths (cli/main.rs, runtime_setup.rs) to pass a real DefaultFeasibilityProber instead of None
- [ ] Author 8 adversarial tests (A1-A8) as described in 4F.4 TASK: crates/contracts/tests/ or orchestrator test module
- [ ] Author missing T9, T13, T14, T15, T16 contract tests
- [ ] Add >= 4 additional baby_step_feasibility tests to reach the >= 6 spec target (4F.7)
- [ ] Measure and populate 6 unmeasured metric fields in phase_4F.toml: system_probe_latency_ms, stub_execution_p50_ms, stub_execution_p99_ms, constraint_check_latency_us, report_serialization_bytes, skip_criteria_evaluation_us, compile_time_delta_s

## Notes
The implementation is architecturally sound and reflects a deliberate divergence from the spec's exact type names (SystemProbeResult -> SystemProbeReport, FeasibilityProbe -> FeasibilityProber, flat module vs feasibility/mod.rs). The phase_4F.toml metrics file honestly documents all gaps. The primary blocking gaps for exit criteria are: the 8 adversarial tests (4F.4) are entirely absent, SSB branch wiring is missing, and production construction paths are unconnected. The spec exit criteria state "all 24 tests pass" — current state is 23 contract tests passing, 0 adversarial tests authored.
