//! Trust-score temporal decay. Per-fact-type half-lives from the E-5
//! plan Task 9. Decay model is monotone exponential:
//!
//! ```text
//! trust(t) = trust_0 * exp(-decay_rate_per_day * elapsed_days)
//! ```
//!
//! The scheduler that runs [`decay_tick`] on a tokio interval is wired
//! at the orchestrator boundary; this module owns the pure math + batch
//! helper so it can be unit-tested without sqlite ceremony.

use crate::error::MemoryError;
use crate::fact::FactType;
use crate::store::FactStore;

/// Default decay rate (per day) per fact-type per E-5 plan Task 9.
///
/// Half-life math (recall `half_life_days = ln(2) / decay_rate_per_day`,
/// `ln(2) ~ 0.693`):
///
/// - voice_style: 0.0001/day  -> ~6932 days (~19 yr)
/// - expertise:   0.0001/day  -> ~19 yr
/// - opinion:     0.0005/day  -> ~3.8 yr
/// - mistake:     0.005/day   -> ~138 days
/// - preference:  0.001/day   -> ~1.9 yr
/// - session_pattern: 0.01/day -> ~69 days
/// - risk_tol:    0.0005/day  -> ~3.8 yr
/// - domain_knowledge: 0.0008/day -> ~2.4 yr
pub fn default_decay_rate(fact_type: FactType) -> f32 {
    match fact_type {
        FactType::VoiceStyle => 0.0001,
        FactType::Expertise => 0.0001,
        FactType::Opinion => 0.0005,
        FactType::Mistake => 0.005,
        FactType::Preference => 0.001,
        FactType::SessionPattern => 0.01,
        FactType::RiskTol => 0.0005,
        FactType::DomainKnowledge => 0.0008,
    }
}

/// Pure math: returns the decayed trust score at `now_ms` for a row last
/// observed at `last_observed_at_ms` with current `trust_score` and
/// `decay_rate_per_day`. Clamped to `[0.0, 1.0]`.
pub fn decayed_trust(
    trust_score: f32,
    decay_rate_per_day: f32,
    last_observed_at_ms: u128,
    now_ms: u128,
) -> f32 {
    if now_ms <= last_observed_at_ms {
        // Future-dated row or identical timestamps — no decay.
        return trust_score.clamp(0.0, 1.0);
    }
    if decay_rate_per_day <= 0.0 {
        return trust_score.clamp(0.0, 1.0);
    }
    let elapsed_ms = now_ms - last_observed_at_ms;
    // Convert to days. 86_400_000 ms in one day.
    let elapsed_days = (elapsed_ms as f64) / 86_400_000.0;
    let factor = (-(decay_rate_per_day as f64) * elapsed_days).exp();
    let decayed = (trust_score as f64) * factor;
    decayed.max(0.0).min(1.0) as f32
}

/// Aggregate stats emitted by [`decay_tick`] — exposed for the SSB
/// payload that the scheduler row carries when it runs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DecayTickStats {
    /// Number of currently-valid rows examined.
    pub rows_examined: usize,
    /// Number of rows whose persisted `trust_score` actually moved (delta
    /// > 1e-6 in absolute value).
    pub rows_updated: usize,
    /// Mean trust score AFTER decay across the examined rows. `0.0` when
    /// `rows_examined == 0`.
    pub mean_trust_score: f32,
}

/// Apply [`decayed_trust`] to every currently-valid row in `store`. Per
/// E-5 plan Task 9 — the scheduler runs this on a default 1h cadence.
/// `now_ms` is the wall-clock time supplied by the caller's
/// `ClockProvider`.
///
/// Does NOT update `last_observed_at_ms` — decay is a passive process
/// that does not count as an observation (otherwise the curve would never
/// progress).
pub fn decay_tick(store: &FactStore, now_ms: u128) -> Result<DecayTickStats, MemoryError> {
    let rows = store.iter_current()?;
    let rows_examined = rows.len();
    let mut rows_updated = 0usize;
    let mut sum_trust: f64 = 0.0;
    for row in &rows {
        let new_trust = decayed_trust(
            row.trust_score,
            row.decay_rate_per_day,
            row.last_observed_at_ms,
            now_ms,
        );
        if (new_trust - row.trust_score).abs() > 1e-6 {
            store.update_trust(&row.fact_id, new_trust, row.last_observed_at_ms)?;
            rows_updated += 1;
            sum_trust += new_trust as f64;
        } else {
            sum_trust += row.trust_score as f64;
        }
    }
    let mean_trust_score = if rows_examined == 0 {
        0.0
    } else {
        (sum_trust / rows_examined as f64) as f32
    };
    Ok(DecayTickStats {
        rows_examined,
        rows_updated,
        mean_trust_score,
    })
}
