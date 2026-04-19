//! Batch-fu (FU.5 / A-31) — IrreversibleActionClassifier + intervention
//! resolution contract.
//!
//! Verifies:
//!   * `IrreversibleActionClassifier` is object-safe (can be stored behind
//!     `Arc<dyn ...>`).
//!   * The `IrreversibleVerdict` enum round-trips all three arms.
//!   * `ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER` env kill switch toggles the
//!     `irreversible_classifier_disabled_via_env()` predicate.
//!   * `InterventionResolved` is a registered `SsbActionType` and round-trips.
//!   * `CONFIRM_IRREVERSIBLE_ACTION_KIND` is the stable action-kind string
//!     embedded on `InterventionRequestPayload.description` so the
//!     dashboard's InterventionCard can branch on it without parsing free text.

use std::sync::Arc;

use elai_orchestrator::{
    irreversible_classifier_disabled_via_env, InterventionResolvedPayload,
    IrreversibleActionClassifier, IrreversibleVerdict, SsbActionType, SsbPayload,
    CONFIRM_IRREVERSIBLE_ACTION_KIND, ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER_ENV,
};

struct CannedClassifier(IrreversibleVerdict);

impl IrreversibleActionClassifier for CannedClassifier {
    fn classify(&self, _kind: &str, _summary: &str) -> IrreversibleVerdict {
        self.0.clone()
    }
}

#[test]
fn contract_classifier_trait_is_object_safe() {
    let c: Arc<dyn IrreversibleActionClassifier> =
        Arc::new(CannedClassifier(IrreversibleVerdict::Allowed));
    assert_eq!(c.classify("bash", "ls"), IrreversibleVerdict::Allowed);
    let c2: Arc<dyn IrreversibleActionClassifier> = Arc::new(CannedClassifier(
        IrreversibleVerdict::ConfirmationRequired {
            reason: "irreversible".into(),
        },
    ));
    match c2.classify("bash", "rm -rf /") {
        IrreversibleVerdict::ConfirmationRequired { reason } => {
            assert_eq!(reason, "irreversible");
        }
        other => panic!("expected ConfirmationRequired, got {other:?}"),
    }
}

#[test]
fn contract_verdict_supports_blocked_arm() {
    let c: Arc<dyn IrreversibleActionClassifier> = Arc::new(CannedClassifier(
        IrreversibleVerdict::Blocked {
            reason: "policy".into(),
        },
    ));
    match c.classify("bash", "force push") {
        IrreversibleVerdict::Blocked { reason } => assert_eq!(reason, "policy"),
        other => panic!("expected Blocked, got {other:?}"),
    }
}

#[test]
fn contract_env_kill_switch_toggles_disabled_state() {
    let k = ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER_ENV;
    std::env::remove_var(k);
    assert!(!irreversible_classifier_disabled_via_env());
    std::env::set_var(k, "1");
    assert!(irreversible_classifier_disabled_via_env());
    std::env::remove_var(k);
}

#[test]
fn contract_intervention_resolved_is_a_registered_action_type() {
    assert_eq!(
        SsbActionType::InterventionResolved.as_sql_str(),
        "intervention_resolved",
    );
}

#[test]
fn contract_intervention_resolved_payload_roundtrips() {
    let p = InterventionResolvedPayload {
        intervention_id: "iv-0001".into(),
        approved: true,
        elapsed_ms: 512,
    };
    let wrapped = SsbPayload::InterventionResolved(p.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize");
    let round: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    assert!(matches!(round, SsbPayload::InterventionResolved(q) if q == p));
}

#[test]
fn contract_confirm_action_kind_is_stable() {
    // Frontend branches on this constant — changing it breaks the
    // InterventionCard branching. Pinned here so unrelated refactors cannot
    // drift the stable contract.
    assert_eq!(CONFIRM_IRREVERSIBLE_ACTION_KIND, "confirm_irreversible");
}
