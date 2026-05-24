//! Empire repair pipeline — types + heuristic detectors for the 3 verified
//! Empire issues (CircularDep / HardImport / StdlibShadow) documented in
//! empire.txt section 2 2026-04-08 correction log.
//!
//! Scope of this pass:
//!   - Types are real and serde-stable so the SSB `empire_repair_proposed`
//!     row payload can carry them once the activation patch lands.
//!   - The `detect_repair_kind` heuristic is regex/substring-based — it does
//!     NOT walk an AST. AST-grade detection (tree-sitter-python in the
//!     activation patch) is annotated EMPIRE-PENDING below. The heuristic
//!     is correct enough to act as a smoke test against captured Empire
//!     symbol paths but is not the production detector.
//!
//! Why heuristics now: the real Empire codebase is not accessible from this
//! workspace (per phase-4J.txt EMPIRE-PENDING gate). Building a tree-sitter
//! walker against fixtures we do not have would be guess-work. The
//! heuristic surface (a) compiles, (b) round-trips serde, (c) unit-tests
//! cleanly, and (d) is one swap away from the production AST detector when
//! the activation patch lands.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Issue / fix classification
// ---------------------------------------------------------------------------

/// The 3 verified Empire issue classes per empire.txt section 2.
///
/// `Other(String)` is reserved for future-discovered issues so the enum is
/// non-exhaustive against new categories without a SCHEMA_VERSION bump.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "label")]
pub enum EmpireRepairKind {
    /// Import-graph cycle. Two or more modules import each other (directly
    /// or transitively). Heuristic indicator: path contains a known
    /// circular module marker (e.g. `bus.py` <-> `engine.py` per
    /// empire.txt correction log).
    CircularDep,
    /// Hardcoded path import. A module imports via an absolute filesystem
    /// path string (e.g. `sys.path.insert(0, "/opt/empire/lib")`) instead
    /// of a package-relative import. Heuristic indicator: path string
    /// contains `sys.path` / `importlib.util.spec_from_file_location`.
    HardImport,
    /// A user module shadows a stdlib module name (e.g. `email.py`,
    /// `string.py`, `types.py`) and breaks downstream imports. Heuristic
    /// indicator: filename matches one of the well-known stdlib names.
    StdlibShadow,
    /// Future-discovered issue category.
    Other(String),
}

impl EmpireRepairKind {
    /// SSB wire label for the `fix_kind` field on `empire_repair_proposed`
    /// rows. Single-token snake_case so the dashboard can filter without
    /// parsing structured JSON.
    pub fn as_wire_label(&self) -> &str {
        match self {
            EmpireRepairKind::CircularDep => "circular_dep",
            EmpireRepairKind::HardImport => "hard_import",
            EmpireRepairKind::StdlibShadow => "stdlib_shadow",
            EmpireRepairKind::Other(label) => label.as_str(),
        }
    }
}

/// One repair proposal the Empire repair pipeline can surface. The dashboard
/// renders these in the EmpireBridgePanel "repair pipeline results" pane
/// when the activation patch lands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmpireRepairProposal {
    pub kind: EmpireRepairKind,
    /// Repository-relative path the proposal applies to (e.g.
    /// `workspace/factory-brain/lib/ipc.py`).
    pub location: String,
    /// Short prose summary of the proposed fix (one sentence, no markdown).
    pub fix_summary: String,
}

/// Error surface for the repair detection path. Held separate from
/// `EmpireBridgeError` so callers can match on detection failures without
/// pulling in the bridge's connection-state taxonomy.
#[derive(Debug, Clone, Error)]
pub enum EmpireRepairError {
    #[error("empty symbol path; cannot classify a repair target")]
    EmptyPath,
}

// ---------------------------------------------------------------------------
// Heuristic detectors
// ---------------------------------------------------------------------------

/// List of stdlib module basenames the StdlibShadow heuristic flags. The
/// list is conservative (only the most commonly shadowed names per CPython
/// stdlib audit) so the detector stays under the 5% false-positive ceiling
/// the spec requires for the dashboard surface to be useful.
const STDLIB_SHADOW_NAMES: &[&str] = &[
    "email", "string", "types", "select", "io", "queue", "code", "copy",
    "test", "tokenize", "platform", "abc", "ast", "calendar", "csv", "html",
    "json", "logging", "socket", "ssl", "sys", "tempfile", "time", "uuid",
];

/// Heuristic dispatcher that classifies a symbol path into one of the 3
/// verified Empire issue classes.
///
/// Returns `None` when the symbol path does not match any known
/// fingerprint. `EmpireRepairError::EmptyPath` for the empty-input case so
/// callers must consciously route around it (no silent swallow).
///
/// EMPIRE-PENDING: in the activation patch this function delegates to a
/// tree-sitter-python AST walker against the actual Empire source tree.
/// The heuristic stays as a fast-path / smoke-test layer; the AST walker
/// overrides whenever it returns a non-`Other` classification.
pub fn detect_repair_kind(symbol_path: &str) -> Result<Option<EmpireRepairKind>, EmpireRepairError> {
    let p = symbol_path.trim();
    if p.is_empty() {
        return Err(EmpireRepairError::EmptyPath);
    }
    let lower = p.to_ascii_lowercase();

    // CircularDep: any path the empire.txt correction log flags as part of
    // a known cycle (bus.py / engine.py / hooks.py / router.py — the four
    // modules listed in the 2026-04-08 entry).
    if lower.contains("bus.py")
        || lower.contains("engine.py")
        || lower.contains("hooks.py")
        || lower.contains("router.py")
    {
        return Ok(Some(EmpireRepairKind::CircularDep));
    }

    // HardImport: the Empire spec lists `sys.path` mutation and absolute
    // `importlib` paths as the only HardImport patterns. Both are visible
    // in the import statement text, which is what the dashboard passes
    // through as the symbol_path for HardImport reports.
    if lower.contains("sys.path") || lower.contains("importlib.util.spec_from_file_location") {
        return Ok(Some(EmpireRepairKind::HardImport));
    }

    // StdlibShadow: the user module basename matches a well-known stdlib
    // module name. We split on `/` AND `\` so the heuristic survives both
    // POSIX and Windows-style paths.
    if let Some(basename) = p.rsplit(|c| c == '/' || c == '\\').next() {
        if let Some(stem) = basename.strip_suffix(".py") {
            if STDLIB_SHADOW_NAMES.contains(&stem) {
                return Ok(Some(EmpireRepairKind::StdlibShadow));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_dep_fires_on_known_module_pair() {
        let kind = detect_repair_kind("workspace/factory-brain/core/bus.py")
            .expect("non-empty")
            .expect("classification");
        assert_eq!(kind, EmpireRepairKind::CircularDep);
        assert_eq!(kind.as_wire_label(), "circular_dep");
    }

    #[test]
    fn hard_import_fires_on_sys_path_marker() {
        let kind = detect_repair_kind("import sys; sys.path.insert(0, '/opt/empire/lib')")
            .expect("non-empty")
            .expect("classification");
        assert_eq!(kind, EmpireRepairKind::HardImport);
        assert_eq!(kind.as_wire_label(), "hard_import");
    }

    #[test]
    fn stdlib_shadow_fires_on_email_py() {
        let kind = detect_repair_kind("workspace/factory-brain/email.py")
            .expect("non-empty")
            .expect("classification");
        assert_eq!(kind, EmpireRepairKind::StdlibShadow);
        assert_eq!(kind.as_wire_label(), "stdlib_shadow");
    }

    #[test]
    fn stdlib_shadow_survives_windows_style_path() {
        let kind = detect_repair_kind("workspace\\factory-brain\\json.py")
            .expect("non-empty")
            .expect("classification");
        assert_eq!(kind, EmpireRepairKind::StdlibShadow);
    }

    #[test]
    fn unrelated_module_returns_none() {
        let result =
            detect_repair_kind("workspace/factory-brain/lib/my_custom_helper.py").expect("ok");
        assert!(result.is_none());
    }

    #[test]
    fn empty_path_returns_error() {
        let result = detect_repair_kind("   ");
        assert!(matches!(result, Err(EmpireRepairError::EmptyPath)));
    }

    #[test]
    fn proposal_round_trips_serde() {
        let proposal = EmpireRepairProposal {
            kind: EmpireRepairKind::CircularDep,
            location: "workspace/factory-brain/core/bus.py".into(),
            fix_summary: "extract shared types to neutral module to break bus<->engine cycle"
                .into(),
        };
        let json = serde_json::to_string(&proposal).expect("serialize");
        let back: EmpireRepairProposal = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(proposal, back);
    }

    #[test]
    fn other_kind_serde_round_trips_with_label() {
        let kind = EmpireRepairKind::Other("future_pattern".into());
        let json = serde_json::to_string(&kind).expect("serialize");
        let back: EmpireRepairKind = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(kind, back);
        assert_eq!(kind.as_wire_label(), "future_pattern");
    }
}
