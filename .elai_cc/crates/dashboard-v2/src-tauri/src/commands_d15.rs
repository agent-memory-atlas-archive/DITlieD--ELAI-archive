//! D-15 — Discovery-vs-search NoveltyClass read-only Tauri command surface.
//!
//! Two read-only commands feed the Svelte `NoveltyPanel`:
//!
//! - [`get_novelty_recent`]       — last N joined `novelty_classified`
//!   rows for the recent-classifications table.
//! - [`get_novelty_distribution`] — per-class count snapshot over the
//!   same window (used for the pie chart + class histogram).
//!
//! Additive — sibling of `commands_d11` (constructive checkers) +
//! `commands_d12` (baby-step predicate) + `commands_d13` (adversary)
//! + `commands_d16` (conformal calibration). The orchestrator
//! producers
//! ([`elai_orchestrator::novelty::emit_novelty_classified`] +
//! [`elai_orchestrator::agent_pool::dispatch_novelty`]) are the sole
//! writers of the underlying SSB rows; this module only reads.
//!
//! Grounding: arXiv:2406.05761 (LLM novelty detection three-bucket
//! taxonomy); docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
//! §1 Mode C + §4.8.
//!
//! See docs/thefinalglance/D-plan/D-15_novelty_class_labeling.md.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// Typed mirror of [`elai_orchestrator::novelty::NoveltyClassifiedRow`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoveltyClassifiedRowDto {
    pub template_id: String,
    pub class: String,
    pub dkg_match_count: u32,
    pub near_duplicate_threshold: f64,
    pub ts_ms: i64,
}

/// Typed mirror of [`elai_orchestrator::novelty::NoveltyDistribution`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoveltyDistributionDto {
    pub total: u32,
    pub search: u32,
    pub distant_search: u32,
    pub discovery_candidate: u32,
}

/// Aggregated response for the panel: recent rows + per-class counts
/// over the same window so the panel can render the histogram without
/// re-walking the rows in JS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentNovelty {
    pub rows: Vec<NoveltyClassifiedRowDto>,
    pub distribution: NoveltyDistributionDto,
}

/// Return the most recent `novelty_classified` rows (capped at
/// `MAX_LIMIT`). Empty response when no SSB viewer is attached
/// (Tauri-dev fresh boot, no DB).
#[tauri::command]
pub async fn get_novelty_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<RecentNovelty, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(RecentNovelty {
                rows: Vec::new(),
                distribution: NoveltyDistributionDto {
                    total: 0,
                    search: 0,
                    distant_search: 0,
                    discovery_candidate: 0,
                },
            });
        }
    };

    let rows: Vec<NoveltyClassifiedRowDto> =
        elai_orchestrator::novelty::recent_novelty_rows(&viewer, cap)
            .into_iter()
            .map(|r| NoveltyClassifiedRowDto {
                template_id: r.template_id,
                class: r.class,
                dkg_match_count: r.dkg_match_count,
                near_duplicate_threshold: r.near_duplicate_threshold,
                ts_ms: r.ts_ms,
            })
            .collect();

    let dist = elai_orchestrator::novelty::novelty_distribution(&viewer, cap);

    Ok(RecentNovelty {
        rows,
        distribution: NoveltyDistributionDto {
            total: dist.total,
            search: dist.search,
            distant_search: dist.distant_search,
            discovery_candidate: dist.discovery_candidate,
        },
    })
}

/// Return the per-class distribution snapshot only (the panel uses this
/// for a refresh-on-tick that does not need to redraw the row table).
#[tauri::command]
pub async fn get_novelty_distribution(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<NoveltyDistributionDto, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(NoveltyDistributionDto {
                total: 0,
                search: 0,
                distant_search: 0,
                discovery_candidate: 0,
            });
        }
    };

    let d = elai_orchestrator::novelty::novelty_distribution(&viewer, cap);
    Ok(NoveltyDistributionDto {
        total: d.total,
        search: d.search,
        distant_search: d.distant_search,
        discovery_candidate: d.discovery_candidate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_dto_serializes_with_expected_field_names() {
        let dto = NoveltyClassifiedRowDto {
            template_id: "tpl-1".into(),
            class: "search".into(),
            dkg_match_count: 7,
            near_duplicate_threshold: 0.9,
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"template_id\"",
            "\"class\"",
            "\"dkg_match_count\"",
            "\"near_duplicate_threshold\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn distribution_dto_serializes_with_expected_field_names() {
        let dto = NoveltyDistributionDto {
            total: 3,
            search: 1,
            distant_search: 1,
            discovery_candidate: 1,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"total\"",
            "\"search\"",
            "\"distant_search\"",
            "\"discovery_candidate\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn recent_novelty_serializes_with_expected_field_names() {
        let r = RecentNovelty {
            rows: Vec::new(),
            distribution: NoveltyDistributionDto {
                total: 0,
                search: 0,
                distant_search: 0,
                discovery_candidate: 0,
            },
        };
        let s = serde_json::to_string(&r).expect("serialize");
        for key in ["\"rows\"", "\"distribution\""] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }
}
