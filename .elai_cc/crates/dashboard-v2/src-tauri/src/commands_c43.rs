//! C-43 — Phase 4H scope-freeze Tauri command surface.
//!
//! Single read-only command [`scope_freeze_status`] feeds the Svelte
//! `FreezeStatusBar` widget. Backed by
//! [`elai_orchestrator::scope_freeze::verify_freeze`] (which loads the
//! manifest and walks each frozen item).
//!
//! Additive — sibling of `commands_d9` (LockedKeysAuditor). The
//! orchestrator producer (`scope_freeze::verify_freeze`) is the sole
//! reader of `<workspace>/scope-freeze.toml`; this module does not
//! mutate the manifest.
//!
//! ELAI design hypothesis. The source plan cited arXiv:2604.21725
//! (AEL) as ground truth but two independent WebFetches confirmed
//! that paper is Thompson-Sampling agent learning, NOT scope-freeze.
//! See workflow/sessions/s-25120/mistakes.md for the bad-cite
//! analysis.

use std::path::PathBuf;

use elai_orchestrator::scope_freeze::{verify_freeze, FreezeViolation};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// Flat status snapshot for the dashboard FreezeStatusBar widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreezeStatusDto {
    /// `true` when the manifest loads cleanly and every frozen item is
    /// unmodified since `ratified_at_ms`.
    pub clean: bool,
    /// `ratification_ts_ms` from the manifest. Zero when the manifest
    /// fails to load (the dashboard renders the badge red in that
    /// case).
    pub ratified_at_ms: u64,
    /// Wall-clock ms at which the verify pass ran.
    pub verified_at_ms: u64,
    /// Number of violations observed. Always zero when `clean`.
    pub violation_count: u32,
    /// Per-violation breakdown (workspace-relative path + mtime).
    /// Empty when `clean`.
    pub violations: Vec<FreezeViolationDto>,
    /// Populated when the manifest is missing or malformed; the
    /// dashboard surfaces this in the badge tooltip.
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreezeViolationDto {
    pub item_path: String,
    pub item_path_hash: u64,
    pub frozen_since_ms: u64,
    pub modified_at_ms: u64,
}

impl From<FreezeViolation> for FreezeViolationDto {
    fn from(v: FreezeViolation) -> Self {
        Self {
            item_path: v.item_path,
            item_path_hash: v.item_path_hash,
            frozen_since_ms: v.frozen_since_ms,
            modified_at_ms: v.modified_at_ms,
        }
    }
}

/// Resolve the workspace root. Falls back to the dashboard's active
/// project_root (set by `open_project_from_folder`) when the
/// caller-provided override is None. Never falls back to CWD: that
/// returned ELAI's own binary tree when the bat was double-clicked.
fn resolve_workspace_root(state: &AppState, override_root: Option<String>) -> Result<PathBuf, String> {
    if let Some(path) = override_root {
        return Ok(PathBuf::from(path));
    }
    Ok(state.project_root_snapshot())
}

/// Read the current scope-freeze manifest from disk + verify every
/// frozen item against its on-disk mtime. Fail-soft on the dashboard
/// surface: a missing or malformed manifest renders the badge red
/// rather than crashing the panel.
#[tauri::command]
pub async fn scope_freeze_status(
    state: State<'_, AppState>,
    workspace_root: Option<String>,
) -> Result<FreezeStatusDto, String> {
    let root = match resolve_workspace_root(&state, workspace_root) {
        Ok(r) => r,
        Err(e) => {
            return Ok(FreezeStatusDto {
                clean: false,
                ratified_at_ms: 0,
                verified_at_ms: 0,
                violation_count: 0,
                violations: Vec::new(),
                error: Some(e),
            });
        }
    };
    match verify_freeze(&root) {
        Ok(report) => {
            let count = report.violations.len() as u32;
            let violations: Vec<FreezeViolationDto> =
                report.violations.into_iter().map(Into::into).collect();
            Ok(FreezeStatusDto {
                clean: count == 0,
                ratified_at_ms: report.ratified_at,
                verified_at_ms: report.verified_at,
                violation_count: count,
                violations,
                error: None,
            })
        }
        Err(e) => Ok(FreezeStatusDto {
            clean: false,
            ratified_at_ms: 0,
            verified_at_ms: 0,
            violation_count: 0,
            violations: Vec::new(),
            error: Some(e.to_string()),
        }),
    }
}
