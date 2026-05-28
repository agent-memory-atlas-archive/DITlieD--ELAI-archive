//! E-54 storage layout integration test -- ingest writes to per-family
//! shard, appends `_index/{family}.jsonl` summary, never writes to a flat
//! held-out/ layout.

use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use tempfile::tempdir;

use elai_eval_corpus::ingest::{ingest_task, IngestConfig, IngestResult, NewTaskInput};
use elai_eval_corpus::storage::{index_path, shard_path};
use elai_eval_corpus::task_schema::{ExecutableTest, Provenance, Task, TestKind};
use elai_eval_corpus::TaskFamily;

fn fixture(id: &str, body: &str) -> Task {
    let mut cutoffs = HashMap::new();
    cutoffs.insert("m".to_string(), "2024-10".to_string());
    Task {
        id: id.to_string(),
        family: TaskFamily::CodeFix,
        created_at: Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
        cutoffs,
        problem: body.to_string(),
        reference_solution: "r".to_string(),
        tests: vec![ExecutableTest {
            input: "i".to_string(),
            expected_output: "o".to_string(),
            kind: TestKind::UnitTest,
        }],
        provenance: Provenance {
            generator: "g".to_string(),
            prompt_id: None,
            date: Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
        },
        framing_pair: None,
        perturbed_pair: None,
        embedding_hash: "deadbeefcafe1234".to_string(),
        ngram_hash: "1234deadbeefcafe".to_string(),
    }
}

#[test]
fn ingest_writes_to_per_family_shard_and_appends_index() {
    let tmp = tempdir().unwrap();
    let task = fixture(
        "shard-1",
        "a long enough problem body that the 13-gram filter has tokens \
        to slide a window across, no fewer than the threshold count of \
        thirteen plus more for good measure to be safe across normalize",
    );
    let input = NewTaskInput {
        task: task.clone(),
        prior_signatures: Vec::new(),
        current_embedding: None,
    };
    let cfg = IngestConfig::default();
    let r = ingest_task(tmp.path(), &input, &cfg).unwrap();
    match r {
        IngestResult::Accepted { path } => {
            // Path uses snake-case family ('code_fix') + YYYY-MM-DD shard.
            let expected =
                shard_path(tmp.path(), TaskFamily::CodeFix, task.created_at, &task.id);
            assert_eq!(path, expected);
            assert!(path.exists());
        }
        IngestResult::Rejected { reason } => {
            panic!("expected acceptance, got rejection {reason:?}")
        }
    }
    // Index file landed.
    let idx = index_path(tmp.path(), TaskFamily::CodeFix);
    assert!(idx.exists());
    let body = std::fs::read_to_string(&idx).unwrap();
    assert!(body.contains("shard-1"));
}

#[test]
fn ingest_rejects_paraphrase_via_ngram_overlap_with_prior() {
    // Same body as prior -> 13-gram overlap with K=4 consecutive grams.
    let body = "Identical paraphrase body matches every contiguous gram \
        even after the normalization step strips punctuation and casing.";
    let cfg = IngestConfig::default();
    let prior_sig = elai_eval_corpus::ngram_filter(body, cfg.ngram_n);
    let tmp = tempdir().unwrap();
    let task = fixture("ngram-2", body);
    let input = NewTaskInput {
        task,
        prior_signatures: vec![elai_eval_corpus::ingest::PriorTaskSignature {
            task_id: "prior-task".to_string(),
            ngram_signature: prior_sig,
            embedding: None,
        }],
        current_embedding: None,
    };
    let r = ingest_task(tmp.path(), &input, &cfg).unwrap();
    assert!(matches!(r, IngestResult::Rejected { .. }));
}
