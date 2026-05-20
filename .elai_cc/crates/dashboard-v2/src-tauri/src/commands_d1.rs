//! D-1: ReasoningTemplate ArchiveChain Tauri command surface.
//!
//! Two commands feed the dashboard `MutationArchivePanel`:
//!
//! * [`get_archive_recent`] — last N [`ArchiveEntry`] rows from
//!   `.elai_cc/archive/reasoning-templates.jsonl` (or empty Vec if the chain
//!   has not been initialised yet — fresh repos render the empty-state
//!   panel without errors).
//! * [`get_reasoning_template_by_id`] — look up a single template by its
//!   hex id (returns Ok(None) if not in the chain).
//!
//! Both commands are additive — they do NOT refactor any existing
//! `commands.rs` surface. Verify-chain is called at load time; a mismatched
//! file surfaces as an error string so the panel can render a clear
//! tamper-warning state.

use std::path::PathBuf;

use elai_core::paths::ELAI_DATA_DIR_NAME;
use elai_orchestrator::archive_chain::ArchiveChain;
use elai_orchestrator::reasoning_template::ReasoningTemplateDto;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// Recent-archive row surfaced to the frontend. Wraps [`ReasoningTemplateDto`]
/// plus the chain-walk lifecycle metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveRecentRow {
    pub template: ReasoningTemplateDto,
    pub self_hash: String,
    pub parent_hash: Option<String>,
    pub promoted_at_ms: Option<i64>,
    pub retired_at_ms: Option<i64>,
    pub retirement_reason: Option<String>,
}

/// Canonical archive path relative to `project_root`.
fn archive_jsonl_path(project_root: &std::path::Path) -> PathBuf {
    project_root
        .join(ELAI_DATA_DIR_NAME)
        .join("archive")
        .join("reasoning-templates.jsonl")
}

/// Return the last `limit` [`ArchiveEntry`] rows from the JSONL archive,
/// flattened to the [`ArchiveRecentRow`] DTO. Missing file -> empty Vec.
/// `verify_chain` is run on every call (cheap for the expected archive size
/// of < 1000 rows) so the dashboard can refuse to render a tampered chain.
#[tauri::command]
pub async fn get_archive_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<ArchiveRecentRow>, String> {
    let limit = limit.unwrap_or(50).min(500);
    let project_root = state.project_root_snapshot();
    let path = archive_jsonl_path(&project_root);
    let chain = ArchiveChain::load(path).map_err(|e| e.to_string())?;
    // Tamper defence — refuse to surface rows on a broken chain.
    chain.verify_chain().map_err(|e| e.to_string())?;
    let rows = chain
        .recent(limit)
        .into_iter()
        .map(|entry| ArchiveRecentRow {
            template: ReasoningTemplateDto::from(&entry.template),
            self_hash: entry.self_hash.as_hex(),
            parent_hash: entry.parent_hash.map(|h| h.as_hex()),
            promoted_at_ms: entry.promoted_at_ms,
            retired_at_ms: entry.retired_at_ms,
            retirement_reason: entry.retirement_reason.map(|r| r.as_str().to_string()),
        })
        .collect();
    Ok(rows)
}

/// Return a single reasoning template (DTO shape) by its hex id, or
/// `Ok(None)` if not present in the archive.
#[tauri::command]
pub async fn get_reasoning_template_by_id(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<ReasoningTemplateDto>, String> {
    let project_root = state.project_root_snapshot();
    let path = archive_jsonl_path(&project_root);
    let chain = ArchiveChain::load(path).map_err(|e| e.to_string())?;
    chain.verify_chain().map_err(|e| e.to_string())?;
    // The DTO carries `id` as the simple-hex form (no hyphens). Accept both
    // canonical UUID hyphenated form and the hex form for caller convenience.
    let normalised = id.replace('-', "").to_lowercase();
    let found = chain
        .entries()
        .iter()
        .find(|e| e.template.id.as_hex().to_lowercase() == normalised)
        .map(|e| ReasoningTemplateDto::from(&e.template));
    Ok(found)
}
