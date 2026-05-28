//! C-41 — Skill discovery walker.
//!
//! Walks a set of root directories (typically `.claude/skills/`,
//! `.elai_cc/skills/`, plus any user-configured roots) and parses every
//! `SKILL.md` file's YAML frontmatter into a [`SkillManifest`].
//!
//! Dedup policy: when the same skill name appears in multiple roots, the
//! FIRST occurrence wins (root list order is precedence). This mirrors the
//! existing `crates/cli/src/commands.rs::load_skills_from_roots` shadowing
//! convention so the registry returns consistent results across both
//! enumeration paths.
//!
//! Reliability: the walker never panics. I/O failures on individual files
//! are logged via `tracing::warn` and skipped; an unreadable directory
//! also produces a warning and an empty contribution. Returning a partial
//! result is preferable to failing the whole registry init because one
//! skill on disk is malformed.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::manifest::{FrontmatterFields, SkillManifest, SkillOrigin};

/// Specification for a discovery root: where to look + what shape to expect.
///
/// `path` is the directory that contains `<skill_name>/SKILL.md` entries
/// (when `origin == SkillsDir`) or `<skill_name>.md` / `<skill_name>/SKILL.md`
/// (when `origin == LegacyCommandsDir`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryRoot {
    pub path: PathBuf,
    pub origin: SkillOrigin,
}

impl DiscoveryRoot {
    /// Convenience constructor for the canonical SKILL.md layout.
    #[must_use]
    pub fn skills_dir<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            origin: SkillOrigin::SkillsDir,
        }
    }

    /// Convenience constructor for the legacy commands/ layout.
    #[must_use]
    pub fn legacy_commands<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            origin: SkillOrigin::LegacyCommandsDir,
        }
    }
}

/// Walk the configured roots and return all unique skill manifests.
///
/// The walker is deterministic for a given filesystem layout: entries
/// within a single root are sorted by `entry.file_name()` before parsing,
/// and roots are visited in the order supplied. The dedup pass uses a
/// case-insensitive name key, matching the `load_skills_from_roots`
/// behaviour on the cli side.
///
/// # Errors
/// Returns `Err` only when every supplied root fails to read; if at least
/// one root contributes (even an empty contribution), the call succeeds
/// with whatever manifests were found. Per-file parse errors are logged
/// and skipped.
pub fn find_skills(roots: &[DiscoveryRoot]) -> Result<Vec<SkillManifest>, String> {
    let mut manifests = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut any_root_succeeded = false;
    let mut last_error: Option<String> = None;

    for root in roots {
        match walk_root(root) {
            Ok(found) => {
                any_root_succeeded = true;
                for manifest in found {
                    let key = manifest.name.to_ascii_lowercase();
                    if seen.contains(&key) {
                        continue;
                    }
                    seen.insert(key);
                    manifests.push(manifest);
                }
            }
            Err(err) => {
                last_error = Some(format!(
                    "skill discovery root {} failed: {err}",
                    root.path.display()
                ));
                tracing::warn!(
                    target: "skill_registry",
                    root = %root.path.display(),
                    error = %err,
                    "skill discovery root walk failed; continuing with remaining roots",
                );
            }
        }
    }

    if !any_root_succeeded && !roots.is_empty() {
        return Err(last_error.unwrap_or_else(|| {
            "no skill discovery root could be read".to_string()
        }));
    }

    Ok(manifests)
}

/// Walk a single discovery root.
fn walk_root(root: &DiscoveryRoot) -> Result<Vec<SkillManifest>, String> {
    if !root.path.is_dir() {
        return Ok(Vec::new());
    }
    let entries = std::fs::read_dir(&root.path).map_err(|err| err.to_string())?;
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    let mut out = Vec::new();
    for entry_path in paths {
        let skill_md_path = match resolve_skill_md(&entry_path, root.origin) {
            Some(path) => path,
            None => continue,
        };
        match parse_skill_file(&skill_md_path, &entry_path, root.origin) {
            Ok(manifest) => out.push(manifest),
            Err(err) => {
                tracing::warn!(
                    target: "skill_registry",
                    path = %skill_md_path.display(),
                    error = %err,
                    "skipping malformed skill manifest",
                );
            }
        }
    }
    Ok(out)
}

/// Locate the SKILL.md file for a discovered entry, honoring the origin shape.
fn resolve_skill_md(entry_path: &Path, origin: SkillOrigin) -> Option<PathBuf> {
    match origin {
        SkillOrigin::SkillsDir => {
            if !entry_path.is_dir() {
                return None;
            }
            let candidate = entry_path.join("SKILL.md");
            if candidate.is_file() {
                Some(candidate)
            } else {
                None
            }
        }
        SkillOrigin::LegacyCommandsDir => {
            if entry_path.is_dir() {
                let candidate = entry_path.join("SKILL.md");
                if candidate.is_file() {
                    return Some(candidate);
                }
                return None;
            }
            let is_md = entry_path
                .extension()
                .is_some_and(|ext| ext.to_string_lossy().eq_ignore_ascii_case("md"));
            if is_md {
                Some(entry_path.to_path_buf())
            } else {
                None
            }
        }
    }
}

/// Read a SKILL.md and convert its frontmatter to a [`SkillManifest`].
fn parse_skill_file(
    skill_md: &Path,
    entry_path: &Path,
    origin: SkillOrigin,
) -> Result<SkillManifest, String> {
    let contents = std::fs::read_to_string(skill_md).map_err(|err| err.to_string())?;
    let fields = parse_skill_frontmatter_extended(&contents);

    let fallback_name = match origin {
        SkillOrigin::SkillsDir => entry_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string()),
        SkillOrigin::LegacyCommandsDir => entry_path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string()),
    };

    let name = fields
        .name
        .or(fallback_name)
        .ok_or_else(|| "skill missing both frontmatter name and fallback path".to_string())?;

    Ok(SkillManifest {
        name,
        version: fields.version.unwrap_or_else(|| "0.0.0".to_string()),
        triggers: fields.triggers,
        inputs: fields.inputs,
        outputs: fields.outputs,
        scope: fields.scope.unwrap_or_default(),
        origin,
        file_path: skill_md.to_path_buf(),
        // C-45: learned_credit_score is populated by the contrastive
        // attribution harness (or its CLI/pilot driver) AFTER discovery.
        // Discovery returns None so legacy SkillManifests remain
        // bit-identical to pre-C-45 form.
        learned_credit_score: None,
        // C-49: distilled_fragments are populated by the experience
        // module's post-audit hook + the CLI distill verb AFTER
        // discovery. Discovery returns an empty Vec so older manifests
        // round-trip unchanged.
        distilled_fragments: Vec::new(),
    })
}

/// Parse the YAML-style frontmatter at the top of a SKILL.md.
///
/// Recognises the existing `name:` + `description:` keys (compatible with
/// `crates/cli/src/commands.rs::parse_skill_frontmatter`) PLUS the C-41
/// extended keys:
///   - `version: "X.Y.Z"`
///   - `triggers: a, b, c` OR `triggers: ["a", "b", "c"]`
///   - `inputs: ...` (same comma-or-bracket syntax)
///   - `outputs: ...` (same)
///   - `scope: token`
///
/// Unknown keys are silently ignored. Missing keys default to empty.
#[must_use]
pub fn parse_skill_frontmatter_extended(contents: &str) -> FrontmatterFields {
    let mut lines = contents.lines();
    if lines.next().map(str::trim) != Some("---") {
        return FrontmatterFields::default();
    }

    let mut fields = FrontmatterFields::default();
    // E-43 -- track when we are inside an indented `metadata:` block so
    // `metadata.version:` / `metadata.author:` sub-keys can be parsed.
    let mut in_metadata_block = false;
    for line in lines {
        // Preserve leading whitespace for indentation detection but also
        // build a trimmed view for key/value parsing.
        if line.trim() == "---" {
            break;
        }
        // Detect indented sub-key (2+ spaces of indent) inside metadata.
        let is_indented = line.starts_with("  ") || line.starts_with("	");
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = raw_key.trim();
        let value = raw_value.trim();
        let unquoted = unquote(value);

        // Inline JSON-flow object form: `metadata: { version: "1.0.0", author: "elai" }`
        if !is_indented && key == "metadata" && value.starts_with('{') && value.ends_with('}') {
            let inner = &value[1..value.len() - 1];
            for segment in inner.split(',') {
                if let Some((k, v)) = segment.split_once(':') {
                    let k = k.trim();
                    let v = unquote(v.trim());
                    match k {
                        "version" => fields.metadata_version = Some(v),
                        "author" => fields.metadata_author = Some(v),
                        _ => {}
                    }
                }
            }
            in_metadata_block = false;
            continue;
        }

        // Top-level `metadata:` introducing an indented block.
        if !is_indented && key == "metadata" && value.is_empty() {
            in_metadata_block = true;
            continue;
        }

        // Indented sub-key (only meaningful inside an active metadata block).
        if is_indented && in_metadata_block {
            match key {
                "version" => {
                    if !unquoted.is_empty() {
                        fields.metadata_version = Some(unquoted);
                    }
                }
                "author" => {
                    if !unquoted.is_empty() {
                        fields.metadata_author = Some(unquoted);
                    }
                }
                _ => {}
            }
            continue;
        }

        // Non-indented key terminates an active metadata block.
        if !is_indented {
            in_metadata_block = false;
        }

        match key {
            "name" => {
                if !unquoted.is_empty() {
                    fields.name = Some(unquoted);
                }
            }
            "description" => {
                if !unquoted.is_empty() {
                    fields.description = Some(unquoted);
                }
            }
            "version" => {
                if !unquoted.is_empty() {
                    fields.version = Some(unquoted);
                }
            }
            "scope" => {
                if !unquoted.is_empty() {
                    fields.scope = Some(unquoted);
                }
            }
            "triggers" => {
                fields.triggers = parse_string_list(value);
            }
            "inputs" => {
                fields.inputs = parse_string_list(value);
            }
            "outputs" => {
                fields.outputs = parse_string_list(value);
            }
            // E-43 agentskills.io v1 additions.
            "compatibility" => {
                if !unquoted.is_empty() {
                    fields.compatibility = Some(unquoted);
                }
            }
            "license" => {
                if !unquoted.is_empty() {
                    fields.license = Some(unquoted);
                }
            }
            _ => {}
        }
    }
    fields
}

/// Strip a single layer of double/single quotes from a frontmatter value.
fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    let stripped = trimmed
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .or_else(|| {
            trimmed
                .strip_prefix('\'')
                .and_then(|inner| inner.strip_suffix('\''))
        })
        .unwrap_or(trimmed);
    stripped.trim().to_string()
}

/// Parse a comma-or-bracket-separated list field. Supports two forms:
///   `triggers: a, b, c`                  (bare CSV)
///   `triggers: ["a", "b", "c"]`          (YAML-flow array)
/// Returns an empty Vec when the value is empty / "[]" / unparseable.
fn parse_string_list(raw: &str) -> Vec<String> {
    let value = raw.trim();
    if value.is_empty() {
        return Vec::new();
    }
    let inner = if value.starts_with('[') && value.ends_with(']') {
        &value[1..value.len() - 1]
    } else {
        value
    };
    inner
        .split(',')
        .map(|segment| {
            let stripped = segment.trim();
            unquote(stripped)
        })
        .filter(|item| !item.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_skill(dir: &Path, name: &str, body: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).expect("create skill dir");
        fs::write(skill_dir.join("SKILL.md"), body).expect("write skill md");
    }

    #[test]
    fn parses_extended_frontmatter_csv_triggers() {
        let body = "---\nname: foo\nversion: \"1.2.3\"\ntriggers: implement, build, ship\nscope: execution\n---\n# Body";
        let fields = parse_skill_frontmatter_extended(body);
        assert_eq!(fields.name.as_deref(), Some("foo"));
        assert_eq!(fields.version.as_deref(), Some("1.2.3"));
        assert_eq!(fields.triggers, vec!["implement", "build", "ship"]);
        assert_eq!(fields.scope.as_deref(), Some("execution"));
    }

    #[test]
    fn parses_extended_frontmatter_yaml_flow_array() {
        let body = "---\nname: bar\ntriggers: [\"alpha\", \"beta\"]\n---\n";
        let fields = parse_skill_frontmatter_extended(body);
        assert_eq!(fields.triggers, vec!["alpha", "beta"]);
    }

    #[test]
    fn missing_extended_keys_default_empty() {
        let body = "---\nname: legacy\ndescription: old skill\n---\n";
        let fields = parse_skill_frontmatter_extended(body);
        assert!(fields.triggers.is_empty());
        assert!(fields.version.is_none());
        assert!(fields.scope.is_none());
        assert_eq!(fields.name.as_deref(), Some("legacy"));
    }

    #[test]
    fn discovery_walks_and_dedupes_across_roots() {
        let tmp = TempDir::new().expect("tempdir");
        let root_a = tmp.path().join("a");
        let root_b = tmp.path().join("b");
        fs::create_dir_all(&root_a).expect("create root_a");
        fs::create_dir_all(&root_b).expect("create root_b");

        write_skill(
            &root_a,
            "shared",
            "---\nname: shared\nversion: 1.0.0\ntriggers: x, y\n---\n# A",
        );
        write_skill(
            &root_b,
            "shared",
            "---\nname: shared\nversion: 2.0.0\n---\n# B",
        );
        write_skill(&root_b, "unique", "---\nname: unique\nversion: 0.1.0\n---\n");

        let roots = vec![
            DiscoveryRoot::skills_dir(root_a),
            DiscoveryRoot::skills_dir(root_b),
        ];
        let manifests = find_skills(&roots).expect("walk should succeed");
        assert_eq!(manifests.len(), 2);
        let names: Vec<_> = manifests.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"shared"));
        assert!(names.contains(&"unique"));
        // Precedence: first-root wins, so shared keeps version 1.0.0.
        let shared = manifests
            .iter()
            .find(|m| m.name == "shared")
            .expect("shared present");
        assert_eq!(shared.version, "1.0.0");
    }

    #[test]
    fn discovery_handles_missing_root_gracefully() {
        let tmp = TempDir::new().expect("tempdir");
        let real = tmp.path().join("real");
        std::fs::create_dir_all(&real).expect("create real");
        write_skill(&real, "alpha", "---\nname: alpha\n---\n");
        let roots = vec![
            DiscoveryRoot::skills_dir(real),
            DiscoveryRoot::skills_dir(tmp.path().join("does_not_exist")),
        ];
        let manifests = find_skills(&roots).expect("partial success");
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].name, "alpha");
    }
}
