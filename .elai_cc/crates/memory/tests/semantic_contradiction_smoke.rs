//! F-12 W-5 deterministic fixture harness — the REAL
//! `collect_cross_key_candidates -> adjudicate_semantic -> soft_close`
//! path exercised with a deterministic `FakeNliJudge`.
//!
//! Only the NLI judge is faked (exactly as the `NliJudge` trait seam
//! intends); the candidate-pairing + adjudication + non-destructive
//! soft-close code is the SAME production path the live tick runs. No
//! ONNX model is required to ship this — the trait seam is the whole
//! point of the Tier-1 / Tier-2 split.

use elai_memory::semantic_contradiction::{
    adjudicate_semantic, collect_cross_key_candidates, pair_fingerprint, NliJudge, NliLabel,
    NliVerdict, SemanticAction, SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD,
};
use elai_memory::{FactStore, FactType, MemoryError, MemoryFact, Provenance, Scope, TrustTier};

/// Deterministic judge: returns a fixed verdict regardless of input.
/// The fixture scripts the verdict so the test asserts on the
/// adjudication + soft-close behaviour, not on any model.
struct FakeNliJudge {
    label: NliLabel,
    confidence: f32,
}

impl NliJudge for FakeNliJudge {
    fn classify(&self, _premise: &str, _hypothesis: &str) -> Result<NliVerdict, MemoryError> {
        Ok(NliVerdict {
            label: self.label,
            confidence: self.confidence,
        })
    }
}

fn fact(id: &str, key: &str, value: serde_json::Value, t: u128) -> MemoryFact {
    MemoryFact {
        fact_id: id.into(),
        user_id: "default".into(),
        project_id: Some("p1".into()),
        scope: Scope::Project,
        fact_type: FactType::Preference,
        key: key.into(),
        value,
        confidence: 0.9,
        trust_score: 0.9,
        ingested_at_ms: t,
        valid_from_ms: t,
        valid_to_ms: None,
        decay_rate_per_day: 0.001,
        last_observed_at_ms: t,
        evidence_pointers: vec!["sess".into(), "msg".into()],
        superseded_by: None,
        trust_tier: TrustTier::T1CodebaseDerived,
        provenance: Provenance::user_direct("test"),
    }
}

/// The W-5 happy path: a cross-key contradiction above threshold soft-closes
/// the OLDER row with `valid_to_ms == newer.valid_from_ms` (the Graphiti
/// `invalid_at = valid_at` event-time semantic), and the newer row survives.
#[test]
fn above_threshold_contradiction_soft_closes_older_with_event_time() {
    let mut store = FactStore::open_in_memory().expect("open");
    let t0 = 1_700_000_000_000u128;
    // Two facts with DIFFERENT keys (audit_conflicts would NOT pair these —
    // that is the recall gap F-12 closes).
    store
        .insert(&fact(
            "older",
            "prefers_db",
            serde_json::json!({"subject": "postgres"}),
            t0,
        ))
        .expect("ins older");
    let newer_valid_from = t0 + 5_000;
    store
        .insert(&fact(
            "newer",
            "migrated_off",
            serde_json::json!({"subject": "postgres in favour of sqlite"}),
            newer_valid_from,
        ))
        .expect("ins newer");

    let candidates =
        collect_cross_key_candidates(&store, "default", FactType::Preference, Some(Scope::Project))
            .expect("collect");
    assert!(
        !candidates.is_empty(),
        "cross-key candidates must include the (older, newer) pair"
    );

    let judge = FakeNliJudge {
        label: NliLabel::Contradiction,
        confidence: 0.97,
    };
    let outcomes = adjudicate_semantic(
        &mut store,
        &judge,
        &candidates,
        SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD,
        t0 + 10_000,
    )
    .expect("adjudicate");

    let invalidated: Vec<_> = outcomes
        .iter()
        .filter(|o| matches!(o.action, SemanticAction::InvalidatedOlder))
        .collect();
    assert_eq!(
        invalidated.len(),
        1,
        "exactly one InvalidatedOlder outcome expected"
    );
    assert_eq!(invalidated[0].older_fact_id, "older");
    assert_eq!(invalidated[0].newer_fact_id, "newer");

    // The OLDER row is soft-closed with valid_to_ms == newer.valid_from_ms
    // (NOT now_ms). This is the bi-temporal correctness the verdict requires.
    let older_row = store.get("older").expect("get older").expect("older exists");
    assert_eq!(
        older_row.valid_to_ms,
        Some(newer_valid_from),
        "older row must be soft-closed at newer.valid_from_ms (Graphiti event-time)"
    );

    // The NEWER row survives (non-destructive).
    let newer_row = store.get("newer").expect("get newer").expect("newer exists");
    assert_eq!(
        newer_row.valid_to_ms, None,
        "newer row must remain currently-valid"
    );
}

/// A neutral verdict leaves BOTH rows valid (SurfaceToUser, the safe default).
#[test]
fn neutral_verdict_leaves_both_rows_valid() {
    let mut store = FactStore::open_in_memory().expect("open");
    let t0 = 1_700_000_000_000u128;
    store
        .insert(&fact(
            "older",
            "prefers_db",
            serde_json::json!({"subject": "postgres"}),
            t0,
        ))
        .expect("ins older");
    store
        .insert(&fact(
            "newer",
            "likes_editor",
            serde_json::json!({"subject": "neovim"}),
            t0 + 5_000,
        ))
        .expect("ins newer");

    let candidates =
        collect_cross_key_candidates(&store, "default", FactType::Preference, Some(Scope::Project))
            .expect("collect");
    let judge = FakeNliJudge {
        label: NliLabel::Neutral,
        confidence: 0.99,
    };
    let outcomes = adjudicate_semantic(
        &mut store,
        &judge,
        &candidates,
        SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD,
        t0 + 10_000,
    )
    .expect("adjudicate");

    assert!(
        outcomes
            .iter()
            .all(|o| matches!(o.action, SemanticAction::SurfaceToUser)),
        "neutral pairs must all be SurfaceToUser"
    );
    assert_eq!(
        store.get("older").expect("get").expect("exists").valid_to_ms,
        None,
        "older row stays valid on neutral"
    );
    assert_eq!(
        store.get("newer").expect("get").expect("exists").valid_to_ms,
        None,
        "newer row stays valid on neutral"
    );
}

/// The blake3 pair fingerprint is stable across runs and is a 16-char hex
/// prefix (mirrors the F-5 broadcast_id format `{:.16}`).
#[test]
fn pair_fingerprint_is_stable_blake3_prefix() {
    let t0 = 1_700_000_000_000u128;
    let older = fact("older", "prefers_db", serde_json::json!({"subject": "postgres"}), t0);
    let newer = fact(
        "newer",
        "migrated_off",
        serde_json::json!({"subject": "sqlite"}),
        t0 + 5_000,
    );

    let fp1 = pair_fingerprint(&older, &newer);
    let fp2 = pair_fingerprint(&older, &newer);
    assert_eq!(fp1, fp2, "fingerprint must be deterministic across calls");
    assert_eq!(fp1.len(), 16, "fingerprint must be a 16-char blake3 hex prefix");
    assert!(
        fp1.chars().all(|c| c.is_ascii_hexdigit()),
        "fingerprint must be all-hex: {fp1}"
    );
}
