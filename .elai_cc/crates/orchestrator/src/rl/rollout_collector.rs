//! E-55 -- Task 5: SFT-on-rollouts collection surface (data-collection only).
//!
//! Collects GREEN-session trajectories (audit verdict = PASS + matched-pair
//! sycophancy clean + no reward-hacking flag from MONA) into a JSONL
//! file under `.elai_cc/training/rollouts/{date}/{session_id}.jsonl`.
//! Updates `_index.jsonl` with one line per rollout.
//!
//! Threshold: SFT fine-tune is NOT TRIGGERED until >= MIN_ROLLOUTS_FOR_SFT
//! (default 200) GREEN rollouts collected. Per SWE-Gym ICML 2025
//! (arXiv:2412.21139) the SHIPPED corpus was 491 traces with +12.3pp /
//! +13.6pp on SWE-Bench Lite / Verified -- the 200 threshold is the
//! conservative lower bound where the SFT-on-rollouts experiment can
//! credibly begin.
//!
//! At threshold, emit `RewardLadderEventKind::SftTrainingThresholdReached`
//! + open a tracking ticket. ACTUAL SFT-LoRA fine-tune is a SEPARATE patch
//! (E-55.1) -- this module ships only the collection + threshold trigger.
//!
//! RUNPOD-PENDING: live training run. The substrate ships fully -- only
//! the LoRA training command is gated behind the threshold emit + user
//! confirmation per CLAUDE.md "No new dependencies without confirmation"
//! (PEFT / accelerate / HF transformers not yet in pyproject.toml).
//!
//! blake3 only for content-hash dedup -- CLAUDE.md HARD RULE anti-stale-
//! training: never blake2 / sha2 for non-RFC uses.

use crate::rl::mona_composite_reward::CompositeReward;
use crate::rl::reward_fusion::FusedReward;
use crate::rl::rlaif_judge::RlaifRating;
use crate::verification_result::AggregateVerdict;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// SFT fine-tune threshold -- min GREEN rollouts before emitting the
/// SFT-training-threshold-reached signal. Per SWE-Gym lower-bound framing.
pub const MIN_ROLLOUTS_FOR_SFT: u64 = 200;

/// Filename of the per-session rollout index. Appended-to, never rewritten.
pub const ROLLOUT_INDEX_FILENAME: &str = "_index.jsonl";

/// Audit verdict the collector recognizes. ONLY `Pass` is accepted into
/// the rollout pool per plan: "Collects GREEN-session trajectories".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditVerdict {
    /// All gates green.
    Pass,
    /// Any BLOCK / FAIL -- skip rollout.
    Block,
}

impl AuditVerdict {
    /// Bridge from the verification_result aggregate to AuditVerdict. Pure
    /// closed-form mapping.
    #[must_use]
    pub fn from_aggregate(v: &AggregateVerdict) -> Self {
        if v.is_pass() {
            Self::Pass
        } else {
            Self::Block
        }
    }

    #[must_use]
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }
}

/// One GREEN-session trajectory, serialized into the JSONL row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GreenSessionTrajectory {
    pub session_id: String,
    pub trajectory_id: String,
    pub prompt: String,
    /// Full reasoning trace as a single string (caller flattens any
    /// multi-step structure before passing it in; the collector stays
    /// schema-agnostic over reasoning trace shape).
    pub full_trajectory: String,
    pub ppm_scores: Vec<f32>,
    pub fused_reward: FusedReward,
    pub composite_reward: CompositeReward,
    pub rlaif_rating: Option<RlaifRating>,
    pub final_diff: String,
    pub audit_verdict: AuditVerdict,
    /// Wall-clock millis at collection time; injected by caller via the
    /// orchestrator's ClockProvider so unit tests stay deterministic.
    pub collected_at_ms: i64,
}

impl GreenSessionTrajectory {
    /// Caller-side eligibility check: must be PASS, no reward-hacking
    /// flag from MONA, and (when an RLAIF rating exists) sycophancy_absence
    /// must be above the matched-pair threshold of 0.5.
    #[must_use]
    pub fn is_eligible_for_collection(&self) -> bool {
        if !self.audit_verdict.is_pass() {
            return false;
        }
        if self.composite_reward.hacking_verdict.suspected {
            return false;
        }
        if let Some(r) = &self.rlaif_rating {
            if r.sycophancy_absence < 0.5 {
                return false;
            }
        }
        true
    }
}

/// Per-row content hash (blake3 prefix) used for dedup across reruns.
#[must_use]
pub fn content_hash(traj: &GreenSessionTrajectory) -> String {
    // Build a stable hashing surface across the meaningful fields. Skips
    // collected_at_ms (would defeat dedup across replays).
    let mut hasher = blake3::Hasher::new();
    hasher.update(traj.session_id.as_bytes());
    hasher.update(traj.trajectory_id.as_bytes());
    hasher.update(traj.prompt.as_bytes());
    hasher.update(traj.full_trajectory.as_bytes());
    hasher.update(traj.final_diff.as_bytes());
    let hex = hasher.finalize().to_hex();
    // 16-hex-char prefix is the dedup key. blake3 collision space at 64
    // bits is far below any single corpus.
    hex.as_str()[..16].to_string()
}

/// Counter-only collector. The actual file-write is delegated to the
/// caller (the orchestrator's session-runtime owns the FS handle + the
/// ClockProvider), so this struct stays unit-testable without touching
/// disk. Per-rollout: caller calls `try_record(traj)`; if accepted, the
/// caller writes the JSONL row + appends to the index.
#[derive(Debug)]
pub struct RolloutCollector {
    storage_root: PathBuf,
    count: u64,
    threshold_emitted: bool,
}

/// Outcome of a single rollout collection attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectOutcome {
    /// Trajectory was eligible -- caller should write the JSONL row.
    Accepted { new_count: u64 },
    /// Trajectory rejected (non-PASS audit OR reward-hacking flag).
    Rejected,
    /// Trajectory accepted AND this push crossed MIN_ROLLOUTS_FOR_SFT.
    /// Caller emits `RewardLadderEventKind::SftTrainingThresholdReached`.
    AcceptedAtThreshold { new_count: u64 },
}

impl RolloutCollector {
    #[must_use]
    pub fn new(storage_root: PathBuf) -> Self {
        Self {
            storage_root,
            count: 0,
            threshold_emitted: false,
        }
    }

    /// Try to record one rollout. Returns the outcome WITHOUT touching
    /// disk -- the caller writes the JSONL row when AcceptedAtThreshold
    /// is the outcome (or Accepted before threshold).
    pub fn try_record(&mut self, traj: &GreenSessionTrajectory) -> CollectOutcome {
        if !traj.is_eligible_for_collection() {
            return CollectOutcome::Rejected;
        }
        self.count = self.count.saturating_add(1);
        if self.count >= MIN_ROLLOUTS_FOR_SFT && !self.threshold_emitted {
            self.threshold_emitted = true;
            CollectOutcome::AcceptedAtThreshold {
                new_count: self.count,
            }
        } else {
            CollectOutcome::Accepted {
                new_count: self.count,
            }
        }
    }

    #[must_use]
    pub fn current_count(&self) -> u64 {
        self.count
    }

    #[must_use]
    pub fn storage_root(&self) -> &PathBuf {
        &self.storage_root
    }

    /// Caller-side helper: return the per-session JSONL path given the
    /// session date (caller supplies a stable date string -- the
    /// orchestrator's ClockProvider produces it). Path layout matches the
    /// plan: `.elai_cc/training/rollouts/{date}/{session_id}.jsonl`.
    #[must_use]
    pub fn session_jsonl_path(&self, date: &str, session_id: &str) -> PathBuf {
        self.storage_root
            .join(date)
            .join(format!("{session_id}.jsonl"))
    }

    /// Caller-side helper: return the per-collector `_index.jsonl` path.
    #[must_use]
    pub fn index_path(&self) -> PathBuf {
        self.storage_root.join(ROLLOUT_INDEX_FILENAME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rl::mona_composite_reward::{
        PenaltyBreakdown, RewardHackingVerdict,
    };
    use crate::rl::reward_fusion::{
        FusedReward, FusionMode, FusionWeights, OrmSignal,
    };

    fn fused_at(score: f32) -> FusedReward {
        FusedReward::compute(
            vec![score],
            OrmSignal::TestsPass {
                fraction_passing: score,
            },
            FusionWeights::default(),
            FusionMode::Linear,
        )
    }

    fn composite_clean(base: f32) -> CompositeReward {
        CompositeReward {
            base,
            penalties: PenaltyBreakdown::zero(),
            final_reward: base,
            myopic_reward: base,
            step_validity: Some(1.0),
            hacking_verdict: RewardHackingVerdict::classify(base, 1.0),
        }
    }

    fn composite_hacked(base: f32) -> CompositeReward {
        CompositeReward {
            base,
            penalties: PenaltyBreakdown::zero(),
            final_reward: base,
            myopic_reward: base,
            step_validity: Some(0.2),
            hacking_verdict: RewardHackingVerdict::classify(base, 0.2),
        }
    }

    fn make_traj(id: &str, verdict: AuditVerdict, hacked: bool) -> GreenSessionTrajectory {
        GreenSessionTrajectory {
            session_id: "s-test".to_string(),
            trajectory_id: id.to_string(),
            prompt: "test prompt".to_string(),
            full_trajectory: "trace".to_string(),
            ppm_scores: vec![0.5],
            fused_reward: fused_at(0.9),
            composite_reward: if hacked {
                composite_hacked(0.9)
            } else {
                composite_clean(0.9)
            },
            rlaif_rating: None,
            final_diff: "diff".to_string(),
            audit_verdict: verdict,
            collected_at_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn threshold_constant_is_200() {
        assert_eq!(MIN_ROLLOUTS_FOR_SFT, 200);
    }

    #[test]
    fn rejects_blocked_session() {
        let mut c = RolloutCollector::new(PathBuf::from("/tmp/test-rollouts"));
        let t = make_traj("t1", AuditVerdict::Block, false);
        match c.try_record(&t) {
            CollectOutcome::Rejected => {}
            other => panic!("expected Rejected, got {other:?}"),
        }
        assert_eq!(c.current_count(), 0);
    }

    #[test]
    fn rejects_reward_hacking_flagged() {
        let mut c = RolloutCollector::new(PathBuf::from("/tmp/test-rollouts"));
        let t = make_traj("t1", AuditVerdict::Pass, true);
        match c.try_record(&t) {
            CollectOutcome::Rejected => {}
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    #[test]
    fn accepts_clean_pass() {
        let mut c = RolloutCollector::new(PathBuf::from("/tmp/test-rollouts"));
        let t = make_traj("t1", AuditVerdict::Pass, false);
        match c.try_record(&t) {
            CollectOutcome::Accepted { new_count } => assert_eq!(new_count, 1),
            other => panic!("expected Accepted, got {other:?}"),
        }
    }

    #[test]
    fn threshold_fires_at_200() {
        let mut c = RolloutCollector::new(PathBuf::from("/tmp/test-rollouts"));
        // 199 clean accepts.
        for i in 0..199 {
            let t = make_traj(&format!("t{i}"), AuditVerdict::Pass, false);
            match c.try_record(&t) {
                CollectOutcome::Accepted { .. } => {}
                other => panic!("expected Accepted, got {other:?}"),
            }
        }
        assert_eq!(c.current_count(), 199);
        let t = make_traj("t200", AuditVerdict::Pass, false);
        match c.try_record(&t) {
            CollectOutcome::AcceptedAtThreshold { new_count } => {
                assert_eq!(new_count, MIN_ROLLOUTS_FOR_SFT);
            }
            other => panic!("expected AcceptedAtThreshold, got {other:?}"),
        }
        // Subsequent rollouts after threshold stay Accepted (not re-fire).
        let t201 = make_traj("t201", AuditVerdict::Pass, false);
        match c.try_record(&t201) {
            CollectOutcome::Accepted { new_count } => {
                assert_eq!(new_count, MIN_ROLLOUTS_FOR_SFT + 1);
            }
            other => panic!("expected Accepted (post-threshold), got {other:?}"),
        }
    }

    #[test]
    fn content_hash_is_deterministic_and_blake3() {
        let t = make_traj("t1", AuditVerdict::Pass, false);
        let h1 = content_hash(&t);
        let h2 = content_hash(&t);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
        // Different traj id -> different hash.
        let t2 = make_traj("t2", AuditVerdict::Pass, false);
        assert_ne!(content_hash(&t2), h1);
    }

    #[test]
    fn session_jsonl_path_layout() {
        let c = RolloutCollector::new(PathBuf::from("/tmp/test-rollouts"));
        let p = c.session_jsonl_path("2026-05-27", "s-25120");
        assert!(p.ends_with("s-25120.jsonl"));
        assert!(p
            .components()
            .any(|c| c.as_os_str() == "2026-05-27"));
    }
}
