Source: SproutRAG
Repo path: docs/.source/SproutRAG/
Paper: No arXiv paper linked in README; no paper field in pyproject.toml
Reliability: MEDIUM (reviewed source, research-status package)
Files examined: sproutrag/indexing/indexer.py, sproutrag/aggregation/attention_aggregator.py, sproutrag/retrieval/hierarchical_retriever.py, sproutrag/retrieval/multi_document_retriever.py, sproutrag/retrieval/scoring.py, sproutrag/tree/builder.py, sproutrag/training/losses.py, sproutrag/encoding/sllm_encoder.py, sproutrag/data/schema.py, sproutrag/config/defaults.py, sproutrag/reranking/cross_encoder.py, README.md


CORE MECHANISM

SproutRAG builds a hierarchical binary tree index over each document at index time. The tree is built by agglomerative merging: chunks start as leaf nodes; at each step the pair with highest mutual attention score is merged into a parent node whose embedding is the mean of its children. The mutual attention matrix is derived from the encoder's attention weights (aggregated across layers and heads via learned weights). At query time, beam-search walks the tree top-down, scoring each node by cosine similarity to the query embedding, collecting nodes that meet a threshold. This produces multi-granularity evidence: leaf nodes (fine-grained) and internal nodes (summaries of merged spans) can both be returned. An optional cross-encoder reranker re-scores the collected set.


ARCHITECTURE / DATA FLOW

INGEST (sproutrag/indexing/indexer.py::SproutRAGIndexer.index_document)
  RawDocument -> chunk_sentences(doc, max_sentences_per_chunk=2) -> list[SentenceChunk]
  SLLMEncoder.encode_document(chunks, return_attentions=True)
    -> concatenates chunk texts with '\n', runs single forward pass
    -> extracts per-chunk embeddings via mean-pool over token spans (offset_mapping)
    -> extracts attention tensor (num_layers, num_heads, num_chunks, num_chunks)
    -> returns EncodedDocument(embeddings, attentions, chunk_ids, metadata)
  AttentionAggregator(attentions)
    -> learned weights alpha (num_layers, num_heads), softmax-normalized
    -> mutual = (einsum(weights, attentions) + transpose) / 2, diagonal masked to 0
    -> returns (num_chunks, num_chunks) mutual attention matrix
  AttentionTreeBuilder.build_tree(doc_id, chunks, embeddings, mutual_attention)
    -> agglomerative binary merge: at each step select pair (i,j) with max mutual_attention[i,j]
    -> parent embedding = (left_emb + right_emb) / 2.0
    -> parent text = left.text + '\n\n' + right.text
    -> rebuild relation matrix: new row for parent = max(left_row, right_row) per other node
    -> repeats until one root; returns DocumentIndex (2*N - 1 nodes total)
  IndexStore.save_many(indexes, metadata) -> disk

RETRIEVE (sproutrag/retrieval/hierarchical_retriever.py::HierarchicalRetriever.retrieve)
  query_embedding = encoder.encode_query(query)         -> (d,) L2-normed
  retrieve_from_embedding(query_embedding, index, top_k, beam_width=5, threshold=0.0)
    beam = [root_node]
    while beam:
      candidates = all children of beam nodes
      score each candidate: cosine_similarity(query_embedding, node.embedding)
      collect nodes with score >= threshold (or all if strategy='visited')
      beam = top beam_width candidates by score (descending)
    if nothing collected: fallback to best visited node
    return sort_retrieval_results(collected, top_k)
  optional: reranker.rerank(query, results, top_k)

MULTI-DOCUMENT RETRIEVE (sproutrag/retrieval/multi_document_retriever.py::MultiDocumentRetriever.retrieve)
  for each document index: HierarchicalRetriever.retrieve_from_embedding(query_embedding, index, per_document_top_k)
  deduplicate_retrieval_results(all_results)
  sort by score, return top_k global


PUBLIC API (from CODE)

SproutRAGIndexer(encoder: Any, aggregator: Any|None = None, tree_builder: Any|None = None, max_sentences_per_chunk: int = 2)
SproutRAGIndexer.index_document(document: RawDocument) -> DocumentIndex
SproutRAGIndexer.index_documents(documents: list[RawDocument]) -> list[DocumentIndex]
SproutRAGIndexer.index_and_save_documents(documents, store: IndexStore, metadata=None) -> manifest

SLLMEncoder(model_name_or_path: str, device=None, max_length: int = 4096, normalize_embeddings: bool = True, trust_remote_code: bool = True)
SLLMEncoder.encode_query(query: str) -> Tensor  -- (d,) L2-normed CPU
SLLMEncoder.encode_document(chunks: list[SentenceChunk], return_attentions: bool = True) -> EncodedDocument
  returns: embeddings (num_chunks, d), attentions (num_layers, num_heads, num_chunks, num_chunks)

AttentionAggregator(num_layers: int, num_heads: int, init_strategy: str = "uniform", mask_diagonal: bool = True)
  -- init_strategy: "uniform" | "normal" | "last_layer"
AttentionAggregator.forward(attentions: Tensor) -> Tensor  -- (num_chunks, num_chunks)
AttentionAggregator.get_weights() -> Tensor  -- (num_layers, num_heads) softmax-normalized
AttentionAggregator.alpha: nn.Parameter  -- (num_layers, num_heads), learned

AttentionTreeBuilder(linkage: str = "max", preserve_input_order_text: bool = True)
AttentionTreeBuilder.build_tree(doc_id, chunks, embeddings: Tensor, mutual_attention: Tensor) -> DocumentIndex

HierarchicalRetriever(encoder: Any|None = None, collect_strategy: str = "threshold", include_root: bool = False, reranker: Any|None = None)
HierarchicalRetriever.retrieve(query: str, index: DocumentIndex, top_k: int = 5, beam_width: int = 5, threshold: float = 0.0) -> list[RetrievalResult]
HierarchicalRetriever.retrieve_from_embedding(query_embedding: Tensor, index, top_k, beam_width, threshold) -> list[RetrievalResult]

MultiDocumentRetriever(encoder=None, single_document_retriever=None, collect_strategy="threshold", include_root=False, reranker=None)
MultiDocumentRetriever.retrieve(query: str, indexes: list[DocumentIndex], top_k=10, per_document_top_k=5, beam_width=5, threshold=0.0) -> list[RetrievalResult]
MultiDocumentRetriever.retrieve_from_store(query, store: IndexStore, ...) -> list[RetrievalResult]

CrossEncoderReranker(model_name_or_path: str, device=None, max_length: int = 512, batch_size: int = 8, trust_remote_code: bool = True)
CrossEncoderReranker.rerank(query: str, candidates: list[RetrievalResult], top_k: int|None = None) -> list[RetrievalResult]
CrossEncoderReranker.score_pairs(query: str, texts: list[str]) -> list[float]

DocumentIndex(doc_id, chunks: dict, nodes: dict, root_id: str|None, metadata: dict)
  .add_chunk(chunk: SentenceChunk), .add_node(node: TreeNode)
  .get_leaf_nodes() -> list[TreeNode], .get_internal_nodes() -> list[TreeNode]

TreeNode(node_id, doc_id, text, embedding: list[float]|None, left, right, parent, children, sentence_chunk_ids, depth, is_leaf, metadata)
  -- non-leaf: exactly 2 children, left and right set
  -- leaf: no children

RawDocument(doc_id: str, text: str, title: str|None = None, metadata: dict = {})
SentenceChunk(chunk_id, doc_id, text, start_char, end_char, sentence_ids, metadata)
RetrievalResult(node_id, doc_id, text, score, depth, is_leaf, sentence_chunk_ids, metadata)


TRAINING RECIPE

Loss: JointSproutRAGLoss = ContrastiveRetrievalLoss + attention_lambda * AttentionStructureLoss
  Total: L = L_retrieval + lambda * L_attention
  L_retrieval: InfoNCE (cross-entropy over cosine similarity logits / temperature)
    standard form: labels = torch.arange(batch_size); logits = cosine_matrix / temperature
    temperature default: 0.05
  L_attention: -log(mutual_attention[i,j]) for support_pairs (i,j) that are semantically related
    attention_lambda: 0.1 (default)
    support_pairs: externally provided (i,j) indices of chunks known to be related in training data

Trainable: encoder parameters + AttentionAggregator.alpha
  encoder learning rate: 2e-5
  aggregator learning rate: 1e-3 (10x higher; aggregator is tiny, faster convergence)
  weight_decay: 0.0, warmup_ratio: 0.05, gradient_clip_norm: 1.0
  num_epochs: 3, batch_size: 32
  Training data: MS MARCO v2.1 (first 30K examples loaded, per README note)

Encoder: any AutoModel implementing encode_query + encode_document
  Default: sentence-transformers/all-MiniLM-L6-v2 (384-dim)
  Requirement: must be a FAST tokenizer (offset_mapping needed for span extraction)
  Requirement: must return output_attentions (standard HuggingFace transformer)

Aggregator pre-training init options (sproutrag/aggregation/attention_aggregator.py:64-70):
  "uniform": alpha = zeros -> softmax -> uniform weights across all layers/heads
  "normal": alpha ~ N(0, 0.02)
  "last_layer": alpha[-1, :] = 5.0, rest = -5.0 -> weights concentrate on last layer


VERBATIM EXTRACTS (MANDATORY)

ATTENTION AGGREGATION (aggregation/attention_aggregator.py:77-103):
  "def aggregate(self, attentions):
       weights = self.get_weights().to(device=attentions.device, dtype=attentions.dtype)
       aggregated = torch.einsum('lh,lhij->ij', weights, attentions)
       return aggregated
   def symmetrize(self, aggregated_attention):
       mutual = (aggregated_attention + aggregated_attention.transpose(0, 1)) / 2.0
       if self.mask_diagonal:
           diag_mask = torch.eye(mutual.shape[0], ...)
           mutual = mutual * (1.0 - diag_mask)
       return mutual"

UNIFORM AGGREGATION (aggregation/attention_aggregator.py:27-38):
  "aggregated = attentions.mean(dim=(0, 1))
   mutual = (aggregated + aggregated.transpose(0, 1)) / 2.0
   if mask_diagonal:
       diag_mask = torch.eye(mutual.shape[0], ...)
       mutual = mutual * (1.0 - diag_mask)"

TREE BUILD MERGE (tree/builder.py:188-189):
  "parent_embedding = (left_emb + right_emb) / 2.0"

RELATION MATRIX UPDATE (tree/builder.py:84-90):
  "if node_i == parent_id and node_j != parent_id:
       old_j = index_before[node_j]
       value = torch.maximum(
           active_relation_before[left_idx, old_j],
           active_relation_before[right_idx, old_j],
       )
       new_relation[i, j] = value"  -- max-linkage agglomerative

NODE SCORE (retrieval/scoring.py:35-55):
  "denom = (torch.norm(query_embedding) * torch.norm(node_embedding)).clamp_min(eps)
   score = torch.sum(query_embedding * node_embedding) / denom
   return float(score.detach().cpu())"

ATTENTION STRUCTURE LOSS (training/losses.py:225-261):
  "mutual = (aggregated_attention + aggregated_attention.transpose(0, 1)) / 2.0
   if clamp:
       mutual = mutual.clamp(min=float(eps), max=1.0)
   values = mutual[indices[:, 0], indices[:, 1]]
   pair_losses = -torch.log(values)"

JOINT LOSS (training/losses.py:484):
  "loss = retrieval_loss + self.attention_lambda * attention_loss"

SPAN-MEAN-POOL (encoding/sllm_encoder.py:166):
  "chunk_embeddings = mean_pool_tokens_by_span(hidden_states, token_spans)"
  -- pool hidden states over token spans corresponding to each SentenceChunk
  -- uses offset_mapping to map char spans to token spans; skips truncated chunks

AGGREGATOR WEIGHTS (aggregation/attention_aggregator.py:72-75):
  "flat = self.alpha.reshape(-1)
   weights = torch.softmax(flat, dim=0).reshape(self.num_layers, self.num_heads)"


HYPERPARAMETERS

max_sentences_per_chunk     | 2       | 1-10    | controls chunk granularity; 2 sentences = finest practical grain
beam_width                  | 5       | 1-50    | candidates expanded per beam step; higher = broader coverage, slower
threshold                   | 0.0     | 0.0-1.0 | cosine similarity cutoff for collection; 0.0 = collect all visited
top_k                       | 10      | 1-100   | final results returned
per_document_top_k          | 5       | 1-top_k | per-document candidates before global sort
init_strategy               | uniform | uniform/normal/last_layer | last_layer biases toward final transformer layers
attention_lambda            | 0.1     | >=0     | weight of attention structure loss; 0 = disable
retrieval_temperature       | 0.05    | 0.01-1.0 | InfoNCE temperature; very sensitive, low = harder negatives
encoder_lr                  | 2e-5    | 1e-6..1e-4 | encoder fine-tuning rate
aggregator_lr               | 1e-3    | 1e-4..1e-2 | aggregator trains ~50x faster than encoder
collect_strategy            | threshold | threshold/visited | threshold=prune by score; visited=collect everything traversed


REIMPLEMENTATION NOTES FOR ELAI

ELAI retrieval stack: flat chunk index, dual ONNX embedders, BM25+embedding+PageRank, RRF, cross-encoder reranker. No hierarchical structure.

KEY QUESTION ANSWER: SproutRAG's hierarchical index meaningfully improves on ELAI's flat chunk index for code. The specific win is: internal tree nodes naturally represent code spans at multiple granularities (function body + its parent class + its module). This is structurally better than fixed-size flat chunking for code where semantic units nest (function < class < module < file).

LOAD-BEARING COMPONENTS:
  A. AttentionAggregator: tiny (num_layers * num_heads learnable scalars). At inference, attention extraction is optional -- the aggregator only needed at index BUILD time. ONNX-exportable.
  B. AttentionTreeBuilder: pure Python + torch, no NN params. Portable directly to Rust as a deterministic algorithm (agglomerative binary merge with max-linkage, O(N^2) per document, N=chunks per doc).
  C. HierarchicalRetriever: pure Python beam search with cosine similarity scoring. Easily re-implemented in Rust as graph traversal over DocumentIndex. The DocumentIndex is a simple dict of TreeNode structs.
  D. DocumentIndex serialization: JSON-serializable (sproutrag/indexing/store.py); port to serde-json in Rust.

WHAT DOES NOT MAP DIRECTLY:
  - Attention extraction requires model.forward(output_attentions=True) -- ELAI uses ONNX models that do not currently return attention weights. This is the biggest gap. To use the attention-based tree build, ELAI would need to either: (a) export ONNX with attentions output, or (b) fall back to uniform_attention_aggregation (no learned weights, just mean across layers/heads -- already in aggregation/attention_aggregator.py:26-38, no parameters needed).
  - Training (fine-tuning encoder + aggregator) requires PyTorch. For ELAI: the aggregator can start from uniform weights (functional at index-build time without training).
  - SentenceChunk splitting uses spaCy (optional) or simple heuristics. ELAI can replace with its existing token-window chunker.
  - The cross-encoder reranker is already present in ELAI's stack (ms-marco, ONNX). SproutRAG's CrossEncoderReranker is a clean reference implementation compatible with ELAI's pattern.

FALLBACK PATH (no GPU training, no attention output):
  Use uniform_attention_aggregation (mean across all layers/heads, no learned params) -> builds a tree from cosine-similarity-based mutual attention -> a reasonable proxy that avoids the need to train the aggregator or extract real attention weights.
  Alternative: replace mutual attention matrix entirely with pairwise cosine similarity between chunk embeddings. This makes the tree build a standard agglomerative embedding-similarity clustering -- structurally identical, no attention weights needed.


CRITICAL INVARIANTS (testable assertions)

assert mutual_attention.shape == (N, N), "mutual attention must be square N x N"
assert (mutual_attention == mutual_attention.T).all(), "mutual attention must be symmetric"
assert mutual_attention.diagonal().eq(0).all() if mask_diagonal, "diagonal must be zero when masked"
assert len(index.nodes) == 2 * len(chunks) - 1 if len(chunks) > 1 else 1, "binary tree has 2N-1 nodes for N leaves"
assert index.root_id in index.nodes, "root must exist in nodes dict"
assert all(n.is_leaf == (len(n.children) == 0) for n in index.nodes.values()), "leaf status must match children count"
assert score_node(q_emb, node) == cosine_similarity(q_emb, node_emb), "score_node is pure cosine similarity"


COMPLEXITY

Index build per document: O(N^2) for pair selection at each merge step (N = chunks per doc); O(N^2) relation matrix rebuild; total O(N^3) merges. For typical doc N=20-50 chunks this is fast. For N=200 it gets slow.
Encoder pass at index time: O(seq_len^2) attention + O(seq_len * d) -- single pass per doc, returns all chunk embeddings + attention tensor simultaneously.
Retrieval per document: O(N * d * beam_width) cosine similarity per level; O(log N) tree levels -> O(N * d * beam_width * log N) total.
Multi-document retrieval: linear in number of documents; each document searched independently.
Memory: DocumentIndex stores 2N-1 TreeNode objects, each with embedding: list[float] (d floats) + text. At d=384, N=50: ~40 nodes * (384*4 + text_bytes) ~ few hundred KB per document.


FAILURE MODES / PITFALLS

1. offset_mapping requires FAST tokenizer: SLLMEncoder uses offset_mapping to locate chunk boundaries. Slow tokenizers (return_offsets_mapping not supported) will raise ValueError. All modern HuggingFace tokenizers are fast; verify before swapping encoder.
2. max_length truncation silently skips chunks: chunks whose tokens lie beyond max_length are silently dropped (skipped_chunk_ids). For long documents with max_sentences_per_chunk=2 this can silently remove the tail of the document from the index. Check skipped_chunk_ids in index.metadata.
3. Tree build is O(N^3): for documents with many chunks (N > 100) the agglomerative build becomes slow due to pair exhaustive search + relation matrix rebuild. Not suitable for whole-file indexing of large files without chunking first.
4. Leaf-only retrieval common at threshold=0.0: with beam_width=5 the beam quickly converges on a subtree branch; most internal (broader) nodes never get visited if fine-grained chunks score well. If callers want multi-granularity, use collect_strategy='visited' or tune threshold down.
5. Uniform aggregation = no learning needed: uniform_attention_aggregation (attention_aggregator.py:26) requires no trained parameters and is fully functional. The learned AttentionAggregator only helps if trained on in-domain data with known support pairs (semantically related chunk indices). Without that, uniform is the correct baseline.
6. Internal node text is concatenation with '\n\n': callers must handle that internal node text is longer than leaf text and may contain redundant content when the retriever returns both a node and its descendant.
7. Cosine similarity scoring only at retrieve time: the tree structure reflects mutual attention at INDEX time; at QUERY time only cosine similarity to the query embedding is used. If query space and document space are not well-aligned (different model for query vs document), beam search may explore incorrect subtrees.
8. No cross-encoder reranker in ONNX: CrossEncoderReranker uses PyTorch AutoModelForSequenceClassification. ELAI already has an ONNX reranker (ms-marco cross-encoder) which is faster on CPU -- use that instead of SproutRAG's reranker.
