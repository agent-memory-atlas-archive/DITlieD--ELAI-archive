//! SpeculativeFileRouter — library-tier AST-based undefined-symbol detector
//! that decides whether a baby step can `Proceed` against `target_file` or
//! whether a `MicroStep` (create_file via ReplanRequest::Minor) is required
//! before the original step is retried. B-28 E-7.
//!
//! Spec: `workflow/papers/focus-tunnel-design-interface.md`
//!   * L48-52: SpeculativeFileRouter design contract — `route(step) -> { UseTarget(path) | NeedsMicroStep(micro) }`.
//!   * L53-58: MicroStep wire form — `{ action: "create_file", path, required_type, mode: STRUCTURED }`.
//!   * L118: cap of 2 micro-steps per baby step (prevent infinite loops).
//!   * L441-490: speculative routing pipeline — orchestrator reads target_file,
//!     runs AST analysis on model output, emits ReplanRequest::Minor on mismatch.
//!
//! Library-tier shape (locked decision §16 from
//! `workflow/sessions/s-27436/context-package-E7.md`):
//!   * Synchronous — no async, no tokio, no I/O.
//!   * Caller supplies `target_file_content` + `model_output` as `&str`.
//!   * No SSB writes; no FSM transitions; no fs reads.
//!   * Per-router counter; lifetime is "one baby step" (caller constructs a
//!     fresh router for each baby step to reset).
//!
//! Wiring (DEFERRED, NOT in this submodule):
//!   * `ReplanRequest::Minor` emission via `Fsm::submit_replan` — landed in the
//!     same WIRING phase that owes E-5b SSB rows (`task_runner.rs:4638-4640`
//!     carries the existing TODO).
//!   * SSB `micro_step_created` action_type — bundled in the BATCHED SSB
//!     schema bump that joins E-5b `pipeline_pre_assembled` + E-6
//!     `dependency_resolved` + E-7 `micro_step_created`. ssb.rs is LOCKED
//!     by session s-28564; this submodule does NOT edit it.
//!
//! LIMITATION (false-negative bias, by design):
//!   The AST symbol-detection heuristic is intentionally conservative: it
//!   only flags identifiers that (a) are not Rust/Python keywords, (b) are
//!   not primitive type names, (c) do not appear in any definition-kind node
//!   of the target_file AST. False negatives (we silently `Proceed` when a
//!   micro-step would have helped) are PREFERRED over false positives
//!   (spurious micro-steps that burn the cap-of-2). Cases we miss include:
//!   macro-defined symbols, trait-method calls that require the trait
//!   import, glob imports, type aliases, and any cross-module symbol resolution.
//!   The post-write deterministic auditor catches anything we miss.
//!
//! `route()` takes `&mut self` because tree-sitter parsers are stateful and
//! cached lazily on the router (mirrors `streaming.rs:576` pattern). The
//! WIRING phase must serialise router access per baby step — single-threaded
//! by construction today.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tree_sitter::{Node, Parser};

use crate::ssb::SsbPayload;
use elai_plugins::import_extractor::ImportLanguage;

use super::EditRegion;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Inferred kind of artifact the undefined symbol most likely is. Derived from
/// the identifier's casing per locked decision §15 (path heuristic + type-hint
/// classifier). The WIRING phase serialises this into the
/// `required_type` field of the MicroStep wire form (spec L53-58).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicroStepType {
    /// Bare lowercase identifier with multi-segment path (`foo::bar`,
    /// `foo.bar`) — most likely a module / file.
    Module,
    /// First-char uppercase identifier (`Widget`, `Foo`) — most likely a
    /// struct / class.
    Struct,
    /// snake_case identifier in call position — free function.
    Function,
    /// SCREAMING_SNAKE_CASE — const / static.
    Const,
}

/// The micro-step the router proposes when an undefined symbol is detected.
/// Spec L53-58 wire form: `{ action: "create_file", path, required_type,
/// mode: STRUCTURED }`. The WIRING phase serialises this into the wire shape
/// for the ReplanRequest::Minor evidence; this struct is the library-tier
/// Rust value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicroStep {
    /// Heuristic suggested path for the new file. Computed from
    /// `target_file.parent()` + snake_case(name) + extension. See module
    /// header LIMITATION block for the edge cases this misses.
    pub path: PathBuf,
    /// Inferred kind. Determines `required_type` in the wire form.
    pub type_hint: MicroStepType,
    /// Human-readable reason for the ReplanRequest::Minor evidence field.
    /// Format: `"undefined symbol `{name}` referenced but not defined in
    /// `{target_file}`"`.
    pub reason: String,
}

/// Outcome of one `SpeculativeFileRouter::route()` call.
///
/// `ParseSkipped` is the false-negative-bias return value: when the language
/// is unsupported, or either the target file content or the model output
/// fails to parse, the router emits `ParseSkipped` rather than spuriously
/// emitting a `NeedsMicroStep`. The orchestrator treats `ParseSkipped` as
/// equivalent to `Proceed` for the purpose of writing the output — the
/// post-write deterministic auditor catches whatever we missed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteDecision {
    /// All identifiers referenced by `model_output` are defined in the
    /// reachable AST of `target_file`. Orchestrator writes the output to
    /// `written_path` at `edit_region`.
    Proceed { written_path: PathBuf },
    /// At least one symbol referenced by `model_output` is not defined in
    /// the reachable AST of `target_file` AND the cap is not yet exhausted.
    /// Orchestrator emits a `ReplanRequest::Minor` (DEFERRED to WIRING phase)
    /// and queues a STRUCTURED create_file step.
    NeedsMicroStep(MicroStep),
    /// Cap of `max_micro_steps` reached but `model_output` still contains
    /// an undefined symbol. Orchestrator fails the baby step with a typed
    /// Finding (do NOT keep emitting Minor replans). Spec L118.
    MicroStepCapExhausted {
        count: usize,
        last_undefined: String,
    },
    /// Language unsupported, or parse failure on either side. False-negative
    /// bias: prefer this over a spurious `NeedsMicroStep`. `reason` is a
    /// short diagnostic string for tracing / SSB.
    ParseSkipped { reason: String },
}

/// Speculative file-creation router. Owns its own micro-step counter (per
/// locked decision §11, Option A) and lazy tree-sitter parsers. Caller
/// constructs ONE router per baby step; counter resets implicitly when the
/// router is dropped.
pub struct SpeculativeFileRouter {
    max_micro_steps: usize,
    current_micro_steps: usize,
    parser_rs: Option<Parser>,
    parser_py: Option<Parser>,
}

impl SpeculativeFileRouter {
    /// Construct a router with the given cap. `max_micro_steps = 2` matches
    /// spec L118. Tests pass 0 for "cap immediately", 1 for boundary, 2 for
    /// production.
    #[must_use]
    pub fn new(max_micro_steps: usize) -> Self {
        Self {
            max_micro_steps,
            current_micro_steps: 0,
            parser_rs: None,
            parser_py: None,
        }
    }

    /// Number of micro-steps consumed so far on this router instance.
    #[must_use]
    pub fn micro_steps_consumed(&self) -> usize {
        self.current_micro_steps
    }

    /// Remaining micro-step budget (saturating subtract — cannot underflow).
    #[must_use]
    pub fn micro_steps_remaining(&self) -> usize {
        self.max_micro_steps.saturating_sub(self.current_micro_steps)
    }

    /// Called by the orchestrator AFTER a NeedsMicroStep micro-step completes
    /// successfully (file created). Increments the counter; does NOT reset.
    /// Subsequent `route()` calls may still emit `NeedsMicroStep` (if there's
    /// budget) or `MicroStepCapExhausted` (if there isn't).
    pub fn record_micro_step_completion(&mut self) {
        self.current_micro_steps = self.current_micro_steps.saturating_add(1);
    }

    /// Pure sync decision: should the orchestrator proceed against
    /// `target_file`, or does `model_output` reference a symbol that requires
    /// a new file?
    ///
    /// `edit_region` is currently observational (the FIM/DIFF region the
    /// caller will splice into); it is preserved on the signature so future
    /// versions can constrain "defined here" to symbols whose definition
    /// node ends BEFORE `edit_region.start_line`. E-7 ignores it.
    pub fn route(
        &mut self,
        target_file: &Path,
        target_file_content: &str,
        _edit_region: Option<EditRegion>,
        model_output: &str,
        language: ImportLanguage,
    ) -> RouteDecision {
        // (1) Language gate — only Rust + Python are supported in E-7
        //     (tree_sitter_rust + tree_sitter_python are the only grammars
        //     pinned in this crate). Anything else → ParseSkipped.
        let lang = match language {
            ImportLanguage::Rust => InternalLang::Rust,
            ImportLanguage::Python => InternalLang::Python,
            other => {
                tracing::debug!(
                    target: "focus_tunnel::file_router",
                    "ParseSkipped: unsupported language {:?}",
                    other,
                );
                return RouteDecision::ParseSkipped {
                    reason: format!("unsupported language: {other:?}"),
                };
            }
        };

        // (2) Parse target_file_content → defined-set. Parse failure → bail.
        let defined = match self.collect_defined(target_file_content, lang) {
            Ok(set) => set,
            Err(reason) => {
                tracing::debug!(
                    target: "focus_tunnel::file_router",
                    "ParseSkipped: target_file parse error — {}",
                    reason,
                );
                return RouteDecision::ParseSkipped {
                    reason: format!("target_file parse error: {reason}"),
                };
            }
        };

        // (3) Parse model_output → referenced-set. Parse failure → bail.
        let referenced = match self.collect_referenced(model_output, lang) {
            Ok(list) => list,
            Err(reason) => {
                tracing::debug!(
                    target: "focus_tunnel::file_router",
                    "ParseSkipped: model_output parse error — {}",
                    reason,
                );
                return RouteDecision::ParseSkipped {
                    reason: format!("model_output parse error: {reason}"),
                };
            }
        };

        // (4) Filter referenced through keyword/primitive deny-lists and the
        //     defined-set. ONLY the first undefined symbol drives the
        //     decision — the orchestrator handles subsequent undefineds
        //     across baby-step retries.
        let undefined = first_undefined(&referenced, &defined, lang);

        let Some(undef_name) = undefined else {
            return RouteDecision::Proceed {
                written_path: target_file.to_path_buf(),
            };
        };

        // (5) Cap enforcement. If we've already consumed the budget, emit
        //     MicroStepCapExhausted and let the orchestrator fail the step.
        if self.current_micro_steps >= self.max_micro_steps {
            return RouteDecision::MicroStepCapExhausted {
                count: self.current_micro_steps,
                last_undefined: undef_name,
            };
        }

        // (6) Path heuristic + type-hint classifier (locked decision §15).
        let type_hint = classify_identifier(&undef_name);
        let suggested_path = suggest_path(target_file, &undef_name, lang);

        // (7) Defensive bail: if our heuristic suggested the very file we
        //     are editing, do NOT emit a micro-step (we cannot create the
        //     file we are editing). False-negative-bias: ParseSkipped.
        if suggested_path == target_file {
            return RouteDecision::ParseSkipped {
                reason: format!(
                    "self-referential collision: suggested path equals target_file ({})",
                    target_file.display(),
                ),
            };
        }

        let reason = format!(
            "undefined symbol `{}` referenced but not defined in `{}`",
            undef_name,
            target_file.display(),
        );

        RouteDecision::NeedsMicroStep(MicroStep {
            path: suggested_path,
            type_hint,
            reason,
        })
    }

    // --------- AST walks -----------------------------------------------------

    /// Parse the target file source and collect every identifier in a
    /// definition-kind node. The set is intentionally over-inclusive: extra
    /// names → false negatives → fewer spurious micro-steps (matches the
    /// LIMITATION block in the module header).
    fn collect_defined(
        &mut self,
        source: &str,
        lang: InternalLang,
    ) -> Result<BTreeSet<String>, String> {
        let parser = self
            .parser_for(lang)
            .ok_or_else(|| "tree-sitter grammar attach failed".to_string())?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "tree-sitter returned no tree".to_string())?;
        if tree.root_node().has_error() {
            return Err("tree-sitter root has ERROR node".to_string());
        }
        let mut set: BTreeSet<String> = BTreeSet::new();
        walk_collect_defined(&tree.root_node(), source.as_bytes(), lang, &mut set);
        Ok(set)
    }

    /// Parse the model output and collect every identifier in a reference-
    /// kind node (call_expression, type_identifier, etc.). Result preserves
    /// insertion order so `first_undefined` is deterministic.
    fn collect_referenced(
        &mut self,
        source: &str,
        lang: InternalLang,
    ) -> Result<Vec<String>, String> {
        let parser = self
            .parser_for(lang)
            .ok_or_else(|| "tree-sitter grammar attach failed".to_string())?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "tree-sitter returned no tree".to_string())?;
        if tree.root_node().has_error() {
            return Err("tree-sitter root has ERROR node".to_string());
        }
        let mut out: Vec<String> = Vec::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        walk_collect_referenced(&tree.root_node(), source.as_bytes(), lang, &mut out, &mut seen);
        Ok(out)
    }

    /// Lazy parser cache, mirrors `focus_tunnel/streaming.rs:820-829`. Builds
    /// the parser exactly once per language per router instance. Returns
    /// `None` if the grammar fails to attach (so callers can bail via
    /// ParseSkipped instead of panicking — CLAUDE.md hard rule: no `.expect()`
    /// in production paths). The grammar IS statically valid for the two
    /// languages we pin, but the pattern mirrors `streaming.rs:820-829` which
    /// also returns Option to avoid `.expect()`.
    fn parser_for(&mut self, lang: InternalLang) -> Option<&mut Parser> {
        match lang {
            InternalLang::Rust => {
                if self.parser_rs.is_none() {
                    let mut p = Parser::new();
                    p.set_language(&tree_sitter_rust::LANGUAGE.into()).ok()?;
                    self.parser_rs = Some(p);
                }
                self.parser_rs.as_mut()
            }
            InternalLang::Python => {
                if self.parser_py.is_none() {
                    let mut p = Parser::new();
                    p.set_language(&tree_sitter_python::LANGUAGE.into()).ok()?;
                    self.parser_py = Some(p);
                }
                self.parser_py.as_mut()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AST walkers — definition-kind + reference-kind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InternalLang {
    Rust,
    Python,
}

/// Definition-kind nodes — `function_item`, `struct_item`, etc. (Rust) and
/// `function_definition`, `class_definition`, etc. (Python). Collects the
/// `name` field of each into `set`. Cursor walk pattern mirrors
/// `deterministic_auditor.rs:818-852`.
fn walk_collect_defined(
    node: &Node<'_>,
    source: &[u8],
    lang: InternalLang,
    set: &mut BTreeSet<String>,
) {
    let kind = node.kind();
    let interesting = match lang {
        InternalLang::Rust => matches!(
            kind,
            "function_item"
                | "struct_item"
                | "enum_item"
                | "trait_item"
                | "impl_item"
                | "const_item"
                | "static_item"
                | "mod_item"
                | "type_item"
                | "union_item"
                | "macro_definition"
                | "use_declaration"
        ),
        InternalLang::Python => matches!(
            kind,
            "function_definition" | "class_definition" | "import_statement" | "import_from_statement"
        ),
    };
    if interesting {
        if let Some(name) = node.child_by_field_name("name") {
            if let Ok(text) = std::str::from_utf8(&source[name.byte_range()]) {
                set.insert(text.to_string());
            }
        }
        // Rust `use foo::Bar;` / `use foo::{Bar, Baz};` / `use foo::Bar as
        // Baz;` / `use foo::{Bar, baz::{Qux}};` — the use_declaration node
        // has no `name` field, so walk the `argument` child and collect every
        // local binding the use statement introduces. tree-sitter-rust grammar
        // (https://github.com/tree-sitter/tree-sitter-rust grammar.js):
        //   use_declaration -> argument: identifier | scoped_identifier
        //                                | use_list | scoped_use_list
        //                                | use_as_clause | use_wildcard
        // False-negative bias (module header L34-37): we deliberately over-
        // collect — an extra defined name is FINE; a missed name burns the
        // cap-of-2 budget.
        if matches!(lang, InternalLang::Rust) && kind == "use_declaration" {
            if let Some(argument) = node.child_by_field_name("argument") {
                collect_rust_use_bindings(&argument, source, set);
            } else {
                // Defensive fallback: walk every child (older tree-sitter-rust
                // versions may not expose the `argument` field name).
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    collect_rust_use_bindings(&child, source, set);
                }
            }
        }
        // Python `import x, y` / `from m import a, b` — name field is absent
        // on the outer node, so walk dotted_name / aliased_import children.
        if matches!(lang, InternalLang::Python)
            && matches!(kind, "import_statement" | "import_from_statement")
        {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_python_import_names(&child, source, set);
            }
        }
    }
    // Python: top-level / class-level assignments — `FOO = 1`, `bar = lambda: 0`.
    // LHS identifier(s) become defined names. Same rule applies inside
    // function bodies but for safety we walk every assignment we see (the
    // false-negative bias means extra names are FINE).
    if matches!(lang, InternalLang::Python) && kind == "assignment" {
        if let Some(lhs) = node.child_by_field_name("left") {
            collect_python_assignment_lhs(&lhs, source, set);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_collect_defined(&child, source, lang, set);
    }
}

fn collect_python_import_names(node: &Node<'_>, source: &[u8], set: &mut BTreeSet<String>) {
    match node.kind() {
        "dotted_name" => {
            // Insert the FIRST segment (top-level module binding) AND the
            // raw text (so `from foo import bar` defines `foo`+`bar`).
            if let Some(first) = node.child(0) {
                if let Ok(t) = std::str::from_utf8(&source[first.byte_range()]) {
                    set.insert(t.to_string());
                }
            }
            if let Ok(t) = std::str::from_utf8(&source[node.byte_range()]) {
                set.insert(t.to_string());
            }
        }
        "aliased_import" => {
            if let Some(alias) = node.child_by_field_name("alias") {
                if let Ok(t) = std::str::from_utf8(&source[alias.byte_range()]) {
                    set.insert(t.to_string());
                }
            }
        }
        "identifier" => {
            if let Ok(t) = std::str::from_utf8(&source[node.byte_range()]) {
                set.insert(t.to_string());
            }
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_python_import_names(&child, source, set);
            }
        }
    }
}

fn collect_python_assignment_lhs(node: &Node<'_>, source: &[u8], set: &mut BTreeSet<String>) {
    match node.kind() {
        "identifier" => {
            if let Ok(t) = std::str::from_utf8(&source[node.byte_range()]) {
                set.insert(t.to_string());
            }
        }
        // Tuple / pattern_list — recurse children.
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_python_assignment_lhs(&child, source, set);
            }
        }
    }
}

/// Collect every local binding introduced by a Rust `use_declaration`. The
/// caller passes the use_declaration's `argument` child (or, in the defensive
/// fallback, raw children of the use_declaration itself). tree-sitter-rust
/// node kinds we handle:
///   * `identifier`                    — bare `use foo;` binding `foo`.
///   * `scoped_identifier`             — `use foo::Bar;` binding `Bar`
///     (the `name` field, last segment).
///   * `use_as_clause`                 — `use foo::Bar as Baz;` binding
///     only `Baz` (the `alias` field). The path side is NOT a binding.
///   * `use_list` / `scoped_use_list`  — `use foo::{Bar, Baz}` /
///     `use foo::{Bar, baz::{Qux}}` — recurse into list members. For
///     `scoped_use_list`, the `path` field (e.g. `foo`) is NOT a binding;
///     only the leaf names in the list are.
///   * `use_wildcard`                  — `use foo::*;` introduces no
///     identifier we can statically name; skip (false-negative bias).
///
/// False-negative bias: when we encounter an unknown node kind, we recurse
/// into its children. Extra names are FINE (module header L34-37).
fn collect_rust_use_bindings(node: &Node<'_>, source: &[u8], set: &mut BTreeSet<String>) {
    match node.kind() {
        "identifier" => {
            if let Ok(t) = std::str::from_utf8(&source[node.byte_range()]) {
                set.insert(t.to_string());
            }
        }
        "scoped_identifier" => {
            // Last segment is the local binding. tree-sitter-rust exposes the
            // last segment as the `name` field.
            if let Some(name) = node.child_by_field_name("name") {
                if let Ok(t) = std::str::from_utf8(&source[name.byte_range()]) {
                    set.insert(t.to_string());
                }
            } else {
                // Defensive fallback: collect every identifier-typed child.
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        if let Ok(t) = std::str::from_utf8(&source[child.byte_range()]) {
                            set.insert(t.to_string());
                        }
                    }
                }
            }
        }
        "use_as_clause" => {
            // Only the alias is a local binding.
            if let Some(alias) = node.child_by_field_name("alias") {
                if let Ok(t) = std::str::from_utf8(&source[alias.byte_range()]) {
                    set.insert(t.to_string());
                }
            }
        }
        "use_list" => {
            // `use foo::{Bar, Baz}` — list of arguments. Recurse into each.
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_rust_use_bindings(&child, source, set);
            }
        }
        "scoped_use_list" => {
            // `use foo::{Bar, Baz}` (the path-with-list shape). The `path`
            // field is NOT a binding; only the `list` field's members are.
            if let Some(list) = node.child_by_field_name("list") {
                collect_rust_use_bindings(&list, source, set);
            } else {
                // Defensive fallback: recurse all children, skipping the
                // path component.
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "use_list" {
                        collect_rust_use_bindings(&child, source, set);
                    }
                }
            }
        }
        "use_wildcard" => {
            // `use foo::*;` — no namable local binding. Skip.
        }
        _ => {
            // Unknown / intermediate (e.g. punctuation, error nodes) — recurse.
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_rust_use_bindings(&child, source, set);
            }
        }
    }
}

/// Reference-kind walk — collects identifiers in call / type position. The
/// preservation of insertion order (Vec, not Set) lets `first_undefined`
/// return the FIRST referenced-but-not-defined symbol deterministically.
fn walk_collect_referenced(
    node: &Node<'_>,
    source: &[u8],
    lang: InternalLang,
    out: &mut Vec<String>,
    seen: &mut BTreeSet<String>,
) {
    let kind = node.kind();
    let mut take_self = false;
    let mut take_text: Option<String> = None;

    match lang {
        InternalLang::Rust => {
            if kind == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    if let Some(name) = leftmost_rust_call_name(&func, source) {
                        take_text = Some(name);
                    }
                }
            } else if kind == "type_identifier" {
                if let Ok(t) = std::str::from_utf8(&source[node.byte_range()]) {
                    take_text = Some(t.to_string());
                }
                take_self = true;
            } else if kind == "scoped_identifier" || kind == "scoped_type_identifier" {
                // First path segment — `foo::Bar` → `foo`.
                if let Some(first) = node.child(0) {
                    if let Ok(t) = std::str::from_utf8(&source[first.byte_range()]) {
                        take_text = Some(t.to_string());
                    }
                }
                take_self = true;
            }
        }
        InternalLang::Python => {
            if kind == "call" {
                if let Some(func) = node.child_by_field_name("function") {
                    if let Some(name) = leftmost_python_call_name(&func, source) {
                        take_text = Some(name);
                    }
                }
            }
        }
    }

    if let Some(text) = take_text {
        if seen.insert(text.clone()) {
            out.push(text);
        }
    }

    if !take_self {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk_collect_referenced(&child, source, lang, out, seen);
        }
    }
}

fn leftmost_rust_call_name(node: &Node<'_>, source: &[u8]) -> Option<String> {
    match node.kind() {
        "identifier" => std::str::from_utf8(&source[node.byte_range()]).ok().map(|t| t.to_string()),
        "scoped_identifier" => {
            // First segment: `foo::bar()` → `foo`.
            let first = node.child(0)?;
            std::str::from_utf8(&source[first.byte_range()]).ok().map(|t| t.to_string())
        }
        "field_expression" => {
            // `x.method()` — the symbol we care about is the receiver `x`.
            let value = node.child_by_field_name("value")?;
            leftmost_rust_call_name(&value, source)
        }
        _ => None,
    }
}

fn leftmost_python_call_name(node: &Node<'_>, source: &[u8]) -> Option<String> {
    match node.kind() {
        "identifier" => std::str::from_utf8(&source[node.byte_range()]).ok().map(|t| t.to_string()),
        "attribute" => {
            // `mod.fn()` — leftmost identifier is the import we care about.
            let object = node.child_by_field_name("object")?;
            leftmost_python_call_name(&object, source)
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Filter / classifier / path heuristic
// ---------------------------------------------------------------------------

/// Returns the FIRST referenced identifier that is neither a keyword, nor a
/// primitive type name, nor a member of `defined`. None → all references are
/// known, caller should Proceed.
fn first_undefined(
    referenced: &[String],
    defined: &BTreeSet<String>,
    lang: InternalLang,
) -> Option<String> {
    for name in referenced {
        if name.is_empty() {
            continue;
        }
        if is_keyword(name, lang) || is_primitive(name, lang) || is_self_marker(name) {
            continue;
        }
        if defined.contains(name) {
            continue;
        }
        // std prefix — `std::*`, `core::*`, `alloc::*` (Rust); `__builtins__`
        // (Python). The leftmost segment of these is treated as defined.
        if is_std_prefix(name, lang) {
            continue;
        }
        return Some(name.clone());
    }
    None
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
    "pub", "ref", "return", "static", "struct", "trait", "true", "type", "unsafe", "use", "where",
    "while", "yield", "macro_rules",
];

const PYTHON_KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
    "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if",
    "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "while", "with", "yield", "match", "case",
];

const RUST_PRIMITIVES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Rc", "Arc",
    "RefCell", "Cell", "Mutex", "RwLock", "HashMap", "HashSet", "BTreeMap", "BTreeSet", "VecDeque",
    "PathBuf", "Path", "OsString", "OsStr", "Some", "None", "Ok", "Err", "Self", "Sized", "Copy",
    "Clone", "Debug", "Default", "Eq", "PartialEq", "Ord", "PartialOrd", "Hash", "Send", "Sync",
    "Drop", "Iterator", "IntoIterator", "FromIterator", "From", "Into", "TryFrom", "TryInto",
    "AsRef", "AsMut", "Display", "Error", "Fn", "FnMut", "FnOnce",
];

const PYTHON_PRIMITIVES: &[&str] = &[
    "int", "str", "float", "bool", "list", "dict", "tuple", "set", "frozenset", "bytes",
    "bytearray", "complex", "object", "type", "range", "slice", "memoryview", "None", "True",
    "False", "print", "len", "abs", "all", "any", "sum", "min", "max", "open", "input", "iter",
    "next", "enumerate", "zip", "map", "filter", "sorted", "reversed", "round", "pow", "divmod",
    "hex", "oct", "bin", "chr", "ord", "repr", "format", "id", "hash", "isinstance",
    "issubclass", "callable", "getattr", "setattr", "hasattr", "delattr", "vars", "dir", "globals",
    "locals", "exec", "eval", "compile", "Exception", "ValueError", "TypeError", "KeyError",
    "IndexError", "RuntimeError", "StopIteration", "ArithmeticError", "AssertionError",
    "AttributeError", "ImportError", "NameError", "OSError", "FileNotFoundError",
];

fn is_keyword(name: &str, lang: InternalLang) -> bool {
    match lang {
        InternalLang::Rust => RUST_KEYWORDS.contains(&name),
        InternalLang::Python => PYTHON_KEYWORDS.contains(&name),
    }
}

fn is_primitive(name: &str, lang: InternalLang) -> bool {
    match lang {
        InternalLang::Rust => RUST_PRIMITIVES.contains(&name),
        InternalLang::Python => PYTHON_PRIMITIVES.contains(&name),
    }
}

fn is_self_marker(name: &str) -> bool {
    matches!(name, "self" | "Self" | "cls" | "super")
}

fn is_std_prefix(name: &str, lang: InternalLang) -> bool {
    match lang {
        InternalLang::Rust => matches!(name, "std" | "core" | "alloc"),
        InternalLang::Python => matches!(
            name,
            "__builtins__" | "__name__" | "__file__" | "__doc__" | "__init__"
        ),
    }
}

/// Identifier-classifier (locked decision §15). Order matters — first match wins.
fn classify_identifier(name: &str) -> MicroStepType {
    // SCREAMING_SNAKE_CASE → Const.
    if !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && name.chars().any(|c| c.is_ascii_uppercase())
    {
        return MicroStepType::Const;
    }
    // First-char uppercase → Struct (PascalCase / type-like).
    if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
        return MicroStepType::Struct;
    }
    // Path-like ('::' or '.') → Module.
    if name.contains("::") || name.contains('.') {
        return MicroStepType::Module;
    }
    // Default — snake_case in call position.
    MicroStepType::Function
}

/// Path heuristic (locked decision §15).
///   * Rust: `parent_dir(target) + snake_case(name) + ".rs"`. For Struct,
///     PascalCase → snake_case via ASCII rule.
///   * Python: `parent_dir(target) + name + ".py"` (case-preserved).
fn suggest_path(target: &Path, name: &str, lang: InternalLang) -> PathBuf {
    let parent = target.parent().map(Path::to_path_buf).unwrap_or_default();
    let base_name = match lang {
        InternalLang::Rust => snake_case_ascii(name),
        InternalLang::Python => name.to_string(),
    };
    let ext = match lang {
        InternalLang::Rust => "rs",
        InternalLang::Python => "py",
    };
    let file_name = format!("{base_name}.{ext}");
    if parent.as_os_str().is_empty() {
        PathBuf::from(file_name)
    } else {
        parent.join(file_name)
    }
}

/// ASCII-only PascalCase → snake_case. Insert `_` before each uppercase letter
/// that follows a lowercase or digit, then lowercase the whole thing. Already-
/// snake_case input passes through unchanged.
fn snake_case_ascii(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 4);
    let mut prev_lower_or_digit = false;
    for c in input.chars() {
        if c.is_ascii_uppercase() {
            if prev_lower_or_digit {
                out.push('_');
            }
            for lc in c.to_lowercase() {
                out.push(lc);
            }
            prev_lower_or_digit = false;
        } else {
            out.push(c);
            prev_lower_or_digit = c.is_ascii_lowercase() || c.is_ascii_digit();
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Reader-side aggregation — parallels `dependency_resolution_stats_for_payloads`
// ---------------------------------------------------------------------------

/// Aggregated stats for the future `micro_step_created` SSB action_type.
/// Today the variant does NOT exist on `SsbPayload` (ssb.rs is LOCKED by
/// session s-28564); the API surface is locked here so the future batched
/// SSB-bump sub-phase only needs to plug in a match arm. Mirrors the
/// E-5a `PreAssemblyStats` and E-6 `DependencyResolutionStats` precedent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileRouterStats {
    /// Number of `micro_step_created` rows observed.
    pub total_micro_steps_created: u64,
    /// Number of `MicroStepCapExhausted` rows observed.
    pub total_cap_exhausted: u64,
    /// Number of `ParseSkipped` rows observed (false-negative bias visibility).
    pub total_parse_skipped: u64,
    /// B-28 WIRING-1: per-`type_hint` counts (`"Module"` / `"Struct"` /
    /// `"Function"` / `"Const"`). BTreeMap for deterministic order (auditor
    /// invariant + dashboard JSON stability).
    pub per_type_hint: std::collections::BTreeMap<String, u32>,
}

/// Aggregate a slice of [`SsbPayload`] borrows into [`FileRouterStats`].
///
/// B-28 WIRING-1: scans `SsbPayload::MicroStepCreated` rows and produces
/// per-decision counters + per-type_hint buckets. Wire-form distinction:
///   * `reason == "cap_exhausted"` -> `total_cap_exhausted += 1`.
///   * `reason == "parse_skipped"` -> `total_parse_skipped += 1`.
///   * any other reason (e.g. `"undefined_symbol"`) -> NeedsMicroStep
///     branch -> `total_micro_steps_created += 1` and bucket
///     `per_type_hint[type_hint] += 1`.
/// Any other payload variants are skipped, so callers may pass an
/// unfiltered SSB stream.
#[must_use]
pub fn file_router_stats_for_payloads(payloads: &[SsbPayload]) -> FileRouterStats {
    let mut stats = FileRouterStats::default();
    for payload in payloads {
        if let SsbPayload::MicroStepCreated(p) = payload {
            match p.reason.as_str() {
                "cap_exhausted" => {
                    stats.total_cap_exhausted = stats.total_cap_exhausted.saturating_add(1);
                }
                "parse_skipped" => {
                    stats.total_parse_skipped = stats.total_parse_skipped.saturating_add(1);
                }
                _ => {
                    stats.total_micro_steps_created =
                        stats.total_micro_steps_created.saturating_add(1);
                    let hint_entry =
                        stats.per_type_hint.entry(p.type_hint.clone()).or_insert(0);
                    *hint_entry = hint_entry.saturating_add(1);
                }
            }
        }
    }
    stats
}

// ---------------------------------------------------------------------------
// Inline unit tests — wide coverage lives in
// crates/contracts/tests/focus_tunnel_e7_contract.rs.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_ascii_handles_pascal_and_mixed() {
        assert_eq!(snake_case_ascii("Widget"), "widget");
        assert_eq!(snake_case_ascii("MyWidget"), "my_widget");
        assert_eq!(snake_case_ascii("HTTPServer"), "httpserver");
        assert_eq!(snake_case_ascii("already_snake"), "already_snake");
        assert_eq!(snake_case_ascii("X"), "x");
        assert_eq!(snake_case_ascii(""), "");
    }

    #[test]
    fn classify_identifier_resolves_by_casing() {
        assert_eq!(classify_identifier("MAX_TOKENS"), MicroStepType::Const);
        assert_eq!(classify_identifier("Widget"), MicroStepType::Struct);
        assert_eq!(classify_identifier("foo::bar"), MicroStepType::Module);
        assert_eq!(classify_identifier("process_foo"), MicroStepType::Function);
        assert_eq!(classify_identifier("x"), MicroStepType::Function);
    }

    #[test]
    fn parser_cache_reused_across_calls() {
        let mut router = SpeculativeFileRouter::new(2);
        let target = PathBuf::from("crates/foo/src/bar.rs");
        // Two routes against the same router instance — parser stays cached.
        let _ = router.route(
            &target,
            "pub fn helper() {}\n",
            None,
            "fn caller() { helper(); }\n",
            ImportLanguage::Rust,
        );
        // After first call, parser_rs is Some.
        assert!(router.parser_rs.is_some());
        let _ = router.route(
            &target,
            "pub fn helper() {}\n",
            None,
            "fn caller() { helper(); }\n",
            ImportLanguage::Rust,
        );
        // Still cached — same Option::Some.
        assert!(router.parser_rs.is_some());
    }

    #[test]
    fn unsupported_language_short_circuits() {
        let mut router = SpeculativeFileRouter::new(2);
        let decision = router.route(
            Path::new("any.go"),
            "",
            None,
            "",
            ImportLanguage::Go,
        );
        assert!(matches!(decision, RouteDecision::ParseSkipped { .. }));
    }
}
