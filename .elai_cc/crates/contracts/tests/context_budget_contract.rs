use elai_orchestrator::context_budget::{
    AdaptiveLevel, CacheBreakpoint, ContextBudget, ContextSlot, SlotAllocation,
    CACHE_BREAKPOINT_COUNT, SLOT_COUNT,
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
