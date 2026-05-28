//! E-8 — `elai optimize <template|status>` handler.
//!
//! Spawns the Python `elai_optimizer` service (GEPA primary per
//! arXiv:2507.19457; DSPy MIPROv2 / OPRO supporting), tails its
//! progress + emits the per-run report. Live LLM calls are
//! RUNPOD-PENDING -- the CLI surface compiles + dispatches; the
//! Python side reads the spec + emits an OptimizedTemplate JSON to
//! disk; the orchestrator admit path consumes the JSON.
//!
//! Reflection-LM refusal: arXiv:2405.10276 (Zhang et al.) -- ELAI
//! local Qwen2.5-Coder-7B (and other <30B local-class models) are
//! INEFFECTIVE as the OPRO optimizer. The args parser hard-refuses
//! before this handler runs.
//!
//! Per H-7 / CLAUDE.md hard rules: no `.unwrap()` / `.expect()`
//! outside test code.

use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

use crate::args::{CliOutputFormat, OptimizeAction};

/// Top-level dispatcher invoked by `main.rs`.
pub(crate) fn run(
    action: OptimizeAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        OptimizeAction::Template {
            target,
            train,
            val,
            reflection_model,
            budget_tokens,
            include_cross_domain,
        } => run_template_dispatch(
            &target,
            &train,
            &val,
            &reflection_model,
            budget_tokens,
            include_cross_domain,
            output_format,
        ),
        OptimizeAction::Status => run_status(output_format),
    }
}

fn project_root_guess() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn python_optimizer_dir(project_root: &PathBuf) -> PathBuf {
    project_root.join(".elai_cc").join("python").join("elai_optimizer")
}

#[allow(clippy::too_many_arguments)]
fn run_template_dispatch(
    target: &str,
    train: &str,
    val: &str,
    reflection_model: &str,
    budget_tokens: u64,
    include_cross_domain: bool,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let project_root = project_root_guess();
    let py_dir = python_optimizer_dir(&project_root);
    // RUNPOD-PENDING: live cloud Sonnet + local Qwen calls happen
    // inside the Python optimizer. If the python directory is absent
    // (developer is on a fresh checkout without the optimizer
    // installed) we report a clean error rather than panicking.
    let py_present = py_dir.exists();
    let report = OptimizeRunReport {
        action: "template",
        target,
        train,
        val,
        reflection_model,
        budget_tokens,
        include_cross_domain,
        python_dir: py_dir.to_string_lossy().to_string(),
        python_present: py_present,
        // Live LLM calls live in the Python side; this CLI handler
        // only dispatches + reports. The exit-1 path triggers on
        // python-dir absent OR on Python invocation failure.
        runpod_pending: "live_gepa_reflection_loop",
    };
    if !py_present {
        match output_format {
            CliOutputFormat::Json => print_json(&report)?,
            _ => print_template_text(&report),
        }
        return Err(format!(
            "python optimizer dir absent at {}; install via `pip install -e .` inside that dir first",
            py_dir.display()
        ));
    }
    // Attempt to invoke `python -m elai_optimizer.optimizer ...`. The
    // Python side parses --train/--val/--target itself + writes the
    // OptimizedTemplate JSON envelope to
    // .elai_cc/optimization/outputs/. We pass --dry-run when no
    // --include-cross-domain is set so a smoke run does not spend
    // the cloud budget.
    let py_module = "elai_optimizer.optimizer";
    let mut cmd = Command::new("python");
    cmd.current_dir(&project_root)
        .arg("-m")
        .arg(py_module)
        .arg("--target")
        .arg(target)
        .arg("--train")
        .arg(train)
        .arg("--val")
        .arg(val)
        .arg("--reflection-model")
        .arg(reflection_model)
        .arg("--budget-tokens")
        .arg(budget_tokens.to_string());
    if include_cross_domain {
        cmd.arg("--include-cross-domain");
    } else {
        cmd.arg("--dry-run");
    }
    // Spawn + wait; surface exit code as a String error.
    let status = cmd
        .status()
        .map_err(|e| format!("python optimizer spawn: {e}"))?;
    if !status.success() {
        return Err(format!(
            "python optimizer exited with status {:?}",
            status.code()
        ));
    }
    match output_format {
        CliOutputFormat::Json => print_json(&report)?,
        _ => print_template_text(&report),
    }
    Ok(())
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let project_root = project_root_guess();
    let py_dir = python_optimizer_dir(&project_root);
    let py_present = py_dir.exists();
    let report = OptimizeStatusReport {
        action: "status",
        python_dir: py_dir.to_string_lossy().to_string(),
        python_present: py_present,
        gepa_pypi_version_pinned: "0.1.x",
        cloud_reflection_only: "rejects qwen / local / 3B / 7B per arXiv:2405.10276",
        runpod_pending: "live_gepa_reflection_loop",
    };
    match output_format {
        CliOutputFormat::Json => print_json(&report)?,
        _ => print_status_text(&report),
    }
    Ok(())
}

fn print_json<T: Serialize>(report: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("optimize json encode: {e}"))?;
    println!("{json}");
    Ok(())
}

fn print_template_text(r: &OptimizeRunReport<'_>) {
    println!("optimize template ({})", r.target);
    println!("  train               : {}", r.train);
    println!("  val                 : {}", r.val);
    println!("  reflection_model    : {}", r.reflection_model);
    println!("  budget_tokens       : {}", r.budget_tokens);
    println!("  include_cross_domain: {}", r.include_cross_domain);
    println!("  python_dir          : {}", r.python_dir);
    println!("  python_present      : {}", r.python_present);
    println!("  runpod_pending      : {}", r.runpod_pending);
}

fn print_status_text(s: &OptimizeStatusReport<'_>) {
    println!("optimize status (E-8 lean ship)");
    println!("  python_dir              : {}", s.python_dir);
    println!("  python_present          : {}", s.python_present);
    println!("  gepa_pypi_version_pinned: {}", s.gepa_pypi_version_pinned);
    println!("  cloud_reflection_only   : {}", s.cloud_reflection_only);
    println!("  runpod_pending          : {}", s.runpod_pending);
}

#[derive(Serialize)]
struct OptimizeRunReport<'a> {
    action: &'a str,
    target: &'a str,
    train: &'a str,
    val: &'a str,
    reflection_model: &'a str,
    budget_tokens: u64,
    include_cross_domain: bool,
    python_dir: String,
    python_present: bool,
    runpod_pending: &'a str,
}

#[derive(Serialize)]
struct OptimizeStatusReport<'a> {
    action: &'a str,
    python_dir: String,
    python_present: bool,
    gepa_pypi_version_pinned: &'a str,
    cloud_reflection_only: &'a str,
    runpod_pending: &'a str,
}
