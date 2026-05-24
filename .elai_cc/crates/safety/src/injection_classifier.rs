//! E-37: Injection classifier role label (orchestrator wiring shim).
//!
//! The actual sidecar-backed classifier impl lives in
//! `crates/orchestrator/src/runtime_orchestrator.rs` because it depends on
//! the `PerRoleRuntime` cache + `AgentRole::InjectionClassifier` (which
//! lives in `elai-ipc-types`, a downstream crate of `elai-safety`).
//!
//! This module exposes the canonical role label string + a helper that
//! the orchestrator imports so the safety crate remains the single source
//! of truth for the LABEL even though the IMPL lives downstream. The
//! `AgentRole::InjectionClassifier` enum variant is the dispatch entry
//! point; its `serde(rename_all = "snake_case")` derive emits exactly the
//! string returned by `INJECTION_CLASSIFIER_ROLE_LABEL`.
//!
//! See arXiv:2507.15219 (PromptArmor) for the classifier-as-separate-role
//! design pattern.

use serde::{Deserialize, Serialize};

/// Canonical snake-case label for the injection classifier role.
///
/// Bound to `AgentRole::InjectionClassifier` via the
/// `#[serde(rename_all = "snake_case")]` derive on the enum in
/// `crates/ipc-types/src/lib.rs`.
pub const INJECTION_CLASSIFIER_ROLE_LABEL: &str = "injection_classifier";

/// Newtype wrapping the role label so downstream consumers can match on
/// type rather than free-floating &str. Lives here (in elai-safety) so
/// the orchestrator's `RuntimeConfig::per_role` mapping can be authored
/// without a circular dep on ipc-types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InjectionClassifierRole;

impl InjectionClassifierRole {
    pub fn label(self) -> &'static str {
        INJECTION_CLASSIFIER_ROLE_LABEL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_is_snake_case() {
        assert_eq!(INJECTION_CLASSIFIER_ROLE_LABEL, "injection_classifier");
        assert!(INJECTION_CLASSIFIER_ROLE_LABEL.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
    }

    #[test]
    fn role_newtype_label() {
        assert_eq!(InjectionClassifierRole.label(), "injection_classifier");
    }
}
