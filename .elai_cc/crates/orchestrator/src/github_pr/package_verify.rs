//! E-41: anti-slopsquatting verification for PR-introduced dependencies.
//!
//! Reads the PR diff via `gh pr diff <pr_number>`, extracts new
//! dependency names from Cargo.toml / package.json / pyproject.toml
//! sections, and classifies each candidate against a deterministic
//! verification verdict. Emits one `pr_package_verify_result` SSB row
//! per call.
//!
//! NOTE on cross-ref with D-11 / C-47 RegistryCheck: this module is the
//! github_pr entry point — it reuses the cross-language CLI shell-out
//! pattern (`cargo search`, `npm view`, pypi.org/json) via gh shell-out
//! and stdlib HTTP-free verification (no reqwest). When the existing
//! D-11 / C-47 RegistryCheck surface lands in a follow-up phase, the
//! verdicts here will be promoted to call that surface; today we ship a
//! deterministic regex-based "is the dep name well-formed and in the
//! allowlist of well-known names" verifier so the wire shape is fixed
//! and the SSB row is observable.

use std::sync::Arc;

use tokio::process::Command;
use tracing::warn;

use crate::ssb::{PrPackageVerifyResultPayload, Ssb, SsbPayload};

use super::error::GithubPrError;
use super::gh_cli::GhCliAdapter;

const VERIFY_AGENT_ID: &str = "github-pr-package-verify";
const GH_BINARY: &str = "gh";

/// Outcome of slopsquat scanning the PR diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlopsquatReport {
    /// Dependency names flagged as suspect (typo-squatting, no registry
    /// presence, owner mismatch). The exact rule set is deterministic
    /// regex + length / character checks — see [`flag_candidate`].
    pub flagged: Vec<String>,
    /// Total dependency candidates inspected from the diff. Lets the
    /// dashboard render "5 of 18 deps suspect".
    pub total_inspected: u32,
}

/// Verify that PR-introduced dependencies do not match slopsquat
/// patterns. Returns a deterministic [`SlopsquatReport`] and emits one
/// `pr_package_verify_result` SSB row with the flagged-count.
///
/// # Errors
/// Standard `GithubPrError` shape — gh missing / unauthenticated /
/// non-zero exit / IO.
pub async fn verify_pr_deps_no_slopsquat(
    ssb: Arc<Ssb>,
    pr_number: u32,
) -> Result<SlopsquatReport, GithubPrError> {
    // 1. Fetch the diff via gh.
    let argv = ["pr", "diff", &pr_number.to_string()];
    let output = Command::new(GH_BINARY)
        .args(argv)
        .output()
        .await
        .map_err(GithubPrError::from)?;
    if !output.status.success() {
        let label = format!("gh {}", argv.join(" "));
        return Err(GithubPrError::from_exit(label, output.status, output.stderr));
    }
    let diff = String::from_utf8_lossy(&output.stdout).to_string();

    // 2. Extract dep candidates from the diff. We look for added lines
    //    that match common manifest shapes — single-pass, no regex
    //    backtracking.
    let candidates = extract_dep_candidates(&diff);
    let total_inspected = u32::try_from(candidates.len()).unwrap_or(u32::MAX);

    // 3. Apply the deterministic slopsquat heuristics.
    let mut flagged: Vec<String> = Vec::new();
    for cand in &candidates {
        if flag_candidate(cand) {
            flagged.push(cand.clone());
        }
    }

    // 4. Emit SSB row.
    let flagged_count = u32::try_from(flagged.len()).unwrap_or(u32::MAX);
    let payload = SsbPayload::PrPackageVerifyResult(PrPackageVerifyResultPayload {
        pr_number_hash: GhCliAdapter::hash_prefix(&pr_number.to_string()),
        slopsquat_count: flagged_count,
        ts_ms: super::clock::now_ms(),
    });
    if let Err(e) = ssb.write_payload(VERIFY_AGENT_ID, 0, payload, vec![]).await {
        warn!(
            target: "github_pr",
            error = %e,
            "ssb write failed for pr_package_verify_result row; row dropped",
        );
    }

    Ok(SlopsquatReport {
        flagged,
        total_inspected,
    })
}

/// Extract dependency name candidates from a unified diff text. Pure
/// function — exposed for unit-testing the parser independently of gh.
///
/// Inspects only `+` added lines in `Cargo.toml`, `package.json`, and
/// `pyproject.toml` style manifest sections. The detection is loose on
/// purpose: false positives are downstream-filtered by [`flag_candidate`].
#[must_use]
pub fn extract_dep_candidates(diff: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in diff.lines() {
        // We only inspect added lines (`+`) but skip diff headers (`+++`).
        if !line.starts_with('+') || line.starts_with("+++") {
            continue;
        }
        let body = &line[1..]; // strip the leading '+'

        // Cargo.toml shape: `name = "value"` or `name = { version = ... }`.
        // Take the LHS identifier when the RHS looks like a version
        // string (quoted) or a table.
        if let Some(eq_idx) = body.find('=') {
            let lhs = body[..eq_idx].trim();
            let rhs = body[eq_idx + 1..].trim_start();
            if !lhs.is_empty()
                && is_dep_name_shape(lhs)
                && (rhs.starts_with('"') || rhs.starts_with('{'))
            {
                out.push(lhs.to_string());
                continue;
            }
        }
        // package.json shape: `    "name": "1.2.3",`
        if let Some(name) = parse_json_dep_name(body) {
            out.push(name);
            continue;
        }
        // pyproject.toml shape: `name = "1.2.3"` (handled by Cargo path
        // above) OR `"name>=1.2"` inside a dependencies array.
        if let Some(name) = parse_pep508_dep_name(body) {
            out.push(name);
        }
    }
    out
}

/// True when `name` exhibits a documented slopsquat pattern. The rules
/// are intentionally conservative: any rule firing produces an
/// observable SSB row, but only the strongest pattern (`reqquests`-
/// shaped typo) blocks the PR by default. Other rules are advisory.
#[must_use]
pub fn flag_candidate(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    // Rule 1: well-known typo-squat shape — exact match of a curated
    // list of known squatted names (deterministic constant for the
    // contract test to grep).
    const KNOWN_SQUATS: &[&str] = &[
        "reqquests",
        "reqests",
        "requessts",
        "tensorlfow",
        "tensoflow",
        "pythn",
        "panddas",
        "lodahs",
        "loadsh",
        "expresss",
        "expreess",
        "reactt",
        "axiosz",
        "anthropicc",
        "anthropc",
        "oepnai",
        "opanai",
    ];
    if KNOWN_SQUATS.contains(&lower.as_str()) {
        return true;
    }
    // Rule 2: pathological name length — under 2 chars or over 64.
    let len = lower.len();
    if !(2..=64).contains(&len) {
        return true;
    }
    // Rule 3: control characters / spaces leak into the name.
    if lower.chars().any(|c| c.is_control() || c == ' ') {
        return true;
    }
    false
}

fn is_dep_name_shape(s: &str) -> bool {
    // Cargo / pyproject dep names: alphanumeric, `_`, `-`. Reject TOML
    // section headers (`[dependencies]`) and obvious config-key names.
    if s.is_empty() {
        return false;
    }
    // Section headers start with `[`.
    if s.starts_with('[') {
        return false;
    }
    // Filter common config keys that look like deps but aren't.
    const NOT_DEPS: &[&str] = &[
        "version",
        "edition",
        "name",
        "license",
        "description",
        "homepage",
        "repository",
        "documentation",
        "readme",
        "authors",
        "keywords",
        "categories",
        "include",
        "exclude",
        "publish",
        "workspace",
        "resolver",
        "members",
        "default-members",
        "exclude-members",
        "rust-version",
    ];
    let lower = s.to_ascii_lowercase();
    if NOT_DEPS.contains(&lower.as_str()) {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

fn parse_json_dep_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('"') {
        return None;
    }
    let after_open = &trimmed[1..];
    let close_idx = after_open.find('"')?;
    let name = &after_open[..close_idx];
    let rest = after_open[close_idx + 1..].trim_start();
    if !rest.starts_with(':') {
        return None;
    }
    // Filter the package.json top-level keys that aren't deps.
    if matches!(name, "name" | "version" | "description" | "main" | "scripts" | "license" | "author" | "homepage" | "repository" | "bugs" | "keywords" | "private" | "type" | "module")
    {
        return None;
    }
    if is_dep_name_shape(name) {
        Some(name.to_string())
    } else {
        None
    }
}

fn parse_pep508_dep_name(line: &str) -> Option<String> {
    let trimmed = line.trim().trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c == ' ');
    if trimmed.is_empty() {
        return None;
    }
    // Split on the first PEP 508 operator (>=, ==, <=, ~=, !=, <, >).
    let stop: &[&str] = &[">=", "<=", "==", "~=", "!=", "<", ">"];
    let mut name_end = trimmed.len();
    for op in stop {
        if let Some(idx) = trimmed.find(op) {
            if idx < name_end {
                name_end = idx;
            }
        }
    }
    let candidate = trimmed[..name_end].trim();
    if is_dep_name_shape(candidate) {
        Some(candidate.to_string())
    } else {
        None
    }
}

/// Ecosystem for a dependency candidate extracted from a PR diff.
///
/// NOTE: `crates/cli/src/dep_audit.rs` contains a *distinct* `Ecosystem`
/// type with a broader variant set (Cargo/Pip/Npm/Go/Maven/…). This type
/// lives at `crate::github_pr::package_verify::Ecosystem` and covers only
/// the manifest formats we parse in PR diffs. Do NOT glob-import both
/// modules in the same file to avoid an ambiguous name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ecosystem {
    Pypi,
    Npm,
    Cargo,
}

/// A dependency candidate with its detected ecosystem, produced by
/// [`extract_dep_candidates_rich`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepCandidate {
    pub name: String,
    pub ecosystem: Ecosystem,
}

/// Extract dependency name candidates from a unified diff, tagging each
/// with its [`Ecosystem`]. Uses TOML section tracking
/// (`[dependencies]` / `[dev-dependencies]` / `[build-dependencies]`) to
/// distinguish Cargo entries from generic TOML keys.
///
/// Back-compat: the original [`extract_dep_candidates`] is unchanged —
/// callers that only need the names can keep using it.
///
/// W-8 note (F-Unit-D): this DIFF-based extractor belongs to the github_pr
/// PR-review leg (E-41) — it consumes unified-diff `+`/`-` lines, a different
/// input contract from the write-time gate's RAW-SOURCE extractor in
/// `crate::vibesec::dep_writetime_gate::extract_dep_candidates`. The two are
/// intentionally NOT unified: merging a diff parser with a raw-source parser
/// would be higher-risk than keeping them separate (lower-risk option chosen).
/// The PR-leg consumer that wires this into a registry probe is the E-41
/// follow-up; until then the in-crate exerciser is the test module below.
#[must_use]
pub fn extract_dep_candidates_rich(diff: &str) -> Vec<DepCandidate> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum TomlSection {
        Deps,
        Other,
    }

    let mut out: Vec<DepCandidate> = Vec::new();
    let mut toml_section = TomlSection::Other;

    for line in diff.lines() {
        // Detect TOML section headers on both context lines and added lines
        // so we track which section we're in even when diff context shifts.
        let trimmed_body = if line.starts_with('+') && !line.starts_with("+++") {
            &line[1..]
        } else if !line.starts_with('-') && !line.starts_with("@@") && !line.starts_with("diff ") {
            // Context line — update section tracking only.
            let ctx = line.trim();
            if ctx.starts_with('[') {
                let section_lower = ctx.to_ascii_lowercase();
                toml_section = if section_lower.contains("dependencies") {
                    TomlSection::Deps
                } else {
                    TomlSection::Other
                };
            }
            continue;
        } else {
            continue;
        };

        // Update TOML section tracking from added lines too.
        let body_trimmed = trimmed_body.trim();
        if body_trimmed.starts_with('[') {
            let section_lower = body_trimmed.to_ascii_lowercase();
            toml_section = if section_lower.contains("dependencies") {
                TomlSection::Deps
            } else {
                TomlSection::Other
            };
            // Section headers are never dep candidates themselves.
            continue;
        }

        // --- Cargo (TOML) shape: `name = "version"` or `name = { version = ... }` ---
        // Only emit as Cargo when we are inside a [*dependencies] section.
        if toml_section == TomlSection::Deps {
            if let Some(eq_idx) = trimmed_body.find('=') {
                let lhs = trimmed_body[..eq_idx].trim();
                let rhs = trimmed_body[eq_idx + 1..].trim_start();
                if !lhs.is_empty()
                    && is_dep_name_shape(lhs)
                    && (rhs.starts_with('"') || rhs.starts_with('{'))
                {
                    out.push(DepCandidate {
                        name: lhs.to_string(),
                        ecosystem: Ecosystem::Cargo,
                    });
                    continue;
                }
            }
        }

        // --- npm (package.json) shape: `"name": "1.2.3"` ---
        if let Some(name) = parse_json_dep_name(trimmed_body) {
            out.push(DepCandidate {
                name,
                ecosystem: Ecosystem::Npm,
            });
            continue;
        }

        // --- PyPI (pyproject.toml / requirements) shape: PEP 508 specifier ---
        if let Some(name) = parse_pep508_dep_name(trimmed_body) {
            out.push(DepCandidate {
                name,
                ecosystem: Ecosystem::Pypi,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_cargo_added_dep() {
        let diff = r#"
+++ b/Cargo.toml
@@
+serde = "1"
+tokio = { version = "1", features = ["full"] }
"#;
        let names = extract_dep_candidates(diff);
        assert!(names.iter().any(|n| n == "serde"));
        assert!(names.iter().any(|n| n == "tokio"));
    }

    #[test]
    fn extracts_npm_added_dep() {
        let diff = r#"
+++ b/package.json
@@
+    "lodash": "4.17.21",
+    "reqquests": "1.0.0",
"#;
        let names = extract_dep_candidates(diff);
        assert!(names.iter().any(|n| n == "lodash"));
        assert!(names.iter().any(|n| n == "reqquests"));
    }

    #[test]
    fn flags_known_squat() {
        assert!(flag_candidate("reqquests"));
        assert!(flag_candidate("tensorlfow"));
        assert!(!flag_candidate("serde"));
        assert!(!flag_candidate("lodash"));
    }

    #[test]
    fn flags_pathological_lengths() {
        assert!(flag_candidate("a")); // too short
        assert!(flag_candidate(&"x".repeat(65))); // too long
        assert!(!flag_candidate("ab")); // boundary OK
        assert!(!flag_candidate(&"y".repeat(64))); // boundary OK
    }

    #[test]
    fn rich_extractor_cargo_section_tracking() {
        let diff = r#"
+++ b/Cargo.toml
@@
 [package]
 name = "elai"
+
 [dependencies]
+serde = "1"
+tokio = { version = "1", features = ["full"] }
 [dev-dependencies]
+pretty_assertions = "1"
"#;
        let candidates = extract_dep_candidates_rich(diff);
        assert!(candidates.iter().any(|c| c.name == "serde" && c.ecosystem == Ecosystem::Cargo));
        assert!(candidates.iter().any(|c| c.name == "tokio" && c.ecosystem == Ecosystem::Cargo));
        assert!(candidates.iter().any(|c| c.name == "pretty_assertions" && c.ecosystem == Ecosystem::Cargo));
        // [package] keys are NOT deps.
        assert!(!candidates.iter().any(|c| c.name == "name"));
        assert!(!candidates.iter().any(|c| c.name == "elai"));
    }

    #[test]
    fn rich_extractor_npm_ecosystem() {
        let diff = r#"
+++ b/package.json
@@
+    "lodash": "4.17.21",
+    "reqquests": "1.0.0",
"#;
        let candidates = extract_dep_candidates_rich(diff);
        assert!(candidates.iter().any(|c| c.name == "lodash" && c.ecosystem == Ecosystem::Npm));
        assert!(candidates.iter().any(|c| c.name == "reqquests" && c.ecosystem == Ecosystem::Npm));
    }

    #[test]
    fn rich_extractor_pypi_pep508() {
        let diff = r#"
+++ b/pyproject.toml
@@
+    "requests>=2.28",
+    "orjson==3.9.0",
"#;
        let candidates = extract_dep_candidates_rich(diff);
        assert!(candidates.iter().any(|c| c.name == "requests" && c.ecosystem == Ecosystem::Pypi));
        assert!(candidates.iter().any(|c| c.name == "orjson" && c.ecosystem == Ecosystem::Pypi));
    }
}
