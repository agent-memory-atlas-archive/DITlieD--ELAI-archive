//! C-32 close-out: orchestrator-side per-blank dispatch wiring that
//! invokes the [`crate::structural_retry::StructuralRetrySequencer`] for
//! every BLANK_N placeholder in a generated [`crate::skeleton_generator::SkeletonSpec`].
//!
//! Closes audit-C32 W1: the prior shim emitted `skeleton_generated` /
//! `skeleton_flushed` as observational beacons only and reported a boolean
//! `1.0 / 0.0` completeness ratio from the executor outcome. This module
//! drives the full §Task 5 dispatch chain:
//!
//!   1. [`crate::bridge_template::BridgeFileTemplate`] is consulted FIRST
//!      when the skeleton's target path matches a deterministic bridge
//!      kind (`mod.rs`, `__init__.py`, `index.ts`, `*.go` package doc).
//!      Hit -> single deterministic body fills every blank.
//!      Miss -> fall through to (2).
//!   2. For each blank, [`crate::structural_retry::StructuralRetrySequencer`]
//!      walks Decompose -> Template -> FewShot -> BestOfN. Candidates are
//!      drawn from the supplied [`PerBlankCandidates`] map (test code +
//!      future LLM-bridge injection point). When all stages exhaust:
//!        - `accept_and_complete_enabled && ratio >= 0.5` ->
//!          [`crate::structural_retry::StructuralRetryOutcome::AcceptAndComplete`]
//!          accepts the partial buffer (BLOCK fill rendered as the
//!          `// BLANK_N -- not filled` comment in the flushed file).
//!        - otherwise -> the blank is BLOCKed.
//!   3. [`crate::mini_critic::MiniCritic`] gates every candidate inside the
//!      sequencer; the final [`crate::skeleton_generator::FilledBuffer`]
//!      records the per-blank verdict and the rolled-up completeness ratio.
//!   4. Single `fs::write` per file via
//!      [`crate::skeleton_generator::SkeletonGenerator::flush_to_disk`].
//!   5. SSB rows emitted: `skeleton_generated`, `mini_critic_verdict`
//!      (per blank), `structural_retry_stage_attempt` (per stage when
//!      the sequencer engages), `skeleton_flushed` (with the real ratio).
//!
//! Errors are tracing::warn'd and swallowed — the dispatch path is
//! observational with respect to the legacy `baby_step_executor.execute`
//! call site; a failed flush must not derail the baby-step loop.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::bridge_template::{BridgeFileSpec, BridgeFileTemplate, BridgeKind};
use crate::mini_critic::{MiniCritic, MiniCriticLanguage};
use crate::skeleton_generator::{
    language_from_str, BlankId, FillResult, FilledBuffer, SkeletonGenerator, SkeletonSpec,
};
use crate::ssb::{
    MiniCriticVerdictPayload, SkeletonFlushedPayload, Ssb, SsbPayload,
    StructuralRetryStageAttemptPayload,
};
use crate::structural_retry::{
    RetryContext, StageCandidates, StaticStageCandidates, StructuralRetryOutcome,
    StructuralRetryPlan, StructuralRetryStage, StructuralRetrySequencer,
};

/// Per-blank candidate map. The orchestrator builds this from whatever
/// signal sources are available at dispatch time — today only the
/// [`BridgeFileTemplate`] fallback path injects entries; a future LLM
/// bridge will populate the four stage vectors per blank. Empty entries
/// cause the sequencer to exhaust the chain on that blank.
#[derive(Debug, Default, Clone)]
pub struct PerBlankCandidates {
    pub by_blank: BTreeMap<u8, StaticStageCandidates>,
}

impl PerBlankCandidates {
    /// Returns the candidates configured for `blank` or an empty bundle
    /// (which exhausts every stage in the sequencer).
    fn for_blank(&self, blank: BlankId) -> StaticStageCandidates {
        self.by_blank
            .get(&blank.0)
            .cloned()
            .unwrap_or_else(StaticStageCandidates::empty)
    }
}

/// Outcome of [`dispatch_skeleton_with_retry`] for a single file. Returned
/// so the caller can decide whether to surface skeleton results in the
/// executor's `BabyStepExecutionResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchOutcome {
    pub completeness_ratio: f32,
    pub partial_accepted: bool,
    pub bridge_short_circuit: bool,
}

/// Run the full §Task 5 chain for a single skeleton. Returns the
/// [`DispatchOutcome`] derived from the assembled [`FilledBuffer`].
///
/// `bridge_kind` is supplied externally so the caller can pre-classify
/// the skeleton's target path (see [`classify_bridge_kind`]). When `Some`,
/// the BridgeFileTemplate fast-path is consulted before the sequencer.
pub async fn dispatch_skeleton_with_retry(
    spec: SkeletonSpec,
    candidates: &PerBlankCandidates,
    critic: &MiniCritic,
    sequencer: &StructuralRetrySequencer,
    bridge_kind: Option<BridgeKind>,
    accept_and_complete_enabled: bool,
    ssb: Option<&Arc<Ssb>>,
    agent_id: &str,
    superstep: i64,
) -> Result<(FilledBuffer, DispatchOutcome), crate::skeleton_generator::SkeletonError> {
    let lang = language_from_str(&spec.language);
    let total_blanks = spec.blanks.len();

    // Step 1 — BridgeTemplate fast path. When the target file is a
    // deterministic bridge kind, the entire body is rendered without any
    // sequencer / LLM activity. Treats every blank as a single PASS fill
    // so the completeness gauge reports 1.0.
    if let Some(kind) = bridge_kind {
        let module_names = derive_bridge_module_names(&spec, kind);
        let template = BridgeFileTemplate::default();
        let bridge_spec = BridgeFileSpec {
            kind,
            target_path: spec.file_path.clone(),
            module_names,
        };
        let body = template.generate(&bridge_spec);
        // Use the rendered body as the fill for every blank; the
        // SkeletonGenerator flush still goes through, but the fills
        // share the same body so the file content is the bridge body.
        let bodies: Vec<String> = std::iter::repeat(body).take(total_blanks.max(1)).collect();
        // When the skeleton has zero blanks (bridge files often have
        // none), we still surface the bridge body as the file content
        // via a direct write.
        if total_blanks == 0 {
            // Surface a vacuous PASS buffer + ratio 1.0; the bridge body
            // was already produced. Write it directly via the
            // SkeletonGenerator helper so the single-write invariant is
            // preserved.
            let gen = SkeletonGenerator::default();
            let buf = FilledBuffer {
                spec: SkeletonSpec {
                    skeleton_text: bodies.first().cloned().unwrap_or_default(),
                    ..spec.clone()
                },
                fills: Vec::new(),
                completeness_ratio: 1.0,
            };
            let _ = gen.flush_to_disk(&buf);
            emit_skeleton_flushed(ssb, agent_id, superstep, &buf).await;
            return Ok((
                buf,
                DispatchOutcome {
                    completeness_ratio: 1.0,
                    partial_accepted: false,
                    bridge_short_circuit: true,
                },
            ));
        }
        let gen = SkeletonGenerator::default();
        let buf = gen.fill_blanks(spec, bodies, critic)?;
        let _ = gen.flush_to_disk(&buf);
        emit_mini_critic_verdicts(ssb, agent_id, superstep, &buf, lang).await;
        emit_skeleton_flushed(ssb, agent_id, superstep, &buf).await;
        let ratio = buf.completeness_ratio;
        return Ok((
            buf,
            DispatchOutcome {
                completeness_ratio: ratio,
                partial_accepted: false,
                bridge_short_circuit: true,
            },
        ));
    }

    // Step 2 — Per-blank structural retry loop. We accumulate fills
    // incrementally so each sequencer call has the live RetryContext
    // (existing_fills + accept_and_complete flag).
    let mut fills: Vec<FillResult> = Vec::with_capacity(total_blanks);
    let mut partial_accepted = false;
    for blank in &spec.blanks {
        let mut plan = StructuralRetryPlan::default_for_blank(*blank);
        let blank_candidates = candidates.for_blank(*blank);
        let ctx = RetryContext {
            project_root: std::path::PathBuf::new(),
            language: lang,
            fn_signature: signature_for_blank(&spec, *blank),
            existing_fills: fills.clone(),
            total_blanks,
            accept_and_complete_enabled,
        };
        let outcome = run_sequencer_with_attempt_emission(
            sequencer,
            &mut plan,
            &ctx,
            critic,
            &blank_candidates,
            ssb,
            agent_id,
            superstep,
            *blank,
        )
        .await;
        match outcome {
            StructuralRetryOutcome::Converged { fill } => {
                let verdict = critic.check(*blank, &fill, lang);
                fills.push(FillResult {
                    blank_id: *blank,
                    body: fill,
                    mini_critic_verdict: verdict.verdict,
                });
            }
            StructuralRetryOutcome::AcceptAndComplete { .. } => {
                partial_accepted = true;
                fills.push(FillResult {
                    blank_id: *blank,
                    body: String::new(),
                    mini_critic_verdict: crate::mini_critic::MiniCriticVerdict::Block,
                });
            }
            StructuralRetryOutcome::Exhausted => {
                fills.push(FillResult {
                    blank_id: *blank,
                    body: String::new(),
                    mini_critic_verdict: crate::mini_critic::MiniCriticVerdict::Block,
                });
            }
        }
    }

    let mut buf = FilledBuffer {
        spec,
        fills,
        completeness_ratio: 0.0,
    };
    buf.refresh_ratio();

    // Step 3 — flush_to_disk (single fs::write per file).
    let gen = SkeletonGenerator::default();
    if let Err(e) = gen.flush_to_disk(&buf) {
        tracing::warn!(error = %e, "c32_dispatch: flush_to_disk failed (non-fatal)");
    }

    // Step 4 — SSB emission of MiniCritic verdicts + final flushed row.
    emit_mini_critic_verdicts(ssb, agent_id, superstep, &buf, lang).await;
    emit_skeleton_flushed(ssb, agent_id, superstep, &buf).await;

    let ratio = buf.completeness_ratio;
    Ok((
        buf,
        DispatchOutcome {
            completeness_ratio: ratio,
            partial_accepted,
            bridge_short_circuit: false,
        },
    ))
}

/// Best-effort SSB emit for `skeleton_flushed`. Errors are swallowed.
async fn emit_skeleton_flushed(
    ssb: Option<&Arc<Ssb>>,
    agent_id: &str,
    superstep: i64,
    buf: &FilledBuffer,
) {
    let Some(ssb) = ssb else { return };
    let payload = SsbPayload::SkeletonFlushed(SkeletonFlushedPayload {
        path: buf.spec.file_path.to_string_lossy().into_owned(),
        completeness_ratio: buf.completeness_ratio,
    });
    let files = vec![buf.spec.file_path.to_string_lossy().into_owned()];
    let _ = ssb.write_payload(agent_id, superstep, payload, files).await;
}

/// Emit one `mini_critic_verdict` row per fill in the buffer.
async fn emit_mini_critic_verdicts(
    ssb: Option<&Arc<Ssb>>,
    agent_id: &str,
    superstep: i64,
    buf: &FilledBuffer,
    lang: MiniCriticLanguage,
) {
    let Some(ssb) = ssb else { return };
    for fill in &buf.fills {
        let reason = match fill.mini_critic_verdict {
            crate::mini_critic::MiniCriticVerdict::Pass => "ok".to_string(),
            crate::mini_critic::MiniCriticVerdict::Warn => "ts_unavailable".to_string(),
            crate::mini_critic::MiniCriticVerdict::Block => "exhausted".to_string(),
        };
        let payload = SsbPayload::MiniCriticVerdict(MiniCriticVerdictPayload {
            blank_id: fill.blank_id.0,
            verdict: fill.mini_critic_verdict.as_str().to_string(),
            reason,
            language: lang.as_str().to_string(),
        });
        let _ = ssb
            .write_payload(agent_id, superstep, payload, Vec::new())
            .await;
    }
}

/// Drive `sequencer.run_sequence` and emit one
/// `structural_retry_stage_attempt` SSB row per stage cursor advance so
/// the dashboard can render the per-stage timeline (plan §Task 6).
async fn run_sequencer_with_attempt_emission(
    sequencer: &StructuralRetrySequencer,
    plan: &mut StructuralRetryPlan,
    ctx: &RetryContext,
    critic: &MiniCritic,
    candidates: &dyn StageCandidates,
    ssb: Option<&Arc<Ssb>>,
    agent_id: &str,
    superstep: i64,
    blank: BlankId,
) -> StructuralRetryOutcome {
    // Snapshot stages so we can emit one row per stage attempted regardless
    // of where the sequencer converges.
    let stages: Vec<StructuralRetryStage> = plan.stages.clone();
    let starting_idx = plan.current_stage_idx;
    let outcome = sequencer.run_sequence(plan, ctx, critic, candidates);
    let ending_idx = plan.current_stage_idx;
    if let Some(ssb) = ssb {
        for (i, stage) in stages.iter().enumerate().skip(starting_idx) {
            if i > ending_idx {
                break;
            }
            let attempt = u32::try_from(i + 1).unwrap_or(u32::MAX);
            let label = if i == ending_idx
                && matches!(outcome, StructuralRetryOutcome::Converged { .. })
            {
                "converged"
            } else if i < ending_idx {
                "advanced"
            } else {
                match outcome {
                    StructuralRetryOutcome::Converged { .. } => "converged",
                    StructuralRetryOutcome::AcceptAndComplete { .. } => "accept_and_complete",
                    StructuralRetryOutcome::Exhausted => "exhausted",
                }
            };
            let payload = SsbPayload::StructuralRetryStageAttempt(
                StructuralRetryStageAttemptPayload {
                    blank_id: blank.0,
                    stage: stage.as_str().to_string(),
                    attempt,
                    outcome: label.to_string(),
                },
            );
            let _ = ssb
                .write_payload(agent_id, superstep, payload, Vec::new())
                .await;
        }
    }
    outcome
}

/// Look up the FunctionSpec.signature backing `blank` in `spec`. Falls
/// back to an empty string when the index is out of range (should not
/// happen for skeletons produced by `SkeletonGenerator::generate_*`).
fn signature_for_blank(spec: &SkeletonSpec, blank: BlankId) -> String {
    let idx = blank.0.saturating_sub(1) as usize;
    spec.functions
        .get(idx)
        .map(|f| f.signature.clone())
        .unwrap_or_default()
}

/// Classify a target file path against the four supported bridge kinds.
/// Returns `None` when the file is not a known bridge file.
pub fn classify_bridge_kind(file_path: &std::path::Path) -> Option<BridgeKind> {
    let name = file_path.file_name()?.to_str()?;
    if name == "mod.rs" {
        return Some(BridgeKind::ModRsAggregator);
    }
    if name == "__init__.py" {
        return Some(BridgeKind::InitPyBarrel);
    }
    if name == "index.ts" || name == "index.tsx" {
        return Some(BridgeKind::IndexTsBarrel);
    }
    // Go package-doc files are conventionally `doc.go`.
    if name == "doc.go" {
        return Some(BridgeKind::GoPackageDoc);
    }
    None
}

/// Derive the module_names vec for a [`BridgeFileSpec`] from the skeleton's
/// declared functions. Used when the bridge fast-path engages — each
/// function name becomes a sub-module entry. Bridge files with zero
/// declared functions get an empty list (the renderer still emits the
/// header banner).
fn derive_bridge_module_names(spec: &SkeletonSpec, _kind: BridgeKind) -> Vec<String> {
    spec.functions.iter().map(|f| f.name.clone()).collect()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton_generator::SkeletonGenerator;
    use elai_core::file_plan::FunctionSpec;
    use std::path::PathBuf;

    fn func(name: &str) -> FunctionSpec {
        FunctionSpec {
            name: name.into(),
            signature: format!("fn {name}()"),
            max_body_lines: 20,
        }
    }

    #[tokio::test]
    async fn dispatch_with_no_candidates_marks_all_blocked_and_ratio_zero() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("a.rs");
        let gen = SkeletonGenerator::default();
        let spec = gen
            .generate_for_functions(target.clone(), "rust".into(), &[func("a"), func("b")])
            .unwrap();
        let critic = MiniCritic::without_tree_sitter();
        let sequencer = StructuralRetrySequencer::default();
        let candidates = PerBlankCandidates::default();
        let (buf, outcome) = dispatch_skeleton_with_retry(
            spec,
            &candidates,
            &critic,
            &sequencer,
            None,
            false,
            None,
            "test-agent",
            0,
        )
        .await
        .expect("dispatch ok");
        assert_eq!(buf.fills.len(), 2);
        assert!(buf
            .fills
            .iter()
            .all(|f| f.mini_critic_verdict == crate::mini_critic::MiniCriticVerdict::Block));
        assert_eq!(outcome.completeness_ratio, 0.0);
        assert!(!outcome.partial_accepted);
        assert!(!outcome.bridge_short_circuit);
    }

    #[tokio::test]
    async fn dispatch_with_template_candidate_converges_first_blank() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("a.rs");
        let gen = SkeletonGenerator::default();
        let spec = gen
            .generate_for_functions(target.clone(), "rust".into(), &[func("a"), func("b")])
            .unwrap();
        let critic = MiniCritic::without_tree_sitter();
        let sequencer = StructuralRetrySequencer::default();
        let mut candidates = PerBlankCandidates::default();
        // Give blank 1 a passing template body; leave blank 2 empty.
        candidates.by_blank.insert(
            1,
            StaticStageCandidates {
                decompose: vec![],
                template: vec!["let x = 1;\nlet y = 2;\nx + y".into()],
                few_shot: vec![],
                best_of_n: vec![],
            },
        );
        let (buf, outcome) = dispatch_skeleton_with_retry(
            spec,
            &candidates,
            &critic,
            &sequencer,
            None,
            false,
            None,
            "test-agent",
            0,
        )
        .await
        .expect("dispatch ok");
        assert_eq!(buf.fills.len(), 2);
        assert_eq!(
            buf.fills[0].mini_critic_verdict,
            crate::mini_critic::MiniCriticVerdict::Pass
        );
        assert_eq!(
            buf.fills[1].mini_critic_verdict,
            crate::mini_critic::MiniCriticVerdict::Block
        );
        assert!((outcome.completeness_ratio - 0.5).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn dispatch_with_accept_and_complete_flag_marks_partial_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("a.rs");
        let gen = SkeletonGenerator::default();
        let spec = gen
            .generate_for_functions(target.clone(), "rust".into(), &[func("a"), func("b")])
            .unwrap();
        let critic = MiniCritic::without_tree_sitter();
        let sequencer = StructuralRetrySequencer::default();
        let mut candidates = PerBlankCandidates::default();
        candidates.by_blank.insert(
            1,
            StaticStageCandidates {
                decompose: vec![],
                template: vec!["let x = 1;\nlet y = 2;\nx + y".into()],
                few_shot: vec![],
                best_of_n: vec![],
            },
        );
        let (_buf, outcome) = dispatch_skeleton_with_retry(
            spec,
            &candidates,
            &critic,
            &sequencer,
            None,
            true,
            None,
            "test-agent",
            0,
        )
        .await
        .expect("dispatch ok");
        assert!(outcome.partial_accepted);
    }

    #[tokio::test]
    async fn dispatch_bridge_short_circuit_yields_ratio_one() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("mod.rs");
        let gen = SkeletonGenerator::default();
        let spec = gen
            .generate_for_functions(target.clone(), "rust".into(), &[func("alpha"), func("beta")])
            .unwrap();
        let critic = MiniCritic::without_tree_sitter();
        let sequencer = StructuralRetrySequencer::default();
        let candidates = PerBlankCandidates::default();
        let bridge_kind = classify_bridge_kind(&target);
        assert_eq!(bridge_kind, Some(BridgeKind::ModRsAggregator));
        let (buf, outcome) = dispatch_skeleton_with_retry(
            spec,
            &candidates,
            &critic,
            &sequencer,
            bridge_kind,
            false,
            None,
            "test-agent",
            0,
        )
        .await
        .expect("dispatch ok");
        assert!(outcome.bridge_short_circuit);
        assert!((outcome.completeness_ratio - 1.0).abs() < f32::EPSILON);
        assert_eq!(buf.fills.len(), 2);
    }

    #[test]
    fn classify_bridge_kind_matches_known_names() {
        assert_eq!(
            classify_bridge_kind(&PathBuf::from("a/b/mod.rs")),
            Some(BridgeKind::ModRsAggregator)
        );
        assert_eq!(
            classify_bridge_kind(&PathBuf::from("a/b/__init__.py")),
            Some(BridgeKind::InitPyBarrel)
        );
        assert_eq!(
            classify_bridge_kind(&PathBuf::from("a/b/index.ts")),
            Some(BridgeKind::IndexTsBarrel)
        );
        assert_eq!(
            classify_bridge_kind(&PathBuf::from("a/b/doc.go")),
            Some(BridgeKind::GoPackageDoc)
        );
        assert_eq!(classify_bridge_kind(&PathBuf::from("a/b/main.rs")), None);
    }
}
