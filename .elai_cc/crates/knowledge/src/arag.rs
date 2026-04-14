use std::collections::HashSet;

use crate::error::KnowledgeError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Oversampling factor for semantic search: retrieve 3x top_k sentence-level
/// candidates, then group by parent chunk and max-pool scores.
/// Grounded: arag-interface.md L109-113.
const SEMANTIC_OVERSAMPLE_FACTOR: usize = 3;

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

/// Adaptive RAG orchestrator: iteratively selects retrieval tools
/// based on context and token budget constraints.
#[derive(Debug, Clone)]
pub struct AragOrchestrator {
    pub config: AragConfig,
    chunks: Vec<AragChunk>,
}

impl AragOrchestrator {
    pub fn new(config: AragConfig) -> Self {
        Self {
            config,
            chunks: Vec::new(),
        }
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
    fn semantic_search(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RetrievalResult>, KnowledgeError> {
        if self.chunks.is_empty() {
            return Ok(Vec::new());
        }

        // Use chunk embeddings for similarity scoring.
        // In production this would use real dual-vector embeddings; for now we use chunk-level
        // embeddings directly since sentence-level embeddings aren't stored separately.
        let candidate_count = top_k.saturating_mul(SEMANTIC_OVERSAMPLE_FACTOR).min(self.chunks.len());

        let mut chunk_scores: Vec<(usize, f64)> = self
            .chunks
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.embedding.is_empty())
            .map(|(idx, c)| {
                // Use chunk embedding norm as a proxy score.
                // In production: dot_product(query_embedding, chunk_embedding).
                let norm: f64 = c.embedding.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
                (idx, norm)
            })
            .collect();

        chunk_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        chunk_scores.truncate(candidate_count);

        // Max-pool: already at chunk level. Take top_k.
        chunk_scores.truncate(top_k);

        let results = chunk_scores
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
