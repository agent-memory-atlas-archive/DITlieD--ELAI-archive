//! J-26: sia evaluator ABI + task-pack format.
//!
//! The fitness evaluator for H-43 is invoked as a SUBPROCESS with
//! `--gen-dir <dir>`; it writes `results.json` INTO that dir; exit 0 = it ran;
//! MISSING `results.json` = the evaluation never ran = hard FAIL, never a
//! default score (R1/R2). Top-level scalar fields of `results.json` are the
//! only score channel the loop reads.
//!
//! The ABI is language-agnostic: evaluate.py today, a compiled gate binary
//! tomorrow, zero loop change. The evaluator fileset is R8-FROZEN
//! (gate-manifest), the candidate can never edit it.
//!
//! # Grounding
//!
//! - sia (Hexo Labs) EVALUATION_GUIDE.md + orchestrator.py:635-961,
//!   arXiv:2605.27276 (Hebbar et al. 2026).
//! - ELAI delta over sia: the evaluator fileset is R8-FROZEN (gate-manifest),
//!   the candidate can never edit it; sia has no freeze.
//!
//! # Wiring discipline
//!
//! - blake3 ONLY if hashing (never blake2/sha2; CLAUDE.md).
//! - No `.unwrap()`/`.expect()` in prod paths; use `?` propagation.
//! - No LLM calls in the ABI path (TD-3).
//! - Missing `results.json` = hard FAIL (R1/R2), never a default score.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use crate::execution_fitness::ExecutionFitness;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Failure modes of the evaluator ABI.
#[derive(Debug, Error)]
pub enum AbiError {
    /// A required public piece of the task pack is missing on disk.
    #[error("evaluator_abi: missing required public piece '{piece}' in pack at {pack}")]
    MissingPackPiece { piece: String, pack: String },
    /// The evaluator subprocess exited with a non-zero status.
    #[error("evaluator_abi: evaluator '{evaluator}' exited non-zero (code={code}) for gen-dir {gen_dir}")]
    EvaluatorNonZeroExit { evaluator: String, code: i32, gen_dir: String },
    /// The evaluator exited 0 but `results.json` is absent (R1/R2: hard FAIL).
    #[error("evaluator_abi: results.json missing after evaluator exit 0 in gen-dir {gen_dir} (R1/R2: evaluation never ran)")]
    ResultsJsonMissing { gen_dir: String },
    /// `results.json` could not be read or parsed.
    #[error("evaluator_abi: results.json read/parse failed in {gen_dir}: {source}")]
    ResultsJsonUnreadable {
        gen_dir: String,
        #[source]
        source: std::io::Error,
    },
    /// `results.json` parsed but no top-level scalar score field was found.
    #[error("evaluator_abi: results.json in {gen_dir} has no top-level scalar score field (fitness/pass_rate/score)")]
    NoScalarScore { gen_dir: String },
    /// The evaluator subprocess could not be spawned.
    #[error("evaluator_abi: failed to spawn evaluator '{evaluator}': {source}")]
    SpawnFailed {
        evaluator: String,
        #[source]
        source: std::io::Error,
    },
}

// ---------------------------------------------------------------------------
// TaskPack (on-disk sia layout reader)
// ---------------------------------------------------------------------------

/// On-disk task-pack layout (sia TaskLayout convention).
///
/// Required public pieces:
///   - `data/public/task.md`
///   - `data/public/<evaluator>` (the evaluator script/binary)
///
/// Optional public pieces:
///   - `data/public/<fixtures>/` (fixture directory)
///
/// Private (held-out):
///   - `data/private/` (held-out rows, D-6 mapping)
///
/// Reference:
///   - `reference/` (sample solutions)
#[derive(Debug, Clone)]
pub struct TaskPack {
    /// Root directory of the pack.
    pub root: PathBuf,
    /// `data/public/task.md` path.
    pub task_md: PathBuf,
    /// `data/public/<evaluator>` path (the evaluator executable).
    pub evaluator: PathBuf,
    /// Optional fixtures directory under `data/public/`.
    pub fixtures: Option<PathBuf>,
    /// Optional `data/private/` (held-out).
    pub private: Option<PathBuf>,
    /// Optional `reference/` (sample solutions).
    pub reference: Option<PathBuf>,
}

impl TaskPack {
    /// Read a task pack from `root`, validating required public pieces.
    ///
    /// Missing `data/public/task.md` or a missing evaluator file = `Err`.
    /// Optional directories (`fixtures`, `private`, `reference`) are probed
    /// and set to `None` when absent.
    pub fn read_from(root: &Path) -> Result<Self, AbiError> {
        let root_str = root.display().to_string();
        let public = root.join("data").join("public");
        let task_md = public.join("task.md");
        if !task_md.is_file() {
            return Err(AbiError::MissingPackPiece {
                piece: "data/public/task.md".to_string(),
                pack: root_str,
            });
        }
        // The evaluator is any executable file directly under data/public/
        // that is NOT task.md and NOT a directory. We pick the first match.
        let evaluator = std::fs::read_dir(&public)
            .map_err(|e| AbiError::MissingPackPiece {
                piece: format!("data/public/ (read_dir failed: {e})"),
                pack: root_str.clone(),
            })?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.is_file()
                    && p.file_name().map(|n| n != "task.md").unwrap_or(false)
            });
        let evaluator = match evaluator {
            Some(p) => p,
            None => {
                return Err(AbiError::MissingPackPiece {
                    piece: "data/public/<evaluator>".to_string(),
                    pack: root_str,
                })
            }
        };
        // Optional pieces.
        let fixtures = {
            let f = public.join("fixtures");
            if f.is_dir() { Some(f) } else { None }
        };
        let private = {
            let p = root.join("data").join("private");
            if p.is_dir() { Some(p) } else { None }
        };
        let reference = {
            let r = root.join("reference");
            if r.is_dir() { Some(r) } else { None }
        };
        Ok(Self {
            root: root.to_path_buf(),
            task_md,
            evaluator,
            fixtures,
            private,
            reference,
        })
    }
}

// ---------------------------------------------------------------------------
// EvaluatorInvocation
// ---------------------------------------------------------------------------

/// Parameters for one evaluator subprocess invocation.
#[derive(Debug, Clone)]
pub struct EvaluatorInvocation {
    /// The generation directory passed as `--gen-dir`.
    pub gen_dir: PathBuf,
    /// Path to the evaluator executable.
    pub evaluator: PathBuf,
    /// Expected path of `results.json` inside `gen_dir`.
    pub results_json: PathBuf,
}

impl EvaluatorInvocation {
    /// Build an invocation for `gen_dir` using the pack's evaluator.
    pub fn for_gen_dir(pack: &TaskPack, gen_dir: &Path) -> Self {
        Self {
            gen_dir: gen_dir.to_path_buf(),
            evaluator: pack.evaluator.clone(),
            results_json: gen_dir.join("results.json"),
        }
    }
}

// ---------------------------------------------------------------------------
// ResultsJson (top-level scalar fields only)
// ---------------------------------------------------------------------------

/// The top-level scalar fields read from `results.json`.
///
/// Only top-level scalar (f64 / u64) fields are read; nested objects are
/// ignored. The primary score is the first of `fitness`, `pass_rate`, `score`
/// that is present.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResultsJson {
    /// Primary fitness scalar (preferred).
    pub fitness: Option<f64>,
    /// Pass-rate scalar (secondary).
    pub pass_rate: Option<f64>,
    /// Generic score scalar (tertiary).
    pub score: Option<f64>,
    /// Total tests (if reported).
    pub tests_total: Option<u64>,
    /// Tests passed (if reported).
    pub tests_passed: Option<u64>,
    /// Dual-agreement / consensus mass (if reported).
    pub dual_agreement: Option<f64>,
}

impl ResultsJson {
    /// Read `results.json` from `path`. Missing file = hard FAIL (R1/R2).
    pub fn read_from(path: &Path) -> Result<Self, AbiError> {
        let gen_dir = path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        if !path.is_file() {
            return Err(AbiError::ResultsJsonMissing { gen_dir });
        }
        let bytes = std::fs::read(path).map_err(|e| AbiError::ResultsJsonUnreadable {
            gen_dir: gen_dir.clone(),
            source: e,
        })?;
        let raw: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
            AbiError::ResultsJsonUnreadable {
                gen_dir: gen_dir.clone(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            }
        })?;
        // Read ONLY top-level scalar fields.
        let map = match raw.as_object() {
            Some(m) => m,
            None => {
                return Err(AbiError::ResultsJsonUnreadable {
                    gen_dir: gen_dir.clone(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "results.json top-level is not an object",
                    ),
                })
            }
        };
        let get_f64 = |k: &str| -> Option<f64> {
            map.get(k).and_then(|v| v.as_f64())
        };
        let get_u64 = |k: &str| -> Option<u64> {
            map.get(k).and_then(|v| v.as_u64())
        };
        Ok(Self {
            fitness: get_f64("fitness"),
            pass_rate: get_f64("pass_rate"),
            score: get_f64("score"),
            tests_total: get_u64("tests_total"),
            tests_passed: get_u64("tests_passed"),
            dual_agreement: get_f64("dual_agreement"),
        })
    }

    /// The primary scalar score: prefer `fitness`, then `pass_rate`, then `score`.
    /// Returns `None` when none are present (caller treats as hard FAIL).
    pub fn primary_scalar(&self) -> Option<f64> {
        self.fitness.or(self.pass_rate).or(self.score)
    }
}

// ---------------------------------------------------------------------------
// run_evaluator (subprocess)
// ---------------------------------------------------------------------------

/// Run the evaluator subprocess with `--gen-dir <dir>`, then read
/// `results.json`.
///
/// - Non-zero exit = `Err(AbiError::EvaluatorNonZeroExit)`.
/// - Exit 0 but missing `results.json` = `Err(AbiError::ResultsJsonMissing)`
///   (R1/R2: hard FAIL, never a default score).
/// - Exit 0 + `results.json` present = `Ok(ResultsJson)`.
pub fn run_evaluator(invocation: &EvaluatorInvocation) -> Result<ResultsJson, AbiError> {
    let evaluator_str = invocation.evaluator.display().to_string();
    let gen_dir_str = invocation.gen_dir.display().to_string();

    debug!(
        evaluator = %evaluator_str,
        gen_dir = %gen_dir_str,
        "run_evaluator: spawning subprocess"
    );

    let status = std::process::Command::new(&invocation.evaluator)
        .arg("--gen-dir")
        .arg(&invocation.gen_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(|e| AbiError::SpawnFailed {
            evaluator: evaluator_str.clone(),
            source: e,
        })?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        warn!(
            evaluator = %evaluator_str,
            gen_dir = %gen_dir_str,
            code,
            "run_evaluator: non-zero exit (hard FAIL)"
        );
        return Err(AbiError::EvaluatorNonZeroExit {
            evaluator: evaluator_str,
            code,
            gen_dir: gen_dir_str,
        });
    }

    // Exit 0: results.json MUST be present (R1/R2).
    let results = ResultsJson::read_from(&invocation.results_json)?;
    if results.primary_scalar().is_none() {
        warn!(
            gen_dir = %gen_dir_str,
            "run_evaluator: results.json has no top-level scalar score (hard FAIL)"
        );
        return Err(AbiError::NoScalarScore { gen_dir: gen_dir_str });
    }
    Ok(results)
}

// ---------------------------------------------------------------------------
// results_to_execution_fitness
// ---------------------------------------------------------------------------

/// Map a `ResultsJson` into an `ExecutionFitness` honestly.
///
/// The primary scalar (prefer `fitness`, then `pass_rate`, then `score`)
/// becomes `fitness`. When only one scalar is available, `pass_rate` and
/// `dual_agreement` are set to it as well (honest: we have no decomposition).
/// `tests_total`/`tests_passed` come from the results when available, else 0.
/// `seed` and `test_set_hash` are caller-supplied (the loop owns them).
pub fn results_to_execution_fitness(
    results: &ResultsJson,
    seed: u64,
    test_set_hash: [u8; 32],
) -> Result<ExecutionFitness, AbiError> {
    let primary = results.primary_scalar().ok_or_else(|| AbiError::NoScalarScore {
        gen_dir: String::new(),
    })?;
    let pass_rate = results.pass_rate.unwrap_or(primary);
    let dual_agreement = results.dual_agreement.unwrap_or(primary);
    let tests_total = results.tests_total.unwrap_or(0) as u32;
    let tests_passed = results.tests_passed.unwrap_or(0) as u32;
    Ok(ExecutionFitness {
        pass_rate,
        dual_agreement,
        tests_total,
        tests_passed,
        fitness: primary,
        seed,
        test_set_hash,
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// NEGATIVE: missing `results.json` after evaluator exit 0 = hard FAIL.
    #[test]
    fn missing_results_json_is_hard_fail() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let gen_dir = tmp.path();
        // Write a dummy evaluator that exits 0 but writes nothing.
        let evaluator = gen_dir.join("dummy_evaluator.sh");
        let mut f = std::fs::File::create(&evaluator).expect("create evaluator");
        writeln!(f, "#!/bin/bash").expect("write");
        writeln!(f, "exit 0").expect("write");
        drop(f);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&evaluator).expect("metadata").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&evaluator, perms).expect("chmod");
        }
        let invocation = EvaluatorInvocation {
            gen_dir: gen_dir.to_path_buf(),
            evaluator: evaluator.clone(),
            results_json: gen_dir.join("results.json"),
        };
        let result = run_evaluator(&invocation);
        assert!(matches!(result, Err(AbiError::ResultsJsonMissing { .. })));
    }

    /// POSITIVE: a golden inline evaluator that writes `results.json` succeeds.
    #[test]
    fn golden_evaluator_succeeds() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let gen_dir = tmp.path();
        // Write a golden evaluator that writes results.json with a known scalar.
        let evaluator = gen_dir.join("golden_evaluator.sh");
        let mut f = std::fs::File::create(&evaluator).expect("create evaluator");
        writeln!(f, "#!/bin/bash").expect("write");
        writeln!(f, "cat > \"$2/results.json\" <<'EOF'").expect("write");
        writeln!(f, "{{\"fitness\": 0.85, \"pass_rate\": 0.9, \"tests_total\": 10, \"tests_passed\": 9}}").expect("write");
        writeln!(f, "EOF").expect("write");
        writeln!(f, "exit 0").expect("write");
        drop(f);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&evaluator).expect("metadata").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&evaluator, perms).expect("chmod");
        }
        let invocation = EvaluatorInvocation {
            gen_dir: gen_dir.to_path_buf(),
            evaluator: evaluator.clone(),
            results_json: gen_dir.join("results.json"),
        };
        let results = run_evaluator(&invocation).expect("golden evaluator succeeds");
        assert_eq!(results.fitness, Some(0.85));
        assert_eq!(results.pass_rate, Some(0.9));
        assert_eq!(results.tests_total, Some(10));
        assert_eq!(results.tests_passed, Some(9));
        // Convert to ExecutionFitness.
        let ef = results_to_execution_fitness(&results, 42, [0u8; 32]).expect("conversion");
        assert_eq!(ef.fitness, 0.85);
        assert_eq!(ef.pass_rate, 0.9);
        assert_eq!(ef.tests_total, 10);
        assert_eq!(ef.tests_passed, 9);
        assert_eq!(ef.seed, 42);
    }

    /// NEGATIVE: evaluator exits non-zero = hard FAIL.
    #[test]
    fn nonzero_exit_is_hard_fail() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let gen_dir = tmp.path();
        let evaluator = gen_dir.join("failing_evaluator.sh");
        let mut f = std::fs::File::create(&evaluator).expect("create evaluator");
        writeln!(f, "#!/bin/bash").expect("write");
        writeln!(f, "exit 1").expect("write");
        drop(f);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&evaluator).expect("metadata").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&evaluator, perms).expect("chmod");
        }
        let invocation = EvaluatorInvocation {
            gen_dir: gen_dir.to_path_buf(),
            evaluator,
            results_json: gen_dir.join("results.json"),
        };
        let result = run_evaluator(&invocation);
        assert!(matches!(result, Err(AbiError::EvaluatorNonZeroExit { .. })));
    }

    /// `ResultsJson` reads only top-level scalars, ignores nested objects.
    #[test]
    fn results_json_reads_top_level_scalars_only() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("results.json");
        std::fs::write(
            &path,
            r#"{"fitness": 0.5, "nested": {"x": 99}, "score": 0.7, "tests_total": 5}"#,
        )
        .expect("write");
        let r = ResultsJson::read_from(&path).expect("read");
        assert_eq!(r.fitness, Some(0.5));
        assert_eq!(r.score, Some(0.7));
        assert_eq!(r.tests_total, Some(5));
        // nested.x is NOT read.
    }

    /// `primary_scalar` prefers fitness > pass_rate > score.
    #[test]
    fn primary_scalar_preference_order() {
        let r = ResultsJson {
            fitness: Some(0.1),
            pass_rate: Some(0.2),
            score: Some(0.3),
            ..Default::default()
        };
        assert_eq!(r.primary_scalar(), Some(0.1));
        let r = ResultsJson {
            fitness: None,
            pass_rate: Some(0.2),
            score: Some(0.3),
            ..Default::default()
        };
        assert_eq!(r.primary_scalar(), Some(0.2));
        let r = ResultsJson {
            fitness: None,
            pass_rate: None,
            score: Some(0.3),
            ..Default::default()
        };
        assert_eq!(r.primary_scalar(), Some(0.3));
        let r = ResultsJson::default();
        assert_eq!(r.primary_scalar(), None);
    }

    /// `TaskPack::read_from` validates required public pieces.
    #[test]
    fn task_pack_validates_required_pieces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // Empty pack: missing task.md and evaluator.
        let result = TaskPack::read_from(tmp.path());
        assert!(matches!(result, Err(AbiError::MissingPackPiece { .. })));
        // Create task.md only: still missing evaluator.
        let public = tmp.path().join("data").join("public");
        std::fs::create_dir_all(&public).expect("mkdir");
        std::fs::write(public.join("task.md"), "task").expect("write");
        let result = TaskPack::read_from(tmp.path());
        assert!(matches!(result, Err(AbiError::MissingPackPiece { .. })));
        // Create evaluator: now ok.
        let evaluator = public.join("evaluate.py");
        std::fs::write(&evaluator, "#!/usr/bin/env python3").expect("write");
        let pack = TaskPack::read_from(tmp.path()).expect("pack ok");
        assert_eq!(pack.evaluator, evaluator);
        assert!(pack.task_md.is_file());
    }
}
