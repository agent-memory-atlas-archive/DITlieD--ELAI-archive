//! J-28 — token ledger / usage unification contract tests.
//!
//! Covers the properties `crates/orchestrator/src/usage_ledger.rs`'s own
//! `#[cfg(test)]` module already asserts in isolation, PLUS the
//! cross-module negatives the plan's observable_effect names explicitly:
//!   (a) T2/R10: a fold that sums cloud+local WITHOUT the lane key must
//!       be impossible to express honestly -- the rollup key always
//!       carries the lane, so an identical-tokens cloud+local pair never
//!       collapses to one entry (mirrors the in-crate unit test, verified
//!       again here through the crate's PUBLIC surface only).
//!   (b) T4: `SpendAccumulator::for_model("claude-haiku-4-5")` returns
//!       haiku rates, NOT sonnet; an unknown model id returns an explicit
//!       no-rate accumulator, never a silent sonnet clone.
//!   (c) T6/R2: a fixture stream with no reported usage yields
//!       `Kind::EstimateApplied`, never a `TurnRecorded` row with
//!       all-zero counters.
//!   (d) T5: a session total is the SUM across multiple rows, not the
//!       most-recent row (closes audit finding C-2); `None + None` stays
//!       `None`, never a fabricated `Some(0)`.

use elai_orchestrator::cloud_mode::SpendAccumulator;
use elai_orchestrator::ssb::{
    UsageLane, UsageSemanticsTag, UsageTurnEventKind, UsageTurnEventPayload,
};
use elai_orchestrator::usage_ledger::{
    resolve_turn_usage, rollup_usage_turns, sum_optional_micros, NormalizedUsage,
};

fn fixture_row(
    lane: UsageLane,
    model: &str,
    input_tokens: u32,
    output_tokens: u32,
    est_cost_micros: u64,
    kind: UsageTurnEventKind,
) -> UsageTurnEventPayload {
    UsageTurnEventPayload {
        identity: "local".to_string(),
        lane,
        backend: "contract-fixture".to_string(),
        model: model.to_string(),
        task_id: Some("t-1".to_string()),
        session_id: Some("s-1".to_string()),
        input_tokens,
        output_tokens,
        cache_creation_tokens: 0,
        cache_read_tokens: 0,
        est_cost_micros,
        elapsed_ms: 5,
        semantics_tag: UsageSemanticsTag::FullPrompt,
        kind,
        ts_ms: 0,
    }
}

// (a) R10 -- NEGATIVE: identical numeric tokens across cloud+local never
// collapse into one rollup entry; the naive-sum path is not expressible.
#[test]
fn rollup_never_sums_cloud_and_local_unlabeled_through_public_surface() {
    let rows = vec![
        fixture_row(UsageLane::Cloud, "claude-sonnet-4-6", 500, 200, 3_000, UsageTurnEventKind::TurnRecorded),
        fixture_row(UsageLane::Local, "claude-sonnet-4-6", 500, 200, 0, UsageTurnEventKind::TurnRecorded),
    ];
    let rollup = rollup_usage_turns(&rows);
    assert_eq!(rollup.len(), 2, "cloud and local rows with identical tokens must stay two entries");
    let total_input: u64 = rollup.values().map(|e| e.input_tokens).sum();
    assert_eq!(total_input, 1_000, "sum across the two SEPARATE entries, never one merged 500");
    for entry in rollup.values() {
        assert_eq!(entry.input_tokens, 500, "no entry absorbed the other lane's tokens");
    }
}

// (b) T4 -- NEGATIVE: haiku != sonnet; unknown model = explicit no-rate.
#[test]
fn for_model_prices_haiku_distinctly_from_sonnet() {
    let mut haiku = SpendAccumulator::for_model("claude-haiku-4-5");
    let mut sonnet = SpendAccumulator::for_claude_sonnet_4_6();
    assert!(haiku.snapshot().priced);
    haiku.accumulate(1_000_000, 1_000_000);
    sonnet.accumulate(1_000_000, 1_000_000);
    assert!(
        haiku.finalize() < sonnet.finalize(),
        "haiku (cheapest tier) must cost less than sonnet for identical usage"
    );
}

#[test]
fn for_model_unknown_id_never_silently_clones_sonnet_rate() {
    let mut unknown = SpendAccumulator::for_model("gpt-4o");
    assert!(!unknown.snapshot().priced, "a non-Anthropic-family id has no known rate");
    unknown.accumulate(1_000_000, 1_000_000);
    assert!(
        (unknown.finalize()).abs() < f64::EPSILON,
        "an unpriced accumulator must total exactly $0.0"
    );
}

// (c) T6/R2 -- NEGATIVE: a zero-usage stream never yields a silent
// all-zero TurnRecorded row.
#[test]
fn zero_reported_usage_never_yields_turn_recorded() {
    let (est_in, est_out, kind) = resolve_turn_usage(
        0,
        0,
        "the assistant was asked to summarize a long document",
        "here is the summary",
    );
    assert_eq!(kind, UsageTurnEventKind::EstimateApplied);
    assert!(est_in > 0);
    assert!(est_out > 0);
    assert_ne!(
        kind,
        UsageTurnEventKind::TurnRecorded,
        "a (0,0) reported stream must never be labeled TurnRecorded"
    );
}

// (d) T5 -- session total is the SUM across rows, not the most-recent
// row; None+None stays None.
#[test]
fn session_total_sums_multiple_rows_not_most_recent_only() {
    let rows = vec![
        fixture_row(UsageLane::Local, "sidecar", 1_000, 500, 0, UsageTurnEventKind::TurnRecorded),
        fixture_row(UsageLane::Local, "sidecar", 2_000, 800, 0, UsageTurnEventKind::TurnRecorded),
        fixture_row(UsageLane::Local, "sidecar", 500, 100, 0, UsageTurnEventKind::TurnRecorded),
    ];
    let rollup = rollup_usage_turns(&rows);
    let entry = rollup.values().next().expect("one grouped entry");
    // The pre-J-28 bug (audit C-2): task_usd = most-recent row only. The
    // most-recent row alone is 500; the true sum is 3,500.
    assert!(entry.input_tokens > 500, "must exceed the most-recent single row");
    assert_eq!(entry.input_tokens, 3_500);
    assert_eq!(entry.output_tokens, 1_400);
}

#[test]
fn sum_optional_micros_none_plus_none_stays_none() {
    assert_eq!(sum_optional_micros(None, None), None);
    assert_eq!(sum_optional_micros(None, Some(10)), Some(10));
    assert_eq!(sum_optional_micros(Some(10), None), Some(10));
    assert_eq!(sum_optional_micros(Some(10), Some(5)), Some(15));
}

// T2 -- NormalizedUsage carries the correct lane + semantics per side.
#[test]
fn normalized_usage_lane_and_semantics_are_distinct_per_lane() {
    let raw = elai_core::TokenUsage {
        input_tokens: 10,
        output_tokens: 5,
        cache_creation_input_tokens: 0,
        cache_read_input_tokens: 0,
    };
    let cloud = NormalizedUsage::cloud(raw);
    let local = NormalizedUsage::local(raw);
    assert_eq!(cloud.lane, UsageLane::Cloud);
    assert_eq!(local.lane, UsageLane::Local);
    assert_ne!(
        cloud.semantics_tag, local.semantics_tag,
        "cloud (SuffixOnly) and local (FullPrompt) semantics must never collapse to one tag"
    );
}
