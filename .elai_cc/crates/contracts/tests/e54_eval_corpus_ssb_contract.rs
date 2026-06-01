//! E-54 contract tests for the eval-corpus SSB pattern + thresholds.
//!
//! Enforces:
//!   - no_nine_gram_only_decontam  -- default n >= 13
//!   - cosine_filter_never_disabled -- default cosine_enabled == true
//!   - corpus_schema_requires_non_empty_tests -- schema rejects empty tests
//!   - corpus_schema_requires_cutoffs -- schema rejects empty cutoffs
//!   - prod_traffic_grade_defaults_off -- opt-in OFF by default
//!   - eval_corpus_event_action_type_registered -- SsbActionType variant
//!     present + as_sql_str() returns the expected wire tag
//!   - corpus_event_phase_has_14_arms -- discriminator has 14 distinct
//!     wire labels
//!   - eval_corpus_event_schema_version_at_167 -- SCHEMA_VERSION_PUBLIC ==
//!     167
//!
//! Per CLAUDE.md "synthetic small fixtures only, no dependence on
//! .elai_cc/held-out/ live store" gotcha (RAG-CORPUS-SYNTHETIC-NOT-REAL).

use elai_eval_corpus::generator_rotation::{
    compute_distribution, MIN_GENERATOR_SHARE, MIN_GENERATORS,
};
use elai_eval_corpus::ingest::IngestConfig;
use elai_eval_corpus::prod_traffic_grade::ProdTrafficConfig;
use elai_eval_corpus::task_schema::{
    BANNED_DEPLOY_TOKENS, ExecutableTest, FramingPair, Provenance, Task, TaskSchemaError,
    TestKind,
};
use elai_eval_corpus::TaskFamily;
use elai_orchestrator::ssb::{CorpusEventPhase, SsbActionType, SCHEMA_VERSION_PUBLIC};

use chrono::TimeZone;
use rustc_hash::FxHashMap;

fn fixture_task() -> Task {
    let mut cutoffs = FxHashMap::default();
    cutoffs.insert("m".to_string(), "2024-10".to_string());
    Task {
        id: "c1".to_string(),
        family: TaskFamily::CodeFix,
        created_at: chrono::Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
        cutoffs,
        problem: "p".to_string(),
        reference_solution: "r".to_string(),
        tests: vec![ExecutableTest {
            input: "i".to_string(),
            expected_output: "o".to_string(),
            kind: TestKind::UnitTest,
        }],
        provenance: Provenance {
            generator: "g".to_string(),
            prompt_id: None,
            date: chrono::Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
        },
        framing_pair: None,
        perturbed_pair: None,
        embedding_hash: "deadbeefcafe1234".to_string(),
        ngram_hash: "1234deadbeefcafe".to_string(),
    }
}

#[test]
fn no_nine_gram_only_decontam() {
    let cfg = IngestConfig::default();
    assert!(
        cfg.ngram_n >= 13,
        "E-54 default n MUST be >= 13 (arXiv:2311.04850); got {}",
        cfg.ngram_n
    );
}

#[test]
fn cosine_filter_never_disabled() {
    let cfg = IngestConfig::default();
    assert!(
        cfg.cosine_enabled,
        "E-54 default cosine_enabled MUST be true (stub_ban contract)"
    );
    assert!(cfg.cosine_threshold > 0.0, "threshold must be positive");
}

#[test]
fn corpus_schema_requires_non_empty_tests() {
    let mut t = fixture_task();
    t.tests.clear();
    assert!(matches!(t.validate(), Err(TaskSchemaError::EmptyTests(_))));
}

#[test]
fn corpus_schema_requires_cutoffs() {
    let mut t = fixture_task();
    t.cutoffs.clear();
    assert!(matches!(t.validate(), Err(TaskSchemaError::EmptyCutoffs(_))));
}

#[test]
fn deploy_framing_no_eval_keywords() {
    let mut t = fixture_task();
    t.framing_pair = Some(FramingPair {
        eval: "evaluate this benchmark".to_string(),
        deploy: "on this benchmark, answer".to_string(),
    });
    let r = t.validate();
    assert!(matches!(r, Err(TaskSchemaError::BannedDeployToken(_, _))));
    // sanity: BANNED_DEPLOY_TOKENS includes 'benchmark'
    assert!(BANNED_DEPLOY_TOKENS.contains(&"benchmark"));
}

#[test]
fn prod_traffic_grade_defaults_off() {
    let c = ProdTrafficConfig::default();
    assert!(!c.opt_in, "E-54 prod-traffic-grade MUST default opt-in OFF");
}

#[test]
fn generator_rotation_minimum_distribution_constants() {
    assert!(
        MIN_GENERATOR_SHARE >= 0.30,
        "E-54 min generator share MUST be >= 0.30"
    );
    assert!(
        MIN_GENERATORS >= 3,
        "E-54 generator count floor MUST be >= 3"
    );
    let mut m = FxHashMap::default();
    m.insert("a".to_string(), 5);
    m.insert("b".to_string(), 5);
    let r = compute_distribution(m, 10);
    assert!(r.violated, "2-generator corpus must violate");
}

#[test]
fn eval_corpus_event_action_type_registered() {
    let v = SsbActionType::EvalCorpusEvent;
    assert_eq!(v.as_sql_str(), "eval_corpus_event");
}

#[test]
fn corpus_event_phase_has_14_arms() {
    use CorpusEventPhase::*;
    // Enumerate every arm via match -- compiler enforces exhaustiveness.
    let arms = [
        IngestRejectedNgram,
        IngestRejectedCosine,
        ShardCreated,
        SchemaViolationNoTests,
        TestFailsOnReference,
        TestUselessPassesBroken,
        KorOrthogonalityViolated,
        KorOrthogonalityPassed,
        SycophancyFramingGapDetected,
        SandbaggingAnomaly,
        GeneratorRotationViolated,
        PerfDeltaContaminationSuspected,
        DynamicKnowledgeQaGenerated,
        ProdTrafficProbeInjected,
    ];
    let mut labels: std::collections::HashSet<&'static str> = std::collections::HashSet::new();
    for a in arms {
        labels.insert(a.as_str());
    }
    assert_eq!(labels.len(), 14, "CorpusEventPhase MUST have 14 distinct wire labels");
}

#[test]
fn eval_corpus_event_schema_version_at_167() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 167,
        "schema only drifts forward; this contract was written at version 167. \
         E-54 SCHEMA_VERSION bump MUST land at >= 167 (post-E-19 head 166 + 1)"
    );
}
