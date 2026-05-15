// B-29 Phase 2 — symbol extraction + call-site detection.
//
// Implements extract_symbols_and_calls (per-language dispatch over tree-sitter),
// raw_calls_to_unresolved_edges (pre-resolution stub), and the internal
// DkgKind -> GraphKind mapping.
//
// Architecture:
//   1. Parse source via grammar.tree_sitter_language().
//   2. Call grammar.extract_nodes() to get Vec<DkgNode> (symbol definitions only).
//   3. Map DkgNode -> SymbolNode (kind translation, id construction).
//   4. Walk tree a second time to collect call-expression nodes per language.
//   5. Attribute each call to its enclosing SymbolNode (by line range).
//   6. Return ExtractedSymbols { symbols, raw_calls }.

use serde::{Deserialize, Serialize};
use tracing::warn;

use super::types::{CallSite, Confidence, SymbolEdge, SymbolNode};
use super::types::SymbolKind as GraphKind;
use crate::LanguageGrammar;
use crate::SymbolKind as DkgKind;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A call site detected during extraction — pre-resolution (callee name only,
/// no candidate lookup yet).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawCall {
    /// `SymbolNode::id` of the symbol that contains this call.
    pub caller_id: String,
    /// Raw callee name as it appears at the call site.
    pub callee_name: String,
    /// Location of the call expression.
    pub call_site: CallSite,
}

/// Output of `extract_symbols_and_calls`.
#[derive(Debug, Clone)]
pub struct ExtractedSymbols {
    /// All symbol definitions in the file. `symbols[0]` is always the
    /// synthetic `<module>` symbol that anchors unattributed calls.
    pub symbols: Vec<SymbolNode>,
    /// All call sites detected in the file (pre-resolution).
    pub raw_calls: Vec<RawCall>,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Extract symbol definitions and call sites from `source`.
///
/// `grammar` provides the tree-sitter language and DKG symbol extraction.
/// `file` is the path string embedded into every produced node/call.
///
/// Always succeeds — on parse failure returns just the synthetic `<module>` symbol
/// and no raw calls, logging a warning via `tracing::warn`.
pub fn extract_symbols_and_calls(
    source: &str,
    grammar: &dyn LanguageGrammar,
    file: &str,
) -> ExtractedSymbols {
    let source_bytes = source.as_bytes();
    let lang_id = grammar.language_id();
    let total_lines = source.lines().count().max(1) as u32;

    // Build synthetic <module> symbol — always symbols[0].
    let module_id = format!("{file}::<module>#1");
    let module_symbol = SymbolNode {
        id: module_id.clone(),
        name: "<module>".to_string(),
        qualified_name: "<module>".to_string(),
        kind: GraphKind::Module,
        file: file.to_string(),
        line: 1,
        end_line: total_lines,
        language: lang_id.to_string(),
    };

    // Parse source.
    let mut parser = tree_sitter::Parser::new();
    if let Err(e) = parser.set_language(&grammar.tree_sitter_language()) {
        warn!("B29 extraction: set_language failed for {lang_id}: {e}");
        return ExtractedSymbols {
            symbols: vec![module_symbol],
            raw_calls: vec![],
        };
    }

    let tree = match parser.parse(source_bytes, None) {
        Some(t) => t,
        None => {
            warn!("B29 extraction: parse returned None for {file}");
            return ExtractedSymbols {
                symbols: vec![module_symbol],
                raw_calls: vec![],
            };
        }
    };

    // -----------------------------------------------------------------------
    // Pass 1 — symbol definitions via grammar.extract_nodes().
    // -----------------------------------------------------------------------
    let dkg_nodes = grammar.extract_nodes(&tree, source_bytes);
    let mut symbols: Vec<SymbolNode> = Vec::with_capacity(dkg_nodes.len() + 1);
    symbols.push(module_symbol);

    for node in &dkg_nodes {
        if let Some(graph_kind) = map_dkg_kind_to_graph_kind(node.kind) {
            // Disambiguate same-name symbols within the same file by suffixing
            // start_line. Required for languages where module-level fn `bar`
            // and a method `bar` can coexist (Python, TS, Ruby).
            // Stable across re-extractions of unchanged source.
            let id = format!("{file}::{name}#L{line}", name = node.name, line = node.start_line);
            let sym = SymbolNode {
                id,
                name: node.name.clone(),
                qualified_name: node.name.clone(),
                kind: graph_kind,
                file: file.to_string(),
                line: node.start_line,
                end_line: node.end_line,
                language: lang_id.to_string(),
            };
            symbols.push(sym);
        }
    }

    // -----------------------------------------------------------------------
    // Pass 2 — call-site detection via a second tree-sitter cursor walk.
    // -----------------------------------------------------------------------

    // Build a sorted scope table: (start_line, end_line, symbol_id).
    // Used to attribute each call to the deepest enclosing symbol.
    let mut scopes: Vec<(u32, u32, String)> = symbols
        .iter()
        .filter(|s| s.kind != GraphKind::Module) // exclude the module catch-all
        .map(|s| (s.line, s.end_line, s.id.clone()))
        .collect();
    // Sort by start line; ties broken by end_line descending (innermost last).
    scopes.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));

    let call_node_kinds = call_node_kinds_for(lang_id);
    let raw_calls = collect_call_sites(
        &tree,
        source_bytes,
        file,
        lang_id,
        &call_node_kinds,
        &scopes,
        &module_id,
    );

    ExtractedSymbols { symbols, raw_calls }
}

/// Convert `RawCall` slice to `SymbolEdge` slice with `Confidence::Unresolved`
/// and empty `callee_candidates`. These edges are enriched by `resolution.rs`.
pub fn raw_calls_to_unresolved_edges(raw_calls: &[RawCall]) -> Vec<SymbolEdge> {
    raw_calls
        .iter()
        .map(|rc| SymbolEdge {
            caller_id: rc.caller_id.clone(),
            callee_name: rc.callee_name.clone(),
            callee_candidates: vec![],
            confidence: Confidence::Unresolved,
            call_site: rc.call_site.clone(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Private — kind mapping
// ---------------------------------------------------------------------------

/// Map a DKG `SymbolKind` to its call-graph `SymbolKind` equivalent.
/// Returns `None` for kinds that have no direct call-graph analog (Impl, Const,
/// Static — these are skipped during SymbolNode construction).
fn map_dkg_kind_to_graph_kind(k: DkgKind) -> Option<GraphKind> {
    match k {
        DkgKind::Function => Some(GraphKind::Function),
        DkgKind::Struct => Some(GraphKind::Struct),
        DkgKind::Enum => Some(GraphKind::Enum),
        DkgKind::Trait => Some(GraphKind::Trait),
        DkgKind::Module => Some(GraphKind::Module),
        DkgKind::TypeAlias => Some(GraphKind::Type),
        DkgKind::Macro => Some(GraphKind::Function), // macros are callable
        // Impl blocks are containers, not callable symbols; Methods emitted via DkgNode children.
        DkgKind::Impl => None,
        // Constants and statics are not call targets.
        DkgKind::Const | DkgKind::Static => None,
    }
}

// ---------------------------------------------------------------------------
// Private — per-language call node kinds
// ---------------------------------------------------------------------------

/// Returns the tree-sitter node kinds that represent call expressions for
/// the given language.
fn call_node_kinds_for(lang_id: &str) -> Vec<&'static str> {
    match lang_id {
        "rust" => vec!["call_expression", "macro_invocation"],
        "python" | "ruby" => vec!["call"],
        "typescript" | "tsx" | "svelte" | "go" => vec!["call_expression"],
        "java" => vec!["method_invocation"],
        "csharp" => vec!["invocation_expression"],
        // Unsupported / unknown: return empty — no calls will be emitted.
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Private — cursor walk for call sites
// ---------------------------------------------------------------------------

/// Walk the tree and collect all nodes whose kind is in `call_kinds`.
/// For each, extract the callee name and attribute to the enclosing scope.
fn collect_call_sites(
    tree: &tree_sitter::Tree,
    source: &[u8],
    file: &str,
    lang_id: &str,
    call_kinds: &[&str],
    scopes: &[(u32, u32, String)],
    module_id: &str,
) -> Vec<RawCall> {
    if call_kinds.is_empty() {
        return vec![];
    }

    let mut raw_calls = Vec::new();
    let mut cursor = tree.root_node().walk();

    // Iterative DFS.
    let mut visited_children = false;
    loop {
        let node = cursor.node();

        if !visited_children {
            let kind = node.kind();
            if call_kinds.contains(&kind) {
                let line = node.start_position().row as u32 + 1; // 1-based
                if let Some(callee_name) = extract_callee_name(node, source, lang_id) {
                    let caller_id = find_enclosing_caller(line, scopes, module_id);
                    raw_calls.push(RawCall {
                        caller_id,
                        callee_name,
                        call_site: CallSite {
                            file: file.to_string(),
                            line,
                        },
                    });
                }
                // Do NOT descend into call nodes — inner calls will be visited
                // when the cursor reaches their subtree children naturally.
                // We still descend to allow nested call extraction.
            }

            if cursor.goto_first_child() {
                visited_children = false;
            } else {
                visited_children = true;
            }
        } else if cursor.goto_next_sibling() {
            visited_children = false;
        } else if cursor.goto_parent() {
            visited_children = true;
        } else {
            break;
        }
    }

    raw_calls
}

/// Find the id of the deepest scope frame that covers `call_line`.
/// Falls back to `module_id` if no function/method scope covers the call.
fn find_enclosing_caller(call_line: u32, scopes: &[(u32, u32, String)], module_id: &str) -> String {
    // Scopes are pre-sorted by (start_line asc, end_line desc) so when ranges
    // overlap, the narrower (deeper) scope is encountered later. Pick the scope
    // with the smallest span that contains call_line.
    let mut best: Option<&(u32, u32, String)> = None;
    for scope in scopes {
        if scope.0 <= call_line && call_line <= scope.1 {
            // Prefer the narrower (innermost) scope.
            match best {
                None => best = Some(scope),
                Some(prev) => {
                    let prev_span = prev.1 - prev.0;
                    let cur_span = scope.1 - scope.0;
                    if cur_span <= prev_span {
                        best = Some(scope);
                    }
                }
            }
        }
    }
    best.map(|s| s.2.clone()).unwrap_or_else(|| module_id.to_string())
}

/// Extract the callee name from a call node.
///
/// Strategy (language-aware):
/// - Rust `call_expression`: field "function" -> get identifier text.
/// - Rust `macro_invocation`: field "macro" -> identifier text (before "!").
/// - Python/Ruby `call`: field "function" -> text of first identifier leaf.
/// - TypeScript/Go/C# `call_expression` / `invocation_expression`: field
///   "function" (TS/Go) or "expression" (C#) -> last identifier segment.
/// - Java `method_invocation`: field "name" -> text.
///
/// Falls back to scanning the first identifier descendant.
/// Returns `None` if no valid identifier found or name is empty.
fn extract_callee_name(
    node: tree_sitter::Node,
    source: &[u8],
    lang_id: &str,
) -> Option<String> {
    let name = match lang_id {
        "java" => {
            // Java method_invocation: field "name" holds the method name directly.
            node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
                .map(|s| s.to_string())
        }
        "rust" if node.kind() == "macro_invocation" => {
            // macro_invocation: field "macro" is the path; take last segment.
            node.child_by_field_name("macro")
                .and_then(|n| n.utf8_text(source).ok())
                .map(|s| last_identifier_segment(s))
        }
        "csharp" => {
            // invocation_expression: field "expression" -> last identifier.
            node.child_by_field_name("expression")
                .and_then(|n| n.utf8_text(source).ok())
                .map(|s| last_identifier_segment(s))
                .or_else(|| first_identifier_descendant(node, source))
        }
        _ => {
            // For rust call_expression, python/ruby call, ts/go call_expression:
            // field "function" (rust/ts/go/python) holds the callee expression.
            node.child_by_field_name("function")
                .and_then(|n| n.utf8_text(source).ok())
                .map(|s| last_identifier_segment(s))
                .or_else(|| first_identifier_descendant(node, source))
        }
    };

    // Validate: must be a non-empty identifier (letters/digits/underscores/dollar).
    name.and_then(|n| {
        let trimmed = n.trim().to_string();
        if trimmed.is_empty() {
            return None;
        }
        // Skip numeric or operator-like names.
        let first = trimmed.chars().next()?;
        if first.is_alphabetic() || first == '_' || first == '$' {
            Some(trimmed)
        } else {
            None
        }
    })
}

/// Return the last dot-separated or `::` segment of a path-like string.
/// `"fmt.Println"` -> `"Println"`, `"std::vec::Vec"` -> `"Vec"`.
fn last_identifier_segment(s: &str) -> String {
    s.split(|c| c == '.' || c == ':')
        .filter(|p| !p.is_empty())
        .last()
        .map(|p| {
            // Strip trailing generic brackets or parentheses.
            p.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
                .to_string()
        })
        .unwrap_or_else(|| s.to_string())
}

/// Walk descendants of `node` and return the text of the first identifier
/// leaf found. Returns `None` if none exists.
fn first_identifier_descendant(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" || n.kind() == "type_identifier" {
            return n.utf8_text(source).ok().map(|s| s.to_string());
        }
        // Push children in reverse so first child is processed first.
        let child_count = n.child_count();
        for i in (0..child_count).rev() {
            if let Some(child) = n.child(i as u32) {
                stack.push(child);
            }
        }
    }
    None
}
