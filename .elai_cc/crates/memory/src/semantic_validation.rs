//! J-21 Step 1 — commit-aware semantic validation.
//!
//! EXTENDS H-32's existence-only citation check with semantic IDENTITY
//! validation. A Tier-A code-grounded fact is stored against a commit hash
//! and an AST/function-signature identity, and re-derived against the current
//! checkout (scoped by repo+branch+commit-range) before injection. A fact
//! whose cited symbol still EXISTS but whose signature/behavior changed is
//! STALE, not Valid — the memory is "valid" and wrong under H-32's
//! existence-only check, which is the gap J-21 closes.
//!
//! # Grounding
//!
//! GEM — "correctness is a property of the state trajectory, not of
//! individual records" (arXiv:2605.26252, main-LLM WebFetch CONFIRMED).
//! This module applies the GEM state-trajectory model to code memory: the
//! identity of a code-cited fact is the `(symbol, commit, signature)` triple,
//! not just the `(symbol, file, line)` triple H-32 checks. A refactor that
//! preserves the path while changing the body preserves existence but
//! invalidates the stored CLAIM — the fact is stale at the state level even
//! though it passes the record-level existence check.
//!
//! # Relationship to H-32
//!
//! This module does NOT amend [`crate::citation_validity`]. The H-32
//! existence check ([`crate::citation_validity::revalidate_citation`])
//! remains the first-stage filter. Semantic validation is the second stage
//! that runs only when existence succeeds. Callers that want the upgraded
//! gate use [`revalidate_semantic`]; callers that only want existence use
//! the H-32 function unchanged.
//!
//! # Wire format
//!
//! The semantic identity is stored alongside the DKG citation pointer in
//! `MemoryFact::evidence_pointers` using the wire prefix `sem:` —
//! `sem:<symbol_name>@<commit_hash>:<signature_blake3>`. This avoids
//! amending the `MemoryFact` struct (H-32/E-5 territory) while persisting
//! the commit+AST identity in the existing provenance field.

use std::fmt;

use crate::citation_validity::{classify_fact_tier, CitationPointer, CitationValidator, Tier, Validity};
use crate::fact::MemoryFact;

// ---------------------------------------------------------------------------
// SymbolIdentity — the (symbol, commit, signature) triple
// ---------------------------------------------------------------------------

/// Compiler-grade symbol identity — the `(symbol, commit, signature)` triple
/// that J-18's LSP channel and J-16's blast-radius cone also resolve. A fact
/// is semantically valid only if its stored identity matches the current
/// checkout's identity for the same symbol.
///
/// `signature_blake3` is a blake3 16-hex fingerprint of the symbol's AST
/// signature (function signature / type signature / structural shape),
/// computed by the concrete [`SemanticValidator`] impl from the DKG or LSP
/// channel. The blake3 choice mirrors [`crate::citation_validity`] (H-32
/// anti-stale-training rule: blake3 for non-RFC hashing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolIdentity {
    /// Symbol name (matches [`CitationPointer::symbol_name`]).
    pub symbol_name: String,
    /// Git commit hash the fact was stored against (40-hex SHA-1 or 64-hex
    /// SHA-256; the concrete impl decides which git object is canonical).
    pub commit_hash: String,
    /// blake3 16-hex fingerprint of the symbol's AST signature at the stored
    /// commit.
    pub signature_blake3: String,
    /// Repo-relative file path at the stored commit.
    pub file: String,
    /// 1-based source line at the stored commit.
    pub line: u32,
}

impl SymbolIdentity {
    /// Format as the canonical wire string `sem:<name>@<commit>:<sig>`.
    pub fn to_wire(&self) -> String {
        format!(
            "sem:{}@{}:{}",
            self.symbol_name, self.commit_hash, self.signature_blake3
        )
    }

    /// Parse a `sem:<name>@<commit>:<sig>` evidence-pointer string.
    ///
    /// Returns `None` if the string does not match the expected format.
    /// The `commit` and `sig` fields MUST be non-empty hex-strings; the
    /// parse rejects empty or non-hex values (fail-closed per R2).
    pub fn parse_wire(s: &str, file: &str, line: u32) -> Option<Self> {
        let rest = s.strip_prefix("sem:")?;
        let at_pos = rest.rfind('@')?;
        let symbol_name = rest[..at_pos].to_owned();
        let after_at = &rest[at_pos + 1..];
        let colon_pos = after_at.rfind(':')?;
        let commit_hash = after_at[..colon_pos].to_owned();
        let signature_blake3 = after_at[colon_pos + 1..].to_owned();
        if symbol_name.is_empty() || commit_hash.is_empty() || signature_blake3.is_empty() {
            return None;
        }
        // Reject non-hex commit/sig — fail-closed on malformed provenance.
        if !commit_hash.chars().all(|c| c.is_ascii_hexdigit())
            || !signature_blake3.chars().all(|c| c.is_ascii_hexdigit())
        {
            return None;
        }
        Some(Self {
            symbol_name,
            commit_hash,
            signature_blake3,
            file: file.to_owned(),
            line,
        })
    }
}

impl fmt::Display for SymbolIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_wire())
    }
}

// ---------------------------------------------------------------------------
// ValidationScope — repo + branch + commit-range
// ---------------------------------------------------------------------------

/// The checkout scope a semantic re-derivation is validated against. A fact
/// stored against commit A is re-derived against the current checkout's
/// commit B within this scope; if the symbol's signature at B differs from
/// A, the fact is STALE.
///
/// `commit_range` is optional: `None` means "validate against HEAD
/// regardless of range" (the simple case); `Some((from, to))` means
/// "validate against the tip `to` of a range whose base is `from`" — the
/// branch-scope guardrail from the plan's open-questions section
/// (revalidate against the relevant checkout, not only HEAD).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationScope {
    /// Repo identifier (path or URL — the concrete impl decides).
    pub repo: String,
    /// Branch name the fact was authored against.
    pub branch: String,
    /// Optional `(base_commit, tip_commit)` range for scoped revalidation.
    pub commit_range: Option<(String, String)>,
}

impl ValidationScope {
    /// Construct a HEAD-scoped validation (no range constraint).
    pub fn at_head(repo: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            branch: branch.into(),
            commit_range: None,
        }
    }
}

// ---------------------------------------------------------------------------
// SemanticValidator TRAIT
// ---------------------------------------------------------------------------

/// Seam for commit-aware semantic re-derivation. The concrete impl lives
/// outside `crates/memory` (in `crates/cli` or wherever the DKG / LSP index
/// is opened) and resolves the current checkout's [`SymbolIdentity`] for a
/// cited symbol.
///
/// # Contract
/// - [`SemanticValidator::rederive`] MUST return `Err` (not panic) when the
///   underlying index is unavailable → caller maps to `Validity::Unknown`.
/// - `Ok(None)` means the symbol does not exist in the current checkout
///   (renamed/deleted) → caller maps to `Validity::Stale`.
/// - `Ok(Some(identity))` means the symbol exists with this identity; the
///   caller compares `identity` against the stored identity for semantic
///   equality.
pub trait SemanticValidator: Send + Sync {
    /// Re-derive the symbol's current identity from the checkout scoped by
    /// `scope`. See the contract above for return-value semantics.
    fn rederive(
        &self,
        pointer: &CitationPointer,
        scope: &ValidationScope,
    ) -> Result<Option<SymbolIdentity>, SemanticError>;
}

/// Error type for semantic re-derivation. Kept separate from
/// [`crate::error::MemoryError`] so this module does not amend the E-5
/// error enum. The caller maps `SemanticError` to `Validity::Unknown`.
#[derive(Debug, thiserror::Error)]
pub enum SemanticError {
    /// The DKG / LSP index is unavailable (not opened, file missing, etc.).
    #[error("semantic index unavailable: {0}")]
    IndexUnavailable(String),
    /// The index returned a parse / query error.
    #[error("semantic query error: {0}")]
    Query(String),
}

// ---------------------------------------------------------------------------
// SemanticValidity — the upgraded verdict
// ---------------------------------------------------------------------------

/// Result of commit-aware semantic validation. Extends H-32's [`Validity`]
/// with a `SemanticStale` arm: the symbol EXISTS (H-32 passes) but its
/// signature/behavior CHANGED (semantic identity mismatch).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticValidity {
    /// The fact is safe to inject: Tier-B (no citation needed) OR Tier-A
    /// with existence + semantic identity both confirmed.
    Valid,
    /// H-32 existence check failed (symbol renamed/deleted/moved). The fact
    /// is HARD-EXCLUDED; the H-32 `fact_validation_failed` SSB row applies.
    ExistenceStale,
    /// H-32 existence check passed BUT semantic identity mismatch: the
    /// symbol exists at the cited path but its signature/behavior changed
    /// since the fact was stored. HARD-EXCLUDED; a J-21
    /// `MemoryValidationEvent::SemanticRevalidated` SSB row applies.
    SemanticStale,
    /// The validator could not run (index unavailable, parse error, no
    /// stored semantic identity for a Tier-A fact). FAIL-CLOSED per R2.
    Unknown,
}

impl fmt::Display for SemanticValidity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticValidity::Valid => f.write_str("valid"),
            SemanticValidity::ExistenceStale => f.write_str("existence_stale"),
            SemanticValidity::SemanticStale => f.write_str("semantic_stale"),
            SemanticValidity::Unknown => f.write_str("unknown"),
        }
    }
}

// ---------------------------------------------------------------------------
// extract_stored_identity — pull the sem: pointer from evidence_pointers
// ---------------------------------------------------------------------------

/// Extract the stored [`SymbolIdentity`] from a fact's `evidence_pointers`.
///
/// Scans for a `sem:<name>@<commit>:<sig>` entry and pairs it with the
/// `dkg:` citation pointer's `file` + `line` (the semantic identity reuses
/// the DKG pointer's location). Returns `None` if no `sem:` pointer is
/// found (pre-J-21 facts have no semantic identity → fail-closed at
/// validation time).
pub fn extract_stored_identity(fact: &MemoryFact) -> Option<SymbolIdentity> {
    let dkg = fact
        .evidence_pointers
        .iter()
        .find_map(|p| CitationPointer::parse(p))?;
    fact.evidence_pointers
        .iter()
        .find_map(|p| SymbolIdentity::parse_wire(p, &dkg.file, dkg.line))
}

// ---------------------------------------------------------------------------
// revalidate_semantic — the upgraded gate
// ---------------------------------------------------------------------------

/// Re-validate a Tier-A fact's semantic identity against the current
/// checkout, composing H-32 existence + J-21 semantic identity.
///
/// # Stages
/// 1. **Tier-B short-circuit**: Tier-B facts (preference-class) return
///    `Valid` immediately — no citation or semantic identity needed.
/// 2. **Existence (H-32)**: delegate to `existence_validator`. `Stale` →
///    `ExistenceStale`; `Unknown` → `Unknown`.
/// 3. **Stored identity**: extract the `sem:` pointer. If none, `Unknown`
///    (fail-closed: pre-J-21 Tier-A facts cannot be semantically validated).
/// 4. **Re-derivation**: call `semantic_validator.rederive` with the DKG
///    pointer + scope. `Err` → `Unknown`; `Ok(None)` → `SemanticStale`
///    (symbol vanished from the scoped checkout); `Ok(Some(current))` →
///    compare `current.signature_blake3` against the stored identity's
///    `signature_blake3`. Mismatch → `SemanticStale`; match → `Valid`.
///
/// # Fail-closed discipline (R2)
/// Missing evidence is a hard FAIL, never a SKIP. Every `Unknown` returned
/// here causes the retrieval gate to hard-exclude the fact, never silently
/// inject it.
pub fn revalidate_semantic(
    fact: &MemoryFact,
    existence_validator: &dyn CitationValidator,
    semantic_validator: &dyn SemanticValidator,
    scope: &ValidationScope,
) -> SemanticValidity {
    // Stage 1: Tier-B short-circuit.
    if classify_fact_tier(fact.fact_type) == Tier::B {
        return SemanticValidity::Valid;
    }

    // Stage 2: H-32 existence check.
    let dkg_pointer = match fact
        .evidence_pointers
        .iter()
        .find_map(|p| CitationPointer::parse(p))
    {
        Some(p) => p,
        None => return SemanticValidity::Unknown, // no dkg: pointer → fail-closed
    };
    match crate::citation_validity::revalidate_citation(fact, existence_validator) {
        Validity::Valid => {} // proceed to semantic stage
        Validity::Stale => return SemanticValidity::ExistenceStale,
        Validity::Unknown => return SemanticValidity::Unknown,
    }

    // Stage 3: extract stored semantic identity.
    let stored = match extract_stored_identity(fact) {
        Some(id) => id,
        None => return SemanticValidity::Unknown, // pre-J-21 fact → fail-closed
    };

    // Stage 4: re-derive current identity and compare signatures.
    let current = match semantic_validator.rederive(&dkg_pointer, scope) {
        Ok(Some(id)) => id,
        Ok(None) => return SemanticValidity::SemanticStale, // symbol vanished
        Err(_) => return SemanticValidity::Unknown,         // index unavailable
    };
    if current.signature_blake3 == stored.signature_blake3 {
        SemanticValidity::Valid
    } else {
        SemanticValidity::SemanticStale
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citation_validity::{CitationValidator, NullValidator, Validity};
    use crate::fact::{FactType, MemoryFact, Scope};
    use crate::trust::{Provenance, TrustTier};

    fn make_tier_a_fact(evidence: Vec<String>) -> MemoryFact {
        MemoryFact {
            fact_id: "fact-sem-1".into(),
            user_id: "u1".into(),
            seat_id: "local".into(),
            project_id: Some("proj-1".into()),
            scope: Scope::Project,
            fact_type: FactType::DomainKnowledge,
            key: "no_unwrap_in_prod".into(),
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
            seat_id: "local".into(),
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
            evidence_pointers: vec!["dkg:foo@bar.rs:1".into()],
            superseded_by: None,
            trust_tier: TrustTier::T0UserDirect,
            provenance: Provenance::user_direct("test"),
            last_validated_at_ms: None,
        }
    }

    const DKG_PTR: &str = "dkg:validate_jwt@crates/auth/src/lib.rs:42";
    const SEM_PTR: &str = "sem:validate_jwt@abc123def456:f1e2d3c4b5a69788";

    struct AlwaysValidExistence;
    impl CitationValidator for AlwaysValidExistence {
        fn validate(&self, _fact: &MemoryFact) -> Validity {
            Validity::Valid
        }
    }

    struct AlwaysStaleExistence;
    impl CitationValidator for AlwaysStaleExistence {
        fn validate(&self, _fact: &MemoryFact) -> Validity {
            Validity::Stale
        }
    }

    /// Semantic validator that returns a fixed identity matching the stored sig.
    struct MatchingSemantic;
    impl SemanticValidator for MatchingSemantic {
        fn rederive(
            &self,
            _pointer: &CitationPointer,
            _scope: &ValidationScope,
        ) -> Result<Option<SymbolIdentity>, SemanticError> {
            Ok(Some(SymbolIdentity {
                symbol_name: "validate_jwt".into(),
                commit_hash: "newcommit".into(),
                signature_blake3: "f1e2d3c4b5a69788".into(),
                file: "crates/auth/src/lib.rs".into(),
                line: 42,
            }))
        }
    }

    /// Semantic validator that returns a DIFFERENT signature (refactored body).
    struct RefactoredSemantic;
    impl SemanticValidator for RefactoredSemantic {
        fn rederive(
            &self,
            _pointer: &CitationPointer,
            _scope: &ValidationScope,
        ) -> Result<Option<SymbolIdentity>, SemanticError> {
            Ok(Some(SymbolIdentity {
                symbol_name: "validate_jwt".into(),
                commit_hash: "newcommit".into(),
                signature_blake3: "aaaaaaaaaaaaaaaa".into(), // different sig
                file: "crates/auth/src/lib.rs".into(),
                line: 42,
            }))
        }
    }

    /// Semantic validator that returns Ok(None) — symbol vanished from checkout.
    struct VanishedSemantic;
    impl SemanticValidator for VanishedSemantic {
        fn rederive(
            &self,
            _pointer: &CitationPointer,
            _scope: &ValidationScope,
        ) -> Result<Option<SymbolIdentity>, SemanticError> {
            Ok(None)
        }
    }

    /// Semantic validator that returns Err — index unavailable.
    struct UnavailableSemantic;
    impl SemanticValidator for UnavailableSemantic {
        fn rederive(
            &self,
            _pointer: &CitationPointer,
            _scope: &ValidationScope,
        ) -> Result<Option<SymbolIdentity>, SemanticError> {
            Err(SemanticError::IndexUnavailable("no DKG index".into()))
        }
    }

    // --- SymbolIdentity wire format ---

    #[test]
    fn symbol_identity_round_trips_through_wire() {
        let id = SymbolIdentity {
            symbol_name: "validate_jwt".into(),
            commit_hash: "abc123".into(),
            signature_blake3: "f1e2d3c4b5a69788".into(),
            file: "src/lib.rs".into(),
            line: 42,
        };
        let wire = id.to_wire();
        let parsed = SymbolIdentity::parse_wire(&wire, "src/lib.rs", 42).expect("parse");
        assert_eq!(parsed, id);
    }

    #[test]
    fn symbol_identity_rejects_missing_prefix() {
        assert!(SymbolIdentity::parse_wire("validate_jwt@abc:f1e2", "f", 1).is_none());
    }

    #[test]
    fn symbol_identity_rejects_non_hex_commit() {
        assert!(SymbolIdentity::parse_wire("sem:fn@nonhex!:f1e2", "f", 1).is_none());
    }

    #[test]
    fn symbol_identity_rejects_empty_fields() {
        assert!(SymbolIdentity::parse_wire("sem:@abc:f1e2", "f", 1).is_none());
        assert!(SymbolIdentity::parse_wire("sem:fn@:f1e2", "f", 1).is_none());
        assert!(SymbolIdentity::parse_wire("sem:fn@abc:", "f", 1).is_none());
    }

    // --- extract_stored_identity ---

    #[test]
    fn extract_stored_identity_finds_sem_pointer() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let id = extract_stored_identity(&fact).expect("found");
        assert_eq!(id.symbol_name, "validate_jwt");
        assert_eq!(id.commit_hash, "abc123def456");
        assert_eq!(id.signature_blake3, "f1e2d3c4b5a69788");
        assert_eq!(id.file, "crates/auth/src/lib.rs");
        assert_eq!(id.line, 42);
    }

    #[test]
    fn extract_stored_identity_returns_none_without_sem_pointer() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into()]);
        assert!(extract_stored_identity(&fact).is_none());
    }

    // --- revalidate_semantic: the upgraded gate ---

    #[test]
    fn tier_b_short_circuits_to_valid() {
        let fact = make_tier_b_fact();
        let result = revalidate_semantic(
            &fact,
            &NullValidator,
            &UnavailableSemantic,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(result, SemanticValidity::Valid);
    }

    #[test]
    fn tier_a_no_dkg_pointer_returns_unknown() {
        let fact = make_tier_a_fact(vec!["sess-123".into()]);
        let result = revalidate_semantic(
            &fact,
            &AlwaysValidExistence,
            &MatchingSemantic,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(result, SemanticValidity::Unknown);
    }

    #[test]
    fn tier_a_no_sem_pointer_returns_unknown_fail_closed() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into()]);
        let result = revalidate_semantic(
            &fact,
            &AlwaysValidExistence,
            &MatchingSemantic,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(
            result,
            SemanticValidity::Unknown,
            "pre-J-21 Tier-A facts with no sem: pointer must fail-closed, not wave through"
        );
    }

    #[test]
    fn tier_a_existence_stale_returns_existence_stale() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let result = revalidate_semantic(
            &fact,
            &AlwaysStaleExistence,
            &MatchingSemantic,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(result, SemanticValidity::ExistenceStale);
    }

    #[test]
    fn tier_a_matching_signature_returns_valid() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let result = revalidate_semantic(
            &fact,
            &AlwaysValidExistence,
            &MatchingSemantic,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(result, SemanticValidity::Valid);
    }

    #[test]
    fn tier_a_refactored_signature_returns_semantic_stale() {
        // The KEY test: existence passes (symbol still at file:line) but the
        // signature changed — the fact is "valid" under H-32 but STALE under
        // J-21. This is the delta the semantic gate closes.
        let fact = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let result = revalidate_semantic(
            &fact,
            &AlwaysValidExistence,
            &RefactoredSemantic,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(
            result,
            SemanticValidity::SemanticStale,
            "existence-only validation would wave this through; semantic check must catch it"
        );
    }

    #[test]
    fn tier_a_vanished_symbol_returns_semantic_stale() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let result = revalidate_semantic(
            &fact,
            &AlwaysValidExistence,
            &VanishedSemantic,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(result, SemanticValidity::SemanticStale);
    }

    #[test]
    fn tier_a_unavailable_index_returns_unknown_fail_closed() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let result = revalidate_semantic(
            &fact,
            &AlwaysValidExistence,
            &UnavailableSemantic,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(
            result,
            SemanticValidity::Unknown,
            "index unavailable must fail-closed (R2), never silently inject"
        );
    }

    #[test]
    fn validation_scope_at_head_has_no_range() {
        let scope = ValidationScope::at_head("my-repo", "feature-branch");
        assert_eq!(scope.repo, "my-repo");
        assert_eq!(scope.branch, "feature-branch");
        assert!(scope.commit_range.is_none());
    }

    #[test]
    fn semantic_validity_display_round_trips() {
        assert_eq!(SemanticValidity::Valid.to_string(), "valid");
        assert_eq!(
            SemanticValidity::ExistenceStale.to_string(),
            "existence_stale"
        );
        assert_eq!(
            SemanticValidity::SemanticStale.to_string(),
            "semantic_stale"
        );
        assert_eq!(SemanticValidity::Unknown.to_string(), "unknown");
    }
}
