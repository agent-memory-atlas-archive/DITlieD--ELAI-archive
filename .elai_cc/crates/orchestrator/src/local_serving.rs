//! J-48: local multi-model serving tier router.
//!
//! Coarse admission + warm/cold whole-model lifecycle + busy/cold cloud-spill.
//! This is the OUTER policy layer ONLY; it never touches tokens, batches, KV
//! cache, paged-cache, preemption, or MLFQ token re-ordering. Those belong to
//! the engine (J-19 vLLM). The router mirrors `CloudModeRouter`'s
//! stateless-decision shape: `decide()` is pure over inputs and returns a
//! typed enum, no I/O.
//!
//! Boundary (stub_ban from plan):
//!   NO continuous batching, NO chunked prefill, NO KV/paged-cache management,
//!   NO token-level preemption, NO MLFQ. `decide()` never sees a prompt body,
//!   only counts and tier ids. Per-tier concurrency caps + warm-set max are
//!   NAMED constants. Big tier (High) = EXCLUSIVE mode (small tiers forced
//!   cold when it is warm). SpillToCloud always carries a non-empty reason.

use crate::cloud_mode::SpendSnapshotInput;
use crate::role_predict::VramTier;
use serde_json::json;

/// Maximum concurrent in-flight requests per tier before admission queues.
/// Permissive defaults until measured on box (GD-5).
pub const TIER_CAP_LOW: u32 = 2;
pub const TIER_CAP_MEDIUM: u32 = 3;
pub const TIER_CAP_HIGH: u32 = 1;

/// Maximum number of warm tiers simultaneously resident. The big tier
/// (High) is EXCLUSIVE: when it is warm, small tiers are forced cold.
pub const WARM_SET_MAX: usize = 2;

/// Coarse FIFO priority classes (<=3, named, not an int knob).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedClass {
    /// Interactive / user-facing: highest local priority.
    Interactive,
    /// Autonomous orchestration loop: medium priority.
    Autonomous,
    /// Background / batch / research: lowest local priority.
    Background,
}

impl NamedClass {
    /// Snake-case wire label for SSB details_json + logs.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::Autonomous => "autonomous",
            Self::Background => "background",
        }
    }
}

/// Per-tier residency state tracked by the RuntimeOrchestrator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TierState {
    /// Whether the tier's model weights are currently warm (resident in VRAM).
    pub warm: bool,
    /// Current in-flight request count on this tier.
    pub inflight: u32,
}

impl TierState {
    /// Construct a cold, idle tier state.
    #[must_use]
    pub fn cold() -> Self {
        Self {
            warm: false,
            inflight: 0,
        }
    }

    /// Construct a warm, idle tier state.
    #[must_use]
    pub fn warm() -> Self {
        Self {
            warm: true,
            inflight: 0,
        }
    }
}

impl Default for TierState {
    fn default() -> Self {
        Self::cold()
    }
}

/// Snapshot of tier residency passed to `decide()`. The caller
/// (RuntimeOrchestrator) reads the RwLock and hands a plain map.
pub type TierResidencySnapshot = std::collections::HashMap<VramTier, TierState>;

/// The decision returned by `LocalServingRouter::decide()`.
#[derive(Debug, Clone, PartialEq)]
pub enum LocalServingDecision {
    /// Serve the request locally on the given tier.
    ServeLocal { tier: VramTier },
    /// Tier is warm but at concurrency cap; queue with a named priority class.
    AdmitQueued { tier: VramTier, class: NamedClass },
    /// Big tier requested but a small tier is warm (EXCLUSIVE): evict the
    /// small tier, then serve the big tier.
    HibernateThenServe { evict: VramTier, serve: VramTier },
    /// Spill to cloud because the local tier is cold/busy and cloud is
    /// available. `reason` is always non-empty.
    SpillToCloud { reason: String },
    /// Safe wait — corrupt input or negative inflight detected; never panic.
    Wait,
}

impl LocalServingDecision {
    /// Snake-case wire label for SSB details_json + logs.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ServeLocal { .. } => "serve_local",
            Self::AdmitQueued { .. } => "admit_queued",
            Self::HibernateThenServe { .. } => "hibernate_then_serve",
            Self::SpillToCloud { .. } => "spill_to_cloud",
            Self::Wait => "wait",
        }
    }

    /// Build a `serde_json::Value` for the `details_json` field of a
    /// `RouteDecisionEventPayload` (E-31 prefer-reuse, net 0 SSB).
    #[must_use]
    pub fn details_json(&self) -> serde_json::Value {
        match self {
            Self::ServeLocal { tier } => json!({
                "tier": tier.as_str(),
                "decision": self.as_str(),
            }),
            Self::AdmitQueued { tier, class } => json!({
                "tier": tier.as_str(),
                "decision": self.as_str(),
                "class": class.as_str(),
            }),
            Self::HibernateThenServe { evict, serve } => json!({
                "decision": self.as_str(),
                "evict": evict.as_str(),
                "serve": serve.as_str(),
            }),
            Self::SpillToCloud { reason } => json!({
                "decision": self.as_str(),
                "reason": reason,
            }),
            Self::Wait => json!({
                "decision": self.as_str(),
            }),
        }
    }
}

/// Inputs to `decide()`, mirroring the CloudModeRouter pattern: the
/// caller assembles the snapshot, the router is stateless.
#[derive(Debug, Clone)]
pub struct DecideInput<'a> {
    /// The tier the requesting role would prefer.
    pub requested_tier: VramTier,
    /// Live tier residency snapshot (warm/cold + inflight per tier).
    pub residency: &'a TierResidencySnapshot,
    /// Whether cloud routing is available (credentials loaded, not
    /// air-gapped). When false, SpillToCloud is never returned.
    pub cloud_available: bool,
    /// Spend snapshot for the cloud path (same type as CloudModeRouter).
    pub spend: Option<&'a SpendSnapshotInput>,
}

/// Stateless local serving router. Construct once, call `decide()`
/// repeatedly. Mirrors `CloudModeRouter`'s pure-decision shape.
#[derive(Debug, Clone)]
pub struct LocalServingRouter;

impl LocalServingRouter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Per-tier concurrency cap lookup.
    fn cap_for(tier: VramTier) -> u32 {
        match tier {
            VramTier::Low => TIER_CAP_LOW,
            VramTier::Medium => TIER_CAP_MEDIUM,
            VramTier::High => TIER_CAP_HIGH,
        }
    }

    /// Pure decision over inputs. Never panics; corrupt input (negative
    /// inflight) degrades to `Wait`.
    pub fn decide(&self, input: &DecideInput<'_>) -> LocalServingDecision {
        let requested = input.requested_tier;

        // Get the state for the requested tier, defaulting to cold/idle.
        let state = input.residency.get(&requested).cloned().unwrap_or_else(TierState::cold);

        // Corrupt input: negative inflight (u32 underflow guard). Safe Wait.
        // Since inflight is u32, negative is impossible, but a corrupt
        // very-large value (> cap + 100) is treated as corrupt.
        if state.inflight > u32::MAX / 2 {
            return LocalServingDecision::Wait;
        }

        let cap = Self::cap_for(requested);

        // EXCLUSIVE mode: if the big tier (High) is requested and any
        // small tier is warm, evict the small tier first. Cold-start the
        // big tier locally rather than spilling to cloud solely for being
        // cold.
        if requested == VramTier::High {
            let small_warm = input
                .residency
                .iter()
                .find(|(t, s)| **t != VramTier::High && s.warm)
                .map(|(t, _)| *t);

            if let Some(evict) = small_warm {
                return LocalServingDecision::HibernateThenServe {
                    evict,
                    serve: requested,
                };
            }
        }

        // If the big tier is warm and a small tier is requested, force
        // the small tier cold (EXCLUSIVE invariant: big tier warm =>
        // small tiers forced cold). The small tier cannot be served
        // locally while the big tier occupies VRAM; spill to cloud if
        // available, otherwise wait.
        if requested != VramTier::High {
            let big_warm = input
                .residency
                .get(&VramTier::High)
                .map(|s| s.warm)
                .unwrap_or(false);
            if big_warm {
                if input.cloud_available {
                    return LocalServingDecision::SpillToCloud {
                        reason: "big tier warm forces small tier cold, cloud available"
                            .to_string(),
                    };
                }
                return LocalServingDecision::Wait;
            }
        }

        // Requested tier is warm and under cap: serve locally.
        if state.warm && state.inflight < cap {
            return LocalServingDecision::ServeLocal { tier: requested };
        }

        // Tier is warm but at cap: queue locally.
        if state.warm && state.inflight >= cap {
            return LocalServingDecision::AdmitQueued {
                tier: requested,
                class: NamedClass::Autonomous,
            };
        }

        // Tier is cold and idle: cold-start admit to warm locally.
        // try_complete bumps warm+inflight on ServeLocal, so the tier
        // transitions from cold to warm through this path.
        if !state.warm && state.inflight == 0 {
            return LocalServingDecision::ServeLocal { tier: requested };
        }

        // Tier is cold but has inflight (warming concurrently): spill to
        // cloud if available, otherwise wait.
        if !state.warm && input.cloud_available {
            return LocalServingDecision::SpillToCloud {
                reason: format!(
                    "tier {} cold with concurrent load, cloud available",
                    requested.as_str()
                ),
            };
        }

        // Tier is cold, no cloud, concurrent load: wait.
        LocalServingDecision::Wait
    }
}

impl Default for LocalServingRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_residency() -> TierResidencySnapshot {
        TierResidencySnapshot::new()
    }

    #[test]
    fn test_serve_local_warm_under_cap() {
        let mut residency = empty_residency();
        residency.insert(VramTier::Medium, TierState { warm: true, inflight: 1 });
        let router = LocalServingRouter::new();
        let input = DecideInput {
            requested_tier: VramTier::Medium,
            residency: &residency,
            cloud_available: true,
            spend: None,
        };
        let decision = router.decide(&input);
        assert_eq!(decision, LocalServingDecision::ServeLocal { tier: VramTier::Medium });
    }

    #[test]
    fn test_admit_queued_at_cap() {
        let mut residency = empty_residency();
        residency.insert(VramTier::Low, TierState { warm: true, inflight: TIER_CAP_LOW });
        let router = LocalServingRouter::new();
        let input = DecideInput {
            requested_tier: VramTier::Low,
            residency: &residency,
            cloud_available: true,
            spend: None,
        };
        let decision = router.decide(&input);
        assert_eq!(
            decision,
            LocalServingDecision::AdmitQueued {
                tier: VramTier::Low,
                class: NamedClass::Autonomous,
            }
        );
    }

    #[test]
    fn test_hibernate_exclusive() {
        let mut residency = empty_residency();
        residency.insert(VramTier::High, TierState { warm: true, inflight: 0 });
        residency.insert(VramTier::Low, TierState { warm: true, inflight: 0 });
        let router = LocalServingRouter::new();
        let input = DecideInput {
            requested_tier: VramTier::High,
            residency: &residency,
            cloud_available: true,
            spend: None,
        };
        let decision = router.decide(&input);
        assert_eq!(
            decision,
            LocalServingDecision::HibernateThenServe {
                evict: VramTier::Low,
                serve: VramTier::High,
            }
        );
    }

    #[test]
    fn test_cold_start_serve_local() {
        let residency = empty_residency();
        let router = LocalServingRouter::new();
        let input = DecideInput {
            requested_tier: VramTier::Medium,
            residency: &residency,
            cloud_available: true,
            spend: None,
        };
        let decision = router.decide(&input);
        assert_eq!(decision, LocalServingDecision::ServeLocal { tier: VramTier::Medium });
    }

    #[test]
    fn test_spill_to_cloud_concurrent_load() {
        let mut residency = empty_residency();
        residency.insert(VramTier::Medium, TierState { warm: false, inflight: 1 });
        let router = LocalServingRouter::new();
        let input = DecideInput {
            requested_tier: VramTier::Medium,
            residency: &residency,
            cloud_available: true,
            spend: None,
        };
        let decision = router.decide(&input);
        match decision {
            LocalServingDecision::SpillToCloud { reason } => {
                assert!(!reason.is_empty(), "spill reason must be non-empty");
            }
            other => panic!("expected SpillToCloud, got {other:?}"),
        }
    }

    #[test]
    fn test_never_spill_when_air_gapped() {
        let residency = empty_residency();
        let router = LocalServingRouter::new();
        let input = DecideInput {
            requested_tier: VramTier::Medium,
            residency: &residency,
            cloud_available: false,
            spend: None,
        };
        let decision = router.decide(&input);
        assert!(
            !matches!(decision, LocalServingDecision::SpillToCloud { .. }),
            "must never spill when cloud unavailable (air-gapped)"
        );
    }

    #[test]
    fn test_corrupt_inflight_safe_wait() {
        let mut residency = empty_residency();
        residency.insert(
            VramTier::Medium,
            TierState {
                warm: true,
                inflight: u32::MAX / 2 + 1,
            },
        );
        let router = LocalServingRouter::new();
        let input = DecideInput {
            requested_tier: VramTier::Medium,
            residency: &residency,
            cloud_available: true,
            spend: None,
        };
        let decision = router.decide(&input);
        assert_eq!(decision, LocalServingDecision::Wait);
    }
}
