// B-29 Phase 2 — public API surface contract.
//
// Locks the signatures of extraction.rs and resolution.rs.
// Mirrors tasks 4-5 in B-29 Phase 2.

use std::collections::BTreeMap;

use elai_knowledge::dkg::RustGrammar;
use elai_knowledge::symbol_graph::extraction::{
    ExtractedSymbols, RawCall, extract_symbols_and_calls, raw_calls_to_unresolved_edges,
};
use elai_knowledge::symbol_graph::resolution::{compute_unresolved_pct, resolve_call_sites};
use elai_knowledge::symbol_graph::types::{
    CallSite, Confidence, SymbolEdge, SymbolKind, SymbolNode,
};

// ---------------------------------------------------------------------------
// Signature locks — compile-time surface contracts
// ---------------------------------------------------------------------------

#[test]
fn b29_phase2_extracted_symbols_fields_lock() {
    let _: fn(&str, &dyn elai_knowledge::LanguageGrammar, &str) -> ExtractedSymbols =
        extract_symbols_and_calls;

    // ExtractedSymbols has .symbols and .raw_calls
    let dummy = ExtractedSymbols {
        symbols: vec![],
        raw_calls: vec![],
    };
    let _: &Vec<SymbolNode> = &dummy.symbols;
    let _: &Vec<RawCall> = &dummy.raw_calls;
}

#[test]
fn b29_phase2_raw_call_fields_lock() {
    let rc = RawCall {
        caller_id: "f.rs::foo".to_string(),
        callee_name: "bar".to_string(),
        call_site: CallSite {
            file: "f.rs".to_string(),
            line: 5,
        },
    };
    assert!(!rc.caller_id.is_empty());
    assert!(!rc.callee_name.is_empty());
    assert_eq!(rc.call_site.line, 5);
}

#[test]
fn b29_phase2_resolve_call_sites_signature() {
    // Verify signature: (BTreeMap, BTreeMap, &mut BTreeMap) -> ()
    let deps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let symbols: BTreeMap<String, Vec<SymbolNode>> = BTreeMap::new();
    let mut outgoing: BTreeMap<String, Vec<SymbolEdge>> = BTreeMap::new();
    resolve_call_sites(&deps, &symbols, &mut outgoing);
}

#[test]
fn b29_phase2_compute_unresolved_pct_signature() {
    let outgoing: BTreeMap<String, Vec<SymbolEdge>> = BTreeMap::new();
    let _: f64 = compute_unresolved_pct(&outgoing);
}

// ---------------------------------------------------------------------------
// Smoke test — end-to-end Rust fixture through extract + resolve
// ---------------------------------------------------------------------------

const RUST_FIXTURE: &str = r#"
pub fn helper(x: i32) -> i32 {
    x + 1
}

pub fn main_fn() -> i32 {
    helper(10)
}
"#;

#[test]
fn b29_phase2_end_to_end_rust_smoke() {
    let file = "smoke.rs";
    let result = extract_symbols_and_calls(RUST_FIXTURE, &RustGrammar, file);

    // Exactly 1 <module> symbol as symbols[0].
    assert!(
        !result.symbols.is_empty(),
        "must produce at least the module symbol"
    );
    assert_eq!(
        result.symbols[0].kind,
        SymbolKind::Module,
        "symbols[0] must be Module"
    );
    // Exactly 1 <module> (not duplicated).
    let module_count = result
        .symbols
        .iter()
        .filter(|s| s.kind == SymbolKind::Module)
        .count();
    assert_eq!(module_count, 1, "exactly 1 Module symbol expected");

    // At least 1 Function symbol.
    let has_function = result
        .symbols
        .iter()
        .any(|s| s.kind == SymbolKind::Function);
    assert!(has_function, "at least 1 Function symbol expected");

    // At least 1 raw_call (main_fn calls helper).
    assert!(
        !result.raw_calls.is_empty(),
        "at least 1 raw_call expected for the Rust smoke fixture"
    );

    // Convert to unresolved edges, then resolve within single file.
    let edges = raw_calls_to_unresolved_edges(&result.raw_calls);
    // All start Unresolved.
    for e in &edges {
        assert_eq!(e.confidence, Confidence::Unresolved, "pre-resolve must be Unresolved");
    }

    // Build resolution maps — both symbols in the same file.
    let mut symbols_by_file: BTreeMap<String, Vec<SymbolNode>> = BTreeMap::new();
    symbols_by_file.insert(file.to_string(), result.symbols.clone());

    let mut outgoing_by_file: BTreeMap<String, Vec<SymbolEdge>> = BTreeMap::new();
    outgoing_by_file.insert(file.to_string(), edges.clone());

    let deps_by_file: BTreeMap<String, Vec<String>> = BTreeMap::new();
    resolve_call_sites(&deps_by_file, &symbols_by_file, &mut outgoing_by_file);

    let resolved_edges = &outgoing_by_file[file];
    assert_eq!(resolved_edges.len(), edges.len());

    // At least 1 edge with non-Unresolved confidence (intra-file match for helper).
    let has_resolved = resolved_edges
        .iter()
        .any(|e| e.confidence != Confidence::Unresolved);
    assert!(
        has_resolved,
        "at least 1 edge should be Local (intra-file); got: {:?}",
        resolved_edges.iter().map(|e| e.confidence).collect::<Vec<_>>()
    );

    // unresolved_pct must be < 100.0 (since at least 1 call resolves locally).
    let pct = compute_unresolved_pct(&outgoing_by_file);
    assert!(
        pct < 100.0,
        "unresolved_pct must be < 100.0 when at least 1 edge resolves; got {pct}"
    );
}
