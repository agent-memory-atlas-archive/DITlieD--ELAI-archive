use std::fmt;

use blake3;

use crate::ssb::{ContextReductionPayload, Ssb, SsbPayload};

pub const SLOT_COUNT: usize = 8;
pub const CACHE_BREAKPOINT_COUNT: usize = 2;

// Floor constants — minimum token allocations for critical slots.
pub const FLOOR_SSB_ENTRIES: usize = 200;
pub const FLOOR_GRC_HISTORY: usize = 500;
pub const FLOOR_OUTPUT_RESERVE: usize = 2000;
pub const SUM_DYNAMIC_FLOORS: usize = FLOOR_SSB_ENTRIES + FLOOR_GRC_HISTORY;

// Percentage splits for dynamic budget (must sum to 100).
const PCT_REPO_MAP: usize = 15;
const PCT_CODE_CONTEXT: usize = 40;
const PCT_SSB_ENTRIES: usize = 25;
const PCT_GRC_HISTORY: usize = 20;
const _: () = assert!(PCT_REPO_MAP + PCT_CODE_CONTEXT + PCT_SSB_ENTRIES + PCT_GRC_HISTORY == 100);

// Adaptive reduction factors for Level 2 (ReducedGranularity).
// CodeContext reduced to 60% of current allocation; RepoMap to 50%.
const LEVEL2_CODE_CONTEXT_FACTOR_NUM: usize = 60;
const LEVEL2_CODE_CONTEXT_FACTOR_DEN: usize = 100;
const LEVEL2_REPO_MAP_FACTOR_NUM: usize = 50;
const LEVEL2_REPO_MAP_FACTOR_DEN: usize = 100;

#[derive(Debug, thiserror::Error)]
pub enum BudgetError {
    #[error("budget infeasible: dynamic_budget={dynamic_budget} < sum_floors={sum_floors}")]
    BudgetInfeasible {
        dynamic_budget: usize,
        sum_floors: usize,
    },
}

/// Execution mode determines terminal error variant when all reduction levels fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Interactive,
    Autonomous,
}

/// Snapshot of slot allocations at the time of an escalation or split decision.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBreakdown {
    pub slots: [SlotAllocation; SLOT_COUNT],
    pub total_used: usize,
    pub model_max_context: usize,
}

/// Errors returned by adaptive_reduce() when reduction cannot bring usage within budget.
#[derive(Debug, thiserror::Error)]
pub enum AdaptiveError {
    #[error("escalate: total_used={} exceeds max={}", .breakdown.total_used, .breakdown.model_max_context)]
    Escalate { breakdown: Box<BudgetBreakdown> },
    #[error("split_task: all reduction levels exhausted")]
    SplitTask,
    #[error("ssb write failed: {0}")]
    SsbWrite(String),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextSlot {
    SystemPrompt = 0,
    Rules = 1,
    PhaseSpec = 2,
    RepoMap = 3,
    CodeContext = 4,
    SsbEntries = 5,
    GrcHistory = 6,
    OutputReserve = 7,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveLevel {
    Normal = 0,
    FilteredByRelevance = 1,
    ReducedGranularity = 2,
    ReducedScope = 3,
    Terminal = 4,
}
const _: () = assert!(AdaptiveLevel::Terminal as u8 == 4);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotAllocation {
    pub slot: ContextSlot,
    pub allocated_tokens: usize,
    pub floor_tokens: usize,
    pub actual_tokens: usize,
    pub source_truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheBreakpoint {
    pub after_slot: ContextSlot,
    pub prefix_hash: u64,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextBudget {
    pub model_max_context: usize,
    pub slots: [SlotAllocation; SLOT_COUNT],
    pub cache_breakpoints: [CacheBreakpoint; CACHE_BREAKPOINT_COUNT],
    pub total_allocated: usize,
    pub total_used: usize,
    pub adaptive_level: AdaptiveLevel,
}

impl ContextBudget {
    pub fn new(model_max_context: usize) -> Self {
        Self {
            model_max_context,
            slots: [
                SlotAllocation {
                    slot: ContextSlot::SystemPrompt,
                    allocated_tokens: 0,
                    floor_tokens: 0,
                    actual_tokens: 0,
                    source_truncated: false,
                },
                SlotAllocation {
                    slot: ContextSlot::Rules,
                    allocated_tokens: 0,
                    floor_tokens: 0,
                    actual_tokens: 0,
                    source_truncated: false,
                },
                SlotAllocation {
                    slot: ContextSlot::PhaseSpec,
                    allocated_tokens: 0,
                    floor_tokens: 0,
                    actual_tokens: 0,
                    source_truncated: false,
                },
                SlotAllocation {
                    slot: ContextSlot::RepoMap,
                    allocated_tokens: 0,
                    floor_tokens: 0,
                    actual_tokens: 0,
                    source_truncated: false,
                },
                SlotAllocation {
                    slot: ContextSlot::CodeContext,
                    allocated_tokens: 0,
                    floor_tokens: 0,
                    actual_tokens: 0,
                    source_truncated: false,
                },
                SlotAllocation {
                    slot: ContextSlot::SsbEntries,
                    allocated_tokens: 0,
                    floor_tokens: 0,
                    actual_tokens: 0,
                    source_truncated: false,
                },
                SlotAllocation {
                    slot: ContextSlot::GrcHistory,
                    allocated_tokens: 0,
                    floor_tokens: 0,
                    actual_tokens: 0,
                    source_truncated: false,
                },
                SlotAllocation {
                    slot: ContextSlot::OutputReserve,
                    allocated_tokens: 0,
                    floor_tokens: 0,
                    actual_tokens: 0,
                    source_truncated: false,
                },
            ],
            cache_breakpoints: [
                CacheBreakpoint {
                    after_slot: ContextSlot::PhaseSpec,
                    prefix_hash: 0,
                    valid: false,
                },
                CacheBreakpoint {
                    after_slot: ContextSlot::CodeContext,
                    prefix_hash: 0,
                    valid: false,
                },
            ],
            total_allocated: 0,
            total_used: 0,
            adaptive_level: AdaptiveLevel::Normal,
        }
    }
}

impl ContextBudget {
    /// Compute a context budget with percentage-based allocation and floor enforcement.
    ///
    /// `fixed_slot_sizes` provides token counts for [SystemPrompt, Rules, PhaseSpec].
    /// Dynamic slots (RepoMap, CodeContext, SsbEntries, GrcHistory) are split by percentage
    /// from the remaining budget, then floor-enforced by stealing from larger donors.
    pub fn compute(
        model_max_context: usize,
        fixed_slot_sizes: [usize; 3],
        output_reserve: usize,
    ) -> Result<Self, BudgetError> {
        let effective_reserve = output_reserve.max(FLOOR_OUTPUT_RESERVE);
        let fixed_total = fixed_slot_sizes[0] + fixed_slot_sizes[1] + fixed_slot_sizes[2];

        // Detect underflow: if fixed + reserve exceeds model capacity.
        let dynamic_budget = model_max_context
            .checked_sub(fixed_total)
            .and_then(|v| v.checked_sub(effective_reserve))
            .unwrap_or(0);

        if dynamic_budget < SUM_DYNAMIC_FLOORS {
            return Err(BudgetError::BudgetInfeasible {
                dynamic_budget,
                sum_floors: SUM_DYNAMIC_FLOORS,
            });
        }

        // Percentage split (integer division = floor).
        let mut repo_map_alloc = dynamic_budget * PCT_REPO_MAP / 100;
        let mut code_ctx_alloc = dynamic_budget * PCT_CODE_CONTEXT / 100;
        let mut ssb_alloc = dynamic_budget * PCT_SSB_ENTRIES / 100;
        let mut grc_alloc = dynamic_budget * PCT_GRC_HISTORY / 100;

        // Assign remainder from integer division to CodeContext (largest slot).
        let subtotal = repo_map_alloc + code_ctx_alloc + ssb_alloc + grc_alloc;
        code_ctx_alloc += dynamic_budget - subtotal;

        // Floor enforcement for GrcHistory: steal from CodeContext first, then RepoMap.
        if grc_alloc < FLOOR_GRC_HISTORY {
            let deficit = FLOOR_GRC_HISTORY - grc_alloc;
            let steal_from_code = deficit.min(code_ctx_alloc);
            code_ctx_alloc -= steal_from_code;
            let remaining_deficit = deficit - steal_from_code;
            let steal_from_repo = remaining_deficit.min(repo_map_alloc);
            repo_map_alloc -= steal_from_repo;
            let final_deficit = remaining_deficit - steal_from_repo;
            if final_deficit > 0 {
                return Err(BudgetError::BudgetInfeasible {
                    dynamic_budget,
                    sum_floors: SUM_DYNAMIC_FLOORS,
                });
            }
            grc_alloc = FLOOR_GRC_HISTORY;
        }

        // Floor enforcement for SsbEntries: steal from CodeContext first, then RepoMap.
        if ssb_alloc < FLOOR_SSB_ENTRIES {
            let deficit = FLOOR_SSB_ENTRIES - ssb_alloc;
            let steal_from_code = deficit.min(code_ctx_alloc);
            code_ctx_alloc -= steal_from_code;
            let remaining_deficit = deficit - steal_from_code;
            let steal_from_repo = remaining_deficit.min(repo_map_alloc);
            repo_map_alloc -= steal_from_repo;
            let final_deficit = remaining_deficit - steal_from_repo;
            if final_deficit > 0 {
                return Err(BudgetError::BudgetInfeasible {
                    dynamic_budget,
                    sum_floors: SUM_DYNAMIC_FLOORS,
                });
            }
            ssb_alloc = FLOOR_SSB_ENTRIES;
        }

        // Zero-sum invariant: dynamic allocations must sum to dynamic_budget.
        assert_eq!(
            repo_map_alloc + code_ctx_alloc + ssb_alloc + grc_alloc,
            dynamic_budget,
            "zero-sum invariant violated after floor enforcement"
        );

        let mut budget = Self::new(model_max_context);

        // Fixed slots.
        budget.slots[0].allocated_tokens = fixed_slot_sizes[0];
        budget.slots[1].allocated_tokens = fixed_slot_sizes[1];
        budget.slots[2].allocated_tokens = fixed_slot_sizes[2];

        // Dynamic slots.
        budget.slots[3].allocated_tokens = repo_map_alloc;
        budget.slots[4].allocated_tokens = code_ctx_alloc;
        budget.slots[5].allocated_tokens = ssb_alloc;
        budget.slots[5].floor_tokens = FLOOR_SSB_ENTRIES;
        budget.slots[6].allocated_tokens = grc_alloc;
        budget.slots[6].floor_tokens = FLOOR_GRC_HISTORY;

        // Output reserve.
        budget.slots[7].allocated_tokens = effective_reserve;
        budget.slots[7].floor_tokens = FLOOR_OUTPUT_RESERVE;

        budget.total_allocated = fixed_total + dynamic_budget + effective_reserve;
        assert_eq!(
            budget.total_allocated, model_max_context,
            "total_allocated must equal model_max_context"
        );

        Ok(budget)
    }
}

impl ContextBudget {
    /// Update cache breakpoint validity by hashing the prefix content for each breakpoint.
    ///
    /// `slot_contents` must have exactly `SLOT_COUNT` entries (one per slot, in order).
    /// Each breakpoint covers slots `0..=after_slot as usize`. If the prefix hash is
    /// unchanged from the previous call, `valid` is set to `true`; otherwise `false`.
    /// On the first call (stored hash is 0), the hash is stored but `valid` remains `false`.
    pub fn update_breakpoints(&mut self, slot_contents: &[&[u8]]) {
        debug_assert_eq!(
            slot_contents.len(),
            SLOT_COUNT,
            "slot_contents must have exactly SLOT_COUNT entries"
        );
        if slot_contents.len() != SLOT_COUNT {
            return;
        }

        for bp in &mut self.cache_breakpoints {
            let bound = bp.after_slot as usize;
            let mut hasher = blake3::Hasher::new();
            for slot_bytes in &slot_contents[..=bound] {
                // Length-prefix each slot to prevent boundary ambiguity:
                // [b"ab", b"cd"] must hash differently from [b"a", b"bcd"].
                hasher.update(&(slot_bytes.len() as u64).to_le_bytes());
                hasher.update(slot_bytes);
            }
            let h = hasher.finalize();
            // blake3 output is [u8; 32]; extracting first 8 bytes is infallible.
            let b = h.as_bytes();
            let new_hash = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);

            let prev_hash = bp.prefix_hash;
            if prev_hash == 0 || prev_hash != new_hash {
                bp.valid = false;
                bp.prefix_hash = new_hash;
            } else {
                bp.valid = true;
            }
        }
    }
}

impl ContextBudget {
    /// Adaptively reduce slot allocations when total_used exceeds model_max_context.
    ///
    /// Applies up to 3 reduction levels in sequence. After each level, checks if
    /// total_used fits within model_max_context. Logs each effective reduction to SSB.
    /// If all 3 levels fail, returns a terminal error based on `mode`.
    ///
    /// `overlap_threshold` controls Level 1 filtering intensity (0.0 = no reduction,
    /// 1.0 = maximum reduction). Decoupled from SSB entry filtering (3E.5).
    pub async fn adaptive_reduce(
        &mut self,
        overlap_threshold: f64,
        mode: ExecutionMode,
        ssb: &Ssb,
    ) -> Result<(), AdaptiveError> {
        let levels = [
            AdaptiveLevel::FilteredByRelevance,
            AdaptiveLevel::ReducedGranularity,
            AdaptiveLevel::ReducedScope,
        ];

        for &level in &levels {
            let before_tokens = self.total_used;
            let slots_affected = self.apply_reduction_level(level, overlap_threshold);

            // Cap actual_tokens to allocated_tokens (models content truncation).
            self.cap_actual_to_allocated();
            self.recalculate_total_used();
            let after_tokens = self.total_used;
            self.adaptive_level = level;

            // Log to SSB only if the level actually changed effective usage.
            if before_tokens != after_tokens {
                let payload = SsbPayload::ContextReduction(ContextReductionPayload {
                    level: level as u8,
                    before_tokens,
                    after_tokens,
                    slots_affected,
                });
                ssb.write_payload("context_budget", 0, payload, vec![])
                    .await
                    .map_err(|e| AdaptiveError::SsbWrite(e.to_string()))?;
            }

            if self.total_used <= self.model_max_context {
                return Ok(());
            }
        }

        // All 3 levels exhausted — terminal.
        self.adaptive_level = AdaptiveLevel::Terminal;
        match mode {
            ExecutionMode::Interactive => Err(AdaptiveError::Escalate {
                breakdown: Box::new(BudgetBreakdown {
                    slots: self.slots,
                    total_used: self.total_used,
                    model_max_context: self.model_max_context,
                }),
            }),
            ExecutionMode::Autonomous => Err(AdaptiveError::SplitTask),
        }
    }

    /// Apply a single reduction level, returning the list of slot indices affected.
    fn apply_reduction_level(
        &mut self,
        level: AdaptiveLevel,
        overlap_threshold: f64,
    ) -> Vec<u8> {
        match level {
            AdaptiveLevel::FilteredByRelevance => {
                self.reduce_level1_filtered(overlap_threshold)
            }
            AdaptiveLevel::ReducedGranularity => {
                self.reduce_level2_granularity()
            }
            AdaptiveLevel::ReducedScope => {
                self.reduce_level3_scope()
            }
            // Normal and Terminal are not reduction levels.
            AdaptiveLevel::Normal | AdaptiveLevel::Terminal => vec![],
        }
    }

    /// Level 1: reduce GrcHistory and SsbEntries allocations by overlap_threshold factor.
    /// Higher threshold = more aggressive reduction. Floors enforced.
    fn reduce_level1_filtered(&mut self, overlap_threshold: f64) -> Vec<u8> {
        let threshold = overlap_threshold.clamp(0.0, 1.0);
        let mut affected = Vec::new();

        // GrcHistory (slot 6): reduce by threshold factor.
        let grc = &mut self.slots[ContextSlot::GrcHistory as usize];
        let new_grc = scale_down(grc.allocated_tokens, threshold, grc.floor_tokens);
        if new_grc != grc.allocated_tokens {
            grc.allocated_tokens = new_grc;
            affected.push(ContextSlot::GrcHistory as u8);
        }

        // SsbEntries (slot 5): reduce by threshold factor.
        let ssb_slot = &mut self.slots[ContextSlot::SsbEntries as usize];
        let new_ssb = scale_down(ssb_slot.allocated_tokens, threshold, ssb_slot.floor_tokens);
        if new_ssb != ssb_slot.allocated_tokens {
            ssb_slot.allocated_tokens = new_ssb;
            affected.push(ContextSlot::SsbEntries as u8);
        }

        affected
    }

    /// Level 2: reduce CodeContext to 60% and RepoMap to 50% of current allocations.
    fn reduce_level2_granularity(&mut self) -> Vec<u8> {
        let mut affected = Vec::new();

        let code = &mut self.slots[ContextSlot::CodeContext as usize];
        let new_code = (code.allocated_tokens * LEVEL2_CODE_CONTEXT_FACTOR_NUM
            / LEVEL2_CODE_CONTEXT_FACTOR_DEN)
            .max(code.floor_tokens);
        if new_code != code.allocated_tokens {
            code.allocated_tokens = new_code;
            affected.push(ContextSlot::CodeContext as u8);
        }

        let repo = &mut self.slots[ContextSlot::RepoMap as usize];
        let new_repo = (repo.allocated_tokens * LEVEL2_REPO_MAP_FACTOR_NUM
            / LEVEL2_REPO_MAP_FACTOR_DEN)
            .max(repo.floor_tokens);
        if new_repo != repo.allocated_tokens {
            repo.allocated_tokens = new_repo;
            affected.push(ContextSlot::RepoMap as u8);
        }

        affected
    }

    /// Level 3: clamp CodeContext, RepoMap, and SsbEntries to their floor values.
    /// Slots with floor_tokens=0 are clamped to 0 (freeing the entire allocation).
    fn reduce_level3_scope(&mut self) -> Vec<u8> {
        let mut affected = Vec::new();

        let targets = [
            ContextSlot::CodeContext,
            ContextSlot::RepoMap,
            ContextSlot::SsbEntries,
        ];
        for &slot in &targets {
            let s = &mut self.slots[slot as usize];
            if s.allocated_tokens > s.floor_tokens {
                s.allocated_tokens = s.floor_tokens;
                affected.push(slot as u8);
            }
        }

        affected
    }

    /// Cap actual_tokens to allocated_tokens for dynamic slots only.
    /// Fixed slots (SystemPrompt, Rules, PhaseSpec) and OutputReserve are not truncatable
    /// by reduction — their content is provided as-is. Only dynamic slots (RepoMap,
    /// CodeContext, SsbEntries, GrcHistory) are capped after allocation reduction.
    pub fn cap_actual_to_allocated(&mut self) {
        const DYNAMIC_SLOTS: [ContextSlot; 4] = [
            ContextSlot::RepoMap,
            ContextSlot::CodeContext,
            ContextSlot::SsbEntries,
            ContextSlot::GrcHistory,
        ];
        for &ds in &DYNAMIC_SLOTS {
            let slot = &mut self.slots[ds as usize];
            if slot.actual_tokens > slot.allocated_tokens {
                slot.actual_tokens = slot.allocated_tokens;
                slot.source_truncated = true;
            }
        }
    }

    /// Recalculate total_used as the sum of actual_tokens across all slots.
    /// Also recalculate total_allocated from allocated_tokens.
    pub fn recalculate_total_used(&mut self) {
        self.total_used = self.slots.iter().map(|s| s.actual_tokens).sum();
        self.total_allocated = self.slots.iter().map(|s| s.allocated_tokens).sum();
    }
}

/// Scale down a token count by `(1 - factor)`, floored at `floor`.
/// `factor` in [0.0, 1.0]: 0.0 = no reduction, 1.0 = reduce to floor.
fn scale_down(current: usize, factor: f64, floor: usize) -> usize {
    let reduced = (current as f64 * (1.0 - factor)) as usize;
    reduced.max(floor)
}

impl fmt::Display for ContextBudget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "ContextBudget(max={}, allocated={}, used={}, level={:?})",
            self.model_max_context, self.total_allocated, self.total_used, self.adaptive_level
        )?;
        for slot in &self.slots {
            writeln!(
                f,
                "  {:?}: alloc={} actual={} floor={} truncated={}",
                slot.slot,
                slot.allocated_tokens,
                slot.actual_tokens,
                slot.floor_tokens,
                slot.source_truncated
            )?;
        }
        for (i, bp) in self.cache_breakpoints.iter().enumerate() {
            writeln!(
                f,
                "  BP{}: after={:?} hash={} valid={}",
                i, bp.after_slot, bp.prefix_hash, bp.valid
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_slot_repr_ordering() {
        assert_eq!(ContextSlot::SystemPrompt as u8, 0);
        assert_eq!(ContextSlot::Rules as u8, 1);
        assert_eq!(ContextSlot::PhaseSpec as u8, 2);
        assert_eq!(ContextSlot::RepoMap as u8, 3);
        assert_eq!(ContextSlot::CodeContext as u8, 4);
        assert_eq!(ContextSlot::SsbEntries as u8, 5);
        assert_eq!(ContextSlot::GrcHistory as u8, 6);
        assert_eq!(ContextSlot::OutputReserve as u8, 7);
    }

    #[test]
    fn new_budget_total_allocated_zero() {
        let budget = ContextBudget::new(131072);
        assert_eq!(budget.total_allocated, 0);
    }

    #[test]
    fn new_budget_total_used_zero() {
        let budget = ContextBudget::new(131072);
        assert_eq!(budget.total_used, 0);
    }

    #[test]
    fn new_budget_adaptive_level_normal() {
        let budget = ContextBudget::new(131072);
        assert_eq!(budget.adaptive_level, AdaptiveLevel::Normal);
    }

    #[test]
    fn new_budget_model_max_context() {
        let budget = ContextBudget::new(131072);
        assert_eq!(budget.model_max_context, 131072);
    }

    #[test]
    fn new_budget_all_slots_zero_initialized() {
        let budget = ContextBudget::new(131072);
        assert_eq!(budget.slots.len(), SLOT_COUNT);
        for (i, slot) in budget.slots.iter().enumerate() {
            assert_eq!(
                slot.slot as u8, i as u8,
                "slot {} has wrong variant",
                i
            );
            assert_eq!(slot.allocated_tokens, 0, "slot {} allocated != 0", i);
            assert_eq!(slot.floor_tokens, 0, "slot {} floor != 0", i);
            assert_eq!(slot.actual_tokens, 0, "slot {} actual != 0", i);
            assert!(!slot.source_truncated, "slot {} truncated != false", i);
        }
    }

    #[test]
    fn new_budget_cache_breakpoints_invalid() {
        let budget = ContextBudget::new(131072);
        assert_eq!(budget.cache_breakpoints.len(), CACHE_BREAKPOINT_COUNT);
        for bp in &budget.cache_breakpoints {
            assert!(!bp.valid);
            assert_eq!(bp.prefix_hash, 0);
        }
    }

    #[test]
    fn cache_breakpoint_slot_positions() {
        let budget = ContextBudget::new(131072);
        assert_eq!(budget.cache_breakpoints[0].after_slot, ContextSlot::PhaseSpec);
        assert_eq!(budget.cache_breakpoints[1].after_slot, ContextSlot::CodeContext);
    }

    #[test]
    fn display_includes_max_context() {
        let budget = ContextBudget::new(131072);
        let output = format!("{budget}");
        assert!(output.contains("max=131072"));
    }

    #[test]
    fn display_includes_all_slots() {
        let budget = ContextBudget::new(131072);
        let output = format!("{budget}");
        assert!(output.contains("SystemPrompt"));
        assert!(output.contains("OutputReserve"));
        assert!(output.contains("GrcHistory"));
    }

    #[test]
    fn display_includes_breakpoints() {
        let budget = ContextBudget::new(131072);
        let output = format!("{budget}");
        assert!(output.contains("BP0"));
        assert!(output.contains("BP1"));
        assert!(output.contains("valid=false"));
    }

    #[test]
    fn floor_enforcement_steals_from_code_context() {
        // compute(4000, [100,200,300], 2000):
        // fixed=600, reserve=2000, dynamic=1400
        // repo=210, code=560, ssb=350, grc=280; remainder=0
        // GRC 280<500: steal 220 from code -> code=340, grc=500
        // SSB 350>=200: no steal
        let budget = ContextBudget::compute(4000, [100, 200, 300], 2000).unwrap();
        assert_eq!(budget.slots[6].allocated_tokens, 500); // GRC floor enforced
        assert!(budget.slots[6].allocated_tokens >= FLOOR_GRC_HISTORY);
        assert_eq!(budget.slots[4].allocated_tokens, 340); // CodeContext after steal
        assert_eq!(budget.slots[3].allocated_tokens, 210); // RepoMap untouched
        assert_eq!(budget.slots[5].allocated_tokens, 350); // SSB no steal needed
        assert!(budget.slots[5].allocated_tokens >= FLOOR_SSB_ENTRIES);
        // Zero-sum: dynamic slots sum to dynamic_budget=1400
        let dynamic_sum = budget.slots[3].allocated_tokens
            + budget.slots[4].allocated_tokens
            + budget.slots[5].allocated_tokens
            + budget.slots[6].allocated_tokens;
        assert_eq!(dynamic_sum, 1400);
        assert_eq!(budget.total_allocated, 4000);
    }

    #[test]
    fn budget_infeasible_when_dynamic_too_small() {
        // compute(3000, [100,200,300], 2000):
        // fixed=600, reserve=2000, dynamic=400 < 700
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
    fn output_reserve_clamped_to_floor() {
        // output_reserve=500 < FLOOR_OUTPUT_RESERVE=2000, should clamp to 2000.
        let budget = ContextBudget::compute(10000, [100, 200, 300], 500).unwrap();
        assert_eq!(budget.slots[7].allocated_tokens, FLOOR_OUTPUT_RESERVE);
        assert_eq!(budget.total_allocated, 10000);
    }

    fn make_slot_contents() -> Vec<&'static [u8]> {
        vec![
            b"system_prompt",
            b"rules_v1",
            b"phase_spec",
            b"repo_map",
            b"code_context",
            b"ssb_entries",
            b"grc_history",
            b"output_reserve",
        ]
    }

    #[test]
    fn bp0_valid_on_repeated_same_content() {
        let mut budget = ContextBudget::new(131072);
        let contents = make_slot_contents();
        budget.update_breakpoints(&contents);
        assert!(!budget.cache_breakpoints[0].valid, "first call must be invalid");
        budget.update_breakpoints(&contents);
        assert!(budget.cache_breakpoints[0].valid, "second call with same content must be valid");
    }

    #[test]
    fn bp0_invalid_on_changed_slot1() {
        let mut budget = ContextBudget::new(131072);
        let contents_a = make_slot_contents();
        budget.update_breakpoints(&contents_a);
        let contents_b: Vec<&[u8]> = vec![
            b"system_prompt",
            b"rules_v2",
            b"phase_spec",
            b"repo_map",
            b"code_context",
            b"ssb_entries",
            b"grc_history",
            b"output_reserve",
        ];
        budget.update_breakpoints(&contents_b);
        assert!(!budget.cache_breakpoints[0].valid, "BP0 must be invalid when slot 1 changes");
    }

    #[test]
    fn bp1_valid_when_only_slot5_changes() {
        let mut budget = ContextBudget::new(131072);
        let contents_a = make_slot_contents();
        budget.update_breakpoints(&contents_a);
        let contents_b: Vec<&[u8]> = vec![
            b"system_prompt",
            b"rules_v1",
            b"phase_spec",
            b"repo_map",
            b"code_context",
            b"ssb_entries_changed",
            b"grc_history",
            b"output_reserve",
        ];
        budget.update_breakpoints(&contents_b);
        assert!(budget.cache_breakpoints[1].valid, "BP1 must be valid when only slot 5 changes");
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "slot_contents must have exactly SLOT_COUNT entries")]
    fn wrong_length_input_panics_in_debug() {
        let mut budget = ContextBudget::new(131072);
        let short: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d", b"e", b"f", b"g"];
        budget.update_breakpoints(&short);
    }

    #[test]
    fn bp1_invalid_when_slot3_changes_bp0_unaffected() {
        let mut budget = ContextBudget::new(131072);
        let contents_a = make_slot_contents();
        budget.update_breakpoints(&contents_a);
        budget.update_breakpoints(&contents_a); // establish valid=true
        assert!(budget.cache_breakpoints[0].valid);
        assert!(budget.cache_breakpoints[1].valid);
        // Change slot 3 (RepoMap) — inside BP1 (0..=4) but outside BP0 (0..=2).
        let contents_b: Vec<&[u8]> = vec![
            b"system_prompt",
            b"rules_v1",
            b"phase_spec",
            b"repo_map_v2",
            b"code_context",
            b"ssb_entries",
            b"grc_history",
            b"output_reserve",
        ];
        budget.update_breakpoints(&contents_b);
        assert!(budget.cache_breakpoints[0].valid, "BP0 must stay valid (slot 3 is outside 0..=2)");
        assert!(!budget.cache_breakpoints[1].valid, "BP1 must be invalid (slot 3 is inside 0..=4)");
    }

    #[test]
    fn first_call_sets_hash_not_valid() {
        let mut budget = ContextBudget::new(131072);
        let contents = make_slot_contents();
        budget.update_breakpoints(&contents);
        assert_ne!(budget.cache_breakpoints[0].prefix_hash, 0, "hash must be set after first call");
        assert!(!budget.cache_breakpoints[0].valid, "first call must leave valid=false");
        assert_ne!(budget.cache_breakpoints[1].prefix_hash, 0, "BP1 hash must be set after first call");
        assert!(!budget.cache_breakpoints[1].valid, "BP1 first call must leave valid=false");
    }

    #[test]
    fn hash_deterministic_1000_calls() {
        let mut budget = ContextBudget::new(131072);
        let contents = make_slot_contents();
        budget.update_breakpoints(&contents);
        let bp0_hash = budget.cache_breakpoints[0].prefix_hash;
        let bp1_hash = budget.cache_breakpoints[1].prefix_hash;
        for _ in 0..1000 {
            budget.update_breakpoints(&contents);
            assert_eq!(budget.cache_breakpoints[0].prefix_hash, bp0_hash, "BP0 hash must be deterministic");
            assert_eq!(budget.cache_breakpoints[1].prefix_hash, bp1_hash, "BP1 hash must be deterministic");
        }
    }

    #[test]
    fn no_stealing_needed_large_budget() {
        // compute(100000, [100,200,300], 2000):
        // fixed=600, reserve=2000, dynamic=97400
        // All slots well above floors.
        let budget = ContextBudget::compute(100000, [100, 200, 300], 2000).unwrap();
        assert!(budget.slots[5].allocated_tokens >= FLOOR_SSB_ENTRIES);
        assert!(budget.slots[6].allocated_tokens >= FLOOR_GRC_HISTORY);
        assert_eq!(budget.total_allocated, 100000);
        // Verify percentages (approximate due to remainder going to CodeContext).
        let dynamic = 97400_usize;
        assert_eq!(budget.slots[3].allocated_tokens, dynamic * PCT_REPO_MAP / 100);
        assert_eq!(budget.slots[5].allocated_tokens, dynamic * PCT_SSB_ENTRIES / 100);
        assert_eq!(budget.slots[6].allocated_tokens, dynamic * PCT_GRC_HISTORY / 100);
        // CodeContext gets its share + remainder.
        let expected_code = dynamic * PCT_CODE_CONTEXT / 100
            + (dynamic
                - dynamic * PCT_REPO_MAP / 100
                - dynamic * PCT_CODE_CONTEXT / 100
                - dynamic * PCT_SSB_ENTRIES / 100
                - dynamic * PCT_GRC_HISTORY / 100);
        assert_eq!(budget.slots[4].allocated_tokens, expected_code);
    }
}
