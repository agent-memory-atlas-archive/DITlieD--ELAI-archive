//! E-53 — `elai formal-verify <run|status>` handler.
//!
//! Drives the orchestrator's [`elai_orchestrator::formal_verify`]
//! module. Two sub-actions:
//!   - `run --target PATH --verifier KIND`: dispatch one verifier
//!     (z3_smt | dafny | kani | verus) against PATH and print the
//!     [`PipelineResult`].
//!   - `status`: report each toolchain's RUNPOD-PENDING posture +
//!     the substitute-models list inherited from C-52
//!     ([`elai_orchestrator::dafny_verify::ELAI_SUBSTITUTE_MODELS`]).
//!     Gemma-4 is BLOCKED per memory `feedback_no_gemma4`.
//!
//! WIRE-DARK avoidance per E40_WIRE_DARK_AVOIDANCE_VIA_CLI: this
//! module is the production call anchor for
//! `formal_verify::orchestrator::run` + every variant of
//! `formal_verify::VerifierKind` + `formal_verify::VerifierOutcome`.
//!
//! Per H-7 / H-9: no `.unwrap()` / `.expect()` outside test code.

use elai_orchestrator::dafny_verify::{ELAI_SUBSTITUTE_MODELS, GEMMA4_BLOCKED};
use elai_orchestrator::formal_verify::{
    run as run_pipeline, PipelineResult, VerifierKind, VerifierOutcome, VerifyRequest,
    ALL_VERIFIER_KINDS,
};
use serde::Serialize;
use tokio::runtime::Builder;

use crate::args::{CliOutputFormat, FormalVerifyAction};

/// Top-level dispatcher invoked by `main.rs`.
pub(crate) fn run(
    action: FormalVerifyAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        FormalVerifyAction::Run { target, verifier } => run_pipeline_dispatch(&target, &verifier, output_format),
        FormalVerifyAction::Status => run_status(output_format),
    }
}

fn parse_verifier_kind(s: &str) -> Result<VerifierKind, String> {
    match s {
        "z3_smt" | "z3" => Ok(VerifierKind::Z3Smt),
        "dafny" => Ok(VerifierKind::Dafny),
        "kani" => Ok(VerifierKind::Kani),
        "verus" => Ok(VerifierKind::Verus),
        other => Err(format!(
            "unknown verifier kind `{other}`; expected one of: z3_smt, dafny, kani, verus"
        )),
    }
}

fn run_pipeline_dispatch(
    target: &str,
    verifier_str: &str,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let verifier = parse_verifier_kind(verifier_str)?;
    // Build a current-thread tokio runtime so the sync CLI entry
    // stays sync (mirrors C-52 dafny_cmd::run_verify shape). Only
    // the Dafny route is genuinely async; the Z3 / Kani / Verus
    // stubs are sync-completable but the orchestrator entry is
    // async for shape uniformity.
    let rt = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|e| format!("tokio runtime: {e}"))?;
    let request = VerifyRequest {
        target_path: target.to_string(),
        verifier,
        cost_cap_ms: None,
    };
    let result: PipelineResult = rt.block_on(async { run_pipeline(request).await });
    let report = RunReport {
        action: "run",
        target,
        verifier: verifier.as_str(),
        outcome: result.outcome.as_str(),
        contract_emitted: result.contract_emitted,
        gemma4_blocked: GEMMA4_BLOCKED,
        substitute_models: &ELAI_SUBSTITUTE_MODELS,
        runpod_pending: "live_formal_verifier_toolchains",
    };
    match output_format {
        CliOutputFormat::Json => print_json(&report)?,
        _ => print_run_text(&report, &result.outcome),
    }
    Ok(())
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let verifiers: Vec<VerifierStatusRow> = ALL_VERIFIER_KINDS
        .iter()
        .map(|k| VerifierStatusRow {
            verifier: k.as_str(),
            expected_local_install: k.expected_local_install(),
            runpod_pending: !k.expected_local_install(),
        })
        .collect();
    let status = StatusReport {
        action: "status",
        verifiers,
        gemma4_blocked: GEMMA4_BLOCKED,
        substitute_models: &ELAI_SUBSTITUTE_MODELS,
        runpod_pending: "live_formal_verifier_toolchains",
    };
    match output_format {
        CliOutputFormat::Json => print_json(&status)?,
        _ => print_status_text(&status),
    }
    Ok(())
}

fn print_json<T: Serialize>(report: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("formal-verify json encode: {e}"))?;
    println!("{json}");
    Ok(())
}

fn print_run_text(r: &RunReport<'_>, outcome: &VerifierOutcome) {
    println!("formal-verify run ({})", r.target);
    println!("  verifier         : {}", r.verifier);
    println!("  outcome          : {}", r.outcome);
    println!("  contract_emitted : {}", r.contract_emitted);
    match outcome {
        VerifierOutcome::Refuted {
            counterexample_label,
        } => {
            println!("  counterexample   : {counterexample_label}");
        }
        VerifierOutcome::Inapplicable { reason } => {
            println!("  inapplicable     : {reason}");
        }
        VerifierOutcome::Verified | VerifierOutcome::RunpodPending => {}
    }
    println!("  gemma4_blocked   : {}", r.gemma4_blocked);
    println!("  substitute_models: {:?}", r.substitute_models);
    println!("  runpod_pending   : {}", r.runpod_pending);
}

fn print_status_text(s: &StatusReport<'_>) {
    println!("formal-verify status (E-53 lean ship)");
    for v in &s.verifiers {
        println!(
            "  {:8} : expected_local_install={} runpod_pending={}",
            v.verifier, v.expected_local_install, v.runpod_pending
        );
    }
    println!("  gemma4_blocked   : {}", s.gemma4_blocked);
    println!("  substitute_models: {:?}", s.substitute_models);
    println!("  runpod_pending   : {}", s.runpod_pending);
}

#[derive(Serialize)]
struct RunReport<'a> {
    action: &'a str,
    target: &'a str,
    verifier: &'a str,
    outcome: &'a str,
    contract_emitted: bool,
    gemma4_blocked: &'a str,
    substitute_models: &'a [&'a str],
    runpod_pending: &'a str,
}

#[derive(Serialize)]
struct VerifierStatusRow {
    verifier: &'static str,
    expected_local_install: bool,
    runpod_pending: bool,
}

#[derive(Serialize)]
struct StatusReport<'a> {
    action: &'a str,
    verifiers: Vec<VerifierStatusRow>,
    gemma4_blocked: &'a str,
    substitute_models: &'a [&'a str],
    runpod_pending: &'a str,
}
