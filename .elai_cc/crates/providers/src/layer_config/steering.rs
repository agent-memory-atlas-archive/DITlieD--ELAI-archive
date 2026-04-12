//! Steering Vectors (ActAdd): Activation Addition for inference-time control.
//!
//! Implementation of ActAdd algorithm (arxiv 2308.10248, Turner et al. 2024).
//! Computes steering vectors from contrast prompt pairs and injects them into
//! the residual stream at target layers during inference.
//!
//! Key invariants (from paper):
//! - Steering vector = h+^l - h-^l (difference of activations from contrast prompts)
//! - Injection: residual_modified = h^l + c * h_A^l at target layer l
//! - Early-mid layers (6-20) peak effectiveness; late layers (25+) declining
//! - Coefficient range [3, 20]; start conservative (c=5)
//! - Multiple vectors composable (additive)
//! - Requires intermediate activation access (local models only, no API models)

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{LayerAccessError, LayerAccessProvider};

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// Default injection coefficient (conservative starting point).
const COEFFICIENT_DEFAULT: f64 = 5.0;

/// Minimum coefficient (below this, steering effect negligible).
const COEFFICIENT_MIN: f64 = 1.0;

/// Maximum coefficient (above this, output degrades to incoherent text).
const COEFFICIENT_MAX: f64 = 30.0;

/// Default target layer: model_layers / 3 (early-mid range where steering is effective).
const LAYER_DEFAULT_FRACTION: f64 = 1.0 / 3.0;

/// Early layer threshold (below this, minimal effect per ActAdd paper Section 6.3).
#[allow(dead_code)]
const LAYER_EARLY_THRESHOLD: usize = 5;

/// Late layer threshold (above this, declining effectiveness per ActAdd paper Section 6.3).
#[allow(dead_code)]
const LAYER_LATE_THRESHOLD: usize = 25;

// ---------------------------------------------------------------------------
// SteeringError
// ---------------------------------------------------------------------------

/// Errors from steering vector operations.
#[derive(Debug, thiserror::Error)]
pub enum SteeringError {
    /// No steering vectors configured or all disabled.
    #[error("no active steering vectors")]
    NoActiveVectors,

    /// Steering vector has zero magnitude (invalid contrast pair).
    #[error("steering vector '{name}' has zero magnitude (contrast pair produced no difference)")]
    ZeroMagnitude { name: String },

    /// Layer index out of range for the model.
    #[error("layer index {index} out of range for model with {total} layers")]
    LayerOutOfRange { index: usize, total: usize },

    /// Coefficient outside valid range.
    #[error("coefficient {coefficient} outside valid range [{min}, {max}]")]
    InvalidCoefficient {
        coefficient: f64,
        min: f64,
        max: f64,
    },

    /// Layer access backend error.
    #[error("layer access error: {0}")]
    LayerAccess(#[from] LayerAccessError),

    /// Vector storage error (e.g., failed to load from .safetensors).
    #[error("vector storage error: {reason}")]
    StorageError { reason: String },
}

// ---------------------------------------------------------------------------
// SteeringVector
// ---------------------------------------------------------------------------

/// A pre-computed steering vector for a specific contrast pair.
///
/// The vector is computed once (amortized) by:
/// 1. Forward model on positive prompt p+, record activations at layer l -> h+
/// 2. Forward model on negative prompt p-, record activations at layer l -> h-
/// 3. Steering vector = h+ - h-
///
/// Stored as .safetensors or .npz for reuse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringVector {
    /// Human-readable name for this vector (e.g., "skepticism", "completeness").
    pub name: String,

    /// Target layer index where this vector is injected.
    pub layer: usize,

    /// Injection coefficient c (typical range [3, 20]).
    pub coefficient: f64,

    /// The steering vector itself (difference of activations h+ - h-).
    /// Dimension = model's hidden dimension at target layer.
    pub vector: Vec<f32>,

    /// Positive prompt used to compute h+ (for documentation/re-computation).
    pub positive_prompt: String,

    /// Negative prompt used to compute h- (for documentation/re-computation).
    pub negative_prompt: String,
}

impl SteeringVector {
    /// Validate that this steering vector is well-formed.
    pub fn validate(&self, total_layers: usize) -> Result<(), SteeringError> {
        // Check layer range
        if self.layer >= total_layers {
            return Err(SteeringError::LayerOutOfRange {
                index: self.layer,
                total: total_layers,
            });
        }

        // Check coefficient range
        if !(COEFFICIENT_MIN..=COEFFICIENT_MAX).contains(&self.coefficient) {
            return Err(SteeringError::InvalidCoefficient {
                coefficient: self.coefficient,
                min: COEFFICIENT_MIN,
                max: COEFFICIENT_MAX,
            });
        }

        // Check vector magnitude (zero vector = invalid contrast pair)
        let magnitude_sq: f32 = self.vector.iter().map(|&x| x * x).sum();
        if magnitude_sq < 1e-6 {
            return Err(SteeringError::ZeroMagnitude {
                name: self.name.clone(),
            });
        }

        Ok(())
    }

    /// Compute the L2 norm (magnitude) of this steering vector.
    pub fn magnitude(&self) -> f32 {
        self.vector.iter().map(|&x| x * x).sum::<f32>().sqrt()
    }
}

// ---------------------------------------------------------------------------
// SteeringVectorSpec
// ---------------------------------------------------------------------------

/// Specification for a steering vector (used in config, before computation).
///
/// This is the configuration format for elai_cc.toml. After vector computation,
/// the actual vector data is stored in a `SteeringVector`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringVectorSpec {
    /// Human-readable name for this vector.
    pub name: String,

    /// Target layer index (or use "auto" for model_layers / 3).
    #[serde(default = "default_layer_auto")]
    pub layer: String,

    /// Injection coefficient (default 5.0).
    #[serde(default = "default_coefficient")]
    pub coefficient: f64,

    /// Positive prompt for contrast pair.
    pub positive_prompt: String,

    /// Negative prompt for contrast pair.
    pub negative_prompt: String,
}

fn default_layer_auto() -> String {
    "auto".to_string()
}

fn default_coefficient() -> f64 {
    COEFFICIENT_DEFAULT
}

impl SteeringVectorSpec {
    /// Resolve the layer index from the spec string.
    ///
    /// - "auto" -> model_layers / 3
    /// - Numeric string -> parse as usize
    pub fn resolve_layer(&self, total_layers: usize) -> Result<usize, SteeringError> {
        if self.layer == "auto" {
            let layer = (total_layers as f64 * LAYER_DEFAULT_FRACTION) as usize;
            Ok(layer)
        } else {
            self.layer
                .parse::<usize>()
                .map_err(|_| SteeringError::LayerOutOfRange {
                    index: 0,
                    total: total_layers,
                })
        }
    }
}

// ---------------------------------------------------------------------------
// SteeringConfig
// ---------------------------------------------------------------------------

/// Configuration for steering vectors per agent role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringConfig {
    /// Whether steering is enabled for this role.
    #[serde(default)]
    pub enabled: bool,

    /// List of steering vector specifications.
    #[serde(default)]
    pub vectors: Vec<SteeringVectorSpec>,
}

impl Default for SteeringConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            vectors: Vec::new(),
        }
    }
}

impl SteeringConfig {
    /// Get default configuration for a specific agent role.
    ///
    /// - executor: No steering by default (needs full autonomy)
    /// - auditor: Skepticism steering (question assumptions)
    /// - prompt-engineer: Completeness steering (ensure thorough prompts)
    /// - context-compiler: Fidelity steering (preserve context integrity)
    pub fn default_for_role(role: &str) -> Self {
        match role {
            "auditor" => Self {
                enabled: true,
                vectors: vec![SteeringVectorSpec {
                    name: "skepticism".to_string(),
                    layer: "auto".to_string(),
                    coefficient: 5.0,
                    positive_prompt: "I should question assumptions and verify claims.".to_string(),
                    negative_prompt: "I should accept information without questioning.".to_string(),
                }],
            },
            "prompt-engineer" => Self {
                enabled: true,
                vectors: vec![SteeringVectorSpec {
                    name: "completeness".to_string(),
                    layer: "auto".to_string(),
                    coefficient: 5.0,
                    positive_prompt: "I should provide complete and thorough information.".to_string(),
                    negative_prompt: "I should provide minimal information.".to_string(),
                }],
            },
            "context-compiler" => Self {
                enabled: true,
                vectors: vec![SteeringVectorSpec {
                    name: "fidelity".to_string(),
                    layer: "auto".to_string(),
                    coefficient: 5.0,
                    positive_prompt: "I should preserve context faithfully.".to_string(),
                    negative_prompt: "I should modify or simplify context.".to_string(),
                }],
            },
            _ => Self::default(), // executor and others: no steering
        }
    }
}

// ---------------------------------------------------------------------------
// SteeringVectorEngine
// ---------------------------------------------------------------------------

/// Engine for managing and applying steering vectors during inference.
///
/// This engine:
/// - Loads pre-computed steering vectors from storage
/// - Validates vectors against model constraints
/// - Applies steering by injecting into the residual stream at target layers
/// - Supports composable multi-vector steering
///
/// NOTE: This is a Rust-side orchestration layer. The actual activation injection
/// happens in the inference backend (vLLM plugin or custom hook). The engine provides
/// the vector data and configuration to the backend.
#[derive(Debug)]
pub struct SteeringVectorEngine {
    /// Pre-computed steering vectors (loaded from storage).
    vectors: Vec<SteeringVector>,

    /// Whether steering is enabled.
    enabled: bool,
}

impl SteeringVectorEngine {
    /// Create a new steering engine from configuration.
    ///
    /// This loads pre-computed vectors from storage (.safetensors or .npz files)
    /// and validates them against the model's layer count.
    pub fn new(
        config: &SteeringConfig,
        _total_layers: usize,
    ) -> Result<Self, SteeringError> {
        if !config.enabled {
            return Ok(Self {
                vectors: Vec::new(),
                enabled: false,
            });
        }

        if config.vectors.is_empty() {
            return Ok(Self {
                vectors: Vec::new(),
                enabled: true, // Enabled but no vectors configured
            });
        }

        // Load vectors from storage
        // TODO: Implement vector loading from .safetensors/.npz files
        // For now, we'll return an empty engine with a note that vectors need to be computed
        tracing::warn!(
            "Steering vectors configured but not yet loaded from storage. \
             Vector computation (E2B testing) is deferred to step 3J.5 implementation completion."
        );

        Ok(Self {
            vectors: Vec::new(),
            enabled: true,
        })
    }

    /// Check if steering is active (enabled + has valid vectors).
    pub fn is_active(&self) -> bool {
        self.enabled && !self.vectors.is_empty()
    }

    /// Get the list of active steering vectors.
    pub fn vectors(&self) -> &[SteeringVector] {
        &self.vectors
    }

    /// Apply steering to the residual stream at the target layer.
    ///
    /// This is a placeholder for the actual injection logic, which requires
    /// integration with the vLLM backend's layer-access hook (same as DoLa).
    ///
    /// The backend will:
    /// 1. Receive the steering vector data from this engine
    /// 2. At the target layer during forward pass: residual = residual + c * vector
    /// 3. Continue forward pass with modified residual
    ///
    /// Returns an error if steering is not active or if layer access fails.
    pub fn apply_steering(
        &self,
        _layer_provider: &dyn LayerAccessProvider,
    ) -> Result<(), SteeringError> {
        if !self.is_active() {
            return Err(SteeringError::NoActiveVectors);
        }

        // TODO: Integrate with vLLM backend's layer-access hook
        // The actual injection happens in the backend, not here.
        // This method validates that steering can proceed and prepares the vector data.
        tracing::warn!(
            "Steering vector application requires vLLM backend integration. \
             Coupled with DoLa (step 3J.3) - if DoLa hook works, steering reuses the same mechanism."
        );

        Ok(())
    }

    /// Compute a steering vector from a contrast prompt pair.
    ///
    /// This is the one-time computation step (amortized over reuse):
    /// 1. Forward model on positive prompt p+, record activations at layer l -> h+
    /// 2. Forward model on negative prompt p-, record activations at layer l -> h-
    /// 3. Steering vector = h+ - h-
    /// 4. Store as .safetensors or .npz for reuse
    ///
    /// NOTE: This requires access to the model's forward pass and intermediate activations.
    /// It should be run during E2B testing (step 3J.5) on Gemma4 E2B via vLLM.
    ///
    /// Returns the computed steering vector for validation and storage.
    pub fn compute_vector(
        &self,
        _spec: &SteeringVectorSpec,
        _layer_provider: &dyn LayerAccessProvider,
    ) -> Result<SteeringVector, SteeringError> {
        // TODO: Implement vector computation using vLLM backend
        // This requires:
        // 1. Run forward pass on positive prompt, capture layer activations
        // 2. Run forward pass on negative prompt, capture layer activations
        // 3. Compute difference h+ - h-
        // 4. Validate magnitude (non-zero)
        // 5. Store to .safetensors/.npz

        tracing::warn!(
            "Steering vector computation requires vLLM backend integration. \
             Deferred to E2B testing phase (Gemma4 E2B on RTX 3060)."
        );

        Err(SteeringError::StorageError {
            reason: "Vector computation not yet implemented".to_string(),
        })
    }
}

impl fmt::Display for SteeringVectorEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SteeringVectorEngine(enabled={}, vectors={})",
            self.enabled,
            self.vectors.len()
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steering_vector_validation() {
        let vector = SteeringVector {
            name: "test".to_string(),
            layer: 10,
            coefficient: 5.0,
            vector: vec![0.1, 0.2, 0.3],
            positive_prompt: "positive".to_string(),
            negative_prompt: "negative".to_string(),
        };

        assert!(vector.validate(20).is_ok());
        assert!(vector.validate(10).is_err()); // Layer out of range
    }

    #[test]
    fn test_steering_vector_zero_magnitude() {
        let vector = SteeringVector {
            name: "zero".to_string(),
            layer: 10,
            coefficient: 5.0,
            vector: vec![0.0, 0.0, 0.0],
            positive_prompt: "positive".to_string(),
            negative_prompt: "negative".to_string(),
        };

        assert!(matches!(
            vector.validate(20),
            Err(SteeringError::ZeroMagnitude { .. })
        ));
    }

    #[test]
    fn test_coefficient_validation() {
        let mut vector = SteeringVector {
            name: "test".to_string(),
            layer: 10,
            coefficient: 5.0,
            vector: vec![0.1, 0.2, 0.3],
            positive_prompt: "positive".to_string(),
            negative_prompt: "negative".to_string(),
        };

        assert!(vector.validate(20).is_ok());

        vector.coefficient = 0.5; // Below minimum
        assert!(matches!(
            vector.validate(20),
            Err(SteeringError::InvalidCoefficient { .. })
        ));

        vector.coefficient = 35.0; // Above maximum
        assert!(matches!(
            vector.validate(20),
            Err(SteeringError::InvalidCoefficient { .. })
        ));
    }

    #[test]
    fn test_spec_resolve_layer_auto() {
        let spec = SteeringVectorSpec {
            name: "test".to_string(),
            layer: "auto".to_string(),
            coefficient: 5.0,
            positive_prompt: "positive".to_string(),
            negative_prompt: "negative".to_string(),
        };

        let layer = spec.resolve_layer(30).unwrap();
        // 30 * 1/3 = 10
        assert_eq!(layer, 10);
    }

    #[test]
    fn test_spec_resolve_layer_numeric() {
        let spec = SteeringVectorSpec {
            name: "test".to_string(),
            layer: "15".to_string(),
            coefficient: 5.0,
            positive_prompt: "positive".to_string(),
            negative_prompt: "negative".to_string(),
        };

        let layer = spec.resolve_layer(30).unwrap();
        assert_eq!(layer, 15);
    }

    #[test]
    fn test_engine_disabled() {
        let config = SteeringConfig {
            enabled: false,
            vectors: vec![],
        };

        let engine = SteeringVectorEngine::new(&config, 30).unwrap();
        assert!(!engine.enabled);
        assert!(!engine.is_active());
    }

    #[test]
    fn test_engine_enabled_no_vectors() {
        let config = SteeringConfig {
            enabled: true,
            vectors: vec![],
        };

        let engine = SteeringVectorEngine::new(&config, 30).unwrap();
        assert!(engine.enabled);
        assert!(!engine.is_active());
    }
}
