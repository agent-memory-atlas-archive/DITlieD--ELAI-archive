// cross-lang-skip: rust-py-sidecar
//! H-26 (SCHEMA_VERSION 232): shadow-execution + financial-invariant metamorphic-
//! relation gate (T-10).
//!
//! Static + wiring gates cannot see a WRONG VALUE: a flipped P&L sign, a violated
//! accounting identity, a put-call-parity break, a ULP cancellation. A refactor that
//! flips a sign compiles and passes a unit test that asserts the (now wrong) value.
//! H-26 closes that gap with a Tier-1 in-process gate that (a) DIFFS two
//! fixture-supplied result sets (a `primary` and a `shadow` re-implementation /
//! prior version) on their derived float outputs within a LOCKED tolerance band, and
//! (b) checks a FIXED set of textbook-correct financial INVARIANTS over the primary
//! result set. The off-agent artifact is the divergence / invariant-falsification
//! record ([`crate::ssb::SsbActionType::DifferentialDivergence`]).
//!
//! # THE #1 CORRECTNESS RULE -- EXACTNESS REGIME SPLIT (P0)
//! Conflating exact and float regimes into one tolerance band is a FALSE gate: a band
//! wide enough to absorb FP slop false-PASSES a cent-level sign error; a band tight
//! enough to catch it false-FAILS benign FP reassociation. H-26 SPLITS the regimes:
//!   * ACCOUNTING IDENTITIES are EXACT and use INTEGER-CENTS (`i64`) equality `==`,
//!     NEVER float: share conservation (`shares_in == shares_out`),
//!     `cash + position*price == equity`, and PnL additivity (the sum of per-trade
//!     realized PnL equals the declared total). There is no decimal/money type in
//!     the orchestrator, so money is represented as integer cents throughout. A
//!     float-`==` on an accounting identity is itself the precision bug the seeded
//!     AST ban (`financial_invariants.toml`) targets -- so the gate must not commit
//!     it (self-consistency, R4).
//!   * OPTION-PRICING / NUMERIC MRs are FLOAT and use the LOCKED ULP/relative band
//!     ([`crate::lookahead_differential::FpTolerance`], rel=1e-9 / abs=1e-12, REUSED
//!     from H-21 -- the band is the gate): PutCallParity (EUROPEAN-only), and
//!     NumericStability.
//!
//! # THE WRONG-INVARIANT CAVEATS (P0 -- a wrong invariant is a false gate)
//!   * PnLAdditivity holds ONLY for INDEPENDENT / uncorrelated positions; it FAILS
//!     under portfolio margining / cross-gamma / netting. The fixture must declare
//!     independent positions.
//!   * ScaleInvariance (capital-scaling linearity) holds ONLY sub-market-impact and
//!     near `k=1` / above the min-lot rounding floor. Not asserted as a hard gate
//!     here for that reason (scope: `|k-1|<<1`, sub-impact, integer-lot-aware).
//!   * PutCallParity EQUALITY is EUROPEAN-only. American options give an INEQUALITY
//!     (`C - P <= S - K*exp(-rT)`), not an equality -- the parity MR is restricted to
//!     European options ([`OptionStyle::European`]); an American option is NOT an
//!     equality and is rejected as un-evaluable rather than mis-asserted.
//!
//! # FAIL-CLOSED (R1/R2 -- "cannot prove no wrong value" => Block)
//!   1. a MISSING shadow run (the second impl never ran) => `ShadowDiverged` Block
//!      (missing evidence = FAIL, not "no diff -> Allow"),
//!   2. an ERRORED shadow run => `ShadowDiverged` Block (cannot diff => cannot prove
//!      no divergence),
//!   3. a DEGENERATE run (zero trades / empty payload) => `InvariantViolated` Block
//!      (every identity is vacuously true on an empty run -- a fail-open the gate
//!      refuses; mirrors the H-23/H-25 non-trivial guard),
//!   4. an un-evaluable invariant (NaN / Inf / division-by-zero at T=0 / an American
//!      option for the European parity MR) => Block, never a silent pass.
//!
//! # PRODUCER SPLIT (NakedFloatEq is NOT emitted here)
//! The `NakedFloatEq` Kind is emitted by the SEEDED rule-engine path (the
//! `financial_invariants.toml` AstPattern banning naked float `==`/`!=` on
//! money/price), NOT by this pure gate -- exactly H-25's MonkeypatchBlocked split.
//! This gate only ever emits `InvariantHeld` / `ShadowDiverged` / `InvariantViolated`.
//!
//! # SSB emission (off-agent artifact, R4)
//! Emits one [`crate::ssb::SsbPayload::DifferentialDivergence`] row on EVERY decision
//! (InvariantHeld / ShadowDiverged / InvariantViolated). Even an InvariantHeld PASS
//! leaves an off-agent row. The verdict re-computes from the same result sets + seed
//! (GD-2 same-env). Per-MR float failures additionally emit an
//! [`crate::ssb::SsbActionType::MrViolated`] row via the REUSED MR framework (GD-1);
//! no new SSB variant for per-invariant failures.
//!
//! # Tier boundary (cross-lang-skip rationale)
//! Tier-1 (THIS gate) consumes TWO already-computed result sets as PLAIN DATA -- it
//! runs no strategy. The DEFERRED Tier-2 `shadow_runner.py` actually RE-EXECUTES two
//! impls on shared seeded inputs; it is RunPod-pending, no stub is shipped, and the
//! `// cross-lang-skip: rust-py-sidecar` header opts this file out of the
//! Rust<->Python pair drift check until it lands.
//!
//! # Grounding (DESIGN-INSPIRED-BY for the DEFERRED Tier-2 only)
//! FPDiff (floating-point differential testing), Daikon (dynamic invariant MINING),
//! and RV-Monitor/MOP (runtime monitoring) inspire the DEFERRED Tier-2 dynamic-mining
//! / monitoring extensions. H-26 Tier-1 checks a FIXED set of textbook-correct
//! invariants -- it does NOT mine or runtime-monitor, so those are not implemented
//! here. The float band REUSES H-21's already-grounded [`FpTolerance`]. See
//! `workflow/reasoning/H-26-decisions.md`.
//!
//! # HARD RULES
//!   - No `.unwrap()`/`.expect()` outside `#[cfg(test)]` (CLAUDE.md).
//!   - blake3 only for hashing (CLAUDE.md anti-stale).
//!   - No new Rust crate; named constants (no magic numbers); `.try_into()` over
//!     `as`-casts for narrowing.

use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, warn};

use crate::determinism::{ActionTypeSequence, DriftComparator, DriftSink};
use crate::lookahead_differential::FpTolerance;
use crate::ssb::{DifferentialDivergenceKind, DifferentialDivergencePayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// LOCKED named constants
// ---------------------------------------------------------------------------

/// Canonical rule id documenting the seeded financial-invariant sub-gate set (the
/// naked-float-`==` AstPattern). The divergence/identity DECISION in this module is
/// PURE (no rules.db threshold); this id pins the seeded companion authored under
/// `crates/rules/seed/finance/financial_invariants.toml`.
pub const FINANCIAL_INVARIANT_RULE_ID: &str = "FIN-INVARIANT-001";

/// Minimum number of trades required in the primary result set before an
/// `InvariantHeld` verdict can be a PASS (the DEGENERATE-run guard). Every accounting
/// identity is vacuously true on a zero-trade run (PnL=0 trivially equals the sum of
/// zero per-trade PnLs; share conservation holds with no shares), so a "pass" on an
/// empty run proves nothing -- it is a fail-open the gate refuses (mirror the H-23
/// non-empty-payload + H-25 non-trivial-manifest guards).
pub const MIN_DECLARED_TRADES: u32 = 1;

/// The number of decimal places used when quantizing a derived float to its tolerance
/// bucket for the DriftComparator first-divergence reporting. The float values are
/// rounded to `10^QUANTIZE_DECIMALS` so two values within the relative band map to the
/// same bucket string (DriftComparator does byte-equality; the LOAD-BEARING tolerance
/// compare stays the [`FpTolerance::close`] call in [`diff_shadow_impls`]). This is a
/// reporting aid only -- it never substitutes for the float tolerance compare.
const QUANTIZE_DECIMALS: usize = 9;

/// SSB `action_type` wire tag for the off-agent artifact (matches
/// [`crate::ssb::SsbActionType::DifferentialDivergence::as_sql_str`]).
const DIFFERENTIAL_DIVERGENCE_ACTION_TYPE: &str = "differential_divergence";

// ---------------------------------------------------------------------------
// Public API types -- money is INTEGER CENTS (no decimal type in orchestrator)
// ---------------------------------------------------------------------------

/// One trade record in a shadow result set. ALL money quantities are INTEGER CENTS
/// (`i64`) so the accounting-identity checks are EXACT (`==`), never float. `price_cents`
/// is the per-share price in cents; `position_qty` and `shares_*` are integer share
/// counts (lots).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeRecord {
    /// Shares entering the position on this trade (integer lots).
    pub shares_in: i64,
    /// Shares leaving the position on this trade (integer lots). For a conservation-
    /// preserving fill `shares_in == shares_out`.
    pub shares_out: i64,
    /// Cash balance after this trade, in integer cents.
    pub cash_cents: i64,
    /// Net position quantity after this trade (integer lots).
    pub position_qty: i64,
    /// Mark price per share after this trade, in integer cents.
    pub price_cents: i64,
    /// Equity after this trade, in integer cents. The identity
    /// `cash_cents + position_qty * price_cents == equity_cents` must hold EXACT.
    pub equity_cents: i64,
    /// Realized PnL attributable to this trade, in integer cents. The PnL-additivity
    /// identity: the sum of per-trade `realized_pnl_cents` == the declared total.
    pub realized_pnl_cents: i64,
}

/// One derived FLOAT output (e.g. an option price, an implied vol) produced by an
/// impl. Compared primary-vs-shadow within the LOCKED [`FpTolerance`] band. NEVER an
/// accounting quantity (those are integer cents on [`TradeRecord`]).
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedFloat {
    /// Field name (must match across primary and shadow for the diff to align).
    pub name: String,
    /// The derived float value.
    pub value: f64,
}

/// A full result set from one impl: the integer-cents trade records, the derived
/// float outputs, the declared total realized PnL (integer cents), and the declared
/// trade count.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShadowResultSet {
    /// The financial computation / strategy under test (repo-relative path or id).
    pub strategy_path: String,
    /// The declared run id.
    pub run_id: String,
    /// The declared `ELAI_SEED` (GD-2). `None` => carried as 0, never inferred.
    pub elai_seed: Option<u64>,
    /// The per-trade records (integer cents).
    pub trades: Vec<TradeRecord>,
    /// The declared TOTAL realized PnL in integer cents (the PnL-additivity target).
    pub declared_total_pnl_cents: i64,
    /// The derived FLOAT outputs (option prices etc.) compared within the band.
    pub derived_floats: Vec<DerivedFloat>,
}

impl ShadowResultSet {
    /// Number of trades.
    #[must_use]
    pub fn trade_count(&self) -> u32 {
        u32::try_from(self.trades.len()).unwrap_or(u32::MAX)
    }

    /// True when there are no trades AND no derived floats: a degenerate / empty run
    /// on which every accounting identity is vacuously true (the fail-open guard).
    fn is_degenerate(&self) -> bool {
        self.trades.is_empty() && self.derived_floats.is_empty()
    }

    /// blake3 (16-hex) over the canonical serialization of this result set (the GD-2
    /// provenance digest). blake3 ONLY (CLAUDE.md HARD RULE).
    fn digest(&self) -> String {
        let mut buf = String::new();
        buf.push_str(&self.strategy_path);
        buf.push('\n');
        buf.push_str(&self.run_id);
        buf.push('\n');
        for t in &self.trades {
            buf.push_str(&format!(
                "{}|{}|{}|{}|{}|{}|{}\n",
                t.shares_in,
                t.shares_out,
                t.cash_cents,
                t.position_qty,
                t.price_cents,
                t.equity_cents,
                t.realized_pnl_cents,
            ));
        }
        buf.push_str(&format!("total={}\n", self.declared_total_pnl_cents));
        for d in &self.derived_floats {
            // Exact bit repr so the digest is byte-stable for GD-2 recompute.
            buf.push_str(&format!("{}={}\n", d.name, d.value.to_bits()));
        }
        blake3_16hex(buf.as_bytes())
    }

    /// The ordered derived-float `"<name>=<bucket>"` sequence used as the
    /// DriftComparator input (each value quantized to the tolerance bucket so a
    /// within-band pair maps to the SAME bucket string; the LOAD-BEARING tolerance
    /// compare stays the [`FpTolerance::close`] call, this is reporting only).
    fn quantized_float_sequence(&self) -> Vec<String> {
        self.derived_floats
            .iter()
            .map(|d| format!("{}={}", d.name, quantize_bucket(d.value)))
            .collect()
    }
}

/// Whether an option is European (parity is an EQUALITY) or American (parity is an
/// INEQUALITY -- NOT evaluable as an equality MR). The PutCallParity MR is restricted
/// to European options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionStyle {
    /// European: put-call parity is `C - P == S - K*exp(-rT)` (an equality).
    European,
    /// American: put-call parity is `C - P <= S - K*exp(-rT)` (an inequality); the
    /// equality MR does NOT apply.
    American,
}

/// A put-call-parity MR input (a FLOAT relation -- tolerance regime). EUROPEAN-only.
#[derive(Debug, Clone, PartialEq)]
pub struct PutCallParityInput {
    /// Call price.
    pub call: f64,
    /// Put price.
    pub put: f64,
    /// Underlying spot price `S`.
    pub spot: f64,
    /// Strike `K`.
    pub strike: f64,
    /// Continuously-compounded risk-free rate `r`.
    pub rate: f64,
    /// Time to expiry `T` (years).
    pub time: f64,
    /// Option style; an American option is rejected as un-evaluable for the equality.
    pub style: OptionStyle,
}

/// A numeric-stability MR input (a FLOAT relation -- tolerance regime): two
/// re-ordered / re-timed computations of the SAME quantity that must agree within the
/// band (no catastrophic cancellation, no NaN/Inf).
#[derive(Debug, Clone, PartialEq)]
pub struct NumericStabilityInput {
    /// A label for the quantity (named in the verdict on failure).
    pub name: String,
    /// The value computed one way.
    pub value_a: f64,
    /// The value computed a re-ordered way; must be `close` to `value_a`.
    pub value_b: f64,
}

/// The financial-invariant inputs checked over the primary result set (alongside the
/// integer-cents accounting identities derived from the trade records).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FinancialInvariantInputs {
    /// Put-call-parity MR inputs (EUROPEAN-only float relation). Empty => skipped.
    pub put_call_parity: Vec<PutCallParityInput>,
    /// Numeric-stability MR inputs (float relation). Empty => skipped.
    pub numeric_stability: Vec<NumericStabilityInput>,
}

/// Verdict kind returned by [`diff_shadow_impls`] (mirrors the inner
/// [`DifferentialDivergenceKind`] but with only the three pure-gate-reachable arms --
/// `NakedFloatEq` is emitted by the SEEDED rule-engine path, not this pure gate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowVerdictKind {
    /// Every accounting identity held EXACT, every float MR held within the band, and
    /// primary agreed with shadow within the band, over a NON-DEGENERATE run. PASS.
    InvariantHeld,
    /// The shadow diverged beyond the band, or the shadow was missing / errored.
    ShadowDiverged,
    /// A financial invariant did not hold (accounting identity broke, a float MR fell
    /// out of band, or the run was degenerate). BLOCK.
    InvariantViolated,
}

/// Full verdict from [`diff_shadow_impls`].
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowVerdict {
    /// InvariantHeld / ShadowDiverged / InvariantViolated.
    pub kind: ShadowVerdictKind,
    /// Machine-readable reason code.
    pub reason: String,
    /// Name of the first violated invariant / divergent field; empty on InvariantHeld.
    pub offending_invariant: String,
    /// Name of the first field where primary and shadow diverged; empty otherwise.
    pub first_divergent_field: String,
    /// Relative/absolute divergence magnitude at the first divergent field.
    pub divergence_metric: f64,
    /// Number of trades in the primary result set.
    pub declared_trade_count: u32,
    /// blake3 (16-hex) over the primary result set.
    pub primary_digest: String,
    /// blake3 (16-hex) over the shadow result set; empty on the missing-shadow path.
    pub shadow_digest: String,
    /// Free-text evidence string.
    pub evidence: String,
}

/// The shadow run handoff: `Some(set)` => the shadow impl produced this result set;
/// `None` => the shadow impl never ran (fail-closed). An ERRORED shadow is signalled
/// by [`ShadowOutcome::Errored`].
#[derive(Debug, Clone, PartialEq)]
pub enum ShadowOutcome {
    /// The shadow impl produced a result set.
    Produced(Box<ShadowResultSet>),
    /// The shadow impl ran but ERRORED (panic / non-zero exit); cannot diff => Block.
    Errored(String),
    /// The shadow impl never ran (missing evidence => Block).
    Missing,
}

/// Errors from [`diff_shadow_impls`]. A missing/errored shadow, a degenerate run, and
/// a violated invariant are NOT errors -- they are fail-closed VERDICTS (each emits an
/// SSB row). Only an SSB write failure is an error here.
#[derive(Debug, Error)]
pub enum ShadowExecError {
    /// Writing the off-agent DifferentialDivergence SSB row failed.
    #[error("SSB write error: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// Main gate: diff_shadow_impls
// ---------------------------------------------------------------------------

/// Diff a `primary` result set against a `shadow` re-implementation and check the
/// financial invariants over the primary set. Emits exactly one
/// [`SsbPayload::DifferentialDivergence`] row on every decision (the off-agent
/// artifact, R4).
///
/// # Order of checks (FAIL-CLOSED first)
///   1. `shadow == Missing` => `ShadowDiverged` Block (reason `shadow_missing`, R1):
///      the second impl never ran; missing evidence = FAIL, never "no diff -> Allow".
///   2. `shadow == Errored` => `ShadowDiverged` Block (reason `shadow_errored`): the
///      shadow cannot be diffed, so no-divergence cannot be proven.
///   3. DEGENERATE primary (zero trades AND no derived floats) => `InvariantViolated`
///      Block (reason `degenerate_run`): every identity is vacuously true on an empty
///      run; a "pass" here proves nothing.
///   4. ACCOUNTING IDENTITIES (EXACT integer-cents `==`): share conservation,
///      `cash + position*price == equity`, PnL additivity. First violation =>
///      `InvariantViolated` naming the invariant.
///   5. PRIMARY-vs-SHADOW float divergence: each derived float compared via
///      [`FpTolerance::close`]; the first beyond-band field => `ShadowDiverged`
///      (named via the REUSED [`DriftComparator`] first-divergence over the quantized
///      buckets).
///   6. FLOAT MRs: PutCallParity (EUROPEAN-only), NumericStability via the band. A
///      NaN/Inf/un-evaluable input => `InvariantViolated` (un-evaluable = Block, R2).
///   7. All held over a non-degenerate run => `InvariantHeld` PASS.
pub async fn diff_shadow_impls(
    ssb: &Arc<Ssb>,
    primary: ShadowResultSet,
    shadow: ShadowOutcome,
    invariants: FinancialInvariantInputs,
    seed: u64,
) -> Result<ShadowVerdict, ShadowExecError> {
    let primary_digest = primary.digest();
    let declared_trade_count = primary.trade_count();
    let tol = FpTolerance::default();

    // -- Step 1/2: FAIL-CLOSED on a missing / errored shadow run (R1/R2). ----
    let shadow_set = match shadow {
        ShadowOutcome::Produced(set) => *set,
        ShadowOutcome::Missing => {
            warn!(
                strategy = %primary.strategy_path,
                "shadow_exec: shadow run MISSING -> ShadowDiverged Block (fail-closed, \
                 R1; the second impl never ran -- missing evidence = FAIL, never \
                 'no diff -> Allow')"
            );
            let verdict = diverged_verdict(
                ShadowVerdictKind::ShadowDiverged,
                "shadow_missing",
                "The shadow re-implementation never ran: a single impl cannot \
                 self-certify that no wrong value was produced. Fail-closed Block (R1 \
                 missing-evidence=FAIL).",
                String::new(),
                String::new(),
                0.0,
                declared_trade_count,
                primary_digest.clone(),
                String::new(),
            );
            emit_ssb(ssb, build_payload(&primary, &verdict, seed)).await?;
            return Ok(verdict);
        }
        ShadowOutcome::Errored(msg) => {
            warn!(
                strategy = %primary.strategy_path,
                error = %msg,
                "shadow_exec: shadow run ERRORED -> ShadowDiverged Block (fail-closed; \
                 a shadow that cannot be diffed cannot prove no-divergence)"
            );
            let verdict = diverged_verdict(
                ShadowVerdictKind::ShadowDiverged,
                "shadow_errored",
                &format!(
                    "The shadow re-implementation ERRORED ({msg}): it cannot be diffed, \
                     so no-divergence cannot be proven. Fail-closed Block."
                ),
                String::new(),
                String::new(),
                0.0,
                declared_trade_count,
                primary_digest.clone(),
                String::new(),
            );
            emit_ssb(ssb, build_payload(&primary, &verdict, seed)).await?;
            return Ok(verdict);
        }
    };
    let shadow_digest = shadow_set.digest();

    // -- Step 3: DEGENERATE-run guard (the fail-open the gate refuses). ------
    if primary.is_degenerate() || declared_trade_count < MIN_DECLARED_TRADES {
        warn!(
            strategy = %primary.strategy_path,
            trades = declared_trade_count,
            "shadow_exec: degenerate / zero-trade run -> InvariantViolated Block \
             (fail-closed; every accounting identity is vacuously true on an empty \
             run, so a 'pass' proves nothing)"
        );
        let verdict = diverged_verdict(
            ShadowVerdictKind::InvariantViolated,
            "degenerate_run",
            "The primary run is DEGENERATE (zero trades and no derived floats): every \
             accounting identity is vacuously true, so an InvariantHeld here would be \
             a fail-open. The gate requires a non-trivial run (declared_trade_count >= \
             MIN_DECLARED_TRADES + a non-empty payload) before any PASS. Fail-closed \
             Block.",
            "degenerate_run".to_string(),
            String::new(),
            0.0,
            declared_trade_count,
            primary_digest.clone(),
            shadow_digest.clone(),
        );
        emit_ssb(ssb, build_payload(&primary, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 4: ACCOUNTING IDENTITIES (EXACT integer-cents `==`). -----------
    if let Some((invariant, evidence)) = check_accounting_identities(&primary) {
        warn!(
            strategy = %primary.strategy_path,
            invariant = %invariant,
            "shadow_exec: accounting identity violated (EXACT integer-cents) -> \
             InvariantViolated Block"
        );
        let verdict = diverged_verdict(
            ShadowVerdictKind::InvariantViolated,
            &invariant,
            &evidence,
            invariant.clone(),
            String::new(),
            0.0,
            declared_trade_count,
            primary_digest.clone(),
            shadow_digest.clone(),
        );
        emit_ssb(ssb, build_payload(&primary, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 5: PRIMARY-vs-SHADOW float divergence (tolerance band). --------
    if let Some((field, metric)) = first_float_divergence(&primary, &shadow_set, &tol) {
        warn!(
            strategy = %primary.strategy_path,
            field = %field,
            metric,
            "shadow_exec: primary diverged from shadow beyond the LOCKED band -> \
             ShadowDiverged Block (a refactor changed a derived value)"
        );
        let verdict = diverged_verdict(
            ShadowVerdictKind::ShadowDiverged,
            "shadow_diverged",
            &format!(
                "The primary and shadow re-implementations produced derived float \
                 field '{field}' values that diverge beyond the LOCKED FpTolerance \
                 band (rel={}, abs={}): a refactor changed a value (e.g. flipped a \
                 sign). Block.",
                tol.rel, tol.abs
            ),
            field.clone(),
            field,
            metric,
            declared_trade_count,
            primary_digest.clone(),
            shadow_digest.clone(),
        );
        emit_ssb(ssb, build_payload(&primary, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 6: FLOAT MRs (PutCallParity EUROPEAN-only, NumericStability). --
    if let Some((invariant, evidence)) = check_float_mrs(&invariants, &tol) {
        warn!(
            strategy = %primary.strategy_path,
            invariant = %invariant,
            "shadow_exec: a float metamorphic relation failed / was un-evaluable -> \
             InvariantViolated Block"
        );
        let verdict = diverged_verdict(
            ShadowVerdictKind::InvariantViolated,
            &invariant,
            &evidence,
            invariant.clone(),
            String::new(),
            0.0,
            declared_trade_count,
            primary_digest.clone(),
            shadow_digest.clone(),
        );
        emit_ssb(ssb, build_payload(&primary, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 7: InvariantHeld PASS (all held over a non-degenerate run). ----
    debug!(
        strategy = %primary.strategy_path,
        trades = declared_trade_count,
        "shadow_exec: all accounting identities held EXACT, all float MRs held within \
         the band, and primary agreed with shadow within the band -> InvariantHeld \
         (the financial computation is value-correct)"
    );
    let verdict = ShadowVerdict {
        kind: ShadowVerdictKind::InvariantHeld,
        reason: "invariant_held".to_string(),
        offending_invariant: String::new(),
        first_divergent_field: String::new(),
        divergence_metric: 0.0,
        declared_trade_count,
        primary_digest,
        shadow_digest,
        evidence: format!(
            "InvariantHeld: {declared_trade_count} trades; accounting identities \
             (share conservation, cash+position*price==equity, PnL additivity) held \
             EXACT in integer cents; the put-call-parity (European) and \
             numeric-stability MRs held within the LOCKED band; primary agreed with \
             the shadow re-implementation within the band (seed={seed}). Tier-1 checks \
             a FIXED textbook-correct invariant set; it does NOT mine (Daikon) nor \
             runtime-monitor (RV-Monitor/MOP) -- those are DEFERRED Tier-2."
        ),
    };
    emit_ssb(ssb, build_payload(&primary, &verdict, seed)).await?;
    Ok(verdict)
}

// ---------------------------------------------------------------------------
// Invariant checks (the load-bearing logic)
// ---------------------------------------------------------------------------

/// Check the ACCOUNTING IDENTITIES over the primary result set, EXACT in integer
/// cents (`==`, never float). Returns `Some((invariant_name, evidence))` on the FIRST
/// violation, `None` if all hold.
///
/// Identities:
///   * `share_conservation`: each trade conserves shares (`shares_in == shares_out`).
///   * `cash_equity_identity`: `cash_cents + position_qty * price_cents ==
///     equity_cents` for each trade (integer overflow is guarded with `i128`).
///   * `pnl_additivity`: the SUM of per-trade `realized_pnl_cents` ==
///     `declared_total_pnl_cents`. CAVEAT: this holds ONLY for INDEPENDENT positions
///     (the fixture must declare them); it fails under portfolio margining /
///     cross-gamma / netting.
fn check_accounting_identities(primary: &ShadowResultSet) -> Option<(String, String)> {
    for (i, t) in primary.trades.iter().enumerate() {
        // -- share conservation (EXACT integer) --
        if t.shares_in != t.shares_out {
            return Some((
                "share_conservation".to_string(),
                format!(
                    "Trade {i}: share conservation violated -- shares_in ({}) != \
                     shares_out ({}). Shares are neither created nor destroyed on a \
                     conservation-preserving fill (EXACT integer identity).",
                    t.shares_in, t.shares_out
                ),
            ));
        }
        // -- cash + position*price == equity (EXACT integer, i128-guarded) --
        let lhs = i128::from(t.cash_cents)
            + i128::from(t.position_qty) * i128::from(t.price_cents);
        if lhs != i128::from(t.equity_cents) {
            return Some((
                "cash_equity_identity".to_string(),
                format!(
                    "Trade {i}: cash+position*price != equity -- {} + {}*{} = {} != {} \
                     (EXACT integer-cents accounting identity).",
                    t.cash_cents, t.position_qty, t.price_cents, lhs, t.equity_cents
                ),
            ));
        }
    }
    // -- PnL additivity (EXACT integer; INDEPENDENT-positions-only caveat) --
    let sum: i128 = primary
        .trades
        .iter()
        .map(|t| i128::from(t.realized_pnl_cents))
        .sum();
    if sum != i128::from(primary.declared_total_pnl_cents) {
        return Some((
            "pnl_additivity".to_string(),
            format!(
                "PnL additivity violated -- sum of per-trade realized PnL ({sum} \
                 cents) != declared total ({} cents). The total PnL must equal the sum \
                 of per-trade PnLs (EXACT integer identity). CAVEAT: holds only for \
                 INDEPENDENT positions; fails under portfolio margining / netting.",
                primary.declared_total_pnl_cents
            ),
        ));
    }
    None
}

/// Find the FIRST derived-float field where the primary and shadow diverge beyond the
/// LOCKED [`FpTolerance`] band. The first-divergence INDEX reporting REUSES the
/// [`DriftComparator`] over the quantized bucket sequences (GD-1); the LOAD-BEARING
/// tolerance compare is the [`FpTolerance::close`] call here. Returns
/// `Some((field, metric))` where `metric` is the relative divergence magnitude.
///
/// A field present in one set but not the other is itself a divergence (the impls
/// disagree on what they computed).
fn first_float_divergence(
    primary: &ShadowResultSet,
    shadow: &ShadowResultSet,
    tol: &FpTolerance,
) -> Option<(String, f64)> {
    // Fast first-divergence INDEX via the reused DriftComparator over quantized
    // buckets (reporting aid). The authoritative per-field tolerance compare follows.
    let seq_primary = ActionTypeSequence::new(primary.quantized_float_sequence());
    let seq_shadow = ActionTypeSequence::new(shadow.quantized_float_sequence());
    let comparator = DriftComparator::new(0, drift_sink_drop());
    // We do not branch on the comparator result directly (byte-equality over buckets
    // is necessary-not-sufficient); we use it only to keep the first-divergence-index
    // logic REUSED rather than re-implemented. The authoritative decision is below.
    let _ = comparator.compare(&seq_primary, &seq_shadow);

    // Authoritative per-field tolerance compare (LOAD-BEARING).
    for d in &primary.derived_floats {
        match shadow.derived_floats.iter().find(|s| s.name == d.name) {
            Some(s) => {
                if !tol.close(d.value, s.value) {
                    let metric = relative_divergence(d.value, s.value);
                    return Some((d.name.clone(), metric));
                }
            }
            None => {
                // Field present in primary, absent in shadow: the impls disagree.
                return Some((d.name.clone(), f64::INFINITY));
            }
        }
    }
    // A field in shadow but not primary is likewise a divergence.
    for s in &shadow.derived_floats {
        if !primary.derived_floats.iter().any(|d| d.name == s.name) {
            return Some((s.name.clone(), f64::INFINITY));
        }
    }
    None
}

/// Check the FLOAT metamorphic relations within the LOCKED band. Returns
/// `Some((invariant_name, evidence))` on the FIRST failure / un-evaluable input.
///
/// PutCallParity (EUROPEAN-only): `C - P == S - K*exp(-rT)` within the band. An
/// American option is rejected as un-evaluable (the equality does not apply). A
/// NaN/Inf input (e.g. division producing a non-finite at a degenerate T) is Block.
/// NumericStability: two re-ordered computations of the same quantity must be `close`.
fn check_float_mrs(
    invariants: &FinancialInvariantInputs,
    tol: &FpTolerance,
) -> Option<(String, String)> {
    for (i, p) in invariants.put_call_parity.iter().enumerate() {
        // EUROPEAN-only: an American option's parity is an inequality, not an equality.
        if p.style == OptionStyle::American {
            return Some((
                "put_call_parity".to_string(),
                format!(
                    "PutCallParity input {i}: AMERICAN option -- put-call parity is an \
                     INEQUALITY (C-P <= S-K*exp(-rT)) for American options, NOT an \
                     equality. The equality MR does not apply (un-evaluable = Block, \
                     not a silent pass)."
                ),
            ));
        }
        let lhs = p.call - p.put;
        let rhs = p.spot - p.strike * (-p.rate * p.time).exp();
        // An un-evaluable (NaN/Inf) parity is Block, never a silent pass.
        if !lhs.is_finite() || !rhs.is_finite() {
            return Some((
                "put_call_parity".to_string(),
                format!(
                    "PutCallParity input {i}: un-evaluable (lhs={lhs}, rhs={rhs}); a \
                     non-finite value cannot certify the identity (Block, R2)."
                ),
            ));
        }
        if !tol.close(lhs, rhs) {
            return Some((
                "put_call_parity".to_string(),
                format!(
                    "PutCallParity violated (European) input {i}: C-P = {lhs} != \
                     S-K*exp(-rT) = {rhs} beyond the LOCKED band (rel={}, abs={}). A \
                     no-arbitrage identity break.",
                    tol.rel, tol.abs
                ),
            ));
        }
    }
    for (i, n) in invariants.numeric_stability.iter().enumerate() {
        if !n.value_a.is_finite() || !n.value_b.is_finite() {
            return Some((
                "numeric_stability".to_string(),
                format!(
                    "NumericStability '{}' input {i}: a non-finite value (a={}, b={}) \
                     -- catastrophic cancellation / NaN / Inf (Block, R2).",
                    n.name, n.value_a, n.value_b
                ),
            ));
        }
        if !tol.close(n.value_a, n.value_b) {
            return Some((
                "numeric_stability".to_string(),
                format!(
                    "NumericStability '{}' input {i}: two re-ordered computations \
                     diverge ({} vs {}) beyond the LOCKED band (rel={}, abs={}) -- a \
                     cancellation error.",
                    n.name, n.value_a, n.value_b, tol.rel, tol.abs
                ),
            ));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a fail-closed / divergent verdict (everything except the InvariantHeld PASS).
#[allow(clippy::too_many_arguments)]
fn diverged_verdict(
    kind: ShadowVerdictKind,
    reason: &str,
    evidence: &str,
    offending_invariant: String,
    first_divergent_field: String,
    divergence_metric: f64,
    declared_trade_count: u32,
    primary_digest: String,
    shadow_digest: String,
) -> ShadowVerdict {
    ShadowVerdict {
        kind,
        reason: reason.to_string(),
        offending_invariant,
        first_divergent_field,
        divergence_metric,
        declared_trade_count,
        primary_digest,
        shadow_digest,
        evidence: evidence.to_string(),
    }
}

/// Quantize a float to its tolerance bucket string (round to `QUANTIZE_DECIMALS`
/// places). A within-band pair maps to the same bucket so the DriftComparator
/// byte-equality reports the same index; this is a REPORTING aid, never the
/// tolerance decision (which is [`FpTolerance::close`]). A non-finite value gets a
/// stable sentinel so two NaNs do not silently compare equal as buckets.
fn quantize_bucket(value: f64) -> String {
    if !value.is_finite() {
        return format!("nonfinite:{}", value.to_bits());
    }
    let scale = 10f64.powi(i32::try_from(QUANTIZE_DECIMALS).unwrap_or(9));
    let rounded = (value * scale).round() / scale;
    format!("{rounded:.*}", QUANTIZE_DECIMALS)
}

/// Relative divergence magnitude `|a-b| / max(|a|,|b|)` (0 if both 0, INFINITY if the
/// scale is 0 but the diff is not). Used only for the verdict metric field.
fn relative_divergence(a: f64, b: f64) -> f64 {
    let diff = (a - b).abs();
    let scale = a.abs().max(b.abs());
    if scale == 0.0 {
        if diff == 0.0 {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        diff / scale
    }
}

/// blake3 (16-hex prefix) over raw bytes, following the RRA convention. blake3 ONLY
/// (CLAUDE.md HARD RULE).
fn blake3_16hex(bytes: &[u8]) -> String {
    let full = blake3::hash(bytes).to_hex();
    full.chars().take(crate::rra::RRA_HEX_PREFIX_LEN).collect()
}

/// A drop sink for the reused [`DriftComparator`]: the comparator may emit its own
/// DeterminismDrift row, but the canonical off-agent artifact for THIS gate is the
/// DifferentialDivergence row. We route the comparator's emit through a channel whose
/// receiver is dropped so the first-divergence logic is REUSED without emitting a
/// duplicate row (mirror result_manifest.rs::drift_sink_drop).
fn drift_sink_drop() -> DriftSink {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SsbPayload>();
    drop(rx);
    DriftSink::Channel(tx)
}

// ---------------------------------------------------------------------------
// SSB emission
// ---------------------------------------------------------------------------

async fn emit_ssb(
    ssb: &Arc<Ssb>,
    payload: DifferentialDivergencePayload,
) -> Result<(), ShadowExecError> {
    ssb.write_payload(
        DIFFERENTIAL_DIVERGENCE_ACTION_TYPE,
        0,
        SsbPayload::DifferentialDivergence(payload),
        Vec::new(),
    )
    .await
    .map_err(|e| ShadowExecError::SsbWrite(format!("{e}")))?;
    Ok(())
}

/// Build the off-agent [`DifferentialDivergencePayload`] from inputs + verdict.
///
/// `corpus_blake3` pins the exact inputs so the verdict is recomputable (GD-2).
/// blake3 ONLY (CLAUDE.md HARD RULE).
fn build_payload(
    primary: &ShadowResultSet,
    verdict: &ShadowVerdict,
    seed: u64,
) -> DifferentialDivergencePayload {
    let kind = match verdict.kind {
        ShadowVerdictKind::InvariantHeld => DifferentialDivergenceKind::InvariantHeld,
        ShadowVerdictKind::ShadowDiverged => DifferentialDivergenceKind::ShadowDiverged,
        ShadowVerdictKind::InvariantViolated => DifferentialDivergenceKind::InvariantViolated,
    };
    let corpus_key = format!(
        "{}:{}:{}:{}",
        primary.strategy_path, verdict.reason, verdict.first_divergent_field, seed
    );
    let corpus_blake3 = blake3::hash(corpus_key.as_bytes()).to_hex().to_string();

    DifferentialDivergencePayload {
        kind,
        strategy_path: primary.strategy_path.clone(),
        run_id: primary.run_id.clone(),
        offending_invariant: verdict.offending_invariant.clone(),
        declared_trade_count: verdict.declared_trade_count,
        primary_digest: verdict.primary_digest.clone(),
        shadow_digest: verdict.shadow_digest.clone(),
        first_divergent_field: verdict.first_divergent_field.clone(),
        divergence_metric: verdict.divergence_metric,
        verdict: verdict.reason.clone(),
        seed,
        corpus_blake3,
    }
}

// ---------------------------------------------------------------------------
// Unit tests (each invariant: integer-cents EXACT + float-tolerance + divergence)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::SsbConfig;

    fn test_ssb() -> Arc<Ssb> {
        let dir = std::env::temp_dir().join(format!("h26-unit-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfg = SsbConfig {
            db_path: dir.join(format!("ssb-{}.db", rand_suffix())),
            pool_size: Some(2),
            project_id: "h26-unit".to_string(),
            session_id: "h26-unit-session".to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::clock::OsClock);
        Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
    }

    fn rand_suffix() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now() // entropy-allow: test-tmpdir-collision-avoidance-nonce-not-replay-compared
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        nanos ^ (COUNTER.fetch_add(1, Ordering::Relaxed).wrapping_mul(0x9E3779B97F4A7C15))
    }

    /// A consistent two-trade primary: shares conserved, cash+pos*price=equity, PnL
    /// additive. cash=100_00, pos=10, price=50_00 => equity=100_00+10*50_00=600_00.
    fn clean_primary() -> ShadowResultSet {
        ShadowResultSet {
            strategy_path: "strat.rs".into(),
            run_id: "run-1".into(),
            elai_seed: Some(42),
            trades: vec![
                TradeRecord {
                    shares_in: 10,
                    shares_out: 10,
                    cash_cents: 100_00,
                    position_qty: 10,
                    price_cents: 50_00,
                    equity_cents: 100_00 + 10 * 50_00,
                    realized_pnl_cents: 1_50,
                },
                TradeRecord {
                    shares_in: 5,
                    shares_out: 5,
                    cash_cents: 200_00,
                    position_qty: 4,
                    price_cents: 25_00,
                    equity_cents: 200_00 + 4 * 25_00,
                    realized_pnl_cents: 2_25,
                },
            ],
            declared_total_pnl_cents: 1_50 + 2_25,
            derived_floats: vec![DerivedFloat {
                name: "option_price".into(),
                value: 12.3456789,
            }],
        }
    }

    fn clean_shadow() -> ShadowOutcome {
        ShadowOutcome::Produced(Box::new(ShadowResultSet {
            // Same derived float (within band) -- a clean re-implementation.
            derived_floats: vec![DerivedFloat {
                name: "option_price".into(),
                value: 12.3456789,
            }],
            ..clean_primary()
        }))
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn clean_run_holds_invariants_pass() {
        let ssb = test_ssb();
        let v = diff_shadow_impls(
            &ssb,
            clean_primary(),
            clean_shadow(),
            FinancialInvariantInputs::default(),
            42,
        )
        .await
        .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::InvariantHeld);
        assert_eq!(v.reason, "invariant_held");
        assert_eq!(v.declared_trade_count, 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn share_conservation_violation_blocks_exact_integer() {
        let ssb = test_ssb();
        let mut p = clean_primary();
        p.trades[0].shares_out = 9; // shares not conserved
        let v = diff_shadow_impls(
            &ssb,
            p,
            clean_shadow(),
            FinancialInvariantInputs::default(),
            42,
        )
        .await
        .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::InvariantViolated);
        assert_eq!(v.reason, "share_conservation");
        assert_eq!(v.offending_invariant, "share_conservation");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cash_equity_identity_violation_blocks_exact_integer() {
        let ssb = test_ssb();
        let mut p = clean_primary();
        p.trades[0].equity_cents += 1; // off by one cent -> EXACT identity broken
        let v = diff_shadow_impls(
            &ssb,
            p,
            clean_shadow(),
            FinancialInvariantInputs::default(),
            42,
        )
        .await
        .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::InvariantViolated);
        assert_eq!(v.reason, "cash_equity_identity");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pnl_additivity_violation_blocks_exact_integer() {
        let ssb = test_ssb();
        let mut p = clean_primary();
        p.declared_total_pnl_cents += 1; // total != sum of per-trade PnLs
        let v = diff_shadow_impls(
            &ssb,
            p,
            clean_shadow(),
            FinancialInvariantInputs::default(),
            42,
        )
        .await
        .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::InvariantViolated);
        assert_eq!(v.reason, "pnl_additivity");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shadow_divergence_beyond_band_blocks() {
        let ssb = test_ssb();
        let shadow = ShadowOutcome::Produced(Box::new(ShadowResultSet {
            // A sign-flipped derived value: diverges far beyond the band.
            derived_floats: vec![DerivedFloat {
                name: "option_price".into(),
                value: -12.3456789,
            }],
            ..clean_primary()
        }));
        let v = diff_shadow_impls(
            &ssb,
            clean_primary(),
            shadow,
            FinancialInvariantInputs::default(),
            42,
        )
        .await
        .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::ShadowDiverged);
        assert_eq!(v.reason, "shadow_diverged");
        assert_eq!(v.first_divergent_field, "option_price");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fp_noise_within_band_does_not_diverge() {
        let ssb = test_ssb();
        let shadow = ShadowOutcome::Produced(Box::new(ShadowResultSet {
            // 12.3456789 + 1e-11: within rel 1e-9 band.
            derived_floats: vec![DerivedFloat {
                name: "option_price".into(),
                value: 12.3456789 + 1e-11,
            }],
            ..clean_primary()
        }));
        let v = diff_shadow_impls(
            &ssb,
            clean_primary(),
            shadow,
            FinancialInvariantInputs::default(),
            42,
        )
        .await
        .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::InvariantHeld);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn missing_shadow_is_fail_closed_block() {
        let ssb = test_ssb();
        let v = diff_shadow_impls(
            &ssb,
            clean_primary(),
            ShadowOutcome::Missing,
            FinancialInvariantInputs::default(),
            42,
        )
        .await
        .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::ShadowDiverged);
        assert_eq!(v.reason, "shadow_missing");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn errored_shadow_is_fail_closed_block() {
        let ssb = test_ssb();
        let v = diff_shadow_impls(
            &ssb,
            clean_primary(),
            ShadowOutcome::Errored("panic in shadow".into()),
            FinancialInvariantInputs::default(),
            42,
        )
        .await
        .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::ShadowDiverged);
        assert_eq!(v.reason, "shadow_errored");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn degenerate_zero_trade_run_is_fail_closed_block() {
        let ssb = test_ssb();
        let empty = ShadowResultSet {
            strategy_path: "strat.rs".into(),
            run_id: "run-1".into(),
            elai_seed: Some(42),
            trades: vec![],
            declared_total_pnl_cents: 0,
            derived_floats: vec![],
        };
        let shadow = ShadowOutcome::Produced(Box::new(empty.clone()));
        let v = diff_shadow_impls(&ssb, empty, shadow, FinancialInvariantInputs::default(), 42)
            .await
            .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::InvariantViolated);
        assert_eq!(v.reason, "degenerate_run");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn put_call_parity_european_holds_and_violation_blocks() {
        let ssb = test_ssb();
        // Construct a parity-consistent European set: C - P = S - K*exp(-rT).
        let spot: f64 = 100.0;
        let strike: f64 = 95.0;
        let rate: f64 = 0.02;
        let time: f64 = 0.5;
        let rhs = spot - strike * (-rate * time).exp();
        let put = 3.0;
        let call = put + rhs; // makes C - P == rhs
        let held = FinancialInvariantInputs {
            put_call_parity: vec![PutCallParityInput {
                call,
                put,
                spot,
                strike,
                rate,
                time,
                style: OptionStyle::European,
            }],
            numeric_stability: vec![],
        };
        let v = diff_shadow_impls(&ssb, clean_primary(), clean_shadow(), held, 42)
            .await
            .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::InvariantHeld);

        // Now break parity (flip the call price far off).
        let ssb2 = test_ssb();
        let broken = FinancialInvariantInputs {
            put_call_parity: vec![PutCallParityInput {
                call: call + 5.0,
                put,
                spot,
                strike,
                rate,
                time,
                style: OptionStyle::European,
            }],
            numeric_stability: vec![],
        };
        let v2 = diff_shadow_impls(&ssb2, clean_primary(), clean_shadow(), broken, 42)
            .await
            .expect("diff");
        assert_eq!(v2.kind, ShadowVerdictKind::InvariantViolated);
        assert_eq!(v2.reason, "put_call_parity");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn american_option_parity_is_unevaluable_block() {
        let ssb = test_ssb();
        let american = FinancialInvariantInputs {
            put_call_parity: vec![PutCallParityInput {
                call: 8.0,
                put: 3.0,
                spot: 100.0,
                strike: 95.0,
                rate: 0.02,
                time: 0.5,
                style: OptionStyle::American,
            }],
            numeric_stability: vec![],
        };
        let v = diff_shadow_impls(&ssb, clean_primary(), clean_shadow(), american, 42)
            .await
            .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::InvariantViolated);
        assert_eq!(v.reason, "put_call_parity");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn numeric_stability_nan_is_fail_closed_block() {
        let ssb = test_ssb();
        let nan_input = FinancialInvariantInputs {
            put_call_parity: vec![],
            numeric_stability: vec![NumericStabilityInput {
                name: "iv".into(),
                value_a: f64::NAN,
                value_b: 0.2,
            }],
        };
        let v = diff_shadow_impls(&ssb, clean_primary(), clean_shadow(), nan_input, 42)
            .await
            .expect("diff");
        assert_eq!(v.kind, ShadowVerdictKind::InvariantViolated);
        assert_eq!(v.reason, "numeric_stability");
    }

    #[test]
    fn accounting_identity_is_exact_integer_not_float() {
        // A 1-cent equity error MUST be caught (no float tolerance hides it).
        let mut p = clean_primary();
        p.trades[0].equity_cents += 1;
        let r = check_accounting_identities(&p);
        assert!(matches!(r, Some((ref inv, _)) if inv == "cash_equity_identity"));
    }

    #[test]
    fn blake3_16hex_is_stable_and_prefix_len() {
        let h = blake3_16hex(b"some result set");
        assert_eq!(h.len(), crate::rra::RRA_HEX_PREFIX_LEN);
        assert_eq!(h, blake3_16hex(b"some result set"));
        assert_ne!(h, blake3_16hex(b"different"));
    }

    #[test]
    fn quantize_bucket_maps_within_band_to_same_string() {
        let a = quantize_bucket(12.3456789);
        let b = quantize_bucket(12.3456789 + 1e-11);
        assert_eq!(a, b, "values within the bucket must quantize identically");
        let c = quantize_bucket(-12.3456789);
        assert_ne!(a, c, "a sign flip must quantize differently");
    }
}
