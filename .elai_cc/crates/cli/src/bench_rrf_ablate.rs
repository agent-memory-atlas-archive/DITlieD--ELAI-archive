//! J-75 — deterministic, measure-first RRF component ablation harness.
use std::collections::HashMap;
use std::path::PathBuf;

use elai_knowledge::retrieval_adapt::{load_gold_jsonl_path, load_ranks_json};
use elai_knowledge::rrf_ablation::{build_rrf_ablation_report, RrfAblationReport};

pub(crate) fn run(
    gold: PathBuf,
    k: u32,
    full_ranks: Option<PathBuf>,
    drop_pagerank_ranks: Option<PathBuf>,
    drop_second_dense_ranks: Option<PathBuf>,
    nav_only_graph_ranks: Option<PathBuf>,
    out: Option<PathBuf>,
    json: bool,
) -> anyhow::Result<()> {
    let triples = load_gold_jsonl_path(&gold)
        .map_err(|e| anyhow::anyhow!("failed to load gold set from {}: {e}", gold.display()))?;
    let mut ranks_by_config: HashMap<String, Option<HashMap<String, Vec<String>>>> = HashMap::new();
    for (name, path) in [
        ("full", full_ranks),
        ("drop-pagerank", drop_pagerank_ranks),
        ("drop-second-dense", drop_second_dense_ranks),
        ("nav-only-graph", nav_only_graph_ranks),
    ] {
        let ranks = match path {
            Some(path) => Some(load_ranks_json(&path).map_err(|e| anyhow::anyhow!("failed to load {name} ranks from {}: {e}", path.display()))?),
            None => None,
        };
        ranks_by_config.insert(name.to_string(), ranks);
    }
    let report = build_rrf_ablation_report(&gold.to_string_lossy(), &triples, k as usize, &ranks_by_config);
    let report_json = serde_json::to_string_pretty(&report)?;
    if let Some(path) = out { std::fs::write(&path, &report_json).map_err(|e| anyhow::anyhow!("failed to write report to {}: {e}", path.display()))?; }
    if json { println!("{report_json}"); } else { print_human_summary(&report); }
    Ok(())
}

fn print_human_summary(report: &RrfAblationReport) {
    println!("rrf-ablate: k={} full MRR={:.4} R@{}={:.4} guard={}", report.k, report.full.mrr, report.k, report.full.recall_at_k, report.measure_first_guard);
    for (config, delta) in &report.delta_mrr { println!("  {config}: ΔMRR={delta:+.4}"); }
}
