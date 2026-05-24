// E-49 — RefineBench live runner (RUNPOD-PENDING).
//
// Same RUNPOD-PENDING posture as SWE-Bench Pro: ship the configuration +
// report shapes + structured error returns. The live RefineBench transport
// is wired in a follow-up patch once a remote pod is available.
//
// Design source: RefineBench (claimed arXiv:2511.22173). Citation
// DEFERRED to mistakes.md cluster #9.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// E-49 — RefineBench live-run configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefineBenchConfig {
    pub pod_endpoint: Option<String>,
    pub suite_version: String,
    pub model_id: String,
    pub max_tasks: Option<u32>,
}

impl RefineBenchConfig {
    /// Construct a RUNPOD-PENDING config.
    pub fn pending(suite_version: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self {
            pod_endpoint: None,
            suite_version: suite_version.into(),
            model_id: model_id.into(),
            max_tasks: None,
        }
    }
}

/// E-49 — RefineBench report shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefineBenchReport {
    pub suite_version: String,
    pub model_id: String,
    pub tasks_attempted: u32,
    pub tasks_refined_successfully: u32,
    pub refine_rate: f32,
    pub ts_ms: u64,
}

#[derive(Debug, Error)]
pub enum RefineBenchError {
    #[error("RefineBench live run is RUNPOD-PENDING (pod_endpoint=None)")]
    RunpodPending,
    #[error("RefineBench pod returned a malformed response: {0}")]
    BadResponse(String),
    #[error("config rejected: {0}")]
    BadConfig(String),
}

/// E-49 — entry point for the RefineBench live run boundary.
///
/// Same posture as `run_swebench_pro`: `None` endpoint => `RunpodPending`;
/// `Some(endpoint)` => `BadConfig("live HTTP transport is RUNPOD-PENDING")`
/// until a real pod is wired.
pub fn run_refinebench(config: &RefineBenchConfig) -> Result<RefineBenchReport, RefineBenchError> {
    if config.suite_version.is_empty() {
        return Err(RefineBenchError::BadConfig(
            "suite_version must be non-empty".into(),
        ));
    }
    if config.model_id.is_empty() {
        return Err(RefineBenchError::BadConfig(
            "model_id must be non-empty".into(),
        ));
    }
    match &config.pod_endpoint {
        None => Err(RefineBenchError::RunpodPending),
        Some(endpoint) if endpoint.is_empty() => Err(RefineBenchError::BadConfig(
            "pod_endpoint must be non-empty when Some".into(),
        )),
        Some(_endpoint) => Err(RefineBenchError::BadConfig(
            "live RefineBench HTTP transport is RUNPOD-PENDING".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_config_returns_runpod_pending() {
        let cfg = RefineBenchConfig::pending("rb-v1", "qwen2.5-7b");
        let err = run_refinebench(&cfg).unwrap_err();
        assert!(matches!(err, RefineBenchError::RunpodPending));
    }

    #[test]
    fn empty_suite_version_is_rejected() {
        let cfg = RefineBenchConfig {
            pod_endpoint: None,
            suite_version: String::new(),
            model_id: "qwen".into(),
            max_tasks: None,
        };
        let err = run_refinebench(&cfg).unwrap_err();
        assert!(matches!(err, RefineBenchError::BadConfig(_)));
    }
}
