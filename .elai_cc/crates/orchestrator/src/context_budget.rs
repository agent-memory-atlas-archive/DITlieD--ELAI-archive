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

// Assembly-time token targets per slot per tier (token-efficiency-design.txt section 9).
// These control content trimming at dispatch time — independent of budget allocation.
pub const MICRO_SLOT_SYSTEM_PROMPT: usize = 200;
pub const MICRO_SLOT_RULES: usize = 150;
pub const MICRO_SLOT_TELESCOPE: usize = 400;
pub const MICRO_SLOT_REPO_MAP: usize = 0;
pub const MICRO_SLOT_CODE_CONTEXT: usize = 1200;
pub const MICRO_SLOT_SSB_ENTRIES: usize = 200;
pub const MICRO_SLOT_GRC_HISTORY: usize = 100;
pub const MICRO_SLOT_OUTPUT_RESERVE: usize = 750;

pub const LARGE_SLOT_SYSTEM_PROMPT: usize = 330;
pub const LARGE_SLOT_RULES: usize = 500;
pub const LARGE_SLOT_TELESCOPE: usize = 1000;
pub const LARGE_SLOT_REPO_MAP: usize = 2000;
pub const LARGE_SLOT_CODE_CONTEXT: usize = 6000;
pub const LARGE_SLOT_SSB_ENTRIES: usize = 1500;
pub const LARGE_SLOT_GRC_HISTORY: usize = 1000;
pub const LARGE_SLOT_OUTPUT_RESERVE: usize = 4000;

// Default percentage splits for dynamic budget (must sum to 100).
// Used by compute() for backward compatibility; compute_with_profile() uses SlotWeights.
const PCT_REPO_MAP: usize = 15;
const PCT_CODE_CONTEXT: usize = 40;
const PCT_SSB_ENTRIES: usize = 25;
const PCT_GRC_HISTORY: usize = 20;
const _: () = assert!(PCT_REPO_MAP + PCT_CODE_CONTEXT + PCT_SSB_ENTRIES + PCT_GRC_HISTORY == 100);

// ---------------------------------------------------------------------------
// Generation profile (A-18: reasoning-architecture-design.txt GENERATION PARAMETERS)
// ---------------------------------------------------------------------------

/// Per-use-case generation parameters for temperature and top-p.
///
/// Research grounding (FixAudit, arXiv:2604.05560): repair retries use
/// temperature=1.0, top-p=1.0 to prevent cyclic traps. Single 7B model
/// zero-shot 11.93% -> FixAudit 30.40%.
///
/// First attempts use low temperature (0.3) for maximum pass@1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenerationProfile {
    /// Initial code generation — focused, high pass@1.
    FirstAttempt,
    /// Micro-reflection diagnosis — focused, precise.
    MicroReflection,
    /// Interrogation tree multiple choice — near-deterministic.
    Interrogation,
    /// Repair retry after failure — high temp to escape local minima.
    RepairRetry,
    /// AB-MCTS branch A — moderate diversity.
    TreeBranchA,
    /// AB-MCTS branch B — max diversity.
    TreeBranchB,
    /// AB-MCTS branch C — focused alternative.
    TreeBranchC,
}

impl GenerationProfile {
    /// Temperature for this profile.
    pub fn temperature(&self) -> f32 {
        match self {
            Self::FirstAttempt => TEMP_FIRST_ATTEMPT,
            Self::MicroReflection => TEMP_MICRO_REFLECTION,
            Self::Interrogation => TEMP_INTERROGATION,
            Self::RepairRetry => TEMP_REPAIR_RETRY,
            Self::TreeBranchA => TEMP_TREE_BRANCH_A,
            Self::TreeBranchB => TEMP_TREE_BRANCH_B,
            Self::TreeBranchC => TEMP_TREE_BRANCH_C,
        }
    }

    /// Top-p for this profile.
    pub fn top_p(&self) -> f32 {
        match self {
            Self::FirstAttempt => TOP_P_FIRST_ATTEMPT,
            Self::MicroReflection => TOP_P_MICRO_REFLECTION,
            Self::Interrogation => TOP_P_INTERROGATION,
            Self::RepairRetry => TOP_P_REPAIR_RETRY,
            Self::TreeBranchA => TOP_P_TREE_BRANCH_A,
            Self::TreeBranchB => TOP_P_TREE_BRANCH_B,
            Self::TreeBranchC => TOP_P_TREE_BRANCH_C,
        }
    }
}

// Named constants for generation parameters (no magic numbers).
const TEMP_FIRST_ATTEMPT: f32 = 0.3;
const TEMP_MICRO_REFLECTION: f32 = 0.3;
const TEMP_INTERROGATION: f32 = 0.1;
const TEMP_REPAIR_RETRY: f32 = 1.0;
const TEMP_TREE_BRANCH_A: f32 = 0.7;
const TEMP_TREE_BRANCH_B: f32 = 1.0;
const TEMP_TREE_BRANCH_C: f32 = 0.3;

const TOP_P_FIRST_ATTEMPT: f32 = 0.9;
const TOP_P_MICRO_REFLECTION: f32 = 0.9;
const TOP_P_INTERROGATION: f32 = 0.9;
const TOP_P_REPAIR_RETRY: f32 = 1.0;
const TOP_P_TREE_BRANCH_A: f32 = 1.0;
const TOP_P_TREE_BRANCH_B: f32 = 1.0;
const TOP_P_TREE_BRANCH_C: f32 = 0.9;

// ---------------------------------------------------------------------------
// REPAIR_CONTEXT slot allocation (A-18: compressed context replay)
// ---------------------------------------------------------------------------

/// Total token budget for repair context (fits any model tier).
const REPAIR_CONTEXT_TOTAL: usize = 1400;

/// Repair context slot token allocations from reasoning-architecture-design.txt.
const REPAIR_SLOT_SYSTEM_PROMPT: usize = 50;
const REPAIR_SLOT_BABY_STEP_SPEC: usize = 200;
const REPAIR_SLOT_MISTAKE_BOOK: usize = 150;
const REPAIR_SLOT_RELEVANT_CODE: usize = 500;
const REPAIR_SLOT_WORKING_EXAMPLE: usize = 300;
const REPAIR_SLOT_FIX_INSTRUCTION: usize = 100;
const REPAIR_SLOT_OUTPUT_RESERVE: usize = 100;

const _: () = assert!(
    REPAIR_SLOT_SYSTEM_PROMPT
        + REPAIR_SLOT_BABY_STEP_SPEC
        + REPAIR_SLOT_MISTAKE_BOOK
        + REPAIR_SLOT_RELEVANT_CODE
        + REPAIR_SLOT_WORKING_EXAMPLE
        + REPAIR_SLOT_FIX_INSTRUCTION
        + REPAIR_SLOT_OUTPUT_RESERVE
        == REPAIR_CONTEXT_TOTAL
);

/// Repair context slot weights for the context budget system.
/// Maps the 7-slot repair layout onto the existing 8-slot ContextSlot scheme:
///   SystemPrompt  -> SystemPrompt (50 tokens)
///   Rules         -> BabyStepSpec (200 tokens — spec replaces rules in retry)
///   PhaseSpec     -> MistakeBook (150 tokens — constraints replace phase spec)
///   RepoMap       -> 0 (dropped in repair context — DKG replaces broad map)
///   CodeContext   -> RelevantCode + WorkingExample (800 tokens combined)
///   SsbEntries    -> 0 (SSB history dropped — fresh start per retry)
///   GrcHistory    -> FixInstruction (100 tokens — diagnosis replaces GRC)
///   OutputReserve -> OutputReserve (100 tokens)
impl SlotWeights {
    /// Repair context weights: heavy code context (DKG + example), no RepoMap/SSB.
    ///
    /// Dynamic slots (repo_map + code_context + ssb_entries + grc_history) must sum to 1.0.
    /// Repair context: code_context=0.89 (relevant code + working example),
    /// grc_history=0.11 (fix instruction), repo_map=0.0, ssb_entries=0.0.
    pub fn repair_context() -> Self {
        Self {
            system_prompt: 0.0,
            rules: 0.0,
            telescope: 0.0,
            repo_map: 0.0,
            code_context: 0.89,
            ssb_entries: 0.0,
            grc_history: 0.11,
        }
    }
}

// ---------------------------------------------------------------------------
// Model capability types (A-10: token-efficiency-design.txt section 13)
// ---------------------------------------------------------------------------

// VramTier moved to elai-core in A-37. Re-export for backwards compatibility.
pub use elai_core::VramTier;

/// Model quality tier — determines baby step sizing and decomposition depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelTier {
    Micro,  // 3B-class, 72% pass@1
    Small,  // 7B-class, 90% pass@1
    Medium, // 14B-class, projected
    Large,  // 32B-class, 92.7% HumanEval
    Cloud,  // frontier API models
}

/// Per-slot allocation weights for dynamic budget distribution.
/// The four dynamic slot weights (repo_map, code_context, ssb_entries, grc_history)
/// must sum to 1.0. Fixed slot weights (system_prompt, rules, telescope) are target
/// sizes used for validation, not distribution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotWeights {
    pub system_prompt: f32,
    pub rules: f32,
    pub telescope: f32,
    pub repo_map: f32,
    pub code_context: f32,
    pub ssb_entries: f32,
    pub grc_history: f32,
}

/// Tolerance for floating-point weight sum validation.
const WEIGHT_SUM_TOLERANCE: f32 = 0.01;

impl SlotWeights {
    /// Default weights matching the original fixed percentages (15/40/25/20).
    pub fn default_fixed() -> Self {
        Self {
            system_prompt: 0.0,
            rules: 0.0,
            telescope: 0.0,
            repo_map: 0.15,
            code_context: 0.40,
            ssb_entries: 0.25,
            grc_history: 0.20,
        }
    }

    /// Validate that dynamic slot weights sum to ~1.0.
    pub fn validate_dynamic_sum(&self) -> bool {
        let sum = self.repo_map + self.code_context + self.ssb_entries + self.grc_history;
        (sum - 1.0).abs() < WEIGHT_SUM_TOLERANCE
    }
}

/// Model feature flags — capabilities detected by baseline measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelFeatures {
    pub supports_structured_output: bool,
    pub supports_grammar: bool,
    pub supports_stop_sequences: bool,
    pub chunked_prefill: bool,
    pub kv_compression: bool,
}

impl ModelFeatures {
    /// Conservative defaults: no optional features.
    pub fn none() -> Self {
        Self {
            supports_structured_output: false,
            supports_grammar: false,
            supports_stop_sequences: false,
            chunked_prefill: false,
            kv_compression: false,
        }
    }
}

/// Complete model capability profile — input to context budget allocator.
/// Constructed from baseline measurements (model_baseline.toml) or config.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelCapabilityProfile {
    pub vram_tier: VramTier,
    pub model_tier: ModelTier,
    pub model_name: String,
    pub model_size_b: f32,
    pub model_vram_mb: usize,
    pub kv_bytes_per_token: usize,
    pub in_vram_ceiling: usize,
    pub attention_ceiling: usize,
    /// min(in_vram_ceiling, attention_ceiling) — the real context limit.
    /// 0 means unmeasured; falls back to model_max_context.
    pub effective_ceiling: usize,
    pub recommended_prompt_budget: usize,
    pub output_reserve: usize,
    pub slot_weights: SlotWeights,
    /// Per-tier absolute token targets for fixed slots: [SystemPrompt, Rules, Telescope].
    /// Used by `build_dispatch_context` to replace the `[0, 0, 0]` default.
    /// From token-efficiency-design.txt section 2.
    pub fixed_slot_targets: [usize; 3],
    pub features: ModelFeatures,
    pub pass_at_1: f32,
    pub optimal_retries: usize,
    pub kv_quant_bits: u8,
    pub requires_fa2: bool,
    pub dola_viable: bool,
    pub steering_viable: bool,
    /// false for PROJECTED rows — numbers are estimates, not measurements.
    pub measured: bool,
}

impl ModelCapabilityProfile {
    /// Build a MICRO-tier profile from A-1 baseline measurements.
    /// Suitable for Qwen2.5-Coder-7B NF4 on 12GB (measured configuration).
    /// Per-tier allocation from TE design section 2:
    ///   12GB-7B (20K): System=330, Rules=400, Telescope=800,
    ///   RepoMap=1500, CodeContext=8000, SSB=1500, GRC=1000, Reserve=750.
    pub fn micro_7b_12gb() -> Self {
        // Dynamic proportions: 1500:8000:1500:1000 = total 12000
        Self {
            vram_tier: VramTier::Vram12Gb,
            model_tier: ModelTier::Micro,
            model_name: "Qwen2.5-Coder-7B-Instruct".to_string(),
            model_size_b: 7.61,
            model_vram_mb: 5180,
            kv_bytes_per_token: 57344,
            in_vram_ceiling: 25000,
            attention_ceiling: 25000,
            effective_ceiling: 25000,
            recommended_prompt_budget: 20000,
            output_reserve: 750,
            slot_weights: SlotWeights {
                system_prompt: 0.0,
                rules: 0.0,
                telescope: 0.0,
                repo_map: 0.125,
                code_context: 0.667,
                ssb_entries: 0.125,
                grc_history: 0.083,
            },
            fixed_slot_targets: [330, 400, 800],
            features: ModelFeatures::none(),
            pass_at_1: 0.65,
            optimal_retries: 3,
            kv_quant_bits: 0,
            requires_fa2: true,
            dola_viable: false,
            steering_viable: false,
            measured: true,
        }
    }

    /// Build a profile for 3B model on 12GB (measured configuration).
    /// Per-tier allocation from TE design section 2:
    ///   12GB-3B (35K): System=330, Rules=500, Telescope=1500,
    ///   RepoMap=3000, CodeContext=14000, SSB=3000, GRC=2000, Reserve=750.
    pub fn micro_3b_12gb() -> Self {
        // Dynamic proportions: 3000:14000:3000:2000 = total 22000
        Self {
            vram_tier: VramTier::Vram12Gb,
            model_tier: ModelTier::Micro,
            model_name: "Qwen2.5-Coder-3B-Instruct".to_string(),
            model_size_b: 3.0,
            model_vram_mb: 1920,
            kv_bytes_per_token: 1024,
            in_vram_ceiling: 40000,
            attention_ceiling: 40000,
            effective_ceiling: 40000,
            recommended_prompt_budget: 35000,
            output_reserve: 750,
            slot_weights: SlotWeights {
                system_prompt: 0.0,
                rules: 0.0,
                telescope: 0.0,
                repo_map: 0.136,
                code_context: 0.637,
                ssb_entries: 0.136,
                grc_history: 0.091,
            },
            fixed_slot_targets: [330, 500, 1500],
            features: ModelFeatures::none(),
            pass_at_1: 0.72,
            optimal_retries: 3,
            kv_quant_bits: 0,
            requires_fa2: true,
            dola_viable: false,
            steering_viable: false,
            measured: true,
        }
    }

    /// Build a projected LARGE-tier profile for 32B on 48GB.
    /// Per-tier allocation from TE design section 2:
    ///   48GB (30K): System=330, Rules=500, Telescope=1500,
    ///   RepoMap=3000, CodeContext=10000, SSB=3000, GRC=2000, Reserve=4000.
    pub fn large_32b_48gb() -> Self {
        // Dynamic proportions: 3000:10000:3000:2000 = total 18000
        Self {
            vram_tier: VramTier::Vram48Gb,
            model_tier: ModelTier::Large,
            model_name: "Qwen2.5-Coder-32B-Instruct".to_string(),
            model_size_b: 32.0,
            model_vram_mb: 20000,
            kv_bytes_per_token: 262144,
            in_vram_ceiling: 35000,
            attention_ceiling: 35000,
            effective_ceiling: 35000,
            recommended_prompt_budget: 30000,
            output_reserve: 4000,
            slot_weights: SlotWeights {
                system_prompt: 0.0,
                rules: 0.0,
                telescope: 0.0,
                repo_map: 0.167,
                code_context: 0.555,
                ssb_entries: 0.167,
                grc_history: 0.111,
            },
            fixed_slot_targets: [330, 500, 1500],
            features: ModelFeatures {
                supports_structured_output: false,
                supports_grammar: false,
                supports_stop_sequences: false,
                chunked_prefill: true,
                kv_compression: true,
            },
            pass_at_1: 0.927,
            optimal_retries: 2,
            kv_quant_bits: 4,
            requires_fa2: true,
            dola_viable: true,
            steering_viable: false,
            measured: false,
        }
    }

    /// Build a profile for small models on 8GB VRAM.
    /// Per-tier allocation from TE design section 2:
    ///   8GB (20K): System=200, Rules=300, Telescope=600,
    ///   RepoMap=1500, CodeContext=8000, SSB=1500, GRC=1000, Reserve=750.
    pub fn micro_8gb() -> Self {
        // Dynamic proportions: 1500:8000:1500:1000 = total 12000
        Self {
            vram_tier: VramTier::Vram8Gb,
            model_tier: ModelTier::Micro,
            model_name: "Qwen2.5-Coder-3B-Instruct".to_string(),
            model_size_b: 3.0,
            model_vram_mb: 1920,
            kv_bytes_per_token: 1024,
            in_vram_ceiling: 25000,
            attention_ceiling: 25000,
            effective_ceiling: 25000,
            recommended_prompt_budget: 20000,
            output_reserve: 750,
            slot_weights: SlotWeights {
                system_prompt: 0.0,
                rules: 0.0,
                telescope: 0.0,
                repo_map: 0.125,
                code_context: 0.667,
                ssb_entries: 0.125,
                grc_history: 0.083,
            },
            fixed_slot_targets: [200, 300, 600],
            features: ModelFeatures::none(),
            pass_at_1: 0.72,
            optimal_retries: 3,
            kv_quant_bits: 0,
            requires_fa2: false,
            dola_viable: false,
            steering_viable: false,
            measured: false,
        }
    }

    /// Resolve effective ceiling: min(in_vram, attention), or model_max_context if both 0.
    pub fn resolve_effective_ceiling(&self, model_max_context: usize) -> usize {
        match (self.in_vram_ceiling, self.attention_ceiling) {
            (0, 0) => model_max_context,
            (0, a) => a,
            (v, 0) => v,
            (v, a) => v.min(a),
        }
    }
}

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

/// Per-slot assembly behavior configuration produced by `slot_assembly_config()`.
/// Controls content trimming at dispatch time, independent of budget allocation.
/// Spec: token-efficiency-design.txt section 9.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotAssemblyConfig {
    /// Hard token ceiling for assembled slot content.
    pub max_tokens: usize,
    /// If true, skip filling this slot entirely (content = empty). Budget allocation unchanged.
    pub skip_content: bool,
    /// For list-type slots (SSB): maximum number of entries to include (last N, reverse-chronological).
    pub entry_limit: Option<usize>,
    /// For GRC history slot: include only the conclusion of the last CONVERGE step.
    pub conclusion_only: bool,
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

    /// Compute a context budget using a ModelCapabilityProfile for tier-adaptive allocation.
    ///
    /// Key differences from `compute()`:
    /// - Attention ceiling enforcement: effective context is capped to profile's effective_ceiling.
    /// - Slot weights from profile replace fixed PCT_ constants.
    /// - Output reserve from profile replaces caller-provided value.
    /// - Floor enforcement and zero-sum invariants are preserved.
    pub fn compute_with_profile(
        model_max_context: usize,
        fixed_slot_sizes: [usize; 3],
        profile: &ModelCapabilityProfile,
    ) -> Result<Self, BudgetError> {
        // Attention ceiling enforcement: fill to quality limit, not VRAM limit.
        let effective_max = profile.resolve_effective_ceiling(model_max_context).min(model_max_context);

        let effective_reserve = profile.output_reserve.max(FLOOR_OUTPUT_RESERVE);
        let fixed_total = fixed_slot_sizes[0] + fixed_slot_sizes[1] + fixed_slot_sizes[2];

        let dynamic_budget = effective_max
            .checked_sub(fixed_total)
            .and_then(|v| v.checked_sub(effective_reserve))
            .unwrap_or(0);

        if dynamic_budget < SUM_DYNAMIC_FLOORS {
            return Err(BudgetError::BudgetInfeasible {
                dynamic_budget,
                sum_floors: SUM_DYNAMIC_FLOORS,
            });
        }

        let weights = &profile.slot_weights;

        // Distribute dynamic budget by profile weights (f32 -> usize truncation).
        let mut repo_map_alloc = (dynamic_budget as f64 * weights.repo_map as f64) as usize;
        let mut code_ctx_alloc = (dynamic_budget as f64 * weights.code_context as f64) as usize;
        let mut ssb_alloc = (dynamic_budget as f64 * weights.ssb_entries as f64) as usize;
        let mut grc_alloc = (dynamic_budget as f64 * weights.grc_history as f64) as usize;

        // Assign remainder from float truncation to CodeContext (largest slot).
        let subtotal = repo_map_alloc + code_ctx_alloc + ssb_alloc + grc_alloc;
        code_ctx_alloc += dynamic_budget.saturating_sub(subtotal);

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

        let mut budget = Self::new(effective_max);

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
            budget.total_allocated, effective_max,
            "total_allocated must equal effective_max"
        );

        Ok(budget)
    }

    /// Build a fixed-size REPAIR_CONTEXT budget for compressed context replay.
    ///
    /// Used for retry attempts after failure. The 1400-token budget is a fresh,
    /// information-dense context assembly that replaces conversation history.
    /// Research: Kimi-Dev (arXiv:2509.23045), E-mem (arXiv:2601.21714).
    ///
    /// Slot mapping to repair context roles:
    ///   SystemPrompt  = role + output format (50 tokens)
    ///   Rules         = baby step spec (200 tokens)
    ///   PhaseSpec     = MistakeBook constraints (150 tokens)
    ///   RepoMap       = 0 (dropped — DKG provides targeted code)
    ///   CodeContext    = relevant code (DKG) + working example (800 tokens)
    ///   SsbEntries    = 0 (SSB history dropped — fresh start)
    ///   GrcHistory    = fix instruction from diagnosis layer (100 tokens)
    ///   OutputReserve = output reserve (100 tokens)
    pub fn compute_repair_context() -> Self {
        let mut budget = Self::new(REPAIR_CONTEXT_TOTAL);

        budget.slots[ContextSlot::SystemPrompt as usize].allocated_tokens = REPAIR_SLOT_SYSTEM_PROMPT;
        budget.slots[ContextSlot::Rules as usize].allocated_tokens = REPAIR_SLOT_BABY_STEP_SPEC;
        budget.slots[ContextSlot::PhaseSpec as usize].allocated_tokens = REPAIR_SLOT_MISTAKE_BOOK;
        budget.slots[ContextSlot::RepoMap as usize].allocated_tokens = 0;
        budget.slots[ContextSlot::CodeContext as usize].allocated_tokens =
            REPAIR_SLOT_RELEVANT_CODE + REPAIR_SLOT_WORKING_EXAMPLE;
        budget.slots[ContextSlot::SsbEntries as usize].allocated_tokens = 0;
        budget.slots[ContextSlot::GrcHistory as usize].allocated_tokens = REPAIR_SLOT_FIX_INSTRUCTION;
        budget.slots[ContextSlot::OutputReserve as usize].allocated_tokens = REPAIR_SLOT_OUTPUT_RESERVE;

        budget.total_allocated = REPAIR_CONTEXT_TOTAL;
        budget.total_used = 0;
        budget.adaptive_level = AdaptiveLevel::Normal;

        budget
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

// ---------------------------------------------------------------------------
// Constitutional CONSTRAINTS block (A-7: anti-staleness injection)
// ---------------------------------------------------------------------------

/// Maximum token budget for the CONSTRAINTS block within PhaseSpec content.
/// If total constraints exceed this, truncate to highest-severity entries.
/// This budget is part of the PhaseSpec fixed allocation, not additive.
const MAX_CONSTRAINT_TOKENS: usize = 200;

/// Approximate characters per token for constraint text estimation.
/// Conservative estimate (matches TOKEN_CHARS_PER_TOKEN used elsewhere).
const CONSTRAINT_CHARS_PER_TOKEN: usize = 4;

/// A single constraint entry for the CONSTRAINTS block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintEntry {
    pub deprecated_name: String,
    pub replacement_name: String,
}

/// Auto-generated CONSTRAINTS block injected into PhaseSpec content at assembly time.
///
/// Built from `deprecated_patterns WHERE severity = 'BLOCK'`.
/// Format (appended to PhaseSpec content before token counting):
/// ```text
/// CONSTRAINTS (auto-generated from rules.db, do not violate):
/// - DO NOT USE blake2. Use blake3.
/// - DO NOT USE rand 0.8. Use rand >= 0.9.
/// ```
///
/// See anti-staleness-enforcement.txt section 1.3 and WARN-7 resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintsBlock {
    entries: Vec<ConstraintEntry>,
    truncated: bool,
}

impl ConstraintsBlock {
    /// Build a ConstraintsBlock from deprecated pattern entries.
    ///
    /// Entries are expected to be pre-filtered to severity=BLOCK and sorted
    /// by name. If the formatted block exceeds [`MAX_CONSTRAINT_TOKENS`],
    /// entries are truncated from the end (lowest priority kept out).
    pub fn from_entries(entries: Vec<ConstraintEntry>) -> Self {
        if entries.is_empty() {
            return Self {
                entries: Vec::new(),
                truncated: false,
            };
        }

        // Check if all entries fit within budget.
        let header = "\nCONSTRAINTS (auto-generated from rules.db, do not violate):\n";
        let truncation_notice = "- [truncated: additional constraints omitted for token budget]\n";
        let header_tokens = header.len() / CONSTRAINT_CHARS_PER_TOKEN;
        let truncation_tokens = truncation_notice.len() / CONSTRAINT_CHARS_PER_TOKEN;

        let mut budget_remaining = MAX_CONSTRAINT_TOKENS.saturating_sub(header_tokens);
        let mut kept = Vec::new();
        let mut truncated = false;

        for (i, entry) in entries.iter().enumerate() {
            let line = format!(
                "- DO NOT USE {}. Use {}.\n",
                entry.deprecated_name, entry.replacement_name
            );
            let line_tokens = line.len() / CONSTRAINT_CHARS_PER_TOKEN;

            // Reserve space for truncation notice if more entries remain.
            let reserve = if i + 1 < entries.len() { truncation_tokens } else { 0 };
            if line_tokens + reserve > budget_remaining {
                truncated = true;
                break;
            }
            budget_remaining -= line_tokens;
            kept.push(entry.clone());
        }

        Self {
            entries: kept,
            truncated,
        }
    }

    /// Returns true if the block has no entries (nothing to inject).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns true if entries were truncated due to token budget.
    pub fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// Number of constraint entries in the block.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Estimated token count for the formatted block.
    pub fn estimated_tokens(&self) -> usize {
        if self.entries.is_empty() {
            return 0;
        }
        self.format().len() / CONSTRAINT_CHARS_PER_TOKEN
    }

    /// Format the CONSTRAINTS block as a string to append to PhaseSpec content.
    ///
    /// Returns an empty string if no entries exist.
    pub fn format(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }

        let mut out = String::from("\nCONSTRAINTS (auto-generated from rules.db, do not violate):\n");
        for entry in &self.entries {
            out.push_str(&format!(
                "- DO NOT USE {}. Use {}.\n",
                entry.deprecated_name, entry.replacement_name
            ));
        }
        if self.truncated {
            out.push_str("- [truncated: additional constraints omitted for token budget]\n");
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Per-slot assembly configuration (A-40: token-efficiency-design.txt section 9)
// ---------------------------------------------------------------------------

impl ContextBudget {
    /// Return assembly-time configuration for a given slot and model tier.
    ///
    /// This is a pure function — it does not read or modify `ContextBudget` fields.
    /// The assembler calls this to decide how aggressively to trim each slot's
    /// content before dispatch. Independent of budget allocation (fixed_slot_targets).
    pub fn slot_assembly_config(slot: ContextSlot, tier: ModelTier) -> SlotAssemblyConfig {
        match tier {
            ModelTier::Micro => match slot {
                ContextSlot::SystemPrompt => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_SYSTEM_PROMPT,
                    skip_content: false,
                    entry_limit: None,
                    conclusion_only: false,
                },
                ContextSlot::Rules => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_RULES,
                    skip_content: false,
                    entry_limit: None,
                    conclusion_only: false,
                },
                ContextSlot::PhaseSpec => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_TELESCOPE,
                    skip_content: false,
                    entry_limit: None,
                    conclusion_only: false,
                },
                ContextSlot::RepoMap => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_REPO_MAP,
                    skip_content: true,
                    entry_limit: None,
                    conclusion_only: false,
                },
                ContextSlot::CodeContext => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_CODE_CONTEXT,
                    skip_content: false,
                    entry_limit: None,
                    conclusion_only: false,
                },
                ContextSlot::SsbEntries => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_SSB_ENTRIES,
                    skip_content: false,
                    entry_limit: Some(2),
                    conclusion_only: false,
                },
                ContextSlot::GrcHistory => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_GRC_HISTORY,
                    skip_content: false,
                    entry_limit: None,
                    conclusion_only: true,
                },
                ContextSlot::OutputReserve => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_OUTPUT_RESERVE,
                    skip_content: false,
                    entry_limit: None,
                    conclusion_only: false,
                },
            },
            // Small, Medium, Large, Cloud all use LARGE_SLOT_* assembly targets.
            // Spec section 9 defines only MICRO and LARGE tiers for assembly.
            // Future phases may differentiate Small/Medium if needed.
            ModelTier::Small | ModelTier::Medium | ModelTier::Large | ModelTier::Cloud => {
                match slot {
                    ContextSlot::SystemPrompt => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_SYSTEM_PROMPT,
                        skip_content: false,
                        entry_limit: None,
                        conclusion_only: false,
                    },
                    ContextSlot::Rules => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_RULES,
                        skip_content: false,
                        entry_limit: None,
                        conclusion_only: false,
                    },
                    ContextSlot::PhaseSpec => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_TELESCOPE,
                        skip_content: false,
                        entry_limit: None,
                        conclusion_only: false,
                    },
                    ContextSlot::RepoMap => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_REPO_MAP,
                        skip_content: false,
                        entry_limit: None,
                        conclusion_only: false,
                    },
                    ContextSlot::CodeContext => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_CODE_CONTEXT,
                        skip_content: false,
                        entry_limit: None,
                        conclusion_only: false,
                    },
                    ContextSlot::SsbEntries => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_SSB_ENTRIES,
                        skip_content: false,
                        entry_limit: None,
                        conclusion_only: false,
                    },
                    ContextSlot::GrcHistory => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_GRC_HISTORY,
                        skip_content: false,
                        entry_limit: None,
                        conclusion_only: false,
                    },
                    ContextSlot::OutputReserve => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_OUTPUT_RESERVE,
                        skip_content: false,
                        entry_limit: None,
                        conclusion_only: false,
                    },
                }
            }
        }
    }
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

    // -----------------------------------------------------------------------
    // ConstraintsBlock tests (A-7)
    // -----------------------------------------------------------------------

    #[test]
    fn constraints_block_empty_entries() {
        let block = ConstraintsBlock::from_entries(vec![]);
        assert!(block.is_empty());
        assert!(!block.is_truncated());
        assert_eq!(block.len(), 0);
        assert_eq!(block.estimated_tokens(), 0);
        assert_eq!(block.format(), "");
    }

    #[test]
    fn constraints_block_single_entry() {
        let entries = vec![ConstraintEntry {
            deprecated_name: "blake2".to_string(),
            replacement_name: "blake3".to_string(),
        }];
        let block = ConstraintsBlock::from_entries(entries);
        assert!(!block.is_empty());
        assert_eq!(block.len(), 1);
        let formatted = block.format();
        assert!(formatted.contains("CONSTRAINTS"));
        assert!(formatted.contains("DO NOT USE blake2"));
        assert!(formatted.contains("Use blake3"));
    }

    #[test]
    fn constraints_block_multiple_entries() {
        let entries = vec![
            ConstraintEntry {
                deprecated_name: "blake2".to_string(),
                replacement_name: "blake3".to_string(),
            },
            ConstraintEntry {
                deprecated_name: "http.server".to_string(),
                replacement_name: "fastapi+uvicorn".to_string(),
            },
            ConstraintEntry {
                deprecated_name: "log".to_string(),
                replacement_name: "tracing".to_string(),
            },
        ];
        let block = ConstraintsBlock::from_entries(entries);
        assert_eq!(block.len(), 3);
        assert!(!block.is_truncated());
        let formatted = block.format();
        assert!(formatted.contains("DO NOT USE blake2"));
        assert!(formatted.contains("DO NOT USE http.server"));
        assert!(formatted.contains("DO NOT USE log"));
    }

    #[test]
    fn constraints_block_truncation_at_budget() {
        // Create many entries that will exceed 200-token budget.
        let entries: Vec<ConstraintEntry> = (0..100)
            .map(|i| ConstraintEntry {
                deprecated_name: format!("deprecated_package_{i:04}"),
                replacement_name: format!("replacement_package_{i:04}"),
            })
            .collect();
        let block = ConstraintsBlock::from_entries(entries);
        assert!(block.is_truncated());
        assert!(block.len() < 100);
        // Allow small rounding tolerance (integer division per-line vs total).
        let rounding_tolerance = 5;
        let est = block.estimated_tokens();
        assert!(
            est <= MAX_CONSTRAINT_TOKENS + rounding_tolerance,
            "estimated_tokens {est} > MAX {MAX_CONSTRAINT_TOKENS} + tolerance {rounding_tolerance}"
        );
        let formatted = block.format();
        assert!(formatted.contains("[truncated:"));
    }

    #[test]
    fn constraints_block_estimated_tokens_within_budget() {
        let entries = vec![
            ConstraintEntry {
                deprecated_name: "blake2".to_string(),
                replacement_name: "blake3".to_string(),
            },
            ConstraintEntry {
                deprecated_name: "rand 0.8".to_string(),
                replacement_name: "rand >= 0.9".to_string(),
            },
        ];
        let block = ConstraintsBlock::from_entries(entries);
        assert!(block.estimated_tokens() <= MAX_CONSTRAINT_TOKENS);
        assert!(block.estimated_tokens() > 0);
    }

    // -----------------------------------------------------------------------
    // A-10: ModelCapabilityProfile and compute_with_profile tests
    // -----------------------------------------------------------------------

    #[test]
    fn slot_weights_default_sums_to_one() {
        let w = SlotWeights::default_fixed();
        assert!(w.validate_dynamic_sum(), "default weights must sum to 1.0");
    }

    #[test]
    fn slot_weights_invalid_sum_detected() {
        let w = SlotWeights {
            system_prompt: 0.0,
            rules: 0.0,
            telescope: 0.0,
            repo_map: 0.50,
            code_context: 0.50,
            ssb_entries: 0.25,
            grc_history: 0.20,
        };
        assert!(!w.validate_dynamic_sum(), "weights summing to 1.45 must fail");
    }

    #[test]
    fn micro_7b_profile_has_valid_weights() {
        let p = ModelCapabilityProfile::micro_7b_12gb();
        assert!(p.slot_weights.validate_dynamic_sum());
        assert_eq!(p.model_tier, ModelTier::Micro);
        assert_eq!(p.vram_tier, VramTier::Vram12Gb);
        assert!(p.measured);
    }

    #[test]
    fn micro_3b_profile_has_valid_weights() {
        let p = ModelCapabilityProfile::micro_3b_12gb();
        assert!(p.slot_weights.validate_dynamic_sum());
        assert_eq!(p.effective_ceiling, 40000);
    }

    #[test]
    fn large_32b_profile_has_valid_weights() {
        let p = ModelCapabilityProfile::large_32b_48gb();
        assert!(p.slot_weights.validate_dynamic_sum());
        assert!(!p.measured);
        assert!(p.dola_viable);
    }

    #[test]
    fn micro_8gb_profile_has_valid_weights() {
        let p = ModelCapabilityProfile::micro_8gb();
        assert!(p.slot_weights.validate_dynamic_sum());
        assert_eq!(p.vram_tier, VramTier::Vram8Gb);
        assert_eq!(p.model_tier, ModelTier::Micro);
        assert_eq!(p.fixed_slot_targets, [200, 300, 600]);
    }

    #[test]
    fn per_tier_output_reserve_micro_750() {
        let p7b = ModelCapabilityProfile::micro_7b_12gb();
        assert_eq!(p7b.output_reserve, 750, "MICRO 7B output_reserve must be 750");
        let p3b = ModelCapabilityProfile::micro_3b_12gb();
        assert_eq!(p3b.output_reserve, 750, "MICRO 3B output_reserve must be 750");
        let p8g = ModelCapabilityProfile::micro_8gb();
        assert_eq!(p8g.output_reserve, 750, "MICRO 8GB output_reserve must be 750");
    }

    #[test]
    fn per_tier_output_reserve_large_4000() {
        let p = ModelCapabilityProfile::large_32b_48gb();
        assert_eq!(p.output_reserve, 4000, "LARGE output_reserve must be 4000");
    }

    #[test]
    fn per_tier_fixed_slot_targets_nonzero() {
        // All profiles must have non-zero fixed slot targets.
        for (name, p) in [
            ("micro_7b", ModelCapabilityProfile::micro_7b_12gb()),
            ("micro_3b", ModelCapabilityProfile::micro_3b_12gb()),
            ("micro_8gb", ModelCapabilityProfile::micro_8gb()),
            ("large_32b", ModelCapabilityProfile::large_32b_48gb()),
        ] {
            assert!(
                p.fixed_slot_targets.iter().all(|&v| v > 0),
                "{name} must have non-zero fixed_slot_targets: {:?}",
                p.fixed_slot_targets,
            );
        }
    }

    #[test]
    fn compute_with_profile_uses_fixed_slot_targets() {
        let p = ModelCapabilityProfile::micro_7b_12gb();
        let budget = ContextBudget::compute_with_profile(
            25000,
            p.fixed_slot_targets,
            &p,
        ).unwrap();
        // Fixed slots should match profile targets.
        assert_eq!(budget.slots[0].allocated_tokens, 330, "SystemPrompt");
        assert_eq!(budget.slots[1].allocated_tokens, 400, "Rules");
        assert_eq!(budget.slots[2].allocated_tokens, 800, "Telescope");
    }

    #[test]
    fn resolve_effective_ceiling_both_nonzero() {
        let mut p = ModelCapabilityProfile::micro_7b_12gb();
        p.in_vram_ceiling = 25000;
        p.attention_ceiling = 30000;
        assert_eq!(p.resolve_effective_ceiling(131072), 25000);
    }

    #[test]
    fn resolve_effective_ceiling_both_zero_fallback() {
        let mut p = ModelCapabilityProfile::micro_7b_12gb();
        p.in_vram_ceiling = 0;
        p.attention_ceiling = 0;
        assert_eq!(p.resolve_effective_ceiling(131072), 131072);
    }

    #[test]
    fn resolve_effective_ceiling_one_zero() {
        let mut p = ModelCapabilityProfile::micro_7b_12gb();
        p.in_vram_ceiling = 0;
        p.attention_ceiling = 16000;
        assert_eq!(p.resolve_effective_ceiling(131072), 16000);
    }

    #[test]
    fn compute_with_profile_matches_compute_for_default_weights() {
        let profile = ModelCapabilityProfile {
            vram_tier: VramTier::Cloud,
            model_tier: ModelTier::Cloud,
            model_name: "test".to_string(),
            model_size_b: 0.0,
            model_vram_mb: 0,
            kv_bytes_per_token: 0,
            in_vram_ceiling: 0,
            attention_ceiling: 0,
            effective_ceiling: 0,
            recommended_prompt_budget: 0,
            output_reserve: 2000,
            slot_weights: SlotWeights::default_fixed(),
            fixed_slot_targets: [100, 200, 300],
            features: ModelFeatures::none(),
            pass_at_1: 0.0,
            optimal_retries: 0,
            kv_quant_bits: 0,
            requires_fa2: false,
            dola_viable: false,
            steering_viable: false,
            measured: false,
        };
        // With both ceilings=0 and default weights, should match compute() output.
        let b1 = ContextBudget::compute(8192, [100, 200, 300], 2000).unwrap();
        let b2 = ContextBudget::compute_with_profile(8192, [100, 200, 300], &profile).unwrap();
        assert_eq!(b1.slots[3].allocated_tokens, b2.slots[3].allocated_tokens);
        assert_eq!(b1.slots[4].allocated_tokens, b2.slots[4].allocated_tokens);
        assert_eq!(b1.slots[5].allocated_tokens, b2.slots[5].allocated_tokens);
        assert_eq!(b1.slots[6].allocated_tokens, b2.slots[6].allocated_tokens);
        assert_eq!(b1.total_allocated, b2.total_allocated);
    }

    #[test]
    fn compute_with_profile_enforces_attention_ceiling() {
        let profile = ModelCapabilityProfile::micro_7b_12gb();
        // model_max_context=131072 but profile.effective_ceiling=25000
        let budget = ContextBudget::compute_with_profile(131072, [100, 200, 300], &profile).unwrap();
        // Budget should be capped to 25000, not 131072.
        assert_eq!(budget.model_max_context, 25000);
        assert_eq!(budget.total_allocated, 25000);
    }

    #[test]
    fn compute_with_profile_uses_profile_output_reserve() {
        let profile = ModelCapabilityProfile::micro_7b_12gb();
        // profile.output_reserve=750, but floor is 2000, so should clamp to 2000.
        let budget = ContextBudget::compute_with_profile(25000, [100, 200, 300], &profile).unwrap();
        assert_eq!(budget.slots[7].allocated_tokens, FLOOR_OUTPUT_RESERVE);
    }

    #[test]
    fn compute_with_profile_floor_enforcement() {
        // Tight budget that forces floor stealing.
        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        profile.in_vram_ceiling = 4000;
        profile.attention_ceiling = 4000;
        profile.effective_ceiling = 4000;
        let budget = ContextBudget::compute_with_profile(4000, [100, 200, 300], &profile).unwrap();
        assert!(budget.slots[6].allocated_tokens >= FLOOR_GRC_HISTORY);
        assert!(budget.slots[5].allocated_tokens >= FLOOR_SSB_ENTRIES);
        assert_eq!(budget.total_allocated, 4000);
    }

    #[test]
    fn compute_with_profile_infeasible_when_ceiling_too_small() {
        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        profile.in_vram_ceiling = 3000;
        profile.attention_ceiling = 3000;
        profile.effective_ceiling = 3000;
        let result = ContextBudget::compute_with_profile(131072, [100, 200, 300], &profile);
        assert!(result.is_err());
    }

    #[test]
    fn compute_with_profile_custom_weights_shift_allocation() {
        // Heavy code_context weight (60%) vs default 40%.
        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        profile.slot_weights = SlotWeights {
            system_prompt: 0.0,
            rules: 0.0,
            telescope: 0.0,
            repo_map: 0.10,
            code_context: 0.60,
            ssb_entries: 0.15,
            grc_history: 0.15,
        };
        profile.in_vram_ceiling = 0;
        profile.attention_ceiling = 0;
        profile.effective_ceiling = 0;
        let budget = ContextBudget::compute_with_profile(20000, [100, 200, 300], &profile).unwrap();
        let default_budget = ContextBudget::compute(20000, [100, 200, 300], 2000).unwrap();
        // With 60% weight, code_context should be larger than default 40%.
        assert!(budget.slots[4].allocated_tokens > default_budget.slots[4].allocated_tokens);
        // RepoMap with 10% should be smaller than default 15%.
        assert!(budget.slots[3].allocated_tokens < default_budget.slots[3].allocated_tokens);
    }

    #[test]
    fn compute_with_profile_zero_sum_invariant_100_configs() {
        let profile = ModelCapabilityProfile::micro_7b_12gb();
        for max_ctx in (5000..=50000).step_by(500) {
            let mut p = profile.clone();
            p.in_vram_ceiling = max_ctx;
            p.attention_ceiling = max_ctx;
            p.effective_ceiling = max_ctx;
            if let Ok(budget) = ContextBudget::compute_with_profile(max_ctx, [100, 200, 300], &p) {
                let dynamic_sum = budget.slots[3].allocated_tokens
                    + budget.slots[4].allocated_tokens
                    + budget.slots[5].allocated_tokens
                    + budget.slots[6].allocated_tokens;
                let expected_dynamic = max_ctx - 600 - budget.slots[7].allocated_tokens;
                assert_eq!(dynamic_sum, expected_dynamic, "zero-sum violated at max_ctx={max_ctx}");
                assert_eq!(budget.total_allocated, max_ctx, "total != max at max_ctx={max_ctx}");
            }
        }
    }

    #[test]
    fn model_features_none_all_false() {
        let f = ModelFeatures::none();
        assert!(!f.supports_structured_output);
        assert!(!f.supports_grammar);
        assert!(!f.supports_stop_sequences);
        assert!(!f.chunked_prefill);
        assert!(!f.kv_compression);
    }

    // -----------------------------------------------------------------------
    // A-18: GenerationProfile tests
    // -----------------------------------------------------------------------

    #[test]
    fn first_attempt_profile_params() {
        let p = GenerationProfile::FirstAttempt;
        assert!((p.temperature() - 0.3).abs() < f32::EPSILON);
        assert!((p.top_p() - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn repair_retry_profile_params() {
        let p = GenerationProfile::RepairRetry;
        assert!((p.temperature() - 1.0).abs() < f32::EPSILON);
        assert!((p.top_p() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn interrogation_profile_params() {
        let p = GenerationProfile::Interrogation;
        assert!((p.temperature() - 0.1).abs() < f32::EPSILON);
        assert!((p.top_p() - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn micro_reflection_profile_params() {
        let p = GenerationProfile::MicroReflection;
        assert!((p.temperature() - 0.3).abs() < f32::EPSILON);
        assert!((p.top_p() - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn tree_branch_profiles_distinct_temperatures() {
        let a = GenerationProfile::TreeBranchA;
        let b = GenerationProfile::TreeBranchB;
        let c = GenerationProfile::TreeBranchC;
        // A=0.7, B=1.0, C=0.3 — all distinct.
        assert!((a.temperature() - 0.7).abs() < f32::EPSILON);
        assert!((b.temperature() - 1.0).abs() < f32::EPSILON);
        assert!((c.temperature() - 0.3).abs() < f32::EPSILON);
        // Top-p: A=1.0, B=1.0, C=0.9.
        assert!((a.top_p() - 1.0).abs() < f32::EPSILON);
        assert!((b.top_p() - 1.0).abs() < f32::EPSILON);
        assert!((c.top_p() - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn all_profiles_temperature_in_valid_range() {
        let profiles = [
            GenerationProfile::FirstAttempt,
            GenerationProfile::MicroReflection,
            GenerationProfile::Interrogation,
            GenerationProfile::RepairRetry,
            GenerationProfile::TreeBranchA,
            GenerationProfile::TreeBranchB,
            GenerationProfile::TreeBranchC,
        ];
        for p in &profiles {
            assert!(p.temperature() >= 0.0 && p.temperature() <= 1.0,
                "{:?} temperature {} out of [0.0, 1.0]", p, p.temperature());
            assert!(p.top_p() > 0.0 && p.top_p() <= 1.0,
                "{:?} top_p {} out of (0.0, 1.0]", p, p.top_p());
        }
    }

    // -----------------------------------------------------------------------
    // A-18: REPAIR_CONTEXT SlotWeights and compute_repair_context tests
    // -----------------------------------------------------------------------

    #[test]
    fn repair_context_weights_sum_to_one() {
        let w = SlotWeights::repair_context();
        assert!(w.validate_dynamic_sum(),
            "repair_context dynamic weights must sum to 1.0, got {}",
            w.repo_map + w.code_context + w.ssb_entries + w.grc_history);
    }

    #[test]
    fn repair_context_weights_drop_repo_map_and_ssb() {
        let w = SlotWeights::repair_context();
        assert!((w.repo_map - 0.0).abs() < f32::EPSILON, "repo_map must be 0 in repair context");
        assert!((w.ssb_entries - 0.0).abs() < f32::EPSILON, "ssb_entries must be 0 in repair context");
    }

    #[test]
    fn repair_context_weights_prioritize_code_context() {
        let w = SlotWeights::repair_context();
        let default_w = SlotWeights::default_fixed();
        assert!(w.code_context > default_w.code_context,
            "repair code_context {} must exceed default {}",
            w.code_context, default_w.code_context);
    }

    #[test]
    fn compute_repair_context_total_1400() {
        let budget = ContextBudget::compute_repair_context();
        assert_eq!(budget.model_max_context, REPAIR_CONTEXT_TOTAL);
        assert_eq!(budget.total_allocated, REPAIR_CONTEXT_TOTAL);
    }

    #[test]
    fn compute_repair_context_slot_allocations() {
        let budget = ContextBudget::compute_repair_context();
        assert_eq!(budget.slots[ContextSlot::SystemPrompt as usize].allocated_tokens, 50);
        assert_eq!(budget.slots[ContextSlot::Rules as usize].allocated_tokens, 200);
        assert_eq!(budget.slots[ContextSlot::PhaseSpec as usize].allocated_tokens, 150);
        assert_eq!(budget.slots[ContextSlot::RepoMap as usize].allocated_tokens, 0);
        assert_eq!(budget.slots[ContextSlot::CodeContext as usize].allocated_tokens, 800);
        assert_eq!(budget.slots[ContextSlot::SsbEntries as usize].allocated_tokens, 0);
        assert_eq!(budget.slots[ContextSlot::GrcHistory as usize].allocated_tokens, 100);
        assert_eq!(budget.slots[ContextSlot::OutputReserve as usize].allocated_tokens, 100);
    }

    #[test]
    fn compute_repair_context_slots_sum_to_total() {
        let budget = ContextBudget::compute_repair_context();
        let sum: usize = budget.slots.iter().map(|s| s.allocated_tokens).sum();
        assert_eq!(sum, REPAIR_CONTEXT_TOTAL,
            "slot allocations sum {} must equal REPAIR_CONTEXT_TOTAL {}",
            sum, REPAIR_CONTEXT_TOTAL);
    }

    #[test]
    fn compute_repair_context_is_normal_level() {
        let budget = ContextBudget::compute_repair_context();
        assert_eq!(budget.adaptive_level, AdaptiveLevel::Normal);
    }

    #[test]
    fn repair_context_much_smaller_than_first_attempt() {
        let repair = ContextBudget::compute_repair_context();
        let first = ContextBudget::compute(25000, [100, 200, 300], 2000).unwrap();
        assert!(repair.total_allocated < first.total_allocated,
            "repair budget {} must be smaller than first-attempt {}",
            repair.total_allocated, first.total_allocated);
    }

    #[test]
    fn repair_context_no_repo_map_or_ssb() {
        let budget = ContextBudget::compute_repair_context();
        assert_eq!(budget.slots[ContextSlot::RepoMap as usize].allocated_tokens, 0,
            "repair context must drop RepoMap");
        assert_eq!(budget.slots[ContextSlot::SsbEntries as usize].allocated_tokens, 0,
            "repair context must drop SsbEntries");
    }
}
