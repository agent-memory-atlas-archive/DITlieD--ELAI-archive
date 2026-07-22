//! J-22 T3 contract — writer-isolation: the ONLY production runtime commit of
//! an `ssb` row is the single-writer path in `ssb/store.rs`.
//!
//! Leg A: structural allowlist lock — exactly 2 `INSERT INTO ssb (` sites in
//! store.rs, one in `insert_session_archived_row` (migration-time exempt), one
//! in `execute_write` (the sole runtime commit). Adding a third INSERT site
//! anywhere else in store.rs turns this RED.
//!
//! Leg B: behavioral — `Ssb::write_payload` produces gapless monotonic seqs
//! and `chain_prev_hash()` is `Some(_)` after writes (IS-1: chain advances
//! only inside write_loop after execute_write commits). Deleting the
//! write_loop>execute_write call or skipping chain advance after commit
//! breaks gapless seq / chain_prev_hash.
//!
//! UNCOVERED: the read-pool / second-Connection INSERT bypass documented in
//! ssb/tests.rs conflict tests is an architectural hole this contract does
//! NOT close. It is named here so it is not silently assumed covered.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{CheckpointPayload, Ssb, SsbConfig, SsbPayload};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper: read a source file relative to the crates/ dir
// ---------------------------------------------------------------------------

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// Leg A — allowlist lock: exactly 2 INSERT INTO ssb ( sites in store.rs
// ---------------------------------------------------------------------------

/// Extract function-level blocks from Rust source: returns a Vec of
/// (fn_name, body_text) for every `fn <name>` ... `}` top-level or nested
/// function. The body includes the signature line through the closing brace.
fn extract_fn_blocks(src: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(name) = extract_fn_name(line) {
            let mut depth: i32 = 0;
            let mut found_open = false;
            let start = i;
            let mut end = i;
            for j in i..lines.len() {
                for ch in lines[j].chars() {
                    if ch == '{' {
                        depth += 1;
                        found_open = true;
                    } else if ch == '}' {
                        depth -= 1;
                    }
                }
                if found_open && depth == 0 {
                    end = j;
                    break;
                }
            }
            let body: String = lines[start..=end].join("\n");
            blocks.push((name, body));
            i = end + 1;
        } else {
            i += 1;
        }
    }
    blocks
}

fn extract_fn_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let patterns = ["pub async fn ", "async fn ", "pub fn ", "fn "];
    for pat in &patterns {
        if let Some(rest) = trimmed.strip_prefix(pat) {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

const INSERT_NEEDLE: &str = "INSERT INTO ssb (";

#[test]
fn leg_a_exactly_two_insert_sites_in_store_rs() {
    let src = read_src("orchestrator/src/ssb/store.rs");
    let total = src.matches(INSERT_NEEDLE).count();
    assert_eq!(
        total, 2,
        "store.rs must have exactly 2 `INSERT INTO ssb (` sites, found {total}"
    );
}

#[test]
fn leg_a_insert_sites_in_named_functions_only() {
    let src = read_src("orchestrator/src/ssb/store.rs");
    let blocks = extract_fn_blocks(&src);
    let mut found_archived = false;
    let mut found_execute_write = false;
    let mut found_elsewhere = false;

    for (name, body) in &blocks {
        let count = body.matches(INSERT_NEEDLE).count();
        if count == 0 {
            continue;
        }
        match name.as_str() {
            "insert_session_archived_row" => {
                assert_eq!(
                    count, 1,
                    "insert_session_archived_row must have exactly 1 INSERT site, found {count}"
                );
                found_archived = true;
            }
            "execute_write" => {
                assert_eq!(
                    count, 1,
                    "execute_write must have exactly 1 INSERT site, found {count}"
                );
                found_execute_write = true;
            }
            _ => {
                found_elsewhere = true;
                eprintln!(
                    "INSERT INTO ssb ( found in unexpected fn `{name}` with {count} occurrence(s)"
                );
            }
        }
    }

    assert!(found_archived, "insert_session_archived_row must contain an INSERT site");
    assert!(found_execute_write, "execute_write must contain an INSERT site");
    assert!(
        !found_elsewhere,
        "INSERT INTO ssb ( must not appear in any other function in store.rs"
    );
}

// ---------------------------------------------------------------------------
// Leg B — behavioral: gapless seqs + chain_prev_hash Some after writes
// ---------------------------------------------------------------------------

fn fresh_db() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("ssb.db");
    (dir, path)
}

async fn open_writer(path: &std::path::Path) -> Ssb {
    let cfg = SsbConfig {
        db_path: path.to_path_buf(),
        pool_size: Some(2),
        project_id: "j22-contract".into(),
        session_id: "j22-test".into(),
    };
    Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init")
}

fn checkpoint_payload() -> SsbPayload {
    SsbPayload::Checkpoint(CheckpointPayload {
        agent_state_hash: "abc123".to_string(),
        grc_history_hash: "def456".to_string(),
        agent_id: "j22-test-agent".to_string(),
        step_index: 0,
        file_sizes: vec![],
    })
}

/// Leg B: write 5 rows via the production `Ssb::write_payload` API and assert
/// returned seqs are gapless monotonic (1..=5) and `chain_prev_hash()` is
/// `Some(_)` after the writes (IS-1: chain advances inside write_loop after
/// execute_write commits).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn leg_b_gapless_seqs_and_chain_advances() {
    let (_d, path) = fresh_db();
    let mut writer = open_writer(&path).await;

    let mut seqs = Vec::new();
    for i in 0..5_u64 {
        let mut payload = checkpoint_payload();
        if let SsbPayload::Checkpoint(ref mut cp) = payload {
            cp.step_index = i;
        }
        let seq = writer
            .write_payload("j22-test-agent", 0, payload, vec![])
            .await
            .expect("write_payload");
        seqs.push(seq);
    }

    // Seqs must be gapless monotonic starting at 1.
    for (idx, seq) in seqs.iter().enumerate() {
        let expected = (idx + 1) as i64;
        assert_eq!(
            *seq, expected,
            "seq at index {idx} must be {expected}, got {seq}"
        );
    }

    // chain_prev_hash must be Some after writes (IS-1).
    let prev_hash = writer.chain_prev_hash();
    assert!(
        prev_hash.is_some(),
        "chain_prev_hash must be Some(_) after writes (IS-1: chain advances inside write_loop)"
    );

    writer.shutdown();
}
