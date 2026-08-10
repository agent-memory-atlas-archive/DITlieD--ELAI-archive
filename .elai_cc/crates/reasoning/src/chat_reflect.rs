//! B-39 Chat REFLECT pipeline — per-message reasoning layer.
//!
//! 4-stage pipeline (TRIAGE → REASONER N best-of-N → AUDITOR → SYNTHESIZE)
//! that wraps a chat turn so the model reasons about every problem-class
//! message instead of one-shot replying. Each stage submits a self-contained
//! prompt to the sidecar via [`ChatInferenceClient`]; separate POSTs give
//! independent KV caches per stage.
//!
//! ## Constants and their grounding
//! * `N_CANDIDATES = 4` — deliberate latency-vs-accuracy tradeoff for local
//!   single-instance Qwen2.5-Coder-3B/7B sidecars. Liu et al. arXiv:2502.06703
//!   Table 5 / Figure 4 shows the empirical Pareto-elbow for ≤7B BoN+PRM lies
//!   at N=16+; N=4 is the floor of the tested range {4,16,64,256}. We accept
//!   ~5–15pp absolute accuracy cost vs N=16 in exchange for ~3× faster
//!   per-turn wall-clock on a GPU-bound single-model setup. Future work:
//!   `/reason wide` flag bumps N to 16.
//! * `REFINE_CAP = 3` — RefineBench (arXiv:2511.22173) Table 3 guided-mode
//!   data: t=2→t=3 yields +12.6pp (Claude-Opus-4.1) / +17.4pp (o3-mini) when
//!   the refiner receives external grounding (which our auditor does:
//!   tools/knowledge crate output, not pure self-judgment). Cap=3 captures
//!   the bulk of the guided-refinement gain; t≥4 is empirically diminishing.
//!   The classic "self-correction caps at 2" finding (Kamoi 2024 TACL) is
//!   for *self*-refinement without external feedback — not our regime.
//!
//! ## Failure mode: never panic
//! Any error (sidecar down, JSON parse, timeout, refinement budget exhausted)
//! returns [`ReflectOutput`] with `skipped = true` and `final_reply` set to
//! the original user input passed straight through to the chat runtime.
//! Callers MUST treat the output as best-effort enrichment, not a hard
//! dependency.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::distill::{admit_distillate, build_distill_prompt, build_reseed_prompt, distill_threshold, DistillateVerdict};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Best-of-N width for the REASONER stage. See module-level rationale.
pub const N_CANDIDATES: u32 = 4;

/// Maximum refinement iterations across the AUDITOR loop. See module-level
/// rationale (RefineBench guided-mode data).
pub const REFINE_CAP: u32 = 3;

/// Tranche B-6 quad B-39 -- 85% soft-warn threshold. Fires when the
/// reflect pipeline's running token total reaches this percentage of
/// `ChatReflectConfig.budget_cap_tokens` AND is still strictly below the
/// hard cap. Named constant per CLAUDE.md "no magic numbers" rule;
/// 85% chosen so operators see a runway-warning with ~3K tokens of
/// headroom under the 20K production budget cited in
/// `project_qwen3b_baseline.md`. The soft-warn never throttles or fails
/// the pipeline -- it is an observability beacon only.
pub const REFLECT_BUDGET_SOFT_WARN_PCT: u8 = 85;

/// Percentage of [`ChatReflectConfig::budget_cap_tokens`] of accumulated AUDITOR
/// tokens that triggers a distill-reset of the refine loop. 75 mirrors the trim
/// fraction the context policy already uses for the local lane; it is restated
/// as a literal because `reasoning` does not depend on `orchestrator`.
pub const REFLECT_DISTILL_RESET_TRIGGER_PCT: u8 = 75;

/// Char ceiling for an admitted distillate.
pub const REFLECT_DISTILLATE_MAX_CHARS: u32 = 1_200;

/// Per-stage token caps. Sum (with N_CANDIDATES reasoner calls) stays under
/// the 20K production budget cited in `project_qwen3b_baseline.md`.
const MAX_TRIAGE_TOKENS: u32 = 256;
const MAX_REASONER_TOKENS: u32 = 2048;
const MAX_AUDITOR_TOKENS: u32 = 1024;
const MAX_DISTILL_TOKENS: u32 = 256;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors emitted by the REFLECT pipeline.
///
/// Every variant maps to `skipped = true` in the public `run()` API; callers
/// never observe a panic. `Display` impls are user-safe (no internal pointers
/// or PII leakage).
#[derive(Debug, Clone, Error)]
pub enum ReflectError {
    #[error("inference client error: {0}")]
    Inference(String),

    #[error("response parse error: {0}")]
    Parse(String),

    #[error("token budget exhausted: stage={stage} used={used} cap={cap}")]
    BudgetExhausted {
        stage: &'static str,
        used: u32,
        cap: u32,
    },

    #[error("refinement loop exceeded REFINE_CAP={REFINE_CAP}")]
    RefineCapExceeded,

    #[error("no candidates available (REASONER produced empty result set)")]
    NoCandidates,
}

// ---------------------------------------------------------------------------
// TurnClass — output of the TRIAGE stage
// ---------------------------------------------------------------------------

/// Classification a single chat turn falls into.
///
/// `Chat` short-circuits the pipeline (skip stages 2–4); `Problem` and
/// `CodeFix` proceed through the full REASONER+AUDITOR loop. The CLI
/// enables the pipeline only when `reflect_enabled` is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnClass {
    Chat,
    Problem,
    CodeFix,
}

impl TurnClass {
    /// Parse from the TRIAGE model output. Accepts the JSON-y forms the model
    /// is most likely to emit (single token, quoted string, `{"class":"…"}`).
    /// Falls back to `Chat` on any unrecognized input — safer than failing
    /// the whole pipeline for a parse error.
    pub fn parse_lenient(raw: &str) -> Self {
        let trimmed = raw.trim().trim_matches(|c: char| {
            c == '"' || c == '\'' || c == '{' || c == '}' || c == ','
        });
        let needle = trimmed.to_ascii_lowercase();
        if needle.contains("code_fix") || needle.contains("codefix") || needle.contains("\"code_fix\"") {
            TurnClass::CodeFix
        } else if needle.contains("problem") {
            TurnClass::Problem
        } else if needle.contains("chat") {
            TurnClass::Chat
        } else {
            // Conservative default: short greetings / acks bypass the
            // pipeline cleanly without burning compute.
            TurnClass::Chat
        }
    }

    pub fn as_wire_str(&self) -> &'static str {
        match self {
            TurnClass::Chat => "chat",
            TurnClass::Problem => "problem",
            TurnClass::CodeFix => "code_fix",
        }
    }
}

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
    fn from_stages(triage: u32, reasoner: u32, auditor: u32, synth: u32) -> Self {
        let stage_tokens = [triage, reasoner, auditor, synth];
        let total = stage_tokens.iter().copied().sum();
        Self {
            stage_tokens,
            total,
        }
    }
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
}

// ---------------------------------------------------------------------------
// ChatReflectConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ChatReflectConfig {
    pub enabled: bool,
    pub n_candidates: u32,
    pub budget_cap_tokens: u32,
    pub max_refine_iterations: u32,
    /// Tranche B-6 quad B-39 -- 85% soft-warn threshold percentage. Range
    /// `[0, 100]`; values outside the range collapse to
    /// [`REFLECT_BUDGET_SOFT_WARN_PCT`] at use-time via `min(100)`. Set
    /// to `0` to disable the soft-warn (it never fires when
    /// `soft_cap = 0`).
    pub budget_soft_warn_pct: u8,
    /// Percentage threshold for distill-reset. Range `[0, 100]`; larger values
    /// collapse via `min(100)` at use-time. `0` disables the trigger.
    pub distill_reset_trigger_pct: u8,
    /// Maximum character count accepted for a distillate.
    pub distillate_max_chars: u32,
}

impl Default for ChatReflectConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            n_candidates: N_CANDIDATES,
            // 20K cap leaves >5K headroom under the 25K hard ceiling
            // measured in `project_qwen3b_baseline.md`.
            budget_cap_tokens: 20_000,
            max_refine_iterations: REFINE_CAP,
            // Tranche B-6 quad B-39: 85% (`REFLECT_BUDGET_SOFT_WARN_PCT`).
            budget_soft_warn_pct: REFLECT_BUDGET_SOFT_WARN_PCT,
            distill_reset_trigger_pct: REFLECT_DISTILL_RESET_TRIGGER_PCT,
            distillate_max_chars: REFLECT_DISTILLATE_MAX_CHARS,
        }
    }
}

// ---------------------------------------------------------------------------
// ChatInferenceClient — the single trait dependency
// ---------------------------------------------------------------------------

/// Type alias for the boxed-future return shape of [`ChatInferenceClient`].
///
/// Mirrors `elai_providers::LlmFuture` style. Keeps the trait object-safe
/// (manual `Pin<Box<dyn Future>>`) without pulling in the `async-trait`
/// crate (banned by repo policy in favour of native async traits).
pub type ChatFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ReflectError>> + Send + 'a>>;

/// The single inference dependency the pipeline takes.
///
/// `complete` accepts a self-contained string prompt and a token cap; it
/// returns the decoded text. Each call is independent — no conversation
/// state is shared across calls. `SidecarProvider` is the production
/// implementation (see `wrap_sidecar` helper); tests use a mock that returns
/// canned responses.
pub trait ChatInferenceClient: Send + Sync {
    fn complete<'a>(&'a self, prompt: &'a str, max_tokens: u32) -> ChatFuture<'a, String>;
}

// ---------------------------------------------------------------------------
// ReflectEvent + ReflectEventSink — decouple from orchestrator/SSB
// ---------------------------------------------------------------------------

/// Per-stage observable event emitted by the pipeline.
///
/// The trait `ReflectEventSink` decouples the pipeline (in `reasoning/`)
/// from `orchestrator::Ssb` so the reasoning crate does not gain an
/// `elai-orchestrator` dependency (orchestrator already depends on reasoning;
/// the reverse would be circular). `Ssb` provides the production impl in
/// `orchestrator/src/ssb.rs`.
#[derive(Debug, Clone)]
pub enum ReflectEvent {
    Triaged {
        turn_id: String,
        class: TurnClass,
    },
    Candidate {
        turn_id: String,
        candidate_index: u32,
        content: String,
    },
    AuditVerdict {
        turn_id: String,
        best_index: u32,
        rationale: String,
    },
    Refined {
        turn_id: String,
        total_tokens_used: u32,
        synthesis: String,
    },
    /// Tranche B-6 quad B-39 -- 85% soft-warn beacon. Fires once per turn
    /// when `used >= soft_cap && used < hard_cap`. The pipeline returns
    /// the best available reply regardless (the soft-warn is purely
    /// observability). Mirrored into an SSB row via
    /// [`crate::ssb::SsbActionType::ReflectBudgetSoftWarn`].
    BudgetSoftWarn {
        turn_id: String,
        used: u32,
        soft_cap: u32,
        hard_cap: u32,
        pct: u8,
    },
}

/// Production sink writes to the SSB; mock sinks count events for tests.
///
/// `emit` is sync and fire-and-forget — implementations MAY spawn a tokio
/// task internally, but the pipeline does not await persistence. SSB write
/// failures are best-effort and never propagate to `ReflectOutput`.
pub trait ReflectEventSink: Send + Sync {
    fn emit(&self, event: ReflectEvent);
}

// ---------------------------------------------------------------------------
// ChatReflectPipeline
// ---------------------------------------------------------------------------

/// Per-message reasoning pipeline. Holds configuration + the optional event
/// sink; the inference client is passed per-call to `run()` so the CLI can
/// rebuild the sidecar provider on backend-config changes without throwing
/// away the pipeline state.
pub struct ChatReflectPipeline {
    config: ChatReflectConfig,
    sink: Option<Arc<dyn ReflectEventSink>>,
}

impl ChatReflectPipeline {
    /// `sink = None` is the test-mock and offline-sidecar case. The pipeline
    /// runs to completion regardless; events are simply discarded.
    pub fn new(config: ChatReflectConfig, sink: Option<Arc<dyn ReflectEventSink>>) -> Self {
        Self { config, sink }
    }

    pub fn config(&self) -> &ChatReflectConfig {
        &self.config
    }

    /// Run the pipeline on a single chat turn.
    ///
    /// `turn_id` correlates SSB rows across stages. The CLI passes its own
    /// monotonic counter; tests pass synthetic IDs.
    ///
    /// Never panics. Any internal error → `ReflectOutput { skipped: true, .. }`
    /// with `final_reply == user_input`.
    pub async fn run(
        &self,
        turn_id: String,
        user_input: &str,
        client: &dyn ChatInferenceClient,
    ) -> ReflectOutput {
        // The internal worker can return early via Result; the public API
        // collapses every error into the skipped-passthrough shape.
        match self.run_inner(&turn_id, user_input, client).await {
            Ok(output) => output,
            Err(err) => {
                tracing::warn!(
                    target: "reasoning::chat_reflect",
                    error = %err,
                    turn_id = %turn_id,
                    "REFLECT pipeline fell back to passthrough"
                );
                ReflectOutput {
                    final_reply: user_input.to_string(),
                    turn_class: TurnClass::Chat,
                    token_summary: ReflectTokenSummary::default(),
                    skipped: true,
                    turn_id,
                }
            }
        }
    }

    async fn run_inner(
        &self,
        turn_id: &str,
        user_input: &str,
        client: &dyn ChatInferenceClient,
    ) -> Result<ReflectOutput, ReflectError> {
        // ---- Stage 1: TRIAGE -------------------------------------------------
        let triage_prompt = build_triage_prompt(user_input);
        let triage_raw = client
            .complete(&triage_prompt, MAX_TRIAGE_TOKENS)
            .await?;
        let class = TurnClass::parse_lenient(&triage_raw);
        let triage_tokens = approx_tokens(&triage_raw);

        self.emit(ReflectEvent::Triaged {
            turn_id: turn_id.to_string(),
            class,
        });

        if class == TurnClass::Chat {
            // Short-circuit: chat-class turns bypass stages 2–4 entirely.
            return Ok(ReflectOutput {
                final_reply: user_input.to_string(),
                turn_class: class,
                token_summary: ReflectTokenSummary::from_stages(triage_tokens, 0, 0, 0),
                skipped: true,
                turn_id: turn_id.to_string(),
            });
        }

        // ---- Stage 2: REASONER (N-best sequential) ---------------------------
        let mut candidates: Vec<String> = Vec::with_capacity(self.config.n_candidates as usize);
        let mut reasoner_tokens: u32 = 0;
        for i in 0..self.config.n_candidates {
            let prompt = build_reasoner_prompt(user_input, class, i);
            let candidate = client
                .complete(&prompt, MAX_REASONER_TOKENS)
                .await?;
            reasoner_tokens = reasoner_tokens.saturating_add(approx_tokens(&candidate));

            self.emit(ReflectEvent::Candidate {
                turn_id: turn_id.to_string(),
                candidate_index: i,
                content: candidate.clone(),
            });
            candidates.push(candidate);
        }

        if candidates.is_empty() {
            return Err(ReflectError::NoCandidates);
        }

        // ---- Stage 3: AUDITOR (refinement loop ≤ REFINE_CAP) -----------------
        // FINAL TRANCHE B1: bounded refinement loop. The first auditor call
        // always runs. When `max_refine_iterations > 1` we re-run the auditor
        // with the previous verdict appended to the prompt and break on
        // convergence (two consecutive iterations agreeing on `best_index`).
        // When `max_refine_iterations == 0` we error per the dead-branch
        // semantic the original code carried; when `<= 1` the loop body is
        // skipped and behaviour is identical to the pre-B1 single-pass code.
        if self.config.max_refine_iterations == 0 {
            return Err(ReflectError::RefineCapExceeded);
        }

        let initial_prompt = build_auditor_prompt(user_input, class, &candidates);
        let auditor_raw = client
            .complete(&initial_prompt, MAX_AUDITOR_TOKENS)
            .await?;
        let mut auditor_tokens = approx_tokens(&auditor_raw);
        let (mut best_index, mut rationale) =
            parse_auditor_verdict(&auditor_raw, candidates.len() as u32);

        self.emit(ReflectEvent::AuditVerdict {
            turn_id: turn_id.to_string(),
            best_index,
            rationale: rationale.clone(),
        });

        // Cap at `max_refine_iterations - 1` extra passes (the first call
        // above counts as iteration 1). `saturating_sub` is the canonical
        // u32-bounded operator per CLAUDE.md (no .unwrap, no underflow).
        let extra_passes: u32 = self.config.max_refine_iterations.saturating_sub(1);
        let mut converged = extra_passes == 0; // single-pass = trivially converged
        let mut iter: u32 = 0;
        while iter < extra_passes {
            let prev_best = best_index;
            let threshold = distill_threshold(
                self.config.budget_cap_tokens,
                self.config.distill_reset_trigger_pct,
            );
            let armed = self.config.distill_reset_trigger_pct > 0
                && threshold > 0
                && auditor_tokens >= threshold;
            let refine_prompt = if !armed {
                build_refine_prompt(user_input, class, &candidates, prev_best, &rationale)
            } else {
                let distill_prompt = build_distill_prompt(user_input, class, prev_best, &rationale);
                let mut distill_raw = client.complete(&distill_prompt, MAX_DISTILL_TOKENS).await?;
                auditor_tokens = auditor_tokens.saturating_add(approx_tokens(&distill_raw));
                let mut verdict = admit_distillate(
                    &rationale,
                    &distill_raw,
                    usize::try_from(self.config.distillate_max_chars).unwrap_or(usize::MAX),
                );
                if !matches!(verdict, DistillateVerdict::Admitted) {
                    tracing::warn!(target: "reasoning::chat_reflect", ?verdict, turn_id = %turn_id, "distillate rejected");
                    distill_raw = client.complete(&distill_prompt, MAX_DISTILL_TOKENS).await?;
                    auditor_tokens = auditor_tokens.saturating_add(approx_tokens(&distill_raw));
                    verdict = admit_distillate(
                        &rationale,
                        &distill_raw,
                        usize::try_from(self.config.distillate_max_chars).unwrap_or(usize::MAX),
                    );
                    if !matches!(verdict, DistillateVerdict::Admitted) {
                        tracing::warn!(target: "reasoning::chat_reflect", ?verdict, turn_id = %turn_id, "distillate retry rejected");
                    }
                }
                match verdict {
                    DistillateVerdict::Admitted => build_reseed_prompt(user_input, class, &candidates, &distill_raw),
                    _ => build_refine_prompt(user_input, class, &candidates, prev_best, &rationale),
                }
            };
            let raw = client
                .complete(&refine_prompt, MAX_AUDITOR_TOKENS)
                .await?;
            auditor_tokens = auditor_tokens.saturating_add(approx_tokens(&raw));
            let (next_best, next_rationale) =
                parse_auditor_verdict(&raw, candidates.len() as u32);

            self.emit(ReflectEvent::AuditVerdict {
                turn_id: turn_id.to_string(),
                best_index: next_best,
                rationale: next_rationale.clone(),
            });

            best_index = next_best;
            rationale = next_rationale;
            iter = iter.saturating_add(1);
            if next_best == prev_best {
                converged = true;
                break;
            }
        }

        if !converged {
            return Err(ReflectError::RefineCapExceeded);
        }

        // ---- Stage 4: SYNTHESIZE (local, no inference call) ------------------
        // Pure local string select. Zero token cost. Out-of-range best_index
        // (model hallucination) collapses to candidate 0 — defensive but
        // never panics.
        let chosen_idx = (best_index as usize).min(candidates.len().saturating_sub(1));
        let final_reply = candidates
            .get(chosen_idx)
            .cloned()
            .unwrap_or_else(|| user_input.to_string());

        let token_summary =
            ReflectTokenSummary::from_stages(triage_tokens, reasoner_tokens, auditor_tokens, 0);

        // Tranche B-6 quad B-39 -- 85% soft-warn gate (BEFORE the hard-cap
        // check). Fires when `used >= soft_cap && used < hard_cap`; the
        // pipeline returns the best available reply regardless. Pure
        // observability beacon; no throttling, no skip.
        //
        // Wire-dark unless ELAI_B39_SOFTWARN_ENABLE=1 on the
        // orchestrator-side SSB sink (the sink itself is the env-gate
        // site; here we just emit and the sink decides whether to
        // persist).
        let pct = self.config.budget_soft_warn_pct.min(100);
        let hard_cap = self.config.budget_cap_tokens;
        // u64 arithmetic to avoid u32 overflow when hard_cap is near the
        // ceiling (~4.29e9 tokens * 100 would overflow u32; u32 * 100 fits
        // in u64 always).
        let soft_cap = ((u64::from(hard_cap) * u64::from(pct)) / 100)
            .try_into()
            .unwrap_or(u32::MAX);
        if soft_cap > 0
            && token_summary.total >= soft_cap
            && token_summary.total < hard_cap
        {
            tracing::warn!(
                target: "reasoning::chat_reflect",
                used = token_summary.total,
                soft_cap,
                hard_cap,
                pct,
                turn_id = %turn_id,
                "REFLECT entered budget soft-warn zone (continuing)"
            );
            self.emit(ReflectEvent::BudgetSoftWarn {
                turn_id: turn_id.to_string(),
                used: token_summary.total,
                soft_cap,
                hard_cap,
                pct,
            });
        }

        // Budget gate (defensive — log and continue; we still return the best
        // available reply rather than skipping a successful pipeline).
        if token_summary.total > self.config.budget_cap_tokens {
            tracing::warn!(
                target: "reasoning::chat_reflect",
                used = token_summary.total,
                cap = self.config.budget_cap_tokens,
                turn_id = %turn_id,
                "REFLECT exceeded budget cap (still returning result)"
            );
        }

        self.emit(ReflectEvent::Refined {
            turn_id: turn_id.to_string(),
            total_tokens_used: token_summary.total,
            synthesis: final_reply.clone(),
        });

        Ok(ReflectOutput {
            final_reply,
            turn_class: class,
            token_summary,
            skipped: false,
            turn_id: turn_id.to_string(),
        })
    }

    fn emit(&self, event: ReflectEvent) {
        if let Some(sink) = &self.sink {
            sink.emit(event);
        }
    }
}

// ---------------------------------------------------------------------------
// Prompt builders — kept private so the wire format stays an internal contract
// ---------------------------------------------------------------------------

fn build_triage_prompt(user_input: &str) -> String {
    format!(
        "You classify a single chat message into one of three categories.\n\
         Output exactly one of: chat, problem, code_fix\n\
         - chat: greetings, acknowledgements, casual conversation, simple lookups\n\
         - problem: any question requiring multi-step reasoning, math, design\n\
         - code_fix: requests to debug, modify, or write code\n\
         Output the bare category word, nothing else.\n\n\
         Message: {user_input}\n\n\
         Category:"
    )
}

fn build_reasoner_prompt(user_input: &str, class: TurnClass, sample_index: u32) -> String {
    let role_hint = match class {
        TurnClass::CodeFix => "You write a focused code-level answer.",
        TurnClass::Problem => "You reason step by step about the problem.",
        TurnClass::Chat => "You answer the message directly.",
    };
    format!(
        "{role_hint}\n\
         Sample variation seed: {sample_index}\n\
         User message: {user_input}\n\n\
         Answer:"
    )
}

fn build_auditor_prompt(user_input: &str, class: TurnClass, candidates: &[String]) -> String {
    let mut joined = String::new();
    for (i, c) in candidates.iter().enumerate() {
        joined.push_str(&format!("[{i}]\n{c}\n\n"));
    }
    format!(
        "You evaluate {n} candidate answers to a {class_str} message and pick the best one.\n\
         Output strictly: <index>|<one-sentence rationale>\n\
         Example: 2|Most directly addresses the requested fix.\n\n\
         User message: {user_input}\n\n\
         Candidates:\n{joined}\
         Verdict:",
        n = candidates.len(),
        class_str = class.as_wire_str(),
    )
}

/// Build the prompt for an auditor refinement pass. Mirrors
/// [`build_auditor_prompt`] but appends the previous verdict + rationale so
/// the auditor can re-evaluate with the prior pass as grounded context (per
/// RefineBench arXiv:2511.22173 guided-mode protocol).
fn build_refine_prompt(
    user_input: &str,
    class: TurnClass,
    candidates: &[String],
    prev_best: u32,
    prev_rationale: &str,
) -> String {
    let mut joined = String::new();
    for (i, c) in candidates.iter().enumerate() {
        joined.push_str(&format!("[{i}]\n{c}\n\n"));
    }
    format!(
        "You re-evaluate {n} candidate answers to a {class_str} message after a \
         prior audit pass picked candidate {prev_best}.\n\
         Either confirm the prior pick or pick a different candidate.\n\
         Output strictly: <index>|<one-sentence rationale>\n\
         Example: 2|Prior pick missed the requested side-effect.\n\n\
         User message: {user_input}\n\n\
         Previous verdict: index={prev_best}; rationale={prev_rationale}\n\n\
         Candidates:\n{joined}\
         Verdict:",
        n = candidates.len(),
        class_str = class.as_wire_str(),
    )
}

/// Lenient parse of the auditor's `<index>|<rationale>` line. Accepts decimal
/// indices, ignores extra whitespace, falls back to index 0 with the raw
/// response as rationale on any malformed output.
fn parse_auditor_verdict(raw: &str, n_candidates: u32) -> (u32, String) {
    let trimmed = raw.trim();
    if let Some((idx_part, rationale_part)) = trimmed.split_once('|') {
        if let Ok(parsed_idx) = idx_part.trim().parse::<u32>() {
            let bounded = parsed_idx.min(n_candidates.saturating_sub(1));
            let rationale = rationale_part.trim().to_string();
            let rationale = if rationale.is_empty() {
                "(no rationale)".to_string()
            } else {
                rationale
            };
            return (bounded, rationale);
        }
    }
    (0, trimmed.to_string())
}

/// Cheap token-count proxy: 4 chars ≈ 1 token for English/code on the Qwen2.5
/// tokenizer. Off by ~10–20% in either direction; only used for telemetry,
/// never for budget enforcement of an in-flight call.
fn approx_tokens(s: &str) -> u32 {
    let chars = s.chars().count();
    // Round up to avoid undercounting near zero.
    u32::try_from(chars.div_ceil(4)).unwrap_or(u32::MAX)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct MockClient {
        responses: Vec<Result<String, ReflectError>>,
        cursor: AtomicU32,
    }

    impl MockClient {
        fn ok(responses: &[&str]) -> Self {
            Self {
                responses: responses.iter().map(|s| Ok(s.to_string())).collect(),
                cursor: AtomicU32::new(0),
            }
        }

        fn calls(&self) -> u32 {
            self.cursor.load(Ordering::SeqCst)
        }
    }

    impl ChatInferenceClient for MockClient {
        fn complete<'a>(&'a self, _prompt: &'a str, _max_tokens: u32) -> ChatFuture<'a, String> {
            Box::pin(async move {
                let idx = self.cursor.fetch_add(1, Ordering::SeqCst) as usize;
                self.responses
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| Err(ReflectError::Inference("ran out of canned responses".to_string())))
            })
        }
    }

    #[derive(Default)]
    struct CountingSink {
        triaged: AtomicU32,
        candidate: AtomicU32,
        verdict: AtomicU32,
        refined: AtomicU32,
        // Tranche B-6 quad B-39: 85% soft-warn beacon counter.
        soft_warn: AtomicU32,
    }

    impl ReflectEventSink for CountingSink {
        fn emit(&self, event: ReflectEvent) {
            match event {
                ReflectEvent::Triaged { .. } => {
                    self.triaged.fetch_add(1, Ordering::SeqCst);
                }
                ReflectEvent::Candidate { .. } => {
                    self.candidate.fetch_add(1, Ordering::SeqCst);
                }
                ReflectEvent::AuditVerdict { .. } => {
                    self.verdict.fetch_add(1, Ordering::SeqCst);
                }
                ReflectEvent::Refined { .. } => {
                    self.refined.fetch_add(1, Ordering::SeqCst);
                }
                ReflectEvent::BudgetSoftWarn { .. } => {
                    self.soft_warn.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    }

    #[tokio::test]
    async fn chat_class_short_circuits_to_passthrough() {
        let client = MockClient::ok(&["chat"]);
        let pipeline = ChatReflectPipeline::new(ChatReflectConfig::default(), None);
        let out = pipeline.run("t1".to_string(), "thanks!", &client).await;
        assert!(out.skipped);
        assert_eq!(out.final_reply, "thanks!");
        assert_eq!(out.turn_class, TurnClass::Chat);
        assert_eq!(client.calls(), 1, "only TRIAGE should be called");
    }

    #[tokio::test]
    async fn problem_runs_full_pipeline_with_n_candidates() {
        let mut canned = vec!["problem".to_string()];
        for i in 0..N_CANDIDATES {
            canned.push(format!("answer-{i}"));
        }
        canned.push("0|chosen for clarity".to_string());
        let responses: Vec<Result<String, ReflectError>> =
            canned.into_iter().map(Ok).collect();
        let client = MockClient {
            responses,
            cursor: AtomicU32::new(0),
        };
        let sink = Arc::new(CountingSink::default());
        // max_refine_iterations = 1: single-pass auditor (no extra refine passes).
        // The test's canned responses provide exactly 1 auditor response, matching
        // the single-pass call count: 1 triage + N reasoner + 1 auditor = 2 + N.
        let config = ChatReflectConfig {
            max_refine_iterations: 1,
            ..ChatReflectConfig::default()
        };
        let pipeline = ChatReflectPipeline::new(
            config,
            Some(sink.clone() as Arc<dyn ReflectEventSink>),
        );
        let out = pipeline.run("t1".to_string(), "what's 2+2?", &client).await;

        assert!(!out.skipped);
        assert_eq!(out.final_reply, "answer-0");
        // 1 triage + N reasoner + 1 auditor + 0 synth = 2 + N inference calls.
        assert_eq!(client.calls(), 2 + N_CANDIDATES);
        assert_eq!(sink.triaged.load(Ordering::SeqCst), 1);
        assert_eq!(sink.candidate.load(Ordering::SeqCst), N_CANDIDATES);
        assert_eq!(sink.verdict.load(Ordering::SeqCst), 1);
        assert_eq!(sink.refined.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn provider_error_collapses_to_skipped() {
        let client = MockClient {
            responses: vec![Err(ReflectError::Inference("boom".to_string()))],
            cursor: AtomicU32::new(0),
        };
        let pipeline = ChatReflectPipeline::new(ChatReflectConfig::default(), None);
        let out = pipeline.run("t1".to_string(), "hello", &client).await;
        assert!(out.skipped);
        assert_eq!(out.final_reply, "hello");
    }

    #[test]
    fn turn_class_parse_lenient_handles_quoted_and_bare() {
        assert_eq!(TurnClass::parse_lenient("chat"), TurnClass::Chat);
        assert_eq!(TurnClass::parse_lenient("\"problem\""), TurnClass::Problem);
        assert_eq!(TurnClass::parse_lenient("code_fix\n"), TurnClass::CodeFix);
        // Unrecognized output → conservative Chat default
        assert_eq!(TurnClass::parse_lenient("xyz"), TurnClass::Chat);
    }

    #[test]
    fn auditor_verdict_clamps_out_of_range_index() {
        let (idx, rationale) = parse_auditor_verdict("99|bad pick", 4);
        assert_eq!(idx, 3);
        assert_eq!(rationale, "bad pick");
    }

    #[test]
    fn auditor_verdict_falls_back_on_malformed_output() {
        let (idx, rationale) = parse_auditor_verdict("nonsense", 4);
        assert_eq!(idx, 0);
        assert_eq!(rationale, "nonsense");
    }

    #[test]
    fn token_summary_total_equals_sum_of_stages() {
        let s = ReflectTokenSummary::from_stages(10, 80, 20, 0);
        assert_eq!(s.total, 110);
        assert_eq!(s.stage_tokens.iter().sum::<u32>(), s.total);
    }

    #[test]
    fn refine_cap_constant_is_three() {
        // RefineBench guided-mode (Table 3) shows t=2->t=3 captures 12-20pp;
        // t>=4 is diminishing. cap=3 is the documented choice.
        assert_eq!(REFINE_CAP, 3);
    }

    #[test]
    fn default_distill_trigger_is_unreachable_at_shipped_caps() {
        // Guarantee live /reason behaviour does not change before EXP-3 enables it.
        let config = ChatReflectConfig::default();
        let threshold = crate::distill::distill_threshold(
            config.budget_cap_tokens,
            config.distill_reset_trigger_pct,
        );
        let max_accumulation = REFINE_CAP.saturating_mul(MAX_AUDITOR_TOKENS);
        assert!(threshold > max_accumulation);
    }

    #[test]
    fn n_candidates_constant_is_four() {
        // Liu et al. (arXiv:2502.06703) Table 5 / Figure 4: N=16 is the
        // empirical Pareto-elbow minimum for Qwen 3B/7B-class. We accept the
        // ~5-15pp accuracy cost at N=4 in exchange for ~3x faster wall-clock
        // on a single-instance GPU-bound sidecar. /reason wide (future work)
        // bumps to 16.
        assert_eq!(N_CANDIDATES, 4);
    }

    /// FINAL TRANCHE B1: with max_refine_iterations > 1 and a verdict that
    /// keeps flipping, the loop must trip RefineCapExceeded and collapse
    /// into the public `skipped=true` passthrough shape.
    #[tokio::test]
    async fn refine_loop_cap_exceeded_collapses_to_skipped() {
        // 1 triage + N reasoner + (cap = REFINE_CAP) auditor passes that
        // never converge (alternating best_index). Sequence: triage="problem",
        // then N candidates, then auditor sequence "0|a", "1|b", "0|c"
        // which alternates and never converges across REFINE_CAP iterations.
        let mut canned = vec!["problem".to_string()];
        for i in 0..N_CANDIDATES {
            canned.push(format!("answer-{i}"));
        }
        // Auditor + (REFINE_CAP - 1) re-passes; alternate indices so
        // convergence never fires.
        for i in 0..REFINE_CAP {
            let idx = i % 2;
            canned.push(format!("{idx}|pass-{i}"));
        }
        let responses: Vec<Result<String, ReflectError>> =
            canned.into_iter().map(Ok).collect();
        let client = MockClient {
            responses,
            cursor: AtomicU32::new(0),
        };
        let pipeline = ChatReflectPipeline::new(ChatReflectConfig::default(), None);
        let out = pipeline
            .run("t1".to_string(), "diverge me", &client)
            .await;
        // Cap exceeded -> internal error -> public passthrough.
        assert!(out.skipped, "expected skipped=true on RefineCapExceeded");
        assert_eq!(out.final_reply, "diverge me");
    }

    /// FINAL TRANCHE B1: when max_refine_iterations <= 1 the loop body is
    /// skipped and behaviour is identical to the pre-B1 single-pass code
    /// (backward-compat invariant).
    #[tokio::test]
    async fn refine_loop_single_pass_matches_legacy() {
        let mut canned = vec!["problem".to_string()];
        for i in 0..N_CANDIDATES {
            canned.push(format!("answer-{i}"));
        }
        canned.push("0|chosen".to_string());
        let responses: Vec<Result<String, ReflectError>> =
            canned.into_iter().map(Ok).collect();
        let client = MockClient {
            responses,
            cursor: AtomicU32::new(0),
        };
        let cfg = ChatReflectConfig {
            max_refine_iterations: 1,
            ..ChatReflectConfig::default()
        };
        let pipeline = ChatReflectPipeline::new(cfg, None);
        let out = pipeline.run("t1".to_string(), "what's 2+2?", &client).await;
        assert!(!out.skipped);
        assert_eq!(out.final_reply, "answer-0");
        // Legacy single-pass = 1 triage + N reasoner + 1 auditor.
        assert_eq!(client.calls(), 2 + N_CANDIDATES);
    }
}
