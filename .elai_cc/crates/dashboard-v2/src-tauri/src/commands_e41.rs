//! E-41: PR review Tauri command surface (read-only).
//!
//! cross-lang-pair: dashboard/tauri/api
//!
//! Single Tauri command — [`pr_review_status`] — that aggregates the
//! five E-41 SSB variants into one read snapshot for the GithubWindow
//! PR Review section. No mutation commands here — write actions live
//! behind the `elai pr-review` CLI subverbs (cross-language symmetry
//! with the C-40 split between dashboard-read and CLI-write).
//!
//! Implementation note: SSB JSON payloads use `serde(tag = "type",
//! rename_all = "snake_case")` shapes for the enum, but
//! `recent_rows_of_type` already unwraps the discriminant — we read
//! both the bare-field path and the prefixed path so this works
//! whichever the upstream SSB row-builder emits.

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::SsbActionType;

use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

/// One row from the `pr_review_composed` SSB stream, flattened for the
/// frontend with the human-readable kind label resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReviewRecentDto {
    pub pr_number_hash: u64,
    pub comment_kind_hash: u64,
    /// Best-effort reverse-lookup of the kind label. The SSB row carries
    /// only the hash so this is "unknown" by default; future revisions
    /// can join against an in-process lookup table.
    pub comment_kind_label: String,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrCommentWatchedDto {
    pub pr_number_hash: u64,
    pub new_comments_count: u32,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrPackageVerifyResultDto {
    pub pr_number_hash: u64,
    pub slopsquat_count: u32,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrSemgrepGateResultDto {
    pub pr_number_hash: u64,
    pub findings_count: u32,
    pub severity_max_hash: u64,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrAutoMergeBlockedDto {
    pub pr_number_hash: u64,
    pub block_reason_hash: u64,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReviewStatusDto {
    pub recent_reviews: Vec<PrReviewRecentDto>,
    pub last_watcher_tick: Option<PrCommentWatchedDto>,
    pub last_slopsquat: Option<PrPackageVerifyResultDto>,
    pub last_semgrep: Option<PrSemgrepGateResultDto>,
    pub last_auto_merge_block: Option<PrAutoMergeBlockedDto>,
}

/// Aggregate the latest E-41 PR review SSB rows into one snapshot for
/// the GithubWindow PR Review section.
#[tauri::command]
pub async fn pr_review_status(state: State<'_, AppState>) -> CmdResult<PrReviewStatusDto> {
    let Some(viewer) = state.viewer.clone() else {
        return Ok(PrReviewStatusDto {
            recent_reviews: Vec::new(),
            last_watcher_tick: None,
            last_slopsquat: None,
            last_semgrep: None,
            last_auto_merge_block: None,
        });
    };

    let mut recent_reviews: Vec<PrReviewRecentDto> = Vec::new();
    let rows = viewer
        .recent_rows_of_type(SsbActionType::PrReviewComposed.as_sql_str(), 8)
        .map_err(|e| e.to_string())?;
    for row in rows {
        let pr_number_hash = lookup_u64(&row.payload_json, "pr_review_composed", "pr_number_hash");
        let comment_kind_hash =
            lookup_u64(&row.payload_json, "pr_review_composed", "comment_kind_hash");
        let ts_ms = lookup_i64(&row.payload_json, "pr_review_composed", "ts_ms");
        recent_reviews.push(PrReviewRecentDto {
            pr_number_hash,
            comment_kind_hash,
            comment_kind_label: "unknown".to_string(),
            ts_ms,
        });
    }

    let last_watcher_tick = viewer
        .recent_rows_of_type(SsbActionType::PrCommentWatched.as_sql_str(), 1)
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .map(|row| PrCommentWatchedDto {
            pr_number_hash: lookup_u64(&row.payload_json, "pr_comment_watched", "pr_number_hash"),
            new_comments_count: lookup_u32(
                &row.payload_json,
                "pr_comment_watched",
                "new_comments_count",
            ),
            ts_ms: lookup_i64(&row.payload_json, "pr_comment_watched", "ts_ms"),
        });

    let last_slopsquat = viewer
        .recent_rows_of_type(SsbActionType::PrPackageVerifyResult.as_sql_str(), 1)
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .map(|row| PrPackageVerifyResultDto {
            pr_number_hash: lookup_u64(
                &row.payload_json,
                "pr_package_verify_result",
                "pr_number_hash",
            ),
            slopsquat_count: lookup_u32(
                &row.payload_json,
                "pr_package_verify_result",
                "slopsquat_count",
            ),
            ts_ms: lookup_i64(&row.payload_json, "pr_package_verify_result", "ts_ms"),
        });

    let last_semgrep = viewer
        .recent_rows_of_type(SsbActionType::PrSemgrepGateResult.as_sql_str(), 1)
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .map(|row| PrSemgrepGateResultDto {
            pr_number_hash: lookup_u64(
                &row.payload_json,
                "pr_semgrep_gate_result",
                "pr_number_hash",
            ),
            findings_count: lookup_u32(
                &row.payload_json,
                "pr_semgrep_gate_result",
                "findings_count",
            ),
            severity_max_hash: lookup_u64(
                &row.payload_json,
                "pr_semgrep_gate_result",
                "severity_max_hash",
            ),
            ts_ms: lookup_i64(&row.payload_json, "pr_semgrep_gate_result", "ts_ms"),
        });

    let last_auto_merge_block = viewer
        .recent_rows_of_type(SsbActionType::PrAutoMergeBlocked.as_sql_str(), 1)
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .map(|row| PrAutoMergeBlockedDto {
            pr_number_hash: lookup_u64(
                &row.payload_json,
                "pr_auto_merge_blocked",
                "pr_number_hash",
            ),
            block_reason_hash: lookup_u64(
                &row.payload_json,
                "pr_auto_merge_blocked",
                "block_reason_hash",
            ),
            ts_ms: lookup_i64(&row.payload_json, "pr_auto_merge_blocked", "ts_ms"),
        });

    Ok(PrReviewStatusDto {
        recent_reviews,
        last_watcher_tick,
        last_slopsquat,
        last_semgrep,
        last_auto_merge_block,
    })
}

/// Best-effort lookup of a u64 field in either the prefixed or bare
/// payload shape (the SSB row may or may not be tagged).
fn lookup_u64(payload: &serde_json::Value, prefix: &str, key: &str) -> u64 {
    payload
        .get(prefix)
        .and_then(|v| v.get(key))
        .and_then(serde_json::Value::as_u64)
        .or_else(|| payload.get(key).and_then(serde_json::Value::as_u64))
        .unwrap_or(0)
}

fn lookup_u32(payload: &serde_json::Value, prefix: &str, key: &str) -> u32 {
    u32::try_from(lookup_u64(payload, prefix, key)).unwrap_or(u32::MAX)
}

fn lookup_i64(payload: &serde_json::Value, prefix: &str, key: &str) -> i64 {
    payload
        .get(prefix)
        .and_then(|v| v.get(key))
        .and_then(serde_json::Value::as_i64)
        .or_else(|| payload.get(key).and_then(serde_json::Value::as_i64))
        .unwrap_or(0)
}
