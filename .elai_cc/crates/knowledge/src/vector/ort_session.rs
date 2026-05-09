use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum OrtError {
    ModelNotFound(String),
    InferenceFailed(String),
    SessionLock(String),
    TokenizerLoad(String),
}

impl std::fmt::Display for OrtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelNotFound(p) => write!(f, "model not found: {p}"),
            Self::InferenceFailed(e) => write!(f, "inference failed: {e}"),
            Self::SessionLock(e) => write!(f, "session lock poisoned: {e}"),
            Self::TokenizerLoad(e) => write!(f, "tokenizer load failed: {e}"),
        }
    }
}

impl std::error::Error for OrtError {}

// ---------------------------------------------------------------------------
// OrtSession
// ---------------------------------------------------------------------------

pub struct OrtSession {
    session: Arc<Mutex<Session>>,
    tokenizer: Arc<tokenizers::Tokenizer>,
    pub max_seq_len: usize,
    pub output_dim: usize,
    /// True when the loaded ONNX model declares a "token_type_ids" input.
    /// RoBERTa-family models (graphcodebert, granite-embed) do not; BERT-family
    /// models (cross-encoder) do.  Captured once at `new()` from `session.get_inputs()`.
    pub has_token_type_ids: bool,
}

impl std::fmt::Debug for OrtSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrtSession")
            .field("max_seq_len", &self.max_seq_len)
            .field("output_dim", &self.output_dim)
            .finish_non_exhaustive()
    }
}

impl OrtSession {
    /// Load an ONNX model from `model_path` and tokenizer from `tokenizer_path`.
    /// Returns `Err(OrtError::ModelNotFound)` if the model file is missing or ort panics.
    /// Returns `Err(OrtError::TokenizerLoad)` if the tokenizer file is missing or invalid.
    pub fn new(
        model_path: &Path,
        tokenizer_path: &Path,
        max_seq_len: usize,
        output_dim: usize,
    ) -> Result<Self, OrtError> {
        if !model_path.exists() {
            return Err(OrtError::ModelNotFound(
                model_path.display().to_string(),
            ));
        }
        if !tokenizer_path.exists() {
            return Err(OrtError::TokenizerLoad(format!(
                "tokenizer not found: {}",
                tokenizer_path.display()
            )));
        }

        let path_owned = model_path.to_path_buf();
        let result = std::panic::catch_unwind(|| {
            Session::builder()
                .and_then(|mut b| b.commit_from_file(&path_owned))
        });

        let session = match result {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                return Err(OrtError::ModelNotFound(format!(
                    "{}: {e}",
                    model_path.display()
                )));
            }
            Err(_) => {
                return Err(OrtError::ModelNotFound(format!(
                    "{}: ort panicked during session creation",
                    model_path.display()
                )));
            }
        };

        let mut tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path)
            .map_err(|e| OrtError::TokenizerLoad(format!("{}: {e}", tokenizer_path.display())))?;

        let truncation = tokenizers::utils::truncation::TruncationParams {
            max_length: max_seq_len,
            strategy: tokenizers::utils::truncation::TruncationStrategy::LongestFirst,
            ..Default::default()
        };
        tokenizer
            .with_truncation(Some(truncation))
            .map_err(|e| OrtError::TokenizerLoad(format!("truncation config: {e}")))?;

        let padding = tokenizers::PaddingParams {
            strategy: tokenizers::PaddingStrategy::Fixed(max_seq_len),
            pad_id: 0,
            pad_token: "[PAD]".to_string(),
            ..Default::default()
        };
        tokenizer.with_padding(Some(padding));

        let has_token_type_ids = session
            .inputs()
            .iter()
            .any(|inp| inp.name() == "token_type_ids");

        Ok(Self {
            session: Arc::new(Mutex::new(session)),
            tokenizer: Arc::new(tokenizer),
            max_seq_len,
            output_dim,
            has_token_type_ids,
        })
    }

    /// Embed text into a vector via ONNX inference.
    ///
    /// Tokenizes input, builds input_ids + attention_mask tensors, runs the
    /// ONNX session, and applies mean-pooling over the token embeddings
    /// (masked by attention_mask) to produce a single vector of `output_dim`.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, OrtError> {
        let encoding = self.tokenizer
            .encode(text, false)
            .map_err(|e| OrtError::InferenceFailed(format!("tokenization: {e}")))?;

        let ids = encoding.get_ids();
        let mask = encoding.get_attention_mask();
        let type_ids = encoding.get_type_ids();
        let seq_len = ids.len();

        // NARROWING-OK: u32->i64 is a widening cast; tokenizer IDs/mask/type_ids are u32, i64 holds all u32 values.
        let input_ids: Vec<i64> = ids.iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = mask.iter().map(|&m| m as i64).collect();
        let token_type_ids: Vec<i64> = type_ids.iter().map(|&t| t as i64).collect();

        // NARROWING-OK: ORT shape contract; seq_len from tokenizer is bounded to model max_length (< i64::MAX).
        let shape = vec![1i64, seq_len as i64];
        let id_tensor = Tensor::from_array((shape.clone(), input_ids))
            .map_err(|e| OrtError::InferenceFailed(format!("tensor input_ids: {e}")))?;
        let mask_tensor = Tensor::from_array((shape.clone(), attention_mask.clone()))
            .map_err(|e| OrtError::InferenceFailed(format!("tensor attention_mask: {e}")))?;
        let type_tensor = Tensor::from_array((shape, token_type_ids))
            .map_err(|e| OrtError::InferenceFailed(format!("tensor token_type_ids: {e}")))?;

        let mut session_inputs: Vec<(std::borrow::Cow<'_, str>, ort::session::SessionInputValue<'_>)> = vec![
            ("input_ids".into(), id_tensor.into()),
            ("attention_mask".into(), mask_tensor.into()),
        ];
        if self.has_token_type_ids {
            session_inputs.push(("token_type_ids".into(), type_tensor.into()));
        }

        let mut session = self.session.lock()
            .map_err(|e| OrtError::SessionLock(e.to_string()))?;

        let outputs = session.run(session_inputs)
            .map_err(|e| OrtError::InferenceFailed(format!("session.run: {e}")))?;

        // Output shape: [1, seq_len, hidden_dim] — apply mean pooling with attention mask.
        let (out_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| OrtError::InferenceFailed(format!("extract tensor: {e}")))?;

        // NARROWING-OK: ORT shape contract; i64 output dimensions from ORT are always non-negative and fit usize on 64-bit.
        let hidden_dim = if out_shape.len() == 3 {
            out_shape[2] as usize
        } else if out_shape.len() == 2 {
            // Some models output [1, hidden_dim] (already pooled).
            let dim = out_shape[1] as usize;
            if dim == self.output_dim {
                return Ok(data.iter().copied().take(dim).collect());
            }
            return Err(OrtError::InferenceFailed(format!(
                "unexpected 2D output shape {:?}, expected dim {}",
                out_shape, self.output_dim
            )));
        } else {
            return Err(OrtError::InferenceFailed(format!(
                "unexpected output shape {:?}",
                out_shape
            )));
        };

        if hidden_dim != self.output_dim {
            return Err(OrtError::InferenceFailed(format!(
                "hidden_dim {hidden_dim} != expected output_dim {}",
                self.output_dim
            )));
        }

        // Mean pooling: average token embeddings weighted by attention mask.
        let mut pooled = vec![0.0f32; hidden_dim];
        let mut mask_sum = 0.0f32;
        // NARROWING-OK: ORT shape contract; out_shape[1] from ORT is seq_len (positive, fits usize).
        let tokens = out_shape[1] as usize;
        for t in 0..tokens {
            // NARROWING-OK: attention mask values are 0 or 1; i64->f32 is exact.
            let m = attention_mask[t] as f32;
            mask_sum += m;
            let offset = t * hidden_dim;
            for d in 0..hidden_dim {
                pooled[d] += data[offset + d] * m;
            }
        }
        if mask_sum > 0.0 {
            for d in 0..hidden_dim {
                pooled[d] /= mask_sum;
            }
        }

        Ok(pooled)
    }

    pub fn session(&self) -> &Arc<Mutex<Session>> {
        &self.session
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn ort_session_bad_model_path_returns_err_not_panic() {
        let bad_model = PathBuf::from("/nonexistent/model.onnx");
        let bad_tok = PathBuf::from("/nonexistent/tokenizer.json");
        let result = OrtSession::new(&bad_model, &bad_tok, 512, 768);
        assert!(result.is_err());
        match result.unwrap_err() {
            OrtError::ModelNotFound(_) => {}
            other => panic!("expected ModelNotFound, got {other:?}"),
        }
    }

    #[test]
    fn ort_session_error_types_display() {
        let err = OrtError::InferenceFailed("test inference error".into());
        assert!(err.to_string().contains("inference failed"));
        let err2 = OrtError::TokenizerLoad("bad tokenizer".into());
        assert!(err2.to_string().contains("tokenizer"));
    }

    #[test]
    fn ort_session_new_propagates_max_seq_len_and_output_dim() {
        let p1 = PathBuf::from("/no/model_a.onnx");
        let t1 = PathBuf::from("/no/tok_a.json");
        let r1 = OrtSession::new(&p1, &t1, 512, 768);
        assert!(r1.is_err());

        let p2 = PathBuf::from("/no/model_b.onnx");
        let t2 = PathBuf::from("/no/tok_b.json");
        let r2 = OrtSession::new(&p2, &t2, 128, 384);
        assert!(r2.is_err());

        match r1.unwrap_err() {
            OrtError::ModelNotFound(msg) => assert!(msg.contains("model_a")),
            other => panic!("expected ModelNotFound, got {other:?}"),
        }
        match r2.unwrap_err() {
            OrtError::ModelNotFound(msg) => assert!(msg.contains("model_b")),
            other => panic!("expected ModelNotFound, got {other:?}"),
        }
    }
}
