//! E-5 — MemoryFacts Tauri command surface.
//!
//! Two read-only commands feed the `MemoryFactsDrawer.svelte` panel:
//!
//! - [`get_memory_facts`] — paginated listing of currently-valid rows for
//!   the active user (+ optional `scope` / `fact_type` filter).
//! - [`get_trust_score`] — fetch a single row's persisted trust score
//!   for the drawer's "decay heat-map" hover-card.
//!
//! Per the executor brief, this is an ADDITIVE surface: a separate module
//! registered in `lib.rs::run` alongside the existing `commands::*`
//! handlers. No mutation commands ship in this slice (the Mem0
//! ADD/UPDATE/DELETE/NOOP dispatcher lives in the orchestrator's
//! follow-up vertical slice).

use std::path::PathBuf;

use elai_core::paths::memory_facts_db_path;
use elai_memory::{FactStore, FactType, Scope};
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
