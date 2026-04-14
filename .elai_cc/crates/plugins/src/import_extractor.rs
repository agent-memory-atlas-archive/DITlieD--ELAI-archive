//! Language-parameterized AST import extraction via tree-sitter.
//!
//! Extracts import/use/dependency declarations from source code across 7 languages,
//! then cross-references against a deprecated patterns list.
//!
//! Performance target: <10ms for files under 5000 lines (shared with T2 budget).

use tree_sitter::StreamingIterator;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A deprecated import pattern for cross-referencing.
#[derive(Debug, Clone)]
pub struct DeprecatedImport {
    pub ecosystem: String,
    pub deprecated_name: String,
    pub replacement_name: String,
    pub severity: String,
    pub reason: String,
}

/// Result of a deprecated import check on a single file.
#[derive(Debug, Clone)]
pub struct ImportViolation {
    pub import_name: String,
    pub deprecated_name: String,
    pub replacement_name: String,
    pub severity: String,
    pub line: u32,
}

/// Supported languages for AST import extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportLanguage {
    Rust,
    Python,
    TypeScript,
    Go,
    Java,
    CSharp,
    Ruby,
}

impl ImportLanguage {
    /// Detect language from file extension.
    pub fn from_extension(file_path: &str) -> Option<Self> {
        let ext = file_path.rsplit('.').next()?;
        match ext {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "cs" => Some(Self::CSharp),
            "rb" => Some(Self::Ruby),
            _ => None,
        }
    }

    /// Map language to ecosystem identifier matching deprecated_patterns table.
    pub fn ecosystem(&self) -> &'static str {
        match self {
            Self::Rust => "cargo",
            Self::Python => "pip",
            Self::TypeScript => "npm",
            Self::Go => "go",
            Self::Java => "maven",
            Self::CSharp => "nuget",
            Self::Ruby => "bundler",
        }
    }

    fn ts_language(&self) -> tree_sitter::Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::Java => tree_sitter_java::LANGUAGE.into(),
            Self::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            Self::Ruby => tree_sitter_ruby::LANGUAGE.into(),
        }
    }

    /// Tree-sitter query for extracting import nodes per language.
    fn import_query(&self) -> &'static str {
        match self {
            // Rust: capture the full use argument, then normalize to crate root.
            // Handles: use foo, use foo::bar, use foo::bar::Baz, use foo::{A, B}
            Self::Rust => r#"
                (use_declaration argument: (_) @import)
                (extern_crate_declaration name: (identifier) @import)
            "#,
            // Python: import X, from X import Y
            Self::Python => r#"
                (import_statement name: (dotted_name (identifier) @import))
                (import_from_statement module_name: (dotted_name (identifier) @import))
                (import_from_statement module_name: (relative_import (dotted_name (identifier) @import)))
            "#,
            // TypeScript/JS: import ... from "X", require("X")
            Self::TypeScript => r#"
                (import_statement source: (string (string_fragment) @import))
                (call_expression
                    function: (identifier) @_req
                    arguments: (arguments (string (string_fragment) @import))
                    (#eq? @_req "require"))
            "#,
            // Go: import "path/to/pkg"
            Self::Go => r#"
                (import_spec path: (interpreted_string_literal) @import)
            "#,
            // Java: import com.example.Foo
            Self::Java => r#"
                (import_declaration (scoped_identifier) @import)
            "#,
            // C#: using Namespace.Something
            Self::CSharp => r#"
                (using_directive (qualified_name) @import)
                (using_directive (identifier) @import)
            "#,
            // Ruby: require "foo", gem "foo"
            Self::Ruby => r#"
                (call
                    method: (identifier) @_method
                    arguments: (argument_list (string (string_content) @import))
                    (#match? @_method "^(require|require_relative|gem)$"))
            "#,
        }
    }
}

/// Extracted import from source code.
#[derive(Debug, Clone)]
pub struct ExtractedImport {
    pub name: String,
    pub line: u32,
}

// ---------------------------------------------------------------------------
// Import extraction
// ---------------------------------------------------------------------------

/// Extract import declarations from source code using tree-sitter AST.
///
/// Returns the list of extracted import names with line numbers.
/// Returns an empty vec for unsupported languages or parse failures.
pub fn extract_imports(
    source: &str,
    language: ImportLanguage,
) -> Result<Vec<ExtractedImport>, ImportExtractorError> {
    let ts_lang = language.ts_language();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&ts_lang)
        .map_err(|e| ImportExtractorError::ParserInit(format!("{e}")))?;

    let tree = parser
        .parse(source.as_bytes(), None)
        .ok_or_else(|| ImportExtractorError::ParseFailed)?;

    let query_src = language.import_query();
    let query = tree_sitter::Query::new(&ts_lang, query_src)
        .map_err(|e| ImportExtractorError::QueryFailed(format!("{e}")))?;

    // Find the capture index for "import" (not helper captures like @_req, @_method)
    let import_idx = query
        .capture_names()
        .iter()
        .position(|name| *name == "import")
        .ok_or_else(|| ImportExtractorError::QueryFailed("no @import capture".to_string()))?;

    let mut cursor = tree_sitter::QueryCursor::new();
    let source_bytes = source.as_bytes();
    let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

    let mut imports = Vec::new();
    while let Some(m) = matches.next() {
        for capture in m.captures {
            if capture.index as usize != import_idx {
                continue;
            }
            let node = capture.node;
            let raw_text = node
                .utf8_text(source_bytes)
                .map_err(|e| ImportExtractorError::Utf8Error(format!("{e}")))?;

            // Normalize: strip quotes for Go string literals, extract root module
            let name = normalize_import_name(raw_text, language);
            if !name.is_empty() {
                imports.push(ExtractedImport {
                    name,
                    line: node.start_position().row as u32 + 1,
                });
            }
        }
    }

    // Deduplicate by name (keep first occurrence for line number)
    let mut seen = std::collections::HashSet::new();
    imports.retain(|imp| seen.insert(imp.name.clone()));

    Ok(imports)
}

/// Normalize raw import text to a comparable name.
fn normalize_import_name(raw: &str, language: ImportLanguage) -> String {
    let trimmed = raw.trim().trim_matches('"').trim_matches('\'');
    match language {
        // Rust: first path segment (e.g. "std" from "std::collections::HashMap")
        ImportLanguage::Rust => trimmed.split("::").next().unwrap_or("").to_string(),
        // Python: first module segment (e.g. "torch" from "torch.no_grad")
        ImportLanguage::Python => trimmed.split('.').next().unwrap_or("").to_string(),
        // TypeScript: package name (strip @scope/ prefix for scoped packages)
        ImportLanguage::TypeScript => {
            let stripped = trimmed.trim_start_matches("./").trim_start_matches("../");
            if stripped.starts_with('@') {
                // @scope/pkg -> @scope/pkg (keep full scoped name)
                stripped.split('/').take(2).collect::<Vec<_>>().join("/")
            } else {
                stripped.split('/').next().unwrap_or("").to_string()
            }
        }
        // Go: last path segment for stdlib, full path for external
        ImportLanguage::Go => trimmed.to_string(),
        // Java: root package (e.g. "com" from "com.example.Foo") — but more useful
        // is the group artifact which is typically the first 2-3 segments
        ImportLanguage::Java => {
            // Return full qualified name; matching does prefix check
            trimmed.to_string()
        }
        // C#: root namespace
        ImportLanguage::CSharp => trimmed.split('.').next().unwrap_or("").to_string(),
        // Ruby: gem name
        ImportLanguage::Ruby => trimmed.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Deprecated pattern cross-reference
// ---------------------------------------------------------------------------

/// Check extracted imports against a list of deprecated patterns.
///
/// Filters patterns by ecosystem matching the detected language.
/// Returns violations sorted by severity (BLOCK first).
pub fn check_deprecated_imports(
    imports: &[ExtractedImport],
    deprecated: &[DeprecatedImport],
    language: ImportLanguage,
) -> Vec<ImportViolation> {
    let ecosystem = language.ecosystem();
    let mut violations = Vec::new();

    for import in imports {
        for dep in deprecated {
            if dep.ecosystem != ecosystem {
                continue;
            }
            if matches_deprecated(&import.name, &dep.deprecated_name) {
                violations.push(ImportViolation {
                    import_name: import.name.clone(),
                    deprecated_name: dep.deprecated_name.clone(),
                    replacement_name: dep.replacement_name.clone(),
                    severity: dep.severity.clone(),
                    line: import.line,
                });
                break; // One violation per import
            }
        }
    }

    // Sort: BLOCK before WARN
    violations.sort_by(|a, b| {
        let a_block = a.severity == "BLOCK";
        let b_block = b.severity == "BLOCK";
        b_block.cmp(&a_block)
    });

    violations
}

/// Check if an extracted import name matches a deprecated pattern name.
///
/// Matching rules:
/// - Exact match: "blake2" == "blake2"
/// - Prefix match: extracted "rand" matches deprecated "rand 0.8"
///   (deprecated_name's first token before space matches extracted)
/// - Dotted prefix: extracted "http" matches deprecated "http.server"
///   (deprecated_name's first segment before dot matches extracted)
/// - Dotted prefix: extracted "torch" matches deprecated "torch.no_grad"
fn matches_deprecated(extracted: &str, deprecated_name: &str) -> bool {
    if extracted == deprecated_name {
        return true;
    }

    // First token of deprecated name (split on space)
    let first_token = deprecated_name.split_whitespace().next().unwrap_or(deprecated_name);

    // First segment of first_token (split on dot)
    let first_segment = first_token.split('.').next().unwrap_or(first_token);

    // Check if extracted matches the root identifier
    extracted == first_token || extracted == first_segment
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ImportExtractorError {
    ParserInit(String),
    ParseFailed,
    QueryFailed(String),
    Utf8Error(String),
}

impl std::fmt::Display for ImportExtractorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParserInit(msg) => write!(f, "parser init: {msg}"),
            Self::ParseFailed => write!(f, "tree-sitter parse returned None"),
            Self::QueryFailed(msg) => write!(f, "query failed: {msg}"),
            Self::Utf8Error(msg) => write!(f, "utf8 error: {msg}"),
        }
    }
}

impl std::error::Error for ImportExtractorError {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_deprecated_patterns() -> Vec<DeprecatedImport> {
        vec![
            DeprecatedImport {
                ecosystem: "cargo".to_string(),
                deprecated_name: "blake2".to_string(),
                replacement_name: "blake3".to_string(),
                severity: "BLOCK".to_string(),
                reason: "blake3 is faster".to_string(),
            },
            DeprecatedImport {
                ecosystem: "cargo".to_string(),
                deprecated_name: "log".to_string(),
                replacement_name: "tracing".to_string(),
                severity: "BLOCK".to_string(),
                reason: "tracing is modern".to_string(),
            },
            DeprecatedImport {
                ecosystem: "cargo".to_string(),
                deprecated_name: "rand 0.8".to_string(),
                replacement_name: "rand >= 0.9".to_string(),
                severity: "BLOCK".to_string(),
                reason: "outdated".to_string(),
            },
            DeprecatedImport {
                ecosystem: "pip".to_string(),
                deprecated_name: "http.server".to_string(),
                replacement_name: "fastapi+uvicorn".to_string(),
                severity: "BLOCK".to_string(),
                reason: "single-threaded".to_string(),
            },
            DeprecatedImport {
                ecosystem: "pip".to_string(),
                deprecated_name: "torch.no_grad".to_string(),
                replacement_name: "torch.inference_mode".to_string(),
                severity: "WARN".to_string(),
                reason: "inference_mode is faster".to_string(),
            },
            DeprecatedImport {
                ecosystem: "npm".to_string(),
                deprecated_name: "request".to_string(),
                replacement_name: "undici/fetch".to_string(),
                severity: "WARN".to_string(),
                reason: "deprecated".to_string(),
            },
        ]
    }

    // -- Rust --

    #[test]
    fn extract_rust_use_declarations() {
        let source = r#"
use blake2::Digest;
use std::collections::HashMap;
use tracing::info;
"#;
        let imports = extract_imports(source, ImportLanguage::Rust).unwrap();
        let names: Vec<&str> = imports.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"blake2"));
        assert!(names.contains(&"std"));
        assert!(names.contains(&"tracing"));
    }

    #[test]
    fn extract_rust_extern_crate() {
        let source = "extern crate log;";
        let imports = extract_imports(source, ImportLanguage::Rust).unwrap();
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].name, "log");
    }

    #[test]
    fn rust_deprecated_check_finds_blake2() {
        let source = "use blake2::Digest;\nuse blake3::Hasher;";
        let imports = extract_imports(source, ImportLanguage::Rust).unwrap();
        let violations = check_deprecated_imports(&imports, &test_deprecated_patterns(), ImportLanguage::Rust);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].import_name, "blake2");
        assert_eq!(violations[0].severity, "BLOCK");
    }

    #[test]
    fn rust_no_false_positive_for_allowed_crate() {
        let source = "use blake3::Hasher;\nuse tracing::info;";
        let imports = extract_imports(source, ImportLanguage::Rust).unwrap();
        let violations = check_deprecated_imports(&imports, &test_deprecated_patterns(), ImportLanguage::Rust);
        assert!(violations.is_empty());
    }

    #[test]
    fn rust_extern_crate_log_detected() {
        let source = "extern crate log;";
        let imports = extract_imports(source, ImportLanguage::Rust).unwrap();
        let violations = check_deprecated_imports(&imports, &test_deprecated_patterns(), ImportLanguage::Rust);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].deprecated_name, "log");
    }

    // -- Python --

    #[test]
    fn extract_python_imports() {
        let source = r#"
import http.server
from torch import no_grad
import os
"#;
        let imports = extract_imports(source, ImportLanguage::Python).unwrap();
        let names: Vec<&str> = imports.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"http"));
        assert!(names.contains(&"torch"));
        assert!(names.contains(&"os"));
    }

    #[test]
    fn python_deprecated_check_finds_http_server() {
        let source = "import http.server\nimport os";
        let imports = extract_imports(source, ImportLanguage::Python).unwrap();
        let violations = check_deprecated_imports(&imports, &test_deprecated_patterns(), ImportLanguage::Python);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].deprecated_name, "http.server");
    }

    #[test]
    fn python_torch_no_false_positive_on_different_ecosystem() {
        // "log" is a cargo deprecated pattern, not pip — should not match Python's "log"
        let source = "import logging";
        let imports = extract_imports(source, ImportLanguage::Python).unwrap();
        let violations = check_deprecated_imports(&imports, &test_deprecated_patterns(), ImportLanguage::Python);
        assert!(violations.is_empty());
    }

    // -- TypeScript --

    #[test]
    fn extract_typescript_imports() {
        let source = r#"
import { Router } from "express";
import React from "react";
const fs = require("fs");
"#;
        let imports = extract_imports(source, ImportLanguage::TypeScript).unwrap();
        let names: Vec<&str> = imports.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"express"));
        assert!(names.contains(&"react"));
        assert!(names.contains(&"fs"));
    }

    // -- Go --

    #[test]
    fn extract_go_imports() {
        let source = r#"
package main

import (
    "fmt"
    "net/http"
)
"#;
        let imports = extract_imports(source, ImportLanguage::Go).unwrap();
        let names: Vec<&str> = imports.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"fmt"));
        assert!(names.contains(&"net/http"));
    }

    // -- Java --

    #[test]
    fn extract_java_imports() {
        let source = r#"
import java.util.HashMap;
import com.google.gson.Gson;
"#;
        let imports = extract_imports(source, ImportLanguage::Java).unwrap();
        assert!(imports.len() >= 2);
    }

    // -- C# --

    #[test]
    fn extract_csharp_imports() {
        let source = r#"
using System;
using System.Collections.Generic;
"#;
        let imports = extract_imports(source, ImportLanguage::CSharp).unwrap();
        let names: Vec<&str> = imports.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"System"));
    }

    // -- Ruby --

    #[test]
    fn extract_ruby_imports() {
        let source = r#"
require "json"
require "net/http"
gem "rails"
"#;
        let imports = extract_imports(source, ImportLanguage::Ruby).unwrap();
        let names: Vec<&str> = imports.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"json"));
    }

    // -- Cross-cutting --

    #[test]
    fn language_from_extension() {
        assert_eq!(ImportLanguage::from_extension("foo.rs"), Some(ImportLanguage::Rust));
        assert_eq!(ImportLanguage::from_extension("bar.py"), Some(ImportLanguage::Python));
        assert_eq!(ImportLanguage::from_extension("baz.ts"), Some(ImportLanguage::TypeScript));
        assert_eq!(ImportLanguage::from_extension("main.go"), Some(ImportLanguage::Go));
        assert_eq!(ImportLanguage::from_extension("Foo.java"), Some(ImportLanguage::Java));
        assert_eq!(ImportLanguage::from_extension("Bar.cs"), Some(ImportLanguage::CSharp));
        assert_eq!(ImportLanguage::from_extension("gem.rb"), Some(ImportLanguage::Ruby));
        assert_eq!(ImportLanguage::from_extension("data.csv"), None);
    }

    #[test]
    fn ecosystem_mapping() {
        assert_eq!(ImportLanguage::Rust.ecosystem(), "cargo");
        assert_eq!(ImportLanguage::Python.ecosystem(), "pip");
        assert_eq!(ImportLanguage::TypeScript.ecosystem(), "npm");
        assert_eq!(ImportLanguage::Go.ecosystem(), "go");
        assert_eq!(ImportLanguage::Java.ecosystem(), "maven");
        assert_eq!(ImportLanguage::CSharp.ecosystem(), "nuget");
        assert_eq!(ImportLanguage::Ruby.ecosystem(), "bundler");
    }

    #[test]
    fn matches_deprecated_exact() {
        assert!(matches_deprecated("blake2", "blake2"));
        assert!(!matches_deprecated("blake3", "blake2"));
    }

    #[test]
    fn matches_deprecated_space_prefix() {
        assert!(matches_deprecated("rand", "rand 0.8"));
        assert!(!matches_deprecated("random", "rand 0.8"));
    }

    #[test]
    fn matches_deprecated_dot_prefix() {
        assert!(matches_deprecated("http", "http.server"));
        assert!(matches_deprecated("torch", "torch.no_grad"));
        assert!(!matches_deprecated("httpx", "http.server"));
    }

    #[test]
    fn ecosystem_filter_prevents_cross_language() {
        let patterns = test_deprecated_patterns();
        // "log" is cargo-ecosystem deprecated, should not match in Python
        let py_imports = vec![ExtractedImport { name: "log".to_string(), line: 1 }];
        let violations = check_deprecated_imports(&py_imports, &patterns, ImportLanguage::Python);
        assert!(violations.is_empty(), "cargo 'log' should not match in Python");
    }

    #[test]
    fn violations_sorted_block_first() {
        let patterns = test_deprecated_patterns();
        // torch matches WARN (torch.no_grad), http matches BLOCK (http.server)
        let imports = vec![
            ExtractedImport { name: "torch".to_string(), line: 1 },
            ExtractedImport { name: "http".to_string(), line: 2 },
        ];
        let violations = check_deprecated_imports(&imports, &patterns, ImportLanguage::Python);
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].severity, "BLOCK");
        assert_eq!(violations[1].severity, "WARN");
    }

    #[test]
    fn empty_source_returns_empty() {
        let imports = extract_imports("", ImportLanguage::Rust).unwrap();
        assert!(imports.is_empty());
    }
}
