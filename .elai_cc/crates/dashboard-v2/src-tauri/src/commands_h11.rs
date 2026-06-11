//! H-11 (SCHEMA_VERSION 220): scoped filesystem surface for the FileExplorer
//! component and the canvas activity overlay read path.
//!
//! # Commands
//!
//! - [`read_dir`]         -- list directory entries scoped to project_root.
//! - [`read_file`]        -- read a text file scoped to project_root.
//! - [`open_in_editor`]   -- open a path in the system editor (shell-out).
//! - [`mount_root`]       -- register an additional project root.
//! - [`get_canvas_activity`] -- recent CanvasActivityEvent SSB rows for the overlay.
//!
//! # Security
//!
//! ALL path arguments are canonicalized server-side and rejected if they resolve
//! outside the registered project_root. Path-traversal escape (e.g. `../`) is
//! refused at the Rust layer -- the client is NEVER trusted to enforce scope.
//! This is the load-bearing security gate per CANVAS-WORKBENCH-DESIGN.md §6
//! and the contract test (h11_live_layer_fs_contract.rs).
//!
//! NEVER `fs:default` (whole-disk grant) -- only `fs:allow-read-dir` +
//! `fs:allow-read-text-file` + `fs:scope` scoped to project_root(s).
//!
//! cross-lang-pair: dashboard/tauri/api (see stub.ts read_dir / read_file /
//! open_in_editor / mount_root / get_canvas_activity wrappers).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::State;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// A single filesystem entry returned by `read_dir`.
/// Field names MUST match the TypeScript `FsEntry` interface in stub.ts
/// (cross_lang_diff enforced).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FsEntry {
    /// File or directory name (not a full path).
    pub name: String,
    /// Full absolute path (scoped to project_root).
    pub path: String,
    /// "file" or "dir".
    pub kind: String,
    /// Size in bytes (None for directories).
    pub size: Option<u64>,
}

/// A recent canvas activity event returned by `get_canvas_activity`.
/// Mirrors the CanvasActivityPayload shape for the overlay subscriber.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasActivityEventDto {
    pub kind: String,
    pub target_node_id: String,
    pub file_path: String,
    pub symbol_name: Option<String>,
    pub mode: String,
    pub label: Option<String>,
    pub task_id: String,
    pub ts: i64,
}

// ---------------------------------------------------------------------------
// Path-traversal defense (server-side, never trust the client)
// ---------------------------------------------------------------------------

/// Named constant: maximum path-segment depth for rel_path to prevent deep
/// traversal attempts from accumulating canonicalization misses.
const MAX_REL_PATH_SEGMENTS: usize = 64;

/// Resolve `rel_path` relative to `root` and verify the result is strictly
/// within `root`. Returns `Err(String)` on any escape or IO failure.
///
/// Security contract (from h11_live_layer_fs_contract.rs test (d)):
///   canonicalize(root / rel_path).starts_with(canonicalize(root))
fn scope_check(root: &Path, rel_path: &str) -> Result<PathBuf, String> {
    // Reject trivially dangerous inputs without hitting the FS.
    if rel_path.contains("..") {
        return Err(format!(
            "path-traversal rejected: rel_path contains '..': {rel_path:?}"
        ));
    }
    let segment_count = Path::new(rel_path)
        .components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .count();
    if segment_count > MAX_REL_PATH_SEGMENTS {
        return Err(format!(
            "path-traversal rejected: rel_path exceeds {MAX_REL_PATH_SEGMENTS} segments"
        ));
    }

    let joined = root.join(rel_path);
    let canonical_root = std::fs::canonicalize(root).map_err(|e| {
        format!("scope_check: cannot canonicalize root {}: {e}", root.display())
    })?;
    let canonical_joined = std::fs::canonicalize(&joined).map_err(|e| {
        format!(
            "scope_check: cannot canonicalize joined path {}: {e}",
            joined.display()
        )
    })?;
    if !canonical_joined.starts_with(&canonical_root) {
        return Err(format!(
            "path-traversal rejected: {} escapes root {}",
            canonical_joined.display(),
            canonical_root.display()
        ));
    }
    Ok(canonical_joined)
}

/// Validate `abs_path` for traversal attacks and canonicalize it.
///
/// Does NOT check the resulting canonical path against any root registry —
/// the caller is responsible for that check via `AppState::is_path_allowed`.
/// Returns `Err` on: non-absolute input, `..` components, or canonicalize
/// failure (file not found, permission denied).
fn canonicalize_abs_path(abs_path: &str) -> Result<PathBuf, String> {
    let candidate = PathBuf::from(abs_path);
    if !candidate.is_absolute() {
        return Err(format!("path must be absolute, got: {abs_path:?}"));
    }
    // Reject upward traversal components before canonicalizing.
    if abs_path.contains("..") {
        return Err(format!(
            "path-traversal rejected: path contains '..': {abs_path:?}"
        ));
    }
    std::fs::canonicalize(&candidate).map_err(|e| {
        format!(
            "canonicalize_abs_path: cannot canonicalize {}: {e}",
            candidate.display()
        )
    })
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// List directory entries within the project root.
///
/// `root` is the project root path (must match AppState project_root or a
/// registered mount root). `rel_path` is relative to `root` ("" or "." for
/// the root itself). Returns a sorted list of `FsEntry` (dirs first, then
/// files, both alphabetical within each group).
///
/// Security: server-side allowed-root registry check BEFORE any FS access.
/// `root` must be within a root registered in AppState::allowed_roots (seeded
/// with project_root; extended by mount_root). Canonicalize + scope_check
/// then gate rel_path within root. Symlink escapes are caught by OS-level
/// canonicalization.
#[tauri::command]
pub fn read_dir(state: State<'_, AppState>, root: String, rel_path: String) -> Result<Vec<FsEntry>, String> {
    let root_path = PathBuf::from(&root);
    // Canonicalize the client-supplied root before the allowed-root check.
    let canonical_root = std::fs::canonicalize(&root_path)
        .map_err(|e| format!("read_dir: cannot canonicalize root {root:?}: {e}"))?;
    // Server-side allowed-root registry check (security gate, R9-wiring).
    // The project_root is always in the registry; mount_root adds extras.
    if !state.is_path_allowed(&canonical_root) {
        return Err(format!(
            "read_dir: root {root:?} is not in the allowed-root registry. \
             Call mount_root first to register an additional project root."
        ));
    }
    let target = if rel_path.is_empty() || rel_path == "." {
        canonical_root
    } else {
        scope_check(&root_path, &rel_path)?
    };

    let rd = std::fs::read_dir(&target)
        .map_err(|e| format!("read_dir: cannot read {}: {e}", target.display()))?;

    let mut dirs: Vec<FsEntry> = Vec::new();
    let mut files: Vec<FsEntry> = Vec::new();

    for entry_res in rd {
        let entry = entry_res.map_err(|e| format!("read_dir: entry error: {e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path().to_string_lossy().to_string();
        let meta = entry
            .metadata()
            .map_err(|e| format!("read_dir: metadata error for {name}: {e}"))?;
        if meta.is_dir() {
            dirs.push(FsEntry {
                name,
                path,
                kind: "dir".to_string(),
                size: None,
            });
        } else {
            files.push(FsEntry {
                name,
                path,
                kind: "file".to_string(),
                size: Some(meta.len()),
            });
        }
    }

    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    files.sort_by(|a, b| a.name.cmp(&b.name));
    dirs.extend(files);
    Ok(dirs)
}

/// Read a text file scoped to any registered allowed root.
///
/// `path` must be an absolute path within a root registered in
/// `AppState::allowed_roots` (seeded with project_root; extended by
/// `mount_root`). Rejects path-traversal (`..`) and any path outside all
/// registered roots. Returns the UTF-8 contents. Binary files should not be
/// read this way; this command is for source code display in the FileExplorer
/// code viewer pane.
#[tauri::command]
pub fn read_file(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let canonical = canonicalize_abs_path(&path)?;
    if !state.is_path_allowed(&canonical) {
        return Err(format!(
            "read_file: path is not within any registered allowed root: {}",
            canonical.display()
        ));
    }
    std::fs::read_to_string(&canonical)
        .map_err(|e| format!("read_file: {}: {e}", canonical.display()))
}

/// Open a path in the system default editor (non-blocking shell-out).
///
/// Scoped to any registered allowed root (project_root + any `mount_root`
/// additions). Uses `open` (macOS/Linux) or `start ""` (Windows). Returns
/// `Err` if the path is outside all registered roots or if path-traversal is
/// detected. The shell-out itself is non-blocking (fire-and-forget).
#[tauri::command]
pub fn open_in_editor(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let canonical = canonicalize_abs_path(&path)?;
    if !state.is_path_allowed(&canonical) {
        return Err(format!(
            "open_in_editor: path is not within any registered allowed root: {}",
            canonical.display()
        ));
    }

    #[cfg(target_os = "windows")]
    {
        let path_str = canonical
            .to_str()
            .ok_or_else(|| "open_in_editor: non-UTF8 canonical path (Windows)".to_string())?;
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path_str])
            .spawn()
            .map_err(|e| format!("open_in_editor: spawn failed: {e}"))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("open")
            .arg(&canonical)
            .spawn()
            .map_err(|e| format!("open_in_editor: spawn failed: {e}"))?;
    }
    Ok(())
}

/// Register an additional project root (multi-root workspace support).
///
/// The root is added to the AppState allowed-root registry immediately, making
/// it valid for read_dir / read_file / open_in_editor for the lifetime of this
/// Tauri session. Does not persist across restarts (H-11 Tier-1 scope).
///
/// Security: this is the intentional mechanism for extending scope — only the
/// user (clicking a folder picker) calls mount_root. The actual FS commands
/// still scope within the root via scope_check; mount_root only grants
/// root-level access to a path the user explicitly opened.
#[tauri::command]
pub fn mount_root(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let candidate = PathBuf::from(&path);
    if !candidate.is_absolute() {
        return Err(format!("mount_root: path must be absolute: {path:?}"));
    }
    if !candidate.exists() {
        return Err(format!("mount_root: path does not exist: {path:?}"));
    }
    // Register in the server-side allowed-root registry.
    // Returns Err on mutex poison or canonicalize failure so the client can
    // distinguish a successful mount from a silent registration failure.
    let added = state.add_allowed_root(candidate)
        .map_err(|e| format!("mount_root: registration failed: {e}"))?;
    if added {
        tracing::info!(root = %path, "mount_root: registered additional project root in allowed_roots");
    } else {
        tracing::debug!(root = %path, "mount_root: root already registered, no-op");
    }
    Ok(())
}

/// Return the N most recent `canvas_activity_event` SSB rows for the overlay.
///
/// The canvas overlay (canvasActivity.svelte.ts) polls this command at a fixed
/// cadence to drive ghost-node reconciliation. Returns an empty list when no
/// viewer is attached (SSB not yet initialised) or when no rows have landed yet.
///
/// `limit` is capped at 100 to prevent large payloads; 0 or missing defaults to 20.
#[tauri::command]
pub async fn get_canvas_activity(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<CanvasActivityEventDto>, String> {
    let n = limit.unwrap_or(20).min(100) as usize;
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(vec![]),
    };

    let raw = viewer
        .recent_rows_of_type(SsbActionType::CanvasActivityEvent.as_sql_str(), n)
        .map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(raw.len());
    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::CanvasActivityEvent(p) = pl {
            result.push(CanvasActivityEventDto {
                kind: p.kind.as_str().to_owned(),
                target_node_id: p.target_node_id,
                file_path: p.file_path,
                symbol_name: p.symbol_name,
                mode: p.mode,
                label: p.label,
                task_id: p.task_id,
                ts: entry.ts,
            });
        }
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Verify that a relative path with ".." is rejected before hitting the FS.
    #[test]
    fn scope_check_rejects_dotdot_traversal() {
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path();
        let result = scope_check(root, "../etc/passwd");
        assert!(result.is_err(), "scope_check must reject '..' traversal");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("path-traversal rejected"),
            "error must name the rejection: {msg}"
        );
    }

    /// Verify that a valid relative path resolves to a real path inside root.
    #[test]
    fn scope_check_allows_valid_rel_path() {
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path();
        fs::write(root.join("main.rs"), b"fn main() {}").expect("write");
        let result = scope_check(root, "main.rs");
        assert!(result.is_ok(), "scope_check must allow a valid path: {result:?}");
        let resolved = result.unwrap();
        assert!(
            resolved.starts_with(root.canonicalize().expect("canon")),
            "resolved path must be inside root"
        );
    }

    /// Verify that FsEntry serializes with the expected field names.
    #[test]
    fn fs_entry_serializes_expected_fields() {
        let entry = FsEntry {
            name: "lib.rs".to_string(),
            path: "/project/src/lib.rs".to_string(),
            kind: "file".to_string(),
            size: Some(1024),
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        assert!(json.contains("\"name\""), "must contain 'name'");
        assert!(json.contains("\"path\""), "must contain 'path'");
        assert!(json.contains("\"kind\""), "must contain 'kind'");
        assert!(json.contains("\"size\""), "must contain 'size'");
        assert!(json.contains("\"lib.rs\""), "name value must match");
        assert!(json.contains("\"file\""), "kind value must match");
        let parsed: FsEntry = serde_json::from_str(&json).expect("round-trip");
        assert_eq!(parsed.name, "lib.rs");
        assert_eq!(parsed.size, Some(1024));
    }

    /// Verify that CanvasActivityEventDto serializes with the expected field names.
    #[test]
    fn canvas_activity_dto_serializes_expected_fields() {
        let dto = CanvasActivityEventDto {
            kind: "reading".to_string(),
            target_node_id: "src/lib.rs".to_string(),
            file_path: "src/lib.rs".to_string(),
            symbol_name: None,
            mode: "file".to_string(),
            label: Some("reading lib.rs".to_string()),
            task_id: "task-seq-1".to_string(),
            ts: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&dto).expect("serialize");
        assert!(json.contains("\"kind\""), "must contain 'kind'");
        assert!(json.contains("\"target_node_id\""), "must contain 'target_node_id'");
        assert!(json.contains("\"file_path\""), "must contain 'file_path'");
        assert!(json.contains("\"task_id\""), "must contain 'task_id'");
        assert!(json.contains("\"ts\""), "must contain 'ts'");
        let parsed: CanvasActivityEventDto =
            serde_json::from_str(&json).expect("round-trip");
        assert_eq!(parsed.kind, "reading");
        assert_eq!(parsed.ts, 1_700_000_000_000);
    }
}
