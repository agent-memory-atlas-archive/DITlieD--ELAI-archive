// B-29 Phase 2 — resolution tests.
//
// Tests for resolve_call_sites (4-tier strategy) and compute_unresolved_pct.
//
// NOTE: These tests are FAILING until resolution.rs is implemented.

use std::collections::BTreeMap;

use elai_knowledge::symbol_graph::resolution::{compute_unresolved_pct, resolve_call_sites};
use elai_knowledge::symbol_graph::types::{
    CallSite, Confidence, SymbolEdge, SymbolKind, SymbolNode,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_symbol(id: &str, name: &str, file: &str) -> SymbolNode {
    SymbolNode {
        id: id.to_string(),
        name: name.to_string(),
        qualified_name: name.to_string(),
        kind: SymbolKind::Function,
        file: file.to_string(),
        line: 1,
        end_line: 5,
        language: "rust".to_string(),
    }
}

fn make_edge(caller_id: &str, callee_name: &str, file: &str) -> SymbolEdge {
    SymbolEdge {
        caller_id: caller_id.to_string(),
        callee_name: callee_name.to_string(),
        callee_candidates: vec![],
        confidence: Confidence::Unresolved,
        call_site: CallSite {
            file: file.to_string(),
            line: 3,
        },
    }
}

// ---------------------------------------------------------------------------
// Fixture 1 — Local: file A defines foo and bar; bar calls foo.
// After resolve, confidence == Local, callee_candidates == [A::foo].
// ---------------------------------------------------------------------------

#[test]
fn resolution_local_same_file() {
    let file_a = "a.rs";

    let foo = make_symbol("a.rs::foo", "foo", file_a);
    let bar = make_symbol("a.rs::bar", "bar", file_a);

    let mut symbols_by_file: BTreeMap<String, Vec<SymbolNode>> = BTreeMap::new();
    symbols_by_file.insert(file_a.to_string(), vec![foo.clone(), bar.clone()]);

    let edge = make_edge("a.rs::bar", "foo", file_a);
    let mut outgoing: BTreeMap<String, Vec<SymbolEdge>> = BTreeMap::new();
    outgoing.insert(file_a.to_string(), vec![edge]);

    let deps_by_file: BTreeMap<String, Vec<String>> = BTreeMap::new();

    resolve_call_sites(&deps_by_file, &symbols_by_file, &mut outgoing);

    let edges = &outgoing[file_a];
    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].confidence,
        Confidence::Local,
        "same-file call must be Local"
    );
    assert_eq!(
        edges[0].callee_candidates,
        vec!["a.rs::foo".to_string()],
        "callee_candidates must contain the local symbol id"
    );
}

// ---------------------------------------------------------------------------
// Fixture 2 — Unique imported: file A imports B; B defines foo; A calls foo.
// After resolve, confidence == Unique, callee_candidates == [B::foo].
// ---------------------------------------------------------------------------

#[test]
fn resolution_unique_imported() {
    let file_a = "a.rs";
    let file_b = "b.rs";

    let foo_b = make_symbol("b.rs::foo", "foo", file_b);
    let bar_a = make_symbol("a.rs::bar", "bar", file_a);

    let mut symbols_by_file: BTreeMap<String, Vec<SymbolNode>> = BTreeMap::new();
    symbols_by_file.insert(file_a.to_string(), vec![bar_a.clone()]);
    symbols_by_file.insert(file_b.to_string(), vec![foo_b.clone()]);

    let edge = make_edge("a.rs::bar", "foo", file_a);
    let mut outgoing: BTreeMap<String, Vec<SymbolEdge>> = BTreeMap::new();
    outgoing.insert(file_a.to_string(), vec![edge]);

    let mut deps_by_file: BTreeMap<String, Vec<String>> = BTreeMap::new();
    deps_by_file.insert(file_a.to_string(), vec![file_b.to_string()]);

    resolve_call_sites(&deps_by_file, &symbols_by_file, &mut outgoing);

    let edges = &outgoing[file_a];
    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].confidence,
        Confidence::Unique,
        "single imported match must be Unique"
    );
    assert_eq!(
        edges[0].callee_candidates,
        vec!["b.rs::foo".to_string()],
        "callee_candidates must contain b.rs::foo"
    );
}

// ---------------------------------------------------------------------------
// Fixture 3 — MultipleCandidates: A imports B and C; both define foo; A calls foo.
// After resolve, confidence == MultipleCandidates, callee_candidates contains both ids (sorted).
// ---------------------------------------------------------------------------

#[test]
fn resolution_multiple_candidates() {
    let file_a = "a.rs";
    let file_b = "b.rs";
    let file_c = "c.rs";

    let foo_b = make_symbol("b.rs::foo", "foo", file_b);
    let foo_c = make_symbol("c.rs::foo", "foo", file_c);
    let bar_a = make_symbol("a.rs::bar", "bar", file_a);

    let mut symbols_by_file: BTreeMap<String, Vec<SymbolNode>> = BTreeMap::new();
    symbols_by_file.insert(file_a.to_string(), vec![bar_a.clone()]);
    symbols_by_file.insert(file_b.to_string(), vec![foo_b.clone()]);
    symbols_by_file.insert(file_c.to_string(), vec![foo_c.clone()]);

    let edge = make_edge("a.rs::bar", "foo", file_a);
    let mut outgoing: BTreeMap<String, Vec<SymbolEdge>> = BTreeMap::new();
    outgoing.insert(file_a.to_string(), vec![edge]);

    let mut deps_by_file: BTreeMap<String, Vec<String>> = BTreeMap::new();
    deps_by_file.insert(
        file_a.to_string(),
        vec![file_b.to_string(), file_c.to_string()],
    );

    resolve_call_sites(&deps_by_file, &symbols_by_file, &mut outgoing);

    let edges = &outgoing[file_a];
    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].confidence,
        Confidence::MultipleCandidates,
        "ambiguous imported match must be MultipleCandidates"
    );
    let mut expected = vec!["b.rs::foo".to_string(), "c.rs::foo".to_string()];
    expected.sort();
    let mut actual = edges[0].callee_candidates.clone();
    actual.sort();
    assert_eq!(actual, expected, "callee_candidates must contain both ids sorted");
}

// ---------------------------------------------------------------------------
// Fixture 4 — Unresolved: A calls bar; nothing defines bar.
// After resolve, confidence == Unresolved, callee_candidates is empty.
// ---------------------------------------------------------------------------

#[test]
fn resolution_unresolved_no_definition() {
    let file_a = "a.rs";

    let fn_a = make_symbol("a.rs::caller", "caller", file_a);

    let mut symbols_by_file: BTreeMap<String, Vec<SymbolNode>> = BTreeMap::new();
    symbols_by_file.insert(file_a.to_string(), vec![fn_a.clone()]);

    let edge = make_edge("a.rs::caller", "bar", file_a);
    let mut outgoing: BTreeMap<String, Vec<SymbolEdge>> = BTreeMap::new();
    outgoing.insert(file_a.to_string(), vec![edge]);

    let deps_by_file: BTreeMap<String, Vec<String>> = BTreeMap::new();

    resolve_call_sites(&deps_by_file, &symbols_by_file, &mut outgoing);

    let edges = &outgoing[file_a];
    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].confidence,
        Confidence::Unresolved,
        "undefined callee must be Unresolved"
    );
    assert!(
        edges[0].callee_candidates.is_empty(),
        "Unresolved edges must have empty callee_candidates"
    );
}

// ---------------------------------------------------------------------------
// Fixture 5 — Re-export 2-hop: A imports B; B imports C; C defines foo; A calls foo.
// After resolve, confidence == Unique, callee_candidates == [C::foo].
// ---------------------------------------------------------------------------

#[test]
fn resolution_reexport_2hop() {
    let file_a = "a.rs";
    let file_b = "b.rs";
    let file_c = "c.rs";

    let foo_c = make_symbol("c.rs::foo", "foo", file_c);
    let bar_b = make_symbol("b.rs::bar", "bar", file_b);
    let caller_a = make_symbol("a.rs::caller", "caller", file_a);

    let mut symbols_by_file: BTreeMap<String, Vec<SymbolNode>> = BTreeMap::new();
    symbols_by_file.insert(file_a.to_string(), vec![caller_a.clone()]);
    symbols_by_file.insert(file_b.to_string(), vec![bar_b.clone()]);
    symbols_by_file.insert(file_c.to_string(), vec![foo_c.clone()]);

    let edge = make_edge("a.rs::caller", "foo", file_a);
    let mut outgoing: BTreeMap<String, Vec<SymbolEdge>> = BTreeMap::new();
    outgoing.insert(file_a.to_string(), vec![edge]);

    let mut deps_by_file: BTreeMap<String, Vec<String>> = BTreeMap::new();
    // A depends on B, B depends on C (2-hop from A's perspective).
    deps_by_file.insert(file_a.to_string(), vec![file_b.to_string()]);
    deps_by_file.insert(file_b.to_string(), vec![file_c.to_string()]);

    resolve_call_sites(&deps_by_file, &symbols_by_file, &mut outgoing);

    let edges = &outgoing[file_a];
    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].confidence,
        Confidence::Unique,
        "2-hop unique match must be Unique"
    );
    assert_eq!(
        edges[0].callee_candidates,
        vec!["c.rs::foo".to_string()],
        "callee_candidates must contain c.rs::foo"
    );
}

// ---------------------------------------------------------------------------
// compute_unresolved_pct: 4 edges, 1 unresolved -> 25.0%
// ---------------------------------------------------------------------------

#[test]
fn compute_unresolved_pct_25_percent() {
    let file_a = "a.rs";

    let mut outgoing: BTreeMap<String, Vec<SymbolEdge>> = BTreeMap::new();

    let resolved_edge = |callee: &str| SymbolEdge {
        caller_id: "a.rs::fn".to_string(),
        callee_name: callee.to_string(),
        callee_candidates: vec!["x".to_string()],
        confidence: Confidence::Unique,
        call_site: CallSite {
            file: file_a.to_string(),
            line: 1,
        },
    };

    let unresolved_edge = SymbolEdge {
        caller_id: "a.rs::fn".to_string(),
        callee_name: "unknown".to_string(),
        callee_candidates: vec![],
        confidence: Confidence::Unresolved,
        call_site: CallSite {
            file: file_a.to_string(),
            line: 2,
        },
    };

    outgoing.insert(
        file_a.to_string(),
        vec![
            resolved_edge("b"),
            resolved_edge("c"),
            resolved_edge("d"),
            unresolved_edge,
        ],
    );

    let pct = compute_unresolved_pct(&outgoing);
    assert!(
        (pct - 25.0).abs() < 1e-9,
        "expected 25.0% unresolved, got {pct}"
    );
}

// Empty map -> 0.0%
#[test]
fn compute_unresolved_pct_empty_is_zero() {
    let outgoing: BTreeMap<String, Vec<SymbolEdge>> = BTreeMap::new();
    assert_eq!(compute_unresolved_pct(&outgoing), 0.0);
}
