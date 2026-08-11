//! Plain public data types of the chat REFLECT pipeline.
//!
//! Extracted from `chat_reflect.rs` so that file stays under the
//! maintainability line budget. These types are re-exported from
//! `chat_reflect` and from the crate root, so all existing import paths
//! (`elai_reasoning::ReflectOutput`, `chat_reflect::ReflectTokenSummary`,
//! etc.) continue to resolve unchanged.

use serde::{Deserialize, Serialize};

use crate::chat_reflect::TurnClass;

// ---------------------------------------------------------------------------
// ReflectOutput — public return shape of `ChatReflectPipeline::run()`
// ---------------------------------------------------------------------------

/// Per-stage token accounting. `total` is the integer sum of `stage_tokens`
/// (TRIAGE, REASONER aggregate across N candidates, AUDITOR, SYNTHESIZE).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectTokenSummary {
    /// `[triage, reasoner_total, auditor, synthesize]`. SYNTHESIZE is a local
    /// string select with zero inference cost — always 0.
    pub stage_tokens: [u32; 4],
    pub total: u32,
}

impl ReflectTokenSummary {
    pub(crate) fn from_stages(triage: u32, reasoner: u32, auditor: u32, synth: u32) -> Self {
        let stage_tokens = [triage, reasoner, auditor, synth];
        let total = stage_tokens.iter().copied().sum();
        Self {
            stage_tokens,
            total,
        }
    }
}

/// L-9: which grounding profile a `/reason` turn runs under. `Default` keeps
/// the shipped B-39 behaviour byte for byte. `Research` opts the turn into the
/// L-9 research grounding pass, which runs in the CLI turn path AFTER the
/// pipeline returns (slice 4), never inside this pipeline.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectProfile {
    #[default]
    Default,
    Research,
}

/// Full result of running the pipeline on a single chat turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectOutput {
    /// What the user sees. On the happy path, the AUDITOR's selected
    /// candidate; on `skipped = true`, the original user input verbatim.
    pub final_reply: String,
    pub turn_class: TurnClass,
    pub token_summary: ReflectTokenSummary,
    /// `true` when the pipeline fell back to passthrough (any error) or when
    /// TRIAGE classified the turn as `Chat` (short-circuit). The chat runtime
    /// must NOT depend on the pipeline succeeding.
    pub skipped: bool,
    /// Stable identifier for correlating SSB rows across stages.
    pub turn_id: String,
    /// L-9: the REASONER stage's N-best candidate texts, in emission order.
    /// Empty on every `skipped = true` path. The L-4 semantic-entropy predictor
    /// needs these as samples; before L-9 they only reached the event sink.
    #[serde(default)]
    pub candidates: Vec<String>,
}
