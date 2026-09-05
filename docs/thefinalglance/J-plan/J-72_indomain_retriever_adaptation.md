# J-72: in-domain retriever adaptation — offline distill/contrastive fine-tune on ELAI's own supervision, ONNX-export, CPU inference unchanged

- Track J (context-retrieval wave) | lever CANT-MISREAD | grounding STRONG (3 independent primary sources converge: MCompassRAG / EvoEmbedding arXiv:2606.21649 / CodeRAG-system arXiv:2509.16112; all interface-mapped) | nature BUILD (offline pipeline) + SWAP (inference artifact) | effort M-L | WAVE CR-1 [KEYSTONE] | depends none (EXTENDS J-18, rides the shipped knowledge crate)

- Premise: the run's convergent meta-finding (context-retrieval-research-2026-06-23.md PART 2) is that the single highest-ROI retrieval lever is NOT another gadget on the cascade — it is to ADAPT ONE RETRIEVER IN-DOMAIN on ELAI's own supervision. Three independent primary sources land on the same shape: a ONE-TIME, OFFLINE, GPU fine-tune (contrastive or teacher-distillation) on ELAI's OWN issue/PR/edit + agent-trace signal -> EXPORT the encoder or a tiny scoring head to ONNX -> the CPU inference runtime is UNCHANGED. The training step is out-of-product (one GPU box, hours); the shipped artifact stays the existing ONNX-CPU path. This is the keystone of the whole wave; J-73/J-74/J-75 are cheaper refinements around it.

## Steal/build
1. EMBEDDER CONTRASTIVE FINE-TUNE (variant a, the encoder path): length-weighted InfoNCE contrastive fine-tune (tau=0.1) of the code embedder on ELAI's issue-PR-edit history mapped onto EvoEmbedding's EvoTrain-180K schema (messages + meta.turns + meta.evidence_turns -> query/positive/hard-negative triples). Export the fine-tuned encoder to INT8 ONNX exactly as J-18's CodeRankEmbed swap does (same usearch HNSW slot, index_version bump + re-embed, no geometry change). FOLD INTO J-18: this is the SAME embedder seam — author C1's encoder variant as the next increment ON J-18's CodeRankEmbed swap, never a duplicate embedder slot.
2. TEACHER-DISTILLED SCORING HEAD (variant b, the reranker path): distill an LLM teacher ONCE offline (~$10-20 for ~20K judgments) into a ~2M-param 3-layer scoring head on top of the FROZEN ONNX embedder (MCompassRAG); at inference it REPLACES the ms-marco cross-encoder reranker and ranks at embedding-model latency with teacher-grade quality, ZERO LLM calls in the hot path. The head is a tiny ONNX module loaded alongside the embedder — CPU, deterministic.
3. GOLD SUPERVISION SET: assemble ELAI's own gold set (issue/PR/edit pairs + sealed-RRA agent traces) as the train/eval source. The done-gate is an ELAI-MEASURED MRR / Recall@k delta on this gold set, NEVER a paper number (W1 below; ID-5 ELAI-measured-baselines-never-paper-constants).
EXCLUDED: KaLM-Reranker-V1 online GPU decoder cross-attention at query time (arXiv:2606.22807 — NOT ONNX-CPU portable, breaks the runtime; R1 in the ledger); EvoRAG dynamic memory-queue + LoRA-swap + flash-attn runtime (NOT ONNX-exportable; only its contrastive loss + schema survive here, R2); training infra in-product (ELAI ships NO in-product training; this is a new OUT-OF-BAND offline pipeline, not a runtime change — anti-slopsquat on our own claim).

## Existing ground truth (grep-confirmed 2026-06-23 against live .elai_cc; re-grep at edit, W-7/W-9)
- VECTOR SEARCH [WIRED] dual-model ONNX (GraphCodeBERT 768d code + Granite 384d text), usearch HNSW, blake3 content-hash cache, RRF k=60, index_version migration (feature map 1.1; crates/knowledge/src/vector/). The embedder slot is the fine-tune target.
- CROSS-ENCODER RERANKER [WIRED] ms-marco-MiniLM-L12-v2 ONNX CPU, <50ms/50 candidates (feature map 1.1; phase-1G). The scoring-head variant (b) replaces THIS.
- RETRIEVAL HARDENING [PLANNED(J-18)] CodeRankEmbed INT8 ONNX swap (exact-dim 768, index_version bump, export-from-source). J-72 EXTENDS J-18 — the in-domain fine-tune rides J-18's exact-dim swap path; C1 is folded into/around J-18, NOT a parallel embedder.
- REPRODUCIBLE-RUN ARTIFACT (RRA) [WIRED] G-2 sealed bundles = the agent-trace supervision source for the gold set.

## REVISED scope (W-10 tiered)
- SHIP NOW Tier-1 (in-product, model-free at runtime): the ONNX load path for the adapted encoder/head + the gold-set assembler (`elai retrieval gold-build` from sealed RRAs + issue/PR/edit) + the ELAI-measured MRR/Recall delta harness (`elai bench retrieval-adapt`, deterministic, never LLM-judge).
- DEFER Tier-2 (out-of-band GPU): the actual offline fine-tune / distillation run (one GPU box, hours) — RunPod/remote-gated, pod_stop after; the ONNX artifact it produces is the only thing that re-enters the product.

## Integration
- crates/knowledge/src/vector/ — the adapted-encoder ONNX load (rides J-18's exact-dim swap + index_version path; ~40 LOC over J-18).
- crates/knowledge/ — the distilled scoring-head ONNX module replacing the cross-encoder rerank call (~80 LOC; the head .onnx asset is exported out-of-band).
- crates/orchestrator or crates/knowledge — `elai retrieval gold-build` (assemble query/positive/hard-negative triples from sealed RRAs + issue/PR/edit history) + `elai bench retrieval-adapt` (MRR/Recall@k delta vs the current slot on the gold set).
- offline (NOT in crates) — the EvoEmbedding-loss contrastive trainer / MCompassRAG distillation script (Python, GPU box); produces the ONNX artifact only.
- SSB: PREFER-REUSE J-18 RetrievalAugmentEvent (Kind: EmbeddingModelSwapped already covers an adapted-model swap; add Kind ScoringHeadDistilled if the head variant ships). net 0 if J-18 unshipped at edit, baseline+1 from LIVE ~269/578 (post-J-66; grep 2026-06-22) otherwise — re-read at edit (W-9).
- cross_lang pair: none (Rust-internal knowledge crate + an out-of-band Python trainer); frontend_pair: the MRR/Recall delta on the existing KnowledgeDrawer (B-29) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/knowledge/src/vector (adapted-encoder ONNX load path, riding J-18's index_version migration)
- crates/knowledge (distilled scoring-head ONNX module replacing the cross-encoder rerank call)
- `elai retrieval gold-build` (ELAI gold-set assembler from sealed RRAs + issue/PR/edit)
- `elai bench retrieval-adapt` (deterministic MRR/Recall@k delta harness)
consumes_symbols:
- J-18 CodeRankEmbed swap path (the exact-dim ONNX slot + index_version migration; NOT re-authored)
- G-2 RRA sealed bundles (the agent-trace supervision; NOT re-authored)
- usearch HNSW 768d slot (UNCHANGED geometry — exact-dim adapted-model swap)
- ms-marco cross-encoder rerank seam (the scoring head REPLACES this call; the rerank interface is reused)
reachable_from:
- a real `elai` retrieval query (search/task path -> HYBRID cascade -> adapted encoder + scoring head)  [PRODUCTION]
- `elai bench retrieval-adapt` and `elai retrieval gold-build` (CLI entrypoints)  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO in-product training code (the fine-tune/distill is out-of-band; only the ONNX artifact re-enters)
- NO paper number as the done-gate (W1: the convergent lever is corroborated but the report's CORE-Bench numbers are UNVERIFIED/not-found — the gate is the ELAI-MEASURED delta on the ELAI gold set, never 20.3/32.8/60.1)
- NO dependency on an unofficial community ONNX quant (export from source; anti-slopsquat)
- NO embedder swap without an index_version bump + re-embed (a silent model change corrupts the HNSW index)
- NO KaLM online-decoder / EvoRAG memory-queue runtime (R1/R2 — not ONNX-CPU portable)
- NO new default-OFF ELAI_*_ENABLE without a wire-dark-allowlist row + promotion plan (R9/W-2)
frontend_pair: the MRR/Recall-delta breakdown on KnowledgeDrawer (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "`elai retrieval gold-build` then `elai bench retrieval-adapt` comparing the adapted encoder/head slot against the current GraphCodeBERT/ms-marco slot on the ELAI gold set, then a real `elai` retrieval query routed through the adapted slot"
- expect_ssb: "RetrievalAugmentEvent (Kind=EmbeddingModelSwapped on the index rebuild with the adapted model + index_version bumped; Kind=ScoringHeadDistilled if the head variant ships)"
- expect_artifact: "the bench report shows MRR/Recall@k for the adapted slot >= the current slot on the ELAI gold set (the number is ELAI-MEASURED). NEGATIVE (R2-fail-closed): an adapted model with a dim/geometry mismatch FAILS the index_version migration test, never silently corrupts the index; a missing adapted .onnx artifact DEGRADES to the current slot (cascade intact, not a crash); the scoring head loaded with a shape mismatch FAILS at load, never returns rank-k noise."

## Grounding
- MCompassRAG (workflow/papers/mcompassrag-interface.md): distill teacher -> ~2M-param 3-layer scoring head on a FROZEN ONNX embedder, replaces the cross-encoder, ranks at embedding latency, zero LLM calls. SoftLTM dep noted (.claude/papers/2602.17907.html).
- EvoEmbedding arXiv:2606.21649 (workflow/papers/evoembedding-interface.md, .claude/papers/2606.21649.html): length-weighted InfoNCE (tau=0.1), EvoTrain-180K schema maps onto ELAI's issue-PR-edit history. arXiv ID re-verify at edit.
- CodeRAG-system arXiv:2509.16112, EMNLP'25 (workflow/papers/coderag-system-interface.md, .claude/papers/2509.16112.html): distilled Qwen3-0.6B LoRA reranker from LLM preference pairs, 0.23s total query cost; ReccEval EM 47.48 vs DraCo 39.99 — DESIGN-rationale only (HD-1/ID-5).
- W1 WARNING (context-retrieval-research-2026-06-23.md PART 1): the report's headline "CORE-Bench" (Qwen3-Embedding-8B 20.3->32.8 nDCG / 48.0->66.4 Recall after in-domain SFT) is NOT FOUND as a paper (25+ searches / 15+ fetches; 2409.11363 is unrelated; CoREB/CoreCode/SweRank ruled out). The numbers are UNVERIFIED/possibly synthesized and MUST NOT be cited; the LEVER (in-domain adaptation) stands on the 3 primary sources above. Stub: workflow/papers/core-bench-interface.md.
- Source synthesis + ELAI seams codegraph-grounded 2026-06-23: workflow/reasoning/context-retrieval-research-2026-06-23.md (C1 + PART 2 meta-finding).
- LIVE SSB ~269/578 (feature-map stamp post-J-66, grep 2026-06-22; J-0_INDEX re-grep 2026-06-15 = 268/577). re-grep at edit (W-9).

## COMPOSES-WITH
J-18 (the embedder seam this folds into — exact-dim swap + index_version path; J-72 is the in-domain increment on J-18's CodeRankEmbed swap, ships AFTER or WITH it), G-2 RRA (the agent-trace supervision source), usearch HNSW + RRF cascade (UNCHANGED), ms-marco rerank seam (the scoring head replaces it), J-75 (the ablation harness measures whether the adapted slot earns its keep on the same gold set). Restates none.

## Open questions / risks
- the gold set is the bottleneck: ELAI's issue/PR/edit + sealed-RRA corpus must be large/clean enough to fine-tune without overfitting — the first checkbox is a held-out split + the J-75 ablation showing the adapted slot beats the stock slot, else don't ship the swap.
- two variants, one slot: variant a (encoder) and variant b (scoring head) are independently shippable; decide at edit which lands first (the head is cheaper to train and reversible — it's a rerank-call swap, not an index rebuild).
- out-of-band training drift: the adapted model is keyed by ELAI's corpus state; G-10 model-drift canary must invalidate the artifact when the corpus identity changes (compose, do not re-author the canary).

End of J-72.
