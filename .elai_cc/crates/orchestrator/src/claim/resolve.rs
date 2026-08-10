use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct WorkspaceRefResolver {
    root: PathBuf,
    packages: BTreeSet<String>,
}

impl WorkspaceRefResolver {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let lock_path = [root.join("Cargo.lock"), root.join(".elai_cc/Cargo.lock")]
            .into_iter()
            .find(|path| path.exists());
        let packages = lock_path
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|contents| {
                let mut packages = BTreeSet::new();
                let mut in_package = false;
                for line in contents.lines() {
                    let trimmed = line.trim();
                    if trimmed == "[[package]]" {
                        in_package = true;
                    } else if in_package && trimmed.starts_with("name = \"") {
                        let rest = &trimmed[8..];
                        if let Some(end) = rest.find('"') {
                            packages.insert(rest[..end].to_owned());
                        }
                        in_package = false;
                    }
                }
                packages
            })
            .unwrap_or_default();
        Self { root, packages }
    }

    fn normalized_path(raw: &str) -> String {
        let mut path = raw.trim().trim_matches(|c| c == '"' || c == '\'').to_owned();
        for _ in 0..2 {
            if let Some(index) = path.find(':') {
                if path[index + 1..].starts_with(|c: char| c.is_ascii_digit()) {
                    path.truncate(index);
                }
            }
        }
        if let Some(index) = path.find('#') {
            path.truncate(index);
        }
        path.trim().to_owned()
    }
}

/// Resolves file and lockfile package references in the workspace. Symbol and API references
/// remain unresolved: this crate has no symbol index, and a real symbol resolver is a later slice.
impl crate::claim::extract::ClaimRefResolver for WorkspaceRefResolver {
    fn resolve_path(&self, raw: &str) -> bool {
        let rel = Self::normalized_path(raw);
        if rel.is_empty()
            || rel.starts_with('/')
            || rel.starts_with('~')
            || rel.starts_with('\\')
            || rel.contains("..")
            || rel.contains(':')
        {
            return false;
        }
        // One or two stat calls per file-shaped reference, bounded by claim count, on a non-hot path.
        self.root.join(&rel).exists() || self.root.join(".elai_cc").join(&rel).exists()
    }

    fn resolve_package(&self, raw: &str) -> bool {
        let name = raw
            .split_once('@')
            .map(|(name, _)| name)
            .or_else(|| raw.split_once('=').map(|(name, _)| name))
            .unwrap_or(raw)
            .trim();
        !name.is_empty() && self.packages.contains(name)
    }
}

/// Process-wide resolver built once from the current working directory.
pub fn workspace_resolver() -> Option<&'static WorkspaceRefResolver> {
    static RESOLVER: OnceLock<Option<WorkspaceRefResolver>> = OnceLock::new();
    RESOLVER
        .get_or_init(|| std::env::current_dir().ok().map(WorkspaceRefResolver::new))
        .as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::extract::ClaimRefResolver;

    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("elai-claim-resolver-{}-{}", std::process::id(), name))
    }

    #[test]
    fn existing_and_missing_paths() {
        let root = scratch("paths");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).ok();
        std::fs::write(root.join("src/lib.rs"), "").ok();
        let resolver = WorkspaceRefResolver::new(&root);
        assert!(resolver.resolve_path("src/lib.rs"));
        assert!(!resolver.resolve_path("src/missing.rs"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn traversal_is_rejected() {
        let root = scratch("traversal");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).ok();
        let resolver = WorkspaceRefResolver::new(&root);
        assert!(!resolver.resolve_path("../../etc/passwd"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn lockfile_packages_and_symbols() {
        let root = scratch("packages");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).ok();
        std::fs::write(&root.join("Cargo.lock"), "[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\n").ok();
        let resolver = WorkspaceRefResolver::new(&root);
        assert!(resolver.resolve_package("serde@1.0.0"));
        assert!(!resolver.resolve_package("absent@1.0.0"));
        assert!(!resolver.resolve_symbol("Thing::method()"));
        let _ = std::fs::remove_dir_all(root);
    }
}
