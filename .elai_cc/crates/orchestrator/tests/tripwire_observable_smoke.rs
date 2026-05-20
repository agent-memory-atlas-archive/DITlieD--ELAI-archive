//! D-4 — observable_effect smoke test for the tripwire stack.
//!
//! Feeds a synthetic IntrinsicSignals stream with a deliberate outlier
//! into the [`TripwireMonitor`], asserts that a `TripwireFired` SSB row
//! is emitted, drives `auto_revert` against a live [`ArchiveChain`], and
//! asserts the matching [`ArchiveEntry`] flipped to `Retired` with
//! `RetirementReason::Degradation`.
//!
//! Run with: `cargo test -p elai-orchestrator --test tripwire_observable_smoke`.

use std::path::PathBuf;

use elai_orchestrator::agent_pool::observe_post_consensus_tripwires;
use elai_orchestrator::archive_chain::{ArchiveChain, ArchiveEntry};
use elai_orchestrator::reasoning_template::{
    compute_content_hash, ArchiveStatus, Domain, PatternKind, Predicate, ReasoningTemplate,
    RetirementReason, TemplateBody, TemplateId, VerifierKind,
};
use elai_orchestrator::ssb::SsbPayload;
use elai_orchestrator::tripwire::{
    TripwireConfig, TripwireMonitor, TripwireSample, VecTripwireEmitter,
};
use elai_orchestrator::verification_result::IntrinsicSignals;
use tempfile::TempDir;

fn mk_signals(logprob_mean: f32, logprob_var: f32, entropy: f32) -> IntrinsicSignals {
    IntrinsicSignals {
        logprob_mean,
        logprob_var,
        entropy,
        step_count: 1,
        kl_to_base: 0.0,
    }
}

fn mk_sample(passed: bool, signals: IntrinsicSignals, ts_ms: i64) -> TripwireSample {
    TripwireSample {
        signals,
        proxy: Some(0.9),
        gold: Some(1.0),
        passed,
        ts_ms,
    }
}

fn jsonl_path(dir: &TempDir) -> PathBuf {
    dir.path().join("archive.jsonl")
}

fn mk_promoted_template() -> ReasoningTemplate {
    let body = TemplateBody::try_new("smoke body").expect("body");
    let hash = compute_content_hash(body.as_str(), None);
    ReasoningTemplate {
        id: TemplateId::new_v4(),
        domain: Domain::Code,
        pattern_kind: PatternKind::Decomposition,
        template_body: body,
        eligibility_predicate: Predicate::Always,
        verifier_required: VerifierKind::Compile,
        content_hash: hash,
        created_at_ms: 0,
        parent_id: None,
        archive_status: ArchiveStatus::Promoted,
    }
}

#[tokio::test]
async fn tripwire_observable_smoke_fires_and_auto_reverts() {
    // 1. Build the monitor with a small min_sample_count so the smoke test
    //    stays small + deterministic.
    let cfg = TripwireConfig {
        min_sample_count: 8,
        window_size: 32,
        ..TripwireConfig::defaults()
    };
    let mut mon = TripwireMonitor::new(cfg);

    // 2. Seed the archive chain with a promoted template.
    let dir = TempDir::new().expect("tempdir");
    let mut chain = ArchiveChain::new(jsonl_path(&dir));
    let template = mk_promoted_template();
    let template_id = template.id;
    chain
        .append(ArchiveEntry::new_root(template))
        .expect("append root");

    let emitter = VecTripwireEmitter::new();
    let candidate_id = "smoke-cand-1";

    // 3. Feed 10 calm samples — variance + ppl steady, gold/proxy aligned,
    //    all pass=true. No firings expected (DDM stays Normal; window
    //    detectors stay below threshold).
    for i in 0..10 {
        let outcome = observe_post_consensus_tripwires(
            &mut mon,
            Some(&mut chain),
            Some(&template_id),
            candidate_id,
            mk_sample(true, mk_signals(-1.0, 0.05, 1.0), 1_700_000_000_000 + i),
            &emitter,
        )
        .await
        .expect("observe ok");
        assert!(
            outcome.fired.is_empty(),
            "tick {i} should not fire on calm stream, got {:?}",
            outcome.fired
        );
    }

    // 4. Inject a tick that trips ≥2 detectors simultaneously: huge ppl
    //    spike (logprob_mean = -20) + huge logprob_var spike (50.0). Both
    //    IqrOutlier (var) and Perplexity must fire.
    let outcome = observe_post_consensus_tripwires(
        &mut mon,
        Some(&mut chain),
        Some(&template_id),
        candidate_id,
        mk_sample(false, mk_signals(-20.0, 50.0, 7.5), 1_700_000_000_100),
        &emitter,
    )
    .await
    .expect("observe ok");

    assert!(
        outcome.fired.len() >= 2,
        "expected ≥2 concurrent firings on spike tick, got {}: {:?}",
        outcome.fired.len(),
        outcome
            .fired
            .iter()
            .map(|f| f.kind.as_str())
            .collect::<Vec<_>>()
    );

    // 5. Assert at least one TripwireFired row + exactly one
    //    TemplateAutoReverted row in the SSB emitter buffer.
    let rows = emitter.rows().await;
    let fired_count = rows
        .iter()
        .filter(|p| matches!(p, SsbPayload::TripwireFired(_)))
        .count();
    let reverted_count = rows
        .iter()
        .filter(|p| matches!(p, SsbPayload::TemplateAutoReverted(_)))
        .count();
    assert!(
        fired_count >= 2,
        "expected ≥2 TripwireFired rows, got {fired_count}: {rows:?}"
    );
    assert_eq!(
        reverted_count, 1,
        "expected exactly one TemplateAutoReverted row, got {reverted_count}"
    );

    // 6. The archive must show the template as Retired with
    //    RetirementReason::Degradation (latest entry for this id).
    let latest = chain
        .entries()
        .iter()
        .filter(|e| e.template.id == template_id)
        .last()
        .expect("at least one entry");
    assert_eq!(latest.template.archive_status, ArchiveStatus::Retired);
    assert_eq!(
        latest.retirement_reason,
        Some(RetirementReason::Degradation),
        "retirement reason mismatch"
    );

    // 7. The TemplateAutoReverted SSB row's template_id matches the
    //    retired template (the producer wiring's identity property).
    let reverted = rows
        .iter()
        .find_map(|p| match p {
            SsbPayload::TemplateAutoReverted(payload) => Some(payload.clone()),
            _ => None,
        })
        .expect("auto reverted payload present");
    assert_eq!(reverted.template_id, template_id.as_hex());
    assert_eq!(reverted.retirement_reason, "degradation");
}
