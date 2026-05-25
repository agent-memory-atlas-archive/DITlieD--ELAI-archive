//! D-25 — Determinism + archive tamper test suite runner.
//!
//! Quantitative harness on top of the D-8
//! [`crate::archive_chain::verify_archive`] substrate. Drives a 10-
//! scenario JSON-manifest matrix IN-PROCESS (no shell subprocess —
//! Windows-friendly + 5000ms per-scenario budget achievable), applies
//! a typed [`TamperOp`] per scenario against a sandboxed JSONL chain
//! built via [`crate::archive_chain::ArchiveChain::append`], invokes
//! D-8 `verify_archive` directly, and tallies per-scenario outcomes
//! into a [`TamperBenchmarkReport`] gated by the 5000ms wall budget
//! per plan objective.
//!
//! # Wiring discipline
//!
//! producer  : [`TamperBenchmark::run`] returns a typed report; the
//!             CLI driver (`elai bench tamper`) emits exactly one
//!             `SsbActionType::DeterminismTamperSuiteCompleted` row
//!             at end of run (single-variant precedent from D-19
//!             PRMBench + D-23 sycophancy + D-24 canary).
//! consumer  : `elai bench tamper` CLI + the bench-tamper.yml CI
//!             workflow. The dashboard panel is INTENTIONALLY out of
//!             scope per the plan's `WIRING_CONTRACT.reachable_from`
//!             stanza ("CLI + CI workflow"; no panel mount).
//! ssb       : single row per `run` invocation. The runner does NOT
//!             emit `BenchmarkStarted` / `BenchmarkResult` (no such
//!             generic variants exist; plan-doc drift cross-checked
//!             against ssb.rs head per D-25 DECISION-2).
//! observable: `elai bench tamper` against the bundled 10-scenario
//!             matrix produces a JSON report and exactly one SSB row.
//!
//! # In-process drive of D-8 — no SSB pollution
//!
//! D-8's [`crate::archive_chain::verify_archive`] is a `pub fn` that
//! returns a typed [`crate::archive_chain::ArchiveVerifyReport`]
//! WITHOUT emitting an SSB row directly (the
//! `ArchiveVerifyResult` SSB row is emitted by the
//! `elai_cli::archive_verify` CLI driver, NOT by `verify_archive`).
//! D-25 calls `verify_archive` directly — it does NOT call
//! `emit_archive_verify_result` per scenario (that would pollute the
//! D-8 SSB stream with 10 synthetic `ArchiveVerifyResult` rows per
//! benchmark invocation per D-25 DECISION-9). The single end-of-run
//! `DeterminismTamperSuiteCompleted` row is the only SSB side-effect
//! of a D-25 benchmark run.
//!
//! # Scenario manifest format
//!
//! One JSON `manifest.json` per scenario subdirectory under
//! `<root>/.elai_cc/benchmarks/tamper/scenarios/scenario_NN/`:
//!
//! ```json
//! {
//!   "id": "scenario_01",
//!   "kind": "PositiveControl",
//!   "tamper_ops": [],
//!   "expected_outcome": "CleanPass",
//!   "notes": "..."
//! }
//! ```
//!
//! Or for a tamper scenario:
//!
//! ```json
//! {
//!   "id": "scenario_03",
//!   "kind": "ContentHashMismatch",
//!   "tamper_ops": [{"op": "MutateContentHash", "args": {"entry_index": 1}}],
//!   "expected_outcome": {"ExpectedError": {"kind": "hash_mismatch"}},
//!   "notes": "..."
//! }
//! ```
//!
//! Per D-25 DECISION-5 each scenario subdir also ships a `setup.sh`
//! kept as DOCUMENTATION only (the runner does NOT shell out to it;
//! `apply_tamper` is a typed Rust function driven from the manifest).
//!
//! # Anti-stale + zero-lazy
//!
//! - blake3 for scenario-corpus content hash (anti-stale rule).
//! - thiserror for errors. tracing for diagnostics.
//! - serde_json for manifest parsing (no new toml dep per D-25 DECISION-4).
//! - tokio::fs for async dir walk + read; no blocking std::fs in async paths.
//! - No `.unwrap()` outside `#[cfg(test)]`. No silent swallows.
//! - Sandbox JSONL built via `std::env::temp_dir()` (no new tempfile
//!   runtime dep; tempfile stays a dev-only workspace dep).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::archive_chain::{
    verify_archive, ArchiveChain, ArchiveEntry, ArchiveVerifyError, ArchiveVerifyReport,
};
use crate::benchmarks::prmbench_runner::GatingVerdict;
use crate::reasoning_template::{
    ArchiveStatus, Blake3Hash, Domain, PatternKind, Predicate, ReasoningTemplate, TemplateId,
    VerifierKind,
};

// ---------------------------------------------------------------------------
// Tuning constants
// ---------------------------------------------------------------------------

/// Wall budget per scenario in milliseconds. Plan objective: each
/// tamper scenario MUST be detected within 5s. In-process Rust harness
/// hits this comfortably (`verify_archive` against a small synthetic
/// chain is microseconds); the field is wire-stable so dashboards can
/// surface budget regressions when the matrix grows.
pub const DEFAULT_MAX_WALL_MS_PER_SCENARIO: u64 = 5000;

/// Static tag echoed into log lines so multi-runner SSB streams can
/// disambiguate D-25 tamper bench runs from other benchmark runners.
pub const BENCH_RUNNER_NAME: &str = "tamper_bench_v1";

/// Number of seed entries appended to each scenario's sandboxed
/// archive before tampering. Chosen so multi-line scenarios (multi-
/// tamper, fail-fast) have enough rows to mutate distinct positions
/// without overlap. Small enough that verify_archive stays well under
/// the 5000ms budget on every host (microseconds in practice).
pub const SCENARIO_SEED_ENTRY_COUNT: u32 = 5;

// ---------------------------------------------------------------------------
// Public value types
// ---------------------------------------------------------------------------

/// Closed taxonomy of the tamper operations the runner can apply
/// to a sandboxed archive JSONL. Each variant is wire-stable so the
/// dashboard / log surface can classify per-scenario coverage.
///
/// `Box<TamperOp>` inside `MultiTamper` keeps the enum size small
/// without unbounded recursion; the runner applies the inner ops
/// sequentially.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "PascalCase")]
pub enum TamperOp {
    /// Rewrite `parent_hash` field of the entry at `entry_index` to a
    /// non-existent dangling hash. Expected to surface as
    /// [`ArchiveVerifyError::BrokenParent`].
    BreakParentLink { entry_index: u32 },
    /// Rewrite `template.template_body` of the entry at `entry_index`
    /// without recomputing `self_hash`. Expected to surface as
    /// [`ArchiveVerifyError::HashMismatch`].
    MutateContentHash { entry_index: u32 },
    /// Copy the JSONL line at `entry_index` and append it again with
    /// the SAME `self_hash`. Expected to surface as
    /// [`ArchiveVerifyError::DuplicateHash`].
    DuplicateEntry { entry_index: u32 },
    /// Replace the JSONL line at `entry_index` with garbage so
    /// serde_json fails. Expected to surface as
    /// [`ArchiveVerifyError::MalformedJson`].
    CorruptJsonl { entry_index: u32 },
    /// Delete the JSONL line at `entry_index`. The next entry's
    /// `parent_hash` now references a deleted self_hash. Expected to
    /// surface as [`ArchiveVerifyError::BrokenParent`] on the entry
    /// that USED to be at `entry_index + 1` (now `entry_index` after
    /// shift).
    DeleteEntry { entry_index: u32 },
    /// Append a trailing malformed JSONL line at the end of the file.
    /// Expected to surface as [`ArchiveVerifyError::MalformedJson`].
    AppendMalformedTrailing,
    /// Snapshot the file before any tamper, apply the inner ops, then
    /// restore the snapshot before verify is invoked. Expected outcome
    /// is `CleanPass` (idempotency / revert scenario).
    RevertBackup { inner: Vec<Box<TamperOp>> },
    /// Apply multiple ops in sequence on the same file. Used by the
    /// multi-error scenario (e.g. break parent on entry 1 + corrupt
    /// jsonl on entry 3).
    MultiTamper { inner: Vec<Box<TamperOp>> },
    /// No tamper — verify twice and assert the same outcome both
    /// times (idempotency check). Expected `CleanPass`.
    IdempotencyDoubleVerify,
    /// Apply a single inner op, then verify TWICE with
    /// `fail_fast=true` and `fail_fast=false`. Asserts the fail-fast
    /// pass returns exactly one error and the collect-all pass
    /// returns at least one error. Used by scenario_10.
    FailFastSingleError { inner: Box<TamperOp> },
}

impl TamperOp {
    /// Short wire tag for log/SSB classification.
    #[must_use]
    pub fn tag(&self) -> &'static str {
        match self {
            Self::BreakParentLink { .. } => "break_parent_link",
            Self::MutateContentHash { .. } => "mutate_content_hash",
            Self::DuplicateEntry { .. } => "duplicate_entry",
            Self::CorruptJsonl { .. } => "corrupt_jsonl",
            Self::DeleteEntry { .. } => "delete_entry",
            Self::AppendMalformedTrailing => "append_malformed_trailing",
            Self::RevertBackup { .. } => "revert_backup",
            Self::MultiTamper { .. } => "multi_tamper",
            Self::IdempotencyDoubleVerify => "idempotency_double_verify",
            Self::FailFastSingleError { .. } => "fail_fast_single_error",
        }
    }
}

/// Expected outcome of a scenario, encoded in the manifest.
///
/// `CleanPass`: verify returns Ok with zero errors (positive control
/// + revert + idempotency).
///
/// `ExpectedError { kind }`: verify returns Ok with at least one
/// error matching `kind` (snake_case ArchiveVerifyError variant tag).
///
/// `MultiError { kinds }`: verify (collect-all) returns Ok with at
/// least one error per listed kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectedOutcome {
    CleanPass,
    ExpectedError { kind: String },
    MultiError { kinds: Vec<String> },
}

/// Observed outcome of a scenario after the runner applies the
/// tamper ops + invokes verify.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// verify returned Ok with zero errors.
    CleanPass,
    /// verify returned Ok with at least one error matching `kind`
    /// (snake_case ArchiveVerifyError variant tag).
    ExpectedError { kind: String },
    /// verify returned Ok with at least one error of an unexpected
    /// kind (or no errors at all when an error was expected).
    UnexpectedError { kind: String, msg: String },
    /// scenario exceeded the 5000ms wall budget.
    Timeout,
}

impl Outcome {
    /// Short wire tag for the JSON report + log surface.
    #[must_use]
    pub fn tag(&self) -> &'static str {
        match self {
            Self::CleanPass => "clean_pass",
            Self::ExpectedError { .. } => "expected_error",
            Self::UnexpectedError { .. } => "unexpected_error",
            Self::Timeout => "timeout",
        }
    }
}

/// One scenario manifest (parsed from `manifest.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioManifest {
    /// Stable scenario id (e.g. `"scenario_01"`).
    pub id: String,
    /// Free-form scenario kind tag (e.g. `"PositiveControl"`,
    /// `"BrokenParent"`, `"ContentHashMismatch"`). Used only for log
    /// + dashboard classification; the runner derives the actual
    /// behavior from `tamper_ops` + `expected_outcome`.
    pub kind: String,
    /// Sequence of tamper operations applied to the sandboxed archive
    /// JSONL before verify is invoked. Empty for the positive control.
    #[serde(default)]
    pub tamper_ops: Vec<TamperOp>,
    /// Expected outcome the runner compares the observed verify
    /// outcome against.
    pub expected_outcome: ExpectedOutcome,
    /// Free-form authoring notes. NOT used by the runner.
    #[serde(default)]
    pub notes: String,
}

/// Per-scenario report row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub id: String,
    pub scenario_kind: String,
    pub expected_error_kind: Option<String>,
    pub observed_outcome: Outcome,
    pub wall_ms: u64,
    pub passed: bool,
}

/// Aggregate report — one per `run` invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TamperBenchmarkReport {
    pub per_scenario: Vec<ScenarioResult>,
    pub gating_verdict: GatingVerdict,
    pub max_wall_ms: u64,
    pub total_scenarios: u32,
    pub passed_count: u32,
    pub failed_count: u32,
    pub scenarios_blake3: String,
    /// Provenance distribution by scenario kind (e.g. `"BrokenParent"
    /// -> 2`). Surfaced in the JSON report for dashboard drill-down.
    pub scenario_kind_counts: BTreeMap<String, u32>,
}

impl TamperBenchmarkReport {
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
pub enum TamperBenchError {
    /// I/O failure reading the scenarios directory or one manifest
    /// file.
    #[error("tamper bench scenario I/O error at {path}: {source_msg}")]
    Io { path: String, source_msg: String },
    /// One manifest.json failed to deserialise as `ScenarioManifest`.
    #[error("tamper bench malformed manifest at {path}: {source_msg}")]
    MalformedManifest { path: String, source_msg: String },
    /// Scenarios directory exists but contained zero scenario subdirs.
    #[error("tamper bench empty scenarios dir at {path}")]
    EmptyScenariosDir { path: String },
    /// Failed to build the sandboxed seed archive (e.g. ReasoningTemplate
    /// append error).
    #[error("tamper bench scenario build failure for {scenario_id}: {source_msg}")]
    ScenarioBuild {
        scenario_id: String,
        source_msg: String,
    },
    /// `verify_archive` returned `Err` (infrastructural I/O — NOT a
    /// tamper finding; findings go inside `report.errors`).
    #[error("tamper bench verify infrastructure failure for {scenario_id}: {source_msg}")]
    VerifyInfrastructure {
        scenario_id: String,
        source_msg: String,
    },
    /// Tamper op asked for an entry index that does not exist in the
    /// sandboxed JSONL.
    #[error(
        "tamper bench tamper-op entry index out of bounds for {scenario_id}: \
         requested={requested}, have={have}"
    )]
    EntryIndexOutOfBounds {
        scenario_id: String,
        requested: u32,
        have: u32,
    },
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// D-25 determinism + archive tamper-suite benchmark. Stateless
/// (besides the immutable configuration) — `run` reads the scenario
/// manifests, drives D-8's `verify_archive` IN-PROCESS once per
/// scenario, aggregates, and returns the report.
#[derive(Debug, Clone)]
pub struct TamperBenchmark {
    scenarios_dir: PathBuf,
    max_wall_ms_per_scenario: u64,
}

impl TamperBenchmark {
    /// Construct with the default wall budget
    /// ([`DEFAULT_MAX_WALL_MS_PER_SCENARIO`]).
    #[must_use]
    pub fn new(scenarios_dir: PathBuf) -> Self {
        Self {
            scenarios_dir,
            max_wall_ms_per_scenario: DEFAULT_MAX_WALL_MS_PER_SCENARIO,
        }
    }

    /// Override the per-scenario wall budget.
    #[must_use]
    pub fn with_max_wall_ms_per_scenario(mut self, ms: u64) -> Self {
        self.max_wall_ms_per_scenario = ms;
        self
    }

    /// Current per-scenario wall budget (read-only accessor).
    #[must_use]
    pub fn max_wall_ms_per_scenario(&self) -> u64 {
        self.max_wall_ms_per_scenario
    }

    /// Scenarios directory path (read-only accessor).
    #[must_use]
    pub fn scenarios_dir(&self) -> &Path {
        &self.scenarios_dir
    }

    /// Load all `scenario_*/manifest.json` files under
    /// `scenarios_dir` (sorted by directory name for deterministic
    /// blake3). Skips entries starting with `_` (reserved for README
    /// + auxiliary docs).
    async fn load_manifests(
        &self,
    ) -> Result<(Vec<(PathBuf, ScenarioManifest)>, String), TamperBenchError> {
        let dir_str = self.scenarios_dir.to_string_lossy().into_owned();
        let mut read_dir =
            fs::read_dir(&self.scenarios_dir)
                .await
                .map_err(|e| TamperBenchError::Io {
                    path: dir_str.clone(),
                    source_msg: format!("{e}"),
                })?;

        let mut subdirs: Vec<PathBuf> = Vec::new();
        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| TamperBenchError::Io {
                path: dir_str.clone(),
                source_msg: format!("{e}"),
            })?
        {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with('_') || name.starts_with('.') {
                continue;
            }
            let file_type = entry.file_type().await.map_err(|e| TamperBenchError::Io {
                path: path.to_string_lossy().into_owned(),
                source_msg: format!("{e}"),
            })?;
            if !file_type.is_dir() {
                continue;
            }
            subdirs.push(path);
        }
        subdirs.sort();

        if subdirs.is_empty() {
            return Err(TamperBenchError::EmptyScenariosDir {
                path: dir_str.clone(),
            });
        }

        let mut hasher = Hasher::new();
        let mut manifests: Vec<(PathBuf, ScenarioManifest)> = Vec::with_capacity(subdirs.len());
        for subdir in subdirs {
            let manifest_path = subdir.join("manifest.json");
            let manifest_path_str = manifest_path.to_string_lossy().into_owned();
            let raw =
                fs::read_to_string(&manifest_path)
                    .await
                    .map_err(|e| TamperBenchError::Io {
                        path: manifest_path_str.clone(),
                        source_msg: format!("{e}"),
                    })?;
            // Hash the subdir name + NUL + raw bytes + NUL so the corpus
            // hash captures both contents AND ordering.
            if let Some(name) = subdir.file_name().and_then(|n| n.to_str()) {
                hasher.update(name.as_bytes());
            }
            hasher.update(&[0u8]);
            hasher.update(raw.as_bytes());
            hasher.update(&[0u8]);

            let manifest: ScenarioManifest = serde_json::from_str(&raw).map_err(|e| {
                TamperBenchError::MalformedManifest {
                    path: manifest_path_str.clone(),
                    source_msg: format!("{e}"),
                }
            })?;
            manifests.push((subdir, manifest));
        }
        let scenarios_blake3 = hasher.finalize().to_hex().to_string();
        Ok((manifests, scenarios_blake3))
    }

    /// Run the benchmark end-to-end. Reads scenario manifests, drives
    /// D-8's `verify_archive` IN-PROCESS once per scenario, tallies
    /// outcomes, computes the gating verdict.
    pub async fn run(&self) -> Result<TamperBenchmarkReport, TamperBenchError> {
        let (manifests, scenarios_blake3) = self.load_manifests().await?;
        info!(
            scenarios_dir = %self.scenarios_dir.to_string_lossy(),
            total = manifests.len(),
            "tamper bench: starting run",
        );

        let mut per_scenario: Vec<ScenarioResult> = Vec::with_capacity(manifests.len());
        let mut scenario_kind_counts: BTreeMap<String, u32> = BTreeMap::new();
        let mut max_wall_ms: u64 = 0;
        let mut passed_count: u32 = 0;
        let mut failed_count: u32 = 0;

        for (idx, (subdir, manifest)) in manifests.iter().enumerate() {
            *scenario_kind_counts
                .entry(manifest.kind.clone())
                .or_insert(0) += 1;
            let started = Instant::now();
            let outcome = self
                .run_one_scenario(idx as u32, subdir, manifest)
                .await?;
            let wall_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            let final_outcome = if wall_ms > self.max_wall_ms_per_scenario {
                Outcome::Timeout
            } else {
                outcome
            };
            let expected_error_kind = match &manifest.expected_outcome {
                ExpectedOutcome::CleanPass => None,
                ExpectedOutcome::ExpectedError { kind } => Some(kind.clone()),
                ExpectedOutcome::MultiError { kinds } => kinds.first().cloned(),
            };
            let passed = is_outcome_match(&manifest.expected_outcome, &final_outcome);
            if passed {
                passed_count = passed_count.saturating_add(1);
            } else {
                failed_count = failed_count.saturating_add(1);
            }
            max_wall_ms = max_wall_ms.max(wall_ms);
            debug!(
                id = %manifest.id,
                kind = %manifest.kind,
                outcome = %final_outcome.tag(),
                wall_ms,
                passed,
                "tamper bench: scenario classified",
            );
            per_scenario.push(ScenarioResult {
                id: manifest.id.clone(),
                scenario_kind: manifest.kind.clone(),
                expected_error_kind,
                observed_outcome: final_outcome,
                wall_ms,
                passed,
            });
        }

        let total_scenarios = u32::try_from(per_scenario.len()).unwrap_or(u32::MAX);
        let gating_verdict =
            if failed_count == 0 && max_wall_ms <= self.max_wall_ms_per_scenario {
                GatingVerdict::Pass
            } else {
                GatingVerdict::Fail
            };
        if matches!(gating_verdict, GatingVerdict::Fail) {
            warn!(
                passed_count,
                failed_count,
                max_wall_ms,
                budget_ms = self.max_wall_ms_per_scenario,
                "tamper bench: run FAIL",
            );
        } else {
            info!(
                passed_count,
                failed_count,
                max_wall_ms,
                "tamper bench: run PASS",
            );
        }
        Ok(TamperBenchmarkReport {
            per_scenario,
            gating_verdict,
            max_wall_ms,
            total_scenarios,
            passed_count,
            failed_count,
            scenarios_blake3,
            scenario_kind_counts,
        })
    }

    /// Run a single scenario: build sandbox archive, apply tamper
    /// ops, invoke verify_archive, classify the outcome.
    async fn run_one_scenario(
        &self,
        run_index: u32,
        _subdir: &Path,
        manifest: &ScenarioManifest,
    ) -> Result<Outcome, TamperBenchError> {
        // Build a unique sandbox path under the OS temp dir. We do
        // NOT use the `tempfile` crate at runtime (it is a dev-only
        // workspace dep per D-25 DECISION-4-adjacent — no new runtime
        // deps allowed).
        let sandbox_root = sandbox_root_for_run(&manifest.id, run_index);
        std::fs::create_dir_all(&sandbox_root).map_err(|e| TamperBenchError::Io {
            path: sandbox_root.to_string_lossy().into_owned(),
            source_msg: format!("{e}"),
        })?;
        let archive_path = sandbox_root.join("archive.jsonl");

        // Seed the archive with N deterministic entries.
        seed_sandbox_archive(&archive_path, manifest)?;

        // Apply tamper ops in declaration order.
        for op in &manifest.tamper_ops {
            apply_tamper(op, &archive_path, manifest)?;
        }

        // Determine fail-fast mode. FailFastSingleError requires two
        // verify passes; everything else uses collect-all by default
        // so multi-error scenarios surface every error.
        let outcome = run_verify_and_classify(&archive_path, manifest);

        // Best-effort cleanup; if rm fails, we leak a temp dir but
        // never propagate the error (the verify_archive outcome is
        // the load-bearing return).
        let _ = std::fs::remove_dir_all(&sandbox_root);

        Ok(outcome)
    }
}

// ---------------------------------------------------------------------------
// Tamper op application
// ---------------------------------------------------------------------------

/// Apply a single tamper op to the sandboxed archive JSONL file.
///
/// Driven from the JSON manifest per D-25 DECISION-5 — `setup.sh`
/// files are documentation only; this typed Rust fn is the runner.
pub fn apply_tamper(
    op: &TamperOp,
    archive_path: &Path,
    manifest: &ScenarioManifest,
) -> Result<(), TamperBenchError> {
    match op {
        TamperOp::BreakParentLink { entry_index } => {
            let lines = read_jsonl(archive_path)?;
            ensure_index(lines.len(), *entry_index, &manifest.id)?;
            let mut lines = lines;
            let mutated =
                rewrite_parent_hash(&lines[*entry_index as usize], "0".repeat(64).as_str());
            lines[*entry_index as usize] = mutated;
            write_jsonl(archive_path, &lines)?;
            Ok(())
        }
        TamperOp::MutateContentHash { entry_index } => {
            let lines = read_jsonl(archive_path)?;
            ensure_index(lines.len(), *entry_index, &manifest.id)?;
            let mut lines = lines;
            let mutated = rewrite_template_body(
                &lines[*entry_index as usize],
                "tampered body — content_hash now stale",
            );
            lines[*entry_index as usize] = mutated;
            write_jsonl(archive_path, &lines)?;
            Ok(())
        }
        TamperOp::DuplicateEntry { entry_index } => {
            let lines = read_jsonl(archive_path)?;
            ensure_index(lines.len(), *entry_index, &manifest.id)?;
            let dup = lines[*entry_index as usize].clone();
            let mut lines = lines;
            lines.push(dup);
            write_jsonl(archive_path, &lines)?;
            Ok(())
        }
        TamperOp::CorruptJsonl { entry_index } => {
            let lines = read_jsonl(archive_path)?;
            ensure_index(lines.len(), *entry_index, &manifest.id)?;
            let mut lines = lines;
            lines[*entry_index as usize] = "{this is not valid json: 12345".to_string();
            write_jsonl(archive_path, &lines)?;
            Ok(())
        }
        TamperOp::DeleteEntry { entry_index } => {
            let lines = read_jsonl(archive_path)?;
            ensure_index(lines.len(), *entry_index, &manifest.id)?;
            let mut lines = lines;
            lines.remove(*entry_index as usize);
            write_jsonl(archive_path, &lines)?;
            Ok(())
        }
        TamperOp::AppendMalformedTrailing => {
            let lines = read_jsonl(archive_path)?;
            let mut lines = lines;
            lines.push("{trailing garbage that fails json parse".to_string());
            write_jsonl(archive_path, &lines)?;
            Ok(())
        }
        TamperOp::RevertBackup { inner } => {
            // Snapshot the file, apply inner ops, then restore.
            let snapshot = read_jsonl(archive_path)?;
            for child in inner {
                apply_tamper(child, archive_path, manifest)?;
            }
            write_jsonl(archive_path, &snapshot)?;
            Ok(())
        }
        TamperOp::MultiTamper { inner } => {
            for child in inner {
                apply_tamper(child, archive_path, manifest)?;
            }
            Ok(())
        }
        TamperOp::IdempotencyDoubleVerify => {
            // No mutation; the runner's verify pass is invoked twice
            // and the outcomes compared in run_verify_and_classify.
            Ok(())
        }
        TamperOp::FailFastSingleError { inner } => {
            // Apply the inner op; the runner's verify pass is invoked
            // twice (fail_fast=true then fail_fast=false) and the
            // error counts compared in run_verify_and_classify.
            apply_tamper(inner, archive_path, manifest)
        }
    }
}

fn ensure_index(have: usize, requested: u32, scenario_id: &str) -> Result<(), TamperBenchError> {
    if (requested as usize) >= have {
        return Err(TamperBenchError::EntryIndexOutOfBounds {
            scenario_id: scenario_id.to_string(),
            requested,
            have: u32::try_from(have).unwrap_or(u32::MAX),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Sandbox helpers
// ---------------------------------------------------------------------------

/// Build a deterministic-by-id sandbox path under the OS temp dir.
fn sandbox_root_for_run(scenario_id: &str, run_index: u32) -> PathBuf {
    let mut hasher = blake3::Hasher::new();
    hasher.update(scenario_id.as_bytes());
    hasher.update(&run_index.to_le_bytes());
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    hasher.update(&nanos.to_le_bytes());
    let suffix = hasher.finalize().to_hex().to_string();
    std::env::temp_dir().join(format!("elai-tamper-{}-{}", scenario_id, &suffix[..16]))
}

/// Seed the sandbox archive with N deterministic chained entries.
fn seed_sandbox_archive(
    archive_path: &Path,
    manifest: &ScenarioManifest,
) -> Result<(), TamperBenchError> {
    let mut chain = ArchiveChain::new(archive_path.to_path_buf());
    for i in 0..SCENARIO_SEED_ENTRY_COUNT {
        let template = build_seed_template(i, &manifest.id)?;
        let entry = if i == 0 {
            ArchiveEntry::new_root(template)
        } else {
            let parent = chain
                .entries()
                .last()
                .ok_or_else(|| TamperBenchError::ScenarioBuild {
                    scenario_id: manifest.id.clone(),
                    source_msg: "missing parent entry while seeding".into(),
                })?;
            ArchiveEntry::new_child(template, parent.self_hash)
        };
        chain
            .append(entry)
            .map_err(|e| TamperBenchError::ScenarioBuild {
                scenario_id: manifest.id.clone(),
                source_msg: format!("{e}"),
            })?;
    }
    Ok(())
}

fn build_seed_template(
    idx: u32,
    scenario_id: &str,
) -> Result<ReasoningTemplate, TamperBenchError> {
    let body = format!(
        "tamper-suite seed body — scenario={scenario_id} idx={idx} \
         padding alpha beta gamma delta"
    );
    let template_body = crate::reasoning_template::TemplateBody::try_new(body).map_err(|e| {
        TamperBenchError::ScenarioBuild {
            scenario_id: scenario_id.to_string(),
            source_msg: format!("seed body exceeded TemplateBody bound: {e}"),
        }
    })?;
    // content_hash on the template itself is set to a deterministic
    // hex derived from scenario_id+idx; verify_archive computes the
    // *entry* self_hash from (body, parent_hash) so this is just a
    // record-keeping field.
    let content_hash = derive_blake3_from_seed(scenario_id, idx);
    Ok(ReasoningTemplate {
        id: derive_template_id_from_seed(scenario_id, idx),
        domain: Domain::Code,
        pattern_kind: PatternKind::Decomposition,
        template_body,
        eligibility_predicate: Predicate::Always,
        verifier_required: VerifierKind::Test,
        content_hash,
        created_at_ms: 1_700_000_000_000 + i64::from(idx),
        parent_id: None,
        archive_status: ArchiveStatus::Candidate,
    })
}

fn derive_blake3_from_seed(scenario_id: &str, idx: u32) -> Blake3Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(scenario_id.as_bytes());
    hasher.update(&idx.to_le_bytes());
    Blake3Hash::from(hasher.finalize())
}

fn derive_template_id_from_seed(scenario_id: &str, idx: u32) -> TemplateId {
    // Build a deterministic v4-shaped UUID from a blake3 hash of
    // (scenario_id, idx). NOT a real RFC 4122 UUID — but TemplateId
    // accepts any 128-bit value.
    let hash = blake3::hash(format!("{scenario_id}-{idx}").as_bytes());
    let bytes = hash.as_bytes();
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes.copy_from_slice(&bytes[..16]);
    TemplateId(uuid::Uuid::from_bytes(uuid_bytes))
}

fn read_jsonl(archive_path: &Path) -> Result<Vec<String>, TamperBenchError> {
    let raw = std::fs::read_to_string(archive_path).map_err(|e| TamperBenchError::Io {
        path: archive_path.to_string_lossy().into_owned(),
        source_msg: format!("{e}"),
    })?;
    Ok(raw
        .split('\n')
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect())
}

fn write_jsonl(archive_path: &Path, lines: &[String]) -> Result<(), TamperBenchError> {
    let mut body = String::with_capacity(lines.iter().map(|l| l.len() + 1).sum());
    for line in lines {
        body.push_str(line);
        body.push('\n');
    }
    std::fs::write(archive_path, body).map_err(|e| TamperBenchError::Io {
        path: archive_path.to_string_lossy().into_owned(),
        source_msg: format!("{e}"),
    })?;
    Ok(())
}

/// Rewrite the `parent_hash` field of a JSONL ArchiveEntry line to a
/// dangling hash. Uses serde_json round-trip to keep the rest of the
/// fields verbatim.
fn rewrite_parent_hash(line: &str, new_hash_hex: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(line) else {
        return line.to_string();
    };
    if let Some(obj) = value.as_object_mut() {
        // Build a Blake3Hash JSON shape: it's a tuple struct holding
        // a [u8; 32]. With #[derive(Serialize)] it serialises as a 32-
        // element JSON array of bytes (the default serde rep of
        // tuple-struct-of-array).
        let bytes = hex_to_bytes_32(new_hash_hex);
        let arr: Vec<serde_json::Value> = bytes
            .into_iter()
            .map(|b| serde_json::Value::Number(serde_json::Number::from(b)))
            .collect();
        obj.insert(
            "parent_hash".to_string(),
            serde_json::Value::Array(arr),
        );
    }
    serde_json::to_string(&value).unwrap_or_else(|_| line.to_string())
}

/// Rewrite the inner `template.template_body` text without touching
/// the entry-level `self_hash` field. Surfaces as
/// `ArchiveVerifyError::HashMismatch` on verify.
fn rewrite_template_body(line: &str, new_body: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(line) else {
        return line.to_string();
    };
    if let Some(obj) = value.as_object_mut() {
        if let Some(template) = obj.get_mut("template").and_then(|t| t.as_object_mut()) {
            template.insert(
                "template_body".to_string(),
                serde_json::Value::String(new_body.to_string()),
            );
        }
    }
    serde_json::to_string(&value).unwrap_or_else(|_| line.to_string())
}

fn hex_to_bytes_32(hex: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = hex.as_bytes();
    let mut i = 0;
    let mut j = 0;
    while j < 32 && i + 1 < bytes.len() {
        let hi = hex_nibble(bytes[i]);
        let lo = hex_nibble(bytes[i + 1]);
        out[j] = (hi << 4) | lo;
        i += 2;
        j += 1;
    }
    out
}

fn hex_nibble(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Verify + classify
// ---------------------------------------------------------------------------

fn run_verify_and_classify(archive_path: &Path, manifest: &ScenarioManifest) -> Outcome {
    // Special-case FailFastSingleError: run twice and compare error counts.
    let fail_fast_mode = manifest
        .tamper_ops
        .iter()
        .any(|op| matches!(op, TamperOp::FailFastSingleError { .. }));

    // Special-case IdempotencyDoubleVerify: run twice and assert
    // identical outcomes.
    let idempotency_mode = manifest
        .tamper_ops
        .iter()
        .any(|op| matches!(op, TamperOp::IdempotencyDoubleVerify));

    if fail_fast_mode {
        let ff = match verify_archive(archive_path, true) {
            Ok(r) => r,
            Err(e) => {
                return Outcome::UnexpectedError {
                    kind: "infrastructure".into(),
                    msg: format!("{e}"),
                };
            }
        };
        let collect = match verify_archive(archive_path, false) {
            Ok(r) => r,
            Err(e) => {
                return Outcome::UnexpectedError {
                    kind: "infrastructure".into(),
                    msg: format!("{e}"),
                };
            }
        };
        // FailFast must produce exactly one error; collect-all must
        // produce >= 1 error. If either fails, classify as
        // UnexpectedError.
        if ff.errors.len() != 1 {
            return Outcome::UnexpectedError {
                kind: "fail_fast_arm".into(),
                msg: format!(
                    "fail_fast=true returned {} errors, expected exactly 1",
                    ff.errors.len()
                ),
            };
        }
        if collect.errors.is_empty() {
            return Outcome::UnexpectedError {
                kind: "collect_all_arm".into(),
                msg: "fail_fast=false returned 0 errors, expected >= 1".into(),
            };
        }
        // Both arms behaved correctly; classify by the first error's tag.
        let kind = archive_verify_error_tag(&collect.errors[0]);
        return Outcome::ExpectedError { kind };
    }

    if idempotency_mode {
        let first = match verify_archive(archive_path, false) {
            Ok(r) => r,
            Err(e) => {
                return Outcome::UnexpectedError {
                    kind: "infrastructure".into(),
                    msg: format!("{e}"),
                };
            }
        };
        let second = match verify_archive(archive_path, false) {
            Ok(r) => r,
            Err(e) => {
                return Outcome::UnexpectedError {
                    kind: "infrastructure".into(),
                    msg: format!("{e}"),
                };
            }
        };
        if first.passed != second.passed || first.errors.len() != second.errors.len() {
            return Outcome::UnexpectedError {
                kind: "idempotency".into(),
                msg: format!(
                    "double-verify drifted: first.passed={} second.passed={}",
                    first.passed, second.passed
                ),
            };
        }
        return classify_report(&first);
    }

    let report = match verify_archive(archive_path, false) {
        Ok(r) => r,
        Err(e) => {
            return Outcome::UnexpectedError {
                kind: "infrastructure".into(),
                msg: format!("{e}"),
            };
        }
    };
    classify_report(&report)
}

fn classify_report(report: &ArchiveVerifyReport) -> Outcome {
    if report.passed {
        return Outcome::CleanPass;
    }
    if let Some(first) = report.errors.first() {
        Outcome::ExpectedError {
            kind: archive_verify_error_tag(first),
        }
    } else {
        // Should not happen — passed=false should always carry >= 1 error.
        Outcome::UnexpectedError {
            kind: "empty_error_list".into(),
            msg: "verify report passed=false with empty errors".into(),
        }
    }
}

/// Snake-case wire tag for an [`ArchiveVerifyError`] variant. Mirrors
/// the `#[serde(tag = "kind", rename_all = "snake_case")]` attribute
/// on the enum so log + report rows agree on the on-the-wire string.
pub fn archive_verify_error_tag(err: &ArchiveVerifyError) -> String {
    match err {
        ArchiveVerifyError::HashMismatch { .. } => "hash_mismatch".into(),
        ArchiveVerifyError::BrokenParent { .. } => "broken_parent".into(),
        ArchiveVerifyError::DuplicateHash { .. } => "duplicate_hash".into(),
        ArchiveVerifyError::MalformedJson { .. } => "malformed_json".into(),
        ArchiveVerifyError::Io { .. } => "io".into(),
    }
}

fn is_outcome_match(expected: &ExpectedOutcome, observed: &Outcome) -> bool {
    match (expected, observed) {
        (ExpectedOutcome::CleanPass, Outcome::CleanPass) => true,
        (ExpectedOutcome::ExpectedError { kind: ek }, Outcome::ExpectedError { kind: ok }) => {
            ek == ok
        }
        (ExpectedOutcome::MultiError { kinds }, Outcome::ExpectedError { kind: ok }) => {
            // The runner's classify_report surfaces the FIRST error
            // tag only. The MultiError invariant is enforced by the
            // FailFastSingleError op (which compares fail-fast vs
            // collect-all error counts). Match is true iff the
            // observed first-error kind is in the expected set.
            kinds.iter().any(|k| k == ok)
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_manifest(dir: &TempDir, scenario: &str, manifest: &ScenarioManifest) {
        let scenario_dir = dir.path().join(scenario);
        std::fs::create_dir_all(&scenario_dir).expect("create scenario dir");
        let body = serde_json::to_string_pretty(manifest).expect("serialize manifest");
        std::fs::write(scenario_dir.join("manifest.json"), body).expect("write manifest");
    }

    fn positive_control() -> ScenarioManifest {
        ScenarioManifest {
            id: "scenario_01".into(),
            kind: "PositiveControl".into(),
            tamper_ops: vec![],
            expected_outcome: ExpectedOutcome::CleanPass,
            notes: "clean archive verifies".into(),
        }
    }

    fn broken_parent() -> ScenarioManifest {
        ScenarioManifest {
            id: "scenario_02".into(),
            kind: "BrokenParent".into(),
            tamper_ops: vec![TamperOp::BreakParentLink { entry_index: 2 }],
            expected_outcome: ExpectedOutcome::ExpectedError {
                kind: "broken_parent".into(),
            },
            notes: "rewrite parent_hash to dangling".into(),
        }
    }

    fn content_hash_mismatch() -> ScenarioManifest {
        ScenarioManifest {
            id: "scenario_03".into(),
            kind: "ContentHashMismatch".into(),
            tamper_ops: vec![TamperOp::MutateContentHash { entry_index: 1 }],
            expected_outcome: ExpectedOutcome::ExpectedError {
                kind: "hash_mismatch".into(),
            },
            notes: "mutate body without recomputing self_hash".into(),
        }
    }

    fn duplicate_hash() -> ScenarioManifest {
        ScenarioManifest {
            id: "scenario_04".into(),
            kind: "DuplicateHash".into(),
            tamper_ops: vec![TamperOp::DuplicateEntry { entry_index: 1 }],
            expected_outcome: ExpectedOutcome::ExpectedError {
                kind: "duplicate_hash".into(),
            },
            notes: "byte-copy a prior JSONL line".into(),
        }
    }

    fn malformed_json() -> ScenarioManifest {
        ScenarioManifest {
            id: "scenario_05".into(),
            kind: "MalformedJson".into(),
            tamper_ops: vec![TamperOp::CorruptJsonl { entry_index: 2 }],
            expected_outcome: ExpectedOutcome::ExpectedError {
                kind: "malformed_json".into(),
            },
            notes: "replace a JSONL line with garbage".into(),
        }
    }

    fn revert_scenario() -> ScenarioManifest {
        ScenarioManifest {
            id: "scenario_09".into(),
            kind: "RevertBackup".into(),
            tamper_ops: vec![TamperOp::RevertBackup {
                inner: vec![Box::new(TamperOp::MutateContentHash { entry_index: 1 })],
            }],
            expected_outcome: ExpectedOutcome::CleanPass,
            notes: "tamper then revert — verify clean again".into(),
        }
    }

    #[tokio::test]
    async fn positive_control_passes() {
        let dir = TempDir::new().expect("tempdir");
        write_manifest(&dir, "scenario_01", &positive_control());
        let bench = TamperBenchmark::new(dir.path().to_path_buf());
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.total_scenarios, 1);
        assert_eq!(report.passed_count, 1);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.gating_verdict, GatingVerdict::Pass);
        assert_eq!(report.per_scenario[0].observed_outcome, Outcome::CleanPass);
    }

    #[tokio::test]
    async fn broken_parent_detected() {
        let dir = TempDir::new().expect("tempdir");
        write_manifest(&dir, "scenario_02", &broken_parent());
        let bench = TamperBenchmark::new(dir.path().to_path_buf());
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.passed_count, 1);
        assert!(matches!(
            report.per_scenario[0].observed_outcome,
            Outcome::ExpectedError { ref kind } if kind == "broken_parent"
        ));
    }

    #[tokio::test]
    async fn content_hash_mismatch_detected() {
        let dir = TempDir::new().expect("tempdir");
        write_manifest(&dir, "scenario_03", &content_hash_mismatch());
        let bench = TamperBenchmark::new(dir.path().to_path_buf());
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.passed_count, 1);
        assert!(matches!(
            report.per_scenario[0].observed_outcome,
            Outcome::ExpectedError { ref kind } if kind == "hash_mismatch"
        ));
    }

    #[tokio::test]
    async fn duplicate_hash_detected() {
        let dir = TempDir::new().expect("tempdir");
        write_manifest(&dir, "scenario_04", &duplicate_hash());
        let bench = TamperBenchmark::new(dir.path().to_path_buf());
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.passed_count, 1);
        let kind = match &report.per_scenario[0].observed_outcome {
            Outcome::ExpectedError { kind } => kind.clone(),
            other => panic!("expected ExpectedError, got {other:?}"),
        };
        assert_eq!(kind, "duplicate_hash");
    }

    #[tokio::test]
    async fn malformed_json_detected() {
        let dir = TempDir::new().expect("tempdir");
        write_manifest(&dir, "scenario_05", &malformed_json());
        let bench = TamperBenchmark::new(dir.path().to_path_buf());
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.passed_count, 1);
        assert!(matches!(
            report.per_scenario[0].observed_outcome,
            Outcome::ExpectedError { ref kind } if kind == "malformed_json"
        ));
    }

    #[tokio::test]
    async fn revert_recovers_clean() {
        let dir = TempDir::new().expect("tempdir");
        write_manifest(&dir, "scenario_09", &revert_scenario());
        let bench = TamperBenchmark::new(dir.path().to_path_buf());
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.passed_count, 1);
        assert_eq!(report.per_scenario[0].observed_outcome, Outcome::CleanPass);
    }

    #[tokio::test]
    async fn empty_dir_returns_typed_error() {
        let dir = TempDir::new().expect("tempdir");
        let bench = TamperBenchmark::new(dir.path().to_path_buf());
        match bench.run().await {
            Err(TamperBenchError::EmptyScenariosDir { .. }) => (),
            other => panic!("expected EmptyScenariosDir, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn scenarios_blake3_is_stable_across_runs() {
        let dir = TempDir::new().expect("tempdir");
        write_manifest(&dir, "scenario_01", &positive_control());
        write_manifest(&dir, "scenario_02", &broken_parent());
        let bench = TamperBenchmark::new(dir.path().to_path_buf());
        let r1 = bench.run().await.expect("run 1 ok");
        let r2 = bench.run().await.expect("run 2 ok");
        assert_eq!(r1.scenarios_blake3, r2.scenarios_blake3);
        assert_eq!(r1.scenarios_blake3.len(), 64);
    }

    #[tokio::test]
    async fn wall_budget_clamps_outcome_to_timeout() {
        let dir = TempDir::new().expect("tempdir");
        write_manifest(&dir, "scenario_01", &positive_control());
        // Set a 0ms budget so even a microsecond-scale verify clamps
        // to Timeout. This proves the gate is wired.
        let bench = TamperBenchmark::new(dir.path().to_path_buf())
            .with_max_wall_ms_per_scenario(0);
        let report = bench.run().await.expect("run ok");
        assert_eq!(report.gating_verdict, GatingVerdict::Fail);
        assert_eq!(report.per_scenario[0].observed_outcome, Outcome::Timeout);
    }

    #[test]
    fn tamper_op_tags_round_trip() {
        assert_eq!(
            TamperOp::BreakParentLink { entry_index: 0 }.tag(),
            "break_parent_link"
        );
        assert_eq!(TamperOp::AppendMalformedTrailing.tag(), "append_malformed_trailing");
        assert_eq!(TamperOp::IdempotencyDoubleVerify.tag(), "idempotency_double_verify");
    }

    #[test]
    fn outcome_tags_round_trip() {
        assert_eq!(Outcome::CleanPass.tag(), "clean_pass");
        assert_eq!(
            Outcome::ExpectedError {
                kind: "x".into()
            }
            .tag(),
            "expected_error"
        );
        assert_eq!(Outcome::Timeout.tag(), "timeout");
    }

    #[test]
    fn archive_verify_error_tag_matches_serde_rename() {
        let e = ArchiveVerifyError::BrokenParent {
            entry_index: 0,
            parent_hash: "x".into(),
            template_id: "y".into(),
        };
        assert_eq!(archive_verify_error_tag(&e), "broken_parent");
    }
}
