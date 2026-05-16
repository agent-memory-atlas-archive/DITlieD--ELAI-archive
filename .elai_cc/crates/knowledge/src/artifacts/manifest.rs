//! Artifact manifest loader — parses [[knowledge.artifacts]] from elai_cc.toml
//! with a fallback to .elai_artifacts.json (SocratiCode parity).
//!
//! Security: every resolved path is verified to canonicalize inside the
//! project root before an ArtifactSpec is returned. Paths that escape (e.g.
//! `../etc/passwd`) are rejected with ArtifactError::PathEscape.
//!
//! Name validation: `^[a-zA-Z0-9_-]+$` enforced at load time via a simple
//! char-class scan (no regex crate — the knowledge crate uses regex only for
//! grammar predicates).

use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;
use tracing::debug;

use super::types::{ArtifactManifest, ArtifactSpec};
use crate::runtime_config::ElaiTomlKnowledgeSlice;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Primary config source inside elai_cc.toml.
const ELAI_TOML_FILENAME: &str = "elai_cc.toml";

/// Fallback standalone file (SocratiCode parity).
const FALLBACK_JSON_FILENAME: &str = ".elai_artifacts.json";

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("artifact io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("artifact toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("artifact json parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("artifact name is invalid (must match [a-zA-Z0-9_-]+): '{0}'")]
    InvalidName(String),

    #[error("duplicate artifact name: '{0}'")]
    DuplicateName(String),

    #[error("artifact path is empty for artifact '{0}'")]
    EmptyPath(String),

    #[error("artifact description is empty for artifact '{0}'")]
    EmptyDescription(String),

    #[error("artifact path escapes project root: '{0}'")]
    PathEscape(String),

    #[error("artifact path does not exist: '{0}'")]
    PathNotFound(String),

    #[error("artifact sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("artifact json serialize error: {0}")]
    JsonSerialize(String),

    #[error("artifact content_hash malformed: {0}")]
    InvalidHash(String),

    #[error("artifact store mutex poisoned")]
    MutexPoisoned,

    #[error("artifact path is not valid UTF-8: {0}")]
    InvalidPath(String),

    #[error("artifact error: {0}")]
    Other(String),
}

// ---------------------------------------------------------------------------
// JSON fallback types (mirrors .elai_artifacts.json schema)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct JsonArtifactEntry {
    name: String,
    path: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct JsonArtifactsFile {
    artifacts: Vec<JsonArtifactEntry>,
}

// ---------------------------------------------------------------------------
// ArtifactManifest::load
// ---------------------------------------------------------------------------

impl ArtifactManifest {
    /// Load and validate the artifact manifest for `project_root`.
    ///
    /// Precedence:
    /// 1. `[[knowledge.artifacts]]` blocks in `elai_cc.toml`
    /// 2. `.elai_artifacts.json` at project root (fallback if TOML block absent)
    ///
    /// Returns an empty manifest (not an error) when neither source defines
    /// any artifacts.
    pub fn load(project_root: &Path) -> Result<Self, ArtifactError> {
        let canonical_root = project_root
            .canonicalize()
            .map_err(|e| ArtifactError::Io(e))?;

        let toml_items = load_from_toml(project_root)?;

        let raw_entries: Vec<(String, String, String)> = if !toml_items.is_empty() {
            toml_items
        } else {
            load_from_json(project_root)?
        };

        if raw_entries.is_empty() {
            return Ok(ArtifactManifest::default());
        }

        validate_names_unique(&raw_entries)?;

        let mut items = Vec::with_capacity(raw_entries.len());
        for (name, path_str, description) in raw_entries {
            validate_name(&name)?;
            if path_str.trim().is_empty() {
                return Err(ArtifactError::EmptyPath(name));
            }
            if description.trim().is_empty() {
                return Err(ArtifactError::EmptyDescription(name));
            }
            let spec = resolve_spec(&name, &path_str, &description, &canonical_root)?;
            items.push(spec);
        }

        Ok(ArtifactManifest { items })
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn load_from_toml(project_root: &Path) -> Result<Vec<(String, String, String)>, ArtifactError> {
    let toml_path = project_root.join(ELAI_TOML_FILENAME);
    if !toml_path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&toml_path)?;
    let slice: ElaiTomlKnowledgeSlice = toml::from_str(&content)?;
    let items = slice
        .knowledge
        .artifacts
        .into_iter()
        .map(|a| (a.name, a.path, a.description))
        .collect();
    Ok(items)
}

fn load_from_json(project_root: &Path) -> Result<Vec<(String, String, String)>, ArtifactError> {
    let json_path = project_root.join(FALLBACK_JSON_FILENAME);
    if !json_path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&json_path)?;
    let parsed: JsonArtifactsFile = serde_json::from_str(&content)?;
    let items = parsed
        .artifacts
        .into_iter()
        .map(|a| (a.name, a.path, a.description))
        .collect();
    Ok(items)
}

/// Validate that `name` matches `[a-zA-Z0-9_-]+` using a char-class scan.
/// No regex crate needed for this simple pattern.
fn validate_name(name: &str) -> Result<(), ArtifactError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ArtifactError::InvalidName(name.to_string()));
    }
    Ok(())
}

fn validate_names_unique(
    entries: &[(String, String, String)],
) -> Result<(), ArtifactError> {
    let mut seen = std::collections::BTreeSet::new();
    for (name, _, _) in entries {
        let normalized = name.trim().to_lowercase();
        if !seen.insert(normalized) {
            return Err(ArtifactError::DuplicateName(name.clone()));
        }
    }
    Ok(())
}

/// Resolve a raw path string into a validated [`ArtifactSpec`].
///
/// Canonicalizes the path relative to `canonical_root` and rejects anything
/// that resolves outside the project root.
fn resolve_spec(
    name: &str,
    path_str: &str,
    description: &str,
    canonical_root: &Path,
) -> Result<ArtifactSpec, ArtifactError> {
    let raw: PathBuf = if Path::new(path_str).is_absolute() {
        PathBuf::from(path_str)
    } else {
        canonical_root.join(path_str)
    };

    let canonical = raw
        .canonicalize()
        .map_err(|_| ArtifactError::PathNotFound(path_str.to_string()))?;

    if !canonical.starts_with(canonical_root) {
        return Err(ArtifactError::PathEscape(path_str.to_string()));
    }

    let is_dir = canonical.is_dir();

    if !canonical.exists() {
        debug!(
            artifact = name,
            path = %canonical.display(),
            "artifact path does not exist after canonicalize (TOCTOU race) — will be skipped at index time"
        );
    }

    Ok(ArtifactSpec {
        name: name.to_string(),
        path: canonical,
        description: description.to_string(),
        is_dir,
    })
}
