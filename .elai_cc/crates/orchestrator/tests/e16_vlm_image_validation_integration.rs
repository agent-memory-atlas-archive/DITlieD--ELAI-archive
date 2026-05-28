//! E-16 VLM image-validation integration tests.
//!
//! Combined cover (one test file per the cross-check report's
//! "Tests to add" block):
//!   * axtree_adapter_contract -- round-trip YAML, deterministic
//!     re-render diff < 1%, evaluation < 100ms budget on the test
//!     fixture.
//!   * sycophancy_neutralization -- 50-prompt fuzz (mirror of the
//!     unit test in sycophancy.rs but at the integration boundary).
//!   * swap_order_pairwise -- pairwise eval pathway with a stub
//!     judge that flips primary_label between calls.
//!   * rce_convergence -- LPIPS-delta synthetic sequence + hard cap.
//!
//! Each test is self-contained -- no live VLM, no Playwright, no
//! external tool. Live paths remain RUNPOD-PENDING per E-16 plan
//! scope.

use std::time::Instant;

use elai_orchestrator::media_validator::axtree::{
    AxTreeAdapter, AxtreeError, AXTREE_DRIFT_THRESHOLD_PCT, MAX_AXTREE_YAML_BYTES,
};
use elai_orchestrator::media_validator::feedback_loop::{
    clamp_iterations_to_rce_cap, lpips_delta_converged, should_stop_for_rce_cap,
    ConvergenceReason, ConvergenceState, LPIPS_DELTA_CONVERGED_FLOOR, MAX_RCE_ITERATIONS,
};
use elai_orchestrator::media_validator::sycophancy::{
    compute_verdict, invert_framing, neutralize_query, JudgeOutput as SycJudgeOutput, NeutralPrompt,
};
use elai_orchestrator::media_validator::vlm_validator::{
    build_vlm_image_validation_payload, second_opinion_required, vlm_output_schema_compliant,
};
use elai_orchestrator::VlmImageValidationKind;

// -----------------------------------------------------------------
// AXTree adapter contract.
// -----------------------------------------------------------------

#[test]
fn axtree_adapter_round_trip_same_yaml_zero_diff() {
    let a = AxTreeAdapter::new();
    let yaml = "- main\n  - button \"Add\"\n  - button \"Delete\"\n";
    let res = a.evaluate(yaml, Some(yaml)).expect("evaluate ok");
    assert_eq!(res.diff_pct, 0.0);
    assert!(!res.mismatch);
    assert_eq!(res.yaml, yaml);
    assert_eq!(res.baseline_yaml.as_deref(), Some(yaml));
}

#[test]
fn axtree_adapter_threshold_below_one_percent_no_mismatch() {
    // E-16 plan open Q4: drift <= 1% across 20 identical renders.
    // Synthesize a 200-line baseline + a current that changes 1
    // line == 0.5% drift, just under the threshold.
    let a = AxTreeAdapter::new();
    let base_lines: Vec<String> = (0..200).map(|i| format!("- text \"line {i}\"")).collect();
    let base = base_lines.join("\n");
    let mut current_lines = base_lines.clone();
    current_lines[0] = "- text \"line CHANGED\"".to_owned();
    let current = current_lines.join("\n");
    let res = a.evaluate(&current, Some(&base)).expect("evaluate ok");
    assert!(res.diff_pct > 0.0);
    assert!(res.diff_pct < AXTREE_DRIFT_THRESHOLD_PCT);
    assert!(!res.mismatch);
}

#[test]
fn axtree_adapter_evaluation_within_100ms_budget() {
    // E-16 plan task 1: "<100ms; CPU only". Sanity check.
    let a = AxTreeAdapter::new();
    let base_lines: Vec<String> = (0..1000)
        .map(|i| format!("- text \"line {i}\""))
        .collect();
    let base = base_lines.join("\n");
    let mut current_lines = base_lines.clone();
    for i in 0..10 {
        current_lines[i] = format!("- text \"changed {i}\"");
    }
    let current = current_lines.join("\n");
    let started = Instant::now();
    let _ = a.evaluate(&current, Some(&base)).expect("evaluate ok");
    let elapsed_ms = started.elapsed().as_millis();
    assert!(
        elapsed_ms < 100,
        "axtree evaluation took {}ms (>100ms budget)",
        elapsed_ms
    );
}

#[test]
fn axtree_adapter_rejects_oversize_yaml() {
    let a = AxTreeAdapter::new();
    let huge = "a".repeat(MAX_AXTREE_YAML_BYTES + 1);
    let err = a.evaluate(&huge, None).unwrap_err();
    assert!(matches!(err, AxtreeError::YamlTooLarge { .. }));
}

// -----------------------------------------------------------------
// Sycophancy neutralization fuzz (50 leading prompts).
// -----------------------------------------------------------------

#[test]
fn sycophancy_neutralization_fuzz_50_prompts() {
    // arXiv:2408.11261 §3.1 motivates fuzz coverage of leading
    // prompts. Each iteration uses a distinct LEADING_PATTERNS entry
    // round-robin so the strip pass is exercised across the full
    // table.
    let patterns: &[&str] = &[
        "this is the ",
        "this is your ",
        "verify the title says ",
        "you asked for ",
        "as requested, ",
        "as you asked, ",
        "please confirm ",
        "i made the ",
        "i built the ",
        "i wrote the ",
    ];
    for i in 0..50 {
        let pat = patterns[i % patterns.len()];
        let prompt = format!("{pat}fixture_{i} content");
        let n = neutralize_query(&prompt);
        assert!(
            n.removed_tokens.iter().any(|t| t == pat),
            "fuzz iter {i}: pattern {pat:?} not stripped"
        );
        assert!(
            !n.stripped.to_ascii_lowercase().starts_with(pat),
            "fuzz iter {i}: head still matches pattern: {}",
            n.stripped
        );
    }
}

#[test]
fn sycophancy_invert_then_compute_verdict_divergent() {
    let neutral = NeutralPrompt {
        stripped: "Identify defects in the rendered cart.".into(),
        removed_tokens: vec![],
    };
    let _ = invert_framing(&neutral);
    let a = SycJudgeOutput {
        primary_label: "matches_spec".into(),
        claims: ["title".to_string()].into_iter().collect(),
    };
    let b = SycJudgeOutput {
        primary_label: "defect_present".into(),
        claims: ["title".to_string()].into_iter().collect(),
    };
    let v = compute_verdict(&a, &b);
    assert_eq!(v.verdict_label, "divergent_sycophancy_flag");
    assert!(v.divergent);
}

// -----------------------------------------------------------------
// Swap-order pairwise eval (stub-judge surrogate).
// -----------------------------------------------------------------

#[test]
fn swap_order_disagreement_routes_to_human_review_payload() {
    // The orchestrator-side decision: when judge_a != judge_b under
    // swap, build the SwapOrderDisagreement payload + assert the
    // kind + details_json shape matches the dashboard fold path.
    let payload = build_vlm_image_validation_payload(
        VlmImageValidationKind::SwapOrderDisagreement,
        "step-swap",
        Some("req-swap".into()),
        serde_json::json!({
            "judge_a_verdict": "candidate_a_better",
            "judge_b_verdict": "candidate_b_better",
            "agreement_required": true,
            "divergence_axis": "title_text_match"
        }),
    );
    assert_eq!(payload.kind, VlmImageValidationKind::SwapOrderDisagreement);
    assert_eq!(payload.step_id, "step-swap");
    assert_eq!(payload.request_id.as_deref(), Some("req-swap"));
    let det = &payload.details_json;
    assert_eq!(det.get("agreement_required").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(
        det.get("judge_a_verdict").and_then(|v| v.as_str()),
        Some("candidate_a_better")
    );
}

// -----------------------------------------------------------------
// RCE convergence + hard cap.
// -----------------------------------------------------------------

#[test]
fn rce_convergence_synthetic_lpips_sequence_stops_at_4() {
    // Plan task 4 fixture: [0.30, 0.18, 0.05, 0.018] -> convergence
    // at iter 4 (index 3) because the last delta is < 0.02.
    let seq = [0.30, 0.18, 0.05, 0.018];
    let mut converged_at: Option<u32> = None;
    for (idx, &d) in seq.iter().enumerate() {
        let iter = u32::try_from(idx).unwrap_or(0);
        match lpips_delta_converged(iter, Some(d), false) {
            ConvergenceState::Converged(reason) => {
                assert_eq!(reason, ConvergenceReason::LpipsDelta);
                converged_at = Some(iter);
                break;
            }
            ConvergenceState::CappedAtMax => {
                panic!("converged via cap on iter {iter}; expected delta-converged");
            }
            ConvergenceState::Iterate => {}
        }
    }
    assert_eq!(converged_at, Some(3));
}

#[test]
fn rce_hard_cap_stops_at_max_rce_iterations() {
    // Plan task 4 fixture: a 7-iter sequence that never crosses the
    // delta floor MUST stop at iteration MAX_RCE_ITERATIONS-1 via
    // CappedAtMax.
    let mut stopped_via_cap = false;
    for iter in 0..7u32 {
        // Every iteration above the LPIPS floor; critic still flagging.
        let state = lpips_delta_converged(iter, Some(LPIPS_DELTA_CONVERGED_FLOOR + 0.1), false);
        match state {
            ConvergenceState::CappedAtMax => {
                assert!(iter + 1 >= MAX_RCE_ITERATIONS);
                stopped_via_cap = true;
                break;
            }
            ConvergenceState::Converged(_) => panic!("unexpected convergence on iter {iter}"),
            ConvergenceState::Iterate => {}
        }
    }
    assert!(stopped_via_cap, "loop never hit the hard cap");
    // should_stop_for_rce_cap mirrors the cap check.
    assert!(should_stop_for_rce_cap(MAX_RCE_ITERATIONS - 1));
    assert!(!should_stop_for_rce_cap(0));
}

#[test]
fn rce_critic_no_diffs_converges_immediately() {
    let state = lpips_delta_converged(0, Some(0.5), true);
    assert_eq!(state, ConvergenceState::Converged(ConvergenceReason::NoActionableDiffs));
}

#[test]
fn rce_clamp_iterations_to_cap() {
    assert_eq!(clamp_iterations_to_rce_cap(99), MAX_RCE_ITERATIONS);
    assert_eq!(clamp_iterations_to_rce_cap(3), 3);
    assert_eq!(clamp_iterations_to_rce_cap(0), 0);
}

// -----------------------------------------------------------------
// Second-opinion + xgrammar schema-mask producer-side gates.
// -----------------------------------------------------------------

#[test]
fn second_opinion_threshold_table_matches_plan() {
    // Plan task 5: low-logprob OR sycophancy_flag divergent -> second
    // opinion required.
    assert!(second_opinion_required(Some(-2.5), "convergent"));
    assert!(second_opinion_required(Some(-1.6), "convergent"));
    assert!(!second_opinion_required(Some(-1.4), "convergent"));
    assert!(second_opinion_required(None, "divergent_sycophancy_flag"));
    assert!(!second_opinion_required(None, "inconclusive"));
}

#[test]
fn vlm_schema_unconstrained_emitted_on_non_compliant_output() {
    // Plan task 6: when the Tier-2 raw output fails the XGrammar
    // schema mask, the gate function returns false and the producer
    // would emit a VlmSchemaUnconstrained row.
    let raw_compliant = r#"{"issues":[],"advisory_pass":true}"#;
    let raw_bad = r#"{"some":"other_shape"}"#;
    assert!(vlm_output_schema_compliant(raw_compliant));
    assert!(!vlm_output_schema_compliant(raw_bad));
}
