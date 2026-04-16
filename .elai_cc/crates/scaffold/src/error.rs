use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ScaffoldError {
    #[error("template not found: {0}")]
    TemplateNotFound(String),

    #[error("template source directory missing: {0}")]
    SourceMissing(PathBuf),

    #[error("manifest missing at: {0}")]
    ManifestMissing(PathBuf),

    #[error("target directory not empty: {0}")]
    TargetDirNotEmpty(PathBuf),

    #[error("unresolved variable: {{{{{0}}}}}")]
    VariableMissing(String),

    #[error("invalid UTF-8 in path: {0}")]
    InvalidUtf8(PathBuf),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}
