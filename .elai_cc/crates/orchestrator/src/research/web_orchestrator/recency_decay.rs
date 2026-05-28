//! E-48 Task 10 — recency-weighted relevance for time-sensitive queries.
//!
//! Detects temporal markers in a research question via regex; when a
//! marker is present, multiplies each source's relevance score by the
//! exponential decay `exp(-ln(2) * age_days / half_life_days)`. Default
//! half-life is 365 days (configurable via `[web_search.recency]` in
//! `elai_cc.toml`; configuration wiring is RUNPOD-PENDING but the
//! pure-compute decay surface ships here).
//!
//! Grounding:
//!   * arXiv:2509.19376 -- Solving Freshness in RAG (recency-weighted
//!     scoring).
//!   * arXiv:2601.17824 -- OwlerLite scope+freshness web retrieval.

use regex::Regex;

/// Default half-life in days for the recency decay multiplier. Per the
/// plan task spec; a source one year old (365 days) gets multiplied by
/// 0.5, a source three years old by 0.125, etc.
pub const DEFAULT_HALF_LIFE_DAYS: u32 = 365;

/// Temporal-marker regex. Matches the plan task spec terms plus year
/// tokens 2024..2030. Case-insensitive.
pub const TEMPORAL_MARKER_PATTERN: &str =
    r"(?i)\b(latest|current|recent|today|this year|202[4-9]|2030)\b";

/// Detect a temporal marker in the question; return the matched token
/// when found.
#[must_use]
pub fn detect_temporal_marker(question: &str) -> Option<String> {
    let re = Regex::new(TEMPORAL_MARKER_PATTERN).expect("TEMPORAL_MARKER_PATTERN compiles");
    re.find(question).map(|m| m.as_str().to_ascii_lowercase())
}

/// Compute the recency decay multiplier for a source `age_days` old
/// given the configured `half_life_days`.
///
/// Formula: `exp(-ln(2) * age_days / half_life_days)`.
///
/// Properties:
///   * `age_days == 0` -> 1.0.
///   * `age_days == half_life_days` -> 0.5.
///   * `age_days == 2 * half_life_days` -> 0.25.
///   * monotonically decreasing in `age_days`.
///   * `half_life_days == 0` falls back to [`DEFAULT_HALF_LIFE_DAYS`]
///     to avoid divide-by-zero.
#[must_use]
pub fn decay_multiplier(age_days: u32, half_life_days: u32) -> f64 {
    let half = if half_life_days == 0 {
        DEFAULT_HALF_LIFE_DAYS
    } else {
        half_life_days
    };
    let ratio = (age_days as f64) / (half as f64);
    (-std::f64::consts::LN_2 * ratio).exp()
}

/// Apply the recency decay multiplier to a slice of `(score, age_days)`
/// pairs. Returns the decayed scores in input order. Mean age across
/// the input slice is also returned for the SSB telemetry row.
///
/// When `n_sources == 0`, returns `(Vec::new(), 0.0)` -- caller should
/// short-circuit the producer-side emit fn (no row written).
#[must_use]
pub fn apply_recency_decay(
    sources: &[(f64, u32)],
    half_life_days: u32,
) -> (Vec<f64>, f64) {
    if sources.is_empty() {
        return (Vec::new(), 0.0);
    }
    let decayed: Vec<f64> = sources
        .iter()
        .map(|(score, age_days)| score * decay_multiplier(*age_days, half_life_days))
        .collect();
    let total_age: u64 = sources.iter().map(|(_, a)| u64::from(*a)).sum();
    let mean_age = (total_age as f64) / (sources.len() as f64);
    (decayed, mean_age)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_temporal_marker_finds_latest() {
        assert_eq!(
            detect_temporal_marker("which python web framework is fastest in 2026?")
                .as_deref(),
            Some("2026")
        );
    }

    #[test]
    fn detect_temporal_marker_finds_recent() {
        assert_eq!(
            detect_temporal_marker("recent benchmarks for FastAPI").as_deref(),
            Some("recent")
        );
    }

    #[test]
    fn detect_temporal_marker_misses_evergreen_question() {
        assert!(detect_temporal_marker("how does HTTP/2 work").is_none());
    }

    #[test]
    fn decay_multiplier_at_zero_age_returns_one() {
        let m = decay_multiplier(0, 365);
        assert!((m - 1.0).abs() < 1e-9);
    }

    #[test]
    fn decay_multiplier_at_half_life_returns_half() {
        let m = decay_multiplier(365, 365);
        assert!((m - 0.5).abs() < 1e-6, "got {m}");
    }

    #[test]
    fn decay_multiplier_at_two_half_lives_returns_quarter() {
        let m = decay_multiplier(730, 365);
        assert!((m - 0.25).abs() < 1e-6, "got {m}");
    }

    #[test]
    fn decay_multiplier_zero_half_life_falls_back_to_default() {
        let m_zero = decay_multiplier(365, 0);
        let m_default = decay_multiplier(365, DEFAULT_HALF_LIFE_DAYS);
        assert!((m_zero - m_default).abs() < f64::EPSILON);
    }

    #[test]
    fn apply_recency_decay_zero_sources_returns_empty() {
        let (decayed, mean_age) = apply_recency_decay(&[], 365);
        assert!(decayed.is_empty());
        assert_eq!(mean_age, 0.0);
    }

    #[test]
    fn apply_recency_decay_monotone_on_age() {
        let sources = vec![(1.0, 30_u32), (1.0, 365), (1.0, 730)];
        let (decayed, mean_age) = apply_recency_decay(&sources, 365);
        assert_eq!(decayed.len(), 3);
        assert!(decayed[0] > decayed[1]);
        assert!(decayed[1] > decayed[2]);
        assert!((mean_age - 375.0).abs() < 0.1);
    }
}
