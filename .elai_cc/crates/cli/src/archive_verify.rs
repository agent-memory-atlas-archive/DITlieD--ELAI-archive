//! D-8 — `elai archive verify` CLI driver.
//!
//! Surfaces D-1's [`elai_orchestrator::ArchiveChain::verify_chain`] as a
//! user-facing subcommand + CI hook. Lobotomy-attack defence per
//! arxiv:2507.05619 (T1) cryptographic-checksum recommendation + DGM §5
//! archive practice (companion §3.8 lines 319-337) — both are supporting
//! documentation for the integrity-rationale; neither paper specifies the
//! wire layout / mechanism (per workflow/reasoning/D-8-decisions.md
//! DECISION-5 PLAN_PAPER_CONFLATION guard).
//!
//! Wiring discipline:
//!   producer  : `elai_orchestrator::ArchiveChain::{load,load_lenient}` (D-1+D-8)
//!   consumer  : this file — loads the JSONL archive and walks the chain.
//!   ssb row   : `SsbActionType::ArchiveVerifyResult` (D-8 SCHEMA_VERSION 115).
//!   ci hook   : `.github/workflows/archive-verify.yml` at the workflow-tooling
//!               root (NOT under `.elai_cc/.github/` — workflows live at the
//!               repo root per F:\.ELAI_workflow\.github\ convention).
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
//! [`ArchiveVerifyError`] (re-exported from
//! [`elai_orchestrator::archive_chain`]) via `thiserror`.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::archive_chain::{
    verify_archive as verify_archive_canonical, ArchiveVerifyError, ArchiveVerifyReport,
};
use elai_orchestrator::ssb::{ArchiveVerifyResultPayload, SsbPayload};
use elai_orchestrator::{Ssb, SsbConfig};

// ---------------------------------------------------------------------------
// Public verify entry — re-export of the orchestrator's canonical walker.
//
// Single source of truth lives in
// `elai_orchestrator::archive_chain::verify_archive` so the CLI, contract
// tests, and any future programmatic consumer share one canonical walker.
// This wrapper exists purely to preserve the call shape callers in this
// crate already use (`&PathBuf`, sync entry).
// ---------------------------------------------------------------------------

pub fn verify_archive(
    archive_path: &PathBuf,
    fail_fast: bool,
) -> Result<ArchiveVerifyReport, ArchiveVerifyError> {
    verify_archive_canonical(archive_path.as_path(), fail_fast)
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

/// CLI entry — parses three flags, runs [`verify_archive`], emits the
/// [`SsbActionType::ArchiveVerifyResult`] row, renders + exits.
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

    // Build a current-thread tokio runtime — the SSB writer requires a
    // runtime context for write_payload (Ssb::new spawns a single-writer
    // task). Sync entry pattern matches github_pr_cmd / ppm_health.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("building tokio runtime for archive verify: {e}"))?;

    let report_result = verify_archive(&archive_path, fail_fast);

    // Emit the ArchiveVerifyResult SSB row regardless of pass / fail.
    // Infrastructural error (Err) -> row carries error_count=1 with the
    // synthesised Io / MalformedJson error in the typed list so dashboards
    // do not silently lose the run. The CLI then exits 1 immediately
    // after.
    let (report_or_err_label, ssb_payload): (Result<&ArchiveVerifyReport, &ArchiveVerifyError>, ArchiveVerifyResultPayload) =
        match &report_result {
            Ok(report) => (
                Ok(report),
                ArchiveVerifyResultPayload {
                    audit_id: report.archive_path.clone(),
                    total_entries: report.total_entries,
                    verified: report.passed,
                    error_count: u32::try_from(report.errors.len()).unwrap_or(u32::MAX),
                    errors: report.errors.clone(),
                    ts_ms: current_ms(),
                },
            ),
            Err(e) => (
                Err(e),
                ArchiveVerifyResultPayload {
                    audit_id: archive_path.to_string_lossy().into_owned(),
                    total_entries: 0,
                    verified: false,
                    error_count: 1,
                    errors: vec![e.clone()],
                    ts_ms: current_ms(),
                },
            ),
        };

    // SSB emission is best-effort — a writer construction failure must not
    // mask the verify result. We log + continue.
    let project_root_owned = project_root.clone();
    let ssb_emit = rt.block_on(async move {
        emit_archive_verify_ssb(&project_root_owned, ssb_payload).await
    });
    if let Err(e) = ssb_emit {
        eprintln!("warning: failed to emit ArchiveVerifyResult SSB row: {e}");
    }

    let report = match report_or_err_label {
        Ok(r) => r.clone(),
        Err(e) => {
            // Infrastructural failure -- emit a one-line error and exit 1.
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

/// Construct an SSB writer against the active project's session DB and
/// emit one `ArchiveVerifyResult` row. Errors propagate to the caller for
/// logging — the verify result has already been computed by this point,
/// so SSB failure must not change the exit code.
async fn emit_archive_verify_ssb(
    project_root: &std::path::Path,
    payload: ArchiveVerifyResultPayload,
) -> anyhow::Result<()> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    let parent = db_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("ssb.db has no parent dir; cannot prepare path"))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| anyhow::anyhow!("creating ssb sessions dir: {e}"))?;
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-archive-verify".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(cfg, clock)
        .map_err(|e| anyhow::anyhow!("opening SSB writer: {e}"))?;
    ssb
        .write_payload(
            "cli:archive-verify",
            0,
            SsbPayload::ArchiveVerifyResult(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("writing ArchiveVerifyResult row: {e}"))?;
    Ok(())
}

fn current_ms() -> i64 {
    // Saturation semantics matching ssb.rs `Ssb::now_ms` — durations before
    // 1970-01-01 are clamped to 0 rather than panicking. UNIX epoch
    // failure is not an error we model in the report.
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_millis()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
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

    /// D-8 GAP-2: lenient JSONL parsing surfaces MalformedJson without
    /// short-circuiting the rest of the walk.
    #[test]
    fn verify_collects_malformed_json_via_lenient_load() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("archive.jsonl");

        // Build a clean one-entry chain then append a corrupted line.
        let mut chain = ArchiveChain::new(path.clone());
        let r = ArchiveEntry::new_root(mk_template("root"));
        chain.append(r).expect("append root");
        // Manually append a malformed line.
        let extra = "{this-is-not-valid-json}\n";
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("open append");
        std::io::Write::write_all(&mut f, extra.as_bytes()).expect("write extra");

        let rep = verify_archive(&path, false).expect("verify ok");
        assert!(!rep.passed);
        // total_entries counts only well-formed parses (1), error list
        // has the MalformedJson row.
        assert_eq!(rep.total_entries, 1);
        let has_malformed = rep
            .errors
            .iter()
            .any(|e| matches!(e, ArchiveVerifyError::MalformedJson { .. }));
        assert!(has_malformed, "expected MalformedJson row, got {rep:?}");
    }
}
