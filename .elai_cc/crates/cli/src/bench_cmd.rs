// B-15 cli-regrowth tranche J (orchestrate s-22600) — bench / eval / critic
// dispatcher consolidation. Eleven CliAction arms that previously lived in
// `fn run()` at main.rs L853-1003 (BenchPrmbench, BenchModeRouting,
// BenchConformal, BenchArc, BenchRag, BenchSycophancy, BenchCanary,
// BenchTamper, BenchAblation, TierEval, CriticReport) are funneled here as
// thin per-arm wrappers. Each wrapper delegates to the underlying
// `<bench_module>::run(...)` callee with the same arg list — the
// underlying per-bench module's `pub(crate) fn run` is left untouched, so
// `tests/bench_prmbench_smoke.rs` and other smoke tests that call those
// fns directly continue to compile.
//
// Ship-shape: Option A from workflow/sessions/s-22600/context-package-trancheJ.md
//   - args::CliAction variant set UNCHANGED (each Bench* / TierEval /
//     CriticReport variant stays distinct; no enum re-grouping)
//   - SCHEMA_VERSION 195 UNCHANGED (no new SsbActionType)
//   - cross-language pair: NONE (pure Rust CLI dispatch)
//   - external callers of the underlying bench_*.rs `run` fns are
//     preserved verbatim (this wrapper layer adds one indirection only)
//
// Excluded from this tranche (per ctx package DECISIONS-5/6):
//   - FarmAdvRm (D-28 sibling boundary; inline FarmAdvRmArgs construction)
//   - DarwinLoop (feature-gated cfg both arms; leaks feature complexity)
//   - shortcoder_compress_text, cmd_incident_record, build_pr_review_action,
//     check_anthropic_gateway, CharEstimatorProvider (separate "main.rs
//     helpers" extraction tranche, deferred)
//
// Arg types mirror crate::args::CliAction variant declarations at
// args.rs L642-754 + L908/921/922 verbatim (floor/threshold fields are
// Option<String> because CliAction derives Eq and f64 does not).

pub(crate) fn run_bench_prmbench(
    dataset: Option<String>,
    hook: String,
    output: Option<String>,
    gating_threshold: Option<String>,
    category: Option<String>,
) -> anyhow::Result<()> {
    // D-19: PRMBench adversarial-gaming benchmark runner. Exits 0 on
    // Pass, 1 on Fail; emits one SsbActionType::PrmBenchRunCompleted row.
    crate::bench_prmbench::run(dataset, hook, output, gating_threshold, category)
}

pub(crate) fn run_bench_mode_routing(
    dataset: Option<String>,
    output: Option<String>,
    macro_floor: Option<String>,
    per_mode_floor: Option<String>,
) -> anyhow::Result<()> {
    // D-21: Replays the hand-labeled fixture through D-10's mode-routing
    // predicate. Exits 0 on Pass, 1 on Fail; emits one
    // SsbActionType::ModeRoutingBenchRunCompleted row.
    crate::bench_mode_routing::run(dataset, output, macro_floor, per_mode_floor)
}

pub(crate) fn run_bench_conformal(
    dataset: Option<String>,
    corpus: Option<String>,
    output: Option<String>,
    epsilon: Option<String>,
) -> anyhow::Result<()> {
    // D-22: Replays the validation fixture through D-16's `decide`
    // predicate per distribution bucket. Exits 0 on Pass, 1 on Fail;
    // emits one SsbActionType::ConformalValidationCompleted row.
    crate::bench_conformal::run(dataset, corpus, output, epsilon)
}

pub(crate) fn run_bench_arc(
    dataset: Option<String>,
    output: Option<String>,
    novelty_config: Option<String>,
) -> anyhow::Result<()> {
    // D-20: Replays an ARC-AGI-2 fixture through D-10's
    // `select_grounding_mode` predicate and D-15's `classify`. Exits 0
    // iff verdict is HonestlyEvaluated; emits one
    // SsbActionType::ArcBenchmarkCompleted row.
    crate::bench_arc::run(dataset, output, novelty_config)
}

pub(crate) fn run_bench_rag(
    tier: String,
    top_k: u32,
    query: String,
    json: bool,
    project_root: std::path::PathBuf,
) -> anyhow::Result<()> {
    // E-17 SP6: Builds the local knowledge pipeline via
    // LiveCli::build_local_knowledge_stack (same constructor production
    // uses), runs query_with_tier at the explicit tier, prints
    // final-stage results. Exits 0 on success.
    crate::bench_rag::run(tier, top_k, query, json, project_root)
}

pub(crate) fn run_bench_sycophancy(
    fixtures: Option<String>,
    output: Option<String>,
    detection_floor: Option<String>,
    fp_ceiling: Option<String>,
) -> anyhow::Result<()> {
    // D-23: Quantifies D-14 SycophancyProbe detection_rate + fp_rate
    // against the labelled fixture corpus. Exits 0 on Pass, 1 on Fail;
    // task_runner emits one SsbActionType::SycophancyBenchRunCompleted
    // row (the CLI does not persist SSB rows directly).
    crate::bench_sycophancy::run(fixtures, output, detection_floor, fp_ceiling)
}

pub(crate) fn run_bench_canary(
    fixtures: Option<String>,
    output: Option<String>,
    harmful_block_floor: Option<String>,
    helpful_block_ceiling: Option<String>,
    neutral_block_ceiling: Option<String>,
) -> anyhow::Result<()> {
    // D-24: Cross-domain canary regression benchmark. Replays a
    // labelled fixture corpus through D-7 run_canary in-memory; emits
    // exactly one SsbActionType::CanaryBenchRunCompleted row when
    // invoked from a task_runner. Exits 0 on Pass, 1 on Fail.
    crate::bench_canary::run(
        fixtures,
        output,
        harmful_block_floor,
        helpful_block_ceiling,
        neutral_block_ceiling,
    )
}

pub(crate) fn run_bench_tamper(
    scenarios: Option<String>,
    output: Option<String>,
    fail_fast: Option<String>,
    max_wall_ms_per_scenario: Option<String>,
) -> anyhow::Result<()> {
    // D-25: Determinism + archive tamper-suite benchmark driver. Drives
    // D-8 verify_archive IN-PROCESS across the 10-scenario matrix;
    // emits one SsbActionType::DeterminismTamperSuiteCompleted row when
    // invoked from a task_runner. Exits 0 on Pass, 1 on Fail.
    crate::bench_tamper::run(scenarios, output, fail_fast, max_wall_ms_per_scenario)
}

pub(crate) fn run_bench_seccodeplt(json: bool) -> anyhow::Result<()> {
    // F-2b: SeCodePLT-structured static-gate insecure-rate benchmark. Drives
    // the REAL evaluate_vibegate over the embedded CWE corpus OFFLINE, writes
    // a timestamped seccodeplt-<ts>.json report under benchmarks/seccodeplt/,
    // and emits exactly one SsbActionType::InsecureRateMeasured row (W-2:
    // always-on bench beacon; no ELAI_*_ENABLE gate). Exits 0 on a clean run,
    // 1 on infrastructural error.
    crate::bench_seccodeplt::run(json)
}

pub(crate) fn run_bench_ablation(
    tile: String,
    fixtures: String,
    budget: u32,
) -> anyhow::Result<()> {
    // E-10: hostile-critic structural simplifications dispatch.
    crate::bench_ablation_cmd::run(tile, fixtures, budget)
}

pub(crate) fn run_tier_eval(
    suites: Vec<String>,
    budget: u32,
) -> anyhow::Result<()> {
    // E-10 trio (2/3): tier-eval suite stub (RUNPOD-PENDING).
    crate::tier_eval_cmd::run(suites, budget)
}

pub(crate) fn run_critic_report(output: Option<String>) -> anyhow::Result<()> {
    // E-10 trio (3/3): hostile-critic decision report generator
    // (RUNPOD-PENDING).
    crate::critic_report::run(output)
}
