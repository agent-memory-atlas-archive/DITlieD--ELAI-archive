//! E-45 — host-side cross-language drift detector.
//!
//! Diffs the Rust orchestrator's serde-serialized boundary structs against
//! their Python sidecar / Tauri TS mirrors using a pair registry loaded
//! from `.elai_cc/cross-lang-pairs.toml`. The detector is a presence /
//! shape lint over the hand-authored TS mirror (per D-10 DECISION-8) and
//! the Python sidecar's Pydantic schemas (C section of this plan); it
//! does NOT auto-generate code.
//!
//! Surface:
//! * [`detector::DriftDetector`] — `detect_pairs(repo_root)` walks the
//!   pair registry and emits one row per pair scan + zero-or-more drift
//!   items per pair. The 3 E-45 SSB action types
//!   (`CrossLangDriftDetected`, `CrossLangPairScanned`,
//!   `SchemaVersionMismatch`) are emitted by the caller (CLI / agent),
//!   not by the detector itself — this keeps the module free of SSB
//!   handle ownership.
//! * [`pair_registry::PairRegistry`] — TOML loader for the registry.
//! * [`types::DriftItem`] / [`types::DriftKind`] / [`types::DriftPair`]
//!   — pure data types consumed by callers + downstream renderers.
//!
//! Design grounding: D-10 DECISION-8 (hand-authored TS mirror over
//! ts-rs codegen). The E-45 plan's Tasks 1+2 (ts-rs codegen) are
//! intentionally DEFERRED — this module is the substitute that lints
//! the hand-authored mirror by presence + serde-shape probes.

pub mod detector;
pub mod pair_registry;
pub mod types;

pub use detector::{DriftDetector, DriftDetectorError};
pub use pair_registry::{PairRegistry, PairRegistryError};
pub use types::{DriftItem, DriftKind, DriftPair, PairKind, Severity};
