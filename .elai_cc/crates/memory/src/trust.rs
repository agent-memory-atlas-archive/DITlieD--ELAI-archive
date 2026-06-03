//! C-48: TrustTier + Provenance types for SafeHarbor memory stratification.

use serde::{Deserialize, Serialize};
use crate::error::MemoryError;

/// Memory trust tier per SafeHarbor (arXiv:2605.05704) stratification.
/// T0 = highest trust (user-direct); T3 = lowest (model-emitted).
/// Safety-critical queries consult ONLY T0+T1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    /// T0: typed by user in CLI/dashboard/explicit user-marked memory write.
    T0UserDirect,
    /// T1: extracted from local source files, git history. Trustworthy for
    /// technical decisions but NOT safety policy.
    T1CodebaseDerived,
    /// T2: WebFetch, paper-ingestor mode=paper. Useful for grounding, NOT
    /// authoritative for control-flow.
    T2WebSourced,
    /// T3: LLM-emitted content captured into memory without human review.
    /// Lowest trust. Default for pre-C-48 rows via migration (most pessimistic).
    T3ModelEmitted,
}

impl TrustTier {
    /// Stable snake_case wire tag for SQL storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            TrustTier::T0UserDirect => "T0",
            TrustTier::T1CodebaseDerived => "T1",
            TrustTier::T2WebSourced => "T2",
            TrustTier::T3ModelEmitted => "T3",
        }
    }

    /// Parse from SQL wire tag.
    pub fn from_str(s: &str) -> Result<Self, MemoryError> {
        match s {
            "T0" => Ok(TrustTier::T0UserDirect),
            "T1" => Ok(TrustTier::T1CodebaseDerived),
            "T2" => Ok(TrustTier::T2WebSourced),
            "T3" => Ok(TrustTier::T3ModelEmitted),
            other => Err(MemoryError::Invalid(format!("unknown trust_tier: {other}"))),
        }
    }

    /// Returns true for tiers permitted in safety-critical queries (T0 + T1).
    pub fn is_safety_critical_allowed(&self) -> bool {
        matches!(self, TrustTier::T0UserDirect | TrustTier::T1CodebaseDerived)
    }
}

impl Default for TrustTier {
    /// Default is T3 (most pessimistic) for migration compatibility.
    fn default() -> Self {
        TrustTier::T3ModelEmitted
    }
}

/// Source provenance for a memory fact. Non-optional post-C-48.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// Human-readable source description (URL, file path, "user_direct", etc.)
    pub source: String,
    /// Trust tier this provenance source belongs to.
    pub tier: TrustTier,
}

impl Provenance {
    /// Construct a T0 user-direct provenance.
    pub fn user_direct(source: impl Into<String>) -> Self {
        Provenance { source: source.into(), tier: TrustTier::T0UserDirect }
    }

    /// Construct a T1 codebase-derived provenance (filepath or git ref).
    pub fn codebase_derived(source: impl Into<String>) -> Self {
        Provenance { source: source.into(), tier: TrustTier::T1CodebaseDerived }
    }

    /// Construct a T2 web-sourced provenance (URL).
    pub fn web_sourced(url: impl Into<String>) -> Self {
        Provenance { source: url.into(), tier: TrustTier::T2WebSourced }
    }

    /// Construct a T3 model-emitted provenance.
    pub fn model_emitted(source: impl Into<String>) -> Self {
        Provenance { source: source.into(), tier: TrustTier::T3ModelEmitted }
    }
}

impl Default for Provenance {
    /// Default provenance is T3 (most pessimistic), empty source.
    fn default() -> Self {
        Provenance { source: String::new(), tier: TrustTier::T3ModelEmitted }
    }
}

#[cfg(test)]
mod cold_wiring_tests {
    //! Cold-wiring (s-81344): deterministic coverage for TrustTier codec +
    //! safety filter + Provenance constructors (previously no inline tests).
    use super::*;

    #[test]
    fn trust_tier_codec_roundtrips() {
        let pairs = [
            (TrustTier::T0UserDirect, "T0"),
            (TrustTier::T1CodebaseDerived, "T1"),
            (TrustTier::T2WebSourced, "T2"),
            (TrustTier::T3ModelEmitted, "T3"),
        ];
        for (t, tag) in pairs {
            assert_eq!(t.as_str(), tag);
            assert_eq!(TrustTier::from_str(tag).unwrap(), t);
        }
        assert!(TrustTier::from_str("T9").is_err());
    }

    #[test]
    fn safety_critical_allows_only_t0_t1() {
        assert!(TrustTier::T0UserDirect.is_safety_critical_allowed());
        assert!(TrustTier::T1CodebaseDerived.is_safety_critical_allowed());
        assert!(!TrustTier::T2WebSourced.is_safety_critical_allowed());
        assert!(!TrustTier::T3ModelEmitted.is_safety_critical_allowed());
    }

    #[test]
    fn trust_tier_default_is_most_pessimistic() {
        assert_eq!(TrustTier::default(), TrustTier::T3ModelEmitted);
    }

    #[test]
    fn provenance_constructors_set_source_and_tier() {
        let u = Provenance::user_direct("cli");
        assert_eq!(u.source, "cli");
        assert_eq!(u.tier, TrustTier::T0UserDirect);

        let c = Provenance::codebase_derived("src/main.rs:42");
        assert_eq!(c.tier, TrustTier::T1CodebaseDerived);

        let w = Provenance::web_sourced("https://x.com");
        assert_eq!(w.source, "https://x.com");
        assert_eq!(w.tier, TrustTier::T2WebSourced);

        let m = Provenance::model_emitted("llm");
        assert_eq!(m.tier, TrustTier::T3ModelEmitted);
    }

    #[test]
    fn provenance_default_is_empty_t3() {
        let d = Provenance::default();
        assert!(d.source.is_empty());
        assert_eq!(d.tier, TrustTier::T3ModelEmitted);
    }
}
