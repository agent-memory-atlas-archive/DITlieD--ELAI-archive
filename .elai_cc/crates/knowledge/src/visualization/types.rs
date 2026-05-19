// B-33 — Visualization data types for the dashboard graph window.
//
// All shape decisions mirror the SocratiCode VizData contract
// (graph-visualize-html.ts:45-92): MAX_SYMBOLS=20_000, MAX_EDGES=60_000,
// MAX_SYMS_PER_FILE=200; symbols_by_file is a map keyed by file path;
// symbol_mode carries Full/Capped(reason)/Omitted(reason).
//
// IMPORTANT INVARIANTS (locked by tests/visualization_types.rs +
// contracts/tests/b33_graph_viewer_contract.rs):
//   - All maps use BTreeMap (deterministic iteration; C2 BTREEMAP gotcha).
//   - SymbolMode externally-tagged so the dashboard sees
//     `{"Full":null}` / `{"Capped":{"reason":"..."}}` /
//     `{"Omitted":{"reason":"..."}}` in JSON.
//   - SymbolMode::Capped and ::Omitted MUST carry a non-empty reason
//     (D6 gating rule; commands.rs constructors are the producer).
//   - VizEdgeKind is a closed enum: Import / DynamicImport / ReExport.
//
// Confidence is referenced by qualified path
// (`crate::symbol_graph::types::Confidence`) per the no-re-export rule
// recorded in symbol_graph::mod.rs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants — match SocratiCode plan-locked caps
// ---------------------------------------------------------------------------

/// Hard cap for symbol-mode payloads: when the symbol_count exceeds this
/// the producer returns `SymbolMode::Omitted{reason}` and the dashboard
/// disables the symbol-mode toggle.
pub const MAX_VIZ_SYMBOLS: usize = 20_000;

/// Hard cap for symbol edges: when total edges exceed this the producer
/// returns `SymbolMode::Capped{reason}` and the dashboard surfaces the hint.
pub const MAX_VIZ_EDGES: usize = 60_000;

/// Per-file cap for sidebar symbol lists. The producer trims to this when
/// streaming `symbols_by_file` so a single huge file never balloons the JSON.
pub const MAX_VIZ_SYMS_PER_FILE: usize = 200;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Closed set of edge kinds the dashboard understands. Anything that does
/// not map to one of these is dropped at the producer rather than passed
/// through as a free-form string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VizEdgeKind {
    Import,
    DynamicImport,
    ReExport,
}

/// Three-tier symbol availability indicator.
///
/// `Capped` = symbol payload was returned but trimmed to fit caps;
/// `Omitted` = symbol payload was withheld entirely. Both MUST carry a
/// populated `reason` string per WIRING_CONTRACT stub_ban.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolMode {
    Full,
    Capped { reason: String },
    Omitted { reason: String },
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Top-level project metadata block surfaced in the GraphWindow header.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VizProject {
    /// Display name for the dashboard header (defaults to project_id).
    pub name: String,
    /// Total number of files indexed.
    pub files: usize,
    /// Total number of file-import edges.
    pub edges: usize,
    /// Total symbol count, when symbol mode resolved to Full or Capped.
    pub symbols: Option<usize>,
    /// Total call edge count, when symbol mode resolved to Full or Capped.
    pub call_edges: Option<usize>,
    /// Wall-clock build timestamp from `SymbolGraphMeta::built_at`
    /// (ms-since-epoch). The Mermaid generator does NOT call SystemTime —
    /// the value is supplied by the caller (ClockProvider boundary).
    pub built_at: Option<i64>,
}

/// One file-level node in the dashboard graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VizFile {
    /// Stable id (typically the same as file path).
    pub id: String,
    /// Display label (basename / relative path).
    pub label: String,
    /// Language identifier for classDef coloring (rust/python/ts/svelte/go/jvm/csharp/other).
    pub language: String,
    /// Outbound dependency count (file-imports out).
    pub deps: usize,
    /// Inbound dependent count (file-imports in).
    pub dependents: usize,
    /// Number of symbols defined inside this file.
    pub symbol_count: usize,
    /// C-24 (DRIFT-7 Path A): decomposition-level discriminator. Set by
    /// the decomposition-view builder to one of `"concept"`, `"phase"`,
    /// `"step"`, or `"baby_step"` so the GraphWindow cytoscape selectors
    /// (`node[level = "Concept"]` etc.) can pick the right shape. File +
    /// symbol modes leave this `None`. Additive `Option<String>` so
    /// existing serde-derived fixtures still decode without an explicit
    /// SCHEMA_VERSION bump.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    /// C-24 (DRIFT-7 Path A): execution-state discriminator for cytoscape
    /// `.status-pending|running|passed|failed|blocked` classes. Sourced
    /// from per-baby-step status (driven by the SSB `FileWrite` payload
    /// path / spawn_loop dispatcher). Today the decomposition builder
    /// leaves this `None` — the dashboard selectors EXIST so the moment
    /// a per-node status source is wired, visual feedback ships without
    /// further GraphWindow changes. See workflow/reasoning/C-24-decisions.md
    /// DECISION-6 for the deferred status-source rationale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// One file-level edge in the dashboard graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VizFileEdge {
    pub source: String,
    pub target: String,
    pub kind: VizEdgeKind,
    /// True when the edge participates in a strongly-connected component
    /// (a cycle). Mermaid renders these as red dotted lines.
    pub cyclic: bool,
}

/// One symbol node in symbol mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VizSymbol {
    pub id: String,
    pub file: String,
    pub name: String,
    pub qualified_name: String,
    /// Free-form kind label (e.g. "Function", "Method"); the producer fills
    /// from `symbol_graph::types::SymbolKind::*` via Display.
    pub kind: String,
    pub line: u32,
}

/// One call edge between symbols.
///
/// `confidence` is the qualified `crate::symbol_graph::types::Confidence`
/// per the no-re-export rule. Tag is externally serialised, e.g.
/// `"confidence": "Local"` or `"confidence": "Unique"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VizSymbolEdge {
    pub source: String,
    pub target: String,
    pub confidence: crate::symbol_graph::types::Confidence,
}

/// Top-level payload returned by `get_graph_view_data` and consumed by the
/// dashboard's GraphWindow / Mermaid exporter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VizData {
    pub project: VizProject,
    pub files: Vec<VizFile>,
    pub file_edges: Vec<VizFileEdge>,
    pub symbols: Vec<VizSymbol>,
    pub symbol_edges: Vec<VizSymbolEdge>,
    /// Per-file sidebar payload. BTreeMap keeps iteration deterministic so
    /// dashboard tests / Mermaid output stay stable across runs.
    pub symbols_by_file: BTreeMap<String, Vec<VizSymbol>>,
    pub symbol_mode: SymbolMode,
}

impl VizData {
    /// Construct an empty payload with the given project name + symbol mode
    /// (used by the "no-index-yet" branch in the dashboard commands).
    pub fn empty(project_name: String, mode: SymbolMode) -> Self {
        Self {
            project: VizProject {
                name: project_name,
                files: 0,
                edges: 0,
                symbols: None,
                call_edges: None,
                built_at: None,
            },
            files: Vec::new(),
            file_edges: Vec::new(),
            symbols: Vec::new(),
            symbol_edges: Vec::new(),
            symbols_by_file: BTreeMap::new(),
            symbol_mode: mode,
        }
    }
}
