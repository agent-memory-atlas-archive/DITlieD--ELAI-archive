//! E-53 — `elai formal-verify <run|status>` handler.
//!
//! Drives the orchestrator's [`elai_orchestrator::formal_verify`]
//! module. Two sub-actions:
//!   - `run --target PATH --verifier KIND [--demo-escape-hatch]`:
//!     dispatch one verifier (z3_smt | dafny | kani | verus) against
//!     PATH and print the [`PipelineResult`]. With `--demo-escape-hatch`,
//!     feed a seeded goal containing a relocated sorry (the AlphaProof
//!     failure mode 1:1) so the scan returns EscapeHatchAnywhere and the
//!     TrapBehaviorDetected(ClosureEscapeHatch) SSB row is emitted.
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
//! H-54 observable_effect: `--demo-escape-hatch` makes a REAL call to
//! `orchestrator::run()` with a seeded displaced-sorry goal, gets an
//! `EscapeHatchAnywhere` block result, and emits a decodable
//! `TrapBehaviorDetected(ClosureEscapeHatch)` SSB row to the session SSB.
//! This closes the W-1 done-gate for H-54 without requiring live verifier
//! binaries (Tier-2 / RunPod deferred).
//!
//! Per H-7 / H-9: no `.unwrap()` / `.expect()` outside test code.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::dafny_verify::{ELAI_SUBSTITUTE_MODELS, GEMMA4_BLOCKED};
use elai_orchestrator::formal_verify::{
    run as run_pipeline, ClosureScanKind, PipelineResult, VerifierKind, VerifierOutcome,
    VerifyRequest, ALL_VERIFIER_KINDS,
};
use elai_orchestrator::ssb::{
    Ssb, SsbConfig, SsbPayload, TrapBehaviorDetectedKind, TrapBehaviorDetectedPayload,
};
use serde::Serialize;
use tokio::runtime::Builder;

use crate::args::{CliOutputFormat, FormalVerifyAction};

const FORMAL_VERIFY_AGENT_ID: &str = "formal-verify-demo";
const FORMAL_VERIFY_PROJECT_ID: &str = "elai-formal-verify";
const FORMAL_VERIFY_SESSION_ID: &str = "formal-verify-demo-session";

/// Top-level dispatcher invoked by `main.rs`.
pub(crate) fn run(
    action: FormalVerifyAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        FormalVerifyAction::Run { target, verifier, demo_escape_hatch } => {
            run_pipeline_dispatch(&target, &verifier, demo_escape_hatch, output_format)
        }
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
    demo_escape_hatch: bool,
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
        demo_escape_hatch,
    };
    let result: PipelineResult = rt.block_on(async { run_pipeline(request).await });

    // H-54 W-1 observable_effect: when --demo-escape-hatch fired, the scan
    // returned EscapeHatchAnywhere. Emit the TrapBehaviorDetected(ClosureEscapeHatch)
    // SSB row to the session SSB so the gate is provably non-inert through a REAL
    // entrypoint. This is the off-agent artifact R4 requires.
    let ssb_row_emitted = if demo_escape_hatch {
        match result.closure_scan.as_ref() {
            Some(scan) if scan.kind == ClosureScanKind::EscapeHatchAnywhere => {
                rt.block_on(async { emit_demo_trap_row(target, verifier, scan.closure_size).await })
            }
            _ => Err("demo_escape_hatch was set but scan did not return EscapeHatchAnywhere".to_string()),
        }
    } else {
        Ok(None)
    };
    let ssb_row_id = ssb_row_emitted.unwrap_or(None);

    let (closure_scan_kind, closure_size, closure_blocked) = result
        .closure_scan
        .as_ref()
        .map(|s| (s.kind.as_str(), s.closure_size, s.kind.is_block()))
        .unwrap_or(("bypassed", 0, false));
    let report = RunReport {
        action: "run",
        target,
        verifier: verifier.as_str(),
        outcome: result.outcome.as_str(),
        contract_emitted: result.contract_emitted,
        closure_scan_kind,
        closure_size,
        closure_blocked,
        demo_escape_hatch,
        ssb_row_id,
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

/// H-54 W-1: emit a `TrapBehaviorDetected(ClosureEscapeHatch)` SSB row to the
/// session SSB. This is the off-agent artifact that proves the gate is non-inert
/// through a REAL entrypoint (`elai formal-verify run --demo-escape-hatch`).
///
/// Uses zero recall/fp_rate/n values (the GD-4 calibration on real verifier runs
/// is Tier-2/RunPod deferred). The candidate_blake3 is the blake3 prefix of the
/// target path bytes (the seeded fixture's proxy identifier).
///
/// Returns the SSB row_id on success.
async fn emit_demo_trap_row(
    target: &str,
    verifier: VerifierKind,
    closure_size: u32,
) -> Result<Option<i64>, String> {
    let project_root = std::env::current_dir()
        .map_err(|e| format!("current_dir: {e}"))?;
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("create sessions_dir: {e}"))?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: None,
        project_id: FORMAL_VERIFY_PROJECT_ID.to_string(),
        session_id: FORMAL_VERIFY_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(cfg, clock).map_err(|e| format!("ssb open: {e}"))?,
    );

    // candidate_blake3: blake3 prefix of the seeded goal's target path bytes
    // (the Tier-1 proxy id; a real candidate_blake3 requires a live config_blob).
    let candidate_hash = blake3::hash(target.as_bytes());
    let candidate_blake3 = &candidate_hash.to_hex()[..16];

    // corpus_hash: blake3 of the constant string "h54_demo_seeded_fixture"
    // (the frozen corpus identifier for the demo path; not a real recall measurement).
    let corpus_hash_bytes: [u8; 32] = *blake3::hash(b"h54_demo_seeded_fixture").as_bytes();

    let payload = TrapBehaviorDetectedPayload {
        kind: TrapBehaviorDetectedKind::ClosureEscapeHatch,
        recall: 0.0, // GD-4 deferred (Tier-2 / RunPod)
        fp_rate: 0.0,
        n: 0,
        corpus_hash: corpus_hash_bytes,
        candidate_blake3: candidate_blake3.to_string(),
        seed: 0,     // ELAI_SEED not used in the demo path
        behavioral_flag: true, // ClosureEscapeHatch IS a behavioral (deterministic) block
    };

    let _ = verifier; // verifier name already in the escape_ids inside the pipeline result
    let row_id = ssb
        .write_payload(
            FORMAL_VERIFY_AGENT_ID,
            0,
            SsbPayload::TrapBehaviorDetected(payload),
            vec![],
        )
        .await
        .map_err(|e| format!("ssb write: {e}"))?;

    tracing::info!(
        row_id,
        closure_size,
        "H-54 demo: TrapBehaviorDetected(ClosureEscapeHatch) SSB row emitted \
         (seeded displaced-sorry goal through real formal-verify entrypoint)"
    );
    Ok(Some(row_id))
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
    // H-54 closure scan line (Layer-2 first leg).
    println!(
        "  closure_scan     : {} (size={}, blocked={})",
        r.closure_scan_kind, r.closure_size, r.closure_blocked
    );
    if r.demo_escape_hatch {
        match r.ssb_row_id {
            Some(row_id) => println!(
                "  demo_escape_hatch: TrapBehaviorDetected(ClosureEscapeHatch) SSB row_id={row_id} \
                 (H-54 W-1 off-agent artifact — seeded displaced-sorry through real entrypoint)"
            ),
            None => println!(
                "  demo_escape_hatch: SSB row NOT emitted (scan did not return EscapeHatchAnywhere)"
            ),
        }
    }
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
    /// H-54: closure scan verdict (Layer-2 first leg).
    closure_scan_kind: &'a str,
    /// H-54: number of obligation nodes visited during BFS.
    closure_size: u32,
    /// H-54: whether the closure scan blocked the verifier dispatch.
    closure_blocked: bool,
    /// H-54: whether the seeded-fixture demo trigger was active.
    demo_escape_hatch: bool,
    /// H-54 W-1 off-agent artifact: SSB row_id when the demo emitted
    /// TrapBehaviorDetected(ClosureEscapeHatch). None when demo was not
    /// active or the emit failed.
    ssb_row_id: Option<i64>,
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
