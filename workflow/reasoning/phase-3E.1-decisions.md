Phase 3E.1 decisions

What: context_budget.rs — type definitions for ContextSlot, AdaptiveLevel, SlotAllocation, CacheBreakpoint, ContextBudget. Zero-init constructor. Display impl.

Key decisions:
- Used #[repr(u8)] on both ContextSlot and AdaptiveLevel for ABI stability and size guarantees.
- Added PartialEq/Eq to SlotAllocation and CacheBreakpoint beyond spec minimum to enable assert_eq in downstream phases (3E.2 allocation tests, 3E.7 determinism checks).
- ContextBudget gets PartialEq but not Eq (usize fields are Eq-compatible but PartialEq is sufficient for testing; adding Eq is trivial if needed).
- No Copy on ContextBudget — 248 bytes, implicit copying undesirable. Clone sufficient.
- Cache breakpoints: BP0 after PhaseSpec, BP1 after CodeContext. Matches spec section 6 fixed slot order.
- prefix_hash is u64, initialized to 0. No hasher invoked in 3E.1 — deferred to 3E.4.
- No new dependencies. blake3 already in Cargo.toml for later use.
- Display includes both per-slot breakdown and cache breakpoint state for debuggability.

Expected test outcome: 11 unit + 5 contract = 16/16 green.

Unknowns: None for 3E.1. Allocation logic (3E.2) will need model-specific token limits from elai-providers.
