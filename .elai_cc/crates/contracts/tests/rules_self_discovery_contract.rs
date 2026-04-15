//! Contract tests for rules/self_discovery (Phase 3F.7).
//!
//! Validates: normalize_pattern determinism, hash_pattern idempotency,
//! record_violation lifecycle (New -> Incremented -> Proposed), UNIQUE
//! constraint handling, try_propose_rule predicate validation, AUDITOR
//! source enforcement, contract test auto-generation, circular trust
//! boundary, and score_violation axis clamping.

use std::path::Path;

use elai_core::clock::SeededClock;
use elai_rules::predicate::{
    AstError, AstEvaluator, AstMatch, DkgError, DkgQuerier, ReachabilityMatch,
    ReachabilityParams,
};
use elai_rules::rule_engine::DefaultRuleEngine;
use elai_rules::schema::open_rules_db;
use elai_rules::self_discovery::{
    hash_pattern, normalize_pattern, record_violation, score_violation, try_propose_rule,
    RecordOutcome, SelfDiscoveryConfig, SelfDiscoveryError, ViolationRecord,
};

use elai_rules::error::LifecycleError;
use elai_rules::lifecycle::{transition_to_candidate, transition_to_shadow, Approver};

// ---------------------------------------------------------------------------
// Stubs (minimal, contract-level)
// ---------------------------------------------------------------------------

struct StubAstEvaluator;

impl AstEvaluator for StubAstEvaluator {
    fn evaluate_query(
        &self,
        _language: &str,
        _source: &str,
        _query: &str,
    ) -> Result<Vec<AstMatch>, AstError> {
        Ok(vec![])
    }

    fn validate_query(&self, _language: &str, _query: &str) -> Result<(), AstError> {
        Ok(())
    }
}

struct StubDkgQuerier;

impl DkgQuerier for StubDkgQuerier {
    fn query_reachability(
        &self,
        _params: &ReachabilityParams,
    ) -> Result<Vec<ReachabilityMatch>, DkgError> {
        Ok(vec![])
    }

    fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_clock() -> SeededClock {
    SeededClock::new()
}

fn mem_db() -> rusqlite::Connection {
    open_rules_db(Path::new(":memory:"), &test_clock()).expect("in-memory DB should open")
}

fn make_engine() -> DefaultRuleEngine {
    DefaultRuleEngine::new(Box::new(StubAstEvaluator), Box::new(StubDkgQuerier))
}

fn default_config() -> SelfDiscoveryConfig {
    SelfDiscoveryConfig::default()
}

// ---------------------------------------------------------------------------
// normalize_pattern contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_normalize_whitespace_collapse() {
    assert_eq!(normalize_pattern("  a   b\t\tc  "), "a b c");
}

#[test]
fn contract_normalize_comment_stripping() {
    assert_eq!(
        normalize_pattern("code // comment\nmore /* block */ end"),
        "code more end"
    );
}

#[test]
fn contract_normalize_trim() {
    assert_eq!(normalize_pattern("  hello  "), "hello");
}

// ---------------------------------------------------------------------------
// hash_pattern contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_hash_idempotent() {
    let h1 = hash_pattern("same text");
    let h2 = hash_pattern("same text");
    assert_eq!(h1, h2, "same input must produce same hash");
}

#[test]
fn contract_hash_distinct() {
    let h1 = hash_pattern("input one");
    let h2 = hash_pattern("input two");
    assert_ne!(h1, h2, "different inputs must produce different hashes");
}

// ---------------------------------------------------------------------------
// record_violation contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_record_first_insert_new() {
    let mut conn = mem_db();
    let engine = make_engine();
    let clock = test_clock();
    let config = default_config();

    let record = ViolationRecord {
        pattern_text: "contract test pattern".to_string(),
        example_code: "let x = 42;".to_string(),
        domain: Some("test".to_string()),
        replacement_name: None,
    };

    let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    assert!(
        matches!(result, RecordOutcome::New { .. }),
        "first record must return New"
    );
}

#[test]
fn contract_record_second_insert_incremented() {
    let mut conn = mem_db();
    let engine = make_engine();
    let clock = test_clock();
    let config = default_config();

    let record = ViolationRecord {
        pattern_text: "duplicate pattern".to_string(),
        example_code: "bad()".to_string(),
        domain: None,
        replacement_name: None,
    };

    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

    match result {
        RecordOutcome::Incremented { occurrences, .. } => {
            assert_eq!(occurrences, 2, "second record must have occurrences=2");
        }
        other => panic!("expected Incremented, got {:?}", other),
    }
}

#[test]
fn contract_record_third_insert_proposed() {
    let mut conn = mem_db();
    let engine = make_engine();
    let clock = test_clock();
    let config = default_config(); // threshold = 3

    let record = ViolationRecord {
        pattern_text: "repeated violation".to_string(),
        example_code: "unsafe_op()".to_string(),
        domain: Some("safety".to_string()),
        replacement_name: None,
    };

    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

    assert!(
        matches!(result, RecordOutcome::Proposed { .. }),
        "third record must trigger proposal"
    );
}

#[test]
fn contract_record_unique_constraint_no_duplicates() {
    let mut conn = mem_db();
    let engine = make_engine();
    let clock = test_clock();
    let config = SelfDiscoveryConfig {
        proposal_threshold: 1000,
    };

    let record = ViolationRecord {
        pattern_text: "unique test".to_string(),
        example_code: "x".to_string(),
        domain: None,
        replacement_name: None,
    };

    for _ in 0..10 {
        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    }

    let count: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM violation_patterns",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(count, 1, "UNIQUE constraint must prevent duplicate rows");
}

// ---------------------------------------------------------------------------
// try_propose_rule contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_propose_valid_predicate_returns_some() {
    let conn = mem_db();
    let clock = test_clock();

    // Insert a violation pattern with example_code that will produce invalid regex.
    // regex::escape handles all inputs safely, so grep_pattern predicates
    // always compile. Verify the positive path here; unit tests cover the
    // rejection path via PatternNotFound.

    // Insert a violation pattern manually.
    let now = 1_700_000_000i64;
    let normalized = normalize_pattern("test pattern");
    let hash = hash_pattern(&normalized);
    let hash_bytes: &[u8] = &hash;
    let pattern_id = "vp-test-invalid";

    conn.execute(
        "INSERT INTO violation_patterns \
         (pattern_id, pattern_hash, occurrences, first_seen, last_seen, example_code) \
         VALUES (?1, ?2, 3, ?3, ?3, 'valid_code()')",
        rusqlite::params![pattern_id, hash_bytes, now],
    )
    .unwrap();

    let engine = make_engine();
    let result = try_propose_rule(&conn, pattern_id, &engine, &clock).unwrap();

    // With a valid engine, this should succeed (Some).
    assert!(result.is_some(), "valid predicate should produce a rule");

    // Verify no rules row exists for a non-existent pattern.
    let err = try_propose_rule(&conn, "nonexistent", &engine, &clock);
    assert!(
        matches!(err, Err(SelfDiscoveryError::PatternNotFound { .. })),
        "nonexistent pattern must return PatternNotFound"
    );
}

#[test]
fn contract_propose_valid_creates_auditor_proposed_rule() {
    let mut conn = mem_db();
    let engine = make_engine();
    let clock = test_clock();
    let config = default_config();

    let record = ViolationRecord {
        pattern_text: "auditor source contract".to_string(),
        example_code: "dangerous_call()".to_string(),
        domain: Some("security".to_string()),
        replacement_name: None,
    };

    // Trigger proposal.
    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

    let rule_id = match result {
        RecordOutcome::Proposed { rule_id, .. } => rule_id,
        other => panic!("expected Proposed, got {:?}", other),
    };

    // Verify rules row.
    let (source, status, domain, tier): (String, String, String, u8) = conn
        .query_row(
            "SELECT source, status, domain, tier FROM rules WHERE rule_id = ?1",
            rusqlite::params![&rule_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();

    assert_eq!(source, "AUDITOR", "source must be AUDITOR");
    assert_eq!(status, "PROPOSED", "status must be PROPOSED");
    assert_eq!(domain, "security", "domain must match violation record");
    assert_eq!(tier, 3, "auto-proposed rules must be Tier 3");
}

#[test]
fn contract_propose_generates_block_contract_test() {
    let mut conn = mem_db();
    let engine = make_engine();
    let clock = test_clock();
    let config = default_config();

    let record = ViolationRecord {
        pattern_text: "contract test generation".to_string(),
        example_code: "eval(user_input)".to_string(),
        domain: None,
        replacement_name: None,
    };

    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

    let rule_id = match result {
        RecordOutcome::Proposed { rule_id, .. } => rule_id,
        other => panic!("expected Proposed, got {:?}", other),
    };

    // Verify contract_tests row.
    let (test_type, expected, source, input_code): (String, String, String, String) = conn
        .query_row(
            "SELECT test_type, expected, source, input_code FROM contract_tests WHERE rule_id = ?1",
            rusqlite::params![&rule_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();

    assert_eq!(test_type, "BLOCK", "test_type must be BLOCK");
    assert_eq!(expected, "BLOCK", "expected must be BLOCK");
    assert_eq!(source, "GENERATED", "source must be GENERATED");
    assert_eq!(
        input_code, "eval(user_input)",
        "input_code must match example_code"
    );
}

// ---------------------------------------------------------------------------
// Circular trust boundary contract
// ---------------------------------------------------------------------------

#[test]
fn contract_circular_trust_blocks_auditor_self_promotion() {
    let mut conn = mem_db();
    let engine = make_engine();
    let clock = test_clock();
    let config = default_config();

    let record = ViolationRecord {
        pattern_text: "circular trust test".to_string(),
        example_code: "risky_fn()".to_string(),
        domain: Some("trust".to_string()),
        replacement_name: None,
    };

    // Trigger proposal.
    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

    let rule_id = match result {
        RecordOutcome::Proposed { rule_id, .. } => rule_id,
        other => panic!("expected Proposed, got {:?}", other),
    };

    // Insert enough contract tests to pass the MIN_CONTRACT_TESTS gate (3).
    for i in 0..3 {
        conn.execute(
            "INSERT INTO contract_tests \
             (test_id, rule_id, test_type, input_code, expected, created_at, source) \
             VALUES (?1, ?2, 'BLOCK', 'test_code', 'BLOCK', 1000, 'GENERATED')",
            rusqlite::params![format!("ct-extra-{}", i), &rule_id],
        )
        .unwrap();
    }

    // Transition PROPOSED -> CANDIDATE (this should succeed).
    transition_to_candidate(&conn, &rule_id, 4, true, &clock).unwrap();

    // Now try to transition CANDIDATE -> SHADOW using FindingSourceAuditor.
    // This MUST be rejected with CircularTrust error.
    let approver = Approver::FindingSourceAuditor {
        auditor_id: "auditor-1".to_string(),
    };
    let shadow_err = transition_to_shadow(&conn, &rule_id, &approver, &clock);

    assert!(
        matches!(shadow_err, Err(LifecycleError::CircularTrust)),
        "auditor-sourced rule must be blocked from self-promotion to SHADOW, got {:?}",
        shadow_err
    );
}

// ---------------------------------------------------------------------------
// VERSION_GATE auto-proposal contract
// ---------------------------------------------------------------------------

#[test]
fn contract_version_gate_proposed_on_replacement_name() {
    let mut conn = mem_db();
    let engine = make_engine();
    let clock = test_clock();
    let config = default_config(); // threshold = 3

    let record = ViolationRecord {
        pattern_text: "blake2 used instead of blake3".to_string(),
        example_code: "blake2".to_string(),
        domain: Some("engineering".to_string()),
        replacement_name: Some("blake3".into()),
    };

    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
    let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

    let rule_id = match result {
        RecordOutcome::Proposed { rule_id, .. } => rule_id,
        other => panic!("expected Proposed, got {:?}", other),
    };

    let predicate_json: String = conn
        .query_row(
            "SELECT predicate FROM rules WHERE rule_id = ?1",
            rusqlite::params![&rule_id],
            |row| row.get(0),
        )
        .unwrap();

    let pred: serde_json::Value = serde_json::from_str(&predicate_json).unwrap();
    assert_eq!(
        pred["type"], "version_gate",
        "replacement_name path must produce version_gate predicate, not grep_pattern"
    );
    assert_eq!(
        pred["replacement_name"], "blake3",
        "predicate must carry the replacement_name from ViolationRecord"
    );
}

// ---------------------------------------------------------------------------
// score_violation contracts
// ---------------------------------------------------------------------------

#[test]
fn contract_score_axes_clamped_0_to_5() {
    for occ in [0, 1, 5, 50, 500, 5000] {
        for pipes in [0, 1, 3, 10, 100] {
            for fpr_x10 in [0, 3, 5, 8, 10, 15] {
                let fpr = f64::from(fpr_x10) / 10.0;
                let (c, f, p) = score_violation(occ, pipes, fpr);
                assert!(c <= 5, "clarity {} > 5 for occ={occ}, fpr={fpr}", c);
                assert!(f <= 5, "frequency {} > 5 for occ={occ}", f);
                assert!(p <= 5, "precision {} > 5 for pipes={pipes}", p);
            }
        }
    }
}

#[test]
fn contract_score_composite_max_15() {
    let (c, f, p) = score_violation(10000, 100, 0.0);
    let composite = u16::from(c) + u16::from(f) + u16::from(p);
    assert!(
        composite <= 15,
        "composite {} > 15 (c={c}, f={f}, p={p})",
        composite
    );
}
