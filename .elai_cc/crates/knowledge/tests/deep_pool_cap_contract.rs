//! E-17 SP2 task 4 — contract tests for the DEEP cap + RRF quality gate.
//!
//! Three fixtures:
//!   (a) 250-candidate pool -> assert cap to 100 per stage + dropped count.
//!   (b) 50-candidate pool below threshold -> no cap fires + zero dropped.
//!   (c) Floor gate over a scored HashMap spanning [0.001, 0.05] -> assert
//!       only scores >= RRF_FLOOR_DEFAULT survive.

use std::collections::HashMap;

use elai_knowledge::arag::RetrievalResult;
use elai_knowledge::deep::{
    deep_pool_cap, rrf_quality_gate, DEEP_CAP_DEFAULT, RRF_FLOOR_DEFAULT,
};
use elai_knowledge::pipeline::PipelineStage;

fn make_result(id: &str) -> RetrievalResult {
    RetrievalResult {
        chunk_id: id.to_string(),
        snippet: format!("snippet for {id}"),
        score: 1.0,
        tool_used: "test".to_string(),
    }
}

fn make_stage(name: &str, n: usize) -> PipelineStage {
    let results: Vec<RetrievalResult> = (0..n)
        .map(|i| make_result(&format!("{name}-{i}")))
        .collect();
    PipelineStage {
        stage: name.to_string(),
        results,
        latency_ms: 0,
        circuit_breaker_state: None,
        error: None,
    }
}

#[test]
fn cap_drops_excess_with_250_candidates_per_stage() {
    let mut stages = vec![
        make_stage("bm25", 250),
        make_stage("vector", 250),
        make_stage("code_vector", 250),
    ];
    let dropped = deep_pool_cap(&mut stages, DEEP_CAP_DEFAULT);
    // 3 stages * (250 - 100) = 450 dropped.
    assert_eq!(dropped, 450, "expected 450 total dropped, got {dropped}");
    for stage in &stages {
        assert_eq!(stage.results.len(), DEEP_CAP_DEFAULT);
    }
}

#[test]
fn cap_under_threshold_is_noop() {
    let mut stages = vec![make_stage("bm25", 50), make_stage("vector", 30)];
    let dropped = deep_pool_cap(&mut stages, DEEP_CAP_DEFAULT);
    assert_eq!(dropped, 0);
    assert_eq!(stages[0].results.len(), 50);
    assert_eq!(stages[1].results.len(), 30);
}

#[test]
fn rrf_quality_gate_drops_below_floor() {
    let mut scores: HashMap<String, (f64, RetrievalResult)> = HashMap::new();
    // span [0.001, 0.05] in increments — 50 entries, half below 0.005.
    for i in 1..=50 {
        let score = (i as f64) * 0.001;
        scores.insert(
            format!("chunk-{i}"),
            (score, make_result(&format!("chunk-{i}"))),
        );
    }
    let before = scores.len();
    let dropped = rrf_quality_gate(&mut scores, RRF_FLOOR_DEFAULT);
    // Scores 0.001, 0.002, 0.003, 0.004 dropped = 4 entries.
    assert_eq!(dropped, 4, "expected 4 dropped, got {dropped}");
    assert_eq!(scores.len(), before - 4);
    // Verify no surviving entry has score < floor.
    for (_, (score, _)) in &scores {
        assert!(*score >= RRF_FLOOR_DEFAULT);
    }
}

#[test]
fn floor_zero_is_noop() {
    let mut scores: HashMap<String, (f64, RetrievalResult)> = HashMap::new();
    scores.insert("a".into(), (0.001, make_result("a")));
    scores.insert("b".into(), (0.002, make_result("b")));
    let dropped = rrf_quality_gate(&mut scores, 0.0);
    assert_eq!(dropped, 0);
    assert_eq!(scores.len(), 2);
}

#[test]
fn cap_and_floor_constants_in_paper_envelope() {
    // arxiv:2411.11767 invariant: cap in [20, 200] range avoids both
    // under-fitting and the K=5000 phantom-hit zone (S4.SS2.p2).
    assert!(DEEP_CAP_DEFAULT >= 20 && DEEP_CAP_DEFAULT <= 200);
    // RRF floor must be strictly positive and below the single-ranker
    // max RRF score (1/(60+1) ≈ 0.0164).
    assert!(RRF_FLOOR_DEFAULT > 0.0);
    assert!(RRF_FLOOR_DEFAULT < 0.0164);
}
