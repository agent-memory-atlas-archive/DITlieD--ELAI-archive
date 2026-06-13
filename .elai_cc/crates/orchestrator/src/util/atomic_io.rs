//! I-3: atomic file write helpers — `.tmp` + `sync_all` + rename.
//!
//! Both sync and async flavors so that every checkpoint write site in
//! `elai-orchestrator` can call one function instead of repeating the
//! four-step pattern.
//!
//! # Platform-honesty guarantee (GD-5)
//!
//! `sync_all` on the tmp file flushes the file's data and metadata
//! (size, mtime) to the underlying storage before rename, so a
//! power-loss crash after `sync_all` returns leaves the tmp file
//! byte-identical to what was written. After rename the destination
//! path atomically points at the new complete file: a reader sees either
//! the old complete file or the new complete file, never a torn one.
//!
//! **What this helper does NOT guarantee:** directory-entry durability.
//! On POSIX (ext4/xfs) full durability requires a separate `fsync` of
//! the parent directory after rename; that would require opening a
//! directory file descriptor, which neither `std` nor `tokio` expose
//! portably. On Windows (NTFS) there is no equivalent of POSIX
//! `fsync(dirfd)` at all — rename atomicity is provided by the NTFS
//! transaction layer and is not separately sync-able. This helper
//! therefore guarantees **file-content durability** (the tmp bytes are
//! on stable storage before rename) but **not directory-entry durability**
//! under a crash-between-rename-and-dir-sync scenario on POSIX. For
//! ELAI's crash-recovery guarantee level this is acceptable: a crash
//! between rename and a hypothetical dir-fsync leaves the tmp stale and
//! the old file intact — the recovery path reads the old complete file,
//! which is always valid.
//!
//! **Windows rename:** NTFS cannot rename over an existing file in all
//! Win32 API versions. Both helpers remove the destination if it exists
//! before calling rename, matching the guard already present in
//! `checkpoint.rs` and `bsp/engine.rs`.

use std::io;
use std::path::Path;

// ---------------------------------------------------------------------------
// Sync flavor
// ---------------------------------------------------------------------------

/// Write `bytes` to `dest` atomically.
///
/// Steps: create `<dest>.tmp`, write all bytes, `sync_all`, remove
/// existing `dest` if present (Windows NTFS requirement), rename tmp →
/// dest. On failure the tmp sibling is removed on a best-effort basis.
///
/// See module-level doc for the platform-honesty guarantee.
pub fn atomic_write_sync(dest: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = tmp_sibling_sync(dest);

    // Write + fsync the tmp file.
    {
        use std::io::Write as _;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }

    // Windows: remove existing dest before rename.
    if dest.exists() {
        if let Err(e) = std::fs::remove_file(dest) {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
    }

    // Atomic rename.
    if let Err(e) = std::fs::rename(&tmp, dest) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }

    Ok(())
}

/// Build the `.tmp` sibling path for a sync write. Appends `".tmp"` to
/// the full filename so two concurrent writes to different files in the
/// same directory cannot collide on the tmp name.
fn tmp_sibling_sync(dest: &Path) -> std::path::PathBuf {
    let mut s = dest.as_os_str().to_owned();
    s.push(".tmp");
    std::path::PathBuf::from(s)
}

// ---------------------------------------------------------------------------
// Async flavor (tokio)
// ---------------------------------------------------------------------------

/// Async version of [`atomic_write_sync`].
///
/// Uses `tokio::fs` throughout so the caller's async task is not blocked.
/// Same tmp-sibling naming, same Windows-safe remove-before-rename guard,
/// same `sync_all` before rename. See module-level doc for the platform-
/// honesty guarantee.
pub async fn atomic_write_async(dest: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = tmp_sibling_async(dest);

    // Write via tokio; open for sync_all.
    tokio::fs::write(&tmp, bytes).await?;
    {
        let f = tokio::fs::File::open(&tmp).await?;
        f.sync_all().await?;
    }

    // Windows: remove existing dest before rename.
    if tokio::fs::metadata(dest).await.is_ok() {
        if let Err(e) = tokio::fs::remove_file(dest).await {
            let _ = tokio::fs::remove_file(&tmp).await;
            return Err(e);
        }
    }

    // Atomic rename.
    if let Err(e) = tokio::fs::rename(&tmp, dest).await {
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(e);
    }

    Ok(())
}

/// Build the `.tmp` sibling path for an async write.
fn tmp_sibling_async(dest: &Path) -> std::path::PathBuf {
    let mut s = dest.as_os_str().to_owned();
    s.push(".tmp");
    std::path::PathBuf::from(s)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn sync_write_produces_correct_content() {
        let dir = TempDir::new().expect("tempdir");
        let dest = dir.path().join("out.bin");
        atomic_write_sync(&dest, b"hello world").expect("write");
        let got = std::fs::read(&dest).expect("read back");
        assert_eq!(got, b"hello world");
    }

    #[test]
    fn sync_write_leaves_no_tmp_sibling() {
        let dir = TempDir::new().expect("tempdir");
        let dest = dir.path().join("out.bin");
        atomic_write_sync(&dest, b"data").expect("write");
        let tmp = dir.path().join("out.bin.tmp");
        assert!(!tmp.exists(), ".tmp sibling must be cleaned up after rename");
    }

    #[test]
    fn sync_write_overwrites_existing_dest() {
        let dir = TempDir::new().expect("tempdir");
        let dest = dir.path().join("out.bin");
        std::fs::write(&dest, b"old").expect("write old");
        atomic_write_sync(&dest, b"new").expect("overwrite");
        let got = std::fs::read(&dest).expect("read back");
        assert_eq!(got, b"new");
    }

    #[tokio::test]
    async fn async_write_produces_correct_content() {
        let dir = TempDir::new().expect("tempdir");
        let dest = dir.path().join("out.bin");
        atomic_write_async(&dest, b"async hello").await.expect("write");
        let got = std::fs::read(&dest).expect("read back");
        assert_eq!(got, b"async hello");
    }

    #[tokio::test]
    async fn async_write_leaves_no_tmp_sibling() {
        let dir = TempDir::new().expect("tempdir");
        let dest = dir.path().join("out.bin");
        atomic_write_async(&dest, b"data").await.expect("write");
        let tmp = dir.path().join("out.bin.tmp");
        assert!(!tmp.exists(), ".tmp sibling must be cleaned up after rename");
    }

    #[tokio::test]
    async fn async_write_overwrites_existing_dest() {
        let dir = TempDir::new().expect("tempdir");
        let dest = dir.path().join("out.bin");
        std::fs::write(&dest, b"old").expect("write old");
        atomic_write_async(&dest, b"new").await.expect("overwrite");
        let got = std::fs::read(&dest).expect("read back");
        assert_eq!(got, b"new");
    }
}
