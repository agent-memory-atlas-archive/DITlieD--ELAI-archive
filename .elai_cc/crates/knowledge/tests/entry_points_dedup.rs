// B-34 contract test: Deduplication — same (id, reason_key) pair never emitted twice.
// A symbol that qualifies under both Orphan + WellKnownName emits 2 entries (different
// reason keys). A symbol that would match WellKnownName twice (impossible by construction
// since the name table has unique names) is still deduplicated.

use std::collections::{BTreeMap, BTreeSet};

use elai_knowledge::symbol_graph::entry_points::detect_entry_points;
use elai_knowledge::symbol_graph::entry_points::types::EntryPointReason;
use elai_knowledge::symbol_graph::types::{
    CallSite, Confidence, SymbolEdge, SymbolGraphFilePayload, SymbolKind, SymbolNode,
};

fn sym(id: &str, name: &str, file: &str, line: u32) -> SymbolNode {
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

fn edge(caller_id: &str, callee_name: &str, file: &str, line: u32) -> SymbolEdge {
    SymbolEdge {
        caller_id: caller_id.to_string(),
        callee_name: callee_name.to_string(),
        callee_candidates: Vec::new(),
        confidence: Confidence::Unresolved,
        call_site: CallSite {
            file: file.to_string(),
            line,
        },
    }
}

/// A file that is an orphan AND has a symbol named "main" should produce
/// 2 separate entries (Orphan for the file, WellKnownName for the symbol)
/// but never 2 Orphan entries for the same file.
#[test]
fn orphan_and_well_known_produce_two_distinct_entries() {
    // main.rs: not imported by anyone (orphan), has a "main" symbol.
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    let main_sym = sym("main.rs::main", "main", "main.rs", 1);
    let payload = SymbolGraphFilePayload {
        file: "main.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "x".to_string(),
        symbols: vec![main_sym],
        outgoing_calls: vec![edge("main.rs::main", "println", "main.rs", 5)],
    };

    let entries = detect_entry_points(&reverse_index, &[payload]);

    let orphan_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.file == "main.rs" && matches!(e.reason, EntryPointReason::Orphan))
        .collect();
    let well_known_entries: Vec<_> = entries
        .iter()
        .filter(|e| {
            e.file == "main.rs"
                && matches!(&e.reason, EntryPointReason::WellKnownName(n) if n == "main")
        })
        .collect();

    // Exactly 1 Orphan + 1 WellKnownName for main.rs.
    assert_eq!(orphan_entries.len(), 1, "expect exactly 1 Orphan for main.rs");
    assert_eq!(
        well_known_entries.len(),
        1,
        "expect exactly 1 WellKnownName(main) for main.rs"
    );
}

/// Supplying the same payload twice must not double-emit entries (idempotent dedup).
#[test]
fn duplicate_payload_does_not_double_emit() {
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    let p = SymbolGraphFilePayload {
        file: "helper.rs".to_string(),
        language: "rust".to_string(),
        content_hash: "x".to_string(),
        symbols: vec![sym("helper.rs::main", "main", "helper.rs", 1)],
        outgoing_calls: vec![edge("helper.rs::main", "println", "helper.rs", 3)],
    };

    // Pass the same payload twice.
    let entries = detect_entry_points(&reverse_index, &[p.clone(), p]);

    // Should deduplicate: still only 1 WellKnownName + 1 Orphan, not 2 each.
    let wk_count = entries
        .iter()
        .filter(|e| matches!(&e.reason, EntryPointReason::WellKnownName(n) if n == "main"))
        .count();
    assert_eq!(wk_count, 1, "duplicate payload must not produce 2 WellKnownName entries");

    let orphan_count = entries
        .iter()
        .filter(|e| matches!(e.reason, EntryPointReason::Orphan))
        .count();
    assert_eq!(orphan_count, 1, "duplicate payload must not produce 2 Orphan entries");
}
