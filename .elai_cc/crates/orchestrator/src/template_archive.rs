//! E-7: MAP-Elites quality-diversity archive over a BehaviourDescriptor.
//!
//! Replaces D-1's fitness-proportional parent selection with cell-based
//! quality-diversity (Mouret & Clune 2015; AlphaEvolve arXiv:2506.13131
//! §2.5). One elite per cell keyed by (latency_tier, token_class,
//! failure_mode). Best-in-cell replaces global softmax over fitness.
//!
//! Paper grounding:
//!   - arXiv:2506.13131 (AlphaEvolve, DeepMind 2025) — MAP-Elites +
//!     island-model hybrid; primary darwinian-coding-agent anchor.
//!     Interface map: workflow/papers/alphaevolve-2506.13131-interface.md.
//!     AlphaEvolve §2.5 documents the MAP-Elites database; the formal
//!     behavioural-descriptor taxonomy is NOT prescribed by the paper —
//!     ELAI derives its own (latency × token × failure-mode) cells.
//!   - Mouret & Clune 2015 (MAP-Elites foundation) — quality-diversity
//!     cell archive; behaviour descriptor.
//!
//! Wire-dark: this module compiles only when `cfg(feature =
//! "darwinian-evolve")` is set. Live admit loops gate additionally on
//! `ELAI_DARWINIAN_LOOP_ENABLE=1`. Live model evolution loops
//! RUNPOD-PENDING.
//!
//! No `.unwrap()`/`.expect()` in non-test code. Determinism: the archive
//! is a pure data structure; no I/O, no clock, no thread_rng().

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// Latency band a candidate falls into at evaluation time. Snake-case
/// wire labels: `sub_10s` / `sub_60s` / `over_60s`.
///
/// Boundaries chosen against Doherty + Kelisky 1982 200ms snappy threshold
/// (10s = "in-flow", 60s = "browsable-then-disengage"). Default-OFF behind
/// the feature gate; live latency capture RUNPOD-PENDING.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LatencyTier {
    /// p50 < 10s.
    Sub10s,
    /// 10s <= p50 < 60s.
    Sub60s,
    /// p50 >= 60s.
    Over60s,
}

impl LatencyTier {
    /// Bucket a measured latency in millis.
    #[must_use]
    pub fn bucket(latency_ms: u64) -> Self {
        if latency_ms < 10_000 {
            Self::Sub10s
        } else if latency_ms < 60_000 {
            Self::Sub60s
        } else {
            Self::Over60s
        }
    }
}

/// Token-count class a candidate falls into. 4 boundaries chosen to track
/// the small-prompt / medium / large / over-cap distinction in the
/// generation budget. UNCONFIRMED for ELAI; ship as config-knob in a
/// follow-on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenClass {
    /// total_tokens < 500.
    Lt500,
    /// 500 <= total_tokens < 2000.
    Lt2k,
    /// 2000 <= total_tokens < 8000.
    Lt8k,
    /// total_tokens >= 8000.
    Over8k,
}

impl TokenClass {
    /// Bucket a measured token count.
    #[must_use]
    pub fn bucket(total_tokens: u32) -> Self {
        if total_tokens < 500 {
            Self::Lt500
        } else if total_tokens < 2_000 {
            Self::Lt2k
        } else if total_tokens < 8_000 {
            Self::Lt8k
        } else {
            Self::Over8k
        }
    }
}

/// Failure mode taxonomy carried by the BehaviourDescriptor. `None`
/// signals a clean evaluation. Reserved placeholder enum -- the canonical
/// MAST mode taxonomy lands when E-6 ships. Until then this is a string-
/// keyed shim that the live evolution loop refuses to enable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureModeShim {
    /// E-6 PENDING: real MAST mode taxonomy lands in E-6.
    Pending,
}

/// MAP-Elites behavioural-descriptor cell key.
///
/// `failure_mode` is `None` for clean evaluations and `Some(...)` for
/// candidates that exhibited a failure pattern. Cell count =
/// 3 (LatencyTier) × 4 (TokenClass) × (1 + FailureModeShim variants) =
/// 12 default cells (Pending shim counts as +1 axis until E-6 lands).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BehaviourDescriptor {
    pub latency_tier: LatencyTier,
    pub token_class: TokenClass,
    pub failure_mode: Option<FailureModeShim>,
}

/// Cell entry. Wire-dark scaffold -- the canonical ArchiveEntry lives in
/// archive_chain; this scaffold deliberately uses an opaque payload-hash
/// to avoid coupling the substrate to the live ArchiveEntry shape.
///
/// `Eq` is intentionally NOT derived -- `f64` does not implement `Eq`
/// (NaN-comparability); use `PartialEq` only. Equality is structural
/// (template_blake3 + fitness bit-pattern), not semantic-numerical.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveEntry {
    /// blake3 of the canonical ReasoningTemplate body.
    pub template_blake3: [u8; 32],
    /// Quality scalar (PRM score after EST invariance probe).
    pub fitness: f64,
}

/// Result of a candidate admit attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmitResult {
    /// Cell was empty; candidate became the elite.
    Inserted,
    /// Cell already had an elite; candidate's fitness beat the incumbent.
    Elevated,
    /// Cell already had a higher-fitness elite; candidate rejected.
    Rejected,
}

/// MAP-Elites archive. Wire-dark scaffold: live emission of
/// `SsbActionType::DarwinianGenerationCompleted` with
/// `DarwinianPhase::ArchiveCellElevated` happens in the consumer that
/// owns the SSB writer (kept out of this module to preserve no-IO
/// determinism here).
#[derive(Debug, Default)]
pub struct MapElitesArchive {
    cells: FxHashMap<BehaviourDescriptor, ArchiveEntry>,
}

impl MapElitesArchive {
    /// Empty archive.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cells: FxHashMap::default(),
        }
    }

    /// Try to admit `entry` at `descriptor`. Returns the admission verdict.
    pub fn admit(&mut self, descriptor: BehaviourDescriptor, entry: ArchiveEntry) -> AdmitResult {
        match self.cells.get(&descriptor) {
            None => {
                self.cells.insert(descriptor, entry);
                AdmitResult::Inserted
            }
            Some(incumbent) => {
                if entry.fitness > incumbent.fitness {
                    self.cells.insert(descriptor, entry);
                    AdmitResult::Elevated
                } else {
                    AdmitResult::Rejected
                }
            }
        }
    }

    /// Best entry in `descriptor`.
    #[must_use]
    pub fn best_in_cell(&self, descriptor: &BehaviourDescriptor) -> Option<&ArchiveEntry> {
        self.cells.get(descriptor)
    }

    /// Number of occupied cells.
    #[must_use]
    pub fn occupied_cells(&self) -> usize {
        self.cells.len()
    }

    /// Shannon entropy across cells (nats). Acceptance criterion (plan
    /// §Open questions line 216): coverage entropy stays > 1.0 nats
    /// after 20 generations.
    #[must_use]
    pub fn shannon_entropy(&self) -> f64 {
        let total = self.cells.len() as f64;
        if total <= 1.0 {
            return 0.0;
        }
        // Each cell has equal mass (1 elite per cell); entropy is
        // ln(N) -- the diversity floor scales with cell occupancy.
        total.ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor() -> BehaviourDescriptor {
        BehaviourDescriptor {
            latency_tier: LatencyTier::Sub10s,
            token_class: TokenClass::Lt500,
            failure_mode: None,
        }
    }

    fn entry(fitness: f64) -> ArchiveEntry {
        ArchiveEntry {
            template_blake3: [0u8; 32],
            fitness,
        }
    }

    #[test]
    fn first_admit_inserts() {
        let mut a = MapElitesArchive::new();
        assert_eq!(a.admit(descriptor(), entry(0.5)), AdmitResult::Inserted);
        assert_eq!(a.occupied_cells(), 1);
    }

    #[test]
    fn higher_fitness_elevates() {
        let mut a = MapElitesArchive::new();
        a.admit(descriptor(), entry(0.5));
        assert_eq!(a.admit(descriptor(), entry(0.7)), AdmitResult::Elevated);
    }

    #[test]
    fn lower_fitness_rejected() {
        let mut a = MapElitesArchive::new();
        a.admit(descriptor(), entry(0.7));
        assert_eq!(a.admit(descriptor(), entry(0.5)), AdmitResult::Rejected);
    }

    #[test]
    fn latency_bucket_boundaries() {
        assert_eq!(LatencyTier::bucket(0), LatencyTier::Sub10s);
        assert_eq!(LatencyTier::bucket(9_999), LatencyTier::Sub10s);
        assert_eq!(LatencyTier::bucket(10_000), LatencyTier::Sub60s);
        assert_eq!(LatencyTier::bucket(59_999), LatencyTier::Sub60s);
        assert_eq!(LatencyTier::bucket(60_000), LatencyTier::Over60s);
    }

    #[test]
    fn token_bucket_boundaries() {
        assert_eq!(TokenClass::bucket(0), TokenClass::Lt500);
        assert_eq!(TokenClass::bucket(499), TokenClass::Lt500);
        assert_eq!(TokenClass::bucket(500), TokenClass::Lt2k);
        assert_eq!(TokenClass::bucket(1_999), TokenClass::Lt2k);
        assert_eq!(TokenClass::bucket(2_000), TokenClass::Lt8k);
        assert_eq!(TokenClass::bucket(7_999), TokenClass::Lt8k);
        assert_eq!(TokenClass::bucket(8_000), TokenClass::Over8k);
    }
}
