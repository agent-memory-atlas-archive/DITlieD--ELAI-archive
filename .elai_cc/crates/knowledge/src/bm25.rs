use std::path::{Path, PathBuf};

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, Term};

use crate::LanguageGrammar;
use crate::dkg::{ext_to_lang_id, RustGrammar, PythonGrammar, GoGrammar, TypeScriptGrammar, JavaGrammar, CSharpGrammar, RubyGrammar};
use crate::chunking;
use crate::error::KnowledgeError;

/// Index format version for tantivy 0.26.0 (verified from tantivy src/lib.rs).
pub const INDEX_FORMAT_VERSION: u32 = 7;

/// Heap budget for the IndexWriter (50 MB).
const WRITER_HEAP_BYTES: usize = 50_000_000;

/// Default sliding window size for line-block chunking (fallback).
const CHUNK_WINDOW_LINES: usize = 50;

/// Overlap between consecutive line-block chunks (fallback).
const CHUNK_OVERLAP_LINES: usize = 10;

/// Bytes to inspect for binary detection (null byte check).
const BINARY_DETECT_BYTES: usize = 8192;

/// Approximate characters per token for chunk size estimation.
const CHARS_PER_TOKEN: usize = 4;

/// Minimum chunk size in tokens (target).
const MIN_CHUNK_TOKENS: usize = 200;

/// Maximum chunk size in tokens (target).
const MAX_CHUNK_TOKENS: usize = 800;

/// Hard maximum chunk size in tokens (deterministic check: no chunk exceeds 1000).
const HARD_MAX_CHUNK_TOKENS: usize = 1000;

/// Overlap in tokens between adjacent AST chunks from the same file.
const OVERLAP_TOKENS: usize = 50;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChunkKind {
    Function,
    Struct,
    Enum,
    Impl,
    Class,
    Module,
    Trait,
    LineBlock,
    File,
    /// B-32: chunks produced by the character-based chunker for minified
    /// files (avg line length > MAX_AVG_LINE_LENGTH). Persists as
    /// "minified"; old indexes lacking this variant fall back to File via
    /// the from_str default arm.
    Minified,
}

impl ChunkKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChunkKind::Function => "function",
            ChunkKind::Struct => "struct",
            ChunkKind::Enum => "enum",
            ChunkKind::Impl => "impl",
            ChunkKind::Class => "class",
            ChunkKind::Module => "module",
            ChunkKind::Trait => "trait",
            ChunkKind::LineBlock => "line_block",
            ChunkKind::File => "file",
            ChunkKind::Minified => "minified",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "function" => ChunkKind::Function,
            "struct" => ChunkKind::Struct,
            "enum" => ChunkKind::Enum,
            "impl" => ChunkKind::Impl,
            "class" => ChunkKind::Class,
            "module" => ChunkKind::Module,
            "trait" => ChunkKind::Trait,
            "line_block" => ChunkKind::LineBlock,
            "minified" => ChunkKind::Minified,
            _ => ChunkKind::File,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BM25Chunk {
    pub path: PathBuf,
    pub content: String,
    pub line_start: u32,
    pub line_end: u32,
    pub kind: ChunkKind,
}

#[derive(Debug, Clone)]
pub struct BM25Hit {
    pub path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub score: f32,
    pub chunk_kind: ChunkKind,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub chunks_indexed: usize,
    pub files_skipped_binary: usize,
    /// Wall-clock elapsed time. Caller-populated (measurement binary sets this
    /// externally via std::time::Instant to avoid raw entropy in library code).
    pub elapsed_ms: u64,
}

// ---------------------------------------------------------------------------
// Schema field handles
// ---------------------------------------------------------------------------

struct SchemaFields {
    path: Field,
    content: Field,
    line_start: Field,
    line_end: Field,
    chunk_kind: Field,
}

// ---------------------------------------------------------------------------
// Bm25Index
// ---------------------------------------------------------------------------

/// Full-text BM25 index backed by tantivy.
///
/// Single writer model: `index_directory` holds exclusive write access.
/// Callers must not invoke `update_file` concurrently with `index_directory`.
pub struct Bm25Index {
    #[allow(dead_code)] // used in Phase 1B.5 measure binary
    index_path: PathBuf,
    index: Index,
    writer: IndexWriter,
    reader: IndexReader,
    #[allow(dead_code)] // used in Phase 1C schema migration
    schema: Schema,
    fields: SchemaFields,
}

impl Bm25Index {
    /// Create or open a BM25 index for the given project.
    ///
    /// Index directory: `project_root/projects/{project_id}/bm25`
    pub fn new(
        project_root: &Path,
        project_id: &str,
    ) -> Result<Self, KnowledgeError> {
        let index_path = project_root
            .join("projects")
            .join(project_id)
            .join("bm25");
        std::fs::create_dir_all(&index_path)?;

        let mut schema_builder = Schema::builder();
        let path = schema_builder.add_text_field("path", STRING | STORED);
        let content = schema_builder.add_text_field("content", TEXT | STORED);
        let line_start = schema_builder.add_u64_field("line_start", STORED);
        let line_end = schema_builder.add_u64_field("line_end", STORED);
        let chunk_kind = schema_builder.add_text_field("chunk_kind", STRING | STORED);
        let schema = schema_builder.build();

        let index = Index::create_in_dir(&index_path, schema.clone())
            .or_else(|_| Index::open_in_dir(&index_path))?;

        // Validate schema fields match expectations (guards against stale index from prior schema)
        let existing_schema = index.schema();
        let expected_fields = ["path", "content", "line_start", "line_end", "chunk_kind"];
        for name in &expected_fields {
            if existing_schema.get_field(name).is_err() {
                return Err(KnowledgeError::SchemaMismatch {
                    path: index_path.clone(),
                    field: (*name).to_string(),
                });
            }
        }

        let writer = index.writer(WRITER_HEAP_BYTES)?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        let fields = SchemaFields {
            path,
            content,
            line_start,
            line_end,
            chunk_kind,
        };

        Ok(Self {
            index_path,
            index,
            writer,
            reader,
            schema,
            fields,
        })
    }

    /// Index a single file. If `chunks` is None, the file is read from disk
    /// and chunked with the default 50-line sliding window.
    /// Returns the number of chunks indexed.
    pub fn index_file(
        &mut self,
        path: &Path,
        chunks: Option<Vec<BM25Chunk>>,
    ) -> Result<usize, KnowledgeError> {
        let chunks = match chunks {
            Some(c) => c,
            None => self.chunk_file(path)?,
        };
        let count = chunks.len();
        let path_str = path.to_string_lossy();
        for chunk in &chunks {
            self.writer.add_document(doc!(
                self.fields.path => path_str.as_ref(),
                self.fields.content => chunk.content.as_str(),
                self.fields.line_start => chunk.line_start as u64,
                self.fields.line_end => chunk.line_end as u64,
                self.fields.chunk_kind => chunk.kind.as_str()
            ))?;
        }
        self.writer.commit()?;
        self.reader.reload()?;
        Ok(count)
    }

    /// Index all non-binary files under `root` recursively.
    /// Returns aggregate statistics.
    pub fn index_directory(
        &mut self,
        root: &Path,
    ) -> Result<IndexStats, KnowledgeError> {
        let mut stats = IndexStats {
            files_indexed: 0,
            chunks_indexed: 0,
            files_skipped_binary: 0,
            elapsed_ms: 0,
        };
        self.walk_dir(root, &mut stats)?;
        self.writer.commit()?;
        self.reader.reload()?;
        Ok(stats)
    }

    /// Search the index with a BM25 query.
    /// Returns up to `limit` hits sorted by descending score.
    pub fn query(
        &self,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<BM25Hit>, KnowledgeError> {
        if query_text.trim().is_empty() {
            return Ok(Vec::new());
        }
        let query_parser = QueryParser::for_index(&self.index, vec![self.fields.content]);
        let query = query_parser.parse_query(query_text)?;
        let searcher = self.reader.searcher();
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit).order_by_score())?;

        let mut hits = Vec::with_capacity(top_docs.len());
        for (score, doc_address) in top_docs {
            let doc = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
            let path_val = doc
                .get_first(self.fields.path)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let content_val = doc
                .get_first(self.fields.content)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let line_start = u32::try_from(
                doc.get_first(self.fields.line_start)
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            )
            .unwrap_or(u32::MAX);
            let line_end = u32::try_from(
                doc.get_first(self.fields.line_end)
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            )
            .unwrap_or(u32::MAX);
            let chunk_kind_str = doc
                .get_first(self.fields.chunk_kind)
                .and_then(|v| v.as_str())
                .unwrap_or("file");
            let snippet: String = content_val.chars().take(200).collect();

            hits.push(BM25Hit {
                path: path_val,
                line_start,
                line_end,
                score,
                chunk_kind: ChunkKind::from_str(chunk_kind_str),
                snippet,
            });
        }
        Ok(hits)
    }

    /// Delete all chunks associated with `path` from the index.
    /// Returns the tantivy opstamp (monotonic operation counter), not a document count.
    pub fn delete_file(
        &mut self,
        path: &Path,
    ) -> Result<usize, KnowledgeError> {
        let path_str = path.to_string_lossy();
        let term = Term::from_field_text(self.fields.path, &path_str);
        let opstamp = self.writer.delete_term(term);
        self.writer.commit()?;
        self.reader.reload()?;
        Ok(opstamp as usize)
    }

    /// Delete then re-index a file. Equivalent to delete_file + index_file.
    /// Returns the number of new chunks indexed.
    pub fn update_file(
        &mut self,
        path: &Path,
        chunks: Option<Vec<BM25Chunk>>,
    ) -> Result<usize, KnowledgeError> {
        let path_str = path.to_string_lossy();
        let term = Term::from_field_text(self.fields.path, &path_str);
        self.writer.delete_term(term);

        let chunks = match chunks {
            Some(c) => c,
            None => self.chunk_file(path)?,
        };
        let count = chunks.len();
        for chunk in &chunks {
            self.writer.add_document(doc!(
                self.fields.path => path_str.as_ref(),
                self.fields.content => chunk.content.as_str(),
                self.fields.line_start => chunk.line_start as u64,
                self.fields.line_end => chunk.line_end as u64,
                self.fields.chunk_kind => chunk.kind.as_str()
            ))?;
        }
        self.writer.commit()?;
        self.reader.reload()?;
        Ok(count)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn is_binary(data: &[u8]) -> bool {
        let check_len = data.len().min(BINARY_DETECT_BYTES);
        data[..check_len].contains(&0)
    }

    fn chunk_file(&self, path: &Path) -> Result<Vec<BM25Chunk>, KnowledgeError> {
        let data = std::fs::read(path)?;
        if Self::is_binary(&data) {
            return Ok(Vec::new());
        }
        let content = String::from_utf8_lossy(&data).into_owned();
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Ok(vec![BM25Chunk {
                path: path.to_path_buf(),
                content: String::new(),
                line_start: 1,
                line_end: 1,
                kind: ChunkKind::File,
            }]);
        }

        // B-32 D1: minified-first short-circuit. Avg line length > 500 chars
        // routes through the character-based chunker; apply_char_cap is the
        // last step. tracing::info! is the Step A observable trigger (D3) —
        // Step B will replace this with an SSB row once the lock releases.
        if chunking::detect::is_minified(&content, lines.len()) {
            let avg_line_length = content.len() / lines.len().max(1);
            let char_count = content.len();
            let line_count_u32 = u32::try_from(lines.len()).unwrap_or(u32::MAX);
            drop(lines);
            let mut chunks = chunking::cap::apply_char_cap(
                chunking::character::chunk_by_characters(path, &content),
            );
            // Zero-orphan File chunk (exempt from cap per D10).
            chunks.push(BM25Chunk {
                path: path.to_path_buf(),
                content,
                line_start: 1,
                line_end: line_count_u32,
                kind: ChunkKind::File,
            });
            tracing::info!(
                target: "knowledge::chunking",
                path = %path.display(),
                char_count = char_count,
                chunk_count = chunks.len(),
                avg_line_length = avg_line_length,
                "minified chunked"
            );
            return Ok(chunks);
        }

        let mut chunks = Self::try_ast_chunk(path, content.as_bytes(), &lines);
        if chunks.is_empty() {
            chunks = Self::line_block_chunks(path, &lines);
        }

        // B-32 D1: apply_char_cap as the LAST step (excludes File-kind per D10).
        chunks = chunking::cap::apply_char_cap(chunks);

        // Zero-orphan invariant: every non-binary file gets at least one File-kind chunk
        let line_count = u32::try_from(lines.len()).unwrap_or(u32::MAX);
        drop(lines);
        chunks.push(BM25Chunk {
            path: path.to_path_buf(),
            content,
            line_start: 1,
            line_end: line_count,
            kind: ChunkKind::File,
        });

        Ok(chunks)
    }

    fn walk_dir(
        &mut self,
        dir: &Path,
        stats: &mut IndexStats,
    ) -> Result<(), KnowledgeError> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            // Skip hidden directories and .elai_cc index store
            if name_str.starts_with('.') {
                continue;
            }
            let path = entry.path();
            if ft.is_dir() {
                self.walk_dir(&path, stats)?;
            } else if ft.is_file() {
                let data = match std::fs::read(&path) {
                    Ok(d) => d,
                    // Intentional: skip unreadable files (permissions, broken symlinks).
                    Err(_) => continue,
                };
                if Self::is_binary(&data) {
                    stats.files_skipped_binary += 1;
                    continue;
                }
                let content = String::from_utf8_lossy(&data);
                let lines: Vec<&str> = content.lines().collect();
                let chunks = Self::make_chunks_from_content(&path, &lines, &content);
                let path_str = path.to_string_lossy();
                for chunk in &chunks {
                    self.writer.add_document(doc!(
                        self.fields.path => path_str.as_ref(),
                        self.fields.content => chunk.content.as_str(),
                        self.fields.line_start => chunk.line_start as u64,
                        self.fields.line_end => chunk.line_end as u64,
                        self.fields.chunk_kind => chunk.kind.as_str()
                    ))?;
                }
                stats.files_indexed += 1;
                stats.chunks_indexed += chunks.len();
            }
        }
        Ok(())
    }

    fn make_chunks_from_content(
        path: &Path,
        lines: &[&str],
        full_content: &str,
    ) -> Vec<BM25Chunk> {
        if lines.is_empty() {
            return vec![BM25Chunk {
                path: path.to_path_buf(),
                content: String::new(),
                line_start: 1,
                line_end: 1,
                kind: ChunkKind::File,
            }];
        }

        // B-32 D1: minified-first short-circuit (mirrors chunk_file).
        if chunking::detect::is_minified(full_content, lines.len()) {
            let avg_line_length = full_content.len() / lines.len().max(1);
            let mut chunks = chunking::cap::apply_char_cap(
                chunking::character::chunk_by_characters(path, full_content),
            );
            chunks.push(BM25Chunk {
                path: path.to_path_buf(),
                content: full_content.to_string(),
                line_start: 1,
                line_end: u32::try_from(lines.len()).unwrap_or(u32::MAX),
                kind: ChunkKind::File,
            });
            tracing::info!(
                target: "knowledge::chunking",
                path = %path.display(),
                char_count = full_content.len(),
                chunk_count = chunks.len(),
                avg_line_length = avg_line_length,
                "minified chunked"
            );
            return chunks;
        }

        let mut chunks = Self::try_ast_chunk(path, full_content.as_bytes(), lines);
        if chunks.is_empty() {
            chunks = Self::line_block_chunks(path, lines);
        }

        // B-32 D1: apply_char_cap as the LAST step (excludes File-kind per D10).
        chunks = chunking::cap::apply_char_cap(chunks);

        // Zero-orphan invariant
        chunks.push(BM25Chunk {
            path: path.to_path_buf(),
            content: full_content.to_string(),
            line_start: 1,
            line_end: u32::try_from(lines.len()).unwrap_or(u32::MAX),
            kind: ChunkKind::File,
        });

        chunks
    }

    /// Produce 50-line sliding-window chunks (fallback for non-AST files).
    fn line_block_chunks(path: &Path, lines: &[&str]) -> Vec<BM25Chunk> {
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
                kind: ChunkKind::LineBlock,
            });
            if end == lines.len() {
                break;
            }
            start += step;
        }
        chunks
    }

    /// Attempt AST-aware chunking. Returns empty Vec if the file has no
    /// supported grammar or if parsing fails (caller falls back to line blocks).
    fn try_ast_chunk(path: &Path, source: &[u8], lines: &[&str]) -> Vec<BM25Chunk> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let lang_id = match ext_to_lang_id(ext) {
            Some(id) => id,
            None => return Vec::new(),
        };

        let grammar: Box<dyn LanguageGrammar> = match lang_id {
            "rust" => Box::new(RustGrammar),
            "python" => Box::new(PythonGrammar),
            "go" => Box::new(GoGrammar),
            "typescript" => Box::new(TypeScriptGrammar),
            "java" => Box::new(JavaGrammar),
            "csharp" => Box::new(CSharpGrammar),
            "ruby" => Box::new(RubyGrammar),
            _ => return Vec::new(),
        };

        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&grammar.tree_sitter_language()).is_err() {
            return Vec::new();
        }

        let tree = match parser.parse(source, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let nodes = grammar.extract_nodes(&tree, source);
        if nodes.is_empty() {
            return Vec::new();
        }

        // Collect top-level AST boundaries (nodes without a parent).
        let mut boundaries: Vec<AstBoundary> = Vec::new();
        for node in &nodes {
            if node.parent_idx.is_some() {
                continue;
            }
            let kind = symbol_kind_to_chunk_kind(node.kind);
            boundaries.push(AstBoundary {
                start_line: node.start_line,
                end_line: node.end_line,
                kind,
            });
        }

        if boundaries.is_empty() {
            return Vec::new();
        }

        // Sort by start_line (should already be in order, but be safe).
        boundaries.sort_by_key(|b| b.start_line);

        // Build chunks from boundaries, handling size constraints and overlap.
        build_ast_chunks(path, lines, &boundaries)
    }
}

// ---------------------------------------------------------------------------
// AST chunking helpers (free functions, outside impl block)
// ---------------------------------------------------------------------------

struct AstBoundary {
    start_line: u32,
    end_line: u32,
    kind: ChunkKind,
}

fn estimate_tokens(text: &str) -> usize {
    // Rough estimate: 1 token ≈ 4 characters for code.
    (text.len() + CHARS_PER_TOKEN - 1) / CHARS_PER_TOKEN
}

fn symbol_kind_to_chunk_kind(kind: crate::SymbolKind) -> ChunkKind {
    match kind {
        crate::SymbolKind::Function => ChunkKind::Function,
        crate::SymbolKind::Struct => ChunkKind::Struct,
        crate::SymbolKind::Enum => ChunkKind::Enum,
        crate::SymbolKind::Impl => ChunkKind::Impl,
        crate::SymbolKind::Trait => ChunkKind::Trait,
        crate::SymbolKind::Module => ChunkKind::Module,
        _ => ChunkKind::Function,
    }
}

/// Build BM25 chunks from sorted AST boundaries with size constraints and overlap.
fn build_ast_chunks(
    path: &Path,
    lines: &[&str],
    boundaries: &[AstBoundary],
) -> Vec<BM25Chunk> {
    let mut chunks: Vec<BM25Chunk> = Vec::new();
    let min_chars = MIN_CHUNK_TOKENS * CHARS_PER_TOKEN;
    let hard_max_chars = HARD_MAX_CHUNK_TOKENS * CHARS_PER_TOKEN;
    let overlap_chars = OVERLAP_TOKENS * CHARS_PER_TOKEN;

    // Also capture gaps between AST nodes (e.g., top-level use/const/comments).
    let mut covered_end: u32 = 0;

    for boundary in boundaries {
        // Emit gap chunk if there are uncovered lines before this boundary.
        if boundary.start_line > covered_end + 1 {
            let gap_start = if covered_end == 0 { 1 } else { covered_end + 1 };
            let gap_content = extract_lines(lines, gap_start, boundary.start_line - 1);
            if estimate_tokens(&gap_content) > 0 {
                chunks.push(BM25Chunk {
                    path: path.to_path_buf(),
                    content: gap_content,
                    line_start: gap_start,
                    line_end: boundary.start_line - 1,
                    kind: ChunkKind::LineBlock,
                });
            }
        }

        let node_content = extract_lines(lines, boundary.start_line, boundary.end_line);
        let node_chars = node_content.len();

        let max_target_chars = MAX_CHUNK_TOKENS * CHARS_PER_TOKEN;
        if node_chars <= max_target_chars {
            // Node fits within target max — emit as single chunk.
            chunks.push(BM25Chunk {
                path: path.to_path_buf(),
                content: node_content,
                line_start: boundary.start_line,
                line_end: boundary.end_line,
                kind: boundary.kind,
            });
        } else {
            // Node too large — split into sub-chunks at hard_max boundary with overlap.
            let split_at = if node_chars > hard_max_chars {
                hard_max_chars
            } else {
                max_target_chars
            };
            let sub_chunks = split_large_node(
                path,
                lines,
                boundary.start_line,
                boundary.end_line,
                boundary.kind,
                split_at,
                overlap_chars,
            );
            chunks.extend(sub_chunks);
        }

        covered_end = boundary.end_line;
    }

    // Trailing gap after last AST node.
    let total_lines = u32::try_from(lines.len()).unwrap_or(u32::MAX);
    if covered_end < total_lines {
        let gap_content = extract_lines(lines, covered_end + 1, total_lines);
        if estimate_tokens(&gap_content) > 0 {
            chunks.push(BM25Chunk {
                path: path.to_path_buf(),
                content: gap_content,
                line_start: covered_end + 1,
                line_end: total_lines,
                kind: ChunkKind::LineBlock,
            });
        }
    }

    // Merge small adjacent chunks (below MIN_CHUNK_TOKENS).
    merge_small_chunks(&mut chunks, min_chars);

    // Add overlap between adjacent chunks from the same file.
    add_overlap(&mut chunks, lines, overlap_chars);

    chunks
}

/// Extract lines [start_line..=end_line] (1-indexed) and join with newline.
fn extract_lines(lines: &[&str], start_line: u32, end_line: u32) -> String {
    let start = (start_line as usize).saturating_sub(1);
    let end = (end_line as usize).min(lines.len());
    if start >= end {
        return String::new();
    }
    lines[start..end].join("\n")
}

/// Split a large AST node into sub-chunks at line boundaries.
fn split_large_node(
    path: &Path,
    lines: &[&str],
    start_line: u32,
    end_line: u32,
    kind: ChunkKind,
    max_chars: usize,
    overlap_chars: usize,
) -> Vec<BM25Chunk> {
    let mut sub_chunks = Vec::new();
    let start_idx = (start_line as usize).saturating_sub(1);
    let end_idx = (end_line as usize).min(lines.len());

    let mut chunk_start = start_idx;
    while chunk_start < end_idx {
        let mut chunk_end = chunk_start;
        let mut char_count = 0;

        // Accumulate lines until we hit the max.
        while chunk_end < end_idx {
            let line_len = lines[chunk_end].len() + 1; // +1 for newline
            if char_count + line_len > max_chars && chunk_end > chunk_start {
                break;
            }
            char_count += line_len;
            chunk_end += 1;
        }

        let content = lines[chunk_start..chunk_end].join("\n");
        sub_chunks.push(BM25Chunk {
            path: path.to_path_buf(),
            content,
            line_start: u32::try_from(chunk_start + 1).unwrap_or(u32::MAX),
            line_end: u32::try_from(chunk_end).unwrap_or(u32::MAX),
            kind,
        });

        if chunk_end >= end_idx {
            break;
        }

        // Step back by overlap amount.
        let overlap_lines = estimate_overlap_lines(lines, chunk_end, overlap_chars);
        chunk_start = chunk_end.saturating_sub(overlap_lines);
    }

    sub_chunks
}

/// Estimate how many lines from the end of a range correspond to `overlap_chars`.
fn estimate_overlap_lines(lines: &[&str], end_idx: usize, overlap_chars: usize) -> usize {
    let mut chars = 0;
    let mut count = 0;
    let start = end_idx.saturating_sub(lines.len());
    for i in (start..end_idx).rev() {
        chars += lines[i].len() + 1;
        count += 1;
        if chars >= overlap_chars {
            break;
        }
    }
    count
}

/// Merge adjacent chunks that are below the minimum size threshold.
/// Never merge if the result would exceed the hard max.
fn merge_small_chunks(chunks: &mut Vec<BM25Chunk>, min_chars: usize) {
    let hard_max_chars = HARD_MAX_CHUNK_TOKENS * CHARS_PER_TOKEN;

    if chunks.len() < 2 {
        return;
    }

    let mut i = 0;
    while i + 1 < chunks.len() {
        if chunks[i].content.len() < min_chars || chunks[i + 1].content.len() < min_chars {
            let merged_len = chunks[i].content.len() + 1 + chunks[i + 1].content.len();
            if merged_len > hard_max_chars {
                i += 1;
                continue;
            }
            // Merge chunks[i] and chunks[i+1].
            let merged_content = format!("{}\n{}", chunks[i].content, chunks[i + 1].content);
            // Keep the kind of the larger chunk.
            let kind = if chunks[i].content.len() >= chunks[i + 1].content.len() {
                chunks[i].kind
            } else {
                chunks[i + 1].kind
            };
            chunks[i] = BM25Chunk {
                path: chunks[i].path.clone(),
                content: merged_content,
                line_start: chunks[i].line_start,
                line_end: chunks[i + 1].line_end,
                kind,
            };
            chunks.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

/// Add overlap context: prepend a snippet from the previous chunk's tail
/// to each chunk (except the first).
fn add_overlap(chunks: &mut [BM25Chunk], lines: &[&str], overlap_chars: usize) {
    if chunks.len() < 2 || overlap_chars == 0 {
        return;
    }

    for i in 1..chunks.len() {
        let prev_end_line = chunks[i - 1].line_end;
        let curr_start_line = chunks[i].line_start;

        // Only add overlap if chunks are adjacent or overlapping.
        if curr_start_line > prev_end_line + 1 {
            continue;
        }

        // Take up to overlap_chars from the tail of the previous chunk.
        let prev_start = (chunks[i - 1].line_start as usize).saturating_sub(1);
        let prev_end = (prev_end_line as usize).min(lines.len());
        let mut overlap_start = prev_end;
        let mut chars = 0;
        while overlap_start > prev_start {
            let line_len = lines[overlap_start - 1].len() + 1;
            if chars + line_len > overlap_chars {
                break;
            }
            chars += line_len;
            overlap_start -= 1;
        }

        if overlap_start < prev_end {
            let overlap_text = lines[overlap_start..prev_end].join("\n");
            chunks[i].content = format!("{}\n{}", overlap_text, chunks[i].content);
            chunks[i].line_start = u32::try_from(overlap_start + 1).unwrap_or(u32::MAX);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn p(name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    // -- AST chunking produces Function chunks for Rust code --
    #[test]
    fn ast_chunk_rust_functions() {
        let source = r#"
fn foo() {
    let x = 1;
    let y = 2;
    x + y
}

fn bar(a: i32) -> i32 {
    a * 2
}
"#;
        let lines: Vec<&str> = source.lines().collect();
        let path = p("test.rs");
        let chunks = Bm25Index::try_ast_chunk(&path, source.as_bytes(), &lines);
        // Should produce AST chunks (not empty — would fall back to line blocks).
        assert!(!chunks.is_empty(), "AST chunking should produce chunks for .rs");

        // At least one chunk should be Function kind.
        let fn_chunks: Vec<_> = chunks.iter().filter(|c| c.kind == ChunkKind::Function).collect();
        assert!(!fn_chunks.is_empty(), "should have Function-kind chunks");
    }

    // -- AST chunking produces chunks for Python code --
    #[test]
    fn ast_chunk_python_classes() {
        let source = r#"
class Foo:
    def __init__(self):
        self.x = 1

    def method(self):
        return self.x

def standalone():
    pass
"#;
        let lines: Vec<&str> = source.lines().collect();
        let path = p("test.py");
        let chunks = Bm25Index::try_ast_chunk(&path, source.as_bytes(), &lines);
        assert!(!chunks.is_empty(), "AST chunking should produce chunks for .py");
    }

    // -- Non-code files fall back to empty (caller uses line blocks) --
    #[test]
    fn ast_chunk_fallback_for_txt() {
        let source = "just some text\nnothing special\n";
        let lines: Vec<&str> = source.lines().collect();
        let path = p("readme.txt");
        let chunks = Bm25Index::try_ast_chunk(&path, source.as_bytes(), &lines);
        assert!(chunks.is_empty(), "should return empty for unsupported extensions");
    }

    // -- No chunk exceeds HARD_MAX_CHUNK_TOKENS (deterministic check) --
    #[test]
    fn no_chunk_exceeds_hard_max() {
        // Build a Rust file with a very large function (200 lines).
        let mut source = String::from("fn giant() {\n");
        for i in 0..200 {
            source.push_str(&format!("    let var_{i} = {i};\n"));
        }
        source.push_str("}\n");

        let lines: Vec<&str> = source.lines().collect();
        let path = p("big.rs");
        let chunks = Bm25Index::try_ast_chunk(&path, source.as_bytes(), &lines);

        for chunk in &chunks {
            let tokens = estimate_tokens(&chunk.content);
            assert!(
                tokens <= HARD_MAX_CHUNK_TOKENS,
                "chunk has {tokens} tokens, exceeds hard max {HARD_MAX_CHUNK_TOKENS}. \
                 lines {}-{}",
                chunk.line_start,
                chunk.line_end,
            );
        }
    }

    // -- Chunk boundaries align with fn/struct/impl in Rust --
    #[test]
    fn chunk_boundaries_align_with_ast() {
        let source = r#"use std::path::Path;

struct Config {
    name: String,
    value: i32,
}

impl Config {
    fn new(name: &str, value: i32) -> Self {
        Config {
            name: name.to_string(),
            value,
        }
    }
}

fn helper() -> bool {
    true
}
"#;
        let lines: Vec<&str> = source.lines().collect();
        let path = p("test.rs");
        let chunks = Bm25Index::try_ast_chunk(&path, source.as_bytes(), &lines);

        // Check that we have struct/impl/function kinds.
        let kinds: Vec<ChunkKind> = chunks.iter().map(|c| c.kind).collect();
        // After merging small chunks, the specific kinds may vary,
        // but we should have at least one non-LineBlock chunk.
        let has_ast_kind = kinds.iter().any(|k| {
            matches!(k, ChunkKind::Function | ChunkKind::Struct | ChunkKind::Impl)
        });
        assert!(has_ast_kind, "should have AST-aligned chunk kinds, got: {kinds:?}");
    }

    // -- make_chunks_from_content uses AST for .rs and line blocks for .txt --
    #[test]
    fn make_chunks_dispatches_correctly() {
        let rs_source = "fn main() {\n    println!(\"hello\");\n}\n";
        let rs_lines: Vec<&str> = rs_source.lines().collect();
        let rs_chunks = Bm25Index::make_chunks_from_content(
            &p("main.rs"),
            &rs_lines,
            rs_source,
        );

        // Should have at least one File chunk (zero-orphan) and AST chunks.
        let file_chunks: Vec<_> = rs_chunks.iter().filter(|c| c.kind == ChunkKind::File).collect();
        assert!(!file_chunks.is_empty(), "should have File chunk");

        let txt_source = "line 1\nline 2\nline 3\n";
        let txt_lines: Vec<&str> = txt_source.lines().collect();
        let txt_chunks = Bm25Index::make_chunks_from_content(
            &p("notes.txt"),
            &txt_lines,
            txt_source,
        );
        let lb_chunks: Vec<_> = txt_chunks.iter().filter(|c| c.kind == ChunkKind::LineBlock).collect();
        assert!(!lb_chunks.is_empty(), "txt should use LineBlock chunks");
    }

    // -- ChunkKind round-trip through as_str/from_str --
    #[test]
    fn chunk_kind_round_trip() {
        let kinds = [
            ChunkKind::Function, ChunkKind::Struct, ChunkKind::Enum,
            ChunkKind::Impl, ChunkKind::Class, ChunkKind::Module,
            ChunkKind::Trait, ChunkKind::LineBlock, ChunkKind::File,
        ];
        for kind in &kinds {
            let s = kind.as_str();
            let round = ChunkKind::from_str(s);
            assert_eq!(*kind, round, "round-trip failed for {s}");
        }
    }

    // -- estimate_tokens basic sanity --
    #[test]
    fn estimate_tokens_sanity() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
        // 100 chars -> 25 tokens
        let s: String = "a".repeat(100);
        assert_eq!(estimate_tokens(&s), 25);
    }

    // -- Overlap exists between adjacent chunks from split large node --
    #[test]
    fn overlap_between_split_chunks() {
        // Build a single giant function that will be split into sub-chunks.
        let mut source = String::from("fn giant() {\n");
        for i in 0..200 {
            source.push_str(&format!("    let var_{i} = {i}; // padding text\n"));
        }
        source.push_str("}\n");

        let lines: Vec<&str> = source.lines().collect();
        let path = p("overlap.rs");
        let chunks = Bm25Index::try_ast_chunk(&path, source.as_bytes(), &lines);

        // Should be split into multiple sub-chunks.
        assert!(chunks.len() >= 2, "large fn should be split, got {} chunks", chunks.len());

        // Check overlap: second chunk's start should be before first chunk's end.
        for i in 1..chunks.len() {
            let prev_end = chunks[i - 1].line_end;
            let curr_start = chunks[i].line_start;
            assert!(
                curr_start <= prev_end,
                "split chunk {i} should overlap with previous: prev ends at {prev_end}, curr starts at {curr_start}"
            );
        }
    }

    // -- Files that fail AST parse fall back to line blocks --
    #[test]
    fn ast_parse_failure_falls_back() {
        let invalid_rust = "fn {{{{{ invalid syntax >>>>>";
        let lines: Vec<&str> = invalid_rust.lines().collect();
        let path = p("broken.rs");
        // tree-sitter is error-tolerant and will still produce a tree,
        // but if it extracts no nodes, we fall back.
        let chunks = Bm25Index::try_ast_chunk(&path, invalid_rust.as_bytes(), &lines);
        // Even if tree-sitter parses something, the result should not crash.
        // The test passes if no panic occurs.
        let _ = chunks;
    }

    // -- Tantivy integration: index + query returns chunk_kind metadata --
    #[test]
    fn tantivy_query_returns_chunk_kind() {
        let dir = tempfile::tempdir().unwrap();
        let mut idx = Bm25Index::new(dir.path(), "test-proj").unwrap();

        let chunks = vec![
            BM25Chunk {
                path: p("main.rs"),
                content: "fn hello_world() { println!(\"hi\"); }".to_string(),
                line_start: 1,
                line_end: 1,
                kind: ChunkKind::Function,
            },
            BM25Chunk {
                path: p("main.rs"),
                content: "struct Config { name: String }".to_string(),
                line_start: 3,
                line_end: 3,
                kind: ChunkKind::Struct,
            },
        ];

        let count = idx.index_file(&p("main.rs"), Some(chunks)).unwrap();
        assert_eq!(count, 2);

        let hits = idx.query("hello_world", 10).unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].chunk_kind, ChunkKind::Function);

        let hits2 = idx.query("Config", 10).unwrap();
        assert!(!hits2.is_empty());
        assert_eq!(hits2[0].chunk_kind, ChunkKind::Struct);
    }

    // -- Index version bumped to v2 --
    #[test]
    fn index_version_is_v2() {
        assert_eq!(
            crate::index_version::CURRENT_INDEX_VERSION,
            2,
            "index version should be 2 for AST chunking"
        );
    }
}
