use crate::ReadinessStatus;

pub mod cache;
pub mod circuit_breaker;
pub mod hnsw_index;
pub mod ort_session;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum VectorError {
    DimensionMismatch { expected: usize, got: usize },
    NotReady,
    CacheError(String),
    OrtError(String),
    IndexError(String),
}

impl std::fmt::Display for VectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionMismatch { expected, got } => {
                write!(f, "dimension mismatch: expected {expected}, got {got}")
            }
            Self::NotReady => write!(f, "vector module not ready"),
            Self::CacheError(e) => write!(f, "cache error: {e}"),
            Self::OrtError(e) => write!(f, "ort error: {e}"),
            Self::IndexError(e) => write!(f, "index error: {e}"),
        }
    }
}

impl std::error::Error for VectorError {}

// ---------------------------------------------------------------------------
// Search mode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Hybrid,
    CodeOnly,
    TextOnly,
    SparseOnly,
}

// ---------------------------------------------------------------------------
// DualVector
// ---------------------------------------------------------------------------

pub const CODE_VEC_DIM: usize = 768;
pub const TEXT_VEC_DIM: usize = 384;

#[derive(Debug, Clone)]
pub struct DualVector {
    pub code_vec: Vec<f32>,
    pub text_vec: Vec<f32>,
}

impl DualVector {
    pub fn new(code_vec: Vec<f32>, text_vec: Vec<f32>) -> Result<Self, VectorError> {
        if code_vec.len() != CODE_VEC_DIM {
            return Err(VectorError::DimensionMismatch {
                expected: CODE_VEC_DIM,
                got: code_vec.len(),
            });
        }
        if text_vec.len() != TEXT_VEC_DIM {
            return Err(VectorError::DimensionMismatch {
                expected: TEXT_VEC_DIM,
                got: text_vec.len(),
            });
        }
        Ok(Self { code_vec, text_vec })
    }
}

// ---------------------------------------------------------------------------
// VectorModule trait
// ---------------------------------------------------------------------------

pub trait VectorModule {
    fn embed_code(&self, text: &str) -> Result<Vec<f32>, VectorError>;
    fn embed_text(&self, text: &str) -> Result<Vec<f32>, VectorError>;
    fn readiness(&self) -> ReadinessStatus;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_vector_correct_dims_ok() {
        let code = vec![0.0f32; CODE_VEC_DIM];
        let text = vec![0.0f32; TEXT_VEC_DIM];
        let dv = DualVector::new(code, text);
        assert!(dv.is_ok());
        let dv = dv.unwrap();
        assert_eq!(dv.code_vec.len(), CODE_VEC_DIM);
        assert_eq!(dv.text_vec.len(), TEXT_VEC_DIM);
    }

    #[test]
    fn dual_vector_wrong_code_dim_returns_err() {
        let code = vec![0.0f32; 512];
        let text = vec![0.0f32; TEXT_VEC_DIM];
        let result = DualVector::new(code, text);
        assert!(result.is_err());
        match result.unwrap_err() {
            VectorError::DimensionMismatch { expected, got } => {
                assert_eq!(expected, CODE_VEC_DIM);
                assert_eq!(got, 512);
            }
            other => panic!("expected DimensionMismatch, got {other:?}"),
        }
    }

    #[test]
    fn dual_vector_wrong_text_dim_returns_err() {
        let code = vec![0.0f32; CODE_VEC_DIM];
        let text = vec![0.0f32; 256];
        let result = DualVector::new(code, text);
        assert!(result.is_err());
        match result.unwrap_err() {
            VectorError::DimensionMismatch { expected, got } => {
                assert_eq!(expected, TEXT_VEC_DIM);
                assert_eq!(got, 256);
            }
            other => panic!("expected DimensionMismatch, got {other:?}"),
        }
    }

    #[test]
    fn dual_vector_both_wrong_returns_code_dim_err_first() {
        let code = vec![0.0f32; 100];
        let text = vec![0.0f32; 100];
        let result = DualVector::new(code, text);
        assert!(result.is_err());
        match result.unwrap_err() {
            VectorError::DimensionMismatch { expected, got } => {
                assert_eq!(expected, CODE_VEC_DIM);
                assert_eq!(got, 100);
            }
            other => panic!("expected DimensionMismatch for code_vec, got {other:?}"),
        }
    }
}
