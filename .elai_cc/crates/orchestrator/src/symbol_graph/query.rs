//! E-40 — query helpers over `SymbolGraphCache`.
//!
//! Four canonical symbol-graph queries:
//! - [`find_defs`]   — every node whose `name` matches.
//! - [`find_refs`]   — every node whose `calls` list includes a target def
//!   (uses the cache's reverse-call index — O(1) lookup per id).
//! - [`find_callers`] — every node that calls (directly) any node named
//!   `symbol`.
//! - [`find_callees`] — every node called (directly) by any node named
//!   `symbol`.
//!
//! All four return [`SymbolMatch`] rows so callers can render the same
//! shape across queries.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::cache::{SymbolGraphCache, SymbolNode};

/// Error surface for query helpers.
#[derive(Debug, Error)]
pub enum QueryError {
    /// Caller passed an empty symbol name. All four query functions
    /// reject that explicitly so the dashboard does not surface huge
    /// result sets on a blank input.
    #[error("symbol_graph query: symbol name must be non-empty")]
    EmptySymbol,
}

/// A single match row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolMatch {
    pub id: String,
    pub name: String,
    pub file: String,
    pub line: u32,
    pub kind: String,
}

impl SymbolMatch {
    fn from_node(node: &SymbolNode) -> Self {
        Self {
            id: node.id.clone(),
            name: node.name.clone(),
            file: node.file.clone(),
            line: node.line,
            kind: node.kind.clone(),
        }
    }
}

/// All definitions whose name matches `symbol`.
pub fn find_defs(
    cache: &mut SymbolGraphCache,
    symbol: &str,
) -> Result<Vec<SymbolMatch>, QueryError> {
    if symbol.is_empty() {
        return Err(QueryError::EmptySymbol);
    }
    let ids = cache.ids_by_name(symbol);
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(node) = cache.get_node(&id) {
            out.push(SymbolMatch::from_node(node));
        }
    }
    Ok(out)
}

/// All references — callers of any node whose name matches `symbol`.
///
/// Uses the reverse-call index, so this is O(|defs(symbol)| + |callers|)
/// per call.
pub fn find_refs(
    cache: &mut SymbolGraphCache,
    symbol: &str,
) -> Result<Vec<SymbolMatch>, QueryError> {
    if symbol.is_empty() {
        return Err(QueryError::EmptySymbol);
    }
    // Snapshot the def ids first to avoid the borrow checker complaining
    // about an `ids_by_name` followed by `callers_of` on the same `&mut`.
    let def_ids = cache.ids_by_name(symbol);
    let mut caller_ids: Vec<String> = Vec::new();
    for id in &def_ids {
        caller_ids.extend(cache.callers_of(id));
    }
    let mut out = Vec::with_capacity(caller_ids.len());
    for id in caller_ids {
        if let Some(node) = cache.get_node(&id) {
            out.push(SymbolMatch::from_node(node));
        }
    }
    Ok(out)
}

/// All direct callers of `symbol` (synonym of [`find_refs`] kept as a
/// dedicated entry point so the CLI surface mirrors the spec verbatim).
pub fn find_callers(
    cache: &mut SymbolGraphCache,
    symbol: &str,
) -> Result<Vec<SymbolMatch>, QueryError> {
    find_refs(cache, symbol)
}

/// All direct callees of any node whose name matches `symbol`.
pub fn find_callees(
    cache: &mut SymbolGraphCache,
    symbol: &str,
) -> Result<Vec<SymbolMatch>, QueryError> {
    if symbol.is_empty() {
        return Err(QueryError::EmptySymbol);
    }
    let def_ids = cache.ids_by_name(symbol);
    let mut callee_ids: Vec<String> = Vec::new();
    for id in &def_ids {
        callee_ids.extend(cache.callees_of(id));
    }
    let mut out = Vec::with_capacity(callee_ids.len());
    for id in callee_ids {
        if let Some(node) = cache.get_node(&id) {
            out.push(SymbolMatch::from_node(node));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol_graph::cache::SymbolNode;

    fn build_cache() -> SymbolGraphCache {
        let mut cache = SymbolGraphCache::empty_with_capacity(16).expect("empty");
        cache.index_node(SymbolNode {
            id: "src/a.rs:foo:10".into(),
            name: "foo".into(),
            file: "src/a.rs".into(),
            line: 10,
            kind: "function".into(),
            calls: vec!["src/b.rs:bar:20".into()],
        });
        cache.index_node(SymbolNode {
            id: "src/b.rs:bar:20".into(),
            name: "bar".into(),
            file: "src/b.rs".into(),
            line: 20,
            kind: "function".into(),
            calls: vec![],
        });
        cache.index_node(SymbolNode {
            id: "src/c.rs:baz:30".into(),
            name: "baz".into(),
            file: "src/c.rs".into(),
            line: 30,
            kind: "function".into(),
            calls: vec!["src/b.rs:bar:20".into()],
        });
        cache
    }

    #[test]
    fn find_defs_returns_named_nodes() {
        let mut cache = build_cache();
        let defs = find_defs(&mut cache, "bar").expect("defs");
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "bar");
    }

    #[test]
    fn find_refs_returns_callers() {
        let mut cache = build_cache();
        let refs = find_refs(&mut cache, "bar").expect("refs");
        let names: Vec<_> = refs.iter().map(|m| m.name.clone()).collect();
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"baz".to_string()));
    }

    #[test]
    fn find_callees_returns_targets() {
        let mut cache = build_cache();
        let callees = find_callees(&mut cache, "foo").expect("callees");
        assert_eq!(callees.len(), 1);
        assert_eq!(callees[0].name, "bar");
    }

    #[test]
    fn empty_symbol_rejected() {
        let mut cache = build_cache();
        assert!(matches!(
            find_defs(&mut cache, ""),
            Err(QueryError::EmptySymbol)
        ));
    }
}
