//! C-52 — Dafny Formal Verification Spike (host-side scaffolding).
//!
//! Paper grounding: arXiv:2604.22601 (Md Erfan, Md Kamal Hossain
//! Chowdhury, Ahmed Ryan, Md Rayhanur Rahman 2026 — "From Natural
//! Language to Verified Code: Toward AI Assisted Problem-to-Code
//! Generation with Dafny-Based Formal Verification"). Verified via
//! arxiv.org/abs/2604.22601 + arxiv.org/html/2604.22601 +
//! WebSearch cross-confirmation on 2026-05-24. Interface map:
//! `workflow/papers/nl2vc-dafny-interface.md`.
//!
//! CITATION DISCIPLINE (load-bearing — do NOT misframe):
//!   - The paper's top performer is Gemma 4-31B (RQ3a 90.91%
//!     verify@5; RQ3b-tied 81.82%). ELAI is BLOCKED from
//!     integrating Gemma 4 per memory rule `feedback_no_gemma4.md`.
//!     ELAI substitutes from the paper's evaluation set:
//!     Codestral-22B-v0.1 (63.64% RQ2), GPT-OSS-120B (63.64% RQ2 +
//!     81.82% RQ3b), Qwen 3 Coder 30B (45.45% RQ2). Current ELAI
//!     sidecar default (Qwen 2.5 Coder 7B) is NOT in the paper's
//!     evaluation set; expected verify@5 below 45% on the
//!     signature tier.
//!   - The uDebug functional-test guard is MANDATORY per paper §3
//!     (vacuous-verification failure mode): raw `dafny verify`
//!     exit-zero is INSUFFICIENT; a spec MUST ALSO pass functional
//!     tests before claiming verified.
//!   - Self-healing iteration cap is 10 per paper §3; the paper
//!     does not characterise diminishing returns past 10.
//!   - Temperature sweep is {0.0, 0.2, 0.4, 0.6, 0.8} per paper
//!     Section 3.
//!   - verify@k reported for k in {1, 3, 5}.
//!   - Live `dafny verify` execution is `RUNPOD-PENDING:
//!     live_dafny_verification_with_non_gemma_substitution`.
//!
//! Contract fences (D1..D7):
//!   D1. [`DafnyTier`] has exactly 3 variants (Contextless /
//!       Signature / SelfHealing) per paper RQ1/RQ2/RQ3.
//!   D2. Self-healing iteration cap `MAX_HEAL_ITER` is 10 per paper
//!       §3 (defensive invariant — see `heal_loop::self_heal_loop`).
//!   D3. `udebug_guard::check_vacuous` flags any spec with no
//!       functional tests OR any failing functional test as VACUOUS
//!       (paper §3).
//!   D4. Temperature sweep set is {0.0, 0.2, 0.4, 0.6, 0.8} per
//!       paper Section 3.
//!   D5. verify@k uses {1, 3, 5} per paper Section 3.
//!   D6. ELAI substitute model list contains NO Gemma 4 entry per
//!       memory rule `feedback_no_gemma4`. Soft check via const
//!       list; explicit Gemma 4 string is BLOCKED.
//!   D7. New `SsbActionType::DafnyVerifyCompleted` variant round-
//!       trips through `as_sql_str` + `ssb_action_type_from_str_public`
//!       and its payload carries the correct discriminant via
//!       `SsbPayload::action_type`. SCHEMA_VERSION floor for the
//!       C-52 cascade (>= 126).

use elai_orchestrator::dafny_verify::{
    build_dafny_prompt, check_vacuous, DafnyTier, FunctionalTestResult, MAX_HEAL_ITER,
    TemperatureSweep, VerifyAt, ELAI_SUBSTITUTE_MODELS, GEMMA4_BLOCKED, TEMPERATURE_SWEEP,
    VERIFY_K_VALUES,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, DafnyVerifyCompletedPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// D1 — DafnyTier has 3 variants per paper RQ1/RQ2/RQ3.
// ---------------------------------------------------------------------------

#[test]
fn d1_dafny_tier_has_three_variants_per_paper_rq() {
    // Paper RQ1 (Contextless) + RQ2 (Signature) + RQ3 (SelfHealing).
    assert_eq!(
        DafnyTier::ALL.len(),
        3,
        "paper arXiv:2604.22601 RQ1/RQ2/RQ3 require exactly 3 tiers"
    );
    assert!(DafnyTier::ALL.contains(&DafnyTier::Contextless));
    assert!(DafnyTier::ALL.contains(&DafnyTier::Signature));
    assert!(DafnyTier::ALL.contains(&DafnyTier::SelfHealing));
}

#[test]
fn d1_tier_wire_tags_match_paper_convention() {
    assert_eq!(DafnyTier::Contextless.as_str(), "contextless");
    assert_eq!(DafnyTier::Signature.as_str(), "signature");
    assert_eq!(DafnyTier::SelfHealing.as_str(), "self_healing");
}

#[test]
fn d1_signature_required_for_rq2_rq3() {
    // Paper §3: signature is mandatory for any non-trivial result;
    // contextless near-universal failure (6 of 7 models at 0% verify@5).
    assert!(!DafnyTier::Contextless.requires_signature());
    assert!(DafnyTier::Signature.requires_signature());
    assert!(DafnyTier::SelfHealing.requires_signature());
}

#[test]
fn d1_build_dafny_prompt_includes_tier_header() {
    // Per-tier prompt body must carry the tier name so downstream
    // tooling (loggers, SSB consumers, dashboard) can introspect.
    let p = build_dafny_prompt("compute the GCD", DafnyTier::Signature, &[]);
    assert!(p.contains("tier=signature"));
    let p = build_dafny_prompt("compute the GCD", DafnyTier::SelfHealing, &[]);
    assert!(p.contains("tier=self_healing"));
}

// ---------------------------------------------------------------------------
// D2 — Self-healing iteration cap is 10 per paper §3.
// ---------------------------------------------------------------------------

#[test]
fn d2_heal_loop_cap_matches_paper_section_3() {
    // Paper §3 caps self-healing at 10 iterations. Do not raise
    // without a fresh ablation.
    assert_eq!(
        MAX_HEAL_ITER, 10,
        "paper arXiv:2604.22601 §3 caps self-healing at 10 iterations"
    );
}

// ---------------------------------------------------------------------------
// D3 — uDebug vacuous-verification guard (paper §3).
// ---------------------------------------------------------------------------

#[test]
fn d3_udebug_guard_flags_empty_test_suite_as_vacuous() {
    let spec = "method foo() returns (x: int) { x := 0; }";
    let tests: Vec<FunctionalTestResult> = Vec::new();
    assert!(
        check_vacuous(spec, &tests),
        "empty functional-test suite must be flagged vacuous per paper §3"
    );
}

#[test]
fn d3_udebug_guard_flags_failing_test_as_vacuous() {
    let spec = "method foo() returns (x: int) { x := 0; }";
    let tests = vec![
        FunctionalTestResult::new("uva-100-case-1", true),
        FunctionalTestResult::new("uva-100-case-2", false),
    ];
    assert!(
        check_vacuous(spec, &tests),
        "any failing functional test must flip the vacuous-verification guard"
    );
}

#[test]
fn d3_udebug_guard_passes_when_all_tests_pass() {
    let spec = "method foo() returns (x: int) { x := 0; }";
    let tests = vec![
        FunctionalTestResult::new("uva-100-case-1", true),
        FunctionalTestResult::new("uva-100-case-2", true),
    ];
    assert!(
        !check_vacuous(spec, &tests),
        "every functional test passing means non-vacuous"
    );
}

// ---------------------------------------------------------------------------
// D4 — Temperature sweep matches paper Section 3.
// ---------------------------------------------------------------------------

#[test]
fn d4_temperature_sweep_matches_paper_section_3() {
    // Paper Section 3: T ∈ {0.0, 0.2, 0.4, 0.6, 0.8}.
    let sweep = TemperatureSweep::canonical();
    assert_eq!(sweep.points.len(), 5);
    assert!(sweep.contains(0.0));
    assert!(sweep.contains(0.2));
    assert!(sweep.contains(0.4));
    assert!(sweep.contains(0.6));
    assert!(sweep.contains(0.8));
    // Outside the sweep.
    assert!(!sweep.contains(1.0));
    assert!(!sweep.contains(0.5));
}

#[test]
fn d4_temperature_sweep_const_is_stable() {
    assert_eq!(TEMPERATURE_SWEEP, [0.0f32, 0.2, 0.4, 0.6, 0.8]);
}

// ---------------------------------------------------------------------------
// D5 — verify@k uses {1, 3, 5} per paper Section 3.
// ---------------------------------------------------------------------------

#[test]
fn d5_verify_k_values_match_paper() {
    assert_eq!(VERIFY_K_VALUES, [1u32, 3, 5]);
}

#[test]
fn d5_verify_at_monotonic_invariant() {
    // Per paper, more samples can only increase the at-least-one-pass
    // count: verify@1 <= verify@3 <= verify@5.
    let good = VerifyAt::new(3, 7, 10);
    assert!(good.is_monotonic());
    let bad = VerifyAt::new(10, 5, 3);
    assert!(!bad.is_monotonic());
}

// ---------------------------------------------------------------------------
// D6 — ELAI substitutes are NOT Gemma 4 per memory feedback_no_gemma4.
// ---------------------------------------------------------------------------

#[test]
fn d6_elai_substitutes_contain_no_gemma4() {
    assert_eq!(
        GEMMA4_BLOCKED, "Gemma 4-31B",
        "GEMMA4_BLOCKED const must match the paper's top-performer string"
    );
    // Soft check: no substitute model carries the literal "Gemma 4"
    // string. The block is informational; the wire-tag enforcement
    // lives in the sidecar selection layer (memory rule
    // feedback_no_gemma4).
    for m in ELAI_SUBSTITUTE_MODELS.iter() {
        assert!(
            !m.contains("Gemma 4"),
            "ELAI substitute list must not include `{m}` (Gemma 4 blocked per memory:feedback_no_gemma4)"
        );
    }
    // Sanity: at least one paper-derived non-Gemma top performer is
    // in the list. Codestral-22B-v0.1 has 63.64% RQ2 verify@5 per
    // paper Table III.
    assert!(
        ELAI_SUBSTITUTE_MODELS
            .iter()
            .any(|m| m.contains("Codestral-22B")),
        "substitute list should retain Codestral-22B as a top non-Gemma performer"
    );
}

// ---------------------------------------------------------------------------
// D7 — SSB variant round-trip + SCHEMA_VERSION floor.
// ---------------------------------------------------------------------------

#[test]
fn d7_dafny_verify_completed_action_type_round_trips() {
    let s = SsbActionType::DafnyVerifyCompleted.as_sql_str();
    assert_eq!(s, "dafny_verify_completed");
    let back = ssb_action_type_from_str_public(s).expect("from_str");
    assert_eq!(back, SsbActionType::DafnyVerifyCompleted);
}

#[test]
fn d7_dafny_verify_completed_payload_carries_correct_discriminant() {
    let payload = DafnyVerifyCompletedPayload {
        tier_hash: 0xC52C_52C5_2C52_C52C,
        dafny_pass: true,
        udebug_pass: true,
        heal_iter: 3,
        ts_ms: 1_748_044_900_000,
    };
    let p = SsbPayload::DafnyVerifyCompleted(payload);
    assert_eq!(p.action_type(), SsbActionType::DafnyVerifyCompleted);
}

#[test]
fn d7_schema_version_floor_for_c52_cascade() {
    // C-52 lands at SCHEMA_VERSION 126 (atomic +1 from C-50 at 125).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 126,
        "C-52 cascade requires SCHEMA_VERSION_PUBLIC >= 126 (was {})",
        SCHEMA_VERSION_PUBLIC
    );
}
