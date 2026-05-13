//! Phase B2 — SymbolGraph library skeleton contract.
//!
//! Pins:
//!   (a) Given a synthetic `elai.map` TSV fixture, `SymbolGraph::query(symbol, 1)`
//!       returns every record whose SYMBOL column matches, with correct
//!       SymbolTag enum, file path, line number, language, and snippet.
//!   (b) `SymbolGraph::query(symbol, 2)` recursively expands CALL /
//!       CALL_RESOLVED / SCIP_REF outgoing edges from depth-1 matches, so
//!       `query("foo", 2)` includes rows for `foo` AND for any callee
//!       referenced from foo's body.
//!   (c) Constructing a `SymbolGraph` against a non-existent map path and
//!       querying it returns `SymbolGraphError::MapNotFound` (typed error,
//!       no panic / no .unwrap()).
//!
//! No live filesystem touched outside `tempfile::TempDir` — the real
//! `.claude/tools/elai.map` is read by integration smoke tests in the
//! orchestrator crate, not here. Contract test stays hermetic.

use std::fs;
use std::path::PathBuf;

use elai_orchestrator::{SymbolGraph, SymbolGraphError, SymbolRecord, SymbolTag};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixture builder
// ---------------------------------------------------------------------------

/// Write a synthetic elai.map TSV fixture into `dir` and return its path.
/// Columns: TAG\tSYMBOL\tFILE:LINE\tLANG\tEXTRA\tSNIPPET.
fn write_fixture(dir: &TempDir, rows: &[&str]) -> PathBuf {
    let path = dir.path().join("elai.map");
    let mut content = String::from("# elai codebase map — test fixture\n");
    for row in rows {
        content.push_str(row);
        content.push('\n');
    }
    fs::write(&path, content).expect("write fixture");
    path
}

// ---------------------------------------------------------------------------
// Test 1: depth=1 returns all matching records with correct fields
// ---------------------------------------------------------------------------

#[test]
fn synthetic_map_returns_matching_records() {
    let dir = TempDir::new().expect("tempdir");
    // Three rows: two refer to `foo` (a DEF_FN and a CALL), one to unrelated `bar`.
    let map = write_fixture(
        &dir,
        &[
            "DEF_FN\tfoo\tcrates/a/src/lib.rs:10\trust\t\tfn foo() -> u32 {",
            "CALL\tfoo\tcrates/b/src/main.rs:42\trust\tmain\tfoo();",
            "DEF_FN\tbar\tcrates/a/src/lib.rs:20\trust\t\tfn bar() {}",
        ],
    );

    let graph = SymbolGraph::new(map);
    let recs = graph.query("foo", 1).expect("query ok");

    assert_eq!(recs.len(), 2, "expected 2 rows for `foo`, got {}", recs.len());
    let tags: Vec<&SymbolTag> = recs.iter().map(|r| &r.tag).collect();
    assert!(
        tags.iter().any(|t| matches!(t, SymbolTag::DefFn)),
        "missing DEF_FN row: {:?}",
        tags
    );
    assert!(
        tags.iter().any(|t| matches!(t, SymbolTag::Call)),
        "missing CALL row: {:?}",
        tags
    );

    let def_row: &SymbolRecord = recs
        .iter()
        .find(|r| matches!(r.tag, SymbolTag::DefFn))
        .expect("def row exists");
    assert_eq!(def_row.symbol, "foo");
    assert_eq!(def_row.line, 10);
    assert_eq!(def_row.lang, "rust");
    assert!(def_row.file.ends_with("crates/a/src/lib.rs"));
    assert!(def_row.snippet.as_deref() == Some("fn foo() -> u32 {"));

    // Bar should NOT appear in a depth-1 query for foo.
    assert!(recs.iter().all(|r| r.symbol == "foo"));
}

// ---------------------------------------------------------------------------
// Test 2: depth=2 expands CALL / CALL_RESOLVED targets
// ---------------------------------------------------------------------------

#[test]
fn depth_2_includes_callees() {
    let dir = TempDir::new().expect("tempdir");
    // `foo` defined and calls `bar`. `bar` is defined elsewhere. With depth=1
    // we should only see foo's two rows; with depth=2 we should also see
    // bar's DEF_FN row because foo->bar is a CALL edge from a depth-1 row.
    let map = write_fixture(
        &dir,
        &[
            "DEF_FN\tfoo\tcrates/a/src/lib.rs:10\trust\t\tfn foo() {",
            "CALL\tbar\tcrates/a/src/lib.rs:11\trust\tfoo\tbar();",
            "DEF_FN\tbar\tcrates/a/src/lib.rs:20\trust\t\tfn bar() {}",
            "DEF_FN\tunrelated\tcrates/z/src/lib.rs:5\trust\t\tfn unrelated() {}",
        ],
    );

    let graph = SymbolGraph::new(map);

    let d1 = graph.query("foo", 1).expect("d1 query ok");
    assert_eq!(d1.len(), 1, "depth=1 should only return foo's own rows");
    assert!(d1.iter().all(|r| r.symbol == "foo"));

    let d2 = graph.query("foo", 2).expect("d2 query ok");
    let symbols: Vec<&str> = d2.iter().map(|r| r.symbol.as_str()).collect();
    assert!(
        symbols.contains(&"foo"),
        "depth=2 must still include foo: {:?}",
        symbols
    );
    assert!(
        symbols.contains(&"bar"),
        "depth=2 must include callee bar: {:?}",
        symbols
    );
    assert!(
        !symbols.contains(&"unrelated"),
        "depth=2 must NOT include unrelated symbols: {:?}",
        symbols
    );
}

// ---------------------------------------------------------------------------
// Test 3: missing map path returns typed MapNotFound error (no panic)
// ---------------------------------------------------------------------------

#[test]
fn missing_map_returns_typed_error() {
    let bogus = PathBuf::from("definitely/does/not/exist/elai.map");
    let graph = SymbolGraph::new(bogus);

    match graph.query("foo", 1) {
        Err(SymbolGraphError::MapNotFound(_)) => { /* expected */ }
        Err(other) => panic!("expected MapNotFound, got {:?}", other),
        Ok(rows) => panic!("expected error, got {} rows", rows.len()),
    }
}

// ---------------------------------------------------------------------------
// Test 4: SCIP_REF / CALL_RESOLVED rows do not poison the outgoing-edge
// index (regression for BLOCK SCIP_REF_INDEX_POLLUTION).
//
// SCIP_REF EXTRA columns carry canonical paths like
// `scip-typescript npm <pkg> <ver> src/...`. CALL_RESOLVED EXTRA columns
// carry `local:DEF_FN:L<line><-actual_caller`. Naive tokenisation of those
// payloads would synthesize fake caller-scope identifiers (`typescript`,
// `npm`, `local`, etc.) and make depth>=2 queries return rows that have
// nothing to do with the input symbol. This test pins that those rows are
// EXCLUDED from the outgoing-by-scope index.
// ---------------------------------------------------------------------------

#[test]
fn scip_ref_rows_do_not_poison_outgoing_index() {
    let dir = TempDir::new().expect("tempdir");
    // Fixture mimics live elai.map: SCIP_REF rows whose EXTRA contains the
    // `typescript` token, plus a CALL_RESOLVED row whose EXTRA contains the
    // `local` token. None of these should attach to those tokens as a
    // caller scope. A clean depth=2 query for those bogus tokens must
    // return zero records.
    let map = write_fixture(
        &dir,
        &[
            // SCIP-resolved references with canonical typescript paths.
            "SCIP_REF\tButton\tcrates/dashboard-v2/src/shell/App.svelte:14\ttypescript\tscip-typescript npm elai-dashboard-frontend 0.1.0 src/shell/App.svelte\t<Button />",
            "SCIP_REF\tStore\tcrates/dashboard-v2/src/store.ts:7\ttypescript\tscip-typescript npm elai-dashboard-frontend 0.1.0 src/store.ts\timport { Store }",
            // CALL_RESOLVED row with structured EXTRA.
            "CALL_RESOLVED\thelper\tcrates/a/src/lib.rs:55\trust\tlocal:DEF_FN:L40<-caller_fn\thelper();",
            // A real CALL row used as positive control: query("foo", 2) must
            // still expand to its callee.
            "DEF_FN\tfoo\tcrates/a/src/lib.rs:10\trust\t\tfn foo() {",
            "CALL\tbar\tcrates/a/src/lib.rs:11\trust\tfoo\tbar();",
            "DEF_FN\tbar\tcrates/a/src/lib.rs:20\trust\t\tfn bar() {}",
        ],
    );

    let graph = SymbolGraph::new(map);

    // Negative cases: tokens that ONLY exist inside SCIP_REF / CALL_RESOLVED
    // EXTRA payloads must not be treated as caller scopes.
    for poisoned_token in ["typescript", "npm", "scip", "local", "elai", "dashboard", "frontend"] {
        let rows = graph
            .query(poisoned_token, 2)
            .unwrap_or_else(|e| panic!("query({poisoned_token}, 2) errored: {e:?}"));
        // A direct SYMBOL hit on these tokens is impossible in the fixture
        // (no row has SYMBOL=typescript / npm / etc.), so any non-empty
        // result must be coming from the polluted outgoing-by-scope index.
        assert!(
            rows.is_empty(),
            "query({poisoned_token}, 2) returned {} rows — SCIP_REF/CALL_RESOLVED poisoned the outgoing-by-scope index: {:?}",
            rows.len(),
            rows.iter().map(|r| (&r.tag, r.symbol.as_str())).collect::<Vec<_>>()
        );
    }

    // Positive control: the real CALL edge from foo->bar still works.
    let foo_d2 = graph.query("foo", 2).expect("query foo ok");
    let foo_symbols: Vec<&str> = foo_d2.iter().map(|r| r.symbol.as_str()).collect();
    assert!(
        foo_symbols.contains(&"bar"),
        "regression: CALL edge foo->bar should still be followed at depth=2, got {:?}",
        foo_symbols
    );
}
