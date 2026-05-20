//! D-10 — mode-routing predicate Tauri command surface.
//!
//! One read-only command feeds the Svelte `ModeRoutingPanel`:
//!
//! - [`get_recent_claim_routings`] — last N `claim_routed` SSB rows,
//!   sorted by `ts_ms` desc, plus an aggregated per-mode count
//!   histogram so the panel can render the distribution without
//!   re-summing on the client.
//!
//! Additive — sibling of `commands_d9` (locked-keys). The orchestrator
//! producer (`elai_orchestrator::route_claim` -> SSB `claim_routed`)
//! is the sole writer; this module only reads.
//!
//! See docs/thefinalglance/D-plan/D-10_mode_routing_predicate.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// Typed mirror of `elai_orchestrator::ClaimRoutedRow`. Lives in this
/// crate so the Tauri-side schema doesn't leak the orchestrator's
/// internal types through the IPC boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimRoutedRowDto {
    pub claim_id: String,
    pub claim_type: String,
    pub mode: String,
    pub ts_ms: i64,
    pub ctx_summary_blake3: String,
}

/// Aggregated response for the panel: recent rows + per-mode counts.
/// The counts cover the same `limit`-bounded window so the panel can
/// render `<mode>: N (X%)` without re-walking the rows array.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentClaimRoutings {
    pub rows: Vec<ClaimRoutedRowDto>,
    /// Snake-case mode label → count within `rows`. Sorted-key
    /// `BTreeMap` so the JSON output is deterministic for snapshot
    /// tests.
    pub counts_by_mode: BTreeMap<String, u32>,
    /// Total number of rows in `rows` (== `rows.len()`; redundant but
    /// keeps the panel's percentage math single-source).
    pub total: u32,
}

/// Return the most recent `claim_routed` rows (capped at `MAX_LIMIT`)
/// + per-mode count aggregation. Empty response when no SSB viewer is
/// attached (Tauri-dev fresh boot, no DB).
#[tauri::command]
pub async fn get_recent_claim_routings(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<RecentClaimRoutings, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(RecentClaimRoutings {
                rows: Vec::new(),
                counts_by_mode: BTreeMap::new(),
                total: 0,
            });
        }
    };

    let rows: Vec<ClaimRoutedRowDto> = elai_orchestrator::recent_claim_routings(&viewer, cap)
        .into_iter()
        .map(|r| ClaimRoutedRowDto {
            claim_id: r.claim_id,
            claim_type: r.claim_type,
            mode: r.mode,
            ts_ms: r.ts_ms,
            ctx_summary_blake3: r.ctx_summary_blake3,
        })
        .collect();

    let mut counts_by_mode: BTreeMap<String, u32> = BTreeMap::new();
    // Seed every known mode at zero so the panel histogram has a
    // stable column set even when a mode is empty.
    for mode in [
        "constructive",
        "falsifiability_cross_family",
        "conformal_refusal",
        "operational_deferred",
        "conservative_fallback",
    ] {
        counts_by_mode.insert(mode.to_string(), 0);
    }
    for row in &rows {
        *counts_by_mode.entry(row.mode.clone()).or_insert(0) += 1;
    }

    let total = u32::try_from(rows.len()).unwrap_or(u32::MAX);
    Ok(RecentClaimRoutings {
        rows,
        counts_by_mode,
        total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_serializes_with_expected_field_names() {
        let dto = ClaimRoutedRowDto {
            claim_id: "c-1".into(),
            claim_type: "factual".into(),
            mode: "constructive".into(),
            ts_ms: 1_700_000_000_000,
            ctx_summary_blake3: "0".repeat(64),
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        assert!(s.contains("\"claim_id\""));
        assert!(s.contains("\"claim_type\""));
        assert!(s.contains("\"mode\""));
        assert!(s.contains("\"ts_ms\""));
        assert!(s.contains("\"ctx_summary_blake3\""));
    }

    #[test]
    fn counts_by_mode_has_all_five_keys_when_seeded() {
        // Mirror the seed used inside get_recent_claim_routings — the
        // panel relies on every known mode appearing in counts_by_mode
        // (zeros are valid histogram columns).
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
        assert_eq!(counts.len(), 5);
    }
}
