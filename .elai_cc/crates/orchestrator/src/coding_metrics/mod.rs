// E-49 (SCHEMA_VERSION 138) — Coding-agent metrics suite (scope-lean ship).
//
// Replaces single-seed Pass@1 reporting with a 2026-grade metric surface:
// STABLE-Pass@1 capture point, SWE-bench Pro / RefineBench runner stubs,
// and a per-session metric aggregator. Live SWE-Bench Pro + RefineBench
// runs against real models are RUNPOD-PENDING — the runner interfaces
// ship as production-callable scaffolding with structured error returns
// for the live boundary.
//
// Scope cut from the E-49 plan's 12-task surface (see cross-check
// report at workflow/sessions/s-25120/cross-check-E/E-49_coding_agent_metrics-report.md).
// The original plan's `metrics/safety/` subdirectory layout is folded into a
// flat module layout per the scope-lean rebase.
//
// Design source: SWE-bench Pro (arXiv:2509.16941, claimed) +
// RefineBench (arXiv:2511.22173, claimed). Citations DEFERRED to
// mistakes.md per cluster-9 cite-deferral discipline; the contract
// asserts wire-shape + module-shape + SSB round-trip independently of
// the paper grounding.

pub mod aggregator;
pub mod collector;
pub mod refinebench;
pub mod swebench_pro;
pub mod types;

pub use aggregator::{aggregate_session_metrics, AggregatedReport, AggregatorError};
pub use collector::{MetricsCollector, MetricsCollectorError};
pub use refinebench::{run_refinebench, RefineBenchConfig, RefineBenchReport, RefineBenchError};
pub use swebench_pro::{
    run_swebench_pro, SweBenchProConfig, SweBenchProError, SweBenchProReport,
};
pub use types::CodingMetric;
