// A-45: Source tier classification for Oracle research pipeline.
// Classifies URLs into 4 reliability tiers using a config-driven lookup table.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Source reliability tier (T1 = most authoritative, T4 = rejected).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceTier {
    /// Official docs, RFCs, standards bodies, peer-reviewed papers, primary sources.
    T1Authoritative,
    /// High-quality community: SO high-score, reputable blogs, established authors.
    T2Strong,
    /// Unknown/unclassified domain (implicit default).
    T3Weak,
    /// Blocklisted: SEO farms, AI content farms, unreliable aggregators.
    T4Rejected,
}

/// How a source was classified (for auditability).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClassificationMethod {
    /// Exact domain match against tier list.
    ExactDomain,
    /// Regex pattern match against tier patterns.
    Pattern,
    /// No match found — assigned T3 by default.
    Default,
}

/// Result of classifying a single URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceClassification {
    pub url: String,
    pub domain: String,
    pub tier: SourceTier,
    pub classification_method: ClassificationMethod,
}

/// Deserialized tier section from source-tiers.toml.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
struct TierSection {
    domains: Vec<String>,
    patterns: Vec<String>,
}

/// Deserialized tier2 section (has min_so_score field).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
struct Tier2Section {
    domains: Vec<String>,
    min_so_score: u32,
}

/// Deserialized blocklist section.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
struct Tier4Section {
    domains: Vec<String>,
    patterns: Vec<String>,
}

/// Raw TOML structure for source-tiers.toml.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
struct RawSourceTierToml {
    tier1: TierSection,
    tier2: Tier2Section,
    tier4_blocklist: Tier4Section,
}

/// Compiled tier configuration loaded from source-tiers.toml.
/// Domains stored as HashSets for O(1) lookup. Patterns compiled to regexes.
#[derive(Debug, Clone)]
pub struct SourceTierConfig {
    tier1_domains: std::collections::HashSet<String>,
    tier1_patterns: Vec<regex::Regex>,
    tier2_domains: std::collections::HashSet<String>,
    tier2_min_so_score: u32,
    tier4_domains: std::collections::HashSet<String>,
    tier4_patterns: Vec<regex::Regex>,
}

/// Errors from source tier operations.
#[derive(Debug, thiserror::Error)]
pub enum SourceTierError {
    #[error("failed to read source tier config: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse source tier config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid regex pattern '{pattern}': {source}")]
    InvalidPattern {
        pattern: String,
        source: regex::Error,
    },
    #[error("invalid URL (no domain extractable): {0}")]
    InvalidUrl(String),
}

/// Compile a list of pattern strings into regex objects.
fn compile_patterns(patterns: &[String]) -> Result<Vec<regex::Regex>, SourceTierError> {
    patterns
        .iter()
        .map(|p| {
            regex::Regex::new(p).map_err(|e| SourceTierError::InvalidPattern {
                pattern: p.clone(),
                source: e,
            })
        })
        .collect()
}

/// Load and compile source tier configuration from a TOML file.
pub fn load_source_tier_config(path: &Path) -> Result<SourceTierConfig, SourceTierError> {
    let content = std::fs::read_to_string(path)?;
    parse_source_tier_config(&content)
}

/// Parse and compile source tier configuration from a TOML string.
/// Useful for testing without filesystem access.
pub fn parse_source_tier_config(content: &str) -> Result<SourceTierConfig, SourceTierError> {
    let raw: RawSourceTierToml = toml::from_str(content)?;

    let tier1_patterns = compile_patterns(&raw.tier1.patterns)?;
    let tier4_patterns = compile_patterns(&raw.tier4_blocklist.patterns)?;

    Ok(SourceTierConfig {
        tier1_domains: raw.tier1.domains.into_iter().collect(),
        tier1_patterns,
        tier2_domains: raw.tier2.domains.into_iter().collect(),
        tier2_min_so_score: raw.tier2.min_so_score,
        tier4_domains: raw.tier4_blocklist.domains.into_iter().collect(),
        tier4_patterns,
    })
}

/// Extract the domain from a URL string.
/// Handles both `https://example.com/path` and bare `example.com/path` forms.
fn extract_domain(url: &str) -> Result<String, SourceTierError> {
    // Strip scheme if present.
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    // Take everything before the first '/'.
    let host = without_scheme.split('/').next().unwrap_or(without_scheme);

    // Strip port if present.
    let domain = if let Some(idx) = host.rfind(':') {
        // Only strip if what follows the colon looks like a port number.
        let after_colon = &host[idx + 1..];
        if after_colon.chars().all(|c| c.is_ascii_digit()) && !after_colon.is_empty() {
            &host[..idx]
        } else {
            host
        }
    } else {
        host
    };

    let domain = domain.trim().to_lowercase();
    if domain.is_empty() || !domain.contains('.') {
        return Err(SourceTierError::InvalidUrl(url.to_owned()));
    }
    Ok(domain)
}

/// Check whether any compiled regex in `patterns` matches `domain`.
fn matches_any_pattern(patterns: &[regex::Regex], domain: &str) -> bool {
    patterns.iter().any(|re| re.is_match(domain))
}

/// Classify a URL against the loaded tier configuration.
///
/// Priority: T4 blocklist (exact then pattern) -> T1 (exact then pattern) -> T2 (exact) -> T3 default.
/// T4 is checked first so blocklisted domains cannot be accidentally promoted.
pub fn classify_source(
    url: &str,
    config: &SourceTierConfig,
) -> Result<SourceClassification, SourceTierError> {
    let domain = extract_domain(url)?;

    // T4 blocklist: exact domain match.
    if config.tier4_domains.contains(&domain) {
        return Ok(SourceClassification {
            url: url.to_owned(),
            domain,
            tier: SourceTier::T4Rejected,
            classification_method: ClassificationMethod::ExactDomain,
        });
    }

    // T4 blocklist: pattern match.
    if matches_any_pattern(&config.tier4_patterns, &domain) {
        return Ok(SourceClassification {
            url: url.to_owned(),
            domain,
            tier: SourceTier::T4Rejected,
            classification_method: ClassificationMethod::Pattern,
        });
    }

    // T1 authoritative: exact domain match.
    if config.tier1_domains.contains(&domain) {
        return Ok(SourceClassification {
            url: url.to_owned(),
            domain,
            tier: SourceTier::T1Authoritative,
            classification_method: ClassificationMethod::ExactDomain,
        });
    }

    // T1 authoritative: pattern match.
    if matches_any_pattern(&config.tier1_patterns, &domain) {
        return Ok(SourceClassification {
            url: url.to_owned(),
            domain,
            tier: SourceTier::T1Authoritative,
            classification_method: ClassificationMethod::Pattern,
        });
    }

    // T2 strong: exact domain match.
    if config.tier2_domains.contains(&domain) {
        return Ok(SourceClassification {
            url: url.to_owned(),
            domain,
            tier: SourceTier::T2Strong,
            classification_method: ClassificationMethod::ExactDomain,
        });
    }

    // T3 default: nothing matched.
    Ok(SourceClassification {
        url: url.to_owned(),
        domain,
        tier: SourceTier::T3Weak,
        classification_method: ClassificationMethod::Default,
    })
}

impl SourceTierConfig {
    /// Minimum StackOverflow score for T2 classification (informational, not enforced in URL classification).
    pub fn min_so_score(&self) -> u32 {
        self.tier2_min_so_score
    }

    /// Number of T1 domains loaded.
    pub fn tier1_domain_count(&self) -> usize {
        self.tier1_domains.len()
    }

    /// Number of T2 domains loaded.
    pub fn tier2_domain_count(&self) -> usize {
        self.tier2_domains.len()
    }

    /// Number of T4 blocklisted domains loaded.
    pub fn tier4_domain_count(&self) -> usize {
        self.tier4_domains.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CONFIG: &str = r#"
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

    fn test_config() -> SourceTierConfig {
        parse_source_tier_config(TEST_CONFIG).expect("test config must parse")
    }

    // --- Contract: config loading ---

    #[test]
    fn load_valid_config() {
        let cfg = test_config();
        assert_eq!(cfg.tier1_domain_count(), 3);
        assert_eq!(cfg.tier2_domain_count(), 2);
        assert_eq!(cfg.tier4_domain_count(), 2);
        assert_eq!(cfg.min_so_score(), 20);
    }

    #[test]
    fn empty_config_defaults_all_to_t3() {
        let cfg = parse_source_tier_config("").expect("empty config must parse");
        assert_eq!(cfg.tier1_domain_count(), 0);
        let result = classify_source("https://example.com/page", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T3Weak);
        assert_eq!(result.classification_method, ClassificationMethod::Default);
    }

    #[test]
    fn invalid_pattern_returns_error() {
        let bad = r#"
[tier1]
domains = []
patterns = ['[invalid']
"#;
        let err = parse_source_tier_config(bad);
        assert!(err.is_err());
        let msg = format!("{}", err.unwrap_err());
        assert!(msg.contains("invalid regex"));
    }

    // --- Contract: domain extraction ---

    #[test]
    fn extract_domain_with_scheme() {
        assert_eq!(
            extract_domain("https://doc.rust-lang.org/book/ch01.html").unwrap(),
            "doc.rust-lang.org"
        );
    }

    #[test]
    fn extract_domain_without_scheme() {
        assert_eq!(
            extract_domain("doc.rust-lang.org/book").unwrap(),
            "doc.rust-lang.org"
        );
    }

    #[test]
    fn extract_domain_with_port() {
        assert_eq!(
            extract_domain("https://localhost:8080/path").unwrap_err().to_string(),
            "invalid URL (no domain extractable): https://localhost:8080/path"
        );
    }

    #[test]
    fn extract_domain_strips_port_real_domain() {
        assert_eq!(
            extract_domain("https://example.com:443/page").unwrap(),
            "example.com"
        );
    }

    #[test]
    fn extract_domain_empty_rejects() {
        assert!(extract_domain("").is_err());
    }

    #[test]
    fn extract_domain_no_dot_rejects() {
        assert!(extract_domain("localhost").is_err());
    }

    // --- Contract: T1 classification ---

    #[test]
    fn t1_exact_domain_match() {
        let cfg = test_config();
        let result =
            classify_source("https://doc.rust-lang.org/std/index.html", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T1Authoritative);
        assert_eq!(result.classification_method, ClassificationMethod::ExactDomain);
        assert_eq!(result.domain, "doc.rust-lang.org");
    }

    #[test]
    fn t1_pattern_gov() {
        let cfg = test_config();
        let result =
            classify_source("https://www.nist.gov/cybersecurity", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T1Authoritative);
        assert_eq!(result.classification_method, ClassificationMethod::Pattern);
    }

    #[test]
    fn t1_pattern_edu() {
        let cfg = test_config();
        let result =
            classify_source("https://cs.stanford.edu/papers", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T1Authoritative);
        assert_eq!(result.classification_method, ClassificationMethod::Pattern);
    }

    #[test]
    fn t1_pattern_docs_prefix() {
        let cfg = test_config();
        let result =
            classify_source("https://docs.somecrate.io/guide", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T1Authoritative);
        assert_eq!(result.classification_method, ClassificationMethod::Pattern);
    }

    // --- Contract: T2 classification ---

    #[test]
    fn t2_exact_domain_match() {
        let cfg = test_config();
        let result =
            classify_source("https://stackoverflow.com/questions/12345", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T2Strong);
        assert_eq!(result.classification_method, ClassificationMethod::ExactDomain);
    }

    // --- Contract: T3 default ---

    #[test]
    fn t3_unknown_domain() {
        let cfg = test_config();
        let result =
            classify_source("https://randomsite.xyz/article", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T3Weak);
        assert_eq!(result.classification_method, ClassificationMethod::Default);
    }

    // --- Contract: T4 blocklist ---

    #[test]
    fn t4_exact_domain_match() {
        let cfg = test_config();
        let result =
            classify_source("https://w3schools.com/html", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T4Rejected);
        assert_eq!(result.classification_method, ClassificationMethod::ExactDomain);
    }

    #[test]
    fn t4_pattern_match() {
        let cfg = test_config();
        let result =
            classify_source("https://seo-content.example.com/stuff", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T4Rejected);
        assert_eq!(result.classification_method, ClassificationMethod::Pattern);
    }

    #[test]
    fn t4_pattern_tutorial_suffix() {
        let cfg = test_config();
        let result =
            classify_source("https://java-tutorial.com/basics", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T4Rejected);
        assert_eq!(result.classification_method, ClassificationMethod::Pattern);
    }

    // --- Contract: T4 takes priority over T1 ---

    #[test]
    fn t4_overrides_t1_if_both_match() {
        // Config where a domain is in both T1 and T4 — T4 must win.
        let cfg_str = r#"
[tier1]
domains = ["evil-but-official.com"]
patterns = []

[tier2]
domains = []
min_so_score = 0

[tier4_blocklist]
domains = ["evil-but-official.com"]
patterns = []
"#;
        let cfg = parse_source_tier_config(cfg_str).expect("parse");
        let result =
            classify_source("https://evil-but-official.com/page", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T4Rejected);
    }

    // --- Contract: case insensitivity ---

    #[test]
    fn classification_is_case_insensitive() {
        let cfg = test_config();
        let result =
            classify_source("https://DOC.RUST-LANG.ORG/std", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T1Authoritative);
    }

    // --- Contract: URL with trailing slash ---

    #[test]
    fn url_with_trailing_slash() {
        let cfg = test_config();
        let result =
            classify_source("https://arxiv.org/", &cfg).expect("classify");
        assert_eq!(result.tier, SourceTier::T1Authoritative);
    }

    // --- Contract: SourceClassification preserves original URL ---

    #[test]
    fn preserves_original_url() {
        let cfg = test_config();
        let url = "https://DOC.RUST-LANG.ORG/std?q=test";
        let result = classify_source(url, &cfg).expect("classify");
        assert_eq!(result.url, url);
        assert_eq!(result.domain, "doc.rust-lang.org");
    }

    // --- Contract: serde round-trip ---

    #[test]
    fn source_tier_serde_roundtrip() {
        let tier = SourceTier::T1Authoritative;
        let json = serde_json::to_string(&tier).expect("serialize");
        let back: SourceTier = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(tier, back);
    }

    #[test]
    fn source_classification_serde_roundtrip() {
        let sc = SourceClassification {
            url: "https://doc.rust-lang.org".to_owned(),
            domain: "doc.rust-lang.org".to_owned(),
            tier: SourceTier::T1Authoritative,
            classification_method: ClassificationMethod::ExactDomain,
        };
        let json = serde_json::to_string(&sc).expect("serialize");
        let back: SourceClassification = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(sc, back);
    }
}
