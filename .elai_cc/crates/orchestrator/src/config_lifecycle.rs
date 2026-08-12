//! P-19 Slice 1: production-reachable `ConfigLifecycle` guard for
//! `/runtime set` and `/runtime composer2 enable|disable` config writes.
//!
//! The guard sits in front of the raw atomic-write helper
//! (`crate::util::atomic_io::atomic_write_sync`) and enforces a
//! shrink-floor: a proposed config that is strictly below half the
//! existing config's byte length is rejected, the rejected bytes are
//! preserved as a `.rejected.<timestamp>` sibling, and the active file
//! is left untouched. Accepted writes persist the pre-write bytes as a
//! `.lkg` (last-known-good) sibling before the atomic swap.
//!
//! Missing and zero-length originals are allowed (no shrink floor
//! applies). The guard uses `Result` propagation throughout and adds no
//! dependencies.

use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Outcome of a [`ConfigLifecycle::guard_write`] call.
#[derive(Debug)]
pub enum ConfigWriteOutcome {
    /// The proposed bytes were accepted and written; the prior content
    /// (if any) was preserved as a `.lkg` sibling.
    Accepted,
    /// The proposed bytes were rejected because they were strictly below
    /// half the existing byte length. The rejected bytes were preserved as
    /// a `.rejected.<timestamp>` sibling and the active file is unchanged.
    Rejected {
        existing_len: usize,
        proposed_len: usize,
    },
}

/// Production-reachable guard for config-file writes. Wraps the raw
/// atomic-write helper with a shrink-floor and last-known-good
/// preservation.
pub struct ConfigLifecycle;

impl ConfigLifecycle {
    /// Guard a proposed config write to `path`.
    ///
    /// - Reads the existing bytes at `path` (missing / zero-length are
    ///   allowed; no shrink floor applies).
    /// - If `proposed.len()` is strictly below half of the existing length,
    ///   the write is rejected: the proposed bytes are preserved as
    ///   `<path>.rejected.<timestamp>` and the active file is left
    ///   untouched.
    /// - On acceptance, the pre-write bytes (if any) are preserved as
    ///   `<path>.lkg` before the atomic write swaps in the new content.
    pub fn guard_write(path: &Path, proposed: &[u8]) -> io::Result<ConfigWriteOutcome> {
        let existing = std::fs::read(path).unwrap_or_default();
        let existing_len = existing.len();

        if existing_len > 0 && proposed.len() * 2 < existing_len {
            // Strictly below half — reject and preserve the rejected bytes.
            let rejected_path = rejected_sibling(path);
            std::fs::write(&rejected_path, proposed)?;
            return Ok(ConfigWriteOutcome::Rejected {
                existing_len,
                proposed_len: proposed.len(),
            });
        }

        // Accepted: preserve pre-write bytes as LKG if there were any.
        if !existing.is_empty() {
            let lkg_path = lkg_sibling(path);
            std::fs::write(&lkg_path, &existing)?;
        }

        crate::util::atomic_io::atomic_write_sync(path, proposed)?;
        Ok(ConfigWriteOutcome::Accepted)
    }
}

/// Build the `.lkg` sibling path for `dest`.
fn lkg_sibling(dest: &Path) -> PathBuf {
    let mut s = dest.as_os_str().to_owned();
    s.push(".lkg");
    PathBuf::from(s)
}

/// Build the `.rejected.<timestamp>` sibling path for `dest`. The
/// timestamp is Unix epoch nanos to guarantee uniqueness across rapid
/// successive rejections.
fn rejected_sibling(dest: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut s = dest.as_os_str().to_owned();
    s.push(format!(".rejected.{nanos}"));
    PathBuf::from(s)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir() -> tempfile::TempDir {
        tempfile::TempDir::new().expect("tempdir")
    }

    #[test]
    fn accepts_when_no_existing_file() {
        let dir = tmp_dir();
        let path = dir.path().join("elai_cc.toml");
        let proposed = b"[test]\nkey = \"value\"\n";
        let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("write");
        assert!(matches!(outcome, ConfigWriteOutcome::Accepted));
        let got = std::fs::read(&path).expect("read back");
        assert_eq!(got, proposed);
        // No LKG because there was no prior content.
        assert!(!dir.path().join("elai_cc.toml.lkg").exists());
    }

    #[test]
    fn accepts_when_existing_is_zero_length() {
        let dir = tmp_dir();
        let path = dir.path().join("elai_cc.toml");
        std::fs::write(&path, b"").expect("write empty");
        let proposed = b"[test]\nkey = \"value\"\n";
        let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("write");
        assert!(matches!(outcome, ConfigWriteOutcome::Accepted));
        let got = std::fs::read(&path).expect("read back");
        assert_eq!(got, proposed);
    }

    #[test]
    fn accepts_normal_write_and_preserves_lkg() {
        let dir = tmp_dir();
        let path = dir.path().join("elai_cc.toml");
        let original = b"[agents.executor.runtime]\nbackend = \"sidecar\"\n";
        std::fs::write(&path, original).expect("write original");
        let proposed = b"[agents.executor.runtime]\nbackend = \"anthropic_api\"\n";
        let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("write");
        assert!(matches!(outcome, ConfigWriteOutcome::Accepted));
        let got = std::fs::read(&path).expect("read back");
        assert_eq!(got, proposed);
        let lkg = std::fs::read(lkg_sibling(&path)).expect("read lkg");
        assert_eq!(lkg, original);
    }

    #[test]
    fn rejects_shrink_below_half_and_preserves_rejected_and_original() {
        let dir = tmp_dir();
        let path = dir.path().join("elai_cc.toml");
        let original = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        std::fs::write(&path, original).expect("write original");
        let proposed = b"X"; // 1 byte, existing 30 bytes -> 1*2=2 < 30
        let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("guard");
        match outcome {
            ConfigWriteOutcome::Rejected {
                existing_len,
                proposed_len,
            } => {
                assert_eq!(existing_len, 30);
                assert_eq!(proposed_len, 1);
            }
            ConfigWriteOutcome::Accepted => panic!("should have rejected"),
        }
        // Active file unchanged.
        let active = std::fs::read(&path).expect("read active");
        assert_eq!(active, original);
        // Rejected bytes preserved.
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.contains(".rejected."))
            .collect();
        assert_eq!(entries.len(), 1, "exactly one rejected sibling: {entries:?}");
        let rejected_content =
            std::fs::read(dir.path().join(&entries[0])).expect("read rejected");
        assert_eq!(rejected_content, proposed);
    }

    #[test]
    fn exact_half_boundary_is_accepted() {
        let dir = tmp_dir();
        let path = dir.path().join("elai_cc.toml");
        let original = b"ABCD"; // 4 bytes
        std::fs::write(&path, original).expect("write original");
        let proposed = b"XY"; // 2 bytes, 2*2=4 == 4, NOT strictly below
        let outcome = ConfigLifecycle::guard_write(&path, proposed).expect("write");
        assert!(matches!(outcome, ConfigWriteOutcome::Accepted));
        let got = std::fs::read(&path).expect("read back");
        assert_eq!(got, proposed);
    }
}
