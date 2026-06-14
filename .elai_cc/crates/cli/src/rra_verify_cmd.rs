//! I-40 — `elai rra verify <path>` spec-conformance checker.
//!
//! Validates that a `.rra` bundle satisfies the rra-spec-v1 structural
//! requirements (docs/spec/rra-spec-v1.md, sections 3 + 8). This is DISTINCT
//! from `elai replay --verify` (which re-derives content hashes; GD-5
//! verify-divergence). Two orthogonal checks:
//!   * `elai rra verify`    — STRUCTURE/SCHEMA conformance (this module).
//!   * `elai replay --verify` — CONTENT tamper / divergence (rra_cmd.rs).
//!
//! Exits nonzero when any C1-C6 required check fails so CI can gate on it
//! (mirrors the `elai replay --verify` exit contract). The informational
//! I1-I4 items are printed but do not affect exit code.

use std::path::{Path, PathBuf};

use elai_orchestrator::rra::read_rra;

use crate::args::CliOutputFormat;

/// Spec-conformance verdict for a single .rra bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConformanceVerdict {
    /// All C1-C6 required checks pass.
    Conformant {
        /// Which informational items (I1-I4) are present.
        info_flags: InfoFlags,
    },
    /// At least one required check failed.
    NonConformant {
        /// The first failure reason encountered.
        reason: String,
        /// All failure reasons (including subsequent checks that still ran).
        all_reasons: Vec<String>,
    },
}

/// Informational items from the I1-I4 checks (present = true, absent = false).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct InfoFlags {
    /// I1: chain_prev_hash is present (I-18 row chaining active).
    pub chain_prev_hash_present: bool,
    /// I2: stack_hash is present (I-18 5-layer hash active).
    pub stack_hash_present: bool,
    /// I3: objective_hash is present (I-45 task-objective sealing active).
    pub objective_hash_present: bool,
    /// I4: events list is non-empty.
    pub events_non_empty: bool,
}

/// `elai rra verify <path>` — spec-conformance check.
///
/// Reads the bundle at `path`, runs the C1-C6 required checks, prints the
/// result, and exits nonzero on any failure. Called from `rra_cmd::run_rra`
/// on `RraAction::Verify`.
pub(crate) fn verify_conformance(
    path: &Path,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let art = read_rra(path)
        .map_err(|e| format!("reading .rra at {}: {e}", path.display()))?;

    // C1: format_version == 1
    let mut failures: Vec<String> = Vec::new();
    if art.format_version != 1 {
        failures.push(format!(
            "C1 FAIL: format_version is {} but spec requires 1",
            art.format_version
        ));
    }

    // C2: session_id non-empty
    if art.session_id.is_empty() {
        failures.push("C2 FAIL: session_id is empty".to_string());
    }

    // C3: root_blake3 non-empty
    if art.root_blake3.is_empty() {
        failures.push("C3 FAIL: root_blake3 is empty".to_string());
    }

    // C4: every RraEvent has non-empty action_type and non-empty event_blake3
    for (i, ev) in art.events.iter().enumerate() {
        if ev.action_type.is_empty() {
            failures.push(format!(
                "C4 FAIL: event[{}] (seq {}) has empty action_type",
                i, ev.seq
            ));
        }
        if ev.event_blake3.is_empty() {
            failures.push(format!(
                "C4 FAIL: event[{}] (seq {}) has empty event_blake3 (content hash)",
                i, ev.seq
            ));
        }
    }

    // C5: every GateVerdictHash entry has non-empty gate, action_type, verdict_blake3
    for (i, gvh) in art.manifest.iter().enumerate() {
        if gvh.gate.is_empty() {
            failures.push(format!(
                "C5 FAIL: manifest[{}] (seq {}) has empty gate",
                i, gvh.ssb_event_seq
            ));
        }
        if gvh.action_type.is_empty() {
            failures.push(format!(
                "C5 FAIL: manifest[{}] (seq {}) has empty action_type",
                i, gvh.ssb_event_seq
            ));
        }
        if gvh.verdict_blake3.is_empty() {
            failures.push(format!(
                "C5 FAIL: manifest[{}] (seq {}) has empty verdict_blake3",
                i, gvh.ssb_event_seq
            ));
        }
    }
    // C6 (decode): if we got here the msgpack decoded without error (read_rra
    // returned Ok), so C6 passes implicitly.

    // Informational items
    let info = InfoFlags {
        chain_prev_hash_present: art.chain_prev_hash.is_some(),
        stack_hash_present: art.stack_hash.is_some(),
        objective_hash_present: art.objective_hash.is_some(),
        events_non_empty: !art.events.is_empty(),
    };

    let verdict = if failures.is_empty() {
        ConformanceVerdict::Conformant { info_flags: info.clone() }
    } else {
        ConformanceVerdict::NonConformant {
            reason: failures[0].clone(),
            all_reasons: failures.clone(),
        }
    };

    // Print the verdict in the requested format.
    match output_format {
        CliOutputFormat::Json => {
            let failure_arr: Vec<serde_json::Value> = failures
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect();
            let out = serde_json::json!({
                "rra_spec_conformance": {
                    "path": path.display().to_string(),
                    "spec_version": 1,
                    "format_version": art.format_version,
                    "session_id": art.session_id,
                    "event_count": art.events.len(),
                    "gate_manifest_count": art.manifest.len(),
                    "root_blake3": art.root_blake3,
                    "conformant": failures.is_empty(),
                    "failures": failure_arr,
                    "info": {
                        "I1_chain_prev_hash": info.chain_prev_hash_present,
                        "I2_stack_hash": info.stack_hash_present,
                        "I3_objective_hash": info.objective_hash_present,
                        "I4_events_non_empty": info.events_non_empty,
                    }
                }
            });
            let json = serde_json::to_string_pretty(&out)
                .map_err(|e| format!("serializing conformance verdict as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("rra-spec-v1 conformance check: {}", path.display());
            println!("  format_version : {}", art.format_version);
            println!("  session_id     : {}", art.session_id);
            println!("  root_blake3    : {}", art.root_blake3);
            println!("  event_count    : {}", art.events.len());
            println!("  gate_manifest  : {} verdict(s)", art.manifest.len());
            println!();
            println!("  Informational:");
            println!("    I1 chain_prev_hash : {}", if info.chain_prev_hash_present { "present" } else { "absent" });
            println!("    I2 stack_hash      : {}", if info.stack_hash_present { "present" } else { "absent" });
            println!("    I3 objective_hash  : {}", if info.objective_hash_present { "present" } else { "absent" });
            println!("    I4 events non-empty: {}", if info.events_non_empty { "yes" } else { "no (empty event list)" });
            println!();
            if failures.is_empty() {
                println!("  CONFORMANT — all C1-C6 required checks pass.");
            } else {
                println!("  NON-CONFORMANT — {} required check(s) failed:", failures.len());
                for f in &failures {
                    println!("    {f}");
                }
            }
        }
    }

    // Exit nonzero on non-conformant so CI can gate on it (mirrors run_verify).
    match verdict {
        ConformanceVerdict::Conformant { .. } => Ok(()),
        ConformanceVerdict::NonConformant { .. } => {
            std::process::exit(1);
        }
    }
}

/// Public entry-point called from `rra_cmd::run_rra` for `RraAction::Verify`.
pub(crate) fn run_verify_conformance(
    path: PathBuf,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    verify_conformance(&path, output_format)
}
