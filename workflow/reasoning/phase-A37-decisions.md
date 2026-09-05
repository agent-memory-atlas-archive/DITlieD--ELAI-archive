A-37 Decision Doc: RA VRAM-Tier Layer Configuration

1. VramTier move decision
Move VramTier enum from orchestrator/src/context_budget.rs to core/src/vram_tier.rs.
Reason: reasoning/ needs VramTier for layer routing, providers/ needs it for detect_vram_tier return type. Both already depend on elai-core. All 9 current uses are in context_budget.rs — replace with `use elai_core::VramTier`.
Orchestrator re-exports not needed since it already depends on elai-core.

2. Detection fallback approach
No sysinfo crate (not confirmed by user). Detection order (code matches):
  a. ELAI_VRAM_OVERRIDE env var (highest priority — user override)
  b. NVML via discover_gpus() (feature="vllm", picks max VRAM GPU)
  c. Conservative default: Vram8Gb (safest capacity assumption)
Note: VramTier::Cloud is only reachable via ELAI_VRAM_OVERRIDE=cloud or programmatic injection.

3. New types in core/src/vram_tier.rs
  VramTier — moved from orchestrator (6 variants: 8/12/16/24/48/Cloud)
  VramTierConfig — constructed from VramTier, holds layer config
  CloudImportance — MoreImportant / Secondary / Primary
  SwapStrategy — None / SwapModel
  ReasoningLayer — Layer1..Layer6

4. New files
  core/src/vram_tier.rs — types + VramTierConfig::from_tier()
  providers/src/vram_tier.rs — detect_vram_tier() wrapping discover_gpus()
  reasoning/src/layer_routing.rs — configure_layers_for_tier()

5. Expected test outcomes
  Contract tests: RED before impl, GREEN after
  cargo check -p elai-core: pass after core/src/vram_tier.rs
  cargo check -p elai-orchestrator: pass after updating context_budget.rs import
  cargo check -p elai-providers: pass after providers/src/vram_tier.rs
  cargo check -p elai-reasoning: pass after reasoning/src/layer_routing.rs

6. Unknowns
  NVML availability in CI — detect_vram_tier returns Err mapped to Vram8Gb fallback.
  Whether orchestrator tests need adjustment — grep shows only field assignments, re-export should suffice.
