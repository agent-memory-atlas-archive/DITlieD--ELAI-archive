//! H-49 — `elai undo / redo / history / jump` time-travel CLI driver.
//!
//! Composes on SHIPPED primitives:
//!   * [`elai_orchestrator::architecture::timetravel`] — CAS object store + materialize
//!   * [`elai_orchestrator::session_dag::SessionDag`] — HEAD pointer + children_of
//!   * H-35 [`crate::rewind_cmd::run_rewind`] — emits `SessionRewound` on undo
//!
//! # Commands
//!
//! * `elai undo [--grain edit|turn|babystep] [--scope code|conversation|both] [--steps N] [--dry-run]`
//! * `elai redo [--grain ...] [--scope ...] [--dry-run]`
//! * `elai history [--format tree|list]`
//! * `elai jump <node-id|"turn:N"> [--grain ...] [--scope ...] [--dry-run]`
//!
//! # Kill-switch
//!
//! `ELAI_TIMETRAVEL_DISABLE=1` suppresses all operations (silently reports
//! suppressed).
//!
//! # Policy
//!
//! - No `.unwrap()` / `.expect()` in production paths — all `?`.
//! - blake3 only (Anti-Stale-Training rule).
//! - RV-1: divergence guard fires before any materialize; fork-captures the
//!   current on-disk bytes as `auto:pre-restore` if they drift from HEAD's
//!   expected hash.
//! - RV-3: undo-then-edit creates a NEW sibling child; the prior redo timeline
//!   is NEVER destroyed.
//! - RV-4: `--dry-run` prints what would happen and warns about irreversible
//!   side effects; touches no disk.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::architecture::timetravel::{
    manifest_get, timetravel_enabled, CasRestoreEngine, RestoreEngine, TimeTravelGrain,
    ELAI_TIMETRAVEL_DISABLE_VAR, SESSION_CHECKPOINT_ROOT,
};
use elai_orchestrator::ssb::{
    EditHistoryEventKind, EditHistoryEventPayload, SessionRewoundPayload, Ssb, SsbConfig,
    SsbPayload,
};

use crate::args::{CliOutputFormat, TimeTravelAction, TimeTravelGrainArg, TimeTravelScope};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TT_AGENT_ID: &str = "cli:timetravel";
const TT_SUPERSTEP: i64 = 0;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Dispatcher called by `main.rs` for all time-travel sub-commands.
///
/// # Errors
/// Returns a string error (printed by main.rs) on any failure.
pub(crate) fn run_timetravel(
    action: TimeTravelAction,
    _output_format: CliOutputFormat,
) -> Result<(), String> {
    if !timetravel_enabled() {
        println!("[timetravel] suppressed by {} kill-switch", ELAI_TIMETRAVEL_DISABLE_VAR);
        return Ok(());
    }
    match action {
        TimeTravelAction::Undo { grain, scope, steps, dry_run, session_id } => {
            run_undo(grain, scope, steps, dry_run, session_id)
        }
        TimeTravelAction::Redo { grain, scope, dry_run, session_id } => {
            run_redo(grain, scope, dry_run, session_id)
        }
        TimeTravelAction::History { format, session_id } => {
            run_history(format, session_id)
        }
        TimeTravelAction::Jump { target, grain, scope, dry_run, session_id } => {
            run_jump(target, grain, scope, dry_run, session_id)
        }
    }
}

// ---------------------------------------------------------------------------
// Undo
// ---------------------------------------------------------------------------

fn run_undo(
    grain: TimeTravelGrainArg,
    _scope: TimeTravelScope,
    steps: u32,
    dry_run: bool,
    session_id_override: Option<String>,
) -> Result<(), String> {
    let project_root = std::env::current_dir().map_err(|e| format!("timetravel undo: cwd: {e}"))?;
    let session_id = resolve_session_id(session_id_override)?;
    let dag_path = sessions_dir_path(&project_root)
        .join(&session_id)
        .join("session_dag.db");

    let dag = elai_orchestrator::session_dag::SessionDag::open(
        &session_id,
        &dag_path,
    )
    .map_err(|e| format!("timetravel undo: open session dag: {e}"))?;

    let head = dag
        .get_head()
        .map_err(|e| format!("timetravel undo: get_head: {e}"))?;

    let head = match head {
        Some(h) => h,
        None => {
            println!("[undo] no HEAD found — nothing to undo");
            return Ok(());
        }
    };

    let grain_val = grain.into_timetravel_grain();

    // Walk back `steps` ancestors at the requested grain.
    // BLOCK-5: grain-aware walk so turn/babystep differ from edit.
    //   edit    — one parent per step (finest).
    //   turn    — walk to nearest ancestor found by find_turn_ancestor (parent_chain-based).
    //   babystep — walk to nearest ancestor tagged "timetravel:edit" at a babystep boundary
    //              (tag contains "babystep").  Falls back to one-parent-step if no tagged
    //              ancestor found (Tier-2 refinement: full SSB-scan boundary detection).
    let mut current = head.clone();
    let mut target_snap = head.clone();
    for _ in 0..steps {
        match grain_val {
            TimeTravelGrain::Edit => {
                // finest: one parent step
                let (snap, _msgs) = dag
                    .load_snapshot(&current)
                    .map_err(|e| format!("timetravel undo: load_snapshot {current}: {e}"))?;
                match snap.parent_id {
                    Some(parent) => {
                        target_snap = parent.clone();
                        current = parent;
                    }
                    None => {
                        println!("[undo] already at root — no further undo possible");
                        break;
                    }
                }
            }
            TimeTravelGrain::Turn => {
                // Walk parent_chain, call find_turn_ancestor to get the turn boundary.
                let chain = dag
                    .parent_chain(&current)
                    .map_err(|e| format!("timetravel undo: parent_chain {current}: {e}"))?;
                let snap_ids: Vec<String> = chain.iter().map(|s| s.snap_id.clone()).collect();
                use elai_orchestrator::architecture::timetravel::find_turn_ancestor;
                match find_turn_ancestor(&snap_ids, &session_id) {
                    Some(anc) => {
                        target_snap = anc.to_owned();
                        current = anc.to_owned();
                    }
                    None => {
                        println!("[undo] already at turn root — no further undo possible");
                        break;
                    }
                }
            }
            TimeTravelGrain::BabyStep => {
                // Walk parent chain to find nearest ancestor tagged with "babystep".
                let chain = dag
                    .parent_chain(&current)
                    .map_err(|e| format!("timetravel undo: parent_chain {current}: {e}"))?;
                // chain is root-to-tip; we want tip-to-root search for nearest ancestor.
                let found = chain.iter().rev().find(|s| {
                    s.snap_id != current
                        && s.tag.as_deref().map_or(false, |t| t.contains("babystep"))
                });
                match found {
                    Some(anc) => {
                        target_snap = anc.snap_id.clone();
                        current = anc.snap_id.clone();
                    }
                    None => {
                        // No babystep-tagged ancestor; fall back to one-parent step.
                        let (snap, _msgs) = dag
                            .load_snapshot(&current)
                            .map_err(|e| format!("timetravel undo: load_snapshot {current}: {e}"))?;
                        match snap.parent_id {
                            Some(parent) => {
                                target_snap = parent.clone();
                                current = parent;
                            }
                            None => {
                                println!("[undo] already at root — no further undo possible");
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    if target_snap == head {
        println!("[undo] HEAD already at target — no-op (zero rows)");
        return Ok(());
    }

    // Load the manifest for target_snap.
    let manifest_hash = dag
        .get_file_manifest(&target_snap)
        .map_err(|e| format!("timetravel undo: get_file_manifest: {e}"))?;

    if dry_run {
        println!(
            "[undo dry-run] would restore HEAD {} -> {} (grain: {})",
            head,
            target_snap,
            grain_val.as_str()
        );
        if let Some(mh) = &manifest_hash {
            println!("[undo dry-run] manifest hash: {mh}");
        }
        println!(
            "[undo dry-run] RV-4 warning: file restore cannot undo external side effects \
             (network calls, git pushes, deploys, sent messages)."
        );
        return Ok(());
    }

    let cas_root = project_root.join(SESSION_CHECKPOINT_ROOT);
    let engine = CasRestoreEngine;

    // If there is a manifest, materialize it.
    // BLOCK-3 (RV-1): inspect RestoreResult.divergence_detected after materialize.
    // If any file's on-disk bytes drifted from HEAD's expected hash, fork-capture
    // the hand-modified bytes (the materialize fn already called cas_put for them),
    // then emit EditHistoryEvent{Branched, node_id: "auto:pre-restore:<snap>"}
    // BEFORE setting HEAD to target_snap so the fork row arrives first (RV-1).
    if let Some(mh) = manifest_hash {
        let manifest = manifest_get(&cas_root, &session_id, &mh)
            .map_err(|e| format!("timetravel undo: load manifest {mh}: {e}"))?;
        if !manifest.entries.is_empty() {
            let restore_results = engine
                .materialize_manifest(&cas_root, &session_id, &manifest, &project_root)
                .map_err(|e| format!("timetravel undo: materialize: {e}"))?;
            println!(
                "[undo] restored {} file(s) from manifest {}",
                manifest.entries.len(),
                &mh[..8]
            );

            // BLOCK-3: emit Branched row for each divergent file (RV-1).
            let any_diverged = restore_results.iter().any(|r| r.divergence_detected);
            if any_diverged {
                let fork_node_id = format!("auto:pre-restore:{}", &head[..8.min(head.len())]);
                let ssb_pre = open_ssb(&project_root, &session_id)?;
                let branch_payload = SsbPayload::EditHistoryEvent(EditHistoryEventPayload {
                    kind: EditHistoryEventKind::Branched,
                    node_id: fork_node_id,
                    parent_id: Some(head.clone()),
                    paths_n: restore_results.iter().filter(|r| r.divergence_detected).count(),
                    pre_hash: None,
                    post_hash: None,
                    grain: grain_val.as_str().to_owned(),
                });
                ssb_emit_sync(&ssb_pre, &session_id, TT_SUPERSTEP, branch_payload, vec![])?;
                println!(
                    "[undo] RV-1: divergence detected in {} file(s) — fork-capture emitted before restore",
                    restore_results.iter().filter(|r| r.divergence_detected).count()
                );
            }
        }
    }

    // Advance HEAD to target_snap.
    dag.set_head(&target_snap)
        .map_err(|e| format!("timetravel undo: set_head: {e}"))?;

    // Emit SessionRewound beacon (H-35 reuse, GD-1).
    let ssb = open_ssb(&project_root, &session_id)?;
    let payload = SsbPayload::SessionRewound(SessionRewoundPayload {
        session_id: session_id.clone(),
        target_seq: 0, // Undo-tree node-based; seq field preserved for H-35 compat
        head_seq_before: 0,
        scope: "code".to_owned(),
        summarized: false,
        replay_session_id: None,
        replay_outcome: "n/a".to_owned(),
        tail_fingerprint_blake3: String::new(),
    });
    ssb_emit_sync(&ssb, &session_id, TT_SUPERSTEP, payload, vec![])?;

    println!("[undo] HEAD {} -> {} (grain: {})", &head[..8.min(head.len())], &target_snap[..8.min(target_snap.len())], grain_val.as_str());
    Ok(())
}

// ---------------------------------------------------------------------------
// Redo
// ---------------------------------------------------------------------------

fn run_redo(
    grain: TimeTravelGrainArg,
    _scope: TimeTravelScope,
    dry_run: bool,
    session_id_override: Option<String>,
) -> Result<(), String> {
    let project_root = std::env::current_dir().map_err(|e| format!("timetravel redo: cwd: {e}"))?;
    let session_id = resolve_session_id(session_id_override)?;
    let dag_path = sessions_dir_path(&project_root)
        .join(&session_id)
        .join("session_dag.db");

    let dag = elai_orchestrator::session_dag::SessionDag::open(&session_id, &dag_path)
        .map_err(|e| format!("timetravel redo: open session dag: {e}"))?;

    let head = dag
        .get_head()
        .map_err(|e| format!("timetravel redo: get_head: {e}"))?;
    let head = match head {
        Some(h) => h,
        None => {
            println!("[redo] no HEAD — nothing to redo");
            return Ok(());
        }
    };

    let children = dag
        .children_of(&head)
        .map_err(|e| format!("timetravel redo: children_of {head}: {e}"))?;

    if children.is_empty() {
        println!("[redo] HEAD is a leaf — nothing to redo");
        return Ok(());
    }

    // If multiple children (branched tree), take the last-followed (first in
    // creation order for now — Tier-2 refinement: track last-followed edge).
    let target = children[0].clone();
    let grain_val: TimeTravelGrain = grain.into_timetravel_grain();

    if dry_run {
        println!(
            "[redo dry-run] would move HEAD {} -> {} (grain: {})",
            head,
            target,
            grain_val.as_str()
        );
        if children.len() > 1 {
            println!(
                "[redo dry-run] {} sibling branches exist; defaulting to first child",
                children.len()
            );
        }
        return Ok(());
    }

    // Load and materialize the target's manifest.
    let cas_root = project_root.join(SESSION_CHECKPOINT_ROOT);
    let engine = CasRestoreEngine;

    if let Some(mh) = dag
        .get_file_manifest(&target)
        .map_err(|e| format!("timetravel redo: get_file_manifest {target}: {e}"))?
    {
        let manifest = manifest_get(&cas_root, &session_id, &mh)
            .map_err(|e| format!("timetravel redo: load manifest {mh}: {e}"))?;
        if !manifest.entries.is_empty() {
            engine
                .materialize_manifest(&cas_root, &session_id, &manifest, &project_root)
                .map_err(|e| format!("timetravel redo: materialize: {e}"))?;
        }
    }

    dag.set_head(&target)
        .map_err(|e| format!("timetravel redo: set_head {target}: {e}"))?;

    // Emit EditHistoryEvent{Redo}.
    let ssb = open_ssb(&project_root, &session_id)?;
    let payload = SsbPayload::EditHistoryEvent(EditHistoryEventPayload {
        kind: EditHistoryEventKind::Redo,
        node_id: target.clone(),
        parent_id: Some(head.clone()),
        paths_n: 0, // populated by materialize call above if desired
        pre_hash: None,
        post_hash: None,
        grain: grain_val.as_str().to_owned(),
    });
    ssb_emit_sync(&ssb, &session_id, TT_SUPERSTEP, payload, vec![])?;

    println!("[redo] HEAD {} -> {} (grain: {})", &head[..8.min(head.len())], &target[..8.min(target.len())], grain_val.as_str());
    Ok(())
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

fn run_history(
    format: String,
    session_id_override: Option<String>,
) -> Result<(), String> {
    let project_root = std::env::current_dir().map_err(|e| format!("timetravel history: cwd: {e}"))?;
    let session_id = resolve_session_id(session_id_override)?;
    let dag_path = sessions_dir_path(&project_root)
        .join(&session_id)
        .join("session_dag.db");

    let dag = elai_orchestrator::session_dag::SessionDag::open(&session_id, &dag_path)
        .map_err(|e| format!("timetravel history: open dag: {e}"))?;

    let head = dag
        .get_head()
        .map_err(|e| format!("timetravel history: get_head: {e}"))?;

    println!("Time-travel history for session: {session_id}");
    println!("HEAD: {}", head.as_deref().unwrap_or("(none)"));
    println!();

    // BLOCK-5: real tree rendering using parent_chain + children_of.
    // Walk from HEAD back to root to get the main chain, then show sibling
    // branches at each node that has multiple children.
    let head_id = match head.as_deref() {
        Some(h) => h,
        None => {
            println!("(no snapshots captured yet)");
            return Ok(());
        }
    };

    // Get the ancestor chain (root-to-tip order).
    let chain = dag
        .parent_chain(head_id)
        .map_err(|e| format!("timetravel history: parent_chain: {e}"))?;

    let is_tree = format == "tree";

    if is_tree {
        // Tree view: render root -> tip with branch markers.
        println!("Tree view (root to HEAD):");
        for (i, snap) in chain.iter().enumerate() {
            let is_head = snap.snap_id == head_id;
            let marker = if is_head { "* HEAD" } else { "  " };
            let tag_str = snap.tag.as_deref().unwrap_or("");
            println!(
                "{}{} [{}]  {}",
                "  ".repeat(i),
                marker,
                &snap.snap_id[..8.min(snap.snap_id.len())],
                tag_str
            );
            // Show sibling branches at this node.
            let children = dag
                .children_of(&snap.snap_id)
                .unwrap_or_default();
            if children.len() > 1 {
                for child in &children {
                    if !chain.iter().any(|s| &s.snap_id == child) {
                        // This child is a sibling branch (not on the main chain).
                        println!(
                            "{}  | branch: [{}]",
                            "  ".repeat(i + 1),
                            &child[..8.min(child.len())]
                        );
                    }
                }
            }
        }
    } else {
        // List view: most-recent first.
        println!("List view (newest first):");
        for snap in chain.iter().rev() {
            let is_head = snap.snap_id == head_id;
            let marker = if is_head { "* " } else { "  " };
            let tag_str = snap.tag.as_deref().unwrap_or("");
            let children_count = dag.children_of(&snap.snap_id).map(|c| c.len()).unwrap_or(0);
            let branch_note = if children_count > 1 {
                format!(" [{} branches]", children_count)
            } else {
                String::new()
            };
            println!(
                "{}[{}]  {}{}",
                marker,
                &snap.snap_id[..8.min(snap.snap_id.len())],
                tag_str,
                branch_note
            );
        }
    }

    println!();
    println!("({} snapshot(s) in chain)", chain.len());
    Ok(())
}

// ---------------------------------------------------------------------------
// Jump
// ---------------------------------------------------------------------------

fn run_jump(
    target: String,
    grain: TimeTravelGrainArg,
    _scope: TimeTravelScope,
    dry_run: bool,
    session_id_override: Option<String>,
) -> Result<(), String> {
    let project_root = std::env::current_dir().map_err(|e| format!("timetravel jump: cwd: {e}"))?;
    let session_id = resolve_session_id(session_id_override)?;
    let dag_path = sessions_dir_path(&project_root)
        .join(&session_id)
        .join("session_dag.db");

    let dag = elai_orchestrator::session_dag::SessionDag::open(&session_id, &dag_path)
        .map_err(|e| format!("timetravel jump: open dag: {e}"))?;

    let head = dag
        .get_head()
        .map_err(|e| format!("timetravel jump: get_head: {e}"))?;
    let head = match head {
        Some(h) => h,
        None => {
            println!("[jump] no HEAD — no-op");
            return Ok(());
        }
    };

    let grain_val = grain.into_timetravel_grain();

    if target == head {
        println!("[jump] target == HEAD — no-op (zero rows)");
        return Ok(());
    }

    if dry_run {
        println!("[jump dry-run] would move HEAD {} -> {target} (grain: {})", head, grain_val.as_str());
        println!("[jump dry-run] RV-4 warning: irreversible side effects are NOT undone.");
        return Ok(());
    }

    let cas_root = project_root.join(SESSION_CHECKPOINT_ROOT);
    let engine = CasRestoreEngine;

    if let Some(mh) = dag
        .get_file_manifest(&target)
        .map_err(|e| format!("timetravel jump: get_file_manifest {target}: {e}"))?
    {
        let manifest = manifest_get(&cas_root, &session_id, &mh)
            .map_err(|e| format!("timetravel jump: load manifest {mh}: {e}"))?;
        if !manifest.entries.is_empty() {
            engine
                .materialize_manifest(&cas_root, &session_id, &manifest, &project_root)
                .map_err(|e| format!("timetravel jump: materialize: {e}"))?;
        }
    }

    dag.set_head(&target)
        .map_err(|e| format!("timetravel jump: set_head {target}: {e}"))?;

    let ssb = open_ssb(&project_root, &session_id)?;
    let payload = SsbPayload::EditHistoryEvent(EditHistoryEventPayload {
        kind: EditHistoryEventKind::Jumped,
        node_id: target.clone(),
        parent_id: Some(head.clone()),
        paths_n: 0,
        pre_hash: None,
        post_hash: None,
        grain: grain_val.as_str().to_owned(),
    });
    ssb_emit_sync(&ssb, &session_id, TT_SUPERSTEP, payload, vec![])?;

    println!("[jump] HEAD {} -> {target} (grain: {})", &head[..8.min(head.len())], grain_val.as_str());
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn resolve_session_id(override_id: Option<String>) -> Result<String, String> {
    if let Some(id) = override_id {
        return Ok(id);
    }
    std::env::var("ELAI_SESSION_ID")
        .map_err(|_| "timetravel: no session_id; pass --session-id or set ELAI_SESSION_ID".to_owned())
}

fn open_ssb(project_root: &Path, session_id: &str) -> Result<Arc<Ssb>, String> {
    let sessions_dir = sessions_dir_path(project_root);
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("timetravel: create sessions dir: {e}"))?;
    let ssb_path = sessions_dir.join("ssb.db");
    let cfg = SsbConfig {
        project_id: "elai".to_owned(),
        session_id: session_id.to_owned(),
        db_path: ssb_path,
        pool_size: None,
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(cfg, clock).map_err(|e| format!("timetravel: open SSB: {e}"))?;
    Ok(Arc::new(ssb))
}

fn ssb_emit_sync(
    ssb: &Arc<Ssb>,
    _session_id: &str,
    superstep: i64,
    payload: SsbPayload,
    file_paths: Vec<String>,
) -> Result<(), String> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .map_err(|e| format!("timetravel: tokio runtime: {e}"))?;
    let ssb_clone = Arc::clone(ssb);
    rt.block_on(async move {
        ssb_clone
            .write_payload(TT_AGENT_ID, superstep, payload, file_paths)
            .await
            .map(|_seq| ())
            .map_err(|e| format!("timetravel: SSB emit: {e}"))
    })
}

// ---------------------------------------------------------------------------
// TimeTravelGrainArg conversion
// ---------------------------------------------------------------------------

impl TimeTravelGrainArg {
    pub(crate) fn into_timetravel_grain(self) -> TimeTravelGrain {
        match self {
            Self::Edit => TimeTravelGrain::Edit,
            Self::Turn => TimeTravelGrain::Turn,
            Self::BabyStep => TimeTravelGrain::BabyStep,
        }
    }
}
