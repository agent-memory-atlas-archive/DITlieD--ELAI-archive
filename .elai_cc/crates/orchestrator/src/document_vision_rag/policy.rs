//! E-36 — vision-RAG policy gate.
//!
//! Pure-compute helpers for the per-project enable / storage-budget /
//! cloud-fallback-mode decisions. Producers of
//! `DocumentVisionRagEventKind::{VisionRagPolicyDecision,
//! DocParserCloudFallbackBlockedByMode}` call these helpers to make
//! the decision; the SSB emit itself is wired by the orchestrator
//! caller (it owns the `Arc<Ssb>` handle).
//!
//! HARD opt-in default per plan task 7: vision-RAG ships disabled
//! (`[knowledge.vision_rag] enabled = false`). Operators flip it
//! per-project only after the eval-harness gate (Task 9; nDCG@5 >= 0.50
//! on the 50-query ELAI corpus) passes.

use serde::{Deserialize, Serialize};

/// Default per-project storage budget (megabytes). Plan task 7 cites
/// 25KB/page * 1000 pages = 25MB as the typical-project ceiling; we
/// ship a 500MB ceiling (20x headroom) so projects with up to ~20k
/// indexed pages stay under the gate before the warning fires.
pub const DEFAULT_STORAGE_BUDGET_MB: u64 = 500;

/// Cloud-mode taxonomy. Mirrors the existing `--mode` flag the CLI
/// surface uses. Wire-dark default for vision-RAG is `Local` (cheapest;
/// HARD opt-in policy). Operators flip to `Hybrid` / `Cloud` once the
/// eval gate passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudMode {
    /// Local backends only. HARD-blocks every cloud parser; the policy
    /// emits `DocParserCloudFallbackBlockedByMode` on any escalation
    /// attempt.
    Local,
    /// Hybrid — local primary, cloud fallback permitted when
    /// `parser_confidence < cloud_fallback_threshold`.
    Hybrid,
    /// Cloud primary. Used only when local backends are unavailable.
    Cloud,
}

impl CloudMode {
    /// Snake-case wire label.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Hybrid => "hybrid",
            Self::Cloud => "cloud",
        }
    }

    /// True when cloud parser fallback is permitted.
    #[must_use]
    pub const fn allows_cloud(&self) -> bool {
        matches!(self, Self::Hybrid | Self::Cloud)
    }
}

/// Per-project vision-RAG policy snapshot. Carries the boolean enable
/// + storage-budget + cloud-mode triple the producer needs to make a
/// decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisionRagPolicy {
    /// HARD opt-in. `false` by default per `[knowledge.vision_rag]
    /// enabled = false` in `elai_cc.toml`. When false:
    /// `ImageArtifactChunker` still runs at ingest IF the source is in
    /// the visually-rich subset, but `multi_vec` is not populated —
    /// only VLM caption + page hash are stored.
    pub enabled: bool,
    /// Storage budget in megabytes. Drives the
    /// `actual_storage_mb > storage_budget_mb` warning surface.
    pub storage_budget_mb: u64,
    /// Cloud-mode for the doc_parser. Drives the
    /// `DocParserCloudFallbackBlockedByMode` decision.
    pub mode: CloudMode,
}

impl Default for VisionRagPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            storage_budget_mb: DEFAULT_STORAGE_BUDGET_MB,
            mode: CloudMode::Local,
        }
    }
}

/// Outcome of the policy gate. Producer call path turns this into the
/// `vision_rag_policy_decision` SSB row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisionRagPolicyDecision {
    /// Project id the decision was made for.
    pub project_id: String,
    /// Whether the vision-RAG path is enabled for this project.
    pub enabled: bool,
    /// Storage budget (MB) recorded at decision time.
    pub storage_budget_mb: u64,
    /// Optional measured storage usage (MB). When `Some(x)` and
    /// `x > storage_budget_mb`, downstream gates surface a budget
    /// warning.
    pub actual_storage_mb: Option<u64>,
}

impl VisionRagPolicyDecision {
    /// True when the actual storage usage exceeds the configured
    /// budget.
    #[must_use]
    pub fn over_budget(&self) -> bool {
        match self.actual_storage_mb {
            Some(actual) => actual > self.storage_budget_mb,
            None => false,
        }
    }
}

/// Pure decision: is the requested cloud parser BLOCKED by the
/// resolved cloud mode? Returns `Some(reason)` when blocked,
/// `None` when allowed. Producer emits the SSB row only when the
/// return is `Some(_)`.
#[must_use]
pub fn check_cloud_parser_blocked(
    backend_is_cloud: bool,
    mode: CloudMode,
) -> Option<&'static str> {
    if backend_is_cloud && !mode.allows_cloud() {
        Some("local_mode_blocks_cloud_parser")
    } else {
        None
    }
}

/// Pure decision: produce a `VisionRagPolicyDecision` from a policy
/// snapshot. Producer wires the SSB emit when the policy enables and
/// the actual_storage_mb is known.
#[must_use]
pub fn decide_policy(
    project_id: impl Into<String>,
    policy: &VisionRagPolicy,
    actual_storage_mb: Option<u64>,
) -> VisionRagPolicyDecision {
    VisionRagPolicyDecision {
        project_id: project_id.into(),
        enabled: policy.enabled,
        storage_budget_mb: policy.storage_budget_mb,
        actual_storage_mb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_is_disabled_local() {
        let p = VisionRagPolicy::default();
        assert!(!p.enabled, "vision-RAG ships disabled per HARD opt-in rule");
        assert_eq!(p.mode, CloudMode::Local);
        assert_eq!(p.storage_budget_mb, DEFAULT_STORAGE_BUDGET_MB);
    }

    #[test]
    fn local_mode_blocks_cloud_parser() {
        assert_eq!(
            check_cloud_parser_blocked(true, CloudMode::Local),
            Some("local_mode_blocks_cloud_parser")
        );
    }

    #[test]
    fn hybrid_mode_allows_cloud_parser() {
        assert_eq!(check_cloud_parser_blocked(true, CloudMode::Hybrid), None);
    }

    #[test]
    fn local_parser_always_allowed() {
        assert_eq!(check_cloud_parser_blocked(false, CloudMode::Local), None);
        assert_eq!(check_cloud_parser_blocked(false, CloudMode::Cloud), None);
    }

    #[test]
    fn decision_over_budget_when_actual_exceeds() {
        let policy = VisionRagPolicy {
            enabled: true,
            storage_budget_mb: 100,
            mode: CloudMode::Hybrid,
        };
        let dec = decide_policy("p1", &policy, Some(150));
        assert!(dec.over_budget());
        assert_eq!(dec.project_id, "p1");
        assert!(dec.enabled);
    }

    #[test]
    fn decision_under_budget_when_actual_below() {
        let policy = VisionRagPolicy {
            enabled: true,
            storage_budget_mb: 100,
            mode: CloudMode::Hybrid,
        };
        let dec = decide_policy("p2", &policy, Some(50));
        assert!(!dec.over_budget());
    }

    #[test]
    fn decision_unknown_storage_is_not_over_budget() {
        let policy = VisionRagPolicy::default();
        let dec = decide_policy("p3", &policy, None);
        assert!(!dec.over_budget());
    }

    #[test]
    fn cloud_mode_wire_labels() {
        assert_eq!(CloudMode::Local.as_str(), "local");
        assert_eq!(CloudMode::Hybrid.as_str(), "hybrid");
        assert_eq!(CloudMode::Cloud.as_str(), "cloud");
    }
}
