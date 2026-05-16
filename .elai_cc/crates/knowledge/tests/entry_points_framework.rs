// B-34 contract test: Heuristic 3 — framework route detection.
//
// Tests that outgoing calls to HTTP verb names (get/post/put/delete/patch/head/
// options/all/use) and "route" are attributed to the enclosing symbol.

use std::collections::{BTreeMap, BTreeSet};

use elai_knowledge::symbol_graph::entry_points::detect_entry_points;
use elai_knowledge::symbol_graph::entry_points::types::EntryPointReason;
use elai_knowledge::symbol_graph::types::{
    CallSite, Confidence, SymbolEdge, SymbolGraphFilePayload, SymbolKind, SymbolNode,
};

fn make_sym(id: &str, name: &str, file: &str, line: u32, end_line: u32) -> SymbolNode {
    SymbolNode {
        id: id.to_string(),
        name: name.to_string(),
        qualified_name: name.to_string(),
        kind: SymbolKind::Function,
        file: file.to_string(),
        line,
        end_line,
        language: "javascript".to_string(),
    }
}

fn make_edge(caller_id: &str, callee_name: &str, file: &str, line: u32) -> SymbolEdge {
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

#[test]
fn app_get_emits_framework_reason() {
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    // Symbol "register_routes" spans lines 1-20; call to "get" at line 5.
    let route_sym = make_sym("app.js::register_routes", "register_routes", "app.js", 1, 20);
    let get_edge = make_edge("app.js::register_routes", "get", "app.js", 5);
    let payload = SymbolGraphFilePayload {
        file: "app.js".to_string(),
        language: "javascript".to_string(),
        content_hash: "hx".to_string(),
        symbols: vec![route_sym],
        outgoing_calls: vec![get_edge],
    };
    let entries = detect_entry_points(&reverse_index, &[payload]);
    let hit = entries.iter().find(|e| {
        matches!(&e.reason, EntryPointReason::FrameworkRoute(r) if r == "framework:http-get")
    });
    assert!(hit.is_some(), "app.get call should emit FrameworkRoute(framework:http-get)");
    // Attributed to the enclosing symbol.
    assert_eq!(hit.unwrap().name, "register_routes");
}

#[test]
fn router_post_emits_framework_reason() {
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let sym = make_sym("router.js::setup", "setup", "router.js", 1, 30);
    let post_edge = make_edge("router.js::setup", "post", "router.js", 10);
    let payload = SymbolGraphFilePayload {
        file: "router.js".to_string(),
        language: "javascript".to_string(),
        content_hash: "h".to_string(),
        symbols: vec![sym],
        outgoing_calls: vec![post_edge],
    };
    let entries = detect_entry_points(&reverse_index, &[payload]);
    let hit = entries.iter().find(|e| {
        matches!(&e.reason, EntryPointReason::FrameworkRoute(r) if r == "framework:http-post")
    });
    assert!(hit.is_some(), "router.post should emit FrameworkRoute(framework:http-post)");
}

#[test]
fn route_call_emits_framework_route() {
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let sym = make_sym("views.py::register", "register", "views.py", 1, 20);
    let route_edge = make_edge("views.py::register", "route", "views.py", 8);
    let payload = SymbolGraphFilePayload {
        file: "views.py".to_string(),
        language: "python".to_string(),
        content_hash: "h2".to_string(),
        symbols: vec![sym],
        outgoing_calls: vec![route_edge],
    };
    let entries = detect_entry_points(&reverse_index, &[payload]);
    let hit = entries.iter().find(|e| {
        matches!(&e.reason, EntryPointReason::FrameworkRoute(r) if r == "framework:route")
    });
    assert!(hit.is_some(), "route() call should emit FrameworkRoute(framework:route)");
}

#[test]
fn call_outside_symbol_scope_skipped() {
    // If no symbol spans the call-site line, no entry should be emitted.
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let sym = make_sym("app.js::fn1", "fn1", "app.js", 1, 5);
    // Call at line 50, outside fn1's end_line=5.
    let get_edge = make_edge("app.js::fn1", "get", "app.js", 50);
    let payload = SymbolGraphFilePayload {
        file: "app.js".to_string(),
        language: "javascript".to_string(),
        content_hash: "hx".to_string(),
        symbols: vec![sym],
        outgoing_calls: vec![get_edge],
    };
    let entries = detect_entry_points(&reverse_index, &[payload]);
    let framework_entries: Vec<_> = entries
        .iter()
        .filter(|e| matches!(&e.reason, EntryPointReason::FrameworkRoute(_)))
        .collect();
    assert!(
        framework_entries.is_empty(),
        "call at line 50 outside any symbol should not emit framework entry; got: {:?}",
        framework_entries
    );
}

#[test]
fn non_verb_call_not_detected() {
    let reverse_index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let sym = make_sym("app.js::init", "init", "app.js", 1, 20);
    // "emit" is not an HTTP verb.
    let emit_edge = make_edge("app.js::init", "emit", "app.js", 5);
    let payload = SymbolGraphFilePayload {
        file: "app.js".to_string(),
        language: "javascript".to_string(),
        content_hash: "hx".to_string(),
        symbols: vec![sym],
        outgoing_calls: vec![emit_edge],
    };
    let entries = detect_entry_points(&reverse_index, &[payload]);
    let framework_entries: Vec<_> = entries
        .iter()
        .filter(|e| matches!(&e.reason, EntryPointReason::FrameworkRoute(_)))
        .collect();
    assert!(
        framework_entries.is_empty(),
        "'emit' is not a framework verb and must not be detected"
    );
}
