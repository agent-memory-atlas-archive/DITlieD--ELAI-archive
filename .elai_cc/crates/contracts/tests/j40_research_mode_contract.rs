// J-40 T2 contract tests: ResearchMode::caps.
// Exercises the SAME production fn wired at
//   web_research_orchestrator.rs run_node_loop (caps() at the three read sites)
//   task_runner.rs build_orchestrator (from_env + with_research_mode)
// No reimplemented clamp logic in the test body.

use elai_core::VramTier;
use elai_orchestrator::research_mode::ResearchMode;
use elai_orchestrator::research_tier::{research_config_for_tier, ResearchTierConfig};

fn test_config(depth: usize, sources: usize, breadth: usize) -> ResearchTierConfig {
    let mut cfg = research_config_for_tier(VramTier::Vram12Gb);
    cfg.depth_limit = depth;
    cfg.max_sources_per_node = sources;
    cfg.breadth_factor = breadth;
    cfg
}

#[test]
fn fast_mode_clamps_caps() {
    let cfg = test_config(5, 10, 4);
    let caps = ResearchMode::Fast.caps(&cfg);
    assert_eq!(caps.max_iterations, 15);
    assert_eq!(caps.max_sources_per_node, 3);
    assert_eq!(caps.breadth_factor, 2);
}

#[test]
fn standard_mode_passthrough() {
    let cfg = test_config(5, 10, 4);
    let caps = ResearchMode::Standard.caps(&cfg);
    assert_eq!(caps.max_iterations, 50);
    assert_eq!(caps.max_sources_per_node, 10);
    assert_eq!(caps.breadth_factor, 4);
}

#[test]
fn deep_mode_augments_caps() {
    let cfg = test_config(5, 10, 4);
    let caps = ResearchMode::Deep.caps(&cfg);
    assert_eq!(caps.max_iterations, 75);
    assert_eq!(caps.max_sources_per_node, 15);
    assert_eq!(caps.breadth_factor, 5);
}

#[test]
fn fast_mode_clamps_when_base_already_low() {
    let cfg = test_config(2, 2, 1);
    let caps = ResearchMode::Fast.caps(&cfg);
    assert_eq!(caps.max_iterations, 6);
    assert_eq!(caps.max_sources_per_node, 2);
    assert_eq!(caps.breadth_factor, 1);
}
