//! D-18 — Typed ObservableEffect predicate (Mode D upgrade).
//!
//! Plan: docs/thefinalglance/D-plan/D-18_observable_effect_typed.md.
//!
//! Scope (this pass, narrower than the plan's 2-session full target):
//!   * Typed `ObservableEffect { effect_id, trigger, expected, sandbox,
//!     confound_flags }` with closed `TriggerSpec` + `TypedObservation` +
//!     `SandboxTier` enums and a bool-field `ConfoundFlags` struct (no
//!     external `bitflags` newtype — `bitflags 2.6` IS in the orchestrator
//!     Cargo.toml but a plain struct keeps the wire form msgpack-friendly
//!     and matches the closed-enum discipline used by [`CheckerKind`] and
//!     [`SandboxTier`] above).
//!   * [`SandboxExecutor`] trait (native async-fn-in-trait, NO
//!     `async-trait` crate; see CLAUDE.md anti-stale-training) +
//!     [`MockSandboxExecutor`] for tests.
//!   * [`verify_observable_effect`] runs the trigger against the executor,
//!     compares the captured observation to the expected value, and emits
//!     one [`SsbActionType::ObservableEffectVerified`] SSB row per call.
//!
//! Out-of-scope (deferred — full plan requires a second session):
//!   * `EffectSet` + ≥2-effect orthogonality validator.
//!   * `ObservationContext::Sandbox|Production` labeling + operator gate.
//!   * `check_goodhart_risk` single-effect flag.
//!   * `FeasibilityProber.probe` Vec<ObservationResult> return-type
//!     extension (touches `feasibility_probe.rs` which is C-22 surface).
//!   * `[observable_effect]` elai_cc.toml section + L3 lock.
//!   * 4-variant SSB pack (`ObservableEffectDispatched` +
//!     `GoodhartRiskFlag` + `SandboxObservation` +
//!     `ProductionObservationProposal`). This pass ships ONE variant
//!     (`ObservableEffectVerified`) sufficient for the smoke test.
//!
//! Producer wiring:
//!   * [`verify_observable_effect`] is the sole writer of the new
//!     `observable_effect_verified` SSB action_type. The Tauri command
//!     `commands_d18::get_observable_effects_recent` is the read-side
//!     consumer (`.elai_cc/crates/dashboard-v2/src-tauri/src`).
//!
//! Citation: the orchestrator brief asked for an arXiv:2412.16367 cite
//! against typed-test predicates. WebSearch (May 2026) shows
//! arXiv:2412.16367 is "A Layered Swarm Optimization Method for Fitting
//! Battery Thermal Runaway Models" (Bhatnagar et al., 2024-12-20) —
//! unrelated to typed-test predicates. Per CLAUDE.md "Verify paper
//! citations" we do NOT cite it. TODO_DOC[D-18]: paper-ingestor follow-up
//! to ground the typed-effect / sandbox-observation discipline (candidate
//! anchors: Voyager arxiv:2305.16291 §3 skill-with-measurable-effect;
//! Coscientist arxiv:2304.05332 §4 sandbox vs production observation).

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

use crate::ssb::{ObservableEffectVerifiedPayload, Ssb, SsbError, SsbPayload};

// ---------------------------------------------------------------------------
// Typed surface (closed enums)
// ---------------------------------------------------------------------------

/// Stable identifier of an [`ObservableEffect`]. Snake-case ASCII, ≤128
/// bytes — matches the wire form the dashboard surfaces.
pub type EffectId = String;

/// Closed taxonomy of triggers the orchestrator can dispatch against an
/// effect. Frozen per D-9 locked-keys discipline — adding a variant is a
/// plan-level schema change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TriggerSpec {
    /// Run a shell command. The string is the command line as the test
    /// would write it (`"cargo test --quiet -p elai_orchestrator"`).
    /// Wire form intentionally string-typed — the executor splits.
    TestCommand(String),
    /// Assert a file exists at the resolved path (no content check).
    FilePresence(PathBuf),
    /// HTTP GET against a URL with an expected status code.
    HttpEndpointGet { url: String, expected_status: u16 },
    /// File contents matched against an anchored regex. The regex is
    /// compiled at verify-time; an invalid regex returns
    /// [`VerificationOutcome::Skipped`].
    FileContents { path: PathBuf, regex: String },
    /// Stdout of the trigger command contains the substring.
    StdoutContains(String),
}

impl TriggerSpec {
    /// Stable snake-case wire label used in SSB rows + dashboard
    /// histograms. Matches the serde tag.
    #[must_use]
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::TestCommand(_) => "test_command",
            Self::FilePresence(_) => "file_presence",
            Self::HttpEndpointGet { .. } => "http_endpoint_get",
            Self::FileContents { .. } => "file_contents",
            Self::StdoutContains(_) => "stdout_contains",
        }
    }
}

/// Closed taxonomy of typed observations the executor can produce. Each
/// variant pairs with a subset of [`TriggerSpec`] kinds — see
/// [`verify_observable_effect`] for the matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypedObservation {
    /// Expected stdout / matched substring.
    StringMatch(String),
    /// Expected file content authenticated by blake3 hash. The hash is
    /// the lowercase hex digest (`blake3::Hash::to_hex().as_str()`).
    FileContent {
        path: PathBuf,
        expected_blake3: String,
    },
    /// Structural JSON match — top-level keys + types must equal those
    /// of the expected `serde_json::Value`. Values are compared by
    /// `==`; nested shapes recurse.
    JsonShape(serde_json::Value),
    /// HTTP response status code.
    HttpStatus(u16),
    /// Subprocess exit code.
    ExitCode(i32),
}

impl TypedObservation {
    /// Stable snake-case wire label used in SSB rows.
    #[must_use]
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::StringMatch(_) => "string_match",
            Self::FileContent { .. } => "file_content",
            Self::JsonShape(_) => "json_shape",
            Self::HttpStatus(_) => "http_status",
            Self::ExitCode(_) => "exit_code",
        }
    }
}

/// Sandbox isolation tier the effect runs under. Matches the
/// B-18 + C-11 sandbox tiers semantically (NoSandbox = bare process,
/// ContainerIsolated ≈ Tier C, NetworkBlocked ≈ Tier C + net-deny).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxTier {
    /// Bare host process. Smoke / dev only.
    NoSandbox,
    /// OS-process isolation (cwd jail + env scrub) — matches B-18 Tier B.
    ProcessIsolated,
    /// Container / namespace isolation — matches B-18 Tier C.
    ContainerIsolated,
    /// Container + net-deny — matches C-11 Tier C with networking off.
    NetworkBlocked,
}

impl SandboxTier {
    /// Stable snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoSandbox => "no_sandbox",
            Self::ProcessIsolated => "process_isolated",
            Self::ContainerIsolated => "container_isolated",
            Self::NetworkBlocked => "network_blocked",
        }
    }
}

/// Bool-field bitflags surface for the known confound risks. We do NOT
/// reach for the `bitflags` crate macro here even though it's in the
/// orchestrator Cargo.toml — the four flags need to round-trip through
/// msgpack as plain JSON-ish bools, and a tagged struct keeps the SSB
/// payload schema human-readable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfoundFlags {
    /// Result depends on environment variables / shell state.
    pub environment_dependent: bool,
    /// Result depends on wall-clock time (rate-limits, expiry, etc).
    pub time_dependent: bool,
    /// Result depends on the order other effects ran in.
    pub order_dependent: bool,
    /// Result is nondeterministic even with everything else fixed
    /// (RNG-driven, parallel scheduling, network jitter).
    pub nondeterministic: bool,
}

impl ConfoundFlags {
    /// True iff any flag is set.
    #[must_use]
    pub fn any(&self) -> bool {
        self.environment_dependent
            || self.time_dependent
            || self.order_dependent
            || self.nondeterministic
    }

    /// Comma-separated snake-case label list. Used in SSB payload + the
    /// dashboard's tag chips. Empty string when no flag is set.
    #[must_use]
    pub fn as_labels(&self) -> String {
        let mut out: Vec<&'static str> = Vec::new();
        if self.environment_dependent {
            out.push("environment_dependent");
        }
        if self.time_dependent {
            out.push("time_dependent");
        }
        if self.order_dependent {
            out.push("order_dependent");
        }
        if self.nondeterministic {
            out.push("nondeterministic");
        }
        out.join(",")
    }
}

/// One typed observable effect — trigger + expected typed observation +
/// sandbox tier + confound annotations.
///
/// `effect_id` MUST be stable per dispatch so the dashboard can join
/// successive runs against the same effect (`time_dependent` panels
/// rely on this).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservableEffect {
    pub effect_id: EffectId,
    pub trigger: TriggerSpec,
    pub expected: TypedObservation,
    pub sandbox: SandboxTier,
    pub confound_flags: ConfoundFlags,
}

// ---------------------------------------------------------------------------
// SandboxExecutor trait + mock
// ---------------------------------------------------------------------------

/// Raw observation an executor returns. Each variant pairs with one or
/// more [`TypedObservation`] kinds; [`verify_observable_effect`] does
/// the typed comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapturedObservation {
    Stdout(String),
    ExitCode(i32),
    /// Raw bytes for blake3-hash comparison.
    FileBytes(Vec<u8>),
    /// File exists at the expected path.
    FilePresent(bool),
    HttpStatus(u16),
    Json(serde_json::Value),
}

/// Errors a sandbox executor can return. The verifier maps these to
/// [`VerificationOutcome::Skipped`] (executor-side problem — not a
/// product failure).
#[derive(Debug, Error)]
pub enum SandboxExecutorError {
    #[error("trigger kind {0} not supported by this executor")]
    UnsupportedTrigger(&'static str),
    #[error("io: {0}")]
    Io(String),
    #[error("regex: {0}")]
    Regex(String),
    #[error("other: {0}")]
    Other(String),
}

/// Native async-fn-in-trait per CLAUDE.md anti-stale-training. Dyn calls
/// require `Pin<Box<dyn Future>>` erasure (see [`crate::checker_registry`]);
/// the verifier takes `&dyn SandboxExecutor` so we use the same boxed-
/// future pattern.
pub trait SandboxExecutor: Send + Sync {
    /// Execute the trigger under the requested sandbox tier and return
    /// the captured observation. Implementations MUST honor the tier
    /// (NoSandbox = bare run; ProcessIsolated and above = sandbox-on).
    fn run<'a>(
        &'a self,
        trigger: &'a TriggerSpec,
        tier: SandboxTier,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<CapturedObservation, SandboxExecutorError>>
                + Send
                + 'a,
        >,
    >;
}

/// In-memory mock for the smoke test. Pre-loads a deterministic map of
/// `TriggerSpec → CapturedObservation` keyed by the trigger's
/// `kind_str()` + a per-effect distinguisher.
#[derive(Debug, Default)]
pub struct MockSandboxExecutor {
    /// (effect distinguisher, trigger kind) → captured observation.
    /// Distinguisher is the trigger's primary payload (e.g. the test
    /// command string, the file path's display form). Keeps the mock
    /// honest — same trigger always returns the same observation in a
    /// single test run.
    fixtures: HashMap<(String, &'static str), CapturedObservation>,
}

impl MockSandboxExecutor {
    /// Construct an empty mock. Tests register fixtures via [`Self::insert`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            fixtures: HashMap::new(),
        }
    }

    /// Register the captured observation for a (distinguisher,
    /// trigger-kind) pair. Distinguisher is the trigger's primary
    /// payload (command line / path display / url / substring).
    pub fn insert(
        &mut self,
        distinguisher: impl Into<String>,
        kind: &'static str,
        observation: CapturedObservation,
    ) {
        self.fixtures.insert((distinguisher.into(), kind), observation);
    }

    fn distinguisher_for(trigger: &TriggerSpec) -> String {
        match trigger {
            TriggerSpec::TestCommand(cmd) => cmd.clone(),
            TriggerSpec::FilePresence(p) => p.display().to_string(),
            TriggerSpec::HttpEndpointGet { url, .. } => url.clone(),
            TriggerSpec::FileContents { path, .. } => path.display().to_string(),
            TriggerSpec::StdoutContains(s) => s.clone(),
        }
    }
}

impl SandboxExecutor for MockSandboxExecutor {
    fn run<'a>(
        &'a self,
        trigger: &'a TriggerSpec,
        _tier: SandboxTier,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<CapturedObservation, SandboxExecutorError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let key = (Self::distinguisher_for(trigger), trigger.kind_str());
            self.fixtures.get(&key).cloned().ok_or_else(|| {
                SandboxExecutorError::Other(format!(
                    "mock fixture missing for ({}, {})",
                    key.0, key.1
                ))
            })
        })
    }
}

// ---------------------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------------------

/// Outcome of [`verify_observable_effect`]. Wire-form mirrors the
/// snake-case label set the dashboard renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationOutcome {
    /// Captured observation matched the typed expectation.
    Pass,
    /// Captured observation differed from the typed expectation.
    Fail,
    /// Executor refused (unsupported trigger), trigger/observation type
    /// mismatch, or regex compile error. NOT a product failure.
    Skipped,
}

impl VerificationOutcome {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Skipped => "skipped",
        }
    }
}

/// Verify the effect's trigger output against its typed expected
/// observation. Emits one `observable_effect_verified` SSB row per call.
///
/// Pairing matrix (trigger kind → allowed observation kinds):
///   * `test_command`        → `stdout` via `string_match` OR `exit_code`
///   * `file_presence`       → `string_match("present")` (sentinel)
///   * `http_endpoint_get`   → `http_status`
///   * `file_contents`       → `file_content` (blake3 hash) OR `string_match`
///                              (regex match yields "matched"/"no_match")
///   * `stdout_contains`     → `string_match`
///
/// Mismatched pairings return `Skipped` (the SSB row carries
/// `outcome=skipped` so dashboards can surface mis-authored effects).
pub async fn verify_observable_effect(
    effect: &ObservableEffect,
    executor: &dyn SandboxExecutor,
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    now_ms: i64,
) -> Result<VerificationOutcome, SsbError> {
    let started = Instant::now();
    let outcome = run_and_compare(effect, executor).await;
    let elapsed_us = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);

    let payload = ObservableEffectVerifiedPayload {
        effect_id: effect.effect_id.clone(),
        trigger_kind: effect.trigger.kind_str().to_owned(),
        observation_kind: effect.expected.kind_str().to_owned(),
        sandbox_tier: effect.sandbox.as_str().to_owned(),
        outcome: outcome.as_str().to_owned(),
        confound_flags: effect.confound_flags.as_labels(),
        elapsed_us,
        ts_ms: now_ms,
    };
    if let Err(e) = ssb
        .write_payload(
            agent_id,
            superstep,
            SsbPayload::ObservableEffectVerified(payload),
            Vec::new(),
        )
        .await
    {
        warn!(
            effect_id = %effect.effect_id,
            outcome = outcome.as_str(),
            error = %e,
            "observable_effect: SSB emission failed (verification result still applies)"
        );
        return Err(e);
    }
    Ok(outcome)
}

/// Gating helper consumed by Mode D dispatch: verify a slice of
/// effects in declaration order and return the aggregated outcome.
///
/// Aggregation rule (defensive — single failure trumps a sea of
/// passes; single pass does not redeem a sea of skips):
///   * any [`VerificationOutcome::Fail`] in the slice → returns
///     `(Fail, outcomes)` immediately and STOPS dispatch (further
///     effects are NOT run).
///   * else if every outcome is [`VerificationOutcome::Pass`] →
///     `(Pass, outcomes)`.
///   * else (mix of Pass + Skipped, or all Skipped) →
///     `(Skipped, outcomes)`.
///
/// One `observable_effect_verified` SSB row per effect that ran. Effects
/// AFTER a Fail are NOT emitted (no row for skipped-due-to-prior-fail).
/// Empty slice returns `(Skipped, vec![])` and emits NO rows.
pub async fn verify_effect_batch(
    effects: &[ObservableEffect],
    executor: &dyn SandboxExecutor,
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    now_ms: i64,
) -> Result<(VerificationOutcome, Vec<VerificationOutcome>), SsbError> {
    let mut outcomes = Vec::with_capacity(effects.len());
    let mut saw_skipped = false;
    let mut saw_pass = false;
    // `now_ms` is shared across the batch; deterministic for the smoke
    // test. Callers wanting per-effect timestamps can call
    // `verify_observable_effect` directly in a loop.
    for effect in effects {
        let o = verify_observable_effect(effect, executor, ssb, agent_id, superstep, now_ms)
            .await?;
        outcomes.push(o);
        match o {
            VerificationOutcome::Fail => {
                // Short-circuit: stop running further effects on first fail.
                return Ok((VerificationOutcome::Fail, outcomes));
            }
            VerificationOutcome::Pass => saw_pass = true,
            VerificationOutcome::Skipped => saw_skipped = true,
        }
    }
    if outcomes.is_empty() {
        return Ok((VerificationOutcome::Skipped, outcomes));
    }
    let aggregated = if saw_skipped {
        VerificationOutcome::Skipped
    } else if saw_pass {
        VerificationOutcome::Pass
    } else {
        VerificationOutcome::Skipped
    };
    Ok((aggregated, outcomes))
}

async fn run_and_compare(
    effect: &ObservableEffect,
    executor: &dyn SandboxExecutor,
) -> VerificationOutcome {
    let captured = match executor.run(&effect.trigger, effect.sandbox).await {
        Ok(c) => c,
        Err(e) => {
            warn!(
                effect_id = %effect.effect_id,
                trigger = effect.trigger.kind_str(),
                error = %e,
                "observable_effect: executor error, returning Skipped"
            );
            return VerificationOutcome::Skipped;
        }
    };

    compare_observation(&captured, &effect.expected, &effect.trigger)
}

fn compare_observation(
    captured: &CapturedObservation,
    expected: &TypedObservation,
    trigger: &TriggerSpec,
) -> VerificationOutcome {
    use CapturedObservation as C;
    use TypedObservation as E;

    match (captured, expected, trigger) {
        // test_command → string_match against stdout.
        (C::Stdout(s), E::StringMatch(target), TriggerSpec::TestCommand(_))
        | (C::Stdout(s), E::StringMatch(target), TriggerSpec::StdoutContains(_)) => {
            if s.contains(target.as_str()) {
                VerificationOutcome::Pass
            } else {
                VerificationOutcome::Fail
            }
        }
        // test_command → exit_code.
        (C::ExitCode(c), E::ExitCode(want), TriggerSpec::TestCommand(_)) => {
            if c == want {
                VerificationOutcome::Pass
            } else {
                VerificationOutcome::Fail
            }
        }
        // file_presence → string_match("present").
        (C::FilePresent(true), E::StringMatch(target), TriggerSpec::FilePresence(_)) => {
            if target == "present" {
                VerificationOutcome::Pass
            } else {
                VerificationOutcome::Fail
            }
        }
        (C::FilePresent(false), E::StringMatch(target), TriggerSpec::FilePresence(_)) => {
            if target == "absent" {
                VerificationOutcome::Pass
            } else {
                VerificationOutcome::Fail
            }
        }
        // file_contents → file_content (blake3).
        (
            C::FileBytes(bytes),
            E::FileContent {
                path: _,
                expected_blake3,
            },
            TriggerSpec::FileContents { .. },
        ) => {
            let got = blake3::hash(bytes).to_hex().to_string();
            if got == *expected_blake3 {
                VerificationOutcome::Pass
            } else {
                VerificationOutcome::Fail
            }
        }
        // file_contents → string_match via regex.
        (
            C::FileBytes(bytes),
            E::StringMatch(_),
            TriggerSpec::FileContents { regex, .. },
        ) => {
            let body = match std::str::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => return VerificationOutcome::Skipped,
            };
            match regex::Regex::new(regex) {
                Ok(re) => {
                    if re.is_match(body) {
                        VerificationOutcome::Pass
                    } else {
                        VerificationOutcome::Fail
                    }
                }
                Err(e) => {
                    warn!(
                        regex = %regex,
                        error = %e,
                        "observable_effect: regex compile failure, returning Skipped"
                    );
                    VerificationOutcome::Skipped
                }
            }
        }
        // http_endpoint_get → http_status.
        (
            C::HttpStatus(got),
            E::HttpStatus(want),
            TriggerSpec::HttpEndpointGet { .. },
        ) => {
            if got == want {
                VerificationOutcome::Pass
            } else {
                VerificationOutcome::Fail
            }
        }
        // Anything else is a mis-authored pairing.
        _ => VerificationOutcome::Skipped,
    }
}

// ---------------------------------------------------------------------------
// Read surface (dashboard)
// ---------------------------------------------------------------------------

/// Typed projection of a single `observable_effect_verified` SSB row —
/// what the Tauri command `get_observable_effects_recent` returns and
/// the Svelte `ObservableEffectPanel` renders.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservableEffectRow {
    pub effect_id: String,
    pub trigger_kind: String,
    pub observation_kind: String,
    pub sandbox_tier: String,
    pub outcome: String,
    pub confound_flags: String,
    pub elapsed_us: u64,
    pub ts_ms: i64,
}

/// D-18 read surface — the most recent N `observable_effect_verified`
/// SSB rows, newest-first. Mirrors
/// [`crate::conformal_calibration::recent_tau_rows`] semantically.
#[must_use]
pub fn recent_observable_effects(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> Vec<ObservableEffectRow> {
    let mut out = Vec::new();
    match viewer.recent_rows_of_type("observable_effect_verified", limit) {
        Ok(details) => {
            for d in details {
                if let Some(row) = observable_effect_detail_to_row(d) {
                    out.push(row);
                }
            }
        }
        Err(e) => {
            warn!(
                action_type = "observable_effect_verified",
                error = %e,
                "recent_observable_effects: viewer error"
            );
        }
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(limit);
    out
}

fn observable_effect_detail_to_row(
    d: crate::ssb_viewer::SsbActionDetail,
) -> Option<ObservableEffectRow> {
    if d.action_type.as_str() != "observable_effect_verified" {
        return None;
    }
    let outer = d.payload_json.as_object()?;
    let obj = if let Some(inner) = outer
        .get("ObservableEffectVerified")
        .and_then(|v| v.as_object())
    {
        inner.clone()
    } else {
        outer.clone()
    };
    Some(ObservableEffectRow {
        effect_id: obj
            .get("effect_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        trigger_kind: obj
            .get("trigger_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        observation_kind: obj
            .get("observation_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        sandbox_tier: obj
            .get("sandbox_tier")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        outcome: obj
            .get("outcome")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        confound_flags: obj
            .get("confound_flags")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        elapsed_us: obj.get("elapsed_us").and_then(|v| v.as_u64()).unwrap_or(0),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;
    use elai_core::ClockProvider;
    use std::sync::Arc;
    use tempfile::TempDir;

    use crate::ssb::{Ssb, SsbConfig};

    fn test_ssb(dir: &TempDir) -> Ssb {
        let cfg = SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "d18-test".into(),
            session_id: "d18-session".into(),
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        Ssb::new(cfg, clock).expect("ssb init")
    }

    const TEST_NOW_MS: i64 = 1_700_000_000_000;

    #[test]
    fn trigger_kind_str_stable() {
        assert_eq!(
            TriggerSpec::TestCommand("x".into()).kind_str(),
            "test_command"
        );
        assert_eq!(
            TriggerSpec::FilePresence(PathBuf::from("/x")).kind_str(),
            "file_presence"
        );
        assert_eq!(
            TriggerSpec::HttpEndpointGet {
                url: "u".into(),
                expected_status: 200,
            }
            .kind_str(),
            "http_endpoint_get"
        );
        assert_eq!(
            TriggerSpec::FileContents {
                path: PathBuf::from("/x"),
                regex: ".*".into(),
            }
            .kind_str(),
            "file_contents"
        );
        assert_eq!(
            TriggerSpec::StdoutContains("x".into()).kind_str(),
            "stdout_contains"
        );
    }

    #[test]
    fn observation_kind_str_stable() {
        assert_eq!(TypedObservation::StringMatch("x".into()).kind_str(), "string_match");
        assert_eq!(
            TypedObservation::FileContent {
                path: PathBuf::from("/x"),
                expected_blake3: "abc".into(),
            }
            .kind_str(),
            "file_content"
        );
        assert_eq!(
            TypedObservation::JsonShape(serde_json::json!({})).kind_str(),
            "json_shape"
        );
        assert_eq!(TypedObservation::HttpStatus(200).kind_str(), "http_status");
        assert_eq!(TypedObservation::ExitCode(0).kind_str(), "exit_code");
    }

    #[test]
    fn sandbox_tier_labels_stable() {
        assert_eq!(SandboxTier::NoSandbox.as_str(), "no_sandbox");
        assert_eq!(SandboxTier::ProcessIsolated.as_str(), "process_isolated");
        assert_eq!(SandboxTier::ContainerIsolated.as_str(), "container_isolated");
        assert_eq!(SandboxTier::NetworkBlocked.as_str(), "network_blocked");
    }

    #[test]
    fn confound_flags_labels_sorted_and_skipping() {
        let f = ConfoundFlags {
            environment_dependent: true,
            time_dependent: false,
            order_dependent: true,
            nondeterministic: false,
        };
        assert!(f.any());
        assert_eq!(f.as_labels(), "environment_dependent,order_dependent");
        let empty = ConfoundFlags::default();
        assert!(!empty.any());
        assert_eq!(empty.as_labels(), "");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn verify_test_command_string_match_passes() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let mut mock = MockSandboxExecutor::new();
        mock.insert(
            "echo hello",
            "test_command",
            CapturedObservation::Stdout("hello world".into()),
        );
        let effect = ObservableEffect {
            effect_id: "e-1".into(),
            trigger: TriggerSpec::TestCommand("echo hello".into()),
            expected: TypedObservation::StringMatch("hello".into()),
            sandbox: SandboxTier::NoSandbox,
            confound_flags: ConfoundFlags::default(),
        };
        let outcome = verify_observable_effect(&effect, &mock, &ssb, "agent-d18", 0, TEST_NOW_MS)
            .await
            .expect("verify");
        assert_eq!(outcome, VerificationOutcome::Pass);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn verify_file_presence_passes() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let mut mock = MockSandboxExecutor::new();
        let target = PathBuf::from("/tmp/x.txt");
        mock.insert(
            target.display().to_string(),
            "file_presence",
            CapturedObservation::FilePresent(true),
        );
        let effect = ObservableEffect {
            effect_id: "e-2".into(),
            trigger: TriggerSpec::FilePresence(target),
            expected: TypedObservation::StringMatch("present".into()),
            sandbox: SandboxTier::ProcessIsolated,
            confound_flags: ConfoundFlags {
                environment_dependent: true,
                ..ConfoundFlags::default()
            },
        };
        let outcome = verify_observable_effect(&effect, &mock, &ssb, "agent-d18", 0, TEST_NOW_MS)
            .await
            .expect("verify");
        assert_eq!(outcome, VerificationOutcome::Pass);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn verify_http_status_fail_on_mismatch() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let mut mock = MockSandboxExecutor::new();
        mock.insert(
            "http://x",
            "http_endpoint_get",
            CapturedObservation::HttpStatus(500),
        );
        let effect = ObservableEffect {
            effect_id: "e-3".into(),
            trigger: TriggerSpec::HttpEndpointGet {
                url: "http://x".into(),
                expected_status: 200,
            },
            expected: TypedObservation::HttpStatus(200),
            sandbox: SandboxTier::NetworkBlocked,
            confound_flags: ConfoundFlags::default(),
        };
        let outcome = verify_observable_effect(&effect, &mock, &ssb, "agent-d18", 0, TEST_NOW_MS)
            .await
            .expect("verify");
        assert_eq!(outcome, VerificationOutcome::Fail);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn verify_blake3_file_content_passes() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let bytes = b"hello-d18".to_vec();
        let want = blake3::hash(&bytes).to_hex().to_string();
        let mut mock = MockSandboxExecutor::new();
        let path = PathBuf::from("/tmp/y.txt");
        mock.insert(
            path.display().to_string(),
            "file_contents",
            CapturedObservation::FileBytes(bytes),
        );
        let effect = ObservableEffect {
            effect_id: "e-4".into(),
            trigger: TriggerSpec::FileContents {
                path: path.clone(),
                regex: String::new(),
            },
            expected: TypedObservation::FileContent {
                path,
                expected_blake3: want,
            },
            sandbox: SandboxTier::ContainerIsolated,
            confound_flags: ConfoundFlags::default(),
        };
        let outcome = verify_observable_effect(&effect, &mock, &ssb, "agent-d18", 0, TEST_NOW_MS)
            .await
            .expect("verify");
        assert_eq!(outcome, VerificationOutcome::Pass);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn verify_mismatched_pairing_is_skipped() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let mut mock = MockSandboxExecutor::new();
        mock.insert(
            "echo x",
            "test_command",
            CapturedObservation::ExitCode(0),
        );
        // Expected JsonShape against test_command/ExitCode → Skipped.
        let effect = ObservableEffect {
            effect_id: "e-5".into(),
            trigger: TriggerSpec::TestCommand("echo x".into()),
            expected: TypedObservation::JsonShape(serde_json::json!({"k": 1})),
            sandbox: SandboxTier::NoSandbox,
            confound_flags: ConfoundFlags::default(),
        };
        let outcome = verify_observable_effect(&effect, &mock, &ssb, "agent-d18", 0, TEST_NOW_MS)
            .await
            .expect("verify");
        assert_eq!(outcome, VerificationOutcome::Skipped);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn verify_executor_error_is_skipped_not_error() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        // Mock with no fixtures → executor returns Err.
        let mock = MockSandboxExecutor::new();
        let effect = ObservableEffect {
            effect_id: "e-6".into(),
            trigger: TriggerSpec::TestCommand("missing".into()),
            expected: TypedObservation::StringMatch("x".into()),
            sandbox: SandboxTier::NoSandbox,
            confound_flags: ConfoundFlags::default(),
        };
        let outcome = verify_observable_effect(&effect, &mock, &ssb, "agent-d18", 0, TEST_NOW_MS)
            .await
            .expect("verify (executor error should not be an SsbError)");
        assert_eq!(outcome, VerificationOutcome::Skipped);
    }
}
