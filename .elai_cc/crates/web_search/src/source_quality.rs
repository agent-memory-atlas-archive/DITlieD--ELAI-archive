// 4D.13: Source quality table — trusted domain lookup for Oracle research pipeline.
// Wraps elai_core::SourceTierConfig to provide domain-to-tier lookup and f64 scoring.

use elai_core::source_tier::{
    self, SourceTier, SourceTierConfig, SourceTierError,
};
use serde::Deserialize;
use std::path::Path;

// Named constants for source quality scores.
// Match research_tree.rs SOURCE_QUALITY_TIER1..TIER4 for the 4 SourceTier variants.
// SCORE_UNKNOWN (0.2) is exported for callers bridging to the 5-tier SourceQuality system
// (research_tree Tier5) but is not produced by SourceQualityTable::score() since
// SourceTier has no 5th variant — unknown domains default to T3Weak (0.6).
pub const SCORE_TIER1: f64 = 1.0;
pub const SCORE_TIER2: f64 = 0.8;
pub const SCORE_TIER3: f64 = 0.6;
pub const SCORE_TIER4: f64 = 0.4;
pub const SCORE_UNKNOWN: f64 = 0.2;

/// Errors from source quality operations.
#[derive(Debug, thiserror::Error)]
pub enum SourceQualityError {
    #[error("I/O error reading config: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("source tier error: {0}")]
    TierError(#[from] SourceTierError),
    #[error("invalid config: {reason}")]
    InvalidConfig { reason: String },
}

/// Raw TOML structure for [research.trusted_domains] in elai_cc.toml.
#[derive(Debug, Deserialize)]
struct ElaiCcToml {
    #[serde(default)]
    research: ResearchSection,
}

#[derive(Debug, Default, Deserialize)]
struct ResearchSection {
    #[serde(default)]
    trusted_domains: TrustedDomainsConfig,
}

#[derive(Debug, Default, Deserialize)]
struct TrustedDomainsConfig {
    #[serde(default)]
    tier1: Vec<String>,
    #[serde(default)]
    tier1_patterns: Vec<String>,
    #[serde(default)]
    tier2: Vec<String>,
    #[serde(default)]
    tier2_min_so_score: Option<u32>,
    #[serde(default)]
    tier4: Vec<String>,
    #[serde(default)]
    tier4_patterns: Vec<String>,
}

/// Deterministic, read-only domain quality lookup table.
/// Wraps SourceTierConfig for O(1) domain classification.
/// Constructed once, used for the process lifetime. No async state, no I/O in lookups.
pub struct SourceQualityTable {
    config: SourceTierConfig,
}

impl SourceQualityTable {
    /// Construct from an existing SourceTierConfig (no I/O).
    /// For testing and Oracle mode in-memory initialization.
    pub fn new_from_config(config: SourceTierConfig) -> Self {
        Self { config }
    }

    /// Load from elai_cc.toml [research.trusted_domains] section.
    /// Validates domain counts and pattern compilation after deserialization.
    pub fn new_from_elai_cc_toml(path: &str) -> Result<Self, SourceQualityError> {
        let content = std::fs::read_to_string(Path::new(path))?;
        let parsed: ElaiCcToml = toml::from_str(&content)?;
        let td = parsed.research.trusted_domains;

        Self::validate_config(&td)?;

        // Synthesize a source-tiers.toml-compatible TOML string for parse_source_tier_config.
        let synth = Self::synthesize_tier_toml(&td);
        let config = source_tier::parse_source_tier_config(&synth)?;
        Ok(Self { config })
    }

    /// Validate the deserialized config before constructing the tier config.
    fn validate_config(td: &TrustedDomainsConfig) -> Result<(), SourceQualityError> {
        let total = td.tier1.len() + td.tier2.len() + td.tier4.len();
        if total == 0 {
            return Err(SourceQualityError::InvalidConfig {
                reason: "trusted_domains contains zero domains".to_owned(),
            });
        }

        // Check for empty domain strings.
        let has_empty = td
            .tier1
            .iter()
            .chain(td.tier2.iter())
            .chain(td.tier4.iter())
            .any(|d| d.trim().is_empty());
        if has_empty {
            return Err(SourceQualityError::InvalidConfig {
                reason: "trusted_domains contains empty domain string".to_owned(),
            });
        }

        Ok(())
    }

    /// Synthesize a source-tiers.toml-format string from the trusted_domains config.
    fn synthesize_tier_toml(td: &TrustedDomainsConfig) -> String {
        let mut out = String::with_capacity(4096);

        out.push_str("[tier1]\ndomains = [\n");
        for d in &td.tier1 {
            out.push_str("    \"");
            out.push_str(d);
            out.push_str("\",\n");
        }
        out.push_str("]\npatterns = [\n");
        for p in &td.tier1_patterns {
            out.push('\'');
            out.push_str(p);
            out.push_str("',\n");
        }
        out.push_str("]\n\n");

        out.push_str("[tier2]\ndomains = [\n");
        for d in &td.tier2 {
            out.push_str("    \"");
            out.push_str(d);
            out.push_str("\",\n");
        }
        out.push_str("]\n");
        let min_so = td.tier2_min_so_score.unwrap_or(20);
        out.push_str(&format!("min_so_score = {min_so}\n\n"));

        out.push_str("[tier4_blocklist]\ndomains = [\n");
        for d in &td.tier4 {
            out.push_str("    \"");
            out.push_str(d);
            out.push_str("\",\n");
        }
        out.push_str("]\npatterns = [\n");
        for p in &td.tier4_patterns {
            out.push('\'');
            out.push_str(p);
            out.push_str("',\n");
        }
        out.push_str("]\n");

        out
    }

    /// Look up the source tier for a domain string (not a full URL).
    /// Resolution order: T4 blocklist -> T1 authoritative -> T2 strong -> T3Weak default.
    /// Unknown domains return T3Weak, NOT T4Rejected.
    pub fn lookup(&self, domain: &str) -> SourceTier {
        match source_tier::classify_source(domain, &self.config) {
            Ok(classification) => classification.tier,
            // If classify_source fails (e.g., empty domain), default to T3Weak.
            Err(_) => SourceTier::T3Weak,
        }
    }

    /// Look up the source tier for a full URL, using url::Url for domain extraction.
    /// Falls back to T3Weak for unparseable URLs.
    pub fn lookup_url(&self, url_str: &str) -> SourceTier {
        match Self::extract_domain(url_str) {
            Some(domain) => self.lookup(&domain),
            None => SourceTier::T3Weak,
        }
    }

    /// Score a domain on the 5-tier scale: T1=1.0, T2=0.8, T3=0.6, T4=0.4.
    /// Pure deterministic function, no I/O.
    pub fn score(&self, domain: &str) -> f64 {
        Self::tier_to_score(self.lookup(domain))
    }

    /// Score a full URL on the 5-tier scale.
    pub fn score_url(&self, url_str: &str) -> f64 {
        Self::tier_to_score(self.lookup_url(url_str))
    }

    /// Map a SourceTier variant to its f64 score.
    pub fn tier_to_score(tier: SourceTier) -> f64 {
        match tier {
            SourceTier::T1Authoritative => SCORE_TIER1,
            SourceTier::T2Strong => SCORE_TIER2,
            SourceTier::T3Weak => SCORE_TIER3,
            SourceTier::T4Rejected => SCORE_TIER4,
        }
    }

    /// Extract domain from a URL using the url crate.
    /// Returns None for malformed URLs.
    fn extract_domain(url_str: &str) -> Option<String> {
        // Try parsing as-is first (has scheme).
        if let Ok(parsed) = url::Url::parse(url_str) {
            return parsed.host_str().map(|h| h.to_lowercase());
        }
        // Retry with https:// prefix for bare domains/paths.
        let with_scheme = format!("https://{url_str}");
        url::Url::parse(&with_scheme)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a minimal SourceTierConfig for testing via parse_source_tier_config.
    fn test_config() -> SourceTierConfig {
        let toml = r#"
[tier1]
domains = ["arxiv.org", "doi.org", "ietf.org", "w3.org", "ieee.org", "acm.org"]
patterns = ['\.gov$', '\.edu$', '\.ac\.[a-z]{2}$']

[tier2]
domains = ["docs.rs", "crates.io", "stackoverflow.com", "github.com"]
min_so_score = 20

[tier4_blocklist]
domains = ["medium.com", "dev.to", "w3schools.com"]
patterns = ['seo-']
"#;
        source_tier::parse_source_tier_config(toml).expect("test config parse")
    }

    fn test_table() -> SourceQualityTable {
        SourceQualityTable::new_from_config(test_config())
    }

    // --- lookup tests ---

    #[test]
    fn lookup_t1_exact_domain() {
        let table = test_table();
        assert_eq!(table.lookup("arxiv.org"), SourceTier::T1Authoritative);
    }

    #[test]
    fn lookup_t1_pattern_gov() {
        let table = test_table();
        assert_eq!(table.lookup("cdc.gov"), SourceTier::T1Authoritative);
        assert_eq!(table.lookup("nih.gov"), SourceTier::T1Authoritative);
    }

    #[test]
    fn lookup_t1_pattern_edu() {
        let table = test_table();
        assert_eq!(table.lookup("mit.edu"), SourceTier::T1Authoritative);
    }

    #[test]
    fn lookup_t1_pattern_ac_uk() {
        let table = test_table();
        assert_eq!(table.lookup("ox.ac.uk"), SourceTier::T1Authoritative);
    }

    #[test]
    fn lookup_t2_exact() {
        let table = test_table();
        assert_eq!(table.lookup("docs.rs"), SourceTier::T2Strong);
        assert_eq!(table.lookup("stackoverflow.com"), SourceTier::T2Strong);
    }

    #[test]
    fn lookup_t4_exact() {
        let table = test_table();
        assert_eq!(table.lookup("w3schools.com"), SourceTier::T4Rejected);
        assert_eq!(table.lookup("medium.com"), SourceTier::T4Rejected);
    }

    #[test]
    fn lookup_t4_pattern() {
        let table = test_table();
        assert_eq!(table.lookup("seo-spam.com"), SourceTier::T4Rejected);
    }

    #[test]
    fn lookup_unknown_defaults_to_t3weak() {
        let table = test_table();
        assert_eq!(table.lookup("unknown-site.com"), SourceTier::T3Weak);
        assert_eq!(table.lookup("random-blog.io"), SourceTier::T3Weak);
    }

    #[test]
    fn lookup_empty_domain_defaults_to_t3weak() {
        let table = test_table();
        assert_eq!(table.lookup(""), SourceTier::T3Weak);
    }

    // --- score tests ---

    #[test]
    fn score_t1_returns_1_0() {
        let table = test_table();
        assert_eq!(table.score("arxiv.org"), SCORE_TIER1);
    }

    #[test]
    fn score_t2_returns_0_8() {
        let table = test_table();
        assert_eq!(table.score("docs.rs"), SCORE_TIER2);
    }

    #[test]
    fn score_unknown_returns_0_6() {
        let table = test_table();
        assert_eq!(table.score("unknown.com"), SCORE_TIER3);
    }

    #[test]
    fn score_t4_returns_0_4() {
        let table = test_table();
        assert_eq!(table.score("w3schools.com"), SCORE_TIER4);
    }

    // --- lookup_url tests ---

    #[test]
    fn lookup_url_extracts_domain() {
        let table = test_table();
        assert_eq!(
            table.lookup_url("https://arxiv.org/abs/2301.00001"),
            SourceTier::T1Authoritative,
        );
    }

    #[test]
    fn lookup_url_with_query_string() {
        let table = test_table();
        assert_eq!(
            table.lookup_url("https://docs.rs/tokio?search=runtime"),
            SourceTier::T2Strong,
        );
    }

    #[test]
    fn lookup_url_malformed_returns_t3weak() {
        let table = test_table();
        assert_eq!(table.lookup_url("not-a-url"), SourceTier::T3Weak);
    }

    // --- extract_domain tests ---

    #[test]
    fn extract_domain_with_scheme() {
        let d = SourceQualityTable::extract_domain("https://ArXiv.Org/abs/123");
        assert_eq!(d.as_deref(), Some("arxiv.org"));
    }

    #[test]
    fn extract_domain_without_scheme() {
        let d = SourceQualityTable::extract_domain("docs.rs/tokio");
        assert_eq!(d.as_deref(), Some("docs.rs"));
    }

    #[test]
    fn extract_domain_empty() {
        assert_eq!(SourceQualityTable::extract_domain(""), None);
    }

    // --- tier_to_score tests ---

    #[test]
    fn tier_to_score_all_variants() {
        assert_eq!(SourceQualityTable::tier_to_score(SourceTier::T1Authoritative), SCORE_TIER1);
        assert_eq!(SourceQualityTable::tier_to_score(SourceTier::T2Strong), SCORE_TIER2);
        assert_eq!(SourceQualityTable::tier_to_score(SourceTier::T3Weak), SCORE_TIER3);
        assert_eq!(SourceQualityTable::tier_to_score(SourceTier::T4Rejected), SCORE_TIER4);
    }

    // --- new_from_config roundtrip ---

    #[test]
    fn new_from_config_preserves_classification() {
        let config = test_config();
        let table = SourceQualityTable::new_from_config(config);
        assert_eq!(table.lookup("arxiv.org"), SourceTier::T1Authoritative);
        assert_eq!(table.lookup("docs.rs"), SourceTier::T2Strong);
        assert_eq!(table.lookup("w3schools.com"), SourceTier::T4Rejected);
        assert_eq!(table.lookup("nope.com"), SourceTier::T3Weak);
    }

    // --- validate_config tests ---

    #[test]
    fn validate_config_rejects_empty() {
        let td = TrustedDomainsConfig::default();
        let result = SourceQualityTable::validate_config(&td);
        assert!(result.is_err());
    }

    #[test]
    fn validate_config_rejects_empty_domain_string() {
        let td = TrustedDomainsConfig {
            tier1: vec!["".to_owned()],
            ..Default::default()
        };
        let result = SourceQualityTable::validate_config(&td);
        assert!(result.is_err());
    }

    // --- synthesize_tier_toml roundtrip ---

    #[test]
    fn synthesize_roundtrip_produces_valid_config() {
        let td = TrustedDomainsConfig {
            tier1: vec!["arxiv.org".to_owned(), "doi.org".to_owned()],
            tier1_patterns: vec![r"\.gov$".to_owned()],
            tier2: vec!["docs.rs".to_owned()],
            tier2_min_so_score: Some(25),
            tier4: vec!["w3schools.com".to_owned()],
            tier4_patterns: vec![],
        };
        let synth = SourceQualityTable::synthesize_tier_toml(&td);
        let config = source_tier::parse_source_tier_config(&synth);
        assert!(config.is_ok(), "synthesized TOML should parse: {synth}");

        let table = SourceQualityTable::new_from_config(config.expect("already asserted ok"));
        assert_eq!(table.lookup("arxiv.org"), SourceTier::T1Authoritative);
        assert_eq!(table.lookup("docs.rs"), SourceTier::T2Strong);
        assert_eq!(table.lookup("w3schools.com"), SourceTier::T4Rejected);
        assert_eq!(table.lookup("cdc.gov"), SourceTier::T1Authoritative);
    }
}
