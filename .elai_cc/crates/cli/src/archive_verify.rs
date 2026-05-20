//! D-8 — `elai archive verify` CLI driver.
//!
//! Surfaces D-1's [`elai_orchestrator::ArchiveChain::verify_chain`] as a
//! user-facing subcommand + CI hook. Lobotomy-attack defence per
//! arxiv:2507.05619 (T1) cryptographic-checksum recommendation + DGM §5
//! archive practice (companion §3.8 lines 319-337).
//!
//! Wiring discipline:
//!   producer  : `elai_orchestrator::ArchiveChain::verify_chain` (D-1)
//!   consumer  : this file — loads the JSONL archive and walks the chain.
//!   ssb row   : `SsbActionType::ArchiveVerifyResult` (D-8 SSB variant).
//!   ci hook   : skipped this session (no `.github/workflows/wiring-audit.yml`
//!               at the project root yet; recorded as followup in the
//!               return summary).
//!
//! Exit codes:
//!   0 = chain verified (no errors)
//!   1 = at least one verify error detected
//!
//! Output:
//!   text (default) or JSON (`--json`). JSON is jq-parseable (single object).
//!
//! Flags:
//!   --path PATH    on-disk archive JSONL (default
//!                  `.elai_cc/archive/reasoning-templates.jsonl`)
//!   --json         emit `ArchiveVerifyReport` as JSON
//!   --fail-fast    short-circuit on first error rather than collect-all
//!
//! No `.unwrap()` in non-test code. All I/O + chain errors flow through
//! [`ArchiveVerifyError`] via `thiserror`.

use std::path::PathBuf;

use elai_orchestrator::{archive_chain::ArchiveChain, reasoning_template::TemplateError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type (thiserror per CLAUDE.md anti-stale rule).
// ---------------------------------------------------------------------------

/// Per-entry verify failure mode. Each variant carries the entry index so the
/// dashboard / CI log can point at the broken row directly.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArchiveVerifyError {
    /// Recomputed `self_hash` did not match the stored value.
    #[error(
        "hash_mismatch at entry {entry_index} (template_id={template_id}): expected={expected} computed={computed}"
    )]
    HashMismatch {
        entry_index: u32,
        expected: String,
        computed: String,
        template_id: String,
    },
    /// `parent_hash` references a hash not present earlier in the chain.
    #[error(
        "broken_parent at entry {entry_index} (template_id={template_id}): parent_hash={parent_hash} not seen earlier"
    )]
    BrokenParent {
        entry_index: u32,
        parent_hash: String,
        template_id: String,
    },
    /// I/O failure loading the JSONL file (missing file becomes an empty
    /// chain in D-1, so this is *only* "exists but can't read").
    #[error("io error reading {path}: {source_msg}")]
    Io { path: String, source_msg: String },
    /// JSONL deserialise failure on at least one line.
    #[error("malformed_json reading {path}: {source_msg}")]
    MalformedJson { path: String, source_msg: String },
}

// ---------------------------------------------------------------------------
// Report struct (JSON + text renderers).
// ---------------------------------------------------------------------------

/// Outcome of one `elai archive verify` invocation. Serialised verbatim
/// for `--json`; the text renderer formats a human-readable summary +
/// per-error line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveVerifyReport {
    pub archive_path: String,
    pub total_entries: u32,
    pub verified: u32,
    pub errors: Vec<ArchiveVerifyError>,
    /// `true` if `errors.is_empty()`.
    pub passed: bool,
    /// `true` if `--fail-fast` was set on this run (so callers can tell
    /// the difference between "really clean" and "stopped early").
    pub fail_fast: bool,
}

impl ArchiveVerifyReport {
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Public verify entry — usable from contract tests + Tauri commands.
// ---------------------------------------------------------------------------

/// Load the chain at `archive_path`, walk every entry, recompute `self_hash`
/// per `(template_body, parent_hash)`, and check parent reachability.
///
/// Returns a populated [`ArchiveVerifyReport`] (errors collected when
/// `fail_fast=false`, single-error short-circuit when `fail_fast=true`).
///
/// `Err` is reserved for *infrastructural* failures (missing-but-required
/// path, IO error, malformed JSON). Tamper / mismatch findings are encoded
/// inside `report.errors`, NOT as `Err`, so the CLI can render a structured
/// report on exit-1 instead of dumping a backtrace.
pub fn verify_archive(
    archive_path: &PathBuf,
    fail_fast: bool,
) -> Result<ArchiveVerifyReport, ArchiveVerifyError> {
    let path_str = archive_path.to_string_lossy().into_owned();

    // D-1's `ArchiveChain::load` returns Ok(empty) for a missing file. That's
    // the desired semantics here: a fresh repo with no archive has nothing to
    // tamper with -> verify passes vacuously.
    let chain = ArchiveChain::load(archive_path.clone()).map_err(|e| match e {
        TemplateError::Io(msg) => ArchiveVerifyError::Io {
            path: path_str.clone(),
            source_msg: msg,
        },
        TemplateError::Serde(msg) => ArchiveVerifyError::MalformedJson {
            path: path_str.clone(),
            source_msg: msg,
        },
        other => ArchiveVerifyError::Io {
            path: path_str.clone(),
            source_msg: format!("{other:?}"),
        },
    })?;

    let total_entries = u32::try_from(chain.len()).unwrap_or(u32::MAX);

    // Empty chain -> trivially clean.
    if chain.is_empty() {
        return Ok(ArchiveVerifyReport {
            archive_path: path_str,
            total_entries: 0,
            verified: 0,
            errors: Vec::new(),
            passed: true,
            fail_fast,
        });
    }

    // Walk the chain ourselves (not via verify_chain) so we can report the
    // exact entry index + expected/computed hash + collect-multiple. D-1's
    // verify_chain returns on first error and does not surface the index.
    let mut errors: Vec<ArchiveVerifyError> = Vec::new();
    let mut verified: u32 = 0;
    let mut seen_hashes: std::collections::HashSet<elai_orchestrator::reasoning_template::Blake3Hash> =
        std::collections::HashSet::new();

    for (idx, entry) in chain.entries().iter().enumerate() {
        let entry_index = u32::try_from(idx).unwrap_or(u32::MAX);
        let template_id = entry.template.id.as_hex();

        // Parent reachability.
        if let Some(parent_hash) = entry.parent_hash {
            if !seen_hashes.contains(&parent_hash) {
                errors.push(ArchiveVerifyError::BrokenParent {
                    entry_index,
                    parent_hash: parent_hash.as_hex(),
                    template_id: template_id.clone(),
                });
                if fail_fast {
                    break;
                }
                // Skip self-hash check for broken-parent rows: parent_hash is
                // an input to the hash, so a fake parent guarantees the
                // self-hash recompute also disagrees -- reporting both would
                // be noise. Move on to next entry.
                seen_hashes.insert(entry.self_hash);
                continue;
            }
        }

        // Self-hash recomputation.
        let recomputed = elai_orchestrator::reasoning_template::compute_content_hash(
            entry.template.template_body.as_str(),
            entry.parent_hash.as_ref(),
        );
        if recomputed != entry.self_hash {
            errors.push(ArchiveVerifyError::HashMismatch {
                entry_index,
                expected: entry.self_hash.as_hex(),
                computed: recomputed.as_hex(),
                template_id,
            });
            if fail_fast {
                break;
            }
        } else {
            verified = verified.saturating_add(1);
        }

        seen_hashes.insert(entry.self_hash);
    }

    let passed = errors.is_empty();
    Ok(ArchiveVerifyReport {
        archive_path: path_str,
        total_entries,
        verified,
        errors,
        passed,
        fail_fast,
    })
}

// ---------------------------------------------------------------------------
// CLI entry — sync wrapper called from main.rs / live_cli.rs.
// ---------------------------------------------------------------------------

/// Default JSONL archive path matching D-1's writer (`reasoning_template.rs`
/// + `archive_chain::append`). The CLI is invoked from `.elai_cc/` (CI cwd),
/// so the relative path resolves the same way `elai wiring audit` resolves
/// its contracts dir.
fn default_archive_path(project_root: &std::path::Path) -> PathBuf {
    project_root.join("archive").join("reasoning-templates.jsonl")
}

/// CLI entry — parses three flags, runs [`verify_archive`], renders + exits.
///
/// Returns `Ok(())` on clean exit (rc=0); calls `std::process::exit(1)` on
/// any verify error. Returning `Err(anyhow::Error)` is reserved for
/// infrastructural failures (unable to even open the file) so the top-level
/// dispatcher can format them through the standard error path.
pub(crate) fn run(
    path: Option<String>,
    json: bool,
    fail_fast: bool,
) -> anyhow::Result<()> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let archive_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| default_archive_path(&project_root));

    let report = match verify_archive(&archive_path, fail_fast) {
        Ok(r) => r,
        Err(e) => {
            // Infrastructural failure -- emit a one-line error and exit 1.
            // Mirrors wiring_audit's approach (printed by upstream printer).
            if json {
                let body = serde_json::json!({
                    "archive_path": archive_path.to_string_lossy(),
                    "passed": false,
                    "errors": [e],
                });
                println!("{}", serde_json::to_string_pretty(&body)?);
            } else {
                println!("ARCHIVE VERIFY FAIL ({e})");
            }
            std::process::exit(1);
        }
    };

    render_report(json, &report)?;
    if !report.passed {
        std::process::exit(1);
    }
    Ok(())
}

fn render_report(json: bool, report: &ArchiveVerifyReport) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    if report.passed {
        println!(
            "ARCHIVE VERIFY PASS ({} entries, path={})",
            report.total_entries, report.archive_path
        );
        return Ok(());
    }

    println!(
        "ARCHIVE VERIFY FAIL ({} errors over {} entries; path={})",
        report.errors.len(),
        report.total_entries,
        report.archive_path
    );
    for err in &report.errors {
        println!("  - {err}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests (in-crate; full tamper matrix lives in the contract test).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::archive_chain::{ArchiveChain, ArchiveEntry};
    use elai_orchestrator::reasoning_template::{
        compute_content_hash, ArchiveStatus, Domain, PatternKind, Predicate, ReasoningTemplate,
        TemplateBody, TemplateId, VerifierKind,
    };
    use tempfile::TempDir;

    fn mk_template(body: &str) -> ReasoningTemplate {
        let body = TemplateBody::try_new(body).expect("body fits");
        let content_hash = compute_content_hash(body.as_str(), None);
        ReasoningTemplate {
            id: TemplateId::new_v4(),
            domain: Domain::Code,
            pattern_kind: PatternKind::Decomposition,
            template_body: body,
            eligibility_predicate: Predicate::Always,
            verifier_required: VerifierKind::Compile,
            content_hash,
            created_at_ms: 0,
            parent_id: None,
            archive_status: ArchiveStatus::Frozen,
        }
    }

    #[test]
    fn verify_empty_archive_is_clean() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("does-not-exist.jsonl");
        let rep = verify_archive(&path, false).expect("verify ok");
        assert!(rep.passed);
        assert_eq!(rep.total_entries, 0);
        assert_eq!(rep.errors.len(), 0);
    }

    #[test]
    fn verify_clean_chain_passes() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("archive.jsonl");
        let mut chain = ArchiveChain::new(path.clone());
        let t1 = mk_template("root");
        let root = ArchiveEntry::new_root(t1);
        let root_hash = root.self_hash;
        chain.append(root).expect("append root");
        let t2 = mk_template("child");
        chain
            .append(ArchiveEntry::new_child(t2, root_hash))
            .expect("append child");

        let rep = verify_archive(&path, false).expect("verify ok");
        assert!(rep.passed, "expected pass, got {rep:?}");
        assert_eq!(rep.total_entries, 2);
        assert_eq!(rep.verified, 2);
        assert!(rep.errors.is_empty());
    }

    #[test]
    fn verify_detects_hash_mismatch_with_entry_index() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("archive.jsonl");

        // Build a clean two-entry chain on disk.
        let mut chain = ArchiveChain::new(path.clone());
        let t1 = mk_template("root");
        let root = ArchiveEntry::new_root(t1);
        let root_hash = root.self_hash;
        chain.append(root).expect("append root");
        let t2 = mk_template("child");
        chain
            .append(ArchiveEntry::new_child(t2, root_hash))
            .expect("append child");

        // Tamper: rewrite the child line so its body says "tampered" but
        // self_hash stays as the original.
        let raw = std::fs::read_to_string(&path).expect("read");
        let mut lines: Vec<String> = raw.lines().map(String::from).collect();
        assert_eq!(lines.len(), 2);
        // Mutate template_body inside line 2 (the child).
        let mutated = lines[1].replace("\"template_body\":\"child\"", "\"template_body\":\"tampered\"");
        assert_ne!(mutated, lines[1], "expected substitution to find the field");
        lines[1] = mutated;
        std::fs::write(&path, lines.join("\n") + "\n").expect("write");

        let rep = verify_archive(&path, false).expect("verify ok");
        assert!(!rep.passed);
        assert_eq!(rep.total_entries, 2);
        assert_eq!(rep.errors.len(), 1);
        match &rep.errors[0] {
            ArchiveVerifyError::HashMismatch {
                entry_index,
                expected,
                computed,
                ..
            } => {
                assert_eq!(*entry_index, 1, "tampered child is at index 1");
                assert_ne!(expected, computed, "hashes must differ on tamper");
            }
            other => panic!("expected HashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn verify_fail_fast_stops_after_first_error() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("archive.jsonl");

        // Build clean three-entry chain.
        let mut chain = ArchiveChain::new(path.clone());
        let r = ArchiveEntry::new_root(mk_template("root"));
        let r_hash = r.self_hash;
        chain.append(r).expect("a1");
        let c1 = ArchiveEntry::new_child(mk_template("c1"), r_hash);
        let c1_hash = c1.self_hash;
        chain.append(c1).expect("a2");
        let c2 = ArchiveEntry::new_child(mk_template("c2"), c1_hash);
        chain.append(c2).expect("a3");

        // Tamper BOTH child rows.
        let raw = std::fs::read_to_string(&path).expect("read");
        let mut lines: Vec<String> = raw.lines().map(String::from).collect();
        lines[1] = lines[1].replace("\"template_body\":\"c1\"", "\"template_body\":\"x1\"");
        lines[2] = lines[2].replace("\"template_body\":\"c2\"", "\"template_body\":\"x2\"");
        std::fs::write(&path, lines.join("\n") + "\n").expect("write");

        let collect = verify_archive(&path, false).expect("verify ok");
        assert_eq!(collect.errors.len(), 2, "collect-mode finds both tampers");

        let fast = verify_archive(&path, true).expect("verify ok");
        assert_eq!(fast.errors.len(), 1, "fail-fast stops at first");
        assert!(fast.fail_fast);
    }
}
