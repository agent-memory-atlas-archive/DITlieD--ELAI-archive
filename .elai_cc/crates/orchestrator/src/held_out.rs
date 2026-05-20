//! D-6: Time-windowed held-out task store + 9-gram contamination filter.
//!
//! Backs the LiveCodeBench-style contamination-resistant evaluation surface
//! consumed by [`crate::bai`] for ReasoningTemplate promotion gating. Tasks
//! are ingested with a submission timestamp and an eligibility window; only
//! tasks that have aged past their window become "active" — fresh tasks
//! cannot leak into the BAI eval pool until the model-cutoff guarantee is
//! plausibly behind them.
//!
//! Two surfaces:
//!   - [`HeldOutTask`] / [`ExpectedOutcome`] / [`TaskSource`] — the typed
//!     task envelope persisted to disk + SQLite.
//!   - [`HeldOutStore`] — rusqlite-backed index keyed by `task_id` and a
//!     `(submission_date, source)` covering index. Mirrors the memory
//!     crate's per-project SQLite pattern (see `crates/memory/src/store.rs`).
//!
//! Contamination defense:
//!   - [`ngram_filter`] — sliding `n`-gram hash signature over the
//!     normalized task body; default `n=9` per LiveCodeBench §3.2
//!     (arxiv:2403.07974, T1).
//!   - [`is_contaminated`] — Jaccard overlap on two `ngram_filter`
//!     signatures. Threshold defaults to 0.3 per the same paper.
//!
//! Anti-stale: blake3 (not sha2 / blake2) is the workspace hash family.
//! 9-gram fingerprints are computed as blake3 truncated-to-`u64` hashes so
//! the on-disk row size stays bounded (8 B × n_grams) and downstream Jaccard
//! comparisons reduce to `HashSet<u64>` intersection.
//!
//! Wiring:
//!   - [`HeldOutStore::insert`] emits an `held_out_task_added` SSB row.
//!   - [`HeldOutStore::mark_used`] emits an `held_out_task_used` SSB row.
//!   - [`crate::agent_pool::sample_held_out_task`] is the BAI consumer: it
//!     queries the store for active tasks and returns one for the
//!     promotion-gate eval. The BAI module itself never touches the store
//!     directly — it consumes the sampled task through the agent_pool
//!     surface so the SSB row producer is the only writer.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::ssb::{
    HeldOutTaskAddedPayload, HeldOutTaskUsedPayload, Ssb, SsbError, SsbPayload,
};

// ---------------------------------------------------------------------------
// Tunables (defaults match LiveCodeBench §3.2)
// ---------------------------------------------------------------------------

/// Default n-gram window size. LiveCodeBench (arxiv:2403.07974, T1) §3.2
/// published 9 as the contamination-detection sweet spot — large enough to
/// avoid common-bigram false positives, small enough that paraphrase still
/// trips the filter.
pub const DEFAULT_NGRAM_SIZE: usize = 9;

/// Default Jaccard threshold above which two signatures count as
/// contaminating each other. LiveCodeBench §3.2 reports 0.3 as the
/// false-positive-balanced operating point.
pub const DEFAULT_JACCARD_THRESHOLD: f32 = 0.3;

/// Default eligibility window in milliseconds. Tasks are not "active" for
/// BAI eval consumption until `now - ingested_ts_ms >= eligibility_window_ms`.
/// Default is zero (the LiveCodeBench seed corpus is pre-aged); production
/// ingest paths override per-source.
pub const DEFAULT_ELIGIBILITY_WINDOW_MS: i64 = 0;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Newtype around a stringly-typed task identifier. We use [`Uuid`] as the
/// underlying representation but expose `&str` everywhere so the SSB
/// payload + SQLite row layouts stay stable across UUID library upgrades.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    /// Generate a fresh v4 UUID in simple-hex (32-char, no dashes) form.
    /// Matches the convention used by [`crate::reasoning_template::TemplateId`].
    #[must_use]
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4().simple().to_string())
    }

    /// Borrow the underlying string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What the held-out task expects on a successful run. Closed enum so the
/// verifier consumer (D-3 multi-verifier consensus or D-2 PPM) can
/// dispatch on type without a fallthrough arm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ExpectedOutcome {
    /// Expected to compile + run + match this canonical string (or a test
    /// command's exit-code-0 outcome). Stored as the raw expected stdout
    /// snippet; the test runner contract is the consumer's responsibility.
    Code(String),
    /// Expected to reduce to this canonical sympy / numerical form.
    Math(String),
    /// Expected to satisfy this rubric (free-form prose).
    Prose(String),
}

/// Provenance of a held-out task. Closed sum; new sources require an
/// additive enum bump (caller-visible, not silent).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source_kind", rename_all = "snake_case")]
pub enum TaskSource {
    /// Sourced from a real GitHub issue. `repo` is `owner/name`; `issue_id`
    /// is the integer issue number.
    GitHubIssue { repo: String, issue_id: u64 },
    /// Sourced from a benchmark fixture (LiveCodeBench, ARC-AGI, etc.).
    BenchmarkFixture { name: String },
    /// Synthesized deterministically from a seed (D-7 cross-domain canary
    /// + D-15 NoveltyClass generators).
    Synthetic { seed: u64 },
}

impl TaskSource {
    /// Snake-case label used by the SSB row (`source` column) so the
    /// dashboard can group by source without re-deserializing the full
    /// `TaskSource` enum.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::GitHubIssue { .. } => "github_issue",
            Self::BenchmarkFixture { .. } => "benchmark_fixture",
            Self::Synthetic { .. } => "synthetic",
        }
    }
}

/// A single held-out task. `content_blake3` hashes the *normalized* task
/// body (the same normalization used by [`ngram_filter`]) so duplicate
/// detection by exact-match short-circuits before the n-gram scan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldOutTask {
    pub task_id: TaskId,
    pub source: TaskSource,
    /// Blake3 hash of the normalized body (hex, 64 chars).
    pub content_blake3: String,
    pub expected_outcome: ExpectedOutcome,
    /// Wall-clock ingest timestamp (UNIX ms). Used together with
    /// `eligibility_window_ms` to compute time-window activation.
    pub ingested_ts_ms: i64,
    /// Tasks become active for BAI eval consumption once
    /// `now - ingested_ts_ms >= eligibility_window_ms`. Zero means
    /// "active immediately".
    pub eligibility_window_ms: i64,
    /// Sliding 9-gram signature (default `DEFAULT_NGRAM_SIZE`) over the
    /// normalized body. Empty for tasks whose body is shorter than `n`
    /// tokens — those still index but cannot collide via this filter.
    pub ngram_signature: Vec<u64>,
    /// The raw normalized task body. Kept on the envelope (not just the
    /// hash) so the BAI evaluator can re-feed it to the verifier without
    /// a second store lookup.
    pub body: String,
}

impl HeldOutTask {
    /// True when the task's eligibility window has elapsed relative to
    /// `now_ms`. Used by [`HeldOutStore::active_tasks`].
    #[must_use]
    pub fn is_active(&self, now_ms: i64) -> bool {
        now_ms.saturating_sub(self.ingested_ts_ms) >= self.eligibility_window_ms
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum HeldOutError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("task already present: {0}")]
    DuplicateTask(String),
    #[error("contaminated task rejected: jaccard={jaccard:.4} >= threshold={threshold:.4}")]
    Contaminated { jaccard: f32, threshold: f32 },
    #[error("ssb error: {0}")]
    Ssb(#[from] SsbError),
}

// ---------------------------------------------------------------------------
// HeldOutStore — rusqlite-backed index
// ---------------------------------------------------------------------------

/// rusqlite-backed held-out task index. Mirrors the memory crate's
/// per-project SQLite pattern: lazy schema creation on `open`, single
/// SQLite file at the canonical [`elai_core::paths::held_out_db_path`].
pub struct HeldOutStore {
    conn: Connection,
}

const CREATE_HELD_OUT_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS held_out_tasks (
    task_id              TEXT PRIMARY KEY,
    source_label         TEXT NOT NULL,
    source_json          TEXT NOT NULL,
    content_blake3       TEXT NOT NULL,
    expected_outcome     TEXT NOT NULL,
    ingested_ts_ms       INTEGER NOT NULL,
    eligibility_window_ms INTEGER NOT NULL,
    ngram_signature      BLOB NOT NULL,
    body                 TEXT NOT NULL,
    used_count           INTEGER NOT NULL DEFAULT 0,
    last_used_ts_ms      INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_held_out_source ON held_out_tasks(source_label);
CREATE INDEX IF NOT EXISTS idx_held_out_ingested ON held_out_tasks(ingested_ts_ms);
CREATE INDEX IF NOT EXISTS idx_held_out_hash ON held_out_tasks(content_blake3);
";

impl HeldOutStore {
    /// Open (or create) the held-out task SQLite at `db_path`. The parent
    /// directory is created if missing. Schema is applied on every call —
    /// `CREATE TABLE IF NOT EXISTS` makes this idempotent.
    pub fn open(db_path: &Path) -> Result<Self, HeldOutError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        conn.execute_batch(CREATE_HELD_OUT_TABLE)?;
        Ok(Self { conn })
    }

    /// In-memory variant for tests.
    pub fn open_in_memory() -> Result<Self, HeldOutError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(CREATE_HELD_OUT_TABLE)?;
        Ok(Self { conn })
    }

    /// Insert a task. Rejects exact-hash duplicates outright; the
    /// contamination check against existing tasks is the caller's job
    /// (typically [`Self::insert_if_unique`]).
    pub fn insert(&self, task: &HeldOutTask) -> Result<(), HeldOutError> {
        let sig_bytes = ngram_signature_to_bytes(&task.ngram_signature);
        let source_json = serde_json::to_string(&task.source)?;
        let outcome_json = serde_json::to_string(&task.expected_outcome)?;
        let res = self.conn.execute(
            "INSERT INTO held_out_tasks \
             (task_id, source_label, source_json, content_blake3, expected_outcome, \
              ingested_ts_ms, eligibility_window_ms, ngram_signature, body) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                task.task_id.as_str(),
                task.source.label(),
                source_json,
                task.content_blake3,
                outcome_json,
                task.ingested_ts_ms,
                task.eligibility_window_ms,
                sig_bytes,
                task.body,
            ],
        );
        match res {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(HeldOutError::DuplicateTask(task.task_id.0.clone()))
            }
            Err(e) => Err(HeldOutError::Sqlite(e)),
        }
    }

    /// Insert only when the task does NOT collide with any existing task's
    /// n-gram signature above `threshold`. Returns `Ok(true)` on insert,
    /// `Ok(false)` on contamination skip, `Err(DuplicateTask)` on exact
    /// hash collision.
    pub fn insert_if_unique(
        &self,
        task: &HeldOutTask,
        threshold: f32,
    ) -> Result<bool, HeldOutError> {
        // Exact-hash short-circuit.
        if self.has_content_hash(&task.content_blake3)? {
            return Err(HeldOutError::DuplicateTask(task.task_id.0.clone()));
        }
        // n-gram contamination check against every existing task. For
        // small held-out stores (≤ ~10K tasks at LiveCodeBench scale)
        // O(N) is acceptable; a Bloom filter is the follow-up if/when
        // the store grows past that.
        let existing = self.all_signatures()?;
        for (other_id, other_sig) in &existing {
            if is_contaminated(&task.ngram_signature, other_sig, threshold) {
                tracing::debug!(
                    new_task = %task.task_id.0,
                    other_task = %other_id,
                    threshold = threshold,
                    "held_out: contamination skip"
                );
                return Ok(false);
            }
        }
        self.insert(task)?;
        Ok(true)
    }

    /// True when any row already carries this content hash.
    pub fn has_content_hash(&self, hash: &str) -> Result<bool, HeldOutError> {
        let n: Option<i64> = self
            .conn
            .query_row(
                "SELECT 1 FROM held_out_tasks WHERE content_blake3 = ?1 LIMIT 1",
                params![hash],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        Ok(n.is_some())
    }

    /// Return (task_id, signature) pairs for every row. Used by the
    /// contamination scan in [`Self::insert_if_unique`].
    pub fn all_signatures(&self) -> Result<Vec<(String, Vec<u64>)>, HeldOutError> {
        let mut stmt = self
            .conn
            .prepare("SELECT task_id, ngram_signature FROM held_out_tasks")?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let task_id: String = row.get(0)?;
            let sig_bytes: Vec<u8> = row.get(1)?;
            out.push((task_id, ngram_signature_from_bytes(&sig_bytes)));
        }
        Ok(out)
    }

    /// Lookup by id. None if absent.
    pub fn get(&self, task_id: &TaskId) -> Result<Option<HeldOutTask>, HeldOutError> {
        let row = self
            .conn
            .query_row(
                "SELECT task_id, source_json, content_blake3, expected_outcome, \
                 ingested_ts_ms, eligibility_window_ms, ngram_signature, body \
                 FROM held_out_tasks WHERE task_id = ?1",
                params![task_id.as_str()],
                row_to_task,
            )
            .optional()?;
        Ok(row)
    }

    /// All tasks whose eligibility window has elapsed relative to `now_ms`.
    pub fn active_tasks(&self, now_ms: i64) -> Result<Vec<HeldOutTask>, HeldOutError> {
        let mut stmt = self.conn.prepare(
            "SELECT task_id, source_json, content_blake3, expected_outcome, \
             ingested_ts_ms, eligibility_window_ms, ngram_signature, body \
             FROM held_out_tasks \
             WHERE ?1 - ingested_ts_ms >= eligibility_window_ms \
             ORDER BY ingested_ts_ms ASC",
        )?;
        let rows = stmt.query_map(params![now_ms], row_to_task)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Recent rows (up to `limit`) ordered by most-recent ingest first.
    /// Used by the dashboard Tauri command.
    pub fn recent(&self, limit: usize) -> Result<Vec<HeldOutTask>, HeldOutError> {
        let mut stmt = self.conn.prepare(
            "SELECT task_id, source_json, content_blake3, expected_outcome, \
             ingested_ts_ms, eligibility_window_ms, ngram_signature, body \
             FROM held_out_tasks ORDER BY ingested_ts_ms DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], row_to_task)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Per-source counts. Used by [`HeldOutStats`].
    pub fn counts_by_source(&self) -> Result<HashMap<String, u32>, HeldOutError> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_label, COUNT(*) FROM held_out_tasks GROUP BY source_label")?;
        let mut rows = stmt.query([])?;
        let mut out = HashMap::new();
        while let Some(row) = rows.next()? {
            let label: String = row.get(0)?;
            let n: i64 = row.get(1)?;
            out.insert(label, n as u32);
        }
        Ok(out)
    }

    /// Increment the `used_count` for the given task; record `now_ms` as
    /// the last-used timestamp. Returns false when the task was absent.
    pub fn mark_used_local(&self, task_id: &TaskId, now_ms: i64) -> Result<bool, HeldOutError> {
        let n = self.conn.execute(
            "UPDATE held_out_tasks SET used_count = used_count + 1, last_used_ts_ms = ?2 \
             WHERE task_id = ?1",
            params![task_id.as_str(), now_ms],
        )?;
        Ok(n > 0)
    }

    /// Total task count.
    pub fn count(&self) -> Result<u32, HeldOutError> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM held_out_tasks", [], |r| r.get(0))?;
        Ok(n as u32)
    }
}

/// Aggregate stats surfaced by the dashboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldOutStats {
    pub total: u32,
    pub by_source: HashMap<String, u32>,
    pub active: u32,
    /// Number of contamination filter hits observed since process start.
    /// Exposed through [`crate::agent_pool::sample_held_out_task`]'s caller
    /// (the dashboard consumes the SSB rows directly so this is the
    /// in-memory mirror).
    pub contamination_hits: u32,
}

fn row_to_task(row: &rusqlite::Row<'_>) -> Result<HeldOutTask, rusqlite::Error> {
    let task_id: String = row.get(0)?;
    let source_json: String = row.get(1)?;
    let content_blake3: String = row.get(2)?;
    let expected_outcome_json: String = row.get(3)?;
    let ingested_ts_ms: i64 = row.get(4)?;
    let eligibility_window_ms: i64 = row.get(5)?;
    let sig_bytes: Vec<u8> = row.get(6)?;
    let body: String = row.get(7)?;
    let source: TaskSource = serde_json::from_str(&source_json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e)))?;
    let expected_outcome: ExpectedOutcome = serde_json::from_str(&expected_outcome_json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?;
    Ok(HeldOutTask {
        task_id: TaskId(task_id),
        source,
        content_blake3,
        expected_outcome,
        ingested_ts_ms,
        eligibility_window_ms,
        ngram_signature: ngram_signature_from_bytes(&sig_bytes),
        body,
    })
}

fn ngram_signature_to_bytes(sig: &[u64]) -> Vec<u8> {
    let mut out = Vec::with_capacity(sig.len() * 8);
    for v in sig {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn ngram_signature_from_bytes(bytes: &[u8]) -> Vec<u64> {
    let mut out = Vec::with_capacity(bytes.len() / 8);
    let chunks = bytes.chunks_exact(8);
    for chunk in chunks {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(chunk);
        out.push(u64::from_le_bytes(arr));
    }
    out
}

// ---------------------------------------------------------------------------
// N-gram filter + contamination detection
// ---------------------------------------------------------------------------

/// Lowercase + collapse whitespace + drop ASCII punctuation. Same
/// normalization used to compute `content_blake3` on the envelope so the
/// exact-hash short-circuit and the n-gram filter agree on what
/// constitutes "the same body."
#[must_use]
pub fn normalize_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_space = true;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Compute the blake3 content hash of the normalized text. Hex output is
/// 64 chars (256-bit digest). Matches the workspace blake3 convention.
#[must_use]
pub fn content_hash(text: &str) -> String {
    let normalized = normalize_text(text);
    blake3::hash(normalized.as_bytes()).to_hex().to_string()
}

/// Sliding `n`-gram signature: tokenize on whitespace, slide a window of
/// `n` tokens, blake3-hash the joined window (with a single space
/// separator), truncate to `u64`. The output is a `Vec<u64>` rather than a
/// `HashSet` so the on-disk layout is byte-stable across runs (insertion
/// order matters for byte-equality; downstream Jaccard de-dupes via
/// `HashSet::from_iter`).
///
/// Empty / under-length input returns an empty signature — the caller's
/// contamination check then trivially returns `false` (no overlap with
/// anything).
#[must_use]
pub fn ngram_filter(text: &str, n: usize) -> Vec<u64> {
    if n == 0 {
        return Vec::new();
    }
    let normalized = normalize_text(text);
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    if tokens.len() < n {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(tokens.len() - n + 1);
    for window in tokens.windows(n) {
        let joined = window.join(" ");
        let h = blake3::hash(joined.as_bytes());
        // Take the first 8 bytes of the 32-byte digest as a u64. blake3
        // output is uniformly distributed so the truncation preserves the
        // collision-resistance budget for held-out sizes ≤ 2^32 tasks.
        let bytes = h.as_bytes();
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes[..8]);
        out.push(u64::from_le_bytes(arr));
    }
    out
}

/// Jaccard overlap of two n-gram signatures: |A ∩ B| / |A ∪ B|. Returns
/// `0.0` when both inputs are empty (vacuously disjoint, not vacuously
/// identical — LiveCodeBench convention).
#[must_use]
pub fn jaccard_overlap(a: &[u64], b: &[u64]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let set_a: HashSet<u64> = a.iter().copied().collect();
    let set_b: HashSet<u64> = b.iter().copied().collect();
    let inter = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        return 0.0;
    }
    inter as f32 / union as f32
}

/// True when `jaccard_overlap(signature_a, signature_b) >= threshold`.
#[must_use]
pub fn is_contaminated(signature_a: &[u64], signature_b: &[u64], threshold: f32) -> bool {
    jaccard_overlap(signature_a, signature_b) >= threshold
}

// ---------------------------------------------------------------------------
// SSB-emitting wrappers
// ---------------------------------------------------------------------------

/// Insert a task into the store AND emit a `held_out_task_added` SSB row.
/// On contamination the task is skipped — no SSB row is emitted (the
/// caller can inspect the bool to count the skip; the dashboard's
/// `contamination_hits` counter mirrors this).
pub async fn insert_and_emit(
    store: &HeldOutStore,
    task: &HeldOutTask,
    threshold: f32,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
) -> Result<bool, HeldOutError> {
    let inserted = store.insert_if_unique(task, threshold)?;
    if !inserted {
        return Ok(false);
    }
    let payload = SsbPayload::HeldOutTaskAdded(HeldOutTaskAddedPayload {
        task_id: task.task_id.0.clone(),
        source: task.source.label().to_string(),
        content_blake3: task.content_blake3.clone(),
        ngram_count: task.ngram_signature.len() as u32,
        ingested_ts_ms: task.ingested_ts_ms,
        eligibility_window_ms: task.eligibility_window_ms,
    });
    if let Err(e) = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        tracing::warn!(
            error = %e,
            task_id = %task.task_id.0,
            "held_out::insert_and_emit: SSB write failed"
        );
    }
    Ok(true)
}

/// Mark a task used AND emit a `held_out_task_used` SSB row. Used by the
/// BAI eval consumer in `agent_pool::sample_held_out_task` so the
/// dashboard can render a "which held-out tasks are feeding promotion"
/// timeline.
pub async fn mark_used_and_emit(
    store: &HeldOutStore,
    task_id: &TaskId,
    now_ms: i64,
    template_id: Option<&str>,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
) -> Result<bool, HeldOutError> {
    let updated = store.mark_used_local(task_id, now_ms)?;
    if !updated {
        return Ok(false);
    }
    let payload = SsbPayload::HeldOutTaskUsed(HeldOutTaskUsedPayload {
        task_id: task_id.0.clone(),
        consumer_template_id: template_id.unwrap_or("").to_string(),
        used_ts_ms: now_ms,
    });
    if let Err(e) = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        tracing::warn!(
            error = %e,
            task_id = %task_id.0,
            "held_out::mark_used_and_emit: SSB write failed"
        );
    }
    Ok(true)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a [`HeldOutTask`] from raw inputs. Computes `content_blake3` and
/// `ngram_signature` deterministically from the normalized body — callers
/// MUST NOT supply these fields themselves (the helper is the only way to
/// keep the two derived fields in sync).
#[must_use]
pub fn build_task(
    body: String,
    source: TaskSource,
    expected_outcome: ExpectedOutcome,
    ingested_ts_ms: i64,
    eligibility_window_ms: i64,
    n: usize,
) -> HeldOutTask {
    let content_blake3 = content_hash(&body);
    let ngram_signature = ngram_filter(&body, n);
    HeldOutTask {
        task_id: TaskId::new_v4(),
        source,
        content_blake3,
        expected_outcome,
        ingested_ts_ms,
        eligibility_window_ms,
        ngram_signature,
        body: normalize_text(&body),
    }
}

/// Path-builder helper for use by [`HeldOutStore`] consumers that want the
/// canonical on-disk JSONL path for a given month. Mirrors the
/// `held_out_dir_path` convention from `elai_core::paths`.
#[must_use]
pub fn held_out_month_dir(root: &Path, year_month: &str) -> PathBuf {
    root.join(year_month)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::SsbReader;

    fn ssb_for_test() -> Arc<Ssb> {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let cfg = crate::ssb::SsbConfig {
            db_path: dir.path().join("ssb_test.db"),
            pool_size: Some(2),
            project_id: "test-project".into(),
            session_id: "test-session".into(),
        };
        let clock: std::sync::Arc<dyn elai_core::ClockProvider> =
            std::sync::Arc::new(elai_core::clock::SeededClock::new());
        let ssb = Ssb::new(cfg, clock).expect("ssb init");
        // Keep the tempdir alive for the lifetime of the test.
        std::mem::forget(dir);
        Arc::new(ssb)
    }

    #[test]
    fn normalize_strips_punctuation_and_lowercases() {
        let s = "Hello, World!  Foo-bar?";
        assert_eq!(normalize_text(s), "hello world foo bar");
    }

    #[test]
    fn normalize_empty_returns_empty() {
        assert_eq!(normalize_text(""), "");
        assert_eq!(normalize_text("    "), "");
    }

    #[test]
    fn content_hash_is_deterministic_under_normalization() {
        let a = content_hash("Hello, World!");
        let b = content_hash("  hello   world  ");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn ngram_filter_3gram_basic() {
        let sig = ngram_filter("a b c d e", 3);
        // tokens: [a, b, c, d, e] -> 3-grams: (a,b,c), (b,c,d), (c,d,e) = 3 entries.
        assert_eq!(sig.len(), 3);
    }

    #[test]
    fn ngram_filter_under_length_returns_empty() {
        let sig = ngram_filter("a b", 9);
        assert!(sig.is_empty());
    }

    #[test]
    fn ngram_filter_empty_input_returns_empty() {
        assert!(ngram_filter("", 9).is_empty());
        assert!(ngram_filter("   ", 9).is_empty());
    }

    #[test]
    fn ngram_filter_zero_n_returns_empty() {
        assert!(ngram_filter("a b c d e f g h i j", 0).is_empty());
    }

    #[test]
    fn jaccard_identical_signatures_is_one() {
        let s = ngram_filter("the quick brown fox jumps over the lazy dog today", 3);
        assert!(!s.is_empty());
        assert!((jaccard_overlap(&s, &s) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn jaccard_disjoint_signatures_is_zero() {
        let a = ngram_filter("alpha beta gamma delta epsilon zeta eta", 3);
        let b = ngram_filter("one two three four five six seven", 3);
        assert!((jaccard_overlap(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn jaccard_empty_both_is_zero() {
        assert!((jaccard_overlap(&[], &[]) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn is_contaminated_fires_above_threshold() {
        let a = ngram_filter(
            "the quick brown fox jumps over the lazy dog today now",
            3,
        );
        let b = a.clone();
        assert!(is_contaminated(&a, &b, 0.3));
    }

    #[test]
    fn is_contaminated_skips_below_threshold() {
        let a = ngram_filter("alpha beta gamma delta epsilon zeta eta theta iota", 3);
        let b = ngram_filter("one two three four five six seven eight nine", 3);
        assert!(!is_contaminated(&a, &b, 0.3));
    }

    #[test]
    fn store_insert_then_get_roundtrip() {
        let store = HeldOutStore::open_in_memory().expect("open");
        let task = build_task(
            "the quick brown fox jumps over the lazy dog".into(),
            TaskSource::BenchmarkFixture {
                name: "fixture-1".into(),
            },
            ExpectedOutcome::Code("ok".into()),
            1_000_000,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        store.insert(&task).expect("insert");
        let got = store.get(&task.task_id).expect("get").expect("present");
        assert_eq!(got.task_id, task.task_id);
        assert_eq!(got.content_blake3, task.content_blake3);
        assert_eq!(got.ngram_signature, task.ngram_signature);
        assert_eq!(got.body, task.body);
    }

    #[test]
    fn store_insert_duplicate_hash_rejected() {
        let store = HeldOutStore::open_in_memory().expect("open");
        let task = build_task(
            "the quick brown fox jumps over the lazy dog".into(),
            TaskSource::Synthetic { seed: 1 },
            ExpectedOutcome::Code("ok".into()),
            1_000_000,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        store.insert(&task).expect("first");
        let err = store.insert(&task).expect_err("duplicate must fail");
        assert!(matches!(err, HeldOutError::DuplicateTask(_)));
    }

    #[test]
    fn store_active_tasks_respects_eligibility_window() {
        let store = HeldOutStore::open_in_memory().expect("open");
        let early = build_task(
            "alpha beta gamma delta epsilon zeta eta theta iota".into(),
            TaskSource::Synthetic { seed: 1 },
            ExpectedOutcome::Code("ok".into()),
            1_000,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        let late = build_task(
            "kappa lambda mu nu xi omicron pi rho sigma".into(),
            TaskSource::Synthetic { seed: 2 },
            ExpectedOutcome::Code("ok".into()),
            5_000,
            10_000,
            DEFAULT_NGRAM_SIZE,
        );
        store.insert(&early).expect("ins early");
        store.insert(&late).expect("ins late");

        // At t=2000, only `early` is active (eligibility_window_ms=0).
        let active = store.active_tasks(2_000).expect("active");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].task_id, early.task_id);

        // At t=20_000, both active (late's 10_000ms window has elapsed).
        let active = store.active_tasks(20_000).expect("active");
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn store_insert_if_unique_skips_contaminated() {
        let store = HeldOutStore::open_in_memory().expect("open");
        let body = "the quick brown fox jumps over the lazy dog and the cat".to_string();
        let a = build_task(
            body.clone(),
            TaskSource::Synthetic { seed: 1 },
            ExpectedOutcome::Code("ok".into()),
            1_000,
            0,
            3,
        );
        // Same body but different source — same signature so contamination
        // check must fire.
        let mut b = build_task(
            body.clone(),
            TaskSource::Synthetic { seed: 2 },
            ExpectedOutcome::Code("ok".into()),
            2_000,
            0,
            3,
        );
        // Bypass hash short-circuit by tweaking one byte of the stored hash
        // (simulating a near-duplicate that happens to have a different
        // content_blake3).
        b.content_blake3.replace_range(0..1, "f");
        store.insert(&a).expect("ins a");
        let inserted = store
            .insert_if_unique(&b, DEFAULT_JACCARD_THRESHOLD)
            .expect("contam check");
        assert!(!inserted, "contaminated near-dup must be skipped");
        assert_eq!(store.count().expect("count"), 1);
    }

    #[test]
    fn store_mark_used_increments_counter() {
        let store = HeldOutStore::open_in_memory().expect("open");
        let task = build_task(
            "the quick brown fox jumps over the lazy dog".into(),
            TaskSource::Synthetic { seed: 1 },
            ExpectedOutcome::Code("ok".into()),
            1_000,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        store.insert(&task).expect("insert");
        let updated = store.mark_used_local(&task.task_id, 5_000).expect("mark");
        assert!(updated);
        let absent = TaskId::new_v4();
        let updated = store.mark_used_local(&absent, 5_000).expect("mark absent");
        assert!(!updated);
    }

    #[test]
    fn store_counts_by_source_groups_correctly() {
        let store = HeldOutStore::open_in_memory().expect("open");
        for i in 0..3 {
            let t = build_task(
                format!("body number {i} unique tokens for jaccard isolation"),
                TaskSource::Synthetic { seed: i },
                ExpectedOutcome::Code("ok".into()),
                1_000 + i as i64,
                0,
                DEFAULT_NGRAM_SIZE,
            );
            store.insert(&t).expect("ins");
        }
        for i in 0..2 {
            let t = build_task(
                format!("issue body {i} with extra distinct vocabulary tokens"),
                TaskSource::GitHubIssue {
                    repo: "foo/bar".into(),
                    issue_id: i,
                },
                ExpectedOutcome::Code("ok".into()),
                2_000 + i as i64,
                0,
                DEFAULT_NGRAM_SIZE,
            );
            store.insert(&t).expect("ins");
        }
        let counts = store.counts_by_source().expect("counts");
        assert_eq!(counts.get("synthetic").copied().unwrap_or(0), 3);
        assert_eq!(counts.get("github_issue").copied().unwrap_or(0), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn insert_and_emit_writes_ssb_row() {
        let ssb = ssb_for_test();
        let store = HeldOutStore::open_in_memory().expect("open");
        let task = build_task(
            "alpha beta gamma delta epsilon zeta eta theta iota kappa".into(),
            TaskSource::Synthetic { seed: 42 },
            ExpectedOutcome::Code("ok".into()),
            1_000,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        let inserted = insert_and_emit(&store, &task, DEFAULT_JACCARD_THRESHOLD, &ssb, "test", 0)
            .await
            .expect("emit");
        assert!(inserted);
        // Verify SSB tail contains the row.
        let entries = ssb
            .query_by_type(crate::ssb::SsbActionType::HeldOutTaskAdded)
            .expect("query");
        assert!(!entries.is_empty(), "ssb must carry a HeldOutTaskAdded row");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mark_used_and_emit_writes_ssb_row() {
        let ssb = ssb_for_test();
        let store = HeldOutStore::open_in_memory().expect("open");
        let task = build_task(
            "alpha beta gamma delta epsilon zeta eta theta iota kappa".into(),
            TaskSource::Synthetic { seed: 42 },
            ExpectedOutcome::Code("ok".into()),
            1_000,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        store.insert(&task).expect("insert");
        let used =
            mark_used_and_emit(&store, &task.task_id, 5_000, Some("tmpl-1"), &ssb, "test", 1)
                .await
                .expect("emit");
        assert!(used);
        let entries = ssb
            .query_by_type(crate::ssb::SsbActionType::HeldOutTaskUsed)
            .expect("query");
        assert!(!entries.is_empty(), "ssb must carry a HeldOutTaskUsed row");
    }
}
