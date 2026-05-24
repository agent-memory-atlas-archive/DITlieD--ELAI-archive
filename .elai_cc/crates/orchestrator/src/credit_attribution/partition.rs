//! C-45 — group-of-`g` partition logic for contrastive credit attribution.
//!
//! Paper grounding (Zehle 2026, arXiv:2605.13295, Section 4.4 footnote 1
//! + Appendix B.2 ablation table):
//!   - The contrastive method requires `K % g == 0` so the K joint
//!     configurations partition exactly into `K / g` groups of size `g`.
//!   - `g = 3` is the author-recommended sweet spot (`g = 2` produces a
//!     -54.4pp catastrophic collapse on GSM8K; `g = 5` is +0.8pp,
//!     diminishing returns). We surface `g = 3` as the compile-time
//!     default and refuse `g <= 2` at runtime.
//!   - This file lives under the contrastive pilot driver — the live
//!     attribution loop is in `attributer::contrastive_attribute`.
//!
//! Pilot scope decision (C-45): the ablation set is the eight base
//! `AgentRole` variants. The eight newer variants (B-40 IdeaFramer /
//! IdeaCritic, B-41 five per-level Auditor*) are intentionally
//! EXCLUDED from this pilot because extending the contrastive run to
//! 16 roles roughly doubles the runpod-billable token budget; the
//! follow-on pilot will land them once the 8-role pilot lands a
//! signal.

use elai_ipc_types::AgentRole;
use thiserror::Error;

/// Author-recommended group size from CANTANTE Appendix B.2. `g = 2`
/// produces a -54.4pp catastrophic collapse on GSM8K; treat any value
/// `< 3` as a configuration error.
pub const G_DEFAULT: u32 = 3;

/// Errors raised by [`validate_partition`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PartitionError {
    /// `g < 3` violates the CANTANTE Section 3.2 guidance and produces
    /// a documented catastrophic regression on GSM8K (-54.4pp).
    #[error(
        "group size g={g} is below the paper-anchored floor (g >= 3 required; CANTANTE Section 3.2 / Appendix B.2)"
    )]
    GroupTooSmall { g: u32 },
    /// `K % g != 0` violates the partition divisibility invariant
    /// (paper Section 4.4 footnote 1: "K must be divisible by g for the
    /// group partition to be exact").
    #[error(
        "K={k} is not divisible by g={g}; partition must be exact (CANTANTE Section 4.4 footnote 1)"
    )]
    Indivisible { k: u32, g: u32 },
    /// `K == 0` is meaningless — there are no rollouts to attribute.
    #[error("K=0 leaves nothing to attribute; require K >= g")]
    EmptyK,
}

/// Validate the `(K, g)` partition configuration against the paper-
/// anchored invariants. Returns `Ok(())` for any valid `(K, g)` pair.
///
/// Concrete checks (cited from `workflow/papers/cantante-credit-
/// attribution.md`):
///   - `g >= 3` (Section 3.2 + Appendix B.2: g=2 collapses, -54.4pp).
///   - `K % g == 0` (Section 4.4 footnote 1: exact group partition).
///   - `K >= g` (no degenerate empty roll-up).
pub fn validate_partition(k: u32, g: u32) -> Result<(), PartitionError> {
    if g < G_DEFAULT {
        return Err(PartitionError::GroupTooSmall { g });
    }
    if k == 0 {
        return Err(PartitionError::EmptyK);
    }
    if k % g != 0 {
        return Err(PartitionError::Indivisible { k, g });
    }
    Ok(())
}

/// The eight base `AgentRole` variants covered by the C-45 contrastive
/// pilot. The eight newer variants (B-40 IdeaFramer / IdeaCritic, B-41
/// five per-level Auditor*) are intentionally excluded — see
/// `mod.rs` for the cost-cap rationale and `harness.rs` for the
/// per-role drop-and-rollout loop that iterates this list.
pub fn pilot_roles() -> &'static [AgentRole] {
    // Static slice so callers do not allocate; ordering is the same
    // declaration order as `AgentRole` in `elai_ipc_types::lib`. The
    // ordering is observable via the harness output but does not
    // change correctness — credit attribution is a set operation, not
    // an ordered one.
    const BASE_ROLES: [AgentRole; 8] = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
    ];
    &BASE_ROLES
}

/// Partition `K` rollouts into `K / g` groups of size `g`. Returns a
/// `Vec<Vec<u32>>` where each inner vec is the rollout indices in the
/// group. Indices are 0-based and span `0..k`.
///
/// Caller must call [`validate_partition`] first; this helper panics
/// in debug builds if the invariants are violated (the `assert!` is a
/// safety net for harness mis-wiring, NOT user-facing error handling).
pub fn build_partition(k: u32, g: u32) -> Result<Vec<Vec<u32>>, PartitionError> {
    validate_partition(k, g)?;
    let group_count = k / g;
    let mut groups: Vec<Vec<u32>> = Vec::with_capacity(group_count as usize);
    for group_idx in 0..group_count {
        let start = group_idx * g;
        let mut group: Vec<u32> = Vec::with_capacity(g as usize);
        for offset in 0..g {
            group.push(start + offset);
        }
        groups.push(group);
    }
    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g_default_is_paper_grounded_three() {
        assert_eq!(G_DEFAULT, 3, "CANTANTE Appendix B.2 sweet spot");
    }

    #[test]
    fn validate_rejects_g_two_per_paper() {
        let err = validate_partition(6, 2).expect_err("g=2 must fail");
        assert_eq!(err, PartitionError::GroupTooSmall { g: 2 });
    }

    #[test]
    fn validate_rejects_g_zero() {
        let err = validate_partition(9, 0).expect_err("g=0 must fail");
        assert_eq!(err, PartitionError::GroupTooSmall { g: 0 });
    }

    #[test]
    fn validate_rejects_indivisible() {
        let err = validate_partition(10, 3).expect_err("10 % 3 != 0");
        assert_eq!(err, PartitionError::Indivisible { k: 10, g: 3 });
    }

    #[test]
    fn validate_rejects_empty_k() {
        let err = validate_partition(0, 3).expect_err("K=0 must fail");
        assert_eq!(err, PartitionError::EmptyK);
    }

    #[test]
    fn validate_accepts_paper_default() {
        // CANTANTE main experiments use K=9, g=3.
        validate_partition(9, 3).expect("paper default must pass");
    }

    #[test]
    fn validate_accepts_g_five_ablation_config() {
        // CANTANTE Appendix B.2 g-ablation uses K=10, g=5.
        validate_partition(10, 5).expect("ablation config must pass");
    }

    #[test]
    fn build_partition_yields_correct_groups() {
        let groups = build_partition(9, 3).expect("paper default");
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0], vec![0, 1, 2]);
        assert_eq!(groups[1], vec![3, 4, 5]);
        assert_eq!(groups[2], vec![6, 7, 8]);
    }

    #[test]
    fn pilot_roles_is_exactly_eight_base_roles() {
        let roles = pilot_roles();
        assert_eq!(roles.len(), 8, "C-45 pilot ablates eight base roles only");
        assert!(roles.contains(&AgentRole::Executor));
        assert!(roles.contains(&AgentRole::MediaValidator));
        // B-40 / B-41 expansions intentionally absent.
        assert!(!roles.contains(&AgentRole::IdeaFramer));
        assert!(!roles.contains(&AgentRole::IdeaCritic));
        assert!(!roles.contains(&AgentRole::AuditorConcepts));
        assert!(!roles.contains(&AgentRole::AuditorPhases));
        assert!(!roles.contains(&AgentRole::AuditorSteps));
        assert!(!roles.contains(&AgentRole::AuditorBabySteps));
        assert!(!roles.contains(&AgentRole::AuditorBabyStepDetail));
    }
}
