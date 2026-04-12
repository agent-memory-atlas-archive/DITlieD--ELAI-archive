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
        // 
        // For now, this is a structural placeholder that returns the model list.
        // The actual pre-loading will be wired in a follow-up task.
        
        models
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
