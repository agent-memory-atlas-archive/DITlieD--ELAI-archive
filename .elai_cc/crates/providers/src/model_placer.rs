//! Model placement strategies for GPU scheduling.
//!
//! Phase 3J.8 deliverable. See docs/specs/guide/steps/phase-3J.txt §3J.8
//! and docs/specs/crates/providers.txt S4 L171-174.
//!
//! Provides greedy baseline placement and optional MILP-based optimal placement.

use crate::multiplexing::{MultiplexingError, MultiplexingPolicy, SloTier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Model Placement Types
// ---------------------------------------------------------------------------

/// Represents a model to be placed on a GPU.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    /// Unique identifier for the model.
    pub id: String,
    /// VRAM required by the model in bytes.
    pub vram_bytes: u64,
    /// SLO quality tier for this model.
    pub tier: SloTier,
    /// Whether this model is needed concurrently with others in the current phase.
    pub concurrent: bool,
}

impl ModelSpec {
    /// Create a new model specification.
    #[must_use]
    pub fn new(id: impl Into<String>, vram_bytes: u64, tier: SloTier) -> Self {
        Self {
            id: id.into(),
            vram_bytes,
            tier,
            concurrent: false,
        }
    }

    /// Mark this model as needed concurrently.
    #[must_use]
    pub fn with_concurrent(mut self) -> Self {
        self.concurrent = true;
        self
    }
}

/// Represents a GPU in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSpec {
    /// GPU identifier (0-indexed).
    pub id: u8,
    /// Total VRAM capacity in bytes.
    pub total_vram_bytes: u64,
    /// Current VRAM usage in bytes.
    pub current_vram_used_bytes: u64,
    /// Number of models currently colocated on this GPU.
    pub colocated_count: usize,
}

impl GpuSpec {
    /// Create a new GPU specification.
    #[must_use]
    pub fn new(id: u8, total_vram_bytes: u64) -> Self {
        Self {
            id,
            total_vram_bytes,
            current_vram_used_bytes: 0,
            colocated_count: 0,
        }
    }

    /// Get available VRAM (total - current).
    #[must_use]
    pub fn available_vram(&self) -> u64 {
        self.total_vram_bytes.saturating_sub(self.current_vram_used_bytes)
    }
}

/// Result of model placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementResult {
    /// Map from model ID to assigned GPU ID.
    pub assignments: HashMap<String, u8>,
    /// Total swap latency estimate in milliseconds.
    pub total_swap_latency_ms: u64,
    /// Whether placement succeeded (all models assigned).
    pub success: bool,
    /// Error message if placement failed.
    pub error: Option<String>,
}

impl PlacementResult {
    /// Create a successful placement result.
    #[must_use]
    pub fn success(assignments: HashMap<String, u8>, total_swap_latency_ms: u64) -> Self {
        Self {
            assignments,
            total_swap_latency_ms,
            success: true,
            error: None,
        }
    }

    /// Create a failed placement result.
    #[must_use]
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            assignments: HashMap::new(),
            total_swap_latency_ms: 0,
            success: false,
            error: Some(error.into()),
        }
    }
}

// ---------------------------------------------------------------------------
// Greedy Model Placer (Baseline)
// ---------------------------------------------------------------------------

/// Greedy model placement strategy.
///
/// Algorithm: sort models by VRAM descending, assign each to GPU with most free space
/// that respects multiplexing policy constraints.
///
/// Source: phase-3J.txt §3J.8, providers.txt S4 L171-174.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreedyModelPlacer {
    /// Multiplexing policy for colocation decisions.
    pub policy: MultiplexingPolicy,
    /// Estimated swap latency per model in milliseconds (constant for greedy).
    pub swap_latency_ms_per_model: u64,
}

impl Default for GreedyModelPlacer {
    fn default() -> Self {
        Self {
            policy: MultiplexingPolicy::default(),
            swap_latency_ms_per_model: 100, // 100ms per model swap (conservative estimate)
        }
    }
}

impl GreedyModelPlacer {
    /// Create a new greedy model placer with custom policy.
    #[must_use]
    pub fn with_policy(policy: MultiplexingPolicy) -> Self {
        Self {
            policy,
            ..Default::default()
        }
    }

    /// Place models on GPUs using greedy strategy.
    ///
    /// # Arguments
    /// * `models` - Models to place
    /// * `gpus` - Available GPUs
    ///
    /// # Returns
    /// Placement result with assignments and estimated swap latency.
    pub fn place(
        &self,
        models: &[ModelSpec],
        gpus: &[GpuSpec],
    ) -> Result<PlacementResult, MultiplexingError> {
        if models.is_empty() {
            return Ok(PlacementResult::success(HashMap::new(), 0));
        }

        if gpus.is_empty() {
            return Ok(PlacementResult::failure("No GPUs available"));
        }

        // Sort models by VRAM descending (largest first)
        let mut sorted_models: Vec<_> = models.to_vec();
        sorted_models.sort_by(|a, b| b.vram_bytes.cmp(&a.vram_bytes));

        // Track GPU state during placement
        let mut gpu_state: Vec<_> = gpus.to_vec();
        let mut assignments = HashMap::new();
        let mut swap_count = 0;

        for model in &sorted_models {
            // Find GPU with most free space that can accommodate this model
            let best_gpu = self.find_best_gpu(&model, &gpu_state)?;

            if let Some(gpu_idx) = best_gpu {
                // Assign model to this GPU
                assignments.insert(model.id.clone(), gpu_state[gpu_idx].id);
                gpu_state[gpu_idx].current_vram_used_bytes += model.vram_bytes;
                gpu_state[gpu_idx].colocated_count += 1;
                swap_count += 1;
            } else {
                // No GPU can accommodate this model
                return Ok(PlacementResult::failure(format!(
                    "No GPU can accommodate model {} ({} bytes)",
                    model.id, model.vram_bytes
                )));
            }
        }

        // Estimate total swap latency
        let total_swap_latency_ms = swap_count as u64 * self.swap_latency_ms_per_model;

        Ok(PlacementResult::success(assignments, total_swap_latency_ms))
    }

    /// Find the best GPU for a model (most free space that respects policy).
    fn find_best_gpu(
        &self,
        model: &ModelSpec,
        gpus: &[GpuSpec],
    ) -> Result<Option<usize>, MultiplexingError> {
        let mut best_gpu_idx: Option<usize> = None;
        let mut max_free_space: u64 = 0;

        for (idx, gpu) in gpus.iter().enumerate() {
            // Check if model can colocate on this GPU
            let can_place = self.policy.can_colocate(
                model.vram_bytes,
                gpu.current_vram_used_bytes,
                gpu.total_vram_bytes,
                gpu.colocated_count,
            )?;

            if can_place {
                let free_space = gpu.available_vram();
                if free_space > max_free_space {
                    max_free_space = free_space;
                    best_gpu_idx = Some(idx);
                }
            }
        }

        Ok(best_gpu_idx)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // ModelSpec tests
    // -----------------------------------------------------------------------

    #[test]
    fn model_spec_creation() {
        let spec = ModelSpec::new("model1", 4_000_000_000, SloTier::Best);
        assert_eq!(spec.id, "model1");
        assert_eq!(spec.vram_bytes, 4_000_000_000);
        assert_eq!(spec.tier, SloTier::Best);
        assert!(!spec.concurrent);
    }

    #[test]
    fn model_spec_with_concurrent() {
        let spec = ModelSpec::new("model1", 4_000_000_000, SloTier::Best).with_concurrent();
        assert!(spec.concurrent);
    }

    // -----------------------------------------------------------------------
    // GpuSpec tests
    // -----------------------------------------------------------------------

    #[test]
    fn gpu_spec_creation() {
        let gpu = GpuSpec::new(0, 12_000_000_000);
        assert_eq!(gpu.id, 0);
        assert_eq!(gpu.total_vram_bytes, 12_000_000_000);
        assert_eq!(gpu.current_vram_used_bytes, 0);
        assert_eq!(gpu.colocated_count, 0);
    }

    #[test]
    fn gpu_spec_available_vram() {
        let gpu = GpuSpec {
            id: 0,
            total_vram_bytes: 12_000_000_000,
            current_vram_used_bytes: 5_000_000_000,
            colocated_count: 1,
        };
        assert_eq!(gpu.available_vram(), 7_000_000_000);
    }

    // -----------------------------------------------------------------------
    // PlacementResult tests
    // -----------------------------------------------------------------------

    #[test]
    fn placement_result_success() {
        let mut assignments = HashMap::new();
        assignments.insert("model1".to_string(), 0);
        let result = PlacementResult::success(assignments, 100);
        assert!(result.success);
        assert_eq!(result.total_swap_latency_ms, 100);
        assert!(result.error.is_none());
    }

    #[test]
    fn placement_result_failure() {
        let result = PlacementResult::failure("No space");
        assert!(!result.success);
        assert_eq!(result.error, Some("No space".to_string()));
    }

    // -----------------------------------------------------------------------
    // GreedyModelPlacer tests
    // -----------------------------------------------------------------------

    #[test]
    fn greedy_placer_default() {
        let placer = GreedyModelPlacer::default();
        assert!(!placer.policy.enabled);
        assert_eq!(placer.swap_latency_ms_per_model, 100);
    }

    #[test]
    fn greedy_placer_place_single_model() {
        let placer = GreedyModelPlacer::default();
        let models = vec![ModelSpec::new("model1", 4_000_000_000, SloTier::Best)];
        let gpus = vec![GpuSpec::new(0, 12_000_000_000)];

        let result = placer.place(&models, &gpus).unwrap();
        assert!(result.success);
        assert_eq!(result.assignments.get("model1"), Some(&0));
        assert_eq!(result.total_swap_latency_ms, 100);
    }

    #[test]
    fn greedy_placer_place_multiple_models() {
        let policy = MultiplexingPolicy {
            enabled: true, // Enable multiplexing for this test
            ..Default::default()
        };
        let placer = GreedyModelPlacer::with_policy(policy);
        let models = vec![
            ModelSpec::new("large", 6_000_000_000, SloTier::Best),
            ModelSpec::new("small", 2_000_000_000, SloTier::Basic),
        ];
        let gpus = vec![GpuSpec::new(0, 12_000_000_000)];

        let result = placer.place(&models, &gpus).unwrap();
        assert!(result.success);
        assert_eq!(result.assignments.len(), 2);
        assert_eq!(result.total_swap_latency_ms, 200);
    }

    #[test]
    fn greedy_placer_respects_vram_limit() {
        let placer = GreedyModelPlacer::default();
        let models = vec![
            ModelSpec::new("model1", 6_000_000_000, SloTier::Best),
            ModelSpec::new("model2", 6_000_000_000, SloTier::Best),
        ];
        let gpus = vec![GpuSpec::new(0, 12_000_000_000)];

        let result = placer.place(&models, &gpus).unwrap();
        // Should fail because 12GB total < 6GB + 6GB + safety margin (1GB)
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn greedy_placer_sorts_by_vram_descending() {
        let policy = MultiplexingPolicy {
            enabled: true, // Enable multiplexing for this test
            ..Default::default()
        };
        let placer = GreedyModelPlacer::with_policy(policy);
        let models = vec![
            ModelSpec::new("small", 2_000_000_000, SloTier::Basic),
            ModelSpec::new("large", 5_000_000_000, SloTier::Best),
            ModelSpec::new("medium", 3_000_000_000, SloTier::Good),
        ];
        let gpus = vec![GpuSpec::new(0, 12_000_000_000)];

        let result = placer.place(&models, &gpus).unwrap();
        assert!(result.success);
        // All models should be placed (5GB + 3GB + 2GB = 10GB, fits within 11GB with 1GB margin)
        assert_eq!(result.assignments.len(), 3);
    }

    #[test]
    fn greedy_placer_empty_models() {
        let placer = GreedyModelPlacer::default();
        let models: Vec<ModelSpec> = vec![];
        let gpus = vec![GpuSpec::new(0, 12_000_000_000)];

        let result = placer.place(&models, &gpus).unwrap();
        assert!(result.success);
        assert!(result.assignments.is_empty());
        assert_eq!(result.total_swap_latency_ms, 0);
    }

    #[test]
    fn greedy_placer_no_gpus() {
        let placer = GreedyModelPlacer::default();
        let models = vec![ModelSpec::new("model1", 4_000_000_000, SloTier::Best)];
        let gpus: Vec<GpuSpec> = vec![];

        let result = placer.place(&models, &gpus).unwrap();
        assert!(!result.success);
        assert_eq!(result.error, Some("No GPUs available".to_string()));
    }

    // -----------------------------------------------------------------------
    // Adversarial tests (P3J.8 ADVERSARIAL requirements)
    // -----------------------------------------------------------------------

    #[test]
    fn adversarial_greedy_placer_vram_overflow_detection() {
        // Test: verify placer detects when model exceeds GPU capacity
        let policy = MultiplexingPolicy {
            enabled: true, // Enable multiplexing for this test
            ..Default::default()
        };
        let placer = GreedyModelPlacer::with_policy(policy);
        let models = vec![ModelSpec::new("huge", 100_000_000_000, SloTier::Best)];
        let gpus = vec![GpuSpec::new(0, 12_000_000_000)];

        let result = placer.place(&models, &gpus).unwrap();
        assert!(!result.success, "should reject model larger than GPU");
        assert!(
            result.error.as_ref().unwrap().contains("No GPU can accommodate"),
            "error should indicate capacity issue"
        );
    }

    #[test]
    fn adversarial_greedy_placer_multiplexing_disabled() {
        // Test: when multiplexing disabled, only one model per GPU
        let policy = MultiplexingPolicy {
            enabled: false,
            ..Default::default()
        };
        let placer = GreedyModelPlacer::with_policy(policy);
        let models = vec![
            ModelSpec::new("model1", 2_000_000_000, SloTier::Basic),
            ModelSpec::new("model2", 2_000_000_000, SloTier::Basic),
        ];
        let gpus = vec![GpuSpec::new(0, 12_000_000_000)];

        let result = placer.place(&models, &gpus).unwrap();
        assert!(!result.success, "should reject second model when multiplexing disabled");
    }

    #[test]
    fn adversarial_greedy_placer_max_colocated_limit() {
        // Test: respect max_colocated limit even with VRAM available
        let policy = MultiplexingPolicy {
            enabled: true,
            max_colocated: 2,
            ..Default::default()
        };
        let placer = GreedyModelPlacer::with_policy(policy);
        let models = vec![
            ModelSpec::new("model1", 1_000_000_000, SloTier::Basic),
            ModelSpec::new("model2", 1_000_000_000, SloTier::Basic),
            ModelSpec::new("model3", 1_000_000_000, SloTier::Basic),
        ];
        let gpus = vec![GpuSpec::new(0, 100_000_000_000)]; // Plenty of VRAM

        let result = placer.place(&models, &gpus).unwrap();
        assert!(!result.success, "should reject third model at max_colocated limit");
    }

    #[test]
    fn adversarial_greedy_placer_safety_margin_enforcement() {
        // Test: safety margin prevents colocation that would be too tight
        let policy = MultiplexingPolicy {
            enabled: true,
            safety_margin_bytes: 2_000_000_000, // 2GB margin
            ..Default::default()
        };
        let placer = GreedyModelPlacer::with_policy(policy);
        let models = vec![
            ModelSpec::new("model1", 5_000_000_000, SloTier::Basic),
            ModelSpec::new("model2", 5_000_000_000, SloTier::Basic),
        ];
        let gpus = vec![GpuSpec::new(0, 12_000_000_000)];

        // Total: 10GB, GPU: 12GB, margin: 2GB -> available: 10GB
        // 5GB + 5GB = 10GB, fits exactly with margin
        let result = placer.place(&models, &gpus).unwrap();
        assert!(result.success, "should accept exact fit with margin");

        // Third model should fail
        let models = vec![
            ModelSpec::new("model1", 5_000_000_000, SloTier::Basic),
            ModelSpec::new("model2", 5_000_000_000, SloTier::Basic),
            ModelSpec::new("model3", 1_000_000_000, SloTier::Basic),
        ];
        let result = placer.place(&models, &gpus).unwrap();
        assert!(!result.success, "should reject when margin exceeded");
    }

    #[test]
    fn adversarial_greedy_placer_zero_vram_model() {
        // Test: reject model with zero VRAM (invalid)
        let placer = GreedyModelPlacer::default();
        let models = vec![ModelSpec::new("invalid", 0, SloTier::Basic)];
        let gpus = vec![GpuSpec::new(0, 12_000_000_000)];

        let result = placer.place(&models, &gpus);
        assert!(result.is_err(), "should return error for zero VRAM model");
    }

    #[test]
    fn adversarial_greedy_placer_zero_gpu_capacity() {
        // Test: reject GPU with zero capacity (invalid)
        let placer = GreedyModelPlacer::default();
        let models = vec![ModelSpec::new("model1", 4_000_000_000, SloTier::Basic)];
        let gpus = vec![GpuSpec::new(0, 0)];

        let result = placer.place(&models, &gpus);
        assert!(result.is_err(), "should return error for zero GPU capacity");
    }
}
