//! F-12 Tier-2 — `OnnxNliJudge`: the LOCAL deterministic-argmax NLI
//! cross-encoder over the already-vetted `ort`/ONNX runtime.
//!
//! Feature-gated behind `nli-onnx` (default-OFF) because no NLI ONNX model
//! is staged on disk today; the runtime + tokenizer + `models/onnx/`
//! convention are REUSED from `crates/knowledge/src/reranker.rs` (the
//! infrastructure is not net-new — only the model FILE is). The default
//! Tier-1 build compiles WITHOUT `ort`; this file is the dark-of-the-model
//! seam that plugs into the SAME [`crate::semantic_contradiction::NliJudge`]
//! trait.
//!
//! Mirrors `reranker.rs` EXACTLY for reproducibility: `intra_op_threads=1`,
//! `with_deterministic_compute(true)`, `GraphOptimizationLevel::All`,
//! `models/onnx/` path via `elai_core::paths::models_dir_path`. The ONE
//! difference is post-processing: the reranker extracts a single `[batch,1]`
//! logit; an NLI cross-encoder emits `[batch,3]` (contradiction / entailment
//! / neutral), so `classify` does a 3-class argmax + a softmax-max for the
//! confidence margin.
//!
//! Determinism framing (HONEST): "practically stable argmax, NOT
//! bit-identical" — the same GPU FP batch-variance class (arXiv:2506.09501)
//! applies to these logits. The deterministic-compute flags + single intra
//! thread make replay robustly stable, not formally seed-invariant.
//!
//! Construction FAILS LOUDLY when the model file is absent (no silent
//! fall-through to a stub): the live tick catches the error and SKIPS the
//! semantic pass (the byte-identical `audit_conflicts` path still runs).

use std::path::{Path, PathBuf};

use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;

use crate::error::MemoryError;
use crate::semantic_contradiction::{NliJudge, NliLabel, NliVerdict};

/// Maximum BERT sequence length (mirrors the reranker default).
const MAX_SEQUENCE_LENGTH: usize = 512;

/// ONNX model filename under `models/onnx/`. The variant suffix is the
/// VRAM-tier selector (xsmall for the 8 GB tier, base for 24 GB), mirroring
/// `ReRankerModel`. The base variant is the documented default.
const DEFAULT_NLI_MODEL_FILENAME: &str = "nli-cross-encoder-base.onnx";

/// Tokenizer filename shared with the reranker convention.
const TOKENIZER_FILENAME: &str = "tokenizer.json";

/// The model's documented 3-class label order. `cross-encoder/nli-deberta-v3-*`
/// emits logits in `[contradiction, entailment, neutral]` order. Recorded
/// here so the argmax maps to the right [`NliLabel`]; the decision doc
/// pins this to the staged model's config at staging time.
const LABEL_ORDER: [NliLabel; 3] = [
    NliLabel::Contradiction,
    NliLabel::Entailment,
    NliLabel::Neutral,
];

/// The live ONNX NLI judge. Holds the loaded session + tokenizer.
pub struct OnnxNliJudge {
    session: std::sync::Mutex<Session>,
    tokenizer: tokenizers::Tokenizer,
    label_order: [NliLabel; 3],
}

impl OnnxNliJudge {
    /// Resolve the model + tokenizer under `<project_root>/.elai_cc/models/onnx/`
    /// and load them. Returns `Err` (FAILS LOUDLY) when either file is
    /// absent — the caller treats this as a fail-safe SKIP, never a stub.
    pub fn with_project_root(project_root: &Path) -> Result<Self, MemoryError> {
        let onnx_dir = elai_core::paths::models_dir_path(project_root).join("onnx");
        let model_path = onnx_dir.join(DEFAULT_NLI_MODEL_FILENAME);
        let tokenizer_path = onnx_dir.join(TOKENIZER_FILENAME);
        Self::from_paths(&model_path, &tokenizer_path)
    }

    /// Load from explicit paths (used by ops staging + tests once a model
    /// is on disk).
    pub fn from_paths(model_path: &Path, tokenizer_path: &Path) -> Result<Self, MemoryError> {
        let session = build_session(model_path)?;
        let tokenizer = load_tokenizer(tokenizer_path)?;
        Ok(Self {
            session: std::sync::Mutex::new(session),
            tokenizer,
            label_order: LABEL_ORDER,
        })
    }
}

/// Build an ORT session with the reranker's exact deterministic settings.
/// Wraps ORT calls in `catch_unwind` to convert panics to typed errors
/// (mirrors `reranker.rs::build_session`).
fn build_session(model_path: &Path) -> Result<Session, MemoryError> {
    if !model_path.exists() {
        return Err(MemoryError::Invalid(format!(
            "nli onnx model not found: {} (Tier-2 model staging gated; the \
             byte-identical audit_conflicts path still runs)",
            model_path.display()
        )));
    }
    let path_owned: PathBuf = model_path.to_path_buf();
    let result = std::panic::catch_unwind(move || -> Result<Session, String> {
        let builder = Session::builder().map_err(|e| e.to_string())?;
        let builder = builder
            .with_optimization_level(GraphOptimizationLevel::All)
            .map_err(|e| e.to_string())?;
        let builder = builder.with_intra_threads(1).map_err(|e| e.to_string())?;
        let mut builder = builder
            .with_deterministic_compute(true)
            .map_err(|e| e.to_string())?;
        let session = builder
            .commit_from_file(&path_owned)
            .map_err(|e| e.to_string())?;
        Ok(session)
    });
    match result {
        Ok(Ok(session)) => Ok(session),
        Ok(Err(msg)) => Err(MemoryError::Invalid(format!(
            "{}: {msg}",
            model_path.display()
        ))),
        Err(_) => Err(MemoryError::Invalid(format!(
            "{}: ort panicked during session creation",
            model_path.display()
        ))),
    }
}

/// Load + configure the tokenizer (truncation to [`MAX_SEQUENCE_LENGTH`]).
fn load_tokenizer(tokenizer_path: &Path) -> Result<tokenizers::Tokenizer, MemoryError> {
    if !tokenizer_path.exists() {
        return Err(MemoryError::Invalid(format!(
            "nli tokenizer not found: {}",
            tokenizer_path.display()
        )));
    }
    let mut tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path)
        .map_err(|e| MemoryError::Invalid(format!("tokenizer load {}: {e}", tokenizer_path.display())))?;
    let truncation = tokenizers::utils::truncation::TruncationParams {
        max_length: MAX_SEQUENCE_LENGTH,
        strategy: tokenizers::utils::truncation::TruncationStrategy::LongestFirst,
        ..Default::default()
    };
    tokenizer
        .with_truncation(Some(truncation))
        .map_err(|e| MemoryError::Invalid(format!("tokenizer truncation config: {e}")))?;
    Ok(tokenizer)
}

/// Numerically-stable softmax-max over a slice of logits.
fn softmax_max(logits: &[f32]) -> f32 {
    if logits.is_empty() {
        return 0.0;
    }
    let max = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let sum: f32 = logits.iter().map(|&x| (x - max).exp()).sum();
    if sum <= 0.0 {
        return 0.0;
    }
    // The max logit maps to exp(0)=1 in the shifted space.
    1.0 / sum
}

impl NliJudge for OnnxNliJudge {
    fn classify(&self, premise: &str, hypothesis: &str) -> Result<NliVerdict, MemoryError> {
        // Tokenize the (premise, hypothesis) pair (token_type_ids 0/1
        // distinguish the two sequences — the same encode((a,b)) shape the
        // reranker uses).
        let encoding = self
            .tokenizer
            .encode((premise, hypothesis), true)
            .map_err(|e| MemoryError::Invalid(format!("nli tokenize: {e}")))?;

        // NARROWING-OK: tokenizer IDs/mask/type_ids are u32; i64 holds all
        // u32 values (widening).
        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&x| x as i64)
            .collect();
        let token_type_ids: Vec<i64> = encoding
            .get_type_ids()
            .iter()
            .map(|&x| x as i64)
            .collect();
        let seq_len = input_ids.len();
        let seq_i64 = i64::try_from(seq_len).unwrap_or(i64::MAX);

        let id_tensor = Tensor::from_array((vec![1_i64, seq_i64], input_ids))
            .map_err(|e| MemoryError::Invalid(format!("tensor input_ids: {e}")))?;
        let mask_tensor = Tensor::from_array((vec![1_i64, seq_i64], attention_mask))
            .map_err(|e| MemoryError::Invalid(format!("tensor attention_mask: {e}")))?;
        let type_tensor = Tensor::from_array((vec![1_i64, seq_i64], token_type_ids))
            .map_err(|e| MemoryError::Invalid(format!("tensor token_type_ids: {e}")))?;

        let inputs = ort::inputs![
            "input_ids" => id_tensor,
            "attention_mask" => mask_tensor,
            "token_type_ids" => type_tensor,
        ];

        let mut session = self
            .session
            .lock()
            .map_err(|_| MemoryError::Invalid("nli session mutex poisoned".into()))?;
        let outputs = session
            .run(inputs)
            .map_err(|e| MemoryError::Invalid(format!("ort session.run: {e}")))?;
        let (_shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| MemoryError::Invalid(format!("extract logits: {e}")))?;

        // Single pair -> output is [1, 3]; take the first 3 logits.
        if data.len() < 3 {
            return Err(MemoryError::Invalid(format!(
                "nli output has {} logits, expected >= 3 (contradiction/entailment/neutral)",
                data.len()
            )));
        }
        let logits = [data[0], data[1], data[2]];

        // 3-class argmax (deterministic over a fixed slice; ties resolve to
        // the lowest index, which is contradiction-first per LABEL_ORDER).
        let mut best_idx = 0usize;
        let mut best_val = logits[0];
        for (i, &v) in logits.iter().enumerate().skip(1) {
            if v > best_val {
                best_val = v;
                best_idx = i;
            }
        }
        let label = self.label_order[best_idx];
        let confidence = softmax_max(&logits);

        Ok(NliVerdict { label, confidence })
    }
}
