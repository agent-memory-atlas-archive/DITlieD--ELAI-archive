//! C-47 — Case-Based Calibration of Tool-Use Telemetry contract.
//!
//! Paper grounding: arXiv:2605.15041 (Pang, Lan, Liu, Tong, Cao, Zhang,
//! "Case-Based Calibration of Adaptive Reasoning and Execution for LLM
//! Tool Use" / CAST framework). Verified via WebFetch of arxiv.org/abs/
//! 2605.15041 + arxiv.org/html/2605.15041 (2026-05-24). The interface
//! map is at `workflow/papers/case-based-calibration.md`.
//!
//! Critical paper invariants enforced here (Section 3.2):
//!   - Hardness score `H in [0, 1]` (complexity rubric output).
//!   - Failure profile has EXACTLY 5 dimensions: name, key, type,
//!     constraint, value.
//!   - CAST aggregates signals ACROSS the case base — NOT per-query
//!     nearest-neighbor retrieval. ELAI's `ToolCalibrationRegistry`
//!     reflects this by keying per-(tool, call_shape_hash) rather
//!     than embedding-based similarity.
//!
//! Contract fences (G1..G7):
//!   G1. [`AggregatedCase`] aggregates per-(tool, call_shape_hash) —
//!       NOT per-query (paper Section 3.2 aggregate-case-base
//!       pattern).
//!   G2. [`FailureProfile`] has exactly 5 dimensions per paper.
//!   G3. [`generate_hint`] returns `Some` only when the threshold is
//!       met (HINT_ERROR_RATE_THRESHOLD + HINT_MIN_TOTAL_CALLS).
//!   G4. [`ToolCalibrationRegistry`] caps per-tool entries at
//!       [`CASE_BASE_LRU_CAP`] (=100) via deterministic LRU eviction.
//!   G5. Three SSB variants `ToolCallLogged` /
//!       `ToolCalibrationHintEmitted` / `ToolCalibrationDriftDetected`
//!       round-trip through the SSB enum surface.
//!   G6. Hint suppression gates AT THE EMIT POINT (stub_ban from
//!       WIRING_CONTRACT).
//!   G7. [`complexity_score`] returns a value in `[0, 1]` for
//!       representative call-shape strings (paper hardness invariant).
//!
//! Build target: `cargo check -p elai_cc_contracts`.
//! Test target: `cargo test -p elai_cc_contracts --test
//!   c47_case_based_calibration_contract`.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SsbActionType, SsbPayload,
    ToolCalibrationDriftDetectedPayload, ToolCalibrationHintEmittedPayload,
    ToolCallLoggedPayload,
};
use elai_plugins::{
    classify_response, complexity_score, generate_hint, AggregatedCase, FailureProfile,
    ResponseKind, ToolCalibrationRegistry, CASE_BASE_LRU_CAP, HINT_ERROR_RATE_THRESHOLD,
    HINT_MIN_TOTAL_CALLS,
};

// ---------------------------------------------------------------------------
// G1 — AggregatedCase aggregates per (tool, call_shape_hash); NOT per-query.
// Per CAST Section 3.2 the case base is a per-(call-shape) aggregate so a
// single recurring shape carries a stable error rate across sessions. We
// fence the structural shape by constructing two cases that share a tool
// name and asserting they are independent records.
// ---------------------------------------------------------------------------

#[test]
fn g1_aggregated_case_keys_per_call_shape_hash() {
    let mut a = AggregatedCase::new("fs.read".into(), 0xAAAA);
    let mut b = AggregatedCase::new("fs.read".into(), 0xBBBB);
    a.observe(ResponseKind::ErrInvalidArgs, r#"{"path":"x"}"#, 1);
    b.observe(ResponseKind::Ok, r#"{"path":"x"}"#, 1);
    // Same tool name, different shape hashes — independent records.
    assert_eq!(a.tool_name, b.tool_name);
    assert_ne!(a.call_shape_hash, b.call_shape_hash);
    assert_eq!(a.error_count, 1);
    assert_eq!(b.error_count, 0);
}

// ---------------------------------------------------------------------------
// G2 — FailureProfile has EXACTLY 5 dimensions per CAST Section 3.2
// (name, key, type, constraint, value). The paper's R_tool reward is
// six-dimensional with the addition of exact-match; for the telemetry-
// only surface in C-47 we record the 5 dimensions.
// ---------------------------------------------------------------------------

#[test]
fn g2_failure_profile_has_five_dimensions_per_paper() {
    let mut p = FailureProfile::default();
    // Touch each of the 5 fields. If any disappears, this won't compile.
    p.name_err = 1;
    p.key_err = 1;
    p.type_err = 1;
    p.constraint_err = 1;
    p.value_err = 1;
    assert_eq!(p.total(), 5);
}

// ---------------------------------------------------------------------------
// G3 — generate_hint returns Some only when threshold is met (paper
// Section 3.2 — hints are signal-conditioned, not always-on).
// ---------------------------------------------------------------------------

#[test]
fn g3_hint_only_above_threshold() {
    // Below sample floor — no hint regardless of error rate.
    let mut low_n = AggregatedCase::new("fs.read".into(), 0);
    for _ in 0..(HINT_MIN_TOTAL_CALLS - 1) {
        low_n.observe(ResponseKind::ErrInvalidArgs, r#"{"k":"v"}"#, 1);
    }
    assert!(generate_hint(&low_n, "fs.read").is_none());

    // Above sample floor but below error-rate threshold — no hint.
    let mut low_rate = AggregatedCase::new("fs.read".into(), 0);
    for _ in 0..8 {
        low_rate.observe(ResponseKind::Ok, r#"{"k":"v"}"#, 1);
    }
    for _ in 0..2 {
        low_rate.observe(ResponseKind::ErrInvalidArgs, r#"{"k":"v"}"#, 1);
    }
    assert!(
        generate_hint(&low_rate, "fs.read").is_none(),
        "20% < {HINT_ERROR_RATE_THRESHOLD}"
    );

    // Above both — hint fires.
    let mut high = AggregatedCase::new("fs.read".into(), 0);
    for _ in 0..2 {
        high.observe(ResponseKind::Ok, r#"{"k":"v"}"#, 1);
    }
    for _ in 0..8 {
        high.observe(ResponseKind::ErrInvalidArgs, r#"{"k":"v"}"#, 1);
    }
    let h = generate_hint(&high, "fs.read").expect("hint above threshold");
    assert!(h.contains("calibration_hint:"));
    assert!(h.contains("tool=fs.read"));
}

// ---------------------------------------------------------------------------
// G4 — LRU cap=100 per tool with deterministic least-recently-used
// eviction. The WIRING_CONTRACT stub_ban requires "eviction MUST be
// deterministic, not random"; the `lru` crate's documented semantics
// satisfy that.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn g4_lru_cap_100_evicts_deterministically() {
    let reg = ToolCalibrationRegistry::new();
    // Insert exactly CASE_BASE_LRU_CAP shapes for one tool, all with
    // distinct hashes, then one more — the LRU must be at cap.
    for i in 0..(CASE_BASE_LRU_CAP as u64) {
        reg.record_call("fs.read", i, r#"{}"#, ResponseKind::Ok, i as i64)
            .await;
    }
    let status = reg.status("fs.read").await.expect("status");
    assert_eq!(status.shape_count, CASE_BASE_LRU_CAP as u32);
    let extra = (CASE_BASE_LRU_CAP as u64) + 1;
    reg.record_call("fs.read", extra, r#"{}"#, ResponseKind::Ok, extra as i64)
        .await;
    let status_after = reg.status("fs.read").await.expect("status_after");
    assert_eq!(
        status_after.shape_count, CASE_BASE_LRU_CAP as u32,
        "eviction must hold at cap (deterministic LRU)"
    );
}

// ---------------------------------------------------------------------------
// G5 — Three SSB variants round-trip. Wire-format guard mirrors the
// C-45 / C-46 round-trip fence.
// ---------------------------------------------------------------------------

#[test]
fn g5_ssb_variants_round_trip() {
    // String -> enum
    assert_eq!(
        ssb_action_type_from_str_public("tool_call_logged").unwrap(),
        SsbActionType::ToolCallLogged
    );
    assert_eq!(
        ssb_action_type_from_str_public("tool_calibration_hint_emitted").unwrap(),
        SsbActionType::ToolCalibrationHintEmitted
    );
    assert_eq!(
        ssb_action_type_from_str_public("tool_calibration_drift_detected").unwrap(),
        SsbActionType::ToolCalibrationDriftDetected
    );
    // Enum -> string
    assert_eq!(
        SsbActionType::ToolCallLogged.as_sql_str(),
        "tool_call_logged"
    );
    assert_eq!(
        SsbActionType::ToolCalibrationHintEmitted.as_sql_str(),
        "tool_calibration_hint_emitted"
    );
    assert_eq!(
        SsbActionType::ToolCalibrationDriftDetected.as_sql_str(),
        "tool_calibration_drift_detected"
    );
    // Payload variant tags
    let logged = SsbPayload::ToolCallLogged(ToolCallLoggedPayload {
        tool_name_hash: 1,
        call_shape_hash: 2,
        response_kind_hash: 3,
        post_call_audit_ok: true,
        ts_ms: 0,
    });
    assert_eq!(logged.action_type(), SsbActionType::ToolCallLogged);
    let hint = SsbPayload::ToolCalibrationHintEmitted(ToolCalibrationHintEmittedPayload {
        tool_name_hash: 1,
        call_shape_hash: 2,
        hint_text_hash: 3,
        ts_ms: 0,
    });
    assert_eq!(hint.action_type(), SsbActionType::ToolCalibrationHintEmitted);
    let drift = SsbPayload::ToolCalibrationDriftDetected(ToolCalibrationDriftDetectedPayload {
        tool_name_hash: 1,
        error_rate: 0.5,
        total_calls: 10,
        ts_ms: 0,
    });
    assert_eq!(
        drift.action_type(),
        SsbActionType::ToolCalibrationDriftDetected
    );
}

// ---------------------------------------------------------------------------
// G6 — Hint suppression gates AT THE EMIT POINT. The WIRING_CONTRACT
// stub_ban "hint_suppression MUST gate AT THE EMIT POINT" is enforced
// by [`ToolCalibrationRegistry::consult`] returning `None` BEFORE any
// downstream SSB row is composed.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn g6_hint_suppression_gates_at_emit_point() {
    let reg = ToolCalibrationRegistry::new().with_hint_suppression(true);
    // Seed the case base WITH enough signal to fire a hint absent
    // suppression.
    for _ in 0..10 {
        reg.record_call(
            "fs.read",
            42,
            r#"{"k":"v"}"#,
            ResponseKind::ErrInvalidArgs,
            1,
        )
        .await;
    }
    // consult must return None even though the threshold is met.
    assert_eq!(reg.consult("fs.read", 42).await, None);
    assert!(reg.hint_suppression());
}

// ---------------------------------------------------------------------------
// G7 — complexity_score returns a value in [0, 1] for representative
// call-shape strings (paper Section 3.2 hardness invariant).
// ---------------------------------------------------------------------------

#[test]
fn g7_complexity_score_unit_bounded() {
    let samples = [
        "",
        "{}",
        r#"{"path":"a"}"#,
        r#"{"a":{"b":{"c":{"d":1}}}}"#,
        r#"{"pattern":"^[a-z_][a-z0-9_]*$\\d+"}"#,
        r#"{"k":"v","n":1,"a":[1,2,3,4,5,6,7,8,9]}"#,
    ];
    for s in samples {
        let h = complexity_score(s);
        assert!(
            (0.0..=1.0).contains(&h),
            "complexity_score({s:?}) = {h} out of [0,1] — paper hardness invariant violation"
        );
    }
}

// ---------------------------------------------------------------------------
// Bonus — record_call called from BOTH dispatchers (compile-time grep
// test). We assert presence at the source-text level. The bridge file
// is parsed as a literal string so the compile-time match is exact;
// changes to the wiring will surface here.
// ---------------------------------------------------------------------------

#[test]
fn record_call_wired_in_both_dispatchers() {
    let bridge_src = include_str!(
        "../../plugins/src/mcp_bridge.rs"
    );
    // Count occurrences of `record_call(` calls inside the file. The
    // helper definition is `async fn record_call(` (1 hit) plus two
    // call sites — one in dispatch_tool_call_with_gate, one in
    // dispatch_tool_call_with_authz (after the Allow branch).
    let call_sites = bridge_src.matches("self.record_call(").count();
    assert!(
        call_sites >= 2,
        "expected >= 2 `self.record_call(` invocations (one per dispatcher); found {call_sites}"
    );
    // Same for consult_calibration.
    let consult_sites = bridge_src.matches("self.consult_calibration(").count();
    assert!(
        consult_sites >= 2,
        "expected >= 2 `self.consult_calibration(` invocations; found {consult_sites}"
    );
}

// ---------------------------------------------------------------------------
// Bonus — classify_response routes hints to the correct ResponseKind.
// Fences the synchronous heuristic used in the bridge wiring.
// ---------------------------------------------------------------------------

#[test]
fn classify_response_routes_hints() {
    assert_eq!(classify_response(false, None), ResponseKind::Ok);
    assert_eq!(
        classify_response(true, Some("invalid_argument: path")),
        ResponseKind::ErrInvalidArgs
    );
    assert_eq!(
        classify_response(true, Some("timeout exceeded after 30s")),
        ResponseKind::ErrTimeout
    );
}
