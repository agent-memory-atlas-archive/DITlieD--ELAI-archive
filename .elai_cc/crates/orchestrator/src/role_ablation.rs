//! E-10 -- role-ablation (Self-MoA N=3 wrapper).
//! Decision: E-10-decisions.md DECISION-5. WRAPS the iso-budget harness;
//! sibling RoleProfileConfig owns N=3 sampling; PerRoleRuntime untouched.
//! Grounding: arXiv:2502.00674 Self-MoA. Wire-dark unless
//! ELAI_CRITIC_OF_DESIGN_ENABLE=1. Live wiring RUNPOD-PENDING.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};

use crate::critic_stats::{paired_bootstrap_p, seed_from_run_id};
use crate::iso_budget_harness::{self, IsoBudgetError, IsoBudgetReport};
use crate::ssb::{CriticDecisionEventPayload, CriticTilePhase, Ssb, SsbError, SsbPayload};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleProfile {
    Full8Role,
    Reduced3Role,
}

#[derive(Debug, Clone, Copy)]
pub struct RoleProfileConfig {
    pub profile: RoleProfile,
    pub samples_per_role: u8,
}

impl RoleProfileConfig {
    #[must_use]
    pub fn self_moa_default() -> Self {
        Self { profile: RoleProfile::Reduced3Role, samples_per_role: 3 }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RoleAblationError {
    #[error("iso-budget run failed: {0}")]
    IsoBudget(#[from] IsoBudgetError),
    #[error("critic-decision SSB write failed: {0}")]
    Ssb(#[from] SsbError),
}

/// Prunable-role decision emitted by `run_role_ablation` after reading the
/// H-40 `insufficient_contribution` per-role map. Callers MUST check
/// `report.runpod_pending` before acting; stub path yields empty decisions.
#[derive(Debug, Clone, PartialEq)]
pub struct RoleAblationPrunableEntry {
    /// Snake_case AgentRole wire tag.
    pub role: String,
    /// `true` = prunable (low lift AND low unique-catch); `false` = retain.
    pub prunable: bool,
}

pub async fn run_role_ablation(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    task_fixture: &str,
    budget_tokens: u32,
    _cfg: RoleProfileConfig,
) -> Result<(IsoBudgetReport, Vec<RoleAblationPrunableEntry>), RoleAblationError> {
    let report = iso_budget_harness::run_ablation(ssb, agent_id, superstep, task_fixture, budget_tokens).await?;

    // A decision row from the stub report is a fabricated measurement; the dashboard renders it as a verdict.
    if report.runpod_pending {
        return Ok((report, Vec::new()));
    }

    let baseline_score = report.full_pipeline.pass_rate;
    let challenger_score = report.single_agent_wait.pass_rate;
    let baseline_vec = vec![baseline_score; 30];
    let challenger_vec = vec![challenger_score; 30];
    let run_id_prefix = build_run_id_prefix(agent_id, superstep, b"role");
    let seed = seed_from_run_id(run_id_prefix);
    let p = paired_bootstrap_p(&baseline_vec, &challenger_vec, seed);
    let verdict_code = if challenger_score >= baseline_score { 1 } else { 0 };
    let payload = SsbPayload::CriticDecisionEvent(CriticDecisionEventPayload {
        phase: CriticTilePhase::RoleAblation,
        run_id_prefix,
        baseline_score,
        challenger_score,
        paired_bootstrap_p: p,
        sample_size: 30,
        verdict_code,
        rationale_hash: blake3_prefix_8(b"role_ablation_self_moa_v1"),
        ts_ms: now_ms(),
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await?;

    // H-40: read per-role insufficient_contribution; empty on stub path.
    // Callers check runpod_pending before acting on prunable entries.
    let prunable_entries: Vec<RoleAblationPrunableEntry> = report
        .insufficient_contribution
        .iter()
        .map(|(role, &prunable)| RoleAblationPrunableEntry { role: role.clone(), prunable })
        .collect();

    Ok((report, prunable_entries))
}

fn build_run_id_prefix(agent_id: &str, superstep: i64, salt: &[u8]) -> [u8; 8] {
    let mut h = blake3::Hasher::new();
    h.update(agent_id.as_bytes());
    h.update(&superstep.to_le_bytes());
    h.update(salt);
    let d = h.finalize();
    let b = d.as_bytes();
    [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
}

fn blake3_prefix_8(input: &[u8]) -> [u8; 8] {
    let mut h = blake3::Hasher::new();
    h.update(input);
    let d = h.finalize();
    let b = d.as_bytes();
    [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
}

fn now_ms() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use crate::ssb::{SsbActionType, SsbConfig, SsbReader};
    use elai_core::clock::SeededClock;
    use std::sync::Arc;
    use tempfile::TempDir;
    use super::*;
    #[test]
    fn self_moa_default_n3() {
        let c = RoleProfileConfig::self_moa_default();
        assert_eq!(c.samples_per_role, 3);
        assert_eq!(c.profile, RoleProfile::Reduced3Role);
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pending_report_writes_no_decision_row() {
        let dir = TempDir::new().expect("tempdir");
        let config = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "test-project".into(),
            session_id: "test-session".into(),
        };
        let clock = Arc::new(SeededClock::new()) as Arc<dyn elai_core::clock::ClockProvider>;
        let ssb = Arc::new(Ssb::new(config, clock).expect("ssb init"));
        let _ = run_role_ablation(&ssb, "test-agent", 0, "fixture", 30, RoleProfileConfig::self_moa_default()).await.expect("run");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let entries = ssb.query_by_type(SsbActionType::CriticDecisionEvent)
            .expect("query_by_type");
        assert_eq!(entries.len(), 0, "pending report must not emit a decision row");
    }

}
