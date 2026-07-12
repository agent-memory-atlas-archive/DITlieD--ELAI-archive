//! I-6 contract smoke for the append-only AnchorPane context-budget slot.

use elai_orchestrator::context_budget::{ContextBudget, ContextSlot, SLOT_COUNT};

#[test]
fn anchor_pane_is_last_zero_default_budget_slot() {
    assert_eq!(ContextSlot::AnchorPane as u8, 10);
    assert_eq!(ContextSlot::AnchorPane as usize, SLOT_COUNT - 1);
    assert_eq!(SLOT_COUNT, 11);

    let budget = ContextBudget::new(131_072);
    assert_eq!(budget.slots.len(), SLOT_COUNT);
    assert_eq!(budget.slots[SLOT_COUNT - 1].slot, ContextSlot::AnchorPane);
    assert_eq!(budget.slots[SLOT_COUNT - 1].allocated_tokens, 0);
}

#[test]
fn anchor_pane_does_not_bust_code_context_cache_prefix() {
    let mut budget = ContextBudget::new(131_072);
    let first: [&[u8]; SLOT_COUNT] = [
        b"system",
        b"rules",
        b"phase",
        b"repo",
        b"code",
        b"ssb",
        b"grc",
        b"output",
        b"memory",
        b"evidence",
        b"anchor-v1",
    ];

    budget.update_breakpoints(&first);
    assert!(!budget.cache_breakpoints[0].valid);
    assert!(!budget.cache_breakpoints[1].valid);

    let second: [&[u8]; SLOT_COUNT] = [
        b"system",
        b"rules",
        b"phase",
        b"repo",
        b"code",
        b"ssb-changed",
        b"grc-changed",
        b"output-changed",
        b"memory-changed",
        b"evidence-changed",
        b"anchor-v2",
    ];

    budget.update_breakpoints(&second);
    assert!(budget.cache_breakpoints[0].valid);
    assert!(budget.cache_breakpoints[1].valid);
}
