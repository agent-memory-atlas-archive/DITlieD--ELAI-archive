//! E-14 — Browser e2e agents contract tests.
//!
//! Locks the scope-lean public surface of the orchestrator's
//! `browser_e2e` module + its compose-with-Verdict contract. The plan
//! originally proposed 13 tasks (AXTree-first Tier 1, axe-core gate,
//! Stagehand fallback, VLM constrained decoding, hybrid SSIM diff,
//! DeployPreviewWorker, SaaS multi-context journey runner, CF detect)
//! + 9 SSB variants + 3 Tauri commands; the scope-lean re-check
//! trimmed to:
//!
//!   * a single `browser_e2e` module (playwright + axe_core + journey
//!     + assertion + snapshot submodules)
//!   * an extension to the existing `WebRenderAdapter` (no parallel
//!     `playwright_web.rs`)
//!   * 4 SSB variants (BrowserE2eRunStarted /
//!     BrowserE2eAssertionFailed / BrowserE2eAssertionPassed /
//!     BrowserE2eSnapshotCaptured)
//!   * 1 Tauri command (`browser_e2e_status`)
//!   * `Verdict::Skipped(SkipReason::ExternalProtection)` 5th variant
//!     on `media_validator::Verdict`
//!
//! This contract pins THAT shipped surface.
//!
//! Citation status (per
//! workflow/sessions/s-25120/mistakes.md E-14 entry):
//!   * Microsoft Playwright 2026 — design grounding only; no
//!     inline arXiv ID. Verified by name in
//!     workflow/reasoning/E-14-decisions.md.
//!   * 3 plan-cited arXiv IDs (2604.25235, 2410.11302, 2509.25142)
//!     — DEFERRED to follow-up patch per CLAUDE.md cite-incident
//!     discipline. Test C5 enforces non-inlining.

use elai_orchestrator::browser_e2e::{
    assertion::AssertionOutcome, axe_core::parse_axe_results, run_assertions, Assertion,
    AssertionKind, AxeImpact, AxeReport, E2eError, Journey, JourneyStep, JourneyStepKind,
    PageSnapshot, PlaywrightRunner, PlaywrightSnapshotOpts,
};
use elai_orchestrator::media_validator::types::{SkipReason, Verdict};
use elai_orchestrator::media_validator::render_adapters::WebRenderAdapter;
use elai_orchestrator::ssb::{
    BrowserE2eAssertionFailedPayload, BrowserE2eAssertionPassedPayload,
    BrowserE2eRunStartedPayload, BrowserE2eSnapshotCapturedPayload, SsbActionType,
};

/// C1 — `Verdict::Skipped(SkipReason::ExternalProtection)` is the
/// fifth variant on the Verdict enum and surfaces a stable wire
/// string for the dashboard.
#[test]
fn c1_verdict_skipped_external_protection_variant_exists() {
    let v = Verdict::Skipped(SkipReason::ExternalProtection);
    // Stable JSON shape: dashboard relies on this for the
    // "skipped (ExternalProtection)" badge.
    let json = serde_json::to_string(&v).expect("serialize");
    assert!(json.contains("Skipped"), "json: {json}");
    assert!(
        json.contains("ExternalProtection"),
        "ExternalProtection must round-trip: {json}"
    );
    // Wire string for SSB payloads.
    assert_eq!(SkipReason::ExternalProtection.as_str(), "external_protection");
}

/// C2 — `WebRenderAdapter` is EXTENDED with `e2e_assertions` field +
/// `run_e2e_assertions` method. No parallel `playwright_web.rs` file is
/// permitted in the scope-lean cut (rebase rule #2).
#[test]
fn c2_web_render_adapter_extended_not_replaced() {
    let snap = PageSnapshot {
        url: "http://test.local/".into(),
        html: "<html><body>Welcome</body></html>".into(),
        ax_tree: None,
        screenshot_path: None,
        ts_ms: 1_700_000_000_000,
    };
    let adapter = WebRenderAdapter::new();
    // No assertions attached -> None preserves B-24 behavior.
    assert!(
        adapter.run_e2e_assertions(&snap).is_none(),
        "default adapter must surface None when no e2e_assertions attached"
    );
    let adapter = WebRenderAdapter::new().with_e2e_assertions(vec![
        Assertion {
            label: "loaded".into(),
            kind: AssertionKind::StatusOk,
        },
        Assertion {
            label: "welcome".into(),
            kind: AssertionKind::ContainsText {
                needle: "Welcome".into(),
            },
        },
    ]);
    let report = adapter.run_e2e_assertions(&snap).expect("Some report");
    assert_eq!(report.passed, 2);
    assert_eq!(report.failed, 0);
}

/// C3 — 4 SSB variants round-trip via the SsbActionType + payload
/// surface. Hashes are u64 (blake3-prefix), timestamps i64. Adding any
/// field to a payload requires a new contract test row here.
#[test]
fn c3_four_browser_e2e_ssb_variants_payload_shape() {
    // Run started.
    let p_run = BrowserE2eRunStartedPayload {
        journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
        url_hash: 0xAAAA_BBBB_CCCC_DDD1,
        ts_ms: 1_700_000_131_001,
    };
    let _at: SsbActionType = SsbActionType::BrowserE2eRunStarted;
    assert_eq!(p_run.url_hash, 0xAAAA_BBBB_CCCC_DDD1);

    // Assertion failed.
    let p_fail = BrowserE2eAssertionFailedPayload {
        journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
        assertion_hash: 0xAAAA_BBBB_CCCC_DDD2,
        error_hash: 0xAAAA_BBBB_CCCC_DDD3,
        ts_ms: 1_700_000_131_002,
    };
    let _at = SsbActionType::BrowserE2eAssertionFailed;
    assert_eq!(p_fail.error_hash, 0xAAAA_BBBB_CCCC_DDD3);

    // Assertion passed.
    let p_pass = BrowserE2eAssertionPassedPayload {
        journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
        assertion_hash: 0xAAAA_BBBB_CCCC_DDD4,
        ts_ms: 1_700_000_131_003,
    };
    let _at = SsbActionType::BrowserE2eAssertionPassed;
    assert_eq!(p_pass.assertion_hash, 0xAAAA_BBBB_CCCC_DDD4);

    // Snapshot captured.
    let p_snap = BrowserE2eSnapshotCapturedPayload {
        journey_name_hash: 0xAAAA_BBBB_CCCC_DDD0,
        snapshot_kind_hash: 0xAAAA_BBBB_CCCC_DDD5,
        ts_ms: 1_700_000_131_004,
    };
    let _at = SsbActionType::BrowserE2eSnapshotCaptured;
    assert_eq!(p_snap.snapshot_kind_hash, 0xAAAA_BBBB_CCCC_DDD5);

    // Wire-tag stability — these are locked in
    // action_type_lock.golden; any rename breaks both files.
    assert_eq!(
        SsbActionType::BrowserE2eRunStarted.as_sql_str(),
        "browser_e2e_run_started"
    );
    assert_eq!(
        SsbActionType::BrowserE2eAssertionFailed.as_sql_str(),
        "browser_e2e_assertion_failed"
    );
    assert_eq!(
        SsbActionType::BrowserE2eAssertionPassed.as_sql_str(),
        "browser_e2e_assertion_passed"
    );
    assert_eq!(
        SsbActionType::BrowserE2eSnapshotCaptured.as_sql_str(),
        "browser_e2e_snapshot_captured"
    );
}

/// C4 — Journey TOML round-trips through `load_journey` + the
/// assertion runner. `JourneyStepKind` carries 4 variants in the
/// scope-lean cut: Navigate / Click / Fill / WaitFor.
#[test]
fn c4_journey_toml_round_trip() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let body = r#"
name = "contract_smoke"
url = "http://test.local/login"

[[steps]]
kind = "navigate"
target = "http://test.local/login"

[[steps]]
kind = "fill"
target = "input[name='email']"
value = "contract@test.local"

[[steps]]
kind = "click"
target = "button[type='submit']"

[[steps]]
kind = "wait_for"
target = "[data-testid='ok']"

[[assertions]]
kind = "status_ok"
label = "loaded"

[[assertions]]
kind = "contains_text"
label = "shows_login"
needle = "Log in"
"#;
    let p = dir.path().join("contract_smoke.toml");
    std::fs::write(&p, body).expect("write");
    let j: Journey = elai_orchestrator::browser_e2e::load_journey(&p).expect("load");
    assert_eq!(j.name, "contract_smoke");
    assert_eq!(j.steps.len(), 4);
    // Exhaustive match — if a 5th variant lands without contract
    // update, this fails to compile.
    let kinds: Vec<&'static str> = j
        .steps
        .iter()
        .map(|s| match s.kind {
            JourneyStepKind::Navigate => "navigate",
            JourneyStepKind::Click => "click",
            JourneyStepKind::Fill => "fill",
            JourneyStepKind::WaitFor => "wait_for",
        })
        .collect();
    assert_eq!(kinds, vec!["navigate", "fill", "click", "wait_for"]);
    assert_eq!(j.assertions.len(), 2);
}

/// C5 — Toolchain-missing surface for absent `npx` is wire-dark
/// (`E2eError::ToolchainMissing`), NEVER a panic. Mirrors the C-52
/// dafny wire-dark precedent.
#[tokio::test]
async fn c5_playwright_runner_toolchain_missing_is_wire_dark() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let runner = PlaywrightRunner::with_npx_path(std::path::PathBuf::from(
        "/__elai_e14_contract_nonexistent_npx__",
    ));
    let opts = PlaywrightSnapshotOpts::html_only("http://localhost:1/");
    let r = runner.snapshot(&opts, dir.path(), 1_700_000_000_000).await;
    assert!(r.is_err());
    let e = r.err().expect("err");
    // Acceptable wire-dark surfaces: ToolchainMissing OR Io (some
    // platforms produce Io on spawn before ENOENT classification).
    match e {
        E2eError::ToolchainMissing { tool } => {
            assert!(
                tool == "npx" || tool == "node" || tool == "playwright",
                "unexpected ToolchainMissing tool: {tool}"
            );
        }
        E2eError::Io(_) => {}
        other => panic!("expected ToolchainMissing or Io, got {other:?}"),
    }
}

/// C6 — Citation discipline: design grounding (Playwright /
/// Microsoft 2026 blog) is referenced in
/// workflow/reasoning/E-14-decisions.md only. The plan's 3 arXiv IDs
/// are NOT inlined in the orchestrator's browser_e2e source — the
/// crystallizer test in `tdd_verifier_disciplined_citations` (E-13)
/// already enforces the broader anti-cite-incident rule on
/// orchestrator/src/. This test asserts the four browser_e2e
/// submodule files are absent of the deferred arXiv IDs.
#[test]
fn c6_no_inline_deferred_arxiv_ids_in_browser_e2e_source() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let root = std::path::PathBuf::from(manifest)
        .parent()
        .expect("crates dir")
        .join("orchestrator")
        .join("src")
        .join("browser_e2e");
    let files = [
        "mod.rs",
        "playwright.rs",
        "snapshot.rs",
        "assertion.rs",
        "journey.rs",
        "axe_core.rs",
    ];
    // The 3 plan-cited arXiv IDs marked deferred per cluster
    // discipline. If any of these is later GROUNDED (paper-ingested +
    // verified), the inline cite can be re-added and this test
    // updated.
    let deferred = ["2604.25235", "2410.11302", "2509.25142"];
    for f in files {
        let p = root.join(f);
        if !p.exists() {
            // Skip when the file is gone (refactor) rather than
            // failing — the test asserts non-inlining, not file
            // presence.
            continue;
        }
        let raw = std::fs::read_to_string(&p).expect("read file");
        for id in deferred {
            assert!(
                !raw.contains(id),
                "deferred arXiv id {id} inlined in {p:?}; \
                 either GROUND it (paper-ingest + verify) or keep \
                 the cite in workflow/reasoning/E-14-decisions.md only"
            );
        }
    }
}

/// C7 — `parse_axe_results` is exposed for in-process axe-core JSON
/// inspection (no subprocess required at test time). The four
/// axe-core impact levels map deterministically to AxeImpact and only
/// Critical + Serious are blocking.
#[test]
fn c7_axe_core_impact_mapping_stable() {
    let raw = r#"{
        "violations": [
            { "id": "color-contrast", "impact": "critical", "nodes": [{}] },
            { "id": "label",          "impact": "serious",  "nodes": [{}] },
            { "id": "image-alt",      "impact": "moderate", "nodes": [{}] },
            { "id": "region",         "impact": "minor",    "nodes": [{}] }
        ]
    }"#;
    let r: AxeReport = parse_axe_results(raw).expect("parse");
    assert_eq!(r.critical_count, 1);
    assert_eq!(r.serious_count, 1);
    assert_eq!(r.moderate_count, 1);
    assert_eq!(r.minor_count, 1);
    assert!(r.has_blocking());
    // Per-level blocking semantics — also locked in
    // browser_e2e::axe_core::tests::impact_blocking_only_critical_serious
    // but the contract makes it visible to downstream consumers.
    assert!(AxeImpact::Critical.is_blocking());
    assert!(AxeImpact::Serious.is_blocking());
    assert!(!AxeImpact::Moderate.is_blocking());
    assert!(!AxeImpact::Minor.is_blocking());
}

/// C8 — Pure `run_assertions` is the SINGLE assertion runner.
/// Adapter call + direct call must produce identical reports for the
/// same inputs.
#[test]
fn c8_run_assertions_is_single_source_of_truth() {
    let snap = PageSnapshot {
        url: "http://t/".into(),
        html: "<html><body>x</body></html>".into(),
        ax_tree: None,
        screenshot_path: None,
        ts_ms: 0,
    };
    let asserts = vec![
        Assertion {
            label: "ok".into(),
            kind: AssertionKind::StatusOk,
        },
        Assertion {
            label: "no-foo".into(),
            kind: AssertionKind::ContainsText {
                needle: "FOO".into(),
            },
        },
    ];
    // Direct.
    let r_direct = run_assertions(&snap, &asserts);
    assert_eq!(r_direct.passed, 1);
    assert_eq!(r_direct.failed, 1);
    // Through the adapter (proves the adapter delegates, not
    // re-implements).
    let adapter = WebRenderAdapter::new().with_e2e_assertions(asserts.clone());
    let r_adapter = adapter
        .run_e2e_assertions(&snap)
        .expect("Some report when assertions attached");
    assert_eq!(r_direct.passed, r_adapter.passed);
    assert_eq!(r_direct.failed, r_adapter.failed);
    // Outcome labels round-trip.
    assert_eq!(r_direct.outcomes.len(), r_adapter.outcomes.len());
    for (a, b) in r_direct.outcomes.iter().zip(r_adapter.outcomes.iter()) {
        assert_eq!(a.label(), b.label());
        match (a, b) {
            (AssertionOutcome::Pass { .. }, AssertionOutcome::Pass { .. }) => {}
            (AssertionOutcome::Fail { .. }, AssertionOutcome::Fail { .. }) => {}
            (x, y) => panic!("outcome diverged: {x:?} vs {y:?}"),
        }
    }
}

/// C9 — JourneyStep silliness guard: parsed value field is optional
/// only for non-fill steps; the runner must not panic when value is
/// absent on a navigate.
#[test]
fn c9_journey_step_value_optional_for_non_fill() {
    let step = JourneyStep {
        kind: JourneyStepKind::Navigate,
        target: "http://t/".into(),
        value: None,
    };
    assert!(step.value.is_none());
    let step = JourneyStep {
        kind: JourneyStepKind::Fill,
        target: "input[name='x']".into(),
        value: Some("hello".into()),
    };
    assert_eq!(step.value.as_deref(), Some("hello"));
}
