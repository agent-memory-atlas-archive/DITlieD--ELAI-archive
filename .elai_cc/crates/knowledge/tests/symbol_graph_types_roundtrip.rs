// Integration tests: postcard roundtrip for all symbol_graph types.
// Tests must compile and pass after impl is written.

use elai_knowledge::symbol_graph::types::{
    CallSite, Confidence, EntryPoint, SymbolEdge, SymbolGraphFilePayload, SymbolGraphMeta,
    SymbolKind, SymbolNode, SymbolRef,
};

// ---------------------------------------------------------------------------
// Helper builders
// ---------------------------------------------------------------------------

fn sample_symbol_node() -> SymbolNode {
    SymbolNode {
        id: "src/lib.rs::my_function".to_owned(),
        name: "my_function".to_owned(),
        qualified_name: "crate::my_function".to_owned(),
        kind: SymbolKind::Function,
        file: "src/lib.rs".to_owned(),
        line: 10,
        end_line: 25,
        language: "rust".to_owned(),
    }
}

fn sample_symbol_edge_unique() -> SymbolEdge {
    SymbolEdge {
        caller_id: "src/lib.rs::my_function".to_owned(),
        callee_name: "helper".to_owned(),
        callee_candidates: vec!["src/util.rs::helper".to_owned()],
        confidence: Confidence::Unique,
        call_site: CallSite { file: "src/lib.rs".to_owned(), line: 15 },
    }
}

fn sample_symbol_edge_multiple_candidates() -> SymbolEdge {
    SymbolEdge {
        caller_id: "src/lib.rs::my_function".to_owned(),
        callee_name: "process".to_owned(),
        callee_candidates: vec![
            "src/a.rs::process".to_owned(),
            "src/b.rs::process".to_owned(),
            "src/c.rs::process".to_owned(),
        ],
        confidence: Confidence::MultipleCandidates,
        call_site: CallSite { file: "src/lib.rs".to_owned(), line: 20 },
    }
}

fn sample_meta() -> SymbolGraphMeta {
    SymbolGraphMeta {
        project_id: "test-project".to_owned(),
        symbol_count: 42,
        edge_count: 17,
        file_count: 5,
        unresolved_edge_pct: 12.0,
        built_at: 1_700_000_000_000,
        schema_version: 3,
    }
}

fn sample_file_payload() -> SymbolGraphFilePayload {
    SymbolGraphFilePayload {
        file: "src/lib.rs".to_owned(),
        language: "rust".to_owned(),
        content_hash: "abc123".to_owned(),
        symbols: vec![sample_symbol_node()],
        outgoing_calls: vec![sample_symbol_edge_unique()],
    }
}

// ---------------------------------------------------------------------------
// Roundtrip helpers
// ---------------------------------------------------------------------------

fn roundtrip<T>(value: &T) -> T
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + PartialEq,
{
    let encoded = postcard::to_allocvec(value).expect("postcard serialize failed");
    postcard::from_bytes(&encoded).expect("postcard deserialize failed")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn symbol_node_roundtrip() {
    let node = sample_symbol_node();
    let recovered: SymbolNode = roundtrip(&node);
    assert_eq!(node, recovered);
}

#[test]
fn symbol_edge_unique_roundtrip() {
    let edge = sample_symbol_edge_unique();
    let recovered: SymbolEdge = roundtrip(&edge);
    assert_eq!(edge, recovered);
}

#[test]
fn symbol_edge_multiple_candidates_roundtrip() {
    let edge = sample_symbol_edge_multiple_candidates();
    assert_eq!(edge.callee_candidates.len(), 3);
    let recovered: SymbolEdge = roundtrip(&edge);
    assert_eq!(edge, recovered);
}

#[test]
fn symbol_graph_meta_roundtrip() {
    let meta = sample_meta();
    let recovered: SymbolGraphMeta = roundtrip(&meta);
    assert_eq!(meta, recovered);
}

#[test]
fn symbol_graph_file_payload_roundtrip() {
    let payload = sample_file_payload();
    let recovered: SymbolGraphFilePayload = roundtrip(&payload);
    assert_eq!(payload, recovered);
}

#[test]
fn all_confidence_variants_roundtrip() {
    let variants = [
        Confidence::Local,
        Confidence::Unique,
        Confidence::MultipleCandidates,
        Confidence::Unresolved,
    ];
    for &conf in &variants {
        let recovered: Confidence = roundtrip(&conf);
        assert_eq!(conf, recovered, "Confidence::{conf:?} roundtrip failed");
    }
}

#[test]
fn all_symbol_kind_variants_roundtrip() {
    let variants = [
        SymbolKind::Function,
        SymbolKind::Method,
        SymbolKind::Class,
        SymbolKind::Struct,
        SymbolKind::Trait,
        SymbolKind::Module,
        SymbolKind::Enum,
        SymbolKind::Interface,
        SymbolKind::Type,
    ];
    for &kind in &variants {
        let recovered: SymbolKind = roundtrip(&kind);
        assert_eq!(kind, recovered, "SymbolKind::{kind:?} roundtrip failed");
    }
}

#[test]
fn symbol_ref_roundtrip() {
    let sr = SymbolRef {
        file: "src/a.rs".to_owned(),
        id: "src/a.rs::foo".to_owned(),
    };
    let recovered: SymbolRef = roundtrip(&sr);
    assert_eq!(sr, recovered);
}

#[test]
fn entry_point_roundtrip() {
    let ep = EntryPoint {
        id: "src/main.rs::main".to_owned(),
        name: "main".to_owned(),
        file: "src/main.rs".to_owned(),
        line: Some(1),
        reason: "function named 'main' in binary crate root".to_owned(),
    };
    let recovered: EntryPoint = roundtrip(&ep);
    assert_eq!(ep, recovered);
}

#[test]
fn entry_point_no_line_roundtrip() {
    let ep = EntryPoint {
        id: "src/lib.rs::lib_module".to_owned(),
        name: "lib_module".to_owned(),
        file: "src/lib.rs".to_owned(),
        line: None,
        reason: "synthetic module entry".to_owned(),
    };
    let recovered: EntryPoint = roundtrip(&ep);
    assert_eq!(ep, recovered);
}
