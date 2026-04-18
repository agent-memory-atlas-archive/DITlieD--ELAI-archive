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

use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, trace, warn};

use elai_plugins::tiered_analysis::{HookVerdict, TieredAnalysis, TieredAnalysisError};

use crate::shortcoder_hook::{ShortCoderHook, ShortCoderOutcome};
use crate::ssb::{
    HookEventPayload, ShortCoderCompressionPayload, Ssb, SsbPayload, TieredAnalysisPayload,
};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookVerdictKind {
    Allow,
    Warn,
    Block,
}

impl HookVerdictKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "Allow",
            Self::Warn => "Warn",
            Self::Block => "Block",
        }
    }

    #[must_use]
    pub fn from_verdict(v: &HookVerdict) -> Self {
        match v {
            HookVerdict::Allow => Self::Allow,
            HookVerdict::Warn { .. } => Self::Warn,
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
    pub metric_values: HashMap<String, f64>,
    /// Step identifier emitted onto the SSB HookEvent row (e.g.
    /// `"baby_step_42"`). Reserved for correlation with TestOutcome rows.
    pub step_id: String,
}

impl<'a> HookInvocation<'a> {
    /// Helper for the common "Stop/PreCompact" call where only a step_id is set.
    #[must_use]
    pub fn for_task_level(action_type: &'a str, step_id: impl Into<String>) -> Self {
        Self {
            action_type,
            source: None,
            file_path: None,
            metric_values: HashMap::new(),
            step_id: step_id.into(),
        }
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
            metric_values: HashMap::new(),
            step_id: step_id.into(),
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
            metric_values: HashMap::new(),
            step_id: step_id.into(),
        }
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
}

// ---------------------------------------------------------------------------
// Default in-process implementation
// ---------------------------------------------------------------------------

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
    disabled: bool,
}

impl InProcessHookDispatcher {
    /// Construct with an optional tiered-analysis backend. When `None` (e.g.
    /// the plugins crate wasn't seeded with rules), every dispatch returns an
    /// empty findings list. This is the "wiring present but no-op" surface
    /// referenced in the Batch 1A spec.
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
            disabled,
        }
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
        let result = tiered.analyze(
            invocation.action_type,
            invocation.source,
            invocation.file_path,
            invocation.metric_values.clone(),
        )?;
        trace!(
            action = invocation.action_type,
            duration_ms = result.duration_ms,
            "tiered analysis complete"
        );

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
        Box::pin(async move { self.analyze(&invocation) })
    }

    fn dispatch_post_tool_use<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        Box::pin(async move { self.analyze(&invocation) })
    }

    fn dispatch_stop<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        Box::pin(async move { self.analyze(&invocation) })
    }

    fn dispatch_pre_compact<'a>(
        &'a self,
        invocation: HookInvocation<'a>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<HookFinding>, HookError>> + Send + 'a>,
    > {
        Box::pin(async move {
            // TODO(batch-1b+): wire a real PreCompact site when the dashboard
            // path grows a compaction threshold. Keeping the method on the
            // trait now lets callers adopt it without a breaking change later.
            debug!(step = %invocation.step_id, "pre_compact invoked (MVP no-op)");
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
}
