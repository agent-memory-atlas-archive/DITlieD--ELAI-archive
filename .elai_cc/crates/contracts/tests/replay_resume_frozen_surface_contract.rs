// E-29 Tier-B contract: a replay/resume MUST NOT bypass the production
// frozen-surface gate. (One of the 6 absent E-29 contract files; the
// feature is already wired -- see
// workflow/sessions/s-plan-audit-2026-06-04/plan-status/E/E-29_agentic_observability.md.)
//
// LOAD-BEARING SAFETY PROPERTY (verbatim from the replay module doc,
// crates/orchestrator/src/observability/replay.rs:6-9 and the E-29
// WIRING_CONTRACT.stub_ban): the replay validator consumes the C-44/D-9
// frozen-surface predicate (LockedKeysAuditor) and is FAIL-CLOSED -- it
// refuses every patch when the registry is unreadable/missing, and it
// refuses any patch whose tool-override surface overlaps a locked key.
//
// WHY THIS IS NOT A TAUTOLOGY (wiring-enforcement R4/R5):
//   * The gate driven here is the REAL production validator
//     `elai_orchestrator::observability::ssb_resume_from` -- the exact
//     symbol registered behind the Tauri `ssb_resume_from` IPC handler
//     (lib.rs generate_handler!, the real entrypoint per the plan-status
//     REACHABILITY note). This test does NOT re-implement the predicate.
//   * The OFF-AGENT ARTIFACT (wiring-enforcement R4) is the
//     `SsbPayload::LockedKeysViolationDetected` row that the production
//     `LockedKeysAuditor::audit_diff` path emits into a capture sink for
//     the SAME synthetic staged-file surface `ssb_resume_from` builds
//     internally (replay.rs:159-166). The test author writes the test
//     INPUT (a registry TOML + a patch); the violation ROW is produced
//     by production orchestrator code, not by this test.
//   * Missing/unreadable registry is asserted to be a hard REFUSAL
//     (PredicateUnavailable), never a permissive Accepted -- this is the
//     bit that would silently fail-open if the wiring regressed.
//
// FAIL CONDITIONS (this test goes RED if the gate is bypassable):
//   * a replay patch touching a locked surface returns Accepted        -> RED
//   * a missing/unreadable registry returns Accepted                   -> RED
//   * the real auditor emits ZERO violation rows for a locked surface  -> RED
//     (the gate is wire-dark: it "ran" but produced no off-agent proof)

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use elai_orchestrator::locked_keys_auditor::{
    LockedKeysAuditor, LockedKeysSink, LOCKED_KEY_FULL_FILE,
};
use elai_orchestrator::observability::replay::ToolOverride;
use elai_orchestrator::observability::{
    ssb_resume_from, ReplayError, ReplayOutcome, ReplayPatch, ReplaySessionId,
};
use elai_orchestrator::ssb::SsbPayload;

/// The frozen surface every case in this file locks. This is a real path
/// in the tree, deliberately the validator's own source so a reader sees
/// the "agent must not edit the function that grades it" intent (D-9
/// grounding arxiv:1908.04734).
const LOCKED_PATH: &str = "crates/orchestrator/src/observability/replay.rs";

/// Write a one-entry locked-keys registry to `path`, locking [`LOCKED_PATH`]
/// as a full-file freeze. This is test INPUT (allowed); the gate behaviour
/// over it is what we assert.
fn write_locked_registry(path: &std::path::Path) {
    let toml_text = format!(
        r#"
[[locked_keys]]
path = "{LOCKED_PATH}"
line_range = [0, 4294967295]
reason = "E-29 frozen-surface contract: replay must not edit its own grader"
owner_session = "contract-e29-replay"
freeze_ts_ms = 0
signature_blake3 = ""
"#
    );
    std::fs::write(path, toml_text).expect("write locked registry");
}

/// Build the patch `ssb_resume_from` will validate: a single tool override
/// bound to the LOCKED path. This mirrors how the dashboard would request
/// a replay that swaps a tool's target onto a frozen file.
fn patch_touching_locked_surface() -> ReplayPatch {
    ReplayPatch {
        prompt: None,
        model: None,
        tool_overrides: vec![ToolOverride {
            tool_name: "edit".into(),
            target_path: LOCKED_PATH.into(),
            label: "swap edit target onto frozen replay.rs".into(),
        }],
    }
}

// -------------------------------------------------------------------------
// Compile-time interface assertions (Layer-2 static leg).
// A signature drift on the production validator breaks this test's compile.
// -------------------------------------------------------------------------

fn _assert_validator_signature() {
    // ssb_resume_from(i64, ReplayPatch, PathBuf)
    //   -> Result<(ReplaySessionId, ReplayOutcome), ReplayError>
    let _f: fn(
        i64,
        ReplayPatch,
        PathBuf,
    ) -> Result<(ReplaySessionId, ReplayOutcome), ReplayError> = ssb_resume_from;
}

// -------------------------------------------------------------------------
// Layer-4 runtime behaviour: drive the REAL validator, assert it cannot be
// routed around the frozen-surface gate.
// -------------------------------------------------------------------------

/// CORE PROPERTY: a replay patch touching a locked surface is REFUSED by
/// the production validator. If this returns Accepted the gate is
/// bypassable and the test is RED.
#[test]
fn replay_refuses_patch_touching_frozen_surface() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let reg_path = dir.path().join("elai-locked-keys.toml");
    write_locked_registry(&reg_path);

    let patch = patch_touching_locked_surface();
    let (session_id, outcome) =
        ssb_resume_from(42, patch, reg_path).expect("validator returns Ok with an outcome");

    assert!(
        matches!(outcome, ReplayOutcome::Refused),
        "FAIL-OPEN: replay accepted a patch onto a frozen surface; expected Refused, got {outcome:?}"
    );
    // The refusal still mints a deterministic, checkpoint-scoped session id
    // (so the dashboard can correlate the refusal row).
    assert!(
        session_id.0.starts_with("replay-42-"),
        "session id must encode the checkpoint, got {}",
        session_id.0
    );
}

/// FAIL-CLOSED on a missing registry: the predicate cannot run, so EVERY
/// patch is refused with PredicateUnavailable -- never silently Accepted.
/// This is the branch that would fail-open if someone "defaulted to allow".
#[test]
fn replay_fails_closed_when_registry_missing() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let missing = dir.path().join("does-not-exist.toml");

    // Use a patch that touches NOTHING locked -- proving the refusal is
    // driven by the unavailable predicate, not by a surface overlap.
    let patch = ReplayPatch {
        prompt: Some("re-run with a tweaked prompt".into()),
        model: Some("qwen-2.5-coder-3b".into()),
        tool_overrides: vec![],
    };

    let (_sid, outcome) =
        ssb_resume_from(7, patch, missing).expect("validator returns Ok with an outcome");

    assert!(
        matches!(outcome, ReplayOutcome::PredicateUnavailable),
        "FAIL-OPEN: missing registry must refuse (PredicateUnavailable), got {outcome:?}"
    );
    assert!(
        !matches!(outcome, ReplayOutcome::Accepted),
        "a replay MUST NOT be accepted when the frozen-surface predicate cannot run"
    );
}

/// Control case: a patch that touches NO locked surface against a present,
/// clean-for-this-surface registry is Accepted. Without this, the two
/// refusal tests above could pass by a validator that refuses everything
/// (a degenerate gate is not a gate). This pins that the gate discriminates.
#[test]
fn replay_accepts_patch_clear_of_frozen_surface() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let reg_path = dir.path().join("elai-locked-keys.toml");
    write_locked_registry(&reg_path);

    let patch = ReplayPatch {
        prompt: None,
        model: None,
        tool_overrides: vec![ToolOverride {
            tool_name: "read".into(),
            target_path: "crates/dashboard-v2/src/App.svelte".into(),
            label: "unrelated, not frozen".into(),
        }],
    };

    let (_sid, outcome) =
        ssb_resume_from(9, patch, reg_path).expect("validator returns Ok with an outcome");

    assert!(
        matches!(outcome, ReplayOutcome::Accepted),
        "a patch clear of every frozen surface must be Accepted, got {outcome:?}; \
         a validator that refuses this is a degenerate (always-refuse) gate"
    );
}

/// OFF-AGENT ARTIFACT (wiring-enforcement R4): drive the SAME production
/// auditor `ssb_resume_from` uses internally, over the SAME synthetic
/// staged-file surface it builds (replay.rs:159-166), with a capture sink,
/// and assert the production `audit_diff` path emits a real
/// `SsbPayload::LockedKeysViolationDetected` row.
///
/// The test author writes only the INPUT (registry + staged path). The
/// violation ROW is authored by production orchestrator code
/// (LockedKeysAuditor::emit_violation). A gate that "refused" without
/// leaving this row would be wire-dark; absent row => RED (R1/R2).
#[test]
fn frozen_surface_gate_emits_offagent_violation_row() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let reg_path = dir.path().join("elai-locked-keys.toml");
    write_locked_registry(&reg_path);

    // Capture sink: the buffer the production auditor pushes SSB payloads
    // into. This is the off-agent artifact channel.
    let buf: Arc<Mutex<Vec<SsbPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = LockedKeysSink::Capture(buf.clone());

    let auditor = LockedKeysAuditor::load(&reg_path, Some(sink))
        .expect("registry present => auditor loads (fail-closed only on missing)");

    // Reconstruct the exact synthetic staged-file list `ssb_resume_from`
    // builds from a patch touching the locked surface (replay.rs:159-166):
    // one PathBuf per tool override target.
    let staged = vec![PathBuf::from(LOCKED_PATH)];
    let result = auditor.audit_diff(&staged);

    assert_eq!(
        result.violations.len(),
        1,
        "production auditor must flag the locked surface exactly once"
    );

    let captured = buf.lock().expect("capture buffer lock");
    // Production emits exactly one audit-run row + one violation row.
    let violation_rows = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysViolationDetected(_)))
        .count();
    let audit_run_rows = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysAuditRun(_)))
        .count();

    assert_eq!(
        violation_rows, 1,
        "OFF-AGENT PROOF MISSING: expected exactly 1 LockedKeysViolationDetected row \
         from the production audit path, got {violation_rows} (gate is wire-dark)"
    );
    assert_eq!(
        audit_run_rows, 1,
        "the audit happened => exactly one LockedKeysAuditRun row must be emitted"
    );

    // The emitted row must carry the locked path (proves the row is real,
    // not an empty placeholder).
    let row_names_path = captured.iter().any(|p| match p {
        SsbPayload::LockedKeysViolationDetected(v) => {
            v.locked_key_path == LOCKED_PATH && v.attempted_edit == LOCKED_PATH
        }
        _ => false,
    });
    assert!(
        row_names_path,
        "the violation row must name the locked path {LOCKED_PATH}; \
         a row without the offending path is not a usable audit artifact"
    );
}

/// Cross-check: a clean surface emits an audit-run row but ZERO violation
/// rows. Pins that the violation row in the test above is caused by the
/// overlap, not unconditionally emitted (otherwise the off-agent artifact
/// would be a constant, not evidence).
#[test]
fn clean_surface_emits_audit_run_but_no_violation_row() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let reg_path = dir.path().join("elai-locked-keys.toml");
    write_locked_registry(&reg_path);

    let buf: Arc<Mutex<Vec<SsbPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let auditor = LockedKeysAuditor::load(&reg_path, Some(LockedKeysSink::Capture(buf.clone())))
        .expect("auditor loads");

    let staged = vec![PathBuf::from("crates/dashboard-v2/src/App.svelte")];
    let result = auditor.audit_diff(&staged);
    assert!(result.violations.is_empty(), "unrelated path must not violate");

    let captured = buf.lock().expect("lock");
    let violation_rows = captured
        .iter()
        .filter(|p| matches!(p, SsbPayload::LockedKeysViolationDetected(_)))
        .count();
    assert_eq!(
        violation_rows, 0,
        "a clean surface must emit ZERO violation rows (the artifact is evidence, not a constant)"
    );
    assert_eq!(
        captured.len(),
        1,
        "exactly one audit-run row and nothing else for a clean surface"
    );
    // Touch LOCKED_KEY_FULL_FILE so an accidental removal of the public
    // const (a real signature drift) is a compile break here too.
    let _full: (u32, u32) = LOCKED_KEY_FULL_FILE;
}
