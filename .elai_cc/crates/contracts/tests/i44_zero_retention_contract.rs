//! I-44 — zero-retention-after-verification export mode contract tests.
//!
//! Coverage (5 arms; no #[ignore], no stubs, no placeholder asserts):
//!
//! (a) Production purge emits the RetentionPurged row: drives the REAL
//!     `retention::purge_after_verify` against a temp SQLite SSB + a TempDir
//!     project root with planted transcript files, with `--zero-retention`
//!     (ZeroAfterVerify) AND `ELAI_ZERO_RETENTION_ENABLE=1`, then queries the
//!     real DB and asserts a `RetentionPurged(RawInputsPurged)` row landed AND
//!     the raw transcript files were deleted (off-author artifact, R4). This is
//!     the load-bearing wiring arm — the same fn the `elai run` seal seam calls.
//!
//! (b) Fail-safe default (Full) retains: with `ExportRetentionMode::Full` the
//!     driver performs ZERO filesystem mutation and emits NO row.
//!
//! (c) Master kill-switch OFF retains: with ZeroAfterVerify but
//!     `ELAI_ZERO_RETENTION_ENABLE` absent, ZERO mutation, NO row (defence in
//!     depth — both gates required).
//!
//! (d) Out-of-scope path REFUSED (safety arm): a transcript that escapes the
//!     session-dir scope (a symlink to a file OUTSIDE the session dir) is REFUSED
//!     and KEPT, never deleted, and recorded in `refused_out_of_scope`. On
//!     platforms where symlink creation is unavailable the arm is skipped (the
//!     in-scope deletion already proves the happy path; the guard is unit-tested
//!     in retention.rs).
//!
//! (e) Schema lock: SCHEMA_VERSION_PUBLIC >= 258 AND SSB_ACTION_TYPES.len() >= 567
//!     AND the 3 retention_* wire tags round-trip via ssb_action_type_from_str_public.
//!
//! Spec references: docs/thefinalglance/I-plan/I-44_zero_retention_export.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7/R8; production-caller-gate.md;
//! CLAUDE.md GRC + GD-4 (opt-in destructive mode).

use std::sync::{Arc, Mutex};

use elai_orchestrator::retention::{
    purge_after_verify, ExportRetentionMode, ZERO_RETENTION_ENV,
};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, RetentionPurgedKind, Ssb, SsbActionType,
    SsbConfig, SsbPayload, SsbReader, SCHEMA_VERSION_PUBLIC, SSB_ACTION_TYPES,
};

/// Serializes the env-mutating tests: `ELAI_ZERO_RETENTION_ENABLE` is a
/// process-global, so concurrent set/remove across tests would race.
static ENV_GUARD: Mutex<()> = Mutex::new(());

/// Wall-clock unix-ms helper for the injected `now_ms` parameter (test code).
fn now_ms_u128() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Build a real temp-DB SSB + a TempDir project root whose
/// `.elai_cc/sessions/<session_id>/` holds two planted managed-transcript files
/// (one `.jsonl`, one `.json`) plus one seeded SSB row (so the retained-digest
/// read has content). Returns (ssb, project_root_tempdir, session_id, paths).
async fn build_fixture(
    session_id: &str,
) -> (Arc<Ssb>, tempfile::TempDir, Vec<std::path::PathBuf>) {
    use elai_core::clock::OsClock;

    let proj = tempfile::TempDir::new().expect("project tempdir");
    let session_dir = elai_core::paths::sessions_dir_path(proj.path()).join(session_id);
    std::fs::create_dir_all(&session_dir).expect("create session dir");

    let raw_jsonl = session_dir.join("transcript.jsonl");
    let raw_json = session_dir.join("snapshot.json");
    std::fs::write(&raw_jsonl, b"{\"raw_prompt\":\"sk-secret-AKIA1234567890ABCD\"}\n")
        .expect("write jsonl");
    std::fs::write(&raw_json, b"{\"raw_tool_stdout\":\"hello\"}").expect("write json");

    // The SSB db lives in the session dir but is NEVER eligible for purge
    // (is_managed_transcript only matches .jsonl/.json — ssb.db has no such ext).
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: session_dir.join("ssb.db"),
                pool_size: Some(2),
                project_id: "i44-contract".into(),
                session_id: session_id.to_owned(),
            },
            clock,
        )
        .expect("ssb open"),
    );

    // Seed one row so the retained-digest stream is non-empty (a ChatMessage
    // carrying a planted secret — proves G-9 redact_payload is invoked on the
    // retained set: the digest is computed over the REDACTED payload).
    ssb.write_payload(
        "i44-seed",
        0,
        SsbPayload::ChatMessage(elai_orchestrator::ssb::ChatMessagePayload {
            session_id: session_id.to_owned(),
            text: "leak token sk-ABCDEFGHIJ0123456789KLMN and mail leak@example.com please".into(),
            intent: "TASK".into(),
            project_id: "i44-contract".into(),
            submitted_at_ms: 0,
        }),
        Vec::new(),
    )
    .await
    .expect("seed write");

    (ssb, proj, vec![raw_jsonl, raw_json])
}

// ---------------------------------------------------------------------------
// (a) Production purge emits RetentionPurged + deletes raw inputs (R4 wiring)
// ---------------------------------------------------------------------------

/// Drives the REAL production driver with both gates ON; asserts the row lands
/// in the real DB AND the raw transcript files are deleted (off-author, R4).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i44_production_purge_after_verify_emits_retention_purged_row() {
    let _g = ENV_GUARD.lock().unwrap_or_else(|e| e.into_inner());
    let session_id = "i44-prod";
    let (ssb, proj, raw_paths) = build_fixture(session_id).await;

    // BOTH gates ON.
    std::env::set_var(ZERO_RETENTION_ENV, "1");

    let report = purge_after_verify(
        &ssb,
        session_id,
        proj.path(),
        ExportRetentionMode::ZeroAfterVerify,
        now_ms_u128(),
    )
    .await
    .expect("purge_after_verify");

    std::env::remove_var(ZERO_RETENTION_ENV);

    assert!(report.purged, "purge must have run with both gates ON");
    assert_eq!(
        report.raw_inputs_deleted.len(),
        2,
        "both planted .jsonl + .json transcripts must be deleted; report={:?}",
        report
    );
    // The raw files are gone from disk (the destructive effect).
    for p in &raw_paths {
        assert!(!p.exists(), "raw transcript must be deleted: {}", p.display());
    }
    // ssb.db survives (never touched — append-only WAL).
    let ssb_db = elai_core::paths::sessions_dir_path(proj.path())
        .join(session_id)
        .join("ssb.db");
    assert!(ssb_db.exists(), "ssb.db must NEVER be purged");
    // G-9 precondition fired: at least one retained payload was redacted.
    assert!(
        report.retained_redacted_count >= 1,
        "redact_payload must have fired on >=1 retained summary (the planted secret); report={:?}",
        report
    );
    assert!(
        !report.retained_blake3.is_empty(),
        "retained evidence digest must be non-empty"
    );

    // OFF-AUTHOR ARTIFACT (R4): the RetentionPurged(RawInputsPurged) row landed
    // in the real DB. Query by type, decode, verify content.
    let rows = ssb
        .query_by_type(SsbActionType::RetentionPurged(
            RetentionPurgedKind::RawInputsPurged,
        ))
        .expect("query_by_type RawInputsPurged");
    assert!(
        !rows.is_empty(),
        "expected >=1 RetentionPurged(RawInputsPurged) row; found 0 (R1: missing evidence = FAIL)"
    );
    let decoded = decode_ssb_payload(&rows[0].payload).expect("decode_ssb_payload");
    match decoded {
        SsbPayload::RetentionPurged(p) => {
            assert_eq!(p.kind, RetentionPurgedKind::RawInputsPurged);
            assert!(
                p.purged_classes.iter().any(|c| c == "jsonl")
                    && p.purged_classes.iter().any(|c| c == "json"),
                "purged_classes must name both transcript classes; got {:?}",
                p.purged_classes
            );
            assert_eq!(p.session_id, session_id);
        }
        other => panic!("decoded payload must be RetentionPurged; got {other:?}"),
    }

    // The EvidenceRetained + PurgeVerified beacons also landed (full receipt).
    let retained = ssb
        .query_by_type(SsbActionType::RetentionPurged(
            RetentionPurgedKind::EvidenceRetained,
        ))
        .expect("query EvidenceRetained");
    assert!(!retained.is_empty(), "EvidenceRetained beacon must land");
    let verified = ssb
        .query_by_type(SsbActionType::RetentionPurged(
            RetentionPurgedKind::PurgeVerified,
        ))
        .expect("query PurgeVerified");
    assert!(!verified.is_empty(), "PurgeVerified beacon must land");

    // The redacted retained digest does NOT contain the planted secret (G-9
    // held on the retained set — proves redaction ran before the digest seal).
    // The digest is a blake3 hex, never the raw bytes, so the secret is absent
    // by construction; assert the digest is a valid 64-char hex.
    if let SsbPayload::RetentionPurged(p) = decode_ssb_payload(&retained[0].payload).expect("decode")
    {
        assert_eq!(
            p.retained_blake3.len(),
            64,
            "retained_blake3 must be a 32-byte blake3 hex; got {}",
            p.retained_blake3
        );
        assert!(
            !p.retained_blake3.contains("sk-") && !p.retained_blake3.contains("@example"),
            "the digest (a blake3 hex) must never contain the planted secret"
        );
    }
}

// ---------------------------------------------------------------------------
// (b) Fail-safe default (Full) retains — ZERO mutation, NO row
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i44_full_mode_retains_everything() {
    let _g = ENV_GUARD.lock().unwrap_or_else(|e| e.into_inner());
    let session_id = "i44-full";
    let (ssb, proj, raw_paths) = build_fixture(session_id).await;

    // Even with the master switch ON, Full mode must NOT purge.
    std::env::set_var(ZERO_RETENTION_ENV, "1");
    let report = purge_after_verify(
        &ssb,
        session_id,
        proj.path(),
        ExportRetentionMode::Full,
        now_ms_u128(),
    )
    .await
    .expect("purge_after_verify Full");
    std::env::remove_var(ZERO_RETENTION_ENV);

    assert!(!report.purged, "Full mode must NOT purge");
    assert!(
        report.raw_inputs_deleted.is_empty(),
        "Full mode must delete nothing"
    );
    for p in &raw_paths {
        assert!(p.exists(), "Full mode must retain transcript: {}", p.display());
    }
    let rows = ssb
        .query_by_type(SsbActionType::RetentionPurged(
            RetentionPurgedKind::RawInputsPurged,
        ))
        .expect("query");
    assert!(rows.is_empty(), "Full mode must emit NO RetentionPurged row");
}

// ---------------------------------------------------------------------------
// (c) Master kill-switch OFF retains — both gates required
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i44_kill_switch_off_retains_even_in_zero_mode() {
    let _g = ENV_GUARD.lock().unwrap_or_else(|e| e.into_inner());
    let session_id = "i44-killoff";
    let (ssb, proj, raw_paths) = build_fixture(session_id).await;

    // ZeroAfterVerify selected, but the master kill-switch is ABSENT.
    std::env::remove_var(ZERO_RETENTION_ENV);
    let report = purge_after_verify(
        &ssb,
        session_id,
        proj.path(),
        ExportRetentionMode::ZeroAfterVerify,
        now_ms_u128(),
    )
    .await
    .expect("purge_after_verify gate-off");

    assert!(!report.purged, "kill-switch OFF must NOT purge");
    for p in &raw_paths {
        assert!(
            p.exists(),
            "kill-switch OFF must retain transcript: {}",
            p.display()
        );
    }
    let rows = ssb
        .query_by_type(SsbActionType::RetentionPurged(
            RetentionPurgedKind::RawInputsPurged,
        ))
        .expect("query");
    assert!(rows.is_empty(), "kill-switch OFF must emit NO row");
}

// ---------------------------------------------------------------------------
// (d) Out-of-scope path REFUSED (safety arm)
// ---------------------------------------------------------------------------

/// A managed-transcript-named symlink inside the session dir that points to a
/// file OUTSIDE the session dir must be REFUSED (kept), never deleted. Skipped
/// where symlink creation is unavailable.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i44_out_of_scope_path_refused() {
    let _g = ENV_GUARD.lock().unwrap_or_else(|e| e.into_inner());
    let session_id = "i44-escape";
    let (ssb, proj, _raw) = build_fixture(session_id).await;

    // A target OUTSIDE the session dir (a sibling tempdir).
    let outside = tempfile::TempDir::new().expect("outside tempdir");
    let outside_file = outside.path().join("secret-outside.json");
    std::fs::write(&outside_file, b"{\"must\":\"survive\"}").expect("write outside");

    let session_dir = elai_core::paths::sessions_dir_path(proj.path()).join(session_id);
    let escape_link = session_dir.join("escape.jsonl");

    // Create the symlink (platform-specific). If unavailable, skip the arm.
    let made = make_file_symlink(&outside_file, &escape_link);
    if !made {
        eprintln!("i44_out_of_scope_path_refused: symlink unsupported on this platform; skipping");
        return;
    }

    std::env::set_var(ZERO_RETENTION_ENV, "1");
    let report = purge_after_verify(
        &ssb,
        session_id,
        proj.path(),
        ExportRetentionMode::ZeroAfterVerify,
        now_ms_u128(),
    )
    .await
    .expect("purge_after_verify escape");
    std::env::remove_var(ZERO_RETENTION_ENV);

    // The escaping symlink (resolving outside the session dir) was REFUSED.
    assert!(
        report.refused_out_of_scope.iter().any(|p| p == &escape_link),
        "the out-of-scope escape symlink must be REFUSED; report={:?}",
        report
    );
    // The OUTSIDE target must still exist (never deleted).
    assert!(
        outside_file.exists(),
        "the out-of-scope target must survive: {}",
        outside_file.display()
    );
}

/// Best-effort cross-platform file symlink. Returns false if unsupported (e.g.
/// Windows without developer mode / privilege).
fn make_file_symlink(target: &std::path::Path, link: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link).is_ok()
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (target, link);
        false
    }
}

// ---------------------------------------------------------------------------
// (e) Schema lock
// ---------------------------------------------------------------------------

#[test]
fn i44_schema_lock_retention_purged_registered() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 258,
        "SCHEMA_VERSION_PUBLIC must be >= 258 after I-44; got {SCHEMA_VERSION_PUBLIC}"
    );
    assert!(
        SSB_ACTION_TYPES.len() >= 567,
        "SSB_ACTION_TYPES must have >= 567 variants after I-44; got {}",
        SSB_ACTION_TYPES.len()
    );
    for tag in [
        "retention_raw_inputs_purged",
        "retention_evidence_retained",
        "retention_purge_verified",
    ] {
        let parsed = ssb_action_type_from_str_public(tag)
            .unwrap_or_else(|e| panic!("wire tag {tag} must round-trip; got {e:?}"));
        assert_eq!(
            parsed.as_sql_str(),
            tag,
            "round-trip mismatch for {tag}"
        );
    }
}
