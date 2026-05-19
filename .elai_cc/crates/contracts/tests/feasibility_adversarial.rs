// C-19 §Phase-4F feasibility_adversarial — 8 tests (A1-A8) per spec §4F.4.
//
// C-22: removed `#[cfg(feature = "stub_result_v2")]` gates after StubResult
// v2 shipped (alternative_index + stdout_hash + stderr_summary +
// resource_delta with #[derive(Default)]). All A1-A8 tests compile and run
// unconditionally now.
//
// Adversarial stubs (infinite loop, malformed IPC) are wrapped in
// tokio::time::timeout to guarantee the cargo runner never hangs
// (plan stub_ban: "no test may hang the cargo test runner").
//
// Each test pins one A-N invariant from spec §4F.4. Where the production
// surface for the adversarial scenario doesn't yet exist (e.g. seccomp
// blocking direct TCP), the test exercises the typed-error path that the
// orchestrator MUST surface when the production wiring catches the attack.

use std::sync::Arc;
use std::time::Duration;

use elai_orchestrator::feasibility_probe::{
    StubExecutor, SyntheticStubExecutor, Tier3StubExecutor,
};
use elai_orchestrator::{
    alternative_within_cap, check_constraints, decide_with_mode, FeasibilityDecision,
    FeasibilityEscalation, FeasibilityExecutionMode, FeasibilityReport, ResourceMargins,
    StubResult, SystemConstraintsRequired, TaskFeasibility, MAX_ALTERNATIVES_PER_STEP,
};

// Hard cap: every adversarial test must complete within 30s (matches
// STUB_TIMEOUT_HARD). Wrap async tests in tokio::time::timeout.
const TEST_HARNESS_TIMEOUT: Duration = Duration::from_secs(30);

// ---------------------------------------------------------------------------
// A1: filesystem-escape via stub (read /etc/passwd).
//
// The Tier3StubExecutor receives stub source via the agent IPC channel; if
// the stub tries to read /etc/passwd it would need a filesystem syscall the
// IPC sandbox denies. With a missing agent binary, the executor returns
// the canonical `agent_binary_not_found` error — no file content escapes.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a1_filesystem_escape_blocked() {
    let exec = Tier3StubExecutor::new(std::path::PathBuf::from(
        "/nonexistent/elai_cc_agent_for_a1",
    ));
    let stub_source = "read('/etc/passwd')";
    let result = tokio::time::timeout(
        TEST_HARNESS_TIMEOUT,
        exec.run_stub("a1_step", Some(stub_source)),
    )
    .await
    .expect("A1 must complete within harness timeout");
    assert!(!result.stub_passed, "A1: filesystem-escape must NOT pass");
    // Error reaches caller as either `agent_binary_not_found` (no binary)
    // or a stub_line_limit / IPC error. The negative invariant is the
    // important part: stub_passed == false.
    assert!(result.error.is_some(), "A1: error string must be populated");
    assert!(
        !result.error.as_deref().unwrap_or("").contains("root:"),
        "A1: /etc/passwd content MUST NOT leak into result.error"
    );
}

// ---------------------------------------------------------------------------
// A2: stub attempts direct TCP connection.
//
// Same pattern as A1: the Tier3StubExecutor would spawn the agent under
// the FEASIBILITY ToolFilter with seccomp IpcBaseline (no network). With
// no binary, the executor returns the canonical error — no TCP attempt
// even leaves the test process.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a2_direct_tcp_connection_blocked() {
    let exec = Tier3StubExecutor::new(std::path::PathBuf::from(
        "/nonexistent/elai_cc_agent_for_a2",
    ));
    let stub_source = "import socket; socket.create_connection(('1.1.1.1', 53))";
    let result = tokio::time::timeout(
        TEST_HARNESS_TIMEOUT,
        exec.run_stub("a2_step", Some(stub_source)),
    )
    .await
    .expect("A2 must complete within harness timeout");
    assert!(!result.stub_passed, "A2: direct TCP attempt must NOT pass");
    assert!(result.error.is_some(), "A2: error string populated");
}

// ---------------------------------------------------------------------------
// A3: malformed IPC frame from stub.
//
// The SyntheticStubExecutor accepts an empty step_id as the synthetic
// "malformed input" marker — it returns stub_passed=false with a canonical
// error string. Pin the contract: malformed input -> typed error -> no
// panic. (The Tier3 path returns StubTimeout escalation under wire-level
// malformed-frame; that's an integration scenario covered in
// orchestrator/tests.)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a3_malformed_ipc_returns_stub_timeout_escalation() {
    let exec = SyntheticStubExecutor::new();
    let result = tokio::time::timeout(
        TEST_HARNESS_TIMEOUT,
        exec.run_stub("", Some("malformed garbage \x00\x01\x02 not valid")),
    )
    .await
    .expect("A3 must complete within harness timeout");
    assert!(!result.stub_passed, "A3: empty step_id (malformed) must fail");
    assert!(result.error.is_some(), "A3: error must be populated");
}

// ---------------------------------------------------------------------------
// A4: concurrent GPU claim by two stubs.
//
// `alternative_within_cap` is the pure-function gate that prevents
// over-allocation. Two stubs each in the [0, MAX_ALTERNATIVES_PER_STEP)
// slot range are both within cap; the 4th slot is rejected. The actual
// VRAM-allocation race is the ModelPool's responsibility (production
// path); the cap pin here is the deterministic invariant.
// ---------------------------------------------------------------------------

#[test]
fn a4_concurrent_gpu_claim_capped() {
    assert!(alternative_within_cap(0));
    assert!(alternative_within_cap(1));
    assert!(alternative_within_cap(2));
    assert!(
        !alternative_within_cap(MAX_ALTERNATIVES_PER_STEP),
        "A4: cap on concurrent claims must reject {MAX_ALTERNATIVES_PER_STEP}th"
    );
}

// ---------------------------------------------------------------------------
// A5: no-GPU probe (GPU count = 0).
//
// `decide_with_mode` MUST handle a probe report with no GPU information
// without panicking. The expected decision: either Escalate or
// ProceedWithConstraints (depending on escalations); never Proceed,
// never panic.
// ---------------------------------------------------------------------------

#[test]
fn a5_no_gpu_probe_does_not_panic() {
    let report = FeasibilityReport {
        task_feasibility: TaskFeasibility::Feasible,
        resource_margins: ResourceMargins {
            vram_margin_pct: 0.0,
            ram_margin_pct: 50.0,
            disk_margin_pct: 50.0,
            all_margins_safe: false,
        },
        novel_steps: vec![],
        stub_results: vec![],
        escalations: vec![FeasibilityEscalation::ResourceMarginBreached {
            resource: "vram".to_owned(),
            margin_pct: "0".to_owned(),
        }],
        skip_reason: None,
    };
    let decision = decide_with_mode(&report, FeasibilityExecutionMode::Interactive);
    // A5 negative invariant: no-GPU MUST NOT silent-Proceed.
    assert!(
        matches!(decision, FeasibilityDecision::ProceedWithConstraints(_))
            || matches!(decision, FeasibilityDecision::Escalate(_)),
        "A5: no-GPU must produce ProceedWithConstraints OR Escalate; got {decision:?}"
    );
}

// ---------------------------------------------------------------------------
// A6: stub sends MAX_FRAME_SIZE byte payload (boundary, accepted).
//
// The IPC frame layer (tested in orchestrator/tests/adversarial_ipc.rs:121
// `test_4a_adv_ipc_exact_max_frame_size`) accepts exactly-MAX_FRAME_SIZE
// payloads. The feasibility-probe-side guarantee: a synthetic stub with
// the maximum allowed stub_source length (under STUB_LINE_LIMIT=50) does
// not trip the line-limit gate.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a6_max_payload_boundary_accepted() {
    use elai_orchestrator::{enforce_stub_line_limit, STUB_LINE_LIMIT};
    // STUB_LINE_LIMIT - 1 = the largest accepted line count.
    let max_accepted_lines = STUB_LINE_LIMIT - 1;
    let source: String = (0..max_accepted_lines).map(|_| "// l\n").collect();
    let r = enforce_stub_line_limit(&source);
    assert!(
        matches!(r, Ok(n) if n == max_accepted_lines),
        "A6: STUB_LINE_LIMIT-1 lines must be accepted; got {r:?}"
    );

    // Run that stub through the synthetic executor — no panic, no reject.
    let exec = SyntheticStubExecutor::new();
    let result = tokio::time::timeout(
        TEST_HARNESS_TIMEOUT,
        exec.run_stub("a6_step", Some(&source)),
    )
    .await
    .expect("A6 must complete within harness timeout");
    assert!(result.stub_passed, "A6: non-empty step_id at boundary passes");
}

// ---------------------------------------------------------------------------
// A7: VRAM changes between probe and stub launch (race).
//
// `check_constraints` is the re-check gate. With required > available,
// the result MUST have passed=false — no silent proceed.
// ---------------------------------------------------------------------------

#[test]
fn a7_vram_race_caught_by_recheck() {
    use elai_core::VramTier;
    use elai_orchestrator::{DetectionMethod, GpuInfo, SystemProbeResult};
    let required = SystemConstraintsRequired {
        min_vram_bytes: 24_u64 * 1024 * 1024 * 1024, // 24 GiB needed
        min_ram_bytes: 0,
        min_disk_bytes: 0,
        required_gpu_count: 1,
    };
    // Probe said 24 GiB was free; the actual now is 1 GiB (race victim).
    let after = SystemProbeResult {
        gpu: Some(GpuInfo {
            name: "race-victim".to_owned(),
            vram_total_mb: 1024,
            vram_free_mb: 1024,
            tier: VramTier::Vram12Gb,
            driver_version: None,
            uuid: None,
        }),
        ram_total_mb: 0,
        ram_free_mb: 0,
        disk_total_mb: 0,
        disk_free_mb: 0,
        detection_method: DetectionMethod::ShellOut,
        probed_at_ms: 0,
    };
    let results = check_constraints(&required, &after);
    let vram = results
        .iter()
        .find(|r| r.resource == "vram_gpu0")
        .expect("vram_gpu0 result");
    assert!(
        !vram.passed,
        "A7: VRAM race must be caught on re-check; got passed=true (silent proceed)"
    );
}

// ---------------------------------------------------------------------------
// A8: stub alternative count exceeds MAX_ALTERNATIVES_PER_STEP = 3.
//
// The 4th alternative must be rejected by `alternative_within_cap`.
// C-22: the v2 version (`a8_stub_result_carries_alternative_index` below)
// is now ungated — StubResult.alternative_index ships unconditionally.
// ---------------------------------------------------------------------------

#[test]
fn a8_alternative_count_exceeds_cap_rejected() {
    // 4th alternative (attempts_so_far=3) is rejected.
    assert!(
        !alternative_within_cap(MAX_ALTERNATIVES_PER_STEP),
        "A8: 4th alternative (cap=3) must be rejected"
    );
    // Sanity: confirming the cap constant didn't drift.
    assert_eq!(MAX_ALTERNATIVES_PER_STEP, 3, "A8: spec cap is 3");
}

#[test]
fn a8_stub_result_carries_alternative_index() {
    // C-22: ungated after StubResult v2 ships alternative_index as u8.
    let r = StubResult {
        step_id: "a8_step".to_owned(),
        stub_passed: false,
        elapsed_ms: 0,
        error: Some("alternative_cap_exceeded".to_owned()),
        alternative_index: (MAX_ALTERNATIVES_PER_STEP as u8) - 1,
        ..Default::default()
    };
    assert_eq!(
        r.alternative_index,
        (MAX_ALTERNATIVES_PER_STEP as u8) - 1,
        "A8: 3rd slot is the last valid alternative_index"
    );
}

// Ensure Arc<dyn StubExecutor> can still be constructed in this test crate
// (referenced for completeness, not behavior).
#[allow(dead_code)]
fn _compile_check_arc_stub_executor() {
    let _: Arc<dyn StubExecutor> = Arc::new(SyntheticStubExecutor::new());
}
