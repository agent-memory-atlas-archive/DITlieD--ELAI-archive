//! Batch 1A — in-process hook dispatcher.
//!
//! Unblocks the "built but unwired" T1/T2/T3 tiered-analysis surface on the
//! task_runner baby-step loop. The dispatcher is an async trait with four hook
//! points (PreToolUse, PostToolUse, Stop, PreCompact). Verdicts flow back as
//! [`HookFinding`] values; the task_runner emits an [`SsbPayload::HookEvent`]
//! row per verdict so the dashboard activity feed can surface them as
//! "hook · T1 BLOCK" / "hook · T2 WARN" style rows.
//!
//! Scope note: this module is the DISPATCHER only. The actual T1 (predicate
//! DSL / regex), T2 (deprecated-import / AST), and T3 (reasoning) analyses
//! live in `elai_plugins::TieredAnalysis` and are reused verbatim — we do
//! NOT duplicate them here.
//!
//! Grounding:
//! - Existing tiered analysis: `elai_plugins::tiered_analysis::TieredAnalysis`
//!   + `HookVerdict` (Allow / Warn / Block).
//! - Dispatch sites per `docs/mvp-dashboard-style-and-wiring.md` §2.3.
//! - Escape hatch via `ELAI_HOOKS_DISABLED` matches the env-var pattern used
//!   by other dashboard-side toggles (see `dashboard/src/lib.rs` setup).

use rustc_hash::FxHashMap;
use std::ffi::OsString;
use std::path::Path;
use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, info, trace, warn};

use elai_core::clock::{ClockProvider, OsClock};
use elai_plugins::post_tool_use::{
    RuffFinding, RuffOutcome, RuffPostToolUseRunner, RuffSkipReason,
};
use elai_plugins::tiered_analysis::{HookVerdict, TieredAnalysis, TieredAnalysisError};

use crate::gate::InteractiveGate;
use crate::shortcoder_hook::{ShortCoderHook, ShortCoderOutcome};
use crate::ssb::{
    HookEventPayload, HookLatencyViolationPayload, ShortCoderCompressionPayload, Ssb,
    SsbPayload, StopConfirmationPayload, StopOutcomeKind, TieredAnalysisPayload,
};
use crate::ssb_viewer::SsbViewer;
use crate::stop_cost_gate::{evaluate_stop, StopCostSnapshot, StopGateVerdict};
use crate::vibesec::dep_writetime_gate::DepWriteTimeGate;

/// H-38: env var that gates SSB telemetry for the cost-confirm gate on the
/// pass-no-friction path. When set to `"1"`, a `stop_confirmation_event` row
/// with `outcome=PassedNoFriction` is emitted for low-cost fully-saved stops.
/// On the ConfirmRequired / RejectedHeld paths the row is emitted unconditionally
/// (the safety behavior is always on). Default-OFF per R9.
///
/// Registered in wire-dark-allowlist.toml (H-38 experiment row).
pub const ELAI_STOP_COST_CONFIRM_ENABLE: &str = "ELAI_STOP_COST_CONFIRM_ENABLE";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// SSB finding / action category for hook-dispatcher emissions.
///
/// The frontend `ActivityEntry.svelte` keys on the action_type string
/// `"hook_event"`, but we also pass this constant as the `category` field
/// on every [`HookEventPayload`] so downstream filters and logs have a
/// stable tag regardless of any future renames.
pub const HOOK_EVENT_CATEGORY: &str = "hook_event";

/// E-30: action_type value used when the caller is dispatching a Bash/shell
/// tool invocation. `dispatch_pre_tool_use` runs `check_argv()` when this
/// action_type is seen and `source` contains the command string.
pub const BASH_TOOL_USE_ACTION: &str = "bash_tool_use";

/// Env var that globally disables hook dispatch. Set to `1` / `true` / `on`
/// to route every `dispatch_*` call to an empty-findings no-op.
///
/// Matches the env-var escape-hatch pattern used by other dashboard toggles.
pub const ELAI_HOOKS_DISABLED_ENV: &str = "ELAI_HOOKS_DISABLED";

/// Batch 2A — sanitized user-facing message for Block verdicts surfaced on
/// the IPC surface. The agent MUST only ever see this string plus the
/// finding's category tag; the detailed `rule_id` / pattern hash lives
/// exclusively on the SSB row for the dashboard.
pub const SANITIZED_BLOCK_MESSAGE: &str = "tool call rejected by policy";

// ---------------------------------------------------------------------------
// B-19A (T10) — hook latency budgets per spec
// ---------------------------------------------------------------------------

/// T1 (predicate / regex) tier latency budget in milliseconds.
/// Source: B-19 plan §"T10 latency budgets" — T1 hooks must complete in
/// <= 1ms because they run on the synchronous baby-step pre-gate hot path.
/// Overruns emit one SSB `hook_latency_violation` row per dispatch.
pub const HOOK_T1_BUDGET_MS: u64 = 1;

/// T2 (deprecated-import / AST) tier latency budget in milliseconds.
/// AST parsing tolerates more headroom than T1 — spec sets the ceiling at
/// 10ms because tree-sitter parses on tens-of-KB Python/Rust source files
/// in a single-digit ms range; anything beyond that means a stuck parser.
pub const HOOK_T2_BUDGET_MS: u64 = 10;

/// T3 (reasoning / advisory) tier latency budget in milliseconds.
/// Spec sets the ceiling at 5ms — T3 does message-pattern matching on the
/// rule output, not full LLM reasoning, so any overrun signals a
/// pathological rule.
pub const HOOK_T3_BUDGET_MS: u64 = 5;

/// B-19: spec handoff threshold mirroring
/// `crate::escalation_policy::HANDOFF_THRESHOLD_FRACTION` (0.85). Re-exposed
/// here so `dispatch_pre_compact` can gate the SessionDag handoff trigger
/// without taking a cross-module dep at the dispatcher hot path.
/// Source: `docs/specs/guide/steps/phase-4C.txt:28-29` —
/// "PreCompact fires when token_used/token_limit > 0.85".
pub const PRE_COMPACT_HANDOFF_FRACTION: f64 = 0.85;

/// Produce the agent-facing sanitized message for a BLOCK finding.
///
/// Format: `"tool call rejected by policy (<category>)"`. The rule_id
/// and original message are intentionally dropped so an adversarial
/// agent can't probe the rule set by triggering BLOCKs.
#[must_use]
pub fn sanitized_block_message(category: &str) -> String {
    if category.is_empty() {
        SANITIZED_BLOCK_MESSAGE.to_string()
    } else {
        format!("{SANITIZED_BLOCK_MESSAGE} ({category})")
    }
}

// ---------------------------------------------------------------------------
// Value types
// ---------------------------------------------------------------------------

/// The four hook points driven by the task_runner baby-step loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookPoint {
    /// Fires before [`BabyStepExecutor::execute`](crate::task_runner::BabyStepExecutor).
    /// Block verdicts abort the baby step.
    PreToolUse,
    /// Fires after `emit_checkpoint_outcome` when the step passed AND produced
    /// at least one changed file. Warn -> Finding. Block -> flip passed=false.
    PostToolUse,
    /// Fires once per task on drive_task completion (success OR hard fail).
    Stop,
    /// Fires when the context budget crosses the handoff threshold. MVP:
    /// no-op wiring surface; the real compaction site lands in a later batch.
    PreCompact,
}

impl HookPoint {
    /// Stable string label surfaced in SSB rows and log events.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreToolUse => "PreToolUse",
            Self::PostToolUse => "PostToolUse",
            Self::Stop => "Stop",
            Self::PreCompact => "PreCompact",
        }
    }
}

/// Which of the three tiers produced a verdict.
///
/// Mirrors the T1/T2/T3 shape already implemented in
/// `elai_plugins::tiered_analysis`. T2 is only runnable when an actual file
/// write is in flight; the dispatcher reports `Composite` when the verdict
/// was produced by [`TieredAnalysis::analyze`]'s combined output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookTier {
    T1,
    T2,
    T3,
    /// Final verdict after T1/T2/T3 merge (see `TieredAnalysisResult::final_verdict`).
    Composite,
}

impl HookTier {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::T1 => "T1",
            Self::T2 => "T2",
            Self::T3 => "T3",
            Self::Composite => "Composite",
        }
    }
}

/// Flat verdict used on the dispatcher boundary. The tiered-analysis crate
/// carries richer payloads (`pattern_hash`, etc.) which stay inside the
/// original [`HookVerdict`]; this enum is the shape emitted on SSB so the
/// dashboard can filter on it without re-implementing pattern hashing.
///
/// Severity order mirrors the producer-side merge in
/// `elai_plugins::tiered_analysis::T1Hook::merge_verdicts`: Block > Escalate >
/// Warn > Allow. `Escalate` is NOT a clean outcome — the hook is not confident
/// enough to Allow or Block on its own and the decision is routed to a higher
/// tier (auditor / human / a named next-tier rule); it surfaces as a finding
/// between Warn and Block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookVerdictKind {
    Allow,
    Warn,
    Escalate,
    Block,
}

impl HookVerdictKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "Allow",
            Self::Warn => "Warn",
            Self::Escalate => "escalate",
            Self::Block => "Block",
        }
    }

    #[must_use]
    pub fn from_verdict(v: &HookVerdict) -> Self {
        match v {
            HookVerdict::Allow => Self::Allow,
            HookVerdict::Warn { .. } => Self::Warn,
            HookVerdict::Escalate { .. } => Self::Escalate,
            HookVerdict::Block { .. } => Self::Block,
        }
    }

    #[must_use]
    pub fn is_block(self) -> bool {
        matches!(self, Self::Block)
    }

    #[must_use]
    pub fn is_warn(self) -> bool {
        matches!(self, Self::Warn)
    }

    /// True when this verdict escalated the decision to a higher tier.
    /// Distinct from `is_warn`/`is_block`: an escalation is unresolved, not
    /// advisory (Warn) and not a hard reject (Block).
    #[must_use]
    pub fn is_escalate(self) -> bool {
        matches!(self, Self::Escalate)
    }
}

/// A single finding produced by a hook at one tier. The dispatcher produces
/// one finding per non-Allow verdict (plus an informational Allow finding per
/// Composite run when requested).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookFinding {
    pub tier: HookTier,
    pub verdict: HookVerdictKind,
    pub rule_id: Option<String>,
    pub category: String,
    pub message: String,
    /// Optional fix-template hint. Reserved for T3 reasoning output; the
    /// default in-process dispatcher leaves this `None` unless the plugins
    /// crate surfaces one in a future update.
    pub fix_template: Option<String>,
}

impl HookFinding {
    #[must_use]
    pub fn is_block(&self) -> bool {
        self.verdict.is_block()
    }

    #[must_use]
    pub fn is_warn(&self) -> bool {
        self.verdict.is_warn()
    }
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum HookError {
    #[error("tiered analysis failure: {0}")]
    Tiered(String),

    #[error("dispatcher is disabled by {ELAI_HOOKS_DISABLED_ENV} env var")]
    Disabled,
}

impl From<TieredAnalysisError> for HookError {
    fn from(err: TieredAnalysisError) -> Self {
        Self::Tiered(format!("{err}"))
    }
}

// ---------------------------------------------------------------------------
// Input context
// ---------------------------------------------------------------------------

/// Context passed to every [`HookDispatcher`] call. Fields are optional so the
/// four hook points can share a single struct without forcing callers to
/// invent placeholder values for irrelevant fields (Stop/PreCompact have no
/// source/file).
#[derive(Debug, Clone, Default)]
pub struct HookInvocation<'a> {
    /// Logical action name (e.g. `"baby_step_pre_gate"`, `"file_write"`,
    /// `"task_stop"`). Fed straight into the underlying rule engine as its
    /// `action` discriminant.
    pub action_type: &'a str,
    /// The source text being evaluated. `None` for Stop/PreCompact.
    pub source: Option<&'a str>,
    /// The changed / candidate file path. `None` when no file is in flight.
    pub file_path: Option<&'a str>,
    /// Optional metric values consumed by the rule engine's MetricGate
    /// predicate. Empty by default.
    pub metric_values: FxHashMap<String, f64>,
    /// Step identifier emitted onto the SSB HookEvent row (e.g.
    /// `"baby_step_42"`). Reserved for correlation with TestOutcome rows.
    pub step_id: String,
    /// B-19: current FSM state label (e.g. `"IDLE"`, `"EXECUTING"`).
    /// Used by `dispatch_stop` to block termination in non-idle states.
    /// Empty string (default) skips the FSM check — backward-compatible for
    /// callers that have not yet plumbed FSM state into the invocation.
    pub fsm_state: String,
    /// B-19: optional `(token_used, token_limit)` snapshot consumed by
    /// `dispatch_pre_compact`. When `Some` AND
    /// `token_used as f64 / token_limit as f64 <= 0.85` (the spec
    /// `HANDOFF_THRESHOLD_FRACTION` from `escalation_policy.rs:34`), the
    /// dispatcher short-circuits to `Ok(empty)` — no handoff snapshot is
    /// taken and no SSB row is emitted. This honours
    /// `docs/specs/guide/steps/phase-4C.txt:28-29` (`PreCompact fires when
    /// token_used/token_limit > 0.85`) without the call-side having to
    /// know the threshold constant.
    /// `None` means "caller has no token budget data; do not apply the
    /// gate" — the dispatcher runs PreCompact analyze() as before. This
    /// preserves backward-compat for the per-task PreCompact firing site
    /// in `task_runner.rs` which has no token telemetry.
    pub pre_compact_token_state: Option<(u32, u32)>,
    /// H-38: number of unsaved / pending-write artifacts in the current run.
    /// Default 0 (no unsaved work). Plumbed from the run's pending-write set
    /// count at the task_runner stop-call site. `0` means fully saved.
    pub unsaved_artifact_count: usize,
    /// H-38: whether a billable pod is currently running.
    /// Default `false`. The ELAI Rust layer cannot query pod state directly
    /// (pod lifecycle is a harness MCP concern); this field exists so harness
    /// callers can pass `true` once wired. Set via
    /// `HookInvocation::with_pod_billable`.
    pub pod_billable: bool,
}

impl<'a> HookInvocation<'a> {
    /// Helper for the common "Stop/PreCompact" call where only a step_id is set.
    #[must_use]
    pub fn for_task_level(action_type: &'a str, step_id: impl Into<String>) -> Self {
        Self {
            action_type,
            source: None,
            file_path: None,
            metric_values: FxHashMap::default(),
            step_id: step_id.into(),
            fsm_state: String::new(),
            pre_compact_token_state: None,
            unsaved_artifact_count: 0,
            pod_billable: false,
        }
    }

    /// H-38: builder-style setter for unsaved artifact count.
    #[must_use]
    pub fn with_unsaved_artifact_count(mut self, count: usize) -> Self {
        self.unsaved_artifact_count = count;
        self
    }

    /// H-38: builder-style setter for pod billable flag.
    #[must_use]
    pub fn with_pod_billable(mut self, billable: bool) -> Self {
        self.pod_billable = billable;
        self
    }

    /// Helper for the baby-step `PreToolUse` call.
    #[must_use]
    pub fn for_pre_tool_use(
        action_type: &'a str,
        source: &'a str,
        step_id: impl Into<String>,
    ) -> Self {
        Self {
            action_type,
            source: Some(source),
            file_path: None,
            metric_values: FxHashMap::default(),
            step_id: step_id.into(),
            fsm_state: String::new(),
            pre_compact_token_state: None,
            unsaved_artifact_count: 0,
            pod_billable: false,
        }
    }

    /// Helper for the baby-step `PostToolUse` call (file_path + source).
    #[must_use]
    pub fn for_post_tool_use(
        action_type: &'a str,
        source: &'a str,
        file_path: &'a str,
        step_id: impl Into<String>,
    ) -> Self {
        Self {
            action_type,
            source: Some(source),
            file_path: Some(file_path),
            metric_values: FxHashMap::default(),
            step_id: step_id.into(),
            fsm_state: String::new(),
            pre_compact_token_state: None,
            unsaved_artifact_count: 0,
            pod_billable: false,
        }
    }

    /// Set the FSM state for Stop-hook enforcement. Chain after any constructor.
    #[must_use]
    pub fn with_fsm_state(mut self, state: impl Into<String>) -> Self {
        self.fsm_state = state.into();
        self
    }

    /// B-19: set the `(token_used, token_limit)` snapshot consumed by
    /// `dispatch_pre_compact`. Callers that know the live context budget
    /// (e.g. the chat-session layer that owns the message history) chain
    /// this after `for_task_level("pre_compact", ...)` so the spec-required
    /// 0.85 gate (`HANDOFF_THRESHOLD_FRACTION`) is enforced at the
    /// dispatcher boundary rather than at every callsite.
    #[must_use]
    pub fn with_pre_compact_token_state(mut self, used: u32, limit: u32) -> Self {
        self.pre_compact_token_state = Some((used, limit));
        self
    }
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Production hook dispatcher surface. Implementations MUST be `Send + Sync`
/// so a single `Arc<dyn HookDispatcher>` can be shared across the task_runner's
/// baby-step loop. All four dispatch methods return the list of findings; a
/// non-empty list with at least one `Block` verdict MUST be treated as a hard
/// abort signal by the caller.
pub trait HookDispatcher: Send + Sync {
    fn dispatch_pre_tool_use<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    >;

    fn dispatch_post_tool_use<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    >;

    fn dispatch_stop<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    >;

    fn dispatch_pre_compact<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    >;

    /// Batch 4B: run the dispatcher's optional ShortCoder compression hook
    /// on a PostToolUse payload. Default returns `None` so every existing
    /// implementor stays backward-compatible without edits. The task_runner
    /// feeds the compressed output to the auditor when this returns Some —
    /// the raw `invocation.source` remains the executor's record of truth.
    #[allow(unused_variables)]
    fn shortcode(
        &self,
        source: &str,
        file_path: Option<&str>,
    ) -> Option<ShortCoderOutcome> {
        None
    }

    /// I-1: expose the installed directive supervisor so the task_runner can
    /// route production side effects through `Supervisor::execute_directive`.
    /// Default returns `None`; `InProcessHookDispatcher` overrides to return
    /// the supervisor it holds (the same instance that `run_gates` delegates
    /// to during `dispatch_pre_tool_use`). Callers holding an
    /// `Arc<dyn HookDispatcher>` can call this without downcasting.
    fn supervisor(&self) -> Option<Arc<dyn crate::directive::Supervisor>> {
        None
    }
}

// ---------------------------------------------------------------------------
// Default in-process implementation
// ---------------------------------------------------------------------------

/// B-19 — handoff-save callback. The PreCompact dispatch handler invokes
/// this callback (when installed AND the token-budget gate passes) to
/// trigger a `SessionDag::create_handoff` snapshot at the spec-required
/// 0.85 threshold. The dispatcher itself does not own the SessionDag /
/// message history — that data lives in the chat-session layer (e.g.
/// `cli::live_cli`) — so we surface the responsibility via a trait the
/// owning layer implements.
///
/// Implementations MUST be `Send + Sync` since the dispatcher is shared
/// across the task_runner via `Arc<dyn HookDispatcher>`. Implementations
/// are synchronous because `create_handoff` is sync (it uses rusqlite
/// directly, not tokio-rusqlite); the dispatcher wraps the call in
/// `tokio::task::spawn_blocking` so the async runtime stays unblocked.
///
/// Returning `Err` is treated as a soft-fail: the dispatcher logs the
/// error via `tracing::warn` and returns `Ok(empty)` so a snapshot-save
/// failure never aborts an in-flight task. Hard-failure semantics would
/// regress prior B-19 W2 behaviour where PreCompact was strictly
/// best-effort.
pub trait PreCompactHandoffSaver: Send + Sync {
    /// Trigger one handoff snapshot. `step_id` is forwarded from the
    /// PreCompact `HookInvocation` so the saver can correlate the
    /// snapshot back to the originating task.
    fn save_handoff(&self, step_id: &str) -> Result<(), String>;
}

/// In-process hook dispatcher that drives `elai_plugins::TieredAnalysis`
/// (T1 predicate rules, T2 deprecated-import / AST, T3 reasoning) for each
/// hook point. Stop and PreCompact currently evaluate T1 only — T2/T3 are
/// gated on `action_type == "file_write"` inside `TieredAnalysis::analyze`,
/// and neither task-lifecycle event carries a file.
///
/// Batch 2A: also holds an optional [`ShortCoderHook`] that fires from
/// `dispatch_post_tool_use` BEFORE the tiered pass — the sanitized
/// compressed output is what the auditor reads while the executor's
/// `invocation.source` stays untouched (this struct is stateless on the
/// input string — it only takes `&HookInvocation` by reference).
pub struct InProcessHookDispatcher {
    tiered: Option<Arc<TieredAnalysis>>,
    shortcoder: Option<Arc<ShortCoderHook>>,
    /// B-19A cycle 2 (WARN-2): optional ruff PostToolUse runner. When set
    /// AND `dispatch_post_tool_use` is invoked with a `.py`/`.pyi` file,
    /// the dispatcher invokes the runner and merges its findings into the
    /// returned `Vec<HookFinding>`. None preserves the previous "tiered
    /// only" behaviour for callers that don't yet wire a runner.
    ruff: Option<Arc<RuffPostToolUseRunner>>,
    /// B-19A cycle 2 (BLOCK-3): injected monotonic clock used by analyze().
    /// `OsClock` in production; `SeededClock` in tests. Replaces the
    /// previous direct `tokio::time::Instant::now()` calls so all timing
    /// flows through the entropy quarantine.
    clock: Arc<dyn ClockProvider>,
    /// B-19: optional handoff-save callback fired from `dispatch_pre_compact`
    /// when the token-budget gate passes (token_used / token_limit > 0.85).
    /// `None` preserves the legacy MVP-no-op behaviour for callers (e.g.
    /// the task_runner per-task firing site) that have no SessionDag handle.
    handoff_saver: Option<Arc<dyn PreCompactHandoffSaver>>,
    disabled: bool,
    /// F-1 VibeGate: optional SSB writer + agent metadata used to emit
    /// [`crate::ssb::SsbPayload::VibeGateBlocked`] rows from the async
    /// `dispatch_pre_tool_use` path. `None` = VibeGate findings still block
    /// (fail-closed) but no dedicated SSB row is emitted (they still travel
    /// as `HookEvent` rows via `emit_hook_events_to_ssb` at the task_runner
    /// call site).
    ///
    /// Tuple: `(writer, agent_id, superstep)`.
    vibegate_ssb: Option<(Arc<crate::ssb::Ssb>, String, i64)>,
    /// F-1 VibeGate: seed integrity flag. Set to `false` when
    /// [`crate::vibegate::check_vibegate_seed_integrity`] returns `false`.
    /// When `false`, every `dispatch_pre_tool_use` call emits a fail-closed
    /// Block finding for the code-vuln axis so the gate cannot be bypassed
    /// by a tampered seed.
    vibegate_seed_ok: bool,
    /// F-9 / Slice D3: optional write-time anti-slopsquatting gate.
    /// When `Some`, `dispatch_pre_tool_use` calls `gate.analyze(source,
    /// file_path)` and extends findings with any Block/Warn verdicts.
    /// `None` preserves pre-F-9 behaviour (gate absent, no registry probe).
    dep_writetime_gate: Option<Arc<DepWriteTimeGate>>,
    /// H-38: optional interactive gate for the stop cost-confirm prompt.
    /// When `Some` AND `evaluate_stop` returns `ConfirmRequired`, the
    /// dispatcher calls `gate.present_gate(...)` and surfaces cost-so-far.
    /// When `None` (headless / test contexts without a gate), the default is
    /// HOLD (fail-closed: no human to answer = do not silently destroy spend).
    stop_confirm_gate: Option<Arc<dyn InteractiveGate>>,
    /// H-38: optional `(SSB writer, db_path)` for reading the H-9 CloudSpend
    /// ledger via `SsbViewer::cloud_spend_summary` and for emitting the
    /// `StopConfirmationEvent` row. `None` = spend assumed $0.00 (degrade
    /// gracefully — a missing ledger is not an error, just means no cloud
    /// spend recorded yet). Pattern mirrors `vibegate_ssb` at :520.
    stop_cost_ssb: Option<(Arc<Ssb>, String)>,
    /// I-1: optional directive supervisor. When `Some`, the E-30 argv gate AND
    /// the F-1 VibeGate inside `dispatch_pre_tool_use` are DELEGATED to
    /// `Supervisor::run_gates` — the inline calls are REPLACED, not stacked, so
    /// exactly ONE gate path fires per invocation (the double-gate hazard the
    /// i1 contract pins). The G-3 blast-radius gates run inside the supervisor
    /// too (an addition; they were never inline). `None` preserves the legacy
    /// inline E-30 + F-1 path verbatim (every existing caller that has not
    /// installed a supervisor sees zero behavioral change). The F-9
    /// dep_writetime_gate is NOT in the supervisor scope and always runs inline.
    supervisor: Option<Arc<dyn crate::directive::Supervisor>>,
}

impl InProcessHookDispatcher {
    /// Construct with an optional tiered-analysis backend. When `None` (e.g.
    /// the plugins crate wasn't seeded with rules), every dispatch returns an
    /// empty findings list. This is the "wiring present but no-op" surface
    /// referenced in the Batch 1A spec.
    ///
    /// Production timing uses [`OsClock`]; for deterministic tests, build
    /// via [`Self::with_clock`] passing a [`elai_core::clock::SeededClock`].
    #[must_use]
    pub fn new(tiered: Option<Arc<TieredAnalysis>>) -> Self {
        let disabled = std::env::var(ELAI_HOOKS_DISABLED_ENV)
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
            .unwrap_or(false);
        if disabled {
            warn!(
                env = ELAI_HOOKS_DISABLED_ENV,
                "hook dispatcher disabled via env — baby steps run without T1/T2/T3",
            );
        }
        Self {
            tiered,
            shortcoder: None,
            ruff: None,
            clock: Arc::new(OsClock),
            handoff_saver: None,
            disabled,
            vibegate_ssb: None,
            vibegate_seed_ok: true,
            dep_writetime_gate: None,
            stop_confirm_gate: None,
            stop_cost_ssb: None,
            supervisor: None,
        }
    }

    /// B-19: install a [`PreCompactHandoffSaver`] callback. Without this,
    /// the PreCompact handler runs only T1 analysis (legacy MVP-no-op for
    /// the handoff side). With it, the dispatcher fires
    /// `saver.save_handoff(step_id)` whenever the token-budget gate
    /// (`token_used / token_limit > 0.85`) is crossed.
    #[must_use]
    pub fn with_handoff_saver(mut self, saver: Arc<dyn PreCompactHandoffSaver>) -> Self {
        self.handoff_saver = Some(saver);
        self
    }

    /// Returns true when a handoff saver is installed.
    #[must_use]
    pub fn has_handoff_saver(&self) -> bool {
        self.handoff_saver.is_some()
    }

    /// F-1 VibeGate: install the SSB writer used to emit
    /// [`crate::ssb::SsbPayload::VibeGateBlocked`] rows from
    /// `dispatch_pre_tool_use`. Without this, VibeGate findings still block
    /// (fail-closed via the HookEvent path) but no dedicated VibeGateBlocked
    /// SSB row is emitted.
    ///
    /// `agent_id` and `superstep` are written verbatim onto every row;
    /// pass `crate::task_runner::RUNNER_AGENT_ID` / `RUNNER_SUPERSTEP` from
    /// the task_runner setup path.
    #[must_use]
    pub fn with_vibegate_ssb(
        mut self,
        writer: Arc<crate::ssb::Ssb>,
        agent_id: impl Into<String>,
        superstep: i64,
    ) -> Self {
        self.vibegate_ssb = Some((writer, agent_id.into(), superstep));
        self
    }

    /// F-1 VibeGate: run the seed integrity check once and record the result.
    ///
    /// If the check fails, all subsequent `dispatch_pre_tool_use` calls block
    /// fail-closed on the code-vuln axis (even if the source is clean). Logs
    /// via `tracing::error` on failure. Safe to call multiple times; only the
    /// last result is retained.
    pub fn check_vibegate_seed_integrity(&mut self, conn: &rusqlite::Connection) {
        self.vibegate_seed_ok = crate::vibegate::check_vibegate_seed_integrity(conn);
    }

    /// F-1 VibeGate: builder-style setter for a pre-computed seed-integrity
    /// result. Use this when the integrity check has already run against the
    /// rules.db at wiring time (e.g. `build_task_runner_wiring`) and the
    /// `rusqlite::Connection` is no longer in scope at dispatcher-construction.
    /// `false` makes every `dispatch_pre_tool_use` code-vuln check block
    /// fail-closed, identical to [`Self::check_vibegate_seed_integrity`].
    #[must_use]
    pub fn with_vibegate_seed_ok(mut self, seed_ok: bool) -> Self {
        self.vibegate_seed_ok = seed_ok;
        self
    }

    /// F-9 / Slice D3: install the write-time anti-slopsquatting gate.
    ///
    /// When installed, `dispatch_pre_tool_use` calls `gate.analyze(source,
    /// file_path)` and merges Block/Warn findings for unrecognised or
    /// registry-absent dependency names.  `None` (the default) preserves
    /// pre-F-9 behaviour.  DEFAULT-ON: production bootstrap always passes
    /// `Some`; test callers may omit this for offline runs.
    #[must_use]
    pub fn with_dep_writetime_gate(mut self, gate: Arc<DepWriteTimeGate>) -> Self {
        self.dep_writetime_gate = Some(gate);
        self
    }

    /// H-38: install the interactive stop-cost confirmation gate.
    ///
    /// When installed and `evaluate_stop` returns `ConfirmRequired`, the
    /// dispatcher calls `gate.present_gate(...)` before proceeding.
    /// `GateDecision::Reject` returns a `HookFinding::Block` holding the stop.
    /// Without this gate, a `ConfirmRequired` verdict defaults to HOLD
    /// (fail-closed: headless context must not silently destroy spend/work).
    /// Production callers pass the `CliInteractiveGate` installed at
    /// `run_subcommand.rs:120`; contract tests pass a `MockInteractiveGate`.
    #[must_use]
    pub fn with_stop_confirm_gate(mut self, gate: Arc<dyn InteractiveGate>) -> Self {
        self.stop_confirm_gate = Some(gate);
        self
    }

    /// H-38: install the SSB writer + db-path used to read the H-9 CloudSpend
    /// ledger and emit `StopConfirmationEvent` rows from `dispatch_stop`.
    ///
    /// `writer` is the live `Ssb` instance; `db_path` is the path to the
    /// SQLite file (same DB) opened read-only via `SsbViewer`.  Pattern mirrors
    /// `with_vibegate_ssb` at line ~593.  Without this, `dispatch_stop` treats
    /// `day_usd` as `0.0` (no-cloud-spend degrade) and emits no SSB beacon.
    #[must_use]
    pub fn with_stop_cost_ssb(
        mut self,
        writer: Arc<Ssb>,
        db_path: impl Into<String>,
    ) -> Self {
        self.stop_cost_ssb = Some((writer, db_path.into()));
        self
    }

    /// I-1: install the directive supervisor. When installed,
    /// `dispatch_pre_tool_use` DELEGATES the E-30 argv gate and the F-1
    /// VibeGate to `Supervisor::run_gates` (replacing the inline calls so
    /// exactly one gate path fires per op), and the supervisor additionally
    /// runs the G-3 blast-radius gates. Without it, the legacy inline E-30 +
    /// F-1 path runs unchanged. Production callers install via
    /// `build_production_hook_dispatcher`; contract tests pass a fixture
    /// `InProcessSupervisor`.
    #[must_use]
    pub fn with_supervisor(mut self, supervisor: Arc<dyn crate::directive::Supervisor>) -> Self {
        self.supervisor = Some(supervisor);
        self
    }

    /// Returns true when the I-1 directive supervisor is installed.
    #[must_use]
    pub fn has_supervisor(&self) -> bool {
        self.supervisor.is_some()
    }

    /// I-1: project a [`HookInvocation`] into a [`crate::directive::Directive`]
    /// for the supervisor gate chain. The hook invocation carries the same
    /// fields the inline E-30/F-1 gates read (action_type as tool_name, source
    /// + file_path as args); the directive is the pure value the supervisor
    /// gate-checks. `intent_id` reuses the invocation's step_id for trace
    /// correlation; `initiator` is the canonical runner agent id.
    fn directive_from_invocation(invocation: &HookInvocation<'_>) -> crate::directive::Directive {
        let mut args = serde_json::Map::new();
        if let Some(source) = invocation.source {
            // The bash gate reads args.command; the fs/vibe gates read
            // args.source / args.path. Populate all three from the single
            // source string so each gate finds its input (the inline path read
            // the same `source` for both argv and vibe).
            args.insert("command".into(), serde_json::Value::String(source.to_owned()));
            args.insert("source".into(), serde_json::Value::String(source.to_owned()));
        }
        if let Some(file_path) = invocation.file_path {
            args.insert("path".into(), serde_json::Value::String(file_path.to_owned()));
        }
        crate::directive::Directive::new(
            invocation.action_type,
            serde_json::Value::Object(args),
            invocation.step_id.clone(),
            crate::task_runner::RUNNER_AGENT_ID,
        )
    }

    /// Batch 2A — construct with a ShortCoder PostToolUse hook installed.
    /// Equivalent to `Self::new(tiered).with_shortcoder(shortcoder)`.
    #[must_use]
    pub fn with_tiered_and_shortcoder(
        tiered: Option<Arc<TieredAnalysis>>,
        shortcoder: Option<Arc<ShortCoderHook>>,
    ) -> Self {
        let mut this = Self::new(tiered);
        this.shortcoder = shortcoder;
        this
    }

    /// Attach a [`ShortCoderHook`] after construction. Overwrites any
    /// previously installed hook; builder-style for tests.
    #[must_use]
    pub fn with_shortcoder(mut self, shortcoder: Arc<ShortCoderHook>) -> Self {
        self.shortcoder = Some(shortcoder);
        self
    }

    /// B-19A cycle 2 (BLOCK-3): inject a [`ClockProvider`] for deterministic
    /// timing in tests. Production callers should rely on the [`OsClock`]
    /// default installed by [`Self::new`] and never call this method.
    #[must_use]
    pub fn with_clock(mut self, clock: Arc<dyn ClockProvider>) -> Self {
        self.clock = clock;
        self
    }

    /// B-19A cycle 2 (WARN-2): install the ruff PostToolUse runner so the
    /// dispatcher actually invokes ruff on Python file writes. Without this
    /// installed, `dispatch_post_tool_use` only runs the tiered pass and
    /// the spec observable_effect ("agent runs Python file write →
    /// ruff_check_passed within 5s") stays unreachable.
    #[must_use]
    pub fn with_ruff_runner(mut self, ruff: Arc<RuffPostToolUseRunner>) -> Self {
        self.ruff = Some(ruff);
        self
    }

    /// Returns true when a ruff PostToolUse runner is installed.
    #[must_use]
    pub fn has_ruff_runner(&self) -> bool {
        self.ruff.is_some()
    }

    /// Returns true when the F-9 write-time dep-existence gate is installed.
    #[must_use]
    pub fn has_dep_writetime_gate(&self) -> bool {
        self.dep_writetime_gate.is_some()
    }

    /// Returns true if the env-var escape hatch is engaged.
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns true when at least one backing tier is installed.
    #[must_use]
    pub fn has_tiered(&self) -> bool {
        self.tiered.is_some()
    }

    /// Returns true when a ShortCoder hook is installed.
    #[must_use]
    pub fn has_shortcoder(&self) -> bool {
        self.shortcoder.is_some()
    }

    /// Run the configured ShortCoder (if any) on the PostToolUse payload.
    /// Returns `None` when the dispatcher is disabled globally, no hook is
    /// installed, or the input can't be compressed.
    #[must_use]
    pub fn shortcode(
        &self,
        source: &str,
        file_path: Option<&str>,
    ) -> Option<ShortCoderOutcome> {
        if self.disabled {
            return None;
        }
        self.shortcoder
            .as_ref()
            .map(|sc| sc.compress(source, file_path))
    }

    fn analyze(&self, invocation: &HookInvocation<'_>) -> Result<Vec<HookFinding>, HookError> {
        if self.disabled {
            return Ok(Vec::new());
        }
        let Some(tiered) = self.tiered.as_ref() else {
            return Ok(Vec::new());
        };
        // B-19A cycle 2 (BLOCK-3): time the tiered analysis pass via the
        // injected ClockProvider. Replaces the previous `tokio::time::Instant`
        // call so all production timing satisfies the entropy quarantine.
        let started = self.clock.now();
        let result = tiered.analyze(
            invocation.action_type,
            invocation.source,
            invocation.file_path,
            invocation.metric_values.clone(),
        )?;
        let composite_elapsed_ms =
            u64::try_from(self.clock.elapsed_since(started).as_millis()).unwrap_or(u64::MAX);
        trace!(
            action = invocation.action_type,
            duration_ms = result.duration_ms,
            composite_elapsed_ms,
            "tiered analysis complete"
        );

        // B-19A cycle 2 (BLOCK-2): partial fix — TieredAnalysis::analyze
        // returns ONE composite `duration_ms` covering T1+T2+T3, not per-tier
        // timings. Splitting the timing requires a cross-crate refactor of
        // `elai_plugins::TieredAnalysis` to surface (t1_ms, t2_ms, t3_ms),
        // which is out of scope for this cycle (would touch every dependent
        // crate's contract tests). We document the limitation here and emit
        // ONE composite-budget warning instead of three falsely-labelled
        // per-tier warnings. The previous code mislabeled the composite
        // timing as "T1 exceeded latency budget" — that misleading log is
        // removed. Per-tier T2/T3 enforcement is deferred to B-19B once
        // TieredAnalysis grows per-tier timings.
        //
        // Composite budget = HOOK_T2_BUDGET_MS (10ms) — matches the
        // existing `hook_tier_budget_ms(HookTier::Composite)` mapping so
        // the SSB-side enforcement helper and this dispatch path agree.
        if composite_elapsed_ms > HOOK_T2_BUDGET_MS {
            warn!(
                action = invocation.action_type,
                tier = HookTier::Composite.as_str(),
                budget_ms = HOOK_T2_BUDGET_MS,
                actual_ms = composite_elapsed_ms,
                "hook_dispatcher: composite tier exceeded latency budget \
                 (per-tier enforcement deferred to B-19B; \
                 TieredAnalysis returns composite timing only)"
            );
        }

        let mut findings: Vec<HookFinding> = Vec::new();
        push_finding(&mut findings, HookTier::T1, &result.t1_verdict);
        if let Some(v) = result.t2_verdict.as_ref() {
            push_finding(&mut findings, HookTier::T2, v);
        }
        if let Some(v) = result.t3_verdict.as_ref() {
            push_finding(&mut findings, HookTier::T3, v);
        }
        push_finding(&mut findings, HookTier::Composite, &result.final_verdict);
        Ok(findings)
    }

    /// B-19A cycle 2 (WARN-2): run the installed ruff PostToolUse runner on a
    /// Python file write and convert its outcome into `HookFinding` rows.
    /// Returns an empty vec when no runner is installed, the file is not
    /// Python, or the runner returns a non-Findings outcome.
    ///
    /// The runner internally enforces a 5-second timeout via tokio (BLOCK-3
    /// notes: this is acceptable because `tokio::time::timeout` is the
    /// runtime's scheduling primitive, not entropy — `now_ms` is what we
    /// quarantine, not the timer wheel). Findings are mapped to the
    /// `Composite` tier with `Warn` verdict by default; ruff exit code 0
    /// with empty findings produces a single `Allow` Composite breadcrumb.
    async fn run_ruff_post_tool_use(
        &self,
        invocation: &HookInvocation<'_>,
    ) -> Vec<HookFinding> {
        let Some(runner) = self.ruff.as_ref() else {
            return Vec::new();
        };
        let Some(path) = invocation.file_path else {
            return Vec::new();
        };
        let Some(source) = invocation.source else {
            return Vec::new();
        };
        let lower = path.to_ascii_lowercase();
        if !(lower.ends_with(".py") || lower.ends_with(".pyi")) {
            return Vec::new();
        }
        match runner.run(path, source).await {
            Ok(outcome) => ruff_outcome_to_findings(&outcome),
            Err(e) => {
                warn!(
                    error = %e,
                    file = path,
                    "hook_dispatcher: ruff runner failed (treated as no-finding)"
                );
                Vec::new()
            }
        }
    }

    /// H-38: helper to emit a `StopConfirmationEvent` SSB row.
    ///
    /// Best-effort; logs warn on failure but never propagates the error to
    /// the caller (non-fatal, consistent with the vibegate_ssb pattern).
    async fn emit_stop_confirmation_ssb(
        &self,
        payload: StopConfirmationPayload,
        step_id: &str,
    ) {
        if let Some((ref writer, _)) = self.stop_cost_ssb {
            if let Err(e) = writer
                .write_payload(
                    step_id,
                    0,
                    SsbPayload::StopConfirmationEvent(payload),
                    Vec::new(),
                )
                .await
            {
                warn!(
                    error = %e,
                    step_id,
                    "stop_cost_gate: SSB write failed (non-fatal)"
                );
            }
        }
    }
}

/// Convert a `RuffOutcome` into zero or more `HookFinding` rows the dispatcher
/// can return alongside the tiered-analysis findings. Mapping rules:
///
/// * `NoOp` → empty (we don't surface "skipped" runs as findings; the
///   tracing breadcrumb in the runner is enough).
/// * `Timeout` → one `Composite Warn` finding with category `ruff_timeout`.
/// * `Findings { exit_code: 0, findings: [] }` → empty (clean run).
/// * `Findings { findings: [..] }` → one `Composite Warn` per ruff finding.
fn ruff_outcome_to_findings(outcome: &RuffOutcome) -> Vec<HookFinding> {
    match outcome {
        RuffOutcome::NoOp { reason } => {
            // Surface BinaryMissing as a single Warn so the dashboard can
            // alert the user to install ruff. Other reasons (NonPython,
            // LineLimit, EmptyPath) are silent — they're expected skips.
            if matches!(reason, RuffSkipReason::BinaryMissing) {
                vec![HookFinding {
                    tier: HookTier::Composite,
                    verdict: HookVerdictKind::Warn,
                    rule_id: Some("ruff_binary_missing".to_string()),
                    category: "ruff_unavailable".to_string(),
                    message: "ruff binary not found on PATH — Python lint disabled".to_string(),
                    fix_template: Some("pip install ruff".to_string()),
                }]
            } else {
                Vec::new()
            }
        }
        RuffOutcome::Timeout { elapsed_ms } => vec![HookFinding {
            tier: HookTier::Composite,
            verdict: HookVerdictKind::Warn,
            rule_id: Some("ruff_timeout".to_string()),
            category: "ruff_timeout".to_string(),
            message: format!("ruff invocation timed out after {elapsed_ms}ms"),
            fix_template: None,
        }],
        RuffOutcome::Findings { findings, .. } => {
            findings.iter().map(ruff_finding_to_hook_finding).collect()
        }
    }
}

/// Map a single ruff diagnostic into a `HookFinding`. Severity tier is
/// `Composite` because ruff runs after T1/T2/T3; verdict is always `Warn`
/// (ruff is advisory, not a policy gate).
fn ruff_finding_to_hook_finding(f: &RuffFinding) -> HookFinding {
    let category = if f.rule_code.is_empty() {
        "ruff".to_string()
    } else {
        format!("ruff_{}", f.rule_code.to_ascii_lowercase())
    };
    HookFinding {
        tier: HookTier::Composite,
        verdict: HookVerdictKind::Warn,
        rule_id: if f.rule_code.is_empty() {
            None
        } else {
            Some(f.rule_code.clone())
        },
        category,
        message: format!(
            "{}:{}:{}: {} {}",
            f.file_path, f.line, f.column, f.rule_code, f.message
        ),
        fix_template: None,
    }
}

fn push_finding(out: &mut Vec<HookFinding>, tier: HookTier, verdict: &HookVerdict) {
    let kind = HookVerdictKind::from_verdict(verdict);
    // Emit for Warn/Block always; Allow only at the Composite tier so the
    // dashboard has a "hook ran, all clear" breadcrumb. Individual-tier Allow
    // verdicts would flood the feed with noise.
    if matches!(kind, HookVerdictKind::Allow) && !matches!(tier, HookTier::Composite) {
        return;
    }
    let (rule_id, category, detailed_message) = match verdict {
        HookVerdict::Allow => (None, HOOK_EVENT_CATEGORY.to_string(), "allow".to_string()),
        HookVerdict::Warn {
            message,
            pattern_hash,
        } => (
            Some(pattern_hash.clone()),
            extract_category(message).unwrap_or_else(|| HOOK_EVENT_CATEGORY.to_string()),
            message.clone(),
        ),
        HookVerdict::Block {
            message,
            pattern_hash,
        } => (
            Some(pattern_hash.clone()),
            extract_category(message).unwrap_or_else(|| HOOK_EVENT_CATEGORY.to_string()),
            message.clone(),
        ),
        // Escalate sits between Warn and Block (see HookVerdictKind doc). It
        // surfaces a finding like Warn/Block; the escalation `target` is folded
        // into the message so the routing destination survives onto the
        // HookFinding. Unlike Block it is NOT sanitized below — an escalation
        // must carry its reason to the auditor/human it routes to.
        HookVerdict::Escalate {
            target,
            message,
            pattern_hash,
        } => (
            Some(pattern_hash.clone()),
            extract_category(message).unwrap_or_else(|| HOOK_EVENT_CATEGORY.to_string()),
            format!("escalate -> {target:?}: {message}"),
        ),
    };
    // Batch 2A: Block messages at the IPC surface are sanitized so the
    // agent never sees rule_ids or internal details. The detailed_message
    // travels to the SSB via emit_hook_events_to_ssb (which reads the
    // original verdict payload via the TieredAnalysisPayload row).
    let message = match kind {
        HookVerdictKind::Block => sanitized_block_message(&category),
        _ => detailed_message,
    };
    out.push(HookFinding {
        tier,
        verdict: kind,
        rule_id,
        category,
        message,
        fix_template: None,
    });
}

/// Extract a category tag from a T3 advisory message of the form
/// `"[category_name] ...details..."`. Returns `None` for messages that
/// don't start with a bracketed tag (T1/T2 verdicts, Allow breadcrumbs).
fn extract_category(message: &str) -> Option<String> {
    let rest = message.strip_prefix('[')?;
    let close = rest.find(']')?;
    let tag = &rest[..close];
    // Guard against ridiculous or empty tags. A sensible category is
    // lowercase ASCII + underscores, bounded to 64 chars.
    if tag.is_empty()
        || tag.len() > 64
        || !tag
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b == b'_' || b.is_ascii_digit())
    {
        return None;
    }
    Some(tag.to_string())
}

impl HookDispatcher for InProcessHookDispatcher {
    fn dispatch_pre_tool_use<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        Box::pin(async move {
            // I-1 directive supervisor delegation. When a supervisor is
            // installed, the E-30 argv gate + the F-1 VibeGate are evaluated
            // ONCE by `Supervisor::run_gates` (which also adds the G-3
            // blast-radius gates). The inline E-30 block below and the inline
            // `evaluate_vibegate` call further down are SKIPPED in that case so
            // exactly one gate path fires per op (the double-gate hazard). The
            // supervisor's findings are routed through the SAME seed-integrity
            // override + VibeGateBlocked SSB emission bookkeeping as the inline
            // path, so observable behavior is identical save for the single
            // evaluation. `None` preserves the legacy inline path verbatim.
            let supervised_gate = match self.supervisor.as_ref() {
                Some(sup) => Some(sup.run_gates(&Self::directive_from_invocation(&invocation))),
                None => None,
            };

            // E-30 inline path: only when NOT delegating to a supervisor.
            if supervised_gate.is_none() && invocation.action_type == BASH_TOOL_USE_ACTION {
                if let Some(source) = invocation.source {
                    let argv: Vec<OsString> = source
                        .split_whitespace()
                        .map(OsString::from)
                        .collect();
                    if !argv.is_empty() {
                        use elai_safety::{check_argv, AllowlistVerdict};
                        match check_argv(&argv) {
                            AllowlistVerdict::Deny { reason, citation } => {
                                let argv_hash = elai_safety::argv_blake3(&argv);
                                trace!(
                                    argv_hash = %argv_hash,
                                    reason = %reason,
                                    citation = %citation,
                                    "E-30: argv_destructive_denied"
                                );
                                return Ok(vec![HookFinding {
                                    tier: HookTier::T1,
                                    verdict: HookVerdictKind::Block,
                                    rule_id: Some("e30-argv-deny".into()),
                                    category: HOOK_EVENT_CATEGORY.into(),
                                    message: format!(
                                        "{SANITIZED_BLOCK_MESSAGE}: {reason}"
                                    ),
                                    fix_template: Some(citation.to_owned()),
                                }]);
                            }
                            AllowlistVerdict::RequireHumanApproval { reason, citation } => {
                                let argv_hash = elai_safety::argv_blake3(&argv);
                                trace!(
                                    argv_hash = %argv_hash,
                                    reason = %reason,
                                    citation = %citation,
                                    "E-30: argv_approval_required"
                                );
                                // Approval-required: emit WARN finding (block = caller decision).
                                let mut findings = self.analyze(&invocation)?;
                                findings.push(HookFinding {
                                    tier: HookTier::T1,
                                    verdict: HookVerdictKind::Warn,
                                    rule_id: Some("e30-argv-approval".into()),
                                    category: HOOK_EVENT_CATEGORY.into(),
                                    message: format!(
                                        "human approval required: {reason}"
                                    ),
                                    fix_template: Some(citation.to_owned()),
                                });
                                return Ok(findings);
                            }
                            AllowlistVerdict::Allow => {}
                        }
                    }
                }
            }

            // I-1 supervised E-30 + G-3 short-circuit. When delegating, a Block
            // in the supervisor gate output (E-30 Deny or G-3 boundary cross)
            // early-returns exactly as the inline E-30 Deny did — the side
            // effect is refused fail-closed before the tiered pass.
            if let Some(gate) = supervised_gate.as_ref() {
                if gate.blocked {
                    return Ok(gate.findings.clone());
                }
            }

            // F-1 VibeGate: compound security evaluator.
            // Runs AFTER the E-30 argv check (which early-returns on Deny).
            // evaluate_vibegate is infallible; findings are merged into the
            // tiered-analysis findings and returned together so the existing
            // task_runner `findings.iter().any(HookFinding::is_block)` check
            // blocks fail-closed without any additional change at that site.
            let mut findings = self.analyze(&invocation)?;

            // F-9 / Slice D3: write-time anti-slopsquatting gate.
            // Runs synchronously (analyze() is not async) after the tiered
            // pass and before VibeGate so all Block findings are collected
            // before the early-return check below.  Fail-open per W-10:
            // Unreachable verdicts emit Warn, not Block.
            if !self.disabled {
                if let Some(gate) = self.dep_writetime_gate.as_ref() {
                    let source = invocation.source.unwrap_or("");
                    let file_path = invocation.file_path.unwrap_or("");
                    let dep_findings = gate.analyze(source, file_path);
                    findings.extend(dep_findings);
                }
            }

            if !self.disabled {
                let source = invocation.source.unwrap_or("");
                let file_path = invocation.file_path.unwrap_or("");
                let tool_name = invocation.action_type;
                // I-1: when delegating, the supervisor already evaluated F-1
                // (and E-30 + G-3) in `run_gates`; reuse those findings so
                // `evaluate_vibegate` is NOT called a second time (single gate
                // path). The early-return above already consumed any Block, so
                // the remaining supervised findings are the Warn/info gate
                // output that feeds the same seed-override + SSB bookkeeping as
                // the inline `evaluate_vibegate` result. `None` = inline path.
                let vibe_findings = match supervised_gate.as_ref() {
                    Some(gate) => gate.findings.clone(),
                    None => crate::vibegate::evaluate_vibegate(tool_name, source, file_path),
                };

                // Seed-integrity fail-closed override: if the seed check ran and
                // failed, inject a synthetic Block regardless of vibe findings.
                if !self.vibegate_seed_ok {
                    let mut with_seed_fail = vibe_findings;
                    with_seed_fail.push(HookFinding {
                        tier: HookTier::T1,
                        verdict: HookVerdictKind::Block,
                        rule_id: Some("vibesec-seed-integrity-fail".to_string()),
                        category: crate::vibegate::VIBESEC_CODE_VULN_CATEGORY.to_string(),
                        message:
                            "vibesec_code_vuln: seed integrity check failed; blocking fail-closed"
                                .to_string(),
                        fix_template: None,
                    });
                    // Emit SSB rows for the seed-fail block if writer is present.
                    if let Some((writer, agent_id, superstep)) = self.vibegate_ssb.as_ref() {
                        let fsm = if invocation.fsm_state.is_empty() {
                            None
                        } else {
                            Some(invocation.fsm_state.as_str())
                        };
                        crate::vibegate::emit_vibegate_blocked_to_ssb(
                            writer,
                            agent_id,
                            *superstep,
                            source,
                            file_path,
                            fsm,
                            &with_seed_fail,
                        )
                        .await;
                    }
                    findings.extend(with_seed_fail);
                    return Ok(findings);
                }

                if !vibe_findings.is_empty() {
                    // Emit dedicated VibeGateBlocked SSB rows before returning.
                    if let Some((writer, agent_id, superstep)) = self.vibegate_ssb.as_ref() {
                        let fsm = if invocation.fsm_state.is_empty() {
                            None
                        } else {
                            Some(invocation.fsm_state.as_str())
                        };
                        crate::vibegate::emit_vibegate_blocked_to_ssb(
                            writer,
                            agent_id,
                            *superstep,
                            source,
                            file_path,
                            fsm,
                            &vibe_findings,
                        )
                        .await;
                    }
                    findings.extend(vibe_findings);
                }
            }

            // I-35: push a HistoryEntry after each completed dispatch cycle so the
            // sequence gate sees a complete ordered record of prior tool invocations.
            // Outcome is "blocked" when any Block finding was produced; "allowed" otherwise.
            // This is the sole push site — the supervisor trait default is a no-op so
            // callers without a supervisor installed are unaffected.
            if let Some(sup) = self.supervisor.as_ref() {
                let outcome_str = if findings.iter().any(|f| f.is_block()) {
                    "blocked"
                } else {
                    "allowed"
                };
                let tool_class =
                    crate::tool_sequence::tool_class_from_action_type(invocation.action_type);
                let ts_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                let entry = crate::tool_sequence::HistoryEntry::new(
                    tool_class,
                    ts_ms,
                    outcome_str,
                );
                sup.push_history_entry(&entry);
            }

            Ok(findings)
        })
    }

    fn dispatch_post_tool_use<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        Box::pin(async move {
            // B-19A cycle 2 (WARN-2): run the tiered pass first so a BLOCK at
            // T1/T2/T3 still aborts. THEN, if the dispatcher has a ruff runner
            // installed AND the invocation targets a Python file write, run
            // ruff and append its findings. The runner itself short-circuits
            // for non-`.py`/`.pyi` paths, but we replicate the predicate here
            // so the dispatcher does not pay the runner-method tax (and the
            // global env-disabled flag) on every call.
            let mut findings = self.analyze(&invocation)?;
            if !self.disabled && self.ruff.is_some() {
                let ruff_findings = self.run_ruff_post_tool_use(&invocation).await;
                findings.extend(ruff_findings);
            }
            Ok(findings)
        })
    }

    /// Batch 4B: expose the inherent ShortCoder method via the trait so
    /// callers holding an `Arc<dyn HookDispatcher>` can invoke it without
    /// downcasting. Behaviour matches the inherent method — env-off returns
    /// None, no-hook returns None, compressed Outcome otherwise.
    fn shortcode(
        &self,
        source: &str,
        file_path: Option<&str>,
    ) -> Option<ShortCoderOutcome> {
        // Forward to the inherent method defined on `InProcessHookDispatcher`.
        InProcessHookDispatcher::shortcode(self, source, file_path)
    }

    /// I-1: expose the installed supervisor so the task_runner can call
    /// `Supervisor::execute_directive` at the production tool-dispatch seam.
    /// Mirrors the shape of `shortcode` above — the Arc is cloned so the
    /// caller gets an owned handle without breaking shared ownership.
    fn supervisor(&self) -> Option<Arc<dyn crate::directive::Supervisor>> {
        self.supervisor.clone()
    }

    fn dispatch_stop<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        Box::pin(async move {
            // B-19: enforce non-IDLE block. If fsm_state is set and is not
            // IDLE or CONFIGURING, short-circuit with a Block finding so the
            // task_runner can surface the rejection to the dashboard.
            // Empty string = FSM state not plumbed by caller; skip check.
            let state_upper = invocation.fsm_state.to_uppercase();
            if !state_upper.is_empty()
                && state_upper != "IDLE"
                && state_upper != "CONFIGURING"
            {
                return Ok(vec![HookFinding {
                    tier: HookTier::Composite,
                    verdict: HookVerdictKind::Block,
                    rule_id: Some("stop_non_idle".to_string()),
                    category: "stop_hook".to_string(),
                    message: format!(
                        "Cannot stop in non-IDLE state — current: {}",
                        invocation.fsm_state
                    ),
                    fix_template: None,
                }]);
            }

            // H-38: cost-confirmation gate.
            // Read day_usd from the H-9 CloudSpend ledger (degrade to 0.0 on
            // viewer-open failure per context package -- a missing ledger just
            // means no cloud spend was recorded, so the cost-floor predicate
            // passes silently, which is correct behavior).
            let now_ms = i64::try_from(self.clock.now_ms()).unwrap_or(i64::MAX);
            let day_usd = if let Some((_, ref db_path)) = self.stop_cost_ssb {
                SsbViewer::open(Path::new(db_path))
                    .and_then(|v| v.cloud_spend_summary(now_ms))
                    .map(|s| s.day_usd)
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            let snapshot = StopCostSnapshot {
                day_usd,
                unsaved_artifacts: invocation.unsaved_artifact_count,
                pod_billable: invocation.pod_billable,
            };
            let verdict = evaluate_stop(snapshot);

            let stop_cost_confirm_enable = std::env::var(ELAI_STOP_COST_CONFIRM_ENABLE)
                .map(|v| v == "1")
                .unwrap_or(false);

            let ts_ms = now_ms;

            let mut extra_findings = Vec::new();

            match &verdict {
                StopGateVerdict::Pass => {
                    // Zero friction: only emit the SSB beacon when telemetry is ON.
                    if stop_cost_confirm_enable {
                        info!(
                            rule = "stop_cost_confirm",
                            day_usd,
                            "stop_cost_gate: PassedNoFriction — stop proceeds without confirmation"
                        );
                        self.emit_stop_confirmation_ssb(
                            StopConfirmationPayload {
                                outcome: StopOutcomeKind::PassedNoFriction,
                                cost_usd: day_usd,
                                unsaved_count: u32::try_from(invocation.unsaved_artifact_count)
                                    .unwrap_or(u32::MAX),
                                ts_ms,
                            },
                            &invocation.step_id,
                        )
                        .await;
                    }
                }
                StopGateVerdict::ConfirmRequired { cost_usd, unsaved_count, reason } => {
                    info!(
                        rule = "stop_cost_confirm",
                        cost_usd,
                        unsaved_count,
                        reason = reason.as_str(),
                        "stop_cost_gate: ConfirmRequired — presenting gate or holding (fail-closed)"
                    );
                    // Determine gate decision: present if a gate is installed,
                    // otherwise default to HOLD (fail-closed for headless context).
                    let gate_decision = if let Some(ref gate) = self.stop_confirm_gate {
                        use crate::gate::{EvidenceSummary, RiskLevel};
                        // Build a minimal EvidenceSummary for the confirm prompt.
                        let summary = EvidenceSummary {
                            source: crate::fsm::FsmState::Idle,
                            target: crate::fsm::FsmState::Idle,
                            phase_label: format!(
                                "stop_cost_confirm: ${cost_usd:.4} spent today, {unsaved_count} unsaved artifact(s) — {reason}"
                            ),
                            gate_results: Vec::new(),
                            all_gates_passed: false,
                        };
                        gate.present_gate(&summary, RiskLevel::High)
                    } else {
                        // No gate installed: fail-closed (headless context).
                        crate::gate::GateDecision::Reject
                    };

                    match gate_decision {
                        crate::gate::GateDecision::Approve => {
                            // User confirmed: stop proceeds; emit beacon unconditionally.
                            self.emit_stop_confirmation_ssb(
                                StopConfirmationPayload {
                                    outcome: StopOutcomeKind::ConfirmedProceed,
                                    cost_usd: *cost_usd,
                                    unsaved_count: u32::try_from(*unsaved_count)
                                        .unwrap_or(u32::MAX),
                                    ts_ms,
                                },
                                &invocation.step_id,
                            )
                            .await;
                        }
                        _ => {
                            // Rejected or any other non-Approve decision: hold stop.
                            // Rule-id "stop_cost_confirm_required" is in the message text
                            // so tracing-test logs_contain("stop_cost_confirm_required")
                            // succeeds (T-1: fields alone are not captured as message text).
                            info!(
                                cost_usd,
                                unsaved_count,
                                "stop_cost_gate: stop_cost_confirm_required — RejectedHeld, stop blocked pending confirm"
                            );
                            // Emit beacon unconditionally on the blocked path.
                            self.emit_stop_confirmation_ssb(
                                StopConfirmationPayload {
                                    outcome: StopOutcomeKind::RejectedHeld,
                                    cost_usd: *cost_usd,
                                    unsaved_count: u32::try_from(*unsaved_count)
                                        .unwrap_or(u32::MAX),
                                    ts_ms,
                                },
                                &invocation.step_id,
                            )
                            .await;
                            extra_findings.push(HookFinding {
                                tier: HookTier::Composite,
                                verdict: HookVerdictKind::Block,
                                rule_id: Some("stop_cost_confirm_required".to_string()),
                                category: "stop_hook".to_string(),
                                message: format!(
                                    "stop blocked: ${cost_usd:.4} day-spend / {unsaved_count} unsaved — confirm required"
                                ),
                                fix_template: None,
                            });
                            return Ok(extra_findings);
                        }
                    }
                }
            }

            // B-19 T1 analysis pass (runs regardless of cost-gate result unless
            // we returned early above due to a RejectedHeld block).
            let mut findings = self.analyze(&invocation)?;
            findings.extend(extra_findings);
            Ok(findings)
        })
    }

    fn dispatch_pre_compact<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        Box::pin(async move {
            // B-19: PreCompact hook fires at context-budget handoff threshold.
            // DairaTrace "HookPre"/"HookPost" rows are emitted by the task_runner
            // caller (which holds Arc<Ssb>) around this dispatch call so the SSB
            // gets compact_start / compact_end breadcrumbs without the dispatcher
            // needing a direct SSB handle. The T1 analysis pass still runs so
            // any predicate rules that gate on "pre_compact" fire correctly.
            //
            // B-19 token-budget gate per spec phase-4C.txt:28-29
            // ("PreCompact fires when token_used/token_limit > 0.85"). If the
            // caller supplied a `(used, limit)` snapshot via
            // `HookInvocation::with_pre_compact_token_state`:
            //   * limit == 0 or used / limit <= PRE_COMPACT_HANDOFF_FRACTION
            //     -> NOOP (no analyze, no handoff). Prevents SSB churn from
            //     per-task unconditional firing.
            //   * used / limit > PRE_COMPACT_HANDOFF_FRACTION
            //     -> run analyze() AND invoke the handoff saver (if installed).
            // When the caller did NOT supply a snapshot (None), we preserve
            // pre-B-19 behaviour (run analyze; no handoff trigger) so the
            // task_runner per-task firing site keeps working without
            // threading token telemetry it does not have.
            if let Some((used, limit)) = invocation.pre_compact_token_state {
                if limit == 0 {
                    debug!(
                        step = %invocation.step_id,
                        used,
                        "pre_compact gated: limit==0, no-op"
                    );
                    return Ok(Vec::new());
                }
                let fraction = f64::from(used) / f64::from(limit);
                if fraction <= PRE_COMPACT_HANDOFF_FRACTION {
                    debug!(
                        step = %invocation.step_id,
                        used,
                        limit,
                        fraction,
                        "pre_compact gated: fraction <= 0.85, no-op"
                    );
                    return Ok(Vec::new());
                }
                // Above-threshold: invoke the handoff saver (if installed)
                // BEFORE running analyze() so the snapshot reflects the
                // pre-compact state. spawn_blocking because save_handoff
                // touches rusqlite (sync) and we must not block the async
                // runtime.
                if let Some(saver) = self.handoff_saver.as_ref() {
                    let saver_clone = Arc::clone(saver);
                    let step_id_owned = invocation.step_id.clone();
                    let join = tokio::task::spawn_blocking(move || {
                        saver_clone.save_handoff(&step_id_owned)
                    })
                    .await;
                    match join {
                        Ok(Ok(())) => {
                            debug!(
                                step = %invocation.step_id,
                                fraction,
                                "pre_compact handoff saved"
                            );
                        }
                        Ok(Err(e)) => {
                            warn!(
                                step = %invocation.step_id,
                                error = %e,
                                "pre_compact handoff_saver returned error (non-blocking)"
                            );
                        }
                        Err(e) => {
                            warn!(
                                step = %invocation.step_id,
                                error = %e,
                                "pre_compact handoff_saver join failed (non-blocking)"
                            );
                        }
                    }
                }
            }
            debug!(step = %invocation.step_id, "pre_compact analyze pass");
            self.analyze(&invocation)
        })
    }
}

// ---------------------------------------------------------------------------
// SSB emission helper
// ---------------------------------------------------------------------------

/// Write one [`SsbPayload::HookEvent`] row per finding to the supplied SSB.
/// Returns the number of rows written.
///
/// Kept as a free function (not a dispatcher method) so in-memory tests can
/// exercise the dispatcher trait without a live `Ssb` writer.
pub async fn emit_hook_events_to_ssb(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    hook_point: HookPoint,
    step_id: &str,
    findings: &[HookFinding],
) -> usize {
    let mut count = 0usize;
    for finding in findings {
        let payload = SsbPayload::HookEvent(HookEventPayload {
            tier: finding.tier.as_str().to_string(),
            verdict: finding.verdict.as_str().to_string(),
            rule_id: finding.rule_id.clone(),
            category: finding.category.clone(),
            message: finding.message.clone(),
            step_id: step_id.to_string(),
            hook_point: hook_point.as_str().to_string(),
        });
        match writer
            .write_payload(agent_id, superstep, payload, Vec::new())
            .await
        {
            Ok(_) => count += 1,
            Err(e) => {
                warn!(
                    error = %e,
                    hook = hook_point.as_str(),
                    "hook_dispatcher: failed to emit HookEvent row"
                );
            }
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Batch 2A — per-invocation summary + ShortCoder SSB emission helpers
// ---------------------------------------------------------------------------

/// Write a per-invocation [`TieredAnalysisPayload`] row summarising one
/// tier's dispatch. Complementary to `emit_hook_events_to_ssb` which
/// emits one row per verdict; this helper emits ONE row per tier per
/// invocation with timing + counts the dashboard can trend.
///
/// Returns `true` when the write succeeded, `false` on any SSB error
/// (logged via tracing::warn at the call site — we never panic).
pub async fn emit_tiered_analysis_summary_to_ssb(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    tier: HookTier,
    verdict: HookVerdictKind,
    rule_count_fired: u32,
    elapsed_us: u64,
    step_id: &str,
    category: &str,
) -> bool {
    let payload = SsbPayload::TieredAnalysis(TieredAnalysisPayload {
        tier: tier.as_str().to_string(),
        verdict: verdict.as_str().to_string(),
        rule_count_fired,
        elapsed_us,
        step_id: step_id.to_string(),
        category: category.to_string(),
    });
    match writer
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        Ok(_) => true,
        Err(e) => {
            warn!(
                error = %e,
                tier = tier.as_str(),
                "hook_dispatcher: failed to emit TieredAnalysis summary"
            );
            false
        }
    }
}

/// Write a [`ShortCoderCompressionPayload`] row for a single PostToolUse
/// compression. Safe to call with `ShortCoderOutcome::Skipped` — in that
/// case the row records zeros so the dashboard can see the hook fired
/// and chose to skip (reason lands in a future payload field extension).
pub async fn emit_shortcoder_compression_to_ssb(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    outcome: &ShortCoderOutcome,
    step_id: &str,
) -> bool {
    let payload = SsbPayload::ShortCoderCompression(ShortCoderCompressionPayload {
        tokens_in: outcome.tokens_in(),
        tokens_out: outcome.tokens_out(),
        reduction_pct: outcome.reduction_pct(),
        step_id: step_id.to_string(),
        rules_applied: outcome.rules_applied_labels(),
    });
    match writer
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        Ok(_) => true,
        Err(e) => {
            warn!(
                error = %e,
                "hook_dispatcher: failed to emit ShortCoderCompression row"
            );
            false
        }
    }
}

// ---------------------------------------------------------------------------
// B-19A (T10) — hook latency budget enforcement helpers
// ---------------------------------------------------------------------------

/// Returns the per-tier latency budget in milliseconds for the given tier.
/// `Composite` aggregates across T1/T2/T3 — we use the largest individual
/// budget (T2 at 10ms) so the composite tier alarms only when a tier
/// overruns its own budget AND drags the whole dispatch with it.
#[must_use]
pub fn hook_tier_budget_ms(tier: HookTier) -> u64 {
    match tier {
        HookTier::T1 => HOOK_T1_BUDGET_MS,
        HookTier::T2 => HOOK_T2_BUDGET_MS,
        HookTier::T3 => HOOK_T3_BUDGET_MS,
        HookTier::Composite => HOOK_T2_BUDGET_MS,
    }
}

/// Write a single [`SsbPayload::HookLatencyViolation`] row recording that
/// `hook_name` at `tier` ran for `actual_ms` exceeding `budget_ms`. Returns
/// `true` on success, `false` on any SSB error (logged via tracing::warn).
///
/// Kept as a free function (not a dispatcher method) so callers without a
/// concrete dispatcher reference (e.g. external instrumented runners) can
/// use it directly.
pub async fn emit_hook_latency_violation_to_ssb(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    hook_name: &str,
    tier: HookTier,
    budget_ms: u64,
    actual_ms: u64,
) -> bool {
    let payload = SsbPayload::HookLatencyViolation(HookLatencyViolationPayload {
        hook_name: hook_name.to_string(),
        tier: tier.as_str().to_string(),
        budget_ms,
        actual_ms,
    });
    match writer
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        Ok(_) => true,
        Err(e) => {
            warn!(
                error = %e,
                hook = hook_name,
                tier = tier.as_str(),
                "hook_dispatcher: failed to emit HookLatencyViolation row"
            );
            false
        }
    }
}

/// Compare `actual_ms` against the spec budget for `tier`. If the budget is
/// exceeded:
///   * Emits a `tracing::warn` log line with hook + tier + actual + budget.
///   * Writes one `hook_latency_violation` SSB row (if `writer` is `Some`).
/// Returns `true` when a violation was recorded; `false` otherwise.
///
/// This is the single place the dispatcher should call after measuring a
/// tier's elapsed time — both the log and the SSB row stay in lockstep.
pub async fn report_hook_latency(
    writer: Option<&Arc<Ssb>>,
    agent_id: &str,
    superstep: i64,
    hook_name: &str,
    tier: HookTier,
    actual_ms: u64,
) -> bool {
    let budget_ms = hook_tier_budget_ms(tier);
    if actual_ms <= budget_ms {
        return false;
    }
    warn!(
        hook = hook_name,
        tier = tier.as_str(),
        budget_ms,
        actual_ms,
        "hook_dispatcher: tier exceeded latency budget"
    );
    if let Some(w) = writer {
        emit_hook_latency_violation_to_ssb(
            w, agent_id, superstep, hook_name, tier, budget_ms, actual_ms,
        )
        .await;
    }
    true
}

// ---------------------------------------------------------------------------
// B-19 — MarkerReport aggregation emit helper
// ---------------------------------------------------------------------------

/// Write a single [`crate::ssb::SsbPayload::MarkerReport`] row aggregating
/// the per-phase DAIRA marker counts for one agent-execution window.
///
/// `phase_counts` keys MUST come from
/// [`crate::ssb::DAIRA_MARKERS`]. Unknown keys are logged via
/// `tracing::warn` and silently dropped before write (release semantics);
/// `debug_assert!` fires in debug builds so test runs surface the bug.
///
/// Caller responsibility: build the `BTreeMap<String, u32>` aggregating
/// `DairaTracePayload::phase` field counts for the window; the dispatcher
/// itself does not maintain per-step counters (those would force shared
/// mutable state across async tasks and create a contention hot-spot on
/// the baby-step loop).
///
/// Returns `true` on successful SSB write, `false` on any SSB / encoding
/// failure (logged via `tracing::warn`).
pub async fn emit_marker_report_to_ssb(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    step_id: &str,
    phase_counts: std::collections::BTreeMap<String, u32>,
    window_us: u64,
) -> bool {
    // Validate keys against DAIRA_MARKERS — strict in debug, lenient in release.
    let mut validated: std::collections::BTreeMap<String, u32> =
        std::collections::BTreeMap::new();
    let mut total_events: u32 = 0;
    for (phase, count) in phase_counts.into_iter() {
        let known = crate::ssb::DAIRA_MARKERS.iter().any(|m| *m == phase.as_str());
        debug_assert!(
            known,
            "emit_marker_report_to_ssb: unknown DAIRA phase '{}' \
             (must be drawn from crate::ssb::DAIRA_MARKERS)",
            phase
        );
        if !known {
            warn!(
                phase = %phase,
                "emit_marker_report_to_ssb: dropping unknown DAIRA phase"
            );
            continue;
        }
        total_events = total_events.saturating_add(count);
        validated.insert(phase, count);
    }
    let payload = crate::ssb::SsbPayload::MarkerReport(crate::ssb::MarkerReportPayload {
        agent_id: agent_id.to_string(),
        step_id: step_id.to_string(),
        phase_counts: validated,
        total_events,
        window_us,
    });
    match writer
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        Ok(_) => true,
        Err(e) => {
            warn!(
                error = %e,
                step = step_id,
                "hook_dispatcher: failed to emit MarkerReport row"
            );
            false
        }
    }
}

// ---------------------------------------------------------------------------
// B-19 — async-bridge adapter from IpcHookTrait (sync) to HookDispatcher (async)
// ---------------------------------------------------------------------------

/// Async-bridge adapter wrapping a sync `IpcHookTrait` (from
/// `elai_plugins::ipc_hooks`) and exposing it as the async
/// `HookDispatcher` trait used by `task_runner`. Bridges the architectural
/// surface gap recorded in `workflow/sessions/s-25120/completion-drive/
/// exploration/B-19_hook_system_unification-exploration.md` GAP-1:
/// `IpcHookTrait` is sync (CPU-bound tiered analysis), `HookDispatcher`
/// is async (`Pin<Box<dyn Future>>`); a direct trait-for-trait impl is
/// not possible without one side conceding its execution model. The
/// adapter calls the sync methods via `tokio::task::spawn_blocking` so
/// the async runtime stays unblocked while the CPU work runs.
///
/// Behaviour mapping (one-to-one with the four hook points):
///   * `dispatch_pre_tool_use`  -> `IpcHookTrait::pre_tool_use`
///   * `dispatch_post_tool_use` -> `IpcHookTrait::post_tool_use`
///   * `dispatch_stop`          -> `IpcHookTrait::stop`
///   * `dispatch_pre_compact`   -> `IpcHookTrait::pre_compact`
///
/// Return value mapping: `IpcHookTrait` returns a single `HookResult`
/// with one verdict; `HookDispatcher` returns `Vec<HookFinding>`. The
/// adapter maps `HookResult::verdict` to ZERO findings (Allow), ONE
/// `Warn` finding (Warn), or ONE `Block` finding (Block) tagged with
/// the `Composite` tier — matching the existing `InProcessHookDispatcher`
/// convention of one finding per non-Allow verdict at the composite tier.
///
/// The dispatcher unification recorded in B-19 decisions lives at the
/// DISPATCHER level (`Arc<dyn HookDispatcher>`), NOT the trait level —
/// the two upstream traits serve different needs (IPC sync transport vs
/// in-process async invocation) and are preserved for their respective
/// callers. The adapter is the missing piece that lets either dispatcher
/// be plugged into the `task_runner` callsite.
pub struct IpcHookAsyncAdapter {
    inner: Arc<dyn elai_plugins::IpcHookTrait + Send + Sync + 'static>,
}

impl IpcHookAsyncAdapter {
    /// Construct an adapter wrapping a sync `IpcHookTrait` implementation.
    #[must_use]
    pub fn new(
        inner: Arc<dyn elai_plugins::IpcHookTrait + Send + Sync + 'static>,
    ) -> Self {
        Self { inner }
    }

    /// Build a `HookContext` from a `HookInvocation`. The adapter intentionally
    /// fills only the fields available on `HookInvocation`; `agent_id` /
    /// `session_id` / `agent_role` default to empty strings because the
    /// `task_runner` dispatch sites do not thread those through. The
    /// downstream `IpcHookDispatcher` already tolerates empty values
    /// (`ctx.tool_id.as_deref().unwrap_or("unknown")` at
    /// `plugins/src/ipc_hooks.rs:304`).
    fn build_context(invocation: &HookInvocation<'_>) -> elai_plugins::HookContext {
        elai_plugins::HookContext {
            agent_id: String::new(),
            agent_role: String::new(),
            session_id: String::new(),
            fsm_state: invocation.fsm_state.clone(),
            tool_id: Some(invocation.action_type.to_string()),
            tool_arguments: None,
            source_content: invocation.source.map(|s| s.to_string()),
            file_path: invocation.file_path.map(|p| p.to_string()),
            active_rules_count: 0,
        }
    }

    /// Map an `IpcHookTrait` `HookResult` to the dispatcher's
    /// `Vec<HookFinding>` shape. Mirrors the per-verdict mapping in
    /// `InProcessHookDispatcher::push_finding`.
    fn map_result(result: elai_plugins::HookResult) -> Vec<HookFinding> {
        use elai_plugins::tiered_analysis::HookVerdict as PluginVerdict;
        match result.verdict {
            PluginVerdict::Allow => Vec::new(),
            PluginVerdict::Warn { message, pattern_hash } => vec![HookFinding {
                tier: HookTier::Composite,
                verdict: HookVerdictKind::Warn,
                rule_id: Some(pattern_hash),
                category: HOOK_EVENT_CATEGORY.to_string(),
                message,
                fix_template: None,
            }],
            PluginVerdict::Block { message, pattern_hash } => vec![HookFinding {
                tier: HookTier::Composite,
                verdict: HookVerdictKind::Block,
                rule_id: Some(pattern_hash),
                category: HOOK_EVENT_CATEGORY.to_string(),
                message,
                fix_template: None,
            }],
            // Escalate surfaces a finding between Warn and Block (see the
            // HookVerdictKind severity-order doc). The routing `target` is
            // folded into the message so it is not dropped on this boundary,
            // mirroring InProcessHookDispatcher::push_finding.
            PluginVerdict::Escalate { target, message, pattern_hash } => vec![HookFinding {
                tier: HookTier::Composite,
                verdict: HookVerdictKind::Escalate,
                rule_id: Some(pattern_hash),
                category: HOOK_EVENT_CATEGORY.to_string(),
                message: format!("escalate -> {target:?}: {message}"),
                fix_template: None,
            }],
        }
    }
}

impl HookDispatcher for IpcHookAsyncAdapter {
    fn dispatch_pre_tool_use<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        let ctx = Self::build_context(&invocation);
        let inner = Arc::clone(&self.inner);
        Box::pin(async move {
            let join = tokio::task::spawn_blocking(move || inner.pre_tool_use(&ctx)).await;
            match join {
                Ok(Ok(r)) => Ok(Self::map_result(r)),
                Ok(Err(e)) => Err(HookError::Tiered(format!("ipc_async_adapter: {e}"))),
                Err(e) => Err(HookError::Tiered(format!("ipc_async_adapter join: {e}"))),
            }
        })
    }

    fn dispatch_post_tool_use<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        let ctx = Self::build_context(&invocation);
        let inner = Arc::clone(&self.inner);
        // `IpcHookTrait::post_tool_use` expects `tool_output: &str`; the
        // dispatcher passes `invocation.source` as the post-output payload
        // (matching the in-process semantic where source IS the file
        // contents written).
        let tool_output = invocation.source.unwrap_or("").to_string();
        Box::pin(async move {
            let join = tokio::task::spawn_blocking(move || {
                inner.post_tool_use(&ctx, &tool_output)
            })
            .await;
            match join {
                Ok(Ok(r)) => Ok(Self::map_result(r)),
                Ok(Err(e)) => Err(HookError::Tiered(format!("ipc_async_adapter: {e}"))),
                Err(e) => Err(HookError::Tiered(format!("ipc_async_adapter join: {e}"))),
            }
        })
    }

    fn dispatch_stop<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        let ctx = Self::build_context(&invocation);
        let inner = Arc::clone(&self.inner);
        Box::pin(async move {
            let join = tokio::task::spawn_blocking(move || inner.stop(&ctx)).await;
            match join {
                Ok(Ok(r)) => Ok(Self::map_result(r)),
                Ok(Err(e)) => Err(HookError::Tiered(format!("ipc_async_adapter: {e}"))),
                Err(e) => Err(HookError::Tiered(format!("ipc_async_adapter join: {e}"))),
            }
        })
    }

    fn dispatch_pre_compact<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        let ctx = Self::build_context(&invocation);
        let inner = Arc::clone(&self.inner);
        // B-19: honour the token-budget gate at the adapter boundary too,
        // mirroring InProcessHookDispatcher::dispatch_pre_compact so a
        // dispatcher swap is behaviour-preserving.
        let gate = invocation.pre_compact_token_state;
        Box::pin(async move {
            if let Some((used, limit)) = gate {
                if limit == 0 {
                    return Ok(Vec::new());
                }
                let fraction = f64::from(used) / f64::from(limit);
                if fraction <= PRE_COMPACT_HANDOFF_FRACTION {
                    return Ok(Vec::new());
                }
            }
            let join = tokio::task::spawn_blocking(move || inner.pre_compact(&ctx)).await;
            match join {
                Ok(Ok(r)) => Ok(Self::map_result(r)),
                Ok(Err(e)) => Err(HookError::Tiered(format!("ipc_async_adapter: {e}"))),
                Err(e) => Err(HookError::Tiered(format!("ipc_async_adapter join: {e}"))),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Tests (unit level — SSB-backed tests live in contracts/)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dispatcher_with_no_backend_returns_empty_findings() {
        let d = InProcessHookDispatcher::new(None);
        let inv = HookInvocation::for_pre_tool_use("baby_step_pre_gate", "noop", "step_1");
        let findings = d.dispatch_pre_tool_use(inv).await.expect("ok");
        assert!(findings.is_empty());
    }

    #[tokio::test]
    async fn hook_finding_is_block_detects_block() {
        let f = HookFinding {
            tier: HookTier::T1,
            verdict: HookVerdictKind::Block,
            rule_id: Some("r1".into()),
            category: HOOK_EVENT_CATEGORY.into(),
            message: "m".into(),
            fix_template: None,
        };
        assert!(f.is_block());
        assert!(!f.is_warn());
    }

    #[test]
    fn hook_point_strings_stable() {
        assert_eq!(HookPoint::PreToolUse.as_str(), "PreToolUse");
        assert_eq!(HookPoint::PostToolUse.as_str(), "PostToolUse");
        assert_eq!(HookPoint::Stop.as_str(), "Stop");
        assert_eq!(HookPoint::PreCompact.as_str(), "PreCompact");
    }

    #[test]
    fn hook_tier_strings_stable() {
        assert_eq!(HookTier::T1.as_str(), "T1");
        assert_eq!(HookTier::T2.as_str(), "T2");
        assert_eq!(HookTier::T3.as_str(), "T3");
        assert_eq!(HookTier::Composite.as_str(), "Composite");
    }

    #[test]
    fn verdict_kind_maps_from_plugin_verdict() {
        assert_eq!(
            HookVerdictKind::from_verdict(&HookVerdict::Allow),
            HookVerdictKind::Allow
        );
        assert_eq!(
            HookVerdictKind::from_verdict(&HookVerdict::Warn {
                message: "m".into(),
                pattern_hash: "h".into()
            }),
            HookVerdictKind::Warn
        );
        assert_eq!(
            HookVerdictKind::from_verdict(&HookVerdict::Block {
                message: "m".into(),
                pattern_hash: "h".into()
            }),
            HookVerdictKind::Block
        );
    }

    // Trait object safety.
    #[test]
    fn dispatcher_is_dyn_safe() {
        let d: Arc<dyn HookDispatcher> = Arc::new(InProcessHookDispatcher::new(None));
        assert!(Arc::strong_count(&d) >= 1);
    }

    /// B-19 W1: dispatch_stop with fsm_state="EXECUTING" MUST return a Block
    /// finding so task_runner can surface the rejection to the dashboard.
    #[tokio::test]
    async fn dispatch_stop_blocks_when_fsm_state_is_executing() {
        let d = InProcessHookDispatcher::new(None);
        let inv = HookInvocation::for_task_level("task_stop", "step_1")
            .with_fsm_state("EXECUTING");
        let findings = d.dispatch_stop(inv).await.expect("ok");
        assert!(
            findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
            "expected at least one Block finding for non-IDLE fsm_state, got: {:?}",
            findings,
        );
    }

    /// B-19 W1: dispatch_stop with fsm_state="IDLE" must NOT block.
    #[tokio::test]
    async fn dispatch_stop_allows_when_fsm_state_is_idle() {
        let d = InProcessHookDispatcher::new(None);
        let inv = HookInvocation::for_task_level("task_stop", "step_1")
            .with_fsm_state("IDLE");
        let findings = d.dispatch_stop(inv).await.expect("ok");
        assert!(
            !findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
            "expected no Block finding for IDLE fsm_state, got: {:?}",
            findings,
        );
    }

    /// B-19: PreCompact must NOOP when token_used / token_limit <= 0.85
    /// (the spec HANDOFF_THRESHOLD_FRACTION). Prevents SSB churn from
    /// per-task unconditional firing.
    #[tokio::test]
    async fn dispatch_pre_compact_noops_below_handoff_threshold() {
        let d = InProcessHookDispatcher::new(None);
        // 50% utilisation — well below 0.85.
        let inv = HookInvocation::for_task_level("pre_compact", "step_1")
            .with_pre_compact_token_state(5_000, 10_000);
        let findings = d.dispatch_pre_compact(inv).await.expect("ok");
        assert!(
            findings.is_empty(),
            "expected NOOP below threshold, got: {:?}",
            findings,
        );
    }

    /// B-19: PreCompact must run analyze() above the 0.85 threshold so the
    /// downstream T1 rules fire AND (when wired) the handoff saver runs.
    /// With no tiered backend installed, analyze() returns empty — the
    /// assertion is on the absence of an error, not findings count.
    #[tokio::test]
    async fn dispatch_pre_compact_runs_above_handoff_threshold() {
        let d = InProcessHookDispatcher::new(None);
        // 90% utilisation — above 0.85.
        let inv = HookInvocation::for_task_level("pre_compact", "step_1")
            .with_pre_compact_token_state(9_000, 10_000);
        let result = d.dispatch_pre_compact(inv).await;
        assert!(result.is_ok(), "expected Ok above threshold, got: {result:?}");
    }

    /// B-19: PreCompact with no token state (None) preserves legacy
    /// behaviour — analyze() runs, no gating happens. This is the
    /// task_runner per-task firing path that has no token telemetry.
    #[tokio::test]
    async fn dispatch_pre_compact_runs_when_no_token_state() {
        let d = InProcessHookDispatcher::new(None);
        let inv = HookInvocation::for_task_level("pre_compact", "step_1");
        assert_eq!(inv.pre_compact_token_state, None);
        let result = d.dispatch_pre_compact(inv).await;
        assert!(result.is_ok(), "expected Ok with no token state, got: {result:?}");
    }

    /// B-19: PRE_COMPACT_HANDOFF_FRACTION must match the spec value (0.85)
    /// and the canonical `escalation_policy::HANDOFF_THRESHOLD_FRACTION`.
    #[test]
    fn pre_compact_handoff_fraction_matches_spec() {
        assert!(
            (PRE_COMPACT_HANDOFF_FRACTION - 0.85_f64).abs() < f64::EPSILON,
            "PRE_COMPACT_HANDOFF_FRACTION must equal 0.85 per phase-4C.txt:28-29",
        );
    }

    /// B-19: handoff saver test — verifies the callback fires above
    /// threshold and does NOT fire below. Uses a simple atomic counter.
    #[tokio::test]
    async fn handoff_saver_fires_only_above_threshold() {
        use std::sync::atomic::{AtomicU32, Ordering};

        struct CountingSaver(Arc<AtomicU32>);
        impl PreCompactHandoffSaver for CountingSaver {
            fn save_handoff(&self, _step_id: &str) -> Result<(), String> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let counter = Arc::new(AtomicU32::new(0));
        let saver = Arc::new(CountingSaver(Arc::clone(&counter)));
        let d = InProcessHookDispatcher::new(None).with_handoff_saver(saver);

        // Below threshold: must NOT fire.
        let inv_below = HookInvocation::for_task_level("pre_compact", "step_b")
            .with_pre_compact_token_state(5_000, 10_000);
        let _ = d.dispatch_pre_compact(inv_below).await;
        assert_eq!(counter.load(Ordering::SeqCst), 0, "saver fired below threshold");

        // Above threshold: must fire once.
        let inv_above = HookInvocation::for_task_level("pre_compact", "step_a")
            .with_pre_compact_token_state(9_000, 10_000);
        let _ = d.dispatch_pre_compact(inv_above).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1, "saver did not fire above threshold");
    }

    /// B-19: IpcHookAsyncAdapter must produce a `dyn HookDispatcher` that
    /// is `Send + Sync` so it can be stored in `Arc<dyn HookDispatcher>`.
    #[test]
    fn ipc_hook_async_adapter_is_dyn_safe() {
        struct StubIpcHook;
        impl elai_plugins::IpcHookTrait for StubIpcHook {
            fn pre_tool_use(
                &self,
                _ctx: &elai_plugins::HookContext,
            ) -> Result<elai_plugins::HookResult, elai_plugins::IpcHookError> {
                Ok(elai_plugins::HookResult {
                    verdict: elai_plugins::tiered_analysis::HookVerdict::Allow,
                    event: elai_plugins::IpcHookEvent::PreToolUse,
                    latency_us: 0,
                    finding_written: false,
                })
            }
            fn post_tool_use(
                &self,
                _ctx: &elai_plugins::HookContext,
                _output: &str,
            ) -> Result<elai_plugins::HookResult, elai_plugins::IpcHookError> {
                Ok(elai_plugins::HookResult {
                    verdict: elai_plugins::tiered_analysis::HookVerdict::Allow,
                    event: elai_plugins::IpcHookEvent::PostToolUse,
                    latency_us: 0,
                    finding_written: false,
                })
            }
            fn stop(
                &self,
                _ctx: &elai_plugins::HookContext,
            ) -> Result<elai_plugins::HookResult, elai_plugins::IpcHookError> {
                Ok(elai_plugins::HookResult {
                    verdict: elai_plugins::tiered_analysis::HookVerdict::Allow,
                    event: elai_plugins::IpcHookEvent::Stop,
                    latency_us: 0,
                    finding_written: false,
                })
            }
            fn pre_compact(
                &self,
                _ctx: &elai_plugins::HookContext,
            ) -> Result<elai_plugins::HookResult, elai_plugins::IpcHookError> {
                Ok(elai_plugins::HookResult {
                    verdict: elai_plugins::tiered_analysis::HookVerdict::Allow,
                    event: elai_plugins::IpcHookEvent::PreCompact,
                    latency_us: 0,
                    finding_written: false,
                })
            }
        }
        let inner: Arc<dyn elai_plugins::IpcHookTrait + Send + Sync> = Arc::new(StubIpcHook);
        let adapter: Arc<dyn HookDispatcher> = Arc::new(IpcHookAsyncAdapter::new(inner));
        assert!(Arc::strong_count(&adapter) >= 1);
    }
}
