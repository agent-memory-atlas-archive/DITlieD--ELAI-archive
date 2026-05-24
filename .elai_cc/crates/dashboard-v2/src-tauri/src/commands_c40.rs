//! C-40: GitHub PR adapter Tauri command surface (read + mutation).
//!
//! cross-lang-pair: dashboard/tauri/api
//!
//! Five Tauri commands feed `GithubWindow.svelte` + `githubStore.ts`:
//!
//! - [`github_pr_open`]            — mutation. Creates a PR via gh.
//! - [`github_pr_list`]            — read. Lists PRs for a repo.
//! - [`github_pr_status`]          — read + SSB-emit. Status for one PR.
//! - [`github_pr_close`]           — mutation. Closes a PR (no merge).
//! - [`github_pr_get_active_url`]  — read-only. Returns the most recent
//!   PR URL surfaced via the active session (looks up the latest
//!   `github_pr_opened` SSB row for this project).
//!
//! Implementation note: the gh-CLI risk-class for all four shell-out
//! commands is already `RiskLevel::SharedState` per
//! `tools::action_classifier::BASH_SHARED_STATE_PATTERNS` (lines
//! 128-141). The adapter does NOT bypass that gate — every gh call
//! goes through `GhCliAdapter::run_gh` which is the canonical entry
//! point.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

use elai_orchestrator::github_pr::{
    GhCliAdapter, GithubPrAdapter, PrOpenRequest, PrState, PrStatus, PrSummary,
};
use elai_orchestrator::SsbActionType;

use crate::state::AppState;

/// Tauri serializes `Result<T, E>` as either `T` on Ok or rejection on
/// Err. Mirrors `commands.rs::CmdResult`.
type CmdResult<T> = Result<T, String>;

/// Construct a `GhCliAdapter` from the live AppState writer. Returns
/// an error string when the SSB writer is not yet attached (fresh
/// install / no project root selected).
fn build_adapter(state: &AppState) -> CmdResult<GhCliAdapter> {
    let writer = state
        .writer
        .clone()
        .ok_or_else(|| "ssb writer unavailable; open a project first".to_string())?;
    Ok(GhCliAdapter::with_agent_id(
        Arc::clone(&writer),
        "dashboard:github_pr",
    ))
}

/// Wire-format DTO for the `github_pr_open` command's incoming payload.
/// Matches the camelCase shape Tauri's JS bridge sends.
#[derive(Debug, Clone, Deserialize)]
pub struct GithubPrOpenArgs {
    #[serde(default)]
    pub repo: Option<String>,
    pub title: String,
    #[serde(default)]
    pub body: String,
    pub head: String,
    #[serde(default)]
    pub base: Option<String>,
    #[serde(default)]
    pub draft: bool,
}

/// Wire-format DTO for `github_pr_get_active_url` so the JS side
/// always sees a typed shape (even when no PR exists yet).
#[derive(Debug, Clone, Serialize)]
pub struct ActivePrUrlDto {
    /// Most recent PR URL surfaced by a `github_pr_opened` SSB row, or
    /// `None` when no row exists for the active project.
    pub url: Option<String>,
    /// PR number associated with the `url` field. Pair-wise with `url`.
    pub pr_number: Option<u32>,
    /// Timestamp (ms since UNIX epoch) of the row, when present.
    pub ts_ms: Option<i64>,
}

/// `github_pr_open` — create a new PR via `gh pr create`. Mutation.
///
/// Returns the freshly created PR's summary (number, URL, title,
/// state). Errors include `gh CLI not found`, `gh not authenticated`,
/// `invalid request` (empty title/head), and any non-zero gh exit.
#[tauri::command]
pub async fn github_pr_open(
    state: State<'_, AppState>,
    args: GithubPrOpenArgs,
) -> CmdResult<Value> {
    let adapter = build_adapter(&state)?;
    let req = PrOpenRequest {
        repo: args.repo,
        title: args.title,
        body: args.body,
        head: args.head,
        base: args.base,
        draft: args.draft,
    };
    let summary = adapter
        .open(req)
        .await
        .map_err(|e| e.to_string())?;
    Ok(json!({
        "number": summary.number,
        "title": summary.title,
        "state": pr_state_to_str(summary.state),
        "url": summary.url,
        "head_ref_name": summary.head_ref_name,
        "base_ref_name": summary.base_ref_name,
        "is_draft": summary.is_draft,
    }))
}

/// `github_pr_list` — list PRs for `repo` (or current git remote).
/// Read-only. `state_filter` accepts gh's wire labels (`open`,
/// `closed`, `merged`, or `all`). Limit defaults to 20 when 0.
#[tauri::command]
pub async fn github_pr_list(
    state: State<'_, AppState>,
    repo: Option<String>,
    state_filter: Option<String>,
    limit: Option<u32>,
) -> CmdResult<Vec<Value>> {
    let adapter = build_adapter(&state)?;
    let summaries: Vec<PrSummary> = adapter
        .list(repo, state_filter, limit.unwrap_or(0))
        .await
        .map_err(|e| e.to_string())?;
    Ok(summaries
        .into_iter()
        .map(|s| {
            json!({
                "number": s.number,
                "title": s.title,
                "state": pr_state_to_str(s.state),
                "url": s.url,
                "head_ref_name": s.head_ref_name,
                "base_ref_name": s.base_ref_name,
                "is_draft": s.is_draft,
            })
        })
        .collect())
}

/// `github_pr_status` — fetch state + review-decision + CI rollup for
/// one PR. Emits exactly one `github_pr_status_fetched` SSB row.
#[tauri::command]
pub async fn github_pr_status(
    state: State<'_, AppState>,
    repo: Option<String>,
    pr_number: u32,
) -> CmdResult<Value> {
    let adapter = build_adapter(&state)?;
    let status: PrStatus = adapter
        .status(repo, pr_number)
        .await
        .map_err(|e| e.to_string())?;
    Ok(json!({
        "state": pr_state_to_str(status.state),
        "review_decision": status.review_decision,
        "ci_rollup_label": status.ci_rollup_label(),
        "review_state_label": status.review_state_label(),
        "check_runs": status.check_runs.iter().map(|c| {
            json!({
                "name": c.name,
                "status": c.status,
                "conclusion": c.conclusion,
            })
        }).collect::<Vec<_>>(),
    }))
}

/// `github_pr_close` — close a PR without merging. Mutation. Emits one
/// `github_pr_closed` SSB row on success.
#[tauri::command]
pub async fn github_pr_close(
    state: State<'_, AppState>,
    repo: Option<String>,
    pr_number: u32,
) -> CmdResult<Value> {
    let adapter = build_adapter(&state)?;
    adapter
        .close(repo, pr_number)
        .await
        .map_err(|e| e.to_string())?;
    Ok(json!({ "pr_number": pr_number, "closed": true }))
}

/// `github_pr_get_active_url` — read-only lookup over the latest
/// `github_pr_opened` SSB row. Returns `{url: null, ...}` when no
/// row exists yet. Used by the dashboard's PR drawer to show a live
/// "active PR" badge without driving the open flow from JS.
///
/// Implementation note: the SSB payload carries the URL hash, not the
/// raw URL. For the dashboard surface we resolve the hash back to a
/// human-readable URL by parsing the most recent `github_pr_opened` row
/// — but since the raw URL is intentionally not in the broadcast
/// stream, this command currently surfaces only the `pr_number` and
/// `ts_ms`. Frontends that need the actual URL must hold a copy from
/// the original `github_pr_open` response (see `githubStore.ts`).
#[tauri::command]
pub async fn github_pr_get_active_url(
    state: State<'_, AppState>,
) -> CmdResult<ActivePrUrlDto> {
    let Some(viewer) = state.viewer.clone() else {
        return Ok(ActivePrUrlDto {
            url: None,
            pr_number: None,
            ts_ms: None,
        });
    };
    let action_str = SsbActionType::GithubPrOpened.as_sql_str();
    let rows = viewer
        .recent_rows_of_type(action_str, 1)
        .map_err(|e| e.to_string())?;
    if let Some(row) = rows.into_iter().next() {
        // payload_json is the typed GithubPrOpenedPayload re-serialized
        // as JSON by recent_rows_of_type — pull pr_number + ts_ms by
        // looking up the snake_case wire keys serde emits.
        let pr_number = row
            .payload_json
            .get("github_pr_opened")
            .and_then(|v| v.get("pr_number"))
            .and_then(serde_json::Value::as_u64)
            .map(|n| n as u32)
            .or_else(|| {
                row.payload_json
                    .get("pr_number")
                    .and_then(serde_json::Value::as_u64)
                    .map(|n| n as u32)
            });
        let ts_ms = row
            .payload_json
            .get("github_pr_opened")
            .and_then(|v| v.get("ts_ms"))
            .and_then(serde_json::Value::as_i64)
            .or_else(|| {
                row.payload_json
                    .get("ts_ms")
                    .and_then(serde_json::Value::as_i64)
            });
        return Ok(ActivePrUrlDto {
            url: None,
            pr_number,
            ts_ms,
        });
    }
    Ok(ActivePrUrlDto {
        url: None,
        pr_number: None,
        ts_ms: None,
    })
}

/// Stringify [`PrState`] using the snake_case wire labels the
/// dashboard renders. Match arms enumerate every variant explicitly so
/// adding a new state at the Rust side forces a deliberate update.
fn pr_state_to_str(state: PrState) -> &'static str {
    match state {
        PrState::Open => "open",
        PrState::Closed => "closed",
        PrState::Merged => "merged",
        PrState::Unknown => "unknown",
    }
}
