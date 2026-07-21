use std::fmt;
use std::sync::Arc;

use blake3;

use crate::ssb::{
    ContextBudgetComputedPayload, ContextReductionPayload, Ssb, SsbFilteredByOverlapPayload,
    SsbPayload,
};

// H-31 (SCHEMA_VERSION 240): bumped 8 -> 9 for the new `ContextSlot::Memory` slot -- the
// user-preference bi-temporal ledger retrieval-into-prompt slot. Memory = 8 is APPENDED after
// OutputReserve = 7 (not inserted before it) so every existing slot ordinal stays stable -- the
// `repr(u8)` ordinal contract that every `slot as usize` index relies on, and the
// `OutputReserve as u8 == 7` assert, are preserved. The slot is bounded to a small token target
// (k <= 8 user-decisions, one line each) and is populated ONLY for the role allowlist
// {AgentRole::Executor} AND ELAI_USER_MEMORY_ENABLE=1 (the Personalization-Trap firewall lives in
// `crate::user_memory_firewall`).
// H-53 (SCHEMA_VERSION 229): bumped 9 -> 10 for the new ContextSlot::Evidence slot.
// The Evidence slot holds the pre-bound EvidenceAnchorSet serialized block and
// MUST land AFTER the last cache breakpoint (after_slot = CodeContext) per HD-2.
// I-6: bumped 10 -> 11 for the append-only ContextSlot::AnchorPane slot.
pub const SLOT_COUNT: usize = 11;
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
// H-31: user-memory slot is RETRIEVAL-ON-DEMAND, never an always-injected block. Bounded small
// (k <= 8 user-decisions, one line each) so it cannot crowd out the local ~25-32K window.
pub const MICRO_SLOT_MEMORY: usize = 300;

pub const LARGE_SLOT_SYSTEM_PROMPT: usize = 330;
pub const LARGE_SLOT_RULES: usize = 500;
pub const LARGE_SLOT_TELESCOPE: usize = 1000;
pub const LARGE_SLOT_REPO_MAP: usize = 2000;
pub const LARGE_SLOT_CODE_CONTEXT: usize = 6000;
pub const LARGE_SLOT_SSB_ENTRIES: usize = 1500;
pub const LARGE_SLOT_GRC_HISTORY: usize = 1000;
pub const LARGE_SLOT_OUTPUT_RESERVE: usize = 4000;
// H-31: user-memory slot target on larger tiers (still small; one line per surfaced decision).
pub const LARGE_SLOT_MEMORY: usize = 500;
// H-53: evidence slot holds the pre-bound ranked evidence excerpts (freeze_anchors input).
// Sits after the last cache breakpoint (HD-2: must be after CodeContext=4). Bounded to a few
// sources: Micro keeps it small (3-5 excerpts x ~80 tokens each), Large allows more (up to 10).
pub const MICRO_SLOT_EVIDENCE: usize = 400;
pub const LARGE_SLOT_EVIDENCE: usize = 1000;
pub const MICRO_SLOT_ANCHOR_PANE: usize = 300;
pub const LARGE_SLOT_ANCHOR_PANE: usize = 500;

/// Fixed Slot 3 (PhaseSpec/Telescope) target across all model tiers.
/// Updated in A-46 to accommodate telescope content (~550-1050 tokens).
pub const FIXED_SLOT_TELESCOPE_TARGET: usize = 1050;

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
    /// B-28 E-1: native Fill-In-the-Middle support via `<|fim_prefix|>` /
    /// `<|fim_suffix|>` / `<|fim_middle|>` sentinel tokens. true for Qwen2.5-Coder
    /// (and StarCoder / DeepSeek-Coder by extension); false for all cloud models
    /// and any sidecar model whose tokenizer does not register the FIM specials.
    /// Queried at registration time, never at runtime — see
    /// `workflow/papers/focus-tunnel-design-interface.md` L21-24.
    pub supports_fim: bool,
    /// J-36 FH-4: architecture admits local PrefixKVCache reuse (standard causal full-attention). Runtime TurboQuant still gates via prefix_kv_eligible().
    pub prefix_kv_eligible: bool,
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
            supports_fim: false,
            prefix_kv_eligible: false,
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
    pub quant_format: String,
    /// J-34: measured fill-degradation curve (verified-outcome rate vs
    /// context-window fill). `None` = no calibration artifact for this
    /// model; the runtime falls back to `CLOUD_TRIM_FRACTION` / the env
    /// constant and LOGS that fallback (no silent pseudo-calibration).
    pub fill_degradation_curve: Option<crate::context_calibration::FillDegradationCurve>,
    /// J-34: measured positional-value curve (constraint recall vs depth).
    /// `None` = no calibration artifact; the 3E slot allocator uses today's
    /// fixed slot order.
    pub positional_value_curve: Option<crate::context_calibration::PositionalValueCurve>,
    /// J-34: calibrated trim fraction derived from `fill_degradation_curve`'s
    /// knee. `None` = no knee (flat curve) or no artifact; `trim_threshold`
    /// (context_policy.rs:223) falls back to the folklore fraction.
    pub calibrated_trim_fraction: Option<f64>,
    /// J-34 leg-3: deterministic ordering seed derived from
    /// `positional_value_curve` by `apply_to_profile`. `None` = no curve;
    /// `compute_with_profile` carries it onto `ContextBudget` and emits a
    /// `PlacementApplied` SSB row. Metadata-only, does NOT reorder `slots`.
    pub ordering_seed: Option<String>,
}

impl ModelCapabilityProfile {
    /// Build a MICRO-tier profile from A-1 baseline measurements.
    /// Suitable for Qwen2.5-Coder-7B NF4 on 12GB (measured configuration).
    /// Per-tier allocation from token-efficiency-design.txt §2 table (lines 516-526):
    ///   12GB-7B (20K): System=330, Rules=400, Telescope=800,
    ///   RepoMap=1500, CodeContext=8000, SSB=1500, GRC=1000, Reserve=512.
    /// B-10 (DECISION-D): fixed_slot_targets and output_reserve corrected to spec.
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
            output_reserve: 512,
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
            features: ModelFeatures {
                supports_fim: true,
                prefix_kv_eligible: true,
                ..ModelFeatures::none()
            },
            pass_at_1: 0.65,
            optimal_retries: 3,
            kv_quant_bits: 0,
            requires_fa2: true,
            dola_viable: false,
            steering_viable: false,
            measured: true,
            quant_format: "NF4+FA2".to_string(),
            fill_degradation_curve: None,
            positional_value_curve: None,
            calibrated_trim_fraction: None,
            ordering_seed: None,
        }
    }

    /// Build a profile for 3B model on 12GB (measured configuration).
    /// Per-tier allocation from token-efficiency-design.txt §2 table (lines 516-526):
    ///   12GB-3B (35K): System=330, Rules=500, Telescope=1500,
    ///   RepoMap=3000, CodeContext=14000, SSB=3000, GRC=2000, Reserve=512.
    /// B-10 (DECISION-D): fixed_slot_targets and output_reserve corrected to spec.
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
            output_reserve: 512,
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
            features: ModelFeatures {
                supports_fim: true,
                prefix_kv_eligible: true,
                ..ModelFeatures::none()
            },
            pass_at_1: 0.72,
            optimal_retries: 3,
            kv_quant_bits: 0,
            requires_fa2: true,
            dola_viable: false,
            steering_viable: false,
            measured: true,
            quant_format: "NF4+FA2".to_string(),
            fill_degradation_curve: None,
            positional_value_curve: None,
            calibrated_trim_fraction: None,
            ordering_seed: None,
        }
    }

    /// B-27: cloud-mode profile for the Anthropic 200K context window.
    /// Used when the user routes a task through cloud_mode.rs (ExecMode::Cloud
    /// or escalated from ExecMode::Auto). Spec D-2 carves a 200K window with
    /// most slots scaled up roughly 5× from the 48GB tier; output reserve is
    /// raised to 8K (vs 512 local) since cloud calls produce longer responses.
    /// Slot weights chosen to favor code_context for deep file dives in a
    /// large window — repo_map=0.15, code_context=0.60, ssb_entries=0.15,
    /// grc_history=0.10 (sums to 1.0, validates against
    /// [`SlotWeights::validate_dynamic_sum`]).
    pub fn cloud_200k() -> Self {
        Self {
            vram_tier: VramTier::Vram48Gb,
            model_tier: ModelTier::Cloud,
            model_name: "claude-sonnet-4-6".to_string(),
            model_size_b: 0.0,
            model_vram_mb: 0,
            kv_bytes_per_token: 0,
            in_vram_ceiling: 0,
            attention_ceiling: 200_000,
            effective_ceiling: 200_000,
            recommended_prompt_budget: 192_000,
            output_reserve: 8_000,
            slot_weights: SlotWeights {
                system_prompt: 0.0,
                rules: 0.0,
                telescope: 0.0,
                repo_map: 0.15,
                code_context: 0.60,
                ssb_entries: 0.15,
                grc_history: 0.10,
            },
            fixed_slot_targets: [1000, 2000, 5000],
            features: ModelFeatures {
                supports_structured_output: true,
                supports_grammar: false,
                supports_stop_sequences: true,
                chunked_prefill: false,
                kv_compression: false,
                supports_fim: false,
                prefix_kv_eligible: false,
            },
            pass_at_1: 0.85,
            optimal_retries: 2,
            kv_quant_bits: 0,
            requires_fa2: false,
            dola_viable: false,
            steering_viable: false,
            measured: false,
            quant_format: "FP16".to_string(),
            fill_degradation_curve: None,
            positional_value_curve: None,
            calibrated_trim_fraction: None,
            ordering_seed: None,
        }
    }

    /// Build a projected LARGE-tier profile for 32B on 48GB.
    /// Per-tier allocation from token-efficiency-design.txt §2 table (lines 516-526):
    ///   48GB (30K): System=330, Rules=500, Telescope=1500,
    ///   RepoMap=3000, CodeContext=10000, SSB=3000, GRC=2000, Reserve=512.
    /// B-10 (DECISION-D): fixed_slot_targets and output_reserve corrected to spec.
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
            output_reserve: 512,
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
                supports_fim: true,
                prefix_kv_eligible: true,
            },
            pass_at_1: 0.927,
            optimal_retries: 2,
            kv_quant_bits: 4,
            requires_fa2: true,
            dola_viable: true,
            steering_viable: false,
            measured: false,
            quant_format: "NF4+FA2".to_string(),
            fill_degradation_curve: None,
            positional_value_curve: None,
            calibrated_trim_fraction: None,
            ordering_seed: None,
        }
    }

    /// Build a profile for small models on 8GB VRAM.
    /// Per-tier allocation from token-efficiency-design.txt §2 table (lines 516-526):
    ///   8GB (20K): System=200, Rules=300, Telescope=600,
    ///   RepoMap=1500, CodeContext=8000, SSB=1500, GRC=1000, Reserve=512.
    /// B-10 (DECISION-D): fixed_slot_targets and output_reserve corrected to spec.
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
            output_reserve: 512,
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
            features: ModelFeatures {
                supports_fim: true,
                prefix_kv_eligible: true,
                ..ModelFeatures::none()
            },
            pass_at_1: 0.72,
            optimal_retries: 3,
            kv_quant_bits: 0,
            requires_fa2: false,
            dola_viable: false,
            steering_viable: false,
            measured: false,
            quant_format: "NF4+FA2".to_string(),
            fill_degradation_curve: None,
            positional_value_curve: None,
            calibrated_trim_fraction: None,
            ordering_seed: None,
        }
    }


    /// E-27: AWQ-Marlin + FP8 KV profile for 7B on 12GB VRAM.
    /// Projected: AWQ W4A16 reduces model VRAM to ~4200 MB (vs 5180 MB NF4),
    /// enabling a larger in-VRAM context ceiling with FP8 KV software-sim.
    /// Throughput uplift: ~3-4x over NF4 per Jarvislabs 2026 Qwen2.5-32B benchmark
    /// extrapolated to 7B (Open Question 1 in E-27 plan; measured after rollout).
    pub fn awq_7b_12gb() -> Self {
        Self {
            vram_tier: VramTier::Vram12Gb,
            model_tier: ModelTier::Micro,
            model_name: "Qwen2.5-Coder-7B-Instruct".to_string(),
            model_size_b: 7.61,
            model_vram_mb: 4200,
            kv_bytes_per_token: 28672,
            in_vram_ceiling: 30000,
            attention_ceiling: 30000,
            effective_ceiling: 30000,
            recommended_prompt_budget: 25000,
            output_reserve: 512,
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
            features: ModelFeatures {
                supports_fim: true,
                prefix_kv_eligible: true,
                ..ModelFeatures::none()
            },
            pass_at_1: 0.65,
            optimal_retries: 3,
            kv_quant_bits: 8,
            requires_fa2: true,
            dola_viable: false,
            steering_viable: false,
            measured: false,
            quant_format: "AWQ-Marlin+FA2+FP8KV".to_string(),
            fill_degradation_curve: None,
            positional_value_curve: None,
            calibrated_trim_fraction: None,
            ordering_seed: None,
        }
    }

    /// E-27: AWQ-Marlin + FP8 KV profile for 3B on 12GB VRAM.
    /// Projected: AWQ 3B at ~1500 MB model VRAM; FP8 KV software-sim on Ampere.
    pub fn awq_3b_12gb() -> Self {
        Self {
            vram_tier: VramTier::Vram12Gb,
            model_tier: ModelTier::Micro,
            model_name: "Qwen2.5-Coder-3B-Instruct".to_string(),
            model_size_b: 3.0,
            model_vram_mb: 1500,
            kv_bytes_per_token: 512,
            in_vram_ceiling: 45000,
            attention_ceiling: 45000,
            effective_ceiling: 45000,
            recommended_prompt_budget: 40000,
            output_reserve: 512,
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
            features: ModelFeatures {
                supports_fim: true,
                prefix_kv_eligible: true,
                ..ModelFeatures::none()
            },
            pass_at_1: 0.72,
            optimal_retries: 3,
            kv_quant_bits: 8,
            requires_fa2: true,
            dola_viable: false,
            steering_viable: false,
            measured: false,
            quant_format: "AWQ-Marlin+FA2+FP8KV".to_string(),
            fill_degradation_curve: None,
            positional_value_curve: None,
            calibrated_trim_fraction: None,
            ordering_seed: None,
        }
    }

    /// E-27: AWQ-Marlin + FP8 KV profile for 3B on 8GB VRAM.
    /// Projected: AWQ 3B at ~1500 MB model VRAM in a tighter VRAM budget.
    pub fn awq_3b_8gb() -> Self {
        Self {
            vram_tier: VramTier::Vram8Gb,
            model_tier: ModelTier::Micro,
            model_name: "Qwen2.5-Coder-3B-Instruct".to_string(),
            model_size_b: 3.0,
            model_vram_mb: 1500,
            kv_bytes_per_token: 512,
            in_vram_ceiling: 28000,
            attention_ceiling: 28000,
            effective_ceiling: 28000,
            recommended_prompt_budget: 22000,
            output_reserve: 512,
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
            features: ModelFeatures {
                supports_fim: true,
                prefix_kv_eligible: true,
                ..ModelFeatures::none()
            },
            pass_at_1: 0.72,
            optimal_retries: 3,
            kv_quant_bits: 8,
            requires_fa2: false,
            dola_viable: false,
            steering_viable: false,
            measured: false,
            quant_format: "AWQ-Marlin+FA2+FP8KV".to_string(),
            fill_degradation_curve: None,
            positional_value_curve: None,
            calibrated_trim_fraction: None,
            ordering_seed: None,
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

/// J-36 FH-4: runtime gate for PrefixKVCache reuse. Eligible only when the
/// model architecture admits it (standard causal full-attention, set on
/// `ModelFeatures.prefix_kv_eligible`) AND TurboQuant is OFF at runtime.
pub fn prefix_kv_eligible(profile: &ModelCapabilityProfile, turboquant_on: bool) -> bool {
    profile.features.prefix_kv_eligible && !turboquant_on
}

// ---------------------------------------------------------------------------
// B-10 (DECISION-F) — model -> ModelCapabilityProfile detection
// ---------------------------------------------------------------------------

/// Map a loaded model identifier + VRAM tier to the most appropriate
/// [`ModelCapabilityProfile`].
///
/// Substring matching on the model name (case-insensitive). Order matters:
/// "32B" is tested before "3B" so the 32B path doesn't accidentally hit the
/// 3B fallback. Unknown model names fall back to `micro_7b_12gb` with a
/// `tracing::warn` (the safest 12GB-class default).
///
/// B-10 (DECISION-F): tier detection happens once at TaskRunner bootstrap
/// and is cached on `ResolvedConfig.model_capability_profile`.
pub fn detect_model_capability_profile(
    model_name: &str,
    vram_tier: VramTier,
) -> ModelCapabilityProfile {
    let name_lower = model_name.to_ascii_lowercase();

    // Order matters: "32b" must be tested before "3b" (substring containment).
    if name_lower.contains("32b") {
        return ModelCapabilityProfile::large_32b_48gb();
    }
    if name_lower.contains("7b") {
        return ModelCapabilityProfile::micro_7b_12gb();
    }
    if name_lower.contains("3b") {
        // 3B has two tiers: 12GB (35K) and 8GB (20K).
        return match vram_tier {
            VramTier::Vram8Gb => ModelCapabilityProfile::micro_8gb(),
            _ => ModelCapabilityProfile::micro_3b_12gb(),
        };
    }

    // Unknown model: warn once and return safest 12GB default.
    tracing::warn!(
        target: "orchestrator::context_budget",
        model = model_name,
        vram_tier = ?vram_tier,
        "detect_model_capability_profile: unknown model name; falling back to micro_7b_12gb"
    );
    ModelCapabilityProfile::micro_7b_12gb()
}

// ---------------------------------------------------------------------------
// C-25 — CumulativeTokenTracker + ProviderUsage mirror
// ---------------------------------------------------------------------------

/// C-25 — local mirror of provider usage fields (per C-25 DRIFT-B Option B).
///
/// Decouples `context_budget.rs` from the `elai-providers` crate: callers at
/// the cloud send site widen `u32` -> `u64` once when constructing this
/// struct from `elai_providers::types::Usage`. Two structs to maintain, one
/// to import — cheaper than a new crate-level dep.
///
/// All counts are cumulative-per-call (one provider response's worth);
/// `CumulativeTokenTracker` accumulates them across the task lifetime.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProviderUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

impl ProviderUsage {
    /// Construct a ProviderUsage from the providers crate's `Usage` shape
    /// (the four u32 fields surfaced by the Anthropic + openai-compat
    /// adapters). Widens to u64 at the boundary so the tracker never has to
    /// worry about per-call overflow.
    #[must_use]
    pub const fn from_u32(
        input_tokens: u32,
        output_tokens: u32,
        cache_read_tokens: u32,
        cache_write_tokens: u32,
    ) -> Self {
        Self {
            input_tokens: input_tokens as u64,
            output_tokens: output_tokens as u64,
            cache_read_tokens: cache_read_tokens as u64,
            cache_write_tokens: cache_write_tokens as u64,
        }
    }
}

/// C-25 — cross-call token accumulator + attention-ceiling recalibrator.
///
/// Companion to `cloud_mode::SpendAccumulator` (which tracks USD): this
/// tracker holds the raw token counts and recalibrates the runtime's
/// `ModelCapabilityProfile.attention_ceiling` based on the largest observed
/// prompt size + a safety margin.
///
/// Per C-25 DRIFT-E resolution, `recalibrate_budget` uses an
/// EXPANSION-ONLY clamp: `max(profile.attention_ceiling, last_response_prompt
/// + 2048)`. This guarantees the function never SHRINKS a measured ceiling
/// (e.g. cloud_200k=200_000) on a short reply; it only raises the ceiling
/// when a measured prompt size exceeds the current profile budget.
///
/// The 2048-token margin reserves room for the next round of system prompt
/// + tool definitions + assistant scratch without re-recalibrating on every
/// call. See `workflow/reasoning/C-25-decisions.md` decision (a).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CumulativeTokenTracker {
    pub prompt_tokens_total: u64,
    pub completion_tokens_total: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub last_response_prompt: u64,
    pub last_response_completion: u64,
}

/// C-25 — safety margin added on top of the largest observed prompt size
/// when recalibrating attention_ceiling. Reserves room for the next round
/// of system prompt + tool definitions + assistant scratch.
pub const RECALIBRATE_PROMPT_MARGIN_TOKENS: u64 = 2048;

/// C-25 — absolute floor for `recalibrate_budget`. Per the plan's
/// stub_ban contract, the function MUST clamp to at least this value;
/// never 0. Real models always have an attention ceiling much larger,
/// but the floor protects against a degenerate `last_response_prompt=0`
/// + corrupted profile case.
pub const RECALIBRATE_MIN_CEILING: usize = 4096;

impl CumulativeTokenTracker {
    /// Construct an empty tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one provider response's token usage. Updates the cumulative
    /// totals AND the per-call snapshot (`last_response_*`).
    pub fn record_response(&mut self, usage: &ProviderUsage) {
        self.prompt_tokens_total = self
            .prompt_tokens_total
            .saturating_add(usage.input_tokens);
        self.completion_tokens_total = self
            .completion_tokens_total
            .saturating_add(usage.output_tokens);
        self.cache_read_tokens = self
            .cache_read_tokens
            .saturating_add(usage.cache_read_tokens);
        self.cache_write_tokens = self
            .cache_write_tokens
            .saturating_add(usage.cache_write_tokens);
        self.last_response_prompt = usage.input_tokens;
        self.last_response_completion = usage.output_tokens;
    }

    /// Recalibrate the runtime model profile's `attention_ceiling` to
    /// accommodate the largest observed prompt. EXPANSION-ONLY per DRIFT-E:
    /// `max(profile.attention_ceiling, last_response_prompt + margin)` with
    /// a hard floor of `RECALIBRATE_MIN_CEILING` (4096) so the result is
    /// never zero. Updates `effective_ceiling` to stay in sync (the
    /// `resolve_effective_ceiling` policy is min(vram, attention); we
    /// recompute manually since this function does not own the vram input).
    ///
    /// Thin wrapper over [`Self::recalibrate_budget_with_calibration`] with
    /// no store, no identity, no artifact, no SSB — the legacy no-calibration
    /// path. The leg-2 production seam
    /// (`recalibrate_budget_with_calibration`) threads `Some(&ssb)` so the
    /// `ThresholdApplied` row lands off-agent; this wrapper stays `None` so
    /// existing callers (and the in-module tests) keep the no-emit contract.
    pub fn recalibrate_budget(&self, profile: &mut ModelCapabilityProfile) {
        self.recalibrate_budget_with_calibration(profile, None, None, None, None);
    }

    /// J-34 leg-2 seam — recalibrate the runtime profile AND apply a
    /// calibration artifact (or invalidate a stale one). This is the
    /// production caller of `context_calibration::apply_to_profile`,
    /// `CalibrationArtifact::from_curves`, and
    /// `CalibrationArtifactStore::invalidate`; the bench arm and the
    /// runtime recalibrate path both go through here.
    ///
    /// 1. ALWAYS runs the expansion-only attention-ceiling clamp first
    ///    (the body previously inlined in `recalibrate_budget`), so the
    ///    attention ceiling stays monotone regardless of calibration state.
    /// 2. If `artifact` is `Some`, applies it to `profile` via
    ///    [`crate::context_calibration::apply_to_profile`] (sets
    ///    `calibrated_trim_fraction` from the knee + copies the curves).
    /// 3. If `artifact` is `None` and `profile.fill_degradation_curve` is
    ///    `Some`, rebuilds a transient artifact from the profile's curves
    ///    via [`crate::context_calibration::CalibrationArtifact::from_curves`]
    ///    and applies it — the re-calibration path for a profile that
    ///    already carries curves but whose caller did not pre-lookup the
    ///    store artifact. The transient artifact uses a placeholder
    ///    identity_key and produced_at_secs=0; only the knee + curves
    ///    affect the profile.
    /// 4. If `artifact` is `None` and both `store` and `identity` are
    ///    `Some`, invalidates any stale artifact for this identity via
    ///    [`crate::context_calibration::CalibrationArtifactStore::invalidate`]
    ///    so the next lookup falls back to folklore constants until a fresh
    ///    bench runs (G-10 model-drift invalidation path). The caller logs
    ///    the fallback (J-34 NEGATIVE contract: no silent pseudo-calibration).
    ///
    /// `store` is `&mut` so `invalidate` can mutate; `artifact` is borrowed
    /// read-only. The two are independent: a caller may pass a pre-looked-up
    /// artifact with `store=None`, or pass `store` + `identity` with
    /// `artifact=None` to trigger invalidation without a prior lookup.
    ///
    /// `ssb` threads an optional SSB writer for the `ThresholdApplied` emit
    /// on the apply-artifact paths (explicit `artifact` OR rebuilt-from-curves).
    /// When `ssb` is `None` the leg falls back to
    /// [`crate::lock::try_ssb_writer`] so a production runtime that has
    /// installed the global writer still emits the row even when the caller
    /// did not thread one explicitly. When neither is `Some` (contract tests,
    /// cold start) the emit is a no-op. This is the production-wiring seam
    /// for `ThresholdApplied` off-agent observability (J-34 slice-3 R7/R10).
    /// The `CalibrationArtifactStore::insert` CurveMeasured emit (slice-2) is
    /// a separate site and stays unchanged.
    pub fn recalibrate_budget_with_calibration(
        &self,
        profile: &mut ModelCapabilityProfile,
        store: Option<&mut crate::context_calibration::CalibrationArtifactStore>,
        identity: Option<&str>,
        artifact: Option<&crate::context_calibration::CalibrationArtifact>,
        ssb: Option<&Arc<crate::ssb::Ssb>>,
    ) {
        let desired = self
            .last_response_prompt
            .saturating_add(RECALIBRATE_PROMPT_MARGIN_TOKENS);
        let desired_usize = usize::try_from(desired).unwrap_or(usize::MAX);
        let new_ceiling = profile
            .attention_ceiling
            .max(desired_usize)
            .max(RECALIBRATE_MIN_CEILING);
        profile.attention_ceiling = new_ceiling;
        match (profile.in_vram_ceiling, profile.attention_ceiling) {
            (0, 0) => {}
            (0, a) => profile.effective_ceiling = a,
            (v, 0) => profile.effective_ceiling = v,
            (v, a) => profile.effective_ceiling = v.min(a),
        }
        // Resolve the SSB writer: explicit `ssb` arg first, then the
        // process-global slot. Kept here so both apply paths below share one
        // resolution site and the emit lands exactly once per call.
        let resolved_ssb: Option<Arc<crate::ssb::Ssb>> = ssb
            .map(Arc::clone)
            .or_else(crate::lock::try_ssb_writer);
        if let Some(art) = artifact {
            crate::context_calibration::apply_to_profile(art, profile);
            if let Some(ssb_arc) = resolved_ssb.as_ref() {
                let calibrated = profile.calibrated_trim_fraction;
                crate::context_calibration::emit_context_calibration_event(
                    ssb_arc,
                    crate::ssb::ContextCalibrationKind::ThresholdApplied,
                    &art.identity_key,
                    crate::context_calibration::threshold_applied_details(
                        calibrated,
                        calibrated.is_none(),
                    ),
                );
            }
        } else if let Some(fill) = profile.fill_degradation_curve.clone() {
            let pos = profile.positional_value_curve.clone().unwrap_or_default();
            let rebuilt = crate::context_calibration::CalibrationArtifact::from_curves(
                String::new(),
                fill,
                pos,
                0,
            );
            crate::context_calibration::apply_to_profile(&rebuilt, profile);
            if let Some(ssb_arc) = resolved_ssb.as_ref() {
                let calibrated = profile.calibrated_trim_fraction;
                crate::context_calibration::emit_context_calibration_event(
                    ssb_arc,
                    crate::ssb::ContextCalibrationKind::ThresholdApplied,
                    &rebuilt.identity_key,
                    crate::context_calibration::threshold_applied_details(
                        calibrated,
                        calibrated.is_none(),
                    ),
                );
            }
        } else if let (Some(s), Some(id)) = (store, identity) {
            s.invalidate(id);
        }
    }

    /// C.5 (s-14276) — live per-session budget probe for the PreCompact
    /// compaction-ladder gate. Returns the IN-WINDOW occupancy, i.e. the
    /// most recent prompt's token count ([`Self::last_response_prompt`]),
    /// NOT the cumulative lifetime spend ([`Self::prompt_tokens_total`]).
    ///
    /// SEMANTIC (load-bearing, pinned by
    /// [`cumulative_token_tracker_tests::current_session_budget_returns_last_response_prompt`]):
    /// the PreCompact dispatcher gate (`hook_dispatcher.rs::dispatch_pre_compact`)
    /// fires when `used / limit > 0.85`. `used` MUST be the live context-window
    /// size at compaction time. Cumulative session spend over a long task can
    /// exceed the window many times over and would trip the 0.85 gate every
    /// PreCompact (causing repeated handoff saves + SSB churn). The most recent
    /// prompt size is what actually occupies the window, so it is the correct
    /// "current budget" signal. A fresh tracker returns `0` (default), so the
    /// gate sees fraction `0.0` and NOOPs — the safe degradation.
    #[must_use]
    pub fn current_session_budget(&self) -> u64 {
        self.last_response_prompt
    }
}

#[cfg(test)]
mod cumulative_token_tracker_tests {
    use super::*;

    fn cloud_profile() -> ModelCapabilityProfile {
        ModelCapabilityProfile::cloud_200k()
    }

    #[test]
    fn record_response_accumulates_across_calls() {
        let mut t = CumulativeTokenTracker::new();
        t.record_response(&ProviderUsage::from_u32(100, 50, 10, 5));
        t.record_response(&ProviderUsage::from_u32(200, 80, 20, 0));
        assert_eq!(t.prompt_tokens_total, 300);
        assert_eq!(t.completion_tokens_total, 130);
        assert_eq!(t.cache_read_tokens, 30);
        assert_eq!(t.cache_write_tokens, 5);
        assert_eq!(t.last_response_prompt, 200);
        assert_eq!(t.last_response_completion, 80);
    }

    #[test]
    fn current_session_budget_returns_last_response_prompt() {
        // C.5 (s-14276): pins the load-bearing semantic — the PreCompact
        // budget probe returns IN-WINDOW occupancy (last_response_prompt),
        // NOT cumulative lifetime spend (prompt_tokens_total). A fresh
        // tracker returns 0 so the 0.85 gate NOOPs (safe default).
        let mut t = CumulativeTokenTracker::new();
        assert_eq!(t.current_session_budget(), 0, "fresh tracker = 0");
        t.record_response(&ProviderUsage::from_u32(100, 50, 10, 5));
        t.record_response(&ProviderUsage::from_u32(200, 80, 20, 0));
        // prompt_tokens_total accumulated to 300; last_response_prompt is 200.
        assert_eq!(t.prompt_tokens_total, 300);
        assert_eq!(
            t.current_session_budget(),
            200,
            "current_session_budget MUST return last_response_prompt (200), \
             NOT cumulative prompt_tokens_total (300)"
        );
        assert_ne!(
            t.current_session_budget(),
            t.prompt_tokens_total,
            "in-window occupancy must differ from lifetime spend here"
        );
    }

    #[test]
    fn recalibrate_never_shrinks_cloud_200k() {
        let mut profile = cloud_profile();
        let original = profile.attention_ceiling;
        assert_eq!(original, 200_000);
        let mut t = CumulativeTokenTracker::new();
        // Tiny prompt: would shrink under naive max(4096, ...) clamp.
        t.record_response(&ProviderUsage::from_u32(50, 10, 0, 0));
        t.recalibrate_budget(&mut profile);
        assert_eq!(
            profile.attention_ceiling, 200_000,
            "expansion-only clamp must not shrink cloud_200k ceiling"
        );
    }

    #[test]
    fn recalibrate_expands_for_large_prompt() {
        // micro_7b_12gb has attention_ceiling = 25000. A 30K prompt + 2K margin
        // = 32048 must EXPAND the ceiling.
        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        let original = profile.attention_ceiling;
        assert_eq!(original, 25_000);
        let mut t = CumulativeTokenTracker::new();
        t.record_response(&ProviderUsage::from_u32(30_000, 200, 0, 0));
        t.recalibrate_budget(&mut profile);
        assert!(
            profile.attention_ceiling >= 32_048,
            "expected expansion to >= 32048, got {}",
            profile.attention_ceiling
        );
    }

    #[test]
    fn recalibrate_enforces_floor_on_degenerate_input() {
        // Synthesize a degenerate profile: attention_ceiling=0, no usage yet.
        let mut profile = ModelCapabilityProfile::cloud_200k();
        profile.attention_ceiling = 0;
        profile.in_vram_ceiling = 0;
        let t = CumulativeTokenTracker::new();
        t.recalibrate_budget(&mut profile);
        assert_eq!(
            profile.attention_ceiling, RECALIBRATE_MIN_CEILING,
            "stub_ban: recalibrate_budget MUST clamp to >= 4096"
        );
        assert!(profile.attention_ceiling > 0, "never 0");
    }

    #[test]
    fn recalibrate_with_calibration_applies_knee_artifact() {
        use crate::context_calibration::{
            apply_to_profile, CalibrationArtifact, CalibrationArtifactStore,
            FillDegradationCurve, FillSample, PositionalValueCurve,
        };
        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        assert!(profile.calibrated_trim_fraction.is_none());
        let curve = FillDegradationCurve {
            samples: vec![
                FillSample { fill_fraction: 0.10, outcome_rate: 1.00 },
                FillSample { fill_fraction: 0.50, outcome_rate: 0.95 },
                FillSample { fill_fraction: 0.67, outcome_rate: 0.60 },
                FillSample { fill_fraction: 0.85, outcome_rate: 0.30 },
            ],
        };
        let art = CalibrationArtifact::from_curves(
            "id-knee".to_string(),
            curve,
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        let t = CumulativeTokenTracker::new();
        t.recalibrate_budget_with_calibration(
            &mut profile,
            None,
            None,
            Some(&art),
            None,
        );
        let frac = profile.calibrated_trim_fraction
            .expect("knee artifact must set calibrated_trim_fraction");
        assert!((frac - 0.50).abs() < 1e-6, "trim fraction = knee fill 0.50, got {}", frac);
        assert!(profile.fill_degradation_curve.is_some());
        // Reference: direct apply_to_profile must agree.
        let mut direct = ModelCapabilityProfile::micro_7b_12gb();
        apply_to_profile(&art, &mut direct);
        assert_eq!(profile.calibrated_trim_fraction, direct.calibrated_trim_fraction);
    }

    #[test]
    fn recalibrate_with_calibration_invalidates_stale_on_none_artifact() {
        use crate::context_calibration::{
            CalibrationArtifact, CalibrationArtifactStore,
            FillDegradationCurve, FillSample, PositionalValueCurve,
        };
        let mut store = CalibrationArtifactStore::new();
        let art = CalibrationArtifact::from_curves(
            "id-stale".to_string(),
            FillDegradationCurve {
                samples: vec![
                    FillSample { fill_fraction: 0.1, outcome_rate: 1.0 },
                    FillSample { fill_fraction: 0.5, outcome_rate: 0.5 },
                ],
            },
            PositionalValueCurve::default(),
            1_700_000_000,
        );
        store.insert(art);
        assert!(store.get("id-stale").is_some());
        let mut profile = ModelCapabilityProfile::micro_7b_12gb();
        let t = CumulativeTokenTracker::new();
        t.recalibrate_budget_with_calibration(
            &mut profile,
            Some(&mut store),
            Some("id-stale"),
            None,
            None,
        );
        assert!(store.get("id-stale").is_none(), "invalidate must drop the stale artifact");
        assert!(profile.calibrated_trim_fraction.is_none(), "no pseudo-calibration on None");
    }
}

// ---------------------------------------------------------------------------
// B-10 / 3E.5 — file_overlap_score + compute_overlap
// ---------------------------------------------------------------------------

/// Default file-overlap threshold for SSB entry filtering (DECISION-G).
/// Entries scoring below this against the active agent's file set are
/// dropped before they compete for SsbEntries slot tokens. Tuned per spec
/// suggestion (e.g. 0.1); future tuning task may revisit with measured data.
pub const FILE_OVERLAP_THRESHOLD: f32 = 0.10;

/// Pure scalar overlap score between an agent's active file set and a single
/// SSB entry's file set. Defined as `|intersection| / |agent_files|`.
///
/// Edge cases:
///   - empty `agent_files` → 0.0 (nothing to score against)
///   - empty `entry_files` → 0.0 (nothing to overlap)
///   - exact equality → 1.0
///
/// O(n*m) where n = `agent_files.len()` and m = `entry_files.len()`. Both
/// are small in practice (single-digit) so a hash set is unnecessary.
pub fn file_overlap_score(agent_files: &[String], entry_files: &[String]) -> f32 {
    if agent_files.is_empty() || entry_files.is_empty() {
        return 0.0;
    }
    let intersect = entry_files
        .iter()
        .filter(|p| agent_files.iter().any(|a| a == *p))
        .count();
    intersect as f32 / agent_files.len() as f32
}

/// Rank a candidate set of SSB entries by file overlap with the active agent
/// and filter entries scoring below [`FILE_OVERLAP_THRESHOLD`].
///
/// Inputs are `(entry_id, entry_files)` so this stays decoupled from the
/// concrete `SsbEntry` shape (the caller queries SSB via `query_by_session`
/// + a per-entry file lookup, then forwards the pair). Output is sorted by
/// score descending so the caller can greedily truncate to slot capacity.
///
/// B-10 / 3E.5: the production caller emits `SsbActionType::SsbFilteredByOverlap`
/// after truncation with `entries_total = input.len()` and `entries_kept =
/// truncated_count` so the dashboard renders the filter ratio.
pub fn compute_overlap<I: Copy>(
    agent_files: &[String],
    entries: &[(I, Vec<String>)],
) -> Vec<(I, f32)> {
    let mut scored: Vec<(I, f32)> = entries
        .iter()
        .map(|(id, files)| (*id, file_overlap_score(agent_files, files)))
        .filter(|(_, score)| *score >= FILE_OVERLAP_THRESHOLD)
        .collect();
    // Stable sort by score descending; ties preserve input order (filter-stable).
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored
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
    /// H-31 (SCHEMA_VERSION 240): the user-preference bi-temporal ledger retrieval-into-prompt
    /// slot. APPENDED (= 8) after OutputReserve so existing ordinals are stable. Populated ONLY for
    /// the role allowlist {AgentRole::Executor} AND ELAI_USER_MEMORY_ENABLE=1 (the
    /// Personalization-Trap firewall, `crate::user_memory_firewall`); forced EMPTY otherwise.
    Memory = 8,
    /// H-53 (SCHEMA_VERSION 229): the pre-bound evidence anchor block. Volatile (query-specific);
    /// MUST land AFTER the last cache breakpoint (after_slot = CodeContext = 4, HD-2) so placing
    /// it after Memory = 8 preserves the cache prefix on all preceding slots. Populated ONLY on
    /// the /research|/ask answer path when ELAI_CITATION_PREBIND_DISABLE is NOT set. The slot is
    /// force-EMPTY otherwise; the ContextBudget allocates 0 tokens for it by default.
    /// NARROWING-OK: #[repr(u8)] discriminant 9 fits u8; `as usize` array-indexing is safe.
    /// H-50 (regime-aware context) will budget this slot per backend (smaller on the local lane);
    /// until H-50 ships the slot is allocated 0 tokens (no-op on local, cloud absorbs it).
    Evidence = 9,
    /// I-6: reserved budget slot for a later TaskSpec/claims/constraints anchor-pane feed.
    /// APPENDED (= 10) after the last cache breakpoint, preserving all existing ordinals and
    /// cache prefixes. Allocated 0 tokens by default; content wiring ships with the supervisor.
    AnchorPane = 10,
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
    pub prefix_kv_eligible: bool,
    /// J-34 leg-3: metadata-only ordering seed derived from
    /// `positional_value_curve`. Does NOT reorder `slots` (fixed ordinals).
    pub ordering_seed: Option<String>,
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
                // H-31: the user-memory slot (appended last before H-53; ordinal 8).
                SlotAllocation {
                    slot: ContextSlot::Memory,
                    allocated_tokens: 0,
                    floor_tokens: 0,
                    actual_tokens: 0,
                    source_truncated: false,
                },
                // H-53: the pre-bound evidence anchor block (ordinal 9). Volatile
                // (query-specific); allocated 0 tokens by default — the /research
                // path sets actual_tokens when the slot is populated. HD-2: MUST
                // land after the last cache breakpoint (CodeContext = 4).
                SlotAllocation {
                    slot: ContextSlot::Evidence,
                    allocated_tokens: 0,
                    floor_tokens: 0,
                    actual_tokens: 0,
                    source_truncated: false,
                },
                SlotAllocation {
                    slot: ContextSlot::AnchorPane,
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
            prefix_kv_eligible: false,
            ordering_seed: None,
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
        // NARROWING-OK: f64->usize; no TryFrom<f64> for usize in std; dynamic_budget is a token
        // count (at most ~2M) so the f64 product fits usize with truncation-as-floor as intended.
        debug_assert!(
            dynamic_budget < (1usize << 53),
            "dynamic_budget {dynamic_budget} exceeds f64 mantissa safe integer range"
        );
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

        if let Some(seed) = &profile.ordering_seed {
            budget.ordering_seed = Some(seed.clone());
            if let Some(ssb_arc) = crate::lock::try_ssb_writer() {
                crate::context_calibration::emit_context_calibration_event(
                    &ssb_arc,
                    crate::ssb::ContextCalibrationKind::PlacementApplied,
                    &profile.model_name,
                    crate::context_calibration::placement_applied_details(
                        SLOT_COUNT,
                        seed,
                    ),
                );
            }
        }

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

        // NARROWING-OK (block): ContextSlot is #[repr(u8)] with values 0..=9 (Evidence=9); `as usize` is enum-discriminant indexing, structurally safe.
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
            let bound = bp.after_slot as usize; // NARROWING-OK: after_slot is ContextSlot (#[repr(u8)]); discriminant ≤ 9 (Evidence), always fits usize
            let mut hasher = blake3::Hasher::new();
            for slot_bytes in &slot_contents[..=bound] {
                // Length-prefix each slot to prevent boundary ambiguity:
                // [b"ab", b"cd"] must hash differently from [b"a", b"bcd"].
                hasher.update(&(slot_bytes.len() as u64).to_le_bytes()); // WIDENING-OK: usize->u64 (widening on 64-bit)
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
                bp.valid = self.prefix_kv_eligible;
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
                    level: level as u8, // NARROWING-OK: AdaptiveLevel is #[repr(u8)]; const assert at line 640 pins Terminal==4
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
        // B-10 (Task 6): emit a Terminal-level context_reduction row before the
        // terminal error returns so the dashboard sees the escalation event.
        // before_tokens / after_tokens both reflect the post-Level3 totals so
        // consumers can distinguish "fired but ineffective" from per-level rows.
        let terminal_payload = SsbPayload::ContextReduction(ContextReductionPayload {
            level: AdaptiveLevel::Terminal as u8, // NARROWING-OK: AdaptiveLevel is #[repr(u8)]; Terminal==4
            before_tokens: self.total_used,
            after_tokens: self.total_used,
            slots_affected: Vec::new(),
        });
        // SSB write failure must not mask the Escalate / SplitTask error.
        if let Err(e) = ssb
            .write_payload("context_budget", 0, terminal_payload, vec![])
            .await
        {
            tracing::warn!(
                target: "orchestrator::context_budget",
                error = %e,
                "Terminal-level context_reduction emit failed; continuing with terminal error"
            );
        }
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

    // NARROWING-OK (block for reduce_* helpers): ContextSlot is #[repr(u8)] with values 0..=9
    // (Memory=8, Evidence=9); `slot as usize` is enum-discriminant array indexing (structurally
    // safe, const-asserted). `slot as u8` produces the on-wire byte for `slots_affected`; same
    // safety guarantee.

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
    // NARROWING-OK: f64->usize; no TryFrom<f64> for usize in std; current is a token count
    // (bounded by model_max_context, at most ~2M), factor in [0.0,1.0] so product <= current.
    let reduced = (current as f64 * (1.0 - factor)) as usize;
    reduced.max(floor)
}

// ---------------------------------------------------------------------------
// B-10 (DECISION-C): SSB emitters for ContextBudgetComputed +
// SsbFilteredByOverlap. Pure async helpers — keep `compute_with_profile`
// sync so the 50+ existing test callsites and `build_dispatch_context` (also
// sync) keep working.
// ---------------------------------------------------------------------------

/// Emit a `ContextBudgetComputed` SSB row summarizing a freshly-computed
/// `ContextBudget`. Called by production wirings after a successful
/// `compute_with_profile`. The dashboard reads this for the per-slot bar chart.
pub async fn emit_context_budget_computed(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    profile: &ModelCapabilityProfile,
    budget: &ContextBudget,
    raw_max: usize,
) -> Result<i64, crate::ssb::SsbError> {
    // Encode slot weights as basis points (0..=10000) so the on-wire payload
    // stays integer-typed across architectures (avoids f32 endianness drift).
    let weights = &profile.slot_weights;
    // NARROWING-OK: f64->u64 after clamp(0.0, 10_000.0); no TryFrom<f64> for u64 in std;
    // value is clamped so always in [0, 10000] which fits u64 exactly.
    let to_bp = |v: f32| -> u64 { (v * 10_000.0).round().clamp(0.0, 10_000.0) as u64 };
    let slots: Vec<u64> = vec![
        to_bp(weights.system_prompt),
        to_bp(weights.rules),
        to_bp(weights.telescope),
        to_bp(weights.repo_map),
        to_bp(weights.code_context),
        to_bp(weights.ssb_entries),
        to_bp(weights.grc_history),
        0, // OutputReserve has no weight (carries fixed reserve, not a ratio).
    ];
    let allocations: Vec<usize> =
        budget.slots.iter().map(|s| s.allocated_tokens).collect();
    let payload = SsbPayload::ContextBudgetComputed(ContextBudgetComputedPayload {
        tier: profile.model_name.clone(),
        raw_max,
        slots,
        allocations,
        adapt_levels_fired: budget.adaptive_level as u8, // NARROWING-OK: AdaptiveLevel is #[repr(u8)]; Terminal==4
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await
}

/// Emit an `SsbFilteredByOverlap` SSB row summarizing a per-slot SSB entry
/// truncation. Called by the assembler after `compute_overlap` ranks and
/// the slot-fill site truncates to fit token capacity.
pub async fn emit_ssb_filtered_by_overlap(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    entries_total: usize,
    entries_kept: usize,
) -> Result<i64, crate::ssb::SsbError> {
    let payload = SsbPayload::SsbFilteredByOverlap(SsbFilteredByOverlapPayload {
        entries_total,
        entries_kept,
        overlap_threshold: FILE_OVERLAP_THRESHOLD,
    });
    ssb.write_payload(agent_id, superstep, payload, Vec::new()).await
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
                // H-31: user-memory slot -- small, one-line-per-decision, k <= 8.
                ContextSlot::Memory => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_MEMORY,
                    skip_content: false,
                    entry_limit: Some(8),
                    conclusion_only: false,
                },
                // H-53: evidence slot -- ranked excerpts for freeze_anchors pre-binding pass.
                // After last cache breakpoint (HD-2). Micro: up to 5 excerpts.
                ContextSlot::Evidence => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_EVIDENCE,
                    skip_content: false,
                    entry_limit: Some(5),
                    conclusion_only: false,
                },
                ContextSlot::AnchorPane => SlotAssemblyConfig {
                    max_tokens: MICRO_SLOT_ANCHOR_PANE,
                    skip_content: false,
                    entry_limit: Some(3),
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
                    // H-31: user-memory slot -- small, one-line-per-decision, k <= 8.
                    ContextSlot::Memory => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_MEMORY,
                        skip_content: false,
                        entry_limit: Some(8),
                        conclusion_only: false,
                    },
                    // H-53: evidence slot -- ranked excerpts for freeze_anchors pre-binding pass.
                    // After last cache breakpoint (HD-2). Large: up to 10 excerpts.
                    ContextSlot::Evidence => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_EVIDENCE,
                        skip_content: false,
                        entry_limit: Some(10),
                        conclusion_only: false,
                    },
                    ContextSlot::AnchorPane => SlotAssemblyConfig {
                        max_tokens: LARGE_SLOT_ANCHOR_PANE,
                        skip_content: false,
                        entry_limit: Some(3),
                        conclusion_only: false,
                    },
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Token reclamation constants (A-42: token-efficiency-design.txt section 9)
// ---------------------------------------------------------------------------

/// Minimum number of consecutive import lines to trigger collapsing.
const IMPORT_COLLAPSE_THRESHOLD: usize = 4;

/// Minimum path length (chars) before abbreviation is applied.
const PATH_ABBREV_MIN_LEN: usize = 8;

/// Minimum symbol name length for deduplication (ignore short names like "T", "E").
const DEDUP_MIN_SYMBOL_LEN: usize = 3;

/// Recognized common path prefixes to strip during path abbreviation.
const PATH_PREFIX_SRC: &str = "src/";
const PATH_PREFIX_LIB: &str = "lib/";
const PATH_PREFIX_CRATES: &str = "crates/";
const PATH_PREFIX_ELAI_CC_SRC: &str = ".elai_cc/src/";

/// All recognized path prefixes, ordered longest-first for greedy match.
const PATH_PREFIXES: &[&str] = &[
    PATH_PREFIX_ELAI_CC_SRC,
    PATH_PREFIX_CRATES,
    PATH_PREFIX_SRC,
    PATH_PREFIX_LIB,
];

/// Rust import keyword prefix.
const RUST_IMPORT_PREFIX: &str = "use ";
/// Python import keyword prefix.
const PYTHON_IMPORT_PREFIX: &str = "import ";
/// Python from-import keyword prefix.
const PYTHON_FROM_PREFIX: &str = "from ";

/// Rust definition keywords for deduplication scanning.
const DEDUP_KEYWORDS: &[&str] = &["struct ", "enum ", "fn ", "type ", "trait "];

// ---------------------------------------------------------------------------
// Token reclamation technique implementations (A-42)
// ---------------------------------------------------------------------------

/// Technique 1: formatting removal.
///
/// Strips trailing whitespace per line, collapses 2+ consecutive blank lines
/// to 1, and removes trailing line comments (// ...) for non-Python files.
/// Preserves: string literals, docstrings, Python indentation (semantic),
/// doc comments (///).
///
/// Research-grounded: 24.5% token reduction on code content, zero accuracy
/// degradation (LLM-Code-Generation-Summary.txt section 3e).
fn reclaim_formatting(content: &str, language: Option<&str>) -> String {
    if content.is_empty() {
        return String::new();
    }

    let is_python = matches!(language, Some("python" | "py"));
    let mut result = String::with_capacity(content.len());
    let mut consecutive_blanks: usize = 0;

    for line in content.lines() {
        // Strip trailing whitespace (safe: trim_end works on char boundaries).
        let trimmed_end = line.trim_end();

        // For non-Python: strip trailing line comments (// but not ///).
        let processed = if !is_python {
            strip_trailing_comment(trimmed_end)
        } else {
            trimmed_end.to_owned()
        };

        // Collapse consecutive blank lines.
        if processed.trim().is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks <= 1 {
                result.push('\n');
            }
            continue;
        }

        consecutive_blanks = 0;

        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&processed);
    }

    // Trim trailing newlines to avoid growth.
    let final_result = result.trim_end_matches('\n');
    final_result.to_owned()
}

/// Strip trailing line comments (// ...) from a line, preserving:
/// - Doc comments (/// ...)
/// - Comments inside string literals
/// - Lines that are purely comments
fn strip_trailing_comment(line: &str) -> String {
    let trimmed = line.trim_start();

    // Pure comment line — preserve entirely.
    if trimmed.starts_with("//") {
        return line.to_owned();
    }

    // Find // that is not inside a string literal.
    let mut in_string = false;
    let mut escape_next = false;
    let mut string_char: char = '"';

    for (i, ch) in line.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape_next = true;
            continue;
        }
        if !in_string && (ch == '"' || ch == '\'') {
            in_string = true;
            string_char = ch;
            continue;
        }
        if in_string && ch == string_char {
            in_string = false;
            continue;
        }
        if !in_string && ch == '/' {
            // Check next char.
            let rest = &line[i..];
            if rest.starts_with("///") {
                // Doc comment — preserve.
                return line.to_owned();
            }
            if rest.starts_with("//") {
                // Trailing comment — strip it.
                return line[..i].trim_end().to_owned();
            }
        }
    }

    line.to_owned()
}

/// Technique 3: path abbreviation.
///
/// Strips recognized common prefixes (src/, lib/, crates/, .elai_cc/src/) from
/// file paths. Model can infer the prefix from context.
/// Only abbreviates paths longer than PATH_ABBREV_MIN_LEN.
/// Note: uses byte length for threshold check. Assumes ASCII file paths
/// (standard for Rust/Python codebases). Non-ASCII paths with multibyte
/// characters may behave slightly differently at the length boundary.
pub fn abbreviate_path(path: &str) -> String {
    if path.len() <= PATH_ABBREV_MIN_LEN {
        return path.to_owned();
    }

    for prefix in PATH_PREFIXES {
        if path.starts_with(prefix) {
            // Safe: prefix is ASCII, so prefix.len() is a valid char boundary.
            return path[prefix.len()..].to_owned();
        }
    }

    path.to_owned()
}

/// Technique 4: import collapsing.
///
/// Detects runs of consecutive import lines (use/import/from). If a run has
/// >= IMPORT_COLLAPSE_THRESHOLD lines, replaces with a placeholder comment.
/// Non-import, non-blank lines break the current block.
fn reclaim_imports(content: &str, language: Option<&str>) -> String {
    if content.is_empty() {
        return String::new();
    }

    let is_python = matches!(language, Some("python" | "py"));
    let lines: Vec<&str> = content.lines().collect();
    let mut result = String::with_capacity(content.len());
    let mut i = 0;

    while i < lines.len() {
        if is_import_line(lines[i], is_python) {
            // Start of a potential import block.
            let block_start = i;
            while i < lines.len() && (is_import_line(lines[i], is_python) || lines[i].trim().is_empty()) {
                i += 1;
            }
            // Count actual import lines (not blank lines).
            let import_count = lines[block_start..i]
                .iter()
                .filter(|l| is_import_line(l, is_python))
                .count();

            if import_count >= IMPORT_COLLAPSE_THRESHOLD {
                let comment_prefix = if is_python { "#" } else { "//" };
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push_str(&format!("{} [{} imports]", comment_prefix, import_count));
            } else {
                // Below threshold — keep all lines.
                for line in &lines[block_start..i] {
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    result.push_str(line);
                }
            }
        } else {
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(lines[i]);
            i += 1;
        }
    }

    result
}

/// Check if a line is an import statement.
fn is_import_line(line: &str, is_python: bool) -> bool {
    let trimmed = line.trim_start();
    if is_python {
        trimmed.starts_with(PYTHON_IMPORT_PREFIX) || trimmed.starts_with(PYTHON_FROM_PREFIX)
    } else {
        trimmed.starts_with(RUST_IMPORT_PREFIX)
    }
}

/// Technique 5: deduplication of repeated type/struct/enum/fn definitions.
///
/// On second and subsequent occurrence of an identical definition header line,
/// replaces with "// [dup: <symbol_name>]". Symbol name must be at least
/// DEDUP_MIN_SYMBOL_LEN characters.
fn reclaim_dedup(content: &str) -> String {
    if content.is_empty() {
        return String::new();
    }

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut result = String::with_capacity(content.len());

    for line in &lines {
        let trimmed = line.trim();
        if let Some(symbol) = extract_definition_symbol(trimmed) {
            if symbol.len() >= DEDUP_MIN_SYMBOL_LEN {
                // Normalize: use the trimmed line as key.
                let key = trimmed.to_owned();
                if seen.contains(&key) {
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    result.push_str(&format!("// [dup: {}]", symbol));
                    continue;
                }
                seen.insert(key);
            }
        }

        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(line);
    }

    result
}

/// Extract a symbol name from a definition header line.
/// Returns None if line is not a recognized definition.
/// Note: handles `pub` prefix but not `pub(crate)` or `pub(super)` visibility
/// qualifiers. Those definitions will pass through without deduplication.
fn extract_definition_symbol(line: &str) -> Option<String> {
    for keyword in DEDUP_KEYWORDS {
        if line.starts_with(keyword) {
            let rest = &line[keyword.len()..];
            // Symbol is the next word (up to whitespace, '<', '(', or '{').
            let symbol: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !symbol.is_empty() {
                return Some(symbol);
            }
        }
        // Also match "pub struct", "pub fn", etc.
        let pub_keyword = format!("pub {}", keyword);
        if line.starts_with(&pub_keyword) {
            let rest = &line[pub_keyword.len()..];
            let symbol: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !symbol.is_empty() {
                return Some(symbol);
            }
        }
    }
    None
}

/// Technique 2: output formatting restoration (stub).
///
/// After LLM generates code, apply tree-sitter pretty-print to restore proper
/// indentation before writing to disk. Model generates minimally formatted code,
/// system restores deterministically.
///
/// STUB: tree-sitter not added to orchestrator due to dep cycle risk. stub-ban-allow: A-42-T2 follow-up
/// with knowledge crate. Wire after dep audit (see A-42-T2).
pub fn restore_output_formatting(code: &str, _language: &str) -> String {
    // A-42-T2: tree-sitter pretty-print blocked — adding tree-sitter to
    // orchestrator risks dep cycle with knowledge crate. Wire after dep audit.
    code.to_owned()
}

// ---------------------------------------------------------------------------
// Token reclamation entry point (A-42)
// ---------------------------------------------------------------------------

/// Apply token reclamation techniques to slot content.
///
/// For CodeContext (slot 4): applies T1 (formatting) -> T4 (imports) -> T5 (dedup).
/// For SsbEntries (slot 5): applies T3 (path abbreviation) line-by-line.
/// Other slots: returns content unchanged.
///
/// The caller must set `SlotAllocation.source_truncated = true` if the returned
/// string is shorter than the input.
pub fn apply_token_reclamation(
    content: &str,
    slot: ContextSlot,
    language: Option<&str>,
) -> String {
    if content.is_empty() {
        return String::new();
    }

    match slot {
        ContextSlot::CodeContext => {
            let step1 = reclaim_formatting(content, language);
            let step2 = reclaim_imports(&step1, language);
            reclaim_dedup(&step2)
        }
        ContextSlot::SsbEntries => {
            // Path abbreviation applied line-by-line to SSB entries.
            content
                .lines()
                .map(|line| {
                    // Try to find a path-like token and abbreviate it.
                    abbreviate_ssb_line(line)
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => content.to_owned(),
    }
}

/// Abbreviate paths found in an SSB entry line.
/// Looks for tokens that look like file paths (contain '/' and end with
/// a file extension) and abbreviates them.
fn abbreviate_ssb_line(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut last_end = 0;

    // Simple heuristic: find tokens that contain '/' (potential paths).
    for (start, _) in line.char_indices() {
        if start < last_end {
            continue;
        }
        // Find word boundaries around this position.
        if start == 0 || line.as_bytes().get(start.wrapping_sub(1)).map_or(true, |b| *b == b' ' || *b == b'\t' || *b == b':' || *b == b'(' || *b == b'[') {
            // Potential start of a path token.
            let rest = &line[start..];
            let token_len = rest.find(|c: char| c == ' ' || c == '\t' || c == ')' || c == ']' || c == ',' || c == ';')
                .unwrap_or(rest.len());
            let token = &rest[..token_len];
            if token.contains('/') && token.len() > PATH_ABBREV_MIN_LEN {
                let abbreviated = abbreviate_path(token);
                if abbreviated.len() < token.len() {
                    result.push_str(&line[last_end..start]);
                    result.push_str(&abbreviated);
                    last_end = start + token_len;
                }
            }
        }
    }

    result.push_str(&line[last_end..]);
    result
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

/// J-24: pre-assessment mode for observation-size steering.
///
/// Classifies whether an incoming tool-result observation will pressure
/// the context window, before it is pushed into the session. The caller
/// emits an `ObservationSizePreassessed` SSB row with the mode + metrics
/// and optionally invokes `adaptive_reduce` when the mode is not `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreassessMode {
    None,
    Selective,
    All,
}

/// J-24: pre-assess an observation against the remaining context slot.
///
/// `alpha` (typically 0.8) is the soft-pressure threshold fraction.
/// Returns `None` when the observation fits comfortably, `Selective` when
/// it exceeds the soft threshold but still fits, and `All` when it
/// exceeds the full remaining slot.
#[must_use]
pub fn preassess_observation_size(
    observation_tokens: usize,
    slot_remaining: usize,
    alpha: f64,
) -> PreassessMode {
    let soft_threshold = slot_remaining as f64 * alpha;
    if observation_tokens as f64 <= soft_threshold {
        PreassessMode::None
    } else if observation_tokens <= slot_remaining {
        PreassessMode::Selective
    } else {
        PreassessMode::All
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
        // H-31: the user-memory slot is APPENDED last; OutputReserve's ordinal is unchanged.
        assert_eq!(ContextSlot::Memory as u8, 8);
        assert_eq!(ContextSlot::Evidence as u8, 9);
        assert_eq!(ContextSlot::AnchorPane as u8, 10);
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
            b"user_memory",
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
            b"user_memory",
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
            b"user_memory",
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
            b"user_memory",
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
    fn ineligible_budget_never_validates_breakpoints() {
        let mut budget = ContextBudget::new(131072);
        assert!(!budget.prefix_kv_eligible, "default budget must be ineligible");
        let contents = make_slot_contents();
        budget.update_breakpoints(&contents);
        budget.update_breakpoints(&contents);
        for bp in &budget.cache_breakpoints {
            assert!(!bp.valid, "ineligible budget must never have valid breakpoints");
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
        // B-10 (DECISION-D): per token-efficiency-design.txt §2 table, 8GB
        // Telescope = 600 (not the universal FIXED_SLOT_TELESCOPE_TARGET=1050).
        assert_eq!(p.fixed_slot_targets, [200, 300, 600]);
    }

    #[test]
    fn per_tier_output_reserve_micro_512() {
        // B-10 (DECISION-D): spec §2 table specifies output_reserve = 512 for ALL tiers
        // (rationale: real code outputs rarely exceed 256, design-grounded; previous
        // values 750 / 4000 were OUT-OF-SPEC divergences).
        let p7b = ModelCapabilityProfile::micro_7b_12gb();
        assert_eq!(p7b.output_reserve, 512, "MICRO 7B output_reserve must be 512 per spec");
        let p3b = ModelCapabilityProfile::micro_3b_12gb();
        assert_eq!(p3b.output_reserve, 512, "MICRO 3B output_reserve must be 512 per spec");
        let p8g = ModelCapabilityProfile::micro_8gb();
        assert_eq!(p8g.output_reserve, 512, "MICRO 8GB output_reserve must be 512 per spec");
    }

    #[test]
    fn per_tier_output_reserve_large_512() {
        // B-10 (DECISION-D): spec §2 table specifies 512 universally.
        let p = ModelCapabilityProfile::large_32b_48gb();
        assert_eq!(p.output_reserve, 512, "LARGE output_reserve must be 512 per spec");
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
        // B-10 (DECISION-D): per spec §2 table, micro_7b_12gb Telescope = 800.
        assert_eq!(budget.slots[0].allocated_tokens, 330, "SystemPrompt");
        assert_eq!(budget.slots[1].allocated_tokens, 400, "Rules");
        assert_eq!(budget.slots[2].allocated_tokens, 800, "Telescope (spec §2 table)");
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
            quant_format: "NF4+FA2".to_string(),
            fill_degradation_curve: None,
            positional_value_curve: None,
            calibrated_trim_fraction: None,
            ordering_seed: None,
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

    // -- Token reclamation tests (A-42) ------------------------------------

    #[test]
    fn test_reclamation_formatting_strips_trailing_whitespace() {
        let input = "fn main() {   \n    let x = 1;  \n}  ";
        let out = reclaim_formatting(input, None);
        for line in out.lines() {
            assert_eq!(line, line.trim_end(), "trailing whitespace found: {:?}", line);
        }
    }

    #[test]
    fn test_reclamation_formatting_collapses_blank_lines() {
        let input = "line1\n\n\n\nline2\n\n\n\n\nline3";
        let out = reclaim_formatting(input, None);
        // No run of 2+ consecutive blank lines.
        assert!(!out.contains("\n\n\n"), "3+ blanks found in: {:?}", out);
        assert!(out.contains("line1"));
        assert!(out.contains("line2"));
        assert!(out.contains("line3"));
    }

    #[test]
    fn test_reclamation_formatting_preserves_python_indentation() {
        let input = "def foo():\n    if True:\n        pass\n    return 1";
        let out = reclaim_formatting(input, Some("python"));
        assert!(out.contains("    if True:"), "Python indentation lost");
        assert!(out.contains("        pass"), "Python nested indentation lost");
    }

    #[test]
    fn test_reclamation_formatting_preserves_string_literals() {
        let input = r#"let s = "hello // world";"#;
        let out = reclaim_formatting(input, None);
        assert!(out.contains(r#""hello // world""#), "string literal modified");
    }

    #[test]
    fn test_reclamation_formatting_empty_input() {
        let out = reclaim_formatting("", None);
        assert_eq!(out, "");
    }

    #[test]
    fn test_reclamation_formatting_strips_trailing_comment() {
        let input = "let x = 1; // set x to 1";
        let out = reclaim_formatting(input, None);
        assert_eq!(out, "let x = 1;");
    }

    #[test]
    fn test_reclamation_formatting_preserves_doc_comment() {
        let input = "/// This is a doc comment";
        let out = reclaim_formatting(input, None);
        assert_eq!(out, "/// This is a doc comment");
    }

    #[test]
    fn test_reclamation_formatting_preserves_pure_comment_line() {
        let input = "// This is a standalone comment";
        let out = reclaim_formatting(input, None);
        assert_eq!(out, "// This is a standalone comment");
    }

    #[test]
    fn test_reclamation_path_abbreviation_strips_src() {
        assert_eq!(abbreviate_path("src/foo/bar.rs"), "foo/bar.rs");
    }

    #[test]
    fn test_reclamation_path_abbreviation_strips_crates() {
        assert_eq!(abbreviate_path("crates/orchestrator/src/main.rs"), "orchestrator/src/main.rs");
    }

    #[test]
    fn test_reclamation_path_abbreviation_no_match() {
        assert_eq!(abbreviate_path("external/path.rs"), "external/path.rs");
    }

    #[test]
    fn test_reclamation_path_abbreviation_short_path() {
        // Path <= PATH_ABBREV_MIN_LEN (8) must not be abbreviated.
        assert_eq!(abbreviate_path("src/a.r"), "src/a.r");
        // Exactly 8 chars — still not abbreviated.
        assert_eq!(abbreviate_path("src/a.rs"), "src/a.rs");
    }

    #[test]
    fn test_reclamation_path_abbreviation_elai_cc_src() {
        assert_eq!(
            abbreviate_path(".elai_cc/src/context_budget.rs"),
            "context_budget.rs"
        );
    }

    #[test]
    fn test_reclamation_import_collapsing_below_threshold() {
        let input = "use std::io;\nuse std::fs;\nuse std::path;\n\nfn main() {}";
        let out = reclaim_imports(input, None);
        assert!(out.contains("use std::io;"), "imports below threshold should be kept");
        assert!(out.contains("use std::fs;"));
        assert!(out.contains("use std::path;"));
    }

    #[test]
    fn test_reclamation_import_collapsing_at_threshold() {
        let input = "use std::io;\nuse std::fs;\nuse std::path;\nuse std::env;\n\nfn main() {}";
        let out = reclaim_imports(input, None);
        assert!(out.contains("// [4 imports]"), "4 imports should be collapsed: {:?}", out);
        assert!(!out.contains("use std::io;"));
    }

    #[test]
    fn test_reclamation_import_collapsing_non_import_breaks_block() {
        let input = "use a;\nuse b;\n\nlet x = 1;\n\nuse c;\nuse d;";
        let out = reclaim_imports(input, None);
        // Both groups have < 4 imports, so both should be kept.
        assert!(out.contains("use a;"));
        assert!(out.contains("use b;"));
        assert!(out.contains("use c;"));
        assert!(out.contains("use d;"));
    }

    #[test]
    fn test_reclamation_import_collapsing_python() {
        let input = "import os\nimport sys\nimport json\nimport math\n\ndef main(): pass";
        let out = reclaim_imports(input, Some("python"));
        assert!(out.contains("# [4 imports]"), "Python imports should collapse: {:?}", out);
    }

    #[test]
    fn test_reclamation_dedup_second_occurrence_replaced() {
        let input = "struct Foo {\n    x: i32,\n}\n\nstruct Foo {\n    x: i32,\n}";
        let out = reclaim_dedup(input);
        // First occurrence kept.
        assert!(out.contains("struct Foo {"));
        // Second occurrence replaced.
        assert!(out.contains("// [dup: Foo]"));
    }

    #[test]
    fn test_reclamation_dedup_first_occurrence_preserved() {
        let input = "struct MyType {\n    val: u32,\n}\nstruct MyType {";
        let out = reclaim_dedup(input);
        let first_pos = out.find("struct MyType");
        assert!(first_pos.is_some(), "first occurrence must be preserved");
    }

    #[test]
    fn test_reclamation_dedup_short_symbol_not_deduped() {
        let input = "fn ab() {}\nfn ab() {}";
        let out = reclaim_dedup(input);
        // "ab" has len 2 < DEDUP_MIN_SYMBOL_LEN (3), so both kept.
        let count = out.matches("fn ab()").count();
        assert_eq!(count, 2, "short symbols should not be deduped");
    }

    #[test]
    fn test_reclamation_dedup_pub_struct() {
        let input = "pub struct Config {\n}\npub struct Config {";
        let out = reclaim_dedup(input);
        assert!(out.contains("// [dup: Config]"), "pub struct should be deduped: {:?}", out);
    }

    #[test]
    fn test_reclamation_apply_reduces_length() {
        let input = "use std::io;\nuse std::fs;\nuse std::path;\nuse std::env;\n\n\n\n\nfn main() {   \n    let x = 1; // comment\n}\n";
        let out = apply_token_reclamation(input, ContextSlot::CodeContext, None);
        assert!(out.len() <= input.len(), "reclamation must not increase length: out={} in={}", out.len(), input.len());
    }

    #[test]
    fn test_reclamation_apply_empty_input() {
        let out = apply_token_reclamation("", ContextSlot::CodeContext, None);
        assert_eq!(out, "");
    }

    #[test]
    fn test_reclamation_apply_non_code_slot_unchanged() {
        let input = "system prompt content";
        let out = apply_token_reclamation(input, ContextSlot::SystemPrompt, None);
        assert_eq!(out, input, "non-CodeContext/SsbEntries slots should be unchanged");
    }

    #[test]
    fn test_reclamation_ssb_path_abbreviation() {
        let input = "changed: src/middleware/auth.rs\nmodified: crates/orchestrator/src/main.rs";
        let out = apply_token_reclamation(input, ContextSlot::SsbEntries, None);
        assert!(out.contains("middleware/auth.rs"), "src/ prefix should be stripped: {:?}", out);
    }

    #[test]
    fn test_reclamation_source_truncated_detection() {
        let input = "fn main() {   \n    let x = 1; // a comment  \n}\n\n\n\n";
        let out = apply_token_reclamation(input, ContextSlot::CodeContext, None);
        // Verify the caller can detect truncation.
        assert!(out.len() < input.len(), "reclamation should shorten this input");
    }

    #[test]
    fn test_reclamation_restore_output_formatting_stub() {
        let code = "fn main() { println!(\"hello\"); }";
        let out = restore_output_formatting(code, "rust");
        assert_eq!(out, code, "stub must return input unchanged");
    }

    #[test]
    fn test_reclamation_constants_valid() {
        assert!(IMPORT_COLLAPSE_THRESHOLD >= 2, "threshold must be >= 2");
        assert!(PATH_ABBREV_MIN_LEN > 0, "min path length must be > 0");
        assert!(DEDUP_MIN_SYMBOL_LEN >= 1, "min symbol length must be >= 1");
        for prefix in PATH_PREFIXES {
            assert!(prefix.ends_with('/'), "prefix {:?} must end with '/'", prefix);
        }
    }

    #[test]
    fn j36_fh4_micro_7b_turboquant_off_eligible() {
        let profile = ModelCapabilityProfile::micro_7b_12gb();
        assert!(prefix_kv_eligible(&profile, false));
    }

    #[test]
    fn j36_fh4_micro_7b_turboquant_on_ineligible() {
        let profile = ModelCapabilityProfile::micro_7b_12gb();
        assert!(!prefix_kv_eligible(&profile, true));
    }

    #[test]
    fn j36_fh4_cloud_200k_ineligible() {
        let profile = ModelCapabilityProfile::cloud_200k();
        assert!(!prefix_kv_eligible(&profile, false));
    }

    #[test]
    fn j36_fh4_none_default_ineligible() {
        assert!(!ModelFeatures::none().prefix_kv_eligible);
    }

    #[test]
    fn j24_preassess_none_when_fits() {
        assert_eq!(preassess_observation_size(100, 500, 0.8), PreassessMode::None);
    }

    #[test]
    fn j24_preassess_none_at_soft_boundary() {
        assert_eq!(preassess_observation_size(400, 500, 0.8), PreassessMode::None);
    }

    #[test]
    fn j24_preassess_selective_above_soft() {
        assert_eq!(preassess_observation_size(450, 500, 0.8), PreassessMode::Selective);
    }

    #[test]
    fn j24_preassess_all_exceeds_slot() {
        assert_eq!(preassess_observation_size(501, 500, 0.8), PreassessMode::All);
    }

    #[test]
    fn j24_preassess_all_from_s1_example() {
        assert_eq!(preassess_observation_size(1000, 500, 0.8), PreassessMode::All);
    }
}
