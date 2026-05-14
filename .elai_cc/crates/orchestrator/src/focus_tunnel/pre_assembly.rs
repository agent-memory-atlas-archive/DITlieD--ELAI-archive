//! Pipeline pre-assembly — B-28 E-5a (library tier).
//!
//! Spec: `workflow/papers/focus-tunnel-design-interface.md`
//!   * L38-43  Pipeline parallelism narrative.
//!   * L54-58  ContextPreAssembler interface shape — depth=1, adaptive
//!             disable at validation_pass_rate < 0.80.
//!   * L101-107 Pipeline parallelism BEHAVIOURAL CONTRACTS — pre-assembly
//!             during step N validation, discard on N failure, 10-50ms
//!             wasted on discard, 100-500ms saved on pass.
//!   * L195-197 Adaptive disable on validation failure spike.
//!   * L227-228 CRITICAL ASSERTIONS:
//!             `pre_assembled_context_discarded if step_n_validation_failed`
//!             `pipeline_depth == 1`.
//!
//! Library-tier shape — synchronous, no I/O, no async, no tokio. Mirrors the
//! E-4 [`crate::focus_tunnel::streaming::StreamingValidator`] posture: the
//! WIRING phase (post-E-7) wraps [`PipelinePreAssembler::prepare`] in a
//! [`tokio::spawn`] when it wants to overlap pre-assembly with step N's
//! validation. The library does not pre-decide the concurrency policy.
//!
//! SSB integration is DEFERRED to sub-phase E-5b. The reader-side aggregator
//! [`pre_assembly_stats_for_payloads`] is shipped in E-5a but matches no
//! [`crate::ssb::SsbPayload`] variant today (the
//! `SsbPayload::PipelinePreAssembled` variant lands in E-5b when ssb.rs
//! releases from session s-28564). The aggregator therefore returns
//! [`PreAssemblyStats::default`] for any input — the API contract is locked
//! so E-5b only needs to plug in the match arm.

use std::collections::VecDeque;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cloud_mode::ExecMode;
use crate::context_budget::ModelFeatures;
use crate::focus_tunnel::mode::{build_step_context_from, ModeSelection, ModeSelector};
use crate::research_tree::BabyStepSpec;
use crate::ssb::SsbPayload;

// ---------------------------------------------------------------------------
// Constants — defaults locked by task brief and spec L57-58.
// ---------------------------------------------------------------------------

/// Default rolling window size for the validation pass-rate tracker. Spec
/// phrasing "validation_pass_rate < 0.80" is interpreted as a ratio over the
/// last N=10 baby-step outcomes.
pub const DEFAULT_HISTORY_WINDOW: usize = 10;

/// Default adaptive-disable threshold (spec L58). `pass_rate < THRESHOLD` →
/// [`PreAssembled::Disabled`]; `pass_rate >= THRESHOLD` → continue to
/// [`PreAssembled::Ready`].
pub const DEFAULT_ADAPTIVE_THRESHOLD: f32 = 0.80;

/// Cold-start (empty window) pass-rate. Optimistic — kicks off pre-assembly
/// from step 0 rather than waiting for `history_window` samples.
pub const DEFAULT_COLD_START_PASS_RATE: f32 = 1.0;

/// Spec hard-limit (L57, L228): only one pre-assembled context may be
/// in-flight at any time.
pub const MAX_DEPTH: u8 = 1;

/// Maximum length of `task_summary` (mirrors `truncate_for_log` at the
/// task_runner call site, conservative cap so the SSB row stays compact).
const TASK_SUMMARY_MAX_LEN: usize = 120;

// ---------------------------------------------------------------------------
// Clock trait — injected for deterministic tests.
// ---------------------------------------------------------------------------

/// Wall-clock abstraction for testable timestamps. The single method
/// [`Clock::now_ms`] returns milliseconds since the UNIX epoch (or any
/// monotonic source acceptable to the WIRING phase — the value is opaque to
/// the library, used only as the `assembled_at_ms` field on
/// [`PreAssembledContext`]).
pub trait Clock: Send + Sync + fmt::Debug {
    fn now_ms(&self) -> u64;
}

/// Production clock — wraps [`std::time::SystemTime`].
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemTimeClock;

impl Clock for SystemTimeClock {
    fn now_ms(&self) -> u64 {
        // Use `unwrap_or` instead of `.unwrap()` so a clock skew (system
        // clock pre-1970) does not panic the orchestrator. Zero is an
        // acceptable degraded value here — the field is informational.
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        // Saturating-cast to u64 ms. u128 → u64 narrows safely for any value
        // representable in current epoch; `try_into` would panic on overflow
        // which is the wrong failure mode for an informational metric.
        let millis_u128 = duration.as_millis();
        if millis_u128 > u128::from(u64::MAX) {
            u64::MAX
        } else {
            millis_u128 as u64
        }
    }
}

// ---------------------------------------------------------------------------
// PreAssembledContext — the materialised payload returned by prepare().
// ---------------------------------------------------------------------------

/// Pre-assembled context for the NEXT baby step. Mirrors the inputs that
/// `task_runner::drive_task` inlines for step N today (task_summary,
/// target_file path, mode_selection) but pre-computed for step N+1 while
/// step N is still running.
///
/// E-5a populates:
///   * `baby_step_id`, `task_summary`, `target_files`, `mode_hint` — from the
///     supplied [`BabyStepSpec`] and target_files slice.
///   * `pre_resolved_imports`, `referenced_symbols` — always empty in E-5a;
///     real population requires the KnowledgeHandle trait which ships in a
///     later phase (see `workflow/sessions/s-27436/context-package-E5.md`
///     VALIDATION WARNINGS MED).
///   * `assembled_at_ms` — from the injected [`Clock`].
///   * `bytes_assembled` — heuristic sum of materialised fields (task summary
///     + target paths). The metric is informational; precision is not
///     load-bearing in E-5a.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreAssembledContext {
    /// The BabyStepId.0 value for the pre-assembled step.
    pub baby_step_id: u64,
    /// Truncated task description (`<= TASK_SUMMARY_MAX_LEN` bytes).
    pub task_summary: String,
    /// Target file paths the upcoming step is expected to edit. The set used
    /// by [`PipelinePreAssembler::on_step_outcome`] to detect a
    /// target-file-change → discard event (see spec L227).
    pub target_files: Vec<PathBuf>,
    /// Pre-resolved imports ready to splice into a FIM prefix or STRUCTURED
    /// prompt. Empty in E-5a — populated by a follow-on phase that wires
    /// the dependency resolver into pre-assembly.
    pub pre_resolved_imports: Vec<String>,
    /// Referenced-symbol snippets (e.g. "fn foo at src/foo.rs:42"). Empty in
    /// E-5a — populated by a follow-on phase that wires KnowledgeHandle.
    pub referenced_symbols: Vec<String>,
    /// Pre-computed ModeSelector result for step N+1, so the WIRING phase
    /// can read the selection without re-running the selector.
    pub mode_hint: Option<ModeSelection>,
    /// Wall-clock ms at assembly time (from the injected [`Clock`]).
    pub assembled_at_ms: u64,
    /// Heuristic byte count of the materialised payload, for SSB metrics.
    pub bytes_assembled: u64,
}

// ---------------------------------------------------------------------------
// PreAssembled — outcome of prepare().
// ---------------------------------------------------------------------------

/// Outcome of one [`PipelinePreAssembler::prepare`] call.
///
/// The three variants correspond to the spec's three terminal states for
/// pre-assembly: success (Ready), adaptive backoff (Disabled), and depth-cap
/// rejection (AlreadyHaveOneInFlight). The caller pattern-matches and may
/// emit a `pipeline_pre_assembled` SSB row (E-5b) per the outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreAssembled {
    /// Pre-assembly succeeded. The `payload` is stored in-flight on the
    /// assembler; call [`PipelinePreAssembler::discard`] when step N's
    /// outcome invalidates it (target_file change, step failure).
    Ready {
        step_idx: usize,
        payload: PreAssembledContext,
    },
    /// Rolling pass-rate fell below the adaptive threshold — pre-assembly
    /// is gated off until the rate recovers.
    Disabled { reason: DisableReason },
    /// Depth cap (1) already in use. Caller MUST call
    /// [`PipelinePreAssembler::discard`] for `existing_step_idx` before
    /// re-attempting.
    AlreadyHaveOneInFlight { existing_step_idx: usize },
}

/// Why pre-assembly was disabled. Single variant in E-5a; extension space
/// (e.g. `ManualKillSwitch`, `DepthCapReached` as a distinct rejection
/// reason) reserved for later sub-phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisableReason {
    /// Rolling validation pass-rate fell below the configured threshold and
    /// the rolling window is full.
    AdaptiveThreshold,
}

// ---------------------------------------------------------------------------
// PipelinePreAssembler — the core type.
// ---------------------------------------------------------------------------

/// Library-tier pre-assembler with rolling pass-rate gating + depth=1 cap.
///
/// Construct via [`PipelinePreAssembler::new`] and tune with the builder
/// setters [`set_history_window`](Self::set_history_window),
/// [`set_adaptive_threshold`](Self::set_adaptive_threshold),
/// [`set_clock`](Self::set_clock). The synchronous [`prepare`](Self::prepare)
/// method consumes a [`BabyStepSpec`] for the upcoming step and returns a
/// [`PreAssembled`] outcome; [`on_step_outcome`](Self::on_step_outcome)
/// records the per-step pass/fail signal and, when the next step's
/// target_files are disjoint from the in-flight payload's, transparently
/// discards the stored context.
#[derive(Debug)]
pub struct PipelinePreAssembler {
    history_window: usize,
    adaptive_threshold: f32,
    cold_start_pass_rate: f32,
    clock: Arc<dyn Clock>,
    pass_history: VecDeque<bool>,
    in_flight: Option<PreAssembledContext>,
    in_flight_step_idx: Option<usize>,
    mode_selector: ModeSelector,
}

impl Default for PipelinePreAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelinePreAssembler {
    /// Construct a pre-assembler with locked defaults
    /// (`history_window=10`, `adaptive_threshold=0.80`,
    /// `cold_start_pass_rate=1.0`, clock = [`SystemTimeClock`]).
    #[must_use]
    pub fn new() -> Self {
        Self {
            history_window: DEFAULT_HISTORY_WINDOW,
            adaptive_threshold: DEFAULT_ADAPTIVE_THRESHOLD,
            cold_start_pass_rate: DEFAULT_COLD_START_PASS_RATE,
            clock: Arc::new(SystemTimeClock),
            pass_history: VecDeque::with_capacity(DEFAULT_HISTORY_WINDOW),
            in_flight: None,
            in_flight_step_idx: None,
            mode_selector: ModeSelector::new(),
        }
    }

    /// Override the rolling pass-rate window size. A `window=0` is coerced
    /// to 1 (one outcome retained) to keep the invariant
    /// `pass_history.len() <= history_window` non-degenerate.
    #[must_use]
    pub fn set_history_window(mut self, window: usize) -> Self {
        self.history_window = window.max(1);
        // Re-allocate the deque with the new capacity hint; preserve any
        // already-recorded outcomes that still fit.
        let mut fresh = VecDeque::with_capacity(self.history_window);
        for v in self.pass_history.iter().copied() {
            if fresh.len() == self.history_window {
                fresh.pop_front();
            }
            fresh.push_back(v);
        }
        self.pass_history = fresh;
        self
    }

    /// Override the adaptive-disable threshold. Clamped to `[0.0, 1.0]` so
    /// runaway configs cannot disable pre-assembly forever or never
    /// (`threshold > 1.0` would force eternal Disabled).
    #[must_use]
    pub fn set_adaptive_threshold(mut self, threshold: f32) -> Self {
        self.adaptive_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Inject a custom [`Clock`]. Production uses [`SystemTimeClock`]; tests
    /// inject a pinned implementation to verify deterministic
    /// `assembled_at_ms` values.
    #[must_use]
    pub fn set_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Returns the current rolling pass-rate. Empty window returns
    /// `cold_start_pass_rate` (default 1.0).
    #[must_use]
    pub fn current_pass_rate(&self) -> f32 {
        let n = self.pass_history.len();
        if n == 0 {
            return self.cold_start_pass_rate;
        }
        let passes = self.pass_history.iter().filter(|v| **v).count();
        // `n` cannot be zero here, division is safe.
        let rate = passes as f32 / n as f32;
        rate.clamp(0.0, 1.0)
    }

    /// `true` iff [`current_pass_rate`](Self::current_pass_rate) is strictly
    /// below the configured threshold AND the window is full (so we have
    /// confidence in the signal).
    #[must_use]
    pub fn is_adaptively_disabled(&self) -> bool {
        self.pass_history.len() >= self.history_window
            && self.current_pass_rate() < self.adaptive_threshold
    }

    /// `true` iff an in-flight pre-assembled context is stored.
    #[must_use]
    pub fn has_in_flight(&self) -> bool {
        self.in_flight.is_some()
    }

    /// Current pipeline depth (0 or 1).
    #[must_use]
    pub fn current_depth(&self) -> u8 {
        if self.in_flight.is_some() {
            1
        } else {
            0
        }
    }

    /// Attempt to pre-assemble the context for `next_step` (step N+1 in the
    /// drive_task loop). See [`PreAssembled`] for outcome variants.
    ///
    /// Synchronous, no I/O. The WIRING phase wraps this in `tokio::spawn`
    /// if it wants to overlap with step N's validation.
    #[must_use]
    pub fn prepare(
        &mut self,
        step_idx: usize,
        next_step: &BabyStepSpec,
        target_files: &[PathBuf],
        supports_fim: bool,
        exec_mode: ExecMode,
    ) -> PreAssembled {
        // Depth cap — spec L57, L228.
        if let Some(existing) = self.in_flight_step_idx {
            return PreAssembled::AlreadyHaveOneInFlight {
                existing_step_idx: existing,
            };
        }

        // Adaptive threshold gate — spec L58, L195-197.
        if self.is_adaptively_disabled() {
            return PreAssembled::Disabled {
                reason: DisableReason::AdaptiveThreshold,
            };
        }

        // Materialise the payload. Sync only — no file reads in E-5a.
        let task_summary = truncate_task_summary(&next_step.task);
        let targets: Vec<PathBuf> = target_files.to_vec();
        let mode_hint = compute_mode_hint(&self.mode_selector, next_step, supports_fim, exec_mode);
        let assembled_at_ms = self.clock.now_ms();
        let bytes_assembled = estimate_bytes(&task_summary, &targets);

        let payload = PreAssembledContext {
            baby_step_id: next_step.id.0,
            task_summary,
            target_files: targets,
            pre_resolved_imports: Vec::new(),
            referenced_symbols: Vec::new(),
            mode_hint,
            assembled_at_ms,
            bytes_assembled,
        };

        self.in_flight = Some(payload.clone());
        self.in_flight_step_idx = Some(step_idx);

        PreAssembled::Ready {
            step_idx,
            payload,
        }
    }

    /// Record the validation outcome for step N. `next_step_target_files`
    /// is the next step's expected target_files set; when non-empty AND
    /// disjoint from the in-flight payload's `target_files`, the in-flight
    /// payload is discarded (spec L227 — discard when output changes the
    /// target file).
    ///
    /// Pass an empty slice (`&[]`) when the caller has no target information
    /// for the next step or when no discard check is required.
    pub fn on_step_outcome(
        &mut self,
        _step_idx: usize,
        passed: bool,
        next_step_target_files: &[PathBuf],
    ) {
        // 1. Update rolling window.
        if self.pass_history.len() >= self.history_window {
            self.pass_history.pop_front();
        }
        self.pass_history.push_back(passed);

        // 2. Optional target-change discard.
        if !next_step_target_files.is_empty() {
            let should_discard = match &self.in_flight {
                Some(stored) => paths_disjoint(&stored.target_files, next_step_target_files),
                None => false,
            };
            if should_discard {
                if let Some(idx) = self.in_flight_step_idx {
                    let _ = self.discard(idx);
                }
            }
        }
    }

    /// Drop the in-flight pre-assembled context. Idempotent — calling this
    /// when nothing is stored is a no-op.
    ///
    /// Returns the formerly-stored [`PreAssembledContext`] when a payload
    /// was discarded, so the caller can emit an SSB row reflecting the
    /// `was_discarded=true` event (E-5b).
    pub fn discard(&mut self, _step_idx: usize) -> Option<PreAssembledContext> {
        self.in_flight_step_idx = None;
        self.in_flight.take()
    }
}

// ---------------------------------------------------------------------------
// Helpers — kept private so the public surface stays small.
// ---------------------------------------------------------------------------

fn truncate_task_summary(task: &str) -> String {
    if task.len() <= TASK_SUMMARY_MAX_LEN {
        return task.to_owned();
    }
    // Truncate at a UTF-8 char boundary so we never produce invalid strings.
    let mut end = TASK_SUMMARY_MAX_LEN;
    while end > 0 && !task.is_char_boundary(end) {
        end -= 1;
    }
    task[..end].to_owned()
}

fn estimate_bytes(task_summary: &str, target_files: &[PathBuf]) -> u64 {
    let summary_bytes = task_summary.len() as u64;
    let target_bytes: u64 = target_files
        .iter()
        .map(|p| p.as_os_str().len() as u64)
        .sum();
    summary_bytes.saturating_add(target_bytes)
}

fn compute_mode_hint(
    selector: &ModeSelector,
    spec: &BabyStepSpec,
    supports_fim: bool,
    exec_mode: ExecMode,
) -> Option<ModeSelection> {
    let step_ctx = build_step_context_from(spec);
    let mut features = ModelFeatures::none();
    features.supports_fim = supports_fim;
    Some(selector.select(&step_ctx, &features, exec_mode))
}

fn paths_disjoint(a: &[PathBuf], b: &[PathBuf]) -> bool {
    for pa in a {
        for pb in b {
            if same_path(pa, pb) {
                return false;
            }
        }
    }
    true
}

fn same_path(a: &Path, b: &Path) -> bool {
    a == b
}

// ---------------------------------------------------------------------------
// Reader-side aggregator — parallels mode_distribution_for_payloads.
// ---------------------------------------------------------------------------

/// Aggregated statistics derived from a slice of SSB payloads. Mirrors
/// [`crate::focus_tunnel::mode::mode_distribution_for_payloads`] and
/// [`crate::focus_tunnel::streaming::streaming_early_termination_stats_for_payloads`].
///
/// E-5a contract: the function exists and accepts `&[SsbPayload]`. The
/// `SsbPayload::PipelinePreAssembled` variant does NOT exist yet (lands in
/// E-5b once ssb.rs releases from session s-28564), so the aggregator
/// matches no variant and returns [`PreAssemblyStats::default`] regardless
/// of input. The locked API surface means E-5b only needs to plug in the
/// match arm — callers and tests remain stable.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PreAssemblyStats {
    pub total_assembled: u64,
    pub total_discarded: u64,
    pub waste_rate: f32,
    pub avg_depth: f32,
    /// B-28 WIRING-1: cumulative bytes_assembled across non-discarded rows
    /// (was_discarded == false). Lets the dashboard render the throughput
    /// gauge without re-scanning rows.
    pub total_bytes_assembled: u64,
}

/// Aggregate a slice of [`SsbPayload`] into [`PreAssemblyStats`].
///
/// B-28 WIRING-1: scans `SsbPayload::PipelinePreAssembled` rows and produces
/// pass/discard counts + waste-rate + avg in-flight depth + cumulative bytes.
/// Any other payload variants are skipped, so callers may pass an
/// unfiltered SSB stream.
#[must_use]
pub fn pre_assembly_stats_for_payloads(payloads: &[SsbPayload]) -> PreAssemblyStats {
    let mut total_assembled = 0u64;
    let mut total_discarded = 0u64;
    let mut depth_sum: u64 = 0;
    let mut depth_count: u64 = 0;
    let mut total_bytes_assembled = 0u64;
    for payload in payloads {
        if let SsbPayload::PipelinePreAssembled(p) = payload {
            if p.was_discarded {
                total_discarded = total_discarded.saturating_add(1);
            } else {
                total_assembled = total_assembled.saturating_add(1);
                total_bytes_assembled =
                    total_bytes_assembled.saturating_add(p.bytes_assembled);
            }
            depth_sum = depth_sum.saturating_add(u64::from(p.depth_at_time));
            depth_count = depth_count.saturating_add(1);
        }
    }
    let total = total_assembled.saturating_add(total_discarded);
    let waste_rate = if total == 0 {
        0.0
    } else {
        (total_discarded as f32 / total as f32).clamp(0.0, 1.0)
    };
    let avg_depth = if depth_count == 0 {
        0.0
    } else {
        depth_sum as f32 / depth_count as f32
    };
    PreAssemblyStats {
        total_assembled,
        total_discarded,
        waste_rate,
        avg_depth,
        total_bytes_assembled,
    }
}

// ---------------------------------------------------------------------------
// Inline unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research_tree::{BabyStepGrounding, BabyStepId};
    use std::sync::Mutex;

    #[derive(Debug)]
    struct FixedClock(Mutex<u64>);

    impl Clock for FixedClock {
        fn now_ms(&self) -> u64 {
            *self.0.lock().expect("fixed clock mutex poisoned")
        }
    }

    fn spec(id: u64, task: &str) -> BabyStepSpec {
        BabyStepSpec {
            id: BabyStepId(id),
            task: task.to_string(),
            estimated_tokens: 128,
            grounding: BabyStepGrounding::default(),
            tdd: None,
            acceptance: None,
            preferred_mode: None,
            target_file: None,
            edit_region: None,
        }
    }

    #[test]
    fn cold_start_pass_rate_is_one() {
        let p = PipelinePreAssembler::new();
        assert!((p.current_pass_rate() - 1.0).abs() < f32::EPSILON);
        assert!(!p.is_adaptively_disabled());
    }

    #[test]
    fn window_eviction_evicts_oldest() {
        let mut p = PipelinePreAssembler::new().set_history_window(3);
        p.on_step_outcome(0, true, &[]);
        p.on_step_outcome(1, true, &[]);
        p.on_step_outcome(2, false, &[]);
        // rate = 2/3
        assert!((p.current_pass_rate() - 2.0 / 3.0).abs() < 1e-6);
        // push 4th — first (true) evicted, leaving [true,false,false] = 1/3.
        p.on_step_outcome(3, false, &[]);
        assert!((p.current_pass_rate() - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn threshold_clamped() {
        let p = PipelinePreAssembler::new().set_adaptive_threshold(-1.0);
        assert!((p.adaptive_threshold - 0.0).abs() < f32::EPSILON);
        let p2 = PipelinePreAssembler::new().set_adaptive_threshold(2.0);
        assert!((p2.adaptive_threshold - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn injected_clock_pins_assembled_at_ms() {
        let mut p = PipelinePreAssembler::new().set_clock(Arc::new(FixedClock(Mutex::new(7777))));
        let s = spec(1, "edit foo");
        let out = p.prepare(0, &s, &[PathBuf::from("foo.rs")], true, ExecMode::Local);
        match out {
            PreAssembled::Ready { payload, .. } => {
                assert_eq!(payload.assembled_at_ms, 7777);
            }
            other => panic!("expected Ready, got {other:?}"),
        }
    }

    #[test]
    fn depth_cap_blocks_second_prepare() {
        let mut p = PipelinePreAssembler::new();
        let s = spec(1, "edit foo");
        let _ = p.prepare(0, &s, &[PathBuf::from("foo.rs")], true, ExecMode::Local);
        let out = p.prepare(1, &s, &[PathBuf::from("bar.rs")], true, ExecMode::Local);
        assert!(matches!(
            out,
            PreAssembled::AlreadyHaveOneInFlight { existing_step_idx: 0 }
        ));
    }

    #[test]
    fn discard_is_idempotent() {
        let mut p = PipelinePreAssembler::new();
        let s = spec(1, "edit foo");
        let _ = p.prepare(0, &s, &[PathBuf::from("foo.rs")], true, ExecMode::Local);
        assert!(p.discard(0).is_some());
        assert!(p.discard(0).is_none());
    }

    #[test]
    fn target_change_via_outcome_discards() {
        let mut p = PipelinePreAssembler::new();
        let s = spec(1, "edit foo");
        let _ = p.prepare(0, &s, &[PathBuf::from("foo.rs")], true, ExecMode::Local);
        assert!(p.has_in_flight());
        p.on_step_outcome(0, true, &[PathBuf::from("different.rs")]);
        assert!(!p.has_in_flight(), "disjoint target_files should discard");
    }

    #[test]
    fn target_overlap_via_outcome_preserves() {
        let mut p = PipelinePreAssembler::new();
        let s = spec(1, "edit foo");
        let _ = p.prepare(0, &s, &[PathBuf::from("foo.rs")], true, ExecMode::Local);
        p.on_step_outcome(0, true, &[PathBuf::from("foo.rs")]);
        assert!(p.has_in_flight(), "overlapping targets must NOT discard");
    }

    #[test]
    fn stats_for_empty_returns_default() {
        let stats = pre_assembly_stats_for_payloads(&[]);
        assert_eq!(stats, PreAssemblyStats::default());
    }

    #[test]
    fn truncate_summary_respects_utf8_boundary() {
        let long = "é".repeat(200); // 2 bytes per char × 200 = 400 bytes
        let truncated = truncate_task_summary(&long);
        assert!(truncated.len() <= TASK_SUMMARY_MAX_LEN);
        // Must be valid UTF-8.
        let _: &str = &truncated;
    }
}
