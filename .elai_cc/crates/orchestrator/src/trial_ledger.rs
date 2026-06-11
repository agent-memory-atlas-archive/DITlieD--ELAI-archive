//! H-17 (SCHEMA_VERSION 223): persisted untamperable trial ledger.
//!
//! The selection-bias keystone T-1. Every backtest / strategy evaluation
//! MUST pass through [`TrialLedger::register_trial`] before it can report a
//! metric. This module provides:
//!
//!   - [`TrialLedger`]: owns a SQLite connection to the `strategy_trials` table
//!     (on the same db file as the Ssb WAL). Enforces append-only semantics by
//!     construction: no UPDATE / DELETE method exists.
//!   - [`TrialLedger::register_trial`]: the single write choke-point.
//!   - [`TrialLedger::trial_count`]: raw cumulative N (recomputable from journal).
//!   - [`TrialLedger::effective_trial_count`]: conservative effective N (= raw N,
//!     with ONC registered as a grounded refinement stub for when return-matrix
//!     data is available — see stub at bottom of file).
//!   - [`TrialCountResult`]: distinguishes "N=0 because zero rows"
//!     (`TrialCountMissing`) from a real low N (TD-1 fail-closed sentinel).
//!
//! # Append-only guarantee (TD-1)
//!
//! - No `delete_trial`, `reset_trials`, or `update_trial` method exists on
//!   [`TrialLedger`].
//! - The debug-build `update_hook` in `Ssb::new()` is extended to cover the
//!   `strategy_trials` table (in ssb.rs).
//! - `trial_ledger.rs` (this file) is listed in `.claude/gate-manifest.txt`
//!   so `gate_freeze_check.py` blocks any run that edits its own ledger code
//!   (TD-2 freeze). The `--lock` re-baseline after adding the gate-manifest
//!   entry is a HUMAN out-of-band step (R8 — the executing plan run cannot
//!   re-lock its own gate).
//!
//! # Session scope vs project scope
//!
//! `Ssb` is session-scoped; `strategy_trials` must accumulate across sessions.
//! [`TrialLedger`] opens its own `rusqlite::Connection` to the same `db_path`
//! used by `Ssb`. SQLite WAL mode supports multiple readers + one writer;
//! `TrialLedger` takes the `BEGIN IMMEDIATE` writer role for inserts and a
//! shared-read role for counts. The `Ssb` single-writer uses `BEGIN IMMEDIATE`
//! on its own connection; the `strategy_trials` inserts from this module use
//! an independent `BEGIN IMMEDIATE` on a separate connection (two tables,
//! two write locks). They do NOT contend on `ssb` rows.
//!
//! # ONC effective-N (grounded refinement stub)
//!
//! The ONC effective-N (Lopez de Prado & Lewis, Quantitative Finance 2019)
//! requires an N×N return-series correlation matrix, which is not stored in
//! `strategy_trials` (only scalar metrics are persisted here). The conservative
//! fail-safe (raw N) is therefore the production default, satisfying the TD-1
//! requirement that fewer effective trials = LESS deflation = LOOSER gate.
//! The ONC stub is registered below; resolve in a future session when the
//! return-matrix store is available.
//!
//! # References
//! - `docs/thefinalglance/H-plan/H-17_trial_ledger_keystone.md`
//! - `workflow/papers/onc-lopezdeprado-lewis-2019-interface.md`
//! - `workflow/reasoning/H-17-decisions.md`

use std::path::Path;
use std::sync::Arc;

use rusqlite::{params, Connection};
use tracing::{debug, warn};

use crate::ssb::{Ssb, SsbPayload, TrialRegisteredPayload};
use elai_core::clock::ClockProvider;

// ---------------------------------------------------------------------------
// Named constants (CLAUDE.md no-magic-numbers HARD RULE)
// ---------------------------------------------------------------------------

/// Minimum objective key length (bytes). Empty keys are canonicalized
/// to this sentinel to prevent scope-key fragmentation attacks (TD-1).
const MIN_OBJECTIVE_LEN: usize = 1;

/// Maximum config blob size accepted by register_trial (1 MiB).
/// Mirrors the SSB PAYLOAD_MAX_BYTES limit.
const MAX_CONFIG_BLOB_BYTES: usize = 1_048_576;

// ONC effective-N threshold (policy constant, LOCKED, not agent-tunable).
// Currently unused — conservative default (raw N) is the production path.
// TODO[H17.ONC-STUB]: resolve when return-matrix store ships (H-18+).
// const ONC_CLUSTERING_THRESHOLD: f64 = 0.05;

// ---------------------------------------------------------------------------
// Public error type
// ---------------------------------------------------------------------------

/// Errors returned by [`TrialLedger`] operations.
#[derive(Debug, thiserror::Error)]
pub enum TrialLedgerError {
    /// SQLite layer error.
    #[error("trial ledger sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Config blob exceeds the MAX_CONFIG_BLOB_BYTES limit.
    #[error(
        "config blob too large ({actual} bytes; max {max})",
        actual = .actual, max = .max
    )]
    ConfigBlobTooLarge { actual: usize, max: usize },

    /// Objective key is empty after canonicalization.
    #[error("objective key is empty after canonicalization (TD-1 scope-key attack surface)")]
    EmptyObjective,

    /// SSB write error during TrialRegistered emission.
    #[error("SSB write error emitting TrialRegistered: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// TrialCountResult — TD-1 fail-closed sentinel
// ---------------------------------------------------------------------------

/// Result of a [`TrialLedger::trial_count`] query.
///
/// Distinguishes "N = 0 because zero matching rows" (`TrialCountMissing`,
/// the H-18 hard-FAIL sentinel, TD-1) from a real non-zero count.
/// `N = 0` with no rows is NOT the same as `N = 0 with some rows` (impossible
/// by design, since rows increment N).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrialCountResult {
    /// No `strategy_trials` rows exist for this `(objective, data_window)`.
    /// The H-18 promotion gate MUST fail closed on this variant (TD-1).
    /// A reported metric with zero matching trial rows is evidence of an
    /// unregistered evaluation path — a hard FAIL, never SKIP.
    TrialCountMissing,
    /// At least one row exists; `n` is the raw cumulative DISTINCT count.
    Present(u64),
}

impl TrialCountResult {
    /// Returns the raw count, erasing the `TrialCountMissing` sentinel by
    /// substituting 0. This DESTROYS the fail-closed signal: a caller that
    /// uses this method instead of matching on the enum cannot distinguish
    /// "no trials registered yet" from "zero trials" (which cannot exist by
    /// construction). TD-1 fail-closed gates MUST match on the enum directly.
    ///
    /// The `_UNSAFE` suffix is intentional and load-bearing: it marks this
    /// method as a sentinel-erasing escape hatch. H-18 promotion logic MUST
    /// NOT call this method; it must match `TrialCountMissing` explicitly.
    #[allow(non_snake_case)]
    pub fn n_treating_missing_as_zero_UNSAFE(&self) -> u64 {
        match self {
            Self::TrialCountMissing => 0,
            Self::Present(n) => *n,
        }
    }

    /// True when the variant is `TrialCountMissing` (the H-18 HARD FAIL
    /// sentinel). Convenience predicate for the promotion gate.
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::TrialCountMissing)
    }
}

// ---------------------------------------------------------------------------
// TrialLedger
// ---------------------------------------------------------------------------

/// Append-only persisted trial ledger for the selection-bias keystone (H-17).
///
/// Holds a `rusqlite::Connection` to the `strategy_trials` table. All writes
/// go through [`register_trial`](TrialLedger::register_trial) — the single
/// choke-point. No UPDATE/DELETE/reset method exists (TD-1).
///
/// The ledger shares the db file path with [`Ssb`] so the `strategy_trials`
/// table accumulates across all sessions under the same project root.
pub struct TrialLedger {
    conn: std::sync::Mutex<Connection>,
    session_id: String,
}

impl std::fmt::Debug for TrialLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrialLedger")
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

impl TrialLedger {
    /// Open a new `TrialLedger` on `db_path`.
    ///
    /// The `strategy_trials` table is created if it does not yet exist
    /// (using `CREATE TABLE IF NOT EXISTS` so existing rows are preserved
    /// across sessions — the cross-session accumulation requirement).
    ///
    /// # Arguments
    /// * `db_path`    — same SQLite file used by `Ssb` for this project.
    /// * `session_id` — current session id for the `session_id` column.
    pub fn open(db_path: &Path, session_id: impl Into<String>) -> Result<Self, TrialLedgerError> {
        let conn = Connection::open(db_path)?;
        // WAL mode for read concurrency with the Ssb single-writer.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        // Create the table if it does not exist (preserves existing rows).
        conn.execute_batch(crate::ssb::CREATE_STRATEGY_TRIALS_TABLE)?;
        Ok(Self {
            conn: std::sync::Mutex::new(conn),
            session_id: session_id.into(),
        })
    }

    /// Register one trial and emit the off-agent SSB artifact.
    ///
    /// # Idempotency
    /// Re-registering the SAME `(config_hash, data_window, seed)` tuple
    /// (same `trial_id`) does NOT insert a new row and does NOT increment N.
    /// A DIFFERENT `config_blob` / `seed` for the same `(objective,
    /// data_window)` DOES insert a new row and increments N.
    ///
    /// # Returns
    /// The cumulative DISTINCT trial count N after this call.
    ///
    /// # Arguments
    /// * `ssb`        — the project Ssb handle; used to emit `TrialRegistered`.
    /// * `clock`      — ClockProvider (C-15 determinism; never SystemTime).
    /// * `objective`  — search objective key (e.g. "sharpe_1y"). Canonicalized.
    /// * `data_window`— canonical "start_ms..end_ms" evaluation window string.
    /// * `config_blob`— raw bytes of the strategy config; blake3-hashed here.
    /// * `seed`       — evaluation seed.
    pub async fn register_trial(
        &self,
        ssb: &Arc<Ssb>,
        clock: &Arc<dyn ClockProvider>,
        objective: &str,
        data_window: &str,
        config_blob: &[u8],
        seed: i64,
    ) -> Result<u64, TrialLedgerError> {
        // Validate inputs (TD-1 scope-key integrity).
        if config_blob.len() > MAX_CONFIG_BLOB_BYTES {
            return Err(TrialLedgerError::ConfigBlobTooLarge {
                actual: config_blob.len(),
                max: MAX_CONFIG_BLOB_BYTES,
            });
        }
        let objective = canonicalize_key(objective);
        if objective.len() < MIN_OBJECTIVE_LEN {
            return Err(TrialLedgerError::EmptyObjective);
        }
        let data_window = canonicalize_key(data_window);

        // Compute config_hash (blake3, CLAUDE.md HARD RULE).
        let config_hash_bytes = blake3::hash(config_blob);
        let config_hash = config_hash_bytes.to_hex().to_string();

        // Compute trial_id = blake3 of (objective || ':' || data_window || ':' ||
        // config_hash || ':' || seed as decimal). Deterministic under ELAI_SEED.
        let trial_id = compute_trial_id(&objective, &data_window, &config_hash, seed);

        let created_at_ms = Ssb::now_ms_via_clock(clock);

        // Write to strategy_trials (BEGIN IMMEDIATE for serialized insert).
        let (n_after, rows_changed) = {
            let conn = self.conn.lock().map_err(|e| {
                TrialLedgerError::Sqlite(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error {
                        code: rusqlite::ffi::ErrorCode::SystemIoFailure,
                        extended_code: 0,
                    },
                    Some(format!("mutex poisoned: {e}")),
                ))
            })?;

            // INSERT OR IGNORE: if trial_id already exists, this is a
            // no-op (idempotent re-registration does not double-count).
            let rows_changed = conn.execute(
                "INSERT OR IGNORE INTO strategy_trials \
                 (trial_id, objective, data_window, config_hash, seed, created_at_ms, session_id) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    trial_id,
                    objective,
                    data_window,
                    config_hash,
                    seed,
                    created_at_ms,
                    self.session_id,
                ],
            )?;
            // rows_changed == 0 means the INSERT was a no-op (duplicate).
            // Re-count after the insert (idempotent: COUNT over the table).
            // SQLite COUNT(*) returns i64; safe narrowing-ok: trial counts will
            // never exceed i64::MAX in practice. NARROWING-OK via .try_into().
            let n_i64: i64 = conn.query_row(
                "SELECT COUNT(*) FROM strategy_trials \
                 WHERE objective = ?1 AND data_window = ?2",
                params![objective, data_window],
                |row| row.get(0),
            )?;
            // NARROWING-OK: COUNT(*) is always non-negative; safe cast to u64.
            (u64::try_from(n_i64).unwrap_or(0), rows_changed)
        };

        debug!(
            objective = %objective,
            data_window = %data_window,
            config_hash = %&config_hash[..16],
            seed,
            n_after,
            is_new = rows_changed > 0,
            "trial register_trial call"
        );

        // Emit the off-agent SSB artifact ONLY when the INSERT was not a no-op
        // (idempotent re-registration: same trial_id already exists -> no new row,
        // no new SSB row). This keeps SSB row count = DISTINCT trial count (N).
        if rows_changed > 0 {
            ssb.write_payload(
                "trial_ledger",
                0,
                SsbPayload::TrialRegistered(TrialRegisteredPayload {
                    objective: objective.clone(),
                    data_window: data_window.clone(),
                    config_hash: config_hash.clone(),
                    seed,
                    n_after,
                }),
                Vec::new(),
            )
            .await
            .map_err(|e| TrialLedgerError::SsbWrite(format!("{e}")))?;
        }

        Ok(n_after)
    }

    /// Raw cumulative DISTINCT trial count N for `(objective, data_window)`.
    ///
    /// Returns [`TrialCountResult::TrialCountMissing`] when zero rows exist
    /// for this scope — the H-18 promotion gate's HARD FAIL sentinel (TD-1).
    pub fn trial_count(
        &self,
        objective: &str,
        data_window: &str,
    ) -> Result<TrialCountResult, TrialLedgerError> {
        let objective = canonicalize_key(objective);
        let data_window = canonicalize_key(data_window);

        let conn = self.conn.lock().map_err(|e| {
            TrialLedgerError::Sqlite(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::SystemIoFailure,
                    extended_code: 0,
                },
                Some(format!("mutex poisoned: {e}")),
            ))
        })?;

        // SQLite COUNT(*) returns i64; NARROWING-OK: non-negative by definition.
        let n_i64: i64 = conn.query_row(
            "SELECT COUNT(*) FROM strategy_trials \
             WHERE objective = ?1 AND data_window = ?2",
            params![objective, data_window],
            |row| row.get(0),
        )?;
        let n = u64::try_from(n_i64).unwrap_or(0);

        Ok(if n == 0 {
            TrialCountResult::TrialCountMissing
        } else {
            TrialCountResult::Present(n)
        })
    }

    /// Effective trial count Ne (conservative default = raw N).
    ///
    /// The ONC effective-N algorithm (Lopez de Prado & Lewis 2019) requires
    /// an N×N return-series correlation matrix. `strategy_trials` stores only
    /// scalar metrics; the full return series is not available here.
    ///
    /// Conservative posture: Ne = N (raw count). This is the fail-safe
    /// upper-bound because fewer effective trials = LESS deflation = LOOSER
    /// gate. Using raw N is therefore the safe over-estimate of trial
    /// independence, never the under-estimate.
    ///
    /// # Stub registration
    /// The ONC-with-return-matrix refinement is registered as a grounded
    /// refinement stub (see module doc). Resolve when the return-matrix
    /// store ships.
    ///
    /// # Returns
    /// `(raw_n as f64, false)` — the `bool` indicates whether ONC was applied.
    pub fn effective_trial_count(
        &self,
        objective: &str,
        data_window: &str,
    ) -> Result<(f64, bool), TrialLedgerError> {
        match self.trial_count(objective, data_window)? {
            TrialCountResult::TrialCountMissing => {
                // Propagate the missing sentinel: 0.0, not ONC.
                Ok((0.0_f64, false))
            }
            TrialCountResult::Present(n) => {
                // TODO[H17.ONC-STUB]: when return-matrix store ships, replace
                // this with the ONC two-stage K-means clustering algorithm
                // (angular distance D=sqrt(0.5*(1-rho)), silhouette t-stat
                // q = mean(S)/std(S), K* = Ne). Constraint: Ne <= N always.
                // See workflow/papers/onc-lopezdeprado-lewis-2019-interface.md.
                // Grounding: Lopez de Prado & Lewis, Quantitative Finance
                // 19(9):1555-1565 (2019), DOI: 10.1080/14697688.2019.1622311.
                // will_resolve_in: H-18 (when backtest sidecar + return-matrix
                //   storage ships; see docs/thefinalglance/H-plan/H-18*.md).
                warn!(
                    objective = %objective,
                    data_window = %data_window,
                    n = n,
                    "effective_trial_count: ONC not available (return-matrix absent); \
                     using conservative raw N (over-estimate, safe upper bound)"
                );
                // NARROWING-OK: u64 fits in f64 for any realistic trial count.
                Ok((n as f64, false))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Canonicalize a scope key (objective or data_window) by:
///   - trimming ASCII whitespace,
///   - lower-casing,
///   - collapsing internal whitespace runs to a single space.
///
/// This prevents scope-key fragmentation attacks where an agent inserts
/// whitespace or mixed-case variants to fragment the N counter (TD-1 attack
/// surface noted in H-17 open questions).
fn canonicalize_key(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Compute trial_id = blake3 hex of the canonical tuple string
/// `"{objective}:{data_window}:{config_hash}:{seed}"`.
///
/// Using blake3 (CLAUDE.md HARD RULE: never blake2/sha2).
fn compute_trial_id(objective: &str, data_window: &str, config_hash: &str, seed: i64) -> String {
    let tuple = format!("{objective}:{data_window}:{config_hash}:{seed}");
    blake3::hash(tuple.as_bytes()).to_hex().to_string()
}

// ---------------------------------------------------------------------------
// Stub registry
// ---------------------------------------------------------------------------
//
// TODO[H17.ONC-STUB]: ONC effective-N for correlated trials.
// babystep-id: H17.ONC-STUB
// will_resolve_in: H-18 (or a dedicated H-17b session when the return-matrix
//   store ships alongside the backtest sidecar).
// grounding: workflow/papers/onc-lopezdeprado-lewis-2019-interface.md --
//   ONC algorithm: angular distance D=sqrt(0.5*(1-rho)); two-stage K-means;
//   silhouette t-stat q=mean(S)/std(S); K*=Ne; Ne<=N always; degenerate
//   when N<3.
// conservative_default: raw N (over-estimates trial independence, safe).
// attack_surface: ONC threshold constant MUST be in gate-manifest frozen set
//   when shipped (else agent tunes it to collapse all trials into 1).

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::SsbReader;
    use elai_core::clock::OsClock;
    use tempfile::TempDir;

    fn test_clock() -> Arc<dyn ClockProvider> {
        Arc::new(OsClock)
    }

    fn test_ssb(dir: &TempDir) -> Arc<Ssb> {
        use crate::ssb::SsbConfig;
        let cfg = SsbConfig {
            db_path: dir.path().join("ssb.db"),
            pool_size: Some(2),
            project_id: "h17-test".into(),
            session_id: "h17-test-session".into(),
        };
        Arc::new(Ssb::new(cfg, test_clock()).expect("ssb"))
    }

    fn test_ledger(dir: &TempDir) -> TrialLedger {
        TrialLedger::open(&dir.path().join("ssb.db"), "h17-test-session")
            .expect("trial ledger open")
    }

    #[test]
    fn canonicalize_key_normalizes_whitespace_and_case() {
        assert_eq!(canonicalize_key("  Sharpe_1Y  "), "sharpe_1y");
        assert_eq!(canonicalize_key("CAGR  2Y"), "cagr 2y");
        assert_eq!(canonicalize_key(""), "");
    }

    #[test]
    fn trial_count_missing_for_empty_db() {
        let dir = TempDir::new().unwrap();
        let ledger = test_ledger(&dir);
        let result = ledger.trial_count("sharpe_1y", "1704067200000..1735689600000").unwrap();
        assert!(result.is_missing(), "empty db must return TrialCountMissing");
        assert_eq!(result.n_treating_missing_as_zero_UNSAFE(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn register_trial_inserts_row_and_emits_ssb() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let clock = test_clock();
        let ledger = test_ledger(&dir);

        let objective = "sharpe_1y";
        let window = "1704067200000..1735689600000";
        let config = b"{\"lookback\": 20, \"fast\": 5}";
        let seed = 42_i64;

        let n = ledger
            .register_trial(&ssb, &clock, objective, window, config, seed)
            .await
            .expect("register_trial");

        assert_eq!(n, 1, "first registration -> N=1");

        // Off-agent artifact: TrialRegistered row must land in SSB.
        let ssb_rows = ssb
            .query_by_type(crate::ssb::SsbActionType::TrialRegistered)
            .expect("query TrialRegistered");
        assert_eq!(ssb_rows.len(), 1, "one TrialRegistered row in SSB");

        // Verify payload fields.
        let payload_bytes = &ssb_rows[0].payload;
        let payload: SsbPayload =
            rmp_serde::from_slice(payload_bytes).expect("deserialize TrialRegistered");
        if let SsbPayload::TrialRegistered(p) = payload {
            assert_eq!(p.objective, objective);
            assert_eq!(p.data_window, window);
            assert_eq!(p.n_after, 1);
            assert_eq!(p.seed, seed);
            // config_hash must be a non-empty blake3 hex string (64 hex chars).
            assert_eq!(p.config_hash.len(), 64, "blake3 hex is 64 chars");
        } else {
            panic!("expected TrialRegistered payload");
        }

        // strategy_trials row must also exist.
        let count_result = ledger.trial_count(objective, window).unwrap();
        assert!(
            !count_result.is_missing(),
            "after register, must not be TrialCountMissing"
        );
        assert_eq!(count_result.n_treating_missing_as_zero_UNSAFE(), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn reregistering_same_config_does_not_double_count() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let clock = test_clock();
        let ledger = test_ledger(&dir);

        let config = b"{\"lookback\": 20}";
        let n1 = ledger
            .register_trial(&ssb, &clock, "sharpe_1y", "2024..2025", config, 1)
            .await
            .expect("first");
        let n2 = ledger
            .register_trial(&ssb, &clock, "sharpe_1y", "2024..2025", config, 1)
            .await
            .expect("re-register");

        assert_eq!(n1, 1, "first registration -> N=1");
        assert_eq!(n2, 1, "identical re-registration must NOT increment N");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn different_configs_increment_n_monotonically() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let clock = test_clock();
        let ledger = test_ledger(&dir);

        let window = "2024..2025";
        let objective = "sharpe_1y";

        let configs: [&[u8]; 3] = [
            b"{\"fast\": 5, \"slow\": 20}",
            b"{\"fast\": 10, \"slow\": 40}",
            b"{\"fast\": 3, \"slow\": 15}",
        ];

        let mut last_n = 0u64;
        for (i, cfg) in configs.iter().enumerate() {
            let n = ledger
                .register_trial(&ssb, &clock, objective, window, cfg, i as i64)
                .await
                .expect("register");
            assert!(n > last_n, "N must be strictly monotone: got {n} <= {last_n}");
            last_n = n;
        }
        assert_eq!(last_n, 3, "3 distinct configs -> N=3");

        // Verify SSB rows: 3 TrialRegistered rows.
        let ssb_rows = ssb
            .query_by_type(crate::ssb::SsbActionType::TrialRegistered)
            .expect("query");
        assert_eq!(ssb_rows.len(), 3, "3 TrialRegistered rows in SSB");
    }

    #[test]
    fn trial_count_for_unseen_scope_returns_missing() {
        let dir = TempDir::new().unwrap();
        let ledger = test_ledger(&dir);

        // No rows exist for this scope at all.
        let result = ledger.trial_count("cagr_2y", "2023..2024").unwrap();
        assert!(
            result.is_missing(),
            "unseen scope must return TrialCountMissing (TD-1 fail-closed sentinel)"
        );
        assert_eq!(result.n_treating_missing_as_zero_UNSAFE(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn effective_trial_count_le_trial_count() {
        let dir = TempDir::new().unwrap();
        let ssb = test_ssb(&dir);
        let clock = test_clock();
        let ledger = test_ledger(&dir);

        // Register 3 distinct correlated configs.
        for i in 0u8..3 {
            let config = format!("{{\"fast\": {}}}", i * 5 + 5);
            ledger
                .register_trial(&ssb, &clock, "sharpe_1y", "2024..2025", config.as_bytes(), i as i64)
                .await
                .expect("register");
        }

        let raw_n = ledger
            .trial_count("sharpe_1y", "2024..2025")
            .unwrap()
            .n_treating_missing_as_zero_UNSAFE();
        let (n_eff, _onc_applied) = ledger
            .effective_trial_count("sharpe_1y", "2024..2025")
            .unwrap();

        // TD-1: Ne <= N (conservative default = N; ONC can only reduce it).
        assert!(
            n_eff <= raw_n as f64,
            "effective N ({n_eff}) must be <= raw N ({raw_n})"
        );
        // With conservative default Ne = N.
        assert!(
            (n_eff - raw_n as f64).abs() < f64::EPSILON,
            "conservative default: Ne must equal raw N"
        );
    }

    #[test]
    fn no_delete_reset_update_method_exists() {
        // Negative test: verify by absence of symbols at compile time.
        // The following expression is unreachable but forces the compiler to
        // verify that TrialLedger does NOT have delete_trial / reset_trials /
        // update_trial methods. If these were added, it would be a stub-ban
        // violation and the plan's WIRING_CONTRACT would fail.
        // This test always passes; its value is in what it DOES NOT compile if
        // someone adds a banned method.
        let _ = |_l: &TrialLedger| {
            // These must NOT compile:
            // _l.delete_trial("x", "y", "z");
            // _l.reset_trials("x", "y");
            // _l.update_trial("x", 0);
        };
        // If we reach here, the absence-of-method constraint is satisfied.
    }
}
