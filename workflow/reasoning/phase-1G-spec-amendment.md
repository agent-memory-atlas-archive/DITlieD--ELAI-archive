Phase 1G Spec Amendment Request
Date: 2026-04-06
Session: s-1007436
Status: PROPOSED (pending user approval)

AMENDMENT 1: L-6 intermediate fallback tier in cross-encoder degradation cascade

Current spec (knowledge.txt L149):
  "Cross-encoder failure -> skip reranking, use vector similarity scores directly."

Proposed change:
  "Cross-encoder (L-12) failure -> attempt L-6 fallback model -> skip reranking, use vector similarity scores directly."

Rationale:
  The current spec has a binary degradation path: full cross-encoder or nothing. The L-6 variant (22.7M params, MRR@10=36.81 vs L-12's 39.02, verified on HuggingFace) provides a middle ground: lower accuracy than L-12 but still measurably better than no reranking. This matches the existing graceful degradation pattern in the knowledge crate (HYBRID -> SPARSE_ONLY -> BM25_ONLY) where each tier sacrifices quality incrementally rather than dropping to the floor.

Evidence:
  - L-6 MRR@10 = 36.81 (HuggingFace model card) vs L-12 = 39.02. Delta = 2.21 points.
  - L-6 has ~2x fewer layers (6 vs 12), proportionally faster inference.
  - FlashRank uses L-6 as default model (~4MB quantized), validating it as production-viable.
  - No reranking loses all cross-encoder signal (precision@5 drops to A-RAG baseline).

Implementation:
  - L-6 gets its own circuit breaker instance (same config as L-12).
  - L-6 session loaded lazily on first L-12 failure (cold-start acceptable during degradation).
  - If both L-12 and L-6 circuit breakers open: fall through to skip reranking (preserves existing spec behavior as final fallback).
  - L-6 uses same tokenizer as L-12 (both derive from microsoft/MiniLM, same WordPiece vocabulary).

Spec files to update if approved:
  docs/specs/crates/knowledge.txt L149 (fallback cascade)
  docs/specs/crates/knowledge.txt L135 (add L-6 circuit breaker mention)

Risk: minimal. L-6 fallback is additive. If L-6 also fails, behavior is identical to current spec.
