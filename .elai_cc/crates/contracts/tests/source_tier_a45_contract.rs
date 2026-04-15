//! Contract tests for A-45 SOURCE TIER CONFIG (elai-core).
//! Validates SourceTier classification invariants at the interface level.

use elai_core::{
    classify_source, load_source_tier_config, parse_source_tier_config, ClassificationMethod,
    SourceTier, SourceTierConfig, SourceTierError,
};
use std::path::Path;

/// Minimal config for hermetic tests (no filesystem dependency).
const HERMETIC_CONFIG: &str = r#"
[tier1]
domains = ["doc.rust-lang.org", "docs.python.org", "arxiv.org"]
patterns = ['\.gov$', '\.edu$', '^docs\.']

[tier2]
domains = ["stackoverflow.com", "realpython.com"]
min_so_score = 20

[tier4_blocklist]
domains = ["w3schools.com", "tutorialspoint.com"]
patterns = ['seo-', '-tutorial\.com$']
"#;

/// Minimum domain counts expected from the real source-tiers.toml.
const MIN_T1_DOMAINS: usize = 30;
const MIN_T2_DOMAINS: usize = 20;

fn hermetic_config() -> SourceTierConfig {
    parse_source_tier_config(HERMETIC_CONFIG).expect("hermetic config must parse")
}

// --- (a) T4 priority invariant: domain in both T1 and T4 must classify as T4 ---

#[test]
fn t4_priority_overrides_t1_when_domain_in_both_tiers() {
    let config_str = r#"
[tier1]
domains = ["dual-listed.example.com"]
patterns = []

[tier2]
domains = []
min_so_score = 0

[tier4_blocklist]
domains = ["dual-listed.example.com"]
patterns = []
"#;
    let cfg = parse_source_tier_config(config_str).expect("parse");
    let result = classify_source("https://dual-listed.example.com/page", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T4Rejected, "T4 must take priority over T1");
    assert_eq!(result.classification_method, ClassificationMethod::ExactDomain);
}

// --- (a2) T4 pattern priority overrides T1 exact domain ---

#[test]
fn t4_pattern_overrides_t1_exact_domain() {
    let config_str = r#"
[tier1]
domains = ["seo-legit.example.com"]
patterns = []

[tier2]
domains = []
min_so_score = 0

[tier4_blocklist]
domains = []
patterns = ['seo-']
"#;
    let cfg = parse_source_tier_config(config_str).expect("parse");
    let result = classify_source("https://seo-legit.example.com/page", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T4Rejected, "T4 pattern must override T1 exact domain");
    assert_eq!(result.classification_method, ClassificationMethod::Pattern);
}

// --- (b) Exact domain T1 ---

#[test]
fn classify_t1_exact_domain() {
    let cfg = hermetic_config();
    let result = classify_source("https://doc.rust-lang.org/std/index.html", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T1Authoritative);
    assert_eq!(result.classification_method, ClassificationMethod::ExactDomain);
    assert_eq!(result.domain, "doc.rust-lang.org");
}

// --- (c) Exact domain T2 ---

#[test]
fn classify_t2_exact_domain() {
    let cfg = hermetic_config();
    let result = classify_source("https://stackoverflow.com/questions/12345", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T2Strong);
    assert_eq!(result.classification_method, ClassificationMethod::ExactDomain);
}

// --- (d) Exact domain T4 ---

#[test]
fn classify_t4_exact_domain() {
    let cfg = hermetic_config();
    let result = classify_source("https://w3schools.com/html/tutorial.html", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T4Rejected);
    assert_eq!(result.classification_method, ClassificationMethod::ExactDomain);
}

// --- (e) Default T3 for unknown domain ---

#[test]
fn classify_unknown_domain_defaults_to_t3() {
    let cfg = hermetic_config();
    let result = classify_source("https://randomsite.xyz/article", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T3Weak);
    assert_eq!(result.classification_method, ClassificationMethod::Default);
}

// --- (f) Pattern T1 ---

#[test]
fn classify_t1_via_gov_pattern() {
    let cfg = hermetic_config();
    let result = classify_source("https://www.nist.gov/cybersecurity", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T1Authoritative);
    assert_eq!(result.classification_method, ClassificationMethod::Pattern);
}

#[test]
fn classify_t1_via_edu_pattern() {
    let cfg = hermetic_config();
    let result = classify_source("https://cs.stanford.edu/papers", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T1Authoritative);
    assert_eq!(result.classification_method, ClassificationMethod::Pattern);
}

// --- (g) Pattern T4 ---

#[test]
fn classify_t4_via_blocklist_pattern() {
    let cfg = hermetic_config();
    let result = classify_source("https://seo-content.example.com/stuff", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T4Rejected);
    assert_eq!(result.classification_method, ClassificationMethod::Pattern);
}

#[test]
fn classify_t4_via_tutorial_suffix_pattern() {
    let cfg = hermetic_config();
    let result = classify_source("https://java-tutorial.com/basics", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T4Rejected);
    assert_eq!(result.classification_method, ClassificationMethod::Pattern);
}

// --- (h) Case insensitivity ---

#[test]
fn classify_is_case_insensitive() {
    let cfg = hermetic_config();
    let result = classify_source("https://DOC.RUST-LANG.ORG/std/vec", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T1Authoritative);
    assert_eq!(result.domain, "doc.rust-lang.org", "domain must be lowercased");
}

// --- (i) Empty config: parse Ok, unknown URL -> T3 ---

#[test]
fn empty_config_parses_ok_and_defaults_to_t3() {
    let cfg = parse_source_tier_config("").expect("empty config must parse to Ok");
    assert_eq!(cfg.tier1_domain_count(), 0);
    assert_eq!(cfg.tier2_domain_count(), 0);
    assert_eq!(cfg.tier4_domain_count(), 0);
    let result = classify_source("https://example.com/page", &cfg).expect("classify");
    assert_eq!(result.tier, SourceTier::T3Weak);
    assert_eq!(result.classification_method, ClassificationMethod::Default);
}

// --- (j) Invalid URL ---

#[test]
fn classify_invalid_url_returns_error() {
    let cfg = hermetic_config();
    let err = classify_source("not-a-url", &cfg);
    assert!(err.is_err(), "bare string without dot should fail");
    let error = err.unwrap_err();
    let msg = error.to_string();
    assert!(
        msg.contains("invalid URL"),
        "error message should mention invalid URL, got: {msg}"
    );
}

// --- (k) Error type Send+Sync (compile-time check) ---

#[test]
fn source_tier_error_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SourceTierError>();
}

// --- (l) Domain count sanity from real source-tiers.toml ---

#[test]
fn real_config_has_minimum_domain_counts() {
    let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("config")
        .join("source-tiers.toml");
    let cfg = load_source_tier_config(&config_path)
        .unwrap_or_else(|e| panic!("failed to load source-tiers.toml from {}: {e}", config_path.display()));
    assert!(
        cfg.tier1_domain_count() >= MIN_T1_DOMAINS,
        "T1 domain count {} is below minimum {MIN_T1_DOMAINS}",
        cfg.tier1_domain_count()
    );
    assert!(
        cfg.tier2_domain_count() >= MIN_T2_DOMAINS,
        "T2 domain count {} is below minimum {MIN_T2_DOMAINS}",
        cfg.tier2_domain_count()
    );
}

// --- Bonus: SourceClassification preserves original URL ---

#[test]
fn classification_preserves_original_url_casing() {
    let cfg = hermetic_config();
    let url = "https://DOC.RUST-LANG.ORG/std?q=Test";
    let result = classify_source(url, &cfg).expect("classify");
    assert_eq!(result.url, url, "original URL must be preserved verbatim");
    assert_eq!(result.domain, "doc.rust-lang.org", "domain must be lowercased");
}
