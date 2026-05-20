//! D-8 observable_effect smoke test.
//!
//! Plan WIRING_CONTRACT observable_effect + orchestrator task 5(e):
//!
//!   "creates a chain of 3 ArchiveEntry rows, runs verify (PASS),
//!    tampers entry 2's content_hash, runs verify (FAIL with
//!    HashMismatch at entry 2)."
//!
//! Strategy: this test exercises the orchestrator-side producer
//! (`ArchiveChain::load` + `verify_chain`) directly, since the CLI
//! [`verify_archive`] wrapper is a thin pass-through and the cli crate is
//! binary-only (no lib target to import). The CLI's own unit tests
//! (in `src/archive_verify.rs::tests`) cover the report-shape /
//! entry-index / fail-fast semantics in-process. Together the two test
//! surfaces close the producer/consumer loop without spawning subprocess.
//!
//! Two additional scenarios (broken-parent + empty-archive) are included
//! to lock the contract surface against future regressions.

use std::path::PathBuf;

use elai_orchestrator::archive_chain::{ArchiveChain, ArchiveEntry};
use elai_orchestrator::reasoning_template::{
    compute_content_hash, ArchiveStatus, Blake3Hash, Domain, PatternKind, Predicate,
    ReasoningTemplate, TemplateBody, TemplateError, TemplateId, VerifierKind,
};
use tempfile::TempDir;

fn mk_template(body: &str) -> ReasoningTemplate {
    let body_obj = TemplateBody::try_new(body).expect("body fits");
    let content_hash = compute_content_hash(body_obj.as_str(), None);
    ReasoningTemplate {
        id: TemplateId::new_v4(),
        domain: Domain::Code,
        pattern_kind: PatternKind::Decomposition,
        template_body: body_obj,
        eligibility_predicate: Predicate::Always,
        verifier_required: VerifierKind::Compile,
        content_hash,
        created_at_ms: 0,
        parent_id: None,
        archive_status: ArchiveStatus::Frozen,
    }
}

fn build_three_entry_chain(path: &PathBuf) {
    let mut chain = ArchiveChain::new(path.clone());
    let e1 = ArchiveEntry::new_root(mk_template("entry-1-body"));
    let e1_hash = e1.self_hash;
    chain.append(e1).expect("append e1");
    let e2 = ArchiveEntry::new_child(mk_template("entry-2-body"), e1_hash);
    let e2_hash = e2.self_hash;
    chain.append(e2).expect("append e2");
    let e3 = ArchiveEntry::new_child(mk_template("entry-3-body"), e2_hash);
    chain.append(e3).expect("append e3");
}

#[test]
fn observable_effect_three_entry_chain_clean_then_tampered() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("archive.jsonl");
    build_three_entry_chain(&path);

    // Step 1: clean chain reloaded from disk -> verify_chain returns Ok.
    let clean_chain = ArchiveChain::load(path.clone()).expect("load clean");
    assert_eq!(clean_chain.len(), 3);
    clean_chain.verify_chain().expect("clean chain verifies");

    // Step 2: tamper entry-2's template_body on disk so its stored
    // self_hash no longer matches the recomputed value. This is the
    // exact lobotomy-attack vector D-8 defends against.
    let raw = std::fs::read_to_string(&path).expect("read raw");
    let mut lines: Vec<String> = raw.lines().map(String::from).collect();
    assert_eq!(lines.len(), 3, "expected 3 JSONL lines, got {}", lines.len());
    let tampered_line =
        lines[1].replace("\"template_body\":\"entry-2-body\"", "\"template_body\":\"TAMPERED\"");
    assert_ne!(
        tampered_line, lines[1],
        "expected substitution to find the template_body field"
    );
    lines[1] = tampered_line;
    std::fs::write(&path, lines.join("\n") + "\n").expect("write tampered");

    // Step 3: tampered chain reload + verify -> HashMismatch.
    // D-1's verify_chain returns the first error and short-circuits;
    // this is the same primitive the CLI dispatches.
    let tampered_chain = ArchiveChain::load(path.clone()).expect("load tampered");
    assert_eq!(tampered_chain.len(), 3);
    match tampered_chain.verify_chain() {
        Err(TemplateError::HashMismatch { template_id }) => {
            assert_eq!(
                template_id.len(),
                32,
                "expected 32-char hex template_id, got len={}",
                template_id.len()
            );
        }
        other => panic!("expected HashMismatch, got {other:?}"),
    }
}

#[test]
fn observable_effect_broken_parent_detected() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("archive.jsonl");

    let mut chain = ArchiveChain::new(path.clone());
    let root = ArchiveEntry::new_root(mk_template("root"));
    chain.append(root).expect("append root");

    // Append a child pointing to a NEVER-SEEN parent hash.
    let fake_parent = Blake3Hash([0xCDu8; 32]);
    let orphan = ArchiveEntry::new_child(mk_template("orphan"), fake_parent);
    chain.append(orphan).expect("append orphan");

    let reloaded = ArchiveChain::load(path).expect("load");
    match reloaded.verify_chain() {
        Err(TemplateError::ParentNotFound { parent_id }) => {
            assert_eq!(parent_id.len(), 32, "expected 32-char hex template_id");
        }
        other => panic!("expected ParentNotFound, got {other:?}"),
    }
}

#[test]
fn observable_effect_empty_archive_passes() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("never-created.jsonl");
    let chain = ArchiveChain::load(path).expect("load missing");
    assert!(chain.is_empty());
    chain.verify_chain().expect("empty chain trivially verifies");
}

#[test]
fn observable_effect_three_entries_persist_and_reload() {
    // Belt-and-suspenders: also assert that append-then-reload round-trips
    // all three entries and their self_hashes survive serialisation.
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("rt.jsonl");
    build_three_entry_chain(&path);

    let reloaded = ArchiveChain::load(path).expect("load");
    assert_eq!(reloaded.len(), 3);
    let entries = reloaded.entries();

    // Each parent_hash references the prior entry's self_hash.
    assert!(entries[0].parent_hash.is_none(), "entry 0 is the root");
    assert_eq!(entries[1].parent_hash, Some(entries[0].self_hash));
    assert_eq!(entries[2].parent_hash, Some(entries[1].self_hash));

    // The blake3 hashes are 32 bytes each -> 64 hex chars.
    for e in entries {
        assert_eq!(e.self_hash.as_hex().len(), 64);
    }

    reloaded.verify_chain().expect("clean round-trip verifies");
}
