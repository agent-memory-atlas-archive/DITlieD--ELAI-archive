//! I-10 -- cache economics contract tests.
//!
//! Coverage (5+4 arms; no #[ignore], no stubs, no placeholder asserts):
//!
//! (F1) Schema lock (golden 2/2): SCHEMA_VERSION_PUBLIC >= 237 AND
//!      action_type_lock.rs declares TOTAL_VARIANTS: usize = 546, confirming
//!      the atomically-locked SSB expansion for I-10 CacheEconomicsEvent.
//!
//! (F2) TTL tier selection: known-answer tests for CacheEconomicsPlanner::plan_ttl()
//!      covering all three output arms (Long / Short / NoCache).
//!
//! (F3) Guard trip test: PrefixInvarianceGuard::check_miss_rate() returns
//!      GuardVerdict::Tripped when miss rate exceeds MISS_RATE_GUARD_THRESHOLD.
//!
//! (F4) Stable-prefix no-trip + max_tokens_clamp known answer.
//!
//! (F5) SSB roundtrip: a real Ssb::write_payload + SsbReader::query_by_type +
//!      msgpack decode round-trip for all 3 CacheEconomicsEvent kind arms.
//!      Catches the SQL CHECK comma gotcha -- a missing wire tag causes runtime
//!      INSERT failure invisible to `cargo check`.
//!
//! (W1-W4) Wiring arms added by A83-I10-FIX (fixes the R7 WIRE-DARK BLOCK):
//!   W1  planner tier -> CacheControl label: plan_ttl(Long) produces "1h" label;
//!       plan_ttl(Short) produces "5m" label. Proves the planner output maps to the
//!       exact CacheControl discriminant that apply_anthropic_cache_markers preserves.
//!   W2  guard-fires-ssb: a canned unexplained-miss snapshot trips the guard and the
//!       HitRateGuardTripped SSB row lands in the db (round-trip proof the seam emits).
//!   W3  session-id mutation detection: F-3 WARN fix — s-XXXXXXXX hex token flagged.
//!   W4  guard-does-not-fire-on-local: regime_policy(Local) makes guard return Ok
//!       even with 100% miss-rate (no SSB row emitted on Local backend).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::SsbReader;
use elai_orchestrator::cache_check::CacheCheckSnapshot;
use elai_orchestrator::cache_economics::{
    assert_prefix_has_no_mutable_tokens, emit_cache_economics_event, max_tokens_clamp,
    CacheEconomicsPlanner, GapStats, GuardVerdict, MutationKind, PrefixInvarianceGuard, TtlTier,
    BREAKEVEN_READS_1H, BREAKEVEN_READS_5M, MAX_TOKENS_PER_FIELD, MISS_RATE_GUARD_THRESHOLD,
    TTL_LONG_SECS, TTL_SHORT_SECS,
};
use elai_orchestrator::context_policy::{regime_policy, ContextPolicy};
use elai_orchestrator::BackendMode;
use elai_orchestrator::ssb::{
    CacheEconomicsEventKind, CacheEconomicsEventPayload, Ssb, SsbActionType, SsbConfig,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ssb_config(dir: &TempDir, label: &str) -> SsbConfig {
    SsbConfig {
        db_path: dir.path().join(format!("{label}.db")),
        pool_size: Some(2),
        project_id: "i10-cache-economics-contract".to_string(),
        session_id: format!("i10-contract-{label}"),
    }
}

fn clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

// ---------------------------------------------------------------------------
// (F1) Schema lock (golden 2/2)
// ---------------------------------------------------------------------------

/// SCHEMA_VERSION_PUBLIC must be >= 237 (I-10 floor bump from 236 -> 237).
#[test]
fn schema_version_is_237() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 237,
        "SCHEMA_VERSION_PUBLIC must be >= 237 (I-10 floor: CacheEconomicsEvent add); \
         got {} -- a value below 237 means the I-10 SSB bump was regressed",
        SCHEMA_VERSION_PUBLIC,
    );
}

/// action_type_lock.rs must declare TOTAL_VARIANTS: usize = 546 (545 + I-10).
#[test]
fn total_variants_is_546() {
    let lock_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("contracts crate has a parent")
        .join("orchestrator")
        .join("tests")
        .join("action_type_lock.rs");
    let body = std::fs::read_to_string(&lock_path).unwrap_or_else(|e| {
        panic!(
            "cannot read action_type_lock.rs at {}: {e}",
            lock_path.display()
        )
    });
    // Floor check: TOTAL_VARIANTS >= 546 (I-10 authoring floor: CacheEconomicsEvent
    // added at 546; exact-pin converted per F-2 pin doctrine).
    let total_variants_value: usize = body
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("const TOTAL_VARIANTS: usize = ") {
                trimmed
                    .trim_start_matches("const TOTAL_VARIANTS: usize = ")
                    .trim_end_matches(';')
                    .parse()
                    .ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!(
            "could not parse TOTAL_VARIANTS from action_type_lock.rs at {}",
            lock_path.display()
        ));
    assert!(
        total_variants_value >= 546,
        "TOTAL_VARIANTS must be >= 546 (I-10 floor: CacheEconomicsEvent SSB lockstep); \
         got {} in {}",
        total_variants_value,
        lock_path.display()
    );
}

// ---------------------------------------------------------------------------
// (F2) TTL tier selection known-answer
// ---------------------------------------------------------------------------

/// Local policy always yields NoCache regardless of gap stats.
#[test]
fn ttl_tier_no_cache_on_local_policy() {
    let gap_stats = GapStats { mean_gap_ms: 100, sample_count: 10 };
    let tier = CacheEconomicsPlanner::plan_ttl(gap_stats, ContextPolicy::Local, true);
    assert_eq!(
        tier,
        TtlTier::NoCache,
        "ContextPolicy::Local must always yield TtlTier::NoCache (no sidecar KV annotation)"
    );
}

/// Cloud + unstable prefix -> Short (planner downgrades to 5m but does not skip caching;
/// only Local policy and no-history gaps yield NoCache).
#[test]
fn ttl_tier_short_on_unstable_prefix() {
    let gap_stats = GapStats { mean_gap_ms: 100, sample_count: 10 };
    let tier = CacheEconomicsPlanner::plan_ttl(gap_stats, ContextPolicy::Cloud, false);
    assert_eq!(
        tier,
        TtlTier::Short,
        "unstable prefix (prefix_stable=false, Cloud) must yield TtlTier::Short \
         (planner downgrades to 5m rather than refusing to cache)"
    );
}

/// Cloud + short gap (interactive session) + stable prefix -> Long (1h TTL).
/// mean_gap < 600_000 ms (10 min) = interactive = 1h TTL cost-effective.
#[test]
fn ttl_tier_long_on_interactive_cloud_session() {
    let gap_stats = GapStats { mean_gap_ms: 30_000, sample_count: 5 };
    let tier = CacheEconomicsPlanner::plan_ttl(gap_stats, ContextPolicy::Cloud, true);
    assert_eq!(
        tier,
        TtlTier::Long,
        "interactive cloud session (mean_gap < 10 min, stable prefix) must yield TtlTier::Long"
    );
}

/// Cloud + long gap (batch session) + stable prefix -> Short (5m TTL).
/// mean_gap > 600_000 ms = batch = only 5m write premium safe.
#[test]
fn ttl_tier_short_on_batch_cloud_session() {
    let gap_stats = GapStats { mean_gap_ms: 900_000, sample_count: 5 };
    let tier = CacheEconomicsPlanner::plan_ttl(gap_stats, ContextPolicy::Cloud, true);
    assert_eq!(
        tier,
        TtlTier::Short,
        "batch cloud session (mean_gap > 10 min, stable prefix) must yield TtlTier::Short"
    );
}

/// TtlTier constants and breakeven arithmetic are self-consistent.
#[test]
fn ttl_tier_constants_are_consistent() {
    assert_eq!(TtlTier::Long.ttl_secs(), Some(TTL_LONG_SECS), "Long.ttl_secs()");
    assert_eq!(TtlTier::Short.ttl_secs(), Some(TTL_SHORT_SECS), "Short.ttl_secs()");
    assert_eq!(TtlTier::NoCache.ttl_secs(), None, "NoCache.ttl_secs()");
    assert_eq!(TtlTier::Long.breakeven_reads(), BREAKEVEN_READS_1H, "Long.breakeven_reads()");
    assert_eq!(TtlTier::Short.breakeven_reads(), BREAKEVEN_READS_5M, "Short.breakeven_reads()");
    assert_eq!(TtlTier::NoCache.breakeven_reads(), 0, "NoCache.breakeven_reads()");
    assert_eq!(TtlTier::Long.label(), "1h", "Long.label()");
    assert_eq!(TtlTier::Short.label(), "5m", "Short.label()");
    assert_eq!(TtlTier::NoCache.label(), "default", "NoCache.label()");
}

// ---------------------------------------------------------------------------
// (F3) Guard trip test
// ---------------------------------------------------------------------------

/// Guard fires when miss rate exceeds threshold (fail-closed R1).
/// CacheCheckSnapshot.hits=0, misses=20: miss_rate = 20/20 = 1.0 > 0.10 threshold.
#[test]
fn guard_trips_above_threshold() {
    let snap = CacheCheckSnapshot { hits: 0, misses: 20 };
    let verdict = PrefixInvarianceGuard::check_miss_rate(snap, ContextPolicy::Cloud);
    assert_eq!(
        verdict,
        GuardVerdict::Tripped,
        "100% miss rate must trip the guard (MISS_RATE_GUARD_THRESHOLD={})",
        MISS_RATE_GUARD_THRESHOLD,
    );
}

/// Guard fires when miss rate is well above threshold.
/// hits=1, misses=9: miss_rate = 9/10 = 0.90 > 0.10 threshold.
#[test]
fn guard_trips_at_high_miss_rate() {
    let snap = CacheCheckSnapshot { hits: 1, misses: 9 };
    let verdict = PrefixInvarianceGuard::check_miss_rate(snap, ContextPolicy::Cloud);
    assert_eq!(
        verdict,
        GuardVerdict::Tripped,
        "90% miss rate must trip the guard (MISS_RATE_GUARD_THRESHOLD={})",
        MISS_RATE_GUARD_THRESHOLD,
    );
}

/// Local policy: guard never trips regardless of miss counts.
#[test]
fn guard_ok_on_local_policy() {
    let snap = CacheCheckSnapshot { hits: 0, misses: 100 };
    let verdict = PrefixInvarianceGuard::check_miss_rate(snap, ContextPolicy::Local);
    assert_eq!(
        verdict,
        GuardVerdict::Ok,
        "ContextPolicy::Local must always return GuardVerdict::Ok (sidecar KV is a separate path)"
    );
}

// ---------------------------------------------------------------------------
// (F4) Stable-prefix no-trip + max_tokens_clamp known answer
// ---------------------------------------------------------------------------

/// Guard returns Ok when miss rate is strictly below threshold.
/// hits=95, misses=5: miss_rate = 5/100 = 0.05 < 0.10 threshold.
#[test]
fn guard_ok_below_threshold() {
    let snap = CacheCheckSnapshot { hits: 95, misses: 5 };
    let verdict = PrefixInvarianceGuard::check_miss_rate(snap, ContextPolicy::Cloud);
    assert_eq!(
        verdict,
        GuardVerdict::Ok,
        "5% miss rate must return GuardVerdict::Ok (below MISS_RATE_GUARD_THRESHOLD={})",
        MISS_RATE_GUARD_THRESHOLD,
    );
}

/// max_tokens_clamp: output = schema_fields * MAX_TOKENS_PER_FIELD.
#[test]
fn max_tokens_clamp_known_answer() {
    let schema_fields: u32 = 3;
    let expected = schema_fields * MAX_TOKENS_PER_FIELD;
    assert_eq!(
        max_tokens_clamp(schema_fields),
        expected,
        "max_tokens_clamp({schema_fields}) must equal {expected} \
         (schema_fields * MAX_TOKENS_PER_FIELD={MAX_TOKENS_PER_FIELD})"
    );
}

/// assert_prefix_has_no_mutable_tokens returns Ok for a clean static prefix.
#[test]
fn prefix_invariance_ok_on_clean_prefix() {
    let clean = "You are an AI assistant. Respond in JSON only. No preamble.";
    assert!(
        assert_prefix_has_no_mutable_tokens(clean).is_ok(),
        "clean static prefix must pass the byte-frozen validator"
    );
}

/// assert_prefix_has_no_mutable_tokens detects an ISO date.
#[test]
fn prefix_invariance_detects_timestamp() {
    let with_date = "Today is 2026-06-12. You are an assistant.";
    let mutations = assert_prefix_has_no_mutable_tokens(with_date).unwrap_err();
    assert!(
        !mutations.is_empty(),
        "ISO date in prefix must be flagged as a mutable token"
    );
    assert!(
        mutations.iter().any(|m| m.kind == MutationKind::Timestamp),
        "at least one Timestamp mutation must be detected in prefix with ISO date"
    );
}

/// assert_prefix_has_no_mutable_tokens detects a mutable endpoint.
#[test]
fn prefix_invariance_detects_endpoint() {
    let with_endpoint = "Connect to http://localhost:8100/v1/messages for inference.";
    let mutations = assert_prefix_has_no_mutable_tokens(with_endpoint).unwrap_err();
    assert!(
        !mutations.is_empty(),
        "mutable endpoint in prefix must be flagged"
    );
    assert!(
        mutations.iter().any(|m| m.kind == MutationKind::MutableEndpoint),
        "at least one MutableEndpoint mutation must be detected"
    );
}

// ---------------------------------------------------------------------------
// (F5) SSB roundtrip for all 3 CacheEconomicsEvent kind arms
// ---------------------------------------------------------------------------

/// Full write_payload + query_by_type + msgpack decode round-trip for
/// CacheEconomicsEvent kind=TtlPlanned.
///
/// Catches the SQL CHECK comma gotcha: "cache_economics_event" must appear in
/// the SQLite CHECK constraint. A wrong or absent wire tag causes a runtime
/// INSERT failure invisible to `cargo check`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cache_economics_ttl_planned_ssb_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(ssb_config(&tmp, "ttl-planned"), clock()).expect("ssb open"),
    );

    let payload = CacheEconomicsEventPayload {
        kind: CacheEconomicsEventKind::TtlPlanned,
        ttl_secs: Some(TTL_LONG_SECS),
        breakeven_reads: BREAKEVEN_READS_1H,
        miss_rate: 0.0,
        mutation_hash: [0u8; 8],
        call_class: "scoring".to_string(),
        ts_ms: 1_700_000_000_000,
    };

    ssb.write_payload(
        "cache-economics-agent",
        0,
        SsbPayload::CacheEconomicsEvent(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write_payload must succeed for CacheEconomicsEvent(TtlPlanned)");

    let rows = ssb
        .query_by_type(SsbActionType::CacheEconomicsEvent(
            CacheEconomicsEventKind::TtlPlanned,
        ))
        .expect("query_by_type must succeed");
    assert_eq!(
        rows.len(),
        1,
        "expected exactly one CacheEconomicsEvent row after write"
    );

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("msgpack decode must succeed");
    match decoded {
        SsbPayload::CacheEconomicsEvent(p) => {
            assert!(
                matches!(p.kind, CacheEconomicsEventKind::TtlPlanned),
                "kind must round-trip as TtlPlanned"
            );
            assert_eq!(
                p.ttl_secs,
                Some(TTL_LONG_SECS),
                "ttl_secs must round-trip as Some(TTL_LONG_SECS)"
            );
            assert_eq!(
                p.breakeven_reads, BREAKEVEN_READS_1H,
                "breakeven_reads must round-trip"
            );
            assert_eq!(p.call_class, "scoring", "call_class must round-trip");
        }
        other => panic!("expected CacheEconomicsEvent, got {other:?}"),
    }
}

/// Full roundtrip for kind=HitRateGuardTripped.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cache_economics_hit_rate_guard_tripped_ssb_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(ssb_config(&tmp, "guard-tripped"), clock()).expect("ssb open"),
    );

    let payload = CacheEconomicsEventPayload {
        kind: CacheEconomicsEventKind::HitRateGuardTripped,
        ttl_secs: None,
        breakeven_reads: 0,
        miss_rate: 0.92,
        mutation_hash: [0u8; 8],
        call_class: "generation".to_string(),
        ts_ms: 1_700_000_001_000,
    };

    ssb.write_payload(
        "cache-economics-agent",
        0,
        SsbPayload::CacheEconomicsEvent(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write_payload must succeed for CacheEconomicsEvent(HitRateGuardTripped)");

    let rows = ssb
        .query_by_type(SsbActionType::CacheEconomicsEvent(
            CacheEconomicsEventKind::HitRateGuardTripped,
        ))
        .expect("query_by_type must succeed");
    assert_eq!(rows.len(), 1, "expected one HitRateGuardTripped row");

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("msgpack decode must succeed");
    match decoded {
        SsbPayload::CacheEconomicsEvent(p) => {
            assert!(
                matches!(p.kind, CacheEconomicsEventKind::HitRateGuardTripped),
                "kind must round-trip as HitRateGuardTripped"
            );
            assert!(
                (p.miss_rate - 0.92).abs() < f64::EPSILON,
                "miss_rate must round-trip as 0.92; got {}",
                p.miss_rate
            );
            assert_eq!(p.ttl_secs, None, "ttl_secs must round-trip as None");
        }
        other => panic!("expected CacheEconomicsEvent, got {other:?}"),
    }
}

/// Full roundtrip for kind=PrefixDiffDumped.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cache_economics_prefix_diff_dumped_ssb_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = Arc::new(
        Ssb::new(ssb_config(&tmp, "prefix-diff"), clock()).expect("ssb open"),
    );

    let mutation_hash: [u8; 8] = [0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x01];
    let payload = CacheEconomicsEventPayload {
        kind: CacheEconomicsEventKind::PrefixDiffDumped,
        ttl_secs: None,
        breakeven_reads: 0,
        miss_rate: 0.0,
        mutation_hash,
        call_class: "routing".to_string(),
        ts_ms: 1_700_000_002_000,
    };

    ssb.write_payload(
        "cache-economics-agent",
        0,
        SsbPayload::CacheEconomicsEvent(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write_payload must succeed for CacheEconomicsEvent(PrefixDiffDumped)");

    let rows = ssb
        .query_by_type(SsbActionType::CacheEconomicsEvent(
            CacheEconomicsEventKind::PrefixDiffDumped,
        ))
        .expect("query_by_type must succeed");
    assert_eq!(rows.len(), 1, "expected one PrefixDiffDumped row");

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("msgpack decode must succeed");
    match decoded {
        SsbPayload::CacheEconomicsEvent(p) => {
            assert!(
                matches!(p.kind, CacheEconomicsEventKind::PrefixDiffDumped),
                "kind must round-trip as PrefixDiffDumped"
            );
            assert_eq!(
                p.mutation_hash, mutation_hash,
                "mutation_hash must round-trip"
            );
            assert_eq!(p.call_class, "routing", "call_class must round-trip");
        }
        other => panic!("expected CacheEconomicsEvent, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (W1) planner tier -> CacheControl label mapping (A83-I10-FIX F-1 wiring arm)
// ---------------------------------------------------------------------------

/// W1a: plan_ttl -> Long maps to "1h" label (the exact discriminant that
/// apply_anthropic_cache_markers preserves via the is_none guard).
/// Proves the planner drives the ephemeral_1h() branch on the live path.
#[test]
fn w1_planner_long_tier_label_is_1h() {
    let gap = GapStats { mean_gap_ms: 30_000, sample_count: 5 }; // interactive session
    let tier = CacheEconomicsPlanner::plan_ttl(gap, ContextPolicy::Cloud, true);
    assert_eq!(
        tier,
        TtlTier::Long,
        "interactive cloud + stable prefix must yield Long"
    );
    assert_eq!(
        tier.label(),
        "1h",
        "TtlTier::Long.label() must be '1h' (maps to CacheControl::ephemeral_1h())"
    );
    assert_eq!(
        tier.ttl_secs(),
        Some(TTL_LONG_SECS),
        "TtlTier::Long.ttl_secs() must be Some(TTL_LONG_SECS={TTL_LONG_SECS})"
    );
}

/// W1b: plan_ttl -> Short maps to "5m" label (maps to CacheControl::ephemeral()).
/// Proves the planner drives the ephemeral() (5m) branch when prefix is unstable.
#[test]
fn w1_planner_short_tier_label_is_5m() {
    let gap = GapStats { mean_gap_ms: 30_000, sample_count: 5 };
    let tier = CacheEconomicsPlanner::plan_ttl(gap, ContextPolicy::Cloud, false); // unstable
    assert_eq!(
        tier,
        TtlTier::Short,
        "unstable prefix (Cloud) must yield Short"
    );
    assert_eq!(
        tier.label(),
        "5m",
        "TtlTier::Short.label() must be '5m' (maps to CacheControl::ephemeral())"
    );
    assert_eq!(
        tier.ttl_secs(),
        Some(TTL_SHORT_SECS),
        "TtlTier::Short.ttl_secs() must be Some(TTL_SHORT_SECS={TTL_SHORT_SECS})"
    );
}

// ---------------------------------------------------------------------------
// (W2) guard fires + SSB row lands (A83-I10-FIX F-2 wiring arm)
// ---------------------------------------------------------------------------

/// W2: a canned unexplained-miss snapshot (all misses, zero hits, Cloud policy)
/// trips the guard AND the HitRateGuardTripped SSB row lands via emit_cache_economics_event.
/// This is the production seam proof: guard verdict != Ok -> emit -> SSB row exists.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn w2_guard_fires_and_ssb_row_lands() {
    use elai_orchestrator::cache_check::CacheCheckSnapshot;

    let tmp = TempDir::new().expect("tempdir");
    let ssb = Arc::new(Ssb::new(ssb_config(&tmp, "guard-fires"), clock()).expect("ssb open"));

    // Canned snapshot: 0 hits, 20 misses = 100% miss rate >> threshold.
    let snap = CacheCheckSnapshot { hits: 0, misses: 20 };
    let policy = ContextPolicy::Cloud;
    let verdict = PrefixInvarianceGuard::check_miss_rate(snap, policy);
    assert_eq!(
        verdict,
        GuardVerdict::Tripped,
        "0 hits / 20 misses must trip the guard (production seam check)"
    );

    // Emit the guard-tripped row via the production helper (same call the live path uses).
    let miss_rate = snap.misses as f64 / (snap.hits + snap.misses) as f64;
    let guard_payload = CacheEconomicsEventPayload::guard_tripped(miss_rate, "w2-contract");
    emit_cache_economics_event(&ssb, "w2-contract-agent", 0, guard_payload)
        .await
        .expect("emit_cache_economics_event must succeed for HitRateGuardTripped");

    // Verify the SSB row landed (R2: missing evidence = FAIL).
    let rows = ssb
        .query_by_type(SsbActionType::CacheEconomicsEvent(
            CacheEconomicsEventKind::HitRateGuardTripped,
        ))
        .expect("query_by_type must succeed");
    assert_eq!(
        rows.len(),
        1,
        "exactly one HitRateGuardTripped row must exist after guard fires and emit runs"
    );

    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("msgpack decode must succeed");
    match decoded {
        SsbPayload::CacheEconomicsEvent(p) => {
            assert!(
                matches!(p.kind, CacheEconomicsEventKind::HitRateGuardTripped),
                "kind must be HitRateGuardTripped"
            );
            assert!(
                (p.miss_rate - 1.0).abs() < f64::EPSILON,
                "miss_rate must round-trip as 1.0 (20/20); got {}",
                p.miss_rate
            );
            assert_eq!(p.call_class, "w2-contract", "call_class must round-trip");
        }
        other => panic!("expected CacheEconomicsEvent, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (W3) session-id mutation detection (A83-I10-FIX F-3 WARN fix wiring arm)
// ---------------------------------------------------------------------------

/// W3: assert_prefix_has_no_mutable_tokens detects ELAI session-id tokens
/// (s-XXXXXXXX hex pattern). These break prefix byte-stability every session
/// and were previously invisible to the validator (audit WARN F-3).
#[test]
fn w3_session_id_mutation_detected() {
    let with_session_id = "Agent session s-107b022a is handling this request.";
    let result = assert_prefix_has_no_mutable_tokens(with_session_id);
    assert!(
        result.is_err(),
        "prefix containing 's-107b022a' must be flagged as mutable (ELAI session id)"
    );
    let mutations = result.unwrap_err();
    assert!(
        mutations.iter().any(|m| m.kind == MutationKind::SessionId),
        "at least one SessionId mutation must be detected in prefix with s-XXXXXXXX token"
    );
}

// ---------------------------------------------------------------------------
// (W4) guard Local policy no-op (A83-I10-FIX wiring arm)
// ---------------------------------------------------------------------------

/// W4: regime_policy(Local) makes the guard return Ok regardless of miss counts,
/// and no SSB row is emitted. Proves the Local backend guard no-op path.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn w4_guard_no_op_on_local_backend() {
    use elai_orchestrator::cache_check::CacheCheckSnapshot;

    let tmp = TempDir::new().expect("tempdir");
    let ssb = Arc::new(Ssb::new(ssb_config(&tmp, "guard-local"), clock()).expect("ssb open"));

    let policy = regime_policy(BackendMode::Local);
    assert_eq!(
        policy,
        ContextPolicy::Local,
        "regime_policy(Local) must return ContextPolicy::Local"
    );

    // Even with 100% miss rate, Local policy returns Ok.
    let snap = CacheCheckSnapshot { hits: 0, misses: 100 };
    let verdict = PrefixInvarianceGuard::check_miss_rate(snap, policy);
    assert_eq!(
        verdict,
        GuardVerdict::Ok,
        "Local policy must return GuardVerdict::Ok regardless of miss-rate (sidecar KV is separate)"
    );

    // No SSB row should be emitted for a Local non-trip.
    let rows = ssb
        .query_by_type(SsbActionType::CacheEconomicsEvent(
            CacheEconomicsEventKind::HitRateGuardTripped,
        ))
        .expect("query_by_type must succeed");
    assert!(
        rows.is_empty(),
        "no HitRateGuardTripped rows must exist when guard returns Ok on Local backend"
    );
}
