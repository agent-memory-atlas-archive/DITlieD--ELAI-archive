//! I-13: dual serialization for [`elai_ipc_types::ToolResultPayload`].
//!
//! Provides two views of a tool result:
//!   - `to_ui_dict()`:    full payload suitable for the dashboard (MemoryFactsDrawer,
//!                        DualSerialized badge, complete diagnostics list).
//!   - `to_llm_content()`: condensed payload for the agent context window. Long
//!                         `output` fields are truncated at [`LLM_OUTPUT_CHAR_LIMIT`]
//!                         with a trailing notice so the model can invoke
//!                         TraceQuery to expand a specific range on demand.
//!
//! The token delta (`to_ui_dict().len() - to_llm_content().len()`) is emitted
//! as `MemoryGuardEvent(DualSerialized { token_delta })` by the production
//! wiring in `production_agent_rpc.rs` (CHOKEPOINT: file_read arm, post-dispatch).
//!
//! # No .unwrap()/.expect() in public fns (CLAUDE.md hard rule)
//! All serialization uses fallback strings on serde failure rather than panicking.

use elai_ipc_types::ToolResultPayload;
use serde_json::json;

// ---------------------------------------------------------------------------
// Truncation threshold
// ---------------------------------------------------------------------------

/// Maximum character count for the `output` field in [`to_llm_content`].
///
/// 8 192 characters ≈ ~2 000 tokens (4 chars/token heuristic) — large enough
/// that most compile errors / file outlines fit, small enough that a 500-file
/// directory listing is always condensed (triggering TraceQuery demand).
pub const LLM_OUTPUT_CHAR_LIMIT: usize = 8_192;

// ---------------------------------------------------------------------------
// DualSerializable trait
// ---------------------------------------------------------------------------

/// Dual-serialization interface for tool results.
///
/// Implemented on [`ToolResultPayload`] so every tool result can produce both
/// a full (dashboard) and a condensed (LLM context) representation.
pub trait DualSerializable {
    /// Full JSON representation for the dashboard.
    ///
    /// Includes the complete `output` string and all `diagnostics`.
    /// Used by MemoryFactsDrawer and the DualSerialized SSB badge.
    fn to_ui_dict(&self) -> String;

    /// Condensed JSON representation for the agent context window.
    ///
    /// Truncates `output` at [`LLM_OUTPUT_CHAR_LIMIT`] characters and appends
    /// a notice so the model knows it can call TraceQuery to expand a range.
    /// Diagnostics are included in full (they are typically short).
    fn to_llm_content(&self) -> String;
}

impl DualSerializable for ToolResultPayload {
    fn to_ui_dict(&self) -> String {
        // Full payload: output + diagnostics as-is.
        serde_json::to_string(&json!({
            "output": self.output,
            "diagnostics": self.diagnostics,
        }))
        .unwrap_or_else(|_| {
            // Fallback: safe string representation (never panic in production).
            format!(
                r#"{{"output":{:?},"diagnostics":[]}}"#,
                &self.output[..self.output.len().min(256)]
            )
        })
    }

    fn to_llm_content(&self) -> String {
        let condensed_output = if self.output.len() > LLM_OUTPUT_CHAR_LIMIT {
            // Truncate and append a notice for the model.
            let truncated = &self.output[..LLM_OUTPUT_CHAR_LIMIT];
            format!(
                "{truncated}\n[truncated at {LLM_OUTPUT_CHAR_LIMIT} chars; use trace_query to expand a specific line range]"
            )
        } else {
            self.output.clone()
        };

        serde_json::to_string(&json!({
            "output": condensed_output,
            "diagnostics": self.diagnostics,
        }))
        .unwrap_or_else(|_| {
            format!(
                r#"{{"output":{:?},"diagnostics":[]}}"#,
                &condensed_output[..condensed_output.len().min(256)]
            )
        })
    }
}

// ---------------------------------------------------------------------------
// Token delta helper
// ---------------------------------------------------------------------------

/// Compute the character-delta between `to_ui_dict` and `to_llm_content`.
///
/// Used by `production_agent_rpc.rs` to populate `MemoryGuardEventPayload::token_delta`.
/// Returns 0 when `to_llm_content` is equal to or larger than `to_ui_dict`
/// (degenerate — never a real case for truncated output).
pub fn dual_serialize_delta(payload: &ToolResultPayload) -> u32 {
    let full_len = payload.to_ui_dict().len();
    let condensed_len = payload.to_llm_content().len();
    full_len.saturating_sub(condensed_len) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_ipc_types::ToolResultPayload;

    fn make_payload(output: &str) -> ToolResultPayload {
        ToolResultPayload {
            output: output.to_string(),
            diagnostics: vec!["diag1".to_string()],
        }
    }

    #[test]
    fn short_output_not_truncated() {
        let p = make_payload("hello world");
        let llm = p.to_llm_content();
        let ui = p.to_ui_dict();
        assert!(llm.contains("hello world"), "short output must appear in llm_content");
        assert!(ui.contains("hello world"), "short output must appear in ui_dict");
        assert_eq!(dual_serialize_delta(&p), 0, "no delta for short output");
    }

    #[test]
    fn long_output_truncated_in_llm_content() {
        // Generate output larger than LLM_OUTPUT_CHAR_LIMIT.
        let long_output = "x".repeat(LLM_OUTPUT_CHAR_LIMIT + 1_000);
        let p = make_payload(&long_output);

        let llm = p.to_llm_content();
        let ui = p.to_ui_dict();

        assert!(llm.len() < ui.len(), "condensed must be shorter than full");
        assert!(
            llm.contains("truncated"),
            "condensed must contain truncation notice"
        );
        assert!(
            !llm.contains(&long_output),
            "condensed must not contain full long output"
        );
        let delta = dual_serialize_delta(&p);
        assert!(delta > 0, "token delta must be positive for truncated output");
    }

    #[test]
    fn ui_dict_has_full_output() {
        let long_output = "y".repeat(LLM_OUTPUT_CHAR_LIMIT + 500);
        let p = make_payload(&long_output);
        let ui = p.to_ui_dict();
        // The full output must be present in ui_dict.
        assert!(ui.contains(&long_output), "ui_dict must contain the full output");
    }

    #[test]
    fn diagnostics_present_in_both() {
        let p = make_payload("output");
        let llm = p.to_llm_content();
        let ui = p.to_ui_dict();
        assert!(llm.contains("diag1"), "diagnostics must appear in llm_content");
        assert!(ui.contains("diag1"), "diagnostics must appear in ui_dict");
    }
}
