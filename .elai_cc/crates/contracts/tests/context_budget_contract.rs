use std::sync::Arc;

use elai_core::clock::SeededClock;
use elai_orchestrator::context_budget::{
    abbreviate_path, apply_token_reclamation, restore_output_formatting,
    AdaptiveError, AdaptiveLevel, BudgetError, CacheBreakpoint, ContextBudget,
    ContextSlot, ExecutionMode, ModelTier, SlotAllocation,
    CACHE_BREAKPOINT_COUNT, FLOOR_GRC_HISTORY, FLOOR_OUTPUT_RESERVE, FLOOR_SSB_ENTRIES,
    LARGE_SLOT_CODE_CONTEXT, LARGE_SLOT_GRC_HISTORY, LARGE_SLOT_OUTPUT_RESERVE,
    LARGE_SLOT_REPO_MAP, LARGE_SLOT_RULES, LARGE_SLOT_SSB_ENTRIES,
    LARGE_SLOT_SYSTEM_PROMPT, LARGE_SLOT_TELESCOPE,
    MICRO_SLOT_CODE_CONTEXT, MICRO_SLOT_GRC_HISTORY, MICRO_SLOT_OUTPUT_RESERVE,
    MICRO_SLOT_RULES, MICRO_SLOT_SSB_ENTRIES,
    MICRO_SLOT_SYSTEM_PROMPT, MICRO_SLOT_TELESCOPE,
    SLOT_COUNT, SUM_DYNAMIC_FLOORS,
};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use tempfile::TempDir;

// H-31 (SCHEMA_VERSION 240): bumped 8 -> 9 for ContextSlot::Memory (the user-preference
// bi-temporal ledger retrieval-into-prompt slot, appended after OutputReserve).
// H-53 (SCHEMA_VERSION 229): bumped 9 -> 10 for ContextSlot::Evidence (the ranked-evidence
// pre-binding slot for freeze_anchors; must sit after the last cache breakpoint per HD-2).
// I-6: bumped 10 -> 11 for append-only ContextSlot::AnchorPane.
const _: () = assert!(SLOT_COUNT == 11);
const _: () = assert!(CACHE_BREAKPOINT_COUNT == 2);

// Compile-time size assertions to detect accidental field additions.
const _: () = assert!(std::mem::size_of::<ContextSlot>() == 1); // repr(u8)
const _: () = assert!(std::mem::size_of::<AdaptiveLevel>() == 1);

#[test]
fn context_slot_is_hashable() {
    let mut set = std::collections::HashSet::new();
    set.insert(ContextSlot::SystemPrompt);
    set.insert(ContextSlot::OutputReserve);
    assert_eq!(set.len(), 2);
}

#[test]
fn slot_allocation_fields_accessible() {
    let sa = SlotAllocation {
        slot: ContextSlot::Rules,
        allocated_tokens: 100,
        floor_tokens: 10,
        actual_tokens: 50,
        source_truncated: false,
    };
    assert_eq!(sa.slot, ContextSlot::Rules);
    assert_eq!(sa.allocated_tokens, 100);
}

#[test]
fn cache_breakpoint_fields_accessible() {
    let bp = CacheBreakpoint {
        after_slot: ContextSlot::PhaseSpec,
        prefix_hash: 0,
        valid: false,
    };
    assert_eq!(bp.after_slot, ContextSlot::PhaseSpec);
    assert!(!bp.valid);
}

#[test]
fn new_budget_zero_allocated() {
    let budget = ContextBudget::new(131072);
    assert_eq!(budget.total_allocated, 0);
    assert_eq!(budget.total_used, 0);
    assert_eq!(budget.adaptive_level, AdaptiveLevel::Normal);
    assert_eq!(budget.slots.len(), SLOT_COUNT);
    assert_eq!(budget.cache_breakpoints.len(), CACHE_BREAKPOINT_COUNT);
}

#[test]
fn display_impl_exists() {
    let budget = ContextBudget::new(131072);
    let output = format!("{budget}");
    assert!(output.contains("max=131072"));
    assert!(output.contains("BP0"));
}

#[test]
fn floor_enforcement_steal_from_code_context() {
    // Contract: compute() enforces GrcHistory floor by stealing from CodeContext.
    // compute(4000, [100,200,300], 2000) -> dynamic=1400, grc starts at 280, needs 500.
    let budget = ContextBudget::compute(4000, [100, 200, 300], 2000).unwrap();
    assert!(budget.slots[6].allocated_tokens >= FLOOR_GRC_HISTORY);
    assert!(budget.slots[5].allocated_tokens >= FLOOR_SSB_ENTRIES);
    assert_eq!(budget.slots[7].allocated_tokens, FLOOR_OUTPUT_RESERVE);
    assert_eq!(budget.total_allocated, 4000);
    // Zero-sum over dynamic slots.
    let dynamic_sum: usize = (3..=6).map(|i| budget.slots[i].allocated_tokens).sum();
    assert_eq!(dynamic_sum, 1400);
    // Floor tokens are set correctly.
    assert_eq!(budget.slots[5].floor_tokens, FLOOR_SSB_ENTRIES);
    assert_eq!(budget.slots[6].floor_tokens, FLOOR_GRC_HISTORY);
    assert_eq!(budget.slots[7].floor_tokens, FLOOR_OUTPUT_RESERVE);
}

#[test]
fn budget_infeasible_when_dynamic_too_small() {
    // Contract: compute() returns BudgetInfeasible when dynamic_budget < SUM_DYNAMIC_FLOORS.
    let result = ContextBudget::compute(3000, [100, 200, 300], 2000);
    assert!(result.is_err());
    match result {
        Err(BudgetError::BudgetInfeasible {
            dynamic_budget,
            sum_floors,
        }) => {
            assert_eq!(dynamic_budget, 400);
            assert_eq!(sum_floors, SUM_DYNAMIC_FLOORS);
        }
        Ok(_) => panic!("expected BudgetInfeasible"),
    }
}

#[test]
fn update_breakpoints_validity_detection() {
    // Contract: BP0 transitions from invalid (first call) to valid (same content) to invalid (changed slot).
    // Slots 0-10: sys, rules, spec, repo, code, ssb, grc, out, mem, evidence, anchor pane.
    let mut budget = ContextBudget::new(131072);
    let contents_a: Vec<&[u8]> = vec![
        b"sys", b"rules", b"spec", b"repo", b"code", b"ssb", b"grc", b"out", b"mem", b"evidence",
        b"anchor",
    ];
    budget.update_breakpoints(&contents_a);
    assert!(!budget.cache_breakpoints[0].valid, "first call: BP0 must be invalid");
    assert!(!budget.cache_breakpoints[1].valid, "first call: BP1 must be invalid");
    assert_ne!(budget.cache_breakpoints[0].prefix_hash, 0);

    // Same content -> valid.
    budget.update_breakpoints(&contents_a);
    assert!(budget.cache_breakpoints[0].valid, "same content: BP0 must be valid");
    assert!(budget.cache_breakpoints[1].valid, "same content: BP1 must be valid");

    // Change slot 2 (PhaseSpec) -> BP0 and BP1 both invalid (slot 2 is in both prefixes).
    let contents_b: Vec<&[u8]> = vec![
        b"sys", b"rules", b"spec_v2", b"repo", b"code", b"ssb", b"grc", b"out", b"mem", b"evidence",
        b"anchor",
    ];
    budget.update_breakpoints(&contents_b);
    assert!(!budget.cache_breakpoints[0].valid, "changed slot 2: BP0 must be invalid");
    assert!(!budget.cache_breakpoints[1].valid, "changed slot 2: BP1 must be invalid");
}

#[test]
fn update_breakpoints_bp1_ignores_post_bp1_slots() {
    // Contract: changing slots 5-10 does not invalidate BP1 (which covers 0..=4).
    // I-6: Memory=8, Evidence=9, and AnchorPane=10 are all post-BP1 slots.
    let mut budget = ContextBudget::new(131072);
    let contents_a: Vec<&[u8]> = vec![
        b"sys", b"rules", b"spec", b"repo", b"code", b"ssb", b"grc", b"out", b"mem", b"evidence",
        b"anchor",
    ];
    budget.update_breakpoints(&contents_a);
    // Second call with same 0..=4 but different 5-9.
    let contents_b: Vec<&[u8]> = vec![
        b"sys", b"rules", b"spec", b"repo", b"code",
        b"ssb_changed", b"grc_changed", b"out_changed", b"mem_changed", b"evidence_changed",
        b"anchor_changed",
    ];
    budget.update_breakpoints(&contents_b);
    assert!(budget.cache_breakpoints[0].valid, "BP0 must be valid (slots 0-2 unchanged)");
    assert!(budget.cache_breakpoints[1].valid, "BP1 must be valid (slots 0-4 unchanged)");
}

#[test]
fn large_budget_no_stealing_needed() {
    // Contract: large model contexts produce valid budgets with no floor stealing.
    let budget = ContextBudget::compute(100_000, [100, 200, 300], 2000).unwrap();
    assert!(budget.slots[5].allocated_tokens >= FLOOR_SSB_ENTRIES);
    assert!(budget.slots[6].allocated_tokens >= FLOOR_GRC_HISTORY);
    assert!(budget.slots[7].allocated_tokens >= FLOOR_OUTPUT_RESERVE);
    assert_eq!(budget.total_allocated, 100_000);
    // All slot allocations are non-zero for dynamic slots.
    for i in 3..=6 {
        assert!(budget.slots[i].allocated_tokens > 0, "slot {} should be non-zero", i);
    }
}

// ---------------------------------------------------------------------------
// Adaptive reduction contract tests
// ---------------------------------------------------------------------------

fn make_ssb(dir: &TempDir) -> Ssb {
    let config = SsbConfig {
        db_path: dir.path().join("test_ssb.db"),
        pool_size: Some(2),
        project_id: "test-project".into(),
        session_id: "test-session".into(),
    };
    Ssb::new(config, Arc::new(SeededClock::new())).expect("ssb init")
}

/// Create a budget where total_used exceeds model_max_context by a controllable amount.
/// Sets actual_tokens on dynamic slots to create overflow.
fn make_overflowing_budget(overflow: usize) -> ContextBudget {
    let mut budget = ContextBudget::compute(10000, [500, 500, 500], 2000).expect("compute");
    // Dynamic budget = 10000 - 1500 - 2000 = 6500.
    // Set actual_tokens = allocated_tokens for all slots.
    for i in 0..SLOT_COUNT {
        budget.slots[i].actual_tokens = budget.slots[i].allocated_tokens;
    }
    // Add overflow to CodeContext (slot 4) to simulate content exceeding budget.
    budget.slots[ContextSlot::CodeContext as usize].actual_tokens += overflow;
    budget.total_used = budget.slots.iter().map(|s| s.actual_tokens).sum();
    budget
}

/// Create a budget with overflow that cannot be resolved by any reduction level.
/// Overflow is placed on fixed slots (SystemPrompt) which adaptive_reduce never touches.
fn make_irrecoverable_budget() -> ContextBudget {
    let mut budget = ContextBudget::compute(10000, [500, 500, 500], 2000).expect("compute");
    for i in 0..SLOT_COUNT {
        budget.slots[i].actual_tokens = budget.slots[i].allocated_tokens;
    }
    // Fixed slot overflow: actual >> allocated, and reduction can't touch fixed slots.
    budget.slots[ContextSlot::SystemPrompt as usize].actual_tokens = 8000;
    budget.total_used = budget.slots.iter().map(|s| s.actual_tokens).sum();
    budget
}

#[tokio::test]
async fn adaptive_reduce_level1_fits_returns_ok() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir);
    // Small overflow that level 1 (reducing GrcHistory + SsbEntries) can handle.
    let mut budget = make_overflowing_budget(200);
    let result = budget.adaptive_reduce(0.5, ExecutionMode::Interactive, &ssb).await;
    assert!(result.is_ok(), "level 1 should bring budget within limits");
    assert_eq!(budget.adaptive_level, AdaptiveLevel::FilteredByRelevance);
}

#[tokio::test]
async fn adaptive_reduce_level2_fits_returns_ok() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir);
    // Larger overflow: level 1 insufficient, level 2 (reducing CodeContext + RepoMap) needed.
    let mut budget = make_overflowing_budget(2000);
    let result = budget.adaptive_reduce(0.3, ExecutionMode::Interactive, &ssb).await;
    assert!(result.is_ok(), "level 2 should bring budget within limits");
    assert!(
        budget.adaptive_level == AdaptiveLevel::FilteredByRelevance
            || budget.adaptive_level == AdaptiveLevel::ReducedGranularity,
        "level should be 1 or 2"
    );
}

#[tokio::test]
async fn adaptive_reduce_level3_fits_returns_ok() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir);
    // Large overflow: levels 1+2 insufficient, level 3 (floor clamp) needed.
    let mut budget = make_overflowing_budget(3000);
    let result = budget.adaptive_reduce(0.3, ExecutionMode::Interactive, &ssb).await;
    assert!(result.is_ok(), "level 3 should bring budget within limits");
}

#[tokio::test]
async fn adaptive_reduce_all_levels_fail_interactive_escalates() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir);
    // Irrecoverable: fixed slot overflow that reduction cannot touch.
    let mut budget = make_irrecoverable_budget();
    let result = budget.adaptive_reduce(0.5, ExecutionMode::Interactive, &ssb).await;
    assert!(result.is_err());
    match result {
        Err(AdaptiveError::Escalate { breakdown }) => {
            assert_eq!(breakdown.model_max_context, 10000);
            assert!(breakdown.total_used > breakdown.model_max_context);
        }
        other => panic!("expected Escalate, got {other:?}"),
    }
    assert_eq!(budget.adaptive_level, AdaptiveLevel::Terminal);
}

#[tokio::test]
async fn adaptive_reduce_all_levels_fail_autonomous_split_task() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir);
    let mut budget = make_irrecoverable_budget();
    let result = budget.adaptive_reduce(0.5, ExecutionMode::Autonomous, &ssb).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(AdaptiveError::SplitTask)));
    assert_eq!(budget.adaptive_level, AdaptiveLevel::Terminal);
}

#[tokio::test]
async fn adaptive_reduce_escalate_carries_breakdown() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir);
    let mut budget = make_irrecoverable_budget();
    let result = budget.adaptive_reduce(0.5, ExecutionMode::Interactive, &ssb).await;
    match result {
        Err(AdaptiveError::Escalate { breakdown }) => {
            assert_eq!(breakdown.slots.len(), SLOT_COUNT);
            assert_eq!(breakdown.model_max_context, 10000);
            // Breakdown should reflect the final state after all reduction attempts.
            let sum: usize = breakdown.slots.iter().map(|s| s.allocated_tokens).sum();
            assert!(sum > 0, "allocated_tokens in breakdown must be non-zero");
        }
        other => panic!("expected Escalate, got {other:?}"),
    }
}

#[tokio::test]
async fn adaptive_reduce_preserves_fixed_slot_allocations() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir);
    let mut budget = make_overflowing_budget(3000);
    let fixed_before = [
        budget.slots[0].allocated_tokens,
        budget.slots[1].allocated_tokens,
        budget.slots[2].allocated_tokens,
        budget.slots[7].allocated_tokens,
    ];
    let _ = budget.adaptive_reduce(0.5, ExecutionMode::Interactive, &ssb).await;
    // Fixed slots (SystemPrompt, Rules, PhaseSpec, OutputReserve) must not be modified.
    assert_eq!(budget.slots[0].allocated_tokens, fixed_before[0]);
    assert_eq!(budget.slots[1].allocated_tokens, fixed_before[1]);
    assert_eq!(budget.slots[2].allocated_tokens, fixed_before[2]);
    assert_eq!(budget.slots[7].allocated_tokens, fixed_before[3]);
}

#[tokio::test]
async fn adaptive_reduce_floors_respected() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&dir);
    let mut budget = make_irrecoverable_budget();
    let _ = budget.adaptive_reduce(0.9, ExecutionMode::Interactive, &ssb).await;
    // After maximum reduction, all floored slots must be at or above their floor.
    assert!(
        budget.slots[ContextSlot::SsbEntries as usize].allocated_tokens
            >= budget.slots[ContextSlot::SsbEntries as usize].floor_tokens,
        "SsbEntries must respect floor"
    );
    assert!(
        budget.slots[ContextSlot::GrcHistory as usize].allocated_tokens
            >= budget.slots[ContextSlot::GrcHistory as usize].floor_tokens,
        "GrcHistory must respect floor"
    );
}

// ---------------------------------------------------------------------------
// A-40: SlotAssemblyConfig contract tests (MICRO per-slot optimizations)
// ---------------------------------------------------------------------------

#[test]
fn micro_slot_system_prompt_target_200() {
    let cfg = ContextBudget::slot_assembly_config(ContextSlot::SystemPrompt, ModelTier::Micro);
    assert_eq!(cfg.max_tokens, MICRO_SLOT_SYSTEM_PROMPT);
    assert!(!cfg.skip_content);
    assert_eq!(cfg.entry_limit, None);
    assert!(!cfg.conclusion_only);
}

#[test]
fn micro_slot_rules_target_150() {
    let cfg = ContextBudget::slot_assembly_config(ContextSlot::Rules, ModelTier::Micro);
    assert_eq!(cfg.max_tokens, MICRO_SLOT_RULES);
    assert!(!cfg.skip_content);
    assert_eq!(cfg.entry_limit, None);
    assert!(!cfg.conclusion_only);
}

#[test]
fn micro_slot_telescope_target_400() {
    // ContextSlot::PhaseSpec maps to spec "Telescope" slot (index 2).
    let cfg = ContextBudget::slot_assembly_config(ContextSlot::PhaseSpec, ModelTier::Micro);
    assert_eq!(cfg.max_tokens, MICRO_SLOT_TELESCOPE);
    assert!(!cfg.skip_content);
    assert_eq!(cfg.entry_limit, None);
    assert!(!cfg.conclusion_only);
}

#[test]
fn micro_slot_repo_map_skipped() {
    let cfg = ContextBudget::slot_assembly_config(ContextSlot::RepoMap, ModelTier::Micro);
    assert!(cfg.skip_content, "MICRO tier must skip RepoMap content entirely");
    assert_eq!(cfg.entry_limit, None);
    assert!(!cfg.conclusion_only);
}

#[test]
fn micro_slot_code_context_target_1200() {
    let cfg = ContextBudget::slot_assembly_config(ContextSlot::CodeContext, ModelTier::Micro);
    assert_eq!(cfg.max_tokens, MICRO_SLOT_CODE_CONTEXT);
    assert!(!cfg.skip_content);
    assert_eq!(cfg.entry_limit, None);
    assert!(!cfg.conclusion_only);
}

#[test]
fn micro_slot_ssb_entry_limit_2() {
    let cfg = ContextBudget::slot_assembly_config(ContextSlot::SsbEntries, ModelTier::Micro);
    assert_eq!(cfg.max_tokens, MICRO_SLOT_SSB_ENTRIES);
    assert!(!cfg.skip_content);
    assert_eq!(cfg.entry_limit, Some(2), "MICRO SSB must limit to last 2 entries");
    assert!(!cfg.conclusion_only);
}

#[test]
fn micro_slot_grc_conclusion_only() {
    let cfg = ContextBudget::slot_assembly_config(ContextSlot::GrcHistory, ModelTier::Micro);
    assert_eq!(cfg.max_tokens, MICRO_SLOT_GRC_HISTORY);
    assert!(!cfg.skip_content);
    assert_eq!(cfg.entry_limit, None);
    assert!(cfg.conclusion_only, "MICRO GRC must include only last CONVERGE conclusion");
}

#[test]
fn micro_slot_output_reserve_750() {
    let cfg = ContextBudget::slot_assembly_config(ContextSlot::OutputReserve, ModelTier::Micro);
    assert_eq!(cfg.max_tokens, MICRO_SLOT_OUTPUT_RESERVE);
    assert!(!cfg.skip_content);
    assert_eq!(cfg.entry_limit, None);
    assert!(!cfg.conclusion_only);
}

#[test]
fn large_slot_targets_match_spec() {
    let sys = ContextBudget::slot_assembly_config(ContextSlot::SystemPrompt, ModelTier::Large);
    assert_eq!(sys.max_tokens, LARGE_SLOT_SYSTEM_PROMPT);
    assert!(!sys.skip_content);

    let rules = ContextBudget::slot_assembly_config(ContextSlot::Rules, ModelTier::Large);
    assert_eq!(rules.max_tokens, LARGE_SLOT_RULES);

    let code = ContextBudget::slot_assembly_config(ContextSlot::CodeContext, ModelTier::Large);
    assert_eq!(code.max_tokens, LARGE_SLOT_CODE_CONTEXT);

    let telescope = ContextBudget::slot_assembly_config(ContextSlot::PhaseSpec, ModelTier::Large);
    assert_eq!(telescope.max_tokens, LARGE_SLOT_TELESCOPE);

    let repo = ContextBudget::slot_assembly_config(ContextSlot::RepoMap, ModelTier::Large);
    assert_eq!(repo.max_tokens, LARGE_SLOT_REPO_MAP);
    assert!(!repo.skip_content, "LARGE tier must not skip RepoMap");

    let ssb = ContextBudget::slot_assembly_config(ContextSlot::SsbEntries, ModelTier::Large);
    assert_eq!(ssb.max_tokens, LARGE_SLOT_SSB_ENTRIES);

    let grc = ContextBudget::slot_assembly_config(ContextSlot::GrcHistory, ModelTier::Large);
    assert_eq!(grc.max_tokens, LARGE_SLOT_GRC_HISTORY);
    assert!(!grc.conclusion_only, "LARGE tier must include full GRC chain");

    let out = ContextBudget::slot_assembly_config(ContextSlot::OutputReserve, ModelTier::Large);
    assert_eq!(out.max_tokens, LARGE_SLOT_OUTPUT_RESERVE);
}

#[test]
fn slot_assembly_config_micro_vs_large_differ_on_all_slots() {
    let slots = [
        ContextSlot::SystemPrompt,
        ContextSlot::Rules,
        ContextSlot::PhaseSpec,
        ContextSlot::RepoMap,
        ContextSlot::CodeContext,
        ContextSlot::SsbEntries,
        ContextSlot::GrcHistory,
        ContextSlot::OutputReserve,
    ];
    for slot in slots {
        let micro = ContextBudget::slot_assembly_config(slot, ModelTier::Micro);
        let large = ContextBudget::slot_assembly_config(slot, ModelTier::Large);
        assert_ne!(
            micro.max_tokens, large.max_tokens,
            "MICRO and LARGE must differ for {:?}",
            slot
        );
    }
}

// ---------------------------------------------------------------------------
// Token reclamation contract tests (A-42)
// ---------------------------------------------------------------------------

#[test]
fn contract_apply_token_reclamation_code_context_never_increases_length() {
    let input = "use std::io;\nuse std::fs;\nuse std::path;\nuse std::env;\n\n\n\nfn main() {   \n    let x = 1; // comment  \n}\n";
    let out = apply_token_reclamation(input, ContextSlot::CodeContext, None);
    assert!(out.len() <= input.len(),
        "reclamation must not increase length: out={} in={}", out.len(), input.len());
}

#[test]
fn contract_apply_token_reclamation_empty_input_is_safe() {
    let out = apply_token_reclamation("", ContextSlot::CodeContext, None);
    assert_eq!(out, "");
    let out2 = apply_token_reclamation("", ContextSlot::SsbEntries, None);
    assert_eq!(out2, "");
    let out3 = apply_token_reclamation("", ContextSlot::SystemPrompt, None);
    assert_eq!(out3, "");
}

#[test]
fn contract_apply_token_reclamation_non_target_slot_unchanged() {
    let input = "unchanged content";
    for slot in [ContextSlot::SystemPrompt, ContextSlot::Rules, ContextSlot::PhaseSpec,
                 ContextSlot::RepoMap, ContextSlot::GrcHistory, ContextSlot::OutputReserve] {
        let out = apply_token_reclamation(input, slot, None);
        assert_eq!(out, input, "slot {:?} should return unchanged content", slot);
    }
}

#[test]
fn contract_abbreviate_path_strips_known_prefixes() {
    assert_eq!(abbreviate_path("src/foo/bar.rs"), "foo/bar.rs");
    assert_eq!(abbreviate_path("crates/core/src/lib.rs"), "core/src/lib.rs");
    assert_eq!(abbreviate_path("lib/utils/helper.rs"), "utils/helper.rs");
}

#[test]
fn contract_abbreviate_path_preserves_unknown_prefix() {
    assert_eq!(abbreviate_path("external/vendor/lib.rs"), "external/vendor/lib.rs");
}

#[test]
fn contract_abbreviate_path_short_path_unchanged() {
    assert_eq!(abbreviate_path("src/a.rs"), "src/a.rs");
}

#[test]
fn contract_restore_output_formatting_is_identity_stub() {
    let code = "fn main() { let x = 1; }";
    assert_eq!(restore_output_formatting(code, "rust"), code);
    assert_eq!(restore_output_formatting(code, "python"), code);
    assert_eq!(restore_output_formatting("", "rust"), "");
}
