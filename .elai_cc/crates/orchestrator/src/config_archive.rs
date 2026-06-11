//! B-23 Phase 4H Group B — DGM ConfigVariant archive (4H.3).
//!
//! Stores [`ConfigVariant`] rows in a SQLite-backed archive (one DB per
//! project, expected path `.elai_cc/projects/{id}/self_mod.db`). Provides
//! DGM-style sigmoid-weighted parent selection (lambda=10, alpha_0=0.5),
//! per-mode unique ACTIVE invariant, archive overflow with deceptive-dip
//! resilience (ancestor-of-active variants survive pruning), and revert
//! protocol.
//!
//! Distinct from the Layer 1A `config_archive` table in `rules.db` which
//! tracks applied-proposal history (batch-3C). The DGM archive lives in
//! its own DB to avoid coupling.
//!
//! Spec: docs/specs/guide/steps/phase-4H.txt section 4H.3 (config archive)
//! and 4H.7 (parent-selection mode-filtering).
//! Grounding: workflow/papers/dgm-interface.md (section 2.3 formulas).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;

use elai_core::EntropySource;

use crate::clade_metaproductivity::{clade_metaproductivity, SelectionKey};
use crate::self_mod_layer1a::PipelineMode;
use crate::self_mod_types::{ConfigVariant, VariantId, VariantStatus};

// ---------------------------------------------------------------------------
// Named constants (spec phase-4H.txt:167-174 + dgm-interface.md)
// ---------------------------------------------------------------------------

/// Sigmoid sharpness (DGM section 2.3).
pub const DGM_LAMBDA: f32 = 10.0;
/// Sigmoid midpoint (50% success rate).
pub const DGM_ALPHA_0: f32 = 0.5;
/// Per-mode rolling cap on archived variants (OUR DESIGN, spec phase-4H.txt:413).
pub const DEFAULT_ARCHIVE_CAP_PER_MODE: u32 = 50;

// ---------------------------------------------------------------------------
// DGM formulas (pure)
// ---------------------------------------------------------------------------

/// `s_i = 1 / (1 + exp(-lambda * (alpha - alpha_0)))` — DGM section 2.3.
/// Always strictly positive (sigmoid lower bound is asymptotic 0, never reached
/// for finite alpha).
#[inline]
pub fn sigmoid_perf_score(alpha: f32) -> f32 {
    1.0 / (1.0 + (-DGM_LAMBDA * (alpha - DGM_ALPHA_0)).exp())
}

/// `h_i = 1 / (1 + n_i)` — DGM exploration bonus.
#[inline]
pub fn exploration_bonus(children_count: usize) -> f32 {
    1.0 / (1.0 + children_count as f32)
}

/// Returns the unnormalised weight `w_i = s_i * h_i` for each variant.
pub fn parent_selection_weights(variants: &[ConfigVariant]) -> Vec<f32> {
    variants
        .iter()
        .map(|v| sigmoid_perf_score(v.score) * exploration_bonus(v.children_count))
        .collect()
}

/// Returns the normalised selection probabilities `p_i = w_i / sum(w_j)`.
/// All probabilities are strictly positive (DGM invariant). Empty input
/// returns an empty vector.
pub fn parent_selection_probabilities(variants: &[ConfigVariant]) -> Vec<f32> {
    if variants.is_empty() {
        return Vec::new();
    }
    let weights = parent_selection_weights(variants);
    let sum: f32 = weights.iter().sum();
    if sum <= 0.0 {
        // Defensive — sigmoid is strictly positive so this branch is unreachable
        // for any real input. Falls back to uniform if it ever fires.
        let n = weights.len() as f32;
        return weights.iter().map(|_| 1.0 / n).collect();
    }
    weights.iter().map(|w| w / sum).collect()
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ConfigArchiveError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("active variant already exists for mode {0}")]
    ActiveAlreadyExists(String),
    #[error("variant not found: {0}")]
    NotFound(String),
    #[error("invalid score {0}: must be in [0.0, 1.0]")]
    InvalidScore(f32),
}

// ---------------------------------------------------------------------------
// Archive (SQLite-backed)
// ---------------------------------------------------------------------------

const CREATE_TABLE_SQL: &str = "\
CREATE TABLE IF NOT EXISTS config_variants (
    variant_id      TEXT PRIMARY KEY,
    parent_id       TEXT,
    score           REAL NOT NULL CHECK(score >= 0.0 AND score <= 1.0),
    children_count  INTEGER NOT NULL DEFAULT 0,
    config_blob     BLOB NOT NULL,
    created_at      INTEGER NOT NULL,
    pipeline_mode   TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active'
        CHECK(status IN ('active', 'archived', 'reverted')),
    pipeline_outcomes_json TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_cv_mode_status ON config_variants(pipeline_mode, status);
CREATE INDEX IF NOT EXISTS idx_cv_score ON config_variants(score DESC);
CREATE INDEX IF NOT EXISTS idx_cv_parent ON config_variants(parent_id);
-- Group B audit BLOCK: enforce one-active-per-mode at the DB level so the
-- invariant survives any code path that bypasses the application guard
-- (e.g. concurrent processes, INSERT OR REPLACE, manual SQL).
CREATE UNIQUE INDEX IF NOT EXISTS idx_one_active_per_mode
    ON config_variants(pipeline_mode) WHERE status = 'active';
";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigArchiveConfig {
    pub archive_cap_per_mode: u32,
}

impl Default for ConfigArchiveConfig {
    fn default() -> Self {
        Self {
            archive_cap_per_mode: DEFAULT_ARCHIVE_CAP_PER_MODE,
        }
    }
}

/// Async-safe handle to the DGM ConfigVariant archive.
///
/// The entropy source is passed to [`ConfigArchive::select_parent_with`] /
/// [`ConfigArchive::sample_variant`] explicitly so the caller controls
/// determinism (production: `OsEntropy`, test: `SeededRng`). This complies
/// with the entropy-quarantine rule (systemPatterns.md, locked 2026-04-05):
/// no module-level `getrandom`/`rand::rng()` calls.
pub struct ConfigArchive {
    conn: Arc<AsyncMutex<Connection>>,
    config: ConfigArchiveConfig,
}

impl ConfigArchive {
    /// Open or create the archive at `db_path`. Runs `CREATE TABLE IF NOT
    /// EXISTS` on every open so cold starts and warm reopens behave the same.
    pub async fn open(db_path: &Path) -> Result<Self, ConfigArchiveError> {
        Self::open_with_config(db_path, ConfigArchiveConfig::default()).await
    }

    pub async fn open_with_config(
        db_path: &Path,
        config: ConfigArchiveConfig,
    ) -> Result<Self, ConfigArchiveError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(CREATE_TABLE_SQL)?;
        Ok(Self {
            conn: Arc::new(AsyncMutex::new(conn)),
            config,
        })
    }

    /// Construct from an existing connection. Useful for tests or for
    /// embedding in a multi-table app DB.
    pub fn from_connection(conn: Connection, config: ConfigArchiveConfig) -> Result<Self, ConfigArchiveError> {
        conn.execute_batch(CREATE_TABLE_SQL)?;
        Ok(Self {
            conn: Arc::new(AsyncMutex::new(conn)),
            config,
        })
    }

    pub fn config(&self) -> &ConfigArchiveConfig {
        &self.config
    }

    /// Insert a new variant. Enforces the per-mode unique-ACTIVE invariant
    /// at TWO layers:
    ///   1. Application-level COUNT-and-check before insert (returns
    ///      [`ConfigArchiveError::ActiveAlreadyExists`] when violated).
    ///   2. SQLite partial unique index `idx_one_active_per_mode` (returns
    ///      a UNIQUE constraint violation surfaced as
    ///      [`ConfigArchiveError::Sqlite`]) — enforced even when callers
    ///      bypass the app guard via concurrent processes or manual SQL.
    ///
    /// Uses plain `INSERT` (NOT `INSERT OR REPLACE`) so the audit trail
    /// stays append-only — overwriting an existing variant_id requires the
    /// caller to explicitly delete or call [`Self::revert`] first.
    pub async fn insert_variant(&self, v: &ConfigVariant) -> Result<(), ConfigArchiveError> {
        if !(0.0..=1.0).contains(&v.score) {
            return Err(ConfigArchiveError::InvalidScore(v.score));
        }
        let conn = self.conn.lock().await;
        if v.status == VariantStatus::Active {
            let existing: i64 = conn.query_row(
                "SELECT COUNT(*) FROM config_variants WHERE pipeline_mode = ?1 AND status = 'active'",
                params![v.pipeline_mode.as_str()],
                |row| row.get(0),
            )?;
            if existing > 0 {
                return Err(ConfigArchiveError::ActiveAlreadyExists(
                    v.pipeline_mode.as_str().to_string(),
                ));
            }
        }
        let outcomes_json = serde_json::to_string(&v.pipeline_outcomes).unwrap_or_else(|_| "[]".into());
        conn.execute(
            "INSERT INTO config_variants
             (variant_id, parent_id, score, children_count, config_blob,
              created_at, pipeline_mode, status, pipeline_outcomes_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                v.variant_id,
                v.parent_id,
                v.score,
                v.children_count as i64,
                v.config_blob,
                v.created_at as i64,
                v.pipeline_mode.as_str(),
                v.status.as_str(),
                outcomes_json,
            ],
        )?;
        Ok(())
    }

    /// List all variants for a given pipeline mode (any status).
    pub async fn list_variants(&self, mode: PipelineMode) -> Result<Vec<ConfigVariant>, ConfigArchiveError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT variant_id, parent_id, score, children_count, config_blob,
                    created_at, pipeline_mode, status, pipeline_outcomes_json
             FROM config_variants WHERE pipeline_mode = ?1
             ORDER BY score DESC, variant_id ASC",
        )?;
        let rows = stmt.query_map(params![mode.as_str()], row_to_variant)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Fetch a single variant by id.
    pub async fn get_variant(&self, id: &str) -> Result<Option<ConfigVariant>, ConfigArchiveError> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT variant_id, parent_id, score, children_count, config_blob,
                    created_at, pipeline_mode, status, pipeline_outcomes_json
             FROM config_variants WHERE variant_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], row_to_variant)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// DGM parent selection — pick a variant from the given mode according
    /// to the sigmoid * exploration weighted distribution.
    ///
    /// Convenience wrapper that uses [`elai_core::OsEntropy`]. For tests or
    /// reproducible runs use [`Self::select_parent_with`] with a
    /// `SeededRng`.
    pub async fn select_parent(&self, mode: PipelineMode) -> Result<VariantId, ConfigArchiveError> {
        let mut entropy = elai_core::OsEntropy;
        self.select_parent_with(mode, &mut entropy).await
    }

    /// Like [`Self::select_parent`] but takes an explicit
    /// [`EntropySource`] so callers can replay deterministic sequences
    /// under DST. Returns the empty string when the archive contains no
    /// variants for the mode.
    pub async fn select_parent_with<E: EntropySource + ?Sized>(
        &self,
        mode: PipelineMode,
        entropy: &mut E,
    ) -> Result<VariantId, ConfigArchiveError> {
        let variants = self.list_variants(mode).await?;
        Ok(sample_variant(&variants, entropy))
    }

    /// H-46 — CMP-weighted parent selection (production path).
    ///
    /// Selects the next parent to self-modify by the node's
    /// **Clade-Metaproductivity** (CMP) rather than the immediate sigmoid
    /// score (HGM 2510.21614: expand the lineage that produced the most-
    /// improving descendants, not the highest single-node score).
    ///
    /// `fitness_deltas` is a map of `variant_id -> execution-grounded fitness
    /// delta` (H-43 signal). A node with no measured descendants falls back to
    /// `ImmediateScoreFallback` (sigmoid * exploration_bonus).
    ///
    /// `seed` is the `ELAI_SEED` value forwarded into the `CladeSelected` SSB
    /// row for GD-2 reproducibility.
    ///
    /// Returns `(selected_id, clade_scores)`. The caller emits the
    /// `CladeSelected` SSB row using the `clade_scores` slice.
    pub async fn select_parent_with_cmp<E: EntropySource + ?Sized>(
        &self,
        mode: PipelineMode,
        fitness_deltas: &HashMap<VariantId, f64>,
        entropy: &mut E,
        force_immediate_score: bool,
    ) -> Result<(VariantId, Vec<crate::clade_metaproductivity::CladeScore>), ConfigArchiveError> {
        let variants = self.list_variants(mode).await?;
        if variants.is_empty() {
            return Ok((String::new(), Vec::new()));
        }

        let clade_scores = if force_immediate_score {
            // R2 broken-arm path: ignore the CMP; use immediate sigmoid score.
            // This arm is ONLY for falsifiability contract tests (it reproduces
            // the dead-end selection the CMP key avoids).
            variants
                .iter()
                .map(|v| {
                    let s = f64::from(sigmoid_perf_score(v.score));
                    crate::clade_metaproductivity::CladeScore {
                        cmp: s,
                        clade_size: 0,
                        immediate_score: s,
                        key: SelectionKey::ImmediateScoreFallback,
                    }
                })
                .collect::<Vec<_>>()
        } else {
            clade_metaproductivity(&variants, fitness_deltas)
        };

        // Build CMP-weighted probabilities for the CDF sampler.
        let weights: Vec<f32> = clade_scores
            .iter()
            .zip(variants.iter())
            .map(|(cs, v)| {
                // For CMP key: weight = cmp * exploration_bonus (so unexplored
                //   nodes still get a diversity bonus even with a good lineage).
                // For fallback key: weight = immediate_score * exploration_bonus
                //   (identical to the original sigmoid * exploration path).
                let base = cs.cmp as f32;
                base * exploration_bonus(v.children_count)
            })
            .collect();

        let selected = sample_variant_weights(&variants, &weights, entropy);
        Ok((selected, clade_scores))
    }

    /// Enforce the per-mode archive cap. Drops oldest archived variants by
    /// (score ASC, created_at ASC) but ALWAYS keeps:
    ///   - status='active' rows
    ///   - any variant on the ancestor chain of an active variant (deceptive
    ///     dip resilience, DGM section 9A, iterations 4 + 56)
    ///
    /// The entire read-compute-delete sequence runs under a single mutex
    /// hold + a SQLite IMMEDIATE transaction so concurrent inserts cannot
    /// race the cap enforcement (Group B audit BLOCK fix).
    pub async fn enforce_cap(&self, mode: PipelineMode) -> Result<usize, ConfigArchiveError> {
        let cap = self.config.archive_cap_per_mode as usize;
        let mut conn = self.conn.lock().await;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let variants = list_variants_with_conn(&tx, mode)?;
        if variants.len() <= cap {
            tx.commit()?;
            return Ok(0);
        }
        let protected = ancestors_of_active(&variants);

        let mut prunable: Vec<&ConfigVariant> = variants
            .iter()
            .filter(|v| {
                v.status != VariantStatus::Active && !protected.contains(v.variant_id.as_str())
            })
            .collect();
        // Lowest-score first — highest archive value last.
        prunable.sort_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });
        let to_drop = variants.len().saturating_sub(cap);
        let drop_ids: Vec<String> = prunable
            .into_iter()
            .take(to_drop)
            .map(|v| v.variant_id.clone())
            .collect();
        if drop_ids.is_empty() {
            tx.commit()?;
            return Ok(0);
        }
        let placeholders = drop_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM config_variants WHERE variant_id IN ({})", placeholders);
        let params: Vec<&dyn rusqlite::ToSql> =
            drop_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        tx.execute(&sql, params.as_slice())?;
        tx.commit()?;
        Ok(drop_ids.len())
    }

    /// Revert the active variant for `mode` — mark it `reverted` and promote
    /// the highest-score archived variant of the same mode to active.
    /// Returns the id of the restored variant (or `None` if nothing to
    /// restore).
    pub async fn revert(&self, mode: PipelineMode) -> Result<Option<VariantId>, ConfigArchiveError> {
        let conn = self.conn.lock().await;
        let active_id: Option<String> = conn
            .query_row(
                "SELECT variant_id FROM config_variants WHERE pipeline_mode = ?1 AND status = 'active'",
                params![mode.as_str()],
                |row| row.get(0),
            )
            .ok();
        if let Some(id) = active_id {
            conn.execute(
                "UPDATE config_variants SET status = 'reverted' WHERE variant_id = ?1",
                params![id],
            )?;
        }
        let next: Option<String> = conn
            .query_row(
                "SELECT variant_id FROM config_variants
                 WHERE pipeline_mode = ?1 AND status = 'archived'
                 ORDER BY score DESC, created_at DESC LIMIT 1",
                params![mode.as_str()],
                |row| row.get(0),
            )
            .ok();
        if let Some(ref restore_id) = next {
            conn.execute(
                "UPDATE config_variants SET status = 'active' WHERE variant_id = ?1",
                params![restore_id],
            )?;
        }
        Ok(next)
    }
}

/// Uniform random `f32` in `[0, 1)` sourced from the supplied
/// [`EntropySource`]. Used for inverse-CDF parent sampling. The top 24
/// bits of a `u64` form the f32 mantissa exactly so the result is uniform
/// over the representable values in `[0, 1)`.
fn uniform_unit_f32<E: EntropySource + ?Sized>(entropy: &mut E) -> f32 {
    let raw = entropy.next_u64();
    let top24 = (raw >> 40) as u32;
    (top24 as f32) / ((1u32 << 24) as f32)
}

/// Pure parent-selection over an in-memory variant slice. Pulled out of
/// [`ConfigArchive::select_parent_with`] so callers can sample without
/// hitting SQLite (e.g., in deterministic tests).
pub fn sample_variant<E: EntropySource + ?Sized>(
    variants: &[ConfigVariant],
    entropy: &mut E,
) -> VariantId {
    if variants.is_empty() {
        return String::new();
    }
    let weights = parent_selection_weights(variants);
    sample_variant_weights(variants, &weights, entropy)
}

/// Like [`sample_variant`] but accepts a pre-computed weights slice.
/// Used by the H-46 CMP path which computes its own weight vector.
/// Parallel to `variants` — panics (debug) if lengths differ.
pub fn sample_variant_weights<E: EntropySource + ?Sized>(
    variants: &[ConfigVariant],
    weights: &[f32],
    entropy: &mut E,
) -> VariantId {
    if variants.is_empty() {
        return String::new();
    }
    debug_assert_eq!(
        variants.len(),
        weights.len(),
        "sample_variant_weights: variants and weights must have the same length"
    );
    let total: f32 = weights.iter().sum();
    if total <= 0.0 {
        return variants[0].variant_id.clone();
    }
    let r = uniform_unit_f32(entropy);
    let mut acc = 0.0f32;
    for (i, w) in weights.iter().enumerate() {
        acc += w / total;
        if r <= acc {
            return variants[i].variant_id.clone();
        }
    }
    variants[variants.len() - 1].variant_id.clone()
}

/// Read all variants for `mode` using an existing rusqlite handle (a
/// `&Connection` or `&Transaction`). Used by [`ConfigArchive::enforce_cap`]
/// to keep the read+delete inside one transaction and avoid TOCTOU.
fn list_variants_with_conn(
    conn: &rusqlite::Connection,
    mode: PipelineMode,
) -> Result<Vec<ConfigVariant>, ConfigArchiveError> {
    let mut stmt = conn.prepare(
        "SELECT variant_id, parent_id, score, children_count, config_blob,
                created_at, pipeline_mode, status, pipeline_outcomes_json
         FROM config_variants WHERE pipeline_mode = ?1
         ORDER BY score DESC, variant_id ASC",
    )?;
    let rows = stmt.query_map(params![mode.as_str()], row_to_variant)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn row_to_variant(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConfigVariant> {
    let mode_str: String = row.get(6)?;
    let pipeline_mode = match mode_str.as_str() {
        "surgical" => PipelineMode::Surgical,
        "greenfield" => PipelineMode::Greenfield,
        "oracle" => PipelineMode::Oracle,
        other => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unknown pipeline_mode {other}"),
                )),
            ))
        }
    };
    let status_str: String = row.get(7)?;
    let status = match status_str.as_str() {
        "active" => VariantStatus::Active,
        "archived" => VariantStatus::Archived,
        "reverted" => VariantStatus::Reverted,
        other => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                7,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unknown status {other}"),
                )),
            ))
        }
    };
    let outcomes_json: String = row.get(8)?;
    let pipeline_outcomes: Vec<String> =
        serde_json::from_str(&outcomes_json).unwrap_or_default();
    let created_at: i64 = row.get(5)?;
    let children_count: i64 = row.get(3)?;
    Ok(ConfigVariant {
        variant_id: row.get(0)?,
        parent_id: row.get(1)?,
        score: row.get(2)?,
        children_count: children_count.max(0) as usize,
        config_blob: row.get(4)?,
        created_at: created_at.max(0) as u64,
        pipeline_outcomes,
        pipeline_mode,
        status,
    })
}

/// Returns the set of `variant_id`s that are EITHER active or on the ancestor
/// chain of an active variant for the same mode. Per DGM section 9A
/// (deceptive dip), these must survive archive overflow pruning.
fn ancestors_of_active(variants: &[ConfigVariant]) -> std::collections::HashSet<String> {
    use std::collections::HashSet;
    let id_to_parent: std::collections::HashMap<&str, Option<&str>> = variants
        .iter()
        .map(|v| (v.variant_id.as_str(), v.parent_id.as_deref()))
        .collect();
    let mut protected: HashSet<String> = HashSet::new();
    for v in variants.iter().filter(|v| v.status == VariantStatus::Active) {
        let mut cursor = Some(v.variant_id.as_str());
        while let Some(id) = cursor {
            if !protected.insert(id.to_string()) {
                break; // already walked
            }
            cursor = id_to_parent.get(id).copied().flatten();
        }
    }
    protected
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variant(id: &str, parent: Option<&str>, score: f32, children: usize, mode: PipelineMode, status: VariantStatus) -> ConfigVariant {
        ConfigVariant {
            variant_id: id.to_string(),
            parent_id: parent.map(|s| s.to_string()),
            score,
            children_count: children,
            config_blob: vec![],
            created_at: 0,
            pipeline_outcomes: vec![],
            pipeline_mode: mode,
            status,
        }
    }

    #[test]
    fn sigmoid_at_midpoint() {
        let s = sigmoid_perf_score(0.5);
        assert!((s - 0.5).abs() < 1e-6);
    }

    #[test]
    fn sigmoid_strictly_positive() {
        for alpha in [0.0_f32, 0.1, 0.5, 0.9, 1.0] {
            assert!(sigmoid_perf_score(alpha) > 0.0);
        }
    }

    #[test]
    fn exploration_bonus_decreases() {
        assert!(exploration_bonus(0) > exploration_bonus(1));
        assert!(exploration_bonus(1) > exploration_bonus(10));
    }

    #[test]
    fn parent_selection_probabilities_sum_to_one() {
        let v = vec![
            variant("a", None, 0.3, 0, PipelineMode::Surgical, VariantStatus::Archived),
            variant("b", None, 0.6, 1, PipelineMode::Surgical, VariantStatus::Archived),
            variant("c", None, 0.9, 5, PipelineMode::Surgical, VariantStatus::Active),
        ];
        let p = parent_selection_probabilities(&v);
        assert_eq!(p.len(), 3);
        let total: f32 = p.iter().sum();
        assert!((total - 1.0).abs() < 1e-5);
        for x in &p {
            assert!(*x > 0.0);
        }
    }

    #[test]
    fn ancestors_of_active_walks_chain() {
        let v = vec![
            variant("A", None, 0.5, 1, PipelineMode::Surgical, VariantStatus::Archived),
            variant("B", Some("A"), 0.3, 1, PipelineMode::Surgical, VariantStatus::Archived),
            variant("C", Some("B"), 0.7, 0, PipelineMode::Surgical, VariantStatus::Active),
            variant("X", None, 0.9, 0, PipelineMode::Surgical, VariantStatus::Archived),
        ];
        let protected = ancestors_of_active(&v);
        assert!(protected.contains("A"));
        assert!(protected.contains("B"));
        assert!(protected.contains("C"));
        assert!(!protected.contains("X"));
    }
}
