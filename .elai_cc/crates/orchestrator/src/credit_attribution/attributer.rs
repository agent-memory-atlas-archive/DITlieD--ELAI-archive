//! C-45 — Core contrastive-attribute driver.
//!
//! Implements the inner loop of CANTANTE (Zehle 2026,
//! arXiv:2605.13295, Section 3.2): given a baseline task and a list of
//! roles to drop in turn, run `K` rollouts per configuration and
//! return per-role scalar credit scores `c in [-1, +1]`.
//!
//! Key paper-grounded invariants enforced here:
//!   - `K` rollouts per configuration, NOT `K^N`. The driver iterates
//!     over `dropped_roles` and runs each role through `K` rollouts in
//!     sequence; the total rollout budget is `K * len(dropped_roles) + K`
//!     (baseline). The cross-product blowup that a naive implementation
//!     would produce is explicitly avoided.
//!   - Scalar credit, NOT a rank. Per-role credit is the contrastive
//!     delta `baseline_mean - dropped_mean`, clamped to `[-1, +1]` via
//!     [`crate::credit_attribution::CreditScore`].
//!   - Uniform mean over the query set (here: rollouts). Per-paper
//!     Section 3.2 last paragraph: aggregate AFTER per-query
//!     attribution, never before.
//!
//! The driver is generic over an async fallible closure that takes a
//! [`DropPolicy`] and returns a single rollout score in `[0.0, 1.0]`
//! (task-success rate proxy). The closure signature is intentionally
//! minimal so the harness can wire it to either a synthetic-fixture
//! callback (unit-test path) or the live cloud-billable rollout path
//! (`runpod-pending` flag).

use std::future::Future;

use elai_ipc_types::AgentRole;
use thiserror::Error;

use super::types::{CreditReport, CreditScore, DropPolicy};

/// Errors raised by [`contrastive_attribute`].
#[derive(Debug, Error)]
pub enum AttributeError {
    /// `K` was zero — no rollouts to attribute.
    #[error("K=0 leaves nothing to attribute; require K >= 1")]
    EmptyRollouts,
    /// The underlying rollout closure failed for one of the drop
    /// policies. Carries the offending policy + the formatted error.
    /// We use a String for the error message (renamed away from
    /// `source` so thiserror does not treat it as a wrapped
    /// `std::error::Error`) so the driver does not require the closure
    /// error type to implement `Send + Sync + 'static + Error`; the
    /// harness boxes it up.
    #[error("rollout closure failed for policy {policy:?}: {message}")]
    RolloutFailed { policy: DropPolicy, message: String },
}

/// Run the contrastive credit-attribution driver. Returns a
/// [`CreditReport`] keyed by `AgentRole` containing per-role scalar
/// credits.
///
/// Algorithm (mirrors CANTANTE Algorithm 1):
///   1. Run `K` baseline rollouts (no role dropped). Average their
///      scores into `baseline_mean in [0.0, 1.0]`.
///   2. For each `role in dropped_roles`:
///      a. Run `K` rollouts under [`DropPolicy::drop(role)`]. Average
///         their scores into `dropped_mean in [0.0, 1.0]`.
///      b. Credit for `role` = `baseline_mean - dropped_mean`. A
///         positive value means "dropping this role hurt" (the role
///         contributed positively). A negative value means the role
///         was net noise.
///   3. Clamp each per-role credit to `[-1.0, +1.0]` via
///      `CreditScore::clamped` (the difference of two `[0, 1]`
///      averages is naturally in `[-1, 1]` but we round-trip through
///      the clamp for non-finite safety).
///
/// `run_with_policy` is the rollout callback: given a `DropPolicy` and
/// the rollout index `i in 0..K`, it must return a scalar in
/// `[0.0, 1.0]` representing task success (1.0 = pass, 0.0 = fail,
/// fractional values allowed for partial-credit scoring functions).
/// Returning a value outside `[0.0, 1.0]` is undefined behaviour at
/// the math layer — the harness clamps but does NOT validate the
/// input range (the synthetic fixture and the live rollout adapter
/// are both already in-range by construction).
///
/// `K` is the per-configuration rollout count. CANTANTE main experiments
/// use `K = 9` (paper Section 4 + Appendix B.2). The driver does not
/// validate the `K % g == 0` divisibility here because the partition
/// is a separate concern (see [`super::partition::validate_partition`])
/// — the harness owns the partition + driver wiring.
///
/// `ts_ms` is the producer-side wall-clock timestamp for the resulting
/// `CreditReport`. Caller resolves it via the orchestrator's injected
/// `ClockProvider` (never `std::time::SystemTime` per the entropy
/// quarantine rule).
pub async fn contrastive_attribute<F, Fut>(
    dropped_roles: &[AgentRole],
    k: u32,
    ts_ms: u64,
    mut run_with_policy: F,
) -> Result<CreditReport, AttributeError>
where
    F: FnMut(DropPolicy, u32) -> Fut,
    Fut: Future<Output = Result<f32, String>>,
{
    if k == 0 {
        return Err(AttributeError::EmptyRollouts);
    }

    // Step 1: baseline rollouts.
    let baseline_policy = DropPolicy::baseline();
    let baseline_mean = run_k_and_mean(&mut run_with_policy, baseline_policy, k).await?;

    // Step 2: per-role dropped rollouts.
    let mut report = CreditReport::new(ts_ms);
    for role in dropped_roles.iter().copied() {
        let dropped_policy = DropPolicy::drop(role);
        let dropped_mean = run_k_and_mean(&mut run_with_policy, dropped_policy, k).await?;
        let delta = baseline_mean - dropped_mean;
        report.record(role, CreditScore::clamped(delta));
    }

    Ok(report)
}

/// Run `K` rollouts under one policy and return the mean. Helper
/// shared by baseline + per-role legs of [`contrastive_attribute`].
async fn run_k_and_mean<F, Fut>(
    run_with_policy: &mut F,
    policy: DropPolicy,
    k: u32,
) -> Result<f32, AttributeError>
where
    F: FnMut(DropPolicy, u32) -> Fut,
    Fut: Future<Output = Result<f32, String>>,
{
    let mut sum: f32 = 0.0;
    for i in 0..k {
        let score = run_with_policy(policy, i).await.map_err(|message| {
            AttributeError::RolloutFailed { policy, message }
        })?;
        sum += score;
    }
    // Safe: k >= 1 enforced at the entry point.
    Ok(sum / (k as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Per-CANTANTE Section 3.2: K rollouts per configuration,
    // ---- NOT K^N. The counting tests below enforce this.

    #[tokio::test]
    async fn empty_k_errors() {
        let res = contrastive_attribute(&[AgentRole::Executor], 0, 0, |_, _| async {
            Ok::<f32, String>(1.0)
        })
        .await;
        assert!(matches!(res, Err(AttributeError::EmptyRollouts)));
    }

    #[tokio::test]
    async fn rollout_count_is_k_times_n_plus_baseline() {
        let dropped = vec![
            AgentRole::Executor,
            AgentRole::Auditor,
            AgentRole::Researcher,
        ];
        let k = 9_u32;
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0_u32));
        let counter_clone = counter.clone();
        let _report = contrastive_attribute(&dropped, k, 0, move |_policy, _i| {
            let counter = counter_clone.clone();
            async move {
                *counter.lock().expect("lock") += 1;
                Ok::<f32, String>(0.5)
            }
        })
        .await
        .expect("attribute succeeds");

        let total = *counter.lock().expect("lock");
        // K baseline rollouts + K * N dropped rollouts. NOT K^N.
        let expected = k + k * (dropped.len() as u32);
        assert_eq!(
            total, expected,
            "CANTANTE invariant: K rollouts per config, NOT K^N (paper Section 3.2)"
        );
        // Sanity: K^N for K=9, N=3 would be 729; we must be far below.
        assert!(total < 9_u32.pow(3), "must not fan out K^N");
    }

    #[tokio::test]
    async fn baseline_minus_dropped_gives_positive_credit_for_useful_role() {
        // Construct a scoring closure where the baseline scores 1.0 and
        // dropping Executor scores 0.0. Executor's credit must be +1.0.
        let report = contrastive_attribute(&[AgentRole::Executor], 4, 0, |policy, _i| async move {
            if policy.is_baseline() {
                Ok::<f32, String>(1.0)
            } else {
                Ok::<f32, String>(0.0)
            }
        })
        .await
        .expect("attribute succeeds");

        let credit = report.get(AgentRole::Executor).expect("Executor recorded");
        assert!((credit - 1.0).abs() < 1e-6, "useful role credit = +1.0");
    }

    #[tokio::test]
    async fn dropping_noise_role_yields_negative_credit() {
        // Construct a scoring closure where the baseline scores 0.0 and
        // dropping Designer scores 1.0 (the role was net noise).
        // Designer's credit must be -1.0.
        let report = contrastive_attribute(&[AgentRole::Designer], 4, 0, |policy, _i| async move {
            if policy.is_baseline() {
                Ok::<f32, String>(0.0)
            } else {
                Ok::<f32, String>(1.0)
            }
        })
        .await
        .expect("attribute succeeds");

        let credit = report.get(AgentRole::Designer).expect("Designer recorded");
        assert!((credit - -1.0).abs() < 1e-6, "noise role credit = -1.0");
    }

    #[tokio::test]
    async fn rollout_error_propagates() {
        let res = contrastive_attribute(&[AgentRole::Executor], 2, 0, |_policy, _i| async {
            Err::<f32, String>("synthetic failure".into())
        })
        .await;
        match res {
            Err(AttributeError::RolloutFailed { message, .. }) => {
                assert!(message.contains("synthetic failure"));
            }
            other => panic!("expected RolloutFailed, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn report_timestamp_threaded_through() {
        let ts = 1_700_000_000_000_u64;
        let report = contrastive_attribute(&[AgentRole::Executor], 1, ts, |_, _| async {
            Ok::<f32, String>(0.5)
        })
        .await
        .expect("attribute succeeds");
        assert_eq!(report.ts_ms, ts);
    }
}
