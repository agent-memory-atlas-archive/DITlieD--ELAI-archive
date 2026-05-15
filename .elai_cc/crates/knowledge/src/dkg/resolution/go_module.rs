// B-30 Pass C: Go module resolver.
//
// Reads project_root/go.mod, parses the `module <path>` directive, and builds
// module_prefix. resolve(import_str) strips the prefix and joins with
// project_root to return a directory PathBuf (not a file).
//
// Limitations (documented, not handled):
// - Parenthesized `module ( ... )` form.
// - `replace` directives.
// - `go.work` multi-module workspaces.
// These are deferred per SocratiCode PATTERN_GO_MODULE_INFO notes.

use std::path::{Path, PathBuf};

use super::ImportResolver;

/// Go module prefix resolver.
pub struct GoModuleResolver {
    module_prefix: Option<String>,
    project_root: PathBuf,
}

impl GoModuleResolver {
    /// Build the resolver by reading `project_root/go.mod`.
    /// If go.mod is missing or has no `module` directive, `module_prefix` is None.
    pub fn new(project_root: &Path) -> Self {
        let module_prefix = read_module_prefix(&project_root.join("go.mod"));
        Self {
            module_prefix,
            project_root: project_root.to_path_buf(),
        }
    }

    /// Return the parsed module prefix, or empty string if none.
    pub fn module_prefix(&self) -> &str {
        self.module_prefix.as_deref().unwrap_or("")
    }
}

impl ImportResolver for GoModuleResolver {
    fn resolve(&self, import_str: &str) -> Option<PathBuf> {
        let prefix = self.module_prefix.as_deref()?;
        // Exact match: import_str IS the module root (e.g. "github.com/myorg/myrepo")
        if import_str == prefix {
            return Some(self.project_root.clone());
        }
        // Prefix match with separator: import_str starts with "<prefix>/"
        let with_sep = format!("{prefix}/");
        if let Some(rel) = import_str.strip_prefix(&with_sep) {
            // rel is a directory path (forward slashes per Go convention).
            // Join with project_root using the OS path separator.
            let mut dir = self.project_root.clone();
            for segment in rel.split('/') {
                dir.push(segment);
            }
            return Some(dir);
        }
        None
    }
}

/// Parse the first `module <path>` directive from go.mod content.
/// Skips blank lines and `//` comment lines. Returns None on any failure.
fn read_module_prefix(go_mod_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(go_mod_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        // Skip blank lines and comments.
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("module") {
            let path = rest.trim();
            if !path.is_empty() {
                // Capture only the first whitespace-delimited token.
                let token = path.split_whitespace().next().unwrap_or(path);
                return Some(token.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn resolves_module_subpackage() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("go.mod"), "module github.com/myorg/myrepo\n\ngo 1.21\n").unwrap();
        fs::create_dir_all(tmp.path().join("pkg/service")).unwrap();
        fs::write(tmp.path().join("pkg/service/main.go"), "package service").unwrap();
        let r = GoModuleResolver::new(tmp.path());
        assert_eq!(r.module_prefix(), "github.com/myorg/myrepo");
        let resolved = r.resolve("github.com/myorg/myrepo/pkg/service")
            .expect("should resolve subpackage");
        assert!(resolved.ends_with("pkg/service") || resolved.ends_with("pkg\\service"),
            "got: {resolved:?}");
    }

    #[test]
    fn stdlib_returns_none() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("go.mod"), "module github.com/myorg/myrepo\n\ngo 1.21\n").unwrap();
        let r = GoModuleResolver::new(tmp.path());
        assert!(r.resolve("fmt").is_none(), "stdlib should not resolve");
        assert!(r.resolve("github.com/other/repo/pkg").is_none(), "external should not resolve");
    }

    #[test]
    fn no_go_mod_returns_none() {
        let tmp = TempDir::new().unwrap();
        let r = GoModuleResolver::new(tmp.path());
        assert_eq!(r.module_prefix(), "");
        assert!(r.resolve("github.com/myorg/myrepo/pkg").is_none());
    }
}
