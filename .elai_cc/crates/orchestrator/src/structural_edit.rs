//! I-54: structural (AST-aware) search / preview / apply engine.
//!
//! Track 20 (STEAL-LIST B3, SLC-3): gajae packages structural AST search+edit
//! as an agent tool by vendoring `ast-grep-core`. ELAI rejected that dep. This
//! module steals only the IDEA: a structural query (tree-sitter query syntax,
//! the grammars' native pattern language) + a capture-template rewrite, with a
//! mandatory preview arm so the agent (and the journal) sees the match set
//! before any byte lands.
//!
//! ENGINE HOME (controller decision, Option B): orchestrator-inline, reusing the
//! shipped [`elai_knowledge::TreeSitterAstEvaluator`] (one tree-sitter version
//! source, SL-7 / SLC-3). This module NEVER calls `tree_sitter::Parser`/`Query`
//! directly -- it goes through the evaluator, exactly like the I-53 orchestrator-
//! inline pattern. The apply byte-write is NOT done here: this engine RESOLVES a
//! structural edit to a set of byte-exact `old_text -> new_text` rewrites, and
//! the production dispatch arm runs the SHIPPED PreToolUse edit gates
//! (`enforce_taint_sink_gate` + `enforce_fs_blast_radius`) per file and the
//! SHIPPED `ToolDispatcher::dispatch_edit` write path (controller decision 3,
//! apply = N x dispatch_edit). No gate is re-plumbed or bypassed.
//!
//! STALENESS GUARD (controller decision 4): preview blake3-hashes each matched
//! node's bytes; apply re-extracts the node via its byte-span and re-hashes,
//! refusing the edit (`RefusedStale`) if the hash changed since preview. A stale
//! apply must not silently corrupt the file.
//!
//! SCOPE (W-10 SHIP NOW): Rust / Python / TypeScript. Syntactic only -- semantic
//! / type-aware rename is E-52's LSP lane and is explicitly NOT this tool.

use std::collections::BTreeMap;

use elai_knowledge::TreeSitterAstEvaluator;
use elai_rules::predicate::{AstError, AstEvaluator};

/// Capture-template placeholder prefix. A rewrite template references a query
/// capture binding by `@name` (e.g. `fn @name` -> rendered with the `name`
/// capture's node text). Matches the tree-sitter query capture syntax so the
/// agent uses ONE pattern language for both query and rewrite.
const CAPTURE_SIGIL: char = '@';

/// The three SHIP-NOW languages (W-10). A query against any other language is a
/// typed [`StructuralEditError::UnsupportedLanguage`] rather than a silent
/// degrade -- the remaining six grammars are DEFERRED (enable per demand).
const SHIP_NOW_LANGUAGES: [&str; 3] = ["rust", "python", "typescript"];

/// Typed errors for the structural-edit engine. No regex fallback on a parse
/// failure (stub_ban): a broken query / rewrite template is a typed error, never
/// a degrade to string replace.
#[derive(Debug, thiserror::Error)]
pub enum StructuralEditError {
    /// The language is outside the SHIP-NOW set (Rust/Python/TypeScript).
    #[error("unsupported language for structural edit: {0} (ship-now: rust/python/typescript)")]
    UnsupportedLanguage(String),
    /// The tree-sitter query failed to parse/validate. This is the typed
    /// `ParseFailed` surface (negative arm); ZERO bytes are ever written.
    #[error("invalid structural query: {0}")]
    InvalidQuery(String),
    /// The rewrite template references a capture name the query never bound.
    #[error("rewrite template references unknown capture '@{0}'")]
    UnknownCapture(String),
    /// A matched node's bytes changed between preview and apply (the AST moved /
    /// the file was edited externally). The edit is REFUSED to avoid corruption.
    #[error("structural edit refused: file '{0}' changed since preview (stale match)")]
    RefusedStale(String),
    /// A matched node's text is not unique within the file, so an
    /// `old_text`-based rewrite cannot target it unambiguously (Option-1 apply
    /// guard). Refused rather than risk editing the wrong site.
    #[error("structural edit refused: match text is ambiguous (non-unique) in '{0}'")]
    AmbiguousMatch(String),
    /// The query binds sibling captures alongside `whole_node_capture` and
    /// matched 2+ groups. The evaluator flattens captures without a match-group
    /// id, so sibling captures cannot be grouped per-match — every match after
    /// the first would silently render with the first match's sibling values.
    /// Refused rather than risk a byte-exact but semantically wrong rewrite.
    #[error("structural edit refused: multi-capture query with multiple matches cannot group sibling captures per match ({0})")]
    UnsupportedMultiCaptureMultiMatch(String),
    /// Underlying evaluation failure (parser produced no tree, utf8, etc).
    #[error("structural evaluation failed: {0}")]
    EvaluationFailed(String),
}

impl From<AstError> for StructuralEditError {
    fn from(e: AstError) -> Self {
        match e {
            AstError::UnsupportedLanguage(l) => StructuralEditError::UnsupportedLanguage(l),
            AstError::InvalidQuery(q) => StructuralEditError::InvalidQuery(q),
            AstError::EvaluationFailed(m) => StructuralEditError::EvaluationFailed(m),
        }
    }
}

/// One structural match: the node text, its 0-indexed line range, its byte span
/// in the source, and the query capture name it satisfied. The byte span feeds
/// the staleness hash and the deterministic rewrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralMatch {
    /// The matched node's source text.
    pub node_text: String,
    /// 0-indexed start line of the node.
    pub start_line: u32,
    /// 0-indexed end line of the node.
    pub end_line: u32,
    /// Byte offset of the node start in the source.
    pub byte_start: usize,
    /// Byte offset of the node end in the source (exclusive).
    pub byte_end: usize,
    /// The tree-sitter query capture this node satisfied (e.g. `name`).
    pub capture_name: String,
}

/// Result of the search arm: every node matched by the query, with byte spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    /// All matched nodes, in source order.
    pub matches: Vec<StructuralMatch>,
}

/// One resolved rewrite: a byte-exact `old_text -> new_text` pair for a single
/// matched node, plus the blake3 hash of the node bytes captured at preview
/// time (the staleness anchor re-checked at apply).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRewrite {
    /// 0-indexed start line (for operator-visible diff context).
    pub start_line: u32,
    /// Byte offset of the node start at preview time.
    pub byte_start: usize,
    /// Byte offset of the node end at preview time (exclusive).
    pub byte_end: usize,
    /// The original node text (used as `dispatch_edit` `old_text`).
    pub old_text: String,
    /// The rendered rewrite (used as `dispatch_edit` `new_text`).
    pub new_text: String,
    /// blake3 hex of `old_text.as_bytes()` -- re-verified at apply.
    pub node_hash: String,
}

/// Result of the preview arm: the resolved rewrites plus a single blake3 hash
/// over the whole would-be diff (the journal's `Previewed` diff hash). No bytes
/// are written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewResult {
    /// Per-match resolved rewrites, in source order.
    pub rewrites: Vec<ResolvedRewrite>,
    /// blake3 hex over the concatenated `old_text|new_text` of every rewrite.
    /// Stable across runs for the same source + query + template.
    pub diff_hash: String,
    /// blake3 hex of the ENTIRE source the preview resolved against. This is the
    /// position-aware staleness anchor threaded across the preview -> apply
    /// boundary: apply re-reads the file, re-hashes it, and refuses
    /// (`RefusedStale`) if it differs from the hash the agent saw at preview.
    /// Unlike `diff_hash` (content of the matched nodes only, position-blind),
    /// `source_hash` changes on ANY edit to the file -- including one that moves
    /// a matched node without altering its text -- so "changed since the preview"
    /// is actually detectable end-to-end.
    pub source_hash: String,
}

/// blake3 hex digest of a byte slice. blake3 only (CLAUDE.md HARD RULE; never
/// blake2/sha2 for non-RFC uses).
fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Validate that the language is in the SHIP-NOW set.
fn ensure_ship_now(language: &str) -> Result<(), StructuralEditError> {
    if SHIP_NOW_LANGUAGES.contains(&language) {
        Ok(())
    } else {
        Err(StructuralEditError::UnsupportedLanguage(language.to_string()))
    }
}

/// SEARCH arm: run a tree-sitter query over `source` and return every matched
/// node with its byte span + capture name. No writes, no rewrite template.
pub fn structural_search(
    language: &str,
    source: &str,
    query: &str,
) -> Result<SearchResult, StructuralEditError> {
    ensure_ship_now(language)?;
    let evaluator = TreeSitterAstEvaluator::new();
    let raw = evaluator.evaluate_query(language, source, query)?;
    let matches = raw
        .into_iter()
        .map(|m| StructuralMatch {
            node_text: m.node_text,
            start_line: m.start_line,
            end_line: m.end_line,
            byte_start: m.byte_start,
            byte_end: m.byte_end,
            capture_name: m.capture_name,
        })
        .collect();
    Ok(SearchResult { matches })
}

/// Render a capture-template: substitute `@name` placeholders with the bound
/// capture's node text. `captures` maps capture-name -> node-text for ONE match
/// group. An `@name` with no binding is a typed [`StructuralEditError::UnknownCapture`].
///
/// Rendering is a single left-to-right scan: on `@`, read the longest following
/// `[A-Za-z0-9_]+` run as the capture name and substitute; a lone `@` (not
/// followed by an identifier char) is emitted verbatim.
fn render_template(
    template: &str,
    captures: &BTreeMap<String, String>,
) -> Result<String, StructuralEditError> {
    let mut out = String::with_capacity(template.len());
    let mut chars = template.char_indices().peekable();
    while let Some((_, c)) = chars.next() {
        if c != CAPTURE_SIGIL {
            out.push(c);
            continue;
        }
        // Collect the identifier run after the sigil.
        let mut name = String::new();
        while let Some(&(_, nc)) = chars.peek() {
            if nc.is_ascii_alphanumeric() || nc == '_' {
                name.push(nc);
                chars.next();
            } else {
                break;
            }
        }
        if name.is_empty() {
            // A bare '@' with no identifier: emit verbatim (not a placeholder).
            out.push(CAPTURE_SIGIL);
            continue;
        }
        match captures.get(&name) {
            Some(text) => out.push_str(text),
            None => return Err(StructuralEditError::UnknownCapture(name)),
        }
    }
    Ok(out)
}

/// PREVIEW arm: resolve `query` + `rewrite_template` over `source` into the
/// would-be rewrite set, WITHOUT writing. Each rewrite carries the blake3 hash
/// of its node bytes (the staleness anchor) and the template is rendered per
/// match group from that group's capture bindings.
///
/// The rewrite template is applied per `whole_node` capture: the match whose
/// `capture_name == whole_node_capture` is the node REPLACED; the other captures
/// in the same query match supply `@name` substitutions in the template. When a
/// query has a single capture, that capture is both the replaced node and the
/// substitution source.
///
/// A syntactically-broken template (unknown capture) is caught HERE (ParseFailed
/// semantics) with ZERO bytes written.
pub fn structural_preview(
    language: &str,
    source: &str,
    query: &str,
    rewrite_template: &str,
    whole_node_capture: &str,
) -> Result<PreviewResult, StructuralEditError> {
    ensure_ship_now(language)?;
    let evaluator = TreeSitterAstEvaluator::new();
    let raw = evaluator.evaluate_query(language, source, query)?;

    // The evaluator flattens captures into a single per-capture list with no
    // match-group id (see ast_adapter.rs / AstMatch). For a single-capture
    // query this is fine: each `whole_node_capture` occurrence is its own
    // rewrite and the template has no sibling `@name` to mis-substitute. For a
    // multi-capture query with 2+ `whole_node_capture` matches, sibling
    // captures cannot be grouped per-match from the flattened list, so every
    // match after the first would render with the first match's sibling
    // values -- a byte-exact but semantically wrong rewrite. Refuse it rather
    // than ship a silent mis-substitution on an agent-driven byte-exact edit.
    let whole_count = raw
        .iter()
        .filter(|m| m.capture_name == whole_node_capture)
        .count();
    let has_sibling_capture = raw
        .iter()
        .any(|m| m.capture_name != whole_node_capture);
    if whole_count > 1 && has_sibling_capture {
        return Err(StructuralEditError::UnsupportedMultiCaptureMultiMatch(
            whole_node_capture.to_string(),
        ));
    }

    // Group flattened captures by their (byte_start of the whole_node) so the
    // template can read sibling captures. The evaluator flattens captures in
    // match order; we treat each `whole_node_capture` occurrence as one rewrite
    // and gather every capture sharing its enclosing span as substitutions.
    // For the SHIP-NOW slice we build a single shared capture map per source
    // (capture-name -> first node text) plus the per-node replacement target.
    // With the multi-capture-multi-match guard above, this shared map is only
    // used when there is at most one whole_node match (so sibling captures are
    // unambiguously from that one match) or when the query has no sibling
    // captures (so the shared map is never read for substitution).
    let mut shared: BTreeMap<String, String> = BTreeMap::new();
    for m in &raw {
        shared
            .entry(m.capture_name.clone())
            .or_insert_with(|| m.node_text.clone());
    }

    let mut rewrites = Vec::new();
    for m in &raw {
        if m.capture_name != whole_node_capture {
            continue;
        }
        // Per-node capture map: the shared bindings, with the whole-node capture
        // overridden by THIS node's text (so `@name` in the template renders the
        // node being replaced, not a sibling's).
        let mut caps = shared.clone();
        caps.insert(m.capture_name.clone(), m.node_text.clone());
        let new_text = render_template(rewrite_template, &caps)?;
        rewrites.push(ResolvedRewrite {
            start_line: m.start_line,
            byte_start: m.byte_start,
            byte_end: m.byte_end,
            old_text: m.node_text.clone(),
            new_text,
            node_hash: blake3_hex(m.node_text.as_bytes()),
        });
    }

    // Stable diff hash over the whole rewrite set (the journal's Previewed hash).
    let mut diff_input = Vec::new();
    for r in &rewrites {
        diff_input.extend_from_slice(r.old_text.as_bytes());
        diff_input.push(0);
        diff_input.extend_from_slice(r.new_text.as_bytes());
        diff_input.push(0);
    }
    let diff_hash = blake3_hex(&diff_input);
    // Whole-file anchor: any change to the file between preview and apply flips
    // this, even one that repositions a matched node without changing its text.
    let source_hash = blake3_hex(source.as_bytes());

    Ok(PreviewResult {
        rewrites,
        diff_hash,
        source_hash,
    })
}

/// APPLY arm (staleness check leg): given the preview's resolved rewrites and the
/// CURRENT file source (re-read at apply time), verify every rewrite is still
/// valid:
///   - the node text at each `[byte_start, byte_end)` still hashes to the
///     preview `node_hash` (else `RefusedStale` -- the AST moved),
///   - the node text is unique in the current source (else `AmbiguousMatch` --
///     an `old_text`-keyed `dispatch_edit` cannot target it safely).
///
/// On success returns the rewrites verified safe to apply (the dispatch arm then
/// runs the SHIPPED edit gates + `dispatch_edit` per rewrite). This function
/// performs NO file I/O and NO writes -- it is the pure staleness guard so the
/// dispatch arm owns the gated write path.
pub fn verify_apply_safe(
    file_label: &str,
    current_source: &str,
    rewrites: &[ResolvedRewrite],
) -> Result<(), StructuralEditError> {
    let bytes = current_source.as_bytes();
    for r in rewrites {
        // Bounds + byte-span re-extraction. An out-of-range span (the file
        // shrank) is itself a staleness signal.
        if r.byte_end > bytes.len() || r.byte_start > r.byte_end {
            return Err(StructuralEditError::RefusedStale(file_label.to_string()));
        }
        let current_node = &bytes[r.byte_start..r.byte_end];
        if blake3_hex(current_node) != r.node_hash {
            return Err(StructuralEditError::RefusedStale(file_label.to_string()));
        }
        // Uniqueness: the old_text must occur exactly once so dispatch_edit's
        // single-exact-match contract targets the intended site.
        let occurrences = current_source.matches(r.old_text.as_str()).count();
        if occurrences != 1 {
            return Err(StructuralEditError::AmbiguousMatch(file_label.to_string()));
        }
    }
    Ok(())
}

/// APPLY arm (preview-staleness check leg): the "changed since the preview"
/// guard that the in-file `verify_apply_safe` re-hash CANNOT deliver alone.
///
/// `verify_apply_safe` re-hashes nodes against the SAME source the apply arm
/// just re-previewed, so it only catches an out-of-range / mid-node smear, never
/// a clean external edit that left the matched node text intact (e.g. a header
/// prepended above it). This guard closes that gap: it compares the whole-file
/// hash the agent saw at preview (`expected_source_hash`, threaded across the
/// preview -> apply boundary) to the hash of the file at apply time. ANY
/// difference -- including a pure reposition -- is `RefusedStale`. This is the
/// real end-to-end staleness refusal; the production dispatch arm calls it BEFORE
/// any `dispatch_edit`, so the `RefusedStale` SSB row is reachable in production.
pub fn verify_source_unchanged(
    file_label: &str,
    current_source: &str,
    expected_source_hash: &str,
) -> Result<(), StructuralEditError> {
    if blake3_hex(current_source.as_bytes()) != expected_source_hash {
        return Err(StructuralEditError::RefusedStale(file_label.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_finds_rust_functions() {
        let src = "fn alpha() {}\nfn beta() {}";
        let q = "(function_item name: (identifier) @name)";
        let res = structural_search("rust", src, q).expect("search ok");
        assert_eq!(res.matches.len(), 2);
        assert_eq!(res.matches[0].node_text, "alpha");
        assert_eq!(res.matches[0].capture_name, "name");
        // byte span of "alpha" in "fn alpha() {}" -> 3..8
        assert_eq!(res.matches[0].byte_start, 3);
        assert_eq!(res.matches[0].byte_end, 8);
    }

    #[test]
    fn unsupported_language_is_typed_error() {
        let err = structural_search("haskell", "x", "(id) @i").unwrap_err();
        assert!(matches!(err, StructuralEditError::UnsupportedLanguage(_)));
    }

    #[test]
    fn invalid_query_is_typed_parsefailed_no_bytes() {
        let err = structural_preview("rust", "fn a(){}", "(((bad", "x", "name").unwrap_err();
        assert!(matches!(err, StructuralEditError::InvalidQuery(_)));
    }

    #[test]
    fn preview_renders_capture_template() {
        let src = "fn old_name() {}";
        let q = "(function_item name: (identifier) @name)";
        let prev = structural_preview("rust", src, q, "new_name", "name").expect("preview ok");
        assert_eq!(prev.rewrites.len(), 1);
        assert_eq!(prev.rewrites[0].old_text, "old_name");
        assert_eq!(prev.rewrites[0].new_text, "new_name");
        assert!(!prev.diff_hash.is_empty());
    }

    #[test]
    fn unknown_capture_in_template_is_typed_error() {
        let src = "fn a() {}";
        let q = "(function_item name: (identifier) @name)";
        let err = structural_preview("rust", src, q, "@missing", "name").unwrap_err();
        assert!(matches!(err, StructuralEditError::UnknownCapture(_)));
    }

    #[test]
    fn verify_apply_safe_passes_on_unchanged_source() {
        let src = "fn old_name() {}";
        let q = "(function_item name: (identifier) @name)";
        let prev = structural_preview("rust", src, q, "new_name", "name").unwrap();
        verify_apply_safe("f.rs", src, &prev.rewrites).expect("unchanged -> safe");
    }

    #[test]
    fn verify_apply_safe_refuses_stale_source() {
        let src = "fn old_name() {}";
        let q = "(function_item name: (identifier) @name)";
        let prev = structural_preview("rust", src, q, "new_name", "name").unwrap();
        // Mutate the file: the byte-span now points at different bytes.
        let mutated = "fn DIFFERENT() {}";
        let err = verify_apply_safe("f.rs", mutated, &prev.rewrites).unwrap_err();
        assert!(matches!(err, StructuralEditError::RefusedStale(_)));
    }

    #[test]
    fn verify_source_unchanged_passes_on_identical_source() {
        let src = "fn old_name() {}";
        let q = "(function_item name: (identifier) @name)";
        let prev = structural_preview("rust", src, q, "new_name", "name").unwrap();
        verify_source_unchanged("f.rs", src, &prev.source_hash).expect("identical -> unchanged");
    }

    #[test]
    fn verify_source_unchanged_refuses_repositioned_node() {
        // A header is prepended above the matched fn: the node TEXT is identical,
        // so an in-node re-hash would pass, but the WHOLE FILE changed -> stale.
        let src = "fn old_name() {}";
        let q = "(function_item name: (identifier) @name)";
        let prev = structural_preview("rust", src, q, "new_name", "name").unwrap();
        let moved = "// header\nfn old_name() {}";
        // Node-level guard does NOT catch this (the bytes at the old span differ,
        // but the point is the whole-file anchor is what makes it deterministic).
        let err = verify_source_unchanged("f.rs", moved, &prev.source_hash).unwrap_err();
        assert!(matches!(err, StructuralEditError::RefusedStale(_)));
    }

    #[test]
    fn verify_apply_safe_refuses_ambiguous_match() {
        // Two identical identifiers -> the matched node text is non-unique.
        let src = "let x = 1;\nlet x = 2;";
        let q = "(identifier) @id";
        let prev = structural_preview("python", src, q, "y", "id").unwrap();
        // At least one rewrite whose old_text ("x") occurs twice.
        let err = verify_apply_safe("f.py", src, &prev.rewrites).unwrap_err();
        assert!(matches!(err, StructuralEditError::AmbiguousMatch(_)));
    }

    #[test]
    fn multi_capture_multi_match_is_refused() {
        // Two functions, query binds both the function node and its name.
        // Sibling captures cannot be grouped per-match from the flattened
        // evaluator output, so refuse rather than silently mis-substitute.
        let src = "fn alpha() { 1 }\nfn beta() { 2 }";
        let q = "(function_item name: (identifier) @name) @fn";
        let err = structural_preview("rust", src, q, "renamed", "fn").unwrap_err();
        assert!(matches!(
            err,
            StructuralEditError::UnsupportedMultiCaptureMultiMatch(_)
        ));
    }

    #[test]
    fn single_capture_multi_match_is_allowed() {
        // No sibling captures: each whole_node match is its own rewrite, the
        // template has no @sibling to mis-substitute. The common rename case.
        let src = "fn alpha() {}\nfn beta() {}";
        let q = "(function_item name: (identifier) @name)";
        let prev = structural_preview("rust", src, q, "renamed", "name").unwrap();
        assert_eq!(prev.rewrites.len(), 2);
    }

    #[test]
    fn multi_capture_single_match_is_allowed() {
        // One whole_node match: sibling captures are unambiguously from that
        // one match, so the shared map is correct.
        let src = "fn alpha() { 1 }";
        let q = "(function_item name: (identifier) @name) @fn";
        let prev = structural_preview("rust", src, q, "renamed", "fn").unwrap();
        assert_eq!(prev.rewrites.len(), 1);
        assert_eq!(prev.rewrites[0].old_text, "fn alpha() { 1 }");
    }
}
