//! E-19 task 3 — L1->L3 direct-route helper.
//!
//! When the [`elai_core::reasoning::ConfidenceGate`] returns
//! `GateVerdict::L3Direct` (L1 self-certainty below the 0.40 floor), the
//! orchestrator skips L2 micro-reflection entirely and promotes L3
//! InterrogationTree dispatch. This module owns the routing decision: it
//! picks a built tree slug from the FailureTaxonomy category so the
//! navigator can call the existing `rules::interrogation_store` constructor
//! without a fresh round-trip.
//!
//! Paper grounding: arXiv:2507.02778 (Self-Correction Bench — LLMs correct
//! external errors not their own; 64.5% blind-spot rate). The L3 tree IS
//! the extrinsic verification path.
//!
//! IO-free.

use serde::{Deserialize, Serialize};

/// Outcome of the L3 direct-route resolver.
///
/// Carries the tree slug the navigator MUST construct + a short snake_case
/// rationale (logged into the SSB row payload extras).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct L3DirectRoute {
    /// Snake_case tree slug — matches one of the canonical
    /// `rules::interrogation_store::build_*_tree` constructors.
    pub tree_slug: String,
    /// Short reason for the chosen tree.
    pub reason: String,
}

/// Pick an InterrogationTree slug from a FailureTaxonomy category. The
/// default tree slug (`generic_low_confidence_tree`) is used when no
/// category-specific tree exists.
#[must_use]
pub fn l3_direct_route(failure_taxonomy_category: &str) -> L3DirectRoute {
    match failure_taxonomy_category {
        "missing_import" => L3DirectRoute {
            tree_slug: "python_import_tree".into(),
            reason: "missing_import_category".into(),
        },
        "silent_exception_swallow" => L3DirectRoute {
            tree_slug: "go_error_tree".into(),
            reason: "silent_exception_swallow_category".into(),
        },
        "concurrent_state_race" => L3DirectRoute {
            tree_slug: "generic_low_confidence_tree".into(),
            reason: "concurrent_state_race_category".into(),
        },
        "cross_lang_drift" => L3DirectRoute {
            tree_slug: "generic_low_confidence_tree".into(),
            reason: "cross_lang_drift_category".into(),
        },
        _ => L3DirectRoute {
            tree_slug: "generic_low_confidence_tree".into(),
            reason: "default_low_confidence".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_import_routes_python_tree() {
        let r = l3_direct_route("missing_import");
        assert_eq!(r.tree_slug, "python_import_tree");
        assert_eq!(r.reason, "missing_import_category");
    }

    #[test]
    fn silent_exception_swallow_routes_go_tree() {
        let r = l3_direct_route("silent_exception_swallow");
        assert_eq!(r.tree_slug, "go_error_tree");
    }

    #[test]
    fn unknown_category_routes_generic_tree() {
        let r = l3_direct_route("something_weird");
        assert_eq!(r.tree_slug, "generic_low_confidence_tree");
        assert_eq!(r.reason, "default_low_confidence");
    }
}
