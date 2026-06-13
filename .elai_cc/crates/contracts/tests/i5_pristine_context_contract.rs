// I-5 — Pristine context injector contract test.
//
// Tier-1 wiring coverage for PristinePathFilter::build_branch_context.
// (I-4 SearchLadder is the production caller; it is Wave-4 and does not yet
// exist. This contract test stands in as the only non-test caller gate until
// I-4 ships — per R7 note, a contract test is NOT a production caller, so the
// WIRE-DARK[I-5] stub entry in stubs.md remains open.)
//
// Two-branch retry fixture (mirrors the doc-comment scenario):
//   branch-1: has a failed 5 KB traceback  -> TracebackContrasted row emitted;
//             constraint_phrase must be present and <= 30 tokens.
//   branch-2: rebuilt pristinely (no siblings, no traceback) -> no ContextPruned
//             row for branch-2 (zero traceback bytes in outcome).
//
// Observable artifacts:
//   - BuildBranchContextOutcome.original_traceback_bytes >= 5000 for branch-1.
//   - BuildBranchContextOutcome.constraint_phrase.is_some() for branch-1.
//   - Constraint phrase word count <= 30.
//   - BuildBranchContextOutcome.original_traceback_bytes == 0 for branch-2
//     (pristine rebuild, no traceback above threshold).
//   - SSB ContextPruned row appears in the db for branch-1 (TracebackContrasted).
//
// R2: SSB database is opened after filter returns and the row is counted. A
// zero row-count is FAIL, never UNVERIFIED.

use elai_core::clock::SeededClock;
use elai_orchestrator::{
    context_budget::ContextBudget,
    dispatcher::BackendMode,
    pristine_path_filter::{PristinePathFilter, TRACEBACK_COMPRESS_THRESHOLD_BYTES},
    ssb::{ContextPrunedKind, Ssb, SsbActionType, SsbConfig, SsbReader},
};
use std::sync::Arc;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_ssb(dir: &TempDir, name: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: dir.path().join(format!("{name}.db")),
        pool_size: None,
        project_id: "i5-contract".into(),
        session_id: "i5-session".into(),
    };
    Arc::new(Ssb::new(cfg, Arc::new(SeededClock::new())).expect("ssb init"))
}

fn make_budget() -> ContextBudget {
    ContextBudget::compute(16_384, [0, 0, 0], 512).expect("budget feasible")
}

/// Synthetic 5 KB failed traceback — well above TRACEBACK_COMPRESS_THRESHOLD_BYTES.
fn traceback_5k() -> Vec<u8> {
    let line = b"error[E0308]: mismatched types -- expected `String`, found `&str` at src/lib.rs:99.\n";
    let reps = (5 * 1024 / line.len()) + 1;
    line.repeat(reps)
}

// ---------------------------------------------------------------------------
// Contract arms
// ---------------------------------------------------------------------------

/// branch-1: large traceback -> TracebackContrasted SSB row; constraint present.
#[tokio::test]
async fn branch1_traceback_emits_context_pruned_and_constraint() {
    let dir = TempDir::new().expect("tmpdir");
    let ssb = make_ssb(&dir, "b1");
    let filter = PristinePathFilter::new(Arc::clone(&ssb));
    let mut budget = make_budget();

    let tb = traceback_5k();
    assert!(
        tb.len() >= TRACEBACK_COMPRESS_THRESHOLD_BYTES,
        "fixture traceback must exceed threshold; got {} bytes",
        tb.len()
    );

    let outcome = filter
        .build_branch_context(
            &mut budget,
            "branch-1",
            "test-session",
            &[], // no siblings for this arm
            Some(&tb),
            BackendMode::Api,
        )
        .await
        .expect("branch-1 build should succeed");

    // Traceback bytes recorded.
    assert!(
        outcome.original_traceback_bytes >= TRACEBACK_COMPRESS_THRESHOLD_BYTES as u64,
        "original_traceback_bytes={} expected >= {}",
        outcome.original_traceback_bytes,
        TRACEBACK_COMPRESS_THRESHOLD_BYTES
    );

    // Constraint phrase present.
    let phrase = outcome
        .constraint_phrase
        .as_deref()
        .expect("constraint_phrase must be set when traceback >= threshold");

    // Constraint phrase must be <= 30 tokens (word-split proxy).
    let word_count = phrase.split_whitespace().count();
    assert!(
        word_count <= 30,
        "constraint phrase has {} words, must be <= 30: {:?}",
        word_count,
        phrase
    );

    // No sibling exclusion bytes (no siblings passed).
    assert_eq!(
        outcome.excluded_bytes, 0,
        "excluded_bytes must be zero (no siblings)"
    );

    // R2: verify the ContextPruned SSB row actually landed in the database.
    // query_by_type uses the action_type SQL tag ("context_pruned") so any
    // ContextPrunedKind arm matches.  A zero row-count means the emit path
    // silently failed or was removed — that is FAIL, never UNVERIFIED.
    let context_pruned_rows = ssb
        .query_by_type(SsbActionType::ContextPruned(
            ContextPrunedKind::TracebackContrasted,
        ))
        .expect("query_by_type(ContextPruned) must not error");
    assert!(
        !context_pruned_rows.is_empty(),
        "expected >= 1 ContextPruned SSB row in the database after branch-1 build; got 0"
    );
}

/// branch-2: pristine rebuild (no siblings, no traceback) -> zero traceback bytes.
#[tokio::test]
async fn branch2_pristine_rebuild_zero_traceback_bytes() {
    let dir = TempDir::new().expect("tmpdir");
    let ssb = make_ssb(&dir, "b2");
    let filter = PristinePathFilter::new(Arc::clone(&ssb));
    let mut budget = make_budget();

    let outcome = filter
        .build_branch_context(
            &mut budget,
            "branch-2",
            "test-session",
            &[], // no siblings
            None, // no traceback
            BackendMode::Api,
        )
        .await
        .expect("branch-2 pristine build should succeed");

    assert_eq!(
        outcome.original_traceback_bytes, 0,
        "pristine rebuild must carry zero traceback bytes"
    );
    assert!(
        outcome.constraint_phrase.is_none(),
        "no traceback -> no constraint phrase"
    );
    assert_eq!(
        outcome.excluded_bytes, 0,
        "no siblings -> zero excluded bytes"
    );
}

/// Verify the max_constraint_tokens constant aligns with the 30-token cap
/// documented in plan I-5 (id-1 doctrine).
#[test]
fn max_constraint_tokens_is_thirty() {
    use elai_orchestrator::pristine_path_filter::MAX_CONSTRAINT_TOKENS;
    assert_eq!(
        MAX_CONSTRAINT_TOKENS, 30,
        "MAX_CONSTRAINT_TOKENS must equal 30 per ID-1 doc"
    );
}
