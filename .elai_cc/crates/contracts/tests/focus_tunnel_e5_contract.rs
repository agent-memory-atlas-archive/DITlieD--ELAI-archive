// B-28 E-5a contract test: PipelinePreAssembler + PreAssembled + DisableReason
// + PreAssembledContext + Clock trait + pre_assembly_stats_for_payloads.
//
// Locks the LIBRARY-TIER public surface added in E-5a. The pre-assembler is
// synchronous (no async, no I/O, no tokio) — same posture as the E-4
// StreamingValidator. SSB action_type `pipeline_pre_assembled` is DEFERRED to
// the E-5b sub-phase once ssb.rs releases from session s-28564. The reader
// aggregator `pre_assembly_stats_for_payloads` is shipped in E-5a but returns
// zero stats today because the SsbPayload::PipelinePreAssembled variant does
// not exist yet — the API contract is locked so E-5b only needs to plug in
// the match arm.
//
// Locked decisions (task brief):
//   1.  Library-tier only. No SSB writes in E-5a; runtime wiring deferred.
//   2.  Rolling pass-rate window = VecDeque<bool> capped at history_window=10.
//   3.  Adaptive threshold default 0.80. cold_start_pass_rate = 1.0.
//   4.  Depth cap = 1 (only one in-flight pre-assembly at a time).
//   5.  PreAssembled enum: Ready / Disabled{AdaptiveThreshold} / AlreadyHaveOneInFlight.
//   6.  PreAssembledContext fields per brief item 6.
//   7.  discard(step_idx) is idempotent.
//   8.  Clock trait + SystemTimeClock — tests pin timestamps via injection.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use elai_orchestrator::cloud_mode::ExecMode;
use elai_orchestrator::focus_tunnel::{
    pre_assembly_stats_for_payloads, Clock, DisableReason, GenerationMode, PipelinePreAssembler,
    PreAssembled, PreAssembledContext, PreAssemblyStats,
};
use elai_orchestrator::research_tree::{BabyStepGrounding, BabyStepId, BabyStepSpec};
use elai_orchestrator::ssb::{
    PipelinePreAssembledPayload, SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct PinnedClock {
    ticks: Mutex<u64>,
}

impl PinnedClock {
    fn new(start: u64) -> Self {
        Self {
            ticks: Mutex::new(start),
        }
    }
}

impl Clock for PinnedClock {
    fn now_ms(&self) -> u64 {
        let mut t = self.ticks.lock().expect("pinned clock mutex poisoned");
        let v = *t;
        *t += 1;
        v
    }
}

fn make_step(id: u64, task: &str) -> BabyStepSpec {
    BabyStepSpec {
        id: BabyStepId(id),
        task: task.to_string(),
        estimated_tokens: 256,
        grounding: BabyStepGrounding::default(),
        tdd: None,
        acceptance: None,
        preferred_mode: None,
        target_file: None,
        edit_region: None,
    }
}

fn default_preassembler() -> PipelinePreAssembler {
    PipelinePreAssembler::new()
        .set_history_window(10)
        .set_adaptive_threshold(0.80)
        .set_clock(Arc::new(PinnedClock::new(1000)))
}

fn primary_targets() -> Vec<PathBuf> {
    vec![PathBuf::from("crates/orchestrator/src/foo.rs")]
}

fn alternate_targets() -> Vec<PathBuf> {
    vec![PathBuf::from("crates/orchestrator/src/bar.rs")]
}

fn assemble_ready(p: &mut PipelinePreAssembler, step_idx: usize, id: u64) -> PreAssembledContext {
    let spec = make_step(id, "edit existing function");
    let outcome = p.prepare(
        step_idx,
        &spec,
        &primary_targets(),
        /* supports_fim */ true,
        ExecMode::Local,
    );
    match outcome {
        PreAssembled::Ready { payload, .. } => payload,
        other => panic!("expected Ready, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 1 — cold start returns Ready
// ---------------------------------------------------------------------------

#[test]
fn cold_start_returns_ready() {
    let mut p = default_preassembler();
    let spec = make_step(1, "edit existing function");
    let out = p.prepare(0, &spec, &primary_targets(), true, ExecMode::Local);
    assert!(
        matches!(out, PreAssembled::Ready { step_idx: 0, .. }),
        "cold-start prepare must return Ready, got {out:?}"
    );
    assert!(
        (p.current_pass_rate() - 1.0).abs() < f32::EPSILON,
        "cold-start pass_rate must be 1.0"
    );
    assert!(!p.is_adaptively_disabled());
}

// ---------------------------------------------------------------------------
// Test 2 — threshold boundary (0.80 = Ready, 0.79 = Disabled)
// ---------------------------------------------------------------------------

#[test]
fn threshold_boundary_inclusive_at_0_80() {
    // Fill window with exactly 8 passes and 2 fails (rate = 0.80).
    let mut p = default_preassembler();
    for i in 0..8 {
        p.on_step_outcome(i, true, &[]);
    }
    for i in 8..10 {
        p.on_step_outcome(i, false, &[]);
    }
    assert!(
        (p.current_pass_rate() - 0.80).abs() < f32::EPSILON,
        "pass_rate should be exactly 0.80, got {}",
        p.current_pass_rate()
    );
    let spec = make_step(11, "edit");
    let out = p.prepare(11, &spec, &primary_targets(), true, ExecMode::Local);
    assert!(
        matches!(out, PreAssembled::Ready { .. }),
        "0.80 (>= threshold) must return Ready, got {out:?}"
    );
}

#[test]
fn threshold_boundary_below_at_0_79() {
    // Window of 100 outcomes? No — history_window=10. Use 7 passes + 3 fails → 0.7.
    // Actually use 79 passes out of 100: but window is 10. Test 0.7 (clearly below).
    let mut p = default_preassembler();
    for i in 0..7 {
        p.on_step_outcome(i, true, &[]);
    }
    for i in 7..10 {
        p.on_step_outcome(i, false, &[]);
    }
    assert!(
        (p.current_pass_rate() - 0.70).abs() < f32::EPSILON,
        "pass_rate should be 0.70, got {}",
        p.current_pass_rate()
    );
    let spec = make_step(11, "edit");
    let out = p.prepare(11, &spec, &primary_targets(), true, ExecMode::Local);
    match out {
        PreAssembled::Disabled {
            reason: DisableReason::AdaptiveThreshold,
            ..
        } => {}
        other => panic!("0.70 must return Disabled::AdaptiveThreshold, got {other:?}"),
    }
    assert!(p.is_adaptively_disabled());
}

// ---------------------------------------------------------------------------
// Test 3 — window eviction (oldest outcome drops off)
// ---------------------------------------------------------------------------

#[test]
fn window_eviction_drops_oldest() {
    let mut p = default_preassembler();
    // Push 10 fails (oldest entries).
    for i in 0..10 {
        p.on_step_outcome(i, false, &[]);
    }
    assert!(p.current_pass_rate() < f32::EPSILON, "window full of fails => 0.0");
    // Push 1 pass — this evicts the FIRST fail, leaving 9 fails + 1 pass = 0.10.
    p.on_step_outcome(10, true, &[]);
    let rate = p.current_pass_rate();
    assert!(
        (rate - 0.10).abs() < 1e-6,
        "after evicting one fail, rate should be 0.10, got {rate}"
    );
}

// ---------------------------------------------------------------------------
// Test 4 — depth cap (second prepare while one in-flight returns AlreadyHaveOneInFlight)
// ---------------------------------------------------------------------------

#[test]
fn depth_cap_enforced() {
    let mut p = default_preassembler();
    let spec_a = make_step(1, "first edit");
    let first = p.prepare(0, &spec_a, &primary_targets(), true, ExecMode::Local);
    assert!(matches!(first, PreAssembled::Ready { step_idx: 0, .. }));

    let spec_b = make_step(2, "second edit");
    let second = p.prepare(1, &spec_b, &alternate_targets(), true, ExecMode::Local);
    match second {
        PreAssembled::AlreadyHaveOneInFlight { existing_step_idx } => {
            assert_eq!(existing_step_idx, 0, "must report the existing in-flight step_idx");
        }
        other => panic!("expected AlreadyHaveOneInFlight, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 5 — discard is idempotent
// ---------------------------------------------------------------------------

#[test]
fn discard_idempotent() {
    let mut p = default_preassembler();
    let _ctx = assemble_ready(&mut p, 0, 1);
    let first = p.discard(0);
    assert!(first.is_some(), "first discard returns the stored payload");
    let second = p.discard(0);
    assert!(second.is_none(), "second discard on same step is a no-op");
}

// ---------------------------------------------------------------------------
// Test 6 — discard clears the in-flight slot
// ---------------------------------------------------------------------------

#[test]
fn discard_clears_in_flight_slot() {
    let mut p = default_preassembler();
    let _ctx = assemble_ready(&mut p, 0, 1);
    let _ = p.discard(0);

    let spec_b = make_step(2, "after discard");
    let out = p.prepare(1, &spec_b, &alternate_targets(), true, ExecMode::Local);
    assert!(
        matches!(out, PreAssembled::Ready { step_idx: 1, .. }),
        "after discard, next prepare must return Ready, got {out:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 7 — target_file change triggers discard via on_step_outcome
// ---------------------------------------------------------------------------

#[test]
fn target_file_change_triggers_discard() {
    let mut p = default_preassembler();
    let _ctx = assemble_ready(&mut p, 0, 1);
    assert!(p.has_in_flight(), "precondition: an in-flight payload exists");

    // Next step's target_file differs (disjoint set) — discard should fire.
    let new_targets = alternate_targets();
    p.on_step_outcome(0, true, &new_targets);
    assert!(
        !p.has_in_flight(),
        "in-flight payload must be discarded when next step's targets are disjoint"
    );
}

#[test]
fn target_file_unchanged_does_not_discard() {
    let mut p = default_preassembler();
    let _ctx = assemble_ready(&mut p, 0, 1);
    // Next step's target_files INCLUDES the stored target — no discard.
    let same_targets = primary_targets();
    p.on_step_outcome(0, true, &same_targets);
    assert!(
        p.has_in_flight(),
        "in-flight payload must survive when next-step targets overlap"
    );
}

// ---------------------------------------------------------------------------
// Test 8 — mode_hint is precomputed by ModeSelector::select
// ---------------------------------------------------------------------------

#[test]
fn mode_hint_precomputed_for_edit_step() {
    let mut p = default_preassembler();
    let spec = make_step(42, "edit the function body in place");
    let out = p.prepare(0, &spec, &primary_targets(), true, ExecMode::Local);
    match out {
        PreAssembled::Ready { payload, .. } => {
            // We do NOT require a specific GenerationMode here (the ModeSelector
            // heuristic on bare task text is conservative). We DO require that
            // mode_hint is populated (Some) so the WIRING phase can read the
            // pre-computed selection without recomputing.
            assert!(
                payload.mode_hint.is_some(),
                "mode_hint must be precomputed by prepare()"
            );
            // Sanity: GenerationMode is one of the three known variants.
            let mode = payload.mode_hint.expect("checked Some above").mode;
            assert!(
                matches!(
                    mode,
                    GenerationMode::Fim | GenerationMode::Diff | GenerationMode::Structured
                ),
                "mode_hint.mode must be a known variant, got {mode:?}"
            );
        }
        other => panic!("expected Ready, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 9 — adaptive recovery: after fails, several passes recover rate
// ---------------------------------------------------------------------------

#[test]
fn adaptive_recovery_re_enables() {
    let mut p = default_preassembler();
    // Sink to 0.0 with 10 fails.
    for i in 0..10 {
        p.on_step_outcome(i, false, &[]);
    }
    let spec = make_step(11, "edit");
    let blocked = p.prepare(11, &spec, &primary_targets(), true, ExecMode::Local);
    assert!(matches!(blocked, PreAssembled::Disabled { .. }));

    // Push 10 passes — window now full of trues, rate = 1.0.
    for i in 10..20 {
        p.on_step_outcome(i, true, &[]);
    }
    assert!(
        (p.current_pass_rate() - 1.0).abs() < f32::EPSILON,
        "after 10 recovery passes, rate must be 1.0"
    );
    let recovered = p.prepare(21, &spec, &primary_targets(), true, ExecMode::Local);
    assert!(
        matches!(recovered, PreAssembled::Ready { .. }),
        "after recovery, prepare must return Ready, got {recovered:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 10 — deterministic clock pins assembled_at_ms
// ---------------------------------------------------------------------------

#[test]
fn deterministic_clock_pins_assembled_at_ms() {
    let pinned = Arc::new(PinnedClock::new(12_345));
    let mut p = PipelinePreAssembler::new()
        .set_history_window(10)
        .set_adaptive_threshold(0.80)
        .set_clock(pinned);
    let spec = make_step(7, "edit");
    let out = p.prepare(0, &spec, &primary_targets(), true, ExecMode::Local);
    match out {
        PreAssembled::Ready { payload, .. } => {
            assert_eq!(payload.assembled_at_ms, 12_345, "clock must pin timestamp");
        }
        other => panic!("expected Ready, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 11 — stats_for_empty_payloads returns zero stats (E-5b placeholder)
// ---------------------------------------------------------------------------

#[test]
fn stats_for_empty_payloads_returns_zero() {
    let empty: Vec<SsbPayload> = Vec::new();
    let stats = pre_assembly_stats_for_payloads(&empty);
    assert_eq!(stats, PreAssemblyStats::default(), "empty input must produce default zero stats");
    assert_eq!(stats.total_assembled, 0);
    assert_eq!(stats.total_discarded, 0);
    assert!(stats.waste_rate.abs() < f32::EPSILON);
    assert!(stats.avg_depth.abs() < f32::EPSILON);
}

// ---------------------------------------------------------------------------
// Test 12 — referenced_symbols stays empty in E-5a (KnowledgeHandle deferred)
// ---------------------------------------------------------------------------

#[test]
fn referenced_symbols_empty_in_e5a() {
    let mut p = default_preassembler();
    let spec = make_step(1, "edit function");
    let out = p.prepare(0, &spec, &primary_targets(), true, ExecMode::Local);
    match out {
        PreAssembled::Ready { payload, .. } => {
            assert!(
                payload.referenced_symbols.is_empty(),
                "referenced_symbols must remain empty until KnowledgeHandle ships"
            );
        }
        other => panic!("expected Ready, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 13 — PreAssembledContext exposes baby_step_id + task_summary + target_files
// ---------------------------------------------------------------------------

#[test]
fn pre_assembled_context_carries_step_metadata() {
    let mut p = default_preassembler();
    let spec = make_step(99, "edit something nontrivial");
    let targets = primary_targets();
    let out = p.prepare(5, &spec, &targets, true, ExecMode::Local);
    match out {
        PreAssembled::Ready { payload, .. } => {
            assert_eq!(payload.baby_step_id, 99);
            assert!(payload.task_summary.contains("nontrivial"));
            assert_eq!(payload.target_files, targets);
            assert!(payload.bytes_assembled > 0, "bytes_assembled must reflect materialised payload");
        }
        other => panic!("expected Ready, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// B-28 WIRING-1 — PipelinePreAssembled SSB round-trip + aggregator wired
// ---------------------------------------------------------------------------

/// SCHEMA_VERSION bumped to 35 atomically with 3 new variants.
#[test]
fn ssb_schema_version_is_35_after_wiring1() {
    assert_eq!(
        SCHEMA_VERSION_PUBLIC, 35,
        "WIRING-1 must bump SSB schema 32 -> 35 atomically (PipelinePreAssembled / DependencyResolved / MicroStepCreated)",
    );
}

/// `SsbActionType::PipelinePreAssembled` and its payload must round-trip
/// via msgpack and report the matching action_type.
#[test]
fn pipeline_pre_assembled_ssb_round_trip() {
    let payload = SsbPayload::PipelinePreAssembled(PipelinePreAssembledPayload {
        step_idx: 7,
        pass_rate_bp: 8500,
        was_discarded: false,
        depth_at_time: 1,
        bytes_assembled: 4096,
    });

    // Discriminant -> action_type wire form.
    assert_eq!(payload.action_type(), SsbActionType::PipelinePreAssembled);
    assert_eq!(
        payload.action_type().as_sql_str(),
        "pipeline_pre_assembled"
    );

    // msgpack round-trip.
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, payload, "msgpack round-trip must preserve payload");
}

/// `pre_assembly_stats_for_payloads` must aggregate real
/// `PipelinePreAssembled` rows (counts + waste_rate + avg_depth + bytes).
#[test]
fn pre_assembly_stats_aggregates_real_rows() {
    let rows = vec![
        SsbPayload::PipelinePreAssembled(PipelinePreAssembledPayload {
            step_idx: 0,
            pass_rate_bp: 10_000,
            was_discarded: false,
            depth_at_time: 1,
            bytes_assembled: 1024,
        }),
        SsbPayload::PipelinePreAssembled(PipelinePreAssembledPayload {
            step_idx: 1,
            pass_rate_bp: 9000,
            was_discarded: false,
            depth_at_time: 1,
            bytes_assembled: 2048,
        }),
        SsbPayload::PipelinePreAssembled(PipelinePreAssembledPayload {
            step_idx: 2,
            pass_rate_bp: 7500,
            was_discarded: true,
            depth_at_time: 1,
            bytes_assembled: 0,
        }),
        SsbPayload::PipelinePreAssembled(PipelinePreAssembledPayload {
            step_idx: 3,
            pass_rate_bp: 7200,
            was_discarded: true,
            depth_at_time: 1,
            bytes_assembled: 0,
        }),
    ];
    let stats = pre_assembly_stats_for_payloads(&rows);
    assert_eq!(stats.total_assembled, 2);
    assert_eq!(stats.total_discarded, 2);
    assert!(
        (stats.waste_rate - 0.5).abs() < f32::EPSILON,
        "waste_rate must be 0.5 with 2 discarded of 4 total; got {}",
        stats.waste_rate
    );
    assert!(
        (stats.avg_depth - 1.0).abs() < f32::EPSILON,
        "avg_depth must be 1.0 over 4 rows of depth=1; got {}",
        stats.avg_depth
    );
    assert_eq!(stats.total_bytes_assembled, 1024 + 2048);
}
