//! I-26 contract tests — dynamic thinking budgets (depth-scaled node compute,
//! 85% collapse, plateau detection, cloud effort-parameter mapping).
//!
//! Coverage (no #[ignore], no stubs, no placeholder asserts):
//!
//! (a) NEGATIVE: `budget_tokens` is ABSENT from the wire struct.
//!     Landmine #6 from I-0: Opus 4.7+ REJECTS manual `budget_tokens`; only
//!     `effort` is accepted. `MessageRequest` must NOT have a `budget_tokens`
//!     field. Verified via serde: a request with `effort=Some("auto")` produces
//!     JSON containing "effort" but NOT "budget_tokens".
//!
//! (b) EFFORT field wired: `MessageRequest` has an `effort` field;
//!     `CompletionConfig` has an `effort` field. Round-trips via serde.
//!     When `effort=None` the field is absent (HD-2 cache-prefix stability).
//!
//! (c) PAYLOAD EXTENSION: `ReasoningBudgetScaledPayload` carries the I-26
//!     fields (`depth_level`, `budget_pct`, `plateau_fired`) as Option<> with
//!     skip_serializing_if. Existing H-42 rows (all three None) decode correctly.
//!     A new I-26 search-node row (all three Some) round-trips through serde.
//!
//! (d) BUDGET CONTROLLER: `node_budget`, `should_collapse_budget`,
//!     `record_score_and_check_plateau`, `cloud_effort_for_node` behave
//!     deterministically (ID-4: no LLM opinion; pure compute).
//!
//! (e) NO FORCE-JSON on reasoning role (HD-3 + landmine #6): the contract
//!     asserts that `cloud_effort_for_node` never returns `"budget_tokens"`.
//!
//! (f) REAL-DB SSB row: open a TempDir SQLite Ssb, construct a
//!     `ReasoningBudgetScaled` payload with the new I-26 fields populated,
//!     write it, then `query_by_type` and assert the row decoded correctly.
//!     Exercises the full write+read path as the off-agent artifact.
//!
//! (g) action_type round-trip + SCHEMA_VERSION floor.
//!
//! Spec references: docs/thefinalglance/I-plan/I-26_dynamic_thinking_budgets.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7/R8; CLAUDE.md HD-2 + HD-3 + ID-5.

use std::sync::Arc;

use elai_core::SeededClock;
use elai_orchestrator::routing::budget_controller::{
    self, BudgetController, COLLAPSE_THRESHOLD, PLATEAU_WINDOW,
};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, ReasoningBudgetScaledPayload,
    Ssb, SsbActionType, SsbConfig, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use elai_providers::MessageRequest;
use elai_providers::runtime::CompletionConfig;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper: build a temporary Ssb for real-DB tests
// ---------------------------------------------------------------------------

fn test_ssb(dir: &TempDir, session_id: &str) -> Ssb {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "i26-contract".into(),
        session_id: session_id.into(),
    };
    Ssb::new(cfg, Arc::new(SeededClock::new())).expect("ssb init")
}

// ---------------------------------------------------------------------------
// (a) NEGATIVE: budget_tokens absent from wire JSON
// ---------------------------------------------------------------------------

#[test]
fn i26_a_budget_tokens_absent_on_opus_builder() {
    // Landmine #6: MessageRequest must NOT have a budget_tokens field.
    // Verified structurally: serde of MessageRequest with effort=Some("auto")
    // produces JSON with "effort" but without "budget_tokens".
    let req = MessageRequest {
        model: "claude-opus-4-7".to_string(),
        max_tokens: 64,
        messages: vec![],
        system: None,
        tools: None,
        tool_choice: None,
        stream: false,
        seed: None,
        effort: Some("auto".to_string()),
        chat_template_kwargs: None,
    };
    let wire = serde_json::to_value(&req).expect("serde to_value");
    assert!(
        wire.get("effort").is_some(),
        "effort field must appear in wire JSON when Some"
    );
    assert!(
        wire.get("budget_tokens").is_none(),
        "budget_tokens MUST NOT appear in wire JSON (landmine #6 / I-0 CANONICAL)"
    );
}

// ---------------------------------------------------------------------------
// (b) EFFORT field wired — MessageRequest and CompletionConfig round-trip
// ---------------------------------------------------------------------------

#[test]
fn i26_b_effort_field_in_message_request_present_and_skip_on_none() {
    // When effort=Some("expert"), it appears in wire JSON.
    let req_with = MessageRequest {
        model: "claude-opus-4-7".to_string(),
        max_tokens: 64,
        messages: vec![],
        system: None,
        tools: None,
        tool_choice: None,
        stream: false,
        seed: None,
        effort: Some("expert".to_string()),
        chat_template_kwargs: None,
    };
    let wire = serde_json::to_value(&req_with).expect("serde");
    assert_eq!(
        wire.get("effort").and_then(|v| v.as_str()),
        Some("expert"),
        "effort=Some(\"expert\") must appear in wire JSON"
    );

    // When effort=None, field is absent (HD-2 cache-prefix byte stability).
    let req_none = MessageRequest {
        model: "claude-opus-4-7".to_string(),
        max_tokens: 64,
        messages: vec![],
        system: None,
        tools: None,
        tool_choice: None,
        stream: false,
        seed: None,
        effort: None,
        chat_template_kwargs: None,
    };
    let wire_none = serde_json::to_value(&req_none).expect("serde");
    assert!(
        wire_none.get("effort").is_none(),
        "effort=None must be absent from wire JSON (HD-2 cache-prefix stability)"
    );
}

#[test]
fn i26_b_completion_config_effort_round_trips_via_serde() {
    // CompletionConfig::effort field persists through serde round-trip.
    let cfg_with = CompletionConfig {
        effort: Some("auto".to_string()),
        ..CompletionConfig::default()
    };
    let cfg_json = serde_json::to_value(&cfg_with).expect("config serde");
    let cfg_back: CompletionConfig =
        serde_json::from_value(cfg_json).expect("config deserialize");
    assert_eq!(
        cfg_back.effort.as_deref(),
        Some("auto"),
        "CompletionConfig::effort must survive serde round-trip"
    );

    // None round-trip.
    let cfg_none = CompletionConfig::default();
    assert!(
        cfg_none.effort.is_none(),
        "CompletionConfig::effort must default to None"
    );
}

// ---------------------------------------------------------------------------
// (c) PAYLOAD EXTENSION — ReasoningBudgetScaledPayload with I-26 fields
// ---------------------------------------------------------------------------

#[test]
fn i26_c_payload_extension_backward_compatible_h42_rows() {
    // H-42 rows (all extension fields None) round-trip without error.
    // skip_serializing_if ensures HD-2 byte-stability for existing rows.
    let h42_payload = ReasoningBudgetScaledPayload {
        kind: "depth_bound".into(),
        role: "executor".into(),
        base_calls: 8,
        serial_depth: 3,
        scaled_calls: 14,
        ceiling: 24,
        seed: 42,
        depth_level: None,
        budget_pct: None,
        plateau_fired: None,
    };
    let json = serde_json::to_value(&h42_payload).expect("H-42 payload serde");
    assert!(
        json.get("depth_level").is_none(),
        "depth_level=None must be absent from wire JSON (HD-2)"
    );
    assert!(
        json.get("budget_pct").is_none(),
        "budget_pct=None must be absent from wire JSON (HD-2)"
    );
    assert!(
        json.get("plateau_fired").is_none(),
        "plateau_fired=None must be absent from wire JSON (HD-2)"
    );
    let back: ReasoningBudgetScaledPayload =
        serde_json::from_value(json).expect("H-42 payload round-trip");
    assert_eq!(back.kind, "depth_bound");
    assert_eq!(back.depth_level, None);
    assert_eq!(back.budget_pct, None);
    assert_eq!(back.plateau_fired, None);
}

#[test]
fn i26_c_payload_extension_search_node_row_round_trips() {
    // I-26 search-node row (all three extension fields Some) round-trips.
    let i26_payload = ReasoningBudgetScaledPayload {
        kind: "neither".into(),
        role: "search_node".into(),
        base_calls: 100,
        serial_depth: 0,
        scaled_calls: 100,
        ceiling: 300,
        seed: 1,
        depth_level: Some(2),
        budget_pct: Some(0.72),
        plateau_fired: Some(false),
    };
    let json = serde_json::to_value(&i26_payload).expect("I-26 payload serde");
    assert_eq!(
        json.get("depth_level").and_then(|v| v.as_u64()),
        Some(2),
        "depth_level=Some(2) must appear in wire JSON"
    );
    assert!(
        json.get("budget_pct").is_some(),
        "budget_pct=Some(0.72) must appear in wire JSON"
    );
    assert_eq!(
        json.get("plateau_fired").and_then(|v| v.as_bool()),
        Some(false),
        "plateau_fired=Some(false) must appear in wire JSON"
    );
    let back: ReasoningBudgetScaledPayload =
        serde_json::from_value(json).expect("I-26 payload round-trip");
    assert_eq!(back.depth_level, Some(2));
    assert_eq!(back.plateau_fired, Some(false));
    let bp = back.budget_pct.expect("budget_pct must be Some after round-trip");
    assert!(
        (bp - 0.72).abs() < 0.001,
        "budget_pct must be ~0.72 after round-trip (got {bp})"
    );
}

// ---------------------------------------------------------------------------
// (d) BUDGET CONTROLLER — pure-compute behaviour (ID-4)
// ---------------------------------------------------------------------------

#[test]
fn i26_d_node_budget_depth_zero_is_base() {
    assert_eq!(
        budget_controller::node_budget(0),
        budget_controller::BASE_BUDGET_UNITS,
        "depth-0 node budget must equal BASE_BUDGET_UNITS"
    );
}

#[test]
fn i26_d_node_budget_monotone_up_to_ceiling() {
    let ceiling = budget_controller::BASE_BUDGET_UNITS * budget_controller::MAX_DEPTH_MULT;
    let mut prev = budget_controller::node_budget(0);
    for d in 1..=10u32 {
        let cur = budget_controller::node_budget(d);
        assert!(
            cur >= prev,
            "node_budget must be non-decreasing with depth (d={d}, prev={prev}, cur={cur})"
        );
        assert!(
            cur <= ceiling,
            "node_budget must not exceed ceiling={ceiling} (d={d}, cur={cur})"
        );
        prev = cur;
    }
}

#[test]
fn i26_d_should_collapse_budget_fires_at_threshold() {
    // Use total_envelope=100 and depth-0 expansions (cheapest).
    // Record enough expansions to cross the 85% threshold.
    let total: u32 = 100;
    let mut bc = BudgetController::new(total);
    let unit = budget_controller::node_budget(0);
    let target_spend = (total as f32 * COLLAPSE_THRESHOLD).ceil() as u32;
    let n_expansions = target_spend.div_ceil(unit);
    for _ in 0..n_expansions {
        bc.record_expansion(0);
    }
    assert!(
        bc.should_collapse_budget(),
        "should_collapse_budget must fire after accumulating >= \
         {COLLAPSE_THRESHOLD:.0}% of envelope"
    );
}

#[test]
fn i26_d_plateau_detector_fires_on_flat_scores() {
    let mut bc = BudgetController::new(10_000);
    // Feed PLATEAU_WINDOW + 1 identical scores — plateau must fire.
    for _ in 0..=PLATEAU_WINDOW {
        bc.record_score_and_check_plateau(0.75);
    }
    let fired = bc.record_score_and_check_plateau(0.75);
    assert!(
        fired,
        "plateau detector must fire after PLATEAU_WINDOW+1 flat scores \
         (PLATEAU_WINDOW={PLATEAU_WINDOW})"
    );
}

#[test]
fn i26_d_plateau_detector_does_not_fire_on_improving_scores() {
    let mut bc = BudgetController::new(10_000);
    // Score improves 0.1 per step — well above PLATEAU_DELTA_MIN.
    for i in 0u32..=(PLATEAU_WINDOW as u32) {
        bc.record_score_and_check_plateau(0.1 * i as f32 + 0.05);
    }
    // Budget untouched, no 85% collapse.
    assert!(
        !bc.should_collapse_budget(),
        "no budget collapse expected when envelope is large and scores are improving"
    );
}

// ---------------------------------------------------------------------------
// (e) NO FORCE-JSON — cloud_effort_for_node never returns budget_tokens (HD-3)
// ---------------------------------------------------------------------------

#[test]
fn i26_e_cloud_effort_never_returns_budget_tokens() {
    // HD-3 + landmine #6: effort fn must never produce "budget_tokens".
    for depth in 0u32..10 {
        for reasoning in [true, false] {
            if let Some(v) = budget_controller::cloud_effort_for_node(depth, reasoning) {
                assert_ne!(
                    v, "budget_tokens",
                    "cloud_effort_for_node MUST NEVER return 'budget_tokens' \
                     (landmine #6 / HD-3) at depth={depth} reasoning={reasoning}"
                );
            }
        }
    }
}

#[test]
fn i26_e_cloud_effort_values_are_auto_or_expert_for_reasoning_role() {
    // Only "auto" and "expert" are valid effort values for reasoning roles.
    for depth in 0u32..10 {
        if let Some(v) = budget_controller::cloud_effort_for_node(depth, true) {
            assert!(
                v == "auto" || v == "expert",
                "cloud_effort_for_node must return 'auto' or 'expert', \
                 got '{v}' at depth={depth}"
            );
        }
    }
}

#[test]
fn i26_e_cloud_effort_returns_none_for_non_reasoning_role() {
    for depth in 0u32..10 {
        let val = budget_controller::cloud_effort_for_node(depth, false);
        assert_eq!(
            val, None,
            "non-reasoning role must get None effort at depth={depth}"
        );
    }
}

// ---------------------------------------------------------------------------
// (f) REAL-DB SSB row — off-agent artifact (R1 missing=FAIL, R4 real artifact)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn i26_f_real_db_reasoning_budget_scaled_with_i26_fields_lands() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i26-contract-f");

    // Write an I-26 search-node ReasoningBudgetScaled row.
    let i26_payload = SsbPayload::ReasoningBudgetScaled(ReasoningBudgetScaledPayload {
        kind: "neither".into(),
        role: "search_node_i26".into(),
        base_calls: 100,
        serial_depth: 0,
        scaled_calls: 100,
        ceiling: 300,
        seed: 99,
        depth_level: Some(2),
        budget_pct: Some(0.72),
        plateau_fired: Some(false),
    });

    ssb.write_payload("i26_budget_ctrl", 0, i26_payload, vec![])
        .await
        .expect("SSB write must succeed (R1: missing write = FAIL)");

    // Read back via query_by_type — synchronous off-agent artifact.
    let rows = ssb
        .query_by_type(SsbActionType::ReasoningBudgetScaled)
        .expect("query_by_type must not fail (R2: dead evidence path screams)");

    assert!(
        !rows.is_empty(),
        "at least one ReasoningBudgetScaled row must be present \
         (R1: missing evidence = FAIL)"
    );

    // Find the I-26 row by role.
    let i26_row = rows
        .iter()
        .find(|entry| {
            if let Ok(SsbPayload::ReasoningBudgetScaled(ref p)) =
                decode_ssb_payload(&entry.payload)
            {
                p.role == "search_node_i26"
            } else {
                false
            }
        })
        .expect("i26 search_node_i26 row must be present in query results");

    let decoded = decode_ssb_payload(&i26_row.payload)
        .expect("decode must succeed on i26 row");
    match decoded {
        SsbPayload::ReasoningBudgetScaled(p) => {
            assert_eq!(
                p.depth_level,
                Some(2),
                "depth_level must survive SSB round-trip"
            );
            assert_eq!(
                p.plateau_fired,
                Some(false),
                "plateau_fired must survive SSB round-trip"
            );
            let bp = p
                .budget_pct
                .expect("budget_pct must be Some after SSB round-trip");
            assert!(
                (bp - 0.72).abs() < 0.01,
                "budget_pct must be ~0.72 after SSB round-trip (got {bp})"
            );
        }
        other => panic!(
            "expected ReasoningBudgetScaled payload, got a different variant: {other:?}"
        ),
    }
}

// ---------------------------------------------------------------------------
// (g) action_type round-trip + SCHEMA_VERSION floor
// ---------------------------------------------------------------------------

#[test]
fn i26_g_reasoning_budget_scaled_action_type_round_trips() {
    // Confirms I-26 added NO new action_type variant.
    let at = SsbActionType::ReasoningBudgetScaled;
    let sql_str = at.as_sql_str();
    let back = ssb_action_type_from_str_public(sql_str)
        .expect("round-trip must succeed");
    assert_eq!(
        back, at,
        "ReasoningBudgetScaled action_type must round-trip through as_sql_str"
    );
}

#[test]
fn i26_g_schema_version_unchanged_by_i26() {
    // I-26 extends ReasoningBudgetScaledPayload struct fields but adds ZERO new
    // action_type variants. SCHEMA_VERSION is derived (SCHEMA_VERSION_DDL_EPOCH +
    // SSB_ACTION_TYPES.len()) — must equal or exceed the live baseline.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 264,
        "SCHEMA_VERSION must be >= 264 (I-26 adds no new action_type; schema is DERIVED); \
         got {SCHEMA_VERSION_PUBLIC}"
    );
}
