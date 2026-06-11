//! H-32 citation-validity gate — validate-before-use for code-grounded facts.
//!
//! A Tier-A (DomainKnowledge/Mistake) fact carries a DKG symbol citation in its
//! `evidence_pointers` in the form `dkg:<name>@<file>:<line>`. Before the fact
//! is injected into a prompt, the citation is re-resolved against the current
//! DKG name index. A citation that no longer resolves (symbol renamed/deleted)
//! makes the fact STALE → hard-excluded + `fact_validation_failed` SSB row
//! emitted (the off-agent artifact, R4). An unavailable index is treated as
//! Unknown → also hard-excluded (R2 fail-closed: missing evidence is FAIL).
//!
//! The `CitationValidator` TRAIT lives here (in crates/memory) so the concrete
//! DKG-backed impl can live in crates/cli without creating a memory→knowledge
//! crate edge. The retrieval gate (`get_relevant_facts` extension) consumes the
//! trait object.
//!
//! Unused-TTL expiry: a Tier-A fact unread for 28 days is soft-closed. The TTL
//! reference point is `max(last_validated_at_ms, valid_from_ms)` — a validated
//! use resets the TTL. 28 days = GitHub Copilot agentic memory measured default
//! (github.blog 2026, third-party-implementation tier).

use std::fmt;

use crate::fact::{FactType, MemoryFact};

/// 28-day unused TTL in milliseconds. A Tier-A fact whose TTL reference point
/// (max(last_validated_at_ms, valid_from_ms)) is older than this is expired and
/// will be soft-closed by `collect_unused_ttl_expired`.
pub const UNUSED_TTL_MS: u128 = 28 * 24 * 60 * 60 * 1_000;

// ---------------------------------------------------------------------------
// Validity enum
// ---------------------------------------------------------------------------

/// Result of re-resolving a Tier-A fact's DKG citation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Validity {
    /// The cited symbol still resolves at the recorded file + line. Fact is
    /// safe to inject; `last_validated_at_ms` is stamped.
    Valid,
    /// The cited symbol previously resolved but no longer does (renamed,
    /// deleted, moved). Fact is HARD-EXCLUDED; a `fact_validation_failed` SSB
    /// row is emitted with `reason = "stale"`.
    Stale,
    /// The validator could not run (index unavailable, parse error, etc.).
    /// Treated as FAIL per R2 (missing evidence is a hard FAIL, not SKIP).
    /// `fact_validation_failed` SSB row emitted with `reason = "unknown"`.
    Unknown,
}

impl fmt::Display for Validity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Validity::Valid => f.write_str("valid"),
            Validity::Stale => f.write_str("stale"),
            Validity::Unknown => f.write_str("unknown"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tier classification
// ---------------------------------------------------------------------------

/// Tier-A = code-grounded (DomainKnowledge / Mistake); citation validity gate
/// applies. Tier-B = preference-class (everything else); validity gate skips
/// (returns Valid immediately — no citation needed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Code-grounded — must carry a resolvable DKG citation before injection.
    A,
    /// Preference-class — no citation requirement; always allowed to inject.
    B,
}

/// Classify a fact's tier based on its `FactType`.
///
/// Tier-A: `DomainKnowledge` and `Mistake` (code-citable per-project conventions
/// and recurring-mistake / footgun facts). Every other variant is Tier-B.
pub fn classify_fact_tier(fact_type: FactType) -> Tier {
    match fact_type {
        FactType::DomainKnowledge | FactType::Mistake => Tier::A,
        FactType::VoiceStyle
        | FactType::Expertise
        | FactType::RiskTol
        | FactType::Opinion
        | FactType::Preference
        | FactType::SessionPattern => Tier::B,
    }
}

// ---------------------------------------------------------------------------
// CitationPointer
// ---------------------------------------------------------------------------

/// Structured DKG symbol citation — parsed from / formatted as the wire string
/// `dkg:<name>@<file>:<line>` stored in `evidence_pointers`.
///
/// The blake3 16-hex key (`citation_key`) is a stable fingerprint of the
/// canonical `<name>@<file>:<line>` string, computed at parse time and used
/// as the evidence fingerprint in `FactValidationFailedPayload.evidence_blake3`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationPointer {
    /// Symbol name (simple/unqualified, e.g. `get_relevant_facts`).
    pub symbol_name: String,
    /// Repo-relative or absolute file path.
    pub file: String,
    /// 1-based source line.
    pub line: u32,
    /// blake3 16-hex fingerprint of `<name>@<file>:<line>`.
    pub citation_key: String,
}

impl CitationPointer {
    /// Parse a `dkg:<name>@<file>:<line>` evidence-pointer string.
    ///
    /// Returns `None` if the string does not match the expected format (treated
    /// as unparseable citation → `Validity::Unknown` at validation time).
    pub fn parse(s: &str) -> Option<Self> {
        let rest = s.strip_prefix("dkg:")?;
        // Split on the last `@` so file paths containing `@` still work.
        let at_pos = rest.rfind('@')?;
        let symbol_name = rest[..at_pos].to_owned();
        let after_at = &rest[at_pos + 1..];
        // Split on the last `:` to allow Windows-style `C:/path/to/file.rs:42`.
        let colon_pos = after_at.rfind(':')?;
        let file = after_at[..colon_pos].to_owned();
        let line: u32 = after_at[colon_pos + 1..].parse().ok()?;
        if symbol_name.is_empty() || file.is_empty() {
            return None;
        }
        let canonical = format!("{symbol_name}@{file}:{line}");
        let citation_key = blake3::hash(canonical.as_bytes())
            .to_hex()
            .chars()
            .take(16)
            .collect();
        Some(Self { symbol_name, file, line, citation_key })
    }

    /// Format as the canonical wire string `dkg:<name>@<file>:<line>`.
    pub fn to_wire(&self) -> String {
        format!("dkg:{}@{}:{}", self.symbol_name, self.file, self.line)
    }
}

// ---------------------------------------------------------------------------
// CitationValidator TRAIT
// ---------------------------------------------------------------------------

/// Seam for DKG-backed citation re-resolution. Defined in crates/memory so
/// retrieval.rs can consume it without a memory→knowledge crate edge. The
/// concrete implementation (`DkgCitationValidator`) lives in crates/cli.
///
/// # Contract
/// - `validate` MUST return `Validity::Unknown` (not panic) when the
///   underlying index is unavailable.
/// - `validate` on a Tier-B fact (no parseable DKG pointer) MUST return
///   `Validity::Valid` (no citation needed for Tier-B).
/// - `validate` is called PER FACT on the already-truncated top-K slice; it
///   MUST NOT iterate the entire store.
pub trait CitationValidator: Send + Sync {
    /// Re-resolve the fact's DKG citation. See [`Validity`] for semantics.
    fn validate(&self, fact: &MemoryFact) -> Validity;
}

// ---------------------------------------------------------------------------
// NullValidator — fail-closed fallback when the DKG index is unavailable
// ---------------------------------------------------------------------------

/// Fail-closed validator used when the concrete DKG-backed validator cannot be
/// opened (e.g. the DKG index file does not exist yet on a fresh project).
///
/// Returns `Validity::Unknown` for every Tier-A fact, causing the retrieval gate
/// to hard-exclude all Tier-A facts (R2: missing evidence is FAIL, not SKIP).
/// Tier-B facts are handled by `revalidate_citation`'s short-circuit, so this
/// impl never receives them directly.
pub struct NullValidator;

impl CitationValidator for NullValidator {
    fn validate(&self, _fact: &MemoryFact) -> Validity {
        Validity::Unknown
    }
}

// ---------------------------------------------------------------------------
// revalidate_citation helper
// ---------------------------------------------------------------------------

/// Re-validate the DKG citation carried by `fact` using `validator`.
///
/// Short-circuits to `Valid` for Tier-B facts (no citation required). For
/// Tier-A facts with no parseable `dkg:` pointer in `evidence_pointers`,
/// returns `Unknown` (R2 fail-closed). Delegates resolution to `validator`.
pub fn revalidate_citation(fact: &MemoryFact, validator: &dyn CitationValidator) -> Validity {
    // Tier-B: preference-class facts need no citation. Always Valid.
    if classify_fact_tier(fact.fact_type) == Tier::B {
        return Validity::Valid;
    }
    // Tier-A: require a parseable dkg: pointer. Missing → Unknown (fail-closed).
    let has_dkg_pointer = fact
        .evidence_pointers
        .iter()
        .any(|p| CitationPointer::parse(p).is_some());
    if !has_dkg_pointer {
        return Validity::Unknown;
    }
    // Delegate full resolution to the injected validator.
    validator.validate(fact)
}

// ---------------------------------------------------------------------------
// collect_unused_ttl_expired
// ---------------------------------------------------------------------------

/// Scan `facts` for Tier-A rows whose TTL has expired. A fact's TTL reference
/// point is `max(last_validated_at_ms, valid_from_ms)`. If that reference
/// point is more than `UNUSED_TTL_MS` before `now_ms`, the row is expired.
///
/// Returns the `fact_id`s of expired rows; the caller is responsible for
/// calling `FactStore::soft_close` on each (soft-close, never DELETE per
/// stub_ban). NotFound from soft_close (fact already closed by a concurrent
/// path) is treated as benign by the caller.
pub fn collect_unused_ttl_expired(facts: &[MemoryFact], now_ms: u128) -> Vec<String> {
    facts
        .iter()
        .filter(|f| classify_fact_tier(f.fact_type) == Tier::A)
        .filter(|f| f.valid_to_ms.is_none())
        .filter(|f| {
            let ttl_ref = f
                .last_validated_at_ms
                .unwrap_or(f.valid_from_ms)
                .max(f.valid_from_ms);
            now_ms.saturating_sub(ttl_ref) > UNUSED_TTL_MS
        })
        .map(|f| f.fact_id.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::{FactType, MemoryFact, Scope};
    use crate::trust::{Provenance, TrustTier};

    fn make_tier_a_fact(key: &str, evidence: Vec<String>) -> MemoryFact {
        MemoryFact {
            fact_id: format!("fact-{key}"),
            user_id: "u1".into(),
            project_id: Some("proj-1".into()),
            scope: Scope::Project,
            fact_type: FactType::DomainKnowledge,
            key: key.into(),
            value: serde_json::json!({ "raw": "no unwrap in prod" }),
            confidence: 0.9,
            trust_score: 0.8,
            ingested_at_ms: 1_000,
            valid_from_ms: 1_000,
            valid_to_ms: None,
            decay_rate_per_day: 0.01,
            last_observed_at_ms: 1_000,
            evidence_pointers: evidence,
            superseded_by: None,
            trust_tier: TrustTier::T0UserDirect,
            provenance: Provenance::user_direct("test"),
            last_validated_at_ms: None,
        }
    }

    fn make_tier_b_fact() -> MemoryFact {
        MemoryFact {
            fact_id: "fact-pref".into(),
            user_id: "u1".into(),
            project_id: None,
            scope: Scope::Global,
            fact_type: FactType::Preference,
            key: "pref_key".into(),
            value: serde_json::json!({ "raw": "prefers tracing" }),
            confidence: 0.9,
            trust_score: 0.8,
            ingested_at_ms: 1_000,
            valid_from_ms: 1_000,
            valid_to_ms: None,
            decay_rate_per_day: 0.01,
            last_observed_at_ms: 1_000,
            evidence_pointers: vec!["dkg:some_fn@src/lib.rs:42".into()],
            superseded_by: None,
            trust_tier: TrustTier::T0UserDirect,
            provenance: Provenance::user_direct("test"),
            last_validated_at_ms: None,
        }
    }

    // --- CitationPointer parse ---

    #[test]
    fn citation_pointer_parses_canonical_form() {
        let p = CitationPointer::parse("dkg:get_relevant_facts@crates/memory/src/retrieval.rs:46")
            .expect("parse");
        assert_eq!(p.symbol_name, "get_relevant_facts");
        assert_eq!(p.file, "crates/memory/src/retrieval.rs");
        assert_eq!(p.line, 46);
        assert_eq!(p.citation_key.len(), 16);
    }

    #[test]
    fn citation_pointer_round_trips_through_to_wire() {
        let wire = "dkg:ensure_name_index@crates/knowledge/src/symbol_graph/cache.rs:276";
        let p = CitationPointer::parse(wire).expect("parse");
        assert_eq!(p.to_wire(), wire);
    }

    #[test]
    fn citation_pointer_rejects_missing_prefix() {
        assert!(CitationPointer::parse("get_relevant_facts@src/lib.rs:10").is_none());
    }

    #[test]
    fn citation_pointer_rejects_non_numeric_line() {
        assert!(CitationPointer::parse("dkg:fn@src/lib.rs:abc").is_none());
    }

    #[test]
    fn citation_pointer_fingerprint_is_stable_and_16_hex() {
        let p = CitationPointer::parse("dkg:foo@bar.rs:1").expect("parse");
        let p2 = CitationPointer::parse("dkg:foo@bar.rs:1").expect("parse again");
        assert_eq!(p.citation_key, p2.citation_key);
        assert!(p.citation_key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // --- classify_fact_tier ---

    #[test]
    fn tier_a_for_domain_knowledge_and_mistake() {
        assert_eq!(classify_fact_tier(FactType::DomainKnowledge), Tier::A);
        assert_eq!(classify_fact_tier(FactType::Mistake), Tier::A);
    }

    #[test]
    fn tier_b_for_all_preference_class_variants() {
        for ft in [
            FactType::VoiceStyle,
            FactType::Expertise,
            FactType::RiskTol,
            FactType::Opinion,
            FactType::Preference,
            FactType::SessionPattern,
        ] {
            assert_eq!(classify_fact_tier(ft), Tier::B, "expected Tier-B for {ft:?}");
        }
    }

    // --- revalidate_citation (static validator stubs) ---

    struct AlwaysValidValidator;
    impl CitationValidator for AlwaysValidValidator {
        fn validate(&self, _fact: &MemoryFact) -> Validity {
            Validity::Valid
        }
    }

    struct AlwaysStaleValidator;
    impl CitationValidator for AlwaysStaleValidator {
        fn validate(&self, _fact: &MemoryFact) -> Validity {
            Validity::Stale
        }
    }

    #[test]
    fn tier_b_fact_short_circuits_to_valid_regardless_of_validator() {
        let fact = make_tier_b_fact();
        let result = revalidate_citation(&fact, &AlwaysStaleValidator);
        assert_eq!(result, Validity::Valid, "Tier-B must short-circuit to Valid");
    }

    #[test]
    fn tier_a_no_dkg_pointer_returns_unknown() {
        let fact = make_tier_a_fact("k1", vec!["sess-123".into()]);
        let result = revalidate_citation(&fact, &AlwaysValidValidator);
        assert_eq!(
            result,
            Validity::Unknown,
            "Tier-A with no dkg: pointer must return Unknown (fail-closed)"
        );
    }

    #[test]
    fn tier_a_valid_dkg_pointer_delegates_to_validator() {
        let fact = make_tier_a_fact(
            "k2",
            vec!["dkg:get_relevant_facts@crates/memory/src/retrieval.rs:46".into()],
        );
        assert_eq!(revalidate_citation(&fact, &AlwaysValidValidator), Validity::Valid);
        assert_eq!(revalidate_citation(&fact, &AlwaysStaleValidator), Validity::Stale);
    }

    // --- collect_unused_ttl_expired ---

    #[test]
    fn expired_tier_a_fact_is_returned() {
        let mut fact = make_tier_a_fact(
            "expired",
            vec!["dkg:foo@bar.rs:1".into()],
        );
        // valid_from_ms = 1_000, last_validated_at_ms = None, now = 1_000 + TTL + 1
        fact.valid_from_ms = 1_000;
        fact.last_validated_at_ms = None;
        let now = 1_000 + UNUSED_TTL_MS + 1;
        let expired = collect_unused_ttl_expired(&[fact.clone()], now);
        assert_eq!(expired, vec![fact.fact_id]);
    }

    #[test]
    fn non_expired_tier_a_fact_is_not_returned() {
        let mut fact = make_tier_a_fact("fresh", vec!["dkg:foo@bar.rs:1".into()]);
        fact.valid_from_ms = 1_000;
        fact.last_validated_at_ms = None;
        let now = 1_000 + UNUSED_TTL_MS - 1;
        let expired = collect_unused_ttl_expired(&[fact], now);
        assert!(expired.is_empty());
    }

    #[test]
    fn validated_use_resets_ttl_reference() {
        let mut fact = make_tier_a_fact("validated", vec!["dkg:foo@bar.rs:1".into()]);
        fact.valid_from_ms = 1_000;
        // Simulated recent validation — well within TTL from now.
        let recent_validation = 100_000_000_u128;
        fact.last_validated_at_ms = Some(recent_validation);
        let now = recent_validation + UNUSED_TTL_MS - 1;
        let expired = collect_unused_ttl_expired(&[fact], now);
        assert!(expired.is_empty(), "recent validated use resets TTL");
    }

    #[test]
    fn tier_b_fact_is_never_expired_by_ttl_collector() {
        let mut fact = make_tier_b_fact();
        fact.valid_from_ms = 1_000;
        fact.last_validated_at_ms = None;
        // Far past the TTL.
        let now = 1_000 + UNUSED_TTL_MS * 10;
        let expired = collect_unused_ttl_expired(&[fact], now);
        assert!(
            expired.is_empty(),
            "Tier-B facts are never subject to citation TTL expiry"
        );
    }

    #[test]
    fn closed_fact_is_not_returned_by_ttl_collector() {
        let mut fact = make_tier_a_fact("closed", vec!["dkg:foo@bar.rs:1".into()]);
        fact.valid_from_ms = 1_000;
        fact.valid_to_ms = Some(5_000); // already closed
        fact.last_validated_at_ms = None;
        let now = 1_000 + UNUSED_TTL_MS * 10;
        let expired = collect_unused_ttl_expired(&[fact], now);
        assert!(expired.is_empty(), "closed (valid_to_ms set) facts are excluded");
    }

    // --- NullValidator (B-2 fail-closed fallback) ---

    #[test]
    fn null_validator_returns_unknown_for_tier_a_fact() {
        let fact = make_tier_a_fact(
            "null_path",
            vec!["dkg:get_relevant_facts@crates/memory/src/retrieval.rs:46".into()],
        );
        // NullValidator must return Unknown — simulates "DKG index unavailable".
        let result = revalidate_citation(&fact, &NullValidator);
        assert_eq!(
            result,
            Validity::Unknown,
            "NullValidator must return Unknown for Tier-A, causing hard-exclusion (R2 fail-closed)"
        );
    }

    #[test]
    fn null_validator_tier_b_still_short_circuits_to_valid() {
        // Tier-B short-circuits BEFORE the validator is called, so NullValidator
        // never sees Tier-B facts; this test proves the short-circuit holds.
        let fact = make_tier_b_fact();
        let result = revalidate_citation(&fact, &NullValidator);
        assert_eq!(
            result,
            Validity::Valid,
            "Tier-B must short-circuit to Valid even with NullValidator"
        );
    }
}
