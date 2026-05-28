//! E-54 ConStat-style performance-delta detector (task 8).
//!
//! When ELAI's self-mod loop improves on family X by >Y over 1 week, fire a
//! detector against a reference model on the SAME held-out slice. Sudden
//! jump beyond a configurable threshold = `perf_delta_contamination_suspected`
//! SSB row.
//!
//! The ConStat paper (cited in the upstream improvement-sweep report; no
//! arXiv ID given) requires per-task reference performance. Per
//! workflow/reasoning/E-54-decisions.md DECISION-7, if reference-performance
//! computation is impractical, the module ships a SIMPLER 2-sigma deviation
//! detector that consumes the recent N=K reference rolling stats and fires
//! when the latest sample exceeds `mean + k * stddev`.
//!
//! This module ships the 2-sigma fallback as the default; live ConStat
//! reference fetch is RUNPOD-PENDING.

use serde::{Deserialize, Serialize};

/// E-54 default 2-sigma threshold for the fallback detector.
pub const DEFAULT_SIGMA_THRESHOLD: f64 = 2.0;

/// E-54 default sudden-jump pp threshold (used by ConStat path when live).
pub const DEFAULT_JUMP_THRESHOLD_PP: f64 = 20.0;

/// Rolling performance history for one (family, model) slice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerfHistory {
    /// Family label (e.g. CanaryFamily::as_str output).
    pub family_label: String,
    /// Past samples (oldest first), in `[0.0, 1.0]`.
    pub samples: Vec<f64>,
    /// The newest sample (= current measurement under inspection).
    pub current: f64,
}

/// Detection verdict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectionVerdict {
    /// `current - mean(samples)`, scaled to percentage points.
    pub delta_pp: f64,
    /// True when the current sample exceeds `mean + sigma_threshold * stddev`
    /// OR `delta_pp > jump_threshold_pp`.
    pub suspect: bool,
    /// True when the detector fell back to the 2-sigma path (= no reference
    /// model snapshot available; ConStat ungrounded).
    pub fallback_2sigma: bool,
    /// Mean of `samples` (None when `samples.is_empty()`).
    pub mean: Option<f64>,
    /// Population stddev of `samples` (None when `samples.len() < 2`).
    pub stddev: Option<f64>,
}

/// Run the 2-sigma fallback detector on a [`PerfHistory`].
///
/// The ConStat path requires a separate reference-model perf number per
/// task; until that snapshot pipeline lands, ELAI uses the rolling-window
/// 2-sigma detector. The verdict's `fallback_2sigma` flag is `true` to
/// surface the regime to the dashboard.
#[must_use]
pub fn detect(history: &PerfHistory) -> DetectionVerdict {
    if history.samples.is_empty() {
        return DetectionVerdict {
            delta_pp: 0.0,
            suspect: false,
            fallback_2sigma: true,
            mean: None,
            stddev: None,
        };
    }
    let n = history.samples.len() as f64;
    let mean = history.samples.iter().sum::<f64>() / n;
    let delta_pp = (history.current - mean) * 100.0;
    let stddev = if history.samples.len() < 2 {
        None
    } else {
        let var = history
            .samples
            .iter()
            .map(|s| (s - mean).powi(2))
            .sum::<f64>()
            / n;
        Some(var.sqrt())
    };
    let suspect = match stddev {
        Some(s) if s > 0.0 => {
            (history.current - mean).abs() > DEFAULT_SIGMA_THRESHOLD * s
                || delta_pp.abs() > DEFAULT_JUMP_THRESHOLD_PP
        }
        _ => delta_pp.abs() > DEFAULT_JUMP_THRESHOLD_PP,
    };
    DetectionVerdict {
        delta_pp,
        suspect,
        fallback_2sigma: true,
        mean: Some(mean),
        stddev,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perf_delta_detector_fires_on_sudden_jump() {
        let h = PerfHistory {
            family_label: "code_fix".to_string(),
            samples: vec![0.40, 0.42, 0.41, 0.43, 0.42],
            current: 0.75,
        };
        let v = detect(&h);
        assert!(v.suspect, "33pp jump must fire");
        assert!(v.fallback_2sigma, "default path is the 2-sigma fallback");
    }

    #[test]
    fn perf_delta_detector_quiet_on_stable_history() {
        let h = PerfHistory {
            family_label: "code_fix".to_string(),
            samples: vec![0.40, 0.41, 0.42, 0.41, 0.40],
            current: 0.42,
        };
        let v = detect(&h);
        assert!(!v.suspect, "within-noise sample must NOT fire");
    }

    #[test]
    fn empty_history_returns_quiet_fallback() {
        let h = PerfHistory {
            family_label: "code_fix".to_string(),
            samples: Vec::new(),
            current: 0.42,
        };
        let v = detect(&h);
        assert!(!v.suspect);
        assert!(v.fallback_2sigma);
    }
}
