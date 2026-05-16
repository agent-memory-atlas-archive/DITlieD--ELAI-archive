// B-34 contract test: Heuristic 1 — orphan file detection.
//
// Fixture: 3-file graph.
//   A.rs: has outgoing calls (calls into B). Imported by B (so it IS imported).
//   B.rs: has outgoing calls (calls into A). NOT imported by anyone -> orphan candidate.
//   C.rs: no outgoing calls, not imported -> orphan candidate but skipped (no outgoing calls).
//
// Expected: only B.rs is an orphan (has callers=0 AND outgoing_calls>0).
// A.rs is NOT orphan (B calls A, so reverse_index["A.rs"] = {B.rs}).
// C.rs is NOT orphan (no outgoing calls, so even though it's not imported, skip it).

use std::collections::{BTreeMap, BTreeSet};

use elai_knowledge::symbol_graph::entry_points::detect_entry_points;
use elai_knowledge::symbol_graph::entry_points::types::EntryPointReason;
use elai_knowledge::symbol_graph::types::{
    CallSite, Confidence, SymbolEdge, SymbolGraphFilePayload, SymbolKind, SymbolNode,
};

fn make_sym(id: &str, name: &str, file: &str, line: u32) -> SymbolNode {
    SymbolNode {
        id: id.to_string(),
        name: name.to_string(),
        qualified_name: name.to_string(),
        kind: SymbolKind::Function,
        file: file.to_string(),
        line,
        end_line: line + 10,
        language: "rust".to_string(),
    }
}

fn make_edge(caller_id: &str, callee_name: &str, call_file: &str, call_line: u32) -> SymbolEdge {
    SymbolEdge {
        caller_id: caller_id.to_string(),
        callee_name: callee_name.to_string(),
        callee_candidates: Vec::new(),
        confidence: Confidence::Unresolved,
        call_site: CallSite {
            file: call_file.to_string(),
            line: call_line,
        },
    }
}

#[test]
fn orphan_b_detected_a_not_orphan_c_skipped() {
    // reverse_index: A.rs is imported by B.rs; B.rs is not imported by anyone.
    let mut reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    // A.rs has caller B.rs (B imports A).
    reverse_index
        .entry("A.rs".to_string())
        .or_default()
        .insert("B.rs".to_string());
    // B.rs has no callers (not a key in reverse_index, or empty set).
    // C.rs has no callers either (not a key).

    // Payloads.
    // A.rs: has symbols, but B imports it -> not orphan.
    let payload_a = SymbolGraphFilePayload {
        file: "A.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "aa".to_string(),
        symbols: vec![make_sym("A::helper", "helper", "A.rs", 1)],
        // A has outgoing calls (calls something in external lib).
        outgoing_calls: vec![make_edge("A::helper", "println", "A.rs", 5)],
    };

    // B.rs: not imported, has outgoing calls (calls A::helper) -> orphan.
    let payload_b = SymbolGraphFilePayload {
        file: "B.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "bb".to_string(),
        symbols: vec![make_sym("B::run", "run", "B.rs", 1)],
        outgoing_calls: vec![make_edge("B::run", "helper", "B.rs", 3)],
    };

    // C.rs: not imported, NO outgoing calls -> NOT an orphan (skip per heuristic).
    let payload_c = SymbolGraphFilePayload {
        file: "C.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "cc".to_string(),
        symbols: vec![make_sym("C::lib", "lib", "C.rs", 1)],
        outgoing_calls: vec![],
    };

    let payloads = vec![payload_a, payload_b, payload_c];
    let entries = detect_entry_points(&reverse_index, &payloads);

    // Only B.rs should be detected as Orphan.
    let orphans: Vec<_> = entries
        .iter()
        .filter(|e| matches!(e.reason, EntryPointReason::Orphan))
        .collect();

    assert_eq!(
        orphans.len(),
        1,
        "expected exactly 1 orphan; got: {:?}",
        orphans.iter().map(|e| &e.file).collect::<Vec<_>>()
    );
    assert_eq!(orphans[0].file, "B.rs", "orphan should be B.rs");

    // A.rs must not appear as orphan.
    let a_orphan = entries
        .iter()
        .any(|e| e.file == "A.rs" && matches!(e.reason, EntryPointReason::Orphan));
    assert!(!a_orphan, "A.rs must not be orphan (it is imported by B)");

    // C.rs must not appear as orphan.
    let c_orphan = entries
        .iter()
        .any(|e| e.file == "C.rs" && matches!(e.reason, EntryPointReason::Orphan));
    assert!(!c_orphan, "C.rs must not be orphan (no outgoing calls)");
}
