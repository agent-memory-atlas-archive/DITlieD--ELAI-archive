//! J-109: the learned-artifact admission-hardening probe at the E-5 capture seam --
//! the focused module extracted from `user_memory_capture` (MR-1: that producer
//! crossed the 1000-line module gate). Behavior is byte-identical to the code it
//! replaces: same env flag, same `AdmissionVerdict` row fields, same position in
//! `capture_remember_with_clock` (immediately BEFORE the `dispatch_one` write).
//!
//! Gated behind `ELAI_ADMISSION_HARDENING_ENABLE` (default-OFF). When unset or not
//! `"1"`, NO admission probe runs at all (zero extra SSB rows; the capture flow is
//! byte-identical to pre-J-109).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::ClockProvider;
use elai_core::config::ConfigLoader;
use elai_orchestrator::admission_pipeline::{
    rebuild_if_stale, AdmissionPipeline, StaticSpanIndex,
};
use elai_orchestrator::ssb::{AdmissionVerdictPayload, SsbPayload};

/// J-109 R9 default-OFF experiment flag: learned-artifact admission hardening at
/// the E-5 capture seam. When unset or not `"1"`, NO admission probe runs at all
/// (zero extra SSB rows; the capture flow is byte-identical to pre-J-109).
pub(crate) const ADMISSION_HARDENING_FLAG: &str = "ELAI_ADMISSION_HARDENING_ENABLE";

/// J-109: `originating_seam` label stamped on the admission verdict rows emitted
/// by this producer (the E-5 capture front of the two write seams). `pub(crate)`
/// so the plan-A1.4 falsifiable seam test in `user_memory_capture` can pin it.
pub(crate) const ADMISSION_SEAM_E5_CAPTURE_REMEMBER: &str = "e5_capture_remember";

/// J-109 UNIT A2: `originating_seam` label stamped on the admission verdict rows
/// emitted by the I-12 rule-intake front (`elai rule add` -> `rule_cmd::add_rule`)
/// -- the second of the two write seams. Same `pub(crate)` visibility as the E-5
/// precedent so the A2 falsifiable seam test in `rule_cmd` can pin it.
pub(crate) const ADMISSION_SEAM_I12_RULE_ADD: &str = "i12_rule_add";

/// True iff the J-109 admission-hardening flag is set to `"1"` (default-OFF).
#[must_use]
pub(crate) fn admission_hardening_enabled() -> bool {
    std::env::var(ADMISSION_HARDENING_FLAG).is_ok_and(|v| v == "1")
}

/// J-109 A1.3 (R-8 production wiring): the flagged pre-`dispatch_one` admission
/// probe for the E-5 capture front -- stamps
/// [`ADMISSION_SEAM_E5_CAPTURE_REMEMBER`] and delegates to
/// [`emit_admission_verdict_for_seam`].
pub(crate) fn emit_admission_verdict(
    clock: &Arc<dyn ClockProvider>,
    project_root: &PathBuf,
    candidate_text: &str,
) {
    emit_admission_verdict_for_seam(
        clock,
        project_root,
        candidate_text,
        ADMISSION_SEAM_E5_CAPTURE_REMEMBER,
    );
}

/// J-109 UNIT A2 generalization of the A1.3 probe (R-8 production wiring): the
/// flagged admission probe with the `originating_seam` provenance label supplied
/// by the calling front, so each write seam's verdict rows stay distinguishable
/// in the journal (GD-2). The E-5 capture front keeps [`emit_admission_verdict`];
/// the I-12 rule-intake front (`rule_cmd::add_rule`) passes
/// [`ADMISSION_SEAM_I12_RULE_ADD`].
///
/// Builds the [`StaticSpanIndex`] STRICTLY from the J-102
/// `ConfigLoader::admission_scan()` rows (GD-1 -- this fn never enumerates a
/// second path list; the cwd CLAUDE.md/AGENTS.md seeding at core/config.rs:316
/// makes the root members first-class index inputs BY CONSTRUCTION), resolves
/// staleness via [`rebuild_if_stale`] BEFORE any verdict, runs the pure AD-2
/// lexical leg over the candidate text and emits the `AdmissionVerdict` row
/// through the shared `emit_ssb_row` helper (which stays in
/// `user_memory_capture`, its other callers being the capture + proposal
/// producers).
///
/// OBSERVE-ONLY (RT-6/GD-4): performs NO store mutation and NEVER blocks the
/// caller -- the write below always proceeds. Probe input is the RAW candidate
/// text the caller is about to persist, so the row's blake3 `candidate_hash` is
/// recomputable from the journal (GD-2).
pub(crate) fn emit_admission_verdict_for_seam(
    clock: &Arc<dyn ClockProvider>,
    project_root: &PathBuf,
    candidate_text: &str,
    originating_seam: &str,
) {
    let rows = ConfigLoader::default_for(project_root).admission_scan();
    if rows.is_empty() {
        // Empty manifest: no immutable surface exists to dedup against, and an
        // `admit_clean` row against an unscanned surface would fake coverage
        // (GD-5 honesty). Log + emit nothing.
        tracing::warn!(
            "admission hardening: admission_scan returned an empty manifest; probe skipped"
        );
        return;
    }
    // Staleness FIRST, verdict after (J-109 contract arm (c)). The cache here is
    // single-shot (one probe per capture); the reused-cache rebuild arm is
    // exercised directly by the contracts-crate tests.
    let mut cache: Option<StaticSpanIndex> = None;
    let rebuilt = rebuild_if_stale(&mut cache, &rows);
    let Some(index) = cache else {
        // Unreachable for a non-empty manifest (from_scan_rows always yields an
        // index); kept so the invariant is enforced at the type level.
        tracing::warn!("admission hardening: index rebuild produced no index; probe skipped");
        return;
    };
    let evaluation = AdmissionPipeline.evaluate_candidate(&index, candidate_text);
    tracing::debug!(
        rebuilt,
        verdict = evaluation.receipts.verdict,
        "admission probe evaluated (observe-only; write proceeds)"
    );
    let payload = SsbPayload::AdmissionVerdict(AdmissionVerdictPayload {
        candidate_hash: evaluation.receipts.candidate_hash,
        surface_leg: evaluation.receipts.surface_leg.to_string(),
        threshold: evaluation.receipts.threshold_used,
        verdict: evaluation.receipts.verdict.to_string(),
        originating_seam: originating_seam.to_string(),
        ts: u64::try_from(clock.now_ms()).unwrap_or(u64::MAX),
    });
    crate::user_memory_capture::emit_ssb_row(clock, project_root, payload);
}
