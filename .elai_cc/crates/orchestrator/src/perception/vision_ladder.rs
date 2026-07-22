//! J-8 T8 (VR-13 / VR-11) — data-driven vision-ladder table loaded from
//! `data/perception/vision_ladder.toml`.
//!
//! Mirrors the `sweet_spot.rs` loader pattern: parse the embedded TOML once
//! via `OnceLock`, expose typed lookups. The table is a J-9 seed handoff;
//! J-9 vision/OCR lane and J-1 leg-4 probe selection consume it later.

use std::sync::OnceLock;

const VISION_LADDER_TOML: &str =
    include_str!("../../../../data/perception/vision_ladder.toml");

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
struct LadderRowRaw {
    tier_key: String,
    max_vram_mb: u32,
    model_id: String,
    mmmu_design: f64,
    source_tag: String,
    #[serde(default)]
    platform_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
struct LadderFile {
    ladder: Vec<LadderRowRaw>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisionLadderRow {
    pub tier_key: String,
    pub max_vram_mb: u32,
    pub model_id: String,
    pub mmmu_design: f64,
    pub source_tag: String,
    pub platform_tags: Vec<String>,
}

static TABLE: OnceLock<Vec<VisionLadderRow>> = OnceLock::new();

fn table() -> &'static [VisionLadderRow] {
    TABLE.get_or_init(|| {
        let parsed: LadderFile = toml::from_str(VISION_LADDER_TOML)
            .expect("vision_ladder.toml must parse");
        parsed
            .ladder
            .into_iter()
            .map(|r| VisionLadderRow {
                tier_key: r.tier_key,
                max_vram_mb: r.max_vram_mb,
                model_id: r.model_id,
                mmmu_design: r.mmmu_design,
                source_tag: r.source_tag,
                platform_tags: r.platform_tags,
            })
            .collect()
    })
}

/// Full vision-ladder table.
// WIRE-DARK[J-8]
#[must_use]
pub fn rows() -> &'static [VisionLadderRow] {
    table()
}

/// Lookup a single row by `tier_key`.
// WIRE-DARK[J-8]
#[must_use]
pub fn row_by_tier_key(tier_key: &str) -> Option<&'static VisionLadderRow> {
    table().iter().find(|r| r.tier_key == tier_key)
}

/// The FastVLM row (VR-11 / JC-9 Apple-silicon rung).
// WIRE-DARK[J-8]
#[must_use]
pub fn fastvlm_row() -> Option<&'static VisionLadderRow> {
    table().iter().find(|r| r.model_id == "FastVLM")
}

/// Local (non-cloud, non-apple-silicon) rows that fit within `vram_mb`.
/// Rows with `max_vram_mb == 0` (cloud sentinel) are excluded.
// WIRE-DARK[J-8]
#[must_use]
pub fn local_rows_fitting_vram(vram_mb: u32) -> Vec<&'static VisionLadderRow> {
    table()
        .iter()
        .filter(|r| {
            !r.platform_tags.iter().any(|t| t == "cloud" || t == "apple_silicon")
                && r.max_vram_mb > 0
                && r.max_vram_mb <= vram_mb
        })
        .collect()
}
