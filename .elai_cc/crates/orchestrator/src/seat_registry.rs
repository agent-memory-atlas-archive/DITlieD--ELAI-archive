//! J-49 Slice-1: multi-seat admission registry.
//!
//! Fair-share weighted token-bucket admission + per-seat concurrency caps +
//! optional daily/monthly spend ceilings (K-24: raw tokens + USD only).
//!
//! `seats.toml` shape (parsed with the existing `toml` workspace crate):
//! ```toml
//! [seats.alice]
//! weight = 1
//! concurrency_cap = 2
//! daily_usd_cap_micros = 5000000
//! monthly_usd_cap_micros = 100000000
//!
//! [seats.bob]
//! weight = 1
//! concurrency_cap = 2
//! ```
//! Absent file => `SeatRegistry::single_local()` (one implicit seat "local"
//! with permissive defaults — behavior matches pre-J-49 single-user).
//! Corrupt TOML => `Err(SeatRegistryError::Parse)`; the caller degrades.

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use serde::Deserialize;

/// Base token refill rate per second for a weight-1 seat. Seats with higher
/// fair-share weights get a proportionally higher rate.
pub const BASE_RATE_TOKENS_PER_SEC: u64 = 5;
/// Base bucket burst for a weight-1 seat (max tokens held). Scaled by weight.
pub const BASE_BURST: u64 = 10;
/// Implicit seat id used when no seats file is present (pre-J-49 behavior).
pub const LOCAL_SEAT_ID: &str = "local";
/// Seconds in a day, for the daily spend-cap window reset.
const SECS_PER_DAY: u64 = 86_400;
/// Seconds in a (30-day) month, for the monthly spend-cap window reset.
const SECS_PER_MONTH: u64 = 2_592_000;

/// Per-seat capability/limit knobs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatCaps {
    /// Fair-share weight; default 1. Higher weight => larger token-bucket
    /// rate and burst.
    pub weight: u32,
    /// Maximum in-flight turns for this seat.
    pub concurrency_cap: u32,
    /// Optional daily spend ceiling in USD micros (1e6 = $1).
    pub daily_usd_cap_micros: Option<u64>,
    /// Optional monthly spend ceiling in USD micros.
    pub monthly_usd_cap_micros: Option<u64>,
    /// Optional per-seat trust tier tag (C-48): `"T0"`|`"T1"`|`"T2"`|`"T3"`,
    /// or `None` when absent.
    pub trust_tier: Option<String>,
}

impl Default for SeatCaps {
    fn default() -> Self {
        Self {
            weight: 1,
            concurrency_cap: 4,
            daily_usd_cap_micros: None,
            monthly_usd_cap_micros: None,
            trust_tier: None,
        }
    }
}

/// Admission decision returned by [`SeatRegistry::admit`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeatAdmitDecision {
    /// Seat is below all caps; the turn may proceed.
    Admit,
    /// Seat is over a hard limit (spend ceiling exceeded).
    Deny {
        /// Non-empty human-readable reason.
        reason: String,
    },
    /// Seat is at the concurrency cap or has an empty token bucket; retry
    /// later. The caller may queue the turn.
    Queued {
        /// Non-empty human-readable reason.
        reason: String,
    },
}

/// Errors surfaced by [`SeatRegistry::load_from_path`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeatRegistryError {
    /// TOML parse failure.
    Parse(String),
    /// I/O failure other than "file not found" (which degrades to
    /// `single_local`).
    Io(String),
}

impl std::fmt::Display for SeatRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(s) => write!(f, "seat registry parse error: {s}"),
            Self::Io(s) => write!(f, "seat registry io error: {s}"),
        }
    }
}

impl std::error::Error for SeatRegistryError {}

/// Internal per-seat runtime state: caps + token bucket + inflight + spend.
#[derive(Debug, Clone)]
struct SeatEntry {
    caps: SeatCaps,
    tokens: u64,
    last_refill: Instant,
    inflight: u32,
    daily_spend_micros: u64,
    daily_window_start: Instant,
    monthly_spend_micros: u64,
    monthly_window_start: Instant,
}

impl SeatEntry {
    fn new(caps: SeatCaps, now: Instant) -> Self {
        let burst = caps.weight.max(1) as u64 * BASE_BURST;
        Self {
            caps,
            tokens: burst,
            last_refill: now,
            inflight: 0,
            daily_spend_micros: 0,
            daily_window_start: now,
            monthly_spend_micros: 0,
            monthly_window_start: now,
        }
    }

    /// Refill the token bucket based on elapsed time since the last refill.
    /// Weight scales both rate and burst.
    fn refill(&mut self, now: Instant) {
        let weight = self.caps.weight.max(1) as u64;
        let rate = weight * BASE_RATE_TOKENS_PER_SEC;
        let burst = weight * BASE_BURST;
        let elapsed = now.saturating_duration_since(self.last_refill).as_secs();
        if elapsed == 0 {
            return;
        }
        let add = elapsed.saturating_mul(rate);
        self.tokens = self.tokens.saturating_add(add).min(burst);
        self.last_refill = now;
    }

    /// Reset daily/monthly spend windows if their period has elapsed.
    fn roll_windows(&mut self, now: Instant) {
        let daily_elapsed = now.saturating_duration_since(self.daily_window_start).as_secs();
        if daily_elapsed >= SECS_PER_DAY {
            self.daily_spend_micros = 0;
            self.daily_window_start = now;
        }
        let monthly_elapsed = now.saturating_duration_since(self.monthly_window_start).as_secs();
        if monthly_elapsed >= SECS_PER_MONTH {
            self.monthly_spend_micros = 0;
            self.monthly_window_start = now;
        }
    }
}

/// Multi-seat admission registry. Maps seat ids (the `identity` field of a
/// `UsageTurnEvent`) to [`SeatCaps`] plus runtime token-bucket / concurrency
/// / spend state.
#[derive(Debug, Clone)]
pub struct SeatRegistry {
    seats: HashMap<String, SeatEntry>,
}

/// Intermediate deserialization shape for `seats.toml`.
#[derive(Debug, Deserialize, Default)]
struct SeatsFile {
    #[serde(default)]
    seats: HashMap<String, SeatCapsToml>,
}

/// Per-seat TOML row; all fields optional (defaults via [`SeatCaps::default`]).
#[derive(Debug, Deserialize, Default)]
struct SeatCapsToml {
    #[serde(default)]
    weight: Option<u32>,
    #[serde(default)]
    concurrency_cap: Option<u32>,
    #[serde(default)]
    daily_usd_cap_micros: Option<u64>,
    #[serde(default)]
    monthly_usd_cap_micros: Option<u64>,
    #[serde(default)]
    trust_tier: Option<String>,
}

impl From<SeatCapsToml> for SeatCaps {
    fn from(t: SeatCapsToml) -> Self {
        let d = SeatCaps::default();
        Self {
            weight: t.weight.unwrap_or(d.weight),
            concurrency_cap: t.concurrency_cap.unwrap_or(d.concurrency_cap),
            daily_usd_cap_micros: t.daily_usd_cap_micros,
            monthly_usd_cap_micros: t.monthly_usd_cap_micros,
            trust_tier: t.trust_tier,
        }
    }
}

impl SeatRegistry {
    /// Load a seat registry from a `seats.toml` file. A missing file degrades
    /// to [`SeatRegistry::single_local`]. Any other I/O error or corrupt TOML
    /// returns [`SeatRegistryError`] so the caller can decide how to degrade.
    pub fn load_from_path(path: &Path) -> Result<Self, SeatRegistryError> {
        let contents = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::single_local());
            }
            Err(e) => return Err(SeatRegistryError::Io(e.to_string())),
        };
        let file: SeatsFile = toml::from_str(&contents)
            .map_err(|e| SeatRegistryError::Parse(e.to_string()))?;
        let now = Instant::now();
        let mut seats = HashMap::new();
        if file.seats.is_empty() {
            // Empty/sectionless file => single local seat (permissive).
            seats.insert(LOCAL_SEAT_ID.to_string(), SeatEntry::new(SeatCaps::default(), now));
        } else {
            for (id, caps_toml) in file.seats {
                let caps = SeatCaps::from(caps_toml);
                seats.insert(id, SeatEntry::new(caps, now));
            }
        }
        Ok(Self { seats })
    }

    /// Construct a registry with a single implicit `"local"` seat and
    /// permissive defaults (matches pre-J-49 single-user behavior).
    #[must_use]
    pub fn single_local() -> Self {
        let now = Instant::now();
        let mut seats = HashMap::new();
        seats.insert(
            LOCAL_SEAT_ID.to_string(),
            SeatEntry::new(SeatCaps::default(), now),
        );
        Self { seats }
    }

    /// Admission check for a turn from `identity`.
    ///
    /// Applies, in order: spend ceilings (daily then monthly => `Deny`),
    /// concurrency cap (`Queued`), token-bucket fair-share (`Queued` if the
    /// bucket cannot supply one token). On `Admit`, inflight is incremented
    /// and the estimated cost is added to the spend accumulators.
    ///
    /// `est_cost_micros` is the turn's estimated cost in USD micros (0 if
    /// unpriced). An unknown `identity` falls back to the `"local"` seat if
    /// present, else `Deny`.
    pub fn admit(&mut self, identity: &str, est_cost_micros: u64) -> SeatAdmitDecision {
        let now = Instant::now();
        let seat_id = self.resolve_seat_id(identity);
        let entry = match self.seats.get_mut(&seat_id) {
            Some(e) => e,
            None => {
                return SeatAdmitDecision::Deny {
                    reason: format!("unknown seat '{identity}' and no local fallback"),
                };
            }
        };
        entry.roll_windows(now);

        // Spend ceilings (hard deny).
        if let Some(cap) = entry.caps.daily_usd_cap_micros {
            if entry.daily_spend_micros.saturating_add(est_cost_micros) > cap {
                return SeatAdmitDecision::Deny {
                    reason: format!(
                        "daily spend cap {cap} micros exceeded for seat '{seat_id}'"
                    ),
                };
            }
        }
        if let Some(cap) = entry.caps.monthly_usd_cap_micros {
            if entry.monthly_spend_micros.saturating_add(est_cost_micros) > cap {
                return SeatAdmitDecision::Deny {
                    reason: format!(
                        "monthly spend cap {cap} micros exceeded for seat '{seat_id}'"
                    ),
                };
            }
        }

        // Concurrency cap (queue, not deny).
        if entry.inflight >= entry.caps.concurrency_cap {
            return SeatAdmitDecision::Queued {
                reason: format!(
                    "concurrency cap {} reached for seat '{seat_id}'",
                    entry.caps.concurrency_cap
                ),
            };
        }

        // Token-bucket fair-share (queue if empty).
        entry.refill(now);
        if entry.tokens == 0 {
            return SeatAdmitDecision::Queued {
                reason: format!("token bucket empty for seat '{seat_id}'"),
            };
        }
        entry.tokens -= 1;
        entry.inflight = entry.inflight.saturating_add(1);
        entry.daily_spend_micros = entry.daily_spend_micros.saturating_add(est_cost_micros);
        entry.monthly_spend_micros = entry.monthly_spend_micros.saturating_add(est_cost_micros);
        SeatAdmitDecision::Admit
    }

    /// Release a turn's concurrency slot after completion (success or fail).
    /// Unknown `identity` is a no-op (safe).
    pub fn release(&mut self, identity: &str) {
        let seat_id = self.resolve_seat_id(identity);
        if let Some(entry) = self.seats.get_mut(&seat_id) {
            entry.inflight = entry.inflight.saturating_sub(1);
        }
    }

    /// Return all registered seat ids.
    pub fn seat_ids(&self) -> Vec<String> {
        self.seats.keys().cloned().collect()
    }

    /// Resolve an identity to a seat id, falling back to `"local"` when the
    /// identity is unknown but a local seat exists.
    pub fn resolve_seat_id(&self, identity: &str) -> String {
        if self.seats.contains_key(identity) {
            identity.to_string()
        } else if self.seats.contains_key(LOCAL_SEAT_ID) {
            LOCAL_SEAT_ID.to_string()
        } else {
            identity.to_string()
        }
    }

    /// Look up the [`SeatCaps`] for the seat resolved from `identity`.
    /// Returns `None` if no matching seat exists (and no local fallback).
    pub fn caps_for(&self, identity: &str) -> Option<&SeatCaps> {
        let seat_id = self.resolve_seat_id(identity);
        self.seats.get(&seat_id).map(|e| &e.caps)
    }

    /// Return the per-seat trust tier tag (C-48) for `identity`, or `None` if
    /// the seat has no `trust_tier` set (or is unknown).
    pub fn trust_tier_tag_for(&self, identity: &str) -> Option<&str> {
        self.caps_for(identity).and_then(|c| c.trust_tier.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // (a) single_local admits "local"
    #[test]
    fn single_local_admits_local() {
        let mut reg = SeatRegistry::single_local();
        assert_eq!(reg.admit("local", 0), SeatAdmitDecision::Admit);
        reg.release("local");
    }

    // (b) two seats equal weight; flooding A does not force Deny on B's first admit
    #[test]
    fn flooding_a_does_not_deny_b() {
        let now = Instant::now();
        let mut seats = HashMap::new();
        seats.insert(
            "alice".to_string(),
            SeatEntry::new(
                SeatCaps {
                    weight: 1,
                    concurrency_cap: 4,
                    daily_usd_cap_micros: None,
                    monthly_usd_cap_micros: None,
                    trust_tier: None,
                },
                now,
            ),
        );
        seats.insert(
            "bob".to_string(),
            SeatEntry::new(
                SeatCaps {
                    weight: 1,
                    concurrency_cap: 4,
                    daily_usd_cap_micros: None,
                    monthly_usd_cap_micros: None,
                    trust_tier: None,
                },
                now,
            ),
        );
        let mut reg = SeatRegistry { seats };

        // Flood alice: drain her bucket (BASE_BURST=10 tokens, cap=4 inflight).
        // First 4 admits consume concurrency; further admits queue.
        let mut alice_admits = 0;
        for _ in 0..20 {
            if reg.admit("alice", 0) == SeatAdmitDecision::Admit {
                alice_admits += 1;
            }
        }
        assert!(alice_admits <= 4, "alice admits bounded by concurrency");

        // Bob's first admit must still succeed — independent bucket.
        assert_eq!(reg.admit("bob", 0), SeatAdmitDecision::Admit);
    }

    // (c) concurrency_cap=1: second concurrent admit is Queued; after release, admits
    #[test]
    fn concurrency_cap_one_queues_then_admits() {
        let now = Instant::now();
        let mut seats = HashMap::new();
        seats.insert(
            "solo".to_string(),
            SeatEntry::new(
                SeatCaps {
                    weight: 1,
                    concurrency_cap: 1,
                    daily_usd_cap_micros: None,
                    monthly_usd_cap_micros: None,
                    trust_tier: None,
                },
                now,
            ),
        );
        let mut reg = SeatRegistry { seats };

        assert_eq!(reg.admit("solo", 0), SeatAdmitDecision::Admit);
        // Second concurrent admit is queued (concurrency cap reached).
        match reg.admit("solo", 0) {
            SeatAdmitDecision::Queued { .. } => {}
            other => panic!("expected Queued, got {other:?}"),
        }
        // After release, admits again.
        reg.release("solo");
        assert_eq!(reg.admit("solo", 0), SeatAdmitDecision::Admit);
    }

    // (d) Deny reasons for spend use usd/cap/micros language, never "credit".
    #[test]
    fn deny_reasons_use_spend_language_not_credit() {
        let now = Instant::now();
        let mut seats = HashMap::new();
        seats.insert(
            "frugal".to_string(),
            SeatEntry::new(
                SeatCaps {
                    weight: 1,
                    concurrency_cap: 4,
                    daily_usd_cap_micros: Some(1_000_000),
                    monthly_usd_cap_micros: None,
                    trust_tier: None,
                },
                now,
            ),
        );
        let mut reg = SeatRegistry { seats };
        // First admit under cap => Admit.
        assert_eq!(reg.admit("frugal", 600_000), SeatAdmitDecision::Admit);
        reg.release("frugal");
        // Second admit over cap => Deny with spend/micros/cap language.
        let reason = match reg.admit("frugal", 600_000) {
            SeatAdmitDecision::Deny { reason } => reason,
            other => panic!("expected Deny, got {other:?}"),
        };
        let lower = reason.to_lowercase();
        assert!(
            lower.contains("cap") || lower.contains("micros") || lower.contains("spend"),
            "deny reason must use cap/micros/spend language, got: {reason}"
        );
        let credit = format!("{}{}", "cred", "it");
        assert!(
            !lower.contains(&credit),
            "deny reason must not introduce synthetic 'credit' unit: {reason}"
        );
    }

    // Bonus: load_from_path missing file => single_local.
    #[test]
    fn missing_file_degrades_to_single_local() {
        let path = std::env::temp_dir().join("elai_j49_nonexistent_seats.toml");
        let reg = SeatRegistry::load_from_path(&path).expect("missing file => single_local");
        assert_eq!(reg.seat_ids(), vec!["local".to_string()]);
    }

    // Bonus: corrupt TOML => Err(Parse).
    #[test]
    fn corrupt_toml_returns_err() {
        let path = std::env::temp_dir().join("elai_j49_corrupt_seats.toml");
        std::fs::write(&path, "this is = not = valid toml [[[").unwrap();
        let res = SeatRegistry::load_from_path(&path);
        assert!(matches!(res, Err(SeatRegistryError::Parse(_))));
        let _ = std::fs::remove_file(&path);
    }

    // Bonus: daily spend cap denies.
    #[test]
    fn daily_cap_denies() {
        let now = Instant::now();
        let mut seats = HashMap::new();
        seats.insert(
            "frugal".to_string(),
            SeatEntry::new(
                SeatCaps {
                    weight: 1,
                    concurrency_cap: 4,
                    daily_usd_cap_micros: Some(1_000_000),
                    monthly_usd_cap_micros: None,
                    trust_tier: None,
                },
                now,
            ),
        );
        let mut reg = SeatRegistry { seats };
        // First admit with cost 600k micros => ok (under 1M).
        assert_eq!(reg.admit("frugal", 600_000), SeatAdmitDecision::Admit);
        reg.release("frugal");
        // Second admit with cost 600k => 1.2M > 1M => Deny.
        match reg.admit("frugal", 600_000) {
            SeatAdmitDecision::Deny { .. } => {}
            other => panic!("expected Deny, got {other:?}"),
        }
    }

    // J-59 residual-A: per-seat trust_tier loads from seats.toml.
    #[test]
    fn trust_tier_tag_loaded_and_absent() {
        let path = std::env::temp_dir().join("elai_j59_trust_tier_seats.toml");
        std::fs::write(
            &path,
            "[seats.low]\ntrust_tier = \"T3\"\n\n[seats.high]\nweight = 2\n",
        )
        .unwrap();
        let reg = SeatRegistry::load_from_path(&path).expect("parse ok");
        assert_eq!(reg.trust_tier_tag_for("low"), Some("T3"));
        // Absent trust_tier field => None.
        assert_eq!(reg.trust_tier_tag_for("high"), None);
        let _ = std::fs::remove_file(&path);
    }
}
