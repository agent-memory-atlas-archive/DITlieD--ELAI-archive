// B-34 — Entry-point detection for the symbol graph.
//
// Sub-modules:
//   names  — conventional entry-point name table (ported from SocratiCode constants.ts:151-166)
//             and conventional entry-point filename set.
//   types  — DetectedEntryPoint, EntryPointReason, EntryPointSummary.
//             NOTE: `types::DetectedEntryPoint` is DISTINCT from
//             `super::types::EntryPoint` (B-29 type). Neither is re-exported
//             at the symbol_graph module root (see mod.rs:9 name-collision note).
//   detect — detect_entry_points (pure fn over reverse_index + payloads),
//             find_symbol_at helper.

pub mod detect;
pub mod names;
pub mod types;

pub use detect::{detect_entry_points, find_symbol_at};
pub use names::{entry_point_filenames, entry_point_names_per_language};
pub use types::{DetectedEntryPoint, EntryPointReason, EntryPointSummary};
