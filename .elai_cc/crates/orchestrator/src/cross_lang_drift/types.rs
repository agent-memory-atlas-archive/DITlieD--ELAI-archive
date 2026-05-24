//! E-45 — pure-data types for cross-language drift detection.
//!
//! Kept separate from `detector.rs` so contract tests + the CLI surface
//! can depend on the types without pulling in std::fs or toml.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Categorical kind of cross-language pair. The variant set is fixed
/// and corresponds 1:1 with the rows in `cross-lang-pairs.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PairKind {
    /// Rust orchestrator SSB types <-> Python sidecar Pydantic schemas.
    /// Spec: orchestrator/src/ssb.rs <-> python/elai_inference/schemas/.
    SidecarOrchestratorSchemas,
    /// Tauri command Rust types <-> Svelte TS stub.
    /// Spec: dashboard-v2/src-tauri/src/commands.rs <-> dashboard-v2/src/lib/api/stub.ts.
    TauriCommandsTsStub,
    /// Tauri lib.rs <-> dashboard-v2 types.ts (hand-authored mirror per D-10 DECISION-8).
    TauriTypesLib,
    /// Anthropic provider Rust types <-> sidecar /v1/messages handler.
    SidecarAnthropicProvider,
    /// Runtime client Rust struct <-> sidecar server.
    SidecarRuntimeClient,
    /// Shared IPC types <-> dashboard api/*.ts (hand-authored mirror).
    IpcTypesDashboard,
}

impl PairKind {
    /// Stable wire-tag string for the pair (used by the SSB action type's
    /// pair_kind_hash field and by the CLI text renderer). 1:1 with the
    /// PairRegistry TOML `name` field.
    pub fn wire_tag(&self) -> &'static str {
        match self {
            Self::SidecarOrchestratorSchemas => "sidecar_orchestrator_schemas",
            Self::TauriCommandsTsStub => "tauri_commands_ts_stub",
            Self::TauriTypesLib => "tauri_types_lib",
            Self::SidecarAnthropicProvider => "sidecar_anthropic_provider",
            Self::SidecarRuntimeClient => "sidecar_runtime_client",
            Self::IpcTypesDashboard => "ipc_types_dashboard",
        }
    }

    /// Parse the registry `name` field back to a `PairKind`. Returns
    /// `None` for unknown names (the registry loader treats that as an
    /// error). No silent fallback.
    pub fn from_wire_tag(s: &str) -> Option<Self> {
        match s {
            "sidecar_orchestrator_schemas" => Some(Self::SidecarOrchestratorSchemas),
            "tauri_commands_ts_stub" => Some(Self::TauriCommandsTsStub),
            "tauri_types_lib" => Some(Self::TauriTypesLib),
            "sidecar_anthropic_provider" => Some(Self::SidecarAnthropicProvider),
            "sidecar_runtime_client" => Some(Self::SidecarRuntimeClient),
            "ipc_types_dashboard" => Some(Self::IpcTypesDashboard),
            _ => None,
        }
    }
}

/// Severity bucket for a single drift item. Maps to the SSB
/// `severity_max` u8 field at emission time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Renamed field, mismatched schema version, type widening that
    /// would silently corrupt the wire — drops/renames category.
    Critical = 3,
    /// Added field on one side without the other — breaks parse if the
    /// reader is strict. Recoverable via #[serde(default)] / Optional.
    Warning = 2,
    /// Doc-only / annotation drift; no on-wire impact.
    Info = 1,
}

impl Severity {
    /// Convert to the u8 carried by `CrossLangDriftDetectedPayload`.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Categorical drift event surfaced by `DriftDetector::diff_pair`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftKind {
    /// A field was added to the Rust side but is absent on the mirror.
    /// Carries the field name (best-effort: extracted from the serde
    /// shape probe, not a full Rust AST walk).
    TypeFieldAdded { field: String },
    /// A field was removed from the Rust side but the mirror still lists it.
    TypeFieldRemoved { field: String },
    /// A field name diverged (one side renamed without the other).
    TypeFieldRenamed { rust_name: String, mirror_name: String },
    /// The Rust SCHEMA_VERSION constant does not match the mirror's
    /// declared schema version (Python `SCHEMA_VERSION` literal or TS
    /// `SCHEMA_VERSION` const).
    SchemaVersionMismatch { rust_version: u32, mirror_version: u32 },
    /// One side of the pair is missing entirely (the file does not exist).
    PairSideMissing { missing_side: PathBuf },
}

/// A single drift event with location + severity. Produced by the
/// detector and aggregated into the `drift_count` / `severity_max`
/// fields of `CrossLangDriftDetectedPayload`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftItem {
    pub kind: DriftKind,
    pub location: PathBuf,
    pub severity: Severity,
}

impl DriftItem {
    /// Helper constructor used by the detector. Centralizes severity
    /// assignment so callers cannot drift on the mapping.
    pub fn new(kind: DriftKind, location: PathBuf) -> Self {
        let severity = match &kind {
            DriftKind::TypeFieldRenamed { .. } => Severity::Critical,
            DriftKind::SchemaVersionMismatch { .. } => Severity::Critical,
            DriftKind::PairSideMissing { .. } => Severity::Critical,
            DriftKind::TypeFieldAdded { .. } => Severity::Warning,
            DriftKind::TypeFieldRemoved { .. } => Severity::Warning,
        };
        Self {
            kind,
            location,
            severity,
        }
    }
}

/// A single entry in the pair registry. Represents one cross-language
/// boundary that the detector should walk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftPair {
    pub kind: PairKind,
    /// Path (relative to repo root) of the Rust-side authoritative source.
    pub rust_path: PathBuf,
    /// Path (relative to repo root) of the mirror side (Python or TS).
    pub mirror_path: PathBuf,
    /// Human-readable note from the TOML registry; surfaced in CLI output.
    pub note: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_kind_wire_tag_round_trip() {
        let kinds = [
            PairKind::SidecarOrchestratorSchemas,
            PairKind::TauriCommandsTsStub,
            PairKind::TauriTypesLib,
            PairKind::SidecarAnthropicProvider,
            PairKind::SidecarRuntimeClient,
            PairKind::IpcTypesDashboard,
        ];
        for k in kinds {
            let tag = k.wire_tag();
            let parsed = PairKind::from_wire_tag(tag);
            assert_eq!(parsed, Some(k), "round-trip failed for {tag}");
        }
    }

    #[test]
    fn pair_kind_unknown_returns_none() {
        assert!(PairKind::from_wire_tag("nonexistent_pair").is_none());
    }

    #[test]
    fn drift_item_severity_is_deterministic() {
        let critical = DriftItem::new(
            DriftKind::TypeFieldRenamed {
                rust_name: "a".into(),
                mirror_name: "b".into(),
            },
            PathBuf::from("x"),
        );
        assert_eq!(critical.severity, Severity::Critical);

        let warning = DriftItem::new(
            DriftKind::TypeFieldAdded { field: "f".into() },
            PathBuf::from("x"),
        );
        assert_eq!(warning.severity, Severity::Warning);
    }
}
