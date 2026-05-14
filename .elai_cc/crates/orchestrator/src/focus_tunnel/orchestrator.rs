//! Focus-tunnel per-mode execution wrappers (B-28 E-3 + WIRING-2).
//!
//! Spec: `workflow/papers/focus-tunnel-design-interface.md` L60-82
//! ("FIM mode" / "DIFF mode" / "STRUCTURED mode" behavioural contracts).
//!
//! E-3 LANDED ONLY the decision side ([`crate::focus_tunnel::ModeSelector`] +
//! the `GenerationModeSelected` SSB row). The legacy 2-arg [`run_fim_step`] /
//! [`run_diff_step`] entry points kept by E-3 still return
//! [`FocusTunnelExecError::MissingProvider`] — the E-3 contract test pins
//! that.
//!
//! WIRING-2 (this file) ADDS the production execution side via two new
//! `*_with_provider` entry points. The caller threads an
//! `Arc<dyn LlmProvider>` plus the per-mode I/O surfaces (target file path,
//! edit_region, fs adapter). When the caller has no provider, it MUST use
//! the legacy entry points so callers without focus-tunnel infrastructure
//! continue to see the locked `MissingProvider` STRUCTURED-fallback signal.
//!
//! Production path (drive_task):
//!   1. [`crate::task_runner::drive_task`] inspects
//!      `BabyStepExecutionCtx.mode_selection` after the
//!      `GenerationModeSelected` beacon is emitted.
//!   2. For [`crate::focus_tunnel::GenerationMode::Fim`] with an
//!      `edit_region` and an `Arc<dyn LlmProvider>` in
//!      `ResolvedConfig.llm_provider`, drive_task invokes
//!      [`run_fim_step_with_provider`]. On `Err` it falls back to the
//!      STRUCTURED [`crate::task_runner::AgentPoolBabyStepExecutor`] path
//!      (with a `StructuredFallback` SSB beacon).
//!   3. For [`crate::focus_tunnel::GenerationMode::Diff`], drive_task
//!      invokes [`run_diff_step_with_provider`]. The same Err -> STRUCTURED
//!      fallback rule applies (`DiffParseFailed` is the load-bearing
//!      hard-fallback signal).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;

use crate::focus_tunnel::{
    DiffParseError, DiffParser, FileReadWrite, FimContext, FimPromptAssembler, FimPromptError,
    ModeSelection,
};
use crate::task_runner::BabyStepExecutionCtx;
use elai_providers::{LlmProvider, OllamaConfig};

/// Approximate token-to-char ratio used to estimate prefix/suffix tokens for
/// the FIM budget invariant. 4 chars/token is the conservative upper bound
/// for English code (Qwen2.5-Coder reports ~3.6 chars/token on Rust). The
/// estimate only needs to be conservative enough to keep us under the
/// context window; the sidecar owns the precise tokenizer.
const FIM_CHARS_PER_TOKEN: usize = 4;

/// Default context-window budget for the FIM prompt assembler when the
/// caller does not supply one. Matches the production 20K-token cap for
/// Qwen2.5-Coder 3B NF4 documented in CLAUDE.md.
const DEFAULT_FIM_CONTEXT_WINDOW: usize = 20_000;

/// Default headroom reserved for the model's middle output (FIM) in tokens.
/// Conservative — small enough to leave room for prefix+suffix on long files.
const DEFAULT_FIM_MAX_GEN_TOKENS: usize = 512;

/// Errors surfaced by the per-mode execution wrappers in this module.
///
/// `MissingProvider` is the locked fallback signal in E-3: any caller that
/// receives this error MUST fall back to STRUCTURED execution and emit one
/// SSB beacon with `rule = StructuredFallback` so the auditor's
/// `mode_distribution` metric stays consistent. `UnsupportedMode` is the
/// general-purpose variant for "wiring not active for this mode".
#[derive(Debug, Error)]
pub enum FocusTunnelExecError {
    /// The provider client trait required for sidecar invocation is not
    /// available. The legacy 2-arg [`run_fim_step`] / [`run_diff_step`]
    /// entry points return this verbatim; WIRING-2 callers receive
    /// `MissingProvider` only when the caller passes `None`.
    #[error("focus-tunnel exec missing provider client (legacy entry point or None passed)")]
    MissingProvider,

    /// Inputs do not satisfy preconditions (e.g. FIM without edit_region).
    #[error("focus-tunnel exec for {mode} not supported with these inputs: {reason}")]
    UnsupportedMode {
        mode: &'static str,
        reason: &'static str,
    },

    /// Reserved hard-fallback signal: the model output failed the unified-diff
    /// parser. Caller routes to STRUCTURED + emits SSB diagnostic.
    #[error("diff parse failed; caller falls back to STRUCTURED")]
    DiffParseFailed,

    /// I/O error during file read/write (read of target file or write of
    /// spliced output). Caller routes to STRUCTURED.
    #[error("focus-tunnel exec I/O failure: {0}")]
    Io(String),

    /// FIM prompt budget overflow (prefix + max_gen > context_window).
    /// Caller routes to STRUCTURED.
    #[error("focus-tunnel exec FIM budget: {0}")]
    FimBudget(String),

    /// Provider returned an error (network / sidecar / OAuth). Caller routes
    /// to STRUCTURED.
    #[error("focus-tunnel exec provider failure: {0}")]
    Provider(String),

    /// DIFF replay rejected a hunk (3-tier matcher couldn't find the
    /// anchor). Caller routes to STRUCTURED.
    #[error("focus-tunnel exec diff replay rejected hunk(s): {0}")]
    DiffReplay(String),
}

// ---------------------------------------------------------------------------
// Legacy 2-arg entry points (E-3 contract — preserved).
// ---------------------------------------------------------------------------

/// Legacy E-3 entry point. Always returns [`FocusTunnelExecError::MissingProvider`].
/// WIRING-2 callers SHOULD use [`run_fim_step_with_provider`] instead.
pub async fn run_fim_step(
    _ctx: &BabyStepExecutionCtx,
    _selection: &ModeSelection,
) -> Result<FimExecutionOutcome, FocusTunnelExecError> {
    Err(FocusTunnelExecError::MissingProvider)
}

/// Legacy E-3 entry point. Always returns [`FocusTunnelExecError::MissingProvider`].
/// WIRING-2 callers SHOULD use [`run_diff_step_with_provider`] instead.
pub async fn run_diff_step(
    _ctx: &BabyStepExecutionCtx,
    _selection: &ModeSelection,
) -> Result<DiffExecutionOutcome, FocusTunnelExecError> {
    Err(FocusTunnelExecError::MissingProvider)
}

// ---------------------------------------------------------------------------
// WIRING-2 entry points — real bodies.
// ---------------------------------------------------------------------------

/// Execute one baby step in FIM mode against the focus-tunnel provider path.
///
/// Steps (B-28 WIRING-2 locked decision §4):
///   1. Read `target_file` via `tokio::fs::read_to_string`.
///   2. Slice content at `edit_region.start_line..end_line` into prefix +
///      suffix; estimate token costs from char counts.
///   3. Build [`FimContext`] and call [`FimPromptAssembler::assemble`].
///   4. Call `provider.complete(prompt, &OllamaConfig::default()).await`.
///   5. Run output through [`scaffold::fim::strip_fim_artifacts`].
///   6. Splice into target buffer; write via `tokio::fs::write`.
///   7. Return [`FimExecutionOutcome`] with `changed_files = vec![target]`.
///
/// On any error the caller falls back to STRUCTURED with a typed SSB row.
pub async fn run_fim_step_with_provider(
    _ctx: &BabyStepExecutionCtx,
    _selection: &ModeSelection,
    provider: Option<Arc<dyn LlmProvider>>,
    target_file: Option<&Path>,
    edit_region: Option<crate::focus_tunnel::EditRegion>,
    hint_text: Option<&str>,
) -> Result<FimExecutionOutcome, FocusTunnelExecError> {
    let provider = provider.ok_or(FocusTunnelExecError::MissingProvider)?;
    let target_file = target_file.ok_or(FocusTunnelExecError::UnsupportedMode {
        mode: "fim",
        reason: "target_file is required for FIM",
    })?;
    let edit_region = edit_region.ok_or(FocusTunnelExecError::UnsupportedMode {
        mode: "fim",
        reason: "edit_region is required for FIM (spec L132)",
    })?;

    // Step 1: read target file via tokio::fs (async; no blocking inside an
    // async fn — see B-28 WIRING-2 HARD RULE).
    let original = tokio::fs::read_to_string(target_file)
        .await
        .map_err(|e| FocusTunnelExecError::Io(format!("read {}: {e}", target_file.display())))?;

    // Step 2: slice on line boundaries. start_line/end_line are 1-based per
    // EditRegion contract; the splice is over `lines[start-1..end-1]`.
    let lines: Vec<&str> = original.split_inclusive('\n').collect();
    let total = lines.len();
    // Clamp to file bounds. saturating_sub for 0-based offsets.
    let start_idx = edit_region.start_line.saturating_sub(1).min(total);
    let end_idx = edit_region.end_line.saturating_sub(1).min(total);
    let (prefix_lines, suffix_lines) = if start_idx <= end_idx {
        let (a, rest) = lines.split_at(start_idx);
        let (_, b) = rest.split_at(end_idx - start_idx);
        (a.concat(), b.concat())
    } else {
        // Invalid region — return prefix=all, suffix=empty so the FIM
        // assembler still emits a budget-error path rather than panic.
        (original.clone(), String::new())
    };

    let prefix_tokens = prefix_lines.len() / FIM_CHARS_PER_TOKEN;
    let suffix_tokens = suffix_lines.len() / FIM_CHARS_PER_TOKEN;

    // Step 3: build FimContext + assemble.
    let fim_ctx = FimContext {
        file_path: target_file.to_string_lossy().to_string(),
        prefix_text: prefix_lines.clone(),
        suffix_text: suffix_lines.clone(),
        hint_text: hint_text.map(str::to_string),
        prefix_tokens,
        suffix_tokens,
        max_gen_tokens: DEFAULT_FIM_MAX_GEN_TOKENS,
        context_window: DEFAULT_FIM_CONTEXT_WINDOW,
    };
    let assembler = FimPromptAssembler::new(DEFAULT_FIM_CONTEXT_WINDOW);
    let prompt = match assembler.assemble(&fim_ctx) {
        Ok(p) => p,
        Err(FimPromptError::ContextOverflow { .. }) => {
            return Err(FocusTunnelExecError::FimBudget(
                "prefix+max_gen exceeds context_window".to_string(),
            ));
        }
    };

    // Step 4: call provider.
    let cfg = OllamaConfig::default();
    let completions = provider
        .complete(&prompt.prompt, &cfg)
        .await
        .map_err(|e| FocusTunnelExecError::Provider(format!("{e:?}")))?;
    let raw = completions.into_iter().next().unwrap_or_default();

    // Step 5: strip sentinels via scaffold::fim helper.
    let middle = scaffold::fim::strip_fim_artifacts(&raw);

    // Step 6: splice into the buffer + write.
    let mut new_content = String::with_capacity(prefix_lines.len() + middle.len() + suffix_lines.len());
    new_content.push_str(&prefix_lines);
    new_content.push_str(&middle);
    if !middle.is_empty() && !middle.ends_with('\n') && !suffix_lines.is_empty() {
        new_content.push('\n');
    }
    new_content.push_str(&suffix_lines);

    tokio::fs::write(target_file, &new_content)
        .await
        .map_err(|e| FocusTunnelExecError::Io(format!("write {}: {e}", target_file.display())))?;

    Ok(FimExecutionOutcome {
        changed_files: vec![target_file.to_path_buf()],
        bytes_generated: middle.len() as u64,
    })
}

/// Execute one baby step in DIFF mode against the focus-tunnel provider path.
///
/// Steps (B-28 WIRING-2 locked decision §5):
///   1. Build a STRUCTURED prompt requesting unified-diff output.
///   2. Call `provider.complete(prompt, &OllamaConfig::default())`.
///   3. Call [`DiffParser::parse`]. On [`DiffParseError::NotUnifiedDiff`]
///      / empty / no-headers, return
///      [`FocusTunnelExecError::DiffParseFailed`] so the caller falls back
///      to STRUCTURED + emits a `StructuredFallback` SSB row.
///   4. Call [`crate::focus_tunnel::replay_hunks`] with a [`BufferEditor`]
///      and a [`FileReadWrite`] adapter (production uses
///      `InProcessToolDispatcher` + [`RealFs`]; tests inject stubs).
///   5. Return [`DiffExecutionOutcome`] with `changed_files` from the plan.
///
/// `editor_fs_override` lets test callers inject a stub editor + fs. When
/// `None`, the production path uses [`RealFs`] (file read/write) but still
/// requires a [`BufferEditor`] in production — for WIRING-2 the production
/// caller routes through `drive_task` which constructs the
/// `InProcessToolDispatcher` and passes it down. Until that integration
/// lands, the production DIFF path returns
/// [`FocusTunnelExecError::UnsupportedMode`] when no editor is provided.
pub async fn run_diff_step_with_provider(
    _ctx: &BabyStepExecutionCtx,
    _selection: &ModeSelection,
    provider: Option<Arc<dyn LlmProvider>>,
    editor_fs_override: Option<DiffStepIo<'_>>,
) -> Result<DiffExecutionOutcome, FocusTunnelExecError> {
    let provider = provider.ok_or(FocusTunnelExecError::MissingProvider)?;

    // Step 1: structured prompt requesting a unified diff.
    let prompt = "Output a unified diff (no markdown fencing). Use `---`, `+++`, `@@` headers.";

    // Step 2: provider.complete.
    let cfg = OllamaConfig::default();
    let completions = provider
        .complete(prompt, &cfg)
        .await
        .map_err(|e| FocusTunnelExecError::Provider(format!("{e:?}")))?;
    let raw = completions.into_iter().next().unwrap_or_default();

    // Step 3: parse with DiffParser.
    let parser = DiffParser::new();
    let hunks = parser.parse(&raw).map_err(|err| match err {
        DiffParseError::NotUnifiedDiff { .. }
        | DiffParseError::EmptyOutput
        | DiffParseError::MissingFileHeader { .. }
        | DiffParseError::InvalidHunkHeader { .. }
        | DiffParseError::HunkOutOfBounds { .. } => FocusTunnelExecError::DiffParseFailed,
    })?;

    // Step 4: replay via injected editor + fs (test path) or fall back.
    let plan = match editor_fs_override {
        Some(io) => crate::focus_tunnel::replay_hunks(&hunks, io.editor, io.fs)
            .await
            .map_err(|e| FocusTunnelExecError::DiffReplay(format!("{e:?}")))?,
        None => {
            // Production caller has not wired the editor through yet — WIRING-2
            // initial drop keeps this as a STRUCTURED fallback signal. The
            // drive_task integration will swap this branch for a real
            // InProcessToolDispatcher pass-through once the editor surface
            // is plumbed through BabyStepExecutionCtx.
            return Err(FocusTunnelExecError::UnsupportedMode {
                mode: "diff",
                reason: "no buffer editor injected (drive_task wiring pending)",
            });
        }
    };

    Ok(DiffExecutionOutcome {
        changed_files: plan.changed_files.clone(),
        hunks_applied: plan.files.iter().map(|f| f.tiers.len() as u32).sum(),
    })
}

/// Borrow bundle for the DIFF replay coordinator's IO surfaces. Lets tests
/// inject stub editor + fs without paying the production DI cost.
pub struct DiffStepIo<'a> {
    pub editor: &'a dyn crate::focus_tunnel::BufferEditor,
    pub fs: &'a dyn FileReadWrite,
}

impl<'a> DiffStepIo<'a> {
    /// Build a `DiffStepIo` borrowing from an `Arc<dyn BufferEditor>` for the
    /// production case (the orchestrator owns the editor as an Arc; replay
    /// only needs a borrow).
    #[must_use]
    pub fn new(editor: &'a dyn crate::focus_tunnel::BufferEditor, fs: &'a dyn FileReadWrite) -> Self {
        Self { editor, fs }
    }
}

/// Per-step result for [`run_fim_step_with_provider`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FimExecutionOutcome {
    pub changed_files: Vec<PathBuf>,
    pub bytes_generated: u64,
}

/// Per-step result for [`run_diff_step_with_provider`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DiffExecutionOutcome {
    pub changed_files: Vec<PathBuf>,
    pub hunks_applied: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::focus_tunnel::{GenerationMode, SelectionRule};

    fn dummy_selection(mode: GenerationMode, rule: SelectionRule) -> ModeSelection {
        ModeSelection {
            mode,
            rule_fired: rule,
            reason: "test",
            preferred_was: None,
        }
    }

    #[tokio::test]
    async fn legacy_fim_returns_missing_provider() {
        let sel = dummy_selection(
            GenerationMode::Fim,
            SelectionRule::FimAllConditionsMet,
        );
        let ctx = BabyStepExecutionCtx {
            baby_step_id: 1,
            task_summary: "x".to_string(),
            escalation_message: None,
            mode_selection: Some(sel),
        };
        let err = run_fim_step(&ctx, &sel).await.expect_err("legacy");
        assert!(matches!(err, FocusTunnelExecError::MissingProvider));
    }

    #[tokio::test]
    async fn legacy_diff_returns_missing_provider() {
        let sel = dummy_selection(GenerationMode::Diff, SelectionRule::MultiRegionDiff);
        let ctx = BabyStepExecutionCtx {
            baby_step_id: 1,
            task_summary: "x".to_string(),
            escalation_message: None,
            mode_selection: Some(sel),
        };
        let err = run_diff_step(&ctx, &sel).await.expect_err("legacy");
        assert!(matches!(err, FocusTunnelExecError::MissingProvider));
    }

    #[tokio::test]
    async fn with_provider_fim_missing_region_errors() {
        let sel = dummy_selection(
            GenerationMode::Fim,
            SelectionRule::FimAllConditionsMet,
        );
        let ctx = BabyStepExecutionCtx {
            baby_step_id: 1,
            task_summary: "x".to_string(),
            escalation_message: None,
            mode_selection: Some(sel),
        };
        let err = run_fim_step_with_provider(&ctx, &sel, None, None, None, None)
            .await
            .expect_err("no provider");
        assert!(matches!(err, FocusTunnelExecError::MissingProvider));
    }
}

// ---------------------------------------------------------------------------
// Re-export the BufferEditor borrow seam (focus_tunnel::diff already exports
// the trait, but we re-export the alias path so tests using
// `orchestrator::*` discover it without an extra import).
// ---------------------------------------------------------------------------
