//! F-2a SecurityRuleBlocked SSB round-trip contract tests.
//!
//! Validates:
//!   1. SecurityRuleBlockedPayload encodes/decodes through msgpack with the
//!      correct sql_str ("security_rule_blocked").
//!   2. ssb_action_type_from_str_public("security_rule_blocked") returns the
//!      correct flat-beacon variant.
//!   3. The SSB query_by_type API recovers the row from the DB.
//!   4. emit_vibegate_blocked_to_ssb emits a SecurityRuleBlocked beacon when
//!      ELAI_SECURITY_RULE_BLOCKED_ENABLE=1 and the finding is a CodeVuln block.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::hook_dispatcher::{HookFinding, HookTier, HookVerdictKind};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, InsecureRateMeasuredPayload, Ssb, SsbActionType, SsbConfig,
    SsbPayload, SsbReader, SecurityRuleBlockedPayload, ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV,
    ELAI_VIBE_GATE_ENABLE_ENV,
};
use elai_orchestrator::vibegate::{emit_vibegate_blocked_to_ssb, VIBESEC_CODE_VULN_CATEGORY};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn make_ssb(tmp: &TempDir, project: &str, session: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: project.into(),
        session_id: session.into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb new"))
}

// ---------------------------------------------------------------------------
// Contract 1: SecurityRuleBlockedPayload round-trip through msgpack
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_security_rule_blocked_payload_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "sec-test", "sec-sess-rt").await;

    let payload = SsbPayload::SecurityRuleBlocked(SecurityRuleBlockedPayload {
        rule_id: "SEC-SQLI-001".to_string(),
        cwe: "CWE-89".to_string(),
        file_path: "app/db.py".to_string(),
        language: "python".to_string(),
        candidate_blake3: "1234567890abcdef".to_string(),
    });

    assert_eq!(
        payload.action_type(),
        SsbActionType::SecurityRuleBlocked,
        "SecurityRuleBlocked payload action_type must be SecurityRuleBlocked"
    );
    assert_eq!(
        payload.action_type().as_sql_str(),
        "security_rule_blocked",
        "SecurityRuleBlocked sql_str must be security_rule_blocked"
    );

    ssb.write_payload("agent", 0, payload, Vec::new())
        .await
        .expect("write SecurityRuleBlocked payload");

    let rows = ssb
        .query_by_type(SsbActionType::SecurityRuleBlocked)
        .expect("query SecurityRuleBlocked rows");
    assert_eq!(rows.len(), 1, "exactly one SecurityRuleBlocked row in DB");

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode SecurityRuleBlocked");
    match decoded {
        SsbPayload::SecurityRuleBlocked(p) => {
            assert_eq!(p.rule_id, "SEC-SQLI-001");
            assert_eq!(p.cwe, "CWE-89");
            assert_eq!(p.file_path, "app/db.py");
            assert_eq!(p.language, "python");
            assert_eq!(p.candidate_blake3, "1234567890abcdef");
        }
        other => panic!("expected SecurityRuleBlocked, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 2: ssb_action_type_from_str_public recognises "security_rule_blocked"
// ---------------------------------------------------------------------------

#[test]
fn contract_security_rule_blocked_str_roundtrip() {
    let at = ssb_action_type_from_str_public("security_rule_blocked")
        .expect("security_rule_blocked must parse");
    assert_eq!(
        at,
        SsbActionType::SecurityRuleBlocked,
        "security_rule_blocked must map to SsbActionType::SecurityRuleBlocked"
    );
    assert_eq!(
        at.as_sql_str(),
        "security_rule_blocked",
        "as_sql_str round-trip must be stable"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: emit_vibegate_blocked_to_ssb writes SecurityRuleBlocked beacon
//             when ELAI_SECURITY_RULE_BLOCKED_ENABLE=1 + finding is CodeVuln
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_emit_vibegate_writes_security_rule_blocked_beacon() {
    // Save + set both env gates.
    let prev_vg = std::env::var(ELAI_VIBE_GATE_ENABLE_ENV).ok();
    let prev_sec = std::env::var(ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV).ok();
    std::env::set_var(ELAI_VIBE_GATE_ENABLE_ENV, "1");
    std::env::set_var(ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV, "1");

    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "sec-test", "sec-sess-emit").await;

    let findings = vec![HookFinding {
        tier: HookTier::T1,
        verdict: HookVerdictKind::Block,
        rule_id: Some("SEC-SQLI-001".to_string()),
        category: VIBESEC_CODE_VULN_CATEGORY.to_string(),
        message: "vibesec_code_vuln: rule 'SEC-SQLI-001' blocked in app/db.py".to_string(),
        fix_template: None,
    }];

    let emitted = emit_vibegate_blocked_to_ssb(
        &ssb,
        "agent",
        0,
        r#"cursor.execute(f"SELECT * FROM users WHERE id = {uid}")"#,
        "app/db.py",
        Some("executing"),
        &findings,
    )
    .await;

    assert_eq!(
        emitted, 1,
        "emit_vibegate_blocked_to_ssb must emit 1 VibeGateBlocked row for CodeVuln"
    );

    // The SecurityRuleBlocked beacon must also have been written.
    let sec_rows = ssb
        .query_by_type(SsbActionType::SecurityRuleBlocked)
        .expect("query SecurityRuleBlocked rows");
    assert_eq!(
        sec_rows.len(),
        1,
        "one SecurityRuleBlocked beacon must accompany the CodeVuln VibeGateBlocked row"
    );

    let decoded: SsbPayload =
        rmp_serde::from_slice(&sec_rows[0].payload).expect("decode beacon");
    match decoded {
        SsbPayload::SecurityRuleBlocked(p) => {
            assert_eq!(p.rule_id, "SEC-SQLI-001");
            assert_eq!(p.cwe, "CWE-89", "CWE must be derived from the SEC-SQLI-* rule_id");
            assert_eq!(p.language, "python", "language must be detected from .py extension");
        }
        other => panic!("expected SecurityRuleBlocked, got {other:?}"),
    }

    // Restore env.
    match prev_vg {
        Some(v) => std::env::set_var(ELAI_VIBE_GATE_ENABLE_ENV, v),
        None => std::env::remove_var(ELAI_VIBE_GATE_ENABLE_ENV),
    }
    match prev_sec {
        Some(v) => std::env::set_var(ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV, v),
        None => std::env::remove_var(ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV),
    }
}

// ---------------------------------------------------------------------------
// Contract 4: SecurityRuleBlocked is NOT emitted for a Secret finding
//             (beacon only fires on CodeVuln axis)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_security_rule_blocked_not_emitted_for_secret() {
    let prev_vg = std::env::var(ELAI_VIBE_GATE_ENABLE_ENV).ok();
    let prev_sec = std::env::var(ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV).ok();
    std::env::set_var(ELAI_VIBE_GATE_ENABLE_ENV, "1");
    std::env::set_var(ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV, "1");

    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "sec-test", "sec-sess-nosec").await;

    let findings = vec![HookFinding {
        tier: HookTier::T1,
        verdict: HookVerdictKind::Block,
        rule_id: Some("vibesec-secret-aws_access_key_id".to_string()),
        category: elai_orchestrator::vibegate::VIBESEC_SECRET_CATEGORY.to_string(),
        message: "vibesec_secret: secret pattern detected".to_string(),
        fix_template: None,
    }];

    emit_vibegate_blocked_to_ssb(
        &ssb,
        "agent",
        0,
        r#"let key = "AKIAIOSFODNN7EXAMPLEKEY1";"#,
        "src/config.rs",
        None,
        &findings,
    )
    .await;

    let sec_rows = ssb
        .query_by_type(SsbActionType::SecurityRuleBlocked)
        .expect("query SecurityRuleBlocked");
    assert_eq!(
        sec_rows.len(),
        0,
        "SecurityRuleBlocked beacon must NOT be emitted for a Secret axis finding"
    );

    match prev_vg {
        Some(v) => std::env::set_var(ELAI_VIBE_GATE_ENABLE_ENV, v),
        None => std::env::remove_var(ELAI_VIBE_GATE_ENABLE_ENV),
    }
    match prev_sec {
        Some(v) => std::env::set_var(ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV, v),
        None => std::env::remove_var(ELAI_SECURITY_RULE_BLOCKED_ENABLE_ENV),
    }
}

// ---------------------------------------------------------------------------
// F-2b / Slice A4: InsecureRateMeasured round-trip
//
// The keystone bench beacon (SCHEMA_VERSION 201, Slice A1) carries the
// static-gate insecure-rate. These tests pin that the payload survives the
// EXACT on-wire encoder `write_payload` uses (rmp_serde::to_vec_named) with all
// f64 fields plus suite / model_id / corpus_blake3 intact.
// ---------------------------------------------------------------------------

/// A representative beacon payload. The numeric values are arbitrary
/// non-trivial doubles chosen to expose any field-truncation / reordering bug
/// (e.g. before/after swap) -- they are NOT a measurement target and have no
/// relation to the corpus.
fn sample_insecure_rate_payload() -> InsecureRateMeasuredPayload {
    InsecureRateMeasuredPayload {
        suite: "seccodeplt_static_gate".to_string(),
        model_id: "seccodeplt_curated_corpus".to_string(),
        n_samples: 80,
        insecure_rate_before: 0.503_125,
        insecure_rate_after: 0.187_5,
        gate_recall: 0.627_329_192_546_583_8,
        seed: 7,
        corpus_blake3: "af1349b9f5f9a1a6a0404dea36dcc949\
                        9bcb25c9adc112b7cc9a93cae41f3262"
            .to_string(),
    }
}

/// Direct encoder round-trip through the SAME serializer `write_payload` uses
/// (`rmp_serde::to_vec_named`), then `from_slice` back. Asserts every f64 field
/// plus suite / model_id / corpus_blake3 / n_samples / seed survive bit-for-bit.
#[test]
fn contract_insecure_rate_measured_wire_encoder_roundtrip() {
    let original = sample_insecure_rate_payload();
    let wrapped = SsbPayload::InsecureRateMeasured(original.clone());

    assert_eq!(
        wrapped.action_type(),
        SsbActionType::InsecureRateMeasured,
        "InsecureRateMeasured payload action_type must be InsecureRateMeasured"
    );
    assert_eq!(
        wrapped.action_type().as_sql_str(),
        "insecure_rate_measured",
        "InsecureRateMeasured sql_str must be insecure_rate_measured"
    );

    // The on-wire encoder: this is the exact call inside Ssb::write_payload.
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode InsecureRateMeasured on-wire");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("decode InsecureRateMeasured from wire bytes");

    match decoded {
        SsbPayload::InsecureRateMeasured(p) => {
            // f64 fields: exact bitwise equality after round-trip (msgpack
            // preserves IEEE-754 doubles exactly).
            assert_eq!(
                p.insecure_rate_before.to_bits(),
                original.insecure_rate_before.to_bits(),
                "insecure_rate_before must survive the wire encoder exactly"
            );
            assert_eq!(
                p.insecure_rate_after.to_bits(),
                original.insecure_rate_after.to_bits(),
                "insecure_rate_after must survive the wire encoder exactly"
            );
            assert_eq!(
                p.gate_recall.to_bits(),
                original.gate_recall.to_bits(),
                "gate_recall must survive the wire encoder exactly"
            );
            assert_eq!(p.suite, original.suite, "suite must round-trip");
            assert_eq!(p.model_id, original.model_id, "model_id must round-trip");
            assert_eq!(
                p.corpus_blake3, original.corpus_blake3,
                "corpus_blake3 must round-trip"
            );
            assert_eq!(p.n_samples, original.n_samples, "n_samples must round-trip");
            assert_eq!(p.seed, original.seed, "seed must round-trip");
        }
        other => panic!("expected InsecureRateMeasured, got {other:?}"),
    }
}

/// Full DB path: write through `Ssb::write_payload` (which internally calls
/// `rmp_serde::to_vec_named`) and recover via `query_by_type`, decoding the
/// stored bytes the same way the dashboard/CLI readers do.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_insecure_rate_measured_db_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "sec-test", "f2b-irm-rt").await;

    let original = sample_insecure_rate_payload();
    ssb.write_payload(
        "bench",
        0,
        SsbPayload::InsecureRateMeasured(original.clone()),
        Vec::new(),
    )
    .await
    .expect("write InsecureRateMeasured payload");

    let rows = ssb
        .query_by_type(SsbActionType::InsecureRateMeasured)
        .expect("query InsecureRateMeasured rows");
    assert_eq!(rows.len(), 1, "exactly one InsecureRateMeasured row in DB");

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode InsecureRateMeasured from DB");
    match decoded {
        SsbPayload::InsecureRateMeasured(p) => {
            assert_eq!(
                p.insecure_rate_before.to_bits(),
                original.insecure_rate_before.to_bits()
            );
            assert_eq!(
                p.insecure_rate_after.to_bits(),
                original.insecure_rate_after.to_bits()
            );
            assert_eq!(p.gate_recall.to_bits(), original.gate_recall.to_bits());
            assert_eq!(p.suite, original.suite);
            assert_eq!(p.model_id, original.model_id);
            assert_eq!(p.corpus_blake3, original.corpus_blake3);
            assert_eq!(p.n_samples, original.n_samples);
            assert_eq!(p.seed, original.seed);
        }
        other => panic!("expected InsecureRateMeasured, got {other:?}"),
    }
}

/// `insecure_rate_measured` must parse via the public string mapper and
/// round-trip back to the same sql_str (mirrors the F-2a string contract).
#[test]
fn contract_insecure_rate_measured_str_roundtrip() {
    let at = ssb_action_type_from_str_public("insecure_rate_measured")
        .expect("insecure_rate_measured must parse");
    assert_eq!(
        at,
        SsbActionType::InsecureRateMeasured,
        "insecure_rate_measured must map to SsbActionType::InsecureRateMeasured"
    );
    assert_eq!(
        at.as_sql_str(),
        "insecure_rate_measured",
        "as_sql_str round-trip must be stable"
    );
}
