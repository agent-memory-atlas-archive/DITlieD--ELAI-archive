//! Batch-fu (FU.5 / A-31) — IrreversibleActionClassifier trait.
//!
//! The action classifier implementation lives in `elai-tools` (dep direction
//! tools → orchestrator). Task_runner therefore cannot call
//! `elai_tools::action_classifier::classify_and_gate` directly; instead, this
//! module publishes a narrow trait and value types the dashboard can bridge
//! via a concrete adapter (`crates/dashboard/src/lib.rs`).
//!
//! Keeping the surface narrow avoids dragging the full action classifier enum
//! set (RiskLevel / ActionPolicy / ActionClassification) into the orchestrator
//! crate. The trait returns one of three states that map 1:1 to the relevant
//! `GateDecision` variants from `elai_tools`:
//!
//! * `Allowed` — no confirmation needed; task_runner continues.
//! * `ConfirmationRequired { reason }` — task_runner must emit an
//!   `InterventionRequest` SSB row and wait for a matching
//!   `InterventionResponse` row.
//! * `Blocked { reason }` — task_runner aborts the step with a Finding.

use std::sync::Arc;

/// Classification verdict returned by [`IrreversibleActionClassifier::classify`].
///
/// Mirrors the three actionable arms of
/// `elai_tools::action_classifier::GateDecision` without exposing the full
/// `RiskLevel` / `ActionPolicy` / `ActionClassification` state the classifier
/// consumes internally. Keeps the orchestrator crate independent of
/// `elai-tools`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrreversibleVerdict {
    /// Action was classified and allowed with no confirmation required.
    Allowed,
    /// Action was classified as IRREVERSIBLE or SHARED_STATE. Task_runner
    /// must surface an `InterventionRequest` and block until a matching
    /// `InterventionResponse` arrives.
    ConfirmationRequired { reason: String },
    /// Action is outright blocked by policy (task_runner emits a
    /// `phase_gate_failure` Finding and skips execution).
    Blocked { reason: String },
}

/// Trait exposed to task_runner for per-dispatch classification of
/// potentially irreversible actions (A-31). The production implementation
/// lives in `dashboard::irreversible_classifier_adapter` and wraps
/// `elai_tools::action_classifier::classify_and_gate`.
pub trait IrreversibleActionClassifier: Send + Sync {
    /// Classify a dispatch described by `action_kind` (free-form label like
    /// `"bash"`, `"edit_file"`) and `action_summary` (short human-readable
    /// description — e.g. the baby-step goal or the shell command). Returns
    /// a verdict the runner can act on without touching the elai-tools crate.
    fn classify(
        &self,
        action_kind: &str,
        action_summary: &str,
    ) -> IrreversibleVerdict;
}

/// Convenience Arc alias used on `TaskRunnerConfig`.
pub type SharedIrreversibleClassifier = Arc<dyn IrreversibleActionClassifier>;

/// Stable action-kind string recorded on `InterventionRequestPayload.description`
/// so the frontend can branch on it without parsing free-form text.
pub const CONFIRM_IRREVERSIBLE_ACTION_KIND: &str = "confirm_irreversible";

/// Env-var kill switch. When set to any non-empty string, the dashboard
/// construction site skips wiring the classifier and `irreversible_classifier`
/// stays `None` — every dispatch proceeds as pre-batch-fu (no
/// confirmation prompts). This is orthogonal to `ELAI_DISABLE_AUTONOMOUS_GATE`
/// which covers the AutonomousGate path; both can be disabled independently.
pub const ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER_ENV: &str =
    "ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER";

pub fn irreversible_classifier_disabled_via_env() -> bool {
    std::env::var(ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER_ENV)
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CannedClassifier(IrreversibleVerdict);

    impl IrreversibleActionClassifier for CannedClassifier {
        fn classify(&self, _kind: &str, _summary: &str) -> IrreversibleVerdict {
            self.0.clone()
        }
    }

    #[test]
    fn trait_is_object_safe() {
        let c: Arc<dyn IrreversibleActionClassifier> =
            Arc::new(CannedClassifier(IrreversibleVerdict::Allowed));
        assert_eq!(c.classify("bash", "ls"), IrreversibleVerdict::Allowed);
    }

    #[test]
    fn env_disable_toggle() {
        let k = ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER_ENV;
        std::env::remove_var(k);
        assert!(!irreversible_classifier_disabled_via_env());
        std::env::set_var(k, "1");
        assert!(irreversible_classifier_disabled_via_env());
        std::env::remove_var(k);
    }
}
