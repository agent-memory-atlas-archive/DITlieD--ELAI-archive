//! E-6 — persona-reanchor: midpoint role-prompt + last-finding re-injection.
//!
//! ## Why
//!
//! Multi-turn agent runs exhibit monotonic persona drift — measured by
//! Echoing (arXiv:2511.09710), Identity Drift (arXiv:2412.00804), and
//! Agent Drift / ASI (arXiv:2601.04170). Echoing onset mean turn 7.6;
//! risk escalates past turn 7. Structured-output mitigation lowers
//! GPT/Sonnet echoing rates from 55-70% down to <10%. The corresponding
//! ELAI mitigation: every N turns (default 20) re-inject the role
//! prompt + the last successful FindingReport as the first message of
//! the agent's next turn.
//!
//! ## Drift metric
//!
//! Two gates fire in parallel:
//!   1. Turn-counter — turns since last anchor `>= N`. Deterministic,
//!      always-on, no model dependency.
//!   2. Cosine-drift — cosine similarity between the current output
//!      embedding and the role-prompt embedding `< COSINE_THRESHOLD`
//!      (default 0.55, UNCONFIRMED, calibration RUNPOD-PENDING).
//!      Requires an [`EmbedProvider`] injection; fallback to (1) only
//!      when provider is None.
//!
//! See plan §Open questions: cosine threshold 0.55 needs empirical
//! calibration across 50+ ELAI sessions. The anchor still emits the
//! `cosine_similarity` value in the SSB row so calibration data
//! accumulates post-ship.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::ssb::{PersonaReanchoredPayload, Ssb, SsbError, SsbPayload};
use elai_ipc_types::AgentRole;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Per-session persona-anchor knobs. Defaults match the plan; env vars
/// `ELAI_PERSONA_ANCHOR_TURNS` / `ELAI_PERSONA_ANCHOR_COSINE` override at
/// construction time (caller resolves env, not this struct).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonaAnchorConfig {
    /// Re-anchor every `anchor_every_n_turns` turns regardless of
    /// embedding state. Default 20 per plan.
    pub anchor_every_n_turns: u32,
    /// Cosine-similarity threshold below which the anchor fires
    /// regardless of turn count. Default 0.55 (UNCONFIRMED).
    pub cosine_threshold: f64,
}

impl Default for PersonaAnchorConfig {
    fn default() -> Self {
        Self {
            anchor_every_n_turns: 20,
            cosine_threshold: 0.55,
        }
    }
}

// ---------------------------------------------------------------------------
// EmbedProvider trait — pluggable embedding backend
// ---------------------------------------------------------------------------

/// Pluggable embedding backend. Concrete implementor (e.g. fastembed)
/// lives outside this module so the orchestrator does not take a hard
/// dep on a specific embedding crate. When `None` is passed to
/// [`PersonaAnchor`], the cosine-drift gate degrades to "turn-counter
/// only" and the SSB row carries `embedding_available=false` so the
/// dashboard reflects the degraded mode.
///
/// Native async-fn-in-trait per Anti-Stale-Training. `Send + Sync` so
/// the provider can cross await points + multiple tokio threads.
pub trait EmbedProvider: Send + Sync {
    /// Embed `text` into a fixed-dim vector. Producers must use a
    /// consistent embedding model across calls so cosine comparison
    /// is meaningful. Returns the embedding as `Vec<f32>` — bounded
    /// length is the implementor's concern.
    fn embed<'a>(
        &'a self,
        text: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<f32>, PersonaAnchorError>> + Send + 'a>>;
}

// ---------------------------------------------------------------------------
// PersonaAnchor — the controller
// ---------------------------------------------------------------------------

/// Per-agent persona-anchor state. Construct once per agent run; call
/// [`tick`] before each agent turn to test for re-anchor.
pub struct PersonaAnchor {
    config: PersonaAnchorConfig,
    embed: Option<Arc<dyn EmbedProvider>>,
    /// Cached role-prompt embedding. Computed once on first `tick` when
    /// an EmbedProvider is present.
    role_prompt_embedding: Option<Vec<f32>>,
    /// Cached role prompt (raw text). Stored so the embedding can be
    /// (re-)computed lazily on first `tick`.
    role_prompt: String,
    /// Turn counter — incremented each `tick`.
    turn_count: u32,
    /// Turn count at which the most recent anchor fired. `0` when no
    /// anchor has fired yet.
    last_anchor_turn: u32,
}

impl PersonaAnchor {
    /// Construct a new anchor for the given agent role + role prompt.
    /// The `embed` provider is optional — None degrades the
    /// cosine-drift gate to no-op (turn-counter only).
    #[must_use]
    pub fn new(
        config: PersonaAnchorConfig,
        embed: Option<Arc<dyn EmbedProvider>>,
        role_prompt: impl Into<String>,
    ) -> Self {
        Self {
            config,
            embed,
            role_prompt_embedding: None,
            role_prompt: role_prompt.into(),
            turn_count: 0,
            last_anchor_turn: 0,
        }
    }

    /// Test for re-anchor after this turn. Returns the outcome describing
    /// whether and why the anchor fired. The caller is responsible for
    /// (a) actually injecting the role prompt + last FindingReport as the
    /// next turn's first message and (b) calling [`emit_ssb_row`] with
    /// the outcome so the dashboard sees the event.
    ///
    /// `current_output` is the agent's most recent reply (used for cosine
    /// drift). May be empty — cosine comparison short-circuits to None.
    pub async fn tick(&mut self, current_output: &str) -> PersonaAnchorOutcome {
        self.turn_count = self.turn_count.saturating_add(1);

        // Gate 1: turn counter
        let turn_gate_fired =
            self.turn_count.saturating_sub(self.last_anchor_turn) >= self.config.anchor_every_n_turns;

        // Gate 2: cosine drift (only if EmbedProvider present + current_output non-empty)
        let (cosine_value, cosine_gate_fired, embedding_available) = self
            .check_cosine_drift(current_output)
            .await;

        let fired = turn_gate_fired || cosine_gate_fired;
        if fired {
            self.last_anchor_turn = self.turn_count;
        }

        PersonaAnchorOutcome {
            fired,
            reason: if turn_gate_fired {
                AnchorReason::TurnCounter
            } else if cosine_gate_fired {
                AnchorReason::CosineDrift
            } else {
                AnchorReason::NoTrigger
            },
            turn_count: self.turn_count,
            cosine_similarity: cosine_value,
            embedding_available,
        }
    }

    /// Compute cosine similarity between cached role-prompt embedding and
    /// `current_output` embedding. Returns
    /// `(Some(cos), cos < threshold, true)` when an EmbedProvider is
    /// available + computation succeeds; `(None, false, false)`
    /// otherwise (degraded mode).
    async fn check_cosine_drift(
        &mut self,
        current_output: &str,
    ) -> (Option<f64>, bool, bool) {
        let provider = match &self.embed {
            Some(p) => p.clone(),
            None => return (None, false, false),
        };
        if current_output.is_empty() {
            return (None, false, true);
        }
        // Lazy-cache role-prompt embedding on first call.
        if self.role_prompt_embedding.is_none() {
            match provider.embed(&self.role_prompt).await {
                Ok(v) => self.role_prompt_embedding = Some(v),
                Err(_) => return (None, false, false),
            }
        }
        let role_emb = match &self.role_prompt_embedding {
            Some(v) => v.clone(),
            None => return (None, false, false),
        };
        let cur_emb = match provider.embed(current_output).await {
            Ok(v) => v,
            Err(_) => return (None, false, false),
        };
        let cos = cosine_similarity(&role_emb, &cur_emb);
        let fired = cos < self.config.cosine_threshold;
        (Some(cos), fired, true)
    }

    /// Current turn count (for tests / instrumentation).
    #[must_use]
    pub const fn turn_count(&self) -> u32 {
        self.turn_count
    }
}

/// Emit a `PersonaReanchored` SSB row from a [`PersonaAnchorOutcome`].
/// Separate from `tick` so the caller can choose to emit only when the
/// outcome `fired` and the application logic actually applied the
/// anchor.
pub async fn emit_ssb_row(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    agent_role: AgentRole,
    outcome: &PersonaAnchorOutcome,
) -> Result<i64, SsbError> {
    let payload = SsbPayload::PersonaReanchored(PersonaReanchoredPayload {
        agent_role: agent_role_as_str(agent_role).to_owned(),
        turn_count: outcome.turn_count,
        reason: outcome.reason.as_str().to_owned(),
        cosine_similarity: outcome.cosine_similarity,
        embedding_available: outcome.embedding_available,
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new())
        .await
}

fn agent_role_as_str(role: AgentRole) -> &'static str {
    match role {
        AgentRole::Executor => "executor",
        AgentRole::Auditor => "auditor",
        AgentRole::Researcher => "researcher",
        AgentRole::Designer => "designer",
        AgentRole::PromptEngineer => "prompt_engineer",
        AgentRole::ContextCompiler => "context_compiler",
        AgentRole::MemoryWriter => "memory_writer",
        AgentRole::MediaValidator => "media_validator",
        AgentRole::IdeaFramer => "idea_framer",
        AgentRole::IdeaCritic => "idea_critic",
        AgentRole::AuditorConcepts => "auditor_concepts",
        AgentRole::AuditorPhases => "auditor_phases",
        AgentRole::AuditorSteps => "auditor_steps",
        AgentRole::AuditorBabySteps => "auditor_baby_steps",
        AgentRole::AuditorBabyStepDetail => "auditor_baby_step_detail",
        AgentRole::AuditorProductionGrade => "auditor_production_grade",
        AgentRole::InjectionClassifier => "injection_classifier",
    }
}

// ---------------------------------------------------------------------------
// PersonaAnchorOutcome / AnchorReason
// ---------------------------------------------------------------------------

/// Outcome of a single [`PersonaAnchor::tick`] call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonaAnchorOutcome {
    /// True iff at least one gate fired. Caller injects the anchor
    /// message at the next turn boundary when this is true.
    pub fired: bool,
    /// Which gate fired (or NoTrigger when none did).
    pub reason: AnchorReason,
    /// Current turn count after this tick.
    pub turn_count: u32,
    /// Cosine similarity between role-prompt and current output. `None`
    /// when no EmbedProvider was injected or the embedding call failed.
    pub cosine_similarity: Option<f64>,
    /// True iff an EmbedProvider was available + embedding computation
    /// succeeded. Surfaces the degraded "turn-counter only" mode on
    /// the dashboard.
    pub embedding_available: bool,
}

/// Reason the anchor fired (or did not).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorReason {
    /// Turn-counter gate hit (>= anchor_every_n_turns since last anchor).
    TurnCounter,
    /// Cosine-drift gate hit (cosine < threshold).
    CosineDrift,
    /// Neither gate fired — anchor was NOT applied.
    NoTrigger,
}

impl AnchorReason {
    /// Wire-stable label for the SSB row.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TurnCounter => "turn_counter",
            Self::CosineDrift => "cosine_drift",
            Self::NoTrigger => "no_trigger",
        }
    }
}

// ---------------------------------------------------------------------------
// PersonaAnchorError
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PersonaAnchorError {
    /// Embedding provider failed (typically backend offline or network
    /// error). Caller falls back to turn-counter-only.
    #[error("embed provider failed: {0}")]
    EmbedFailed(String),
}

// ---------------------------------------------------------------------------
// cosine_similarity — pure helper
// ---------------------------------------------------------------------------

/// Cosine similarity between two equal-length vectors. Returns 0.0 if
/// either vector is empty or zero-norm. Length mismatch returns 0.0
/// (defensive: caller should not pass mismatched embeddings, but a
/// silent 0 avoids a panic deep in the agent loop).
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot: f64 = 0.0;
    let mut na: f64 = 0.0;
    let mut nb: f64 = 0.0;
    for i in 0..a.len() {
        let ai = a[i] as f64;
        let bi = b[i] as f64;
        dot += ai * bi;
        na += ai * ai;
        nb += bi * bi;
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn anchor_fires_at_default_turn_count_no_embed() {
        let cfg = PersonaAnchorConfig::default();
        let mut a = PersonaAnchor::new(cfg, None, "you are an executor");
        for t in 1..=19 {
            let out = a.tick(&format!("turn {t} output")).await;
            assert!(!out.fired, "turn {t} should NOT fire (no embed, turn < 20)");
        }
        let out = a.tick("turn 20 output").await;
        assert!(out.fired, "turn 20 must fire turn-counter gate");
        assert_eq!(out.reason, AnchorReason::TurnCounter);
        assert!(!out.embedding_available, "no embed provider was supplied");
    }

    #[tokio::test]
    async fn custom_n_fires_at_configured_turn() {
        let mut a = PersonaAnchor::new(
            PersonaAnchorConfig {
                anchor_every_n_turns: 5,
                cosine_threshold: 0.55,
            },
            None,
            "role",
        );
        for _ in 1..=4 {
            let out = a.tick("x").await;
            assert!(!out.fired);
        }
        let out = a.tick("x").await;
        assert!(out.fired);
        assert_eq!(out.reason, AnchorReason::TurnCounter);
    }

    #[test]
    fn cosine_handles_empty_and_mismatch() {
        assert_eq!(cosine_similarity(&[], &[1.0]), 0.0);
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 2.0]), 0.0);
        assert_eq!(cosine_similarity(&[0.0, 0.0], &[1.0, 1.0]), 0.0);
        // identical vectors -> 1.0
        let v = vec![1.0_f32, 2.0, 3.0];
        let s = cosine_similarity(&v, &v);
        assert!((s - 1.0).abs() < 1e-9);
    }

    #[test]
    fn reason_str_stable() {
        assert_eq!(AnchorReason::TurnCounter.as_str(), "turn_counter");
        assert_eq!(AnchorReason::CosineDrift.as_str(), "cosine_drift");
        assert_eq!(AnchorReason::NoTrigger.as_str(), "no_trigger");
    }
}
