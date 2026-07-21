//! H-50: Regime-aware in-session context management.
//!
//! Provides a single `effective_context_window` truth and a `ContextPolicy`
//! enum that selects the correct trim threshold and ladder behaviour for the
//! current backend:
//!
//! * [`ContextPolicy::Local`] -- window = `ELAI_LOCAL_MAX_CONTEXT` env var
//!   (or 32 000 default), trim at `TRIM_FRACTION` (~0.75).  Runs the
//!   compaction ladder in the interactive REPL turn; does NOT auto-recover all
//!   stubs (selective-recover only).  J-36 frozen-head invariant: the
//!   sys+rules+anchor head (seq <= `FROZEN_HEAD_SEQ_BOUNDARY`) is never
//!   trimmed, reordered, or unloaded; only the task tail is minimized.  Local
//!   activates prefix-KV reuse when eligible (see `prefix_kv_eligible`).
//!
//! * [`ContextPolicy::Cloud`] -- window = 200 000 (Anthropic cap), trim at
//!   `CLOUD_TRIM_FRACTION` (0.85 = `SESSION_HANDOFF_THRESHOLD`).  Leaves the
//!   cache-stable prefix untouched.  Honors the frozen-head invariant.
//!
//! Kill-switch: `ELAI_CONTEXT_POLICY_DISABLE=1` returns the flat legacy
//! behaviour from any `trim_threshold` call (falls back to the pre-H-50
//! 100 000-token flat threshold for `maybe_auto_compact` consumers).
//!
//! The previous flat constant
//! `DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD = 100_000` in
//! `conversation.rs` is replaced by `trim_threshold(regime_policy(mode),
//! effective_context_window(mode))`.
//!
//! Wire-in points (production, non-test):
//!   * `conversation.rs:789` `maybe_auto_compact` -- reads the threshold via
//!     `ConversationRuntime::auto_compaction_input_tokens_threshold` which is
//!     now set from `trim_threshold(policy, window)` at construction time.
//!   * `live_cli.rs:1185` `prepare_turn_runtime` -- replaces
//!     `max_tokens_for_model(&self.model)` with
//!     `effective_context_window_for_backend(backend_mode)`.

use crate::context_budget::ModelCapabilityProfile;
use crate::dispatcher::BackendMode;

// ---------------------------------------------------------------------------
// Environment variable names
// ---------------------------------------------------------------------------

/// Kill-switch: set to `"1"` to disable the H-50 regime-aware policy and fall
/// back to the old flat-100 000-token behaviour.  Default: POLICY ACTIVE.
pub const ELAI_CONTEXT_POLICY_DISABLE_VAR: &str = "ELAI_CONTEXT_POLICY_DISABLE";

/// Local window upper bound read from this env var (same var as in
/// `runtime_client.rs:391`).  Falls back to [`LOCAL_DEFAULT_WINDOW`] when not
/// set.
pub const ELAI_LOCAL_MAX_CONTEXT_VAR: &str = "ELAI_LOCAL_MAX_CONTEXT";

/// Fallback local window when `ELAI_LOCAL_MAX_CONTEXT` is not set.
pub const LOCAL_DEFAULT_WINDOW: u32 = 32_000;

/// Fallback cloud / API window when the model lookup has no specific value.
pub const CLOUD_DEFAULT_WINDOW: u32 = 200_000;

/// Legacy flat threshold returned when the kill-switch is ON.  Matches the
/// old `DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD` in `conversation.rs`.
pub const LEGACY_FLAT_THRESHOLD: u32 = 100_000;

// ---------------------------------------------------------------------------
// Trim fractions
// ---------------------------------------------------------------------------

/// Local trim fraction: fire the ladder at ~75 % of the real window.
/// Conservative (prefer avoiding false-drops over maximum headroom); GD-4
/// calibration on pythondev required before tightening toward 0.70.
pub const LOCAL_TRIM_FRACTION: f64 = 0.75;

/// Cloud trim fraction: preserves the existing `SESSION_HANDOFF_THRESHOLD`
/// semantics (0.85) so the cache-stable prefix is never perturbed by this
/// change.
pub const CLOUD_TRIM_FRACTION: f64 = 0.85;

/// J-36 frozen-head seq ceiling. MUST equal
/// `session_dag::PROMPT_CACHE_PREFIX_BOUNDARY` (3).  Messages with
/// seq <= this value are the frozen head (sys+rules+anchor head); never
/// stub/drop them.
pub const FROZEN_HEAD_SEQ_BOUNDARY: u32 = 3;

/// J-36: both regimes honor the frozen-head invariant.  Local newly does
/// (this unit); Cloud already kept the cache-stable prefix intact.
#[must_use]
pub fn honors_frozen_head(policy: ContextPolicy) -> bool {
    match policy {
        ContextPolicy::Local | ContextPolicy::Cloud => true,
    }
}

/// Leading System messages that are NOT an existing compact-summary
/// preamble form the frozen head on the ConversationMessage path
/// (compact.rs).  Pure count helper for the compact PRODUCTION_HOOK.
#[must_use]
pub fn frozen_head_system_prefix_len(
    is_system_at: impl Fn(usize) -> bool,
    is_compact_summary_at: impl Fn(usize) -> bool,
    len: usize,
) -> usize {
    let mut n = 0usize;
    while n < len && is_system_at(n) && !is_compact_summary_at(n) {
        n += 1;
    }
    n
}

/// J-25 T4: default output-token cap subtracted from the effective window to
/// derive the usable input budget. Option (b) from S1 §5: a const rather than
/// threading `RuntimeCapabilities.max_output` to the tick site, keeping the
/// `maybe_auto_compact` signature stable.
pub const DEFAULT_MAX_OUTPUT_CAP: u32 = 20_000;

// ---------------------------------------------------------------------------
// ContextPolicy
// ---------------------------------------------------------------------------

/// Which context-management regime applies for this session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPolicy {
    /// Local model (Qwen sidecar): minimise re-sent tokens, window typically
    /// 25 000-32 000 tokens.  J-36 activates local prefix-KV reuse when
    /// eligible; the frozen head (sys+rules+anchor head) is never
    /// trimmed/reordered/unloaded, only the task tail is minimized.
    Local,
    /// Cloud / API model (Sonnet etc.): keep the cache-stable prefix intact,
    /// use the large Anthropic context window.  Honors the frozen-head
    /// invariant.
    Cloud,
}

/// Derive the policy from the active [`BackendMode`].
///
/// `Hybrid` is treated as `Cloud` because the cloud call is the expensive leg
/// and the cache-stable-prefix benefit applies.
#[must_use]
pub fn regime_policy(mode: BackendMode) -> ContextPolicy {
    match mode {
        BackendMode::Local => ContextPolicy::Local,
        BackendMode::Api | BackendMode::Hybrid => ContextPolicy::Cloud,
    }
}

// ---------------------------------------------------------------------------
// J-25 T4 — PressureLevel (graded pressure ladder)
// ---------------------------------------------------------------------------

/// Graded pressure on the usable context budget, consumed by
/// `ConversationRuntime::maybe_auto_compact` to decide whether the
/// compaction ladder should fire. Lower grades short-circuit the ladder;
/// higher grades let it run.
///
/// Boundaries (ratio = `used / usable_context_tokens(window)`):
///   * `Normal`   -- ratio < 0.50
///   * `Moderate` -- 0.50 <= ratio < 0.70
///   * `High`     -- 0.70 <= ratio < 0.85
///   * `Critical` -- ratio >= 0.85
///
/// When the kill-switch `ELAI_CONTEXT_POLICY_DISABLE=1` is active the grade
/// collapses to the flat-legacy behaviour: `Critical` when
/// `used >= LEGACY_FLAT_THRESHOLD`, else `Normal`. This mirrors the
/// `trim_threshold` kill-switch path so the existing flat gate stays intact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PressureLevel {
    Normal = 0,
    Moderate = 1,
    High = 2,
    Critical = 3,
}

impl From<PressureLevel> for u8 {
    fn from(level: PressureLevel) -> Self {
        level as u8
    }
}

/// Usable input-token budget for grading: `window - min(DEFAULT_MAX_OUTPUT_CAP,
/// max_output_cap)`. With `max_output_cap = DEFAULT_MAX_OUTPUT_CAP` this is
/// `window.saturating_sub(DEFAULT_MAX_OUTPUT_CAP)` (option (b) from S1 §5;
/// `RuntimeCapabilities.max_output` is NOT threaded to the tick site).
#[must_use]
pub fn usable_context_tokens(window: u32) -> u32 {
    let max_output_cap = DEFAULT_MAX_OUTPUT_CAP;
    window.saturating_sub(max_output_cap.min(DEFAULT_MAX_OUTPUT_CAP))
}

/// Grade the current context pressure from `used` input tokens against a
/// `window`-sized effective context window under `policy`. Pure + deterministic
/// (ABORT-2: no LLM, no I/O).
///
/// Kill-switch path: when `context_policy_disabled()` is true, the grade
/// collapses to the flat-legacy behaviour (`Critical` if
/// `used >= LEGACY_FLAT_THRESHOLD`, else `Normal`) so the existing
/// `auto_compaction_input_tokens_threshold` flat gate keeps working.
#[must_use]
pub fn pressure_level(used: u32, window: u32, _policy: ContextPolicy) -> PressureLevel {
    if context_policy_disabled() {
        return if used >= LEGACY_FLAT_THRESHOLD {
            PressureLevel::Critical
        } else {
            PressureLevel::Normal
        };
    }
    let usable = usable_context_tokens(window);
    if usable == 0 {
        return PressureLevel::Critical;
    }
    #[allow(clippy::cast_precision_loss)]
    let ratio = f64::from(used) / f64::from(usable);
    if ratio < 0.50 {
        PressureLevel::Normal
    } else if ratio < 0.70 {
        PressureLevel::Moderate
    } else if ratio < 0.85 {
        PressureLevel::High
    } else {
        PressureLevel::Critical
    }
}

// ---------------------------------------------------------------------------
// Window truth
// ---------------------------------------------------------------------------

/// Returns the real effective context window for the given backend mode.
///
/// * `Local` => reads `ELAI_LOCAL_MAX_CONTEXT` env var (u32, same read as
///   `runtime_client.rs:391`).  Falls back to [`LOCAL_DEFAULT_WINDOW`].
/// * `Api | Hybrid` => [`CLOUD_DEFAULT_WINDOW`] (200 000).
///
/// This is the **single source of truth** replacing both the flat 100 000
/// constant in `conversation.rs` and the `max_tokens_for_model` call in
/// `live_cli.rs:1185`.
#[must_use]
pub fn effective_context_window(mode: BackendMode) -> u32 {
    match mode {
        BackendMode::Local => {
            std::env::var(ELAI_LOCAL_MAX_CONTEXT_VAR)
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(LOCAL_DEFAULT_WINDOW)
        }
        BackendMode::Api | BackendMode::Hybrid => CLOUD_DEFAULT_WINDOW,
    }
}

// ---------------------------------------------------------------------------
// Trim threshold
// ---------------------------------------------------------------------------

/// Returns the token count at which the interactive REPL should fire the
/// compaction ladder / handoff logic.
///
/// When `ELAI_CONTEXT_POLICY_DISABLE=1`, returns [`LEGACY_FLAT_THRESHOLD`]
/// regardless of policy or window, preserving the pre-H-50 behaviour exactly.
///
/// Otherwise:
/// * `Local`  => `floor(window * LOCAL_TRIM_FRACTION)`
/// * `Cloud`  => `floor(window * CLOUD_TRIM_FRACTION)`
#[must_use]
pub fn trim_threshold(policy: ContextPolicy, window: u32) -> u32 {
    trim_threshold_with_calibration(policy, window, None)
}

/// J-34 leg-2 seam — calibrated preference for `trim_threshold`.
///
/// Kill-switch first (same as `trim_threshold`): when
/// `ELAI_CONTEXT_POLICY_DISABLE=1`, returns [`LEGACY_FLAT_THRESHOLD`] without
/// consulting `calibrated_trim_fraction`.
///
/// When `calibrated_trim_fraction` is `Some(f)` and `f` is in `(0.0, 1.0]`,
/// returns `floor(window * f)` — the measured knee overrides the folklore
/// fraction. When `calibrated_trim_fraction` is `None` or out of range, the
/// folklore fraction (`LOCAL_TRIM_FRACTION` / `CLOUD_TRIM_FRACTION`) is used
/// exactly as in `trim_threshold`, so H-50 contract tests stay green.
#[must_use]
pub fn trim_threshold_with_calibration(
    policy: ContextPolicy,
    window: u32,
    calibrated_trim_fraction: Option<f64>,
) -> u32 {
    if context_policy_disabled() {
        return LEGACY_FLAT_THRESHOLD;
    }
    if let Some(f) = calibrated_trim_fraction {
        if f > 0.0 && f <= 1.0 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let result = (f64::from(window) * f) as u32;
            return result;
        }
    }
    let fraction = match policy {
        ContextPolicy::Local => LOCAL_TRIM_FRACTION,
        ContextPolicy::Cloud => CLOUD_TRIM_FRACTION,
    };
    // Saturating cast: window * fraction always fits in u32 for sane window sizes.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let result = (f64::from(window) * fraction) as u32;
    result
}

/// Convenience wrapper: derive policy + window from `BackendMode`, return
/// threshold in one call.  Used by `ConversationRuntime::new_with_features`
/// and `live_cli.rs prepare_turn_runtime`.
#[must_use]
pub fn trim_threshold_for_mode(mode: BackendMode) -> u32 {
    trim_threshold_for_mode_with_profile(mode, None)
}

/// J-34 leg-2 seam — calibrated preference wrapper. Derives policy + window
/// from `BackendMode`, then threads `profile.calibrated_trim_fraction` (when
/// `Some`) into [`trim_threshold_with_calibration`]. When `profile` is `None`
/// or its `calibrated_trim_fraction` is `None`, behaviour is identical to
/// [`trim_threshold_for_mode`] (folklore fraction).
#[must_use]
pub fn trim_threshold_for_mode_with_profile(
    mode: BackendMode,
    profile: Option<&ModelCapabilityProfile>,
) -> u32 {
    let policy = regime_policy(mode);
    let window = effective_context_window(mode);
    let calibrated = profile.and_then(|p| p.calibrated_trim_fraction);
    trim_threshold_with_calibration(policy, window, calibrated)
}

// ---------------------------------------------------------------------------
// Kill-switch helper
// ---------------------------------------------------------------------------

/// Returns `true` when `ELAI_CONTEXT_POLICY_DISABLE=1` (kill-switch ON,
/// revert to flat legacy behaviour).  Any other value (including unset) =>
/// policy active (kill-switch OFF).
#[must_use]
pub fn context_policy_disabled() -> bool {
    std::env::var(ELAI_CONTEXT_POLICY_DISABLE_VAR)
        .map(|v| v == "1")
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Serialize all env-mutating tests to prevent race conditions from
    // parallel test runners reading stale env state.  Using a plain
    // std::sync::Mutex so no external dep is required (serial_test not
    // in dev-deps; BLOCK-5 fix).
    static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    // J-34 MR-2: centralize the env-mutating unsafe behind test helpers so the
    // diff against tip adds zero net `unsafe {` blocks (each helper owns one
    // unsafe block, replacing many scattered call sites).
    fn clear_context_policy_disable() {
        unsafe { std::env::remove_var(ELAI_CONTEXT_POLICY_DISABLE_VAR) };
    }

    fn enable_context_policy_disable() {
        unsafe { std::env::set_var(ELAI_CONTEXT_POLICY_DISABLE_VAR, "1") };
    }

    fn clear_local_max_context() {
        unsafe { std::env::remove_var(ELAI_LOCAL_MAX_CONTEXT_VAR) };
    }

    fn set_local_max_context(value: &str) {
        unsafe { std::env::set_var(ELAI_LOCAL_MAX_CONTEXT_VAR, value) };
    }

    #[test]
    fn local_trim_threshold_respects_env() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        // D1: effective_context_window(Local) reads ELAI_LOCAL_MAX_CONTEXT.
        // Safety: test-only env mutation, #[cfg(test)] gate.
        // Explicitly clear the kill-switch so this test is order-independent
        // with respect to kill_switch_returns_legacy_flat (both mutate env).
        clear_context_policy_disable();
        set_local_max_context("32000");
        let window = effective_context_window(BackendMode::Local);
        let threshold = trim_threshold(ContextPolicy::Local, 32_000);
        clear_local_max_context();
        assert_eq!(window, 32_000, "window must match ELAI_LOCAL_MAX_CONTEXT=32000");
        // D2: trim_threshold(Local, 32000) = floor(32000 * 0.75) = 24000.
        assert_eq!(threshold, 24_000, "trim threshold must be floor(32000 * 0.75) = 24000");
    }

    #[test]
    fn cloud_trim_threshold_uses_cloud_fraction() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        // Ensure kill-switch is OFF so we get the real threshold.
        clear_context_policy_disable();
        let threshold = trim_threshold(ContextPolicy::Cloud, 200_000);
        assert_eq!(threshold, 170_000, "cloud threshold must be floor(200000 * 0.85)");
    }

    #[test]
    fn kill_switch_returns_legacy_flat() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        enable_context_policy_disable();
        let threshold = trim_threshold_for_mode(BackendMode::Local);
        clear_context_policy_disable();
        assert_eq!(threshold, LEGACY_FLAT_THRESHOLD, "kill-switch must return legacy threshold");
    }

    #[test]
    fn regime_policy_local_is_local() {
        assert_eq!(regime_policy(BackendMode::Local), ContextPolicy::Local);
    }

    #[test]
    fn regime_policy_api_is_cloud() {
        assert_eq!(regime_policy(BackendMode::Api), ContextPolicy::Cloud);
    }

    #[test]
    fn regime_policy_hybrid_is_cloud() {
        assert_eq!(regime_policy(BackendMode::Hybrid), ContextPolicy::Cloud);
    }

    #[test]
    fn default_window_when_env_missing() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_local_max_context();
        let window = effective_context_window(BackendMode::Local);
        assert_eq!(window, LOCAL_DEFAULT_WINDOW);
    }

    // J-34 slice-3 — calibrated preference tests.

    #[test]
    fn calibrated_trim_fraction_overrides_folklore() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_context_policy_disable();
        // calibrated 0.50 wins over cloud folklore 0.85.
        let t = trim_threshold_with_calibration(
            ContextPolicy::Cloud,
            200_000,
            Some(0.50),
        );
        assert_eq!(t, 100_000, "calibrated 0.50 must override folklore 0.85");
    }

    #[test]
    fn calibrated_none_falls_back_to_folklore() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_context_policy_disable();
        let t = trim_threshold_with_calibration(
            ContextPolicy::Cloud,
            200_000,
            None,
        );
        assert_eq!(t, 170_000, "None calibration must match folklore 0.85");
    }

    #[test]
    fn calibrated_out_of_range_falls_back_to_folklore() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_context_policy_disable();
        // 0.0 and 1.5 are out of (0.0, 1.0]; folklore must win.
        let t_zero = trim_threshold_with_calibration(
            ContextPolicy::Cloud,
            200_000,
            Some(0.0),
        );
        assert_eq!(t_zero, 170_000, "0.0 is out of range, folklore wins");
        let t_high = trim_threshold_with_calibration(
            ContextPolicy::Cloud,
            200_000,
            Some(1.5),
        );
        assert_eq!(t_high, 170_000, "1.5 is out of range, folklore wins");
    }

    #[test]
    fn calibrated_one_yields_full_window() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_context_policy_disable();
        let t = trim_threshold_with_calibration(
            ContextPolicy::Local,
            32_000,
            Some(1.0),
        );
        assert_eq!(t, 32_000, "1.0 means trim at full window");
    }

    #[test]
    fn kill_switch_overrides_calibrated_fraction() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        enable_context_policy_disable();
        let t = trim_threshold_with_calibration(
            ContextPolicy::Cloud,
            200_000,
            Some(0.50),
        );
        clear_context_policy_disable();
        assert_eq!(
            t, LEGACY_FLAT_THRESHOLD,
            "kill-switch must override calibrated fraction"
        );
    }

    #[test]
    fn for_mode_with_profile_threads_calibrated_fraction() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_context_policy_disable();
        // Build a profile with calibrated_trim_fraction = 0.50.
        let mut profile = crate::context_budget::ModelCapabilityProfile::cloud_200k();
        profile.calibrated_trim_fraction = Some(0.50);
        let t = trim_threshold_for_mode_with_profile(
            BackendMode::Api,
            Some(&profile),
        );
        assert_eq!(t, 100_000, "profile 0.50 must thread through to threshold");
        // None profile matches folklore.
        let t_none = trim_threshold_for_mode_with_profile(
            BackendMode::Api,
            None,
        );
        assert_eq!(t_none, 170_000, "None profile = folklore 0.85");
    }

    // J-36 slice-5 — frozen-head invariant tests.

    #[test]
    fn honors_frozen_head_both_regimes() {
        assert!(honors_frozen_head(ContextPolicy::Local));
        assert!(honors_frozen_head(ContextPolicy::Cloud));
    }

    #[test]
    fn frozen_head_seq_boundary_is_three() {
        assert_eq!(FROZEN_HEAD_SEQ_BOUNDARY, 3);
    }

    #[test]
    fn frozen_head_system_prefix_len_counts_leading_non_summary_systems() {
        // 3 leading System rows (indices 0-2), then non-System at 3.
        let n = frozen_head_system_prefix_len(
            |i| i < 3,
            |_| false,
            5,
        );
        assert_eq!(n, 3, "3 leading non-summary System rows");

        // First System row IS a compact summary => frozen head is 0.
        let n = frozen_head_system_prefix_len(
            |_| true,
            |i| i == 0,
            5,
        );
        assert_eq!(n, 0, "summary at index 0 => frozen head is 0");

        // Mixed: 2 System then a User at index 2 => stops at 2.
        let n = frozen_head_system_prefix_len(
            |i| i < 2,
            |_| false,
            5,
        );
        assert_eq!(n, 2, "stops at first non-System");

        // Empty => 0.
        let n = frozen_head_system_prefix_len(|_| true, |_| false, 0);
        assert_eq!(n, 0, "empty => 0");
    }
}
