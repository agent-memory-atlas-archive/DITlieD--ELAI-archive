//! Fill-In-the-Middle (FIM) prompt assembly for the focus-tunnel path.
//!
//! Spec: workflow/papers/focus-tunnel-design-interface.md (HIGH-reliability).
//! Sentinel tokens and the `strip_fim_artifacts` cleaner are reused verbatim
//! from `scaffold::fim` so the byte ordering locked by
//! `dashboard_scaffold_phase_a_contract.rs` (prefix < suffix < middle) cannot
//! drift between the website-scaffold and orchestrator code paths.
//!
//! Budget invariant (spec L170-171):
//!   prefix_tokens + suffix_tokens + max_gen_tokens <= context_window
//!
//! When the inputs overflow, the suffix is truncated to at most 50 lines
//! (spec L171, "next 50 lines if file is large"). If the prefix alone already
//! exceeds the window, this is unrecoverable — the caller must fall back to
//! STRUCTURED mode rather than emit a partial prefix.

use std::cmp::min;

use scaffold::fim::{QWEN_FIM_MIDDLE, QWEN_FIM_PREFIX, QWEN_FIM_SUFFIX};
use thiserror::Error;

/// Hard upper bound on the truncated suffix line count. Matches
/// focus-tunnel-design.txt L171: "next 50 lines if file is large".
pub const FIM_SUFFIX_TRUNCATION_LINES: usize = 50;

/// Inputs to a single FIM prompt assembly.
#[derive(Debug, Clone)]
pub struct FimContext {
    /// Absolute or repo-relative path to the file being edited. Not embedded in
    /// the prompt (the model never specifies file paths in FIM mode — spec
    /// L120-124); carried for diagnostics + downstream splicing.
    pub file_path: String,
    /// Text that appears in the file BEFORE the edit region, as already
    /// assembled by the caller (header comments, imports, lines[..start]).
    pub prefix_text: String,
    /// Text that appears in the file AFTER the edit region (lines[end..]).
    pub suffix_text: String,
    /// Optional natural-language hint appended after `<|fim_middle|>`. The
    /// caller is responsible for trimming and sanitising; assemble emits the
    /// hint verbatim.
    pub hint_text: Option<String>,
    /// Tokenised cost of `prefix_text` (caller-supplied; the orchestrator does
    /// not tokenise here — the sidecar owns the tokenizer).
    pub prefix_tokens: usize,
    /// Tokenised cost of `suffix_text` (caller-supplied).
    pub suffix_tokens: usize,
    /// Caller's request for response length. Counted against the budget so the
    /// model has headroom to emit the middle.
    pub max_gen_tokens: usize,
    /// Total context window for the target model (e.g. 20_000 for Qwen2.5-Coder
    /// 3B at the 12 GB tier).
    pub context_window: usize,
}

/// Output of [`FimPromptAssembler::assemble`].
#[derive(Debug, Clone)]
pub struct FimPromptOutput {
    /// Assembled prompt body. Layout:
    ///   `<|fim_prefix|>{prefix}<|fim_suffix|>{suffix}<|fim_middle|>{hint?}`
    /// where `{hint?}` is empty when `FimContext::hint_text` is `None` or
    /// trims to empty.
    pub prompt: String,
    /// `true` when the suffix had to be truncated to satisfy the context
    /// budget. Surfaced so the caller can downgrade confidence / track the
    /// fallback rate in SSB.
    pub expected_suffix_truncated: bool,
}

/// Errors returned by [`FimPromptAssembler::assemble`].
#[derive(Debug, Error)]
pub enum FimPromptError {
    /// The prefix alone (plus the requested generation headroom) exceeds the
    /// model's context window. Suffix truncation cannot save this; the caller
    /// must fall back to STRUCTURED mode.
    #[error(
        "FIM context overflow: prefix_tokens={prefix_tokens} + max_gen_tokens={max_gen_tokens} \
         (={required}) exceeds context_window={context_window}"
    )]
    ContextOverflow {
        prefix_tokens: usize,
        max_gen_tokens: usize,
        context_window: usize,
        required: usize,
    },
}

/// Stateless assembler. Holds no I/O handles; safe to share/clone across
/// tasks. The `context_window` field is captured at construction so the
/// caller does not have to thread the model's window through every call.
#[derive(Debug, Clone)]
pub struct FimPromptAssembler {
    default_context_window: usize,
    suffix_truncation_lines: usize,
}

impl FimPromptAssembler {
    /// Build an assembler with the model's full context window pre-bound.
    /// `FimContext::context_window` overrides this on a per-call basis;
    /// callers that always use the same model can leave that field equal to
    /// `default_context_window`.
    pub fn new(default_context_window: usize) -> Self {
        Self {
            default_context_window,
            suffix_truncation_lines: FIM_SUFFIX_TRUNCATION_LINES,
        }
    }

    /// Default context window captured at construction.
    pub fn default_context_window(&self) -> usize {
        self.default_context_window
    }

    /// Assemble a Qwen-shaped FIM prompt under the spec budget invariant.
    pub fn assemble(&self, ctx: &FimContext) -> Result<FimPromptOutput, FimPromptError> {
        let window = if ctx.context_window == 0 {
            self.default_context_window
        } else {
            ctx.context_window
        };

        // Spec L170-171: prefix + suffix + max_gen <= context_window. If the
        // prefix alone (with generation headroom) already overflows, the
        // suffix cannot rescue us — surface a hard error.
        let unsavable = ctx
            .prefix_tokens
            .saturating_add(ctx.max_gen_tokens);
        if unsavable > window {
            return Err(FimPromptError::ContextOverflow {
                prefix_tokens: ctx.prefix_tokens,
                max_gen_tokens: ctx.max_gen_tokens,
                context_window: window,
                required: unsavable,
            });
        }

        let combined = unsavable.saturating_add(ctx.suffix_tokens);
        let (suffix_to_emit, truncated) = if combined > window {
            (
                Self::truncate_suffix(&ctx.suffix_text, self.suffix_truncation_lines),
                true,
            )
        } else {
            (ctx.suffix_text.clone(), false)
        };

        let hint = ctx
            .hint_text
            .as_deref()
            .map(str::trim)
            .filter(|h| !h.is_empty());
        let hint_len = hint.map(|h| h.len() + 8).unwrap_or(0);

        let mut prompt = String::with_capacity(
            QWEN_FIM_PREFIX.len()
                + ctx.prefix_text.len()
                + QWEN_FIM_SUFFIX.len()
                + suffix_to_emit.len()
                + QWEN_FIM_MIDDLE.len()
                + hint_len,
        );
        prompt.push_str(QWEN_FIM_PREFIX);
        prompt.push_str(&ctx.prefix_text);
        prompt.push_str(QWEN_FIM_SUFFIX);
        prompt.push_str(&suffix_to_emit);
        prompt.push_str(QWEN_FIM_MIDDLE);
        if let Some(h) = hint {
            prompt.push('\n');
            prompt.push_str("<!-- ");
            prompt.push_str(h);
            prompt.push_str(" -->");
        }

        Ok(FimPromptOutput {
            prompt,
            expected_suffix_truncated: truncated,
        })
    }

    fn truncate_suffix(suffix: &str, max_lines: usize) -> String {
        if max_lines == 0 {
            return String::new();
        }
        let cap = min(max_lines, suffix.lines().count());
        let mut out = String::with_capacity(suffix.len().min(cap * 80));
        for (i, line) in suffix.lines().take(cap).enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(line);
        }
        if suffix.ends_with('\n') && !out.ends_with('\n') {
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scaffold::fim::{QWEN_END_OF_TEXT, QWEN_FIM_PAD};

    fn ctx_basic() -> FimContext {
        FimContext {
            file_path: "src/lib.rs".to_string(),
            prefix_text: "fn before() {}\n".to_string(),
            suffix_text: "fn after() {}\n".to_string(),
            hint_text: None,
            prefix_tokens: 8,
            suffix_tokens: 8,
            max_gen_tokens: 64,
            context_window: 4096,
        }
    }

    #[test]
    fn happy_path_emits_three_sentinels_in_order() {
        let asm = FimPromptAssembler::new(4096);
        let out = asm.assemble(&ctx_basic()).expect("budget fits");
        let p = out.prompt.find(QWEN_FIM_PREFIX).unwrap();
        let s = out.prompt.find(QWEN_FIM_SUFFIX).unwrap();
        let m = out.prompt.find(QWEN_FIM_MIDDLE).unwrap();
        assert!(p < s && s < m, "ordering must be prefix < suffix < middle");
        assert!(!out.expected_suffix_truncated);
    }

    #[test]
    fn hint_appended_after_middle() {
        let asm = FimPromptAssembler::new(4096);
        let mut ctx = ctx_basic();
        ctx.hint_text = Some("  do the thing  ".to_string());
        let out = asm.assemble(&ctx).expect("budget fits");
        let m = out.prompt.find(QWEN_FIM_MIDDLE).unwrap();
        let after_middle = &out.prompt[m + QWEN_FIM_MIDDLE.len()..];
        assert!(after_middle.contains("do the thing"));
        assert!(!after_middle.contains("  do the thing  "), "hint trimmed");
    }

    #[test]
    fn empty_hint_omitted() {
        let asm = FimPromptAssembler::new(4096);
        let mut ctx = ctx_basic();
        ctx.hint_text = Some("   ".to_string());
        let out = asm.assemble(&ctx).expect("budget fits");
        assert!(out.prompt.ends_with(QWEN_FIM_MIDDLE));
    }

    #[test]
    fn oversize_triggers_truncation_not_error() {
        let asm = FimPromptAssembler::new(1024);
        let suffix: String = (0..500).map(|i| format!("line {i}\n")).collect();
        let ctx = FimContext {
            file_path: "src/big.rs".to_string(),
            prefix_text: "fn f() {}\n".to_string(),
            suffix_text: suffix,
            hint_text: None,
            prefix_tokens: 50,
            suffix_tokens: 2000,
            max_gen_tokens: 256,
            context_window: 1024,
        };
        let out = asm.assemble(&ctx).expect("truncation path");
        assert!(out.expected_suffix_truncated);
        let s = out.prompt.find(QWEN_FIM_SUFFIX).unwrap();
        let m = out.prompt.find(QWEN_FIM_MIDDLE).unwrap();
        let region = &out.prompt[s + QWEN_FIM_SUFFIX.len()..m];
        assert!(region.lines().count() <= FIM_SUFFIX_TRUNCATION_LINES);
    }

    #[test]
    fn unsavable_prefix_returns_error() {
        let asm = FimPromptAssembler::new(256);
        let ctx = FimContext {
            file_path: "src/x.rs".to_string(),
            prefix_text: "x".to_string(),
            suffix_text: "y".to_string(),
            hint_text: None,
            prefix_tokens: 300,
            suffix_tokens: 8,
            max_gen_tokens: 16,
            context_window: 256,
        };
        let err = asm.assemble(&ctx).unwrap_err();
        assert!(matches!(err, FimPromptError::ContextOverflow { .. }));
    }

    #[test]
    fn per_call_window_overrides_default() {
        let asm = FimPromptAssembler::new(1_000_000);
        let ctx = FimContext {
            file_path: "src/x.rs".to_string(),
            prefix_text: "x".to_string(),
            suffix_text: "y".to_string(),
            hint_text: None,
            prefix_tokens: 200,
            suffix_tokens: 8,
            max_gen_tokens: 16,
            context_window: 100, // overrides 1_000_000
        };
        let err = asm.assemble(&ctx).unwrap_err();
        assert!(matches!(err, FimPromptError::ContextOverflow { .. }));
    }

    #[test]
    fn strip_artifacts_round_trip_through_scaffold() {
        // The orchestrator does not own a stripper; it MUST reuse scaffold's.
        let raw = format!("middle{QWEN_FIM_PAD}{QWEN_END_OF_TEXT}");
        let cleaned = scaffold::fim::strip_fim_artifacts(&raw);
        assert_eq!(cleaned, "middle");
    }
}
