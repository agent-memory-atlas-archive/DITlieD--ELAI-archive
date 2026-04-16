use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::ScaffoldError;
use crate::substitute::substitute_variables;

pub const TEMPLATES_WEB_SUBDIR: &str = "templates/web";
pub const MANIFEST_FILENAME: &str = "manifest.toml";

const SUBSTITUTABLE_EXTENSIONS: &[&str] = &[
    "json", "toml", "js", "ts", "svelte", "jsx", "tsx", "css", "html", "md", "mjs", "cjs",
    "astro", "txt", "gitignore", "map",
];

const SUBSTITUTABLE_FILENAMES: &[&str] = &[".gitignore", "README", "README.md"];

pub fn apply_template(
    template_name: &str,
    target_dir: &Path,
    variables: &BTreeMap<String, String>,
) -> Result<(), ScaffoldError> {
    let root = templates_root()?;
    apply_template_from_root(&root, template_name, target_dir, variables)
}

pub fn apply_template_from_root(
    templates_root: &Path,
    template_name: &str,
    target_dir: &Path,
    variables: &BTreeMap<String, String>,
) -> Result<(), ScaffoldError> {
    let src_root = templates_root.join(template_name);
    if !src_root.exists() || !src_root.is_dir() {
        return Err(ScaffoldError::TemplateNotFound(template_name.to_string()));
    }

    if target_dir.exists() {
        let mut has_entries = false;
        for _ in std::fs::read_dir(target_dir)? {
            has_entries = true;
            break;
        }
        if has_entries {
            return Err(ScaffoldError::TargetDirNotEmpty(target_dir.to_path_buf()));
        }
    } else {
        std::fs::create_dir_all(target_dir)?;
    }

    let mut rel_entries: Vec<PathBuf> = Vec::new();
    collect_recursive(&src_root, &src_root, &mut rel_entries)?;
    rel_entries.sort();

    for rel in &rel_entries {
        let src = src_root.join(rel);
        let dst = target_dir.join(rel);
        if src.is_dir() {
            std::fs::create_dir_all(&dst)?;
            continue;
        }
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if is_text_file(rel) {
            let bytes = std::fs::read(&src)?;
            let text = std::str::from_utf8(&bytes)
                .map_err(|_| ScaffoldError::InvalidUtf8(src.clone()))?;
            let substituted = substitute_variables(text, variables)?;
            std::fs::write(&dst, substituted.as_bytes())?;
        } else {
            std::fs::copy(&src, &dst)?;
        }
    }
    tracing::debug!(
        template = %template_name,
        files = rel_entries.len(),
        "apply_template complete"
    );
    Ok(())
}

pub fn list_templates(templates_root: &Path) -> Result<Vec<String>, ScaffoldError> {
    if !templates_root.exists() || !templates_root.is_dir() {
        return Err(ScaffoldError::SourceMissing(templates_root.to_path_buf()));
    }
    let mut out: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(templates_root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name();
            let name_str = name
                .to_str()
                .ok_or_else(|| ScaffoldError::InvalidUtf8(entry.path()))?;
            out.push(name_str.to_string());
        }
    }
    out.sort();
    Ok(out)
}

fn templates_root() -> Result<PathBuf, ScaffoldError> {
    let manifest_dir = env_manifest_dir()?;
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| ScaffoldError::SourceMissing(manifest_dir.clone()))?;
    Ok(workspace_root.join(TEMPLATES_WEB_SUBDIR))
}

fn env_manifest_dir() -> Result<PathBuf, ScaffoldError> {
    let raw = env!("CARGO_MANIFEST_DIR");
    Ok(PathBuf::from(raw))
}

fn collect_recursive(
    root: &Path,
    dir: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), ScaffoldError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map_err(|_| ScaffoldError::InvalidUtf8(path.clone()))?
            .to_path_buf();
        if entry.file_type()?.is_dir() {
            out.push(rel.clone());
            collect_recursive(root, &path, out)?;
        } else {
            out.push(rel);
        }
    }
    Ok(())
}

fn is_text_file(rel: &Path) -> bool {
    if let Some(fname) = rel.file_name().and_then(|n| n.to_str()) {
        if SUBSTITUTABLE_FILENAMES.iter().any(|f| *f == fname) {
            return true;
        }
    }
    if let Some(ext) = rel.extension().and_then(|e| e.to_str()) {
        return SUBSTITUTABLE_EXTENSIONS.iter().any(|e| *e == ext);
    }
    false
}
