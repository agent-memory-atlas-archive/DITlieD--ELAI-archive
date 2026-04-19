//! A-36 Tier 3 completions: language-specific fix templates + DKG few-shot.
//!
//! This module loads curated fix templates from `.elai_cc/data/fix_templates/`
//! (one plain-text file per key) and attaches them to a `ProjectProfile`'s
//! Tier 3 output via `FixTemplateRegistry::load_default`. It also exposes a
//! convention-aware template picker and the DKG few-shot helper the
//! deterministic auditor uses when emitting a Finding.
//!
//! The templates are intentionally text-only: the main agent fills in the
//! placeholders per fix site. The registry itself is pure in-memory state.
//!
//! Backward compatibility: a missing `fix_templates/` directory yields an
//! empty registry (no Tier 3 regression) and callers degrade gracefully.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use elai_core::project_profile::{Convention, NamingConventions, ProjectProfile};

use crate::{dkg::DkgEngine, DkgNodeInfo, KnowledgeError};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default relative directory under the project root where curated fix
/// templates live. Mirrors the `data/rules/` layout.
pub const DEFAULT_FIX_TEMPLATES_DIR: &str = "data/fix_templates";

/// Maximum number of few-shot examples attached to a single Finding.
pub const MAX_FEW_SHOT_EXAMPLES: usize = 3;

/// Template filename -> registry-key mapping. Seeded with the four
/// A-36 acceptance keys; additional templates dropped into the directory are
/// loaded lazily by their filename stem.
const SEED_KEYS: &[(&str, &str)] = &[
    ("rust_import_path_fix.txt", "rust.import_path_fix"),
    ("python_import_stub.txt", "python.import_stub"),
    ("typescript_null_check.txt", "typescript.null_check"),
    ("go_error_wrap.txt", "go.error_wrap"),
];

// ---------------------------------------------------------------------------
// FixTemplateRegistry
// ---------------------------------------------------------------------------

/// In-memory registry of fix templates keyed by `"{language}.{kind}"`.
///
/// Values are the raw template file contents — the caller substitutes
/// placeholders such as `{deprecated}` or `{replacement}` at fix time.
#[derive(Debug, Default, Clone)]
pub struct FixTemplateRegistry {
    templates: BTreeMap<String, String>,
}

impl FixTemplateRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load templates from the given directory. Each file's name stem is
    /// converted to `{language}.{kind}` using the seed mapping, or falls
    /// back to the stem itself with `_` replaced by `.`.
    ///
    /// Missing directory is treated as "no templates"; malformed UTF-8 or
    /// per-file read errors are skipped (logged at `warn` level) so a
    /// single corrupt file does not take the registry offline.
    pub fn load_from_dir(dir: &Path) -> Self {
        let mut reg = Self::new();
        if !dir.is_dir() {
            tracing::debug!(
                dir = %dir.display(),
                "fix_templates: directory missing, empty registry"
            );
            return reg;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(
                    dir = %dir.display(),
                    error = %e,
                    "fix_templates: read_dir failed, empty registry"
                );
                return reg;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            let key = key_from_filename(file_name);
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    reg.templates.insert(key, content);
                }
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "fix_templates: read failed for file, skipping"
                    );
                }
            }
        }
        tracing::info!(
            dir = %dir.display(),
            loaded = reg.templates.len(),
            "fix_templates: registry loaded"
        );
        reg
    }

    /// Load templates from the default `.elai_cc/data/fix_templates/` path
    /// relative to `project_root`.
    pub fn load_default(project_root: &Path) -> Self {
        Self::load_from_dir(&project_root.join(DEFAULT_FIX_TEMPLATES_DIR))
    }

    /// Number of loaded templates.
    pub fn len(&self) -> usize {
        self.templates.len()
    }

    /// True when no templates are loaded.
    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    /// Fetch a template body by its canonical key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.templates.get(key).map(String::as_str)
    }

    /// Borrow the full set of keys (sorted — `BTreeMap` iteration).
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.templates.keys().map(String::as_str)
    }

    /// Insert a template directly (test-only helper — production callers go
    /// through `load_from_dir`).
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn insert(&mut self, key: impl Into<String>, body: impl Into<String>) {
        self.templates.insert(key.into(), body.into());
    }

    /// Convention-aware template lookup. When the project profile's
    /// `NamingConventions` indicate `camelCase` or `PascalCase` function
    /// names, lookup keys automatically try a `.camel` variant before
    /// falling back to the default. Keeps the registry flat while still
    /// honouring detected conventions.
    pub fn get_for_profile(&self, base_key: &str, profile: &ProjectProfile) -> Option<&str> {
        if casing_is_camel_or_pascal(&profile.naming.functions) {
            let camel_key = format!("{base_key}.camel");
            if let Some(t) = self.get(&camel_key) {
                return Some(t);
            }
        }
        self.get(base_key)
    }
}

/// Detect whether the function-name convention is `camelCase` or `PascalCase`.
/// `Convention` wraps the raw casing string (e.g. `"snake_case"`,
/// `"camelCase"`), so we compare on the underlying token.
fn casing_is_camel_or_pascal(conv: &Convention) -> bool {
    let raw = match conv {
        Convention::Strong(s) | Convention::Moderate(s) => s.as_str(),
        Convention::Mixed | Convention::Unknown => return false,
    };
    matches!(raw, "camelCase" | "PascalCase")
}

/// Convert a raw filename (`rust_import_path_fix.txt`) into the canonical
/// `{language}.{kind}` key (`rust.import_path_fix`).
fn key_from_filename(file_name: &str) -> String {
    for (seed_file, seed_key) in SEED_KEYS {
        if *seed_file == file_name {
            return (*seed_key).to_owned();
        }
    }
    // Fallback: strip `.txt` and convert the first underscore into a dot so
    // `c_sharp_null_check.txt` -> `c.sharp_null_check`. We pick the first
    // underscore deliberately so `{language}.{kind}` always has a single
    // separator.
    let stem = file_name.strip_suffix(".txt").unwrap_or(file_name);
    match stem.find('_') {
        Some(idx) => {
            let (lang, rest) = stem.split_at(idx);
            // `rest` starts with `_`; trim it.
            let rest = rest.trim_start_matches('_');
            format!("{lang}.{rest}")
        }
        None => stem.to_owned(),
    }
}

// ---------------------------------------------------------------------------
// FewShotExample
// ---------------------------------------------------------------------------

/// Few-shot exemplar pulled from DKG similarity. Structurally equivalent to
/// the SSB `FewShotExample` type in `elai_orchestrator::ssb` — we redefine it
/// here to keep `knowledge` independent of `orchestrator`. The orchestrator
/// converts one to the other at the emit site.
#[derive(Debug, Clone, PartialEq)]
pub struct FewShotExample {
    pub symbol_name: String,
    pub file_path: PathBuf,
    pub line: u32,
    pub snippet: String,
}

impl From<&DkgNodeInfo> for FewShotExample {
    fn from(node: &DkgNodeInfo) -> Self {
        let snippet = truncate_snippet(&node.signature);
        Self {
            symbol_name: node.symbol_name.clone(),
            file_path: node.file_path.clone(),
            line: node.start_line,
            snippet,
        }
    }
}

/// Trim a signature to at most two newline-delimited lines so the dashboard
/// sub-block stays within the "max 3 examples × 2 lines" budget from the
/// batch-4a brief.
fn truncate_snippet(signature: &str) -> String {
    let lines: Vec<&str> = signature.lines().take(2).collect();
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// DKG few-shot helper
// ---------------------------------------------------------------------------

/// Resolve up to `MAX_FEW_SHOT_EXAMPLES` in-project few-shot exemplars for a
/// given symbol name via DKG token-overlap similarity. Returns an empty vec
/// (not an error) when the DKG is empty, unavailable, or yields no matches.
pub fn few_shot_for_symbol(
    dkg: &DkgEngine,
    symbol: &str,
) -> Result<Vec<FewShotExample>, KnowledgeError> {
    if symbol.is_empty() {
        return Ok(Vec::new());
    }
    let neighbors = dkg.find_similar_functions(symbol, MAX_FEW_SHOT_EXAMPLES)?;
    Ok(neighbors.iter().map(FewShotExample::from).collect())
}

// ---------------------------------------------------------------------------
// Profile integration — attach registry keys to Tier 3 output
// ---------------------------------------------------------------------------

/// Populate the `fix_templates` section on a `ProjectProfile` with the
/// currently-available keys. Idempotent — calling it twice with the same
/// registry is a no-op.
pub fn attach_fix_template_keys(profile: &mut ProjectProfile, registry: &FixTemplateRegistry) {
    let keys: Vec<String> = registry.keys().map(str::to_owned).collect();
    profile.fix_template_keys = keys;
}

// ---------------------------------------------------------------------------
// Convention-aware template picker (stand-alone helper)
// ---------------------------------------------------------------------------

/// Pick the variant of `base_key` that matches the detected naming
/// conventions. Thin re-export of `FixTemplateRegistry::get_for_profile` for
/// call sites that already have a `NamingConventions` handy but not the
/// whole profile — keeps the deterministic auditor's usage concise.
pub fn convention_aware_key<'a>(
    registry: &'a FixTemplateRegistry,
    base_key: &str,
    naming: &NamingConventions,
) -> Option<&'a str> {
    if casing_is_camel_or_pascal(&naming.functions) {
        let camel_key = format!("{base_key}.camel");
        if let Some(t) = registry.get(&camel_key) {
            return Some(t);
        }
    }
    registry.get(base_key)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn empty_dir_yields_empty_registry() {
        let tmp = tempfile::tempdir().unwrap();
        let reg = FixTemplateRegistry::load_from_dir(tmp.path());
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn missing_dir_yields_empty_registry() {
        let reg = FixTemplateRegistry::load_from_dir(Path::new("/nonexistent/path/xyz"));
        assert!(reg.is_empty());
    }

    #[test]
    fn seed_mapping_applied() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("rust_import_path_fix.txt"), "use {replacement};").unwrap();
        let reg = FixTemplateRegistry::load_from_dir(tmp.path());
        assert_eq!(reg.len(), 1);
        assert!(reg.get("rust.import_path_fix").is_some());
    }

    #[test]
    fn non_seed_file_uses_first_underscore_split() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("kotlin_null_check.txt"), "body").unwrap();
        let reg = FixTemplateRegistry::load_from_dir(tmp.path());
        assert!(reg.get("kotlin.null_check").is_some());
    }

    #[test]
    fn snake_case_profile_returns_default_key() {
        let mut reg = FixTemplateRegistry::new();
        reg.insert("rust.import_path_fix", "default body");
        reg.insert("rust.import_path_fix.camel", "camel body");
        let mut naming = NamingConventions::default();
        naming.functions = Convention::Strong("snake_case".to_owned());
        let body = convention_aware_key(&reg, "rust.import_path_fix", &naming).unwrap();
        assert_eq!(body, "default body");
    }

    #[test]
    fn camel_profile_prefers_camel_variant() {
        let mut reg = FixTemplateRegistry::new();
        reg.insert("ts.null_check", "default body");
        reg.insert("ts.null_check.camel", "camel body");
        let mut naming = NamingConventions::default();
        naming.functions = Convention::Strong("camelCase".to_owned());
        let body = convention_aware_key(&reg, "ts.null_check", &naming).unwrap();
        assert_eq!(body, "camel body");
    }

    #[test]
    fn camel_profile_falls_back_when_no_camel_variant() {
        let mut reg = FixTemplateRegistry::new();
        reg.insert("ts.null_check", "default body");
        let mut naming = NamingConventions::default();
        naming.functions = Convention::Strong("camelCase".to_owned());
        let body = convention_aware_key(&reg, "ts.null_check", &naming).unwrap();
        assert_eq!(body, "default body");
    }

    #[test]
    fn truncate_snippet_takes_at_most_two_lines() {
        let sig = "line1\nline2\nline3\nline4";
        let out = truncate_snippet(sig);
        assert_eq!(out, "line1\nline2");
    }

    #[test]
    fn few_shot_from_dkg_node() {
        let node = DkgNodeInfo {
            node_id: 1,
            symbol_name: "foo".into(),
            file_path: PathBuf::from("src/a.rs"),
            signature: "fn foo()".into(),
            kind: crate::SymbolKind::Function,
            start_line: 42,
            end_line: 44,
        };
        let fs: FewShotExample = (&node).into();
        assert_eq!(fs.symbol_name, "foo");
        assert_eq!(fs.line, 42);
        assert_eq!(fs.snippet, "fn foo()");
    }
}
