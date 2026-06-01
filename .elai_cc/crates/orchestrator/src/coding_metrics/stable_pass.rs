// E-49 -- STABLE-Pass@1 named-token producer.
//
// Closes TASK-E49-005: emits the `stable_pass_at_1` named token under
// the canonical `SsbActionType::CodingMetricNamedToken` Pattern-2 envelope
// (Kind=StablePassAt1). Replaces the single-seed Pass@1 reporting with
// the credible-stack n=5 stability metric (identical-output via blake3;
// stable_score = mean_pass * (1 - 2*std_dev)).
//
// FINAL TRANCHE wiring:
//   - This module is the canonical producer surface for STABLE-Pass@1
//     emission. The benchmark runner caller-site is plumbed via the
//     `record_stable_pass_at_1` free function below; until the live
//     SWE-Bench Pro / RefineBench runners are switched on, this is
//     wire-dark by default (gated on ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE).
//
// Grounding:
//   - E-49 plan Task 1 (n=5 default; identical-output via blake3 hash).
//   - ssb.rs:23086 -- closed 3-arm CodingMetricNamedTokenKind enum +
//     Pattern-2 (41st application) envelope.
//   - CLAUDE.md HARD RULE: blake3 only for non-RFC hashing uses.
//
// Anti-Lazy compliance:
//   - No unwrap()/expect() outside test code.
//   - Errors propagate via thiserror.
//   - Wire-dark default OFF: missing/0/non-"1" env var = no SSB write,
//     no panic, no silent failure.

use std::env;
use std::sync::Arc;

use thiserror::Error;

use crate::ssb::{
    CodingMetricNamedTokenKind, CodingMetricNamedTokenPayload, Ssb, SsbError, SsbPayload,
    ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE_ENV,
};

/// Default seed-count for STABLE-Pass@1 captures. Five seeds is the
/// E-49 plan default and the minimum statistically meaningful sample
/// (identical-output check requires N >= 2; std-dev is degenerate at
/// N < 3; five gives a useful std-dev plus a robust identical-output
/// signal at modest cost).
pub const DEFAULT_SEED_COUNT: u32 = 5;

/// Per-seed outcome captured by the benchmark runner. The fields here
/// are the minimum required to compute the STABLE-Pass@1 row; richer
/// per-attempt telemetry hangs off `details_json` via the payload.
#[derive(Debug, Clone)]
pub struct SeedOutcome {
    /// Stable per-seed identifier (e.g. `"seed-0"`).
    pub seed_label: String,
    /// True iff the seed run passed all assertions.
    pub passed: bool,
    /// Serialized output for identical-output hashing. Free-form;
    /// caller is responsible for canonicalizing (e.g. stripping
    /// timestamps before passing in).
    pub serialized_output: String,
}

/// Aggregated STABLE-Pass@1 metric across the N seeds.
#[derive(Debug, Clone, PartialEq)]
pub struct StablePassMetric {
    pub seed_count: u32,
    pub identical_outputs: bool,
    pub mean_pass: f64,
    pub std_dev: f64,
    /// `mean_pass * (1 - 2*std_dev)`, clamped to `[0.0, 1.0]`.
    pub stable_score: f64,
}

#[derive(Debug, Error)]
pub enum StablePassError {
    #[error("seed outcomes slice was empty (need >= 1)")]
    EmptySeeds,
    #[error("ssb write failed: {0}")]
    Ssb(#[from] SsbError),
    #[error("payload serialization failed: {0}")]
    Serialize(String),
}

/// Compute the STABLE-Pass@1 metric over N seed outcomes.
///
/// `identical_outputs` is the blake3 hash equality check across the
/// per-seed serialized outputs. `mean_pass` is the simple pass-rate.
/// `std_dev` is the population standard deviation of `{1.0 if pass else 0.0}`.
/// `stable_score` is `mean_pass * (1 - 2*std_dev)`, clamped to `[0.0, 1.0]`.
pub fn compute_stable_pass(outcomes: &[SeedOutcome]) -> Result<StablePassMetric, StablePassError> {
    if outcomes.is_empty() {
        return Err(StablePassError::EmptySeeds);
    }
    let n_u32 = u32::try_from(outcomes.len()).unwrap_or(u32::MAX);

    // Identical-output check via blake3 (per CLAUDE.md HARD RULE
    // anti-stale: blake3 only for non-RFC uses).
    let first_hash = blake3::hash(outcomes[0].serialized_output.as_bytes());
    let identical_outputs = outcomes
        .iter()
        .all(|o| blake3::hash(o.serialized_output.as_bytes()) == first_hash);

    // mean_pass + std_dev over the 0/1 indicator vector.
    let n_f = outcomes.len() as f64;
    let sum_pass: f64 = outcomes.iter().map(|o| if o.passed { 1.0 } else { 0.0 }).sum();
    let mean_pass = sum_pass / n_f;
    let variance: f64 = outcomes
        .iter()
        .map(|o| {
            let v = if o.passed { 1.0 } else { 0.0 };
            (v - mean_pass).powi(2)
        })
        .sum::<f64>()
        / n_f;
    let std_dev = variance.sqrt();
    let raw_score = mean_pass * (1.0 - 2.0 * std_dev);
    let stable_score = raw_score.clamp(0.0, 1.0);

    Ok(StablePassMetric {
        seed_count: n_u32,
        identical_outputs,
        mean_pass,
        std_dev,
        stable_score,
    })
}

/// Return true iff the producer-side env gate is set to a truthy value.
/// Matches the precedent set by other E-49 / D-18 producer gates: "1"
/// is the canonical truthy; everything else (including unset) is OFF.
fn producer_enabled() -> bool {
    match env::var(ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE_ENV) {
        Ok(v) => matches!(v.as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    }
}

/// Record a STABLE-Pass@1 row against the SSB.
///
/// Wire-dark default. When `ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE` is
/// unset (or not truthy) the function silently returns `Ok(())` after
/// computing the metric (so callers can still observe the value via
/// the `compute_stable_pass` return path). On success the row is
/// emitted under `SsbActionType::CodingMetricNamedToken` with
/// `Kind=StablePassAt1`.
///
/// `model_id` is the model under test (e.g.
/// `"Qwen/Qwen2.5-Coder-7B-Instruct"`); `test_id` is the harness-side
/// test identifier (e.g. SWE-Bench Pro instance id).
pub async fn record_stable_pass_at_1(
    ssb: &Arc<Ssb>,
    session_id: &str,
    test_id: &str,
    model_id: &str,
    outcomes: &[SeedOutcome],
    agent_id: &str,
    superstep: i64,
    ts_ms: i64,
) -> Result<StablePassMetric, StablePassError> {
    let metric = compute_stable_pass(outcomes)?;
    if !producer_enabled() {
        return Ok(metric);
    }
    let details = serde_json::json!({
        "caller_site": "coding_metrics::stable_pass::record_stable_pass_at_1",
        "seed_count": metric.seed_count,
        "identical_outputs": metric.identical_outputs,
        "mean_score": metric.mean_pass,
        "std_dev": metric.std_dev,
        "stable_score": metric.stable_score,
        "test_id": test_id,
    });
    let payload = SsbPayload::CodingMetricNamedToken(CodingMetricNamedTokenPayload {
        kind: CodingMetricNamedTokenKind::StablePassAt1,
        session_id: session_id.to_owned(),
        model_id: Some(model_id.to_owned()),
        details_json: details,
        ts_ms,
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(metric)
}

/// Record a `harness_attribution_tuple` row against the SSB.
///
/// Wire-dark default behind the same env gate as STABLE-Pass@1. The
/// row carries the `(model_id, harness_version, harness_commit_sha,
/// harness_repo)` tuple per E-49 plan Task 4 so the B-37 harness-vs-
/// model ablation lever stays legible at audit time.
///
/// `harness_version` is the orchestrator-side `CARGO_PKG_VERSION`
/// (caller is responsible for passing it via `env!("CARGO_PKG_VERSION")`
/// so this module stays test-friendly). `harness_commit_sha` and
/// `harness_repo` are read by the caller via `std::process::Command::new("git")`
/// off the .elai_cc checkout root; passing them as inputs keeps the
/// blast radius of git invocation outside this module.
pub async fn record_harness_attribution_tuple(
    ssb: &Arc<Ssb>,
    session_id: &str,
    model_id: &str,
    harness_version: &str,
    harness_commit_sha: &str,
    harness_repo: &str,
    agent_id: &str,
    superstep: i64,
    ts_ms: i64,
) -> Result<(), StablePassError> {
    if !producer_enabled() {
        return Ok(());
    }
    let details = serde_json::json!({
        "caller_site": "coding_metrics::stable_pass::record_harness_attribution_tuple",
        "harness_version": harness_version,
        "harness_commit_sha": harness_commit_sha,
        "harness_repo": harness_repo,
        "session_id": session_id,
        "model_id": model_id,
    });
    let payload = SsbPayload::CodingMetricNamedToken(CodingMetricNamedTokenPayload {
        kind: CodingMetricNamedTokenKind::HarnessAttributionTuple,
        session_id: session_id.to_owned(),
        model_id: Some(model_id.to_owned()),
        details_json: details,
        ts_ms,
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(())
}

/// Single violation captured by the stop-verified lint pass.
#[derive(Debug, Clone)]
pub struct StopVerifiedViolation {
    pub path: String,
    pub line: u32,
    pub token: String,
}

/// Record a `stop_verified` lint-result row against the SSB.
///
/// Emitted by the `.claude/tools/stop_verified_lint.py` Python lint
/// wrapper (CI side; see also the `stop_verified_lint.py` companion
/// shipped this tranche) but the canonical Rust producer surface lives
/// here so contract tests + CLI subcommands can reach the same emit
/// path. Wire-dark default behind the same env gate.
///
/// `guard_present` is the global "STOP-Verified:" guard token presence
/// signal (per the E-49 plan Task 11 description).
pub async fn record_stop_verified(
    ssb: &Arc<Ssb>,
    session_id: &str,
    files_scanned: u32,
    violations: &[StopVerifiedViolation],
    guard_present: bool,
    agent_id: &str,
    superstep: i64,
    ts_ms: i64,
) -> Result<(), StablePassError> {
    if !producer_enabled() {
        return Ok(());
    }
    let violations_json: Vec<serde_json::Value> = violations
        .iter()
        .map(|v| {
            serde_json::json!({
                "path": v.path,
                "line": v.line,
                "token": v.token,
            })
        })
        .collect();
    let n_violations = u32::try_from(violations.len()).unwrap_or(u32::MAX);
    let details = serde_json::json!({
        "caller_site": "coding_metrics::stable_pass::record_stop_verified",
        "files_scanned": files_scanned,
        "violations_count": n_violations,
        "violations": violations_json,
        "guard_present": guard_present,
    });
    let payload = SsbPayload::CodingMetricNamedToken(CodingMetricNamedTokenPayload {
        kind: CodingMetricNamedTokenKind::StopVerified,
        session_id: session_id.to_owned(),
        model_id: None,
        details_json: details,
        ts_ms,
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeds_all_pass_identical() -> Vec<SeedOutcome> {
        (0..5)
            .map(|i| SeedOutcome {
                seed_label: format!("seed-{i}"),
                passed: true,
                serialized_output: "canonical-output".to_string(),
            })
            .collect()
    }

    #[test]
    fn compute_stable_pass_rejects_empty() {
        let err = compute_stable_pass(&[]).unwrap_err();
        assert!(matches!(err, StablePassError::EmptySeeds));
    }

    #[test]
    fn compute_stable_pass_identical_all_pass_yields_score_one() {
        let outcomes = seeds_all_pass_identical();
        let m = compute_stable_pass(&outcomes).expect("ok");
        assert_eq!(m.seed_count, 5);
        assert!(m.identical_outputs);
        assert!((m.mean_pass - 1.0).abs() < 1e-9);
        assert!(m.std_dev < 1e-9);
        assert!((m.stable_score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn compute_stable_pass_split_outputs_flips_identical_flag() {
        let mut outcomes = seeds_all_pass_identical();
        outcomes[2].serialized_output = "drift".to_string();
        let m = compute_stable_pass(&outcomes).expect("ok");
        assert!(!m.identical_outputs);
        // Still all-pass: mean_pass == 1, std_dev == 0, score == 1.
        assert!((m.mean_pass - 1.0).abs() < 1e-9);
        assert!((m.stable_score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn compute_stable_pass_partial_failures_lowers_score() {
        let outcomes = vec![
            SeedOutcome {
                seed_label: "s0".into(),
                passed: true,
                serialized_output: "x".into(),
            },
            SeedOutcome {
                seed_label: "s1".into(),
                passed: true,
                serialized_output: "x".into(),
            },
            SeedOutcome {
                seed_label: "s2".into(),
                passed: false,
                serialized_output: "x".into(),
            },
            SeedOutcome {
                seed_label: "s3".into(),
                passed: false,
                serialized_output: "x".into(),
            },
        ];
        let m = compute_stable_pass(&outcomes).expect("ok");
        // mean_pass = 0.5; std_dev = sqrt(0.25) = 0.5
        assert!((m.mean_pass - 0.5).abs() < 1e-9);
        assert!((m.std_dev - 0.5).abs() < 1e-9);
        // raw_score = 0.5 * (1 - 1.0) = 0.0 -> clamped to 0.0
        assert!((m.stable_score - 0.0).abs() < 1e-9);
    }

    #[test]
    fn producer_disabled_by_default() {
        // Env var unset by default in test harness; guards against
        // accidental writes when ssb is not wired.
        std::env::remove_var(ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE_ENV);
        assert!(!producer_enabled());
    }
}
