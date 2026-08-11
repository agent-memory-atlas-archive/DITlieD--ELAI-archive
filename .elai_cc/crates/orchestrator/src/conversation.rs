use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use serde_json::{Map, Value};
use crate::telemetry_types::SessionTracer;

use crate::architecture::approval_fatigue_guard::{
    approval_fatigue_guard_enabled, ApprovalFatigueGuard, ApprovalFatigueVerdict, ApprovalKind,
};
use crate::context_policy::{
    effective_context_window, pressure_level, regime_policy, ContextPolicy, PressureLevel,
};
use crate::context_budget::{
    preassess_observation_size, ContextBudget, ExecutionMode, PreassessMode,
};
use crate::behavior_steering_ssb::emit_behavior_steering_event;
use crate::ssb::BehaviorSteeringKind;
use crate::dispatcher::BackendMode;
use crate::compact::{
    compact_session, estimate_session_tokens, CompactionConfig,
    CompactionResult,
};
use crate::lossless_compaction_gate::enforce_lossless_compaction;
use crate::compaction_ladder::{
    compaction_ladder_enabled, run_compaction_ladder, CompactionLadder, LadderEventEmitter,
    NullEmitter,
};
use crate::ssb::{ContextReductionPayload, OutputClassifierFlaggedPayload, SsbPayload};
use crate::output_postprocessor::{self, OutputConfig};
use crate::config::RuntimeFeatureConfig;
use crate::hooks::{HookAbortSignal, HookProgressReporter, HookRunResult, HookRunner};
use crate::permissions::{
    PermissionContext, PermissionOutcome, PermissionPolicy, PermissionPrompter,
};
use crate::session::{ContentBlock, ConversationMessage, Session};
use crate::usage::{TokenUsage, UsageTracker};

const DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD: u32 = 100_000;
const AUTO_COMPACTION_THRESHOLD_ENV_VAR: &str = "CLAUDE_CODE_AUTO_COMPACT_INPUT_TOKENS";
pub const ELAI_J95_ADAPTIVE_REDUCE_ON_COMPACT_VAR: &str =
    "ELAI_J95_ADAPTIVE_REDUCE_ON_COMPACT";

/// Default OFF. Set to "1" to enable adaptive reduction before auto-compaction.
pub fn j95_adaptive_reduce_on_compact_enabled() -> bool {
    std::env::var(ELAI_J95_ADAPTIVE_REDUCE_ON_COMPACT_VAR)
        .map(|value| value == "1")
        .unwrap_or(false)
}

/// Fully assembled request payload sent to the upstream model client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiRequest {
    pub system_prompt: Vec<String>,
    pub messages: Vec<ConversationMessage>,
}

/// Streamed events emitted while processing a single assistant turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantEvent {
    TextDelta(String),
    ToolUse {
        id: String,
        name: String,
        input: String,
    },
    Usage(TokenUsage),
    PromptCache(PromptCacheEvent),
    MessageStop,
}

/// Prompt-cache telemetry captured from the provider response stream.
///
/// Phase A6 — determinism plumbing: `fingerprint_hash` exposes the
/// `RequestFingerprint` digest (providers/src/prompt_cache.rs:273-289) onto the
/// telemetry surface so downstream consumers (dashboard, audits) can correlate
/// a `PromptCacheEvent` with the exact request shape that produced it. The
/// underlying `RequestFingerprint` already mixes model + system + tools +
/// messages hashes (prompt_cache.rs is LOCKED — A6 does not modify it), so this
/// field is purely additive: a key the dashboard can group/filter on. Actual
/// KV-cache REUSE keyed on this hash is deferred (remote-sidecar handoff makes
/// the cache-locality decision non-trivial). `None` preserves pre-A6 emitters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptCacheEvent {
    pub unexpected: bool,
    pub reason: String,
    pub previous_cache_read_input_tokens: u32,
    pub current_cache_read_input_tokens: u32,
    pub token_drop: u32,
    /// Phase A6: optional `RequestFingerprint` digest associated with the
    /// request that produced this event. Builders set it via
    /// `with_fingerprint_hash`; pre-A6 emitters leave it `None`. NOT serialised
    /// to wire (telemetry-only field — `PromptCacheEvent` has no serde derive).
    pub fingerprint_hash: Option<u64>,
}

impl PromptCacheEvent {
    /// Attach a `RequestFingerprint` digest (computed by the caller from the
    /// LOCKED `providers/src/prompt_cache.rs` machinery) to a freshly built
    /// `PromptCacheEvent`. Builder-style so existing emitter sites can opt in
    /// incrementally without refactoring their constructor calls.
    #[must_use]
    pub fn with_fingerprint_hash(mut self, hash: u64) -> Self {
        self.fingerprint_hash = Some(hash);
        self
    }
}

/// Minimal streaming API contract required by [`ConversationRuntime`].
pub trait ApiClient {
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError>;
}

/// Trait implemented by tool dispatchers that execute model-requested tools.
pub trait ToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, ToolError>;
}

/// Error returned when a tool invocation fails locally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolError {
    message: String,
}

impl ToolError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ToolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ToolError {}

/// Error returned when a conversation turn cannot be completed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

/// Summary of one completed runtime turn, including tool results and usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnSummary {
    pub assistant_messages: Vec<ConversationMessage>,
    pub tool_results: Vec<ConversationMessage>,
    pub prompt_cache_events: Vec<PromptCacheEvent>,
    pub iterations: usize,
    pub usage: TokenUsage,
    pub auto_compaction: Option<AutoCompactionEvent>,
}

/// Details about automatic session compaction applied during a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoCompactionEvent {
    pub removed_message_count: usize,
}

/// Coordinates the model loop, tool execution, hooks, and session updates.
pub struct ConversationRuntime<C, T> {
    session: Session,
    api_client: C,
    tool_executor: T,
    permission_policy: PermissionPolicy,
    system_prompt: Vec<String>,
    max_iterations: usize,
    usage_tracker: UsageTracker,
    hook_runner: HookRunner,
    auto_compaction_input_tokens_threshold: u32,
    context_backend_mode: BackendMode,
    hook_abort_signal: HookAbortSignal,
    hook_progress_reporter: Option<Box<dyn HookProgressReporter>>,
    session_tracer: Option<SessionTracer>,
    output_config: OutputConfig,
    /// E-25 Tranche L (F.1.c) -- per-session ApprovalFatigueGuard.
    /// Double-gated: this field must be `Some(_)` AND
    /// [`approval_fatigue_guard_enabled`] must read truthy from
    /// `ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE`. Default `None` keeps
    /// the guard dormant across every existing constructor and test
    /// fixture (Tranche F precedent: wire-dark, opt-in builder).
    approval_fatigue: Option<ApprovalFatigueGuard>,
    /// E-25 stubs F1c / F1a -- optional SSB writer for guard emits.
    /// When `Some`, the `ApprovalFatigueDetected` and
    /// `BashComplexityBlocked` SSB rows are emitted at the refusal sites
    /// in [`Self::run_turn`]. When `None`, refusals still fire but no SSB
    /// row is written (legacy / unit-test path). Wire-in via
    /// [`Self::with_ssb_writer`]; default `None`.
    ssb_writer: Option<std::sync::Arc<crate::ssb::Ssb>>,
    context_budget: Option<ContextBudget>,
    /// L-16 CC-2: the active projection epoch. `None` = no compaction has been
    /// warranted yet and the provider-facing array is the verbatim log. `Some(cfg)`
    /// = every later turn folds the whole immutable log at `cfg` instead of
    /// rewriting `session.messages`.
    projection_epoch: Option<CompactionConfig>,
    freeze_epoch: Option<crate::context_projection::FreezeEpoch>,
    last_freeze_decision: Option<crate::context_projection::FreezeDecision>,
}

/// L-16 CC-2: the projection budget an oversized log warrants on its own, with no
/// prior compaction event. Pure in (log, mode): the threshold comes from the
/// shipped H-50 regime table and the decision from the shipped compaction budget
/// test, so no second budget policy is invented here.
fn projection_epoch_for_log(session: &Session, mode: BackendMode) -> Option<CompactionConfig> {
    let config = CompactionConfig {
        max_estimated_tokens: crate::context_policy::trim_threshold_for_mode(mode) as usize,
        ..CompactionConfig::default()
    };
    crate::compact::should_compact(session, config).then_some(config)
}

impl<C, T> ConversationRuntime<C, T>
where
    C: ApiClient,
    T: ToolExecutor,
{
    #[must_use]
    pub fn new(
        session: Session,
        api_client: C,
        tool_executor: T,
        permission_policy: PermissionPolicy,
        system_prompt: Vec<String>,
    ) -> Self {
        Self::new_with_features(
            session,
            api_client,
            tool_executor,
            permission_policy,
            system_prompt,
            &RuntimeFeatureConfig::default(),
        )
    }

    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new_with_features(
        session: Session,
        api_client: C,
        tool_executor: T,
        permission_policy: PermissionPolicy,
        system_prompt: Vec<String>,
        feature_config: &RuntimeFeatureConfig,
    ) -> Self {
        let usage_tracker = {
            let mut tracker = UsageTracker::new();
            for message in &session.messages {
                if let Some(usage) = message.usage {
                    tracker.record(usage);
                }
            }
            tracker
        };
        Self {
            session,
            api_client,
            tool_executor,
            permission_policy,
            system_prompt,
            max_iterations: usize::MAX,
            usage_tracker,
            hook_runner: HookRunner::from_feature_config(feature_config),
            auto_compaction_input_tokens_threshold: auto_compaction_threshold_from_env(),
            context_backend_mode: BackendMode::Api,
            hook_abort_signal: HookAbortSignal::default(),
            hook_progress_reporter: None,
            session_tracer: None,
            output_config: OutputConfig::default(),
            // E-25 Tranche L (F.1.c): guard is opt-in via
            // `with_approval_fatigue` AND env-gated by
            // ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE. Default None keeps
            // the field dormant across every existing constructor.
            approval_fatigue: None,
            // E-25 stubs F1c / F1a: SSB writer is opt-in via
            // `with_ssb_writer`. Default None keeps emission dormant
            // across every existing constructor and test fixture.
            ssb_writer: None,
            context_budget: None,
            projection_epoch: None,
            freeze_epoch: None,
            last_freeze_decision: None,
        }
    }

    #[must_use]
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    #[must_use]
    pub fn with_auto_compaction_input_tokens_threshold(mut self, threshold: u32) -> Self {
        self.auto_compaction_input_tokens_threshold = threshold;
        self
    }

    #[must_use]
    pub fn with_context_backend_mode(mut self, mode: BackendMode) -> Self {
        self.context_backend_mode = mode;
        self
    }

    #[must_use]
    pub fn with_hook_abort_signal(mut self, hook_abort_signal: HookAbortSignal) -> Self {
        self.hook_abort_signal = hook_abort_signal;
        self
    }

    #[must_use]
    pub fn with_hook_progress_reporter(
        mut self,
        hook_progress_reporter: Box<dyn HookProgressReporter>,
    ) -> Self {
        self.hook_progress_reporter = Some(hook_progress_reporter);
        self
    }

    #[must_use]
    pub fn with_session_tracer(mut self, session_tracer: SessionTracer) -> Self {
        self.session_tracer = Some(session_tracer);
        self
    }

    #[must_use]
    pub fn with_output_config(mut self, output_config: OutputConfig) -> Self {
        self.output_config = output_config;
        self
    }

    /// E-25 Tranche L (F.1.c) -- install a per-session
    /// [`ApprovalFatigueGuard`]. The guard is double-gated:
    ///   1. this builder must be called (field is `Some(_)`), AND
    ///   2. [`approval_fatigue_guard_enabled`] must read truthy from
    ///      the `ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE` env var.
    /// When both gates pass, the observation block at the
    /// `authorize_with_context` site records each permission decision
    /// into the guard's rolling window and overrides the outcome to
    /// `Deny` on a `Refused` verdict (force reconfirmation).
    #[must_use]
    pub fn with_approval_fatigue(mut self, guard: ApprovalFatigueGuard) -> Self {
        self.approval_fatigue = Some(guard);
        self
    }

    /// E-25 stubs F1c / F1a -- install an SSB writer for guard emits.
    /// When this builder is called and the guard env-gate is truthy, a
    /// refusal by either the `ApprovalFatigueGuard` or the
    /// `BashComplexityGuard` will emit a `ClaudeCodeArchitectureEvent`
    /// SSB row via a fire-and-forget background thread (same pattern as
    /// `crates/cli/src/live_cli.rs`). Default `None` keeps the field
    /// dormant across every existing constructor and test fixture.
    #[must_use]
    pub fn with_ssb_writer(mut self, ssb: std::sync::Arc<crate::ssb::Ssb>) -> Self {
        self.ssb_writer = Some(ssb);
        self
    }

    #[must_use]
    pub fn with_context_budget(mut self, budget: ContextBudget) -> Self {
        self.context_budget = Some(budget);
        self
    }

    /// E-25 Tranche L (F.1.c) -- test-only read accessor for the
    /// `approval_fatigue` field. Used by contract tests to assert the
    /// builder + default-None semantics. Gated under `test-support` so
    /// the production surface stays minimal.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn approval_fatigue_is_installed(&self) -> bool {
        self.approval_fatigue.is_some()
    }

    fn run_pre_tool_use_hook(&mut self, tool_name: &str, input: &str) -> HookRunResult {
        if let Some(reporter) = self.hook_progress_reporter.as_mut() {
            self.hook_runner.run_pre_tool_use_with_context(
                tool_name,
                input,
                Some(&self.hook_abort_signal),
                Some(reporter.as_mut()),
            )
        } else {
            self.hook_runner.run_pre_tool_use_with_context(
                tool_name,
                input,
                Some(&self.hook_abort_signal),
                None,
            )
        }
    }

    fn run_post_tool_use_hook(
        &mut self,
        tool_name: &str,
        input: &str,
        output: &mut String,
        is_error: bool,
    ) -> HookRunResult {
        // E-37 U1: sanitize tool output for non-allowlisted markdown images
        // (markdown-image exfil channel, OWASP LLM06 + CVE-2025-32711
        // CometJacking family) and classify for script / role-tag
        // impersonation (OWASP LLM01). Empty allowlist = strip every
        // http(s) markdown image; the orchestrator-side allowlist is a
        // future U-thread, the strip is fail-closed on the unknown host.
        // Pure stdlib-only fns; no LLM classifier, no regex crate.
        let allowlist: Vec<String> = Vec::new();
        let sanitized =
            elai_safety::output_sanitizer::strip_markdown_images(output.as_str(), &allowlist);
        let verdict =
            elai_safety::output_sanitizer::classify_output(&sanitized.cleaned);
        let flagged = !matches!(
            verdict,
            elai_safety::output_sanitizer::OutputVerdict::Clean
        );
        if sanitized.stripped_count > 0 || flagged {
            if let Some(ssb) = self.ssb_writer.clone() {
                let patterns = match &verdict {
                    elai_safety::output_sanitizer::OutputVerdict::Flagged { patterns } => {
                        patterns.clone()
                    }
                    elai_safety::output_sanitizer::OutputVerdict::Clean => Vec::new(),
                };
                let payload = SsbPayload::OutputClassifierFlagged(
                    OutputClassifierFlaggedPayload {
                        tool_name: tool_name.to_string(),
                        stripped_count: sanitized.stripped_count,
                        stripped_hosts: sanitized.stripped_hosts.clone(),
                        patterns,
                    },
                );
                std::thread::spawn(move || {
                    if let Ok(rt) =
                        tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                    {
                        let _ = rt.block_on(ssb.write_payload(
                            "conversation.output_classifier",
                            0,
                            payload,
                            Vec::new(),
                        ));
                    }
                });
            }
        }
        let cleaned = sanitized.cleaned;
        *output = cleaned;
        if let Some(reporter) = self.hook_progress_reporter.as_mut() {
            self.hook_runner.run_post_tool_use_with_context(
                tool_name,
                input,
                output.as_str(),
                is_error,
                Some(&self.hook_abort_signal),
                Some(reporter.as_mut()),
            )
        } else {
            self.hook_runner.run_post_tool_use_with_context(
                tool_name,
                input,
                output.as_str(),
                is_error,
                Some(&self.hook_abort_signal),
                None,
            )
        }
    }

    fn run_post_tool_use_failure_hook(
        &mut self,
        tool_name: &str,
        input: &str,
        output: &str,
    ) -> HookRunResult {
        if let Some(reporter) = self.hook_progress_reporter.as_mut() {
            self.hook_runner.run_post_tool_use_failure_with_context(
                tool_name,
                input,
                output,
                Some(&self.hook_abort_signal),
                Some(reporter.as_mut()),
            )
        } else {
            self.hook_runner.run_post_tool_use_failure_with_context(
                tool_name,
                input,
                output,
                Some(&self.hook_abort_signal),
                None,
            )
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn run_turn(
        &mut self,
        user_input: impl Into<String>,
        mut prompter: Option<&mut dyn PermissionPrompter>,
    ) -> Result<TurnSummary, RuntimeError> {
        let user_input = user_input.into();
        self.record_turn_started(&user_input);
        self.session
            .push_user_text(user_input)
            .map_err(|error| RuntimeError::new(error.to_string()))?;

        let mut assistant_messages = Vec::new();
        let mut tool_results = Vec::new();
        let mut prompt_cache_events = Vec::new();
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > self.max_iterations {
                let error = RuntimeError::new(
                    "conversation loop exceeded the maximum number of iterations",
                );
                self.record_turn_failed(iterations, &error);
                return Err(error);
            }

            let request = ApiRequest {
                system_prompt: self.system_prompt.clone(),
                messages: self.freeze_gated_view_messages(),
            };
            let events = match self.api_client.stream(request) {
                Ok(events) => events,
                Err(error) => {
                    self.record_turn_failed(iterations, &error);
                    return Err(error);
                }
            };
            let (mut assistant_message, usage, turn_prompt_cache_events) =
                match build_assistant_message(events) {
                    Ok(result) => result,
                    Err(error) => {
                        self.record_turn_failed(iterations, &error);
                        return Err(error);
                    }
                };

            // A-30: postprocess text blocks (emoji strip, sycophancy, length).
            postprocess_message_blocks(&mut assistant_message.blocks, &self.output_config);

            if let Some(usage) = usage {
                self.usage_tracker.record(usage);
            }
            prompt_cache_events.extend(turn_prompt_cache_events);
            let pending_tool_uses = assistant_message
                .blocks
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::ToolUse { id, name, input } => {
                        Some((id.clone(), name.clone(), input.clone()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            self.record_assistant_iteration(
                iterations,
                &assistant_message,
                pending_tool_uses.len(),
            );

            self.session
                .push_message(assistant_message.clone())
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            assistant_messages.push(assistant_message);

            if pending_tool_uses.is_empty() {
                break;
            }

            for (tool_use_id, tool_name, input) in pending_tool_uses {
                let pre_hook_result = self.run_pre_tool_use_hook(&tool_name, &input);
                let effective_input = pre_hook_result
                    .updated_input()
                    .map_or_else(|| input.clone(), ToOwned::to_owned);
                let permission_context = PermissionContext::new(
                    pre_hook_result.permission_override(),
                    pre_hook_result.permission_reason().map(ToOwned::to_owned),
                );

                let mut permission_outcome = if pre_hook_result.is_cancelled() {
                    PermissionOutcome::Deny {
                        reason: format_hook_message(
                            &pre_hook_result,
                            &format!("PreToolUse hook cancelled tool `{tool_name}`"),
                        ),
                    }
                } else if pre_hook_result.is_failed() {
                    PermissionOutcome::Deny {
                        reason: format_hook_message(
                            &pre_hook_result,
                            &format!("PreToolUse hook failed for tool `{tool_name}`"),
                        ),
                    }
                } else if pre_hook_result.is_denied() {
                    PermissionOutcome::Deny {
                        reason: format_hook_message(
                            &pre_hook_result,
                            &format!("PreToolUse hook denied tool `{tool_name}`"),
                        ),
                    }
                } else if let Some(prompt) = prompter.as_mut() {
                    self.permission_policy.authorize_with_context(
                        &tool_name,
                        &effective_input,
                        &permission_context,
                        Some(*prompt),
                    )
                } else {
                    self.permission_policy.authorize_with_context(
                        &tool_name,
                        &effective_input,
                        &permission_context,
                        None,
                    )
                };

                // E-25 Tranche L (F.1.c) -- ApprovalFatigueGuard observation
                // + verdict-override. Per-session rolling-window detector.
                // Double-gated: field-Some AND env truthy. When tripped,
                // override the outcome to Deny to force reconfirmation on
                // the current prompt (matches substrate doc-comment L5-8).
                // Stub F1c-approval-fatigue-emit: emit SSB row when ssb_writer
                // is wired and the verdict is Refused. Fire-and-forget via
                // background thread (run_turn is sync; see live_cli.rs pattern).
                if let Some(guard) = self.approval_fatigue.as_mut() {
                    if approval_fatigue_guard_enabled() {
                        let kind = match &permission_outcome {
                            PermissionOutcome::Allow => ApprovalKind::Allow,
                            PermissionOutcome::Deny { .. } => ApprovalKind::Deny,
                        };
                        let verdict = guard.record_and_check(kind);
                        if let ApprovalFatigueVerdict::Refused { reason, .. } = &verdict {
                            permission_outcome = PermissionOutcome::Deny {
                                reason: reason.clone(),
                            };
                            // F1c emit: build payload using guard state + verdict,
                            // then fire-and-forget into the SSB writer.
                            if let Some(ssb) = self.ssb_writer.clone() {
                                let ts_ms = i64::try_from(
                                    std::time::SystemTime::now() // entropy-allow: ssb-emit-observability-ts-ms
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_millis())
                                        .unwrap_or(0),
                                )
                                .unwrap_or(i64::MAX);
                                let session_id =
                                    Some(self.session.session_id.clone());
                                if let Some(payload) =
                                    guard.build_payload(&verdict, session_id, ts_ms)
                                {
                                    let ssb_payload =
                                        crate::ssb::SsbPayload::ClaudeCodeArchitectureEvent(
                                            payload,
                                        );
                                    std::thread::spawn(move || {
                                        if let Ok(rt) =
                                            tokio::runtime::Builder::new_current_thread()
                                                .enable_all()
                                                .build()
                                        {
                                            let _ = rt.block_on(ssb.write_payload(
                                                "conversation.approval_fatigue_guard",
                                                0,
                                                ssb_payload,
                                                Vec::new(),
                                            ));
                                        }
                                    });
                                }
                            }
                        }
                    }
                }

                // E-25 stub F1a-bash-complexity-emit: emit BashComplexityBlocked SSB
                // row when the permission outcome was a BashComplexityGuard refusal
                // (reason starts with "bash_complexity:") and ssb_writer is wired.
                // The guard already ran inside authorize_with_context (permissions.rs)
                // and encoded the verdict in the Deny reason; we detect it here so
                // the emitter shares the same ssb_writer already on this struct
                // rather than pushing a handle into the permissions module.
                // Gate: ELAI_E25_BASH_COMPLEXITY_GUARD_ENABLE is the same env gate
                // the guard itself checked before producing the reason prefix, so
                // no additional gate read is needed.
                if let PermissionOutcome::Deny { reason } = &permission_outcome {
                    if reason.starts_with("bash_complexity:")
                        && tool_name == "Bash"
                    {
                        if let Some(ssb) = self.ssb_writer.clone() {
                            let ts_ms = i64::try_from(
                                std::time::SystemTime::now() // entropy-allow: ssb-emit-observability-ts-ms
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_millis())
                                    .unwrap_or(0),
                            )
                            .unwrap_or(i64::MAX);
                            let session_id =
                                Some(self.session.session_id.clone());
                            let guard =
                                crate::architecture::bash_complexity_guard::BashComplexityGuard::new();
                            let verdict =
                                crate::architecture::bash_complexity_guard::BashComplexityVerdict::Refused {
                                    reason: reason.clone(),
                                };
                            if let Some(payload) =
                                guard.build_payload(&effective_input, &verdict, session_id, ts_ms)
                            {
                                let ssb_payload =
                                    crate::ssb::SsbPayload::ClaudeCodeArchitectureEvent(payload);
                                std::thread::spawn(move || {
                                    if let Ok(rt) =
                                        tokio::runtime::Builder::new_current_thread()
                                            .enable_all()
                                            .build()
                                    {
                                        let _ = rt.block_on(ssb.write_payload(
                                            "conversation.bash_complexity_guard",
                                            0,
                                            ssb_payload,
                                            Vec::new(),
                                        ));
                                    }
                                });
                            }
                        }
                    }
                }

                let result_message = match permission_outcome {
                    PermissionOutcome::Allow => {
                        self.record_tool_started(iterations, &tool_name);
                        let (mut output, mut is_error) =
                            match self.tool_executor.execute(&tool_name, &effective_input) {
                                Ok(output) => (output, false),
                                Err(error) => (error.to_string(), true),
                            };
                        output = merge_hook_feedback(pre_hook_result.messages(), output, false);

                        let post_hook_result = if is_error {
                            self.run_post_tool_use_failure_hook(
                                &tool_name,
                                &effective_input,
                                &output,
                            )
                        } else {
                            self.run_post_tool_use_hook(
                                &tool_name,
                                &effective_input,
                                &mut output,
                                false,
                            )
                        };
                        if post_hook_result.is_denied()
                            || post_hook_result.is_failed()
                            || post_hook_result.is_cancelled()
                        {
                            is_error = true;
                        }
                        output = merge_hook_feedback(
                            post_hook_result.messages(),
                            output,
                            post_hook_result.is_denied()
                                || post_hook_result.is_failed()
                                || post_hook_result.is_cancelled(),
                        );

                        ConversationMessage::tool_result(tool_use_id, tool_name, output, is_error)
                    }
                    PermissionOutcome::Deny { reason } => ConversationMessage::tool_result(
                        tool_use_id,
                        tool_name,
                        merge_hook_feedback(pre_hook_result.messages(), reason, true),
                        true,
                    ),
                };

                let observation_tokens: usize = result_message
                    .blocks
                    .iter()
                    .map(|block| match block {
                        ContentBlock::Text { text } => text.len() / 4 + 1,
                        ContentBlock::ToolUse { name, input, .. } => {
                            (name.len() + input.len()) / 4 + 1
                        }
                        ContentBlock::ToolResult { tool_name, output, .. } => {
                            (tool_name.len() + output.len()) / 4 + 1
                        }
                    })
                    .sum();
                let window = effective_context_window(self.context_backend_mode) as usize;
                let used = self.usage_tracker.cumulative_usage().input_tokens as usize;
                let slot_remaining = window.saturating_sub(used);
                let alpha = 0.8;
                let mode = preassess_observation_size(observation_tokens, slot_remaining, alpha);
                if mode != PreassessMode::None {
                    if let Some(ssb) = self.ssb_writer.clone() {
                        emit_behavior_steering_event(
                            &ssb,
                            BehaviorSteeringKind::ObservationSizePreassessed,
                            serde_json::json!({
                                "observation_tokens": observation_tokens,
                                "slot_remaining": slot_remaining,
                                "alpha": alpha,
                                "mode": match mode {
                                    PreassessMode::None => "none",
                                    PreassessMode::Selective => "selective",
                                    PreassessMode::All => "all",
                                },
                            }),
                        );
                    }
                    if let (Some(budget), Some(ssb)) =
                        (self.context_budget.as_mut(), self.ssb_writer.clone())
                    {
                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            if let Err(e) = rt.block_on(budget.adaptive_reduce(
                                0.5,
                                ExecutionMode::Autonomous,
                                &ssb,
                            )) {
                                tracing::warn!(
                                    context_budget.adaptive_reduce_err = %e,
                                    "adaptive_reduce failed in preassess (non-fatal)"
                                );
                            }
                        }
                    }
                }

                self.session
                    .push_message(result_message.clone())
                    .map_err(|error| RuntimeError::new(error.to_string()))?;
                self.record_tool_finished(iterations, &result_message);
                tool_results.push(result_message);
            }
        }

        let auto_compaction = self.maybe_auto_compact();

        let summary = TurnSummary {
            assistant_messages,
            tool_results,
            prompt_cache_events,
            iterations,
            usage: self.usage_tracker.cumulative_usage(),
            auto_compaction,
        };
        self.record_turn_completed(&summary);

        Ok(summary)
    }

    #[must_use]
    pub fn compact(&self, config: CompactionConfig) -> CompactionResult {
        if enforce_lossless_compaction(
            &self.session,
            config,
            None,
            self.ssb_writer.as_deref(),
        )
        .is_err()
        {
            return CompactionResult {
                summary: String::new(),
                formatted_summary: String::new(),
                compacted_session: self.session.clone(),
                removed_message_count: 0,
            };
        }
        compact_session(&self.session, config)
    }

    #[must_use]
    pub fn estimated_tokens(&self) -> usize {
        estimate_session_tokens(&self.session)
    }

    #[must_use]
    pub fn usage(&self) -> &UsageTracker {
        &self.usage_tracker
    }

    #[must_use]
    pub fn session(&self) -> &Session {
        &self.session
    }

    #[must_use]
    pub fn fork_session(&self, branch_name: Option<String>) -> Session {
        self.session.fork(branch_name)
    }

    #[must_use]
    pub fn into_session(self) -> Session {
        self.session
    }

    /// Append a user-role message to the session without invoking the model.
    ///
    /// Used by the chat REFLECT substitution path (B-39C) to keep
    /// conversation history coherent when the model bypass returns the
    /// pipeline's answer directly. Errors map session failures into
    /// `RuntimeError` so callers can use the `?` operator uniformly.
    pub fn push_user_text(
        &mut self,
        text: impl Into<String>,
    ) -> Result<(), RuntimeError> {
        self.session
            .push_user_text(text)
            .map_err(|error| RuntimeError::new(error.to_string()))
    }

    /// Append an assistant-role text message to the session without invoking
    /// the model. The message is wrapped in a single `ContentBlock::Text`.
    ///
    /// Used by the chat REFLECT substitution path (B-39C) to record the
    /// pipeline-selected reply so subsequent turns see prior context.
    pub fn push_assistant_text(
        &mut self,
        text: impl Into<String>,
    ) -> Result<(), RuntimeError> {
        let message = ConversationMessage::assistant(vec![ContentBlock::Text {
            text: text.into(),
        }]);
        self.session
            .push_message(message)
            .map_err(|error| RuntimeError::new(error.to_string()))
    }

    /// Record token usage from a REFLECT pipeline run.
    ///
    /// `total_tokens` is the combined output-token cost across pipeline
    /// stages (TRIAGE + REASONER * N + AUDITOR + SYNTHESIZE). The pipeline
    /// does not track per-stage input tokens, so `input_tokens` is set to
    /// 0 by design — the value is the pipeline's reported aggregate, not a
    /// placeholder.
    pub fn record_pipeline_usage(&mut self, total_tokens: u32) {
        self.usage_tracker.record(TokenUsage {
            input_tokens: 0,
            output_tokens: total_tokens,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        });
    }

    fn derive_freeze(&self, _config: CompactionConfig) -> crate::context_projection::FreezeState {
        let frozen_head_len = crate::context_policy::frozen_head_system_prefix_len(
            |i| self.session.messages.get(i).is_some_and(|m| m.role == crate::session::MessageRole::System),
            |i| self.session.messages.get(i)
                .and_then(crate::compact::extract_existing_compacted_summary).is_some(),
            self.session.messages.len(),
        );
        crate::context_projection::FreezeState {
            frozen_head_len,
            compacted_prefix_len: usize::from(self.session.compaction.is_some()),
        }
    }

    fn project_with_freeze(
        &self,
        config: CompactionConfig,
        freeze: crate::context_projection::FreezeState,
    ) -> crate::context_projection::View {
        let mut projection_session = self.session.clone();
        if let Some(compaction) = self.session.compaction.as_ref() {
            let continuation = crate::compact::get_compact_continuation_message(
                &compaction.summary,
                true,
                projection_session.messages.len() > freeze.frozen_head_len,
            );
            projection_session.messages.insert(
                freeze.frozen_head_len.min(projection_session.messages.len()),
                ConversationMessage {
                    role: crate::session::MessageRole::System,
                    blocks: vec![crate::session::ContentBlock::Text { text: continuation }],
                    usage: None,
                },
            );
        }
        let log = crate::context_projection::TranscriptLog::from_session(&projection_session);
        let mut view = crate::context_projection::fold(
            &log, &config, regime_policy(self.context_backend_mode), &freeze,
        );
        let mode = self.context_backend_mode;
        let level = pressure_level(
            self.usage_tracker.cumulative_usage().input_tokens,
            effective_context_window(mode),
            regime_policy(mode),
        );
        crate::context_projection::append_recall_cards(&mut view, &log, &freeze, level);
        // L-19 slice 2: the recall receipt rides the same projection that built
        // the cards, so the row can never disagree with the view the provider
        // saw. Best-effort like the sibling ContextReduction emit below: no
        // writer wired (legacy / test path) means no row, and the write is
        // fire-and-forget so the sync projection path never blocks on SSB.
        if let Some(ssb) = self.ssb_writer.clone() {
            if let Some(payload) =
                crate::context_projection::recall_receipt_payload(&view, level)
            {
                std::thread::spawn(move || {
                    if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                    {
                        let _ = rt.block_on(ssb.write_payload(
                            "conversation.project_with_freeze",
                            0,
                            payload,
                            Vec::new(),
                        ));
                    }
                });
            }
        }
        view
    }

    fn freeze_gated_view_messages(&mut self) -> Vec<ConversationMessage> {
        if crate::context_projection::projection_disabled() {
            self.last_freeze_decision = None;
            return self.session.messages.clone();
        }
        let Some(config) = self.projection_epoch.or_else(|| {
            projection_epoch_for_log(&self.session, self.context_backend_mode)
        }) else {
            self.last_freeze_decision = None;
            return self.session.messages.clone();
        };
        let policy = regime_policy(self.context_backend_mode);
        let log = crate::context_projection::TranscriptLog::from_session(&self.session);
        let compaction_generation =
            self.session.compaction.as_ref().map_or(0, |value| value.count);
        let now_ms = self.session.updated_at_ms;
        let ttl_ms = crate::cache_economics::CacheEconomicsPlanner::plan_ttl(
            crate::cache_economics::GapStats::default(),
            policy,
            false,
        )
        .ttl_secs()
        .map(|secs| u64::from(secs) * 1000);
        let decision = crate::context_projection::evaluate_freeze(
            self.freeze_epoch.as_ref(), &log, &config, policy, compaction_generation, now_ms, ttl_ms,
        );
        self.last_freeze_decision = Some(decision);
        match decision {
            crate::context_projection::FreezeDecision::Reuse => {
                if let Some(epoch) = self.freeze_epoch.as_mut() {
                    epoch.reuse_count = epoch.reuse_count.saturating_add(1);
                    let mut projected = epoch.projected_prefix.clone();
                    projected.extend(self.session.messages[epoch.tail_start..].to_vec());
                    projected
                } else {
                    self.session.messages.clone()
                }
            }
            crate::context_projection::FreezeDecision::Recompute(cause) => {
                let freeze = self.derive_freeze(config);
                let boundary_index = freeze.frozen_head_len.saturating_sub(1);
                let boundary_hash = log
                    .messages()
                    .get(boundary_index)
                    .map(crate::context_projection::boundary_fingerprint)
                    .unwrap_or([0u8; 8]);
                let previous_epoch = self.freeze_epoch.as_ref().map_or(0, |epoch| epoch.epoch);
                let view = self.project_with_freeze(config, freeze);
                let prefix_len = freeze.frozen_head_len.min(view.messages.len()).saturating_add(1).min(view.messages.len());
                let projected_prefix = view.messages[..prefix_len].to_vec();
                let inserted_compaction_continuation = usize::from(self.session.compaction.is_some());
                let tail_start = view.tail_start.saturating_sub(inserted_compaction_continuation);
                self.freeze_epoch = Some(crate::context_projection::FreezeEpoch {
                    freeze,
                    boundary_hash,
                    raw_log_len: log.len(),
                    epoch: previous_epoch.saturating_add(1),
                    reuse_count: 0,
                    policy,
                    projected_prefix,
                    tail_start,
                    compaction_generation,
                    computed_at_ms: now_ms,
                });
                if let Some(ssb) = self.ssb_writer.clone() {
                    let payload = crate::ssb::CacheEconomicsEventPayload::prefix_recomputed(
                        cause,
                        boundary_hash,
                        "conversation.freeze_prefix_recompute",
                    );
                    let handle = std::thread::spawn(move || {
                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            let _ = rt.block_on(crate::cache_economics::emit_cache_economics_event(
                                &ssb,
                                "conversation.freeze_prefix_recompute",
                                0,
                                payload,
                            ));
                        }
                    });
                    // Join so the row is observable; one local SQLite write only on Recompute,
                    // never on Reuse, after re-folding the transcript.
                    let _ = handle.join();
                }
                tracing::debug!(?cause, epoch = self.freeze_epoch_number(), "freeze prefix recomputed");
                view.messages
            }
        }
    }

    pub(crate) fn last_freeze_decision(&self) -> Option<crate::context_projection::FreezeDecision> {
        self.last_freeze_decision
    }

    pub(crate) fn freeze_epoch_number(&self) -> u32 {
        self.freeze_epoch.as_ref().map_or(0, |epoch| epoch.epoch)
    }

    fn maybe_auto_compact(&mut self) -> Option<AutoCompactionEvent> {
        let input_tokens = self.usage_tracker.cumulative_usage().input_tokens;
        // J-25 T4: graded pressure ladder drives entry. The window and policy
        // are derived from the runtime's `context_backend_mode` (threaded via
        // `with_context_backend_mode` at construction) so Local sessions grade
        // against the local window, not the Cloud default. The flat threshold
        // remains a secondary hard floor below the grade so the kill-switch
        // path keeps its old gate.
        let mode = self.context_backend_mode;
        let policy = regime_policy(mode);
        let window = effective_context_window(mode);
        let level = pressure_level(input_tokens, window, policy);
        if level == PressureLevel::Normal {
            return None;
        }
        // Secondary hard floor: even above the grade, never fire below the flat
        // threshold (preserves the kill-switch flat path semantics when the
        // policy is disabled; under the active policy this is a no-op for any
        // grade >= Moderate because the grade already implies used >= 50% of
        // usable, which is well above the legacy 100k floor for cloud windows).
        if input_tokens < self.auto_compaction_input_tokens_threshold {
            return None;
        }

        // J-95: prefer relevance-based adaptive reduction before lossy
        // compaction. This opt-in path is best-effort and never blocks a turn.
        if j95_adaptive_reduce_on_compact_enabled() {
            if let (Some(budget), Some(ssb)) =
                (self.context_budget.as_mut(), self.ssb_writer.clone())
            {
                if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    if let Err(e) = rt.block_on(budget.adaptive_reduce(
                        0.5,
                        ExecutionMode::Interactive,
                        &ssb,
                    )) {
                        tracing::warn!(
                            context_budget.adaptive_reduce_err = %e,
                            "adaptive_reduce failed before auto-compaction (non-fatal)"
                        );
                    }
                }
            }
        }

        // BLOCK-3 fix: run the cheapest-first compaction ladder in the
        // interactive turn (LOCAL regime).  The ladder is default-ON
        // (compaction_ladder_enabled()) and sync — use NullEmitter here
        // (no SSB writer ref in this sync fn; full telemetry emitter is
        // the autonomous task_runner path).  This accounts for the tier
        // budget before the actual compact_session trim so the ladder
        // decision is observed in the token budget log.
        if compaction_ladder_enabled() {
            let ladder = CompactionLadder::default_order(
                u64::from(self.auto_compaction_input_tokens_threshold),
            );
            let _ = run_compaction_ladder(
                &ladder,
                u64::from(input_tokens),
                Some(self.session.session_id.clone()),
                &NullEmitter,
            );
        }

        let compaction_config = CompactionConfig {
            max_estimated_tokens: 0,
            preserve_intermediate_numerics: matches!(policy, ContextPolicy::Local),
            ..CompactionConfig::default()
        };
        if enforce_lossless_compaction(
            &self.session,
            compaction_config,
            None,
            self.ssb_writer.as_deref(),
        )
        .is_err()
        {
            tracing::error!(
                "auto-compaction skipped because lossless recovery could not be proven"
            );
            return None;
        }
        let result = compact_session(&self.session, compaction_config);

        if result.removed_message_count == 0 {
            return None;
        }

        if crate::context_projection::projection_disabled() {
            self.session = result.compacted_session;
        } else {
            self.projection_epoch = Some(compaction_config);
            self.session
                .record_compaction(result.summary, result.removed_message_count);
        }

        // BLOCK-2 fix: emit a ContextReduction SSB row from the real local
        // trim path.  The ssb_writer is threaded in via with_ssb_writer;
        // when absent (legacy / test path) no row is written (best-effort,
        // never blocks or panics).  Fire-and-forget via std::thread::spawn
        // (same pattern as approval-fatigue emit above) to keep this fn sync.
        if let Some(ssb) = self.ssb_writer.clone() {
            // input_tokens is u32; ContextReductionPayload.before/after_tokens are usize.
            let before_usize: usize = input_tokens as usize;
            let after_tokens = self.usage_tracker.cumulative_usage().input_tokens;
            let after_usize: usize = after_tokens as usize;
            let slots_affected: Vec<u8> = Vec::new();
            let payload = SsbPayload::ContextReduction(ContextReductionPayload {
                // level 0 = FilteredByRelevance (cheapest, mirrors AdaptiveLevel::FilteredByRelevance)
                level: 0,
                before_tokens: before_usize,
                after_tokens: after_usize,
                slots_affected,
            });
            std::thread::spawn(move || {
                if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    let _ = rt.block_on(ssb.write_payload(
                        "conversation.maybe_auto_compact",
                        0,
                        payload,
                        Vec::new(),
                    ));
                }
            });
        }

        Some(AutoCompactionEvent {
            removed_message_count: result.removed_message_count,
        })
    }

    fn record_turn_started(&self, user_input: &str) {
        let Some(session_tracer) = &self.session_tracer else {
            return;
        };

        let mut attributes = Map::new();
        attributes.insert(
            "user_input".to_string(),
            Value::String(user_input.to_string()),
        );
        session_tracer.record("turn_started", attributes);
    }

    fn record_assistant_iteration(
        &self,
        iteration: usize,
        assistant_message: &ConversationMessage,
        pending_tool_use_count: usize,
    ) {
        let Some(session_tracer) = &self.session_tracer else {
            return;
        };

        let mut attributes = Map::new();
        attributes.insert("iteration".to_string(), Value::from(iteration as u64));
        attributes.insert(
            "assistant_blocks".to_string(),
            Value::from(assistant_message.blocks.len() as u64),
        );
        attributes.insert(
            "pending_tool_use_count".to_string(),
            Value::from(pending_tool_use_count as u64),
        );
        session_tracer.record("assistant_iteration_completed", attributes);
    }

    fn record_tool_started(&self, iteration: usize, tool_name: &str) {
        let Some(session_tracer) = &self.session_tracer else {
            return;
        };

        let mut attributes = Map::new();
        attributes.insert("iteration".to_string(), Value::from(iteration as u64));
        attributes.insert(
            "tool_name".to_string(),
            Value::String(tool_name.to_string()),
        );
        session_tracer.record("tool_execution_started", attributes);
    }

    fn record_tool_finished(&self, iteration: usize, result_message: &ConversationMessage) {
        let Some(session_tracer) = &self.session_tracer else {
            return;
        };

        let Some(ContentBlock::ToolResult {
            tool_name,
            is_error,
            ..
        }) = result_message.blocks.first()
        else {
            return;
        };

        let mut attributes = Map::new();
        attributes.insert("iteration".to_string(), Value::from(iteration as u64));
        attributes.insert("tool_name".to_string(), Value::String(tool_name.clone()));
        attributes.insert("is_error".to_string(), Value::Bool(*is_error));
        session_tracer.record("tool_execution_finished", attributes);
    }

    fn record_turn_completed(&self, summary: &TurnSummary) {
        let Some(session_tracer) = &self.session_tracer else {
            return;
        };

        let mut attributes = Map::new();
        attributes.insert(
            "iterations".to_string(),
            Value::from(summary.iterations as u64),
        );
        attributes.insert(
            "assistant_messages".to_string(),
            Value::from(summary.assistant_messages.len() as u64),
        );
        attributes.insert(
            "tool_results".to_string(),
            Value::from(summary.tool_results.len() as u64),
        );
        attributes.insert(
            "prompt_cache_events".to_string(),
            Value::from(summary.prompt_cache_events.len() as u64),
        );
        session_tracer.record("turn_completed", attributes);
    }

    fn record_turn_failed(&self, iteration: usize, error: &RuntimeError) {
        let Some(session_tracer) = &self.session_tracer else {
            return;
        };

        let mut attributes = Map::new();
        attributes.insert("iteration".to_string(), Value::from(iteration as u64));
        attributes.insert("error".to_string(), Value::String(error.to_string()));
        session_tracer.record("turn_failed", attributes);
    }
}

/// Reads the automatic compaction threshold from the environment.
#[must_use]
pub fn auto_compaction_threshold_from_env() -> u32 {
    parse_auto_compaction_threshold(
        std::env::var(AUTO_COMPACTION_THRESHOLD_ENV_VAR)
            .ok()
            .as_deref(),
    )
}

#[must_use]
fn parse_auto_compaction_threshold(value: Option<&str>) -> u32 {
    value
        .and_then(|raw| raw.trim().parse::<u32>().ok())
        .filter(|threshold| *threshold > 0)
        .unwrap_or(DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD)
}

fn build_assistant_message(
    events: Vec<AssistantEvent>,
) -> Result<
    (
        ConversationMessage,
        Option<TokenUsage>,
        Vec<PromptCacheEvent>,
    ),
    RuntimeError,
> {
    let mut text = String::new();
    let mut blocks = Vec::new();
    let mut prompt_cache_events = Vec::new();
    let mut finished = false;
    let mut usage = None;

    for event in events {
        match event {
            AssistantEvent::TextDelta(delta) => text.push_str(&delta),
            AssistantEvent::ToolUse { id, name, input } => {
                flush_text_block(&mut text, &mut blocks);
                blocks.push(ContentBlock::ToolUse { id, name, input });
            }
            AssistantEvent::Usage(value) => usage = Some(value),
            AssistantEvent::PromptCache(event) => prompt_cache_events.push(event),
            AssistantEvent::MessageStop => {
                finished = true;
            }
        }
    }

    flush_text_block(&mut text, &mut blocks);

    if !finished {
        return Err(RuntimeError::new(
            "assistant stream ended without a message stop event",
        ));
    }
    if blocks.is_empty() {
        return Err(RuntimeError::new("assistant stream produced no content"));
    }

    Ok((
        ConversationMessage::assistant_with_usage(blocks, usage),
        usage,
        prompt_cache_events,
    ))
}

/// Apply output postprocessor to all text blocks in a message.
fn postprocess_message_blocks(blocks: &mut Vec<ContentBlock>, config: &OutputConfig) {
    for block in blocks.iter_mut() {
        if let ContentBlock::Text { text } = block {
            let processed = output_postprocessor::process(text, config);
            *text = processed;
        }
    }
    // Remove text blocks that became empty after postprocessing.
    blocks.retain(|block| match block {
        ContentBlock::Text { text } => !text.is_empty(),
        _ => true,
    });
}

fn flush_text_block(text: &mut String, blocks: &mut Vec<ContentBlock>) {
    if !text.is_empty() {
        blocks.push(ContentBlock::Text {
            text: std::mem::take(text),
        });
    }
}

fn format_hook_message(result: &HookRunResult, fallback: &str) -> String {
    if result.messages().is_empty() {
        fallback.to_string()
    } else {
        result.messages().join("\n")
    }
}

fn merge_hook_feedback(messages: &[String], output: String, is_error: bool) -> String {
    if messages.is_empty() {
        return output;
    }

    let mut sections = Vec::new();
    if !output.trim().is_empty() {
        sections.push(output);
    }
    let label = if is_error {
        "Hook feedback (error)"
    } else {
        "Hook feedback"
    };
    sections.push(format!("{label}:\n{}", messages.join("\n")));
    sections.join("\n\n")
}

type ToolHandler = Box<dyn FnMut(&str) -> Result<String, ToolError>>;

/// Simple in-memory tool executor for tests and lightweight integrations.
#[derive(Default)]
pub struct StaticToolExecutor {
    handlers: BTreeMap<String, ToolHandler>,
}

impl StaticToolExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn register(
        mut self,
        tool_name: impl Into<String>,
        handler: impl FnMut(&str) -> Result<String, ToolError> + 'static,
    ) -> Self {
        self.handlers.insert(tool_name.into(), Box::new(handler));
        self
    }
}

impl ToolExecutor for StaticToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, ToolError> {
        self.handlers
            .get_mut(tool_name)
            .ok_or_else(|| ToolError::new(format!("unknown tool: {tool_name}")))?(input)
    }
}

// ---------------------------------------------------------------------------
// E-30: Content-source trust labels (plumbing only — no behaviour change yet).
// ---------------------------------------------------------------------------

/// Trust classification for text entering the ELAI pipeline.
///
/// Labels every piece of content by provenance so downstream gates can
/// refuse writes from untrusted sources without parsing content.
/// See E-30 incident hardening: untrusted-memory-write pattern.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TextSource {
    /// Typed directly by the human operator in the current session.
    UserPrompt,
    /// Generated by ELAI's own model (post-trust-dialog).
    AssistantGenerated,
    /// Output of an external tool (shell, MCP, file read, etc.).
    /// External tool output is NOT implicitly trusted for memory writes.
    ExternalTool { tool_name: String },
    /// Content read from a project file that has not been reviewed this session.
    UntrustedFile { path: String },
    /// Content explicitly approved by the operator (e.g. copy-pasted with intent).
    OperatorApproved,
}

impl TextSource {
    /// Returns `true` if this source is trusted for memory / SSB writes.
    ///
    /// Trust hierarchy (strongest to weakest):
    ///   OperatorApproved > UserPrompt > AssistantGenerated > ExternalTool > UntrustedFile
    ///
    /// Only the top two are considered trusted for memory writes.
    pub fn is_trusted_for_memory_write(&self) -> bool {
        matches!(self, Self::UserPrompt | Self::OperatorApproved)
    }
}

/// A piece of text with an attached [`TextSource`] label.
///
/// Wraps any `String` payload with provenance metadata so callers do not
/// need to pass source alongside content as separate arguments.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LabeledText {
    pub source: TextSource,
    pub content: String,
}

impl LabeledText {
    /// Construct a new labeled text.
    pub fn new(source: TextSource, content: impl Into<String>) -> Self {
        Self { source, content: content.into() }
    }

    /// Convenience: wrap with [`TextSource::UserPrompt`].
    pub fn from_user(content: impl Into<String>) -> Self {
        Self::new(TextSource::UserPrompt, content)
    }

    /// Convenience: wrap with [`TextSource::AssistantGenerated`].
    pub fn from_assistant(content: impl Into<String>) -> Self {
        Self::new(TextSource::AssistantGenerated, content)
    }

    /// Convenience: wrap with [`TextSource::ExternalTool`].
    pub fn from_tool(tool_name: impl Into<String>, content: impl Into<String>) -> Self {
        Self::new(TextSource::ExternalTool { tool_name: tool_name.into() }, content)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_assistant_message, parse_auto_compaction_threshold, ApiClient, ApiRequest,
        AssistantEvent, AutoCompactionEvent, ConversationRuntime, PromptCacheEvent, RuntimeError,
        StaticToolExecutor, ToolExecutor, DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD,
    };
    use crate::compact::CompactionConfig;
    use crate::config::{RuntimeFeatureConfig, RuntimeHookConfig};
    use crate::permissions::{
        PermissionMode, PermissionPolicy, PermissionPromptDecision, PermissionPrompter,
        PermissionRequest,
    };
    use crate::prompt::{ProjectContext, SystemPromptBuilder};
    use crate::session::{ContentBlock, MessageRole, Session};
    use crate::usage::TokenUsage;
    use crate::ToolError;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::telemetry_types::{MemoryTelemetrySink, SessionTracer, TelemetryEvent};

    struct ScriptedApiClient {
        call_count: usize,
    }

    impl ApiClient for ScriptedApiClient {
        fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
            self.call_count += 1;
            match self.call_count {
                1 => {
                    assert!(request
                        .messages
                        .iter()
                        .any(|message| message.role == MessageRole::User));
                    Ok(vec![
                        AssistantEvent::TextDelta("Let me calculate that.".to_string()),
                        AssistantEvent::ToolUse {
                            id: "tool-1".to_string(),
                            name: "add".to_string(),
                            input: "2,2".to_string(),
                        },
                        AssistantEvent::Usage(TokenUsage {
                            input_tokens: 20,
                            output_tokens: 6,
                            cache_creation_input_tokens: 1,
                            cache_read_input_tokens: 2,
                        }),
                        AssistantEvent::MessageStop,
                    ])
                }
                2 => {
                    let last_message = request
                        .messages
                        .last()
                        .expect("tool result should be present");
                    assert_eq!(last_message.role, MessageRole::Tool);
                    Ok(vec![
                        AssistantEvent::TextDelta("The answer is 4.".to_string()),
                        AssistantEvent::Usage(TokenUsage {
                            input_tokens: 24,
                            output_tokens: 4,
                            cache_creation_input_tokens: 1,
                            cache_read_input_tokens: 3,
                        }),
                        AssistantEvent::PromptCache(PromptCacheEvent {
                            unexpected: true,
                            reason:
                                "cache read tokens dropped while prompt fingerprint remained stable"
                                    .to_string(),
                            previous_cache_read_input_tokens: 6_000,
                            current_cache_read_input_tokens: 1_000,
                            token_drop: 5_000,
                            fingerprint_hash: None,
                        }),
                        AssistantEvent::MessageStop,
                    ])
                }
                _ => unreachable!("extra API call"),
            }
        }
    }

    struct PromptAllowOnce;

    impl PermissionPrompter for PromptAllowOnce {
        fn decide(&mut self, request: &PermissionRequest) -> PermissionPromptDecision {
            assert_eq!(request.tool_name, "add");
            PermissionPromptDecision::Allow
        }
    }

    #[test]
    fn runs_user_to_tool_to_result_loop_end_to_end_and_tracks_usage() {
        let api_client = ScriptedApiClient { call_count: 0 };
        let tool_executor = StaticToolExecutor::new().register("add", |input| {
            let total = input
                .split(',')
                .map(|part| part.parse::<i32>().expect("input must be valid integer"))
                .sum::<i32>();
            Ok(total.to_string())
        });
        let permission_policy = PermissionPolicy::new(PermissionMode::WorkspaceWrite);
        let system_prompt = SystemPromptBuilder::new()
            .with_project_context(ProjectContext {
                cwd: PathBuf::from("/tmp/project"),
                current_date: "2026-03-31".to_string(),
                git_status: None,
                git_diff: None,
                instruction_files: Vec::new(),
            })
            .with_os("linux", "6.8")
            .build();
        let mut runtime = ConversationRuntime::new(
            Session::new(),
            api_client,
            tool_executor,
            permission_policy,
            system_prompt,
        );

        let summary = runtime
            .run_turn("what is 2 + 2?", Some(&mut PromptAllowOnce))
            .expect("conversation loop should succeed");

        assert_eq!(summary.iterations, 2);
        assert_eq!(summary.assistant_messages.len(), 2);
        assert_eq!(summary.tool_results.len(), 1);
        assert_eq!(summary.prompt_cache_events.len(), 1);
        assert_eq!(runtime.session().messages.len(), 4);
        assert_eq!(summary.usage.output_tokens, 10);
        assert_eq!(summary.auto_compaction, None);
        assert!(matches!(
            runtime.session().messages[1].blocks[1],
            ContentBlock::ToolUse { .. }
        ));
        assert!(matches!(
            runtime.session().messages[2].blocks[0],
            ContentBlock::ToolResult {
                is_error: false,
                ..
            }
        ));
    }

    #[test]
    fn records_runtime_session_trace_events() {
        let sink = Arc::new(MemoryTelemetrySink::default());
        let tracer = SessionTracer::new("session-runtime", sink.clone());
        let mut runtime = ConversationRuntime::new(
            Session::new(),
            ScriptedApiClient { call_count: 0 },
            StaticToolExecutor::new().register("add", |_input| Ok("4".to_string())),
            PermissionPolicy::new(PermissionMode::WorkspaceWrite),
            vec!["system".to_string()],
        )
        .with_session_tracer(tracer);

        runtime
            .run_turn("what is 2 + 2?", Some(&mut PromptAllowOnce))
            .expect("conversation loop should succeed");

        let events = sink.events();
        let trace_names = events
            .iter()
            .filter_map(|event| match event {
                TelemetryEvent::SessionTrace(trace) => Some(trace.name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(trace_names.contains(&"turn_started"));
        assert!(trace_names.contains(&"assistant_iteration_completed"));
        assert!(trace_names.contains(&"tool_execution_started"));
        assert!(trace_names.contains(&"tool_execution_finished"));
        assert!(trace_names.contains(&"turn_completed"));
    }

    #[test]
    fn records_denied_tool_results_when_prompt_rejects() {
        struct RejectPrompter;
        impl PermissionPrompter for RejectPrompter {
            fn decide(&mut self, _request: &PermissionRequest) -> PermissionPromptDecision {
                PermissionPromptDecision::Deny {
                    reason: "not now".to_string(),
                }
            }
        }

        struct SingleCallApiClient;
        impl ApiClient for SingleCallApiClient {
            fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
                if request
                    .messages
                    .iter()
                    .any(|message| message.role == MessageRole::Tool)
                {
                    return Ok(vec![
                        AssistantEvent::TextDelta("I could not use the tool.".to_string()),
                        AssistantEvent::MessageStop,
                    ]);
                }
                Ok(vec![
                    AssistantEvent::ToolUse {
                        id: "tool-1".to_string(),
                        name: "blocked".to_string(),
                        input: "secret".to_string(),
                    },
                    AssistantEvent::MessageStop,
                ])
            }
        }

        let mut runtime = ConversationRuntime::new(
            Session::new(),
            SingleCallApiClient,
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::WorkspaceWrite),
            vec!["system".to_string()],
        );

        let summary = runtime
            .run_turn("use the tool", Some(&mut RejectPrompter))
            .expect("conversation should continue after denied tool");

        assert_eq!(summary.tool_results.len(), 1);
        assert!(matches!(
            &summary.tool_results[0].blocks[0],
            ContentBlock::ToolResult { is_error: true, output, .. } if output == "not now"
        ));
    }

    #[test]
    #[cfg_attr(windows, ignore = "P: Windows cmd.exe has no `printf` and uses `&` not `;`; POSIX-shell hook test")]
    fn denies_tool_use_when_pre_tool_hook_blocks() {
        struct SingleCallApiClient;
        impl ApiClient for SingleCallApiClient {
            fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
                if request
                    .messages
                    .iter()
                    .any(|message| message.role == MessageRole::Tool)
                {
                    return Ok(vec![
                        AssistantEvent::TextDelta("blocked".to_string()),
                        AssistantEvent::MessageStop,
                    ]);
                }
                Ok(vec![
                    AssistantEvent::ToolUse {
                        id: "tool-1".to_string(),
                        name: "blocked".to_string(),
                        input: r#"{"path":"secret.txt"}"#.to_string(),
                    },
                    AssistantEvent::MessageStop,
                ])
            }
        }

        let mut runtime = ConversationRuntime::new_with_features(
            Session::new(),
            SingleCallApiClient,
            StaticToolExecutor::new().register("blocked", |_input| {
                panic!("tool should not execute when hook denies")
            }),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
            &RuntimeFeatureConfig::default().with_hooks(RuntimeHookConfig::new(
                vec![shell_snippet("printf 'blocked by hook'; exit 2")],
                Vec::new(),
                Vec::new(),
            )),
        );

        let summary = runtime
            .run_turn("use the tool", None)
            .expect("conversation should continue after hook denial");

        assert_eq!(summary.tool_results.len(), 1);
        let ContentBlock::ToolResult {
            is_error, output, ..
        } = &summary.tool_results[0].blocks[0]
        else {
            panic!("expected tool result block");
        };
        assert!(
            *is_error,
            "hook denial should produce an error result: {output}"
        );
        assert!(
            output.contains("denied tool") || output.contains("blocked by hook"),
            "unexpected hook denial output: {output:?}"
        );
    }

    #[test]
    #[cfg_attr(windows, ignore = "P: Windows cmd.exe has no `printf`; POSIX-shell hook test")]
    fn denies_tool_use_when_pre_tool_hook_fails() {
        struct SingleCallApiClient;
        impl ApiClient for SingleCallApiClient {
            fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
                if request
                    .messages
                    .iter()
                    .any(|message| message.role == MessageRole::Tool)
                {
                    return Ok(vec![
                        AssistantEvent::TextDelta("failed".to_string()),
                        AssistantEvent::MessageStop,
                    ]);
                }
                Ok(vec![
                    AssistantEvent::ToolUse {
                        id: "tool-1".to_string(),
                        name: "blocked".to_string(),
                        input: r#"{"path":"secret.txt"}"#.to_string(),
                    },
                    AssistantEvent::MessageStop,
                ])
            }
        }

        // given
        let mut runtime = ConversationRuntime::new_with_features(
            Session::new(),
            SingleCallApiClient,
            StaticToolExecutor::new().register("blocked", |_input| {
                panic!("tool should not execute when hook fails")
            }),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
            &RuntimeFeatureConfig::default().with_hooks(RuntimeHookConfig::new(
                vec![shell_snippet("printf 'broken hook'; exit 1")],
                Vec::new(),
                Vec::new(),
            )),
        );

        // when
        let summary = runtime
            .run_turn("use the tool", None)
            .expect("conversation should continue after hook failure");

        // then
        assert_eq!(summary.tool_results.len(), 1);
        let ContentBlock::ToolResult {
            is_error, output, ..
        } = &summary.tool_results[0].blocks[0]
        else {
            panic!("expected tool result block");
        };
        assert!(
            *is_error,
            "hook failure should produce an error result: {output}"
        );
        assert!(
            output.contains("exited with status 1") || output.contains("broken hook"),
            "unexpected hook failure output: {output:?}"
        );
    }

    #[test]
    fn appends_post_tool_hook_feedback_to_tool_result() {
        struct TwoCallApiClient {
            calls: usize,
        }

        impl ApiClient for TwoCallApiClient {
            fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
                self.calls += 1;
                match self.calls {
                    1 => Ok(vec![
                        AssistantEvent::ToolUse {
                            id: "tool-1".to_string(),
                            name: "add".to_string(),
                            input: r#"{"lhs":2,"rhs":2}"#.to_string(),
                        },
                        AssistantEvent::MessageStop,
                    ]),
                    2 => {
                        assert!(request
                            .messages
                            .iter()
                            .any(|message| message.role == MessageRole::Tool));
                        Ok(vec![
                            AssistantEvent::TextDelta("done".to_string()),
                            AssistantEvent::MessageStop,
                        ])
                    }
                    _ => unreachable!("extra API call"),
                }
            }
        }

        let mut runtime = ConversationRuntime::new_with_features(
            Session::new(),
            TwoCallApiClient { calls: 0 },
            StaticToolExecutor::new().register("add", |_input| Ok("4".to_string())),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
            &RuntimeFeatureConfig::default().with_hooks(RuntimeHookConfig::new(
                vec![shell_snippet("printf 'pre hook ran'")],
                vec![shell_snippet("printf 'post hook ran'")],
                Vec::new(),
            )),
        );

        let summary = runtime
            .run_turn("use add", None)
            .expect("tool loop succeeds");

        assert_eq!(summary.tool_results.len(), 1);
        let ContentBlock::ToolResult {
            is_error, output, ..
        } = &summary.tool_results[0].blocks[0]
        else {
            panic!("expected tool result block");
        };
        assert!(
            !*is_error,
            "post hook should preserve non-error result: {output:?}"
        );
        assert!(
            output.contains('4'),
            "tool output missing value: {output:?}"
        );
        assert!(
            output.contains("pre hook ran"),
            "tool output missing pre hook feedback: {output:?}"
        );
        assert!(
            output.contains("post hook ran"),
            "tool output missing post hook feedback: {output:?}"
        );
    }

    #[test]
    #[cfg_attr(windows, ignore = "P: Windows cmd.exe mangles `printf '...'` output; POSIX-shell hook test")]
    fn appends_post_tool_use_failure_hook_feedback_to_tool_result() {
        struct TwoCallApiClient {
            calls: usize,
        }

        impl ApiClient for TwoCallApiClient {
            fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
                self.calls += 1;
                match self.calls {
                    1 => Ok(vec![
                        AssistantEvent::ToolUse {
                            id: "tool-1".to_string(),
                            name: "fail".to_string(),
                            input: r#"{"path":"README.md"}"#.to_string(),
                        },
                        AssistantEvent::MessageStop,
                    ]),
                    2 => {
                        assert!(request
                            .messages
                            .iter()
                            .any(|message| message.role == MessageRole::Tool));
                        Ok(vec![
                            AssistantEvent::TextDelta("done".to_string()),
                            AssistantEvent::MessageStop,
                        ])
                    }
                    _ => unreachable!("extra API call"),
                }
            }
        }

        // given
        let mut runtime = ConversationRuntime::new_with_features(
            Session::new(),
            TwoCallApiClient { calls: 0 },
            StaticToolExecutor::new()
                .register("fail", |_input| Err(ToolError::new("tool exploded"))),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
            &RuntimeFeatureConfig::default().with_hooks(RuntimeHookConfig::new(
                Vec::new(),
                vec![shell_snippet("printf 'post hook should not run'")],
                vec![shell_snippet("printf 'failure hook ran'")],
            )),
        );

        // when
        let summary = runtime
            .run_turn("use fail", None)
            .expect("tool loop succeeds");

        // then
        assert_eq!(summary.tool_results.len(), 1);
        let ContentBlock::ToolResult {
            is_error, output, ..
        } = &summary.tool_results[0].blocks[0]
        else {
            panic!("expected tool result block");
        };
        assert!(
            *is_error,
            "failure hook path should preserve error result: {output:?}"
        );
        assert!(
            output.contains("tool exploded"),
            "tool output missing failure reason: {output:?}"
        );
        assert!(
            output.contains("failure hook ran"),
            "tool output missing failure hook feedback: {output:?}"
        );
        assert!(
            !output.contains("post hook should not run"),
            "normal post hook should not run on tool failure: {output:?}"
        );
    }

    #[test]
    fn reconstructs_usage_tracker_from_restored_session() {
        struct SimpleApi;
        impl ApiClient for SimpleApi {
            fn stream(
                &mut self,
                _request: ApiRequest,
            ) -> Result<Vec<AssistantEvent>, RuntimeError> {
                Ok(vec![
                    AssistantEvent::TextDelta("done".to_string()),
                    AssistantEvent::MessageStop,
                ])
            }
        }

        let mut session = Session::new();
        session
            .messages
            .push(crate::session::ConversationMessage::assistant_with_usage(
                vec![ContentBlock::Text {
                    text: "earlier".to_string(),
                }],
                Some(TokenUsage {
                    input_tokens: 11,
                    output_tokens: 7,
                    cache_creation_input_tokens: 2,
                    cache_read_input_tokens: 1,
                }),
            ));

        let runtime = ConversationRuntime::new(
            session,
            SimpleApi,
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        );

        assert_eq!(runtime.usage().turns(), 1);
        assert_eq!(runtime.usage().cumulative_usage().total_tokens(), 21);
    }

    #[test]
    fn compacts_session_after_turns() {
        struct SimpleApi;
        impl ApiClient for SimpleApi {
            fn stream(
                &mut self,
                _request: ApiRequest,
            ) -> Result<Vec<AssistantEvent>, RuntimeError> {
                Ok(vec![
                    AssistantEvent::TextDelta("done".to_string()),
                    AssistantEvent::MessageStop,
                ])
            }
        }

        let mut runtime = ConversationRuntime::new(
            Session::new(),
            SimpleApi,
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        );
        runtime.run_turn("a", None).expect("turn a");
        runtime.run_turn("b", None).expect("turn b");
        runtime.run_turn("c", None).expect("turn c");

        let result = runtime.compact(CompactionConfig {
            preserve_recent_messages: 2,
            max_estimated_tokens: 1,
            ..CompactionConfig::default()
        });
        assert!(result.summary.contains("Conversation summary"));
        assert_eq!(
            result.compacted_session.messages[0].role,
            MessageRole::System
        );
        assert_eq!(
            result.compacted_session.session_id,
            runtime.session().session_id
        );
        assert!(result.compacted_session.compaction.is_some());
    }

    #[test]
    fn persists_conversation_turn_messages_to_jsonl_session() {
        struct SimpleApi;
        impl ApiClient for SimpleApi {
            fn stream(
                &mut self,
                _request: ApiRequest,
            ) -> Result<Vec<AssistantEvent>, RuntimeError> {
                Ok(vec![
                    AssistantEvent::TextDelta("done".to_string()),
                    AssistantEvent::MessageStop,
                ])
            }
        }

        let path = temp_session_path("persisted-turn");
        let session = Session::new().with_persistence_path(path.clone());
        let mut runtime = ConversationRuntime::new(
            session,
            SimpleApi,
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        );

        runtime
            .run_turn("persist this turn", None)
            .expect("turn should succeed");

        let restored = Session::load_from_path(&path).expect("persisted session should reload");
        fs::remove_file(&path).expect("temp session file should be removable");

        assert_eq!(restored.messages.len(), 2);
        assert_eq!(restored.messages[0].role, MessageRole::User);
        assert_eq!(restored.messages[1].role, MessageRole::Assistant);
        assert_eq!(restored.session_id, runtime.session().session_id);
    }

    #[test]
    fn forks_runtime_session_without_mutating_original() {
        let mut session = Session::new();
        session
            .push_user_text("branch me")
            .expect("message should append");

        let runtime = ConversationRuntime::new(
            session.clone(),
            ScriptedApiClient { call_count: 0 },
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        );

        let forked = runtime.fork_session(Some("alt-path".to_string()));

        assert_eq!(forked.messages, session.messages);
        assert_ne!(forked.session_id, session.session_id);
        assert_eq!(
            forked
                .fork
                .as_ref()
                .map(|fork| (fork.parent_session_id.as_str(), fork.branch_name.as_deref())),
            Some((session.session_id.as_str(), Some("alt-path")))
        );
        assert!(runtime.session().fork.is_none());
    }

    fn temp_session_path(label: &str) -> PathBuf { // TEST_KEEP
        let nanos = SystemTime::now() // TEST_KEEP
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("runtime-conversation-{label}-{nanos}.json"))
    }

    #[cfg(windows)]
    fn shell_snippet(script: &str) -> String {
        script.replace('\'', "\"")
    }

    #[cfg(not(windows))]
    fn shell_snippet(script: &str) -> String {
        script.to_string()
    }

    #[test]
fn auto_compacts_when_cumulative_input_threshold_is_crossed() {
    // Records every provider request AND reports usage above the threshold,
    // so the cumulative-input trigger fires exactly as it did on main.
    struct RecordingHighUsageApi {
        seen: std::rc::Rc<std::cell::RefCell<Vec<Vec<crate::session::ConversationMessage>>>>,
    }
    impl ApiClient for RecordingHighUsageApi {
        fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
            self.seen.borrow_mut().push(request.messages);
            Ok(vec![
                AssistantEvent::TextDelta("done".to_string()),
                AssistantEvent::Usage(TokenUsage {
                    input_tokens: 120_000,
                    output_tokens: 4,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 0,
                }),
                AssistantEvent::MessageStop,
            ])
        }
    }

    let mut session = Session::new();
    session.messages = vec![
        crate::session::ConversationMessage::user_text("one"),
        crate::session::ConversationMessage::assistant(vec![ContentBlock::Text {
            text: "two".to_string(),
        }]),
        crate::session::ConversationMessage::user_text("three"),
        crate::session::ConversationMessage::assistant(vec![ContentBlock::Text {
            text: "four".to_string(),
        }]),
    ];

    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let mut runtime = ConversationRuntime::new(
        session,
        RecordingHighUsageApi { seen: seen.clone() },
        StaticToolExecutor::new(),
        PermissionPolicy::new(PermissionMode::DangerFullAccess),
        vec!["system".to_string()],
    )
    .with_auto_compaction_input_tokens_threshold(100_000);

    let summary = runtime
        .run_turn("trigger", None)
        .expect("turn should succeed");

    // UNCHANGED: auto-compaction still fires on the cumulative trigger and reports.
    assert_eq!(
        summary.auto_compaction,
        Some(AutoCompactionEvent {
            removed_message_count: 2,
        })
    );

    // CC-1 (deliberate contract change, L-16 slice 2): the transcript log is
    // immutable ground truth. The System summary is NOT written into the log;
    // messages[0] stays the original User message. On main this assertion was
    // `MessageRole::System` and pinned the destructive rewrite.
    assert_eq!(runtime.session().messages[0].role, MessageRole::User);

    // The compaction is still recorded on the session metadata (this is the
    // durable record the projection's summary continuation is rebuilt from).
    let recorded = runtime
        .session()
        .compaction
        .as_ref()
        .expect("compaction must be recorded on Session.compaction");
    assert_eq!(recorded.removed_message_count, 2);

    // RE-POINTED: the System summary now lives in the PROVIDER VIEW, recomputed
    // per turn. The compacting turn's own request was built before compaction
    // ran, so run a second turn and assert on what the provider actually received.
    runtime
        .run_turn("after compaction", None)
        .expect("post-compaction turn should succeed");
    let requests = seen.borrow();
    let last = requests
        .last()
        .expect("recording API should see the post-compaction request");
    assert_eq!(
        last[0].role,
        MessageRole::System,
        "provider view must lead with the System compaction continuation"
    );
    assert!(
        last.iter()
            .any(|message| message_text(message).contains("Conversation summary")),
        "provider view must carry the compacted summary continuation"
    );
}

    #[test]
    fn skips_auto_compaction_below_threshold() {
        struct SimpleApi;
        impl ApiClient for SimpleApi {
            fn stream(
                &mut self,
                _request: ApiRequest,
            ) -> Result<Vec<AssistantEvent>, RuntimeError> {
                Ok(vec![
                    AssistantEvent::TextDelta("done".to_string()),
                    AssistantEvent::Usage(TokenUsage {
                        input_tokens: 99_999,
                        output_tokens: 4,
                        cache_creation_input_tokens: 0,
                        cache_read_input_tokens: 0,
                    }),
                    AssistantEvent::MessageStop,
                ])
            }
        }

        let mut runtime = ConversationRuntime::new(
            Session::new(),
            SimpleApi,
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        )
        .with_auto_compaction_input_tokens_threshold(100_000);

        let summary = runtime
            .run_turn("trigger", None)
            .expect("turn should succeed");
        assert_eq!(summary.auto_compaction, None);
        assert_eq!(runtime.session().messages.len(), 2);
    }

    #[test]
    fn auto_compaction_threshold_defaults_and_parses_values() {
        assert_eq!(
            parse_auto_compaction_threshold(None),
            DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD
        );
        assert_eq!(parse_auto_compaction_threshold(Some("4321")), 4321);
        assert_eq!(
            parse_auto_compaction_threshold(Some("0")),
            DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD
        );
        assert_eq!(
            parse_auto_compaction_threshold(Some("not-a-number")),
            DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD
        );
    }

    #[test]
    fn build_assistant_message_requires_message_stop_event() {
        // given
        let events = vec![AssistantEvent::TextDelta("hello".to_string())];

        // when
        let error = build_assistant_message(events)
            .expect_err("assistant messages should require a stop event");

        // then
        assert!(error
            .to_string()
            .contains("assistant stream ended without a message stop event"));
    }

    #[test]
    fn build_assistant_message_requires_content() {
        // given
        let events = vec![AssistantEvent::MessageStop];

        // when
        let error =
            build_assistant_message(events).expect_err("assistant messages should require content");

        // then
        assert!(error
            .to_string()
            .contains("assistant stream produced no content"));
    }

    #[test]
    fn static_tool_executor_rejects_unknown_tools() {
        // given
        let mut executor = StaticToolExecutor::new();

        // when
        let error = executor
            .execute("missing", "{}")
            .expect_err("unregistered tools should fail");

        // then
        assert_eq!(error.to_string(), "unknown tool: missing");
    }

    #[test]
    fn run_turn_errors_when_max_iterations_is_exceeded() {
        struct LoopingApi;

        impl ApiClient for LoopingApi {
            fn stream(
                &mut self,
                _request: ApiRequest,
            ) -> Result<Vec<AssistantEvent>, RuntimeError> {
                Ok(vec![
                    AssistantEvent::ToolUse {
                        id: "tool-1".to_string(),
                        name: "echo".to_string(),
                        input: "payload".to_string(),
                    },
                    AssistantEvent::MessageStop,
                ])
            }
        }

        // given
        let mut runtime = ConversationRuntime::new(
            Session::new(),
            LoopingApi,
            StaticToolExecutor::new().register("echo", |input| Ok(input.to_string())),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        )
        .with_max_iterations(1);

        // when
        let error = runtime
            .run_turn("loop", None)
            .expect_err("conversation loop should stop after the configured limit");

        // then
        assert!(error
            .to_string()
            .contains("conversation loop exceeded the maximum number of iterations"));
    }

    #[test]
    fn run_turn_propagates_api_errors() {
        struct FailingApi;

        impl ApiClient for FailingApi {
            fn stream(
                &mut self,
                _request: ApiRequest,
            ) -> Result<Vec<AssistantEvent>, RuntimeError> {
                Err(RuntimeError::new("upstream failed"))
            }
        }

        // given
        let mut runtime = ConversationRuntime::new(
            Session::new(),
            FailingApi,
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        );

        // when
        let error = runtime
            .run_turn("hello", None)
            .expect_err("API failures should propagate");

        // then
        assert_eq!(error.to_string(), "upstream failed");
    }

    #[derive(Clone, Default)]
    struct RecordingApi {
        seen: std::rc::Rc<std::cell::RefCell<Vec<Vec<crate::session::ConversationMessage>>>>,
    }

    impl ApiClient for RecordingApi {
        fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
            self.seen.borrow_mut().push(request.messages);
            Ok(vec![
                AssistantEvent::TextDelta("done".to_string()),
                AssistantEvent::Usage(TokenUsage {
                    input_tokens: 120_000,
                    output_tokens: 1,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 0,
                }),
                AssistantEvent::MessageStop,
            ])
        }
    }

    fn projection_test_session() -> Session {
        let mut session = Session::new();
        session.messages = vec![
            crate::session::ConversationMessage::user_text("original-first"),
            crate::session::ConversationMessage::assistant(vec![ContentBlock::Text {
                text: "assistant-one".to_string(),
            }]),
            crate::session::ConversationMessage::user_text("user-two"),
            crate::session::ConversationMessage::assistant(vec![ContentBlock::Text {
                text: "assistant-two".to_string(),
            }]),
            crate::session::ConversationMessage::user_text("user-three"),
            crate::session::ConversationMessage::assistant(vec![ContentBlock::Text {
                text: "assistant-three".to_string(),
            }]),
        ];
        session
    }

    fn message_text(message: &crate::session::ConversationMessage) -> String {
        message
            .blocks
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn run_turn_sends_the_projected_view_not_the_raw_log() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let api = RecordingApi { seen: seen.clone() };
        let mut runtime = ConversationRuntime::new(
            projection_test_session(),
            api,
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        )
        .with_auto_compaction_input_tokens_threshold(1);

        runtime.run_turn("open epoch", None).expect("epoch-opening turn");
        runtime
            .run_turn("after epoch", None)
            .expect("post-epoch turn");

        let requests = seen.borrow();
        let last = requests.last().expect("recording API should see the last request");
        assert!(
            last.len() < requests[0].len(),
            "provider view must be shorter than the immutable transcript"
        );
        assert!(
            last.iter().any(|message| message_text(message).contains("Conversation summary")),
            "provider view must contain the compacted summary continuation"
        );
    }

    #[test]
    fn run_turn_reuses_the_frozen_prefix_on_a_quiet_second_turn_freeze() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            projection_test_session(), RecordingApi { seen: seen.clone() },
            StaticToolExecutor::new(), PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        ).with_auto_compaction_input_tokens_threshold(1);
        runtime.run_turn("warm-up", None).expect("warm-up turn");
        // The warm-up turn's compaction is what opens the projection epoch. Raise the
        // threshold now so no further compaction fires: a compaction invalidates the
        // frozen prefix by design, so the turn under test must genuinely be quiet.
        let mut runtime = runtime.with_auto_compaction_input_tokens_threshold(u32::MAX);
        assert_eq!(runtime.last_freeze_decision(), None);
        assert_eq!(runtime.freeze_epoch_number(), 0);
        runtime.run_turn("first freeze", None).expect("first freeze turn");
        assert_eq!(runtime.last_freeze_decision(), Some(crate::context_projection::FreezeDecision::Recompute(crate::context_projection::RecomputeCause::FirstCall)));
        assert_eq!(runtime.freeze_epoch_number(), 1);
        let second = seen.borrow()[1].clone();
        runtime.run_turn("second freeze", None).expect("second freeze turn");
        assert_eq!(runtime.last_freeze_decision(), Some(crate::context_projection::FreezeDecision::Reuse));
        assert_eq!(runtime.freeze_epoch_number(), 1);
        let third = seen.borrow()[2].clone();
        let frozen_head_len = runtime.freeze_epoch.as_ref().expect("freeze epoch").freeze.frozen_head_len;
        // The frozen head is stable on a Reuse turn; the continuation is not yet, and full prefix-byte stability belongs to the append-tail slice (L-17 slice 2).
        assert_eq!(second[..frozen_head_len], third[..frozen_head_len]);
    }

    #[test]
    fn run_turn_reuse_keeps_the_entire_cached_prefix_byte_stable() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            projection_test_session(), RecordingApi { seen: seen.clone() },
            StaticToolExecutor::new(), PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        ).with_auto_compaction_input_tokens_threshold(1);
        runtime.run_turn("warm-up", None).expect("warm-up turn");
        // The warm-up turn's compaction is what opens the projection epoch. Raise the
        // threshold now so no further compaction fires: a compaction invalidates the
        // frozen prefix by design, so the turn under test must genuinely be quiet.
        let mut runtime = runtime.with_auto_compaction_input_tokens_threshold(u32::MAX);
        runtime.run_turn("first freeze", None).expect("first freeze turn");
        let first = seen.borrow()[1].clone();
        runtime.run_turn("second freeze", None).expect("second freeze turn");
        assert_eq!(runtime.last_freeze_decision(), Some(crate::context_projection::FreezeDecision::Reuse));
        let second = seen.borrow()[2].clone();
        let frozen_head_len = runtime.freeze_epoch.as_ref().expect("freeze epoch").freeze.frozen_head_len;
        let prefix_len = frozen_head_len + 1;
        assert_eq!(first[..prefix_len], second[..prefix_len], "frozen prefix content must be byte-stable");
        assert_eq!(message_text(second.last().expect("tail message")), "second freeze");
    }

    #[test]
    fn run_turn_recomputes_when_the_idle_gap_exceeds_the_ttl_freeze() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            projection_test_session(), RecordingApi { seen: seen.clone() },
            StaticToolExecutor::new(), PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        ).with_auto_compaction_input_tokens_threshold(1);
        runtime.run_turn("warm-up", None).expect("warm-up turn");
        let mut runtime = runtime.with_auto_compaction_input_tokens_threshold(u32::MAX);
        runtime.run_turn("first freeze", None).expect("first freeze turn");
        {
            let epoch = runtime.freeze_epoch.as_mut().expect("freeze epoch");
            let ttl_ms = u64::from(crate::cache_economics::TTL_SHORT_SECS) * 1000;
            epoch.computed_at_ms = epoch.computed_at_ms.saturating_sub(ttl_ms + 1_000);
        }
        runtime.run_turn("second freeze", None).expect("second freeze turn");
        assert_eq!(
            runtime.last_freeze_decision(),
            Some(crate::context_projection::FreezeDecision::Recompute(
                crate::context_projection::RecomputeCause::ColdGapTtl,
            ))
        );
    }

    #[test]
    fn run_turn_local_policy_never_cold_gaps_freeze() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            projection_test_session(), RecordingApi { seen: seen.clone() },
            StaticToolExecutor::new(), PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        )
        .with_auto_compaction_input_tokens_threshold(1)
        .with_context_backend_mode(crate::dispatcher::BackendMode::Local);
        runtime.run_turn("warm-up", None).expect("warm-up turn");
        let mut runtime = runtime.with_auto_compaction_input_tokens_threshold(u32::MAX);
        runtime.run_turn("first freeze", None).expect("first freeze turn");
        {
            let epoch = runtime.freeze_epoch.as_mut().expect("freeze epoch");
            let ttl_ms = u64::from(crate::cache_economics::TTL_SHORT_SECS) * 1000;
            epoch.computed_at_ms = epoch.computed_at_ms.saturating_sub(ttl_ms + 1_000);
        }
        runtime.run_turn("second freeze", None).expect("second freeze turn");
        assert_eq!(
            runtime.last_freeze_decision(),
            Some(crate::context_projection::FreezeDecision::Reuse)
        );
    }

    #[test]
    fn run_turn_recomputes_after_an_auto_compaction_moves_the_generation_freeze() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            projection_test_session(), RecordingApi { seen: seen.clone() },
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        ).with_auto_compaction_input_tokens_threshold(1);
        runtime.run_turn("warm-up", None).expect("warm-up turn");
        runtime.run_turn("first freeze", None).expect("first freeze turn");
        let expected_count = runtime
            .session
            .compaction
            .as_ref()
            .expect("a compaction before the third turn")
            .count;
        assert!(
            expected_count > 1,
            "a second compaction must have fired before the third turn"
        );
        let second = seen.borrow()[1].clone();
        runtime.run_turn("second freeze", None).expect("second freeze turn");

        let third = seen.borrow()[2].clone();
        let frozen_head_len = runtime.freeze_epoch.as_ref().expect("freeze epoch").freeze.frozen_head_len;
        let prefix_len = frozen_head_len + 1;
        assert_ne!(
            second[..prefix_len],
            third[..prefix_len],
            "a compaction must invalidate the frozen prefix; the provider is still receiving the stale bytes"
        );
        assert_eq!(
            runtime.last_freeze_decision(),
            Some(crate::context_projection::FreezeDecision::Recompute(
                crate::context_projection::RecomputeCause::PolicyChanged,
            ))
        );
    }

    #[test]
    fn run_turn_recomputes_when_the_boundary_message_changes_freeze() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            projection_test_session(), RecordingApi { seen }, StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess), vec!["system".to_string()],
        ).with_auto_compaction_input_tokens_threshold(1);
        runtime.run_turn("warm-up", None).expect("warm-up turn");
        assert_eq!(runtime.last_freeze_decision(), None);
        assert_eq!(runtime.freeze_epoch_number(), 0);
        runtime.run_turn("first freeze", None).expect("first freeze turn");
        assert_eq!(runtime.last_freeze_decision(), Some(crate::context_projection::FreezeDecision::Recompute(crate::context_projection::RecomputeCause::FirstCall)));
        assert_eq!(runtime.freeze_epoch_number(), 1);
        runtime.session.messages[0] = crate::session::ConversationMessage::user_text("changed boundary");
        runtime.run_turn("second freeze", None).expect("second freeze turn");
        assert_eq!(runtime.last_freeze_decision(), Some(crate::context_projection::FreezeDecision::Recompute(crate::context_projection::RecomputeCause::BoundaryMismatch)));
        assert_eq!(runtime.freeze_epoch_number(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn recompute_journals_typed_boundary_mismatch() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = crate::ssb::SsbConfig {
            db_path: dir.path().join(format!("ssb-{}.db", std::process::id())),
            pool_size: Some(2),
            project_id: "l17-slice2".to_string(),
            session_id: "l17-slice2-session".to_string(),
        };
        let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
            std::sync::Arc::new(elai_core::clock::OsClock);
        let ssb = std::sync::Arc::new(crate::ssb::Ssb::new(cfg, clock).expect("ssb init"));
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            projection_test_session(), RecordingApi { seen }, StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess), vec!["system".to_string()],
        )
        .with_auto_compaction_input_tokens_threshold(1)
        .with_ssb_writer(ssb.clone());
        runtime.run_turn("warm-up", None).expect("warm-up turn");
        runtime.run_turn("first freeze", None).expect("first freeze turn");
        runtime.session.messages[0] = crate::session::ConversationMessage::user_text("changed boundary");
        runtime.run_turn("second freeze", None).expect("second freeze turn");
        let boundary_hash = runtime.freeze_epoch.as_ref().expect("freeze epoch").boundary_hash;
        use crate::ssb::SsbReader;
        let rows = ssb
            .query_by_type(crate::ssb::SsbActionType::CacheEconomicsEvent(
                crate::ssb::CacheEconomicsEventKind::TtlPlanned,
            ))
            .expect("query cache economics rows");
        let found = rows.into_iter().filter_map(|row| {
            let payload: crate::ssb::SsbPayload = rmp_serde::from_slice(&row.payload).ok()?;
            match payload {
                crate::ssb::SsbPayload::CacheEconomicsEvent(payload)
                    if payload.kind
                        == crate::ssb::CacheEconomicsEventKind::PrefixRecomputed(
                            crate::ssb::FreezePrefixRecomputeCause::BoundaryMismatch,
                        ) && payload.mutation_hash == boundary_hash => Some(()),
                _ => None,
            }
        }).next();
        assert!(found.is_some(), "typed BoundaryMismatch recompute row missing");
    }

    #[test]
    fn run_turn_recomputes_when_the_regime_policy_changes_freeze() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            projection_test_session(), RecordingApi { seen }, StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess), vec!["system".to_string()],
        ).with_auto_compaction_input_tokens_threshold(1);
        runtime.run_turn("warm-up", None).expect("warm-up turn");
        assert_eq!(runtime.last_freeze_decision(), None);
        assert_eq!(runtime.freeze_epoch_number(), 0);
        runtime.run_turn("first freeze", None).expect("first freeze turn");
        assert_eq!(runtime.last_freeze_decision(), Some(crate::context_projection::FreezeDecision::Recompute(crate::context_projection::RecomputeCause::FirstCall)));
        assert_eq!(runtime.freeze_epoch_number(), 1);
        runtime = runtime.with_context_backend_mode(crate::dispatcher::BackendMode::Local);
        runtime.run_turn("second freeze", None).expect("second freeze turn");
        assert_eq!(runtime.last_freeze_decision(), Some(crate::context_projection::FreezeDecision::Recompute(crate::context_projection::RecomputeCause::PolicyChanged)));
    }

    #[test]
    fn run_turn_recomputes_when_the_tail_outgrows_the_preserve_window_freeze() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            projection_test_session(), RecordingApi { seen }, StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess), vec!["system".to_string()],
        ).with_auto_compaction_input_tokens_threshold(1);
        runtime.run_turn("warm-up", None).expect("warm-up turn");
        assert_eq!(runtime.last_freeze_decision(), None);
        assert_eq!(runtime.freeze_epoch_number(), 0);
        runtime.run_turn("first freeze", None).expect("first freeze turn");
        assert_eq!(runtime.last_freeze_decision(), Some(crate::context_projection::FreezeDecision::Recompute(crate::context_projection::RecomputeCause::FirstCall)));
        assert_eq!(runtime.freeze_epoch_number(), 1);
        for index in 0..5 { runtime.session.messages.push(crate::session::ConversationMessage::user_text(format!("extra freeze {index}"))); }
        runtime.run_turn("second freeze", None).expect("second freeze turn");
        assert_eq!(runtime.last_freeze_decision(), Some(crate::context_projection::FreezeDecision::Recompute(crate::context_projection::RecomputeCause::TailOverflow)));
    }

    #[test]
    fn projection_disable_rollback_keeps_the_raw_log_and_no_decision_freeze() {
        crate::context_projection::set_projection_disabled_for_test(Some(true));
        let original = projection_test_session();
        let mut expected_request = original.messages.clone();
        expected_request.push(crate::session::ConversationMessage::user_text("raw freeze"));
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            original, RecordingApi { seen: seen.clone() }, StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess), vec!["system".to_string()],
        ).with_auto_compaction_input_tokens_threshold(1);
        runtime.run_turn("raw freeze", None).expect("turn");
        let request = seen.borrow()[0].clone();
        assert_eq!(request, expected_request);
        assert_eq!(runtime.last_freeze_decision(), None);
        crate::context_projection::set_projection_disabled_for_test(None);
    }

    fn oversized_projection_session() -> Session {
        let mut session = Session::new();
        session
            .messages
            .push(crate::session::ConversationMessage::user_text("original-first"));
        let filler = "x".repeat(4_000);
        let threshold =
            crate::context_policy::trim_threshold_for_mode(crate::dispatcher::BackendMode::Api)
                as usize;
        while crate::compact::estimate_session_tokens(&session) <= threshold {
            session
                .messages
                .push(crate::session::ConversationMessage::user_text(&filler));
            session
                .messages
                .push(crate::session::ConversationMessage::assistant(vec![
                    ContentBlock::Text {
                        text: filler.clone(),
                    },
                ]));
        }
        session
    }

    #[test]
    fn oversized_log_is_projected_without_any_compaction_event() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let session = oversized_projection_session();
        let log_len = session.messages.len();
        let first_message = session.messages[0].clone();
        let mut runtime = ConversationRuntime::new(
            session,
            RecordingApi { seen: seen.clone() },
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        )
        .with_auto_compaction_input_tokens_threshold(u32::MAX);

        runtime.run_turn("first turn", None).expect("turn");

        let requests = seen.borrow();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].len() < log_len);
        assert!(requests[0]
            .iter()
            .any(|message| message_text(message).contains("Conversation summary")));
        assert!(
            runtime.session().messages.len() >= log_len,
            "CC-1: the transcript log must never shrink"
        );
        assert_eq!(runtime.session().messages[0], first_message);
        assert!(runtime.session().compaction.is_none());
    }

    #[test]
    fn auto_compaction_never_shortens_the_transcript_log() {
        let original = projection_test_session();
        let first_message = original.messages[0].clone();
        let before = original.messages.len();
        let mut runtime = ConversationRuntime::new(
            original,
            RecordingApi::default(),
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        )
        .with_auto_compaction_input_tokens_threshold(1);

        runtime.run_turn("trigger compaction", None).expect("turn");

        assert!(runtime.session().messages.len() > before);
        assert_eq!(runtime.session().messages[0], first_message);
    }

    #[test]
    fn provider_view_includes_messages_appended_after_the_epoch() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let mut runtime = ConversationRuntime::new(
            projection_test_session(),
            RecordingApi { seen: seen.clone() },
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        )
        .with_auto_compaction_input_tokens_threshold(1);

        runtime.run_turn("open epoch", None).expect("epoch-opening turn");
        runtime
            .run_turn("needle-after-epoch", None)
            .expect("post-epoch turn");

        let requests = seen.borrow();
        let last = requests.last().expect("post-epoch request should be recorded");
        assert!(
            last.iter().any(|message| message_text(message).contains("needle-after-epoch")),
            "freshly appended post-epoch user text must reach the provider"
        );
    }

    #[test]
    fn projection_disabled_keeps_the_legacy_mutating_path() {
        crate::context_projection::set_projection_disabled_for_test(Some(true));

        struct RecordingHighUsageApi;
        impl ApiClient for RecordingHighUsageApi {
            fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
                let _ = request;
                Ok(vec![
                    AssistantEvent::TextDelta("done".to_string()),
                    AssistantEvent::Usage(TokenUsage {
                        input_tokens: 120_000,
                        output_tokens: 4,
                        cache_creation_input_tokens: 0,
                        cache_read_input_tokens: 0,
                    }),
                    AssistantEvent::MessageStop,
                ])
            }
        }

        let session = projection_test_session();
        let before = session.messages.len();
        let mut runtime = ConversationRuntime::new(
            session,
            RecordingHighUsageApi,
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        )
        .with_auto_compaction_input_tokens_threshold(100_000);
        let turn_result = runtime.run_turn("legacy compaction", None);
        let shortened = runtime.session().messages.len() < before;

        crate::context_projection::set_projection_disabled_for_test(None);
        turn_result.expect("turn");
        assert!(shortened, "rollback mode should retain the legacy mutating path");
    }


    #[test]
    fn l19_production_view_appends_recall_card_when_folded_path_resurfaces() {
        let mut session = Session::new();
        session.messages = vec![
            crate::session::ConversationMessage {
                role: MessageRole::System,
                blocks: vec![ContentBlock::Text { text: "head".to_string() }],
                usage: None,
            },
            crate::session::ConversationMessage::user_text("crates/orchestrator/src/session.rs zebrafish"),
            crate::session::ConversationMessage::assistant(vec![ContentBlock::Text { text: "middle".to_string() }]),
            crate::session::ConversationMessage::user_text("tail crates/orchestrator/src/session.rs"),
        ];
        let mut runtime = ConversationRuntime::new(
            session,
            RecordingApi { seen: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())) },
            StaticToolExecutor::new(),
            PermissionPolicy::new(PermissionMode::DangerFullAccess),
            vec!["system".to_string()],
        );
        runtime.projection_epoch = Some(CompactionConfig { preserve_recent_messages: 1, ..CompactionConfig::default() });
        let messages = runtime.freeze_gated_view_messages();
        let recall = messages.iter().rev().find(|message| {
            message.role == MessageRole::System
                && matches!(message.blocks.first(), Some(ContentBlock::Text { text }) if text.starts_with("[recall]"))
        });
        assert!(recall.is_some(), "production view should append a recall card after the preserved tail");
        assert!(matches!(recall.unwrap().blocks.first(), Some(ContentBlock::Text { text }) if text.contains("crates/orchestrator/src/session.rs")));
    }

}
