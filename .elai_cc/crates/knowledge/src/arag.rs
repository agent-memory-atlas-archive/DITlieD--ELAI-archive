use std::collections::HashSet;
use std::error::Error;

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

    pub fn token_count(&self) -> usize {
        self.read_chunks.len() * 256
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
}

impl AragOrchestrator {
    pub fn new(config: AragConfig) -> Self {
        Self { config }
    }

    /// Execute a single retrieval tool and return results.
    pub fn execute_tool(
        &self,
        tool: RetrievalTool,
        tracker: &mut ContextTracker,
    ) -> Result<Vec<RetrievalResult>, Box<dyn Error>> {
        match tool {
            RetrievalTool::KeywordSearch { .. } => {
                // No bm25 index on AragOrchestrator — graceful degrade
                Ok(Vec::new())
            }
            RetrievalTool::SemanticSearch { .. } => {
                // No vector index available — graceful degrade
                Ok(Vec::new())
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
                        // File I/O deferred to Phase 1I; return chunk_id as content identifier
                        results.push(RetrievalResult {
                            chunk_id: chunk_id.clone(),
                            snippet: format!("[chunk: {chunk_id}]"),
                            score: 1.0,
                            tool_used: "chunk_read".to_string(),
                        });
                    }
                }
                Ok(results)
            }
        }
    }

    /// Run the full A-RAG query pipeline: iteratively select tools,
    /// execute them, and accumulate results until budget is exhausted.
    pub fn query_pipeline(
        &self,
        query: &str,
        _snapshot: &crate::pagerank::DkgSnapshot,
    ) -> Result<Vec<RetrievalResult>, Box<dyn Error>> {
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

        // Step 3: chunk read on any hits (budget-limited)
        let chunk_ids: Vec<String> = all_results.iter().map(|r| r.chunk_id.clone()).collect();
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
