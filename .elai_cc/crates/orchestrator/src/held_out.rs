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
//! Contamination defense (two stacked techniques):
//!   - TEMPORAL DATE-WINDOWING — `task.release_date > model.training_cutoff`
//!     per LiveCodeBench (arxiv:2403.07974, T1) §S3.SS1 + §S6. This is the
//!     paper's central + only contamination mechanism. See
//!     [`ModelTrainingCutoff`] / [`is_task_after_cutoff`].
//!   - 9-gram Jaccard fingerprint — sliding n-gram hash signature with the
//!     n + threshold defaults below. The n-gram method itself is the
//!     external Shi et al. 2023 / Riddell et al. 2024 family that
//!     LiveCodeBench §S6.SS3 cites as PRIOR WORK, not a paper technique.
//!     ELAI bolts n-gram on top of date-windowing as a defense-in-depth
//!     layer — the defaults below are ELAI-empirical, NOT
//!     LiveCodeBench-specified.
//!   - Soft Contamination (arxiv:2602.12413, T1) is a SEPARATE replacement
//!     for n-gram (cosine + LLM annotation), not an n-gram extension. It
//!     does not cite LiveCodeBench. ELAI integrates the two with explicit
//!     layering; see workflow/reasoning/D-6-decisions.md DECISION-6.
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
// Tunables (ELAI-empirical defaults — see DECISION-4 in D-6-decisions.md)
// ---------------------------------------------------------------------------

/// Default n-gram window size. ELAI-empirical default: 9. The n-gram
/// contamination-detection technique itself is from Shi et al. 2023
/// (arXiv:2303.10130) and Riddell et al. 2024 — LiveCodeBench
/// (arxiv:2403.07974) §S6.SS3 cites them as PRIOR WORK. The paper does NOT
/// publish n-gram defaults. ELAI picked 9 as a defense-in-depth operating
/// point: large enough to avoid common-bigram false positives, small enough
/// that paraphrase still trips the filter. Override via the `[held_out]`
/// section of elai_cc.toml (`ngram_n`).
pub const DEFAULT_NGRAM_SIZE: usize = 9;

/// Default Jaccard threshold above which two signatures count as
/// contaminating each other. ELAI-empirical default: 0.3, calibrated
/// against the ELAI baseline corpus (NOT a LiveCodeBench published value).
/// Override via the `[held_out]` section of elai_cc.toml
/// (`ngram_threshold`).
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
        Self(Uuid::new_v4().simple().to_string()) // entropy-allow: held-out task id, opaque identifier never byte-compared by replay
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
///
/// `fetched_at_ms` / `curated_at_ms` carries the UNIX-ms wall-clock at
/// fetch/curate time on the two D-6-closure variants (ArxivRepro +
/// HumanCurated). The original three variants (GitHubIssue,
/// BenchmarkFixture, Synthetic) intentionally do NOT carry a fetched-at
/// field: their `ingested_ts_ms` on the surrounding [`HeldOutTask`]
/// already covers the wall-clock dimension and 15+ construction sites
/// across the workspace would break under an additive field change. ELAI
/// workspace policy is i64-ms (chrono is not a workspace dep — see
/// D-6-decisions.md DECISION-8).
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
    /// D-6 closure (GAP-4) — sourced from an arXiv reproducibility task.
    /// `arxiv_id` is the bare paper id (e.g. "2403.07974"); `fetched_at_ms`
    /// is wall-clock at fetch time. Used by the paper-ingestor follow-up
    /// to seed held-out tasks tied to a paper's release date for the
    /// temporal date-windowing defense (LiveCodeBench S3.SS1).
    ArxivRepro {
        arxiv_id: String,
        #[serde(default)]
        fetched_at_ms: i64,
    },
    /// D-6 closure (GAP-4) — hand-curated by a named operator. `curator_id`
    /// is the curator's stable identifier; `curated_at_ms` is wall-clock
    /// at curation. Useful for human-authored math/prose seeds where
    /// neither GitHubIssue nor BenchmarkFixture nor Synthetic applies.
    HumanCurated {
        curator_id: String,
        #[serde(default)]
        curated_at_ms: i64,
    },
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
            Self::ArxivRepro { .. } => "arxiv_repro",
            Self::HumanCurated { .. } => "human_curated",
        }
    }

    /// D-6 closure (GAP-4) — best-effort wall-clock UNIX-ms when this
    /// task source was fetched / curated. Returns `None` for the original
    /// three variants (where the surrounding `HeldOutTask.ingested_ts_ms`
    /// is the source of truth) and `Some(_)` for the ArxivRepro /
    /// HumanCurated variants whose provenance carries an explicit fetch
    /// timestamp distinct from ingest.
    #[must_use]
    pub fn fetched_at_ms(&self) -> Option<i64> {
        match self {
            Self::ArxivRepro { fetched_at_ms, .. } => Some(*fetched_at_ms),
            Self::HumanCurated { curated_at_ms, .. } => Some(*curated_at_ms),
            Self::GitHubIssue { .. } | Self::BenchmarkFixture { .. } | Self::Synthetic { .. } => {
                None
            }
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
    /// H-47: Optional stable template identifier. When `Some`, the task was
    /// generated from a named template and `eval_firewall::regenerate_instance`
    /// may produce a fresh variant per generation (GSM-Symbolic 2410.05229
    /// score-time regeneration). `None` means the body is authoritative and
    /// no template-regen applies. The field is backwards-compatible (`serde`
    /// default = `None`), so existing serialized tasks deserialize unchanged.
    #[serde(default)]
    pub template_id: Option<String>,
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
        template_id: None,
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
        template_id: None,
    }
}

/// Path-builder helper for use by [`HeldOutStore`] consumers that want the
/// canonical on-disk JSONL path for a given month. Mirrors the
/// `held_out_dir_path` convention from `elai_core::paths`.
#[must_use]
pub fn held_out_month_dir(root: &Path, year_month: &str) -> PathBuf {
    root.join(year_month)
}

// ---------------------------------------------------------------------------
// D-6 closure (GAP-3) — HeldOutStatus / RetireReason / AutoRefreshPolicy
// ---------------------------------------------------------------------------
//
// Status + retire-reason taxonomy. Adds the AutoRetired marker path called
// for by the D-6 plan (§stub_ban "HeldOutStore MUST NOT delete task files;
// AutoRetired is status-marked"). The status is currently surfaced via
// SSB rows + the new helpers below; the SQLite store does NOT carry a
// status column on the row layout (the closure preserves wire-compat with
// the shipped row layout — status transitions are recorded in SSB as
// HeldOutAutoRetire rows, mirrored to memory).

/// Lifecycle status of a held-out task. `Active` covers the
/// just-ingested-and-eligible path; `RecentlyAdded` covers the
/// pre-eligibility window; the three retire / quarantine states capture
/// the LiveCodeBench-style refresh policy + the contamination-detected
/// path the n-gram filter exposes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeldOutStatus {
    /// Past eligibility window; currently usable by the BAI eval pool.
    Active,
    /// Inside the eligibility window; not yet exposed to the eval pool.
    RecentlyAdded,
    /// Auto-retired because the task uniformly passed across the
    /// retirement-window. ELAI policy choice — NOT a LiveCodeBench
    /// technique (the paper retires by date, not by pass-rate). See
    /// [`RetireReason::UniformlyPassing`].
    RetiredUniformlyPassing,
    /// Auto-retired because the contamination filter (n-gram /
    /// soft-contamination) flagged the body as overlapping training
    /// corpus content.
    RetiredContaminated,
    /// Manually held out of the pool by operator decision.
    Quarantined,
}

/// Provenance of a contamination signal. Closed enum so the retire-reason
/// path can carry the originating signal source without crossing module
/// boundaries. `Eq` is intentionally NOT derived: `f32` payloads
/// (jaccard / cosine / threshold) do not satisfy total equality; use
/// `PartialEq` for round-trip + assertion paths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContaminationSource {
    /// 9-gram Jaccard-overlap signature (Shi et al. 2023 / Riddell et al.
    /// 2024) — the [`is_contaminated`] path. `jaccard` is the measured
    /// overlap that crossed [`DEFAULT_JACCARD_THRESHOLD`] (or its
    /// elai_cc.toml override).
    NgramJaccard { jaccard: f32, threshold: f32 },
    /// Cosine + LLM-annotation semantic fingerprint (Soft Contamination
    /// arxiv:2602.12413). Reserved for the follow-up that wires the
    /// sidecar-embedded fingerprint path; the variant lands now so the
    /// retire-reason serde surface is stable.
    SoftContamination { cosine: f32, threshold: f32 },
    /// Operator-supplied contamination evidence (e.g. paper-ingestor
    /// flagged the source as already in training corpus).
    OperatorEvidence { note: String },
}

/// Reason a held-out task was retired. Carried inside SSB
/// `HeldOutAutoRetire` rows + memory-bank entries so the audit trail is
/// machine-readable. `Eq` not derived: the nested
/// [`ContaminationSource`] carries `f32` payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RetireReason {
    /// Task passed uniformly inside the retirement window (suspected
    /// contamination via training-set leakage). ELAI policy — NOT
    /// LiveCodeBench-specified; the paper retires by date-windowing
    /// alone.
    UniformlyPassing { passes_in_window: u32, window_months: u32 },
    /// Contamination filter flagged the body. Carries the originating
    /// signal source for audit.
    ContaminationDetected { source: ContaminationSource },
    /// Operator decision (manual). `note` is free text.
    OperatorRetired { note: String },
}

/// Auto-refresh policy. Controls when [`check_for_auto_retire`] picks a
/// candidate task for retirement; loaded from the `[held_out]` section of
/// elai_cc.toml at boot (see [`elai_rules::held_out::HeldOutTunables`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum AutoRefreshPolicy {
    /// Refresh disabled (default for `cargo test` runs to keep determinism
    /// untouched by clock-dependent retirement).
    Disabled,
    /// Refresh every `interval_days` of wall-clock time.
    OnSchedule { interval_days: u32 },
    /// Refresh whenever the contamination filter fires (event-driven).
    OnContamination,
    /// Refresh when the held-out accuracy drift exceeds `threshold`
    /// against the BAI baseline.
    OnAccuracyDrift { threshold: f64 },
}

/// Model training cutoff for the temporal date-windowing defense. Tasks
/// with `release_date > cutoff.date_ms` are eligible; older tasks are
/// presumed in-training and excluded. Matches LiveCodeBench (arxiv:
/// 2403.07974) §S3.SS1 — the paper's PRIMARY contamination mechanism.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelTrainingCutoff {
    /// Stable label for the model (e.g. `claude-sonnet-4.6`,
    /// `qwen2.5-coder-7b`). Used by the audit trail.
    pub model_id: String,
    /// UNIX-ms wall-clock of the training data cutoff. Tasks with
    /// `release_date_ms > date_ms` are considered post-cutoff and
    /// therefore eligible.
    pub date_ms: i64,
}

/// True when the task's release date is STRICTLY AFTER the model's
/// training cutoff — the LiveCodeBench S3.SS1 temporal eligibility
/// predicate. ELAI bridges `release_date` from the surrounding
/// `HeldOutTask.ingested_ts_ms` (ingest is a proxy for release on
/// synthesized / curated tasks) plus the optional [`TaskSource::fetched_at_ms`]
/// when the source carries a more precise wall-clock.
#[must_use]
pub fn is_task_after_cutoff(task: &HeldOutTask, cutoff: &ModelTrainingCutoff) -> bool {
    // Prefer the source-supplied fetched_at_ms when present (ArxivRepro
    // and HumanCurated variants), otherwise fall back to ingest.
    let release_ms = task.source.fetched_at_ms().unwrap_or(task.ingested_ts_ms);
    release_ms > cutoff.date_ms
}

// ---------------------------------------------------------------------------
// D-6 closure (GAP-3) — check_for_auto_retire
// ---------------------------------------------------------------------------

/// Per-task evaluation history snapshot. Provided by the BAI / canary
/// evaluator pipeline so the retire-decision logic is unit-testable
/// without dragging in the full Bandit posterior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalHistorySnapshot {
    /// How many distinct evaluations have run against this task.
    pub eval_count: u32,
    /// Number of those evaluations that passed.
    pub pass_count: u32,
}

impl EvalHistorySnapshot {
    /// Pass-rate in `[0.0, 1.0]`. Returns 0.0 on `eval_count == 0` to
    /// avoid undefined behaviour at the divide-by-zero path.
    #[must_use]
    pub fn pass_rate(&self) -> f32 {
        if self.eval_count == 0 {
            0.0
        } else {
            self.pass_count as f32 / self.eval_count as f32
        }
    }
}

/// Walk the store and decide which tasks merit auto-retire under the
/// `policy`. Returns `(task_id, retire_reason)` pairs the caller can feed
/// to the SSB `HeldOutAutoRetire` emitter.
///
/// Pure decision logic — does NOT mutate the store. The caller is
/// responsible for writing the SSB row + updating the in-memory status
/// mirror. `eval_history` is a function that maps a task id to its eval
/// snapshot (Closure-driven so tests can provide synthetic histories
/// without an EvalHistory store dependency).
///
/// Defense against premature retirement (plan stub_ban): no task with
/// fewer than `min_evals_for_retire` evaluations can be retired, even if
/// the pass-rate predicate fires.
pub fn check_for_auto_retire<F>(
    store: &HeldOutStore,
    now_ms: i64,
    policy: &AutoRefreshPolicy,
    eval_history: F,
    retirement_window_months: u32,
    retirement_uniformly_passing_count: u32,
    min_evals_for_retire: u32,
) -> Result<Vec<(TaskId, RetireReason)>, HeldOutError>
where
    F: Fn(&TaskId) -> EvalHistorySnapshot,
{
    // Disabled-policy short-circuit — no work, no retire candidates.
    if matches!(policy, AutoRefreshPolicy::Disabled) {
        return Ok(Vec::new());
    }
    let window_ms = (retirement_window_months as i64) * 30 * 86_400 * 1000;
    let active = store.active_tasks(now_ms)?;
    let mut out = Vec::new();
    for task in &active {
        // Age gate — task must be older than the retirement window.
        if now_ms.saturating_sub(task.ingested_ts_ms) < window_ms {
            continue;
        }
        let snap = eval_history(&task.task_id);
        // Premature-retirement guard — small samples never retire.
        if snap.eval_count < min_evals_for_retire {
            continue;
        }
        // Uniformly-passing predicate. ELAI policy choice (NOT
        // LiveCodeBench-specified): pass_count >= retirement_uniformly_
        // passing_count AND pass_rate == 1.0. The hard count guards
        // against tiny eval pools coincidentally hitting 100%.
        if snap.pass_count >= retirement_uniformly_passing_count
            && (snap.pass_rate() - 1.0).abs() < f32::EPSILON
        {
            out.push((
                task.task_id.clone(),
                RetireReason::UniformlyPassing {
                    passes_in_window: snap.pass_count,
                    window_months: retirement_window_months,
                },
            ));
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// D-6 closure (GAP-8) — Promotion gate (PromotionGateResult, gate_promotion,
// held_out_regression, gate_promotion_via_held_out)
// ---------------------------------------------------------------------------

/// Outcome of a held-out promotion gate evaluation. Renamed from
/// `GateResult` to avoid the collision with the
/// [`crate::gate::GateResult`] evidence-gate type — see
/// workflow/reasoning/D-6-decisions.md DECISION-3.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromotionGateResult {
    /// True when the candidate template does NOT regress on the held-out
    /// pool beyond the configured threshold.
    pub allow: bool,
    /// Human-readable reason for the gate's decision. Carries the
    /// measured delta in pp + the threshold for the audit trail.
    pub reason: String,
    /// Measured pass-rate delta in `[-1.0, 1.0]` (candidate - baseline).
    pub delta: f32,
}

/// Compute the pass-rate delta between baseline and candidate eval
/// histories on a shared held-out subset (positive = improvement,
/// negative = regression). The pass-rate is taken over the union of task
/// ids in both histories.
#[must_use]
pub fn held_out_regression(
    baseline: &[(TaskId, EvalHistorySnapshot)],
    candidate: &[(TaskId, EvalHistorySnapshot)],
) -> f32 {
    let baseline_rate: f32 = if baseline.is_empty() {
        0.0
    } else {
        baseline.iter().map(|(_, s)| s.pass_rate()).sum::<f32>() / baseline.len() as f32
    };
    let candidate_rate: f32 = if candidate.is_empty() {
        0.0
    } else {
        candidate.iter().map(|(_, s)| s.pass_rate()).sum::<f32>() / candidate.len() as f32
    };
    candidate_rate - baseline_rate
}

/// Apply the regression-threshold predicate to a measured `delta`. The
/// threshold MUST come from the loaded `[held_out]` section of
/// elai_cc.toml — never hard-coded by the caller (per the D-6 plan
/// stub_ban rule).
#[must_use]
pub fn gate_promotion(delta: f32, regression_threshold: f32) -> PromotionGateResult {
    let allow = delta >= -regression_threshold;
    let reason = if allow {
        format!(
            "delta={:.4} pp (allowed: >= -{:.4} pp regression threshold)",
            delta, regression_threshold
        )
    } else {
        format!(
            "delta={:.4} pp (BLOCK: exceeds -{:.4} pp regression threshold)",
            delta, regression_threshold
        )
    };
    PromotionGateResult {
        allow,
        reason,
        delta,
    }
}

/// D-6 closure (GAP-8) — thin wrapper that documents the BAI consumer
/// layering. The shipped BAI promotion path routes through
/// `agent_pool::sample_held_out_task` which yields the next held-out
/// task; this helper bundles the regression + gate computation against
/// pre-collected eval histories so a call site that wants the
/// promotion-gate decision can stay inside this module rather than
/// reimporting agent_pool. The decision doc DECISION-7 has the layering
/// rationale.
#[must_use]
pub fn gate_promotion_via_held_out(
    baseline: &[(TaskId, EvalHistorySnapshot)],
    candidate: &[(TaskId, EvalHistorySnapshot)],
    regression_threshold: f32,
) -> PromotionGateResult {
    let delta = held_out_regression(baseline, candidate);
    gate_promotion(delta, regression_threshold)
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

    // -----------------------------------------------------------------------
    // D-6 closure (GAP-3/4/7/8) — new types + helpers coverage.
    // -----------------------------------------------------------------------

    #[test]
    fn task_source_label_covers_all_variants() {
        assert_eq!(
            TaskSource::GitHubIssue {
                repo: "owner/name".into(),
                issue_id: 1
            }
            .label(),
            "github_issue"
        );
        assert_eq!(
            TaskSource::BenchmarkFixture {
                name: "lcb".into()
            }
            .label(),
            "benchmark_fixture"
        );
        assert_eq!(
            TaskSource::Synthetic { seed: 7 }.label(),
            "synthetic"
        );
        assert_eq!(
            TaskSource::ArxivRepro {
                arxiv_id: "2403.07974".into(),
                fetched_at_ms: 1_700_000_000_000
            }
            .label(),
            "arxiv_repro"
        );
        assert_eq!(
            TaskSource::HumanCurated {
                curator_id: "maintainer".into(),
                curated_at_ms: 1_700_000_000_000
            }
            .label(),
            "human_curated"
        );
    }

    #[test]
    fn task_source_fetched_at_ms_some_for_d6_variants() {
        assert_eq!(
            TaskSource::ArxivRepro {
                arxiv_id: "2403.07974".into(),
                fetched_at_ms: 42
            }
            .fetched_at_ms(),
            Some(42)
        );
        assert_eq!(
            TaskSource::HumanCurated {
                curator_id: "maintainer".into(),
                curated_at_ms: 99
            }
            .fetched_at_ms(),
            Some(99)
        );
        assert!(TaskSource::Synthetic { seed: 1 }.fetched_at_ms().is_none());
    }

    #[test]
    fn is_task_after_cutoff_uses_fetched_at_ms_when_present() {
        let cutoff = ModelTrainingCutoff {
            model_id: "qwen2.5-coder-7b".into(),
            date_ms: 1_700_000_000_000,
        };
        // ArxivRepro with fetched_at_ms strictly after cutoff -> true.
        let task_after = build_task(
            "alpha beta gamma delta epsilon zeta eta theta iota kappa".into(),
            TaskSource::ArxivRepro {
                arxiv_id: "2403.07974".into(),
                fetched_at_ms: 1_700_000_000_001,
            },
            ExpectedOutcome::Code("ok".into()),
            // ingest BEFORE cutoff to prove the predicate prefers
            // fetched_at_ms when present.
            1_500_000_000_000,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        assert!(is_task_after_cutoff(&task_after, &cutoff));

        // Synthetic falls back to ingested_ts_ms.
        let task_before = build_task(
            "alpha beta gamma delta epsilon zeta eta theta iota kappa".into(),
            TaskSource::Synthetic { seed: 1 },
            ExpectedOutcome::Code("ok".into()),
            1_500_000_000_000,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        assert!(!is_task_after_cutoff(&task_before, &cutoff));
    }

    #[test]
    fn check_for_auto_retire_respects_disabled_policy() {
        let store = HeldOutStore::open_in_memory().expect("open");
        let task = build_task(
            "alpha beta gamma delta epsilon zeta eta theta iota kappa".into(),
            TaskSource::Synthetic { seed: 1 },
            ExpectedOutcome::Code("ok".into()),
            1_000,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        store.insert(&task).expect("insert");
        let out = check_for_auto_retire(
            &store,
            i64::MAX / 4,
            &AutoRefreshPolicy::Disabled,
            |_| EvalHistorySnapshot {
                eval_count: 1_000,
                pass_count: 1_000,
            },
            6,
            100,
            20,
        )
        .expect("retire");
        assert!(out.is_empty(), "Disabled policy must never retire");
    }

    #[test]
    fn check_for_auto_retire_requires_min_evals() {
        let store = HeldOutStore::open_in_memory().expect("open");
        let task = build_task(
            "alpha beta gamma delta epsilon zeta eta theta iota kappa".into(),
            TaskSource::Synthetic { seed: 1 },
            ExpectedOutcome::Code("ok".into()),
            // ingested 12 months ago at "now".
            0,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        store.insert(&task).expect("insert");
        let now = 12 * 30 * 86_400 * 1_000;
        // pass_rate is 1.0 but eval_count below min_evals_for_retire.
        let out = check_for_auto_retire(
            &store,
            now,
            &AutoRefreshPolicy::OnSchedule { interval_days: 30 },
            |_| EvalHistorySnapshot {
                eval_count: 5,
                pass_count: 5,
            },
            6,
            3,
            20, // min_evals_for_retire = 20, only 5 evals available
        )
        .expect("retire");
        assert!(out.is_empty(), "must not retire below min_evals_for_retire");
    }

    #[test]
    fn check_for_auto_retire_uniformly_passing_retires() {
        let store = HeldOutStore::open_in_memory().expect("open");
        let task = build_task(
            "alpha beta gamma delta epsilon zeta eta theta iota kappa".into(),
            TaskSource::Synthetic { seed: 1 },
            ExpectedOutcome::Code("ok".into()),
            // ingested 12 months ago at "now".
            0,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        store.insert(&task).expect("insert");
        let now = 12 * 30 * 86_400 * 1_000;
        let out = check_for_auto_retire(
            &store,
            now,
            &AutoRefreshPolicy::OnSchedule { interval_days: 30 },
            |_| EvalHistorySnapshot {
                eval_count: 100,
                pass_count: 100,
            },
            6,
            100,
            20,
        )
        .expect("retire");
        assert_eq!(out.len(), 1, "uniformly-passing aged task must retire");
        match &out[0].1 {
            RetireReason::UniformlyPassing {
                passes_in_window,
                window_months,
            } => {
                assert_eq!(*passes_in_window, 100);
                assert_eq!(*window_months, 6);
            }
            other => panic!("unexpected reason: {other:?}"),
        }
    }

    #[test]
    fn gate_promotion_allows_within_threshold() {
        let r = gate_promotion(-0.03, 0.05);
        assert!(r.allow);
        assert!((r.delta - (-0.03)).abs() < 1e-6);
        assert!(r.reason.contains("allowed"));
    }

    #[test]
    fn gate_promotion_blocks_beyond_threshold() {
        let r = gate_promotion(-0.07, 0.05);
        assert!(!r.allow);
        assert!(r.reason.contains("BLOCK"));
    }

    #[test]
    fn held_out_regression_returns_delta() {
        let baseline = vec![
            (
                TaskId::new_v4(),
                EvalHistorySnapshot {
                    eval_count: 10,
                    pass_count: 8,
                },
            ),
            (
                TaskId::new_v4(),
                EvalHistorySnapshot {
                    eval_count: 10,
                    pass_count: 6,
                },
            ),
        ];
        let candidate = vec![
            (
                TaskId::new_v4(),
                EvalHistorySnapshot {
                    eval_count: 10,
                    pass_count: 9,
                },
            ),
            (
                TaskId::new_v4(),
                EvalHistorySnapshot {
                    eval_count: 10,
                    pass_count: 9,
                },
            ),
        ];
        // baseline mean = (0.8 + 0.6) / 2 = 0.7
        // candidate mean = (0.9 + 0.9) / 2 = 0.9
        // delta = 0.2
        let delta = held_out_regression(&baseline, &candidate);
        assert!((delta - 0.2).abs() < 1e-6, "delta={delta}");
    }

    #[test]
    fn gate_promotion_via_held_out_combines_path() {
        let baseline = vec![(
            TaskId::new_v4(),
            EvalHistorySnapshot {
                eval_count: 10,
                pass_count: 10,
            },
        )];
        let candidate = vec![(
            TaskId::new_v4(),
            EvalHistorySnapshot {
                eval_count: 10,
                pass_count: 3,
            },
        )];
        let r = gate_promotion_via_held_out(&baseline, &candidate, 0.05);
        assert!(!r.allow, "70% regression must block");
        assert!((r.delta - (-0.7)).abs() < 1e-6);
    }
}
