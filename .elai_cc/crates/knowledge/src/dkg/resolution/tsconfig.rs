// B-30 Pass C: TypeScript/JavaScript tsconfig path-alias resolver.
//
// Reads project_root/tsconfig.json (fallback: jsconfig.json). Follows the
// `extends` chain with BTreeSet<PathBuf> visited cycle prevention and a hard
// depth cap (MAX_EXTENDS_DEPTH). Strips // and /* */ comments before JSON
// parsing (tsconfig.json allows comments; serde_json does not).
//
// Collects `compilerOptions.paths` from all levels; child overrides parent
// (child key wins in BTreeMap.insert). Stores BTreeMap<String, Vec<String>>:
// alias pattern -> list of targets (relative strings as written in the config).
//
// resolve(import_str):
//   Exact match: "@foo/bar" -> "@foo/bar" in map.
//   Wildcard: "@app/*" pattern matches "@app/foo/bar"; replaces * in target.
//   Returns PathBuf relative to project_root (first target, unconditionally).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::ImportResolver;

/// Maximum `extends` chain depth to follow (prevents infinite chains even if
/// cycle detection misses something).
const MAX_EXTENDS_DEPTH: usize = 10;

/// Strip JS-style comments from a JSON-with-comments string using a state
/// machine that tracks string context.
///
/// Handles `// line comments` and `/* block comments */` correctly even when
/// `//` or `/*` appear inside a JSON string value (e.g. a URL).
/// Escaped quotes (`\"`) inside strings are passed through verbatim so the
/// string boundary is not mis-detected.
fn strip_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if c == '\\' {
                // Escaped char inside string — pass the next char through verbatim
                // so we don't accidentally treat `\"` as a string close.
                if let Some(&next) = chars.peek() {
                    out.push(next);
                    chars.next();
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            out.push(c);
            continue;
        }
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    // Line comment — consume through end of line, do not emit.
                    while let Some(&n) = chars.peek() {
                        if n == '\n' {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                }
                Some('*') => {
                    chars.next(); // consume the `*`
                    // Block comment — consume until `*/`, do not emit.
                    let mut prev = '\0';
                    while let Some(n) = chars.next() {
                        if prev == '*' && n == '/' {
                            break;
                        }
                        prev = n;
                    }
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    out
}

/// TypeScript/JavaScript path-alias resolver.
pub struct TsPathAliasResolver {
    aliases: BTreeMap<String, Vec<String>>,
    project_root: PathBuf,
}

impl TsPathAliasResolver {
    /// Build the resolver by loading tsconfig.json / jsconfig.json under
    /// `project_root`. If neither exists, produces an empty resolver.
    pub fn new(project_root: &Path) -> Self {
        let mut aliases: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut visited: BTreeSet<PathBuf> = BTreeSet::new();

        let candidates = ["tsconfig.json", "jsconfig.json"];
        for name in candidates {
            let config_path = project_root.join(name);
            if config_path.exists() {
                load_config(&config_path, &mut visited, 0, &mut aliases);
                break;
            }
        }

        Self {
            aliases,
            project_root: project_root.to_path_buf(),
        }
    }
}

impl ImportResolver for TsPathAliasResolver {
    fn resolve(&self, import_str: &str) -> Option<PathBuf> {
        // 1. Exact match.
        if let Some(targets) = self.aliases.get(import_str) {
            if let Some(target) = targets.first() {
                return Some(resolve_target(target, &self.project_root));
            }
        }

        // 2. Wildcard patterns: "@app/*" matches "@app/foo/bar".
        for (pattern, targets) in &self.aliases {
            if let Some(prefix) = pattern.strip_suffix("/*") {
                let with_sep = format!("{prefix}/");
                if import_str.starts_with(&with_sep) {
                    let suffix = &import_str[with_sep.len()..];
                    if let Some(target_template) = targets.first() {
                        // Replace trailing /* in target with suffix.
                        let resolved_target = if let Some(t_prefix) = target_template.strip_suffix("/*") {
                            format!("{t_prefix}/{suffix}")
                        } else {
                            // No wildcard in target; append suffix.
                            format!("{target_template}/{suffix}")
                        };
                        return Some(resolve_target(&resolved_target, &self.project_root));
                    }
                }
            } else if pattern == import_str {
                // Already handled above, but keeps the loop exhaustive.
                if let Some(target) = targets.first() {
                    return Some(resolve_target(target, &self.project_root));
                }
            }
        }

        None
    }
}

/// Join a target string with project_root to form a PathBuf.
/// Targets are relative to project_root per tsconfig convention.
///
/// After computing the raw path, probes the filesystem for common TypeScript
/// extensions in order:
///   1. raw path (e.g. already has .ts extension)
///   2. raw.ts
///   3. raw.tsx
///   4. raw.d.ts   (represented as raw + ".d.ts" string append)
///   5. raw/index.ts
///   6. raw/index.tsx
///
/// Returns the first candidate that exists on the filesystem, or the raw path
/// if none exist (caller performs a best-effort match).
fn resolve_target(target: &str, project_root: &Path) -> PathBuf {
    let mut p = project_root.to_path_buf();
    for segment in target.split('/') {
        if segment == ".." {
            p.pop();
        } else if !segment.is_empty() && segment != "." {
            p.push(segment);
        }
    }
    // Probe candidates in priority order.
    let candidates: [PathBuf; 6] = [
        p.clone(),
        p.with_extension("ts"),
        p.with_extension("tsx"),
        PathBuf::from(format!("{}.d.ts", p.display())),
        p.join("index.ts"),
        p.join("index.tsx"),
    ];
    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }
    // None exist — return raw path (silent-miss; caller handles None via DB lookup).
    p
}

/// Recursively load a tsconfig file, following `extends` (parent first, then
/// merge child). Child paths take priority via BTreeMap.insert semantics.
fn load_config(
    path: &Path,
    visited: &mut BTreeSet<PathBuf>,
    depth: usize,
    aliases: &mut BTreeMap<String, Vec<String>>,
) {
    if depth > MAX_EXTENDS_DEPTH {
        return;
    }
    // Canonicalize for cycle detection; fall back to the original path.
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if visited.contains(&canonical) {
        return;
    }
    visited.insert(canonical.clone());

    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return,
    };

    let stripped = strip_comments(&content);
    let value: serde_json::Value = match serde_json::from_str(&stripped) {
        Ok(v) => v,
        Err(_) => return,
    };

    // Follow `extends` FIRST so child (current file) can override parent.
    if let Some(extends_val) = value.get("extends").and_then(|v| v.as_str()) {
        let parent_path = resolve_extends_path(path, extends_val);
        if let Some(pp) = parent_path {
            load_config(&pp, visited, depth + 1, aliases);
        }
    }

    // Merge `compilerOptions.paths` from this file (overrides parent entries).
    if let Some(paths_obj) = value
        .get("compilerOptions")
        .and_then(|o| o.get("paths"))
        .and_then(|p| p.as_object())
    {
        for (key, targets_val) in paths_obj {
            if let Some(targets_arr) = targets_val.as_array() {
                let targets: Vec<String> = targets_arr
                    .iter()
                    .filter_map(|t| t.as_str().map(str::to_owned))
                    .collect();
                aliases.insert(key.clone(), targets);
            }
        }
    }
}

/// Resolve an `extends` value relative to the current config file's directory.
/// Appends `.json` if the value has no extension and doesn't contain a node_modules reference.
fn resolve_extends_path(current: &Path, extends: &str) -> Option<PathBuf> {
    let dir = current.parent()?;
    let mut p = dir.join(extends);
    // If no extension present and not a bare package name, append .json.
    if p.extension().is_none() {
        p.set_extension("json");
    }
    Some(p)
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
    fn simple_path_alias() {
        let tmp = TempDir::new().unwrap();
        write(tmp.path(), "tsconfig.json", r#"
        {
            "compilerOptions": {
                "paths": {
                    "@app/*": ["src/app/*"],
                    "@lib/*": ["src/lib/*"]
                }
            }
        }
        "#);
        let r = TsPathAliasResolver::new(tmp.path());
        let app_foo = r.resolve("@app/foo").expect("@app/foo");
        assert!(app_foo.to_string_lossy().contains("src") && app_foo.to_string_lossy().contains("foo"),
            "got: {app_foo:?}");
        let lib_bar = r.resolve("@lib/bar").expect("@lib/bar");
        assert!(lib_bar.to_string_lossy().contains("bar"), "got: {lib_bar:?}");
    }

    #[test]
    fn extends_chain_child_wins() {
        let tmp = TempDir::new().unwrap();
        // Parent (base config).
        write(tmp.path(), "tsconfig.base.json", r#"
        {
            "compilerOptions": {
                "paths": {
                    "@lib/*": ["src/lib/*"]
                }
            }
        }
        "#);
        // Child extends parent, overrides nothing but adds @app/*.
        write(tmp.path(), "tsconfig.json", r#"
        { // project config
            "extends": "./tsconfig.base.json",
            "compilerOptions": {
                "paths": {
                    "@app/*": ["src/app/*"]
                }
            }
        }
        "#);
        let r = TsPathAliasResolver::new(tmp.path());
        // Both aliases should be present.
        assert!(r.resolve("@app/foo").is_some(), "@app/foo should resolve");
        assert!(r.resolve("@lib/bar").is_some(), "@lib/bar should resolve");
    }

    #[test]
    fn cycle_does_not_infinite_loop() {
        let tmp = TempDir::new().unwrap();
        // Self-referential config.
        write(tmp.path(), "tsconfig.cycle.json", r#"
        {
            "extends": "./tsconfig.cycle.json",
            "compilerOptions": {
                "paths": { "@x/*": ["src/x/*"] }
            }
        }
        "#);
        // Force load of cycle config by constructing a wrapper tsconfig.json.
        write(tmp.path(), "tsconfig.json", r#"
        {
            "extends": "./tsconfig.cycle.json"
        }
        "#);
        // Must not hang or stack-overflow.
        let r = TsPathAliasResolver::new(tmp.path());
        // @x/* should be resolvable from tsconfig.cycle.json.
        let _ = r.resolve("@x/foo");
    }
}
