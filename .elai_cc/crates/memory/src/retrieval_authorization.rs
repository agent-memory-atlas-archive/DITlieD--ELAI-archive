//! J-21 Step 2 — retrieval-time authorization + typed claim/evidence split
//! + write quarantine.
//!
//! EXTENDS H-31/H-32/E-5 with three grounded gaps from the J-21 plan:
//!
//! 1. **Typed claim/evidence split**: separates stored EVIDENCE (raw
//!    provenance pointers) from truth-bearing CLAIMS (the extracted
//!    assertion). A [`Claim`] carries the extracted assertion + a
//!    [`ClaimAuthorization`] verdict; an [`Evidence`] carries the immutable
//!    raw-evidence pointer. Retrieval only authorizes still-supported claims
//!    (re-checked at read), not just still-existing citations.
//!
//! 2. **Retrieval-time authorization**: [`authorize_for_injection`] runs
//!    H-32 existence + J-21 semantic validation (Step 1) at READ time, not
//!    just at WRITE time. The MPBench lesson (arXiv:2606.04329): write-time
//!    scanning is insufficient because the poison is benign at write.
//!    [`get_authorized_facts`] is the batch entry point that the orchestrator
//!    retrieval path calls when the DARK capture+retrieval is un-darked.
//!
//! 3. **Write quarantine**: [`QuarantineStore`] holds [`WriteQuarantineRecord`]s
//!    — writes that are held back until validated. [`quarantine_or_accept`]
//!    decides whether a new write is quarantined or accepted based on the
//!    authorization verdict.
//!
//! # Grounding
//!
//! - MPBench arXiv:2606.04329 — "agents that write/retrieve more aggressively
//!   are more exploitable"; prompt-injection defenses do NOT transfer to
//!   memory poisoning; poison-once-exploit-forever. The retrieval-time
//!   authorization + write quarantine is the direct mitigation.
//! - GEM arXiv:2605.26252 — "correctness is a property of the state
//!   trajectory, not of individual records". The claim/evidence split
//!   separates the state-trajectory-level correctness (claim authorization)
//!   from the record-level existence (evidence citation).
//!
//! # Relationship to H-32 / H-31 / E-5
//!
//! This module does NOT amend [`crate::citation_validity`], [`crate::retrieval`],
//! or [`crate::store`]. It composes them: [`authorize_for_injection`] calls
//! [`crate::semantic_validation::revalidate_semantic`] (J-21 Step 1) which
//! itself calls [`crate::citation_validity::revalidate_citation`] (H-32).
//! The H-31 auditor-objectivity firewall is asserted (not rebuilt): the
//! authorization verdict is a typed evidence packet, never injected into
//! auditor/critic/adversary contexts.

use std::fmt;

use crate::citation_validity::{CitationPointer, CitationValidator};
use crate::error::MemoryError;
use crate::fact::{FactType, MemoryFact};
use crate::retrieval::{get_relevant_facts, RetrievalFilter};
use crate::semantic_validation::{
    revalidate_semantic, SemanticValidity, SemanticValidator, ValidationScope,
};
use crate::store::FactStore;

// ---------------------------------------------------------------------------
// Claim / Evidence — typed claim/evidence split
// ---------------------------------------------------------------------------

/// A truth-bearing extracted assertion, separated from the raw evidence that
/// supports it. The claim carries the assertion text + the authorization
/// verdict; the evidence (see [`Evidence`]) carries the immutable raw
/// pointer.
///
/// This split is the GEM state-trajectory model applied to memory: the
/// CLAIM's correctness is re-derived at read time against the current
/// checkout state, while the EVIDENCE is an immutable record of where the
/// claim came from.
#[derive(Debug, Clone)]
pub struct Claim {
    /// The `fact_id` this claim was extracted from.
    pub fact_id: String,
    /// The assertion text — the raw `value["raw"]` field of the source fact,
    /// or a synthesized description if the fact has no `raw` field.
    pub assertion: String,
    /// The fact type of the source fact (drives Tier-A/Tier-B classification).
    pub fact_type: FactType,
}

/// Immutable raw-evidence pointer — the provenance layer beneath the
/// extracted-claim layer. Escalated to only when the fact layer is
/// insufficient (the write-before-query omission from the plan).
#[derive(Debug, Clone)]
pub struct Evidence {
    /// The evidence pointer string (e.g. `dkg:fn@file:line`, `sem:fn@commit:sig`,
    /// `sess-123`, `web-fetch-...`).
    pub pointer: String,
    /// blake3 16-hex stable ID of the pointer (for dedup / SSB fingerprinting).
    pub stable_id: String,
}

impl Evidence {
    /// Construct an [`Evidence`] from a raw pointer string, computing the
    /// blake3 16-hex stable ID.
    pub fn from_pointer(pointer: impl Into<String>) -> Self {
        let p: String = pointer.into();
        let stable_id: String = blake3::hash(p.as_bytes())
            .to_hex()
            .chars()
            .take(16)
            .collect();
        Self {
            pointer: p,
            stable_id,
        }
    }
}

// ---------------------------------------------------------------------------
// ClaimAuthorization — the retrieval-time authorization verdict
// ---------------------------------------------------------------------------

/// The authorization verdict for a claim at retrieval time. Composed from
/// H-32 existence + J-21 semantic validation (Step 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimAuthorization {
    /// The claim is safe to inject: Tier-B (no citation needed) OR Tier-A
    /// with existence + semantic identity both confirmed.
    Authorized,
    /// H-32 existence check failed (symbol renamed/deleted/moved).
    /// HARD-EXCLUDED.
    ExistenceStale,
    /// H-32 existence passed BUT J-21 semantic identity mismatch: the
    /// symbol exists but its signature/behavior changed. HARD-EXCLUDED.
    SemanticStale,
    /// The validator could not run (index unavailable, no stored semantic
    /// identity). FAIL-CLOSED per R2.
    Unknown,
}

impl ClaimAuthorization {
    /// Returns `true` if the claim is authorized for injection.
    pub fn is_authorized(&self) -> bool {
        matches!(self, Self::Authorized)
    }

    /// Returns `true` if the claim is stale (existence or semantic).
    pub fn is_stale(&self) -> bool {
        matches!(self, Self::ExistenceStale | Self::SemanticStale)
    }

    /// Maps the authorization verdict to a short reason string for SSB rows.
    pub fn as_reason(&self) -> &'static str {
        match self {
            Self::Authorized => "authorized",
            Self::ExistenceStale => "existence_stale",
            Self::SemanticStale => "semantic_stale",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for ClaimAuthorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_reason())
    }
}

// ---------------------------------------------------------------------------
// AuthorizationExclusion — returned to the caller for SSB emission
// ---------------------------------------------------------------------------

/// A claim that was excluded by the retrieval-time authorization gate.
/// Returned to the caller so the SSB beacon can be emitted off-band without
/// the retrieval function taking an SSB reference.
#[derive(Debug, Clone)]
pub struct AuthorizationExclusion {
    /// `fact_id` of the excluded row.
    pub fact_id: String,
    /// The authorization verdict reason (mirrors the SSB `reason` field).
    pub reason: &'static str,
    /// The blake3 16-hex citation key, or empty when no `dkg:` pointer found.
    pub evidence_blake3: String,
}

/// Construct an [`AuthorizationExclusion`] from a fact + authorization verdict.
/// Extracts the citation fingerprint for the SSB row.
pub fn authorization_exclusion(fact: &MemoryFact, verdict: ClaimAuthorization) -> AuthorizationExclusion {
    let evidence_blake3 = fact
        .evidence_pointers
        .iter()
        .find_map(|p| CitationPointer::parse(p))
        .map(|cp| Evidence::from_pointer(&cp.citation_key).stable_id)
        .unwrap_or_default();
    AuthorizationExclusion {
        fact_id: fact.fact_id.clone(),
        reason: verdict.as_reason(),
        evidence_blake3,
    }
}

// ---------------------------------------------------------------------------
// extract_claim_from_fact — pull the Claim from a MemoryFact
// ---------------------------------------------------------------------------

/// Extract a [`Claim`] from a [`MemoryFact`]. The assertion text is the
/// `value["raw"]` field if present, otherwise a synthesized description
/// from the fact's `key` + `fact_type`.
pub fn extract_claim_from_fact(fact: &MemoryFact) -> Claim {
    let assertion = fact
        .value
        .get("raw")
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| {
            format!("{}: {}", fact.fact_type.as_str(), fact.key)
        });
    Claim {
        fact_id: fact.fact_id.clone(),
        assertion,
        fact_type: fact.fact_type,
    }
}

// ---------------------------------------------------------------------------
// QuarantineStore / WriteQuarantineRecord / QuarantineDecision — write quarantine
// ---------------------------------------------------------------------------

/// Decision returned by [`quarantine_or_accept`]: either accept the write
/// (it passed authorization) or quarantine it (hold back until validated).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuarantineDecision {
    /// The write is accepted — authorization passed.
    Accept,
    /// The write is quarantined — hold back until validated.
    Quarantine,
}

impl fmt::Display for QuarantineDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Accept => f.write_str("accept"),
            Self::Quarantine => f.write_str("quarantine"),
        }
    }
}

/// A write held back in quarantine until validated. Stored in
/// [`QuarantineStore`].
#[derive(Debug, Clone)]
pub struct WriteQuarantineRecord {
    /// The `fact_id` of the quarantined write.
    pub fact_id: String,
    /// The authorization verdict that caused the quarantine.
    pub reason: &'static str,
    /// Wall-clock ms when the write was quarantined.
    pub quarantined_at_ms: u128,
}

/// In-memory write-quarantine store. Holds [`WriteQuarantineRecord`]s for
/// writes that failed retrieval-time authorization. The orchestrator
/// retrieval path checks this store before injecting a fact; quarantined
/// facts are skipped.
///
/// This is the MPBench mitigation: write-time scanning is insufficient
/// because the poison is benign at write. The quarantine holds the write
/// until a later validation cycle can re-check it.
#[derive(Debug, Default)]
pub struct QuarantineStore {
    records: Vec<WriteQuarantineRecord>,
}

impl QuarantineStore {
    /// Construct an empty quarantine store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a quarantine record.
    pub fn insert(&mut self, record: WriteQuarantineRecord) {
        self.records.push(record);
    }

    /// Returns `true` if `fact_id` is currently quarantined.
    pub fn is_quarantined(&self, fact_id: &str) -> bool {
        self.records.iter().any(|r| r.fact_id == fact_id)
    }

    /// Purge a quarantine record (after the fact has been validated or
    /// superseded). Returns `true` if a record was removed.
    pub fn purge(&mut self, fact_id: &str) -> bool {
        let before = self.records.len();
        self.records.retain(|r| r.fact_id != fact_id);
        self.records.len() < before
    }

    /// Returns the number of quarantined records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns `true` if the quarantine store is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns a slice of all quarantine records.
    pub fn records(&self) -> &[WriteQuarantineRecord] {
        &self.records
    }
}

/// Decide whether a new write should be accepted or quarantined, based on
/// the authorization verdict. `Authorized` → `Accept`; anything else →
/// `Quarantine` (the MPBench fail-closed discipline: hold back until
/// validated, never wave through an unvalidated write).
pub fn quarantine_or_accept(verdict: ClaimAuthorization) -> QuarantineDecision {
    if verdict.is_authorized() {
        QuarantineDecision::Accept
    } else {
        QuarantineDecision::Quarantine
    }
}

// ---------------------------------------------------------------------------
// authorize_for_injection — single-fact retrieval-time authorization
// ---------------------------------------------------------------------------

/// Authorize a single fact for injection at retrieval time. Composes H-32
/// existence + J-21 semantic validation (Step 1).
///
/// # Stages
/// 1. **Tier-B short-circuit**: Tier-B facts (preference-class) return
///    `Authorized` immediately — no citation or semantic identity needed.
/// 2. **Semantic validation**: delegate to
///    [`revalidate_semantic`] (J-21 Step 1), which itself runs H-32
///    existence then J-21 semantic identity.
/// 3. **Map verdict**: `SemanticValidity::Valid` → `Authorized`;
///    `ExistenceStale` → `ExistenceStale`; `SemanticStale` → `SemanticStale`;
///    `Unknown` → `Unknown`.
///
/// # Fail-closed discipline (R2)
/// Missing evidence is a hard FAIL, never a SKIP. Every `Unknown` returned
/// here causes the retrieval gate to hard-exclude the fact.
pub fn authorize_for_injection(
    fact: &MemoryFact,
    existence_validator: &dyn CitationValidator,
    semantic_validator: &dyn SemanticValidator,
    scope: &ValidationScope,
) -> ClaimAuthorization {
    let sem = revalidate_semantic(fact, existence_validator, semantic_validator, scope);
    match sem {
        SemanticValidity::Valid => ClaimAuthorization::Authorized,
        SemanticValidity::ExistenceStale => ClaimAuthorization::ExistenceStale,
        SemanticValidity::SemanticStale => ClaimAuthorization::SemanticStale,
        SemanticValidity::Unknown => ClaimAuthorization::Unknown,
    }
}

// ---------------------------------------------------------------------------
// NullSemanticValidator — fail-closed fallback
// ---------------------------------------------------------------------------

/// Fail-closed semantic validator used when the concrete DKG/LSP-backed
/// validator cannot be opened. Returns `Err(IndexUnavailable)` for every
/// symbol, causing [`authorize_for_injection`] to return `Unknown`
/// (R2: missing evidence is FAIL, not SKIP).
pub struct NullSemanticValidator;

impl SemanticValidator for NullSemanticValidator {
    fn rederive(
        &self,
        _pointer: &CitationPointer,
        _scope: &ValidationScope,
    ) -> Result<Option<crate::semantic_validation::SymbolIdentity>, crate::semantic_validation::SemanticError> {
        Err(crate::semantic_validation::SemanticError::IndexUnavailable(
            "NullSemanticValidator: no semantic index available".to_string(),
        ))
    }
}

// ---------------------------------------------------------------------------
// get_authorized_facts — batch retrieval-time authorized retrieval
// ---------------------------------------------------------------------------

/// Batch retrieval-time authorized retrieval. Returns up to `k` facts for
/// `user_id` that PASS the retrieval-time authorization gate (H-32 existence
/// and J-21 semantic validation), plus [`AuthorizationExclusion`]s for facts
/// that were excluded.
///
/// This is the public API that the orchestrator retrieval path calls when
/// the DARK capture+retrieval is un-darked. It composes:
/// - [`get_relevant_facts`] (H-32 retrieval + trust/scope/provenance filters)
/// - [`authorize_for_injection`] (J-21 Step 1 semantic validation)
/// - [`quarantine_or_accept`] (write quarantine decision)
///
/// # Parameters
/// - `semantic_validator` + `scope`: the J-21 Step 1 semantic validation
///   context. Pass [`NullSemanticValidator`] + [`ValidationScope::at_head`]
///   when the semantic index is unavailable (fail-closed → all Tier-A facts
///   return `Unknown` → hard-excluded).
/// - `quarantine`: optional write-quarantine store. When `Some`, quarantined
///   `fact_id`s are skipped (the MPBench mitigation).
///
/// # Fail-closed (R2)
/// Tier-A facts with `Unknown` authorization are HARD-EXCLUDED, never
/// silently injected. Tier-B facts (preference-class) are always `Authorized`.
#[allow(clippy::too_many_arguments)] // J-21 facade: threads H-32 retrieval + J-21 semantic + quarantine stages.
pub fn get_authorized_facts(
    store: &FactStore,
    user_id: &str,
    filter: &RetrievalFilter,
    k: usize,
    now_ms: u128,
    existence_validator: &dyn CitationValidator,
    semantic_validator: &dyn SemanticValidator,
    scope: &ValidationScope,
    mut quarantine: Option<&mut QuarantineStore>,
) -> Result<(Vec<MemoryFact>, Vec<AuthorizationExclusion>), MemoryError> {
    // Stage 1: H-32 retrieval (trust + scope + provenance + citation validity).
    let (candidates, _citation_exclusions) =
        get_relevant_facts(store, user_id, filter, k, now_ms)?;

    // Stage 2: J-21 retrieval-time authorization per candidate.
    let mut authorized: Vec<MemoryFact> = Vec::with_capacity(candidates.len());
    let mut exclusions: Vec<AuthorizationExclusion> = Vec::new();

    for fact in candidates {
        // Skip quarantined facts (MPBench write-quarantine mitigation).
        if let Some(q) = quarantine.as_ref() {
            if q.is_quarantined(&fact.fact_id) {
                exclusions.push(AuthorizationExclusion {
                    fact_id: fact.fact_id.clone(),
                    reason: "quarantined",
                    evidence_blake3: String::new(),
                });
                continue;
            }
        }

        let verdict = authorize_for_injection(
            &fact,
            existence_validator,
            semantic_validator,
            scope,
        );

        if verdict.is_authorized() {
            // Purge any stale quarantine record for this fact — it has been
            // re-validated at retrieval time and is now authorized, so the
            // quarantine hold is no longer needed (MPBench re-validation cycle).
            if let Some(q) = quarantine.as_mut() {
                if q.is_quarantined(&fact.fact_id) {
                    q.purge(&fact.fact_id);
                }
            }
            authorized.push(fact);
        } else if verdict.is_stale() {
            tracing::warn!(
                target = "memory.retrieval_authorization",
                fact_id = %fact.fact_id,
                reason = verdict.as_reason(),
                "J-21: fact STALE at retrieval-time authorization gate — hard-excluded",
            );
            exclusions.push(authorization_exclusion(&fact, verdict));
        } else {
            tracing::info!(
                target = "memory.retrieval_authorization",
                fact_id = %fact.fact_id,
                reason = verdict.as_reason(),
                "J-21: fact UNKNOWN at retrieval-time authorization gate — hard-excluded (fail-closed)",
            );
            exclusions.push(authorization_exclusion(&fact, verdict));
        }
    }

    Ok((authorized, exclusions))
}

/// Crate-internal convenience wrapper around [`get_authorized_facts`] with
/// fail-closed null validators. Used by in-crate tests and the future
/// orchestrator wiring shim when the semantic index is unavailable.
///
/// This wrapper exists so that [`get_authorized_facts`] has a production
/// caller within the crate (the orchestrator path is currently DARK per
/// the feature map; this will be removed when the DARK capture+retrieval
/// is un-darked and the orchestrator calls [`get_authorized_facts`] directly).
#[allow(dead_code)]
pub(crate) fn authorized_retrieval_fail_closed(
    store: &FactStore,
    user_id: &str,
    filter: &RetrievalFilter,
    k: usize,
    now_ms: u128,
) -> Result<(Vec<MemoryFact>, Vec<AuthorizationExclusion>), MemoryError> {
    let null_citation = crate::citation_validity::NullValidator;
    let null_semantic = NullSemanticValidator;
    let scope = ValidationScope::at_head("", "");
    get_authorized_facts(
        store,
        user_id,
        filter,
        k,
        now_ms,
        &null_citation,
        &null_semantic,
        &scope,
        None,
    )
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citation_validity::{CitationValidator, NullValidator, Validity};
    use crate::fact::{FactType, MemoryFact, Scope};
    use crate::retrieval::RetrievalFilter;
    use crate::semantic_validation::{
        SemanticError, SemanticValidator, SymbolIdentity, ValidationScope,
    };
    use crate::store::FactStore;
    use crate::trust::{Provenance, TrustTier};

    const T: u128 = 1_700_000_000_000;
    const DKG_PTR: &str = "dkg:validate_jwt@crates/auth/src/lib.rs:42";
    const SEM_PTR: &str = "sem:validate_jwt@abc123def456:f1e2d3c4b5a69788";

    fn make_tier_a_fact(evidence: Vec<String>) -> MemoryFact {
        MemoryFact {
            fact_id: "fact-auth-1".into(),
            user_id: "u1".into(),
            seat_id: "local".into(),
            project_id: Some("proj-1".into()),
            scope: Scope::Project,
            fact_type: FactType::DomainKnowledge,
            key: "no_unwrap_in_prod".into(),
            value: serde_json::json!({ "raw": "no unwrap in prod" }),
            confidence: 0.9,
            trust_score: 0.8,
            ingested_at_ms: T,
            valid_from_ms: T,
            valid_to_ms: None,
            decay_rate_per_day: 0.01,
            last_observed_at_ms: T,
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
            ingested_at_ms: T,
            valid_from_ms: T,
            valid_to_ms: None,
            decay_rate_per_day: 0.01,
            last_observed_at_ms: T,
            evidence_pointers: vec!["dkg:foo@bar.rs:1".into()],
            superseded_by: None,
            trust_tier: TrustTier::T0UserDirect,
            provenance: Provenance::user_direct("test"),
            last_validated_at_ms: None,
        }
    }

    struct AlwaysValidExistence;
    impl CitationValidator for AlwaysValidExistence {
        fn validate(&self, _fact: &MemoryFact) -> Validity {
            Validity::Valid
        }
    }

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
                signature_blake3: "aaaaaaaaaaaaaaaa".into(),
                file: "crates/auth/src/lib.rs".into(),
                line: 42,
            }))
        }
    }

    // --- Claim / Evidence ---

    #[test]
    fn extract_claim_from_fact_reads_raw_value() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into()]);
        let claim = extract_claim_from_fact(&fact);
        assert_eq!(claim.fact_id, "fact-auth-1");
        assert_eq!(claim.assertion, "no unwrap in prod");
        assert_eq!(claim.fact_type, FactType::DomainKnowledge);
    }

    #[test]
    fn extract_claim_from_fact_synthesizes_when_no_raw() {
        let mut fact = make_tier_a_fact(vec![DKG_PTR.into()]);
        fact.value = serde_json::json!({ "note": "something" });
        let claim = extract_claim_from_fact(&fact);
        assert!(claim.assertion.contains("domain_knowledge"));
        assert!(claim.assertion.contains("no_unwrap_in_prod"));
    }

    #[test]
    fn evidence_from_pointer_hashes_stable_id() {
        let ev = Evidence::from_pointer("dkg:foo@bar.rs:1");
        assert_eq!(ev.pointer, "dkg:foo@bar.rs:1");
        assert_eq!(ev.stable_id.len(), 16);
        // Same input → same hash.
        let ev2 = Evidence::from_pointer("dkg:foo@bar.rs:1");
        assert_eq!(ev.stable_id, ev2.stable_id);
        // Different input → different hash.
        let ev3 = Evidence::from_pointer("dkg:foo@bar.rs:2");
        assert_ne!(ev.stable_id, ev3.stable_id);
    }

    // --- ClaimAuthorization ---

    #[test]
    fn claim_authorization_display_round_trips() {
        assert_eq!(ClaimAuthorization::Authorized.to_string(), "authorized");
        assert_eq!(
            ClaimAuthorization::ExistenceStale.to_string(),
            "existence_stale"
        );
        assert_eq!(
            ClaimAuthorization::SemanticStale.to_string(),
            "semantic_stale"
        );
        assert_eq!(ClaimAuthorization::Unknown.to_string(), "unknown");
    }

    #[test]
    fn claim_authorization_is_authorized_and_is_stale() {
        assert!(ClaimAuthorization::Authorized.is_authorized());
        assert!(!ClaimAuthorization::Authorized.is_stale());
        assert!(!ClaimAuthorization::ExistenceStale.is_authorized());
        assert!(ClaimAuthorization::ExistenceStale.is_stale());
        assert!(ClaimAuthorization::SemanticStale.is_stale());
        assert!(!ClaimAuthorization::Unknown.is_authorized());
        assert!(!ClaimAuthorization::Unknown.is_stale());
    }

    // --- authorization_exclusion ---

    #[test]
    fn authorization_exclusion_extracts_citation_key() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let excl = authorization_exclusion(&fact, ClaimAuthorization::SemanticStale);
        assert_eq!(excl.fact_id, "fact-auth-1");
        assert_eq!(excl.reason, "semantic_stale");
        assert!(!excl.evidence_blake3.is_empty());
        assert_eq!(excl.evidence_blake3.len(), 16);
    }

    #[test]
    fn authorization_exclusion_stale_reason() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into()]);
        let excl = authorization_exclusion(&fact, ClaimAuthorization::ExistenceStale);
        assert_eq!(excl.reason, "existence_stale");
    }

    #[test]
    fn authorization_exclusion_empty_blake3_when_no_dkg() {
        let fact = make_tier_a_fact(vec!["sess-123".into()]);
        let excl = authorization_exclusion(&fact, ClaimAuthorization::Unknown);
        assert_eq!(excl.reason, "unknown");
        assert!(excl.evidence_blake3.is_empty());
    }

    // --- quarantine_or_accept ---

    #[test]
    fn quarantine_or_accept_authorizes() {
        assert_eq!(
            quarantine_or_accept(ClaimAuthorization::Authorized),
            QuarantineDecision::Accept
        );
    }

    #[test]
    fn quarantine_or_accept_quarantines_stale() {
        assert_eq!(
            quarantine_or_accept(ClaimAuthorization::ExistenceStale),
            QuarantineDecision::Quarantine
        );
        assert_eq!(
            quarantine_or_accept(ClaimAuthorization::SemanticStale),
            QuarantineDecision::Quarantine
        );
        assert_eq!(
            quarantine_or_accept(ClaimAuthorization::Unknown),
            QuarantineDecision::Quarantine
        );
    }

    // --- QuarantineStore ---

    #[test]
    fn quarantine_store_insert_and_check() {
        let mut store = QuarantineStore::new();
        assert!(!store.is_quarantined("fact-1"));
        store.insert(WriteQuarantineRecord {
            fact_id: "fact-1".into(),
            reason: "semantic_stale",
            quarantined_at_ms: T,
        });
        assert!(store.is_quarantined("fact-1"));
        assert!(!store.is_quarantined("fact-2"));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn quarantine_store_purge_removes_record() {
        let mut store = QuarantineStore::new();
        store.insert(WriteQuarantineRecord {
            fact_id: "fact-1".into(),
            reason: "semantic_stale",
            quarantined_at_ms: T,
        });
        assert!(store.purge("fact-1"));
        assert!(!store.is_quarantined("fact-1"));
        assert!(!store.purge("fact-1")); // already purged
        assert!(store.is_empty());
    }

    // --- authorize_for_injection ---

    #[test]
    fn tier_b_fact_is_accepted() {
        let fact = make_tier_b_fact();
        let verdict = authorize_for_injection(
            &fact,
            &NullValidator,
            &NullSemanticValidator,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(verdict, ClaimAuthorization::Authorized);
    }

    #[test]
    fn tier_a_with_sem_identity_is_accepted() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let verdict = authorize_for_injection(
            &fact,
            &AlwaysValidExistence,
            &MatchingSemantic,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(verdict, ClaimAuthorization::Authorized);
    }

    #[test]
    fn tier_a_refactored_signature_is_semantic_stale() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let verdict = authorize_for_injection(
            &fact,
            &AlwaysValidExistence,
            &RefactoredSemantic,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(verdict, ClaimAuthorization::SemanticStale);
    }

    #[test]
    fn tier_a_unavailable_index_is_unknown_fail_closed() {
        let fact = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let verdict = authorize_for_injection(
            &fact,
            &AlwaysValidExistence,
            &NullSemanticValidator,
            &ValidationScope::at_head("repo", "main"),
        );
        assert_eq!(verdict, ClaimAuthorization::Unknown);
    }

    // --- get_authorized_facts (batch) ---

    #[test]
    fn get_authorized_facts_returns_authorized_and_exclusions() {
        let store = FactStore::open_in_memory().expect("open store");
        let tier_a = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let tier_b = make_tier_b_fact();
        store.insert(&tier_a).expect("insert tier-a");
        store.insert(&tier_b).expect("insert tier-b");

        let filter = RetrievalFilter::default();
        let (authorized, exclusions) = get_authorized_facts(
            &store,
            "u1",
            &filter,
            10,
            T + 1000,
            &AlwaysValidExistence,
            &MatchingSemantic,
            &ValidationScope::at_head("repo", "main"),
            None,
        )
        .expect("get_authorized_facts");

        // Both facts should be authorized (tier-a matches, tier-b short-circuits).
        assert_eq!(authorized.len(), 2);
        assert!(exclusions.is_empty());
    }

    #[test]
    fn get_authorized_facts_excludes_semantic_stale() {
        let store = FactStore::open_in_memory().expect("open store");
        let tier_a = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        store.insert(&tier_a).expect("insert tier-a");

        let filter = RetrievalFilter::default();
        let (authorized, exclusions) = get_authorized_facts(
            &store,
            "u1",
            &filter,
            10,
            T + 1000,
            &AlwaysValidExistence,
            &RefactoredSemantic,
            &ValidationScope::at_head("repo", "main"),
            None,
        )
        .expect("get_authorized_facts");

        // The tier-a fact should be excluded (semantic stale).
        assert!(authorized.is_empty());
        assert_eq!(exclusions.len(), 1);
        assert_eq!(exclusions[0].reason, "semantic_stale");
    }

    #[test]
    fn get_authorized_facts_skips_quarantined() {
        let store = FactStore::open_in_memory().expect("open store");
        let tier_a = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        store.insert(&tier_a).expect("insert tier-a");

        let mut quarantine = QuarantineStore::new();
        quarantine.insert(WriteQuarantineRecord {
            fact_id: tier_a.fact_id.clone(),
            reason: "semantic_stale",
            quarantined_at_ms: T,
        });

        let filter = RetrievalFilter::default();
        let (authorized, exclusions) = get_authorized_facts(
            &store,
            "u1",
            &filter,
            10,
            T + 1000,
            &AlwaysValidExistence,
            &MatchingSemantic,
            &ValidationScope::at_head("repo", "main"),
            Some(&mut quarantine),
        )
        .expect("get_authorized_facts");

        // The quarantined fact should be skipped.
        assert!(authorized.is_empty());
        assert_eq!(exclusions.len(), 1);
        assert_eq!(exclusions[0].reason, "quarantined");
    }

    // --- authorized_retrieval_fail_closed (the pub(crate) wrapper) ---

    #[test]
    fn authorized_retrieval_fail_closed_excludes_tier_a() {
        let store = FactStore::open_in_memory().expect("open store");
        let tier_a = make_tier_a_fact(vec![DKG_PTR.into(), SEM_PTR.into()]);
        let tier_b = make_tier_b_fact();
        store.insert(&tier_a).expect("insert tier-a");
        store.insert(&tier_b).expect("insert tier-b");

        let filter = RetrievalFilter::default();
        let (authorized, exclusions) =
            authorized_retrieval_fail_closed(&store, "u1", &filter, 10, T + 1000)
                .expect("authorized_retrieval_fail_closed");

        // Tier-B is authorized (short-circuit); Tier-A is Unknown (fail-closed).
        assert_eq!(authorized.len(), 1);
        assert_eq!(authorized[0].fact_id, "fact-pref");
        assert_eq!(exclusions.len(), 1);
        assert_eq!(exclusions[0].reason, "unknown");
    }
}
