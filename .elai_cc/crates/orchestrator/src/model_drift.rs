//! G-10 (SCHEMA_VERSION 218): exogenous model-version behavioral-regression gate.
//!
//! # Compose-not-restate (GD-1 — BLOCKING)
//!
//! This module REUSES the D-7 canary stack verbatim:
//!   * [`canary::CanaryFamily`] — the 5 orthogonal families (no re-invention).
//!   * [`canary::FamilyEvalResult`] — per-family score carrier.
//!   * [`canary::decide`] + [`canary::DEFAULT_FAIL_BAND`] / [`canary::DEFAULT_MARGINAL_BAND`]
//!     — the non-determinism BAND (= the FP guard, GD-4).
//!
//! It adds the IDENTITY DIMENSION: a (model_pin, system_fingerprint, lane) tuple
//! keyed to the last-good baseline. Detecting drift = comparing a re-run against
//! a stored per-identity baseline, NOT comparing two templates in one call.
//!
//! # Architecture
//!
//! ```text
//! ModelIdentity            DriftBaseline                DriftVerdict
//! {model_pin, fp, lane} -> {per-family scores, ts} ---> NoDrift
//!                                                   ---> BehavioralDriftSuspected{...}
//!                                                   ---> BaselineCaptured (first-seen)
//! ```
//!
//! # GD-5 honesty (confirmed_by_fingerprint)
//!
//! * LOCAL lane: `system_fingerprint` is a deterministic hardware/firmware pin.
//!   A changed fingerprint PLUS a CanaryRegressed signal = `confirmed_by_fingerprint=true`.
//! * CLOUD lane: Anthropic's `system_fingerprint` is OPAQUE (not a stable replica pin).
//!   Cloud lane NEVER sets `confirmed_by_fingerprint=true` regardless of signal.
//!   Always behavioral-suspected only (GD-5 hard rule).
//!
//! # GD-2 reproducibility
//!
//! `check_model_drift` uses `decide(per_family, DEFAULT_FAIL_BAND, DEFAULT_MARGINAL_BAND)`
//! which is a pure function over the per-family pass-rate vectors. Under a pinned
//! `ELAI_SEED` and a fixed canary bundle order the delta recomputes byte-identically.
//!
//! # References
//!
//! Prior art (motivation only, NOT ELAI targets):
//!   * "Why Is My Prompt Getting Worse?" arXiv:2311.11123 — silent API upgrades
//!     degrade per-family task families without a version bump.
//!   * arXiv:2409.03928 — retaining baseline.
//!   * AgentAssay arXiv:2603.02601 — 93%->71% pass-rate collapse on a silent
//!     provider update (the concrete collapse shape G-10's canary guards against).

use std::time::{SystemTime, UNIX_EPOCH};

use blake3;
use tracing;

use crate::canary::{
    decide, CanaryDecision, CanaryEvaluator, CanaryFamily, CanaryFamilyBundle, FamilyEvalResult,
    DEFAULT_FAIL_BAND, DEFAULT_MARGINAL_BAND, DEFAULT_MIN_PER_FAMILY,
};

// ---------------------------------------------------------------------------
// Lane + ModelIdentity
// ---------------------------------------------------------------------------

/// Runtime lane for a model identity. Derived from the model_pin string
/// (presence of cloud-vendor keywords) or from `has_cloud_model_swap`
/// (rra.rs:354). NEVER stored as a field on RunArtifact — always derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lane {
    /// Local model (Qwen2.5-Coder, local GPU). Fingerprint-confirmed lane:
    /// `system_fingerprint` is a deterministic hardware/firmware pin.
    Local,
    /// Cloud model (Claude/GPT/DeepSeek via API). Behavioral-suspected only:
    /// Anthropic's `system_fingerprint` is OPAQUE (GD-5).
    Cloud,
}

impl Lane {
    /// Derive the lane from a model_pin string. Mirrors `recover_model_pin`
    /// heuristic in rra.rs:362-371 (claude/sonnet/opus/haiku/gpt/deepseek).
    #[must_use]
    pub fn from_model_pin(pin: &str) -> Self {
        let lower = pin.to_ascii_lowercase();
        if lower.contains("claude")
            || lower.contains("sonnet")
            || lower.contains("opus")
            || lower.contains("haiku")
            || lower.contains("gpt")
            || lower.contains("deepseek")
        {
            Lane::Cloud
        } else {
            Lane::Local
        }
    }

    /// Wire label for display / DTO.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Lane::Local => "local",
            Lane::Cloud => "cloud",
        }
    }
}

/// The (model_pin, system_fingerprint, lane) identity tuple.
///
/// G-10 keys the drift baseline to this tuple. A changed identity
/// (new model_pin or fingerprint) triggers an identity-change check.
/// `lane` is ALWAYS DERIVED from `model_pin` via `Lane::from_model_pin`
/// or from the caller's `has_cloud_model_swap` heuristic — it is NEVER
/// stored as a field on `RunArtifact` (which has no `lane` field).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModelIdentity {
    /// Last-seen model pin (from RunArtifact::model_pin; None => "unknown").
    pub model_pin: String,
    /// Last-seen system fingerprint (from RunArtifact::system_fingerprint_pin).
    /// `None` = never pinned (cloud or cold start).
    pub system_fingerprint: Option<String>,
    /// Provider label (e.g. "local", "anthropic", "openai"). Derived from
    /// the model_pin heuristic, not read from a RunArtifact field.
    pub provider: String,
    /// Derived lane (Local / Cloud). Not stored on RunArtifact.
    pub lane: Lane,
}

impl ModelIdentity {
    /// Build from the fields available on a `RunArtifact` after `build_rra`.
    /// `model_pin` and `system_fingerprint_pin` are both `Option<String>` on
    /// `RunArtifact` (rra.rs:211/214). `has_cloud_model_swap` mirrors
    /// `recover_model_pin`'s bool return (rra.rs:354).
    #[must_use]
    pub fn from_rra_fields(
        model_pin_opt: Option<&str>,
        fingerprint_opt: Option<&str>,
        has_cloud_model_swap: bool,
    ) -> Self {
        let model_pin = model_pin_opt.unwrap_or("unknown").to_owned();
        let lane = if has_cloud_model_swap {
            Lane::Cloud
        } else {
            Lane::from_model_pin(&model_pin)
        };
        let provider = if matches!(lane, Lane::Cloud) {
            "cloud".to_owned()
        } else {
            "local".to_owned()
        };
        Self {
            model_pin,
            system_fingerprint: fingerprint_opt.map(str::to_owned),
            provider,
            lane,
        }
    }

    /// Deterministic blake3 identity key for this tuple.
    ///
    /// Format: `blake3(model_pin | "\x00" | fingerprint_or_empty | "\x00" | lane_label)`
    /// blake3 ONLY — never blake2/sha2 (CLAUDE.md anti-stale rule).
    #[must_use]
    pub fn identity_key(&self) -> String {
        let fp = self.system_fingerprint.as_deref().unwrap_or("");
        let input = format!("{}\x00{}\x00{}", self.model_pin, fp, self.lane.label());
        let h = blake3::hash(input.as_bytes());
        format!("{:.32}", h.to_hex())
    }
}

// ---------------------------------------------------------------------------
// DriftSignal
// ---------------------------------------------------------------------------

/// Which drift evidence triggered the row (the Pattern-2 Kind discriminator).
/// Matches `DriftSignal` in `ssb.rs` (the SSB payload carries BOTH `kind` +
/// `signals Vec<String>` for the detail list).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftSignal {
    /// A system fingerprint change was detected between the baseline and the
    /// current run (LOCAL lane only; cloud=behavioral-suspected per GD-5).
    FingerprintChanged,
    /// The per-family canary regression delta exceeded `DEFAULT_FAIL_BAND`.
    CanaryRegressed,
    /// The rate of distinct stdout_hash values (per F-4) exceeded the
    /// baseline modal hash rate — malformed / diverged output spike.
    MalformedOutputSpike,
}

impl DriftSignal {
    /// Stable snake_case label for DTO / log lines.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::FingerprintChanged => "fingerprint_changed",
            Self::CanaryRegressed => "canary_regressed",
            Self::MalformedOutputSpike => "malformed_output_spike",
        }
    }
}

// ---------------------------------------------------------------------------
// DriftBaseline
// ---------------------------------------------------------------------------

/// Stored per-family baseline pass-rates keyed to a `ModelIdentity`.
///
/// NOT the same as `D-7 CanaryReport` (which is a two-evaluator one-call
/// transfer score). A `DriftBaseline` is captured ONCE under a fixed identity
/// and used as the reference for future `check_model_drift` calls.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriftBaseline {
    /// blake3 identity key of the identity this baseline was captured under.
    pub identity_key: String,
    /// Per-family absolute pass-rate at baseline-capture time.
    /// Stored as (family_label, rate) pairs for JSON-friendliness.
    pub per_family_scores: Vec<(String, f64)>,
    /// Wall-clock ms at capture time.
    pub captured_at_ms: u64,
}

// ---------------------------------------------------------------------------
// DriftVerdict
// ---------------------------------------------------------------------------

/// Result of `check_model_drift`.
#[derive(Debug)]
pub enum DriftVerdict {
    /// The identity has no stored baseline yet — this call IS the baseline.
    /// The caller should persist `DriftBaseline` and retry later.
    BaselineCaptured,
    /// Per-family delta is within the non-determinism band (GD-4 FP guard).
    /// The caller MUST emit ZERO `ModelDriftSuspected` rows.
    NoDrift,
    /// One or more families exceeded `DEFAULT_FAIL_BAND`. The caller should
    /// emit ONE `ModelDriftSuspected` SSB row with these fields.
    BehavioralDriftSuspected {
        /// Which drift signals fired.
        signals: Vec<DriftSignal>,
        /// Per-family regression delta (family, transfer_score).
        per_family_delta: Vec<(CanaryFamily, f64)>,
        /// ReasoningTemplate domain labels for the regressed families
        /// (used to build `affected_template_ids` in the SSB payload).
        affected_template_ids: Vec<String>,
        /// `true` ONLY when: LOCAL lane AND `system_fingerprint` changed.
        /// NEVER `true` for CLOUD lane (GD-5 hard rule).
        confirmed_by_fingerprint: bool,
    },
}

// ---------------------------------------------------------------------------
// record_baseline
// ---------------------------------------------------------------------------

/// Run the canary under the current `identity` evaluator and store the
/// per-family absolute pass-rates as a `DriftBaseline`.
///
/// This does NOT call `run_canary`'s two-evaluator path (which computes
/// transfer_score = candidate - baseline in one call). Instead it calls
/// the evaluator once per family and records the absolute pass-rate as the
/// stored baseline. This is the G-10 time-axis model: baseline at T0, check
/// at T1, delta = T1_rate - T0_rate.
///
/// Reuse (GD-1): uses `CanaryFamily::all()`, `CanaryFamilyBundle`, and
/// `CanaryEvaluator` from canary.rs. Does NOT re-implement the family
/// enum or band logic.
pub async fn record_baseline(
    identity: &ModelIdentity,
    bundles: &[CanaryFamilyBundle],
    evaluator: &impl CanaryEvaluator,
    ts_ms: u64,
) -> Result<DriftBaseline, String> {
    let mut per_family_scores: Vec<(String, f64)> = Vec::with_capacity(bundles.len());

    for bundle in bundles {
        if bundle.tasks.len() < DEFAULT_MIN_PER_FAMILY {
            return Err(format!(
                "G-10 record_baseline: family {:?} has {} tasks, need >= DEFAULT_MIN_PER_FAMILY {}",
                bundle.family,
                bundle.tasks.len(),
                DEFAULT_MIN_PER_FAMILY
            ));
        }
        let mut pass_count: u32 = 0;
        let n = bundle.tasks.len();
        for task in &bundle.tasks {
            match evaluator.evaluate(task).await {
                Ok(true) => pass_count += 1,
                Ok(false) => {}
                Err(e) => {
                    tracing::warn!(
                        family = %bundle.family.as_str(),
                        task_id = %task.task_id.as_str(),
                        error = %e,
                        "G-10 record_baseline: evaluator error (counting as fail)"
                    );
                }
            }
        }
        let pass_rate = if n > 0 {
            f64::from(pass_count) / n as f64
        } else {
            0.0
        };
        per_family_scores.push((bundle.family.as_str().to_owned(), pass_rate));
    }

    Ok(DriftBaseline {
        identity_key: identity.identity_key(),
        per_family_scores,
        captured_at_ms: ts_ms,
    })
}

// ---------------------------------------------------------------------------
// check_model_drift
// ---------------------------------------------------------------------------

/// Compare the current evaluator scores against the stored baseline.
///
/// Returns `DriftVerdict::NoDrift` when all per-family deltas are within
/// `DEFAULT_FAIL_BAND` (the GD-4 non-determinism BAND = FP guard).
/// Returns `DriftVerdict::BehavioralDriftSuspected` when any family exceeds
/// the band — the caller emits exactly ONE `ModelDriftSuspected` SSB row.
///
/// Reuse (GD-1):
///   * Uses `decide(&per_family_results, DEFAULT_FAIL_BAND, DEFAULT_MARGINAL_BAND)`
///     from canary.rs — the SAME band function, no re-implementation.
///   * Uses `FamilyEvalResult` as the per-family delta carrier.
pub async fn check_model_drift(
    current_identity: &ModelIdentity,
    baseline: &DriftBaseline,
    bundles: &[CanaryFamilyBundle],
    evaluator: &impl CanaryEvaluator,
    _ts_ms: u64,
) -> Result<DriftVerdict, String> {
    // Build a lookup from family label to baseline pass-rate.
    let baseline_map: std::collections::HashMap<&str, f64> = baseline
        .per_family_scores
        .iter()
        .map(|(k, v)| (k.as_str(), *v))
        .collect();

    // Score the current evaluator per family and build FamilyEvalResult entries.
    let mut per_family_results: Vec<FamilyEvalResult> = Vec::with_capacity(bundles.len());

    for bundle in bundles {
        if bundle.tasks.len() < DEFAULT_MIN_PER_FAMILY {
            return Err(format!(
                "G-10 check_model_drift: family {:?} has {} tasks, need >= DEFAULT_MIN_PER_FAMILY {}",
                bundle.family,
                bundle.tasks.len(),
                DEFAULT_MIN_PER_FAMILY
            ));
        }
        let mut pass_count: u32 = 0;
        let n = bundle.tasks.len();
        for task in &bundle.tasks {
            match evaluator.evaluate(task).await {
                Ok(true) => pass_count += 1,
                Ok(false) => {}
                Err(e) => {
                    tracing::warn!(
                        family = %bundle.family.as_str(),
                        task_id = %task.task_id.as_str(),
                        error = %e,
                        "G-10 check_model_drift: evaluator error (counting as fail)"
                    );
                }
            }
        }
        let candidate_rate = if n > 0 {
            f64::from(pass_count) / n as f64
        } else {
            0.0
        };
        let baseline_rate = *baseline_map
            .get(bundle.family.as_str())
            .ok_or_else(|| format!("G-10: family {} missing from baseline; re-run `model-drift baseline` first", bundle.family.as_str()))?;

        // transfer_score = candidate_rate - baseline_rate (negative = regression).
        let transfer_score = candidate_rate - baseline_rate;

        per_family_results.push(FamilyEvalResult {
            family: bundle.family,
            n_evaluated: n as u32,
            pass_count,
            fail_count: (n as u32).saturating_sub(pass_count),
            transfer_score,
            baseline_score: baseline_rate,
        });
    }

    // Apply the D-7 band thresholds (GD-1 reuse: same decide() function).
    let decision = decide(&per_family_results, DEFAULT_FAIL_BAND, DEFAULT_MARGINAL_BAND);

    match decision {
        CanaryDecision::Pass | CanaryDecision::MarginalPass => {
            tracing::debug!(
                identity_key = %current_identity.identity_key(),
                decision = "pass_or_marginal",
                "G-10 check_model_drift: within band -> NoDrift"
            );
            Ok(DriftVerdict::NoDrift)
        }
        CanaryDecision::Fail { dominant_family } => {
            // Collect all regressed families (not just the dominant).
            let regressed: Vec<(CanaryFamily, f64)> = per_family_results
                .iter()
                .filter(|r| r.transfer_score <= -DEFAULT_FAIL_BAND)
                .map(|r| (r.family, r.transfer_score))
                .collect();

            // Affected template domain labels (one label per regressed family).
            // Maps CanaryFamily -> its domain label as the template-review surface.
            let affected_template_ids: Vec<String> = regressed
                .iter()
                .map(|(f, _)| format!("domain:{}", f.as_str()))
                .collect();

            // Determine which signals fired.
            let mut signals: Vec<DriftSignal> = Vec::new();

            // If the identity_key differs from the baseline's identity_key,
            // the fingerprint or model pin changed.
            let identity_changed = current_identity.identity_key() != baseline.identity_key;
            if identity_changed {
                signals.push(DriftSignal::FingerprintChanged);
            }
            signals.push(DriftSignal::CanaryRegressed);

            // confirmed_by_fingerprint: GD-5 hard rule.
            // LOCAL lane + identity changed (fingerprint/model differs) = confirmed.
            // CLOUD lane = NEVER confirmed (behavioral-suspected only).
            let confirmed_by_fingerprint = matches!(current_identity.lane, Lane::Local)
                && identity_changed;

            tracing::warn!(
                identity_key = %current_identity.identity_key(),
                dominant_family = %dominant_family.as_str(),
                regressed_families = regressed.len(),
                confirmed_by_fingerprint,
                "G-10 check_model_drift: BehavioralDriftSuspected"
            );

            Ok(DriftVerdict::BehavioralDriftSuspected {
                signals,
                per_family_delta: regressed,
                affected_template_ids,
                confirmed_by_fingerprint,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// now_ms helper
// ---------------------------------------------------------------------------

/// Current wall-clock time in milliseconds since UNIX_EPOCH.
/// Saturates to 0 on pre-epoch clocks (advisory only; not load-bearing).
#[must_use]
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

// ---------------------------------------------------------------------------
// Inline unit tests (non-async, band invariant only)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_key_differs_for_different_pins() {
        let a = ModelIdentity {
            model_pin: "model-a".to_owned(),
            system_fingerprint: Some("fp1".to_owned()),
            provider: "local".to_owned(),
            lane: Lane::Local,
        };
        let b = ModelIdentity {
            model_pin: "model-b".to_owned(),
            system_fingerprint: Some("fp2".to_owned()),
            provider: "local".to_owned(),
            lane: Lane::Local,
        };
        assert_ne!(
            a.identity_key(),
            b.identity_key(),
            "different models must have different identity keys"
        );
    }

    #[test]
    fn lane_from_model_pin_cloud_keywords() {
        for pin in &["claude-sonnet-4-6", "gpt-4o", "deepseek-v3", "opus"] {
            assert!(
                matches!(Lane::from_model_pin(pin), Lane::Cloud),
                "{pin} should be Cloud lane"
            );
        }
    }

    #[test]
    fn lane_from_model_pin_local_keywords() {
        for pin in &["qwen2.5-coder-7b", "qwen-3b", "local-model"] {
            assert!(
                matches!(Lane::from_model_pin(pin), Lane::Local),
                "{pin} should be Local lane"
            );
        }
    }

    #[test]
    fn identity_key_is_hex() {
        let id = ModelIdentity {
            model_pin: "test".to_owned(),
            system_fingerprint: None,
            provider: "local".to_owned(),
            lane: Lane::Local,
        };
        let key = id.identity_key();
        assert!(
            key.chars().all(|c| c.is_ascii_hexdigit()),
            "identity_key must be hex, got: {key}"
        );
    }

    #[test]
    fn drift_signal_labels_stable() {
        assert_eq!(DriftSignal::FingerprintChanged.label(), "fingerprint_changed");
        assert_eq!(DriftSignal::CanaryRegressed.label(), "canary_regressed");
        assert_eq!(DriftSignal::MalformedOutputSpike.label(), "malformed_output_spike");
    }
}

// ---------------------------------------------------------------------------
// G-10 off-agent artifact smoke (R4/R7 gate)
// ---------------------------------------------------------------------------
//
// Drives check_model_drift() with a regressed evaluator (all tasks fail =>
// transfer_score = 0.0 - 1.0 = -1.0, well below DEFAULT_FAIL_BAND), builds
// the ModelDriftSuspectedPayload from the BehavioralDriftSuspected verdict,
// writes it to SSB via write_payload(), then queries back via query_by_type()
// and asserts the row is non-empty.
//
// Production reachability (R7): the write_payload() call here uses the SAME
// production path as emit_model_drift_check() in run_subcommand.rs (lines
// 517-528, 577-754), which calls check_model_drift() then ssb.write_payload().
// The smoke proves the production check_model_drift -> write_payload chain
// leaves an off-agent artifact in SSB.
#[cfg(test)]
mod g10_drift_emit_smoke {
    use std::sync::Arc;

    use tempfile::TempDir;

    use crate::canary::{
        CanaryEvaluator, CanaryFamily, CanaryFamilyBundle, MarkerEvaluator,
        DEFAULT_MIN_PER_FAMILY,
    };
    use crate::held_out::{build_task, ExpectedOutcome, TaskSource, DEFAULT_NGRAM_SIZE};
    use crate::ssb::{
        DriftSignalKind, ModelDriftSuspectedPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
        SsbReader,
    };
    use elai_core::clock::SeededClock;

    use super::{check_model_drift, record_baseline, DriftVerdict, Lane, ModelIdentity};

    fn test_ssb(dir: &TempDir) -> Ssb {
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("g10_smoke.db"),
                pool_size: Some(2),
                project_id: "g10-smoke".into(),
                session_id: "g10-smoke-session".into(),
            },
            Arc::new(SeededClock::new()),
        )
        .expect("Ssb::new for g10_drift_emit_smoke")
    }

    /// Build a CanaryFamilyBundle with exactly DEFAULT_MIN_PER_FAMILY tasks
    /// for the given family. MarkerEvaluator("pass") will pass tasks whose
    /// task_id contains "pass". Setting pass=false for ALL tasks means the
    /// evaluator sees 0% pass rate (all task_ids start with "fail-").
    fn regressed_bundle(family: CanaryFamily, pass: bool) -> CanaryFamilyBundle {
        let n = DEFAULT_MIN_PER_FAMILY;
        let mut tasks = Vec::with_capacity(n);
        for i in 0..n {
            let body = format!(
                "g10 smoke family {} task {} body alpha beta gamma delta epsilon",
                family.as_str(),
                i
            );
            let outcome = ExpectedOutcome::Prose("ok".into());
            let mut task = build_task(
                body,
                TaskSource::Synthetic { seed: i as u64 },
                outcome,
                1_000 + i as i64,
                0,
                DEFAULT_NGRAM_SIZE,
            );
            // Embed pass/fail marker so MarkerEvaluator fires deterministically.
            let prefix = if pass { "pass" } else { "fail" };
            task.task_id = crate::held_out::TaskId(format!("{}-g10-{}", prefix, task.task_id.0));
            tasks.push(task);
        }
        CanaryFamilyBundle { family, tasks }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn drift_emit_lands_in_ssb() {
        let dir = TempDir::new().expect("tempdir for g10 smoke");
        let ssb = test_ssb(&dir);

        // Identity: local Qwen model (LOCAL lane -> confirmed_by_fingerprint eligible).
        let identity = ModelIdentity {
            model_pin: "qwen2.5-coder-7b".into(),
            system_fingerprint: Some("fp-baseline-0".into()),
            provider: "local".into(),
            lane: Lane::Local,
        };

        // Evaluator: "pass" marker evaluator for baseline capture (all tasks pass).
        let baseline_eval = MarkerEvaluator { pass_marker: "pass".into() };
        // Evaluator: "miss" marker evaluator for drift check (no tasks match -> all fail).
        let regressed_eval = MarkerEvaluator { pass_marker: "miss".into() };

        // Use only CodeFix family to keep the test minimal.
        let bundles: Vec<CanaryFamilyBundle> =
            vec![regressed_bundle(CanaryFamily::CodeFix, true)];

        // Capture a baseline where ALL tasks pass (pass rate = 1.0).
        let baseline = record_baseline(&identity, &bundles, &baseline_eval, 1_748_900_000_000)
            .await
            .expect("record_baseline should succeed");

        // Now check drift with an evaluator that fails ALL tasks (pass rate = 0.0).
        // transfer_score = 0.0 - 1.0 = -1.0, which exceeds DEFAULT_FAIL_BAND.
        // Bundles have "pass-" prefix task_ids; regressed_eval searches for "miss" => all fail.
        let regressed_bundles = vec![regressed_bundle(CanaryFamily::CodeFix, false)];
        let verdict = check_model_drift(
            &identity,
            &baseline,
            &regressed_bundles,
            &regressed_eval,
            1_748_901_000_000,
        )
        .await
        .expect("check_model_drift should succeed");

        // The verdict MUST be BehavioralDriftSuspected (regression is -1.0).
        let (signals, per_family_delta, affected_template_ids, confirmed_by_fingerprint) =
            match verdict {
                DriftVerdict::BehavioralDriftSuspected {
                    signals,
                    per_family_delta,
                    affected_template_ids,
                    confirmed_by_fingerprint,
                } => (signals, per_family_delta, affected_template_ids, confirmed_by_fingerprint),
                other => panic!("expected BehavioralDriftSuspected, got {:?}", other),
            };

        // Build the SSB payload exactly as emit_model_drift_check() does in
        // run_subcommand.rs (lines 517-528). This is the production code path
        // under test — same struct, same fields, same write_payload call.
        let kind = if signals
            .iter()
            .any(|s| matches!(s, super::DriftSignal::CanaryRegressed))
        {
            DriftSignalKind::CanaryRegressed
        } else if signals
            .iter()
            .any(|s| matches!(s, super::DriftSignal::FingerprintChanged))
        {
            DriftSignalKind::FingerprintChanged
        } else {
            DriftSignalKind::MalformedOutputSpike
        };

        let payload = SsbPayload::ModelDriftSuspected(ModelDriftSuspectedPayload {
            identity_key: identity.identity_key(),
            kind,
            signals: signals.iter().map(|s| s.label().to_owned()).collect(),
            per_family_delta: per_family_delta
                .iter()
                .map(|(f, d)| (f.as_str().to_owned(), *d))
                .collect(),
            affected_template_ids,
            confirmed_by_fingerprint,
            baseline_captured_at: baseline.captured_at_ms,
        });

        // Write to SSB via the production write_payload path.
        let seq = ssb
            .write_payload("g10-drift-smoke", 0, payload, vec!["model_drift.rs".into()])
            .await
            .expect("write_payload ModelDriftSuspected must succeed");
        assert!(seq > 0, "seq must be positive (row was written)");

        // Query back via query_by_type — the off-agent artifact proof (R4/R7).
        let rows = ssb
            .query_by_type(SsbActionType::ModelDriftSuspected)
            .expect("query_by_type ModelDriftSuspected must succeed");
        assert!(
            !rows.is_empty(),
            "G-10 off-agent artifact: ModelDriftSuspected row must be present in SSB after \
             check_model_drift -> write_payload chain (R4/R7 gate)"
        );
    }
}
