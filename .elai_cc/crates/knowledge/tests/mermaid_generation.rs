// B-33 Phase B — Mermaid generator unit tests.
//
// Locks the contract for `generate_mermaid(data, max_nodes) -> String`:
//   - Always begins `flowchart LR\n`.
//   - Always emits classDef block (legend stays consistent on empty too).
//   - Empty input -> header + classDef + `%% empty graph` (stub_ban).
//   - Truncation footer when files > max_nodes.
//   - Cyclic edges use `-.->|cycle|`.
//   - IDs sanitised (`/`, `.`, `-` all become `_`).
//   - Deterministic across calls (BTreeMap iteration; no HashMap).

use std::collections::BTreeMap;

use elai_knowledge::visualization::mermaid::generate_mermaid;
use elai_knowledge::visualization::types::{
    SymbolMode, VizData, VizEdgeKind, VizFile, VizFileEdge, VizProject,
};

fn empty_data() -> VizData {
    VizData::empty("demo".to_string(), SymbolMode::Full)
}

fn fixture(files: Vec<(&str, &str)>, edges: Vec<(&str, &str, bool)>) -> VizData {
    VizData {
        project: VizProject {
            name: "demo".to_string(),
            files: files.len(),
            edges: edges.len(),
            symbols: None,
            call_edges: None,
            built_at: None,
        },
        files: files
            .iter()
            .map(|(id, lang)| VizFile {
                id: id.to_string(),
                label: id.to_string(),
                language: lang.to_string(),
                deps: 0,
                dependents: 0,
                symbol_count: 0,
                // C-24 (DRIFT-7 Path A): file-mode test fixtures leave
                // level/status None; the cytoscape decomposition selectors
                // are gated by graphMode === "decomposition".
                level: None,
                status: None,
                // H-10: dead-code fields; None in test fixtures.
                is_dead_code: None,
                coverage_count: None,
            })
            .collect(),
        file_edges: edges
            .iter()
            .map(|(s, t, cyc)| VizFileEdge {
                source: s.to_string(),
                target: t.to_string(),
                kind: VizEdgeKind::Import,
                cyclic: *cyc,
            })
            .collect(),
        symbols: vec![],
        symbol_edges: vec![],
        symbols_by_file: BTreeMap::new(),
        symbol_mode: SymbolMode::Full,
    }
}

#[test]
fn mermaid_starts_with_flowchart_header() {
    let out = generate_mermaid(&empty_data(), 200);
    assert!(
        out.starts_with("flowchart LR\n"),
        "Output must start with `flowchart LR\\n`; got:\n{out}"
    );
}

#[test]
fn mermaid_classdef_per_language() {
    let out = generate_mermaid(&empty_data(), 200);
    // The legend block is always emitted (even for empty graphs) so the
    // dashboard's classDef references never dangle.
    for lang in ["rust", "python", "ts", "svelte", "go", "jvm", "csharp", "other", "cyclic"] {
        let needle = format!("classDef {lang}");
        assert!(
            out.contains(&needle),
            "Mermaid output is missing classDef for `{lang}`:\n{out}"
        );
    }
}

#[test]
fn mermaid_cyclic_edge_uses_dotted_red() {
    let data = fixture(
        vec![("a.rs", "rust"), ("b.rs", "rust")],
        vec![("a.rs", "b.rs", true), ("b.rs", "a.rs", true)],
    );
    let out = generate_mermaid(&data, 200);
    assert!(
        out.contains("-.->|cycle|"),
        "Cyclic edges must use `-.->|cycle|`; got:\n{out}"
    );
    assert!(
        out.contains(":::cyclic"),
        "Cyclic-edge endpoints must carry `:::cyclic` class; got:\n{out}"
    );
}

#[test]
fn mermaid_max_nodes_truncates() {
    let files = vec![
        ("a.rs", "rust"),
        ("b.rs", "rust"),
        ("c.rs", "rust"),
        ("d.rs", "rust"),
        ("e.rs", "rust"),
    ];
    let data = fixture(files, vec![]);
    let out = generate_mermaid(&data, 3);
    // Node-line count: lines starting with 4-space indent + alpha id +
    // `[` ... `]:::lang`. We grep for `]:::` which only appears on real nodes.
    let node_count = out.matches("]:::").count();
    assert_eq!(
        node_count, 3,
        "Expected exactly 3 node lines after truncation; got {node_count}.\nOutput:\n{out}"
    );
    assert!(
        out.contains("%% truncated: showing 3 of 5"),
        "Truncation footer missing; got:\n{out}"
    );
}

#[test]
fn mermaid_deterministic_for_fixed_input() {
    let data = fixture(
        vec![("a.rs", "rust"), ("b.rs", "python"), ("c.rs", "go")],
        vec![("a.rs", "b.rs", false), ("b.rs", "c.rs", false)],
    );
    let a = generate_mermaid(&data, 200);
    let b = generate_mermaid(&data, 200);
    assert_eq!(
        a, b,
        "generate_mermaid must produce identical output for identical input"
    );
}

#[test]
fn mermaid_empty_graph_emits_header_only() {
    let out = generate_mermaid(&empty_data(), 200);
    // Stub-ban regression: empty payload still produces valid mermaid with
    // header + classDef + `%% empty graph` comment, never an empty string.
    assert!(out.starts_with("flowchart LR\n"));
    assert!(out.contains("classDef rust"));
    assert!(out.contains("%% empty graph"));
    assert!(
        !out.is_empty(),
        "Empty input must still produce a non-empty Mermaid header (stub_ban regression)"
    );
}

#[test]
fn mermaid_sanitizes_node_ids() {
    let data = fixture(
        vec![("crates/foo/src/lib.rs", "rust"), ("3leading.py", "python")],
        vec![("crates/foo/src/lib.rs", "3leading.py", false)],
    );
    let out = generate_mermaid(&data, 200);

    // Path with slashes / dots becomes valid id with `_` substitution.
    assert!(
        out.contains("crates_foo_src_lib_rs"),
        "ID sanitisation must replace `/` and `.` with `_`; got:\n{out}"
    );
    // Leading-digit id gains `n_` prefix.
    assert!(
        out.contains("n_3leading_py"),
        "IDs starting with a digit must be prefixed with `n_`; got:\n{out}"
    );
    // Negative: ID position (before `[` or after a `-->`) must never carry
    // a raw `/`. We check edge lines explicitly: there should be no `crates/`
    // outside a Mermaid label `["..."]` block. Labels CAN contain raw paths
    // (Mermaid quotes them) so we strip everything inside `["..."]` first.
    let stripped = strip_mermaid_labels(&out);
    assert!(
        !stripped.contains("crates/"),
        "Raw path separator leaked into a Mermaid identifier (outside a quoted label); stripped output:\n{stripped}\nfull:\n{out}"
    );
}

/// Remove `["..."]` label segments from a Mermaid output so identifier-only
/// linting can ignore raw text inside labels.
fn strip_mermaid_labels(input: &str) -> String {
    let mut buf = String::with_capacity(input.len());
    let mut in_label = false;
    let mut prev = '\0';
    for ch in input.chars() {
        if !in_label && prev == '[' && ch == '"' {
            in_label = true;
            // Drop the leading `[` we just emitted plus the opening `"`.
            buf.pop();
            prev = ch;
            continue;
        }
        if in_label && ch == ']' && prev == '"' {
            in_label = false;
            prev = ch;
            continue;
        }
        if !in_label {
            buf.push(ch);
        }
        prev = ch;
    }
    buf
}
