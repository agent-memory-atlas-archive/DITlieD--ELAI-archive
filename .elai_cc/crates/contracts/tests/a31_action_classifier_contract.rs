//! Batch 4C (A-31) — Action classifier + confirmation gate contract.
//!
//! Pins the four classification tiers (Safe / Reversible / Irreversible /
//! SharedState) produced by `elai_tools::action_classifier::classify`, and
//! asserts the gate decision the production runtime consumes:
//!   * SAFE tools never require confirmation
//!   * Irreversible + SharedState trigger ConfirmationRequired under the
//!     default policy (or Block in autonomous mode)
//!   * the InterventionRequest / InterventionResponse payloads round-trip
//!     through the SSB discriminant table
//!
//! Grounding:
//!   * crates/tools/src/action_classifier.rs (RiskLevel, ActionPolicy,
//!     classify, classify_and_gate, GateDecision)
//!   * crates/orchestrator/src/ssb.rs InterventionRequestPayload

use elai_orchestrator::{
    InterventionKeyword, InterventionOption, InterventionRequestPayload,
    InterventionResponsePayload, InterventionType, SsbActionType, SsbPayload,
};
use elai_tools::action_classifier::{
    classify, classify_and_gate, ActionClassification, ActionPolicy, GateDecision, RiskLevel,
};

// --- (a) Read-only built-in tools classify as Safe. ---

#[test]
fn contract_safe_tools_classify_as_safe() {
    let val = serde_json::json!({});
    // `read_file` is the canonical Safe tool name per
    // crates/tools/src/action_classifier.rs SAFE_TOOLS.
    let c = classify("read_file", &val);
    assert_eq!(c.risk, RiskLevel::Safe);
}

// --- (b) Edit classifies as Reversible. ---

#[test]
fn contract_edit_classifies_as_reversible() {
    let val = serde_json::json!({
        "file": "src/foo.rs",
        "old_string": "a",
        "new_string": "b"
    });
    // `edit_file` is the canonical REVERSIBLE_TOOLS entry.
    let c = classify("edit_file", &val);
    assert_eq!(c.risk, RiskLevel::Reversible);
}

// --- (c) WorkerTerminate classifies as SharedState. ---

#[test]
fn contract_worker_terminate_classifies_as_shared_state() {
    let val = serde_json::json!({ "worker_id": "w-1" });
    // `WorkerTerminate` is the canonical SHARED_STATE_TOOLS entry.
    let c = classify("WorkerTerminate", &val);
    assert_eq!(c.risk, RiskLevel::SharedState);
}

// --- (d) Bash rm -rf escalates to Irreversible. ---

#[test]
fn contract_bash_rm_rf_escalates_to_irreversible() {
    let val = serde_json::json!({ "command": "rm -rf /tmp/target" });
    let c = classify("bash", &val);
    assert_eq!(c.risk, RiskLevel::Irreversible);
}

// --- (e) Default policy allows Safe. ---

#[test]
fn contract_default_policy_allows_safe() {
    let policy = ActionPolicy::default();
    let val = serde_json::json!({});
    let (decision, _trace) = classify_and_gate("read", &val, &policy);
    assert!(
        matches!(decision, GateDecision::Allow),
        "Safe must be allowed under default policy, got: {decision:?}"
    );
}

// --- (f) Default policy never silently allows Irreversible. ---

#[test]
fn contract_default_policy_gates_dangerous_tiers() {
    let policy = ActionPolicy::default();
    let val = serde_json::json!({ "command": "rm -rf /tmp/x" });
    let (decision, _trace) = classify_and_gate("bash", &val, &policy);
    match decision {
        GateDecision::Allow => panic!("Irreversible must not be silently allowed"),
        GateDecision::Block { .. } | GateDecision::ConfirmationRequired { .. } => {}
    }
}

// --- (g) InterventionRequest payload round-trips through the SSB enum. ---

#[test]
fn contract_intervention_request_payload_roundtrips() {
    let p = InterventionRequestPayload {
        step_id: "step-1".into(),
        intervention_type: InterventionType::ApproveChange,
        description: "confirm_irreversible".into(),
        diff_summary: None,
        options: vec![
            InterventionOption::Keyword(InterventionKeyword::Accept),
            InterventionOption::Keyword(InterventionKeyword::Decline),
        ],
    };
    let wrapped = SsbPayload::InterventionRequest(p);
    assert_eq!(wrapped.action_type(), SsbActionType::InterventionRequest);
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("msgpack");
    let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(back.action_type(), SsbActionType::InterventionRequest);
}

// --- (h) InterventionResponse carries the approve/decline outcome. ---

#[test]
fn contract_intervention_response_carries_approved_flag() {
    for (k, approved) in [
        (InterventionKeyword::Accept, true),
        (InterventionKeyword::Decline, false),
    ] {
        let resp = InterventionResponsePayload {
            request_id: 42,
            chosen: InterventionOption::Keyword(k),
            user_note: None,
        };
        let wrapped = SsbPayload::InterventionResponse(resp);
        let bytes = rmp_serde::to_vec_named(&wrapped).expect("msgpack");
        let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
        match back {
            SsbPayload::InterventionResponse(r) => {
                let is_accept = matches!(
                    r.chosen,
                    InterventionOption::Keyword(InterventionKeyword::Accept)
                );
                assert_eq!(
                    is_accept, approved,
                    "approved bool must match chosen keyword"
                );
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }
}

// --- (i) ActionClassification is Clone so it can be passed by value. ---

#[test]
fn contract_action_classification_is_clone() {
    let c = ActionClassification {
        risk: RiskLevel::Reversible,
        reason: "edit".into(),
    };
    let c2 = c.clone();
    assert_eq!(c.risk, c2.risk);
}
