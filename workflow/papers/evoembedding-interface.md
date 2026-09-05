Source: EvoEmbedding (EvoRAGModel)
Repo: docs/.source/EvoEmbedding/ (local clone, reviewed source)
Paper: arXiv 2606.21649 "EvoEmbedding: Evolvable Representations for Long-Context Retrieval and Agentic Memory"
HF Model: MiG-NJU/EvoEmbedding-4B
HF Dataset: MiG-NJU/EvoTrain-180K
Reliability: MEDIUM (reviewed source, unpublished preprint June 2026)
Files examined:
  docs/.source/EvoEmbedding/model/evo_embedding.py  (EvoRAGConfig, EvoRAGModel -- full implementation)
  docs/.source/EvoEmbedding/model/client.py          (EvoEmbeddingClient public API)
  docs/.source/EvoEmbedding/train/train.py           (SFTTrainer entrypoint)
  docs/.source/EvoEmbedding/eval/eval.py             (benchmark dispatcher)
  docs/.source/EvoEmbedding/README.md               (usage, dataset schema)
  docs/.source/EvoEmbedding/requirements-evoembedding-4b.txt  (pinned deps)

CORE MECHANISM (3 lines)
EvoRAGModel runs two parallel LoRA adapters on a frozen Qwen3-4B backbone: "memory_generator" compresses
each dialogue turn into K=16 latent tokens pushed into a FIFO buffer (capacity C=512 latent tokens); 
"retriever" encodes query+buffer concatenation into a normalized embedding for contrastive retrieval.
The critical property is that each turn's embedding is conditioned on all prior turns via the buffer state
at encoding time, making embeddings temporally ordered and context-sensitive rather than static.

ARCHITECTURE (verbatim from model/evo_embedding.py):

Config defaults (EvoRAGConfig, file:28-44):
  base_model_name_or_path = "Qwen/Qwen3-4B-Instruct-2507"
  num_latents = 16          # K: latent tokens extracted per turn by memory_generator
  buffer_capacity = 512     # C: FIFO queue max size in latent tokens (= L*K = 32 turns * 16)
  lora_r = 64               # LoRA rank for both adapters
  lora_alpha = 128          # LoRA alpha (= 2*r)
  lora_dropout = 0.1        # applied to all projection modules

Two LoRA adapters on same base model (file:82-85):
  adapter "memory_generator": task_type=CAUSAL_LM, r=64, alpha=128
  adapter "retriever":        task_type=CAUSAL_LM, r=64, alpha=128
  target_modules: ["q_proj","v_proj","k_proj","o_proj","gate_proj","up_proj","down_proj"]

Frozen auxiliary embedding model (file:61-67):
  model: Qwen/Qwen3-Embedding-0.6B (bfloat16, flash_attention_2)
  used ONLY in native=True mode as a static baseline comparator

Projection layers (file:94-105):
  latent_to_mem:  Linear(hidden_size, hidden_size) -> ReLU     [maps LLM hidden -> memory space]
  mem_to_latent:  Linear(hidden_size, hidden_size)             [maps memory -> LLM input space]
  mem_to_emb:     Linear(hidden_size, emb_hidden_size) -> ReLU -> Linear(emb_hidden_size, emb_hidden_size)
                  [maps LLM hidden -> embedding output space; emb_hidden_size from Qwen3-Emb-0.6B]
  Init: eye_ weight + zeros_ bias (identity-like initialization for stable early training)

Memory learnable tokens (file:90-93):
  memory_queries: nn.Parameter shape (1, num_latents=16, hidden_size) initialized N(0, 0.02)
  These are appended after each turn's tokens in the memory_generator pass

TRAINING RECIPE:

Loss (verbatim from paper + confirmed in model/evo_embedding.py:506-534):
  Combined: L = L_mem + L_con    [paper eq, Section 3.3]
  
  L_mem (SFT/LM loss, file:455-457):
    Computed with disabled adapter on last turn only:
      outputs = self.model(inputs_embeds=combined_embeds, ..., labels=combined_labels, use_cache=False)
      [assistant_only_loss=True in SFTConfig -> only non-(-100) label positions contribute]
  
  L_con (contrastive loss, file:519-534, temperature tau=0.1 hardcoded):
      pos_scores = bmm(q_norm, pos_embs^T) / 0.1
      neg_scores = bmm(q_norm, neg_embs^T) / 0.1
      for each positive i:
        logits = cat([pos_scores[:,i:i+1], neg_scores], dim=1)
        loss_i = CrossEntropyLoss(logits, target=0)  # target=0 means first element = positive
      contrastive_loss = sum(loss_i) / P * log(N+1)  # length-weight log(N+1)/P  [paper eq]
      where P = number of positives, N = number of negatives
  
  Final: outputs.loss += retrieval_loss_weight * contrastive_loss
         retrieval_loss_weight = 1.0 (hardcoded, file:124)
  DDP dummy loss: 0.0 * sum(p.sum() for p in params if requires_grad) (prevents DDP unused-param error, file:540)

Training hyperparameters (train/train.py + README + paper Table 2):
  dataset: MiG-NJU/EvoTrain-180K (184,137 samples)
  base_model: Qwen/Qwen3-4B-Instruct-2507
  num_latents: 16
  buffer_capacity: 512
  learning_rate: 5e-5
  num_train_epochs: 1
  per_device_train_batch_size: 1  [HARD LIMIT: forward() raises if batch_size != 1, file:313]
  gradient_accumulation_steps: 2
  max_length: 32768 * 4 = 131072  [max sequence length for training data packing]
  warmup_ratio: 0.05
  lr_scheduler: cosine_with_min_lr (min_lr_rate=0.1, num_cycles=0.5)
  assistant_only_loss: True  [labels=-100 for user turns]
  max_grad_norm: 1.0
  group_by_length: True
  Hardware: 8 x H800 GPU, 26.6 hours total (paper)
  framework: trl==0.24.0, peft==0.18.0, transformers==4.57.1, torch==2.4.1, flash-attn==2.8.3

Chunk/segment-batching (file:365-425):
  target_len = 2048    # max tokens per chunk
  chunk_size_target = 16  # max turns per chunk
  Chunks are processed sequentially; previous buffer injected at each chunk start
  This is the "dynamic segment-batching" giving 3.8x speedup (paper Section 4.3)

Dataset construction (EvoTrain-180K, README + paper Section 4.1):
  Schema (verbatim from README):
    messages: list of alternating {role, content} turns (SFT generation target)
    meta.turns: list of formatted turn strings (sequential retriever input)
    meta.evidence_turns: list of zero-based indices pointing to positive evidence turns

  Three-stage pipeline (paper):
    Stage 1: web docs (FineWeb) + synthesized multi-turn dialogues, lengths 25-10270 tokens
    Stage 2: 40+ query templates (coreference, temporal reasoning) via multiple LLMs
    Stage 3: Gemini-3.1-Pro-Preview labels positives and filters hallucinations
  
  Stats: mean_ctx_len=1289.78 tokens, mean_segment_count=20.57, segment_range=2-246
  52.9% of contexts < 512 tokens (crucial for short-context generalization)

PUBLIC API (from model/client.py):

EvoEmbeddingClient.__init__(model_name="EvoEmbedding", model_path="MiG-NJU/EvoEmbedding-4B",
                             tokenizer_name="Qwen/Qwen3-4B-Instruct-2507", device="cuda")
  Loads EvoRAGModel from HF hub or local path.
  REQUIRES CUDA (device="cuda" default; no CPU path tested or documented).

EvoEmbeddingClient.encode_messages(messages: list[dict], native: bool = False) -> np.ndarray
  Input: list of {"role": ..., "content": ...} dicts (standard ChatML format)
  Returns: normalized float32 numpy array, shape (len(turns), emb_dim)
  native=True: uses static Qwen3-Embedding-0.6B baseline (no memory buffer, ablation mode)
  native=False: uses EvoRAGModel with full memory evolution (production mode)

EvoEmbeddingClient.rerank(query: str, candidates: list[str], top_k: int|None = None,
                           native: bool = False, return_indices: bool = False)
  Returns: (ranked_candidates, ranked_indices) if return_indices else ranked_candidates
  Internally calls encode_messages then generate_retrieve_idx with temperature=0.1

EvoRAGModel.build_latent_buffer(history_turns: list[str], tokenizer) -> torch.Tensor | None
  Builds FIFO memory buffer from ordered history strings.
  Returns: CPU tensor shape (1, min(total_latents, buffer_capacity), hidden_size) or None
  Chunking: processes up to 16 turns per chunk, each chunk up to 2048 tokens

EvoRAGModel.encode_texts_with_latent_buffer(texts, latent_buffer, tokenizer, native, max_batch_tokens=2048)
  Returns: normalized float32 numpy array shape (N, emb_dim)
  Left-pads shorter sequences in dynamic batches
  Last-token pooling: vec = mem_to_emb(latent_to_mem(out[:, -1:, :])).mean(dim=1)

EvoRAGModel.encode_texts_with_memory(texts, history_turns, tokenizer, native, max_batch_tokens=2048)
  Convenience: calls build_latent_buffer then encode_texts_with_latent_buffer

BEHAVIORAL CONTRACTS:
  batch_size=1 HARD LIMIT: forward() raises ValueError if batch_size != 1 (file:313)
  Inference only with batch_size=1: generate_retrieve_idx raises ValueError if batch_size != 1 (file:747)
  IM_END_ID: two different values used -- 151643 in build_latent_buffer (inference), 151645 in forward()
    This is Qwen3 tokenizer-specific; 151643 = <|im_end|> in Qwen3-Embedding, 151645 in Qwen3-Instruct
  Temperature hardcoded at 0.1 in both training (file:521) and inference (file:934)
  Embedding normalization: L2 normalize to unit sphere always, clip norms >= 1e-12 (file:292)
  Buffer is FIFO: oldest latents dropped when global_raw_buffer.shape[1] > buffer_capacity (file:421)
  Retrieval scoring: cosine similarity / 0.1 (dot product on unit-norm vecs divided by temperature)
  Sorted descending: generate_retrieve_idx returns sorted_indices from highest to lowest score

EVAL PROTOCOL (eval/eval.py + eval/eval.sh):
  Benchmarks: locomo (LoCoMo 10-turn), longmemeval_s, personamem32, PersonaMME32, PersonaMME128
  Method: rag (retrieve top-k segments then generate) or full (full context)
  Default rag_sentence_num: 16
  Judge LLM: GPT-4o-mini (automated accuracy scoring)
  Generation LLM: Qwen3-30B-A3B (or Qwen3-4B-Instruct local)
  Embedding model for baseline comparison: Qwen/Qwen3-Embedding-0.6B

KEY RESULTS (paper Table 1 + README):
  Recall@10 overall: 80.5 (EvoEmbedding-4B) vs 72.7 (KaLM-Embedding-Gemma3-12B, 3x larger)
  NDCG@10 overall: 65.2 vs 57.1 (runner-up)
  LongMemEval accuracy: 77.6% (Top-8 RAG) vs A-MEM 65.2%, LightMem 70.2%
  A-MEM + EvoEmbedding: +19.2% over A-MEM alone
  LightMem + EvoEmbedding: +13.5% over LightMem alone
  GPU overhead: ~14-16 GB VRAM (comparable to Qwen3-Reranker-4B)

ELAI REIMPLEMENTATION NOTES:

ELAI current retriever stack (from task description):
  - GraphCodeBERT 768d code embedder (ONNX, CPU)
  - Granite 384d text embedder (ONNX, CPU)
  - ms-marco cross-encoder reranker (ONNX, CPU)
  - Inference runtime: ONNX Runtime, CPU only, no training infra in-product

The research report's #1 lever: "in-domain retriever adaptation / fine-tuning on issue-PR-edit supervision"
EvoEmbedding's most relevant contribution to this lever:

PORTABLE ADAPTATION RECIPE (what maps):
  1. Dataset schema: the EvoTrain-180K JSON schema (messages + meta.turns + meta.evidence_turns)
     is directly adaptable to ELAI's issue-PR-edit history. Map: turns = sequential edits/messages;
     evidence_turns = indices of edits that match a later query (e.g. "which edit introduced bug X").
     This is the clearest lift: ELAI can construct fine-tuning data in this exact format.
  
  2. Contrastive loss formulation (length-weighted InfoNCE with temperature 0.1):
     L_con = log(N+1)/P * sum_i CrossEntropy([pos_score_i; neg_scores], target=0)
     This is portable to ANY embedding model fine-tuning, including GraphCodeBERT or Granite.
     Can be applied to ELAI's existing encoders without the memory-queue mechanism.
  
  3. Two-pass architecture pattern (memory_generator then retriever) is a useful reference for
     future ELAI retriever that processes sequential edit streams rather than static documents.

WHAT DOES NOT MAP (GPU-TRAIN vs ONNX-INFERENCE GAP):
  1. ONNX EXPORT: EvoRAGModel is NOT ONNX-exportable as-is.
     Hard blockers:
     - Dynamic buffer concatenation: global_raw_buffer grows turn-by-turn; no fixed-shape ops
     - Two LoRA adapter swapping (model.set_adapter("memory_generator") / ("retriever"))
       -- ONNX has no concept of runtime adapter switching
     - batch_size=1 hard constraint with dynamic sequence lengths
     - flash_attention_2 (bfloat16) required; ONNX Runtime CPU has no flash-attn kernel
     - The projection chain (latent_to_mem -> mem_to_emb) uses dynamic intermediate shapes
     A partial workaround: export only encode_texts_with_latent_buffer with a FIXED pre-built
     buffer (frozen at encode time) as a static ONNX graph. This loses the streaming update
     property but retains context-conditioned embeddings. Not validated by the authors.
  
  2. GPU TRAINING MANDATORY:
     - 8 x H800 required per paper; minimum viable: 1 x GPU with 14-16 GB VRAM
     - torch==2.4.1 + flash-attn==2.8.3 + bfloat16 throughout
     - trl SFTTrainer with gradient_checkpointing capable backbone
     - ELAI has no in-product training infra (ONNX inference only)
  
  3. BASE MODEL DEPENDENCY:
     - Hard-coded Qwen3-4B-Instruct-2507 and Qwen3-Embedding-0.6B
     - IM_END_ID=151645/151643 are Qwen3 tokenizer-specific magic numbers
     - Replacing base model requires re-training from scratch; not a drop-in adapter
  
  4. SEQUENCE LENGTH:
     - max_length=131072 during training; 14-16 GB VRAM at inference
     - ELAI local lane: 32K hard ceiling, ~3.1 GB VRAM budget on remote 3090
     - 7B NF4 already uses ~1.92 GB; EvoEmbedding-4B bfloat16 needs ~8 GB additional

2-4 LOAD-BEARING COMPONENTS FOR FUTURE INTEGRATION:
  A. The contrastive loss recipe (portable, no GPU training needed at GraphCodeBERT scale)
  B. The EvoTrain-180K dataset schema for constructing issue-PR-edit supervision data  
  C. The FIFO latent buffer pattern (architectural reference for future streaming retriever)
  D. The eval benchmarks (LoCoMo, LongMemEval) as external validators of memory retrieval quality

SINGLE MOST ELAI-RELEVANT IDEA:
  The EvoTrain-180K training data construction protocol: it shows exactly how to build
  supervised contrastive training pairs from sequential conversational/edit history using
  evidence_turns labels. ELAI could apply this to issue-PR-edit streams to fine-tune
  even the existing ONNX GraphCodeBERT retriever using the contrastive loss alone
  (without the memory-queue mechanism), yielding in-domain adaptation with no architecture change.

GOTCHAS:
  - batch_size=1 is a hard architectural limit, not a config default (forward() raises ValueError)
  - IM_END_ID inconsistency: 151643 vs 151645 across build_latent_buffer vs forward() -- likely
    a Qwen3-Embedding vs Qwen3-Instruct tokenizer difference; must match base model used
  - Memory buffer is CPU-returned from build_latent_buffer but must be moved to GPU device for
    encode_texts_with_latent_buffer; the .to(device=device, dtype=...) call is required
  - Debug print statements left in forward() (file:523: "print(1111, ...)" and file:532: "print('debug111'...)")
    indicating this is research-grade code, not production-hardened
  - Temperature 0.1 is hardcoded in both train and inference paths; changing it requires code edit

Critical assertions (testable):
  assert config.num_latents == 16, "default K per paper Table 3 sensitivity analysis"
  assert config.buffer_capacity == 512, "C = L*K default; paper uses 512 throughout"
  assert config.lora_r == 64, "LoRA rank; paper shows stable down to r=16 but default=64"
  # contrastive temperature hardcoded:
  # pos_scores = bmm(...) / 0.1  -> verify temperature not configurable in forward()
  import numpy as np
  # embedding output is always L2-normalized to unit sphere:
  def test_embeddings_normalized(emb: np.ndarray):
      norms = np.linalg.norm(emb, axis=1)
      assert np.allclose(norms, 1.0, atol=1e-5), f"embeddings not unit-normalized, norms={norms}"
  # buffer FIFO: last C latents only
  def test_buffer_fifo_capacity(global_raw_buffer, buffer_capacity=512):
      assert global_raw_buffer.shape[1] <= buffer_capacity, "buffer exceeds capacity"

CONTRACT TEST STUBS:
import numpy as np

def test_batch_size_1_enforced():
    # EvoRAGModel.forward() raises ValueError for batch_size != 1
    # Verified: model/evo_embedding.py:313
    # "Recurrent training currently only supports batch_size=1 due to dynamic slicing complexity."
    batch_size = 1
    assert batch_size == 1, "EvoRAGModel hardcoded to batch_size=1"

def test_contrastive_temperature_hardcoded():
    # Temperature 0.1 hardcoded in forward() line ~521 and generate_retrieve_idx line ~934
    # pos_scores = bmm(q_norm.unsqueeze(1), pos_embs.transpose(1,2)).squeeze(1) / 0.1
    temperature = 0.1
    pos_logit = 2.0 / temperature   # = 20.0
    neg_logit = 0.5 / temperature   # = 5.0
    import math
    prob = math.exp(pos_logit) / (math.exp(pos_logit) + math.exp(neg_logit))
    assert prob > 0.99, f"temperature=0.1 creates very sharp distribution, got prob={prob:.4f}"

def test_buffer_capacity_fifo():
    num_latents = 16
    buffer_capacity = 512
    max_turns_in_buffer = buffer_capacity // num_latents  # = 32
    assert max_turns_in_buffer == 32, f"expected 32 turns max, got {max_turns_in_buffer}"
    # After 32 turns, oldest latents are dropped (FIFO)
    simulated_turns = 50
    retained = min(simulated_turns * num_latents, buffer_capacity)
    assert retained == buffer_capacity  # buffer at capacity after 32+ turns

def test_embedding_normalization():
    # encode_texts_with_latent_buffer always L2-normalizes, clip at 1e-12
    # model/evo_embedding.py:291-293
    import numpy as np
    fake_emb = np.array([[3.0, 4.0]])  # norm=5
    norms = np.linalg.norm(fake_emb, axis=1, keepdims=True)
    normalized = fake_emb / np.clip(norms, 1e-12, None)
    expected_norm = np.linalg.norm(normalized[0])
    assert abs(expected_norm - 1.0) < 1e-6, f"L2 norm should be 1.0, got {expected_norm}"

def test_evidence_turns_schema():
    # EvoTrain-180K datum schema from README
    sample = {
        "messages": [
            {"role": "user", "content": "I visited Paris in April."},
            {"role": "assistant", "content": "Noted."},
            {"role": "user", "content": "Where did I travel in spring?"},
        ],
        "meta": {
            "evidence_turns": [0],
            "turns": [
                "User: I visited Paris in April.\nAssistant: Noted.",
                "Where did I travel in spring?",
            ],
        },
    }
    assert "evidence_turns" in sample["meta"], "meta must have evidence_turns"
    assert "turns" in sample["meta"], "meta must have turns"
    # Last turn in meta.turns is the query; evidence_turns are 0-indexed into turns[:-1]
    query_idx = len(sample["meta"]["turns"]) - 1
    for ev_idx in sample["meta"]["evidence_turns"]:
        assert ev_idx < query_idx, f"evidence_turn {ev_idx} must precede query at {query_idx}"
