use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use elai_core::{ClockProvider as _, OsClock};

use crate::bm25::BM25Chunk;
use crate::error::KnowledgeError;
use crate::vector::cache::VectorCache;
use crate::vector::hnsw_index::{BruteForceIndex, VectorIndex};
use crate::vector::VectorModule;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CHUNK_WINDOW_LINES: usize = 50;
const CHUNK_OVERLAP_LINES: usize = 10;
const BINARY_DETECT_BYTES: usize = 512;

// ---------------------------------------------------------------------------
// VectorIndexer — file -> chunks -> embeddings -> BruteForceIndex
// ---------------------------------------------------------------------------

pub struct VectorIndexer {
    pub index: BruteForceIndex,
    /// Maps chunk_key -> (file_path, line_start, line_end, snippet).
    /// Used to convert search results back to RetrievalResult.
    chunk_meta: Vec<ChunkMeta>,
    dim: usize,
    // B-4 SCHEMA: code_index is opt-in (code_vec_enabled=false by default).
    // Serialized to <index_path>.code_index.json adjacent to text vector_index.json.
    // Cold-start rebuild from VectorCache required when flag flips true.
    /// Secondary code-space index (GraphCodeBERT dim=768). None when code_vec_enabled=false.
    pub code_index: Option<BruteForceIndex>,
}

#[derive(Debug, Clone)]
pub struct ChunkMeta {
    pub key: u64,
    pub path: PathBuf,
    pub line_start: u32,
    pub line_end: u32,
    pub snippet: String,
}

#[derive(Debug)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub chunks_embedded: usize,
    pub chunks_cached: usize,
    pub chunks_failed: usize,
}

impl VectorIndexer {
    pub fn new(dim: usize) -> Self {
        Self {
            index: BruteForceIndex::new(dim),
            chunk_meta: Vec::new(),
            dim,
            code_index: None,
        }
    }

    /// Enable the secondary code-space index (dim=CODE_VEC_DIM).
    /// Called from main.rs when code_vec_enabled=true. Does not rebuild from
    /// cache — caller is responsible for populating via `index_file`.
    pub fn enable_code_index(&mut self) {
        use crate::vector::CODE_VEC_DIM;
        if self.code_index.is_none() {
            self.code_index = Some(BruteForceIndex::new(CODE_VEC_DIM));
        }
    }

    /// Load a persisted index from disk.
    pub fn load(path: &Path, dim: usize) -> Result<Self, KnowledgeError> {
        let index = BruteForceIndex::load(path, dim)?;
        // Meta must be loaded separately (saved alongside index).
        let meta_path = path.with_extension("meta.json");
        let meta: Vec<ChunkMeta> = if meta_path.exists() {
            let data = std::fs::read(&meta_path)?;
            serde_json::from_slice(&data)?
        } else {
            Vec::new()
        };
        // B-4: attempt to load adjacent code index if present.
        // Path: <stem>.code_index.json (e.g. vector_index.code_index.json).
        // If absent, code_index stays None — existing serialized files unaffected.
        let code_index_path = code_index_path_for(path);
        let code_index = if code_index_path.exists() {
            use crate::vector::CODE_VEC_DIM;
            match BruteForceIndex::load(&code_index_path, CODE_VEC_DIM) {
                Ok(idx) => Some(idx),
                Err(e) => {
                    tracing::warn!(path = %code_index_path.display(), error = %e,
                        "code_index load failed; code_vec search disabled");
                    None
                }
            }
        } else {
            None
        };
        Ok(Self { index, chunk_meta: meta, dim, code_index })
    }

    /// Persist index + metadata to disk.
    /// Also persists code_index to adjacent path when present.
    pub fn save(&self, path: &Path) -> Result<(), KnowledgeError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.index.save(path)?;
        let meta_path = path.with_extension("meta.json");
        let meta_bytes = serde_json::to_vec(&self.chunk_meta)?;
        std::fs::write(&meta_path, meta_bytes)?;
        // B-4: persist code_index if populated.
        if let Some(ref cidx) = self.code_index {
            let code_path = code_index_path_for(path);
            if let Err(e) = cidx.save(&code_path) {
                tracing::warn!(path = %code_path.display(), error = %e,
                    "code_index save failed; code_vec search will cold-start next run");
            }
        }
        Ok(())
    }

    /// Index a single file: chunk it, embed each chunk, add to index.
    /// Uses VectorCache for dedup (skip embedding if content hash matches).
    pub fn index_file(
        &mut self,
        path: &Path,
        vector_module: &dyn VectorModule,
        cache: Option<&VectorCache>,
    ) -> Result<IndexStats, KnowledgeError> {
        let chunks = chunk_file(path)?;
        let mut stats = IndexStats {
            files_indexed: 1,
            chunks_embedded: 0,
            chunks_cached: 0,
            chunks_failed: 0,
        };

        // Remove old chunks for this file before re-indexing.
        self.remove_file(path);

        for chunk in &chunks {
            let key = chunk_key(&chunk.path, chunk.line_start, chunk.line_end);
            let content_bytes = chunk.content.as_bytes();
            let content_hash = VectorCache::content_hash(content_bytes);

            // Check cache first.
            let (text_embedding, cached_code_vec) = if let Some(c) = cache {
                if let Ok(Some(dual)) = c.lookup(&content_hash) {
                    stats.chunks_cached += 1;
                    // Use the text embedding for search (more general than code-specific).
                    (Some(dual.text_vec), Some(dual.code_vec))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            let (embedding, code_embedding_opt) = match text_embedding {
                Some(e) => (e, cached_code_vec),
                None => {
                    // Embed the chunk content.
                    match vector_module.embed_text(&chunk.content) {
                        Ok(vec) => {
                            stats.chunks_embedded += 1;
                            // Compute code embedding; store dual in cache if available.
                            let code_vec_opt = match vector_module.embed_code(&chunk.content) {
                                Ok(code_vec) => {
                                    if let Some(c) = cache {
                                        if let Ok(dual) = crate::vector::DualVector::new(code_vec.clone(), vec.clone()) {
                                            let now_secs = OsClock
                                                .system_now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs();
                                            let now = i64::try_from(now_secs).unwrap_or(i64::MAX);
                                            let _ = c.insert(&content_hash, &dual, now);
                                        }
                                    }
                                    Some(code_vec)
                                }
                                Err(e) => {
                                    tracing::debug!("embed_code failed for {}:{}-{}: {e}",
                                        chunk.path.display(), chunk.line_start, chunk.line_end);
                                    None
                                }
                            };
                            (vec, code_vec_opt)
                        }
                        Err(e) => {
                            tracing::debug!("embed failed for {}:{}-{}: {e}",
                                chunk.path.display(), chunk.line_start, chunk.line_end);
                            stats.chunks_failed += 1;
                            continue;
                        }
                    }
                }
            };

            // Truncate or pad if dimension mismatch (defensive).
            let vec = if embedding.len() == self.dim {
                embedding
            } else if embedding.len() > self.dim {
                embedding[..self.dim].to_vec()
            } else {
                let mut padded = embedding;
                padded.resize(self.dim, 0.0);
                padded
            };

            if let Err(e) = self.index.add(key, &vec) {
                tracing::debug!("index add failed for key {key}: {e}");
                stats.chunks_failed += 1;
                continue;
            }

            // B-4: populate secondary code index if enabled and code embedding is available.
            if let (Some(ref mut cidx), Some(ref code_vec)) = (&mut self.code_index, &code_embedding_opt) {
                use crate::vector::CODE_VEC_DIM;
                let cvec = if code_vec.len() == CODE_VEC_DIM {
                    code_vec.clone()
                } else if code_vec.len() > CODE_VEC_DIM {
                    code_vec[..CODE_VEC_DIM].to_vec()
                } else {
                    let mut padded = code_vec.clone();
                    padded.resize(CODE_VEC_DIM, 0.0);
                    padded
                };
                if let Err(e) = cidx.add(key, &cvec) {
                    tracing::debug!("code_index add failed for key {key}: {e}");
                }
            }

            self.chunk_meta.push(ChunkMeta {
                key,
                path: chunk.path.clone(),
                line_start: chunk.line_start,
                line_end: chunk.line_end,
                snippet: truncate_snippet(&chunk.content, 300),
            });
        }

        Ok(stats)
    }

    /// Remove all chunks belonging to a file.
    pub fn remove_file(&mut self, path: &Path) {
        let keys_to_remove: Vec<u64> = self
            .chunk_meta
            .iter()
            .filter(|m| m.path == path)
            .map(|m| m.key)
            .collect();
        for key in &keys_to_remove {
            self.index.remove(*key);
            if let Some(ref mut cidx) = self.code_index {
                cidx.remove(*key);
            }
        }
        self.chunk_meta.retain(|m| m.path != path);
    }

    /// Search the text-space vector index and return chunk metadata for the top-k results.
    pub fn search(&self, query_vec: &[f32], k: usize) -> Result<Vec<(ChunkMeta, f32)>, KnowledgeError> {
        let results = self.index.search(query_vec, k)?;
        let mut out = Vec::with_capacity(results.len());
        for (key, score) in results {
            if let Some(meta) = self.chunk_meta.iter().find(|m| m.key == key) {
                out.push((meta.clone(), score));
            }
        }
        Ok(out)
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Search the code-space vector index (opt-in; returns empty if code_index absent).
    /// Returns empty rather than error when code_index is None (graceful degradation).
    pub fn search_code(&self, query_vec: &[f32], k: usize) -> Result<Vec<(ChunkMeta, f32)>, KnowledgeError> {
        let cidx = match &self.code_index {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };
        let results = cidx.search(query_vec, k)?;
        let mut out = Vec::with_capacity(results.len());
        for (key, score) in results {
            if let Some(meta) = self.chunk_meta.iter().find(|m| m.key == key) {
                out.push((meta.clone(), score));
            }
        }
        Ok(out)
    }

    /// Returns true if the secondary code index is enabled and non-empty.
    pub fn has_code_index(&self) -> bool {
        self.code_index.as_ref().map(|c| !c.is_empty()).unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Adjacent code index path: replaces `vector_index.json` with `vector_index.code_index.json`.
/// Isolates naming convention so callers and tests use a single definition.
fn code_index_path_for(text_index_path: &Path) -> PathBuf {
    // stem: "vector_index", extension: "json" -> "vector_index.code_index.json"
    let stem = text_index_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "vector_index".to_string());
    let parent = text_index_path.parent().unwrap_or(Path::new("."));
    parent.join(format!("{stem}.code_index.json"))
}

// ---------------------------------------------------------------------------
// Chunking (reuses BM25 strategy: 50-line windows, 10-line overlap)
// ---------------------------------------------------------------------------

fn chunk_file(path: &Path) -> Result<Vec<BM25Chunk>, KnowledgeError> {
    let data = std::fs::read(path)?;
    let check_len = data.len().min(BINARY_DETECT_BYTES);
    if data[..check_len].contains(&0) {
        return Ok(Vec::new()); // binary file
    }
    let content = String::from_utf8_lossy(&data).into_owned();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let mut chunks = Vec::new();
    let step = CHUNK_WINDOW_LINES - CHUNK_OVERLAP_LINES;
    let mut start = 0;
    while start < lines.len() {
        let end = (start + CHUNK_WINDOW_LINES).min(lines.len());
        let chunk_content = lines[start..end].join("\n");
        chunks.push(BM25Chunk {
            path: path.to_path_buf(),
            content: chunk_content,
            line_start: u32::try_from(start + 1).unwrap_or(u32::MAX),
            line_end: u32::try_from(end).unwrap_or(u32::MAX),
            kind: crate::bm25::ChunkKind::LineBlock,
        });
        if end == lines.len() {
            break;
        }
        start += step;
    }

    Ok(chunks)
}

/// Deterministic key from (path, line_start, line_end).
fn chunk_key(path: &Path, line_start: u32, line_end: u32) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    line_start.hash(&mut hasher);
    line_end.hash(&mut hasher);
    hasher.finish()
}

fn truncate_snippet(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        s.to_string()
    } else {
        let mut end = max_bytes;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}

// Serde for ChunkMeta (needed for persistence)
impl serde::Serialize for ChunkMeta {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ChunkMeta", 5)?;
        s.serialize_field("key", &self.key)?;
        s.serialize_field("path", &self.path)?;
        s.serialize_field("line_start", &self.line_start)?;
        s.serialize_field("line_end", &self.line_end)?;
        s.serialize_field("snippet", &self.snippet)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for ChunkMeta {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Inner {
            key: u64,
            path: PathBuf,
            line_start: u32,
            line_end: u32,
            snippet: String,
        }
        let inner = Inner::deserialize(deserializer)?;
        Ok(ChunkMeta {
            key: inner.key,
            path: inner.path,
            line_start: inner.line_start,
            line_end: inner.line_end,
            snippet: inner.snippet,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_key_deterministic() {
        let k1 = chunk_key(Path::new("foo.rs"), 1, 50);
        let k2 = chunk_key(Path::new("foo.rs"), 1, 50);
        assert_eq!(k1, k2);
    }

    #[test]
    fn chunk_key_different_ranges_differ() {
        let k1 = chunk_key(Path::new("foo.rs"), 1, 50);
        let k2 = chunk_key(Path::new("foo.rs"), 41, 90);
        assert_ne!(k1, k2);
    }

    #[test]
    fn truncate_snippet_short() {
        let s = "short";
        assert_eq!(truncate_snippet(s, 100), "short");
    }

    #[test]
    fn truncate_snippet_long() {
        let s = "a".repeat(500);
        let result = truncate_snippet(&s, 100);
        assert!(result.len() <= 104); // 100 + "..."
        assert!(result.ends_with("..."));
    }
}
