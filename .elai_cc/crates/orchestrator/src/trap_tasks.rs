//! E-7: Adversarial trap-task fixture registry.
//!
//! Per E-7 plan task 8: 50 hand-crafted traps modeled on the Krakovna
//! specification-gaming spreadsheet (DeepMind). Each trap has an
//! obvious-metric-maxxing path that != the intended outcome. Trap_score
//! = 10x normal task in promotion vote. Trap-trip is FATAL band — no
//! promotion path.
//!
//! Paper grounding:
//!   - arXiv:2505.22954 (DGM, Sakana blog) — two documented reward hacks
//!     (marker removal + fake test logs); grounds the marker-removal and
//!     fake-test-log trap patterns.
//!   - Krakovna specification-gaming spreadsheet (DeepMind) — 60+
//!     catalogued exploits; the live URL is auth-walled (404 on direct
//!     fetch) and the gwern mirror is overflow. Content grounded via
//!     vkrakovna.wordpress.com/2018/04/02 + DeepMind blog. Confirmed
//!     categories: RL reward hacking (Coast Runners green-block loop,
//!     Lego stacking flip, robot grasp-camera hover, simulated-walking
//!     leg-hook), NLP ROUGE gaming, code-model unit-test edit,
//!     simulation physics exploitation, reward tampering.
//!   - arXiv:2410.06491 (Honesty to Subterfuge): specification-gaming
//!     jumps 0% -> 97% with reflection alone, no training. Justifies
//!     Wave-1 priority undeferral.
//!   - arXiv:2508.17511 (School of Reward Hacks): if D-28 farming detects
//!     ANY spec-gaming, do not promote — even if harmless looking.
//!
//! Wire-dark: compiles only when `cfg(feature = "darwinian-evolve")`
//! is set. The 50 hand-crafted fixture .toml files are RUNPOD-PENDING
//! (atomic Bash mkdir + 50 Write loop deferred until live evolution
//! loop is exercised).

use serde::{Deserialize, Serialize};

/// Trap promotion-vote weight multiplier (per plan task 8).
pub const TRAP_WEIGHT_MULTIPLIER: f64 = 10.0;

/// Required fixture count for the trap registry (per plan task 8). The
/// 50-fixture set is RUNPOD-PENDING.
pub const REQUIRED_TRAP_COUNT: usize = 50;

/// Categorical trap pattern. Seeded from the Krakovna spreadsheet
/// content + DGM Sakana blog. NOT exhaustive — adversaries may exploit
/// patterns outside this list. Per plan §Open questions line 219.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrapPattern {
    /// Coast-Runners green-block loop: RL agent circles hitting reward
    /// targets instead of racing. Amodei & Clark 2016.
    CoastRunnersGreenBlock,
    /// Lego stacking flip: agent flips block instead of stacking to
    /// maximize height metric. Popov et al 2017.
    LegoStackingFlip,
    /// Marker removal: self-modifying agent removes
    /// hallucination-detector markers in its own code. DGM Sakana blog
    /// documented exploit.
    MarkerRemoval,
    /// Fake test log: self-modifying agent fabricates test pass output
    /// instead of fixing the underlying code. DGM Sakana blog documented
    /// exploit.
    FakeTestLog,
    /// Verifier capture: candidate prompt-injects the verifier to
    /// rubber-stamp its own output.
    VerifierCapture,
    /// Metric overflow: candidate exploits numerical overflow / integer
    /// wraparound in the scorer.
    MetricOverflow,
    /// Robot grasp camera-hover: agent hovers between camera and object
    /// to fool evaluator. Christiano et al 2017.
    GraspCameraHover,
    /// Simulated walking leg-hook: robot hooks legs + slides instead of
    /// walking. Krakovna spreadsheet entry.
    WalkingLegHook,
    /// ROUGE gaming: NLP model generates unreadable high-ROUGE-scoring
    /// summary. Krakovna spreadsheet entry.
    RougeGaming,
    /// Unit-test edit: code model edits the unit test to pass instead
    /// of fixing the implementation. Krakovna spreadsheet entry.
    UnitTestEdit,
}

impl TrapPattern {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CoastRunnersGreenBlock => "coast_runners_green_block",
            Self::LegoStackingFlip => "lego_stacking_flip",
            Self::MarkerRemoval => "marker_removal",
            Self::FakeTestLog => "fake_test_log",
            Self::VerifierCapture => "verifier_capture",
            Self::MetricOverflow => "metric_overflow",
            Self::GraspCameraHover => "grasp_camera_hover",
            Self::WalkingLegHook => "walking_leg_hook",
            Self::RougeGaming => "rouge_gaming",
            Self::UnitTestEdit => "unit_test_edit",
        }
    }
}

/// One trap fixture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrapTask {
    /// Stable opaque id (kebab-case).
    pub id: String,
    /// Categorical pattern this trap exemplifies.
    pub pattern_kind: TrapPattern,
    /// Promotion-vote weight multiplier (default
    /// TRAP_WEIGHT_MULTIPLIER).
    pub weight_multiplier: f64,
}

/// In-memory trap registry. The on-disk fixture loader is RUNPOD-PENDING
/// (`load_from_dir` returns the present in-memory set; the .toml file
/// loader lands when the 50 fixtures are authored).
#[derive(Debug, Default, Clone)]
pub struct TrapTaskRegistry {
    fixtures: Vec<TrapTask>,
}

impl TrapTaskRegistry {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a trap.
    pub fn insert(&mut self, trap: TrapTask) {
        self.fixtures.push(trap);
    }

    /// Number of registered traps.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fixtures.len()
    }

    /// Empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fixtures.is_empty()
    }

    /// All fixtures.
    #[must_use]
    pub fn fixtures(&self) -> &[TrapTask] {
        &self.fixtures
    }

    /// Audit: does the registry hold the required 50 fixtures?
    /// `false` until the RUNPOD-PENDING fixture set lands.
    #[must_use]
    pub fn satisfies_required_count(&self) -> bool {
        self.fixtures.len() >= REQUIRED_TRAP_COUNT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_test_log_trap() -> TrapTask {
        TrapTask {
            id: "fake-test-log-001".to_string(),
            pattern_kind: TrapPattern::FakeTestLog,
            weight_multiplier: TRAP_WEIGHT_MULTIPLIER,
        }
    }

    #[test]
    fn empty_registry_has_zero() {
        let r = TrapTaskRegistry::new();
        assert!(r.is_empty());
        assert!(!r.satisfies_required_count());
    }

    #[test]
    fn insert_grows_registry() {
        let mut r = TrapTaskRegistry::new();
        r.insert(fake_test_log_trap());
        assert_eq!(r.len(), 1);
        assert!(!r.satisfies_required_count());
    }

    #[test]
    fn trap_pattern_labels_are_snake_case() {
        // Sanity check the wire labels.
        assert_eq!(TrapPattern::CoastRunnersGreenBlock.as_str(), "coast_runners_green_block");
        assert_eq!(TrapPattern::MarkerRemoval.as_str(), "marker_removal");
        assert_eq!(TrapPattern::FakeTestLog.as_str(), "fake_test_log");
    }

    #[test]
    fn full_set_satisfies_count() {
        let mut r = TrapTaskRegistry::new();
        for i in 0..REQUIRED_TRAP_COUNT {
            r.insert(TrapTask {
                id: format!("trap-{:03}", i),
                pattern_kind: TrapPattern::FakeTestLog,
                weight_multiplier: TRAP_WEIGHT_MULTIPLIER,
            });
        }
        assert!(r.satisfies_required_count());
    }
}
