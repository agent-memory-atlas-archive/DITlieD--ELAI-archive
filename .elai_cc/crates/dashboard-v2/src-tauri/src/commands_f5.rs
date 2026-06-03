//! F-5 / Slice A5: `StalenessQuarantineBroadcast` SSB read surface for the
//! Svelte `StalenessQuarantineDrawer` drawer.
//!
//! cross-lang-skip: tauri_commands
//! (covered by the dedicated `staleness_quarantine_drawer` pair, not the broad
//! `tauri_commands` / stub.ts wildcard).
//!
//! One read-only command:
//!
//! - [`get_staleness_quarantine`] -- last N `staleness_quarantine_broadcast`
//!   rows emitted at the orchestrator/CLI memory-decay-tick boundary when a
//!   decaying fact crosses the staleness threshold and the quarantine cascades
//!   to DKG `concept_stale=2` + a non-destructive `staleness_quarantined` marker
//!   edge + Cosmos artifact flagging.
//!
//! No wire-dark ENABLE gate for the SSB row itself (F-5 W-2): decay cadence and
//! the bounded `QuarantineBroadcastQueue` (capacity 256, DROP-OLDEST) are
//! ELAI-owned; the broadcast always writes its row, so this command always
//! queries the SSB viewer and returns whatever rows exist. The `dropped_count`
//! field surfaces queue overflow so the panel can show silent-drop pressure
//! (W-2: no silent drop).
//!
//! Sibling of `commands_f10` (SecurityMonotonicityDrawer surface) and
//! `commands_f9` (DepsSecurityPanel surface): one command per distinct Svelte
//! drawer key (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload};

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One `staleness_quarantine_broadcast` row for the StalenessQuarantineDrawer.
/// Field names are in exact sync with the TypeScript `StalenessQuarantineRow`
/// interface in `src/lib/api/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StalenessQuarantineRow {
    /// SSB monotonic sequence number (newest first ordering key).
    pub seq: i64,
    /// SSB row timestamp (epoch millis).
    pub ts: i64,
    /// Stable id of the quarantined fact.
    pub fact_id: String,
    /// Decay staleness ratio at quarantine time (the value that crossed the
    /// threshold; 0.0 fresh .. 1.0+ fully stale).
    pub staleness_ratio: f64,
    /// Count of DKG concepts demoted to `concept_stale=2` by this cascade.
    pub dependents_demoted: u32,
    /// Count of Cosmos artifacts flagged stale by this cascade.
    pub cosmos_artifacts_flagged: u32,
    /// Queue records DROPPED-OLDEST due to capacity overflow at emit time.
    pub dropped_count: u32,
}

/// Response envelope for [`get_staleness_quarantine`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StalenessQuarantineResponse {
    /// Most recent rows, newest first, capped at `DEFAULT_LIMIT`.
    pub rows: Vec<StalenessQuarantineRow>,
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return the last `limit` `staleness_quarantine_broadcast` rows from SSB,
/// newest first. Returns an empty Vec only when the SSB viewer is not attached
/// (fresh install) or no rows have been written yet.
///
/// The cascade has no wire-dark ENABLE gate (F-5 W-2): the broadcast writes its
/// row whenever a memory-decay tick quarantines a fact, so this command always
/// queries the SSB viewer.
#[tauri::command]
pub async fn get_staleness_quarantine(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<StalenessQuarantineResponse, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);

    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(StalenessQuarantineResponse { rows: Vec::new() });
        }
    };

    let raw = viewer
        .recent_rows_of_type(
            SsbActionType::StalenessQuarantineBroadcast.as_sql_str(),
            cap,
        )
        .map_err(|e| e.to_string())?;

    let mut rows: Vec<StalenessQuarantineRow> = Vec::new();
    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::StalenessQuarantineBroadcast(p) = pl {
            rows.push(StalenessQuarantineRow {
                seq: entry.seq,
                ts: entry.ts,
                fact_id: p.fact_id.clone(),
                staleness_ratio: p.staleness_ratio,
                dependents_demoted: p.dependents_demoted,
                cosmos_artifacts_flagged: p.cosmos_artifacts_flagged,
                dropped_count: p.dropped_count,
            });
        }
    }

    Ok(StalenessQuarantineResponse { rows })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staleness_quarantine_row_serializes_expected_fields() {
        let row = StalenessQuarantineRow {
            seq: 11,
            ts: 1_700_000_000_005,
            fact_id: "fact-abc123".into(),
            staleness_ratio: 1.25,
            dependents_demoted: 3,
            cosmos_artifacts_flagged: 2,
            dropped_count: 0,
        };
        let s = serde_json::to_string(&row).expect("serialize");
        for key in [
            "\"seq\"",
            "\"ts\"",
            "\"fact_id\"",
            "\"staleness_ratio\"",
            "\"dependents_demoted\"",
            "\"cosmos_artifacts_flagged\"",
            "\"dropped_count\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn staleness_quarantine_response_serializes_expected_fields() {
        let resp = StalenessQuarantineResponse { rows: Vec::new() };
        let s = serde_json::to_string(&resp).expect("serialize");
        assert!(s.contains("\"rows\""), "missing rows in {s}");
    }
}
