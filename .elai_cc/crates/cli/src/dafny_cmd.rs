//! C-52 — `elai dafny <verify|status|list-tiers>` handler.
//!
//! Drives the orchestrator's [`elai_orchestrator::dafny_verify`]
//! module. Three sub-actions:
//!   - `verify --spec-path PATH [--tier signature] [--max-iter 10]`:
//!     shell out to `dafny verify <spec>` and print the parsed
//!     [`DafnyVerifyResult`]. Tier defaults to `signature` per
//!     paper Section 3 (minimum viable tier; contextless has
//!     near-universal failure across 6 of 7 models).
//!   - `status`: report `dafny --version` availability + the
//!     current ELAI substitute model selection. Exits 0 even when
//!     the toolchain is missing (the absence is the canonical
//!     `RUNPOD-PENDING: live_dafny_verification_with_non_gemma_substitution`
//!     signal).
//!   - `list-tiers`: emit the canonical paper tier list
//!     (Contextless / Signature / SelfHealing per RQ1/RQ2/RQ3).
//!
//! Paper grounding caveats (arXiv:2604.22601):
//!   - Model substitution MANDATORY — Gemma 4 BLOCKED per memory
//!     rule `feedback_no_gemma4`. ELAI substitutes from the paper's
//!     evaluation set: Codestral-22B-v0.1 (63.64% RQ2 verify@5),
//!     GPT-OSS-120B (63.64% RQ2 + 81.82% RQ3b), Qwen 3 Coder 30B
//!     (45.45% RQ2). Current ELAI sidecar default (Qwen 2.5 Coder
//!     7B) is NOT in the paper's evaluation set; expected verify@5
//!     below 45% on the signature tier.
//!   - Live `dafny verify` execution is RUNPOD-PENDING. The local
//!     dev host lacks the `dafny` binary on PATH; this CLI surface
//!     ships fully but live results require a remote runner with
//!     Dafny installed.

use std::path::PathBuf;

use elai_orchestrator::dafny_verify::{
    invoke_dafny_cli, DafnyCliError, DafnyTier, ELAI_SUBSTITUTE_MODELS, GEMMA4_BLOCKED,
    MAX_HEAL_ITER, TEMPERATURE_SWEEP, VERIFY_K_VALUES,
};
use serde::Serialize;
use tokio::runtime::Builder;

use crate::args::{CliOutputFormat, DafnyAction};

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(action: DafnyAction, output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        DafnyAction::Verify {
            spec_path,
            tier,
            max_iter,
        } => run_verify(&spec_path, tier.as_deref(), max_iter, output_format),
        DafnyAction::Status => run_status(output_format),
        DafnyAction::ListTiers => run_list_tiers(output_format),
    }
}

fn run_verify(
    spec_path: &str,
    tier_str: Option<&str>,
    max_iter: Option<u32>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let tier = match tier_str {
        Some(s) => DafnyTier::from_str(s)
            .ok_or_else(|| format!("unknown --tier `{s}`; expected: contextless | signature | self_healing"))?,
        // Default to Signature per paper §3 (minimum viable tier;
        // Contextless has near-universal failure).
        None => DafnyTier::Signature,
    };
    let effective_max_iter = max_iter.unwrap_or(MAX_HEAL_ITER).min(MAX_HEAL_ITER);

    // Build a current-thread runtime so the sync CLI entry stays
    // sync (mirrors archive_verify.rs pattern). The dafny shellout
    // is the only async call so a single-threaded runtime is enough.
    let rt = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|e| format!("tokio runtime: {e}"))?;
    let spec_pathbuf = PathBuf::from(spec_path);
    let result = rt.block_on(async { invoke_dafny_cli(&spec_pathbuf).await });
    match result {
        Ok(res) => {
            let report = VerifyReport {
                action: "verify",
                spec_path,
                tier: tier.as_str(),
                max_iter: effective_max_iter,
                dafny_pass: res.dafny_pass,
                udebug_pass: res.udebug_pass,
                vacuous_guard_passed: res.vacuous_guard_passed,
                exit_code: res.exit_code,
                substitute_models: &ELAI_SUBSTITUTE_MODELS,
                gemma4_blocked: GEMMA4_BLOCKED,
                runpod_pending: "live_dafny_verification_with_non_gemma_substitution",
            };
            match output_format {
                CliOutputFormat::Json => print_json(&report)?,
                _ => print_verify_text(&report),
            }
            Ok(())
        }
        Err(DafnyCliError::ToolchainNotFound(_)) => {
            // Expected on local dev host. Surface as a structured
            // RUNPOD-PENDING marker; exit 0 (absence is informative).
            let report = ToolchainReport {
                action: "verify",
                spec_path,
                tier: tier.as_str(),
                max_iter: effective_max_iter,
                dafny_available: false,
                substitute_models: &ELAI_SUBSTITUTE_MODELS,
                gemma4_blocked: GEMMA4_BLOCKED,
                runpod_pending: "live_dafny_verification_with_non_gemma_substitution",
                note: "dafny binary not on PATH (install per dafny.org)",
            };
            match output_format {
                CliOutputFormat::Json => print_json(&report)?,
                _ => print_toolchain_text(&report),
            }
            Ok(())
        }
        Err(e) => Err(format!("dafny verify: {e}")),
    }
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    // Probe `dafny --version`; absence is informative not fatal.
    let rt = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|e| format!("tokio runtime: {e}"))?;
    let dafny_available = rt.block_on(async { probe_dafny_version().await });

    let status = StatusReport {
        action: "status",
        dafny_available,
        substitute_models: &ELAI_SUBSTITUTE_MODELS,
        gemma4_blocked: GEMMA4_BLOCKED,
        temperature_sweep: &TEMPERATURE_SWEEP,
        verify_k_values: &VERIFY_K_VALUES,
        runpod_pending: "live_dafny_verification_with_non_gemma_substitution",
    };
    match output_format {
        CliOutputFormat::Json => print_json(&status)?,
        _ => print_status_text(&status),
    }
    Ok(())
}

fn run_list_tiers(output_format: CliOutputFormat) -> Result<(), String> {
    let tiers: Vec<&'static str> = DafnyTier::ALL.iter().map(|t| t.as_str()).collect();
    let report = TiersReport {
        action: "list_tiers",
        tiers,
        paper: "arXiv:2604.22601",
        rq_mapping: &[
            ("contextless", "RQ1"),
            ("signature", "RQ2"),
            ("self_healing", "RQ3"),
        ],
    };
    match output_format {
        CliOutputFormat::Json => print_json(&report)?,
        _ => {
            println!("Dafny prompting tiers (arXiv:2604.22601):");
            for (tier, rq) in report.rq_mapping {
                println!("  {tier:13}  ({rq})");
            }
        }
    }
    Ok(())
}

async fn probe_dafny_version() -> bool {
    let mut cmd = tokio::process::Command::new("dafny");
    cmd.arg("--version");
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());
    matches!(cmd.status().await, Ok(s) if s.success())
}

fn print_json<T: Serialize>(report: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("dafny json encode: {e}"))?;
    println!("{json}");
    Ok(())
}

fn print_verify_text(r: &VerifyReport<'_>) {
    println!("dafny verify ({})", r.spec_path);
    println!("  tier             : {}", r.tier);
    println!("  max_iter         : {} (cap = {})", r.max_iter, MAX_HEAL_ITER);
    println!("  dafny_pass       : {}", r.dafny_pass);
    println!("  udebug_pass      : {}", r.udebug_pass);
    println!("  vacuous_guard    : {}", r.vacuous_guard_passed);
    println!("  exit_code        : {}", r.exit_code);
    println!("  gemma4_blocked   : {}", r.gemma4_blocked);
    println!("  substitute_models: {:?}", r.substitute_models);
    println!("  runpod_pending   : {}", r.runpod_pending);
}

fn print_toolchain_text(r: &ToolchainReport<'_>) {
    println!("dafny verify ({}) [toolchain absent]", r.spec_path);
    println!("  tier             : {}", r.tier);
    println!("  dafny_available  : {}", r.dafny_available);
    println!("  note             : {}", r.note);
    println!("  gemma4_blocked   : {}", r.gemma4_blocked);
    println!("  substitute_models: {:?}", r.substitute_models);
    println!("  runpod_pending   : {}", r.runpod_pending);
}

fn print_status_text(r: &StatusReport<'_>) {
    println!("dafny status (paper arXiv:2604.22601)");
    println!("  dafny_available  : {}", r.dafny_available);
    println!("  gemma4_blocked   : {}", r.gemma4_blocked);
    println!("  substitute_models: {:?}", r.substitute_models);
    println!("  temperature_sweep: {:?}", r.temperature_sweep);
    println!("  verify_k_values  : {:?}", r.verify_k_values);
    println!("  runpod_pending   : {}", r.runpod_pending);
}

#[derive(Serialize)]
struct VerifyReport<'a> {
    action: &'a str,
    spec_path: &'a str,
    tier: &'a str,
    max_iter: u32,
    dafny_pass: bool,
    udebug_pass: bool,
    vacuous_guard_passed: bool,
    exit_code: i32,
    substitute_models: &'a [&'a str],
    gemma4_blocked: &'a str,
    runpod_pending: &'a str,
}

#[derive(Serialize)]
struct ToolchainReport<'a> {
    action: &'a str,
    spec_path: &'a str,
    tier: &'a str,
    max_iter: u32,
    dafny_available: bool,
    substitute_models: &'a [&'a str],
    gemma4_blocked: &'a str,
    runpod_pending: &'a str,
    note: &'a str,
}

#[derive(Serialize)]
struct StatusReport<'a> {
    action: &'a str,
    dafny_available: bool,
    substitute_models: &'a [&'a str],
    gemma4_blocked: &'a str,
    temperature_sweep: &'a [f32],
    verify_k_values: &'a [u32],
    runpod_pending: &'a str,
}

#[derive(Serialize)]
struct TiersReport {
    action: &'static str,
    tiers: Vec<&'static str>,
    paper: &'static str,
    rq_mapping: &'static [(&'static str, &'static str)],
}
