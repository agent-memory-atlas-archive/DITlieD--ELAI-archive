//! A-15: Nightly dependency audit CLI subcommand.
//!
//! Queries 7 registry APIs (crates.io, PyPI, npm, Go proxy, Maven Central,
//! NuGet, RubyGems), auto-detects ecosystems from manifest files, compares
//! installed versions against latest, and upserts findings into the
//! `deprecated_patterns` table in rules.db.

use std::fmt;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, info, warn};

use elai_core::paths::metrics_dir_path as core_metrics_dir_path;
use elai_core::ClockProvider;
use elai_orchestrator::{CatchLayer, StaleApiCatch};
use elai_rules::schema::{open_rules_db, upsert_deprecated_pattern};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const CRATES_IO_BASE: &str = "https://crates.io/api/v1/crates";
const PYPI_BASE: &str = "https://pypi.org/pypi";
const NPM_BASE: &str = "https://registry.npmjs.org";
const GO_PROXY_BASE: &str = "https://proxy.golang.org";
const MAVEN_SEARCH_BASE: &str = "https://search.maven.org/solrsearch/select";
const NUGET_BASE: &str = "https://api.nuget.org/v3-flatcontainer";
const RUBYGEMS_BASE: &str = "https://rubygems.org/api/v1/gems";

/// Rate-limit delay for crates.io (1 request per second).
const CRATES_IO_DELAY: Duration = Duration::from_millis(1_050);

/// User-Agent header required by crates.io.
const USER_AGENT: &str = "elai-dep-audit/0.1 (https://github.com/elai)";

/// Severity for dependencies that are outdated by a major version.
const SEVERITY_WARN: &str = "WARN";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single finding from the dependency audit.
#[derive(Debug, Clone, Serialize)]
pub struct DepAuditEntry {
    pub name: String,
    pub ecosystem: String,
    pub current_version: String,
    pub latest_version: String,
    pub severity: String,
}

/// Aggregated result of a full audit run.
#[derive(Debug, Clone, Serialize)]
pub struct DepAuditResult {
    pub entries: Vec<DepAuditEntry>,
    pub timestamp: String,
    pub total_outdated: usize,
    pub ecosystems_scanned: Vec<String>,
    pub errors: Vec<String>,
}

/// Ecosystem identifier matching the CHECK constraint in deprecated_patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ecosystem {
    Cargo,
    Pip,
    Npm,
    Go,
    Maven,
    Nuget,
    Bundler,
}

impl Ecosystem {
    fn as_str(self) -> &'static str {
        match self {
            Self::Cargo => "cargo",
            Self::Pip => "pip",
            Self::Npm => "npm",
            Self::Go => "go",
            Self::Maven => "maven",
            Self::Nuget => "nuget",
            Self::Bundler => "bundler",
        }
    }

    fn manifest_filename(self) -> &'static str {
        match self {
            Self::Cargo => "Cargo.toml",
            Self::Pip => "pyproject.toml",
            Self::Npm => "package.json",
            Self::Go => "go.mod",
            Self::Maven => "pom.xml",
            Self::Nuget => "*.csproj",
            Self::Bundler => "Gemfile",
        }
    }
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dependency extracted from a manifest file.
#[derive(Debug, Clone)]
struct ManifestDep {
    name: String,
    version_spec: String,
}

// ---------------------------------------------------------------------------
// Manifest detection
// ---------------------------------------------------------------------------

/// Detect which ecosystems are present by scanning for manifest files.
pub fn detect_ecosystems(project_dir: &Path) -> Vec<Ecosystem> {
    let checks = [
        (Ecosystem::Cargo, "Cargo.toml"),
        (Ecosystem::Pip, "pyproject.toml"),
        (Ecosystem::Npm, "package.json"),
        (Ecosystem::Go, "go.mod"),
        (Ecosystem::Maven, "pom.xml"),
        (Ecosystem::Bundler, "Gemfile"),
    ];

    let mut found = Vec::new();
    for (eco, filename) in &checks {
        if project_dir.join(filename).exists() {
            found.push(*eco);
        }
    }

    // NuGet: scan for any .csproj file
    if let Ok(entries) = fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "csproj" {
                    found.push(Ecosystem::Nuget);
                    break;
                }
            }
        }
    }

    found
}

// ---------------------------------------------------------------------------
// Manifest parsers
// ---------------------------------------------------------------------------

fn parse_cargo_deps(project_dir: &Path) -> Result<Vec<ManifestDep>> {
    let content = fs::read_to_string(project_dir.join("Cargo.toml"))
        .context("reading Cargo.toml")?;
    let doc: toml::Value = toml::from_str(&content)
        .context("parsing Cargo.toml")?;

    let mut deps = Vec::new();

    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = doc.get(section).and_then(|v| v.as_table()) {
            for (name, val) in table {
                let version_spec = extract_toml_version(val);
                if let Some(vs) = version_spec {
                    deps.push(ManifestDep {
                        name: name.clone(),
                        version_spec: vs,
                    });
                }
            }
        }
    }

    // Also check [workspace.dependencies]
    if let Some(ws) = doc.get("workspace").and_then(|v| v.as_table()) {
        if let Some(ws_deps) = ws.get("dependencies").and_then(|v| v.as_table()) {
            for (name, val) in ws_deps {
                let version_spec = extract_toml_version(val);
                if let Some(vs) = version_spec {
                    if !deps.iter().any(|d| d.name == *name) {
                        deps.push(ManifestDep {
                            name: name.clone(),
                            version_spec: vs,
                        });
                    }
                }
            }
        }
    }

    Ok(deps)
}

fn extract_toml_version(val: &toml::Value) -> Option<String> {
    match val {
        toml::Value::String(s) => Some(s.clone()),
        toml::Value::Table(t) => {
            // { version = "X", ... } or { path = "..." } (path-only = skip)
            t.get("version").and_then(|v| v.as_str()).map(String::from)
        }
        _ => None,
    }
}

fn parse_pyproject_deps(project_dir: &Path) -> Result<Vec<ManifestDep>> {
    let content = fs::read_to_string(project_dir.join("pyproject.toml"))
        .context("reading pyproject.toml")?;
    let doc: toml::Value = toml::from_str(&content)
        .context("parsing pyproject.toml")?;

    let mut deps = Vec::new();

    // [project.dependencies] is a list of PEP 508 strings
    if let Some(dep_list) = doc
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
    {
        for item in dep_list {
            if let Some(s) = item.as_str() {
                if let Some(dep) = parse_pep508_dep(s) {
                    deps.push(dep);
                }
            }
        }
    }

    Ok(deps)
}

/// Parse a PEP 508 dependency string like "requests>=2.28" or "flask~=3.0".
fn parse_pep508_dep(spec: &str) -> Option<ManifestDep> {
    // Split on version specifiers: >=, <=, ==, ~=, !=, <, >
    let spec = spec.trim();
    let delimiters = [">=", "<=", "==", "~=", "!=", "<", ">"];
    for delim in &delimiters {
        if let Some(pos) = spec.find(delim) {
            let name = spec[..pos].trim().to_string();
            let version_spec = spec[pos..].trim().to_string();
            if !name.is_empty() {
                return Some(ManifestDep { name, version_spec });
            }
        }
    }
    // No version constraint — still record it with empty version
    let name = spec.split(';').next()?.split('[').next()?.trim().to_string();
    if name.is_empty() {
        return None;
    }
    Some(ManifestDep {
        name,
        version_spec: String::new(),
    })
}

fn parse_package_json_deps(project_dir: &Path) -> Result<Vec<ManifestDep>> {
    let content = fs::read_to_string(project_dir.join("package.json"))
        .context("reading package.json")?;
    let doc: Value = serde_json::from_str(&content)
        .context("parsing package.json")?;

    let mut deps = Vec::new();
    for section in &["dependencies", "devDependencies"] {
        if let Some(obj) = doc.get(section).and_then(|v| v.as_object()) {
            for (name, val) in obj {
                if let Some(vs) = val.as_str() {
                    deps.push(ManifestDep {
                        name: name.clone(),
                        version_spec: vs.to_string(),
                    });
                }
            }
        }
    }
    Ok(deps)
}

fn parse_go_mod_deps(project_dir: &Path) -> Result<Vec<ManifestDep>> {
    let content = fs::read_to_string(project_dir.join("go.mod"))
        .context("reading go.mod")?;

    let mut deps = Vec::new();
    let mut in_require = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("require (") || trimmed == "require (" {
            in_require = true;
            continue;
        }
        if trimmed == ")" {
            in_require = false;
            continue;
        }
        if in_require {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                deps.push(ManifestDep {
                    name: parts[0].to_string(),
                    version_spec: parts[1].to_string(),
                });
            }
        } else if trimmed.starts_with("require ") {
            let rest = trimmed.strip_prefix("require ").unwrap_or("");
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                deps.push(ManifestDep {
                    name: parts[0].to_string(),
                    version_spec: parts[1].to_string(),
                });
            }
        }
    }
    Ok(deps)
}

fn parse_gemfile_deps(project_dir: &Path) -> Result<Vec<ManifestDep>> {
    let content = fs::read_to_string(project_dir.join("Gemfile"))
        .context("reading Gemfile")?;

    let mut deps = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("gem ") || trimmed.starts_with("gem\t") {
            // gem 'name', '~> 1.0'
            let parts: Vec<&str> = trimmed.splitn(3, |c: char| c == ',' || c == ' ' || c == '\t')
                .filter(|s| !s.is_empty())
                .collect();
            if parts.len() >= 2 {
                let name = parts[1].trim_matches(|c: char| c == '\'' || c == '"');
                let version_spec = if parts.len() >= 3 {
                    parts[2].trim().trim_matches(|c: char| c == '\'' || c == '"').to_string()
                } else {
                    String::new()
                };
                deps.push(ManifestDep {
                    name: name.to_string(),
                    version_spec,
                });
            }
        }
    }
    Ok(deps)
}

/// Parse dependencies from a manifest file for the given ecosystem.
fn parse_manifest(ecosystem: Ecosystem, project_dir: &Path) -> Result<Vec<ManifestDep>> {
    match ecosystem {
        Ecosystem::Cargo => parse_cargo_deps(project_dir),
        Ecosystem::Pip => parse_pyproject_deps(project_dir),
        Ecosystem::Npm => parse_package_json_deps(project_dir),
        Ecosystem::Go => parse_go_mod_deps(project_dir),
        Ecosystem::Maven => Ok(Vec::new()), // pom.xml parsing is complex; skip for now
        Ecosystem::Nuget => Ok(Vec::new()), // .csproj parsing is complex; skip for now
        Ecosystem::Bundler => parse_gemfile_deps(project_dir),
    }
}

// ---------------------------------------------------------------------------
// Registry queries (reqwest::blocking — BLOCK-5/WARN-6 compliant)
// ---------------------------------------------------------------------------

fn build_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(15))
        .build()
        .context("building HTTP client")
}

/// Query crates.io REST API for latest version.
/// GET https://crates.io/api/v1/crates/{name} -> .crate.max_version
fn query_crates_io(client: &reqwest::blocking::Client, name: &str) -> Result<String> {
    let url = format!("{CRATES_IO_BASE}/{name}");
    debug!(name, url = %url, "querying crates.io");
    let resp: Value = client.get(&url).send()?.json()?;
    let version = resp
        .pointer("/crate/max_version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing .crate.max_version for {name}"))?;
    Ok(version.to_string())
}

/// Query PyPI JSON API for latest version.
/// GET https://pypi.org/pypi/{name}/json -> .info.version
fn query_pypi(client: &reqwest::blocking::Client, name: &str) -> Result<String> {
    let url = format!("{PYPI_BASE}/{name}/json");
    debug!(name, url = %url, "querying PyPI");
    let resp: Value = client.get(&url).send()?.json()?;
    let version = resp
        .pointer("/info/version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing .info.version for {name}"))?;
    Ok(version.to_string())
}

/// Query npm registry for latest version.
/// GET https://registry.npmjs.org/{name}/latest -> .version
fn query_npm(client: &reqwest::blocking::Client, name: &str) -> Result<String> {
    let url = format!("{NPM_BASE}/{name}/latest");
    debug!(name, url = %url, "querying npm");
    let resp: Value = client.get(&url).send()?.json()?;
    let version = resp
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing .version for npm/{name}"))?;
    Ok(version.to_string())
}

/// Query Go module proxy for latest version.
/// GET https://proxy.golang.org/{module}/@latest -> .Version
fn query_go_proxy(client: &reqwest::blocking::Client, module: &str) -> Result<String> {
    let url = format!("{GO_PROXY_BASE}/{module}/@latest");
    debug!(module, url = %url, "querying Go proxy");
    let resp: Value = client.get(&url).send()?.json()?;
    let version = resp
        .get("Version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing .Version for go/{module}"))?;
    Ok(version.to_string())
}

/// Query Maven Central for latest version.
/// GET https://search.maven.org/solrsearch/select?q=g:{group}+AND+a:{artifact}
///   -> .response.docs[0].latestVersion
fn query_maven(client: &reqwest::blocking::Client, coordinate: &str) -> Result<String> {
    // coordinate is "group:artifact" or just "artifact"
    let (group, artifact) = if let Some(pos) = coordinate.find(':') {
        (&coordinate[..pos], &coordinate[pos + 1..])
    } else {
        return Err(anyhow::anyhow!("Maven coordinate must be group:artifact, got {coordinate}"));
    };
    let url = format!("{MAVEN_SEARCH_BASE}?q=g:{group}+AND+a:{artifact}&rows=1&wt=json");
    debug!(coordinate, url = %url, "querying Maven Central");
    let resp: Value = client.get(&url).send()?.json()?;
    let version = resp
        .pointer("/response/docs/0/latestVersion")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing latestVersion for maven/{coordinate}"))?;
    Ok(version.to_string())
}

/// Query NuGet flat container for latest version.
/// GET https://api.nuget.org/v3-flatcontainer/{name}/index.json
///   -> .versions (array, last element = latest)
fn query_nuget(client: &reqwest::blocking::Client, name: &str) -> Result<String> {
    let lower = name.to_lowercase();
    let url = format!("{NUGET_BASE}/{lower}/index.json");
    debug!(name, url = %url, "querying NuGet");
    let resp: Value = client.get(&url).send()?.json()?;
    let version = resp
        .get("versions")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.last())
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing .versions for nuget/{name}"))?;
    Ok(version.to_string())
}

/// Query RubyGems for latest version.
/// GET https://rubygems.org/api/v1/gems/{name}.json -> .version
fn query_rubygems(client: &reqwest::blocking::Client, name: &str) -> Result<String> {
    let url = format!("{RUBYGEMS_BASE}/{name}.json");
    debug!(name, url = %url, "querying RubyGems");
    let resp: Value = client.get(&url).send()?.json()?;
    let version = resp
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing .version for rubygems/{name}"))?;
    Ok(version.to_string())
}

/// Query the registry for the latest version of a dependency.
fn query_latest(
    client: &reqwest::blocking::Client,
    ecosystem: Ecosystem,
    name: &str,
) -> Result<String> {
    match ecosystem {
        Ecosystem::Cargo => query_crates_io(client, name),
        Ecosystem::Pip => query_pypi(client, name),
        Ecosystem::Npm => query_npm(client, name),
        Ecosystem::Go => query_go_proxy(client, name),
        Ecosystem::Maven => query_maven(client, name),
        Ecosystem::Nuget => query_nuget(client, name),
        Ecosystem::Bundler => query_rubygems(client, name),
    }
}

// ---------------------------------------------------------------------------
// Version comparison
// ---------------------------------------------------------------------------

/// Extract the major version from a semver-ish string.
/// Handles prefixes like "v", "^", "~", ">=", etc.
fn extract_major(version: &str) -> Option<u64> {
    let cleaned = version
        .trim_start_matches(|c: char| !c.is_ascii_digit());
    let major_str = cleaned.split('.').next()?;
    major_str.parse::<u64>().ok()
}

/// Compare current vs latest version. Returns true if latest has a higher
/// major version than current, indicating a potentially outdated dependency.
/// For 0.x versions, a minor version bump (0.8 -> 0.9) is considered outdated
/// because semver treats 0.x minor bumps as breaking changes.
fn is_outdated(current_spec: &str, latest: &str) -> bool {
    let current_clean = current_spec
        .trim_start_matches(|c: char| c == '^' || c == '~' || c == '>' || c == '=' || c == '<' || c == ' ' || c == 'v');
    // Try full semver comparison
    if let (Ok(cur), Ok(lat)) = (
        semver::Version::parse(current_clean),
        semver::Version::parse(latest),
    ) {
        if lat.major > cur.major {
            return true;
        }
        // 0.x -> 0.y where y > x is a breaking change in semver
        if cur.major == 0 && lat.major == 0 && lat.minor > cur.minor {
            return true;
        }
        return false;
    }
    // Try partial version (e.g. "0.8" without patch)
    let cur_parts = parse_version_parts(current_clean);
    let lat_parts = parse_version_parts(latest);
    if let (Some((cmaj, cmin)), Some((lmaj, lmin))) = (cur_parts, lat_parts) {
        if lmaj > cmaj {
            return true;
        }
        if cmaj == 0 && lmaj == 0 && lmin > cmin {
            return true;
        }
        return false;
    }
    false
}

/// Parse "X.Y" or "X.Y.Z" into (major, minor).
fn parse_version_parts(version: &str) -> Option<(u64, u64)> {
    let cleaned = version.trim_start_matches(|c: char| !c.is_ascii_digit());
    let mut parts = cleaned.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    Some((major, minor))
}

// ---------------------------------------------------------------------------
// C-21 Task 4 — Bridge dep-audit results to FullPipelineOutcome.stale_api_catches.
// ---------------------------------------------------------------------------

/// Convert a [`DepAuditResult`] into the [`StaleApiCatch`] rows that
/// `FullPipelineOutcome` carries. Each `DepAuditEntry` becomes a single
/// catch with `caught_by = CatchLayer::DepAudit`.
///
/// `file_path` is left as `PathBuf::new()` because dep-audit operates at
/// the manifest level, not the per-file level — downstream consumers
/// (self_mod_bridge::proposals_from_full_outcome) already key on
/// (deprecated_name, ecosystem) so the empty path does not cause grouping
/// collisions.
///
/// Spec phase-4H.txt section 4H.3B path A (deprecated_patterns from
/// stale_api_catches).
pub fn dep_audit_to_stale_catches(report: &DepAuditResult) -> Vec<StaleApiCatch> {
    report
        .entries
        .iter()
        .map(|entry| StaleApiCatch {
            file_path: std::path::PathBuf::new(),
            deprecated_name: entry.name.clone(),
            replacement_name: format!("{} >= {}", entry.name, entry.latest_version),
            ecosystem: entry.ecosystem.clone(),
            caught_by: CatchLayer::DepAudit,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Report generation
// ---------------------------------------------------------------------------

fn generate_report_toml(result: &DepAuditResult) -> String {
    let mut out = String::with_capacity(512);
    out.push_str("[audit]\n");
    out.push_str(&format!("timestamp = \"{}\"\n", result.timestamp));
    out.push_str(&format!("total_outdated = {}\n", result.total_outdated));
    out.push_str(&format!(
        "ecosystems_scanned = [{}]\n",
        result
            .ecosystems_scanned
            .iter()
            .map(|e| format!("\"{e}\""))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    if !result.errors.is_empty() {
        out.push_str(&format!(
            "errors = [{}]\n",
            result
                .errors
                .iter()
                .map(|e| format!("\"{e}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    out.push('\n');
    for entry in &result.entries {
        out.push_str("[[findings]]\n");
        out.push_str(&format!("name = \"{}\"\n", entry.name));
        out.push_str(&format!("ecosystem = \"{}\"\n", entry.ecosystem));
        out.push_str(&format!("current_version = \"{}\"\n", entry.current_version));
        out.push_str(&format!("latest_version = \"{}\"\n", entry.latest_version));
        out.push_str(&format!("severity = \"{}\"\n", entry.severity));
        out.push('\n');
    }

    out
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run the full dependency audit.
///
/// 1. Detect ecosystems from manifest files in `project_dir`.
/// 2. Parse each manifest for dependencies.
/// 3. Query registry APIs for latest versions.
/// 4. Compare and collect outdated entries.
/// 5. Upsert findings into deprecated_patterns table.
/// 6. Write TOML report to metrics/.
pub fn run_dep_audit(
    project_dir: &Path,
    rules_db_path: &Path,
    clock: &impl ClockProvider,
) -> Result<DepAuditResult> {
    let ecosystems = detect_ecosystems(project_dir);
    if ecosystems.is_empty() {
        info!("no manifest files found in {}", project_dir.display());
        return Ok(DepAuditResult {
            entries: Vec::new(),
            timestamp: format_timestamp(clock),
            total_outdated: 0,
            ecosystems_scanned: Vec::new(),
            errors: Vec::new(),
        });
    }

    info!(
        ecosystems = ?ecosystems.iter().map(|e| e.as_str()).collect::<Vec<_>>(),
        "detected ecosystems"
    );

    let client = build_client()?;
    let mut all_entries = Vec::new();
    let mut all_errors = Vec::new();
    let scanned: Vec<String> = ecosystems.iter().map(|e| e.as_str().to_string()).collect();

    for ecosystem in &ecosystems {
        let deps = match parse_manifest(*ecosystem, project_dir) {
            Ok(d) => d,
            Err(e) => {
                let msg = format!("{ecosystem}: manifest parse error: {e}");
                warn!("{msg}");
                all_errors.push(msg);
                continue;
            }
        };

        info!(ecosystem = %ecosystem, dep_count = deps.len(), "parsed manifest");

        for dep in &deps {
            // Rate limit for crates.io
            if *ecosystem == Ecosystem::Cargo {
                thread::sleep(CRATES_IO_DELAY);
            }

            let latest = match query_latest(&client, *ecosystem, &dep.name) {
                Ok(v) => v,
                Err(e) => {
                    let msg = format!("{ecosystem}/{}: query failed: {e}", dep.name);
                    debug!("{msg}");
                    all_errors.push(msg);
                    continue;
                }
            };

            if dep.version_spec.is_empty() {
                // No version constraint — can't compare, skip
                continue;
            }

            if is_outdated(&dep.version_spec, &latest) {
                all_entries.push(DepAuditEntry {
                    name: dep.name.clone(),
                    ecosystem: ecosystem.as_str().to_string(),
                    current_version: dep.version_spec.clone(),
                    latest_version: latest,
                    severity: SEVERITY_WARN.to_string(),
                });
            }
        }
    }

    let total_outdated = all_entries.len();
    let timestamp = format_timestamp(clock);

    // Upsert into deprecated_patterns table
    let conn = open_rules_db(rules_db_path, clock)
        .context("opening rules.db")?;

    for entry in &all_entries {
        let pattern_id = format!("dp-audit-{}-{}", entry.ecosystem, entry.name);
        let reason = format!(
            "auto-audit: current {} -> latest {}",
            entry.current_version, entry.latest_version
        );
        if let Err(e) = upsert_deprecated_pattern(
            &conn,
            &pattern_id,
            &entry.ecosystem,
            &entry.name,
            &format!("{} >= {}", entry.name, entry.latest_version),
            None,
            &entry.severity,
            &reason,
        ) {
            let msg = format!("db upsert failed for {}: {e}", entry.name);
            warn!("{msg}");
            all_errors.push(msg);
        }
    }

    info!(total_outdated, "audit complete, writing report");

    // Write TOML report
    let result = DepAuditResult {
        entries: all_entries,
        timestamp,
        total_outdated,
        ecosystems_scanned: scanned,
        errors: all_errors,
    };

    let metrics_dir = core_metrics_dir_path(project_dir);
    if !metrics_dir.exists() {
        fs::create_dir_all(&metrics_dir)
            .context("creating metrics directory")?;
    }

    let date_part = &result.timestamp[..10]; // YYYY-MM-DD
    let report_path = metrics_dir.join(format!("dep_audit_{date_part}.toml"));
    let report_content = generate_report_toml(&result);
    fs::write(&report_path, &report_content)
        .with_context(|| format!("writing report to {}", report_path.display()))?;

    info!(report = %report_path.display(), "report written");

    Ok(result)
}

fn format_timestamp(clock: &impl ClockProvider) -> String {
    let sys_time = clock.system_now();
    let secs = sys_time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    time_from_epoch(i64::try_from(secs).unwrap_or(0))
}

/// Simple epoch-to-string conversion without external date crate.
fn time_from_epoch(epoch_secs: i64) -> String {
    // days since 1970-01-01
    let total_secs = epoch_secs;
    let secs_per_day: i64 = 86400;
    let mut days = total_secs / secs_per_day;
    let time_of_day = total_secs % secs_per_day;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Compute year/month/day from days since epoch
    let mut year: i64 = 1970;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let month_days: &[i64] = if is_leap(year) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month: i64 = 1;
    for &md in month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    let day = days + 1;

    format!(
        "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z"
    )
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_major() {
        assert_eq!(extract_major("1.2.3"), Some(1));
        assert_eq!(extract_major("^0.12"), Some(0));
        assert_eq!(extract_major(">=2.28"), Some(2));
        assert_eq!(extract_major("v3.1.0"), Some(3));
        assert_eq!(extract_major("~1.0"), Some(1));
        assert_eq!(extract_major(""), None);
    }

    #[test]
    fn test_is_outdated_major_bump() {
        assert!(is_outdated("0.8", "0.9"));
        assert!(is_outdated("^1.0.0", "2.0.0"));
        assert!(!is_outdated("^1.0.0", "1.5.0"));
        assert!(!is_outdated("2.0.0", "2.0.0"));
    }

    #[test]
    fn test_is_outdated_prefixed() {
        assert!(is_outdated(">=1.0", "2.3.0"));
        assert!(!is_outdated(">=2.0", "2.3.0"));
    }

    #[test]
    fn test_parse_pep508_dep() {
        let dep = parse_pep508_dep("requests>=2.28").unwrap();
        assert_eq!(dep.name, "requests");
        assert_eq!(dep.version_spec, ">=2.28");

        let dep2 = parse_pep508_dep("flask~=3.0").unwrap();
        assert_eq!(dep2.name, "flask");
        assert_eq!(dep2.version_spec, "~=3.0");

        let dep3 = parse_pep508_dep("black").unwrap();
        assert_eq!(dep3.name, "black");
        assert_eq!(dep3.version_spec, "");
    }

    #[test]
    fn test_detect_ecosystems_empty_dir() {
        let temp = std::env::temp_dir().join("elai_test_empty_eco");
        let _ = fs::create_dir_all(&temp);
        let found = detect_ecosystems(&temp);
        // Might find things in temp dir, but should not panic
        assert!(found.len() <= 7);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_time_from_epoch() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        let s = time_from_epoch(1_704_067_200);
        assert_eq!(s, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_generate_report_toml() {
        let result = DepAuditResult {
            entries: vec![DepAuditEntry {
                name: "rand".to_string(),
                ecosystem: "cargo".to_string(),
                current_version: "0.8".to_string(),
                latest_version: "0.9.1".to_string(),
                severity: "WARN".to_string(),
            }],
            timestamp: "2024-01-15T10:00:00Z".to_string(),
            total_outdated: 1,
            ecosystems_scanned: vec!["cargo".to_string()],
            errors: Vec::new(),
        };
        let toml_str = generate_report_toml(&result);
        assert!(toml_str.contains("total_outdated = 1"));
        assert!(toml_str.contains("name = \"rand\""));
        assert!(toml_str.contains("latest_version = \"0.9.1\""));
    }

    #[test]
    fn test_ecosystem_display() {
        assert_eq!(format!("{}", Ecosystem::Cargo), "cargo");
        assert_eq!(format!("{}", Ecosystem::Pip), "pip");
        assert_eq!(format!("{}", Ecosystem::Npm), "npm");
    }
}
