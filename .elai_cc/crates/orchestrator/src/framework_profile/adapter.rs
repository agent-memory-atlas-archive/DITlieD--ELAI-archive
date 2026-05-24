//! E-12 Tier-B — FrameworkAdapter trait.
//!
//! MIRRORS `crate::task_runner::ScaffoldApplier` (canonical
//! FrameworkAdapter shape at `task_runner.rs:2173`). Tier-A guard 1
//! `no_parallel_framework_adapter_trait` bans parallel forks
//! (FrameworkAdapterV2 / BespokeFrameworkAdapter / FullstackEmitterV2
//! / AdHocScaffoldEmitter). Adapters live alongside, not in place of,
//! `ScaffoldApplier` — they share the same `apply` signature so a
//! production-graded adapter can drop into the existing
//! `ResolvedConfig.scaffold_applier` slot in `task_runner.rs:2216`.
//!
//! Extension over ScaffoldApplier: two extra methods —
//!   1. `production_checklist()` — returns the framework's curated 15-
//!      check list (subset of `SecurityCheckId::all()`).
//!   2. `apply_lint_defaults(&FilePlan) -> Vec<PathBuf>` — returns the
//!      list of legacy linter config paths that the framework adapter
//!      considers stale (`.eslintrc`, `.prettierrc`, `.flake8`, ...).
//!      Callers (typically the C-32 `flush_to_disk` path) delete these
//!      atomically.

use std::path::{Path, PathBuf};

use thiserror::Error;

use elai_core::file_plan::FilePlan;

use crate::framework_profile::profile::SecurityCheckId;

/// Errors surfaced by [`FrameworkAdapter::apply`]. Mirrors the string
/// `Result<(), String>` used by `ScaffoldApplier` but wraps the message
/// in a structured `thiserror` enum so downstream callers can match on
/// kind (Tier-A guard 1 + memory:wire-dark-posture: structured error,
/// not `Box<dyn Error>` and not raw `String`).
#[derive(Debug, Error)]
pub enum FrameworkAdapterError {
    #[error("framework adapter received unknown template `{template}`")]
    UnknownTemplate { template: String },
    #[error("framework adapter target dir does not exist: {path}")]
    TargetMissing { path: String },
    #[error("framework adapter scaffold failed: {reason}")]
    ScaffoldFailed { reason: String },
    #[error("framework adapter io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Pluggable framework adapter MIRRORING
/// `crate::task_runner::ScaffoldApplier`. The `apply` signature is
/// byte-identical (sync, takes `template_name`, `target_dir`,
/// `task_text`) so a `FrameworkAdapter` implementation can also be
/// registered as the runner's `scaffold_applier` without an adapter
/// shim. The two extension methods are framework-specific and have no
/// counterpart in `ScaffoldApplier`.
///
/// All implementations MUST be `Send + Sync` — the orchestrator stores
/// them behind `Arc<dyn FrameworkAdapter>` for cross-thread dispatch.
pub trait FrameworkAdapter: Send + Sync {
    /// Apply a scaffold template to the target directory. Mirrors
    /// `ScaffoldApplier::apply`. Returns `Ok(())` on success or a
    /// structured `FrameworkAdapterError` on failure (NOT a string).
    fn apply(
        &self,
        template_name: &str,
        target_dir: &Path,
        task_text: &str,
    ) -> Result<(), FrameworkAdapterError>;

    /// The framework-specific production checklist — subset of the 15
    /// universal `SecurityCheckId` values that this framework actually
    /// runs. The default impl returns every check (full coverage); a
    /// framework-specific adapter can narrow the list (e.g. a pure-Rust
    /// adapter can omit `Biome` / `Ruff` since they only apply to TS /
    /// Python sources).
    fn production_checklist(&self) -> Vec<SecurityCheckId> {
        SecurityCheckId::all().to_vec()
    }

    /// Return the list of legacy linter config files this framework
    /// considers stale and that should be removed from the project
    /// (`.eslintrc`, `.prettierrc`, `.flake8`, `setup.cfg [flake8]`,
    /// `pyproject.toml [tool.black]/[tool.isort]`). The default impl
    /// returns an empty vec (no removals); framework adapters override
    /// to declare their canonical lint engine via the manifest.
    ///
    /// The caller (C-32 `flush_to_disk` path or the production
    /// auditor) is responsible for actually deleting the files —
    /// `FrameworkAdapter` only declares intent. This decouples the
    /// adapter from `FilePlan`'s mutation surface and lets the
    /// auditor emit a `legacy_config_stripped` SSB row per removed
    /// path.
    fn apply_lint_defaults(&self, _file_plan: &FilePlan) -> Vec<PathBuf> {
        Vec::new()
    }
}

/// Generic / fallback adapter. Used when no concrete framework
/// adapter has been wired for the current `FrameworkProfile`. Returns
/// `UnknownTemplate` from `apply` (the scaffold pipeline falls back to
/// the workspace's canonical `ScaffoldApplier` impl); returns the full
/// 15-check list from `production_checklist` so the production-grade
/// auditor still runs every gate.
#[derive(Debug, Default)]
pub struct GenericFrameworkAdapter;

impl FrameworkAdapter for GenericFrameworkAdapter {
    fn apply(
        &self,
        template_name: &str,
        target_dir: &Path,
        _task_text: &str,
    ) -> Result<(), FrameworkAdapterError> {
        if !target_dir.exists() {
            return Err(FrameworkAdapterError::TargetMissing {
                path: target_dir.display().to_string(),
            });
        }
        // The generic adapter does NOT own a template engine — the
        // existing `ScaffoldApplier` impls (DashboardScaffoldApplier
        // at task_runner.rs:2332) own the byte-substitute path
        // (scaffold/src/substitute.rs). Returning UnknownTemplate
        // signals "delegate to the canonical applier".
        Err(FrameworkAdapterError::UnknownTemplate {
            template: template_name.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_adapter_returns_full_checklist() {
        let g = GenericFrameworkAdapter;
        assert_eq!(g.production_checklist().len(), 15);
    }

    #[test]
    fn generic_adapter_returns_no_lint_removals() {
        let g = GenericFrameworkAdapter;
        let fp = FilePlan::empty();
        assert!(g.apply_lint_defaults(&fp).is_empty());
    }

    #[test]
    fn generic_adapter_apply_with_missing_target() {
        let g = GenericFrameworkAdapter;
        let bogus = std::path::Path::new("/elai_e12_bogus_target_xyzzy");
        let res = g.apply("any", bogus, "task");
        assert!(matches!(res, Err(FrameworkAdapterError::TargetMissing { .. })));
    }
}
