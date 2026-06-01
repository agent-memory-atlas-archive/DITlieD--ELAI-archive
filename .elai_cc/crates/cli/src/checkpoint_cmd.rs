//! E-25 Tranche C (C.4) -- `elai checkpoint <list|rewind>` dispatcher.
//!
//! Surfaces the
//! `elai_orchestrator::architecture::edit_checkpoints` storage layer:
//!   * `elai checkpoint list [--session-id ID] [--json]`
//!   * `elai checkpoint rewind --steps N [--session-id ID] [--dry-run]`
//!
//! Wire-dark default OFF for SSB emit (gated by
//! `ELAI_E25_EDIT_CHECKPOINTS_ENABLE=1`); the CLI inspection paths work
//! regardless so the operator can audit existing snapshots without
//! flipping the gate.

use std::path::PathBuf;

use elai_orchestrator::architecture::edit_checkpoints::{
    edit_checkpoints_enabled, list_session_checkpoints, rewind_to, EditCheckpoint,
    EditCheckpointError, SESSION_CHECKPOINT_ROOT,
};

fn checkpoint_root() -> PathBuf {
    // Resolve relative to the current working directory. The CLI
    // expects to be invoked from the workspace root via `elai.bat`
    // (CLAUDE.md projectPATHs).
    PathBuf::from(SESSION_CHECKPOINT_ROOT)
}

fn resolve_session_id(supplied: Option<String>) -> Result<String, anyhow::Error> {
    if let Some(s) = supplied {
        if s.is_empty() {
            return Err(anyhow::anyhow!("--session-id must not be empty"));
        }
        return Ok(s);
    }
    if let Ok(env_id) = std::env::var("ELAI_SESSION_ID") {
        if !env_id.is_empty() {
            return Ok(env_id);
        }
    }
    Err(anyhow::anyhow!(
        "no session id supplied (--session-id ID or ELAI_SESSION_ID env var)"
    ))
}

fn render_list(snapshots: &[EditCheckpoint], session_id: &str, json: bool) {
    if json {
        let arr: Vec<serde_json::Value> = snapshots
            .iter()
            .map(|cp| {
                serde_json::json!({
                    "seq": cp.seq,
                    "blake3_prefix": cp.blake3_prefix,
                    "path": cp.path.to_string_lossy(),
                    "snapshot_path": cp.snapshot_path.to_string_lossy(),
                    "size_bytes": cp.size_bytes,
                })
            })
            .collect();
        let payload = serde_json::json!({
            "session_id": session_id,
            "snapshots": arr,
            "count": snapshots.len(),
        });
        println!("{}", payload);
    } else {
        println!("checkpoint list");
        println!("  session_id = {session_id}");
        println!("  count      = {}", snapshots.len());
        for cp in snapshots {
            println!(
                "  seq={:06} blake3={} path={} size={}",
                cp.seq,
                cp.blake3_prefix,
                cp.path.to_string_lossy(),
                cp.size_bytes
            );
        }
        if !edit_checkpoints_enabled() {
            println!(
                "  ssb_emit   = skipped (ELAI_E25_EDIT_CHECKPOINTS_ENABLE unset; wire-dark default)"
            );
        }
    }
}

/// Synchronous entrypoint mirroring `lora_gate_check_cmd::run`.
pub fn run(
    sub: crate::args::CheckpointSub,
) -> Result<(), anyhow::Error> {
    let root = checkpoint_root();
    match sub {
        crate::args::CheckpointSub::List { session_id, json } => {
            let sid = resolve_session_id(session_id)?;
            let snapshots = list_session_checkpoints(&root, &sid)
                .map_err(map_err)?;
            render_list(&snapshots, &sid, json);
        }
        crate::args::CheckpointSub::Rewind {
            steps,
            session_id,
            dry_run,
        } => {
            let sid = resolve_session_id(session_id)?;
            let snapshots = list_session_checkpoints(&root, &sid).map_err(map_err)?;
            if snapshots.is_empty() {
                println!(
                    "checkpoint rewind: no snapshots for session {sid}; nothing to do"
                );
                return Ok(());
            }
            // Determine target seq: highest_seq - (steps - 1), clamped to 0.
            let max_seq = snapshots.iter().map(|c| c.seq).max().unwrap_or(0);
            let steps_minus_one = u64::from(steps.saturating_sub(1));
            let target = max_seq.saturating_sub(steps_minus_one);
            if dry_run {
                let restore_count = snapshots.iter().filter(|c| c.seq >= target).count();
                println!(
                    "checkpoint rewind --dry-run: would restore {restore_count} snapshot(s) (target_seq={target:06}, max_seq={max_seq:06}) for session {sid}"
                );
                return Ok(());
            }
            let outcome = rewind_to(&root, &sid, target).map_err(map_err)?;
            println!(
                "checkpoint rewind: restored {} snapshot(s) to target_seq={:06} for session {sid}",
                outcome.restored.len(),
                outcome.target_seq
            );
            for cp in &outcome.restored {
                println!(
                    "  restored seq={:06} path={}",
                    cp.seq,
                    cp.path.to_string_lossy()
                );
            }
        }
    }
    Ok(())
}

fn map_err(e: EditCheckpointError) -> anyhow::Error {
    anyhow::anyhow!("edit_checkpoints error: {e}")
}
