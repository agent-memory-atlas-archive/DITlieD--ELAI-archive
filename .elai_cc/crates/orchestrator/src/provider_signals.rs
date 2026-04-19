//! Batch 1B — provider extended-signal reader.
//!
//! Unblocks MVP-wiring-audit items A-8 / A-9 / A-17 / A-25 / A-37 by threading
//! the extended fields populated by `elai_providers::SidecarProvider` (patch-
//! 1.32 plumbed the provider into `TaskRunnerConfig.llm_provider`, but as an
//! `Arc<dyn LlmProvider>` — the concrete getters for `last_search_trigger` and
//! `last_gg_signals` were never reachable from the baby-step loop).
//!
//! Design:
//! - `ProviderSignalReader` is a dyn-safe trait with async getters for each
//!   extended signal. A `None` return means "not populated yet" OR "signals
//!   globally disabled via `ELAI_DISABLE_PROVIDER_SIGNALS`"; callers must not
//!   emit SSB rows for `None`.
//! - `SidecarProviderSignalReader` is the concrete impl wrapping an
//!   `Arc<SidecarProvider>`. Gated behind the `sidecar` feature so crates that
//!   don't link the sidecar (CLI/tests) still see the trait + value types.
//! - `NullProviderSignalReader` is the always-`None` impl used by unit tests
//!   and by any task_runner caller that wants the trait surface without the
//!   concrete provider.
//!
//! Python sidecar: OFF LIMITS for this batch. The fields consumed here are
//! already plumbed on the Rust side (`sidecar_client.rs:143-167` — optional
//! deserialization with `#[serde(default)]`) so the reader degrades to `None`
//! if the upstream server predates the extended-response upgrade.

#[cfg(any(test, feature = "sidecar"))]
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use elai_core::VramTier;

// ---------------------------------------------------------------------------
// Env-var escape hatch
// ---------------------------------------------------------------------------

/// Set to `1`/`true`/`on` to force every [`ProviderSignalReader`] method to
/// return `None`. Mirrors the `ELAI_HOOKS_DISABLED` pattern introduced in
/// Batch 1A — the task_runner treats an all-`None` reader as "no signals".
pub const ELAI_DISABLE_PROVIDER_SIGNALS_ENV: &str = "ELAI_DISABLE_PROVIDER_SIGNALS";

/// Returns `true` when the env-var escape hatch is engaged.
#[must_use]
pub fn provider_signals_disabled_via_env() -> bool {
    std::env::var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV)
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Value types (distinct from provider crate to avoid leaking sidecar-specific
// types into the orchestrator public API — mirrors the Batch 1A HookFinding
// vs HookVerdict split).
// ---------------------------------------------------------------------------

/// A-17: Generative-Grounded intrinsic verification signals read back from the
/// most recent sidecar completion at zero additional inference cost. All four
/// fields are `None` on providers that predate the extended-response upgrade.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GgSignals {
    pub mean_logprob: Option<f64>,
    pub min_logprob: Option<f64>,
    pub entropy_mean: Option<f64>,
    pub entropy_max: Option<f64>,
}

impl GgSignals {
    /// `true` when every field is `None` — lets the task_runner skip SSB
    /// emission without a dedicated guard.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mean_logprob.is_none()
            && self.min_logprob.is_none()
            && self.entropy_mean.is_none()
            && self.entropy_max.is_none()
    }
}

/// A-25: Cosine-similarity-based novelty score between the current generation
/// and the previous generation for the same prompt. Populated by the sidecar
/// only after the second generation for a given prompt — `None` on the first
/// attempt. Value range: `0.0` = identical to previous, `1.0` = fully novel.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NoveltySignal {
    pub cosine_distance: Option<f64>,
}

/// A-9: ETC (Entropy-Trend Constraint) monitoring snapshot. The `threshold_hit`
/// field is `Some(reason)` only when the sidecar emitted a search_trigger
/// payload on the last completion. `disabled` mirrors the default-off stance
/// per MVP audit item A-39 (ETC stays infinity until calibration lands).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EtcState {
    pub threshold_hit: Option<String>,
    pub disabled: bool,
}

/// A-9: Search-trigger payload lifted from the sidecar's `search_trigger`
/// extended-response field. Distinct from `elai_providers::SearchTriggerPayload`
/// so orchestrator tests / SSB consumers don't need the `sidecar` feature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchTrigger {
    /// Opaque reason emitted by the sidecar (e.g. `"import"`, `"low_confidence"`).
    pub reason: String,
    /// Symbolic threshold-hit label for dashboard rendering.
    pub threshold_hit: String,
}

/// A-8: Steering-vector availability for the active project. The `reason`
/// field documents why a vector is or is not present (e.g.
/// `"modern_api_preference_missing"`). The actual vector computation runs on
/// the Python sidecar; the Rust side only knows the boolean + reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteeringVectorStatus {
    pub active: bool,
    pub reason: String,
}

/// A-37: Per-machine VRAM tier detected at task_runner startup. Source field
/// distinguishes NVML readout, env-var override, and default-fallback so the
/// dashboard can show provenance.
///
/// Intentionally does NOT derive `Serialize`/`Deserialize`: `elai_core::VramTier`
/// is serde-free by design (the core crate keeps zero serde deps). The
/// dashboard renders this via the string-only [`crate::ssb::VramTierDetectedPayload`]
/// which is fully serde-capable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VramTierConfig {
    pub tier: VramTier,
    pub source: VramDetectionSource,
    pub free_mb: Option<u32>,
}

/// Where the VRAM tier came from. Matches the ordered fallback chain used by
/// `elai_providers::detect_vram_tier`: env override > NVML > default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VramDetectionSource {
    /// `elai_providers::detect_vram_tier` read NVML successfully (requires
    /// `providers` crate with `vllm` feature + an NVIDIA driver).
    Nvml,
    /// `ELAI_VRAM_OVERRIDE` env var provided an explicit tier.
    Env,
    /// No NVML and no env var — conservative `Vram8Gb` fallback.
    Default,
}

impl VramDetectionSource {
    /// Stable string form surfaced on SSB rows.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nvml => "nvml",
            Self::Env => "env",
            Self::Default => "default",
        }
    }
}

impl VramTierConfig {
    /// Detect the current VRAM tier via the providers crate's detection chain,
    /// preserving the source so the dashboard can render provenance.
    ///
    /// Detection is side-effect-free for the dashboard path: the providers
    /// crate's `detect_vram_tier` already logs via `tracing`, we don't re-log.
    ///
    /// On any detection error we return a `Default`-source fallback at
    /// `Vram8Gb` rather than propagating — a task_runner that fails to
    /// detect VRAM still has to run, and the dashboard must always have a
    /// VramTierDetected row.
    #[must_use]
    pub fn detect() -> Self {
        // Env override takes precedence in `detect_vram_tier`; mirror that
        // precedence here so the source label matches.
        let env_set = std::env::var("ELAI_VRAM_OVERRIDE").is_ok();
        match elai_providers::detect_vram_tier() {
            Ok(tier) => {
                let source = if env_set {
                    VramDetectionSource::Env
                } else {
                    // `detect_vram_tier` returns Vram8Gb on NVML failure OR
                    // on a sub-10 GiB GPU. We can't distinguish here — treat
                    // Vram8Gb without an env override as `Default` so the
                    // dashboard doesn't falsely advertise NVML.
                    match tier {
                        VramTier::Vram8Gb => VramDetectionSource::Default,
                        _ => VramDetectionSource::Nvml,
                    }
                };
                Self {
                    tier,
                    source,
                    free_mb: None,
                }
            }
            Err(_) => Self {
                tier: VramTier::Vram8Gb,
                source: VramDetectionSource::Default,
                free_mb: None,
            },
        }
    }

    /// Human-readable tier label (`"8GB"`, `"12GB"`, ...) matching the dashboard
    /// style guide (Part 2 §2.5).
    #[must_use]
    pub fn tier_label(&self) -> &'static str {
        match self.tier {
            VramTier::Vram8Gb => "8GB",
            VramTier::Vram12Gb => "12GB",
            VramTier::Vram16Gb => "16GB",
            VramTier::Vram24Gb => "24GB",
            VramTier::Vram48Gb => "48GB",
            VramTier::Cloud => "cloud",
        }
    }
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Dyn-safe reader over the extended-signal surface exposed by sidecar-backed
/// providers. Implementations MUST be `Send + Sync` so the task_runner can
/// share an `Arc<dyn ProviderSignalReader>` across the baby-step loop.
///
/// All methods return `Option<T>` so a sidecar that predates the extended
/// response, or a provider that has never received a completion request, is
/// treated as "no signal" rather than as an error.
pub trait ProviderSignalReader: Send + Sync {
    /// A-9: last search_trigger payload from the sidecar (if ETC fired).
    fn read_last_search_trigger(&self) -> Option<SearchTrigger>;

    /// A-17: last GG intrinsic-verification signals (logprob + entropy bundle).
    fn read_last_gg_signals(&self) -> Option<GgSignals>;

    /// A-25: last generation-novelty score.
    fn read_last_novelty_signal(&self) -> Option<NoveltySignal>;

    /// A-37: machine-wide VRAM tier configuration (static across a run).
    fn read_vram_tier_config(&self) -> Option<VramTierConfig>;

    /// A-8: steering-vector availability status for the active project.
    fn read_steering_vector_status(&self) -> Option<SteeringVectorStatus>;

    /// A-9: ETC monitoring snapshot (threshold state, disabled bit).
    fn read_etc_state(&self) -> Option<EtcState>;
}

// ---------------------------------------------------------------------------
// Null implementation (no sidecar / disabled) — always returns None
// ---------------------------------------------------------------------------

/// Always-`None` implementation. Used by CLI-only paths that want the trait
/// surface without a live sidecar, and by every test that wants to exercise
/// the task_runner's "no signals" code path.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullProviderSignalReader;

impl ProviderSignalReader for NullProviderSignalReader {
    fn read_last_search_trigger(&self) -> Option<SearchTrigger> {
        None
    }
    fn read_last_gg_signals(&self) -> Option<GgSignals> {
        None
    }
    fn read_last_novelty_signal(&self) -> Option<NoveltySignal> {
        None
    }
    fn read_vram_tier_config(&self) -> Option<VramTierConfig> {
        None
    }
    fn read_steering_vector_status(&self) -> Option<SteeringVectorStatus> {
        None
    }
    fn read_etc_state(&self) -> Option<EtcState> {
        None
    }
}

// ---------------------------------------------------------------------------
// SidecarProviderSignalReader (concrete impl, gated on the sidecar feature)
// ---------------------------------------------------------------------------

/// Concrete reader wrapping an `Arc<SidecarProvider>`. Delegates to the
/// provider's public getters (`last_search_trigger` / `last_gg_signals` —
/// sidecar_client.rs:311-326) and populates the remaining signals from
/// static / environmental inputs (VRAM tier is detected once at construction;
/// steering + ETC state are stubs awaiting sidecar-side plumbing).
///
/// The env-var escape hatch (`ELAI_DISABLE_PROVIDER_SIGNALS`) is checked on
/// every read so operators can flip signals off at runtime without restarting
/// the task runner.
#[cfg(feature = "sidecar")]
pub struct SidecarProviderSignalReader {
    provider: Arc<elai_providers::SidecarProvider>,
    vram_tier: VramTierConfig,
    steering_status: SteeringVectorStatus,
    etc_disabled: bool,
}

#[cfg(feature = "sidecar")]
impl SidecarProviderSignalReader {
    /// Construct with a live sidecar provider. VRAM tier is detected once
    /// here so subsequent reads are O(1).
    ///
    /// `etc_disabled` is sourced from the supplied flag (the dashboard reads
    /// it off `SidecarConfig::etc_threshold == INFINITY`). `steering_status`
    /// defaults to "inactive, no vector installed" — real wiring follows when
    /// the sidecar exposes a projectwise steering-vector registry.
    #[must_use]
    pub fn new(
        provider: Arc<elai_providers::SidecarProvider>,
        etc_disabled: bool,
    ) -> Self {
        Self {
            provider,
            vram_tier: VramTierConfig::detect(),
            steering_status: SteeringVectorStatus {
                active: false,
                reason: "modern_api_preference_vector_unavailable".to_string(),
            },
            etc_disabled,
        }
    }

    /// Override the steering-vector status. Called by the dashboard boot path
    /// once a real vector registry is plumbed (outside Batch 1B scope).
    pub fn set_steering_status(&mut self, status: SteeringVectorStatus) {
        self.steering_status = status;
    }

    /// Batch-ful-c (A-8) — refresh `steering_status` from the on-disk cache
    /// at `<project_root>/.elai_cc/steering/<project_id>/`.
    ///
    /// Uses `SteeringVectorEngine::scan_steering_cache_dir` so the status
    /// semantics match everything else in the batch. Returns the new status
    /// (for logging / dashboard push) alongside updating the internal field.
    /// Never panics: on scan errors falls back to a stubbed inactive status.
    pub fn refresh_steering_status_from_cache(
        &mut self,
        project_root: &std::path::Path,
        project_id: &str,
    ) -> SteeringVectorStatus {
        match elai_providers::layer_config::steering::SteeringVectorEngine::scan_steering_cache_dir(
            project_root,
            project_id,
        ) {
            Ok(scanned) => {
                let status = SteeringVectorStatus {
                    active: scanned.active,
                    reason: scanned.reason,
                };
                self.steering_status = status.clone();
                status
            }
            Err(e) => {
                tracing::warn!(error = %e, project_id, "steering cache scan failed");
                let status = SteeringVectorStatus {
                    active: false,
                    reason: format!(
                        "modern_api_preference_vector_unavailable (scan_error: {e})"
                    ),
                };
                self.steering_status = status.clone();
                status
            }
        }
    }
}

#[cfg(feature = "sidecar")]
impl ProviderSignalReader for SidecarProviderSignalReader {
    fn read_last_search_trigger(&self) -> Option<SearchTrigger> {
        if provider_signals_disabled_via_env() {
            return None;
        }
        self.provider.last_search_trigger().map(|p| SearchTrigger {
            reason: p.detected_context.clone(),
            threshold_hit: format!("trigger_token_idx={}", p.trigger_token_idx),
        })
    }

    fn read_last_gg_signals(&self) -> Option<GgSignals> {
        if provider_signals_disabled_via_env() {
            return None;
        }
        let g = self.provider.last_gg_signals();
        let out = GgSignals {
            mean_logprob: g.mean_logprob,
            min_logprob: g.min_logprob,
            entropy_mean: g.entropy_mean,
            entropy_max: g.entropy_max,
        };
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn read_last_novelty_signal(&self) -> Option<NoveltySignal> {
        if provider_signals_disabled_via_env() {
            return None;
        }
        let g = self.provider.last_gg_signals();
        g.generation_novelty.map(|d| NoveltySignal {
            cosine_distance: Some(d),
        })
    }

    fn read_vram_tier_config(&self) -> Option<VramTierConfig> {
        if provider_signals_disabled_via_env() {
            return None;
        }
        Some(self.vram_tier.clone())
    }

    fn read_steering_vector_status(&self) -> Option<SteeringVectorStatus> {
        if provider_signals_disabled_via_env() {
            return None;
        }
        Some(self.steering_status.clone())
    }

    fn read_etc_state(&self) -> Option<EtcState> {
        if provider_signals_disabled_via_env() {
            return None;
        }
        let threshold_hit = self
            .provider
            .last_search_trigger()
            .map(|p| p.detected_context.clone());
        Some(EtcState {
            threshold_hit,
            disabled: self.etc_disabled,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_reader_returns_none_for_all_methods() {
        let r: Arc<dyn ProviderSignalReader> = Arc::new(NullProviderSignalReader);
        assert!(r.read_last_search_trigger().is_none());
        assert!(r.read_last_gg_signals().is_none());
        assert!(r.read_last_novelty_signal().is_none());
        assert!(r.read_vram_tier_config().is_none());
        assert!(r.read_steering_vector_status().is_none());
        assert!(r.read_etc_state().is_none());
    }

    #[test]
    fn gg_signals_is_empty_tracks_all_fields() {
        let g = GgSignals::default();
        assert!(g.is_empty());
        let g = GgSignals {
            mean_logprob: Some(-1.0),
            ..Default::default()
        };
        assert!(!g.is_empty());
    }

    #[test]
    fn vram_detection_source_as_str_is_stable() {
        assert_eq!(VramDetectionSource::Nvml.as_str(), "nvml");
        assert_eq!(VramDetectionSource::Env.as_str(), "env");
        assert_eq!(VramDetectionSource::Default.as_str(), "default");
    }

    #[test]
    fn vram_tier_detect_returns_some_tier() {
        // Contract: VramTierConfig::detect() never panics and always returns a
        // tier classification. The exact tier depends on the test host (will
        // usually be `Default`/`Vram8Gb` in CI), so we only assert the public
        // invariants — tier label present, source label present.
        let cfg = VramTierConfig::detect();
        assert!(!cfg.tier_label().is_empty());
        assert!(!cfg.source.as_str().is_empty());
    }

    #[test]
    fn env_var_disable_flag() {
        // Env-var scope is process-global; snapshot + restore so parallel
        // tests do not interfere.
        let prev = std::env::var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV).ok();
        std::env::set_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV, "1");
        assert!(provider_signals_disabled_via_env());
        std::env::set_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV, "0");
        assert!(!provider_signals_disabled_via_env());
        match prev {
            Some(v) => std::env::set_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV, v),
            None => std::env::remove_var(ELAI_DISABLE_PROVIDER_SIGNALS_ENV),
        }
    }
}
