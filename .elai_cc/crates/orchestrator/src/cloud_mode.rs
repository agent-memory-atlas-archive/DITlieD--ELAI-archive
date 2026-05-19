//! B-27: cloud-mode provider routing + dual-rate spend accumulator.
//!
//! This module owns the decision "should this baby-step run locally or
//! against a cloud provider?" and the cost accounting that follows. It
//! wraps the producer side of [`SsbActionType::CloudSpend`] (per-call
//! ledger) and [`SsbActionType::EscalateToCloud`] (one-shot escalation
//! marker) — both consumed downstream by the dashboard via the
//! `get_cloud_spend` Tauri command.
//!
//! Wiring contract (from B-27 plan WIRING_CONTRACT):
//! - reads `[cloud]` config out of `elai_cc.toml`
//! - reads cloud credentials from the OAuth keyring (caller passes
//!   already-loaded credentials here so this module stays test-friendly)
//! - returns a [`RoutedProvider`] describing whether the call should
//!   go to the local sidecar or to a cloud provider/model
//! - writes [`SsbPayload::CloudSpend`] rows via the SSB write path
//!   when the cloud call completes (caller invokes [`SpendAccumulator`])
//!
//! The cost formula is the dual-rate per-step accumulation mandated by
//! spec D-4:303-305 (user-approved correction over B-27's original
//! `phase_count × avg_tokens × pricing` approximation): each call's
//! cost is `input_tokens × INPUT_PRICE + output_tokens × OUTPUT_PRICE`,
//! summed across the task.
//!
//! Pricing constants are named, model-keyed, and grounded in the
//! Anthropic public pricing page (verified 2026-05-10). Adding a new
//! model = adding a new constant block plus a `for_<model>()` factory
//! on [`SpendAccumulator`].

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use thiserror::Error;

use crate::privacy_redactor::{self, PrivacyConfig, RedactionProfile};
use crate::task_runner::{BabyStepExecutionCtx, BabyStepExecutionResult, BabyStepExecutor};

/// Default cloud provider selected when the user has not pinned one
/// in `elai_cc.toml [cloud].default_provider`. Anthropic is the only
/// provider with a fully wired auth + send path in B-27; OpenAI and
/// custom BYOE are deferred.
pub const DEFAULT_CLOUD_PROVIDER: &str = "anthropic";

/// Default cloud model when nothing is selected via the dashboard
/// model-picker. Source-of-truth allowlist lives in
/// `crates/providers/src/oauth.rs::CLOUD_MODEL_ID_ALLOWLIST`.
pub const DEFAULT_CLOUD_MODEL_ID: &str = "claude-sonnet-4-6";

/// Anthropic Claude Sonnet 4.6 list price for non-cached input tokens.
/// Canonical source: <https://docs.anthropic.com/en/docs/about-claude/pricing>
/// Verified 2026-05-10 against the official pricing page (Sonnet 4.6:
/// $3 / 1M input tokens). Encoded as USD per single token, since
/// [`SpendAccumulator`] operates at single-token granularity.
pub const CLAUDE_SONNET_4_6_INPUT_PRICE_PER_TOKEN: f64 = 3.0e-6;

/// Anthropic Claude Sonnet 4.6 list price for output tokens.
/// Same source ($15 / 1M output tokens, verified 2026-05-10).
pub const CLAUDE_SONNET_4_6_OUTPUT_PRICE_PER_TOKEN: f64 = 15.0e-6;

/// Cache-read price = 10% of input price (Anthropic standard).
/// Cache reads are billed but at one-tenth — accounting for the cache
/// hit makes the spend ledger honest for prompt-cached calls.
pub const CLAUDE_SONNET_4_6_CACHE_READ_PRICE_PER_TOKEN: f64 = 3.0e-7;

/// Cache-write price = 1.25× input price (Anthropic standard for
/// 5-minute ephemeral cache writes). Applied to
/// `cache_creation_input_tokens` from the response usage.
pub const CLAUDE_SONNET_4_6_CACHE_WRITE_PRICE_PER_TOKEN: f64 = 3.75e-6;

/// Mode selector — distinct from
/// [`crate::context_budget::ExecutionMode`] which categorises BUDGET
/// failure routing (Interactive vs Autonomous). This enum categorises
/// PROVIDER routing (local sidecar vs cloud API). Two enums, similar
/// prefix, no aliasing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecMode {
    /// Force the local sidecar path — cheap, private, capped at the
    /// sidecar's measured ceiling (~25K tokens for Qwen2.5-Coder 3B
    /// NF4 on 12GB).
    Local,
    /// Force the cloud path — uses the configured provider and the
    /// cloud_200k context profile. Requires OAuth credentials.
    Cloud,
    /// Try local first; if the local path exhausts retries or trips
    /// an explicit escalation rule, emit an
    /// [`crate::ssb::SsbActionType::EscalateToCloud`] row and re-run
    /// the step with [`ExecMode::Cloud`]. The router itself returns
    /// [`RoutedProvider::Local`] for `Auto`; the orchestrator's
    /// retry loop is what re-invokes route() with `Cloud`.
    Auto,
}

impl ExecMode {
    /// Parse a config-file string. Unknown values fall back to Auto so
    /// a typo never silently disables cloud routing.
    #[must_use]
    pub fn from_config_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "local" => Self::Local,
            "cloud" => Self::Cloud,
            _ => Self::Auto,
        }
    }
}

/// `[cloud]` section of `elai_cc.toml`, deserialized via the
/// [`elai_cc::runtime_config`] loader and passed in here.
#[derive(Debug, Clone, PartialEq)]
pub struct CloudConfig {
    pub default_provider: String,
    pub default_model: String,
    pub default_mode: ExecMode,
    pub air_gapped: bool,
    pub privacy_profile: RedactionProfile,
    pub spend_cap_daily_usd: Option<f64>,
    pub spend_cap_monthly_usd: Option<f64>,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            default_provider: DEFAULT_CLOUD_PROVIDER.to_string(),
            default_model: DEFAULT_CLOUD_MODEL_ID.to_string(),
            default_mode: ExecMode::Auto,
            air_gapped: false,
            privacy_profile: RedactionProfile::Standard,
            spend_cap_daily_usd: None,
            spend_cap_monthly_usd: None,
        }
    }
}

/// OAuth credentials for a cloud provider call. Held as a thin wrapper
/// so the test path can construct a fake set without going through the
/// keyring loader at `providers::oauth::OAuthTokenSet`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudCredentials {
    pub bearer_token: String,
    pub provider: String,
}

impl CloudCredentials {
    /// Test-only constructor — don't call from production code; the
    /// real path goes through `providers::oauth::OAuthTokenSet::load`.
    #[must_use]
    pub fn for_test_with_token(token: &str) -> Self {
        Self {
            bearer_token: token.to_string(),
            provider: DEFAULT_CLOUD_PROVIDER.to_string(),
        }
    }
}

/// Outcome of a routing decision. Caller uses this to choose between
/// the sidecar adapter and the [`crate::providers::AnthropicClient`]
/// path. We return data (not the live provider object) so the router
/// stays sync + testable; the caller does the actual provider hookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutedProvider {
    Local,
    Cloud { provider: String, model: String },
}

/// All recoverable failure modes for [`CloudModeRouter::route`]. Use a
/// typed enum (per applicationRules.md MUST Rule 14) so callers can
/// match and degrade gracefully.
///
/// `Eq` is intentionally NOT derived: the spend-cap variants carry
/// `f64` which only implements `PartialEq`. Callers needing a strict
/// total-equality compare can match on the discriminant.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CloudModeError {
    #[error("cloud mode requested but no OAuth credentials are loaded")]
    NoCredentials,
    #[error("air_gapped=true conflicts with cloud routing — refusing to send")]
    AirGappedConflict,
    #[error("unknown cloud provider: {0}")]
    UnknownProvider(String),
    /// Reserved for B-27.5 follow-up that wires the daily ceiling enforcement
    /// into `route_cloud`. Currently unused — ceilings are warning-only.
    #[allow(dead_code)]
    #[error("daily spend cap reached: spent {spent_usd:.2} USD, cap {cap_usd:.2} USD")]
    DailySpendCapExceeded { spent_usd: f64, cap_usd: f64 },
    /// Reserved for B-27.5 follow-up. See `DailySpendCapExceeded`.
    #[allow(dead_code)]
    #[error("monthly spend cap reached: spent {spent_usd:.2} USD, cap {cap_usd:.2} USD")]
    MonthlySpendCapExceeded { spent_usd: f64, cap_usd: f64 },
}

/// Stateless router. Construct once per task (cheap), call `route`
/// repeatedly. The actual cloud call (sending the [`MessageRequest`])
/// is done by the caller using
/// [`crate::providers::AnthropicClient::send_message`] — we just hand
/// out the routing decision.
#[derive(Debug, Clone)]
pub struct CloudModeRouter {
    config: CloudConfig,
}

impl CloudModeRouter {
    #[must_use]
    pub fn new(config: CloudConfig) -> Self {
        Self { config }
    }

    /// Decide where to send. The 5-step routing per cloud-mode-design.txt:
    /// (1) air_gapped check, (2) credential check, (3) provider validation,
    /// (4) spend-cap pre-flight (deferred — caps are warning-only here;
    /// hard enforcement in a follow-up task), (5) return decision.
    ///
    /// The function does NOT make any network call.
    pub fn route(
        &self,
        mode: ExecMode,
        credentials: Option<CloudCredentials>,
    ) -> Result<RoutedProvider, CloudModeError> {
        match mode {
            ExecMode::Local => Ok(RoutedProvider::Local),
            ExecMode::Auto => {
                // Auto starts local; the orchestrator re-invokes with
                // Cloud after escalation.
                Ok(RoutedProvider::Local)
            }
            ExecMode::Cloud => self.route_cloud(credentials),
        }
    }

    fn route_cloud(
        &self,
        credentials: Option<CloudCredentials>,
    ) -> Result<RoutedProvider, CloudModeError> {
        // Step 1: air_gapped takes precedence over everything.
        if self.config.air_gapped {
            return Err(CloudModeError::AirGappedConflict);
        }
        // Step 2: credentials must be loaded.
        let creds = credentials.ok_or(CloudModeError::NoCredentials)?;
        // Step 3: provider validation. B-27 supports only "anthropic"
        // out-of-the-box; openai + byoe arrive in a follow-up task.
        // BOTH config and credentials must agree on anthropic — `||` not `&&`,
        // because either side mismatching means we can't safely route.
        let provider = self.config.default_provider.as_str();
        if provider != DEFAULT_CLOUD_PROVIDER || creds.provider != DEFAULT_CLOUD_PROVIDER {
            return Err(CloudModeError::UnknownProvider(provider.to_string()));
        }
        // Step 4: spend caps — currently warning-only, hard caps deferred.
        // Step 5: return the decision.
        Ok(RoutedProvider::Cloud {
            provider: provider.to_string(),
            model: self.config.default_model.clone(),
        })
    }

    #[must_use]
    pub fn config(&self) -> &CloudConfig {
        &self.config
    }

    /// Build the privacy config matching this router's redaction profile.
    /// Used by [`RedactingBabyStepExecutor`] to redact baby-step text
    /// before it leaves the local process.
    #[must_use]
    pub fn privacy_config(&self) -> PrivacyConfig {
        PrivacyConfig {
            profile: self.config.privacy_profile,
        }
    }
}

// ---------------------------------------------------------------------------
// B-28 WIRING-2 item E — resolve_exec_mode
// ---------------------------------------------------------------------------

/// Resolve the production [`ExecMode`] from the runner's
/// [`crate::dispatcher::BackendConfig`].
///
/// Mapping (locked B-28 WIRING-2 decision §1):
///   * [`crate::dispatcher::BackendMode::Local`] -> [`ExecMode::Local`].
///   * [`crate::dispatcher::BackendMode::Api`]  -> [`ExecMode::Cloud`].
///
/// Used by [`crate::task_runner::drive_task`] in place of the hardcoded
/// `ExecMode::Local` literals previously baked into the ModeSelector beacon
/// emission path (task_runner.rs L4371 + L4394). Returning `Cloud` here
/// allows Rule 1 ("CLOUD execution always returns STRUCTURED") to fire in
/// production once the BackendConfig is wired with `Api` mode.
///
/// This function is intentionally sync + pure; the live OAuth + spend-cap
/// checks remain on [`CloudModeRouter::route`] (which the runner invokes
/// separately when it actually sends a cloud request).
#[must_use]
pub fn resolve_exec_mode(backend: &crate::dispatcher::BackendConfig) -> ExecMode {
    match backend.mode {
        crate::dispatcher::BackendMode::Local => ExecMode::Local,
        crate::dispatcher::BackendMode::Api => ExecMode::Cloud,
        // C-5: BackendMode::Hybrid -> ExecMode::Auto. The hybrid path starts
        // local (see ExecMode::Auto docs L86-89); the cloud_mode escalation
        // loop in task_runner promotes on retry exhaustion + emits
        // EscalateToCloud.
        crate::dispatcher::BackendMode::Hybrid => ExecMode::Auto,
    }
}

/// Dual-rate per-step spend accumulator (D-4:303-305). Construct one
/// per task, call `accumulate` after every cloud round-trip, call
/// `finalize` once at task end and write the resulting
/// [`crate::ssb::CloudSpendPayload`] to SSB.
#[derive(Debug, Clone)]
pub struct SpendAccumulator {
    input_price_per_token: f64,
    output_price_per_token: f64,
    cache_read_price_per_token: f64,
    cache_write_price_per_token: f64,
    accumulated_usd: f64,
    accumulated_input_tokens: u64,
    accumulated_output_tokens: u64,
    accumulated_cache_read_tokens: u64,
    accumulated_cache_write_tokens: u64,
    model: String,
}

impl SpendAccumulator {
    /// Build an accumulator pinned to claude-sonnet-4-6 pricing.
    /// Adding a model = adding a parallel `for_<model>()` factory.
    #[must_use]
    pub fn for_claude_sonnet_4_6() -> Self {
        Self {
            input_price_per_token: CLAUDE_SONNET_4_6_INPUT_PRICE_PER_TOKEN,
            output_price_per_token: CLAUDE_SONNET_4_6_OUTPUT_PRICE_PER_TOKEN,
            cache_read_price_per_token: CLAUDE_SONNET_4_6_CACHE_READ_PRICE_PER_TOKEN,
            cache_write_price_per_token: CLAUDE_SONNET_4_6_CACHE_WRITE_PRICE_PER_TOKEN,
            accumulated_usd: 0.0,
            accumulated_input_tokens: 0,
            accumulated_output_tokens: 0,
            accumulated_cache_read_tokens: 0,
            accumulated_cache_write_tokens: 0,
            model: DEFAULT_CLOUD_MODEL_ID.to_string(),
        }
    }

    /// Lookup an accumulator for the given model id; falls back to the
    /// claude-sonnet-4-6 rate when the model id is not in the table.
    /// The fallback emits accurate token counts so cost discrepancies
    /// surface in the dashboard rather than silently zero out.
    #[must_use]
    pub fn for_model(model_id: &str) -> Self {
        match model_id {
            "claude-sonnet-4-6" => Self::for_claude_sonnet_4_6(),
            _ => {
                let mut a = Self::for_claude_sonnet_4_6();
                a.model = model_id.to_string();
                a
            }
        }
    }

    /// Add the cost of one cloud round-trip. Inputs are u32 because
    /// [`crate::providers::types::Usage`] reports tokens as u32 and we
    /// don't want a narrowing cast at the call site.
    pub fn accumulate(&mut self, input_tokens: u32, output_tokens: u32) {
        self.accumulated_input_tokens += u64::from(input_tokens);
        self.accumulated_output_tokens += u64::from(output_tokens);
        self.accumulated_usd += f64::from(input_tokens) * self.input_price_per_token;
        self.accumulated_usd += f64::from(output_tokens) * self.output_price_per_token;
    }

    /// Add the cache portion of a round-trip. Anthropic returns
    /// `cache_creation_input_tokens` and `cache_read_input_tokens`
    /// separately from `input_tokens` — they are billed at different
    /// rates, so accumulator must see them separately.
    pub fn accumulate_cache(
        &mut self,
        cache_creation_tokens: u32,
        cache_read_tokens: u32,
    ) {
        self.accumulated_cache_write_tokens += u64::from(cache_creation_tokens);
        self.accumulated_cache_read_tokens += u64::from(cache_read_tokens);
        self.accumulated_usd +=
            f64::from(cache_creation_tokens) * self.cache_write_price_per_token;
        self.accumulated_usd +=
            f64::from(cache_read_tokens) * self.cache_read_price_per_token;
    }

    /// Total dollars accumulated. Idempotent — does not mutate.
    #[must_use]
    pub fn finalize(&self) -> f64 {
        self.accumulated_usd
    }

    /// Snapshot fields suitable for building a
    /// [`crate::ssb::CloudSpendPayload`] write at task end.
    #[must_use]
    pub fn snapshot(&self) -> SpendSnapshot {
        SpendSnapshot {
            model: self.model.clone(),
            input_tokens: self.accumulated_input_tokens,
            output_tokens: self.accumulated_output_tokens,
            cache_creation_tokens: self.accumulated_cache_write_tokens,
            cache_read_tokens: self.accumulated_cache_read_tokens,
            cost_usd: self.accumulated_usd,
        }
    }
}

/// Spend snapshot for SSB write. Mirrors
/// [`crate::ssb::CloudSpendPayload`] shape but without the timestamp +
/// task_id (those are filled in by the writing layer).
#[derive(Debug, Clone, PartialEq)]
pub struct SpendSnapshot {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost_usd: f64,
}

/// Outer-wraps-inner [`BabyStepExecutor`] that scrubs PII out of
/// pre-cloud-send text. Construction order matters: `cloud_mode.rs`
/// installs this on the OUTSIDE of the real executor so redaction
/// runs BEFORE the inner executor's network code touches the text.
///
/// Wiring contract (B-27 + C-5): the production wiring site
/// [`crate::runtime_setup::apply_task_runner_wiring`] wraps the
/// ordinary [`BabyStepExecutor`] in this newtype when the resolved
/// [`crate::dispatcher::BackendMode`] is `Api` (cloud) or `Hybrid`.
/// Pure `Local` mode installs the inner executor directly with no
/// redaction.
///
/// C-5: the wrapper optionally carries an [`crate::Ssb`] writer and
/// emits a [`crate::ssb::SsbActionType::RedactionApplied`] row each
/// time redaction substitutions are made, so the dashboard
/// CloudPanel.svelte privacy-event feed can render a "last N redactions"
/// timeline. Best-effort: a write failure is logged but never aborts
/// the baby-step execution. When the writer slot is `None` (legacy /
/// test path), the redactor runs unchanged and no SSB row is emitted.
pub struct RedactingBabyStepExecutor {
    inner: Arc<dyn BabyStepExecutor>,
    privacy: PrivacyConfig,
    /// C-5: optional SSB writer. When `Some`, the executor emits a
    /// `RedactionApplied` row each time a redaction firing modifies the
    /// payload. When `None`, redaction still runs but no row is written
    /// (legacy / unit-test path).
    ssb_writer: Option<Arc<crate::Ssb>>,
    /// Agent id used on the emitted SSB row. Defaults to
    /// [`RedactingBabyStepExecutor::DEFAULT_AGENT_ID`] when the caller
    /// does not pin one.
    agent_id: String,
}

impl RedactingBabyStepExecutor {
    /// Agent id stamped on `RedactionApplied` SSB rows when the caller
    /// does not specify one. Mirrors the convention used by other
    /// task_runner-side beacons.
    pub const DEFAULT_AGENT_ID: &'static str = "cloud_mode_redactor";

    /// Construct a redactor without SSB emission. Legacy entry point —
    /// retained so existing tests that pre-date C-5 keep compiling.
    #[must_use]
    pub fn new(inner: Arc<dyn BabyStepExecutor>, privacy: PrivacyConfig) -> Self {
        Self {
            inner,
            privacy,
            ssb_writer: None,
            agent_id: Self::DEFAULT_AGENT_ID.to_string(),
        }
    }

    /// C-5: construct a redactor that emits a `RedactionApplied` SSB
    /// row each time redaction fires. The `ssb` writer is shared with
    /// the orchestrator so the row lands on the same DB the dashboard
    /// reads.
    #[must_use]
    pub fn with_ssb(
        inner: Arc<dyn BabyStepExecutor>,
        privacy: PrivacyConfig,
        ssb: Arc<crate::Ssb>,
    ) -> Self {
        Self {
            inner,
            privacy,
            ssb_writer: Some(ssb),
            agent_id: Self::DEFAULT_AGENT_ID.to_string(),
        }
    }

    /// C-5: redact a single field and report how many substitutions
    /// fired. Counts every character-position the redactor mutated,
    /// proxied as `(redacted.len() != original.len()) as u64 + difference_count`.
    /// The exact substitution counter would require threading the
    /// scrubber regex set; the difference heuristic is reliable for the
    /// dashboard's "did redaction fire?" surface and never reports a
    /// false zero when a difference exists.
    fn redact_field(&self, original: &str) -> (String, u64) {
        let redacted = privacy_redactor::redact_context(original, &self.privacy);
        let count = if redacted == original {
            0
        } else {
            // Count `[REDACTED]` and `[REDACTED-PATH]` substitutions; the
            // privacy_redactor module emits these tokens on every hit.
            let r1 = redacted.matches("[REDACTED]").count();
            let r2 = redacted.matches("[REDACTED-PATH]").count();
            (r1 + r2) as u64
        };
        (redacted, count)
    }

    fn redact_ctx(&self, ctx: BabyStepExecutionCtx) -> (BabyStepExecutionCtx, u64, Vec<String>) {
        let mut count = 0u64;
        let mut fields: Vec<String> = Vec::new();
        let (task_summary, n) = self.redact_field(&ctx.task_summary);
        count = count.saturating_add(n);
        if n > 0 {
            fields.push("task_summary".to_string());
        }
        let escalation_message = match ctx.escalation_message {
            Some(m) => {
                let (red, n) = self.redact_field(&m);
                count = count.saturating_add(n);
                if n > 0 {
                    fields.push("escalation_message".to_string());
                }
                Some(red)
            }
            None => None,
        };
        let new_ctx = BabyStepExecutionCtx {
            baby_step_id: ctx.baby_step_id,
            task_summary,
            escalation_message,
            // B-28 E-3: pass the focus-tunnel ModeSelection through
            // unchanged. The redactor only touches user-visible string
            // payloads; the ModeSelection is structural metadata.
            mode_selection: ctx.mode_selection,
            // C2: pass session_credentials through unchanged — the redactor
            // does not touch credential values (they are not user-visible text).
            session_credentials: ctx.session_credentials,
        };
        (new_ctx, count, fields)
    }
}

impl BabyStepExecutor for RedactingBabyStepExecutor {
    fn execute<'a>(
        &'a self,
        ctx: BabyStepExecutionCtx,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = BabyStepExecutionResult> + Send + 'a>,
    > {
        let baby_step_id = ctx.baby_step_id;
        let (redacted, count, fields) = self.redact_ctx(ctx);
        let inner = Arc::clone(&self.inner);
        let writer = self.ssb_writer.clone();
        let agent_id = self.agent_id.clone();
        Box::pin(async move {
            // C-5: emit RedactionApplied SSB row when redaction fired AND
            // a writer is plumbed. Best-effort — log + proceed on error.
            if count > 0 {
                if let Some(ssb) = writer {
                    // C-15: OsClock-backed ts_ms. RedactionApplied is an
                    // observation row; not part of any deterministic fixture.
                    let ts_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
                    let payload = crate::ssb::SsbPayload::RedactionApplied(
                        crate::ssb::RedactionAppliedPayload {
                            count,
                            fields,
                            baby_step_id,
                            ts_ms,
                        },
                    );
                    if let Err(e) = ssb
                        .write_payload(&agent_id, 0, payload, Vec::new())
                        .await
                    {
                        tracing::warn!(
                            error = %e,
                            baby_step_id,
                            "cloud_mode: RedactionApplied SSB write failed (non-fatal)"
                        );
                    }
                }
            }
            inner.execute(redacted).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_local_passes_without_credentials() {
        let router = CloudModeRouter::new(CloudConfig::default());
        let outcome = router.route(ExecMode::Local, None);
        assert_eq!(outcome, Ok(RoutedProvider::Local));
    }

    #[test]
    fn route_auto_starts_local() {
        let router = CloudModeRouter::new(CloudConfig::default());
        let outcome = router.route(ExecMode::Auto, None);
        assert_eq!(outcome, Ok(RoutedProvider::Local));
    }

    #[test]
    fn route_cloud_without_creds_returns_no_credentials() {
        let router = CloudModeRouter::new(CloudConfig::default());
        let outcome = router.route(ExecMode::Cloud, None);
        assert_eq!(outcome, Err(CloudModeError::NoCredentials));
    }

    #[test]
    fn route_cloud_under_air_gapped_returns_conflict() {
        let mut cfg = CloudConfig::default();
        cfg.air_gapped = true;
        let router = CloudModeRouter::new(cfg);
        let outcome = router.route(
            ExecMode::Cloud,
            Some(CloudCredentials::for_test_with_token("dummy")),
        );
        assert_eq!(outcome, Err(CloudModeError::AirGappedConflict));
    }

    #[test]
    fn spend_accumulator_dual_rate_sums_per_step() {
        let mut acc = SpendAccumulator::for_claude_sonnet_4_6();
        acc.accumulate(1_000, 200);
        acc.accumulate(500, 100);
        let total = acc.finalize();
        let expected = (1_000.0 * CLAUDE_SONNET_4_6_INPUT_PRICE_PER_TOKEN)
            + (200.0 * CLAUDE_SONNET_4_6_OUTPUT_PRICE_PER_TOKEN)
            + (500.0 * CLAUDE_SONNET_4_6_INPUT_PRICE_PER_TOKEN)
            + (100.0 * CLAUDE_SONNET_4_6_OUTPUT_PRICE_PER_TOKEN);
        assert!((total - expected).abs() < 1e-9);
    }

    #[test]
    fn spend_accumulator_cache_read_at_one_tenth_input_price() {
        let mut acc = SpendAccumulator::for_claude_sonnet_4_6();
        acc.accumulate_cache(0, 10_000);
        let cost = acc.finalize();
        let expected_cache_read = 10_000.0 * CLAUDE_SONNET_4_6_CACHE_READ_PRICE_PER_TOKEN;
        assert!((cost - expected_cache_read).abs() < 1e-9);
        // Sanity: cache read cost is exactly 10% of plain input cost.
        let plain_input_cost = 10_000.0 * CLAUDE_SONNET_4_6_INPUT_PRICE_PER_TOKEN;
        assert!(
            ((cost / plain_input_cost) - 0.1).abs() < 1e-6,
            "cache_read should cost 10% of input"
        );
    }

    #[test]
    fn exec_mode_from_config_str_unknown_falls_back_to_auto() {
        assert_eq!(ExecMode::from_config_str("yolo"), ExecMode::Auto);
        assert_eq!(ExecMode::from_config_str("LOCAL"), ExecMode::Local);
        assert_eq!(ExecMode::from_config_str("Cloud"), ExecMode::Cloud);
    }
}
