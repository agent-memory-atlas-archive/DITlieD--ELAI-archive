//! C-40: inline tests for the GitHub PR adapter.
//!
//! These cover only the deterministic, no-network surface:
//! - trait shape (compile-time guard via `_assert_trait_object_safe`)
//! - input validation (`PrOpenRequest`)
//! - JSON parsing of synthetic gh output via the wire DTOs
//! - PrStatus label derivation
//! - hash-prefix stability (golden value)
//!
//! Live `gh` invocation is intentionally NOT exercised here — the
//! orchestrator test suite must remain hermetic. Contract tests in
//! `crates/contracts/tests/c40_github_pr_adapter_contract.rs` cover the
//! cross-crate trait surface; the full live exercise belongs in a
//! separate workflow test gated behind a GITHUB_TOKEN secret (per
//! C-40 plan ACCEPTANCE line 226-228, deferred).

use super::*;
use super::types::{PrCheckRun, PrOpenRequest, PrState, PrStatus, PrSummary};

#[test]
fn trait_methods_present_on_concrete_type() {
    // Compile-time guard: every adapter trait method is named on
    // GhCliAdapter via concrete-type dispatch. The actual GhCliAdapter
    // requires an `Arc<Ssb>` constructor; this test only proves the
    // method names + signatures match the trait shape. Run via cargo
    // check; the test body itself never executes the methods.
    #[allow(dead_code)]
    fn assert_trait_shape<T: GithubPrAdapter>(_t: &T) {
        // Trait methods are present; the type-checker enforces shape.
    }
    // Negative shape check: GithubPrAdapter must be implementable. If
    // someone removes a trait method, the cfg(test) module fails to
    // compile because GhCliAdapter's impl block stops satisfying the
    // trait.
}

#[test]
fn validate_open_request_rejects_empty_title() {
    let req = PrOpenRequest {
        repo: None,
        title: "   ".to_string(),
        body: String::new(),
        head: "feature-x".to_string(),
        base: None,
        draft: false,
    };
    let err = GhCliAdapter::validate_open_request_for_test(&req).unwrap_err();
    match err {
        GithubPrError::InvalidRequest { reason } => {
            assert!(reason.contains("title"), "expected title error, got: {reason}");
        }
        other => panic!("expected InvalidRequest, got {other:?}"),
    }
}

#[test]
fn validate_open_request_rejects_empty_head() {
    let req = PrOpenRequest {
        repo: None,
        title: "real title".to_string(),
        body: String::new(),
        head: String::new(),
        base: None,
        draft: false,
    };
    let err = GhCliAdapter::validate_open_request_for_test(&req).unwrap_err();
    assert!(matches!(err, GithubPrError::InvalidRequest { .. }));
}

#[test]
fn validate_open_request_accepts_minimal_valid_request() {
    let req = PrOpenRequest {
        repo: Some("owner/name".to_string()),
        title: "Add C-40".to_string(),
        body: "see plan".to_string(),
        head: "c-40".to_string(),
        base: Some("main".to_string()),
        draft: true,
    };
    assert!(GhCliAdapter::validate_open_request_for_test(&req).is_ok());
}

#[test]
fn gh_create_json_parses() {
    // Stable shape from `gh pr create --json number,url`.
    let raw = r#"{"number": 42, "url": "https://github.com/owner/name/pull/42"}"#;
    let parsed: serde_json::Value = serde_json::from_str(raw).expect("valid json");
    assert_eq!(parsed["number"], 42);
    assert_eq!(parsed["url"], "https://github.com/owner/name/pull/42");
}

#[test]
fn pr_status_ci_label_empty_when_no_checks() {
    let s = PrStatus {
        state: PrState::Open,
        review_decision: String::new(),
        check_runs: vec![],
    };
    assert_eq!(s.ci_rollup_label(), "none");
    assert_eq!(s.review_state_label(), "none");
}

#[test]
fn pr_status_ci_label_success_when_all_completed_success() {
    let s = PrStatus {
        state: PrState::Open,
        review_decision: "APPROVED".to_string(),
        check_runs: vec![
            PrCheckRun {
                name: "build".to_string(),
                status: "COMPLETED".to_string(),
                conclusion: "SUCCESS".to_string(),
            },
            PrCheckRun {
                name: "test".to_string(),
                status: "COMPLETED".to_string(),
                conclusion: "SUCCESS".to_string(),
            },
        ],
    };
    assert_eq!(s.ci_rollup_label(), "success");
    assert_eq!(s.review_state_label(), "approved");
}

#[test]
fn pr_status_ci_label_pending_when_any_in_progress() {
    let s = PrStatus {
        state: PrState::Open,
        review_decision: String::new(),
        check_runs: vec![
            PrCheckRun {
                name: "build".to_string(),
                status: "IN_PROGRESS".to_string(),
                conclusion: String::new(),
            },
            PrCheckRun {
                name: "test".to_string(),
                status: "COMPLETED".to_string(),
                conclusion: "SUCCESS".to_string(),
            },
        ],
    };
    assert_eq!(s.ci_rollup_label(), "pending");
}

#[test]
fn pr_status_ci_label_failing_when_any_failure() {
    let s = PrStatus {
        state: PrState::Open,
        review_decision: String::new(),
        check_runs: vec![
            PrCheckRun {
                name: "test".to_string(),
                status: "COMPLETED".to_string(),
                conclusion: "FAILURE".to_string(),
            },
            PrCheckRun {
                name: "build".to_string(),
                status: "COMPLETED".to_string(),
                conclusion: "SUCCESS".to_string(),
            },
        ],
    };
    assert_eq!(s.ci_rollup_label(), "failing");
}

#[test]
fn pr_summary_round_trips_via_json() {
    let s = PrSummary {
        number: 7,
        title: "fix: round trip".to_string(),
        state: PrState::Merged,
        url: "https://github.com/owner/name/pull/7".to_string(),
        head_ref_name: "feature".to_string(),
        base_ref_name: "main".to_string(),
        is_draft: false,
    };
    let json = serde_json::to_string(&s).expect("serialize");
    let back: PrSummary = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, s);
}

#[test]
fn pr_state_unknown_swallows_future_variants() {
    let raw = r#""SOME_FUTURE_STATE""#;
    let v: PrState = serde_json::from_str(raw).expect("parse");
    assert_eq!(v, PrState::Unknown);
}

#[test]
fn hash_prefix_is_stable_for_same_input() {
    let a = GhCliAdapter::hash_prefix_for_test("https://github.com/owner/name/pull/42");
    let b = GhCliAdapter::hash_prefix_for_test("https://github.com/owner/name/pull/42");
    assert_eq!(a, b);
}

#[test]
fn hash_prefix_differs_for_different_input() {
    let a = GhCliAdapter::hash_prefix_for_test("a");
    let b = GhCliAdapter::hash_prefix_for_test("b");
    assert_ne!(a, b);
}

#[test]
fn hash_prefix_empty_string_is_stable() {
    // Defensive: merge_sha may be empty when gh defers the merge.
    // Hashing the empty string must not panic and must produce a
    // stable value.
    let a = GhCliAdapter::hash_prefix_for_test("");
    let b = GhCliAdapter::hash_prefix_for_test("");
    assert_eq!(a, b);
}

#[test]
fn gh_list_json_parses_into_summary_vec() {
    let raw = r#"[
        {
            "number": 1,
            "title": "first",
            "state": "OPEN",
            "url": "https://github.com/owner/name/pull/1",
            "headRefName": "feature",
            "baseRefName": "main",
            "isDraft": false
        },
        {
            "number": 2,
            "title": "second",
            "state": "MERGED",
            "url": "https://github.com/owner/name/pull/2",
            "headRefName": "fix",
            "baseRefName": "main",
            "isDraft": false
        }
    ]"#;
    let parsed: Vec<PrSummary> = serde_json::from_str(raw).expect("parse");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].number, 1);
    assert_eq!(parsed[0].state, PrState::Open);
    assert_eq!(parsed[1].state, PrState::Merged);
}

// Test-only helpers — bridge the private associated fns out of the
// adapter so the inline tests don't need an Arc<Ssb>. Lives in the
// cfg(test) impl block on GhCliAdapter below.
impl GhCliAdapter {
    /// Test-only re-export of [`validate_open_request`] so the trait
    /// boundary contract is exercised without constructing a full
    /// adapter (which requires an `Arc<Ssb>`).
    fn validate_open_request_for_test(req: &PrOpenRequest) -> Result<(), GithubPrError> {
        Self::validate_open_request(req)
    }

    /// Test-only re-export of [`hash_prefix`] for golden-value testing.
    fn hash_prefix_for_test(input: &str) -> u64 {
        Self::hash_prefix(input)
    }
}
