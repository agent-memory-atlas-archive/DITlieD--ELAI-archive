use elai_orchestrator::context_budget::{
    AdaptiveLevel, BudgetError, CacheBreakpoint, ContextBudget, ContextSlot, SlotAllocation,
    CACHE_BREAKPOINT_COUNT, FLOOR_GRC_HISTORY, FLOOR_OUTPUT_RESERVE, FLOOR_SSB_ENTRIES,
    SLOT_COUNT, SUM_DYNAMIC_FLOORS,
};

const _: () = assert!(SLOT_COUNT == 8);
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
    let mut budget = ContextBudget::new(131072);
    let contents_a: Vec<&[u8]> = vec![
        b"sys", b"rules", b"spec", b"repo", b"code", b"ssb", b"grc", b"out",
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
        b"sys", b"rules", b"spec_v2", b"repo", b"code", b"ssb", b"grc", b"out",
    ];
    budget.update_breakpoints(&contents_b);
    assert!(!budget.cache_breakpoints[0].valid, "changed slot 2: BP0 must be invalid");
    assert!(!budget.cache_breakpoints[1].valid, "changed slot 2: BP1 must be invalid");
}

#[test]
fn update_breakpoints_bp1_ignores_post_bp1_slots() {
    // Contract: changing slots 5, 6, 7 does not invalidate BP1 (which covers 0..=4).
    let mut budget = ContextBudget::new(131072);
    let contents_a: Vec<&[u8]> = vec![
        b"sys", b"rules", b"spec", b"repo", b"code", b"ssb", b"grc", b"out",
    ];
    budget.update_breakpoints(&contents_a);
    // Second call with same 0..=4 but different 5,6,7.
    let contents_b: Vec<&[u8]> = vec![
        b"sys", b"rules", b"spec", b"repo", b"code", b"ssb_changed", b"grc_changed", b"out_changed",
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
