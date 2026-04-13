//! GPU multiplexing, SLO quality tiers, and predictive pre-loading.
//!
//! Phase 3J.6 deliverable. See docs/specs/guide/steps/phase-3J.txt §3J.6
//! and docs/specs/crates/providers.txt S4 L120-140.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// SLO Quality Tiers (Murakkab-derived)
// ---------------------------------------------------------------------------

/// SLO quality tier for agent roles.
/// Controls model selection, GPU assignment, and VRAM allocation.
/// Source: providers.txt S4 L120-128.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SloTier {
    /// Highest precision model, full VRAM allocation.
    /// Roles: auditor, researcher.
    Best,
    /// Standard model, standard allocation.
    /// Roles: executor, context-compiler.
    Good,
    /// Can use smaller model variant (e.g., quantized).
    /// Roles: prompt-engineer.
    Fair,
    /// Smallest capable model, minimal resources.
    /// Roles: memory-writer.
    Basic,
}

impl SloTier {
    /// Default tier for a given agent role.
    /// Returns None if role is unknown (caller should decide).
    #[must_use]
    pub fn for_role(role: &str) -> Option<Self> {
        match role {
            "auditor" | "researcher" => Some(SloTier::Best),
            "executor" | "context-compiler" => Some(SloTier::Good),
            "prompt-engineer" => Some(SloTier::Fair),
            "memory-writer" => Some(SloTier::Basic),
            _ => None,
        }
    }

    /// VRAM allocation factor (fraction of GPU VRAM to reserve).
    /// Best = 1.0 (full GPU), Good = 0.8, Fair = 0.5, Basic = 0.3.
    #[must_use]
    pub fn vram_factor(&self) -> f64 {
        match self {
            SloTier::Best => 1.0,
            SloTier::Good => 0.8,
            SloTier::Fair => 0.5,
            SloTier::Basic => 0.3,
        }
    }
}

impl std::fmt::Display for SloTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SloTier::Best => write!(f, "BEST"),
            SloTier::Good => write!(f, "GOOD"),
            SloTier::Fair => write!(f, "FAIR"),
            SloTier::Basic => write!(f, "BASIC"),
        }
    }
}

// ---------------------------------------------------------------------------
// Multiplexing Policy
// ---------------------------------------------------------------------------

/// GPU multiplexing policy for collocating small models on same GPU.
/// Source: providers.txt S4 L130-136, phase-3J.txt §3J.6.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplexingPolicy {
    /// Safety margin in bytes to prevent GPU OOM.
    /// Default: 1GB = 1_073_741_824 bytes.
    pub safety_margin_bytes: u64,
    /// Maximum number of models that can colocate on a single GPU.
    /// Default: 3.
    pub max_colocated: usize,
    /// Minimum model size (in bytes) that requires a dedicated GPU.
    /// Models smaller than this may be colocated. Default: 4GB.
    pub min_model_size_for_dedicated_gpu: u64,
    /// Whether multiplexing is enabled.
    /// Phase 0-1 (single GPU): false, sequential loading only.
    /// Phase 2+ (dual GPU): true for GPU1.
    pub enabled: bool,
}

impl Default for MultiplexingPolicy {
    fn default() -> Self {
        Self {
            safety_margin_bytes: 1_073_741_824, // 1GB
            max_colocated: 3,
            min_model_size_for_dedicated_gpu: 4_294_967_296, // 4GB
            enabled: false, // Disabled in single-GPU mode (Phase 0-1)
        }
    }
}

impl MultiplexingPolicy {
    /// Check if a model can colocate on a target GPU given current VRAM usage.
    ///
    /// # Arguments
    /// * `model_vram_bytes` - VRAM required by the model in bytes
    /// * `current_vram_used_bytes` - Current VRAM usage on target GPU in bytes
    /// * `gpu_total_vram_bytes` - Total VRAM capacity of target GPU in bytes
    /// * `current_colocated_count` - Number of models already colocated on target GPU
    ///
    /// # Returns
    /// * `Ok(true)` - Colocation is safe
    /// * `Ok(false)` - Colocation would exceed VRAM or policy limits
    /// * `Err` - Invalid parameters (e.g., model_vram_bytes == 0)
    #[must_use]
    pub fn can_colocate(
        &self,
        model_vram_bytes: u64,
        current_vram_used_bytes: u64,
        gpu_total_vram_bytes: u64,
        current_colocated_count: usize,
    ) -> Result<bool, MultiplexingError> {
        if model_vram_bytes == 0 {
            return Err(MultiplexingError::InvalidModelSize);
        }
        if gpu_total_vram_bytes == 0 {
            return Err(MultiplexingError::InvalidGpuCapacity);
        }

        // If multiplexing is disabled, only allow single model
        if !self.enabled {
            return Ok(current_vram_used_bytes == 0);
        }

        // Check max colocated limit
        if current_colocated_count >= self.max_colocated {
            return Ok(false);
        }

        // Check VRAM capacity with safety margin
        let projected_usage = current_vram_used_bytes
            .checked_add(model_vram_bytes)
            .ok_or(MultiplexingError::VramOverflow)?;
        let available_with_margin = gpu_total_vram_bytes
            .checked_sub(self.safety_margin_bytes)
            .ok_or(MultiplexingError::SafetyMarginExceedsCapacity)?;

        Ok(projected_usage <= available_with_margin)
    }

    /// Check if a model is large enough to require a dedicated GPU.
    #[must_use]
    pub fn requires_dedicated_gpu(&self, model_vram_bytes: u64) -> bool {
        model_vram_bytes >= self.min_model_size_for_dedicated_gpu
    }
}

// ---------------------------------------------------------------------------
// Preload Policy (FSM-driven predictive pre-loading)
// ---------------------------------------------------------------------------

/// Predictive pre-loading policy mapping FSM states to model IDs.
/// Source: providers.txt S4 L138-140, phase-3J.txt §3J.6.
///
/// FSM state transitions trigger pre-loading:
/// - EXECUTING -> pre-load auditor model
/// - AUDITING -> pre-load executor model
/// - PLANNING -> pre-load prompt-engineer model
/// - TESTING_FIRST -> pre-load executor model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreloadPolicy {
    /// Map from FSM state to list of model IDs to pre-load.
    /// Configurable in elai_cc.toml [providers.preload].
    pub state_to_models: HashMap<String, Vec<String>>,
}

impl Default for PreloadPolicy {
    fn default() -> Self {
        let mut state_to_models = HashMap::new();
        // Default preload mappings from phase-3J.txt
        state_to_models.insert("EXECUTING".to_string(), vec!["auditor".to_string()]);
        state_to_models.insert("AUDITING".to_string(), vec!["executor".to_string()]);
        state_to_models.insert("PLANNING".to_string(), vec!["prompt-engineer".to_string()]);
        state_to_models.insert("TESTING_FIRST".to_string(), vec!["executor".to_string()]);
        Self { state_to_models }
    }
}

impl PreloadPolicy {
    /// Get the list of model IDs to pre-load for a given FSM state.
    #[must_use]
    pub fn models_for_state(&self, state: &str) -> Vec<String> {
        self.state_to_models
            .get(state)
            .cloned()
            .unwrap_or_default()
    }

    /// Add a preload mapping for a state.
    pub fn add_mapping(&mut self, state: &str, model_ids: Vec<String>) {
        self.state_to_models.insert(state.to_string(), model_ids);
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors related to GPU multiplexing decisions.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MultiplexingError {
    #[error("model VRAM size cannot be zero")]
    InvalidModelSize,
    #[error("GPU total VRAM capacity cannot be zero")]
    InvalidGpuCapacity,
    #[error("VRAM arithmetic overflow")]
    VramOverflow,
    #[error("safety margin exceeds GPU capacity")]
    SafetyMarginExceedsCapacity,
    #[error("no GPU available (NVML discovery failed or no devices found)")]
    NoGpuAvailable,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // SloTier tests
    // -----------------------------------------------------------------------

    #[test]
    fn slo_tier_for_role_known_roles() {
        assert_eq!(SloTier::for_role("auditor"), Some(SloTier::Best));
        assert_eq!(SloTier::for_role("researcher"), Some(SloTier::Best));
        assert_eq!(SloTier::for_role("executor"), Some(SloTier::Good));
        assert_eq!(SloTier::for_role("context-compiler"), Some(SloTier::Good));
        assert_eq!(SloTier::for_role("prompt-engineer"), Some(SloTier::Fair));
        assert_eq!(SloTier::for_role("memory-writer"), Some(SloTier::Basic));
    }

    #[test]
    fn slo_tier_for_role_unknown_returns_none() {
        assert_eq!(SloTier::for_role("unknown_role"), None);
    }

    #[test]
    fn slo_tier_vram_factor_decreases_with_tier() {
        assert!(SloTier::Best.vram_factor() > SloTier::Good.vram_factor());
        assert!(SloTier::Good.vram_factor() > SloTier::Fair.vram_factor());
        assert!(SloTier::Fair.vram_factor() > SloTier::Basic.vram_factor());
    }

    #[test]
    fn slo_tier_display_format() {
        assert_eq!(SloTier::Best.to_string(), "BEST");
        assert_eq!(SloTier::Good.to_string(), "GOOD");
        assert_eq!(SloTier::Fair.to_string(), "FAIR");
        assert_eq!(SloTier::Basic.to_string(), "BASIC");
    }

    // -----------------------------------------------------------------------
    // MultiplexingPolicy tests
    // -----------------------------------------------------------------------

    #[test]
    fn multiplexing_policy_default_values() {
        let policy = MultiplexingPolicy::default();
        assert_eq!(policy.safety_margin_bytes, 1_073_741_824);
        assert_eq!(policy.max_colocated, 3);
        assert_eq!(policy.min_model_size_for_dedicated_gpu, 4_294_967_296);
        assert!(!policy.enabled);
    }

    #[test]
    fn can_colocate_when_disabled_only_allows_single_model() {
        let policy = MultiplexingPolicy {
            enabled: false,
            ..Default::default()
        };
        assert!(policy
            .can_colocate(1_000_000_000, 0, 12_000_000_000, 0)
            .unwrap());
        assert!(!policy
            .can_colocate(1_000_000_000, 1_000_000_000, 12_000_000_000, 1)
            .unwrap());
    }

    #[test]
    fn can_colocate_respects_safety_margin() {
        let policy = MultiplexingPolicy {
            enabled: true,
            safety_margin_bytes: 1_000_000_000,
            ..Default::default()
        };
        // GPU has 10GB, margin 1GB -> 9GB available
        // Current usage: 5GB, model needs 4GB -> 9GB total, fits
        assert!(policy
            .can_colocate(4_000_000_000, 5_000_000_000, 10_000_000_000, 1)
            .unwrap());
        // Current usage: 5GB, model needs 5GB -> 10GB total, exceeds margin
        assert!(!policy
            .can_colocate(5_000_000_000, 5_000_000_000, 10_000_000_000, 1)
            .unwrap());
    }

    #[test]
    fn can_colocate_respects_max_colocated_limit() {
        let policy = MultiplexingPolicy {
            enabled: true,
            max_colocated: 2,
            ..Default::default()
        };
        // Already at limit
        assert!(!policy
            .can_colocate(1_000_000_000, 2_000_000_000, 12_000_000_000, 2)
            .unwrap());
        // Under limit
        assert!(policy
            .can_colocate(1_000_000_000, 2_000_000_000, 12_000_000_000, 1)
            .unwrap());
    }

    #[test]
    fn can_colocate_rejects_zero_model_size() {
        let policy = MultiplexingPolicy::default();
        assert_eq!(
            policy.can_colocate(0, 0, 12_000_000_000, 0),
            Err(MultiplexingError::InvalidModelSize)
        );
    }

    #[test]
    fn can_colocate_rejects_zero_gpu_capacity() {
        let policy = MultiplexingPolicy::default();
        assert_eq!(
            policy.can_colocate(1_000_000_000, 0, 0, 0),
            Err(MultiplexingError::InvalidGpuCapacity)
        );
    }

    #[test]
    fn requires_dedicated_gpu_based_on_size() {
        let policy = MultiplexingPolicy {
            min_model_size_for_dedicated_gpu: 4_000_000_000,
            ..Default::default()
        };
        assert!(policy.requires_dedicated_gpu(5_000_000_000));
        assert!(!policy.requires_dedicated_gpu(3_000_000_000));
    }

    // -----------------------------------------------------------------------
    // PreloadPolicy tests
    // -----------------------------------------------------------------------

    #[test]
    fn preload_policy_default_mappings() {
        let policy = PreloadPolicy::default();
        assert_eq!(
            policy.models_for_state("EXECUTING"),
            vec!["auditor".to_string()]
        );
        assert_eq!(
            policy.models_for_state("AUDITING"),
            vec!["executor".to_string()]
        );
        assert_eq!(
            policy.models_for_state("PLANNING"),
            vec!["prompt-engineer".to_string()]
        );
        assert_eq!(
            policy.models_for_state("TESTING_FIRST"),
            vec!["executor".to_string()]
        );
    }

    #[test]
    fn preload_policy_unknown_state_returns_empty() {
        let policy = PreloadPolicy::default();
        assert!(policy.models_for_state("UNKNOWN_STATE").is_empty());
    }

    #[test]
    fn preload_policy_add_mapping() {
        let mut policy = PreloadPolicy::default();
        policy.add_mapping("CUSTOM_STATE", vec!["custom_model".to_string()]);
        assert_eq!(
            policy.models_for_state("CUSTOM_STATE"),
            vec!["custom_model".to_string()]
        );
    }

    // -----------------------------------------------------------------------
    // Adversarial tests (P3J.6 ADVERSARIAL requirements)
    // -----------------------------------------------------------------------

    #[test]
    fn adversarial_can_colocate_rejects_overflow_scenario() {
        // Test: simulate colocation where sum > GPU_VRAM - margin, verify returns false
        let policy = MultiplexingPolicy {
            enabled: true,
            safety_margin_bytes: 1_000_000_000,
            ..Default::default()
        };
        // GPU: 10GB, margin: 1GB -> 9GB available
        // Current: 8GB, model: 2GB -> 10GB total > 9GB available
        let result = policy.can_colocate(2_000_000_000, 8_000_000_000, 10_000_000_000, 2);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "should reject colocation that exceeds VRAM with margin");
    }

    #[test]
    fn adversarial_can_colocate_handles_zero_safety_margin() {
        // Test: edge case where safety_margin is set to 0 (should still work)
        let policy = MultiplexingPolicy {
            enabled: true,
            safety_margin_bytes: 0,
            ..Default::default()
        };
        // GPU: 10GB, margin: 0 -> 10GB available
        // Current: 9GB, model: 1GB -> 10GB total = 10GB available (fits exactly)
        let result = policy.can_colocate(1_000_000_000, 9_000_000_000, 10_000_000_000, 1);
        assert!(result.is_ok());
        assert!(result.unwrap(), "should accept exact fit with zero margin");
    }

    #[test]
    fn adversarial_can_colocate_handles_u64_overflow() {
        // Test: verify arithmetic overflow is detected and returns error
        let policy = MultiplexingPolicy {
            enabled: true,
            ..Default::default()
        };
        // Current usage near u64::MAX, adding any model should overflow
        let near_max = u64::MAX - 1_000_000_000;
        let result = policy.can_colocate(2_000_000_000, near_max, u64::MAX, 1);
        assert!(result.is_err(), "should detect arithmetic overflow");
    }

    #[test]
    fn adversarial_can_colocate_respects_max_colocated_zero() {
        // Test: edge case where max_colocated is set to 0 (should reject all)
        let policy = MultiplexingPolicy {
            enabled: true,
            max_colocated: 0,
            ..Default::default()
        };
        // Even with plenty of VRAM, should reject because max_colocated is 0
        let result = policy.can_colocate(1_000_000_000, 0, 100_000_000_000, 0);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "should reject when max_colocated is 0");
    }

    #[test]
    fn adversarial_slo_tier_mismatch_detection() {
        // Test: verify tier mismatch can be detected (role tier vs model tier)
        // This is a structural test - the actual warning would be logged by the caller
        let auditor_tier = SloTier::for_role("auditor").unwrap();
        let basic_tier = SloTier::Basic;
        
        // Auditor should be BEST, not BASIC
        assert_ne!(auditor_tier, basic_tier);
        assert_eq!(auditor_tier, SloTier::Best);
        
        // Verify VRAM factor difference would trigger warning
        let auditor_vram = auditor_tier.vram_factor();
        let basic_vram = basic_tier.vram_factor();
        assert!(
            auditor_vram > basic_vram,
            "BEST tier should have higher VRAM allocation than BASIC"
        );
    }

    #[test]
    fn adversarial_preload_policy_empty_state_handling() {
        // Test: verify empty state list doesn't cause crashes
        let policy = PreloadPolicy {
            state_to_models: HashMap::new(),
        };
        assert!(policy.models_for_state("ANY_STATE").is_empty());
        
        // Add empty list for a state
        let mut policy = PreloadPolicy::default();
        policy.add_mapping("EMPTY_STATE", vec![]);
        assert!(policy.models_for_state("EMPTY_STATE").is_empty());
    }

    #[test]
    fn adversarial_multiplexing_disabled_with_existing_models() {
        // Test: when multiplexing is disabled, existing models should block new ones
        let policy = MultiplexingPolicy {
            enabled: false, // Disabled (single-GPU mode)
            ..Default::default()
        };
        // Even with plenty of VRAM, should reject if any model is already loaded
        let result = policy.can_colocate(1_000_000_000, 1_000_000_000, 100_000_000_000, 1);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "disabled multiplexing should reject any colocation");

        // But should allow first model
        let result = policy.can_colocate(1_000_000_000, 0, 100_000_000_000, 0);
        assert!(result.is_ok());
        assert!(result.unwrap(), "disabled multiplexing should allow first model");
    }

    #[test]
    fn adversarial_preload_race_condition_phase_boundary() {
        // Test: verify pre-load policy respects phase boundaries (structural test)
        // The actual race prevention is implemented by the orchestrator's FSM hooks,
        // but we verify the policy provides the correct model lists for each state.
        let policy = PreloadPolicy::default();

        // EXECUTING -> pre-load auditor (not executor, which is current)
        let executing_models = policy.models_for_state("EXECUTING");
        assert!(
            !executing_models.is_empty(),
            "EXECUTING state should have preload targets"
        );
        assert!(
            !executing_models.contains(&"executor".to_string()),
            "EXECUTING should not preload executor (it's the current model)"
        );

        // AUDITING -> pre-load executor (not auditor, which is current)
        let auditing_models = policy.models_for_state("AUDITING");
        assert!(
            !auditing_models.is_empty(),
            "AUDITING state should have preload targets"
        );
        assert!(
            !auditing_models.contains(&"auditor".to_string()),
            "AUDITING should not preload auditor (it's the current model)"
        );
    }

    #[test]
    fn adversarial_safety_margin_too_small_recovery() {
        // Test: simulate scenario where safety margin is insufficient, verify policy can be adjusted
        let initial_policy = MultiplexingPolicy {
            enabled: true,
            safety_margin_bytes: 500_000_000, // 500MB margin (too small for some models)
            ..Default::default()
        };

        // Model passes check with small margin
        let result = initial_policy.can_colocate(4_000_000_000, 5_000_000_000, 10_000_000_000, 1);
        assert!(result.is_ok());
        assert!(result.unwrap(), "should pass with initial margin");

        // Simulate OOM by increasing margin (recovery strategy)
        let recovered_policy = MultiplexingPolicy {
            enabled: true,
            safety_margin_bytes: 2_000_000_000, // Increased to 2GB
            ..Default::default()
        };

        // Same model now fails with increased margin (safer)
        let result = recovered_policy.can_colocate(4_000_000_000, 5_000_000_000, 10_000_000_000, 1);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "should reject with increased margin (safer)");

        // Verify policy is serializable for dynamic adjustment
        let json = serde_json::to_string(&recovered_policy).expect("serialize");
        let back: MultiplexingPolicy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.safety_margin_bytes, 2_000_000_000);
    }

    #[test]
    fn adversarial_colocated_count_exceeds_limit_with_vram_available() {
        // Test: even with plenty of VRAM, max_colocated limit should be enforced
        let policy = MultiplexingPolicy {
            enabled: true,
            max_colocated: 2,
            safety_margin_bytes: 1_000_000_000,
            ..Default::default()
        };

        // GPU: 100GB, margin: 1GB -> 99GB available
        // Current: 2GB, model: 1GB, colocated: 2 (at limit)
        let result = policy.can_colocate(1_000_000_000, 2_000_000_000, 100_000_000_000, 2);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "should reject when at max_colocated limit despite VRAM availability");
    }
}
