//! C-10 — Phase 4G.9 anti-spiral / lateral-thinking primitives.
//!
//! Task-runner step-boundary detector (a different altitude than the
//! existing GRC step-level `anti_spiral.rs` SpiralTracker). Both modules
//! coexist:
//!   * `anti_spiral.rs` operates per-GRC reasoning step, single-pattern
//!     (Spinning), surfaces `SpiralTracker.hard_stop()` into
//!     `GateContext.anti_spiral_fired`.
//!   * `anti_spiral_extended.rs` (this module) operates per task baby-step,
//!     4 stagnation patterns + 5-persona lateral thinking switch +
//!     GoalDriftCanary + GenerationWatchdog + AdaptiveFutilityTracker.
//!
//! Plan: docs/thefinalglance/C-plan/C-10_anti_spiral_primitives.md
//! Decisions: workflow/reasoning/C-10-decisions.md
//!
//! Determinism notes:
//!   * GoalDriftCanary tokenizes with `BTreeSet` (not `HashSet`) so the
//!     Jaccard score is reproducible across runs / platforms.
//!   * GenerationWatchdog fingerprints use blake3 (verified present in
//!     workspace Cargo.toml; orchestrator Cargo.toml line 64).
//!   * Affinity table (`suggest_persona_for_pattern`) follows the plan
//!     §Task 1 mapping verbatim. See C-10-decisions.md DRIFT-4 for the
//!     three conflicting sources and why the plan's table wins.
//!   * GoalDriftCanary verdict bands follow the plan-direction reading
//!     (Jaccard-similarity; higher = aligned, lower = drifted). See
//!     C-10-decisions.md DRIFT-3 for the spec-direction (1.0 - jaccard)
//!     trade-off — the field name `drift_score` in
//!     `GoalDriftWarningPayload` retains its C-2 schema-frozen name; the
//!     value carried is the Jaccard similarity itself.
//!
//! No `.unwrap()` / `.expect()` in this module. Stdlib + blake3 only.

use std::collections::{BTreeSet, HashMap, VecDeque};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Named constants (no magic numbers per CLAUDE.md anti-stale rules)
// ---------------------------------------------------------------------------

/// Number of consecutive identical `error_class` strings that flips the
/// Spinning verdict. Plan §Task 1: `spin_threshold = 3`.
pub const SPIN_THRESHOLD: usize = 3;

/// Trailing-history window inspected for the Oscillation pattern.
/// Plan §Task 1: `oscillation_window = 4` (period-2 A-B-A-B over 4 entries).
pub const OSC_WINDOW: usize = 4;

/// Trailing-history window inspected for the DiminishingReturns pattern.
/// Plan §Task 1: `diminishing_min_window = 4`.
pub const DIMINISHING_MIN_WINDOW: usize = 4;

/// Minimum per-step improvement, in percent of the previous attempt's
/// error_count, that prevents the DiminishingReturns verdict. Plan §Task 1:
/// `diminishing_min_improvement_pct = 5.0`.
pub const DIMINISHING_MIN_IMPROVEMENT_PCT: f32 = 5.0;

/// ExecutionHistory ring-buffer capacity. Plan §Task 1: 32 entries.
pub const HISTORY_CAPACITY: usize = 32;

/// GoalDriftCanary upper threshold: Jaccard >= 0.7 -> verdict::Green.
pub const JACCARD_GREEN: f32 = 0.7;

/// GoalDriftCanary lower threshold: Jaccard < 0.4 -> verdict::Red.
pub const JACCARD_YELLOW: f32 = 0.4;

/// GenerationWatchdog: idle (no SSB event) timeout. 2 minutes.
pub const IDLE_TIMEOUT_MS: u64 = 120_000;

/// GenerationWatchdog: same fingerprint (no material progress) timeout.
/// 10 minutes per spec §4G.9 task D.
pub const NO_PROGRESS_TIMEOUT_MS: u64 = 600_000;

/// GenerationWatchdog: safety ceiling — 30 minutes regardless of activity.
pub const SAFETY_TIMEOUT_MS: u64 = 1_800_000;

/// AdaptiveFutilityTracker: rate below this is considered futile (15%).
pub const FUTILITY_THRESHOLD: f32 = 0.15;

/// AdaptiveFutilityTracker: minimum attempts before futile verdict fires.
pub const FUTILITY_MIN_ATTEMPTS: u32 = 4;

// ---------------------------------------------------------------------------
// StagnationPattern + AttemptResult + ExecutionHistory
// ---------------------------------------------------------------------------

/// Detector verdicts. `Spinning` and `Oscillation` map 1:1 to the C-2 SSB
/// variants; `NoDrift` and `DiminishingReturns` piggyback on the existing
/// `StagnationSpinningDetected` / `StagnationOscillationDetected` rows via
/// the payload `pattern` field — see workflow/reasoning/C-10-decisions.md
/// DRIFT-2 + D2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StagnationPattern {
    /// Same `error_class` repeated `SPIN_THRESHOLD` times.
    Spinning,
    /// Last `OSC_WINDOW` entries alternate between exactly two error_class
    /// strings (e.g. A B A B).
    Oscillation,
    /// Last two `output_hash` values are identical — no material change
    /// between consecutive attempts.
    NoDrift,
    /// Error-count improvement is below `DIMINISHING_MIN_IMPROVEMENT_PCT`
    /// per step across a `DIMINISHING_MIN_WINDOW`-attempt window.
    DiminishingReturns,
}

impl StagnationPattern {
    /// Snake_case label written into the SSB payload `pattern` field.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Spinning => "spinning",
            Self::Oscillation => "oscillation",
            Self::NoDrift => "no_drift",
            Self::DiminishingReturns => "diminishing_returns",
        }
    }
}

/// One row of attempt-level history fed into `StagnationDetection`.
/// Produced by task_runner at each baby-step completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptResult {
    pub attempt_index: u32,
    pub error_class: String,
    pub output_hash: u64,
    pub error_count: u32,
    pub timestamp_ms: u64,
}

/// Ring-buffer history (newest at the back). Capacity bounded by
/// `HISTORY_CAPACITY`; older entries are evicted on push.
#[derive(Debug, Clone, Default)]
pub struct ExecutionHistory {
    inner: VecDeque<AttemptResult>,
}

impl ExecutionHistory {
    pub fn new() -> Self {
        Self {
            inner: VecDeque::with_capacity(HISTORY_CAPACITY),
        }
    }

    pub fn capacity(&self) -> usize {
        HISTORY_CAPACITY
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Pushes a new attempt. Evicts the oldest entry when the buffer is at
    /// capacity.
    pub fn push(&mut self, attempt: AttemptResult) {
        if self.inner.len() == HISTORY_CAPACITY {
            self.inner.pop_front();
        }
        self.inner.push_back(attempt);
    }

    pub fn is_full(&self) -> bool {
        self.inner.len() == HISTORY_CAPACITY
    }

    /// Returns the most recent `n` attempts (or all entries if `n > len`),
    /// oldest first within the returned slice.
    pub fn last_n(&self, n: usize) -> Vec<&AttemptResult> {
        let take = n.min(self.inner.len());
        let start = self.inner.len() - take;
        self.inner.range(start..).collect()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Iterator over all entries (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &AttemptResult> {
        self.inner.iter()
    }
}

// ---------------------------------------------------------------------------
// StagnationDetection
// ---------------------------------------------------------------------------

/// Configurable detector. Defaults follow the plan §Task 1 constants.
#[derive(Debug, Clone, Copy)]
pub struct StagnationDetection {
    pub spin_threshold: usize,
    pub oscillation_window: usize,
    pub diminishing_min_window: usize,
    pub diminishing_min_improvement_pct: f32,
}

impl Default for StagnationDetection {
    fn default() -> Self {
        Self {
            spin_threshold: SPIN_THRESHOLD,
            oscillation_window: OSC_WINDOW,
            diminishing_min_window: DIMINISHING_MIN_WINDOW,
            diminishing_min_improvement_pct: DIMINISHING_MIN_IMPROVEMENT_PCT,
        }
    }
}

impl StagnationDetection {
    /// Detect a stagnation pattern in the supplied history. Returns `None`
    /// when history is shorter than 2 (need at least two attempts to
    /// observe any pattern). Rule precedence:
    ///   1. NoDrift  (last two output_hash equal)
    ///   2. Spinning (last `spin_threshold` error_class identical)
    ///   3. Oscillation (last `oscillation_window` alternate between 2)
    ///   4. DiminishingReturns (4-attempt window, <5% improvement / step)
    /// First match wins; later rules are skipped.
    pub fn detect_pattern(&self, history: &ExecutionHistory) -> Option<StagnationPattern> {
        if history.len() < 2 {
            return None;
        }
        // Rule 1 — NoDrift: last two output_hash equal.
        let last_two = history.last_n(2);
        if let (Some(a), Some(b)) = (last_two.first(), last_two.last()) {
            if a.output_hash == b.output_hash {
                return Some(StagnationPattern::NoDrift);
            }
        }

        // Rule 2 — Spinning: last `spin_threshold` error_class identical.
        if history.len() >= self.spin_threshold {
            let tail = history.last_n(self.spin_threshold);
            let first_err = tail.first().map(|a| a.error_class.as_str());
            if let Some(first) = first_err {
                if !first.is_empty() && tail.iter().all(|a| a.error_class == first) {
                    return Some(StagnationPattern::Spinning);
                }
            }
        }

        // Rule 3 — Oscillation: window of size `oscillation_window`
        // alternates between exactly two distinct error_class strings.
        if history.len() >= self.oscillation_window && self.oscillation_window >= 4 {
            let tail = history.last_n(self.oscillation_window);
            let distinct: BTreeSet<&str> = tail.iter().map(|a| a.error_class.as_str()).collect();
            if distinct.len() == 2 {
                let mut alternating = true;
                for i in 2..tail.len() {
                    if tail[i].error_class != tail[i - 2].error_class
                        || tail[i].error_class == tail[i - 1].error_class
                    {
                        alternating = false;
                        break;
                    }
                }
                if alternating {
                    return Some(StagnationPattern::Oscillation);
                }
            }
        }

        // Rule 4 — DiminishingReturns: error_count improvement per step
        // averages below threshold across a window.
        if history.len() >= self.diminishing_min_window {
            let tail = history.last_n(self.diminishing_min_window);
            let mut total_pct = 0.0_f32;
            let mut steps = 0_u32;
            for w in tail.windows(2) {
                let prev = w[0].error_count as f32;
                let cur = w[1].error_count as f32;
                if prev <= 0.0 {
                    continue;
                }
                let improvement = (prev - cur).max(0.0);
                let pct = (improvement / prev) * 100.0;
                total_pct += pct;
                steps += 1;
            }
            if steps > 0 {
                let avg = total_pct / steps as f32;
                if avg < self.diminishing_min_improvement_pct {
                    return Some(StagnationPattern::DiminishingReturns);
                }
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// ThinkingPersona + PersonaStrategy
// ---------------------------------------------------------------------------

/// Lateral-thinking personas the runtime can switch to when a stagnation
/// pattern fires. Each persona has a corresponding `.claude/agents/persona-*.md`
/// file consumed by the agent-spawn path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingPersona {
    Hacker,
    Researcher,
    Simplifier,
    Architect,
    Contrarian,
}

impl ThinkingPersona {
    /// Snake_case label written into SSB payload `persona` fields.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hacker => "hacker",
            Self::Researcher => "researcher",
            Self::Simplifier => "simplifier",
            Self::Architect => "architect",
            Self::Contrarian => "contrarian",
        }
    }

    /// Stable index used for round-robin rotation and SSB ordering.
    pub fn index(self) -> usize {
        match self {
            Self::Hacker => 0,
            Self::Researcher => 1,
            Self::Simplifier => 2,
            Self::Architect => 3,
            Self::Contrarian => 4,
        }
    }
}

/// Static mapping between persona and the imperative prefix that
/// task_runner prepends to the next baby_step prompt when a pattern fires.
#[derive(Debug, Clone, Copy)]
pub struct PersonaStrategy {
    pub persona: ThinkingPersona,
    pub prompt_prefix: &'static str,
}

/// Stable order used by `suggest_persona_for_pattern` fallback rotation
/// and by the dashboard's persona-row rendering.
pub static PERSONA_STRATEGIES: [PersonaStrategy; 5] = [
    PersonaStrategy {
        persona: ThinkingPersona::Hacker,
        prompt_prefix:
            "Ignore elegance. Propose the shortest dirty/clean/hybrid path; commit one concrete baby_step.",
    },
    PersonaStrategy {
        persona: ThinkingPersona::Researcher,
        prompt_prefix:
            "Web-search three prior-art references; cite the most relevant; derive next action.",
    },
    PersonaStrategy {
        persona: ThinkingPersona::Simplifier,
        prompt_prefix:
            "Strip to the smallest skeleton that compiles; add layers until failure reappears; name the bad layer.",
    },
    PersonaStrategy {
        persona: ThinkingPersona::Architect,
        prompt_prefix:
            "Map the dependency graph of the oscillating errors; identify the coupling root; output a decoupling baby_step.",
    },
    PersonaStrategy {
        persona: ThinkingPersona::Contrarian,
        prompt_prefix:
            "State the opposite assumption; derive the alternative baby_step; no hedging.",
    },
];

/// Deterministic persona affinity (plan §Task 1, DRIFT-4 Source A):
///   Spinning           -> Contrarian
///   Oscillation        -> Architect
///   NoDrift            -> Hacker
///   DiminishingReturns -> Simplifier
/// `Researcher` is the catch-all fallback used by a round-robin rotation
/// in callers that prefer "try every persona at least once before
/// looping". The static rotation index is exposed via
/// `PERSONA_STRATEGIES[idx % 5]` for that purpose.
pub fn suggest_persona_for_pattern(pattern: StagnationPattern) -> ThinkingPersona {
    match pattern {
        StagnationPattern::Spinning => ThinkingPersona::Contrarian,
        StagnationPattern::Oscillation => ThinkingPersona::Architect,
        StagnationPattern::NoDrift => ThinkingPersona::Hacker,
        StagnationPattern::DiminishingReturns => ThinkingPersona::Simplifier,
    }
}

/// Returns the strategy entry for the supplied persona. Stable index in
/// `PERSONA_STRATEGIES`.
pub fn strategy_for(persona: ThinkingPersona) -> &'static PersonaStrategy {
    &PERSONA_STRATEGIES[persona.index()]
}

// ---------------------------------------------------------------------------
// GoalDriftCanary + DriftVerdict
// ---------------------------------------------------------------------------

/// Verdict bands for `GoalDriftCanary` (plan-direction Jaccard-similarity).
/// Higher score = better alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftVerdict {
    /// Jaccard similarity >= `JACCARD_GREEN` (0.7) — aligned with intent.
    Green,
    /// 0.4 <= Jaccard similarity < 0.7 — emit `GoalDriftWarning`.
    Yellow,
    /// Jaccard similarity < `JACCARD_YELLOW` (0.4) — emit `GoalDriftBlock`.
    Red,
}

/// Token-overlap canary used to detect when the active baby-step's task
/// summary has drifted away from the original task intent. `BTreeSet`
/// (NOT `HashSet`) is mandatory for determinism — see plan
/// WIRING_CONTRACT stub_ban.
///
/// NOTE on field-name semantic drift: the SSB payload
/// `GoalDriftWarningPayload.drift_score` retains its C-2 schema-frozen
/// name but carries the Jaccard *similarity* (not the divergence
/// 1.0 - jaccard). See workflow/reasoning/C-10-decisions.md DRIFT-3 +
/// D3 for the trade-off. Downstream consumers should read the value via
/// the verdict bands rather than the field name.
#[derive(Debug, Clone, Copy)]
pub struct GoalDriftCanary {
    pub green_threshold: f32,
    pub yellow_threshold: f32,
}

impl Default for GoalDriftCanary {
    fn default() -> Self {
        Self {
            green_threshold: JACCARD_GREEN,
            yellow_threshold: JACCARD_YELLOW,
        }
    }
}

impl GoalDriftCanary {
    /// Whitespace-split, strip non-alphanumeric, lowercase. Returns a
    /// Vec to preserve duplicates for callers that want frequency info;
    /// internal compute() uses `BTreeSet` to dedupe.
    pub fn tokenize(text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        for raw in text.split_whitespace() {
            let cleaned: String = raw
                .chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(|c| c.to_lowercase())
                .collect();
            if !cleaned.is_empty() {
                tokens.push(cleaned);
            }
        }
        tokens
    }

    /// Jaccard similarity computed over the `BTreeSet` of tokens from each
    /// side. Returns 0.0 when both sides are empty (canonical convention:
    /// no signal -> max drift; caller's verdict path will flag Red).
    pub fn compute(&self, original: &str, current: &str) -> f32 {
        let a_tokens = Self::tokenize(original);
        let b_tokens = Self::tokenize(current);
        let a: BTreeSet<&str> = a_tokens.iter().map(String::as_str).collect();
        let b: BTreeSet<&str> = b_tokens.iter().map(String::as_str).collect();
        if a.is_empty() && b.is_empty() {
            return 0.0;
        }
        let intersection = a.intersection(&b).count();
        let union = a.union(&b).count();
        if union == 0 {
            return 0.0;
        }
        intersection as f32 / union as f32
    }

    pub fn verdict(&self, score: f32) -> DriftVerdict {
        if score >= self.green_threshold {
            DriftVerdict::Green
        } else if score >= self.yellow_threshold {
            DriftVerdict::Yellow
        } else {
            DriftVerdict::Red
        }
    }
}

// ---------------------------------------------------------------------------
// GenerationWatchdog
// ---------------------------------------------------------------------------

/// Kind of timeout the watchdog fired on. Order in `check()` is:
/// Safety > NoMaterialProgress > Idle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchdogTimeoutKind {
    /// `IDLE_TIMEOUT_MS` elapsed since the last observable SSB event.
    Idle,
    /// `NO_PROGRESS_TIMEOUT_MS` elapsed without the workflow fingerprint
    /// changing (same row count + same output hash).
    NoMaterialProgress,
    /// `SAFETY_TIMEOUT_MS` elapsed since task start regardless of activity.
    Safety,
}

/// Per-task wall-clock watchdog. Tracks (SSB row count + output hash)
/// fingerprint via blake3 to detect "no material progress" without
/// requiring an SSB-row-content diff.
#[derive(Debug, Clone, Copy)]
pub struct GenerationWatchdog {
    pub idle_timeout_ms: u64,
    pub no_progress_timeout_ms: u64,
    pub safety_timeout_ms: u64,
    pub workflow_fingerprint: Option<u64>,
    /// Wall-clock time (ms) the fingerprint last changed. Used by
    /// `check()` to compute the "no material progress" window.
    pub last_fingerprint_change_ms: Option<u64>,
}

impl Default for GenerationWatchdog {
    fn default() -> Self {
        Self {
            idle_timeout_ms: IDLE_TIMEOUT_MS,
            no_progress_timeout_ms: NO_PROGRESS_TIMEOUT_MS,
            safety_timeout_ms: SAFETY_TIMEOUT_MS,
            workflow_fingerprint: None,
            last_fingerprint_change_ms: None,
        }
    }
}

impl GenerationWatchdog {
    /// blake3 of `ssb_row_count` || `output_hash` (16-byte concat). Returns
    /// the first 8 bytes interpreted as little-endian u64.
    pub fn compute_fingerprint(ssb_row_count: u64, output_hash: u64) -> u64 {
        let mut buf = [0_u8; 16];
        buf[..8].copy_from_slice(&ssb_row_count.to_le_bytes());
        buf[8..].copy_from_slice(&output_hash.to_le_bytes());
        let digest = blake3::hash(&buf);
        let bytes = digest.as_bytes();
        let mut head = [0_u8; 8];
        head.copy_from_slice(&bytes[..8]);
        u64::from_le_bytes(head)
    }

    /// Evaluate timeouts. Order: Safety > NoMaterialProgress > Idle.
    /// Updates `workflow_fingerprint` and `last_fingerprint_change_ms`
    /// when the fingerprint changed.
    ///
    /// * `last_event_ms` — wall-clock ms of the most recent SSB event.
    /// * `now_ms` — current wall-clock ms.
    /// * `current_fingerprint` — blake3-derived workflow fingerprint
    ///   (callers compute via `compute_fingerprint`).
    /// * `start_ms` — wall-clock ms when the task started.
    pub fn check(
        &mut self,
        last_event_ms: u64,
        now_ms: u64,
        current_fingerprint: u64,
        start_ms: u64,
    ) -> Option<WatchdogTimeoutKind> {
        // Fingerprint bookkeeping. We update *before* the timeout decisions
        // so a fingerprint-change inside this tick correctly resets the
        // "no progress" window.
        let changed = match self.workflow_fingerprint {
            Some(prev) => prev != current_fingerprint,
            None => true,
        };
        if changed {
            self.workflow_fingerprint = Some(current_fingerprint);
            self.last_fingerprint_change_ms = Some(now_ms);
        }

        // 1. Safety ceiling first (highest precedence).
        if now_ms.saturating_sub(start_ms) >= self.safety_timeout_ms {
            return Some(WatchdogTimeoutKind::Safety);
        }

        // 2. No material progress.
        if let Some(changed_ms) = self.last_fingerprint_change_ms {
            if now_ms.saturating_sub(changed_ms) >= self.no_progress_timeout_ms {
                return Some(WatchdogTimeoutKind::NoMaterialProgress);
            }
        }

        // 3. Idle (no SSB event).
        if now_ms.saturating_sub(last_event_ms) >= self.idle_timeout_ms {
            return Some(WatchdogTimeoutKind::Idle);
        }

        None
    }
}

// ---------------------------------------------------------------------------
// AdaptiveFutilityTracker
// ---------------------------------------------------------------------------

/// Per-error-class success/attempt counter. The tracker is consulted
/// BEFORE each repair-loop iteration; futile classes short-circuit to
/// `EscalationTrigger::CircuitBreaker` rather than burning more
/// attempts.
///
/// SSB serialization: the tracker snapshot is serialized into the
/// `EscalationFired.detail` JSON field at the call site (no new SsbActionType
/// variant — see C-10-decisions.md DRIFT-2).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdaptiveFutilityTracker {
    pub success_count: HashMap<String, u32>,
    pub attempt_count: HashMap<String, u32>,
}

impl AdaptiveFutilityTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one attempt; bumps the corresponding counters.
    pub fn record(&mut self, error_class: &str, succeeded: bool) {
        let key = error_class.to_owned();
        *self.attempt_count.entry(key.clone()).or_insert(0) += 1;
        if succeeded {
            *self.success_count.entry(key).or_insert(0) += 1;
        }
    }

    /// Success rate for the supplied class. Returns 0.5 (uninformative
    /// prior) when no data has been recorded yet, avoiding the
    /// false-positive "0/0 is futile" verdict.
    pub fn success_rate(&self, error_class: &str) -> f32 {
        let attempts = self.attempt_count.get(error_class).copied().unwrap_or(0);
        if attempts == 0 {
            return 0.5;
        }
        let successes = self.success_count.get(error_class).copied().unwrap_or(0);
        successes as f32 / attempts as f32
    }

    /// `true` iff the class is below the threshold AND has at least
    /// `FUTILITY_MIN_ATTEMPTS` attempts on record.
    pub fn is_futile(&self, error_class: &str, threshold: f32) -> bool {
        let attempts = self.attempt_count.get(error_class).copied().unwrap_or(0);
        if attempts < FUTILITY_MIN_ATTEMPTS {
            return false;
        }
        self.success_rate(error_class) < threshold
    }

    /// Total attempts recorded across all error classes (for SSB detail
    /// fields + dashboard rendering).
    pub fn total_attempts(&self) -> u32 {
        self.attempt_count.values().sum()
    }
}
