//! J-26 S5 (slice S4): `elai eval-corpus-ingest --pack <dir> [--root <corpus-root>]`
//! — load a TaskPack's `data/public/task.toml` (+ optional `task.md` overlay),
//! build a [`NewTaskInput`], and call [`elai_eval_corpus::ingest::ingest_task`].
//!
//! Exit 0 on Accepted, 1 on Rejected / Err. Matches the migrate_held_out bin
//! pattern: `IngestConfig::default_const()` with `current_embedding: None` and
//! empty `prior_signatures` (the 13-gram filter still runs; the cosine leg is a
//! no-op when no embeddings are supplied, so no network fetch is triggered).

use std::path::PathBuf;

use elai_eval_corpus::ingest::{
    ingest_task, IngestConfig, IngestResult, NewTaskInput,
};
use elai_eval_corpus::task_schema::Task;

/// Default corpus root when `--root` is not supplied. Mirrors the
/// migrate_held_out bin default (`.elai_cc` under the current working
/// directory) so shards land at `{root}/held-out/{family}/{date}/{id}.toml`.
fn default_corpus_root() -> Result<PathBuf, anyhow::Error> {
    Ok(std::env::current_dir()?.join(".elai_cc"))
}

/// Run the `eval-corpus-ingest` subcommand. `pack` is the TaskPack directory
/// (must contain `data/public/task.toml`); `root` is the corpus root that
/// holds the `held-out/` shard tree.
pub fn run(pack: PathBuf, root: Option<PathBuf>) -> Result<(), anyhow::Error> {
    let corpus_root = match root {
        Some(r) => r,
        None => default_corpus_root()?,
    };

    let task_toml = pack.join("data").join("public").join("task.toml");
    if !task_toml.is_file() {
        anyhow::bail!(
            "eval-corpus-ingest: missing required pack file {} (pack = {})",
            task_toml.display(),
            pack.display()
        );
    }

    let mut task = Task::load_from_toml(&task_toml).map_err(|e| {
        anyhow::anyhow!("eval-corpus-ingest: task.toml load failed: {e}")
    })?;

    if task.tests.is_empty() {
        anyhow::bail!(
            "eval-corpus-ingest: task.toml has empty `tests` (id={}); refusing to ingest",
            task.id
        );
    }
    if task.cutoffs.is_empty() {
        anyhow::bail!(
            "eval-corpus-ingest: task.toml has empty `cutoffs` (id={}); refusing to ingest",
            task.id
        );
    }

    let task_md = pack.join("data").join("public").join("task.md");
    if task_md.is_file() {
        let body = std::fs::read_to_string(&task_md).map_err(|e| {
            anyhow::anyhow!(
                "eval-corpus-ingest: task.md read failed at {}: {e}",
                task_md.display()
            )
        })?;
        task.problem = body;
    }

    let input = NewTaskInput {
        task,
        prior_signatures: Vec::new(),
        current_embedding: None,
    };

    let cfg = IngestConfig::default_const();

    match ingest_task(&corpus_root, &input, &cfg) {
        Ok(IngestResult::Accepted { path }) => {
            println!(
                "eval-corpus-ingest: ACCEPTED id={} path={}",
                input.task.id,
                path.display()
            );
            Ok(())
        }
        Ok(IngestResult::Rejected { reason }) => {
            eprintln!(
                "eval-corpus-ingest: REJECTED id={} reason={:?}",
                input.task.id, reason
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!(
                "eval-corpus-ingest: ERR id={} error={}",
                input.task.id, e
            );
            std::process::exit(1);
        }
    }
}
