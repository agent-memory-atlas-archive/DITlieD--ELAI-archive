//! F-1 VibeGate SSB round-trip contract tests.
//!
//! Validates:
//!   1. VibeGateBlockedPayload for CodeVuln arm encodes/decodes through msgpack
//!      and produces the correct sql_str ("vibesec_code_vuln_blocked").
//!   2. VibeGateBlockedPayload for Secret arm round-trips and produces
//!      "vibesec_secret_blocked".
//!   3. Reserved arms DepHallucinated and Skill parse via ssb_action_type_from_str_public
//!      (sql arms exist) but no producer fires them: the emit function emits zero
//!      rows for findings whose category is not vibesec_secret or vibesec_code_vuln.
//!   4. VibeGateBlockedKind::as_sql_str() matches the sql_str on SsbActionType.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
    VibeGateBlockedKind, VibeGateBlockedPayload, ELAI_VIBE_GATE_ENABLE_ENV,
};
use elai_orchestrator::vibegate::emit_vibegate_blocked_to_ssb;
use elai_orchestrator::hook_dispatcher::{HookFinding, HookTier, HookVerdictKind};
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
// Contract 1: CodeVuln arm round-trip + sql_str
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_vibegate_codevuln_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "vg-test", "vg-sess-cv").await;

    let payload = SsbPayload::VibeGateBlocked(VibeGateBlockedPayload {
        kind: VibeGateBlockedKind::CodeVuln,
        rule_id: Some("SEC-SQLI-001".to_string()),
        cwe: Some("CWE-89".to_string()),
        file_path: "app/db.py".to_string(),
        source_blake3: "abcdef1234567890".to_string(),
        citation: None,
        fsm_state: Some("executing".to_string()),
    });

    // Action type must reflect the CodeVuln arm's sql_str.
    assert_eq!(
        payload.action_type(),
        SsbActionType::VibeGateBlocked(VibeGateBlockedKind::CodeVuln)
    );
    assert_eq!(
        payload.action_type().as_sql_str(),
        "vibesec_code_vuln_blocked",
        "CodeVuln arm sql_str must be vibesec_code_vuln_blocked"
    );

    ssb.write_payload("agent", 0, payload, Vec::new())
        .await
        .expect("write CodeVuln payload");

    let rows = ssb
        .query_by_type(SsbActionType::VibeGateBlocked(
            VibeGateBlockedKind::CodeVuln,
        ))
        .expect("query CodeVuln rows");
    assert_eq!(rows.len(), 1, "exactly one CodeVuln row");

    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode CodeVuln");
    match decoded {
        SsbPayload::VibeGateBlocked(p) => {
            assert_eq!(p.kind, VibeGateBlockedKind::CodeVuln);
            assert_eq!(p.rule_id.as_deref(), Some("SEC-SQLI-001"));
            assert_eq!(p.cwe.as_deref(), Some("CWE-89"));
            assert_eq!(p.file_path, "app/db.py");
            assert_eq!(p.source_blake3, "abcdef1234567890");
            assert_eq!(p.fsm_state.as_deref(), Some("executing"));
        }
        other => panic!("expected VibeGateBlocked(CodeVuln), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 2: Secret arm round-trip + sql_str
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_vibegate_secret_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "vg-test", "vg-sess-sec").await;

    let payload = SsbPayload::VibeGateBlocked(VibeGateBlockedPayload {
        kind: VibeGateBlockedKind::Secret,
        rule_id: Some("vibesec-secret-aws_access_key_id".to_string()),
        cwe: None,
        file_path: "src/config.rs".to_string(),
        source_blake3: "fedcba9876543210".to_string(),
        citation: None,
        fsm_state: None,
    });

    assert_eq!(
        payload.action_type(),
        SsbActionType::VibeGateBlocked(VibeGateBlockedKind::Secret)
    );
    assert_eq!(
        payload.action_type().as_sql_str(),
        "vibesec_secret_blocked",
        "Secret arm sql_str must be vibesec_secret_blocked"
    );

    ssb.write_payload("agent", 1, payload, Vec::new())
        .await
        .expect("write Secret payload");

    let rows = ssb
        .query_by_type(SsbActionType::VibeGateBlocked(VibeGateBlockedKind::Secret))
        .expect("query Secret rows");
    assert_eq!(rows.len(), 1, "exactly one Secret row");

    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode Secret");
    match decoded {
        SsbPayload::VibeGateBlocked(p) => {
            assert_eq!(p.kind, VibeGateBlockedKind::Secret);
            assert_eq!(
                p.rule_id.as_deref(),
                Some("vibesec-secret-aws_access_key_id")
            );
            assert!(p.cwe.is_none(), "Secret arm must not carry a CWE");
            assert_eq!(p.file_path, "src/config.rs");
            assert!(p.fsm_state.is_none());
        }
        other => panic!("expected VibeGateBlocked(Secret), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 3: reserved arms parse but emit zero rows
// ---------------------------------------------------------------------------

#[test]
fn contract_reserved_arms_parse_via_str() {
    let dep_at = ssb_action_type_from_str_public("vibesec_dep_hallucinated_blocked")
        .expect("DepHallucinated arm must parse");
    assert_eq!(
        dep_at,
        SsbActionType::VibeGateBlocked(VibeGateBlockedKind::DepHallucinated),
        "DepHallucinated must round-trip through ssb_action_type_from_str_public"
    );
    assert_eq!(
        dep_at.as_sql_str(),
        "vibesec_dep_hallucinated_blocked",
        "DepHallucinated sql_str must be vibesec_dep_hallucinated_blocked"
    );

    let skill_at = ssb_action_type_from_str_public("vibesec_skill_blocked")
        .expect("Skill arm must parse");
    assert_eq!(
        skill_at,
        SsbActionType::VibeGateBlocked(VibeGateBlockedKind::Skill),
        "Skill must round-trip through ssb_action_type_from_str_public"
    );
    assert_eq!(
        skill_at.as_sql_str(),
        "vibesec_skill_blocked",
        "Skill sql_str must be vibesec_skill_blocked"
    );
}

/// No producer fires DepHallucinated or Skill — emit function must skip
/// findings with categories that don't match the two active axes.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_reserved_arms_produce_zero_ssb_rows() {
    let prev = std::env::var(ELAI_VIBE_GATE_ENABLE_ENV).ok();
    std::env::set_var(ELAI_VIBE_GATE_ENABLE_ENV, "1");

    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "vg-test", "vg-sess-reserved").await;

    // A finding with an unknown category (not vibesec_secret or vibesec_code_vuln)
    // must not be emitted by emit_vibegate_blocked_to_ssb.
    let unknown_findings = vec![HookFinding {
        tier: HookTier::T1,
        verdict: HookVerdictKind::Block,
        rule_id: Some("vibesec-dep-hallucinated".to_string()),
        category: "vibesec_dep_hallucinated".to_string(), // not a producer category
        message: "dep hallucinated block".to_string(),
        fix_template: None,
    }];

    let emitted = emit_vibegate_blocked_to_ssb(
        &ssb,
        "agent",
        0,
        "import dep_that_doesnt_exist",
        "src/main.rs",
        None,
        &unknown_findings,
    )
    .await;

    assert_eq!(
        emitted, 0,
        "emit_vibegate_blocked_to_ssb must skip findings with unknown categories"
    );

    // Restore env.
    match prev {
        Some(v) => std::env::set_var(ELAI_VIBE_GATE_ENABLE_ENV, v),
        None => std::env::remove_var(ELAI_VIBE_GATE_ENABLE_ENV),
    }
}

// ---------------------------------------------------------------------------
// Contract 4: VibeGateBlockedKind::as_sql_str matches SsbActionType delegation
// ---------------------------------------------------------------------------

#[test]
fn contract_kind_sql_str_matches_action_type_delegation() {
    let pairs = [
        (
            VibeGateBlockedKind::CodeVuln,
            "vibesec_code_vuln_blocked",
        ),
        (VibeGateBlockedKind::Secret, "vibesec_secret_blocked"),
        (
            VibeGateBlockedKind::DepHallucinated,
            "vibesec_dep_hallucinated_blocked",
        ),
        (VibeGateBlockedKind::Skill, "vibesec_skill_blocked"),
    ];

    for (kind, expected_sql) in pairs {
        assert_eq!(
            kind.as_sql_str(),
            expected_sql,
            "VibeGateBlockedKind::{kind:?} as_sql_str mismatch"
        );
        let at = SsbActionType::VibeGateBlocked(kind);
        assert_eq!(
            at.as_sql_str(),
            expected_sql,
            "SsbActionType::VibeGateBlocked({kind:?}) as_sql_str mismatch"
        );
    }
}
