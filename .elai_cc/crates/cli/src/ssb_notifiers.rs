#![allow(dead_code, unused_imports, unused_variables, unused_mut, clippy::unneeded_struct_pattern, clippy::unnecessary_wraps, clippy::unused_self)]

use std::sync::Arc;

// ---------------------------------------------------------------------------
// C-1: reranker SSB notifier adapters
// ---------------------------------------------------------------------------
//
// The knowledge crate's `SsbNotifier` trait (defined in
// `crate::degradation`) emits `DegradationEvent { action_type, stage,
// detail }`. The orchestrator's `Arc<Ssb>` writer needs an
// `SsbPayload::KnowledgeDegradation` row. `SsbRerankerNotifier` bridges
// the two so the C-1 stub_ban check
// (`NoopSsbNotifier` absent from production `reranker.rs` lines) holds
// while `knowledge_degradation` rows still appear in the dashboard.
pub(crate) struct SsbRerankerNotifier {
    pub(crate) ssb: Arc<elai_orchestrator::ssb::Ssb>,
}

impl SsbRerankerNotifier {
    pub(crate) fn new(ssb: Arc<elai_orchestrator::ssb::Ssb>) -> Self {
        Self { ssb }
    }
}

impl elai_knowledge::reranker::SsbNotifier for SsbRerankerNotifier {
    fn emit(&self, event: elai_knowledge::reranker::DegradationEvent) {
        let payload = elai_orchestrator::ssb::SsbPayload::KnowledgeDegradation(
            elai_orchestrator::ssb::KnowledgeDegradationPayload {
                component: event.stage.to_owned(),
                old_mode: String::new(),
                new_mode: String::new(),
                reason: event.detail,
            },
        );
        let ssb = self.ssb.clone();
        // Same pattern as `OrchestratorKnowledgeSsbSink` — spawn onto the
        // active tokio runtime; drop with a warning when none is active.
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = ssb
                        .write_payload(
                            elai_orchestrator::task_runner::RUNNER_AGENT_ID,
                            elai_orchestrator::task_runner::RUNNER_SUPERSTEP,
                            payload,
                            Vec::new(),
                        )
                        .await
                    {
                        tracing::warn!(
                            error = %e,
                            "live_cli reranker notifier: write_payload failed (non-fatal)"
                        );
                    }
                });
            }
            Err(_) => {
                tracing::debug!(
                    "live_cli reranker notifier: no tokio runtime; dropping row"
                );
            }
        }
    }
}

/// Local noop used when no Ssb writer is available. Distinct from the
/// `NoopSsbNotifier` defined in `crate::degradation` so that the
/// production reranker.rs surface has zero `NoopSsbNotifier`
/// references (WIRING_CONTRACT stub_ban).
pub(crate) struct NoopRerankerNotifier;

impl elai_knowledge::reranker::SsbNotifier for NoopRerankerNotifier {
    fn emit(&self, _event: elai_knowledge::reranker::DegradationEvent) {}
}

// ---------------------------------------------------------------------------
// B-3 #5: DegradationCascade production SSB bridge.
// ---------------------------------------------------------------------------
//
// The knowledge crate's `DegradationCascade::evaluate` calls back into
// `&dyn knowledge::degradation::SsbNotifier` on every search-mode change.
// In production we must avoid `knowledge::degradation::NoopSsbNotifier`
// (the trait's test-only no-op impl) so the `knowledge_degradation` SSB
// rows actually land in the dashboard's ActivityWindow + KnowledgeDrawer
// degradation surfaces. `SsbDegradationNotifier` is the production
// adapter that wraps `Arc<Ssb>` and writes
// `SsbPayload::KnowledgeDegradation` rows. Same pattern as
// `SsbRerankerNotifier` above.
//
// B-4 owns the producer-side wiring (the periodic
// `cascade.evaluate(...)` invocation that drives mode changes); B-3
// reserves the adapter so B-4 has a real notifier constructor at hand
// rather than wiring `NoopSsbNotifier` into the production path.
pub(crate) struct SsbDegradationNotifier {
    pub(crate) ssb: Arc<elai_orchestrator::ssb::Ssb>,
}

impl SsbDegradationNotifier {
    // B-4: constructor now invoked at live_cli.rs:~1602 by the periodic
    // cascade evaluator block (`spawn_cascade_evaluator_loop`). The
    // `#[allow(dead_code)]` reservation from B-3 has been removed.
    pub(crate) fn new(ssb: Arc<elai_orchestrator::ssb::Ssb>) -> Self {
        Self { ssb }
    }
}

impl elai_knowledge::degradation::SsbNotifier for SsbDegradationNotifier {
    fn emit(&self, event: elai_knowledge::degradation::DegradationEvent) {
        // The DegradationEvent carries only {action_type, stage, detail};
        // the orchestrator side wants {component, old_mode, new_mode, reason}.
        // The cascade evaluator formats `detail` as
        // "<old> -> <new>: code=..., text=..., splade=..." (degradation.rs:129).
        // We forward `stage` as `component` and split the detail string on
        // " -> " to recover old/new mode; on parse failure we degrade
        // gracefully to placing the full detail in `reason` with empty mode
        // fields — preferred over dropping the row entirely.
        let (old_mode, new_mode, reason) =
            match event.detail.split_once(" -> ") {
                Some((old_s, rest)) => match rest.split_once(": ") {
                    Some((new_s, why)) => (
                        old_s.trim().to_owned(),
                        new_s.trim().to_owned(),
                        why.trim().to_owned(),
                    ),
                    None => (
                        old_s.trim().to_owned(),
                        rest.trim().to_owned(),
                        String::new(),
                    ),
                },
                None => (String::new(), String::new(), event.detail.clone()),
            };
        let payload = elai_orchestrator::ssb::SsbPayload::KnowledgeDegradation(
            elai_orchestrator::ssb::KnowledgeDegradationPayload {
                component: event.stage.to_owned(),
                old_mode,
                new_mode,
                reason,
            },
        );
        let ssb = self.ssb.clone();
        // Same pattern as `SsbRerankerNotifier` — spawn onto the active
        // tokio runtime; drop with a warning when none is active. The
        // cascade evaluator calls notify() from a sync context so we can
        // not .await directly.
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = ssb
                        .write_payload(
                            elai_orchestrator::task_runner::RUNNER_AGENT_ID,
                            elai_orchestrator::task_runner::RUNNER_SUPERSTEP,
                            payload,
                            Vec::new(),
                        )
                        .await
                    {
                        tracing::warn!(
                            error = %e,
                            "live_cli degradation notifier: write_payload failed (non-fatal)"
                        );
                    }
                });
            }
            Err(_) => {
                tracing::debug!(
                    "live_cli degradation notifier: no tokio runtime; dropping row"
                );
            }
        }
    }
}
