//! E-13 — `cargo-mutants 27.0.0` subprocess wrapper.
//!
//! Composes argv for a targeted-files cargo-mutants invocation and
//! shells out via `tokio::process::Command`. The wrapper is a SIDE
//! channel — D-3's verification surface aggregates the mutation
//! result; the orchestrator's existing
//! [`crate::mutation_testing::RustMutationRunner`] (shipped at
//! C-28) remains the canonical runner. This module exists ONLY for
//! the TDD agent's per-cycle "did the latest change kill the
//! mutant I introduced" probe.
//!
//! Plan-vs-reality correction (per re-check report DRIFT entry):
//! the plan §2 calls for `cargo mutants --shard <touched_files>` —
//! but `cargo-mutants 27.0` does NOT document a `--shard <files>`
//! flag. The canonical CLI accepts `--file <PATH>` repeated for
//! per-file scoping (per cargo-mutants 27.0 README on
//! github.com/sourcefrog/cargo-mutants). This module emits the
//! correct argv list.
//!
//! Toolchain gating: when the `cargo-mutants` binary is absent from
//! PATH, [`run_cargo_mutants`] returns
//! [`MutatorError::ToolchainMissing`] (wire-dark posture per E-12
//! Tier-B and C-52 dafny precedents — the absence is the
//! `RUNPOD-PENDING` signal, not a hard failure).

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;
use tokio::time::timeout;

/// Default timeout (seconds) for one cargo-mutants invocation.
/// The plan's per-CONCEPT budget is 300s; per-STEP is 60s. The
/// default here splits the difference and individual callers can
/// override via [`MutatorConfig::timeout_secs`].
pub const MUTANTS_DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Canonical name of the cargo-mutants binary on PATH. cargo-mutants
/// ships as `cargo-mutants` (with the `cargo-` prefix that lets
/// `cargo mutants` route to it).
pub const CARGO_MUTANTS_BIN: &str = "cargo-mutants";

/// Pinned major version per E-13 plan §12 + C-28 baseline. The
/// wrapper does NOT enforce the pin at runtime (calling
/// `cargo-mutants --version` adds a round-trip cost); the
/// `elai_cc.toml::[testing.mutation_targeted].cargo_mutants_pin`
/// stanza carries the authoritative pin and `elai doctor` verifies
/// it once per session.
pub const CARGO_MUTANTS_PINNED_MAJOR: u32 = 27;

/// Per-invocation configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutatorConfig {
    /// Absolute or workspace-relative paths to the source files
    /// under test. Each path becomes a `--file <PATH>` argv pair.
    /// MUST be non-empty; an empty list is rejected at argv-build
    /// time.
    pub files: Vec<PathBuf>,
    /// Optional working directory the subprocess runs in. `None`
    /// defaults to the orchestrator's CWD.
    pub working_dir: Option<PathBuf>,
    /// Per-invocation timeout. Defaults to
    /// [`MUTANTS_DEFAULT_TIMEOUT_SECS`] when `None`.
    pub timeout_secs: Option<u64>,
}

/// Closed-enum outcome for a single cargo-mutants invocation.
///
/// `AllCaught` -- every mutant was killed by an existing test
/// (mutation_score = 100). Strongest signal.
/// `SomeSurvived` -- at least one mutant survived; `surviving_count`
/// carries the count and `mutation_score` is the kill-rate in [0,1].
/// `NoMutantsProduced` -- cargo-mutants produced zero mutants for
/// the file set (typical when the files are configs or have no
/// mutable expressions). Treated as inconclusive by D-3.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MutationOutcome {
    AllCaught { mutant_count: u32 },
    SomeSurvived {
        surviving_count: u32,
        mutant_count: u32,
        mutation_score: f32,
    },
    NoMutantsProduced,
}

impl MutationOutcome {
    /// True when the invocation supports a non-LLM "pass" verifier
    /// signal for D-3 aggregation. `AllCaught` is the only
    /// pass-signal variant.
    #[must_use]
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::AllCaught { .. })
    }

    /// Kill-rate in `[0.0, 1.0]`. `None` for `NoMutantsProduced`.
    #[must_use]
    pub fn mutation_score(&self) -> Option<f32> {
        match self {
            Self::AllCaught { .. } => Some(1.0),
            Self::SomeSurvived { mutation_score, .. } => Some(*mutation_score),
            Self::NoMutantsProduced => None,
        }
    }
}

/// Errors surfaced by the cargo-mutants wrapper.
#[derive(Debug, Error)]
pub enum MutatorError {
    /// `cargo-mutants` binary is not on PATH. Wire-dark posture:
    /// the caller treats this as `RUNPOD-PENDING` rather than a
    /// hard failure (the rest of the TDD cycle still emits the
    /// `TddTestPassed` beacon when example tests flip green).
    #[error("cargo-mutants binary not found on PATH (RUNPOD-PENDING)")]
    ToolchainMissing,
    /// Empty `files` list passed to argv builder. Argv-build is the
    /// only error surface that is structurally impossible at runtime
    /// once the caller has at least one touched file.
    #[error("cargo-mutants requires at least one --file argument")]
    EmptyFileList,
    /// Subprocess exceeded the per-invocation timeout.
    #[error("cargo-mutants timed out after {0}s")]
    Timeout(u64),
    /// I/O or spawn error reported by `tokio::process::Command`.
    #[error("cargo-mutants subprocess error: {0}")]
    Process(String),
    /// Non-zero exit from cargo-mutants. `code` is `None` when the
    /// subprocess was killed by a signal.
    #[error("cargo-mutants exited with status {code:?} (stderr: {stderr})")]
    ExitNonZero { code: Option<i32>, stderr: String },
}

/// Build the canonical argv vector for a `cargo-mutants` invocation.
///
/// Pure function — no I/O, no subprocess spawn. Used by the
/// wrapper [`run_cargo_mutants`] and by tests that lock the argv
/// shape (contract D3 in
/// `crates/contracts/tests/e13_tdd_verification_contract.rs`).
///
/// Layout: `cargo-mutants --in-place --baseline=skip --no-shuffle
/// --file <P1> --file <P2> ...`. Order of `--file` entries follows
/// the input `files` slice (deterministic for caller-visible
/// reproducibility).
pub fn cargo_mutants_argv(files: &[PathBuf]) -> Result<Vec<String>, MutatorError> {
    if files.is_empty() {
        return Err(MutatorError::EmptyFileList);
    }
    let mut argv: Vec<String> = vec![
        "--in-place".into(),
        "--baseline=skip".into(),
        "--no-shuffle".into(),
    ];
    for path in files {
        argv.push("--file".into());
        argv.push(path.display().to_string());
    }
    Ok(argv)
}

/// Run `cargo-mutants` with the supplied config and return the
/// parsed [`MutationOutcome`].
///
/// Failure modes:
///   - `ToolchainMissing` if the binary is not on PATH.
///   - `EmptyFileList` if the config has no files.
///   - `Timeout` if the subprocess exceeds the configured deadline.
///   - `Process` for spawn / wait errors.
///   - `ExitNonZero` for non-zero exit codes (some-survived is
///     SIGNAL, not error — cargo-mutants exits zero on
///     all-caught/some-survived/no-mutants alike).
///
/// Output parsing is intentionally LIGHT — the wrapper looks for
/// "mutants tested: N" + "N missed" patterns in stdout and falls
/// back to `NoMutantsProduced` when neither line is present. The
/// canonical [`crate::mutation_testing::RustMutationRunner`] does
/// richer JSON parsing; this module's role is the targeted-files
/// probe, not the rich report.
pub async fn run_cargo_mutants(cfg: &MutatorConfig) -> Result<MutationOutcome, MutatorError> {
    let argv = cargo_mutants_argv(&cfg.files)?;
    let timeout_secs = cfg.timeout_secs.unwrap_or(MUTANTS_DEFAULT_TIMEOUT_SECS);

    let mut cmd = Command::new(CARGO_MUTANTS_BIN);
    cmd.args(&argv);
    if let Some(wd) = cfg.working_dir.as_deref() {
        cmd.current_dir(wd);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(MutatorError::ToolchainMissing);
        }
        Err(e) => return Err(MutatorError::Process(e.to_string())),
    };

    let wait_result = timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await;
    let output = match wait_result {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => return Err(MutatorError::Process(e.to_string())),
        Err(_) => return Err(MutatorError::Timeout(timeout_secs)),
    };

    if !output.status.success() {
        return Err(MutatorError::ExitNonZero {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_cargo_mutants_stdout(&stdout))
}

/// Parse a cargo-mutants stdout transcript into a coarse
/// [`MutationOutcome`]. Public for the contract test; not part of
/// the stable surface beyond that.
#[must_use]
pub fn parse_cargo_mutants_stdout(stdout: &str) -> MutationOutcome {
    // cargo-mutants 27.0 final summary lines look like:
    //   "12 mutants tested in 5.4s: 12 caught"
    //   "12 mutants tested in 5.4s: 10 caught, 2 missed"
    //   "0 mutants tested in 0.1s"
    let mut total: Option<u32> = None;
    let mut missed: Option<u32> = None;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
            // Pull the leading number using a char-walk that does NOT
            // panic on weird input.
            let digit_end = trimmed.len() - rest.len();
            if let Ok(n) = trimmed[..digit_end].parse::<u32>() {
                if trimmed.contains("mutants tested") && total.is_none() {
                    total = Some(n);
                }
            }
        }
        if trimmed.contains("missed") {
            for tok in trimmed.split(|c: char| !c.is_ascii_digit()).filter(|s| !s.is_empty()) {
                if let Ok(n) = tok.parse::<u32>() {
                    missed = Some(n);
                    break;
                }
            }
        }
    }

    let total = total.unwrap_or(0);
    if total == 0 {
        return MutationOutcome::NoMutantsProduced;
    }
    let missed = missed.unwrap_or(0);
    if missed == 0 {
        MutationOutcome::AllCaught { mutant_count: total }
    } else {
        let killed = total.saturating_sub(missed);
        let score = if total > 0 {
            killed as f32 / total as f32
        } else {
            0.0
        };
        MutationOutcome::SomeSurvived {
            surviving_count: missed,
            mutant_count: total,
            mutation_score: score,
        }
    }
}

/// Convenience: build a [`MutatorConfig`] over a single file with
/// the default timeout. Used by the agent driver.
#[must_use]
pub fn single_file_config(file: impl Into<PathBuf>) -> MutatorConfig {
    MutatorConfig {
        files: vec![file.into()],
        working_dir: None,
        timeout_secs: None,
    }
}

/// True if `path` looks like a Rust source file we can mutate.
/// Used by the agent to filter touched-files before invoking the
/// wrapper. Pure / no I/O.
#[must_use]
pub fn is_rust_source(path: &Path) -> bool {
    matches!(path.extension().and_then(|e| e.to_str()), Some("rs"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn argv_uses_file_flag_not_shard() {
        let files = vec![PathBuf::from("crates/orchestrator/src/lib.rs")];
        let argv = cargo_mutants_argv(&files).expect("argv");
        assert!(argv.iter().any(|a| a == "--file"));
        assert!(!argv.iter().any(|a| a == "--shard"));
        // The shape must include the in-place + baseline flags.
        assert!(argv.iter().any(|a| a == "--in-place"));
        assert!(argv.iter().any(|a| a == "--baseline=skip"));
    }

    #[test]
    fn argv_repeats_file_flag_per_path() {
        let files = vec![
            PathBuf::from("a.rs"),
            PathBuf::from("b.rs"),
            PathBuf::from("c.rs"),
        ];
        let argv = cargo_mutants_argv(&files).expect("argv");
        let file_count = argv.iter().filter(|a| *a == "--file").count();
        assert_eq!(file_count, 3);
    }

    #[test]
    fn argv_empty_files_errors() {
        let err = cargo_mutants_argv(&[]).expect_err("empty files must error");
        assert!(matches!(err, MutatorError::EmptyFileList));
    }

    #[test]
    fn parse_all_caught_stdout() {
        let stdout = "build successful\n12 mutants tested in 5.4s: 12 caught\n";
        match parse_cargo_mutants_stdout(stdout) {
            MutationOutcome::AllCaught { mutant_count } => assert_eq!(mutant_count, 12),
            other => panic!("expected AllCaught, got {other:?}"),
        }
    }

    #[test]
    fn parse_some_survived_stdout() {
        let stdout = "12 mutants tested in 5.4s: 10 caught, 2 missed\n";
        match parse_cargo_mutants_stdout(stdout) {
            MutationOutcome::SomeSurvived {
                surviving_count,
                mutant_count,
                mutation_score,
            } => {
                assert_eq!(surviving_count, 2);
                assert_eq!(mutant_count, 12);
                assert!((mutation_score - (10.0 / 12.0)).abs() < 0.001);
            }
            other => panic!("expected SomeSurvived, got {other:?}"),
        }
    }

    #[test]
    fn parse_no_mutants_stdout() {
        let stdout = "0 mutants tested in 0.1s\n";
        assert!(matches!(
            parse_cargo_mutants_stdout(stdout),
            MutationOutcome::NoMutantsProduced
        ));
    }

    #[test]
    fn is_rust_source_filters() {
        assert!(is_rust_source(Path::new("foo.rs")));
        assert!(!is_rust_source(Path::new("foo.toml")));
        assert!(!is_rust_source(Path::new("foo")));
    }

    #[test]
    fn outcome_is_pass_only_for_all_caught() {
        assert!(MutationOutcome::AllCaught { mutant_count: 4 }.is_pass());
        assert!(!MutationOutcome::SomeSurvived {
            surviving_count: 1,
            mutant_count: 4,
            mutation_score: 0.75,
        }
        .is_pass());
        assert!(!MutationOutcome::NoMutantsProduced.is_pass());
    }
}
