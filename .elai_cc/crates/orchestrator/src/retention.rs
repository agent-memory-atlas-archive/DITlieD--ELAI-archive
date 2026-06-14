//! I-44 — zero-retention-after-verification export mode.
//!
//! Once a run reaches a sealed verdict (the G-2 RRA + G-8 attestation have
//! committed), an operator may optionally PURGE the raw intermediate material
//! the run retained — raw prompt bytes, raw tool stdin/stdout, raw model I/O
//! held in the session's managed-transcript files — leaving only the redacted,
//! hash-checkable evidence (the G-2 evidence manifest + blake3 root and the G-8
//! attestation, which are NEVER touched). The invariant: after a
//! [`ExportRetentionMode::ZeroAfterVerify`] run, a grep over the exported bundle
//! for raw-input markers returns EMPTY, AND `elai replay --verify` STILL PASSES
//! on the hash-checkable steps (verification does not depend on the purged raw
//! bytes).
//!
//! ## Safety posture (this module DELETES files at runtime)
//! - FAIL-SAFE DEFAULT: [`ExportRetentionMode::Full`] retains everything and is
//!   the default. The purge fires ONLY when the operator passes the user-facing
//!   `--zero-retention` flag AND the destructive feature is enabled by the
//!   default-OFF [`ZERO_RETENTION_ENV`] kill-switch (R9, registered in
//!   `wire-dark-allowlist.toml`). No accidental deletion path.
//! - TIGHTLY SCOPED: the immediate verdict-driven purge deletes ONLY the active
//!   session's own managed-transcript files under its staging dir. Every
//!   candidate path is canonicalized and verified to live UNDER the session dir
//!   before any unlink; a path that escapes scope is REFUSED (kept), never
//!   deleted. The append-only SSB WAL (`ssb.db`) is NEVER touched — retention is
//!   file-granularity, not a row UPDATE/DELETE (the G-9 rationale).
//! - G-9 PRECONDITION: each retained-summary payload is passed through
//!   [`crate::redaction::redact_payload`] before the retained-set blake3 is
//!   sealed, so any secret in a RETAINED summary is masked first (an unredacted
//!   retained summary is a P0).
//! - ENTROPY-QUARANTINE: `now_ms` is injected (pass `clock.now_ms()` from an
//!   [`elai_core::clock::ClockProvider`]); this module never calls
//!   `SystemTime::now` on the purge path.
//!
//! Composes (does NOT re-implement — GD-1): G-9 `redaction::redact_payload` +
//! `redaction::purge_expired` (the TTL sweep leg) + the F-13 archive shape +
//! the G-2 RRA / G-8 attestation evidence that survives.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::redaction::{redact_payload, purge_expired, RetentionPolicy};
use crate::ssb::{RetentionPurgedKind, RetentionPurgedPayload, Ssb, SsbPayload};

/// Default-OFF kill-switch master gate for the destructive zero-retention purge.
/// Registered in `wire-dark-allowlist.toml` (R9, type=kill_switch). The
/// user-facing `--zero-retention` run flag selects the mode; this env var is the
/// independent master enable that must ALSO be truthy before any file is
/// deleted. Truthy values: `1` / `true` / `yes` / `on` (case-insensitive).
/// Single source of truth = the SSB-module const (keeps the schema doc + the
/// gate read on the SAME literal).
pub const ZERO_RETENTION_ENV: &str = crate::ssb::ELAI_ZERO_RETENTION_ENABLE_ENV;

/// Managed-session transcript extensions eligible for the immediate purge.
/// Mirrors `redaction::TRANSCRIPT_EXTENSIONS` (the same `.jsonl` / `.json`
/// managed-session class); inlined because that const is module-private.
const TRANSCRIPT_EXTENSIONS: [&str; 2] = ["jsonl", "json"];

/// Agent id stamped on the emitted RetentionPurged SSB rows.
const RETENTION_AGENT_ID: &str = "cli:run-retention";

/// Per-run retention policy selected by the operator. RENAMED from the plan's
/// literal `RetentionPolicy` to avoid colliding with the pre-existing G-9
/// [`crate::redaction::RetentionPolicy`] TTL-tier struct (same crate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportRetentionMode {
    /// Retain everything (the fail-safe default — behaviour unchanged).
    #[default]
    Full,
    /// After the sealed verdict commits, purge the active session's raw
    /// intermediate material; keep only the redacted hash-checkable evidence.
    ZeroAfterVerify,
}

impl ExportRetentionMode {
    /// Resolve the mode from the parsed `--zero-retention` CLI flag. `false`
    /// (flag absent) is the fail-safe [`ExportRetentionMode::Full`] default.
    #[must_use]
    pub fn from_cli_flag(zero_retention: bool) -> Self {
        if zero_retention {
            Self::ZeroAfterVerify
        } else {
            Self::Full
        }
    }
}

/// `true` iff the destructive zero-retention master kill-switch is enabled.
/// Default OFF. Independent of the `--zero-retention` flag: BOTH must be set
/// before any file is deleted (defence in depth on an irreversible op).
#[must_use]
pub fn zero_retention_enabled() -> bool {
    match std::env::var(ZERO_RETENTION_ENV) {
        Ok(raw) => {
            let v = raw.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        Err(_) => false,
    }
}

/// Outcome of one [`purge_after_verify`] sweep. Deterministic + secret-free;
/// suitable for an SSB row or a dashboard read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetentionReport {
    /// Whether the destructive purge ran at all (mode == ZeroAfterVerify AND the
    /// master kill-switch enabled). `false` => zero filesystem mutation.
    pub purged: bool,
    /// Active-session managed-transcript files deleted by the immediate leg.
    pub raw_inputs_deleted: Vec<PathBuf>,
    /// Candidate paths REFUSED because they escaped the session-dir scope guard.
    pub refused_out_of_scope: Vec<PathBuf>,
    /// blake3 (hex) over the retained, redacted evidence set. Empty when nothing
    /// was retained / the purge did not run.
    pub retained_blake3: String,
    /// Number of retained-summary payloads passed through G-9 redaction.
    pub retained_redacted_count: usize,
}

/// `true` iff `path`'s extension is a managed-session transcript (`.jsonl` /
/// `.json`). The guard that keeps the purge from ever considering `ssb.db`.
fn is_managed_transcript(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| TRANSCRIPT_EXTENSIONS.iter().any(|known| *known == ext))
}

/// `true` iff `candidate` resolves to a path strictly UNDER `scope_dir`. Both are
/// canonicalized first; a candidate that fails to canonicalize, or that resolves
/// outside `scope_dir`, returns `false` (REFUSED — fail-safe). This is the
/// blast-radius guard: the immediate purge will only ever unlink a file this
/// returns `true` for.
fn is_within_scope(candidate: &Path, scope_dir: &Path) -> bool {
    let scope = match scope_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };
    match candidate.canonicalize() {
        Ok(c) => c.starts_with(&scope),
        Err(_) => false,
    }
}

/// Post-seal retention driver. Called from the `elai run` seal block AFTER the
/// G-2 RRA + G-8 attestation have committed.
///
/// In [`ExportRetentionMode::ZeroAfterVerify`] (and only when
/// [`zero_retention_enabled`] is also true) this:
///   1. reads the session's retained-summary payloads and runs each through
///      G-9 [`redact_payload`] (the precondition — mask before sealing);
///   2. computes a blake3 over the redacted retained set (the survivable
///      evidence digest) and emits `RetentionPurged(EvidenceRetained)`;
///   3. deletes the active session's own managed-transcript files under its
///      staging dir — each path validated in-scope first (out-of-scope =
///      REFUSED) — and emits `RetentionPurged(RawInputsPurged)`;
///   4. runs the G-9 TTL sweep [`purge_expired`] over OTHER expired sessions
///      (its own `ELAI_RETENTION_PURGE_ENABLE` gate; it SKIPS the active
///      session by design) — the first production caller of that primitive;
///   5. emits `RetentionPurged(PurgeVerified)` recording the retained digest.
///
/// FAIL-SAFE: in [`ExportRetentionMode::Full`], or when the master kill-switch
/// is off, this performs ZERO filesystem mutation and returns a report with
/// `purged = false`. Errors from individual SSB writes are surfaced; a refused
/// out-of-scope path is recorded, never deleted.
pub async fn purge_after_verify(
    ssb: &Arc<Ssb>,
    session_id: &str,
    project_root: &Path,
    mode: ExportRetentionMode,
    now_ms: u128,
) -> Result<RetentionReport, anyhow::Error> {
    let mut report = RetentionReport::default();

    // FAIL-SAFE GATE: only ZeroAfterVerify + master kill-switch deletes anything.
    if mode != ExportRetentionMode::ZeroAfterVerify || !zero_retention_enabled() {
        return Ok(report);
    }
    report.purged = true;

    let session_dir = elai_core::paths::sessions_dir_path(project_root).join(session_id);

    // 1. G-9 precondition: redact every retained-summary payload, then seal a
    //    blake3 over the REDACTED retained set (the evidence that survives).
    let retained = redacted_retained_digest(ssb, session_id, &mut report)?;
    report.retained_blake3 = retained.clone();

    emit_retention_row(
        ssb,
        RetentionPurgedKind::EvidenceRetained,
        Vec::new(),
        retained.clone(),
        session_id,
        now_ms,
    )
    .await?;

    // 2. Immediate verdict-driven purge of the ACTIVE session's raw transcripts,
    //    strictly scoped to its own staging dir.
    purge_active_session_transcripts(&session_dir, &mut report);

    let purged_classes: Vec<String> = report
        .raw_inputs_deleted
        .iter()
        .filter_map(|p| p.extension().and_then(|e| e.to_str()).map(|s| s.to_owned()))
        .collect();
    emit_retention_row(
        ssb,
        RetentionPurgedKind::RawInputsPurged,
        purged_classes,
        retained.clone(),
        session_id,
        now_ms,
    )
    .await?;

    // 3. CLOSE the dark `purge_expired`: TTL sweep over OTHER expired sessions
    //    (its own ELAI_RETENTION_PURGE_ENABLE gate; SKIPS the active session).
    //    Best-effort — its internal gate fail-closes when not enabled.
    let sessions_dir = elai_core::paths::sessions_dir_path(project_root);
    let _ttl = purge_expired(
        &sessions_dir,
        &sessions_dir,
        RetentionPolicy::default(),
        now_ms,
        session_id,
        project_root,
    );

    // 4. Verdict beacon: the raw bytes are gone, the redacted evidence digest
    //    survives and is verifiable.
    emit_retention_row(
        ssb,
        RetentionPurgedKind::PurgeVerified,
        Vec::new(),
        retained,
        session_id,
        now_ms,
    )
    .await?;

    tracing::info!(
        session_id = %session_id,
        raw_inputs_deleted = report.raw_inputs_deleted.len(),
        refused_out_of_scope = report.refused_out_of_scope.len(),
        retained_redacted = report.retained_redacted_count,
        retained_blake3 = %report.retained_blake3,
        "I-44: zero-retention purge complete; RetentionPurged rows written (raw bytes purged, redacted evidence retained)"
    );

    Ok(report)
}

/// Read the session stream, run G-9 [`redact_payload`] over each retained
/// payload, and return the blake3 (hex) over the redacted, re-serialized set.
/// This both ENFORCES the redaction precondition and produces the survivable
/// evidence digest. `redact_payload` is invoked here — this is its first real
/// production caller (I-44 closes it from DARK to WIRED).
fn redacted_retained_digest(
    ssb: &Arc<Ssb>,
    session_id: &str,
    report: &mut RetentionReport,
) -> Result<String, anyhow::Error> {
    let stream = ssb
        .query_session_stream_asc(session_id)
        .map_err(|e| anyhow::anyhow!("I-44: reading session stream for retained digest: {e}"))?;

    let mut hasher = blake3::Hasher::new();
    let mut redacted_count = 0usize;
    for entry in &stream {
        // Decode, redact in place (the G-9 precondition), re-serialize, hash.
        let mut payload: SsbPayload = match rmp_serde::from_slice(&entry.payload) {
            Ok(p) => p,
            Err(_) => {
                // Undecodable payload: hash the raw bytes as-is so the digest is
                // still total + deterministic (never silently dropped, R1).
                hasher.update(&entry.payload);
                continue;
            }
        };
        let red = redact_payload(&mut payload);
        if red.fired() {
            redacted_count += 1;
        }
        match rmp_serde::to_vec_named(&payload) {
            Ok(bytes) => {
                hasher.update(&bytes);
            }
            Err(_) => {
                hasher.update(&entry.payload);
            }
        }
    }
    report.retained_redacted_count = redacted_count;
    Ok(hasher.finalize().to_hex().to_string())
}

/// Delete the active session's own managed-transcript files under `session_dir`.
/// Every candidate is validated in-scope ([`is_within_scope`]) before the
/// unlink; a path that escapes scope is recorded in `refused_out_of_scope` and
/// KEPT. Never enumerates or touches `ssb.db`.
fn purge_active_session_transcripts(session_dir: &Path, report: &mut RetentionReport) {
    let entries = match std::fs::read_dir(session_dir) {
        Ok(e) => e,
        Err(_) => return, // no staging dir yet — nothing to purge
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !is_managed_transcript(&path) {
            continue; // never ssb.db — only managed-session transcripts
        }
        // BLAST-RADIUS GUARD: refuse anything that does not resolve strictly
        // under the session dir (symlink escape, traversal) — fail-safe KEEP.
        if !is_within_scope(&path, session_dir) {
            report.refused_out_of_scope.push(path);
            continue;
        }
        match std::fs::remove_file(&path) {
            Ok(()) => report.raw_inputs_deleted.push(path),
            // A delete failure is non-fatal: the file is KEPT, recorded as
            // out-of-scope-style diagnostic (we did not delete it).
            Err(_) => report.refused_out_of_scope.push(path),
        }
    }
}

/// Emit ONE `RetentionPurged` SSB row through the LIVE run handle (R4: a real
/// run entrypoint sealed the row, off-author artifact).
async fn emit_retention_row(
    ssb: &Arc<Ssb>,
    kind: RetentionPurgedKind,
    purged_classes: Vec<String>,
    retained_blake3: String,
    session_id: &str,
    now_ms: u128,
) -> Result<(), anyhow::Error> {
    let ts_ms = i64::try_from(now_ms).unwrap_or(i64::MAX);
    let payload = RetentionPurgedPayload {
        kind,
        purged_classes,
        retained_blake3,
        session_id: session_id.to_owned(),
        ts_ms,
    };
    ssb.write_payload(
        RETENTION_AGENT_ID,
        0,
        SsbPayload::RetentionPurged(payload),
        Vec::new(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("I-44: writing retention_purged({}) row: {e}", kind.as_sql_str()))?;
    Ok(())
}
