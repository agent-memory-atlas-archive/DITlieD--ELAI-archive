use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::bm25::BM25Chunk;
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
        }
    }

    /// Load a persisted index from disk.
    pub fn load(path: &Path, dim: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let index = BruteForceIndex::load(path, dim)?;
        // Meta must be loaded separately (saved alongside index).
        let meta_path = path.with_extension("meta.json");
        let meta: Vec<ChunkMeta> = if meta_path.exists() {
            let data = std::fs::read(&meta_path)?;
            serde_json::from_slice(&data)?
        } else {
            Vec::new()
        };
        Ok(Self { index, chunk_meta: meta, dim })
    }

    /// Persist index + metadata to disk.
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.index.save(path)?;
        let meta_path = path.with_extension("meta.json");
        let meta_bytes = serde_json::to_vec(&self.chunk_meta)?;
        std::fs::write(&meta_path, meta_bytes)?;
        Ok(())
    }

    /// Index a single file: chunk it, embed each chunk, add to index.
    /// Uses VectorCache for dedup (skip embedding if content hash matches).
    pub fn index_file(
        &mut self,
        path: &Path,
        vector_module: &dyn VectorModule,
        cache: Option<&VectorCache>,
    ) -> Result<IndexStats, Box<dyn std::error::Error>> {
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
            let embedding = if let Some(c) = cache {
                if let Ok(Some(dual)) = c.lookup(&content_hash) {
                    stats.chunks_cached += 1;
                    // Use the text embedding for search (more general than code-specific).
                    Some(dual.text_vec)
                } else {
                    None
                }
            } else {
                None
            };

            let embedding = match embedding {
                Some(e) => e,
                None => {
                    // Embed the chunk content.
                    match vector_module.embed_text(&chunk.content) {
                        Ok(vec) => {
                            stats.chunks_embedded += 1;
                            // Store in cache if available.
                            if let Some(c) = cache {
                                if let Ok(code_vec) = vector_module.embed_code(&chunk.content) {
                                    if let Ok(dual) = crate::vector::DualVector::new(code_vec, vec.clone()) {
                                        let now = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs() as i64;
                                        let _ = c.insert(&content_hash, &dual, now);
                                    }
                                }
                            }
                            vec
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
        }
        self.chunk_meta.retain(|m| m.path != path);
    }

    /// Search the vector index and return chunk metadata for the top-k results.
    pub fn search(&self, query_vec: &[f32], k: usize) -> Result<Vec<(ChunkMeta, f32)>, Box<dyn std::error::Error>> {
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
}

// ---------------------------------------------------------------------------
// Chunking (reuses BM25 strategy: 50-line windows, 10-line overlap)
// ---------------------------------------------------------------------------

fn chunk_file(path: &Path) -> Result<Vec<BM25Chunk>, Box<dyn std::error::Error>> {
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
            line_start: (start + 1) as u32,
            line_end: end as u32,
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
