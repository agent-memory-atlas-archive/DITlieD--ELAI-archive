//! E-5 — MemoryFacts Tauri command surface.
//!
//! Read-only commands feed the `MemoryFactsDrawer.svelte` panel:
//!
//! - [`get_memory_facts`] — paginated listing of currently-valid rows.
//! - [`get_trust_score`] — single row trust score for hover-card.
//! - [`get_semantic_contradictions`] — F-12 semantic contradiction pairs.
//!
//! H-33 (SCHEMA_VERSION 211) adds three mutation commands:
//! - [`memory_forget`] — soft-close + MemoryGovernanceEvent::Forgotten beacon.
//! - [`memory_pin`] — MemoryGovernanceEvent::Pinned or ::Unpinned beacon.
//! - [`get_memory_facts_with_history`] — list_current + recently-superseded rows.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::paths::memory_facts_db_path;
use elai_memory::{FactStore, FactType, Scope};
use elai_orchestrator::ssb::{
    MemoryGovernanceEventPayload, MemoryGovernanceKind, SsbPayload,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 500;

/// Serializable mirror of [`elai_memory::MemoryFact`]. Stable shape for
/// the Svelte frontend — the in-Rust crate type is `pub` but lives in a
/// different crate so we mirror it here to keep the TS contract narrow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFactDto {
    pub fact_id: String,
    pub user_id: String,
    pub project_id: Option<String>,
    pub scope: String,
    pub fact_type: String,
    pub key: String,
    pub value: serde_json::Value,
    pub confidence: f32,
    pub trust_score: f32,
    pub trust_threshold: f32,
    pub ingested_at_ms: u64,
    pub valid_from_ms: u64,
    pub valid_to_ms: Option<u64>,
    pub decay_rate_per_day: f32,
    pub last_observed_at_ms: u64,
    pub evidence_pointers: Vec<String>,
    pub superseded_by: Option<String>,
}

impl From<elai_memory::MemoryFact> for MemoryFactDto {
    fn from(f: elai_memory::MemoryFact) -> Self {
        let trust_threshold = f.fact_type.retrieval_threshold();
        MemoryFactDto {
            fact_id: f.fact_id,
            user_id: f.user_id,
            project_id: f.project_id,
            scope: f.scope.as_str().to_owned(),
            fact_type: f.fact_type.as_str().to_owned(),
            key: f.key,
            value: f.value,
            confidence: f.confidence,
            trust_score: f.trust_score,
            trust_threshold,
            ingested_at_ms: f.ingested_at_ms as u64,
            valid_from_ms: f.valid_from_ms as u64,
            valid_to_ms: f.valid_to_ms.map(|v| v as u64),
            decay_rate_per_day: f.decay_rate_per_day,
            last_observed_at_ms: f.last_observed_at_ms as u64,
            evidence_pointers: f.evidence_pointers,
            superseded_by: f.superseded_by,
        }
    }
}

/// Result shape returned by [`get_memory_facts`].
#[derive(Debug, Clone, Serialize)]
pub struct MemoryFactsResponse {
    /// On-disk store path (absolute). Surfaced so the drawer can render
    /// "store: <path>" — also lets the UI distinguish "store missing"
    /// (drawer shows empty state) from "store exists, no rows".
    pub store_path: String,
    /// `true` when the store file did not exist at command time (lazy
    /// open creates it; the response then includes the empty `facts`
    /// list).
    pub store_just_created: bool,
    /// Currently-valid facts, ordered most-recently-observed first.
    pub facts: Vec<MemoryFactDto>,
}

/// Trust-score read shape — mirrors `MemoryFactDto` but with only the
/// columns the hover-card needs.
#[derive(Debug, Clone, Serialize)]
pub struct TrustScoreResponse {
    pub fact_id: String,
    pub trust_score: f32,
    pub trust_threshold: f32,
    pub decay_rate_per_day: f32,
    pub last_observed_at_ms: u64,
    pub valid_to_ms: Option<u64>,
}

fn resolve_store_path(state: &AppState) -> PathBuf {
    memory_facts_db_path(&state.project_root_snapshot())
}

fn open_store(state: &AppState) -> Result<(FactStore, PathBuf, bool), String> {
    let path = resolve_store_path(state);
    let already_existed = path.exists();
    let store = FactStore::open(&path).map_err(|e| e.to_string())?;
    Ok((store, path, !already_existed))
}

fn parse_optional_scope(raw: Option<String>) -> Result<Option<Scope>, String> {
    match raw.as_deref() {
        None => Ok(None),
        Some("") | Some("all") | Some("any") => Ok(None),
        Some(other) => Scope::from_str(other).map(Some).map_err(|e| e.to_string()),
    }
}

fn parse_optional_fact_type(raw: Option<String>) -> Result<Option<FactType>, String> {
    match raw.as_deref() {
        None => Ok(None),
        Some("") | Some("all") | Some("any") => Ok(None),
        Some(other) => FactType::from_str(other)
            .map(Some)
            .map_err(|e| e.to_string()),
    }
}

/// Tauri command — paginated listing of currently-valid memory_facts
/// rows for the active user.
#[tauri::command]
pub async fn get_memory_facts(
    state: State<'_, AppState>,
    user_id: Option<String>,
    scope: Option<String>,
    fact_type: Option<String>,
    limit: Option<usize>,
) -> Result<MemoryFactsResponse, String> {
    let user = user_id.unwrap_or_else(|| "default".to_owned());
    let scope = parse_optional_scope(scope)?;
    let fact_type = parse_optional_fact_type(fact_type)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT).max(1);

    let (store, path, store_just_created) = open_store(&state)?;
    let rows = store
        .list_current(&user, scope, fact_type, limit)
        .map_err(|e| e.to_string())?;
    Ok(MemoryFactsResponse {
        store_path: path.to_string_lossy().into_owned(),
        store_just_created,
        facts: rows.into_iter().map(MemoryFactDto::from).collect(),
    })
}

/// Tauri command — fetch the trust score for a single fact (drawer
/// hover-card path).
#[tauri::command]
pub async fn get_trust_score(
    state: State<'_, AppState>,
    fact_id: String,
) -> Result<TrustScoreResponse, String> {
    let (store, _path, _just) = open_store(&state)?;
    let fact = store
        .get(&fact_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("fact_id not found: {fact_id}"))?;
    let trust_threshold = fact.fact_type.retrieval_threshold();
    Ok(TrustScoreResponse {
        fact_id: fact.fact_id,
        trust_score: fact.trust_score,
        trust_threshold,
        decay_rate_per_day: fact.decay_rate_per_day,
        last_observed_at_ms: fact.last_observed_at_ms as u64,
        valid_to_ms: fact.valid_to_ms.map(|v| v as u64),
    })
}

// ---------------------------------------------------------------------------
// F-12 — semantic contradiction read surface.
//
// `get_semantic_contradictions` reads the SSB VIEWER (NOT the FactStore) for
// `memory_conflict_detected` rows filtered to
// `conflict_kind == "semantic_contradiction"` — the rows the live memory
// tick's deterministic-NLI second pass emits. Mirrors the
// `gateway_authz_recent_denies` (commands_c42) viewer-read shape: empty Vec
// when no viewer is attached (browser-dev / no session DB), defensive
// per-row decode that skips (never panics) on a shape mismatch.
// ---------------------------------------------------------------------------

const SEMANTIC_CONTRADICTION_KIND: &str = "semantic_contradiction";
const MEMORY_CONFLICT_ACTION_TYPE: &str = "memory_conflict_detected";

/// One semantic-contradiction pair surfaced to the drawer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticContradictionDto {
    /// Older row id (`fact_id_a`).
    pub fact_id_a: String,
    /// Newer row id (`fact_id_b`).
    pub fact_id_b: String,
    /// Free-form conflict kind (always "semantic_contradiction" here).
    pub conflict_kind: String,
    /// SSB row sequence (monotone; newest first in the response).
    pub seq: i64,
    /// SSB row timestamp (ms).
    pub ts_ms: i64,
}

/// Response shape for [`get_semantic_contradictions`].
#[derive(Debug, Clone, Serialize)]
pub struct SemanticContradictionsResponse {
    /// Whether an SSB viewer was attached (false in browser-dev / no DB).
    pub viewer_attached: bool,
    /// The semantic-contradiction pairs, newest first.
    pub pairs: Vec<SemanticContradictionDto>,
}

/// Extract the `MemoryConflictDetectedPayload` object from an
/// `SsbActionDetail.payload_json`, tolerating both the externally-tagged
/// shape (`{"MemoryConflictDetected": {...}}`) and a flat shape (`{...}`).
fn conflict_payload_object(
    payload_json: &serde_json::Value,
) -> Option<&serde_json::Map<String, serde_json::Value>> {
    let obj = payload_json.as_object()?;
    if let Some(inner) = obj.get("MemoryConflictDetected").and_then(|v| v.as_object()) {
        return Some(inner);
    }
    // Flat shape fallback: the object already carries the payload fields.
    if obj.contains_key("conflict_kind") {
        return Some(obj);
    }
    None
}

fn detail_to_semantic_pair(
    d: elai_orchestrator::SsbActionDetail,
) -> Option<SemanticContradictionDto> {
    let obj = conflict_payload_object(&d.payload_json)?;
    let conflict_kind = obj.get("conflict_kind")?.as_str()?.to_owned();
    if conflict_kind != SEMANTIC_CONTRADICTION_KIND {
        return None;
    }
    let fact_id_a = obj.get("fact_id_a")?.as_str()?.to_owned();
    let fact_id_b = obj.get("fact_id_b")?.as_str()?.to_owned();
    Some(SemanticContradictionDto {
        fact_id_a,
        fact_id_b,
        conflict_kind,
        seq: d.seq,
        ts_ms: d.ts,
    })
}

/// Tauri command — recent `memory_conflict_detected` SSB rows whose
/// `conflict_kind == "semantic_contradiction"`, newest first.
#[tauri::command]
pub async fn get_semantic_contradictions(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<SemanticContradictionsResponse, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT).max(1);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(SemanticContradictionsResponse {
                viewer_attached: false,
                pairs: Vec::new(),
            })
        }
    };
    let pairs: Vec<SemanticContradictionDto> =
        match viewer.recent_rows_of_type(MEMORY_CONFLICT_ACTION_TYPE, limit) {
            Ok(details) => details
                .into_iter()
                .filter_map(detail_to_semantic_pair)
                .collect(),
            Err(e) => {
                tracing::warn!(
                    action_type = MEMORY_CONFLICT_ACTION_TYPE,
                    error = %e,
                    "get_semantic_contradictions: viewer error"
                );
                Vec::new()
            }
        };
    Ok(SemanticContradictionsResponse {
        viewer_attached: true,
        pairs,
    })
}

// ---------------------------------------------------------------------------
// H-33 (SCHEMA_VERSION 211): governance mutation commands + history surface.
// ---------------------------------------------------------------------------

const H33_ACTOR: &str = "dashboard";
const H33_SESSION_ID: &str = "dashboard:memory-governance";

/// Response for [`memory_forget`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgetResponse {
    pub fact_id: String,
    pub soft_closed_at_ms: u64,
    /// SSB row sequence, or -1 when SSB writer was unavailable.
    pub ssb_seq: i64,
}

/// Response for [`memory_pin`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinResponse {
    pub fact_id: String,
    pub pinned: bool,
    /// SSB row sequence, or -1 when SSB writer was unavailable.
    pub ssb_seq: i64,
}

/// Response for [`get_memory_facts_with_history`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFactsHistoryResponse {
    pub store_path: String,
    /// Currently-valid facts.
    pub facts: Vec<MemoryFactDto>,
    /// Recently-superseded rows (superseded_by IS NOT NULL among active set).
    pub history_facts: Vec<MemoryFactDto>,
}

/// One governance event (DTO for the frontend history view).
/// H-33: used by future `get_governance_history` command (currently unused directly).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct MemoryGovernanceDto {
    pub fact_id: String,
    /// "forgotten" | "pinned" | "unpinned"
    pub kind: String,
    pub actor: String,
    pub ts_ms: u64,
}

/// H-33: soft-close a fact and emit `MemoryGovernanceEvent::Forgotten` beacon.
///
/// The FactStore soft_close succeeds independently of SSB availability — the
/// mutation is authoritative. SSB unavailability produces a warning in the
/// response (`ssb_seq == -1`) but is NOT an error.
#[tauri::command]
pub async fn memory_forget(
    state: State<'_, AppState>,
    fact_id: String,
) -> Result<ForgetResponse, String> {
    use elai_core::clock::OsClock;
    use elai_core::clock::ClockProvider;

    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let now_ms = clock.now_ms() as u64;

    let path = resolve_store_path(&state);
    let mut store = FactStore::open(&path).map_err(|e| e.to_string())?;
    store
        .soft_close(&fact_id, now_ms as u128)
        .map_err(|e| format!("soft_close failed: {e}"))?;

    let ssb_seq = match &state.writer {
        None => -1,
        Some(ssb) => {
            let payload = SsbPayload::MemoryGovernanceEvent(MemoryGovernanceEventPayload {
                fact_id: fact_id.clone(),
                kind: MemoryGovernanceKind::Forgotten,
                actor: H33_ACTOR.to_owned(),
                session_id: H33_SESSION_ID.to_owned(),
                created_at_ms: now_ms,
                seat_id: String::new(),
            });
            match ssb
                .write_payload(H33_ACTOR, 0, payload, vec![])
                .await
            {
                Ok(seq) => seq,
                Err(e) => {
                    tracing::warn!(error = %e, fact_id, "memory_forget: SSB beacon failed");
                    -1
                }
            }
        }
    };

    Ok(ForgetResponse {
        fact_id,
        soft_closed_at_ms: now_ms,
        ssb_seq,
    })
}

/// H-33: emit a pin or unpin `MemoryGovernanceEvent` beacon for a fact.
///
/// `pinned = true` emits `Pinned`; `pinned = false` emits `Unpinned`.
/// This does NOT modify the FactStore schema (no `pinned` column exists —
/// the pin state is derived from the SSB beacon stream by `build_pin_set_from_ssb`
/// in the decay path).
#[tauri::command]
pub async fn memory_pin(
    state: State<'_, AppState>,
    fact_id: String,
    pinned: bool,
) -> Result<PinResponse, String> {
    use elai_core::clock::OsClock;
    use elai_core::clock::ClockProvider;

    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let now_ms = clock.now_ms() as u64;

    let kind = if pinned {
        MemoryGovernanceKind::Pinned
    } else {
        MemoryGovernanceKind::Unpinned
    };

    let ssb_seq = match &state.writer {
        None => -1,
        Some(ssb) => {
            let payload = SsbPayload::MemoryGovernanceEvent(MemoryGovernanceEventPayload {
                fact_id: fact_id.clone(),
                kind,
                actor: H33_ACTOR.to_owned(),
                session_id: H33_SESSION_ID.to_owned(),
                created_at_ms: now_ms,
                seat_id: String::new(),
            });
            match ssb
                .write_payload(H33_ACTOR, 0, payload, vec![])
                .await
            {
                Ok(seq) => seq,
                Err(e) => {
                    tracing::warn!(error = %e, fact_id, "memory_pin: SSB beacon failed");
                    -1
                }
            }
        }
    };

    Ok(PinResponse {
        fact_id,
        pinned,
        ssb_seq,
    })
}

/// H-33: list currently-valid facts PLUS recently-superseded rows for the
/// active user. History facts are those with `superseded_by IS NOT NULL`
/// among the active set (soft-close means valid_to_ms is set so they do
/// not appear in list_current; the superseded_by chain is surfaced here
/// for the "show history" drawer toggle).
#[tauri::command]
pub async fn get_memory_facts_with_history(
    state: State<'_, AppState>,
    user_id: Option<String>,
    scope: Option<String>,
    fact_type: Option<String>,
    limit: Option<usize>,
) -> Result<MemoryFactsHistoryResponse, String> {
    let user = user_id.unwrap_or_else(|| "default".to_owned());
    let scope = parse_optional_scope(scope)?;
    let fact_type = parse_optional_fact_type(fact_type)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT).max(1);

    let (store, path, _) = open_store(&state)?;
    // list_including_closed returns ALL rows (valid + closed) for the user.
    // Post-filter by scope if provided (list_including_closed has no scope param).
    let all_rows = store
        .list_including_closed(&user, fact_type, limit)
        .map_err(|e| e.to_string())?;
    let rows: Vec<_> = if let Some(ref s) = scope {
        all_rows.into_iter().filter(|r| &r.scope == s).collect()
    } else {
        all_rows
    };

    // Split: closed/superseded rows (valid_to_ms IS NOT NULL) vs current.
    let mut facts: Vec<MemoryFactDto> = Vec::new();
    let mut history_facts: Vec<MemoryFactDto> = Vec::new();
    for row in rows {
        let dto = MemoryFactDto::from(row.clone());
        if row.valid_to_ms.is_some() {
            history_facts.push(dto);
        } else {
            facts.push(dto);
        }
    }

    Ok(MemoryFactsHistoryResponse {
        store_path: path.to_string_lossy().into_owned(),
        facts,
        history_facts,
    })
}
