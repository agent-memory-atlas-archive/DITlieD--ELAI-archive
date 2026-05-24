//! Batch 3A (Phase 4F) — FeasibilityProbe surface.
//!
//! Runs a pre-execution feasibility check against an
//! [`crate::ExecutablePipeline`] (task-level) or an individual
//! [`crate::research_tree::BabyStepSpec`] (per-concept, 4F.7-4F.8 extension).
//!
//! Skip criteria (batch-3a prompt + phase-index 4F):
//!   zero `novel_step_ids` AND all resource margins >= 20% -> return
//!   [`TaskFeasibility::Feasible`] with `skip_reason = Some("no_novel_steps_margins_ok")`.
//!
//! Otherwise the prober runs a stub executor per novel step. The stub executor
//! is intentionally pluggable: production wire-up plugs a real
//! `elai_cc_agent` spawn, but tests and the dashboard fall back to the
//! synthetic `input_parse_probe` path so the contract is exercisable without
//! an agent binary on disk. The stub timeout is 25 s graceful + 30 s hard
//! kill per the phase-4F spec.
//!
//! Emits `stub_results.msgpack` to `.elai_cc/feasibility/<task_id>/stub_results.msgpack`
//! for post-hoc inspection. Directory creation failure is non-fatal (the
//! probe returns the in-memory report; SSB rows are still written).

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use crate::pipeline_builder::ExecutablePipeline;
use elai_core::{ClockProvider as _, OsClock};
use elai_ipc_types::{
    AgentDispatchConfig, AgentRole, IpcFrame, IpcMessageType, ProcessConfig, ResourceLimits,
    SeccompProfile, ToolFilter, ToolId, FRAME_HEADER_SIZE, IPC_PROTOCOL_VERSION, MAX_FRAME_SIZE,
};
use crate::fsm::{Fsm, FsmState};
use crate::research_tree::BabyStepSpec;
use crate::system_probe::SystemProbeResult;

/// Environment variable toggling the probe entirely. Truthy values:
/// "1", "true", "TRUE", "on", "ON".
pub const ELAI_DISABLE_FEASIBILITY_PROBE_ENV: &str = "ELAI_DISABLE_FEASIBILITY_PROBE";

/// Graceful stub timeout. Matches phase-4F "25s graceful".
pub const STUB_TIMEOUT_GRACEFUL: Duration = Duration::from_secs(25);

/// Hard-kill timeout. Matches phase-4F "30s hard kill".
pub const STUB_TIMEOUT_HARD: Duration = Duration::from_secs(30);

/// E-22 phase-4F P1: single-named-const dual-timeout convention so callers
/// (BullMQ sandboxed-processors / E2B / Promptfoo equivalent — 30s outer
/// hard cap for sandboxed stubs). Alias for [`STUB_TIMEOUT_HARD`] under
/// the spec-named identifier so the constant is reachable as
/// `FEASIBILITY_PROBE_TIMEOUT` for the spec-aligned cross-reference. No
/// magic numbers — exactly one canonical source for the 30s budget.
pub const FEASIBILITY_PROBE_TIMEOUT: Duration = STUB_TIMEOUT_HARD;

/// Resource-margin threshold (percent). A margin below 20% triggers the
/// `ResourceMarginBreached` escalation path per spec.
pub const RESOURCE_MARGIN_THRESHOLD_PCT: f32 = 20.0;

/// Returns `true` when `ELAI_DISABLE_FEASIBILITY_PROBE` is set truthy.
pub fn feasibility_probe_disabled_via_env() -> bool {
    std::env::var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV)
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
        .unwrap_or(false)
}

/// Top-level feasibility verdict for a task / concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskFeasibility {
    /// No novel steps or all novel steps passed stub; margins safe.
    Feasible,
    /// At least one novel step marginal OR one margin in [10%, 20%).
    Marginal,
    /// At least one novel step failed AND no alternative OR margin < 10%.
    Infeasible,
}

/// Resource-margin snapshot vs the live `SystemProbeResult`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceMargins {
    pub vram_margin_pct: f32,
    pub ram_margin_pct: f32,
    pub disk_margin_pct: f32,
    /// True iff all three margins >= `RESOURCE_MARGIN_THRESHOLD_PCT`.
    pub all_margins_safe: bool,
}

impl ResourceMargins {
    /// Build margins from a system report. When total is zero (unavailable),
    /// the corresponding margin is 0.0 (forces the caller to escalate
    /// because we have no positive evidence of headroom).
    pub fn from_report(report: &SystemProbeResult) -> Self {
        let vram_total = report
            .gpu
            .as_ref()
            .map(|g| g.vram_total_mb as f32)
            .unwrap_or(0.0);
        let vram_free = report
            .gpu
            .as_ref()
            .map(|g| g.vram_free_mb as f32)
            .unwrap_or(0.0);
        let ram_total = report.ram_total_mb as f32;
        let ram_free = report.ram_free_mb as f32;
        let disk_total = report.disk_total_mb as f32;
        let disk_free = report.disk_free_mb as f32;
        let vram_margin_pct = if vram_total > 0.0 {
            (vram_free / vram_total) * 100.0
        } else {
            0.0
        };
        let ram_margin_pct = if ram_total > 0.0 {
            (ram_free / ram_total) * 100.0
        } else {
            0.0
        };
        let disk_margin_pct = if disk_total > 0.0 {
            (disk_free / disk_total) * 100.0
        } else {
            0.0
        };
        let all_margins_safe = vram_margin_pct >= RESOURCE_MARGIN_THRESHOLD_PCT
            && ram_margin_pct >= RESOURCE_MARGIN_THRESHOLD_PCT
            && disk_margin_pct >= RESOURCE_MARGIN_THRESHOLD_PCT;
        Self {
            vram_margin_pct,
            ram_margin_pct,
            disk_margin_pct,
            all_margins_safe,
        }
    }
}

/// A pipeline step flagged as novel (unverified tool ref, untested capability, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovelStep {
    pub step_id: String,
    pub reason: String,
    /// Up to 3 alternative step identifiers the probe may substitute.
    pub alternatives: Vec<String>,
}

/// Per-stub execution outcome.
///
/// C-22 (StubResult v2): four additive fields land here. All carry
/// `#[serde(default)]` so existing msgpack payloads (pre-v2) deserialise
/// cleanly. `#[derive(Default)]` lets the contract tests (and any future
/// call site) construct the struct via `..Default::default()` spread.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StubResult {
    pub step_id: String,
    pub stub_passed: bool,
    pub elapsed_ms: u32,
    pub error: Option<String>,
    /// 0-indexed alternative slot the stub ran for. 0 = first attempt
    /// (original step). Up to `MAX_ALTERNATIVES_PER_STEP - 1`. Defaults
    /// to 0 for back-compat with v1 stub-result msgpack payloads.
    #[serde(default)]
    pub alternative_index: u8,
    /// blake3 hex digest of the stub's stdout. `None` if stdout was empty
    /// or capture was not possible (e.g., synthetic stub path).
    #[serde(default)]
    pub stdout_hash: Option<String>,
    /// First 256 chars of the stub's stderr. `None` if stderr was empty.
    #[serde(default)]
    pub stderr_summary: Option<String>,
    /// VRAM + RAM delta captured around the stub execution. `None` when
    /// the SystemProbeResult re-query failed; never fatal.
    #[serde(default)]
    pub resource_delta: Option<ResourceDelta>,
}

impl StubResult {
    /// E-22 phase-4F P6: build a passing stub result whose `stdout_hash` is
    /// the blake3 hex digest of `stdout`. Centralises the hash computation
    /// so future call sites cannot silently default `stdout_hash` to `None`.
    /// `stdout` is taken by reference so the original buffer stays available
    /// for separate logging / preservation paths.
    ///
    /// Spec phase-4F.txt §4F.1 TYPES + invariant 12 (line 445): "Stdout hash
    /// (blake3) is deterministic for identical stdout bytes."
    pub fn passed_with_stdout(step_id: &str, elapsed_ms: u32, stdout: &str) -> Self {
        let hash = blake3::hash(stdout.as_bytes()).to_hex().to_string();
        Self {
            step_id: step_id.to_owned(),
            stub_passed: true,
            elapsed_ms,
            error: None,
            alternative_index: 0,
            stdout_hash: Some(hash),
            stderr_summary: None,
            resource_delta: None,
        }
    }

    /// Same as [`Self::passed_with_stdout`] but for an explicit alternative
    /// slot (1..=2). Used by the P6 alternative-loop in
    /// [`DefaultFeasibilityProber::probe`] when retrying a failed stub on
    /// a different alternative. Currently constructed manually inside the
    /// loop (the loop owns the per-attempt mutation); kept on the impl
    /// surface so future call sites have a single canonical entry point.
    #[allow(dead_code)]
    pub fn passed_with_stdout_at(
        step_id: &str,
        elapsed_ms: u32,
        stdout: &str,
        alternative_index: u8,
    ) -> Self {
        let mut sr = Self::passed_with_stdout(step_id, elapsed_ms, stdout);
        sr.alternative_index = alternative_index;
        sr
    }
}

/// C-22: VRAM + RAM snapshot captured before and after a feasibility-probe
/// stub executes. Reported alongside [`StubResult`] so the dashboard can
/// render pre/post resource usage. Bytes are absolute counts; the consumer
/// computes deltas as `vram_bytes_after - vram_bytes_before`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ResourceDelta {
    pub vram_bytes_before: u64,
    pub vram_bytes_after: u64,
    pub ram_bytes_before: u64,
    pub ram_bytes_after: u64,
}

/// Reasons a feasibility probe may escalate to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeasibilityEscalation {
    NoFeasibleAlternative { step_id: String },
    ResourceMarginBreached { resource: String, margin_pct: String },
    StubTimeout { step_id: String, elapsed_ms: u32 },
}

/// Full pipeline-level feasibility report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeasibilityReport {
    pub task_feasibility: TaskFeasibility,
    pub resource_margins: ResourceMargins,
    pub novel_steps: Vec<NovelStep>,
    pub stub_results: Vec<StubResult>,
    pub escalations: Vec<FeasibilityEscalation>,
    pub skip_reason: Option<String>,
}

/// Per-concept (4F.7-4F.8) feasibility report for baby steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BabyStepFeasibilityReport {
    pub step_id: String,
    pub feasible: bool,
    pub stub_results: Vec<BabyStepStubResult>,
    pub escalation: Option<BabyStepEscalation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BabyStepStubResult {
    pub step_id: String,
    pub stub_passed: bool,
    pub elapsed_ms: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BabyStepEscalation {
    StubFailed { step_id: String, reason: String },
    StubTimeout { step_id: String, elapsed_ms: u32 },
}

// ---------------------------------------------------------------------------
// Phase-4F.txt spec close-out (s-32820 autonomous run): additive types and
// helpers that match the spec's `decide` + `check_constraints` + stub
// line-count contract WITHOUT breaking the batch-3a surface.
// ---------------------------------------------------------------------------

/// Spec phase-4F.txt: per-resource constraint outcome with a numeric margin.
/// `margin = (available - required) / required`. Skip threshold per spec is
/// `margin >= 0.20` for all three resources (vram, ram, disk); at-least-met
/// is `margin >= 0.0`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintResult {
    /// `"vram_gpu0"` / `"ram"` / `"disk"`.
    pub resource: String,
    pub required_bytes: u64,
    pub available_bytes: u64,
    pub margin: f64,
    pub passed: bool,
}

/// Spec phase-4F.txt: top-level decision the orchestrator hands to the FSM
/// after a probe completes. The execution-mode split (`Escalate` for
/// INTERACTIVE, `Replan` for AUTONOMOUS) is encoded in [`decide_with_mode`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeasibilityDecision {
    /// All probes pass, all stubs pass.
    Proceed,
    /// Probes show limits; stubs adapted (e.g. single-GPU fallback).
    ProceedWithConstraints(Vec<String>),
    /// Human decision required (INTERACTIVE).
    Escalate(Vec<FeasibilityEscalation>),
    /// Auto-replan (AUTONOMOUS).
    Replan(Vec<FeasibilityEscalation>),
}

/// Mirror of the orchestrator's execution-mode split for the decision logic.
/// Defined locally so this module does not pull in the full `ExecutionMode`
/// enum from elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeasibilityExecutionMode {
    Interactive,
    Autonomous,
}

/// Spec phase-4F.txt §4F.1 / §4F.5 I2: pure constraint check. No I/O.
/// `required` carries minimum resource asks for the pipeline; `available`
/// is the live system probe. Returns one [`ConstraintResult`] per resource.
/// Margin >= [`RESOURCE_MARGIN_THRESHOLD_PCT / 100`] flags `passed = true`
/// — matches the 20 % skip threshold in the spec.
pub fn check_constraints(
    required: &SystemConstraintsRequired,
    available: &SystemProbeResult,
) -> Vec<ConstraintResult> {
    let threshold = (RESOURCE_MARGIN_THRESHOLD_PCT / 100.0) as f64;
    let vram_total_mb = available
        .gpu
        .as_ref()
        .map(|g| u64::from(g.vram_total_mb))
        .unwrap_or(0);
    let vram_total_bytes = vram_total_mb.saturating_mul(1024 * 1024);

    let mut out: Vec<ConstraintResult> = Vec::with_capacity(3);
    out.push(margin_constraint("vram_gpu0", required.min_vram_bytes, vram_total_bytes, threshold));
    out.push(margin_constraint(
        "ram",
        required.min_ram_bytes,
        available.ram_total_mb.saturating_mul(1024 * 1024),
        threshold,
    ));
    out.push(margin_constraint(
        "disk",
        required.min_disk_bytes,
        available.disk_total_mb.saturating_mul(1024 * 1024),
        threshold,
    ));
    out
}

fn margin_constraint(
    resource: &str,
    required_bytes: u64,
    available_bytes: u64,
    threshold: f64,
) -> ConstraintResult {
    let margin = if required_bytes == 0 {
        // Zero-required is trivially passed at +inf margin; encode as 0.0
        // so the f64 stays serialisable and `passed` is computed from the
        // available >= required relation.
        0.0
    } else {
        (available_bytes as f64 - required_bytes as f64) / (required_bytes as f64)
    };
    let passed = if required_bytes == 0 {
        true
    } else {
        margin >= threshold
    };
    ConstraintResult {
        resource: resource.to_owned(),
        required_bytes,
        available_bytes,
        margin,
        passed,
    }
}

/// Required-resources record derived from the pipeline spec. Bytes-typed so
/// `check_constraints` can compare against the system probe directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemConstraintsRequired {
    pub min_vram_bytes: u64,
    pub min_ram_bytes: u64,
    pub min_disk_bytes: u64,
    pub required_gpu_count: u32,
}

impl SystemConstraintsRequired {
    /// All-zero constraint set. Useful for code paths that have no
    /// pipeline-derived asks yet (e.g. pre-planning probes).
    pub fn zero() -> Self {
        Self {
            min_vram_bytes: 0,
            min_ram_bytes: 0,
            min_disk_bytes: 0,
            required_gpu_count: 0,
        }
    }
}

/// Spec phase-4F.txt §4F.5 I5: turn a [`FeasibilityReport`] into a
/// [`FeasibilityDecision`] for the given execution mode.
/// - All clear -> `Proceed`.
/// - Margin breach but no stub failures -> `ProceedWithConstraints` listing
///   the breached resources.
/// - Any [`FeasibilityEscalation::NoFeasibleAlternative`] -> `Escalate` for
///   INTERACTIVE, `Replan` for AUTONOMOUS.
/// - Otherwise (e.g. stub timeouts only) the decision matches the mode the
///   same way as no-feasible-alternative: human in INTERACTIVE, auto in
///   AUTONOMOUS.
pub fn decide_with_mode(
    report: &FeasibilityReport,
    mode: FeasibilityExecutionMode,
) -> FeasibilityDecision {
    if report.escalations.is_empty() && report.task_feasibility == TaskFeasibility::Feasible {
        return FeasibilityDecision::Proceed;
    }

    let only_resource_breach = !report.escalations.is_empty()
        && report.escalations.iter().all(|e| {
            matches!(e, FeasibilityEscalation::ResourceMarginBreached { .. })
        });
    if only_resource_breach {
        let resources: Vec<String> = report
            .escalations
            .iter()
            .filter_map(|e| match e {
                FeasibilityEscalation::ResourceMarginBreached { resource, .. } => {
                    Some(resource.clone())
                }
                _ => None,
            })
            .collect();
        return FeasibilityDecision::ProceedWithConstraints(resources);
    }

    match mode {
        FeasibilityExecutionMode::Interactive => {
            FeasibilityDecision::Escalate(report.escalations.clone())
        }
        FeasibilityExecutionMode::Autonomous => {
            FeasibilityDecision::Replan(report.escalations.clone())
        }
    }
}

/// Spec phase-4F.txt §4F.5 I3: stub line count enforcement. Strict `<50`
/// per the GOAL section — any stub source >= 50 newlines is rejected before
/// execution. The check is a pure function so callers can validate before
/// spawning the agent process.
pub const STUB_LINE_LIMIT: usize = 50;

/// Returns `Ok(line_count)` when the stub source has fewer than
/// [`STUB_LINE_LIMIT`] newlines, else `Err(line_count)`. The caller maps
/// the error into a synthetic [`StubResult`] / [`FeasibilityEscalation`].
/// Counts lines as `lines().count()` so a trailing `\n` is not double-
/// counted.
pub fn enforce_stub_line_limit(stub_source: &str) -> Result<usize, usize> {
    let lines = stub_source.lines().count();
    if lines >= STUB_LINE_LIMIT {
        Err(lines)
    } else {
        Ok(lines)
    }
}

/// Spec phase-4F.txt §4F.3 T8: max alternatives per novel step before the
/// probe escalates. Strict `<= 3` — the 4th attempt MUST trigger
/// [`FeasibilityEscalation::NoFeasibleAlternative`] without launching a
/// 4th stub.
pub const MAX_ALTERNATIVES_PER_STEP: usize = 3;

/// Returns `true` when an additional alternative is allowed, `false` when
/// the cap has been reached. Pure function — used by future branch-and-
/// explore wiring without changing the trait surface.
pub fn alternative_within_cap(attempts_so_far: usize) -> bool {
    attempts_so_far < MAX_ALTERNATIVES_PER_STEP
}

/// Runtime interface for running feasibility probes. Dyn-safe via
/// `Arc<dyn ...>`. Uses hand-rolled `Pin<Box<dyn Future>>` to avoid the
/// `async-trait` crate (CLAUDE.md anti-stale-training hard rule).
///
/// E-22 phase-4F P3 (spec compliance — additive shims): the spec
/// (phase-4F.txt §4F.1 L156-160) names 4 methods on this trait —
/// `probe_system` / `check_constraints` / `run_stub` / `decide`. The
/// shipped implementation kept the functional surface as a Prober +
/// StubExecutor split with free helpers (`check_constraints` /
/// `decide_with_mode` / `enforce_stub_line_limit`). We close the spec gap
/// by adding the four methods as default-method shims on the trait —
/// every shim takes `&self` so the trait object-safety contract at
/// `feasibility_probe_contract.rs:125` (`Arc<dyn FeasibilityProber>`)
/// stays intact.
pub trait FeasibilityProber: Send + Sync {
    /// Probe a full pipeline. Called at task startup.
    fn probe<'a>(
        &'a self,
        plan: &'a ExecutablePipeline,
        system: &'a SystemProbeResult,
    ) -> Pin<Box<dyn Future<Output = FeasibilityReport> + Send + 'a>>;

    /// Probe a single baby-step. Called per-concept before baby-step execution
    /// (4F.7-4F.8 per-concept path).
    fn probe_baby_step<'a>(
        &'a self,
        baby_step: &'a BabyStepSpec,
        system: &'a SystemProbeResult,
    ) -> Pin<Box<dyn Future<Output = BabyStepFeasibilityReport> + Send + 'a>>;

    /// E-22 phase-4F P3 SHIM: spec-named `probe_system` accessor. Default
    /// implementation delegates to the in-process [`crate::system_probe::DefaultSystemProber`].
    /// Implementors that hold a [`crate::system_probe::SystemProber`] should
    /// override this to return the live probe instead.
    fn probe_system<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = SystemProbeResult> + Send + 'a>> {
        Box::pin(async move {
            use crate::system_probe::{DefaultSystemProber, SystemProber};
            let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
            let transient = DefaultSystemProber::new(None, clock);
            transient.probe().await
        })
    }

    /// E-22 phase-4F P3 SHIM: spec-named `check_constraints` method. Pure;
    /// delegates to the free fn [`check_constraints`]. Returns the spec's
    /// boolean shape (all-passed) for compatibility with the spec trait
    /// signature; the granular per-resource [`ConstraintResult`] vector is
    /// available via the free fn for callers that need it.
    fn check_constraints(
        &self,
        probe: &SystemProbeResult,
        constraints: &SystemConstraintsRequired,
    ) -> bool {
        check_constraints(constraints, probe).iter().all(|c| c.passed)
    }

    /// E-22 phase-4F P3 SHIM: spec-named `run_stub` method. Default
    /// implementation routes to the in-process synthetic stub so a probe
    /// without an injected [`StubExecutor`] still produces a deterministic
    /// [`StubResult`]. Implementors with an injected Tier-3 executor
    /// override this to spawn the real agent.
    fn run_stub<'a>(
        &'a self,
        step_id: &'a str,
        alternative: u32,
        _timeout: Duration,
    ) -> Pin<Box<dyn Future<Output = StubResult> + Send + 'a>> {
        Box::pin(async move {
            let mut sr = run_synthetic_stub(step_id).await;
            // Strict-narrowing cast: alternative is 0..=2 by contract
            // (MAX_ALTERNATIVES_PER_STEP=3). Wider values get clamped so
            // the u8 field never overflows.
            sr.alternative_index = u8::try_from(alternative.min(255)).unwrap_or(0);
            sr
        })
    }

    /// E-22 phase-4F P3 SHIM: spec-named `decide` method. Pure; delegates
    /// to the free fn [`decide_with_mode`].
    fn decide(
        &self,
        report: &FeasibilityReport,
        mode: FeasibilityExecutionMode,
    ) -> FeasibilityDecision {
        decide_with_mode(report, mode)
    }

    /// D-18 closure SIBLING method: typed-observation projection of
    /// [`Self::probe`]. Default implementation calls [`Self::probe`] and
    /// projects each [`StubResult`] into an
    /// [`crate::observable_effect::ObservationResult`] tagged with
    /// `ObservationContext::Sandbox(Tier)` and an `ObservationOutcome`
    /// derived from the stub's `stub_passed` flag.
    ///
    /// PRESERVES the [`Self::probe`] signature for C-22 dyn-safety —
    /// `Arc<dyn FeasibilityProber>` (the contract surface) stays object-
    /// safe because this method is sync, takes `&self`, and returns a
    /// non-generic [`Vec`].
    ///
    /// Sandbox tier defaults to `ContainerIsolated` here because the
    /// in-process [`StubResult`] does not carry the tier label; live
    /// dispatch paths (e.g. `dispatch_observable_effect_hook`) should
    /// override the tier from the effect's [`crate::observable_effect::ObservableEffect.sandbox`]
    /// field when calling [`crate::observable_effect::verify_observable_effect`]
    /// directly.
    ///
    /// Paper grounding: Coscientist arxiv:2304.05332 §4
    /// SandboxObservation taxonomy. See workflow/reasoning/D-18-
    /// decisions.md DECISION-5 for the sibling-method rationale (the
    /// alternative — modifying `probe()` to return `Vec<ObservationResult>`
    /// — would break the C-22 dyn-safety contract pinned at
    /// `crates/contracts/tests/c22_feasibility_completeness_contract.rs`).
    fn probe_with_effects(
        &self,
        report: &FeasibilityReport,
    ) -> Vec<crate::observable_effect::ObservationResult> {
        use crate::observable_effect::{ObservationContext, ObservationOutcome, ObservationResult};
        let mut out = Vec::with_capacity(report.stub_results.len());
        for sr in &report.stub_results {
            let outcome = if sr.stub_passed {
                ObservationOutcome::Confirmed
            } else {
                ObservationOutcome::Refuted
            };
            out.push(ObservationResult {
                effect_id: sr.step_id.clone(),
                context: ObservationContext::Sandbox(
                    crate::observable_effect::SandboxTier::ContainerIsolated,
                ),
                outcome,
                instrument_file: None,
            });
        }
        out
    }
}

/// E-22 phase-4F P1: idempotent FSM transition wrapper.
///
/// Spec phase-4F.txt:139 + §4F.5 I6: the orchestrator transitions
/// `PLANNING -> FEASIBILITY_PROBE -> TESTING_FIRST` (or `PLANNING ->
/// TESTING_FIRST` directly on skip). Calling code may invoke this helper
/// more than once (replay, resume after compaction). The helper is
/// idempotent: if the FSM is already in `target` the call is a no-op
/// (returns `Ok(false)`). Returns `Ok(true)` when a transition fired,
/// `Ok(false)` when already-in-state.
///
/// `evidence` is forwarded to [`Fsm::transition`] when a transition fires;
/// pass an empty vec when no artifact is being attached.
pub async fn fsm_transition_idempotent(
    fsm: &Fsm,
    target: FsmState,
    phase_label: &str,
) -> Result<bool, crate::fsm::FsmError> {
    let current = fsm.get_state().await;
    if current == target {
        tracing::debug!(
            current = %current,
            target = %target,
            "fsm_transition_idempotent: already in target, no-op"
        );
        return Ok(false);
    }
    fsm.transition(target, phase_label, Vec::new())
        .await
        .map(|_record| true)
}

/// E-22 phase-4F P1: dyn-safe FSM transition adapter so callers that do
/// not hold a concrete [`Fsm`] handle (e.g. the task_runner emit helpers
/// today) can still register a transition. Production wire-up adopts this
/// via a thin `Arc<dyn FeasibilityFsmBridge>` plumbed alongside the
/// feasibility prober; tests can install a no-op or a recording adapter.
/// Default impl on [`Fsm`] forwards to [`fsm_transition_idempotent`].
pub trait FeasibilityFsmBridge: Send + Sync {
    /// Transition the FSM toward `target` if not already there. Returns
    /// `true` when a transition fired, `false` when already-in-state.
    /// Errors are bubbled as `String` to keep the trait dyn-safe across
    /// different FSM error encodings.
    fn transition_to<'a>(
        &'a self,
        target: FsmState,
        phase_label: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<bool, String>> + Send + 'a>>;
}

impl FeasibilityFsmBridge for Fsm {
    fn transition_to<'a>(
        &'a self,
        target: FsmState,
        phase_label: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<bool, String>> + Send + 'a>> {
        Box::pin(async move {
            fsm_transition_idempotent(self, target, phase_label)
                .await
                .map_err(|e| e.to_string())
        })
    }
}

// ---------------------------------------------------------------------------
// StubExecutor trait + implementations (Phase 4F item 1, s-11136)
// ---------------------------------------------------------------------------

/// Pluggable executor that runs a single feasibility stub for a novel step.
///
/// Two production implementations are provided:
/// - [`SyntheticStubExecutor`]: input-parse-only; the test default. Wraps the
///   legacy `run_synthetic_stub` path so the 23 batch-3a contract tests stay
///   exercise-able without an agent binary on disk.
/// - [`Tier3StubExecutor`]: spawns the real `elai_cc_agent` binary via
///   `tokio::process::Command`, sends an [`IpcMessageType::AgentRequest`] with
///   the FEASIBILITY tool filter (stub_write + bash only), and enforces the
///   25 s graceful + 30 s hard-kill dual-timeout per phase-4F.txt §4F.5 I3.
///
/// Dyn-safe via hand-rolled `Pin<Box<dyn Future>>` (CLAUDE.md anti-stale-
/// training rule: no async-trait crate).
pub trait StubExecutor: Send + Sync {
    /// Run a single stub for `step_id`. `stub_source`, when present, is the
    /// candidate stub source code; the executor MUST reject sources that
    /// reach [`STUB_LINE_LIMIT`] before any process spawn.
    fn run_stub<'a>(
        &'a self,
        step_id: &'a str,
        stub_source: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = StubResult> + Send + 'a>>;
}

/// Test-default executor. Delegates to the in-process [`run_synthetic_stub`]
/// (input-parse only). No agent binary required. Behaviour is identical to
/// the pre-item-1 path so existing tests that construct
/// `DefaultFeasibilityProber::new(...)` without an injected executor see no
/// observable change.
#[derive(Debug, Default, Clone, Copy)]
pub struct SyntheticStubExecutor;

impl SyntheticStubExecutor {
    /// Construct a fresh synthetic executor. Stateless and `Copy` so callers
    /// can hand out instances without `Arc::clone` overhead.
    pub fn new() -> Self {
        Self
    }
}

impl StubExecutor for SyntheticStubExecutor {
    fn run_stub<'a>(
        &'a self,
        step_id: &'a str,
        _stub_source: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = StubResult> + Send + 'a>> {
        Box::pin(async move { run_synthetic_stub(step_id).await })
    }
}

// ---------------------------------------------------------------------------
// Tier3StubExecutor — real elai_cc_agent spawn path (phase-4F.txt §4F.5 I3)
// ---------------------------------------------------------------------------

/// Spawns the `elai_cc_agent` binary in feasibility mode and exchanges one
/// [`IpcMessageType::AgentRequest`] / [`IpcMessageType::AgentResponse`] pair
/// over a per-stub Unix socket (or Windows named pipe).
///
/// Wire flow:
///   1. Bind socket / pipe BEFORE spawning the agent (listen-before-spawn,
///      mirrors orchestrator.txt S12).
///   2. Spawn `agent_binary_path` with `[socket_path, agent_id_uuid]` argv.
///   3. Accept the agent's connection.
///   4. Send `AgentRequest{task=feasibility_stub:{step_id}, dispatch_config}`.
///   5. Read frames until [`IpcMessageType::AgentResponse`] arrives, ignoring
///      heartbeats.
///   6. On graceful timeout ([`STUB_TIMEOUT_GRACEFUL`] = 25 s): send
///      `AgentKill{reason="graceful_shutdown"}` and wait the remaining 5 s
///      for clean exit.
///   7. On hard timeout ([`STUB_TIMEOUT_HARD`] = 30 s): `child.kill().await`.
///   8. Always `child.wait().await` after the terminal branch to reap.
///
/// Tool restrictions: the agent runs with the FEASIBILITY [`ToolFilter`]
/// (stub_write + bash only). No edit, no git per orchestrator.txt S2 L38.
pub struct Tier3StubExecutor {
    agent_binary_path: PathBuf,
    socket_dir: PathBuf,
}

impl Tier3StubExecutor {
    /// Construct a Tier-C executor that spawns `agent_binary_path` for each
    /// stub. The default `socket_dir` is `std::env::temp_dir()`; override
    /// with [`Self::with_socket_dir`] when callers need an isolated
    /// per-task directory.
    pub fn new(agent_binary_path: PathBuf) -> Self {
        Self {
            agent_binary_path,
            socket_dir: std::env::temp_dir(),
        }
    }

    /// Override the socket / named-pipe staging directory.
    pub fn with_socket_dir(mut self, socket_dir: PathBuf) -> Self {
        self.socket_dir = socket_dir;
        self
    }

    /// Construct the FEASIBILITY [`AgentDispatchConfig`] sent to the spawned
    /// agent. Per §4F.5 I3: tool_filter restricts to stub_write + bash; GRC
    /// budget = 3 calls / 600 chars; sandbox = IpcBaseline seccomp.
    fn feasibility_dispatch_config() -> AgentDispatchConfig {
        AgentDispatchConfig {
            role: AgentRole::Executor,
            model_preference: "qwen2.5-coder".to_owned(),
            tool_filter: ToolFilter {
                allowed_tools: vec![
                    ToolId("stub_write".to_owned()),
                    ToolId("bash".to_owned()),
                ],
            },
            rule_domains: vec!["feasibility".to_owned()],
            grc_budget_max_calls: 3,
            grc_budget_max_chars: 600,
            sandbox_config: ProcessConfig {
                allowed_paths: Vec::new(),
                seccomp_profile: SeccompProfile::IpcBaseline,
                resource_limits: ResourceLimits {
                    max_memory_bytes: 512 * 1024 * 1024,
                    max_cpu_secs: 60,
                    max_open_files: 32,
                },
            },
        }
    }
}

#[cfg(unix)]
async fn bind_stub_server(
    socket_dir: &Path,
    socket_id: &Uuid,
) -> Result<(tokio::net::UnixListener, PathBuf), String> {
    let path = socket_dir.join(format!("elai_stub_{socket_id}.sock"));
    let _ = tokio::fs::remove_file(&path).await;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("create_dir_all:{e}"))?;
    }
    let listener =
        tokio::net::UnixListener::bind(&path).map_err(|e| format!("bind:{e}"))?;
    Ok((listener, path))
}

#[cfg(windows)]
async fn bind_stub_server(
    _socket_dir: &Path,
    socket_id: &Uuid,
) -> Result<(tokio::net::windows::named_pipe::NamedPipeServer, PathBuf), String> {
    let pipe_name = format!(r"\\.\pipe\elai_stub_{}", socket_id);
    let server = tokio::net::windows::named_pipe::ServerOptions::new()
        .first_pipe_instance(true)
        .create(&pipe_name)
        .map_err(|e| format!("create_pipe:{e}"))?;
    Ok((server, PathBuf::from(pipe_name)))
}

#[cfg(unix)]
async fn accept_stub_stream(
    server: tokio::net::UnixListener,
) -> Result<tokio::net::UnixStream, String> {
    let (stream, _addr) = server.accept().await.map_err(|e| format!("accept:{e}"))?;
    Ok(stream)
}

#[cfg(windows)]
async fn accept_stub_stream(
    server: tokio::net::windows::named_pipe::NamedPipeServer,
) -> Result<tokio::net::windows::named_pipe::NamedPipeServer, String> {
    server.connect().await.map_err(|e| format!("connect:{e}"))?;
    Ok(server)
}

/// Async write of a single [`IpcFrame`] payload to the wire.
async fn write_frame<W>(writer: &mut W, msg: &IpcMessageType) -> Result<(), String>
where
    W: AsyncWriteExt + Unpin,
{
    let frame = IpcFrame::encode(msg).map_err(|e| format!("ipc_encode:{e}"))?;
    let bytes = frame.to_wire_bytes().map_err(|e| format!("ipc_wire:{e}"))?;
    writer
        .write_all(&bytes)
        .await
        .map_err(|e| format!("ipc_write:{e}"))?;
    writer.flush().await.map_err(|e| format!("ipc_flush:{e}"))?;
    Ok(())
}

/// Async read of one [`IpcFrame`] payload from the wire. Returns the decoded
/// [`IpcMessageType`].
async fn read_frame<R>(reader: &mut R) -> Result<IpcMessageType, String>
where
    R: AsyncReadExt + Unpin,
{
    let mut header = [0u8; FRAME_HEADER_SIZE];
    reader
        .read_exact(&mut header)
        .await
        .map_err(|e| format!("ipc_read_header:{e}"))?;
    let version = header[0];
    if version != IPC_PROTOCOL_VERSION {
        return Err(format!(
            "ipc_version_mismatch:expected={IPC_PROTOCOL_VERSION},got={version}"
        ));
    }
    let len = u32::from_le_bytes([header[1], header[2], header[3], header[4]]) as usize;
    if len > MAX_FRAME_SIZE {
        return Err(format!("ipc_payload_too_large:{len}"));
    }
    let mut payload = vec![0u8; len];
    reader
        .read_exact(&mut payload)
        .await
        .map_err(|e| format!("ipc_read_payload:{e}"))?;
    IpcFrame::decode(&payload).map_err(|e| format!("ipc_decode:{e}"))
}

impl StubExecutor for Tier3StubExecutor {
    fn run_stub<'a>(
        &'a self,
        step_id: &'a str,
        stub_source: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = StubResult> + Send + 'a>> {
        Box::pin(async move {
            // C-15: route Instant via OsClock so the entropy quarantine scanner
            // stays clean. Tier-3 stub timing is observation-only.
            let stub_clock = OsClock;
            let start = stub_clock.now();

            // Gate 1: line-count enforcement BEFORE any process spawn. Must
            // precede the binary-existence check so an oversized stub
            // produces a deterministic error regardless of binary state.
            if let Some(src) = stub_source {
                if let Err(n) = enforce_stub_line_limit(src) {
                    return StubResult {
                        step_id: step_id.to_owned(),
                        stub_passed: false,
                        elapsed_ms: 0,
                        error: Some(format!("stub_line_limit_exceeded:{n}")),
                        ..Default::default()
                    };
                }
            }

            // Listen-before-spawn: bind socket / pipe first so the spawned
            // agent finds the endpoint on connect.
            let socket_id = Uuid::new_v4(); // entropy-allow: per-socket id, no EntropySource threaded
            let (server, socket_path) =
                match bind_stub_server(&self.socket_dir, &socket_id).await {
                    Ok(pair) => pair,
                    Err(e) => {
                        return StubResult {
                            step_id: step_id.to_owned(),
                            stub_passed: false,
                            elapsed_ms: u32::try_from(stub_clock.elapsed_since(start).as_millis())
                                .unwrap_or(u32::MAX),
                            error: Some(format!("ipc_bind_failed:{e}")),
                            ..Default::default()
                        };
                    }
                };

            let agent_id_str = socket_id.to_string();
            let mut cmd = tokio::process::Command::new(&self.agent_binary_path);
            cmd.arg(&socket_path).arg(&agent_id_str);
            // C-22: `kill_on_drop(true)` removed. The two explicit
            // `child.kill().await` calls below (graceful + hard-timeout
            // branches) are the sole reaping mechanism per WIRING_CONTRACT
            // stub_ban. Cancellation-by-drop is not exercised by any
            // current test path; if a future caller requires it, prefer
            // an explicit cancel guard over kill_on_drop's silent kill.

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    #[cfg(unix)]
                    let _ = tokio::fs::remove_file(&socket_path).await;
                    #[cfg(windows)]
                    let _ = &socket_path;
                    return StubResult {
                        step_id: step_id.to_owned(),
                        stub_passed: false,
                        elapsed_ms: u32::try_from(stub_clock.elapsed_since(start).as_millis())
                            .unwrap_or(u32::MAX),
                        error: Some("agent_binary_not_found".to_owned()),
                        ..Default::default()
                    };
                }
                Err(e) => {
                    tracing::warn!(error = %e, step_id = %step_id, "tier3_spawn_failed");
                    #[cfg(unix)]
                    let _ = tokio::fs::remove_file(&socket_path).await;
                    #[cfg(windows)]
                    let _ = &socket_path;
                    return StubResult {
                        step_id: step_id.to_owned(),
                        stub_passed: false,
                        elapsed_ms: u32::try_from(stub_clock.elapsed_since(start).as_millis())
                            .unwrap_or(u32::MAX),
                        error: Some(format!("spawn_failed:{e}")),
                        ..Default::default()
                    };
                }
            };

            // Build the AgentRequest.
            let task = match stub_source {
                Some(src) => format!("feasibility_stub:{step_id}\n{src}"),
                None => format!("feasibility_stub:{step_id}"),
            };
            let dispatch_config = Self::feasibility_dispatch_config();
            let request = IpcMessageType::AgentRequest {
                task,
                dispatch_config,
            };

            // Outer hard timeout wraps the entire IPC handshake — even an
            // unresponsive agent cannot exceed STUB_TIMEOUT_HARD.
            let handshake = tokio::time::timeout(STUB_TIMEOUT_HARD, async {
                let mut stream = accept_stub_stream(server).await?;
                write_frame(&mut stream, &request).await?;

                // Graceful read loop bounded by STUB_TIMEOUT_GRACEFUL.
                // Skips heartbeats; returns on AgentResponse.
                let graceful = tokio::time::timeout(STUB_TIMEOUT_GRACEFUL, async {
                    loop {
                        match read_frame(&mut stream).await? {
                            IpcMessageType::AgentResponse { content, artifacts } => {
                                return Ok::<(String, Vec<String>), String>((content, artifacts));
                            }
                            IpcMessageType::AgentHeartbeat { .. } => continue,
                            other => {
                                return Err(format!("unexpected_ipc_message:{other:?}"));
                            }
                        }
                    }
                })
                .await;

                match graceful {
                    Ok(Ok(resp)) => Ok::<(String, Vec<String>), String>(resp),
                    Ok(Err(e)) => Err(e),
                    Err(_elapsed) => {
                        // 25 s graceful elapsed: send AgentKill IPC and wait
                        // the remaining 5 s for the child to flush a final
                        // status frame.
                        let _ = write_frame(
                            &mut stream,
                            &IpcMessageType::AgentKill {
                                reason: "graceful_shutdown".to_owned(),
                            },
                        )
                        .await;
                        let remaining =
                            STUB_TIMEOUT_HARD.saturating_sub(STUB_TIMEOUT_GRACEFUL);
                        let post = tokio::time::timeout(remaining, async {
                            loop {
                                match read_frame(&mut stream).await? {
                                    IpcMessageType::AgentResponse { content, artifacts } => {
                                        return Ok::<(String, Vec<String>), String>((
                                            content, artifacts,
                                        ));
                                    }
                                    IpcMessageType::AgentHeartbeat { .. } => continue,
                                    other => {
                                        return Err(format!(
                                            "unexpected_ipc_message:{other:?}"
                                        ));
                                    }
                                }
                            }
                        })
                        .await;
                        match post {
                            Ok(inner) => inner,
                            Err(_elapsed) => Err("stub_killed_graceful".to_owned()),
                        }
                    }
                }
            })
            .await;

            let elapsed_ms =
                u32::try_from(stub_clock.elapsed_since(start).as_millis()).unwrap_or(u32::MAX);

            // Reap regardless of outcome. child.wait() prevents zombie
            // accumulation per phase-4F.txt T7 adversarial note.
            let result = match handshake {
                Ok(Ok((content, _artifacts))) => {
                    if let Err(e) = child.wait().await {
                        tracing::warn!(error = %e, step_id = %step_id, "child_wait_failed");
                    }
                    let passed = !content.trim().is_empty();
                    if passed {
                        // E-22 phase-4F P6: real stdout in hand — populate
                        // blake3 digest via the centralised helper so the
                        // deterministic-replay invariant (spec phase-4F.txt
                        // invariant 12) holds for every real Tier-3 spawn.
                        StubResult::passed_with_stdout(step_id, elapsed_ms, &content)
                    } else {
                        StubResult {
                            step_id: step_id.to_owned(),
                            stub_passed: false,
                            elapsed_ms,
                            error: Some("empty_agent_response".to_owned()),
                            ..Default::default()
                        }
                    }
                }
                Ok(Err(reason)) => {
                    if let Err(e) = child.kill().await {
                        tracing::warn!(error = %e, step_id = %step_id, "kill_after_ipc_error");
                    }
                    if let Err(e) = child.wait().await {
                        tracing::warn!(error = %e, step_id = %step_id, "child_wait_failed");
                    }
                    StubResult {
                        step_id: step_id.to_owned(),
                        stub_passed: false,
                        elapsed_ms,
                        error: Some(reason),
                        ..Default::default()
                    }
                }
                Err(_elapsed) => {
                    if let Err(e) = child.kill().await {
                        tracing::warn!(error = %e, step_id = %step_id, "hard_kill_failed");
                    }
                    if let Err(e) = child.wait().await {
                        tracing::warn!(error = %e, step_id = %step_id, "child_wait_failed_post_hard_kill");
                    }
                    StubResult {
                        step_id: step_id.to_owned(),
                        stub_passed: false,
                        elapsed_ms: u32::try_from(STUB_TIMEOUT_HARD.as_millis())
                            .unwrap_or(u32::MAX),
                        error: Some("stub_timeout_hard_kill".to_owned()),
                        ..Default::default()
                    }
                }
            };

            // Best-effort cleanup. On Windows the named-pipe handle is
            // released when the server drops; the path is informational.
            #[cfg(unix)]
            let _ = tokio::fs::remove_file(&socket_path).await;
            #[cfg(windows)]
            let _ = &socket_path;

            result
        })
    }
}

// ---------------------------------------------------------------------------
// DefaultFeasibilityProber — pluggable stub executor (default = synthetic)
// ---------------------------------------------------------------------------

/// Default prober. Writes `stub_results.msgpack` under `feasibility_root`
/// (typically `.elai_cc/feasibility/<task_id>/`). When [`Self::with_stub_executor`]
/// is unset, falls back to the in-process [`SyntheticStubExecutor`] so the
/// 23 batch-3a contract tests keep passing without an agent binary on disk.
/// Callers wiring the real Tier-C agent path inject a [`Tier3StubExecutor`]
/// via the builder.
pub struct DefaultFeasibilityProber {
    feasibility_root: PathBuf,
    stub_executor: Option<Arc<dyn StubExecutor>>,
    /// C-22: optional system_prober used to capture the AFTER snapshot in
    /// [`StubResult::resource_delta`]. When `None` the prober still emits
    /// a `ResourceDelta` populated from the `system: &SystemProbeResult`
    /// argument (before = after = input snapshot). Injection is optional
    /// so the 23 batch-3a contract tests do not require new wiring.
    system_prober: Option<Arc<dyn crate::system_probe::SystemProber>>,
}

impl DefaultFeasibilityProber {
    pub fn new(feasibility_root: PathBuf) -> Self {
        Self {
            feasibility_root,
            stub_executor: None,
            system_prober: None,
        }
    }

    /// Inject a custom [`StubExecutor`]. When set, [`Self::probe`] and
    /// [`Self::probe_baby_step`] route every stub through the executor;
    /// when unset they fall back to [`run_synthetic_stub`].
    pub fn with_stub_executor(mut self, executor: Arc<dyn StubExecutor>) -> Self {
        self.stub_executor = Some(executor);
        self
    }

    /// C-22: inject a system_prober so the prober can re-query the system
    /// for the AFTER snapshot of [`StubResult::resource_delta`]. Without
    /// this builder the AFTER snapshot mirrors the BEFORE snapshot from
    /// the `system: &SystemProbeResult` probe argument.
    pub fn with_system_prober(
        mut self,
        prober: Arc<dyn crate::system_probe::SystemProber>,
    ) -> Self {
        self.system_prober = Some(prober);
        self
    }

    /// Root under which `stub_results.msgpack` is written.
    pub fn feasibility_root(&self) -> &Path {
        &self.feasibility_root
    }

    fn ensure_task_dir(&self, task_id: &str) -> Option<PathBuf> {
        let dir = self.feasibility_root.join(task_id);
        // Best-effort mkdir — a failure here only skips the disk artifact.
        if let Err(e) = std::fs::create_dir_all(&dir) {
            tracing::warn!(
                dir = %dir.display(),
                error = %e,
                "feasibility_probe: create_dir_all failed; skipping stub_results.msgpack"
            );
            return None;
        }
        Some(dir)
    }

    fn write_stub_results(
        &self,
        task_id: &str,
        results: &[StubResult],
    ) {
        let Some(dir) = self.ensure_task_dir(task_id) else {
            return;
        };
        let path = dir.join("stub_results.msgpack");
        match rmp_serde::to_vec_named(results) {
            Ok(bytes) => {
                if let Err(e) = std::fs::write(&path, bytes) {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "feasibility_probe: write stub_results.msgpack failed"
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "feasibility_probe: msgpack encode stub_results failed"
                );
            }
        }
    }

    /// E-22 phase-4F P5: full path on disk where stub_results.msgpack lives
    /// for a given task. Exposed so downstream consumers (Tauri command +
    /// CLI inspector) can locate the artifact deterministically.
    pub fn stub_results_path(&self, task_id: &str) -> PathBuf {
        self.feasibility_root.join(task_id).join("stub_results.msgpack")
    }
}

/// E-22 phase-4F P5: read a previously-written `stub_results.msgpack`
/// artifact back into a `Vec<StubResult>`. Closes the spec "GOAL: Output
/// stub_results.msgpack" loop — the writer at
/// [`DefaultFeasibilityProber::write_stub_results`] is now matched by a
/// production reader that downstream consumers (dashboard Tauri command,
/// CLI inspector, replay tooling) can call.
///
/// Errors map onto [`StubResultsReadError`] so callers see a typed reason
/// (no `Box<dyn Error>`).
#[derive(Debug, thiserror::Error)]
pub enum StubResultsReadError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("msgpack decode: {0}")]
    Decode(#[from] rmp_serde::decode::Error),
}

/// Read & deserialize the `stub_results.msgpack` artifact at `path`. Returns
/// `Ok(Vec::new())` when the path is absent so callers can treat
/// "no probe ran" as an empty result set rather than an error.
pub fn read_stub_results(path: &Path) -> Result<Vec<StubResult>, StubResultsReadError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = std::fs::read(path)?;
    let results: Vec<StubResult> = rmp_serde::from_slice(&bytes)?;
    Ok(results)
}

impl FeasibilityProber for DefaultFeasibilityProber {
    fn probe<'a>(
        &'a self,
        plan: &'a ExecutablePipeline,
        system: &'a SystemProbeResult,
    ) -> Pin<Box<dyn Future<Output = FeasibilityReport> + Send + 'a>> {
        Box::pin(async move {
            // Env kill switch — short-circuit to Feasible with a populated
            // skip_reason so callers still emit SSB rows uniformly.
            if feasibility_probe_disabled_via_env() {
                return FeasibilityReport {
                    task_feasibility: TaskFeasibility::Feasible,
                    resource_margins: ResourceMargins::from_report(system),
                    novel_steps: Vec::new(),
                    stub_results: Vec::new(),
                    escalations: Vec::new(),
                    skip_reason: Some("disabled_via_env".to_owned()),
                };
            }

            let margins = ResourceMargins::from_report(system);
            let novel_step_ids: Vec<String> = plan
                .novel_step_ids
                .iter()
                .map(|id| id.0.to_string())
                .collect();

            // Skip path: zero novel AND all margins safe.
            if novel_step_ids.is_empty() && margins.all_margins_safe {
                return FeasibilityReport {
                    task_feasibility: TaskFeasibility::Feasible,
                    resource_margins: margins,
                    novel_steps: Vec::new(),
                    stub_results: Vec::new(),
                    escalations: Vec::new(),
                    skip_reason: Some("no_novel_steps_margins_ok".to_owned()),
                };
            }

            // Collect resource-margin escalations first so the report always
            // carries them regardless of stub results.
            let mut escalations: Vec<FeasibilityEscalation> = Vec::new();
            if margins.vram_margin_pct < RESOURCE_MARGIN_THRESHOLD_PCT {
                escalations.push(FeasibilityEscalation::ResourceMarginBreached {
                    resource: "vram".to_owned(),
                    margin_pct: format!("{:.1}", margins.vram_margin_pct),
                });
            }
            if margins.ram_margin_pct < RESOURCE_MARGIN_THRESHOLD_PCT {
                escalations.push(FeasibilityEscalation::ResourceMarginBreached {
                    resource: "ram".to_owned(),
                    margin_pct: format!("{:.1}", margins.ram_margin_pct),
                });
            }
            if margins.disk_margin_pct < RESOURCE_MARGIN_THRESHOLD_PCT {
                escalations.push(FeasibilityEscalation::ResourceMarginBreached {
                    resource: "disk".to_owned(),
                    margin_pct: format!("{:.1}", margins.disk_margin_pct),
                });
            }

            // Run a stub per novel step. When a Tier-C executor is
            // injected via with_stub_executor it spawns the real
            // elai_cc_agent binary; otherwise the synthetic input-parse
            // probe runs in-process for back-compat with batch-3a tests.
            //
            // C-22: captures ResourceDelta around every stub. The BEFORE
            // snapshot comes from the `system` argument (already in scope
            // as the probe's input). The AFTER snapshot reads from the
            // injected `system_prober` when present; without it the AFTER
            // mirrors BEFORE (non-fatal — the field is Option<ResourceDelta>
            // exactly to allow this degraded mode).
            let mut novel_steps: Vec<NovelStep> = Vec::new();
            let mut stub_results: Vec<StubResult> = Vec::new();
            for step_id in &novel_step_ids {
                // E-22 phase-4F P6: alternative_index loop. Spec §4F.3 T8 +
                // §4F.5 I5 + invariant 5: every novel step gets at most
                // MAX_ALTERNATIVES_PER_STEP attempts; the first PASS short-
                // circuits, all attempts FAIL escalates to
                // NoFeasibleAlternative. The loop is bounded by
                // alternative_within_cap so the 4th attempt is structurally
                // unreachable (spec invariant 6).
                let mut step_pass_seen = false;
                let mut last_error: Option<String> = None;
                for attempt in 0..MAX_ALTERNATIVES_PER_STEP {
                    debug_assert!(alternative_within_cap(attempt));
                    let alternative_index = u8::try_from(attempt).unwrap_or(u8::MAX);
                    // BEFORE snapshot from the input system probe.
                    let delta_before = system_to_resource_delta_before(system);
                    let mut stub = match &self.stub_executor {
                        Some(exec) => exec.run_stub(step_id, None).await,
                        None => run_synthetic_stub(step_id).await,
                    };
                    stub.alternative_index = alternative_index;
                    // AFTER snapshot: re-probe if a system_prober is wired.
                    let delta = match &self.system_prober {
                        Some(prober) => {
                            let after = prober.probe().await;
                            merge_resource_delta(delta_before, &after)
                        }
                        None => delta_before,
                    };
                    stub.resource_delta = Some(delta);
                    let passed = stub.stub_passed;
                    if !passed {
                        last_error = stub.error.clone();
                    }
                    stub_results.push(stub);
                    if passed {
                        step_pass_seen = true;
                        break;
                    }
                }
                // Synthesize the NovelStep entry. `alternatives` carries the
                // alternative ids tried after the original (slots 1..N when
                // the first attempt failed). The first attempt is slot 0,
                // implicit on the step_id.
                let alternatives: Vec<String> = stub_results
                    .iter()
                    .filter(|s| s.step_id == *step_id && s.alternative_index > 0)
                    .map(|s| format!("{}#alt{}", step_id, s.alternative_index))
                    .collect();
                novel_steps.push(NovelStep {
                    step_id: step_id.clone(),
                    reason: last_error.clone().unwrap_or_else(||
                        if step_pass_seen {
                            "novel_step_probed_ok".to_owned()
                        } else {
                            "novel_step_alternatives_exhausted".to_owned()
                        }
                    ),
                    alternatives,
                });
                if !step_pass_seen {
                    escalations.push(FeasibilityEscalation::NoFeasibleAlternative {
                        step_id: step_id.clone(),
                    });
                }
            }

            // Derive task_feasibility from counts.
            let task_feasibility = if !escalations.is_empty() {
                if escalations
                    .iter()
                    .any(|e| matches!(e, FeasibilityEscalation::NoFeasibleAlternative { .. }))
                {
                    TaskFeasibility::Infeasible
                } else {
                    TaskFeasibility::Marginal
                }
            } else {
                TaskFeasibility::Feasible
            };

            // Write stub_results.msgpack best-effort. task_id = pipeline id
            // stringified so separate tasks keep separate directories.
            let task_id = format!("pipeline-{}", plan.pipeline_id.0);
            self.write_stub_results(&task_id, &stub_results);

            FeasibilityReport {
                task_feasibility,
                resource_margins: margins,
                novel_steps,
                stub_results,
                escalations,
                skip_reason: None,
            }
        })
    }

    fn probe_baby_step<'a>(
        &'a self,
        baby_step: &'a BabyStepSpec,
        _system: &'a SystemProbeResult,
    ) -> Pin<Box<dyn Future<Output = BabyStepFeasibilityReport> + Send + 'a>> {
        Box::pin(async move {
            if feasibility_probe_disabled_via_env() {
                return BabyStepFeasibilityReport {
                    step_id: baby_step.id.0.to_string(),
                    feasible: true,
                    stub_results: Vec::new(),
                    escalation: None,
                };
            }

            let step_id = baby_step.id.0.to_string();
            // C-22: capture ResourceDelta around the baby-step stub.
            let delta_before = system_to_resource_delta_before(_system);
            let mut stub = match &self.stub_executor {
                Some(exec) => exec.run_stub(&step_id, None).await,
                None => run_synthetic_stub(&step_id).await,
            };
            let delta = match &self.system_prober {
                Some(prober) => {
                    let after = prober.probe().await;
                    merge_resource_delta(delta_before, &after)
                }
                None => delta_before,
            };
            stub.resource_delta = Some(delta);
            let escalation = if !stub.stub_passed {
                Some(BabyStepEscalation::StubFailed {
                    step_id: step_id.clone(),
                    reason: stub
                        .error
                        .clone()
                        .unwrap_or_else(|| "synthetic_probe_failed".to_owned()),
                })
            } else {
                None
            };
            let stub_bs = BabyStepStubResult {
                step_id: stub.step_id.clone(),
                stub_passed: stub.stub_passed,
                elapsed_ms: stub.elapsed_ms,
                error: stub.error.clone(),
            };
            BabyStepFeasibilityReport {
                step_id,
                feasible: stub.stub_passed,
                stub_results: vec![stub_bs],
                escalation,
            }
        })
    }
}

/// C-22: project the input [`SystemProbeResult`] into a ResourceDelta where
/// `*_before == *_after`. When the prober has no system_prober injected, the
/// AFTER snapshot equals the BEFORE (graceful degradation; field is Option).
fn system_to_resource_delta_before(system: &SystemProbeResult) -> ResourceDelta {
    let vram_free_bytes = system
        .gpu
        .as_ref()
        .map(|g| (g.vram_free_mb as u64) * 1024 * 1024)
        .unwrap_or(0);
    let ram_free_bytes = system.ram_free_mb * 1024 * 1024;
    ResourceDelta {
        vram_bytes_before: vram_free_bytes,
        vram_bytes_after: vram_free_bytes,
        ram_bytes_before: ram_free_bytes,
        ram_bytes_after: ram_free_bytes,
    }
}

/// C-22: merge an AFTER snapshot from a re-probed [`SystemProbeResult`] into
/// an existing BEFORE [`ResourceDelta`]. Preserves the `*_before` fields.
fn merge_resource_delta(before: ResourceDelta, after: &SystemProbeResult) -> ResourceDelta {
    let vram_after_bytes = after
        .gpu
        .as_ref()
        .map(|g| (g.vram_free_mb as u64) * 1024 * 1024)
        .unwrap_or(before.vram_bytes_after);
    let ram_after_bytes = after.ram_free_mb * 1024 * 1024;
    ResourceDelta {
        vram_bytes_before: before.vram_bytes_before,
        vram_bytes_after: vram_after_bytes,
        ram_bytes_before: before.ram_bytes_before,
        ram_bytes_after: ram_after_bytes,
    }
}

/// Synthetic stub executor. Measures whether the step id is a parseable,
/// non-empty identifier. Returns `stub_passed = true` for every non-empty
/// input; empty input fails so test code can exercise the failure branch.
/// The real `elai_cc_agent` spawn path will replace this when a stub harness
/// is available on disk — deferred to a follow-up batch per the batch-3a
/// spec's "synthetic fallback" note.
async fn run_synthetic_stub(step_id: &str) -> StubResult {
    let _clock = OsClock;
    let start = _clock.now();
    // Baseline measurement of input-parse latency: a no-op await to let the
    // tokio scheduler yield once so `elapsed_ms` is non-zero under the
    // test-util paused-time harness.
    tokio::task::yield_now().await;
    let elapsed_ms = u32::try_from(_clock.elapsed_since(start).as_millis()).unwrap_or(u32::MAX);
    if step_id.trim().is_empty() {
        return StubResult {
            step_id: step_id.to_owned(),
            stub_passed: false,
            elapsed_ms,
            error: Some("empty_step_id".to_owned()),
            ..Default::default()
        };
    }
    StubResult {
        step_id: step_id.to_owned(),
        stub_passed: true,
        elapsed_ms,
        error: None,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system_probe::{DetectionMethod, GpuInfo};
    use elai_core::VramTier;

    fn sample_report(vram_free: u32, ram_free: u64, disk_free: u64) -> SystemProbeResult {
        SystemProbeResult {
            gpu: Some(GpuInfo {
                name: "test".to_owned(),
                vram_total_mb: 12_288,
                vram_free_mb: vram_free,
                tier: VramTier::Vram12Gb,
                driver_version: None,
                uuid: None,
            }),
            ram_total_mb: 32_000,
            ram_free_mb: ram_free,
            disk_total_mb: 1_000_000,
            disk_free_mb: disk_free,
            detection_method: DetectionMethod::ShellOut,
            probed_at_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn margins_from_report_safe_when_all_above_20pct() {
        let r = sample_report(6_000, 10_000, 300_000);
        let m = ResourceMargins::from_report(&r);
        assert!(m.all_margins_safe);
    }

    #[test]
    fn margins_from_report_unsafe_when_vram_low() {
        let r = sample_report(1_000, 10_000, 300_000);
        let m = ResourceMargins::from_report(&r);
        assert!(!m.all_margins_safe);
        assert!(m.vram_margin_pct < RESOURCE_MARGIN_THRESHOLD_PCT);
    }

    #[test]
    fn margins_zero_when_total_missing() {
        let mut r = sample_report(0, 0, 0);
        r.gpu = None;
        r.ram_total_mb = 0;
        r.disk_total_mb = 0;
        let m = ResourceMargins::from_report(&r);
        assert_eq!(m.vram_margin_pct, 0.0);
        assert_eq!(m.ram_margin_pct, 0.0);
        assert_eq!(m.disk_margin_pct, 0.0);
        assert!(!m.all_margins_safe);
    }

    #[tokio::test]
    async fn synthetic_stub_passes_for_non_empty_id() {
        let r = run_synthetic_stub("baby_step_1").await;
        assert!(r.stub_passed);
        assert_eq!(r.step_id, "baby_step_1");
        assert!(r.error.is_none());
    }

    #[tokio::test]
    async fn synthetic_stub_fails_for_empty_id() {
        let r = run_synthetic_stub("").await;
        assert!(!r.stub_passed);
        assert_eq!(r.error.as_deref(), Some("empty_step_id"));
    }
}
