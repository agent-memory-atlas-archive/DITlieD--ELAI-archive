Phase 1C Decisions
Session: s-1375148
Date: 2026-04-06

HNSW DECISION: usearch
  Reason: usearch crate v2.24.0 confirmed on crates.io with Rust bindings.
  SIMD-accelerated HNSW. Supports f32 cosine metric.
  No custom stub needed. No Phase 1G deferral for HNSW.

FASTEMBED EXCLUSION
  fastembed (v5.12.0) wraps ort internally. Importing both risks ort version conflicts.
  Decision: use ort directly for ONNX inference. Do not add fastembed to Cargo.toml.
  If fastembed convenience (tokenizer integration, model downloading) needed later,
  evaluate ort version alignment at that time.

BLAKE2 CRATE SELECTION
  blake2 crate (80M+ downloads) over blake2s crate (1.9K downloads).
  Use blake2::Blake2s256 for content_blake2s column hashing.

TURBOQUANT STATUS
  Paper: arXiv 2504.19874, ICLR 2026.
  Implementation BLOCKED until paper-ingestor completes.
  Phase 1C stores raw f32 vectors only. Quantization deferred.

TENSOR FORMAT (verified 2026-04-07, session s-818976)
  tensor_format_verified: true

  GraphCodeBERT (microsoft/graphcodebert-base)
    Architecture: RoBERTa-based, 12 layers, 12 attention heads
    Input tensors:
      input_ids:      shape (batch, seq_len), dtype int64
      attention_mask: shape (batch, seq_len), dtype int64
      token_type_ids: optional (zeros for RoBERTa), not required
    Max seq_len: 512
    Output: last_hidden_state (batch, seq_len, 768)
    Pooling: CLS token (index 0) -> (batch, 768)
    Tokenizer: RoBERTa BPE, vocab_size=50265
    Source: huggingface.co/microsoft/graphcodebert-base

  Granite-embedding-30m-english (ibm-granite/granite-embedding-30m-english)
    Architecture: RoBERTa-like encoder, 6 layers, 12 attention heads
    Input tensors:
      input_ids:      shape (batch, seq_len), dtype int64
      attention_mask: shape (batch, seq_len), dtype int64
      token_type_ids: not required
    Max seq_len: 512
    Output: sentence_embedding (batch, 384), CLS-pooled + L2-normalized
    Tokenizer: RoBERTa BPE, vocab_size=50265
    ONNX variants: fp32, fp16, q8, q4, q4f16 (onnx-community/granite-embedding-30m-english-ONNX)
    Source: huggingface.co/ibm-granite/granite-embedding-30m-english

  Shared properties:
    Both models use RoBERTa tokenizer (same vocab, same BPE merges)
    Both accept int64 input tensors
    Both max_seq_len=512
    Both use CLS pooling (GraphCodeBERT manually, Granite built-in)

  Tokenizer dependency:
    Crate: tokenizers v0.22 (huggingface/tokenizers Rust crate)
    Feature: "http" for Tokenizer::from_pretrained()
    API: Encoding::get_ids() -> &[u32], Encoding::get_attention_mask() -> &[u32]
    IMPORTANT: tokenizers returns u32, ONNX expects int64. Step 4 must cast u32->i64.
    Source: docs.rs/tokenizers/0.22.2
