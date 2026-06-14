//! I-24 — isomorphic canary anti-gaming (scorer-side bias detection).
//!
//! Injects AST-preserving isomorphic code pairs into the i-17 L2 scoring path.
//! A scorer whose rating diverges across semantically-identical variants is
//! detected as style-biased and (after [`CONSECUTIVE_NEUTRALIZE_THRESHOLD`]
//! consecutive detections) its weight is zeroed for the run.
//!
//! # Design
//!
//! * **Frozen fixture set (R8)**: canary pairs are authored once and stored in
//!   `fixtures/canary_pairs.json` (versioned, listed in `gate-manifest.txt`).
//!   The running system NEVER generates new canaries from the model under test.
//! * **Probabilistic injection**: each L2 invocation samples with probability
//!   `p` (from [`CanaryInjectorConfig::injection_probability`], deterministic
//!   under `ELAI_SEED`). On miss, the L2 scorer runs normally.
//! * **Bias threshold (ID-5)**: read from `metrics/canary_bias_threshold.toml`;
//!   ELAI-measured on known-unbiased scorers, never a paper-derived constant.
//! * **ID-4 compliance**: `ScorerNeutralized` zeroes the scorer's weight for
//!   the run; it CANNOT override an L1 HardVeto (L1 is structurally prior to
//!   L2 inside `TriLayerStack::run_verdict`, so a neutralized scorer never
//!   resurrects a branch that L1 already vetoed).
//! * **GD-1 (compose, never re-implement)**: the injector wraps whatever scorer
//!   closure the caller provides; it does NOT re-implement L2 invocation logic.
//!   H-44 `TrapBehaviorDetected` fires on the CANDIDATE side; I-24 fires on the
//!   SCORER side — complementary, not overlapping.
//! * **GD-2 (recomputable)**: pair selection and injection are deterministic
//!   under the same `ELAI_SEED` (via `CanaryInjectorConfig::seed`).
//!
//! # Production hook
//!
//! `CanaryInjector::maybe_inject` is called from
//! `crates/orchestrator/src/verifier_stack.rs::TriLayerStack::run_verdict`
//! at the L2 call site, gated by `ELAI_CANARY_INJECTOR_ENABLE`.
//!
//! WIRE-DARK[I-24]: the live dual-score through a neural PpmClient is PENDING
//! until D-026 wires `PpmClient` into the L2 path. The current injector wraps
//! the conformal-tau L2 stub and measures THAT scorer's delta; the SKELETON
//! confirms the SSB plumbing, threshold logic, neutralize state machine, and
//! production call-site wiring. The neural-scorer path activates when D-026
//! lands and `run_l2_score` begins calling `PpmClient`.

use std::path::Path;
use std::sync::Arc;

use tracing::{debug, warn};

use elai_core::clock::{ClockProvider, OsClock};

use crate::ssb::{
    Ssb, SsbPayload, PrmCanaryEventKind, PrmCanaryEventPayload, ELAI_CANARY_INJECTOR_ENABLE_ENV,
};

// ---------------------------------------------------------------------------
// Local now_ms helper (pattern from verifier_stack.rs / cache_economics.rs)
// ---------------------------------------------------------------------------

fn now_ms() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

// ---------------------------------------------------------------------------
// Named constants (no magic numbers — HARD RULE)
// ---------------------------------------------------------------------------

/// Default injection probability when no config is provided or when the
/// `ELAI_SEED` env var is absent.  0.05 = 5 % of L2 invocations carry a
/// canary pair.  This is a `[DESIGN]` constant — calibrate with real data
/// before hard-gating (ID-5).
const DEFAULT_INJECTION_PROBABILITY: f64 = 0.05;

/// Default bias-detection threshold in basis points (1/10 000). Used when
/// `metrics/canary_bias_threshold.toml` is absent at startup. This is a
/// `[DESIGN]` bootstrap value; the ID-5 measured baseline should overwrite
/// it once the fixture-scorer corpus run completes.
const DEFAULT_BIAS_THRESHOLD_BP: u32 = 500; // 5 % score delta

/// Number of consecutive `StyleBiasDetected` events within one run that
/// trigger `ScorerNeutralized` (scorer weight zeroed for the run). `[DESIGN]`
/// constant — tune against real runs (ID-5 cadence).
pub const CONSECUTIVE_NEUTRALIZE_THRESHOLD: u32 = 3;

/// The scorer-weight multiplier applied after neutralization. `0.0` = weight
/// fully zeroed for the run (the only sound option per ID-4 veto-supremacy;
/// a partial weight reduction would let a biased scorer partially influence
/// the ranking).
const NEUTRALIZED_WEIGHT: f64 = 0.0;

/// Maximum consecutive-detection count retained in the payload field (avoids
/// unbounded counter growth in a long run with a persistently biased scorer).
const MAX_CONSECUTIVE_CAP: u32 = 255;

/// Scorer-id string used when the canary wraps the conformal-tau L2 stub
/// (the current `run_l2_score` implementation prior to D-026 PpmClient wiring).
pub const SCORER_ID_CONFORMAL_TAU: &str = "conformal_tau";

// ---------------------------------------------------------------------------
// CanaryPair — the frozen fixture unit
// ---------------------------------------------------------------------------

/// One entry from `fixtures/canary_pairs.json`.
///
/// The pair consists of an original code snippet and an AST-preserving
/// isomorphic variant (produced by one of the MuCoCo operators — `for_to_while`,
/// `rename_idempotent`, `constant_unfolding` — authored out-of-band and
/// frozen). Both variants are semantically equivalent; a correctly-implemented
/// scorer MUST rate them within the bias threshold.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CanaryPair {
    /// A short human-readable label (e.g. `"for_to_while_01"`). Informational.
    pub label: String,
    /// The original (unobfuscated) code snippet fed to the L2 scorer.
    pub original: String,
    /// The isomorphic (AST-preserving obfuscated) variant fed to the L2 scorer.
    pub obfuscated: String,
}

/// The full frozen fixture set loaded from `fixtures/canary_pairs.json`.
#[derive(Debug, serde::Deserialize)]
struct CanaryFixture {
    /// Ordered list of canary pairs. Pair selection is index-based and deterministic
    /// under the configured seed (GD-2).
    pub pairs: Vec<CanaryPair>,
}

// ---------------------------------------------------------------------------
// CanaryInjectorConfig
// ---------------------------------------------------------------------------

/// Configuration for [`CanaryInjector`].
///
/// Constructed at `TriLayerStack` boot time and passed into `maybe_inject`.
#[derive(Debug, Clone)]
pub struct CanaryInjectorConfig {
    /// Probability in `[0.0, 1.0]` that a given L2 invocation carries a canary
    /// pair.  Deterministic under `seed` (GD-2).
    pub injection_probability: f64,
    /// Seed for the PRNG used for pair selection and injection probability.
    /// Should be set from `ELAI_SEED` at start-up for GD-2 reproducibility.
    pub seed: u64,
    /// Bias-detection threshold in basis points (from
    /// `metrics/canary_bias_threshold.toml`, ID-5).
    pub bias_threshold_bp: u32,
}

impl Default for CanaryInjectorConfig {
    fn default() -> Self {
        Self {
            injection_probability: DEFAULT_INJECTION_PROBABILITY,
            seed: 0,
            bias_threshold_bp: DEFAULT_BIAS_THRESHOLD_BP,
        }
    }
}

impl CanaryInjectorConfig {
    /// Read `metrics/canary_bias_threshold.toml` (relative to `project_root`)
    /// and return a config with the measured threshold. Falls back to
    /// [`DEFAULT_BIAS_THRESHOLD_BP`] if the file is absent or malformed (logs
    /// a warning; the gate is not blocked by missing threshold data — the
    /// `[DESIGN]` bootstrap value is conservative enough to prevent over-firing).
    #[must_use]
    pub fn load(project_root: &Path, injection_probability: f64, seed: u64) -> Self {
        let threshold_bp = load_bias_threshold_bp(project_root);
        Self {
            injection_probability,
            seed,
            bias_threshold_bp: threshold_bp,
        }
    }
}

/// Parse `bias_threshold_bp` from `metrics/canary_bias_threshold.toml`.
///
/// The file shape is:
/// ```toml
/// [canary]
/// bias_threshold_bp = 500
/// ```
///
/// Returns [`DEFAULT_BIAS_THRESHOLD_BP`] on any parse failure.
fn load_bias_threshold_bp(project_root: &Path) -> u32 {
    let path = project_root
        .join("metrics")
        .join("canary_bias_threshold.toml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        debug!(
            canary.threshold_toml = %path.display(),
            "metrics/canary_bias_threshold.toml not found; using bootstrap default {DEFAULT_BIAS_THRESHOLD_BP} bp"
        );
        return DEFAULT_BIAS_THRESHOLD_BP;
    };
    // Minimal TOML parse: look for `bias_threshold_bp = <N>`.
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("bias_threshold_bp") {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix('=') {
                if let Ok(val) = rest.trim().parse::<u32>() {
                    debug!(canary.threshold_bp = val, "loaded bias threshold from TOML");
                    return val;
                }
            }
        }
    }
    warn!(
        canary.threshold_toml = %path.display(),
        "could not parse bias_threshold_bp from TOML; using bootstrap default"
    );
    DEFAULT_BIAS_THRESHOLD_BP
}

// ---------------------------------------------------------------------------
// BiasVerdict
// ---------------------------------------------------------------------------

/// Result of one canary bias check.
#[derive(Debug, Clone, PartialEq)]
pub enum BiasVerdict {
    /// Delta is below the threshold. The scorer behaved consistently.
    NoBias {
        delta_bp: u32,
    },
    /// Delta meets or exceeds the threshold. The scorer is style-biased.
    Biased {
        delta_bp: u32,
    },
}

impl BiasVerdict {
    /// True iff the scorer was detected as style-biased on this pair.
    #[must_use]
    pub fn is_biased(&self) -> bool {
        matches!(self, Self::Biased { .. })
    }
}

// ---------------------------------------------------------------------------
// CanaryInjector
// ---------------------------------------------------------------------------

/// Scorer-side anti-gaming canary injector.
///
/// One `CanaryInjector` is held per `TriLayerStack` instance and is called
/// from `run_verdict` for every L2 invocation that is sampled for canary
/// injection.
///
/// # State
///
/// The injector tracks consecutive `StyleBiasDetected` events across the
/// lifetime of the stack (one run). When `consecutive_count` reaches
/// [`CONSECUTIVE_NEUTRALIZE_THRESHOLD`], the scorer weight is set to
/// [`NEUTRALIZED_WEIGHT`] and remains there for the run.
pub struct CanaryInjector {
    /// Frozen canary pairs loaded at construction (R8 — never regenerated).
    pairs: Vec<CanaryPair>,
    /// Active configuration (thresholds, seed, probability).
    config: CanaryInjectorConfig,
    /// Monotonically increasing run counter. Used to deterministically advance
    /// the PRNG state across consecutive `maybe_inject` calls (GD-2).
    call_count: u64,
    /// Number of consecutive `StyleBiasDetected` events in this run.
    consecutive_count: u32,
    /// True once `ScorerNeutralized` has fired; the scorer weight stays at 0.0.
    neutralized: bool,
    /// Shared SSB writer for emitting [`PrmCanaryEventPayload`] rows.
    ssb: Arc<Ssb>,
}

impl CanaryInjector {
    /// Construct a `CanaryInjector` from the frozen fixture path and config.
    ///
    /// Returns `None` when `fixtures/canary_pairs.json` is absent (the
    /// injector is non-operative; the caller falls through to the normal L2
    /// path). Returns `Some(Err(_))` on a parse failure so the caller can
    /// log a warning without blocking the L2 path.
    ///
    /// # Errors
    ///
    /// `Err(msg)` on JSON parse failure; the caller logs and continues without
    /// canary injection.
    pub fn new(
        fixture_path: &Path,
        config: CanaryInjectorConfig,
        ssb: Arc<Ssb>,
    ) -> Result<Option<Self>, String> {
        if !fixture_path.exists() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(fixture_path)
            .map_err(|e| format!("canary fixture read error: {e}"))?;
        let fixture: CanaryFixture = serde_json::from_str(&text)
            .map_err(|e| format!("canary fixture JSON parse error: {e}"))?;
        if fixture.pairs.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            pairs: fixture.pairs,
            config,
            call_count: 0,
            consecutive_count: 0,
            neutralized: false,
            ssb,
        }))
    }

    /// Current scorer weight. `1.0` while unbiased/not-yet-neutralized, `0.0`
    /// after neutralization. The caller multiplies the raw L2 score by this value.
    ///
    /// ID-4: this weight affects L2 ORDERING only. A zero weight for an L2
    /// scorer does NOT change any L1 HardVeto; the veto is structurally prior.
    /// Dashboard scorer-health badge consumer is deferred (follow-on plan).
    // WIRE-DARK[I-24]: dashboard scorer-health badge reads this; deferred dashboard plan.
    #[must_use]
    pub fn scorer_weight(&self) -> f64 {
        if self.neutralized {
            NEUTRALIZED_WEIGHT
        } else {
            1.0_f64
        }
    }

    /// True if the scorer has been neutralized for this run.
    /// Dashboard scorer-health badge consumer is deferred (follow-on plan).
    // WIRE-DARK[I-24]: dashboard scorer-health badge reads this; deferred dashboard plan.
    #[must_use]
    pub fn is_neutralized(&self) -> bool {
        self.neutralized
    }

    /// Attempt a canary injection on this L2 call.
    ///
    /// * If the env var `ELAI_CANARY_INJECTOR_ENABLE` is not set to a truthy
    ///   value (`"1"`, `"true"`, `"yes"`, `"on"`, case-insensitive), returns
    ///   immediately without injection (wire-dark gate, default-OFF).
    /// * With probability `config.injection_probability` (deterministic under
    ///   `config.seed` XOR `call_count`), selects a pair from the frozen
    ///   fixture set (index deterministic under seed).
    /// * Calls `scorer` twice (original + obfuscated variant) to obtain both
    ///   scores.
    /// * Computes `delta_bp`, compares with `bias_threshold_bp`.
    /// * Emits a `PrmCanaryEvent` SSB row (fire-and-forget).
    /// * Updates `consecutive_count` and (if threshold reached) sets
    ///   `neutralized = true`.
    ///
    /// `scorer` is a closure `&str -> f64` (branch source -> score in `[0.0,1.0]`);
    /// wraps whatever L2 scorer the caller already uses (GD-1: does NOT
    /// re-implement L2 logic).
    ///
    /// `agent_id` and `superstep` are forwarded to the SSB emit call.
    pub async fn maybe_inject<F>(
        &mut self,
        scorer: F,
        scorer_id: &str,
        agent_id: &str,
        superstep: i64,
    ) where
        F: Fn(&str) -> f64,
    {
        if !canary_injector_enabled() {
            self.call_count = self.call_count.saturating_add(1);
            return;
        }

        let call = self.call_count;
        self.call_count = self.call_count.saturating_add(1);

        // Deterministic injection decision (GD-2): XOR seed with call counter.
        // A 64-bit LCG step suffices for this application.
        let prng_state = lcg_step(self.config.seed ^ call);
        let inject_frac = (prng_state >> 11) as f64 / (1u64 << 53) as f64;
        if inject_frac >= self.config.injection_probability {
            return; // Not sampled this invocation.
        }

        // Deterministic pair selection (GD-2).
        let pair_index =
            (lcg_step(prng_state) as usize) % self.pairs.len();
        let pair = &self.pairs[pair_index];

        // Score both variants through the SAME scorer closure (GD-1).
        let score_original = scorer(&pair.original).clamp(0.0, 1.0);
        let score_obfuscated = scorer(&pair.obfuscated).clamp(0.0, 1.0);

        // Compute delta in basis points (non-negative).
        let raw_delta = (score_original - score_obfuscated).abs();
        let delta_bp = (raw_delta * 10_000.0).round() as u32;

        let verdict = self.detect_bias(delta_bp);
        let is_biased = verdict.is_biased();

        if is_biased {
            self.consecutive_count = self
                .consecutive_count
                .saturating_add(1)
                .min(MAX_CONSECUTIVE_CAP);
        } else {
            self.consecutive_count = 0;
        }

        // Determine the kind and (possibly) neutralize.
        let kind = if !is_biased {
            PrmCanaryEventKind::CanaryInjected
        } else if self.consecutive_count >= CONSECUTIVE_NEUTRALIZE_THRESHOLD {
            self.neutralized = true;
            PrmCanaryEventKind::ScorerNeutralized
        } else {
            PrmCanaryEventKind::StyleBiasDetected
        };

        emit_prm_canary_to_ssb(
            &self.ssb,
            agent_id,
            superstep,
            PrmCanaryEventPayload {
                kind,
                scorer_id: scorer_id.to_owned(),
                pair_index: pair_index as u32,
                score_original,
                score_obfuscated,
                delta_bp,
                threshold_bp: self.config.bias_threshold_bp,
                consecutive_count: self.consecutive_count,
                ts_ms: now_ms(),
            },
        )
        .await;
    }

    /// Compute the bias verdict for a given delta (in basis points).
    ///
    /// Pure helper — no state mutation. Called from `maybe_inject`.
    fn detect_bias(&self, delta_bp: u32) -> BiasVerdict {
        if delta_bp >= self.config.bias_threshold_bp {
            BiasVerdict::Biased { delta_bp }
        } else {
            BiasVerdict::NoBias { delta_bp }
        }
    }
}

// ---------------------------------------------------------------------------
// SSB emit helper
// ---------------------------------------------------------------------------

/// Emit a [`PrmCanaryEventPayload`] SSB row. Fire-and-forget (errors are
/// traced but not propagated — the canary gate verdict is unaffected by SSB
/// write failures). Mirrors the pattern of
/// `tdd_preflight::emit_tdd_preflight_blocked_to_ssb`.
pub async fn emit_prm_canary_to_ssb(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    payload: PrmCanaryEventPayload,
) {
    let ssb_payload = SsbPayload::PrmCanaryEvent(payload);
    match writer
        .write_payload(agent_id, superstep, ssb_payload, Vec::new())
        .await
    {
        Ok(_) => {
            debug!(canary.ssb = "emitted PrmCanaryEvent row");
        }
        Err(e) => {
            warn!(
                canary.ssb_err = %e,
                "PrmCanaryEvent SSB emit failed (canary verdict unaffected)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Gate helper
// ---------------------------------------------------------------------------

/// True when `ELAI_CANARY_INJECTOR_ENABLE` is set to a truthy literal
/// (`"1"`, `"true"`, `"yes"`, `"on"`, case-insensitive). Default-OFF (R9).
#[must_use]
pub fn canary_injector_enabled() -> bool {
    match std::env::var(ELAI_CANARY_INJECTOR_ENABLE_ENV) {
        Ok(v) => matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// PRNG helper (minimal LCG — no external dep)
// ---------------------------------------------------------------------------

/// One step of a 64-bit linear congruential generator (Knuth's constants).
/// Used for deterministic pair selection and injection probability (GD-2).
/// Not a cryptographic PRNG — correctness here is reproducibility, not
/// unpredictability.
#[must_use]
fn lcg_step(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal `CanaryInjectorConfig` with always-inject probability
    /// and the given threshold (no file I/O, no SSB needed for pure logic).
    fn test_config_with_threshold(bias_threshold_bp: u32) -> CanaryInjectorConfig {
        CanaryInjectorConfig {
            injection_probability: 1.0,
            seed: 42,
            bias_threshold_bp,
        }
    }

    /// A single canary pair representative of the `for_to_while` MuCoCo obfuscation.
    fn sample_pair() -> CanaryPair {
        CanaryPair {
            label: "for_to_while_01".to_owned(),
            original:   "for i in range(n): total += i".to_owned(),
            obfuscated: "i = 0\nwhile i < n:\n    total += i\n    i += 1".to_owned(),
        }
    }

    // --- Pure-logic helpers that do NOT require an Ssb instance ---------------

    /// A minimal stand-alone bias-detector that avoids constructing `CanaryInjector`.
    fn raw_detect_bias(delta_bp: u32, threshold_bp: u32) -> BiasVerdict {
        if delta_bp >= threshold_bp {
            BiasVerdict::Biased { delta_bp }
        } else {
            BiasVerdict::NoBias { delta_bp }
        }
    }

    #[test]
    fn detect_bias_below_threshold() {
        let v = raw_detect_bias(499, 500);
        assert_eq!(v, BiasVerdict::NoBias { delta_bp: 499 });
        assert!(!v.is_biased());
    }

    #[test]
    fn detect_bias_at_threshold() {
        let v = raw_detect_bias(500, 500);
        assert_eq!(v, BiasVerdict::Biased { delta_bp: 500 });
        assert!(v.is_biased());
    }

    #[test]
    fn detect_bias_above_threshold() {
        let v = raw_detect_bias(1001, 500);
        assert!(v.is_biased());
    }

    #[test]
    fn scorer_weight_when_not_neutralized() {
        // Weight logic is a pure fn of `neutralized` field.
        let weight = |neutralized: bool| -> f64 {
            if neutralized {
                NEUTRALIZED_WEIGHT
            } else {
                1.0_f64
            }
        };
        assert!((weight(false) - 1.0_f64).abs() < f64::EPSILON);
        assert!((weight(true) - 0.0_f64).abs() < f64::EPSILON);
    }

    #[test]
    fn lcg_step_is_deterministic() {
        let a = lcg_step(42);
        let b = lcg_step(42);
        assert_eq!(a, b);
        assert_ne!(a, 42, "LCG must advance the state");
    }

    #[test]
    fn lcg_step_differs_on_distinct_inputs() {
        let a = lcg_step(1);
        let b = lcg_step(2);
        assert_ne!(a, b);
    }

    #[test]
    fn load_bias_threshold_bootstrap_on_missing_file() {
        let thresh = load_bias_threshold_bp(Path::new("__nonexistent_elai_test_dir__"));
        assert_eq!(thresh, DEFAULT_BIAS_THRESHOLD_BP);
    }

    #[test]
    fn pair_index_selection_is_bounded() {
        // Verify the modulo wraps into a valid index regardless of LCG output.
        let n_pairs = 5_usize;
        for seed in 0_u64..20 {
            let prng = lcg_step(seed);
            let prng2 = lcg_step(prng);
            let idx = (prng2 as usize) % n_pairs;
            assert!(idx < n_pairs, "index {idx} out of bounds for {n_pairs} pairs");
        }
    }

    #[test]
    fn consecutive_neutralize_threshold_constant_is_positive() {
        assert!(CONSECUTIVE_NEUTRALIZE_THRESHOLD >= 1);
    }

    #[test]
    fn default_bias_threshold_bp_is_positive() {
        assert!(DEFAULT_BIAS_THRESHOLD_BP > 0);
    }

    #[test]
    fn injection_probability_range_check() {
        let cfg = CanaryInjectorConfig::default();
        assert!(cfg.injection_probability >= 0.0);
        assert!(cfg.injection_probability <= 1.0);
    }

    #[test]
    fn sample_pair_has_nonempty_fields() {
        let p = sample_pair();
        assert!(!p.label.is_empty());
        assert!(!p.original.is_empty());
        assert!(!p.obfuscated.is_empty());
        assert_ne!(p.original, p.obfuscated, "original and obfuscated must differ");
    }

    #[test]
    fn canary_injector_enabled_is_false_without_env() {
        // Env var should NOT be set in the test environment.
        // Remove it just in case.
        std::env::remove_var(ELAI_CANARY_INJECTOR_ENABLE_ENV);
        assert!(!canary_injector_enabled());
    }
}
