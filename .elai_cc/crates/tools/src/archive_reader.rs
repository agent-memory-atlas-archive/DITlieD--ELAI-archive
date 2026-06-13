//! I-53: archive reader — list and read entries from plain .tar archives.
//!
//! SCOPE: plain .tar only (512-byte POSIX header walk, std::io only, no new dep).
//! .tar.gz and .zip are deferred pending anti-slopsquat dep approval (2026-06-13
//! decision doc). Honest errors are returned for unsupported formats.
//!
//! Security invariants:
//!   - No extraction to disk (read-only; bytes returned in-memory).
//!   - Path traversal guard: entries with `..` components or absolute paths are
//!     rejected with a descriptive error (not silently skipped).
//!   - Maximum entry size: `MAX_ENTRY_BYTES` — entries larger than this limit are
//!     truncated to the limit (data, not an error, to avoid crashing on large archives).

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum bytes read from a single archive entry before truncation.
/// Prevents context flooding and OOM on large archives.
const MAX_ENTRY_BYTES: usize = 512 * 1024; // 512 KiB

// ---------------------------------------------------------------------------
// TAR format constants (POSIX ustar / GNU tar header layout)
// ---------------------------------------------------------------------------

const TAR_BLOCK: usize = 512;
const TAR_NAME_OFFSET: usize = 0;
const TAR_NAME_LEN: usize = 100;
const TAR_SIZE_OFFSET: usize = 124;
const TAR_SIZE_LEN: usize = 12;
const TAR_TYPEFLAG_OFFSET: usize = 156;
const TAR_PREFIX_OFFSET: usize = 345;
const TAR_PREFIX_LEN: usize = 155;

/// POSIX 1988 ustar magic ("ustar") for header validation.
const TAR_MAGIC_OFFSET: usize = 257;
const TAR_MAGIC_LEN: usize = 6;

// ---------------------------------------------------------------------------
// Header parsing helpers
// ---------------------------------------------------------------------------

/// Read a null-terminated C string from a fixed-size byte slice.
fn c_str(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

/// Parse an octal size string from a TAR header field.
fn parse_octal(bytes: &[u8]) -> Option<u64> {
    let s = c_str(bytes);
    u64::from_str_radix(s.trim(), 8).ok()
}

/// Check whether a 512-byte block is all zeros (end-of-archive sentinel).
fn is_zero_block(block: &[u8; TAR_BLOCK]) -> bool {
    block.iter().all(|&b| b == 0)
}

// ---------------------------------------------------------------------------
// Path traversal guard
// ---------------------------------------------------------------------------

/// Validate that `entry_path` does not escape the archive root.
/// Returns Err if the path is absolute or contains `..` components.
fn validate_entry_path(entry_path: &str) -> Result<(), String> {
    if entry_path.starts_with('/') || entry_path.starts_with('\\') {
        return Err(format!(
            "archive entry '{entry_path}' has an absolute path (traversal rejected)"
        ));
    }
    for component in entry_path.split('/') {
        if component == ".." {
            return Err(format!(
                "archive entry '{entry_path}' contains '..' component (traversal rejected)"
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Entry metadata
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct TarEntry {
    path: String,
    is_dir: bool,
    size: u64,
    /// Byte offset in the file where this entry's DATA begins (after header block).
    data_offset: u64,
}

// ---------------------------------------------------------------------------
// Parse all entries from a TAR file (header-walk; no decompression)
// ---------------------------------------------------------------------------

fn parse_tar_entries<R: Read + Seek>(reader: &mut R) -> Result<Vec<TarEntry>, String> {
    let mut entries: Vec<TarEntry> = Vec::new();
    let mut position: u64 = 0;

    loop {
        let mut block = [0u8; TAR_BLOCK];
        let n = reader
            .read(&mut block)
            .map_err(|e| format!("archive_read: I/O error reading header block: {e}"))?;
        if n == 0 {
            break; // EOF
        }
        if n < TAR_BLOCK {
            break; // Truncated archive — stop cleanly.
        }
        position += TAR_BLOCK as u64;

        if is_zero_block(&block) {
            // First of two end-of-archive zero blocks — try to read the second.
            let mut second = [0u8; TAR_BLOCK];
            let _ = reader.read(&mut second);
            break;
        }

        // Parse name (with optional ustar prefix for paths > 100 bytes).
        let name = c_str(&block[TAR_NAME_OFFSET..TAR_NAME_OFFSET + TAR_NAME_LEN]);
        let prefix = c_str(&block[TAR_PREFIX_OFFSET..TAR_PREFIX_OFFSET + TAR_PREFIX_LEN]);
        let full_name = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };

        // Validate magic bytes (ustar) — skip blocks that are not valid headers
        // (e.g. extended header blocks from pax format).
        let magic = &block[TAR_MAGIC_OFFSET..TAR_MAGIC_OFFSET + TAR_MAGIC_LEN];
        let magic_str = String::from_utf8_lossy(magic);
        let is_ustar = magic_str.starts_with("ustar");
        // Old-style GNU tar has no magic; tolerate by checking that name is non-empty.
        let has_name = !full_name.trim_matches('/').is_empty();
        if !is_ustar && !has_name {
            // Likely a pax extended header or padding — skip.
            continue;
        }

        // Validate the entry path for traversal attempts.
        let clean_name = full_name.trim_matches('/').to_string();
        if let Err(e) = validate_entry_path(&clean_name) {
            return Err(e);
        }

        let typeflag = block[TAR_TYPEFLAG_OFFSET] as char;
        let is_dir = typeflag == '5' || clean_name.ends_with('/');

        let size = parse_octal(&block[TAR_SIZE_OFFSET..TAR_SIZE_OFFSET + TAR_SIZE_LEN])
            .unwrap_or(0);

        let data_offset = position; // Data starts immediately after the header block.

        entries.push(TarEntry {
            path: clean_name,
            is_dir,
            size,
            data_offset,
        });

        // Advance past the data blocks (rounded up to TAR_BLOCK boundary).
        let data_blocks = size.div_ceil(TAR_BLOCK as u64);
        let skip_bytes = data_blocks * TAR_BLOCK as u64;
        reader
            .seek(SeekFrom::Start(position + skip_bytes))
            .map_err(|e| format!("archive_read: seek past data failed: {e}"))?;
        position += skip_bytes;
    }

    Ok(entries)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all entries in a plain .tar archive.
///
/// Returns JSON: `{"entries": [{"path":..., "is_dir":..., "size":...}]}`
///
/// Returns `Err` for unsupported formats (.tar.gz, .zip) or I/O failures.
pub fn list_archive(path: &Path) -> Result<Value, String> {
    check_format_supported(path)?;

    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("archive_read: cannot open '{}': {e}", path.display()))?;

    let entries = parse_tar_entries(&mut file)?;

    let entry_values: Vec<Value> = entries
        .iter()
        .map(|e| {
            json!({
                "path": e.path,
                "is_dir": e.is_dir,
                "size": e.size,
            })
        })
        .collect();

    Ok(json!({ "entries": entry_values }))
}

/// Read the contents of a specific entry from a plain .tar archive.
///
/// Returns the entry bytes as a UTF-8 string. Non-UTF-8 bytes are hex-encoded
/// with the prefix `<binary>` so agents can identify them without crashing.
/// Entries larger than `MAX_ENTRY_BYTES` are truncated with a marker.
///
/// Returns `Err` for unsupported formats, path traversal attempts, or I/O failures.
pub fn read_archive_entry(archive_path: &Path, entry_path: &str) -> Result<String, String> {
    check_format_supported(archive_path)?;
    validate_entry_path(entry_path)?;

    let mut file = std::fs::File::open(archive_path)
        .map_err(|e| format!("archive_read: cannot open '{}': {e}", archive_path.display()))?;

    let entries = parse_tar_entries(&mut file)?;

    // Find the matching entry (normalize trailing slash for dirs).
    let target = entry_path.trim_matches('/');
    let entry = entries
        .iter()
        .find(|e| e.path.trim_matches('/') == target)
        .ok_or_else(|| format!("archive_read: entry '{entry_path}' not found in archive"))?;

    if entry.is_dir {
        return Err(format!(
            "archive_read: '{entry_path}' is a directory, not a file"
        ));
    }

    // Seek to the data offset and read up to MAX_ENTRY_BYTES.
    file.seek(SeekFrom::Start(entry.data_offset))
        .map_err(|e| format!("archive_read: seek to entry data failed: {e}"))?;

    let read_len = (entry.size as usize).min(MAX_ENTRY_BYTES);
    let mut buf = vec![0u8; read_len];
    file.read_exact(&mut buf)
        .map_err(|e| format!("archive_read: read entry data failed: {e}"))?;

    let truncated = entry.size as usize > MAX_ENTRY_BYTES;

    let content = match std::str::from_utf8(&buf) {
        Ok(s) => {
            if truncated {
                format!("{s}\n<truncated: entry is {} bytes, limit is {MAX_ENTRY_BYTES}>", entry.size)
            } else {
                s.to_string()
            }
        }
        Err(_) => {
            let hex: String = buf
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ");
            if truncated {
                format!("<binary> {hex}\n<truncated: entry is {} bytes, limit is {MAX_ENTRY_BYTES}>", entry.size)
            } else {
                format!("<binary> {hex}")
            }
        }
    };

    Ok(content)
}

// ---------------------------------------------------------------------------
// Format check helper
// ---------------------------------------------------------------------------

fn check_format_supported(path: &Path) -> Result<(), String> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        return Err(
            "archive_read: .tar.gz is not supported in this build (requires flate2 dep; \
             defer to follow-on patch — decision doc 2026-06-13 DECISION-2)"
                .to_string(),
        );
    }
    if name.ends_with(".zip") {
        return Err(
            "archive_read: .zip is not supported in this build (requires zip crate dep; \
             defer to follow-on patch — decision doc 2026-06-13 DECISION-2)"
                .to_string(),
        );
    }
    if !name.ends_with(".tar") {
        return Err(format!(
            "archive_read: unsupported archive format '{}' (only .tar is supported; \
             .tar.gz and .zip deferred to follow-on patch)",
            path.display()
        ));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Build a minimal valid .tar archive in memory.
    ///
    /// Each entry is represented as a (path, content) pair.
    /// Directories must have a path ending in `/` and empty content.
    fn build_tar(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        for (path, data) in entries {
            let mut header = [0u8; TAR_BLOCK];

            // Name field (up to 100 bytes, null-padded).
            let name_bytes = path.as_bytes();
            let name_len = name_bytes.len().min(TAR_NAME_LEN);
            header[TAR_NAME_OFFSET..TAR_NAME_OFFSET + name_len]
                .copy_from_slice(&name_bytes[..name_len]);

            // Type flag: '5' for directory, '0' for regular file.
            let is_dir = path.ends_with('/');
            header[TAR_TYPEFLAG_OFFSET] = if is_dir { b'5' } else { b'0' };

            // Size: 0 for directories.
            let size = if is_dir { 0u64 } else { data.len() as u64 };
            // Octal size string, null-terminated (11 octal digits + null).
            let size_str = format!("{size:011o}\0");
            let size_bytes = size_str.as_bytes();
            header[TAR_SIZE_OFFSET..TAR_SIZE_OFFSET + size_bytes.len().min(TAR_SIZE_LEN)]
                .copy_from_slice(&size_bytes[..size_bytes.len().min(TAR_SIZE_LEN)]);

            // Magic: "ustar\0" at offset 257.
            let magic = b"ustar\0";
            header[TAR_MAGIC_OFFSET..TAR_MAGIC_OFFSET + magic.len()]
                .copy_from_slice(magic);

            // Compute checksum (sum of all header bytes treating the checksum
            // field as spaces).
            for b in header[148..156].iter_mut() {
                *b = b' ';
            }
            let checksum: u32 = header.iter().map(|&b| b as u32).sum();
            let cs_str = format!("{checksum:06o}\0 ");
            header[148..148 + cs_str.len().min(8)]
                .copy_from_slice(&cs_str.as_bytes()[..cs_str.len().min(8)]);

            buf.extend_from_slice(&header);

            if !is_dir && !data.is_empty() {
                buf.extend_from_slice(data);
                // Pad to 512-byte boundary.
                let remainder = TAR_BLOCK - (data.len() % TAR_BLOCK);
                if remainder < TAR_BLOCK {
                    buf.extend(std::iter::repeat(0u8).take(remainder));
                }
            }
        }

        // Two zero blocks = end of archive.
        buf.extend(std::iter::repeat(0u8).take(TAR_BLOCK * 2));
        buf
    }

    fn write_tar_file(dir: &TempDir, name: &str, entries: &[(&str, &[u8])]) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let data = build_tar(entries);
        std::fs::write(&path, &data).expect("write tar");
        path
    }

    #[test]
    fn list_archive_three_entries() {
        let dir = tempfile::tempdir().expect("tempdir");
        let tar_path = write_tar_file(
            &dir,
            "test.tar",
            &[
                ("dir/", b""),
                ("dir/hello.txt", b"hello world"),
                ("dir/data.bin", &[0x00, 0xFF, 0xAB]),
            ],
        );

        let result = list_archive(&tar_path).expect("list_archive");
        let entries = result["entries"].as_array().expect("entries array");
        assert_eq!(entries.len(), 3, "expected 3 entries; got {entries:?}");

        let paths: Vec<&str> = entries
            .iter()
            .map(|e| e["path"].as_str().unwrap_or(""))
            .collect();
        assert!(paths.contains(&"dir"), "should contain dir");
        assert!(paths.contains(&"dir/hello.txt"), "should contain dir/hello.txt");
        assert!(paths.contains(&"dir/data.bin"), "should contain dir/data.bin");
    }

    #[test]
    fn list_archive_rejects_traversal_entry() {
        let dir = tempfile::tempdir().expect("tempdir");
        // Build a tar with a malicious `../escape.txt` entry.
        let tar_path = write_tar_file(
            &dir,
            "malicious.tar",
            &[("../escape.txt", b"evil")],
        );

        let result = list_archive(&tar_path);
        assert!(
            result.is_err(),
            "list_archive must reject '../' traversal entry"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("traversal") || err.contains(".."),
            "error must mention traversal; got: {err}"
        );
    }

    #[test]
    fn read_archive_entry_text() {
        let dir = tempfile::tempdir().expect("tempdir");
        let tar_path = write_tar_file(
            &dir,
            "text.tar",
            &[("readme.txt", b"hello from archive\n")],
        );

        let result = read_archive_entry(&tar_path, "readme.txt").expect("read_archive_entry");
        assert!(
            result.contains("hello from archive"),
            "content should include 'hello from archive'; got: {result}"
        );
    }

    #[test]
    fn read_archive_entry_rejects_path_traversal_request() {
        let dir = tempfile::tempdir().expect("tempdir");
        let tar_path = write_tar_file(&dir, "ok.tar", &[("safe.txt", b"ok")]);

        // Attempt to read a traversal path.
        let result = read_archive_entry(&tar_path, "../../../etc/passwd");
        assert!(result.is_err(), "must reject traversal request");
        let err = result.unwrap_err();
        assert!(
            err.contains("traversal") || err.contains(".."),
            "error must mention traversal; got: {err}"
        );
    }

    #[test]
    fn unsupported_format_tar_gz_returns_honest_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("archive.tar.gz");
        std::fs::write(&path, b"not real gz data").expect("write");
        let result = list_archive(&path);
        assert!(result.is_err(), "tar.gz must return error");
        let err = result.unwrap_err();
        assert!(
            err.contains("not supported") || err.contains("flate2"),
            "error must mention 'not supported' or 'flate2'; got: {err}"
        );
    }

    #[test]
    fn unsupported_format_zip_returns_honest_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("archive.zip");
        std::fs::write(&path, b"PK\x03\x04fake").expect("write");
        let result = list_archive(&path);
        assert!(result.is_err(), "zip must return error");
        let err = result.unwrap_err();
        assert!(
            err.contains("not supported") || err.contains("zip"),
            "error must mention 'not supported' or 'zip'; got: {err}"
        );
    }
}
