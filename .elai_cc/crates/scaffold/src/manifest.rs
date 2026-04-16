use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::ScaffoldError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateEntry {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub build_command: String,
    #[serde(default)]
    pub dev_command: String,
    #[serde(default)]
    pub dev_port: u16,
    #[serde(default)]
    pub recommended_for: Vec<String>,
    #[serde(default)]
    pub model_familiarity: String,
    pub sections: Vec<String>,
    pub theme_system: String,
    #[serde(default)]
    pub validation_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateManifest {
    #[serde(rename = "template", default)]
    pub templates: Vec<TemplateEntry>,
}

pub fn load_manifest(path: &Path) -> Result<TemplateManifest, ScaffoldError> {
    if !path.exists() {
        return Err(ScaffoldError::ManifestMissing(path.to_path_buf()));
    }
    let bytes = std::fs::read(path)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| ScaffoldError::InvalidUtf8(path.to_path_buf()))?;
    let manifest: TemplateManifest = toml::from_str(text)?;
    Ok(manifest)
}
