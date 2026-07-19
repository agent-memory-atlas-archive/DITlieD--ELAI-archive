//! I-15 (SCHEMA_VERSION 263): oracle cache-source discriminator SSB read
//! surface for the Svelte `ResearchDrawer` badge.
//!
//! One read-only command:
//!
//! - [`get_research_cache_source`] -- returns the cache layer discriminator
//!   for the most-recent `oracle_cache_hit` / `oracle_cache_miss` row whose
//!   `query_hash_hex` matches the argument. `"blake3_exact"` = in-session
//!   exact hit (zero ONNX calls); `"semantic_vector"` = full pipeline ran;
//!   `None` = no matching row yet (cold start) or legacy pre-I-15 row whose
//!   payload carried no `source` field.
//!
//! Default-ON: rows emit from `OracleAnswerCache::get` / `insert` on every
//! `/task` research-pipeline call (no ELAI_*_ENABLE gate).
//!
//! Mirror of commands_i38.rs::get_qdelta_latest pattern (single-row SSB
//! read, viewer-absent = Ok(None)).
//!
//! cross-lang-pair: dashboard/tauri/api

use tauri::State;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload};

use crate::state::AppState;

/// How many recent rows of each action_type to scan when looking for the
/// newest matching `query_hash_hex`. SSB rows arrive newest-first from
/// `recent_rows_of_type`; 50 is enough to cover a typical session's
/// distinct oracle queries without an unbounded scan.
const RECENT_SCAN_LIMIT: usize = 50;

/// Return the cache-source discriminator for the most-recent
/// `oracle_cache_hit` or `oracle_cache_miss` SSB row whose
/// `query_hash_hex` matches `query_hash_hex`.
///
/// Returns `Ok(None)` when:
/// - no viewer is attached,
/// - no matching row exists for the given hash (cold start), or
/// - the matching row is a legacy pre-I-15 row whose payload carried
///   `source: None`.
///
/// The returned string is exactly `"blake3_exact"` or `"semantic_vector"`;
/// any other (unexpected) value is treated as None rather than forwarded
/// to the UI, so the badge never shows an unrecognised label.
#[tauri::command]
pub async fn get_research_cache_source(
    query_hash_hex: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(None),
    };

    let mut best: Option<(i64, String)> = None;

    for action_type in [
        SsbActionType::OracleCacheHit.as_sql_str(),
        SsbActionType::OracleCacheMiss.as_sql_str(),
    ] {
        let raw = match viewer.recent_rows_of_type(action_type, RECENT_SCAN_LIMIT) {
            Ok(rows) => rows,
            Err(e) => return Err(e.to_string()),
        };
        for entry in raw {
            if entry.ts <= best.as_ref().map(|(ts, _)| *ts).unwrap_or(i64::MIN) {
                continue;
            }
            let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(s) = extract_source_for_hash(&pl, &query_hash_hex) {
                best = Some((entry.ts, s));
            }
        }
    }

    Ok(best.map(|(_, s)| s))
}

/// Project a deserialised SSB payload to its cache-source discriminator
/// when it is an `OracleCacheHit` / `OracleCacheMiss` whose
/// `query_hash_hex` matches `query_hash_hex`. Returns `None` for any
/// other payload, a hash mismatch, a legacy `source: None` row, or an
/// unrecognised source string.
fn extract_source_for_hash(pl: &SsbPayload, query_hash_hex: &str) -> Option<String> {
    let (hash, source) = match pl {
        SsbPayload::OracleCacheHit(p) => (p.query_hash_hex.as_str(), p.source.as_ref()),
        SsbPayload::OracleCacheMiss(p) => (p.query_hash_hex.as_str(), p.source.as_ref()),
        _ => return None,
    };
    if hash != query_hash_hex {
        return None;
    }
    match source {
        Some(s) if s == "blake3_exact" || s == "semantic_vector" => Some(s.clone()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::ssb::{OracleCacheHitPayload, OracleCacheMissPayload};

    #[test]
    fn hit_payload_matching_hash_returns_source() {
        let pl = SsbPayload::OracleCacheHit(OracleCacheHitPayload {
            query_hash_hex: "abc123".to_string(),
            source_tier: "t1_authoritative".to_string(),
            cached_age_secs: 0,
            source: Some("blake3_exact".to_string()),
        });
        assert_eq!(
            extract_source_for_hash(&pl, "abc123"),
            Some("blake3_exact".to_string())
        );
    }

    #[test]
    fn miss_payload_matching_hash_returns_semantic_source() {
        let pl = SsbPayload::OracleCacheMiss(OracleCacheMissPayload {
            query_hash_hex: "deadbeef".to_string(),
            confidence: "high".to_string(),
            source: Some("semantic_vector".to_string()),
        });
        assert_eq!(
            extract_source_for_hash(&pl, "deadbeef"),
            Some("semantic_vector".to_string())
        );
    }

    #[test]
    fn hash_mismatch_returns_none() {
        let pl = SsbPayload::OracleCacheHit(OracleCacheHitPayload {
            query_hash_hex: "abc123".to_string(),
            source_tier: "t1_authoritative".to_string(),
            cached_age_secs: 0,
            source: Some("blake3_exact".to_string()),
        });
        assert_eq!(extract_source_for_hash(&pl, "wrong"), None);
    }

    #[test]
    fn legacy_none_source_returns_none() {
        let pl = SsbPayload::OracleCacheHit(OracleCacheHitPayload {
            query_hash_hex: "abc123".to_string(),
            source_tier: "t1_authoritative".to_string(),
            cached_age_secs: 0,
            source: None,
        });
        assert_eq!(extract_source_for_hash(&pl, "abc123"), None);
    }

    #[test]
    fn unrecognised_source_string_returns_none() {
        let pl = SsbPayload::OracleCacheHit(OracleCacheHitPayload {
            query_hash_hex: "abc123".to_string(),
            source_tier: "t1_authoritative".to_string(),
            cached_age_secs: 0,
            source: Some("unknown_layer".to_string()),
        });
        assert_eq!(extract_source_for_hash(&pl, "abc123"), None);
    }

    #[test]
    fn unrelated_payload_variant_returns_none() {
        let pl = SsbPayload::QdeltaMeasured(elai_orchestrator::ssb::QdeltaMeasuredPayload {
            suite: "s".to_string(),
            model_id: "m".to_string(),
            n_tasks: 0,
            seed_count: 0,
            paired: false,
            on_defective_rate: 0.0,
            off_defective_rate: 0.0,
            delta: 0.0,
            ci_low: 0.0,
            ci_high: 0.0,
            dollar_per_task: 0.0,
            contamination_probe_result: "not_run".to_string(),
            surviving_mutants_per_task: 0.0,
            corpus_blake3: "x".to_string(),
            runpod_pending: false,
        });
        assert_eq!(extract_source_for_hash(&pl, "abc123"), None);
    }
}
