//! E-25 Tranche C (C.4) -- edit_checkpoints (Primitive 6b).
//!
//! Pre-edit snapshot store + rewind path. Snapshots are written under
//! `.elai_cc/.session-checkpoints/{session_id}/{seq:06}/{blake3_prefix:8}/{path}`
//! before any Edit / Write tool call so the operator can recover by
//! running `elai checkpoint rewind --steps N`.
//!
//! Trigger site: `HookPoint::PreToolUse` (hook_dispatcher.rs:1603)
//! filtered to `tool_name in {"Edit", "Write", "NotebookEdit"}`. The
//! filter is enforced by the caller; this module is a pure storage +
//! query surface.
//!
//! Storage backend: on-disk content-addressed. The blake3 prefix
//! folder makes content-identical snapshots dedup-free; the seq
//! counter (zero-padded to 6 digits for stable lex sort) is the
//! monotonic per-session id. Append-only.
//!
//! Hashing: blake3 ONLY (CLAUDE.md HARD RULE Anti-Stale-Training;
//! DETERMINISTIC_CHECK #9 enforces zero blake2 / sha2 hits).
//! Workspace dep already cleared at Cargo.toml:56 +
//! orchestrator/Cargo.toml:82 + Cargo.lock:265.
//!
//! Emission: reuses the SHIPPED
//! [`crate::architecture::claude_code_arch::ClaudeCodeArchitectureEventKind::EditCheckpointSnapshotted`]
//! arm (claude_code_arch.rs:262). For rewind events, the same
//! `EditCheckpointSnapshotted` arm is emitted with
//! `details_json.rewind_event = true` -- the 9-arm Kind enum is
//! LOCKED per session-4 DECISION-5 (no 10th arm) so the rewind
//! discriminator rides on the free-form `details_json` field per
//! Pattern-2 lean.
//!
//! Wire-dark default OFF; gated by
//! [`ELAI_E25_EDIT_CHECKPOINTS_ENABLE_VAR`].

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use elai_core::clock::{ClockProvider, OsClock};

use crate::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    ClaudeCodeArchitectureEventPayload,
};

/// Environment-variable name controlling the wire-dark gate. Set to a
/// truthy value (`"1"` / `"true"` / `"yes"` / `"on"`) to enable.
/// Default OFF.
pub const ELAI_E25_EDIT_CHECKPOINTS_ENABLE_VAR: &str = "ELAI_E25_EDIT_CHECKPOINTS_ENABLE";

/// Root directory under the workspace where session checkpoints live.
/// Relative to the current working directory at runtime; CLI surface
/// resolves it from the project root.
pub const SESSION_CHECKPOINT_ROOT: &str = ".elai_cc/.session-checkpoints";

/// Length of the blake3 prefix (hex chars) embedded in the snapshot
/// directory path. Tuned so collisions over a single session are
/// negligible while keeping the directory name short.
pub const BLAKE3_PREFIX_HEX_LEN: usize = 8;

/// Zero-pad width for the per-session sequence counter. 6 digits
/// supports up to 1M snapshots per session before overflow; way above
/// any realistic Claude Code session length.
pub const SEQ_DIGITS: usize = 6;

/// Zero-pad width for the per-snapshot subsec-nanos tie-break (s-22600
/// Tranche G G.2 -- audit WARN-2 followup). On-disk seq directory name
/// is `{seq:06}_{nanos:09}` -- the `_{nanos:09}` suffix ensures two
/// concurrent writers that both observe `max_seen = N` and both compute
/// `seq = N+1` land in DISTINCT directories. 9 digits hold the full
/// `Duration::subsec_nanos()` range (`0..=999_999_999`).
pub const NANOS_DIGITS: usize = 9;

/// Default eviction policy: keep the last N snapshots per session.
/// Full evict happens at session END (caller responsibility).
pub const DEFAULT_LAST_N_KEPT: usize = 10;

/// Errors surfaced by the snapshot / rewind path. Uses `thiserror` per
/// CLAUDE.md HARD RULE (no `Box<dyn Error>`).
#[derive(Debug, thiserror::Error)]
pub enum EditCheckpointError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("checkpoint root has no session entries for session `{session_id}`")]
    NoSnapshots { session_id: String },
    #[error("requested seq `{seq:06}` not found for session `{session_id}`")]
    SeqNotFound { session_id: String, seq: u64 },
    #[error("invalid session id `{session_id}`: must not be empty or contain path separators")]
    InvalidSessionId { session_id: String },
}

/// Returns true iff the edit_checkpoints subsystem is wire-enabled.
#[must_use]
pub fn edit_checkpoints_enabled() -> bool {
    matches_truthy_env(ELAI_E25_EDIT_CHECKPOINTS_ENABLE_VAR)
}

fn matches_truthy_env(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let trimmed = v.trim();
            trimmed.eq_ignore_ascii_case("1")
                || trimmed.eq_ignore_ascii_case("true")
                || trimmed.eq_ignore_ascii_case("yes")
                || trimmed.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// Metadata for a stored snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditCheckpoint {
    pub session_id: String,
    pub seq: u64,
    /// s-22600 Tranche G G.2: subsec-nanos tie-break (audit WARN-2
    /// followup). Embedded in the on-disk directory name as
    /// `{seq:06}_{nanos:09}` so two concurrent writers observing the
    /// same `max_seen` land in distinct directories. Legacy
    /// `{seq:06}` directories (pre-G.2 snapshots) read back with
    /// `nanos = 0` via [`parse_seq_dirname`].
    pub nanos: u64,
    pub blake3_prefix: String,
    /// Original source path (snapshotted before edit).
    pub path: PathBuf,
    /// On-disk location of the snapshot copy.
    pub snapshot_path: PathBuf,
    /// Snapshot byte size.
    pub size_bytes: u64,
    /// Unix-millis when the snapshot was created.
    pub timestamp_ms: i64,
}

/// Outcome of a [`rewind_to`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewindOutcome {
    pub session_id: String,
    pub target_seq: u64,
    /// Snapshots restored in this run, in restoration order
    /// (highest-seq first -> oldest-seq last so a mid-rewind crash
    /// leaves the most recent file written).
    pub restored: Vec<EditCheckpoint>,
}

fn validate_session_id(session_id: &str) -> Result<(), EditCheckpointError> {
    if session_id.is_empty()
        || session_id.contains(std::path::MAIN_SEPARATOR)
        || session_id.contains('/')
        || session_id.contains('\\')
    {
        return Err(EditCheckpointError::InvalidSessionId {
            session_id: session_id.to_owned(),
        });
    }
    Ok(())
}

fn session_root(root: &Path, session_id: &str) -> PathBuf {
    root.join(session_id)
}

fn seq_dir(root: &Path, session_id: &str, seq: u64, nanos: u64) -> PathBuf {
    let mut p = session_root(root, session_id);
    p.push(format!(
        "{seq:0sw$}_{nanos:0nw$}",
        sw = SEQ_DIGITS,
        nw = NANOS_DIGITS,
    ));
    p
}

/// s-22600 Tranche G G.2: parse a `{seq:06}` or `{seq:06}_{nanos:09}`
/// subdir name into `(seq, nanos)`. Returns `None` for any name that
/// does not begin with a parseable u64. Legacy directories that don't
/// carry the `_{nanos}` suffix (pre-G.2 snapshots) parse as
/// `(seq, 0)` -- this keeps reading legacy on-disk state working
/// across the upgrade.
fn parse_seq_dirname(name: &str) -> Option<(u64, u64)> {
    let mut parts = name.splitn(2, '_');
    let seq_str = parts.next()?;
    let seq = seq_str.parse::<u64>().ok()?;
    let nanos = parts
        .next()
        .and_then(|n| n.parse::<u64>().ok())
        .unwrap_or(0);
    Some((seq, nanos))
}

fn current_unix_millis() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

/// Encode an arbitrary source path into a filename-safe label so it
/// can live inside the snapshot directory. Replaces path separators
/// with `__` and strips any leading separators. Round-trip is NOT
/// required -- callers re-derive the original path from the
/// EditCheckpoint metadata.
fn encode_path_for_snapshot(path: &Path) -> String {
    let s = path.to_string_lossy();
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '/' | '\\' | ':' => out.push_str("__"),
            other => out.push(other),
        }
    }
    if out.is_empty() {
        "_unnamed".to_owned()
    } else {
        out
    }
}

fn next_seq(root: &Path, session_id: &str) -> Result<u64, EditCheckpointError> {
    let dir = session_root(root, session_id);
    if !dir.exists() {
        return Ok(0);
    }
    let mut max_seen: Option<u64> = None;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // s-22600 Tranche G G.2: parse `{seq:06}` or `{seq:06}_{nanos:09}`;
        // legacy bare-seq directories parse as `(seq, 0)`.
        if let Some((seq, _nanos)) = parse_seq_dirname(&name_str) {
            max_seen = Some(max_seen.map_or(seq, |m| m.max(seq)));
        }
    }
    Ok(max_seen.map(|m| m + 1).unwrap_or(0))
}

/// Take a snapshot of `content` for `path` under `root` for
/// `session_id`. Returns the [`EditCheckpoint`] metadata.
///
/// The seq counter is derived from the filesystem (max + 1 over
/// existing per-session subdirs) so concurrent writers race but never
/// silently overwrite; the worst case is two snapshots claim the same
/// seq, in which case the second loses its rewind discoverability --
/// acceptable per Claude Code session-single-writer model.
pub fn snapshot_pre_edit(
    root: &Path,
    session_id: &str,
    path: &Path,
    content: &[u8],
) -> Result<EditCheckpoint, EditCheckpointError> {
    validate_session_id(session_id)?;
    let seq = next_seq(root, session_id)?;
    // s-22600 Tranche G G.2 (audit WARN-2 followup): tie-break the
    // {seq:06} dir with a subsec-nanos suffix so two concurrent writers
    // observing the same `max_seen` land in DISTINCT directories.
    // System-time subsec nanos are sufficient at OS clock resolution
    // (~100ns Windows, ~1ns Linux) for any realistic concurrent-writer
    // load on a single session. Falls back to 0 on the (impossible in
    // practice) `SystemTime::now < UNIX_EPOCH` branch.
    let nanos: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::from(d.subsec_nanos()))
        .unwrap_or(0);
    let hash = blake3::hash(content);
    let hex = hash.to_hex();
    let prefix_full = hex.as_str();
    let cut = prefix_full.len().min(BLAKE3_PREFIX_HEX_LEN);
    let prefix = prefix_full[..cut].to_owned();
    let mut dir = seq_dir(root, session_id, seq, nanos);
    dir.push(&prefix);
    fs::create_dir_all(&dir)?;
    let encoded = encode_path_for_snapshot(path);
    let snapshot_path = dir.join(&encoded);
    fs::write(&snapshot_path, content)?;
    let size_bytes = u64::try_from(content.len()).unwrap_or(u64::MAX);
    Ok(EditCheckpoint {
        session_id: session_id.to_owned(),
        seq,
        nanos,
        blake3_prefix: prefix,
        path: path.to_path_buf(),
        snapshot_path,
        size_bytes,
        timestamp_ms: current_unix_millis(),
    })
}

/// List all checkpoints for a session in ascending seq order.
pub fn list_session_checkpoints(
    root: &Path,
    session_id: &str,
) -> Result<Vec<EditCheckpoint>, EditCheckpointError> {
    validate_session_id(session_id)?;
    let dir = session_root(root, session_id);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    // s-22600 Tranche G G.2: entries now carry `(seq, nanos)` so
    // concurrent-write tie-broken directories sort stably and read back
    // with the right `EditCheckpoint.nanos` value. Legacy bare-seq
    // directories parse with `nanos = 0`.
    let mut entries: Vec<(u64, u64, PathBuf)> = Vec::new();
    for e in fs::read_dir(&dir)? {
        let e = e?;
        let name = e.file_name();
        let name_str = name.to_string_lossy();
        if let Some((seq, nanos)) = parse_seq_dirname(&name_str) {
            entries.push((seq, nanos, e.path()));
        }
    }
    entries.sort_by_key(|(s, n, _)| (*s, *n));
    let mut out: Vec<EditCheckpoint> = Vec::with_capacity(entries.len());
    for (seq, nanos, seq_path) in entries {
        for prefix_entry in fs::read_dir(&seq_path)? {
            let prefix_entry = prefix_entry?;
            let prefix_name = prefix_entry.file_name();
            let prefix = prefix_name.to_string_lossy().to_string();
            for snap_entry in fs::read_dir(prefix_entry.path())? {
                let snap_entry = snap_entry?;
                let snapshot_path = snap_entry.path();
                let encoded = snap_entry.file_name();
                let encoded_str = encoded.to_string_lossy().to_string();
                let decoded = decode_path_from_snapshot(&encoded_str);
                let size_bytes =
                    snap_entry.metadata().map(|m| m.len()).unwrap_or(0);
                out.push(EditCheckpoint {
                    session_id: session_id.to_owned(),
                    seq,
                    nanos,
                    blake3_prefix: prefix.clone(),
                    path: decoded,
                    snapshot_path,
                    size_bytes,
                    timestamp_ms: 0,
                });
            }
        }
    }
    Ok(out)
}

fn decode_path_from_snapshot(encoded: &str) -> PathBuf {
    PathBuf::from(encoded.replace("__", std::path::MAIN_SEPARATOR_STR))
}

/// Restore session state to the snapshot at `target_seq`, rolling
/// back any newer snapshots in reverse-seq order. Each restored file
/// is written from its snapshot copy. Atomic per-file (one
/// `fs::write` per checkpoint).
///
/// Roll-forward: a crash mid-rewind leaves the most-recent-restored
/// files on disk; the next invocation re-runs from the next-highest
/// seq still present (the filesystem IS the index).
pub fn rewind_to(
    root: &Path,
    session_id: &str,
    target_seq: u64,
) -> Result<RewindOutcome, EditCheckpointError> {
    validate_session_id(session_id)?;
    let all = list_session_checkpoints(root, session_id)?;
    if all.is_empty() {
        return Err(EditCheckpointError::NoSnapshots {
            session_id: session_id.to_owned(),
        });
    }
    let mut target_seen = false;
    for cp in &all {
        if cp.seq == target_seq {
            target_seen = true;
            break;
        }
    }
    if !target_seen {
        return Err(EditCheckpointError::SeqNotFound {
            session_id: session_id.to_owned(),
            seq: target_seq,
        });
    }
    let mut to_restore: Vec<EditCheckpoint> = all
        .into_iter()
        .filter(|cp| cp.seq >= target_seq)
        .collect();
    // Restore highest-seq first so mid-rewind crash leaves most-recent
    // file written.
    to_restore.sort_by(|a, b| b.seq.cmp(&a.seq));
    for cp in &to_restore {
        let content = fs::read(&cp.snapshot_path)?;
        if let Some(parent) = cp.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&cp.path, content)?;
    }
    Ok(RewindOutcome {
        session_id: session_id.to_owned(),
        target_seq,
        restored: to_restore,
    })
}

/// Evict the oldest checkpoints so at most `keep_last_n` per session
/// remain. Removes the per-seq directory tree wholesale.
pub fn evict_oldest(
    root: &Path,
    session_id: &str,
    keep_last_n: usize,
) -> Result<usize, EditCheckpointError> {
    validate_session_id(session_id)?;
    let dir = session_root(root, session_id);
    if !dir.exists() {
        return Ok(0);
    }
    // s-22600 Tranche G G.2: collect full on-disk dir names so eviction
    // removes the right `{seq:06}_{nanos:09}` directory (or legacy
    // `{seq:06}`). Sort by seq for "oldest first"; nanos is only a
    // tie-break and not a meaningful order key for eviction purposes.
    let mut entries: Vec<(u64, std::ffi::OsString)> = Vec::new();
    for e in fs::read_dir(&dir)? {
        let e = e?;
        let name = e.file_name();
        let name_str = name.to_string_lossy();
        if let Some((seq, _nanos)) = parse_seq_dirname(&name_str) {
            entries.push((seq, name));
        }
    }
    if entries.len() <= keep_last_n {
        return Ok(0);
    }
    entries.sort_by_key(|(s, _)| *s);
    let to_remove = entries.len() - keep_last_n;
    let mut removed = 0;
    for (_seq, dirname) in entries.into_iter().take(to_remove) {
        let seq_dir_path = dir.join(&dirname);
        if seq_dir_path.exists() {
            fs::remove_dir_all(&seq_dir_path)?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// Evict ALL checkpoints for a session (called at session END).
pub fn evict_session(root: &Path, session_id: &str) -> Result<(), EditCheckpointError> {
    validate_session_id(session_id)?;
    let dir = session_root(root, session_id);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

/// Build the SSB payload for a snapshot event. The rewind path emits
/// the SAME `EditCheckpointSnapshotted` kind with
/// `details_json.rewind_event = true` per GAP-3 (9-arm enum locked).
#[must_use]
pub fn build_snapshot_payload(
    cp: &EditCheckpoint,
    rewind_event: bool,
) -> ClaudeCodeArchitectureEventPayload {
    let details_json = serde_json::json!({
        "checkpoint_event": if rewind_event { "rewind_restore" } else { "snapshot" },
        "rewind_event": rewind_event,
        "seq": cp.seq,
        "blake3_prefix": cp.blake3_prefix,
        "path": cp.path.to_string_lossy(),
        "size_bytes": cp.size_bytes,
    });
    ClaudeCodeArchitectureEventPayload {
        kind: ClaudeCodeArchitectureEventKind::EditCheckpointSnapshotted,
        primitive: ClaudeCodeArchPrimitive::EditCheckpoints,
        session_id: Some(cp.session_id.clone()),
        details_json,
        ts_ms: cp.timestamp_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    // Use a process-local atomic + the test thread-id to avoid TempDir
    // dependency (CLAUDE.md no-new-deps without confirm). Each test
    // builds its own subdir under the system temp dir.
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn fresh_test_root() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut root = std::env::temp_dir();
        root.push(format!(
            "elai-e25-edit-checkpoints-test-{}-{}-{}",
            std::process::id(),
            n,
            current_unix_millis()
        ));
        fs::create_dir_all(&root).expect("create test root");
        root
    }

    #[test]
    fn validate_rejects_empty_session_id() {
        let err = validate_session_id("").unwrap_err();
        assert!(matches!(err, EditCheckpointError::InvalidSessionId { .. }));
    }

    #[test]
    fn validate_rejects_session_id_with_separators() {
        let err = validate_session_id("a/b").unwrap_err();
        assert!(matches!(err, EditCheckpointError::InvalidSessionId { .. }));
        let err = validate_session_id("a\\b").unwrap_err();
        assert!(matches!(err, EditCheckpointError::InvalidSessionId { .. }));
    }

    #[test]
    fn snapshot_creates_file_at_expected_path() {
        let root = fresh_test_root();
        let src = root.join("source.txt");
        fs::write(&src, b"hello").expect("write source");
        let cp = snapshot_pre_edit(&root, "s-test", &src, b"hello")
            .expect("snapshot");
        assert_eq!(cp.seq, 0);
        assert!(cp.snapshot_path.exists());
        let body = fs::read(&cp.snapshot_path).expect("read snapshot");
        assert_eq!(body, b"hello");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn seq_counter_is_monotonic() {
        let root = fresh_test_root();
        let path = root.join("file.txt");
        let a = snapshot_pre_edit(&root, "s-test", &path, b"v1").unwrap();
        let b = snapshot_pre_edit(&root, "s-test", &path, b"v2").unwrap();
        let c = snapshot_pre_edit(&root, "s-test", &path, b"v3").unwrap();
        assert_eq!(a.seq, 0);
        assert_eq!(b.seq, 1);
        assert_eq!(c.seq, 2);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rewind_restores_file_content() {
        let root = fresh_test_root();
        let target = root.join("subdir").join("restore.txt");
        let original = b"original";
        let _ = snapshot_pre_edit(&root, "s-test", &target, original).unwrap();
        // Simulate post-edit state by writing something different on disk.
        if let Some(p) = target.parent() {
            fs::create_dir_all(p).unwrap();
        }
        fs::write(&target, b"mutated").unwrap();
        let outcome = rewind_to(&root, "s-test", 0).unwrap();
        assert_eq!(outcome.target_seq, 0);
        assert!(!outcome.restored.is_empty());
        let restored = fs::read(&target).unwrap();
        assert_eq!(restored.as_slice(), original);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rewind_unknown_seq_returns_error_not_panic() {
        let root = fresh_test_root();
        // No snapshots at all -> NoSnapshots.
        let err = rewind_to(&root, "s-test", 0).unwrap_err();
        assert!(matches!(err, EditCheckpointError::NoSnapshots { .. }));
        // Now a snapshot exists, but a different seq is asked for.
        let path = root.join("z.txt");
        let _ = snapshot_pre_edit(&root, "s-test", &path, b"x").unwrap();
        let err = rewind_to(&root, "s-test", 99).unwrap_err();
        assert!(matches!(err, EditCheckpointError::SeqNotFound { .. }));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn build_payload_carries_rewind_event_marker() {
        let cp = EditCheckpoint {
            session_id: "s".to_owned(),
            seq: 5,
            // s-22600 Tranche G G.2: nanos tie-break field (audit
            // WARN-2 followup).
            nanos: 123_456_789,
            blake3_prefix: "abcd1234".to_owned(),
            path: PathBuf::from("foo.txt"),
            snapshot_path: PathBuf::from("/tmp/snap"),
            size_bytes: 42,
            timestamp_ms: 1_700_000_000_000,
        };
        let snap = build_snapshot_payload(&cp, false);
        assert_eq!(
            snap.kind,
            ClaudeCodeArchitectureEventKind::EditCheckpointSnapshotted
        );
        assert_eq!(
            snap.details_json
                .get("rewind_event")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        let rew = build_snapshot_payload(&cp, true);
        assert_eq!(
            rew.kind,
            ClaudeCodeArchitectureEventKind::EditCheckpointSnapshotted
        );
        assert_eq!(
            rew.details_json
                .get("rewind_event")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn evict_oldest_drops_excess_snapshots() {
        let root = fresh_test_root();
        let path = root.join("a.txt");
        for i in 0..15 {
            let body = format!("v{i}");
            snapshot_pre_edit(&root, "s-test", &path, body.as_bytes()).unwrap();
        }
        let removed = evict_oldest(&root, "s-test", 10).unwrap();
        assert_eq!(removed, 5);
        let remaining = list_session_checkpoints(&root, "s-test").unwrap();
        // After eviction at most 10 seq directories remain.
        let mut seqs: std::collections::BTreeSet<u64> =
            remaining.iter().map(|c| c.seq).collect();
        assert!(seqs.len() <= 10, "remaining seqs {seqs:?}");
        // The lowest 5 are gone (seqs 0..5).
        for evicted in 0u64..5 {
            assert!(!seqs.contains(&evicted));
        }
        seqs.clear();
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn evict_session_removes_all() {
        let root = fresh_test_root();
        let path = root.join("a.txt");
        snapshot_pre_edit(&root, "s-test", &path, b"x").unwrap();
        evict_session(&root, "s-test").unwrap();
        let remaining = list_session_checkpoints(&root, "s-test").unwrap();
        assert!(remaining.is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    // s-22600 Tranche G G.2 (audit WARN-2 followup): parse_seq_dirname
    // round-trip + legacy compat.

    #[test]
    fn parse_seq_dirname_round_trips_new_format() {
        let (seq, nanos) = parse_seq_dirname("000042_123456789").expect("new-format parse");
        assert_eq!(seq, 42);
        assert_eq!(nanos, 123_456_789);
    }

    #[test]
    fn parse_seq_dirname_accepts_legacy_bare_seq() {
        // Pre-G.2 directories had no `_{nanos}` suffix; read-compat is a
        // hard requirement so existing snapshots remain rewindable.
        let (seq, nanos) = parse_seq_dirname("000007").expect("legacy parse");
        assert_eq!(seq, 7);
        assert_eq!(nanos, 0);
    }

    #[test]
    fn parse_seq_dirname_rejects_non_numeric_prefix() {
        assert!(parse_seq_dirname("notaseq").is_none());
        assert!(parse_seq_dirname("abc_123").is_none());
    }

    #[test]
    fn snapshot_unique_under_parallel_writers() {
        // Two threads taking snapshots concurrently against the same
        // session must both succeed AND land in distinct on-disk
        // directories. Pre-G.2 they could race on max_seen and clobber
        // each other's `{seq:06}` parent (cosmetic but breaks rewind
        // discoverability). With the nanos tie-break the (seq, nanos)
        // pair is unique per writer.
        let root = std::sync::Arc::new(fresh_test_root());
        let path = std::sync::Arc::new(root.join("file.txt"));
        let mut handles = Vec::new();
        for i in 0..8 {
            let p = std::sync::Arc::clone(&path);
            let r = std::sync::Arc::clone(&root);
            handles.push(std::thread::spawn(move || {
                let body = format!("v{i}");
                snapshot_pre_edit(&r, "s-parallel", &p, body.as_bytes())
                    .expect("parallel snapshot ok")
            }));
        }
        let snaps: Vec<EditCheckpoint> =
            handles.into_iter().map(|h| h.join().expect("join")).collect();
        let pairs: std::collections::HashSet<(u64, u64)> =
            snaps.iter().map(|c| (c.seq, c.nanos)).collect();
        assert_eq!(
            pairs.len(),
            snaps.len(),
            "duplicate (seq, nanos) pair across parallel writers: {snaps:?}"
        );
        let on_disk: std::collections::HashSet<_> =
            snaps.iter().map(|c| c.snapshot_path.clone()).collect();
        assert_eq!(
            on_disk.len(),
            snaps.len(),
            "duplicate on-disk snapshot_path across parallel writers"
        );
        let _ = fs::remove_dir_all(&*root);
    }
}
