// A-43: Types for the Oracle research pipeline.
// All types shared between core/ and orchestrator/.

use serde::{Deserialize, Serialize};

use crate::source_tier::{SourceClassification, SourceTier};

/// Maximum search queries generated from a single research question.
pub const MAX_SEARCH_QUERIES: u32 = 4;
/// Minimum search queries generated from a single research question.
pub const MIN_SEARCH_QUERIES: u32 = 2;
/// Default maximum sources to consult per research query.
pub const DEFAULT_MAX_SOURCES: u32 = 20;
/// Minimum independent T1/T2 sources for VERIFIED status.
pub const VERIFIED_MIN_SOURCES: u32 = 2;
/// Minimum independent T1/T2 sources for SUPPORTED status (exactly 1).
pub const SUPPORTED_MIN_SOURCES: u32 = 1;
/// Maximum re-synthesis retries in post-verification (Phase G).
pub const MAX_RESYNTHESIS_RETRIES: u32 = 2;
/// Maximum sentences allowed without citation before removal.
pub const MAX_UNCITED_SENTENCES: u32 = 0;
/// Length of truncated blake3 hex ID for ExtractedFact.
pub const FACT_ID_HEX_LEN: usize = 16;

/// A research question with optional constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchQuery {
    /// The user's original question.
    pub question: String,
    /// Optional context narrowing the scope.
    pub context: Option<String>,
    /// Maximum number of sources to consult.
    pub max_sources: u32,
    /// If set, only accept facts from sources at or above this tier.
    pub required_tier: Option<SourceTier>,
}

impl ResearchQuery {
    /// Create a new research query with defaults.
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            context: None,
            max_sources: DEFAULT_MAX_SOURCES,
            required_tier: None,
        }
    }

    /// Set optional context.
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Set maximum sources.
    pub fn with_max_sources(mut self, n: u32) -> Self {
        self.max_sources = n;
        self
    }

    /// Set minimum required tier.
    pub fn with_required_tier(mut self, tier: SourceTier) -> Self {
        self.required_tier = Some(tier);
        self
    }
}

/// A single fact extracted from a source document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedFact {
    /// blake3 hash (truncated hex) of claim + source_url.
    pub id: String,
    /// The factual claim extracted.
    pub claim: String,
    /// URL of the source document.
    pub source_url: String,
    /// Reliability tier of the source.
    pub source_tier: SourceTier,
    /// Exact quote from the source supporting this claim. Required for SUPPORTED+.
    pub exact_quote: Option<String>,
    /// Section or heading where the fact was found.
    pub page_section: Option<String>,
    /// Confidence in the extraction (0.0 = uncertain, 1.0 = verbatim match).
    pub extraction_confidence: f64,
}

/// Generate a fact ID by hashing claim + source_url with blake3 (truncated hex).
/// Uses length-prefixed encoding to avoid ambiguity (e.g. "a|" + "b" vs "a" + "|b").
pub fn generate_fact_id(claim: &str, source_url: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    // Length-prefix each field as 4-byte little-endian to prevent collisions.
    let claim_bytes = claim.as_bytes();
    let url_bytes = source_url.as_bytes();
    hasher.update(&(claim_bytes.len() as u32).to_le_bytes());
    hasher.update(claim_bytes);
    hasher.update(&(url_bytes.len() as u32).to_le_bytes());
    hasher.update(url_bytes);
    let hash = hasher.finalize();
    hash.to_hex()[..FACT_ID_HEX_LEN].to_string()
}

impl ExtractedFact {
    /// Create a new fact with a generated ID.
    pub fn new(
        claim: impl Into<String>,
        source_url: impl Into<String>,
        source_tier: SourceTier,
    ) -> Self {
        let claim = claim.into();
        let source_url = source_url.into();
        let id = generate_fact_id(&claim, &source_url);
        Self {
            id,
            claim,
            source_url,
            source_tier,
            exact_quote: None,
            page_section: None,
            extraction_confidence: 0.0,
        }
    }

    /// Set exact quote.
    pub fn with_exact_quote(mut self, quote: impl Into<String>) -> Self {
        self.exact_quote = Some(quote.into());
        self
    }

    /// Set page section.
    pub fn with_page_section(mut self, section: impl Into<String>) -> Self {
        self.page_section = Some(section.into());
        self
    }

    /// Set extraction confidence.
    pub fn with_confidence(mut self, c: f64) -> Self {
        self.extraction_confidence = c;
        self
    }

    /// Whether this fact has an exact quote (required for SUPPORTED or higher).
    pub fn has_exact_quote(&self) -> bool {
        self.exact_quote.is_some()
    }
}

/// Verification level for a claim based on cross-referencing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerificationLevel {
    /// 2+ independent T1/T2 sources with exact quotes.
    Verified,
    /// Exactly 1 T1/T2 source with exact quote.
    Supported,
    /// T3-only sources, or T1/T2 source without an exact quote.
    Weak,
    /// No exact quotes from any source.
    Unsupported,
}

/// A claim with its supporting evidence and verification level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifiedClaim {
    /// The claim text.
    pub claim: String,
    /// All facts supporting this claim.
    pub supporting_facts: Vec<ExtractedFact>,
    /// Computed verification level.
    pub verification_level: VerificationLevel,
    /// Number of independent T1/T2 sources with exact quotes.
    pub independent_source_count: u32,
}

/// Overall answer confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnswerConfidence {
    /// All key claims VERIFIED (2+ independent T1/T2 sources each).
    High,
    /// Most claims SUPPORTED or VERIFIED; no UNSUPPORTED claims.
    Medium,
    /// Some claims WEAK or UNSUPPORTED.
    Low,
    /// Cannot determine — insufficient evidence or pipeline failure.
    Unknown,
}

/// Complete research answer with evidence chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchAnswer {
    /// The original question.
    pub question: String,
    /// Synthesized answer text (every sentence cites a fact).
    pub answer_text: String,
    /// Evidence chain: all verified claims with supporting facts.
    pub evidence_chain: Vec<VerifiedClaim>,
    /// Overall answer confidence.
    pub confidence: AnswerConfidence,
    /// Known limitations or caveats.
    pub caveats: Vec<String>,
    /// Parts of the question that could not be answered.
    pub unanswered_parts: Vec<String>,
    /// All sources that were consulted (classified).
    pub sources_consulted: Vec<SourceClassification>,
    /// Sources that were rejected (T4 or below required tier).
    pub sources_rejected: Vec<SourceClassification>,
}

/// Errors from the research pipeline.
#[derive(Debug, thiserror::Error)]
pub enum ResearchPipelineError {
    #[error("empty research question")]
    EmptyQuestion,
    #[error("search backend error: {0}")]
    SearchBackend(String),
    #[error("source tier classification error: {0}")]
    SourceTier(#[from] crate::source_tier::SourceTierError),
    #[error("synthesis failed after {0} retries")]
    SynthesisFailed(u32),
    #[error("no usable sources found for query")]
    NoUsableSources,
    #[error("post-verification failed: {0}")]
    PostVerification(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Contract: ResearchQuery builder ---

    #[test]
    fn query_defaults() {
        let q = ResearchQuery::new("What is Rust?");
        assert_eq!(q.question, "What is Rust?");
        assert_eq!(q.max_sources, DEFAULT_MAX_SOURCES);
        assert!(q.context.is_none());
        assert!(q.required_tier.is_none());
    }

    #[test]
    fn query_builder_chain() {
        let q = ResearchQuery::new("test")
            .with_context("Rust async")
            .with_max_sources(10)
            .with_required_tier(SourceTier::T1Authoritative);
        assert_eq!(q.context.as_deref(), Some("Rust async"));
        assert_eq!(q.max_sources, 10);
        assert_eq!(q.required_tier, Some(SourceTier::T1Authoritative));
    }

    // --- Contract: ExtractedFact ID generation ---

    #[test]
    fn fact_id_uses_blake3_and_is_deterministic() {
        let f1 = ExtractedFact::new("claim A", "https://a.com", SourceTier::T1Authoritative);
        let f2 = ExtractedFact::new("claim A", "https://a.com", SourceTier::T1Authoritative);
        assert_eq!(f1.id, f2.id);
        assert_eq!(f1.id.len(), FACT_ID_HEX_LEN);
    }

    #[test]
    fn fact_id_differs_for_different_claims() {
        let f1 = ExtractedFact::new("claim A", "https://a.com", SourceTier::T1Authoritative);
        let f2 = ExtractedFact::new("claim B", "https://a.com", SourceTier::T1Authoritative);
        assert_ne!(f1.id, f2.id);
    }

    #[test]
    fn fact_id_differs_for_different_urls() {
        let f1 = ExtractedFact::new("claim A", "https://a.com", SourceTier::T1Authoritative);
        let f2 = ExtractedFact::new("claim A", "https://b.com", SourceTier::T1Authoritative);
        assert_ne!(f1.id, f2.id);
    }

    #[test]
    fn fact_without_quote_is_not_supported() {
        let f = ExtractedFact::new("claim", "https://a.com", SourceTier::T1Authoritative);
        assert!(!f.has_exact_quote());
    }

    #[test]
    fn fact_with_quote_is_supported() {
        let f = ExtractedFact::new("claim", "https://a.com", SourceTier::T1Authoritative)
            .with_exact_quote("verbatim text");
        assert!(f.has_exact_quote());
    }

    // --- Contract: serde round-trips ---

    #[test]
    fn verification_level_serde() {
        for level in [
            VerificationLevel::Verified,
            VerificationLevel::Supported,
            VerificationLevel::Weak,
            VerificationLevel::Unsupported,
        ] {
            let json = serde_json::to_string(&level).expect("ser");
            let back: VerificationLevel = serde_json::from_str(&json).expect("de");
            assert_eq!(level, back);
        }
    }

    #[test]
    fn answer_confidence_serde() {
        for conf in [
            AnswerConfidence::High,
            AnswerConfidence::Medium,
            AnswerConfidence::Low,
            AnswerConfidence::Unknown,
        ] {
            let json = serde_json::to_string(&conf).expect("ser");
            let back: AnswerConfidence = serde_json::from_str(&json).expect("de");
            assert_eq!(conf, back);
        }
    }

    #[test]
    fn research_query_serde() {
        let q = ResearchQuery::new("test").with_context("ctx");
        let json = serde_json::to_string(&q).expect("ser");
        let back: ResearchQuery = serde_json::from_str(&json).expect("de");
        assert_eq!(q, back);
    }

    #[test]
    fn extracted_fact_serde() {
        let f = ExtractedFact::new("claim", "https://a.com", SourceTier::T2Strong)
            .with_exact_quote("quote")
            .with_confidence(0.95);
        let json = serde_json::to_string(&f).expect("ser");
        let back: ExtractedFact = serde_json::from_str(&json).expect("de");
        assert_eq!(f, back);
    }

    // --- Contract: constants are sane ---

    #[test]
    fn constants_are_valid() {
        assert!(MIN_SEARCH_QUERIES <= MAX_SEARCH_QUERIES);
        assert!(SUPPORTED_MIN_SOURCES < VERIFIED_MIN_SOURCES);
        assert!(DEFAULT_MAX_SOURCES > 0);
        assert!(FACT_ID_HEX_LEN > 0);
        assert!(FACT_ID_HEX_LEN <= 64); // blake3 hex is 64 chars max
    }
}
