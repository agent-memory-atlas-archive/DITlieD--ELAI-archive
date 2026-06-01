//! E-25 Tranche N (s-22600 2026-06-01) -- Compaction-Ladder dashboard
//! read surface. Pairs with the producer wire at
//! `task_runner.rs::drive_task` PreCompact branch + the SHIPPED
//! `compaction_ladder::run_compaction_ladder` runner.
//!
//! Two read-only Tauri commands feed the Svelte
//! `CompactionLadderPanel`:
//!
//! - [`get_compaction_ladder_state`] -- pure config snapshot. Reads the
//!   `ELAI_E25_COMPACTION_LADDER_ENABLE` env-gate + emits the LOCKED
//!   6-tier canonical traversal order + the default target budget. No
//!   SSB read; deterministic across calls.
//! - [`recent_compaction_events`] -- last N
//!   `ClaudeCodeArchitectureEvent` SSB rows filtered to
//!   `kind = compaction_ladder_tier_applied`, flattened into a per-row
//!   timeline DTO. Mirrors the `commands_c41::recent_skill_routing_events`
//!   pattern.
//!
//! Additive sibling of `commands_c41` / `commands_c43`. The orchestrator
//! producer (`task_runner` PreCompact branch, gated by
//! `ELAI_E25_COMPACTION_LADDER_ENABLE`) is the sole writer; this module
//! only reads. ZERO new SsbActionType / Kind arm / SCHEMA_VERSION bump
//! -- the wire rides the SHIPPED Pattern-2 single-variant cascade
//! (CompactionLadderTierApplied / CompactionLadder).
//!
//! See `workflow/sessions/s-22600/context-package-trancheN.md` for the
//! full scope + DECISIONS.

use elai_orchestrator::SsbActionDetail;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// SSB `action_type` string emitted by the orchestrator producer. Matches
/// `crate::ssb::SsbActionType::ClaudeCodeArchitectureEvent` at
/// `crates/orchestrator/src/ssb.rs:8618`.
const SSB_ACTION_TYPE: &str = "claude_code_architecture_event";

/// Wire label of the kind discriminator carried in the payload. Matches
/// `ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied::as_str()`
/// at `crates/orchestrator/src/architecture/claude_code_arch.rs:276`.
const KIND_COMPACTION_LADDER_TIER_APPLIED: &str = "compaction_ladder_tier_applied";

/// Outer-tagged serde variant string for `SsbPayload::ClaudeCodeArchitectureEvent`.
const VARIANT_CLAUDE_CODE_ARCH: &str = "ClaudeCodeArchitectureEvent";

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// LOCKED 6-tier canonical traversal order. Mirrors
/// `ALL_COMPACTION_TIERS` at
/// `crates/orchestrator/src/architecture/claude_code_arch.rs` + the
/// `CompactionTierId::as_snake_case` mapping.
const CANONICAL_TIERS: [&str; 6] = [
    "budget_reduction",
    "snip",
    "microcompact",
    "context_collapse",
    "auto_compact",
    "narrative_summary",
];

/// Default post-ladder target budget passed to
/// `CompactionLadder::default_order` from the PreCompact producer.
/// Matches `task_runner.rs::PRE_COMPACT_LADDER_TARGET_BUDGET`.
const DEFAULT_TARGET_TOKENS: u64 = 20_000;

/// Flat DTO returned by [`get_compaction_ladder_state`]. Pure config
/// snapshot -- the panel renders the wire-dark banner when
/// `enabled == false` and the tier list when `enabled == true`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionLadderStateDto {
    /// `true` iff `ELAI_E25_COMPACTION_LADDER_ENABLE == "1"`. Drives the
    /// wire-dark banner in the Svelte panel.
    pub enabled: bool,
    /// Default post-ladder target budget the producer passes to the
    /// runner. Surface-only; the panel renders this as the "target"
    /// column in the tier table.
    pub default_target_tokens: u64,
    /// LOCKED 6-tier traversal order (cheapest-first). The panel renders
    /// these as the tier rows; the recent-events table joins on
    /// `tier == tier`.
    pub tiers: Vec<String>,
}

/// Flat DTO for a single recorded compaction tier event. One row per
/// SSB `claude_code_architecture_event` write with
/// `kind = compaction_ladder_tier_applied`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactionEventDto {
    /// Producer-side wall-clock timestamp in ms since epoch. Source:
    /// `ClaudeCodeArchitectureEventPayload::ts_ms`.
    pub timestamp_ms: i64,
    /// Optional session id the producer attached. PreCompact wire uses
    /// `format!("task_{seq}")`; tests may pass arbitrary labels.
    pub session_id: Option<String>,
    /// Snake-case tier id (`budget_reduction` .. `narrative_summary`).
    pub tier: String,
    /// Stable ordinal 1..=6 from the LOCKED cheapest-first ordering.
    pub tier_ordinal: u8,
    /// Bytes the tier freed during this firing. `0` when the producer
    /// short-circuited; never observed in practice (the runner stops
    /// once budget falls below target).
    pub bytes_freed: u64,
    /// Pre-application token budget. Equals `budget_after` when the
    /// runner short-circuited early.
    pub budget_before: u64,
    /// Post-application token budget. Matches the runner's local
    /// `budget` after the tier subtraction.
    pub budget_after: u64,
    /// Configured `target_budget_tokens` the runner aims to drop below.
    pub target_budget: u64,
    /// Wall-clock duration of the tier in ms. Always >= 0; producer
    /// uses `OsClock::instant_now().elapsed()`.
    pub wall_ms: u64,
}

/// Read-only config snapshot. No SSB query; pure deterministic
/// projection of the env-gate + LOCKED tier order + default target.
///
/// Returns `Ok(...)` always -- there is no failure path (env reads
/// degrade to `false`, the tier list is `const`). The `state: State`
/// argument is accepted for signature parity with the other
/// dashboard read commands; no inner field is touched.
#[tauri::command]
pub async fn get_compaction_ladder_state(
    _state: State<'_, AppState>,
) -> Result<CompactionLadderStateDto, String> {
    let enabled = std::env::var("ELAI_E25_COMPACTION_LADDER_ENABLE")
        .map(|v| v == "1")
        .unwrap_or(false);
    Ok(CompactionLadderStateDto {
        enabled,
        default_target_tokens: DEFAULT_TARGET_TOKENS,
        tiers: CANONICAL_TIERS.iter().map(|s| (*s).to_string()).collect(),
    })
}

/// Return the most recent `limit` compaction-ladder tier events,
/// sorted by `timestamp_ms` descending. Filters SSB rows of
/// `action_type = claude_code_architecture_event` to those whose payload
/// variant kind equals `compaction_ladder_tier_applied` (the other 8
/// kinds in the 9-arm `ClaudeCodeArchitectureEventKind` enum belong to
/// sibling primitives and are skipped here).
///
/// Empty response when no SSB viewer is attached (dashboard launched
/// without a project, or `AppState.viewer` is `None`). Per-row
/// payload-decode failures are logged via `tracing::warn` and skipped.
#[tauri::command]
pub async fn recent_compaction_events(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<CompactionEventDto>, String> {
    let cap = limit
        .map(|n| n as usize)
        .unwrap_or(DEFAULT_LIMIT)
        .min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut events: Vec<CompactionEventDto> = Vec::new();
    match viewer.recent_rows_of_type(SSB_ACTION_TYPE, cap) {
        Ok(details) => {
            for d in details {
                if let Some(ev) = detail_to_compaction_dto(d) {
                    events.push(ev);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = SSB_ACTION_TYPE,
                error = %e,
                "recent_compaction_events: viewer error"
            );
        }
    }
    events.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
    events.truncate(cap);
    Ok(events)
}

/// Unwrap the outer serde-tag wrapper around a
/// `ClaudeCodeArchitectureEvent` payload. The payload is serialised
/// externally tagged (`{"ClaudeCodeArchitectureEvent": {..}}`); the
/// inner object carries `kind` / `primitive` / `session_id` /
/// `details_json` / `ts_ms`. Falls through to the outer map when no
/// wrapper present so the helper works against both tagged and
/// untagged shapes.
fn unwrap_variant<'a>(
    payload: &'a serde_json::Value,
    variant: &str,
) -> Option<&'a serde_json::Map<String, serde_json::Value>> {
    let outer = payload.as_object()?;
    if let Some(inner) = outer.get(variant).and_then(|v| v.as_object()) {
        return Some(inner);
    }
    Some(outer)
}

/// Map one [`SsbActionDetail`] row to a flat compaction-event DTO.
/// Returns `None` when the row is not a compaction-tier event (the SSB
/// `claude_code_architecture_event` action_type is shared across 9
/// kinds; we filter here).
fn detail_to_compaction_dto(d: SsbActionDetail) -> Option<CompactionEventDto> {
    if d.action_type.as_str() != SSB_ACTION_TYPE {
        return None;
    }
    let obj = unwrap_variant(&d.payload_json, VARIANT_CLAUDE_CODE_ARCH)?;
    let kind = obj.get("kind").and_then(|v| v.as_str())?;
    if kind != KIND_COMPACTION_LADDER_TIER_APPLIED {
        return None;
    }
    let ts_ms = obj
        .get("ts_ms")
        .and_then(|v| v.as_i64())
        .or_else(|| obj.get("ts_ms").and_then(|v| v.as_u64()).map(|u| u as i64))
        .unwrap_or(0);
    let session_id = obj
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let details = obj.get("details_json").and_then(|v| v.as_object());
    let tier = details
        .and_then(|m| m.get("tier"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tier_ordinal = details
        .and_then(|m| m.get("tier_ordinal"))
        .and_then(|v| v.as_u64())
        .map(|n| u8::try_from(n).unwrap_or(u8::MAX))
        .unwrap_or(0);
    let bytes_freed = details
        .and_then(|m| m.get("bytes_freed"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let budget_before = details
        .and_then(|m| m.get("budget_before"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let budget_after = details
        .and_then(|m| m.get("budget_after"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let target_budget = details
        .and_then(|m| m.get("target_budget"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let wall_ms = details
        .and_then(|m| m.get("wall_ms"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    Some(CompactionEventDto {
        timestamp_ms: ts_ms,
        session_id,
        tier,
        tier_ordinal,
        bytes_freed,
        budget_before,
        budget_after,
        target_budget,
        wall_ms,
    })
}
