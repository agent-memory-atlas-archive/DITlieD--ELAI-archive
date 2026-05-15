// B-29 Phase 2 — extraction tests.
//
// Tests for extract_symbols_and_calls, raw_calls_to_unresolved_edges,
// and postcard round-trip of the produced types.
//
// NOTE: These tests are FAILING until extraction.rs is implemented.

use elai_knowledge::dkg::{
    CSharpGrammar, GoGrammar, JavaGrammar, PythonGrammar, RubyGrammar, RustGrammar,
    TypeScriptGrammar,
};
use elai_knowledge::symbol_graph::extraction::{
    extract_symbols_and_calls, raw_calls_to_unresolved_edges,
};
use elai_knowledge::symbol_graph::types::{Confidence, SymbolKind};
use elai_knowledge::LanguageGrammar;

// ---------------------------------------------------------------------------
// Fixtures — one per language with at least 1 function that calls another.
// ---------------------------------------------------------------------------

const RUST_SRC: &str = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn compute(x: i32) -> i32 {
    add(x, 1)
}
"#;

const PYTHON_SRC: &str = r#"
def helper(x):
    return x * 2

def main():
    result = helper(10)
    return result
"#;

const TS_SRC: &str = r#"
export function helper(x: number): number {
    return x * 2;
}

export function main(): number {
    return helper(10);
}
"#;

const GO_SRC: &str = r#"package main

func add(a int, b int) int {
	return a + b
}

func compute(x int) int {
	return add(x, 1)
}
"#;

const JAVA_SRC: &str = r#"
public class Calculator {
    public static int add(int a, int b) {
        return a + b;
    }

    public static int compute(int x) {
        return add(x, 1);
    }
}
"#;

const CSHARP_SRC: &str = r#"
public class Calculator {
    public int Add(int a, int b) {
        return a + b;
    }

    public int Compute(int x) {
        return Add(x, 1);
    }
}
"#;

const RUBY_SRC: &str = r#"
def helper(x)
  x * 2
end

def main
  helper(10)
end
"#;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn assert_extraction(
    grammar: &dyn LanguageGrammar,
    source: &str,
    file: &str,
    expected_fn_name: &str,
) {
    let result = extract_symbols_and_calls(source, grammar, file);

    // symbols[0] must be the synthetic <module> symbol.
    assert!(
        !result.symbols.is_empty(),
        "expected at least the <module> symbol for {file}"
    );
    let module = &result.symbols[0];
    assert_eq!(
        module.kind,
        SymbolKind::Module,
        "symbols[0] must be Module for {file}, got {:?}",
        module.kind
    );
    assert!(
        module.name.contains("module"),
        "symbols[0].name should contain 'module' for {file}, got {:?}",
        module.name
    );
    assert_eq!(
        module.file, file,
        "module symbol file must match input for {file}"
    );

    // At least one expected function symbol.
    let has_fn = result
        .symbols
        .iter()
        .any(|s| s.name == expected_fn_name);
    assert!(
        has_fn,
        "expected function symbol '{expected_fn_name}' for {file}; symbols: {:?}",
        result.symbols.iter().map(|s| &s.name).collect::<Vec<_>>()
    );

    // At least one raw_call (because each fixture calls another fn).
    assert!(
        !result.raw_calls.is_empty(),
        "expected at least 1 raw_call for {file}"
    );

    // Every call_site.file must match the input file path.
    for rc in &result.raw_calls {
        assert_eq!(
            rc.call_site.file, file,
            "raw_call.call_site.file must equal input file for {file}"
        );
    }
}

// ---------------------------------------------------------------------------
// Per-language extraction tests
// ---------------------------------------------------------------------------

#[test]
fn extraction_rust_basic() {
    assert_extraction(&RustGrammar, RUST_SRC, "src/lib.rs", "add");
}

#[test]
fn extraction_python_basic() {
    assert_extraction(&PythonGrammar, PYTHON_SRC, "src/main.py", "helper");
}

#[test]
fn extraction_typescript_basic() {
    assert_extraction(&TypeScriptGrammar, TS_SRC, "src/index.ts", "helper");
}

#[test]
fn extraction_go_basic() {
    assert_extraction(&GoGrammar, GO_SRC, "src/main.go", "add");
}

#[test]
fn extraction_java_basic() {
    assert_extraction(&JavaGrammar, JAVA_SRC, "src/Calculator.java", "add");
}

#[test]
fn extraction_csharp_basic() {
    assert_extraction(&CSharpGrammar, CSHARP_SRC, "src/Calculator.cs", "Add");
}

#[test]
fn extraction_ruby_basic() {
    assert_extraction(&RubyGrammar, RUBY_SRC, "src/main.rb", "helper");
}

// ---------------------------------------------------------------------------
// raw_calls_to_unresolved_edges: all edges must be Unresolved + empty candidates
// ---------------------------------------------------------------------------

#[test]
fn raw_calls_to_unresolved_edges_all_unresolved() {
    let result = extract_symbols_and_calls(RUST_SRC, &RustGrammar, "src/lib.rs");
    assert!(
        !result.raw_calls.is_empty(),
        "need at least one raw_call for this test"
    );

    let edges = raw_calls_to_unresolved_edges(&result.raw_calls);

    assert_eq!(
        edges.len(),
        result.raw_calls.len(),
        "edge count must equal raw_call count"
    );
    for edge in &edges {
        assert_eq!(
            edge.confidence,
            Confidence::Unresolved,
            "all edges from raw_calls_to_unresolved_edges must have Unresolved confidence"
        );
        assert!(
            edge.callee_candidates.is_empty(),
            "all edges from raw_calls_to_unresolved_edges must have empty callee_candidates"
        );
    }
}

// ---------------------------------------------------------------------------
// postcard round-trip for ExtractedSymbols types
// ---------------------------------------------------------------------------

#[test]
fn extracted_symbols_postcard_roundtrip() {
    let result = extract_symbols_and_calls(RUST_SRC, &RustGrammar, "src/lib.rs");

    let symbols_bytes = postcard::to_allocvec(&result.symbols).expect("postcard encode symbols");
    let symbols_back: Vec<elai_knowledge::symbol_graph::types::SymbolNode> =
        postcard::from_bytes(&symbols_bytes).expect("postcard decode symbols");
    assert_eq!(result.symbols, symbols_back);

    let edges = raw_calls_to_unresolved_edges(&result.raw_calls);
    let edges_bytes = postcard::to_allocvec(&edges).expect("postcard encode edges");
    let edges_back: Vec<elai_knowledge::symbol_graph::types::SymbolEdge> =
        postcard::from_bytes(&edges_bytes).expect("postcard decode edges");
    assert_eq!(edges, edges_back);
}
