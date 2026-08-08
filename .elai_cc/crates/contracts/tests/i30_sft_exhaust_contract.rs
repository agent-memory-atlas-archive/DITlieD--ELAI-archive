//! I-30 Slice 2 — contract test for the SftExhaust CLI path.
//!
//! Source plan: docs/thefinalglance/I-plan/I-30_verifier_ranked_sft_exhaust.md
//! Slice 1a (SftExhaustEvent SSB) + Slice 1b (CLI sft_exhaust.rs +
//! dataset_cmd.rs) landed at d188553 + 7e90f81 on main.
//!
//! Drives the REAL `elai_cli::sft_exhaust::SftExhaust::new(..).run()` path
//! (NOT a unit shim). `run()` is SYNC — it builds its own current_thread
//! tokio runtime internally — so these tests use plain `#[test]` and do NOT
//! nest a runtime (prior failure mode: "Cannot start a runtime from within
//! a runtime").
//!
//! Tests:
//! (a) EMPTY rra_dir: run() Ok -> path_extracted=0, skipped_unsealed=0,
//!     dataset_sealed=1 (final seal still emits), no panic.
//! (b) NON-VERIFIED / malformed .rra fixture: bogus `*.rra` (random bytes)
//!     -> skipped_unsealed >= 1, path_extracted=0, run() Ok (clean skip,
//!     not a whole-run error).
//! (c) SCHEMA/guard assertions (no real sealed RRA needed):
//!     - SCHEMA_VERSION_PUBLIC >= 224 (I-30 SftExhaustEvent variant epoch).
//!     - "sft_exhaust_event" round-trips through ssb_action_type_from_str_public
//!       / as_sql_str.
//!     - SftExhaustEventKind::PathExtracted.as_str() == "path_extracted"
//!       (and the other two arms).
//! (d) JSONL out: after run() on empty/non-verified cases, the out_dir
//!     JSONL either does not exist or is empty/whitespace (no spurious
//!     records for non-verified RRAs).

use elai_cli::sft_exhaust::SftExhaust;
use elai_orchestrator::ssb::{
    SsbActionType, SftExhaustEventKind, SCHEMA_VERSION_PUBLIC,
    ssb_action_type_from_str_public,
};
use std::sync::Mutex;
use tempfile::TempDir;

/// SftExhaust::run opens the SHARED global sessions_dir ssb.db; serialize the two
/// SSB-init tests so they never collide on the sqlite lock under cargo's parallel runner.
static SSB_INIT_GUARD: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// (c) SCHEMA_VERSION_PUBLIC >= 224 (I-30 SftExhaustEvent variant epoch).
// ---------------------------------------------------------------------------

#[test]
fn schema_version_is_at_least_224() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 224,
        "SCHEMA_VERSION_PUBLIC must be >= 224 after I-30; got {}",
        SCHEMA_VERSION_PUBLIC
    );
}

// ---------------------------------------------------------------------------
// (c) Action-type lock: "sft_exhaust_event" round-trips.
// ---------------------------------------------------------------------------

#[test]
fn sft_exhaust_event_action_type_round_trips() {
    // str -> enum
    let parsed = ssb_action_type_from_str_public("sft_exhaust_event")
        .expect("wire string 'sft_exhaust_event' must parse");
    assert_eq!(
        parsed,
        SsbActionType::SftExhaustEvent,
        "parsed action type must be SftExhaustEvent"
    );
    // enum -> str
    assert_eq!(
        SsbActionType::SftExhaustEvent.as_sql_str(),
        "sft_exhaust_event",
        "SsbActionType::SftExhaustEvent wire tag mismatch"
    );
}

// ---------------------------------------------------------------------------
// (c) SftExhaustEventKind::as_str() for all three arms.
// ---------------------------------------------------------------------------

#[test]
fn sft_exhaust_event_kind_as_str_all_arms() {
    assert_eq!(
        SftExhaustEventKind::PathExtracted.as_str(),
        "path_extracted"
    );
    assert_eq!(
        SftExhaustEventKind::NegativePaired.as_str(),
        "negative_paired"
    );
    assert_eq!(
        SftExhaustEventKind::DatasetSealed.as_str(),
        "dataset_sealed"
    );
}

// ---------------------------------------------------------------------------
// (a) EMPTY rra_dir: run() Ok, zero counts except dataset_sealed=1.
// ---------------------------------------------------------------------------

#[test]
fn empty_rra_dir_runs_clean_and_seals() {
    let _ssb_guard = SSB_INIT_GUARD.lock().unwrap_or_else(|e| e.into_inner());
    let rra_dir = TempDir::new().expect("tempdir rra");
    let out_dir = TempDir::new().expect("tempdir out");

    let report = SftExhaust::new(
        rra_dir.path().to_path_buf(),
        out_dir.path().to_path_buf(),
    )
    .run()
    .expect("empty rra_dir run must Ok");

    assert_eq!(report.path_extracted, 0, "no RRAs -> path_extracted=0");
    assert_eq!(report.skipped_unsealed, 0, "no RRAs -> skipped_unsealed=0");
    assert_eq!(
        report.dataset_sealed, 1,
        "final DatasetSealed event still emits -> dataset_sealed=1"
    );
    assert_eq!(report.negative_paired, 0);

    // (d) JSONL: no verified RRAs -> no JSONL records. The file may not
    // exist at all (no append-opened path) or be empty/whitespace.
    assert_jsonl_empty_or_absent(out_dir.path());
}

// ---------------------------------------------------------------------------
// (b) NON-VERIFIED / malformed .rra: skipped_unsealed >= 1, path_extracted=0.
// ---------------------------------------------------------------------------

#[test]
fn bogus_rra_is_skipped_not_errored() {
    let _ssb_guard = SSB_INIT_GUARD.lock().unwrap_or_else(|e| e.into_inner());
    let rra_dir = TempDir::new().expect("tempdir rra");
    let out_dir = TempDir::new().expect("tempdir out");

    // Write a bogus .rra — random bytes, NOT a valid sealed RRA.
    std::fs::write(rra_dir.path().join("bogus.rra"), b"not a real rra")
        .expect("write bogus rra");

    let report = SftExhaust::new(
        rra_dir.path().to_path_buf(),
        out_dir.path().to_path_buf(),
    )
    .run()
    .expect("malformed RRA must not error the whole run");

    assert_eq!(
        report.path_extracted, 0,
        "non-verified RRA -> path_extracted=0"
    );
    assert!(
        report.skipped_unsealed >= 1,
        "malformed RRA must be skipped: skipped_unsealed >= 1, got {}",
        report.skipped_unsealed
    );
    assert_eq!(
        report.dataset_sealed, 1,
        "final DatasetSealed still emits even with all skips"
    );

    // (d) JSONL: no verified RRAs -> no spurious records.
    assert_jsonl_empty_or_absent(out_dir.path());
}

// ---------------------------------------------------------------------------
// (d) Helper: assert out_dir JSONL is absent or empty/whitespace.
// ---------------------------------------------------------------------------

fn assert_jsonl_empty_or_absent(out_dir: &std::path::Path) {
    let mut found_jsonl = false;
    if out_dir.exists() {
        for entry in std::fs::read_dir(out_dir).expect("read out_dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonl") {
                found_jsonl = true;
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_default();
                let trimmed = content.trim();
                assert!(
                    trimmed.is_empty(),
                    "JSONL {} must be empty/whitespace for non-verified RRAs; got: {:?}",
                    path.display(),
                    trimmed
                );
            }
        }
    }
    // It's fine if no .jsonl exists at all (no verified RRA -> no append-open).
    let _ = found_jsonl;
}