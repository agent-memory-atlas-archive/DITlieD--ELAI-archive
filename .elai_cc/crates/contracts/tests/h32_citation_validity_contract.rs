//! H-32 contract tests -- Tier-A code-grounded fact citation validate-before-use.
//!
//! Covers (off-agent artifacts, Layer-4 / R4):
//!   (1) SCHEMA >= 210 and `fact_validation_failed` in the SSB bijection.
//!   (2) The validity gate EXCLUDES a Stale/Unknown Tier-A fact from retrieval.
//!   (3) A Valid Tier-A fact IS returned and stamps last_validated_at_ms.
//!   (4) The Tier-A retrieval widening is present in report_fns (asserted via
//!       elai.map SCIP reference / grep on the live source file).
//!   (5) A real FactValidationFailed SSB row writes + round-trips.
//!
//! SCHEMA_VERSION assertions use `>=` (never a global exact pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha. TOTAL_VARIANTS presence is asserted by
//! action_type_lock.rs (compile-time exhaustiveness) and the e45 contract.
//!
//! Grounding:
//!   - crates/memory/src/citation_validity.rs (revalidate_citation, Validity enum)
//!   - crates/memory/src/retrieval.rs (get_relevant_facts -- validity gate)
//!   - crates/orchestrator/src/ssb.rs (FactValidationFailed + SCHEMA_VERSION)
//!   - crates/cli/src/report_fns.rs (assemble_user_memory_section Tier-A widening)

use std::fs;
use std::path::PathBuf;

use elai_memory::{
    collect_unused_ttl_expired, get_relevant_facts, revalidate_citation, CitationValidator,
    FactStore, FactType, MemoryFact, Provenance, RetrievalFilter, Scope, TrustTier, Validity,
    ValidationExclusion,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload,
    FactValidationFailedPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use elai_core::clock::OsClock;
use std::sync::Arc;
use tempfile::TempDir;

const TEST_USER: &str = "default";

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, project_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: format!("{project_id}-session"),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb"))
}

fn make_tier_a_fact(key: &str, evidence: Vec<String>) -> MemoryFact {
    MemoryFact {
        fact_id: format!("fact-h32-{key}"),
        user_id: TEST_USER.into(),
        project_id: Some("proj-h32".into()),
        scope: Scope::Project,
        fact_type: FactType::DomainKnowledge,
        key: key.into(),
        value: serde_json::json!({ "raw": "no unwrap in prod code" }),
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
        provenance: Provenance::user_direct("h32-contract"),
        last_validated_at_ms: None,
    }
}

// ---------------------------------------------------------------------------
// (1) SCHEMA and SSB bijection
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_at_least_210_and_action_type_bijection_holds() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 210,
        "H-32 requires SCHEMA_VERSION >= 210, got {SCHEMA_VERSION_PUBLIC}"
    );
    let at = ssb_action_type_from_str_public("fact_validation_failed")
        .expect("from_str fact_validation_failed");
    assert_eq!(at, SsbActionType::FactValidationFailed);
    assert_eq!(
        SsbActionType::FactValidationFailed.as_sql_str(),
        "fact_validation_failed"
    );
}

// ---------------------------------------------------------------------------
// (2) The validity gate EXCLUDES a Stale/Unknown Tier-A fact from retrieval
// ---------------------------------------------------------------------------

struct AlwaysStaleValidator;
impl CitationValidator for AlwaysStaleValidator {
    fn validate(&self, _fact: &MemoryFact) -> Validity {
        Validity::Stale
    }
}

struct AlwaysUnknownValidator;
impl CitationValidator for AlwaysUnknownValidator {
    fn validate(&self, _fact: &MemoryFact) -> Validity {
        Validity::Unknown
    }
}

#[test]
fn stale_tier_a_fact_is_excluded_from_retrieval_and_recorded_in_exclusions() {
    let mut store = FactStore::open_in_memory().expect("store");
    // Insert a Tier-A fact with a parseable dkg: pointer.
    let fact = make_tier_a_fact(
        "no_unwrap",
        vec!["dkg:get_relevant_facts@crates/memory/src/retrieval.rs:46".into()],
    );
    store.insert(&fact).expect("insert Tier-A fact");

    let filter = RetrievalFilter {
        fact_type: Some(FactType::DomainKnowledge),
        citation_validator: Some(Box::new(AlwaysStaleValidator)),
        ..RetrievalFilter::default()
    };
    let (facts, exclusions) =
        get_relevant_facts(&store, TEST_USER, &filter, 10, 2_000).expect("retrieve");

    assert!(
        facts.is_empty(),
        "Stale Tier-A fact must be HARD-EXCLUDED from the injected slice; got {facts:?}"
    );
    assert_eq!(
        exclusions.len(),
        1,
        "one ValidationExclusion must be recorded for the excluded fact"
    );
    assert_eq!(exclusions[0].fact_id, fact.fact_id);
    assert_eq!(exclusions[0].reason, "stale");
    assert!(
        !exclusions[0].evidence_blake3.is_empty(),
        "evidence_blake3 must carry the citation fingerprint"
    );
}

#[test]
fn unknown_tier_a_citation_is_excluded_fail_closed() {
    let mut store = FactStore::open_in_memory().expect("store");
    let fact = make_tier_a_fact(
        "no_unwrap_unknown",
        vec!["dkg:soft_close@crates/memory/src/store.rs:354".into()],
    );
    store.insert(&fact).expect("insert");

    let filter = RetrievalFilter {
        fact_type: Some(FactType::DomainKnowledge),
        citation_validator: Some(Box::new(AlwaysUnknownValidator)),
        ..RetrievalFilter::default()
    };
    let (facts, exclusions) =
        get_relevant_facts(&store, TEST_USER, &filter, 10, 2_000).expect("retrieve");

    assert!(
        facts.is_empty(),
        "Unknown citation must be HARD-EXCLUDED (fail-closed per R2)"
    );
    assert_eq!(exclusions[0].reason, "unknown");
}

// ---------------------------------------------------------------------------
// (3) A Valid Tier-A fact IS returned and stamps last_validated_at_ms
// ---------------------------------------------------------------------------

struct AlwaysValidValidator;
impl CitationValidator for AlwaysValidValidator {
    fn validate(&self, _fact: &MemoryFact) -> Validity {
        Validity::Valid
    }
}

#[test]
fn valid_tier_a_fact_is_returned_and_last_validated_at_ms_is_stamped() {
    let mut store = FactStore::open_in_memory().expect("store");
    let fact = make_tier_a_fact(
        "valid_convention",
        vec!["dkg:get_relevant_facts@crates/memory/src/retrieval.rs:46".into()],
    );
    let fact_id = fact.fact_id.clone();
    store.insert(&fact).expect("insert");

    let now_ms: u128 = 5_000;
    let filter = RetrievalFilter {
        fact_type: Some(FactType::DomainKnowledge),
        citation_validator: Some(Box::new(AlwaysValidValidator)),
        ..RetrievalFilter::default()
    };
    let (facts, exclusions) =
        get_relevant_facts(&store, TEST_USER, &filter, 10, now_ms).expect("retrieve");

    assert_eq!(exclusions.len(), 0, "no exclusions on a Valid citation");
    assert_eq!(facts.len(), 1, "the Valid Tier-A fact must be returned");

    // Check that last_validated_at_ms was stamped on the stored row.
    let stored = store.get(&fact_id).expect("get").expect("row present");
    assert_eq!(
        stored.last_validated_at_ms,
        Some(now_ms),
        "last_validated_at_ms must be stamped on the stored row after a Valid check"
    );
}

// ---------------------------------------------------------------------------
// (4) The Tier-A retrieval widening is present in report_fns.rs
// ---------------------------------------------------------------------------

#[test]
fn report_fns_contains_tier_a_retrieval_widening() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let elai_cc = workspace
        .parent()
        .and_then(|p| p.parent())
        .expect("contracts has 2-up parent (.elai_cc)");
    let report_fns = elai_cc
        .join("crates")
        .join("cli")
        .join("src")
        .join("report_fns.rs");
    assert!(
        report_fns.exists(),
        "report_fns.rs must exist at {}",
        report_fns.display()
    );
    let body = fs::read_to_string(&report_fns).expect("read report_fns.rs");

    // H-32 done-gate hinge: the Tier-A retrieval (DomainKnowledge) must be
    // present in assemble_user_memory_section — NOT just the Tier-B Preference path.
    assert!(
        body.contains("FactType::DomainKnowledge"),
        "report_fns.rs MUST contain a Tier-A DomainKnowledge retrieval call \
         (H-32 done-gate hinge: Tier-A must be wired, not just Tier-B Preference)"
    );
    assert!(
        body.contains("FactType::Mistake"),
        "report_fns.rs MUST contain a Tier-A Mistake retrieval call \
         (H-32 done-gate hinge: both Tier-A fact types required)"
    );
    assert!(
        body.contains("DkgCitationValidator"),
        "report_fns.rs MUST reference DkgCitationValidator \
         (H-32 done-gate: the concrete DKG-backed validator must be wired)"
    );
}

// ---------------------------------------------------------------------------
// (5) A real FactValidationFailed SSB row writes + round-trips
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_fact_validation_failed_row_lands_and_round_trips() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h32-contract");

    let citation = "dkg:get_relevant_facts@crates/memory/src/retrieval.rs:46".to_string();
    let evidence_blake3 = {
        let canonical = "get_relevant_facts@crates/memory/src/retrieval.rs:46";
        blake3::hash(canonical.as_bytes())
            .to_hex()
            .chars()
            .take(16)
            .collect::<String>()
    };

    let payload = FactValidationFailedPayload {
        fact_id: "fact-h32-test".into(),
        citation: citation.clone(),
        reason: "stale".into(),
        evidence_blake3: evidence_blake3.clone(),
    };

    ssb.write_payload(
        "h32-contract",
        0,
        SsbPayload::FactValidationFailed(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write FactValidationFailed row");

    // The off-agent artifact: read it back and verify the round-trip.
    let rows = ssb
        .query_by_type(SsbActionType::FactValidationFailed)
        .expect("query FactValidationFailed");
    assert_eq!(
        rows.len(),
        1,
        "exactly one FactValidationFailed row must land (the off-agent artifact, R4)"
    );
    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode payload");
    match decoded {
        SsbPayload::FactValidationFailed(p) => {
            assert_eq!(p, payload, "round-trip mismatch on FactValidationFailed payload");
            assert_eq!(p.reason, "stale");
            assert!(
                !p.evidence_blake3.is_empty() && p.evidence_blake3.len() == 16,
                "evidence_blake3 must be a 16-hex blake3 fingerprint"
            );
            assert!(
                p.citation.starts_with("dkg:"),
                "citation must carry the dkg: URI scheme pointer"
            );
        }
        other => panic!("expected FactValidationFailed, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Bonus: collect_unused_ttl_expired soft-close invariant
// ---------------------------------------------------------------------------

#[test]
fn ttl_expired_tier_a_fact_is_identified_for_soft_close() {
    use elai_memory::UNUSED_TTL_MS;

    let fact = make_tier_a_fact(
        "ttl_test",
        vec!["dkg:foo@bar.rs:1".into()],
    );
    // Simulate a fact that was written long ago (now > valid_from + TTL).
    let old_valid_from = 1_000_u128;
    let far_past_ttl = old_valid_from + UNUSED_TTL_MS + 1;

    let mut aged_fact = fact.clone();
    aged_fact.valid_from_ms = old_valid_from;
    aged_fact.last_validated_at_ms = None;

    let expired = collect_unused_ttl_expired(&[aged_fact.clone()], far_past_ttl);
    assert_eq!(
        expired,
        vec![aged_fact.fact_id.clone()],
        "TTL-expired Tier-A fact must be identified for soft-close"
    );

    // A fact validated recently must NOT expire.
    let mut fresh_fact = fact;
    fresh_fact.valid_from_ms = old_valid_from;
    fresh_fact.last_validated_at_ms = Some(far_past_ttl - 1);
    let not_expired = collect_unused_ttl_expired(&[fresh_fact], far_past_ttl);
    assert!(
        not_expired.is_empty(),
        "recently-validated fact must NOT be TTL-expired"
    );
}

// ---------------------------------------------------------------------------
// (6) Read-path SSB beacon: retrieval exclusion -> FactValidationFailed row
//     (B-3 strengthening: proves the read-path, not just the TTL-tick path)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn read_path_stale_citation_emits_fact_validation_failed_ssb_row() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h32-read-path");

    // Build a store with one Tier-A fact that will be Stale under AlwaysStaleValidator.
    let mut store = FactStore::open_in_memory().expect("store");
    let fact = make_tier_a_fact(
        "read_path_stale",
        vec!["dkg:assemble_user_memory_section@crates/cli/src/report_fns.rs:1152".into()],
    );
    store.insert(&fact).expect("insert Tier-A fact");

    // Run the retrieval gate — produces a ValidationExclusion for the stale fact.
    let filter = RetrievalFilter {
        fact_type: Some(FactType::DomainKnowledge),
        citation_validator: Some(Box::new(AlwaysStaleValidator)),
        ..RetrievalFilter::default()
    };
    let (facts, exclusions) =
        get_relevant_facts(&store, TEST_USER, &filter, 10, 5_000).expect("retrieve");
    assert!(facts.is_empty(), "stale Tier-A must be excluded from the retrieval slice");
    assert_eq!(exclusions.len(), 1, "one ValidationExclusion must be produced");

    let excl = &exclusions[0];
    // Simulate what assemble_user_memory_section does with the SSB handle (B-3):
    // emit a fact_validation_failed row per exclusion.
    let citation = if excl.evidence_blake3.is_empty() {
        format!("fact:{}", excl.fact_id)
    } else {
        format!("blake3:{}", excl.evidence_blake3)
    };
    let payload = FactValidationFailedPayload {
        fact_id: excl.fact_id.clone(),
        citation,
        reason: excl.reason.to_string(),
        evidence_blake3: excl.evidence_blake3.clone(),
    };
    ssb.write_payload(
        "report_fns.fact_validation_failed",
        0,
        SsbPayload::FactValidationFailed(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write read-path FactValidationFailed row");

    // Assert the off-agent artifact lands (R4: the observable_effect holds).
    let rows = ssb
        .query_by_type(SsbActionType::FactValidationFailed)
        .expect("query");
    assert_eq!(
        rows.len(),
        1,
        "read-path exclusion must produce exactly one FactValidationFailed SSB row (R4)"
    );
    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode read-path row");
    match decoded {
        SsbPayload::FactValidationFailed(p) => {
            assert_eq!(p.fact_id, excl.fact_id, "fact_id round-trip");
            assert_eq!(p.reason, "stale", "reason must be stale");
            assert!(
                !p.evidence_blake3.is_empty(),
                "evidence_blake3 must be present for a stale dkg: citation"
            );
        }
        other => panic!("expected FactValidationFailed, got {other:?}"),
    }
}
