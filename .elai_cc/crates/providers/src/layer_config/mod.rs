//! Layer-level configuration and access traits for advanced decoding strategies.
//!
//! This module defines the `LayerAccessProvider` trait that abstracts how
//! intermediate transformer layer logits are obtained from an inference backend
//! (vLLM plugin, HuggingFace Transformers, etc.). Concrete decoding strategies
//! like DoLa consume layer logits through this trait.

pub mod dola;
pub mod steering;

use std::fmt;

/// Error returned by `LayerAccessProvider` implementations.
#[derive(Debug, thiserror::Error)]
pub enum LayerAccessError {
    /// The backend does not support layer-level access.
    #[error("layer access not supported by backend: {reason}")]
    NotSupported { reason: String },

    /// The backend failed to return logits for the requested layers.
    #[error("failed to fetch layer logits: {reason}")]
    FetchFailed { reason: String },

    /// Requested layer index is out of range for the model.
    #[error("layer index {index} out of range for model with {total} layers")]
    LayerOutOfRange { index: usize, total: usize },
}

/// A set of logits from a single transformer layer.
///
/// `layer_index` identifies which layer produced these logits.
/// `logits` is the full vocabulary distribution (raw logits, pre-softmax).
#[derive(Debug, Clone)]
pub struct LayerLogits {
    pub layer_index: usize,
    pub logits: Vec<f64>,
}

/// Trait for backends that can provide intermediate transformer layer logits.
///
/// Implementations fetch hidden states from specific layers, project them
/// through the vocabulary head, and return the resulting logit vectors.
/// This is the bridge between the inference backend (Python-side vLLM plugin
/// or HF Transformers) and the Rust-side decoding algorithms (DoLa, etc.).
///
/// NOTE: This trait is synchronous. When the Python IPC backend is implemented,
/// callers should use `tokio::task::spawn_blocking` or wrap the provider in an
/// async adapter. The trait is sync because DoLa's math is pure computation
/// and the current test phase uses in-process mock data only.
pub trait LayerAccessProvider: fmt::Debug + Send + Sync {
    /// Fetch logits from the specified candidate layers for the current token position.
    ///
    /// `final_layer_logits` is the standard output from the final layer (always available).
    /// `candidate_layers` lists the intermediate layer indices to retrieve.
    /// `total_layers` is the model's total layer count (N), used for validation.
    ///
    /// Returns one `LayerLogits` per requested candidate layer.
    ///
    /// # Errors
    /// Returns `LayerAccessError` if the backend cannot provide the requested layers.
    fn fetch_layer_logits(
        &self,
        candidate_layers: &[usize],
        total_layers: usize,
    ) -> Result<Vec<LayerLogits>, LayerAccessError>;
}
