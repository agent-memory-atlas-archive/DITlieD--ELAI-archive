//! E-54 -- eval-corpus repair pre-deploy dashboard surface.
//!
//! Three read-only Tauri commands feed the Svelte EvalCorpusPanel +
//! SycophancyMatchedPairPanel:
//!
//! - get_eval_corpus_status -- per-family task count + last decontam sweep
//!   result + cosine threshold + 13-gram threshold + KOR-orthogonality
//!   summary, computed from the most recent EvalCorpusEvent SSB rows.
//! - get_sycophancy_matched_pair_results -- recent matched-pair sycophancy
//!   probe results (eval-vs-deploy gap + sandbagging-anomaly flags) from
//!   EvalCorpusEvent rows with phase == "sycophancy_framing_gap_detected" /
//!   "sandbagging_anomaly".
//! - get_kor_orthogonality_status -- most recent KOR-orthogonality verdict
//!   per family pair from EvalCorpusEvent rows with phase ==
//!   "kor_orthogonality_violated" / "kor_orthogonality_passed".
//!
//! Additive -- sibling of commands_e10 (CriticDecisionPanel). The
//! orchestrator producer modules (eval-corpus crate) are the sole writers of
//! the underlying SSB rows; this module only reads.
//!
//! Wire-dark: the producer is gated by `ELAI_EVAL_CORPUS_ENABLE`; this
//! reader returns empty state when no rows have been written.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 200;
const MAX_LIMIT: usize = 2000;

/// Per-family task count snapshot for the eval-corpus panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FamilyCount {
    /// Canonical snake-case wire label (e.g. "code_fix").
    pub family: String,
    /// Cumulative `shard_created` count for this family.
    pub shard_created_count: u32,
    /// Cumulative `ingest_rejected_ngram + ingest_rejected_cosine` count.
    pub rejected_count: u32,
}

/// Aggregated panel response for EvalCorpusPanel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalCorpusStatus {
    /// True when the producer is wire-dark (no rows AT ALL). Lets the
    /// frontend render an explicit "(no data yet)" message instead of
    /// fabricated zeros.
    pub known: bool,
    /// Per-family aggregate counts.
    pub families: Vec<FamilyCount>,
    /// Cumulative ingest_rejected_ngram rows.
    pub ngram_rejected_count: u32,
    /// Cumulative ingest_rejected_cosine rows.
    pub cosine_rejected_count: u32,
    /// Cumulative schema-violation rows (no tests).
    pub schema_violation_count: u32,
    /// Cumulative test-fails-on-reference rows.
    pub test_fails_reference_count: u32,
    /// Cumulative test-useless-passes-broken rows.
    pub test_useless_count: u32,
    /// Cumulative generator-rotation-violated rows.
    pub generator_rotation_violation_count: u32,
    /// Cumulative perf-delta-contamination-suspected rows.
    pub perf_delta_count: u32,
    /// 13-gram threshold echoed from the eval-corpus crate constants
    /// (NGRAM_DEFAULT_N).
    pub ngram_n: u32,
    /// Cosine threshold echoed from the eval-corpus crate constants
    /// (COSINE_DEFAULT_THRESHOLD).
    pub cosine_threshold: f64,
    /// Total EvalCorpusEvent SSB rows scanned.
    pub grand_total: u32,
    /// Wall-clock ts of the most recent row.
    pub last_ts_ms: i64,
}

/// One matched-pair probe verdict for the sycophancy panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchedPairProbeView {
    pub task_id: String,
    pub family: String,
    pub phase: String,
    pub gap_pp: Option<f64>,
    pub sandbagging_anomaly: bool,
    pub ts_ms: i64,
}

/// Aggregated panel response for SycophancyMatchedPairPanel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SycophancyMatchedPairResults {
    pub known: bool,
    pub probes: Vec<MatchedPairProbeView>,
    pub framing_gap_count: u32,
    pub sandbagging_anomaly_count: u32,
    pub grand_total: u32,
}

/// One family-pair KOR-orthogonality verdict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KorPairVerdict {
    pub family_a: String,
    pub family_b: String,
    pub passed: bool,
    pub delta_pp: Option<f64>,
    pub ts_ms: i64,
}

/// Aggregated panel response for KOR-orthogonality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KorOrthogonalityStatus {
    pub known: bool,
    pub pairs: Vec<KorPairVerdict>,
    pub violated_count: u32,
    pub passed_count: u32,
    pub grand_total: u32,
    pub last_ts_ms: i64,
}

fn cap(limit: Option<usize>) -> usize {
    limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT)
}

/// Return the aggregate eval-corpus dashboard state. Reads up to `cap` recent
/// `eval_corpus_event` SSB rows.
#[tauri::command]
pub async fn get_eval_corpus_status(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<EvalCorpusStatus, String> {
    let cap_n = cap(limit);
    let mut out = EvalCorpusStatus {
        known: false,
        families: Vec::new(),
        ngram_rejected_count: 0,
        cosine_rejected_count: 0,
        schema_violation_count: 0,
        test_fails_reference_count: 0,
        test_useless_count: 0,
        generator_rotation_violation_count: 0,
        perf_delta_count: 0,
        // Echoed from eval-corpus crate constants.
        ngram_n: 13,
        cosine_threshold: 0.85,
        grand_total: 0,
        last_ts_ms: 0,
    };
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(out),
    };
    let rows = match viewer.recent_rows_of_type("eval_corpus_event", cap_n) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "get_eval_corpus_status: viewer error");
            return Ok(out);
        }
    };
    if rows.is_empty() {
        return Ok(out);
    }
    out.known = true;
    out.grand_total = rows.len() as u32;
    let mut family_index: std::collections::BTreeMap<String, FamilyCount> =
        std::collections::BTreeMap::new();
    for detail in rows {
        let inner = detail
            .payload_json
            .get("EvalCorpusEvent")
            .cloned()
            .unwrap_or(detail.payload_json);
        let phase = inner
            .get("phase")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let family_label = inner
            .get("family")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if detail.ts > out.last_ts_ms {
            out.last_ts_ms = detail.ts;
        }
        match phase.as_str() {
            "ingest_rejected_ngram" => out.ngram_rejected_count += 1,
            "ingest_rejected_cosine" => out.cosine_rejected_count += 1,
            "schema_violation_no_tests" => out.schema_violation_count += 1,
            "test_fails_on_reference" => out.test_fails_reference_count += 1,
            "test_useless_passes_broken" => out.test_useless_count += 1,
            "generator_rotation_violated" => out.generator_rotation_violation_count += 1,
            "perf_delta_contamination_suspected" => out.perf_delta_count += 1,
            _ => {}
        }
        if let Some(fam) = family_label {
            let entry = family_index
                .entry(fam.clone())
                .or_insert_with(|| FamilyCount {
                    family: fam,
                    shard_created_count: 0,
                    rejected_count: 0,
                });
            match phase.as_str() {
                "shard_created" => entry.shard_created_count += 1,
                "ingest_rejected_ngram" | "ingest_rejected_cosine" => {
                    entry.rejected_count += 1
                }
                _ => {}
            }
        }
    }
    out.families = family_index.into_values().collect();
    Ok(out)
}

/// Return the recent matched-pair sycophancy probe verdicts.
#[tauri::command]
pub async fn get_sycophancy_matched_pair_results(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<SycophancyMatchedPairResults, String> {
    let cap_n = cap(limit);
    let mut out = SycophancyMatchedPairResults {
        known: false,
        probes: Vec::new(),
        framing_gap_count: 0,
        sandbagging_anomaly_count: 0,
        grand_total: 0,
    };
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(out),
    };
    let rows = match viewer.recent_rows_of_type("eval_corpus_event", cap_n) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "get_sycophancy_matched_pair_results: viewer error");
            return Ok(out);
        }
    };
    if rows.is_empty() {
        return Ok(out);
    }
    out.known = true;
    out.grand_total = rows.len() as u32;
    for detail in rows {
        let inner = detail
            .payload_json
            .get("EvalCorpusEvent")
            .cloned()
            .unwrap_or(detail.payload_json);
        let phase = inner
            .get("phase")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if phase != "sycophancy_framing_gap_detected" && phase != "sandbagging_anomaly" {
            continue;
        }
        let task_id = inner
            .get("task_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let family = inner
            .get("family")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let details = inner.get("details_json").cloned().unwrap_or(Value::Null);
        let gap_pp = details.get("gap_pp").and_then(|v| v.as_f64());
        let sandbagging_anomaly = matches!(phase.as_str(), "sandbagging_anomaly");
        if phase == "sycophancy_framing_gap_detected" {
            out.framing_gap_count += 1;
        }
        if sandbagging_anomaly {
            out.sandbagging_anomaly_count += 1;
        }
        out.probes.push(MatchedPairProbeView {
            task_id,
            family,
            phase,
            gap_pp,
            sandbagging_anomaly,
            ts_ms: detail.ts,
        });
    }
    Ok(out)
}

/// Return the most recent KOR-orthogonality verdicts per (family_a, family_b)
/// pair.
#[tauri::command]
pub async fn get_kor_orthogonality_status(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<KorOrthogonalityStatus, String> {
    let cap_n = cap(limit);
    let mut out = KorOrthogonalityStatus {
        known: false,
        pairs: Vec::new(),
        violated_count: 0,
        passed_count: 0,
        grand_total: 0,
        last_ts_ms: 0,
    };
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(out),
    };
    let rows = match viewer.recent_rows_of_type("eval_corpus_event", cap_n) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "get_kor_orthogonality_status: viewer error");
            return Ok(out);
        }
    };
    if rows.is_empty() {
        return Ok(out);
    }
    out.known = true;
    out.grand_total = rows.len() as u32;
    let mut last_pair_seen: std::collections::BTreeMap<(String, String), KorPairVerdict> =
        std::collections::BTreeMap::new();
    for detail in rows {
        let inner = detail
            .payload_json
            .get("EvalCorpusEvent")
            .cloned()
            .unwrap_or(detail.payload_json);
        let phase = inner
            .get("phase")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if phase != "kor_orthogonality_violated" && phase != "kor_orthogonality_passed" {
            continue;
        }
        let details = inner.get("details_json").cloned().unwrap_or(json!({}));
        let family_a = details
            .get("family_a")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let family_b = details
            .get("family_b")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let delta_pp = details.get("delta_pp").and_then(|v| v.as_f64());
        let passed = phase == "kor_orthogonality_passed";
        if passed {
            out.passed_count += 1;
        } else {
            out.violated_count += 1;
        }
        if detail.ts > out.last_ts_ms {
            out.last_ts_ms = detail.ts;
        }
        let key = (family_a.clone(), family_b.clone());
        let existing = last_pair_seen.get(&key).map(|v| v.ts_ms).unwrap_or(0);
        if detail.ts >= existing {
            last_pair_seen.insert(
                key,
                KorPairVerdict {
                    family_a,
                    family_b,
                    passed,
                    delta_pp,
                    ts_ms: detail.ts,
                },
            );
        }
    }
    out.pairs = last_pair_seen.into_values().collect();
    Ok(out)
}
