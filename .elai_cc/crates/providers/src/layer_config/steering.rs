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
use std::path::{Path, PathBuf};

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
            // NARROWING-OK: f64->usize; no TryFrom<f64> for usize in std; result is fraction of
            // total_layers which is always a valid array index when total_layers > 0.
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

    // ---------------------------------------------------------------------
    // Batch-ful-c (A-8) — role-scoped sidecar compute with on-disk cache.
    //
    // Additive sibling of the legacy `compute_vector_via_sidecar` — takes a
    // `ContrastPairs` bundle, a role label, and an absolute cache path (see
    // `crate::steering_cache_path` for the canonical layout). On success
    // writes the raw little-endian f32 bytes to the cache path AND returns
    // the fully-populated `SteeringVector`. Callers emit an
    // `SteeringComputed` SSB row; on any failure path they emit
    // `SteeringComputeFailed` with a classifier reason.
    //
    // Wire format (matches the new `/v1/steering/compute` Python endpoint):
    //   Request:  { "role": str, "positive_pairs": [str], "negative_pairs":
    //               [str], "layer": int | null, "project_id": str }
    //   Response: { "vector": [f32], "vector_dim": int, "layer": int,
    //               "norm": float, "model": str, "persisted_path": str }
    //
    // We decode the JSON array directly — no base64 path — because
    // `orjson`/serde_json already stream f32 natively and bringing in a
    // base64 crate would violate the no-new-deps rule in the batch brief.
    // ---------------------------------------------------------------------

    /// Canonical request body for the new `/v1/steering/compute` endpoint.
    /// Kept private so callers can't drift from the wire format.
    fn build_compute_request_body(
        role: &str,
        pairs: &ContrastPairs,
        layer: Option<u32>,
        project_id: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "role": role,
            "positive_pairs": pairs.positive,
            "negative_pairs": pairs.negative,
            "layer": layer,
            "project_id": project_id,
        })
    }

    /// Batch-ful-c — async sibling of `compute_vector_via_sidecar` that
    /// accepts a `ContrastPairs` bundle and persists the result to `cache_path`.
    ///
    /// Contract highlights (see `fulc_steering_vector_contract.rs`):
    /// * Connection refused → `Err(SteeringError::StorageError { reason })`.
    ///   No partial cache file is written.
    /// * HTTP != 200 → `Err` with the upstream status + body.
    /// * Zero-magnitude vector → `Err(SteeringError::ZeroMagnitude)`.
    /// * Successful write → the file at `cache_path` is exactly
    ///   `len * 4` bytes, little-endian f32, no header.
    ///
    /// `total_layers` is the model's hidden-layer count; used only for
    /// validation. The sidecar picks the actual layer index (default =
    /// `n_layers / 3`) and returns it in the response so we can store it
    /// alongside the vector.
    pub async fn compute_vector_via_sidecar_role(
        sidecar_url: &str,
        role: &str,
        pairs: &ContrastPairs,
        total_layers: usize,
        cache_path: &Path,
    ) -> Result<SteeringVector, SteeringError> {
        if pairs.positive.is_empty() || pairs.negative.is_empty() {
            return Err(SteeringError::StorageError {
                reason: "empty_pairs: positive and negative pair lists must be non-empty"
                    .to_owned(),
            });
        }

        // Derive a project_id from the cache path — the canonical layout is
        // `<project_root>/.elai_cc/steering/<project_id>/<role>.bin`, so the
        // parent-of-parent gives us the project_id component. Falls back to
        // "global" when the path doesn't follow the layout (e.g. in tests
        // that pass a tempdir directly).
        let project_id = cache_path
            .parent()
            .and_then(Path::file_name)
            .and_then(|s| s.to_str())
            .unwrap_or("global")
            .to_owned();

        let url = format!("{}/v1/steering/compute", sidecar_url.trim_end_matches('/'));
        let body = Self::build_compute_request_body(role, pairs, None, &project_id);

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| SteeringError::StorageError {
                reason: format!("reqwest client build failed: {e}"),
            })?;

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| SteeringError::StorageError {
                reason: format!("sidecar_unreachable: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(SteeringError::StorageError {
                reason: format!("http_error: {status}: {text}"),
            });
        }

        #[derive(serde::Deserialize)]
        struct ComputeResponse {
            vector: Vec<f32>,
            #[serde(default)]
            vector_dim: Option<u32>,
            #[serde(default)]
            layer: Option<u32>,
            /// Optional server-side norm — used only as a consistency check
            /// against the locally-computed magnitude so we notice drift
            /// between the sidecar's and the Rust side's f32 summation.
            #[serde(default)]
            norm: Option<f32>,
            #[serde(default)]
            model: Option<String>,
        }

        let parsed: ComputeResponse = resp.json().await.map_err(|e| SteeringError::StorageError {
            reason: format!("http_error: failed to parse compute response: {e}"),
        })?;

        // NARROWING-OK: u32->usize (widening on 64-bit) and f64->usize (no TryFrom<f64> in std; fraction of total_layers).
        let layer = parsed.layer.map(|l| l as usize).unwrap_or_else(|| {
            // Fall back to the same default the sidecar uses so the cached
            // metadata stays consistent when the response omits the layer.
            (total_layers as f64 * LAYER_DEFAULT_FRACTION) as usize
        });

        let dim = parsed.vector_dim.map(|d| d as usize).unwrap_or(parsed.vector.len());
        if dim != parsed.vector.len() {
            return Err(SteeringError::StorageError {
                reason: format!(
                    "http_error: vector length {} mismatches declared dim {}",
                    parsed.vector.len(),
                    dim
                ),
            });
        }

        // Zero-magnitude rejection — consistent with the pre-existing
        // `SteeringVector::validate` path.
        let magnitude_sq: f32 = parsed.vector.iter().map(|&x| x * x).sum();
        if magnitude_sq < 1e-6 {
            return Err(SteeringError::ZeroMagnitude {
                name: role.to_owned(),
            });
        }

        // Persist the raw little-endian f32 bytes. Create the parent dir
        // eagerly — the canonical layout nests under a per-project subdir
        // that may not exist yet.
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| SteeringError::StorageError {
                reason: format!("cache_write_failed: mkdir {}: {e}", parent.display()),
            })?;
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(parsed.vector.len() * 4);
        for &x in &parsed.vector {
            bytes.extend_from_slice(&x.to_le_bytes());
        }
        std::fs::write(cache_path, &bytes).map_err(|e| SteeringError::StorageError {
            reason: format!("cache_write_failed: {} — {e}", cache_path.display()),
        })?;

        let sv = SteeringVector {
            name: role.to_owned(),
            layer,
            coefficient: COEFFICIENT_DEFAULT,
            vector: parsed.vector,
            positive_prompt: pairs.positive.join("\n---\n"),
            negative_prompt: pairs.negative.join("\n---\n"),
        };

        let local_norm = sv.magnitude();
        if let Some(server_norm) = parsed.norm {
            // Warn when the sidecar's and the Rust side's f32 summation
            // disagree beyond a loose absolute tolerance — different f32
            // accumulation orders will not produce identical bits, but a
            // large delta hints at payload corruption.
            if (server_norm - local_norm).abs() > 0.05 {
                tracing::warn!(
                    role,
                    server_norm,
                    local_norm,
                    "steering vector norm mismatch between sidecar and client"
                );
            }
        }
        tracing::info!(
            role,
            dim = sv.vector.len(),
            layer = sv.layer,
            norm = local_norm,
            model = parsed.model.as_deref().unwrap_or("(unknown)"),
            cache_path = %cache_path.display(),
            "steering vector computed and cached"
        );
        Ok(sv)
    }

    /// Read the cache bin at `path` back into `Vec<f32>`. Returns an error
    /// when the file is missing, empty, or not a multiple of 4 bytes.
    pub fn load_cached_vector_bytes(path: &Path) -> Result<Vec<f32>, SteeringError> {
        let raw = std::fs::read(path).map_err(|e| SteeringError::StorageError {
            reason: format!("cache_read_failed: {} — {e}", path.display()),
        })?;
        if raw.is_empty() {
            return Err(SteeringError::StorageError {
                reason: format!("cache_read_failed: {} is empty", path.display()),
            });
        }
        if raw.len() % 4 != 0 {
            return Err(SteeringError::StorageError {
                reason: format!(
                    "cache_read_failed: {} length {} not aligned to f32 (4 bytes)",
                    path.display(),
                    raw.len()
                ),
            });
        }
        let vector: Vec<f32> = raw
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        Ok(vector)
    }

    /// Scan the per-project steering cache directory and return a status
    /// describing whether any per-role vector is currently cached.
    ///
    /// The directory layout (from `crate::steering_cache_path`) is
    /// `<project_root>/.elai_cc/steering/<project_id>/<role>.bin`. Files
    /// with a zero-byte payload are ignored — the writer never produces
    /// empty files, so a zero-byte file indicates an interrupted write.
    pub fn scan_steering_cache_dir(
        project_root: &Path,
        project_id: &str,
    ) -> Result<ScannedSteeringStatus, SteeringError> {
        let dir = project_root
            .join(".elai_cc")
            .join("steering")
            .join(project_id);

        if !dir.exists() {
            return Ok(ScannedSteeringStatus {
                active: false,
                reason: "modern_api_preference_vector_unavailable".to_owned(),
                roles: Vec::new(),
                vector_dir: dir.to_string_lossy().into_owned(),
            });
        }

        let mut roles: Vec<String> = Vec::new();
        let read = std::fs::read_dir(&dir).map_err(|e| SteeringError::StorageError {
            reason: format!("cache_read_failed: {} — {e}", dir.display()),
        })?;
        for entry in read.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.len() == 0 {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                    roles.push(stem.to_owned());
                }
            }
        }
        roles.sort();

        let vector_dir = dir.to_string_lossy().into_owned();
        if roles.is_empty() {
            Ok(ScannedSteeringStatus {
                active: false,
                reason: "modern_api_preference_vector_unavailable (empty directory)".to_owned(),
                roles,
                vector_dir,
            })
        } else {
            let reason = format!("cached roles: {}", roles.join(","));
            Ok(ScannedSteeringStatus {
                active: true,
                reason,
                roles,
                vector_dir,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// ContrastPairs (batch-ful-c)
// ---------------------------------------------------------------------------

/// Bundle of positive + negative example prompts used to compute a freshness
/// steering vector via the `/v1/steering/compute` sidecar endpoint.
///
/// The vector is computed as `mean(embed(positive)) - mean(embed(negative))`
/// at the chosen layer. A batch of examples (N=3-5) yields a more stable
/// direction than a single contrast pair.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContrastPairs {
    /// Examples of the "modern / preferred" API style.
    pub positive: Vec<String>,
    /// Examples of the "deprecated / avoided" API style.
    pub negative: Vec<String>,
}

impl ContrastPairs {
    /// Construct a new pair set. Either side is allowed to be empty at
    /// construction time; the helper itself rejects empty pairs before
    /// hitting the sidecar.
    pub fn new(positive: Vec<String>, negative: Vec<String>) -> Self {
        Self { positive, negative }
    }
}

/// Result of [`SteeringVectorEngine::scan_steering_cache_dir`]. Mirrors the
/// shape the dashboard / SSB reader expect so they can both call the scan
/// helper without a second translation layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedSteeringStatus {
    /// `true` when at least one role has a non-empty cache file.
    pub active: bool,
    /// Human-readable reason for the status, suitable for SSB emission.
    pub reason: String,
    /// Sorted list of roles with a cached vector. Empty when `active=false`.
    pub roles: Vec<String>,
    /// Directory scanned, stringified so callers can log it without a second
    /// `PathBuf` allocation.
    pub vector_dir: String,
}

/// Convenience alias so callers can reach `PathBuf` without importing from
/// `std::path` — matches the pattern used by `SidecarProvider::steering_vector_cached`.
pub type SteeringCachePath = PathBuf;

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
