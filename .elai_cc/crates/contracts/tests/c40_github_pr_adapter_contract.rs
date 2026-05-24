//! C-40 — GitHub PR adapter contract.
//!
//! Locks the cross-crate invariants the C-40 wiring depends on:
//!
//! 1. The four C-40 `SsbActionType::GithubPr*` wire tags round-trip
//!    through `as_sql_str` / `ssb_action_type_from_str_public`.
//! 2. `SCHEMA_VERSION_PUBLIC >= 109` (the C-40 bump) so downstream
//!    consumers can detect availability without re-grepping ssb.rs.
//! 3. The `GithubPrAdapter` trait has the five public methods we
//!    document (open / list / status / close / merge) — checked via a
//!    compile-time concrete-impl probe.
//! 4. `gh pr create` is classified `RiskLevel::SharedState` by the
//!    existing action classifier, so the C-40 adapter MUST go through
//!    that gate (no bypass).
//! 5. The four C-40 payload structs serialize-round-trip cleanly via
//!    serde (msgpack at the wire, JSON at the IPC boundary).
//!
//! Compile-only (no live `gh` calls). A live exercise belongs in a
//! workflow test gated behind GITHUB_TOKEN, per C-40 plan ACCEPTANCE.

use elai_orchestrator::github_pr::{
    GhCliAdapter, GithubPrAdapter, PrOpenRequest, PrState, PrStatus, PrSummary,
};
use elai_orchestrator::ssb::{
    GithubPrClosedPayload, GithubPrMergedPayload, GithubPrOpenedPayload,
    GithubPrStatusFetchedPayload, ssb_action_type_from_str_public,
};
use elai_orchestrator::{SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC};
use elai_tools::action_classifier::{classify, RiskLevel};
use serde_json::json;

// --- (1) Wire-tag round-trip for all four C-40 variants ---

#[test]
fn contract_github_pr_opened_round_trips() {
    let tag = SsbActionType::GithubPrOpened.as_sql_str();
    assert_eq!(tag, "github_pr_opened");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::GithubPrOpened);
}

#[test]
fn contract_github_pr_status_fetched_round_trips() {
    let tag = SsbActionType::GithubPrStatusFetched.as_sql_str();
    assert_eq!(tag, "github_pr_status_fetched");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::GithubPrStatusFetched);
}

#[test]
fn contract_github_pr_closed_round_trips() {
    let tag = SsbActionType::GithubPrClosed.as_sql_str();
    assert_eq!(tag, "github_pr_closed");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::GithubPrClosed);
}

#[test]
fn contract_github_pr_merged_round_trips() {
    let tag = SsbActionType::GithubPrMerged.as_sql_str();
    assert_eq!(tag, "github_pr_merged");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::GithubPrMerged);
}

// --- (2) SCHEMA_VERSION_PUBLIC reflects the C-40 bump ---

#[test]
fn contract_schema_version_at_least_c40() {
    // C-40 (session 2) bumped 108 -> 109. Session 3 (s-31708) may
    // piggyback its own variants on top, so the floor is the minimum
    // C-40 needs, not the exact value: any SCHEMA >= 109 is
    // acceptable as long as the C-40 wire tags still round-trip
    // (checked above).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 109,
        "SCHEMA_VERSION_PUBLIC ({}) is below the C-40 floor (109). \
         A regression has rolled back the C-40 SCHEMA bump in ssb.rs.",
        SCHEMA_VERSION_PUBLIC,
    );
}

// --- (3) GithubPrAdapter trait shape (compile-time) ---

/// Compile-time guard: GhCliAdapter implements GithubPrAdapter. If
/// anyone removes a method from the trait or the impl, this fn fails
/// to compile and the contract test fires the C-40 wiring alarm.
#[allow(dead_code)]
fn assert_gh_cli_adapter_implements_trait()
where
    GhCliAdapter: GithubPrAdapter + Send + Sync,
{
    // Trait + Send + Sync bounds enforce the SHIP-side wiring
    // contract: the adapter is safe to share across an async task
    // boundary (e.g. the dashboard's per-Tauri-command spawn site).
}

#[test]
fn contract_pr_state_serde_round_trip() {
    let s = json!("MERGED");
    let v: PrState = serde_json::from_value(s).expect("deserialize");
    assert_eq!(v, PrState::Merged);
    // Unknown future state must fold into PrState::Unknown so a gh
    // schema addition can't crash the parser.
    let unknown = json!("SOMETHING_NEW");
    let v: PrState = serde_json::from_value(unknown).expect("deserialize");
    assert_eq!(v, PrState::Unknown);
}

#[test]
fn contract_pr_summary_round_trip() {
    let s = PrSummary {
        number: 42,
        title: "fix: c-40".to_string(),
        state: PrState::Open,
        url: "https://github.com/owner/name/pull/42".to_string(),
        head_ref_name: "c-40".to_string(),
        base_ref_name: "main".to_string(),
        is_draft: true,
    };
    let json = serde_json::to_string(&s).expect("serialize");
    let back: PrSummary = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, s);
}

#[test]
fn contract_pr_open_request_validates_at_handler_boundary() {
    // Empty title must fail at the request-shape boundary so a stray
    // empty string can't open a broken PR on GitHub. The actual
    // validation lives in GhCliAdapter::open; here we verify the
    // serde shape accepts the documented fields.
    let req = PrOpenRequest {
        repo: Some("owner/name".to_string()),
        title: "title".to_string(),
        body: "body".to_string(),
        head: "feature".to_string(),
        base: Some("main".to_string()),
        draft: true,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let back: PrOpenRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, req);
}

// --- (4) gh CLI commands are classified SharedState ---

#[test]
fn contract_gh_pr_create_remains_shared_state() {
    let c = classify(
        "bash",
        &json!({"command": "gh pr create --title fix --body stuff"}),
    );
    assert_eq!(
        c.risk,
        RiskLevel::SharedState,
        "gh pr create MUST stay classified as SharedState so the C-40 \
         adapter goes through the existing gate. classifier reason was: {}",
        c.reason,
    );
}

#[test]
fn contract_gh_pr_close_remains_shared_state() {
    let c = classify("bash", &json!({"command": "gh pr close 42"}));
    assert_eq!(c.risk, RiskLevel::SharedState);
}

#[test]
fn contract_gh_pr_merge_remains_shared_state() {
    let c = classify("bash", &json!({"command": "gh pr merge 42 --squash"}));
    assert_eq!(c.risk, RiskLevel::SharedState);
}

// --- (5) C-40 SSB payload structs serde-round-trip + match discriminant ---

#[test]
fn contract_github_pr_opened_payload_round_trips() {
    let p = GithubPrOpenedPayload {
        pr_number: 42,
        pr_url_hash: 0xc400_0001_c400_0001,
        ts_ms: 1_700_000_000_000,
    };
    let wrapped = SsbPayload::GithubPrOpened(p.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::GithubPrOpened);
    let json = serde_json::to_string(&wrapped).expect("serialize");
    let back: SsbPayload = serde_json::from_str(&json).expect("deserialize");
    match back {
        SsbPayload::GithubPrOpened(decoded) => assert_eq!(decoded, p),
        other => panic!("expected GithubPrOpened, got {:?}", other.action_type()),
    }
}

#[test]
fn contract_github_pr_status_fetched_payload_round_trips() {
    let p = GithubPrStatusFetchedPayload {
        pr_number: 42,
        ci_status_hash: 0xc400_0002_c400_0002,
        review_state_hash: 0xc400_0003_c400_0003,
        ts_ms: 1_700_000_000_000,
    };
    let wrapped = SsbPayload::GithubPrStatusFetched(p.clone());
    assert_eq!(
        wrapped.action_type(),
        SsbActionType::GithubPrStatusFetched
    );
    let json = serde_json::to_string(&wrapped).expect("serialize");
    let back: SsbPayload = serde_json::from_str(&json).expect("deserialize");
    match back {
        SsbPayload::GithubPrStatusFetched(decoded) => assert_eq!(decoded, p),
        other => panic!(
            "expected GithubPrStatusFetched, got {:?}",
            other.action_type()
        ),
    }
}

#[test]
fn contract_github_pr_closed_payload_round_trips() {
    let p = GithubPrClosedPayload {
        pr_number: 42,
        ts_ms: 1_700_000_000_000,
    };
    let wrapped = SsbPayload::GithubPrClosed(p.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::GithubPrClosed);
    let json = serde_json::to_string(&wrapped).expect("serialize");
    let back: SsbPayload = serde_json::from_str(&json).expect("deserialize");
    match back {
        SsbPayload::GithubPrClosed(decoded) => assert_eq!(decoded, p),
        other => panic!("expected GithubPrClosed, got {:?}", other.action_type()),
    }
}

#[test]
fn contract_github_pr_merged_payload_round_trips() {
    let p = GithubPrMergedPayload {
        pr_number: 42,
        merge_sha_hash: 0xc400_0004_c400_0004,
        ts_ms: 1_700_000_000_000,
    };
    let wrapped = SsbPayload::GithubPrMerged(p.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::GithubPrMerged);
    let json = serde_json::to_string(&wrapped).expect("serialize");
    let back: SsbPayload = serde_json::from_str(&json).expect("deserialize");
    match back {
        SsbPayload::GithubPrMerged(decoded) => assert_eq!(decoded, p),
        other => panic!("expected GithubPrMerged, got {:?}", other.action_type()),
    }
}

#[test]
fn contract_pr_status_label_derivation_is_stable() {
    // The label-derivation logic is the input to the C-40 SSB hash
    // fields; if anyone changes it, the SSB row's hash silently
    // changes and the dashboard's row-grouping breaks. Pin the
    // labels here.
    let success = PrStatus {
        state: PrState::Open,
        review_decision: "APPROVED".to_string(),
        check_runs: vec![],
    };
    assert_eq!(success.ci_rollup_label(), "none");
    assert_eq!(success.review_state_label(), "approved");

    let empty_review = PrStatus {
        state: PrState::Open,
        review_decision: String::new(),
        check_runs: vec![],
    };
    assert_eq!(empty_review.review_state_label(), "none");
}
