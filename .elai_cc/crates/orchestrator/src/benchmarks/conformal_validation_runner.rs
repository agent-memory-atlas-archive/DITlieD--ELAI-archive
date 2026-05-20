//! D-22: Conformal calibration validation under distribution shift.
//!
//! Verifies the D-16 UniCR (arXiv:2503.01345, T1) empirical refusal rate
//! stays within the user-set `epsilon` band across three orthogonal
//! distributions (in-domain, near-OOD, far-OOD). Gates the formal
//! distribution-free guarantee from UniCR §3 + Vovk et al. 2005 §2.2 by
//! measuring it on held-out fixtures whose category labels mark the
//! distribution bucket.
//!
//! # Runner shape
//!
//! `ConformalValidationRunner { calibration_corpus, validation_fixture }`
//! computes tau from the calibration corpus once, splits the validation
//! fixture into in_domain / near_ood / far_ood buckets via the per-row
//! `distribution` label, runs the pure
//! [`crate::conformal_calibration::decide`] predicate on every row, then
//! reports per-domain refusal rate + whether each domain landed inside
//! the `[target - epsilon, target + epsilon]` band. The aggregate verdict
//! is `Pass` iff every domain stayed in-band AND at least one row was
//! scored per domain (no class collapse).
//!
//! # Wiring discipline
//!
//! producer  : [`ConformalValidationRunner::run_all`] emits ONE
//!             [`crate::ssb::SsbActionType::ConformalValidationCompleted`]
//!             row at end of run via the [`ConformalValidationSsbEmitter`]
//!             trait. Mirrors the D-19 / D-21 single-row pattern.
//! consumer  : `elai bench conformal` CLI subcommand exits 0/1 to drive
//!             CI; the dashboard `ConformalValidationPanel` reads the row
//!             via `get_conformal_validation_recent` Tauri command.
//! observable: `elai bench conformal` against the bundled mini fixture
//!             must produce a JSON report + exactly one SSB row.
//!
//! # Fixture format
//!
//! One JSON object per line, with the schema captured by
//! [`ValidationFixtureRow`]:
//! ```json
//! {
//!   "row_id": "in_domain_0001",
//!   "distribution": "in_domain",
//!   "domain": "code",
//!   "predicted_score": 0.92,
//!   "ground_truth": true
//! }
//! ```
//!
//! `distribution` is one of `in_domain | near_ood | far_ood`. `domain`
//! mirrors [`crate::reasoning_template::Domain`] snake-case wire-labels so
//! the per-domain breakdown can be cross-referenced with D-7 canary
//! families if needed.
//!
//! # Anti-stale + zero-lazy
//!
//! - blake3 for fixture content hash (anti-stale rule).
//! - thiserror for errors; tracing for diagnostics.
//! - native async-fn in trait (no `async-trait` crate).
//! - No `.unwrap()` outside `#[cfg(test)]`. No silent swallows.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, info};

use crate::benchmarks::prmbench_runner::GatingVerdict;
use crate::conformal_calibration::{
    compute_tau, decide, CalibrationCorpus, ConformalError, RefusalDecision, DEFAULT_EPSILON,
};
use crate::reasoning_template::Domain;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// D-22 plan default minimum rows per distribution bucket. Below this the
/// verdict trips to `Fail` (no class collapse — matches the D-21 gate
/// philosophy).
pub const DEFAULT_MIN_ROWS_PER_DISTRIBUTION: u32 = 5;

// ---------------------------------------------------------------------------
// Public value types
// ---------------------------------------------------------------------------

/// Distribution bucket label — the 3 orthogonal task families the runner
/// validates against. Mirrors `serde(rename_all = "snake_case")` so fixture
/// authors write the bare snake-case form.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum DistributionBucket {
    InDomain,
    NearOod,
    FarOod,
}

impl DistributionBucket {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InDomain => "in_domain",
            Self::NearOod => "near_ood",
            Self::FarOod => "far_ood",
        }
    }

    /// Closed-set enumeration helper. Order matches the dashboard render
    /// order (low-shift to high-shift).
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::InDomain, Self::NearOod, Self::FarOod]
    }
}

/// Per-domain breakdown emitted alongside the aggregate report. `domain`
/// mirrors [`crate::reasoning_template::Domain`] (D-1 + D-7); `in_band`
/// is `true` iff `|refusal_rate - target| <= epsilon`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainBreakdown {
    /// Per-row [`Domain`] (snake-case wire-label).
    pub domain: Domain,
    /// Distribution bucket (in_domain / near_ood / far_ood) the row landed
    /// in. The same `domain` may appear under multiple buckets when the
    /// fixture authors deliberately probe cross-bucket coverage.
    pub bucket: DistributionBucket,
    /// Rows tested in this `(domain, bucket)` cell.
    pub n_tested: u32,
    /// Empirical refusal rate. Defined as `0.0` when `n_tested == 0`
    /// (denominator-zero clamp; the verdict separately requires
    /// `n_tested >= DEFAULT_MIN_ROWS_PER_DISTRIBUTION` to clear class
    /// collapse).
    pub refusal_rate: f64,
    /// `|refusal_rate - target| <= epsilon`.
    pub in_band: bool,
}

/// Aggregate report — one per [`ConformalValidationRunner::run_all`]
/// invocation. Carries the tau the corpus produced, the operating point
/// epsilon, the empirical refusal rate (across all rows), the target band
/// the gate compared against, and the per-domain breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalValidationReport {
    /// UNIX ms at run completion.
    pub ts_ms: i64,
    /// Tau computed from the calibration corpus.
    pub tau: f64,
    /// Operating point. Defaults to
    /// [`crate::conformal_calibration::DEFAULT_EPSILON`] (0.1).
    pub epsilon: f64,
    /// Aggregate empirical refusal rate across every fixture row scored.
    pub empirical_refusal_rate: f64,
    /// `[target - epsilon, target + epsilon]`. `target` is `epsilon` —
    /// per UniCR §3, the long-run empirical refusal rate should converge
    /// to the operating point.
    pub target_band: (f64, f64),
    /// Total rows the runner scored across all distributions.
    pub total: u32,
    /// Per-bucket aggregate refusal rate.
    pub per_bucket: BTreeMap<String, f64>,
    /// Per-domain rolled-up cells.
    pub per_domain: Vec<DomainBreakdown>,
    /// Aggregate verdict — `Pass` iff every bucket's empirical refusal
    /// rate is in-band AND every bucket has `>= min_rows_per_bucket` rows.
    pub gating_verdict: GatingVerdict,
    /// 64-hex blake3 of the validation fixture file contents.
    pub fixture_blake3: String,
}

impl ConformalValidationReport {
    /// Convenience predicate for callers that only care about pass/fail.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.gating_verdict == GatingVerdict::Pass
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConformalValidationError {
    /// I/O failure reading the fixture JSONL.
    #[error("conformal-validation fixture I/O error reading {path}: {source_msg}")]
    Io { path: String, source_msg: String },
    /// One line of the fixture failed to deserialise.
    #[error(
        "conformal-validation fixture malformed json at line {line_index} of {path}: {source_msg}"
    )]
    MalformedJson {
        path: String,
        line_index: usize,
        source_msg: String,
    },
    /// `distribution` outside the closed [`DistributionBucket`] set.
    #[error(
        "conformal-validation fixture invalid distribution '{value}' at line {line_index} of {path}"
    )]
    InvalidDistribution {
        path: String,
        line_index: usize,
        value: String,
    },
    /// `domain` outside the closed [`Domain`] set.
    #[error(
        "conformal-validation fixture invalid domain '{value}' at line {line_index} of {path}"
    )]
    InvalidDomain {
        path: String,
        line_index: usize,
        value: String,
    },
    /// Empty fixture (zero usable rows).
    #[error("conformal-validation fixture {path} contains zero usable rows")]
    EmptyFixture { path: String },
    /// Calibration-corpus failure (tau could not be computed).
    #[error("conformal-validation calibration error: {0}")]
    Calibration(#[from] ConformalError),
    /// SSB emitter failure on the final row write.
    #[error("conformal-validation ssb emit error: {0}")]
    SsbEmit(String),
}

// ---------------------------------------------------------------------------
// SSB emitter trait
// ---------------------------------------------------------------------------

/// Minimal trait the runner uses to write one end-of-run row. Production
/// callers wrap `Ssb::write_payload`; tests use
/// [`VecConformalValidationSsbEmitter`]. Native async-fn-in-trait per the
/// anti-stale-training rule.
pub trait ConformalValidationSsbEmitter: Send + Sync {
    fn emit_run_completed(
        &self,
        payload: crate::ssb::ConformalValidationCompletedPayload,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}

/// Test-only emitter that records every payload into an in-memory vector.
#[derive(Debug, Default)]
pub struct VecConformalValidationSsbEmitter {
    rows: tokio::sync::Mutex<Vec<crate::ssb::ConformalValidationCompletedPayload>>,
}

impl VecConformalValidationSsbEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<crate::ssb::ConformalValidationCompletedPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl ConformalValidationSsbEmitter for VecConformalValidationSsbEmitter {
    async fn emit_run_completed(
        &self,
        payload: crate::ssb::ConformalValidationCompletedPayload,
    ) -> Result<(), String> {
        let mut guard = self.rows.lock().await;
        guard.push(payload);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Fixture row
// ---------------------------------------------------------------------------

/// One row of the on-disk JSONL validation fixture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationFixtureRow {
    pub row_id: String,
    /// Snake-case [`DistributionBucket`] wire-label.
    pub distribution: String,
    /// Snake-case [`Domain`] wire-label.
    pub domain: String,
    /// Composite confidence in `[0.0, 1.0]` at decision time. Mirrors the
    /// calibration corpus's `predicted_score`.
    pub predicted_score: f64,
    /// Post-hoc verdict — `true` iff the claim was confirmed correct.
    pub ground_truth: bool,
}

impl ValidationFixtureRow {
    fn distribution(
        &self,
        line_index: usize,
        path: &str,
    ) -> Result<DistributionBucket, ConformalValidationError> {
        parse_distribution(&self.distribution).ok_or_else(|| {
            ConformalValidationError::InvalidDistribution {
                path: path.to_owned(),
                line_index,
                value: self.distribution.clone(),
            }
        })
    }

    fn domain(
        &self,
        line_index: usize,
        path: &str,
    ) -> Result<Domain, ConformalValidationError> {
        parse_domain(&self.domain).ok_or_else(|| ConformalValidationError::InvalidDomain {
            path: path.to_owned(),
            line_index,
            value: self.domain.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Snake-case enum parsers (mirrors of the `serde(rename_all = "snake_case")`
// derives — bare strings stay valid even after a `Cargo.toml` edition bump).
// ---------------------------------------------------------------------------

fn parse_distribution(s: &str) -> Option<DistributionBucket> {
    match s {
        "in_domain" => Some(DistributionBucket::InDomain),
        "near_ood" => Some(DistributionBucket::NearOod),
        "far_ood" => Some(DistributionBucket::FarOod),
        _ => None,
    }
}

fn parse_domain(s: &str) -> Option<Domain> {
    match s {
        "code" => Some(Domain::Code),
        "math" => Some(Domain::Math),
        "prose" => Some(Domain::Prose),
        "mixed" => Some(Domain::Mixed),
        "planning" => Some(Domain::Planning),
        "tool_use" => Some(Domain::ToolUse),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Conformal calibration validation runner. Construction is cheap; the
/// runner is single-shot (mirror of `PRMBenchRunner` + `ModeRoutingBenchmark`).
pub struct ConformalValidationRunner {
    calibration_corpus: CalibrationCorpus,
    validation_fixture: PathBuf,
    epsilon: f64,
    min_rows_per_bucket: u32,
}

impl ConformalValidationRunner {
    /// Construct a runner against the supplied corpus + validation fixture
    /// path. Uses the D-16 default epsilon (0.1) and the D-22 plan default
    /// minimum per-bucket row count (5).
    #[must_use]
    pub fn new(calibration_corpus: CalibrationCorpus, validation_fixture: PathBuf) -> Self {
        Self {
            calibration_corpus,
            validation_fixture,
            epsilon: DEFAULT_EPSILON,
            min_rows_per_bucket: DEFAULT_MIN_ROWS_PER_DISTRIBUTION,
        }
    }

    /// Override the operating-point epsilon. Returns the builder for chained
    /// construction. Caller is responsible for staying in `(0.0, 1.0)`;
    /// out-of-range values fail at [`run_all`] when `compute_tau` rejects.
    #[must_use]
    pub fn with_epsilon(mut self, epsilon: f64) -> Self {
        self.epsilon = epsilon;
        self
    }

    /// Override the per-bucket minimum row count. Pin to `0` in tests that
    /// want to exercise class-collapse paths.
    #[must_use]
    pub fn with_min_rows_per_bucket(mut self, n: u32) -> Self {
        self.min_rows_per_bucket = n;
        self
    }

    async fn load_fixture(
        &self,
    ) -> Result<(Vec<ValidationFixtureRow>, String), ConformalValidationError> {
        let path_str = self.validation_fixture.to_string_lossy().into_owned();
        let raw = fs::read_to_string(&self.validation_fixture)
            .await
            .map_err(|e| ConformalValidationError::Io {
                path: path_str.clone(),
                source_msg: format!("{e}"),
            })?;
        let mut rows: Vec<ValidationFixtureRow> = Vec::new();
        for (idx, line) in raw.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let row: ValidationFixtureRow = serde_json::from_str(trimmed).map_err(|e| {
                ConformalValidationError::MalformedJson {
                    path: path_str.clone(),
                    line_index: idx,
                    source_msg: format!("{e}"),
                }
            })?;
            // Validate label up front so a runtime parse failure cannot
            // silently dilute a bucket's empirical rate mid-loop.
            let _ = row.distribution(idx, &path_str)?;
            let _ = row.domain(idx, &path_str)?;
            rows.push(row);
        }
        if rows.is_empty() {
            return Err(ConformalValidationError::EmptyFixture { path: path_str });
        }
        let mut hasher = Hasher::new();
        hasher.update(raw.as_bytes());
        let fixture_blake3 = hasher.finalize().to_hex().to_string();
        Ok((rows, fixture_blake3))
    }

    /// Run the full fixture: compute tau, score each row through
    /// [`decide`], roll per-bucket + per-domain refusal rates, emit one
    /// SSB row.
    ///
    /// `now_ms` is injected (no clock read inside the producer) per the
    /// entropy-quarantine rule — matches the D-16 producer surfaces.
    pub async fn run_all<E>(
        &self,
        emitter: &E,
        now_ms: i64,
    ) -> Result<ConformalValidationReport, ConformalValidationError>
    where
        E: ConformalValidationSsbEmitter + ?Sized,
    {
        // 1) Compute tau on the calibration corpus. Bubble up rather than
        //    silently degrading — a bad corpus is a CI signal, not a row.
        let tau = compute_tau(&self.calibration_corpus, self.epsilon)?;

        // 2) Load + parse the validation fixture.
        let (rows, fixture_blake3) = self.load_fixture().await?;
        info!(
            fixture_path = %self.validation_fixture.to_string_lossy(),
            total_rows = rows.len(),
            tau = tau,
            epsilon = self.epsilon,
            "conformal-validation: starting run",
        );
        let path_str = self.validation_fixture.to_string_lossy().into_owned();

        // 3) Score every row through `decide`. Bucket + domain counters
        //    feed the breakdown.
        let mut bucket_totals: BTreeMap<DistributionBucket, (u32, u32)> = BTreeMap::new();
        // Per-(domain, bucket) cell counters: (n_tested, n_refused).
        // Domain does not implement Ord (closed enum, only Hash + Eq), so
        // we accumulate into a HashMap and emit deterministic Vec output
        // via a fixed enum-order sort below.
        let mut domain_cells: HashMap<(Domain, DistributionBucket), (u32, u32)> =
            HashMap::new();
        let mut total_refused: u32 = 0;

        for (idx, row) in rows.iter().enumerate() {
            let bucket = row.distribution(idx, &path_str)?;
            let domain = row.domain(idx, &path_str)?;
            let decision = decide(row.predicted_score, tau);
            let refused = matches!(decision, RefusalDecision::Abstain);
            let b_entry = bucket_totals.entry(bucket).or_insert((0, 0));
            b_entry.0 = b_entry.0.saturating_add(1);
            if refused {
                b_entry.1 = b_entry.1.saturating_add(1);
                total_refused = total_refused.saturating_add(1);
            }
            let d_entry = domain_cells.entry((domain, bucket)).or_insert((0, 0));
            d_entry.0 = d_entry.0.saturating_add(1);
            if refused {
                d_entry.1 = d_entry.1.saturating_add(1);
            }
            debug!(
                row_id = %row.row_id,
                bucket = bucket.as_str(),
                domain = domain.as_str(),
                refused,
                "conformal-validation: row scored",
            );
        }

        // 4) Roll up aggregate empirical refusal rate.
        let total_u32 = u32::try_from(rows.len()).unwrap_or(u32::MAX);
        let empirical_refusal_rate = if total_u32 == 0 {
            0.0
        } else {
            f64::from(total_refused) / f64::from(total_u32)
        };

        // 5) Target band: `[target - epsilon, target + epsilon]`. Per
        //    UniCR §3 the long-run rate converges to the operating point,
        //    so `target = epsilon`. Lower-clamp to 0 because a refusal
        //    rate cannot be negative.
        let target = self.epsilon;
        let lower = (target - self.epsilon).max(0.0);
        let upper = (target + self.epsilon).min(1.0);
        let target_band = (lower, upper);

        // 6) Per-bucket aggregate refusal rates + in-band check.
        let mut per_bucket: BTreeMap<String, f64> = BTreeMap::new();
        let mut bucket_in_band: BTreeMap<DistributionBucket, bool> = BTreeMap::new();
        let mut all_buckets_ok = true;
        for bucket in DistributionBucket::all() {
            let (n, refused) = bucket_totals.get(&bucket).copied().unwrap_or((0, 0));
            let rate = if n == 0 {
                0.0
            } else {
                f64::from(refused) / f64::from(n)
            };
            let in_band = (rate - target).abs() <= self.epsilon
                && n >= self.min_rows_per_bucket;
            if !in_band {
                all_buckets_ok = false;
            }
            per_bucket.insert(bucket.as_str().to_string(), rate);
            bucket_in_band.insert(bucket, in_band);
        }

        // 7) Per-(domain, bucket) cell breakdown. Domain has no Ord
        //    derive, so we sort the final Vec by the snake-case wire
        //    labels to guarantee deterministic output across runs (locks
        //    the D-22 ACCEPTANCE "deterministic same-inputs same-output"
        //    contract against HashMap iteration drift).
        let mut per_domain: Vec<DomainBreakdown> = Vec::with_capacity(domain_cells.len());
        for ((domain, bucket), (n, refused)) in domain_cells.into_iter() {
            let rate = if n == 0 {
                0.0
            } else {
                f64::from(refused) / f64::from(n)
            };
            let in_band = (rate - target).abs() <= self.epsilon;
            per_domain.push(DomainBreakdown {
                domain,
                bucket,
                n_tested: n,
                refusal_rate: rate,
                in_band,
            });
        }
        per_domain.sort_by(|a, b| {
            a.domain
                .as_str()
                .cmp(b.domain.as_str())
                .then_with(|| a.bucket.as_str().cmp(b.bucket.as_str()))
        });

        // 8) Verdict: pass iff every bucket cleared the band AND the
        //    minimum-row-count gate. Single-bucket fail trips the gate.
        let gating_verdict = if all_buckets_ok && total_u32 > 0 {
            GatingVerdict::Pass
        } else {
            GatingVerdict::Fail
        };

        let report = ConformalValidationReport {
            ts_ms: now_ms,
            tau,
            epsilon: self.epsilon,
            empirical_refusal_rate,
            target_band,
            total: total_u32,
            per_bucket,
            per_domain,
            gating_verdict,
            fixture_blake3: fixture_blake3.clone(),
        };

        // 9) Emit the single SSB row.
        let payload = crate::ssb::ConformalValidationCompletedPayload {
            tau: report.tau,
            epsilon: report.epsilon,
            empirical_refusal_rate: report.empirical_refusal_rate,
            gating_verdict: report.gating_verdict.as_str().to_string(),
            fixture_blake3: fixture_blake3.clone(),
            total: report.total,
            ts_ms: now_ms,
        };
        emitter
            .emit_run_completed(payload)
            .await
            .map_err(ConformalValidationError::SsbEmit)?;
        info!(
            verdict = %report.gating_verdict.as_str(),
            empirical_refusal_rate = report.empirical_refusal_rate,
            tau = report.tau,
            "conformal-validation: run complete",
        );
        Ok(report)
    }
}

// ---------------------------------------------------------------------------
// Public helper: locate the bundled mini fixture.
// ---------------------------------------------------------------------------

/// Returns the default mini-fixture path relative to a project root
/// (`<project_root>/benchmarks/conformal/fixtures/conformal_validation_mini.jsonl`).
/// Pure path-arithmetic — does NOT touch the filesystem.
#[must_use]
pub fn default_mini_fixture_path(project_root: &Path) -> PathBuf {
    project_root
        .join("benchmarks")
        .join("conformal")
        .join("fixtures")
        .join("conformal_validation_mini.jsonl")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformal_calibration::{CalibrationTuple, CorpusSource};
    use tempfile::TempDir;

    fn balanced_corpus(n_pairs: usize) -> CalibrationCorpus {
        let mut c = CalibrationCorpus::new(CorpusSource::HeldOut);
        for i in 0..n_pairs {
            c.append(CalibrationTuple::new(
                format!("t-{i}"),
                format!("c-{i}"),
                "true claim",
                true,
                0.9,
                1_700_000_000 + (i as i64),
            ))
            .expect("append");
            c.append(CalibrationTuple::new(
                format!("f-{i}"),
                format!("d-{i}"),
                "false claim",
                false,
                0.2,
                1_700_000_500 + (i as i64),
            ))
            .expect("append");
        }
        c
    }

    async fn write_fixture(dir: &TempDir, rows: &[ValidationFixtureRow]) -> PathBuf {
        let path = dir.path().join("validation.jsonl");
        let mut out = String::new();
        for r in rows {
            out.push_str(&serde_json::to_string(r).expect("serialise row"));
            out.push('\n');
        }
        fs::write(&path, out).await.expect("write fixture");
        path
    }

    fn row(
        id: &str,
        distribution: &str,
        domain: &str,
        predicted_score: f64,
        ground_truth: bool,
    ) -> ValidationFixtureRow {
        ValidationFixtureRow {
            row_id: id.to_string(),
            distribution: distribution.to_string(),
            domain: domain.to_string(),
            predicted_score,
            ground_truth,
        }
    }

    #[test]
    fn distribution_bucket_round_trips_snake_case() {
        for b in DistributionBucket::all() {
            let parsed = parse_distribution(b.as_str()).expect("known bucket parses");
            assert_eq!(parsed, b);
        }
        assert!(parse_distribution("bogus").is_none());
    }

    #[test]
    fn parse_domain_covers_all_six_variants() {
        for d in [
            Domain::Code,
            Domain::Math,
            Domain::Prose,
            Domain::Mixed,
            Domain::Planning,
            Domain::ToolUse,
        ] {
            let parsed = parse_domain(d.as_str()).expect("known domain parses");
            assert_eq!(parsed, d);
        }
        assert!(parse_domain("bogus").is_none());
    }

    #[tokio::test]
    async fn run_all_emits_one_ssb_row_and_returns_report() {
        let dir = TempDir::new().expect("tempdir");
        // 5 in_domain rows + 5 near_ood + 5 far_ood, each with high
        // predicted_score (>= 0.95) -> almost all Answer -> low empirical
        // refusal rate within target_band (epsilon=0.1, target=0.1).
        let mut rows: Vec<ValidationFixtureRow> = Vec::new();
        for i in 0..5 {
            rows.push(row(&format!("id_{i}"), "in_domain", "code", 0.95, true));
            rows.push(row(&format!("nd_{i}"), "near_ood", "math", 0.95, true));
            rows.push(row(&format!("fd_{i}"), "far_ood", "prose", 0.95, true));
        }
        let path = write_fixture(&dir, &rows).await;
        let corpus = balanced_corpus(20);
        let runner = ConformalValidationRunner::new(corpus, path);
        let emitter = VecConformalValidationSsbEmitter::new();
        let report = runner
            .run_all(&emitter, 1_700_000_999_000)
            .await
            .expect("run ok");
        assert_eq!(report.total, 15);
        assert!(report.empirical_refusal_rate <= 0.1 + f64::EPSILON);
        assert_eq!(emitter.len().await, 1);
        let written = emitter.rows().await;
        assert_eq!(written[0].fixture_blake3.len(), 64);
        assert_eq!(written[0].total, 15);
        assert_eq!(written[0].ts_ms, 1_700_000_999_000);
    }

    #[tokio::test]
    async fn empty_fixture_returns_error() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("empty.jsonl");
        fs::write(&path, "").await.expect("write empty");
        let runner = ConformalValidationRunner::new(balanced_corpus(20), path);
        let emitter = VecConformalValidationSsbEmitter::new();
        match runner.run_all(&emitter, 0).await {
            Err(ConformalValidationError::EmptyFixture { .. }) => {}
            other => panic!("expected EmptyFixture, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn invalid_distribution_returns_error() {
        let dir = TempDir::new().expect("tempdir");
        let bad = row("bad", "bogus_bucket", "code", 0.5, true);
        let path = write_fixture(&dir, &[bad]).await;
        let runner = ConformalValidationRunner::new(balanced_corpus(20), path);
        let emitter = VecConformalValidationSsbEmitter::new();
        match runner.run_all(&emitter, 0).await {
            Err(ConformalValidationError::InvalidDistribution { value, .. }) => {
                assert_eq!(value, "bogus_bucket");
            }
            other => panic!("expected InvalidDistribution, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn invalid_domain_returns_error() {
        let dir = TempDir::new().expect("tempdir");
        let bad = row("bad", "in_domain", "bogus_domain", 0.5, true);
        let path = write_fixture(&dir, &[bad]).await;
        let runner = ConformalValidationRunner::new(balanced_corpus(20), path);
        let emitter = VecConformalValidationSsbEmitter::new();
        match runner.run_all(&emitter, 0).await {
            Err(ConformalValidationError::InvalidDomain { value, .. }) => {
                assert_eq!(value, "bogus_domain");
            }
            other => panic!("expected InvalidDomain, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn class_collapse_trips_gate() {
        // Fixture omits far_ood entirely -> n_tested for that bucket is 0
        // -> in_band false -> gate Fail. Locks the no-class-collapse
        // contract against drift.
        let dir = TempDir::new().expect("tempdir");
        let mut rows: Vec<ValidationFixtureRow> = Vec::new();
        for i in 0..5 {
            rows.push(row(&format!("id_{i}"), "in_domain", "code", 0.95, true));
            rows.push(row(&format!("nd_{i}"), "near_ood", "math", 0.95, true));
        }
        let path = write_fixture(&dir, &rows).await;
        let runner = ConformalValidationRunner::new(balanced_corpus(20), path);
        let emitter = VecConformalValidationSsbEmitter::new();
        let report = runner.run_all(&emitter, 0).await.expect("run ok");
        assert_eq!(report.gating_verdict, GatingVerdict::Fail);
    }

    #[tokio::test]
    async fn under_min_rows_trips_gate() {
        // 4 rows per bucket — below the default 5 minimum.
        let dir = TempDir::new().expect("tempdir");
        let mut rows: Vec<ValidationFixtureRow> = Vec::new();
        for i in 0..4 {
            rows.push(row(&format!("id_{i}"), "in_domain", "code", 0.95, true));
            rows.push(row(&format!("nd_{i}"), "near_ood", "math", 0.95, true));
            rows.push(row(&format!("fd_{i}"), "far_ood", "prose", 0.95, true));
        }
        let path = write_fixture(&dir, &rows).await;
        let runner = ConformalValidationRunner::new(balanced_corpus(20), path);
        let emitter = VecConformalValidationSsbEmitter::new();
        let report = runner.run_all(&emitter, 0).await.expect("run ok");
        assert_eq!(report.gating_verdict, GatingVerdict::Fail);
    }

    #[test]
    fn default_mini_fixture_path_is_under_benchmarks_conformal() {
        let root = Path::new("/tmp/elai-project");
        let p = default_mini_fixture_path(root);
        assert!(
            p.ends_with("benchmarks/conformal/fixtures/conformal_validation_mini.jsonl"),
            "got {}",
            p.display()
        );
    }
}
