use std::borrow::ToOwned;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use elai_orchestrator::{ContentBlock, ConversationMessage, MessageRole, Session};
use elai_core::{ClockProvider, EntropySource, OsEntropy};
use elai_core::paths::{ELAI_DATA_DIR_NAME, SESSIONS_DIR_NAME};

use crate::{
    create_managed_session_handle, build_runtime, plan_phase_slot_tokens,
    LiveCli, SessionHandle, ManagedSessionSummary,
    PRIMARY_SESSION_EXTENSION, LEGACY_SESSION_EXTENSION,
    SESSION_REFERENCE_ALIASES, LATEST_SESSION_REFERENCE,
};

pub(crate) fn sessions_dir() -> Result<PathBuf, anyhow::Error> {
    // B-1: canonical sessions dir.
    let cwd = env::current_dir()?;
    let path = cwd.join(ELAI_DATA_DIR_NAME).join(SESSIONS_DIR_NAME);
    fs::create_dir_all(&path)?;
    warn_legacy_sessions_if_stranded(&cwd, &path);
    Ok(path)
}

/// B-1 WARN-3: emit a one-time deprecation warning when legacy `.elai/sessions/`
/// exists alongside the canonical `.elai_cc/sessions/` directory and still contains
/// session files. Sessions written before the B-1 path unification are invisible to
/// `/resume` and `/session list` until migrated manually.
/// Analogous to the stale-path detection in indexing.rs:149-162.
fn warn_legacy_sessions_if_stranded(project_root: &Path, canonical_sessions: &Path) {
    let legacy_sessions = project_root.join(".elai").join(SESSIONS_DIR_NAME);
    if !legacy_sessions.exists() || !canonical_sessions.exists() {
        return;
    }
    let count = fs::read_dir(&legacy_sessions)
        .map(|rd| rd.flatten().count())
        .unwrap_or(0);
    if count > 0 {
        tracing::warn!(
            legacy = %legacy_sessions.display(),
            canonical = %canonical_sessions.display(),
            stranded_files = count,
            "B-1: {} session file(s) found in legacy `.elai/sessions/`; \
             ELAI now reads sessions from `.elai_cc/sessions/` — \
             /resume and /session list cannot see these sessions. \
             Move them to `.elai_cc/sessions/` to restore visibility.",
            count,
        );
    }
}

/// B-9 cycle 2 (BLOCK-1 fix): rehydrate a switched-into session from the
/// per-session SessionDag SQLite database. Falls back to the legacy JSONL
/// renderer only when no DAG handoff snapshot exists for the target. The
/// JSONL fallback is gated on `export_jsonl_enabled` so a session created
/// in DAG-only mode does not silently revive a stale JSONL file.
pub(crate) fn rehydrate_session_for_switch(
    handle: &SessionHandle,
    export_jsonl_enabled: bool,
) -> Result<Session, anyhow::Error> {
    let dag_path = handle
        .path
        .parent()
        .map(|p| p.join("dag.db"))
        .unwrap_or_else(|| PathBuf::from("dag.db"));

    if dag_path.exists() {
        if let Ok(dag) = elai_orchestrator::SessionDag::open(&handle.id, &dag_path) {
            if let Ok(Some((_snapshot, messages))) = dag.load_handoff() {
                let mut session = Session::new();
                for tm in &messages {
                    let role = match tm.role {
                        elai_orchestrator::DagMessageRole::User => MessageRole::User,
                        elai_orchestrator::DagMessageRole::Assistant => MessageRole::Assistant,
                        elai_orchestrator::DagMessageRole::System => MessageRole::System,
                        elai_orchestrator::DagMessageRole::Tool => MessageRole::Tool,
                    };
                    let blocks = tm.blocks.clone().unwrap_or_else(|| {
                        tracing::warn!(
                            seq = tm.seq,
                            "rehydrate_session_for_switch: legacy DAG row without raw_blocks; \
                             ToolUse/ToolResult fidelity is lost — falling back to text",
                        );
                        vec![ContentBlock::Text {
                            text: String::from_utf8_lossy(&tm.content).into_owned(),
                        }]
                    });
                    let msg = ConversationMessage {
                        role,
                        blocks,
                        usage: None,
                    };
                    if let Err(e) = session.push_message(msg) {
                        tracing::warn!(error = %e, "rehydrate_session_for_switch: push_message failed");
                    }
                }
                return Ok(session);
            }
        }
    }

    // B-9 cycle 3 (WARN fix): the previous `||` condition fell through to
    // the JSONL renderer whenever a stale .jsonl was present on disk —
    // ignoring the flag and silently reviving legacy state. Require BOTH
    // the opt-in flag AND the file. Without `--export-jsonl`, the session
    // is DAG-only and not recoverable through the legacy renderer.
    if export_jsonl_enabled && handle.path.exists() {
        return Ok(Session::load_from_path(&handle.path)?);
    }

    Err(anyhow::anyhow!(
        "no SessionDag snapshot for {} and JSONL fallback is disabled (--export-jsonl)",
        handle.id
    ))
}

pub(crate) fn resolve_session_reference(reference: &str) -> Result<SessionHandle, anyhow::Error> {
    if SESSION_REFERENCE_ALIASES
        .iter()
        .any(|alias| reference.eq_ignore_ascii_case(alias))
    {
        let latest = latest_managed_session()?;
        return Ok(SessionHandle {
            id: latest.id,
            path: latest.path,
        });
    }

    let direct = PathBuf::from(reference);
    let looks_like_path = direct.extension().is_some() || direct.components().count() > 1;
    let path = if direct.exists() {
        direct
    } else if looks_like_path {
        return Err(anyhow::anyhow!(format_missing_session_reference(reference)));
    } else {
        resolve_managed_session_path(reference)?
    };
    let id = path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|name| {
            name.strip_suffix(&format!(".{PRIMARY_SESSION_EXTENSION}"))
                .or_else(|| name.strip_suffix(&format!(".{LEGACY_SESSION_EXTENSION}")))
        })
        .unwrap_or(reference)
        .to_string();
    Ok(SessionHandle { id, path })
}

pub(crate) fn resolve_managed_session_path(session_id: &str) -> Result<PathBuf, anyhow::Error> {
    let directory = sessions_dir()?;
    for extension in [PRIMARY_SESSION_EXTENSION, LEGACY_SESSION_EXTENSION] {
        let path = directory.join(format!("{session_id}.{extension}"));
        if path.exists() {
            return Ok(path);
        }
    }
    Err(anyhow::anyhow!(format_missing_session_reference(session_id)))
}

pub(crate) fn is_managed_session_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|extension| {
            extension == PRIMARY_SESSION_EXTENSION || extension == LEGACY_SESSION_EXTENSION
        })
}

pub(crate) fn list_managed_sessions() -> Result<Vec<ManagedSessionSummary>, anyhow::Error> {
    let mut sessions = Vec::new();
    for entry in fs::read_dir(sessions_dir()?)? {
        let entry = entry?;
        let path = entry.path();
        if !is_managed_session_file(&path) {
            continue;
        }
        let metadata = entry.metadata()?;
        let modified_epoch_millis = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis())
            .unwrap_or_default();
        let (id, message_count, parent_session_id, branch_name) =
            match Session::load_from_path(&path) {
                Ok(session) => {
                    let parent_session_id = session
                        .fork
                        .as_ref()
                        .map(|fork| fork.parent_session_id.clone());
                    let branch_name = session
                        .fork
                        .as_ref()
                        .and_then(|fork| fork.branch_name.clone());
                    (
                        session.session_id,
                        session.messages.len(),
                        parent_session_id,
                        branch_name,
                    )
                }
                Err(_) => (
                    path.file_stem()
                        .and_then(|value| value.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    0,
                    None,
                    None,
                ),
            };
        sessions.push(ManagedSessionSummary {
            id,
            path,
            modified_epoch_millis,
            message_count,
            parent_session_id,
            branch_name,
        });
    }
    sessions.sort_by(|left, right| {
        right
            .modified_epoch_millis
            .cmp(&left.modified_epoch_millis)
            .then_with(|| right.id.cmp(&left.id))
    });
    Ok(sessions)
}

pub(crate) fn latest_managed_session() -> Result<ManagedSessionSummary, anyhow::Error> {
    list_managed_sessions()?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!(format_no_managed_sessions()))
}

pub(crate) fn format_missing_session_reference(reference: &str) -> String {
    format!(
        "session not found: {reference}\nHint: managed sessions live in .elai_cc/sessions/. Try `{LATEST_SESSION_REFERENCE}` for the most recent session or `/session list` in the REPL."
    )
}

pub(crate) fn format_no_managed_sessions() -> String {
    format!(
        "no managed sessions found in .elai_cc/sessions/\nStart `elai` to create a session, then rerun with `--resume {LATEST_SESSION_REFERENCE}`."
    )
}

pub(crate) fn render_session_list(active_session_id: &str) -> Result<String, anyhow::Error> {
    let sessions = list_managed_sessions()?;
    let mut lines = vec![
        "Sessions".to_string(),
        format!("  Directory         {}", sessions_dir()?.display()),
    ];
    if sessions.is_empty() {
        lines.push("  No managed sessions saved yet.".to_string());
        return Ok(lines.join("\n"));
    }
    for session in sessions {
        let marker = if session.id == active_session_id {
            "● current"
        } else {
            "○ saved"
        };
        let lineage = match (
            session.branch_name.as_deref(),
            session.parent_session_id.as_deref(),
        ) {
            (Some(branch_name), Some(parent_session_id)) => {
                format!(" branch={branch_name} from={parent_session_id}")
            }
            (None, Some(parent_session_id)) => format!(" from={parent_session_id}"),
            (Some(branch_name), None) => format!(" branch={branch_name}"),
            (None, None) => String::new(),
        };
        lines.push(format!(
            "  {id:<20} {marker:<10} msgs={msgs:<4} modified={modified}{lineage} path={path}",
            id = session.id,
            msgs = session.message_count,
            modified = format_session_modified_age(session.modified_epoch_millis),
            lineage = lineage,
            path = session.path.display(),
        ));
    }
    Ok(lines.join("\n"))
}

pub(crate) fn format_session_modified_age(modified_epoch_millis: u128) -> String {
    let now = elai_core::OsClock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map_or(modified_epoch_millis, |duration| duration.as_millis());
    let delta_seconds = now
        .saturating_sub(modified_epoch_millis)
        .checked_div(1_000)
        .unwrap_or_default();
    match delta_seconds {
        0..=4 => "just-now".to_string(),
        5..=59 => format!("{delta_seconds}s-ago"),
        60..=3_599 => format!("{}m-ago", delta_seconds / 60),
        3_600..=86_399 => format!("{}h-ago", delta_seconds / 3_600),
        _ => format!("{}d-ago", delta_seconds / 86_400),
    }
}

pub(crate) fn write_session_clear_backup(
    session: &Session,
    session_path: &Path,
) -> Result<PathBuf, anyhow::Error> {
    let backup_path = session_clear_backup_path(session_path);
    session.save_to_path(&backup_path)?;
    Ok(backup_path)
}

pub(crate) fn session_clear_backup_path(session_path: &Path) -> PathBuf {
    let timestamp = elai_core::OsClock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map_or(0, |duration| duration.as_millis());
    let file_name = session_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("session.jsonl");
    session_path.with_file_name(format!("{file_name}.before-clear-{timestamp}.bak"))
}

/// B-9 T3: rehydrate the most recent SessionDag handoff snapshot. This is
/// the canonical /continue path — replaces the legacy JSONL
/// `Session::load_from_path` for handoff resume. The CLI keeps /resume for
/// the legacy pre-DAG path (explicit, opt-in); /continue is the path the
/// new dashboard's IPC consumes.
pub(crate) fn continue_from_handoff(
    cli: &mut LiveCli,
    snap_id: Option<&str>,
) -> Result<bool, anyhow::Error> {
    let Some(dag) = cli.session_dag.as_ref() else {
        eprintln!(
            "/continue requires a SessionDag for the active session. \
             Use /resume <session-path> to load a legacy JSONL session, \
             then /continue to follow up at handoff."
        );
        return Ok(false);
    };

    // Branch-tip-first via SessionDag::load_handoff (DECISION-C: backend
    // only, no JSONL touch). Optional snap_id selects an explicit snapshot.
    let loaded = if let Some(id) = snap_id {
        dag.load_snapshot(id)
            .map(Some)
            .map_err(|e| anyhow::anyhow!("load_snapshot({id}) failed: {e}"))?
    } else {
        dag.load_handoff()
            .map_err(|e| anyhow::anyhow!("load_handoff failed: {e}"))?
    };
    let Some((snapshot, messages)) = loaded else {
        println!("/continue: no handoff snapshot found for this session.");
        return Ok(false);
    };

    // Reconstruct the in-memory session from the rehydrated message stream.
    //
    // B-9 cycle 2 (BLOCK-2 fix): prefer the structured `blocks` field
    // when the DAG row carries one. Flattening to `ContentBlock::Text`
    // alone caused every `ToolUse` / `ToolResult` block to vanish on
    // /continue, which made the Anthropic API reject the rehydrated
    // conversation with `400 invalid_request_error` because tool_use
    // blocks must each be answered by a tool_result.
    //
    // `blocks: None` means either a legacy DAG row written before this
    // migration, or a row that trim Pass 1 stubbed away. Both fall
    // back to the text-only path with a tracing warning so operators
    // can correlate /continue failures with stale data.
    let mut session = Session::new();
    for tm in &messages {
        let role = match tm.role {
            elai_orchestrator::DagMessageRole::User => MessageRole::User,
            elai_orchestrator::DagMessageRole::Assistant => MessageRole::Assistant,
            elai_orchestrator::DagMessageRole::System => MessageRole::System,
            elai_orchestrator::DagMessageRole::Tool => MessageRole::Tool,
        };
        let blocks = tm.blocks.clone().unwrap_or_else(|| {
            tracing::warn!(
                seq = tm.seq,
                is_stubbed = tm.is_stubbed,
                "continue_from_handoff: legacy DAG row without raw_blocks; \
                 ToolUse/ToolResult fidelity is lost — falling back to text. \
                 The Anthropic API may reject this conversation if tool-use \
                 pairs were present in the original session.",
            );
            vec![ContentBlock::Text {
                text: String::from_utf8_lossy(&tm.content).into_owned(),
            }]
        });
        let msg = ConversationMessage {
            role,
            blocks,
            usage: None,
        };
        // push_message returns Result; on persistence error we still want
        // the in-memory session to reflect the rehydrated message.
        if let Err(e) = session.push_message(msg) {
            eprintln!("warn: continue_from_handoff push_message failed: {e}");
        }
    }
    let message_count = session.messages.len();
    let (runtime, _ssb) = build_runtime(
        session,
        &cli.session.id,
        cli.model.clone(),
        cli.system_prompt.clone(),
        true,
        true,
        cli.allowed_tools.clone(),
        cli.permission_mode,
        None,
        cli.backend_config.as_ref(),
        Some(cli.fsm.clone()),
        plan_phase_slot_tokens(cli.plan.as_ref()),
    )?;
    cli.replace_runtime(runtime)?;
    cli.last_snap_id = Some(snapshot.snap_id.clone());

    println!(
        "Continue\n  Snapshot         {}\n  FSM state        {}\n  Messages         {}",
        snapshot.snap_id, snapshot.fsm_state, message_count,
    );
    Ok(true)
}

pub(crate) fn handle_session_command(
    cli: &mut LiveCli,
    action: Option<&str>,
    target: Option<&str>,
) -> Result<bool, anyhow::Error> {
    match action {
        None | Some("list") => {
            println!("{}", render_session_list(&cli.session.id)?);
            Ok(false)
        }
        Some("switch") => {
            let Some(target) = target else {
                println!("Usage: /session switch <session-id>");
                return Ok(false);
            };
            let handle = resolve_session_reference(target)?;
            // B-9 cycle 2 (BLOCK-1 fix): try the SessionDag first —
            // every REPL turn writes a snapshot per L4134/L4278, so
            // the DAG holds the authoritative conversation state. Fall
            // back to JSONL only when no DAG snapshot exists for the
            // target session (legacy session) AND `--export-jsonl`
            // was set (otherwise there is nothing to load).
            let session = rehydrate_session_for_switch(
                &handle,
                cli.export_jsonl_enabled,
            )?;
            let message_count = session.messages.len();
            let session_id = session.session_id.clone();
            let (runtime, _ssb) = build_runtime(
                session,
                &handle.id,
                cli.model.clone(),
                cli.system_prompt.clone(),
                true,
                true,
                cli.allowed_tools.clone(),
                cli.permission_mode,
                None,
                cli.backend_config.as_ref(),
                Some(cli.fsm.clone()),
                plan_phase_slot_tokens(cli.plan.as_ref()),
            )?;
            cli.replace_runtime(runtime)?;
            cli.session = SessionHandle {
                id: session_id,
                path: handle.path,
            };
            println!(
                "Session switched\n  Active session   {}\n  File             {}\n  Messages         {}",
                cli.session.id,
                cli.session.path.display(),
                message_count,
            );
            Ok(true)
        }
        Some("fork") => {
            let branch_name_str = target.unwrap_or("fork");

            // Try DAG-based branching first
            if let Some(dag) = &mut cli.session_dag {
                if let Some(snap_id) = &cli.last_snap_id {
                    match dag.create_branch(
                        branch_name_str,
                        snap_id,
                        &mut OsEntropy,
                        &elai_core::OsClock,
                    ) {
                        Ok(branch_ref) => {
                            println!(
                                "Session branched (DAG)\n  Branch           {}\n  Tip snapshot     {}\n  Parent session   {}",
                                branch_ref.name,
                                branch_ref.tip_snap_id,
                                cli.session.id,
                            );
                            return Ok(false);
                        }
                        Err(elai_orchestrator::SessionDagError::BranchLimitExceeded { .. }) => {
                            eprintln!("Max 3 active branches — merge or delete one first.");
                            return Ok(false);
                        }
                        Err(e) => {
                            eprintln!("warn: DAG branch failed, falling back to session fork: {e}");
                        }
                    }
                }
            }

            // Fallback: session-based fork
            let forked = cli.runtime.fork_session(target.map(ToOwned::to_owned));
            let parent_session_id = cli.session.id.clone();
            let handle = create_managed_session_handle(&forked.session_id)?;
            let branch_name = forked
                .fork
                .as_ref()
                .and_then(|fork| fork.branch_name.clone());
            let forked = forked.with_persistence_path(handle.path.clone());
            let message_count = forked.messages.len();
            // B-9 cycle 3 (BLOCK fix): gate the legacy JSONL writer on
            // `--export-jsonl`. The DAG `create_branch` path above is
            // authoritative for forks; this fallback only runs when DAG
            // forking returns an unexpected `Err`. Mirrors the gate in
            // `persist_session` (L4964).
            if cli.export_jsonl_enabled {
                forked.save_to_path(&handle.path)?;
            }
            let (runtime, _ssb) = build_runtime(
                forked,
                &handle.id,
                cli.model.clone(),
                cli.system_prompt.clone(),
                true,
                true,
                cli.allowed_tools.clone(),
                cli.permission_mode,
                None,
                cli.backend_config.as_ref(),
                Some(cli.fsm.clone()),
                plan_phase_slot_tokens(cli.plan.as_ref()),
            )?;
            cli.replace_runtime(runtime)?;
            cli.session = handle;
            println!(
                "Session forked\n  Parent session   {}\n  Active session   {}\n  Branch           {}\n  File             {}\n  Messages         {}",
                parent_session_id,
                cli.session.id,
                branch_name.as_deref().unwrap_or("(unnamed)"),
                cli.session.path.display(),
                message_count,
            );
            Ok(true)
        }
        Some(other) => {
            println!(
                "Unknown /session action '{other}'. Use /session list, /session switch <session-id>, or /session fork [branch-name]."
            );
            Ok(false)
        }
    }
}
