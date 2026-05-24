//! E-18 — Spill subsystem.
//!
//! Writes oversize tool-call bodies to disk under
//! `.elai_cc/.elai_cc/spill/<ts>-<hash>.json` (matching the
//! `.elai_cc/.elai_cc/grammar_cache/` + `.elai_cc/.elai_cc/ssb.db`
//! convention shipped by E-35 + B-7). The plan's original
//! `.elai_cc/sessions/{id}/spill/` path was rejected during the
//! context-compiler pass — `.elai_cc/.elai_cc/` is the canonical
//! per-project store sibling to `ssb.db`.
//!
//! Anti-stale grounding:
//!   * `blake3` for the content digest (NEVER blake2 / sha2).
//!   * `tokio::fs` would be wrong here — the spill happens on a
//!     dispatcher tick already inside an async runtime, but the I/O
//!     itself is fire-and-forget and the body is at most a few hundred
//!     KiB. Using sync `std::fs::write` keeps the function signature
//!     callable from non-async test code and avoids a fake-async
//!     wrapper. The dispatcher offloads via `spawn_blocking` when it
//!     decides the size warrants it.
//!   * No `.unwrap()` / `.expect()` outside `#[cfg(test)]`.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::types::{SpillManifest, ToolCallSnapshot};
use super::ToolReliabilityError;

/// Sibling-convention directory name. Per E-35 grammar_cache + ssb.db
/// the spill root lives at `<project_root>/.elai_cc/spill/`. The
/// dispatcher resolves the project root and passes it in so tests can
/// redirect to a tempdir.
pub const SPILL_DIR_NAME: &str = "spill";

/// Manifest record persisted alongside the spill body. Holds the
/// dispatcher-side blake3 of the body so a later read-back can
/// detect corruption + match by hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpillRecord {
    pub tool_name: String,
    pub manifest: SpillManifest,
    pub created_ts_ms: u64,
}

impl SpillRecord {
    pub fn path(&self) -> &Path {
        &self.manifest.path
    }

    pub fn payload_bytes(&self) -> usize {
        self.manifest.payload_bytes
    }
}

/// Default project-root spill dir resolver. Returns
/// `<project_root>/.elai_cc/spill`. Callers that want a different
/// location pass an explicit path to [`spill_call`].
///
/// The `project_root` argument is typically `.elai_cc/.elai_cc` (the
/// inner `.elai_cc` is the per-project state store sibling to
/// `ssb.db` — see E-35 grammar_cache convention).
pub fn spill_root_default(project_state_root: &Path) -> PathBuf {
    project_state_root.join(SPILL_DIR_NAME)
}

/// Spill `call.args` to disk under `spill_dir`. Creates the directory
/// (and any parents) if missing. Returns a [`SpillRecord`] the caller
/// embeds in the IPC ToolResult (or the SSB `ToolCallSpilled` row).
///
/// Filename: `<ts_ms>-<blake3_first8_hex>.json`. The blake3 prefix
/// disambiguates simultaneous spills at the same millisecond; the full
/// 32-byte hex digest is stored in the manifest field for integrity
/// checking.
pub fn spill_call(call: &ToolCallSnapshot, spill_dir: &Path) -> Result<SpillRecord, ToolReliabilityError> {
    std::fs::create_dir_all(spill_dir)?;

    // Serialize args (deterministic for tests).
    let body_bytes = serde_json::to_vec(&call.args).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("serialize args: {e}"))
    })?;

    // blake3 digest of the body — full hex stored in manifest; first 8
    // bytes form the filename suffix.
    let digest = blake3::hash(&body_bytes);
    let hex_full = digest.to_hex().to_string();
    let prefix_hex: String = hex_full.chars().take(16).collect(); // 8 bytes = 16 hex chars

    // Timestamp suffix for monotonic ordering. SystemTime can fail in
    // theory; in practice we are post-1970, but we guard against the
    // Err arm rather than `.unwrap()`ing per CLAUDE.md.
    let ts_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 0,
    };

    let fname = format!("{ts_ms}-{prefix_hex}.json");
    let path = spill_dir.join(fname);

    std::fs::write(&path, &body_bytes)?;

    let manifest = SpillManifest {
        path: path.clone(),
        payload_bytes: body_bytes.len(),
        blake3_hex: hex_full,
    };

    Ok(SpillRecord {
        tool_name: call.tool_name.clone(),
        manifest,
        created_ts_ms: ts_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn spill_writes_file_and_returns_manifest() {
        let dir = TempDir::new().expect("tempdir");
        let call = ToolCallSnapshot::new("edit", json!({"path": "src/lib.rs", "body": "hello"}));
        let rec = spill_call(&call, dir.path()).expect("spill ok");
        assert_eq!(rec.tool_name, "edit");
        assert!(rec.path().exists(), "spill file must exist on disk");
        assert!(rec.payload_bytes() > 0);
        // Filename: <ts>-<16-hex>.json
        let name = rec.path().file_name().expect("file name").to_string_lossy().to_string();
        assert!(name.ends_with(".json"), "expected .json suffix, got {name}");
        assert!(name.contains('-'), "expected ts-hash separator in {name}");
    }

    #[test]
    fn spill_creates_parent_dirs() {
        let dir = TempDir::new().expect("tempdir");
        let nested = dir.path().join("nested").join("spill");
        let call = ToolCallSnapshot::new("edit", json!({"x": 1}));
        let rec = spill_call(&call, &nested).expect("spill ok");
        assert!(rec.path().starts_with(&nested));
    }

    #[test]
    fn spill_root_default_uses_sibling_convention() {
        let project_state = PathBuf::from(".elai_cc").join(".elai_cc");
        let p = spill_root_default(&project_state);
        // Path is .elai_cc/.elai_cc/spill — matches E-35 grammar_cache + ssb.db.
        let s = p.to_string_lossy().to_string();
        assert!(s.contains(".elai_cc"), "expected .elai_cc in {s}");
        assert!(s.ends_with("spill"), "expected /spill suffix in {s}");
    }

    #[test]
    fn spill_two_calls_yield_distinct_files() {
        let dir = TempDir::new().expect("tempdir");
        let a = ToolCallSnapshot::new("edit", json!({"path": "a"}));
        let b = ToolCallSnapshot::new("edit", json!({"path": "b"}));
        let r1 = spill_call(&a, dir.path()).expect("spill a");
        let r2 = spill_call(&b, dir.path()).expect("spill b");
        // Different content -> different blake3 -> different filename even
        // if the ts collides.
        assert_ne!(r1.manifest.blake3_hex, r2.manifest.blake3_hex);
    }
}
