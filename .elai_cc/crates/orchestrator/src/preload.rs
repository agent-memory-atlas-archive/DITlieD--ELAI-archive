//! Predictive pre-loading module for FSM-driven model pre-loading.
//!
//! Phase 3J.6 deliverable. Triggers model pre-loading on FSM state transitions.
//! Uses PreloadPolicy from providers crate to determine which models to pre-load.

use elai_providers::multiplexing::PreloadPolicy;
use crate::fsm::FsmState;

/// Pre-loading manager that triggers model pre-loading on FSM state transitions.
pub struct PreloadManager {
    policy: PreloadPolicy,
}

impl PreloadManager {
    /// Create a new preload manager with the default policy.
    pub fn new() -> Self {
        Self {
            policy: PreloadPolicy::default(),
        }
    }

    /// Create a new preload manager with a custom policy.
    pub fn with_policy(policy: PreloadPolicy) -> Self {
        Self { policy }
    }

    /// Trigger pre-loading for models associated with the given FSM state.
    ///
    /// This should be called on FSM state entry (not exit).
    /// Pre-loading happens in the background via tokio::spawn.
    ///
    /// # Arguments
    /// * `state` - The FSM state being entered
    ///
    /// # Returns
    /// A list of model IDs that will be pre-loaded (for logging/telemetry).
    pub fn trigger_preload(&self, state: FsmState) -> Vec<String> {
        let state_str = state.to_string();
        let models = self.policy.models_for_state(&state_str);

        // TODO: Implement actual background pre-loading with tokio::spawn
        // This requires:
        // 1. Access to ModelPool to check VRAM availability
        // 2. Access to provider to trigger model loading
        // 3. VRAM checks using MultiplexingPolicy.can_colocate()
        // 4. Model placement using GreedyModelPlacer (phase 3J.8)
        //
        // For now, this is a structural placeholder that returns the model list.
        // The actual pre-loading will be wired in a follow-up task.

        models
    }

    /// Trigger model placement for a set of models across available GPUs.
    /// Phase 3J.8 deliverable. Uses GreedyModelPlacer for optimal assignment.
    ///
    /// # TODO
    /// This is a structural hook. Actual wiring requires:
    /// - Access to VllmModelPool or ModelPool trait implementation
    /// - Real GPU discovery (NVML or vLLM GPU query)
    /// - Real model VRAM requirements from model metadata
    /// - Integration with agent pool to trigger placement at phase boundaries
    ///
    /// # Arguments
    /// * `models` - Models to place (list of (id, vram_bytes, tier) tuples)
    /// * `gpus` - Available GPUs (list of (id, total_vram_bytes) tuples)
    ///
    /// # Returns
    /// Placement result with assignments and estimated swap latency.
    #[allow(dead_code)]
    pub fn place_models(
        &self,
        models: Vec<(String, u64, elai_providers::SloTier)>,
        gpus: Vec<(u8, u64)>,
    ) -> Result<elai_providers::PlacementResult, elai_providers::MultiplexingError> {
        // TODO: Wire to actual ModelPool implementation
        // For now, this is a structural placeholder
        Err(elai_providers::MultiplexingError::InvalidModelSize)
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
