use elai_knowledge::symbol_graph::{cache::SymbolGraphCache, sharding::SYMBOL_FILE_LRU_SIZE};
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

    /// Reduce a raw claim ref to the simple symbol name the DKG name index is keyed by.
    /// The index stores simple names, so `elai_core::paths::dkg_db_path(&root)` must become
    /// `dkg_db_path`. Order matters: cut the argument list and the generic list BEFORE taking
    /// the terminal `::` segment, because both can themselves contain `::`.
    fn normalized_symbol(raw: &str) -> String {
        let mut name = raw.trim().trim_matches(|c| c == '`' || c == '"' || c == '\'');
        // An Api-kind ref looks like `fn foo(a: A) -> B`; drop the signature tail first.
        if let Some(index) = name.find('(') {
            name = &name[..index];
        }
        if let Some(index) = name.find('<') {
            name = &name[..index];
        }
        let name = name.trim();
        // `fn foo` / `pub fn foo` shapes: the symbol is the last whitespace-separated token.
        let name = name.split_whitespace().last().unwrap_or("");
        let name = name.rsplit("::").next().unwrap_or("");
        name.trim().to_owned()
    }
}

/// Resolves file references, lockfile package references, and symbol/API references. Symbols
/// resolve against the on-disk DKG symbol name index; an absent or unreadable index resolves to
/// `false`, which is the pre-slice behaviour.
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

    fn resolve_symbol(&self, raw: &str) -> bool {
        let name = Self::normalized_symbol(raw);
        if name.is_empty() {
            return false;
        }
        let db_path = elai_core::paths::dkg_db_path(&self.root);
        if !db_path.exists() {
            return false;
        }
        let project_id = elai_core::project_id_from_path(&self.root);
        let mut cache = match SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE) {
            Ok(cache) => cache,
            Err(_) => return false,
        };
        match cache.ensure_name_index() {
            Ok(index) => index.get(&name).is_some_and(|refs| !refs.is_empty()),
            Err(_) => false,
        }
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
