//! E-1 — Chat-memory / `memory_notes` + `core_blocks` Tauri command surface.
//!
//! Two read-only commands feed the `MemoryNotesPanel.svelte` panel:
//!
//! - [`get_memory_notes`] — paginated listing of currently-valid rows
//!   from the A-MEM Zettelkasten store (arXiv:2502.12110 §3.2),
//!   filtered by optional `scope` (`"session"` | `"project"` | `"global"`).
//! - [`get_chat_memory_state`] — composite snapshot used by the drawer
//!   header: current `CoreBlock` body + byte-size, most-recent
//!   ChatReflectionCompleted SSB timestamp, recent-notes count, and a
//!   placeholder `sleep_compute_active` flag (FSM substate query needs
//!   a hook that is not surfaced through `AppState` yet — followup row
//!   in the executor brief).
//!
//! Per the executor brief, this is an ADDITIVE surface that lives in its
//! own module to avoid racing the other E-plan orchestration sessions
//! editing `commands.rs`. No mutation commands ship in this slice (the
//! orchestrator's chat-turn-committed hook owns inserts).

use std::path::PathBuf;

use elai_core::paths::memory_dir_path;
use elai_memory::{CoreBlockStore, NoteStore};
use elai_orchestrator::ssb::SsbActionType;
use elai_orchestrator::SsbReader;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// Default limit for [`get_memory_notes`] when the caller omits one.
const DEFAULT_LIMIT: usize = 50;
/// Hard cap to bound the SQL `LIMIT` from a malicious / runaway caller.
const MAX_LIMIT: usize = 500;

/// Single-row sqlite filename for the `memory_notes` store. Lives in
/// the same `<project_root>/.elai_cc/memory/` directory as the E-5
/// `memory_facts.sqlite`. The shared parent dir lets a future
/// orchestration step union the two stores at retrieval time without
/// re-resolving paths.
const MEMORY_NOTES_DB_FILENAME: &str = "memory_notes.sqlite";

/// Sibling sqlite filename for the MemGPT-style `core_blocks` table
/// (arXiv:2310.08560 §3.2). Same parent dir as `memory_notes.sqlite`.
const CORE_BLOCKS_DB_FILENAME: &str = "core_blocks.sqlite";

/// Serializable mirror of [`elai_memory::MemoryNote`]. The wire shape
/// is intentionally narrow — we drop the `keywords` / `context` /
/// `fact_type` columns that the Svelte drawer does not render today.
/// Adding columns is additive (Svelte ignores unknown fields) so a
/// future drawer revision can light them up without a schema bump.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNoteRow {
    /// UUIDv4 hex (`memory_notes.note_id`).
    pub note_id: String,
    /// `"session"` | `"project"` | `"global"`.
    pub scope: String,
    /// Free-form tag array.
    pub tags: Vec<String>,
    /// Note body — head-truncated client-side; we ship the full string
    /// so the drawer can pick its own ellipsis policy.
    pub body: String,
    /// `note_id` references to other notes (Zettelkasten adjacency).
    pub links: Vec<String>,
    /// `link_count` derived from `links.len()` — pre-computed so the
    /// Svelte panel does not double-loop.
    pub link_count: u32,
    /// ms-since-epoch of row creation.
    pub created_at_ms: u64,
    /// ms-since-epoch of the most recent PPR-walker hit (drives the
    /// "X min ago" relative-age render).
    pub last_accessed_ms: u64,
    /// [0.0, 1.0] — initial trust, decayed by `elai_memory::decay`.
    pub trust_score: f32,
}

impl From<elai_memory::MemoryNote> for MemoryNoteRow {
    fn from(n: elai_memory::MemoryNote) -> Self {
        let link_count = n.links.len() as u32;
        MemoryNoteRow {
            note_id: n.note_id,
            scope: n.scope,
            tags: n.tags,
            body: n.body,
            links: n.links,
            link_count,
            created_at_ms: n.created_at_ms as u64,
            last_accessed_ms: n.last_accessed_ms as u64,
            trust_score: n.trust_score,
        }
    }
}

/// Result shape returned by [`get_memory_notes`].
#[derive(Debug, Clone, Serialize)]
pub struct MemoryNotesResponse {
    /// Absolute path to the underlying sqlite file. Surfaced so the
    /// drawer can render "store: <path>" + distinguish "store missing"
    /// (drawer shows empty state) from "store exists, no rows".
    pub store_path: String,
    /// `true` when the store file did not exist at command time (lazy
    /// open creates it; response then includes the empty `notes` list).
    pub store_just_created: bool,
    /// Currently-valid notes, most-recent first, filtered by `scope`.
    pub notes: Vec<MemoryNoteRow>,
}

/// Composite header snapshot returned by [`get_chat_memory_state`].
///
/// Fields use snake_case to match Tauri's auto-derived JSON wire
/// format (every other dashboard DTO follows the same convention).
#[derive(Debug, Clone, Serialize)]
pub struct ChatMemoryState {
    /// Current `CoreBlock` body. Empty string when no row exists yet.
    pub core_block_text: String,
    /// Byte size of the body — always <= [`elai_memory::CORE_BLOCK_MAX_BYTES`].
    pub core_block_bytes: u32,
    /// ms-since-epoch of the most recent `ChatReflectionCompleted`
    /// SSB row, or `None` when no reflection has fired this session.
    pub last_reflection_ms: Option<u64>,
    /// FSM SLEEP_COMPUTE substate flag. Hardcoded to `false` for now
    /// — the orchestrator's `FsmSubstate::SleepCompute` is not exposed
    /// through `AppState` yet; the wiring lives in `FsmInternalState`
    /// inside the runner loop, which dashboard commands don't currently
    /// read. Followup row in the executor brief.
    pub sleep_compute_active: bool,
    /// Count of currently-valid `memory_notes` rows
    /// (`NoteStore::count_current`).
    pub recent_notes_count: u32,
}

fn resolve_memory_dir(state: &AppState) -> PathBuf {
    memory_dir_path(&state.project_root_snapshot())
}

fn open_note_store(state: &AppState) -> Result<(NoteStore, PathBuf, bool), String> {
    let path = resolve_memory_dir(state).join(MEMORY_NOTES_DB_FILENAME);
    let already_existed = path.exists();
    let store = NoteStore::open(&path).map_err(|e| e.to_string())?;
    Ok((store, path, !already_existed))
}

fn open_core_block_store(state: &AppState) -> Result<(CoreBlockStore, PathBuf, bool), String> {
    let path = resolve_memory_dir(state).join(CORE_BLOCKS_DB_FILENAME);
    let already_existed = path.exists();
    let store = CoreBlockStore::open(&path).map_err(|e| e.to_string())?;
    Ok((store, path, !already_existed))
}

fn normalize_scope(raw: Option<String>) -> Option<String> {
    match raw.as_deref() {
        None => None,
        Some("") | Some("all") | Some("any") => None,
        Some(other) => Some(other.to_owned()),
    }
}

fn matches_scope(note_scope: &str, filter: Option<&str>) -> bool {
    match filter {
        None => true,
        Some(s) => note_scope == s,
    }
}

/// Tauri command — list currently-valid `memory_notes` rows.
///
/// `scope` is an optional client-side filter; passing `None` /
/// `"all"` returns every row up to `limit`. The store is opened
/// lazily — if the underlying sqlite file does not exist yet, the
/// command creates an empty database and returns `notes: []` with
/// `store_just_created: true` so the drawer can render the
/// first-run-empty-state.
#[tauri::command]
pub async fn get_memory_notes(
    state: State<'_, AppState>,
    scope: Option<String>,
    limit: Option<usize>,
) -> Result<MemoryNotesResponse, String> {
    let scope_filter = normalize_scope(scope);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);

    let (store, path, store_just_created) = open_note_store(&state)?;
    // Pull a generous superset, then filter client-side. `list_recent`
    // already orders DESC by `created_at_ms`, so the slice we keep is
    // already in the order the panel wants.
    let raw_limit = if scope_filter.is_some() {
        // Filtering throws rows away — overshoot so the post-filter
        // list still hits `limit` when most rows are in-scope.
        (limit * 4).min(MAX_LIMIT)
    } else {
        limit
    };
    let rows = store
        .list_recent(raw_limit)
        .map_err(|e| e.to_string())?;
    let scope_str = scope_filter.as_deref();
    let notes: Vec<MemoryNoteRow> = rows
        .into_iter()
        .filter(|n| matches_scope(&n.scope, scope_str))
        .take(limit)
        .map(MemoryNoteRow::from)
        .collect();

    Ok(MemoryNotesResponse {
        store_path: path.to_string_lossy().into_owned(),
        store_just_created,
        notes,
    })
}

/// Tauri command — composite drawer-header snapshot.
///
/// Reads from three sources:
/// 1. [`CoreBlockStore`] for the current body bytes;
/// 2. [`NoteStore::count_current`] for the recent-notes count;
/// 3. The dashboard's SSB writer broadcast for the most-recent
///    `ChatReflectionCompleted` timestamp (best-effort — when no writer
///    is attached, `last_reflection_ms` returns `None`).
///
/// `sleep_compute_active` is currently hardcoded to `false`. Wiring it
/// requires a hook on `FsmInternalState::substate` that is not
/// surfaced through `AppState` — see the followup row in the executor
/// brief for the producer-side change in `task_runner.rs`.
#[tauri::command]
pub async fn get_chat_memory_state(
    state: State<'_, AppState>,
) -> Result<ChatMemoryState, String> {
    // CoreBlock body — single-user installs use `"default"` / `"global"`.
    let core = match open_core_block_store(&state) {
        Ok((store, _path, _just)) => store
            .get("default", "global")
            .map_err(|e| e.to_string())?
            .map(|cb| (cb.content, cb.updated_at_ms))
            .unwrap_or_else(|| (String::new(), 0u128)),
        Err(e) => {
            tracing::warn!(
                error = %e,
                "dashboard-v2: core_block_store open failed; \
                 returning empty body in get_chat_memory_state"
            );
            (String::new(), 0u128)
        }
    };
    let core_block_bytes = u32::try_from(core.0.as_bytes().len()).unwrap_or(u32::MAX);

    // Recent-notes count.
    let recent_notes_count = match open_note_store(&state) {
        Ok((notes, _path, _just)) => notes
            .count_current()
            .map_err(|e| e.to_string())?
            .min(u32::MAX as usize) as u32,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "dashboard-v2: note_store open failed; \
                 returning recent_notes_count=0 in get_chat_memory_state"
            );
            0u32
        }
    };

    // Most-recent reflection timestamp via the dashboard's SSB writer.
    // When no writer is attached (fresh install / no SSB DB), surface
    // `None` rather than an error — the drawer renders "never" cleanly.
    let last_reflection_ms = match state.writer.as_ref() {
        Some(writer) => writer
            .query_by_type(SsbActionType::ChatReflectionCompleted)
            .ok()
            .and_then(|rows| rows.last().map(|r| r.created_at as u64)),
        None => None,
    };

    Ok(ChatMemoryState {
        core_block_text: core.0,
        core_block_bytes,
        last_reflection_ms,
        // Followup: hook FsmInternalState::substate so this reflects
        // SLEEP_COMPUTE without polling SSB rows.
        sleep_compute_active: false,
        recent_notes_count,
    })
}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_scope_pass_through() {
        assert!(matches_scope("session", None));
        assert!(matches_scope("session", Some("session")));
        assert!(!matches_scope("project", Some("session")));
        assert!(matches_scope("global", Some("global")));
    }

    #[test]
    fn normalize_scope_collapses_synonyms() {
        assert_eq!(normalize_scope(None), None);
        assert_eq!(normalize_scope(Some(String::new())), None);
        assert_eq!(normalize_scope(Some("all".into())), None);
        assert_eq!(normalize_scope(Some("any".into())), None);
        assert_eq!(normalize_scope(Some("session".into())), Some("session".into()));
    }

    #[test]
    fn memory_note_row_from_preserves_link_count() {
        let n = elai_memory::MemoryNote {
            note_id: "id".into(),
            scope: "session".into(),
            tags: vec!["a".into()],
            keywords: vec![],
            context: String::new(),
            body: "body".into(),
            links: vec!["x".into(), "y".into(), "z".into()],
            fact_type: "note".into(),
            created_at_ms: 1_000_000,
            last_accessed_ms: 2_000_000,
            valid_to_ms: None,
            trust_score: 0.7,
        };
        let row: MemoryNoteRow = n.into();
        assert_eq!(row.link_count, 3);
        assert_eq!(row.created_at_ms, 1_000_000);
        assert_eq!(row.last_accessed_ms, 2_000_000);
    }

    /// Compile-time check that the path helpers we depend on stay
    /// stable. Wraps the path-helper import so the IDE flags any future
    /// signature change.
    #[allow(dead_code)]
    fn _path_helper_compile_check(root: &std::path::Path) {
        let _ = memory_dir_path(root);
    }
}
