//! J-26 evaluator ABI contract test (S6) -- orchestrator-crate mirror.
//!
//! The canonical test lives in crates/contracts/tests/j26_evaluator_abi_contract.rs
//! because the contracts crate already has both `elai-orchestrator` and
//! `elai-eval-corpus` as dev-deps. This file is a thin re-export shim so
//! `cargo test -p elai-orchestrator --test j26_evaluator_abi_contract` also
//! surfaces the contract (the orchestrator crate does NOT depend on
//! elai-eval-corpus, so the full contract assertions live in the contracts
//! crate). The smoke test below exercises ONLY the evaluator_abi surface
//! (no elai-eval-corpus dependency) so it compiles cleanly here.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use elai_orchestrator::evaluator_abi::{
    results_to_execution_fitness, run_evaluator, EvaluatorInvocation, ResultsJson, TaskPack,
};

fn golden_pack_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .join("..")
        .join("eval-corpus")
        .join("fixtures")
        .join("j26-golden-pack")
        .canonicalize()
        .expect("j26-golden-pack fixture must exist")
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

fn copy_pack_into(tmp: &Path) -> PathBuf {
    let src = golden_pack_root();
    let dst = tmp.join("pack");
    copy_dir_all(&src, &dst);
    dst
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
    assert!(pack.evaluator.is_file(), "evaluator must exist in golden pack");

    let gen_dir = tmp.path().join("gen");
    fs::create_dir_all(&gen_dir).expect("mkdir gen-dir");

    let invocation = EvaluatorInvocation::for_gen_dir(&pack, &gen_dir);
    let results = run_evaluator(&invocation).expect("golden evaluator must succeed");

    let golden_fitness = 0.87f64;
    assert_eq!(results.primary_scalar(), Some(golden_fitness));
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

    assert!(gen_dir.join("results.json").is_file());
}

/// B) R2 NEGATIVE: a no-op evaluator (exit 0, write nothing) must yield
/// AbiError::ResultsJsonMissing, never a default-zero ExecutionFitness.
#[test]
fn j26_r2_negative_missing_results_json_is_hard_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pack_root = copy_pack_into(tmp.path());

    // TaskPack::read_from picks the FIRST non-task.md file under data/public/
    // via read_dir (filesystem-dependent order). The golden pack ships
    // evaluate.sh, evaluate.py, and task.toml; overwriting only evaluate.sh
    // leaves evaluate.py / task.toml as candidate evaluators, so read_from
    // may select one of those instead and the test would flake. Delete every
    // non-task.md file first so the only candidate is the no-op evaluator we
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
    assert!(!gen_dir.join("results.json").exists());
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

/// B) R2 NEGATIVE (variant 3): results.json with no top-level scalar score
/// field -> NoScalarScore, never a default score.
#[test]
fn j26_r2_negative_no_scalar_score_is_hard_fail() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let gen_dir = tmp.path().join("gen");
    fs::create_dir_all(&gen_dir).expect("mkdir gen-dir");

    fs::write(
        gen_dir.join("results.json"),
        r#"{"meta": "no score here", "nested": {"x": 1}}"#,
    )
    .expect("write empty results.json");

    let path = gen_dir.join("results.json");
    let read = ResultsJson::read_from(&path).expect("read succeeds");
    assert!(read.primary_scalar().is_none());

    let empty = ResultsJson::default();
    let conv = results_to_execution_fitness(&empty, 0, [0u8; 32]);
    assert!(conv.is_err());
}

/// Sanity: the golden pack's evaluate.sh is runnable directly via
/// Command::new (catches a broken shebang / missing +x bit that the ABI
/// test would surface as SpawnFailed rather than the real evaluator
/// behavior).
#[test]
fn j26_golden_evaluator_script_is_runnable_directly() {
    let pack_root = golden_pack_root();
    let evaluator = pack_root.join("data").join("public").join("evaluate.sh");
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
