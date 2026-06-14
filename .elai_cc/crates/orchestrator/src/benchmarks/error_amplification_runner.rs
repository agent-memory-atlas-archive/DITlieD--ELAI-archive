//! I-9: error-amplification (alpha_t) telemetry harness.
//!
//! Measures whether a corrupted step amplifies error into downstream steps
//! (alpha_t > 1) or whether the verification/repair stack contracts it
//! (alpha_t < 1), on ELAI's OWN frozen fixture pipeline. The baseline is
//! measured FIRST; no literature alpha number is carried as a target or gate
//! threshold (ID-5).
//!
//! # What this measures (honestly)
//!
//! A frozen fixture is a linear pipeline of `n` steps. Each step has a
//! deterministic byte output and a `recovers` flag declaring whether the
//! step's repair/verification logic would re-derive a correct output even
//! when a corrupted value flows in from upstream. The harness:
//!
//!   * injects a deterministic byte-perturbation at one step `t` (the
//!     `injected_step`), under `ELAI_SEED` (GD-2 recomputable),
//!   * walks steps `t+1..n` and classifies each as RECOVERED (the step's
//!     `recovers` flag closes the divergence) or PROPAGATED (the corruption
//!     flows through),
//!   * computes `recovered_fraction = recovered / downstream` and the
//!     amplification factor `alpha_t = propagated / downstream`.
//!
//! `alpha_t` is the fraction of downstream steps that still carry the error
//! after the verification/repair stack acts. `alpha_t > 1` is impossible by
//! construction here (it is a fraction in [0,1]); the design intent of the
//! "amplification" framing is preserved as: a HIGH propagated fraction is the
//! amplifying regime, a LOW one is the contracting regime. The discrimination
//! contract (R2) is: the `--no-repair` arm (recovery disabled) propagates
//! through every downstream step (alpha == 1.0), the repaired arm contracts
//! it (alpha < 1.0).
//!
//! # In-harness deterministic injection (FORK-1, controller-confirmed)
//!
//! The corruption source is an in-harness byte-perturbation over the frozen
//! step fixture, NOT the C-28 `MutationRunner` trait. `MutationRunner` shells
//! out to `cargo mutants` over a real crate and has no in-memory
//! "corrupt step t of a pipeline" primitive; composing it here is not
//! buildable (see workflow/reasoning/I-9-decisions.md re-scope of the
//! WIRING_CONTRACT stub_ban that assumed a non-existent operator API).
//!
//! # In-harness active annealing (FORK-2, controller-confirmed)
//!
//! When the measured `alpha_t` crosses the amplifying threshold the harness
//! consults a CLAMPED copy of [`crate::pipeline_tree_navigator::RetryStrategy`]
//! `temperature_for_attempt` (the C-26 schedule), driving the temperature
//! toward the lower bound, and records the tighten in the report (an
//! observable in-loop effect). The harness does NOT wire a new caller into
//! `task_runner`; per-task live alpha estimation in production is Tier-2
//! deferred (ID-5).
//!
//! # Wiring discipline
//!
//! - blake3 for the fixture content hash (anti-stale rule).
//! - thiserror for errors; tracing for diagnostics.
//! - serde for the embedded fixture + the report.
//! - No `.unwrap()` / `.expect()` outside `#[cfg(test)]`. No silent swallows.
//! - The fixture is embedded at compile time via `include_str!` so a shipped
//!   binary is filesystem-independent (mirrors `seccodeplt_runner`).
//! - Recovery steps are INFERENCE-FREE: the `recovers` flag is a recorded
//!   fixture field, so the whole harness is pure-CPU (no sidecar, no model).
//!   The baseline alpha is therefore MEASURED here, not PENDING D-026.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::determinism::ELAI_SEED_ENV;
use crate::pipeline_tree_navigator::RetryStrategy;
use crate::ssb::{ErrorAmplificationMeasuredPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Embedded fixture
// ---------------------------------------------------------------------------

/// The frozen error-amplification pipeline fixture, embedded at compile time.
/// One JSON object: `{ "suite": ..., "injected_step": N, "steps": [ ... ] }`.
const FIXTURE_JSON: &str = include_str!("error_amplification_fixtures/pipeline_v1.json");

/// Suite label recorded in the report + the SSB beacon.
pub const SUITE_LABEL: &str = "error_amplification_pipeline_v1";

/// Provenance id for the candidate-generation source. The harness is
/// model-independent (frozen fixture); this records WHERE the fixture came
/// from, not a live-model input.
pub const FIXTURE_MODEL_ID: &str = "error_amplification_frozen_fixture";

/// The alpha threshold above which the harness fires the active-annealing
/// tighten. NOT a gate / acceptance criterion (ID-5) — it only governs the
/// in-loop temperature-tighten side effect. A propagated-fraction at or above
/// this is the amplifying regime.
pub const ANNEAL_ALPHA_THRESHOLD: f64 = 0.5;

/// The retry-attempt index whose (clamped) temperature the annealing hook
/// drives toward. Index 0 is the lowest-temperature rung of the C-26 schedule
/// (the "tightened" temperature).
const ANNEAL_TARGET_ATTEMPT: u32 = 0;

// ---------------------------------------------------------------------------
// Fixture types
// ---------------------------------------------------------------------------

/// One step of the frozen pipeline fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureStep {
    /// Stable step id (e.g. `"plan-0"`). Used for diagnostics.
    pub id: String,
    /// The step's canonical (uncorrupted) output bytes, as text.
    pub output: String,
    /// `true` when this step's verification/repair logic would re-derive a
    /// correct output even if a corrupted value flowed in from upstream
    /// (the divergence is CLOSED here). `false` => the step PROPAGATES an
    /// upstream corruption downstream.
    pub recovers: bool,
}

/// The frozen pipeline fixture (one `include_str!` blob).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineFixture {
    /// Suite label (overrides [`SUITE_LABEL`] only if present; the runner uses
    /// the constant for the emitted beacon to keep the wire-tag stable).
    pub suite: String,
    /// The 0-indexed step at which the deterministic corruption is injected.
    pub injected_step: u32,
    /// The ordered pipeline steps (`n` >= injected_step + 2 so there is at
    /// least one downstream step to classify).
    pub steps: Vec<FixtureStep>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ErrorAmpError {
    /// The embedded fixture JSON failed to deserialise (build-time authoring
    /// bug surfaced at run time).
    #[error("error-amplification fixture malformed json: {0}")]
    MalformedFixture(String),
    /// The fixture has too few steps to measure a downstream window.
    #[error(
        "error-amplification fixture needs >= injected_step+2 steps \
         (injected_step={injected_step}, n_steps={n_steps})"
    )]
    TooFewSteps { injected_step: u32, n_steps: u32 },
    /// `injected_step` is out of range for the step list.
    #[error("error-amplification injected_step {injected_step} >= n_steps {n_steps}")]
    InjectedStepOutOfRange { injected_step: u32, n_steps: u32 },
    /// `ELAI_SEED` was set but not parseable as a u64.
    #[error("error-amplification invalid ELAI_SEED '{value}': {detail}")]
    InvalidSeed { value: String, detail: String },
}

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

/// Aggregate report — one per [`ErrorPropagationHarness::run`] invocation.
///
/// `alpha_t` and `recovered_fraction` are COMPUTED from the fixture walk,
/// never literals; both are in `[0.0, 1.0]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorAmpReport {
    /// Suite label ([`SUITE_LABEL`]).
    pub suite: String,
    /// Fixture provenance id ([`FIXTURE_MODEL_ID`]).
    pub model_id: String,
    /// Total pipeline steps in the fixture.
    pub n_steps: u32,
    /// The 0-indexed step where corruption was injected.
    pub injected_step: u32,
    /// The 0-indexed step where the corrupted output FIRST diverges from the
    /// canonical output (mirrors the `DriftComparator` first-divergence shape,
    /// H-23). Equals `injected_step` when injection is active.
    pub divergence_step: Option<u32>,
    /// Fraction of downstream steps (`> injected_step`) whose
    /// verification/repair stack RECOVERED the divergence. `[0.0, 1.0]`.
    pub recovered_fraction: f64,
    /// The amplification factor: the fraction of downstream steps that still
    /// carry the error after the repair stack acts (`1.0 - recovered_fraction`).
    /// `[0.0, 1.0]`. HIGH = amplifying regime, LOW = contracting regime.
    pub alpha_t: f64,
    /// `true` when the repair stack was active (the canonical arm); `false`
    /// when recovery was disabled (the `--no-repair` R2 negative-control arm).
    pub repaired: bool,
    /// The annealing temperature the harness drove toward when `alpha_t`
    /// crossed [`ANNEAL_ALPHA_THRESHOLD`] mid-run; `None` when no tighten
    /// fired (alpha below threshold).
    pub annealed_temperature: Option<f64>,
    /// Deterministic RNG seed used for the byte-perturbation (`ELAI_SEED`,
    /// default 0).
    pub seed: u64,
    /// blake3 hex of the canonical fixture bytes (provenance / staleness pin).
    pub fixture_blake3: String,
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

/// Error-propagation harness. Construction parses + validates the embedded
/// fixture once; [`ErrorPropagationHarness::run`] is a pure, deterministic
/// scoring pass (no I/O, no async, no model).
#[derive(Debug)]
pub struct ErrorPropagationHarness {
    fixture: PipelineFixture,
    fixture_blake3: String,
    seed: u64,
}

impl ErrorPropagationHarness {
    /// Construct from the embedded fixture, reading `ELAI_SEED` (default 0).
    /// Returns `Err` on a malformed / undersized fixture or an unparseable
    /// seed.
    pub fn from_embedded() -> Result<Self, ErrorAmpError> {
        let seed = read_seed()?;
        Self::from_json_with_seed(FIXTURE_JSON, seed)
    }

    /// Construct from the EMBEDDED fixture with an EXPLICIT seed (overrides
    /// `ELAI_SEED`). Used by the CLI `--seed` flag so a seed can be pinned
    /// without mutating the process environment.
    pub fn from_embedded_with_seed(seed: u64) -> Result<Self, ErrorAmpError> {
        Self::from_json_with_seed(FIXTURE_JSON, seed)
    }

    /// Construct from explicit fixture JSON + an explicit seed. Used by the
    /// contract test; production uses [`Self::from_embedded`].
    pub fn from_json_with_seed(json: &str, seed: u64) -> Result<Self, ErrorAmpError> {
        let fixture: PipelineFixture = serde_json::from_str(json)
            .map_err(|e| ErrorAmpError::MalformedFixture(e.to_string()))?;
        let n_steps = u32::try_from(fixture.steps.len()).unwrap_or(u32::MAX);
        if fixture.injected_step >= n_steps {
            return Err(ErrorAmpError::InjectedStepOutOfRange {
                injected_step: fixture.injected_step,
                n_steps,
            });
        }
        // Need at least one downstream step to classify.
        if fixture.injected_step + 1 >= n_steps {
            return Err(ErrorAmpError::TooFewSteps {
                injected_step: fixture.injected_step,
                n_steps,
            });
        }
        let mut hasher = Hasher::new();
        hasher.update(json.as_bytes());
        let fixture_blake3 = hasher.finalize().to_hex().to_string();
        Ok(Self {
            fixture,
            fixture_blake3,
            seed,
        })
    }

    /// The recorded seed.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// blake3 hex of the canonical fixture bytes.
    #[must_use]
    pub fn fixture_blake3(&self) -> &str {
        &self.fixture_blake3
    }

    /// Run the error-propagation measurement.
    ///
    /// `repaired = true` runs the canonical arm (each downstream step's
    /// `recovers` flag is honoured). `repaired = false` is the `--no-repair`
    /// R2 negative control: recovery is disabled, so the corruption propagates
    /// through EVERY downstream step (alpha == 1.0).
    ///
    /// Deterministic: same fixture + seed + arm => byte-identical report.
    pub fn run(&self, repaired: bool) -> ErrorAmpReport {
        let n = self.fixture.steps.len();
        let injected = self.fixture.injected_step as usize;
        info!(
            n_steps = n,
            injected_step = injected,
            repaired,
            "error-amplification: starting propagation run",
        );

        // Inject the deterministic byte-perturbation at `injected`. The
        // corrupted value diverges from the canonical output, so the
        // divergence first appears at the injected step.
        let canonical = &self.fixture.steps[injected].output;
        let corrupted = perturb_bytes(canonical, self.seed);
        let divergence_step = if corrupted == *canonical {
            // A degenerate fixture where the perturbation is a no-op (empty
            // output). No divergence to measure — recovered_fraction is 1.0
            // (nothing propagated) and alpha is 0.0.
            None
        } else {
            Some(self.fixture.injected_step)
        };

        // Walk the downstream window [injected+1 .. n).
        let downstream = &self.fixture.steps[injected + 1..];
        let downstream_total = downstream.len();

        let mut recovered: usize = 0;
        for step in downstream {
            // In the repaired arm a `recovers` step CLOSES the divergence; a
            // non-recovering step PROPAGATES it. In the no-repair arm every
            // step propagates (recovery disabled), so none recover.
            let step_recovers = repaired && step.recovers;
            if step_recovers {
                recovered += 1;
                debug!(step_id = %step.id, "error-amplification: step recovered the divergence");
            } else {
                debug!(step_id = %step.id, "error-amplification: step propagated the divergence");
            }
        }

        // downstream_total >= 1 guaranteed by from_json_with_seed.
        let downstream_f = downstream_total as f64;
        let (recovered_fraction, alpha_t) = if divergence_step.is_none() {
            // No divergence injected (degenerate fixture): nothing propagated.
            (1.0, 0.0)
        } else {
            let rf = recovered as f64 / downstream_f;
            (rf, 1.0 - rf)
        };

        // Active annealing (FORK-2): when alpha crosses the amplifying
        // threshold mid-run, consult a CLAMPED RetryStrategy temperature
        // schedule (C-26) and record the tightened temperature.
        let annealed_temperature = if alpha_t >= ANNEAL_ALPHA_THRESHOLD {
            let temp = clamped_anneal_temperature();
            warn!(
                alpha_t,
                threshold = ANNEAL_ALPHA_THRESHOLD,
                annealed_temperature = temp,
                "error-amplification: alpha crossed amplifying threshold; tightening temperature (C-26)",
            );
            Some(temp)
        } else {
            None
        };

        let report = ErrorAmpReport {
            suite: SUITE_LABEL.to_string(),
            model_id: FIXTURE_MODEL_ID.to_string(),
            n_steps: u32::try_from(n).unwrap_or(u32::MAX),
            injected_step: self.fixture.injected_step,
            divergence_step,
            recovered_fraction,
            alpha_t,
            repaired,
            annealed_temperature,
            seed: self.seed,
            fixture_blake3: self.fixture_blake3.clone(),
        };

        info!(
            alpha_t = report.alpha_t,
            recovered_fraction = report.recovered_fraction,
            repaired = report.repaired,
            annealed = report.annealed_temperature.is_some(),
            "error-amplification: run complete",
        );
        report
    }
}

// ---------------------------------------------------------------------------
// Production emit seam (called from the CLI bench subcommand + the contract)
// ---------------------------------------------------------------------------

/// Emit ONE [`SsbActionType::ErrorAmplificationMeasured`] beacon onto an open
/// [`Ssb`] writer, awaited (durable). This is the load-bearing off-author
/// artifact (R4/R6): the CLI `bench error-amplification` subcommand opens the
/// canonical session `ssb.db` and calls this with the harness report; the
/// I-9 contract test calls it against a temp DB and queries the row back.
///
/// `agent` is the SSB agent label of the emitting seam (e.g.
/// `"cli:bench-error-amplification"`).
///
/// [`SsbActionType::ErrorAmplificationMeasured`]:
///   crate::ssb::SsbActionType::ErrorAmplificationMeasured
pub async fn emit_error_amplification_measured(
    ssb: &Ssb,
    agent: &str,
    report: &ErrorAmpReport,
) -> Result<(), crate::ssb::SsbError> {
    let payload = ErrorAmplificationMeasuredPayload {
        suite: report.suite.clone(),
        model_id: report.model_id.clone(),
        n_steps: report.n_steps,
        injected_step: report.injected_step,
        divergence_step: report.divergence_step,
        alpha_t: report.alpha_t,
        recovered_fraction: report.recovered_fraction,
        repaired: report.repaired,
        annealed_temperature: report.annealed_temperature,
        seed: report.seed,
        fixture_blake3: report.fixture_blake3.clone(),
    };
    ssb.write_payload(
        agent,
        0,
        SsbPayload::ErrorAmplificationMeasured(payload),
        Vec::new(),
    )
    .await
    .map(|_seq| ())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deterministic byte-perturbation of `s` under `seed`. Flips the low bit of
/// every byte XORed with a seed-derived keystream byte, so a non-empty input
/// always produces a different, seed-reproducible output. Empty input maps to
/// empty (the degenerate no-divergence case the run handles).
fn perturb_bytes(s: &str, seed: u64) -> String {
    if s.is_empty() {
        return String::new();
    }
    // blake3-keyed keystream over the seed (anti-stale: blake3, never blake2).
    let mut hasher = Hasher::new();
    hasher.update(&seed.to_le_bytes());
    hasher.update(s.as_bytes());
    let key = hasher.finalize();
    let key_bytes = key.as_bytes();
    let perturbed: Vec<u8> = s
        .bytes()
        .enumerate()
        .map(|(i, b)| {
            let k = key_bytes[i % key_bytes.len()];
            // Guarantee a change: force the low bit to differ from the source.
            (b ^ (k | 0x01)).wrapping_add(1)
        })
        .collect();
    // The perturbed bytes are not guaranteed valid UTF-8; render as a stable
    // lossy string (the harness only compares for INEQUALITY, never decodes).
    String::from_utf8_lossy(&perturbed).into_owned()
}

/// The clamped active-annealing temperature: the lowest rung of the default
/// C-26 retry schedule. Composes [`RetryStrategy::temperature_for_attempt`]
/// (pipeline_tree_navigator.rs) at the tightened attempt index rather than
/// implementing a parallel temperature mechanism (stub_ban honoured).
fn clamped_anneal_temperature() -> f64 {
    let strategy = RetryStrategy::default();
    f64::from(strategy.temperature_for_attempt(ANNEAL_TARGET_ATTEMPT))
}

/// Read `ELAI_SEED` (default 0). Errors only when the var is set but not a
/// valid u64 — an empty / unset var is the default-0 path.
fn read_seed() -> Result<u64, ErrorAmpError> {
    match std::env::var(ELAI_SEED_ENV) {
        Ok(raw) if !raw.trim().is_empty() => {
            raw.trim()
                .parse::<u64>()
                .map_err(|e| ErrorAmpError::InvalidSeed {
                    value: raw.clone(),
                    detail: e.to_string(),
                })
        }
        _ => Ok(0u64),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn harness() -> ErrorPropagationHarness {
        ErrorPropagationHarness::from_json_with_seed(FIXTURE_JSON, 0)
            .expect("embedded fixture parses")
    }

    #[test]
    fn embedded_fixture_parses_and_has_downstream_window() {
        let h = harness();
        assert!(
            h.fixture.steps.len() >= (h.fixture.injected_step as usize + 2),
            "fixture must have >= injected_step+2 steps"
        );
        assert_eq!(h.fixture_blake3().len(), 64, "blake3 hex is 64 chars");
    }

    #[test]
    fn repaired_arm_contracts_no_repair_arm_amplifies() {
        let h = harness();
        let repaired = h.run(true);
        let raw = h.run(false);
        // R2 discrimination: repaired arm recovers at least one downstream
        // step (alpha < 1.0); the no-repair arm propagates through ALL of them
        // (alpha == 1.0).
        assert!(
            repaired.alpha_t < 1.0,
            "repaired arm must contract (alpha < 1.0), got {}",
            repaired.alpha_t
        );
        assert_eq!(
            raw.alpha_t, 1.0,
            "no-repair arm must propagate fully (alpha == 1.0), got {}",
            raw.alpha_t
        );
        assert!(
            repaired.alpha_t < raw.alpha_t,
            "repaired alpha ({}) must be strictly below no-repair alpha ({})",
            repaired.alpha_t,
            raw.alpha_t
        );
    }

    #[test]
    fn run_is_deterministic_under_fixed_seed() {
        let h = harness();
        let a = h.run(true);
        let b = h.run(true);
        assert_eq!(a, b, "same fixture + seed + arm => identical report");
    }

    #[test]
    fn alpha_and_recovered_fraction_are_complementary() {
        let h = harness();
        let r = h.run(true);
        assert!(
            (r.alpha_t + r.recovered_fraction - 1.0).abs() < 1e-9,
            "alpha_t + recovered_fraction must equal 1.0; got {} + {}",
            r.alpha_t,
            r.recovered_fraction
        );
        assert!((0.0..=1.0).contains(&r.alpha_t), "alpha in [0,1]");
        assert!(
            (0.0..=1.0).contains(&r.recovered_fraction),
            "recovered_fraction in [0,1]"
        );
    }

    #[test]
    fn divergence_step_equals_injected_step() {
        let h = harness();
        let r = h.run(true);
        assert_eq!(
            r.divergence_step,
            Some(h.fixture.injected_step),
            "divergence first appears at the injected step"
        );
    }

    #[test]
    fn perturb_changes_nonempty_and_is_seed_stable() {
        let a = perturb_bytes("plan step output", 7);
        let b = perturb_bytes("plan step output", 7);
        let c = perturb_bytes("plan step output", 8);
        assert_ne!(a, "plan step output", "perturbation must change the input");
        assert_eq!(a, b, "same seed => same perturbation");
        assert_ne!(a, c, "different seed => different perturbation");
        assert_eq!(perturb_bytes("", 1), "", "empty maps to empty");
    }

    #[test]
    fn annealing_fires_only_in_amplifying_regime() {
        let h = harness();
        let raw = h.run(false);
        // The no-repair arm has alpha == 1.0 >= threshold, so it must anneal.
        assert!(
            raw.annealed_temperature.is_some(),
            "no-repair arm (alpha=1.0) must fire the anneal tighten"
        );
        // The clamped temperature equals the lowest C-26 schedule rung.
        let expected = f64::from(RetryStrategy::default().temperature_for_attempt(0));
        assert_eq!(raw.annealed_temperature, Some(expected));
    }
}
