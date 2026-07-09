//! J-28: token ledger -- usage unification, per-turn rollup, estimator
//! fallback (T2/T5/T6/T7).
//!
//! Consumes `UsageTurnEvent` rows written at the two production emit seams
//! (`grounding_client.rs::CloudGroundingClient::send_message` for the cloud
//! lane, `runtime_orchestrator.rs::RuntimeOrchestrator::try_complete` for
//! the local lane). Never re-implements `elai_core::TokenUsage` /
//! `UsageTracker` (the per-turn capture) or `elai_providers::pricing` (the
//! sole rate source) -- GD-1.
//!
//! Production reader: `crates/cli/src/usage_cmd.rs::run_usage` (the `elai
//! usage` verb), the sole production caller of [`rollup_usage_turns`].
//!
//! See `workflow/reasoning/J-28-decisions.md` for the FORK-1/2/3 verdicts
//! this module composes with (FORK-1 ROUTE B client-side normalization;
//! FORK-3 the T7 compaction rule lands here, not `context_policy.rs`).

use std::collections::{BTreeMap, VecDeque};
use std::sync::Mutex;

use elai_core::TokenUsage;

use crate::ssb::{UsageLane, UsageSemanticsTag, UsageTurnEventKind, UsageTurnEventPayload};

// ---------------------------------------------------------------------------
// T2 -- NormalizedUsage (ROUTE B: client-side normalization keyed by lane)
// ---------------------------------------------------------------------------

/// A usage sample normalized with its lane + semantics tag BEFORE the SSB
/// row is written. FORK-1 ROUTE B: normalization happens client-side,
/// keyed by which seam produced the sample -- no sidecar/`server.py`
/// touch. Composes, never re-derives, [`elai_core::TokenUsage`].
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedUsage {
    pub raw: TokenUsage,
    pub lane: UsageLane,
    pub semantics_tag: UsageSemanticsTag,
}

impl NormalizedUsage {
    /// Cloud lane: `input_tokens` is the Anthropic-spec uncached suffix.
    #[must_use]
    pub const fn cloud(raw: TokenUsage) -> Self {
        Self {
            raw,
            lane: UsageLane::Cloud,
            semantics_tag: UsageSemanticsTag::SuffixOnly,
        }
    }

    /// Local lane: `input_tokens` is the full prompt sent to the sidecar.
    #[must_use]
    pub const fn local(raw: TokenUsage) -> Self {
        Self {
            raw,
            lane: UsageLane::Local,
            semantics_tag: UsageSemanticsTag::FullPrompt,
        }
    }
}

// ---------------------------------------------------------------------------
// T6 -- estimator fallback (blake3-keyed bounded cache)
// ---------------------------------------------------------------------------

/// Rough chars-per-token ratio for the client-side estimator. No canonical
/// shared tokenizer exists in this codebase to reuse (residue: the same
/// chars/N heuristic is already duplicated per-module -- see
/// `providers::ollama::ollama_token_estimate`,
/// `orchestrator::research_budget::estimate_tokens`); this follows the
/// same established local convention rather than inventing a third shape.
const CHARS_PER_TOKEN_ESTIMATE: usize = 4;

/// Bounded cache capacity. No `lru` crate in this workspace (adding one
/// needs explicit user confirmation per house rule); this is a hand-rolled
/// bounded FIFO cache keyed by a blake3 prefix -- a dependency-free
/// substitute for goose's tiktoken-backed `token_counter.rs` LRU (S-03).
const ESTIMATOR_CACHE_CAPACITY: usize = 256;

static ESTIMATOR_CACHE: Mutex<VecDeque<(u64, u32)>> = Mutex::new(VecDeque::new());

fn blake3_u64_prefix(text: &str) -> u64 {
    let digest = blake3::hash(text.as_bytes());
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// T6 -- rough per-lane token estimate for text the backend did not
/// report usage for. Memoized by a blake3 prefix of the text in a bounded
/// FIFO cache so repeated identical prompts/completions do not re-count.
#[must_use]
pub fn estimate_tokens_cached(text: &str) -> u32 {
    if text.is_empty() {
        return 0;
    }
    let key = blake3_u64_prefix(text);
    let mut cache = ESTIMATOR_CACHE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((_, cached)) = cache.iter().find(|(k, _)| *k == key) {
        return *cached;
    }
    let char_count = text.chars().count();
    let estimate = u32::try_from(char_count / CHARS_PER_TOKEN_ESTIMATE).unwrap_or(u32::MAX);
    if cache.len() >= ESTIMATOR_CACHE_CAPACITY {
        cache.pop_front();
    }
    cache.push_back((key, estimate));
    estimate
}

/// T6 -- decide `TurnRecorded` vs `EstimateApplied` for a completed turn.
/// A real (non-zero) reported usage always wins (`TurnRecorded`). When the
/// backend reported nothing (`0, 0`), the estimator seam fires so the row
/// NEVER carries a silent all-zero `TurnRecorded` (R2: a dead-evidence
/// path that looks like a real zero-cost turn is the worst class of
/// silent failure).
#[must_use]
pub fn resolve_turn_usage(
    reported_input: u32,
    reported_output: u32,
    input_text: &str,
    output_text: &str,
) -> (u32, u32, UsageTurnEventKind) {
    if reported_input > 0 || reported_output > 0 {
        (reported_input, reported_output, UsageTurnEventKind::TurnRecorded)
    } else {
        (
            estimate_tokens_cached(input_text),
            estimate_tokens_cached(output_text),
            UsageTurnEventKind::EstimateApplied,
        )
    }
}

// ---------------------------------------------------------------------------
// T5 -- rollup
// ---------------------------------------------------------------------------

/// Fold key: rows are grouped by identity/model/task/session/lane so a
/// cloud row and a local row NEVER merge unlabeled (R10 -- the SSB row
/// itself is the shared contract). `BTreeMap`-keyed (ELAI bans
/// `std::HashMap` in the orchestrator, determinism rule) so `elai usage`
/// renders in a stable order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UsageRollupKey {
    pub identity: String,
    pub model: String,
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    pub lane: UsageLane,
}

/// Accumulated total for one [`UsageRollupKey`]. `cost_micros` uses
/// Option-semantics (S-02, ported from goose's `sum_optionals` idea):
/// `None` means every folded row's cost was unknown; `Some(total)` sums
/// every row whose cost WAS known (including a real `Some(0)` for a
/// free/local turn). See [`row_cost_micros`] for how a row's cost is
/// classified known vs unknown.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UsageRollupEntry {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost_micros: Option<u64>,
    pub row_count: u32,
}

/// S-02 -- goose's `sum_optionals` idea, ported to THIS layer only (the
/// shipped `u32` `TokenUsage` at `core::usage.rs` is untouched, per the
/// plan's explicit scope). `None` combined with `None` stays `None` (not
/// a fabricated `Some(0)`); either side present sums/wins.
#[must_use]
pub const fn sum_optional_micros(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) | (None, Some(x)) => Some(x),
        (Some(x), Some(y)) => Some(x + y),
    }
}

/// A row's cost, Option-normalized. An `EstimateApplied` row's TOKEN
/// count was estimated, not its dollar rate -- treating its cost as
/// "unknown" (`None`) rather than trusting `est_cost_micros` (which the
/// emit seams set to the estimated-token count priced at the real rate,
/// or 0 when no rate/model is known) keeps the rollup honest: an
/// estimate-derived dollar figure never silently masquerades as a
/// measured one in a session total.
fn row_cost_micros(row: &UsageTurnEventPayload) -> Option<u64> {
    match row.kind {
        UsageTurnEventKind::EstimateApplied => None,
        UsageTurnEventKind::TurnRecorded | UsageTurnEventKind::NormalizationApplied => {
            Some(row.est_cost_micros)
        }
    }
}

fn row_key(row: &UsageTurnEventPayload) -> UsageRollupKey {
    UsageRollupKey {
        identity: row.identity.clone(),
        model: row.model.clone(),
        task_id: row.task_id.clone(),
        session_id: row.session_id.clone(),
        lane: row.lane,
    }
}

/// T5 -- fold a set of `UsageTurnEvent` rows (as read back from the SSB
/// journal) into a per-key rollup. A session/task total is the SUM across
/// every row for that key, never the most-recent row (closes the pre-J-28
/// audit finding C-2: "today task_usd = most-recent row only").
#[must_use]
pub fn rollup_usage_turns(
    rows: &[UsageTurnEventPayload],
) -> BTreeMap<UsageRollupKey, UsageRollupEntry> {
    let mut out: BTreeMap<UsageRollupKey, UsageRollupEntry> = BTreeMap::new();
    for row in rows {
        let entry = out.entry(row_key(row)).or_default();
        entry.input_tokens += u64::from(row.input_tokens);
        entry.output_tokens += u64::from(row.output_tokens);
        entry.cache_creation_tokens += u64::from(row.cache_creation_tokens);
        entry.cache_read_tokens += u64::from(row.cache_read_tokens);
        entry.cost_micros = sum_optional_micros(entry.cost_micros, row_cost_micros(row));
        entry.row_count += 1;
    }
    out
}

// ---------------------------------------------------------------------------
// T7 -- compaction accounting rule (FORK-3: lands here, not context_policy.rs)
// ---------------------------------------------------------------------------

/// T7 -- after a compaction/summarization turn, the summary's OUTPUT
/// tokens become the NEXT turn's input-context tokens, so cumulative
/// input-context keeps growing across the boundary rather than resetting.
///
/// FORK-3 (settled, `workflow/reasoning/J-28-decisions.md`): this rule
/// lives HERE, in the ledger's own accumulation, NOT in
/// `context_policy.rs` -- that module was verified stateless (threshold
/// math only: `trim_threshold`, `effective_context_window`,
/// `regime_policy`) with no per-turn token-delta seam to extend. Landing
/// the rule in two homes would let the two cumulative-growth numbers
/// diverge per lane with nothing catching it (the do-not-do-both rule);
/// this is the single owner.
///
/// No production caller in this changeset: `compact.rs`'s existing
/// `compact_session()` tracks message/removed-message COUNTS, not a
/// per-turn cumulative-token-context number this fn could consume without
/// `compact.rs` itself first growing token-delta tracking (structural
/// threading, out of scope for this fixture-driven Tier-1 build; ABORT-4:
/// never fabricate a live-aggregate consumer to satisfy a checker).
/// Backing row: `workflow/reasoning/J-28-decisions.md` WIRE-DARK REGISTER
/// (the FSM EXECUTING-state write gate does not permit
/// `workflow/sessions/*/stubs.md` from this seat).
// WIRE-DARK[J-28]: unwired pending a follow-on unit threading cumulative
// token state through compact.rs's real trigger site.
#[must_use]
pub const fn apply_compaction_growth(
    cumulative_input_before: u64,
    compaction_output_tokens: u64,
) -> u64 {
    cumulative_input_before.saturating_add(compaction_output_tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            backend: "test-backend".to_string(),
            model: model.to_string(),
            task_id: Some("task-1".to_string()),
            session_id: Some("session-1".to_string()),
            input_tokens,
            output_tokens,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            est_cost_micros,
            elapsed_ms: 10,
            semantics_tag: UsageSemanticsTag::FullPrompt,
            kind,
            ts_ms: 0,
        }
    }

    // T5 EXPECTED_OBSERVATION: a multi-row fixture yields a session total
    // that is the SUM across rows, not the most-recent row (closes C-2).
    #[test]
    fn rollup_sums_across_multiple_rows_not_latest_only() {
        let rows = vec![
            fixture_row(
                UsageLane::Local,
                "sidecar-model",
                100,
                50,
                0,
                UsageTurnEventKind::TurnRecorded,
            ),
            fixture_row(
                UsageLane::Local,
                "sidecar-model",
                200,
                75,
                0,
                UsageTurnEventKind::TurnRecorded,
            ),
        ];
        let rollup = rollup_usage_turns(&rows);
        assert_eq!(rollup.len(), 1, "both rows share the same rollup key");
        let entry = rollup.values().next().expect("one rollup entry");
        assert_eq!(entry.input_tokens, 300, "session_sum > any single row");
        assert_eq!(entry.output_tokens, 125);
        assert_eq!(entry.row_count, 2);
    }

    // T5 NEGATIVE: None+None stays None (not a fabricated 0).
    #[test]
    fn sum_optional_micros_none_plus_none_is_none() {
        assert_eq!(sum_optional_micros(None, None), None);
        assert_eq!(sum_optional_micros(Some(5), None), Some(5));
        assert_eq!(sum_optional_micros(None, Some(7)), Some(7));
        assert_eq!(sum_optional_micros(Some(3), Some(4)), Some(7));
    }

    // T5 / R10 NEGATIVE: a cloud row and a local row NEVER fold into the
    // same rollup entry even with identical numeric token counts.
    #[test]
    fn rollup_never_merges_cloud_and_local_rows_unlabeled() {
        let rows = vec![
            fixture_row(
                UsageLane::Cloud,
                "claude-sonnet-4-6",
                100,
                50,
                1_000,
                UsageTurnEventKind::TurnRecorded,
            ),
            fixture_row(
                UsageLane::Local,
                "claude-sonnet-4-6",
                100,
                50,
                0,
                UsageTurnEventKind::TurnRecorded,
            ),
        ];
        let rollup = rollup_usage_turns(&rows);
        assert_eq!(
            rollup.len(),
            2,
            "identical model+tokens but different lane must NEVER collapse to one entry"
        );
        let cloud_key = UsageRollupKey {
            identity: "local".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            task_id: Some("task-1".to_string()),
            session_id: Some("session-1".to_string()),
            lane: UsageLane::Cloud,
        };
        let local_key = UsageRollupKey {
            lane: UsageLane::Local,
            ..cloud_key.clone()
        };
        assert!(rollup.contains_key(&cloud_key));
        assert!(rollup.contains_key(&local_key));
        // A naive unlabeled sum would be 200; each lane's own entry must
        // stay at its own 100 -- the un-labeled-sum stub_ban.
        assert_eq!(rollup[&cloud_key].input_tokens, 100);
        assert_eq!(rollup[&local_key].input_tokens, 100);
    }

    // T6 EXPECTED_OBSERVATION: a fixture with no reported usage yields
    // Kind=EstimateApplied (labeled), never a silent all-zero TurnRecorded.
    #[test]
    fn resolve_turn_usage_falls_back_to_estimate_on_zero_reported() {
        let (est_in, est_out, kind) =
            resolve_turn_usage(0, 0, "a somewhat longer prompt text here", "a short reply");
        assert_eq!(kind, UsageTurnEventKind::EstimateApplied);
        assert!(est_in > 0, "estimator must produce a non-zero estimate for non-empty text");
        assert!(est_out > 0);
    }

    // T6 NEGATIVE: no row this module produces is ever a TurnRecorded
    // with all-zero counters.
    #[test]
    fn resolve_turn_usage_never_returns_turn_recorded_with_all_zero_counters() {
        let (est_in, est_out, kind) = resolve_turn_usage(0, 0, "", "");
        // Even on fully empty text (a genuine edge case), the kind must
        // stay EstimateApplied -- never silently promoted to TurnRecorded.
        assert_eq!(kind, UsageTurnEventKind::EstimateApplied);
        assert_eq!(est_in, 0);
        assert_eq!(est_out, 0);

        let (reported_in, reported_out, reported_kind) = resolve_turn_usage(5, 3, "x", "y");
        assert_eq!(reported_kind, UsageTurnEventKind::TurnRecorded);
        assert!(reported_in > 0 || reported_out > 0);
    }

    #[test]
    fn estimate_tokens_cached_is_memoized_and_deterministic() {
        let text = "the quick brown fox jumps over the lazy dog";
        let first = estimate_tokens_cached(text);
        let second = estimate_tokens_cached(text);
        assert_eq!(first, second);
        assert!(first > 0);
    }

    // T7 EXPECTED_OBSERVATION: cumulative input-context grows monotonically
    // across a compaction boundary (summary output becomes next input).
    #[test]
    fn apply_compaction_growth_is_monotonic_across_boundary() {
        let before = 10_000_u64;
        let after = apply_compaction_growth(before, 1_500);
        assert!(after > before, "cumulative input-context must grow, never reset");
        assert_eq!(after, 11_500);
    }

    #[test]
    fn normalized_usage_carries_the_correct_lane_and_semantics() {
        let raw = TokenUsage {
            input_tokens: 10,
            output_tokens: 5,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };
        let cloud = NormalizedUsage::cloud(raw);
        assert_eq!(cloud.lane, UsageLane::Cloud);
        assert_eq!(cloud.semantics_tag, UsageSemanticsTag::SuffixOnly);
        let local = NormalizedUsage::local(raw);
        assert_eq!(local.lane, UsageLane::Local);
        assert_eq!(local.semantics_tag, UsageSemanticsTag::FullPrompt);
    }
}
