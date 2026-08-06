//! J-75 measure-first RRF ablation contract.
//!
//! These contracts call the production report and flag APIs directly.  The
//! environment is treated as read-only: tests do not mutate process
//! environment variables, avoiding races with concurrent tests.
//!
//! WIRED[J-75] production callers:
//! - `main.rs` `CliAction::BenchRrfAblate` ->
//!   `bench_cmd::run_bench_rrf_ablate`
//! - `pipeline.rs` `ablation_flags_from_env` at stage gates
//! - W-6 justified frontend absence (`ExternalBenchPanel` delta deferred;
//!   report is a CLI artifact).
//!
//! FALSIFICATION NOTE: deleting the `main.rs` `BenchRrfAblate` arm must fail
//! `production_caller_check` --symbols `run_bench_rrf_ablate` (or the
//! `build_rrf_ablation_report` path).  This contract intentionally does not
//! grep source for that check.

use std::collections::HashMap;

use elai_knowledge::gold_set::GoldTriple;
use elai_knowledge::rrf_ablation::{
    ablation_flags_from_env, build_rrf_ablation_report, production_defaults_are_full_stack,
    AblationFlags,
};

fn triple() -> GoldTriple {
    GoldTriple {
        id: "q1".into(),
        query: "query".into(),
        positive: "good".into(),
        hard_negatives: vec!["bad".into()],
        source: "j75-contract".into(),
    }
}

#[test]
fn j75_production_defaults_are_full_stack_when_env_is_unset() {
    // The test does not unset or set the environment: concurrent environment
    // mutation is not used.  The normal process environment is expected to
    // have no J-75 ablation flags.
    let flags: AblationFlags = ablation_flags_from_env();
    assert_eq!(flags, AblationFlags::default(), "J-75 flags must be unset");
    assert!(production_defaults_are_full_stack());
}

#[test]
fn j75_drop_pagerank_reports_negative_delta_against_full_ranks() {
    let mut ranks_by_config = HashMap::new();
    ranks_by_config.insert(
        "full".to_string(),
        Some(HashMap::from([(
            "q1".to_string(),
            vec!["good".to_string(), "bad".to_string()],
        )])),
    );
    ranks_by_config.insert(
        "drop-pagerank".to_string(),
        Some(HashMap::from([(
            "q1".to_string(),
            vec!["bad".to_string(), "good".to_string()],
        )])),
    );

    let report = build_rrf_ablation_report("gold.jsonl", &[triple()], 1, &ranks_by_config);
    let delta = report
        .delta_mrr
        .iter()
        .find(|(config, _)| config == "drop-pagerank")
        .map(|(_, delta)| *delta)
        .expect("drop-pagerank must have a delta");
    assert!(delta < 0.0, "drop-pagerank delta must be negative, got {delta}");
}

#[test]
fn j75_measure_first_guard_requires_signoff_language() {
    let report = build_rrf_ablation_report("gold.jsonl", &[triple()], 1, &HashMap::new());
    assert!(report.measure_first_guard.contains("MEASURE-FIRST"));
    assert!(
        report.measure_first_guard.contains("sign-off")
            || report.measure_first_guard.contains("user")
    );
}

#[test]
fn j75_missing_ranks_use_hermetic_lexical_path_and_mark_missing() {
    let mut ranks_by_config = HashMap::new();
    ranks_by_config.insert("full".to_string(), None);

    let report = build_rrf_ablation_report("gold.jsonl", &[triple()], 1, &ranks_by_config);
    assert!(report.full.ranks_missing);
}
