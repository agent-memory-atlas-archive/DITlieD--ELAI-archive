//! Contract tests for A-35: PI Multi-Language DKG Grammars.
//!
//! Validates: ext_to_lang_id mappings for all supported extensions,
//! DkgEngine.build_graph on multi-language source fixtures, per-language
//! symbol extraction via nodes(), empty-file safety, and dkg_readiness
//! after multi-language indexing.

use std::fs;
use std::path::Path;

use elai_knowledge::dkg::ext_to_lang_id;
use elai_knowledge::dkg::DkgEngine;
use elai_knowledge::{DkgReadiness, SymbolKind};

// ---------------------------------------------------------------------------
// Fixtures: minimal source strings per language
// ---------------------------------------------------------------------------

mod fixtures {
    pub const RUST_SRC: &str = r#"
use std::collections::HashMap;

pub fn greet(name: &str) -> String {
    format!("hello {}", name)
}

pub struct Config {
    pub verbose: bool,
}
"#;

    pub const PYTHON_SRC: &str = r#"
import os
from pathlib import Path

def process_data(items: list) -> dict:
    return {i: len(i) for i in items}

class DataLoader:
    def __init__(self, path):
        self.path = path
"#;

    pub const GO_SRC: &str = r#"package main

import "fmt"

func Hello(name string) string {
    return fmt.Sprintf("hello %s", name)
}

type Config struct {
    Verbose bool
}
"#;

    pub const TYPESCRIPT_SRC: &str = r#"
import { readFile } from 'fs';

export function greet(name: string): string {
    return `hello ${name}`;
}

export class DataLoader {
    constructor(private path: string) {}
}

export interface Config {
    verbose: boolean;
}
"#;

    pub const JAVA_SRC: &str = r#"
import java.util.List;

public class DataProcessor {
    public static String greet(String name) {
        return "hello " + name;
    }

    public void process(List<String> items) {
    }
}
"#;

    pub const CSHARP_SRC: &str = r#"
using System.Collections.Generic;

namespace Example {
    public class DataProcessor {
        public static string Greet(string name) {
            return "hello " + name;
        }

        public void Process(List<string> items) {
        }
    }
}
"#;

    pub const TSX_SRC: &str = r#"
import React from 'react';

export function Greeting({ name }: { name: string }): JSX.Element {
    return <div>hello {name}</div>;
}

export class AppComponent extends React.Component {
    render() {
        return <Greeting name="world" />;
    }
}
"#;

    pub const RUBY_SRC: &str = r#"
require 'json'

def process_data(items)
  items.map { |i| [i, i.length] }.to_h
end

class DataLoader
  def initialize(path)
    @path = path
  end
end
"#;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const EXPECTED_LANGUAGES: &[(&str, &str)] = &[
    ("rust", "rs"),
    ("python", "py"),
    ("go", "go"),
    ("typescript", "ts"),
    ("tsx", "tsx"),
    ("java", "java"),
    ("csharp", "cs"),
    ("ruby", "rb"),
];

const EXPECTED_EXT_MAPPINGS: &[(&str, &str)] = &[
    ("rs", "rust"),
    ("py", "python"),
    ("go", "go"),
    ("ts", "typescript"),
    ("tsx", "tsx"),
    ("java", "java"),
    ("cs", "csharp"),
    ("rb", "ruby"),
];

fn fixture_for_ext(ext: &str) -> &'static str {
    match ext {
        "rs" => fixtures::RUST_SRC,
        "py" => fixtures::PYTHON_SRC,
        "go" => fixtures::GO_SRC,
        "ts" => fixtures::TYPESCRIPT_SRC,
        "tsx" => fixtures::TSX_SRC,
        "java" => fixtures::JAVA_SRC,
        "cs" => fixtures::CSHARP_SRC,
        "rb" => fixtures::RUBY_SRC,
        _ => panic!("no fixture for extension: {ext}"),
    }
}

fn create_engine(db_path: &Path) -> DkgEngine {
    DkgEngine::new(db_path, None, 1_000_000)
        .expect("DkgEngine::new should succeed on valid path")
}

fn write_fixture_file(dir: &Path, name: &str, ext: &str, content: &str) {
    let file_path = dir.join(format!("{name}.{ext}"));
    fs::write(&file_path, content).expect("write fixture file");
}

// ---------------------------------------------------------------------------
// Contract: ext_to_lang_id covers all expected mappings
// ---------------------------------------------------------------------------

#[test]
fn contract_ext_to_lang_id_maps_all_extensions() {
    for &(ext, expected_lang) in EXPECTED_EXT_MAPPINGS {
        let result = ext_to_lang_id(ext);
        assert_eq!(
            result,
            Some(expected_lang),
            "ext_to_lang_id(\"{ext}\") should map to \"{expected_lang}\""
        );
    }
}

#[test]
fn contract_ext_to_lang_id_unknown_returns_none() {
    // C-50 + Pass D expanded the supported list: cpp/cc/cxx,
    // swift, kt/kts, scala, php, svelte, vue, sh, etc. Pick
    // extensions that genuinely have no grammar wired in
    // ext_to_lang_id (knowledge/src/dkg/mod.rs L3396-3424).
    let unknown_exts = &["h", "hpp", "hxx", "zig", "dart", "lua", "m", "mm", ""];
    for ext in unknown_exts {
        assert_eq!(
            ext_to_lang_id(ext),
            None,
            "ext_to_lang_id(\"{ext}\") should return None for unsupported extension"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract: build_graph indexes all language files and produces nodes
// ---------------------------------------------------------------------------

#[test]
fn contract_build_graph_indexes_all_languages() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let db_path = tmp.path().join("dkg.db");

    // Create a source subdirectory with one file per language.
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    for &(_lang, ext) in EXPECTED_LANGUAGES {
        write_fixture_file(&src_dir, "example", ext, fixture_for_ext(ext));
    }

    let mut engine = create_engine(&db_path);
    let graph = engine
        .build_graph(tmp.path())
        .expect("build_graph should succeed on multi-language fixtures");

    assert!(
        graph.files_parsed >= EXPECTED_LANGUAGES.len(),
        "build_graph should parse at least {} files, got {}",
        EXPECTED_LANGUAGES.len(),
        graph.files_parsed
    );
    assert!(
        graph.node_count > 0,
        "build_graph should extract at least one node"
    );
}

#[test]
fn contract_dkg_readiness_ready_after_multilang_build() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let db_path = tmp.path().join("dkg.db");
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    for &(_lang, ext) in EXPECTED_LANGUAGES {
        write_fixture_file(&src_dir, "example", ext, fixture_for_ext(ext));
    }

    let mut engine = create_engine(&db_path);
    engine
        .build_graph(tmp.path())
        .expect("build_graph should succeed");

    assert_eq!(
        engine.dkg_readiness(),
        DkgReadiness::Ready,
        "dkg_readiness should be Ready after indexing non-empty source files"
    );
}

// ---------------------------------------------------------------------------
// Contract: each language grammar extracts at least one Function node
// ---------------------------------------------------------------------------

#[test]
fn contract_python_grammar_extracts_function() {
    assert_language_extracts_function("py", "process_data");
}

#[test]
fn contract_go_grammar_extracts_function() {
    assert_language_extracts_function("go", "Hello");
}

#[test]
fn contract_typescript_grammar_extracts_function() {
    assert_language_extracts_function("ts", "greet");
}

#[test]
fn contract_tsx_grammar_extracts_function() {
    assert_language_extracts_function("tsx", "Greeting");
}

#[test]
fn contract_java_grammar_extracts_function() {
    // Java methods are extracted as Function kind.
    assert_language_extracts_function("java", "greet");
}

#[test]
fn contract_csharp_grammar_extracts_function() {
    assert_language_extracts_function("cs", "Greet");
}

#[test]
fn contract_ruby_grammar_extracts_function() {
    assert_language_extracts_function("rb", "process_data");
}

#[test]
fn contract_rust_grammar_extracts_function() {
    assert_language_extracts_function("rs", "greet");
}

fn assert_language_extracts_function(ext: &str, expected_fn_name: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let db_path = tmp.path().join("dkg.db");
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    write_fixture_file(&src_dir, "example", ext, fixture_for_ext(ext));

    let mut engine = create_engine(&db_path);
    engine
        .build_graph(tmp.path())
        .expect("build_graph should succeed");

    let nodes = engine.nodes().expect("nodes() should succeed");

    let function_nodes: Vec<_> = nodes
        .iter()
        .filter(|n| n.kind == SymbolKind::Function)
        .collect();

    assert!(
        !function_nodes.is_empty(),
        "grammar for .{ext} should extract at least one Function node, got none"
    );

    let has_expected = function_nodes
        .iter()
        .any(|n| n.symbol_name == expected_fn_name);

    assert!(
        has_expected,
        "grammar for .{ext} should extract function \"{expected_fn_name}\", \
         found: {:?}",
        function_nodes
            .iter()
            .map(|n| &n.symbol_name)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Contract: empty source files do not panic
// ---------------------------------------------------------------------------

#[test]
fn contract_empty_source_files_no_panic() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let db_path = tmp.path().join("dkg.db");
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    // Write empty files for every supported extension.
    for &(_lang, ext) in EXPECTED_LANGUAGES {
        write_fixture_file(&src_dir, "empty", ext, "");
    }

    let mut engine = create_engine(&db_path);

    // Should not panic — empty files are valid, just produce no nodes.
    let result = engine.build_graph(tmp.path());
    assert!(
        result.is_ok(),
        "build_graph should not panic/error on empty source files: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// Contract: each language extracts struct/class-like nodes
// ---------------------------------------------------------------------------

#[test]
fn contract_python_grammar_extracts_class() {
    assert_language_extracts_kind("py", SymbolKind::Struct, "DataLoader");
}

#[test]
fn contract_go_grammar_extracts_struct() {
    assert_language_extracts_kind("go", SymbolKind::Struct, "Config");
}

#[test]
fn contract_typescript_grammar_extracts_class() {
    assert_language_extracts_kind("ts", SymbolKind::Struct, "DataLoader");
}

#[test]
fn contract_tsx_grammar_extracts_class() {
    assert_language_extracts_kind("tsx", SymbolKind::Struct, "AppComponent");
}

#[test]
fn contract_tsx_jsx_syntax_does_not_fail_parse() {
    // TSX files with JSX syntax must parse without errors when using LANGUAGE_TSX.
    let tmp = tempfile::tempdir().expect("create temp dir");
    let db_path = tmp.path().join("dkg.db");
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    write_fixture_file(&src_dir, "app", "tsx", fixtures::TSX_SRC);

    let mut engine = create_engine(&db_path);
    engine
        .build_graph(tmp.path())
        .expect("build_graph should succeed on TSX with JSX syntax");

    let failed = engine.failed_files().expect("failed_files() should succeed");
    assert!(
        failed.is_empty(),
        "TSX with JSX syntax should not fail parsing, failed: {failed:?}"
    );

    let nodes = engine.nodes().expect("nodes() should succeed");
    assert!(
        !nodes.is_empty(),
        "TSX file with JSX should produce nodes"
    );
}

#[test]
fn contract_java_grammar_extracts_class() {
    assert_language_extracts_kind("java", SymbolKind::Struct, "DataProcessor");
}

#[test]
fn contract_csharp_grammar_extracts_class() {
    assert_language_extracts_kind("cs", SymbolKind::Struct, "DataProcessor");
}

#[test]
fn contract_ruby_grammar_extracts_class() {
    assert_language_extracts_kind("rb", SymbolKind::Struct, "DataLoader");
}

#[test]
fn contract_rust_grammar_extracts_struct() {
    assert_language_extracts_kind("rs", SymbolKind::Struct, "Config");
}

fn assert_language_extracts_kind(ext: &str, expected_kind: SymbolKind, expected_name: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let db_path = tmp.path().join("dkg.db");
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    write_fixture_file(&src_dir, "example", ext, fixture_for_ext(ext));

    let mut engine = create_engine(&db_path);
    engine
        .build_graph(tmp.path())
        .expect("build_graph should succeed");

    let nodes = engine.nodes().expect("nodes() should succeed");

    let matching: Vec<_> = nodes
        .iter()
        .filter(|n| n.kind == expected_kind)
        .collect();

    assert!(
        !matching.is_empty(),
        "grammar for .{ext} should extract at least one {expected_kind:?} node, got none"
    );

    let has_expected = matching.iter().any(|n| n.symbol_name == expected_name);

    assert!(
        has_expected,
        "grammar for .{ext} should extract {expected_kind:?} \"{expected_name}\", \
         found: {:?}",
        matching
            .iter()
            .map(|n| &n.symbol_name)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Contract: nodes have valid line numbers (start <= end, start > 0)
// ---------------------------------------------------------------------------

#[test]
fn contract_all_nodes_have_valid_line_ranges() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let db_path = tmp.path().join("dkg.db");
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    for &(_lang, ext) in EXPECTED_LANGUAGES {
        write_fixture_file(&src_dir, "example", ext, fixture_for_ext(ext));
    }

    let mut engine = create_engine(&db_path);
    engine
        .build_graph(tmp.path())
        .expect("build_graph should succeed");

    let nodes = engine.nodes().expect("nodes() should succeed");

    for node in &nodes {
        assert!(
            node.start_line > 0,
            "node {} in {:?} has start_line=0, expected >0",
            node.symbol_name,
            node.file_path
        );
        assert!(
            node.start_line <= node.end_line,
            "node {} in {:?} has start_line={} > end_line={}",
            node.symbol_name,
            node.file_path,
            node.start_line,
            node.end_line
        );
    }
}

// ---------------------------------------------------------------------------
// Contract: failed_files is empty for valid multi-language sources
// ---------------------------------------------------------------------------

#[test]
fn contract_no_failed_files_for_valid_sources() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let db_path = tmp.path().join("dkg.db");
    let src_dir = tmp.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    for &(_lang, ext) in EXPECTED_LANGUAGES {
        write_fixture_file(&src_dir, "example", ext, fixture_for_ext(ext));
    }

    let mut engine = create_engine(&db_path);
    engine
        .build_graph(tmp.path())
        .expect("build_graph should succeed");

    let failed = engine.failed_files().expect("failed_files() should succeed");

    assert!(
        failed.is_empty(),
        "no files should fail parsing for valid fixtures, failed: {failed:?}"
    );
}
