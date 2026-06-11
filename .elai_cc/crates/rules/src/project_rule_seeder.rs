// ProjectRuleSeeder: auto-seed rules from detected project conventions.
// Spec: docs/specs/guide/project-intelligence-design.txt section 6.
//
// Takes a ProjectProfile, generates SHADOW rules for detected conventions.
// Rules enter the existing Phase 3F lifecycle: after 5 successful enforcements
// they auto-promote to ACTIVE. 3+ false positives -> auto-retire.
//
// Trust level: MEDIUM (source=SEED). Can be overridden by user rules (HUMAN).

use rusqlite::{params, Connection};
use tracing::debug;

use elai_core::clock::ClockProvider;
use elai_core::project_profile::{
    Convention, ErrorHandlingProfile, ErrorPattern, LintProfile, NamingConventions, ProjectProfile,
    TestLocationPattern, TestProfile,
};

use crate::error::RulesDbError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Phase mask: EXECUTING + AUDITING only (bits 3,4).
const EXECUTING_AUDITING_MASK: u32 = (1 << 3) | (1 << 4);

/// Default tier for project-seeded rules.
const SEEDED_TIER: u8 = 2;

/// Source tag for project-seeded rules.
const SEEDED_SOURCE: &str = "SEED";

/// Initial status: SHADOW (logged but not blocking until promoted).
///
/// # 3F.shadow DECISION (SHADOW-not-ACTIVE seed behavior)
///
/// Every rule this seeder inserts -- convention-derived (`seed_rules`) AND the
/// immutable security/finance gates (`seed_security_rules`, `seed_deflation_
/// rules`, ...) -- enters at status=SHADOW, NOT status=ACTIVE. This is a
/// deliberate trust decision, not an oversight:
///
///   * A SHADOW rule is OBSERVED but does NOT block. It accrues counterfactual
///     evidence (true/false positives) over real evaluations; only after
///     `AUTO_PROMOTE_SUCCESSES` (5) true positives WITH a non-None
///     counterfactual_causal_rate does `check_and_promote_seeded_rules`
///     transition it to ACTIVE (the enforcing state). 3 false positives
///     auto-retire it. Auto-derived rules are MEDIUM trust (source=SEED) and
///     must EARN enforcement; shipping them ACTIVE would let a noisy regex
///     block real work on day one.
///
///   * SECURITY/FINANCE seeds also ship SHADOW. They are mutable=0 (lifecycle-
///     immune: cannot be auto-retired or overridden), but they still start
///     SHADOW so a fresh project's first runs surface false positives before
///     the CWE/deflation gates begin hard-blocking. Their LOCKED thresholds are
///     enforced by dedicated Rust gates (deflation_gate / leakage_gate /
///     trading_null_gate) that evaluate their constants in-code and do NOT read
///     this rules.db row at decision time (the seed row is documentation +
///     the vibesec blake3 seed-integrity baseline), so the SHADOW status of the
///     ROW does not weaken those gates.
///
/// # 3J.WIRE.RULES SECURITY FINDING (no evaluation-bypass gap)
///
/// SHADOW does NOT create a "should-block-but-doesn't" bypass. The SOLE
/// production query that feeds the rule engine is
/// `lifecycle::fetch_all_active_rules`, whose SQL is
/// `... FROM rules WHERE status = 'ACTIVE' ...`. SHADOW rows are NEVER loaded
/// into the evaluation set, so a SHADOW rule is never in a position to block AND
/// is never EXPECTED to block -- there is no path where a SHADOW rule "should
/// have blocked but was silently skipped." The in-memory
/// `RuleEngine::get_active_rules` filters only by phase_mask + domain (it
/// assumes the caller already status-filtered via fetch_all_active_rules), so
/// the status gate lives at the DB boundary, fail-closed by construction.
/// VERDICT: NO GAP. (Verified base-phase-3 PART 2: rule_engine.rs:283 doc-SQL,
/// lifecycle.rs:317-341 production query, project_rule_seeder.rs status=SHADOW.)
const SEEDED_STATUS: &str = "SHADOW";

// ---------------------------------------------------------------------------
// SeedRule: internal representation before DB insertion
// ---------------------------------------------------------------------------

struct SeedRule {
    rule_id: String,
    domain: String,
    tier: u8,
    title: String,
    description: String,
    predicate_json: String,
    phase_mask: u32,
    severity: String,
}

// ---------------------------------------------------------------------------
// ProjectRuleSeeder
// ---------------------------------------------------------------------------

/// Generates project-specific seed rules from a [`ProjectProfile`].
///
/// Call [`seed_rules`] during rule engine initialization to populate the
/// rules database with convention-derived rules. Rules are inserted as
/// SHADOW with source=SEED, entering the existing lifecycle for promotion.
pub struct ProjectRuleSeeder;

impl ProjectRuleSeeder {
    /// Seed project-specific rules from the given profile into the rules DB.
    ///
    /// Skips rules whose `rule_id` already exists in the database (idempotent).
    /// Returns the number of newly inserted rules.
    pub fn seed_rules(
        conn: &Connection,
        profile: &ProjectProfile,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let mut rules = Vec::new();

        // Naming convention rules (spec section 6, row 1).
        Self::naming_rules(&profile.naming, &mut rules);

        // Error handling rules (spec section 6, row 2).
        Self::error_handling_rules(&profile.error_handling, &mut rules);

        // Test location rules (spec section 6, row 3).
        Self::test_location_rules(&profile.test_profile, &mut rules);

        // Lint tool rules (spec section 6, rows 4-5).
        Self::lint_rules(&profile.lint_tools, &mut rules);

        // Insert into DB, skipping duplicates.
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        let mut inserted = 0usize;
        for rule in &rules {
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
                params![&rule.rule_id],
                |row| row.get(0),
            )?;
            if exists {
                debug!(rule_id = %rule.rule_id, "project seed rule already exists, skipping");
                continue;
            }

            conn.execute(
                "INSERT INTO rules \
                 (rule_id, domain, tier, status, title, description, predicate, \
                  phase_mask, severity, mutable, created_at, source) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11)",
                params![
                    &rule.rule_id,
                    &rule.domain,
                    rule.tier,
                    SEEDED_STATUS,
                    &rule.title,
                    &rule.description,
                    &rule.predicate_json,
                    rule.phase_mask,
                    &rule.severity,
                    now,
                    SEEDED_SOURCE,
                ],
            )?;
            inserted += 1;
            debug!(rule_id = %rule.rule_id, "inserted project seed rule");
        }

        debug!(total_generated = rules.len(), inserted, "project rule seeding complete");
        Ok(inserted)
    }

    // -----------------------------------------------------------------------
    // Naming conventions -> rules (spec 3A + section 6 row 1)
    // -----------------------------------------------------------------------

    fn naming_rules(naming: &NamingConventions, out: &mut Vec<SeedRule>) {
        Self::naming_rule_for_category("functions", &naming.functions, out);
        Self::naming_rule_for_category("types", &naming.types, out);
        Self::naming_rule_for_category("constants", &naming.constants, out);
        Self::naming_rule_for_category("modules", &naming.modules, out);
    }

    fn naming_rule_for_category(
        category: &str,
        convention: &Convention,
        out: &mut Vec<SeedRule>,
    ) {
        let (style, severity) = match convention {
            Convention::Strong(s) => (s.as_str(), "WARN"),
            Convention::Moderate(s) => (s.as_str(), "INFO"),
            Convention::Mixed | Convention::Unknown => return,
        };

        let pattern = naming_regex_for_style(style);
        if pattern.is_empty() {
            return;
        }

        let predicate_json = serde_json::json!({
            "type": "grep_pattern",
            "pattern": pattern,
            "scope": null,
            "context_lines": 0
        })
        .to_string();

        out.push(SeedRule {
            rule_id: format!("proj.naming.{category}.{style}"),
            domain: "engineering".to_owned(),
            tier: SEEDED_TIER,
            title: format!("{category} should use {style}"),
            description: format!(
                "Project convention: {category} use {style} naming. \
                 Auto-detected from existing codebase."
            ),
            predicate_json,
            phase_mask: EXECUTING_AUDITING_MASK,
            severity: severity.to_owned(),
        });
    }

    // -----------------------------------------------------------------------
    // Error handling -> rules (spec 3B + section 6 row 2)
    // -----------------------------------------------------------------------

    fn error_handling_rules(
        eh: &ErrorHandlingProfile,
        out: &mut Vec<SeedRule>,
    ) {
        // Rule: if project uses thiserror, error types must derive thiserror::Error.
        if let Some(ref crate_name) = eh.error_crate {
            let pattern = match crate_name.as_str() {
                "thiserror" => r"#\[derive\(.*thiserror::Error",
                "anyhow" => r"anyhow::(Result|Error|bail|ensure)",
                _ => return,
            };

            let predicate_json = serde_json::json!({
                "type": "grep_pattern",
                "pattern": pattern,
                "scope": null,
                "context_lines": 0
            })
            .to_string();

            out.push(SeedRule {
                rule_id: format!("proj.error.crate.{crate_name}"),
                domain: "engineering".to_owned(),
                tier: SEEDED_TIER,
                title: format!("error handling should use {crate_name}"),
                description: format!(
                    "Project convention: error types use {crate_name}. \
                     Auto-detected from existing codebase."
                ),
                predicate_json,
                phase_mask: EXECUTING_AUDITING_MASK,
                severity: "WARN".to_owned(),
            });
        }

        // Rule: if Result-based, functions should return Result (not raw types).
        if eh.pattern == ErrorPattern::ResultBased {
            let predicate_json = serde_json::json!({
                "type": "grep_pattern",
                "pattern": r"-> Result<",
                "scope": null,
                "context_lines": 0
            })
            .to_string();

            out.push(SeedRule {
                rule_id: "proj.error.result_based".to_owned(),
                domain: "engineering".to_owned(),
                tier: SEEDED_TIER,
                title: "fallible functions should return Result".to_owned(),
                description: "Project convention: use Result<T, E> for fallible operations. \
                              Auto-detected from existing codebase."
                    .to_owned(),
                predicate_json,
                phase_mask: EXECUTING_AUDITING_MASK,
                severity: "INFO".to_owned(),
            });
        }
    }

    // -----------------------------------------------------------------------
    // Test location -> rules (spec 3D + section 6 row 3)
    // -----------------------------------------------------------------------

    fn test_location_rules(test_profile: &TestProfile, out: &mut Vec<SeedRule>) {
        match test_profile.location_pattern {
            TestLocationPattern::SameFile => {
                let predicate_json = serde_json::json!({
                    "type": "grep_pattern",
                    "pattern": r"#\[cfg\(test\)\]",
                    "scope": null,
                    "context_lines": 0
                })
                .to_string();

                out.push(SeedRule {
                    rule_id: "proj.test.same_file".to_owned(),
                    domain: "engineering".to_owned(),
                    tier: SEEDED_TIER,
                    title: "modules should include inline test block".to_owned(),
                    description: "Project convention: tests co-located in source files \
                                  via #[cfg(test)]. Auto-detected."
                        .to_owned(),
                    predicate_json,
                    phase_mask: EXECUTING_AUDITING_MASK,
                    severity: "WARN".to_owned(),
                });
            }
            TestLocationPattern::SeparateDirectory => {
                let predicate_json = serde_json::json!({
                    "type": "grep_pattern",
                    "pattern": r"tests/",
                    "scope": null,
                    "context_lines": 0
                })
                .to_string();

                out.push(SeedRule {
                    rule_id: "proj.test.separate_dir".to_owned(),
                    domain: "engineering".to_owned(),
                    tier: SEEDED_TIER,
                    title: "tests should be in separate tests/ directory".to_owned(),
                    description: "Project convention: tests in separate tests/ directory. \
                                  Auto-detected."
                        .to_owned(),
                    predicate_json,
                    phase_mask: EXECUTING_AUDITING_MASK,
                    severity: "INFO".to_owned(),
                });
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Lint tools -> rules (spec section 6 rows 4-5)
    // -----------------------------------------------------------------------

    fn lint_rules(lint_tools: &[LintProfile], out: &mut Vec<SeedRule>) {
        for lint in lint_tools {
            let tool_name = lint.tool.to_lowercase();
            match tool_name.as_str() {
                "clippy" => {
                    let predicate_json = serde_json::json!({
                        "type": "metric_gate",
                        "metric_name": "clippy_warnings",
                        "operator": "eq",
                        "threshold": 0.0,
                        "source": "tool_output"
                    })
                    .to_string();

                    out.push(SeedRule {
                        rule_id: "proj.lint.clippy".to_owned(),
                        domain: "engineering".to_owned(),
                        tier: 1,
                        title: "cargo clippy must pass with zero warnings".to_owned(),
                        description: "Project uses clippy. Auto-detected from clippy.toml."
                            .to_owned(),
                        predicate_json,
                        phase_mask: EXECUTING_AUDITING_MASK,
                        severity: "WARN".to_owned(),
                    });
                }
                "ruff" => {
                    let predicate_json = serde_json::json!({
                        "type": "metric_gate",
                        "metric_name": "ruff_violations",
                        "operator": "eq",
                        "threshold": 0.0,
                        "source": "tool_output"
                    })
                    .to_string();

                    out.push(SeedRule {
                        rule_id: "proj.lint.ruff".to_owned(),
                        domain: "engineering".to_owned(),
                        tier: 1,
                        title: "ruff check must pass with zero violations".to_owned(),
                        description: "Project uses ruff. Auto-detected from pyproject.toml."
                            .to_owned(),
                        predicate_json,
                        phase_mask: EXECUTING_AUDITING_MASK,
                        severity: "WARN".to_owned(),
                    });
                }
                "eslint" => {
                    let predicate_json = serde_json::json!({
                        "type": "metric_gate",
                        "metric_name": "eslint_errors",
                        "operator": "eq",
                        "threshold": 0.0,
                        "source": "tool_output"
                    })
                    .to_string();

                    out.push(SeedRule {
                        rule_id: "proj.lint.eslint".to_owned(),
                        domain: "engineering".to_owned(),
                        tier: 1,
                        title: "eslint must pass with zero errors".to_owned(),
                        description: "Project uses eslint. Auto-detected from config."
                            .to_owned(),
                        predicate_json,
                        phase_mask: EXECUTING_AUDITING_MASK,
                        severity: "WARN".to_owned(),
                    });
                }
                _ => {
                    debug!(tool = %lint.tool, "no seed rule template for lint tool");
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Security seed rules (F-2a) — mutable=0, lifecycle-immune (spec F-plan A2)
    // -----------------------------------------------------------------------

    /// Seed the 12 immutable F-2a security rules (CWE-89/79/327/862) into the
    /// rules DB with `mutable=0`.  These rows are immune to auto-retirement and
    /// cannot be overwritten by convention-derived rules.
    ///
    /// Idempotent: rows whose `rule_id` already exists are silently skipped.
    /// Returns the count of newly inserted rows (0 if already present).
    pub fn seed_security_rules(
        conn: &Connection,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        let rows = crate::vibesec_seed::CANONICAL_ROWS;
        let mut inserted = 0usize;

        for &(rule_id, predicate_json) in rows {
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
                rusqlite::params![rule_id],
                |row| row.get(0),
            )?;
            if exists {
                debug!(rule_id, "security seed rule already exists, skipping");
                continue;
            }

            let (title, severity, cwe_comment) = security_rule_meta(rule_id);

            conn.execute(
                "INSERT INTO rules \
                 (rule_id, domain, tier, status, title, description, predicate, \
                  phase_mask, severity, mutable, created_at, source) \
                 VALUES (?1, 'security', 1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, 'SEED')",
                rusqlite::params![
                    rule_id,
                    SEEDED_STATUS,
                    title,
                    cwe_comment,
                    predicate_json,
                    u32::MAX,
                    severity,
                    now,
                ],
            )?;
            inserted += 1;
            debug!(rule_id, "inserted immutable security seed rule");
        }

        debug!(
            total = rows.len(),
            inserted,
            "security rule seeding complete"
        );
        Ok(inserted)
    }

    // -----------------------------------------------------------------------
    // Finance / deflation seed rules (H-18) — mutable=0, lifecycle-immune
    // -----------------------------------------------------------------------

    /// Seed the H-18 deflation promotion gate rule into the rules DB with `mutable=0`.
    ///
    /// This is the LOCKED COMPOSITE gate (FIN-DEFL-001) that blocks strategy
    /// promotion unless DSR >= 0.95 AND PBO < 0.5 AND MinTRL satisfied. The
    /// `mutable=0` flag makes the row lifecycle-immune (cannot be auto-retired
    /// or mutated by the self-mod loop — TD-2).
    ///
    /// # DOMAIN NOTE
    /// Uses domain="trading" (NOT "finance") because "finance" is not in
    /// DOMAIN_KEYWORDS (rule_engine.rs:206-214). "trading" is detected by
    /// path keywords ["trading","backtest","signal","pipeline"]. The rule
    /// also fires unconditionally for any promotion check regardless of
    /// domain because `evaluate_deflation` calls it directly.
    ///
    /// # Canonical thresholds (LOCKED, from crates/rules/seed/finance/deflation_promotion.toml)
    ///   - DSR_THRESHOLD >= 0.95 (ELAI default, conservative vs Harvey-Liu t>3.0 = DSR>=0.9987)
    ///   - PBO_THRESHOLD <  0.50 (Bailey, Borwein, Lopez de Prado, Zhu 2015)
    ///
    /// Idempotent: rows whose `rule_id` already exists are silently skipped.
    /// Returns the count of newly inserted rows (0 if already present).
    pub fn seed_deflation_rules(
        conn: &Connection,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        const RULE_ID: &str = "FIN-DEFL-001";
        const DOMAIN: &str = "trading";
        const TIER: u8 = 1;
        const SEVERITY: &str = "BLOCK";
        const TITLE: &str =
            "Strategy promotion requires DSR >= 0.95, PBO < 0.5, and MinTRL satisfied";
        const DESCRIPTION: &str = concat!(
            "LOCKED COMPOSITE promotion gate (H-18, T-2). ",
            "DSR >= 0.95 (Deflated Sharpe, selection-bias corrected, Bailey & Lopez de Prado 2014). ",
            "PBO < 0.5 (Prob of Backtest Overfitting, CSCV, Bailey et al. 2015). ",
            "T >= MinTRL (minimum track record length). ",
            "Fail-closed on TrialCountMissing (TD-1) + absent PBO (R1/R2). ",
            "mutable=0: LOCKED, lifecycle-immune (TD-2). ",
            "See crates/rules/seed/finance/deflation_promotion.toml."
        );
        // Predicate JSON: COMPOSITE(AND) wrapping three MetricGate children.
        // TrialCountMissing + absent-PBO are enforced at the Rust gate level
        // before the metric values are populated, so they are not representable
        // as MetricGate children (they are enum-level, not scalar).
        const PREDICATE_JSON: &str = concat!(
            r#"{"type":"composite","operator":"and","children":["#,
            r#"{"type":"metric_gate","metric_name":"dsr","operator":"gte","threshold":0.95,"source":"deflation_gate"},"#,
            r#"{"type":"metric_gate","metric_name":"pbo","operator":"lt","threshold":0.5,"source":"deflation_gate"},"#,
            r#"{"type":"metric_gate","metric_name":"t_periods","operator":"gte","threshold":1.0,"source":"deflation_gate"}"#,
            r#"]}"#,
        );

        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
            rusqlite::params![RULE_ID],
            |row| row.get(0),
        )?;
        if exists {
            debug!(rule_id = RULE_ID, "deflation seed rule already exists, skipping");
            return Ok(0);
        }

        conn.execute(
            "INSERT INTO rules \
             (rule_id, domain, tier, status, title, description, predicate, \
              phase_mask, severity, mutable, created_at, source) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 'SEED')",
            rusqlite::params![
                RULE_ID,
                DOMAIN,
                TIER,
                SEEDED_STATUS,
                TITLE,
                DESCRIPTION,
                PREDICATE_JSON,
                u32::MAX, // all phases
                SEVERITY,
                now,
            ],
        )?;

        debug!(rule_id = RULE_ID, "inserted LOCKED deflation promotion seed rule (mutable=0)");
        Ok(1)
    }

    /// Seed the H-19 reusable-holdout query-budget rule into the rules DB with `mutable=0`.
    ///
    /// This is the LOCKED Thresholdout DP gate (FIN-HOLDOUT-001) that blocks an out-of-sample
    /// holdout query once the Thresholdout budget is spent: `budget_remaining > 0` must hold.
    /// `budget_remaining = HOLDOUT_BUDGET_TOTAL - queries_spent`, where `queries_spent` is the
    /// append-only COUNT of above-threshold (budget-consuming) queries. Once exhausted the OOS
    /// query is REFUSED (budget_exhausted) — the holdout cannot be refreshed in-place. The
    /// `mutable=0` flag makes the row lifecycle-immune (TD-2).
    ///
    /// # DOMAIN NOTE
    /// Uses domain="trading" (NOT "finance") for the same reason as
    /// [`Self::seed_deflation_rules`]: "finance" is not in DOMAIN_KEYWORDS
    /// (rule_engine.rs:206-214); "trading" matches the
    /// ["trading","backtest","signal","pipeline"] path keywords.
    ///
    /// # Enforcement boundary
    /// The actual exhaustion refusal is enforced at the Rust gate level
    /// (crates/orchestrator/src/holdout_budget.rs::query_holdout) BEFORE the rules engine; the
    /// MetricGate here is the rules-engine echo + the vibesec blake3 seed-integrity baseline.
    /// The LOCKED Thresholdout parameters (Theorem 25, arXiv:1506.02629) are implemented as
    /// named Rust constants (HOLDOUT_BUDGET_TOTAL, HOLDOUT_TAU, T_FRACTION_OF_TAU, the noise
    /// multipliers, SIGMA_DENOMINATOR_CONSTANT). See
    /// crates/rules/seed/finance/holdout_budget.toml.
    ///
    /// Idempotent: rows whose `rule_id` already exists are silently skipped.
    /// Returns the count of newly inserted rows (0 if already present).
    pub fn seed_holdout_budget_rules(
        conn: &Connection,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        const RULE_ID: &str = "FIN-HOLDOUT-001";
        const DOMAIN: &str = "trading";
        const TIER: u8 = 1;
        const SEVERITY: &str = "BLOCK";
        const TITLE: &str =
            "OOS holdout query requires budget_remaining > 0 (Thresholdout exhaustion)";
        const DESCRIPTION: &str = concat!(
            "LOCKED reusable-holdout query-budget gate (H-19, TB-1 deployment-trust half). ",
            "budget_remaining = HOLDOUT_BUDGET_TOTAL - queries_spent must be > 0; once the ",
            "Thresholdout budget B is spent the OOS query is REFUSED (budget_exhausted) -- the ",
            "honest exhaustion limit, the holdout cannot be refreshed in-place. ",
            "Thresholdout DP, Dwork et al. NIPS 2015 (arXiv:1506.02629), Theorem 25: ",
            "sigma=tau/(96*ln(4*m/beta)), T=3*tau/4, Laplace noise scales 2*sigma/4*sigma/sigma. ",
            "Enforced at the Rust gate level (holdout_budget.rs::query_holdout) BEFORE the rules ",
            "engine. mutable=0: LOCKED, lifecycle-immune (TD-2). Fresh-holdout provisioning is a ",
            "human/frozen Layer-3 step (R8 residue). ",
            "See crates/rules/seed/finance/holdout_budget.toml."
        );
        // A single MetricGate: budget_remaining must be > 0 to pass; <= 0 => Block
        // (budget_exhausted). The Rust gate fail-closes on BudgetMissing/exhausted BEFORE the
        // metric is populated; this is the rules-engine echo.
        const PREDICATE_JSON: &str = concat!(
            r#"{"type":"metric_gate","metric_name":"budget_remaining","operator":"gt","#,
            r#""threshold":0.0,"source":"holdout_budget"}"#,
        );

        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
            rusqlite::params![RULE_ID],
            |row| row.get(0),
        )?;
        if exists {
            debug!(rule_id = RULE_ID, "holdout budget seed rule already exists, skipping");
            return Ok(0);
        }

        conn.execute(
            "INSERT INTO rules \
             (rule_id, domain, tier, status, title, description, predicate, \
              phase_mask, severity, mutable, created_at, source) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 'SEED')",
            rusqlite::params![
                RULE_ID,
                DOMAIN,
                TIER,
                SEEDED_STATUS,
                TITLE,
                DESCRIPTION,
                PREDICATE_JSON,
                u32::MAX, // all phases
                SEVERITY,
                now,
            ],
        )?;

        debug!(rule_id = RULE_ID, "inserted LOCKED holdout budget seed rule (mutable=0)");
        Ok(1)
    }

    /// Seed the H-29 drift-interlock rules into the rules DB with `mutable=0`.
    ///
    /// These are the LOCKED rows for the master deployment-trust interlock (FIN-DRIFT-001 +
    /// FIN-DRIFT-002, the T-13 drift state machine). FIN-DRIFT-001 is the population-stability
    /// axis (PSI on inputs+outputs must be `< PSI_HALT_BAND = 0.25` to resume); FIN-DRIFT-002 is
    /// the schema-fingerprint axis (the live vendor schema's blake3 must match the FROZEN
    /// baseline). The full four-axis interlock (PSI + schema-fingerprint + rolling-14d deviation
    /// + 90d TTL + composed G-10/D-7/H-18 signals) is enforced by the Rust gate
    /// (`drift_interlock.rs::evaluate_drift_interlock`), which DEFAULT-HALTS on missing /
    /// un-evaluable evidence; these seed rows are the rules-engine echo + the vibesec blake3
    /// seed-integrity baseline. The `mutable=0` flag makes the rows lifecycle-immune (cannot be
    /// auto-retired or mutated -- TD-2).
    ///
    /// # DOMAIN NOTE
    /// Uses domain="trading" (NOT "finance") for the same reason as
    /// [`Self::seed_deflation_rules`]: "finance" is not in DOMAIN_KEYWORDS
    /// (rule_engine.rs:206-214); "trading" matches the
    /// ["trading","backtest","signal","pipeline"] path keywords.
    ///
    /// Idempotent: rows whose `rule_id` already exists are silently skipped.
    /// Returns the count of newly inserted rows (0 if all already present).
    pub fn seed_drift_interlock_rules(
        conn: &Connection,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        const DOMAIN: &str = "trading";
        const TIER: u8 = 1;
        const SEVERITY: &str = "BLOCK";

        // FIN-DRIFT-001: the population-stability (PSI) axis.
        const PSI_RULE_ID: &str = "FIN-DRIFT-001";
        const PSI_TITLE: &str = concat!(
            "Live-trading resume requires PSI < 0.25 on input AND output distributions ",
            "(population stability)"
        );
        const PSI_DESCRIPTION: &str = concat!(
            "LOCKED population-stability axis of the H-29 drift interlock (T-13). ",
            "PSI = sum_bins (actual_pct - expected_pct) * ln(actual_pct/expected_pct); ",
            "PSI >= 0.25 (PSI_HALT_BAND) is a major shift => HALT, 0.10 <= PSI < 0.25 is a ",
            "Warning (not latched), an empty/zero bin is fail-closed (floored toward HALT, never ",
            "NaN). Enforced at the Rust gate level (drift_interlock.rs::evaluate_drift_interlock), ",
            "which DEFAULT-HALTS on missing/un-evaluable evidence (R1/R2). mutable=0: LOCKED, ",
            "lifecycle-immune (TD-2). A false HALT is the SAFE failure; the pre-label P(Y|X) ",
            "concept-drift residual routes to a T3 conformal advisory (halt within ONE ",
            "realized-label cycle, NOT never trade a regime break). ",
            "See crates/rules/seed/finance/drift_interlock.toml."
        );
        const PSI_PREDICATE_JSON: &str = concat!(
            r#"{"type":"metric_gate","metric_name":"drift_psi_max","operator":"lt","#,
            r#""threshold":0.25,"source":"drift_interlock"}"#,
        );

        // FIN-DRIFT-002: the schema-fingerprint + TTL/deviation axes.
        const SCHEMA_RULE_ID: &str = "FIN-DRIFT-002";
        const SCHEMA_TITLE: &str = concat!(
            "Live-trading resume requires a matching vendor-schema fingerprint (blake3) and a ",
            "fresh 90d validation"
        );
        const SCHEMA_DESCRIPTION: &str = concat!(
            "LOCKED schema-fingerprint + validation-TTL + rolling-14d-deviation axes of the H-29 ",
            "drift interlock (T-13). The live vendor schema's blake3 fingerprint must equal the ",
            "FROZEN baseline (a mismatch = vendor drift => HALT; blake3 ONLY); the last ",
            "validation must be newer than the 90d TTL (VALIDATION_TTL_DAYS, ClockProvider) which ",
            "forces an H-18 deflated-Sharpe re-check; the rolling-14d live-vs-backtest deviation ",
            "must stay within DEVIATION_HALT_BAND. Enforced at the Rust gate level ",
            "(drift_interlock.rs::evaluate_drift_interlock); missing/un-evaluable evidence on ANY ",
            "axis => HALT (DEFAULT-HALTED, R1/R2). The HALTED state is a persistent ",
            "journal-derived latch (COUNT('halt') > COUNT('unlock')) that SURVIVES restart; the ",
            "unlock is a human/frozen Layer-3 step requiring fresh re-validation evidence (R8 ",
            "residue). mutable=0: LOCKED, lifecycle-immune (TD-2). ",
            "See crates/rules/seed/finance/drift_interlock.toml."
        );
        const SCHEMA_PREDICATE_JSON: &str = concat!(
            r#"{"type":"schema_required","schema_id":"drift_interlock_vendor_schema","#,
            r#""source":"drift_interlock"}"#,
        );

        let mut inserted = 0usize;
        for (rule_id, title, description, predicate_json) in [
            (PSI_RULE_ID, PSI_TITLE, PSI_DESCRIPTION, PSI_PREDICATE_JSON),
            (
                SCHEMA_RULE_ID,
                SCHEMA_TITLE,
                SCHEMA_DESCRIPTION,
                SCHEMA_PREDICATE_JSON,
            ),
        ] {
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
                rusqlite::params![rule_id],
                |row| row.get(0),
            )?;
            if exists {
                debug!(rule_id, "drift interlock seed rule already exists, skipping");
                continue;
            }
            conn.execute(
                "INSERT INTO rules \
                 (rule_id, domain, tier, status, title, description, predicate, \
                  phase_mask, severity, mutable, created_at, source) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 'SEED')",
                rusqlite::params![
                    rule_id,
                    DOMAIN,
                    TIER,
                    SEEDED_STATUS,
                    title,
                    description,
                    predicate_json,
                    u32::MAX, // all phases
                    SEVERITY,
                    now,
                ],
            )?;
            debug!(rule_id, "inserted LOCKED drift interlock seed rule (mutable=0)");
            inserted += 1;
        }
        Ok(inserted)
    }

    /// Seed the H-20 leakage static-analysis COMPOSITE rule into the rules DB
    /// with `mutable=0`.
    ///
    /// This is the LOCKED COMPOSITE gate (FIN-LEAK-001) that blocks ML-pipeline
    /// / backtest promotion when a statically-SOUND data-leakage class fires
    /// (L1.1 no test set, L1.2 preprocessing-before-split, L1.3
    /// feature-selection-before-split). The `mutable=0` flag makes the row
    /// lifecycle-immune (cannot be auto-retired or mutated — TD-2).
    ///
    /// # DOMAIN NOTE
    /// Uses domain="trading" (NOT "finance") for the same reason as
    /// [`Self::seed_deflation_rules`]: "finance" is not in DOMAIN_KEYWORDS
    /// (rule_engine.rs:206-214); "trading" matches the
    /// ["trading","backtest","signal","pipeline"] path keywords. The rule also
    /// fires unconditionally for any leakage check because
    /// `leakage_gate::evaluate_leakage` evaluates the COMPOSITE directly.
    ///
    /// # Soundness boundary (Kapoor & Narayanan 2207.07048)
    /// Sound for only 3 of the 8 Kapoor classes. The residue (L1.4/L2/L3.x) is
    /// routed to human Layer-3 by the gate's `HumanReviewRequired` arm, NEVER
    /// emitted as a sound block. See crates/rules/seed/finance/leakage.toml.
    ///
    /// Idempotent: rows whose `rule_id` already exists are silently skipped.
    /// Returns the count of newly inserted rows (0 if already present).
    pub fn seed_leakage_rules(
        conn: &Connection,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        const RULE_ID: &str = "FIN-LEAK-001";
        const DOMAIN: &str = "trading";
        const TIER: u8 = 1;
        const SEVERITY: &str = "BLOCK";
        const TITLE: &str = concat!(
            "ML-pipeline data-leakage: no fit-site may reach both a train and a ",
            "test sink before split"
        );
        const DESCRIPTION: &str = concat!(
            "LOCKED COMPOSITE leakage gate (H-20, T-3). ",
            "Sound static block for L1.1 (no test set), L1.2 (preprocessing ",
            "before split), L1.3 (feature-selection before split) only. ",
            "Composes GraphReachability over the DKG: a fit-site reaching BOTH a ",
            "train sink AND a test sink is the L1.2/L1.3 violation. ",
            "Residue (L1.4/L2/L3.x) routes to human Layer-3 (never a sound block). ",
            "FAIL-CLOSED on absent/empty DKG (R1/R2). mutable=0: LOCKED, ",
            "lifecycle-immune (TD-2). Suppression requires a REGISTERED allowlist ",
            "row (never silent). See crates/rules/seed/finance/leakage.toml + ",
            "workflow/papers/kapoor-narayanan-leakage-taxonomy-interface.md."
        );
        // Predicate JSON: COMPOSITE(AND) wrapping a GraphReachability child
        // (fit-site -> test sink, must_not_exist=true negative reachability) +
        // a SchemaRequired split-boundary leg. The fit-reaches-BOTH-sinks
        // property is enforced as two positive reachability queries in the Rust
        // gate (leakage_gate::evaluate_leakage); this JSON documents the
        // canonical invariant for the rules engine. The missing-.shift(1)
        // AstPattern is a HEURISTIC residue flag (NOT in the sound composite).
        const PREDICATE_JSON: &str = concat!(
            r#"{"type":"composite","operator":"and","children":["#,
            r#"{"type":"graph_reachability","source_pattern":"fit","sink_pattern":"test","#,
            r#""must_not_exist":true,"max_depth":8,"#,
            r#""edge_types":["data_flow","PARAMETER","RETURN","ASSIGNMENT","FIELD_ACCESS"]},"#,
            r#"{"type":"schema_required","schema":{},"output_type":"train_test_split_boundary"}"#,
            r#"]}"#,
        );

        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
            rusqlite::params![RULE_ID],
            |row| row.get(0),
        )?;
        if exists {
            debug!(rule_id = RULE_ID, "leakage seed rule already exists, skipping");
            return Ok(0);
        }

        conn.execute(
            "INSERT INTO rules \
             (rule_id, domain, tier, status, title, description, predicate, \
              phase_mask, severity, mutable, created_at, source) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 'SEED')",
            rusqlite::params![
                RULE_ID,
                DOMAIN,
                TIER,
                SEEDED_STATUS,
                TITLE,
                DESCRIPTION,
                PREDICATE_JSON,
                u32::MAX, // all phases
                SEVERITY,
                now,
            ],
        )?;

        debug!(rule_id = RULE_ID, "inserted LOCKED leakage COMPOSITE seed rule (mutable=0)");
        Ok(1)
    }

    /// Seed the H-22 trading null-test COMPOSITE rule into the rules DB with
    /// `mutable=0`.
    ///
    /// This is the LOCKED COMPOSITE gate (FIN-NULL-001) that documents the
    /// permutation-null significance bar for strategy promotion: a genuine edge
    /// must be significant against a block-permutation null after the Bonferroni
    /// selection correction (`p < alpha / N`, N from the H-17 trial ledger). The
    /// `mutable=0` flag makes the row lifecycle-immune (cannot be auto-retired or
    /// mutated — TD-2).
    ///
    /// # DOMAIN NOTE
    /// Uses domain="trading" (NOT "finance") for the same reason as
    /// [`Self::seed_deflation_rules`]: "finance" is not in DOMAIN_KEYWORDS
    /// (rule_engine.rs:206-214); "trading" matches the
    /// ["trading","backtest","signal","pipeline"] path keywords.
    ///
    /// # Seed-as-documentation+integrity-baseline (NOT a runtime predicate read)
    /// Like the H-18/H-20 sibling gates, the trading null-test gate
    /// (`trading_null_gate::evaluate_trading_null`) evaluates its LOCKED `ALPHA`
    /// + `K_MIN_PERMUTATIONS` as named Rust constants in-code; it does NOT query
    /// this rules.db row at decision time. This seed row exists for (a) canonical
    /// documentation of the locked alpha and (b) the vibesec blake3 seed-integrity
    /// baseline (a tampered seed fails the integrity check closed). The
    /// `metric_gate` child documents the canonical invariant
    /// (`p_value lt alpha`); the Bonferroni `alpha / N` correction + the K-floor
    /// fail-closed guard are enforced in the Rust gate.
    ///
    /// Idempotent: rows whose `rule_id` already exists are silently skipped.
    /// Returns the count of newly inserted rows (0 if already present).
    pub fn seed_trading_nulls_rules(
        conn: &Connection,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        const RULE_ID: &str = "FIN-NULL-001";
        const DOMAIN: &str = "trading";
        const TIER: u8 = 1;
        const SEVERITY: &str = "BLOCK";
        const TITLE: &str =
            "Strategy edge must be significant against a block-permutation null (p < alpha/N)";
        const DESCRIPTION: &str = concat!(
            "LOCKED COMPOSITE trading null-test gate (H-22, T-6). ",
            "A genuine edge sits in the tail of a block-permutation null ",
            "(p = (C+1)/(K+1) < alpha/N, Bonferroni over N trials from H-17); ",
            "an overfit edge sits inside the null band (p >= alpha/N) and is ",
            "indistinguishable from a permuted version of itself => it survived ",
            "the null => BLOCK (Ojala & Garriga 2010 JMLR 11:1833-1863). ",
            "Block permutation preserves return autocorrelation (iid shuffle is ",
            "anticonservative on time-series). FAIL-CLOSED on missing series, ",
            "under-powered K (K < ceil(1/alpha)-1), TrialCountMissing (TD-1), or ",
            "a NaN statistic (R1/R2). alpha is the ONLY tunable and is LOCKED ",
            "(mutable=0, lifecycle-immune, TD-2). CostMonotoneConcave is DEFERRED ",
            "(sqrt-law of impact holds at institutional scale only; retail is in ",
            "the linear regime, Bucci et al. 2019). ",
            "See crates/rules/seed/finance/trading_nulls.toml."
        );
        // Predicate JSON: COMPOSITE(AND) wrapping one MetricGate child. The
        // permutation p-value must be below alpha. TrialCountMissing, the K-floor,
        // and the Bonferroni alpha/N correction are enforced at the Rust gate
        // level (trading_null_gate::evaluate_trading_null) before the scalar
        // p-value exists, so they are not representable as MetricGate children
        // (they are enum-/guard-level, not scalar).
        const PREDICATE_JSON: &str = concat!(
            r#"{"type":"composite","operator":"and","children":["#,
            r#"{"type":"metric_gate","metric_name":"permutation_p_value","operator":"lt","threshold":0.05,"source":"trading_null_gate"}"#,
            r#"]}"#,
        );

        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
            rusqlite::params![RULE_ID],
            |row| row.get(0),
        )?;
        if exists {
            debug!(rule_id = RULE_ID, "trading null-test seed rule already exists, skipping");
            return Ok(0);
        }

        conn.execute(
            "INSERT INTO rules \
             (rule_id, domain, tier, status, title, description, predicate, \
              phase_mask, severity, mutable, created_at, source) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 'SEED')",
            rusqlite::params![
                RULE_ID,
                DOMAIN,
                TIER,
                SEEDED_STATUS,
                TITLE,
                DESCRIPTION,
                PREDICATE_JSON,
                u32::MAX, // all phases
                SEVERITY,
                now,
            ],
        )?;

        debug!(rule_id = RULE_ID, "inserted LOCKED trading null-test COMPOSITE seed rule (mutable=0)");
        Ok(1)
    }

    /// Seed the H-23 reproducibility sub-gates into the rules DB with
    /// `mutable=0` (two LOCKED rows):
    ///   * `REPRO-SEED-001` — an `AstPattern` seed-coverage rule asserting the
    ///     Python experiment seeds its RNG (no unseeded `np.random` / `torch` /
    ///     `random` call, no `DataLoader(shuffle=True)` without a seeded
    ///     generator). The CHEAP per-edit early-catch for the CAUSE of
    ///     non-reproducibility, before the expensive re-run.
    ///   * `REPRO-ENV-001` — a `SchemaRequired` env-pin manifest rule forcing the
    ///     experiment to declare a pinned-env manifest (lockfile hash, image
    ///     `@sha256`, cuda/driver/torch versions + the determinism env vars
    ///     PYTHONHASHSEED / CUBLAS_WORKSPACE_CONFIG). "Same env" is verifiable
    ///     from this; missing => BLOCK. This STRUCTURALLY enforces hazard A (a
    ///     run that never declared its seed/determinism env cannot pass).
    ///
    /// Both rows are `mutable=0` (lifecycle-immune, TD-2) and feed the vibesec
    /// blake3 seed-integrity baseline (a tampered seed fails the integrity check
    /// closed). The re-run hash-equality DECISION itself is pure Rust
    /// (`rerun_equality::verify_rerun_digest`) and does NOT query these rows at
    /// decision time; these rows are the static CAUSE-catchers + the env-pin
    /// precondition. Domain="trading" (NOT "finance", same reason as the sibling
    /// gates). Idempotent: rows whose `rule_id` already exists are skipped.
    /// Returns the count of newly inserted rows.
    pub fn seed_repro_rules(
        conn: &Connection,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        const DOMAIN: &str = "trading";
        const TIER: u8 = 1;
        const SEVERITY: &str = "BLOCK";

        // -- Sub-gate 1: seed-coverage AST pattern (REPRO-SEED-001). ----------
        const SEED_RULE_ID: &str = "REPRO-SEED-001";
        const SEED_TITLE: &str =
            "Experiment RNG must be seeded (no unseeded np.random/torch/random / DataLoader shuffle)";
        const SEED_DESCRIPTION: &str = concat!(
            "LOCKED seed-coverage AST gate (H-23, T-7). FAILS on any unseeded ",
            "np.random / torch.* / random call or DataLoader(shuffle=True) without ",
            "a seeded generator, and on a missing determinism env var ",
            "(PYTHONHASHSEED, CUBLAS_WORKSPACE_CONFIG, torch.use_deterministic_",
            "algorithms). This is the CHEAP per-edit early-catch for the CAUSE of ",
            "non-reproducibility (caught statically before the expensive same-seed ",
            "re-run). The EFFECT-prover is the runtime gate ",
            "rerun_equality::verify_rerun_digest. mutable=0: LOCKED, lifecycle-",
            "immune (TD-2). Grounding: PyTorch reproducibility docs ",
            "(CUBLAS_WORKSPACE_CONFIG=:16:8 + torch.use_deterministic_algorithms",
            "(True) + manual_seed/numpy.random.seed/random.seed, verified ",
            "2026-06-05). See crates/rules/seed/repro/seed_coverage.toml."
        );
        // AstPattern: a tree-sitter python query flagging an np.random / torch
        // random call whose enclosing scope has no preceding seed call. The
        // canonical query is documented in the toml; the Rust gate (the runtime
        // re-run) is the sound backstop (this static leg is necessary-not-
        // sufficient, R5).
        const SEED_PREDICATE_JSON: &str = concat!(
            r#"{"type":"ast_pattern","language":"python","#,
            r#""query":"(call function: (attribute object: (identifier) @mod attribute: (identifier) @fn) (#match? @mod \"^(np|numpy|torch|random)$\") (#match? @fn \"^(rand|randn|random|shuffle|permutation|randint|manual_seed)$\")) @rng_call","#,
            r#""query_format":"tree_sitter","scope":null}"#,
        );

        // -- Sub-gate 2: env-pin manifest SchemaRequired (REPRO-ENV-001). -----
        const ENV_RULE_ID: &str = "REPRO-ENV-001";
        const ENV_TITLE: &str =
            "Experiment must declare a pinned-env manifest (lockfile hash, image @sha256, cuda/driver/torch, determinism env)";
        const ENV_DESCRIPTION: &str = concat!(
            "LOCKED env-pin manifest gate (H-23, T-7). A SchemaRequired predicate ",
            "forcing the experiment to declare a pinned-env manifest: lockfile ",
            "hash, container image @sha256, cuda/driver/torch versions, and the ",
            "determinism env vars (PYTHONHASHSEED, CUBLAS_WORKSPACE_CONFIG, ",
            "ELAI_SEED). 'Same env' is verifiable from this manifest; a missing ",
            "manifest => BLOCK. This STRUCTURALLY enforces hazard A: a run that ",
            "never declared its seed/determinism env cannot pass the manifest ",
            "schema, so the re-run gate is never reached with an undeclared seed. ",
            "The manifest hash is itself recorded in the RRA evidence bundle ",
            "(tamper-evident). mutable=0: LOCKED, lifecycle-immune (TD-2). ",
            "See crates/rules/seed/repro/env_pin_manifest.toml."
        );
        // SchemaRequired: the output must carry an env-pin manifest object with
        // the required keys. The schema documents the required fields; the
        // rules-engine SchemaRequired predicate enforces presence.
        const ENV_PREDICATE_JSON: &str = concat!(
            r#"{"type":"schema_required","schema":{"required":["#,
            r#""lockfile_blake3","image_sha256","cuda_version","driver_version",""#,
            r#"torch_version","pythonhashseed","cublas_workspace_config","elai_seed"]},"#,
            r#""output_type":"repro_env_pin_manifest"}"#,
        );

        let mut inserted = 0usize;
        for (rule_id, title, description, predicate_json) in [
            (SEED_RULE_ID, SEED_TITLE, SEED_DESCRIPTION, SEED_PREDICATE_JSON),
            (ENV_RULE_ID, ENV_TITLE, ENV_DESCRIPTION, ENV_PREDICATE_JSON),
        ] {
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
                rusqlite::params![rule_id],
                |row| row.get(0),
            )?;
            if exists {
                debug!(rule_id, "repro seed rule already exists, skipping");
                continue;
            }
            conn.execute(
                "INSERT INTO rules \
                 (rule_id, domain, tier, status, title, description, predicate, \
                  phase_mask, severity, mutable, created_at, source) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 'SEED')",
                rusqlite::params![
                    rule_id,
                    DOMAIN,
                    TIER,
                    SEEDED_STATUS,
                    title,
                    description,
                    predicate_json,
                    u32::MAX, // all phases
                    SEVERITY,
                    now,
                ],
            )?;
            debug!(rule_id, "inserted LOCKED H-23 repro seed rule (mutable=0)");
            inserted += 1;
        }

        Ok(inserted)
    }

    /// Seed the H-24 sim-to-real parity Part-A sub-gates into the rules DB with
    /// `mutable=0` (two LOCKED rows):
    ///   * `FIN-SIMREAL-001` — an `AstPattern` mode-divergent-branch ban. FAILS at
    ///     TESTING_FIRST on a branch on backtest-vs-live mode
    ///     (`if self.LiveMode:` / `if backtest:` / `if live:`) found INSIDE the
    ///     order/signal execution path. SCOPED (the `scope` field narrows it to
    ///     the execution-path call-graph) — LiveMode is LEGITIMATE for
    ///     notifications/logging per LEAN, so a blanket ban is wrong.
    ///   * `FIN-SIMREAL-002` — a `SchemaRequired` single-entrypoint manifest rule
    ///     forcing the strategy to declare ONE shared execution entrypoint + its
    ///     swappable adapters (data source, fill model, clock). Missing => BLOCK.
    ///
    /// Both rows are `mutable=0` (lifecycle-immune, TD-2) and feed the vibesec
    /// blake3 seed-integrity baseline (a tampered seed fails the integrity check
    /// closed). The replay==golden DECISION itself is pure Rust
    /// (`sim_real_parity::verify_replay_golden`) and does NOT query these rows at
    /// decision time; these rows are the static STRUCTURAL CAUSE-catchers (R5
    /// necessary-not-sufficient) + the single-entrypoint precondition. Domain=
    /// "trading" (NOT "finance", same reason as the sibling gates). Idempotent:
    /// rows whose `rule_id` already exists are skipped. Returns the count of newly
    /// inserted rows. See crates/rules/seed/repro/sim_real_parity.toml.
    pub fn seed_sim_real_parity_rules(
        conn: &Connection,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        const DOMAIN: &str = "trading";
        const TIER: u8 = 1;
        const SEVERITY: &str = "BLOCK";

        // -- Sub-gate 1: mode-divergent-branch AST ban (FIN-SIMREAL-001). -----
        const AST_RULE_ID: &str = "FIN-SIMREAL-001";
        const AST_TITLE: &str =
            "No mode-divergent branch (if LiveMode / if backtest / if live) on the order/signal execution path";
        const AST_DESCRIPTION: &str = concat!(
            "LOCKED mode-divergent-branch AST ban (H-24, T-8). FAILS on a branch ",
            "on backtest-vs-live mode (if self.LiveMode / if backtest / if live / ",
            "if mode==\"live\") found INSIDE the order-emission / signal / OnData ",
            "execution path: a mode-branch guarding execution/signal/order logic ",
            "makes the backtest run a DIFFERENT code path than live, voiding the ",
            "parity guarantee (the classic sim-real divergence). SCOPED to the ",
            "execution-path modules/methods -- LiveMode is LEGITIMATE for ",
            "notifications/logging/runtime-stats per LEAN, so a blanket ban would ",
            "false-positive-block every legitimate conditional. The EFFECT-prover ",
            "is the runtime gate sim_real_parity::verify_replay_golden (blake3 ",
            "EXACT-equality of a seeded replay vs a frozen golden event log). This ",
            "static AST leg is necessary-not-sufficient (R5); the runtime replay ",
            "is the load-bearing backstop. mutable=0: LOCKED, lifecycle-immune ",
            "(TD-2). Grounding: LEAN live-vs-backtest reconciliation, verified ",
            "2026-06-05. See crates/rules/seed/repro/sim_real_parity.toml."
        );
        // AstPattern: a tree-sitter python query flagging an if/elif whose
        // condition tests a backtest-vs-live mode flag, SCOPED (the `scope` field)
        // to the order/signal execution path so legitimate notification/logging
        // LiveMode branches are not false-blocked. The canonical query + scope are
        // documented in the toml; the Rust runtime replay==golden gate is the
        // sound backstop (this static leg is necessary-not-sufficient, R5).
        const AST_PREDICATE_JSON: &str = concat!(
            r#"{"type":"ast_pattern","language":"python","#,
            r#""query":"(if_statement condition: [(attribute attribute: (identifier) @flag (#match? @flag \"^(LiveMode|AlgorithmMode)$\")) (comparison_operator (attribute attribute: (identifier) @flag2 (#match? @flag2 \"^(LiveMode|AlgorithmMode)$\"))) (identifier) @mode (#match? @mode \"^(live|backtest|mode)$\") (comparison_operator (identifier) @mode2 (#match? @mode2 \"^(live|backtest|mode)$\"))]) @mode_branch","#,
            r#""query_format":"sexp","#,
            r#""scope":"execution_path:order,signal,fill,execution,strategy,on_data,ondata,initialize"}"#,
        );

        // -- Sub-gate 2: single-entrypoint manifest SchemaRequired (002). -----
        const ENTRYPOINT_RULE_ID: &str = "FIN-SIMREAL-002";
        const ENTRYPOINT_TITLE: &str =
            "Strategy must declare a single shared execution entrypoint + swappable adapters (data source, fill model, clock)";
        const ENTRYPOINT_DESCRIPTION: &str = concat!(
            "LOCKED single-entrypoint manifest gate (H-24, T-8). A SchemaRequired ",
            "predicate forcing the strategy to declare ONE shared execution ",
            "entrypoint + its swappable adapters (data source, fill model, clock), ",
            "so the live and backtest paths differ ONLY in the engine-injected ",
            "adapter, never in the strategy logic (the LEAN adapter-swap pattern: ",
            "one IAlgorithm with Initialize()+OnData(); the engine swaps ",
            "IBrokerage/ISynchronizer). The manifest must declare entrypoint, ",
            "init_entrypoint, data_source_adapter, fill_model_adapter, and ",
            "clock_adapter. A missing manifest => BLOCK: without a declared single ",
            "entrypoint the engine cannot guarantee live and backtest run the same ",
            "code path. mutable=0: LOCKED, lifecycle-immune (TD-2). See ",
            "crates/rules/seed/repro/sim_real_parity.toml."
        );
        // SchemaRequired: the output must carry a single-entrypoint manifest with
        // the required keys. The schema documents the required fields; the
        // rules-engine SchemaRequired predicate enforces presence.
        const ENTRYPOINT_PREDICATE_JSON: &str = concat!(
            r#"{"type":"schema_required","schema":{"required":["#,
            r#""entrypoint","init_entrypoint","data_source_adapter",""#,
            r#"fill_model_adapter","clock_adapter"]},"#,
            r#""output_type":"sim_real_entrypoint_manifest"}"#,
        );

        let mut inserted = 0usize;
        for (rule_id, title, description, predicate_json) in [
            (AST_RULE_ID, AST_TITLE, AST_DESCRIPTION, AST_PREDICATE_JSON),
            (
                ENTRYPOINT_RULE_ID,
                ENTRYPOINT_TITLE,
                ENTRYPOINT_DESCRIPTION,
                ENTRYPOINT_PREDICATE_JSON,
            ),
        ] {
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
                rusqlite::params![rule_id],
                |row| row.get(0),
            )?;
            if exists {
                debug!(rule_id, "sim-real parity seed rule already exists, skipping");
                continue;
            }
            conn.execute(
                "INSERT INTO rules \
                 (rule_id, domain, tier, status, title, description, predicate, \
                  phase_mask, severity, mutable, created_at, source) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 'SEED')",
                rusqlite::params![
                    rule_id,
                    DOMAIN,
                    TIER,
                    SEEDED_STATUS,
                    title,
                    description,
                    predicate_json,
                    u32::MAX, // all phases
                    SEVERITY,
                    now,
                ],
            )?;
            debug!(rule_id, "inserted LOCKED H-24 sim-real parity seed rule (mutable=0)");
            inserted += 1;
        }

        Ok(inserted)
    }

    /// Seed the LOCKED H-25 result-manifest re-derivation Part-A sub-gates into the
    /// project rules.db (FIN-RESMANIFEST-001 SchemaRequired manifest-shape +
    /// FIN-RESMANIFEST-002 monkeypatch AstPattern SCOPED to the result-emitting
    /// surface, both mutable=0). These are the STATIC legs of the H-25 anti-
    /// fabrication gate; the re-parse/re-hash DECISION itself is pure Rust
    /// (`result_manifest::verify_result`) and does NOT query these rows at decision
    /// time -- they are the structural manifest-shape precondition + the monkeypatch
    /// CAUSE-catcher (R5 necessary-not-sufficient). Domain="trading" (NOT "finance",
    /// same reason as the sibling gates). Idempotent: rows whose `rule_id` already
    /// exists are skipped. Returns the count of newly inserted rows. See
    /// crates/rules/seed/repro/result_manifest.toml.
    pub fn seed_result_manifest_rules(
        conn: &Connection,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        const DOMAIN: &str = "trading";
        const TIER: u8 = 1;
        const SEVERITY: &str = "BLOCK";

        // -- Sub-gate 1: result-manifest SchemaRequired (FIN-RESMANIFEST-001). ---
        const SCHEMA_RULE_ID: &str = "FIN-RESMANIFEST-001";
        const SCHEMA_TITLE: &str =
            "Every reported empirical number must carry a result manifest binding it to its provenance";
        const SCHEMA_DESCRIPTION: &str = concat!(
            "LOCKED result-manifest SchemaRequired gate (H-25, T-9). A reported ",
            "empirical NUMBER (Sharpe / P&L / plot-derived stat) MUST carry a result ",
            "manifest declaring its provenance; a missing manifest (or required ",
            "field) => BLOCK. The anti-fabrication signature -- a reported number ",
            "with no manifest is the F-4 None-hash analog. The runtime re-derivation ",
            "gate (result_manifest::verify_result) RE-PARSES the number from the ",
            "orchestrator-captured output and RE-HASHES the pinned inputs; this ",
            "SchemaRequired leg is the structural precondition. Required: ",
            "metric_name, value, captured_output_blake3, parser_expr, data_hash, ",
            "code_commit, env_lock_hash, run_id, elai_seed, n_trials. DSR/PBO/N are ",
            "BOUND from H-17/H-18, never recomputed (GD-1). mutable=0: LOCKED, ",
            "lifecycle-immune (TD-2). See crates/rules/seed/repro/result_manifest.toml."
        );
        const SCHEMA_PREDICATE_JSON: &str = concat!(
            r#"{"type":"schema_required","schema":{"required":["#,
            r#""metric_name","value","captured_output_blake3","parser_expr","#,
            r#""data_hash","code_commit","env_lock_hash","run_id","elai_seed",""#,
            r#"n_trials"]},"output_type":"result_manifest"}"#,
        );

        // -- Sub-gate 2: monkeypatch-ban AST (FIN-RESMANIFEST-002). -----------
        const AST_RULE_ID: &str = "FIN-RESMANIFEST-002";
        const AST_TITLE: &str =
            "No monkeypatching / redefining a result-emitting function to fake a number";
        const AST_DESCRIPTION: &str = concat!(
            "LOCKED monkeypatch-ban AST gate (H-25, T-9). FAILS on strategy code ",
            "that monkeypatches / redefines / rebinds a RESULT-EMITTING function ",
            "(setattr(obj,\"compute_sharpe\",...), a module-level assignment ",
            "rebinding compute_sharpe / report_pnl / emit_metric / record_result) so ",
            "the run reports a FABRICATED number while the frozen metric/backtest ",
            "harness is bypassed. The harness is gate-manifest-frozen (TD-2); a ",
            "monkeypatch voids the re-derivation guarantee by construction. SCOPED to ",
            "the result/metric/report execution surface -- legitimate dynamic wiring ",
            "/ DI in non-result modules is out of scope, so a blanket ban would ",
            "false-block. The EFFECT-prover is the runtime gate ",
            "result_manifest::verify_result (blake3 re-parse + re-hash of an ",
            "orchestrator-captured output). This static AST leg is ",
            "necessary-not-sufficient (R5); the runtime re-parse is the load-bearing ",
            "backstop. mutable=0: LOCKED, lifecycle-immune (TD-2). See ",
            "crates/rules/seed/repro/result_manifest.toml."
        );
        const AST_PREDICATE_JSON: &str = concat!(
            r#"{"type":"ast_pattern","language":"python","#,
            r#""query":"[(call function: (identifier) @f (#eq? @f \"setattr\") arguments: (argument_list (_) (string) @attr (#match? @attr \"(compute_sharpe|report_pnl|emit_metric|record_result|compute_metric)\"))) (assignment left: (identifier) @lhs (#match? @lhs \"^(compute_sharpe|report_pnl|emit_metric|record_result|compute_metric)$\") right: [(lambda) (identifier) (attribute)])] @monkeypatch","#,
            r#""query_format":"sexp","#,
            r#""scope":"execution_path:result,metric,report,sharpe,pnl,emit_metric,record_result"}"#,
        );

        let mut inserted = 0usize;
        for (rule_id, title, description, predicate_json) in [
            (
                SCHEMA_RULE_ID,
                SCHEMA_TITLE,
                SCHEMA_DESCRIPTION,
                SCHEMA_PREDICATE_JSON,
            ),
            (AST_RULE_ID, AST_TITLE, AST_DESCRIPTION, AST_PREDICATE_JSON),
        ] {
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
                rusqlite::params![rule_id],
                |row| row.get(0),
            )?;
            if exists {
                debug!(rule_id, "result-manifest seed rule already exists, skipping");
                continue;
            }
            conn.execute(
                "INSERT INTO rules \
                 (rule_id, domain, tier, status, title, description, predicate, \
                  phase_mask, severity, mutable, created_at, source) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 'SEED')",
                rusqlite::params![
                    rule_id,
                    DOMAIN,
                    TIER,
                    SEEDED_STATUS,
                    title,
                    description,
                    predicate_json,
                    u32::MAX, // all phases
                    SEVERITY,
                    now,
                ],
            )?;
            debug!(rule_id, "inserted LOCKED H-25 result-manifest seed rule (mutable=0)");
            inserted += 1;
        }

        Ok(inserted)
    }

    /// H-26 (T-10) -- seed the LOCKED financial-invariant naked-float-`==` ban
    /// (FIN-INVARIANT-001 AstPattern, mutable=0, domain="trading"). One row.
    ///
    /// The DECISION leg (the accounting-identity EXACT integer-cents checks + the
    /// option-pricing / numeric MR tolerance checks + the primary-vs-shadow
    /// divergence) is PURE Rust in shadow_exec::diff_shadow_impls; this seed row is
    /// the STATIC necessary-not-sufficient leg (R5) banning a naked float `==`/`!=`
    /// on money/price in execution code (the WRONG regime -- a tolerance compare
    /// belongs there). SCOPED to the order/signal/pnl/price execution surface via the
    /// execution_path:<csv> CONTAINS predicate (NOT a blanket ban). Idempotent
    /// (skips if the rule already exists). See
    /// crates/rules/seed/finance/financial_invariants.toml +
    /// workflow/reasoning/H-26-decisions.md.
    pub fn seed_financial_invariant_rules(
        conn: &Connection,
        clock: &impl ClockProvider,
    ) -> Result<usize, RulesDbError> {
        let now = clock
            .system_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0);

        const DOMAIN: &str = "trading";
        const TIER: u8 = 1;
        const SEVERITY: &str = "BLOCK";

        // -- Naked-float-`==` ban AstPattern (FIN-INVARIANT-001). -------------
        const RULE_ID: &str = "FIN-INVARIANT-001";
        const TITLE: &str =
            "No naked float == / != on money or price in execution code (use a tolerance compare)";
        const DESCRIPTION: &str = concat!(
            "LOCKED naked-float-equality ban (H-26, T-10). FAILS on execution code ",
            "that compares a float money/price quantity with a naked `==`/`!=` against ",
            "a float LITERAL (pnl == 0.0, price != 0.0) instead of a tolerance compare. ",
            "A float-== on money is a precision bug: FP round-off makes byte-equality ",
            "of two computed prices brittle. The CORRECT regime for an accounting ",
            "identity is EXACT INTEGER CENTS (== on i64), checked by ",
            "shadow_exec::diff_shadow_impls; the CORRECT regime for derived ",
            "option-pricing / numeric values is a tolerance band. This static AST leg ",
            "catches the WRONG regime (a naked float ==/!=). SCOPED to the ",
            "order/signal/pnl/price/fill execution surface (NOT a blanket ban). ",
            "RESIDUAL (R5, necessary-not-sufficient): the grammar catches a ",
            "float-LITERAL on the RIGHT operand only; a left-literal or a ",
            "float-VARIABLE == is not caught -- the integer-cents EXACT regime in ",
            "diff_shadow_impls is the load-bearing backstop. mutable=0: LOCKED, ",
            "lifecycle-immune (TD-2). See ",
            "crates/rules/seed/finance/financial_invariants.toml."
        );
        // A tree-sitter-rust query matching a binary_expression with a `==` or `!=`
        // operator and a float_literal RIGHT operand (the dominant naked-float-literal
        // idiom). Verified live (2026-06-05) against tree-sitter-rust 0.24: fires on
        // `pnl == 0.0`, does NOT fire on `(a-b).abs() < 1e-9`. The `left:` field is
        // rejected by the grammar, so left-literal / float-variable forms are residual.
        const PREDICATE_JSON: &str = concat!(
            r#"{"type":"ast_pattern","language":"rust","#,
            r#""query":"[(binary_expression operator: \"==\" right: (float_literal)) "#,
            r#"(binary_expression operator: \"!=\" right: (float_literal))] @naked_float_eq","#,
            r#""query_format":"sexp","#,
            r#""scope":"execution_path:order,signal,pnl,price,fill,trade,equity,cash"}"#,
        );

        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM rules WHERE rule_id = ?1",
            rusqlite::params![RULE_ID],
            |row| row.get(0),
        )?;
        if exists {
            debug!(rule_id = RULE_ID, "financial-invariant seed rule already exists, skipping");
            return Ok(0);
        }
        conn.execute(
            "INSERT INTO rules \
             (rule_id, domain, tier, status, title, description, predicate, \
              phase_mask, severity, mutable, created_at, source) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 'SEED')",
            rusqlite::params![
                RULE_ID,
                DOMAIN,
                TIER,
                SEEDED_STATUS,
                TITLE,
                DESCRIPTION,
                PREDICATE_JSON,
                u32::MAX, // all phases
                SEVERITY,
                now,
            ],
        )?;
        debug!(rule_id = RULE_ID, "inserted LOCKED H-26 financial-invariant seed rule (mutable=0)");
        Ok(1)
    }
}

// ---------------------------------------------------------------------------
// Lifecycle wiring: auto-promotion and auto-retirement (A-33, spec section 6)
// ---------------------------------------------------------------------------

/// Minimum true-positive (success) count before a SHADOW seed rule can promote.
const AUTO_PROMOTE_SUCCESSES: i64 = 5;

/// False-positive count threshold for auto-retiring an ACTIVE seed rule.
const AUTO_RETIRE_FP_COUNT: i64 = 3;

/// Check whether a seeded rule qualifies for auto-promotion (SHADOW -> ACTIVE).
///
/// Returns `Ok(())` if promotion succeeds, rule lacks sufficient data, or rule
/// is not in SHADOW state (no-op). Only returns `Err` on unexpected DB errors.
pub fn check_and_promote_seeded_rules(
    conn: &Connection,
    rule_id: &str,
    clock: &impl ClockProvider,
) -> Result<(), RulesDbError> {
    let metrics = match crate::lifecycle::compute_metrics_snapshot(conn, rule_id) {
        Ok(m) => m,
        Err(crate::error::LifecycleError::RuleNotFound { .. }) => return Ok(()),
        Err(crate::error::LifecycleError::Sqlite(e)) => return Err(RulesDbError::Sqlite(e)),
        Err(_) => return Ok(()),
    };

    // LIFECYCLE_GATE_ORDER: check InsufficientShadowData before thresholds.
    // CCR=None means no shadow evaluations recorded — too early to promote.
    if metrics.counterfactual_causal_rate.is_none() {
        debug!(rule_id, "insufficient shadow data for promotion check, skipping");
        return Ok(());
    }

    // Not enough successes yet.
    if metrics.true_positives < AUTO_PROMOTE_SUCCESSES {
        return Ok(());
    }

    // Attempt transition; InvalidTransition means not in SHADOW (already promoted or retired).
    match crate::lifecycle::transition_to_active(conn, rule_id, &metrics, false, clock) {
        Ok(()) => {
            debug!(rule_id, "auto-promoted seeded rule to ACTIVE");
            Ok(())
        }
        Err(crate::error::LifecycleError::InvalidTransition { .. }) => Ok(()),
        Err(crate::error::LifecycleError::ThresholdNotMet { .. }) => Ok(()),
        Err(crate::error::LifecycleError::InsufficientShadowData) => Ok(()),
        Err(crate::error::LifecycleError::Sqlite(e)) => Err(RulesDbError::Sqlite(e)),
        Err(_) => Ok(()),
    }
}

/// Check whether a seeded rule qualifies for auto-retirement (ACTIVE -> RETIRED).
///
/// Returns `Ok(())` if retirement succeeds, rule has insufficient FPs, or rule
/// is not in ACTIVE state (no-op). Only returns `Err` on unexpected DB errors.
pub fn check_and_retire_seeded_rules(
    conn: &Connection,
    rule_id: &str,
    clock: &impl ClockProvider,
) -> Result<(), RulesDbError> {
    let metrics = match crate::lifecycle::compute_metrics_snapshot(conn, rule_id) {
        Ok(m) => m,
        Err(crate::error::LifecycleError::RuleNotFound { .. }) => return Ok(()),
        Err(crate::error::LifecycleError::Sqlite(e)) => return Err(RulesDbError::Sqlite(e)),
        Err(_) => return Ok(()),
    };

    // Not enough false positives to retire.
    if metrics.false_positives < AUTO_RETIRE_FP_COUNT {
        return Ok(());
    }

    // Attempt transition; InvalidTransition means not in ACTIVE (still SHADOW or already retired).
    match crate::lifecycle::transition_to_retired(
        conn,
        rule_id,
        "auto-retired: false positive count exceeded threshold",
        false,
        clock,
    ) {
        Ok(()) => {
            debug!(rule_id, "auto-retired seeded rule");
            Ok(())
        }
        Err(crate::error::LifecycleError::InvalidTransition { .. }) => Ok(()),
        Err(crate::error::LifecycleError::ImmutableRuleCannotAutoRetire) => Ok(()),
        Err(crate::error::LifecycleError::Sqlite(e)) => Err(RulesDbError::Sqlite(e)),
        Err(_) => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return (title, severity, cwe_description) metadata for a known F-2a security rule_id.
///
/// Used exclusively by [`ProjectRuleSeeder::seed_security_rules`].
/// The tuples are ordered by rule_id prefix for readability; all IDs must
/// appear in [`crate::vibesec_seed::CANONICAL_ROWS`].
fn security_rule_meta(rule_id: &str) -> (&'static str, &'static str, &'static str) {
    match rule_id {
        "SEC-SQLI-001" => (
            "SQL injection via Python f-string interpolation (CWE-89)",
            "BLOCK",
            "CWE-89: SQL injection through direct variable interpolation into SQL string via f-string.",
        ),
        "SEC-SQLI-002" => (
            "SQL injection via Python % or .format() string interpolation (CWE-89)",
            "BLOCK",
            "CWE-89: SQL injection through % or .format() interpolation into SQL string.",
        ),
        "SEC-SQLI-003" => (
            "SQL injection via TypeScript template-literal interpolation (CWE-89)",
            "BLOCK",
            "CWE-89: SQL injection through template-literal ${} interpolation into SQL string.",
        ),
        "SEC-XSS-001" => (
            "XSS via Python mark_safe / Markup with variable content (CWE-79)",
            "BLOCK",
            "CWE-79: Cross-site scripting through use of mark_safe or Markup bypassing auto-escaping.",
        ),
        "SEC-XSS-002" => (
            "XSS via TypeScript innerHTML / outerHTML assignment (CWE-79)",
            "BLOCK",
            "CWE-79: Cross-site scripting through direct innerHTML or outerHTML assignment.",
        ),
        "SEC-XSS-003" => (
            "XSS via TypeScript dangerouslySetInnerHTML (CWE-79)",
            "BLOCK",
            "CWE-79: Cross-site scripting through React dangerouslySetInnerHTML without sanitization.",
        ),
        "SEC-WCRYPTO-001" => (
            "Weak hash: Python hashlib MD5 or SHA-1 in production (CWE-327)",
            "BLOCK",
            "CWE-327: Use of a broken or risky cryptographic algorithm (MD5/SHA-1 for integrity).",
        ),
        "SEC-WCRYPTO-002" => (
            "Weak cipher: Python DES or RC4 usage (CWE-327)",
            "BLOCK",
            "CWE-327: Use of a broken or risky cryptographic algorithm (DES/RC4).",
        ),
        "SEC-WCRYPTO-003" => (
            "Weak hash: TypeScript crypto.createHash with MD5 or SHA1 (CWE-327)",
            "BLOCK",
            "CWE-327: Use of a broken or risky cryptographic algorithm (MD5/SHA1 in Node.js crypto).",
        ),
        "SEC-WCRYPTO-004" => (
            "Weak cipher: TypeScript crypto.createCipheriv with DES or RC4 (CWE-327)",
            "BLOCK",
            "CWE-327: Use of a broken or risky cryptographic algorithm (DES/RC4 in Node.js crypto).",
        ),
        "SEC-AUTHZ-001" => (
            "Possible missing authz: Python FastAPI route without security dependency (CWE-862)",
            "WARN",
            "CWE-862: Missing authorization check on FastAPI route handler.",
        ),
        "SEC-AUTHZ-002" => (
            "Possible missing authz: TypeScript Express route without auth middleware (CWE-862)",
            "WARN",
            "CWE-862: Missing authorization check on Express route handler.",
        ),
        // F-2c: hardcoded credentials (CWE-798).
        "SEC-CREDS-001" => (
            "Hardcoded credential: Python secret-named variable assigned a literal (CWE-798)",
            "BLOCK",
            "CWE-798: Use of hard-coded credentials (secret-named variable assigned a string literal).",
        ),
        "SEC-CREDS-002" => (
            "Hardcoded credential: TypeScript secret-named variable assigned a literal (CWE-798)",
            "BLOCK",
            "CWE-798: Use of hard-coded credentials (secret-named variable assigned a string literal).",
        ),
        // F-2c: eval/exec code injection (CWE-95).
        "SEC-EVAL-001" => (
            "Code injection: Python eval/exec on a non-literal argument (CWE-95)",
            "BLOCK",
            "CWE-95: Eval injection via eval/exec on a non-literal (dynamic) argument.",
        ),
        "SEC-EVAL-002" => (
            "Code injection: TypeScript eval / new Function on a non-literal argument (CWE-95)",
            "BLOCK",
            "CWE-95: Eval injection via eval / new Function on a non-literal (dynamic) argument.",
        ),
        // F-2c: hardcoded cryptographic key (CWE-321).
        "SEC-KEY-001" => (
            "Hardcoded crypto key: Python literal key passed to AES/DES/ARC4/Fernet (CWE-321)",
            "BLOCK",
            "CWE-321: Use of a hard-coded cryptographic key (literal key at the crypto sink).",
        ),
        "SEC-KEY-002" => (
            "Hardcoded crypto key: TypeScript literal key passed to createCipheriv (CWE-321)",
            "BLOCK",
            "CWE-321: Use of a hard-coded cryptographic key (literal key at the crypto sink).",
        ),
        // F-2c: secrets written to logs (CWE-532).
        "SEC-LOG-001" => (
            "Secret in logs: Python secret-named identifier in a logging/print sink (CWE-532)",
            "BLOCK",
            "CWE-532: Insertion of sensitive information into a log file (secret in a log/print sink).",
        ),
        "SEC-LOG-002" => (
            "Secret in logs: TypeScript secret-named identifier in a console sink (CWE-532)",
            "BLOCK",
            "CWE-532: Insertion of sensitive information into a log file (secret in a console sink).",
        ),
        _ => ("Unknown security rule", "WARN", "Unknown CWE."),
    }
}

/// Generate a regex pattern that matches identifiers violating the given style.
/// Used as GREP_PATTERN predicates. The regex is inverted: it matches
/// function/type definitions that do NOT follow the convention.
fn naming_regex_for_style(style: &str) -> &'static str {
    match style {
        // Match function defs with non-snake_case names.
        "snake_case" => r"(?:fn|def|function)\s+[A-Z]",
        // Match type defs with non-PascalCase names.
        "PascalCase" => r"(?:struct|enum|trait|class|interface|type)\s+[a-z]",
        // Match const defs with non-SCREAMING_SNAKE names.
        "SCREAMING_SNAKE_CASE" => r"(?:const|static)\s+[a-z]",
        // camelCase: match function defs with underscores (not camelCase).
        "camelCase" => r"(?:function|const|let|var)\s+[a-z]+_[a-z]",
        _ => "",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use elai_core::clock::SeededClock;
    use elai_core::project_profile::*;

    use crate::schema::open_rules_db;

    use super::*;

    fn test_clock() -> SeededClock {
        SeededClock::new()
    }

    fn test_profile() -> ProjectProfile {
        ProjectProfile {
            schema_version: PROFILE_SCHEMA_VERSION,
            generated_at: "2026-04-14T00:00:00Z".to_owned(),
            project_root: PathBuf::from("/test"),
            build_system: BuildSystemProfile {
                system: BuildSystem::Cargo,
                config_path: PathBuf::from("Cargo.toml"),
                version: None,
                monorepo: false,
                workspaces: Vec::new(),
            },
            languages: LanguageProfile {
                primary: Language::Rust,
                secondary: Vec::new(),
                distribution: std::collections::BTreeMap::new(),
                tree_sitter_coverage: 0.0,
            },
            frameworks: Vec::new(),
            test_profile: TestProfile {
                runner_cmd: Some("cargo test".to_owned()),
                location_pattern: TestLocationPattern::SameFile,
                coverage_tool: None,
                fixture_dir: None,
            },
            lint_tools: vec![LintProfile {
                tool: "clippy".to_owned(),
                config_path: Some(PathBuf::from("clippy.toml")),
                language: Language::Rust,
                auto_fix: false,
            }],
            entry_points: Vec::new(),
            ci: None,
            naming: NamingConventions {
                functions: Convention::Strong("snake_case".to_owned()),
                types: Convention::Strong("PascalCase".to_owned()),
                constants: Convention::Strong("SCREAMING_SNAKE_CASE".to_owned()),
                modules: Convention::Unknown,
            },
            error_handling: ErrorHandlingProfile {
                pattern: ErrorPattern::ResultBased,
                propagation: PropagationStyle::QuestionMark,
                error_crate: Some("thiserror".to_owned()),
            },
            module_organization: ModuleOrganization {
                pattern: OrgPattern::Monorepo,
            },
            dep_profile: elai_core::project_profile::DependencyProfile::default(),
            project_vision: None,
            architecture: None,
            domains: Vec::new(),
            commands: BuildCommandRegistry::default(),
            fix_template_keys: Vec::new(),
            constraints: Vec::new(),
            overrides_applied: false,
        }
    }

    #[test]
    fn seed_rules_inserts_naming_rules() {
        let clock = test_clock();
        let conn = open_rules_db(std::path::Path::new(":memory:"), &clock)
            .expect("in-memory DB");
        let profile = test_profile();

        let count = ProjectRuleSeeder::seed_rules(&conn, &profile, &clock)
            .expect("seeding should succeed");

        // Expected: 3 naming rules (functions, types, constants; modules=Unknown skipped)
        //         + 1 error crate (thiserror)
        //         + 1 result_based
        //         + 1 test same_file
        //         + 1 clippy lint
        //         = 7 rules
        assert_eq!(count, 7);

        // Verify they're in the DB with correct status and source.
        let rule_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM rules WHERE source = 'SEED' AND status = 'SHADOW'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(rule_count, 7);
    }

    #[test]
    fn seed_rules_idempotent() {
        let clock = test_clock();
        let conn = open_rules_db(std::path::Path::new(":memory:"), &clock)
            .expect("in-memory DB");
        let profile = test_profile();

        let first = ProjectRuleSeeder::seed_rules(&conn, &profile, &clock)
            .expect("first seed");
        let second = ProjectRuleSeeder::seed_rules(&conn, &profile, &clock)
            .expect("second seed");

        assert!(first > 0);
        assert_eq!(second, 0, "second seed should insert nothing (idempotent)");
    }

    #[test]
    fn seed_rules_no_rules_for_unknown_conventions() {
        let clock = test_clock();
        let conn = open_rules_db(std::path::Path::new(":memory:"), &clock)
            .expect("in-memory DB");
        let mut profile = test_profile();

        // Set all conventions to Unknown.
        profile.naming = NamingConventions::default();
        profile.error_handling = ErrorHandlingProfile::default();
        profile.test_profile.location_pattern = TestLocationPattern::Unknown;
        profile.lint_tools.clear();

        let count = ProjectRuleSeeder::seed_rules(&conn, &profile, &clock)
            .expect("seeding should succeed");
        assert_eq!(count, 0, "no rules for unknown conventions");
    }

    #[test]
    fn seed_rules_moderate_convention_gets_info_severity() {
        let clock = test_clock();
        let conn = open_rules_db(std::path::Path::new(":memory:"), &clock)
            .expect("in-memory DB");
        let mut profile = test_profile();
        profile.naming.functions = Convention::Moderate("snake_case".to_owned());
        profile.naming.types = Convention::Unknown;
        profile.naming.constants = Convention::Unknown;
        profile.error_handling = ErrorHandlingProfile::default();
        profile.test_profile.location_pattern = TestLocationPattern::Unknown;
        profile.lint_tools.clear();

        let count = ProjectRuleSeeder::seed_rules(&conn, &profile, &clock)
            .expect("seeding should succeed");
        assert_eq!(count, 1);

        let severity: String = conn
            .query_row(
                "SELECT severity FROM rules WHERE rule_id = 'proj.naming.functions.snake_case'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(severity, "INFO");
    }

    #[test]
    fn naming_regex_rejects_violating_names() {
        let pattern = naming_regex_for_style("snake_case");
        assert!(!pattern.is_empty());
        let re = regex::Regex::new(pattern).expect("valid regex");
        // Should match: fn CamelCase (violates snake_case)
        assert!(re.is_match("fn MyFunction()"));
        // Should NOT match: fn my_function (follows snake_case)
        assert!(!re.is_match("fn my_function()"));
    }

    #[test]
    fn naming_regex_pascal_case() {
        let pattern = naming_regex_for_style("PascalCase");
        let re = regex::Regex::new(pattern).expect("valid regex");
        // Should match: struct lowercase (violates PascalCase)
        assert!(re.is_match("struct my_struct"));
        // Should NOT match: struct MyStruct (follows PascalCase)
        assert!(!re.is_match("struct MyStruct"));
    }
}
