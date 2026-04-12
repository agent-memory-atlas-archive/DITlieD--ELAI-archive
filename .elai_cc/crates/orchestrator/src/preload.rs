//! Predictive pre-loading module for FSM-driven model pre-loading.
//!
//! Phase 3J.6 deliverable. Triggers model pre-loading on FSM state transitions.
//! Uses PreloadPolicy from providers crate to determine which models to pre-load.
//! Optionally uses GreedyModelPlacer for GPU placement decisions.

use elai_providers::multiplexing::PreloadPolicy;
use elai_providers::model_placer::{GreedyModelPlacer, GpuSpec, ModelSpec, PlacementResult};
use elai_providers::multiplexing::SloTier;
use crate::fsm::FsmState;

/// Estimated VRAM per model role in bytes (conservative defaults).
const DEFAULT_MODEL_VRAM_BYTES: u64 = 4_000_000_000; // 4GB

/// Pre-loading manager that triggers model pre-loading on FSM state transitions.
/// Integrates with GreedyModelPlacer for GPU placement decisions.
pub struct PreloadManager {
    policy: PreloadPolicy,
    placer: GreedyModelPlacer,
    gpus: Vec<GpuSpec>,
}

impl PreloadManager {
    /// Create a new preload manager with the default policy and a single GPU spec.
    pub fn new() -> Self {
        Self {
            policy: PreloadPolicy::default(),
            placer: GreedyModelPlacer::default(),
            gpus: vec![GpuSpec::new(0, 12_000_000_000)], // Default: single 12GB GPU
        }
    }

    /// Create a new preload manager with a custom policy.
    pub fn with_policy(policy: PreloadPolicy) -> Self {
        Self {
            policy,
            placer: GreedyModelPlacer::default(),
            gpus: vec![GpuSpec::new(0, 12_000_000_000)],
        }
    }

    /// Configure available GPUs for placement decisions.
    pub fn with_gpus(mut self, gpus: Vec<GpuSpec>) -> Self {
        self.gpus = gpus;
        self
    }

    /// Configure the model placer.
    pub fn with_placer(mut self, placer: GreedyModelPlacer) -> Self {
        self.placer = placer;
        self
    }

    /// Trigger pre-loading for models associated with the given FSM state.
    ///
    /// Determines which models to pre-load via PreloadPolicy, then uses
    /// GreedyModelPlacer to decide GPU placement. Returns the model IDs
    /// that will be pre-loaded.
    pub fn trigger_preload(&self, state: FsmState) -> Vec<String> {
        let state_str = state.to_string();
        let model_ids = self.policy.models_for_state(&state_str);

        if model_ids.is_empty() {
            return model_ids;
        }

        // Build ModelSpec list for placement
        let specs: Vec<ModelSpec> = model_ids
            .iter()
            .map(|id| ModelSpec::new(id.clone(), DEFAULT_MODEL_VRAM_BYTES, SloTier::Basic))
            .collect();

        // Compute placement (which GPU each model goes on)
        match self.placer.place(&specs, &self.gpus) {
            Ok(result) => {
                self.log_placement(&result, &model_ids);
            }
            Err(e) => {
                tracing::warn!(
                    state = %state,
                    error = %e,
                    "Model placement failed during pre-load"
                );
            }
        }

        model_ids
    }

    fn log_placement(&self, result: &PlacementResult, model_ids: &[String]) {
        if result.success {
            tracing::info!(
                assignments = ?result.assignments,
                swap_latency_ms = result.total_swap_latency_ms,
                "Model placement computed for pre-load"
            );
        } else {
            tracing::warn!(
                models = ?model_ids,
                error = ?result.error,
                "Model placement failed: insufficient GPU resources for pre-load"
            );
        }
    }
}

impl Default for PreloadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preload_manager_returns_models_for_state() {
        let manager = PreloadManager::new();
        
        // EXECUTING should pre-load auditor
        let models = manager.trigger_preload(FsmState::Executing);
        assert_eq!(models, vec!["auditor".to_string()]);
        
        // AUDITING should pre-load executor
        let models = manager.trigger_preload(FsmState::Auditing);
        assert_eq!(models, vec!["executor".to_string()]);
        
        // PLANNING should pre-load prompt-engineer
        let models = manager.trigger_preload(FsmState::Planning);
        assert_eq!(models, vec!["prompt-engineer".to_string()]);
    }

    #[test]
    fn test_preload_manager_custom_policy() {
        let mut policy = PreloadPolicy::default();
        policy.add_mapping("CUSTOM_STATE", vec!["custom_model".to_string()]);
        
        let manager = PreloadManager::with_policy(policy);
        let models = manager.trigger_preload(FsmState::Planning);
        
        // Default mappings still work
        assert!(!models.is_empty());
    }
}
