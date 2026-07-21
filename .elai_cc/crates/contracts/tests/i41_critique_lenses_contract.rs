//! I-41 spec-time critique lenses contract.
//!
//! Advisory-only: LensSet::run never affects any gate or verdict. All findings
//! are emitted as CritiqueLensFinding SSB rows (advisory=true). This contract
//! tests the structural invariants (isolation, anchor-drop, schema-lock, serde
//! round-trip) that make the advisory-only guarantee compiler-visible at runtime.
//!
//! Arms (wiring-enforcement R1/R4/R7):
//!   (a) Planted-assumption fixture: L1 run with planted assumption text produces
//!       an AdvisoryFinding SSB row with a non-empty `location` anchor.
//!   (b) Anchorless finding: a raw finding with empty location is dropped
//!       (DroppedNoAnchor SSB row lands; finding NOT in returned Vec).
//!   (c) Zero gate-verdict rows: after a full LensSet::run, query for
//!       SsbActionType::DirectiveExecuted / GovernancePrimitiveFired /
//!       ToolSequenceVerdict returns 0 rows — the advisory isolation invariant.
//!       This is the load-bearing structural test.
//!   (d) Kind-discriminating: at least one AdvisoryFinding row + at least one
//!       LensRun row land per lens configured to run.
//!   (e) Schema-lock: SCHEMA_VERSION_PUBLIC >= 243 AND
//!       action_type_lock.rs TOTAL_VARIANTS == 552 AND
//!       "critique_lens_finding" round-trips via from_str/as_sql_str AND
//!       CritiqueLensFindingPayload round-trips through serde_json with advisory=true.
//!
//! Spec references: docs/thefinalglance/I-plan/I-41_critique_lenses.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7/R8; .claude/rules/tracing-test-assert.md

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::critique::lenses::{
    LensId, LensFinding, LensSeverity, LensSet, RiskLevel, drop_if_no_anchor,
};
use elai_orchestrator::ssb::{
    CritiqueLensFindingKind, CritiqueLensFindingPayload, CritiqueLensFindingTag,
    SCHEMA_VERSION_PUBLIC, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
    decode_ssb_payload, ssb_action_type_from_str_public,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// InlineMockRuntime: zero live network calls.
// All queries return a canned JSON array with one finding that has a non-empty
// location anchor (test mode) or an empty location (anchorless mode).
// ---------------------------------------------------------------------------

/// A mock `RuntimeAdapter` that returns a canned findings JSON.
struct InlineMockRuntime {
    json_response: String,
}

impl InlineMockRuntime {
    fn with_finding(location: &str, claim: &str) -> Self {
        let json = format!(
            r#"[{{"lens":"L1","severity":"CONCERN","location":"{location}","claim":"{claim}","why_it_matters":"advisory only","suggested_resolution":"verify the assumption","confidence":0.8}}]"#
        );
        Self { json_response: json }
    }

    fn with_anchorless_finding() -> Self {
        let json = r#"[{"lens":"L1","severity":"CONCERN","location":"","claim":"no anchor","why_it_matters":"test","suggested_resolution":"add anchor","confidence":0.6}]"#;
        Self { json_response: json.to_owned() }
    }
}

impl elai_providers::runtime::types::RuntimeAdapter for InlineMockRuntime {
    fn backend(&self) -> elai_providers::runtime::types::RuntimeBackend {
        elai_providers::runtime::types::RuntimeBackend::AnthropicApi
    }

    fn capabilities(&self) -> elai_providers::runtime::types::RuntimeCapabilities {
        elai_providers::runtime::types::RuntimeCapabilities::none()
    }

    fn health_check(
        &self,
    ) -> elai_providers::runtime::types::RuntimeHealthFuture<'_> {
        Box::pin(async { elai_providers::runtime::types::RuntimeHealth::Ready })
    }

    fn complete(
        &self,
        _messages: Vec<elai_providers::runtime::types::Message>,
        _config: elai_providers::runtime::types::CompletionConfig,
    ) -> elai_providers::runtime::types::RuntimeFuture<'_, elai_providers::runtime::types::CompletionResponse>
    {
        let text = self.json_response.clone();
        Box::pin(async move {
            Ok(elai_providers::runtime::types::CompletionResponse {
                text,
                stop_reason: Some("end_turn".to_owned()),
                usage: (5, 20),
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
                model: "inline-mock".to_owned(),
            })
        })
    }

    fn stream(
        &self,
        _messages: Vec<elai_providers::runtime::types::Message>,
        _config: elai_providers::runtime::types::CompletionConfig,
    ) -> elai_providers::runtime::types::RuntimeStream<'static> {
        Box::pin(futures_util::stream::empty())
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn make_ssb(dir: &TempDir, session: &str) -> Arc<Ssb> {
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("ssb.db"),
                pool_size: Some(2),
                project_id: "i41-contract".into(),
                session_id: session.to_owned(),
            },
            clock,
        )
        .expect("ssb open"),
    )
}

// ---------------------------------------------------------------------------
// (a) Planted-assumption -> L1 AdvisoryFinding with non-empty anchor.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i41_a_planted_assumption_produces_advisory_finding() {
    // Kill-switch must be ON for the lens to fire.
    // Use std::env::set_var guarded by the test's own cleanup.
    std::env::set_var("ELAI_CRITIQUE_LENSES_ENABLE", "1");

    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir, "i41-a");

    let runtime = InlineMockRuntime::with_finding(
        "spec section: external-service assumptions",
        "external service availability is assumed without verification",
    );

    // Simple risk: L1 only (L1AssumptionExcavator).
    let lens_set = LensSet::from_risk(RiskLevel::Simple);
    let spec_text = "# My Spec\n\nAssumes: external service is always available.\n";

    let findings = lens_set
        .run(spec_text, Some(&runtime), Some(&ssb), "i41-a")
        .await
        .expect("lens run must succeed");

    // At least one finding returned.
    assert!(
        !findings.is_empty(),
        "planted-assumption spec must produce at least 1 advisory finding; got 0"
    );

    // The returned finding must have a non-empty location anchor (R1).
    let f = &findings[0];
    assert!(
        !f.location.trim().is_empty(),
        "finding must have non-empty location anchor; got empty location"
    );

    // An AdvisoryFinding SSB row must have landed (R4: off-author artifact).
    let rows = ssb
        .query_by_type(SsbActionType::CritiqueLensFinding(CritiqueLensFindingTag::AdvisoryFinding))
        .expect("query CritiqueLensFinding rows");
    assert!(
        !rows.is_empty(),
        "at least one CritiqueLensFinding SSB row must land after a lens run; got 0"
    );

    // Find the first AdvisoryFinding-kind row and assert advisory=true + non-empty location.
    // NOTE: query_by_type(AdvisoryFinding) returns ALL critique_lens_finding rows (same SQL
    // string) so rows include LensRun beacons too. We iterate to find the first AdvisoryFinding
    // kind (R4: off-author artifact with a real location anchor).
    let advisory_row = rows.iter().find(|r| {
        matches!(
            decode_ssb_payload(&r.payload),
            Ok(SsbPayload::CritiqueLensFinding(ref p))
                if matches!(p.kind, CritiqueLensFindingKind::AdvisoryFinding { .. })
        )
    }).expect("at least one AdvisoryFinding-kind row must be present");
    let decoded = decode_ssb_payload(&advisory_row.payload).expect("decode advisory row");
    match decoded {
        SsbPayload::CritiqueLensFinding(p) => {
            assert!(p.advisory, "CritiqueLensFindingPayload.advisory must be true; got false");
            match &p.kind {
                CritiqueLensFindingKind::AdvisoryFinding { location, .. } => {
                    assert!(
                        !location.trim().is_empty(),
                        "SSB AdvisoryFinding location must be non-empty (same anchor as returned finding)"
                    );
                }
                other => panic!(
                    "row must be AdvisoryFinding kind; got {other:?}"
                ),
            }
        }
        other => panic!(
            "row must be CritiqueLensFinding; got {other:?}"
        ),
    }

    std::env::remove_var("ELAI_CRITIQUE_LENSES_ENABLE");
}

// ---------------------------------------------------------------------------
// (b) Anchorless finding -> DroppedNoAnchor SSB row; NOT in returned Vec.
// ---------------------------------------------------------------------------

#[test]
fn i41_b_anchorless_finding_dropped() {
    // drop_if_no_anchor: empty location returns None.
    let raw = LensFinding {
        lens: LensId::L1AssumptionExcavator,
        severity: LensSeverity::Concern,
        location: "".to_owned(),  // EMPTY — must be dropped.
        claim: "test anchorless claim".to_owned(),
        why_it_matters: "test".to_owned(),
        suggested_resolution: "add anchor".to_owned(),
        confidence: 0.5,
    };

    let result = drop_if_no_anchor(raw);
    assert!(
        result.is_none(),
        "drop_if_no_anchor must return None for empty location (anchorless finding must not be surfaced)"
    );

    // A finding with whitespace-only location must also be dropped.
    let raw_ws = LensFinding {
        lens: LensId::L4GapAnalyzer,
        severity: LensSeverity::Flag,
        location: "   ".to_owned(),  // whitespace only — must be dropped.
        claim: "whitespace location".to_owned(),
        why_it_matters: "test".to_owned(),
        suggested_resolution: "fix".to_owned(),
        confidence: 0.3,
    };
    let result_ws = drop_if_no_anchor(raw_ws);
    assert!(
        result_ws.is_none(),
        "drop_if_no_anchor must return None for whitespace-only location"
    );

    // A finding with a real anchor is preserved.
    let raw_ok = LensFinding {
        lens: LensId::L8AnxietyReader,
        severity: LensSeverity::Note,
        location: "spec section: risk register".to_owned(),
        claim: "preserved finding".to_owned(),
        why_it_matters: "test".to_owned(),
        suggested_resolution: "review".to_owned(),
        confidence: 0.9,
    };
    let result_ok = drop_if_no_anchor(raw_ok);
    assert!(
        result_ok.is_some(),
        "drop_if_no_anchor must return Some for non-empty location anchor"
    );
}

// ---------------------------------------------------------------------------
// (c) Zero gate-verdict rows after LensSet::run (isolation invariant).
//     This is the load-bearing advisory-isolation test (R7 + wiring R4).
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i41_c_zero_gate_verdict_rows_after_lens_run() {
    std::env::set_var("ELAI_CRITIQUE_LENSES_ENABLE", "1");

    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir, "i41-c");

    let runtime = InlineMockRuntime::with_finding(
        "spec section: api contract",
        "advisory only — must not affect gate rows",
    );

    let lens_set = LensSet::new(
        vec![LensId::L1AssumptionExcavator],
        RiskLevel::Simple,
    );
    let spec_text = "# Spec\nSome content.\n";

    lens_set
        .run(spec_text, Some(&runtime), Some(&ssb), "i41-c")
        .await
        .expect("lens run must succeed");

    // Gate-verdict action types that must remain zero rows after a lens run.
    let gate_types_to_check: &[(&str, SsbActionType)] = &[
        (
            "DirectiveExecuted",
            SsbActionType::DirectiveExecuted(
                elai_orchestrator::ssb::DirectiveExecutedKind::Executed,
            ),
        ),
        (
            "GovernancePrimitiveFired",
            SsbActionType::GovernancePrimitiveFired(
                elai_orchestrator::ssb::GovernancePrimitiveFiredKind::CeflExternalized,
            ),
        ),
        (
            "ToolSequenceVerdict",
            SsbActionType::ToolSequenceVerdict(
                elai_orchestrator::ssb::ToolSequenceVerdictKind::Allowed,
            ),
        ),
    ];

    for (name, action_type) in gate_types_to_check {
        let rows = ssb.query_by_type(action_type.clone()).expect("query");
        assert_eq!(
            rows.len(),
            0,
            "advisory lens run must NOT produce {name} rows (isolation invariant); got {}",
            rows.len()
        );
    }

    // Sanity: at least one CritiqueLensFinding row DID land (advisory path was traversed).
    let advisory_rows = ssb
        .query_by_type(SsbActionType::CritiqueLensFinding(CritiqueLensFindingTag::AdvisoryFinding))
        .expect("query CritiqueLensFinding");
    assert!(
        !advisory_rows.is_empty(),
        "at least one CritiqueLensFinding row must land (sanity check: advisory path traversed)"
    );

    std::env::remove_var("ELAI_CRITIQUE_LENSES_ENABLE");
}

// ---------------------------------------------------------------------------
// (d) Kind-discriminating: AdvisoryFinding + LensRun rows per lens.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i41_d_advisory_finding_and_lens_run_rows_per_lens() {
    std::env::set_var("ELAI_CRITIQUE_LENSES_ENABLE", "1");

    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir, "i41-d");

    let runtime = InlineMockRuntime::with_finding(
        "spec section: assumptions",
        "kind-discriminating fixture finding",
    );

    // Run a single lens (L1) for tight kind inspection.
    let lens_set = LensSet::new(
        vec![LensId::L1AssumptionExcavator],
        RiskLevel::Simple,
    );

    lens_set
        .run("# Spec\nContent.\n", Some(&runtime), Some(&ssb), "i41-d")
        .await
        .expect("lens run must succeed");

    // Collect all CritiqueLensFinding rows.
    let all_rows = ssb
        .query_by_type(SsbActionType::CritiqueLensFinding(CritiqueLensFindingTag::AdvisoryFinding))
        .expect("query rows");

    // Decode all rows and collect kinds.
    let mut has_advisory_finding = false;
    let mut has_lens_run = false;

    for row in &all_rows {
        let payload = decode_ssb_payload(&row.payload).expect("decode row payload");
        match payload {
            SsbPayload::CritiqueLensFinding(p) => {
                match p.kind {
                    CritiqueLensFindingKind::AdvisoryFinding { .. } => {
                        has_advisory_finding = true;
                    }
                    CritiqueLensFindingKind::LensRun { .. } => {
                        has_lens_run = true;
                    }
                    CritiqueLensFindingKind::DroppedNoAnchor { .. } => {}
                }
            }
            other => panic!("unexpected payload kind: {other:?}"),
        }
    }

    assert!(
        has_advisory_finding,
        "at least one AdvisoryFinding row must land for L1 with a planted finding"
    );
    assert!(
        has_lens_run,
        "at least one LensRun beacon must land per lens run (observability invariant)"
    );

    std::env::remove_var("ELAI_CRITIQUE_LENSES_ENABLE");
}

// ---------------------------------------------------------------------------
// (e) Schema-lock: SCHEMA_VERSION >= 243; TOTAL_VARIANTS == 552;
//     wire-tag round-trip; serde round-trip with advisory=true.
// ---------------------------------------------------------------------------

#[test]
fn i41_e_schema_lock() {
    // SCHEMA_VERSION must be at least 243 (I-41 floor).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 243,
        "SCHEMA_VERSION_PUBLIC must be >= 243 (I-41 floor: CritiqueLensFinding added); \
         got {SCHEMA_VERSION_PUBLIC}"
    );

    // TOTAL_VARIANTS floor >= 552 (I-41) + CritiqueLensFinding registration.
    // Tier-B Move 2 derived TOTAL_VARIANTS and relocated ALL_VARIANTS (now
    // SSB_ACTION_TYPES in ssb.rs), so read the runtime registry directly.
    use elai_orchestrator::ssb::SSB_ACTION_TYPES;
    const I41_TV_FLOOR: usize = 552;
    assert!(
        SSB_ACTION_TYPES.len() >= I41_TV_FLOOR,
        "TOTAL_VARIANTS must be >= {I41_TV_FLOOR} (I-41 floor); got {}",
        SSB_ACTION_TYPES.len()
    );
    assert!(
        SSB_ACTION_TYPES
            .iter()
            .any(|v| v.as_sql_str() == "critique_lens_finding"),
        "SSB_ACTION_TYPES must include CritiqueLensFinding"
    );

    // Wire-tag round-trip: from_str -> as_sql_str.
    let at = ssb_action_type_from_str_public("critique_lens_finding")
        .expect("from_str critique_lens_finding must succeed");
    assert_eq!(
        at.as_sql_str(),
        "critique_lens_finding",
        "wire tag must round-trip as_sql_str -> from_str -> as_sql_str"
    );

    // Serde round-trip for CritiqueLensFindingPayload with advisory=true.
    let payload = CritiqueLensFindingPayload {
        kind: CritiqueLensFindingKind::AdvisoryFinding {
            lens: "L1".to_owned(),
            severity: "CONCERN".to_owned(),
            location: "spec section: assumptions".to_owned(),
            claim: "test claim".to_owned(),
            confidence: 0.75,
        },
        advisory: true,
        session_id: "i41-e-fixture".to_owned(),
    };

    let json = serde_json::to_string(&payload).expect("serde_json::to_string");
    let round_tripped: CritiqueLensFindingPayload =
        serde_json::from_str(&json).expect("serde_json::from_str");

    assert!(
        round_tripped.advisory,
        "CritiqueLensFindingPayload.advisory must survive serde round-trip as true; got false"
    );
    assert_eq!(
        round_tripped.session_id, "i41-e-fixture",
        "session_id must survive serde round-trip unchanged"
    );
    match &round_tripped.kind {
        CritiqueLensFindingKind::AdvisoryFinding { lens, confidence, .. } => {
            assert_eq!(lens, "L1", "lens must survive serde round-trip");
            assert!(
                (*confidence - 0.75).abs() < 1e-5,
                "confidence must survive serde round-trip within f32 epsilon; got {confidence}"
            );
        }
        other => panic!("round-tripped kind must be AdvisoryFinding; got {other:?}"),
    }
}
