//! Contract tests for WEB.2 — web-domain rule pack seed + schema v6 migration.
//!
//! Validates:
//!   (a) data/rules/web.toml exists, parses, and contains exactly 27 rules.
//!   (b) rule_ids match ^WEB-\d{3}$, are unique, cover WEB-001..WEB-027, and
//!       do not collide with data/rules/engineering.toml rule_ids.
//!   (c) All 27 predicates compile via DefaultRuleEngine::compile_predicate.
//!   (d) Every rule has domain="web" and phase_mask=24 (EXECUTING+AUDITING).
//!   (e) Tier distribution: 12 T1 + 11 T2 + 4 T3 (per MVP spec WEB.2).
//!   (f) SCHEMA_VERSION is 6 (bumped from 5 for the v5->v6 npm backfill).
//!   (g) Fresh open_rules_db seeds >= 6 npm deprecated_patterns.
//!   (h) A v5 database upgraded via run_migration backfills the 6 npm rows.
//!   (i) DOMAIN_KEYWORDS includes "web" — detect_domains on ".svelte" / ".astro"
//!       / ".html" paths returns a list containing "web".
//!   (j) detect_domains_from_frameworks(&ProjectProfile) emits "web" when a
//!       FrameworkCategory::Web framework is present; empty otherwise.
//!   (k) Adversarial T1 fixture: a string that looks like a banned pattern but
//!       lives in a comment must NOT match a tight T1 regex.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use elai_core::clock::SeededClock;
use elai_core::project_profile::{FrameworkCategory, FrameworkDetection};
use elai_rules::predicate::{
    AstError, AstEvaluator, AstMatch, DkgError, DkgQuerier, ReachabilityMatch, ReachabilityParams,
};
use elai_rules::rule_engine::{
    detect_domains, detect_domains_from_frameworks, DefaultRuleEngine, RuleEngine,
};
use elai_rules::schema::{open_rules_db, run_migration, schema_version, SCHEMA_VERSION};

// ---------------------------------------------------------------------------
// Expected counts (WEB.2 spec constants)
// ---------------------------------------------------------------------------

const EXPECTED_RULE_COUNT: usize = 27;
const EXPECTED_TIER_1_COUNT: usize = 12;
const EXPECTED_TIER_2_COUNT: usize = 11;
const EXPECTED_TIER_3_COUNT: usize = 4;
const EXPECTED_NPM_DEPRECATED_COUNT: i64 = 6;
const WEB_PHASE_MASK: u32 = 24;
const EXPECTED_SCHEMA_VERSION: i64 = 6;

// ---------------------------------------------------------------------------
// Stub evaluators (match rules_rule_engine_contract.rs pattern)
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

fn engine() -> DefaultRuleEngine {
    DefaultRuleEngine::new(Box::new(StubAstEvaluator), Box::new(StubDkgQuerier))
}

fn clock() -> SeededClock {
    SeededClock::new()
}

// ---------------------------------------------------------------------------
// Seed loader — manual toml::Value parsing to avoid pulling serde-derive into
// the contracts dev-deps. We only need field-level access; the full typed Rule
// struct is instantiated elsewhere (cli/src/main.rs::load_seed_rules).
// ---------------------------------------------------------------------------

struct SeedRuleView {
    rule_id: String,
    tier: i64,
    severity: String,
    domain: String,
    phase_mask: i64,
    predicate_json: String,
}

fn rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/rules")
}

fn load_rules(name: &str) -> Vec<SeedRuleView> {
    let path = rules_dir().join(name);
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    let doc: toml::Value = toml::from_str(&body)
        .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e));
    let arr = doc
        .get("rules")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("{} missing [[rules]] array", path.display()));
    arr.iter()
        .enumerate()
        .map(|(i, v)| SeedRuleView {
            rule_id: v
                .get("rule_id")
                .and_then(|x| x.as_str())
                .unwrap_or_else(|| panic!("rule[{}] missing rule_id in {}", i, path.display()))
                .to_owned(),
            tier: v
                .get("tier")
                .and_then(|x| x.as_integer())
                .unwrap_or_else(|| panic!("rule[{}] missing tier in {}", i, path.display())),
            severity: v
                .get("severity")
                .and_then(|x| x.as_str())
                .unwrap_or_else(|| panic!("rule[{}] missing severity in {}", i, path.display()))
                .to_owned(),
            domain: v
                .get("domain")
                .and_then(|x| x.as_str())
                .unwrap_or_else(|| panic!("rule[{}] missing domain in {}", i, path.display()))
                .to_owned(),
            phase_mask: v
                .get("phase_mask")
                .and_then(|x| x.as_integer())
                .unwrap_or_else(|| panic!("rule[{}] missing phase_mask in {}", i, path.display())),
            predicate_json: v
                .get("predicate_json")
                .and_then(|x| x.as_str())
                .unwrap_or_else(|| panic!("rule[{}] missing predicate_json in {}", i, path.display()))
                .to_owned(),
        })
        .collect()
}

fn is_web_id(id: &str) -> bool {
    id.len() == 7
        && id.starts_with("WEB-")
        && id.as_bytes()[4..].iter().all(u8::is_ascii_digit)
}

// ---------------------------------------------------------------------------
// Rule-pack tests
// ---------------------------------------------------------------------------

#[test]
fn web_toml_exists_and_has_expected_rule_count() {
    let rules = load_rules("web.toml");
    assert_eq!(
        rules.len(),
        EXPECTED_RULE_COUNT,
        "WEB.2 spec: 12 T1 + 11 T2 + 4 T3 = 27"
    );
}

#[test]
fn web_toml_tier_distribution_matches_spec() {
    let rules = load_rules("web.toml");
    let t1 = rules.iter().filter(|r| r.tier == 1).count();
    let t2 = rules.iter().filter(|r| r.tier == 2).count();
    let t3 = rules.iter().filter(|r| r.tier == 3).count();
    assert_eq!(t1, EXPECTED_TIER_1_COUNT, "tier 1 count");
    assert_eq!(t2, EXPECTED_TIER_2_COUNT, "tier 2 count");
    assert_eq!(t3, EXPECTED_TIER_3_COUNT, "tier 3 count");
}

#[test]
fn web_toml_rule_ids_are_unique_and_prefixed() {
    let rules = load_rules("web.toml");
    let mut ids: HashSet<String> = HashSet::new();
    for rule in &rules {
        assert!(
            is_web_id(&rule.rule_id),
            "rule_id {} does not match WEB-NNN",
            rule.rule_id
        );
        assert!(
            ids.insert(rule.rule_id.clone()),
            "duplicate rule_id {}",
            rule.rule_id
        );
    }
    assert_eq!(ids.len(), EXPECTED_RULE_COUNT);
}

#[test]
fn web_toml_rule_ids_do_not_collide_with_engineering() {
    let web = load_rules("web.toml");
    let eng = load_rules("engineering.toml");
    let web_ids: HashSet<&str> = web.iter().map(|r| r.rule_id.as_str()).collect();
    let eng_ids: HashSet<&str> = eng.iter().map(|r| r.rule_id.as_str()).collect();
    let overlap: Vec<&&str> = web_ids.intersection(&eng_ids).collect();
    assert!(
        overlap.is_empty(),
        "rule_id collision between web.toml and engineering.toml: {:?}",
        overlap
    );
}

#[test]
fn web_toml_all_domains_and_phase_masks_are_canonical() {
    let rules = load_rules("web.toml");
    let expected_mask = WEB_PHASE_MASK as i64;
    for rule in &rules {
        assert_eq!(rule.domain, "web", "rule {} domain must be 'web'", rule.rule_id);
        assert_eq!(
            rule.phase_mask, expected_mask,
            "rule {} phase_mask must be 24 (EXECUTING+AUDITING)",
            rule.rule_id
        );
    }
}

#[test]
fn web_toml_all_severities_are_valid() {
    let rules = load_rules("web.toml");
    for rule in &rules {
        let sev = rule.severity.as_str();
        assert!(
            matches!(sev, "BLOCK" | "WARN" | "INFO"),
            "rule {} severity {} not in {{BLOCK,WARN,INFO}}",
            rule.rule_id,
            sev
        );
    }
}

#[test]
fn web_toml_all_predicates_compile() {
    let rules = load_rules("web.toml");
    let eng = engine();
    for rule in &rules {
        eng.compile_predicate(&rule.predicate_json)
            .unwrap_or_else(|e| panic!("rule {} failed to compile: {}", rule.rule_id, e));
    }
}

// ---------------------------------------------------------------------------
// Schema v6 migration + npm backfill tests
// ---------------------------------------------------------------------------

#[test]
fn schema_version_constant_is_v6() {
    assert_eq!(
        SCHEMA_VERSION, EXPECTED_SCHEMA_VERSION,
        "WEB.2 bumps schema to v6"
    );
}

#[test]
fn fresh_db_seeds_six_npm_deprecated_patterns() {
    let conn = open_rules_db(Path::new(":memory:"), &clock())
        .expect("in-memory DB should open");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM deprecated_patterns WHERE ecosystem = 'npm'",
            [],
            |row| row.get(0),
        )
        .expect("count npm patterns");
    assert!(
        count >= EXPECTED_NPM_DEPRECATED_COUNT,
        "fresh DB must seed >= 6 npm rows, got {}",
        count
    );
}

#[test]
fn v5_db_upgrades_to_v6_and_backfills_npm() {
    // Build a DB in the state schema v5 would leave it in: all tables present,
    // schema_version table records max(version)==5, BUT deprecated_patterns
    // has no npm rows yet (simulating a DB that was migrated before v6).
    let conn = open_rules_db(Path::new(":memory:"), &clock())
        .expect("in-memory DB should open");

    // Simulate a pre-v6 DB: delete all npm rows and roll schema_version back to 5.
    conn.execute(
        "DELETE FROM deprecated_patterns WHERE ecosystem = 'npm'",
        [],
    )
    .expect("delete npm rows");
    conn.execute("DELETE FROM schema_version WHERE version >= 6", [])
        .expect("roll version back");

    // Sanity: npm count is now 0.
    let before: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM deprecated_patterns WHERE ecosystem = 'npm'",
            [],
            |row| row.get(0),
        )
        .expect("count before");
    assert_eq!(before, 0, "setup: npm rows should be purged");

    // Pre-check: recorded schema version is <= 5.
    assert!(
        schema_version(&conn).expect("read version") <= 5,
        "setup: schema version should be <= 5"
    );

    // Run migration — v5 -> v6 must backfill npm rows via seed_deprecated_patterns.
    run_migration(&conn, &clock()).expect("v5->v6 migration must succeed");

    let after: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM deprecated_patterns WHERE ecosystem = 'npm'",
            [],
            |row| row.get(0),
        )
        .expect("count after");
    assert!(
        after >= EXPECTED_NPM_DEPRECATED_COUNT,
        "v5->v6 migration must backfill >= 6 npm rows, got {}",
        after
    );
    assert_eq!(
        schema_version(&conn).expect("read version"),
        EXPECTED_SCHEMA_VERSION,
        "post-migration schema version must be v6"
    );
}

#[test]
fn npm_deprecated_entries_include_expected_names() {
    let conn = open_rules_db(Path::new(":memory:"), &clock())
        .expect("in-memory DB should open");
    let names: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT deprecated_name FROM deprecated_patterns WHERE ecosystem = 'npm'",
            )
            .expect("prepare npm select");
        let iter = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query npm");
        iter.filter_map(|r| r.ok()).collect()
    };
    for required in ["moment", "request", "create-react-app", "enzyme", "webpack", "jquery"] {
        assert!(
            names.iter().any(|n| n == required),
            "missing npm deprecated entry '{}'; have {:?}",
            required,
            names
        );
    }
}

// ---------------------------------------------------------------------------
// Domain activation tests
// ---------------------------------------------------------------------------

#[test]
fn detect_domains_returns_web_for_svelte_path() {
    let domains = detect_domains("src/components/Hero.svelte");
    assert!(
        domains.contains(&"web".to_string()),
        "svelte path should detect web domain; got {:?}",
        domains
    );
}

#[test]
fn detect_domains_returns_web_for_astro_path() {
    let domains = detect_domains("src/pages/index.astro");
    assert!(
        domains.contains(&"web".to_string()),
        "astro path should detect web domain; got {:?}",
        domains
    );
}

#[test]
fn detect_domains_returns_web_for_html_path() {
    let domains = detect_domains("public/index.html");
    assert!(
        domains.contains(&"web".to_string()),
        "html path should detect web domain; got {:?}",
        domains
    );
}

fn fw(name: &str, category: FrameworkCategory) -> FrameworkDetection {
    FrameworkDetection {
        name: name.to_owned(),
        version: None,
        category,
        confidence: 0.9,
        detected_via: "test".to_owned(),
    }
}

#[test]
fn detect_domains_from_frameworks_emits_web_for_web_category() {
    let frameworks = vec![fw("Axum", FrameworkCategory::Web)];
    let out = detect_domains_from_frameworks(&frameworks);
    assert!(
        out.contains(&"web".to_string()),
        "FrameworkCategory::Web must emit 'web'; got {:?}",
        out
    );
}

#[test]
fn detect_domains_from_frameworks_is_empty_without_web_framework() {
    let frameworks = vec![fw("Bevy", FrameworkCategory::Game)];
    let out = detect_domains_from_frameworks(&frameworks);
    assert!(
        !out.contains(&"web".to_string()),
        "Game-only profile must NOT emit 'web'; got {:?}",
        out
    );
}

#[test]
fn detect_domains_from_frameworks_empty_profile_empty_output() {
    let frameworks: Vec<FrameworkDetection> = Vec::new();
    let out = detect_domains_from_frameworks(&frameworks);
    assert!(out.is_empty(), "empty frameworks -> empty domains; got {:?}", out);
}

#[test]
fn detect_domains_from_frameworks_is_deduplicated() {
    let frameworks = vec![
        fw("Axum", FrameworkCategory::Web),
        fw("Actix", FrameworkCategory::Web),
    ];
    let out = detect_domains_from_frameworks(&frameworks);
    let web_count = out.iter().filter(|d| d.as_str() == "web").count();
    assert_eq!(
        web_count, 1,
        "multiple Web frameworks must collapse to a single 'web' entry; got {:?}",
        out
    );
}

// ---------------------------------------------------------------------------
// Adversarial fixture: WEB-005 (document.write ban) must match an actual call
// and must NOT fire on a safe string literal mentioning the name.
// ---------------------------------------------------------------------------

#[test]
fn web_005_document_write_regex_matches_call_but_not_mention() {
    let rules = load_rules("web.toml");
    let eng = engine();
    let web_005 = rules
        .iter()
        .find(|r| r.rule_id == "WEB-005")
        .expect("WEB-005 must exist in web.toml");

    // Parse and recompile via the engine to validate the JSON + regex pipeline.
    let compiled = eng
        .compile_predicate(&web_005.predicate_json)
        .expect("WEB-005 predicate compiles");
    let regex = match compiled {
        elai_rules::rule_engine::CompiledPredicate::GrepPattern { compiled_regex, .. } => {
            compiled_regex
        }
        _ => panic!("WEB-005 must be a GrepPattern"),
    };

    // Positive: actual call fires.
    let positive = "function init() { document.write('<h1>hi</h1>'); }";
    assert!(
        regex.is_match(positive),
        "WEB-005 must fire on an actual document.write call"
    );

    // Negative: a doc-string mention without the call must NOT fire.
    let negative = "// Do not use the old document write technique in components.";
    assert!(
        !regex.is_match(negative),
        "WEB-005 must NOT fire on a comment mentioning 'document write' without a call"
    );
}

