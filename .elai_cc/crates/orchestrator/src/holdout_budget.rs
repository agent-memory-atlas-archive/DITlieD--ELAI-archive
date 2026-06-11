//! H-19 (SCHEMA_VERSION 237): reusable-holdout query budget — the Thresholdout DP gate.
//!
//! The deployment-trust half of the selection-bias keystone (Track-3 TB-1; depends on the
//! H-17 trial ledger). Adaptive reuse of a held-out evaluation set silently destroys its
//! statistical validity: an analyst (or a self-improving optimizer) that repeatedly queries
//! the holdout and keeps the configs that score well is overfitting the holdout, not the
//! population. The Thresholdout mechanism (Dwork, Feldman, Hardt, Pitassi, Reingold, Roth,
//! "Generalization in Adaptive Data Analysis and Holdout Reuse", NIPS 2015, arXiv:1506.02629,
//! Figure 1 + Theorem 25) makes the reuse budget EXPLICIT and finite via differential
//! privacy:
//!
//!   * a per-query NOISY comparison: answer from the TRAINING set FOR FREE when the empirical
//!     train/holdout gap is below a noisy threshold (the holdout is not consulted, no budget
//!     spent);
//!   * only when the gap exceeds the noisy threshold does the gate CONSULT the holdout,
//!     charge ONE budget unit, and release a Laplace-noised holdout statistic;
//!   * once the budget `B` is spent the holdout is EXHAUSTED — every further query is REFUSED
//!     (the Thresholdout BOTTOM symbol). The holdout CANNOT be refreshed in-place; only a
//!     FRESH, independent sample restores validity (paper Section 4, the honest exhaustion
//!     limit). Re-labeling or re-splitting the SAME underlying data does NOT restore it.
//!
//! # The EXACT LOCKED Thresholdout parameters (Theorem 25, 1506.02629 — NOT tunable)
//!
//!   sigma   = tau / (96 * ln(4 * m / beta))   [`SIGMA_DENOMINATOR_CONSTANT` = 96 is the
//!                                              union-bound PROOF constant — changing it
//!                                              silently breaks the DP guarantee]
//!   T       = (3/4) * tau                      [`T_FRACTION_OF_TAU`]
//!   gamma   ~ Lap(2 * sigma)  (threshold noise, T_hat = T + gamma, re-drawn on each spend)
//!   eta     ~ Lap(4 * sigma)  (per-query comparison noise, drawn fresh every query)
//!   xi      ~ Lap(1 * sigma)  (answer noise, added to the holdout mean on a spend)
//!
//! The three noise multipliers (2, 4, 1) are `GAMMA_SCALE_MULTIPLIER`,
//! `ETA_SCALE_MULTIPLIER`, `XI_SCALE_MULTIPLIER`. Holdout-size floor (Lemma 24):
//! `n >= max(2*B/(sigma*tau), ln(6/beta)/tau^2)` with a conservative constant `C = 2` (NOT
//! pinned in the paper — documented here, see `HOLDOUT_SIZE_FLOOR_CONSTANT`).
//!
//! # The query algorithm (paper Figure 1, EXACT)
//!
//! Initialization (per query, derived deterministically from the seed + the spend index so
//! it is byte-identically recomputable, GD-2): T_hat = T + Lap(2*sigma).
//! Per query phi:
//!   (a) if budget < 1: REFUSE (BOTTOM / `Exhausted`).
//!   (b) else: draw eta ~ Lap(4*sigma);
//!       if |E_Sh[phi] - E_St[phi]| > T_hat + eta:   [ABOVE THRESHOLD — budget charged]
//!           draw xi ~ Lap(sigma); B <- B - 1; return E_Sh[phi] + xi;
//!       else:                                       [BELOW THRESHOLD — free]
//!           return E_St[phi].
//!
//! # Laplace via inverse-CDF (NO new dependency, Laplace ONLY)
//!
//! `Lap(scale)` is sampled by inverse transform: for `u ~ Uniform(-1/2, 1/2)`,
//! `x = -scale * sign(u) * ln(1 - 2*|u|)`. The uniform `u` is drawn from
//! [`elai_core::entropy::SeededRng`] (ChaCha8, already a core dep — NO `rand_distr`, NO new
//! dependency). Laplace ONLY: the formal DP guarantee in Theorem 25 is for Laplace noise;
//! Gaussian gives a DIFFERENT (epsilon, delta)-DP bound and is BANNED here (paper Section 5
//! / pitfall 6).
//!
//! # Append-only budget counter (mirrors the H-17 [`crate::trial_ledger::TrialLedger`])
//!
//! [`HoldoutBudgetLedger`] owns a SQLite connection to the `holdout_budget` table (on the
//! same db file as the Ssb WAL). Append-only by construction: NO reset / delete / update /
//! provision method exists on the agent-facing type (a covert budget reset would defeat the
//! whole gate). `queries_spent` for a holdout = COUNT of `'spent'` rows; `budget_remaining =
//! budget_total - queries_spent`. The budget once spent STAYS spent.
//!
//! # The no-agent-reset invariant — three legs (mirror H-17)
//!   1. NO reset/delete/update/provision method on [`HoldoutBudgetLedger`] (negative
//!      compile-test `no_reset_or_provision_method_exists`).
//!   2. the debug `update_hook` in `Ssb::new()` panics on any UPDATE/DELETE against the
//!      `holdout_budget` table (ssb.rs).
//!   3. `holdout_budget.rs` + `holdout_budget.toml` are in `.claude/gate-manifest.txt`,
//!      re-locked by a HUMAN out-of-band (`gate_freeze_check.py --lock` — the executing run
//!      cannot re-lock its own gate, R8).
//!
//! # Provision-fresh — human / frozen path only ([`provision_fresh_holdout`])
//!
//! A FRESH holdout is provisioned by a privileged out-of-band function, NOT a method on the
//! agent-reachable ledger. It asserts the new `data_window` is DISJOINT from every spent
//! window for the holdout — the LOAD-BEARING semantic guard against the covert reset attack:
//! re-labeling / re-splitting the SAME series is a disguised reset and a privilege check
//! alone does NOT catch it (paper pitfall 5). R8 residue: the human/frozen provision path is
//! a residual human-trust dependency that BOUNDS adaptive reuse — it does NOT make OOS
//! infinite. Documented, not eliminated.
//!
//! # Fail-closed (R1/R2)
//!   * budget exhausted => REFUSE (`Exhausted`), never SKIP.
//!   * a missing budget counter (no provision row) => [`BudgetRemainingResult::BudgetMissing`]
//!     sentinel => the caller fail-closes (REFUSE). "Not found means it never ran" (R2).
//!   * an overlapping provision window => the provision is REJECTED (the reset attack).
//!
//! # No `.unwrap()` / `.expect()` in non-test code (CLAUDE.md HARD RULE). blake3 only.
//!
//! # References
//! - `workflow/papers/reusable-holdout-thresholdout-interface.md` (the LOCKED formulas)
//! - `docs/thefinalglance/H-plan/H-19_reusable_holdout_budget.md`
//! - `workflow/reasoning/H-19-decisions.md`

use std::path::Path;
use std::sync::Arc;

use rusqlite::{params, Connection};
use thiserror::Error;
use tracing::{debug, warn};

use crate::ssb::{HoldoutBudgetKind, HoldoutBudgetPayload, Ssb, SsbPayload};
use elai_core::clock::ClockProvider;
use elai_core::entropy::{EntropySource, SeededRng};

// ---------------------------------------------------------------------------
// LOCKED Thresholdout constants (Theorem 25, 1506.02629 — NOT tunable).
// All frozen in .claude/gate-manifest.txt (TD-2). Named constants, no magic
// numbers (CLAUDE.md HARD RULE).
// ---------------------------------------------------------------------------

/// Target end-to-end accuracy `tau` (Theorem 25). Range (0, 0.5); scales inversely with
/// sqrt(n). 0.05 is a conservative practical default (the paper's experiments use T = 0.04).
const HOLDOUT_TAU: f64 = 0.05;

/// Failure probability `beta` (Theorem 25). The guarantee holds with probability >= 1-beta.
const HOLDOUT_BETA: f64 = 0.05;

/// The maximum query horizon `m` (total adaptive queries, above- AND below-threshold) the
/// `sigma` calibration is sized for. Conservative: a large horizon makes `sigma` SMALLER
/// (tighter noise), which is the safe direction for the DP budget.
const HOLDOUT_M_QUERIES: f64 = 10_000.0;

/// The union-bound PROOF constant in `sigma = tau / (96 * ln(4*m/beta))` (Theorem 25). This
/// is NOT a tunable hyperparameter — it is the constant proved correct in the paper. Changing
/// it silently breaks the generalization guarantee. LOCKED.
const SIGMA_DENOMINATOR_CONSTANT: f64 = 96.0;

/// `T = (3/4) * tau` (Theorem 25). The base threshold the noisy threshold `T_hat` is built
/// on. LOCKED.
const T_FRACTION_OF_TAU: f64 = 0.75;

/// `gamma ~ Lap(2 * sigma)` — the threshold-noise scale multiplier (`c_gamma = 2*sigma`).
const GAMMA_SCALE_MULTIPLIER: f64 = 2.0;

/// `eta ~ Lap(4 * sigma)` — the per-query comparison-noise scale multiplier
/// (`c_eta = 4*sigma`).
const ETA_SCALE_MULTIPLIER: f64 = 4.0;

/// `xi ~ Lap(1 * sigma)` — the answer-noise scale multiplier (`c_xi = sigma`).
const XI_SCALE_MULTIPLIER: f64 = 1.0;

/// The LOCKED Thresholdout budget `B`: the maximum number of above-threshold
/// (holdout-consulting) responses before the holdout is EXHAUSTED. Set CONSERVATIVELY: too
/// small starves legitimate validation, too large under-bounds reuse. `budget_remaining` is
/// surfaced so the operator sees the tradeoff. Paper bound: `B <= n^2` for exponential query
/// capacity; 16 is a conservative fixed default well below any realistic holdout-size
/// quadratic.
pub const HOLDOUT_BUDGET_TOTAL: u64 = 16;

/// Conservative holdout-size-floor constant `C` in `n >= C * max(2*B/(sigma*tau),
/// ln(6/beta)/tau^2)` (Lemma 24). The paper states "sufficiently large" and does NOT pin it;
/// `C = 2` is the documented conservative choice (interface map Section 14). Used by
/// [`min_holdout_size`] (advisory: the operator sizes the holdout, this gate does not collect
/// the raw sample).
const HOLDOUT_SIZE_FLOOR_CONSTANT: f64 = 2.0;

/// The constant `6` inside `ln(6/beta)/tau^2` (Lemma 24 `n0`).
const HOLDOUT_SIZE_N0_NUMERATOR: f64 = 6.0;

/// Agent id stamped on the emitted [`SsbPayload::HoldoutBudgetEvent`] row.
const HOLDOUT_BUDGET_AGENT_ID: &str = "holdout-budget";

/// Superstep stamped on the emitted row (single-shot gate, not part of a graph loop).
const HOLDOUT_BUDGET_SUPERSTEP: i64 = 0;

/// Minimum scope-key length (bytes) after canonicalization (TD-1 scope-key integrity).
const MIN_KEY_LEN: usize = 1;

/// The `'spent'` row kind (one append-only row per above-threshold/budget-consuming query).
const ROW_KIND_SPENT: &str = "spent";

/// The `'provisioned'` row kind (one append-only row per out-of-band holdout grant).
const ROW_KIND_PROVISIONED: &str = "provisioned";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors returned by [`HoldoutBudgetLedger`] operations.
#[derive(Debug, Error)]
pub enum HoldoutBudgetError {
    /// SQLite layer error.
    #[error("holdout budget sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// A scope key (holdout_id / objective) is empty after canonicalization.
    #[error("scope key is empty after canonicalization (TD-1 scope-key attack surface)")]
    EmptyKey,

    /// SSB write error during HoldoutBudgetEvent emission.
    #[error("SSB write error emitting HoldoutBudgetEvent: {0}")]
    SsbWrite(String),

    /// A fresh-holdout provision was rejected because the new data_window OVERLAPS a window
    /// already spent for this holdout — the covert reset attack (re-labeling the same series).
    #[error(
        "provision rejected: new data_window {new_window} OVERLAPS a spent window \
         {spent_window} for holdout {holdout_id} (covert reset attack — re-labeling the same \
         series does not restore validity)"
    )]
    OverlappingProvisionWindow {
        holdout_id: String,
        new_window: String,
        spent_window: String,
    },

    /// A data_window string could not be parsed as the canonical "start_ms..end_ms" form.
    #[error("malformed data_window {window:?} (expected canonical \"start_ms..end_ms\")")]
    MalformedDataWindow { window: String },
}

// ---------------------------------------------------------------------------
// BudgetRemainingResult — TD-1 fail-closed sentinel (mirror TrialCountResult)
// ---------------------------------------------------------------------------

/// Result of a [`HoldoutBudgetLedger::budget_remaining`] query.
///
/// Distinguishes "no budget provisioned for this holdout" ([`BudgetMissing`], the H-19 hard
/// fail-closed sentinel, TD-1/R1/R2) from a real remaining count. A missing counter is NOT
/// the same as "budget 0": it means the holdout was never provisioned, so the OOS query path
/// must FAIL CLOSED (REFUSE), never silently treat it as a permissive zero.
///
/// [`BudgetMissing`]: BudgetRemainingResult::BudgetMissing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetRemainingResult {
    /// No `'provisioned'` row exists for this holdout. The OOS query gate MUST fail closed on
    /// this variant (REFUSE), never SKIP. "Not found = never provisioned = FAIL" (R2).
    BudgetMissing,
    /// At least a provision exists; `remaining = budget_total - queries_spent` (saturating at
    /// 0). `remaining == 0` means the budget is EXHAUSTED.
    Remaining(u64),
}

impl BudgetRemainingResult {
    /// True when the variant is [`BudgetRemainingResult::BudgetMissing`] (the fail-closed
    /// sentinel).
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::BudgetMissing)
    }

    /// True when an OOS query must be REFUSED: either the counter is missing (fail-closed) OR
    /// the budget is exhausted (`Remaining(0)`).
    pub fn must_refuse(&self) -> bool {
        matches!(self, Self::BudgetMissing | Self::Remaining(0))
    }
}

// ---------------------------------------------------------------------------
// HoldoutQueryOutcome — the verdict of one OOS query
// ---------------------------------------------------------------------------

/// The verdict of a single [`HoldoutBudgetLedger::query_holdout`] OOS query.
#[derive(Debug, Clone, PartialEq)]
pub struct HoldoutQueryOutcome {
    /// The verdict class (maps to the emitted [`HoldoutBudgetKind`]).
    pub kind: HoldoutBudgetKind,
    /// The released statistic. `Some(E_Sh + xi)` on an above-threshold spend (`QuerySpent`);
    /// `Some(E_St)` on a below-threshold FREE answer (also `QuerySpent` kind? — NO: see
    /// `is_spend`); `None` on `Exhausted`.
    pub answer: Option<f64>,
    /// `queries_spent` for the holdout AFTER this decision (the cumulative above-threshold
    /// count). Unchanged on a below-threshold free answer and on `Exhausted`.
    pub queries_spent: u64,
    /// The LOCKED Thresholdout budget `B`.
    pub budget_total: u64,
    /// True iff this query CHARGED the budget (an above-threshold response). False for a
    /// below-threshold FREE answer and for `Exhausted`.
    pub charged: bool,
}

impl HoldoutQueryOutcome {
    /// True iff the OOS query was REFUSED (budget exhausted / missing) — the fail-closed arm.
    pub fn is_refused(&self) -> bool {
        self.kind == HoldoutBudgetKind::Exhausted
    }
}

// ---------------------------------------------------------------------------
// Pure core — the deterministic Thresholdout classification (GD-2 recomputable)
// ---------------------------------------------------------------------------

/// The classification of one Thresholdout query, BEFORE the budget side-effect / SSB row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThresholdoutDecision {
    /// Above the noisy threshold: charge one budget unit, release the noised holdout
    /// statistic `E_Sh + xi`.
    AboveThreshold { noised_holdout: f64 },
    /// Below the noisy threshold: FREE answer from the training mean `E_St` (no charge).
    BelowThreshold { train_stat: f64 },
}

/// PURE, deterministic Thresholdout classification for ONE query (no IO / clock / SSB). The
/// load-bearing math, factored out of [`HoldoutBudgetLedger::query_holdout`] so the contract
/// test can drive it without an SSB (mirrors `classify_reward_tampering`). Recomputable
/// under `ELAI_SEED` (GD-2): same `(train_stat, holdout_stat, seed, query_index)` => same
/// decision => same noised answer.
///
/// The per-query noise is derived from a [`SeededRng`] keyed on `(seed, query_index)` so each
/// query in a session draws an INDEPENDENT but REPRODUCIBLE noise triple (gamma, eta, xi).
///
/// # Algorithm (paper Figure 1, EXACT)
///   1. T_hat = T + Lap(2*sigma)                          (gamma, threshold noise)
///   2. eta = Lap(4*sigma)                                (per-query comparison noise)
///   3. if |holdout_stat - train_stat| > T_hat + eta:     (ABOVE THRESHOLD)
///          xi = Lap(sigma); return AboveThreshold(holdout_stat + xi)
///      else:                                             (BELOW THRESHOLD)
///          return BelowThreshold(train_stat)
///
/// # Arguments
/// * `train_stat`  — `E_St[phi]`, the training-set statistic (1-sensitivity, in `[0,1]`).
/// * `holdout_stat`— `E_Sh[phi]`, the holdout-set statistic.
/// * `seed`        — the declared `ELAI_SEED` (GD-2).
/// * `query_index` — the cumulative spend index BEFORE this query (so each query's noise is
///   distinct and reproducible across re-runs).
pub fn classify_holdout_query(
    train_stat: f64,
    holdout_stat: f64,
    seed: u64,
    query_index: u64,
) -> ThresholdoutDecision {
    let sigma = sigma();
    let threshold = threshold_base();

    // A fresh, reproducible RNG per query: key the seed with the query index so query k draws
    // a distinct-but-deterministic noise triple. rotate_left de-correlates adjacent indices.
    let mut rng = SeededRng::new(seed ^ query_index.rotate_left(32));

    // 1. gamma ~ Lap(2*sigma) -> T_hat = T + gamma.
    let gamma = laplace_sample(&mut rng, GAMMA_SCALE_MULTIPLIER * sigma);
    let t_hat = threshold + gamma;

    // 2. eta ~ Lap(4*sigma).
    let eta = laplace_sample(&mut rng, ETA_SCALE_MULTIPLIER * sigma);

    // 3. the noisy threshold comparison (EXACT: uses the freshly-sampled eta).
    let empirical_gap = (holdout_stat - train_stat).abs();
    if empirical_gap > t_hat + eta {
        // ABOVE THRESHOLD — charge budget, release the noised holdout mean.
        let xi = laplace_sample(&mut rng, XI_SCALE_MULTIPLIER * sigma);
        debug!(
            empirical_gap,
            t_hat,
            eta,
            "holdout_query: ABOVE threshold -> charge 1 unit, release E_Sh + xi"
        );
        ThresholdoutDecision::AboveThreshold {
            noised_holdout: holdout_stat + xi,
        }
    } else {
        // BELOW THRESHOLD — FREE answer from the training mean (no noise, no charge).
        debug!(
            empirical_gap,
            t_hat, eta, "holdout_query: BELOW threshold -> FREE answer E_St (no charge)"
        );
        ThresholdoutDecision::BelowThreshold { train_stat }
    }
}

/// `sigma = tau / (96 * ln(4 * m / beta))` (Theorem 25). The single noise rate that all three
/// Laplace scales are multiples of.
fn sigma() -> f64 {
    HOLDOUT_TAU / (SIGMA_DENOMINATOR_CONSTANT * (4.0 * HOLDOUT_M_QUERIES / HOLDOUT_BETA).ln())
}

/// `T = (3/4) * tau` (Theorem 25). The base threshold (before the gamma noise).
fn threshold_base() -> f64 {
    T_FRACTION_OF_TAU * HOLDOUT_TAU
}

/// Advisory minimum holdout size `n >= C * max(2*B/(sigma*tau), ln(6/beta)/tau^2)` (Lemma 24,
/// conservative `C = 2`). The operator sizes the raw holdout sample; this gate tracks the
/// budget, not the sample, so this is surfaced for tuning, not enforced here.
pub fn min_holdout_size() -> f64 {
    let s = sigma();
    // NARROWING-OK: HOLDOUT_BUDGET_TOTAL is a small const; f64 represents it exactly.
    let b = HOLDOUT_BUDGET_TOTAL as f64;
    let n0_a = (2.0 * b) / (s * HOLDOUT_TAU);
    let n0_b = (HOLDOUT_SIZE_N0_NUMERATOR / HOLDOUT_BETA).ln() / (HOLDOUT_TAU * HOLDOUT_TAU);
    HOLDOUT_SIZE_FLOOR_CONSTANT * n0_a.max(n0_b)
}

/// Sample `Lap(scale)` by inverse-CDF transform: for `u ~ Uniform(-1/2, 1/2)`,
/// `x = -scale * sign(u) * ln(1 - 2*|u|)`. Laplace ONLY (the DP guarantee is for Laplace).
///
/// The uniform draw is `next_u64 / 2^64 - 1/2` (a float in `[-1/2, 1/2)`). `ln(0)` (at the
/// `|u| = 1/2` boundary) is guarded by clamping `1 - 2*|u|` away from zero.
fn laplace_sample(rng: &mut SeededRng, scale: f64) -> f64 {
    // u64 -> uniform in [0, 1): divide by 2^64. Shift to [-1/2, 1/2).
    const TWO_POW_64: f64 = 18_446_744_073_709_551_616.0; // 2^64
    // Smallest positive value to keep ln() finite at the boundary (u = ±1/2 => 1-2|u| = 0).
    const LN_ARG_FLOOR: f64 = 1.0e-12;
    // NARROWING-OK: u64 -> f64 is a widening of the integer magnitude; the / 2^64 maps the
    // full u64 range into [0, 1).
    let unit = rng.next_u64() as f64 / TWO_POW_64;
    let u = unit - 0.5;
    let sign = if u < 0.0 { -1.0 } else { 1.0 };
    let ln_arg = (1.0 - 2.0 * u.abs()).max(LN_ARG_FLOOR);
    -scale * sign * ln_arg.ln()
}

// ---------------------------------------------------------------------------
// HoldoutBudgetLedger — append-only, NO agent reset (mirror TrialLedger)
// ---------------------------------------------------------------------------

/// Append-only persisted holdout query budget (H-19, the Thresholdout DP gate).
///
/// Holds a `rusqlite::Connection` to the `holdout_budget` table (on the same db file as
/// [`Ssb`]). The budget COUNTER is `queries_spent = COUNT('spent' rows)`; `budget_remaining =
/// budget_total - queries_spent`. NO reset / delete / update / provision method exists on
/// this type (TD-1 — a covert budget reset would defeat the gate). The budget once spent
/// STAYS spent.
pub struct HoldoutBudgetLedger {
    conn: std::sync::Mutex<Connection>,
    session_id: String,
}

impl std::fmt::Debug for HoldoutBudgetLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoldoutBudgetLedger")
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

impl HoldoutBudgetLedger {
    /// Open a new `HoldoutBudgetLedger` on `db_path` (the same SQLite file used by `Ssb`).
    /// The `holdout_budget` table is created if it does not yet exist (existing rows
    /// preserved — cross-session accumulation).
    pub fn open(
        db_path: &Path,
        session_id: impl Into<String>,
    ) -> Result<Self, HoldoutBudgetError> {
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(crate::ssb::CREATE_HOLDOUT_BUDGET_TABLE)?;
        // Idempotent migrations for existing databases that have the old 5-column
        // schema (H-19 original). Errors are silently ignored so that both fresh
        // databases (columns already exist via CREATE TABLE) and existing databases
        // (need the column added) open cleanly without a version-gate.
        // Pattern mirrors self_mod_layer1a.rs:631-638 (established codebase pattern).
        let _ = conn.execute("ALTER TABLE holdout_budget ADD COLUMN row_id TEXT NOT NULL DEFAULT ''", []);
        let _ = conn.execute("ALTER TABLE holdout_budget ADD COLUMN kind TEXT NOT NULL DEFAULT 'spent'", []);
        let _ = conn.execute("ALTER TABLE holdout_budget ADD COLUMN data_window TEXT NOT NULL DEFAULT ''", []);
        let _ = conn.execute("ALTER TABLE holdout_budget ADD COLUMN seed INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE holdout_budget ADD COLUMN created_at_ms INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE holdout_budget ADD COLUMN session_id TEXT NOT NULL DEFAULT ''", []);
        Ok(Self {
            conn: std::sync::Mutex::new(conn),
            session_id: session_id.into(),
        })
    }

    /// Read the remaining budget for `(holdout_id, objective)`.
    ///
    /// Returns [`BudgetRemainingResult::BudgetMissing`] when NO `'provisioned'` row exists —
    /// the H-19 fail-closed sentinel (TD-1/R1/R2). Otherwise `Remaining(budget_total -
    /// queries_spent)` (saturating at 0). The OOS query gate REFUSES on `must_refuse()`.
    pub fn budget_remaining(
        &self,
        holdout_id: &str,
        objective: &str,
    ) -> Result<BudgetRemainingResult, HoldoutBudgetError> {
        let holdout_id = canonicalize_key(holdout_id);
        let objective = canonicalize_key(objective);

        let conn = self.lock_conn()?;

        // budget_total comes from the provision row; absent => BudgetMissing (fail closed).
        let budget_total: Option<i64> = conn
            .query_row(
                "SELECT budget_total FROM holdout_budget \
                 WHERE holdout_id = ?1 AND objective = ?2 AND kind = ?3 \
                 ORDER BY created_at_ms ASC LIMIT 1",
                params![holdout_id, objective, ROW_KIND_PROVISIONED],
                |row| row.get(0),
            )
            .ok();

        let Some(budget_total_i64) = budget_total else {
            warn!(
                holdout_id = %holdout_id,
                objective = %objective,
                "budget_remaining: no provisioned row -> BudgetMissing (fail-closed R2)"
            );
            return Ok(BudgetRemainingResult::BudgetMissing);
        };

        // queries_spent = COUNT of 'spent' rows for this holdout (append-only counter).
        // SQLite COUNT(*) returns i64; NARROWING-OK: non-negative by definition.
        let spent_i64: i64 = conn.query_row(
            "SELECT COUNT(*) FROM holdout_budget \
             WHERE holdout_id = ?1 AND objective = ?2 AND kind = ?3",
            params![holdout_id, objective, ROW_KIND_SPENT],
            |row| row.get(0),
        )?;

        let budget_total = u64::try_from(budget_total_i64).unwrap_or(0);
        let spent = u64::try_from(spent_i64).unwrap_or(0);
        Ok(BudgetRemainingResult::Remaining(budget_total.saturating_sub(spent)))
    }

    /// THE CHOKE-POINT: run one Thresholdout OOS query and emit the off-agent SSB row.
    ///
    /// Steps (paper Figure 1):
    ///   1. read `budget_remaining`; if `must_refuse()` (exhausted OR missing) => emit an
    ///      `Exhausted` row and return a REFUSED outcome (fail-closed, R1/R2).
    ///   2. else classify the query (PURE [`classify_holdout_query`]) with the seeded Laplace
    ///      noise (GD-2 reproducible);
    ///   3. on an ABOVE-threshold decision: INSERT one append-only `'spent'` row (the counter
    ///      is the COUNT of these — there is NO UPDATE), then emit a `QuerySpent` row carrying
    ///      the noised statistic;
    ///   4. on a BELOW-threshold decision: emit nothing-charging — return the FREE training
    ///      answer with `charged = false` and the UNCHANGED `queries_spent`.
    ///
    /// # Arguments
    /// * `ssb`         — the project Ssb handle; used to emit `HoldoutBudgetEvent`.
    /// * `clock`       — ClockProvider (C-15 determinism; never SystemTime).
    /// * `holdout_id`  — the holdout-set identity (canonicalized).
    /// * `objective`   — the search/evaluation objective key (canonicalized).
    /// * `data_window` — the canonical "start_ms..end_ms" holdout window string.
    /// * `train_stat`  — `E_St[phi]`.
    /// * `holdout_stat`— `E_Sh[phi]`.
    /// * `run_id`      — declared run id stamped on the row.
    /// * `seed`        — declared `ELAI_SEED` (GD-2 noise reproducibility).
    #[allow(clippy::too_many_arguments)]
    pub async fn query_holdout(
        &self,
        ssb: &Arc<Ssb>,
        clock: &Arc<dyn ClockProvider>,
        holdout_id: &str,
        objective: &str,
        data_window: &str,
        train_stat: f64,
        holdout_stat: f64,
        run_id: &str,
        seed: u64,
    ) -> Result<HoldoutQueryOutcome, HoldoutBudgetError> {
        let holdout_id = canonicalize_key(holdout_id);
        let objective = canonicalize_key(objective);
        if holdout_id.len() < MIN_KEY_LEN || objective.len() < MIN_KEY_LEN {
            return Err(HoldoutBudgetError::EmptyKey);
        }
        let data_window = canonicalize_key(data_window);

        // 1. Budget check FIRST (fail-closed: exhausted OR missing => REFUSE).
        let remaining = self.budget_remaining(&holdout_id, &objective)?;
        let budget_total = match &remaining {
            // Even when missing, surface the LOCKED budget total on the emitted row.
            BudgetRemainingResult::BudgetMissing => HOLDOUT_BUDGET_TOTAL,
            BudgetRemainingResult::Remaining(_) => HOLDOUT_BUDGET_TOTAL,
        };
        if remaining.must_refuse() {
            let spent = self.count_spent(&holdout_id, &objective)?;
            warn!(
                holdout_id = %holdout_id,
                objective = %objective,
                missing = remaining.is_missing(),
                spent,
                "query_holdout: budget exhausted/missing -> REFUSE (Exhausted, fail-closed)"
            );
            self.emit_event(
                ssb,
                HoldoutBudgetKind::Exhausted,
                &holdout_id,
                &objective,
                spent,
                budget_total,
                None,
                &data_window,
                run_id,
                seed,
            )
            .await?;
            return Ok(HoldoutQueryOutcome {
                kind: HoldoutBudgetKind::Exhausted,
                answer: None,
                queries_spent: spent,
                budget_total,
                charged: false,
            });
        }

        // 2. Classify (PURE, seeded, GD-2 recomputable). query_index = spend count BEFORE this
        // query so each query's noise is distinct + reproducible.
        let query_index = self.count_spent(&holdout_id, &objective)?;
        let decision = classify_holdout_query(train_stat, holdout_stat, seed, query_index);

        match decision {
            ThresholdoutDecision::AboveThreshold { noised_holdout } => {
                // 3. INSERT one append-only 'spent' row (the counter is COUNT of these; NO
                //    UPDATE). created_at_ms via the injected clock (C-15).
                let created_at_ms = Ssb::now_ms_via_clock(clock);
                let row_id = compute_row_id(
                    ROW_KIND_SPENT,
                    &holdout_id,
                    &objective,
                    &data_window,
                    seed,
                    query_index,
                );
                {
                    let conn = self.lock_conn()?;
                    conn.execute(
                        "INSERT INTO holdout_budget \
                         (row_id, kind, holdout_id, objective, data_window, budget_total, \
                          seed, created_at_ms, session_id) \
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![
                            row_id,
                            ROW_KIND_SPENT,
                            holdout_id,
                            objective,
                            data_window,
                            // NARROWING-OK via try_into: budget_total is a small const u64.
                            i64::try_from(budget_total).unwrap_or(i64::MAX),
                            // NARROWING-OK: seed is a declared u64; store as i64 bit pattern.
                            i64::from_ne_bytes(seed.to_ne_bytes()),
                            created_at_ms,
                            self.session_id,
                        ],
                    )?;
                }
                let spent_after = query_index + 1;
                // 4. Emit the QuerySpent row carrying the noised statistic.
                self.emit_event(
                    ssb,
                    HoldoutBudgetKind::QuerySpent,
                    &holdout_id,
                    &objective,
                    spent_after,
                    budget_total,
                    Some(noised_holdout),
                    &data_window,
                    run_id,
                    seed,
                )
                .await?;
                Ok(HoldoutQueryOutcome {
                    kind: HoldoutBudgetKind::QuerySpent,
                    answer: Some(noised_holdout),
                    queries_spent: spent_after,
                    budget_total,
                    charged: true,
                })
            }
            ThresholdoutDecision::BelowThreshold { train_stat } => {
                // FREE answer from the training mean — budget UNCHANGED, no spend row, no SSB
                // spend. (The below-threshold path emits no budget event: budget rows == spent
                // count, GD-2.)
                Ok(HoldoutQueryOutcome {
                    kind: HoldoutBudgetKind::QuerySpent,
                    answer: Some(train_stat),
                    queries_spent: query_index,
                    budget_total,
                    charged: false,
                })
            }
        }
    }

    /// Raw count of `'spent'` rows for `(holdout_id, objective)` (the append-only counter).
    fn count_spent(&self, holdout_id: &str, objective: &str) -> Result<u64, HoldoutBudgetError> {
        let conn = self.lock_conn()?;
        // NARROWING-OK: COUNT(*) is non-negative.
        let n_i64: i64 = conn.query_row(
            "SELECT COUNT(*) FROM holdout_budget \
             WHERE holdout_id = ?1 AND objective = ?2 AND kind = ?3",
            params![holdout_id, objective, ROW_KIND_SPENT],
            |row| row.get(0),
        )?;
        Ok(u64::try_from(n_i64).unwrap_or(0))
    }

    /// Lock the connection mutex, mapping a poisoned lock to a SQLite IO error.
    fn lock_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, HoldoutBudgetError> {
        self.conn.lock().map_err(|e| {
            HoldoutBudgetError::Sqlite(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::SystemIoFailure,
                    extended_code: 0,
                },
                Some(format!("mutex poisoned: {e}")),
            ))
        })
    }

    /// Emit one off-agent [`SsbPayload::HoldoutBudgetEvent`] row. `corpus_blake3` binds the
    /// verdict to the query provenance so it is recomputable (GD-2). BLAKE3 ONLY.
    #[allow(clippy::too_many_arguments)]
    async fn emit_event(
        &self,
        ssb: &Arc<Ssb>,
        kind: HoldoutBudgetKind,
        holdout_id: &str,
        objective: &str,
        queries_spent: u64,
        budget_total: u64,
        noisy_statistic: Option<f64>,
        data_window: &str,
        run_id: &str,
        seed: u64,
    ) -> Result<(), HoldoutBudgetError> {
        let key = format!(
            "{run_id}:{holdout_id}:{objective}:{data_window}:{queries_spent}:{budget_total}:{seed}"
        );
        let corpus_blake3 = blake3::hash(key.as_bytes()).to_hex().to_string();

        let payload = SsbPayload::HoldoutBudgetEvent(HoldoutBudgetPayload {
            kind,
            holdout_id: holdout_id.to_string(),
            objective: objective.to_string(),
            queries_spent,
            budget_total,
            noisy_statistic,
            data_window: data_window.to_string(),
            run_id: run_id.to_string(),
            seed,
            corpus_blake3,
        });

        ssb.write_payload(
            HOLDOUT_BUDGET_AGENT_ID,
            HOLDOUT_BUDGET_SUPERSTEP,
            payload,
            vec!["holdout_budget.rs".to_string()],
        )
        .await
        .map_err(|e| HoldoutBudgetError::SsbWrite(format!("{e}")))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Provision-fresh — HUMAN / FROZEN path only (NOT a method on the ledger).
// ---------------------------------------------------------------------------

/// Provision a FRESH holdout out-of-band (the privileged human / frozen path).
///
/// This is deliberately a FREE FUNCTION, NOT a method on [`HoldoutBudgetLedger`], so an agent
/// holding a ledger handle CANNOT call it (the agent-reachable surface has no provision /
/// reset path). It is invoked only by an out-of-band operator tool / frozen seed.
///
/// THE LOAD-BEARING GUARD (the covert reset attack, paper pitfall 5): the new `data_window`
/// MUST be DISJOINT from every window already SPENT for this holdout. Re-labeling or
/// re-splitting the SAME underlying series and provisioning it as "fresh" is a disguised
/// budget reset; a privilege check alone does NOT catch it, so the disjointness assertion is
/// the semantic guard. An overlapping window => [`HoldoutBudgetError::OverlappingProvisionWindow`].
///
/// On success it INSERTS one append-only `'provisioned'` row (carrying `budget_total`) and
/// emits a `Provisioned` SSB row. The spent counter for the NEW (holdout_id, objective) scope
/// starts at zero because the spend rows are keyed by scope; a genuinely fresh holdout uses a
/// fresh scope key.
///
/// R8 residue: the human/frozen provision path is a residual human-trust dependency. It
/// BOUNDS adaptive reuse; it does NOT make OOS evaluation infinite.
#[allow(clippy::too_many_arguments)]
pub async fn provision_fresh_holdout(
    ledger: &HoldoutBudgetLedger,
    ssb: &Arc<Ssb>,
    clock: &Arc<dyn ClockProvider>,
    holdout_id: &str,
    objective: &str,
    data_window: &str,
    budget_total: u64,
    seed: u64,
    run_id: &str,
) -> Result<(), HoldoutBudgetError> {
    let holdout_id = canonicalize_key(holdout_id);
    let objective = canonicalize_key(objective);
    if holdout_id.len() < MIN_KEY_LEN || objective.len() < MIN_KEY_LEN {
        return Err(HoldoutBudgetError::EmptyKey);
    }
    let data_window = canonicalize_key(data_window);
    let new_window = parse_window(&data_window)?;

    // THE GUARD: assert the new window is DISJOINT from every SPENT window for this holdout.
    {
        let conn = ledger.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT data_window FROM holdout_budget \
             WHERE holdout_id = ?1 AND objective = ?2 AND kind = ?3",
        )?;
        let spent_windows = stmt.query_map(
            params![holdout_id, objective, ROW_KIND_SPENT],
            |row| row.get::<_, String>(0),
        )?;
        for spent in spent_windows {
            let spent = spent?;
            let spent_win = parse_window(&spent)?;
            if windows_overlap(new_window, spent_win) {
                return Err(HoldoutBudgetError::OverlappingProvisionWindow {
                    holdout_id: holdout_id.clone(),
                    new_window: data_window.clone(),
                    spent_window: spent,
                });
            }
        }
    }

    // INSERT the append-only 'provisioned' row.
    let created_at_ms = Ssb::now_ms_via_clock(clock);
    let row_id = compute_row_id(
        ROW_KIND_PROVISIONED,
        &holdout_id,
        &objective,
        &data_window,
        seed,
        0,
    );
    {
        let conn = ledger.lock_conn()?;
        conn.execute(
            "INSERT INTO holdout_budget \
             (row_id, kind, holdout_id, objective, data_window, budget_total, seed, \
              created_at_ms, session_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                row_id,
                ROW_KIND_PROVISIONED,
                holdout_id,
                objective,
                data_window,
                i64::try_from(budget_total).unwrap_or(i64::MAX),
                i64::from_ne_bytes(seed.to_ne_bytes()),
                created_at_ms,
                ledger.session_id,
            ],
        )?;
    }

    ledger
        .emit_event(
            ssb,
            HoldoutBudgetKind::Provisioned,
            &holdout_id,
            &objective,
            0,
            budget_total,
            None,
            &data_window,
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
/// fragmentation attacks (mirrors `trial_ledger::canonicalize_key`, TD-1).
fn canonicalize_key(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Compute a row_id = blake3 hex of the canonical tuple. blake3 ONLY (CLAUDE.md HARD RULE).
fn compute_row_id(
    kind: &str,
    holdout_id: &str,
    objective: &str,
    data_window: &str,
    seed: u64,
    query_index: u64,
) -> String {
    let tuple = format!("{kind}:{holdout_id}:{objective}:{data_window}:{seed}:{query_index}");
    blake3::hash(tuple.as_bytes()).to_hex().to_string()
}

/// Parse a canonical "start_ms..end_ms" data_window into an `(start, end)` pair.
fn parse_window(window: &str) -> Result<(i64, i64), HoldoutBudgetError> {
    let parts: Vec<&str> = window.split("..").collect();
    if parts.len() != 2 {
        return Err(HoldoutBudgetError::MalformedDataWindow {
            window: window.to_string(),
        });
    }
    let start = parts[0]
        .trim()
        .parse::<i64>()
        .map_err(|_| HoldoutBudgetError::MalformedDataWindow {
            window: window.to_string(),
        })?;
    let end = parts[1]
        .trim()
        .parse::<i64>()
        .map_err(|_| HoldoutBudgetError::MalformedDataWindow {
            window: window.to_string(),
        })?;
    Ok((start, end))
}

/// Half-open interval overlap: `[a0, a1)` overlaps `[b0, b1)` iff `a0 < b1 && b0 < a1`.
fn windows_overlap(a: (i64, i64), b: (i64, i64)) -> bool {
    a.0 < b.1 && b.0 < a.1
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::{SsbActionType, SsbConfig, SsbReader};
    use elai_core::clock::OsClock;
    use tempfile::TempDir;

    fn test_clock() -> Arc<dyn ClockProvider> {
        Arc::new(OsClock)
    }

    fn test_ssb(dir: &TempDir) -> Arc<Ssb> {
        let cfg = SsbConfig {
            db_path: dir.path().join("ssb.db"),
            pool_size: Some(2),
            project_id: "h19-test".into(),
            session_id: "h19-test-session".into(),
        };
        Arc::new(Ssb::new(cfg, test_clock()).expect("ssb"))
    }

    fn test_ledger(dir: &TempDir) -> HoldoutBudgetLedger {
        HoldoutBudgetLedger::open(&dir.path().join("ssb.db"), "h19-test-session")
            .expect("holdout ledger open")
    }

    #[test]
    fn sigma_formula_matches_theorem_25() {
        // sigma * 96 * ln(4*m/beta) == tau (Theorem 25 calibration).
        let lhs = sigma() * SIGMA_DENOMINATOR_CONSTANT
            * (4.0 * HOLDOUT_M_QUERIES / HOLDOUT_BETA).ln();
        assert!(
            (lhs - HOLDOUT_TAU).abs() < 1e-12,
            "sigma*96*ln(4m/beta) must equal tau"
        );
    }

    #[test]
    fn threshold_is_three_quarters_tau() {
        assert!((threshold_base() - 0.75 * HOLDOUT_TAU).abs() < 1e-12);
    }

    #[test]
    fn laplace_mean_is_near_zero_and_scale_is_correct() {
        // Empirical mean ~ 0; empirical variance ~ 2*scale^2 (Laplace).
        let mut rng = SeededRng::new(12345);
        let scale = 0.3_f64;
        let n = 200_000;
        let mut sum = 0.0_f64;
        let mut sumsq = 0.0_f64;
        for _ in 0..n {
            let x = laplace_sample(&mut rng, scale);
            sum += x;
            sumsq += x * x;
        }
        let mean = sum / n as f64;
        let var = sumsq / n as f64 - mean * mean;
        assert!(mean.abs() < 0.02, "Laplace mean ~ 0, got {mean}");
        // Var(Lap(b)) = 2*b^2.
        let expected_var = 2.0 * scale * scale;
        assert!(
            (var - expected_var).abs() < 0.02,
            "Laplace var ~ 2*scale^2 ({expected_var}), got {var}"
        );
    }

    #[test]
    fn noise_scales_are_two_four_one_times_sigma() {
        let s = sigma();
        assert!((GAMMA_SCALE_MULTIPLIER * s - 2.0 * s).abs() < 1e-12);
        assert!((ETA_SCALE_MULTIPLIER * s - 4.0 * s).abs() < 1e-12);
        assert!((XI_SCALE_MULTIPLIER * s - 1.0 * s).abs() < 1e-12);
    }

    #[test]
    fn below_threshold_returns_train_stat_no_charge() {
        // train == holdout => zero empirical gap => always below threshold => FREE train_stat.
        let d = classify_holdout_query(0.6, 0.6, 42, 0);
        match d {
            ThresholdoutDecision::BelowThreshold { train_stat } => {
                assert!((train_stat - 0.6).abs() < 1e-12);
            }
            ThresholdoutDecision::AboveThreshold { .. } => {
                panic!("zero gap must be below threshold");
            }
        }
    }

    #[test]
    fn above_threshold_returns_noised_holdout() {
        // A large gap (|0.9 - 0.1| = 0.8 >> T~0.0375) => above threshold => noised holdout.
        let d = classify_holdout_query(0.1, 0.9, 42, 0);
        match d {
            ThresholdoutDecision::AboveThreshold { noised_holdout } => {
                // Noised, so not exactly 0.9, but the xi scale (sigma~tiny) keeps it close.
                assert!((noised_holdout - 0.9).abs() < 0.5, "noised holdout near 0.9");
            }
            ThresholdoutDecision::BelowThreshold { .. } => {
                panic!("0.8 gap must be above threshold");
            }
        }
    }

    #[test]
    fn classify_is_seed_reproducible() {
        let a = classify_holdout_query(0.1, 0.9, 777, 3);
        let b = classify_holdout_query(0.1, 0.9, 777, 3);
        assert_eq!(a, b, "same (inputs, seed, index) => identical decision (GD-2)");
    }

    #[test]
    fn budget_missing_for_unprovisioned_holdout() {
        let dir = TempDir::new().unwrap();
        let ledger = test_ledger(&dir);
        let r = ledger.budget_remaining("sharpe_1y", "sharpe_1y").unwrap();
        assert!(r.is_missing(), "unprovisioned holdout => BudgetMissing");
        assert!(r.must_refuse(), "missing must REFUSE (fail-closed)");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn above_threshold_charges_and_emits_query_spent() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let clock = test_clock();
        let ledger = test_ledger(&dir);

        provision_fresh_holdout(
            &ledger, &ssb, &clock, "h", "sharpe", "0..100", HOLDOUT_BUDGET_TOTAL, 42, "r",
        )
        .await
        .expect("provision");

        let out = ledger
            .query_holdout(&ssb, &clock, "h", "sharpe", "0..100", 0.1, 0.9, "r", 42)
            .await
            .expect("query");
        assert_eq!(out.kind, HoldoutBudgetKind::QuerySpent);
        assert!(out.charged, "a 0.8 gap charges the budget");
        assert_eq!(out.queries_spent, 1);

        let rows = ssb
            .query_by_type(SsbActionType::HoldoutBudgetEvent)
            .expect("query");
        // 1 Provisioned + 1 QuerySpent.
        assert_eq!(rows.len(), 2, "Provisioned + QuerySpent rows");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn exhausted_budget_refuses() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let clock = test_clock();
        let ledger = test_ledger(&dir);

        provision_fresh_holdout(&ledger, &ssb, &clock, "h", "obj", "0..100", 1, 42, "r")
            .await
            .expect("provision budget=1");

        // Spend the single unit (0.8 gap => above threshold).
        let first = ledger
            .query_holdout(&ssb, &clock, "h", "obj", "0..100", 0.1, 0.9, "r", 42)
            .await
            .expect("first");
        assert!(first.charged);

        // Next above-threshold query must be REFUSED (budget exhausted).
        let second = ledger
            .query_holdout(&ssb, &clock, "h", "obj", "0..100", 0.1, 0.9, "r", 42)
            .await
            .expect("second");
        assert_eq!(second.kind, HoldoutBudgetKind::Exhausted);
        assert!(second.is_refused(), "exhausted => REFUSE");
        assert!(second.answer.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn provision_overlapping_window_is_rejected() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let clock = test_clock();
        let ledger = test_ledger(&dir);

        provision_fresh_holdout(&ledger, &ssb, &clock, "h", "obj", "0..100", 4, 42, "r")
            .await
            .expect("provision");
        // Spend one unit over window 0..100.
        ledger
            .query_holdout(&ssb, &clock, "h", "obj", "0..100", 0.1, 0.9, "r", 42)
            .await
            .expect("spend");

        // Re-provision a window OVERLAPPING the spent one (50..150) => rejected (reset attack).
        let err = provision_fresh_holdout(
            &ledger, &ssb, &clock, "h", "obj", "50..150", 4, 42, "r",
        )
        .await;
        assert!(
            matches!(err, Err(HoldoutBudgetError::OverlappingProvisionWindow { .. })),
            "overlapping provision window must be rejected (covert reset attack)"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn budget_stays_spent_across_reopen() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let clock = test_clock();
        {
            let ledger = test_ledger(&dir);
            provision_fresh_holdout(&ledger, &ssb, &clock, "h", "obj", "0..100", 4, 42, "r")
                .await
                .expect("provision");
            ledger
                .query_holdout(&ssb, &clock, "h", "obj", "0..100", 0.1, 0.9, "r", 42)
                .await
                .expect("spend");
        }
        // Re-open: the spent counter persists (the budget once spent STAYS spent).
        let ledger2 = test_ledger(&dir);
        let r = ledger2.budget_remaining("h", "obj").unwrap();
        assert_eq!(
            r,
            BudgetRemainingResult::Remaining(3),
            "1 spent of 4 => 3 remaining after re-open"
        );
    }

    #[test]
    fn no_reset_or_provision_method_exists() {
        // Negative compile-test (mirror trial_ledger.rs:664). The agent-facing
        // HoldoutBudgetLedger must NOT expose any reset / delete / update / provision method.
        // provision_fresh_holdout is a FREE FUNCTION, not a method, so an agent holding only a
        // ledger handle cannot reach it. If a banned method were added, the commented calls
        // below would compile and this test's intent would be violated.
        let _ = |_l: &HoldoutBudgetLedger| {
            // These must NOT compile:
            // _l.reset_budget("h", "o");
            // _l.delete_budget("h", "o");
            // _l.update_budget("h", 0);
            // _l.provision_fresh_holdout(...);
            // _l.refresh_holdout("h");
        };
    }
}
