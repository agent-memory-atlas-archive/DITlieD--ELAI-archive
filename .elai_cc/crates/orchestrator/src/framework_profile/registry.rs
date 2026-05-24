//! E-12 Tier-B — FrameworkProfileRegistry.
//!
//! Loads framework profile manifests from `workflow/archetypes/`
//! (canonical scaffold home per E-11.A, NOT a parallel
//! `.claude/tools/framework-conventions/` directory — Tier-A decision
//! D9 closed). Resolves `ProjectClass -> FrameworkProfile` via the
//! `PROJECT_CLASS_TO_FRAMEWORK` table.
//!
//! Wire-dark posture: if `workflow/archetypes/` is missing or empty,
//! the registry returns an empty map and `get_framework_profile`
//! falls back to `FrameworkProfile::generic()`. The 4 representative
//! manifests authored in this session (nextjs15.toml, svelte5.toml,
//! fastapi-modern.toml, hono.toml) cover the most-common
//! PROJECT_CLASS_TO_FRAMEWORK outputs; the remaining 12 framework
//! manifests (Wasp / Encore / RedwoodSDK / T3 / etc.) are
//! RUNPOD-PENDING per `workflow/reasoning/E-12-tier-b-decisions.md`
//! pending paper-ingestor source-grounding on the official docs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::framework_profile::profile::{
    framework_for_project_class, FrameworkProfile, FrameworkProfileError, SecurityCheckId,
};
use crate::idea_framing::ProjectClass;

/// Errors surfaced by the registry loader. Distinct from
/// `FrameworkProfileError` (per-manifest parse error) so callers can
/// distinguish "registry never loaded" from "this manifest is bad".
#[derive(Debug, Error)]
pub enum FrameworkProfileRegistryError {
    #[error("workflow/archetypes/ directory not found at {path}")]
    ArchetypesDirMissing { path: String },
    #[error("workflow/archetypes/ directory is empty at {path}")]
    RegistryEmpty { path: String },
    #[error("framework profile `{name}` not registered (loaded {count} profiles)")]
    NotFound { name: String, count: usize },
    #[error("registry io error reading {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("manifest parse error: {0}")]
    Manifest(#[from] FrameworkProfileError),
}

/// Raw TOML shape of a framework manifest. Owned by this module so
/// the on-disk format can evolve without leaking serde derive into
/// the public `FrameworkProfile` value type.
#[derive(Debug, Deserialize)]
struct ManifestRaw {
    name: String,
    version_pin: String,
    language: String,
    package_manager: String,
    lint_config_path: String,
    test_runner: String,
    security_checks: Vec<String>,
}

fn parse_security_check(label: &str) -> Option<SecurityCheckId> {
    match label {
        "secrets_scan" => Some(SecurityCheckId::SecretsScan),
        "trufflehog_scan" => Some(SecurityCheckId::TrufflehogScan),
        "owasp_static" => Some(SecurityCheckId::OwaspStatic),
        "npm_audit" => Some(SecurityCheckId::NpmAudit),
        "pip_audit" => Some(SecurityCheckId::PipAudit),
        "cargo_audit" => Some(SecurityCheckId::CargoAudit),
        "lighthouse" => Some(SecurityCheckId::Lighthouse),
        "biome" => Some(SecurityCheckId::Biome),
        "ruff" => Some(SecurityCheckId::Ruff),
        "clippy" => Some(SecurityCheckId::Clippy),
        "sqlite3_rls" => Some(SecurityCheckId::Sqlite3Rls),
        "cargo_machete" => Some(SecurityCheckId::CargoMachete),
        "otel_check" => Some(SecurityCheckId::OtelCheck),
        "sentry_check" => Some(SecurityCheckId::SentryCheck),
        "lsof_port" => Some(SecurityCheckId::LsofPort),
        _ => None,
    }
}

fn parse_profile(path: &Path, contents: &str) -> Result<FrameworkProfile, FrameworkProfileError> {
    let raw: ManifestRaw = toml::from_str(contents).map_err(|e| FrameworkProfileError::Toml {
        path: path.display().to_string(),
        source: e,
    })?;
    let mut checks: Vec<SecurityCheckId> = Vec::with_capacity(raw.security_checks.len());
    for label in raw.security_checks.iter() {
        match parse_security_check(label) {
            Some(id) => checks.push(id),
            None => {
                return Err(FrameworkProfileError::UnknownSecurityCheck {
                    id: label.clone(),
                    path: path.display().to_string(),
                });
            }
        }
    }
    Ok(FrameworkProfile {
        name: raw.name,
        version_pin: raw.version_pin,
        language: raw.language,
        package_manager: raw.package_manager,
        lint_config: PathBuf::from(raw.lint_config_path),
        test_runner: raw.test_runner,
        security_checks: checks,
    })
}

/// Registry of loaded framework profiles. Keyed by canonical name
/// (manifest basename without `.toml`).
#[derive(Debug, Clone, Default)]
pub struct FrameworkProfileRegistry {
    profiles: BTreeMap<String, FrameworkProfile>,
    source_dir: Option<PathBuf>,
}

impl FrameworkProfileRegistry {
    /// Construct an empty registry (no manifests loaded). Callers can
    /// either populate via `insert` (for tests) or load from disk via
    /// `load_from_dir`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load every `*.toml` manifest from `archetypes_dir`. Skips files
    /// that fail to parse with a tracing warning so a single broken
    /// manifest does not poison the entire registry. Returns the
    /// constructed registry; the count of successfully loaded
    /// profiles is `registry.len()`.
    pub fn load_from_dir(
        archetypes_dir: &Path,
    ) -> Result<Self, FrameworkProfileRegistryError> {
        if !archetypes_dir.exists() {
            return Err(FrameworkProfileRegistryError::ArchetypesDirMissing {
                path: archetypes_dir.display().to_string(),
            });
        }
        let mut profiles: BTreeMap<String, FrameworkProfile> = BTreeMap::new();
        let entries = std::fs::read_dir(archetypes_dir).map_err(|e| {
            FrameworkProfileRegistryError::Io {
                path: archetypes_dir.display().to_string(),
                source: e,
            }
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let contents = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "framework-profile: skipping unreadable manifest");
                    continue;
                }
            };
            match parse_profile(&path, &contents) {
                Ok(profile) => {
                    profiles.insert(profile.name.clone(), profile);
                }
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "framework-profile: skipping invalid manifest");
                }
            }
        }
        Ok(Self {
            profiles,
            source_dir: Some(archetypes_dir.to_path_buf()),
        })
    }

    /// Insert a profile manually (used by tests + the generic
    /// fallback path).
    pub fn insert(&mut self, profile: FrameworkProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    /// Look up a profile by canonical name. Returns
    /// `FrameworkProfileRegistryError::NotFound` if absent.
    pub fn get(&self, name: &str) -> Result<&FrameworkProfile, FrameworkProfileRegistryError> {
        self.profiles
            .get(name)
            .ok_or_else(|| FrameworkProfileRegistryError::NotFound {
                name: name.to_owned(),
                count: self.profiles.len(),
            })
    }

    /// Resolve `ProjectClass` to a `FrameworkProfile` via the
    /// canonical `PROJECT_CLASS_TO_FRAMEWORK` table. Falls back to
    /// `FrameworkProfile::generic()` when the resolved profile name
    /// is not registered (the wire-dark posture path).
    pub fn for_project_class(&self, class: ProjectClass) -> FrameworkProfile {
        let name = framework_for_project_class(class);
        self.profiles
            .get(name)
            .cloned()
            .unwrap_or_else(FrameworkProfile::generic)
    }

    /// Number of profiles loaded.
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    /// True iff the registry has no loaded profiles.
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    /// Source directory the registry was loaded from (if any). `None`
    /// when constructed via `new` / `insert` (test path).
    pub fn source_dir(&self) -> Option<&Path> {
        self.source_dir.as_deref()
    }

    /// Iterate over all registered (name, profile) pairs in
    /// alphabetical name order. BTreeMap iteration is deterministic.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &FrameworkProfile)> {
        self.profiles.iter()
    }
}

/// Canonical entry point. Loads the registry from
/// `<project_root>/workflow/archetypes/` and resolves the framework
/// profile for the given project class. Wire-dark fallback: returns
/// `FrameworkProfile::generic()` if the registry is unloadable or the
/// resolved profile name is not present.
///
/// `project_root` is typically `F:\.ELAI_workflow\` on dev and
/// `/workspace/.ELAI_workflow` on the remote runpod host.
pub fn get_framework_profile(project_root: &Path, class: ProjectClass) -> FrameworkProfile {
    let archetypes = project_root.join("workflow").join("archetypes");
    match FrameworkProfileRegistry::load_from_dir(&archetypes) {
        Ok(reg) => reg.for_project_class(class),
        Err(e) => {
            tracing::info!(
                error = %e,
                "framework-profile: registry load failed, returning generic profile"
            );
            FrameworkProfile::generic()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &Path, name: &str, body: &str) {
        std::fs::write(dir.join(name), body).expect("write manifest");
    }

    fn ok_manifest_nextjs() -> &'static str {
        r#"
name = "nextjs15"
version_pin = "15.0.0"
language = "typescript"
package_manager = "pnpm"
lint_config_path = "biome.json"
test_runner = "vitest"
security_checks = ["secrets_scan", "owasp_static", "npm_audit", "biome", "lighthouse", "otel_check", "sentry_check"]
"#
    }

    #[test]
    fn registry_load_missing_dir_returns_archetype_missing() {
        let bogus = std::path::Path::new("/elai_e12_bogus_archetypes_xyzzy_z");
        let err = FrameworkProfileRegistry::load_from_dir(bogus).unwrap_err();
        assert!(matches!(
            err,
            FrameworkProfileRegistryError::ArchetypesDirMissing { .. }
        ));
    }

    #[test]
    fn registry_load_one_manifest() {
        let tmp = TempDir::new().expect("tempdir");
        write(tmp.path(), "nextjs15.toml", ok_manifest_nextjs());
        let reg = FrameworkProfileRegistry::load_from_dir(tmp.path()).expect("load");
        assert_eq!(reg.len(), 1);
        let p = reg.get("nextjs15").expect("found");
        assert_eq!(p.version_pin, "15.0.0");
        assert_eq!(p.language, "typescript");
        assert_eq!(p.security_checks.len(), 7);
    }

    #[test]
    fn registry_skips_invalid_manifest() {
        let tmp = TempDir::new().expect("tempdir");
        write(tmp.path(), "nextjs15.toml", ok_manifest_nextjs());
        write(tmp.path(), "broken.toml", "this is not toml = [");
        let reg = FrameworkProfileRegistry::load_from_dir(tmp.path()).expect("load");
        // Only nextjs15 should be loaded; broken.toml is skipped.
        assert_eq!(reg.len(), 1);
        assert!(reg.get("nextjs15").is_ok());
    }

    #[test]
    fn for_project_class_falls_back_to_generic() {
        let reg = FrameworkProfileRegistry::new();
        let p = reg.for_project_class(ProjectClass::SaaSDashboard);
        // No manifest loaded -> generic fallback.
        assert_eq!(p.name, "generic");
        assert_eq!(p.security_checks.len(), 15);
    }
}
