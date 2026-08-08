//! Deterministic task-shape cost profiling for the live routing seam.
use std::env;

/// Environment variable controlling the optional cost profiler.
pub const ENV_COST_PROFILER_ENABLE: &str = "ELAI_COST_PROFILER_ENABLE";

/// Coarse, deterministic prediction used before the budget-aware router.
#[derive(Debug, Clone, PartialEq)]
pub enum CostProfileVerdict {
    /// The task does not match a known expensive shape.
    Cheap,
    /// The task matches an expensive shape, with stable observability data.
    Expensive { fingerprint: String, feature_vec: Vec<f32> },
}

/// Small keyword-based classifier for expensive task shapes.
#[derive(Debug, Clone, Default)]
pub struct CostProfiler;

impl CostProfiler {
    /// Construct the built-in deterministic profiler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Classify task text without network, filesystem, or model access.
    #[must_use]
    pub fn classify(&self, task_text: &str) -> CostProfileVerdict {
        let text = task_text.to_ascii_lowercase();
        let patterns: [(&str, &[&str]); 5] = [
            ("multi_file_refactor", &["multi-file", "refactor"]),
            ("full_repo_audit", &["full-repo", "audit"]),
            ("longcot", &["longcot"]),
            ("gepa", &["gepa"]),
            ("rewrite_entire", &["rewrite entire"]),
        ];
        for (fingerprint, terms) in patterns {
            if terms.iter().all(|term| text.contains(term)) {
                let feature_vec = vec![
                    task_text.split_whitespace().count() as f32,
                    text.matches('/').count() as f32,
                    terms.len() as f32,
                ];
                return CostProfileVerdict::Expensive {
                    fingerprint: fingerprint.to_string(),
                    feature_vec,
                };
            }
        }
        if text.contains("across the crate") {
            return CostProfileVerdict::Expensive {
                fingerprint: "across_crate".to_string(),
                feature_vec: vec![task_text.split_whitespace().count() as f32, 1.0, 1.0],
            };
        }
        CostProfileVerdict::Cheap
    }
}

/// Whether the profiler is enabled; unset and unrecognized values are off.
#[must_use]
pub fn cost_profiler_enabled() -> bool {
    env::var(ENV_COST_PROFILER_ENABLE)
        .ok()
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}
