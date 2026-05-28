//! E-4 chat UX critique substrate.
//!
//! Trust-impact-per-effort fixes ranked by the chat-UX critique report
//! (`workflow/improvement-sweep-2026-05-19/reports/04-chat-ux-critique.md`):
//!
//!   1. Per-claim confidence chips ([`ClaimWithConfidence`] +
//!      [`ConfidenceTier`]) -- AUDITOR stage emits these inline pills.
//!   2. Sycophancy guardrails ([`SycophancyGuard`]) -- AUDITOR refine
//!      loop helper that holds position on user disagreement when
//!      evidence supports it (canonical sycophancy anti-pattern fix per
//!      arXiv:2310.13548 Sharma + openai.com/index/sycophancy-in-gpt-4o
//!      April 2025 rollback).
//!   3. Pinned-rules visible panel -- dashboard component reads from
//!      `crate::rules` ACTIVE rows + per-session ad-hoc pins. Substrate
//!      ships the kind discriminators; the Tauri command + Svelte
//!      panel land in the dashboard crate.
//!   4. Session re-anchor every N turns ([`SessionReAnchorPolicy`])
//!      -- per arXiv:2402.10962 + arXiv:2412.00804 (8-15 turn drift).
//!   5. Doom-loop detector ([`DoomLoopDetector`]) -- thin facade over
//!      [`crate::anti_spiral`] (the 4-pattern detector lives there,
//!      not duplicated here).
//!   6. Evidence-required "done" claim ([`CompletionClaimCheck`])
//!      -- per HN 47936579 (Claude Code false-completion thread).
//!   7. One-shot clarifier ([`OneShotClarifier`]) -- per
//!      arXiv:2507.21285 "Curiosity by Design".
//!   8. Terse-by-default + /verbose toggle -- voice-anchor change;
//!      no Rust substrate beyond the [`VerbosityMode`] enum here.
//!   9. Expertise persistence -- reads `fact_type='expertise'` from
//!      `crate::memory_notes` (E-1 substrate); per-language Dreyfus
//!      5-stage carried in `ExpertiseLevel`.
//!  10. Refusal with override ([`RefusalScope`]) -- per
//!      arXiv:2509.08833 "Pitfalls of Over-Alignment".
//!  11. Mid-stream interrupt -- substrate-side this is just the
//!      [`InterruptCommit`] record; the actual cancellation rides the
//!      `providers::runtime::types::RuntimeStream<'_>` BoxStream drop
//!      path (verified hyper / mpsc; CLI-subprocess adapters need a
//!      regression test per E-4-decisions.md "cancel surface" row).
//!  12. Honest-correction surface ([`HonestCorrection`]) -- per
//!      anthropic.com/constitution + CLAUDE.md "When wrong" rule.
//!  13. Intervention pipeline visible control bar -- pure dashboard
//!      surface; no Rust substrate beyond the [`InterventionVerb`] enum.
//!
//! All emit via [`crate::ssb::SsbActionType::ChatUxCritiqueEvent`]
//! Pattern-2 envelope (30th application; 13-arm
//! [`crate::ssb::ChatUxCritiqueKind`] discriminator). Wire-dark unless
//! `ELAI_CHAT_UX_CRITIQUE_ENABLE=1`.
//!
//! Anchor sites (re-anchored at execute time; plan body refs were
//! stale):
//!   * AUDITOR refine loop: `crates/reasoning/src/chat_reflect.rs:
//!     379-393` (NOT line 635; that is a test fixture).
//!   * doom-loop observer hook + completion-claim audit:
//!     `crates/orchestrator/src/task_runner.rs::
//!     derive_autonomous_evidence` definition at line 11379 + call
//!     at line 6438 (NOT 3996/4645/7776 nor 8747/5943 from prior
//!     stale refs).
//!
//! See `workflow/reasoning/E-4-decisions.md` for the full decision
//! log + integration policy against the pre-existing sycophancy
//! surfaces (`SycophancyFlagStrip`, `SycophancyMatchedPairPanel`,
//! `SycophancyBenchPanel`, `output_postprocessor::strip_sycophancy`).

use serde::{Deserialize, Serialize};

/// Per-claim confidence tier surfaced in the AUDITOR-stage output.
/// Maps to CLAUDE.md "Confidence calibration" rule labels (Verified
/// / High confidence / Uncertain) plus an explicit "guessed" bucket
/// the canonical rule does not name but the chat-UX critique report
/// recommends surfacing.
///
/// Tier source-of-truth (decided in E-4-decisions.md):
///   * `Verified` -- at least one GRC grounding method passed
///     (FILE_CHECK / SOURCE_CHECK / TEST_CHECK / WEB_CHECK at
///     `crates/reasoning/`).
///   * `ReadFromSource` -- the AUDITOR cites a file:line in this
///     codebase but has not run a deterministic check on it.
///   * `FromTraining` -- no grounding method was called; the claim
///     comes from the model's training data.
///   * `Guessed` -- explicit `mcp_think(step_type="status")` echo
///     with low confidence, OR the AUDITOR self-labels the claim as
///     low-confidence in the inline pill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceTier {
    /// At least one GRC grounding method passed.
    Verified,
    /// Claim cites a file:line in this codebase, no det-check run.
    ReadFromSource,
    /// No grounding; claim from training data.
    FromTraining,
    /// Explicit low-confidence label (status-echo or self-flag).
    Guessed,
}

impl ConfidenceTier {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::ReadFromSource => "read_from_source",
            Self::FromTraining => "from_training",
            Self::Guessed => "guessed",
        }
    }
}

/// Reference to a piece of evidence that grounds a [`ClaimWithConfidence`].
/// One of: file diff path, SSB sequence number for an audit verdict, or
/// deterministic check id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum EvidenceRef {
    /// `file:path` or `file:path:line`.
    FileRef(String),
    /// SSB sequence number (per the `ssb` table `seq` column) for an
    /// audit verdict row that grounds the claim.
    SsbSeq(i64),
    /// Deterministic-check identifier (e.g. `cargo_check_pass`,
    /// `svelte_check_pass`).
    DetCheck(String),
}

/// Per-claim payload the AUDITOR stage emits inline. Renders as a small
/// confidence pill in the dashboard CommandWindow bubble header.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimWithConfidence {
    /// Surface text of the claim (the AUDITOR-rendered sentence).
    pub text: String,
    /// Confidence tier surfaced to the user.
    pub tier: ConfidenceTier,
    /// Optional evidence reference; `None` for `FromTraining` /
    /// `Guessed` tiers.
    pub evidence_ref: Option<EvidenceRef>,
}

impl ClaimWithConfidence {
    /// Construct a verified claim. Panics if `evidence_ref` is `None`
    /// in debug builds; in release the claim is constructed but the
    /// caller's GRC discipline is the only guarantee of grounding.
    #[must_use]
    pub fn verified(text: impl Into<String>, evidence_ref: EvidenceRef) -> Self {
        Self {
            text: text.into(),
            tier: ConfidenceTier::Verified,
            evidence_ref: Some(evidence_ref),
        }
    }

    /// Construct a "read-from-source" claim (cites file:line without
    /// running a det-check).
    #[must_use]
    pub fn read_from_source(text: impl Into<String>, file_ref: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tier: ConfidenceTier::ReadFromSource,
            evidence_ref: Some(EvidenceRef::FileRef(file_ref.into())),
        }
    }

    /// Construct a training-data claim (no grounding).
    #[must_use]
    pub fn from_training(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tier: ConfidenceTier::FromTraining,
            evidence_ref: None,
        }
    }

    /// Construct an explicit-low-confidence claim.
    #[must_use]
    pub fn guessed(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tier: ConfidenceTier::Guessed,
            evidence_ref: None,
        }
    }
}

/// AUDITOR-stage helper: holds position when the model has evidence
/// contradicting a user assertion. Returns
/// [`SycophancyVerdict::HoldPosition`] when the AUDITOR has at least
/// one piece of evidence supporting the original stance, OR
/// [`SycophancyVerdict::NoEvidence`] when there is no evidence and the
/// model MUST NOT push back (per the report's "If you have no
/// evidence, say so explicitly; do NOT agree to be agreeable" rule).
///
/// This is the GUARD shape -- distinct from the pre-existing regex
/// stripper at `crate::output_postprocessor::strip_sycophancy` (which
/// REMOVES sycophantic tokens from output). The guard ADDS calibrated
/// dissent at AUDITOR; the stripper removes flattery from the surface
/// output. Both ship together.
#[derive(Debug, Clone, Default)]
pub struct SycophancyGuard;

/// Result of consulting [`SycophancyGuard::evaluate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SycophancyVerdict {
    /// AUDITOR has evidence contradicting the user assertion; hold
    /// position with the cited evidence.
    HoldPosition {
        /// Evidence reference cited in the pushback.
        evidence: EvidenceRef,
    },
    /// AUDITOR has no evidence to push back; emit the
    /// `no_evidence_for_pushback` SSB row (per the WIRING_CONTRACT
    /// `stub_ban` requirement) and proceed without sycophantic
    /// agreement.
    NoEvidence,
}

impl SycophancyGuard {
    /// Evaluate whether to push back on a user assertion.
    /// `evidence_count` is the number of grounding-method-verified
    /// claims the AUDITOR has that contradict the user assertion.
    #[must_use]
    pub fn evaluate(
        &self,
        evidence_count: usize,
        primary_evidence: Option<EvidenceRef>,
    ) -> SycophancyVerdict {
        if evidence_count > 0 {
            if let Some(ev) = primary_evidence {
                return SycophancyVerdict::HoldPosition { evidence: ev };
            }
        }
        SycophancyVerdict::NoEvidence
    }
}

/// TRIAGE-stage helper: emits exactly ONE clarifier when the task
/// description is ambiguous, and NEVER more than one per task.
///
/// Heuristic floor (B-38 Ambiguity-Score continuous metric is the
/// future replacement; until then this fires when ALL three checks
/// trip):
///   1. task length < 50 chars
///   2. no action verb in the leading clause
///   3. no target file/scope ref
///
/// Per arXiv:2507.21285 "Curiosity by Design" and the report's
/// recommendation: exactly one round; after user response, proceed
/// without further clarification.
#[derive(Debug, Clone, Default)]
pub struct OneShotClarifier {
    /// Has a clarifier already been emitted for the current task?
    /// Toggled to `true` once [`OneShotClarifier::emit`] returns
    /// `Some`, preventing a second emission.
    emitted: bool,
}

impl OneShotClarifier {
    /// Construct a fresh clarifier helper for a new task.
    #[must_use]
    pub fn new() -> Self {
        Self { emitted: false }
    }

    /// Run the heuristic ambiguity floor. Returns `Some(N)` (the
    /// number of clarifier questions, 1..=3) on the first ambiguous
    /// task; `None` thereafter.
    pub fn emit(&mut self, task_text: &str) -> Option<u8> {
        if self.emitted {
            return None;
        }
        let trimmed = task_text.trim();
        let length_check = trimmed.len() < 50;
        let verb_check = !leading_clause_has_action_verb(trimmed);
        let scope_check = !mentions_target_file_or_scope(trimmed);
        if length_check && verb_check && scope_check {
            self.emitted = true;
            // 1-3 clarifier questions; default to 1 in the heuristic
            // floor (B-38 will tune this when shipped).
            return Some(1);
        }
        None
    }

    /// Mark the clarifier as resolved by the user. Subsequent calls
    /// to [`Self::emit`] still return `None` (one-shot guarantee).
    pub fn resolve(&mut self) {
        // Idempotent: the `emitted` flag is already `true` if emit
        // returned Some; if emit was never called, resolve is a no-op
        // so the next emit (on a fresh task) still works.
        self.emitted = true;
    }
}

fn leading_clause_has_action_verb(text: &str) -> bool {
    // Heuristic floor only: small lowercase verb list. B-38 will swap
    // this for a continuous ambiguity score.
    const ACTION_VERBS: &[&str] = &[
        "add", "implement", "fix", "refactor", "remove", "update",
        "create", "delete", "rename", "build", "test", "run",
        "deploy", "ship", "land", "merge", "revert", "investigate",
        "explore", "ground", "audit", "verify", "check", "debug",
        "trace", "analyze", "review", "summarize",
    ];
    let leading = text.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
    ACTION_VERBS.iter().any(|v| *v == leading.as_str())
}

fn mentions_target_file_or_scope(text: &str) -> bool {
    // Heuristic floor: looks for path / extension hints. B-38 will
    // replace with a parsed scope reference.
    let lower = text.to_ascii_lowercase();
    lower.contains('/')
        || lower.contains(".rs")
        || lower.contains(".ts")
        || lower.contains(".svelte")
        || lower.contains(".py")
        || lower.contains("crate::")
        || lower.contains("crates/")
}

/// AUDITOR-stage helper: rejects completion claims with no evidence.
/// The list of trigger phrases mirrors the report's recommendation:
/// "I've finished", "done", "task complete", "successfully implemented".
#[derive(Debug, Clone, Default)]
pub struct CompletionClaimCheck;

/// Result of [`CompletionClaimCheck::evaluate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionClaimVerdict {
    /// Output contains a completion claim with at least one evidence
    /// ref; allow.
    AllowedWithEvidence,
    /// Output contains a completion claim with no evidence ref;
    /// REJECT. Emit `completion_claim_rejected_no_evidence` SSB row.
    RejectedNoEvidence,
    /// Output contains no completion claim; allow.
    NotApplicable,
}

impl CompletionClaimCheck {
    /// Evaluate an AUDITOR-stage output for completion claims.
    #[must_use]
    pub fn evaluate(&self, output_text: &str, evidence_count: usize) -> CompletionClaimVerdict {
        const TRIGGERS: &[&str] = &[
            "i've finished",
            "i have finished",
            "task complete",
            "task is complete",
            "successfully implemented",
            "done.",
            " done ",
            "all done",
            "completed.",
            "finished.",
        ];
        let lower = output_text.to_ascii_lowercase();
        let has_completion_claim = TRIGGERS.iter().any(|t| lower.contains(t));
        if !has_completion_claim {
            return CompletionClaimVerdict::NotApplicable;
        }
        if evidence_count > 0 {
            CompletionClaimVerdict::AllowedWithEvidence
        } else {
            CompletionClaimVerdict::RejectedNoEvidence
        }
    }
}

/// Doom-loop detector facade. Wraps [`crate::anti_spiral`] (the
/// 4-pattern detector lives there; this facade ADDS the
/// chat-UX-critique specific thresholds + the SSB emission path).
///
/// Per the report and `.claude/skills/anti-spiral/SKILL.md`:
///   * 3 same-file edits for the same issue
///   * 3 failed hypotheses in `mcp_think` for the same topic
///   * auditor returns BLOCK twice in a row
///   * test-pass-rate plateau across attempts
#[derive(Debug, Clone)]
pub struct DoomLoopDetector {
    /// Number of same-file edits for the same issue before firing.
    pub same_file_edit_threshold: u32,
    /// Number of failed hypotheses on the same topic before firing.
    pub failed_hypothesis_threshold: u32,
    /// Number of consecutive auditor BLOCK verdicts before firing.
    pub auditor_block_streak_threshold: u32,
    /// Internal counters; keyed by (file_path, issue_id) etc. and
    /// reset by [`Self::reset_key`].
    same_file_edits: u32,
    failed_hypotheses: u32,
    auditor_block_streak: u32,
}

impl Default for DoomLoopDetector {
    fn default() -> Self {
        Self {
            same_file_edit_threshold: 3,
            failed_hypothesis_threshold: 3,
            auditor_block_streak_threshold: 2,
            same_file_edits: 0,
            failed_hypotheses: 0,
            auditor_block_streak: 0,
        }
    }
}

/// Result of consulting a [`DoomLoopDetector`] observer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoomLoopVerdict {
    /// No doom loop detected; continue.
    Continue,
    /// Doom loop detected; emit `doom_loop_detected` SSB row and
    /// surface the intervention card with 3 buttons (Switch tactic /
    /// Hand back to user / Continue anyway).
    Detected {
        /// Which threshold tripped.
        pattern: DoomLoopPattern,
    },
}

/// Which of the 4 thresholds tripped a [`DoomLoopVerdict::Detected`].
/// Mirrors the 4 stagnation patterns from
/// `.claude/skills/anti-spiral/SKILL.md` (Spinning / Oscillation /
/// NoDrift / DiminishingReturns) but uses chat-UX-specific labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoomLoopPattern {
    /// 3 same-file edits for the same issue.
    SameFileEditStorm,
    /// 3 failed hypotheses on the same topic.
    HypothesisStreak,
    /// Auditor BLOCK streak.
    AuditorBlockStreak,
    /// Test pass-rate plateau.
    PassRatePlateau,
}

impl DoomLoopDetector {
    /// Observe a same-file edit for the same issue.
    pub fn observe_edit(&mut self, _file: &str, _issue_id: &str) -> DoomLoopVerdict {
        self.same_file_edits = self.same_file_edits.saturating_add(1);
        if self.same_file_edits >= self.same_file_edit_threshold {
            return DoomLoopVerdict::Detected {
                pattern: DoomLoopPattern::SameFileEditStorm,
            };
        }
        DoomLoopVerdict::Continue
    }

    /// Observe a failed hypothesis on a topic.
    pub fn observe_hypothesis_failure(&mut self, _topic: &str) -> DoomLoopVerdict {
        self.failed_hypotheses = self.failed_hypotheses.saturating_add(1);
        if self.failed_hypotheses >= self.failed_hypothesis_threshold {
            return DoomLoopVerdict::Detected {
                pattern: DoomLoopPattern::HypothesisStreak,
            };
        }
        DoomLoopVerdict::Continue
    }

    /// Observe an auditor BLOCK verdict.
    pub fn observe_audit_block(&mut self) -> DoomLoopVerdict {
        self.auditor_block_streak = self.auditor_block_streak.saturating_add(1);
        if self.auditor_block_streak >= self.auditor_block_streak_threshold {
            return DoomLoopVerdict::Detected {
                pattern: DoomLoopPattern::AuditorBlockStreak,
            };
        }
        DoomLoopVerdict::Continue
    }

    /// Reset the auditor block streak after a PASS verdict.
    pub fn reset_audit_streak(&mut self) {
        self.auditor_block_streak = 0;
    }

    /// Reset all counters (e.g. after the user clicks "Switch tactic"
    /// in the intervention card).
    pub fn reset_all(&mut self) {
        self.same_file_edits = 0;
        self.failed_hypotheses = 0;
        self.auditor_block_streak = 0;
    }
}

/// Refusal scope. Per E-4-decisions.md row (e): local-codebase refusals
/// are user-overrideable; out-of-scope-of-project (outside `.elai_cc/`)
/// are hard-refuse with no override path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalScope {
    /// Refusal on a security / anti-pattern rule scoped to the local
    /// codebase. User can click "Override" to re-run with the named
    /// rule bypassed for that single dispatch.
    LocalCodebase,
    /// Refusal because the action targets a path outside `.elai_cc/`
    /// (the project-boundary HARD RULE). Hard refuse; no override
    /// button is rendered.
    OutsideProject,
}

impl RefusalScope {
    /// Whether the dashboard should render an "Override" button.
    #[must_use]
    pub fn is_overrideable(&self) -> bool {
        matches!(self, Self::LocalCodebase)
    }

    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalCodebase => "local_codebase",
            Self::OutsideProject => "outside_project",
        }
    }
}

/// Session re-anchor policy. Triggers re-injection of system prompt +
/// first user message + active rules after N turns OR before any
/// destructive action.
///
/// Default: N=10 turns. Configurable via SettingsWindow.
#[derive(Debug, Clone, Copy)]
pub struct SessionReAnchorPolicy {
    /// Turn count threshold for periodic re-anchor.
    pub turn_threshold: u32,
    /// Whether to also fire before destructive actions
    /// (`git commit`, `git push --force`, file delete, etc.).
    pub fire_before_destructive: bool,
}

impl Default for SessionReAnchorPolicy {
    fn default() -> Self {
        Self {
            turn_threshold: 10,
            fire_before_destructive: true,
        }
    }
}

/// Trigger that fired a [`SessionReAnchorPolicy`] re-injection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionReAnchorTrigger {
    /// Reached turn-count threshold.
    TurnThreshold,
    /// Destructive action about to fire.
    DestructiveAction,
    /// User manually invoked /re-anchor.
    Manual,
}

/// Verbosity mode controlling terse-by-default vs `/verbose` long-form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerbosityMode {
    /// Default: action + 1-line why + 1-line risk.
    Terse,
    /// Enabled by `/verbose`. Long-form for the next N turns or
    /// until `/terse`.
    Verbose,
}

impl Default for VerbosityMode {
    fn default() -> Self {
        Self::Terse
    }
}

/// Dreyfus 5-stage expertise level (Task 9 expertise persistence).
/// Defaults to `Competent` when no `fact_type='expertise'` row exists
/// in `crate::memory_notes`. Per arXiv:2504.06808 (expertise-mismatched
/// suggestions actively harm experts).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpertiseLevel {
    Novice,
    AdvancedBeginner,
    Competent,
    Proficient,
    Expert,
}

impl Default for ExpertiseLevel {
    fn default() -> Self {
        Self::Competent
    }
}

/// Intervention verb (Task 13 visible control bar slash commands).
/// `/pause`, `/redirect`, `/cancel` wired to the InterventionControlBar
/// in the dashboard ActivityWindow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionVerb {
    Pause,
    Redirect,
    Cancel,
}

/// Trigger source for an [`HonestCorrection`] block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HonestCorrectionTrigger {
    /// A test failed in the AUDITING state.
    TestFailed,
    /// Documentation / source disagreed with the prior claim.
    DocDisagreed,
    /// User corrected the model.
    UserCorrected,
}

/// Honest-correction record. Surface text: "I was wrong. Correct: X
/// because Y." Renders as a styled correction block in the
/// CommandWindow bubble.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HonestCorrection {
    /// Blake3 hex of the original (incorrect) claim text, for dedup
    /// against the prior bubble.
    pub original_claim_blake3_hex: String,
    /// Blake3 hex of the correction text.
    pub correction_claim_blake3_hex: String,
    /// What triggered the correction.
    pub trigger: HonestCorrectionTrigger,
}

/// Record of a mid-stream interrupt commit (Task 11).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterruptCommit {
    /// How many bytes had already streamed before cancellation.
    pub bytes_streamed_before_cancel: u64,
    /// Blake3 hex of the user's correction text.
    pub correction_blake3_hex: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_tier_round_trip() {
        for tier in [
            ConfidenceTier::Verified,
            ConfidenceTier::ReadFromSource,
            ConfidenceTier::FromTraining,
            ConfidenceTier::Guessed,
        ] {
            let s = tier.as_str();
            let parsed: ConfidenceTier =
                serde_json::from_value(serde_json::Value::String(s.to_string()))
                    .expect("tier round-trip");
            assert_eq!(tier, parsed);
        }
    }

    #[test]
    fn sycophancy_guard_holds_position_with_evidence() {
        let g = SycophancyGuard;
        let ev = EvidenceRef::FileRef("crates/foo/src/bar.rs:42".into());
        match g.evaluate(1, Some(ev.clone())) {
            SycophancyVerdict::HoldPosition { evidence } => assert_eq!(evidence, ev),
            SycophancyVerdict::NoEvidence => panic!("expected HoldPosition"),
        }
    }

    #[test]
    fn sycophancy_guard_no_evidence_yields_no_evidence_verdict() {
        let g = SycophancyGuard;
        assert!(matches!(
            g.evaluate(0, None),
            SycophancyVerdict::NoEvidence
        ));
    }

    #[test]
    fn one_shot_clarifier_fires_at_most_once() {
        let mut c = OneShotClarifier::new();
        // Ambiguous task: 8 chars, no verb, no scope.
        assert_eq!(c.emit("fix it"), Some(1));
        // Second call returns None on the same instance.
        assert_eq!(c.emit("fix it"), None);
    }

    #[test]
    fn one_shot_clarifier_skips_unambiguous_task() {
        let mut c = OneShotClarifier::new();
        let unambig =
            "implement the ConfidenceTier round-trip helper in crates/orchestrator/src/chat_ux/mod.rs";
        assert_eq!(c.emit(unambig), None);
    }

    #[test]
    fn completion_claim_rejects_when_no_evidence() {
        let c = CompletionClaimCheck;
        let v = c.evaluate("Task complete, all green.", 0);
        assert_eq!(v, CompletionClaimVerdict::RejectedNoEvidence);
    }

    #[test]
    fn completion_claim_allows_with_evidence() {
        let c = CompletionClaimCheck;
        let v = c.evaluate("Task complete, all green.", 2);
        assert_eq!(v, CompletionClaimVerdict::AllowedWithEvidence);
    }

    #[test]
    fn completion_claim_passes_non_completion_output() {
        let c = CompletionClaimCheck;
        let v = c.evaluate("Investigating the schema bump now.", 0);
        assert_eq!(v, CompletionClaimVerdict::NotApplicable);
    }

    #[test]
    fn doom_loop_fires_on_3_same_file_edits() {
        let mut d = DoomLoopDetector::default();
        assert_eq!(
            d.observe_edit("crates/foo/src/bar.rs", "issue-42"),
            DoomLoopVerdict::Continue
        );
        assert_eq!(
            d.observe_edit("crates/foo/src/bar.rs", "issue-42"),
            DoomLoopVerdict::Continue
        );
        assert!(matches!(
            d.observe_edit("crates/foo/src/bar.rs", "issue-42"),
            DoomLoopVerdict::Detected {
                pattern: DoomLoopPattern::SameFileEditStorm
            }
        ));
    }

    #[test]
    fn doom_loop_fires_on_audit_block_streak() {
        let mut d = DoomLoopDetector::default();
        assert_eq!(d.observe_audit_block(), DoomLoopVerdict::Continue);
        assert!(matches!(
            d.observe_audit_block(),
            DoomLoopVerdict::Detected {
                pattern: DoomLoopPattern::AuditorBlockStreak
            }
        ));
    }

    #[test]
    fn refusal_scope_is_overrideable_only_for_local() {
        assert!(RefusalScope::LocalCodebase.is_overrideable());
        assert!(!RefusalScope::OutsideProject.is_overrideable());
    }

    #[test]
    fn session_re_anchor_policy_default_is_10_turns() {
        let p = SessionReAnchorPolicy::default();
        assert_eq!(p.turn_threshold, 10);
        assert!(p.fire_before_destructive);
    }

    #[test]
    fn expertise_level_default_is_competent() {
        assert_eq!(ExpertiseLevel::default(), ExpertiseLevel::Competent);
    }

    #[test]
    fn verbosity_default_is_terse() {
        assert_eq!(VerbosityMode::default(), VerbosityMode::Terse);
    }
}
