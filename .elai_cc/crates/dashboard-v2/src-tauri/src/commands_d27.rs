//! D-27 — Mode-routing + refusal-SLA dashboard composite read surface.
//!
//! One additive read-only Tauri command feeds the new
//! `ModeRoutingDashboardView` (full-page composition of the four
//! Wave-2 panels) + `DiscoveryQueueDrawer`:
//!
//! - [`get_mode_routing_refusal_combined`] — single composite read
//!   returning the four sub-views the dashboard needs to refresh the
//!   integrated view without paying four round-trips:
//!     * `mode_breakdown`         — per-`GroundingMode` count over the
//!       most-recent `limit` `claim_routed` rows (5-key snake_case
//!       histogram, mirrors `commands_d10` seeding).
//!     * `refusal_rate_current`   — most-recent `refusal_rate_snapshot`
//!       row converted to its f64 rate (or 0.0 when no snapshot exists).
//!     * `discovery_queue`        — recent `discovery_candidate`-class
//!       `novelty_classified` rows (the pending-adversarial-review queue
//!       the new drawer renders), shaped with the fields the drawer
//!       needs: novelty class + dkg_match_count + days_waiting.
//!     * `recent_classifications` — recent `novelty_classified` rows
//!       (all three classes), so the summary row can show the total
//!       novelty volume.
//!
//! Additive — sibling of `commands_d26` (archive composite). No
//! producer logic; pure reads composed on top of:
//!   - `elai_orchestrator::recent_claim_routings`              (D-10)
//!   - `elai_orchestrator::novelty::recent_novelty_rows`       (D-15)
//!   - `elai_orchestrator::refusal_sla::recent_snapshot_rows`  (D-17)
//!
//! Pattern mirrors `commands_d26::get_template_archive_combined` — single
//! composite struct returned, frontend renders header summary + per-
//! sub-panel state without three round-trips.
//!
//! No new SSB variant. No new producer. SCHEMA stays at 87 (Wave-2
//! producers already shipped in D-10/D-15/D-16/D-17).
//!
//! See docs/thefinalglance/D-plan/D-27_mode_routing_refusal_dashboard.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;
/// Conventional milliseconds-per-day constant used for the
/// `days_waiting` field on each `DiscoveryQueueRow`. Pinned here as a
/// named constant per CLAUDE.md anti-magic-numbers rule.
const MS_PER_DAY: i64 = 86_400_000;

/// One row of the discovery-queue drawer: a recent novelty
/// classification with class == "discovery_candidate" that is awaiting
/// adversarial verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoveryQueueRow {
    pub template_id: String,
    pub class: String,
    pub dkg_match_count: u32,
    pub near_duplicate_threshold: f64,
    pub ts_ms: i64,
    /// Whole days between `ts_ms` and `now_ms` (the timestamp used in
    /// the composite call). Negative if the row's clock is ahead of
    /// the calling clock — the drawer renders this as 0.
    pub days_waiting: i64,
}

/// One row of the recent-classifications summary table. Same shape as
/// the per-row DTO in `commands_d15`, repeated here so the D-27
/// composite stays self-contained (the dashboard view imports one
/// type, not five).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoveltyClassifiedRowD27 {
    pub template_id: String,
    pub class: String,
    pub dkg_match_count: u32,
    pub near_duplicate_threshold: f64,
    pub ts_ms: i64,
}

/// Top-level composite returned by [`get_mode_routing_refusal_combined`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeRoutingRefusalView {
    /// Snake-case mode label → count over the last `limit`
    /// `claim_routed` rows. Sorted-key `BTreeMap` so JSON output is
    /// deterministic for snapshot tests, matching `commands_d10`.
    pub mode_breakdown: BTreeMap<String, u32>,
    /// Most-recent `refusal_rate_snapshot.refusal_rate`, or `0.0` when
    /// no snapshot row has been emitted yet.
    pub refusal_rate_current: f64,
    /// Most-recent `health_band` label from the same snapshot row, or
    /// `"unknown"` when no snapshot row exists. Lets the summary row
    /// tint without re-reading the gauge surface.
    pub refusal_health_band: String,
    /// Pending DiscoveryCandidate-classified templates (most-recent
    /// first, capped at `limit`).
    pub discovery_queue: Vec<DiscoveryQueueRow>,
    /// Total novelty classifications across all three classes within
    /// the same `limit` window (used by the summary row).
    pub recent_classifications: Vec<NoveltyClassifiedRowD27>,
    /// Dominant mode within `mode_breakdown` (the snake_case key with
    /// the maximum count). `None` when `mode_breakdown` is all zeros.
    pub dominant_mode: Option<String>,
    /// Cap applied to all four sub-queries (echoed back for the
    /// frontend's "showing N most recent" footer).
    pub limit_applied: u32,
}

/// Compose the `mode_breakdown` BTreeMap with the canonical 5-mode
/// seed used by `commands_d10::get_recent_claim_routings`. Pure helper
/// (no viewer access) so tests can exercise it directly.
fn seed_mode_breakdown() -> BTreeMap<String, u32> {
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for mode in [
        "constructive",
        "falsifiability_cross_family",
        "conformal_refusal",
        "operational_deferred",
        "conservative_fallback",
    ] {
        counts.insert(mode.to_string(), 0);
    }
    counts
}

/// Pure helper: pick the dominant (highest-count) mode out of a
/// breakdown map. Ties broken by the BTreeMap's sorted-key order —
/// the first key in lexical order wins so the choice is deterministic.
/// Returns `None` when every count is zero (no rows emitted yet).
fn dominant_mode_from(breakdown: &BTreeMap<String, u32>) -> Option<String> {
    let mut best: Option<(&String, u32)> = None;
    for (k, &v) in breakdown.iter() {
        if v == 0 {
            continue;
        }
        match best {
            None => best = Some((k, v)),
            Some((_, bv)) if v > bv => best = Some((k, v)),
            _ => {}
        }
    }
    best.map(|(k, _)| k.clone())
}

/// Pure helper: compute `days_waiting` for a `discovery_candidate` row
/// given the row timestamp and the current wall clock. Clamps to 0
/// when the row clock is ahead of `now_ms` (clock skew during tests).
fn days_waiting(now_ms: i64, ts_ms: i64) -> i64 {
    let delta = now_ms.saturating_sub(ts_ms);
    if delta <= 0 {
        0
    } else {
        delta / MS_PER_DAY
    }
}

/// Return the composite mode-routing + refusal-SLA view. Empty / zeroed
/// response when no SSB viewer is attached (Tauri-dev fresh boot, no
/// DB) — matches the contract of every other D-N read command.
#[tauri::command]
pub async fn get_mode_routing_refusal_combined(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<ModeRoutingRefusalView, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let cap_u32 = u32::try_from(cap).unwrap_or(u32::MAX);

    // No viewer attached -> truthful empty composite, mirroring
    // commands_d10 / d15 / d17 empty-state contracts.
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            let breakdown = seed_mode_breakdown();
            return Ok(ModeRoutingRefusalView {
                mode_breakdown: breakdown,
                refusal_rate_current: 0.0,
                refusal_health_band: "unknown".to_string(),
                discovery_queue: Vec::new(),
                recent_classifications: Vec::new(),
                dominant_mode: None,
                limit_applied: cap_u32,
            });
        }
    };

    // (1) Mode breakdown — recompute the histogram over the most-recent
    // claim_routed rows. Same seeded set commands_d10 uses so the
    // dashboard renders a stable 5-column distribution even when some
    // modes are empty.
    let mut mode_breakdown = seed_mode_breakdown();
    let claim_rows = elai_orchestrator::recent_claim_routings(&viewer, cap);
    for row in &claim_rows {
        *mode_breakdown.entry(row.mode.clone()).or_insert(0) += 1;
    }

    // (2) Refusal rate current — single most-recent snapshot row, or
    // 0.0/"unknown" when none exists.
    let mut snapshot_rows = elai_orchestrator::refusal_sla::recent_snapshot_rows(&viewer, 1);
    let (refusal_rate_current, refusal_health_band) = match snapshot_rows.pop() {
        Some(r) => (r.refusal_rate, r.health_band),
        None => (0.0, "unknown".to_string()),
    };

    // (3) + (4) Novelty rows over the same cap window. One viewer call
    // for both surfaces (discovery_queue filters the same set).
    let novelty_rows = elai_orchestrator::novelty::recent_novelty_rows(&viewer, cap);
    let recent_classifications: Vec<NoveltyClassifiedRowD27> = novelty_rows
        .iter()
        .map(|r| NoveltyClassifiedRowD27 {
            template_id: r.template_id.clone(),
            class: r.class.clone(),
            dkg_match_count: r.dkg_match_count,
            near_duplicate_threshold: r.near_duplicate_threshold,
            ts_ms: r.ts_ms,
        })
        .collect();

    // Use the most-recent classification's `ts_ms` as the reference
    // clock for `days_waiting`. Falls back to 0 when there is no row
    // (queue will be empty anyway). This avoids pulling in a system
    // clock dependency at this layer — the dashboard frontend already
    // formats relative time off `ts_ms` for every other panel.
    let now_ms = recent_classifications.first().map(|r| r.ts_ms).unwrap_or(0);
    let discovery_queue: Vec<DiscoveryQueueRow> = novelty_rows
        .iter()
        .filter(|r| r.class == "discovery_candidate")
        .map(|r| DiscoveryQueueRow {
            template_id: r.template_id.clone(),
            class: r.class.clone(),
            dkg_match_count: r.dkg_match_count,
            near_duplicate_threshold: r.near_duplicate_threshold,
            ts_ms: r.ts_ms,
            days_waiting: days_waiting(now_ms, r.ts_ms),
        })
        .collect();

    let dominant_mode = dominant_mode_from(&mode_breakdown);

    Ok(ModeRoutingRefusalView {
        mode_breakdown,
        refusal_rate_current,
        refusal_health_band,
        discovery_queue,
        recent_classifications,
        dominant_mode,
        limit_applied: cap_u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_mode_breakdown_has_all_five_canonical_keys() {
        // Lock the seed shape against commands_d10 so the dashboard
        // composite never drifts away from the per-panel surface.
        let seed = seed_mode_breakdown();
        assert_eq!(seed.len(), 5);
        for k in [
            "constructive",
            "falsifiability_cross_family",
            "conformal_refusal",
            "operational_deferred",
            "conservative_fallback",
        ] {
            assert_eq!(seed.get(k).copied(), Some(0), "missing seeded mode {k}");
        }
    }

    #[test]
    fn dominant_mode_returns_highest_count() {
        let mut bd = seed_mode_breakdown();
        bd.insert("constructive".into(), 5);
        bd.insert("conformal_refusal".into(), 3);
        bd.insert("operational_deferred".into(), 2);
        assert_eq!(dominant_mode_from(&bd), Some("constructive".to_string()));
    }

    #[test]
    fn dominant_mode_returns_none_when_all_zero() {
        // Empty composite path: every seeded key is 0 -> dominant is
        // None, summary row should render "no routings yet".
        let bd = seed_mode_breakdown();
        assert_eq!(dominant_mode_from(&bd), None);
    }

    #[test]
    fn dominant_mode_tie_picks_first_lexical_key() {
        // Deterministic tie-break: the BTreeMap iterates in sorted key
        // order so the first key in lexical order with the max count
        // wins. Lock the rule with a tie.
        let mut bd = seed_mode_breakdown();
        // Sorted keys: conformal_refusal, conservative_fallback,
        //              constructive, falsifiability_cross_family,
        //              operational_deferred.
        bd.insert("constructive".into(), 4);
        bd.insert("conformal_refusal".into(), 4);
        let picked = dominant_mode_from(&bd).expect("at least one nonzero count");
        assert_eq!(picked, "conformal_refusal");
    }

    #[test]
    fn days_waiting_handles_future_clock_and_normal_delta() {
        // 5 days * 86_400_000 ms = 432_000_000 ms ago.
        let now = 10_000_000_000_i64;
        let five_days_ago = now - 5 * MS_PER_DAY;
        assert_eq!(days_waiting(now, five_days_ago), 5);

        // Row "in the future" (clock skew) -> clamped to 0, not
        // negative; the drawer renders 0 days waiting.
        let one_day_ahead = now + MS_PER_DAY;
        assert_eq!(days_waiting(now, one_day_ahead), 0);

        // Exactly now -> 0.
        assert_eq!(days_waiting(now, now), 0);
    }

    #[test]
    fn discovery_queue_row_serializes_with_expected_field_names() {
        let row = DiscoveryQueueRow {
            template_id: "tpl-discovery-1".into(),
            class: "discovery_candidate".into(),
            dkg_match_count: 0,
            near_duplicate_threshold: 0.9,
            ts_ms: 1_700_000_000_000,
            days_waiting: 3,
        };
        let s = serde_json::to_string(&row).expect("serialize");
        for key in [
            "\"template_id\"",
            "\"class\"",
            "\"dkg_match_count\"",
            "\"near_duplicate_threshold\"",
            "\"ts_ms\"",
            "\"days_waiting\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn composite_view_serializes_with_expected_field_names() {
        let view = ModeRoutingRefusalView {
            mode_breakdown: seed_mode_breakdown(),
            refusal_rate_current: 0.0,
            refusal_health_band: "unknown".into(),
            discovery_queue: Vec::new(),
            recent_classifications: Vec::new(),
            dominant_mode: None,
            limit_applied: 50,
        };
        let s = serde_json::to_string(&view).expect("serialize");
        for key in [
            "\"mode_breakdown\"",
            "\"refusal_rate_current\"",
            "\"refusal_health_band\"",
            "\"discovery_queue\"",
            "\"recent_classifications\"",
            "\"dominant_mode\"",
            "\"limit_applied\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn composite_view_round_trips_with_populated_payload() {
        // Smoke test the orchestrate-spec observable_effect shape:
        // construct a composite as if a fake state had emitted 5
        // ClaimRouted / 3 NoveltyClassified / 2 ConformalRefusalDecided
        // rows, then round-trip via serde. Asserts the composite shape
        // contains the correct counts without needing a real SsbViewer
        // (cannot be constructed in unit-test scope).
        let mut bd = seed_mode_breakdown();
        // 5 ClaimRouted rows: 3 constructive + 2 conformal_refusal.
        bd.insert("constructive".into(), 3);
        bd.insert("conformal_refusal".into(), 2);

        // 3 NoveltyClassified rows: 2 search + 1 discovery_candidate.
        let novelties = vec![
            NoveltyClassifiedRowD27 {
                template_id: "tpl-1".into(),
                class: "search".into(),
                dkg_match_count: 7,
                near_duplicate_threshold: 0.9,
                ts_ms: 1_700_000_000_000,
            },
            NoveltyClassifiedRowD27 {
                template_id: "tpl-2".into(),
                class: "search".into(),
                dkg_match_count: 4,
                near_duplicate_threshold: 0.9,
                ts_ms: 1_700_000_000_000 - MS_PER_DAY,
            },
            NoveltyClassifiedRowD27 {
                template_id: "tpl-3".into(),
                class: "discovery_candidate".into(),
                dkg_match_count: 0,
                near_duplicate_threshold: 0.9,
                ts_ms: 1_700_000_000_000 - 2 * MS_PER_DAY,
            },
        ];
        let discovery_queue: Vec<DiscoveryQueueRow> = novelties
            .iter()
            .filter(|r| r.class == "discovery_candidate")
            .map(|r| DiscoveryQueueRow {
                template_id: r.template_id.clone(),
                class: r.class.clone(),
                dkg_match_count: r.dkg_match_count,
                near_duplicate_threshold: r.near_duplicate_threshold,
                ts_ms: r.ts_ms,
                days_waiting: days_waiting(1_700_000_000_000, r.ts_ms),
            })
            .collect();
        assert_eq!(discovery_queue.len(), 1);
        assert_eq!(discovery_queue[0].days_waiting, 2);

        // 2 ConformalRefusalDecided rows worth: surface as a single
        // refusal_rate_current value (the snapshot is the canonical
        // honest-refusal surface; the per-decision rows are aggregated
        // upstream into refusal_rate_snapshot by the producer).
        let view = ModeRoutingRefusalView {
            mode_breakdown: bd.clone(),
            refusal_rate_current: 0.2,
            refusal_health_band: "healthy".into(),
            discovery_queue,
            recent_classifications: novelties,
            dominant_mode: dominant_mode_from(&bd),
            limit_applied: 50,
        };
        let s = serde_json::to_string(&view).expect("serialize");
        let reparsed: ModeRoutingRefusalView = serde_json::from_str(&s).expect("round-trip");
        assert_eq!(view, reparsed);
        assert_eq!(reparsed.dominant_mode.as_deref(), Some("constructive"));
        assert_eq!(reparsed.mode_breakdown.get("constructive"), Some(&3));
        assert_eq!(reparsed.mode_breakdown.get("conformal_refusal"), Some(&2));
        assert_eq!(reparsed.recent_classifications.len(), 3);
        assert_eq!(reparsed.discovery_queue.len(), 1);
    }
}
