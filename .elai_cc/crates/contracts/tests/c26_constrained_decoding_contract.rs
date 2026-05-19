//! C-26 contract: constrained-decoding + retry/futility primitives.
//!
//! Pins:
//!   * `DecodingMode` + `ConstrainedDecodingConfig` shape + defaults
//!     (providers crate, [`elai_providers::DecodingMode`]).
//!   * `RetryStrategy` + `BubbleUpPolicy.retry_strategy` (orchestrator
//!     crate, [`elai_orchestrator::RetryStrategy`]).
//!   * Futility same-error-3x escalation path in `NavigatorTree`.
//!   * Futility output-similarity persona switch (Hacker per C-10 D4).
//!   * SSB schema bump 57 → 58 with the two new variants visible via
//!     `SsbActionType::from_str` / `as_sql_str` round-trip.
//!   * Per-role defaults (designer=FreeText, executor=Grammar,
//!     critic=JsonSchema, auditor=JsonSchema, researcher=FreeText).
//!   * Benchmark fixture supplementary files present + valid.

use std::path::PathBuf;

use elai_orchestrator::pipeline_tree_navigator::{
    BubbleUpPolicy, FutilityVerdict, NavigatorTree, RetryStrategy, StagnationVerdict,
    TreeNavigatorState,
};
use elai_orchestrator::research_tree::{
    BabyStepGrounding, BabyStepId, BabyStepSpec, ConceptDecomposition, ConceptId, ConceptNode,
    EvidenceLevel, ExecutionMode, PhaseId, PhaseNode, StepId, StepNode,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SsbActionType, SCHEMA_VERSION_PUBLIC,
};
use elai_providers::{
    crane_adaptive_eligible, default_decoding_for, ConstrainedDecodingConfig, DecodingMode,
    RoleDecoding, PER_ROLE_DECODING_DEFAULTS,
};

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

fn single_baby_step_tree() -> NavigatorTree {
    let baby = BabyStepSpec {
        id: BabyStepId(1),
        task: "C-26 contract baby step".to_owned(),
        estimated_tokens: 500,
        grounding: BabyStepGrounding::default(),
        tdd: None,
        acceptance: None,
        preferred_mode: None,
        target_file: None,
        edit_region: None,
        intent_pairs: elai_orchestrator::IntentPairSet::default(),
        file_plan: None,
    };
    let step = StepNode {
        id: StepId(1),
        description: "C-26 contract step".to_owned(),
        baby_steps: vec![baby],
        evidence_level: EvidenceLevel::Anecdotal,
    };
    let phase = PhaseNode {
        id: PhaseId(1),
        name: "C-26 contract phase".to_owned(),
        steps: vec![step],
        depends_on: vec![],
        research: None,
        is_parallel: false,
    };
    let concept = ConceptNode {
        id: ConceptId(1),
        name: "C-26 contract concept".to_owned(),
        description: "C-26 contract concept description".to_owned(),
        phases: vec![phase],
        research: None,
    };
    let decomp = ConceptDecomposition {
        idea: "C-26 contract".to_owned(),
        concepts: vec![concept],
        mode: ExecutionMode::Autonomous,
        total_research_calls: 0,
    };
    NavigatorTree::new(decomp, BubbleUpPolicy::default())
}

// ---------------------------------------------------------------------------
// DecodingMode + ConstrainedDecodingConfig
// ---------------------------------------------------------------------------

#[test]
fn decoding_mode_default_is_free_text() {
    let cfg = ConstrainedDecodingConfig::default();
    assert!(matches!(cfg.mode, DecodingMode::FreeText), "default must be FreeText");
    assert_eq!(cfg.max_retries_on_invalid, 3);
    assert!(cfg.strict);
}

#[test]
fn json_schema_mode_serializes_schema_json_field() {
    let schema = r#"{"type":"object","required":["x"]}"#;
    let cfg = ConstrainedDecodingConfig::json_schema(schema);
    match &cfg.mode {
        DecodingMode::JsonSchema { schema_json } => {
            assert_eq!(schema_json, schema, "schema_json must round-trip exactly");
        }
        other => panic!("expected JsonSchema, got {other:?}"),
    }
    // Discriminant label stays stable.
    assert_eq!(cfg.mode.discriminant(), "json_schema");
}

#[test]
fn grammar_mode_crane_adaptive_true_eligible() {
    let m = DecodingMode::Grammar {
        ebnf_path: PathBuf::from("file_plan.ebnf"),
        crane_adaptive: true,
    };
    assert!(crane_adaptive_eligible(&m));
}

#[test]
fn grammar_mode_crane_adaptive_false_not_eligible() {
    let m = DecodingMode::Grammar {
        ebnf_path: PathBuf::from("json_response.ebnf"),
        crane_adaptive: false,
    };
    assert!(!crane_adaptive_eligible(&m));
}

#[test]
fn free_text_mode_not_crane_adaptive_eligible() {
    assert!(!crane_adaptive_eligible(&DecodingMode::FreeText));
    assert!(!crane_adaptive_eligible(&DecodingMode::Regex {
        pattern: "^.*$".to_owned()
    }));
    assert!(!crane_adaptive_eligible(&DecodingMode::JsonSchema {
        schema_json: "{}".to_owned()
    }));
}

// ---------------------------------------------------------------------------
// RetryStrategy
// ---------------------------------------------------------------------------

#[test]
fn retry_strategy_temperature_at_attempt_0_is_zero() {
    let s = RetryStrategy::default();
    assert_eq!(s.temperature_schedule, vec![0.0, 0.2, 0.5]);
    assert!((s.temperature_for_attempt(0) - 0.0).abs() < 1e-6);
}

#[test]
fn retry_strategy_temperature_clamps_at_last_entry() {
    let s = RetryStrategy::default();
    let t_99 = s.temperature_for_attempt(99);
    assert!(
        (t_99 - 0.5).abs() < 1e-6,
        "must clamp to last entry (0.5), got {t_99}"
    );
}

#[test]
fn retry_strategy_temperature_empty_schedule_does_not_panic() {
    let s = RetryStrategy {
        temperature_schedule: vec![],
        futility_same_error_threshold: 3,
        decompose_on_failure: false,
    };
    assert!((s.temperature_for_attempt(0) - 0.0).abs() < 1e-6);
    assert!((s.temperature_for_attempt(99) - 0.0).abs() < 1e-6);
}

#[test]
fn bubble_up_policy_default_includes_retry_strategy() {
    let p = BubbleUpPolicy::default();
    assert_eq!(p.retry_strategy.futility_same_error_threshold, 3);
    assert_eq!(p.retry_strategy.temperature_schedule, vec![0.0, 0.2, 0.5]);
    assert!(!p.retry_strategy.decompose_on_failure);
}

// ---------------------------------------------------------------------------
// Futility detection — same-error path
// ---------------------------------------------------------------------------

#[test]
fn futility_same_error_3x_triggers_escalation() {
    let tree = single_baby_step_tree();
    let mut state = TreeNavigatorState::new();
    let bs = BabyStepId(1);
    let reason = "constrained_decode_reject";

    let v1 = tree.record_failure_reason(&mut state, bs, reason);
    assert!(matches!(v1, FutilityVerdict::NotYet), "1st must be NotYet");
    let v2 = tree.record_failure_reason(&mut state, bs, reason);
    assert!(matches!(v2, FutilityVerdict::NotYet), "2nd must be NotYet");
    let v3 = tree.record_failure_reason(&mut state, bs, reason);
    match v3 {
        FutilityVerdict::Detected { consecutive_count } => {
            assert_eq!(consecutive_count, 3, "3x same error must trip threshold");
        }
        other => panic!("3rd must be Detected, got {other:?}"),
    }
}

#[test]
fn futility_different_errors_do_not_trigger() {
    let tree = single_baby_step_tree();
    let mut state = TreeNavigatorState::new();
    let bs = BabyStepId(1);

    let v1 = tree.record_failure_reason(&mut state, bs, "compile_error");
    let v2 = tree.record_failure_reason(&mut state, bs, "type_error");
    let v3 = tree.record_failure_reason(&mut state, bs, "compile_error");
    assert!(matches!(v1, FutilityVerdict::NotYet));
    assert!(matches!(v2, FutilityVerdict::NotYet));
    // Window holds [compile_error, type_error, compile_error] — not all equal.
    assert!(matches!(v3, FutilityVerdict::NotYet));
}

// ---------------------------------------------------------------------------
// Futility detection — output similarity path
// ---------------------------------------------------------------------------

#[test]
fn futility_output_similarity_90pct_triggers_persona_switch() {
    let tree = single_baby_step_tree();
    let mut state = TreeNavigatorState::new();
    let bs = BabyStepId(1);

    // First output: no prior output, must be NotYet.
    let v1 = tree.record_output(
        &mut state,
        bs,
        "alpha beta gamma delta epsilon zeta eta theta iota kappa",
        0.90,
    );
    assert!(matches!(v1, StagnationVerdict::NotYet { .. }), "1st must be NotYet");

    // Second output: identical tokens, Jaccard similarity = 1.0 > 0.90.
    let v2 = tree.record_output(
        &mut state,
        bs,
        "alpha beta gamma delta epsilon zeta eta theta iota kappa",
        0.90,
    );
    match v2 {
        StagnationVerdict::Stagnated { similarity, new_persona } => {
            assert!(similarity > 0.90, "similarity must exceed threshold");
            // DRIFT-5 / C-10 D4: NoDrift -> Hacker, NOT Researcher.
            assert_eq!(new_persona, "hacker", "persona must be Hacker per C-10 D4");
        }
        other => panic!("2nd identical output must be Stagnated, got {other:?}"),
    }
    assert_eq!(
        state.suggested_persona.as_deref(),
        Some("hacker"),
        "state.suggested_persona must also be set"
    );
}

#[test]
fn futility_output_similarity_low_does_not_trigger() {
    let tree = single_baby_step_tree();
    let mut state = TreeNavigatorState::new();
    let bs = BabyStepId(1);

    tree.record_output(&mut state, bs, "alpha beta gamma", 0.90);
    // Completely disjoint token set -> similarity = 0.0.
    let v = tree.record_output(&mut state, bs, "delta epsilon zeta", 0.90);
    assert!(matches!(v, StagnationVerdict::NotYet { .. }));
    assert!(state.suggested_persona.is_none());
}

// ---------------------------------------------------------------------------
// Per-role defaults
// ---------------------------------------------------------------------------

#[test]
fn per_role_defaults_executor_is_grammar() {
    let cfg = default_decoding_for(RoleDecoding::Executor);
    assert!(matches!(cfg.mode, DecodingMode::Grammar { .. }));
    assert_eq!(cfg.mode.discriminant(), "grammar");
}

#[test]
fn per_role_defaults_auditor_is_json_schema() {
    let cfg = default_decoding_for(RoleDecoding::Auditor);
    assert!(matches!(cfg.mode, DecodingMode::JsonSchema { .. }));
    assert_eq!(cfg.mode.discriminant(), "json_schema");
}

#[test]
fn per_role_defaults_critic_is_json_schema() {
    let cfg = default_decoding_for(RoleDecoding::Critic);
    assert_eq!(cfg.mode.discriminant(), "json_schema");
}

#[test]
fn per_role_defaults_designer_is_free_text() {
    let cfg = default_decoding_for(RoleDecoding::Designer);
    assert!(matches!(cfg.mode, DecodingMode::FreeText));
}

#[test]
fn per_role_defaults_researcher_is_free_text() {
    let cfg = default_decoding_for(RoleDecoding::Researcher);
    assert!(matches!(cfg.mode, DecodingMode::FreeText));
}

#[test]
fn per_role_table_has_five_entries() {
    assert_eq!(PER_ROLE_DECODING_DEFAULTS.len(), 5);
}

// ---------------------------------------------------------------------------
// SSB schema bump 57 -> 58 + 2 new variants
// ---------------------------------------------------------------------------

#[test]
fn ssb_schema_version_is_58() {
    // C-28: changed from exact `== 58` to floor `>= 58` so later
    // SCHEMA bumps (C-27 -> 59, C-28 -> 60, ...) do not break this
    // C-26 contract. See workflow/reasoning/C-28-decisions.md DECISION-D13.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 58,
        "C-26 atomic bump landed at 58 (post-bumps allowed). \
         Current: {SCHEMA_VERSION_PUBLIC}",
    );
}

#[test]
fn ssb_futility_detected_variant_round_trips() {
    let s = SsbActionType::FutilityDetected.as_sql_str();
    assert_eq!(s, "futility_detected");
    let parsed = ssb_action_type_from_str_public(s).expect("parse");
    assert_eq!(parsed, SsbActionType::FutilityDetected);
}

#[test]
fn ssb_futility_output_stagnation_variant_round_trips() {
    let s = SsbActionType::FutilityOutputStagnation.as_sql_str();
    assert_eq!(s, "futility_output_stagnation");
    let parsed = ssb_action_type_from_str_public(s).expect("parse");
    assert_eq!(parsed, SsbActionType::FutilityOutputStagnation);
}

// ---------------------------------------------------------------------------
// Benchmark fixture supplementary files
// ---------------------------------------------------------------------------

#[test]
fn benchmark_supplementary_templates_json_present_and_valid() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("benches")
        .join("model_capability")
        .join("fixtures")
        .join("templates.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("templates.json must exist at {path:?}: {e}"));
    let parsed: serde_json::Value =
        serde_json::from_str(&raw).expect("templates.json must be valid JSON");
    let templates = parsed
        .get("templates")
        .and_then(|v| v.as_array())
        .expect("templates.json must carry a `templates` array");
    assert!(
        !templates.is_empty(),
        "templates.json must declare at least one template"
    );
}

#[test]
fn benchmark_supplementary_padding_txt_present_and_nonempty() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("benches")
        .join("model_capability")
        .join("fixtures")
        .join("padding.txt");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("padding.txt must exist at {path:?}: {e}"));
    assert!(
        raw.lines().count() >= 50,
        "padding.txt must hold at least 50 lines (got {})",
        raw.lines().count()
    );
}

// ---------------------------------------------------------------------------
// EBNF grammar files
// ---------------------------------------------------------------------------

#[test]
fn ebnf_grammars_present_and_nonempty() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("orchestrator")
        .join("src")
        .join("grammars");
    for name in ["json_response.ebnf", "baby_step_spec.ebnf", "file_plan.ebnf"] {
        let path = base.join(name);
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("grammar file {name} must exist at {path:?}: {e}"));
        assert!(
            raw.lines().count() >= 5,
            "grammar file {name} must have at least 5 lines (got {})",
            raw.lines().count()
        );
    }
}
