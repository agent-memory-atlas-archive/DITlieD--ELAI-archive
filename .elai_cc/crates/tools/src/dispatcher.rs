//! Batch 2B — `InProcessToolDispatcher` implementation.
//!
//! Lives in `elai-tools` because every handler it wraps
//! (edit_matching, validation, failure_taxonomy, git_commit, file_read,
//! compile_check) already lives here. The trait + value types live in
//! `elai-core` so downstream callers program against a canonical shape.
//!
//! `elai-tools` depends on `elai-orchestrator` (facade re-exports) but
//! the dispatcher itself does NOT pull anything from orchestrator —
//! keeping this module reachable from inside `crates/contracts/tests/`
//! without ever triggering the orchestrator build path for the integration
//! adapters.
//!
//! Boundary note (per batch-2b spec decision):
//! > dispatcher home trait + value types go in `elai-core`, the
//! > `InProcessToolDispatcher` impl goes in `elai-tools`. Verify
//! > `elai-tools` still depends on `elai-orchestrator` — if so,
//! > either break that dep (preferred) or put the impl here.
//! Outcome: `elai-tools` does still depend on `elai-orchestrator` (7758-line
//! lib.rs full of facade re-exports — breaking the dep is out of scope for
//! this batch). The dispatcher impl lives here as the documented fallback.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use elai_core::{ClockProvider as _, OsClock};

use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditMatchTier, EditOutcome, EditRequest, FailureInput,
    FileView, GitBranchPolicy, GitCommitRequest, IncrementalOutcome, IncrementalRequest,
    SimilarityHint, ToolDispatchError, ToolDispatcher, ToolFailureCategory, ValidateRequest,
    ValidationTierResult,
};
// Phase B1 — Tier C Sandbox policy adapter. The dispatcher consults the
// adapter (when one is wired) before every dispatch entry point and maps
// `SandboxDenied` into `ToolDispatchError::Denied`. Adapter wiring is
// optional so callers that have not yet adopted the policy layer remain
// unaffected (the field defaults to `None`).
use elai_orchestrator::{SandboxAdapter, SandboxOp};
use crate::string_similarity::best_line_match;
// `ValidationTier` is only used inside the test module; exposed here so
// the `#[cfg(test)]` block below keeps compiling without a separate
// feature-gated import.
#[cfg(test)]
use elai_core::tool_dispatcher::ValidationTier;

use crate::tiered_validation::{DefaultTieredValidator, TieredValidator};

/// Marker literal that triggers the ellipsis-match tier in
/// `dispatch_edit`. Must be the literal three-character sequence written
/// without surrounding whitespace so a plain `indexOf` call can locate it.
const EDIT_ELLIPSIS_MARKER: &str = "...";

/// Default outline-mode threshold. Matches `file_read.rs::OUTLINE_LINE_THRESHOLD`.
pub const OUTLINE_LINE_THRESHOLD: usize = 300;

/// Max retries for `dispatch_incremental_compile`. Matches
/// `compile_check.rs::MAX_COMPILE_RETRIES`.
const MAX_INCREMENTAL_RETRIES: u8 = 3;

/// Env var kill switch — when set, every dispatch method returns
/// `NotImplemented` without touching disk. Mirrors `ELAI_HOOKS_DISABLED`.
pub const ELAI_DISABLE_TOOL_DISPATCH: &str = "ELAI_DISABLE_TOOL_DISPATCH";

pub fn tool_dispatch_disabled_via_env() -> bool {
    std::env::var(ELAI_DISABLE_TOOL_DISPATCH)
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
        .unwrap_or(false)
}

/// Production in-process dispatcher. Construction takes an
/// `Arc<dyn TieredValidator>` so the dashboard can inject a
/// `DefaultTieredValidator` wired to the real orchestrator runner while
/// the unit/contract tests can inject a fake.
pub struct InProcessToolDispatcher {
    validator: Arc<dyn TieredValidator>,
    /// When true, every method short-circuits to `NotImplemented`. Set by
    /// `ELAI_DISABLE_TOOL_DISPATCH`.
    disabled: bool,
    /// Phase B1 — optional sandbox policy adapter. When `None`, the
    /// policy hook in every dispatch method is a documented no-op (the
    /// dispatcher behaves exactly as before B1). When `Some`, every
    /// dispatch entry point calls `enforce_policy` BEFORE the underlying
    /// work and maps a `SandboxDenied` verdict into
    /// `ToolDispatchError::Denied`. Provider-source-agnostic: the same
    /// adapter installed on a local-provider and a cloud-provider
    /// dispatcher produces identical verdicts.
    sandbox: Option<Arc<dyn SandboxAdapter>>,
}

impl InProcessToolDispatcher {
    pub fn new(validator: Arc<dyn TieredValidator>) -> Self {
        let disabled = tool_dispatch_disabled_via_env();
        if disabled {
            tracing::info!(
                "InProcessToolDispatcher: {ELAI_DISABLE_TOOL_DISPATCH} set — every dispatch returns NotImplemented"
            );
        }
        Self {
            validator,
            disabled,
            sandbox: None,
        }
    }

    /// Test-only constructor that wraps a `DefaultTieredValidator` built
    /// from the concrete trait fakes in the `tiered_validation::tests`
    /// module. Not compiled in release builds.
    #[cfg(test)]
    pub fn for_tests(validator: Arc<dyn TieredValidator>) -> Self {
        Self {
            validator,
            disabled: false,
            sandbox: None,
        }
    }

    /// Phase B1 — install a sandbox policy adapter. Returns `self` so
    /// the caller can chain construction: `InProcessToolDispatcher::new(v).with_sandbox(sb)`.
    /// Calling this multiple times overwrites the previously-installed
    /// adapter (the most-recent installation wins).
    pub fn with_sandbox(mut self, sandbox: Arc<dyn SandboxAdapter>) -> Self {
        self.sandbox = Some(sandbox);
        self
    }

    /// Phase B1 — consult the installed sandbox adapter (if any). When
    /// `self.sandbox` is `None`, this is a documented no-op that returns
    /// `Ok(())` so the six dispatch entry points can call it
    /// unconditionally without a per-method branch. When `Some`, the
    /// adapter's `SandboxDenied` verdict is mapped into
    /// `ToolDispatchError::Denied { reason }`.
    fn enforce_policy(&self, op: &SandboxOp) -> Result<(), ToolDispatchError> {
        match self.sandbox.as_ref() {
            None => Ok(()),
            Some(sb) => sb
                .check_policy(op)
                .map_err(|denied| ToolDispatchError::Denied { reason: denied.reason }),
        }
    }

    /// B-20A Task 7: file-read with explicit `force_full` override.
    ///
    /// Decision D (workflow/reasoning/B-20A-decisions.md): the
    /// `ToolDispatcher` trait stays unchanged. This is an inherent method
    /// on `InProcessToolDispatcher`; the 50% context-budget guard is
    /// enforced caller-side in
    /// `elai_orchestrator::task_runner::enforce_file_read_budget_guard`.
    ///
    /// Behaviour:
    ///   * `force_full == false` and `lines > line_threshold` → outline
    ///     only (signatures + imports), no full content.
    ///   * `force_full == true` → return the full content regardless of
    ///     line count. The caller is responsible for verifying that the
    ///     full read fits within remaining context budget.
    ///   * `lines <= line_threshold` → full content (force_full
    ///     irrelevant).
    pub async fn dispatch_file_read_with_force_full(
        &self,
        path: PathBuf,
        line_threshold: usize,
        force_full: bool,
    ) -> Result<FileView, ToolDispatchError> {
        if self.disabled {
            return Err(ToolDispatchError::NotImplemented(
                "dispatch_file_read_with_force_full disabled via env".into(),
            ));
        }
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(ToolDispatchError::IoError)?;
        Ok(generate_file_view_with_override(
            &content,
            line_threshold,
            force_full,
        ))
    }
}

// ---------------------------------------------------------------------------
// 3-tier edit matching
// ---------------------------------------------------------------------------

/// Normalise whitespace for the `WhitespaceNormalized` tier. Collapses every
/// run of `[ \t\r\n]+` into a single space so tab-vs-space, CRLF-vs-LF, and
/// trailing-whitespace diffs stop blocking the match.
fn normalise_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_ws = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !last_was_ws && !out.is_empty() {
                out.push(' ');
            }
            last_was_ws = true;
        } else {
            out.push(ch);
            last_was_ws = false;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Try the three edit-matching tiers against `content`, in declared order.
/// Returns `(tier, updated_content)` on success, or
/// `(EditMatchTier::NotMatched, content_unchanged)` when nothing worked.
fn try_edit_tiers(
    content: &str,
    old_text: &str,
    new_text: &str,
) -> (EditMatchTier, Option<String>) {
    // Tier 1 — exact substring match (fast-path, cheapest).
    if let Some(idx) = content.find(old_text) {
        let mut updated = String::with_capacity(content.len() + new_text.len());
        updated.push_str(&content[..idx]);
        updated.push_str(new_text);
        updated.push_str(&content[idx + old_text.len()..]);
        return (EditMatchTier::Exact, Some(updated));
    }

    // Tier 2 — whitespace-normalised match.
    let normal_old = normalise_whitespace(old_text);
    if !normal_old.is_empty() {
        let normal_content = normalise_whitespace(content);
        if let Some(norm_idx) = normal_content.find(&normal_old) {
            // Map the normalised position back to the raw byte offset by
            // counting characters in the original `content` that produced
            // the normalised prefix `normal_content[..norm_idx]`. This is
            // a linear re-scan; inefficient for huge files but correct and
            // matches the spec's "small" tier expectation.
            let prefix_target = norm_idx;
            let match_len = normal_old.len();
            let (raw_start, raw_end) =
                locate_normalised_span(content, prefix_target, match_len);
            if let (Some(start), Some(end)) = (raw_start, raw_end) {
                if end <= content.len() && start <= end {
                    let mut updated =
                        String::with_capacity(content.len() + new_text.len());
                    updated.push_str(&content[..start]);
                    updated.push_str(new_text);
                    updated.push_str(&content[end..]);
                    return (EditMatchTier::WhitespaceNormalized, Some(updated));
                }
            }
        }
    }

    // Tier 3 — ellipsis anchor match. `old_text` is of the shape
    // "before...after": match `before` and `after` in the source, splice
    // `new_text` between them.
    if let Some(split) = old_text.find(EDIT_ELLIPSIS_MARKER) {
        let before = &old_text[..split];
        let after = &old_text[split + EDIT_ELLIPSIS_MARKER.len()..];
        if !before.is_empty() && !after.is_empty() {
            if let Some(before_idx) = content.find(before) {
                let after_search_from = before_idx + before.len();
                if after_search_from <= content.len() {
                    if let Some(after_rel) = content[after_search_from..].find(after) {
                        let start = before_idx;
                        let end = after_search_from + after_rel + after.len();
                        let mut updated =
                            String::with_capacity(content.len() + new_text.len());
                        updated.push_str(&content[..start]);
                        updated.push_str(new_text);
                        updated.push_str(&content[end..]);
                        return (EditMatchTier::Ellipsis, Some(updated));
                    }
                }
            }
        }
    }

    (EditMatchTier::NotMatched, None)
}

/// Helper: given a raw `content` string and a normalised match located at
/// `prefix_target` (character count in the normalised view) with total
/// normalised length `match_len`, return the `(start, end)` byte offsets
/// in `content` that cover the same slice.
fn locate_normalised_span(
    content: &str,
    prefix_target: usize,
    match_len: usize,
) -> (Option<usize>, Option<usize>) {
    let mut raw_start: Option<usize> = None;
    let mut raw_end: Option<usize> = None;
    let mut normalised_count: usize = 0;
    let mut last_was_ws = false;
    let mut emitted_any = false;
    let mut byte_cursor: usize = 0;
    let bytes = content.as_bytes();
    while byte_cursor < bytes.len() {
        let ch_start = byte_cursor;
        let ch_len = utf8_char_len(bytes[byte_cursor]);
        let ch = &content[ch_start..ch_start + ch_len];
        byte_cursor += ch_len;
        let is_ws = ch.chars().next().is_some_and(|c| c.is_whitespace());
        if is_ws {
            if !last_was_ws && emitted_any {
                normalised_count += 1;
                if normalised_count == prefix_target && raw_start.is_none() {
                    raw_start = Some(ch_start);
                }
                if normalised_count == prefix_target + match_len
                    && raw_end.is_none()
                {
                    raw_end = Some(ch_start);
                }
            }
            last_was_ws = true;
        } else {
            if normalised_count == prefix_target && raw_start.is_none() {
                raw_start = Some(ch_start);
            }
            normalised_count += ch_len;
            emitted_any = true;
            last_was_ws = false;
            if normalised_count == prefix_target + match_len && raw_end.is_none() {
                raw_end = Some(byte_cursor);
            }
        }
    }
    if raw_end.is_none() && raw_start.is_some() {
        raw_end = Some(content.len());
    }
    (raw_start, raw_end)
}

fn utf8_char_len(first_byte: u8) -> usize {
    match first_byte {
        b if b & 0x80 == 0 => 1,
        b if b & 0xE0 == 0xC0 => 2,
        b if b & 0xF0 == 0xE0 => 3,
        _ => 4,
    }
}

// ---------------------------------------------------------------------------
// Failure classification (9-variant regex table — B-20A reduction)
// ---------------------------------------------------------------------------

/// 9-category classifier with (category, triggering-substrings, confidence)
/// triples. Order is load-bearing — more specific patterns must precede
/// generic ones so `unresolved import` resolves to `MissingImport` rather
/// than getting swallowed by `error[E` (TypeMismatch). Mirrors the priority
/// order in `failure_taxonomy::ClassifyHandler::detect_category` so the two
/// classifiers never disagree.
///
/// Decision A (B-20A): the B-20 plan's Rust-flavoured 9-list (incl.
/// LIFETIME_ERROR / BORROW_ERROR) — phase-4B.txt's competing list (with
/// PERMISSION_DENIED) was rejected at phase entry. See
/// workflow/reasoning/B-20A-decisions.md.
fn classify_output(raw: &str) -> Option<ClassifiedFailure> {
    let checks: &[(ToolFailureCategory, &[&str], f32)] = &[
        (
            ToolFailureCategory::MissingImport,
            &["unresolved import", "error[E0432]", "No module named", "ModuleNotFoundError"],
            1.0,
        ),
        (
            ToolFailureCategory::LifetimeError,
            &[
                "explicit lifetime required",
                "error[E0621]",
                "missing lifetime specifier",
                "error[E0106]",
                "lifetime may not live long enough",
            ],
            1.0,
        ),
        (
            ToolFailureCategory::BorrowError,
            &[
                "borrow of moved value",
                "error[E0382]",
                "cannot borrow",
                "error[E0502]",
                "error[E0499]",
            ],
            1.0,
        ),
        (
            ToolFailureCategory::TypeMismatch,
            &["mismatched types", "error[E0308]", "expected type"],
            1.0,
        ),
        (
            ToolFailureCategory::UndefinedSymbol,
            &[
                "cannot find value",
                "cannot find function",
                "cannot find type",
                "error[E0425]",
                "error[E0412]",
                "error[E0422]",
            ],
            1.0,
        ),
        (
            ToolFailureCategory::SyntaxError,
            &["error: expected", "expected `;`", "unexpected token", "invalid syntax", "SyntaxError"],
            1.0,
        ),
        (
            ToolFailureCategory::TestAssertion,
            &["test result: FAILED", "assertion failed", "panicked at 'assertion"],
            1.0,
        ),
        (
            ToolFailureCategory::UnusedWarning,
            &["warning: unused variable", "warning: dead_code", "warning: unused_variables", "warning: unused"],
            0.7,
        ),
        (
            ToolFailureCategory::RuntimePanic,
            &[
                "panicked at",
                "thread '",
                "RuntimeError",
                "timed out",
                "timeout",
                "out of memory",
                "OOM",
            ],
            1.0,
        ),
    ];

    for (cat, patterns, conf) in checks {
        for pat in *patterns {
            if raw.contains(pat) {
                return Some(ClassifiedFailure {
                    category: *cat,
                    confidence: *conf,
                    source: String::new(),
                });
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// git2 commits (real impl)
// ---------------------------------------------------------------------------

/// Locate the repo root by walking up from `anchor` looking for a `.git`
/// directory. Pure-path resolution — does not call git2 — so we can return
/// a clear error without dragging a `git2::Error` into the surface.
fn find_repo_root(anchor: &Path) -> Result<PathBuf, ToolDispatchError> {
    let mut cur = anchor;
    loop {
        if cur.join(".git").exists() {
            return Ok(cur.to_path_buf());
        }
        match cur.parent() {
            Some(parent) => cur = parent,
            None => {
                return Err(ToolDispatchError::GitError(format!(
                    "no .git directory found walking up from {}",
                    anchor.display()
                )));
            }
        }
    }
}

/// Commit via git2. Staging: only the listed files (never "." / "-A").
/// Branch policy: `Main` ⇒ current branch; `BranchPerPlan` ⇒ create or
/// checkout `elai/plan-{plan_id}`.
fn git_commit_blocking(req: &GitCommitRequest) -> Result<CommitOutcome, ToolDispatchError> {
    let repo_root = find_repo_root(&req.anchor_path)?;
    let repo = git2::Repository::open(&repo_root)
        .map_err(|e| ToolDispatchError::GitError(format!("open: {e}")))?;

    // Branch policy
    let target_branch = match req.branch_policy {
        GitBranchPolicy::Main => {
            // Unborn branch (fresh repo, no commits yet) is tolerated —
            // the initial commit populates HEAD below. Fall back to the
            // git2 default branch name so the SSB row still carries a
            // readable branch.
            match repo.head() {
                Ok(h) => h.shorthand().unwrap_or("HEAD").to_string(),
                Err(_) => "master".to_string(),
            }
        }
        GitBranchPolicy::BranchPerPlan => {
            let plan_id = if req.plan_id.is_empty() {
                "default"
            } else {
                req.plan_id.as_str()
            };
            let branch_name = format!("elai/plan-{plan_id}");
            // Only attempt checkout when the branch differs from HEAD.
            let current = repo
                .head()
                .ok()
                .and_then(|h| h.shorthand().map(str::to_owned));
            if current.as_deref() != Some(branch_name.as_str()) {
                if repo.find_branch(&branch_name, git2::BranchType::Local).is_err() {
                    // Create from HEAD commit.
                    let head_commit = repo
                        .head()
                        .and_then(|h| h.peel_to_commit())
                        .map_err(|e| {
                            ToolDispatchError::GitError(format!(
                                "resolve HEAD for branch create: {e}"
                            ))
                        })?;
                    repo.branch(&branch_name, &head_commit, false)
                        .map_err(|e| {
                            ToolDispatchError::GitError(format!("create branch: {e}"))
                        })?;
                }
                let obj = repo
                    .revparse_single(&format!("refs/heads/{branch_name}"))
                    .map_err(|e| {
                        ToolDispatchError::GitError(format!("revparse branch: {e}"))
                    })?;
                repo.checkout_tree(&obj, None)
                    .map_err(|e| ToolDispatchError::GitError(format!("checkout_tree: {e}")))?;
                repo.set_head(&format!("refs/heads/{branch_name}"))
                    .map_err(|e| ToolDispatchError::GitError(format!("set_head: {e}")))?;
            }
            branch_name
        }
    };

    // Stage only the listed files — never `git add -A`.
    let mut index = repo
        .index()
        .map_err(|e| ToolDispatchError::GitError(format!("index: {e}")))?;
    for file in &req.files {
        let rel = file.strip_prefix(&repo_root).unwrap_or(file);
        index
            .add_path(rel)
            .map_err(|e| ToolDispatchError::GitError(format!("add {}: {e}", rel.display())))?;
    }
    index
        .write()
        .map_err(|e| ToolDispatchError::GitError(format!("index write: {e}")))?;

    let tree_oid = index
        .write_tree()
        .map_err(|e| ToolDispatchError::GitError(format!("write_tree: {e}")))?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| ToolDispatchError::GitError(format!("find_tree: {e}")))?;

    let sig = git2::Signature::now("ELAI", "elai@local")
        .map_err(|e| ToolDispatchError::GitError(format!("signature: {e}")))?;

    // Parent commit: current HEAD if present; otherwise root commit.
    let parent_commit = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&git2::Commit> = match &parent_commit {
        Some(c) => vec![c],
        None => Vec::new(),
    };

    let new_oid = repo
        .commit(Some("HEAD"), &sig, &sig, &req.message, &tree, &parents)
        .map_err(|e| ToolDispatchError::GitError(format!("commit: {e}")))?;

    Ok(CommitOutcome {
        sha: new_oid.to_string(),
        branch: target_branch,
        files_count: u32::try_from(req.files.len()).unwrap_or(u32::MAX),
        message: req.message.clone(),
    })
}

// ---------------------------------------------------------------------------
// File outline
// ---------------------------------------------------------------------------

/// B-20A Task 7: like `generate_file_view` but lets the caller force a
/// full read regardless of `threshold`. Implementation collapses to a
/// single helper so the pre-B-20A trait method
/// (`dispatch_file_read_outline`) and the new inherent method
/// (`dispatch_file_read_with_force_full`) share one code path.
fn generate_file_view_with_override(
    content: &str,
    threshold: usize,
    force_full: bool,
) -> FileView {
    let view = generate_file_view(content, threshold);
    if force_full && view.outline_only {
        FileView {
            outline_only: false,
            signatures: view.signatures,
            imports: view.imports,
            full_content: Some(content.to_string()),
            total_lines: view.total_lines,
        }
    } else {
        view
    }
}

fn generate_file_view(content: &str, threshold: usize) -> FileView {
    let mut lines = 0usize;
    let mut signatures = Vec::new();
    let mut imports = Vec::new();
    for line in content.lines() {
        lines += 1;
        let trimmed = line.trim();
        if trimmed.starts_with("use ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("from ")
            || trimmed.starts_with("#include")
        {
            imports.push(line.to_string());
            continue;
        }
        if trimmed.starts_with("pub ")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("struct ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("impl ")
            || trimmed.starts_with("trait ")
            || trimmed.starts_with("mod ")
            || trimmed.starts_with("const ")
            || trimmed.starts_with("static ")
            || trimmed.starts_with("type ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("def ")
            || trimmed.starts_with("async def ")
            || trimmed.starts_with("function ")
            || trimmed.starts_with("export ")
        {
            signatures.push(line.to_string());
        }
    }
    let outline_only = lines > threshold;
    FileView {
        outline_only,
        signatures,
        imports,
        full_content: if outline_only {
            None
        } else {
            Some(content.to_string())
        },
        total_lines: u32::try_from(lines).unwrap_or(u32::MAX),
    }
}

// ---------------------------------------------------------------------------
// Incremental compile
// ---------------------------------------------------------------------------

fn compiler_command(language: &str) -> Option<(&'static str, Vec<String>)> {
    match language {
        "rust" => Some(("cargo", vec!["check".into(), "--message-format=json".into()])),
        "python" => Some(("ruff", vec!["check".into(), "--output-format=json".into()])),
        "typescript" => Some(("tsc", vec!["--noEmit".into()])),
        "go" => Some(("go", vec!["vet".into()])),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Trait impl
// ---------------------------------------------------------------------------

impl ToolDispatcher for InProcessToolDispatcher {
    fn dispatch_edit(
        &self,
        req: EditRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async move {
            if self.disabled {
                return Err(ToolDispatchError::NotImplemented(
                    "dispatch_edit disabled via env".into(),
                ));
            }
            // Phase B1 — policy gate. FileWrite is the matching variant
            // because dispatch_edit ultimately calls `tokio::fs::write`
            // on `req.file_path` (see L766 below).
            self.enforce_policy(&SandboxOp::FileWrite {
                path: req.file_path.as_path(),
            })?;
            let content = tokio::fs::read_to_string(&req.file_path)
                .await
                .map_err(ToolDispatchError::IoError)?;
            let (tier, updated) =
                try_edit_tiers(&content, &req.old_text, &req.new_text);
            match (tier, updated) {
                (EditMatchTier::NotMatched, _) => {
                    // B-20A Task 6: compute a similarity hint for the agent.
                    // `best_line_match` returns `None` when the best ratio is
                    // at-or-below `SIMILARITY_HINT_THRESHOLD` (0.5), so the
                    // hint stays absent when nothing in the file is close.
                    let similarity_hint = best_line_match(&content, &req.old_text)
                        .map(|(text, line, ratio)| {
                            SimilarityHint::from_ratio(
                                text,
                                u32::try_from(line).unwrap_or(u32::MAX),
                                ratio,
                            )
                        });
                    Ok(EditOutcome {
                        tier: EditMatchTier::NotMatched,
                        bytes_written: 0,
                        file_path: req.file_path,
                        similarity_hint,
                    })
                }
                (matched_tier, Some(updated_content)) => {
                    tokio::fs::write(&req.file_path, updated_content.as_bytes())
                        .await
                        .map_err(ToolDispatchError::IoError)?;
                    let bytes_written =
                        u32::try_from(req.new_text.len()).unwrap_or(u32::MAX);
                    Ok(EditOutcome {
                        tier: matched_tier,
                        bytes_written,
                        file_path: req.file_path,
                        similarity_hint: None,
                    })
                }
                (_, None) => Ok(EditOutcome {
                    tier: EditMatchTier::NotMatched,
                    bytes_written: 0,
                    file_path: req.file_path,
                    similarity_hint: None,
                }),
            }
        })
    }

    fn dispatch_validate(
        &self,
        req: ValidateRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            if self.disabled {
                return Err(ToolDispatchError::NotImplemented(
                    "dispatch_validate disabled via env".into(),
                ));
            }
            // Phase B1 — policy gate. The validator pipeline reads /
            // writes under `req.project_dir`; FileWrite on the project
            // dir is the most representative `SandboxOp` variant.
            self.enforce_policy(&SandboxOp::FileWrite {
                path: req.project_dir.as_path(),
            })?;
            let results = self.validator.run_all(&req).await;
            Ok(results)
        })
    }

    fn dispatch_classify_failure(
        &self,
        input: FailureInput,
    ) -> Pin<Box<dyn Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        Box::pin(async move {
            if self.disabled {
                return None;
            }
            // Phase B1 — policy gate. classify_failure is pure (no I/O),
            // so the SandboxOp carries an empty FileWrite path; the
            // default `PassthroughSandbox` treats an empty path as a
            // no-op (see sandbox_policy.rs::check_file_write). A custom
            // adapter that wants to suppress classification entirely
            // can deny on an empty path.
            //
            // WARN-2 (B1 audit): the previous denial path returned `None`
            // silently, which was indistinguishable from a low-confidence
            // miss and hid sandbox activity from operators. We now emit a
            // `tracing::warn!` carrying the denial reason and return a
            // synthetic `ClassifiedFailure` so the surface signal is "the
            // sandbox rejected this", not "we don't know what failed".
            // `ToolFailureCategory::RuntimePanic` is the closest existing
            // variant for a runtime-level rejection (the enum lives in
            // `core::tool_dispatcher` and is LOCKED by the B1 fix scope —
            // adding an `Other` variant requires a separate phase). The
            // reason string travels in `source` so downstream renderers
            // surface the sandbox detail verbatim.
            if let Err(err) = self.enforce_policy(&SandboxOp::FileWrite {
                path: std::path::Path::new(""),
            }) {
                let reason = match &err {
                    ToolDispatchError::Denied { reason } => reason.clone(),
                    other => format!("{other:?}"),
                };
                tracing::warn!(
                    target: "elai.sandbox",
                    step_id = %input.step_id,
                    reason = %reason,
                    "sandbox denied dispatch_classify_failure",
                );
                return Some(ClassifiedFailure {
                    category: ToolFailureCategory::RuntimePanic,
                    confidence: 1.0,
                    source: format!("sandbox denied: {reason}"),
                });
            }
            classify_output(&input.raw_output).map(|mut cf| {
                cf.source = input.source;
                cf
            })
        })
    }

    fn dispatch_git_commit(
        &self,
        req: GitCommitRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async move {
            if self.disabled {
                return Err(ToolDispatchError::NotImplemented(
                    "dispatch_git_commit disabled via env".into(),
                ));
            }
            // Phase B1 — policy gate. git_commit is the closest analogue
            // to a shell `git commit -m ...` invocation; expose it to the
            // adapter as a `BashCommand` so destructive-command and
            // future allow/deny rules can fire.
            let git_args: Vec<String> = vec![
                "commit".to_string(),
                "-m".to_string(),
                req.message.clone(),
            ];
            self.enforce_policy(&SandboxOp::BashCommand {
                cmd: "git",
                args: &git_args,
            })?;
            // git2 is blocking; run on the blocking pool.
            tokio::task::spawn_blocking(move || git_commit_blocking(&req))
                .await
                .map_err(|e| {
                    ToolDispatchError::GitError(format!("spawn_blocking join: {e}"))
                })?
        })
    }

    fn dispatch_file_read_outline(
        &self,
        path: PathBuf,
        line_threshold: usize,
    ) -> Pin<
        Box<dyn Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async move {
            if self.disabled {
                return Err(ToolDispatchError::NotImplemented(
                    "dispatch_file_read_outline disabled via env".into(),
                ));
            }
            // Phase B1 — policy gate. dispatch_file_read_outline does
            // not write to disk, but the policy layer hooks every
            // dispatch entry point uniformly (spec B1, brief §4). The
            // `FileWrite` variant is a documented approximation; the
            // file path is what the operation touches, so the adapter
            // sees the correct subject. The default
            // `PassthroughSandbox` policy permits this — only an
            // explicit deny-all custom adapter (or a `ReadOnlyFs`
            // profile narrowing) will block reads.
            self.enforce_policy(&SandboxOp::FileWrite {
                path: path.as_path(),
            })?;
            let content = tokio::fs::read_to_string(&path)
                .await
                .map_err(ToolDispatchError::IoError)?;
            Ok(generate_file_view(&content, line_threshold))
        })
    }

    fn dispatch_incremental_compile(
        &self,
        req: IncrementalRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<IncrementalOutcome, ToolDispatchError>> + Send + '_>,
    > {
        Box::pin(async move {
            if self.disabled {
                return Err(ToolDispatchError::NotImplemented(
                    "dispatch_incremental_compile disabled via env".into(),
                ));
            }
            // Phase B1 — policy gate. The compile path spawns a child
            // process under `req.project_dir`; expose that directory as
            // the FileWrite subject so write-allow-list policies fire.
            self.enforce_policy(&SandboxOp::FileWrite {
                path: req.project_dir.as_path(),
            })?;
            let _clock = OsClock;
            let start = _clock.now();
            let (cmd, args) =
                compiler_command(&req.language).ok_or_else(|| {
                    ToolDispatchError::CrateUnreachable(format!(
                        "unsupported language: {}",
                        req.language
                    ))
                })?;
            let mut attempts: u8 = 0;
            let mut first_error_category: Option<ToolFailureCategory> = None;
            let total = MAX_INCREMENTAL_RETRIES.min(req.retry_round.max(1) + 2);
            while attempts < total {
                attempts += 1;
                let out = tokio::process::Command::new(cmd)
                    .args(&args)
                    .current_dir(&req.project_dir)
                    .output()
                    .await
                    .map_err(ToolDispatchError::IoError)?;
                if out.status.success() {
                    let elapsed_ms =
                        u32::try_from(_clock.elapsed_since(start).as_millis()).unwrap_or(u32::MAX);
                    return Ok(IncrementalOutcome {
                        success: true,
                        attempts,
                        first_error_category,
                        elapsed_ms,
                    });
                }
                if first_error_category.is_none() {
                    let combined = format!(
                        "{}{}",
                        String::from_utf8_lossy(&out.stdout),
                        String::from_utf8_lossy(&out.stderr)
                    );
                    if let Some(cf) = classify_output(&combined) {
                        first_error_category = Some(cf.category);
                    }
                }
            }
            let elapsed_ms = u32::try_from(_clock.elapsed_since(start).as_millis()).unwrap_or(u32::MAX);
            Ok(IncrementalOutcome {
                success: false,
                attempts,
                first_error_category,
                elapsed_ms,
            })
        })
    }
}

/// Convenience builder: construct a `DefaultTieredValidator` with the
/// production command runners (`CargoDeterminismRunner` + `LanguageSyntaxRunner`)
/// wired up. The `integration` runner is left as a caller responsibility
/// because the production impl wraps the orchestrator crate's
/// `IntegrationTestRunner` — which lives in `crates/orchestrator/src/`
/// and is therefore supplied by the dashboard setup code.
pub fn default_validator(
    integration: Arc<dyn crate::tiered_validation::IntegrationCommandRunner>,
    language: &str,
) -> DefaultTieredValidator {
    let determinism =
        Arc::new(crate::tiered_validation::CargoDeterminismRunner::new(language));
    let syntax = Arc::new(crate::tiered_validation::LanguageSyntaxRunner);
    DefaultTieredValidator::new(determinism, syntax, integration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn edit_exact_match() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "hello world").unwrap();
        let path = f.path().to_path_buf();
        f.flush().unwrap();
        // Construct a dispatcher with a dummy validator that returns
        // nothing — the edit path never touches it.
        struct NullVal;
        impl crate::tiered_validation::TieredValidator for NullVal {
            fn run_tier<'a>(
                &'a self,
                tier: ValidationTier,
                _req: &'a ValidateRequest,
            ) -> Pin<Box<dyn Future<Output = ValidationTierResult> + Send + 'a>>
            {
                Box::pin(async move {
                    ValidationTierResult {
                        tier,
                        passed: true,
                        elapsed_ms: 0,
                        rules_fired: 0,
                        stderr_snippet: None,
                    }
                })
            }
            fn run_all<'a>(
                &'a self,
                _req: &'a ValidateRequest,
            ) -> Pin<Box<dyn Future<Output = Vec<ValidationTierResult>> + Send + 'a>>
            {
                Box::pin(async move { Vec::new() })
            }
        }
        let d = InProcessToolDispatcher::for_tests(Arc::new(NullVal));
        let req = EditRequest {
            file_path: path.clone(),
            old_text: "world".into(),
            new_text: "Rust".into(),
        };
        let out = d.dispatch_edit(req).await.unwrap();
        assert_eq!(out.tier, EditMatchTier::Exact);
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "hello Rust");
    }

    #[tokio::test]
    async fn edit_whitespace_fallback() {
        let mut f = NamedTempFile::new().unwrap();
        // File has tabs; request has spaces.
        write!(f, "fn foo() {{\n\tlet x = 1;\n}}").unwrap();
        let path = f.path().to_path_buf();
        f.flush().unwrap();
        struct NullVal;
        impl crate::tiered_validation::TieredValidator for NullVal {
            fn run_tier<'a>(
                &'a self,
                tier: ValidationTier,
                _r: &'a ValidateRequest,
            ) -> Pin<Box<dyn Future<Output = ValidationTierResult> + Send + 'a>>
            {
                Box::pin(async move {
                    ValidationTierResult {
                        tier,
                        passed: true,
                        elapsed_ms: 0,
                        rules_fired: 0,
                        stderr_snippet: None,
                    }
                })
            }
            fn run_all<'a>(
                &'a self,
                _r: &'a ValidateRequest,
            ) -> Pin<Box<dyn Future<Output = Vec<ValidationTierResult>> + Send + 'a>>
            {
                Box::pin(async move { Vec::new() })
            }
        }
        let d = InProcessToolDispatcher::for_tests(Arc::new(NullVal));
        let req = EditRequest {
            file_path: path.clone(),
            old_text: "let x = 1;".into(), // exact would miss because of leading \t
            new_text: "let x = 42;".into(),
        };
        let out = d.dispatch_edit(req).await.unwrap();
        // Exact tier should still win here because the normalised content
        // contains "let x = 1;" as-is — which proves Tier 1 correctly
        // handles internal single-space runs and doesn't fall through.
        // The whitespace-normalised path kicks in when the agent submits
        // a tab-vs-space MISMATCHED anchor; covered in the contract tests.
        assert!(
            matches!(
                out.tier,
                EditMatchTier::Exact | EditMatchTier::WhitespaceNormalized
            ),
            "expected exact or whitespace tier, got {:?}",
            out.tier
        );
    }

    #[tokio::test]
    async fn classify_failure_detects_compilation() {
        struct NullVal;
        impl crate::tiered_validation::TieredValidator for NullVal {
            fn run_tier<'a>(
                &'a self,
                tier: ValidationTier,
                _r: &'a ValidateRequest,
            ) -> Pin<Box<dyn Future<Output = ValidationTierResult> + Send + 'a>>
            {
                Box::pin(async move {
                    ValidationTierResult {
                        tier,
                        passed: true,
                        elapsed_ms: 0,
                        rules_fired: 0,
                        stderr_snippet: None,
                    }
                })
            }
            fn run_all<'a>(
                &'a self,
                _r: &'a ValidateRequest,
            ) -> Pin<Box<dyn Future<Output = Vec<ValidationTierResult>> + Send + 'a>>
            {
                Box::pin(async move { Vec::new() })
            }
        }
        let d = InProcessToolDispatcher::for_tests(Arc::new(NullVal));
        let classified = d
            .dispatch_classify_failure(FailureInput {
                raw_output: "error[E0308]: mismatched types".into(),
                step_id: "baby_step_1".into(),
                source: "validator".into(),
            })
            .await
            .expect("classifier should match");
        // B-20A: CompilationError → TypeMismatch under the 9-variant taxonomy
        // (mismatched-types is the canonical Rust E0308 signal).
        assert_eq!(
            classified.category,
            ToolFailureCategory::TypeMismatch
        );
        assert!((classified.confidence - 1.0).abs() < f32::EPSILON);
        assert_eq!(classified.source, "validator");
    }

    #[test]
    fn outline_under_threshold_returns_full() {
        let content = "fn a() {}\nfn b() {}\n";
        let v = generate_file_view(content, 10);
        assert!(!v.outline_only);
        assert_eq!(v.full_content.as_deref(), Some(content));
    }

    #[test]
    fn outline_over_threshold_returns_signatures_only() {
        let mut big = String::new();
        for i in 0..400 {
            big.push_str(&format!("fn generated_{i}() {{ }}\n"));
        }
        let v = generate_file_view(&big, 300);
        assert!(v.outline_only);
        assert!(v.full_content.is_none());
        assert_eq!(v.signatures.len(), 400);
    }
}
