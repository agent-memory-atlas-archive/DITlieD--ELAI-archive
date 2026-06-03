//! F-6 / Slice A5: learned-skill library (`SkillLibraryEvent`) SSB read surface
//! for the Svelte `SkillsPanel` Trust-Ladder section.
//!
//! cross-lang-skip: tauri_commands
//! (covered by the dedicated `skill_library_panel` pair, not the broad
//! `tauri_commands` / stub.ts wildcard.)
//!
//! One read-only command:
//!
//! - [`get_skill_library`] -- merges the last N rows of the three
//!   `SkillLibraryEvent` arms (`skill_acquired` / `skill_trust_promoted` /
//!   `skill_demoted_stale`) into a single newest-first timeline of
//!   [`SkillLibraryRow`], plus a derived per-skill latest-tier ladder snapshot
//!   ([`SkillLadderEntry`]) folded from the same rows.
//!
//! WHY DERIVE THE SNAPSHOT FROM SSB: the dashboard backend never holds a live
//! [`elai_orchestrator::skill_trust::SkillStore`] handle (the store is an
//! in-process orchestrator authority). Every `commands_f*` read surface in this
//! crate queries the SSB viewer instead, so the "store snapshot" the panel needs
//! is reconstructed by folding the per-skill events: the newest event for each
//! `skill_id` carries that skill's current tier (acquisition seeds S1, a
//! promotion/demotion carries the post-transition tier).
//!
//! NO `ELAI_*_ENABLE` default-off flag (F-6 W-2): the learned-skill library is
//! governed by the autonomy barrier (`ELAI_SKILL_AUTONOMY_BARRIER_DISABLE`
//! kill-switch on the producer side), not a wire-dark gate. This command always
//! queries the SSB viewer and returns whatever rows exist.
//!
//! Sibling of `commands_f10` (SecurityMonotonicityDrawer surface): one command
//! per distinct Svelte surface (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{
    SkillLibraryEventKind, SsbActionType, SsbPayload,
};

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// One `SkillLibraryEvent` row for the SkillsPanel Trust-Ladder timeline.
/// Field names are in exact sync with the TypeScript `SkillLibraryRow`
/// interface in `src/lib/api/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillLibraryRow {
    /// SSB monotonic sequence number (newest first ordering key).
    pub seq: i64,
    /// SSB row timestamp (epoch millis).
    pub ts: i64,
    /// Stable id of the learned skill.
    pub skill_id: String,
    /// Tier AFTER the event (snake-case wire label: `"s1"`..`"s4"`).
    pub tier: String,
    /// Tier BEFORE the event, when the event is a transition (promotion /
    /// demotion). `null` for an acquisition (no prior tier).
    pub prior_tier: Option<String>,
    /// Fire-point discriminator wire label: `"acquired"` / `"trust_promoted"`
    /// / `"demoted_stale"`.
    pub kind: String,
    /// Cumulative successful invocations at event time.
    pub invocations: u32,
    /// Failure rate (failures / total) at event time.
    pub failure_rate: f64,
}

/// One rung of the derived current-state Trust-Ladder. Field names are in exact
/// sync with the TypeScript `SkillLadderEntry` interface in
/// `src/lib/api/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillLadderEntry {
    /// Stable id of the learned skill.
    pub skill_id: String,
    /// Current tier (snake-case wire label) per the newest event for this skill.
    pub tier: String,
    /// Cumulative successful invocations at the newest event.
    pub invocations: u32,
    /// Failure rate at the newest event.
    pub failure_rate: f64,
    /// SSB row timestamp (epoch millis) of the newest event for this skill.
    pub ts: i64,
}

/// Response envelope for [`get_skill_library`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillLibraryResponse {
    /// Most recent `SkillLibraryEvent` rows across all three arms, newest first,
    /// capped at `limit`.
    pub events: Vec<SkillLibraryRow>,
    /// Derived per-skill current-tier snapshot (the Trust-Ladder), sorted by
    /// skill id. One entry per distinct `skill_id` seen in `events`, carrying
    /// the tier from that skill's newest event.
    pub ladder: Vec<SkillLadderEntry>,
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return the last `limit` `SkillLibraryEvent` rows from SSB (merged across the
/// `skill_acquired` / `skill_trust_promoted` / `skill_demoted_stale` arms),
/// newest first, plus a derived per-skill latest-tier ladder snapshot.
///
/// Returns an empty response only when the SSB viewer is not attached (fresh
/// install) or no skill-library rows have been written yet. The learned-skill
/// library has no wire-dark ENABLE gate (F-6 W-2); acquisition is governed by
/// the producer-side autonomy barrier, not a dashboard env flag.
#[tauri::command]
pub async fn get_skill_library(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<SkillLibraryResponse, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);

    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(SkillLibraryResponse {
                events: Vec::new(),
                ladder: Vec::new(),
            });
        }
    };

    // The three SkillLibraryEvent arms are distinct SSB action_type strings.
    // Query each, merge, then sort by seq descending so the timeline is a single
    // newest-first stream regardless of which arm produced each row.
    let arms = [
        SkillLibraryEventKind::Acquired,
        SkillLibraryEventKind::TrustPromoted,
        SkillLibraryEventKind::DemotedStale,
    ];

    let mut rows: Vec<SkillLibraryRow> = Vec::new();
    for kind in arms {
        let action = SsbActionType::SkillLibraryEvent(kind);
        let raw = viewer
            .recent_rows_of_type(action.as_sql_str(), cap)
            .map_err(|e| e.to_string())?;
        for entry in raw {
            let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let SsbPayload::SkillLibraryEvent(p) = pl {
                rows.push(SkillLibraryRow {
                    seq: entry.seq,
                    ts: entry.ts,
                    skill_id: p.skill_id.clone(),
                    tier: p.tier.clone(),
                    prior_tier: p.prior_tier.clone(),
                    kind: kind.as_sql_str().trim_start_matches("skill_").to_owned(),
                    invocations: p.invocations,
                    failure_rate: p.failure_rate,
                });
            }
        }
    }

    // Newest first across all three arms.
    rows.sort_by(|a, b| b.seq.cmp(&a.seq));
    rows.truncate(cap);

    // Derive the current-state ladder: the FIRST (newest) row seen for each
    // skill_id wins, since `rows` is already newest-first. BTreeMap keeps the
    // ladder output deterministically sorted by skill id (HashMap banned, B-15).
    let mut latest: BTreeMap<String, SkillLadderEntry> = BTreeMap::new();
    for row in &rows {
        latest.entry(row.skill_id.clone()).or_insert_with(|| SkillLadderEntry {
            skill_id: row.skill_id.clone(),
            tier: row.tier.clone(),
            invocations: row.invocations,
            failure_rate: row.failure_rate,
            ts: row.ts,
        });
    }
    let ladder: Vec<SkillLadderEntry> = latest.into_values().collect();

    Ok(SkillLibraryResponse {
        events: rows,
        ladder,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_library_row_serializes_expected_fields() {
        let row = SkillLibraryRow {
            seq: 9,
            ts: 1_700_000_000_003,
            skill_id: "scaffold-tauri".into(),
            tier: "s2".into(),
            prior_tier: Some("s1".into()),
            kind: "trust_promoted".into(),
            invocations: 4,
            failure_rate: 0.0,
        };
        let s = serde_json::to_string(&row).expect("serialize");
        for key in [
            "\"seq\"",
            "\"ts\"",
            "\"skill_id\"",
            "\"tier\"",
            "\"prior_tier\"",
            "\"kind\"",
            "\"invocations\"",
            "\"failure_rate\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn skill_ladder_entry_serializes_expected_fields() {
        let e = SkillLadderEntry {
            skill_id: "scaffold-tauri".into(),
            tier: "s4".into(),
            invocations: 12,
            failure_rate: 0.05,
            ts: 1_700_000_000_004,
        };
        let s = serde_json::to_string(&e).expect("serialize");
        for key in [
            "\"skill_id\"",
            "\"tier\"",
            "\"invocations\"",
            "\"failure_rate\"",
            "\"ts\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn skill_library_response_serializes_expected_fields() {
        let resp = SkillLibraryResponse {
            events: Vec::new(),
            ladder: Vec::new(),
        };
        let s = serde_json::to_string(&resp).expect("serialize");
        assert!(s.contains("\"events\""), "missing events in {s}");
        assert!(s.contains("\"ladder\""), "missing ladder in {s}");
    }

    #[test]
    fn kind_wire_label_strips_skill_prefix() {
        // The DTO kind field is the sql_str arm minus the "skill_" prefix, so the
        // TS side reads "acquired" / "trust_promoted" / "demoted_stale".
        assert_eq!(
            SkillLibraryEventKind::Acquired
                .as_sql_str()
                .trim_start_matches("skill_"),
            "acquired"
        );
        assert_eq!(
            SkillLibraryEventKind::TrustPromoted
                .as_sql_str()
                .trim_start_matches("skill_"),
            "trust_promoted"
        );
        assert_eq!(
            SkillLibraryEventKind::DemotedStale
                .as_sql_str()
                .trim_start_matches("skill_"),
            "demoted_stale"
        );
    }
}
