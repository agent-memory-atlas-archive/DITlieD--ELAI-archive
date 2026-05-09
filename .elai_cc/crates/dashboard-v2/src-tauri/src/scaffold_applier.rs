//! `ScaffoldApplier` implementation that lives inside the dashboard.
//!
//! Ported from the (now-deleted) `crates/dashboard` v1 build during
//! session s-69820 / Phase A. The orchestrator crate exposes the
//! [`ScaffoldApplier`] trait but deliberately does not depend on
//! `scaffold` itself — that dependency is concentrated here so the
//! task runner can keep its `scaffold_applier` slot a `dyn` pointer.
//!
//! Variable injection is delegated to [`scaffold::default_build_vars`]
//! so contract tests can assert the seven-key shape without needing to
//! link against the dashboard.

use std::path::{Path, PathBuf};

use elai_orchestrator::ScaffoldApplier;
use scaffold::{apply_template_from_root, default_build_vars, TEMPLATES_WEB_SUBDIR};

/// Env override pointing at a custom `templates/web/` directory. Required
/// in shipped Tauri bundles, where `CARGO_MANIFEST_DIR` no longer exists.
pub const TEMPLATES_DIR_ENV: &str = "ELAI_TEMPLATES_DIR";

/// Pure resolver: env override → dev-tree fallback. Extracted from
/// [`default_templates_dir`] so unit tests can drive it without
/// `std::env::set_var`/`remove_var` (which is unsound under parallel
/// `cargo test` per Rust 1.81).
pub fn resolve_templates_dir(env_value: Option<&str>) -> PathBuf {
    if let Some(v) = env_value {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join(TEMPLATES_WEB_SUBDIR)
}

/// Resolve the `templates/web/` directory.
///
/// Resolution order:
///   1. `ELAI_TEMPLATES_DIR` env var if set and non-empty.
///   2. `CARGO_MANIFEST_DIR/../../../templates/web` (dev-tree fallback).
///      Three uplift levels because `CARGO_MANIFEST_DIR` for this crate is
///      `crates/dashboard-v2/src-tauri/`, three steps shallower than the
///      `.bkp_old` location was.
pub fn default_templates_dir() -> PathBuf {
    let value = std::env::var(TEMPLATES_DIR_ENV).ok();
    resolve_templates_dir(value.as_deref())
}

/// Concrete [`ScaffoldApplier`] used by the dashboard. Holds no state —
/// constructed once at task-runner config time and shared via `Arc`.
pub struct DashboardScaffoldApplier;

impl ScaffoldApplier for DashboardScaffoldApplier {
    fn apply(
        &self,
        template_name: &str,
        target_dir: &Path,
        task_text: &str,
    ) -> Result<(), String> {
        let templates_root = default_templates_dir();
        let vars = default_build_vars(task_text, None, None, None);
        apply_template_from_root(&templates_root, template_name, target_dir, &vars)
            .map_err(|e| format!("apply_template({template_name}): {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Audit fix (s-69820): drives the pure resolver so the test does not
    /// touch process-global env vars (set_var/remove_var is unsound under
    /// the parallel cargo test runner per Rust 1.81).
    #[test]
    fn templates_dir_env_takes_precedence() {
        let custom = std::env::temp_dir().join("phase-a-templates-dir-override");
        let custom_str = custom.to_string_lossy().to_string();
        let resolved = resolve_templates_dir(Some(&custom_str));
        assert_eq!(resolved, custom);
    }

    #[test]
    fn templates_dir_falls_back_when_env_missing_or_blank() {
        let none_resolved = resolve_templates_dir(None);
        let blank_resolved = resolve_templates_dir(Some("   "));
        assert_eq!(none_resolved, blank_resolved);
        assert!(none_resolved.ends_with(TEMPLATES_WEB_SUBDIR));
    }
}
