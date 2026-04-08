//! Self-discovery: violation pattern detection, deduplication, and auto-proposal.
//!
//! When the auditor repeatedly finds the same violation pattern, this module
//! automatically proposes a new rule to prevent future occurrences. Maintains
//! a circular trust boundary: auditor-sourced proposed rules cannot
//! auto-promote themselves.

use std::time::UNIX_EPOCH;

use blake2::{Blake2s256, Digest};
use rusqlite::{params, Connection, OptionalExtension};

use elai_core::clock::ClockProvider;

use crate::error::RulesDbError;
use crate::predicate::PredicateError;
use crate::rule_engine::RuleEngine;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum occurrences before auto-proposing a rule.
const DEFAULT_PROPOSAL_THRESHOLD: u32 = 3;

/// Maximum value for any single score axis.
const MAX_SCORE: u8 = 5;

/// Default rule tier for auto-proposed rules (Tier 3 = lowest).
const DEFAULT_RULE_TIER: u8 = 3;

/// Default severity for auto-proposed rules.
const DEFAULT_SEVERITY: &str = "WARN";

/// Conservative default for pipeline count (per-pattern tracking not yet implemented).
const INITIAL_PIPELINES: u32 = 1;

/// Conservative default for false-positive rate (no FP data at discovery time).
const INITIAL_FPR: f64 = 0.0;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single violation occurrence reported by the auditor.
#[derive(Debug, Clone)]
pub struct ViolationRecord {
    pub pattern_text: String,
    pub example_code: String,
    pub domain: Option<String>,
}

/// Configuration for the self-discovery mechanism.
#[derive(Debug, Clone)]
pub struct SelfDiscoveryConfig {
    pub proposal_threshold: u32,
}

impl Default for SelfDiscoveryConfig {
    fn default() -> Self {
        Self {
            proposal_threshold: DEFAULT_PROPOSAL_THRESHOLD,
        }
    }
}

/// Outcome of recording a violation pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordOutcome {
    /// First time this pattern was seen.
    New { pattern_id: String },
    /// Pattern seen before but below proposal threshold.
    Incremented {
        pattern_id: String,
        occurrences: u32,
    },
    /// Threshold reached; a rule was proposed (or attempted).
    Proposed {
        pattern_id: String,
        rule_id: String,
    },
}

/// Errors from the self-discovery subsystem.
#[derive(Debug, thiserror::Error)]
pub enum SelfDiscoveryError {
    #[error("rules database error: {0}")]
    Db(#[from] RulesDbError),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("predicate compilation failed: {0}")]
    InvalidPredicate(#[from] PredicateError),

    #[error("pattern not found: {pattern_id}")]
    PatternNotFound { pattern_id: String },

    #[error("empty pattern after normalization")]
    EmptyPattern,
}

// ---------------------------------------------------------------------------
// Pure functions
// ---------------------------------------------------------------------------

/// Normalize a violation pattern for deduplication.
///
/// Strips inline comments (`// ...` and `/* ... */`), collapses consecutive
/// whitespace to single space, trims leading/trailing whitespace, and removes
/// null bytes.
pub fn normalize_pattern(text: &str) -> String {
    let without_nulls: String = text.chars().filter(|c| *c != '\0').collect();

    // Strip single-line comments (// ... to end of line)
    let mut result = String::with_capacity(without_nulls.len());
    let mut chars = without_nulls.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '/' {
            if chars.peek() == Some(&'/') {
                // Skip until newline
                for c in chars.by_ref() {
                    if c == '\n' {
                        result.push(' ');
                        break;
                    }
                }
            } else if chars.peek() == Some(&'*') {
                // Skip block comment
                chars.next(); // consume '*'
                let mut prev = ' ';
                for c in chars.by_ref() {
                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                }
                result.push(' ');
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    // Collapse whitespace and trim
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_was_space = true; // trim leading
    for ch in result.chars() {
        if ch.is_whitespace() {
            if !prev_was_space {
                collapsed.push(' ');
                prev_was_space = true;
            }
        } else {
            collapsed.push(ch);
            prev_was_space = false;
        }
    }

    // Trim trailing space
    if collapsed.ends_with(' ') {
        collapsed.pop();
    }

    collapsed
}

/// Compute the Blake2s256 hash of a normalized pattern string.
pub fn hash_pattern(normalized: &str) -> [u8; 32] {
    let mut hasher = Blake2s256::new();
    hasher.update(normalized.as_bytes());
    hasher.finalize().into()
}

/// Score a violation pattern across three axes: clarity, frequency, precision.
///
/// Each axis is clamped to [0, MAX_SCORE]. Composite score = sum of all three
/// (max = 15).
///
/// - clarity: inversely proportional to false_positive_rate
/// - frequency: log2-scaled occurrence count
/// - precision: scaled by pipeline observation breadth
pub fn score_violation(
    occurrences: u32,
    pipelines_observed: u32,
    false_positive_rate: f64,
) -> (u8, u8, u8) {
    // Frequency: log2(occurrences), clamped to [0, 5]
    let frequency = if occurrences == 0 {
        0u8
    } else {
        let raw = (occurrences as f64).log2().round() as u8;
        raw.min(MAX_SCORE)
    };

    // Clarity: inverse of FPR, scaled to [0, 5]
    // FPR 0.0 -> 5, FPR 1.0 -> 0
    let clarity = if false_positive_rate <= 0.0 {
        MAX_SCORE
    } else if false_positive_rate >= 1.0 {
        0u8
    } else {
        let raw = ((1.0 - false_positive_rate) * f64::from(MAX_SCORE)).round() as u8;
        raw.min(MAX_SCORE)
    };

    // Precision: pipeline breadth scaled to [0, 5]
    let precision = if pipelines_observed == 0 {
        0u8
    } else {
        (pipelines_observed.min(u32::from(MAX_SCORE)) as u8).min(MAX_SCORE)
    };

    (clarity, frequency, precision)
}

// ---------------------------------------------------------------------------
// Database operations
// ---------------------------------------------------------------------------

/// Extract epoch seconds from a ClockProvider.
fn epoch_secs(clock: &impl ClockProvider) -> i64 {
    clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Generate a pattern_id from hash prefix and epoch.
fn make_pattern_id(hash: &[u8; 32], epoch: i64) -> String {
    format!(
        "vp-{:02x}{:02x}{:02x}{:02x}-{}",
        hash[0], hash[1], hash[2], hash[3], epoch
    )
}

/// Generate a rule_id from hash prefix and epoch.
fn make_rule_id(hash: &[u8; 32], epoch: i64) -> String {
    format!(
        "ar-{:02x}{:02x}{:02x}{:02x}-{}",
        hash[0], hash[1], hash[2], hash[3], epoch
    )
}

/// Generate a contract test_id from rule_id and index.
fn make_test_id(rule_id: &str, idx: u32) -> String {
    format!("{}-ct{}", rule_id, idx)
}

/// Record a violation pattern. Deduplicates by blake2s hash.
///
/// Returns the outcome: New, Incremented, or Proposed (if threshold met).
///
/// All database mutations are wrapped in a SAVEPOINT so that a failure in
/// `try_propose_rule` rolls back the occurrence increment (preventing the
/// race where occurrences is committed but proposed_rule stays NULL).
///
/// Scores are computed and persisted on every insert/update using
/// conservative defaults (pipelines=1, fpr=0.0) since per-pattern pipeline
/// tracking is not yet implemented.
pub fn record_violation(
    conn: &mut Connection,
    record: &ViolationRecord,
    engine: &dyn RuleEngine,
    clock: &impl ClockProvider,
    config: &SelfDiscoveryConfig,
) -> Result<RecordOutcome, SelfDiscoveryError> {
    let normalized = normalize_pattern(&record.pattern_text);
    if normalized.is_empty() {
        return Err(SelfDiscoveryError::EmptyPattern);
    }

    let hash = hash_pattern(&normalized);
    let now = epoch_secs(clock);
    let hash_bytes: &[u8] = &hash;

    // Wrap all DB mutations in a savepoint for atomicity.
    let sp = conn.savepoint()?;

    // Check if this pattern already exists.
    let existing: Option<(String, u32)> = sp
        .query_row(
            "SELECT pattern_id, occurrences FROM violation_patterns WHERE pattern_hash = ?1",
            params![hash_bytes],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    let outcome = match existing {
        None => {
            // New pattern — insert with initial scores.
            let pattern_id = make_pattern_id(&hash, now);
            let initial_occurrences: u32 = 1;
            let (clarity, frequency, precision) =
                score_violation(initial_occurrences, INITIAL_PIPELINES, INITIAL_FPR);
            sp.execute(
                "INSERT INTO violation_patterns \
                 (pattern_id, pattern_hash, occurrences, first_seen, last_seen, \
                  domain, example_code, score_clarity, score_frequency, score_precision) \
                 VALUES (?1, ?2, 1, ?3, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    &pattern_id,
                    hash_bytes,
                    now,
                    &record.domain,
                    &record.example_code,
                    clarity,
                    frequency,
                    precision,
                ],
            )?;
            RecordOutcome::New { pattern_id }
        }
        Some((pattern_id, current_occurrences)) => {
            let new_occurrences = current_occurrences + 1;
            let (clarity, frequency, precision) =
                score_violation(new_occurrences, INITIAL_PIPELINES, INITIAL_FPR);
            sp.execute(
                "UPDATE violation_patterns \
                 SET occurrences = ?1, last_seen = ?2, \
                     score_clarity = ?3, score_frequency = ?4, score_precision = ?5 \
                 WHERE pattern_hash = ?6",
                params![new_occurrences, now, clarity, frequency, precision, hash_bytes],
            )?;

            if new_occurrences >= config.proposal_threshold {
                // Check if already proposed.
                let already_proposed: Option<String> = sp
                    .query_row(
                        "SELECT proposed_rule FROM violation_patterns WHERE pattern_id = ?1",
                        params![&pattern_id],
                        |row| row.get(0),
                    )
                    .optional()?
                    .flatten();

                if already_proposed.is_some() {
                    RecordOutcome::Incremented {
                        pattern_id,
                        occurrences: new_occurrences,
                    }
                } else {
                    match try_propose_rule(&sp, &pattern_id, engine, clock)? {
                        Some(rule_id) => RecordOutcome::Proposed {
                            pattern_id,
                            rule_id,
                        },
                        None => RecordOutcome::Incremented {
                            pattern_id,
                            occurrences: new_occurrences,
                        },
                    }
                }
            } else {
                RecordOutcome::Incremented {
                    pattern_id,
                    occurrences: new_occurrences,
                }
            }
        }
    };

    sp.commit()?;
    Ok(outcome)
}

/// Attempt to propose a rule from a violation pattern.
///
/// Generates a GrepPattern predicate from the example_code, validates it via
/// `compile_predicate`, and inserts into `rules` with source=AUDITOR,
/// status=PROPOSED. If the generated predicate fails compilation, returns
/// `Ok(None)` (intentional discard — invalid predicates must never be inserted).
pub fn try_propose_rule(
    conn: &Connection,
    pattern_id: &str,
    engine: &dyn RuleEngine,
    clock: &impl ClockProvider,
) -> Result<Option<String>, SelfDiscoveryError> {
    // Fetch the violation pattern record.
    let (example_code, domain): (String, Option<String>) = conn
        .query_row(
            "SELECT example_code, domain FROM violation_patterns WHERE pattern_id = ?1",
            params![pattern_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| SelfDiscoveryError::PatternNotFound {
            pattern_id: pattern_id.to_string(),
        })?;

    // Generate a grep_pattern predicate JSON from the example code.
    // Serde tag is "snake_case" per predicate.rs, fields: pattern, scope, context_lines.
    // Use serde_json to properly escape the pattern string for JSON embedding.
    let escaped_code = regex::escape(&example_code);
    let pattern_json_value = serde_json::json!({
        "type": "grep_pattern",
        "pattern": escaped_code,
        "scope": null,
        "context_lines": 0
    });
    let predicate_json = pattern_json_value.to_string();

    // Validate the predicate via compile_predicate.
    // If it fails, this is an intentional discard: invalid predicates are never inserted.
    if let Err(_pred_err) = engine.compile_predicate(&predicate_json) {
        return Ok(None);
    }

    let now = epoch_secs(clock);
    let hash = hash_pattern(&normalize_pattern(&example_code));
    let rule_id = make_rule_id(&hash, now);
    let rule_domain = domain.as_deref().unwrap_or("general");

    // Insert the proposed rule.
    conn.execute(
        "INSERT INTO rules \
         (rule_id, domain, tier, status, title, description, predicate, \
          severity, mutable, created_at, source) \
         VALUES (?1, ?2, ?3, 'PROPOSED', ?4, ?5, ?6, ?7, 1, ?8, 'AUDITOR')",
        params![
            &rule_id,
            rule_domain,
            DEFAULT_RULE_TIER,
            format!("Auto-proposed: {}", &example_code.chars().take(60).collect::<String>()),
            "Automatically proposed from repeated violation pattern",
            &predicate_json,
            DEFAULT_SEVERITY,
            now,
        ],
    )?;

    // Insert a BLOCK contract test for the example code.
    let test_id = make_test_id(&rule_id, 0);
    conn.execute(
        "INSERT INTO contract_tests \
         (test_id, rule_id, test_type, input_code, expected, created_at, source) \
         VALUES (?1, ?2, 'BLOCK', ?3, 'BLOCK', ?4, 'GENERATED')",
        params![&test_id, &rule_id, &example_code, now],
    )?;

    // Update violation_patterns with the proposed rule reference.
    conn.execute(
        "UPDATE violation_patterns SET proposed_rule = ?1 WHERE pattern_id = ?2",
        params![&rule_id, pattern_id],
    )?;

    Ok(Some(rule_id))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use elai_core::clock::SeededClock;

    use crate::predicate::{
        AstError, AstEvaluator, AstMatch, DkgError, DkgQuerier, ReachabilityMatch,
        ReachabilityParams,
    };
    use crate::rule_engine::DefaultRuleEngine;
    use crate::schema::open_rules_db;

    fn test_clock() -> SeededClock {
        SeededClock::new()
    }

    fn mem_db() -> Connection {
        open_rules_db(Path::new(":memory:"), &test_clock()).unwrap()
    }

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

    fn make_engine() -> DefaultRuleEngine {
        DefaultRuleEngine::new(Box::new(StubAstEvaluator), Box::new(StubDkgQuerier))
    }

    // -- normalize_pattern tests --

    #[test]
    fn normalize_trims_whitespace() {
        assert_eq!(normalize_pattern("  hello world  "), "hello world");
    }

    #[test]
    fn normalize_collapses_internal_whitespace() {
        assert_eq!(normalize_pattern("a   b\t\tc"), "a b c");
    }

    #[test]
    fn normalize_strips_line_comments() {
        assert_eq!(
            normalize_pattern("code // comment\nmore"),
            "code more"
        );
    }

    #[test]
    fn normalize_strips_block_comments() {
        assert_eq!(
            normalize_pattern("before /* block */ after"),
            "before after"
        );
    }

    #[test]
    fn normalize_removes_null_bytes() {
        assert_eq!(normalize_pattern("ab\0cd"), "abcd");
    }

    #[test]
    fn normalize_empty_after_strip_is_empty() {
        assert_eq!(normalize_pattern("// only a comment\n"), "");
    }

    #[test]
    fn normalize_preserves_division_operator() {
        assert_eq!(normalize_pattern("a / b"), "a / b");
    }

    // -- hash_pattern tests --

    #[test]
    fn hash_idempotent() {
        let h1 = hash_pattern("same text");
        let h2 = hash_pattern("same text");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_distinct_for_different_input() {
        let h1 = hash_pattern("text one");
        let h2 = hash_pattern("text two");
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_returns_32_bytes() {
        let h = hash_pattern("anything");
        assert_eq!(h.len(), 32);
    }

    // -- score_violation tests --

    #[test]
    fn score_zero_occurrences() {
        let (clarity, frequency, precision) = score_violation(0, 0, 0.0);
        assert_eq!(frequency, 0);
        assert_eq!(precision, 0);
        assert_eq!(clarity, MAX_SCORE); // FPR 0.0 = max clarity
    }

    #[test]
    fn score_clamped_to_max() {
        let (clarity, frequency, precision) = score_violation(10000, 100, 0.0);
        assert!(clarity <= MAX_SCORE);
        assert!(frequency <= MAX_SCORE);
        assert!(precision <= MAX_SCORE);
    }

    #[test]
    fn score_high_fpr_zero_clarity() {
        let (clarity, _, _) = score_violation(10, 5, 1.0);
        assert_eq!(clarity, 0);
    }

    #[test]
    fn score_composite_max_15() {
        let (c, f, p) = score_violation(1000, 10, 0.0);
        let composite = u16::from(c) + u16::from(f) + u16::from(p);
        assert!(composite <= 15);
    }

    #[test]
    fn score_all_axes_within_bounds() {
        for occ in [0, 1, 3, 10, 100, 1000] {
            for pipes in [0, 1, 3, 10] {
                for fpr_x10 in [0, 2, 5, 8, 10] {
                    let fpr = f64::from(fpr_x10) / 10.0;
                    let (c, f, p) = score_violation(occ, pipes, fpr);
                    assert!(c <= MAX_SCORE, "clarity {} > {} for occ={occ}", c, MAX_SCORE);
                    assert!(f <= MAX_SCORE, "frequency {} > {} for occ={occ}", f, MAX_SCORE);
                    assert!(p <= MAX_SCORE, "precision {} > {} for occ={occ}", p, MAX_SCORE);
                }
            }
        }
    }

    // -- record_violation tests --

    #[test]
    fn record_first_violation_returns_new() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        let record = ViolationRecord {
            pattern_text: "unsafe block without justification".to_string(),
            example_code: "unsafe { ptr::read(p) }".to_string(),
            domain: Some("safety".to_string()),
        };

        let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        assert!(matches!(result, RecordOutcome::New { .. }));
    }

    #[test]
    fn record_second_violation_returns_incremented() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        let record = ViolationRecord {
            pattern_text: "unused import".to_string(),
            example_code: "use std::io;".to_string(),
            domain: Some("style".to_string()),
        };

        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        match result {
            RecordOutcome::Incremented { occurrences, .. } => assert_eq!(occurrences, 2),
            other => panic!("expected Incremented, got {:?}", other),
        }
    }

    #[test]
    fn record_third_violation_proposes_rule() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default(); // threshold = 3

        let record = ViolationRecord {
            pattern_text: "hardcoded password".to_string(),
            example_code: "password = \"secret123\"".to_string(),
            domain: Some("security".to_string()),
        };

        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        assert!(matches!(result, RecordOutcome::Proposed { .. }));
    }

    #[test]
    fn record_empty_pattern_returns_error() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        let record = ViolationRecord {
            pattern_text: "  // only a comment\n ".to_string(),
            example_code: "x".to_string(),
            domain: None,
        };

        let err = record_violation(&mut conn, &record, &engine, &clock, &config);
        assert!(matches!(err, Err(SelfDiscoveryError::EmptyPattern)));
    }

    #[test]
    fn record_no_duplicate_rows_for_same_hash() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig { proposal_threshold: 100 };

        let record = ViolationRecord {
            pattern_text: "duplicate test".to_string(),
            example_code: "x = 1".to_string(),
            domain: None,
        };

        for _ in 0..5 {
            let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        }

        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM violation_patterns",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "must be exactly 1 row, not duplicated");
    }

    #[test]
    fn record_fourth_violation_after_proposal_stays_incremented() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        let record = ViolationRecord {
            pattern_text: "already proposed".to_string(),
            example_code: "bad_code()".to_string(),
            domain: None,
        };

        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let r3 = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        assert!(matches!(r3, RecordOutcome::Proposed { .. }));

        // Fourth should NOT re-propose.
        let r4 = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        assert!(matches!(r4, RecordOutcome::Incremented { .. }));
    }

    // -- try_propose_rule tests --

    #[test]
    fn propose_rule_inserts_with_auditor_source() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        let record = ViolationRecord {
            pattern_text: "test auditor source".to_string(),
            example_code: "eval(input())".to_string(),
            domain: Some("security".to_string()),
        };

        // Record 3 times to trigger proposal.
        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

        if let RecordOutcome::Proposed { rule_id, .. } = result {
            let (source, status): (String, String) = conn
                .query_row(
                    "SELECT source, status FROM rules WHERE rule_id = ?1",
                    params![&rule_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(source, "AUDITOR");
            assert_eq!(status, "PROPOSED");
        } else {
            panic!("expected Proposed, got {:?}", result);
        }
    }

    #[test]
    fn propose_rule_creates_contract_test() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        let record = ViolationRecord {
            pattern_text: "contract test creation".to_string(),
            example_code: "os.system(cmd)".to_string(),
            domain: None,
        };

        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

        if let RecordOutcome::Proposed { rule_id, .. } = result {
            let (test_type, source, expected): (String, String, String) = conn
                .query_row(
                    "SELECT test_type, source, expected FROM contract_tests WHERE rule_id = ?1",
                    params![&rule_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .unwrap();
            assert_eq!(test_type, "BLOCK");
            assert_eq!(source, "GENERATED");
            assert_eq!(expected, "BLOCK");
        } else {
            panic!("expected Proposed, got {:?}", result);
        }
    }

    #[test]
    fn propose_rule_not_found_returns_error() {
        let conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();

        let result = try_propose_rule(&conn, "nonexistent-id", &engine, &clock);
        assert!(matches!(
            result,
            Err(SelfDiscoveryError::PatternNotFound { .. })
        ));
    }

    // -- whitespace-invariant hashing (adversarial) --

    #[test]
    fn same_violation_different_whitespace_same_hash() {
        let h1 = hash_pattern(&normalize_pattern("  unsafe   block  "));
        let h2 = hash_pattern(&normalize_pattern("unsafe block"));
        assert_eq!(h1, h2);
    }

    #[test]
    fn same_violation_with_comments_same_hash() {
        let h1 = hash_pattern(&normalize_pattern("code // comment\nmore"));
        let h2 = hash_pattern(&normalize_pattern("code more"));
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_violations_different_hashes() {
        let h1 = hash_pattern(&normalize_pattern("unsafe block without justification"));
        let h2 = hash_pattern(&normalize_pattern("unsafe pointer dereference without check"));
        assert_ne!(h1, h2);
    }

    // -- SelfDiscoveryConfig --

    #[test]
    fn default_config_threshold_is_3() {
        let config = SelfDiscoveryConfig::default();
        assert_eq!(config.proposal_threshold, DEFAULT_PROPOSAL_THRESHOLD);
        assert_eq!(config.proposal_threshold, 3);
    }
}
