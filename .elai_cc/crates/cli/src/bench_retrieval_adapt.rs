//! J-72 slice D — CLI driver for `elai bench retrieval-adapt`.
//!
//! Measures ELAI gold-set MRR and Recall@k for a baseline slot vs an adapted
//! slot, then reports the delta. Deterministic. Never LLM-judge.
//!
//! ```text
//! elai bench retrieval-adapt --gold <path> [--k <N>]
//!   [--baseline-ranks <json>] [--adapted-ranks <json>]
//!   [--out <report.json>] [--json]
//! ```
//!
//! Ranks JSON shape: `{"triple-id": ["doc_a", "doc_b", ...], ...}`.
//!
//! When BOTH `--baseline-ranks` and `--adapted-ranks` are omitted, both slots
//! use the lexical-candidates scorer (delta ~0) so the CLI is runnable
//! hermetically without fixture ranks. When `--baseline-ranks` is present but
//! `--adapted-ranks` is omitted, the adapted slot is marked `missing`
//! (degrade, not crash): deltas = 0 - baseline.

use std::path::PathBuf;

use elai_knowledge::retrieval_adapt::{
    build_adapt_report, load_gold_jsonl_path, load_ranks_json, AdaptReport,
};

/// Run the `elai bench retrieval-adapt` command.
pub(crate) fn run(
    gold: PathBuf,
    k: u32,
    baseline_ranks: Option<PathBuf>,
    adapted_ranks: Option<PathBuf>,
    out: Option<PathBuf>,
    json: bool,
) -> anyhow::Result<()> {
    let k = k as usize;
    let triples = load_gold_jsonl_path(&gold)
        .map_err(|e| anyhow::anyhow!("failed to load gold set from {}: {e}", gold.display()))?;

    let baseline_map = match &baseline_ranks {
        Some(p) => Some(
            load_ranks_json(p)
                .map_err(|e| anyhow::anyhow!("failed to load baseline ranks from {}: {e}", p.display()))?,
        ),
        None => None,
    };
    let adapted_map = match &adapted_ranks {
        Some(p) => Some(
            load_ranks_json(p)
                .map_err(|e| anyhow::anyhow!("failed to load adapted ranks from {}: {e}", p.display()))?,
        ),
        None => None,
    };

    let gold_path_str = gold.to_string_lossy().to_string();
    let report = build_adapt_report(
        &gold_path_str,
        &triples,
        k,
        baseline_map.as_ref(),
        adapted_map.as_ref(),
    );

    let report_json = serde_json::to_string_pretty(&report)?;

    if let Some(out_path) = &out {
        std::fs::write(out_path, &report_json)
            .map_err(|e| anyhow::anyhow!("failed to write report to {}: {e}", out_path.display()))?;
    }

    if json {
        println!("{}", report_json);
    } else {
        print_human_summary(&report);
    }

    Ok(())
}

fn print_human_summary(report: &AdaptReport) {
    let adapted_label = if report.adapted_missing {
        "MISSING".to_string()
    } else {
        format!("{:.4}", report.adapted.mrr)
    };
    println!(
        "retrieval-adapt: n={} k={} scorer={} | baseline MRR={:.4} R@{}={:.4} | adapted MRR={} R@{}={:.4} | ΔMRR={:+.4} ΔR@{}={:+.4}{}",
        report.n_triples,
        report.k,
        report.scorer,
        report.baseline.mrr,
        report.k,
        report.baseline.recall_at_k,
        adapted_label,
        report.k,
        report.adapted.recall_at_k,
        report.delta_mrr,
        report.k,
        report.delta_recall_at_k,
        if report.adapted_missing { " [adapted MISSING]" } else { "" },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_knowledge::gold_set::{write_gold_jsonl, GoldTriple};

    fn triple(id: &str, query: &str, positive: &str, negs: &[&str]) -> GoldTriple {
        GoldTriple {
            id: id.to_string(),
            query: query.to_string(),
            positive: positive.to_string(),
            hard_negatives: negs.iter().map(|s| s.to_string()).collect(),
            source: "test".to_string(),
        }
    }

    #[test]
    fn run_no_ranks_lexical_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let gold_path = dir.path().join("gold.jsonl");
        let triples = vec![
            triple("q1", "query one", "doc_pos_a", &["doc_neg_a"]),
            triple("q2", "query two", "doc_pos_b", &["doc_neg_b"]),
        ];
        write_gold_jsonl(&gold_path, &triples).unwrap();

        let out_path = dir.path().join("report.json");
        run(gold_path, 10, None, None, Some(out_path.clone()), false).unwrap();

        let content = std::fs::read_to_string(&out_path).unwrap();
        let report: AdaptReport = serde_json::from_str(&content).unwrap();
        assert_eq!(report.n_triples, 2);
        assert_eq!(report.scorer, "lexical-candidates");
        assert!(!report.adapted_missing);
    }

    #[test]
    fn run_fixture_ranks_positive_delta() {
        let dir = tempfile::tempdir().unwrap();
        let gold_path = dir.path().join("gold.jsonl");
        let triples = vec![triple("q1", "query one", "doc_pos", &["doc_neg"])];
        write_gold_jsonl(&gold_path, &triples).unwrap();

        let baseline_path = dir.path().join("baseline.json");
        std::fs::write(
            &baseline_path,
            r#"{"q1":["doc_neg","doc_pos"]}"#,
        )
        .unwrap();

        let adapted_path = dir.path().join("adapted.json");
        std::fs::write(
            &adapted_path,
            r#"{"q1":["doc_pos","doc_neg"]}"#,
        )
        .unwrap();

        run(
            gold_path,
            10,
            Some(baseline_path.clone()),
            Some(adapted_path.clone()),
            None,
            true,
        )
        .unwrap();

        // Verify the builder produces a positive delta with fixture ranks.
        let bl = elai_knowledge::retrieval_adapt::load_ranks_json(&baseline_path).unwrap();
        let ad = elai_knowledge::retrieval_adapt::load_ranks_json(&adapted_path).unwrap();
        let report = build_adapt_report("gold.jsonl", &triples, 10, Some(&bl), Some(&ad));
        assert!(report.delta_mrr > 0.0, "delta_mrr={}", report.delta_mrr);
        assert_eq!(report.scorer, "fixture-ranks");
    }
}
