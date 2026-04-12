use std::collections::HashMap;
use std::path::{Path, PathBuf};

use blake3;
use rusqlite::Connection;

use crate::{
    DkgNode, DkgNodeInfo, DkgReadiness, DependencyGraph, LanguageGrammar, SymbolKind,
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
    build_graph_called: bool,
    index_version_path: Option<PathBuf>,
    created_at: i64,
}

impl DkgEngine {
    pub fn new(
        db_path: &Path,
        index_version_path: Option<&Path>,
        created_at: i64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(SCHEMA_SQL)?;

        conn.execute(
            "INSERT OR IGNORE INTO dkg_meta (key, value) VALUES ('schema_version', '1')",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO dkg_meta (key, value) VALUES ('created_at', ?1)",
            [created_at.to_string()],
        )?;

        if let Some(ivp) = index_version_path {
            let version_file = crate::index_version::current_version(created_at);
            crate::index_version::write(ivp, &version_file)?;
        }

        let mut grammars: HashMap<&'static str, Box<dyn LanguageGrammar>> = HashMap::new();
        let rust_grammar = RustGrammar;
        grammars.insert(rust_grammar.language_id(), Box::new(rust_grammar));

        Ok(Self {
            conn,
            grammars,
            build_graph_called: false,
            index_version_path: index_version_path.map(|p| p.to_path_buf()),
            created_at,
        })
    }

    pub fn build_graph(&mut self, project_root: &Path) -> Result<DependencyGraph, Box<dyn std::error::Error>> {
        let mut rs_files = Vec::new();
        walk_rs_files(project_root, &mut rs_files);
        rs_files.sort();

        let mut files_parsed: usize = 0;
        let mut total_node_count: usize = 0;

        let mut parser = tree_sitter::Parser::new();

        for file_path in &rs_files {
            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let lang_id = match ext {
                "rs" => "rust",
                _ => { continue; }
            };
            let grammar = match self.grammars.get(lang_id) {
                Some(g) => g,
                None => { continue; }
            };
            parser.set_language(&grammar.tree_sitter_language())?;
            let source_bytes = std::fs::read(file_path)?;
            let file_path_str = file_path.to_string_lossy().to_string();

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
                    total_node_count += count as usize;
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
                    "INSERT INTO dkg_nodes (file_path, name, kind, signature, start_line, end_line, concept_stale) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
                    rusqlite::params![
                        file_path_str,
                        node.name,
                        kind_str,
                        node.signature,
                        node.start_line,
                        node.end_line,
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

        // Generate cross-file edges: "imports" and "implements"
        self.generate_cross_edges()?;

        let edge_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM dkg_edges",
            [],
            |row| row.get(0),
        )?;

        self.build_graph_called = true;

        // Write index_version.toml if path is set
        if let Some(ref ivp) = self.index_version_path {
            let version_file = crate::index_version::current_version(self.created_at);
            crate::index_version::write(ivp, &version_file)?;
        }

        Ok(DependencyGraph {
            node_count: total_node_count,
            edge_count: edge_count as usize,
            files_parsed,
        })
    }

    fn generate_cross_edges(&self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn edges(&self) -> Result<EdgeList, Box<dyn std::error::Error>> {
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
            Ok(_) if self.build_graph_called => DkgReadiness::Empty,
            Ok(_) => DkgReadiness::Unavailable,
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

    fn extract_nodes(&self, tree: &tree_sitter::Tree, source: &[u8]) -> Vec<DkgNode> {
        let mut nodes = Vec::new();
        let mut cursor = tree.root_node().walk();
        extract_recursive(&mut cursor, source, &mut nodes, None);
        nodes
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
            let start_line = node.start_position().row as u32 + 1;
            let end_line = node.end_position().row as u32 + 1;

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

const EXCLUDED_DIRS: &[&str] = &["target", ".git", ".cargo", "node_modules"];

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
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
            walk_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}
