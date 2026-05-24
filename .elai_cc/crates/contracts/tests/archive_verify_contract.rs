//! D-8 — Contract test for the `elai archive verify` 5-scenario tamper
//! matrix. Walks the canonical
//! [`elai_orchestrator::archive_chain::verify_archive`] over synthetic
//! tampered JSONL archives, asserting that every tamper class surfaces
//! the expected typed [`ArchiveVerifyError`] variant.
//!
//! Tamper classes (one test each):
//!   1. happy_path — clean archive verifies (verified=true, errors=0).
//!   2. broken_parent — link broken at index N -> BrokenParent.
//!   3. content_hash_mismatch — tampered entry body -> HashMismatch.
//!   4. duplicate_hash — two entries with same self_hash -> DuplicateHash.
//!   5. malformed_json — corrupted JSONL line -> MalformedJson via
//!      load_lenient (collect-mode keeps walking).
//!
//! Grounding: arxiv:2507.05619 (T1 cryptographic-checksum) + DGM §5
//! (archive practice) are supporting documentation for the lobotomy-
//! attack rationale; neither paper specifies the tamper matrix. See
//! workflow/reasoning/D-8-decisions.md DECISION-5 PLAN_PAPER_CONFLATION
//! guard.

use std::fs;
use std::path::PathBuf;

use elai_orchestrator::archive_chain::{
    verify_archive, ArchiveChain, ArchiveEntry, ArchiveVerifyError,
};
use elai_orchestrator::reasoning_template::{
    compute_content_hash, ArchiveStatus, Blake3Hash, Domain, PatternKind, Predicate,
    ReasoningTemplate, TemplateBody, TemplateId, VerifierKind,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

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

/// Build a clean N-entry chain in `dir` with the given bodies. Returns
/// the JSONL path + the in-memory chain (so the test can inspect the
/// canonical hashes).
fn build_clean_chain(dir: &TempDir, bodies: &[&str]) -> (PathBuf, ArchiveChain) {
    assert!(!bodies.is_empty(), "need at least one body");
    let path = dir.path().join("archive.jsonl");
    let mut chain = ArchiveChain::new(path.clone());
    let mut last_hash: Option<Blake3Hash> = None;
    for body in bodies {
        let t = mk_template(body);
        let entry = match last_hash {
            None => ArchiveEntry::new_root(t),
            Some(parent) => ArchiveEntry::new_child(t, parent),
        };
        last_hash = Some(entry.self_hash);
        chain.append(entry).expect("append");
    }
    (path, chain)
}

// ---------------------------------------------------------------------------
// 1. Happy path
// ---------------------------------------------------------------------------

#[test]
fn d8_happy_path_clean_archive_verifies() {
    let dir = TempDir::new().expect("tempdir");
    let (path, _chain) = build_clean_chain(&dir, &["root", "child1", "child2"]);

    let report = verify_archive(&path, false).expect("verify ok");
    assert!(report.passed, "clean chain must pass, got: {report:?}");
    assert_eq!(report.total_entries, 3);
    assert_eq!(report.verified, 3);
    assert_eq!(report.errors.len(), 0);
    assert!(!report.fail_fast);
}

// ---------------------------------------------------------------------------
// 2. BrokenParent
// ---------------------------------------------------------------------------

#[test]
fn d8_broken_parent_link_detected() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("archive.jsonl");

    // Build a root, then a child whose parent_hash refers to a hash that
    // was NEVER appended (forged parent). Use a deterministic blake3
    // pattern so the asserted hex string is stable across platforms.
    let mut chain = ArchiveChain::new(path.clone());
    let root = ArchiveEntry::new_root(mk_template("root"));
    chain.append(root).expect("append root");

    let fake_parent = Blake3Hash([0x42u8; 32]);
    let child = ArchiveEntry::new_child(mk_template("orphan"), fake_parent);
    chain.append(child).expect("append child");

    let report = verify_archive(&path, false).expect("verify ok");
    assert!(!report.passed, "broken parent must fail");
    assert_eq!(report.total_entries, 2);
    assert_eq!(report.errors.len(), 1, "exactly one BrokenParent expected");
    match &report.errors[0] {
        ArchiveVerifyError::BrokenParent {
            entry_index,
            parent_hash,
            ..
        } => {
            assert_eq!(*entry_index, 1, "orphan is at index 1");
            // 0x42 repeated 32 times -> hex string of 64 '42's.
            assert_eq!(parent_hash, &"42".repeat(32));
        }
        other => panic!("expected BrokenParent, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 3. ContentHashMismatch
// ---------------------------------------------------------------------------

#[test]
fn d8_content_hash_mismatch_detected() {
    let dir = TempDir::new().expect("tempdir");
    let (path, _) = build_clean_chain(&dir, &["root", "child"]);

    // Tamper: rewrite the child line so its body says "tampered" but the
    // self_hash field stays as the original. This is the canonical
    // lobotomy attack pattern (body mutation post-archive).
    let raw = fs::read_to_string(&path).expect("read");
    let mut lines: Vec<String> = raw.lines().map(String::from).collect();
    assert_eq!(lines.len(), 2, "fixture must have 2 lines");
    let mutated = lines[1].replace(
        "\"template_body\":\"child\"",
        "\"template_body\":\"tampered\"",
    );
    assert_ne!(mutated, lines[1], "expected substitution to land");
    lines[1] = mutated;
    fs::write(&path, lines.join("\n") + "\n").expect("write");

    let report = verify_archive(&path, false).expect("verify ok");
    assert!(!report.passed);
    assert_eq!(report.total_entries, 2);
    assert_eq!(report.errors.len(), 1);
    match &report.errors[0] {
        ArchiveVerifyError::HashMismatch {
            entry_index,
            expected,
            computed,
            ..
        } => {
            assert_eq!(*entry_index, 1);
            assert_ne!(expected, computed, "tamper must shift the hash");
            // Both are blake3 64-hex strings.
            assert_eq!(expected.len(), 64);
            assert_eq!(computed.len(), 64);
        }
        other => panic!("expected HashMismatch, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 4. DuplicateHash
// ---------------------------------------------------------------------------

#[test]
fn d8_duplicate_hash_detected() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("archive.jsonl");

    // Build a root entry and serialise it twice (same body -> same
    // template_body input -> same self_hash). Writing the JSON line
    // twice gives us two entries with identical self_hash values, which
    // is the duplicate-hash detection contract.
    let root = ArchiveEntry::new_root(mk_template("dup_body"));
    let line = serde_json::to_string(&root).expect("serialize root");
    fs::write(&path, format!("{line}\n{line}\n")).expect("write");

    let report = verify_archive(&path, false).expect("verify ok");
    assert!(!report.passed);
    assert_eq!(report.total_entries, 2);
    let dup_count = report
        .errors
        .iter()
        .filter(|e| matches!(e, ArchiveVerifyError::DuplicateHash { .. }))
        .count();
    assert_eq!(dup_count, 1, "expected one DuplicateHash row, got {report:?}");
    let Some(ArchiveVerifyError::DuplicateHash {
        hash,
        first_at_index,
        second_at_index,
    }) = report
        .errors
        .iter()
        .find(|e| matches!(e, ArchiveVerifyError::DuplicateHash { .. }))
    else {
        panic!("DuplicateHash variant not found in {report:?}");
    };
    assert_eq!(*first_at_index, 0);
    assert_eq!(*second_at_index, 1);
    assert_eq!(hash.len(), 64, "blake3 hex is 64 chars");
}

// ---------------------------------------------------------------------------
// 5. MalformedJson (via load_lenient)
// ---------------------------------------------------------------------------

#[test]
fn d8_malformed_json_surfaced_via_lenient_load() {
    let dir = TempDir::new().expect("tempdir");
    let (path, _) = build_clean_chain(&dir, &["root"]);

    // Append a corrupted line after the clean entry. load_lenient must
    // keep walking and surface MalformedJson with entry_index=1.
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .expect("open append");
    std::io::Write::write_all(&mut f, b"{this-is-not-valid-json}\n").expect("write corrupt");

    let report = verify_archive(&path, false).expect("verify ok");
    assert!(!report.passed);
    // total_entries counts well-formed parses only (1).
    assert_eq!(report.total_entries, 1);
    let malformed: Vec<_> = report
        .errors
        .iter()
        .filter_map(|e| match e {
            ArchiveVerifyError::MalformedJson {
                entry_index,
                raw,
                parse_error,
            } => Some((entry_index, raw, parse_error)),
            _ => None,
        })
        .collect();
    assert_eq!(malformed.len(), 1, "expected one MalformedJson row");
    let (entry_index, raw, parse_error) = malformed[0];
    assert_eq!(*entry_index, 1, "malformed line is the 2nd attempted parse");
    assert!(raw.contains("not-valid-json"), "raw text must echo the corrupt line");
    assert!(!parse_error.is_empty(), "parse_error must carry a serde_json message");
}

// ---------------------------------------------------------------------------
// 6. Bonus — fail_fast short-circuits on multi-error chains
// ---------------------------------------------------------------------------

#[test]
fn d8_fail_fast_short_circuits_collect_mode_does_not() {
    let dir = TempDir::new().expect("tempdir");
    let (path, _) = build_clean_chain(&dir, &["root", "c1", "c2"]);

    // Tamper both child bodies.
    let raw = fs::read_to_string(&path).expect("read");
    let mut lines: Vec<String> = raw.lines().map(String::from).collect();
    lines[1] = lines[1].replace("\"template_body\":\"c1\"", "\"template_body\":\"x1\"");
    lines[2] = lines[2].replace("\"template_body\":\"c2\"", "\"template_body\":\"x2\"");
    fs::write(&path, lines.join("\n") + "\n").expect("write");

    let collect = verify_archive(&path, false).expect("collect verify ok");
    assert_eq!(
        collect.errors.len(),
        2,
        "collect mode must surface both tampers"
    );
    assert!(!collect.fail_fast);

    let fast = verify_archive(&path, true).expect("fast verify ok");
    assert_eq!(fast.errors.len(), 1, "fail_fast stops at the first error");
    assert!(fast.fail_fast);
}
