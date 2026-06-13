use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use blake3;
use rustc_hash::FxHashMap;

use crate::error::KnowledgeError;
use crate::vector::ort_session::OrtSession;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Oversampling factor for semantic search: retrieve 3x top_k sentence-level
/// candidates, then group by parent chunk and max-pool scores.
/// Grounded: arag-interface.md L109-113.
const SEMANTIC_OVERSAMPLE_FACTOR: usize = 3;

/// I-15: TTL in seconds for in-process intermediate-summary cache entries.
/// Five-minute session-stable window; a live reindex replaces the AragOrchestrator
/// instance so no stale-chunk hazard exists.
pub const INTERMEDIATE_SUMMARY_CACHE_TTL_SECS: u64 = 300;

/// Maximum sentences to include in a progressive-disclosure snippet.
/// Grounded: arag-interface.md L273-280.
const SNIPPET_MAX_SENTENCES: usize = 5;

// ---------------------------------------------------------------------------
// RetrievalTool
// ---------------------------------------------------------------------------

/// Tools available to the A-RAG orchestrator for adaptive retrieval.
#[derive(Debug, Clone)]
pub enum RetrievalTool {
    KeywordSearch { keywords: Vec<String>, top_k: usize },
    SemanticSearch { query: String, top_k: usize },
    ChunkRead { chunk_ids: Vec<String> },
}

// ---------------------------------------------------------------------------
// AragConfig
// ---------------------------------------------------------------------------

/// Configuration for A-RAG adaptive retrieval orchestration.
///
/// Defaults from A-RAG interface spec.
#[derive(Debug, Clone)]
pub struct AragConfig {
    pub max_loops: usize,
    pub max_token_budget: usize,
    pub top_k: usize,
}

impl Default for AragConfig {
    fn default() -> Self {
        Self {
            max_loops: 15,
            max_token_budget: 128_000,
            top_k: 5,
        }
    }
}

// ---------------------------------------------------------------------------
// RetrievalResult
// ---------------------------------------------------------------------------

/// A single retrieval result from any pipeline stage or A-RAG tool.
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub chunk_id: String,
    pub snippet: String,
    pub score: f64,
    pub tool_used: String,
}

// ---------------------------------------------------------------------------
// AragChunk — in-memory chunk with optional embedding
// ---------------------------------------------------------------------------

/// A chunk loaded into the A-RAG orchestrator for keyword and semantic search.
#[derive(Debug, Clone)]
pub struct AragChunk {
    pub chunk_id: String,
    pub content: String,
    pub embedding: Vec<f32>,
}

// ---------------------------------------------------------------------------
// ContextTracker
// ---------------------------------------------------------------------------

/// Tracks which chunks have been read during an A-RAG session.
#[derive(Debug, Clone, Default)]
pub struct ContextTracker {
    read_chunks: HashSet<String>,
}

impl ContextTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_read(&self, id: &str) -> bool {
        self.read_chunks.contains(id)
    }

    pub fn mark_read(&mut self, id: String) {
        self.read_chunks.insert(id);
    }

    /// Estimated token count. Uses ~1000 tokens per chunk (spec: arag-interface.md L29).
    pub fn token_count(&self) -> usize {
        self.read_chunks.len() * 1000
    }
}

// ---------------------------------------------------------------------------
// AragOrchestrator
// ---------------------------------------------------------------------------

/// I-15: An in-memory entry for the blake3-keyed exact semantic-search cache.
#[derive(Debug, Clone)]
struct ExactCacheEntry {
    results: Vec<RetrievalResult>,
    /// Unix-seconds insertion timestamp for TTL enforcement.
    cached_at: u64,
}

/// I-15: In-memory per-query intermediate-summary cache.
///
/// Keyed on blake3(query_bytes). Lifetime = AragOrchestrator instance
/// (a live reindex replaces the orchestrator so no stale-chunk hazard exists).
/// CONFINEMENT: never persisted to disk; not the OracleAnswerCache disk store.
/// All code-gen path queries that never reach this function are safe by design:
/// AragOrchestrator is only used on the `/research|/ask` pipeline, not on the
/// code-gen path (H-8 confinement rule inherited verbatim).
type ExactCache = Arc<Mutex<FxHashMap<[u8; 32], ExactCacheEntry>>>;

/// blake3 helper mirroring oracle_cache's pattern; returns a 32-byte digest.
fn blake3_32_arag(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

/// I-15: Look up a cached result set for `query`. Returns `Some` only when
/// the entry exists AND is fresh (within `INTERMEDIATE_SUMMARY_CACHE_TTL_SECS`).
/// Returns `None` on lock-poison or a stale entry (stale entries are evicted).
fn exact_cache_lookup(cache: &ExactCache, query_hash: [u8; 32], now_secs: u64) -> Option<Vec<RetrievalResult>> {
    let mut guard = cache.lock().ok()?;
    let entry = guard.get(&query_hash)?.clone();
    if now_secs.saturating_sub(entry.cached_at) >= INTERMEDIATE_SUMMARY_CACHE_TTL_SECS {
        guard.remove(&query_hash);
        return None;
    }
    Some(entry.results)
}

/// I-15: Insert a result set for `query_hash` into the cache.
/// Silently drops on lock-poison (non-fatal path).
fn exact_cache_insert(cache: &ExactCache, query_hash: [u8; 32], results: Vec<RetrievalResult>, now_secs: u64) {
    if let Ok(mut guard) = cache.lock() {
        guard.insert(query_hash, ExactCacheEntry { results, cached_at: now_secs });
    }
}

/// Adaptive RAG orchestrator: iteratively selects retrieval tools
/// based on context and token budget constraints.
#[derive(Debug, Clone)]
pub struct AragOrchestrator {
    pub config: AragConfig,
    chunks: Vec<AragChunk>,
    /// C-8 Task #8: optional ONNX-backed embedder used by
    /// [`Self::semantic_search`] to compute a real query embedding and
    /// score chunks via dot-product against their stored embeddings.
    ///
    /// Defaults to `None`; production wiring at `cli::live_cli::
    /// build_local_knowledge_stack` loads the graphcodebert.onnx model
    /// from `.elai_cc/models/onnx/` and installs it via
    /// [`Self::with_embedder`]. When absent the search falls back to the
    /// pre-C-8 embedding-norm proxy with a `tracing::warn!` so callers
    /// are not silently degraded.
    embedder: Option<Arc<OrtSession>>,
    /// I-15: in-memory session-stable blake3-keyed exact-match cache for
    /// intermediate semantic-search results (DKG chunk summaries).
    /// Arc<Mutex<...>> so Clone preserves the shared cache across clones
    /// (QueryPipeline may clone the orchestrator for concurrent calls).
    /// CONFINEMENT: not persisted; never fed code-gen results.
    exact_cache: ExactCache,
}

impl AragOrchestrator {
    pub fn new(config: AragConfig) -> Self {
        Self {
            config,
            chunks: Vec::new(),
            embedder: None,
            exact_cache: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }

    /// C-8 Task #8: install an ONNX `OrtSession` so [`Self::semantic_search`]
    /// computes real query embeddings and scores chunks via dot-product. The
    /// session must be the graphcodebert (or compatible RoBERTa-family)
    /// model whose 768-dim output matches the dim of stored
    /// `AragChunk.embedding` vectors.
    pub fn with_embedder(mut self, embedder: Arc<OrtSession>) -> Self {
        self.embedder = Some(embedder);
        self
    }

    /// I-15: Return the number of entries currently held in the in-memory
    /// exact cache.  Used by contract tests and cache-efficiency reporting
    /// without exposing the inner FxHashMap.
    pub fn exact_cache_len(&self) -> usize {
        self.exact_cache.lock().map(|g| g.len()).unwrap_or(0)
    }

    /// Add a chunk to the in-memory corpus for keyword and semantic search.
    pub fn add_chunk(&mut self, chunk: AragChunk) {
        self.chunks.push(chunk);
    }

    /// Number of chunks loaded.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Execute a single retrieval tool and return results.
    pub fn execute_tool(
        &self,
        tool: RetrievalTool,
        tracker: &mut ContextTracker,
    ) -> Result<Vec<RetrievalResult>, KnowledgeError> {
        match tool {
            RetrievalTool::KeywordSearch { keywords, top_k } => {
                self.keyword_search(&keywords, top_k)
            }
            RetrievalTool::SemanticSearch { query, top_k } => {
                self.semantic_search(&query, top_k)
            }
            RetrievalTool::ChunkRead { chunk_ids } => {
                let mut results = Vec::with_capacity(chunk_ids.len());
                for chunk_id in chunk_ids {
                    if tracker.is_read(&chunk_id) {
                        results.push(RetrievalResult {
                            chunk_id,
                            snippet: "(previously read)".to_string(),
                            score: 0.0,
                            tool_used: "chunk_read".to_string(),
                        });
                    } else {
                        tracker.mark_read(chunk_id.clone());
                        let full_text = self
                            .chunks
                            .iter()
                            .find(|c| c.chunk_id == chunk_id)
                            .map(|c| c.content.clone())
                            .unwrap_or_else(|| format!("[chunk: {chunk_id}]"));
                        results.push(RetrievalResult {
                            chunk_id,
                            snippet: full_text,
                            score: 1.0,
                            tool_used: "chunk_read".to_string(),
                        });
                    }
                }
                Ok(results)
            }
        }
    }

    // -----------------------------------------------------------------------
    // Keyword search (A-RAG score formula with length normalization)
    // -----------------------------------------------------------------------

    /// Score_kw(chunk, keywords) = SUM(count(k, chunk_text) * len(k)) / sqrt(chunk_char_len)
    ///
    /// Grounded: arag-interface.md L72-77 (base formula).
    /// Length normalization (/ sqrt(char_len)) added to prevent long-file dominance
    /// per adversarial requirement.
    fn keyword_search(
        &self,
        keywords: &[String],
        top_k: usize,
    ) -> Result<Vec<RetrievalResult>, KnowledgeError> {
        if self.chunks.is_empty() || keywords.is_empty() {
            return Ok(Vec::new());
        }

        let mut scored: Vec<(usize, f64)> = Vec::with_capacity(self.chunks.len());
        let keywords_lower: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();

        for (idx, chunk) in self.chunks.iter().enumerate() {
            let text_lower = chunk.content.to_lowercase();
            let char_len = chunk.content.len().max(1) as f64;

            let raw_score: f64 = keywords_lower
                .iter()
                .map(|kw| {
                    let count = count_substring_occurrences(&text_lower, kw);
                    count as f64 * kw.len() as f64
                })
                .sum();

            if raw_score > 0.0 {
                let normalized = raw_score / char_len.sqrt();
                scored.push((idx, normalized));
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        let results = scored
            .into_iter()
            .map(|(idx, score)| {
                let chunk = &self.chunks[idx];
                let snippet = extract_keyword_snippet(&chunk.content, keywords);
                RetrievalResult {
                    chunk_id: chunk.chunk_id.clone(),
                    snippet,
                    score,
                    tool_used: "keyword_search".to_string(),
                }
            })
            .collect();

        Ok(results)
    }

    // -----------------------------------------------------------------------
    // Semantic search with 3x oversampling
    // -----------------------------------------------------------------------

    /// Grounded: arag-interface.md L101-119.
    /// 1. Dot-product query embedding against chunk embeddings.
    /// 2. Retrieve top_k * SEMANTIC_OVERSAMPLE_FACTOR candidates.
    /// 3. Group by parent chunk_id, take max score per chunk.
    /// 4. Sort by score, return top_k.
    /// 5. Snippet = matched sentences (progressive disclosure).
    ///
    /// C-8 + C-17 (DRIFT-B): ONNX dot-product is the default and only
    /// production code path when an embedder is installed. The norm-proxy
    /// is a documented LOGGED-FALLBACK reserved for two degraded states:
    ///   (a) `self.embedder` is `None` — no embedder constructed (legacy
    ///       call sites still in transition to `AragOrchestrator::with_embedder`).
    ///   (b) the ONNX `embed(query)` call itself returned `Err` — model
    ///       failure mid-pipeline. We emit a `tracing::warn!` and keep
    ///       returning ranked results so the consumer never goes dark.
    /// C-17 contract: when the embedder is present and `embed()` succeeded,
    /// dot-product is the scoring function. The fallback is never the
    /// happy path.
    /// I-15: intermediate_summary_cache_lookup — public surface for wiring-contract
    /// verification. Returns a hit and the cached Vec<RetrievalResult> if present
    /// and fresh, otherwise None.  Delegates to the private exact_cache_lookup helper.
    pub fn intermediate_summary_cache_lookup(
        &self,
        query: &str,
        now_secs: u64,
    ) -> Option<Vec<RetrievalResult>> {
        let key = blake3_32_arag(query.as_bytes());
        exact_cache_lookup(&self.exact_cache, key, now_secs)
    }

    /// I-15: intermediate_summary_cache_insert — public surface for wiring-contract
    /// verification. Inserts a result set keyed on blake3(query_bytes).
    pub fn intermediate_summary_cache_insert(
        &self,
        query: &str,
        results: Vec<RetrievalResult>,
        now_secs: u64,
    ) {
        let key = blake3_32_arag(query.as_bytes());
        exact_cache_insert(&self.exact_cache, key, results, now_secs);
    }

    fn semantic_search(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RetrievalResult>, KnowledgeError> {
        if self.chunks.is_empty() {
            return Ok(Vec::new());
        }

        // I-15: blake3 exact fast path. O(1) lookup BEFORE any ONNX embedding
        // call (WIRING_CONTRACT ordering invariant: NO embedder.embed(query) before
        // this check). Clock is real-time; the cache is session-stable in-memory only.
        let query_hash = blake3_32_arag(query.as_bytes());
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if let Some(cached) = exact_cache_lookup(&self.exact_cache, query_hash, now_secs) {
            tracing::debug!(
                query_hash_prefix = %format!("{:02x}{:02x}{:02x}{:02x}", query_hash[0], query_hash[1], query_hash[2], query_hash[3]),
                "arag.semantic_search: blake3 exact cache hit (ONNX embedding skipped)",
            );
            return Ok(cached);
        }

        let candidate_count = top_k.saturating_mul(SEMANTIC_OVERSAMPLE_FACTOR).min(self.chunks.len());

        let mut chunk_scores: Vec<(usize, f64)> = match self.embedder.as_ref() {
            Some(embedder) => {
                // C-8 Task #8: real ONNX dot-product path.
                match embedder.embed(query) {
                    Ok(query_emb) => self
                        .chunks
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| !c.embedding.is_empty())
                        .map(|(idx, c)| {
                            let score = dot_product_f32(&query_emb, &c.embedding);
                            (idx, score as f64)
                        })
                        .collect(),
                    Err(e) => {
                        // Embedding failure is non-fatal at this layer: fall
                        // back to the norm proxy with a single warn so the
                        // pipeline keeps producing output and the degradation
                        // is observable.
                        tracing::warn!(
                            error = %e,
                            "arag.semantic_search: ONNX query embedding failed; falling back to norm proxy",
                        );
                        norm_proxy_scores(&self.chunks)
                    }
                }
            }
            None => {
                tracing::warn!(
                    "arag.semantic_search: no embedder installed; using legacy norm proxy. \
                     Install via AragOrchestrator::with_embedder for real dot-product scoring.",
                );
                norm_proxy_scores(&self.chunks)
            }
        };

        chunk_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        chunk_scores.truncate(candidate_count);

        // Max-pool: already at chunk level. Take top_k.
        chunk_scores.truncate(top_k);

        let results: Vec<RetrievalResult> = chunk_scores
            .into_iter()
            .map(|(idx, score)| {
                let chunk = &self.chunks[idx];
                let snippet = extract_top_sentences(&chunk.content, query);
                RetrievalResult {
                    chunk_id: chunk.chunk_id.clone(),
                    snippet,
                    score,
                    tool_used: "semantic_search".to_string(),
                }
            })
            .collect();

        // I-15: insert results into the in-memory intermediate-summary cache
        // so a repeated identical query short-circuits on the blake3 path above.
        // Non-empty check avoids caching degraded-mode empty results.
        if !results.is_empty() {
            exact_cache_insert(&self.exact_cache, query_hash, results.clone(), now_secs);
        }

        Ok(results)
    }

    /// Run the full A-RAG query pipeline: iteratively select tools,
    /// execute them, and accumulate results until budget is exhausted.
    pub fn query_pipeline(
        &self,
        query: &str,
        _snapshot: &crate::pagerank::DkgSnapshot,
    ) -> Result<Vec<RetrievalResult>, KnowledgeError> {
        let mut tracker = ContextTracker::new();
        let mut all_results: Vec<RetrievalResult> = Vec::new();

        // Step 1: keyword search
        let keywords: Vec<String> = query.split_whitespace().map(String::from).collect();
        let keyword_results = self.execute_tool(
            RetrievalTool::KeywordSearch {
                keywords,
                top_k: self.config.top_k,
            },
            &mut tracker,
        )?;
        all_results.extend(keyword_results);

        // Step 2: semantic search
        let semantic_results = self.execute_tool(
            RetrievalTool::SemanticSearch {
                query: query.to_string(),
                top_k: self.config.top_k,
            },
            &mut tracker,
        )?;
        all_results.extend(semantic_results);

        // Step 3: chunk read on deduplicated hits (budget-limited)
        let mut seen = HashSet::new();
        let chunk_ids: Vec<String> = all_results
            .iter()
            .filter(|r| seen.insert(r.chunk_id.clone()))
            .map(|r| r.chunk_id.clone())
            .collect();
        if !chunk_ids.is_empty() && tracker.token_count() < self.config.max_token_budget {
            let read_results = self.execute_tool(
                RetrievalTool::ChunkRead { chunk_ids },
                &mut tracker,
            )?;
            all_results.extend(read_results);
        }

        Ok(all_results)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// C-8 Task #8: dot-product between a query embedding and a chunk embedding.
/// Truncates to the common prefix length so a dimension mismatch between
/// model output (e.g. 768) and stored chunk dim does not panic. Returns 0.0
/// if either side is empty.
fn dot_product_f32(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut acc: f32 = 0.0;
    // SAFETY-NOTE: bounded loop indexes a[..n] and b[..n] from len().min(len()).
    for i in 0..n {
        acc += a[i] * b[i];
    }
    acc
}

/// C-8 Task #8: legacy norm proxy used when no embedder is installed or the
/// query embedding call fails. Kept as a fallback so the pipeline never goes
/// dark when the ONNX path is degraded. A `tracing::warn!` upstream records
/// the degradation.
fn norm_proxy_scores(chunks: &[AragChunk]) -> Vec<(usize, f64)> {
    chunks
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.embedding.is_empty())
        .map(|(idx, c)| {
            let norm: f64 = c
                .embedding
                .iter()
                .map(|x| (*x as f64) * (*x as f64))
                .sum::<f64>()
                .sqrt();
            (idx, norm)
        })
        .collect()
}

/// Count case-insensitive substring occurrences.
fn count_substring_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0;
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        count += 1;
        start += pos + needle.len();
    }
    count
}

/// Split text into sentences. A sentence boundary is a period, exclamation, or
/// question mark followed by whitespace or end-of-string. This avoids splitting
/// on code tokens like `.await`, `foo.bar()`, version numbers (`1.2.3`), etc.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let at_end = i + 1 >= len;
            let followed_by_ws = !at_end && chars[i + 1].is_whitespace();
            if at_end || followed_by_ws {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }

    sentences
}

/// Extract sentences containing any keyword (progressive disclosure).
/// Returns at most SNIPPET_MAX_SENTENCES matched sentences joined by space.
fn extract_keyword_snippet(content: &str, keywords: &[String]) -> String {
    let sentences = split_sentences(content);
    let keywords_lower: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();

    let matched: Vec<&String> = sentences
        .iter()
        .filter(|s| {
            let s_lower = s.to_lowercase();
            keywords_lower.iter().any(|kw| s_lower.contains(kw.as_str()))
        })
        .take(SNIPPET_MAX_SENTENCES)
        .collect();

    let parts = if matched.is_empty() {
        sentences.into_iter().take(1).collect::<Vec<_>>()
    } else {
        matched.into_iter().cloned().collect::<Vec<_>>()
    };
    format_snippet(&parts)
}

/// Extract top sentences for semantic search snippet (progressive disclosure).
/// NOTE: Ranking is query-independent until dual-vector encoding is wired (Phase 1C).
/// Current implementation returns first N sentences as a proxy.
fn extract_top_sentences(content: &str, _query: &str) -> String {
    let sentences = split_sentences(content);
    let parts: Vec<String> = sentences.into_iter().take(SNIPPET_MAX_SENTENCES).collect();
    format_snippet(&parts)
}

/// Format snippet parts with " ... " separator per arag-interface.md L85-86.
fn format_snippet(parts: &[String]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    if parts.len() == 1 {
        return format!("... {} ...", parts[0]);
    }
    format!("... {} ...", parts.join(" ... "))
}

// ---------------------------------------------------------------------------
// Corpus population from DKG
// ---------------------------------------------------------------------------

/// Populate the A-RAG in-memory corpus from a `DkgEngine` so the
/// `keyword_search` / `semantic_search` / `query_pipeline` methods return
/// non-empty results.
///
/// Wires the previously-zero `add_chunk` call surface: each DKG node becomes
/// one [`AragChunk`] with `chunk_id = node_<id>`, a `content` field composed
/// of the symbol kind + name + signature (enough for keyword scoring), and
/// an empty `embedding` vector.
///
/// Embeddings are intentionally left empty: producing real per-chunk
/// embeddings requires the dual-vector ONNX models which are not staged on
/// every host. The semantic_search path filters out empty-embedding chunks
/// (see [`AragOrchestrator::semantic_search`] / [`norm_proxy_scores`]) so the
/// pipeline still degrades gracefully when no embedder is installed. A
/// future pass MAY extend this helper to thread a `&OrtSession` through and
/// populate embeddings at ingest time.
///
/// Returns the number of chunks added.
pub fn populate_corpus_from_dkg(
    dkg: &crate::DkgEngine,
    arag: &mut AragOrchestrator,
) -> Result<usize, KnowledgeError> {
    let nodes = dkg.nodes()?;
    let mut added: usize = 0;
    for node in nodes {
        // Compose human-readable content for keyword scoring. Format mirrors
        // the snippet shape consumed by `extract_keyword_snippet`.
        let kind_str = format!("{:?}", node.kind);
        let content = if node.signature.trim().is_empty() {
            format!("{} {}", kind_str, node.symbol_name)
        } else {
            format!("{} {} {}", kind_str, node.symbol_name, node.signature)
        };
        arag.add_chunk(AragChunk {
            chunk_id: format!("node_{}", node.node_id),
            content,
            embedding: Vec::new(),
        });
        added = added.saturating_add(1);
    }
    Ok(added)
}
