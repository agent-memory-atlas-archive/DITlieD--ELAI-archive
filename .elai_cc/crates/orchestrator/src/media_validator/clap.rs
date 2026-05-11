//! B-24 Phase 4I (Group C1): CLAP audio embedding skeleton.
//!
//! Real impl (audio + text encoders, mel-spectrogram preprocessing, ort
//! inference) lands in Group C3. C1 only ships the configuration types,
//! the `cosine_similarity` utility (used by Group C2 unit tests), and a
//! `feature = "vlm-integration"` gated `load_model` stub so the dep tree
//! stays small in the default build.

use std::path::PathBuf;

use super::types::{ClapConfig, ClapDevice, MediaValidatorError};

/// CLAP audio model wrapper. Group C3 will carry two `ort::Session`s
/// (audio + text); the skeleton only retains the config so the type is
/// constructable.
#[derive(Debug, Clone)]
pub struct ClapAudioModel {
    pub config: ClapConfig,
}

impl ClapAudioModel {
    pub fn new(config: ClapConfig) -> Self {
        Self { config }
    }

    /// Load the underlying ONNX models. Real impl uses
    /// `ort::Session::builder().with_execution_providers([CPUExecutionProvider::default().build()])`
    /// per workflow/papers/ort-cpu-ep-interface.md L57,75 for 1e-6 reproducibility.
    /// Skeleton returns `MediaValidatorError::NotImplemented`.
    #[cfg(feature = "vlm-integration")]
    pub fn load_model(&self) -> Result<(), MediaValidatorError> {
        Err(MediaValidatorError::NotImplemented(
            "ClapAudioModel::load_model (Group C3)",
        ))
    }

    #[cfg(not(feature = "vlm-integration"))]
    pub fn load_model(&self) -> Result<(), MediaValidatorError> {
        Err(MediaValidatorError::NotImplemented(
            "ClapAudioModel::load_model (vlm-integration feature off)",
        ))
    }

    /// Embed an audio buffer (Mel-spectrogram preprocessing in Rust per
    /// spec L275). Real impl returns a 512-dim Vec<f32>; skeleton returns
    /// `NotImplemented`.
    pub fn embed_audio(&self, _pcm_f32: &[f32]) -> Result<Vec<f32>, MediaValidatorError> {
        Err(MediaValidatorError::NotImplemented(
            "ClapAudioModel::embed_audio (Group C3)",
        ))
    }

    /// Embed a text prompt. Real impl runs the RoBERTa text_model — paper
    /// L158 says NO token_type_ids. Skeleton returns `NotImplemented`.
    pub fn embed_text(&self, _text: &str) -> Result<Vec<f32>, MediaValidatorError> {
        Err(MediaValidatorError::NotImplemented(
            "ClapAudioModel::embed_text (Group C3)",
        ))
    }
}

/// Cosine similarity between two equal-length f32 vectors. REAL impl —
/// the C2 contract tests pin its determinism. Returns 0.0 when either
/// vector has zero L2 norm (avoids division-by-zero NaN propagation).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0_f32;
    let mut na = 0.0_f32;
    let mut nb = 0.0_f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        // Clamp to [-1.0, 1.0] for floating-point round-off safety.
        (dot / denom).clamp(-1.0, 1.0)
    }
}

/// Convenience: default ClapConfig pointing at a placeholder path. Tests
/// can override `model_path` to point at fixture ONNX files in C3.
pub fn default_clap_config() -> ClapConfig {
    ClapConfig {
        model_path: PathBuf::from("models/clap-htsat-unfused"),
        device: ClapDevice::Cpu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical_vectors() {
        let v = vec![1.0, 0.0, 0.0];
        let s = cosine_similarity(&v, &v);
        assert!((s - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_orthogonal_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let s = cosine_similarity(&a, &b);
        assert!(s.abs() < 1e-6);
    }

    #[test]
    fn cosine_opposite_vectors() {
        let a = vec![1.0, 1.0, 0.0];
        let b = vec![-1.0, -1.0, 0.0];
        let s = cosine_similarity(&a, &b);
        assert!((s - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn cosine_zero_vector_returns_zero() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 1.0, 1.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn cosine_mismatched_lengths_returns_zero() {
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 1.0]), 0.0);
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }

    #[test]
    fn clap_skeleton_methods_return_not_implemented() {
        let model = ClapAudioModel::new(default_clap_config());
        assert!(matches!(
            model.embed_audio(&[0.0; 16]),
            Err(MediaValidatorError::NotImplemented(_))
        ));
        assert!(matches!(
            model.embed_text("hi"),
            Err(MediaValidatorError::NotImplemented(_))
        ));
    }
}
