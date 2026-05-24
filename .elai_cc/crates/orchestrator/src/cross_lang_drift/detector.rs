//! E-45 — host-side drift detector.
//!
//! `DriftDetector::detect_pairs` walks the registry returned from
//! `PairRegistry::pairs()` and emits one `DriftPair`-keyed scan result
//! per registered pair. `DriftDetector::diff_pair` performs the actual
//! presence + heuristic shape comparison and returns the list of
//! `DriftItem`s.
//!
//! Shape comparison strategy (per D-10 DECISION-8 hand-authored TS
//! mirror discipline):
//! * `PairSideMissing` — either side of the pair does not exist on
//!   disk. Severity `Critical`.
//! * Field-level comparison — both files are read as UTF-8 text and
//!   inspected for serde-style `pub fieldname:` / `fieldname:` literal
//!   tokens. Names present on one side but not the other are reported
//!   as `TypeFieldAdded` / `TypeFieldRemoved`. This is a heuristic, not
//!   a full Rust/TS/Python AST parse; conservative by design —
//!   surfaces only the most common drift signals without false
//!   positives on unrelated text.
//! * Schema version — both files are scanned for a `SCHEMA_VERSION`
//!   integer literal; mismatches emit `SchemaVersionMismatch`. Absent
//!   on either side = no comparison (not all pairs carry a version).
//!
//! Anti-Slopsquatting: every dependency below is in the workspace at
//! the time of writing (`std`, `thiserror`, `regex`, `blake3`).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use thiserror::Error;

use super::pair_registry::PairRegistry;
use super::types::{DriftItem, DriftKind, DriftPair};

#[derive(Debug, Error)]
pub enum DriftDetectorError {
    #[error("repo root does not exist: {0}")]
    RepoRootMissing(PathBuf),
    #[error("file I/O for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("regex compile failure (internal): {0}")]
    Regex(#[from] regex::Error),
}

/// Result of `detect_pairs` — one entry per pair scanned plus the
/// aggregate drift items.
#[derive(Debug, Clone)]
pub struct PairScanResult {
    pub pair: DriftPair,
    pub drift_items: Vec<DriftItem>,
}

/// Stateless detector. Constructed per-scan because each invocation
/// reads a fresh registry from disk.
#[derive(Debug)]
pub struct DriftDetector<'a> {
    registry: &'a PairRegistry,
    field_re: Regex,
    schema_version_re: Regex,
}

impl<'a> DriftDetector<'a> {
    /// Construct a detector backed by a pre-loaded registry. The
    /// caller owns the registry lifetime; the detector is short-lived.
    pub fn new(registry: &'a PairRegistry) -> Result<Self, DriftDetectorError> {
        // Match either `pub field:` (Rust) or `field:` (TS / Pydantic). Field
        // identifiers must follow a struct/object-body boundary so that
        // surrounding declarations (e.g. `class Foo:` in Python, `struct Foo`
        // in Rust) are not mistaken for fields. Accepted boundaries:
        //   - `{` optionally followed by whitespace (struct/object body start),
        //   - `,` optionally followed by whitespace (next field in a list),
        //   - a newline followed by indentation (multi-line bodies).
        // This catches canonical one-line inline structs like
        // `pub struct Foo { pub alpha: u32, pub beta: String }` AND multi-line
        // Python/TS classes without picking up the surrounding type name.
        // Field name = [A-Za-z_][A-Za-z0-9_]*.
        let field_re = Regex::new(
            r"(?:[{,]\s*|\n[ \t]+)(?:pub\s+)?(?:pub\(crate\)\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:",
        )?;
        // Match `SCHEMA_VERSION = 137` or `SCHEMA_VERSION: i64 = 137` or `SCHEMA_VERSION: int = 137`.
        let schema_version_re =
            Regex::new(r"(?m)SCHEMA_VERSION\s*(?::\s*\w+)?\s*=\s*(\d+)")?;
        Ok(Self {
            registry,
            field_re,
            schema_version_re,
        })
    }

    /// Walk every registered pair and produce a per-pair scan result.
    /// Never returns drift via `Err` — drift is in-band on the result.
    /// `Err` is reserved for repo/io/regex faults.
    pub fn detect_pairs(
        &self,
        repo_root: &Path,
    ) -> Result<Vec<PairScanResult>, DriftDetectorError> {
        if !repo_root.exists() {
            return Err(DriftDetectorError::RepoRootMissing(repo_root.to_path_buf()));
        }
        let mut out = Vec::with_capacity(self.registry.len());
        for pair in self.registry.pairs() {
            let drift_items = self.diff_pair(repo_root, pair)?;
            out.push(PairScanResult {
                pair: pair.clone(),
                drift_items,
            });
        }
        Ok(out)
    }

    /// Diff a single pair. Reads both sides, performs presence + field
    /// + schema-version checks, and returns drift items.
    pub fn diff_pair(
        &self,
        repo_root: &Path,
        pair: &DriftPair,
    ) -> Result<Vec<DriftItem>, DriftDetectorError> {
        let rust_abs = repo_root.join(&pair.rust_path);
        let mirror_abs = repo_root.join(&pair.mirror_path);

        let mut items = Vec::new();

        // (1) Presence check: either side missing is Critical.
        let rust_exists = rust_abs.exists();
        let mirror_exists = mirror_abs.exists();
        if !rust_exists {
            items.push(DriftItem::new(
                DriftKind::PairSideMissing {
                    missing_side: pair.rust_path.clone(),
                },
                pair.rust_path.clone(),
            ));
        }
        if !mirror_exists {
            items.push(DriftItem::new(
                DriftKind::PairSideMissing {
                    missing_side: pair.mirror_path.clone(),
                },
                pair.mirror_path.clone(),
            ));
        }
        if !rust_exists || !mirror_exists {
            // Cannot do field/schema comparison if a side is absent.
            return Ok(items);
        }

        let rust_text = read_file(&rust_abs)?;
        let mirror_text = read_file(&mirror_abs)?;

        // (2) Field-level comparison.
        let rust_fields = self.extract_field_names(&rust_text);
        let mirror_fields = self.extract_field_names(&mirror_text);

        // Fields present in Rust but not mirror -> added on Rust.
        let only_rust: Vec<&str> = rust_fields.difference(&mirror_fields).copied().collect();
        for f in only_rust {
            items.push(DriftItem::new(
                DriftKind::TypeFieldAdded { field: f.to_string() },
                pair.mirror_path.clone(),
            ));
        }
        // Fields present in mirror but not Rust -> dropped on Rust.
        let only_mirror: Vec<&str> = mirror_fields.difference(&rust_fields).copied().collect();
        for f in only_mirror {
            items.push(DriftItem::new(
                DriftKind::TypeFieldRemoved {
                    field: f.to_string(),
                },
                pair.rust_path.clone(),
            ));
        }

        // (3) Schema version comparison (only if both sides declare).
        let rust_ver = self.extract_schema_version(&rust_text);
        let mirror_ver = self.extract_schema_version(&mirror_text);
        if let (Some(r), Some(m)) = (rust_ver, mirror_ver) {
            if r != m {
                items.push(DriftItem::new(
                    DriftKind::SchemaVersionMismatch {
                        rust_version: r,
                        mirror_version: m,
                    },
                    pair.mirror_path.clone(),
                ));
            }
        }

        Ok(items)
    }

    /// Extract the set of field names from `text` using the configured
    /// `field_re`. Returns a BTreeSet to make subsequent set diffs
    /// deterministic.
    fn extract_field_names<'b>(&self, text: &'b str) -> BTreeSet<&'b str> {
        let mut out = BTreeSet::new();
        for cap in self.field_re.captures_iter(text) {
            if let Some(m) = cap.get(1) {
                let name = m.as_str();
                // Skip well-known noise keywords that satisfy the regex
                // but are not actual fields.
                if matches!(
                    name,
                    "fn" | "pub" | "let" | "match" | "if" | "for" | "while"
                        | "return" | "struct" | "enum" | "impl" | "trait"
                        | "use" | "mod" | "where" | "self" | "Self" | "ref"
                        | "type" | "as" | "in" | "loop" | "move" | "async"
                        | "await" | "dyn" | "const" | "static" | "extern"
                ) {
                    continue;
                }
                out.insert(name);
            }
        }
        out
    }

    /// Extract the SCHEMA_VERSION literal. Returns `None` if absent or
    /// unparseable. Multiple matches: first one wins (canonical
    /// SCHEMA_VERSION is declared once at the top of the file).
    fn extract_schema_version(&self, text: &str) -> Option<u32> {
        let cap = self.schema_version_re.captures(text)?;
        let m = cap.get(1)?;
        m.as_str().parse::<u32>().ok()
    }
}

fn read_file(path: &Path) -> Result<String, DriftDetectorError> {
    fs::read_to_string(path).map_err(|e| DriftDetectorError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_lang_drift::pair_registry::PairRegistry;
    use tempfile::TempDir;

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        fs::write(path, body).expect("write");
    }

    fn make_registry(dir: &TempDir, rust_rel: &str, mirror_rel: &str) -> PairRegistry {
        let toml_body = format!(
            r#"
[[pairs]]
name = "sidecar_orchestrator_schemas"
rust_path = "{rust_rel}"
mirror_path = "{mirror_rel}"
"#
        );
        let toml_path = dir.path().join("cross-lang-pairs.toml");
        fs::write(&toml_path, toml_body).expect("write toml");
        PairRegistry::load_from(&toml_path).expect("load")
    }

    #[test]
    fn detects_no_drift_when_fields_match() {
        let dir = TempDir::new().expect("tempdir");
        let rust_rel = "rust_src.rs";
        let mirror_rel = "mirror.py";
        let rust_text = "pub struct Foo { pub alpha: u32, pub beta: String }";
        let mirror_text = "class Foo:\n    alpha: int\n    beta: str";
        write(&dir.path().join(rust_rel), rust_text);
        write(&dir.path().join(mirror_rel), mirror_text);

        let reg = make_registry(&dir, rust_rel, mirror_rel);
        let detector = DriftDetector::new(&reg).expect("detector");
        let results = detector.detect_pairs(dir.path()).expect("detect");
        assert_eq!(results.len(), 1);
        assert!(
            results[0].drift_items.is_empty(),
            "expected zero drift, got {:?}",
            results[0].drift_items
        );
    }

    #[test]
    fn detects_added_field_on_rust_side() {
        let dir = TempDir::new().expect("tempdir");
        let rust_rel = "rust_src.rs";
        let mirror_rel = "mirror.py";
        let rust_text = "pub struct Foo { pub alpha: u32, pub beta: String, pub gamma: bool }";
        let mirror_text = "class Foo:\n    alpha: int\n    beta: str";
        write(&dir.path().join(rust_rel), rust_text);
        write(&dir.path().join(mirror_rel), mirror_text);

        let reg = make_registry(&dir, rust_rel, mirror_rel);
        let detector = DriftDetector::new(&reg).expect("detector");
        let results = detector.detect_pairs(dir.path()).expect("detect");
        assert_eq!(results.len(), 1);
        let drift = &results[0].drift_items;
        let added: Vec<&DriftKind> = drift
            .iter()
            .filter_map(|d| match &d.kind {
                DriftKind::TypeFieldAdded { .. } => Some(&d.kind),
                _ => None,
            })
            .collect();
        assert_eq!(added.len(), 1, "expected one TypeFieldAdded, got {drift:?}");
        if let DriftKind::TypeFieldAdded { field } = added[0] {
            assert_eq!(field, "gamma");
        }
    }

    #[test]
    fn detects_missing_mirror_side() {
        let dir = TempDir::new().expect("tempdir");
        let rust_rel = "rust_src.rs";
        let mirror_rel = "absent.py";
        write(&dir.path().join(rust_rel), "pub struct X {}");
        let reg = make_registry(&dir, rust_rel, mirror_rel);
        let detector = DriftDetector::new(&reg).expect("detector");
        let results = detector.detect_pairs(dir.path()).expect("detect");
        let drift = &results[0].drift_items;
        assert_eq!(drift.len(), 1);
        match &drift[0].kind {
            DriftKind::PairSideMissing { .. } => {}
            other => panic!("expected PairSideMissing, got {other:?}"),
        }
    }

    #[test]
    fn detects_schema_version_mismatch() {
        let dir = TempDir::new().expect("tempdir");
        let rust_rel = "rust_src.rs";
        let mirror_rel = "mirror.py";
        let rust_text = "pub struct Foo { pub a: u32 }\nconst SCHEMA_VERSION: i64 = 137;";
        let mirror_text = "class Foo:\n    a: int\nSCHEMA_VERSION = 135";
        write(&dir.path().join(rust_rel), rust_text);
        write(&dir.path().join(mirror_rel), mirror_text);

        let reg = make_registry(&dir, rust_rel, mirror_rel);
        let detector = DriftDetector::new(&reg).expect("detector");
        let results = detector.detect_pairs(dir.path()).expect("detect");
        let drift = &results[0].drift_items;
        let has_version_drift = drift.iter().any(|d| {
            matches!(
                d.kind,
                DriftKind::SchemaVersionMismatch {
                    rust_version: 137,
                    mirror_version: 135
                }
            )
        });
        assert!(has_version_drift, "expected SchemaVersionMismatch, got {drift:?}");
    }
}
