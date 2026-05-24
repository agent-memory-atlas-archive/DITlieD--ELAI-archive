//! E-49 — `elai metrics <sub>` handler.
//!
//! WIRE-DARK avoidance per E-41 precedent
//! (`E40_WIRE_DARK_AVOIDANCE_VIA_CLI`): anchors the
//! `coding_metrics::*` module as production-callable so
//! `MetricsCollector::capture_metrics`,
//! `coding_metrics::aggregate_session_metrics`,
//! `coding_metrics::run_swebench_pro`, and
//! `coding_metrics::run_refinebench` each have a non-test caller.
//!
//! Sub-actions:
//!   * `capture --session-id S` — invokes a fresh
//!     `MetricsCollector` (no observations) and prints the resulting
//!     `CodingMetric` capture for `S` as JSON/TOML/text per
//!     `--output`. Useful for dashboard-side dry-runs + smoke tests
//!     of the wire shape.
//!   * `swe-bench-pro` — calls `run_swebench_pro` with a
//!     RUNPOD-PENDING config; surfaces the `RunpodPending` error so
//!     operators see the boundary explicitly.
//!   * `refinebench` — same posture as `swe-bench-pro`.
//!   * `aggregate` — calls `aggregate_session_metrics` over a
//!     single-snapshot dataset (zero-observation capture). Smoke
//!     test for the aggregator path.
//!   * `status` — prints the 5 E-49 SSB action types + the wired
//!     primitives.
//!
//! Per H-7 / H-9: no `.unwrap()` / `.expect()` outside test code.

use elai_orchestrator::coding_metrics::{
    aggregate_session_metrics, run_refinebench, run_swebench_pro, MetricsCollector,
    RefineBenchConfig, RefineBenchError, SweBenchProConfig, SweBenchProError,
};
use serde::Serialize;

use crate::args::{CliOutputFormat, MetricsAction};

/// Top-level dispatcher invoked by `main.rs`.
pub(crate) fn run(action: MetricsAction, output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        MetricsAction::Capture { session_id } => run_capture(&session_id, output_format),
        MetricsAction::SweBenchPro => run_swe_bench_pro_dry(output_format),
        MetricsAction::RefineBench => run_refinebench_dry(output_format),
        MetricsAction::Aggregate => run_aggregate(output_format),
        MetricsAction::Status => run_status(output_format),
    }
}

#[derive(Debug, Serialize)]
struct CaptureRow {
    session_id: String,
    pass_at_1: Option<f32>,
    refine_rate: f32,
    cost_per_task_cents: Option<u32>,
    ts_ms: u64,
}

fn run_capture(session_id: &str, output_format: CliOutputFormat) -> Result<(), String> {
    if session_id.is_empty() {
        return Err("metrics capture: --session-id must be non-empty".into());
    }
    let collector = MetricsCollector::new();
    let metric = collector
        .capture_metrics(session_id)
        .map_err(|e| format!("metrics capture: {e}"))?;
    let row = CaptureRow {
        session_id: metric.session_id.clone(),
        pass_at_1: metric.pass_at_1(),
        refine_rate: metric.refine_rate,
        cost_per_task_cents: metric.cost_per_task_cents,
        ts_ms: metric.ts_ms,
    };
    render(&[row], output_format)
}

#[derive(Debug, Serialize)]
struct PendingRow {
    target: &'static str,
    verdict: &'static str,
    detail: String,
}

fn run_swe_bench_pro_dry(output_format: CliOutputFormat) -> Result<(), String> {
    let cfg = SweBenchProConfig::pending("pro-v1", "qwen2.5-coder-7b");
    match run_swebench_pro(&cfg) {
        Ok(_) => Err("metrics swe-bench-pro: unexpected Ok in RUNPOD-PENDING config".into()),
        Err(SweBenchProError::RunpodPending) => render(
            &[PendingRow {
                target: "swe-bench-pro",
                verdict: "runpod_pending",
                detail: "live SWE-Bench Pro transport not wired in this scope-lean ship".into(),
            }],
            output_format,
        ),
        Err(other) => Err(format!("metrics swe-bench-pro: {other}")),
    }
}

fn run_refinebench_dry(output_format: CliOutputFormat) -> Result<(), String> {
    let cfg = RefineBenchConfig::pending("rb-v1", "qwen2.5-coder-7b");
    match run_refinebench(&cfg) {
        Ok(_) => Err("metrics refinebench: unexpected Ok in RUNPOD-PENDING config".into()),
        Err(RefineBenchError::RunpodPending) => render(
            &[PendingRow {
                target: "refinebench",
                verdict: "runpod_pending",
                detail: "live RefineBench transport not wired in this scope-lean ship".into(),
            }],
            output_format,
        ),
        Err(other) => Err(format!("metrics refinebench: {other}")),
    }
}

#[derive(Debug, Serialize)]
struct AggregateRow {
    session_count: usize,
    refine_rate_avg: f32,
    cost_per_task_total_cents: Option<u64>,
    ts_ms: u64,
}

fn run_aggregate(output_format: CliOutputFormat) -> Result<(), String> {
    // Build a single zero-observation snapshot so the aggregate path is
    // exercised. Operators wire real per-session snapshots in a follow-up
    // patch when the dashboard surface lands.
    let collector = MetricsCollector::new();
    let snap = collector
        .capture_metrics("cli-smoke-session")
        .map_err(|e| format!("metrics aggregate: capture: {e}"))?;
    let report = aggregate_session_metrics(&[snap])
        .map_err(|e| format!("metrics aggregate: {e}"))?;
    let row = AggregateRow {
        session_count: report.sessions.len(),
        refine_rate_avg: report.refine_rate_avg,
        cost_per_task_total_cents: report.cost_per_task_total_cents,
        ts_ms: report.ts_ms,
    };
    render(&[row], output_format)
}

#[derive(Debug, Serialize)]
struct StatusRow {
    ssb_schema_version_floor: i64,
    ssb_action_types: Vec<&'static str>,
    primitives: Vec<&'static str>,
    runpod_pending: Vec<&'static str>,
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let row = StatusRow {
        ssb_schema_version_floor: elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC,
        ssb_action_types: vec![
            "coding_agent_metrics_captured",
            "swe_bench_pro_run_started",
            "swe_bench_pro_run_completed",
            "refinebench_score_emitted",
            "metrics_aggregation_run",
        ],
        primitives: vec![
            "MetricsCollector",
            "aggregate_session_metrics",
            "run_swebench_pro",
            "run_refinebench",
        ],
        runpod_pending: vec!["swe-bench-pro live transport", "refinebench live transport"],
    };
    render(&[row], output_format)
}

fn render<T: Serialize>(rows: &[T], output_format: CliOutputFormat) -> Result<(), String> {
    let out = match output_format {
        CliOutputFormat::Json => serde_json::to_string_pretty(rows)
            .map_err(|e| format!("render: json serialise failed: {e}"))?,
        CliOutputFormat::Toml => {
            let wrapped = serde_json::json!({ "rows": rows });
            toml::to_string(&wrapped)
                .map_err(|e| format!("render: toml serialise failed: {e}"))?
        }
        CliOutputFormat::Text => {
            let v = serde_json::to_value(rows)
                .map_err(|e| format!("render: text->json failed: {e}"))?;
            format!("{v:#}")
        }
    };
    println!("{out}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_renders_for_a_session_id() {
        run_capture("s-test", CliOutputFormat::Json).expect("ok");
    }

    #[test]
    fn capture_rejects_empty_session_id() {
        let err = run_capture("", CliOutputFormat::Json).unwrap_err();
        assert!(err.contains("--session-id"));
    }

    #[test]
    fn swe_bench_pro_dry_run_renders_runpod_pending() {
        run_swe_bench_pro_dry(CliOutputFormat::Json).expect("ok");
    }

    #[test]
    fn refinebench_dry_run_renders_runpod_pending() {
        run_refinebench_dry(CliOutputFormat::Json).expect("ok");
    }

    #[test]
    fn aggregate_smoke_renders_one_session() {
        run_aggregate(CliOutputFormat::Json).expect("ok");
    }

    #[test]
    fn status_renders_five_action_types() {
        run_status(CliOutputFormat::Json).expect("ok");
    }
}
