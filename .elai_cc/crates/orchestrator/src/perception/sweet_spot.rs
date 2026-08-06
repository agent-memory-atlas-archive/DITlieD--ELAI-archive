//! J-8 T1 (VR-1) — data-driven sweet-spot table loaded from
//! `data/perception/sweet_spots.toml`.
//!
//! Replaces the hardcoded match arms in `preprocess.rs` with a TOML lookup.
//! The table is parsed once at first access via `OnceLock` and reused for
//! all subsequent lookups. The production call site is
//! `TargetModel::sweet_spot_longest_side_px` in `preprocess.rs`, which
//! delegates to [`longest_side_px`] here.
//!
//! J-77 S5 (I1/I2) — optional tiling fields `tile_px` + `tile_overlap`
//! are loaded alongside the existing resize fields. The production call
//! site is `preprocess()` in `preprocess.rs`, which delegates to
//! [`tile_px`] and [`tile_overlap`] here when building the preprocess
//! result. The owner-tile policy ([`owner_tile_index`]) is a pure fn
//! consumed by the preprocess path to assign each detected box to the
//! tile containing its center.
//!
//! HONEST CAVEAT: the 12.5% default overlap is a config default, NOT a
//! literature-settled optimum. Re-verify on model swaps; tune per-model
//! if measured deltas warrant (G-10 canary is the runtime sibling).

use std::sync::OnceLock;

const SWEET_SPOTS_TOML: &str =
    include_str!("../../../../data/perception/sweet_spots.toml");

/// Default tile overlap fraction when the TOML row omits `tile_overlap`.
/// 12.5% — HONEST CAVEAT: a config default, NOT a literature-settled
/// optimum (re-verify on model swaps; G-10 canary is the runtime sibling).
const DEFAULT_TILE_OVERLAP: f32 = 0.125;

#[derive(Debug, Clone, serde::Deserialize)]
struct SweetSpotRow {
    model_key: String,
    longest_side_px: u32,
    native_res: bool,
    #[serde(default)]
    tile_px: u32,
    #[serde(default = "default_tile_overlap")]
    tile_overlap: f32,
}

fn default_tile_overlap() -> f32 {
    DEFAULT_TILE_OVERLAP
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SweetSpotFile {
    sweet_spot: Vec<SweetSpotRow>,
}

#[derive(Debug, Clone)]
struct SweetSpotEntry {
    longest_side_px: u32,
    native_res: bool,
    tile_px: u32,
    tile_overlap: f32,
}

static TABLE: OnceLock<std::collections::HashMap<String, SweetSpotEntry>> =
    OnceLock::new();

fn table() -> &'static std::collections::HashMap<String, SweetSpotEntry> {
    TABLE.get_or_init(|| {
        let parsed: SweetSpotFile = toml::from_str(SWEET_SPOTS_TOML)
            .expect("sweet_spots.toml must parse");
        let mut map = std::collections::HashMap::with_capacity(parsed.sweet_spot.len());
        for row in parsed.sweet_spot {
            map.insert(row.model_key, SweetSpotEntry {
                longest_side_px: row.longest_side_px,
                native_res: row.native_res,
                tile_px: row.tile_px,
                tile_overlap: row.tile_overlap,
            });
        }
        map
    })
}

/// Longest-side sweet-spot edge in pixels for the given model key.
/// Returns `None` when the model is not in the table.
///
/// EXTERNAL SRC-3 / HD-1 — design rationale only; every shipped number is
/// ELAI-measured (re-verify on model swaps).
#[must_use]
pub fn longest_side_px(model_key: &str) -> Option<u32> {
    table().get(model_key).map(|e| e.longest_side_px)
}

/// Whether the given model key uses native-resolution passthrough (no
/// resize). Returns `None` when the model is not in the table.
#[must_use]
pub fn native_res(model_key: &str) -> Option<bool> {
    table().get(model_key).map(|e| e.native_res)
}

/// Tile edge in pixels for the given model key. `0` means no tiling
/// (whole-image passthrough). Returns `None` when the model is not in
/// the table.
///
/// J-77 S5 (I1/I2) — config only; the sidecar tiler (T5) consumes this
/// value when the preprocess path builds the tile grid.
#[must_use]
pub fn tile_px(model_key: &str) -> Option<u32> {
    table().get(model_key).map(|e| e.tile_px)
}

/// Tile overlap fraction in `[0.0, 1.0]` for the given model key.
/// Returns the 12.5% default when the model is not in the table OR the
/// row omits `tile_overlap`. HONEST CAVEAT: 12.5% is a config default,
/// NOT a literature-settled optimum.
#[must_use]
pub fn tile_overlap(model_key: &str) -> f32 {
    table()
        .get(model_key)
        .map(|e| e.tile_overlap)
        .unwrap_or(DEFAULT_TILE_OVERLAP)
}

/// Owner-tile policy: a box is owned by the tile containing its CENTER.
/// Overlap rescues only truncated boxes (a box whose center is in tile A
/// but whose edges spill into tile B is owned by A; B's overlap region
/// captures the spillover).
///
/// `bbox` is `[x_min, y_min, x_max, y_max]` in image pixel coordinates.
/// `tile_origins` is a slice of `(origin_x, origin_y)` for each tile in
/// the grid. `tile_px` is the tile edge length. Returns the index of the
/// tile whose origin rectangle contains the box center, or `None` when
/// no tile contains the center (the center is outside the tiled region).
///
/// J-77 S5 (I1/I2) — pure fn consumed by the preprocess path when
/// assigning detected boxes to tiles. The live caller is the sidecar
/// tiler (T5, sidecar-gated W-10); the Rust-side `preprocess()` skeleton
/// does not yet perform tile assignment.
#[must_use]
pub fn owner_tile_index(
    bbox: [u32; 4],
    tile_origins: &[(u32, u32)],
    tile_px: u32,
) -> Option<usize> {
    if tile_px == 0 || tile_origins.is_empty() {
        return None;
    }
    let [x_min, y_min, x_max, y_max] = bbox;
    let cx = x_min.saturating_add(x_max) / 2;
    let cy = y_min.saturating_add(y_max) / 2;
    for (i, &(ox, oy)) in tile_origins.iter().enumerate() {
        if cx >= ox && cx < ox.saturating_add(tile_px)
            && cy >= oy && cy < oy.saturating_add(tile_px)
        {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tile_overlap_is_0_125() {
        assert_eq!(tile_overlap("claude"), 0.125);
        assert_eq!(tile_overlap("gpt"), 0.125);
        assert_eq!(tile_overlap("qwen_vl"), 0.125);
        assert_eq!(tile_overlap("gemini"), 0.125);
    }

    #[test]
    fn tile_overlap_unknown_model_returns_default() {
        assert_eq!(tile_overlap("nonexistent_model"), 0.125);
    }

    #[test]
    fn tile_px_known_models() {
        assert_eq!(tile_px("claude"), Some(0));
        assert_eq!(tile_px("gpt"), Some(0));
        assert_eq!(tile_px("qwen_vl"), Some(0));
        assert_eq!(tile_px("gemini"), Some(0));
    }

    #[test]
    fn tile_px_unknown_model_returns_none() {
        assert!(tile_px("nonexistent_model").is_none());
    }

    #[test]
    fn owner_tile_index_center_in_tile() {
        let origins = vec![(0, 0), (100, 0), (0, 100), (100, 100)];
        // bbox center at (50, 50) -> tile 0
        assert_eq!(
            owner_tile_index([0, 0, 100, 100], &origins, 100),
            Some(0)
        );
        // bbox center at (150, 50) -> tile 1
        assert_eq!(
            owner_tile_index([100, 0, 200, 100], &origins, 100),
            Some(1)
        );
        // bbox center at (50, 150) -> tile 2
        assert_eq!(
            owner_tile_index([0, 100, 100, 200], &origins, 100),
            Some(2)
        );
        // bbox center at (150, 150) -> tile 3
        assert_eq!(
            owner_tile_index([100, 100, 200, 200], &origins, 100),
            Some(3)
        );
    }

    #[test]
    fn owner_tile_index_truncated_box_owned_by_center_tile() {
        let origins = vec![(0, 0), (100, 0)];
        // Box spans both tiles but center at (60, 50) is in tile 0
        assert_eq!(
            owner_tile_index([40, 0, 120, 100], &origins, 100),
            Some(0)
        );
        // Box spans both tiles but center at (140, 50) is in tile 1
        assert_eq!(
            owner_tile_index([80, 0, 160, 100], &origins, 100),
            Some(1)
        );
    }

    #[test]
    fn owner_tile_index_center_outside_returns_none() {
        let origins = vec![(0, 0)];
        // center at (200, 200) is outside the single tile
        assert_eq!(
            owner_tile_index([150, 150, 250, 250], &origins, 100),
            None
        );
    }

    #[test]
    fn owner_tile_index_zero_tile_px_returns_none() {
        let origins = vec![(0, 0)];
        assert_eq!(owner_tile_index([0, 0, 50, 50], &origins, 0), None);
    }

    #[test]
    fn owner_tile_index_empty_origins_returns_none() {
        assert_eq!(owner_tile_index([0, 0, 50, 50], &[], 100), None);
    }
}
