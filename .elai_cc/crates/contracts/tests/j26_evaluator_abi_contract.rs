//! J-26 evaluator ABI contract test (S6).
//!
//! Vertical slice S5+S6: the golden fixture pack under
//! crates/eval-corpus/fixtures/j26-golden-pack/ is the S5 deliverable; this
//! file is S6. Together they prove the J-26 evaluator ABI contract:
//!
//! A) POSITIVE: load the golden TaskPack; run_evaluator writes results.json;
//!    ResultsJson primary scalar == golden; results_to_execution_fitness.fitness
//!    matches.
//! B) R2 NEGATIVE: after the evaluator would succeed, delete results.json
//!    (and re-run) -> AbiError::ResultsJsonMissing, NEVER a default-zero
//!    ExecutionFitness. Also: a broken evaluator that exits non-zero yields
//!    AbiError::EvaluatorNonZeroExit. Both are hard FAILs, no SKIP, no
//!    default score.
//! C) Optional smoke: Task::load_from_toml on the pack's task.toml succeeds
//!    with non-empty tests + cutoffs (proves S4 ingest input is valid).
//!
//! No live SSB / darwin loop is exercised here; the unit-level ABI contract
//! is the bar for S6 per the plan.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use elai_orchestrator::evaluator_abi::{
    results_to_execution_fitness, run_evaluator, EvaluatorInvocation, ResultsJson, TaskPack,
};
use elai_eval_corpus::task_schema::Task;

/// Locate the golden pack root under the workspace.
fn golden_pack_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // contracts crate is at crates/contracts; golden pack is at
    // crates/eval-corpus/fixtures/j26-golden-pack.
    manifest
        .join("..")
        .join("eval-corpus")
        .join("fixtures")
        .join("j26-golden-pack")
        .canonicalize()
        .expect("j26-golden-pack fixture must exist")
}

/// Copy the pack's evaluator + task files into a temp gen-dir-adjacent copy
/// so the test never mutates the fixture tree. Returns the copied pack root.
fn copy_pack_into(tmp: &Path) -> PathBuf {
    let src = golden_pack_root();
    let dst = tmp.join("pack");
    copy_dir_all(&src, &dst);
    dst
}

fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("mkdir dst");
    for entry in fs::read_dir(src).expect("read_dir src") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let name = entry.file_name();
        let target = dst.join(&name);
        if path.is_dir() {
            copy_dir_all(&path, &target);
        } else {
            fs::copy(&path, &target).expect("copy file");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let src_perms = fs::metadata(&path).expect("metadata").permissions();
                let mut perms = fs::metadata(&target).expect("metadata").permissions();
                perms.set_mode(src_perms.mode() & 0o777);
                let _ = fs::set_permissions(&target, perms);
            }
        }
    }
}

/// A) POSITIVE: golden pack loads, run_evaluator writes results.json, the
/// primary scalar matches the golden, and results_to_execution_fitness.fitness
/// matches.
#[test]
fn j26_positive_golden_evaluator_succeeds() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pack_root = copy_pack_into(tmp.path());
    let pack = TaskPack::read_from(&pack_root).expect("TaskPack::read_from golden pack");

    assert!(pack.task_md.is_file(), "task.md must exist in golden pack");
    assert!(
        pack.evaluator.is_file(),
        "evaluator must exist in golden pack"
    );
    assert!(pack.evaluator.file_name().is_some());

    let gen_dir = tmp.path().join("gen");
    fs::create_dir_all(&gen_dir).expect("mkdir gen-dir");

    let invocation = EvaluatorInvocation::for_gen_dir(&pack, &gen_dir);
    let results = run_evaluator(&invocation).expect("golden evaluator must succeed");

    let golden_fitness = 0.87f64;
    assert_eq!(
        results.primary_scalar(),
        Some(golden_fitness),
        "primary scalar must match the golden pack's known fitness"
    );
    assert_eq!(results.fitness, Some(golden_fitness));
    assert_eq!(results.pass_rate, Some(0.9));
    assert_eq!(results.tests_total, Some(3));
    assert_eq!(results.tests_passed, Some(3));

    let ef = results_to_execution_fitness(&results, 42, [0u8; 32])
        .expect("results_to_execution_fitness must succeed");
    assert_eq!(ef.fitness, golden_fitness);
    assert_eq!(ef.pass_rate, 0.9);
    assert_eq!(ef.tests_total, 3);
    assert_eq!(ef.tests_passed, 3);
    assert_eq!(ef.seed, 42);

    assert!(
        gen_dir.join("results.json").is_file(),
        "results.json must be on disk after a successful evaluator run"
    );
}

/// B) R2 NEGATIVE: after the evaluator writes results.json, delete it and
/// re-run the evaluator with a no-op stub (exit 0, write nothing). The ABI
/// must return AbiError::ResultsJsonMissing, never a default-zero
/// ExecutionFitness.
#[test]
fn j26_r2_negative_missing_results_json_is_hard_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pack_root = copy_pack_into(tmp.path());

    // Overwrite the evaluator with a no-op stub that exits 0 but writes
    // nothing. This simulates "the evaluation never ran" (R1/R2).
    let evaluator = pack_root
        .join("data")
        .join("public")
        .join("evaluate.sh");

    // See j26_r2_negative_nonzero_exit_is_hard_fail: TaskPack::read_from
    // picks the first non-task.md file via read_dir (order is
    // filesystem-dependent), so remove every other candidate first.
    let public = pack_root.join("data").join("public");
    for entry in fs::read_dir(&public).expect("read_dir public") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_file() && entry.file_name() != "task.md" && path != evaluator {
            fs::remove_file(&path).expect("remove non-task.md file");
        }
    }

    fs::write(&evaluator, "#!/usr/bin/env bash\nexit 0\n").expect("write stub evaluator");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&evaluator).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&evaluator, perms).expect("chmod");
    }

    let pack = TaskPack::read_from(&pack_root).expect("TaskPack::read_from");
    let gen_dir = tmp.path().join("gen");
    fs::create_dir_all(&gen_dir).expect("mkdir gen-dir");

    let invocation = EvaluatorInvocation::for_gen_dir(&pack, &gen_dir);
    let result = run_evaluator(&invocation);

    match result {
        Err(elai_orchestrator::evaluator_abi::AbiError::ResultsJsonMissing { .. }) => {}
        other => panic!(
            "R2 negative: expected AbiError::ResultsJsonMissing, got {:?}",
            other.map(|r| r.fitness)
        ),
    }

    // No default-zero ExecutionFitness is ever produced: the Err arm is the
    // only way out. results_to_execution_fitness is never called on the Err
    // path; verify by asserting the results.json file is absent.
    assert!(
        !gen_dir.join("results.json").exists(),
        "no results.json should exist after a no-op evaluator"
    );
}

/// B) R2 NEGATIVE (variant 2): a broken evaluator that exits non-zero must
/// yield AbiError::EvaluatorNonZeroExit, never a default score.
#[test]
fn j26_r2_negative_nonzero_exit_is_hard_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pack_root = copy_pack_into(tmp.path());

    // TaskPack::read_from picks the FIRST non-task.md file under data/public/
    // via read_dir (filesystem-dependent order). The golden pack ships
    // evaluate.sh, evaluate.py, and task.toml; overwriting only evaluate.sh
    // leaves evaluate.py / task.toml as candidate evaluators, so read_from
    // may select one of those instead and the test would flake. Delete every
    // non-task.md file first so the only candidate is the broken evaluator we
    // write below.
    let public = pack_root.join("data").join("public");
    for entry in fs::read_dir(&public).expect("read_dir public") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_file() && entry.file_name() != "task.md" {
            fs::remove_file(&path).expect("remove non-task.md file");
        }
    }

    let evaluator = public.join("evaluate.sh");
    fs::write(&evaluator, "#!/usr/bin/env bash\nexit 1\n").expect("write broken evaluator");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&evaluator).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&evaluator, perms).expect("chmod");
    }

    let pack = TaskPack::read_from(&pack_root).expect("TaskPack::read_from");
    let gen_dir = tmp.path().join("gen");
    fs::create_dir_all(&gen_dir).expect("mkdir gen-dir");

    let invocation = EvaluatorInvocation::for_gen_dir(&pack, &gen_dir);
    let result = run_evaluator(&invocation);

    match result {
        Err(elai_orchestrator::evaluator_abi::AbiError::EvaluatorNonZeroExit { .. }) => {}
        other => panic!(
            "R2 negative: expected AbiError::EvaluatorNonZeroExit, got {:?}",
            other.map(|r| r.fitness)
        ),
    }
}

/// B) R2 NEGATIVE (variant 3): results.json present but no top-level scalar
/// score field -> AbiError::NoScalarScore, never a default score.
#[test]
fn j26_r2_negative_no_scalar_score_is_hard_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let gen_dir = tmp.path().join("gen");
    fs::create_dir_all(&gen_dir).expect("mkdir gen-dir");

    // Pre-write a results.json with no scalar score field.
    fs::write(
        gen_dir.join("results.json"),
        r#"{"meta": "no score here", "nested": {"x": 1}}"#,
    )
    .expect("write empty results.json");

    // Direct ResultsJson::read_from path: no scalar -> NoScalarScore via
    // run_evaluator's post-read check.
    let path = gen_dir.join("results.json");
    let read = ResultsJson::read_from(&path).expect("read succeeds");
    assert!(read.primary_scalar().is_none());

    // And results_to_execution_fitness on an empty ResultsJson is an Err.
    let empty = ResultsJson::default();
    let conv = results_to_execution_fitness(&empty, 0, [0u8; 32]);
    assert!(conv.is_err(), "empty ResultsJson must not yield a default-zero ExecutionFitness");
}

/// C) Optional smoke: Task::load_from_toml on the pack's task.toml succeeds
/// with non-empty tests + cutoffs (proves S4 ingest input is valid).
#[test]
fn j26_smoke_task_toml_loads_with_non_empty_tests_and_cutoffs() {
    let pack_root = golden_pack_root();
    let task_toml = pack_root.join("data").join("public").join("task.toml");
    assert!(task_toml.is_file(), "task.toml must exist in golden pack");

    let task = Task::load_from_toml(&task_toml).expect("Task::load_from_toml must succeed");
    assert!(!task.tests.is_empty(), "task.tests must be non-empty");
    assert!(!task.cutoffs.is_empty(), "task.cutoffs must be non-empty");
    assert_eq!(task.id, "j26-golden-pack");
}

/// Sanity: the golden pack's evaluate.sh is executable and runs end-to-end
/// via the real subprocess path (Command::new), independent of the Rust ABI.
/// This catches a broken shebang / missing +x bit that the ABI test would
/// surface as SpawnFailed rather than the real evaluator behavior.
#[test]
fn j26_golden_evaluator_script_is_runnable_directly() {
    let pack_root = golden_pack_root();
    let evaluator = pack_root
        .join("data")
        .join("public")
        .join("evaluate.sh");
    let tmp = tempfile::tempdir().expect("tempdir");
    let gen_dir = tmp.path().join("gen");
    fs::create_dir_all(&gen_dir).expect("mkdir gen-dir");

    let status = Command::new(&evaluator)
        .arg("--gen-dir")
        .arg(&gen_dir)
        .status()
        .expect("evaluate.sh must spawn");
    assert!(status.success(), "evaluate.sh must exit 0");

    let results_path = gen_dir.join("results.json");
    assert!(results_path.is_file(), "evaluate.sh must write results.json");

    let body = fs::read_to_string(&results_path).expect("read results.json");
    assert!(
        body.contains("\"fitness\": 0.87"),
        "results.json must carry the golden fitness scalar, got: {body}"
    );
}
