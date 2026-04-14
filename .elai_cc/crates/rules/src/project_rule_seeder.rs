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
            .map(|d| d.as_secs() as i64)
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
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
                distribution: std::collections::HashMap::new(),
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
            project_vision: None,
            architecture: None,
            domains: Vec::new(),
            commands: BuildCommandRegistry::default(),
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
