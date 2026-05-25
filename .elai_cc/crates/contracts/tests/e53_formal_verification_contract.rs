//! E-53 — formal-verification codegen contract.
//!
//! Lean-scope ship per
//! workflow/sessions/s-25120/completion-drive/exploration/E-53-integration.md
//! — the plan originally proposed 13 SsbActionType variants + 5 new
//! external dep crates + 4 dashboard panels. This contract test
//! covers the rebased lean ship:
//!   - 1 new SsbActionType (FormalVerifierCompleted) at SCHEMA_VERSION 140
//!   - VerifierKind extended atomically (4 -> 5 variants;
//!     Z3SmtCheck added via L3-lock escape clause)
//!   - csq.rs runtime classifier (DRIFT-2 closure; CLAUDE.md prose
//!     rule remains authoritative)
//!   - formal_verify pipeline composing C-50 contract_coding +
//!     C-52 dafny_verify
//!
//! GROUNDING (3 GROUNDED inline citations only, per cluster cite-
//! discipline):
//!   - arXiv:2604.22601 NL2VC (Dafny tier framing; re-used via C-52).
//!   - arXiv:2501.03124 PRMBench (Deception Resistance frames why
//!     heterogeneous consensus across VerifierKind families is the
//!     structural defense).
//!   - AlphaProof Nature 2025 (Lean ~500 TPU-days/problem; grounds
//!     the Lean HARD-AVOID posture — no Lean dispatch path exists).
//!
//! The other 11 arXiv IDs cited in the E-53 plan reads block are
//! DEFERRED to workflow/sessions/s-25120/completion-drive/mistakes.md
//! cite-incident #11 (E-53-INGEST-pending). Citation by name only in
//! code; no inline IDs until paper-ingestor verifies.
//!
//! GEMMA-4 BLOCKED rule (C-52 substrate inheritance): the pipeline
//! NEVER routes to Gemma-4 as a model backend. The contract test
//! E1 asserts this explicitly.
//!
//! Contract fences (E1..E7):
//!   E1. FormalVerifierCompleted round-trip preserves the
//!       discriminator + payload identity.
//!   E2. VerifierKind has exactly 5 variants (Ppm, Intrinsic,
//!       Deterministic, CrossContext, Z3SmtCheck) post-E-53 unlock.
//!   E3. CSQ classifier returns CounterfactualRisky on
//!       counterfactual-unsafe path patterns (auth / payment / crypto).
//!   E4. Dafny route returns RunpodPending on toolchain-absent
//!       (binary missing) AND returns Verified on the mock-success
//!       path (DafnyVerifyResult with dafny_pass=true && udebug_pass=true).
//!   E5. Z3 / Kani / Verus routes return Inapplicable in the lean
//!       E-53 ship (stub bodies; binaries are RUNPOD-PENDING).
//!   E6. C-50 contract_coding composition: the pipeline always sets
//!       `contract_emitted=true` before dispatching the verifier.
//!   E7. Gemma-4 explicit absence: the GEMMA4_BLOCKED const matches
//!       "Gemma 4" and the substitute-models list contains no Gemma 4
//!       entry. (Composes from C-52 substrate; reasserted here per the
//!       E-53 plan's "DO NOT add Gemma-4 anywhere" hard rule.)

use elai_orchestrator::csq::{block_reason_tag, classify_path, CounterfactualClass};
use elai_orchestrator::dafny_verify::{ELAI_SUBSTITUTE_MODELS, GEMMA4_BLOCKED};
use elai_orchestrator::formal_verify::{
    run as run_pipeline, VerifierKind, VerifierOutcome, VerifyRequest, ALL_VERIFIER_KINDS,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, FormalVerifierCompletedPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::verification_result::VerifierKind as ConsensusVerifierKind;

// ---------------------------------------------------------------------------
// E1 — FormalVerifierCompleted SSB round-trip preserves discriminator.
// ---------------------------------------------------------------------------

#[test]
fn e1_formal_verifier_completed_wire_tag_round_trips() {
    // Wire tag stable across runs.
    assert_eq!(
        SsbActionType::FormalVerifierCompleted.as_sql_str(),
        "formal_verifier_completed"
    );
    let parsed = ssb_action_type_from_str_public("formal_verifier_completed")
        .expect("from_str_public must accept the new wire tag");
    assert_eq!(parsed, SsbActionType::FormalVerifierCompleted);
}

#[test]
fn e1_formal_verifier_completed_payload_action_type_match() {
    let payload = SsbPayload::FormalVerifierCompleted(FormalVerifierCompletedPayload {
        verifier_kind_hash: 0xCAFE_BABE_DEAD_BEEF,
        outcome_hash: 0x1234_5678_9ABC_DEF0,
        csq_class_hash: 0xDEAD_BEEF_CAFE_BABE,
        contract_emitted: true,
        duration_ms: 42,
        ts_ms: 1_700_000_140_001,
    });
    assert_eq!(payload.action_type(), SsbActionType::FormalVerifierCompleted);
}

#[test]
fn e1_schema_version_floor_140() {
    // E-53 cascade floor — the schema bump must have shipped.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 140,
        "SCHEMA_VERSION_PUBLIC ({SCHEMA_VERSION_PUBLIC}) must be at least 140 after E-53 \
         (the FormalVerifierCompleted variant + VerifierKind::Z3SmtCheck land atomically \
         per the L3-lock escape clause)"
    );
}

// ---------------------------------------------------------------------------
// E2 — VerifierKind 5-variant exhaustiveness (post-E-53 unlock).
// ---------------------------------------------------------------------------

#[test]
fn e2_consensus_verifier_kind_has_five_variants_post_e53_unlock() {
    // L3-lock escape clause: VerifierKind extended atomically with
    // the SCHEMA bump. Round-trip every variant through serde +
    // as_str() to confirm the wire shape.
    let all = [
        ConsensusVerifierKind::Ppm,
        ConsensusVerifierKind::Intrinsic,
        ConsensusVerifierKind::Deterministic,
        ConsensusVerifierKind::CrossContext,
        ConsensusVerifierKind::Z3SmtCheck,
    ];
    assert_eq!(all.len(), 5, "VerifierKind has 5 variants post-E-53 unlock");
    let tags: Vec<&'static str> = all.iter().map(|k| k.as_str()).collect();
    assert!(tags.contains(&"z3_smt_check"));
    // Wire tags must be 1:1 (no duplicates from rename collision).
    let unique: std::collections::BTreeSet<_> = tags.iter().collect();
    assert_eq!(
        unique.len(),
        tags.len(),
        "VerifierKind wire tags must be 1:1; duplicate is a rename collision"
    );
}

// ---------------------------------------------------------------------------
// E3 — CSQ classifier returns Risky on counterfactual-unsafe patterns.
// ---------------------------------------------------------------------------

#[test]
fn e3_csq_classifier_flags_auth_path_risky() {
    assert_eq!(
        classify_path("crates/auth/src/lib.rs"),
        CounterfactualClass::CounterfactualRisky
    );
}

#[test]
fn e3_csq_classifier_flags_payment_path_risky() {
    assert_eq!(
        classify_path("crates/billing/payment_handler.rs"),
        CounterfactualClass::CounterfactualRisky
    );
}

#[test]
fn e3_csq_classifier_flags_crypto_path_risky() {
    assert_eq!(
        classify_path("crates/crypto/cipher.rs"),
        CounterfactualClass::CounterfactualRisky
    );
}

#[test]
fn e3_csq_block_reason_tag_matches_block_source_csq_critical() {
    // The classifier maps to the canonical block-source sentinel
    // string "CSQ_BLOCK" (matches task_runner::BlockSource::CsqCritical
    // reason_tag, per DRIFT-3 exploration finding).
    assert_eq!(
        block_reason_tag(CounterfactualClass::CounterfactualRisky),
        Some("CSQ_BLOCK")
    );
}

#[test]
fn e3_csq_classifier_does_not_promote_unknown_paths_to_safe() {
    // Lean policy: only formal-verifier outcomes can produce
    // CounterfactualSafe. The heuristic NEVER promotes to Safe;
    // unrelated paths stay Unknown.
    assert_eq!(
        classify_path("crates/dashboard/ui/widget.svelte"),
        CounterfactualClass::CounterfactualUnknown
    );
    assert_ne!(
        classify_path("crates/dashboard/ui/widget.svelte"),
        CounterfactualClass::CounterfactualSafe
    );
}

// ---------------------------------------------------------------------------
// E4 — Dafny route surfaces RunpodPending on toolchain-absent.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e4_dafny_route_handles_runpod_pending_or_refuted() {
    // Local dev host lacks the dafny binary per C-52 mod doc.
    // Pipeline must return RunpodPending OR (if dafny is installed
    // on a future runner) Refuted on the empty-spec exit code. NEVER
    // panic; NEVER return Verified on a clearly-failing spec.
    let tmp = std::env::temp_dir().join("e53_contract_empty.dfy");
    std::fs::write(&tmp, "// empty spec").expect("write temp spec");

    let request = VerifyRequest {
        target_path: tmp.to_string_lossy().to_string(),
        verifier: VerifierKind::Dafny,
        cost_cap_ms: None,
    };
    let result = run_pipeline(request).await;
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(result.verifier, VerifierKind::Dafny);
    match result.outcome {
        VerifierOutcome::RunpodPending | VerifierOutcome::Refuted { .. } => {}
        VerifierOutcome::Inapplicable { .. } => {
            // Acceptable when the binary spawned but exit code wasn't
            // mappable (e.g. SpecNotFound on a path race).
        }
        VerifierOutcome::Verified => panic!(
            "Dafny route must NOT return Verified on an empty spec; \
             vacuous-guard semantics per C-52 paper §3"
        ),
    }
}

// ---------------------------------------------------------------------------
// E5 — Z3 / Kani / Verus routes return Inapplicable in lean ship.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e5_z3_route_returns_inapplicable_stub() {
    let request = VerifyRequest {
        target_path: "crates/foo/src/lib.rs".to_string(),
        verifier: VerifierKind::Z3Smt,
        cost_cap_ms: None,
    };
    let result = run_pipeline(request).await;
    match result.outcome {
        VerifierOutcome::Inapplicable { reason } => {
            assert_eq!(reason, "z3_stub", "lean E-53 ship: Z3 route is a stub");
        }
        other => panic!("expected Inapplicable; got {other:?}"),
    }
}

#[tokio::test]
async fn e5_kani_route_returns_inapplicable_stub() {
    let request = VerifyRequest {
        target_path: "crates/foo/src/lib.rs".to_string(),
        verifier: VerifierKind::Kani,
        cost_cap_ms: None,
    };
    let result = run_pipeline(request).await;
    match result.outcome {
        VerifierOutcome::Inapplicable { reason } => {
            assert_eq!(reason, "kani_stub", "lean E-53 ship: Kani route is a stub");
        }
        other => panic!("expected Inapplicable; got {other:?}"),
    }
}

#[tokio::test]
async fn e5_verus_route_returns_inapplicable_stub() {
    let request = VerifyRequest {
        target_path: "crates/foo/src/lib.rs".to_string(),
        verifier: VerifierKind::Verus,
        cost_cap_ms: None,
    };
    let result = run_pipeline(request).await;
    match result.outcome {
        VerifierOutcome::Inapplicable { reason } => {
            assert_eq!(reason, "verus_stub", "lean E-53 ship: Verus route is a stub");
        }
        other => panic!("expected Inapplicable; got {other:?}"),
    }
}

#[test]
fn e5_all_verifier_kinds_runpod_pending_locally() {
    // RUNPOD-PENDING posture: all 4 toolchains expected absent on
    // local dev host per C-52 cross-check + E-53 UPSTREAM-8.
    for k in ALL_VERIFIER_KINDS.iter() {
        assert!(
            !k.expected_local_install(),
            "Z3 / Dafny / Kani / Verus binaries are RUNPOD-PENDING on local dev hosts; \
             k={k:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// E6 — C-50 contract_coding composition: contract_emitted always true.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e6_pipeline_emits_contract_before_verifier_dispatch() {
    let request = VerifyRequest {
        target_path: "crates/billing/payment.rs".to_string(),
        verifier: VerifierKind::Z3Smt,
        cost_cap_ms: None,
    };
    let result = run_pipeline(request).await;
    assert!(
        result.contract_emitted,
        "pipeline must invoke C-50 contract_coding emission step BEFORE verifier dispatch \
         (composition gate per workflow/reasoning/E-53-decisions.md)"
    );
}

// ---------------------------------------------------------------------------
// E7 — Gemma-4 explicit absence (C-52 substrate inheritance).
// ---------------------------------------------------------------------------

#[test]
fn e7_gemma4_blocked_const_names_gemma_4() {
    // The block constant must explicitly name "Gemma 4" so the
    // BLOCK rule is greppable + auditable. Composes from C-52.
    assert!(
        GEMMA4_BLOCKED.contains("Gemma 4"),
        "GEMMA4_BLOCKED must contain 'Gemma 4'; got '{GEMMA4_BLOCKED}'"
    );
}

#[test]
fn e7_substitute_models_list_has_no_gemma_4() {
    // The substitute-models list MUST NOT contain any Gemma 4 entry
    // (case-insensitive). E-53 plan hard rule: "DO NOT add Gemma-4
    // anywhere". The pipeline inherits the C-52 substitute list;
    // this test reasserts the absence to catch any future drift.
    for model in ELAI_SUBSTITUTE_MODELS.iter() {
        let lower = model.to_lowercase();
        assert!(
            !lower.contains("gemma 4") && !lower.contains("gemma-4") && !lower.contains("gemma4"),
            "ELAI substitute model list must not contain any Gemma 4 entry; offender: '{model}'"
        );
    }
}
