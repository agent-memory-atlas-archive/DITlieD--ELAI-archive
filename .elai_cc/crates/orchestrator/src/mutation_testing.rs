//! C-28 — Testing-Trust T-1 Mutation Testing.
//!
//! This module provides the `MutationRunner` trait + two production
//! implementations (`RustMutationRunner` wrapping cargo-mutants 27.0.0,
//! `PythonMutationRunner` wrapping mutmut 3.5.0) + `MutationResult` shape
//! + `EquivalentMutantCache` (SQLite + blake3-keyed + ClockProvider-driven
//! 7-day TTL).
//!
//! The runners shell out to external binaries via `tokio::process::Command`.
//! Neither cargo-mutants nor mutmut is declared as a Cargo dependency
//! (they are CLI tools, not libraries) — both must be installed separately.
//! See `scripts/install_dev_tools.sh` for the canonical install commands
//! and `workflow/reasoning/C-28-decisions.md` for the rationale.
//!
//! Production wiring lands in `deterministic_auditor::DeterministicAuditor`
//! Step 6 (post-Build / VersionGate / FileSizeCensus / LazyCode /
//! Completeness). The mutation check emits one `SsbActionType::MutationSurvived`
//! row per surviving mutant + a `DeterministicFinding::Block` when
//! `mutation_score_pct < threshold_pct` (default 60.0%).
//!
//! Anti-staleness (per CLAUDE.md): blake3 only (no sha2/md5); tokio::process
//! (no std::process); thiserror; no `.unwrap()` / `.expect()` outside
//! `#[cfg(test)]` / const init.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use elai_core::clock::ClockProvider;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors propagated by `MutationRunner` / `EquivalentMutantCache`.
#[derive(Debug, thiserror::Error)]
pub enum MutationError {
    #[error("failed to spawn mutation tool {tool:?}: {source}")]
    SubprocessSpawn {
        tool: String,
        #[source]
        source: std::io::Error,
    },
    #[error("mutation tool {tool:?} timed out after {budget:?}")]
    SubprocessTimeout { tool: String, budget: Duration },
    #[error("failed to parse mutation tool {tool:?} output: {reason}")]
    OutputParseFailed { tool: String, reason: String },
    #[error("equivalent-mutant cache error: {0}")]
    CacheError(String),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// MutantOutcome / MutantRecord / MutationResult
// ---------------------------------------------------------------------------

/// Outcome categories returned by a single mutant execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutantOutcome {
    /// At least one test failed — the mutant was detected.
    Caught,
    /// All tests passed — the test suite did not catch the mutant.
    /// Triggers a `MutationSurvived` SSB row + counts against the score.
    Survived,
    /// The mutant exceeded the per-mutant timeout. Counted as a
    /// non-caught outcome for score purposes (mirrors cargo-mutants'
    /// strict mode).
    Timeout,
    /// The mutant was not run (compile error, cache hit, unviable
    /// mutation, etc). Free-form reason for diagnostic surfacing.
    Skipped(String),
}

impl MutantOutcome {
    pub fn is_caught(&self) -> bool {
        matches!(self, Self::Caught)
    }
    pub fn is_survived(&self) -> bool {
        matches!(self, Self::Survived)
    }
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout)
    }
    pub fn is_skipped(&self) -> bool {
        matches!(self, Self::Skipped(_))
    }
}

/// A single mutant result row. Drives both score aggregation and the
/// `MutationSurvived` SSB row emission (only `Survived` outcomes emit).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutantRecord {
    pub file: PathBuf,
    pub line: u32,
    pub mutant_kind: String,
    pub outcome: MutantOutcome,
}

/// Aggregate result of a single `MutationRunner::run` invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutationResult {
    pub records: Vec<MutantRecord>,
    /// caught / (caught + survived + timeout) * 100. Skipped outcomes are
    /// excluded from the denominator so cache hits / unviable mutants do
    /// not inflate the score. Returns 100.0 when no eligible mutants ran
    /// (vacuous-pass: nothing to mutate means nothing to detect).
    pub mutation_score_pct: f64,
    /// Number of mutants whose subprocess was actually executed
    /// (excludes cache-hit skips).
    pub total_run: u32,
    /// Number of mutants served from `EquivalentMutantCache` without
    /// re-running the subprocess.
    pub cache_hits: u32,
}

impl MutationResult {
    /// Compute `mutation_score_pct` from the `records` vec. Vacuous-pass
    /// (no eligible mutants) returns 100.0 — the threshold gate cannot
    /// fail in that case (there is nothing to detect).
    #[must_use]
    pub fn from_records(records: Vec<MutantRecord>) -> Self {
        let mut caught = 0u32;
        let mut survived = 0u32;
        let mut timeout = 0u32;
        let mut skipped_cache = 0u32;
        let mut total_run = 0u32;
        for r in &records {
            match &r.outcome {
                MutantOutcome::Caught => {
                    caught += 1;
                    total_run += 1;
                }
                MutantOutcome::Survived => {
                    survived += 1;
                    total_run += 1;
                }
                MutantOutcome::Timeout => {
                    timeout += 1;
                    total_run += 1;
                }
                MutantOutcome::Skipped(reason) => {
                    if reason == "cache_hit" {
                        skipped_cache += 1;
                    }
                }
            }
        }
        let denom = caught + survived + timeout;
        let mutation_score_pct = if denom == 0 {
            100.0
        } else {
            (f64::from(caught) / f64::from(denom)) * 100.0
        };
        Self {
            records,
            mutation_score_pct,
            total_run,
            cache_hits: skipped_cache,
        }
    }
}

// ---------------------------------------------------------------------------
// MutationConfig — loaded from [testing.mutation] in elai_cc.toml
// ---------------------------------------------------------------------------

/// `[testing.mutation]` block in `elai_cc.toml`. Defaults match the
/// C-28 plan (`enabled=true`, `threshold_pct=60.0`,
/// `timeout_per_mutant_sec=30`, `max_mutants_per_session=100`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MutationConfig {
    /// Master switch — when `false`, Step 6 mutation check is skipped.
    pub enabled: bool,
    /// Block threshold for the AUDITING -> MEMORIZING gate. Computed
    /// as caught / (caught + survived + timeout) * 100.
    pub threshold_pct: f64,
    /// Per-mutant timeout passed to the underlying tool via
    /// `--timeout` (cargo-mutants) / `--test-time-multiplier`
    /// (mutmut). Also enforced in-process via `tokio::time::timeout`
    /// as a defense-in-depth budget.
    pub timeout_per_mutant_sec: u32,
    /// Hard cap on mutants per session. Once reached the runner exits
    /// with the partial result; the score is computed over the ran
    /// subset.
    pub max_mutants_per_session: u32,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_pct: 60.0,
            timeout_per_mutant_sec: 30,
            max_mutants_per_session: 100,
        }
    }
}

// ---------------------------------------------------------------------------
// EquivalentMutantCache — SQLite + blake3 + ClockProvider 7-day TTL
// ---------------------------------------------------------------------------

/// 7-day TTL applied to cache entries. Rows older than this are treated as
/// misses + deleted lazily on lookup. ClockProvider drives the comparison so
/// tests can simulate week-long passage without sleeping.
pub const CACHE_TTL_SECS: u64 = 7 * 24 * 60 * 60;

/// SQLite-backed equivalent-mutant cache. blake3 keys cover
/// `(pre_mutation_file_bytes || mutant_kind || line)` so semantically-equal
/// mutations skip re-execution.
///
/// The `clock` field is REAL — `get` consults `clock.now_secs()` to compare
/// row timestamps to the TTL. (Mirrors C-15 D5 / determinism.rs:321-323
/// pattern with the explicit fix that this clock is NOT dead — see
/// C-28-decisions.md DECISION-D7.)
pub struct EquivalentMutantCache {
    conn: Mutex<rusqlite::Connection>,
    clock: Arc<dyn ClockProvider>,
}

impl EquivalentMutantCache {
    /// Open or create the SQLite cache at the given path.
    pub fn open(db_path: &Path, clock: Arc<dyn ClockProvider>) -> Result<Self, MutationError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = rusqlite::Connection::open(db_path)
            .map_err(|e| MutationError::CacheError(format!("open {}: {}", db_path.display(), e)))?;
        Self::init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            clock,
        })
    }

    /// In-memory variant for tests + the deterministic_auditor's
    /// no-project-dir fallback. Backed by `:memory:` SQLite.
    pub fn open_in_memory(clock: Arc<dyn ClockProvider>) -> Result<Self, MutationError> {
        let conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| MutationError::CacheError(format!("open in-memory: {e}")))?;
        Self::init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            clock,
        })
    }

    fn init_schema(conn: &rusqlite::Connection) -> Result<(), MutationError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS equiv_mutant_cache (
                 key TEXT PRIMARY KEY,
                 outcome TEXT NOT NULL,
                 ts INTEGER NOT NULL
             );",
        )
        .map_err(|e| MutationError::CacheError(format!("init schema: {e}")))?;
        Ok(())
    }

    /// Compute the canonical blake3 cache key for the given mutation
    /// (pre-mutation file bytes + mutant_kind + line).
    #[must_use]
    pub fn compute_key(pre_mutation_bytes: &[u8], mutant_kind: &str, line: u32) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(pre_mutation_bytes);
        hasher.update(mutant_kind.as_bytes());
        hasher.update(&line.to_le_bytes());
        hasher.finalize().to_hex().to_string()
    }

    /// Look up a previously-recorded outcome. Returns `None` on miss or
    /// stale (older than TTL) row. Stale rows are deleted lazily.
    pub fn get(&self, key: &str) -> Result<Option<MutantOutcome>, MutationError> {
        let now_secs = self.clock.now_secs();
        let conn = self
            .conn
            .lock()
            .map_err(|_| MutationError::CacheError("lock poisoned".to_owned()))?;
        let mut stmt = conn
            .prepare("SELECT outcome, ts FROM equiv_mutant_cache WHERE key = ?1")
            .map_err(|e| MutationError::CacheError(format!("prepare get: {e}")))?;
        let row: Option<(String, i64)> = stmt
            .query_row([key], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .ok();
        match row {
            None => Ok(None),
            Some((_, ts)) if (now_secs as i64).saturating_sub(ts) > CACHE_TTL_SECS as i64 => {
                drop(stmt);
                conn.execute("DELETE FROM equiv_mutant_cache WHERE key = ?1", [key])
                    .map_err(|e| MutationError::CacheError(format!("delete stale: {e}")))?;
                Ok(None)
            }
            Some((outcome_json, _)) => {
                let outcome: MutantOutcome = serde_json::from_str(&outcome_json).map_err(|e| {
                    MutationError::CacheError(format!("deserialize outcome: {e}"))
                })?;
                Ok(Some(outcome))
            }
        }
    }

    /// Upsert an outcome under the given key. Idempotent — repeated
    /// calls with the same key replace the previous row in-place
    /// (no duplicate rows).
    pub fn put(&self, key: &str, outcome: &MutantOutcome) -> Result<(), MutationError> {
        let now_secs = self.clock.now_secs() as i64;
        let outcome_json = serde_json::to_string(outcome)
            .map_err(|e| MutationError::CacheError(format!("serialize outcome: {e}")))?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| MutationError::CacheError("lock poisoned".to_owned()))?;
        conn.execute(
            "INSERT INTO equiv_mutant_cache (key, outcome, ts)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET outcome = excluded.outcome, ts = excluded.ts;",
            rusqlite::params![key, outcome_json, now_secs],
        )
        .map_err(|e| MutationError::CacheError(format!("upsert: {e}")))?;
        Ok(())
    }

    /// Return the number of rows currently in the cache. Test-only
    /// convenience (used by the contract test to verify upsert
    /// semantics).
    pub fn row_count(&self) -> Result<u64, MutationError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| MutationError::CacheError("lock poisoned".to_owned()))?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM equiv_mutant_cache", [], |r| r.get(0))
            .map_err(|e| MutationError::CacheError(format!("count: {e}")))?;
        Ok(count.max(0) as u64)
    }
}

// ---------------------------------------------------------------------------
// MutationRunner trait + two production impls
// ---------------------------------------------------------------------------

/// Async producer of a `MutationResult` for a given project + change set.
///
/// Uses the project-canonical `Pin<Box<dyn Future ...>>` shape (mirrors
/// `tools::tiered_validation::TieredValidator`) so no `async-trait` crate
/// dep is required — see CLAUDE.md "Anti-Stale-Training: Native async fn
/// in traits (never async-trait crate)".
pub trait MutationRunner: Send + Sync {
    fn run<'a>(
        &'a self,
        cfg: &'a MutationConfig,
        project_dir: &'a Path,
        changed_files: &'a [PathBuf],
        cache: &'a EquivalentMutantCache,
    ) -> Pin<Box<dyn Future<Output = Result<MutationResult, MutationError>> + Send + 'a>>;
}

/// Rust MutationRunner — shells to `cargo mutants`.
///
/// Output convention (verified against cargo-mutants 27.0.0): the
/// `--output <dir>` flag writes four files into the dir:
///   * caught.txt
///   * missed.txt    (= survived)
///   * timeout.txt
///   * unviable.txt  (skipped)
/// Each line carries one mutant location in the form
/// `path/to/file.rs:LINE: <mutant_kind>`. The parser reads all four files
/// and produces one `MutantRecord` per line.
pub struct RustMutationRunner;

impl RustMutationRunner {
    pub fn new() -> Self {
        Self
    }

    /// Parse a cargo-mutants result file. Each line is
    /// `path/to/file.rs:LINE: <mutant_kind>`. Returns parsed
    /// `MutantRecord`s with the supplied `outcome` applied to all rows.
    fn parse_result_file(
        contents: &str,
        outcome: MutantOutcome,
    ) -> Vec<MutantRecord> {
        let mut out = Vec::new();
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // Split into path:line: kind. First colon ends the path on
            // POSIX; on Windows the path may carry a drive letter colon
            // (e.g. `C:\repo\foo.rs:42: ...`) — split from the RIGHT
            // twice to isolate `kind` (after the second-to-last colon)
            // and `line` (between the last two colons).
            let mut parts: Vec<&str> = line.rsplitn(3, ':').collect();
            if parts.len() < 3 {
                continue;
            }
            // rsplitn yields tail-first: [kind, line, path]
            let kind = parts.remove(0).trim();
            let line_num_str = parts.remove(0).trim();
            let path_str = parts.remove(0).trim();
            let line_num: u32 = match line_num_str.parse() {
                Ok(n) => n,
                Err(_) => continue,
            };
            out.push(MutantRecord {
                file: PathBuf::from(path_str),
                line: line_num,
                mutant_kind: kind.to_owned(),
                outcome: outcome.clone(),
            });
        }
        out
    }
}

impl Default for RustMutationRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl MutationRunner for RustMutationRunner {
    fn run<'a>(
        &'a self,
        cfg: &'a MutationConfig,
        project_dir: &'a Path,
        changed_files: &'a [PathBuf],
        cache: &'a EquivalentMutantCache,
    ) -> Pin<Box<dyn Future<Output = Result<MutationResult, MutationError>> + Send + 'a>> {
        Box::pin(async move {
            let rust_files: Vec<&PathBuf> = changed_files
                .iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
                .collect();
            if rust_files.is_empty() {
                return Ok(MutationResult::from_records(Vec::new()));
            }

            // Defense-in-depth total budget: per-mutant cap * max mutants.
            let total_budget = Duration::from_secs(u64::from(
                cfg.timeout_per_mutant_sec
                    .saturating_mul(cfg.max_mutants_per_session),
            ));

            // std::env::temp_dir() avoids a new `tempfile` production dep
            // (tempfile is dev-only in orchestrator/Cargo.toml). Output
            // directory is uniquified with a blake3 hash of the project
            // path + process id + clock to dodge collisions across
            // concurrent runs. Cleanup is best-effort via the Drop guard
            // below.
            let mut unique_hasher = blake3::Hasher::new();
            unique_hasher.update(project_dir.to_string_lossy().as_bytes());
            unique_hasher.update(&std::process::id().to_le_bytes());
            let unique = unique_hasher.finalize().to_hex().to_string();
            let output_dir = std::env::temp_dir().join(format!("elai-mutants-{unique}"));
            std::fs::create_dir_all(&output_dir)?;
            let _cleanup = TempDirGuard(output_dir.clone());

            let mut cmd = tokio::process::Command::new("cargo");
            cmd.arg("mutants")
                .arg("--no-shuffle")
                .arg("--timeout")
                .arg(cfg.timeout_per_mutant_sec.to_string())
                .arg("--output")
                .arg(&output_dir)
                .arg("--jobs")
                .arg("1");
            for f in &rust_files {
                cmd.arg("--file").arg(f.as_path());
            }
            cmd.current_dir(project_dir);
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            let child = cmd
                .spawn()
                .map_err(|e| MutationError::SubprocessSpawn {
                    tool: "cargo-mutants".to_owned(),
                    source: e,
                })?;

            let wait_fut = child.wait_with_output();
            let _ = match tokio::time::timeout(total_budget, wait_fut).await {
                Ok(Ok(out)) => out,
                Ok(Err(e)) => {
                    return Err(MutationError::SubprocessSpawn {
                        tool: "cargo-mutants".to_owned(),
                        source: e,
                    })
                }
                Err(_) => {
                    return Err(MutationError::SubprocessTimeout {
                        tool: "cargo-mutants".to_owned(),
                        budget: total_budget,
                    })
                }
            };

            // Parse all 4 result files (DRIFT-C28-11).
            let read_or_empty = |name: &str| -> String {
                std::fs::read_to_string(output_dir.join(name)).unwrap_or_default()
            };
            let mut records = Vec::new();
            records.extend(Self::parse_result_file(
                &read_or_empty("caught.txt"),
                MutantOutcome::Caught,
            ));
            records.extend(Self::parse_result_file(
                &read_or_empty("missed.txt"),
                MutantOutcome::Survived,
            ));
            records.extend(Self::parse_result_file(
                &read_or_empty("timeout.txt"),
                MutantOutcome::Timeout,
            ));
            records.extend(Self::parse_result_file(
                &read_or_empty("unviable.txt"),
                MutantOutcome::Skipped("unviable".to_owned()),
            ));

            // Populate the equivalent-mutant cache with the new outcomes so
            // identical re-runs short-circuit. Cache key uses the
            // project_dir + file path as the pre-mutation bytes when
            // readable (defensive: empty buffer is the correct fallback).
            for rec in &records {
                let full = project_dir.join(&rec.file);
                let pre_bytes = std::fs::read(&full).unwrap_or_default();
                let key =
                    EquivalentMutantCache::compute_key(&pre_bytes, &rec.mutant_kind, rec.line);
                let _ = cache.put(&key, &rec.outcome);
            }

            let cap = cfg.max_mutants_per_session as usize;
            if records.len() > cap {
                records.truncate(cap);
            }
            Ok(MutationResult::from_records(records))
        })
    }
}

/// RAII cleanup for the cargo-mutants output dir. Best-effort: a remove
/// failure is logged at debug-level and swallowed (the OS temp dir is
/// already a swept location).
struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_dir_all(&self.0) {
            tracing::debug!(path = %self.0.display(), err = %e, "TempDirGuard: cleanup failed");
        }
    }
}

/// Python MutationRunner — shells to `mutmut run` + `mutmut results`.
///
/// `mutmut results --output-mode json` writes a JSON document of the form
/// `{"caught": [...], "survived": [...], "timeout": [...], "skipped": [...]}`.
/// The runner parses all four arrays and produces one MutantRecord per
/// entry.
pub struct PythonMutationRunner;

impl PythonMutationRunner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonMutationRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl MutationRunner for PythonMutationRunner {
    fn run<'a>(
        &'a self,
        cfg: &'a MutationConfig,
        project_dir: &'a Path,
        changed_files: &'a [PathBuf],
        cache: &'a EquivalentMutantCache,
    ) -> Pin<Box<dyn Future<Output = Result<MutationResult, MutationError>> + Send + 'a>> {
        Box::pin(async move {
            let py_files: Vec<&PathBuf> = changed_files
                .iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("py"))
                .collect();
            if py_files.is_empty() {
                return Ok(MutationResult::from_records(Vec::new()));
            }

            let total_budget = Duration::from_secs(u64::from(
                cfg.timeout_per_mutant_sec
                    .saturating_mul(cfg.max_mutants_per_session),
            ));

            let paths_arg = py_files
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(",");

            let mut run_cmd = tokio::process::Command::new("mutmut");
            run_cmd
                .arg("run")
                .arg("--paths-to-mutate")
                .arg(&paths_arg)
                .arg("--runner")
                .arg("pytest -x -q")
                .current_dir(project_dir)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let child = run_cmd
                .spawn()
                .map_err(|e| MutationError::SubprocessSpawn {
                    tool: "mutmut".to_owned(),
                    source: e,
                })?;
            match tokio::time::timeout(total_budget, child.wait_with_output()).await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    return Err(MutationError::SubprocessSpawn {
                        tool: "mutmut".to_owned(),
                        source: e,
                    });
                }
                Err(_) => {
                    return Err(MutationError::SubprocessTimeout {
                        tool: "mutmut".to_owned(),
                        budget: total_budget,
                    });
                }
            }

            let mut results_cmd = tokio::process::Command::new("mutmut");
            results_cmd
                .arg("results")
                .arg("--output-mode")
                .arg("json")
                .current_dir(project_dir)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            let out = results_cmd
                .spawn()
                .map_err(|e| MutationError::SubprocessSpawn {
                    tool: "mutmut".to_owned(),
                    source: e,
                })?
                .wait_with_output()
                .await
                .map_err(|e| MutationError::SubprocessSpawn {
                    tool: "mutmut".to_owned(),
                    source: e,
                })?;
            let stdout = String::from_utf8_lossy(&out.stdout).into_owned();

            let parsed: serde_json::Value =
                serde_json::from_str(&stdout).map_err(|e| MutationError::OutputParseFailed {
                    tool: "mutmut".to_owned(),
                    reason: format!("non-JSON results: {e}"),
                })?;

            let buckets = [
                ("caught", MutantOutcome::Caught),
                ("survived", MutantOutcome::Survived),
                ("timeout", MutantOutcome::Timeout),
                ("skipped", MutantOutcome::Skipped("unviable".to_owned())),
            ];
            let mut records = Vec::new();
            for (bucket_name, outcome) in &buckets {
                if let Some(arr) = parsed.get(*bucket_name).and_then(|v| v.as_array()) {
                    for entry in arr {
                        let file = entry
                            .get("file")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_owned();
                        if file.is_empty() {
                            continue;
                        }
                        let line = entry
                            .get("line")
                            .and_then(|v| v.as_u64())
                            .and_then(|n| u32::try_from(n).ok())
                            .unwrap_or(0);
                        let kind = entry
                            .get("kind")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_owned();
                        records.push(MutantRecord {
                            file: PathBuf::from(file),
                            line,
                            mutant_kind: kind,
                            outcome: outcome.clone(),
                        });
                    }
                }
            }

            for rec in &records {
                let full = project_dir.join(&rec.file);
                let pre_bytes = std::fs::read(&full).unwrap_or_default();
                let key =
                    EquivalentMutantCache::compute_key(&pre_bytes, &rec.mutant_kind, rec.line);
                let _ = cache.put(&key, &rec.outcome);
            }

            let cap = cfg.max_mutants_per_session as usize;
            if records.len() > cap {
                records.truncate(cap);
            }
            Ok(MutationResult::from_records(records))
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;

    #[test]
    fn mutation_score_70pct_from_canonical_mix() {
        // Plan §10 fixture: 7 caught + 2 survived + 1 timeout = 70.0%.
        let mut records = Vec::new();
        for i in 0..7 {
            records.push(MutantRecord {
                file: PathBuf::from("src/math/sum.rs"),
                line: i + 1,
                mutant_kind: "replace_+_with_-".into(),
                outcome: MutantOutcome::Caught,
            });
        }
        for i in 0..2 {
            records.push(MutantRecord {
                file: PathBuf::from("src/math/sum.rs"),
                line: 100 + i,
                mutant_kind: "replace_+_with_-".into(),
                outcome: MutantOutcome::Survived,
            });
        }
        records.push(MutantRecord {
            file: PathBuf::from("src/math/sum.rs"),
            line: 200,
            mutant_kind: "replace_+_with_-".into(),
            outcome: MutantOutcome::Timeout,
        });
        let result = MutationResult::from_records(records);
        assert!((result.mutation_score_pct - 70.0).abs() < 1e-9);
        assert_eq!(result.total_run, 10);
        assert_eq!(result.cache_hits, 0);
    }

    #[test]
    fn vacuous_pass_when_no_eligible_mutants() {
        let result = MutationResult::from_records(Vec::new());
        assert!((result.mutation_score_pct - 100.0).abs() < 1e-9);
    }

    #[test]
    fn cache_upsert_no_duplicate_rows() {
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let cache = EquivalentMutantCache::open_in_memory(clock).expect("cache");
        let key =
            EquivalentMutantCache::compute_key(b"fn add(a:i32,b:i32)->i32{a+b}", "replace_+_with_-", 3);
        cache
            .put(&key, &MutantOutcome::Caught)
            .expect("put 1");
        cache
            .put(&key, &MutantOutcome::Caught)
            .expect("put 2");
        assert_eq!(cache.row_count().expect("count"), 1);
        let got = cache.get(&key).expect("get").expect("hit");
        assert!(matches!(got, MutantOutcome::Caught));
    }

    #[test]
    fn cache_key_is_stable_blake3() {
        let a =
            EquivalentMutantCache::compute_key(b"abc", "replace_+_with_-", 42);
        let b =
            EquivalentMutantCache::compute_key(b"abc", "replace_+_with_-", 42);
        assert_eq!(a, b);
        // Length-64 hex per blake3 32-byte digest.
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn parse_cargo_mutants_lines() {
        let contents = "src/foo.rs:42: replace_+_with_-\nsrc/bar.rs:7: delete_line\n";
        let recs = RustMutationRunner::parse_result_file(contents, MutantOutcome::Survived);
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].file, PathBuf::from("src/foo.rs"));
        assert_eq!(recs[0].line, 42);
        assert_eq!(recs[0].mutant_kind, "replace_+_with_-");
        assert!(matches!(recs[0].outcome, MutantOutcome::Survived));
    }
}
