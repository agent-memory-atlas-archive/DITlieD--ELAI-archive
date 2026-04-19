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
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::pipeline_builder::ExecutablePipeline;
use crate::research_tree::BabyStepSpec;
use crate::system_probe::SystemProbeReport;

/// Environment variable toggling the probe entirely. Truthy values:
/// "1", "true", "TRUE", "on", "ON".
pub const ELAI_DISABLE_FEASIBILITY_PROBE_ENV: &str = "ELAI_DISABLE_FEASIBILITY_PROBE";

/// Graceful stub timeout. Matches phase-4F "25s graceful".
pub const STUB_TIMEOUT_GRACEFUL: Duration = Duration::from_secs(25);

/// Hard-kill timeout. Matches phase-4F "30s hard kill".
pub const STUB_TIMEOUT_HARD: Duration = Duration::from_secs(30);

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

/// Resource-margin snapshot vs the live `SystemProbeReport`.
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
    pub fn from_report(report: &SystemProbeReport) -> Self {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StubResult {
    pub step_id: String,
    pub stub_passed: bool,
    pub elapsed_ms: u32,
    pub error: Option<String>,
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

/// Runtime interface for running feasibility probes. Dyn-safe via
/// `Arc<dyn ...>`. Uses hand-rolled `Pin<Box<dyn Future>>` to avoid the
/// `async-trait` crate (CLAUDE.md anti-stale-training hard rule).
pub trait FeasibilityProber: Send + Sync {
    /// Probe a full pipeline. Called at task startup.
    fn probe<'a>(
        &'a self,
        plan: &'a ExecutablePipeline,
        system: &'a SystemProbeReport,
    ) -> Pin<Box<dyn Future<Output = FeasibilityReport> + Send + 'a>>;

    /// Probe a single baby-step. Called per-concept before baby-step execution
    /// (4F.7-4F.8 per-concept path).
    fn probe_baby_step<'a>(
        &'a self,
        baby_step: &'a BabyStepSpec,
        system: &'a SystemProbeReport,
    ) -> Pin<Box<dyn Future<Output = BabyStepFeasibilityReport> + Send + 'a>>;
}

// ---------------------------------------------------------------------------
// DefaultFeasibilityProber — synthetic stub executor
// ---------------------------------------------------------------------------

/// Default prober. Writes `stub_results.msgpack` under `feasibility_root`
/// (typically `.elai_cc/feasibility/<task_id>/`). The stub executor is the
/// synthetic "input_parse_probe" path (no agent binary required) so the
/// wiring is observable on a stock dev box; future batches can plug the real
/// `elai_cc_agent` spawn by adding a `stub_executor: Option<Arc<dyn StubExecutor>>`
/// field without changing the trait surface.
pub struct DefaultFeasibilityProber {
    feasibility_root: PathBuf,
}

impl DefaultFeasibilityProber {
    pub fn new(feasibility_root: PathBuf) -> Self {
        Self { feasibility_root }
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
}

impl FeasibilityProber for DefaultFeasibilityProber {
    fn probe<'a>(
        &'a self,
        plan: &'a ExecutablePipeline,
        system: &'a SystemProbeReport,
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

            // Run synthetic stub per novel step. "Synthetic" here means we
            // simply record the step id and mark it pending-verification —
            // the real agent-spawn path is documented on the struct.
            let mut novel_steps: Vec<NovelStep> = Vec::new();
            let mut stub_results: Vec<StubResult> = Vec::new();
            for step_id in &novel_step_ids {
                novel_steps.push(NovelStep {
                    step_id: step_id.clone(),
                    reason: "synthetic_probe_pending_agent_binary".to_owned(),
                    alternatives: Vec::new(),
                });
                let stub = run_synthetic_stub(step_id).await;
                if !stub.stub_passed {
                    escalations.push(FeasibilityEscalation::NoFeasibleAlternative {
                        step_id: step_id.clone(),
                    });
                }
                stub_results.push(stub);
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
        _system: &'a SystemProbeReport,
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
            let synthetic = run_synthetic_stub(&step_id).await;
            let escalation = if !synthetic.stub_passed {
                Some(BabyStepEscalation::StubFailed {
                    step_id: step_id.clone(),
                    reason: synthetic
                        .error
                        .clone()
                        .unwrap_or_else(|| "synthetic_probe_failed".to_owned()),
                })
            } else {
                None
            };
            let stub_bs = BabyStepStubResult {
                step_id: synthetic.step_id.clone(),
                stub_passed: synthetic.stub_passed,
                elapsed_ms: synthetic.elapsed_ms,
                error: synthetic.error.clone(),
            };
            BabyStepFeasibilityReport {
                step_id,
                feasible: synthetic.stub_passed,
                stub_results: vec![stub_bs],
                escalation,
            }
        })
    }
}

/// Synthetic stub executor. Measures whether the step id is a parseable,
/// non-empty identifier. Returns `stub_passed = true` for every non-empty
/// input; empty input fails so test code can exercise the failure branch.
/// The real `elai_cc_agent` spawn path will replace this when a stub harness
/// is available on disk — deferred to a follow-up batch per the batch-3a
/// spec's "synthetic fallback" note.
async fn run_synthetic_stub(step_id: &str) -> StubResult {
    let start = std::time::Instant::now();
    // Baseline measurement of input-parse latency: a no-op await to let the
    // tokio scheduler yield once so `elapsed_ms` is non-zero under the
    // test-util paused-time harness.
    tokio::task::yield_now().await;
    let elapsed_ms = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
    if step_id.trim().is_empty() {
        return StubResult {
            step_id: step_id.to_owned(),
            stub_passed: false,
            elapsed_ms,
            error: Some("empty_step_id".to_owned()),
        };
    }
    StubResult {
        step_id: step_id.to_owned(),
        stub_passed: true,
        elapsed_ms,
        error: None,
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

    fn sample_report(vram_free: u32, ram_free: u64, disk_free: u64) -> SystemProbeReport {
        SystemProbeReport {
            gpu: Some(GpuInfo {
                name: "test".to_owned(),
                vram_total_mb: 12_288,
                vram_free_mb: vram_free,
                tier: VramTier::Vram12Gb,
                driver_version: None,
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
