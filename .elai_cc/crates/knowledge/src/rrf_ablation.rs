//! J-75 measure-first RRF component ablation reporting.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::gold_set::GoldTriple;
use crate::retrieval_adapt::{score_lexical_slot, score_ranked_lists};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AblationFlags {
    pub drop_pagerank: bool,
    pub drop_second_dense: bool,
    pub nav_only_graph: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetrics {
    pub config: String,
    pub mrr: f64,
    pub recall_at_k: f64,
    pub ranks_missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RrfAblationReport {
    pub gold_path: String,
    pub k: usize,
    pub full: ConfigMetrics,
    pub ablations: Vec<ConfigMetrics>,
    pub delta_mrr: Vec<(String, f64)>,
    pub delta_recall_at_k: Vec<(String, f64)>,
    pub measure_first_guard: String,
}

pub const ELAI_ABLATION_DROP_PAGERANK_ENV: &str = "ELAI_ABLATION_DROP_PAGERANK";
pub const ELAI_ABLATION_DROP_SECOND_DENSE_ENV: &str = "ELAI_ABLATION_DROP_SECOND_DENSE";
pub const ELAI_ABLATION_NAV_ONLY_GRAPH_ENV: &str = "ELAI_ABLATION_NAV_ONLY_GRAPH";

fn env_enabled(name: &str) -> bool {
    matches!(std::env::var(name).ok().as_deref(), Some(value) if matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
}

pub fn ablation_flags_from_env() -> AblationFlags {
    AblationFlags {
        drop_pagerank: env_enabled(ELAI_ABLATION_DROP_PAGERANK_ENV),
        drop_second_dense: env_enabled(ELAI_ABLATION_DROP_SECOND_DENSE_ENV),
        nav_only_graph: env_enabled(ELAI_ABLATION_NAV_ONLY_GRAPH_ENV),
    }
}

pub fn production_defaults_are_full_stack() -> bool {
    let flags = ablation_flags_from_env();
    !flags.drop_pagerank && !flags.drop_second_dense && !flags.nav_only_graph
}

const GUARD: &str = "C5 MEASURE-FIRST: produces numbers only; any stage removal requires measured delta AND explicit user sign-off. Cross-encoder is never a prune target.";

fn metrics(config: &str, triples: &[GoldTriple], k: usize, ranks: Option<&HashMap<String, Vec<String>>>) -> ConfigMetrics {
    let (mrr, recall_at_k) = match ranks {
        Some(ranks) => {
            let score = score_ranked_lists(triples, ranks, k, config);
            (score.mrr, score.recall_at_k)
        }
        None => {
            let score = score_lexical_slot(triples, k, config);
            (score.mrr, score.recall_at_k)
        }
    };
    ConfigMetrics { config: config.to_string(), mrr, recall_at_k, ranks_missing: ranks.is_none() }
}

pub fn build_rrf_ablation_report(
    gold_path: &str,
    triples: &[GoldTriple],
    k: usize,
    ranks_by_config: &HashMap<String, Option<HashMap<String, Vec<String>>>>,
) -> RrfAblationReport {
    let full = metrics("full", triples, k, ranks_by_config.get("full").and_then(Option::as_ref));
    let mut ablations = Vec::new();
    for config in ["drop-pagerank", "drop-second-dense", "nav-only-graph"] {
        if let Some(ranks) = ranks_by_config.get(config) {
            ablations.push(metrics(config, triples, k, ranks.as_ref()));
        }
    }
    let delta_mrr = ablations.iter().map(|m| (m.config.clone(), m.mrr - full.mrr)).collect();
    let delta_recall_at_k = ablations.iter().map(|m| (m.config.clone(), m.recall_at_k - full.recall_at_k)).collect();
    RrfAblationReport { gold_path: gold_path.to_string(), k, full, ablations, delta_mrr, delta_recall_at_k, measure_first_guard: GUARD.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_records_degraded_ablation() {
        let triple = GoldTriple { id: "q1".into(), query: "query".into(), positive: "good".into(), hard_negatives: vec!["bad".into()], source: "test".into() };
        let mut ranks = HashMap::new();
        ranks.insert("full".into(), Some(HashMap::from([("q1".into(), vec!["good".into(), "bad".into()])] )));
        ranks.insert("drop-pagerank".into(), Some(HashMap::from([("q1".into(), vec!["bad".into(), "good".into()])] )));
        let report = build_rrf_ablation_report("gold.jsonl", &[triple], 1, &ranks);
        assert!(report.delta_mrr[0].1 < 0.0);
    }
    #[test]
    fn guard_is_measure_first() { assert!(GUARD.contains("MEASURE-FIRST")); }
}
