//! Artifact indexer — chunks artifact files and stores results.
//!
//! # Chunking strategy
//!
//! * `.md`, `.yaml`, `.yml`, `.json`, `.toml`, `.tf` — sliding line-window
//!   (ARTIFACT_CHUNK_WINDOW_LINES=50, ARTIFACT_CHUNK_OVERLAP_LINES=10).
//!   Mirrors `Bm25Index::line_block_chunks` but lives here to avoid coupling.
//!
//! * `.sql` — split at `CREATE (TABLE|VIEW|INDEX|FUNCTION|TRIGGER|TYPE)`
//!   statement boundaries. Implemented as a line-state machine; no regex crate.
//!
//! * Everything else — same line-window as above.
//!
//! All chunk paths pass through `apply_char_cap` as the final step (D10 rule).
//!
//! # Embedding
//!
//! `vm: Option<&dyn VectorModule>` is accepted. When `None`, chunks are stored
//! with `embedding=None`. The in-memory cosine search in `search.rs` will
//! return empty results for that artifact until it is re-indexed with a live
//! model. This is the correct Phase A behaviour — no panic, no stub embeddings.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use elai_core::ClockProvider;
use tracing::{debug, info, warn};

use crate::chunking::cap::apply_char_cap;
use crate::chunking::constants::MAX_CHUNK_CHARS;
use crate::bm25::{BM25Chunk, ChunkKind};
use crate::vector::VectorModule;

use super::manifest::ArtifactError;
use super::staleness::{detect_staleness, hash_artifact_files};
use super::store::{chunk_id, ArtifactStore};
use super::types::{
    ArtifactChunk, ArtifactIndexAllResult, ArtifactIndexState, ArtifactManifest, ArtifactSpec,
    ArtifactStatus,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Sliding window size for line-based artifact chunking.
pub const ARTIFACT_CHUNK_WINDOW_LINES: usize = 50;

/// Overlap between consecutive line-window chunks.
pub const ARTIFACT_CHUNK_OVERLAP_LINES: usize = 10;

// ---------------------------------------------------------------------------
// ArtifactIndexer
// ---------------------------------------------------------------------------

/// Indexes artifact specs into the artifact SQLite store.
///
/// Does not own a ClockProvider field — callers pass `now_secs` directly to
/// avoid requiring `Arc<dyn ClockProvider>` in unit tests.
pub struct ArtifactIndexer<'a> {
    pub store: &'a ArtifactStore,
    pub vm: Option<&'a dyn VectorModule>,
}

impl<'a> ArtifactIndexer<'a> {
    pub fn new(store: &'a ArtifactStore, vm: Option<&'a dyn VectorModule>) -> Self {
        Self { store, vm }
    }

    // -----------------------------------------------------------------------
    // index_artifact
    // -----------------------------------------------------------------------

    /// Index one artifact. Returns the new [`ArtifactIndexState`].
    ///
    /// Per file:
    ///   1. Read content + blake3 hash.
    ///   2. If hash matches existing → skip.
    ///   3. Else: chunk → apply_char_cap → embed (if vm present) → upsert.
    ///
    /// Errors on individual files are non-fatal: the indexer logs a warning,
    /// records the file in `changed_files`, and continues. A final status of
    /// `Stale` is returned only if at least one file failed.
    pub fn index_artifact(
        &self,
        spec: &ArtifactSpec,
        prev_state: Option<&ArtifactIndexState>,
        now_secs: i64,
    ) -> Result<ArtifactIndexState, ArtifactError> {
        info!(artifact = spec.name, path = %spec.path.display(), "indexing artifact");

        let stale_result = detect_staleness(spec, prev_state)?;

        if stale_result.is_clean() {
            debug!(artifact = spec.name, "artifact unchanged — skipping re-index");
            if let Some(prev) = prev_state {
                return Ok(ArtifactIndexState {
                    name: prev.name.clone(),
                    last_indexed_at: prev.last_indexed_at,
                    file_hashes: prev.file_hashes.clone(),
                    chunk_count: prev.chunk_count,
                    status: ArtifactStatus::Indexed,
                });
            }
        }

        // Drop removed files
        for removed in &stale_result.removed {
            let rel = removed.to_string_lossy();
            if let Err(e) = self.store.drop_chunks_for_file(&spec.name, &rel) {
                warn!(artifact = spec.name, file = %rel, error = %e, "failed to drop removed file chunks");
            }
        }

        let current_hashes = hash_artifact_files(spec)?;
        let mut new_file_hashes: BTreeMap<String, String> = BTreeMap::new();
        let mut total_chunks: usize = 0;
        let mut failed_files: Vec<String> = Vec::new();

        // Files to (re-)index: changed + added
        let needs_index: Vec<PathBuf> = stale_result
            .changed
            .iter()
            .chain(stale_result.added.iter())
            .cloned()
            .collect();

        for rel_path in &needs_index {
            let abs_path = if spec.is_dir {
                spec.path.join(rel_path)
            } else {
                spec.path.clone()
            };
            let rel_str = rel_path.to_string_lossy().to_string();

            match self.index_single_file(spec, &abs_path, &rel_str, now_secs) {
                Ok(chunk_count) => {
                    total_chunks += chunk_count;
                    if let Some(hash) = current_hashes.get(rel_path) {
                        new_file_hashes.insert(rel_str, hash.clone());
                    }
                }
                Err(e) => {
                    warn!(artifact = spec.name, file = %rel_str, error = %e, "failed to index file");
                    failed_files.push(rel_str);
                }
            }
        }

        // Carry forward unchanged hashes from previous state
        if let Some(prev) = prev_state {
            for (path, hash) in &prev.file_hashes {
                if !new_file_hashes.contains_key(path)
                    && !stale_result.removed.iter().any(|r| r.to_string_lossy() == path.as_str())
                {
                    new_file_hashes.insert(path.clone(), hash.clone());
                }
            }
        } else {
            // First index — new_file_hashes covers everything indexed above
            // (unchanged set is empty when no prev_state)
        }

        // For files not in needs_index, add hashes from current scan
        for unchanged in &stale_result.unchanged {
            let rel_str = unchanged.to_string_lossy().to_string();
            if let Some(hash) = current_hashes.get(unchanged) {
                new_file_hashes.entry(rel_str).or_insert_with(|| hash.clone());
            }
        }

        let status = if failed_files.is_empty() {
            ArtifactStatus::Indexed
        } else {
            ArtifactStatus::Stale {
                changed_files: failed_files,
            }
        };

        // Re-count from store for accuracy after upserts
        let stored_count = self.store.count_chunks(&spec.name).unwrap_or(total_chunks);

        Ok(ArtifactIndexState {
            name: spec.name.clone(),
            last_indexed_at: now_secs,
            file_hashes: new_file_hashes,
            chunk_count: stored_count,
            status,
        })
    }

    // -----------------------------------------------------------------------
    // index_all_artifacts
    // -----------------------------------------------------------------------

    /// Index all artifacts from `manifest`. Returns a summary.
    ///
    /// Individual artifact failures are non-fatal: stored in `errors`, loop
    /// continues.
    pub fn index_all_artifacts(
        &self,
        manifest: &ArtifactManifest,
        prev_states: &BTreeMap<String, ArtifactIndexState>,
        now_secs: i64,
    ) -> ArtifactIndexAllResult {
        let mut result = ArtifactIndexAllResult::default();

        for spec in &manifest.items {
            let prev = prev_states.get(&spec.name);
            match self.index_artifact(spec, prev, now_secs) {
                Ok(_state) => {
                    result.indexed.push(spec.name.clone());
                }
                Err(e) => {
                    warn!(artifact = spec.name, error = %e, "artifact indexing failed");
                    result.errors.push((spec.name.clone(), e.to_string()));
                }
            }
        }

        result
    }

    // -----------------------------------------------------------------------
    // Private: index one file
    // -----------------------------------------------------------------------

    fn index_single_file(
        &self,
        spec: &ArtifactSpec,
        abs_path: &Path,
        rel_str: &str,
        now_secs: i64,
    ) -> Result<usize, ArtifactError> {
        // J-33 Tier-1: when ELAI_DOC_PARSER_ENABLE is ON and the file is a
        // .pdf or .docx, route through the MinerU `serve` doc-parser lane
        // instead of read_to_string (which would decode raw PDF bytes as
        // UTF-8 text and produce garbage chunks). When the flag is OFF,
        // today's read_to_string behavior is preserved for every extension.
        // When the flag is ON but the client cannot be constructed (e.g.
        // base_url unset), the error is surfaced explicitly via
        // ArtifactError::Other rather than silently falling back to
        // read_to_string on a PDF (which would hide a misconfiguration).
        let content = if is_pdf_or_docx(abs_path) {
            match crate::doc_parser_client::DocParserClient::from_env() {
                Some(Ok(client)) => {
                    let parsed = client.parse(abs_path).map_err(|e| {
                        ArtifactError::Other(format!(
                            "doc_parser failed for {rel_str}: {e}"
                        ))
                    })?;
                    parsed.markdown
                }
                Some(Err(e)) => {
                    return Err(ArtifactError::Other(format!(
                        "doc_parser misconfigured for {rel_str}: {e}"
                    )));
                }
                None => std::fs::read_to_string(abs_path).map_err(ArtifactError::Io)?,
            }
        } else {
            std::fs::read_to_string(abs_path).map_err(ArtifactError::Io)?
        };
        if content.is_empty() {
            return Ok(0);
        }

        let file_hash = {
            let bytes = content.as_bytes();
            blake3::hash(bytes).to_hex().to_string()
        };

        // Drop existing chunks for this file before upserting new ones
        let _ = self.store.drop_chunks_for_file(&spec.name, rel_str);

        let bm25_chunks = chunk_file(abs_path, &content);
        let capped = apply_char_cap(bm25_chunks);

        let mut count = 0;
        for bm25_chunk in &capped {
            let start_line = bm25_chunk.line_start;
            let end_line = bm25_chunk.line_end;
            let id = chunk_id(&spec.name, rel_str, start_line);

            let prefixed_content = format!(
                "context_artifact: {name}\n{content}",
                name = spec.name,
                content = bm25_chunk.content,
            );

            let embedding = match self.vm {
                Some(vm) => {
                    match vm.embed_text(&prefixed_content) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            warn!(
                                artifact = spec.name,
                                file = rel_str,
                                error = %e,
                                "embed_text failed — storing chunk without embedding"
                            );
                            None
                        }
                    }
                }
                None => None,
            };

            let artifact_chunk = ArtifactChunk {
                id,
                artifact_name: spec.name.clone(),
                artifact_description: spec.description.clone(),
                relative_path: rel_str.to_string(),
                content: bm25_chunk.content.clone(),
                start_line,
                end_line,
                content_hash: file_hash.clone(),
                embedding,
            };

            self.store.upsert_chunk(&artifact_chunk, now_secs)?;
            count += 1;
        }

        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// Chunking — line-window + SQL-boundary
// ---------------------------------------------------------------------------

/// Chunk `content` for the given file path.
///
/// Dispatch:
/// - `.sql` → CREATE-boundary state machine
/// - everything else → line-window (ARTIFACT_CHUNK_WINDOW_LINES / OVERLAP)
///
/// Returns `Vec<BM25Chunk>` so callers can pass to `apply_char_cap`.
pub fn chunk_file(path: &Path, content: &str) -> Vec<BM25Chunk> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "sql" {
        chunk_sql(path, content)
    } else {
        chunk_line_window(path, content)
    }
}

/// Line-window chunking: sliding window of ARTIFACT_CHUNK_WINDOW_LINES lines
/// with ARTIFACT_CHUNK_OVERLAP_LINES line overlap. Produces `ChunkKind::LineBlock`.
pub fn chunk_line_window(path: &Path, content: &str) -> Vec<BM25Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    let step = ARTIFACT_CHUNK_WINDOW_LINES.saturating_sub(ARTIFACT_CHUNK_OVERLAP_LINES).max(1);
    let mut chunks = Vec::new();
    let mut start = 0usize;

    loop {
        let end = (start + ARTIFACT_CHUNK_WINDOW_LINES).min(lines.len());
        let chunk_content = lines[start..end].join("\n");

        // Enforce hard char cap inline (apply_char_cap will also run, defence-in-depth)
        let chunk_content = if chunk_content.len() > MAX_CHUNK_CHARS {
            chunk_content[..MAX_CHUNK_CHARS].to_string()
        } else {
            chunk_content
        };

        chunks.push(BM25Chunk {
            path: path.to_path_buf(),
            content: chunk_content,
            line_start: u32::try_from(start + 1).unwrap_or(1),
            line_end: u32::try_from(end).unwrap_or(1),
            kind: ChunkKind::LineBlock,
        });

        if end >= lines.len() {
            break;
        }
        start += step;
    }

    chunks
}

/// SQL CREATE-boundary chunking.
///
/// Splits the file at lines that begin a new top-level CREATE statement
/// (`CREATE TABLE|VIEW|INDEX|FUNCTION|TRIGGER|TYPE`). The preceding
/// accumulated lines form one chunk. Implemented as a simple line-state machine
/// — no regex crate.
pub fn chunk_sql(path: &Path, content: &str) -> Vec<BM25Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    let mut chunks: Vec<BM25Chunk> = Vec::new();
    let mut current_start = 0usize;

    for (i, line) in lines.iter().enumerate() {
        if i > 0 && is_sql_create_boundary(line) {
            let end = i;
            let text = lines[current_start..end].join("\n");
            if !text.trim().is_empty() {
                let capped = if text.len() > MAX_CHUNK_CHARS {
                    text[..MAX_CHUNK_CHARS].to_string()
                } else {
                    text
                };
                chunks.push(BM25Chunk {
                    path: path.to_path_buf(),
                    content: capped,
                    line_start: u32::try_from(current_start + 1).unwrap_or(1),
                    line_end: u32::try_from(end).unwrap_or(1),
                    kind: ChunkKind::LineBlock,
                });
            }
            current_start = i;
        }
    }

    // Trailing chunk
    let text = lines[current_start..].join("\n");
    if !text.trim().is_empty() {
        let capped = if text.len() > MAX_CHUNK_CHARS {
            text[..MAX_CHUNK_CHARS].to_string()
        } else {
            text
        };
        chunks.push(BM25Chunk {
            path: path.to_path_buf(),
            content: capped,
            line_start: u32::try_from(current_start + 1).unwrap_or(1),
            line_end: u32::try_from(lines.len()).unwrap_or(1),
            kind: ChunkKind::LineBlock,
        });
    }

    // Fallback: if no boundaries were found, use line-window chunking instead
    if chunks.len() <= 1 {
        return chunk_line_window(path, content);
    }

    chunks
}

/// Returns true if `line` starts a new SQL CREATE boundary.
/// Checks for: `CREATE TABLE`, `CREATE VIEW`, `CREATE INDEX`, `CREATE FUNCTION`,
/// `CREATE TRIGGER`, `CREATE TYPE` (case-insensitive, leading whitespace allowed).
fn is_sql_create_boundary(line: &str) -> bool {
    let trimmed = line.trim();
    let upper = trimmed.to_uppercase();
    if !upper.starts_with("CREATE") {
        return false;
    }
    // Tokenize to find the second keyword
    let mut parts = upper.split_whitespace();
    let _ = parts.next(); // "CREATE"
    let next = match parts.next() {
        Some(t) => t,
        None => return false,
    };
    // Allow CREATE [OR REPLACE] — skip "OR REPLACE"
    let keyword = if next == "OR" {
        // CREATE OR REPLACE TABLE/VIEW/...
        let _ = parts.next(); // "REPLACE"
        match parts.next() {
            Some(t) => t,
            None => return false,
        }
    } else {
        next
    };
    matches!(
        keyword,
        "TABLE" | "VIEW" | "INDEX" | "UNIQUE" | "FUNCTION" | "TRIGGER" | "TYPE" | "PROCEDURE"
    )
}

/// J-33 Tier-1: return true when `path`'s extension is `pdf` or `docx`
/// (case-insensitive). The doc-parser lane routes these through MinerU
/// `serve` when ELAI_DOC_PARSER_ENABLE is ON; all other extensions stay on
/// the existing read_to_string path regardless of the flag.
fn is_pdf_or_docx(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => {
            let lower = ext.to_ascii_lowercase();
            lower == "pdf" || lower == "docx"
        }
        None => false,
    }
}

// ---------------------------------------------------------------------------
// Clock helper
// ---------------------------------------------------------------------------

/// Extract unix seconds from a [`ClockProvider`]. Returns 0 on overflow.
pub fn clock_unix_secs(clock: &dyn ClockProvider) -> i64 {
    clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .unwrap_or(0)
}
