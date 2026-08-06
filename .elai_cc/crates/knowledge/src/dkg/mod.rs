pub mod observed_edges;
pub use observed_edges::{EdgeProvenance, ObservedCallEdge, ObservedEdgeIngestStats, EDGE_OBSERVED_CALLS};

pub mod language_registry;
pub use language_registry::{LanguageRegistry, LanguageRegistryStatus, GrammarLoadError};

pub mod resolution;
pub use resolution::{ImportResolver, JvmResolver, CsNamespaceResolver, GoModuleResolver, TsPathAliasResolver};

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;
use rusqlite::Connection;

use crate::{
    DataFlowEdge, DkgNode, DkgNodeInfo, DkgReadiness, DependencyGraph, FlowType, KnowledgeError,
    LanguageGrammar, SymbolKind,
};

type EdgeList = Vec<(i64, i64, String)>;

// ---------------------------------------------------------------------------
// Schema SQL
// ---------------------------------------------------------------------------

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS dkg_nodes (
    id         INTEGER PRIMARY KEY,
    file_path  TEXT    NOT NULL,
    name       TEXT    NOT NULL,
    kind       TEXT    NOT NULL,
    signature  TEXT    DEFAULT '',
    start_line INTEGER NOT NULL,
    end_line   INTEGER NOT NULL,
    parent_id  INTEGER REFERENCES dkg_nodes ON DELETE CASCADE,
    concept       TEXT DEFAULT NULL,
    concept_stale INTEGER DEFAULT 1,
    file_hash  BLOB
);

CREATE TABLE IF NOT EXISTS dkg_edges (
    source_id  INTEGER NOT NULL REFERENCES dkg_nodes ON DELETE CASCADE,
    target_id  INTEGER NOT NULL REFERENCES dkg_nodes ON DELETE CASCADE,
    edge_type  TEXT    NOT NULL,
    PRIMARY KEY (source_id, target_id, edge_type)
);

CREATE TABLE IF NOT EXISTS dkg_files (
    file_path    TEXT PRIMARY KEY,
    file_hash    BLOB,
    parse_status TEXT DEFAULT 'pending',
    parse_error  TEXT DEFAULT NULL,
    last_indexed INTEGER
);

CREATE TABLE IF NOT EXISTS dkg_meta (
    key   TEXT PRIMARY KEY,
    value TEXT
);

CREATE TABLE IF NOT EXISTS dkg_data_flow (
    id             INTEGER PRIMARY KEY,
    source_node_id INTEGER NOT NULL REFERENCES dkg_nodes ON DELETE CASCADE,
    target_node_id INTEGER NOT NULL REFERENCES dkg_nodes ON DELETE CASCADE,
    variable_name  TEXT    NOT NULL,
    flow_type      TEXT    NOT NULL CHECK(flow_type IN
                   ('PARAMETER','RETURN','ASSIGNMENT','FIELD_ACCESS')),
    UNIQUE(source_node_id, target_node_id, variable_name, flow_type)
);

CREATE INDEX IF NOT EXISTS idx_nodes_name   ON dkg_nodes(name);
CREATE INDEX IF NOT EXISTS idx_nodes_file   ON dkg_nodes(file_path);
CREATE INDEX IF NOT EXISTS idx_nodes_kind   ON dkg_nodes(kind);
CREATE INDEX IF NOT EXISTS idx_files_status ON dkg_files(parse_status);
CREATE INDEX IF NOT EXISTS idx_data_flow_source ON dkg_data_flow(source_node_id);
CREATE INDEX IF NOT EXISTS idx_data_flow_target ON dkg_data_flow(target_node_id);
CREATE TABLE IF NOT EXISTS dkg_observed_edge_provenance (
    source_id          INTEGER NOT NULL REFERENCES dkg_nodes ON DELETE CASCADE,
    target_id          INTEGER NOT NULL REFERENCES dkg_nodes ON DELETE CASCADE,
    provenance_source  TEXT    NOT NULL,
    source_seq         INTEGER,
    observed_at        INTEGER NOT NULL,
    PRIMARY KEY (source_id, target_id, provenance_source, source_seq)
);
CREATE INDEX IF NOT EXISTS idx_observed_prov_source ON dkg_observed_edge_provenance(source_id);
CREATE INDEX IF NOT EXISTS idx_observed_prov_target ON dkg_observed_edge_provenance(target_id);
";

// ---------------------------------------------------------------------------
// Schema SQL v3 — B-29 Phase 1: symbol graph tables
// ---------------------------------------------------------------------------
// v2 was reserved per B-29 plan terminology; schema jumps 1->3.
// Column names use *_msgpack convention per spec contract; blobs are
// encoded/decoded via postcard (not rmp-serde — postcard is already a
// workspace dep, keeping zero new external dependencies).
// All statements are idempotent (CREATE TABLE IF NOT EXISTS).

const SCHEMA_SQL_V3: &str = "
CREATE TABLE IF NOT EXISTS symbol_name_index (
    shard_key    TEXT    NOT NULL,
    symbol_name  TEXT    NOT NULL,
    refs_msgpack BLOB    NOT NULL,
    PRIMARY KEY (shard_key, symbol_name)
);

CREATE TABLE IF NOT EXISTS symbol_reverse_index (
    shard_bucket         INTEGER NOT NULL,
    callee_file          TEXT    NOT NULL,
    caller_files_msgpack BLOB    NOT NULL,
    PRIMARY KEY (shard_bucket, callee_file)
);

CREATE TABLE IF NOT EXISTS symbol_file_payload (
    file             TEXT PRIMARY KEY,
    language         TEXT NOT NULL,
    content_hash     TEXT NOT NULL,
    symbols_msgpack  BLOB NOT NULL,
    edges_msgpack    BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS symbol_graph_meta (
    project_id          TEXT PRIMARY KEY,
    symbol_count        INTEGER NOT NULL,
    edge_count          INTEGER NOT NULL,
    file_count          INTEGER NOT NULL,
    unresolved_edge_pct REAL    NOT NULL,
    built_at            INTEGER NOT NULL,
    schema_version      INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_symbol_name_shard    ON symbol_name_index(shard_key);
CREATE INDEX IF NOT EXISTS idx_symbol_reverse_shard ON symbol_reverse_index(shard_bucket);
";

/// H-10: Combined schema SQL exposed for integration tests that need to
/// construct a minimal dkg.db in-process without going through DkgEngine::new.
/// Only available under the `test-helpers` feature gate.
#[cfg(feature = "test-helpers")]
pub const SCHEMA_SQL_FOR_TEST: &str = SCHEMA_SQL;

// ---------------------------------------------------------------------------
// Schema v4 migration — adds `language` column to dkg_nodes
// ---------------------------------------------------------------------------

/// Apply the v4 schema migration idempotently.
///
/// SQLite's `ALTER TABLE ADD COLUMN` is not idempotent — running it twice raises
/// "duplicate column name". We guard with a `pragma_table_info` check first.
fn apply_v4_migration(conn: &Connection) -> Result<(), KnowledgeError> {
    let col_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('dkg_nodes') WHERE name='language'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .unwrap_or(false);

    if !col_exists {
        conn.execute_batch("ALTER TABLE dkg_nodes ADD COLUMN language TEXT DEFAULT NULL;")?;
    }

    conn.execute(
        "INSERT OR REPLACE INTO dkg_meta (key, value) VALUES ('schema_version', '4')",
        [],
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Edge-type constants
// ---------------------------------------------------------------------------

pub const EDGE_IMPORTS: &str = "imports";
pub const EDGE_CONTAINS: &str = "contains";
pub const EDGE_IMPLEMENTS: &str = "implements";
/// F-5 / Slice A4: non-destructive marker edge inserted by
/// [`DkgEngine::apply_staleness_broadcast`] on a quarantined concept's node.
/// `dkg_edges` has no temporal column, so the bi-temporal "close" is an INSERT
/// of this self-loop marker edge (Graphiti / Mem0 non-destructive
/// invalidation), never an UPDATE.
pub const EDGE_STALENESS_QUARANTINED: &str = "staleness_quarantined";

/// F-5 / Slice A4: `concept_stale` value written by the staleness-quarantine
/// cascade. Semantics: 0 = explicitly fresh, 1 = default-unknown (the DDL
/// default + the value the sole node INSERT hardcodes), 2 = quarantined. The
/// cascade only ever WRITES 2; it never disturbs legacy 1s.
pub const CONCEPT_STALE_QUARANTINED: i64 = 2;

// Data flow edge type strings (match CHECK constraint in dkg_data_flow)
pub const FLOW_PARAMETER: &str = "PARAMETER";
pub const FLOW_RETURN: &str = "RETURN";
pub const FLOW_ASSIGNMENT: &str = "ASSIGNMENT";
pub const FLOW_FIELD_ACCESS: &str = "FIELD_ACCESS";

// ---------------------------------------------------------------------------
// F-5 / Slice A4 — staleness-quarantine cascade (knowledge side)
// ---------------------------------------------------------------------------

/// F-5 / Slice A4: a knowledge-side view of one memory `QuarantineRecord`.
///
/// `crates/knowledge` has no dependency on `crates/memory`, so the CLI bridge
/// (which depends on both) projects each drained `QuarantineRecord` into this
/// local view before calling [`DkgEngine::apply_staleness_broadcast`]. Only the
/// two fields the DKG cascade needs are carried: the `fact_id` (joined against
/// `dkg_nodes.concept`) and the `staleness_ratio` (carried for SSB / logging).
#[derive(Debug, Clone, PartialEq)]
pub struct QuarantineRecordView {
    /// Quarantined fact's stable id. Matched EXACTLY against
    /// `dkg_nodes.concept`.
    pub fact_id: String,
    /// Decay staleness ratio at quarantine time (carried through for
    /// observability; not used in the match itself).
    pub staleness_ratio: f64,
}

impl QuarantineRecordView {
    /// Construct a view from its two fields.
    pub fn new(fact_id: impl Into<String>, staleness_ratio: f64) -> Self {
        Self {
            fact_id: fact_id.into(),
            staleness_ratio,
        }
    }
}

/// F-5 / Slice A4: aggregate result of one
/// [`DkgEngine::apply_staleness_broadcast`] batch.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StalenessApplyStats {
    /// Total DKG concept rows demoted to `concept_stale = 2` across the batch.
    pub dependents_demoted: u64,
    /// Total non-destructive `staleness_quarantined` marker edges inserted
    /// across the batch (already-present markers are `INSERT OR IGNORE`d and
    /// not counted).
    pub marker_edges_inserted: u64,
}

// ---------------------------------------------------------------------------
// DkgEngine
// ---------------------------------------------------------------------------

pub struct DkgEngine {
    conn: Connection,
    registry: LanguageRegistry,
    build_graph_called: bool,
    index_version_path: Option<PathBuf>,
    created_at: i64,
    /// E-42: knowledge-side SSB sink for tree-sitter grammar health events.
    /// Wire-dark by default (`noop_sink()`). Set via
    /// [`DkgEngine::new_with_sink`].
    sink: std::sync::Arc<dyn crate::knowledge_ssb_sink::KnowledgeSsbSink>,
}

impl DkgEngine {
    /// Construct a `DkgEngine` with the default no-op SSB sink. All existing
    /// callers that do not need tree-sitter health telemetry call this variant.
    pub fn new(
        db_path: &Path,
        index_version_path: Option<&Path>,
        created_at: i64,
    ) -> Result<Self, KnowledgeError> {
        Self::new_with_sink(
            db_path,
            index_version_path,
            created_at,
            crate::knowledge_ssb_sink::noop_sink(),
        )
    }

    /// E-42: construct a `DkgEngine` with a live knowledge SSB sink so
    /// `build_graph` can emit `tree_sitter_grammar_event` rows when
    /// `ELAI_TREE_SITTER_GRAMMARS_ENABLE=1`. The `dkg_lifecycle::run_rebuild`
    /// seam passes the real `OrchestratorKnowledgeSsbSink` here; test code
    /// passes a recording sink or the default noop.
    pub fn new_with_sink(
        db_path: &Path,
        index_version_path: Option<&Path>,
        created_at: i64,
        sink: std::sync::Arc<dyn crate::knowledge_ssb_sink::KnowledgeSsbSink>,
    ) -> Result<Self, KnowledgeError> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(SCHEMA_SQL)?;
        // B-29 Phase 1: apply v3 tables (idempotent; safe to run on every open).
        // v2 was reserved per plan terminology; this jumps 1->3.
        conn.execute_batch(SCHEMA_SQL_V3)?;

        // v2 reserved; v3 added symbol graph tables (B-29); v4 adds `language` column
        // to dkg_nodes. apply_v4_migration is idempotent and bumps schema_version to '4'.
        apply_v4_migration(&conn)?;
        conn.execute(
            "INSERT OR IGNORE INTO dkg_meta (key, value) VALUES ('created_at', ?1)",
            [created_at.to_string()],
        )?;

        if let Some(ivp) = index_version_path {
            let version_file = crate::index_version::current_version(created_at);
            crate::index_version::write(ivp, &version_file)
                .map_err(|e| KnowledgeError::Graph(format!("index_version write: {e}")))?;
        }

        let mut registry = LanguageRegistry::new();
        // Reuse the canonical Tier-A registration helper instead of duplicating
        // the 18-grammar `try_register` block inline. The helper covers every
        // TIER_A_GRAMMARS entry plus Vue (composite-reparse special case) and
        // is the single source of truth consumed by audits / dashboard surfaces.
        // Per-grammar failures are recorded inside the helper; we mirror the
        // previous `let _ = …` swallow by ignoring the per-language outcome here.
        let _ = crate::tree_sitter_health::registration::register_tier_a_languages(&mut registry);
        // Tier-B (scala/swift/php/bash) is registered flat for backwards
        // compatibility with the pre-tier split; the experimental_grammars
        // feature gate inside the helper keeps it a no-op unless opted in.
        let tier_b_outcome =
            crate::tree_sitter_health::registration::register_tier_b_languages(&mut registry);

        let status = registry.status();
        tracing::info!(
            loaded = ?status.loaded,
            failed = ?status.failed,
            "LanguageRegistry initialized: {}/{} grammars loaded",
            status.loaded.len(),
            status.loaded.len() + status.failed.len()
        );

        if crate::tree_sitter_health::is_enabled() {
            use crate::knowledge_ssb_sink::{TreeSitterGrammarEvent, TreeSitterGrammarEventKind};
            for lang_id in &tier_b_outcome.registered {
                sink.emit_tree_sitter_grammar_event(TreeSitterGrammarEvent {
                    kind: TreeSitterGrammarEventKind::TierBGrammarLoaded,
                    language_id: Some(lang_id.clone()),
                    detail: Some("may emit ERROR nodes".to_owned()),
                });
            }
        }

        Ok(Self {
            conn,
            registry,
            build_graph_called: false,
            index_version_path: index_version_path.map(|p| p.to_path_buf()),
            created_at,
            sink,
        })
    }

    pub fn build_graph(&mut self, project_root: &Path) -> Result<DependencyGraph, KnowledgeError> {
        dkg_phase_marker("walk_start");
        let mut source_files = Vec::new();
        walk_source_files(project_root, &mut source_files);
        source_files.sort();
        dkg_phase_marker("parse_loop_start");

        // E-42: ABI probe — one pass over the registry before the source loop.
        // Guarded by the ELAI_TREE_SITTER_GRAMMARS_ENABLE env var so this is
        // wire-dark by default. Each mismatch emits one SSB row.
        if crate::tree_sitter_health::is_enabled() {
            use crate::knowledge_ssb_sink::{TreeSitterGrammarEvent, TreeSitterGrammarEventKind};
            use crate::tree_sitter_health::abi_probe::probe_abi_versions;
            let records = probe_abi_versions(self.registry.iter());
            for rec in &records {
                if !rec.in_range {
                    self.sink.emit_tree_sitter_grammar_event(TreeSitterGrammarEvent {
                        kind: TreeSitterGrammarEventKind::AbiProbeMismatch,
                        language_id: Some(rec.abi.language_id.clone()),
                        detail: Some(format!("version={}", rec.abi.version)),
                    });
                }
            }
        }

        let mut files_parsed: usize = 0;
        let mut total_node_count: usize = 0;

        let mut parser = tree_sitter::Parser::new();

        for file_path in &source_files {
            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let lang_id = match ext_to_lang_id(ext) {
                Some(id) => id,
                None => { continue; }
            };
            let grammar = match self.registry.get(lang_id) {
                Some(g) => g,
                None => { continue; }
            };
            // Read source bytes before set_language so we can run the large-file
            // guard (E-42) without entering the parser.
            let source_bytes = std::fs::read(file_path)?;
            let file_path_str = file_path.to_string_lossy().to_string();

            // E-42: large-file guard. Skip files above 2 MiB for tree-sitter;
            // they will be handled as raw text by the chunking pipeline.
            // Guarded by ELAI_TREE_SITTER_GRAMMARS_ENABLE for SSB emission only;
            // the skip itself is always enforced.
            {
                use crate::tree_sitter_health::large_file::{
                    should_skip_for_size, TREE_SITTER_LARGE_FILE_BYTES,
                };
                let byte_count = source_bytes.len() as u64;
                if should_skip_for_size(byte_count) {
                    if crate::tree_sitter_health::is_enabled() {
                        use crate::knowledge_ssb_sink::{
                            TreeSitterGrammarEvent, TreeSitterGrammarEventKind,
                        };
                        self.sink.emit_tree_sitter_grammar_event(TreeSitterGrammarEvent {
                            kind: TreeSitterGrammarEventKind::TreeSitterSkippedLargeFile,
                            language_id: None,
                            detail: Some(format!(
                                "path={file_path_str} bytes={byte_count} threshold={}",
                                TREE_SITTER_LARGE_FILE_BYTES
                            )),
                        });
                    }
                    files_parsed += 1;
                    continue;
                }
            }
            parser
                .set_language(&grammar.tree_sitter_language())
                .map_err(|e| KnowledgeError::Graph(format!("set_language failed: {e}")))?;

            // Compute blake3 hash
            let hash = blake3::hash(&source_bytes);
            let hash_bytes = hash.as_bytes().as_slice();

            // Check for incremental skip
            let existing_hash: Option<Vec<u8>> = self.conn.query_row(
                "SELECT file_hash FROM dkg_files WHERE file_path = ?1",
                [&file_path_str],
                |row| row.get(0),
            ).ok();

            if let Some(ref old_hash) = existing_hash {
                if old_hash.as_slice() == hash_bytes {
                    files_parsed += 1;
                    // Hash unchanged, skip re-parse
                    let count: i64 = self.conn.query_row(
                        "SELECT COUNT(*) FROM dkg_nodes WHERE file_path = ?1",
                        [&file_path_str],
                        |row| row.get(0),
                    )?;
                    total_node_count += usize::try_from(count).unwrap_or(0);
                    continue;
                }
                // Hash changed: delete old nodes (edges cascade)
                self.conn.execute(
                    "DELETE FROM dkg_nodes WHERE file_path = ?1",
                    [&file_path_str],
                )?;
                self.conn.execute(
                    "DELETE FROM dkg_files WHERE file_path = ?1",
                    [&file_path_str],
                )?;
            }

            // Binary file detection
            if std::str::from_utf8(&source_bytes).is_err() {
                tracing::warn!("binary file detected, skipping parse: {}", file_path_str);
                self.conn.execute(
                    "INSERT OR REPLACE INTO dkg_files (file_path, file_hash, parse_status, parse_error, last_indexed) \
                     VALUES (?1, ?2, 'failed', 'binary file', ?3)",
                    rusqlite::params![file_path_str, hash_bytes, self.created_at],
                )?;
                files_parsed += 1;
                continue;
            }

            // Parse with tree-sitter
            let tree = match parser.parse(&source_bytes, None) {
                Some(t) => t,
                None => {
                    tracing::warn!("tree-sitter returned None for: {}", file_path_str);
                    self.conn.execute(
                        "INSERT OR REPLACE INTO dkg_files (file_path, file_hash, parse_status, parse_error, last_indexed) \
                         VALUES (?1, ?2, 'failed', 'parse returned None', ?3)",
                        rusqlite::params![file_path_str, hash_bytes, self.created_at],
                    )?;
                    files_parsed += 1;
                    continue;
                }
            };

            // Check for syntax errors
            if tree.root_node().has_error() {
                tracing::warn!("syntax errors in source: {}", file_path_str);
                self.conn.execute(
                    "INSERT OR REPLACE INTO dkg_files (file_path, file_hash, parse_status, parse_error, last_indexed) \
                     VALUES (?1, ?2, 'failed', 'syntax errors in source', ?3)",
                    rusqlite::params![file_path_str, hash_bytes, self.created_at],
                )?;
                files_parsed += 1;
                continue;
            }

            // E-42: Svelte runes parse health — emit one SSB row per .svelte file
            // carrying the ERROR / MISSING node ratio and runes detection flag.
            // Gated by ELAI_TREE_SITTER_GRAMMARS_ENABLE.
            if lang_id == "svelte" && crate::tree_sitter_health::is_enabled() {
                use crate::knowledge_ssb_sink::{TreeSitterGrammarEvent, TreeSitterGrammarEventKind};
                use crate::tree_sitter_health::svelte_health::{compute_health, detect_runes};
                if let Ok(source_str) = std::str::from_utf8(&source_bytes) {
                    let runes = detect_runes(source_str);
                    let health = compute_health(&tree);
                    self.sink.emit_tree_sitter_grammar_event(TreeSitterGrammarEvent {
                        kind: TreeSitterGrammarEventKind::SvelteRunesParseHealth,
                        language_id: Some("svelte".to_owned()),
                        detail: Some(format!(
                            "runes={runes} error={} missing={} total={} ratio={:.4}",
                            health.error, health.missing, health.total, health.ratio()
                        )),
                    });
                }
            }

            // Extract nodes
            let nodes = grammar.extract_nodes(&tree, &source_bytes);

            // Insert dkg_files row
            self.conn.execute(
                "INSERT OR REPLACE INTO dkg_files (file_path, file_hash, parse_status, parse_error, last_indexed) \
                 VALUES (?1, ?2, 'ok', NULL, ?3)",
                rusqlite::params![file_path_str, hash_bytes, self.created_at],
            )?;

            // Insert dkg_nodes, tracking inserted IDs for edge generation
            let mut node_db_ids: Vec<i64> = Vec::with_capacity(nodes.len());
            for node in &nodes {
                let kind_str = symbol_kind_to_str(node.kind);
                self.conn.execute(
                    "INSERT INTO dkg_nodes \
                     (file_path, name, kind, signature, start_line, end_line, concept_stale, language) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)",
                    rusqlite::params![
                        file_path_str,
                        node.name,
                        kind_str,
                        node.signature,
                        node.start_line,
                        node.end_line,
                        lang_id,
                    ],
                )?;
                node_db_ids.push(self.conn.last_insert_rowid());
            }

            // Generate "contains" edges: parent -> child
            for (idx, node) in nodes.iter().enumerate() {
                if let Some(parent_idx) = node.parent_idx {
                    if parent_idx < node_db_ids.len() {
                        let parent_db_id = node_db_ids[parent_idx];
                        let child_db_id = node_db_ids[idx];
                        self.conn.execute(
                            "INSERT OR IGNORE INTO dkg_edges (source_id, target_id, edge_type) VALUES (?1, ?2, ?3)",
                            rusqlite::params![parent_db_id, child_db_id, EDGE_CONTAINS],
                        )?;
                    }
                }
            }

            total_node_count += nodes.len();
            files_parsed += 1;
        }

        dkg_phase_marker("parse_loop_end");

        // B-30 Pass C: resolve non-Rust module imports to cross-file edges.
        self.resolve_multi_module_imports(project_root)?;
        dkg_phase_marker("resolve_imports_end");

        // Generate cross-file edges: "imports" and "implements"
        self.generate_cross_edges()?;
        dkg_phase_marker("cross_edges_end");

        // H-10: materialise the dkg_nodes_pagerank table so get_pagerank_top
        // returns non-empty results. Runs after generate_cross_edges so the
        // full import topology is available to the PageRank pass. Failures are
        // logged and do NOT abort the build (the dashboard degrades gracefully
        // via the existing IF-EXISTS probe in ssb_viewer::pagerank_top).
        if let Err(e) = self.materialise_pagerank() {
            tracing::warn!("H-10 pagerank materialisation failed (non-fatal): {e}");
        }
        dkg_phase_marker("pagerank_end");

        // A-48: Extract heuristic data flow edges from tree-sitter ASTs
        self.extract_all_data_flow_edges(&source_files)?;
        dkg_phase_marker("data_flow_end");

        let edge_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM dkg_edges",
            [],
            |row| row.get(0),
        )?;

        let data_flow_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM dkg_data_flow",
            [],
            |row| row.get(0),
        )?;

        self.build_graph_called = true;

        // Write index_version.toml if path is set
        if let Some(ref ivp) = self.index_version_path {
            let version_file = crate::index_version::current_version(self.created_at);
            crate::index_version::write(ivp, &version_file)
                .map_err(|e| KnowledgeError::Graph(format!("index_version write: {e}")))?;
        }

        Ok(DependencyGraph {
            node_count: total_node_count,
            edge_count: usize::try_from(edge_count).unwrap_or(0),
            files_parsed,
            data_flow_count: usize::try_from(data_flow_count).unwrap_or(0),
        })
    }

    // H-10: materialise the dkg_nodes_pagerank table.
    //
    // The table schema matches what SsbViewer::pagerank_top expects:
    //   CREATE TABLE dkg_nodes_pagerank (symbol TEXT, path TEXT, score REAL)
    //
    // We run a lightweight iterative PageRank over the DkgSnapshot (which
    // has only contains/imports/implements edges — no call edges).  The
    // resulting scores rank files by structural centrality in the import
    // graph, which is a useful approximation until the full call-weighted
    // variant (SymbolGraphCache join) is wired in H-11+.
    //
    // Performance: for a ~399K-node DKG, building the DkgSnapshot is
    // memory-heavy (all raw_nodes loaded into RAM). We guard with a node
    // count cap (MAX_PAGERANK_NODES) and log a warning if exceeded — the
    // dashboard degrades gracefully via the IF-EXISTS probe.
    fn materialise_pagerank(&self) -> Result<(), KnowledgeError> {
        use crate::pagerank::{DkgSnapshot, PageRankConfig};

        /// Maximum number of DKG nodes for which we run full symbol-level PageRank.
        /// Above this threshold the symbol graph is too large to rank in-process;
        /// we skip writing uniform scores (which carry no ranking signal) and leave
        /// the table empty. The canvas detects an empty table via `get_pagerank_top`
        /// returning `known: false` and renders without hub highlighting — honest
        /// degradation per GD-2 (every emitted number must be a real metric).
        /// The 50 000 bound was measured as safe on the smallest target configs
        /// (≤8 GB RAM); the ELAI self-index (398 741 nodes) always exceeds it.
        const MAX_PAGERANK_NODES: usize = 50_000;

        // Index-time memory guard (giant-repo OOM fix): DkgSnapshot::from_engine
        // loads EVERY node (with cloned name/path/signature strings) + all edges
        // into RAM. On a large self-index (~399K nodes) that is a multi-GB
        // allocation that happens BEFORE the MAX_PAGERANK_NODES cap below could
        // skip it — the index-time OOM. Count cheaply first and, when over the
        // cap, skip the materialisation entirely (identical empty-table outcome
        // to the over-cap branch further down, but without ever loading the graph).
        let pre_node_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM dkg_nodes", [], |row| row.get(0))?;
        if usize::try_from(pre_node_count).unwrap_or(usize::MAX) > MAX_PAGERANK_NODES {
            tracing::warn!(
                node_count = pre_node_count,
                cap = MAX_PAGERANK_NODES,
                "H-10 pagerank: graph exceeds cap; skipping snapshot materialisation \
                 (index-time memory guard). PageRank unavailable for this index."
            );
            self.conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS dkg_nodes_pagerank (
                    symbol TEXT NOT NULL,
                    path   TEXT NOT NULL,
                    score  REAL NOT NULL
                );
                DELETE FROM dkg_nodes_pagerank;",
            )?;
            return Ok(());
        }

        let snapshot = DkgSnapshot::from_engine(self)?;

        if snapshot.nodes.is_empty() {
            return Ok(());
        }

        // Create the table if it does not already exist.
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS dkg_nodes_pagerank (
                symbol TEXT NOT NULL,
                path   TEXT NOT NULL,
                score  REAL NOT NULL
            );
            DELETE FROM dkg_nodes_pagerank;",
        )?;

        let n = snapshot.nodes.len();

        // For graphs exceeding the cap: leave the table empty so the canvas
        // degrades to "PageRank unavailable" rather than displaying meaningless
        // uniform scores. Surface the degradation state via a tracing::warn
        // that the dashboard log panel can surface.
        if n > MAX_PAGERANK_NODES {
            tracing::warn!(
                node_count = n,
                cap = MAX_PAGERANK_NODES,
                "H-10 pagerank: graph exceeds cap; PageRank unavailable for this index. \
                 Canvas hub highlighting disabled. Re-index on a file-level snapshot \
                 (H-11+) to enable PageRank at this scale."
            );
            // Table exists but is empty — get_pagerank_top returns known:false,
            // canvas renders with pageRankKnown=false (no COLOR_HUB applied).
            return Ok(());
        }

        // Compute scores via power-iteration PageRank.
        let scores: Vec<(String, String, f64)> = {
            // Run PageRank with default config, no task-personalization.
            let config = PageRankConfig::default();
            // We need a simple power-iteration here; reuse the existing
            // internal infrastructure via compute_repo_map with an empty
            // task string. The ranked output gives us scores per node.
            let repo_map = crate::pagerank::compute_repo_map(
                &snapshot,
                "",             // no task string — global rank
                &[],            // no active files pinned
                &config,
                &|s: &str| s.len(), // trivial token estimator; result is only scores
            );

            // compute_repo_map returns ranked_symbols sorted descending.
            // We need to re-derive the per-node score from the rank field.
            // Build a lookup from node_id -> score.
            use std::collections::HashMap;
            let id_to_score: HashMap<i64, f64> = repo_map.ranked_symbols.iter()
                .map(|rs| (rs.node_id, rs.rank))
                .collect();

            snapshot.nodes.iter().map(|node| {
                let score = id_to_score.get(&node.node_id).copied().unwrap_or(0.0);
                (
                    node.symbol_name.clone(),
                    node.file_path.to_string_lossy().into_owned(),
                    score,
                )
            }).collect()
        };

        // Batch insert.
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO dkg_nodes_pagerank (symbol, path, score) VALUES (?1, ?2, ?3)"
            )?;
            for (symbol, path, score) in &scores {
                stmt.execute(rusqlite::params![symbol, path, score])?;
            }
        }
        tx.commit()?;

        tracing::debug!(
            "H-10 pagerank: materialised {} rows into dkg_nodes_pagerank",
            scores.len()
        );

        Ok(())
    }

    fn generate_cross_edges(&self) -> Result<(), KnowledgeError> {
        // "imports" edges: use_declaration nodes -> matching named nodes
        // Gather use_declaration signatures
        let mut import_stmt = self.conn.prepare(
            "SELECT id, signature FROM dkg_nodes WHERE signature LIKE 'use %'"
        )?;
        let imports: Vec<(i64, String)> = import_stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.filter_map(|r| r.ok()).collect();

        for (use_id, sig) in &imports {
            // Extract the last segment of the use path as the target name
            let path_str = sig.strip_prefix("use ").unwrap_or(sig);
            if let Some(last_segment) = path_str.rsplit("::").next() {
                // Find a node with that name (not the use node itself)
                let target: Option<i64> = self.conn.query_row(
                    "SELECT id FROM dkg_nodes WHERE name = ?1 AND id != ?2 ORDER BY id ASC LIMIT 1",
                    rusqlite::params![last_segment, use_id],
                    |row| row.get(0),
                ).ok();
                if let Some(target_id) = target {
                    self.conn.execute(
                        "INSERT OR IGNORE INTO dkg_edges (source_id, target_id, edge_type) VALUES (?1, ?2, ?3)",
                        rusqlite::params![use_id, target_id, EDGE_IMPORTS],
                    )?;
                } else {
                    tracing::warn!("unresolved import: {}", path_str);
                }
            }
        }

        // "implements" edges: impl nodes with trait -> trait nodes
        let mut impl_stmt = self.conn.prepare(
            "SELECT id, name FROM dkg_nodes WHERE kind = 'impl'"
        )?;
        let impls: Vec<(i64, String)> = impl_stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.filter_map(|r| r.ok()).collect();

        for (impl_id, impl_name) in &impls {
            // Check if name contains "for" indicating trait impl: "impl TraitName for TypeName"
            if let Some(trait_name) = extract_trait_from_impl_name(impl_name) {
                let target: Option<i64> = self.conn.query_row(
                    "SELECT id FROM dkg_nodes WHERE name = ?1 AND kind = 'trait' ORDER BY id ASC LIMIT 1",
                    [&trait_name],
                    |row| row.get(0),
                ).ok();
                if let Some(trait_id) = target {
                    self.conn.execute(
                        "INSERT OR IGNORE INTO dkg_edges (source_id, target_id, edge_type) VALUES (?1, ?2, ?3)",
                        rusqlite::params![impl_id, trait_id, EDGE_IMPLEMENTS],
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Post-extraction pass: resolve module/import nodes for non-Rust languages
    /// to their target file paths and insert `imports` edges.
    ///
    /// Called inside build_graph immediately before generate_cross_edges().
    /// Builds all four resolvers lazily (each takes <100ms on typical projects).
    /// Rows for each resolved import produce a dkg_edges (source_id, target_id, 'imports') row.
    ///
    /// Name cleaning: dkg_nodes.name holds the raw AST text for import nodes
    /// (e.g. "import com.example.Foo;" for Java). This method strips language-
    /// specific prefixes/suffixes before calling the resolver.
    fn resolve_multi_module_imports(&self, project_root: &Path) -> Result<(), KnowledgeError> {
        let jvm = JvmResolver::new(project_root);
        let csharp = CsNamespaceResolver::new(project_root);
        let go = GoModuleResolver::new(project_root);
        let ts = TsPathAliasResolver::new(project_root);

        let mut stmt = self.conn.prepare(
            "SELECT id, name, language FROM dkg_nodes \
             WHERE kind='module' \
             AND language IN ('java','kotlin','scala','csharp','go','typescript','tsx','javascript','svelte','vue')"
        )?;

        let rows: Vec<(i64, String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);

        let mut edge_inserts: Vec<(i64, i64)> = Vec::new();

        for (src_id, raw_name, lang) in &rows {
            let cleaned = clean_import_name(raw_name, lang);
            for import_str in cleaned {
                let resolved = match lang.as_str() {
                    "java" | "kotlin" | "scala" => jvm.resolve(&import_str),
                    "csharp" => csharp.resolve(&import_str),
                    "go" => go.resolve(&import_str),
                    "typescript" | "tsx" | "javascript" | "svelte" | "vue" => ts.resolve(&import_str),
                    _ => None,
                };

                if let Some(target_path) = resolved {
                    // For Go imports the resolver returns a directory PathBuf, not
                    // an individual file. Enumerate .go files in that directory and
                    // use the first one that has an entry in dkg_nodes.
                    // For all other languages the resolved path is a file path.
                    let target_ids: Vec<i64> = if lang == "go" {
                        let go_files: Vec<PathBuf> = std::fs::read_dir(&target_path)
                            .ok()
                            .into_iter()
                            .flatten()
                            .flatten()
                            .filter(|e| {
                                e.path().extension().map_or(false, |x| x == "go")
                            })
                            .map(|e| e.path())
                            .collect();
                        go_files
                            .iter()
                            .filter_map(|gf| {
                                let gf_str = gf.to_string_lossy().to_string();
                                self.conn.query_row(
                                    "SELECT id FROM dkg_nodes WHERE file_path = ?1 AND kind='module' LIMIT 1",
                                    rusqlite::params![gf_str],
                                    |r| r.get::<_, i64>(0),
                                ).ok()
                            })
                            .take(1)
                            .collect()
                    } else {
                        let target_path_str = target_path.to_string_lossy().to_string();
                        self.conn.query_row(
                            "SELECT id FROM dkg_nodes WHERE file_path = ?1 AND kind='module' LIMIT 1",
                            rusqlite::params![target_path_str],
                            |r| r.get::<_, i64>(0),
                        ).ok().into_iter().collect()
                    };
                    for t in target_ids {
                        edge_inserts.push((*src_id, t));
                    }
                }
            }
        }

        for (src, dst) in edge_inserts {
            let _ = self.conn.execute(
                "INSERT OR IGNORE INTO dkg_edges (source_id, target_id, edge_type) VALUES (?1, ?2, 'imports')",
                rusqlite::params![src, dst],
            );
        }

        Ok(())
    }

    pub fn nodes(&self) -> Result<Vec<DkgNodeInfo>, KnowledgeError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, file_path, signature, kind, start_line, end_line FROM dkg_nodes",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DkgNodeInfo {
                node_id: row.get(0)?,
                symbol_name: row.get(1)?,
                file_path: {
                    let p: String = row.get(2)?;
                    std::path::PathBuf::from(p)
                },
                signature: row.get(3)?,
                kind: parse_symbol_kind(&row.get::<_, String>(4)?),
                start_line: row.get(5)?,
                end_line: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    /// Cheap node-count probe used as the memory guard before a caller
    /// materialises the full node + edge set (e.g. `DkgSnapshot::from_engine`).
    /// Runs a single `COUNT(*)` against `dkg_nodes` instead of loading every
    /// row, so it is safe to call per-query on arbitrarily large graphs.
    ///
    /// retrieval-mem-fixes FIX #2a: the pagerank Stage 2 path uses this to skip
    /// the snapshot build entirely when the graph exceeds `ELAI_PAGERANK_MAX_NODES`,
    /// bounding peak memory on giant codebases regardless of repo size.
    pub fn node_count(&self) -> Result<usize, KnowledgeError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM dkg_nodes", [], |row| row.get(0))?;
        Ok(usize::try_from(count).unwrap_or(0))
    }

    /// B-30: per-language node counts for the `phase_1A.toml`
    /// `[per_language_node_counts]` metric table. Returns one `(language_id,
    /// count)` pair per populated language, ordered by `language` ascending so
    /// the caller can emit a deterministic TOML table. Languages with zero
    /// nodes are absent from the result; the caller is responsible for filling
    /// the full known key set with `0` where missing.
    pub fn node_counts_by_language(&self) -> Result<Vec<(String, i64)>, KnowledgeError> {
        let mut stmt = self
            .conn
            .prepare("SELECT language, COUNT(*) FROM dkg_nodes GROUP BY language ORDER BY language ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut result: Vec<(String, i64)> = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn edges(&self) -> Result<EdgeList, KnowledgeError> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_id, target_id, edge_type FROM dkg_edges")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn node_by_id(&self, id: i64) -> Result<Option<DkgNodeInfo>, KnowledgeError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, file_path, signature, kind, start_line, end_line FROM dkg_nodes WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(DkgNodeInfo {
                node_id: row.get(0)?,
                symbol_name: row.get(1)?,
                file_path: {
                    let p: String = row.get(2)?;
                    std::path::PathBuf::from(p)
                },
                signature: row.get(3)?,
                kind: parse_symbol_kind(&row.get::<_, String>(4)?),
                start_line: row.get(5)?,
                end_line: row.get(6)?,
            })
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// F-5 staleness-quarantine cascade: write-through demotion of a DKG concept's
    /// freshness marker. `concept_stale` semantics: 0 = explicitly fresh, 1 = default
    /// (the DDL default at dkg/mod.rs schema + the value the sole node INSERT hardcodes),
    /// 2 = quarantined by the staleness cascade. There are ZERO production READ consumers
    /// of `concept_stale` today, so this mutator is purely write-through; the Phase-1
    /// contract asserts the UPDATE round-trip (write 2, re-query reads 2). Matching is on
    /// the `concept` TEXT column (dkg_nodes has no fact_id; fact->concept join is by text).
    /// Returns the number of rows updated. Idempotent for a given (concept, value).
    pub fn set_concept_stale(&self, concept: &str, value: i64) -> Result<u64, KnowledgeError> {
        let rows = self.conn.execute(
            "UPDATE dkg_nodes SET concept_stale = ?2 WHERE concept = ?1",
            rusqlite::params![concept, value],
        )?;
        u64::try_from(rows).map_err(|e| {
            KnowledgeError::Graph(format!("set_concept_stale rows_updated overflow: {e}"))
        })
    }

    /// F-5 / Slice A4: apply a batch of memory-side staleness-quarantine
    /// records to the DKG (the knowledge half of the cross-subsystem cascade).
    ///
    /// The CLI/orchestrator boundary drains the `QuarantineBroadcastQueue`
    /// (owned by `crates/memory`) and calls this with one
    /// [`QuarantineRecordView`] per quarantined fact. `crates/knowledge` has no
    /// dependency on `crates/memory`, so the caller projects each
    /// `QuarantineRecord` into the local [`QuarantineRecordView`] view struct at
    /// the bridge.
    ///
    /// Per record this does two non-destructive things:
    ///   1. Demote every DKG concept whose `concept` TEXT exactly equals the
    ///      quarantined fact's id to `concept_stale = 2` (the quarantine
    ///      marker; reuses [`Self::set_concept_stale`]).
    ///   2. Insert a non-destructive `staleness_quarantined` marker edge for
    ///      every matched node. `dkg_edges` has NO temporal column, so the
    ///      bi-temporal "close" is expressed as an INSERT of a marker edge
    ///      (the Graphiti / Mem0 non-destructive-invalidation pattern) rather
    ///      than an UPDATE of a `valid_to`. The marker is a self-loop
    ///      (`source_id == target_id == node.id`, `edge_type =
    ///      'staleness_quarantined'`); `INSERT OR IGNORE` keeps it idempotent.
    ///
    /// Match criterion (documented per F-5-decisions.md unknown #1): EXACT
    /// equality `dkg_nodes.concept == record.fact_id`. `dkg_nodes` has no
    /// `fact_id` column, so the only fact->concept join surface is the
    /// `concept` TEXT column. Exact equality is the conservative choice: a
    /// substring / LIKE join would over-demote unrelated concepts that merely
    /// contain the fact id as a fragment.
    ///
    /// Returns the aggregate [`StalenessApplyStats`] across the whole batch.
    pub fn apply_staleness_broadcast(
        &self,
        records: &[QuarantineRecordView],
    ) -> Result<StalenessApplyStats, KnowledgeError> {
        let mut stats = StalenessApplyStats::default();
        for record in records {
            // (1) Demote matching concepts to the quarantine marker value 2.
            let demoted = self.set_concept_stale(&record.fact_id, CONCEPT_STALE_QUARANTINED)?;
            stats.dependents_demoted = stats.dependents_demoted.saturating_add(demoted);

            // (2) Insert one non-destructive self-loop marker edge per matched
            //     node. Collect the matched node ids first (the borrow of the
            //     prepared statement must end before the edge INSERTs run).
            let node_ids = {
                let mut stmt = self
                    .conn
                    .prepare("SELECT id FROM dkg_nodes WHERE concept = ?1")?;
                let rows = stmt.query_map(rusqlite::params![record.fact_id], |row| {
                    row.get::<_, i64>(0)
                })?;
                let mut ids = Vec::new();
                for r in rows {
                    ids.push(r?);
                }
                ids
            };
            for node_id in node_ids {
                let inserted = self.conn.execute(
                    "INSERT OR IGNORE INTO dkg_edges (source_id, target_id, edge_type) \
                     VALUES (?1, ?1, ?2)",
                    rusqlite::params![node_id, EDGE_STALENESS_QUARANTINED],
                )?;
                let inserted = u64::try_from(inserted).map_err(|e| {
                    KnowledgeError::Graph(format!(
                        "apply_staleness_broadcast marker-edge rows overflow: {e}"
                    ))
                })?;
                stats.marker_edges_inserted =
                    stats.marker_edges_inserted.saturating_add(inserted);
            }
        }
        Ok(stats)
    }

    pub fn outbound_edges(&self, node_id: i64) -> Result<Vec<(i64, String)>, KnowledgeError> {
        let mut stmt = self
            .conn
            .prepare("SELECT target_id, edge_type FROM dkg_edges WHERE source_id = ?1")?;
        let rows = stmt.query_map([node_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn inbound_edges(&self, node_id: i64) -> Result<Vec<(i64, String)>, KnowledgeError> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_id, edge_type FROM dkg_edges WHERE target_id = ?1")?;
        let rows = stmt.query_map([node_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn failed_files(&self) -> Result<Vec<String>, KnowledgeError> {
        let mut stmt = self
            .conn
            .prepare("SELECT file_path FROM dkg_files WHERE parse_status = 'failed'")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    /// Return a snapshot of the language registry status (loaded + failed grammars).
    ///
    /// Callers (e.g. indexing_service) can convert this to `LanguageGrammarStatusPayload`
    /// via the `From` impl in the orchestrator crate for SSB emission.
    pub fn registry_status(&self) -> LanguageRegistryStatus {
        self.registry.status()
    }

    // -----------------------------------------------------------------------
    // A-48: Data flow edge queries
    // -----------------------------------------------------------------------

    /// Return all data flow edges in the graph.
    pub fn data_flow_edges(&self) -> Result<Vec<DataFlowEdge>, KnowledgeError> {
        let mut stmt = self.conn.prepare(
            "SELECT source_node_id, target_node_id, variable_name, flow_type FROM dkg_data_flow",
        )?;
        let rows = stmt.query_map([], |row| {
            let ft_str: String = row.get(3)?;
            Ok(DataFlowEdge {
                source_node_id: row.get(0)?,
                target_node_id: row.get(1)?,
                variable_name: row.get(2)?,
                flow_type: FlowType::from_str(&ft_str).unwrap_or_else(|| {
                    tracing::warn!("unknown flow_type in dkg_data_flow: {ft_str}");
                    FlowType::Assignment
                }),
            })
        })?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    /// Return data flow edges where the given node is source or target.
    pub fn data_flow_for_node(&self, node_id: i64) -> Result<Vec<DataFlowEdge>, KnowledgeError> {
        let mut stmt = self.conn.prepare(
            "SELECT source_node_id, target_node_id, variable_name, flow_type \
             FROM dkg_data_flow WHERE source_node_id = ?1 OR target_node_id = ?1",
        )?;
        let rows = stmt.query_map([node_id], |row| {
            let ft_str: String = row.get(3)?;
            Ok(DataFlowEdge {
                source_node_id: row.get(0)?,
                target_node_id: row.get(1)?,
                variable_name: row.get(2)?,
                flow_type: FlowType::from_str(&ft_str).unwrap_or_else(|| {
                    tracing::warn!("unknown flow_type in dkg_data_flow: {ft_str}");
                    FlowType::Assignment
                }),
            })
        })?;
        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    /// Insert a data flow edge directly (test-only helper).
    /// Auto-inserts stub dkg_nodes if the referenced IDs don't exist (avoids FK errors).
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn insert_test_data_flow(
        &self,
        source_node_id: i64,
        target_node_id: i64,
        variable_name: &str,
        flow_type: FlowType,
    ) {
        for nid in [source_node_id, target_node_id] {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO dkg_nodes (id, file_path, name, kind, signature, start_line, end_line) \
                     VALUES (?1, 'test.rs', ?2, 'function', '', 1, 1)",
                    rusqlite::params![nid, format!("stub_{nid}")],
                )
                .expect("insert stub node");
        }
        self.conn
            .execute(
                "INSERT INTO dkg_data_flow (source_node_id, target_node_id, variable_name, flow_type) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![source_node_id, target_node_id, variable_name, flow_type.as_str()],
            )
            .expect("insert_test_data_flow is test-only");
    }

    // -----------------------------------------------------------------------
    // A-48: Data flow extraction (heuristic, tree-sitter based)
    // -----------------------------------------------------------------------

    fn extract_all_data_flow_edges(
        &mut self,
        source_files: &[PathBuf],
    ) -> Result<(), KnowledgeError> {
        // Clear existing data flow edges (full rebuild, like cross-edges)
        self.conn
            .execute("DELETE FROM dkg_data_flow", [])?;

        let mut parser = tree_sitter::Parser::new();

        for file_path in source_files {
            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let lang_id = match ext_to_lang_id(ext) {
                Some(id) => id,
                None => continue,
            };

            // Only extract data flow for Rust files for now (primary language)
            if lang_id != "rust" {
                continue;
            }

            let grammar = match self.registry.get(lang_id) {
                Some(g) => g,
                None => continue,
            };

            let source_bytes = match std::fs::read(file_path) {
                Ok(b) => b,
                Err(_) => continue,
            };

            if std::str::from_utf8(&source_bytes).is_err() {
                continue;
            }

            parser
                .set_language(&grammar.tree_sitter_language())
                .map_err(|e| KnowledgeError::Graph(format!("set_language: {e}")))?;

            let tree = match parser.parse(&source_bytes, None) {
                Some(t) => t,
                None => continue,
            };

            if tree.root_node().has_error() {
                continue;
            }

            let file_path_str = file_path.to_string_lossy().to_string();
            self.extract_rust_data_flow(&tree, &source_bytes, &file_path_str)?;
        }

        Ok(())
    }

    /// Walk tree-sitter AST iteratively (cursor-based, stack-safe) and extract
    /// heuristic data flow edges between DKG-level nodes.
    ///
    /// Edges extracted:
    /// - call_expression `callee(args)` → RETURN edge callee->enclosing_fn
    /// - call_expression arguments → PARAMETER edge enclosing_fn->callee
    /// - let_declaration with identifier RHS → ASSIGNMENT edge (if both resolve)
    /// - field_expression `obj.field` → FIELD_ACCESS edge obj_type->enclosing_fn
    ///
    /// Limitations (heuristic, not semantic analysis):
    /// - Method calls (`self.foo()`) resolve callee as `foo`, not the full path
    /// - Cannot track closures, dynamic dispatch, macros, complex control flow
    /// - Only resolves names that exist as DKG nodes (functions, structs, etc.)
    fn extract_rust_data_flow(
        &self,
        tree: &tree_sitter::Tree,
        source: &[u8],
        file_path: &str,
    ) -> Result<(), KnowledgeError> {
        let mut cursor = tree.root_node().walk();
        let mut did_enter = true;

        loop {
            if did_enter {
                let node = cursor.node();
                let kind = node.kind();

                match kind {
                    "call_expression" => {
                        // Extract the bare function name (strip method receiver)
                        let callee_raw = node
                            .child_by_field_name("function")
                            .and_then(|n| n.utf8_text(source).ok())
                            .unwrap_or("");
                        // For method calls like `self.conn.execute`, take last segment
                        let callee_name = callee_raw
                            .rsplit('.')
                            .next()
                            .unwrap_or(callee_raw);

                        let enclosing = find_enclosing_function(node, source);

                        if !callee_name.is_empty() {
                            if let Some(ref enc_name) = enclosing {
                                // RETURN edge: callee -> enclosing function
                                self.insert_flow_edge_by_name(
                                    file_path,
                                    callee_name,
                                    enc_name,
                                    callee_name,
                                    FLOW_RETURN,
                                )?;
                            }

                            // PARAMETER edges for identifier arguments
                            if let Some(args_node) = node.child_by_field_name("arguments") {
                                // NARROWING-OK: pre-bounded by .min(u32::MAX as usize) before cast.
                                let arg_count = args_node.named_child_count().min(u32::MAX as usize) as u32;
                                for i in 0..arg_count {
                                    if let Some(arg) = args_node.named_child(i) {
                                        if arg.kind() == "identifier" {
                                            let arg_text = arg.utf8_text(source).unwrap_or("");
                                            if !arg_text.is_empty() {
                                                if let Some(ref enc_name) = enclosing {
                                                    self.insert_flow_edge_by_name(
                                                        file_path,
                                                        enc_name,
                                                        callee_name,
                                                        arg_text,
                                                        FLOW_PARAMETER,
                                                    )?;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "let_declaration" => {
                        // ASSIGNMENT: let x = some_identifier
                        if let Some(value_node) = node.child_by_field_name("value") {
                            if value_node.kind() == "identifier" {
                                let rhs_name = value_node
                                    .utf8_text(source)
                                    .unwrap_or("");
                                let lhs_name = node
                                    .child_by_field_name("pattern")
                                    .and_then(|n| n.utf8_text(source).ok())
                                    .unwrap_or("");
                                if !rhs_name.is_empty() && !lhs_name.is_empty() {
                                    if let Some(enc_name) = find_enclosing_function(node, source) {
                                        // ASSIGNMENT between DKG nodes (if rhs resolves)
                                        self.insert_flow_edge_by_name(
                                            file_path,
                                            rhs_name,
                                            &enc_name,
                                            lhs_name,
                                            FLOW_ASSIGNMENT,
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                    "field_expression" => {
                        // FIELD_ACCESS: obj.field → edge from obj_type to enclosing fn
                        let field_name = node
                            .child_by_field_name("field")
                            .and_then(|n| n.utf8_text(source).ok())
                            .unwrap_or("");
                        // Try to get the object identifier (for `self.field`, obj = "self")
                        let obj_node = node.child_by_field_name("value");
                        let obj_name = obj_node
                            .and_then(|n| {
                                // For simple identifiers, use directly
                                if n.kind() == "identifier" || n.kind() == "self" {
                                    n.utf8_text(source).ok()
                                } else {
                                    None
                                }
                            })
                            .unwrap_or("");

                        if !field_name.is_empty() {
                            if let Some(enc_name) = find_enclosing_function(node, source) {
                                // Try to resolve obj as a struct/type DKG node
                                // For `self`, look up the enclosing impl's type
                                let type_name = if obj_name == "self" || obj_name == "Self" {
                                    find_enclosing_impl_type(node, source)
                                } else {
                                    Some(obj_name.to_string())
                                };

                                if let Some(ref tname) = type_name {
                                    self.insert_flow_edge_by_name(
                                        file_path,
                                        tname,
                                        &enc_name,
                                        field_name,
                                        FLOW_FIELD_ACCESS,
                                    )?;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Iterative traversal: try child, then sibling, then parent's sibling
            if did_enter && cursor.goto_first_child() {
                did_enter = true;
            } else if cursor.goto_next_sibling() {
                did_enter = true;
            } else if cursor.goto_parent() {
                did_enter = false;
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Insert a data flow edge resolving names to DKG node IDs.
    /// If either name cannot be resolved to a DKG node, the edge is silently
    /// skipped (heuristic enrichment, not a gate).
    fn insert_flow_edge_by_name(
        &self,
        file_path: &str,
        source_name: &str,
        target_name: &str,
        variable_name: &str,
        flow_type: &str,
    ) -> Result<(), KnowledgeError> {
        let source_id = self.resolve_node_id(source_name, file_path);
        let target_id = self.resolve_node_id(target_name, file_path);

        if let (Some(sid), Some(tid)) = (source_id, target_id) {
            if sid != tid {
                self.conn.execute(
                    "INSERT OR IGNORE INTO dkg_data_flow (source_node_id, target_node_id, variable_name, flow_type) \
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![sid, tid, variable_name, flow_type],
                )?;
            }
        }
        Ok(())
    }

    fn resolve_node_id(&self, name: &str, file_path: &str) -> Option<i64> {
        self.conn
            .query_row(
                "SELECT id FROM dkg_nodes WHERE name = ?1 AND file_path = ?2 LIMIT 1",
                rusqlite::params![name, file_path],
                |row| row.get(0),
            )
            .ok()
            .or_else(|| {
                self.conn
                    .query_row(
                        "SELECT id FROM dkg_nodes WHERE name = ?1 LIMIT 1",
                        [name],
                        |row| row.get(0),
                    )
                    .ok()
            })
    }

    /// Extract project conventions from the DKG symbol graph.
    ///
    /// Queries all nodes, classifies naming patterns, error handling, module
    /// organization, and test patterns. Requires a populated graph (Rust-only
    /// as of Phase 1A). Returns an error if the graph is empty/unavailable.
    ///
    /// Spec: project-intelligence-design.txt sections 3A-3D, 10.
    pub fn extract_conventions(
        &self,
        project_root: &Path,
    ) -> Result<crate::conventions::ConventionStats, KnowledgeError> {
        let readiness = self.dkg_readiness();
        if readiness != DkgReadiness::Ready {
            return Err(KnowledgeError::Graph(format!(
                "DKG not ready for convention extraction (status: {readiness:?})"
            )));
        }
        let nodes = self.nodes()?;
        Ok(crate::conventions::extract_from_dkg(&nodes, project_root))
    }

    pub fn dkg_readiness(&self) -> DkgReadiness {
        let count: Result<i64, _> = self
            .conn
            .query_row("SELECT COUNT(*) FROM dkg_nodes", [], |row| row.get(0));
        match count {
            Ok(n) if n > 0 => DkgReadiness::Ready,
            Ok(_) if self.build_graph_called => DkgReadiness::Empty,
            Ok(_) => DkgReadiness::Unavailable,
            Err(_) => DkgReadiness::Unavailable,
        }
    }

    /// Find functions/methods with symbol names similar to `query` by token overlap.
    ///
    /// Algorithm: tokenize `query` and each symbol_name by splitting on `_` and
    /// lowercasing. Score = |intersection(query_tokens, name_tokens)|. Return
    /// nodes with score >= 1, sorted descending by score, limited to `k`.
    /// Returns empty vec (not error) when DB is empty or no matches found.
    pub fn find_similar_functions(
        &self,
        query: &str,
        k: usize,
    ) -> Result<Vec<DkgNodeInfo>, KnowledgeError> {
        if query.is_empty() || k == 0 {
            return Ok(Vec::new());
        }

        let query_tokens: Vec<String> = tokenize_symbol(query);
        if query_tokens.is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = self.conn.prepare(
            "SELECT id, name, file_path, signature, kind, start_line, end_line \
             FROM dkg_nodes WHERE kind IN ('function', 'method')",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DkgNodeInfo {
                node_id: row.get(0)?,
                symbol_name: row.get(1)?,
                file_path: {
                    let p: String = row.get(2)?;
                    PathBuf::from(p)
                },
                signature: row.get(3)?,
                kind: parse_symbol_kind(&row.get::<_, String>(4)?),
                start_line: row.get(5)?,
                end_line: row.get(6)?,
            })
        })?;

        let mut scored: Vec<(usize, DkgNodeInfo)> = Vec::new();
        for row_result in rows {
            let node = row_result?;
            let name_tokens = tokenize_symbol(&node.symbol_name);
            let score = query_tokens
                .iter()
                .filter(|qt| name_tokens.iter().any(|nt| nt == *qt))
                .count();
            if score >= SIMILARITY_MIN_TOKENS {
                scored.push((score, node));
            }
        }

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.truncate(k);
        Ok(scored.into_iter().map(|(_, node)| node).collect())
    }

    /// Insert a test node directly (test-only helper, not for production use).
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn insert_test_node(
        &self,
        name: &str,
        kind: &str,
        signature: &str,
        start_line: u32,
        end_line: u32,
    ) {
        self.conn
            .execute(
                "INSERT INTO dkg_nodes (file_path, name, kind, signature, start_line, end_line) \
                 VALUES ('test.rs', ?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![name, kind, signature, start_line, end_line],
            )
            .expect("insert_test_node is test-only");
    }
}

/// Minimum token overlap for similarity match inclusion.
const SIMILARITY_MIN_TOKENS: usize = 1;

/// Env-gated phase marker for index-time memory profiling. Emits a single line
/// to stderr only when `ELAI_DKG_PHASE_TRACE=1`, so it is zero-cost (one env
/// lookup) on the production path. Used by the `dkg_mem_probe` example to
/// correlate peak RSS samples with the active `build_graph` phase. NOT a
/// production observability surface — it is a diagnostic seam.
fn dkg_phase_marker(phase: &str) {
    if std::env::var_os("ELAI_DKG_PHASE_TRACE").is_some() {
        eprintln!("DKG_PHASE {phase}");
    }
}

/// Tokenize a symbol name by splitting on `_`, `-`, and camelCase boundaries,
/// then lowercasing. Handles snake_case, camelCase, PascalCase, kebab-case,
/// and SCREAMING_SNAKE_CASE (consecutive uppercase stays as one token).
fn tokenize_symbol(name: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut prev_was_lower = false;

    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            prev_was_lower = false;
        } else if ch.is_uppercase() && prev_was_lower {
            // True camelCase boundary: lowercase followed by uppercase.
            tokens.push(std::mem::take(&mut current));
            current.push(ch);
            prev_was_lower = false;
        } else {
            current.push(ch);
            prev_was_lower = ch.is_lowercase();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
        .into_iter()
        .map(|t| t.to_lowercase())
        .filter(|t| !t.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Data flow helpers
// ---------------------------------------------------------------------------

/// Walk up the tree-sitter AST from `node` to find the nearest enclosing
/// function_item (Rust). Returns the function name if found.
fn find_enclosing_function(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "function_item" {
            return parent
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .map(|s| s.to_string());
        }
        current = parent.parent();
    }
    None
}

/// Walk up from `node` to find the enclosing impl_item and return its type name.
/// For `impl Config { ... }` returns `"Config"`. For `impl Trait for Type`
/// returns `"Type"` (the concrete type, not the trait).
fn find_enclosing_impl_type(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "impl_item" {
            return parent
                .child_by_field_name("type")
                .and_then(|n| n.utf8_text(source).ok())
                .map(|s| s.to_string());
        }
        current = parent.parent();
    }
    None
}

// ---------------------------------------------------------------------------
// RustGrammar
// ---------------------------------------------------------------------------

pub struct RustGrammar;

impl LanguageGrammar for RustGrammar {
    fn language_id(&self) -> &'static str {
        "rust"
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

// ---------------------------------------------------------------------------
// PythonGrammar
// ---------------------------------------------------------------------------

pub struct PythonGrammar;

const PYTHON_KINDS: &[&str] = &[
    "function_definition",
    "class_definition",
    "import_statement",
    "import_from_statement",
    "decorated_definition",
];

impl LanguageGrammar for PythonGrammar {
    fn language_id(&self) -> &'static str { "python" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_python::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        python_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn python_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        // decorated_definition: descend into the inner definition
        if kind == "decorated_definition" {
            if cursor.goto_first_child() {
                python_extract_recursive(cursor, source, nodes, parent_idx);
                cursor.goto_parent();
            }
            if !cursor.goto_next_sibling() { break; }
            continue;
        }

        if PYTHON_KINDS.contains(&kind) {
            let name = python_extract_name(&node, source, kind);
            let signature = python_build_signature(&node, source, kind, &name);
            let symbol_kind = python_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);

            let idx = nodes.len();
            nodes.push(DkgNode {
                name,
                kind: symbol_kind,
                signature,
                start_line,
                end_line,
                parent_idx,
            });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            python_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() { break; }
    }
}

fn python_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "import_statement" | "import_from_statement" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        _ => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn python_build_signature(node: &tree_sitter::Node, source: &[u8], kind: &str, name: &str) -> String {
    match kind {
        "function_definition" => {
            let params = node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("(...)");
            let ret = node
                .child_by_field_name("return_type")
                .and_then(|n| n.utf8_text(source).ok());
            match ret {
                Some(r) => format!("def {}({}) -> {}", name, params.trim_start_matches('(').trim_end_matches(')'), r),
                None => format!("def {}({})", name, params.trim_start_matches('(').trim_end_matches(')')),
            }
        }
        "class_definition" => format!("class {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn python_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "function_definition" => SymbolKind::Function,
        "class_definition" => SymbolKind::Struct,
        "import_statement" | "import_from_statement" => SymbolKind::Module,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// GoGrammar
// ---------------------------------------------------------------------------

pub struct GoGrammar;

const GO_KINDS: &[&str] = &[
    "function_declaration",
    "method_declaration",
    "type_declaration",
    "import_declaration",
    "const_declaration",
    "var_declaration",
    "package_clause",
];

impl LanguageGrammar for GoGrammar {
    fn language_id(&self) -> &'static str { "go" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_go::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        go_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn go_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if GO_KINDS.contains(&kind) {
            let name = go_extract_name(&node, source, kind);
            let signature = go_build_signature(&node, source, kind, &name);
            let symbol_kind = go_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);

            let idx = nodes.len();
            nodes.push(DkgNode {
                name,
                kind: symbol_kind,
                signature,
                start_line,
                end_line,
                parent_idx,
            });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            go_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() { break; }
    }
}

fn go_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "import_declaration" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        "package_clause" => {
            node.child_by_field_name("name")
                .or_else(|| node.child(1u32))
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
        "type_declaration" => {
            // type_declaration contains type_spec children
            for i in 0..u32::try_from(node.child_count()).unwrap_or(u32::MAX) {
                if let Some(child) = node.child(i) {
                    if child.kind() == "type_spec" {
                        return child.child_by_field_name("name")
                            .and_then(|n| n.utf8_text(source).ok())
                            .unwrap_or("?")
                            .to_string();
                    }
                }
            }
            "?".to_string()
        }
        "method_declaration" => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
        _ => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn go_build_signature(node: &tree_sitter::Node, source: &[u8], kind: &str, name: &str) -> String {
    match kind {
        "function_declaration" => {
            let params = node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("(...)");
            let result = node
                .child_by_field_name("result")
                .and_then(|n| n.utf8_text(source).ok());
            match result {
                Some(r) => format!("func {}({}) {}", name, params.trim_start_matches('(').trim_end_matches(')'), r),
                None => format!("func {}({})", name, params.trim_start_matches('(').trim_end_matches(')')),
            }
        }
        "method_declaration" => {
            let receiver = node
                .child_by_field_name("receiver")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("(?)");
            let params = node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("(...)");
            format!("func {} {}({})", receiver, name, params.trim_start_matches('(').trim_end_matches(')'))
        }
        "type_declaration" => format!("type {}", name),
        "package_clause" => format!("package {}", name),
        "const_declaration" => format!("const {}", name),
        "var_declaration" => format!("var {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn go_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "function_declaration" | "method_declaration" => SymbolKind::Function,
        "type_declaration" => SymbolKind::Struct,
        "import_declaration" => SymbolKind::Module,
        "package_clause" => SymbolKind::Module,
        "const_declaration" => SymbolKind::Const,
        "var_declaration" => SymbolKind::Static,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// TypeScriptGrammar
// ---------------------------------------------------------------------------

pub struct TypeScriptGrammar;

const TS_KINDS: &[&str] = &[
    "function_declaration",
    "class_declaration",
    "interface_declaration",
    "enum_declaration",
    "type_alias_declaration",
    "import_statement",
    "export_statement",
    "method_definition",
    "lexical_declaration",
    "arrow_function",
];

impl LanguageGrammar for TypeScriptGrammar {
    fn language_id(&self) -> &'static str { "typescript" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        ts_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

// TSX grammar — uses LANGUAGE_TSX for JSX-aware parsing but same extraction logic.
pub struct TsxGrammar;

impl LanguageGrammar for TsxGrammar {
    fn language_id(&self) -> &'static str { "tsx" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        ts_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

// ---------------------------------------------------------------------------
// SvelteGrammar (C-pack C1)
// ---------------------------------------------------------------------------
//
// Minimal Svelte grammar. tree-sitter-svelte-ng exposes a unified tree where
// the embedded `<script>` block's children are JS/TS declaration nodes —
// `function_declaration`, `class_declaration`, etc. — using the same node
// kind strings as the TypeScript grammar. We can therefore reuse
// `ts_extract_recursive` and the `TS_KINDS` filter without modification.
// Files that contain no `<script>` block produce zero `DkgNode`s, which is
// the correct behaviour (no symbols to extract).
//
// Anti-Slopsquatting: `tree_sitter_svelte_ng` is already in
// `knowledge/Cargo.toml` (workspace dep, verified in s-44680 Phase B). The
// grammar's `LANGUAGE` constant is the same symbol we register in
// `ast_adapter.rs` L34-37.
pub struct SvelteGrammar;

impl LanguageGrammar for SvelteGrammar {
    fn language_id(&self) -> &'static str { "svelte" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_svelte_ng::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        ts_extract_recursive(&mut cursor, source, &mut nodes, None);
        // Pass D: also extract CSS @import paths from <style> blocks.
        extract_css_at_imports(tree.root_node(), source, &mut nodes);
        nodes
    }
}

// ---------------------------------------------------------------------------
// CSS @import extraction (shared by SvelteGrammar and VueGrammar, Pass D)
// ---------------------------------------------------------------------------

static CSS_AT_IMPORT_REGEX: OnceLock<Regex> = OnceLock::new();

fn css_at_import_regex() -> &'static Regex {
    CSS_AT_IMPORT_REGEX.get_or_init(|| {
        // Matches: @import "foo" | @import 'foo' | @import url("foo") | @import url('foo')
        Regex::new(r#"@import\s+(?:url\s*\(\s*)?["']([^"']+)["']"#)
            .expect("CSS_AT_IMPORT_REGEX is a valid static literal")
    })
}

/// Walk the tree recursively for `style_element` nodes, extract raw_text,
/// and push one `DkgNode` per CSS @import path (excluding http/https URLs).
fn extract_css_at_imports(
    root: tree_sitter::Node<'_>,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
) {
    let mut cursor = root.walk();
    extract_css_at_imports_recursive(&mut cursor, source, nodes);
}

fn extract_css_at_imports_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
) {
    loop {
        let node = cursor.node();
        if node.kind() == "style_element" {
            // Walk immediate children to find raw_text; do not recurse deeper.
            for i in 0..u32::try_from(node.child_count()).unwrap_or(u32::MAX) {
                if let Some(child) = node.child(i) {
                    if child.kind() == "raw_text" {
                        if let Ok(text) = child.utf8_text(source) {
                            let base_line = u32::try_from(child.start_position().row)
                                .unwrap_or(0)
                                .saturating_add(1);
                            for cap in css_at_import_regex().captures_iter(text) {
                                let path = &cap[1];
                                // Skip absolute URLs — they are external resources.
                                if path.starts_with("http:") || path.starts_with("https:") {
                                    continue;
                                }
                                nodes.push(DkgNode {
                                    name: path.to_string(),
                                    kind: SymbolKind::Module,
                                    signature: format!("css:@import {path}"),
                                    start_line: base_line,
                                    end_line: base_line,
                                    parent_idx: None,
                                });
                            }
                        }
                    }
                }
            }
            // Do not recurse into style_element children.
        } else if cursor.goto_first_child() {
            extract_css_at_imports_recursive(cursor, source, nodes);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// VueGrammar (Pass D)
// ---------------------------------------------------------------------------
//
// Vue SFCs (Single File Components) are parsed in two passes:
//   1. tree-sitter-html parses the outer structure exposing `script_element`
//      and `style_element` top-level nodes.
//   2. For each `script_element`, the raw_text child is extracted and
//      re-parsed with tree-sitter-typescript::LANGUAGE_TYPESCRIPT. The
//      resulting TS tree is walked with ts_extract_recursive to extract
//      function/class/interface/etc. nodes.
//   3. CSS @import paths inside `style_element` blocks are extracted via
//      extract_css_at_imports.
//
// Line numbers are relative to the extracted script/style block content
// (offset from the original .vue file — acceptable approximation for Pass D).
pub struct VueGrammar;

impl LanguageGrammar for VueGrammar {
    fn language_id(&self) -> &'static str { "vue" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        // The HTML grammar is used as the outer parser; the extract_nodes method
        // creates a secondary TS parser for <script> content.
        tree_sitter_html::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let root = tree.root_node();

        // Pass 1: extract TypeScript symbols from <script> blocks.
        let mut cursor = root.walk();
        vue_extract_script_recursive(&mut cursor, source, &mut nodes);

        // Pass 2: extract CSS @import paths from <style> blocks.
        extract_css_at_imports(root, source, &mut nodes);

        nodes
    }
}

/// Recursively walk an HTML tree (tree-sitter-html) looking for
/// `script_element` nodes. For each one found, extract the `raw_text` child
/// and re-parse it with the TypeScript grammar. Ignores other node kinds.
fn vue_extract_script_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
) {
    loop {
        let node = cursor.node();
        if node.kind() == "script_element" {
            // Find the raw_text child and re-parse as TypeScript.
            for i in 0..u32::try_from(node.child_count()).unwrap_or(u32::MAX) {
                if let Some(child) = node.child(i) {
                    if child.kind() == "raw_text" {
                        if let Ok(script_text) = child.utf8_text(source) {
                            let script_bytes = script_text.as_bytes();
                            let mut ts_parser = tree_sitter::Parser::new();
                            if ts_parser
                                .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                                .is_ok()
                            {
                                if let Some(ts_tree) = ts_parser.parse(script_bytes, None) {
                                    let mut ts_cursor = ts_tree.root_node().walk();
                                    ts_extract_recursive(
                                        &mut ts_cursor,
                                        script_bytes,
                                        nodes,
                                        None,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            // Do not recurse into script_element children.
        } else if cursor.goto_first_child() {
            vue_extract_script_recursive(cursor, source, nodes);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

fn ts_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        // export_statement wraps declarations — descend into it
        if kind == "export_statement" {
            if cursor.goto_first_child() {
                ts_extract_recursive(cursor, source, nodes, parent_idx);
                cursor.goto_parent();
            }
            if !cursor.goto_next_sibling() { break; }
            continue;
        }

        if TS_KINDS.contains(&kind) {
            let name = ts_extract_name(&node, source, kind);
            let signature = ts_build_signature(&node, source, kind, &name);
            let symbol_kind = ts_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);

            let idx = nodes.len();
            nodes.push(DkgNode {
                name,
                kind: symbol_kind,
                signature,
                start_line,
                end_line,
                parent_idx,
            });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            ts_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() { break; }
    }
}

fn ts_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "import_statement" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        "lexical_declaration" => {
            // const/let/var — find the first variable_declarator's name
            for i in 0..u32::try_from(node.child_count()).unwrap_or(u32::MAX) {
                if let Some(child) = node.child(i) {
                    if child.kind() == "variable_declarator" {
                        return child.child_by_field_name("name")
                            .and_then(|n| n.utf8_text(source).ok())
                            .unwrap_or("?")
                            .to_string();
                    }
                }
            }
            "?".to_string()
        }
        _ => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn ts_build_signature(node: &tree_sitter::Node, source: &[u8], kind: &str, name: &str) -> String {
    match kind {
        "function_declaration" => {
            let params = node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("(...)");
            let ret = node
                .child_by_field_name("return_type")
                .and_then(|n| n.utf8_text(source).ok());
            match ret {
                Some(r) => format!("function {}({}): {}", name, params.trim_start_matches('(').trim_end_matches(')'), r),
                None => format!("function {}({})", name, params.trim_start_matches('(').trim_end_matches(')')),
            }
        }
        "class_declaration" => format!("class {}", name),
        "interface_declaration" => format!("interface {}", name),
        "enum_declaration" => format!("enum {}", name),
        "type_alias_declaration" => format!("type {}", name),
        "method_definition" => format!("method {}", name),
        "lexical_declaration" => format!("const {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn ts_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "function_declaration" | "method_definition" | "arrow_function" => SymbolKind::Function,
        "class_declaration" => SymbolKind::Struct,
        "interface_declaration" => SymbolKind::Trait,
        "enum_declaration" => SymbolKind::Enum,
        "type_alias_declaration" => SymbolKind::TypeAlias,
        "import_statement" => SymbolKind::Module,
        "lexical_declaration" => SymbolKind::Const,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// JavaGrammar
// ---------------------------------------------------------------------------

pub struct JavaGrammar;

const JAVA_KINDS: &[&str] = &[
    "method_declaration",
    "constructor_declaration",
    "class_declaration",
    "interface_declaration",
    "enum_declaration",
    "import_declaration",
    "package_declaration",
    "field_declaration",
    "annotation_type_declaration",
];

impl LanguageGrammar for JavaGrammar {
    fn language_id(&self) -> &'static str { "java" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_java::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        java_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn java_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if JAVA_KINDS.contains(&kind) {
            let name = java_extract_name(&node, source, kind);
            let signature = java_build_signature(&node, source, kind, &name);
            let symbol_kind = java_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);

            let idx = nodes.len();
            nodes.push(DkgNode {
                name,
                kind: symbol_kind,
                signature,
                start_line,
                end_line,
                parent_idx,
            });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            java_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() { break; }
    }
}

fn java_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "import_declaration" | "package_declaration" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        "field_declaration" => {
            // Find the variable_declarator child for the field name
            for i in 0..u32::try_from(node.child_count()).unwrap_or(u32::MAX) {
                if let Some(child) = node.child(i) {
                    if child.kind() == "variable_declarator" {
                        return child.child_by_field_name("name")
                            .and_then(|n| n.utf8_text(source).ok())
                            .unwrap_or("?")
                            .to_string();
                    }
                }
            }
            "?".to_string()
        }
        _ => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn java_build_signature(node: &tree_sitter::Node, source: &[u8], kind: &str, name: &str) -> String {
    match kind {
        "method_declaration" | "constructor_declaration" => {
            let params = node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("(...)");
            let ret = node
                .child_by_field_name("type")
                .and_then(|n| n.utf8_text(source).ok());
            match ret {
                Some(r) => format!("{} {}({})", r, name, params.trim_start_matches('(').trim_end_matches(')')),
                None => format!("{}({})", name, params.trim_start_matches('(').trim_end_matches(')')),
            }
        }
        "class_declaration" => format!("class {}", name),
        "interface_declaration" => format!("interface {}", name),
        "enum_declaration" => format!("enum {}", name),
        "annotation_type_declaration" => format!("@interface {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn java_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "method_declaration" | "constructor_declaration" => SymbolKind::Function,
        "class_declaration" => SymbolKind::Struct,
        "interface_declaration" => SymbolKind::Trait,
        "enum_declaration" => SymbolKind::Enum,
        "import_declaration" | "package_declaration" => SymbolKind::Module,
        "field_declaration" => SymbolKind::Static,
        "annotation_type_declaration" => SymbolKind::Trait,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// KotlinGrammar (C-17: closes B-30 MISSING for .kt/.kts coverage)
// ---------------------------------------------------------------------------
//
// Uses the official tree-sitter-grammars/tree-sitter-kotlin grammar via the
// tree-sitter-kotlin-ng crate. Kind names follow that grammar's node
// inventory: `function_declaration`, `class_declaration`, `object_declaration`,
// `interface_declaration`, `property_declaration`, `import_header`,
// `package_header`, `enum_class_body`, `enum_entry`, `secondary_constructor`,
// `primary_constructor`. The walker mirrors JavaGrammar's structural pattern
// (recursive cursor walk + parent_idx chaining).

pub struct KotlinGrammar;

const KOTLIN_KINDS: &[&str] = &[
    "function_declaration",
    "primary_constructor",
    "secondary_constructor",
    "class_declaration",
    "object_declaration",
    "interface_declaration",
    "import_header",
    "package_header",
    "property_declaration",
    "enum_entry",
    "type_alias",
    "anonymous_initializer",
];

impl LanguageGrammar for KotlinGrammar {
    fn language_id(&self) -> &'static str { "kotlin" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_kotlin_ng::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        kotlin_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn kotlin_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if KOTLIN_KINDS.contains(&kind) {
            let name = kotlin_extract_name(&node, source, kind);
            let signature = kotlin_build_signature(&node, source, kind, &name);
            let symbol_kind = kotlin_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);

            let idx = nodes.len();
            nodes.push(DkgNode {
                name,
                kind: symbol_kind,
                signature,
                start_line,
                end_line,
                parent_idx,
            });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            kotlin_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() { break; }
    }
}

fn kotlin_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "import_header" | "package_header" => {
            // Carry the raw declaration text so clean_import_name's
            // kotlin arm (dkg/mod.rs ~L3279) can strip the leading
            // `import ` / `package ` keyword on the resolver side.
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        "primary_constructor" | "secondary_constructor" | "anonymous_initializer" => {
            // Anonymous initialisers and primary constructors have no
            // explicit name in Kotlin grammar; synthesize a stable label.
            kind.to_string()
        }
        _ => {
            node.child_by_field_name("name")
                .or_else(|| {
                    // Fallback: scan children for a `simple_identifier` /
                    // `type_identifier` node — grammar field naming varies
                    // across tree-sitter-kotlin versions.
                    let mut i = 0u32;
                    let max = u32::try_from(node.child_count()).unwrap_or(u32::MAX);
                    while i < max {
                        if let Some(c) = node.child(i) {
                            if c.kind() == "simple_identifier" || c.kind() == "type_identifier" {
                                return Some(c);
                            }
                        }
                        i = i.saturating_add(1);
                    }
                    None
                })
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn kotlin_build_signature(node: &tree_sitter::Node, source: &[u8], kind: &str, name: &str) -> String {
    match kind {
        "function_declaration" => {
            let params = node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("(...)");
            let ret = node
                .child_by_field_name("type")
                .and_then(|n| n.utf8_text(source).ok());
            match ret {
                Some(r) => format!("fun {}{}: {}", name, params, r),
                None => format!("fun {}{}", name, params),
            }
        }
        "class_declaration" => format!("class {}", name),
        "object_declaration" => format!("object {}", name),
        "interface_declaration" => format!("interface {}", name),
        "property_declaration" => format!("val/var {}", name),
        "enum_entry" => format!("enum_entry {}", name),
        "type_alias" => format!("typealias {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn kotlin_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "function_declaration" | "primary_constructor" | "secondary_constructor"
        | "anonymous_initializer" => SymbolKind::Function,
        "class_declaration" | "object_declaration" => SymbolKind::Struct,
        "interface_declaration" => SymbolKind::Trait,
        "import_header" | "package_header" => SymbolKind::Module,
        "property_declaration" => SymbolKind::Static,
        "enum_entry" => SymbolKind::Enum,
        "type_alias" => SymbolKind::Struct,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// CSharpGrammar
// ---------------------------------------------------------------------------

pub struct CSharpGrammar;

const CSHARP_KINDS: &[&str] = &[
    "method_declaration",
    "constructor_declaration",
    "class_declaration",
    "interface_declaration",
    "enum_declaration",
    "struct_declaration",
    "namespace_declaration",
    "using_directive",
    "property_declaration",
    "field_declaration",
    "record_declaration",
];

impl LanguageGrammar for CSharpGrammar {
    fn language_id(&self) -> &'static str { "csharp" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_c_sharp::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        csharp_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn csharp_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if CSHARP_KINDS.contains(&kind) {
            let name = csharp_extract_name(&node, source, kind);
            let signature = csharp_build_signature(&node, source, kind, &name);
            let symbol_kind = csharp_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);

            let idx = nodes.len();
            nodes.push(DkgNode {
                name,
                kind: symbol_kind,
                signature,
                start_line,
                end_line,
                parent_idx,
            });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            csharp_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() { break; }
    }
}

fn csharp_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "using_directive" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        "namespace_declaration" => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
        "field_declaration" => {
            // Walk children to find variable_declaration -> variable_declarator
            for i in 0..u32::try_from(node.child_count()).unwrap_or(u32::MAX) {
                if let Some(child) = node.child(i) {
                    if child.kind() == "variable_declaration" {
                        for j in 0..u32::try_from(child.child_count()).unwrap_or(u32::MAX) {
                            if let Some(decl) = child.child(j) {
                                if decl.kind() == "variable_declarator" {
                                    return decl.child_by_field_name("name")
                                        .or_else(|| decl.child(0u32))
                                        .and_then(|n| n.utf8_text(source).ok())
                                        .unwrap_or("?")
                                        .to_string();
                                }
                            }
                        }
                    }
                }
            }
            "?".to_string()
        }
        _ => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn csharp_build_signature(node: &tree_sitter::Node, source: &[u8], kind: &str, name: &str) -> String {
    match kind {
        "method_declaration" | "constructor_declaration" => {
            let params = node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("(...)");
            let ret = node
                .child_by_field_name("type")
                .or_else(|| node.child_by_field_name("returns"))
                .and_then(|n| n.utf8_text(source).ok());
            match ret {
                Some(r) => format!("{} {}({})", r, name, params.trim_start_matches('(').trim_end_matches(')')),
                None => format!("{}({})", name, params.trim_start_matches('(').trim_end_matches(')')),
            }
        }
        "class_declaration" | "record_declaration" => format!("class {}", name),
        "struct_declaration" => format!("struct {}", name),
        "interface_declaration" => format!("interface {}", name),
        "enum_declaration" => format!("enum {}", name),
        "namespace_declaration" => format!("namespace {}", name),
        "property_declaration" => format!("property {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn csharp_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "method_declaration" | "constructor_declaration" => SymbolKind::Function,
        "class_declaration" | "record_declaration" | "struct_declaration" => SymbolKind::Struct,
        "interface_declaration" => SymbolKind::Trait,
        "enum_declaration" => SymbolKind::Enum,
        "namespace_declaration" | "using_directive" => SymbolKind::Module,
        "property_declaration" | "field_declaration" => SymbolKind::Static,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// RubyGrammar
// ---------------------------------------------------------------------------

pub struct RubyGrammar;

const RUBY_KINDS: &[&str] = &[
    "method",
    "singleton_method",
    "class",
    "module",
    "constant",
    "assignment",
];

impl LanguageGrammar for RubyGrammar {
    fn language_id(&self) -> &'static str { "ruby" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_ruby::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        ruby_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn ruby_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        // Only capture top-level constant assignments (CONST = ...)
        if kind == "assignment" {
            let lhs = node.child(0u32);
            let is_const = lhs.map_or(false, |n| n.kind() == "constant");
            if is_const {
                let name = lhs
                    .and_then(|n| n.utf8_text(source).ok())
                    .unwrap_or("?")
                    .to_string();
                let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
                let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);
                nodes.push(DkgNode {
                    name: name.clone(),
                    kind: SymbolKind::Const,
                    signature: format!("const {}", name),
                    start_line,
                    end_line,
                    parent_idx,
                });
            }
            if !cursor.goto_next_sibling() { break; }
            continue;
        }

        if RUBY_KINDS.contains(&kind) && kind != "assignment" && kind != "constant" {
            let name = ruby_extract_name(&node, source, kind);
            let signature = ruby_build_signature(kind, &name);
            let symbol_kind = ruby_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);

            let idx = nodes.len();
            nodes.push(DkgNode {
                name,
                kind: symbol_kind,
                signature,
                start_line,
                end_line,
                parent_idx,
            });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            ruby_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() { break; }
    }
}

fn ruby_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "method" | "singleton_method" => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
        "class" | "module" => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
        _ => "?".to_string(),
    }
}

fn ruby_build_signature(kind: &str, name: &str) -> String {
    match kind {
        "method" | "singleton_method" => format!("def {}", name),
        "class" => format!("class {}", name),
        "module" => format!("module {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn ruby_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "method" | "singleton_method" => SymbolKind::Function,
        "class" => SymbolKind::Struct,
        "module" => SymbolKind::Module,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// JavaScriptGrammar
// ---------------------------------------------------------------------------

pub struct JavaScriptGrammar;

const JS_KINDS: &[&str] = &[
    "function_declaration",
    "class_declaration",
    "method_definition",
    "import_statement",
    "export_statement",
];

impl LanguageGrammar for JavaScriptGrammar {
    fn language_id(&self) -> &'static str { "javascript" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_javascript::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        js_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn js_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if JS_KINDS.contains(&kind) {
            let name = js_extract_name(&node, source, kind);
            let signature = js_build_signature(kind, &name);
            let symbol_kind = js_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let idx = nodes.len();
            nodes.push(DkgNode { name, kind: symbol_kind, signature, start_line, end_line, parent_idx });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            js_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() { break; }
    }
}

fn js_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "import_statement" | "export_statement" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        _ => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn js_build_signature(kind: &str, name: &str) -> String {
    match kind {
        "function_declaration" => format!("function {}", name),
        "class_declaration" => format!("class {}", name),
        "method_definition" => format!("method {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn js_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "function_declaration" | "method_definition" => SymbolKind::Function,
        "class_declaration" => SymbolKind::Struct,
        "import_statement" | "export_statement" => SymbolKind::Module,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// CGrammar
// ---------------------------------------------------------------------------

pub struct CGrammar;

const C_KINDS: &[&str] = &[
    "function_definition",
    "struct_specifier",
    "enum_specifier",
    "preproc_include",
];

impl LanguageGrammar for CGrammar {
    fn language_id(&self) -> &'static str { "c" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_c::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        c_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn c_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if C_KINDS.contains(&kind) {
            let name = c_extract_name(&node, source, kind);
            let signature = c_build_signature(kind, &name);
            let symbol_kind = c_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let idx = nodes.len();
            nodes.push(DkgNode { name, kind: symbol_kind, signature, start_line, end_line, parent_idx });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            c_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() { break; }
    }
}

fn c_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "function_definition" => {
            // function_definition -> declarator (function_declarator) -> declarator (identifier)
            let declarator = node.child_by_field_name("declarator");
            let name = declarator
                .and_then(|d| {
                    // May be a pointer_declarator wrapping function_declarator
                    let fd = if d.kind() == "function_declarator" {
                        Some(d)
                    } else {
                        d.child_by_field_name("declarator")
                    };
                    fd.and_then(|fd| fd.child_by_field_name("declarator"))
                        .and_then(|id| id.utf8_text(source).ok())
                })
                .unwrap_or("?");
            name.to_string()
        }
        "struct_specifier" | "enum_specifier" => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
        "preproc_include" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        _ => "?".to_string(),
    }
}

fn c_build_signature(kind: &str, name: &str) -> String {
    match kind {
        "function_definition" => format!("fn {}", name),
        "struct_specifier" => format!("struct {}", name),
        "enum_specifier" => format!("enum {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn c_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "function_definition" => SymbolKind::Function,
        "struct_specifier" => SymbolKind::Struct,
        "enum_specifier" => SymbolKind::Enum,
        "preproc_include" => SymbolKind::Module,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// CppGrammar
// ---------------------------------------------------------------------------

pub struct CppGrammar;

const CPP_KINDS: &[&str] = &[
    "function_definition",
    "class_specifier",
    "struct_specifier",
    "namespace_definition",
    "preproc_include",
];

impl LanguageGrammar for CppGrammar {
    fn language_id(&self) -> &'static str { "cpp" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_cpp::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        cpp_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn cpp_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if CPP_KINDS.contains(&kind) {
            let name = cpp_extract_name(&node, source, kind);
            let signature = cpp_build_signature(kind, &name);
            let symbol_kind = cpp_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let idx = nodes.len();
            nodes.push(DkgNode { name, kind: symbol_kind, signature, start_line, end_line, parent_idx });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            cpp_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() { break; }
    }
}

fn cpp_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "function_definition" => {
            // Descend: function_definition -> declarator -> (function_declarator) -> declarator
            let declarator = node.child_by_field_name("declarator");
            // Could be function_declarator directly or pointer_declarator wrapping it
            let fn_decl = declarator.and_then(|d| {
                if d.kind() == "function_declarator" {
                    Some(d)
                } else {
                    d.child_by_field_name("declarator")
                }
            });
            let inner = fn_decl.and_then(|fd| fd.child_by_field_name("declarator"));
            inner
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
        "class_specifier" | "struct_specifier" => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
        "namespace_definition" => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("(anonymous)")
                .to_string()
        }
        "preproc_include" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        _ => "?".to_string(),
    }
}

fn cpp_build_signature(kind: &str, name: &str) -> String {
    match kind {
        "function_definition" => format!("fn {}", name),
        "class_specifier" => format!("class {}", name),
        "struct_specifier" => format!("struct {}", name),
        "namespace_definition" => format!("namespace {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn cpp_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "function_definition" => SymbolKind::Function,
        "class_specifier" => SymbolKind::Struct,
        "struct_specifier" => SymbolKind::Struct,
        "namespace_definition" => SymbolKind::Module,
        "preproc_include" => SymbolKind::Module,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// ScalaGrammar
// ---------------------------------------------------------------------------

pub struct ScalaGrammar;

const SCALA_KINDS: &[&str] = &[
    "class_definition",
    "object_definition",
    "trait_definition",
    "function_definition",
    "import_declaration",
];

impl LanguageGrammar for ScalaGrammar {
    fn language_id(&self) -> &'static str { "scala" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_scala::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        scala_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn scala_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if SCALA_KINDS.contains(&kind) {
            let name = scala_extract_name(&node, source, kind);
            let signature = scala_build_signature(kind, &name);
            let symbol_kind = scala_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let idx = nodes.len();
            nodes.push(DkgNode { name, kind: symbol_kind, signature, start_line, end_line, parent_idx });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            scala_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() { break; }
    }
}

fn scala_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "import_declaration" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        _ => {
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn scala_build_signature(kind: &str, name: &str) -> String {
    match kind {
        "class_definition" => format!("class {}", name),
        "object_definition" => format!("object {}", name),
        "trait_definition" => format!("trait {}", name),
        "function_definition" => format!("def {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn scala_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "class_definition" => SymbolKind::Struct,
        "object_definition" => SymbolKind::Module,
        "trait_definition" => SymbolKind::Trait,
        "function_definition" => SymbolKind::Function,
        "import_declaration" => SymbolKind::Module,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// PhpGrammar
// ---------------------------------------------------------------------------

pub struct PhpGrammar;

const PHP_KINDS: &[&str] = &[
    "function_definition",
    "method_declaration",
    "class_declaration",
    "interface_declaration",
    "trait_declaration",
    "namespace_use_clause",
];

impl LanguageGrammar for PhpGrammar {
    fn language_id(&self) -> &'static str { "php" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_php::LANGUAGE_PHP.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        php_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn php_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if PHP_KINDS.contains(&kind) {
            let name = php_extract_name(&node, source, kind);
            let signature = php_build_signature(kind, &name);
            let symbol_kind = php_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let idx = nodes.len();
            nodes.push(DkgNode { name, kind: symbol_kind, signature, start_line, end_line, parent_idx });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            php_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() { break; }
    }
}

fn php_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "namespace_use_clause" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        _ => {
            // PHP grammar uses `name` node kind (not `identifier`) for names.
            // child_by_field_name("name") returns the correct node regardless of its kind string.
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn php_build_signature(kind: &str, name: &str) -> String {
    match kind {
        "function_definition" => format!("function {}", name),
        "method_declaration" => format!("method {}", name),
        "class_declaration" => format!("class {}", name),
        "interface_declaration" => format!("interface {}", name),
        "trait_declaration" => format!("trait {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn php_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "function_definition" | "method_declaration" => SymbolKind::Function,
        "class_declaration" => SymbolKind::Struct,
        "interface_declaration" => SymbolKind::Trait,
        "trait_declaration" => SymbolKind::Trait,
        "namespace_use_clause" => SymbolKind::Module,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// SwiftGrammar
// ---------------------------------------------------------------------------

pub struct SwiftGrammar;

const SWIFT_KINDS: &[&str] = &[
    "function_declaration",
    "class_declaration",
    "struct_declaration",
    "protocol_declaration",
    "enum_declaration",
    "import_declaration",
];

impl LanguageGrammar for SwiftGrammar {
    fn language_id(&self) -> &'static str { "swift" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_swift::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        swift_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn swift_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if SWIFT_KINDS.contains(&kind) {
            let name = swift_extract_name(&node, source, kind);
            let signature = swift_build_signature(kind, &name);
            let symbol_kind = swift_kind_to_symbol(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let idx = nodes.len();
            nodes.push(DkgNode { name, kind: symbol_kind, signature, start_line, end_line, parent_idx });
            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            swift_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() { break; }
    }
}

fn swift_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "import_declaration" => {
            node.utf8_text(source).unwrap_or("").trim().to_string()
        }
        "function_declaration" => {
            // Swift uses `simple_identifier` as the preferred name child; fallback to `name`.
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
        _ => {
            // class/struct/protocol/enum: `type_identifier` child or `name` field
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn swift_build_signature(kind: &str, name: &str) -> String {
    match kind {
        "function_declaration" => format!("func {}", name),
        "class_declaration" => format!("class {}", name),
        "struct_declaration" => format!("struct {}", name),
        "protocol_declaration" => format!("protocol {}", name),
        "enum_declaration" => format!("enum {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn swift_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "function_declaration" => SymbolKind::Function,
        "class_declaration" => SymbolKind::Struct,
        "struct_declaration" => SymbolKind::Struct,
        // protocol maps to Trait for cross-language uniformity (per context package)
        "protocol_declaration" => SymbolKind::Trait,
        "enum_declaration" => SymbolKind::Enum,
        "import_declaration" => SymbolKind::Module,
        _ => SymbolKind::Function,
    }
}

// ---------------------------------------------------------------------------
// BashGrammar
// ---------------------------------------------------------------------------

pub struct BashGrammar;

const BASH_KINDS: &[&str] = &[
    "function_definition",
    "command",
];

/// Identifier-like regex pattern for Bash command names (avoids shell builtins/paths).
///
/// A "word" node whose text matches this pattern is treated as a function call name.
/// Shell built-ins (if, for, while, [, etc.) are filtered by this pattern.
fn bash_is_identifier_like(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl LanguageGrammar for BashGrammar {
    fn language_id(&self) -> &'static str { "bash" }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        tree_sitter_bash::LANGUAGE.into()
    }

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        bash_extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
    }
}

fn bash_extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if BASH_KINDS.contains(&kind) {
            let name_opt = bash_extract_name(&node, source, kind);
            if let Some(name) = name_opt {
                let signature = bash_build_signature(kind, &name);
                let symbol_kind = bash_kind_to_symbol(kind);
                let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
                let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);
                let idx = nodes.len();
                nodes.push(DkgNode { name, kind: symbol_kind, signature, start_line, end_line, parent_idx });
                current_parent = Some(idx);
            }
        }

        if cursor.goto_first_child() {
            bash_extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() { break; }
    }
}

fn bash_extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> Option<String> {
    match kind {
        "function_definition" => {
            // function_definition -> name field -> word node
            let name = node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?");
            Some(name.to_string())
        }
        "command" => {
            // command -> command_name child -> word; filter to identifier-like names only
            let name = node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("");
            if bash_is_identifier_like(name) {
                Some(name.to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn bash_build_signature(kind: &str, name: &str) -> String {
    match kind {
        "function_definition" => format!("function {}", name),
        "command" => format!("cmd {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn bash_kind_to_symbol(kind: &str) -> SymbolKind {
    match kind {
        "function_definition" => SymbolKind::Function,
        "command" => SymbolKind::Function,
        _ => SymbolKind::Function,
    }
}

const EXTRACTED_KINDS: &[&str] = &[
    "function_item",
    "struct_item",
    "impl_item",
    "use_declaration",
    "mod_item",
    "trait_item",
    "enum_item",
    "type_alias",
];

fn extract_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &[u8],
    nodes: &mut Vec<DkgNode>,
    parent_idx: Option<usize>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();
        let mut current_parent = parent_idx;

        if EXTRACTED_KINDS.contains(&kind) {
            let name = extract_name(&node, source, kind);
            let signature = build_signature(&node, source, kind, &name);
            let symbol_kind = ts_kind_to_symbol_kind(kind);
            let start_line = u32::try_from(node.start_position().row).unwrap_or(u32::MAX).saturating_add(1);
            let end_line = u32::try_from(node.end_position().row).unwrap_or(u32::MAX).saturating_add(1);

            let idx = nodes.len();
            nodes.push(DkgNode {
                name,
                kind: symbol_kind,
                signature,
                start_line,
                end_line,
                parent_idx,
            });

            current_parent = Some(idx);
        }

        if cursor.goto_first_child() {
            extract_recursive(cursor, source, nodes, current_parent);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

fn extract_name(node: &tree_sitter::Node, source: &[u8], kind: &str) -> String {
    match kind {
        "use_declaration" => {
            // Full path text minus "use " and ";"
            node.utf8_text(source)
                .unwrap_or("")
                .trim()
                .strip_prefix("use ")
                .unwrap_or("")
                .strip_suffix(';')
                .unwrap_or("")
                .trim()
                .to_string()
        }
        "impl_item" => {
            // "impl TraitName for TypeName" or "impl TypeName"
            let type_name = node
                .child_by_field_name("type")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?");
            let trait_name = node
                .child_by_field_name("trait")
                .and_then(|n| n.utf8_text(source).ok());
            match trait_name {
                Some(t) => format!("impl {} for {}", t, type_name),
                None => format!("impl {}", type_name),
            }
        }
        _ => {
            // Most items have a "name" field
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("?")
                .to_string()
        }
    }
}

fn build_signature(node: &tree_sitter::Node, source: &[u8], kind: &str, name: &str) -> String {
    match kind {
        "function_item" => {
            let params = node
                .child_by_field_name("parameters")
                .and_then(|n| n.utf8_text(source).ok())
                .unwrap_or("(...)");
            let ret = node
                .child_by_field_name("return_type")
                .and_then(|n| n.utf8_text(source).ok());
            match ret {
                Some(r) => format!("fn {}({}) -> {}", name, params.trim_start_matches('(').trim_end_matches(')'), r),
                None => format!("fn {}({})", name, params.trim_start_matches('(').trim_end_matches(')')),
            }
        }
        "struct_item" => format!("struct {}", name),
        "enum_item" => format!("enum {}", name),
        "trait_item" => format!("trait {}", name),
        "impl_item" => {
            // name already contains "impl X" or "impl T for X"
            name.to_string()
        }
        "mod_item" => format!("mod {}", name),
        "use_declaration" => format!("use {}", name),
        "type_alias" => format!("type {}", name),
        _ => format!("{} {}", kind, name),
    }
}

fn ts_kind_to_symbol_kind(kind: &str) -> SymbolKind {
    match kind {
        "function_item" => SymbolKind::Function,
        "struct_item" => SymbolKind::Struct,
        "enum_item" => SymbolKind::Enum,
        "trait_item" => SymbolKind::Trait,
        "impl_item" => SymbolKind::Impl,
        "mod_item" => SymbolKind::Module,
        "type_alias" => SymbolKind::TypeAlias,
        "use_declaration" => SymbolKind::Module,
        _ => SymbolKind::Function,
    }
}

fn symbol_kind_to_str(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Function => "function",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Trait => "trait",
        SymbolKind::Impl => "impl",
        SymbolKind::Module => "module",
        SymbolKind::Const => "const",
        SymbolKind::Static => "static",
        SymbolKind::TypeAlias => "type_alias",
        SymbolKind::Macro => "macro",
    }
}

fn extract_trait_from_impl_name(name: &str) -> Option<String> {
    // "impl TraitName for TypeName" -> Some("TraitName")
    let stripped = name.strip_prefix("impl ")?;
    let parts: Vec<&str> = stripped.splitn(3, ' ').collect();
    if parts.len() >= 3 && parts[1] == "for" {
        Some(parts[0].to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_symbol_kind(s: &str) -> SymbolKind {
    match s {
        "function" => SymbolKind::Function,
        "struct" => SymbolKind::Struct,
        "enum" => SymbolKind::Enum,
        "trait" => SymbolKind::Trait,
        "impl" => SymbolKind::Impl,
        "module" => SymbolKind::Module,
        "const" => SymbolKind::Const,
        "static" => SymbolKind::Static,
        "type_alias" => SymbolKind::TypeAlias,
        "macro" => SymbolKind::Macro,
        _ => SymbolKind::Function,
    }
}

const EXCLUDED_DIRS: &[&str] = &["target", ".git", ".cargo", "node_modules", "__pycache__", ".venv", "vendor", "dist", "build"];

/// Extensions that have a registered grammar.
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "go", "ts", "tsx", "svelte", "vue", "java", "kt", "kts", "cs", "rb",
    "js", "c", "cpp", "cc", "cxx", "scala", "php", "swift", "sh",
];

pub fn ext_to_lang_id(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "go" => Some("go"),
        "ts" => Some("typescript"),
        "tsx" => Some("tsx"),
        // C-pack C1: register .svelte alongside .ts/.tsx. The
        // SvelteGrammar reuses ts_extract_recursive because
        // tree-sitter-svelte-ng emits JS/TS declaration nodes inside
        // <script> blocks under the same node-kind strings.
        "svelte" => Some("svelte"),
        // Pass D: Vue SFC — HTML parse for structure + TS re-parse + CSS @import.
        "vue" => Some("vue"),
        "java" => Some("java"),
        // C-17: Kotlin source + Gradle Kotlin DSL scripts both route through
        // KotlinGrammar. tree-sitter-kotlin-ng parses both surfaces.
        "kt" | "kts" => Some("kotlin"),
        "cs" => Some("csharp"),
        "rb" => Some("ruby"),
        "js" => Some("javascript"),
        "c" => Some("c"),
        "cpp" | "cc" | "cxx" => Some("cpp"),
        "scala" => Some("scala"),
        "php" => Some("php"),
        "swift" => Some("swift"),
        "sh" => Some("bash"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Import name cleaning (B-30 Pass C)
// ---------------------------------------------------------------------------

/// Clean a raw dkg_nodes.name import string to user-visible import path(s).
///
/// Grammars store the full raw AST text in `name` for import nodes:
///   Java:  "import com.example.Foo;"   -> ["com.example.Foo"]
///   C#:    "using App.Models;"         -> ["App.Models"]
///   Go:    `import "fmt"` (or grouped) -> ["fmt"] (or multiple paths)
///   TS/JS: `import { Foo } from './bar'` -> ["./bar"]
///   Scala: "import com.example.{A, B}" -> ["com.example.A", "com.example.B"]
///          "import com.example._"      -> [] (wildcard, skip)
///
/// Returns 0..N cleaned strings. Callers pass each to the resolver.
fn clean_import_name(raw: &str, lang: &str) -> Vec<String> {
    let trimmed = raw.trim();
    match lang {
        "java" | "kotlin" | "scala" => {
            // Java: "import com.example.Foo;" or "import static com.example.Util;"
            // Kotlin: "import com.example.Foo" or "import com.example.*"
            // Scala: "import com.example.Foo" or "import com.example.{A, B}"
            let after_import = trimmed
                .strip_prefix("import static ")
                .or_else(|| trimmed.strip_prefix("import "))
                .unwrap_or(trimmed)
                .trim()
                .trim_end_matches(';')
                .trim();

            if after_import.is_empty() {
                return Vec::new();
            }

            // Scala group import: "com.example.{Foo, Bar}"
            if let Some(brace_pos) = after_import.find('{') {
                let prefix = after_import[..brace_pos].trim_end_matches('.').trim();
                let inner = &after_import[brace_pos + 1..];
                let inner = inner.trim_end_matches('}').trim();
                // Wildcard inside braces: skip.
                if inner == "_" || inner == "*" {
                    return Vec::new();
                }
                return inner
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty() && *s != "_" && *s != "*")
                    .map(|s| format!("{prefix}.{s}"))
                    .collect();
            }

            // Wildcard: "com.example.*" or "com.example._"
            if after_import.ends_with(".*") || after_import.ends_with("._") {
                return Vec::new();
            }

            vec![after_import.to_string()]
        }

        "csharp" => {
            // C# using_directive: "using App.Models;" or "using static App.Util;"
            // namespace_declaration: just the namespace name (no "using" prefix).
            // Skip namespace declarations (they are not imports to resolve).
            if !trimmed.starts_with("using ") {
                return Vec::new();
            }
            let after_using = trimmed
                .strip_prefix("using static ")
                .or_else(|| trimmed.strip_prefix("using "))
                .unwrap_or(trimmed)
                .trim()
                .trim_end_matches(';')
                .trim();

            // Skip using aliases: "using Foo = App.Bar;"
            if after_using.contains(" = ") || after_using.contains('=') {
                return Vec::new();
            }

            if after_using.is_empty() {
                return Vec::new();
            }
            vec![after_using.to_string()]
        }

        "go" => {
            // Go import_declaration: raw node text.
            // Single: `import "github.com/foo/bar"` or `import _ "fmt"`
            // Grouped: `import (\n\t"fmt"\n\t"os"\n)`
            // Extract all double-quoted strings.
            let mut paths = Vec::new();
            let mut remaining = trimmed;
            while let Some(q1) = remaining.find('"') {
                remaining = &remaining[q1 + 1..];
                if let Some(q2) = remaining.find('"') {
                    let path = &remaining[..q2];
                    // Skip blank-identifier side-effect imports (still record the path).
                    if !path.is_empty() {
                        paths.push(path.to_string());
                    }
                    remaining = &remaining[q2 + 1..];
                } else {
                    break;
                }
            }
            paths
        }

        "typescript" | "tsx" | "javascript" | "svelte" => {
            // TypeScript/JS import_statement: `import { Foo } from './bar'`
            // or `import './bar'` (side-effect import).
            // Extract the `from '...'` or `from "..."` path, or bare string.
            let lower = trimmed.to_lowercase();
            if let Some(from_pos) = lower.rfind(" from ") {
                // " from " is 6 bytes (ASCII); skip past it.
                let after_from = trimmed[from_pos + " from ".len()..].trim();
                let path = after_from
                    .trim_start_matches('"')
                    .trim_start_matches('\'')
                    .trim_end_matches('"')
                    .trim_end_matches('\'')
                    .trim_end_matches(';');
                if !path.is_empty() {
                    return vec![path.to_string()];
                }
            }
            // Bare import (no `from`): `import './polyfill'` or `require('./foo')`
            // Try to find a quoted string. Use byte-safe indexing (import paths are ASCII).
            let dq = trimmed.find('"');
            let sq = trimmed.find('\'');
            let q1 = match (dq, sq) {
                (Some(d), Some(s)) => Some((d.min(s), if d < s { '"' } else { '\'' })),
                (Some(d), None) => Some((d, '"')),
                (None, Some(s)) => Some((s, '\'')),
                (None, None) => None,
            };
            if let Some((start, close)) = q1 {
                let after = &trimmed[start + 1..];
                if let Some(end) = after.find(close) {
                    let path = after[..end].trim();
                    if !path.is_empty() {
                        return vec![path.to_string()];
                    }
                }
            }
            Vec::new()
        }

        _ => Vec::new(),
    }
}

fn walk_source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if EXCLUDED_DIRS.contains(&name) {
                    continue;
                }
            }
            walk_source_files(&path, out);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if SOURCE_EXTENSIONS.contains(&ext) {
                out.push(path);
            }
        }
    }
}

#[cfg(test)]
mod grammar_tests {
    use super::*;

    fn parse_and_extract(grammar: &dyn LanguageGrammar, source: &str) -> Vec<DkgNode> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&grammar.tree_sitter_language()).expect("set language");
        let tree = parser.parse(source.as_bytes(), None).expect("parse");
        grammar.extract_nodes(&tree, source.as_bytes())
    }

    #[test]
    fn python_grammar_extracts_class_and_functions() {
        let src = r#"
class Foo:
    def bar(self):
        pass

def standalone(x: int) -> str:
    return str(x)
"#;
        let nodes = parse_and_extract(&PythonGrammar, src);
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Foo"), "missing class Foo, got: {:?}", names);
        assert!(names.contains(&"bar"), "missing method bar, got: {:?}", names);
        assert!(names.contains(&"standalone"), "missing function standalone, got: {:?}", names);
        // Verify kinds
        let foo = nodes.iter().find(|n| n.name == "Foo").expect("Foo node");
        assert_eq!(foo.kind, SymbolKind::Struct);
        let bar = nodes.iter().find(|n| n.name == "bar").expect("bar node");
        assert_eq!(bar.kind, SymbolKind::Function);
        // bar should be a child of Foo
        assert!(bar.parent_idx.is_some());
    }

    #[test]
    fn go_grammar_extracts_func_and_type() {
        let src = r#"
package main

import "fmt"

type Server struct {
    Port int
}

func (s *Server) Start() error {
    return nil
}

func main() {
    fmt.Println("hello")
}
"#;
        let nodes = parse_and_extract(&GoGrammar, src);
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"main"), "missing func main, got: {:?}", names);
        assert!(names.contains(&"Server"), "missing type Server, got: {:?}", names);
        assert!(names.contains(&"Start"), "missing method Start, got: {:?}", names);
        let server = nodes.iter().find(|n| n.name == "Server").expect("Server");
        assert_eq!(server.kind, SymbolKind::Struct);
    }

    #[test]
    fn typescript_grammar_extracts_class_and_interface() {
        let src = r#"
import { foo } from 'bar';

interface Config {
    port: number;
}

class App {
    constructor(private config: Config) {}

    start(): void {
        console.log("started");
    }
}

function helper(x: number): string {
    return x.toString();
}

enum Status { Active, Inactive }
"#;
        let nodes = parse_and_extract(&TypeScriptGrammar, src);
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Config"), "missing interface Config, got: {:?}", names);
        assert!(names.contains(&"App"), "missing class App, got: {:?}", names);
        assert!(names.contains(&"helper"), "missing function helper, got: {:?}", names);
        assert!(names.contains(&"Status"), "missing enum Status, got: {:?}", names);
        let config = nodes.iter().find(|n| n.name == "Config").expect("Config");
        assert_eq!(config.kind, SymbolKind::Trait);
    }

    #[test]
    fn java_grammar_extracts_class_and_methods() {
        let src = r#"
package com.example;

import java.util.List;

public class MyService {
    private int count;

    public MyService() {
        this.count = 0;
    }

    public List<String> process(String input) {
        return List.of(input);
    }
}

enum Color { RED, GREEN, BLUE }
"#;
        let nodes = parse_and_extract(&JavaGrammar, src);
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"MyService"), "missing class MyService, got: {:?}", names);
        assert!(names.contains(&"process"), "missing method process, got: {:?}", names);
        assert!(names.contains(&"Color"), "missing enum Color, got: {:?}", names);
        let svc = nodes.iter().find(|n| n.name == "MyService").expect("MyService");
        assert_eq!(svc.kind, SymbolKind::Struct);
    }

    #[test]
    fn csharp_grammar_extracts_class_and_namespace() {
        let src = r#"
using System;

namespace MyApp {
    public class Program {
        public static void Main(string[] args) {
            Console.WriteLine("Hello");
        }
    }

    public interface IService {
        void Run();
    }

    public enum Status { Active, Inactive }
}
"#;
        let nodes = parse_and_extract(&CSharpGrammar, src);
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"MyApp"), "missing namespace MyApp, got: {:?}", names);
        assert!(names.contains(&"Program"), "missing class Program, got: {:?}", names);
        assert!(names.contains(&"Main"), "missing method Main, got: {:?}", names);
        assert!(names.contains(&"IService"), "missing interface IService, got: {:?}", names);
        assert!(names.contains(&"Status"), "missing enum Status, got: {:?}", names);
    }

    #[test]
    fn ruby_grammar_extracts_class_and_methods() {
        let src = r#"
module Helpers
  class Greeter
    def greet(name)
      "Hello #{name}"
    end

    def self.default
      new
    end
  end
end
"#;
        let nodes = parse_and_extract(&RubyGrammar, src);
        let names: Vec<&str> = nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Helpers"), "missing module Helpers, got: {:?}", names);
        assert!(names.contains(&"Greeter"), "missing class Greeter, got: {:?}", names);
        assert!(names.contains(&"greet"), "missing method greet, got: {:?}", names);
        assert!(names.contains(&"default"), "missing singleton method default, got: {:?}", names);
        let helpers = nodes.iter().find(|n| n.name == "Helpers").expect("Helpers");
        assert_eq!(helpers.kind, SymbolKind::Module);
        let greeter = nodes.iter().find(|n| n.name == "Greeter").expect("Greeter");
        assert_eq!(greeter.kind, SymbolKind::Struct);
        // Greeter should be child of Helpers
        assert!(greeter.parent_idx.is_some());
    }

    #[test]
    fn ext_to_lang_id_mapping() {
        assert_eq!(ext_to_lang_id("py"), Some("python"));
        assert_eq!(ext_to_lang_id("go"), Some("go"));
        assert_eq!(ext_to_lang_id("ts"), Some("typescript"));
        assert_eq!(ext_to_lang_id("tsx"), Some("tsx"));
        assert_eq!(ext_to_lang_id("java"), Some("java"));
        assert_eq!(ext_to_lang_id("cs"), Some("csharp"));
        assert_eq!(ext_to_lang_id("rb"), Some("ruby"));
        assert_eq!(ext_to_lang_id("rs"), Some("rust"));
        assert_eq!(ext_to_lang_id("txt"), None);
    }
}
