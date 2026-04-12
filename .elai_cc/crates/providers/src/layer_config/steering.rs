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
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{LayerAccessError, LayerAccessProvider};

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// Default injection coefficient (conservative starting point).
pub const COEFFICIENT_DEFAULT: f64 = 5.0;

/// Minimum coefficient (below this, steering effect negligible).
pub const COEFFICIENT_MIN: f64 = 1.0;

/// Maximum coefficient (above this, output degrades to incoherent text).
pub const COEFFICIENT_MAX: f64 = 30.0;

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
    /// Loads pre-computed vectors from storage (.safetensors files) if
    /// `storage_dir` is provided. Validates each vector against the model's
    /// layer count before activation.
    pub fn new(
        config: &SteeringConfig,
        total_layers: usize,
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
                enabled: true,
            });
        }

        // Resolve layers for each spec but don't load data yet —
        // data loading requires a storage path (see load_from_directory).
        for spec in &config.vectors {
            let layer = spec.resolve_layer(total_layers)?;
            if layer >= total_layers {
                return Err(SteeringError::LayerOutOfRange {
                    index: layer,
                    total: total_layers,
                });
            }
            if !(COEFFICIENT_MIN..=COEFFICIENT_MAX).contains(&spec.coefficient) {
                return Err(SteeringError::InvalidCoefficient {
                    coefficient: spec.coefficient,
                    min: COEFFICIENT_MIN,
                    max: COEFFICIENT_MAX,
                });
            }
        }

        tracing::info!(
            count = config.vectors.len(),
            "Steering specs validated; call load_from_directory() to load vector data"
        );

        Ok(Self {
            vectors: Vec::new(),
            enabled: true,
        })
    }

    /// Load pre-computed steering vectors from a directory of .safetensors files.
    ///
    /// File naming convention: `{vector_name}.safetensors`
    /// Each file must contain a tensor named "steering_vector" with shape [hidden_dim].
    pub fn load_from_directory(
        &mut self,
        dir: &Path,
        specs: &[SteeringVectorSpec],
        total_layers: usize,
    ) -> Result<usize, SteeringError> {
        let mut loaded = 0;
        for spec in specs {
            let file_path = dir.join(format!("{}.safetensors", spec.name));
            if !file_path.exists() {
                tracing::debug!(name = %spec.name, "No .safetensors file found, skipping");
                continue;
            }
            match Self::load_single_vector(&file_path, spec, total_layers) {
                Ok(vector) => {
                    tracing::info!(
                        name = %vector.name,
                        layer = vector.layer,
                        coefficient = vector.coefficient,
                        dim = vector.vector.len(),
                        magnitude = vector.magnitude(),
                        "Loaded steering vector"
                    );
                    self.vectors.push(vector);
                    loaded += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        name = %spec.name,
                        error = %e,
                        "Failed to load steering vector, skipping"
                    );
                }
            }
        }
        Ok(loaded)
    }

    /// Load a single steering vector from a .safetensors file.
    fn load_single_vector(
        path: &Path,
        spec: &SteeringVectorSpec,
        total_layers: usize,
    ) -> Result<SteeringVector, SteeringError> {
        let data = std::fs::read(path).map_err(|e| SteeringError::StorageError {
            reason: format!("failed to read {}: {e}", path.display()),
        })?;

        let tensors = safetensors::SafeTensors::deserialize(&data)
            .map_err(|e| SteeringError::StorageError {
                reason: format!("failed to parse safetensors {}: {e}", path.display()),
            })?;

        let tensor = tensors
            .tensor("steering_vector")
            .map_err(|e| SteeringError::StorageError {
                reason: format!(
                    "tensor 'steering_vector' not found in {}: {e}",
                    path.display()
                ),
            })?;

        // Validate dtype is F32 before byte reinterpretation
        if tensor.dtype() != safetensors::Dtype::F32 {
            return Err(SteeringError::StorageError {
                reason: format!(
                    "tensor 'steering_vector' in {} has dtype {:?}, expected F32",
                    path.display(),
                    tensor.dtype()
                ),
            });
        }

        // Convert raw bytes to Vec<f32> (little-endian f32)
        let raw = tensor.data();
        if raw.len() % 4 != 0 {
            return Err(SteeringError::StorageError {
                reason: format!(
                    "tensor data length {} not aligned to f32 (4 bytes)",
                    raw.len()
                ),
            });
        }
        let vector: Vec<f32> = raw
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        let layer = spec.resolve_layer(total_layers)?;

        let sv = SteeringVector {
            name: spec.name.clone(),
            layer,
            coefficient: spec.coefficient,
            vector,
            positive_prompt: spec.positive_prompt.clone(),
            negative_prompt: spec.negative_prompt.clone(),
        };

        sv.validate(total_layers)?;
        Ok(sv)
    }

    /// Check if steering is active (enabled + has valid vectors).
    pub fn is_active(&self) -> bool {
        self.enabled && !self.vectors.is_empty()
    }

    /// Get the list of active steering vectors.
    pub fn vectors(&self) -> &[SteeringVector] {
        &self.vectors
    }

    /// Verify steering is active and vectors are loaded.
    ///
    /// The actual activation injection happens in the Python sidecar during
    /// forward pass. This method validates Rust-side readiness. The sidecar
    /// receives steering config via SidecarConfig (steering_enabled,
    /// steering_coefficient, steering_layer).
    pub fn apply_steering(
        &self,
        _layer_provider: &dyn LayerAccessProvider,
    ) -> Result<(), SteeringError> {
        if !self.is_active() {
            return Err(SteeringError::NoActiveVectors);
        }

        tracing::debug!(
            count = self.vectors.len(),
            "Steering vectors ready; injection delegated to sidecar"
        );
        Ok(())
    }

    /// Request the Python sidecar to compute a steering vector from contrast prompts.
    ///
    /// Sends an HTTP POST to `{sidecar_url}/v1/compute_steering` with the
    /// contrast pair. The sidecar runs the ActAdd algorithm:
    ///   steering_vector = h+(layer) - h-(layer)
    ///
    /// Requires a running sidecar with the /v1/compute_steering endpoint.
    pub async fn compute_vector_via_sidecar(
        spec: &SteeringVectorSpec,
        sidecar_url: &str,
        total_layers: usize,
    ) -> Result<SteeringVector, SteeringError> {
        let layer = spec.resolve_layer(total_layers)?;
        let url = format!("{}/v1/compute_steering", sidecar_url.trim_end_matches('/'));

        let body = serde_json::json!({
            "positive_prompt": spec.positive_prompt,
            "negative_prompt": spec.negative_prompt,
            "layer_idx": layer,
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| SteeringError::StorageError {
                reason: format!("sidecar request failed: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(SteeringError::StorageError {
                reason: format!("sidecar returned {status}: {text}"),
            });
        }

        #[derive(serde::Deserialize)]
        struct ComputeResponse {
            vector: Vec<f32>,
        }

        let parsed: ComputeResponse =
            resp.json().await.map_err(|e| SteeringError::StorageError {
                reason: format!("failed to parse sidecar response: {e}"),
            })?;

        let sv = SteeringVector {
            name: spec.name.clone(),
            layer,
            coefficient: spec.coefficient,
            vector: parsed.vector,
            positive_prompt: spec.positive_prompt.clone(),
            negative_prompt: spec.negative_prompt.clone(),
        };

        sv.validate(total_layers)?;
        Ok(sv)
    }

    /// Legacy sync interface — delegates to sidecar but kept for trait compat.
    pub fn compute_vector(
        &self,
        _spec: &SteeringVectorSpec,
        _layer_provider: &dyn LayerAccessProvider,
    ) -> Result<SteeringVector, SteeringError> {
        Err(SteeringError::StorageError {
            reason: "Use compute_vector_via_sidecar() for async HTTP computation".to_string(),
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
