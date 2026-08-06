//! J-93 LEG-B — tool-keyed compensation registry for H-35 rewind of external effects.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

pub const ELAI_COMPENSATION_REGISTRY_ENABLE: &str = "ELAI_COMPENSATION_REGISTRY_ENABLE";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Invertibility { Invertible, NonInvertible, Unknown }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompensationRecipe {
    DeletePath { path: String },
    GitRevertForceWithLease { ref_name: String },
    SurfaceToHuman { reason: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideEffectRecord {
    pub tool_name: String,
    pub action_type: String,
    pub path: Option<String>,
    pub summary: String,
    pub invertibility: Invertibility,
    pub recipe: CompensationRecipe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompensationDisposition { Observed, Applied, SurfacedToHuman }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompensationPlanEntry {
    pub record: SideEffectRecord,
    pub disposition: CompensationDisposition,
}

fn table() -> BTreeMap<&'static str, Invertibility> {
    BTreeMap::from([
        ("write", Invertibility::Invertible),
        ("file_write", Invertibility::Invertible),
        ("create_file", Invertibility::Invertible),
        ("Write", Invertibility::Invertible),
        ("bash", Invertibility::NonInvertible),
        ("shell", Invertibility::NonInvertible),
        ("Bash", Invertibility::NonInvertible),
        ("git_push", Invertibility::Invertible),
        ("GitPush", Invertibility::Invertible),
    ])
}

pub fn lookup_tool(tool_name: &str) -> (Invertibility, CompensationRecipe) {
    let kind = table().get(tool_name).copied().unwrap_or(Invertibility::Unknown);
    let recipe = match kind {
        Invertibility::Invertible if tool_name == "git_push" || tool_name == "GitPush" =>
            CompensationRecipe::GitRevertForceWithLease { ref_name: "HEAD".to_string() },
        Invertibility::Invertible => CompensationRecipe::DeletePath { path: String::new() },
        Invertibility::NonInvertible => CompensationRecipe::SurfaceToHuman {
            reason: "shell side effects not auto-reversible",
        },
        Invertibility::Unknown => CompensationRecipe::SurfaceToHuman {
            reason: "unknown tool; fail-closed",
        },
    };
    (kind, recipe)
}

pub fn compensation_registry_enabled() -> bool {
    match std::env::var(ELAI_COMPENSATION_REGISTRY_ENABLE) {
        Ok(value) => matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    }
}

pub fn classify_side_effect(tool_name: &str, action_type: &str, path: Option<&str>) -> SideEffectRecord {
    let (invertibility, mut recipe) = lookup_tool(tool_name);
    if matches!(recipe, CompensationRecipe::DeletePath { .. }) {
        recipe = CompensationRecipe::DeletePath { path: path.unwrap_or_default().to_string() };
    }
    let summary = match path {
        Some(path) if !path.is_empty() => format!("{action_type}: {tool_name} ({path})"),
        _ => format!("{action_type}: {tool_name}"),
    };
    SideEffectRecord {
        tool_name: tool_name.to_string(), action_type: action_type.to_string(),
        path: path.map(str::to_string), summary, invertibility, recipe,
    }
}

fn journal() -> &'static Mutex<Vec<SideEffectRecord>> {
    static JOURNAL: OnceLock<Mutex<Vec<SideEffectRecord>>> = OnceLock::new();
    JOURNAL.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn record_tool_side_effect(tool_name: &str, action_type: &str, path: Option<&str>) {
    let record = classify_side_effect(tool_name, action_type, path);
    if let Ok(mut entries) = journal().lock() { entries.push(record); }
    else { tracing::warn!("compensation journal lock poisoned; side effect was not recorded"); }
}

pub fn journal_snapshot() -> Vec<SideEffectRecord> {
    journal().lock().map(|entries| entries.clone()).unwrap_or_default()
}

#[cfg(test)]
fn journal_clear() {
    if let Ok(mut entries) = journal().lock() { entries.clear(); }
}

pub fn plan_and_apply_for_rewind() -> Vec<CompensationPlanEntry> {
    let enabled = compensation_registry_enabled();
    journal_snapshot().into_iter().map(|record| {
        let disposition = match record.invertibility {
            Invertibility::NonInvertible | Invertibility::Unknown => CompensationDisposition::SurfacedToHuman,
            Invertibility::Invertible if !enabled => CompensationDisposition::Observed,
            Invertibility::Invertible => {
                if let CompensationRecipe::DeletePath { ref path } = record.recipe {
                    let candidate = Path::new(path);
                    if candidate.is_file() {
                        if let Err(error) = std::fs::remove_file(candidate) {
                            tracing::warn!(path = %path, %error, "compensation file removal failed");
                        }
                    }
                }
                CompensationDisposition::Applied
            }
        };
        CompensationPlanEntry { record, disposition }
    }).collect()
}

#[cfg(test)]
mod j93_compensation_registry_tests {
    use super::*;
    fn reset() { journal_clear(); std::env::remove_var(ELAI_COMPENSATION_REGISTRY_ENABLE); }

    #[test]
    fn flag_defaults_off() { reset(); assert!(!compensation_registry_enabled()); }
    #[test]
    fn write_tool_is_invertible() {
        reset(); let (kind, recipe) = lookup_tool("write");
        assert_eq!(kind, Invertibility::Invertible);
        assert_eq!(recipe, CompensationRecipe::DeletePath { path: String::new() });
        assert_eq!(classify_side_effect("write", "create", Some("x")).recipe,
                   CompensationRecipe::DeletePath { path: "x".into() });
    }
    #[test]
    fn bash_is_non_invertible() { reset(); assert_eq!(lookup_tool("bash").0, Invertibility::NonInvertible); }
    #[test]
    fn unknown_fail_closed() {
        reset(); let (kind, recipe) = lookup_tool("totally_unknown_tool_xyz");
        assert_eq!(kind, Invertibility::Unknown); assert!(matches!(recipe, CompensationRecipe::SurfaceToHuman { .. }));
    }
    #[test]
    fn plan_flag_off_observes_invertible() {
        reset(); record_tool_side_effect("write", "create", None);
        assert_eq!(plan_and_apply_for_rewind()[0].disposition, CompensationDisposition::Observed);
    }
    #[test]
    fn non_invertible_always_surfaced() {
        reset(); record_tool_side_effect("bash", "run", None);
        assert_eq!(plan_and_apply_for_rewind()[0].disposition, CompensationDisposition::SurfacedToHuman);
        std::env::set_var(ELAI_COMPENSATION_REGISTRY_ENABLE, "1");
        assert_eq!(plan_and_apply_for_rewind()[0].disposition, CompensationDisposition::SurfacedToHuman);
    }
    #[test]
    fn git_push_never_shells() {
        reset(); let (kind, recipe) = lookup_tool("git_push");
        assert_eq!(kind, Invertibility::Invertible);
        assert!(matches!(recipe, CompensationRecipe::GitRevertForceWithLease { .. }));
        record_tool_side_effect("git_push", "push", None); std::env::set_var(ELAI_COMPENSATION_REGISTRY_ENABLE, "1");
        assert_eq!(plan_and_apply_for_rewind()[0].disposition, CompensationDisposition::Applied);
    }
}
