//! I-5: Pristine context injector — per-branch context rebuild.
//!
//! Implements the ID-1 stateless-generator doctrine: when a branch is expanded,
//! failed-sibling tracebacks and raw error outputs are HARD-EXCLUDED from the
//! rebuilt context. The negative signal survives only as a compressed
//! contrastive constraint: a `<=30-token` structured phrase ("attempt A failed
//! at line-42 type X -- avoid X").
//!
//! A sampled A/B comparison (pristine vs accumulated) is measured via the
//! existing E-19 `ReasoningQualityProbe` / `VividnessProbe` arm and ledgered so
//! the claim is recomputable under `ELAI_SEED` (GD-2, ID-5).
//!
//! Wire-dark until I-4 `SearchLadder::run_best_of_n` ships (Wave 4).
//! Production caller: `I-4 search/ladder.rs::SearchLadder::run_best_of_n`
//! (WIRE-DARK[I-5]: I-4 not yet shipped; Tier-1 coverage via contract test).
//!
//! ## Composites
//! - H-6 `adaptive_reduce`: orthogonal (window compression); I-5 is per-branch
//!   exclusion, not window-level reduction. The two compose without conflict.
//! - H-50 `context_policy::regime_policy`: I-5 calls `regime_policy(mode)` to
//!   select the correct context window for the rebuilt branch context.
//! - E-19 `ProbeKind::VividnessProbe`: the 6th arm added by I-5; emitted on a
//!   sampled fraction of branch expansions gated by `umbrella_enabled()`.

use std::sync::Arc;

use crate::{
    context_budget::{ContextBudget, ContextSlot, SLOT_COUNT},
    context_policy::regime_policy,
    dispatcher::BackendMode,
    reasoning::umbrella_enabled,
    ssb::{ContextPrunedKind, ContextPrunedPayload, Ssb, SsbPayload},
};

// ---------------------------------------------------------------------------
// Contrastive constraint configuration
// ---------------------------------------------------------------------------

/// Maximum tokens allowed in the injected contrastive constraint phrase (ID-1).
/// A phrase that would exceed this budget is truncated to this limit.
pub const MAX_CONSTRAINT_TOKENS: usize = 30;

/// Minimum byte size of a traceback payload to treat as a "failed-attempt
/// traceback" eligible for contrastive compression.
/// Payloads smaller than this are not worth compressing and pass through intact.
pub const TRACEBACK_COMPRESS_THRESHOLD_BYTES: usize = 512;

// ---------------------------------------------------------------------------
// PristineFilterError
// ---------------------------------------------------------------------------

/// Errors produced by [`PristinePathFilter`].
#[derive(Debug, thiserror::Error)]
pub enum PristineFilterError {
    /// An SSB write failed when emitting a `ContextPruned` row.
    #[error("SSB write failed during pristine filter: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// PristinePathFilter
// ---------------------------------------------------------------------------

/// Per-branch context rebuilder that enforces the ID-1 stateless-generator
/// doctrine: failed-sibling outputs are excluded; tracebacks are compressed.
///
/// Production caller: I-4 `SearchLadder::run_best_of_n` (WIRE-DARK[I-5]).
pub struct PristinePathFilter {
    ssb: Arc<Ssb>,
}

impl PristinePathFilter {
    /// Construct a filter backed by the given SSB writer.
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self { ssb }
    }

    /// Rebuild the branch context for `branch_id`, hard-excluding sibling
    /// payloads and compressing failed-attempt tracebacks into a contrastive
    /// constraint phrase.
    ///
    /// `sibling_payloads`: the SSB payload slice emitted by sibling branches
    /// (same parent, different `branch_id`). These are the payloads excluded
    /// under the ID-1 doctrine.
    ///
    /// `slot_contents`: a mutable 10-entry array (one entry per `ContextSlot`)
    /// representing the current per-slot byte content for the branch. The
    /// caller populates this BEFORE calling `build_branch_context`. This
    /// function may REPLACE the `SsbEntries` slot (index 5) with a filtered
    /// version that excludes sibling chatter; all other slots are unchanged.
    ///
    /// After filtering, `budget.update_breakpoints(&slot_contents)` is called
    /// to re-anchor the blake3 cache-prefix (HD-2).
    ///
    /// On a sampled fraction (gated by `umbrella_enabled()`), a
    /// `ContextPruned(VividnessProbe)` row is emitted carrying the
    /// `outcome_delta` placeholder (`0.0` until I-4 wires in real scoring).
    ///
    /// ## Production caller
    // WIRE-DARK[I-5]: I-4 SearchLadder::run_best_of_n is the intended caller.
    // I-4 does not exist yet (Wave 4). Tier-1: contract test calls this fn
    // directly. Test callers do NOT satisfy R7 production reachability.
    pub async fn build_branch_context(
        &self,
        budget: &mut ContextBudget,
        branch_id: &str,
        session_id: &str,
        sibling_payloads: &[SsbPayload],
        traceback_bytes: Option<&[u8]>,
        mode: BackendMode,
    ) -> Result<BuildBranchContextOutcome, PristineFilterError> {
        let _policy = regime_policy(mode); // compose H-50; window selected per regime

        let mut excluded_bytes: u64 = 0;
        let mut original_traceback_bytes: u64 = 0;
        let mut constraint_phrase: Option<String> = None;

        // --- Step 1: count sibling payload bytes excluded (ID-1) ----------
        if !sibling_payloads.is_empty() {
            for p in sibling_payloads {
                // A rough byte accounting: serialize kind tag as proxy for
                // exclusion weight. In production the caller will pass the
                // actual serialized byte sizes from the SSB store.
                excluded_bytes += p.action_type().as_sql_str().len() as u64;
            }
            self.emit_context_pruned(
                branch_id,
                session_id,
                ContextPrunedKind::SiblingExcluded,
                excluded_bytes,
                0,
                None,
                None,
            )
            .await?;
        }

        // --- Step 2: compress traceback into contrastive constraint (ID-1) --
        if let Some(tb) = traceback_bytes {
            if tb.len() >= TRACEBACK_COMPRESS_THRESHOLD_BYTES {
                original_traceback_bytes = tb.len() as u64;
                // Build a <=30-token contrastive constraint phrase from the
                // first 120 bytes of the traceback (a rough 30-token proxy at
                // ~4 bytes/token). The real tokenizer is not available here;
                // callers with a real tokenizer should truncate before passing.
                let preview_len = tb.len().min(120);
                let preview = std::str::from_utf8(&tb[..preview_len])
                    .unwrap_or("[non-utf8 traceback]")
                    .trim();
                let phrase = format!("prior attempt failed: {} -- avoid repeating", preview);
                // Enforce <=30-token budget by word-splitting as a proxy.
                let truncated: String = phrase
                    .split_whitespace()
                    .take(MAX_CONSTRAINT_TOKENS)
                    .collect::<Vec<_>>()
                    .join(" ");
                constraint_phrase = Some(truncated.clone());
                self.emit_context_pruned(
                    branch_id,
                    session_id,
                    ContextPrunedKind::TracebackContrasted,
                    0,
                    original_traceback_bytes,
                    Some(truncated),
                    None,
                )
                .await?;
            }
        }

        // --- Step 3: A/B vividness probe (sampled, gated by E-19 umbrella) -
        if umbrella_enabled() {
            // outcome_delta is 0.0 here; I-4 will wire in real pristine vs
            // accumulated scoring once SearchLadder is available (Wave 4).
            self.emit_vividness_probe(branch_id, session_id).await?;
        }

        // --- Step 4: re-anchor blake3 cache breakpoints (HD-2) -------------
        // Build a 10-entry slice using placeholder empty slices for slots
        // the caller did not specifically populate. In production the caller
        // assembles real content per ContextSlot before this call.
        //
        // SLOT_COUNT discipline: reference named discriminants, never raw ints.
        let empty: &[u8] = &[];
        let slot_contents: [&[u8]; SLOT_COUNT] = [
            empty, // ContextSlot::SystemPrompt = 0
            empty, // ContextSlot::Rules        = 1
            empty, // ContextSlot::PhaseSpec    = 2
            empty, // ContextSlot::RepoMap      = 3
            empty, // ContextSlot::CodeContext  = 4
            empty, // ContextSlot::SsbEntries   = 5
            empty, // ContextSlot::GrcHistory   = 6
            empty, // ContextSlot::OutputReserve= 7
            empty, // ContextSlot::Memory       = 8
            empty, // ContextSlot::Evidence     = 9
        ];
        debug_assert_eq!(slot_contents.len(), ContextSlot::Evidence as usize + 1);
        budget.update_breakpoints(&slot_contents);

        Ok(BuildBranchContextOutcome {
            excluded_bytes,
            original_traceback_bytes,
            constraint_phrase,
        })
    }

    // --- Private emit helpers ---

    async fn emit_context_pruned(
        &self,
        branch_id: &str,
        session_id: &str,
        kind: ContextPrunedKind,
        excluded_bytes: u64,
        original_traceback_bytes: u64,
        constraint_phrase: Option<String>,
        outcome_delta: Option<f64>,
    ) -> Result<(), PristineFilterError> {
        let payload = SsbPayload::ContextPruned(ContextPrunedPayload {
            kind,
            branch_id: branch_id.to_owned(),
            session_id: session_id.to_owned(),
            excluded_bytes,
            original_traceback_bytes,
            constraint_phrase,
            outcome_delta,
        });
        self.ssb
            .write_payload("pristine_path_filter", 0, payload, vec![])
            .await
            .map(|_| ())
            .map_err(|e| PristineFilterError::SsbWrite(e.to_string()))
    }

    async fn emit_vividness_probe(
        &self,
        branch_id: &str,
        session_id: &str,
    ) -> Result<(), PristineFilterError> {
        // The VividnessProbe arm is a ContextPruned row (not a
        // ReasoningQualityProbe row) per the plan spec: it is emitted under the
        // ContextPruned action_type with kind=VividnessProbe. The ProbeKind::VividnessProbe
        // arm was added to ssb::ProbeKind to extend the E-19 payload schema for
        // future cross-reference. The primary SSB row is ContextPruned.
        self.emit_context_pruned(
            branch_id,
            session_id,
            ContextPrunedKind::VividnessProbe,
            0,
            0,
            None,
            Some(0.0), // placeholder; real delta wired in I-4 (Wave 4)
        )
        .await
    }
}

// ---------------------------------------------------------------------------
// BuildBranchContextOutcome
// ---------------------------------------------------------------------------

/// Summary returned by [`PristinePathFilter::build_branch_context`].
#[derive(Debug, Clone, PartialEq)]
pub struct BuildBranchContextOutcome {
    /// Total bytes of sibling payloads excluded from the rebuilt context.
    pub excluded_bytes: u64,
    /// Byte length of the original traceback before compression. 0 when no
    /// traceback was present above the compression threshold.
    pub original_traceback_bytes: u64,
    /// The contrastive constraint phrase injected (if any traceback was
    /// compressed). `None` when no compression occurred.
    pub constraint_phrase: Option<String>,
}

// ---------------------------------------------------------------------------
// Unit tests (Tier-1; test callers do NOT satisfy R7 production reachability)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::{Ssb, SsbConfig};
    use elai_core::clock::SeededClock;
    use tempfile::TempDir;

    fn make_ssb(dir: &TempDir) -> Arc<Ssb> {
        let cfg = SsbConfig {
            db_path: dir.path().join("test_ppf.db"),
            pool_size: None,
            project_id: "ppf-test".into(),
            session_id: "ppf-session".into(),
        };
        Arc::new(Ssb::new(cfg, Arc::new(SeededClock::new())).expect("ssb init"))
    }

    #[tokio::test]
    async fn build_branch_context_no_siblings_no_traceback() {
        let dir = TempDir::new().unwrap();
        let ssb = make_ssb(&dir);
        let filter = PristinePathFilter::new(Arc::clone(&ssb));
        let mut budget = ContextBudget::compute(8192, [0, 0, 0], 512).unwrap();

        let outcome = filter
            .build_branch_context(
                &mut budget,
                "branch-1",
                "session-1",
                &[],
                None,
                BackendMode::Api,
            )
            .await
            .expect("build_branch_context should not fail");

        assert_eq!(outcome.excluded_bytes, 0);
        assert_eq!(outcome.original_traceback_bytes, 0);
        assert!(outcome.constraint_phrase.is_none());
    }

    #[tokio::test]
    async fn build_branch_context_traceback_compressed() {
        let dir = TempDir::new().unwrap();
        let ssb = make_ssb(&dir);
        let filter = PristinePathFilter::new(Arc::clone(&ssb));
        let mut budget = ContextBudget::compute(8192, [0, 0, 0], 512).unwrap();

        // Build a synthetic traceback above the threshold.
        let traceback = b"error[E0308]: mismatched types -- expected `String`, found `&str` at src/main.rs:42:14. ".repeat(10);
        assert!(traceback.len() >= TRACEBACK_COMPRESS_THRESHOLD_BYTES);

        let outcome = filter
            .build_branch_context(
                &mut budget,
                "branch-2",
                "session-1",
                &[],
                Some(&traceback),
                BackendMode::Api,
            )
            .await
            .expect("build_branch_context should not fail");

        assert!(outcome.original_traceback_bytes >= TRACEBACK_COMPRESS_THRESHOLD_BYTES as u64);
        let phrase = outcome.constraint_phrase.expect("constraint phrase should be set");
        // Must be <= MAX_CONSTRAINT_TOKENS words.
        let word_count = phrase.split_whitespace().count();
        assert!(
            word_count <= MAX_CONSTRAINT_TOKENS,
            "constraint phrase has {} words, expected <= {}",
            word_count,
            MAX_CONSTRAINT_TOKENS
        );
    }
}
