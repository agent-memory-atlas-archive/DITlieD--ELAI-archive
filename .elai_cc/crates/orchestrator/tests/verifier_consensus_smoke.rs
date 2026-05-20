//! D-3 — observable_effect smoke test for the multi-verifier consensus pass.
//!
//! Three mock verifier adapters (a PPM-shaped adapter via D-2's
//! MockSidecarPpmHook, an intrinsic-signal adapter, and a deterministic-check
//! adapter) feed `run_consensus_pass` with a `KofN { k: 2, n: 3 }` rule. The
//! test asserts:
//!   1. consensus = Pass when >= 2 verifiers individually pass.
//!   2. consensus = Fail when only 1 verifier passes.
//!   3. exactly one SSB row is emitted per consensus call (VerifierConsensusPass
//!      on pass / VerifierConsensusFail on fail) and the row content matches
//!      the verdict surface.
//!
//! No sidecar required — adapters synthesise their verdicts in-process so
//! the test runs deterministically in CI without GPU / network.

use std::sync::Arc;

use elai_orchestrator::{
    agent_pool::{
        run_consensus_pass, AdapterErased, DynVerifierAdapter, VecConsensusEmitter,
        VerifierAdapter,
    },
    ppm_client::PpmClient,
    sidecar_ppm_hook::{MockSidecarPpmHook, PpmMode, PpmScoreRequest},
    ssb::SsbPayload,
    verification_result::{
        AggregateVerdict, ConsensusRule, IntrinsicSignals, VerificationResult, VerifierKind,
    },
};

// ---------------------------------------------------------------------------
// Mock verifier adapters
// ---------------------------------------------------------------------------

/// PPM-shaped adapter. Uses the real `PpmClient` + `MockSidecarPpmHook` from
/// D-2 so the smoke test exercises the same code path as production wiring.
struct PpmMockAdapter {
    client: PpmClient<MockSidecarPpmHook>,
    verifier_id: String,
}

impl VerifierAdapter for PpmMockAdapter {
    async fn verify(
        &self,
        candidate_id: &str,
    ) -> Result<VerificationResult, String> {
        let req = PpmScoreRequest {
            candidate_id: candidate_id.into(),
            verifier_id: self.verifier_id.clone(),
            problem_statement: "smoke".into(),
            reasoning_steps: vec!["step1".into(), "step2".into()],
            mode: PpmMode::Math,
        };
        let score = self.client.score(req).await.map_err(|e| e.to_string())?;
        Ok(VerificationResult::from_ppm(candidate_id, &score, 1_700_000_000_000))
    }
}

/// Intrinsic-signal adapter. Emits a fixed `IntrinsicSignals` bundle so the
/// test controls the pass / fail outcome via constructor arguments.
struct IntrinsicMockAdapter {
    verifier_id: String,
    signals: IntrinsicSignals,
}

impl VerifierAdapter for IntrinsicMockAdapter {
    async fn verify(
        &self,
        candidate_id: &str,
    ) -> Result<VerificationResult, String> {
        Ok(VerificationResult::from_intrinsic(
            candidate_id,
            &self.verifier_id,
            self.signals,
            1_700_000_000_001,
        ))
    }
}

/// Deterministic-check adapter. Returns a deterministic binary verdict —
/// stands in for compile / test / lint hooks.
struct DeterministicMockAdapter {
    verifier_id: String,
    passed: bool,
}

impl VerifierAdapter for DeterministicMockAdapter {
    async fn verify(
        &self,
        candidate_id: &str,
    ) -> Result<VerificationResult, String> {
        Ok(VerificationResult::from_deterministic(
            candidate_id,
            &self.verifier_id,
            self.passed,
            1_700_000_000_002,
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn consensus_pass_when_two_of_three_verifiers_pass() {
    // PPM forced to 0.9 (passes default threshold 0.7).
    let ppm_hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.9));
    let ppm_adapter = PpmMockAdapter {
        client: PpmClient::new(ppm_hook),
        verifier_id: "ppm-1".into(),
    };
    // Intrinsic with strong logprob + low entropy — passes default floor.
    let intrinsic_adapter = IntrinsicMockAdapter {
        verifier_id: "intrinsic-1".into(),
        signals: IntrinsicSignals {
            logprob_mean: -1.0,
            logprob_var: 0.1,
            entropy: 1.5,
            step_count: 3,
            kl_to_base: 0.0,
        },
    };
    // Deterministic check fails — only 2 of 3 pass.
    let det_adapter = DeterministicMockAdapter {
        verifier_id: "det-1".into(),
        passed: false,
    };

    let ppm_boxed = AdapterErased::new(ppm_adapter);
    let intr_boxed = AdapterErased::new(intrinsic_adapter);
    let det_boxed = AdapterErased::new(det_adapter);

    let adapters: Vec<&dyn DynVerifierAdapter> =
        vec![&ppm_boxed, &intr_boxed, &det_boxed];

    let emitter = VecConsensusEmitter::new();
    let rule = ConsensusRule::KofN { k: 2, n: 3 };

    let summary = run_consensus_pass(
        "cand-smoke-1",
        "tpl-smoke-1",
        &adapters,
        &rule,
        &emitter,
    )
    .await
    .expect("consensus pass runs to completion");

    assert_eq!(summary.verdict, AggregateVerdict::Pass);
    assert_eq!(summary.results.len(), 3);
    assert_eq!(summary.rule_id, "k_of_n");

    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1, "exactly one SSB row per consensus pass");
    match &rows[0] {
        SsbPayload::VerifierConsensusPass(p) => {
            assert_eq!(p.candidate_id, "cand-smoke-1");
            assert_eq!(p.template_id, "tpl-smoke-1");
            assert_eq!(p.verdict, "pass");
            assert_eq!(p.verifier_count, 3);
            assert_eq!(p.pass_count, 2);
            assert_eq!(p.rule_id, "k_of_n");
        }
        other => panic!("expected VerifierConsensusPass, got {other:?}"),
    }
}

#[tokio::test]
async fn consensus_fail_when_only_one_verifier_passes() {
    // PPM forced low (gate fails).
    let ppm_hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.3));
    let ppm_adapter = PpmMockAdapter {
        client: PpmClient::new(ppm_hook),
        verifier_id: "ppm-1".into(),
    };
    // Intrinsic passes (single pass signal).
    let intrinsic_adapter = IntrinsicMockAdapter {
        verifier_id: "intrinsic-1".into(),
        signals: IntrinsicSignals {
            logprob_mean: -1.0,
            logprob_var: 0.1,
            entropy: 1.5,
            step_count: 3,
            kl_to_base: 0.0,
        },
    };
    // Deterministic check fails.
    let det_adapter = DeterministicMockAdapter {
        verifier_id: "det-1".into(),
        passed: false,
    };

    let ppm_boxed = AdapterErased::new(ppm_adapter);
    let intr_boxed = AdapterErased::new(intrinsic_adapter);
    let det_boxed = AdapterErased::new(det_adapter);

    let adapters: Vec<&dyn DynVerifierAdapter> =
        vec![&ppm_boxed, &intr_boxed, &det_boxed];

    let emitter = VecConsensusEmitter::new();
    let rule = ConsensusRule::KofN { k: 2, n: 3 };

    let summary = run_consensus_pass(
        "cand-smoke-2",
        "tpl-smoke-2",
        &adapters,
        &rule,
        &emitter,
    )
    .await
    .expect("consensus pass runs to completion");

    match &summary.verdict {
        AggregateVerdict::Fail { reasons } => {
            assert!(!reasons.is_empty(), "fail reasons must list the failing verifier ids");
        }
        other => panic!("expected Fail, got {other:?}"),
    }

    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1, "exactly one SSB row per consensus pass");
    match &rows[0] {
        SsbPayload::VerifierConsensusFail(p) => {
            assert_eq!(p.candidate_id, "cand-smoke-2");
            assert_eq!(p.template_id, "tpl-smoke-2");
            assert_eq!(p.reason, "fail");
            assert!(
                p.failing_verifier_ids.len() >= 1,
                "at least one failing verifier id present"
            );
        }
        other => panic!("expected VerifierConsensusFail, got {other:?}"),
    }
}

#[tokio::test]
async fn observable_effect_get_recent_verifications_surface() {
    // Mirrors the WIRING_CONTRACT's observable_effect block: produce one
    // VerifierConsensusPass row, then assert the in-memory emitter (which
    // stands in for the SSB store that backs the `get_recent_verifications`
    // Tauri command) returns the same row.
    let ppm_hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.95));
    let ppm_adapter = PpmMockAdapter {
        client: PpmClient::new(ppm_hook),
        verifier_id: "ppm-1".into(),
    };
    let intrinsic_adapter = IntrinsicMockAdapter {
        verifier_id: "intrinsic-1".into(),
        signals: IntrinsicSignals {
            logprob_mean: -0.5,
            logprob_var: 0.05,
            entropy: 1.2,
            step_count: 4,
            kl_to_base: 0.0,
        },
    };
    let det_adapter = DeterministicMockAdapter {
        verifier_id: "det-1".into(),
        passed: true,
    };

    let ppm_boxed = AdapterErased::new(ppm_adapter);
    let intr_boxed = AdapterErased::new(intrinsic_adapter);
    let det_boxed = AdapterErased::new(det_adapter);
    let adapters: Vec<&dyn DynVerifierAdapter> =
        vec![&ppm_boxed, &intr_boxed, &det_boxed];

    let emitter = VecConsensusEmitter::new();
    let rule = ConsensusRule::VerifierKindHeterogeneous {
        required_kinds: vec![
            VerifierKind::Ppm,
            VerifierKind::Intrinsic,
            VerifierKind::Deterministic,
        ],
    };

    let summary = run_consensus_pass(
        "cand-obs-1",
        "tpl-obs-1",
        &adapters,
        &rule,
        &emitter,
    )
    .await
    .expect("consensus runs");

    assert_eq!(summary.verdict, AggregateVerdict::Pass);
    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1);
    // The dashboard `get_recent_verifications` command reads exactly this
    // payload family — assert the row carries the canonical fields.
    match &rows[0] {
        SsbPayload::VerifierConsensusPass(p) => {
            assert_eq!(p.candidate_id, "cand-obs-1");
            assert_eq!(p.rule_id, "verifier_kind_heterogeneous");
        }
        other => panic!("expected VerifierConsensusPass row, got {other:?}"),
    }
}
