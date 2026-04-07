use std::collections::HashMap;
use std::path::Path;

use rusqlite::Connection;

use crate::{
    DkgNode, DkgNodeInfo, DkgReadiness, LanguageGrammar, SymbolKind,
};

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

CREATE INDEX IF NOT EXISTS idx_nodes_name   ON dkg_nodes(name);
CREATE INDEX IF NOT EXISTS idx_nodes_file   ON dkg_nodes(file_path);
CREATE INDEX IF NOT EXISTS idx_nodes_kind   ON dkg_nodes(kind);
CREATE INDEX IF NOT EXISTS idx_files_status ON dkg_files(parse_status);
";

// ---------------------------------------------------------------------------
// Edge-type constants
// ---------------------------------------------------------------------------

pub const EDGE_IMPORTS: &str = "imports";
pub const EDGE_CONTAINS: &str = "contains";
pub const EDGE_IMPLEMENTS: &str = "implements";

// ---------------------------------------------------------------------------
// DkgEngine
// ---------------------------------------------------------------------------

pub struct DkgEngine {
    conn: Connection,
    grammars: HashMap<&'static str, Box<dyn LanguageGrammar>>,
}

impl DkgEngine {
    /// Open (or create) the DKG SQLite database at `db_path`.
    ///
    /// Executes WAL journal mode and foreign key pragmas, then creates the
    /// schema tables if they do not exist. Inserts initial metadata rows.
    /// If `index_version_path` is `Some`, writes an `index_version.toml` file
    /// with the 5 required fields (caller provides `created_at` timestamp).
    pub fn new(
        db_path: &Path,
        index_version_path: Option<&Path>,
        created_at: i64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(SCHEMA_SQL)?;

        // Insert initial metadata (ignore if already present).
        conn.execute(
            "INSERT OR IGNORE INTO dkg_meta (key, value) VALUES ('schema_version', '1')",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO dkg_meta (key, value) VALUES ('created_at', ?1)",
            [created_at.to_string()],
        )?;

        if let Some(ivp) = index_version_path {
            let content = format!(
                "version = 1\n\
                 dkg_schema_version = 1\n\
                 vector_dimensions = {{ code = 0, text = 0 }}\n\
                 bm25_format = \"\"\n\
                 created_at = {created_at}\n"
            );
            std::fs::write(ivp, content)?;
        }

        let mut grammars: HashMap<&'static str, Box<dyn LanguageGrammar>> = HashMap::new();
        let rust_grammar = RustGrammar;
        grammars.insert(rust_grammar.language_id(), Box::new(rust_grammar));

        Ok(Self { conn, grammars })
    }

    pub fn build_graph(&mut self, project_root: &Path) -> Result<crate::DependencyGraph, Box<dyn std::error::Error>> {
        todo!("1A.3 stub — implementation in 1A.4")
    }

    pub fn nodes(&self) -> Result<Vec<DkgNodeInfo>, Box<dyn std::error::Error>> {
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

    pub fn edges(&self) -> Result<Vec<(i64, i64, String)>, Box<dyn std::error::Error>> {
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

    pub fn node_by_id(&self, id: i64) -> Result<Option<DkgNodeInfo>, Box<dyn std::error::Error>> {
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

    pub fn outbound_edges(&self, node_id: i64) -> Result<Vec<(i64, String)>, Box<dyn std::error::Error>> {
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

    pub fn inbound_edges(&self, node_id: i64) -> Result<Vec<(i64, String)>, Box<dyn std::error::Error>> {
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

    pub fn failed_files(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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

    pub fn dkg_readiness(&self) -> DkgReadiness {
        let count: Result<i64, _> = self
            .conn
            .query_row("SELECT COUNT(*) FROM dkg_nodes", [], |row| row.get(0));
        match count {
            Ok(n) if n > 0 => DkgReadiness::Ready,
            Ok(_) => DkgReadiness::Empty,
            Err(_) => DkgReadiness::Unavailable,
        }
    }
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

    fn extract_nodes(&self, _tree: &tree_sitter::Tree, _source: &[u8]) -> Vec<DkgNode> {
        Vec::new()
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
