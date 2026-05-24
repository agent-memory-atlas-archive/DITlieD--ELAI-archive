//! E-12 Tier-B — FrameworkProfile value type + PROJECT_CLASS_TO_FRAMEWORK.
//!
//! The 8 FrameworkProfile names (Wasp / EncoreDev / RedwoodSdk /
//! T3Stack / HonoEdge / FastApi / NextAppRouter / Generic) are an
//! exhaustive cover of the production fullstack targets surveyed in
//! the E-12 plan. The names are pinned in this enum so the contract
//! tests can assert exact match (Tier-A guard 1 forbids parallel names
//! like FrameworkAdapterV2).
//!
//! Version pins live in `workflow/archetypes/<name>.toml`, NOT here.
//! The Profile value carries the manifest path; the loader resolves
//! the version pin at registry build time. This indirection is
//! mandatory per Tier-A guard 4 (no inlined framework version
//! literals in source) + the anti-slopsquatting CLAUDE.md HARD RULE.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::idea_framing::ProjectClass;

/// Security-check identifier referenced by the 15 production checklist
/// runners. Snake-case wire labels match the runner module file names
/// at `crate::production_checklist::*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SecurityCheckId {
    SecretsScan,
    TrufflehogScan,
    OwaspStatic,
    NpmAudit,
    PipAudit,
    CargoAudit,
    Lighthouse,
    Biome,
    Ruff,
    Clippy,
    Sqlite3Rls,
    CargoMachete,
    OtelCheck,
    SentryCheck,
    LsofPort,
}

impl SecurityCheckId {
    /// Canonical snake_case wire label. Stable across versions; used in
    /// SSB rows + manifest TOML keys.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SecretsScan => "secrets_scan",
            Self::TrufflehogScan => "trufflehog_scan",
            Self::OwaspStatic => "owasp_static",
            Self::NpmAudit => "npm_audit",
            Self::PipAudit => "pip_audit",
            Self::CargoAudit => "cargo_audit",
            Self::Lighthouse => "lighthouse",
            Self::Biome => "biome",
            Self::Ruff => "ruff",
            Self::Clippy => "clippy",
            Self::Sqlite3Rls => "sqlite3_rls",
            Self::CargoMachete => "cargo_machete",
            Self::OtelCheck => "otel_check",
            Self::SentryCheck => "sentry_check",
            Self::LsofPort => "lsof_port",
        }
    }

    /// All 15 security-check IDs in declaration order. Equal to the
    /// number of runner modules under `production_checklist/`.
    pub const fn all() -> &'static [SecurityCheckId; 15] {
        &[
            SecurityCheckId::SecretsScan,
            SecurityCheckId::TrufflehogScan,
            SecurityCheckId::OwaspStatic,
            SecurityCheckId::NpmAudit,
            SecurityCheckId::PipAudit,
            SecurityCheckId::CargoAudit,
            SecurityCheckId::Lighthouse,
            SecurityCheckId::Biome,
            SecurityCheckId::Ruff,
            SecurityCheckId::Clippy,
            SecurityCheckId::Sqlite3Rls,
            SecurityCheckId::CargoMachete,
            SecurityCheckId::OtelCheck,
            SecurityCheckId::SentryCheck,
            SecurityCheckId::LsofPort,
        ]
    }
}

/// Framework profile value. Carries the resolved manifest fields after
/// `FrameworkProfileRegistry::load` has read the corresponding
/// `workflow/archetypes/<name>.toml` file. Never inline literal version
/// pins here — load them from the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkProfile {
    /// Canonical profile name (e.g. `nextjs15`, `svelte5`, `fastapi-modern`,
    /// `hono`, `generic`). Matches the manifest file basename.
    pub name: String,
    /// Version pin string as recorded in the manifest. NEVER inlined as
    /// a source literal — Tier-A guard 4 enforces.
    pub version_pin: String,
    /// Primary language (typescript / python / rust / go / generic).
    pub language: String,
    /// Package manager (npm / pnpm / yarn / pip / uv / poetry / cargo /
    /// generic).
    pub package_manager: String,
    /// Path to the manifest-declared lint config (biome.json / ruff.toml
    /// / clippy.toml / .eslintrc.json / etc). Resolved relative to the
    /// project root by the caller.
    pub lint_config: PathBuf,
    /// Test-runner binary name (vitest / pytest / cargo-test / etc).
    pub test_runner: String,
    /// Security checks to run for this framework. Ordered list; checks
    /// are dispatched in this sequence by
    /// `crate::production_checklist::run_all`.
    pub security_checks: Vec<SecurityCheckId>,
}

impl FrameworkProfile {
    /// Generic fallback profile used when the registry has no matching
    /// manifest (or the archetypes dir is absent). Carries an empty
    /// version pin and the language-neutral `generic` defaults. All 15
    /// security checks are eligible — the runner's `ToolchainMissing`
    /// branch handles the missing-CLI case.
    pub fn generic() -> Self {
        Self {
            name: "generic".to_owned(),
            version_pin: String::new(),
            language: "generic".to_owned(),
            package_manager: "generic".to_owned(),
            lint_config: PathBuf::new(),
            test_runner: "generic".to_owned(),
            security_checks: SecurityCheckId::all().to_vec(),
        }
    }
}

/// Errors surfaced when parsing a manifest into a [`FrameworkProfile`].
#[derive(Debug, Error)]
pub enum FrameworkProfileError {
    #[error("framework manifest missing required field `{field}` in {path}")]
    MissingField { field: String, path: String },
    #[error("framework manifest has invalid security_check_id `{id}` in {path}")]
    UnknownSecurityCheck { id: String, path: String },
    #[error("framework manifest TOML parse error in {path}: {source}")]
    Toml {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("framework manifest io error in {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Deterministic dispatch table: `ProjectClass` (9 variants from B-40,
/// `idea_framing.rs:89`) -> canonical framework profile name. The
/// returned `&'static str` is the manifest basename (without the
/// `.toml` extension) and matches what
/// `FrameworkProfileRegistry::load_by_name` looks up.
///
/// Anti-slopsquatting: the names here are profile slugs (NOT framework
/// version literals); the actual version pins live in
/// `workflow/archetypes/<name>.toml` and the auditor / executor verify
/// them via the manifest reader, not via a string-equality check here.
///
/// The mapping is a starting heuristic per the plan's
/// Open-questions block (`PROJECT_CLASS_TO_FRAMEWORK` is non-empirical
/// until 20+ idea-framing sessions ship). Wire-dark posture: all 9
/// classes route SOMEWHERE — `Other` and the long-tail mobile/desktop/
/// game/research/library classes fall to `generic` until concrete
/// archetype manifests ship in a RUNPOD-PENDING follow-up.
pub const fn framework_for_project_class(class: ProjectClass) -> &'static str {
    match class {
        ProjectClass::SaaSDashboard => "nextjs15",
        ProjectClass::WebApp => "svelte5",
        ProjectClass::ConsumerMobileApp => "generic",
        ProjectClass::DesktopApp => "generic",
        ProjectClass::GameMod => "generic",
        ProjectClass::ResearchTool => "fastapi-modern",
        ProjectClass::LibraryOrSDK => "generic",
        ProjectClass::CLITool => "hono",
        ProjectClass::Other => "generic",
    }
}

/// Read-only snapshot of the full `ProjectClass -> profile-name`
/// mapping in declaration order. Produced from
/// `framework_for_project_class` on demand so the canonical source-
/// of-truth is the const fn (no risk of drift between two parallel
/// tables). Returns a `Vec<(ProjectClass, &'static str)>` rather
/// than a BTreeMap because `ProjectClass` does not implement `Ord`
/// (declared in B-40 idea_framing.rs:87 — adding `Ord` derives there
/// would be a cross-plan change outside E-12 scope).
#[allow(non_snake_case)]
pub fn PROJECT_CLASS_TO_FRAMEWORK() -> Vec<(ProjectClass, &'static str)> {
    [
        ProjectClass::ConsumerMobileApp,
        ProjectClass::WebApp,
        ProjectClass::SaaSDashboard,
        ProjectClass::DesktopApp,
        ProjectClass::GameMod,
        ProjectClass::ResearchTool,
        ProjectClass::LibraryOrSDK,
        ProjectClass::CLITool,
        ProjectClass::Other,
    ]
    .into_iter()
    .map(|c| (c, framework_for_project_class(c)))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_check_id_all_has_15_entries() {
        assert_eq!(SecurityCheckId::all().len(), 15);
    }

    #[test]
    fn security_check_id_wire_labels_are_unique() {
        use std::collections::BTreeSet;
        let mut seen = BTreeSet::new();
        for id in SecurityCheckId::all() {
            assert!(seen.insert(id.as_str()), "duplicate {}", id.as_str());
        }
    }

    #[test]
    fn project_class_table_covers_all_9_variants() {
        let map = PROJECT_CLASS_TO_FRAMEWORK();
        assert_eq!(map.len(), 9, "every ProjectClass must map to a profile");
    }

    #[test]
    fn generic_profile_has_15_checks() {
        let g = FrameworkProfile::generic();
        assert_eq!(g.security_checks.len(), 15);
        assert_eq!(g.name, "generic");
    }
}
