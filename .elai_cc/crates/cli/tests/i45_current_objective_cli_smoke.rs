//! I-45 S6b — CLI smoke for `elai replay --verify --current-objective <text>`.
//!
//! Drives the COMPILED `elai` binary (via `CARGO_BIN_EXE_elai`) against a
//! tempdir `.rra` fixture sealed in-process from a one-row
//! `IdeaFramingStarted` SSB stream. Pins three observable-effect contracts
//! of the GAP-1 `--current-objective` threading:
//!
//! 1. SAME text  -> exit 0  (Verified).
//! 2. DIFFERENT text -> exit 1  (ObjectiveHashMismatch; the anti-fail-open arm).
//! 3. `--current-objective` mixed with a demo gate flag (`--golden`) -> nonzero
//!    parse error (fail-closed mix rejection in args.rs).
//!
//! The .rra is built with the SAME orchestrator primitives the contract test
//! uses (`build_rra` + `write_rra` + `encode_ssb_payload`), so the seal is
//! the real production seal, not a hand-rolled fixture. No live sidecar, no
//! GPU, no network; pure in-process seal + offline CLI verify.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use elai_orchestrator::determinism::{DeterminismConfig, HardwareClass, PREFIX_CACHE_KEY_BYTES};
use elai_orchestrator::rra::{build_rra, write_rra};
use elai_orchestrator::ssb::{
    IdeaFramingStartedPayload, SsbActionType, SsbEntry, SsbPayload,
};
use elai_orchestrator::encode_ssb_payload;

use tempfile::TempDir;

/// Minimal `DeterminismConfig` with a fixed seed (no env reads). Mirrors the
/// contract test at crates/contracts/tests/i45_objective_seal_contract.rs.
fn test_pins() -> DeterminismConfig {
    DeterminismConfig {
        seed: 42,
        hardware_class: HardwareClass::Cpu,
        system_fingerprint_pin: None,
        blake3_cache_prefix: [0u8; PREFIX_CACHE_KEY_BYTES],
    }
}

/// Build a one-row SSB stream whose single row is an `IdeaFramingStarted`
/// payload carrying `task_text` as the sealed objective.
fn stream_with_objective(task_text: &str) -> Vec<SsbEntry> {
    let payload = SsbPayload::IdeaFramingStarted(IdeaFramingStartedPayload {
        session_id: "s-i45-cli-smoke".to_owned(),
        prompt: task_text.to_owned(),
        started_at_ms: 1_700_000_000_000,
    });
    let payload_bytes: Arc<[u8]> = encode_ssb_payload(&payload)
        .expect("encode IdeaFramingStarted payload")
        .into();
    vec![SsbEntry {
        seq: 1,
        project_id: "test-project".to_owned(),
        session_id: "s-i45-cli-smoke".to_owned(),
        agent_id: "cli:smoke".to_owned(),
        superstep: 0,
        action_type: SsbActionType::IdeaFramingStarted,
        payload: payload_bytes,
        created_at: 1_700_000_000_000,
    }]
}

/// Seal a `.rra` for `task_text` into `dir/objective.rra` and return its path.
fn seal_fixture(dir: &Path, task_text: &str) -> PathBuf {
    let stream = stream_with_objective(task_text);
    let art = build_rra("s-i45-cli-smoke", &stream, &test_pins());
    assert!(
        art.objective_hash.is_some(),
        "fixture precondition: sealed objective_hash must be Some"
    );
    let path = dir.join("objective.rra");
    write_rra(&art, &path).expect("write .rra fixture");
    path
}

fn elai_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_elai"))
}

// ---------------------------------------------------------------------------
// (1) SAME objective text -> exit 0 (Verified).
// ---------------------------------------------------------------------------

#[test]
fn same_objective_exits_zero() {
    let tmp = TempDir::new().expect("tempdir");
    let task = "implement the foo feature";
    let rra_path = seal_fixture(tmp.path(), task);

    let out = Command::new(elai_binary())
        .args(["replay", "--verify"])
        .arg(&rra_path)
        .args(["--current-objective", task])
        .output()
        .expect("spawn elai replay --verify <path> --current-objective <same>");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        out.status.success(),
        "expected exit 0 for matching objective; status={:?}; output=\n{}",
        out.status.code(),
        combined,
    );
}

// ---------------------------------------------------------------------------
// (2) DIFFERENT objective text -> exit 1 (ObjectiveHashMismatch).
//     THE CRITICAL ANTI-FAIL-OPEN ARM: if this exits 0 the implementation is
//     fail-open and the seal is meaningless.
// ---------------------------------------------------------------------------

#[test]
fn mutated_objective_exits_nonzero() {
    let tmp = TempDir::new().expect("tempdir");
    let original_task = "implement the foo feature";
    let mutated_task = "implement the BAR feature";
    let rra_path = seal_fixture(tmp.path(), original_task);

    let out = Command::new(elai_binary())
        .args(["replay", "--verify"])
        .arg(&rra_path)
        .args(["--current-objective", mutated_task])
        .output()
        .expect("spawn elai replay --verify <path> --current-objective <mutated>");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    let code = out.status.code();
    assert_eq!(
        code,
        Some(1),
        "mutated objective MUST exit 1 (ObjectiveHashMismatch); got status={:?}; output=\n{}",
        code,
        combined,
    );
    assert!(
        combined.contains("OBJECTIVE-MISMATCH") || combined.contains("ObjectiveHashMismatch"),
        "expected objective-mismatch signal in output; got:\n{}",
        combined,
    );
}

// ---------------------------------------------------------------------------
// (3) `--current-objective` mixed with a demo gate flag (`--golden`) is
//     rejected at the parser with a nonzero parse error. The args.rs
//     fail-closed mix guard already enforces this; the smoke pins the
//     observable CLI behavior so a future loosening cannot silently pass.
// ---------------------------------------------------------------------------

#[test]
fn current_objective_rejected_with_demo_flag() {
    let tmp = TempDir::new().expect("tempdir");
    let task = "implement the foo feature";
    let rra_path = seal_fixture(tmp.path(), task);

    let out = Command::new(elai_binary())
        .args(["replay", "--verify"])
        .arg(&rra_path)
        .args([
            "--current-objective",
            task,
            "--golden",
            "parity",
        ])
        .output()
        .expect("spawn elai replay --verify <path> --current-objective + --golden");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        !out.status.success(),
        "mixing --current-objective with --golden MUST exit nonzero; status={:?}; output=\n{}",
        out.status.code(),
        combined,
    );
    assert!(
        combined.contains("--current-objective") || combined.contains("cannot be combined"),
        "expected parse-error mentioning --current-objective mix rejection; got:\n{}",
        combined,
    );
}
