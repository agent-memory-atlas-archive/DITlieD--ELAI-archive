use std::cmp::min;

use crate::research_tier::ResearchTierConfig;
use crate::web_research_orchestrator::MAX_ITERATIONS_PER_DEPTH;

pub enum ResearchMode {
    Fast,
    Standard,
    Deep,
}

pub struct ModeCaps {
    pub max_iterations: u32,
    pub max_sources_per_node: usize,
    pub breadth_factor: usize,
}

impl ResearchMode {
    pub fn caps(&self, base: &ResearchTierConfig) -> ModeCaps {
        let depth = u32::try_from(base.depth_limit).unwrap_or(u32::MAX);
        match self {
            ResearchMode::Fast => ModeCaps {
                max_iterations: depth * 3,
                max_sources_per_node: min(base.max_sources_per_node, 3),
                breadth_factor: min(base.breadth_factor, 2),
            },
            ResearchMode::Standard => ModeCaps {
                max_iterations: depth * MAX_ITERATIONS_PER_DEPTH,
                max_sources_per_node: base.max_sources_per_node,
                breadth_factor: base.breadth_factor,
            },
            ResearchMode::Deep => ModeCaps {
                max_iterations: depth * 15,
                max_sources_per_node: base.max_sources_per_node + 5,
                breadth_factor: base.breadth_factor + 1,
            },
        }
    }

    pub fn from_env() -> ResearchMode {
        match std::env::var("ELAI_RESEARCH_MODE")
            .ok()
            .as_deref()
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
        {
            Some("fast") => ResearchMode::Fast,
            Some("deep") => ResearchMode::Deep,
            _ => ResearchMode::Standard,
        }
    }
}
