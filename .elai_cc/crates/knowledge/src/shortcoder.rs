//! ShortCoder: AST-preserving code simplification for context compression.
//!
//! Applies tree-sitter-based simplification rules to code snippets before they
//! enter the CodeContext slot (slot 5) of the context budget. Reduces token
//! count while preserving semantic structure (signatures, types, control flow).
//!
//! Research grounding: ShortCoder (arXiv:2601.09703) demonstrated 18-38% token
//! reduction via AST-level simplification. Our adaptation targets *input context*
//! compression rather than output generation, using tree-sitter across 7 languages.

use std::collections::HashMap;
use std::fmt;

use tracing::debug;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of simplification rules.
pub const RULE_COUNT: usize = 10;

/// Placeholder for collapsed function/method bodies.
const BODY_PLACEHOLDER: &str = " { ... }";
/// Placeholder for collapsed Python function bodies.
const PY_BODY_PLACEHOLDER: &str = " ...";
/// Placeholder for truncated string literals beyond this byte length.
const STRING_TRUNCATION_THRESHOLD: usize = 40;
/// Placeholder suffix for truncated strings.
const STRING_TRUNCATION_SUFFIX: &str = "...\"";
/// Maximum consecutive empty lines to keep.
const MAX_CONSECUTIVE_EMPTY_LINES: usize = 1;

// ---------------------------------------------------------------------------
// Rule identifiers
// ---------------------------------------------------------------------------

/// Identifies which simplification rule was applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RuleId {
    /// R1: Collapse function/method bodies to signatures only.
    CollapseBodies = 0,
    /// R2: Strip single-line comments.
    StripLineComments = 1,
    /// R3: Strip block/doc comments.
    StripBlockComments = 2,
    /// R4: Strip docstrings (Python/Ruby).
    StripDocstrings = 3,
    /// R5: Elide type annotations on `let` bindings with obvious RHS (Rust/TS).
    ElideObviousTypes = 4,
    /// R6: Collapse import/use blocks to summary line.
    CollapseImports = 5,
    /// R7: Truncate long string literals.
    TruncateStrings = 6,
    /// R8: Remove excessive empty lines.
    RemoveEmptyLines = 7,
    /// R9: Strip decorator/attribute annotations.
    StripDecorators = 8,
    /// R10: Remove trailing whitespace per line.
    TrimTrailingWhitespace = 9,
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::CollapseBodies => "collapse_bodies",
            Self::StripLineComments => "strip_line_comments",
            Self::StripBlockComments => "strip_block_comments",
            Self::StripDocstrings => "strip_docstrings",
            Self::ElideObviousTypes => "elide_obvious_types",
            Self::CollapseImports => "collapse_imports",
            Self::TruncateStrings => "truncate_strings",
            Self::RemoveEmptyLines => "remove_empty_lines",
            Self::StripDecorators => "strip_decorators",
            Self::TrimTrailingWhitespace => "trim_trailing_ws",
        };
        f.write_str(label)
    }
}

// ---------------------------------------------------------------------------
// Result type
// ---------------------------------------------------------------------------

/// Outcome of simplifying a code snippet.
#[derive(Debug, Clone)]
pub struct SimplificationResult {
    /// The simplified source code.
    pub simplified: String,
    /// Byte length of the original source.
    pub original_bytes: usize,
    /// Byte length of the simplified source.
    pub simplified_bytes: usize,
    /// Which rules were applied (non-empty = at least one change).
    pub rules_applied: Vec<RuleId>,
}

impl SimplificationResult {
    /// Compression ratio as a fraction in [0.0, 1.0]. Lower = more compression.
    pub fn compression_ratio(&self) -> f64 {
        if self.original_bytes == 0 {
            return 1.0;
        }
        self.simplified_bytes as f64 / self.original_bytes as f64
    }

    /// Whether any simplification was applied.
    pub fn was_simplified(&self) -> bool {
        !self.rules_applied.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ShortCoderError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("parse failed: {0}")]
    ParseFailed(String),
}

// ---------------------------------------------------------------------------
// ShortCoder engine
// ---------------------------------------------------------------------------

/// AST-preserving code simplifier for context compression.
///
/// Holds tree-sitter language grammars and applies 10 deterministic
/// simplification rules to reduce token count while preserving semantic
/// structure needed by the local model.
pub struct ShortCoder {
    languages: HashMap<String, tree_sitter::Language>,
}

impl ShortCoder {
    /// Create a new ShortCoder with all 7 supported languages.
    pub fn new() -> Self {
        let mut languages = HashMap::with_capacity(7);
        languages.insert("rust".to_string(), tree_sitter_rust::LANGUAGE.into());
        languages.insert("python".to_string(), tree_sitter_python::LANGUAGE.into());
        languages.insert(
            "typescript".to_string(),
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        );
        languages.insert("go".to_string(), tree_sitter_go::LANGUAGE.into());
        languages.insert("java".to_string(), tree_sitter_java::LANGUAGE.into());
        languages.insert("c_sharp".to_string(), tree_sitter_c_sharp::LANGUAGE.into());
        languages.insert("ruby".to_string(), tree_sitter_ruby::LANGUAGE.into());
        Self { languages }
    }

    /// Simplify source code for context compression.
    ///
    /// Applies all 10 rules in a single pass. Returns the simplified source
    /// and metadata about which rules fired. If the language is unsupported or
    /// parsing fails, returns an error — callers should fall back to the
    /// original source.
    pub fn simplify(
        &self,
        source: &str,
        language: &str,
    ) -> Result<SimplificationResult, ShortCoderError> {
        let lang = self
            .languages
            .get(language)
            .ok_or_else(|| ShortCoderError::UnsupportedLanguage(language.to_string()))?;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(lang)
            .map_err(|e| ShortCoderError::ParseFailed(format!("set_language: {e}")))?;

        let tree = parser
            .parse(source.as_bytes(), None)
            .ok_or_else(|| ShortCoderError::ParseFailed("parser returned no tree".to_string()))?;

        let original_bytes = source.len();
        let mut rules_applied = Vec::new();

        // Phase 1: Collect AST-based replacements (byte ranges to replace).
        let mut replacements = Vec::new();
        collect_replacements(
            &tree,
            source.as_bytes(),
            language,
            &mut replacements,
            &mut rules_applied,
        );

        // Sort replacements by start offset descending so we can apply back-to-front
        // without invalidating earlier offsets.
        replacements.sort_by(|a, b| b.start.cmp(&a.start));

        // Apply replacements.
        let mut result = source.to_string();
        for rep in &replacements {
            if rep.start <= result.len() && rep.end <= result.len() && rep.start <= rep.end {
                result.replace_range(rep.start..rep.end, &rep.replacement);
            }
        }

        // Phase 2: Line-level rules (empty lines, trailing whitespace).
        let (line_result, line_rules) = apply_line_rules(&result);
        rules_applied.extend(line_rules);
        let simplified = line_result;

        // Deduplicate rule IDs.
        rules_applied.sort_by_key(|r| *r as u8);
        rules_applied.dedup();

        let simplified_bytes = simplified.len();
        debug!(
            language,
            original_bytes,
            simplified_bytes,
            rules = rules_applied.len(),
            "shortcoder simplification complete"
        );

        Ok(SimplificationResult {
            simplified,
            original_bytes,
            simplified_bytes,
            rules_applied,
        })
    }
}

// ---------------------------------------------------------------------------
// Replacement collection (AST-driven rules)
// ---------------------------------------------------------------------------

struct Replacement {
    start: usize,
    end: usize,
    replacement: String,
}

fn collect_replacements(
    tree: &tree_sitter::Tree,
    source: &[u8],
    language: &str,
    replacements: &mut Vec<Replacement>,
    rules: &mut Vec<RuleId>,
) {
    let root = tree.root_node();
    collect_node_replacements(root, source, language, replacements, rules, 0);
}

fn collect_node_replacements(
    node: tree_sitter::Node,
    source: &[u8],
    language: &str,
    replacements: &mut Vec<Replacement>,
    rules: &mut Vec<RuleId>,
    depth: usize,
) {
    let kind = node.kind();

    // R1: Collapse function/method bodies (only top-level and impl-level, depth <= 2).
    if depth <= 2 {
        if let Some(rep) = try_collapse_body(node, source, language, kind) {
            replacements.push(rep);
            rules.push(RuleId::CollapseBodies);
            // Don't recurse into collapsed bodies.
            return;
        }
    }

    // R2: Strip line comments.
    if is_line_comment(kind, language) {
        let start = node.start_byte();
        let end = node.end_byte();
        // Remove the entire line if the comment is the only content.
        let line_start = find_line_start(source, start);
        let line_end = find_line_end(source, end);
        if source[line_start..start].iter().all(|&b| b == b' ' || b == b'\t') {
            replacements.push(Replacement {
                start: line_start,
                end: line_end,
                replacement: String::new(),
            });
        } else {
            replacements.push(Replacement {
                start,
                end,
                replacement: String::new(),
            });
        }
        rules.push(RuleId::StripLineComments);
        return;
    }

    // R3: Strip block comments.
    if is_block_comment(kind, language) {
        let start = node.start_byte();
        let end = node.end_byte();
        let line_start = find_line_start(source, start);
        let line_end = find_line_end(source, end);
        if source[line_start..start].iter().all(|&b| b == b' ' || b == b'\t') {
            replacements.push(Replacement {
                start: line_start,
                end: line_end,
                replacement: String::new(),
            });
        } else {
            replacements.push(Replacement {
                start,
                end,
                replacement: String::new(),
            });
        }
        rules.push(RuleId::StripBlockComments);
        return;
    }

    // R4: Strip Python/Ruby docstrings (expression_statement containing a string).
    if language == "python" && kind == "expression_statement" && child_count_u32(node) == 1 {
        if let Some(child) = node.child(0) {
            if child.kind() == "string" {
                let text = child.utf8_text(source).unwrap_or("");
                if text.starts_with("\"\"\"") || text.starts_with("'''") {
                    let line_start = find_line_start(source, node.start_byte());
                    let line_end = find_line_end(source, node.end_byte());
                    replacements.push(Replacement {
                        start: line_start,
                        end: line_end,
                        replacement: String::new(),
                    });
                    rules.push(RuleId::StripDocstrings);
                    return;
                }
            }
        }
    }

    // R5: Elide obvious type annotations on let bindings (Rust: `let x: i32 = 42;`).
    if language == "rust" && kind == "let_declaration" {
        if let Some(rep) = try_elide_rust_type(node, source) {
            replacements.push(rep);
            rules.push(RuleId::ElideObviousTypes);
        }
    }

    // R6: Collapse consecutive use/import statements.
    if is_import_node(kind, language) {
        if let Some(rep) = try_collapse_imports(node, source, language, kind) {
            replacements.push(rep);
            rules.push(RuleId::CollapseImports);
            return;
        }
    }

    // R7: Truncate long string literals.
    if is_string_literal(kind) {
        let text = node.utf8_text(source).unwrap_or("");
        if text.len() > STRING_TRUNCATION_THRESHOLD {
            let truncated = format!(
                "{}{}",
                &text[..STRING_TRUNCATION_THRESHOLD.min(text.len())],
                STRING_TRUNCATION_SUFFIX
            );
            replacements.push(Replacement {
                start: node.start_byte(),
                end: node.end_byte(),
                replacement: truncated,
            });
            rules.push(RuleId::TruncateStrings);
            return;
        }
    }

    // R9: Strip decorators/attributes.
    if is_decorator(kind, language) {
        let line_start = find_line_start(source, node.start_byte());
        let line_end = find_line_end(source, node.end_byte());
        replacements.push(Replacement {
            start: line_start,
            end: line_end,
            replacement: String::new(),
        });
        rules.push(RuleId::StripDecorators);
        return;
    }

    // Recurse into children.
    for i in 0..child_count_u32(node) {
        if let Some(child) = node.child(i) {
            collect_node_replacements(child, source, language, replacements, rules, depth + 1);
        }
    }
}

// ---------------------------------------------------------------------------
// Rule helpers
// ---------------------------------------------------------------------------

/// R1: Try to collapse a function/method body to its signature.
fn try_collapse_body(
    node: tree_sitter::Node,
    source: &[u8],
    language: &str,
    kind: &str,
) -> Option<Replacement> {
    let (sig_end_kind, body_kind, placeholder) = match language {
        "rust" => match kind {
            "function_item" => (Some("parameters"), "block", BODY_PLACEHOLDER),
            _ => return None,
        },
        "python" => match kind {
            "function_definition" => (Some("parameters"), "block", PY_BODY_PLACEHOLDER),
            _ => return None,
        },
        "go" => match kind {
            "function_declaration" | "method_declaration" => {
                (Some("parameter_list"), "block", BODY_PLACEHOLDER)
            }
            _ => return None,
        },
        "typescript" => match kind {
            "function_declaration" | "method_definition" => {
                (Some("formal_parameters"), "statement_block", BODY_PLACEHOLDER)
            }
            _ => return None,
        },
        "java" => match kind {
            "method_declaration" | "constructor_declaration" => {
                (Some("formal_parameters"), "block", BODY_PLACEHOLDER)
            }
            _ => return None,
        },
        "c_sharp" => match kind {
            "method_declaration" | "constructor_declaration" => {
                (Some("parameter_list"), "block", BODY_PLACEHOLDER)
            }
            _ => return None,
        },
        "ruby" => match kind {
            "method" => (None, "body_statement", BODY_PLACEHOLDER),
            _ => return None,
        },
        _ => return None,
    };

    // Find the body child node.
    let mut body_node = None;
    let mut sig_end_byte = node.start_byte();
    let cc = child_count_u32(node);
    for i in 0..cc {
        if let Some(child) = node.child(i) {
            if child.kind() == body_kind {
                body_node = Some(child);
                break;
            }
            // Track end of signature (last node before body, or specific sig_end_kind).
            if let Some(sek) = sig_end_kind {
                if child.kind() == sek {
                    sig_end_byte = child.end_byte();
                }
            } else {
                sig_end_byte = child.end_byte();
            }
        }
    }

    let body = body_node?;
    // Only collapse bodies with more than 1 line (trivial bodies are already short).
    let body_lines = body.end_position().row.saturating_sub(body.start_position().row);
    if body_lines < 2 {
        return None;
    }

    // For Rust, include return type in signature if present.
    let mut actual_sig_end = sig_end_byte;
    if language == "rust" {
        for i in 0..cc {
            if let Some(child) = node.child(i) {
                // Return type comes after parameters, before block.
                if child.kind() == "type_identifier"
                    || child.kind() == "generic_type"
                    || child.kind() == "scoped_type_identifier"
                    || child.kind() == "->"
                {
                    actual_sig_end = child.end_byte();
                }
                if child.kind() == body_kind {
                    break;
                }
            }
        }
    }
    let _ = actual_sig_end; // Rust return type tracking — used via sig_end_byte fallback.

    // Build signature text + placeholder.
    let sig_text = std::str::from_utf8(&source[node.start_byte()..body.start_byte()])
        .unwrap_or("")
        .trim_end();

    let replacement = format!("{sig_text}{placeholder}");

    Some(Replacement {
        start: node.start_byte(),
        end: node.end_byte(),
        replacement,
    })
}

fn is_line_comment(kind: &str, language: &str) -> bool {
    match language {
        "rust" | "go" | "java" | "c_sharp" | "typescript" => kind == "line_comment",
        "python" => kind == "comment",
        "ruby" => kind == "comment",
        _ => false,
    }
}

fn is_block_comment(kind: &str, language: &str) -> bool {
    match language {
        "rust" | "go" | "java" | "c_sharp" | "typescript" => kind == "block_comment",
        // Python and Ruby don't have block comments (docstrings handled by R4).
        _ => false,
    }
}

fn is_import_node(kind: &str, language: &str) -> bool {
    match language {
        "rust" => kind == "use_declaration",
        "python" => kind == "import_statement" || kind == "import_from_statement",
        "go" => kind == "import_declaration",
        "typescript" => kind == "import_statement",
        "java" => kind == "import_declaration",
        "c_sharp" => kind == "using_directive",
        "ruby" => kind == "call" , // require/include — too ambiguous, skip
        _ => false,
    }
}

fn is_string_literal(kind: &str) -> bool {
    matches!(
        kind,
        "string_literal"
            | "raw_string_literal"
            | "string"
            | "template_string"
            | "interpreted_string_literal"
    )
}

fn is_decorator(kind: &str, language: &str) -> bool {
    match language {
        "python" => kind == "decorator",
        "java" | "c_sharp" => kind == "annotation" || kind == "attribute",
        "typescript" => kind == "decorator",
        _ => false,
    }
}

/// R5: Elide type annotation on a Rust let binding when RHS is a literal.
fn try_elide_rust_type(node: tree_sitter::Node, _source: &[u8]) -> Option<Replacement> {
    // Pattern: `let name: Type = literal_expr;`
    // We want to remove `: Type` when the RHS is an obvious literal.
    let mut has_type = false;
    let mut type_start = 0usize;
    let mut type_end = 0usize;
    let mut has_value = false;
    let mut value_is_literal = false;

    let cc = child_count_u32(node);
    for i in 0..cc {
        if let Some(child) = node.child(i) {
            if child.kind() == ":" {
                has_type = true;
                type_start = child.start_byte();
            }
            if has_type && type_end == 0 && child.kind() == "=" {
                type_end = child.start_byte();
            }
            if child.kind() == "=" {
                // Next sibling is the value.
                if let Some(val) = node.child(i + 1) {
                    has_value = true;
                    value_is_literal = matches!(
                        val.kind(),
                        "integer_literal"
                            | "float_literal"
                            | "string_literal"
                            | "boolean_literal"
                            | "char_literal"
                    );
                }
            }
        }
    }

    if has_type && has_value && value_is_literal && type_start < type_end {
        Some(Replacement {
            start: type_start,
            end: type_end,
            replacement: " ".to_string(),
        })
    } else {
        None
    }
}

/// R6: Collapse consecutive import statements into a summary line.
/// Only fires when 4+ consecutive import lines are found.
fn try_collapse_imports(
    node: tree_sitter::Node,
    source: &[u8],
    language: &str,
    kind: &str,
) -> Option<Replacement> {
    // Count consecutive siblings of the same import kind.
    let mut count = 1usize;
    let mut last_end = node.end_byte();
    let mut next = node.next_sibling();
    while let Some(sib) = next {
        if sib.kind() == kind {
            count += 1;
            last_end = sib.end_byte();
            next = sib.next_sibling();
        } else {
            break;
        }
    }

    // Also check that we are the FIRST in the run (avoid double-collapsing).
    if let Some(prev) = node.prev_sibling() {
        if prev.kind() == kind {
            // We are not the first — skip (the first one already handles the run).
            return None;
        }
    }

    let min_collapse = 4usize;
    if count < min_collapse {
        return None;
    }

    let import_keyword = match language {
        "rust" => "use",
        "python" => "import",
        "go" => "import",
        "typescript" | "java" => "import",
        "c_sharp" => "using",
        _ => "import",
    };

    let line_start = find_line_start(source, node.start_byte());
    let line_end = find_line_end(source, last_end);

    let indent = &source[line_start..node.start_byte()];
    let indent_str = std::str::from_utf8(indent).unwrap_or("");

    let summary = format!("{indent_str}// [{count} {import_keyword} statements]\n");

    Some(Replacement {
        start: line_start,
        end: line_end,
        replacement: summary,
    })
}

// ---------------------------------------------------------------------------
// Line-level rules (R8, R10)
// ---------------------------------------------------------------------------

fn apply_line_rules(source: &str) -> (String, Vec<RuleId>) {
    let mut rules = Vec::new();
    let mut result = String::with_capacity(source.len());
    let mut consecutive_empty = 0usize;

    for line in source.lines() {
        let trimmed = line.trim_end();

        // R8: Remove excessive empty lines.
        if trimmed.is_empty() {
            consecutive_empty += 1;
            if consecutive_empty > MAX_CONSECUTIVE_EMPTY_LINES {
                rules.push(RuleId::RemoveEmptyLines);
                continue;
            }
            result.push('\n');
            continue;
        }
        consecutive_empty = 0;

        // R10: Trim trailing whitespace.
        if trimmed.len() < line.len() {
            rules.push(RuleId::TrimTrailingWhitespace);
        }
        result.push_str(trimmed);
        result.push('\n');
    }

    // Remove trailing newline if original didn't have one.
    if !source.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    (result, rules)
}

// ---------------------------------------------------------------------------
// Byte-level utilities
// ---------------------------------------------------------------------------

/// Convert tree-sitter `child_count()` (usize) to u32 for `child()` API.
/// Safe: tree-sitter node child count cannot exceed u32::MAX.
fn child_count_u32(node: tree_sitter::Node) -> u32 {
    // tree-sitter child_count returns usize, child() takes u32.
    // Node child count is bounded by file size; u32 overflow is impossible.
    let count = node.child_count();
    debug_assert!(count <= u32::MAX as usize);
    count as u32
}

fn find_line_start(source: &[u8], pos: usize) -> usize {
    let mut i = pos;
    while i > 0 && source[i - 1] != b'\n' {
        i -= 1;
    }
    i
}

fn find_line_end(source: &[u8], pos: usize) -> usize {
    let mut i = pos;
    while i < source.len() && source[i] != b'\n' {
        i += 1;
    }
    // Include the newline character if present.
    if i < source.len() {
        i + 1
    } else {
        i
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sc() -> ShortCoder {
        ShortCoder::new()
    }

    #[test]
    fn collapse_rust_function_body() {
        let source = "fn compute(x: i32, y: i32) -> i32 {\n    let z = x + y;\n    z * 2\n}";
        let result = sc().simplify(source, "rust").unwrap();
        assert!(result.was_simplified());
        assert!(result.simplified.contains("fn compute(x: i32, y: i32) -> i32"));
        assert!(result.simplified.contains(BODY_PLACEHOLDER));
        assert!(!result.simplified.contains("let z"));
        assert!(result.rules_applied.contains(&RuleId::CollapseBodies));
    }

    #[test]
    fn collapse_python_function_body() {
        let source = "def hello(name):\n    greeting = f'Hello {name}'\n    print(greeting)\n    return greeting\n";
        let result = sc().simplify(source, "python").unwrap();
        assert!(result.was_simplified());
        assert!(result.simplified.contains("def hello(name):"));
        assert!(result.simplified.contains(PY_BODY_PLACEHOLDER));
        assert!(result.rules_applied.contains(&RuleId::CollapseBodies));
    }

    #[test]
    fn strip_line_comments_rust() {
        let source = "// This is a comment\nfn main() {}";
        let result = sc().simplify(source, "rust").unwrap();
        assert!(result.was_simplified());
        assert!(!result.simplified.contains("This is a comment"));
        assert!(result.rules_applied.contains(&RuleId::StripLineComments));
    }

    #[test]
    fn strip_python_docstring() {
        let source = "def foo():\n    \"\"\"This is a docstring.\"\"\"\n    return 1\n";
        let result = sc().simplify(source, "python").unwrap();
        assert!(result.was_simplified());
        assert!(!result.simplified.contains("docstring"));
        assert!(result.rules_applied.contains(&RuleId::StripDocstrings));
    }

    #[test]
    fn truncate_long_string() {
        let long_str = format!("\"{}\"", "a".repeat(100));
        let source = format!("let s = {};", long_str);
        let result = sc().simplify(&source, "rust").unwrap();
        assert!(result.was_simplified());
        assert!(result.simplified.len() < source.len());
        assert!(result.rules_applied.contains(&RuleId::TruncateStrings));
    }

    #[test]
    fn remove_excessive_empty_lines() {
        let source = "fn a() {}\n\n\n\n\nfn b() {}";
        let result = sc().simplify(source, "rust").unwrap();
        assert!(result.was_simplified());
        // Should have at most 1 consecutive empty line.
        assert!(!result.simplified.contains("\n\n\n"));
        assert!(result.rules_applied.contains(&RuleId::RemoveEmptyLines));
    }

    #[test]
    fn collapse_imports_rust() {
        let source = "use std::io;\nuse std::fs;\nuse std::path;\nuse std::env;\nuse std::fmt;\n\nfn main() {}";
        let result = sc().simplify(source, "rust").unwrap();
        assert!(result.was_simplified());
        assert!(result.simplified.contains("[5 use statements]"));
        assert!(result.rules_applied.contains(&RuleId::CollapseImports));
    }

    #[test]
    fn strip_python_decorator() {
        let source = "@staticmethod\ndef foo():\n    return 1\n";
        let result = sc().simplify(source, "python").unwrap();
        assert!(result.was_simplified());
        assert!(!result.simplified.contains("@staticmethod"));
        assert!(result.rules_applied.contains(&RuleId::StripDecorators));
    }

    #[test]
    fn unsupported_language_returns_error() {
        let result = sc().simplify("code", "haskell");
        assert!(matches!(result, Err(ShortCoderError::UnsupportedLanguage(_))));
    }

    #[test]
    fn empty_source_returns_noop() {
        let result = sc().simplify("", "rust").unwrap();
        assert!(!result.was_simplified());
        assert_eq!(result.compression_ratio(), 1.0);
    }

    #[test]
    fn trivial_function_not_collapsed() {
        // Single-line body should not be collapsed.
        let source = "fn id(x: i32) -> i32 { x }";
        let result = sc().simplify(source, "rust").unwrap();
        assert!(!result.rules_applied.contains(&RuleId::CollapseBodies));
    }

    #[test]
    fn compression_ratio_decreases() {
        let source = "\
// Header comment explaining the module
// Another comment line
use std::io;
use std::fs;
use std::path;
use std::env;

fn process(data: &str) -> Result<String, std::io::Error> {
    // Validate input
    let trimmed = data.trim();
    let result = trimmed.to_uppercase();
    Ok(result)
}
";
        let result = sc().simplify(source, "rust").unwrap();
        assert!(result.was_simplified());
        assert!(
            result.compression_ratio() < 0.85,
            "expected compression ratio < 0.85, got {}",
            result.compression_ratio()
        );
    }
}
