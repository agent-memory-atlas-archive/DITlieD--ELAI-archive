//! C-37 contract: per-role decoding HYPERPARAMETERS (Structure
//! Snowballing re-evaluation).
//!
//! Paper grounding: arXiv:2604.06066 Section 4.3 (Zhou 2026, "From
//! Hallucination to Structure Snowballing: The Alignment Tax of
//! Constrained Decoding in LLM Reflection"). The paper-grounded values
//! are pinned for Critic + Reflection; Planner + Coder retain the
//! pre-C-37 ELAI defaults and are tagged `ungrounded-pending-eval` so
//! future grounding work flips them per-role rather than en bloc.
//!
//! Pins (drift detection):
//!   * `PER_ROLE_DECODING_DEFAULTS` table has 4 unique entries spanning
//!     the existing 5-variant `RoleDecoding` enum (designer / executor /
//!     critic / auditor / researcher); count assertion guards against
//!     silent deletion of policy rows.
//!   * `PER_ROLE_DECODING_PARAMS_PAPER` table has exactly 4 entries
//!     covering every `PaperRole` variant.
//!   * `paper_decoding_params_for(Critic)` returns the paper Section 4.3
//!     values (temperature=0.1, max_new_tokens=1024, constrained=false).
//!   * `paper_decoding_params_for(Reflection)` returns the paper Section
//!     3.3 + 4.3 values (temperature=0.1, max_new_tokens=1024,
//!     constrained=true) — the structure-snowballing trigger condition.
//!   * `default_decoding_for(role)` is callable from a downstream crate
//!     (closes GAP-B: the function had ZERO production callers pre-C-37
//!     and was only exercised by this contract surface).
//!   * `SsbActionType::PerRoleDecodingApplied` round-trips via
//!     `as_sql_str` + `from_str`.
//!   * `PerRoleDecodingAppliedPayload` round-trips via serde JSON.
//!   * `SCHEMA_VERSION_PUBLIC >= 106` (C-37 bumped 105 -> 106).

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, PerRoleDecodingAppliedPayload, SsbActionType,
    SCHEMA_VERSION_PUBLIC,
};
use elai_providers::{
    default_decoding_for, paper_decoding_params_for, PaperRole, RoleDecoding,
    PER_ROLE_DECODING_DEFAULTS, PER_ROLE_DECODING_PARAMS_PAPER,
};

// ---------------------------------------------------------------------------
// F1 — PER_ROLE_DECODING_DEFAULTS table has 4 unique roles (paper +
// ungrounded combined) spanning the existing 5-variant enum.
// ---------------------------------------------------------------------------

#[test]
fn c37_per_role_decoding_defaults_table_has_five_entries() {
    // C-26 shipped the table with 5 entries (one per RoleDecoding
    // variant). C-37 keeps the C-26 surface intact and adds a parallel
    // 4-entry paper-grounded table for the 4-role taxonomy.
    assert_eq!(PER_ROLE_DECODING_DEFAULTS.len(), 5);
}

#[test]
fn c37_paper_table_has_four_entries() {
    // Paper Section 3.3 defines exactly 3 roles (Actor/Evaluator/Reflector)
    // but ELAI's internal taxonomy adds Planner, so 4 entries.
    assert_eq!(PER_ROLE_DECODING_PARAMS_PAPER.len(), 4);
}

// ---------------------------------------------------------------------------
// F2 — CRITIC + REFLECTION values match arXiv:2604.06066 grounding.
// ---------------------------------------------------------------------------

#[test]
fn c37_critic_decoding_matches_paper_section_43() {
    let p = paper_decoding_params_for(PaperRole::Critic);
    // Paper Section 4.3 verbatim: temperature=0.1, max_new_tokens=1024.
    assert_eq!(p.temperature, 0.1);
    assert_eq!(p.max_new_tokens, 1024);
    // Evaluator (paper) <=> Critic (ELAI) is FREE-TEXT in the paper — the
    // "strictly limit the evaluator to verifying semantic equivalence"
    // language at Section 4.3 implies no FSM constraint.
    assert!(!p.constrained, "Critic must be free-text per paper Section 4.3");
    // Grounding tag is the audit hook for downstream SSB beacons.
    assert_eq!(p.grounding, "paper-2604.06066-S4.3");
}

#[test]
fn c37_reflection_decoding_matches_paper_section_33() {
    let p = paper_decoding_params_for(PaperRole::Reflection);
    // Paper Section 4.3: uniform hyperparameters.
    assert_eq!(p.temperature, 0.1);
    assert_eq!(p.max_new_tokens, 1024);
    // Paper Section 3.3: Reflector is constrained via Outlines FSM
    // over the 5-class pydantic schema. This is the trigger condition
    // under measurement — do not flip without pilot data.
    assert!(
        p.constrained,
        "Reflection must be constrained per paper Section 3.3 — this is the structure-snowballing trigger"
    );
    assert_eq!(p.grounding, "paper-2604.06066-S3.3-S4.3");
}

#[test]
fn c37_planner_and_coder_marked_ungrounded() {
    // Planner is NOT in the paper (ELAI extension); Coder defers per
    // the explicit C-37 scope ("PLANNER + CODER values stay as current
    // defaults UNTIL grounded"). Both must carry the ungrounded marker
    // so future plans can find them by grep.
    for role in [PaperRole::Planner, PaperRole::Coder] {
        let p = paper_decoding_params_for(role);
        assert_eq!(
            p.grounding, "ungrounded-pending-eval",
            "{role:?} must carry ungrounded marker until measured"
        );
    }
}

#[test]
fn c37_paper_table_matches_lookup_fn() {
    for &(role, temp, mnt, con) in PER_ROLE_DECODING_PARAMS_PAPER.iter() {
        let p = paper_decoding_params_for(role);
        assert_eq!(p.temperature, temp, "{role:?} temperature drift");
        assert_eq!(p.max_new_tokens, mnt, "{role:?} max_new_tokens drift");
        assert_eq!(p.constrained, con, "{role:?} constrained drift");
    }
}

// ---------------------------------------------------------------------------
// F3 — default_decoding_for(role) is callable (closes GAP-B existence test).
// ---------------------------------------------------------------------------

#[test]
fn c37_default_decoding_for_is_callable_from_downstream_crate() {
    // GAP-B: pre-C-37 the only callers of default_decoding_for were
    // the providers-internal tests + the c26 contract test. This test
    // re-asserts callability from the contracts crate (downstream of
    // providers) so any visibility regression surfaces here.
    for role in [
        RoleDecoding::Designer,
        RoleDecoding::Executor,
        RoleDecoding::Critic,
        RoleDecoding::Auditor,
        RoleDecoding::Researcher,
    ] {
        let cfg = default_decoding_for(role);
        // Sanity: the config has at least the legacy default fields wired.
        assert!(cfg.max_retries_on_invalid >= 1);
    }
}

#[test]
fn c37_role_decoding_from_snake_case_covers_per_level_auditors() {
    // C-37 added a from_snake_case helper so the agent_pool dispatch
    // site can map AgentRole strings onto RoleDecoding without taking
    // a providers->orchestrator dep. The B-41 per-level auditors must
    // collapse to RoleDecoding::Auditor so the JsonSchema default applies
    // uniformly.
    for label in [
        "auditor",
        "auditor_concepts",
        "auditor_phases",
        "auditor_steps",
        "auditor_babysteps",
        "auditor_babystep_detail",
    ] {
        assert_eq!(
            RoleDecoding::from_snake_case(label),
            Some(RoleDecoding::Auditor),
            "auditor variant {label:?} must collapse to RoleDecoding::Auditor"
        );
    }
    // Unknown role -> None (caller falls back to legacy behaviour).
    assert_eq!(RoleDecoding::from_snake_case("idea_framer"), None);
    assert_eq!(RoleDecoding::from_snake_case("not_a_role"), None);
}

// ---------------------------------------------------------------------------
// F4 — SsbActionType::PerRoleDecodingApplied round-trips.
// ---------------------------------------------------------------------------

#[test]
fn c37_per_role_decoding_applied_action_type_roundtrip() {
    let v = SsbActionType::PerRoleDecodingApplied;
    let s = v.as_sql_str();
    assert_eq!(s, "per_role_decoding_applied");
    let back = ssb_action_type_from_str_public(s).expect("from_str must succeed");
    assert_eq!(back, v);
}

#[test]
fn c37_per_role_decoding_applied_payload_roundtrip() {
    let payload = PerRoleDecodingAppliedPayload {
        role: "auditor".into(),
        decoding_mode: "json_schema".into(),
        constrained: true,
        max_retries_on_invalid: 3,
    };
    let json = serde_json::to_string(&payload).expect("serialize");
    let back: PerRoleDecodingAppliedPayload =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(payload, back);
}

#[test]
fn c37_schema_version_bumped_to_at_least_106() {
    // C-37 bumped SCHEMA_VERSION from 105 to 106. The >= assertion
    // tolerates concurrent bumps from other C-plan / D-plan sessions
    // that may push the value higher between contract authoring and
    // CI run.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 106,
        "SCHEMA_VERSION_PUBLIC must be >= 106 after C-37; got {SCHEMA_VERSION_PUBLIC}"
    );
}
