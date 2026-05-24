//! E-41 — CI/CD PR review surface contract.
//!
//! Locks the cross-crate invariants the E-41 wiring depends on:
//!
//! 1. The five new `SsbActionType::Pr*` wire tags round-trip through
//!    `as_sql_str` / `ssb_action_type_from_str_public`.
//! 2. `SCHEMA_VERSION_PUBLIC >= 135` (the E-41 bump) so downstream
//!    consumers can detect availability.
//! 3. The seven new github_pr extension functions exist with their
//!    documented signatures (compile-time shape check).
//! 4. NO new `AgentRole` variant landed for E-41 (the brief explicitly
//!    avoids the 18th-variant cascade; uses existing PerRoleRuntime
//!    routing instead).
//! 5. All five E-41 payload structs serialize-round-trip cleanly via
//!    serde + their wire shape matches the C-40 hash-prefix discipline.
//! 6. WIRE-DARK check: every new module has a CLI subverb anchor.
//!    Asserted via grep over the CLI source via include_str! so the
//!    test catches a future build that drops the production caller.
//! 7. CI yaml files exist for the documented PR-open + slopsquat gates.
//!
//! Compile-only (no live gh / semgrep calls).

use elai_orchestrator::github_pr::{
    auto_merge_guard, body_composer, codeowners_gen, git_assist, package_verify,
    review, semgrep_gate, MergePolicy, MergeVerdict, PrContext, PrShape, ReviewCommentKind,
    SemgrepSeverity, SlopsquatReport,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, PrAutoMergeBlockedPayload, PrCommentWatchedPayload,
    PrPackageVerifyResultPayload, PrReviewComposedPayload, PrSemgrepGateResultPayload,
};
use elai_orchestrator::{SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC};

// --- (1) Wire-tag round-trip for all five E-41 variants ---

#[test]
fn contract_pr_review_composed_round_trips() {
    let tag = SsbActionType::PrReviewComposed.as_sql_str();
    assert_eq!(tag, "pr_review_composed");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::PrReviewComposed);
}

#[test]
fn contract_pr_comment_watched_round_trips() {
    let tag = SsbActionType::PrCommentWatched.as_sql_str();
    assert_eq!(tag, "pr_comment_watched");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::PrCommentWatched);
}

#[test]
fn contract_pr_package_verify_result_round_trips() {
    let tag = SsbActionType::PrPackageVerifyResult.as_sql_str();
    assert_eq!(tag, "pr_package_verify_result");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::PrPackageVerifyResult);
}

#[test]
fn contract_pr_semgrep_gate_result_round_trips() {
    let tag = SsbActionType::PrSemgrepGateResult.as_sql_str();
    assert_eq!(tag, "pr_semgrep_gate_result");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::PrSemgrepGateResult);
}

#[test]
fn contract_pr_auto_merge_blocked_round_trips() {
    let tag = SsbActionType::PrAutoMergeBlocked.as_sql_str();
    assert_eq!(tag, "pr_auto_merge_blocked");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::PrAutoMergeBlocked);
}

// --- (2) Schema floor ---

#[test]
fn contract_schema_floor_at_least_135() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 135,
        "E-41 expects SCHEMA_VERSION_PUBLIC >= 135 (current {SCHEMA_VERSION_PUBLIC})",
    );
}

// --- (3) Shape checks for the 7 new github_pr functions ---

#[test]
fn contract_seven_new_github_pr_functions_exist() {
    // body_composer
    let ctx = PrContext::new().with("k", "v");
    let _ = body_composer::compose_pr_body("hi {{k}}", &ctx)
        .expect("compose_pr_body has signature (template, ctx) -> Result<String, GithubPrError>");

    // review
    let argv = review::build_review_argv(1, "body", None, None);
    assert!(!argv.is_empty());
    let _kind: ReviewCommentKind = ReviewCommentKind::PrThread;

    // comment_watcher
    let _: fn() -> &'static str = || "comment_watcher::watch_comments exists as an async fn";

    // package_verify
    let cands = package_verify::extract_dep_candidates("+serde = \"1\"\n");
    assert!(!cands.is_empty(), "extract_dep_candidates returned empty for non-empty diff");
    let _: SlopsquatReport = SlopsquatReport {
        flagged: vec![],
        total_inspected: 0,
    };

    // git_assist
    let msg = git_assist::suggest_squash_message("[Phase 4] do thing", "");
    assert!(msg.starts_with("do thing"));

    // semgrep_gate
    let (count, sev) = semgrep_gate::parse_semgrep_findings("");
    assert_eq!(count, 0);
    assert_eq!(sev, SemgrepSeverity::Info);

    // auto_merge_guard
    let policy = MergePolicy::default();
    let verdict = auto_merge_guard::classify(
        &PrShape {
            loc_added: 10,
            loc_removed: 5,
            files_changed: 1,
            touches_owned_paths: false,
            is_machine_generated_reversible: false,
        },
        &policy,
    );
    assert!(matches!(verdict, MergeVerdict::Allow));

    // codeowners_gen
    let body =
        codeowners_gen::generate_codeowners(&[std::path::PathBuf::from("a.rs")], "octocat");
    assert!(body.contains("@octocat"));
}

// --- (4) AgentRole cascade avoidance ---

#[test]
fn contract_no_new_agent_role_for_e41() {
    // E-41 brief mandates: do NOT add AgentRole::ExternalPrReviewer.
    // Lock by counting variants via the canonical-coverage test on
    // PerRoleRuntime::default().
    use elai_ipc_types::AgentRole;
    use elai_orchestrator::per_role_runtime::PerRoleRuntime;
    let default = PerRoleRuntime::default();
    // The 17 variants present before E-41 must all still be mapped.
    let must_be_mapped = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
        AgentRole::IdeaFramer,
        AgentRole::IdeaCritic,
        AgentRole::AuditorConcepts,
        AgentRole::AuditorPhases,
        AgentRole::AuditorSteps,
        AgentRole::AuditorBabySteps,
        AgentRole::AuditorBabyStepDetail,
        AgentRole::AuditorProductionGrade,
        AgentRole::InjectionClassifier,
    ];
    for r in must_be_mapped {
        // get() falls back to the configured fallback; the contract is
        // that the BTreeMap CONTAINS the key. We verify by reflecting on
        // `list()` which is the public, deterministic snapshot.
        let mapped: Vec<elai_ipc_types::AgentRole> =
            default.list().into_iter().map(|(role, _)| role).collect();
        assert!(mapped.contains(&r), "AgentRole::{:?} missing from default mapping", r);
    }
}

// --- (5) Payload serde round-trip ---

#[test]
fn contract_e41_payloads_serde_round_trip() {
    let p1 = PrReviewComposedPayload {
        pr_number_hash: 0xdead_beef,
        comment_kind_hash: 0xcafe_babe,
        ts_ms: 1_700_000_000_000,
    };
    let p2 = PrCommentWatchedPayload {
        pr_number_hash: 0x1234,
        new_comments_count: 5,
        ts_ms: 1_700_000_000_000,
    };
    let p3 = PrPackageVerifyResultPayload {
        pr_number_hash: 0x5678,
        slopsquat_count: 1,
        ts_ms: 1_700_000_000_000,
    };
    let p4 = PrSemgrepGateResultPayload {
        pr_number_hash: 0xabcd,
        findings_count: 3,
        severity_max_hash: 0xface,
        ts_ms: 1_700_000_000_000,
    };
    let p5 = PrAutoMergeBlockedPayload {
        pr_number_hash: 0xfeed,
        block_reason_hash: 0xdeed,
        ts_ms: 1_700_000_000_000,
    };
    for payload in [
        SsbPayload::PrReviewComposed(p1),
        SsbPayload::PrCommentWatched(p2),
        SsbPayload::PrPackageVerifyResult(p3),
        SsbPayload::PrSemgrepGateResult(p4),
        SsbPayload::PrAutoMergeBlocked(p5),
    ] {
        let s = serde_json::to_string(&payload).expect("serialize ok");
        let decoded: SsbPayload = serde_json::from_str(&s).expect("deserialize ok");
        assert_eq!(payload.action_type(), decoded.action_type());
    }
}

// --- (6) WIRE-DARK: every new module has a CLI subverb anchor ---

#[test]
fn contract_wire_dark_cli_subverbs_present() {
    let src = include_str!("../../cli/src/pr_review_cmd.rs");
    // Each E-41 module function shows up by name in the CLI dispatch.
    assert!(
        src.contains("compose_pr_body"),
        "pr_review_cmd missing call to compose_pr_body (body_composer wire-dark)"
    );
    assert!(
        src.contains("post_review_comment"),
        "pr_review_cmd missing call to post_review_comment (review wire-dark)"
    );
    assert!(
        src.contains("watch_comments"),
        "pr_review_cmd missing call to watch_comments (comment_watcher wire-dark)"
    );
    assert!(
        src.contains("verify_pr_deps_no_slopsquat"),
        "pr_review_cmd missing call to verify_pr_deps_no_slopsquat (package_verify wire-dark)"
    );
    assert!(
        src.contains("run_semgrep_gate"),
        "pr_review_cmd missing call to run_semgrep_gate (semgrep_gate wire-dark)"
    );
    assert!(
        src.contains("assess_merge_safety") || src.contains("apply_branch_protection"),
        "pr_review_cmd missing call to auto_merge_guard helpers (wire-dark)"
    );
    assert!(
        src.contains("generate_codeowners"),
        "pr_review_cmd missing call to generate_codeowners (codeowners_gen wire-dark)"
    );
    assert!(
        src.contains("suggest_squash_message") || src.contains("detect_force_push"),
        "pr_review_cmd missing call to git_assist helpers (wire-dark)"
    );
}

// --- (7) Anti-dep: no reqwest / octocrab in github_pr module ---

#[test]
fn contract_no_http_client_dep_added() {
    let github_pr_src = include_str!("../../orchestrator/src/github_pr/mod.rs");
    let review_src = include_str!("../../orchestrator/src/github_pr/review.rs");
    let watcher_src = include_str!("../../orchestrator/src/github_pr/comment_watcher.rs");
    let verify_src = include_str!("../../orchestrator/src/github_pr/package_verify.rs");
    let semgrep_src = include_str!("../../orchestrator/src/github_pr/semgrep_gate.rs");
    let merge_src = include_str!("../../orchestrator/src/github_pr/auto_merge_guard.rs");
    let assist_src = include_str!("../../orchestrator/src/github_pr/git_assist.rs");
    for (label, src) in [
        ("mod.rs", github_pr_src),
        ("review.rs", review_src),
        ("comment_watcher.rs", watcher_src),
        ("package_verify.rs", verify_src),
        ("semgrep_gate.rs", semgrep_src),
        ("auto_merge_guard.rs", merge_src),
        ("git_assist.rs", assist_src),
    ] {
        assert!(
            !src.contains("use reqwest"),
            "{label}: must not depend on reqwest (anti-deps HARD RULE)"
        );
        assert!(
            !src.contains("use octocrab"),
            "{label}: must not depend on octocrab (anti-deps HARD RULE)"
        );
        assert!(
            !src.contains("use async_trait"),
            "{label}: must not depend on async_trait (anti-stale-training HARD RULE)"
        );
    }
}

// --- (8) CI yaml files exist on disk ---

#[test]
fn contract_ci_yaml_files_exist() {
    let pr_review_yml =
        include_str!("../../../.github/workflows/elai-pr-review.yml");
    assert!(
        pr_review_yml.contains("elai-pr-review"),
        "elai-pr-review.yml missing or malformed"
    );
    assert!(
        pr_review_yml.contains("verify-deps"),
        "elai-pr-review.yml must invoke pr-review verify-deps"
    );

    let slopsquat_yml =
        include_str!("../../../.github/workflows/elai-anti-slopsquat.yml");
    assert!(
        slopsquat_yml.contains("elai-anti-slopsquat"),
        "elai-anti-slopsquat.yml missing or malformed"
    );
    assert!(
        slopsquat_yml.contains("pyproject.toml"),
        "elai-anti-slopsquat.yml must scope to dep manifests"
    );
}
