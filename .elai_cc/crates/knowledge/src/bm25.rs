use std::path::{Path, PathBuf};

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, Term};

/// Index format version for tantivy 0.26.0 (verified from tantivy src/lib.rs).
pub const INDEX_FORMAT_VERSION: u32 = 7;

/// Heap budget for the IndexWriter (50 MB).
const WRITER_HEAP_BYTES: usize = 50_000_000;

/// Default sliding window size for line-block chunking.
const CHUNK_WINDOW_LINES: usize = 50;

/// Overlap between consecutive line-block chunks.
const CHUNK_OVERLAP_LINES: usize = 10;

/// Bytes to inspect for binary detection (null byte check).
const BINARY_DETECT_BYTES: usize = 8192;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChunkKind {
    Function,
    LineBlock,
    File,
}

impl ChunkKind {
    fn as_str(&self) -> &'static str {
        match self {
            ChunkKind::Function => "function",
            ChunkKind::LineBlock => "line_block",
            ChunkKind::File => "file",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "function" => ChunkKind::Function,
            "line_block" => ChunkKind::LineBlock,
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
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
                return Err(format!(
                    "index schema mismatch: missing field '{name}' — delete index at {} and rebuild",
                    index_path.display()
                )
                .into());
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
    ) -> Result<usize, Box<dyn std::error::Error>> {
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
    ) -> Result<IndexStats, Box<dyn std::error::Error>> {
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
    ) -> Result<Vec<BM25Hit>, Box<dyn std::error::Error>> {
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
            let line_start = doc
                .get_first(self.fields.line_start)
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            let line_end = doc
                .get_first(self.fields.line_end)
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
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
    ) -> Result<usize, Box<dyn std::error::Error>> {
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
    ) -> Result<usize, Box<dyn std::error::Error>> {
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

    fn chunk_file(&self, path: &Path) -> Result<Vec<BM25Chunk>, Box<dyn std::error::Error>> {
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

        let mut chunks = Vec::new();

        // Sliding window: 50 lines, 10-line overlap
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
                kind: ChunkKind::LineBlock,
            });
            if end == lines.len() {
                break;
            }
            start += step;
        }

        // Zero-orphan invariant: every non-binary file gets at least one File-kind chunk
        let line_count = lines.len() as u32;
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
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                    // Tracing/counter deferred to Phase 1B.5.
                    Err(_) => continue,
                };
                if Self::is_binary(&data) {
                    stats.files_skipped_binary += 1;
                    continue;
                }
                let content = String::from_utf8_lossy(&data);
                let lines: Vec<&str> = content.lines().collect();
                let chunks = self.make_chunks_from_lines(&path, &lines, &content);
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

    fn make_chunks_from_lines(
        &self,
        path: &Path,
        lines: &[&str],
        full_content: &str,
    ) -> Vec<BM25Chunk> {
        let mut chunks = Vec::new();

        if lines.is_empty() {
            chunks.push(BM25Chunk {
                path: path.to_path_buf(),
                content: String::new(),
                line_start: 1,
                line_end: 1,
                kind: ChunkKind::File,
            });
            return chunks;
        }

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
                kind: ChunkKind::LineBlock,
            });
            if end == lines.len() {
                break;
            }
            start += step;
        }

        // Zero-orphan invariant
        chunks.push(BM25Chunk {
            path: path.to_path_buf(),
            content: full_content.to_string(),
            line_start: 1,
            line_end: lines.len() as u32,
            kind: ChunkKind::File,
        });

        chunks
    }
}
