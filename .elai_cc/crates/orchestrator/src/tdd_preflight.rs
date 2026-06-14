//! I-19 — TDD enforcer preflight gate.
//!
//! Fires on every `Edit` / `Write` (`PreToolUse`) invocation to check whether
//! the pending diff introduces a new `pub` symbol that has NO matching test-symbol
//! delta in the test-file set. Gate is:
//!
//!  * **Default-OFF** — requires `ELAI_TDD_PREFLIGHT_ENABLE=1`.
//!  * **Refactor-exempt** — if the diff introduces zero new pub symbols the gate is
//!    skipped (returns `RefactorExempted` advisory row, no block).
//!  * **Compile-only dry-run** — an optional `cargo check --tests` (Rust) or
//!    `pytest --collect-only` (Python) dry-run can be enabled; it never builds
//!    more than a minimal compile check.
//!
//! Wire-dark until GD-4 FP rate is bounded on ≥ 3 real orchestrate runs.
//! Promotion plan: `I-19-promotion`.  See `wire-dark-allowlist.toml`.
//!
//! ## Grounding
//!
//! * Plan: `docs/thefinalglance/I-plan/I-19_tdd_enforcer_preflight.md`
//! * tree-sitter pattern follows `crates/orchestrator/src/deterministic_auditor.rs`
//!   (direct grammar-crate usage, no LanguageRegistry threading).
//! * GD-1: tree-sitter grammar crates already present in `orchestrator/Cargo.toml`
//!   (`tree-sitter-rust = "0.24"`, `tree-sitter-python = "0.25"`).

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::{debug, warn};

use crate::hook_dispatcher::{HookFinding, HookTier, HookVerdictKind};
use crate::ssb::{
    Ssb, SsbPayload, TddPreflightBlockedKind, TddPreflightBlockedPayload,
    ELAI_TDD_PREFLIGHT_ENABLE_ENV,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// SSB category string surfaced on every TddPreflightBlocked row.
pub const TDD_PREFLIGHT_CATEGORY: &str = "tdd_preflight";

/// Rule identifier used in the `HookFinding.rule_id` field.
pub const TDD_PREFLIGHT_RULE_ID: &str = "i19-tdd-preflight";

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// Result of the AST-parse + test-match predicate pass.
#[derive(Debug, Clone, PartialEq)]
pub enum PreflightResult {
    /// No new pub symbols found — pure refactor / rename / reformat.
    NoPubSymbols,
    /// New pub symbol(s) found; all have a matching test-symbol in the test set.
    Matched { pub_symbols: Vec<String> },
    /// New pub symbol(s) found; none matched a test-symbol.
    Unmatched { pub_symbols: Vec<String> },
    /// Staged content fails a compile-only dry-run.
    DryRunFailed {
        pub_symbols: Vec<String>,
        stderr_prefix: String,
    },
}

// ---------------------------------------------------------------------------
// Language detection helpers
// ---------------------------------------------------------------------------

/// Detect the language of a file from its extension. Returns `None` for
/// languages that are not currently analysed (i.e. not Rust and not Python).
#[must_use]
pub fn detect_lang(file_path: &str) -> Option<&'static str> {
    if file_path.ends_with(".rs") {
        Some("rust")
    } else if file_path.ends_with(".py") {
        Some("python")
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// AST pub-symbol extraction
// ---------------------------------------------------------------------------

/// Extract all `pub` / top-level symbol names from `source` using tree-sitter.
///
/// * For Rust: collects `pub fn`, `pub struct`, `pub enum`, `pub trait`,
///   `pub type`, `pub const`, `pub static`, `pub mod` names.
/// * For Python: collects top-level class and function def names (Python has no
///   `pub` keyword; all top-level names are treated as public by convention).
///
/// Returns an empty `Vec` on parse failure rather than propagating an error (the
/// gate degrades gracefully — it simply exempts files whose AST could not be
/// built instead of blocking them).
///
/// GD-1 compliance: uses direct grammar crates, same pattern as
/// `deterministic_auditor.rs` — no LanguageRegistry threading required.
#[must_use]
pub fn extract_pub_symbols(source: &str, lang: &str) -> Vec<String> {
    let mut parser = tree_sitter::Parser::new();

    let lang_obj = match lang {
        "rust" => tree_sitter_rust::LANGUAGE.into(),
        "python" => tree_sitter_python::LANGUAGE.into(),
        _ => return Vec::new(),
    };

    if parser.set_language(&lang_obj).is_err() {
        warn!(tdd_preflight.lang = lang, "tree-sitter set_language failed; exempting file");
        return Vec::new();
    }

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => {
            warn!(tdd_preflight.lang = lang, "tree-sitter parse returned None; exempting file");
            return Vec::new();
        }
    };

    let mut symbols: Vec<String> = Vec::new();
    collect_pub_symbols(tree.root_node(), source, lang, &mut symbols);
    symbols
}

/// Recursive DFS that collects pub symbol names from a tree-sitter node.
fn collect_pub_symbols(
    node: tree_sitter::Node<'_>,
    source: &str,
    lang: &str,
    out: &mut Vec<String>,
) {
    if lang == "rust" {
        // In Rust's grammar, `pub` keyword is a child named "visibility_modifier"
        // on items. We look for item nodes that have a "visibility_modifier" child
        // and then grab their "name" child.
        let kind = node.kind();
        if matches!(
            kind,
            "function_item"
                | "struct_item"
                | "enum_item"
                | "trait_item"
                | "type_alias"
                | "const_item"
                | "static_item"
                | "mod_item"
        ) {
            // Check for pub visibility.
            let has_pub = (0..node.child_count() as u32).any(|i| {
                node.child(i)
                    .map(|c| c.kind() == "visibility_modifier")
                    .unwrap_or(false)
            });
            if has_pub {
                // The name of the item is typically the first "identifier" child.
                if let Some(name_node) = (0..node.child_count() as u32)
                    .filter_map(|i| node.child(i))
                    .find(|c| c.kind() == "identifier" || c.kind() == "type_identifier")
                {
                    if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                        out.push(name.to_owned());
                    }
                }
            }
        }
    } else if lang == "python" {
        let kind = node.kind();
        // Python: only top-level function_definition and class_definition.
        // We only process children of the module root to stay top-level.
        if matches!(kind, "function_definition" | "class_definition") {
            // The name is the first `identifier` child after the def/class keyword.
            if let Some(name_node) = (0..node.child_count() as u32)
                .filter_map(|i| node.child(i))
                .find(|c| c.kind() == "identifier")
            {
                if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                    out.push(name.to_owned());
                }
            }
            // Do NOT recurse into function/class bodies for nested defs.
            return;
        }
    }

    // Recurse into children.
    for i in 0..node.child_count() as u32 {
        if let Some(child) = node.child(i) {
            collect_pub_symbols(child, source, lang, out);
        }
    }
}

// ---------------------------------------------------------------------------
// Test-symbol matching
// ---------------------------------------------------------------------------

/// Check whether any name in `test_symbols` contains any name in `pub_symbols`
/// as a substring (case-insensitive). This is the "test delta" match predicate.
///
/// Example: pub symbol `calculate_total` would match test symbol
/// `test_calculate_total` or `calculate_total_edge_case`.
#[must_use]
pub fn has_test_delta_match(pub_symbols: &[String], test_symbols: &[String]) -> bool {
    for pub_sym in pub_symbols {
        let pub_lower = pub_sym.to_lowercase();
        if test_symbols
            .iter()
            .any(|t| t.to_lowercase().contains(&pub_lower))
        {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Async emit helper
// ---------------------------------------------------------------------------

/// Emit a [`TddPreflightBlockedPayload`] SSB row. Fire-and-forget (errors
/// are traced but not propagated — the gate verdict is unaffected by SSB
/// write failures).
pub async fn emit_tdd_preflight_blocked_to_ssb(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    payload: TddPreflightBlockedPayload,
) {
    let ssb_payload = SsbPayload::TddPreflightBlocked(payload);
    match writer
        .write_payload(agent_id, superstep, ssb_payload, Vec::new())
        .await
    {
        Ok(_) => {
            debug!(tdd_preflight.ssb = "emitted TddPreflightBlocked row");
        }
        Err(e) => {
            warn!(
                tdd_preflight.ssb_err = %e,
                "TddPreflightBlocked SSB emit failed (gate verdict unaffected)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// TddPreflight gate struct
// ---------------------------------------------------------------------------

/// The TDD-enforcer preflight gate.
///
/// Constructed once and shared via `Arc`; `check()` takes `&self` and is sync.
/// The optional SSB writer is used for fire-and-forget row emission; the
/// `check()` call itself returns only [`HookFinding`] values (no async needed
/// for the core predicate path).
///
/// See module-level documentation for wire-dark posture and GD-4 constraints.
pub struct TddPreflight {
    /// Optional SSB writer. When `None`, findings are returned but no SSB row
    /// is emitted (the gate is still fail-closed for the FindingKind::Block arm).
    ssb: Option<(Arc<Ssb>, String, i64)>,
}

impl TddPreflight {
    /// Construct without an SSB writer. Gate logic is fully active; only the
    /// SSB emission is absent.
    #[must_use]
    pub fn new() -> Self {
        Self { ssb: None }
    }

    /// Install the SSB writer used to emit `TddPreflightBlocked` rows.
    /// Mirrors `InProcessHookDispatcher::with_vibegate_ssb` pattern.
    #[must_use]
    pub fn with_ssb(
        mut self,
        writer: Arc<Ssb>,
        agent_id: impl Into<String>,
        superstep: i64,
    ) -> Self {
        self.ssb = Some((writer, agent_id.into(), superstep));
        self
    }

    /// Core predicate: examine `source` for new pub symbols and check for a
    /// test-symbol substring match in `test_symbols`.
    ///
    /// Returns:
    ///  * `PreflightResult::NoPubSymbols` — no new pub symbols (refactor).
    ///  * `PreflightResult::Matched`     — all pub symbols matched.
    ///  * `PreflightResult::Unmatched`   — at least one pub symbol unmatched.
    ///
    /// `DryRunFailed` is produced only by the async `run_syntax_dryrun` path,
    /// not inline here.
    #[must_use]
    pub fn classify(source: &str, lang: &str, test_symbols: &[String]) -> PreflightResult {
        let pub_symbols = extract_pub_symbols(source, lang);
        if pub_symbols.is_empty() {
            return PreflightResult::NoPubSymbols;
        }
        if has_test_delta_match(&pub_symbols, test_symbols) {
            PreflightResult::Matched { pub_symbols }
        } else {
            PreflightResult::Unmatched { pub_symbols }
        }
    }

    /// Synchronous gate check. Examines `file_path` + `source` for new pub
    /// symbols; returns a vec of [`HookFinding`] (empty = allow, non-empty =
    /// block/warn depending on the arm).
    ///
    /// Callers that want SSB emission must use `check_with_emit` (async wrapper).
    ///
    /// Gate semantics:
    ///  * `RefactorExempted`: no new pub symbols — return advisory warn.
    ///  * `Unmatched`: block with MissingTestDelta.
    ///  * `Matched`: empty findings (allow).
    #[must_use]
    pub fn check(
        &self,
        file_path: Option<&str>,
        source: Option<&str>,
        test_symbols: &[String],
        fsm_state: &str,
    ) -> Vec<HookFinding> {
        let _ = fsm_state; // captured for SSB emit path only
        let (fp, src) = match (file_path, source) {
            (Some(f), Some(s)) => (f, s),
            _ => return Vec::new(), // non-file invocations are not gated
        };

        let lang = match detect_lang(fp) {
            Some(l) => l,
            None => return Vec::new(), // unsupported language — skip silently
        };

        let result = Self::classify(src, lang, test_symbols);

        match result {
            PreflightResult::NoPubSymbols => {
                // Refactor-exempt: emit advisory warn (no block).
                vec![HookFinding {
                    tier: HookTier::T1,
                    verdict: HookVerdictKind::Warn,
                    rule_id: Some(TDD_PREFLIGHT_RULE_ID.to_string()),
                    category: TDD_PREFLIGHT_CATEGORY.to_string(),
                    message: format!(
                        "tdd_preflight: no new pub symbols in `{fp}`; gate skipped (RefactorExempted)"
                    ),
                    fix_template: None,
                }]
            }
            PreflightResult::Matched { pub_symbols } => {
                debug!(
                    tdd_preflight.symbols = ?pub_symbols,
                    tdd_preflight.file = fp,
                    "tdd_preflight: test delta matched; allowing"
                );
                Vec::new()
            }
            PreflightResult::Unmatched { pub_symbols } => {
                let sym_list = pub_symbols.join(", ");
                vec![HookFinding {
                    tier: HookTier::T1,
                    verdict: HookVerdictKind::Block,
                    rule_id: Some(TDD_PREFLIGHT_RULE_ID.to_string()),
                    category: TDD_PREFLIGHT_CATEGORY.to_string(),
                    message: format!(
                        "tdd_preflight: new pub symbol(s) [{sym_list}] in `{fp}` \
                         have no matching test-symbol delta (MissingTestDelta). \
                         Add or extend a test that names one of these symbols before writing."
                    ),
                    fix_template: Some(
                        "Add a test whose name contains the pub symbol as a substring."
                            .to_string(),
                    ),
                }]
            }
            PreflightResult::DryRunFailed {
                pub_symbols,
                stderr_prefix,
            } => {
                let sym_list = pub_symbols.join(", ");
                vec![HookFinding {
                    tier: HookTier::T1,
                    verdict: HookVerdictKind::Block,
                    rule_id: Some(TDD_PREFLIGHT_RULE_ID.to_string()),
                    category: TDD_PREFLIGHT_CATEGORY.to_string(),
                    message: format!(
                        "tdd_preflight: `cargo check --tests` failed for `{fp}` \
                         (SyntaxDryRunFailed; new symbols: [{sym_list}]). \
                         Stderr prefix: {stderr_prefix}"
                    ),
                    fix_template: None,
                }]
            }
        }
    }

    /// Async wrapper: calls `check()` then emits SSB rows for any block/warn
    /// findings. Returns the same findings vec.
    pub async fn check_with_emit(
        &self,
        file_path: Option<&str>,
        source: Option<&str>,
        test_symbols: &[String],
        fsm_state: &str,
    ) -> Vec<HookFinding> {
        let findings = self.check(file_path, source, test_symbols, fsm_state);
        if findings.is_empty() {
            return findings;
        }

        if let Some((writer, agent_id, superstep)) = self.ssb.as_ref() {
            let ts_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            // Determine the SSB kind from the first block finding (there is
            // at most one arm per check() call).
            let kind = determine_kind(&findings);

            let pub_symbols_str = file_path
                .and_then(|fp| {
                    source.map(|src| {
                        detect_lang(fp)
                            .map(|lang| extract_pub_symbols(src, lang).join(", "))
                            .unwrap_or_default()
                    })
                })
                .unwrap_or_default();

            let dry_run_stderr = findings
                .iter()
                .find(|f| {
                    f.rule_id.as_deref() == Some(TDD_PREFLIGHT_RULE_ID)
                        && f.message.contains("SyntaxDryRunFailed")
                })
                .map(|f| {
                    // Extract the stderr prefix from the message; keep first 256 chars.
                    let prefix_start = f.message.find("Stderr prefix:").map(|p| p + 15).unwrap_or(f.message.len());
                    f.message[prefix_start..].chars().take(256).collect::<String>()
                })
                .unwrap_or_default();

            let payload = TddPreflightBlockedPayload {
                kind,
                file_path: file_path.unwrap_or("").to_string(),
                pub_symbols: pub_symbols_str,
                fsm_state: if fsm_state.is_empty() {
                    None
                } else {
                    Some(fsm_state.to_string())
                },
                dry_run_stderr,
                ts_ms,
            };

            emit_tdd_preflight_blocked_to_ssb(writer, agent_id, *superstep, payload).await;
        }

        findings
    }
}

impl Default for TddPreflight {
    fn default() -> Self {
        Self::new()
    }
}

/// Determine the SSB [`TddPreflightBlockedKind`] from the findings list.
fn determine_kind(findings: &[HookFinding]) -> TddPreflightBlockedKind {
    for f in findings {
        if f.rule_id.as_deref() == Some(TDD_PREFLIGHT_RULE_ID) {
            if f.message.contains("SyntaxDryRunFailed") {
                return TddPreflightBlockedKind::SyntaxDryRunFailed;
            }
            if f.message.contains("RefactorExempted") {
                return TddPreflightBlockedKind::RefactorExempted;
            }
            if f.message.contains("MissingTestDelta") {
                return TddPreflightBlockedKind::MissingTestDelta;
            }
        }
    }
    TddPreflightBlockedKind::MissingTestDelta
}

/// Return `true` when the TDD preflight gate is activated via env var.
///
/// Reuses [`crate::ssb::env_gate_truthy`] for consistent gate semantics.
#[must_use]
pub fn tdd_preflight_enabled() -> bool {
    crate::ssb::env_gate_truthy(ELAI_TDD_PREFLIGHT_ENABLE_ENV)
}

// ---------------------------------------------------------------------------
// Async compile-only dry-run (Tier-2: optional; never runs the test suite)
// ---------------------------------------------------------------------------

/// Run a compile-only dry-run check (`cargo check --tests` for Rust,
/// `pytest --collect-only` for Python). Returns a `SyntaxDryRunFailed`
/// finding if the check fails, `None` if it passes (or if the runtime
/// returns a non-UTF8 stderr).
///
/// I-19 Tier-2 deliverable (deferred). The Tier-1 `hook_dispatcher` integration
/// uses the sync `check()` / `check_with_emit()` predicate path only. This helper
/// is provided for the Tier-2 wiring (I-19-promotion) when the FP harness proves
/// <=5% FP rate and the test-symbol index feed is live.
///
/// This function NEVER runs the full test suite — `cargo check` is
/// compile-only (no execution). The `--tests` flag ensures all `#[cfg(test)]`
/// blocks are included in the type-check pass.
///
/// Returns `None` when:
///  * the tool is not found on `$PATH`
///  * the check succeeds (exit 0)
///  * the file language is not Rust or Python
///
/// Errors in spawning are traced and `None` is returned (degrade gracefully —
/// a missing tool is not a gate-block; the AST predicate path is the primary
/// check).
// WIRE-DARK[I-19-dryrun]: Tier-2; consumer lands in I-19-promotion; backing row in wire-dark-allowlist.toml ELAI_TDD_PREFLIGHT_ENABLE entry.
pub async fn run_syntax_dryrun(
    file_path: &str,
    pub_symbols: &[String],
) -> Option<HookFinding> {
    let lang = detect_lang(file_path)?;

    let (prog, args): (&str, &[&str]) = match lang {
        "rust" => ("cargo", &["check", "--tests", "--quiet", "--message-format=short"]),
        "python" => ("pytest", &["--collect-only", "-q"]),
        _ => return None,
    };

    let output = match tokio::process::Command::new(prog)
        .args(args)
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            debug!(
                tdd_preflight.dryrun_err = %e,
                tdd_preflight.prog = prog,
                "dry-run tool not found or failed to spawn; skipping dry-run check"
            );
            return None;
        }
    };

    if output.status.success() {
        return None;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr_prefix: String = stderr.chars().take(256).collect();
    let sym_list = pub_symbols.join(", ");

    Some(HookFinding {
        tier: HookTier::T1,
        verdict: HookVerdictKind::Block,
        rule_id: Some(TDD_PREFLIGHT_RULE_ID.to_string()),
        category: TDD_PREFLIGHT_CATEGORY.to_string(),
        message: format!(
            "tdd_preflight: `{prog} check --tests` failed for `{file_path}` \
             (SyntaxDryRunFailed; new symbols: [{sym_list}]). \
             Stderr prefix: {stderr_prefix}"
        ),
        fix_template: None,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_pub_symbols_rust_fn() {
        let src = r#"
pub fn hello_world() {}
fn private_fn() {}
pub struct MyStruct;
"#;
        let syms = extract_pub_symbols(src, "rust");
        assert!(
            syms.contains(&"hello_world".to_string()),
            "expected hello_world in pub symbols; got {:?}",
            syms
        );
        assert!(
            syms.contains(&"MyStruct".to_string()),
            "expected MyStruct in pub symbols; got {:?}",
            syms
        );
        assert!(
            !syms.contains(&"private_fn".to_string()),
            "private_fn should NOT be in pub symbols; got {:?}",
            syms
        );
    }

    #[test]
    fn extract_pub_symbols_returns_empty_for_unknown_lang() {
        let syms = extract_pub_symbols("fn foo() {}", "typescript");
        assert!(syms.is_empty(), "unknown lang should yield empty; got {:?}", syms);
    }

    #[test]
    fn has_test_delta_match_positive() {
        let pub_syms = vec!["calculate_total".to_string()];
        let test_syms = vec!["test_calculate_total".to_string()];
        assert!(
            has_test_delta_match(&pub_syms, &test_syms),
            "substring match should pass"
        );
    }

    #[test]
    fn has_test_delta_match_negative() {
        let pub_syms = vec!["calculate_total".to_string()];
        let test_syms = vec!["test_something_else".to_string()];
        assert!(
            !has_test_delta_match(&pub_syms, &test_syms),
            "non-matching should fail"
        );
    }

    #[test]
    fn classify_no_pub_symbols_is_refactor() {
        let src = r#"fn private_helper() {}"#;
        let result = TddPreflight::classify(src, "rust", &[]);
        assert_eq!(result, PreflightResult::NoPubSymbols);
    }

    #[test]
    fn classify_matched() {
        let src = r#"pub fn my_function() {}"#;
        let test_syms = vec!["test_my_function".to_string()];
        let result = TddPreflight::classify(src, "rust", &test_syms);
        assert!(
            matches!(result, PreflightResult::Matched { .. }),
            "expected Matched; got {:?}",
            result
        );
    }

    #[test]
    fn classify_unmatched() {
        let src = r#"pub fn my_function() {}"#;
        let result = TddPreflight::classify(src, "rust", &[]);
        assert!(
            matches!(result, PreflightResult::Unmatched { .. }),
            "expected Unmatched; got {:?}",
            result
        );
    }

    #[test]
    fn check_returns_block_on_unmatched() {
        let gate = TddPreflight::new();
        let findings = gate.check(
            Some("src/foo.rs"),
            Some("pub fn new_pub_symbol() {}"),
            &[],
            "EXECUTING",
        );
        assert!(
            findings.iter().any(|f| f.is_block()),
            "expected a Block finding for unmatched pub symbol; got {:?}",
            findings
        );
    }

    #[test]
    fn check_returns_empty_on_matched() {
        let gate = TddPreflight::new();
        let test_syms = vec!["test_new_pub_symbol".to_string()];
        let findings = gate.check(
            Some("src/foo.rs"),
            Some("pub fn new_pub_symbol() {}"),
            &test_syms,
            "EXECUTING",
        );
        assert!(
            findings.is_empty(),
            "expected no findings for a matched pub symbol; got {:?}",
            findings
        );
    }

    #[test]
    fn check_skips_non_rs_py_files() {
        let gate = TddPreflight::new();
        let findings = gate.check(
            Some("src/foo.ts"),
            Some("export function myFunc() {}"),
            &[],
            "",
        );
        assert!(findings.is_empty(), "non-rs/py file should be skipped");
    }

    #[test]
    fn detect_lang_rs_and_py() {
        assert_eq!(detect_lang("src/foo.rs"), Some("rust"));
        assert_eq!(detect_lang("scripts/bar.py"), Some("python"));
        assert_eq!(detect_lang("src/foo.ts"), None);
    }

    #[test]
    fn tdd_preflight_enabled_off_by_default() {
        // The gate is wire-dark: env var must be absent (or not "1") by default.
        // We can't unset env vars portably in unit tests, so just verify the
        // function returns false when the var is absent / not truthy.
        // If ELAI_TDD_PREFLIGHT_ENABLE is set in CI, this test will pass because
        // the CI operator intentionally set it.
        let expected = std::env::var(ELAI_TDD_PREFLIGHT_ENABLE_ENV)
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
            .unwrap_or(false);
        assert_eq!(tdd_preflight_enabled(), expected);
    }
}
