//! E-33 — Dashboard UX agentic — SSB-readiness contract.
//!
//! Source plan:    docs/thefinalglance/E-plan/E-33_dashboard_ux_agentic.md
//!                 Task 9 (5 new SsbActionType variants for A/B telemetry of
//!                 the persistent ribbon + binary confidence + skeleton +
//!                 reduced-motion + window-footer-mode interactions).
//! Decisions:      workflow/reasoning/E-33-decisions.md
//!                 (c) SKIP-SSB-INSTRUMENTATION RATIONALE — the 5 variants
//!                 ship SCHEMA-DEFERRED per the binding from completion-drive
//!                 session 1 closure (E-17 precedent: producer code wired,
//!                 enum addition + emission gated on next QUINTUPLE bump).
//!
//! Purpose:
//!   - assert SCHEMA_VERSION_PUBLIC stays at 103 (the completion-drive
//!     session 1 LOCK) so a stray bump caught immediately
//!   - pin the bijection helper round-trip on the canonical SSB action
//!     types the dashboard UX surface already consumes today
//!     (intervention_request — backing permission_prompts store;
//!     reasoning_trace — backing grc confidence store)
//!   - lock the SCHEMA-DEFERRED string names for the 5 future variants so
//!     when SCHEMA unlocks the future test author already has the
//!     canonical SQL allowlist labels (no plan-vs-code drift on names)
//!
//! Build invocation:
//!   cargo check -p elai_cc_contracts --test activity_ribbon_ssb_contract
//!
//! No cargo test run in this closure (TEST-VERIFICATION-PENDING per user).

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SsbActionType, SCHEMA_VERSION_PUBLIC,
};

/// Canonical SQL allowlist labels reserved for the 5 SCHEMA-DEFERRED
/// E-33 SSB variants. When the next QUINTUPLE bump batch unlocks
/// SCHEMA_VERSION beyond 103 these labels move into ssb.rs and the
/// `*_round_trips_after_unlock` tests below get un-`#[ignore]`d.
const E33_DEFERRED_LABELS: [&str; 5] = [
    "activity_ribbon_interacted",
    "confidence_tag_hovered",
    "skeleton_replaced",
    "reduced_motion_activated",
    "window_footer_mode_changed",
];

// ---------------------------------------------------------------------------
// (a) SCHEMA freeze guard: SCHEMA_VERSION_PUBLIC must stay at the
// completion-drive session 1 LOCK (103). Any drift caught here so a stray
// bump cannot land silently mid-closure.
// ---------------------------------------------------------------------------

#[test]
fn e33_schema_version_remains_at_completion_drive_session_one_lock() {
    assert_eq!(
        SCHEMA_VERSION_PUBLIC, 103,
        "E-33 ships SCHEMA-DEFERRED per the completion-drive session 1 binding: \
         SCHEMA_VERSION_PUBLIC must remain 103 until the next QUINTUPLE bump batch \
         unlocks the 5 new variants together"
    );
}

// ---------------------------------------------------------------------------
// (b) Producer-ready surface: the existing SSB action types the dashboard UX
// stores already poll on today MUST round-trip through the bijection
// helpers. The 5 new variants are SCHEMA-DEFERRED so they live in the
// future-labels constant above; this test pins the two action types the
// current E-33 wires actually consume (intervention_request for the
// permission_prompts store backing SnackBarPermission; reasoning_trace for
// the grc store backing ConfidenceTag).
// ---------------------------------------------------------------------------

#[test]
fn intervention_request_round_trips_for_permission_prompts_consumer() {
    assert_eq!(
        SsbActionType::InterventionRequest.as_sql_str(),
        "intervention_request"
    );
    let parsed =
        ssb_action_type_from_str_public("intervention_request").expect("intervention_request parse");
    assert_eq!(parsed, SsbActionType::InterventionRequest);
}

#[test]
fn reasoning_trace_round_trips_for_grc_confidence_consumer() {
    assert_eq!(
        SsbActionType::ReasoningTrace.as_sql_str(),
        "reasoning_trace"
    );
    let parsed = ssb_action_type_from_str_public("reasoning_trace").expect("reasoning_trace parse");
    assert_eq!(parsed, SsbActionType::ReasoningTrace);
}

// ---------------------------------------------------------------------------
// (c) SCHEMA-DEFERRED stub_ban: the 5 future labels MUST NOT yet parse out
// of ssb_action_type_from_str_public — if a future commit silently lands
// one of them without bumping SCHEMA_VERSION and registering the producer
// code, this test fires and forces the QUINTUPLE/SEXTUPLE discipline.
// ---------------------------------------------------------------------------

#[test]
fn e33_deferred_labels_remain_unregistered_until_schema_unlock() {
    for label in E33_DEFERRED_LABELS {
        let parsed = ssb_action_type_from_str_public(label);
        assert!(
            parsed.is_err(),
            "E-33 SCHEMA-DEFERRED label '{label}' must NOT parse until the next \
             QUINTUPLE bump batch lands all 5 variants together. \
             If you're here from that bump, un-`#[ignore]` the round-trip tests \
             below and remove the label from E33_DEFERRED_LABELS."
        );
    }
}

// ---------------------------------------------------------------------------
// (d) Future round-trip lock: these tests are `#[ignore]`d while the
// variants are SCHEMA-DEFERRED. When the next bump lands they un-`#[ignore]`
// and assert the canonical SQL label survives the bijection helpers. The
// test bodies are intentionally minimal so the future author only has to
// (1) remove `#[ignore]`, (2) replace the placeholder match arm with the
// realized SsbActionType variant, (3) bump SCHEMA_VERSION_PUBLIC in the
// freeze guard above.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "SCHEMA-DEFERRED — un-ignore when SCHEMA bumps past 103 and the 5 \
            variants land in ssb.rs together"]
fn activity_ribbon_interacted_round_trips_after_unlock() {
    // After SCHEMA unlock: replace the early return with the realized variant.
    let parsed = ssb_action_type_from_str_public("activity_ribbon_interacted");
    assert!(parsed.is_ok(), "label must parse once the variant lands");
}

#[test]
#[ignore = "SCHEMA-DEFERRED — un-ignore on next QUINTUPLE bump batch"]
fn confidence_tag_hovered_round_trips_after_unlock() {
    let parsed = ssb_action_type_from_str_public("confidence_tag_hovered");
    assert!(parsed.is_ok(), "label must parse once the variant lands");
}

#[test]
#[ignore = "SCHEMA-DEFERRED — un-ignore on next QUINTUPLE bump batch"]
fn skeleton_replaced_round_trips_after_unlock() {
    let parsed = ssb_action_type_from_str_public("skeleton_replaced");
    assert!(parsed.is_ok(), "label must parse once the variant lands");
}

#[test]
#[ignore = "SCHEMA-DEFERRED — un-ignore on next QUINTUPLE bump batch"]
fn reduced_motion_activated_round_trips_after_unlock() {
    let parsed = ssb_action_type_from_str_public("reduced_motion_activated");
    assert!(parsed.is_ok(), "label must parse once the variant lands");
}

#[test]
#[ignore = "SCHEMA-DEFERRED — un-ignore on next QUINTUPLE bump batch"]
fn window_footer_mode_changed_round_trips_after_unlock() {
    let parsed = ssb_action_type_from_str_public("window_footer_mode_changed");
    assert!(parsed.is_ok(), "label must parse once the variant lands");
}

// ---------------------------------------------------------------------------
// (e) Catalogue lock: the SCHEMA-DEFERRED label set is exactly 5 entries
// (matching plan task 9). A future planner that adds a 6th UX-instrumentation
// label without bumping SCHEMA + revisiting this contract is caught here.
// ---------------------------------------------------------------------------

#[test]
fn e33_deferred_label_set_size_is_five() {
    assert_eq!(
        E33_DEFERRED_LABELS.len(),
        5,
        "plan task 9 mandates exactly 5 SCHEMA-DEFERRED variants — \
         a different count means the plan + contract have drifted"
    );
}
