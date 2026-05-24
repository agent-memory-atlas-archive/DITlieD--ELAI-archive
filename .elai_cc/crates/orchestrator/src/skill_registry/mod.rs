//! C-41 — Skills-driven workflow registry.
//!
//! First-class Rust-side registry for "skills" (markdown SKILL.md scaffolds
//! the orchestrator can consume during task routing). Sits ON TOP OF the
//! existing prior art in `crates/cli` (`SkillSummary`, `SkillRoot`,
//! `discover_skill_roots`, `load_skills_from_roots`, `parse_skill_frontmatter`)
//! and the harness Skill tool in `crates/tools` (`execute_skill`,
//! `resolve_skill_path`). C-41 EXTENDS those surfaces with:
//!   - a richer manifest schema (triggers / version / inputs / outputs /
//!     scope, parsed from extended YAML frontmatter — NOT a separate
//!     `meta.toml` file);
//!   - deterministic scoring + top-k ranking for context-aware routing;
//!   - an SSB observability surface (see `SsbActionType::SkillTriggered` +
//!     `SsbActionType::SkillExecuted` in `crate::ssb`).
//!
//! Paper grounding (workflow/papers/scientific-workflow-interface.md):
//! the headline 44.0% -> 83.3% full-match intent accuracy on Claude Opus
//! 4.6 (S0 vs S3, Section 5.1 Table 2) IS paper-grounded; the manifest
//! schema, scoring formula, and SSB vocabulary in this module are
//! LOCALLY AUTHORED and labeled as such in the per-file headers. Do NOT
//! claim paper-grounded for the wire shape — only for the motivation.

pub mod discovery;
pub mod eval;
pub mod manifest;
pub mod ranker;
pub mod scoring;

pub use discovery::{find_skills, parse_skill_frontmatter_extended, DiscoveryRoot};
pub use eval::{eval_skill_routing, EvalResult, SkillEvalQuery};
pub use manifest::{FrontmatterFields, SkillManifest, SkillOrigin};
pub use ranker::{top_k_skills, ScoredSkill};
pub use scoring::{score_skill_for_context, SkillContext, SKILL_CONFIDENCE_THRESHOLD};

/// blake3-prefix hash of the skill name suitable for the SSB
/// `skill_name_hash` payload field.
///
/// Mirrors the C-40 / D-5 hashing pattern (truncated blake3 -> u64) so
/// dashboards can group rows by skill identity without surfacing the raw
/// name. Reuses the workspace's existing `blake3` dep — no new external
/// dependency introduced.
#[must_use]
pub fn skill_name_hash(name: &str) -> u64 {
    let hash = blake3::hash(name.as_bytes());
    let bytes = hash.as_bytes();
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_name_hash_is_stable() {
        let a = skill_name_hash("planning_workflow");
        let b = skill_name_hash("planning_workflow");
        assert_eq!(a, b, "blake3 hash must be deterministic");
    }

    #[test]
    fn skill_name_hash_distinguishes_names() {
        let a = skill_name_hash("alpha");
        let b = skill_name_hash("beta");
        assert_ne!(a, b);
    }
}
