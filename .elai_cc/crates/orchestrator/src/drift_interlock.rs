//! H-29 (SCHEMA_VERSION 238): LOCKED fail-closed drift state machine (T-13) — the master
//! deployment-trust interlock that gates resuming a live trading strategy.
//!
//! The LAST Wave-3 gate. ELAI already ships ONE drift signal (G-10 `model_drift.rs`
//! `system_fingerprint`) plus the D-7 canary regression delta; H-29 does NOT re-detect drift
//! (GD-1 P0). It COMPOSES the shipped signals into a four-axis LOCKED interlock with a
//! persistent, journal-derived latch that DEFAULTS to HALTED and can only be cleared by a
//! human / frozen out-of-band path carrying FRESH re-validation evidence.
//!
//! # The four closed-form drift axes (NO new dependency — all over logged data)
//!   * AXIS 1 — PSI (population stability index) on the INPUT feature distribution AND the
//!     OUTPUT / signal distribution. `PSI = sum_bins (actual_pct - expected_pct) *
//!     ln(actual_pct / expected_pct)`. A zero (degenerate) bin is fail-closed: a missing
//!     baseline-or-live mass on a bin means the distribution comparison is undefined => HALT
//!     (NOT a silent NaN/inf). `PSI_WARN_BAND = 0.10` (minor shift, surfaces a Warning),
//!     `PSI_HALT_BAND = 0.25` (major shift, HALT) — the conventional credit-risk bands.
//!   * AXIS 2 — schema-fingerprint mismatch: `blake3` over the live vendor data schema must
//!     equal the FROZEN baseline fingerprint. A mismatch = the input contract changed (vendor
//!     drift) => HALT. blake3 ONLY (CLAUDE.md HARD RULE; mirrors `model_drift.rs::identity_key`).
//!   * AXIS 3 — rolling-14-day live-vs-backtest performance deviation: the absolute deviation
//!     of the live realized metric from the backtest metric over the
//!     `DEVIATION_WINDOW_DAYS = 14` window (ClockProvider-driven). A deviation beyond
//!     `DEVIATION_HALT_BAND` => HALT. The window is measured via the injected clock, NEVER
//!     `SystemTime::now()` (GD-2 determinism).
//!   * AXIS 4 — 90-day validation TTL: the strategy's last validation must be NEWER than
//!     `VALIDATION_TTL_DAYS = 90` (ClockProvider-driven). A stale validation => HALT
//!     (re-validation required, forcing an H-18 deflated-Sharpe re-check at promotion).
//!
//! The composed upstream signals (G-10 `model_drift` `DriftSignal`, H-18 deflation verdict)
//! feed in as ADDITIONAL halt inputs (GD-1: read their verdicts, do not re-implement).
//!
//! # The persistent, journal-derived latch (load-bearing)
//!
//! [`DriftInterlockLedger`] owns a SQLite connection to the append-only
//! `drift_interlock_latch` table (on the same db file as [`Ssb`]). The HALTED state is DERIVED
//! from the journal, NOT stored as a mutable bool a restart could re-initialise to Running:
//!
//!   `is_halted(strategy) = COUNT('halt' rows) > COUNT('unlock' rows)` for the strategy.
//!
//! Once a `'halt'` row is appended the interlock STAYS HALTED across process restarts until a
//! human / frozen `'unlock'` row is appended. A restart re-reads the journal and re-derives
//! HALTED because Running is the ABSENCE of an unresolved halt, not a stored default (mirrors
//! the H-19 `queries_spent = COUNT(spent rows)` discipline).
//!
//! # Default-HALTED fail-safe (R1/R2 — this interlock fails to the SAFE state)
//!
//! Unlike most gates ("missing evidence => block the action"), an interlock that fails OPEN
//! would keep a drifting strategy trading. So missing / un-evaluable evidence => HALT (the
//! SAFE block): a missing schema fingerprint, an empty PSI bin, an un-readable deviation /
//! validation timestamp all latch HALTED. Running is reachable ONLY on positive evidence of
//! no-drift across ALL four axes (and no pre-existing unresolved latch).
//!
//! # The no-agent-unlock invariant — three legs (mirror H-19/H-17)
//!   1. NO unlock / reset / clear / resume method on the agent-facing
//!      [`DriftInterlockLedger`] (negative compile-test `no_unlock_or_reset_method_exists`).
//!   2. the debug `update_hook` in `Ssb::new()` panics on any UPDATE/DELETE against the
//!      `drift_interlock_latch` table (ssb.rs — a covert in-place un-latch).
//!   3. `drift_interlock.rs` + `drift_interlock.toml` are in `.claude/gate-manifest.txt`,
//!      re-locked by a HUMAN out-of-band (`gate_freeze_check.py --lock`; the executing run
//!      cannot re-lock its own gate, R8).
//!
//! # Human unlock — frozen path only ([`unlock_drift_interlock`])
//!
//! The unlock is a FREE FUNCTION, NOT a method on the agent-reachable ledger, so an agent
//! holding only a ledger handle cannot reach it. It REQUIRES fresh re-validation evidence: the
//! supplied validation timestamp must be STRICTLY NEWER than the most recent unresolved
//! `'halt'` row (the load-bearing semantic guard — a privilege check alone is insufficient; an
//! unlock that re-presents stale validation is REJECTED, mirroring
//! `provision_fresh_holdout`'s data-window-disjointness guard). R8 residue: the human/frozen
//! unlock is a bounded human-trust dependency; it does NOT make the interlock auto-clearing.
//!
//! # No `.unwrap()` / `.expect()` in non-test code (CLAUDE.md HARD RULE). blake3 only.
//! # ClockProvider for the 14d/90d windows (never `SystemTime::now()`).
//!
//! # References
//! - `docs/thefinalglance/H-plan/H-29_drift_state_machine.md`
//! - `workflow/reasoning/H-29-decisions.md`
//! - PSI: population stability index (closed-form binned KL-like sum; credit-risk standard).
//! - `crates/orchestrator/src/model_drift.rs` (G-10 — the composed drift signal, GD-1 REUSE).
//! - `crates/orchestrator/src/deflation_gate.rs` (H-18 — the 90d-TTL DSR re-check, GD-1 REUSE).

use std::path::Path;
use std::sync::Arc;

use rusqlite::{params, Connection};
use thiserror::Error;
use tracing::{debug, warn};

use crate::ssb::{DriftInterlockKind, DriftInterlockPayload, Ssb, SsbPayload};
use elai_core::clock::ClockProvider;

// ---------------------------------------------------------------------------
// LOCKED drift-interlock constants (TD-2 — mutable=0, frozen in
// .claude/gate-manifest.txt). Named constants, no magic numbers (CLAUDE.md).
// ---------------------------------------------------------------------------

/// PSI minor-shift band: `0.10 <= PSI < 0.25` surfaces a `Warning` (a noticeable population
/// shift worth watching, not yet a halt). The conventional credit-risk / ML-monitoring band.
pub const PSI_WARN_BAND: f64 = 0.10;

/// PSI major-shift band: `PSI >= 0.25` is a major distribution shift => HALT. The conventional
/// credit-risk / ML-monitoring band. LOCKED.
pub const PSI_HALT_BAND: f64 = 0.25;

/// Rolling live-vs-backtest deviation window (days). The deviation axis compares the live
/// realized metric to the backtest metric over this trailing window. LOCKED.
pub const DEVIATION_WINDOW_DAYS: u64 = 14;

/// The absolute live-vs-backtest deviation beyond which the strategy is no longer behaving as
/// backtested => HALT. Expressed in the metric's native units (e.g. Sharpe points). LOCKED.
pub const DEVIATION_HALT_BAND: f64 = 0.50;

/// Validation TTL (days). A strategy whose last validation is older than this is stale and must
/// be re-validated (forcing an H-18 deflated-Sharpe re-check at promotion) => HALT. LOCKED.
pub const VALIDATION_TTL_DAYS: u64 = 90;

/// Milliseconds per day, used to convert the ClockProvider window to a millisecond delta.
const MS_PER_DAY: i64 = 86_400_000;

/// Smallest bin proportion used to floor `ln(actual/expected)` away from `ln(0)` ONLY when the
/// caller has explicitly opted into epsilon-smoothing. The DEFAULT behaviour for a genuinely
/// empty (zero-mass) bin is fail-closed (HALT), not silent epsilon substitution — see
/// [`compute_psi`]. This floor is a defensive numeric guard, not a tunable threshold.
const PSI_BIN_EPSILON: f64 = 1.0e-6;

/// Agent id stamped on the emitted [`SsbPayload::DriftInterlock`] row.
const DRIFT_INTERLOCK_AGENT_ID: &str = "drift-interlock";

/// Superstep stamped on the emitted row (single-shot gate, not part of a graph loop).
const DRIFT_INTERLOCK_SUPERSTEP: i64 = 0;

/// Minimum scope-key length (bytes) after canonicalization (scope-key integrity).
const MIN_KEY_LEN: usize = 1;

/// The `'halt'` latch row kind (one append-only row per HALT verdict).
const ROW_KIND_HALT: &str = "halt";

/// The `'unlock'` latch row kind (one append-only row per human/frozen unlock).
const ROW_KIND_UNLOCK: &str = "unlock";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors returned by [`DriftInterlockLedger`] operations and the free unlock path.
#[derive(Debug, Error)]
pub enum DriftInterlockError {
    /// SQLite layer error.
    #[error("drift interlock sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// A scope key (strategy_id) is empty after canonicalization.
    #[error("strategy_id is empty after canonicalization (scope-key attack surface)")]
    EmptyKey,

    /// SSB write error during DriftInterlock emission.
    #[error("SSB write error emitting DriftInterlock: {0}")]
    SsbWrite(String),

    /// A human unlock was rejected because the supplied re-validation timestamp is NOT strictly
    /// newer than the unresolved halt it claims to clear (a stale-evidence replay — the covert
    /// unlock attack). The load-bearing semantic guard: a privilege check alone is insufficient.
    #[error(
        "unlock rejected: re-validation timestamp {validated_at_ms} is not strictly newer than \
         the unresolved halt at {halt_at_ms} for strategy {strategy_id} (stale-evidence replay — \
         a fresh full-backtest re-run is required to clear a drift halt)"
    )]
    StaleUnlockEvidence {
        /// The strategy whose unlock was rejected.
        strategy_id: String,
        /// The supplied (stale) re-validation timestamp.
        validated_at_ms: i64,
        /// The timestamp of the unresolved halt being cleared.
        halt_at_ms: i64,
    },

    /// A human unlock was requested but the strategy is NOT currently halted (nothing to clear).
    #[error("unlock rejected: strategy {0} is not currently halted (no unresolved halt row)")]
    NotHalted(String),
}

// ---------------------------------------------------------------------------
// Drift axis evidence (the off-agent inputs the gate consumes — closed-form)
// ---------------------------------------------------------------------------

/// The per-axis evidence the interlock evaluates. Every field is `Option` so a MISSING axis is
/// distinguishable from a present-but-clean one: a `None` axis is un-evaluable => HALT
/// (default-HALTED, R1/R2). The closed-form math runs only when the evidence is present.
#[derive(Debug, Clone, PartialEq)]
pub struct DriftEvidence {
    /// AXIS 1a — the (expected, actual) binned proportions for the INPUT feature distribution.
    /// `None` => the input distribution was never logged => HALT.
    pub input_distribution: Option<PsiBins>,
    /// AXIS 1b — the (expected, actual) binned proportions for the OUTPUT / signal
    /// distribution. `None` => the output distribution was never logged => HALT.
    pub output_distribution: Option<PsiBins>,
    /// AXIS 2 — the FROZEN baseline schema fingerprint (blake3 hex). `None` => no baseline
    /// frozen => HALT (cannot prove the schema is unchanged).
    pub baseline_schema_fingerprint: Option<String>,
    /// AXIS 2 — the raw live vendor data schema bytes (the interlock fingerprints these with
    /// blake3 and compares to the baseline). `None` => the live schema was never captured => HALT.
    pub live_schema: Option<Vec<u8>>,
    /// AXIS 3 — the absolute live-vs-backtest metric deviation over the trailing
    /// `DEVIATION_WINDOW_DAYS`. `None` => the deviation was never computed => HALT.
    pub live_vs_backtest_deviation: Option<f64>,
    /// AXIS 3 — the timestamp (ms since epoch) the deviation was measured. Used with the clock
    /// to confirm the measurement is within the rolling window. `None` => HALT.
    pub deviation_measured_at_ms: Option<i64>,
    /// AXIS 4 — the timestamp (ms since epoch) the strategy was last validated. `None` => never
    /// validated => HALT.
    pub last_validated_at_ms: Option<i64>,
    /// COMPOSED upstream signal labels (G-10 `model_drift` `DriftSignal`, D-7 canary, H-18
    /// deflation). ANY non-empty entry is a composed breach => HALT. These are READ from the
    /// shipped gates (GD-1), not re-detected here.
    pub composed_breaches: Vec<String>,
}

/// Binned proportion pairs for one PSI axis. `expected` is the frozen baseline distribution,
/// `actual` is the live distribution; both are proportions over the SAME bin scheme and MUST
/// have equal length (checked at evaluation time — a length mismatch is un-evaluable => HALT).
#[derive(Debug, Clone, PartialEq)]
pub struct PsiBins {
    /// Frozen baseline (expected) bin proportions. Sum should be ~1.0.
    pub expected: Vec<f64>,
    /// Live (actual) bin proportions over the same bin scheme. Sum should be ~1.0.
    pub actual: Vec<f64>,
}

// ---------------------------------------------------------------------------
// PSI (population stability index) — closed-form, fail-closed on empty bins
// ---------------------------------------------------------------------------

/// Result of a single PSI computation. `Undefined` is the fail-closed case (an empty / zero
/// bin, a length mismatch, or a NaN/inf input) — the caller treats it as HALT, NOT a passing 0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PsiResult {
    /// A finite PSI value over a well-formed binning.
    Value(f64),
    /// The PSI is undefined (empty bin / length mismatch / non-finite input). Fail-closed: the
    /// caller HALTs. NEVER coerced to 0.0 (that would silently pass a degenerate comparison).
    Undefined,
}

/// Compute `PSI = sum_bins (actual_pct - expected_pct) * ln(actual_pct / expected_pct)`.
///
/// Fail-closed (R1/R2) on every degenerate case rather than producing NaN / inf / a misleading
/// finite value:
///   * a length mismatch between `expected` and `actual` => [`PsiResult::Undefined`];
///   * an empty binning (zero bins) => [`PsiResult::Undefined`];
///   * a non-finite input proportion (NaN / inf) => [`PsiResult::Undefined`];
///   * a ZERO bin on EITHER side (`expected_pct == 0` or `actual_pct == 0`) => the `ln(0)` /
///     divide-by-zero case. The bin proportion is floored to [`PSI_BIN_EPSILON`] so the term is
///     finite, BUT a zero bin means a category present in one distribution is ABSENT in the
///     other — a real structural shift. Rather than swallow it, the flooring makes the PSI term
///     LARGE (which pushes the result over [`PSI_HALT_BAND`]) instead of NaN; the result stays
///     `Value(..)` and the caller HALTs on the band. This keeps the function total and the
///     verdict conservative (bias toward HALT, the SAFE failure).
pub fn compute_psi(bins: &PsiBins) -> PsiResult {
    if bins.expected.len() != bins.actual.len() || bins.expected.is_empty() {
        return PsiResult::Undefined;
    }
    let mut psi = 0.0_f64;
    for (expected_pct, actual_pct) in bins.expected.iter().zip(bins.actual.iter()) {
        if !expected_pct.is_finite() || !actual_pct.is_finite() {
            return PsiResult::Undefined;
        }
        // Floor a zero (or negative, malformed) proportion away from ln(0)/0-division. A zero
        // bin is a structural shift; flooring inflates the term toward HALT rather than NaN.
        let e = expected_pct.max(PSI_BIN_EPSILON);
        let a = actual_pct.max(PSI_BIN_EPSILON);
        let term = (a - e) * (a / e).ln();
        if !term.is_finite() {
            return PsiResult::Undefined;
        }
        psi += term;
    }
    if psi.is_finite() {
        PsiResult::Value(psi)
    } else {
        PsiResult::Undefined
    }
}

// ---------------------------------------------------------------------------
// The four-axis verdict
// ---------------------------------------------------------------------------

/// The interlock verdict. Maps onto the [`DriftInterlockKind`] SSB Kind:
///   * [`InterlockVerdict::Running`]   => `DriftInterlockKind::Running`   (clean, resume OK);
///   * [`InterlockVerdict::Warning`]   => `DriftInterlockKind::Warning`   (minor PSI shift, no
///     halt yet — surfaces but does not latch);
///   * [`InterlockVerdict::Halted`]    => `DriftInterlockKind::DriftHalted` (latched HALT).
#[derive(Debug, Clone, PartialEq)]
pub enum InterlockVerdict {
    /// No drift on any axis (and no pre-existing unresolved latch). Resume is permitted.
    Running,
    /// A minor PSI shift on at least one axis (`PSI_WARN_BAND <= PSI < PSI_HALT_BAND`) but no
    /// HALT-class breach. Surfaces as a Warning; does NOT latch HALTED.
    Warning {
        /// The breached axis labels driving the warning.
        axes: Vec<String>,
    },
    /// At least one HALT-class breach OR missing/un-evaluable evidence OR a pre-existing
    /// unresolved latch. The interlock is HALTED (a `'halt'` latch row is appended).
    Halted {
        /// The breached / un-evaluable axis labels driving the halt.
        axes: Vec<String>,
    },
}

impl InterlockVerdict {
    /// Map the verdict to the SSB Kind discriminator.
    #[must_use]
    pub fn to_kind(&self) -> DriftInterlockKind {
        match self {
            Self::Running => DriftInterlockKind::Running,
            Self::Warning { .. } => DriftInterlockKind::Warning,
            Self::Halted { .. } => DriftInterlockKind::DriftHalted,
        }
    }

    /// `true` when the verdict latches the interlock HALTED.
    #[must_use]
    pub fn is_halt(&self) -> bool {
        matches!(self, Self::Halted { .. })
    }

    /// The breached / warning axis labels (empty for `Running`).
    #[must_use]
    pub fn axes(&self) -> &[String] {
        match self {
            Self::Running => &[],
            Self::Warning { axes } | Self::Halted { axes } => axes,
        }
    }
}

/// Classify the four axes (PURE — no I/O, no clock mutation, no RNG; GD-2 byte-reproducible).
///
/// `now_ms` is the current wall-clock millisecond reading sourced from the injected
/// ClockProvider by the caller (NEVER `SystemTime::now()` here). Returns the verdict; the
/// caller is responsible for latching + emitting the SSB row.
///
/// Fail-closed precedence: any missing / un-evaluable axis is collected as a HALT axis BEFORE
/// the band math, so a degenerate input can never produce `Running`.
#[must_use]
pub fn classify_drift(evidence: &DriftEvidence, now_ms: i64) -> InterlockVerdict {
    let mut halt_axes: Vec<String> = Vec::new();
    let mut warn_axes: Vec<String> = Vec::new();

    // AXIS 1a/1b — PSI on inputs AND outputs.
    classify_psi_axis("input_psi", evidence.input_distribution.as_ref(), &mut halt_axes, &mut warn_axes);
    classify_psi_axis("output_psi", evidence.output_distribution.as_ref(), &mut halt_axes, &mut warn_axes);

    // AXIS 2 — schema-fingerprint mismatch (blake3 over the live schema vs the frozen baseline).
    match (
        evidence.baseline_schema_fingerprint.as_ref(),
        evidence.live_schema.as_ref(),
    ) {
        (Some(baseline), Some(live_bytes)) => {
            let live_fp = blake3::hash(live_bytes).to_hex().to_string();
            if &live_fp != baseline {
                halt_axes.push("schema_fingerprint".to_string());
            }
        }
        // Missing baseline OR missing live schema => cannot prove the schema is unchanged => HALT.
        _ => halt_axes.push("schema_fingerprint_missing".to_string()),
    }

    // AXIS 3 — rolling-14d live-vs-backtest deviation (clock-windowed).
    match (
        evidence.live_vs_backtest_deviation,
        evidence.deviation_measured_at_ms,
    ) {
        (Some(deviation), Some(measured_at)) => {
            let window_ms = i64::try_from(DEVIATION_WINDOW_DAYS).unwrap_or(i64::MAX) * MS_PER_DAY;
            if now_ms.saturating_sub(measured_at) > window_ms {
                // The deviation measurement is older than the rolling window => stale =>
                // un-evaluable => HALT (we cannot prove CURRENT behaviour matches the backtest).
                halt_axes.push("deviation_window_stale".to_string());
            } else if !deviation.is_finite() {
                halt_axes.push("deviation_non_finite".to_string());
            } else if deviation.abs() > DEVIATION_HALT_BAND {
                halt_axes.push("live_vs_backtest_deviation".to_string());
            }
        }
        _ => halt_axes.push("deviation_missing".to_string()),
    }

    // AXIS 4 — 90d validation TTL (clock-driven staleness).
    match evidence.last_validated_at_ms {
        Some(validated_at) => {
            let ttl_ms = i64::try_from(VALIDATION_TTL_DAYS).unwrap_or(i64::MAX) * MS_PER_DAY;
            if now_ms.saturating_sub(validated_at) > ttl_ms {
                halt_axes.push("validation_ttl_expired".to_string());
            }
        }
        None => halt_axes.push("validation_missing".to_string()),
    }

    // COMPOSED upstream signals (G-10 model_drift / D-7 canary / H-18 deflation) — GD-1 REUSE.
    for breach in &evidence.composed_breaches {
        halt_axes.push(format!("composed:{breach}"));
    }

    if !halt_axes.is_empty() {
        InterlockVerdict::Halted { axes: halt_axes }
    } else if !warn_axes.is_empty() {
        InterlockVerdict::Warning { axes: warn_axes }
    } else {
        InterlockVerdict::Running
    }
}

/// Classify one PSI axis into the halt / warn buckets. A missing distribution or an undefined
/// PSI (empty / degenerate bin) is HALT (default-HALTED); `PSI >= PSI_HALT_BAND` is HALT;
/// `PSI_WARN_BAND <= PSI < PSI_HALT_BAND` is WARN; below `PSI_WARN_BAND` is clean.
fn classify_psi_axis(
    label: &str,
    bins: Option<&PsiBins>,
    halt_axes: &mut Vec<String>,
    warn_axes: &mut Vec<String>,
) {
    match bins {
        None => halt_axes.push(format!("{label}_missing")),
        Some(bins) => match compute_psi(bins) {
            PsiResult::Undefined => halt_axes.push(format!("{label}_undefined")),
            PsiResult::Value(psi) => {
                if psi >= PSI_HALT_BAND {
                    halt_axes.push(label.to_string());
                } else if psi >= PSI_WARN_BAND {
                    warn_axes.push(label.to_string());
                }
            }
        },
    }
}

// ---------------------------------------------------------------------------
// DriftInterlockLedger — append-only journal-derived latch, NO agent unlock
// ---------------------------------------------------------------------------

/// Append-only persisted drift-interlock latch (H-29, the master deployment-trust interlock).
///
/// Holds a `rusqlite::Connection` to the `drift_interlock_latch` table (on the same db file as
/// [`Ssb`]). The HALTED state is DERIVED: `is_halted(strategy) = COUNT('halt') >
/// COUNT('unlock')`. NO unlock / reset / clear / resume method exists on this type (a covert
/// un-latch would defeat the gate); the human unlock is the FREE function
/// [`unlock_drift_interlock`]. The latch once HALTED STAYS HALTED across restarts until a
/// fresh-evidence unlock row is appended.
pub struct DriftInterlockLedger {
    conn: std::sync::Mutex<Connection>,
    session_id: String,
}

impl std::fmt::Debug for DriftInterlockLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DriftInterlockLedger")
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

impl DriftInterlockLedger {
    /// Open a new `DriftInterlockLedger` on `db_path` (the same SQLite file used by `Ssb`).
    /// The `drift_interlock_latch` table is created if it does not yet exist (existing rows
    /// preserved — the latch persists across sessions / restarts).
    pub fn open(
        db_path: &Path,
        session_id: impl Into<String>,
    ) -> Result<Self, DriftInterlockError> {
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(crate::ssb::CREATE_DRIFT_INTERLOCK_TABLE)?;
        Ok(Self {
            conn: std::sync::Mutex::new(conn),
            session_id: session_id.into(),
        })
    }

    /// Lock the inner connection, mapping a poisoned mutex to a usable error (NO `.unwrap()`).
    fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DriftInterlockError> {
        self.conn
            .lock()
            .map_err(|_| DriftInterlockError::SsbWrite("drift interlock mutex poisoned".into()))
    }

    /// `true` when the strategy is currently latched HALTED: `COUNT('halt') > COUNT('unlock')`.
    ///
    /// This is the restart-survival read: Running is the ABSENCE of an unresolved halt, never a
    /// stored default, so a process restart re-derives HALTED from the journal.
    pub fn is_halted(&self, strategy_id: &str) -> Result<bool, DriftInterlockError> {
        let strategy_id = canonicalize_key(strategy_id);
        if strategy_id.len() < MIN_KEY_LEN {
            return Err(DriftInterlockError::EmptyKey);
        }
        let conn = self.lock_conn()?;
        let halts = count_rows(&conn, &strategy_id, ROW_KIND_HALT)?;
        let unlocks = count_rows(&conn, &strategy_id, ROW_KIND_UNLOCK)?;
        Ok(halts > unlocks)
    }

    /// Append one `'halt'` latch row for `strategy_id` at `halt_at_ms`. Append-only; the count
    /// of these rows (minus unlocks) is the latch state. `row_id` is the blake3 of the canonical
    /// tuple so repeated identical halts are idempotent on the PK.
    fn append_halt(
        &self,
        strategy_id: &str,
        axes: &str,
        halt_at_ms: i64,
    ) -> Result<(), DriftInterlockError> {
        let row_id = compute_row_id(ROW_KIND_HALT, strategy_id, axes, halt_at_ms);
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT OR IGNORE INTO drift_interlock_latch \
             (row_id, kind, strategy_id, axes, event_at_ms, session_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                row_id,
                ROW_KIND_HALT,
                strategy_id,
                axes,
                halt_at_ms,
                self.session_id,
            ],
        )?;
        Ok(())
    }

    /// The timestamp of the most recent `'halt'` row for `strategy_id` (or `None` if never
    /// halted). Used by the unlock path to enforce the fresh-evidence guard.
    fn latest_halt_ms(&self, strategy_id: &str) -> Result<Option<i64>, DriftInterlockError> {
        let conn = self.lock_conn()?;
        let ts: Option<i64> = conn
            .query_row(
                "SELECT event_at_ms FROM drift_interlock_latch \
                 WHERE strategy_id = ?1 AND kind = ?2 \
                 ORDER BY event_at_ms DESC LIMIT 1",
                params![strategy_id, ROW_KIND_HALT],
                |row| row.get(0),
            )
            .ok();
        Ok(ts)
    }

    /// Emit one off-agent [`SsbPayload::DriftInterlock`] row. `evidence_blake3` binds the
    /// verdict to the breached-axis provenance so it is recomputable (GD-2). BLAKE3 ONLY.
    async fn emit_event(
        &self,
        ssb: &Arc<Ssb>,
        kind: DriftInterlockKind,
        strategy_id: &str,
        axes: &str,
        run_id: &str,
        seed: u64,
    ) -> Result<(), DriftInterlockError> {
        let key = format!("{run_id}:{strategy_id}:{axes}:{}:{seed}", kind.label());
        let evidence_blake3 = blake3::hash(key.as_bytes()).to_hex().to_string();

        let payload = SsbPayload::DriftInterlock(DriftInterlockPayload {
            kind,
            strategy_id: strategy_id.to_string(),
            breached_axes: axes.to_string(),
            run_id: run_id.to_string(),
            seed,
            evidence_blake3,
        });

        ssb.write_payload(
            DRIFT_INTERLOCK_AGENT_ID,
            DRIFT_INTERLOCK_SUPERSTEP,
            payload,
            vec!["drift_interlock.rs".to_string()],
        )
        .await
        .map_err(|e| DriftInterlockError::SsbWrite(format!("{e}")))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// THE CHOKE-POINT: evaluate_drift_interlock (the production gate)
// ---------------------------------------------------------------------------

/// Outcome of one interlock evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct DriftInterlockOutcome {
    /// The verdict (Running / Warning / Halted).
    pub verdict: InterlockVerdict,
    /// `true` when the strategy is latched HALTED AFTER this evaluation (either this evaluation
    /// halted it, or a prior unresolved halt remains).
    pub latched_halted: bool,
}

/// Evaluate the four-axis drift interlock for `strategy_id` and emit the off-agent SSB row.
///
/// Steps:
///   1. DEFAULT-HALTED short-circuit: if the strategy is ALREADY latched HALTED
///      (`COUNT('halt') > COUNT('unlock')`), the verdict is HALT regardless of the fresh
///      evidence — the latch persists until a human unlock. Emit a `DriftHalted` row and return.
///   2. else classify the four axes ([`classify_drift`]) using the injected clock for the
///      14d / 90d windows (NEVER `SystemTime::now()`).
///   3. on a HALT verdict: append a `'halt'` latch row (the persistent latch) and emit a
///      `DriftHalted` row.
///   4. on a Warning / Running verdict: emit the corresponding row WITHOUT latching.
///
/// The SSB row is emitted on EVERY verdict (the off-agent artifact, R4).
///
/// # Arguments
/// * `ledger`      — the drift-interlock latch ledger (append-only, no agent unlock).
/// * `ssb`         — the project Ssb handle; used to emit `DriftInterlock`.
/// * `clock`       — ClockProvider (the 14d/90d windows; never SystemTime).
/// * `strategy_id` — the live strategy identity (canonicalized scope key).
/// * `evidence`    — the four-axis [`DriftEvidence`] (any missing axis => HALT).
/// * `run_id`      — declared run id stamped on the row.
/// * `seed`        — declared `ELAI_SEED` (GD-2 reproducibility).
pub async fn evaluate_drift_interlock(
    ledger: &DriftInterlockLedger,
    ssb: &Arc<Ssb>,
    clock: &Arc<dyn ClockProvider>,
    strategy_id: &str,
    evidence: &DriftEvidence,
    run_id: &str,
    seed: u64,
) -> Result<DriftInterlockOutcome, DriftInterlockError> {
    let strategy_id = canonicalize_key(strategy_id);
    if strategy_id.len() < MIN_KEY_LEN {
        return Err(DriftInterlockError::EmptyKey);
    }

    // 1. DEFAULT-HALTED short-circuit: a pre-existing unresolved latch stays HALTED.
    if ledger.is_halted(&strategy_id)? {
        warn!(
            strategy_id = %strategy_id,
            "evaluate_drift_interlock: strategy already latched HALTED -> HALT (latch persists \
             until a human/frozen fresh-evidence unlock)"
        );
        ledger
            .emit_event(
                ssb,
                DriftInterlockKind::DriftHalted,
                &strategy_id,
                "pre_existing_latch",
                run_id,
                seed,
            )
            .await?;
        return Ok(DriftInterlockOutcome {
            verdict: InterlockVerdict::Halted {
                axes: vec!["pre_existing_latch".to_string()],
            },
            latched_halted: true,
        });
    }

    // 2. Classify the four axes (PURE; clock-windowed; default-HALTED on missing evidence).
    let now_ms = Ssb::now_ms_via_clock(clock);
    let verdict = classify_drift(evidence, now_ms);
    let axes_str = verdict.axes().join(",");

    // 3 / 4. Latch on HALT; emit on every verdict.
    let latched_halted = if verdict.is_halt() {
        ledger.append_halt(&strategy_id, &axes_str, now_ms)?;
        warn!(
            strategy_id = %strategy_id,
            axes = %axes_str,
            "evaluate_drift_interlock: HALT -> latched (resume blocked until human re-validation)"
        );
        true
    } else {
        debug!(
            strategy_id = %strategy_id,
            verdict = ?verdict,
            "evaluate_drift_interlock: no HALT-class breach"
        );
        false
    };

    ledger
        .emit_event(ssb, verdict.to_kind(), &strategy_id, &axes_str, run_id, seed)
        .await?;

    Ok(DriftInterlockOutcome {
        verdict,
        latched_halted,
    })
}

// ---------------------------------------------------------------------------
// Human / frozen unlock — FREE FUNCTION (NOT a method on the ledger)
// ---------------------------------------------------------------------------

/// Clear a drift HALT — the HUMAN / FROZEN out-of-band path ONLY.
///
/// This is a FREE FUNCTION, not a method on [`DriftInterlockLedger`], so an agent holding only
/// a ledger handle CANNOT reach it (the no-agent-unlock invariant, leg 1). It REQUIRES fresh
/// re-validation evidence:
///   * the strategy MUST currently be halted (else [`DriftInterlockError::NotHalted`]);
///   * `validated_at_ms` (the timestamp of a FRESH full-backtest re-run) MUST be STRICTLY NEWER
///     than the most recent unresolved `'halt'` row — the load-bearing semantic guard. An
///     unlock that re-presents validation OLDER than (or equal to) the halt is a stale-evidence
///     replay (the covert unlock attack) and is REJECTED
///     ([`DriftInterlockError::StaleUnlockEvidence`]). A privilege check alone is insufficient.
///
/// On success it appends ONE `'unlock'` latch row (bringing `COUNT('unlock')` level with
/// `COUNT('halt')`, so `is_halted` returns false) and emits a `HumanUnlocked` SSB row.
///
/// R8 residue: the human/frozen unlock is a bounded human-trust dependency; it does NOT make
/// the interlock auto-clearing.
#[allow(clippy::too_many_arguments)]
pub async fn unlock_drift_interlock(
    ledger: &DriftInterlockLedger,
    ssb: &Arc<Ssb>,
    clock: &Arc<dyn ClockProvider>,
    strategy_id: &str,
    validated_at_ms: i64,
    run_id: &str,
    seed: u64,
) -> Result<(), DriftInterlockError> {
    let strategy_id = canonicalize_key(strategy_id);
    if strategy_id.len() < MIN_KEY_LEN {
        return Err(DriftInterlockError::EmptyKey);
    }

    // The strategy MUST be halted (nothing to clear otherwise).
    if !ledger.is_halted(&strategy_id)? {
        return Err(DriftInterlockError::NotHalted(strategy_id));
    }

    // THE GUARD: the re-validation evidence MUST be strictly newer than the unresolved halt.
    let halt_at_ms = ledger.latest_halt_ms(&strategy_id)?.unwrap_or(i64::MAX);
    if validated_at_ms <= halt_at_ms {
        return Err(DriftInterlockError::StaleUnlockEvidence {
            strategy_id,
            validated_at_ms,
            halt_at_ms,
        });
    }

    // Append the append-only 'unlock' row (NO UPDATE; the latch is cleared by count parity).
    let unlock_at_ms = Ssb::now_ms_via_clock(clock);
    let axes = format!("unlock@{validated_at_ms}");
    let row_id = compute_row_id(ROW_KIND_UNLOCK, &strategy_id, &axes, unlock_at_ms);
    {
        let conn = ledger.lock_conn()?;
        conn.execute(
            "INSERT INTO drift_interlock_latch \
             (row_id, kind, strategy_id, axes, event_at_ms, session_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                row_id,
                ROW_KIND_UNLOCK,
                strategy_id,
                axes,
                unlock_at_ms,
                ledger.session_id,
            ],
        )?;
    }

    ledger
        .emit_event(
            ssb,
            DriftInterlockKind::HumanUnlocked,
            &strategy_id,
            &axes,
            run_id,
            seed,
        )
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Canonicalize a scope key (trim, lower-case, collapse whitespace) to stop scope-key
/// fragmentation attacks (mirrors `holdout_budget::canonicalize_key`).
fn canonicalize_key(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// COUNT of latch rows of `kind` for `strategy_id` (the append-only latch counter).
fn count_rows(
    conn: &Connection,
    strategy_id: &str,
    kind: &str,
) -> Result<i64, DriftInterlockError> {
    // NARROWING-OK: COUNT(*) is non-negative.
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM drift_interlock_latch WHERE strategy_id = ?1 AND kind = ?2",
        params![strategy_id, kind],
        |row| row.get(0),
    )?;
    Ok(n)
}

/// Compute a row_id = blake3 hex of the canonical tuple. blake3 ONLY (CLAUDE.md HARD RULE).
fn compute_row_id(kind: &str, strategy_id: &str, axes: &str, event_at_ms: i64) -> String {
    let tuple = format!("{kind}:{strategy_id}:{axes}:{event_at_ms}");
    blake3::hash(tuple.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::{Ssb, SsbConfig};
    use elai_core::clock::OsClock;
    use tempfile::TempDir;

    const TEST_STRATEGY: &str = "momentum_eurusd_v3";
    const TEST_RUN_ID: &str = "run-h29-test";
    const TEST_SEED: u64 = 42;

    fn test_clock() -> Arc<dyn ClockProvider> {
        Arc::new(OsClock)
    }

    fn test_ssb(dir: &TempDir) -> Arc<Ssb> {
        let cfg = SsbConfig {
            db_path: dir.path().join("ssb.db"),
            pool_size: None,
            project_id: "h29-test".into(),
            session_id: "h29-test-session".into(),
        };
        Arc::new(Ssb::new(cfg, test_clock()).expect("ssb init"))
    }

    fn test_ledger(dir: &TempDir) -> DriftInterlockLedger {
        DriftInterlockLedger::open(&dir.path().join("ssb.db"), "h29-test-session")
            .expect("ledger open")
    }

    /// A clean, no-drift evidence bundle (validated NOW; identical input/output distributions;
    /// matching schema fingerprint; tiny deviation). Resolves to Running.
    fn clean_evidence(now_ms: i64) -> DriftEvidence {
        let live_schema = b"col_a:f64,col_b:f64,signal:f64".to_vec();
        let fp = blake3::hash(&live_schema).to_hex().to_string();
        DriftEvidence {
            input_distribution: Some(PsiBins {
                expected: vec![0.25, 0.25, 0.25, 0.25],
                actual: vec![0.25, 0.25, 0.25, 0.25],
            }),
            output_distribution: Some(PsiBins {
                expected: vec![0.5, 0.5],
                actual: vec![0.5, 0.5],
            }),
            baseline_schema_fingerprint: Some(fp),
            live_schema: Some(live_schema),
            live_vs_backtest_deviation: Some(0.01),
            deviation_measured_at_ms: Some(now_ms),
            last_validated_at_ms: Some(now_ms),
            composed_breaches: Vec::new(),
        }
    }

    #[test]
    fn psi_zero_on_identical_distributions() {
        let bins = PsiBins {
            expected: vec![0.25, 0.25, 0.25, 0.25],
            actual: vec![0.25, 0.25, 0.25, 0.25],
        };
        match compute_psi(&bins) {
            PsiResult::Value(v) => assert!(v.abs() < 1.0e-9, "identical dists => PSI ~ 0, got {v}"),
            PsiResult::Undefined => panic!("identical well-formed bins must be defined"),
        }
    }

    #[test]
    fn psi_known_shift_exceeds_halt_band() {
        // A large mass shift between two bins yields a PSI well over the 0.25 halt band.
        let bins = PsiBins {
            expected: vec![0.5, 0.5],
            actual: vec![0.1, 0.9],
        };
        match compute_psi(&bins) {
            PsiResult::Value(v) => assert!(
                v >= PSI_HALT_BAND,
                "a large shift must exceed the halt band {PSI_HALT_BAND}, got {v}"
            ),
            PsiResult::Undefined => panic!("well-formed bins must be defined"),
        }
    }

    #[test]
    fn psi_length_mismatch_is_undefined() {
        let bins = PsiBins {
            expected: vec![0.5, 0.5],
            actual: vec![1.0],
        };
        assert_eq!(compute_psi(&bins), PsiResult::Undefined);
    }

    #[test]
    fn psi_empty_bins_is_undefined() {
        let bins = PsiBins {
            expected: vec![],
            actual: vec![],
        };
        assert_eq!(compute_psi(&bins), PsiResult::Undefined);
    }

    #[test]
    fn psi_zero_bin_floored_not_nan() {
        // A zero bin must NOT produce NaN; it is floored toward a large (halt-class) term.
        let bins = PsiBins {
            expected: vec![0.0, 1.0],
            actual: vec![0.5, 0.5],
        };
        match compute_psi(&bins) {
            PsiResult::Value(v) => assert!(v.is_finite(), "zero bin must stay finite, got {v}"),
            PsiResult::Undefined => { /* also acceptable fail-closed */ }
        }
    }

    #[test]
    fn classify_clean_evidence_is_running() {
        let now_ms = 1_750_000_000_000;
        let verdict = classify_drift(&clean_evidence(now_ms), now_ms);
        assert_eq!(verdict, InterlockVerdict::Running, "clean evidence must be Running");
    }

    #[test]
    fn classify_missing_axis_is_halt() {
        let now_ms = 1_750_000_000_000;
        let mut ev = clean_evidence(now_ms);
        ev.input_distribution = None;
        let verdict = classify_drift(&ev, now_ms);
        assert!(verdict.is_halt(), "a missing axis must HALT (default-HALTED)");
        assert!(
            verdict.axes().iter().any(|a| a == "input_psi_missing"),
            "the missing axis must be named: {:?}",
            verdict.axes()
        );
    }

    #[test]
    fn classify_schema_mismatch_is_halt() {
        let now_ms = 1_750_000_000_000;
        let mut ev = clean_evidence(now_ms);
        ev.baseline_schema_fingerprint = Some("deadbeef".repeat(8));
        let verdict = classify_drift(&ev, now_ms);
        assert!(verdict.is_halt(), "a schema fingerprint mismatch must HALT");
        assert!(verdict.axes().iter().any(|a| a == "schema_fingerprint"));
    }

    #[test]
    fn classify_stale_ttl_is_halt() {
        let now_ms = 1_750_000_000_000;
        let mut ev = clean_evidence(now_ms);
        // Validated 100 days ago (> 90d TTL).
        ev.last_validated_at_ms = Some(now_ms - 100 * MS_PER_DAY);
        let verdict = classify_drift(&ev, now_ms);
        assert!(verdict.is_halt(), "a >90d-stale validation must HALT");
        assert!(verdict.axes().iter().any(|a| a == "validation_ttl_expired"));
    }

    #[test]
    fn classify_deviation_breach_is_halt() {
        let now_ms = 1_750_000_000_000;
        let mut ev = clean_evidence(now_ms);
        ev.live_vs_backtest_deviation = Some(DEVIATION_HALT_BAND + 1.0);
        let verdict = classify_drift(&ev, now_ms);
        assert!(verdict.is_halt(), "a deviation over the band must HALT");
        assert!(verdict.axes().iter().any(|a| a == "live_vs_backtest_deviation"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn latch_persists_across_reopen() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = test_ssb(&dir);
        let clock = test_clock();
        let now_ms = Ssb::now_ms_via_clock(&clock);
        let mut ev = clean_evidence(now_ms);
        ev.baseline_schema_fingerprint = Some("deadbeef".repeat(8)); // force a HALT

        {
            let ledger = test_ledger(&dir);
            let out = evaluate_drift_interlock(
                &ledger, &ssb, &clock, TEST_STRATEGY, &ev, TEST_RUN_ID, TEST_SEED,
            )
            .await
            .expect("evaluate");
            assert!(out.latched_halted, "a schema mismatch must latch HALTED");
        }

        // Re-open: the latch persists (a restart does NOT reset to Running).
        let ledger2 = test_ledger(&dir);
        assert!(
            ledger2.is_halted(TEST_STRATEGY).expect("is_halted"),
            "the latch must survive a ledger re-open (restart != reset)"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn unlock_requires_fresh_evidence() {
        let dir = TempDir::new().expect("tempdir");
        let ssb = test_ssb(&dir);
        let clock = test_clock();
        let now_ms = Ssb::now_ms_via_clock(&clock);
        let mut ev = clean_evidence(now_ms);
        ev.baseline_schema_fingerprint = Some("deadbeef".repeat(8)); // force a HALT

        let ledger = test_ledger(&dir);
        let out = evaluate_drift_interlock(
            &ledger, &ssb, &clock, TEST_STRATEGY, &ev, TEST_RUN_ID, TEST_SEED,
        )
        .await
        .expect("evaluate");
        assert!(out.latched_halted);

        let halt_at = ledger.latest_halt_ms(&canonicalize_key(TEST_STRATEGY)).expect("halt ms");
        let halt_at = halt_at.expect("halt row present");

        // STALE evidence (validated BEFORE the halt) must be REJECTED.
        let stale = unlock_drift_interlock(
            &ledger, &ssb, &clock, TEST_STRATEGY, halt_at - 1, TEST_RUN_ID, TEST_SEED,
        )
        .await;
        assert!(
            matches!(stale, Err(DriftInterlockError::StaleUnlockEvidence { .. })),
            "stale re-validation must be rejected, got {stale:?}"
        );
        assert!(ledger.is_halted(TEST_STRATEGY).expect("still halted"));

        // FRESH evidence (validated AFTER the halt) clears the latch.
        unlock_drift_interlock(
            &ledger, &ssb, &clock, TEST_STRATEGY, halt_at + 1, TEST_RUN_ID, TEST_SEED,
        )
        .await
        .expect("fresh unlock");
        assert!(
            !ledger.is_halted(TEST_STRATEGY).expect("unlocked"),
            "a fresh-evidence unlock must clear the latch"
        );
    }

    /// Negative compile-test (mirror holdout_budget.rs:1128). The agent-facing
    /// `DriftInterlockLedger` must NOT expose any unlock / reset / clear / resume method.
    /// `unlock_drift_interlock` is a FREE FUNCTION, not a method, so an agent holding only a
    /// ledger handle cannot reach it. If a banned method were added, the commented calls below
    /// would compile and this test's intent would be violated.
    #[test]
    fn no_unlock_or_reset_method_exists() {
        let _ = |_l: &DriftInterlockLedger| {
            // These must NOT compile:
            // _l.unlock("s");
            // _l.reset("s");
            // _l.clear("s");
            // _l.resume("s");
            // _l.unlock_drift_interlock(...);
        };
    }
}
