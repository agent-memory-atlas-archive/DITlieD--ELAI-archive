//! H-49 — time-travel Tauri command surface.
//!
//! Exposes undo / redo / history / jump to the dashboard frontend.
//! The Tauri commands share the SAME CAS + SessionDag restore logic as the
//! CLI (`crates/cli/src/timetravel_cmd.rs`); the restore orchestration lives
//! in `elai_orchestrator::architecture::timetravel` so both callers compose
//! on one implementation (not a fork).
//!
//! Kill-switch: `ELAI_TIMETRAVEL_DISABLE=1` blocks all time-travel from the
//! dashboard (mirrors the CLI kill-switch; default-ON feature).
//!
//! Cross-lang pair: BranchTree.svelte + CommandWindow.svelte invoke()
//! + stub.ts wrappers + types.ts DTOs must all land in the same edit batch
//! (cross_lang_diff.py enforces this).
//!
//! Grounding:
//!   - crates/orchestrator/src/architecture/timetravel.rs (CAS + materialize)
//!   - crates/orchestrator/src/session_dag.rs (HEAD + children_of + file_manifest)
//!   - crates/cli/src/timetravel_cmd.rs (CLI dispatcher, shared engine)

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::architecture::timetravel::{
    manifest_get, materialize, timetravel_enabled, CasRestoreEngine, RestoreEngine,
    ELAI_TIMETRAVEL_DISABLE_VAR, SESSION_CHECKPOINT_ROOT,
};
use elai_orchestrator::session_dag::SessionDag;
use elai_orchestrator::ssb::{
    EditHistoryEventKind, EditHistoryEventPayload, SsbPayload,
};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Request / Response DTOs
// ---------------------------------------------------------------------------

/// Response shape for time-travel commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeTravelResponse {
    /// Whether the kill-switch suppressed this operation.
    pub disabled: bool,
    /// Whether this was a no-op (HEAD already at target).
    pub no_op: bool,
    /// The SSB seq of the emitted beacon, if any (0 = none).
    pub beacon_seq: i64,
    /// The current HEAD snap_id after the operation.
    pub head: Option<String>,
    /// Human-readable summary of what happened.
    pub summary: String,
    /// Whether divergence was detected and fork-captured (RV-1).
    pub divergence_detected: bool,
}

/// One node in the undo-tree history listing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeTravelNode {
    pub snap_id: String,
    pub parent_id: Option<String>,
    pub is_head: bool,
    /// Number of children (>1 means this node is a branch point).
    pub children_count: usize,
    pub tag: Option<String>,
    pub created_at_ms: u64,
    /// blake3 hash of the file manifest at this node, if any.
    pub manifest_hash: Option<String>,
}

/// Request shape for jump — identifies the target snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeTravelJumpRequest {
    pub session_id: String,
    /// Target snap_id to jump to.
    pub target: String,
    /// "edit" | "turn" | "babystep" (default "turn")
    pub grain: Option<String>,
    /// "code" | "conversation" | "both" (default "code")
    pub scope: Option<String>,
    /// If true, print what would happen but touch no disk.
    pub dry_run: Option<bool>,
}

// ---------------------------------------------------------------------------
// Shared restore helper
// ---------------------------------------------------------------------------

/// Open the SessionDag for a given session under the project root.
fn open_dag(session_id: &str) -> Result<(PathBuf, SessionDag), String> {
    let project_root = std::env::current_dir()
        .map_err(|e| format!("timetravel: cwd: {e}"))?;
    // Mirror the path convention from timetravel_cmd.rs.
    let dag_dir = elai_core::paths::sessions_dir_path(&project_root)
        .join(session_id);
    let dag_path = dag_dir.join("session_dag.db");

    let dag = SessionDag::open(session_id, &dag_path)
        .map_err(|e| format!("timetravel: open_dag {session_id}: {e}"))?;
    Ok((project_root, dag))
}

/// Core undo logic shared by CLI and Tauri.
///
/// Walks HEAD -> parent, materializes the parent's manifest.
/// RV-1 divergence guard fires inside `materialize()`.
/// Returns `(head_after, divergence_detected, no_op)`.
pub(crate) fn do_undo(
    session_id: &str,
    scope: &str,
) -> Result<(Option<String>, bool, bool), String> {
    let (project_root, dag) = open_dag(session_id)?;
    let cas_root = project_root.join(SESSION_CHECKPOINT_ROOT);

    let head = dag
        .get_head()
        .map_err(|e| format!("timetravel undo: get_head: {e}"))?;

    let head = match head {
        Some(h) => h,
        None => return Ok((None, false, true)), // no HEAD — no-op
    };

    let (snap, _msgs) = dag
        .load_snapshot(&head)
        .map_err(|e| format!("timetravel undo: load_snapshot {head}: {e}"))?;

    let parent_id = match snap.parent_id {
        Some(p) => p,
        None => return Ok((Some(head), false, true)), // at root — no-op
    };

    if scope == "conversation" {
        // conversation-only undo: just move HEAD, no file restore.
        dag.set_head(&parent_id)
            .map_err(|e| format!("timetravel undo: set_head: {e}"))?;
        return Ok((Some(parent_id), false, false));
    }

    // Load parent manifest and materialize files.
    let manifest_hash = dag
        .get_file_manifest(&parent_id)
        .map_err(|e| format!("timetravel undo: get_file_manifest: {e}"))?;

    let mut any_divergence = false;
    if let Some(mh) = manifest_hash {
        let manifest = manifest_get(&cas_root, session_id, &mh)
            .map_err(|e| format!("timetravel undo: manifest_get: {e}"))?;
        let engine = CasRestoreEngine;
        let results = engine
            .materialize_manifest(&cas_root, session_id, &manifest, &project_root)
            .map_err(|e| format!("timetravel undo: materialize: {e}"))?;
        any_divergence = results.iter().any(|r| r.divergence_detected);
    }

    dag.set_head(&parent_id)
        .map_err(|e| format!("timetravel undo: set_head: {e}"))?;

    Ok((Some(parent_id), any_divergence, false))
}

/// Core redo logic shared by CLI and Tauri.
///
/// Walks HEAD -> last child (or prompts on ambiguity by returning first child).
/// Returns `(head_after, no_op)`.
pub(crate) fn do_redo(
    session_id: &str,
    scope: &str,
) -> Result<(Option<String>, bool), String> {
    let (project_root, dag) = open_dag(session_id)?;
    let cas_root = project_root.join(SESSION_CHECKPOINT_ROOT);

    let head = dag
        .get_head()
        .map_err(|e| format!("timetravel redo: get_head: {e}"))?;

    let head = match head {
        Some(h) => h,
        None => return Ok((None, true)),
    };

    let children = dag
        .children_of(&head)
        .map_err(|e| format!("timetravel redo: children_of: {e}"))?;

    let target = match children.first() {
        Some(c) => c.clone(),
        None => return Ok((Some(head), true)), // at tip — no-op
    };

    if scope != "conversation" {
        let manifest_hash = dag
            .get_file_manifest(&target)
            .map_err(|e| format!("timetravel redo: get_file_manifest: {e}"))?;
        if let Some(mh) = manifest_hash {
            let manifest = manifest_get(&cas_root, session_id, &mh)
                .map_err(|e| format!("timetravel redo: manifest_get: {e}"))?;
            let engine = CasRestoreEngine;
            engine
                .materialize_manifest(&cas_root, session_id, &manifest, &project_root)
                .map_err(|e| format!("timetravel redo: materialize: {e}"))?;
        }
    }

    dag.set_head(&target)
        .map_err(|e| format!("timetravel redo: set_head: {e}"))?;

    Ok((Some(target), false))
}

/// Core jump logic shared by CLI and Tauri.
pub(crate) fn do_jump(
    session_id: &str,
    target: &str,
    scope: &str,
    dry_run: bool,
) -> Result<(Option<String>, bool), String> {
    let (project_root, dag) = open_dag(session_id)?;
    let cas_root = project_root.join(SESSION_CHECKPOINT_ROOT);

    let head = dag
        .get_head()
        .map_err(|e| format!("timetravel jump: get_head: {e}"))?;

    if head.as_deref() == Some(target) {
        return Ok((Some(target.to_owned()), true)); // no-op
    }

    if dry_run {
        tracing::info!(
            "timetravel jump --dry-run: would restore {} -> {}",
            head.as_deref().unwrap_or("(none)"),
            target
        );
        return Ok((head, false));
    }

    if scope != "conversation" {
        let manifest_hash = dag
            .get_file_manifest(target)
            .map_err(|e| format!("timetravel jump: get_file_manifest: {e}"))?;
        if let Some(mh) = manifest_hash {
            let manifest = manifest_get(&cas_root, session_id, &mh)
                .map_err(|e| format!("timetravel jump: manifest_get: {e}"))?;
            materialize(&cas_root, session_id, &manifest, &project_root)
                .map_err(|e| format!("timetravel jump: materialize: {e}"))?;
        }
    }

    dag.set_head(target)
        .map_err(|e| format!("timetravel jump: set_head: {e}"))?;

    Ok((Some(target.to_owned()), false))
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// H-49 — undo the most recent LLM file edit(s).
///
/// Restores files to the state captured before the last edit at the requested
/// grain. Emits `SessionRewound` (H-35 reused) and `EditHistoryEvent` if an
/// SSB writer is available.
///
/// Kill-switch: `ELAI_TIMETRAVEL_DISABLE=1` suppresses.
///
/// Cross-lang pair: BranchTree.svelte + CommandWindow.svelte + stub.ts.
#[tauri::command]
pub async fn timetravel_undo(
    state: State<'_, AppState>,
    session_id: String,
    scope: Option<String>,
) -> Result<TimeTravelResponse, String> {
    if !timetravel_enabled() {
        tracing::info!("timetravel_undo: suppressed by {} kill-switch", ELAI_TIMETRAVEL_DISABLE_VAR);
        return Ok(TimeTravelResponse {
            disabled: true,
            no_op: false,
            beacon_seq: 0,
            head: None,
            summary: "suppressed by kill-switch".into(),
            divergence_detected: false,
        });
    }

    let scope_str = scope.as_deref().unwrap_or("code");
    let (head_after, divergence, no_op) = do_undo(&session_id, scope_str)?;

    // Emit SSB beacon if a writer is available.
    let mut beacon_seq: i64 = 0;
    if !no_op {
        if let Some(writer) = state.writer.as_ref() {
            let payload = SsbPayload::EditHistoryEvent(EditHistoryEventPayload {
                kind: if divergence {
                    EditHistoryEventKind::Branched
                } else {
                    EditHistoryEventKind::SnapshotCaptured
                },
                node_id: head_after.clone().unwrap_or_default(),
                parent_id: None,
                paths_n: 0,
                pre_hash: None,
                post_hash: None,
                grain: "turn".into(),
            });
            beacon_seq = writer
                .write_payload("dashboard:timetravel", 0, payload, vec![])
                .await
                .unwrap_or(0);
        }
    }

    Ok(TimeTravelResponse {
        disabled: false,
        no_op,
        beacon_seq,
        head: head_after,
        summary: if no_op { "no-op (already at root)".into() } else { "undo complete".into() },
        divergence_detected: divergence,
    })
}

/// H-49 — redo the previously undone LLM edit.
///
/// Advances HEAD to the last-followed child and re-materializes its manifest.
///
/// Cross-lang pair: BranchTree.svelte + CommandWindow.svelte + stub.ts.
#[tauri::command]
pub async fn timetravel_redo(
    state: State<'_, AppState>,
    session_id: String,
    scope: Option<String>,
) -> Result<TimeTravelResponse, String> {
    if !timetravel_enabled() {
        tracing::info!("timetravel_redo: suppressed by {} kill-switch", ELAI_TIMETRAVEL_DISABLE_VAR);
        return Ok(TimeTravelResponse {
            disabled: true,
            no_op: false,
            beacon_seq: 0,
            head: None,
            summary: "suppressed by kill-switch".into(),
            divergence_detected: false,
        });
    }

    let scope_str = scope.as_deref().unwrap_or("code");
    let (head_after, no_op) = do_redo(&session_id, scope_str)?;

    let mut beacon_seq: i64 = 0;
    if !no_op {
        if let Some(writer) = state.writer.as_ref() {
            let payload = SsbPayload::EditHistoryEvent(EditHistoryEventPayload {
                kind: EditHistoryEventKind::Redo,
                node_id: head_after.clone().unwrap_or_default(),
                parent_id: None,
                paths_n: 0,
                pre_hash: None,
                post_hash: None,
                grain: "turn".into(),
            });
            beacon_seq = writer
                .write_payload("dashboard:timetravel", 0, payload, vec![])
                .await
                .unwrap_or(0);
        }
    }

    Ok(TimeTravelResponse {
        disabled: false,
        no_op,
        beacon_seq,
        head: head_after,
        summary: if no_op { "no-op (already at tip)".into() } else { "redo complete".into() },
        divergence_detected: false,
    })
}

/// H-49 — jump HEAD to a specific snapshot node.
///
/// Restores files from that node's manifest. Emits `EditHistoryEvent kind=Jumped`.
/// `--dry-run` prints what would change without touching disk.
///
/// Cross-lang pair: BranchTree.svelte + CommandWindow.svelte + stub.ts.
#[tauri::command]
pub async fn timetravel_jump(
    state: State<'_, AppState>,
    request: TimeTravelJumpRequest,
) -> Result<TimeTravelResponse, String> {
    if !timetravel_enabled() {
        tracing::info!("timetravel_jump: suppressed by {} kill-switch", ELAI_TIMETRAVEL_DISABLE_VAR);
        return Ok(TimeTravelResponse {
            disabled: true,
            no_op: false,
            beacon_seq: 0,
            head: None,
            summary: "suppressed by kill-switch".into(),
            divergence_detected: false,
        });
    }

    let scope_str = request.scope.as_deref().unwrap_or("code");
    let dry_run = request.dry_run.unwrap_or(false);
    let (head_after, no_op) = do_jump(&request.session_id, &request.target, scope_str, dry_run)?;

    let mut beacon_seq: i64 = 0;
    if !no_op && !dry_run {
        if let Some(writer) = state.writer.as_ref() {
            let payload = SsbPayload::EditHistoryEvent(EditHistoryEventPayload {
                kind: EditHistoryEventKind::Jumped,
                node_id: request.target.clone(),
                parent_id: None,
                paths_n: 0,
                pre_hash: None,
                post_hash: None,
                grain: request.grain.clone().unwrap_or_else(|| "turn".into()),
            });
            beacon_seq = writer
                .write_payload("dashboard:timetravel", 0, payload, vec![])
                .await
                .unwrap_or(0);
        }
    }

    Ok(TimeTravelResponse {
        disabled: false,
        no_op,
        beacon_seq,
        head: head_after,
        summary: if no_op {
            "no-op (already at target)".into()
        } else if dry_run {
            format!("dry-run: would jump to {}", request.target)
        } else {
            format!("jumped to {}", request.target)
        },
        divergence_detected: false,
    })
}

/// H-49 — return the undo-tree nodes for the given session.
///
/// Used by BranchTree.svelte to render the timeline: current node marked,
/// sibling branches shown, redo affordance indicated.
///
/// Cross-lang pair: BranchTree.svelte + stub.ts TimeTravelNode.
#[tauri::command]
pub async fn timetravel_history(
    _state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<TimeTravelNode>, String> {
    let (_, dag) = open_dag(&session_id)?;

    let head = dag
        .get_head()
        .map_err(|e| format!("timetravel_history: get_head: {e}"))?;

    // Walk the parent chain from HEAD to root.
    let head_id = match &head {
        Some(h) => h.clone(),
        None => return Ok(Vec::new()),
    };

    let chain = dag
        .parent_chain(&head_id)
        .map_err(|e| format!("timetravel_history: parent_chain: {e}"))?;

    let mut nodes: Vec<TimeTravelNode> = Vec::new();
    for snap in &chain {
        let children = dag
            .children_of(&snap.snap_id)
            .map_err(|e| format!("timetravel_history: children_of {}: {e}", snap.snap_id))?;
        let manifest_hash = dag
            .get_file_manifest(&snap.snap_id)
            .unwrap_or(None);
        nodes.push(TimeTravelNode {
            snap_id: snap.snap_id.clone(),
            parent_id: snap.parent_id.clone(),
            is_head: head.as_deref() == Some(&snap.snap_id),
            children_count: children.len(),
            tag: snap.tag.clone(),
            created_at_ms: snap.created_at,
            manifest_hash,
        });
    }

    Ok(nodes)
}

/// H-49 — restore files only (no conversation state change).
///
/// Thin variant of jump that only changes on-disk files, not the HEAD pointer.
/// Matches Cline's "Restore Files Only" affordance.
///
/// Cross-lang pair: CommandWindow.svelte + stub.ts.
#[tauri::command]
pub async fn timetravel_restore_files_only(
    _state: State<'_, AppState>,
    session_id: String,
    target: String,
) -> Result<TimeTravelResponse, String> {
    if !timetravel_enabled() {
        return Ok(TimeTravelResponse {
            disabled: true,
            no_op: false,
            beacon_seq: 0,
            head: None,
            summary: "suppressed by kill-switch".into(),
            divergence_detected: false,
        });
    }

    let project_root = std::env::current_dir()
        .map_err(|e| format!("timetravel restore_files_only: cwd: {e}"))?;
    let cas_root = project_root.join(SESSION_CHECKPOINT_ROOT);
    let dag_dir = elai_core::paths::sessions_dir_path(&project_root).join(&session_id);
    let dag_path = dag_dir.join("session_dag.db");
    let dag = SessionDag::open(&session_id, &dag_path)
        .map_err(|e| format!("timetravel restore_files_only: open_dag: {e}"))?;

    let manifest_hash = dag
        .get_file_manifest(&target)
        .map_err(|e| format!("timetravel restore_files_only: get_file_manifest: {e}"))?;

    let no_op = manifest_hash.is_none();
    let mut divergence_detected = false;

    if let Some(mh) = manifest_hash {
        let manifest = manifest_get(&cas_root, &session_id, &mh)
            .map_err(|e| format!("timetravel restore_files_only: manifest_get: {e}"))?;
        let results = materialize(&cas_root, &session_id, &manifest, &project_root)
            .map_err(|e| format!("timetravel restore_files_only: materialize: {e}"))?;
        divergence_detected = results.iter().any(|r| r.divergence_detected);
    }

    // Intentionally does NOT call set_head — files only, conversation state untouched.
    Ok(TimeTravelResponse {
        disabled: false,
        no_op,
        beacon_seq: 0,
        head: None, // HEAD unchanged
        summary: if no_op {
            "no-op (no manifest at target)".into()
        } else {
            format!("files restored to {target}")
        },
        divergence_detected,
    })
}
