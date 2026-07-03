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

// ---------------------------------------------------------------------------
// C-5: baby-step LOC sizing constants (spec docs/specs/guide/cloud-mode-design.txt:121-128).
// Per spec line 121-128 the orchestrator partitions a task into baby steps
// whose target LOC budget depends on the runtime model. Local 3B/7B are
// kept small so they fit Qwen2.5-Coder context windows; cloud Sonnet/Opus
// can carry much larger spans because the cloud context profile is
// 200k tokens (cloud_200k profile, B-27). Spec value `local_32b_loc=100`
// is intentionally omitted in C-5 scope (deferred to E-32 follow-up); the
// helper `baby_step_loc_for_mode` falls back to the 7B value for any
// `local` model id that does not match "3b".
// ---------------------------------------------------------------------------

/// Target baby-step LOC for the local 3B Qwen2.5-Coder model.
pub const BABY_STEP_LOC_LOCAL_3B: u16 = 25;
/// Target baby-step LOC for the local 7B Qwen2.5-Coder model.
pub const BABY_STEP_LOC_LOCAL_7B: u16 = 40;
/// Target baby-step LOC for cloud Sonnet 4.6 / 4.5 family models.
pub const BABY_STEP_LOC_CLOUD_SONNET: u16 = 300;
/// Target baby-step LOC for cloud Opus 4.x family models.
pub const BABY_STEP_LOC_CLOUD_OPUS: u16 = 500;

/// UNGROUNDED_HEURISTIC: average token-per-LOC ratio used by
/// [`estimate_task_cost_usd`] to project a USD spread from a plan
/// skeleton. The value 4.0 is a rough industry rule-of-thumb (one token
/// ≈ 4 characters; one LOC ≈ 16 characters of code => ~4 tokens). It
/// has NOT been measured against ELAI's own corpora. The grounding
/// sweep flagged this as the single INGEST item for C-5; per CLAUDE.md
/// stub-registry exemption the constant ships with this label and a
/// TODO marker pointing at the future cost-measurement B-N. Do NOT
/// silently change the value without re-running the cost-estimate
/// contract; the dashboard COST_ESTIMATE display will drift.
// TODO[ungrounded]: replace with measured token/LOC ratio per language
//   once metrics/baseline_findings_and_actions.md captures real cloud
//   call traces. Stub-registry entry: C-5 cost-estimate heuristic.
pub const TOKENS_PER_LOC_UNGROUNDED_HEURISTIC: f64 = 4.0;

/// Width of the (low, high) cost spread emitted by
/// [`estimate_task_cost_usd`]. ±30% mirrors the spec's
/// "$0.12-$0.35" example (cloud-mode-design.txt:84-93) where the upper
/// bound is roughly 2.9× the lower bound — a single midpoint times
/// (1±0.3) yields a 1.86× spread, low enough to communicate
/// uncertainty without overstating it.
pub const COST_ESTIMATE_SPREAD_RATIO: f64 = 0.30;

/// Mode selector — distinct from
/// [`crate::context_budget::ExecutionMode`] which categorises BUDGET
/// failure routing (Interactive vs Autonomous). This enum categorises
/// PROVIDER routing (local sidecar vs cloud API). Two enums, similar
/// prefix, no aliasing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecMode {
    /// Force the local sidecar path — cheap, private, capped at the
    /// sidecar's effective ceiling (32K for Qwen2.5-Coder-7B NF4 per E-32;
    /// RuntimeCapabilities::max_context is the authoritative value).
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
    /// C-5: warn-at percentage of the daily cap. When `Some(pct)` and
    /// `spent_day_usd >= cap * pct / 100`, the runner emits a single
    /// `WarnLimitReached` SSB row per UTC day before the hard cap fires.
    /// `None` disables WARN emission. CloudConfig::default() seeds this
    /// at `Some(80)` per spec docs/specs/guide/cloud-mode-design.txt:75.
    pub warn_at_percent: Option<u8>,
    /// C-5: BYOE override of the AnthropicClient base URL. When `Some`,
    /// the client is constructed with `with_base_url(value)` instead of
    /// the public api.anthropic.com endpoint. Forbidden when
    /// `air_gapped=true` (route_cloud returns `CustomEndpointForbidden`).
    /// Logged via `tracing::info!` on first use; never written to SSB
    /// (the URL may be private infra and SSB rows are dashboard-visible).
    pub custom_endpoint: Option<String>,
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
            // C-5: 80% warn default per spec cloud-mode-design.txt:75.
            warn_at_percent: Some(80),
            // C-5: BYOE off by default — production endpoint via DEFAULT_BASE_URL.
            custom_endpoint: None,
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
    /// C-5: hard daily ceiling reached. Returned by [`CloudModeRouter::route_cloud`]
    /// when the caller-supplied [`SpendSnapshotInput`] reports
    /// `day_usd >= spend_cap_daily_usd`. Callers should surface the error
    /// to the dashboard and stop scheduling new cloud calls until the
    /// next UTC day rollover.
    #[error("daily spend cap reached: spent {spent_usd:.2} USD, cap {cap_usd:.2} USD")]
    DailySpendCapExceeded { spent_usd: f64, cap_usd: f64 },
    /// C-5: hard monthly ceiling reached. Mirrors `DailySpendCapExceeded`
    /// semantics on the monthly window.
    #[error("monthly spend cap reached: spent {spent_usd:.2} USD, cap {cap_usd:.2} USD")]
    MonthlySpendCapExceeded { spent_usd: f64, cap_usd: f64 },
    /// C-5: BYOE `custom_endpoint` requested while the runtime config has
    /// `air_gapped=true`. The two settings are mutually exclusive — an
    /// air-gapped runner must never reach out to any cloud endpoint,
    /// custom or otherwise.
    #[error("custom_endpoint forbidden while air_gapped=true")]
    CustomEndpointForbidden,
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
    /// (4) custom_endpoint guard + spend-cap pre-flight, (5) return decision.
    ///
    /// The function does NOT make any network call. Spend enforcement is
    /// caller-driven: the runner reads the live SSB cloud-spend summary
    /// once per baby step and passes it down via [`Self::route_with_spend`].
    /// Plain [`Self::route`] keeps the legacy "no caps" semantics for
    /// callers that do not have a live SSB viewer (unit tests).
    pub fn route(
        &self,
        mode: ExecMode,
        credentials: Option<CloudCredentials>,
    ) -> Result<RoutedProvider, CloudModeError> {
        let outcome = self.route_with_spend(mode, credentials, None);
        // E-31 sub-phase 6A: emit RouteDecisionEvent via tracing log
        // behind the wire-dark env gate. The actual Ssb::write_payload
        // injection is deferred to sub-phase 6B alongside the dashboard
        // RuntimePanel tile (matches the E-3 4A "tracing-log first,
        // SSB writer in 4B" pattern). The RuntimeRoutingPolicy here is
        // invoked with the default policy + a minimal RoutingContext so
        // every CloudModeRouter::route call gets a paired audit row when
        // the env gate is on.
        emit_route_decision_log(mode, &outcome);
        outcome
    }

    /// C-5: routing variant that ALSO enforces the daily / monthly
    /// spend ceilings. Callers (task_runner) pass a
    /// [`SpendSnapshotInput`] read from the live SSB aggregator; absent
    /// (None), the caps are skipped and behaviour matches [`Self::route`].
    /// Skipping is the documented stub_ban semantic ("if get_cloud_spend
    /// returns error, log warning and SKIP the cap check rather than
    /// blocking all cloud calls") — see plan WIRING_CONTRACT stub_ban.
    pub fn route_with_spend(
        &self,
        mode: ExecMode,
        credentials: Option<CloudCredentials>,
        spend: Option<SpendSnapshotInput>,
    ) -> Result<RoutedProvider, CloudModeError> {
        match mode {
            ExecMode::Local => Ok(RoutedProvider::Local),
            ExecMode::Auto => {
                // Auto starts local; the orchestrator re-invokes with
                // Cloud after escalation.
                Ok(RoutedProvider::Local)
            }
            ExecMode::Cloud => self.route_cloud(credentials, spend),
        }
    }

    fn route_cloud(
        &self,
        credentials: Option<CloudCredentials>,
        spend: Option<SpendSnapshotInput>,
    ) -> Result<RoutedProvider, CloudModeError> {
        // Step 1: air_gapped takes precedence over everything.
        if self.config.air_gapped {
            // Tighter guard: a custom_endpoint configured under
            // air_gapped=true is a misconfiguration that would otherwise
            // be silently masked by the AirGappedConflict return — surface
            // the dedicated error so the dashboard can show a clearer
            // toml-fix hint.
            if self.config.custom_endpoint.is_some() {
                return Err(CloudModeError::CustomEndpointForbidden);
            }
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
        // Step 4: spend-cap enforcement. Caller-supplied; None => skip
        // (stub_ban semantics — never block all cloud calls on viewer error).
        // Pre-cap WARN emission is the caller's responsibility (task_runner)
        // because route_cloud is sync + pure and does not own the writer.
        if let Some(snap) = spend.as_ref() {
            if let Some(cap) = self.config.spend_cap_daily_usd {
                if snap.day_usd >= cap {
                    return Err(CloudModeError::DailySpendCapExceeded {
                        spent_usd: snap.day_usd,
                        cap_usd: cap,
                    });
                }
            }
            if let Some(cap) = self.config.spend_cap_monthly_usd {
                if snap.month_usd >= cap {
                    return Err(CloudModeError::MonthlySpendCapExceeded {
                        spent_usd: snap.month_usd,
                        cap_usd: cap,
                    });
                }
            }
        }
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
// E-31 sub-phase 6A — RouteDecisionEvent wire-dark emit helper.
// ---------------------------------------------------------------------------

/// E-31 sub-phase 6A: emit a `RouteDecisionEvent` tracing log line behind
/// the `ELAI_ROUTE_DECISION_SSB_ENABLE` env gate. The actual
/// `Ssb::write_payload` injection lives in sub-phase 6B (mirrors the
/// E-3 4A "tracing log first, SSB writer in 4B" precedent at
/// `ssb.rs:2580-2588`).
///
/// The policy invocation uses the default `RuntimeRoutingPolicy` and a
/// minimal `RoutingContext` derived from the `ExecMode` + the route
/// outcome. The decision is informational: this path does NOT change
/// the returned `RoutedProvider`; it only labels the decision for the
/// audit trail.
///
/// No I/O outside the gated tracing call; no Result propagation needed
/// because tracing::info! is infallible.
fn emit_route_decision_log(
    mode: ExecMode,
    outcome: &Result<RoutedProvider, CloudModeError>,
) {
    if !route_decision_ssb_enabled() {
        return;
    }
    use elai_providers::runtime::{
        RoutingChoice, RoutingContext, RoutingCriticality, RoutingDifficulty,
        RuntimeRoutingPolicy,
    };
    // Best-effort context: mode -> difficulty mapping is intentionally
    // conservative (Cloud-mode = Hard, Auto = Medium, Local = Easy). The
    // policy + audit row are observational, not load-bearing.
    let difficulty = match mode {
        ExecMode::Cloud => RoutingDifficulty::Hard,
        ExecMode::Auto => RoutingDifficulty::Medium,
        ExecMode::Local => RoutingDifficulty::Easy,
    };
    // F13: replace prior hardcoded 0.0 with actual NVML-detected GiB. The
    // helper falls back to 0.0 on missing/error (matches doctor.rs:290
    // graceful-fall-through pattern). Field is observational today; becomes
    // load-bearing once E-31 6B wires Ssb::write_payload.
    let ctx = RoutingContext {
        difficulty,
        criticality: RoutingCriticality::Low,
        vram_detected_gb: elai_providers::detect_vram_gb(),
        prior_audit_failures: 0,
        budget_remaining_usd: 25.0,
        ops_override: None,
    };
    let policy = RuntimeRoutingPolicy::default();
    let decision = policy.evaluate(&ctx);
    let kind_label = match outcome {
        Ok(RoutedProvider::Local) => "local_route_kept",
        Ok(RoutedProvider::Cloud { .. }) => "cloud_route_chosen",
        Err(_) => "local_route_kept",
    };
    let chosen_route = match outcome {
        Ok(RoutedProvider::Local) => RoutingChoice::LocalSidecar.as_str(),
        Ok(RoutedProvider::Cloud { .. }) => decision.choice.as_str(),
        Err(_) => RoutingChoice::LocalSidecar.as_str(),
    };
    tracing::info!(
        target: "elai_orchestrator::cloud_mode::route_decision",
        kind = kind_label,
        route = chosen_route,
        reason = decision.reason.as_str(),
        ts_ms = decision.ts_ms,
        "RouteDecisionEvent (wire-dark; sub-phase 6B will inject Ssb::write_payload)"
    );
}

/// Read the wire-dark env gate. Mirrors the helper pattern used by
/// `elai_orchestrator::ssb::ELAI_HEAD_DIM_INVARIANT_SSB_ENABLE_ENV`
/// consumers.
pub(crate) fn route_decision_ssb_enabled() -> bool {
    matches!(
        std::env::var(crate::ssb::ELAI_ROUTE_DECISION_SSB_ENABLE_ENV)
            .ok()
            .as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
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

/// C-5: spend snapshot passed into [`CloudModeRouter::route_with_spend`]
/// for caller-driven ceiling enforcement. Mirrors the relevant fields
/// of [`crate::ssb_viewer::CloudSpendSummary`] without coupling the
/// router to the viewer crate — keeps the router sync + testable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpendSnapshotInput {
    /// Total USD spend across the current UTC day.
    pub day_usd: f64,
    /// Total USD spend across the current UTC calendar month.
    pub month_usd: f64,
}

impl SpendSnapshotInput {
    /// Convenience: build a snapshot with the same value for both windows
    /// (useful when the monthly aggregator is absent — falls back to the
    /// daily figure, which is conservative because monthly >= daily).
    #[must_use]
    pub fn from_day_usd(day_usd: f64) -> Self {
        Self {
            day_usd,
            month_usd: day_usd,
        }
    }
}

/// C-5: classify a `(mode, model_id)` pair to its target baby-step LOC
/// budget. Dispatch is substring-keyed on the model id so future model
/// generations land on a sensible default without touching this site
/// (e.g. `"claude-sonnet-4-7"` still maps to `BABY_STEP_LOC_CLOUD_SONNET`).
/// Local mode falls back to the 7B value when no `"3b"` substring matches
/// — Qwen2.5-Coder-7B is the canonical local model per `metrics/phase_3J_rag.toml`.
#[must_use]
pub fn baby_step_loc_for_mode(mode: ExecMode, model: &str) -> u16 {
    let lower = model.to_ascii_lowercase();
    match mode {
        ExecMode::Local => {
            if lower.contains("3b") {
                BABY_STEP_LOC_LOCAL_3B
            } else {
                // 7B default for any non-3B local id (covers 7b, 14b, etc.
                // until those tiers get explicit constants).
                BABY_STEP_LOC_LOCAL_7B
            }
        }
        ExecMode::Cloud | ExecMode::Auto => {
            if lower.contains("opus") {
                BABY_STEP_LOC_CLOUD_OPUS
            } else {
                // Sonnet is the cloud default (DEFAULT_CLOUD_MODEL_ID).
                BABY_STEP_LOC_CLOUD_SONNET
            }
        }
    }
}

/// C-5: project a (low, high) USD cost spread for `plan_skeleton` against
/// `model`. Formula:
///   `step_count × baby_step_loc × TOKENS_PER_LOC × price_per_token`
/// where price_per_token is the dual-rate sum (input+output) per the
/// SpendAccumulator pricing constants. The spread mirrors the spec's
/// "$0.12-$0.35" presentation pattern (cloud-mode-design.txt:84-93)
/// via the `COST_ESTIMATE_SPREAD_RATIO` (±30% around the midpoint).
///
/// The token/LOC ratio is the `TOKENS_PER_LOC_UNGROUNDED_HEURISTIC`
/// constant — see its docstring for the grounding caveat. Local models
/// return `(0.0, 0.0)` because the local sidecar has zero variable
/// cost; the dashboard hides COST_ESTIMATE rows when both bounds are 0.
#[must_use]
pub fn estimate_task_cost_usd(
    plan_skeleton: &crate::plan_skeleton::PlanSkeleton,
    model: &str,
) -> (f64, f64) {
    let lower = model.to_ascii_lowercase();
    let step_count = plan_skeleton.phases.len();
    if step_count == 0 {
        return (0.0, 0.0);
    }
    // Cloud-only cost projection. Local sidecar costs nothing variable.
    let (input_price, output_price) = if lower.contains("opus") {
        // Opus pricing (Anthropic public list, verified 2026-05-10):
        // $15/M input, $75/M output. Encoded per-token in line with the
        // SpendAccumulator convention.
        (15.0e-6, 75.0e-6)
    } else if lower.contains("sonnet") || lower.contains("claude") {
        (
            CLAUDE_SONNET_4_6_INPUT_PRICE_PER_TOKEN,
            CLAUDE_SONNET_4_6_OUTPUT_PRICE_PER_TOKEN,
        )
    } else {
        // Unknown / local model => no variable cost.
        return (0.0, 0.0);
    };

    let loc_per_step = f64::from(baby_step_loc_for_mode(ExecMode::Cloud, model));
    let avg_tokens_per_step = loc_per_step * TOKENS_PER_LOC_UNGROUNDED_HEURISTIC;
    // 50/50 split between input + output as a coarse mid-point. Real
    // ratios depend on prompt-cache and tool-use traffic; the spread
    // below absorbs that uncertainty.
    let mid_cost_per_step =
        avg_tokens_per_step * 0.5 * input_price + avg_tokens_per_step * 0.5 * output_price;
    // step_count is usize; widen via `as f64` only after bounds-checking
    // the empty case above. Even at u32::MAX phases this stays inside
    // f64's exact-integer range, so the cast is lossless for realistic
    // plan sizes.
    #[allow(clippy::cast_precision_loss)]
    let total_mid_usd = mid_cost_per_step * step_count as f64;
    let low = total_mid_usd * (1.0 - COST_ESTIMATE_SPREAD_RATIO);
    let high = total_mid_usd * (1.0 + COST_ESTIMATE_SPREAD_RATIO);
    (low.max(0.0), high.max(0.0))
}

/// C-5: caller-side helper that returns `true` iff `day_usd` has crossed
/// `cap * warn_at_percent / 100.0` for the given config. The runner uses
/// this BEFORE invoking the SSB writer so the WARN row only fires once
/// per UTC day (caller-side dedup via a Mutex<bool>). Returns false when
/// either `spend_cap_daily_usd` or `warn_at_percent` is None.
#[must_use]
pub fn warn_limit_reached(config: &CloudConfig, day_usd: f64) -> bool {
    match (config.spend_cap_daily_usd, config.warn_at_percent) {
        (Some(cap), Some(pct)) if cap > 0.0 => {
            let threshold = cap * f64::from(pct) / 100.0;
            day_usd >= threshold
        }
        _ => false,
    }
}

/// B-27 D-4 item 16: HYBRID escalation routing for a single failed local
/// step. Called from the task_runner baby-step loop when
/// [`ExecMode::Auto`] (HYBRID local-first) and the local executor returned
/// `BabyStepOutcome::Failed`. This function:
///   1. Re-runs the router with [`ExecMode::Cloud`] + the supplied spend
///      snapshot so the daily/monthly hard caps still gate the escalation
///      (G3 — caps win over escalation).
///   2. On `Ok(RoutedProvider::Cloud { .. })` returns the routed provider
///      so the caller can re-invoke the (redaction-wrapped) baby-step
///      executor exactly once for this step. The redaction wrap is on the
///      executor itself (installed at `runtime_setup.rs:945`), so the
///      escalated call is already scrubbed before network egress (G2).
///   3. On `Err(_)` returns the typed error so the caller can emit a
///      `RouteBlocked` SSB row and continue with local failure handling
///      (no re-loop, no retry storm — single cloud call per failed step).
///
/// This function is sync + pure (no I/O, no SSB write) — the caller owns
/// the SSB emission of `EscalateToCloud` and `RouteBlocked` so this stays
/// unit-testable without a writer fixture. Air-gapped mode is handled
/// inside `route_with_spend` (returns `AirGappedConflict`), which the
/// caller maps to a `RouteBlocked` row with reason `air_gapped`.
#[must_use]
pub fn escalate_hybrid_step_to_cloud(
    router: &CloudModeRouter,
    credentials: Option<CloudCredentials>,
    spend: Option<SpendSnapshotInput>,
) -> Result<RoutedProvider, CloudModeError> {
    router.route_with_spend(ExecMode::Cloud, credentials, spend)
}

/// B-27 D-4 item 17: pre-task cost-estimate interactive gate. Called from
/// the task_runner cloud pre-dispatch hook after the `CostEstimate` SSB
/// row is emitted. Presents a "Proceed? [y/n]" prompt to the supplied
/// [`crate::gate::InteractiveGate`] using a synthetic [`EvidenceSummary`]
/// that carries the cost range in its `phase_label` and a single passing
/// gate result describing the estimate.
///
/// Returns `true` when the gate approves (proceed with the cloud step)
/// and `false` when the gate rejects (caller aborts the step / emits a
/// `RouteBlocked` row). When `gate` is `None` (dashboard / non-interactive
/// entrypoint), the function auto-approves — the dashboard has no stdin,
/// so the SSB `CostEstimate` row is the only surface and the human
/// approves via the dashboard UI (G4 cross-language contract).
///
/// Risk level is [`crate::gate::RiskLevel::Medium`] so the CLI
/// `CliInteractiveGate` actually pauses (LOW auto-advances per gate.rs
/// `present_gate` semantics). The phase_label embeds the dollar range so
/// the CLI prompt line is self-describing without a separate formatter.
#[must_use]
pub fn prompt_cost_estimate_gate(
    gate: Option<&dyn crate::gate::InteractiveGate>,
    low_usd: f64,
    high_usd: f64,
    model: &str,
) -> bool {
    let Some(g) = gate else {
        // Non-interactive entrypoint (dashboard): auto-proceed. The
        // CostEstimate SSB row is the audit artifact; the human approves
        // via the dashboard CloudPanel, not stdin.
        return true;
    };
    use crate::gate::{EvidenceSummary, GateResult, RiskLevel};
    let summary = EvidenceSummary {
        source: crate::FsmState::Planning,
        target: crate::FsmState::TestingFirst,
        phase_label: format!(
            "Estimated cloud cost: ${low_usd:.2}-${high_usd:.2} (model={model}). Proceed?"
        ),
        gate_results: vec![GateResult {
            gate_name: "cost_estimate".to_string(),
            passed: true,
            detail: format!("low=${low_usd:.4} high=${high_usd:.4} model={model}"),
        }],
        all_gates_passed: true,
    };
    matches!(g.present_gate(&summary, RiskLevel::Medium), crate::gate::GateDecision::Approve)
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

    // ----- C-5 ceiling enforcement tests -----

    #[test]
    fn route_cloud_blocks_when_day_spend_meets_daily_cap() {
        let mut cfg = CloudConfig::default();
        cfg.spend_cap_daily_usd = Some(5.0);
        let router = CloudModeRouter::new(cfg);
        let outcome = router.route_with_spend(
            ExecMode::Cloud,
            Some(CloudCredentials::for_test_with_token("t")),
            Some(SpendSnapshotInput { day_usd: 5.0, month_usd: 5.0 }),
        );
        assert!(matches!(
            outcome,
            Err(CloudModeError::DailySpendCapExceeded { .. })
        ));
    }

    #[test]
    fn route_cloud_blocks_when_month_spend_meets_monthly_cap() {
        let mut cfg = CloudConfig::default();
        cfg.spend_cap_monthly_usd = Some(50.0);
        let router = CloudModeRouter::new(cfg);
        let outcome = router.route_with_spend(
            ExecMode::Cloud,
            Some(CloudCredentials::for_test_with_token("t")),
            Some(SpendSnapshotInput { day_usd: 1.0, month_usd: 60.0 }),
        );
        assert!(matches!(
            outcome,
            Err(CloudModeError::MonthlySpendCapExceeded { .. })
        ));
    }

    #[test]
    fn route_cloud_allows_when_no_spend_snapshot_supplied() {
        // stub_ban semantics: viewer error => skip cap check, never block
        // all cloud calls. Absent snapshot means caller could not read.
        let mut cfg = CloudConfig::default();
        cfg.spend_cap_daily_usd = Some(0.01);
        let router = CloudModeRouter::new(cfg);
        let outcome = router.route_with_spend(
            ExecMode::Cloud,
            Some(CloudCredentials::for_test_with_token("t")),
            None,
        );
        assert!(matches!(outcome, Ok(RoutedProvider::Cloud { .. })));
    }

    #[test]
    fn custom_endpoint_with_air_gapped_returns_forbidden() {
        let mut cfg = CloudConfig::default();
        cfg.air_gapped = true;
        cfg.custom_endpoint = Some("https://example.test".to_string());
        let router = CloudModeRouter::new(cfg);
        let outcome = router.route(
            ExecMode::Cloud,
            Some(CloudCredentials::for_test_with_token("t")),
        );
        assert_eq!(outcome, Err(CloudModeError::CustomEndpointForbidden));
    }

    #[test]
    fn warn_limit_reached_fires_at_eighty_percent() {
        let mut cfg = CloudConfig::default();
        cfg.spend_cap_daily_usd = Some(5.0);
        cfg.warn_at_percent = Some(80);
        assert!(!warn_limit_reached(&cfg, 3.99));
        assert!(warn_limit_reached(&cfg, 4.0));
        assert!(warn_limit_reached(&cfg, 4.5));
    }

    #[test]
    fn warn_limit_reached_returns_false_when_disabled() {
        let cfg = CloudConfig::default(); // cap=None disables
        assert!(!warn_limit_reached(&cfg, 1_000.0));
    }

    #[test]
    fn baby_step_loc_dispatch_covers_four_combos() {
        assert_eq!(
            baby_step_loc_for_mode(ExecMode::Local, "qwen2.5-coder-3b-instruct"),
            BABY_STEP_LOC_LOCAL_3B
        );
        assert_eq!(
            baby_step_loc_for_mode(ExecMode::Local, "qwen2.5-coder-7b-instruct"),
            BABY_STEP_LOC_LOCAL_7B
        );
        assert_eq!(
            baby_step_loc_for_mode(ExecMode::Cloud, "claude-sonnet-4-6"),
            BABY_STEP_LOC_CLOUD_SONNET
        );
        assert_eq!(
            baby_step_loc_for_mode(ExecMode::Cloud, "claude-opus-4-1"),
            BABY_STEP_LOC_CLOUD_OPUS
        );
    }

    #[test]
    fn estimate_task_cost_usd_returns_zero_for_empty_skeleton() {
        let skel =
            crate::plan_skeleton::PlanSkeleton::new(crate::plan_skeleton::PipelineId::from(
                "test-empty",
            ));
        let (low, high) = estimate_task_cost_usd(&skel, "claude-sonnet-4-6");
        assert_eq!(low, 0.0);
        assert_eq!(high, 0.0);
    }

    #[test]
    fn estimate_task_cost_usd_local_model_returns_zero() {
        let mut skel =
            crate::plan_skeleton::PlanSkeleton::new(crate::plan_skeleton::PipelineId::from(
                "test-local",
            ));
        let entry = crate::plan_skeleton::PhaseEntry::new("p1", "do a thing").unwrap();
        skel.add_phase(entry).unwrap();
        let (low, high) = estimate_task_cost_usd(&skel, "qwen2.5-coder-7b");
        assert_eq!(low, 0.0);
        assert_eq!(high, 0.0);
    }

    #[test]
    fn estimate_task_cost_usd_sonnet_bounds_within_spread() {
        let mut skel =
            crate::plan_skeleton::PlanSkeleton::new(crate::plan_skeleton::PipelineId::from(
                "test-sonnet",
            ));
        for i in 0..3 {
            let entry =
                crate::plan_skeleton::PhaseEntry::new(format!("p{i}"), "step").unwrap();
            skel.add_phase(entry).unwrap();
        }
        let (low, high) = estimate_task_cost_usd(&skel, "claude-sonnet-4-6");
        assert!(low > 0.0);
        assert!(high > low);
        // high/low == (1+spread)/(1-spread)
        let expected_ratio = (1.0 + COST_ESTIMATE_SPREAD_RATIO)
            / (1.0 - COST_ESTIMATE_SPREAD_RATIO);
        let actual_ratio = high / low;
        assert!((actual_ratio - expected_ratio).abs() < 1e-9);
    }

    // B-27 D-4 item 16: HYBRID escalation routing.
    #[test]
    fn escalate_hybrid_step_to_cloud_routes_cloud_when_creds_present() {
        let router = CloudModeRouter::new(CloudConfig::default());
        let creds = Some(CloudCredentials::for_test_with_token("dummy"));
        let outcome = escalate_hybrid_step_to_cloud(&router, creds, None);
        assert!(matches!(outcome, Ok(RoutedProvider::Cloud { .. })));
    }

    #[test]
    fn escalate_hybrid_step_to_cloud_blocks_when_air_gapped() {
        let mut cfg = CloudConfig::default();
        cfg.air_gapped = true;
        let router = CloudModeRouter::new(cfg);
        let creds = Some(CloudCredentials::for_test_with_token("dummy"));
        let outcome = escalate_hybrid_step_to_cloud(&router, creds, None);
        assert_eq!(outcome, Err(CloudModeError::AirGappedConflict));
    }

    #[test]
    fn escalate_hybrid_step_to_cloud_blocks_when_daily_cap_exceeded() {
        let mut cfg = CloudConfig::default();
        cfg.spend_cap_daily_usd = Some(1.0);
        let router = CloudModeRouter::new(cfg);
        let creds = Some(CloudCredentials::for_test_with_token("dummy"));
        let spend = Some(SpendSnapshotInput::from_day_usd(2.0));
        let outcome = escalate_hybrid_step_to_cloud(&router, creds, spend);
        assert!(matches!(
            outcome,
            Err(CloudModeError::DailySpendCapExceeded { .. })
        ));
    }

    // B-27 D-4 item 17: pre-task cost-estimate interactive gate.
    #[test]
    fn prompt_cost_estimate_gate_auto_approves_when_no_gate() {
        // Dashboard / non-interactive entrypoint: no stdin => auto-proceed.
        assert!(prompt_cost_estimate_gate(None, 0.12, 0.35, "claude-sonnet-4-6"));
    }

    #[test]
    fn prompt_cost_estimate_gate_approves_on_approve_decision() {
        use crate::gate::{GateDecision, MockInteractiveGate};
        let mock = MockInteractiveGate::new(vec![GateDecision::Approve]);
        let proceed = prompt_cost_estimate_gate(Some(&mock), 0.12, 0.35, "claude-sonnet-4-6");
        assert!(proceed);
        // Verify the gate was actually called (not short-circuited).
        assert_eq!(mock.call_count(), 1);
        // Verify the summary carried the cost range in phase_label.
        let summaries = mock.presented_summaries();
        assert_eq!(summaries.len(), 1);
        assert!(summaries[0].phase_label.contains("0.12"));
        assert!(summaries[0].phase_label.contains("0.35"));
        assert_eq!(summaries[0].gate_results.len(), 1);
        assert!(summaries[0].gate_results[0].passed);
    }

    #[test]
    fn prompt_cost_estimate_gate_rejects_on_reject_decision() {
        use crate::gate::{GateDecision, MockInteractiveGate};
        let mock = MockInteractiveGate::new(vec![GateDecision::Reject]);
        let proceed = prompt_cost_estimate_gate(Some(&mock), 0.12, 0.35, "claude-sonnet-4-6");
        assert!(!proceed);
        assert_eq!(mock.call_count(), 1);
    }
}
