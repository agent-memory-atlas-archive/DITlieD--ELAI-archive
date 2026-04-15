//! Self-discovery: violation pattern detection, deduplication, and auto-proposal.
//!
//! When the auditor repeatedly finds the same violation pattern, this module
//! automatically proposes a new rule to prevent future occurrences. Maintains
//! a circular trust boundary: auditor-sourced proposed rules cannot
//! auto-promote themselves.

use std::time::UNIX_EPOCH;

use blake3;
use rusqlite::{params, Connection, OptionalExtension};

use elai_core::clock::ClockProvider;
use elai_core::FailureCategory;

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

/// Minimum occurrences before a group of patterns qualifies for a category proposal.
const CATEGORY_PROPOSAL_THRESHOLD: u32 = 3;

/// Minimum confidence (0-100) required to store a category proposal.
const MIN_CATEGORY_CONFIDENCE: u8 = 40;

/// Maximum number of category proposals returned per scan.
const MAX_CATEGORY_PROPOSALS_PER_SCAN: usize = 10;

/// Predicate type for symbol-relationship patterns (imports, type deps).
const PREDICATE_GRAPH_REACHABILITY: &str = "GRAPH_REACHABILITY";

/// Predicate type for simple text/regex patterns.
const PREDICATE_GREP_PATTERN: &str = "GREP_PATTERN";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single violation occurrence reported by the auditor.
///
/// When `replacement_name` is `Some(...)`, self-discovery proposes a
/// `VERSION_GATE` predicate instead of `GREP_PATTERN`. This enables
/// the anti-staleness pipeline (see anti-staleness-enforcement.txt 1.2).
#[derive(Debug, Clone)]
pub struct ViolationRecord {
    pub pattern_text: String,
    pub example_code: String,
    pub domain: Option<String>,
    /// If the auditor identified a replacement, set this to enable VERSION_GATE
    /// auto-proposal. When None, a GREP_PATTERN is proposed instead (existing behavior).
    pub replacement_name: Option<String>,
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

/// A proposed failure taxonomy category, generated when recurring patterns
/// do not map cleanly to existing `FailureCategory` variants.
///
/// Proposals are stored in the `failure_category_proposals` table and require
/// human review before any enum modification. This keeps the blast radius
/// of taxonomy changes under control.
#[derive(Debug, Clone)]
pub struct FailureCategoryProposal {
    /// Unique proposal identifier (fcp-{hash_prefix}-{epoch}).
    pub proposal_id: String,
    /// Suggested category name (snake_case, suitable for a future enum variant).
    pub suggested_name: String,
    /// Human-readable description of what this category covers.
    pub description: String,
    /// Pattern IDs from violation_patterns that evidence the gap.
    pub evidence_pattern_ids: Vec<String>,
    /// Suggested predicate type for root-cause localization.
    /// One of GREP_PATTERN or GRAPH_REACHABILITY.
    pub suggested_predicate_type: String,
    /// Fix template with {placeholders} for Layer 1 resolution.
    pub fix_template: String,
    /// Closest existing category (for triage context).
    pub closest_existing: FailureCategory,
    /// Confidence score (0-100).
    pub confidence: u8,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
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

    #[error("serialization error: {0}")]
    Serialization(String),
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

/// Compute the blake3 hash of a normalized pattern string.
pub fn hash_pattern(normalized: &str) -> [u8; 32] {
    *blake3::hash(normalized.as_bytes()).as_bytes()
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
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
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

/// Map a rule domain to the most likely package ecosystem.
///
/// Used by self-discovery to populate VERSION_GATE ecosystem field when
/// the auditor provides a replacement_name but no explicit ecosystem.
fn domain_to_ecosystem(domain: Option<&str>) -> &'static str {
    match domain {
        Some("python") | Some("data_science") => "pip",
        Some("frontend") | Some("web") => "npm",
        Some("java") | Some("android") => "maven",
        Some("dotnet") | Some("csharp") => "nuget",
        Some("ruby") => "bundler",
        Some("php") => "composer",
        Some("go") | Some("golang") => "go",
        // Default to cargo for Rust-centric project.
        _ => "cargo",
    }
}

/// Record a violation pattern. Deduplicates by blake3 hash.
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
                  domain, example_code, replacement_name, \
                  score_clarity, score_frequency, score_precision) \
                 VALUES (?1, ?2, 1, ?3, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    &pattern_id,
                    hash_bytes,
                    now,
                    &record.domain,
                    &record.example_code,
                    &record.replacement_name,
                    clarity,
                    frequency,
                    precision,
                ],
            )?;
            RecordOutcome::New { pattern_id }
        }
        Some((pattern_id, current_occurrences)) => {
            let new_occurrences = current_occurrences.saturating_add(1);
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
    // Fetch the violation pattern record including replacement_name.
    let (example_code, domain, replacement_name): (String, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT example_code, domain, replacement_name \
             FROM violation_patterns WHERE pattern_id = ?1",
            params![pattern_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| SelfDiscoveryError::PatternNotFound {
            pattern_id: pattern_id.to_string(),
        })?;

    // Generate predicate JSON: VERSION_GATE when replacement_name is available,
    // GREP_PATTERN otherwise (preserves existing behavior per WARN-4 resolution).
    let predicate_json = match &replacement_name {
        Some(repl) => {
            // Infer ecosystem from domain: "engineering" -> "cargo" (Rust-centric project).
            // Fallback to "cargo" if domain doesn't map to a known ecosystem.
            let ecosystem = domain_to_ecosystem(domain.as_deref());
            serde_json::json!({
                "type": "version_gate",
                "ecosystem": ecosystem,
                "deprecated_name": example_code,
                "replacement_name": repl,
                "severity": DEFAULT_SEVERITY,
            })
            .to_string()
        }
        None => {
            let escaped_code = regex::escape(&example_code);
            serde_json::json!({
                "type": "grep_pattern",
                "pattern": escaped_code,
                "scope": null,
                "context_lines": 0
            })
            .to_string()
        }
    };

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
// Failure taxonomy self-discovery (A-34)
// ---------------------------------------------------------------------------

/// Keywords that suggest a symbol-relationship pattern, warranting
/// GRAPH_REACHABILITY for root-cause localization rather than GREP_PATTERN.
/// Keywords that suggest a symbol-relationship pattern, warranting
/// GRAPH_REACHABILITY for root-cause localization rather than GREP_PATTERN.
///
/// Each entry is a (keyword, require_word_boundary) pair. When require_word_boundary
/// is true, the keyword must appear at the start of the string or after whitespace.
const SYMBOL_RELATIONSHIP_KEYWORDS: &[(&str, bool)] = &[
    ("import", false),
    ("use ", true),       // "use " at word boundary, not inside "reuse"
    ("mod ", true),
    ("pub use", false),
    ("extern crate", false),
    ("impl ", true),
    ("trait ", true),
    ("derive", false),
    ("::new(", false),
    ("::from(", false),
];

/// Generate a proposal_id from a domain+hash and epoch.
fn make_proposal_id(domain: &str, epoch: i64) -> String {
    let hash = hash_pattern(domain);
    format!(
        "fcp-{:02x}{:02x}{:02x}{:02x}-{}",
        hash[0], hash[1], hash[2], hash[3], epoch
    )
}

/// Select the closest existing FailureCategory for a domain string.
///
/// Uses simple keyword matching against domain names. Falls back to Unknown.
fn closest_category_for_domain(domain: &str) -> FailureCategory {
    let d = domain.to_lowercase();
    if d.contains("compile") || d.contains("build") {
        FailureCategory::CompileError
    } else if d.contains("test") {
        FailureCategory::TestFailure
    } else if d.contains("type") {
        FailureCategory::TypeMismatch
    } else if d.contains("import") {
        FailureCategory::MissingImport
    } else if d.contains("borrow") {
        FailureCategory::BorrowCheck
    } else if d.contains("lifetime") {
        FailureCategory::LifetimeError
    } else if d.contains("lazy") || d.contains("empty") {
        FailureCategory::LazyCode
    } else if d.contains("panic") {
        FailureCategory::RuntimePanic
    } else if d.contains("logic") {
        FailureCategory::LogicError
    } else if d.contains("timeout") || d.contains("time") {
        FailureCategory::Timeout
    } else {
        FailureCategory::Unknown
    }
}

/// Determine whether a pattern's example code suggests symbol-relationship errors
/// (warranting GRAPH_REACHABILITY) or simple text patterns (GREP_PATTERN).
fn select_predicate_type(example_code: &str) -> &'static str {
    let lower = example_code.to_lowercase();
    for &(keyword, require_boundary) in SYMBOL_RELATIONSHIP_KEYWORDS {
        if require_boundary {
            // Keyword must appear at start of string or after whitespace.
            if lower.starts_with(keyword)
                || lower.contains(&format!(" {}", keyword))
                || lower.contains(&format!("\n{}", keyword))
                || lower.contains(&format!("\t{}", keyword))
            {
                return PREDICATE_GRAPH_REACHABILITY;
            }
        } else if lower.contains(keyword) {
            return PREDICATE_GRAPH_REACHABILITY;
        }
    }
    PREDICATE_GREP_PATTERN
}

/// Generate a fix template from a domain and example code.
///
/// Produces a template with {placeholders} suitable for Layer 1 resolution.
fn generate_fix_template(domain: &str, example_code: &str) -> String {
    format!(
        "// Fix for {domain} category violation\n\
         // Original: {example}\n\
         // Replace with: {{replacement}}\n\
         // Rationale: {{rationale}}",
        domain = domain,
        example = example_code.chars().take(80).collect::<String>(),
    )
}

/// Convert an epoch timestamp to an ISO 8601 string (UTC, second precision).
///
/// Falls back to "1970-01-01T00:00:00Z" on overflow (should never happen in practice).
fn epoch_to_iso8601(epoch: i64) -> String {
    // Simple conversion without external crate. Acceptable precision for proposals.
    let secs = epoch.max(0) as u64;
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Compute year/month/day from days since epoch (civil calendar).
    let (year, month, day) = days_to_civil(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since Unix epoch to (year, month, day) civil date.
///
/// Algorithm from Howard Hinnant's `chrono`-compatible date library.
fn days_to_civil(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Scan violation_patterns for recurring patterns that suggest a missing
/// FailureTaxonomy category, and propose new categories.
///
/// Returns proposals for pattern groups meeting these criteria:
/// - occurrences >= CATEGORY_PROPOSAL_THRESHOLD
/// - no existing proposed_rule (not already handled by rule auto-proposal)
/// - grouped by domain (patterns in the same domain cluster together)
///
/// Proposals are stored in the `failure_category_proposals` table for human review.
/// Idempotent: re-running with the same data returns existing proposals.
pub fn detect_recurring_category_gap(
    conn: &Connection,
    clock: &impl ClockProvider,
    config: &SelfDiscoveryConfig,
) -> Result<Vec<FailureCategoryProposal>, SelfDiscoveryError> {
    let threshold = config.proposal_threshold.max(CATEGORY_PROPOSAL_THRESHOLD);

    // Query violation patterns that are frequent enough but haven't been
    // turned into rules yet, grouped by domain.
    let mut stmt = conn.prepare(
        "SELECT domain, GROUP_CONCAT(pattern_id, ','), \
                COUNT(*) as pattern_count, \
                SUM(occurrences) as total_occurrences, \
                GROUP_CONCAT(example_code, '|||') \
         FROM violation_patterns \
         WHERE occurrences >= ?1 \
           AND proposed_rule IS NULL \
         GROUP BY domain \
         HAVING pattern_count >= 1 \
         ORDER BY total_occurrences DESC \
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(
        params![threshold, MAX_CATEGORY_PROPOSALS_PER_SCAN as i64],
        |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u32>(2)?,
                row.get::<_, u32>(3)?,
                row.get::<_, String>(4)?,
            ))
        },
    )?;

    let now = epoch_secs(clock);
    let iso_now = epoch_to_iso8601(now);
    let mut proposals = Vec::new();

    for row_result in rows {
        let (domain_opt, pattern_ids_csv, _pattern_count, total_occurrences, examples_concat) =
            row_result?;

        let domain = domain_opt.as_deref().unwrap_or("general");
        let evidence_ids: Vec<String> = pattern_ids_csv
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // Check if a proposal already exists for this domain grouping.
        let existing_proposal: Option<String> = conn
            .query_row(
                "SELECT proposal_id FROM failure_category_proposals WHERE domain = ?1",
                params![domain],
                |row| row.get(0),
            )
            .optional()?;

        if existing_proposal.is_some() {
            continue; // Idempotent: skip already-proposed domains.
        }

        // Pick the first example for predicate type selection.
        let first_example = examples_concat
            .split("|||")
            .next()
            .unwrap_or("");

        let predicate_type = select_predicate_type(first_example);
        let closest = closest_category_for_domain(domain);
        let fix_template = generate_fix_template(domain, first_example);

        // Confidence based on total occurrences:
        // threshold -> 40, 2x threshold -> 60, 3x+ -> 80. Cap at 90.
        let ratio = total_occurrences as f64 / threshold as f64;
        let raw_confidence = (MIN_CATEGORY_CONFIDENCE as f64 + (ratio - 1.0) * 20.0).round();
        let confidence = (raw_confidence as u8).min(90).max(MIN_CATEGORY_CONFIDENCE);

        // Generate a suggested name from domain.
        let suggested_name = format!("{}_recurring", domain.replace(' ', "_").to_lowercase());

        let proposal_id = make_proposal_id(domain, now);
        let description = format!(
            "Recurring violation pattern in domain '{}' ({} total occurrences across {} patterns). \
             Closest existing category: {}.",
            domain,
            total_occurrences,
            evidence_ids.len(),
            closest.as_str(),
        );

        // Store the proposal.
        let evidence_json = serde_json::to_string(&evidence_ids)
            .map_err(|e| SelfDiscoveryError::Serialization(e.to_string()))?;

        conn.execute(
            "INSERT OR IGNORE INTO failure_category_proposals \
             (proposal_id, suggested_name, description, evidence_pattern_ids, \
              suggested_predicate_type, fix_template, closest_existing, confidence, \
              created_at, domain) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                &proposal_id,
                &suggested_name,
                &description,
                &evidence_json,
                predicate_type,
                &fix_template,
                closest.as_str(),
                confidence,
                &iso_now,
                domain,
            ],
        )?;

        proposals.push(FailureCategoryProposal {
            proposal_id,
            suggested_name,
            description,
            evidence_pattern_ids: evidence_ids,
            suggested_predicate_type: predicate_type.to_string(),
            fix_template,
            closest_existing: closest,
            confidence,
            created_at: iso_now.clone(),
        });
    }

    Ok(proposals)
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
            replacement_name: None,
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
            replacement_name: None,
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
            replacement_name: None,
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
            replacement_name: None,
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
            replacement_name: None,
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
            replacement_name: None,
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
            replacement_name: None,
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
            replacement_name: None,
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

    // -- VERSION_GATE proposal path --

    #[test]
    fn record_with_replacement_proposes_version_gate() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        let record = ViolationRecord {
            pattern_text: "blake2 used instead of blake3".to_string(),
            example_code: "blake2".to_string(),
            domain: Some("engineering".to_string()),
            replacement_name: Some("blake3".to_string()),
        };

        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let result = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

        let rule_id = match result {
            RecordOutcome::Proposed { rule_id, .. } => rule_id,
            other => panic!("expected Proposed, got {:?}", other),
        };

        // Verify the proposed rule uses version_gate predicate type.
        let predicate_json: String = conn
            .query_row(
                "SELECT predicate FROM rules WHERE rule_id = ?1",
                params![&rule_id],
                |row| row.get(0),
            )
            .unwrap();

        let pred: serde_json::Value = serde_json::from_str(&predicate_json).unwrap();
        assert_eq!(pred["type"], "version_gate", "must be version_gate, not grep_pattern");
        assert_eq!(pred["ecosystem"], "cargo", "engineering domain -> cargo ecosystem");
        assert_eq!(pred["deprecated_name"], "blake2");
        assert_eq!(pred["replacement_name"], "blake3");
    }

    #[test]
    fn record_without_replacement_proposes_grep_pattern() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        let record = ViolationRecord {
            pattern_text: "generic violation".to_string(),
            example_code: "bad_pattern()".to_string(),
            domain: Some("engineering".to_string()),
            replacement_name: None,
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
                params![&rule_id],
                |row| row.get(0),
            )
            .unwrap();

        let pred: serde_json::Value = serde_json::from_str(&predicate_json).unwrap();
        assert_eq!(pred["type"], "grep_pattern", "must be grep_pattern when no replacement");
    }

    #[test]
    fn replacement_name_stored_in_violation_patterns() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig { proposal_threshold: 100 };

        let record = ViolationRecord {
            pattern_text: "http.server usage".to_string(),
            example_code: "http.server".to_string(),
            domain: Some("python".to_string()),
            replacement_name: Some("fastapi+uvicorn".to_string()),
        };

        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

        let stored_replacement: Option<String> = conn
            .query_row(
                "SELECT replacement_name FROM violation_patterns LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored_replacement, Some("fastapi+uvicorn".to_string()));
    }

    // -- domain_to_ecosystem --

    #[test]
    fn domain_to_ecosystem_mappings() {
        assert_eq!(domain_to_ecosystem(Some("python")), "pip");
        assert_eq!(domain_to_ecosystem(Some("data_science")), "pip");
        assert_eq!(domain_to_ecosystem(Some("frontend")), "npm");
        assert_eq!(domain_to_ecosystem(Some("go")), "go");
        assert_eq!(domain_to_ecosystem(Some("engineering")), "cargo");
        assert_eq!(domain_to_ecosystem(None), "cargo");
        assert_eq!(domain_to_ecosystem(Some("unknown")), "cargo");
    }

    // -- SelfDiscoveryConfig --

    #[test]
    fn default_config_threshold_is_3() {
        let config = SelfDiscoveryConfig::default();
        assert_eq!(config.proposal_threshold, DEFAULT_PROPOSAL_THRESHOLD);
        assert_eq!(config.proposal_threshold, 3);
    }

    // -- A-34: failure taxonomy self-discovery tests --

    /// Helper: insert N violation patterns with given domain and occurrences.
    fn insert_violation_patterns(
        conn: &Connection,
        domain: &str,
        patterns: &[(&str, u32)],
    ) {
        for (idx, (example, occurrences)) in patterns.iter().enumerate() {
            let pattern_text = format!("{}-pattern-{}", domain, idx);
            let normalized = normalize_pattern(&pattern_text);
            let hash = hash_pattern(&normalized);
            let hash_bytes: &[u8] = &hash;
            let pattern_id = format!("vp-test-{}-{}", domain, idx);
            let (c, f, p) = score_violation(*occurrences, INITIAL_PIPELINES, INITIAL_FPR);
            conn.execute(
                "INSERT INTO violation_patterns \
                 (pattern_id, pattern_hash, occurrences, first_seen, last_seen, \
                  domain, example_code, score_clarity, score_frequency, score_precision) \
                 VALUES (?1, ?2, ?3, 0, 0, ?4, ?5, ?6, ?7, ?8)",
                params![
                    &pattern_id, hash_bytes, *occurrences,
                    domain, *example, c, f, p,
                ],
            )
            .unwrap_or_else(|e| panic!("insert failed for {}: {}", pattern_id, e));
        }
    }

    #[test]
    fn category_gap_no_proposal_below_threshold() {
        let conn = mem_db();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        // Insert patterns with only 2 occurrences (below threshold of 3).
        insert_violation_patterns(&conn, "low_count", &[
            ("let x = bad()", 2),
            ("let y = bad()", 2),
        ]);

        let proposals = detect_recurring_category_gap(&conn, &clock, &config).unwrap();
        assert!(
            proposals.is_empty(),
            "expected no proposals below threshold, got {}",
            proposals.len()
        );
    }

    #[test]
    fn category_gap_proposal_at_threshold() {
        let conn = mem_db();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        // Insert patterns with exactly 3 occurrences each (meets threshold).
        insert_violation_patterns(&conn, "security", &[
            ("eval(input())", 3),
            ("os.system(cmd)", 4),
        ]);

        let proposals = detect_recurring_category_gap(&conn, &clock, &config).unwrap();
        assert_eq!(proposals.len(), 1, "expected 1 proposal for 'security' domain");

        let p = &proposals[0];
        assert_eq!(p.suggested_name, "security_recurring");
        assert!(p.confidence >= MIN_CATEGORY_CONFIDENCE);
        assert!(!p.evidence_pattern_ids.is_empty());
        assert!(!p.fix_template.is_empty());
        assert!(!p.created_at.is_empty());
    }

    #[test]
    fn category_gap_duplicate_proposal_idempotent() {
        let conn = mem_db();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        insert_violation_patterns(&conn, "safety", &[
            ("unsafe { ptr::read(p) }", 5),
        ]);

        let first = detect_recurring_category_gap(&conn, &clock, &config).unwrap();
        assert_eq!(first.len(), 1);

        // Second call should return empty (proposal already exists).
        let second = detect_recurring_category_gap(&conn, &clock, &config).unwrap();
        assert!(
            second.is_empty(),
            "duplicate scan must be idempotent, got {} proposals",
            second.len()
        );

        // Verify only 1 row in the table.
        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM failure_category_proposals",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "must have exactly 1 proposal row");
    }

    #[test]
    fn category_gap_predicate_type_selection() {
        let conn = mem_db();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        // Insert patterns with symbol-relationship keywords (should get GRAPH_REACHABILITY).
        insert_violation_patterns(&conn, "imports", &[
            ("use std::collections::HashMap", 5),
            ("use crate::module::Type", 4),
        ]);

        let proposals = detect_recurring_category_gap(&conn, &clock, &config).unwrap();
        assert_eq!(proposals.len(), 1);
        assert_eq!(
            proposals[0].suggested_predicate_type, PREDICATE_GRAPH_REACHABILITY,
            "symbol-relationship pattern should use GRAPH_REACHABILITY"
        );
    }

    #[test]
    fn category_gap_grep_predicate_for_text_patterns() {
        let conn = mem_db();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        // Insert patterns without symbol keywords (should get GREP_PATTERN).
        insert_violation_patterns(&conn, "style", &[
            ("let x = 42", 3),
            ("let y = 99", 4),
        ]);

        let proposals = detect_recurring_category_gap(&conn, &clock, &config).unwrap();
        assert_eq!(proposals.len(), 1);
        assert_eq!(
            proposals[0].suggested_predicate_type, PREDICATE_GREP_PATTERN,
            "non-symbol pattern should use GREP_PATTERN"
        );
    }

    #[test]
    fn category_gap_skips_already_proposed_rules() {
        let mut conn = mem_db();
        let engine = make_engine();
        let clock = test_clock();
        let config = SelfDiscoveryConfig::default();

        // Record 3 violations to trigger rule proposal (existing mechanism).
        let record = ViolationRecord {
            pattern_text: "already handled".to_string(),
            example_code: "bad_code()".to_string(),
            domain: Some("handled".to_string()),
            replacement_name: None,
        };
        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let r = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        assert!(matches!(r, RecordOutcome::Proposed { .. }));

        // Category gap scan should find nothing (proposed_rule is set).
        let proposals = detect_recurring_category_gap(&conn, &clock, &config).unwrap();
        assert!(
            proposals.is_empty(),
            "patterns with proposed_rule should be skipped"
        );
    }

    #[test]
    fn category_gap_closest_existing_category() {
        assert_eq!(closest_category_for_domain("compile_errors"), FailureCategory::CompileError);
        assert_eq!(closest_category_for_domain("test_failures"), FailureCategory::TestFailure);
        assert_eq!(closest_category_for_domain("borrow_issues"), FailureCategory::BorrowCheck);
        assert_eq!(closest_category_for_domain("unknown_stuff"), FailureCategory::Unknown);
        assert_eq!(closest_category_for_domain("security"), FailureCategory::Unknown);
    }

    #[test]
    fn select_predicate_type_symbol_patterns() {
        assert_eq!(select_predicate_type("use std::io"), PREDICATE_GRAPH_REACHABILITY);
        assert_eq!(select_predicate_type("impl Foo for Bar"), PREDICATE_GRAPH_REACHABILITY);
        assert_eq!(select_predicate_type("pub use crate::x"), PREDICATE_GRAPH_REACHABILITY);
    }

    #[test]
    fn select_predicate_type_text_patterns() {
        assert_eq!(select_predicate_type("let x = 42"), PREDICATE_GREP_PATTERN);
        assert_eq!(select_predicate_type("password = secret"), PREDICATE_GREP_PATTERN);
    }

    #[test]
    fn epoch_to_iso8601_known_date() {
        // 2024-01-01T00:00:00Z = 1704067200
        let result = epoch_to_iso8601(1704067200);
        assert_eq!(result, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn epoch_to_iso8601_zero() {
        assert_eq!(epoch_to_iso8601(0), "1970-01-01T00:00:00Z");
    }
}
