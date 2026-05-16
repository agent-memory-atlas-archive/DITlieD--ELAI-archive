// B-34 contract test: Heuristic 4 — test function name detection.
//
// Patterns: ^test[_A-Z] | ^Test[A-Z_] | "it" | "describe".
// Regular functions must not be detected.

use std::collections::{BTreeMap, BTreeSet};

use elai_knowledge::symbol_graph::entry_points::detect_entry_points;
use elai_knowledge::symbol_graph::entry_points::types::EntryPointReason;
use elai_knowledge::symbol_graph::types::{
    SymbolGraphFilePayload, SymbolKind, SymbolNode,
};

fn sym(name: &str, file: &str) -> SymbolNode {
    SymbolNode {
        id: format!("{file}::{name}"),
        name: name.to_string(),
        qualified_name: name.to_string(),
        kind: SymbolKind::Function,
        file: file.to_string(),
        line: 1,
        end_line: 10,
        language: "rust".to_string(),
    }
}

fn payload_of(file: &str, symbols: Vec<SymbolNode>) -> SymbolGraphFilePayload {
    SymbolGraphFilePayload {
        file: file.to_string(),
        language: "rust".to_string(),
        content_hash: "t".to_string(),
        symbols,
        outgoing_calls: vec![],
    }
}

fn test_entries(payloads: &[SymbolGraphFilePayload]) -> Vec<String> {
    let ri: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    detect_entry_points(&ri, payloads)
        .into_iter()
        .filter(|e| matches!(e.reason, EntryPointReason::Test))
        .map(|e| e.name)
        .collect()
}

#[test]
fn test_underscore_prefix_detected() {
    let p = payload_of("a.rs", vec![sym("test_foo", "a.rs")]);
    assert!(test_entries(&[p]).contains(&"test_foo".to_string()));
}

#[test]
fn test_uppercase_prefix_detected() {
    let p = payload_of("a.rs", vec![sym("testBar", "a.rs")]);
    assert!(test_entries(&[p]).contains(&"testBar".to_string()));
}

#[test]
fn test_capital_t_upper_detected() {
    let p = payload_of("a.rs", vec![sym("TestFoo", "a.rs")]);
    assert!(test_entries(&[p]).contains(&"TestFoo".to_string()));
}

#[test]
fn test_capital_t_underscore_detected() {
    let p = payload_of("a.rs", vec![sym("Test_something", "a.rs")]);
    assert!(test_entries(&[p]).contains(&"Test_something".to_string()));
}

#[test]
fn it_detected() {
    let p = payload_of("spec.js", vec![sym("it", "spec.js")]);
    assert!(test_entries(&[p]).contains(&"it".to_string()));
}

#[test]
fn describe_detected() {
    let p = payload_of("spec.js", vec![sym("describe", "spec.js")]);
    assert!(test_entries(&[p]).contains(&"describe".to_string()));
}

#[test]
fn regular_function_not_detected() {
    let p = payload_of("a.rs", vec![sym("regular_fn", "a.rs")]);
    assert!(test_entries(&[p]).is_empty(), "regular_fn should not be detected as test");
}

#[test]
fn test_lowercase_no_suffix_not_detected() {
    // "testx" — starts with "test" but 5th char is lowercase 'x' -> false
    let p = payload_of("a.rs", vec![sym("testx", "a.rs")]);
    assert!(test_entries(&[p]).is_empty(), "testx (5th char lowercase) must not match");
}
