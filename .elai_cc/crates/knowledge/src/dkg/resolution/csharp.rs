// B-30 Pass C: C# namespace resolver.
//
// Walks .cs files under project_root in lexicographic order (critical for
// determinism — see CSHARP_NAMESPACE_MAP pattern and audit-pass-A.md).
// Reads each file, extracts namespace declarations via regex (both block-scope
// and file-scoped forms), and builds BTreeMap<String, Vec<PathBuf>>.
//
// Supported forms:
//   namespace Foo.Bar { ... }   (block-scope)
//   namespace Foo.Bar;           (file-scoped, C# 10+)
//
// Multiple files can declare the same namespace (partial classes); all are
// appended in lexicographic file order.
//
// resolve(import_str) tries:
//   1. Exact namespace match ("App.Models")
//   2. Strip last dot-segment and retry ("App.Models.User" -> "App.Models")
//   Returns first PathBuf in the Vec (lexicographically earliest file).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;

use super::ImportResolver;

static NS_REGEX: OnceLock<Regex> = OnceLock::new();

fn ns_regex() -> &'static Regex {
    NS_REGEX.get_or_init(|| {
        // Matches both:
        //   namespace Foo.Bar {   (block-scope)
        //   namespace Foo.Bar;    (file-scoped)
        // Anchored with ^\s* (allows indented nested namespaces, multiline).
        Regex::new(r"(?m)^\s*namespace\s+([A-Za-z_][A-Za-z0-9_.]*)\s*[;{]")
            .expect("NS_REGEX is valid")
    })
}

/// C# namespace-to-file resolver.
pub struct CsNamespaceResolver {
    ns_to_paths: BTreeMap<String, Vec<PathBuf>>,
}

impl CsNamespaceResolver {
    /// Build the resolver by walking `project_root` for .cs files.
    /// Files are processed in lexicographic path order (CRITICAL for determinism).
    /// Missing or unreadable files are silently skipped.
    pub fn new(project_root: &Path) -> Self {
        let regex = ns_regex();
        let mut ns_to_paths: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

        // Collect all .cs paths first, then sort lexicographically.
        let mut cs_paths: Vec<PathBuf> = ignore::WalkBuilder::new(project_root)
            .sort_by_file_path(|a, b| a.cmp(b))
            .build()
            .flatten()
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| x == "cs")
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        // Explicit sort as belt-and-suspenders (ignore WalkBuilder sort is by
        // file name within a directory; full-path sort ensures cross-dir order).
        cs_paths.sort();

        for cs_path in cs_paths {
            let content = match std::fs::read_to_string(&cs_path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            for cap in regex.captures_iter(&content) {
                let ns = cap[1].to_string();
                let entry = ns_to_paths.entry(ns).or_default();
                // Dedup: don't add the same file twice for the same namespace.
                if !entry.contains(&cs_path) {
                    entry.push(cs_path.clone());
                }
            }
        }

        Self { ns_to_paths }
    }
}

impl ImportResolver for CsNamespaceResolver {
    fn resolve(&self, import_str: &str) -> Option<PathBuf> {
        // 1. Exact namespace match.
        if let Some(paths) = self.ns_to_paths.get(import_str) {
            return paths.first().cloned();
        }
        // 2. Strip last dot-segment (handles "Foo.Bar.ClassName" -> "Foo.Bar").
        if let Some(dot_pos) = import_str.rfind('.') {
            let ns_part = &import_str[..dot_pos];
            if let Some(paths) = self.ns_to_paths.get(ns_part) {
                return paths.first().cloned();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(dir: &Path, rel: &str, content: &str) {
        let full = dir.join(rel);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(full, content).unwrap();
    }

    #[test]
    fn block_and_file_scoped_namespaces() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "src/Models/User.cs", "namespace App.Models { class User {} }");
        write(tmp.path(), "src/Services/Auth.cs", "namespace App.Services;");
        let r = CsNamespaceResolver::new(tmp.path());
        let user = r.resolve("App.Models").expect("App.Models");
        assert!(user.to_string_lossy().contains("User.cs"), "got: {user:?}");
        let auth = r.resolve("App.Services").expect("App.Services");
        assert!(auth.to_string_lossy().contains("Auth.cs"), "got: {auth:?}");
    }

    #[test]
    fn strip_class_segment_match() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "src/Models/User.cs", "namespace App.Models { class User {} }");
        let r = CsNamespaceResolver::new(tmp.path());
        let result = r.resolve("App.Models.User").expect("App.Models.User -> App.Models");
        assert!(result.to_string_lossy().contains("User.cs"));
    }

    #[test]
    fn lexicographic_determinism() {
        let tmp = TempDir::new().unwrap();
        // Two files, same namespace — lexicographically earlier should win.
        write(tmp.path(), "src/A/Alpha.cs", "namespace Shared.Util {}");
        write(tmp.path(), "src/B/Beta.cs", "namespace Shared.Util {}");
        let r1 = CsNamespaceResolver::new(tmp.path());
        let r2 = CsNamespaceResolver::new(tmp.path());
        let p1 = r1.resolve("Shared.Util").unwrap();
        let p2 = r2.resolve("Shared.Util").unwrap();
        assert_eq!(p1, p2, "must be deterministic across runs");
        // Alpha.cs sorts before Beta.cs
        assert!(p1.to_string_lossy().contains("Alpha.cs"), "Alpha.cs expected, got {p1:?}");
    }
}
