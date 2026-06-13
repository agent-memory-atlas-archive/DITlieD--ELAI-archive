//! I-39 — plan-state SSB migration contract.
//!
//! Verifies that the REAL archived SSB fixture from the F-13 session can be
//! opened and that every payload row survives `decode_ssb_payload` without
//! error. This is the Layer-3 migration check: an old DB (schema v205) must
//! remain decodable after the schema bumps introduced by I-39 (v230).
//!
//! R1/R2 compliance (wiring-enforcement.md):
//!   - Missing fixture file → FAIL loudly (not SKIP, not UNVERIFIED).
//!   - Zero rows decoded   → FAIL loudly (dead evidence path screams).
//!   - Any row decode error → FAIL immediately (no silent swallows).
//!
//! Uses rusqlite directly (NOT `Ssb::new`) to avoid the
//! `SSB_NEW_MUST_BE_BUILT_INSIDE_BLOCK_ON_MULTI_THREAD_RUNTIME` pitfall:
//! `Ssb::new` panics outside a tokio multi-thread runtime; direct rusqlite
//! read is safe in a standard sync test.

mod common;

use std::path::PathBuf;

use elai_orchestrator::ssb::decode_ssb_payload;
use rusqlite::{Connection, OpenFlags};

// ---------------------------------------------------------------------------
// Fixture path
// ---------------------------------------------------------------------------

fn fixture_path() -> PathBuf {
    // f13-drop-on has 1 real row (file_write, schema v205) — verified non-empty.
    // f13-enabled has a valid schema but zero rows (schema-only fixture, unsuitable
    // as a migration evidence source per R2).
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("workflow/sessions/f13-drop-on/archive/ssb-v205.sqlite")
}

// ---------------------------------------------------------------------------
// Contract: fixture is present, has rows, all rows decode
// ---------------------------------------------------------------------------

/// R1: The archived fixture exists (not SKIP; a missing fixture is FAIL).
#[test]
fn i39_fixture_exists() {
    let path = fixture_path();
    assert!(
        path.exists(),
        "I-39 migration contract: SSB fixture not found at {} — \
         the fixture is a required input, not optional; \
         its absence means the migration gate has no evidence (R1/R2 FAIL)",
        path.display()
    );
}

/// R2: The fixture has at least one row, and every row's payload round-trips
/// through `decode_ssb_payload` without error.
///
/// Failure modes detected:
///   (a) Zero rows in the fixture → dead evidence path (R2 FAIL).
///   (b) Any row returns `Err` from `decode_ssb_payload` → decode regression.
///   (c) rusqlite can't open the file → file is corrupt or wrong path.
#[test]
fn i39_all_rows_decode_without_error() {
    let path = fixture_path();

    // R1: fixture must exist (duplicate of i39_fixture_exists for standalone run).
    assert!(
        path.exists(),
        "I-39: fixture not found at {} (R1 FAIL — never SKIP)",
        path.display()
    );

    // Open read-only — no write must ever touch the archived fixture.
    let conn = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .unwrap_or_else(|e| {
        panic!(
            "I-39: failed to open fixture {} read-only: {e}",
            path.display()
        )
    });

    // Fetch all payload bytes from the ssb table.
    let mut stmt = conn
        .prepare("SELECT seq, action_type, payload FROM ssb ORDER BY seq ASC")
        .unwrap_or_else(|e| panic!("I-39: prepare SELECT: {e}"));

    let rows: Vec<(i64, String, Vec<u8>)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })
        .unwrap_or_else(|e| panic!("I-39: query_map: {e}"))
        .map(|r| r.unwrap_or_else(|e| panic!("I-39: row error: {e}")))
        .collect();

    // R2: dead evidence path must scream (R2 FAIL, not SKIP).
    assert!(
        !rows.is_empty(),
        "I-39: fixture {} has ZERO rows — the gate has no evidence to evaluate; \
         this is a wiring-enforcement R2 FAIL, not an empty-file OK",
        path.display()
    );

    // Decode every payload. Any error = decode regression (no silent swallows).
    let total = rows.len();
    let mut decode_errors: Vec<String> = Vec::new();
    for (seq, action_type, payload_bytes) in &rows {
        match decode_ssb_payload(payload_bytes) {
            Ok(_) => { /* pass */ }
            Err(e) => {
                decode_errors.push(format!(
                    "  seq={seq} action_type={action_type} error={e}"
                ));
            }
        }
    }

    assert!(
        decode_errors.is_empty(),
        "I-39: {}/{total} rows failed decode_ssb_payload (migration regression):\n{}",
        decode_errors.len(),
        decode_errors.join("\n")
    );
}
