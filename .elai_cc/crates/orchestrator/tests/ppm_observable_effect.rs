//! D-2 — PPM observable_effect smoke test.
//!
//! Trigger: call the production producer (`run_ppm_verifier_check`) with a
//! `MockSidecarPpmHook`, capture the row via `VecPpmSsbEmitter`.
//! Expect: exactly one `PpmScored` (or `PpmGatingFailed`) row, payload
//! shape matches the candidate input, and the gate decision matches the
//! score relative to the threshold.
//!
//! This satisfies the D-2 plan WIRING_CONTRACT.observable_effect clause:
//! "trigger PPM scoring, persist, query, assert shape." The real Tauri
//! `get_ppm_recent_scores` command reads from `SsbViewer::recent_rows_of_type`
//! which has dedicated SsbViewer tests; this test runs the end-to-end
//! Rust-side path against the in-memory emitter to keep the smoke
//! deterministic without spinning up sqlite.

use std::sync::Arc;

use elai_orchestrator::{
    apply_ppm_gate, run_ppm_verifier_check, MockSidecarPpmHook, PpmClient, PpmMode,
    PpmGatingFailedPayload, PpmScoredPayload, SsbPayload, VecPpmSsbEmitter,
};

#[tokio::test]
async fn ppm_observable_effect_scored_path() {
    let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.9));
    let client = PpmClient::new(hook);
    let emitter = VecPpmSsbEmitter::new();
    let score = run_ppm_verifier_check(
        &client,
        "cand-smoke-1",
        "ppm-smoke",
        "problem text",
        vec!["s1".to_string(), "s2".to_string(), "s3".to_string()],
        PpmMode::Math,
        &emitter,
    )
    .await
    .expect("score ok");
    assert!(score.gate_passed);
    assert!(apply_ppm_gate(&score));
    assert_eq!(score.num_steps, 3);

    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1, "exactly one PPM row emitted");
    match &rows[0] {
        SsbPayload::PpmScored(PpmScoredPayload {
            candidate_id,
            verifier_id,
            mode,
            num_steps,
            score_mean,
            ..
        }) => {
            assert_eq!(candidate_id, "cand-smoke-1");
            assert_eq!(verifier_id, "ppm-smoke");
            assert_eq!(mode, "math");
            assert_eq!(*num_steps, 3);
            assert!((*score_mean - 0.9).abs() < 1e-6);
        }
        other => panic!("expected PpmScored, got {other:?}"),
    }
}

#[tokio::test]
async fn ppm_observable_effect_gating_failed_path() {
    let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.3));
    let client = PpmClient::new(hook);
    let emitter = VecPpmSsbEmitter::new();
    let score = run_ppm_verifier_check(
        &client,
        "cand-smoke-2",
        "ppm-smoke",
        "problem text",
        vec!["bad step".to_string()],
        PpmMode::Code,
        &emitter,
    )
    .await
    .expect("score ok");
    assert!(!score.gate_passed);
    assert!(!apply_ppm_gate(&score));

    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1, "exactly one PPM row emitted");
    match &rows[0] {
        SsbPayload::PpmGatingFailed(PpmGatingFailedPayload {
            candidate_id,
            verifier_id,
            mode,
            score_mean,
            threshold,
            ..
        }) => {
            assert_eq!(candidate_id, "cand-smoke-2");
            assert_eq!(verifier_id, "ppm-smoke");
            assert_eq!(mode, "code");
            assert!((*score_mean - 0.3).abs() < 1e-6);
            assert!((*threshold - 0.7).abs() < 1e-6);
        }
        other => panic!("expected PpmGatingFailed, got {other:?}"),
    }
}

#[tokio::test]
async fn ppm_observable_effect_cache_repeat_call() {
    let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.95));
    let client = PpmClient::new(hook);
    let emitter = VecPpmSsbEmitter::new();
    let s1 = run_ppm_verifier_check(
        &client,
        "cand-A",
        "ppm-smoke",
        "P",
        vec!["x".to_string()],
        PpmMode::Math,
        &emitter,
    )
    .await
    .expect("ok");
    assert!(!s1.cache_hit);
    let s2 = run_ppm_verifier_check(
        &client,
        "cand-A",
        "ppm-smoke",
        "P",
        vec!["x".to_string()],
        PpmMode::Math,
        &emitter,
    )
    .await
    .expect("ok");
    assert!(s2.cache_hit);
    // Each call still emits its own SSB row even when the hook was cached.
    assert_eq!(emitter.len().await, 2);
}
