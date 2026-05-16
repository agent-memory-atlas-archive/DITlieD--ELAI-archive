//! Knowledge-crate runtime configuration subset.
//!
//! The knowledge crate cannot depend on the CLI crate (it would be circular).
//! This module defines [`KnowledgeRuntimeConfig`] which covers only the
//! `[knowledge]` TOML table. It is parsed directly from `elai_cc.toml` by
//! [`crate::artifacts::manifest::ArtifactManifest::load`].
//!
//! The CLI crate's `ElaiRuntimeConfig` may optionally embed a `knowledge:
//! KnowledgeRuntimeConfig` field; they share the same TOML schema but are
//! deserialized independently.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ArtifactsConfig
// ---------------------------------------------------------------------------

/// One [[knowledge.artifacts]] entry in elai_cc.toml.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ArtifactItemToml {
    pub name: String,
    pub path: String,
    pub description: String,
    /// When true, this artifact is pre-loaded into GRC verify_claim context.
    pub include_in_grc: bool,
}

// ---------------------------------------------------------------------------
// KnowledgeRuntimeConfig
// ---------------------------------------------------------------------------

/// The `[knowledge]` TOML section from elai_cc.toml.
///
/// All fields have `#[serde(default)]` so an absent file or absent section
/// degrades gracefully to empty/default.
///
/// `artifacts` maps directly to a `[[knowledge.artifacts]]` array-of-tables.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct KnowledgeRuntimeConfig {
    #[serde(default)]
    pub artifacts: Vec<ArtifactItemToml>,
}

// ---------------------------------------------------------------------------
// TopLevelWrapper (for parsing elai_cc.toml → [knowledge] only)
// ---------------------------------------------------------------------------

/// Thin TOML wrapper to extract only the `[knowledge]` table from
/// `elai_cc.toml`. The other top-level sections (tools, output, actions,
/// inference, agents, cloud) are ignored via `#[serde(default)]` on the
/// missing-section-friendly deserialization path.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct ElaiTomlKnowledgeSlice {
    pub knowledge: KnowledgeRuntimeConfig,
}
