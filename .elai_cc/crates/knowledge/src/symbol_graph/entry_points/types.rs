// B-34 — Entry-point detection types.
//
// `DetectedEntryPoint` is DISTINCT from `super::super::types::EntryPoint` (B-29).
// Both exist in the same crate but at different module paths. Neither is
// re-exported at the symbol_graph module root (name-collision note in mod.rs:9).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Why a file or symbol was classified as an entry point.
///
/// Each variant corresponds to one heuristic from `detect::detect_entry_points`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "detail")]
pub enum EntryPointReason {
    /// No other indexed file imports this file, but it has outgoing calls
    /// (Heuristic 1: graph orphan with outgoing edges).
    Orphan,
    /// Symbol name matches a conventional entry-point name for its language
    /// (Heuristic 2: well-known name table from SocratiCode constants.ts:151-166).
    /// Inner string is the matched name (e.g. "main", "Main", "__main__").
    WellKnownName(String),
    /// An outgoing call from this symbol targets a known HTTP-router or route
    /// method (Heuristic 3: framework route detection).
    /// Inner string is the framework reason (e.g. "framework:http-get").
    FrameworkRoute(String),
    /// Symbol name matches a test-function pattern (Heuristic 4).
    Test,
}

impl EntryPointReason {
    /// A stable, compact key used for deduplication (`id::reason_key`).
    pub fn key(&self) -> String {
        match self {
            Self::Orphan => "orphan".to_string(),
            Self::WellKnownName(name) => format!("well_known:{name}"),
            Self::FrameworkRoute(r) => r.clone(),
            Self::Test => "test".to_string(),
        }
    }
}

/// A single detected entry point (symbol or file).
///
/// One `DetectedEntryPoint` per (id, reason) pair — same symbol can appear
/// multiple times if it matches multiple heuristics, but never twice for the
/// same heuristic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedEntryPoint {
    /// Stable unique identifier: symbol id from `SymbolNode::id` (heuristics 2–4)
    /// or file path (heuristic 1 orphan).
    pub id: String,
    /// Human-readable name: qualified symbol name or file path.
    pub name: String,
    /// Source file containing this entry point (repo-relative path).
    pub file: String,
    /// 1-based start line of the symbol, if known (None for orphan file entries).
    pub line: Option<u32>,
    /// Classification reason.
    pub reason: EntryPointReason,
}

/// Aggregate summary across all detected entry points.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointSummary {
    /// Count per reason key (e.g. "orphan" -> 3, "well_known:main" -> 2).
    pub by_reason: BTreeMap<String, u32>,
    /// Total entry-point count (sum of all reason values).
    pub total: u32,
}

impl EntryPointSummary {
    /// Build a summary from a slice of detected entry points.
    pub fn from_entries(entries: &[DetectedEntryPoint]) -> Self {
        let mut by_reason: BTreeMap<String, u32> = BTreeMap::new();
        for e in entries {
            *by_reason.entry(e.reason.key()).or_insert(0) += 1;
        }
        let total: u32 = entries.len().try_into().unwrap_or(u32::MAX);
        Self { by_reason, total }
    }
}
