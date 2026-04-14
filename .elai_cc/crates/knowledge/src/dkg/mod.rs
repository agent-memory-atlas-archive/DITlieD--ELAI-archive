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
        let langs: Vec<Box<dyn LanguageGrammar>> = vec![
            Box::new(RustGrammar),
            Box::new(PythonGrammar),
            Box::new(GoGrammar),
            Box::new(TypeScriptGrammar),
            Box::new(JavaGrammar),
            Box::new(CSharpGrammar),
            Box::new(RubyGrammar),
        ];
        for g in langs {
            grammars.insert(g.language_id(), g);
        }

        Ok(Self {
            conn,
            grammars,
            build_graph_called: false,
            index_version_path: index_version_path.map(|p| p.to_path_buf()),
            created_at,
        })
    }

    pub fn build_graph(&mut self, project_root: &Path) -> Result<DependencyGraph, Box<dyn std::error::Error>> {
        let mut source_files = Vec::new();
        walk_source_files(project_root, &mut source_files);
        source_files.sort();

        let mut files_parsed: usize = 0;
        let mut total_node_count: usize = 0;

        let mut parser = tree_sitter::Parser::new();

        for file_path in &source_files {
            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let lang_id = match ext_to_lang_id(ext) {
                Some(id) => id,
                None => { continue; }
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
    ) -> Result<crate::conventions::ConventionStats, Box<dyn std::error::Error>> {
        let readiness = self.dkg_readiness();
        if readiness != DkgReadiness::Ready {
            return Err(format!(
                "DKG not ready for convention extraction (status: {readiness:?})"
            )
            .into());
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
            for i in 0..node.child_count() as u32 {
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
            for i in 0..node.child_count() as u32 {
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
            for i in 0..node.child_count() as u32 {
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
            for i in 0..node.child_count() as u32 {
                if let Some(child) = node.child(i) {
                    if child.kind() == "variable_declaration" {
                        for j in 0..child.child_count() as u32 {
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
                let start_line = node.start_position().row as u32 + 1;
                let end_line = node.end_position().row as u32 + 1;
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

const EXCLUDED_DIRS: &[&str] = &["target", ".git", ".cargo", "node_modules", "__pycache__", ".venv", "vendor", "dist", "build"];

/// Extensions that have a registered grammar.
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "go", "ts", "tsx", "java", "cs", "rb",
];

pub fn ext_to_lang_id(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "go" => Some("go"),
        "ts" | "tsx" => Some("typescript"),
        "java" => Some("java"),
        "cs" => Some("csharp"),
        "rb" => Some("ruby"),
        _ => None,
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
        assert_eq!(ext_to_lang_id("tsx"), Some("typescript"));
        assert_eq!(ext_to_lang_id("java"), Some("java"));
        assert_eq!(ext_to_lang_id("cs"), Some("csharp"));
        assert_eq!(ext_to_lang_id("rb"), Some("ruby"));
        assert_eq!(ext_to_lang_id("rs"), Some("rust"));
        assert_eq!(ext_to_lang_id("txt"), None);
    }
}
