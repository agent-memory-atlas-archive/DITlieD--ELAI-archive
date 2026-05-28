// E-22 (SCHEMA_VERSION 170) contract: may-2026 github trending sweep
// substrate SSB variant.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 170 (E-22 atomic bump for the single
//    GithubTrendingEvent variant with 3-arm TrendingEventKind
//    discriminator).
// 2. GithubTrendingEvent round-trips via as_sql_str /
//    ssb_action_type_from_str_public.
// 3. action_type() on SsbPayload::GithubTrendingEvent returns the matching
//    SsbActionType.
// 4. GithubTrendingEventPayload serializes via msgpack (rmp_serde) without
//    loss for all 3 kinds.
// 5. TrendingEventKind covers all 3 documented fire-points and the
//    snake_case wire labels match the dashboard tile mapping.
// 6. ELAI_GITHUB_TRENDING_ENABLE_ENV constant matches the documented
//    env var name (wire-dark gate).
//
// Grounding: Anthropic SKILL.md spec (code.claude.com/docs/en/skills),
// microsandbox README (github.com/superradcompany/microsandbox, YC X26),
// arXiv:2604.21816 "Tool Attention Is All You Need" (95% MCP-tax
// reduction; VERIFIED 2026-05-27 per E-22 grounding triage).

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, GithubTrendingEventPayload, SsbActionType, SsbPayload,
    TrendingEventKind, ELAI_GITHUB_TRENDING_ENABLE_ENV, SCHEMA_VERSION_PUBLIC,
};

const E22_SCHEMA_FLOOR: i64 = 170;

#[test]
fn schema_version_includes_e22_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= E22_SCHEMA_FLOOR,
        "schema must include E-22 SSB variant (got {SCHEMA_VERSION_PUBLIC}, floor {E22_SCHEMA_FLOOR})"
    );
}

#[test]
fn github_trending_event_action_type_roundtrip() {
    let at = SsbActionType::GithubTrendingEvent;
    let s = at.as_sql_str();
    assert_eq!(s, "github_trending_event", "as_sql_str mismatch");
    let decoded = ssb_action_type_from_str_public(s).expect("from_str failed");
    assert_eq!(decoded, at, "round-trip failed for {s}");
}

#[test]
fn trending_event_kind_wire_labels() {
    // 3 kinds per E-22 decisions; wire labels match dashboard mapping.
    let cases = [
        (
            TrendingEventKind::SkillExportNormalized,
            "skill_export_normalized",
        ),
        (TrendingEventKind::MicroVmFallback, "microvm_fallback"),
        (TrendingEventKind::LazySchemaSavings, "lazy_schema_savings"),
    ];
    for (kind, label) in cases {
        assert_eq!(kind.as_str(), label, "kind wire label mismatch");
    }
}

#[test]
fn github_trending_event_payload_bijection_skill_export() {
    let original = GithubTrendingEventPayload {
        kind: TrendingEventKind::SkillExportNormalized,
        session_id: Some("s-e22".into()),
        skill_name: "my-skill".into(),
        reason_label: String::new(),
        fallback_backend: String::new(),
        pre_bytes: 0,
        post_bytes: 0,
        tool_count: 0,
        drift_count: 0,
        details_json: serde_json::json!({
            "dist_path": "dist/claude/skills/my-skill/SKILL.md",
            "target": "claude",
            "caller_site": "contract_test"
        }),
        ts_ms: 1_700_000_170_001,
    };
    let payload = SsbPayload::GithubTrendingEvent(original.clone());
    assert_eq!(payload.action_type(), SsbActionType::GithubTrendingEvent);
    let encoded = rmp_serde::to_vec(&payload).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
    match decoded {
        SsbPayload::GithubTrendingEvent(p) => assert_eq!(p, original),
        other => panic!("unexpected variant after round-trip: {other:?}"),
    }
}

#[test]
fn github_trending_event_payload_bijection_microvm_fallback() {
    let original = GithubTrendingEventPayload {
        kind: TrendingEventKind::MicroVmFallback,
        session_id: None,
        skill_name: String::new(),
        reason_label: "missing_bin".into(),
        fallback_backend: "firecracker".into(),
        pre_bytes: 0,
        post_bytes: 0,
        tool_count: 0,
        drift_count: 0,
        details_json: serde_json::json!({
            "caller_site": "contract_test"
        }),
        ts_ms: 1_700_000_170_002,
    };
    let payload = SsbPayload::GithubTrendingEvent(original.clone());
    let encoded = rmp_serde::to_vec(&payload).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
    match decoded {
        SsbPayload::GithubTrendingEvent(p) => assert_eq!(p, original),
        other => panic!("unexpected variant after round-trip: {other:?}"),
    }
}

#[test]
fn github_trending_event_payload_bijection_lazy_schema_savings() {
    let original = GithubTrendingEventPayload {
        kind: TrendingEventKind::LazySchemaSavings,
        session_id: Some("s-e22-savings".into()),
        skill_name: String::new(),
        reason_label: String::new(),
        fallback_backend: String::new(),
        pre_bytes: 55_000,
        post_bytes: 2_400,
        tool_count: 120,
        drift_count: 0,
        details_json: serde_json::json!({
            "caller_site": "contract_test",
            "paper_ref": "arXiv:2604.21816"
        }),
        ts_ms: 1_700_000_170_003,
    };
    let payload = SsbPayload::GithubTrendingEvent(original.clone());
    let encoded = rmp_serde::to_vec(&payload).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
    match decoded {
        SsbPayload::GithubTrendingEvent(p) => assert_eq!(p, original),
        other => panic!("unexpected variant after round-trip: {other:?}"),
    }
}

#[test]
fn enable_env_const_matches_docs() {
    // The wire-dark env gate must match the name documented in
    // workflow/reasoning/E-22-decisions.md and CLAUDE.md.
    assert_eq!(
        ELAI_GITHUB_TRENDING_ENABLE_ENV, "ELAI_GITHUB_TRENDING_ENABLE",
        "env gate name drift -- decisions doc / dashboard / CLI all read this"
    );
}
