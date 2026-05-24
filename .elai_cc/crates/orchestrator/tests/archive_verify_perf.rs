//! D-8 — Performance test for `verify_archive`. Generates a synthetic
//! 10K-entry ArchiveChain in a tempdir, runs `verify_archive`, and asserts
//! the wall-clock walk completes in under 5 seconds.
//!
//! Marked `#[ignore]` because it is intentionally slow (writes 10K JSONL
//! entries to a tempdir + reads them back); the gate runs only via
//! `cargo test -p elai-orchestrator --test archive_verify_perf -- --ignored`.
//! Per typical Rust perf-test convention this keeps the default
//! `cargo test` runtime stable.
//!
//! Bound rationale (5s): D-1's `compute_content_hash` uses blake3 (anti-
//! stale-training rule + workspace dep). On commodity hardware blake3
//! sustains ~5 GB/s on AVX2; a 10K-entry walk is dominated by JSONL
//! parse + entry deserialise (rmp / serde_json), not the hash itself.
//! 5 seconds is a generous ceiling that catches O(n²) regressions
//! without flaking on slow CI runners.

use std::time::Instant;

use elai_orchestrator::archive_chain::{verify_archive, ArchiveChain, ArchiveEntry};
use elai_orchestrator::reasoning_template::{
    compute_content_hash, ArchiveStatus, Blake3Hash, Domain, PatternKind, Predicate,
    ReasoningTemplate, TemplateBody, TemplateId, VerifierKind,
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

/// D-8 perf gate: 10K-entry verify <= 5 seconds wall-clock. Ignored by
/// default — run via `--ignored`.
#[test]
#[ignore]
fn verify_archive_10k_entries_under_5s() {
    const N: usize = 10_000;
    const BUDGET_SECS: u64 = 5;

    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("perf-archive.jsonl");

    // Build a linear chain of N entries (each child references the
    // previous self_hash). The body varies per entry so blake3 inputs
    // are not memoised.
    let mut chain = ArchiveChain::new(path.clone());
    let mut prev_hash: Option<Blake3Hash> = None;
    for i in 0..N {
        let t = mk_template(&format!("perf-body-{i:06}"));
        let entry = match prev_hash {
            None => ArchiveEntry::new_root(t),
            Some(parent) => ArchiveEntry::new_child(t, parent),
        };
        prev_hash = Some(entry.self_hash);
        chain.append(entry).expect("append");
    }

    let started = Instant::now();
    let report = verify_archive(&path, false).expect("verify ok");
    let elapsed = started.elapsed();

    assert!(
        report.passed,
        "synthetic chain must verify clean; got {} errors",
        report.errors.len()
    );
    assert_eq!(report.total_entries as usize, N);
    assert_eq!(report.verified as usize, N);

    let elapsed_secs = elapsed.as_secs_f64();
    assert!(
        elapsed.as_secs() < BUDGET_SECS,
        "verify_archive over {N} entries took {elapsed_secs:.3}s (>= {BUDGET_SECS}s budget). \
         Likely an O(n^2) regression in the walker."
    );
    eprintln!(
        "verify_archive {N} entries: {elapsed_secs:.3}s (budget {BUDGET_SECS}s)"
    );
}
