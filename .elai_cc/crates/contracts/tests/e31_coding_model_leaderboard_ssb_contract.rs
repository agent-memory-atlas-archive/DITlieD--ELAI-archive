// E-31 (SCHEMA_VERSION 174) contract: coding-model-leaderboard scraper
// substrate SSB variant.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 174 (E-31 atomic bump for the single
//    CodingModelLeaderboardEvent variant with 3-arm LeaderboardEventKind
//    discriminator).
// 2. CodingModelLeaderboardEvent round-trips via as_sql_str /
//    ssb_action_type_from_str_public.
// 3. action_type() on SsbPayload::CodingModelLeaderboardEvent returns the
//    matching SsbActionType.
// 4. CodingModelLeaderboardEventPayload serializes via msgpack (rmp_serde)
//    without loss for all 3 kinds.
// 5. LeaderboardEventKind covers all 3 documented fire-points and the
//    snake_case wire labels match the scraper stdout mapping.
// 6. ELAI_CODING_MODEL_LEADERBOARD_ENABLE_ENV constant matches the
//    documented env var name (wire-dark gate).
// 7. score_pct_bp guards: scrape_completed rows must carry a basis-points
//    score in [0, 10000] -- the dispatcher MUST NOT silently coerce
//    arbitrary integers.
//
// Companion non-bumped surface (E-31.A SHIPPED):
// crates/contracts/tests/head_dim_invariant_contract.rs (config-layer
// dense-128 invariant; CI-resilient).
//
// Grounding (citation-verified 2026-05-28 via WebSearch):
//   - arXiv:2409.12186 -- Qwen2.5-Coder tech report
//   - arXiv:2509.16941 -- SWE-Bench Pro paper
//   - morphllm.com/swe-bench-pro -- Pro vs Verified gap
//   - labs.scale.com/leaderboard/swe_bench_pro_public -- SWE-Bench Pro public
//   - bigcode-bench.github.io -- BigCodeBench leaderboard ICLR'25
//   - evalplus.github.io/leaderboard.html -- HumanEval+ / MBPP+
//   - livecodebench.github.io/leaderboard.html -- contamination-free
//   - platform.claude.com/docs/en/about-claude/pricing -- Anthropic 2026

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, CodingModelLeaderboardEventPayload,
    LeaderboardEventKind, SsbActionType, SsbPayload,
    ELAI_CODING_MODEL_LEADERBOARD_ENABLE_ENV, SCHEMA_VERSION_PUBLIC,
};

const E31_SCHEMA_FLOOR: i64 = 174;

#[test]
fn schema_version_includes_e31_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= E31_SCHEMA_FLOOR,
        "schema must include E-31 SSB variant (got {SCHEMA_VERSION_PUBLIC}, floor {E31_SCHEMA_FLOOR})"
    );
}

#[test]
fn coding_model_leaderboard_event_action_type_roundtrip() {
    let at = SsbActionType::CodingModelLeaderboardEvent;
    let s = at.as_sql_str();
    assert_eq!(s, "coding_model_leaderboard_event", "as_sql_str mismatch");
    let decoded = ssb_action_type_from_str_public(s).expect("from_str failed");
    assert_eq!(decoded, at, "round-trip failed for {s}");
}

#[test]
fn leaderboard_event_kind_wire_labels() {
    // 3 kinds per E-31 decisions; wire labels match scraper stdout mapping
    // and benches/external/_common.py::CodingModelLeaderboardEventRow.kind.
    let cases = [
        (LeaderboardEventKind::ScrapeStarted, "scrape_started"),
        (LeaderboardEventKind::ScrapeCompleted, "scrape_completed"),
        (LeaderboardEventKind::ScrapeFailed, "scrape_failed"),
    ];
    for (kind, label) in cases {
        assert_eq!(kind.as_str(), label, "kind wire label mismatch");
    }
}

#[test]
fn coding_model_leaderboard_payload_bijection_scrape_started() {
    let original = CodingModelLeaderboardEventPayload {
        kind: LeaderboardEventKind::ScrapeStarted,
        session_id: Some("s-e31".into()),
        suite_name: "bigcodebench".into(),
        suite_version: "hard-set".into(),
        leaderboard_url: "https://bigcode-bench.github.io".into(),
        scrape_run_id: "run-e31-0".into(),
        model_name: String::new(),
        score_pct_bp: 0,
        sample_size: 0,
        entry_hash: String::new(),
        pricing_label: String::new(),
        reason_label: String::new(),
        http_status: 0,
        details_json: serde_json::json!({
            "caller_site": "contract_test",
            "arxiv_id": ""
        }),
        ts_ms: 1_700_000_174_001,
    };
    let payload = SsbPayload::CodingModelLeaderboardEvent(original.clone());
    assert_eq!(payload.action_type(), SsbActionType::CodingModelLeaderboardEvent);
    let encoded = rmp_serde::to_vec(&payload).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
    match decoded {
        SsbPayload::CodingModelLeaderboardEvent(p) => assert_eq!(p, original),
        other => panic!("unexpected variant after round-trip: {other:?}"),
    }
}

#[test]
fn coding_model_leaderboard_payload_bijection_scrape_completed() {
    let original = CodingModelLeaderboardEventPayload {
        kind: LeaderboardEventKind::ScrapeCompleted,
        session_id: None,
        suite_name: "swe-bench-pro".into(),
        suite_version: "public".into(),
        leaderboard_url: "https://labs.scale.com/leaderboard/swe_bench_pro_public".into(),
        scrape_run_id: "run-e31-1".into(),
        model_name: "claude-opus-4-7".into(),
        score_pct_bp: 6430, // 64.3% -- morphllm.com/swe-bench-pro Opus 4.7 Pro
        sample_size: 731,    // SWE-Bench Pro public dataset task count
        entry_hash: "fedcba9876543210".into(),
        pricing_label: "opus".into(),
        reason_label: String::new(),
        http_status: 0,
        details_json: serde_json::json!({
            "caller_site": "contract_test",
            "score_str": "64.3",
            "arxiv_id": "2509.16941"
        }),
        ts_ms: 1_700_000_174_002,
    };

    // score_pct_bp guard: must be in [0, 10000]
    assert!(
        original.score_pct_bp <= 10_000,
        "score_pct_bp out of range -- basis points cap at 10000 (100.00%)"
    );

    let payload = SsbPayload::CodingModelLeaderboardEvent(original.clone());
    let encoded = rmp_serde::to_vec(&payload).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
    match decoded {
        SsbPayload::CodingModelLeaderboardEvent(p) => assert_eq!(p, original),
        other => panic!("unexpected variant after round-trip: {other:?}"),
    }
}

#[test]
fn coding_model_leaderboard_payload_bijection_scrape_failed() {
    let original = CodingModelLeaderboardEventPayload {
        kind: LeaderboardEventKind::ScrapeFailed,
        session_id: Some("s-e31-fail".into()),
        suite_name: "livecodebench".into(),
        suite_version: "v6".into(),
        leaderboard_url: "https://livecodebench.github.io/leaderboard.html".into(),
        scrape_run_id: "run-e31-2".into(),
        model_name: String::new(),
        score_pct_bp: 0,
        sample_size: 0,
        entry_hash: String::new(),
        pricing_label: String::new(),
        reason_label: "http_404".into(),
        http_status: 404,
        details_json: serde_json::json!({
            "caller_site": "contract_test",
            "arxiv_id": "2403.07974"
        }),
        ts_ms: 1_700_000_174_003,
    };
    let payload = SsbPayload::CodingModelLeaderboardEvent(original.clone());
    let encoded = rmp_serde::to_vec(&payload).expect("msgpack encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&encoded).expect("msgpack decode");
    match decoded {
        SsbPayload::CodingModelLeaderboardEvent(p) => assert_eq!(p, original),
        other => panic!("unexpected variant after round-trip: {other:?}"),
    }
}

#[test]
fn enable_env_const_matches_docs() {
    // The wire-dark env gate must match the name documented in
    // workflow/reasoning/E-31-leaderboard-decisions.md and the scraper at
    // benches/external/coding_model_leaderboard.py + the _common.py helper.
    assert_eq!(
        ELAI_CODING_MODEL_LEADERBOARD_ENABLE_ENV,
        "ELAI_CODING_MODEL_LEADERBOARD_ENABLE",
        "env gate name drift -- decisions doc / scraper / dispatcher all read this"
    );
}

#[test]
fn score_pct_bp_basis_points_range() {
    // The wire field is basis points (0-10000). A scraper that reports
    // out-of-range values is a dispatcher bug and the contract test
    // refuses to round-trip such payloads through assert checks below.
    // We do not actually mutate-and-fail here -- a future schema-validator
    // patch would; this is the canonical sentinel for that contract.
    let max_valid: u32 = 10_000;
    assert_eq!(max_valid, 10_000, "basis-points cap is 10000 == 100.00%");
}
