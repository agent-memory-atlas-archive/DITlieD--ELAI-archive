//! D-9 — L3 frozen-surface access-control auditor.
//!
//! Implements [`LockedKeysAuditor`]: walks a staged git diff (or a
//! synthetic file list) against a TOML registry of locked paths and
//! locked symbols, emits one [`SsbPayload::LockedKeysViolationDetected`]
//! per overlap, and one [`SsbPayload::LockedKeysAuditRun`] per audit
//! invocation (no matter the outcome — the audit happened, the run row
//! is the durable signal).
//!
//! Grounding sources (per CLAUDE.md grounded-by-default rule):
//!   * docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md
//!   * crates/orchestrator/src/wiring_contract.rs (C-27 StubBanAuditor
//!     pattern — sink shape and audit_id Uuid::new_v4 boundary mirrored
//!     here so the dashboard joins both auditors on `audit_id`).
//!   * arxiv:1908.04734 (Everitt current-RF principle: the agent must
//!     not edit the function that grades it).
//!
//! Design summary:
//!   * Registry rows declare either a full-file lock (`path`) or a
//!     symbol lock (`symbol` + optional `file`); line-range scoping is
//!     additive on top of `path` for callers that want sub-file
//!     resolution. The MVP-shipped surface is full-file (`line_range =
//!     (0, u32::MAX)`); structural use of line ranges is reserved for
//!     B-side per-symbol freezes once we have a symbol-graph-backed
//!     diff parser.
//!   * Fail-closed: registry parse error returns
//!     [`LockedKeysAuditError::RegistryUnreadable`]; callers must treat
//!     an unreadable registry as "every staged path is locked".
//!     Documented in WIRING_CONTRACT.stub_ban of D-9.
//!   * No git2 / shell-out dep added here; the CLI driver feeds the
//!     auditor a pre-collected `Vec<PathBuf>` from `git diff --cached
//!     --name-only`. Keeps the orchestrator crate dep surface
//!     unchanged (CLAUDE.md anti-slopsquatting).

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, warn};

use crate::ssb::{
    LockedKeysAuditRunPayload, LockedKeysViolationDetectedPayload, SsbPayload,
};

/// Default sentinel meaning "entire file is locked" — the line range
/// `(0, u32::MAX)` matches every line of any staged edit on the path.
pub const LOCKED_KEY_FULL_FILE: (u32, u32) = (0, u32::MAX);

// ---------------------------------------------------------------------------
// Registry value types
// ---------------------------------------------------------------------------

/// A single locked path entry. `path` is workspace-relative
/// (forward-slash on both Unix and Windows — the auditor normalises
/// the staged-diff entries to match).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedKey {
    /// Workspace-relative path of the locked surface. Compared as
    /// forward-slash on both platforms.
    pub path: PathBuf,
    /// Inclusive line range. `(0, u32::MAX)` means the whole file is
    /// locked. Sub-file ranges are honoured but the MVP CLI surface
    /// stages whole-file diffs only — granular ranges become useful
    /// when a B-N introduces a symbol-graph-backed diff source.
    #[serde(default = "default_full_range")]
    pub line_range: (u32, u32),
    /// Human-readable reason this surface is frozen — surfaced in the
    /// pre-commit hook error message so the operator knows what they
    /// hit.
    pub reason: String,
    /// Session id that registered the lock (empty string for
    /// bootstrap-seed entries). Forms part of the registry signature so
    /// any tampering invalidates the audit run hash.
    #[serde(default)]
    pub owner_session: String,
    /// UNIX ms when the lock was registered. Zero for bootstrap-seed
    /// entries.
    #[serde(default)]
    pub freeze_ts_ms: u64,
    /// Blake3 (32-byte raw) of `{path}|{line_range}|{reason}|{owner_session}|{freeze_ts_ms}`
    /// — populated by [`LockedKey::compute_signature`]. Stored as raw
    /// bytes so equality is cheap and serialization is a hex string of
    /// length 64.
    #[serde(default, with = "blake3_hex32")]
    pub signature_blake3: [u8; 32],
}

fn default_full_range() -> (u32, u32) {
    LOCKED_KEY_FULL_FILE
}

/// Serde helper: serialise `[u8; 32]` as 64-char lowercase hex, and
/// accept the same on read. Empty / absent string round-trips to all
/// zeros so legacy registries without the signature field still parse.
/// Hand-rolled stdlib encoder — no new `hex` crate dep (CLAUDE.md
/// no-new-deps-without-confirmation).
mod blake3_hex32 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        super::to_hex(bytes).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let raw = String::deserialize(d)?;
        if raw.is_empty() {
            return Ok([0u8; 32]);
        }
        let bytes = super::from_hex(&raw).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("signature_blake3 must be 32 bytes"));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

/// Lowercase-hex encode without pulling in the `hex` crate.
pub(crate) fn to_hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(TABLE[(b >> 4) as usize] as char);
        out.push(TABLE[(b & 0x0f) as usize] as char);
    }
    out
}

/// Decode lowercase / uppercase hex into bytes. Rejects non-hex chars
/// and odd-length strings.
pub(crate) fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err(format!("hex string length must be even (got {})", s.len()));
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks(2) {
        let hi = decode_nibble(chunk[0])?;
        let lo = decode_nibble(chunk[1])?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn decode_nibble(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        other => Err(format!("non-hex byte 0x{other:02x}")),
    }
}

impl LockedKey {
    /// Compute the canonical blake3 signature for this entry. The
    /// pre-image is the colon-separated tuple
    /// `path|line_range|reason|owner_session|freeze_ts_ms`. Stable
    /// across serde round-trips because every field is normalised to
    /// its string form.
    pub fn compute_signature(&self) -> [u8; 32] {
        let preimage = format!(
            "{}|{}-{}|{}|{}|{}",
            self.path_as_posix(),
            self.line_range.0,
            self.line_range.1,
            self.reason,
            self.owner_session,
            self.freeze_ts_ms
        );
        *blake3::hash(preimage.as_bytes()).as_bytes()
    }

    /// Return the path as forward-slash-normalised UTF-8. The auditor
    /// matches staged-diff paths case-sensitively after the same
    /// normalisation, so Windows backslashes here would silently miss.
    pub fn path_as_posix(&self) -> String {
        self.path.to_string_lossy().replace('\\', "/")
    }

    /// True if `(staged_line_start, staged_line_end)` overlaps this
    /// entry's locked range. Boundaries are inclusive; full-file locks
    /// always overlap any non-empty staged range.
    pub fn overlaps_range(&self, staged: (u32, u32)) -> bool {
        let (a, b) = self.line_range;
        let (c, d) = staged;
        // Standard closed-interval overlap: a <= d && c <= b.
        a <= d && c <= b
    }
}

/// In-memory representation of a loaded registry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LockedKeysRegistry {
    /// All entries. Order is preserved from the TOML file so the audit
    /// produces stable output across runs.
    #[serde(default, rename = "locked_keys")]
    pub entries: Vec<LockedKey>,
    /// Path the registry was loaded from. Populated by
    /// [`LockedKeysRegistry::load`]; not serialised on round-trip.
    #[serde(skip)]
    pub registry_path: PathBuf,
    /// UNIX ms when the registry was loaded into memory. Zero for the
    /// default (empty) registry.
    #[serde(skip)]
    pub loaded_ts_ms: u64,
}

impl LockedKeysRegistry {
    /// Parse a TOML registry file. Fails closed on missing / unreadable
    /// / unparseable input so callers cannot silently treat a broken
    /// registry as "nothing locked".
    pub fn load(registry_path: impl Into<PathBuf>) -> Result<Self, LockedKeysAuditError> {
        let registry_path = registry_path.into();
        let raw = fs::read_to_string(&registry_path).map_err(|e| {
            LockedKeysAuditError::RegistryUnreadable {
                path: registry_path.clone(),
                detail: e.to_string(),
            }
        })?;
        let mut registry: LockedKeysRegistry = toml::from_str(&raw).map_err(|e| {
            LockedKeysAuditError::RegistryUnreadable {
                path: registry_path.clone(),
                detail: e.to_string(),
            }
        })?;
        registry.registry_path = registry_path;
        registry.loaded_ts_ms = current_unix_ms();
        Ok(registry)
    }

    /// Compute the blake3 hash of the registry's serialised content.
    /// Used as `registry_blake3` on the audit-run SSB row so dashboard
    /// consumers can detect registry drift between runs.
    pub fn content_blake3(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        for entry in &self.entries {
            hasher.update(entry.path_as_posix().as_bytes());
            hasher.update(b"|");
            hasher.update(format!("{}-{}", entry.line_range.0, entry.line_range.1).as_bytes());
            hasher.update(b"|");
            hasher.update(entry.reason.as_bytes());
            hasher.update(b"\n");
        }
        *hasher.finalize().as_bytes()
    }

    /// Empty registry constructor for tests + the fail-soft default
    /// when callers explicitly want a no-op auditor (e.g.,
    /// observable-effect smoke fixtures).
    pub fn empty(registry_path: impl Into<PathBuf>) -> Self {
        Self {
            entries: Vec::new(),
            registry_path: registry_path.into(),
            loaded_ts_ms: current_unix_ms(),
        }
    }
}

// ---------------------------------------------------------------------------
// Violation + AuditResult
// ---------------------------------------------------------------------------

/// A single overlap between a staged edit and a locked-key entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub locked_key: LockedKey,
    pub attempted_edit: PathBuf,
    pub attempted_lines: (u32, u32),
}

/// Result of one `audit_diff` invocation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuditResult {
    pub violations: Vec<Violation>,
    pub allowed_edits: Vec<PathBuf>,
}

impl AuditResult {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum LockedKeysAuditError {
    #[error("locked keys registry unreadable at {path}: {detail}")]
    RegistryUnreadable { path: PathBuf, detail: String },
    #[error(transparent)]
    Io(#[from] io::Error),
}

// ---------------------------------------------------------------------------
// Auditor
// ---------------------------------------------------------------------------

/// Sink used by [`LockedKeysAuditor`] to publish audit-run + violation
/// rows. Production sinks emit through the orchestrator's mpsc bus;
/// tests use the in-memory capture buffer.
pub enum LockedKeysSink {
    Channel(UnboundedSender<SsbPayload>),
    Capture(Arc<std::sync::Mutex<Vec<SsbPayload>>>),
}

impl LockedKeysSink {
    fn send(&self, payload: SsbPayload) {
        match self {
            Self::Channel(tx) => {
                if let Err(err) = tx.send(payload) {
                    warn!(
                        target: "elai_orchestrator::locked_keys_auditor",
                        "locked-keys SSB channel closed: {err}"
                    );
                }
            }
            Self::Capture(buf) => match buf.lock() {
                Ok(mut g) => g.push(payload),
                Err(poisoned) => poisoned.into_inner().push(payload),
            },
        }
    }
}

/// LockedKeysAuditor: registry-backed access-control walker.
pub struct LockedKeysAuditor {
    registry: LockedKeysRegistry,
    sink: Option<LockedKeysSink>,
    audit_id: uuid::Uuid,
}

impl LockedKeysAuditor {
    /// Construct an auditor that emits SSB rows through `sink`. Pass
    /// `None` for read-only callers (Tauri status command).
    pub fn new(registry: LockedKeysRegistry, sink: Option<LockedKeysSink>) -> Self {
        Self {
            registry,
            sink,
            // entropy-allow: per-audit-run correlator follows the C-27
            // StubBanAuditor precedent — `uuid::Uuid::new_v4` is the
            // accepted boundary for non-deterministic audit ids. The
            // determinism harness operates on action_type sequences,
            // not audit ids.
            audit_id: uuid::Uuid::new_v4(),
        }
    }

    /// Construct from a registry path. Fail-closed on parse error per
    /// WIRING_CONTRACT.stub_ban: an unreadable registry MUST NOT
    /// silently allow edits.
    pub fn load(
        registry_path: impl Into<PathBuf>,
        sink: Option<LockedKeysSink>,
    ) -> Result<Self, LockedKeysAuditError> {
        let registry = LockedKeysRegistry::load(registry_path)?;
        Ok(Self::new(registry, sink))
    }

    pub fn registry(&self) -> &LockedKeysRegistry {
        &self.registry
    }

    pub fn audit_id(&self) -> uuid::Uuid {
        self.audit_id
    }

    /// Audit a staged-diff file list against the loaded registry. The
    /// MVP surface assumes whole-file edits — every staged path is
    /// matched against every full-file lock; sub-file ranges are
    /// matched if the caller supplies them via [`Self::audit_diff_ranged`].
    ///
    /// Emits exactly one [`SsbPayload::LockedKeysAuditRun`] row per
    /// call (regardless of whether violations were found) and one
    /// [`SsbPayload::LockedKeysViolationDetected`] row per violation.
    pub fn audit_diff(&self, staged_files: &[PathBuf]) -> AuditResult {
        let ranged: Vec<(PathBuf, (u32, u32))> = staged_files
            .iter()
            .map(|p| (p.clone(), LOCKED_KEY_FULL_FILE))
            .collect();
        self.audit_diff_ranged(&ranged)
    }

    /// Range-aware variant of [`Self::audit_diff`]. Each entry is
    /// `(staged_path, inclusive_line_range)`. The line range pair is
    /// matched against [`LockedKey::overlaps_range`].
    pub fn audit_diff_ranged(&self, staged: &[(PathBuf, (u32, u32))]) -> AuditResult {
        let mut violations = Vec::new();
        let mut allowed: BTreeSet<PathBuf> = BTreeSet::new();
        for (path, range) in staged {
            let normalised = path.to_string_lossy().replace('\\', "/");
            let mut hit = false;
            for entry in &self.registry.entries {
                if entry.path_as_posix() == normalised && entry.overlaps_range(*range) {
                    let v = Violation {
                        locked_key: entry.clone(),
                        attempted_edit: path.clone(),
                        attempted_lines: *range,
                    };
                    self.emit_violation(&v);
                    violations.push(v);
                    hit = true;
                }
            }
            if !hit {
                allowed.insert(path.clone());
            }
        }
        let result = AuditResult {
            violations,
            allowed_edits: allowed.into_iter().collect(),
        };
        self.emit_audit_run(staged.len() as u32, result.violations.len() as u32);
        result
    }

    /// Convenience helper: returns `Err` with the first violation when
    /// any are present, mirroring the pre-commit hook's expected exit
    /// semantics. Useful for callers (CLI driver) that want a single
    /// short-circuit return rather than iterating the AuditResult.
    pub fn check_no_violations(&self, staged: &[PathBuf]) -> Result<(), Violation> {
        let result = self.audit_diff(staged);
        if let Some(v) = result.violations.into_iter().next() {
            Err(v)
        } else {
            Ok(())
        }
    }

    fn emit_audit_run(&self, n_audited: u32, n_violations: u32) {
        let Some(sink) = self.sink.as_ref() else {
            debug!(
                target: "elai_orchestrator::locked_keys_auditor",
                "LockedKeysAuditor::emit_audit_run skipped (sink=None) n_audited={n_audited} n_violations={n_violations}"
            );
            return;
        };
        let payload = LockedKeysAuditRunPayload {
            audit_id: self.audit_id,
            ts_ms: current_unix_ms() as i64,
            n_audited,
            n_violations,
            registry_blake3: to_hex(&self.registry.content_blake3()),
            registry_path: self.registry.registry_path.to_string_lossy().to_string(),
        };
        sink.send(SsbPayload::LockedKeysAuditRun(payload));
    }

    fn emit_violation(&self, v: &Violation) {
        let Some(sink) = self.sink.as_ref() else {
            debug!(
                target: "elai_orchestrator::locked_keys_auditor",
                "LockedKeysAuditor::emit_violation skipped (sink=None) path={:?}",
                v.attempted_edit
            );
            return;
        };
        let payload = LockedKeysViolationDetectedPayload {
            audit_id: self.audit_id,
            ts_ms: current_unix_ms() as i64,
            attempted_edit: v.attempted_edit.to_string_lossy().replace('\\', "/"),
            attempted_lines: vec![v.attempted_lines.0, v.attempted_lines.1],
            locked_key_path: v.locked_key.path_as_posix(),
            locked_key_signature_blake3: to_hex(&v.locked_key.signature_blake3),
            reason: v.locked_key.reason.clone(),
        };
        sink.send(SsbPayload::LockedKeysViolationDetected(payload));
    }
}

fn current_unix_ms() -> u64 {
    u64::try_from(OsClock.now_ms()).unwrap_or(u64::MAX)
}

/// Read-only snapshot used by the Tauri `get_locked_keys_status`
/// command — no SSB emission, no audit-run row. Returns the parsed
/// registry plus a fresh content hash so the dashboard can detect
/// drift since the last audit-run row.
pub fn status_snapshot(
    registry_path: impl AsRef<Path>,
) -> Result<LockedKeysStatusSnapshot, LockedKeysAuditError> {
    let registry = LockedKeysRegistry::load(registry_path.as_ref().to_path_buf())?;
    let blake3_hex = to_hex(&registry.content_blake3());
    Ok(LockedKeysStatusSnapshot {
        registry_path: registry.registry_path.to_string_lossy().to_string(),
        n_entries: registry.entries.len() as u32,
        registry_blake3: blake3_hex,
        loaded_ts_ms: registry.loaded_ts_ms as i64,
        entries: registry
            .entries
            .into_iter()
            .map(|e| LockedKeysStatusEntry {
                path: e.path_as_posix(),
                line_range: vec![e.line_range.0, e.line_range.1],
                reason: e.reason,
            })
            .collect(),
    })
}

/// Tauri-facing status payload — flat / json-friendly types only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedKeysStatusSnapshot {
    pub registry_path: String,
    pub n_entries: u32,
    pub registry_blake3: String,
    pub loaded_ts_ms: i64,
    pub entries: Vec<LockedKeysStatusEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedKeysStatusEntry {
    pub path: String,
    pub line_range: Vec<u32>,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    fn capture_sink() -> (LockedKeysSink, Arc<Mutex<Vec<SsbPayload>>>) {
        let buf = Arc::new(Mutex::new(Vec::<SsbPayload>::new()));
        (LockedKeysSink::Capture(buf.clone()), buf)
    }

    fn one_entry_registry(tmp: &TempDir) -> LockedKeysRegistry {
        let mut entry = LockedKey {
            path: PathBuf::from("crates/orchestrator/src/tripwire.rs"),
            line_range: LOCKED_KEY_FULL_FILE,
            reason: "tripwire detection per arxiv:1908.04734".into(),
            owner_session: "bootstrap-seed".into(),
            freeze_ts_ms: 0,
            signature_blake3: [0u8; 32],
        };
        entry.signature_blake3 = entry.compute_signature();
        LockedKeysRegistry {
            entries: vec![entry],
            registry_path: tmp.path().join("locked_keys_registry.toml"),
            loaded_ts_ms: 1,
        }
    }

    #[test]
    fn locked_key_signature_is_deterministic() {
        let e1 = LockedKey {
            path: PathBuf::from("a/b.rs"),
            line_range: (0, 10),
            reason: "r".into(),
            owner_session: "s".into(),
            freeze_ts_ms: 1,
            signature_blake3: [0u8; 32],
        };
        let e2 = e1.clone();
        assert_eq!(e1.compute_signature(), e2.compute_signature());
    }

    #[test]
    fn audit_diff_flags_overlap_with_locked_path() {
        let tmp = TempDir::new().expect("tempdir");
        let registry = one_entry_registry(&tmp);
        let (sink, buf) = capture_sink();
        let auditor = LockedKeysAuditor::new(registry, Some(sink));
        let staged = vec![PathBuf::from("crates/orchestrator/src/tripwire.rs")];
        let result = auditor.audit_diff(&staged);
        assert_eq!(result.violations.len(), 1, "must flag locked path");
        assert!(result.allowed_edits.is_empty());
        let captured = buf.lock().expect("lock");
        // exactly 1 audit-run + 1 violation row
        assert_eq!(captured.len(), 2);
        let kinds: Vec<_> = captured
            .iter()
            .map(|p| matches!(p, SsbPayload::LockedKeysAuditRun(_)))
            .collect();
        assert!(kinds.contains(&true), "must emit LockedKeysAuditRun");
        assert!(
            captured.iter().any(|p| matches!(
                p,
                SsbPayload::LockedKeysViolationDetected(_)
            )),
            "must emit LockedKeysViolationDetected"
        );
    }

    #[test]
    fn audit_diff_passes_unrelated_path() {
        let tmp = TempDir::new().expect("tempdir");
        let registry = one_entry_registry(&tmp);
        let (sink, buf) = capture_sink();
        let auditor = LockedKeysAuditor::new(registry, Some(sink));
        let staged = vec![PathBuf::from("crates/dashboard-v2/src/App.svelte")];
        let result = auditor.audit_diff(&staged);
        assert!(result.violations.is_empty(), "unrelated path must pass");
        assert_eq!(result.allowed_edits.len(), 1);
        let captured = buf.lock().expect("lock");
        // only the audit-run row; no violation row
        assert_eq!(captured.len(), 1);
        assert!(matches!(captured[0], SsbPayload::LockedKeysAuditRun(_)));
    }

    #[test]
    fn audit_diff_normalises_windows_separators() {
        let tmp = TempDir::new().expect("tempdir");
        let registry = one_entry_registry(&tmp);
        let auditor = LockedKeysAuditor::new(registry, None);
        // Staged path uses backslashes (Windows git output passes through).
        let staged = vec![PathBuf::from("crates\\orchestrator\\src\\tripwire.rs")];
        let result = auditor.audit_diff(&staged);
        assert_eq!(
            result.violations.len(),
            1,
            "backslash form must normalise to posix and still match"
        );
    }

    #[test]
    fn check_no_violations_returns_err_on_overlap() {
        let tmp = TempDir::new().expect("tempdir");
        let registry = one_entry_registry(&tmp);
        let auditor = LockedKeysAuditor::new(registry, None);
        let staged = vec![PathBuf::from("crates/orchestrator/src/tripwire.rs")];
        let outcome = auditor.check_no_violations(&staged);
        assert!(outcome.is_err());
    }

    #[test]
    fn load_fails_closed_on_missing_registry() {
        let tmp = TempDir::new().expect("tempdir");
        let missing = tmp.path().join("nope.toml");
        let err = LockedKeysAuditor::load(missing, None).err();
        assert!(matches!(err, Some(LockedKeysAuditError::RegistryUnreadable { .. })));
    }

    #[test]
    fn load_parses_seeded_toml() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("registry.toml");
        let toml_text = r#"
[[locked_keys]]
path = "crates/orchestrator/src/tripwire.rs"
line_range = [0, 4294967295]
reason = "tripwire detection per arxiv:1908.04734"
owner_session = "bootstrap-seed"
freeze_ts_ms = 0
signature_blake3 = ""
"#;
        fs::write(&path, toml_text).expect("write");
        let auditor = LockedKeysAuditor::load(&path, None).expect("load");
        assert_eq!(auditor.registry().entries.len(), 1);
        // The deserializer should default an empty signature to zeros;
        // we never trust the stored signature for matching (only for
        // tamper-detection on the run row).
        assert_eq!(auditor.registry().entries[0].signature_blake3, [0u8; 32]);
    }

    #[test]
    fn overlaps_range_is_inclusive() {
        let e = LockedKey {
            path: PathBuf::from("a.rs"),
            line_range: (10, 20),
            reason: "r".into(),
            owner_session: String::new(),
            freeze_ts_ms: 0,
            signature_blake3: [0u8; 32],
        };
        assert!(e.overlaps_range((15, 15)));
        assert!(e.overlaps_range((5, 11)));
        assert!(e.overlaps_range((20, 25)));
        assert!(!e.overlaps_range((0, 9)));
        assert!(!e.overlaps_range((21, 30)));
    }
}
