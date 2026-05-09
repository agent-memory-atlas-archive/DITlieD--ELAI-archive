//! B-10 ContextBudget Profile Activation + 3E.5 file_overlap_score
//!
//! Unit tests for:
//!   - file_overlap_score (pure scalar)
//!   - compute_overlap (ranking + threshold filtering)
//!   - ModelCapabilityProfile constructors match token-efficiency-design.txt §2 table
//!   - detect_model_capability_profile substring matching with tier hints

use elai_orchestrator::context_budget::{
    compute_overlap, detect_model_capability_profile, file_overlap_score, ContextBudget,
    ModelCapabilityProfile, FILE_OVERLAP_THRESHOLD,
};
use elai_core::VramTier;

// ---------------------------------------------------------------------------
// 3E.5 file_overlap_score scalar tests
// ---------------------------------------------------------------------------

#[test]
fn file_overlap_score_empty_agent_files_returns_zero() {
    let agent: Vec<String> = vec![];
    let entry = vec!["a.rs".to_string()];
    assert_eq!(file_overlap_score(&agent, &entry), 0.0);
}

#[test]
fn file_overlap_score_empty_entry_files_returns_zero() {
    let agent = vec!["a.rs".to_string()];
    let entry: Vec<String> = vec![];
    assert_eq!(file_overlap_score(&agent, &entry), 0.0);
}

#[test]
fn file_overlap_score_exact_match_returns_one() {
    let agent = vec!["a.rs".to_string(), "b.rs".to_string()];
    let entry = vec!["a.rs".to_string(), "b.rs".to_string()];
    assert!((file_overlap_score(&agent, &entry) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn file_overlap_score_partial_overlap_one_of_two() {
    let agent = vec!["a.rs".to_string(), "b.rs".to_string()];
    let entry = vec!["a.rs".to_string(), "c.rs".to_string()];
    // Intersection = {a.rs} (1), agent.len() = 2 → 0.5.
    let score = file_overlap_score(&agent, &entry);
    assert!((score - 0.5).abs() < f32::EPSILON, "got {score}, expected 0.5");
}

#[test]
fn file_overlap_score_no_overlap_returns_zero() {
    let agent = vec!["a.rs".to_string()];
    let entry = vec!["c.rs".to_string()];
    assert_eq!(file_overlap_score(&agent, &entry), 0.0);
}

// ---------------------------------------------------------------------------
// 3E.5 compute_overlap (ranking + threshold filtering)
// ---------------------------------------------------------------------------

#[test]
fn compute_overlap_ranks_by_score_descending_and_filters_below_threshold() {
    let agent = vec!["a.rs".to_string(), "b.rs".to_string()];
    // entry_id, files
    let entries: Vec<(u64, Vec<String>)> = vec![
        (1, vec!["a.rs".to_string()]),                          // score 0.5
        (2, vec!["a.rs".to_string(), "b.rs".to_string()]),       // score 1.0
        (3, vec!["c.rs".to_string()]),                          // score 0.0 — filtered
    ];

    let ranked = compute_overlap(&agent, &entries);

    // entry 2 (1.0) > entry 1 (0.5); entry 3 dropped (< threshold 0.10).
    assert_eq!(ranked.len(), 2, "expected 2 entries after threshold filter, got {}", ranked.len());
    assert_eq!(ranked[0].0, 2, "highest-overlap entry must come first");
    assert_eq!(ranked[1].0, 1);
    // Verify scores monotonic descending.
    assert!(ranked[0].1 >= ranked[1].1);
}

#[test]
fn compute_overlap_empty_agent_files_returns_empty_filtered_list() {
    // No agent_files = nothing scores positively = all filtered out.
    let agent: Vec<String> = vec![];
    let entries: Vec<(u64, Vec<String>)> = vec![
        (1, vec!["a.rs".to_string()]),
        (2, vec!["b.rs".to_string()]),
    ];
    let ranked = compute_overlap(&agent, &entries);
    assert!(ranked.is_empty(), "no agent_files = nothing kept (every score 0.0 < threshold)");
}

#[test]
fn compute_overlap_threshold_constant_is_ten_percent() {
    // DECISION-G: documented constant.
    assert!((FILE_OVERLAP_THRESHOLD - 0.10).abs() < f32::EPSILON);
}

// ---------------------------------------------------------------------------
// DECISION-D: per-tier slot weights match spec §2 table exactly
// ---------------------------------------------------------------------------

#[test]
fn micro_7b_12gb_constructor_matches_spec_table() {
    let p = ModelCapabilityProfile::micro_7b_12gb();
    // Per token-efficiency-design.txt §2 table (12GB-7B, 20K total).
    assert_eq!(p.fixed_slot_targets, [330, 400, 800], "micro_7b_12gb fixed_slot_targets");
    assert_eq!(p.output_reserve, 512, "micro_7b_12gb output_reserve");
    assert_eq!(p.recommended_prompt_budget, 20000);
    assert!(p.slot_weights.validate_dynamic_sum());
}

#[test]
fn micro_3b_12gb_constructor_matches_spec_table() {
    let p = ModelCapabilityProfile::micro_3b_12gb();
    // Per spec table (12GB-3B, 35K).
    assert_eq!(p.fixed_slot_targets, [330, 500, 1500], "micro_3b_12gb fixed_slot_targets");
    assert_eq!(p.output_reserve, 512, "micro_3b_12gb output_reserve");
    assert_eq!(p.recommended_prompt_budget, 35000);
    assert!(p.slot_weights.validate_dynamic_sum());
}

#[test]
fn large_32b_48gb_constructor_matches_spec_table() {
    let p = ModelCapabilityProfile::large_32b_48gb();
    // Per spec table (48GB-32B, 30K).
    assert_eq!(p.fixed_slot_targets, [330, 500, 1500], "large_32b_48gb fixed_slot_targets");
    assert_eq!(p.output_reserve, 512, "large_32b_48gb output_reserve");
    assert_eq!(p.recommended_prompt_budget, 30000);
    assert!(p.slot_weights.validate_dynamic_sum());
}

#[test]
fn micro_8gb_constructor_matches_spec_table() {
    let p = ModelCapabilityProfile::micro_8gb();
    // Per spec table (8GB, 20K).
    assert_eq!(p.fixed_slot_targets, [200, 300, 600], "micro_8gb fixed_slot_targets");
    assert_eq!(p.output_reserve, 512, "micro_8gb output_reserve");
    assert_eq!(p.recommended_prompt_budget, 20000);
    assert!(p.slot_weights.validate_dynamic_sum());
}

// ---------------------------------------------------------------------------
// DECISION-F: detect_model_capability_profile substring matching
// ---------------------------------------------------------------------------

#[test]
fn detect_qwen_7b_abliterated_maps_to_micro_7b_12gb() {
    // The actual loaded model name uses the abliterated suffix.
    let p = detect_model_capability_profile(
        "Qwen2.5-Coder-7B-Instruct-abliterated",
        VramTier::Vram12Gb,
    );
    // Expect the 7B-12GB tier.
    assert_eq!(p.fixed_slot_targets, [330, 400, 800]);
    assert_eq!(p.recommended_prompt_budget, 20000);
}

#[test]
fn detect_qwen_3b_12gb_maps_to_micro_3b_12gb() {
    let p = detect_model_capability_profile(
        "Qwen2.5-Coder-3B-Instruct-abliterated",
        VramTier::Vram12Gb,
    );
    assert_eq!(p.fixed_slot_targets, [330, 500, 1500]);
    assert_eq!(p.recommended_prompt_budget, 35000);
}

#[test]
fn detect_qwen_3b_8gb_maps_to_micro_8gb() {
    let p = detect_model_capability_profile("Qwen2.5-Coder-3B-Instruct", VramTier::Vram8Gb);
    assert_eq!(p.fixed_slot_targets, [200, 300, 600]);
}

#[test]
fn detect_qwen_32b_maps_to_large_32b_48gb() {
    let p = detect_model_capability_profile("Qwen2.5-Coder-32B-Instruct", VramTier::Vram48Gb);
    assert_eq!(p.fixed_slot_targets, [330, 500, 1500]);
    assert_eq!(p.recommended_prompt_budget, 30000);
}

#[test]
fn detect_unknown_model_falls_back_to_micro_7b_12gb() {
    // Unknown model name + 12GB → safe default (warns, not panics).
    let p = detect_model_capability_profile("totally-unknown-model", VramTier::Vram12Gb);
    // Default fallback per DECISION-F is micro_7b_12gb.
    assert_eq!(p.fixed_slot_targets, [330, 400, 800]);
}

// ---------------------------------------------------------------------------
// compute_with_profile after constructor changes still preserves zero-sum
// ---------------------------------------------------------------------------

#[test]
fn compute_with_profile_micro_7b_zero_sum_invariant() {
    let p = ModelCapabilityProfile::micro_7b_12gb();
    let budget =
        ContextBudget::compute_with_profile(25000, p.fixed_slot_targets, &p).expect("valid");
    // Total allocated must equal effective ceiling (25000).
    assert_eq!(budget.total_allocated, 25000);
    // Spec fixed slots match.
    assert_eq!(budget.slots[0].allocated_tokens, 330);
    assert_eq!(budget.slots[1].allocated_tokens, 400);
    assert_eq!(budget.slots[2].allocated_tokens, 800);
    // Output reserve is clamped up to FLOOR_OUTPUT_RESERVE=2000 (it's higher than 512).
    assert!(budget.slots[7].allocated_tokens >= 512);
}
