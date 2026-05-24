//! C-45 — Top-level harness driving a full contrastive credit pilot.
//!
//! Wraps [`super::attributer::contrastive_attribute`] with the partition
//! validation, the canonical "eight base roles" pilot set, and a single
//! entry point the CLI subcommand + the integration test call into.
//!
//! Boundary: the harness is rollout-source-agnostic — it does NOT
//! decide whether to invoke the live remote sidecar or a synthetic
//! fixture. The caller passes a closure (the same closure shape as
//! [`super::attributer::contrastive_attribute`]'s `run_with_policy`).
//! The CLI driver wires this to the runpod-pending live rollout; the
//! unit tests wire it to a deterministic synthetic scorer.

use std::future::Future;

use elai_ipc_types::AgentRole;
use thiserror::Error;

use super::attributer::{contrastive_attribute, AttributeError};
use super::partition::{pilot_roles, validate_partition, PartitionError};
use super::types::CreditReport;

/// Errors raised by [`CreditAttributionHarness::run`].
#[derive(Debug, Error)]
pub enum HarnessError {
    /// Partition invariants (`g >= 3`, `K % g == 0`, `K >= g`) were
    /// violated. The harness refuses to run because a bad partition
    /// produces statistically unreliable credits per CANTANTE Section
    /// 4.4 footnote 1.
    #[error("partition validation failed: {0}")]
    Partition(#[from] PartitionError),
    /// The inner driver failed (empty K, or a rollout error). Source
    /// preserves the offending policy when applicable.
    #[error("contrastive attribute failed: {0}")]
    Attribute(#[from] AttributeError),
}

/// Top-level harness for a single contrastive credit-attribution pilot.
///
/// One `CreditAttributionHarness` instance corresponds to one pilot
/// run: the configured `(k, g)` partition + the closure that scores a
/// rollout under a given drop policy. The harness is one-shot — invoke
/// `run()` once and consume the resulting `CreditReport`.
pub struct CreditAttributionHarness {
    /// Per-configuration rollout count. CANTANTE main exps use K=9.
    pub k: u32,
    /// Group size (paper-anchored default = 3). Refused if `< 3`.
    pub g: u32,
    /// Roles to ablate. Defaults to [`pilot_roles()`] (the eight base
    /// roles) when constructed via [`Self::pilot`].
    pub dropped_roles: Vec<AgentRole>,
    /// Producer-side wall-clock timestamp (milliseconds since UNIX
    /// epoch) the resulting `CreditReport` carries. Caller resolves
    /// it via the orchestrator's injected `ClockProvider`.
    pub ts_ms: u64,
}

impl CreditAttributionHarness {
    /// Pilot constructor — uses the canonical eight-role ablation set
    /// (`pilot_roles()`) + paper-anchored partition `(K=9, g=3)`.
    #[must_use]
    pub fn pilot(ts_ms: u64) -> Self {
        Self {
            k: 9,
            g: 3,
            dropped_roles: pilot_roles().to_vec(),
            ts_ms,
        }
    }

    /// Explicit constructor — caller supplies the partition + role set.
    #[must_use]
    pub fn new(k: u32, g: u32, dropped_roles: Vec<AgentRole>, ts_ms: u64) -> Self {
        Self {
            k,
            g,
            dropped_roles,
            ts_ms,
        }
    }

    /// Run the harness. Validates the partition, then drives
    /// [`contrastive_attribute`] over the configured role set.
    ///
    /// `run_with_policy` is the rollout-source closure (see
    /// [`super::attributer::contrastive_attribute`]).
    pub async fn run<F, Fut>(self, run_with_policy: F) -> Result<CreditReport, HarnessError>
    where
        F: FnMut(super::types::DropPolicy, u32) -> Fut,
        Fut: Future<Output = Result<f32, String>>,
    {
        validate_partition(self.k, self.g)?;
        let report = contrastive_attribute(
            &self.dropped_roles,
            self.k,
            self.ts_ms,
            run_with_policy,
        )
        .await?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pilot_harness_covers_eight_base_roles() {
        let harness = CreditAttributionHarness::pilot(1_700_000_000_000);
        assert_eq!(harness.dropped_roles.len(), 8);
        let report = harness
            .run(|_policy, _i| async { Ok::<f32, String>(0.5) })
            .await
            .expect("harness must run");
        assert_eq!(report.len(), 8);
        assert!(report.get(AgentRole::Executor).is_some());
        assert!(report.get(AgentRole::MediaValidator).is_some());
    }

    #[tokio::test]
    async fn harness_refuses_g_two() {
        let harness =
            CreditAttributionHarness::new(4, 2, vec![AgentRole::Executor], 0);
        let res = harness
            .run(|_, _| async { Ok::<f32, String>(0.5) })
            .await;
        match res {
            Err(HarnessError::Partition(PartitionError::GroupTooSmall { g })) => {
                assert_eq!(g, 2, "paper-anchored guard");
            }
            other => panic!("expected GroupTooSmall, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn harness_refuses_indivisible_partition() {
        let harness =
            CreditAttributionHarness::new(10, 3, vec![AgentRole::Executor], 0);
        let res = harness
            .run(|_, _| async { Ok::<f32, String>(0.5) })
            .await;
        assert!(matches!(
            res,
            Err(HarnessError::Partition(PartitionError::Indivisible { .. }))
        ));
    }

    #[tokio::test]
    async fn harness_threads_through_constant_baseline_yields_zero_credit() {
        // Baseline == dropped for all roles -> all credits are 0.
        let harness = CreditAttributionHarness::pilot(0);
        let report = harness
            .run(|_policy, _i| async { Ok::<f32, String>(0.42) })
            .await
            .expect("harness ok");
        for role in pilot_roles() {
            let c = report.get(*role).expect("role present");
            assert!(c.abs() < 1e-6, "constant rollout = zero credit");
        }
    }
}
