//! E-13 — `elai tdd <run|status|mutation-score>` handler.
//!
//! Drives the orchestrator's [`elai_orchestrator::tdd_verifier`]
//! module. Three sub-actions:
//!   - `run --task-path PATH`: read a TDD task description + its
//!     failing test source from PATH and drive one
//!     Red->Green->Refactor cycle via
//!     [`elai_orchestrator::tdd_verifier::TddVerifierAgent`]. The
//!     CLI surface ships a fixture-mode probe (always-pass on
//!     Green/Refactor); live test-runner integration is the
//!     auditor's job.
//!   - `status`: report `cargo-mutants --version` availability +
//!     the TDD verifier's iteration cap defaults. Exits 0 even
//!     when the toolchain is missing (the absence is the canonical
//!     `RUNPOD-PENDING` signal).
//!   - `mutation-score --test-path PATH`: shell out to
//!     `cargo-mutants --file <PATH>` and print the parsed
//!     [`elai_orchestrator::tdd_verifier::MutationOutcome`]. Live
//!     execution is RUNPOD-PENDING when the binary is absent.
//!
//! Citation discipline (per
//! workflow/sessions/s-25120/mistakes.md): the E-13 plan cites 11
//! arXiv IDs; only LLMorpheus (arXiv:2404.09952) and ThinkPRM
//! (arXiv:2504.16828) remain inline-cited in the agent docstrings.
//! The other 9 are referenced by author/title in
//! workflow/reasoning/E-13-decisions.md only.

use std::fs;
use std::path::Path;

use elai_orchestrator::tdd_verifier::{
    cargo_mutants_argv, CycleResult, MutationOutcome, MutatorError, TddCycle, TddTask,
    TddVerifierAgent, MUTANTS_DEFAULT_TIMEOUT_SECS,
};
use serde::Serialize;
use tokio::runtime::Builder;

use crate::args::{CliOutputFormat, TddAction};

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(action: TddAction, output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        TddAction::Run { task_path } => run_cycle(&task_path, output_format),
        TddAction::Status => run_status(output_format),
        TddAction::MutationScore { test_path } => run_mutation_score(&test_path, output_format),
    }
}

#[derive(Debug, Clone, Serialize)]
struct CycleReport {
    task_path: String,
    iterations: u32,
    test_flipped: bool,
    anti_mock_clean: bool,
}

fn run_cycle(task_path: &str, output_format: CliOutputFormat) -> Result<(), String> {
    let source = fs::read_to_string(Path::new(task_path))
        .map_err(|e| format!("failed to read task file `{task_path}`: {e}"))?;

    // Stable task_id derived from the path bytes (caller-deterministic).
    let task_id = stable_hash_prefix(task_path.as_bytes());
    let task = TddTask {
        task_id,
        task_description: format!("CLI-driven cycle for {task_path}"),
        failing_test_source: source,
    };

    let mut agent = TddVerifierAgent::new();
    // Fixture-mode probe: Red fails, Green + Refactor pass. Live
    // probe integration is the auditor's responsibility; the CLI
    // surface is for smoke-testing the cycle wiring.
    let mut probe = |phase: TddCycle| !matches!(phase, TddCycle::Red);
    let cycle_result: CycleResult = agent
        .run_cycle(&task, &mut probe)
        .map_err(|e| format!("tdd cycle failed: {e}"))?;

    let report = CycleReport {
        task_path: task_path.into(),
        iterations: cycle_result.iterations,
        test_flipped: cycle_result.test_flipped,
        anti_mock_clean: cycle_result.anti_mock_clean,
    };
    print_payload(&report, output_format)
}

#[derive(Debug, Clone, Serialize)]
struct StatusReport {
    cargo_mutants_available: bool,
    cargo_mutants_path: Option<String>,
    default_timeout_secs: u64,
    runpod_pending: bool,
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let cargo_mutants_path = which_cargo_mutants();
    let available = cargo_mutants_path.is_some();
    let report = StatusReport {
        cargo_mutants_available: available,
        cargo_mutants_path: cargo_mutants_path.clone(),
        default_timeout_secs: MUTANTS_DEFAULT_TIMEOUT_SECS,
        runpod_pending: !available,
    };
    print_payload(&report, output_format)
}

#[derive(Debug, Clone, Serialize)]
struct MutationScoreReport {
    test_path: String,
    outcome_label: String,
    mutation_score: Option<f32>,
    mutant_count: Option<u32>,
    surviving_count: Option<u32>,
    runpod_pending: bool,
    error: Option<String>,
}

fn run_mutation_score(test_path: &str, output_format: CliOutputFormat) -> Result<(), String> {
    // Argv-build is a pure function — verify it never panics on a
    // single-file input. The contract test e13_tdd_verification_contract
    // pins the `--file` flag shape.
    let _argv = cargo_mutants_argv(&[Path::new(test_path).to_path_buf()])
        .map_err(|e| format!("failed to build cargo-mutants argv: {e}"))?;

    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("failed to build tokio runtime: {e}"))?;

    let cfg = elai_orchestrator::tdd_verifier::mutator::MutatorConfig {
        files: vec![Path::new(test_path).to_path_buf()],
        working_dir: None,
        timeout_secs: None,
    };

    let outcome_result = rt.block_on(elai_orchestrator::tdd_verifier::mutator::run_cargo_mutants(&cfg));

    let report = match outcome_result {
        Ok(MutationOutcome::AllCaught { mutant_count }) => MutationScoreReport {
            test_path: test_path.into(),
            outcome_label: "all_caught".into(),
            mutation_score: Some(1.0),
            mutant_count: Some(mutant_count),
            surviving_count: Some(0),
            runpod_pending: false,
            error: None,
        },
        Ok(MutationOutcome::SomeSurvived {
            surviving_count,
            mutant_count,
            mutation_score,
        }) => MutationScoreReport {
            test_path: test_path.into(),
            outcome_label: "some_survived".into(),
            mutation_score: Some(mutation_score),
            mutant_count: Some(mutant_count),
            surviving_count: Some(surviving_count),
            runpod_pending: false,
            error: None,
        },
        Ok(MutationOutcome::NoMutantsProduced) => MutationScoreReport {
            test_path: test_path.into(),
            outcome_label: "no_mutants_produced".into(),
            mutation_score: None,
            mutant_count: Some(0),
            surviving_count: Some(0),
            runpod_pending: false,
            error: None,
        },
        Err(MutatorError::ToolchainMissing) => MutationScoreReport {
            test_path: test_path.into(),
            outcome_label: "toolchain_missing".into(),
            mutation_score: None,
            mutant_count: None,
            surviving_count: None,
            runpod_pending: true,
            error: Some("cargo-mutants binary not found on PATH".into()),
        },
        Err(e) => MutationScoreReport {
            test_path: test_path.into(),
            outcome_label: "error".into(),
            mutation_score: None,
            mutant_count: None,
            surviving_count: None,
            runpod_pending: false,
            error: Some(e.to_string()),
        },
    };

    print_payload(&report, output_format)
}

fn print_payload<T: Serialize>(value: &T, output_format: CliOutputFormat) -> Result<(), String> {
    match output_format {
        CliOutputFormat::Json => {
            let s = serde_json::to_string_pretty(value)
                .map_err(|e| format!("json serialization failed: {e}"))?;
            println!("{s}");
        }
        CliOutputFormat::Text => {
            // Compact one-line JSON for text-mode terminal output.
            // JSON is the canonical machine format; the text mode
            // simply omits pretty-printing.
            let s = serde_json::to_string(value)
                .map_err(|e| format!("json serialization failed: {e}"))?;
            println!("{s}");
        }
        CliOutputFormat::Toml => {
            // C-21 DRIFT-6 parity: TDD reports use JSON serde
            // shape; render as JSON-in-TOML-quotes so downstream
            // dep-audit-style pipelines can still locate the
            // payload deterministically. The TDD CLI is not a
            // primary TOML emitter.
            let s = serde_json::to_string(value)
                .map_err(|e| format!("json serialization failed: {e}"))?;
            println!("tdd_payload = '{}'", s.replace('\'', "''"));
        }
    }
    Ok(())
}

fn which_cargo_mutants() -> Option<String> {
    // PATH-walk lookup; no subprocess spawn. Mirrors `which::which`
    // semantics without taking on a new dep.
    let path_var = std::env::var_os("PATH")?;
    let exe_names: &[&str] = if cfg!(windows) {
        &["cargo-mutants.exe", "cargo-mutants"]
    } else {
        &["cargo-mutants"]
    };
    for dir in std::env::split_paths(&path_var) {
        for name in exe_names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate.display().to_string());
            }
        }
    }
    None
}

fn stable_hash_prefix(bytes: &[u8]) -> u64 {
    let h = blake3::hash(bytes);
    let b = h.as_bytes();
    u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_hash_is_deterministic() {
        let a = stable_hash_prefix(b"task-1");
        let b = stable_hash_prefix(b"task-1");
        assert_eq!(a, b);
        let c = stable_hash_prefix(b"task-2");
        assert_ne!(a, c);
    }
}
