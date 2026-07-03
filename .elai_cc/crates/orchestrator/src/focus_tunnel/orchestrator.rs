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
    LanguageHint, ModeSelection, StreamingValidator, StreamingValidatorConfig, ValidationDecision,
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

    /// Streaming validator (B-28 E-4) terminated generation early. The
    /// caller routes to STRUCTURED + emits a `StreamingEarlyTerminated`
    /// SSB row using the carried wire-form reason + token count. This
    /// variant is the load-bearing signal that the
    /// [`StreamingValidator`] gate fired on the post-completion
    /// validation pass over the provider's raw output.
    #[error("focus-tunnel exec streaming early-term: reason={reason}, tokens={token_count}")]
    StreamingEarlyTerm {
        reason: String,
        token_count: u32,
    },
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

/// Infer the streaming-validator language hint from a file extension.
/// Returns [`LanguageHint::Generic`] for unknown extensions so the
/// validator still runs the brace-balance + OutputTooLong gates even
/// when the lazy-pattern gate is language-filtered out.
fn infer_language_hint(path: Option<&Path>) -> LanguageHint {
    let Some(path) = path else { return LanguageHint::Generic; };
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => LanguageHint::Rust,
        Some("py") => LanguageHint::Python,
        _ => LanguageHint::Generic,
    }
}

/// B-28 E-4 WIRING helper — drive a [`StreamingValidator`] over the
/// provider's raw completion text as an internal post-completion pass.
///
/// The `LlmProvider` trait (`elai_providers::LlmProvider`) exposes only
/// `complete` (returning `Vec<String>`); there is no live token stream
/// seam today. To honour the library-tier posture of E-4 (sync, no I/O)
/// while still giving [`StreamingValidator`] a production call site
/// reachable from `drive_task`, this helper feeds the raw output through
/// the validator token-by-token (split on whitespace boundaries, which
/// is the same granularity the sidecar SSE stream would emit), runs one
/// `on_chunk` pass for the tree-sitter + deprecated-import gates, then
/// finalizes with `on_complete`. An `EarlyTerm` decision surfaces as
/// [`FocusTunnelExecError::StreamingEarlyTerm`] so the caller falls back
/// to STRUCTURED and emits the `StreamingEarlyTerminated` SSB row.
///
/// `mode` is `"fim"` / `"diff"` / `"structured"` and is emitted with the
/// SSB row. `baby_step_id` is formatted into the validator's `step_id`
/// so the SSB row can be linked to its originating step.
fn validate_streaming_output(
    raw: &str,
    mode: &str,
    baby_step_id: u64,
    language: LanguageHint,
) -> Result<(), FocusTunnelExecError> {
    // Empty output short-circuits — the validator must not panic on an
    // empty stream (edge case (a) in the impact briefing).
    if raw.is_empty() {
        return Ok(());
    }
    let config = StreamingValidatorConfig {
        language,
        max_gen_tokens: DEFAULT_FIM_MAX_GEN_TOKENS as u32,
        step_id: format!("baby_step_{baby_step_id}"),
        mode: mode.to_string(),
        ..StreamingValidatorConfig::default()
    };
    let mut validator = StreamingValidator::new(config);
    let chunk_tokens = validator.chunk_tokens();
    let mut tokens_since_chunk: u32 = 0;
    let mut early_term: Option<(String, u32)> = None;
    for fragment in raw.split_whitespace() {
        let decision = validator.on_token(fragment);
        if let ValidationDecision::EarlyTerm { reason, token_count } = decision {
            early_term = Some((reason.ssb_wire_str().to_string(), token_count));
            break;
        }
        tokens_since_chunk = tokens_since_chunk.saturating_add(1);
        if tokens_since_chunk >= chunk_tokens {
            let decision = validator.on_chunk(raw);
            if let ValidationDecision::EarlyTerm { reason, token_count } = decision {
                early_term = Some((reason.ssb_wire_str().to_string(), token_count));
                break;
            }
            tokens_since_chunk = 0;
        }
    }
    // If the stream ended without an early-term, run a final chunk pass
    // over the full output so the deprecated-import + tree-sitter gates
    // get a chance to inspect the complete text.
    if early_term.is_none() {
        let decision = validator.on_chunk(raw);
        if let ValidationDecision::EarlyTerm { reason, token_count } = decision {
            early_term = Some((reason.ssb_wire_str().to_string(), token_count));
        }
    }
    match early_term {
        Some((reason, token_count)) => {
            // on_complete is not called here because the validator is
            // already in the terminated state; the captured reason is
            // surfaced directly. The caller's SSB emission uses the
            // wire-form reason + token_count.
            let _ = validator.on_complete();
            Err(FocusTunnelExecError::StreamingEarlyTerm { reason, token_count })
        }
        None => {
            // Natural completion — on_complete returns Ok for a clean
            // stream. We discard the Ok payload (total_tokens) since the
            // caller does not consume it today.
            let _ = validator.on_complete();
            Ok(())
        }
    }
}

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

    // Step 4b: B-28 E-4 WIRING — run the streaming validator over the
    // provider output as an internal detail. The LlmProvider trait
    // exposes only `complete` (no live token stream), so the validator
    // is driven post-completion: each whitespace-separated fragment is
    // fed to on_token, then a single on_chunk pass runs the tree-sitter
    // + deprecated-import gates, then on_complete. An EarlyTerm decision
    // surfaces as FocusTunnelExecError::StreamingEarlyTerm so the caller
    // falls back to STRUCTURED and emits the StreamingEarlyTerminated
    // SSB row. This is the production call site that makes
    // StreamingValidator reachable from drive_task when
    // ELAI_FOCUS_TUNNEL_EXECUTE=1.
    let lang = infer_language_hint(Some(target_file));
    validate_streaming_output(&raw, "fim", _ctx.baby_step_id, lang)?;

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

    // Step 2b: B-28 E-4 WIRING — streaming validator over the raw diff
    // output (same internal-detail pass as FIM). Catches lazy patterns
    // (todo!/pass/etc.) before the diff parser runs. EarlyTerm surfaces
    // as StreamingEarlyTerm so the caller falls back to STRUCTURED.
    // DIFF mode has no single target file (multi-region), so the
    // language hint defaults to Generic — the brace-balance +
    // OutputTooLong gates still run; language-filtered lazy patterns
    // are not checked on the DIFF path.
    validate_streaming_output(&raw, "diff", _ctx.baby_step_id, LanguageHint::Generic)?;

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
            session_credentials: std::collections::BTreeMap::new(),
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
            session_credentials: std::collections::BTreeMap::new(),
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
            session_credentials: std::collections::BTreeMap::new(),
        };
        let err = run_fim_step_with_provider(&ctx, &sel, None, None, None, None)
            .await
            .expect_err("no provider");
        assert!(matches!(err, FocusTunnelExecError::MissingProvider));
    }

    /// B-28 E-4 WIRING unit test — the streaming validator must be
    /// invoked inside `run_fim_step_with_provider` and surface a
    /// `StreamingEarlyTerm` error when the provider output contains a
    /// lazy pattern (`todo!()`). Uses a stub provider that returns the
    /// lazy token; the FIM region is valid so the only failure path is
    /// the validator gate. This pins the production call site that
    /// makes `StreamingValidator` reachable from `drive_task`.
    #[tokio::test]
    async fn fim_step_streaming_validator_terminates_on_lazy_pattern() {
        use elai_providers::{LlmFuture, LlmProvider, OllamaConfig};

        struct LazyProvider;
        impl LlmProvider for LazyProvider {
            fn complete<'a>(
                &'a self,
                _prompt: &'a str,
                _config: &'a OllamaConfig,
            ) -> LlmFuture<'a, Vec<String>> {
                Box::pin(async move { Ok(vec!["todo!()".to_string()]) })
            }
            fn embed<'a>(&'a self, _text: &'a str) -> LlmFuture<'a, Vec<f32>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn token_count(&self, _text: &str) -> Result<usize, elai_providers::ApiError> {
                Ok(0)
            }
            fn token_count_estimate(&self, text: &str) -> usize {
                text.len() / 4
            }
            fn model_name(&self) -> &str {
                "stub-lazy"
            }
        }

        let tmp = tempfile::tempdir().expect("tempdir");
        let file = tmp.path().join("lazy.rs");
        std::fs::write(&file, "fn before() {}\n// HERE\nfn after() {}\n").expect("seed file");

        let provider: Arc<dyn LlmProvider> = Arc::new(LazyProvider);
        let sel = dummy_selection(GenerationMode::Fim, SelectionRule::FimAllConditionsMet);
        let ctx = BabyStepExecutionCtx {
            baby_step_id: 77,
            task_summary: "lazy test".to_string(),
            escalation_message: None,
            mode_selection: Some(sel.clone()),
            session_credentials: std::collections::BTreeMap::new(),
        };
        let region = crate::focus_tunnel::EditRegion { start_line: 2, end_line: 3 };
        let err = run_fim_step_with_provider(
            &ctx,
            &sel,
            Some(provider),
            Some(&file),
            Some(region),
            None,
        )
        .await
        .expect_err("lazy pattern must early-term");
        assert!(
            matches!(err, FocusTunnelExecError::StreamingEarlyTerm { .. }),
            "expected StreamingEarlyTerm; got {err:?}"
        );
    }

    /// B-28 E-4 WIRING unit test — DIFF path: streaming validator runs
    /// but does NOT false-positive on a clean unified diff. The DIFF
    /// path uses `LanguageHint::Generic` (no single target file), so
    /// language-filtered lazy patterns are not checked; the
    /// brace-balance + OutputTooLong gates still run. A clean diff
    /// must pass through without `StreamingEarlyTerm`.
    #[tokio::test]
    async fn diff_step_streaming_validator_passes_clean_diff() {
        use elai_providers::{LlmFuture, LlmProvider, OllamaConfig};

        struct CleanDiffProvider;
        impl LlmProvider for CleanDiffProvider {
            fn complete<'a>(
                &'a self,
                _prompt: &'a str,
                _config: &'a OllamaConfig,
            ) -> LlmFuture<'a, Vec<String>> {
                let diff = "--- a/foo.rs\n+++ b/foo.rs\n@@ -1,1 +1,2 @@\n fn hello() {}\n+fn world() {}\n";
                Box::pin(async move { Ok(vec![diff.to_string()]) })
            }
            fn embed<'a>(&'a self, _text: &'a str) -> LlmFuture<'a, Vec<f32>> {
                Box::pin(async move { Ok(vec![]) })
            }
            fn token_count(&self, _text: &str) -> Result<usize, elai_providers::ApiError> {
                Ok(0)
            }
            fn token_count_estimate(&self, text: &str) -> usize {
                text.len() / 4
            }
            fn model_name(&self) -> &str {
                "stub-clean-diff"
            }
        }

        let provider: Arc<dyn LlmProvider> = Arc::new(CleanDiffProvider);
        let sel = dummy_selection(GenerationMode::Diff, SelectionRule::MultiRegionDiff);
        let ctx = BabyStepExecutionCtx {
            baby_step_id: 79,
            task_summary: "clean diff test".to_string(),
            escalation_message: None,
            mode_selection: Some(sel.clone()),
            session_credentials: std::collections::BTreeMap::new(),
        };
        // The DIFF path will return UnsupportedMode (no editor injected)
        // — but that happens AFTER the streaming validator runs. So the
        // error must NOT be StreamingEarlyTerm; it should be
        // UnsupportedMode, proving the validator passed the clean diff.
        let err = run_diff_step_with_provider(&ctx, &sel, Some(provider), None)
            .await
            .expect_err("no editor injected");
        assert!(
            !matches!(err, FocusTunnelExecError::StreamingEarlyTerm { .. }),
            "clean diff must not early-term; got {err:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Re-export the BufferEditor borrow seam (focus_tunnel::diff already exports
// the trait, but we re-export the alias path so tests using
// `orchestrator::*` discover it without an extra import).
// ---------------------------------------------------------------------------
