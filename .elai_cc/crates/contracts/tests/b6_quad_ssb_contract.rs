//! Tranche B-6 quad -- coordinated SSB contract tests for the four
//! plans landing in one orchestrate run: C-42 (McpAuthzEvent), C-47
//! (ToolCalibrationEvent Pattern-2 collapse), B-39 (ReflectBudgetSoftWarn
//! single-add), C-50 (ContractCodingEvent Pattern-2 collapse).
//!
//! Pins:
//!   * `SCHEMA_VERSION_PUBLIC` reflects the tranche bump (193 -> 194).
//!   * Each of the 4 new `SsbActionType` variants round-trips via
//!     `ssb_action_type_from_str_public`.
//!   * Each Kind discriminator's `as_str()` matches the wire labels
//!     declared in the SQL CHECK constraint + the golden lock file.
//!   * The 4 new env-gate names are stable string constants.
//!   * Helper `env_gate_truthy` returns `false` by default (wire-dark
//!     posture) and recognises the documented truthy synonyms.
//!
//! NO cargo test execution by the main LLM; structural shape mirrors
//! crates/contracts/tests/e40_symbol_graph_contract.rs +
//! e47_lora_ship_gate_contract.rs.

use elai_orchestrator::ssb::{
    env_gate_truthy, ContractCodingEventKind, McpAuthzEventKind,
    ToolCalibrationEventKind, ELAI_B39_SOFTWARN_ENABLE_ENV,
    ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV, ELAI_C47_CASE_BASE_ENABLE_ENV,
    ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV, MCP_FUZZ_CORPUS_DIR,
};
use elai_orchestrator::{
    ssb_action_type_from_str_public, SsbActionType, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// Contract -- SCHEMA_VERSION reflects the Tranche B-6 quad bump.
// ---------------------------------------------------------------------------

#[test]
fn contract_schema_version_is_194_after_b6_quad_tranche() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 194,
        "schema only drifts forward; this contract was written at version 194. \
         CONTRACT: SCHEMA_VERSION_PUBLIC must be >= 194 after the \
         Tranche B-6 quad bump (193 -> 194) covering C-42 McpAuthzEvent + \
         C-47 ToolCalibrationEvent + B-39 ReflectBudgetSoftWarn + C-50 \
         ContractCodingEvent. Update ssb.rs ledger \
         comments + action_type_lock.golden together on the next bump."
    );
}

// ---------------------------------------------------------------------------
// Contract -- 4 new SsbActionType variants round-trip via from_str.
// ---------------------------------------------------------------------------

#[test]
fn contract_mcp_authz_event_from_str_round_trip() {
    let actual =
        ssb_action_type_from_str_public("mcp_authz_event").expect("from_str");
    assert_eq!(
        actual,
        SsbActionType::McpAuthzEvent,
        "CONTRACT: from_str(\"mcp_authz_event\") must yield McpAuthzEvent"
    );
}

#[test]
fn contract_tool_calibration_event_from_str_round_trip() {
    let actual =
        ssb_action_type_from_str_public("tool_calibration_event").expect("from_str");
    assert_eq!(
        actual,
        SsbActionType::ToolCalibrationEvent,
        "CONTRACT: from_str(\"tool_calibration_event\") must yield ToolCalibrationEvent"
    );
}

#[test]
fn contract_reflect_budget_soft_warn_from_str_round_trip() {
    let actual =
        ssb_action_type_from_str_public("reflect_budget_soft_warn").expect("from_str");
    assert_eq!(
        actual,
        SsbActionType::ReflectBudgetSoftWarn,
        "CONTRACT: from_str(\"reflect_budget_soft_warn\") must yield ReflectBudgetSoftWarn"
    );
}

#[test]
fn contract_contract_coding_event_from_str_round_trip() {
    let actual =
        ssb_action_type_from_str_public("contract_coding_event").expect("from_str");
    assert_eq!(
        actual,
        SsbActionType::ContractCodingEvent,
        "CONTRACT: from_str(\"contract_coding_event\") must yield ContractCodingEvent"
    );
}

// ---------------------------------------------------------------------------
// Contract -- as_sql_str round-trip back through from_str (full cycle).
// ---------------------------------------------------------------------------

#[test]
fn contract_all_b6_quad_variants_as_sql_str_round_trip() {
    let cases = [
        (SsbActionType::McpAuthzEvent, "mcp_authz_event"),
        (SsbActionType::ToolCalibrationEvent, "tool_calibration_event"),
        (SsbActionType::ReflectBudgetSoftWarn, "reflect_budget_soft_warn"),
        (SsbActionType::ContractCodingEvent, "contract_coding_event"),
    ];
    for (variant, expected_wire) in &cases {
        assert_eq!(
            variant.as_sql_str(),
            *expected_wire,
            "CONTRACT: {:?}.as_sql_str() must equal {:?}",
            variant,
            expected_wire,
        );
        let parsed =
            ssb_action_type_from_str_public(expected_wire).expect("from_str");
        assert_eq!(
            parsed, *variant,
            "CONTRACT: from_str({:?}) must round-trip to {:?}",
            expected_wire, variant
        );
    }
}

// ---------------------------------------------------------------------------
// Contract -- Kind discriminators serde labels are stable wire strings.
// ---------------------------------------------------------------------------

#[test]
fn contract_mcp_authz_event_kind_labels() {
    let cases = [
        (McpAuthzEventKind::AuthzCheckPassed, "authz_check_passed"),
        (McpAuthzEventKind::AuthzCheckFailed, "authz_check_failed"),
        (McpAuthzEventKind::CorpusReplayed, "corpus_replayed"),
    ];
    for (kind, expected) in &cases {
        assert_eq!(
            kind.as_str(),
            *expected,
            "CONTRACT: McpAuthzEventKind::{:?}.as_str() must equal {:?}",
            kind,
            expected,
        );
    }
}

#[test]
fn contract_tool_calibration_event_kind_labels() {
    let cases = [
        (ToolCalibrationEventKind::HintEmitted, "hint_emitted"),
        (ToolCalibrationEventKind::DriftDetected, "drift_detected"),
        (ToolCalibrationEventKind::CallLogged, "call_logged"),
        (ToolCalibrationEventKind::CaseAdded, "case_added"),
        (ToolCalibrationEventKind::CaseSelected, "case_selected"),
        (
            ToolCalibrationEventKind::CalibrationApplied,
            "calibration_applied",
        ),
    ];
    for (kind, expected) in &cases {
        assert_eq!(
            kind.as_str(),
            *expected,
            "CONTRACT: ToolCalibrationEventKind::{:?}.as_str() must equal {:?}",
            kind,
            expected,
        );
    }
}

#[test]
fn contract_contract_coding_event_kind_labels() {
    let cases = [
        (ContractCodingEventKind::RetryAttempted, "retry_attempted"),
        (ContractCodingEventKind::RetryResolved, "retry_resolved"),
        (ContractCodingEventKind::RetryAbandoned, "retry_abandoned"),
    ];
    for (kind, expected) in &cases {
        assert_eq!(
            kind.as_str(),
            *expected,
            "CONTRACT: ContractCodingEventKind::{:?}.as_str() must equal {:?}",
            kind,
            expected,
        );
    }
}

// ---------------------------------------------------------------------------
// Contract -- the 4 env-gate names are stable constants matching docs.
// ---------------------------------------------------------------------------

#[test]
fn contract_b6_quad_env_gate_names_are_stable() {
    assert_eq!(
        ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV, "ELAI_C42_MCP_AUTHZ_EMIT_ENABLE",
        "CONTRACT: C-42 env-gate name must stay stable across releases"
    );
    assert_eq!(
        ELAI_C47_CASE_BASE_ENABLE_ENV, "ELAI_C47_CASE_BASE_ENABLE",
        "CONTRACT: C-47 env-gate name must stay stable across releases"
    );
    assert_eq!(
        ELAI_B39_SOFTWARN_ENABLE_ENV, "ELAI_B39_SOFTWARN_ENABLE",
        "CONTRACT: B-39 env-gate name must stay stable across releases"
    );
    assert_eq!(
        ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV, "ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE",
        "CONTRACT: C-50 env-gate name must stay stable across releases"
    );
}

#[test]
fn contract_b6_quad_default_off_posture() {
    // Wire-dark default: with no env var set, env_gate_truthy returns false.
    // The 4 gate names are sufficiently unique (uppercase + ELAI_ prefix)
    // that they will not be set in a normal test environment; defensive
    // remove_var still happens to avoid CI cross-contamination.
    for name in [
        ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV,
        ELAI_C47_CASE_BASE_ENABLE_ENV,
        ELAI_B39_SOFTWARN_ENABLE_ENV,
        ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV,
    ] {
        // SAFETY of env mutation in test: this test crate runs single-threaded
        // by default (no #[tokio::test] / no rayon); env API is process-wide.
        // Stick to remove_var (no write race window). std::env::set_var/remove_var
        // are marked unsafe on some recent toolchain pre-releases; we guard with
        // explicit unsafe in case the contract crate pulls a 2026-edition pin.
        // SAFETY: see comment above.
        // (The unsafe block compiles on both 2024 and 2026 editions.)
        #[allow(unused_unsafe)]
        unsafe {
            std::env::remove_var(name);
        }
        assert!(
            !env_gate_truthy(name),
            "CONTRACT: {} must be wire-dark by default (env_gate_truthy false)",
            name
        );
    }
}

#[test]
fn contract_env_gate_truthy_recognises_documented_synonyms() {
    let name = "ELAI_B6_QUAD_CONTRACT_TEST_GATE_TRUTHY_PROBE";
    let truthy = ["1", "true", "yes", "on", "TRUE", "Yes", "ON"];
    let falsey = ["0", "false", "no", "off", "", "maybe", "2"];
    for v in &truthy {
        // SAFETY: see contract_b6_quad_default_off_posture comment.
        #[allow(unused_unsafe)]
        unsafe {
            std::env::set_var(name, v);
        }
        assert!(
            env_gate_truthy(name),
            "CONTRACT: env_gate_truthy({:?}) must be true",
            v
        );
    }
    for v in &falsey {
        #[allow(unused_unsafe)]
        unsafe {
            std::env::set_var(name, v);
        }
        assert!(
            !env_gate_truthy(name),
            "CONTRACT: env_gate_truthy({:?}) must be false",
            v
        );
    }
    #[allow(unused_unsafe)]
    unsafe {
        std::env::remove_var(name);
    }
}

// ---------------------------------------------------------------------------
// Contract -- C-42 corpus directory constant matches the SHIPPABLE-NOW
// `.elai_cc/data/mcp_fuzz_corpus` location. The proc-macro + 200+ tuple
// generator are DEFERRED to C-42b.
// ---------------------------------------------------------------------------

#[test]
fn contract_mcp_fuzz_corpus_dir_constant_is_stable() {
    assert_eq!(
        MCP_FUZZ_CORPUS_DIR, ".elai_cc/data/mcp_fuzz_corpus",
        "CONTRACT: MCP_FUZZ_CORPUS_DIR must point at the canonical \
         workspace-relative corpus dir (proc-macro + populated corpus \
         deferred to C-42b)."
    );
}
