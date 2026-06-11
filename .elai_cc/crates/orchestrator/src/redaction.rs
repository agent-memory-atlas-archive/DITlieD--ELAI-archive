//! G-9 — LOCAL-only, fail-closed, write-time secret + PII redactor.
//!
//! The SSB journal is append-only (ssb.rs:5: "no UPDATE or DELETE"), so a
//! secret that lands in a row can never be scrubbed afterwards. The only
//! correct cut-point is WRITE-TIME, on the typed [`SsbPayload`] BEFORE
//! `rmp_serde::to_vec_named` serializes it ([`crate::ssb::Ssb::write_payload`]).
//! This module is wired there (and into the Rust hook-log sink) so every
//! committed row — and, transitively, every G-2 Reproducible-Run Artifact that
//! re-bundles those rows ([`crate::rra::build_rra`]) — is already redacted.
//!
//! Design (G-9-decisions.md DD1-DD3):
//!   * Detectors are the UNION of F-1's `SECRET_PATTERNS` strings
//!     ([`crate::vibegate_secrets`]) and B-27's PII pattern strings
//!     ([`crate::privacy_redactor`]). F-1's `SECRET_PATTERNS` is a `RegexSet`
//!     (membership only — `.is_match()` / `.matches()`, no spans), so to
//!     REPLACE each hit this module compiles a span-yielding per-pattern
//!     `regex::Regex` over the SAME const strings (GD-1: REUSE, re-implement no
//!     detector). The set is compiled ONCE via [`std::sync::OnceLock`] —
//!     `write_payload` is the hot single-writer path.
//!   * Each match is replaced with a STABLE, irreversible token
//!     `[REDACTED:<label>:<blake3-16>]` where the 16 hex chars are the blake3
//!     prefix of the matched secret VALUE (mirrors
//!     [`crate::vibegate::source_blake3_prefix`], vibegate.rs:148). Identical
//!     secrets map to identical tokens (correlation survives) and the token is
//!     never the secret (blake3 is one-way; we keep only a prefix). blake3 is
//!     the workspace hash — NO new dependency (blake3 + regex are in-tree).
//!   * FAIL-CLOSED: a match that would yield an empty/degenerate token is
//!     replaced by a fully-redacted placeholder, never the raw value. The
//!     redactor never returns the raw secret on any path.
//!
//! E-38 honesty (DD9): pattern-based secret + PII detection is high-recall /
//! low-precision (qualitative; the refuted PRvL Presidio F1 numbers are NOT
//! cited here). The EU AI Act transparency-tier penalty is EUR 15M / 3% of
//! turnover (Art-99), NOT 35M / 7% (that is the Art-5 prohibited-practices
//! tier) — source: artificialintelligenceact.eu/article/99 and the project's
//! own ELAI-TRUST-MAXIMIZATION-2026-06-02.md R9. NO Presidio / ggshield
//! dependency — this is a hand-rolled local redactor.
//!
//! Hook-log scope (DD10): [`redact`] is wired into the Rust
//! `skill_telemetry::log_skill_invocation` sink over its structured string
//! fields (a small surface — that sink already hashes the prompt, it never
//! writes a raw prompt body). The per-session `hook-executions.log` written by
//! the Python `.claude/hooks` harness is OUTSIDE the project boundary; this
//! Rust redactor CANNOT and does NOT cover it. We do not claim otherwise.

use std::sync::OnceLock;

use regex::Regex;

use crate::ssb::SsbPayload;
use crate::vibegate_secrets::{PAT_AWS_KEY, PAT_PEM_GENERIC, PAT_PEM_TYPED, PAT_SECRET_ASSIGN};

// ---------------------------------------------------------------------------
// Named constants (token-discipline: no magic numbers)
// ---------------------------------------------------------------------------

/// Hex length of the blake3 prefix in a redaction token. Mirrors
/// `vibegate::source_blake3_prefix` (the first 16 hex chars of `blake3().to_hex()`)
/// and `session_archive::MANIFEST_HEX_PREFIX_LEN`. Long enough to be
/// collision-resistant for correlation, short enough to stay non-reversible.
pub const TOKEN_HEX_LEN: usize = 16;

/// The placeholder used when fail-closed redaction cannot derive a stable token
/// (degenerate / empty match). Carries NO secret value.
const FAIL_CLOSED_TOKEN: &str = "[REDACTED:fail_closed]";

// ---------------------------------------------------------------------------
// Detector pattern table (UNION of F-1 secret strings + B-27 PII strings)
// ---------------------------------------------------------------------------

/// PII: RFC-ish email address. Mirrors B-27 `privacy_redactor::email_re`.
const PAT_EMAIL: &str = r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}";

/// PII: well-known cloud / git token prefixes (Anthropic / OpenAI / GitHub).
/// Mirrors B-27 `privacy_redactor::token_re`. `sk-` is anchored at a word
/// boundary AND requires the 20+ char run to avoid the "sk-" prose over-match
/// hot-spot (DD7).
const PAT_API_TOKEN: &str =
    r"\b(?:sk-ant-[A-Za-z0-9_\-]{16,}|sk-[A-Za-z0-9]{20,}|ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,})\b";

// DD7: a bare IPv4 detector is DELIBERATELY OMITTED. The regex crate has no
// lookaround, so a dotted quad like `1.2.3.4` (a version string in the binding
// benign corpus) is INDISTINGUISHABLE from a real IP by pattern alone — any
// octet-range regex still matches it. Per G-9-decisions.md ("err toward NOT
// redacting ambiguous low-entropy tokens; a redactor that mangles code is worse
// than a missed low-entropy token") and the trust-max R9 priority (secrets >
// bare IPs), IPv4 is the lowest-value / highest-FP PII and is left un-redacted.

/// Stable label for each detector, indexed in lockstep with [`detectors`].
/// Used in the `[REDACTED:<label>:...]` token so a consumer can tell WHAT kind
/// of secret fired without seeing the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretKind {
    AwsAccessKeyId,
    PemPrivateKeyTyped,
    PemPrivateKeyGeneric,
    SecretLiteralAssignment,
    Email,
    ApiToken,
}

impl SecretKind {
    /// Snake-case label embedded in the redaction token. Stable wire string.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::AwsAccessKeyId => "aws_access_key_id",
            Self::PemPrivateKeyTyped => "pem_private_key_typed",
            Self::PemPrivateKeyGeneric => "pem_private_key_generic",
            Self::SecretLiteralAssignment => "secret_literal_assignment",
            Self::Email => "email",
            Self::ApiToken => "api_token",
        }
    }
}

/// Detector ordering. PEM-typed BEFORE PEM-generic so the more specific header
/// wins; otherwise order is irrelevant because spans are de-overlapped before
/// splicing.
const DETECTOR_KINDS: [SecretKind; 6] = [
    SecretKind::AwsAccessKeyId,
    SecretKind::PemPrivateKeyTyped,
    SecretKind::PemPrivateKeyGeneric,
    SecretKind::SecretLiteralAssignment,
    SecretKind::Email,
    SecretKind::ApiToken,
];

const DETECTOR_PATTERNS: [&str; 6] = [
    PAT_AWS_KEY,
    PAT_PEM_TYPED,
    PAT_PEM_GENERIC,
    PAT_SECRET_ASSIGN,
    PAT_EMAIL,
    PAT_API_TOKEN,
];

/// Compiled span-yielding detector set, built ONCE. Reuses the F-1 + B-27
/// pattern STRINGS; F-1's own `SECRET_PATTERNS` `RegexSet` cannot yield spans.
///
/// `.expect` is acceptable here under the CLAUDE.md LazyLock/OnceLock init
/// exception: every pattern is a literal const verified at review time, so a
/// compile failure means "won't start, you'll see it immediately" — it is not
/// runtime error handling.
fn detectors() -> &'static [(SecretKind, Regex)] {
    static SET: OnceLock<Vec<(SecretKind, Regex)>> = OnceLock::new();
    SET.get_or_init(|| {
        DETECTOR_KINDS
            .iter()
            .zip(DETECTOR_PATTERNS.iter())
            .map(|(kind, pat)| {
                (
                    *kind,
                    Regex::new(pat).expect("redaction: literal detector pattern must compile"),
                )
            })
            .collect()
    })
}

// ---------------------------------------------------------------------------
// Token derivation
// ---------------------------------------------------------------------------

/// Derive the STABLE, irreversible redaction token for a matched secret.
///
/// `[REDACTED:<label>:<blake3-16>]`. The 16 hex chars are the blake3 prefix of
/// the matched VALUE — same secret -> same token (correlatable), never the
/// secret itself. Fail-closed: a degenerate (empty) match yields the
/// [`FAIL_CLOSED_TOKEN`], never the raw bytes.
fn token_for(kind: SecretKind, matched: &str) -> String {
    if matched.is_empty() {
        return FAIL_CLOSED_TOKEN.to_owned();
    }
    let hex = blake3::hash(matched.as_bytes()).to_hex();
    let prefix = hex.get(..TOKEN_HEX_LEN).unwrap_or("");
    if prefix.is_empty() {
        // blake3 hex is always 64 chars, so this is unreachable in practice;
        // the guard keeps the function fail-closed by construction.
        return FAIL_CLOSED_TOKEN.to_owned();
    }
    format!("[REDACTED:{}:{}]", kind.label(), prefix)
}

// ---------------------------------------------------------------------------
// A single redaction hit (off-agent advisory evidence — carries NO secret)
// ---------------------------------------------------------------------------

/// One redaction occurrence. Carries the stable token + the kind + the field —
/// NEVER the raw secret value. Feeds the `secret_redacted` advisory SSB row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionHit {
    /// The kind of detector that fired.
    pub kind: SecretKind,
    /// The stable `[REDACTED:<label>:<blake3-16>]` token that replaced the
    /// match. Safe to persist — it is a blake3 prefix, not the secret.
    pub token: String,
}

// ---------------------------------------------------------------------------
// Core text redactor (DD2/DD3)
// ---------------------------------------------------------------------------

/// Redact every secret / PII match in `text`, returning the redacted string and
/// the list of hits (each hit carries the stable token + kind, NEVER the
/// secret). On no match, returns the input unchanged (byte-identical) and an
/// empty hit list — the FP-corpus guarantee (DD7).
///
/// Overlap handling: matches from all detectors are collected, sorted by start,
/// and any span overlapping an already-accepted span is dropped (longest-first
/// within an offset wins) so a single secret is tokenized exactly once.
#[must_use]
pub fn redact(text: &str) -> (String, Vec<RedactionHit>) {
    // Collect candidate (start, end, kind) spans across every detector.
    let mut spans: Vec<(usize, usize, SecretKind)> = Vec::new();
    for (kind, re) in detectors() {
        for m in re.find_iter(text) {
            spans.push((m.start(), m.end(), *kind));
        }
    }
    if spans.is_empty() {
        return (text.to_owned(), Vec::new());
    }

    // De-overlap: sort by start asc, then by length desc so the widest span at
    // a given start is preferred. Then sweep, dropping any span that starts
    // before the previous accepted span's end.
    spans.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| (b.1 - b.0).cmp(&(a.1 - a.0))));

    let mut out = String::with_capacity(text.len());
    let mut hits: Vec<RedactionHit> = Vec::new();
    let mut cursor: usize = 0;
    for (start, end, kind) in spans {
        if start < cursor {
            // Overlaps an already-emitted token — skip (counted once).
            continue;
        }
        // Copy the un-redacted gap verbatim.
        if let Some(gap) = text.get(cursor..start) {
            out.push_str(gap);
        }
        let matched = text.get(start..end).unwrap_or("");
        let token = token_for(kind, matched);
        out.push_str(&token);
        hits.push(RedactionHit { kind, token });
        cursor = end;
    }
    // Trailing un-redacted remainder.
    if let Some(tail) = text.get(cursor..) {
        out.push_str(tail);
    }

    (out, hits)
}

// ---------------------------------------------------------------------------
// Typed-payload redactor wired into write_payload (DD1)
// ---------------------------------------------------------------------------

/// Outcome of redacting a typed payload: the redacted-field identity + the hits.
/// `field` is the dotted `variant.field` identity (e.g.
/// `idea_framing_started.prompt`) — NEVER a value. Empty `hits` means no-op.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PayloadRedaction {
    /// The dotted `variant.field` identity that was redacted (identity only).
    /// Empty when nothing fired.
    pub field: String,
    /// Every redaction hit (each carries the stable token + kind, NO secret).
    pub hits: Vec<RedactionHit>,
}

impl PayloadRedaction {
    /// `true` iff at least one redaction fired.
    #[must_use]
    pub fn fired(&self) -> bool {
        !self.hits.is_empty()
    }
}

/// Redact the free-text String fields of a typed [`SsbPayload`] IN PLACE, prior
/// to msgpack serialization. Returns the redacted-field identity + every hit
/// (empty when nothing fired). Called at the TOP of
/// [`crate::ssb::Ssb::write_payload`].
///
/// Hot-path discipline: only the variants that carry raw user / free-text
/// String fields are walked; every other variant is a cheap no-op (the match
/// arm returns immediately). The free-text variant list is the FULL set
/// enumerated from the `SsbPayload` field audit — every variant carrying a
/// String field whose name is free prose (`prompt` / `message` / `content` /
/// `text` / `body` / `query` / `note` / `description` / `reason` / `summary` /
/// `output` / `title`) and is NOT a hash / id / path / enum-discriminator
/// (`*_hash`, `*_blake3`, `*_hex`, `*_kind`, `*_class`, `*_category`, `*_type`,
/// `intent`, `card_kind`, …). 82 free-text-bearing variants total.
///
/// FAIL-CLOSED-BY-CONSTRUCTION: the wildcard `_ => no-op` arm cannot, by
/// itself, force a future free-text variant to be considered (491 `SsbPayload`
/// variants make an exhaustive match impractical). The regression guard is the
/// reflective test `all_payloads_with_free_text_are_redacted` (this module's
/// `tests`): it injects a known secret into every classifier-free-text String
/// field of every `all_test_payloads()` fixture, runs `redact_payload`, and
/// asserts the raw secret is gone — so a new free-text variant shipped WITHOUT
/// an arm here FAILS that test (a missed field is a leak hole).
///
/// ANTI-RECURSION (DD6): the `SecretRedacted` advisory variant is clean by
/// construction (it only ever carries tokens + counts, never a secret), so it
/// is an explicit no-op. This is what lets the advisory row be emitted THROUGH
/// the SSB writer without the redactor recursing on its own output.
pub fn redact_payload(payload: &mut SsbPayload) -> PayloadRedaction {
    /// Redact one required `String` field in place; identity-only on a hit.
    fn apply(field: &mut String, ident: &str) -> PayloadRedaction {
        let (redacted, hits) = redact(field);
        if hits.is_empty() {
            PayloadRedaction::default()
        } else {
            *field = redacted;
            PayloadRedaction {
                field: ident.to_owned(),
                hits,
            }
        }
    }

    /// Redact one optional `String` field in place (no-op when `None` or no hit).
    fn apply_opt(field: &mut Option<String>, ident: &str) -> PayloadRedaction {
        match field {
            Some(s) => apply(s, ident),
            None => PayloadRedaction::default(),
        }
    }

    /// Merge several per-field redactions into one. The `field` identity is the
    /// comma-joined list of the fields that actually fired (identity only, never
    /// a value); hits are concatenated. Empty when nothing fired.
    fn merge(parts: Vec<PayloadRedaction>) -> PayloadRedaction {
        let mut fields: Vec<String> = Vec::new();
        let mut hits: Vec<RedactionHit> = Vec::new();
        for part in parts {
            if part.fired() {
                fields.push(part.field);
                hits.extend(part.hits);
            }
        }
        if hits.is_empty() {
            PayloadRedaction::default()
        } else {
            PayloadRedaction {
                field: fields.join(","),
                hits,
            }
        }
    }

    match payload {
        // Anti-recursion: the advisory row is clean by construction — NO-OP so
        // emitting it through the writer does not re-enter the redactor.
        SsbPayload::SecretRedacted(_) => PayloadRedaction::default(),

        // Documented raw user input (ssb.rs: "the raw user input (trimmed)").
        SsbPayload::IdeaFramingStarted(p) => apply(&mut p.prompt, "idea_framing_started.prompt"),

        // --- Free-text-bearing variants (FULL enumerated set, G-9 leak fix) ---
        SsbPayload::AgenticObservabilityEvent(p) => merge(vec![
            apply(&mut p.refusal_reason, "agentic_observability_event.refusal_reason"),
            apply_opt(&mut p.span_status_message, "agentic_observability_event.span_status_message"),
        ]),
        SsbPayload::AgentlessShortcutTaken(p) => apply(&mut p.reason, "agentless_shortcut_taken.reason"),
        SsbPayload::ArchiveEntryRetired(p) => apply(&mut p.retirement_reason, "archive_entry_retired.retirement_reason"),
        SsbPayload::ArgvApprovalRequired(p) => apply(&mut p.reason, "argv_approval_required.reason"),
        SsbPayload::ArgvDestructiveDenied(p) => apply(&mut p.reason, "argv_destructive_denied.reason"),
        SsbPayload::ArtifactSearched(p) => apply(&mut p.query, "artifact_searched.query"),
        SsbPayload::BackgroundTaskStarted(p) => apply(&mut p.command_summary, "background_task_started.command_summary"),
        SsbPayload::BranchClosed(p) => apply(&mut p.reason, "branch_closed.reason"),
        SsbPayload::BubbleUpTyped(p) => apply(&mut p.reason, "bubble_up_typed.reason"),
        SsbPayload::ChangeDecision(p) => apply_opt(&mut p.note, "change_decision.note"),
        SsbPayload::ChatMessage(p) => apply(&mut p.text, "chat_message.text"),
        SsbPayload::ChatReflectionCompleted(p) => apply(&mut p.note_preview, "chat_reflection_completed.note_preview"),
        SsbPayload::ChatResponse(p) => apply(&mut p.body, "chat_response.body"),
        SsbPayload::CheckerResultEmitted(p) => apply(&mut p.reason, "checker_result_emitted.reason"),
        SsbPayload::Claim(p) => apply(&mut p.claim_text, "claim.claim_text"),
        SsbPayload::CliSubprocessKilled(p) => apply(&mut p.reason, "cli_subprocess_killed.reason"),
        SsbPayload::ConfigWarning(p) => apply(&mut p.reason, "config_warning.reason"),
        SsbPayload::ConformalRefusalDecided(p) => apply_opt(&mut p.reason, "conformal_refusal_decided.reason"),
        SsbPayload::DagRepairRound(p) => apply(&mut p.reason, "dag_repair_round.reason"),
        SsbPayload::DeepModeEscalationChosen(p) => apply(&mut p.reason, "deep_mode_escalation_chosen.reason"),
        SsbPayload::DependencyResolved(p) => apply(&mut p.reason, "dependency_resolved.reason"),
        SsbPayload::DkgRebuildStarted(p) => apply(&mut p.trigger_reason, "dkg_rebuild_started.trigger_reason"),
        SsbPayload::EmpireDisconnected(p) => apply(&mut p.reason, "empire_disconnected.reason"),
        SsbPayload::EscalateToCloud(p) => apply(&mut p.trigger_reason, "escalate_to_cloud.trigger_reason"),
        SsbPayload::EvolutionWithheld(p) => apply(&mut p.reason, "evolution_withheld.reason"),
        SsbPayload::FailureRecorded(p) => apply(&mut p.reason, "failure_recorded.reason"),
        SsbPayload::FeasibilityEscalated(p) => apply(&mut p.reason, "feasibility_escalated.reason"),
        SsbPayload::FeasibilityReport(p) => apply_opt(&mut p.skip_reason, "feasibility_report.skip_reason"),
        SsbPayload::Finding(p) => apply(&mut p.message, "finding.message"),
        SsbPayload::FsmTransition(p) => apply(&mut p.reason, "fsm_transition.reason"),
        SsbPayload::GateDecision(p) => merge(vec![
            apply_opt(&mut p.reason, "gate_decision.reason"),
            apply(&mut p.evidence_summary, "gate_decision.evidence_summary"),
        ]),
        SsbPayload::GenerationModeSelected(p) => apply(&mut p.reason, "generation_mode_selected.reason"),
        SsbPayload::GitCommit(p) => apply(&mut p.message, "git_commit.message"),
        SsbPayload::HeldOutPromotionGate(p) => apply(&mut p.reason, "held_out_promotion_gate.reason"),
        SsbPayload::HookEvent(p) => apply(&mut p.message, "hook_event.message"),
        SsbPayload::IncidentRecordCreated(p) => apply(&mut p.title, "incident_record_created.title"),
        SsbPayload::IndexingProgress(p) => merge(vec![
            apply_opt(&mut p.message, "indexing_progress.message"),
            apply_opt(&mut p.error_message, "indexing_progress.error_message"),
        ]),
        SsbPayload::IntegrationCheckpointSuccess(p) => apply_opt(&mut p.failure_reason, "integration_checkpoint_success.failure_reason"),
        SsbPayload::InterventionRequest(p) => apply(&mut p.description, "intervention_request.description"),
        SsbPayload::InterventionResponse(p) => apply_opt(&mut p.user_note, "intervention_response.user_note"),
        SsbPayload::KnowledgeDegradation(p) => apply(&mut p.reason, "knowledge_degradation.reason"),
        SsbPayload::L3Proposal(p) => apply(&mut p.description, "l3_proposal.description"),
        SsbPayload::LlamaCppSubprocessKilled(p) => apply(&mut p.reason, "llama_cpp_subprocess_killed.reason"),
        SsbPayload::LockedKeysViolationDetected(p) => apply(&mut p.reason, "locked_keys_violation_detected.reason"),
        SsbPayload::McpLlmGuardBlocked(p) => apply(&mut p.reason, "mcp_llm_guard_blocked.reason"),
        SsbPayload::McpPinVersionRefused(p) => apply(&mut p.refusal_reason, "mcp_pin_version_refused.refusal_reason"),
        SsbPayload::McpServerDisconnected(p) => apply(&mut p.reason, "mcp_server_disconnected.reason"),
        SsbPayload::MediaHumanReviewRequired(p) => apply(&mut p.reason, "media_human_review_required.reason"),
        SsbPayload::MicroStepCreated(p) => apply(&mut p.reason, "micro_step_created.reason"),
        SsbPayload::MiniCriticVerdict(p) => apply(&mut p.reason, "mini_critic_verdict.reason"),
        SsbPayload::MlxSubprocessKilled(p) => apply(&mut p.reason, "mlx_subprocess_killed.reason"),
        SsbPayload::ModelSwap(p) => apply(&mut p.reason, "model_swap.reason"),
        SsbPayload::MucocoMutationApplied(p) => apply_opt(&mut p.skip_reason, "mucoco_mutation_applied.skip_reason"),
        SsbPayload::NovelStepDetected(p) => apply(&mut p.reason, "novel_step_detected.reason"),
        SsbPayload::PersonaReanchored(p) => apply(&mut p.reason, "persona_reanchored.reason"),
        SsbPayload::PhaseTransition(p) => apply(&mut p.evidence_summary, "phase_transition.evidence_summary"),
        SsbPayload::PlanUpdate(p) => apply(&mut p.change_summary, "plan_update.change_summary"),
        SsbPayload::PpmUnavailable(p) => apply(&mut p.reason, "ppm_unavailable.reason"),
        SsbPayload::ProposalGenerated(p) => apply(&mut p.new_row_summary, "proposal_generated.new_row_summary"),
        SsbPayload::ProposalRejected(p) => apply(&mut p.reason, "proposal_rejected.reason"),
        SsbPayload::ReasoningTrace(p) => apply(&mut p.content_summary, "reasoning_trace.content_summary"),
        SsbPayload::ReflectCandidate(p) => apply(&mut p.content, "reflect_candidate.content"),
        SsbPayload::ResearchPathChosen(p) => apply(&mut p.reason, "research_path_chosen.reason"),
        SsbPayload::ReviewSurfaceScored(p) => apply_opt(&mut p.diff_unavailable_reason, "review_surface_scored.diff_unavailable_reason"),
        SsbPayload::RouteBlocked(p) => apply(&mut p.reason, "route_blocked.reason"),
        SsbPayload::RouteDecisionEvent(p) => apply(&mut p.reason, "route_decision_event.reason"),
        SsbPayload::RuleTransition(p) => apply(&mut p.reason, "rule_transition.reason"),
        SsbPayload::RuntimeFallback(p) => apply(&mut p.reason, "runtime_fallback.reason"),
        SsbPayload::RuntimeHealthChanged(p) => {
            // `RuntimeHealth::Degraded` and `RuntimeHealth::Unavailable` both carry
            // a free-text `reason` field (runtime error strings / paths). `Ready`
            // has no free-text field — no-op for that variant.
            let mut parts: Vec<PayloadRedaction> = Vec::new();
            if let elai_providers::runtime::RuntimeHealth::Degraded { reason }
                | elai_providers::runtime::RuntimeHealth::Unavailable { reason } = &mut p.prev
            {
                parts.push(apply(reason, "runtime_health_changed.prev.reason"));
            }
            if let elai_providers::runtime::RuntimeHealth::Degraded { reason }
                | elai_providers::runtime::RuntimeHealth::Unavailable { reason } = &mut p.next
            {
                parts.push(apply(reason, "runtime_health_changed.next.reason"));
            }
            merge(parts)
        }
        SsbPayload::SearchTriggerFired(p) => apply(&mut p.reason, "search_trigger_fired.reason"),
        SsbPayload::SkillLibraryEvent(p) => merge(vec![
            apply(&mut p.description, "skill_library_event.description"),
            apply(&mut p.reason, "skill_library_event.reason"),
        ]),
        SsbPayload::SnapshotEvicted(p) => apply(&mut p.reason, "snapshot_evicted.reason"),
        SsbPayload::SteeringComputeFailed(p) => apply(&mut p.reason, "steering_compute_failed.reason"),
        SsbPayload::StreamingEarlyTerminated(p) => merge(vec![
            apply(&mut p.reason, "streaming_early_terminated.reason"),
            apply_opt(&mut p.reason_detail, "streaming_early_terminated.reason_detail"),
        ]),
        SsbPayload::StreamingTimingRecorded(p) => apply(&mut p.stop_reason, "streaming_timing_recorded.stop_reason"),
        SsbPayload::SycophancyProbeDispatched(p) => apply(&mut p.user_message, "sycophancy_probe_dispatched.user_message"),
        SsbPayload::TemplateAutoReverted(p) => apply(&mut p.retirement_reason, "template_auto_reverted.retirement_reason"),
        SsbPayload::TemplateCandidateRejected(p) => apply(&mut p.rejection_reason, "template_candidate_rejected.rejection_reason"),
        SsbPayload::ToolDispatch(p) => apply(&mut p.args_summary, "tool_dispatch.args_summary"),
        SsbPayload::ToolResult(p) => apply(&mut p.output_summary, "tool_result.output_summary"),
        SsbPayload::UntrustedMemoryWriteBlocked(p) => apply(&mut p.write_summary, "untrusted_memory_write_blocked.write_summary"),
        SsbPayload::VerifierConsensusFail(p) => apply(&mut p.reason, "verifier_consensus_fail.reason"),
        SsbPayload::YarnRefused(p) => apply(&mut p.reason, "yarn_refused.reason"),

        // Every other variant carries no raw free-text String field
        // (hashes / enums / ints / structured tags / free-form details_json) —
        // quick no-op on the hot path. A NEW free-text variant added without an
        // arm above is caught by `all_payloads_with_free_text_are_redacted`
        // (this module's tests): that test fails on any classifier-free-text
        // field whose raw secret survives `redact_payload` — a missed field is a
        // leak hole, and the test makes the miss RED, not silent.
        _ => PayloadRedaction::default(),
    }
}

/// Build the field summary the `secret_redacted` advisory row records: a sorted,
/// deduped count of `(label, count)` across the hits. Carries NO secret value.
#[must_use]
pub fn summarize_hits(hits: &[RedactionHit]) -> Vec<(String, u32)> {
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<&'static str, u32> = BTreeMap::new();
    for hit in hits {
        *counts.entry(hit.kind.label()).or_insert(0) += 1;
    }
    counts.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()
}

// ===========================================================================
// G-9 retention purge (DD4 / DD5) — file-granular TTL over transcript + archive
// ===========================================================================
//
// HONEST SCOPE (DD4 / GD-5): `.elai_cc/sessions/` holds ONE shared `ssb.db`
// (sessions multiplexed by a `session_id` COLUMN, runtime_setup.rs) — there is
// NO per-session ssb.db file, so deleting ssb.db would nuke EVERY session. This
// purge therefore NEVER touches ssb.db. It is file-granular ONLY over:
//   (1) the per-session managed-transcript files `<id>.jsonl` / `<id>.json`
//       (session_control.rs), and
//   (2) the F-13 forensic archive copies `workflow/sessions/<id>/archive/*.sqlite`.
// The shared ssb.db rows for an expired session are protected by WRITE-TIME
// redaction ([`redact_payload`]), NOT by deletion. We do not claim otherwise.
//
// SAFETY (DD5): file deletion is irreversible and there is NO persistent
// active-session registry (the live session_id is an ephemeral PID). The purge
// is therefore:
//   * DEFAULT-OFF behind [`RETENTION_PURGE_ENV`] (`ELAI_RETENTION_PURGE_ENABLE`),
//     registered in wire-dark-allowlist.toml (type=kill_switch, R9 default-off
//     for a destructive op);
//   * run IN-PROCESS at run-start so it can self-EXCLUDE the live `session_id`;
//   * cross-checked against a live index-lock holder (lock.rs::read_lock_holder);
//   * conservative on clock skew: a file whose mtime is in the future, zero, or
//     unparsable, or within the [`PURGE_GRACE_WINDOW_SECS`] window, is KEPT.
// The archive TTL is intentionally LONGER than the transcript TTL: the archive
// copy is the last forensic backup.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Default-OFF kill-switch for the destructive retention purge. Registered in
/// wire-dark-allowlist.toml (R9). Truthy values: `1`/`true`/`yes`/`on`.
pub const RETENTION_PURGE_ENV: &str = "ELAI_RETENTION_PURGE_ENABLE";

/// Seconds per day (named constant; no magic 86_400).
const SECONDS_PER_DAY: u64 = 86_400;

/// Default TTL for per-session transcript files (`.jsonl`/`.json`): 7 days.
/// Mirrors the oracle-cache T2 tier shape.
pub const TRANSCRIPT_TTL_DAYS: u64 = 7;

/// Default TTL for the F-13 forensic archive copies: 30 days (longer than the
/// transcript TTL — the archive is the last forensic backup, DD5).
pub const ARCHIVE_TTL_DAYS: u64 = 30;

/// Clock-skew grace window. A file modified within this many seconds of "now"
/// is KEPT regardless of TTL — guards against an in-flight write or a skewed
/// filesystem clock causing a live file to be judged expired.
const PURGE_GRACE_WINDOW_SECS: u64 = 3_600;

/// Tier TTLs for a purge run. Days, converted to [`Duration`] internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionPolicy {
    /// TTL for managed-transcript files (`<id>.jsonl` / `<id>.json`).
    pub transcript_ttl: Duration,
    /// TTL for the F-13 archive `*.sqlite` copies (intentionally longer).
    pub archive_ttl: Duration,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            transcript_ttl: Duration::from_secs(TRANSCRIPT_TTL_DAYS * SECONDS_PER_DAY),
            archive_ttl: Duration::from_secs(ARCHIVE_TTL_DAYS * SECONDS_PER_DAY),
        }
    }
}

/// Outcome of one purge sweep. Deterministic + secret-free; suitable for an SSB
/// row or a dashboard read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurgeReport {
    /// Whether the purge was enabled at all (`RETENTION_PURGE_ENV` truthy).
    pub enabled: bool,
    /// Transcript files deleted.
    pub transcripts_deleted: Vec<PathBuf>,
    /// Archive `*.sqlite` files deleted.
    pub archives_deleted: Vec<PathBuf>,
    /// Files KEPT despite age, with the reason (active session, grace window,
    /// lock holder, future/zero/unparsable mtime). Diagnostic only.
    pub kept_with_reason: Vec<(PathBuf, &'static str)>,
}

/// Managed-session transcript file extensions. Mirrors
/// `session_control::{PRIMARY_SESSION_EXTENSION, LEGACY_SESSION_EXTENSION}`
/// (session_control.rs:12-13). Inlined here because `session_control` is a
/// `#[cfg(test)]`-only module (lib.rs:292) and the purge runs in production.
const TRANSCRIPT_EXTENSIONS: [&str; 2] = ["jsonl", "json"];

/// `true` iff `path` is a managed-session transcript (`.jsonl` / `.json`). This
/// is the guard that keeps the purge from ever considering `ssb.db` (or any
/// other file) — only managed-session transcripts are eligible.
fn is_managed_session_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| TRANSCRIPT_EXTENSIONS.iter().any(|known| *known == ext))
}

/// `true` iff `var` is set to a recognised truthy value. Mirrors the
/// `session_archive::env_is_truthy` convention (named, no magic literals).
fn env_is_truthy(var: &str) -> bool {
    match std::env::var(var) {
        Ok(v) => {
            let t = v.trim().to_ascii_lowercase();
            t == "1" || t == "true" || t == "yes" || t == "on"
        }
        Err(_) => false,
    }
}

/// `true` iff the destructive retention purge is enabled. Default OFF.
#[must_use]
pub fn retention_purge_enabled() -> bool {
    env_is_truthy(RETENTION_PURGE_ENV)
}

/// File-granular retention purge over the managed-transcript files under
/// `sessions_dir` and the F-13 archive copies under `archive_root`.
///
/// `now_ms` is the injected wall-clock (milliseconds since UNIX epoch) — pass
/// `clock.now_ms()` from an [`elai_core::clock::ClockProvider`] so the TTL
/// comparison is deterministic + testable and never calls `SystemTime::now`
/// directly (the ELAI entropy-quarantine rule).
///
/// `active_session_id` is the live session — its files are NEVER deleted.
/// `project_root` is used to cross-check a live index-lock holder; when a lock
/// holder is present the sweep KEEPS everything (a session is running).
///
/// FAIL-SAFE: when the purge is disabled the function returns an empty report
/// with `enabled = false` and performs ZERO filesystem mutation. A file with a
/// future/zero/unparsable mtime, or one inside the grace window, is KEPT.
///
/// This function NEVER touches `ssb.db` (it only enumerates managed-session
/// files via `is_managed_session_file` and the archive `*.sqlite` copies).
pub fn purge_expired(
    sessions_dir: &Path,
    archive_root: &Path,
    policy: RetentionPolicy,
    now_ms: u128,
    active_session_id: &str,
    project_root: &Path,
) -> PurgeReport {
    let mut report = PurgeReport {
        enabled: retention_purge_enabled(),
        ..Default::default()
    };
    if !report.enabled {
        return report;
    }

    // If any index-lock is held, a session is actively running: KEEP everything
    // (conservative — we cannot enumerate which other sessions are live).
    if crate::lock::read_lock_holder(project_root, crate::lock::LockScope::Index).is_some() {
        report
            .kept_with_reason
            .push((sessions_dir.to_path_buf(), "index_lock_held"));
        return report;
    }

    // --- Transcript files (.jsonl / .json) ---
    if let Ok(entries) = std::fs::read_dir(sessions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !is_managed_session_file(&path) {
                continue; // never ssb.db — only managed-session files
            }
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            if id == active_session_id {
                report
                    .kept_with_reason
                    .push((path, "active_session"));
                continue;
            }
            match classify_age(&path, now_ms, policy.transcript_ttl) {
                AgeVerdict::Expired => match std::fs::remove_file(&path) {
                    Ok(()) => report.transcripts_deleted.push(path),
                    Err(_) => report.kept_with_reason.push((path, "delete_failed")),
                },
                AgeVerdict::Keep(reason) => report.kept_with_reason.push((path, reason)),
            }
        }
    }

    // --- Archive copies (workflow/sessions/<id>/archive/*.sqlite) ---
    purge_archive_copies(archive_root, policy.archive_ttl, now_ms, active_session_id, &mut report);

    report
}

/// Verdict for a single file's age vs a TTL, with a KEEP reason for diagnostics.
enum AgeVerdict {
    Expired,
    Keep(&'static str),
}

/// Classify a file's mtime against `ttl` relative to `now_ms`. Conservative:
/// a missing/zero/future mtime, or one inside the grace window, is KEPT.
fn classify_age(path: &Path, now_ms: u128, ttl: Duration) -> AgeVerdict {
    let mtime_ms = match file_mtime_ms(path) {
        Some(0) | None => return AgeVerdict::Keep("mtime_unparsable"),
        Some(ms) => ms,
    };
    if mtime_ms > now_ms {
        return AgeVerdict::Keep("mtime_future"); // clock skew — never delete
    }
    let age_ms = now_ms.saturating_sub(mtime_ms);
    let grace_ms = u128::from(PURGE_GRACE_WINDOW_SECS) * 1_000;
    if age_ms < grace_ms {
        return AgeVerdict::Keep("grace_window");
    }
    let ttl_ms = u128::from(ttl.as_secs()) * 1_000;
    if age_ms > ttl_ms {
        AgeVerdict::Expired
    } else {
        AgeVerdict::Keep("within_ttl")
    }
}

/// Read a file's mtime as milliseconds since UNIX epoch. `None` if unavailable
/// (treated as KEEP by [`classify_age`]).
fn file_mtime_ms(path: &Path) -> Option<u128> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis())
}

/// Walk `archive_root/<session_id>/archive/*.sqlite` and purge expired copies.
/// Skips the active session's archive dir. Only `.sqlite` files are eligible.
fn purge_archive_copies(
    archive_root: &Path,
    archive_ttl: Duration,
    now_ms: u128,
    active_session_id: &str,
    report: &mut PurgeReport,
) {
    let session_dirs = match std::fs::read_dir(archive_root) {
        Ok(d) => d,
        Err(_) => return, // no archive root yet — nothing to purge
    };
    for session_dir in session_dirs.flatten() {
        let session_path = session_dir.path();
        if !session_path.is_dir() {
            continue;
        }
        let id = session_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if id == active_session_id {
            report
                .kept_with_reason
                .push((session_path, "active_session"));
            continue;
        }
        let archive_dir = session_path.join("archive");
        let copies = match std::fs::read_dir(&archive_dir) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for copy in copies.flatten() {
            let copy_path = copy.path();
            let is_sqlite = copy_path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("sqlite"));
            if !is_sqlite {
                continue; // never anything but the .sqlite archive copies
            }
            match classify_age(&copy_path, now_ms, archive_ttl) {
                AgeVerdict::Expired => match std::fs::remove_file(&copy_path) {
                    Ok(()) => report.archives_deleted.push(copy_path),
                    Err(_) => report.kept_with_reason.push((copy_path, "delete_failed")),
                },
                AgeVerdict::Keep(reason) => report.kept_with_reason.push((copy_path, reason)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AWS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";
    const EMAIL: &str = "alice@example.com";

    #[test]
    fn redacts_aws_key_with_stable_token() {
        let (out, hits) = redact(&format!("key is {AWS_KEY} ok"));
        assert!(!out.contains(AWS_KEY), "raw AWS key must be gone");
        assert!(out.contains("[REDACTED:aws_access_key_id:"));
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn redacts_email() {
        let (out, hits) = redact(&format!("ping {EMAIL} please"));
        assert!(!out.contains(EMAIL));
        assert!(out.contains("[REDACTED:email:"));
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn same_secret_same_token() {
        let (a, _) = redact(AWS_KEY);
        let (b, _) = redact(&format!("other {AWS_KEY}"));
        // Extract the token (between the brackets) and compare the blake3 tail.
        assert!(a.contains("aws_access_key_id"));
        assert!(b.contains(&a));
    }

    #[test]
    fn benign_text_unchanged() {
        let benign = "version 1.2.3.4 build, uuid 550e8400-e29b-41d4-a716-446655440000";
        let (out, hits) = redact(benign);
        assert_eq!(out, benign, "benign text must be byte-identical");
        assert!(hits.is_empty());
    }

    #[test]
    fn fail_closed_empty_match_never_raw() {
        // token_for on an empty match yields the fail-closed placeholder, never
        // the raw value.
        assert_eq!(token_for(SecretKind::Email, ""), FAIL_CLOSED_TOKEN);
    }
}
