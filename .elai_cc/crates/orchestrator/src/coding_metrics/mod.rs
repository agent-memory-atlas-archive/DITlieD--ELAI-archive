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
pub mod stable_pass;
pub mod swebench_pro;
pub mod types;

pub use aggregator::{aggregate_session_metrics, AggregatedReport, AggregatorError};
pub use collector::{MetricsCollector, MetricsCollectorError};
pub use refinebench::{run_refinebench, RefineBenchConfig, RefineBenchReport, RefineBenchError};
pub use stable_pass::{
    compute_stable_pass, record_harness_attribution_tuple, record_stable_pass_at_1,
    record_stop_verified, SeedOutcome, StablePassError, StablePassMetric,
    StopVerifiedViolation, DEFAULT_SEED_COUNT,
};
pub use swebench_pro::{
    run_swebench_pro, SweBenchProConfig, SweBenchProError, SweBenchProReport,
};
pub use types::CodingMetric;

// E-49 (SCHEMA_VERSION 189) -- coding-agent metrics named-token substrate.
// SINGLE-VARIANT Pattern-2 (41st application). Re-exports the named-token
// payload + kind discriminator from ssb.rs so callers in this module
// (and downstream consumers) reach them through the canonical
// coding_metrics:: surface. Closes TASK-E49-005 / 006 / 007 SPEC-DRIFT
// against the 3 spec-named tokens stable_pass_at_1 /
// harness_attribution_tuple / stop_verified (zero hits across the
// codebase before this bump per cross-check E-16_e47_e48_e49.md). Wire-
// dark unless ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE=1.
pub use crate::ssb::{
    CodingMetricNamedTokenKind, CodingMetricNamedTokenPayload,
    ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE_ENV,
};
