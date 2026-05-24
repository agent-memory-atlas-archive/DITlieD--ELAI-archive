// E-49 — SWE-Bench Pro live runner (RUNPOD-PENDING).
//
// The live SWE-Bench Pro run boundary requires a remote GPU pod with the
// vendored Pro task pool and a configured model runtime. The host-side
// scaffolding here ships the configuration shape + report shape +
// structured error returns so the CLI / dashboard / contract test can
// exercise the boundary without paying the live-run cost.
//
// Live execution boundary: `run_swebench_pro` returns
// `SweBenchProError::RunpodPending` when invoked without a live pod
// handle. Contract test asserts that branch + the report shape.
//
// Design source: SWE-bench Pro (claimed arXiv:2509.16941). Citation
// DEFERRED to mistakes.md cluster #9.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// E-49 — SWE-Bench Pro live-run configuration.
///
/// Fields:
///   * `pod_endpoint`: URL of the remote pod hosting the Pro task pool
///     and the candidate model. None = RUNPOD-PENDING boundary.
///   * `task_pool_version`: pinned identifier for the Pro task pool
///     (e.g. `"pro-v1-2026-05-25"`).
///   * `model_id`: candidate model identifier.
///   * `harness_commit_sha`: ELAI harness commit SHA to record in the
///     report (attribution lever).
///   * `max_tasks`: cap the run to at most N tasks (None = full pool).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SweBenchProConfig {
    pub pod_endpoint: Option<String>,
    pub task_pool_version: String,
    pub model_id: String,
    pub harness_commit_sha: String,
    pub max_tasks: Option<u32>,
}

impl SweBenchProConfig {
    /// Construct a config in RUNPOD-PENDING posture: `pod_endpoint =
    /// None`. The live boundary is `run_swebench_pro`; this constructor
    /// is convenient for the CLI `elai metrics swe-bench-pro` dry-run.
    pub fn pending(task_pool_version: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self {
            pod_endpoint: None,
            task_pool_version: task_pool_version.into(),
            model_id: model_id.into(),
            harness_commit_sha: String::new(),
            max_tasks: None,
        }
    }
}

/// E-49 — SWE-Bench Pro report shape (one row per run).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SweBenchProReport {
    pub task_pool_version: String,
    pub model_id: String,
    pub harness_commit_sha: String,
    pub tasks_attempted: u32,
    pub tasks_solved: u32,
    pub pass_at_1: f32,
    pub ts_ms: u64,
}

#[derive(Debug, Error)]
pub enum SweBenchProError {
    #[error("SWE-Bench Pro live run is RUNPOD-PENDING (pod_endpoint=None)")]
    RunpodPending,
    #[error("SWE-Bench Pro pod returned a malformed response: {0}")]
    BadResponse(String),
    #[error("config rejected: {0}")]
    BadConfig(String),
}

/// E-49 — entry point for the SWE-Bench Pro live run boundary.
///
/// When `config.pod_endpoint` is `None`, returns
/// `SweBenchProError::RunpodPending` (the documented live-run boundary).
/// When `config.pod_endpoint` is `Some(url)`, the function returns
/// `SweBenchProError::BadConfig` since the live transport is
/// RUNPOD-PENDING — this branch will be replaced with the real HTTP
/// call in a follow-up patch when a remote pod is wired.
///
/// Per H-3 (no auditing in ungated code paths) the function is pure
/// modulo the error return: no SSB writes, no filesystem mutation.
pub fn run_swebench_pro(config: &SweBenchProConfig) -> Result<SweBenchProReport, SweBenchProError> {
    if config.task_pool_version.is_empty() {
        return Err(SweBenchProError::BadConfig(
            "task_pool_version must be non-empty".into(),
        ));
    }
    if config.model_id.is_empty() {
        return Err(SweBenchProError::BadConfig(
            "model_id must be non-empty".into(),
        ));
    }

    match &config.pod_endpoint {
        None => Err(SweBenchProError::RunpodPending),
        Some(endpoint) if endpoint.is_empty() => Err(SweBenchProError::BadConfig(
            "pod_endpoint must be non-empty when Some".into(),
        )),
        // Live transport not wired in this scope-lean ship. The CLI surface
        // is anchored via `metrics_cmd::swe_bench_pro_dry_run`; the live
        // HTTP path lands in a follow-up patch once a pod is provisioned.
        Some(_endpoint) => Err(SweBenchProError::BadConfig(
            "live SWE-Bench Pro HTTP transport is RUNPOD-PENDING".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_config_returns_runpod_pending() {
        let cfg = SweBenchProConfig::pending("pro-v1", "qwen2.5-7b");
        let err = run_swebench_pro(&cfg).unwrap_err();
        assert!(matches!(err, SweBenchProError::RunpodPending));
    }

    #[test]
    fn empty_task_pool_version_is_rejected() {
        let mut cfg = SweBenchProConfig::pending("", "qwen");
        cfg.task_pool_version = String::new();
        let err = run_swebench_pro(&cfg).unwrap_err();
        assert!(matches!(err, SweBenchProError::BadConfig(_)));
    }

    #[test]
    fn empty_pod_endpoint_when_some_is_rejected() {
        let cfg = SweBenchProConfig {
            pod_endpoint: Some(String::new()),
            task_pool_version: "pro-v1".into(),
            model_id: "qwen".into(),
            harness_commit_sha: String::new(),
            max_tasks: None,
        };
        let err = run_swebench_pro(&cfg).unwrap_err();
        assert!(matches!(err, SweBenchProError::BadConfig(_)));
    }
}
