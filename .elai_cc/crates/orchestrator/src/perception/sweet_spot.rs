//! J-8 T1 (VR-1) — data-driven sweet-spot table loaded from
//! `data/perception/sweet_spots.toml`.
//!
//! Replaces the hardcoded match arms in `preprocess.rs` with a TOML lookup.
//! The table is parsed once at first access via `OnceLock` and reused for
//! all subsequent lookups. The production call site is
//! `TargetModel::sweet_spot_longest_side_px` in `preprocess.rs`, which
//! delegates to [`longest_side_px`] here.

use std::sync::OnceLock;

const SWEET_SPOTS_TOML: &str =
    include_str!("../../../../data/perception/sweet_spots.toml");

#[derive(Debug, Clone, serde::Deserialize)]
struct SweetSpotRow {
    model_key: String,
    longest_side_px: u32,
    native_res: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SweetSpotFile {
    sweet_spot: Vec<SweetSpotRow>,
}

#[derive(Debug, Clone)]
struct SweetSpotEntry {
    longest_side_px: u32,
    native_res: bool,
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
