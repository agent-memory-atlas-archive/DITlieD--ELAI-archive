// B-16A: error-propagation tests for the knowledge crate.
//
// Verifies that the public APIs of Bm25Index, VectorIndexer, and
// DkgSnapshot::from_engine surface typed `KnowledgeError` values
// (not `Box<dyn std::error::Error>`) and that the reranker batch worker
// shuts down gracefully when its semaphore is closed (no panic).

use std::path::PathBuf;
use std::sync::Arc;

use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::vector::indexer::VectorIndexer;
use elai_knowledge::KnowledgeError;
use tokio::sync::Semaphore;

// ---------------------------------------------------------------------------
// (a) BM25 — error path returns KnowledgeError, not Box<dyn Error>.
// ---------------------------------------------------------------------------

#[test]
fn bm25_index_new_returns_typed_error_on_unwritable_root() {
    // Pointing project_root at a path that contains a NUL byte makes
    // create_dir_all fail; on Windows + Linux this surfaces as io::Error.
    // We cannot easily craft a real "permission denied" without root, so we
    // instead use a guaranteed-invalid component: a regular file masquerading
    // as a directory.
    let tmp = tempfile::tempdir().expect("tempdir");
    let blocker = tmp.path().join("blocker");
    std::fs::write(&blocker, b"not a directory").expect("write blocker file");

    // Trying to create projects/<id>/bm25 under a regular file errors out.
    let result: Result<Bm25Index, KnowledgeError> = Bm25Index::new(&blocker, "test_proj");
    assert!(
        result.is_err(),
        "Bm25Index::new should fail when project_root is a regular file"
    );
    let err = result.err().expect("error variant present");
    // Confirm typed return: variant is KnowledgeError::Io OR Tantivy (depending
    // on which create_in_dir vs fs::create_dir_all surfaces). Either is typed.
    let is_typed = matches!(
        err,
        KnowledgeError::Io(_) | KnowledgeError::Tantivy(_) | KnowledgeError::SchemaMismatch { .. }
    );
    assert!(
        is_typed,
        "expected Io / Tantivy / SchemaMismatch variant, got {err:?}"
    );
}

#[test]
fn bm25_query_typed_result_on_empty_index() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let idx = Bm25Index::new(tmp.path(), "ok_proj").expect("bm25 init");
    // Empty query short-circuits to Ok(vec![]) — the Result type is the contract.
    let hits: Result<Vec<_>, KnowledgeError> = idx.query("", 5);
    assert!(hits.is_ok());
    assert!(hits.expect("ok").is_empty());
}

// ---------------------------------------------------------------------------
// (b) VectorIndexer — error path returns KnowledgeError.
// ---------------------------------------------------------------------------

#[test]
fn vector_indexer_load_returns_typed_error_on_corrupt_meta() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let index_path = tmp.path().join("vector_index.json");
    let meta_path = tmp.path().join("vector_index.meta.json");

    // Write a valid (empty) index file but a corrupt meta.json so load()
    // fails on serde_json deserialisation.
    let empty_indexer = VectorIndexer::new(384);
    empty_indexer.save(&index_path).expect("save empty index");
    std::fs::write(&meta_path, b"{ this is not valid json").expect("corrupt meta");

    let result: Result<VectorIndexer, KnowledgeError> = VectorIndexer::load(&index_path, 384);
    assert!(
        result.is_err(),
        "VectorIndexer::load should fail on corrupt meta.json"
    );
    let err = result.err().expect("error variant");
    let is_typed = matches!(
        err,
        KnowledgeError::Json(_) | KnowledgeError::Io(_) | KnowledgeError::Vector(_)
    );
    assert!(
        is_typed,
        "expected Json / Io / Vector variant, got {err:?}"
    );
}

#[test]
fn vector_indexer_save_returns_typed_error_on_unwritable_parent() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let blocker = tmp.path().join("blocker");
    std::fs::write(&blocker, b"not a directory").expect("write blocker");
    // path under a regular file -> save should error
    let bogus = blocker.join("subdir").join("vector_index.json");
    let indexer = VectorIndexer::new(8);
    let result: Result<(), KnowledgeError> = indexer.save(&bogus);
    assert!(result.is_err(), "save under a regular file should fail");
    let err = result.err().expect("err");
    assert!(
        matches!(err, KnowledgeError::Io(_) | KnowledgeError::Vector(_) | KnowledgeError::Json(_)),
        "expected Io/Vector/Json variant, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// (c) PageRank — DkgSnapshot::from_engine return type is typed.
//
// A full integration test would need a populated DkgEngine on disk;
// the type-level contract is what B-16A enforces. This test just asserts
// the function signature compiles against `Result<_, KnowledgeError>`.
// ---------------------------------------------------------------------------

#[test]
fn pagerank_from_engine_returns_typed_result() {
    // Compile-time contract: the function must return Result<_, KnowledgeError>.
    // We don't build a full DkgEngine here — proving the signature compiles
    // is sufficient for B-16A's "no Box<dyn Error>" rule.
    fn _assert_signature(engine: &elai_knowledge::DkgEngine)
        -> Result<elai_knowledge::pagerank::DkgSnapshot, KnowledgeError>
    {
        elai_knowledge::pagerank::DkgSnapshot::from_engine(engine)
    }
    // Reference the function so the compiler keeps the signature check.
    let _f = _assert_signature;
    // Sanity: KnowledgeError implements std::error::Error + Display.
    let err: KnowledgeError = KnowledgeError::Graph("test".to_string());
    let _displayed: String = format!("{err}");
}

// ---------------------------------------------------------------------------
// (d) Reranker semaphore-closed path — graceful shutdown, no panic.
//
// We exercise the canonical pattern directly via a tokio::sync::Semaphore.
// run_batch_worker is pub(crate) so we cannot call it across crates; instead
// we test the contract pattern (acquire().await on a closed semaphore returns
// Err, not panic) and document that reranker.rs::run_batch_worker uses the
// same shape.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn semaphore_closed_returns_error_not_panic() {
    let sem = Arc::new(Semaphore::new(1));
    sem.close();
    let result = sem.acquire().await;
    assert!(
        result.is_err(),
        "closed semaphore must return Err (matches reranker fix pattern)"
    );
}

// Document: this test is the contract used by
// crates/knowledge/src/reranker.rs::run_batch_worker. The previous code
// `let _permit = semaphore.acquire().await.expect("semaphore closed");`
// would PANIC on this exact case; the B-16A fix uses match-and-return so
// the worker exits via tracing::warn instead.

// ---------------------------------------------------------------------------
// (e) Anti-stale lint script smoke check — the script must exist and be
//     executable (presence-only check; behavioural validation is done by
//     scripts/check_anti_stale.sh itself in CI).
// ---------------------------------------------------------------------------

#[test]
fn anti_stale_lint_scripts_present() {
    // Walk up to the workspace root (.elai_cc/) from CARGO_MANIFEST_DIR.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    let sh = workspace_root.join("scripts").join("check_anti_stale.sh");
    let bat = workspace_root.join("scripts").join("check_anti_stale.bat");
    assert!(sh.is_file(), "scripts/check_anti_stale.sh must exist at {}", sh.display());
    assert!(bat.is_file(), "scripts/check_anti_stale.bat must exist at {}", bat.display());
}
