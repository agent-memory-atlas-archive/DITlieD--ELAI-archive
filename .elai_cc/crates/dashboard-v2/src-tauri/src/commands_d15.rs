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
///
/// FINAL TRANCHE B4 / D-15 -- `candidate_status` carries the optional
/// lifecycle stage ("queued" / "ratified" / "rejected") for discovery
/// candidates; absent for legacy rows + non-DiscoveryCandidate classes.
/// Serialized via `#[serde(default, skip_serializing_if = "Option::is_none")]`
/// so the dashboard front-end sees the field only when set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoveltyClassifiedRowDto {
    pub template_id: String,
    pub class: String,
    pub dkg_match_count: u32,
    pub near_duplicate_threshold: f64,
    pub ts_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_status: Option<String>,
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
                // FINAL TRANCHE B4 / D-15: forward the optional lifecycle
                // stage to the front-end so NoveltyPanel can render
                // Queued / Ratified / Rejected badges per row.
                candidate_status: r.candidate_status,
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

/// D-27 gap closure: per-class breakdown view consumed by the
/// `NoveltyBreakdown.svelte` inline widget. Extends
/// [`NoveltyDistributionDto`] with per-class percentages and a dominant-
/// class label so the panel can render bars + a tinted header without
/// running division client-side. Read-only — composes the same producer
/// rows as [`get_novelty_distribution`].
///
/// Percentages are clamped to the inclusive `[0.0, 100.0]` range and
/// computed against the same `total` count returned by the orchestrator
/// distribution helper. `dominant_class` is the snake_case class label
/// with the highest count (ties broken by the canonical class order
/// search > distant_search > discovery_candidate so the choice is
/// deterministic). `None` when `total == 0`.
///
/// Added per workflow/sessions/s-night-2026-05-30/cross-check/D/
/// D-09_d26_d27_d28.md TASK-D27-003 / TASK-D27-005 — the D-27 plan
/// names `get_novelty_breakdown` as a literal deliverable but only the
/// composite `get_mode_routing_refusal_combined` + the D-15 distribution
/// surface existed. This command closes the literal-name gap and feeds
/// the matching `NoveltyBreakdown.svelte` panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoveltyBreakdownDto {
    pub total: u32,
    pub search: u32,
    pub distant_search: u32,
    pub discovery_candidate: u32,
    /// `search` count as a percentage of `total`, clamped to `[0.0, 100.0]`.
    pub search_pct: f64,
    /// `distant_search` count as a percentage of `total`, clamped to `[0.0, 100.0]`.
    pub distant_search_pct: f64,
    /// `discovery_candidate` count as a percentage of `total`, clamped to `[0.0, 100.0]`.
    pub discovery_candidate_pct: f64,
    /// Highest-count class (snake_case) or `None` when `total == 0`.
    pub dominant_class: Option<String>,
    /// Cap applied to the underlying distribution query — echoed back so
    /// the frontend can render a "last N classifications" footer matching
    /// the panel contract used elsewhere.
    pub limit_applied: u32,
}

/// Pure helper: compute the canonical breakdown from a raw distribution.
/// Pulled out so the unit test can exercise it without spinning up an
/// `AppState` / SSB viewer pair.
fn breakdown_from_distribution(
    total: u32,
    search: u32,
    distant_search: u32,
    discovery_candidate: u32,
    limit_applied: u32,
) -> NoveltyBreakdownDto {
    let pct = |n: u32| -> f64 {
        if total == 0 {
            return 0.0;
        }
        let raw = (f64::from(n) * 100.0) / f64::from(total);
        raw.clamp(0.0, 100.0)
    };
    let dominant_class = if total == 0 {
        None
    } else {
        // Canonical class order for tie-break: search > distant_search >
        // discovery_candidate. Mirrors arXiv:2406.05761 three-bucket
        // taxonomy column order used in D-15 producer.
        let mut best: (&str, u32) = ("search", search);
        if distant_search > best.1 {
            best = ("distant_search", distant_search);
        }
        if discovery_candidate > best.1 {
            best = ("discovery_candidate", discovery_candidate);
        }
        Some(best.0.to_string())
    };
    NoveltyBreakdownDto {
        total,
        search,
        distant_search,
        discovery_candidate,
        search_pct: pct(search),
        distant_search_pct: pct(distant_search),
        discovery_candidate_pct: pct(discovery_candidate),
        dominant_class,
        limit_applied,
    }
}

#[tauri::command]
pub async fn get_novelty_breakdown(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<NoveltyBreakdownDto, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let limit_applied = u32::try_from(cap).unwrap_or(u32::MAX);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(breakdown_from_distribution(0, 0, 0, 0, limit_applied));
        }
    };

    let d = elai_orchestrator::novelty::novelty_distribution(&viewer, cap);
    Ok(breakdown_from_distribution(
        d.total,
        d.search,
        d.distant_search,
        d.discovery_candidate,
        limit_applied,
    ))
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
            // FINAL TRANCHE B4 / D-15: None preserves the legacy wire
            // shape for Search-class rows (no lifecycle).
            candidate_status: None,
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

    #[test]
    fn breakdown_zero_total_returns_zero_pct_and_none_dominant() {
        let b = breakdown_from_distribution(0, 0, 0, 0, 50);
        assert_eq!(b.total, 0);
        assert_eq!(b.search_pct, 0.0);
        assert_eq!(b.distant_search_pct, 0.0);
        assert_eq!(b.discovery_candidate_pct, 0.0);
        assert!(b.dominant_class.is_none());
        assert_eq!(b.limit_applied, 50);
    }

    #[test]
    fn breakdown_percentages_sum_within_rounding_when_classes_split() {
        let b = breakdown_from_distribution(10, 5, 3, 2, 50);
        assert_eq!(b.total, 10);
        assert!((b.search_pct - 50.0).abs() < 1e-9);
        assert!((b.distant_search_pct - 30.0).abs() < 1e-9);
        assert!((b.discovery_candidate_pct - 20.0).abs() < 1e-9);
        assert_eq!(b.dominant_class.as_deref(), Some("search"));
    }

    #[test]
    fn breakdown_dominant_class_tie_break_prefers_search() {
        // Tie between search and distant_search: canonical order picks
        // search first per arXiv:2406.05761 column-order convention.
        let b = breakdown_from_distribution(4, 2, 2, 0, 50);
        assert_eq!(b.dominant_class.as_deref(), Some("search"));

        // Distant_search strictly larger -> wins.
        let b = breakdown_from_distribution(5, 1, 3, 1, 50);
        assert_eq!(b.dominant_class.as_deref(), Some("distant_search"));

        // Discovery_candidate strictly larger -> wins.
        let b = breakdown_from_distribution(7, 1, 1, 5, 50);
        assert_eq!(b.dominant_class.as_deref(), Some("discovery_candidate"));
    }

    #[test]
    fn breakdown_dto_serializes_with_expected_field_names() {
        let b = breakdown_from_distribution(4, 2, 1, 1, 50);
        let s = serde_json::to_string(&b).expect("serialize");
        for key in [
            "\"total\"",
            "\"search\"",
            "\"distant_search\"",
            "\"discovery_candidate\"",
            "\"search_pct\"",
            "\"distant_search_pct\"",
            "\"discovery_candidate_pct\"",
            "\"dominant_class\"",
            "\"limit_applied\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }
}
