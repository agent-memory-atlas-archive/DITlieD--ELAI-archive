//! I-10 -- prefix invariance + TTL tiering + cache-hit hard guard.
//!
//! Provides three composable units built on top of the H-1/H-2 shipped
//! substrate:
//!
//! * [`CacheEconomicsPlanner`] — reads inter-call gap stats from SSB journal
//!   rows and outputs a [`TtlTier`] per call class. Gates on
//!   [`ContextPolicy::Cloud`]; is a no-op on [`ContextPolicy::Local`].
//!
//! * [`PrefixInvarianceGuard`] — drives from REAL cache_read_tokens (SSB
//!   `cloud_spend` rows) rather than from `bp.valid` alone (which is blind to
//!   the pre-existing 8-slot/10-slot drift in runtime_client.rs — NOTE: that
//!   drift is an out-of-scope fix tracked in gotchas.md). A turn where
//!   `bp.valid=true` but `cache_read_tokens=0` is the unexplained-miss signal.
//!   Unexplained miss-rate >10% emits [`CacheEconomicsEventKind::HitRateGuardTripped`]
//!   and optionally [`CacheEconomicsEventKind::PrefixDiffDumped`].
//!
//! * [`max_tokens_clamp`] — schema-bound max_tokens helper preventing the
//!   4096-token default from burning quota 25x on narrow scoring calls.
//!
//! Kill-switch: `ELAI_CACHE_ECONOMICS_DISABLE=1` suppresses guard emission
//! (passes through silently). The planner still returns a tier; only the SSB
//! emit is gated, so callers reading the tier are unaffected.
//!
//! Compose rules (HD-1, GD-1):
//!   - NEVER re-implement `apply_anthropic_cache_markers` (H-1 shipped).
//!   - NEVER re-implement `update_breakpoints` blake3 logic (H-2 shipped).
//!   - TTL annotation = set `cache_control` on the system head BEFORE calling
//!     `apply_anthropic_cache_markers`; the `is_none` guard preserves it.
//!   - No fabricated miss-rate or cost measurements (HD-1).
//!   - No timestamps/UUIDs in the cached prefix region.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use thiserror::Error;

use crate::cache_check::CacheCheckSnapshot;
use crate::context_policy::ContextPolicy;
use crate::ssb::{
    CacheEconomicsEventKind, CacheEconomicsEventPayload, Ssb, SsbError, SsbPayload,
};

// ---------------------------------------------------------------------------
// Kill-switch
// ---------------------------------------------------------------------------

const ELAI_CACHE_ECONOMICS_DISABLE_VAR: &str = "ELAI_CACHE_ECONOMICS_DISABLE";

fn cache_economics_disabled() -> bool {
    std::env::var(ELAI_CACHE_ECONOMICS_DISABLE_VAR)
        .map(|v| v == "1")
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// 1-hour TTL in seconds. Billed at 2x write cost; breakeven ~3 reads.
pub const TTL_LONG_SECS: u32 = 3_600;

/// 5-minute TTL in seconds. Billed at 1.25x write cost; breakeven 1 read.
pub const TTL_SHORT_SECS: u32 = 300;

/// Write-premium multiplier for the 1h tier (default; re-verify at edit).
pub const WRITE_PREMIUM_1H: f64 = 2.0;

/// Write-premium multiplier for the 5m tier (default; re-verify at edit).
pub const WRITE_PREMIUM_5M: f64 = 1.25;

/// Reads required to break even on the 1h write premium (ceil(2.0 / 1.0) = 2 extra,
/// so 3 total reads before net positive — landmine #4 default).
pub const BREAKEVEN_READS_1H: u32 = 3;

/// Reads required to break even on the 5m write premium (1 read >= breakeven).
pub const BREAKEVEN_READS_5M: u32 = 1;

/// Guard fires when the unexplained miss-rate exceeds this fraction.
pub const MISS_RATE_GUARD_THRESHOLD: f64 = 0.10;

/// Schema-bound: max output tokens per schema field (conservative).
pub const MAX_TOKENS_PER_FIELD: u32 = 32;

/// Minimum plausible cached prefix length in bytes. Prefixes shorter than
/// this are not worth writing even at the 5m tier.
pub const MIN_CACHEABLE_PREFIX_BYTES: usize = 256;

// ---------------------------------------------------------------------------
// Timestamp / UUID pattern bytes used by the prefix validator.
// These are conservative prefix-of-pattern checks to keep the validator
// allocation-free and constant-time on short strings.
// ---------------------------------------------------------------------------

/// ISO-8601 date prefix pattern: "2026-" or "2025-" etc.
const TS_DIGIT_DASH: &[u8] = b"202";

/// UUID format marker: 8 hex chars + dash.
const UUID_HEX_DASH_LEN: usize = 9;

/// ELAI session-id prefix: "s-" followed by 6-10 lowercase hex chars.
/// E.g. "s-107b022a", "s-abc123ef". Present in CLI prompts and telemetry prefixes.
const SESSION_ID_PREFIX: &[u8] = b"s-";
/// Minimum hex-char count after "s-" to qualify as a session id.
const SESSION_ID_MIN_HEX: usize = 6;
/// Maximum hex-char count after "s-" before we stop scanning.
const SESSION_ID_MAX_HEX: usize = 10;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum CacheEconomicsError {
    #[error("cache economics SSB write failed: {0}")]
    Ssb(#[from] SsbError),
}

// ---------------------------------------------------------------------------
// TTL tier
// ---------------------------------------------------------------------------

/// TTL tier output of [`CacheEconomicsPlanner::plan_ttl`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtlTier {
    /// 1-hour TTL: use [`elai_providers::types::CacheControl::ephemeral_1h()`].
    Long,
    /// 5-minute TTL: use [`elai_providers::types::CacheControl::ephemeral()`].
    Short,
    /// Local policy or no-cache context: do not annotate.
    NoCache,
}

impl TtlTier {
    /// The TTL in seconds, or `None` for [`TtlTier::NoCache`].
    pub fn ttl_secs(self) -> Option<u32> {
        match self {
            Self::Long => Some(TTL_LONG_SECS),
            Self::Short => Some(TTL_SHORT_SECS),
            Self::NoCache => None,
        }
    }

    /// Breakeven read count for this tier (how many reads before net positive
    /// over skipping the write premium entirely).
    pub fn breakeven_reads(self) -> u32 {
        match self {
            Self::Long => BREAKEVEN_READS_1H,
            Self::Short => BREAKEVEN_READS_5M,
            Self::NoCache => 0,
        }
    }

    /// Human-readable label for SSB / dashboard display.
    pub fn label(self) -> &'static str {
        match self {
            Self::Long => "1h",
            Self::Short => "5m",
            Self::NoCache => "default",
        }
    }
}

// ---------------------------------------------------------------------------
// Gap stats (inter-call gap from journal rows)
// ---------------------------------------------------------------------------

/// Summary of inter-call gap statistics derived from SSB `cloud_spend` rows.
/// The planner uses `mean_gap_ms` to decide whether 1h TTL is cost-effective
/// (short gaps = high reuse = 1h preferred; long gaps = 5m safer).
#[derive(Debug, Clone, Copy, Default)]
pub struct GapStats {
    /// Mean inter-call gap in milliseconds (0 when no history).
    pub mean_gap_ms: u64,
    /// Number of observed gaps (0 = no history = use conservative default).
    pub sample_count: u32,
}

impl GapStats {
    /// Heuristic: if mean inter-call gap is under 10 minutes the session is
    /// interactive and 1h TTL is cost-effective.
    pub fn suggests_long_ttl(self) -> bool {
        self.sample_count > 0 && self.mean_gap_ms < 600_000
    }
}

// ---------------------------------------------------------------------------
// Guard verdict
// ---------------------------------------------------------------------------

/// Result of [`PrefixInvarianceGuard::check_miss_rate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardVerdict {
    /// Miss-rate is within threshold or policy is Local — no action.
    Ok,
    /// Unexplained miss-rate exceeds threshold; caller should halt and emit
    /// [`CacheEconomicsEventKind::HitRateGuardTripped`].
    Tripped,
}

// ---------------------------------------------------------------------------
// Prefix mutation detection
// ---------------------------------------------------------------------------

/// A detected mutable token in a prefix byte range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixMutation {
    /// Byte offset in the prefix string where the mutable token begins.
    pub byte_offset: usize,
    /// Kind of mutation detected.
    pub kind: MutationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationKind {
    /// Timestamp-shaped sequence (ISO-8601, unix epoch string, etc.).
    Timestamp,
    /// UUID-shaped token (8-4-4-4-12 hex).
    Uuid,
    /// Mutable endpoint string (contains ":8100", ":8080", "localhost", etc.).
    MutableEndpoint,
    /// ELAI session-id token (pattern: "s-" followed by 6-10 lowercase hex chars).
    /// Common in telemetry/logging prefixes; breaks prefix byte-stability every session.
    SessionId,
}

/// Check `prefix_text` for timestamp, UUID, or mutable endpoint tokens.
///
/// Returns `Ok(())` if the prefix appears byte-frozen, or an `Err` listing
/// all detected mutations. This is a conservative byte-scan (no allocator
/// per call; works on borrowed slices). Called before stamping 1h TTL.
pub fn assert_prefix_has_no_mutable_tokens(
    prefix_text: &str,
) -> Result<(), Vec<PrefixMutation>> {
    let bytes = prefix_text.as_bytes();
    let mut mutations: Vec<PrefixMutation> = Vec::new();

    let mut i = 0usize;
    while i < bytes.len() {
        // Timestamp pattern: starts with "202" (covers 2020-2029 range safely).
        if bytes[i..].starts_with(TS_DIGIT_DASH) && i + 4 < bytes.len() {
            // Check for "2026-06-12" shaped 10-char date or "2026-06-12T" ISO variant.
            let slice = &bytes[i..];
            if slice.len() >= 10
                && slice[4] == b'-'
                && slice[7] == b'-'
                && slice[3].is_ascii_digit()
                && slice[5].is_ascii_digit()
                && slice[6].is_ascii_digit()
                && slice[8].is_ascii_digit()
                && slice[9].is_ascii_digit()
            {
                mutations.push(PrefixMutation { byte_offset: i, kind: MutationKind::Timestamp });
                i += 10;
                continue;
            }
        }
        // UUID pattern: 8 hex digits + '-'
        if i + UUID_HEX_DASH_LEN <= bytes.len() {
            let candidate = &bytes[i..i + UUID_HEX_DASH_LEN];
            let is_8hex_dash = candidate[8] == b'-'
                && candidate[..8].iter().all(|b| b.is_ascii_hexdigit());
            if is_8hex_dash && i + 36 <= bytes.len() {
                // Check the rest of the UUID pattern (groups: 4-4-4-12).
                let rest = &bytes[i + 9..];
                if rest.len() >= 27
                    && rest[4] == b'-'
                    && rest[9] == b'-'
                    && rest[14] == b'-'
                    && rest[..4].iter().all(|b| b.is_ascii_hexdigit())
                    && rest[5..9].iter().all(|b| b.is_ascii_hexdigit())
                    && rest[10..14].iter().all(|b| b.is_ascii_hexdigit())
                    && rest[15..27].iter().all(|b| b.is_ascii_hexdigit())
                {
                    mutations.push(PrefixMutation { byte_offset: i, kind: MutationKind::Uuid });
                    i += 36;
                    continue;
                }
            }
        }
        // Mutable endpoint: "localhost", ":8100", ":8080", ":3000".
        for needle in [b"localhost".as_ref(), b":8100", b":8080", b":3000"] {
            if bytes[i..].starts_with(needle) {
                mutations.push(PrefixMutation {
                    byte_offset: i,
                    kind: MutationKind::MutableEndpoint,
                });
                i += needle.len();
            }
        }
        // ELAI session id: "s-" + 6-10 hex chars (digits 0-9 or lowercase a-f).
        // Uppercase hex (A-F) is NOT matched -- ELAI session ids use lowercase only.
        if bytes[i..].starts_with(SESSION_ID_PREFIX) {
            let rest = &bytes[i + SESSION_ID_PREFIX.len()..];
            let hex_run = rest
                .iter()
                .take(SESSION_ID_MAX_HEX)
                .take_while(|&&b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
                .count();
            if hex_run >= SESSION_ID_MIN_HEX {
                mutations.push(PrefixMutation {
                    byte_offset: i,
                    kind: MutationKind::SessionId,
                });
                i += SESSION_ID_PREFIX.len() + hex_run;
                continue;
            }
        }
        i += 1;
    }

    if mutations.is_empty() { Ok(()) } else { Err(mutations) }
}

// ---------------------------------------------------------------------------
// CacheEconomicsPlanner
// ---------------------------------------------------------------------------

/// Journal-driven TTL tier selector.
///
/// The planner COMPOSES on top of H-1/H-2 (never re-implements them).
/// It reads `GapStats` (derived by the caller from SSB `cloud_spend` row
/// timestamps) and the `ContextPolicy` to output a [`TtlTier`].
pub struct CacheEconomicsPlanner;

impl CacheEconomicsPlanner {
    /// Select the TTL tier for a single cloud call.
    ///
    /// * `gap_stats` — inter-call gap statistics from SSB journal rows.
    /// * `policy` — context policy for this session (from `regime_policy`).
    /// * `prefix_stable` — `bp.valid` value from the most recent
    ///   `update_breakpoints` call (prefix hash matched previous turn).
    ///
    /// Returns [`TtlTier::NoCache`] when `policy == Local` (sidecar KV cache
    /// is a separate Python-side mechanism; I-10 is a cloud-lane concern).
    #[must_use]
    pub fn plan_ttl(gap_stats: GapStats, policy: ContextPolicy, prefix_stable: bool) -> TtlTier {
        if policy == ContextPolicy::Local {
            return TtlTier::NoCache;
        }
        // Only stamp 1h on a verified-stable prefix (bp.valid=true).
        // Landmine #1: ALWAYS set TTL explicitly — never rely on provider default.
        if prefix_stable && gap_stats.suggests_long_ttl() {
            TtlTier::Long
        } else {
            TtlTier::Short
        }
    }
}

// ---------------------------------------------------------------------------
// PrefixInvarianceGuard
// ---------------------------------------------------------------------------

/// Miss-rate monitor that halts and emits [`CacheEconomicsEventKind::HitRateGuardTripped`]
/// when the unexplained miss-rate exceeds [`MISS_RATE_GUARD_THRESHOLD`].
///
/// "Unexplained" means: `bp.valid=true` on a turn AND `cache_read_tokens=0` in
/// the SSB row for that turn (the prefix was byte-stable but Anthropic returned
/// no cache hit — indicates a silent prefix mutation not caught by bp.valid due
/// to the pre-existing 8-slot/10-slot drift or a write-premium miss).
///
/// NOTE: runtime_client.rs feeds 8 slots to update_breakpoints but SLOT_COUNT=10
/// (H-53 added Evidence slot 9, H-31 Memory slot 8). The 2 missing slots may
/// contain mutable content invisible to bp.valid. This guard cross-checks with
/// cache_read_tokens>0 from SSB rows as the additional signal. The slot drift
/// is out-of-scope for I-10 (R-3 scope contract); logged in gotchas.md.
pub struct PrefixInvarianceGuard;

impl PrefixInvarianceGuard {
    /// Check the current miss-rate and return a [`GuardVerdict`].
    ///
    /// * `snap` — `CacheCheckSnapshot` built from SSB `cloud_spend` rows
    ///   (reuses the H-9 `CacheCheckSnapshot` struct; does NOT call
    ///   `measure_cache_hit_rate` which emits the wrong SSB type).
    /// * `policy` — no-op verdict when `ContextPolicy::Local`.
    #[must_use]
    pub fn check_miss_rate(snap: CacheCheckSnapshot, policy: ContextPolicy) -> GuardVerdict {
        if policy == ContextPolicy::Local {
            return GuardVerdict::Ok;
        }
        let total = snap.hits + snap.misses;
        if total == 0 {
            return GuardVerdict::Ok; // no evidence = R2: dead evidence path returns Ok (not FAIL)
        }
        let miss_rate = (snap.misses as f64) / (total as f64);
        if miss_rate > MISS_RATE_GUARD_THRESHOLD {
            GuardVerdict::Tripped
        } else {
            GuardVerdict::Ok
        }
    }

    /// Build the blake3-based mutation hash for a detected byte-range mutation.
    /// Used as `mutation_hash` in the [`CacheEconomicsEventPayload`].
    #[must_use]
    pub fn mutation_hash_for(prefix_text: &str, byte_offset: usize) -> [u8; 8] {
        let mut h = blake3::Hasher::new();
        h.update(prefix_text.as_bytes());
        h.update(&(byte_offset as u64).to_le_bytes());
        let d = h.finalize();
        let b = d.as_bytes();
        [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]
    }
}

// ---------------------------------------------------------------------------
// max_tokens_clamp
// ---------------------------------------------------------------------------

/// Schema-bound max_tokens helper.
///
/// Landmine #2: unclamped `max_tokens=4096` burns quota ~25x on a 3-field
/// score response. This function returns a value clamped to
/// `schema_fields * MAX_TOKENS_PER_FIELD`, ensuring the provider does not
/// over-allocate KV cache for short structured responses.
///
/// `schema_fields` must be > 0; returns `MAX_TOKENS_PER_FIELD` as a floor
/// when 0 is passed (defensive, avoids returning 0 which would cause an API
/// error).
#[must_use]
pub fn max_tokens_clamp(schema_fields: u32) -> u32 {
    let fields = if schema_fields == 0 { 1 } else { schema_fields };
    fields.saturating_mul(MAX_TOKENS_PER_FIELD)
}

// ---------------------------------------------------------------------------
// SSB emit helper
// ---------------------------------------------------------------------------

/// Emit a [`CacheEconomicsEventPayload`] to the SSB.
///
/// No-ops when `ELAI_CACHE_ECONOMICS_DISABLE=1`. Returns `Ok(())` on skip
/// so callers do not need to check the kill-switch themselves.
pub async fn emit_cache_economics_event(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    payload: CacheEconomicsEventPayload,
) -> Result<(), CacheEconomicsError> {
    if cache_economics_disabled() {
        return Ok(());
    }
    ssb.write_payload(
        agent_id,
        superstep,
        SsbPayload::CacheEconomicsEvent(payload),
        Vec::new(),
    )
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// CacheEconomicsEventPayload helpers
// (Struct and Kind live in ssb.rs as SSB wire types; imported at top of file.)
// ---------------------------------------------------------------------------

impl CacheEconomicsEventPayload {
    /// Construct a `TtlPlanned` payload.
    pub fn ttl_planned(tier: TtlTier, call_class: &str) -> Self {
        Self {
            kind: CacheEconomicsEventKind::TtlPlanned,
            ttl_secs: tier.ttl_secs(),
            breakeven_reads: tier.breakeven_reads(),
            miss_rate: 0.0,
            mutation_hash: [0u8; 8],
            call_class: call_class.to_string(),
            ts_ms: now_ms(),
        }
    }

    /// Construct a `HitRateGuardTripped` payload.
    pub fn guard_tripped(miss_rate: f64, call_class: &str) -> Self {
        Self {
            kind: CacheEconomicsEventKind::HitRateGuardTripped,
            ttl_secs: None,
            breakeven_reads: 0,
            miss_rate,
            mutation_hash: [0u8; 8],
            call_class: call_class.to_string(),
            ts_ms: now_ms(),
        }
    }

    /// Construct a `PrefixDiffDumped` payload.
    pub fn prefix_diff_dumped(mutation_hash: [u8; 8], call_class: &str) -> Self {
        Self {
            kind: CacheEconomicsEventKind::PrefixDiffDumped,
            ttl_secs: None,
            breakeven_reads: 0,
            miss_rate: 0.0,
            mutation_hash,
            call_class: call_class.to_string(),
            ts_ms: now_ms(),
        }
    }

    /// Journal an L-17 freeze recompute with a typed cause, not a string.
    pub fn prefix_recomputed(
        cause: crate::context_projection::RecomputeCause,
        boundary_hash: [u8; 8],
        call_class: &str,
    ) -> Self {
        let cause = match cause {
            crate::context_projection::RecomputeCause::FirstCall => {
                crate::ssb::FreezePrefixRecomputeCause::FirstCall
            }
            crate::context_projection::RecomputeCause::TailOverflow => {
                crate::ssb::FreezePrefixRecomputeCause::TailOverflow
            }
            crate::context_projection::RecomputeCause::BoundaryMismatch => {
                crate::ssb::FreezePrefixRecomputeCause::BoundaryMismatch
            }
            crate::context_projection::RecomputeCause::HistoryRewound => {
                crate::ssb::FreezePrefixRecomputeCause::HistoryRewound
            }
            crate::context_projection::RecomputeCause::PolicyChanged => {
                crate::ssb::FreezePrefixRecomputeCause::PolicyChanged
            }
        };
        Self {
            kind: CacheEconomicsEventKind::PrefixRecomputed(cause),
            ttl_secs: None,
            breakeven_reads: 0,
            miss_rate: 0.0,
            mutation_hash: boundary_hash,
            call_class: call_class.to_string(),
            ts_ms: now_ms(),
        }
    }
}

fn now_ms() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttl_tier_long_constants() {
        assert_eq!(TtlTier::Long.ttl_secs(), Some(TTL_LONG_SECS));
        assert_eq!(TtlTier::Long.breakeven_reads(), BREAKEVEN_READS_1H);
        assert_eq!(TtlTier::Long.label(), "1h");
    }

    #[test]
    fn ttl_tier_short_constants() {
        assert_eq!(TtlTier::Short.ttl_secs(), Some(TTL_SHORT_SECS));
        assert_eq!(TtlTier::Short.breakeven_reads(), BREAKEVEN_READS_5M);
        assert_eq!(TtlTier::Short.label(), "5m");
    }

    #[test]
    fn ttl_tier_no_cache() {
        assert_eq!(TtlTier::NoCache.ttl_secs(), None);
        assert_eq!(TtlTier::NoCache.breakeven_reads(), 0);
        assert_eq!(TtlTier::NoCache.label(), "default");
    }

    #[test]
    fn planner_local_policy_no_cache() {
        let gap = GapStats { mean_gap_ms: 1_000, sample_count: 5 };
        assert_eq!(
            CacheEconomicsPlanner::plan_ttl(gap, ContextPolicy::Local, true),
            TtlTier::NoCache
        );
    }

    #[test]
    fn planner_cloud_stable_prefix_short_gap_gives_long() {
        let gap = GapStats { mean_gap_ms: 30_000, sample_count: 3 }; // 30s < 10min
        assert_eq!(
            CacheEconomicsPlanner::plan_ttl(gap, ContextPolicy::Cloud, true),
            TtlTier::Long
        );
    }

    #[test]
    fn planner_cloud_unstable_prefix_gives_short() {
        let gap = GapStats { mean_gap_ms: 30_000, sample_count: 3 };
        assert_eq!(
            CacheEconomicsPlanner::plan_ttl(gap, ContextPolicy::Cloud, false),
            TtlTier::Short
        );
    }

    #[test]
    fn planner_cloud_long_gap_gives_short() {
        let gap = GapStats { mean_gap_ms: 700_000, sample_count: 3 }; // > 10min
        assert_eq!(
            CacheEconomicsPlanner::plan_ttl(gap, ContextPolicy::Cloud, true),
            TtlTier::Short
        );
    }

    #[test]
    fn planner_no_gap_history_gives_short() {
        let gap = GapStats { mean_gap_ms: 0, sample_count: 0 };
        assert_eq!(
            CacheEconomicsPlanner::plan_ttl(gap, ContextPolicy::Cloud, true),
            TtlTier::Short // no history = conservative default
        );
    }

    #[test]
    fn guard_local_policy_ok() {
        let snap = CacheCheckSnapshot { hits: 0, misses: 100 };
        assert_eq!(
            PrefixInvarianceGuard::check_miss_rate(snap, ContextPolicy::Local),
            GuardVerdict::Ok
        );
    }

    #[test]
    fn guard_cloud_zero_total_ok() {
        let snap = CacheCheckSnapshot { hits: 0, misses: 0 };
        assert_eq!(
            PrefixInvarianceGuard::check_miss_rate(snap, ContextPolicy::Cloud),
            GuardVerdict::Ok
        );
    }

    #[test]
    fn guard_cloud_below_threshold_ok() {
        // 9 hits, 1 miss = 10% miss which is == threshold (not >), so Ok.
        let snap = CacheCheckSnapshot { hits: 9, misses: 1 };
        assert_eq!(
            PrefixInvarianceGuard::check_miss_rate(snap, ContextPolicy::Cloud),
            GuardVerdict::Ok
        );
    }

    #[test]
    fn guard_cloud_above_threshold_tripped() {
        // 8 hits, 2 misses = 20% miss > 10% threshold.
        let snap = CacheCheckSnapshot { hits: 8, misses: 2 };
        assert_eq!(
            PrefixInvarianceGuard::check_miss_rate(snap, ContextPolicy::Cloud),
            GuardVerdict::Tripped
        );
    }

    #[test]
    fn max_tokens_clamp_three_fields() {
        // 3-field schema: 3 * 32 = 96, well below the naive 4096 default.
        assert_eq!(max_tokens_clamp(3), 96);
    }

    #[test]
    fn max_tokens_clamp_zero_fields_floor() {
        assert_eq!(max_tokens_clamp(0), MAX_TOKENS_PER_FIELD);
    }

    #[test]
    fn prefix_validator_clean_text() {
        let text = "You are a coding assistant. Always write safe, idiomatic Rust.";
        assert!(assert_prefix_has_no_mutable_tokens(text).is_ok());
    }

    #[test]
    fn prefix_validator_detects_timestamp() {
        let text = "Session started at 2026-06-12. Please assist.";
        let result = assert_prefix_has_no_mutable_tokens(text);
        assert!(result.is_err(), "should detect ISO date");
        let mutations = result.unwrap_err();
        assert!(mutations.iter().any(|m| m.kind == MutationKind::Timestamp));
    }

    #[test]
    fn prefix_validator_detects_uuid() {
        let text = "Request-id: 550e8400-e29b-41d4-a716-446655440000 for routing.";
        let result = assert_prefix_has_no_mutable_tokens(text);
        assert!(result.is_err(), "should detect UUID");
        let mutations = result.unwrap_err();
        assert!(mutations.iter().any(|m| m.kind == MutationKind::Uuid));
    }

    #[test]
    fn prefix_validator_detects_endpoint() {
        let text = "Connect to localhost:8100 for inference.";
        let result = assert_prefix_has_no_mutable_tokens(text);
        assert!(result.is_err(), "should detect mutable endpoint");
        let mutations = result.unwrap_err();
        assert!(mutations.iter().any(|m| m.kind == MutationKind::MutableEndpoint));
    }

    #[test]
    fn prefix_validator_detects_session_id() {
        // F-3 WARN fix: ELAI session ids like "s-107b022a" must be flagged.
        let text = "Session s-107b022a telemetry prefix.";
        let result = assert_prefix_has_no_mutable_tokens(text);
        assert!(result.is_err(), "should detect ELAI session id s-107b022a");
        let mutations = result.unwrap_err();
        assert!(
            mutations.iter().any(|m| m.kind == MutationKind::SessionId),
            "at least one SessionId mutation must be detected"
        );
    }

    #[test]
    fn prefix_validator_session_id_below_min_hex_passes() {
        // "s-ab12" (4 hex chars) is below the 6-hex minimum and must NOT be flagged.
        let text = "Token s-ab12 is too short to be a session id.";
        let result = assert_prefix_has_no_mutable_tokens(text);
        assert!(
            result.is_ok(),
            "short s-XXXX token (< 6 hex) must NOT be flagged as SessionId"
        );
    }

    #[test]
    fn payload_constructors_set_kind() {
        let p = CacheEconomicsEventPayload::ttl_planned(TtlTier::Long, "grounding");
        assert_eq!(p.kind, CacheEconomicsEventKind::TtlPlanned);
        assert_eq!(p.ttl_secs, Some(3600));
        assert_eq!(p.breakeven_reads, BREAKEVEN_READS_1H);

        let g = CacheEconomicsEventPayload::guard_tripped(0.15, "grounding");
        assert_eq!(g.kind, CacheEconomicsEventKind::HitRateGuardTripped);
        assert!((g.miss_rate - 0.15).abs() < 1e-9);

        let d = CacheEconomicsEventPayload::prefix_diff_dumped([1u8; 8], "grounding");
        assert_eq!(d.kind, CacheEconomicsEventKind::PrefixDiffDumped);
        assert_eq!(d.mutation_hash, [1u8; 8]);
    }
}
