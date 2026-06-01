//! E-54 migration bin: convert legacy flat `held-out/{YYYY-MM}/*.jsonl` rows
//! into per-family TOML shards.
//!
//! Idempotent. Reads each JSONL row, attempts to construct a [`Task`] via the
//! eval-corpus schema, runs validate() + 13-gram + cosine pre-checks, then
//! either writes the per-family shard + appends the index row OR writes the
//! raw body to `held-out/_quarantine/{family}/` with a `.reason` sibling.
//!
//! Per workflow/reasoning/E-54-decisions.md DECISION-3: most legacy rows
//! will quarantine on first run because they lack `tests` + `cutoffs` +
//! `family`. Expected behaviour; documented.

use std::path::PathBuf;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use elai_eval_corpus::ingest::{ingest_task, IngestConfig, NewTaskInput, IngestResult};
use elai_eval_corpus::storage::{quarantine_raw, StorageError};
use elai_eval_corpus::task_schema::{Task, TaskSchemaError};
use elai_eval_corpus::TaskFamily;

fn main() -> Result<()> {
    // Resolve the root: defaults to `.elai_cc/` under the current working
    // directory; override via the first CLI arg.
    let mut args = std::env::args().skip(1);
    let root = match args.next() {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir()?.join(".elai_cc"),
    };
    println!("[E-54 migrate] root = {}", root.display());

    // Walk legacy held-out/{YYYY-MM}/*.jsonl. If the directory doesn't
    // exist, exit clean (no legacy data to migrate).
    let held_out = root.join("held-out");
    if !held_out.exists() {
        println!("[E-54 migrate] no held-out directory; nothing to migrate");
        return Ok(());
    }
    let mut total = 0usize;
    let mut quarantined = 0usize;
    let mut accepted = 0usize;
    let cfg = IngestConfig::default();
    for entry in std::fs::read_dir(&held_out)? {
        let entry = entry?;
        let path = entry.path();
        let fname = entry
            .file_name()
            .to_string_lossy()
            .to_string();
        // Skip the new per-family directories + the _index / _quarantine /
        // _audit / from-prod sub-dirs.
        if fname.starts_with('_')
            || fname == "from-prod"
            || matches!(
                fname.as_str(),
                "code_fix" | "math_proof" | "prose_reason" | "planning_decomposition" | "tool_use"
            )
        {
            continue;
        }
        if !entry.file_type()?.is_dir() {
            continue;
        }
        // Legacy YYYY-MM directories containing flat jsonl seeds.
        for sub in std::fs::read_dir(&path)? {
            let sub = sub?;
            let sp = sub.path();
            if !sp
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("jsonl"))
                .unwrap_or(false)
            {
                continue;
            }
            let body = std::fs::read_to_string(&sp)?;
            for (lineno, line) in body.lines().enumerate() {
                total += 1;
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                // Try parsing as a Task TOML inline (legacy may have stuffed
                // a TOML body into a JSONL line; if not, quarantine).
                let id_guess = format!("{}_{lineno}", sp.file_stem().and_then(|s| s.to_str()).unwrap_or("legacy"));
                match toml::from_str::<Task>(trimmed) {
                    Ok(task) => {
                        // Force created_at if epoch-0 (legacy may not set):
                        let task = if task.created_at == Utc.timestamp_opt(0, 0).single().unwrap_or(Utc::now()) {
                            let mut t = task;
                            t.created_at = Utc::now();
                            t
                        } else {
                            task
                        };
                        let input = NewTaskInput {
                            task,
                            prior_signatures: Vec::new(),
                            current_embedding: None,
                        };
                        match ingest_task(&root, &input, &cfg) {
                            Ok(IngestResult::Accepted { .. }) => accepted += 1,
                            Ok(IngestResult::Rejected { reason }) => {
                                quarantine_raw(
                                    &root,
                                    TaskFamily::CodeFix,
                                    &id_guess,
                                    trimmed,
                                    &format!("rejected: {reason:?}"),
                                )?;
                                quarantined += 1;
                            }
                            Err(e) => {
                                quarantine_raw(
                                    &root,
                                    TaskFamily::CodeFix,
                                    &id_guess,
                                    trimmed,
                                    &format!("ingest error: {e}"),
                                )?;
                                quarantined += 1;
                            }
                        }
                    }
                    Err(e) => {
                        // Schema-incompatible legacy row -- quarantine.
                        quarantine_raw(
                            &root,
                            TaskFamily::CodeFix,
                            &id_guess,
                            trimmed,
                            &format!("legacy schema mismatch: {e}"),
                        )?;
                        quarantined += 1;
                    }
                }
            }
        }
    }
    println!(
        "[E-54 migrate] total={total} accepted={accepted} quarantined={quarantined}"
    );
    Ok(())
}

// Silence dead-code on the imports when this bin builds with only some
// re-exports used (the binary is invoked manually -- one-shot).
#[allow(dead_code)]
fn _unused_imports_marker() {
    let _ = std::mem::size_of::<TaskSchemaError>();
    let _ = std::mem::size_of::<StorageError>();
}
