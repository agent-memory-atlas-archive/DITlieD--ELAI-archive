Source: MCompassRAG
Repo path: docs/.source/MCompassRAG/
Paper: SoftLTM referenced at arxiv.org/abs/2602.17907 (upstream topic model dep only; MCompassRAG itself has no arXiv paper)
Reliability: MEDIUM (reviewed source, research-status repo)
Files examined: src/models/compass_retriever.py, src/models/selector.py, src/models/abstraction.py, src/models/scorer.py, src/index/metadata_bank.py, src/inference/retrieve.py, src/pipeline.py, src/training/losses.py, src/training/trainer.py, src/training/train.py, src/training/dataset.py, src/encoders/retriever_encoder.py, data_gen/teacher.py, topic_models/base.py, configs/, README.md


CORE MECHANISM

MCompassRAG adds a "semantic compass" to dense retrieval: every chunk is annotated at index time with a topic distribution theta (K-dim simplex) from a pluggable topic model, and a compressed topic summary g_c = top-M weighted sum of topic centroids is concatenated with the chunk embedding to form the chunk representation r_c = [e_c ; g_c] (2d-dim). At query time a lightweight neural retriever (Selector + Abstraction + Scorer) derives a query topic summary g_q by attending to a bank of precomputed per-chunk topic distributions, producing r_q = [e_q ; g_q]. Final score is an MLP over [r_q ; r_c]. The LLM teacher (GPT-4o via OpenRouter) provides binary relevance labels and soft logit targets z_t during a one-time offline distillation phase; at inference no LLM calls occur.


ARCHITECTURE / DATA FLOW

INGEST
  src/pipeline.py::build_index_from_config
    -> load topic model from outputs/topic_models/<name>/
    -> src/index/metadata_bank.py::MetadataBank.build(chunks, encoder, topic_model, IndexConfig)
       (1) e_c = RetrieverEncoder.encode(texts, is_query=False)  -> (N, d) float32 L2-normed CPU
       (2) theta = topic_model.encode_topic_distribution(texts)  -> (N, K) simplex
       (3) T = topic_model.get_topic_centroids()                 -> (K, d) -- native or empirical
       (4) m = theta @ T                                         -> (N, d)  ALL-K selector input
       (5) g_c = top-M masked: mask K->M, g_c = masked_theta @ T -> (N, d)
       (6) r_c = [e_c ; g_c]                                     -> (N, 2d)
    -> MetadataBank.save(out_dir)  -- safetensors + chunks.jsonl + meta.json

TRAIN (distillation)
  data_gen/build_training_data.py -> data_gen/teacher.py::judge_candidates
    teacher_model = GPT-4o via OpenRouter, EXPANDED query vs candidates
    z_t = logp("1") - logp("0")   (teacher soft logit, not self-reported confidence)
    y   = 1 if chosen=="1" else 0
  src/training/train.py::main
    -> precompute_query_embeddings(records, encoder)  -- BASE query, cached safetensors
    -> CompassTrainer.train(train_ds, val_ds)
       optimizer: AdamW(selector + abstraction + scorer params only)
       loss: DistillLoss (alpha=0.5, tau=2.0)

RETRIEVE (Algorithm 1, zero LLM calls)
  src/inference/retrieve.py::CompassRAG.retrieve(query, k)
    e_q = RetrieverEncoder.encode([query], is_query=True)          -> (1, d)
    r_q, aux = CompassRetriever.build_query_rep(e_q)
      a_i = Selector(e_q, bank.m)                                  -> (1, N) logits
      idx, s_full, s_tilde = Selector.select_topl(a, top_l=16)     -> (1,L)
      H0 = theta_bank[idx]                                         -> (1, L, K)
      theta_hat_q = Abstraction(H0, s_tilde)                       -> (1, K) via 2-layer TransformerEncoder
      g_q = weighted_topic_summary(theta_hat_q, centroids, top_m)  -> (1, d)
      r_q = [e_q ; g_q]                                            -> (1, 2d)
    scores = CompassRetriever.score_all(r_q, bank.chunk_reps, block_size=4096) -> (1, N)
    top-k by score -> list[RetrievalResult]


PUBLIC API (from CODE)

MetadataBank.build(chunks: list[Chunk], encoder: RetrieverEncoder|None, topic_provider: TopicModel, cfg: IndexConfig) -> MetadataBank
MetadataBank.load(in_dir: str, mmap: bool = True) -> MetadataBank
MetadataBank.save(out_dir: str) -> None
MetadataBank.to(device: str) -> MetadataBank

CompassRetriever(cfg: CompassModelConfig)  -- nn.Module
CompassRetriever.set_index(meta_embeddings: Tensor, theta_bank: Tensor, centroids: Tensor) -> None
CompassRetriever.build_query_rep(e_q: Tensor) -> (r_q: Tensor, aux: dict)
CompassRetriever.score(r_q: Tensor, cand_r_c: Tensor) -> Tensor   -- (B,P) logits
CompassRetriever.score_all(r_q: Tensor, chunk_reps: Tensor, block_size: int = 4096) -> Tensor  -- (B,N)
CompassRetriever.trainable_parameters() -> Iterator[Parameter]

CompassRAG(encoder, model, bank, cfg: CompassRAGConfig)
CompassRAG.from_pretrained(bank_dir, model_ckpt, encoder=None, ...) -> CompassRAG
CompassRAG.retrieve(query: str, k: int|None = None) -> list[RetrievalResult]
CompassRAG.retrieve_batch(queries: list[str], k, batch_size) -> list[list[RetrievalResult]]
CompassRAG.build_query_reps(queries: list[str], batch_size: int|None) -> Tensor  -- (B, 2d)

RetrieverEncoder(model_name="Qwen/Qwen3-Embedding-4B", device=None, dtype=bfloat16, max_length=2048, query_instruction=str)
RetrieverEncoder.encode(texts: list[str], batch_size: int = 16, is_query: bool = False) -> Tensor  -- (B,d) float32 CPU L2-normed
RetrieverEncoder.from_backbone(backbone: Qwen3TextBackbone, ...) -> RetrieverEncoder

TopicModel (base class, topic_models/base.py)
  .encode_topic_distribution(texts, batch_size) -> (N, K) simplex Tensor
  .get_topic_centroids() -> (K, d) Tensor
  .supports_native_centroids: bool
  .fit_empirical_centroids(e_c, theta) -> None

CompassTrainer(model: CompassRetriever, bank: MetadataBank, cfg: TrainConfig)
CompassTrainer.train(train_ds, val_ds) -> None
CompassTrainer.load_model(path: str) -> CompassRetriever  -- static


TRAINING RECIPE (distillation)

TEACHER MODEL: GPT-4o via OpenRouter (configurable: datagen.teacher_model)
  Prompt template (data_gen/teacher.py:TEACHER_SYSTEM, TEACHER_USER_TEMPLATE):
    System: "You are a strict relevance judge for retrieval. Decide whether the candidate passage provides DIRECT or SUPPORTING evidence to answer the query. Answer with a single digit and nothing else: 1 for relevant, 0 for not relevant."
    User:   "You are given a question and a candidate knowledge chunk. Decide whether the chunk contains information that is useful for answering the question.\n\nMark the chunk as relevant only if it provides direct or supporting evidence needed to answer the question. Do not mark a chunk as relevant based only on vague topical similarity.\n\nQuestion: {query}\n\nCandidate chunk: {candidate}\n\nOutput only one number:\n1 = relevant\n0 = not relevant"
  Teacher reads token log-probs: z_t = logp("1") - logp("0")  (data_gen/teacher.py:96)
  Teacher always sees the EXPANDED query; student trains on BASE query (information asymmetry)

STUDENT MODEL: CompassRetriever (Selector 2d->1, Abstraction 2-layer TransformerEncoder K-dim, Scorer 3-layer MLP 4d->hidden->1)
  Encoder (frozen): Qwen/Qwen3-Embedding-4B (last-token pool + L2 normalize, bfloat16)
  d = 2560 (Qwen3-Embedding-4B hidden size), K = 100 (default topics)

LOSS FUNCTION (src/training/losses.py::DistillLoss.forward):
  L = (1 - alpha) * BCE + alpha * KD
  BCE = binary_cross_entropy_with_logits(logits, y)  over valid candidates
  KD  = Bernoulli KL: KL(sigma(z_t/tau) || sigma(z/tau))
       = pt*(log_pt - log_ps) + (1-pt)*(log(1-pt) - log(1-ps))
       where ps = sigma(z/tau), pt = sigma(z_t/tau)
  if tau_squared_scale: kl_el *= tau * tau  (compensates KL scale by temperature)
  Defaults: alpha=0.5, tau=2.0, tau_squared_scale=True

DATASET CONSTRUCTION (configs/gen_train_data.yaml):
  n_target_chunks: 2000  (chunks to generate queries for)
  queries_per_chunk: 10  (base + expanded pairs per chunk)
  n_random_negatives: 4
  n_hard_candidates: 20, n_hard_negatives: 4
  neighbor_window: 1  (adjacent chunks used for expanded query)
  query_gen_model: openai/gpt-4o, temperature_qgen: 0.7
  teacher_model: openai/gpt-4o, temperature_teacher: 0.0

OPTIMIZER + SCHEDULE (src/training/trainer.py::build_optimizer_scheduler):
  AdamW, lr=2e-4, weight_decay=0.01
  Linear warmup (warmup_ratio=0.05 of total_steps) then linear decay to 0
  grad_clip: 1.0
  epochs: 5, batch_size: 32
  Trainable params only: Selector (Linear 2d->1) + Abstraction (2-layer TransformerEncoder K-dim) + Scorer (MLP 4d->1024->256->1)
  Index buffers (m, theta_bank, centroids) never touch optimizer -- asserted in trainer


VERBATIM EXTRACTS (MANDATORY)

LOSS FUNCTION (src/training/losses.py:38-65):
  "bce_el = F.binary_cross_entropy_with_logits(logits, y, reduction='none')
   bce = (bce_el * cand_mask).sum() / cand_mask.sum().clamp_min(1.0)
   ls = logits / self.tau
   lt = z_t / self.tau
   log_ps = F.logsigmoid(ls)
   log_1mps = F.logsigmoid(-ls)
   log_pt = F.logsigmoid(lt)
   log_1mpt = F.logsigmoid(-lt)
   pt = log_pt.exp()
   one_mpt = log_1mpt.exp()
   kl_el = pt * (log_pt - log_ps) + one_mpt * (log_1mpt - log_1mps)
   if self.tau_squared_scale:
       kl_el = kl_el * (self.tau * self.tau)
   kd = (kl_el * kd_mask).sum() / kd_mask.sum().clamp_min(1.0)
   loss = (1.0 - self.alpha) * bce + self.alpha * kd"

SELECTOR (src/models/selector.py:39-48):
  "w = self.linear.weight.view(-1)  # (2d,)
   w_q = w[:d]  # (d,)
   w_m = w[d:]  # (d,)
   bias = self.linear.bias  # (1,)
   a_q = e_q @ w_q  # (B,)
   a_m = m @ w_m    # (N,)
   a = a_q[:, None] + a_m[None, :] + bias  # (B, N)  -- avoids (B,N,2d) materialization"

TOPIC SUMMARY (src/models/compass_retriever.py:55-64):
  "th = theta.float()
   Tf = T.float()
   K = th.shape[-1]
   m = min(int(top_m), K)
   topk_idx = th.topk(m, dim=-1).indices  # (B, m)
   mask = torch.zeros_like(th)
   mask.scatter_(-1, topk_idx, 1.0)
   masked_theta = th * mask  # (B, K)
   g = masked_theta @ Tf  # (B, d)
   return g.float()"

ABSTRACTION POOLING (src/models/abstraction.py:74-81):
  "if self.pool == 'mean':
       theta_hat = out.mean(dim=1)  # (B, K)
   else:  # selection_weighted
       w = s_tilde.float().unsqueeze(-1)  # (B, L, 1)
       theta_hat = (w * out).sum(dim=1)   # (B, K)
   if self.apply_softmax:
       theta_hat = F.softmax(theta_hat, dim=-1)"

TEACHER LOGIT (data_gen/teacher.py:93-96):
  "lp1 = _LOGPROB_FLOOR if lp1 is None else float(lp1)
   lp0 = _LOGPROB_FLOOR if lp0 is None else float(lp0)
   z_t = lp1 - lp0   # teacher logit"

ENCODER (src/encoders/retriever_encoder.py:88-106):
  "if is_query:
       prepared = [f'Instruct: {self.query_instruction}\nQuery: {t}' for t in texts]
   ...
   pooled = Qwen3TextBackbone.pool_last_token(last_hidden, attn)  # last-token pool
   pooled = F.normalize(pooled.float(), p=2, dim=-1)              # L2 normalize
   out_chunks.append(pooled.cpu())"

METADATA BANK INVARIANT ASSERTIONS (src/index/metadata_bank.py:128-153):
  "row_sums = theta.sum(dim=-1)
   if not torch.allclose(row_sums, torch.ones_like(row_sums), atol=1e-4):
       raise ValueError('theta rows do not sum to 1 (atol=1e-4)')
   if not torch.allclose(r_c[:, :d], e_c, atol=1e-6):
       raise ValueError('r_c[:, :d] does not match e_c')
   if not torch.allclose(r_c[:, d:], g_c, atol=1e-6):
       raise ValueError('r_c[:, d:] does not match g_c')"

MODEL CONFIG DEFAULTS (src/models/compass_retriever.py::CompassModelConfig):
  "top_l: int = 16       # bank entries Selector pulls
   top_m: int = 8        # topics in g_c / g_q
   abstraction_layers: int = 2
   abstraction_heads: int = 4
   abstraction_ff: int = 256
   abstraction_dropout: float = 0.1
   abstraction_softmax: bool = True
   pool: str = 'selection_weighted'
   scorer_hidden: tuple[int, int] = (1024, 256)
   scorer_dropout: float = 0.1"


HYPERPARAMETERS

top_l       | 16    | >=1 <=N  | controls bank coverage; higher = richer query topic rep but O(L*K) abstraction cost
top_m       | 8     | >=1 <=K  | MUST match index and model; controls topic summary sparsity
K           | 100   | 20-200   | num topics; higher = finer discrimination, larger theta/T tensors; K must be divisible by abstraction_heads
alpha       | 0.5   | [0,1]    | BCE vs KD weight; 0=pure BCE, 1=pure distillation; paper uses 0.5
tau         | 2.0   | >0       | distillation temperature; lower=sharper teacher distribution; interacts with tau_squared_scale
tau_squared_scale | True | bool | multiply KL by tau^2 to compensate KL scale; recommended True
lr          | 2e-4  | 1e-5..1e-3 | only selector+abstraction+scorer; encoder always frozen
epochs      | 5     | 3-20     | few epochs sufficient; encoder frozen so only tiny heads trained
batch_size  | 32    | 8-128    | limited by memory of (B, P, 2d) score matrix
top_m == index.top_m: MUST match or g_q / g_c representations mis-align (enforced at load time)


REIMPLEMENTATION NOTES FOR ELAI

ELAI retrieval stack: dual ONNX embedders (GraphCodeBERT 768d + Granite 384d) + tantivy BM25 + PageRank + RRF(k=60) + ms-marco cross-encoder reranker. All CPU/ONNX. No LLM at inference.

KEY QUESTION ANSWER: YES, the MCompassRAG distill-teacher-into-tiny-retriever recipe gives ELAI a portable path to an in-domain-adapted retriever WITHOUT an LLM at inference. The critical insight is:
  (1) The STUDENT (Selector + Abstraction + Scorer) is ~3 PyTorch layers totaling ~2M params, easily ONNX-exportable.
  (2) The ENCODER is FROZEN throughout training -- only the tiny heads learn. ELAI can keep its existing ONNX embedders and adapt only the scoring head.
  (3) The teacher labeling (GPT-4o) is a ONE-TIME offline cost -- ~2000 chunks * 10 queries = 20K teacher calls; at current pricing ~$10-20 in tokens.
  (4) The MetadataBank is a plain safetensors file loadable without PyTorch at inference (tensors only), CPU-compatible.

LOAD-BEARING COMPONENTS:
  A. Topic model (CEMTM/ETM/softLTM) -> produces (K, d) centroids + (N, K) theta per chunk
     Adaptation for ELAI: run a 100-topic ETM on ELAI's code corpus (Rust/Python snippets). ETM is self-contained, no third_party dep.
  B. Selector (Linear 2d->1, ~5K params) + Abstraction (2-layer Transformer on K-dim, ~200K params) + Scorer (MLP, ~1.5M params)
     These are tiny; ONNX export is straightforward.
  C. MetadataBank: precomputed at index time, mmap-loaded at serve time. Per-chunk overhead: 2d floats for r_c + K floats for theta. At d=768 (GraphCodeBERT) and K=100: 2*768+100=1636 floats = ~6.4 KB/chunk; 100K chunks = 640 MB.
  D. DistillLoss (ELAI-portable): pure PyTorch, no dependencies. Copy verbatim.

WHAT DOES NOT MAP:
  - Qwen3-Embedding-4B backbone (4B params, GPU-required for training). ELAI uses ONNX embedders. Replacement: use ELAI's existing GraphCodeBERT 768d as the embedding space. Topic centroids must live in the SAME space as chunk embeddings.
  - PyTorch training loop -- ELAI has no PyTorch train infra. Training (offline, one-time) requires a GPU machine (the topic model + distillation is the only GPU-required step). Once trained, export Selector+Abstraction+Scorer to ONNX for inference.
  - OpenRouter/GPT-4o for teacher labels -- can substitute any LLM with logprob access. ELAI's existing Sonnet 4.6 lane can serve as teacher.
  - WikiWeb2M corpus for topic model training -- replace with ELAI's own code corpus.


CRITICAL INVARIANTS (testable assertions)

assert theta.sum(dim=-1).allclose(torch.ones(N), atol=1e-4), "topic distributions must be on simplex"
assert r_c[:, :d].allclose(e_c, atol=1e-6), "chunk rep first half must equal raw embedding"
assert r_c[:, d:].allclose(g_c, atol=1e-6), "chunk rep second half must equal topic summary"
assert model.cfg.top_m == bank.cfg.top_m, "top_m must match between model and index"
assert torch.isfinite(loss), "distillation loss must be finite (non-finite = training diverged)"
assert not any(p.data_ptr() in buffer_ptrs for p in trainable), "index buffers must not be in optimizer"
assert r_q[:, :d].equal(e_q), "query rep first half must equal raw query embedding (compass_retriever.py:154)"


COMPLEXITY

Index build: O(N * d) embed + O(N * K * d) topic encoding + O(N * K) theta@T; N=corpus size
Training: O(epochs * |D| * (L*K + 4d^2)) per step; L=top_l, K=topics, d=embedding dim; encoder frozen
Inference per query: O(d * N) selector + O(L * K^2) abstraction + O(P * 4d) scorer; P=candidates; all matrix multiplies, no LLM
Memory: bank.chunk_reps (N, 2d) float32 = 2*d*4*N bytes; at d=768, N=100K: ~600 MB VRAM or CPU RAM
Parallelism: score_all blocked in chunks of block_size=4096 to bound peak memory; fully batch-parallelizable


FAILURE MODES / PITFALLS

1. top_m mismatch between index and model: MetadataBank bakes g_c with one top_m; model recomputes g_q with another. Representations mis-align silently (shapes are identical). Enforced at CompassRAG init with ValueError.
2. Encoder space mismatch: topic centroids T must live in the SAME embedding space as chunk embeddings e_c. If topic model uses a different backbone (e.g. cwtm uses bert-base-uncased and empirical centroids), centroids are mean of chunk embeddings -- correct but noisier.
3. selection_weighted vs mean pool: plain mean pool severs the gradient path into the Selector. Only selection_weighted provides gradient through s_tilde into the selector. Using pool="mean" trains abstraction+scorer but the selector gets no gradient signal (mentioned in abstraction.py docstring, line 12-13).
4. Teacher batch_size=1 forced: judge_candidates ignores teacher_batch_size parameter (line 63: "unused: logit readout requires one candidate per call"). Any attempt to batch teacher calls will silently produce wrong z_t.
5. K must divide abstraction_heads: TransformerEncoderLayer requires d_model (=K) % nhead == 0. Asserted at Abstraction init.
6. Safetensors on Windows: safe_open with mmap=True may have file-lock issues on Windows. Set mmap=False for write-then-read in same process.
7. Topic model must be trained BEFORE index build: build_index_from_config loads topic_model.dir which must exist with trained weights. Running out of order produces a random-centroid index.
8. Query embedding uses instruction prefix; chunk embedding does not: asymmetric encoding is intentional (Qwen3-Embedding convention). Breaking this symmetry (encoding chunks with instruction) degrades retrieval quality.
