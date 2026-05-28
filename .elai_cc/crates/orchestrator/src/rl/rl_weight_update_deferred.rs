//! E-55 -- Task 8: weight-update fine-tune is HARD-DEFERRED until 3 unlock
//! conditions all TRUE.
//!
//! The 3 escalation triggers (per source report condition list + plan):
//!   1. BAI provably saturated: `BAI_SATURATED_STABLE_SESSIONS` (default 3)
//!      sessions where BAI converged to a stable arm AND tripwire stack
//!      stopped firing BUT performance still misses target.
//!   2. Dense verifiable reward available: ORM signal from reward_fusion
//!      covers >= `ORM_COVERAGE_THRESHOLD` (default 80%) of trajectories.
//!      Goodhart guarantee without this.
//!   3. Local-model VRAM headroom: per memory:project_qwen3b_baseline.md,
//!      current 12GB ceiling leaves <2GB headroom; SFT-LoRA needs 4-6GB
//!      additional. Trigger requires either 2x3090 upgrade (per
//!      memory:project_vram_tier_plan) OR 24GB single-GPU rent.
//!
//! When all three are TRUE, emit `RewardLadderEventKind::RlWeightUpdateUnlocked`
//! + open a structured ticket. UNTIL ALL THREE TRUE: DPO / PPO / GRPO /
//! DAPO / KTO weight-update modules remain BLOCKED (contract test
//! `crates/contracts/tests/no_dpo_no_grpo_until_unlock.rs` enforces).
//!
//! Per CLAUDE.md stub registry: this module is the gated SURFACE. The
//! module BODY is fully implemented -- the STUB is the absence of the
//! dpo/ppo/grpo/dapo/kto modules. See E-55.B7 entry in
//! workflow/sessions/<sid>/stubs.md (registered lazily at first
//! `check_unlock` invocation per plan Risk 7).
//!
//! Pure, deterministic. No I/O. No clock. Caller injects the three
//! signals (bai stability counter / orm coverage fraction / vram
//! available GB) and reads the verdict.

use crate::bai::ArmPosterior;
use serde::{Deserialize, Serialize};

/// Number of consecutive sessions of stable BAI convergence required to
/// satisfy condition 1.
pub const BAI_SATURATED_STABLE_SESSIONS: u32 = 3;

/// Min fraction of trajectories that must carry a real ORM signal
/// (not just a learned PRM heuristic) to satisfy condition 2.
pub const ORM_COVERAGE_THRESHOLD: f32 = 0.80;

/// Min free VRAM headroom (GB) above current model footprint required for
/// SFT-LoRA. Per memory:project_qwen3b_baseline.md the local 3B NF4
/// baseline is ~1.92GB; SFT-LoRA needs +4-6GB. The 4GB constant is the
/// conservative lower bound.
pub const VRAM_HEADROOM_MIN_GB: f32 = 4.0;

/// Stability proxy: an arm is "stable" when its posterior is tight
/// (alpha + beta > this many pulls) AND its mean is above 0.6 (caller
/// can override via `UnlockInputs::bai_stable_arm_count`).
pub const BAI_ARM_STABILITY_PULL_MIN: u32 = 20;

/// Bundle of the 3 input signals + the all-three-true verdict.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UnlockStatus {
    pub bai_saturated: bool,
    pub orm_coverage: f32,
    pub vram_headroom_gb: f32,
    pub all_three_true: bool,
}

impl UnlockStatus {
    /// Build the verdict from the 3 raw signals. Pure -- no I/O.
    #[must_use]
    pub fn build(
        bai_saturated: bool,
        orm_coverage: f32,
        vram_headroom_gb: f32,
    ) -> Self {
        let orm_ok = orm_coverage >= ORM_COVERAGE_THRESHOLD;
        let vram_ok = vram_headroom_gb >= VRAM_HEADROOM_MIN_GB;
        let all_three_true = bai_saturated && orm_ok && vram_ok;
        Self {
            bai_saturated,
            orm_coverage,
            vram_headroom_gb,
            all_three_true,
        }
    }

    /// Per-condition pass flags. For dashboard rendering + auditor
    /// reporting.
    #[must_use]
    pub fn condition_flags(&self) -> [bool; 3] {
        [
            self.bai_saturated,
            self.orm_coverage >= ORM_COVERAGE_THRESHOLD,
            self.vram_headroom_gb >= VRAM_HEADROOM_MIN_GB,
        ]
    }
}

/// Inputs to `check_unlock`. The orchestrator wires these from live state:
///   - `arm_history` -> from D-5 BAI posterior list
///   - `orm_signal_coverage` -> running fraction from reward_fusion emit
///   - `available_vram_gb` -> from sidecar host probe
///   - `bai_stable_arm_count` -> caller-tracked count across sessions
#[derive(Debug, Clone)]
pub struct UnlockInputs<'a> {
    pub arm_history: &'a [ArmPosterior],
    pub orm_signal_coverage: f32,
    pub available_vram_gb: f32,
    pub bai_stable_arm_count: u32,
}

/// Closed-form unlock check. Pure -- no I/O. Caller emits the SSB row
/// when `all_three_true` and `previously_unlocked == false` (the latter
/// is owned by the caller's session-runtime).
#[must_use]
pub fn check_unlock(inputs: UnlockInputs<'_>) -> UnlockStatus {
    // Condition 1: BAI saturation. The caller-tracked
    // `bai_stable_arm_count` is the primary signal; the arm_history is
    // sanity-checked to confirm at least one arm has hit the pull-min.
    let any_arm_stable = inputs.arm_history.iter().any(|a| {
        a.n_pulls >= BAI_ARM_STABILITY_PULL_MIN && a.mean() >= 0.6
    });
    let bai_saturated =
        inputs.bai_stable_arm_count >= BAI_SATURATED_STABLE_SESSIONS && any_arm_stable;

    UnlockStatus::build(
        bai_saturated,
        inputs.orm_signal_coverage,
        inputs.available_vram_gb,
    )
}

/// Eviction marker: when the unlock fires, this is the structured ticket
/// payload the caller writes into the orchestrator's audit log + Tauri
/// command response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnlockTicket {
    pub status: UnlockStatus,
    pub ticket_id: String,
    pub note: String,
}

impl UnlockTicket {
    /// Build a ticket from a `Pass`-verdict status. Caller provides the
    /// `session_id` so the ticket is traceable.
    ///
    /// Returns `None` when the status is not `all_three_true` (caller
    /// must not create a ticket prematurely).
    #[must_use]
    pub fn from_status(status: UnlockStatus, session_id: &str) -> Option<Self> {
        if !status.all_three_true {
            return None;
        }
        // Deterministic ticket id: blake3(session_id) prefix.
        let h = blake3::hash(session_id.as_bytes());
        let prefix = h.to_hex();
        Some(Self {
            status,
            ticket_id: format!("rl-unlock-{}", &prefix.as_str()[..12]),
            note: format!(
                "Weight-update RL unlocked: bai_saturated={} orm_coverage={:.3} vram_headroom_gb={:.2}",
                status.bai_saturated, status.orm_coverage, status.vram_headroom_gb
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// E-55.B7 stub registration marker
//
// Per CLAUDE.md stub registry: the gated DPO/PPO/GRPO/DAPO/KTO weight-
// update modules do NOT exist by design. The marker below is the
// structured TODO that workflow/sessions/<sid>/stubs.md registers (lazy
// registration -- emitted by the orchestrator's session-runtime at
// first `check_unlock` invocation).
//
// TODO[E-55.B7]: when UnlockTicket fires, open the SFT/RL weight-update
// fine-tune patch (planned id: E-55.1). Until then, the contract test
// `no_dpo_no_grpo_until_unlock.rs` HARD-BLOCKS any dpo/ppo/grpo/dapo/kto
// module from being added to crates/orchestrator/src/ or crates/training/.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reasoning_template::TemplateId;
    use uuid::Uuid;

    fn make_tid(seed: u8) -> TemplateId {
        TemplateId::from(Uuid::from_bytes([seed; 16]))
    }

    fn stable_arm() -> ArmPosterior {
        let mut a = ArmPosterior::new(make_tid(1));
        for _ in 0..30 {
            a.update(true);
        }
        a
    }

    #[test]
    fn three_constants_match_plan() {
        assert_eq!(BAI_SATURATED_STABLE_SESSIONS, 3);
        assert!((ORM_COVERAGE_THRESHOLD - 0.80).abs() < f32::EPSILON);
        assert!((VRAM_HEADROOM_MIN_GB - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn unlock_status_carries_three_conditions() {
        let s = UnlockStatus::build(true, 0.9, 5.0);
        assert!(s.all_three_true);
        let flags = s.condition_flags();
        assert_eq!(flags, [true, true, true]);
    }

    #[test]
    fn one_of_three_false_blocks_unlock() {
        // BAI false
        let s = UnlockStatus::build(false, 0.9, 5.0);
        assert!(!s.all_three_true);
        // ORM below threshold
        let s = UnlockStatus::build(true, 0.5, 5.0);
        assert!(!s.all_three_true);
        // VRAM below threshold
        let s = UnlockStatus::build(true, 0.9, 2.0);
        assert!(!s.all_three_true);
    }

    #[test]
    fn check_unlock_requires_stable_arm_and_stable_sessions() {
        let arm = stable_arm();
        let arms = vec![arm];
        let inputs = UnlockInputs {
            arm_history: &arms,
            orm_signal_coverage: 0.9,
            available_vram_gb: 5.0,
            bai_stable_arm_count: 3,
        };
        let s = check_unlock(inputs);
        assert!(s.all_three_true);
    }

    #[test]
    fn check_unlock_blocks_when_stable_session_count_short() {
        let arm = stable_arm();
        let arms = vec![arm];
        let inputs = UnlockInputs {
            arm_history: &arms,
            orm_signal_coverage: 0.9,
            available_vram_gb: 5.0,
            bai_stable_arm_count: 2, // < BAI_SATURATED_STABLE_SESSIONS
        };
        let s = check_unlock(inputs);
        assert!(!s.bai_saturated);
        assert!(!s.all_three_true);
    }

    #[test]
    fn check_unlock_blocks_when_no_arm_at_pull_min() {
        let arm = ArmPosterior::new(make_tid(1)); // 0 pulls
        let arms = vec![arm];
        let inputs = UnlockInputs {
            arm_history: &arms,
            orm_signal_coverage: 0.9,
            available_vram_gb: 5.0,
            bai_stable_arm_count: 3,
        };
        let s = check_unlock(inputs);
        assert!(!s.bai_saturated);
    }

    #[test]
    fn unlock_ticket_from_status_only_when_all_three_true() {
        let s_blocked = UnlockStatus::build(true, 0.5, 5.0);
        assert!(UnlockTicket::from_status(s_blocked, "s-1").is_none());
        let s_unlocked = UnlockStatus::build(true, 0.9, 5.0);
        let ticket = UnlockTicket::from_status(s_unlocked, "s-1").expect("unlocked");
        assert!(ticket.ticket_id.starts_with("rl-unlock-"));
    }
}
