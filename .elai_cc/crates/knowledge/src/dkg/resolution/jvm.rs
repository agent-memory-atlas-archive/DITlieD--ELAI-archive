// B-30 Pass C: JVM (Java / Kotlin / Scala) import resolver.
//
// Walks .java, .kt, .kts, .scala files under project_root. For each file
// that matches a Maven/Gradle src/main/<lang>/ layout, derives the FQN from
// the path suffix and records fqn -> file_path in a BTreeMap.
//
// FQN derivation:
//   src/main/java/com/example/Foo.java  ->  "com.example.Foo"
//   src/main/kotlin/com/lib/Util.kt     ->  "com.lib.Util"
//   src/main/scala/com/lib/Bar.scala    ->  "com.lib.Bar"
//
// First-match-wins: because files are visited in lexicographic order
// (ignore::WalkBuilder sorts entries before yielding), the first entry for
// any given FQN wins if two files produce the same FQN.
//
// resolve(import_str) strips trailing ".*" (wildcard imports) and does an
// exact BTreeMap lookup. Wildcard imports cannot resolve to a single file,
// so they return None.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::ImportResolver;

/// JVM FQN-to-file resolver.
pub struct JvmResolver {
    fqn_to_path: BTreeMap<String, PathBuf>,
}

impl JvmResolver {
    /// Build the resolver by walking `project_root` for JVM source files.
    /// Missing or unreadable directories produce an empty resolver (no panic).
    ///
    /// Collects ALL matching paths first, then sorts them globally by their
    /// string representation so that first-match-wins for duplicate FQNs is
    /// globally lexicographically deterministic — not dependent on the OS
    /// directory-entry traversal order within each subtree.
    pub fn new(project_root: &Path) -> Self {
        let mut fqn_to_path: BTreeMap<String, PathBuf> = BTreeMap::new();

        // Collect all JVM source file paths first.
        let mut paths: Vec<PathBuf> = ignore::WalkBuilder::new(project_root)
            .build()
            .flatten()
            .filter_map(|entry| {
                let path = entry.into_path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_owned();
                if matches!(ext.as_str(), "java" | "kt" | "kts" | "scala") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // Global lex sort ensures deterministic first-match-wins across all
        // directories — not just within a single WalkBuilder traversal window.
        paths.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

        for path in &paths {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let lang_dir = match ext {
                "java" => "java",
                "kt" | "kts" => "kotlin",
                "scala" => "scala",
                _ => continue,
            };
            if let Some(fqn) = derive_fqn(path, lang_dir) {
                fqn_to_path.entry(fqn).or_insert_with(|| path.clone());
            }
        }

        Self { fqn_to_path }
    }
}

impl ImportResolver for JvmResolver {
    fn resolve(&self, import_str: &str) -> Option<PathBuf> {
        // Wildcard imports ("com.example.*") cannot resolve to a single file.
        if import_str.ends_with(".*") || import_str.ends_with("._") {
            return None;
        }
        self.fqn_to_path.get(import_str).cloned()
    }
}

/// Derive the FQN from a file path by locating the `src/main/<lang>/` boundary.
///
/// Returns `None` if no such boundary exists (e.g. test files, non-Maven layout).
fn derive_fqn(path: &Path, lang_dir: &str) -> Option<String> {
    // Collect path components as strings.
    let components: Vec<&str> = path
        .components()
        .filter_map(|c| {
            use std::path::Component;
            match c {
                Component::Normal(s) => s.to_str(),
                _ => None,
            }
        })
        .collect();

    // Find the index of `main` preceded by `src` and followed by `lang_dir`.
    // Pattern: [..., "src", "main", lang_dir, ...]
    let lang_start = components
        .windows(3)
        .enumerate()
        .find_map(|(i, w)| {
            if w[0] == "src" && w[1] == "main" && w[2] == lang_dir {
                Some(i + 3)
            } else {
                None
            }
        });

    let start_idx = lang_start?;

    if start_idx >= components.len() {
        return None;
    }

    // Last component is the filename; strip its extension.
    let last = *components.last()?;
    let stem = Path::new(last)
        .file_stem()
        .and_then(|s| s.to_str())?;

    // Build FQN: all components after lang_dir, except last, joined with ".",
    // then dot-separated stem appended.
    let middle: Vec<&str> = components[start_idx..components.len() - 1].to_vec();
    let mut fqn_parts = middle;
    fqn_parts.push(stem);
    Some(fqn_parts.join("."))
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
    fn empty_project_returns_none() {
        let tmp = TempDir::new().unwrap();
        let resolver = JvmResolver::new(tmp.path());
        assert!(resolver.resolve("com.example.Foo").is_none());
    }

    #[test]
    fn single_java_file_resolves() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "src/main/java/com/example/Foo.java", "package com.example; class Foo {}");
        write(tmp.path(), "src/main/kotlin/com/lib/Util.kt", "package com.lib");
        let resolver = JvmResolver::new(tmp.path());
        let foo = resolver.resolve("com.example.Foo").expect("should resolve Foo");
        assert!(foo.to_string_lossy().contains("Foo.java"));
        let util = resolver.resolve("com.lib.Util").expect("should resolve Util");
        assert!(util.to_string_lossy().contains("Util.kt"));
    }

    #[test]
    fn wildcard_import_returns_none() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "src/main/java/com/example/Foo.java", "package com.example; class Foo {}");
        let resolver = JvmResolver::new(tmp.path());
        assert!(resolver.resolve("com.example.*").is_none());
    }
}
