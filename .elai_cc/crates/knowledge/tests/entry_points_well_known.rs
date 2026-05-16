// B-34 contract test: Heuristic 2 — well-known name detection.
//
// Tests case-sensitivity: Rust "main" -> WellKnownName("main"),
// C# "Main" -> WellKnownName("Main"). Python "__main__" -> WellKnownName("__main__").
// "main" in a C# file should NOT match (C# expects "Main", not "main").

use std::collections::{BTreeMap, BTreeSet};

use elai_knowledge::symbol_graph::entry_points::detect_entry_points;
use elai_knowledge::symbol_graph::entry_points::types::EntryPointReason;
use elai_knowledge::symbol_graph::types::{
    SymbolGraphFilePayload, SymbolKind, SymbolNode,
};

fn make_sym(id: &str, name: &str, file: &str, language: &str) -> SymbolNode {
    SymbolNode {
        id: id.to_string(),
        name: name.to_string(),
        qualified_name: name.to_string(),
        kind: SymbolKind::Function,
        file: file.to_string(),
        line: 1,
        end_line: 10,
        language: language.to_string(),
    }
}

fn payload(file: &str, language: &str, symbols: Vec<SymbolNode>) -> SymbolGraphFilePayload {
    SymbolGraphFilePayload {
        file: file.to_string(),
        language: language.to_string(),
        content_hash: "x".to_string(),
        symbols,
        outgoing_calls: vec![],
    }
}

#[test]
fn rust_main_detected() {
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let sym = make_sym("main.rs::main", "main", "main.rs", "rust");
    let payloads = vec![payload("main.rs", "rust", vec![sym])];
    let entries = detect_entry_points(&reverse_index, &payloads);
    let hit = entries.iter().find(|e| {
        matches!(&e.reason, EntryPointReason::WellKnownName(n) if n == "main")
    });
    assert!(hit.is_some(), "rust 'main' should be detected as WellKnownName");
}

#[test]
fn go_main_detected() {
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let sym = make_sym("main.go::main", "main", "main.go", "go");
    let payloads = vec![payload("main.go", "go", vec![sym])];
    let entries = detect_entry_points(&reverse_index, &payloads);
    let hit = entries.iter().find(|e| {
        matches!(&e.reason, EntryPointReason::WellKnownName(n) if n == "main")
    });
    assert!(hit.is_some(), "go 'main' should be detected");
}

#[test]
fn csharp_main_capital_detected() {
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    // C# uses "Main" (capital M).
    let sym = make_sym("Program.cs::Main", "Main", "Program.cs", "csharp");
    let payloads = vec![payload("Program.cs", "csharp", vec![sym])];
    let entries = detect_entry_points(&reverse_index, &payloads);
    let hit = entries.iter().find(|e| {
        matches!(&e.reason, EntryPointReason::WellKnownName(n) if n == "Main")
    });
    assert!(hit.is_some(), "csharp 'Main' should be detected");
}

#[test]
fn csharp_lowercase_main_not_detected() {
    // C# only recognises "Main" (capital M); "main" should NOT match.
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let sym = make_sym("Prog.cs::main", "main", "Prog.cs", "csharp");
    let payloads = vec![payload("Prog.cs", "csharp", vec![sym])];
    let entries = detect_entry_points(&reverse_index, &payloads);
    let hit = entries.iter().find(|e| {
        matches!(&e.reason, EntryPointReason::WellKnownName(n) if n == "main")
    });
    assert!(hit.is_none(), "csharp lowercase 'main' must NOT match");
}

#[test]
fn python_dunder_main_detected() {
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let sym = make_sym("__main__.py::__main__", "__main__", "__main__.py", "python");
    let payloads = vec![payload("__main__.py", "python", vec![sym])];
    let entries = detect_entry_points(&reverse_index, &payloads);
    let hit = entries.iter().find(|e| {
        matches!(&e.reason, EntryPointReason::WellKnownName(n) if n == "__main__")
    });
    assert!(hit.is_some(), "python '__main__' should be detected as WellKnownName");
}

#[test]
fn module_symbol_skipped() {
    // The "<module>" pseudo-symbol must never be emitted.
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let sym = make_sym("mod.py::<module>", "<module>", "mod.py", "python");
    let payloads = vec![payload("mod.py", "python", vec![sym])];
    let entries = detect_entry_points(&reverse_index, &payloads);
    let bad = entries.iter().find(|e| e.name == "<module>");
    assert!(bad.is_none(), "<module> must not be emitted as entry point");
}
