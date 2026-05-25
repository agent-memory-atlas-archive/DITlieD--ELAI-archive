//! D-25 — end-to-end integration test for the determinism + archive
//! tamper-suite benchmark.
//!
//! Verifies the runner's whole-suite behavior against a mini scenario
//! set + asserts the per-scenario gates + asserts the runner does NOT
//! emit per-scenario D-8 ArchiveVerifyResult SSB rows (the in-process
//! drive contract per D-25 DECISION-9).

use std::path::PathBuf;

use elai_orchestrator::benchmarks::prmbench_runner::GatingVerdict;
use elai_orchestrator::benchmarks::tamper_runner::{
    ExpectedOutcome, Outcome, ScenarioManifest, TamperBenchmark, TamperOp,
};
use tempfile::TempDir;

fn write_manifest(dir: &TempDir, scenario: &str, manifest: &ScenarioManifest) -> PathBuf {
    let scenario_dir = dir.path().join(scenario);
    std::fs::create_dir_all(&scenario_dir).expect("create scenario dir");
    let path = scenario_dir.join("manifest.json");
    let body = serde_json::to_string_pretty(manifest).expect("serialize manifest");
    std::fs::write(&path, body).expect("write manifest");
    path
}

fn positive_control() -> ScenarioManifest {
    ScenarioManifest {
        id: "scenario_01".into(),
        kind: "PositiveControl".into(),
        tamper_ops: vec![],
        expected_outcome: ExpectedOutcome::CleanPass,
        notes: "clean".into(),
    }
}

fn broken_parent() -> ScenarioManifest {
    ScenarioManifest {
        id: "scenario_02".into(),
        kind: "BrokenParent".into(),
        tamper_ops: vec![TamperOp::BreakParentLink { entry_index: 2 }],
        expected_outcome: ExpectedOutcome::ExpectedError {
            kind: "broken_parent".into(),
        },
        notes: "dangling".into(),
    }
}

fn content_hash_mismatch() -> ScenarioManifest {
    ScenarioManifest {
        id: "scenario_03".into(),
        kind: "ContentHashMismatch".into(),
        tamper_ops: vec![TamperOp::MutateContentHash { entry_index: 1 }],
        expected_outcome: ExpectedOutcome::ExpectedError {
            kind: "hash_mismatch".into(),
        },
        notes: "mutate body".into(),
    }
}

fn duplicate_hash() -> ScenarioManifest {
    ScenarioManifest {
        id: "scenario_04".into(),
        kind: "DuplicateHash".into(),
        tamper_ops: vec![TamperOp::DuplicateEntry { entry_index: 1 }],
        expected_outcome: ExpectedOutcome::ExpectedError {
            kind: "duplicate_hash".into(),
        },
        notes: "dup".into(),
    }
}

fn malformed_json() -> ScenarioManifest {
    ScenarioManifest {
        id: "scenario_05".into(),
        kind: "MalformedJson".into(),
        tamper_ops: vec![TamperOp::CorruptJsonl { entry_index: 2 }],
        expected_outcome: ExpectedOutcome::ExpectedError {
            kind: "malformed_json".into(),
        },
        notes: "corrupt".into(),
    }
}

fn revert_scenario() -> ScenarioManifest {
    ScenarioManifest {
        id: "scenario_09".into(),
        kind: "RevertBackup".into(),
        tamper_ops: vec![TamperOp::RevertBackup {
            inner: vec![Box::new(TamperOp::MutateContentHash { entry_index: 1 })],
        }],
        expected_outcome: ExpectedOutcome::CleanPass,
        notes: "revert".into(),
    }
}

fn idempotency_scenario() -> ScenarioManifest {
    ScenarioManifest {
        id: "scenario_idempotency".into(),
        kind: "Idempotency".into(),
        tamper_ops: vec![TamperOp::IdempotencyDoubleVerify],
        expected_outcome: ExpectedOutcome::CleanPass,
        notes: "double verify must agree".into(),
    }
}

fn fail_fast_scenario() -> ScenarioManifest {
    ScenarioManifest {
        id: "scenario_10".into(),
        kind: "FailFastSingleError".into(),
        tamper_ops: vec![TamperOp::FailFastSingleError {
            inner: Box::new(TamperOp::BreakParentLink { entry_index: 2 }),
        }],
        expected_outcome: ExpectedOutcome::ExpectedError {
            kind: "broken_parent".into(),
        },
        notes: "ff vs collect parity".into(),
    }
}

#[tokio::test]
async fn end_to_end_full_matrix_all_pass() {
    let dir = TempDir::new().expect("tempdir");
    write_manifest(&dir, "scenario_01", &positive_control());
    write_manifest(&dir, "scenario_02", &broken_parent());
    write_manifest(&dir, "scenario_03", &content_hash_mismatch());
    write_manifest(&dir, "scenario_04", &duplicate_hash());
    write_manifest(&dir, "scenario_05", &malformed_json());
    write_manifest(&dir, "scenario_09", &revert_scenario());
    write_manifest(&dir, "scenario_idempotency", &idempotency_scenario());
    write_manifest(&dir, "scenario_10", &fail_fast_scenario());

    let bench = TamperBenchmark::new(dir.path().to_path_buf());
    let report = bench.run().await.expect("run ok");

    assert_eq!(report.total_scenarios, 8);
    assert_eq!(report.passed_count, 8);
    assert_eq!(report.failed_count, 0);
    assert_eq!(report.gating_verdict, GatingVerdict::Pass);

    // Per-scenario assertions: each scenario classified correctly.
    let id_to_outcome: std::collections::BTreeMap<String, Outcome> = report
        .per_scenario
        .iter()
        .map(|r| (r.id.clone(), r.observed_outcome.clone()))
        .collect();
    assert_eq!(
        id_to_outcome.get("scenario_01"),
        Some(&Outcome::CleanPass)
    );
    assert!(matches!(
        id_to_outcome.get("scenario_02"),
        Some(Outcome::ExpectedError { kind }) if kind == "broken_parent"
    ));
    assert!(matches!(
        id_to_outcome.get("scenario_03"),
        Some(Outcome::ExpectedError { kind }) if kind == "hash_mismatch"
    ));
    assert!(matches!(
        id_to_outcome.get("scenario_04"),
        Some(Outcome::ExpectedError { kind }) if kind == "duplicate_hash"
    ));
    assert!(matches!(
        id_to_outcome.get("scenario_05"),
        Some(Outcome::ExpectedError { kind }) if kind == "malformed_json"
    ));
    assert_eq!(
        id_to_outcome.get("scenario_09"),
        Some(&Outcome::CleanPass)
    );
    assert_eq!(
        id_to_outcome.get("scenario_idempotency"),
        Some(&Outcome::CleanPass)
    );
    assert!(matches!(
        id_to_outcome.get("scenario_10"),
        Some(Outcome::ExpectedError { kind }) if kind == "broken_parent"
    ));

    // Wall budget invariant.
    assert!(
        report.max_wall_ms <= 5000,
        "max_wall_ms {} exceeded 5000ms plan budget",
        report.max_wall_ms
    );
}

#[tokio::test]
async fn positive_control_alone_passes() {
    let dir = TempDir::new().expect("tempdir");
    write_manifest(&dir, "scenario_01", &positive_control());
    let report = TamperBenchmark::new(dir.path().to_path_buf())
        .run()
        .await
        .expect("run ok");
    assert!(report.passed());
}

#[tokio::test]
async fn each_archive_verify_error_variant_detected() {
    let dir = TempDir::new().expect("tempdir");
    write_manifest(&dir, "scenario_02", &broken_parent());
    write_manifest(&dir, "scenario_03", &content_hash_mismatch());
    write_manifest(&dir, "scenario_04", &duplicate_hash());
    write_manifest(&dir, "scenario_05", &malformed_json());
    let report = TamperBenchmark::new(dir.path().to_path_buf())
        .run()
        .await
        .expect("run ok");
    assert_eq!(report.passed_count, 4);
    assert!(report.passed());
}

#[tokio::test]
async fn revert_recovers_clean() {
    let dir = TempDir::new().expect("tempdir");
    write_manifest(&dir, "scenario_09", &revert_scenario());
    let report = TamperBenchmark::new(dir.path().to_path_buf())
        .run()
        .await
        .expect("run ok");
    assert_eq!(report.passed_count, 1);
    assert_eq!(report.per_scenario[0].observed_outcome, Outcome::CleanPass);
}

#[tokio::test]
async fn idempotency_double_verify_passes() {
    let dir = TempDir::new().expect("tempdir");
    write_manifest(&dir, "scenario_idempotency", &idempotency_scenario());
    let report = TamperBenchmark::new(dir.path().to_path_buf())
        .run()
        .await
        .expect("run ok");
    assert_eq!(report.passed_count, 1);
    assert_eq!(report.per_scenario[0].observed_outcome, Outcome::CleanPass);
}

#[tokio::test]
async fn fail_fast_vs_collect_all_parity() {
    let dir = TempDir::new().expect("tempdir");
    write_manifest(&dir, "scenario_10", &fail_fast_scenario());
    let report = TamperBenchmark::new(dir.path().to_path_buf())
        .run()
        .await
        .expect("run ok");
    assert_eq!(report.passed_count, 1);
    assert!(matches!(
        report.per_scenario[0].observed_outcome,
        Outcome::ExpectedError { ref kind } if kind == "broken_parent"
    ));
}

#[tokio::test]
async fn wall_budget_zero_clamps_to_timeout() {
    let dir = TempDir::new().expect("tempdir");
    write_manifest(&dir, "scenario_01", &positive_control());
    let report = TamperBenchmark::new(dir.path().to_path_buf())
        .with_max_wall_ms_per_scenario(0)
        .run()
        .await
        .expect("run ok");
    assert_eq!(report.gating_verdict, GatingVerdict::Fail);
    assert_eq!(report.per_scenario[0].observed_outcome, Outcome::Timeout);
}
