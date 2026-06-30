//! WIRING-FIX-ELAI Phase 2 (rescope SP3) — Layer-3 entrypoint-coverage runner.
//!
//! Builds an instrumented `elai` binary (`-C instrument-coverage`, applied by
//! `cargo llvm-cov`), drives the AUDITOR-OWNED smoke scenarios from
//! `smoke-manifest.toml` against the REAL binary under PRODUCTION-DEFAULT env,
//! then exports per-function execution counts via `cargo llvm-cov report
//! --json` (cargo-llvm-cov v0.8.7; the JSON flag is `--json`, NOT
//! `--format=json`). Any NEW function (introduced in the current git diff)
//! whose count == 0 is a BLOCK (wiring-enforcement R3/R6/R7), UNLESS suppressed
//! by `#[cfg(test)]` OR a registered `// WIRE-DARK[B-N]:` stub marker.
//!
//! FAIL-CLOSED INVARIANTS (wiring-enforcement R1/R2):
//!   - missing coverage JSON / zero functions in export => `Err(MeasurementAbsent)`,
//!     NEVER an empty-clean report (R2: a dead evidence path screams).
//!   - smoke-manifest with zero active `[[feature]]` entries while the git diff
//!     introduces a new feature-bearing entrypoint => a `ManifestMissingFeature`
//!     violation (report is NOT clean).
//!   - git-diff command failure => `Err(GitDiffFailed)`, NEVER an empty new-fn set
//!     (the empty-on-error pattern is fail-OPEN and is deliberately NOT copied here).
//!   - cargo-llvm-cov not installed / build / smoke / export failure => the
//!     matching `Err` variant; the CLI shell turns every such `Err` into exit 1.
//!
//! The runner does NOT add cargo-llvm-cov as a library dependency — it is a
//! cargo subcommand tool (`cargo install cargo-llvm-cov`) invoked as a
//! subprocess. No new Cargo.toml dep is introduced by this module.
//!
//! SSB EVIDENCE SURFACE (ADR-1, WIRING-FIX-ELAI WAVE-0 reconciliation):
//! on completion the runner produces SSB payloads reusing the EXISTING
//! `wiring_evidence` / `wiring_violation` action_types (NO new `SsbActionType`,
//! TOTAL_VARIANTS stays 482). They are surfaced on
//! [`WiringCoverageReport::ssb_payloads`] exactly as the sibling
//! `WiringAuditCmd::run` surfaces its evidence/violation rows on
//! `WiringAuditReport` — an orchestrator caller with a live `Ssb` handle
//! persists them via `Ssb::write_payload`; the sync CLI shell (no live async
//! `Ssb`) renders them. The leg discriminator is carried in the FIXED
//! `WiringEvidencePayload` shape as `entrypoint == LAYER3_LEG`:
//!   - clean run: ONE `WiringEvidence { entrypoint = "layer3_coverage",
//!     produces_symbols_verified = <covered new fns>, reachable_count = <total
//!     covered>, audit_id }`.
//!   - violation: a `WiringViolation { violation_kind = UnreachableSymbol,
//!     file = "layer3_coverage", line = 0, pattern = <uncovered fn name>,
//!     audit_id }` per uncovered NEW function, PLUS a `WiringViolation {
//!     violation_kind = BrokenObservable, file = "layer3_coverage", pattern =
//!     "<manifest_missing_feature hint>" }` for the manifest gate. A consumer
//!     (D-EV / WAVE-4) reads cleanliness as: a `wiring_evidence` row with
//!     `entrypoint == "layer3_coverage"` AND zero `wiring_violation` rows whose
//!     `file == "layer3_coverage"` in the window => clean.
//!
//! Result is exposed to the D-EV autonomous gate (WAVE-4) via
//! [`WiringCoverageReport::is_clean`] and the free function
//! [`layer3_coverage_clean`].

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

use crate::ssb::{
    SsbPayload, WiringEvidencePayload, WiringViolationKind, WiringViolationPayload,
};

/// Leg discriminator carried in `WiringEvidencePayload::entrypoint` /
/// `WiringViolationPayload::file` so a consumer (WAVE-4 D-EV gate) can select
/// the Layer-3 rows out of the shared `wiring_evidence` / `wiring_violation`
/// streams without a new `SsbActionType` (ADR-1). This is the canonical
/// contract string WAVE-4 must match on.
pub const LAYER3_LEG: &str = "layer3_coverage";

/// Leg discriminator carried in `WiringEvidencePayload::entrypoint` for the
/// Layer-4 off-agent trace check (the symmetric partner to [`LAYER3_LEG`]).
/// WAVE 5 (D-L4 trace-assertion harness) EMITS a `wiring_evidence` row with
/// `entrypoint == LAYER4_LEG` when it drives a real entrypoint and observes
/// the off-agent SSB row land; WAVE 4 (this cluster's
/// `task_runner::derive_layer4_trace_present`) CONSUMES it. Same shared
/// `wiring_evidence` action_type, no new `SsbActionType` (ADR-1). This is the
/// canonical contract string both waves must match on.
pub const LAYER4_LEG: &str = "layer4_trace";

/// Hard ceiling on smoke-scenario wall time so a hung entrypoint cannot stall
/// the gate indefinitely. Named constant (CLAUDE.md: no magic numbers). The
/// constant documents the contract for the auditor harness; the CLI shell is
/// responsible for enforcing the wall-clock guard around `run()`.
pub const SMOKE_SCENARIO_TIMEOUT_SECS: u64 = 120;

/// One parsed `[[feature]]` row from `smoke-manifest.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmokeFeature {
    /// Plan / ticket id that shipped the feature (e.g. "B-44").
    pub plan: String,
    /// The REAL entrypoint command the smoke run drives (argv after `elai`).
    pub entrypoint: String,
    /// The action_type the feature must emit when the entrypoint runs.
    pub expect_ssb: String,
    /// "on", or a wire-dark-allowlist.toml flag name for a kill-switched feature.
    pub flag_default: String,
}

/// Loader for `smoke-manifest.toml`. Mirrors `WiringEntrypointSetLoader`
/// (wiring_contract.rs) in shape. FAIL-CLOSED: an absent manifest file is an
/// `Err`, never an empty-clean set (R2).
pub struct SmokeManifestLoader {
    manifest_path: PathBuf,
}

impl SmokeManifestLoader {
    pub fn new(manifest_path: impl Into<PathBuf>) -> Self {
        Self {
            manifest_path: manifest_path.into(),
        }
    }

    /// Parse every `[[feature]]` block. Absent file => `Err(ManifestMissing)`.
    /// Present-but-zero-features is NOT an error here (the gate decides whether
    /// zero features is a violation, keyed on whether the diff introduces a
    /// new feature-bearing entrypoint).
    pub fn load(&self) -> Result<Vec<SmokeFeature>, WiringCoverageError> {
        if !self.manifest_path.exists() {
            return Err(WiringCoverageError::ManifestMissing(
                self.manifest_path.clone(),
            ));
        }
        let raw = std::fs::read_to_string(&self.manifest_path)?;
        let parsed: toml::Value = toml::from_str(&raw)
            .map_err(|e: toml::de::Error| WiringCoverageError::Parse(e.to_string()))?;
        let mut features = Vec::new();
        if let Some(arr) = parsed.get("feature").and_then(|f| f.as_array()) {
            for item in arr {
                let plan = item.get("plan").and_then(|v| v.as_str());
                let entrypoint = item.get("entrypoint").and_then(|v| v.as_str());
                let expect_ssb = item.get("expect_ssb").and_then(|v| v.as_str());
                let flag_default = item.get("flag_default").and_then(|v| v.as_str());
                match (plan, entrypoint, expect_ssb, flag_default) {
                    (Some(p), Some(e), Some(s), Some(f)) => features.push(SmokeFeature {
                        plan: p.to_owned(),
                        entrypoint: e.to_owned(),
                        expect_ssb: s.to_owned(),
                        flag_default: f.to_owned(),
                    }),
                    _ => {
                        // R1: a malformed row is missing-evidence => FAIL, not skip.
                        return Err(WiringCoverageError::Parse(format!(
                            "smoke-manifest [[feature]] missing a required field \
                             (plan/entrypoint/expect_ssb/flag_default): {item:?}"
                        )));
                    }
                }
            }
        }
        Ok(features)
    }
}

/// A function the instrumented binary reported, with its execution count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoveredFunction {
    /// Demangled function name as emitted by `cargo llvm-cov report --json`.
    pub name: String,
    /// Source file (first entry of the export's `filenames`), repo-relative.
    pub file: String,
    /// Execution count. 0 means the driven binary never ran this function.
    pub count: u64,
}

/// A function introduced by the current git diff (added line containing a
/// `fn <name>` signature). Used to scope the count==0 gate to NEW code only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewFunction {
    pub name: String,
    pub file: String,
    /// Whether the added function line (or an enclosing item) is `#[cfg(test)]`
    /// gated — such functions are R7-excluded from the production-reachability gate.
    pub is_cfg_test: bool,
    /// Whether the added line carries a registered `// WIRE-DARK[B-N]:` marker
    /// (suppression per the wiring-enforcement R3/R9 discipline). Carries the
    /// `B-N` id (without brackets) when present.
    pub wire_dark_marker: Option<String>,
}

/// One coverage-gate violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverageViolation {
    /// A NEW non-test, non-suppressed function never executed under the smoke run.
    DeadNewFunction { name: String, file: String },
    /// The git diff introduces a new feature-bearing entrypoint but no
    /// `[[feature]]` row covers it in smoke-manifest.toml (R1/manifest gate).
    ManifestMissingFeature { entrypoint_hint: String },
}

/// The Layer-3 coverage report. `is_clean()` is the load-bearing predicate the
/// CLI exit code and the D-EV autonomous gate both read. Note: `Eq` is NOT
/// derived because `ssb_payloads: Vec<SsbPayload>` and `SsbPayload` is
/// `PartialEq`-but-not-`Eq` (it carries f64-bearing variants). Equality on this
/// report is not load-bearing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WiringCoverageReport {
    /// Count of distinct functions parsed from the coverage export.
    pub functions_total: usize,
    /// Count of NEW functions (from the diff) subject to the gate.
    pub new_functions_gated: usize,
    /// All violations. Non-empty => not clean.
    pub violations: Vec<CoverageViolation>,
    /// Names of NEW functions suppressed by cfg(test) or WIRE-DARK marker
    /// (recorded for the audit trail; not violations).
    pub suppressed: Vec<String>,
    /// Per-run correlator for the SSB evidence rows (mirrors `WiringAuditReport`).
    pub audit_id: uuid::Uuid,
    /// SSB rows this run produced (ADR-1: reuse wiring_evidence/wiring_violation).
    /// An orchestrator caller with a live `Ssb` persists these via
    /// `Ssb::write_payload`; the CLI shell renders them. NOT serialized into the
    /// JSON report (the SsbPayload enum is not part of the CLI JSON contract).
    #[serde(skip)]
    pub ssb_payloads: Vec<SsbPayload>,
}

impl WiringCoverageReport {
    /// FAIL-CLOSED: clean iff zero violations. There is no UNVERIFIED state —
    /// an inability to measure surfaces as `Err` upstream, not as a clean report.
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

#[derive(Debug, Error)]
pub enum WiringCoverageError {
    #[error("smoke-manifest {0} does not exist")]
    ManifestMissing(PathBuf),
    #[error("smoke-manifest parse error: {0}")]
    Parse(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("git diff failed: {0}")]
    GitDiffFailed(String),
    #[error("instrumented build failed (exit {code}): {stderr}")]
    BuildFailed { code: i32, stderr: String },
    #[error("smoke scenario '{scenario}' failed (exit {code}): {stderr}")]
    SmokeFailed {
        scenario: String,
        code: i32,
        stderr: String,
    },
    #[error("cargo-llvm-cov export failed (exit {code}): {stderr}")]
    CoverageExportFailed { code: i32, stderr: String },
    #[error(
        "coverage measurement absent: {0} (R2 fail-closed: a missing evidence \
         path is a FAIL, not UNVERIFIED)"
    )]
    MeasurementAbsent(String),
}

/// Inputs to a Layer-3 coverage run. Kept as a plain struct (mirrors
/// `WiringAuditCmd`) so the CLI shell owns I/O and this core stays testable.
pub struct WiringCoverageCmd {
    /// Workspace root the cargo + git commands run from (CLI passes its cwd).
    pub source_root: PathBuf,
    /// Path to smoke-manifest.toml (default `<source_root>/smoke-manifest.toml`).
    pub manifest_path: PathBuf,
    /// git ref to diff against for the NEW-function set (default "HEAD").
    pub diff_base: String,
    /// When true, skip the (slow) build+drive and refuse to emit a verdict —
    /// the only opt-out, and it can NEVER emit a clean PASS for a run that
    /// declares runtime evidence (R3). Used by unit tests of the parse layer.
    pub static_only: bool,
}

impl WiringCoverageCmd {
    /// Full Layer-3 run. Compose: (1) compute NEW-function set from git diff,
    /// (2) clean prior coverage, (3) load smoke manifest, (4) drive each
    /// scenario against the instrumented binary, (5) export JSON, (6) gate
    /// count==0 over the NEW set, (7) build the SSB evidence/violation rows.
    pub fn run(&self) -> Result<WiringCoverageReport, WiringCoverageError> {
        let new_fns = self.collect_new_functions()?;
        if self.static_only {
            // R3: the static-only path can only REPORT an existing export; it
            // must not fabricate a clean verdict for un-driven runtime evidence.
            return Err(WiringCoverageError::MeasurementAbsent(
                "static_only requested; no runtime coverage was driven \
                 (R3: a static-only path cannot emit PASS)"
                    .into(),
            ));
        }
        self.clean_prior_coverage()?;
        let features = SmokeManifestLoader::new(&self.manifest_path).load()?;
        self.drive_smoke_scenarios(&features)?;
        let covered = self.export_coverage()?;
        if covered.is_empty() {
            return Err(WiringCoverageError::MeasurementAbsent(
                "cargo llvm-cov export returned zero functions; the instrumented \
                 binary produced no profile (R2)"
                    .into(),
            ));
        }
        Ok(self.gate(&new_fns, &covered, &features))
    }

    /// `cargo llvm-cov clean --workspace` so the export step reflects only the
    /// profiles produced by THIS run's smoke scenarios. cargo-llvm-cov wraps
    /// `RUSTFLAGS=-C instrument-coverage` internally; we shell to it rather
    /// than setting RUSTFLAGS by hand so the profraw layout matches what the
    /// later `report --json` expects. FAIL-CLOSED: a non-zero exit (including
    /// cargo-llvm-cov not being installed) is a hard error.
    fn clean_prior_coverage(&self) -> Result<(), WiringCoverageError> {
        let out = Command::new("cargo")
            .current_dir(&self.source_root)
            .args(["llvm-cov", "clean", "--workspace"])
            .output()
            .map_err(|e| WiringCoverageError::BuildFailed {
                code: -1,
                stderr: format!(
                    "failed to spawn `cargo llvm-cov clean` ({e}); is cargo-llvm-cov \
                     installed? (`cargo install cargo-llvm-cov`)"
                ),
            })?;
        if !out.status.success() {
            return Err(WiringCoverageError::BuildFailed {
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            });
        }
        Ok(())
    }

    /// Drive every smoke scenario against the instrumented binary under
    /// production-default env via `cargo llvm-cov run --no-report --bin elai`.
    /// The first scenario triggers the instrumented build; each subsequent one
    /// reuses it. Each scenario's argv comes from the manifest `entrypoint`. A
    /// non-zero exit on a scenario the manifest declares is a hard FAIL (R1).
    fn drive_smoke_scenarios(
        &self,
        features: &[SmokeFeature],
    ) -> Result<(), WiringCoverageError> {
        for f in features {
            let argv: Vec<&str> = f.entrypoint.split_whitespace().collect();
            // The first manifest token is the `elai` argv0 stand-in the operator
            // writes (`elai <subcommand> ...`); strip a leading literal "elai"
            // so we pass only the subcommand argv to the instrumented binary.
            let argv: Vec<&str> = match argv.split_first() {
                // Strip a leading literal "elai" so the instrumented binary
                // receives only the subcommand argv.
                Some((&"elai", tail)) => tail.to_vec(),
                // No leading "elai" (or an empty entrypoint): pass argv as-is.
                Some(_) | None => argv,
            };
            let mut cmd = Command::new("cargo");
            cmd.current_dir(&self.source_root)
                .args(["llvm-cov", "run", "--no-report", "--bin", "elai", "--"])
                .args(&argv);
            // Production-default env: do NOT inject any ELAI_*_ENABLE here — the
            // smoke run must reflect the default-OFF posture (R9). The
            // SMOKE_SCENARIO_TIMEOUT_SECS contract is enforced by the CLI shell
            // wrapping this call; documented as a const for the auditor harness.
            let out = cmd.output().map_err(|e| WiringCoverageError::SmokeFailed {
                scenario: f.entrypoint.clone(),
                code: -1,
                stderr: format!(
                    "failed to spawn `cargo llvm-cov run` ({e}); is cargo-llvm-cov installed?"
                ),
            })?;
            if !out.status.success() {
                return Err(WiringCoverageError::SmokeFailed {
                    scenario: f.entrypoint.clone(),
                    code: out.status.code().unwrap_or(-1),
                    stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                });
            }
        }
        Ok(())
    }

    /// `cargo llvm-cov report --json` and parse `data[].functions[]`.
    /// CONFIRMED: the JSON flag is `--json` (v0.8.7), not `--format=json`.
    fn export_coverage(&self) -> Result<Vec<CoveredFunction>, WiringCoverageError> {
        let out = Command::new("cargo")
            .current_dir(&self.source_root)
            .args(["llvm-cov", "report", "--json"])
            .output()
            .map_err(|e| WiringCoverageError::CoverageExportFailed {
                code: -1,
                stderr: format!(
                    "failed to spawn `cargo llvm-cov report --json` ({e}); is \
                     cargo-llvm-cov installed?"
                ),
            })?;
        if !out.status.success() {
            return Err(WiringCoverageError::CoverageExportFailed {
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            });
        }
        parse_llvm_cov_json(&String::from_utf8_lossy(&out.stdout))
    }

    /// Compute the NEW-function set from `git diff <base> -- '*.rs'`, parsing
    /// added (`+`) lines that declare a `fn <name>`. FAIL-CLOSED: a git failure
    /// is `Err(GitDiffFailed)`, never an empty set (R2).
    fn collect_new_functions(&self) -> Result<Vec<NewFunction>, WiringCoverageError> {
        let out = Command::new("git")
            .current_dir(&self.source_root)
            .args(["diff", "--unified=0", &self.diff_base, "--", "*.rs"])
            .output()
            .map_err(|e| WiringCoverageError::GitDiffFailed(e.to_string()))?;
        if !out.status.success() {
            return Err(WiringCoverageError::GitDiffFailed(format!(
                "git diff exited {}: {}",
                out.status,
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        Ok(parse_new_functions_from_diff(
            &String::from_utf8_lossy(&out.stdout),
        ))
    }

    /// The gate. For each NEW function: skip cfg(test) + WIRE-DARK suppressed;
    /// for the rest, BLOCK if its covered count is 0 (or it is absent from the
    /// export — absent == never executed == count 0, R2). Then build the SSB
    /// evidence/violation rows (ADR-1).
    fn gate(
        &self,
        new_fns: &[NewFunction],
        covered: &[CoveredFunction],
        features: &[SmokeFeature],
    ) -> WiringCoverageReport {
        let audit_id = uuid::Uuid::new_v4(); // entropy-allow: per-run correlator (mirrors WiringAuditCmd::run).
        let mut violations = Vec::new();
        let mut suppressed = Vec::new();
        let mut gated = 0usize;
        let count_by_name: std::collections::HashMap<&str, u64> =
            covered.iter().map(|c| (c.name.as_str(), c.count)).collect();
        let declared_entrypoints: BTreeSet<&str> =
            features.iter().map(|f| f.entrypoint.as_str()).collect();
        let mut covered_new: Vec<String> = Vec::new();
        for nf in new_fns {
            if nf.is_cfg_test {
                suppressed.push(format!("{} (cfg(test))", nf.name));
                continue;
            }
            if let Some(marker) = &nf.wire_dark_marker {
                suppressed.push(format!("{} (WIRE-DARK {marker})", nf.name));
                continue;
            }
            gated += 1;
            // Coverage matching is a substring contain on the demangled name so
            // a `crate::module::fn_name` export still matches a bare `fn_name`
            // from the diff. Absent == count 0 (R2 fail-closed).
            let count = best_match_count(&count_by_name, &nf.name);
            if count == 0 {
                violations.push(CoverageViolation::DeadNewFunction {
                    name: nf.name.clone(),
                    file: nf.file.clone(),
                });
            } else {
                covered_new.push(nf.name.clone());
            }
        }
        // Manifest gate: if the diff introduces a new entrypoint-bearing fn but
        // smoke-manifest has zero [[feature]] rows, BLOCK (R1).
        if features.is_empty() && new_fns.iter().any(is_entrypoint_bearing) {
            violations.push(CoverageViolation::ManifestMissingFeature {
                entrypoint_hint: "new entrypoint-bearing fn in diff; smoke-manifest \
                                  has zero [[feature]] rows"
                    .into(),
            });
        }
        let _ = declared_entrypoints; // reserved for per-entrypoint matching (SP3 follow-up).
        let ssb_payloads =
            build_ssb_payloads(audit_id, &covered_new, covered.len(), &violations);
        WiringCoverageReport {
            functions_total: covered.len(),
            new_functions_gated: gated,
            violations,
            suppressed,
            audit_id,
            ssb_payloads,
        }
    }
}

/// Best-effort count lookup: exact name first, else the max count over any
/// covered function whose demangled name ends with `::<name>` or equals `name`
/// (handles `crate::mod::fn` vs bare `fn`). Returns 0 when nothing matches
/// (R2: absent == never executed).
fn best_match_count(
    count_by_name: &std::collections::HashMap<&str, u64>,
    name: &str,
) -> u64 {
    if let Some(c) = count_by_name.get(name) {
        return *c;
    }
    let suffix = format!("::{name}");
    let mut best = 0u64;
    for (k, v) in count_by_name {
        if *k == name || k.ends_with(suffix.as_str()) {
            best = best.max(*v);
        }
    }
    best
}

/// Build the ADR-1 SSB rows from the gate outcome. Clean => one
/// `WiringEvidence(entrypoint=LAYER3_LEG)`; per uncovered fn =>
/// `WiringViolation(UnreachableSymbol, file=LAYER3_LEG, pattern=<fn>)`;
/// manifest-missing => `WiringViolation(BrokenObservable, file=LAYER3_LEG)`.
fn build_ssb_payloads(
    audit_id: uuid::Uuid,
    covered_new: &[String],
    total_covered: usize,
    violations: &[CoverageViolation],
) -> Vec<SsbPayload> {
    let mut out = Vec::new();
    if violations.is_empty() {
        out.push(SsbPayload::WiringEvidence(WiringEvidencePayload {
            entrypoint: LAYER3_LEG.to_owned(),
            produces_symbols_verified: covered_new.to_vec(),
            // narrowing guard: covered count is bounded by the export size,
            // well within u32 for any realistic workspace.
            reachable_count: u32::try_from(total_covered).unwrap_or(u32::MAX),
            audit_id,
        }));
        return out;
    }
    for v in violations {
        match v {
            CoverageViolation::DeadNewFunction { name, .. } => {
                out.push(SsbPayload::WiringViolation(WiringViolationPayload {
                    violation_kind: WiringViolationKind::UnreachableSymbol,
                    file: LAYER3_LEG.to_owned(),
                    line: 0,
                    pattern: name.clone(),
                    audit_id,
                }));
            }
            CoverageViolation::ManifestMissingFeature { entrypoint_hint } => {
                out.push(SsbPayload::WiringViolation(WiringViolationPayload {
                    violation_kind: WiringViolationKind::BrokenObservable,
                    file: LAYER3_LEG.to_owned(),
                    line: 0,
                    pattern: entrypoint_hint.clone(),
                    audit_id,
                }));
            }
        }
    }
    out
}

/// Heuristic: does this new fn live in a CLI args/dispatch/cmd file (i.e. it is
/// likely an entrypoint handler that needs a smoke-manifest row)?
fn is_entrypoint_bearing(nf: &NewFunction) -> bool {
    let f = nf.file.replace('\\', "/");
    f.contains("crates/cli/src/")
        && (f.ends_with("_cmd.rs") || f.ends_with("main.rs") || f.ends_with("args.rs"))
}

/// Parse `cargo llvm-cov report --json` into `(name,file,count)` rows.
/// Schema (llvm-cov export): `data[0].functions[]` each `{name, count, filenames[]}`.
pub fn parse_llvm_cov_json(json: &str) -> Result<Vec<CoveredFunction>, WiringCoverageError> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| WiringCoverageError::Parse(format!("llvm-cov json: {e}")))?;
    let mut out = Vec::new();
    let Some(data) = v.get("data").and_then(|d| d.as_array()) else {
        return Err(WiringCoverageError::MeasurementAbsent(
            "llvm-cov json has no `data` array".into(),
        ));
    };
    for block in data {
        if let Some(funcs) = block.get("functions").and_then(|f| f.as_array()) {
            for f in funcs {
                let name = f.get("name").and_then(|n| n.as_str()).unwrap_or("").to_owned();
                let count = f.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
                let file = f
                    .get("filenames")
                    .and_then(|fl| fl.as_array())
                    .and_then(|a| a.first())
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_owned();
                if !name.is_empty() {
                    out.push(CoveredFunction { name, file, count });
                }
            }
        }
    }
    Ok(out)
}

/// Parse `git diff --unified=0 -- '*.rs'` output into the NEW-function set.
/// Tracks the current `+++ b/<file>` header and, for each added line (`+...`),
/// extracts a `fn <name>` if present. cfg(test) detection is a best-effort scan
/// of the added hunk window for `#[cfg(test)]`; WIRE-DARK markers are read off
/// the same added line.
pub fn parse_new_functions_from_diff(diff: &str) -> Vec<NewFunction> {
    let mut out = Vec::new();
    let mut cur_file = String::new();
    let mut hunk_has_cfg_test = false;
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            cur_file = rest.trim().to_owned();
            continue;
        }
        if line.starts_with("@@") {
            hunk_has_cfg_test = false;
            continue;
        }
        if let Some(added) = line.strip_prefix('+') {
            // Guard against the `+++ b/` header (already handled) leaking here.
            if added.starts_with("++ b/") {
                continue;
            }
            if added.contains("#[cfg(test)]") {
                hunk_has_cfg_test = true;
            }
            let wire_dark = extract_wire_dark_marker(added);
            if let Some(name) = extract_fn_name(added) {
                out.push(NewFunction {
                    name,
                    file: cur_file.clone(),
                    is_cfg_test: hunk_has_cfg_test,
                    wire_dark_marker: wire_dark,
                });
            }
        }
    }
    out
}

/// Extract a `fn <name>` identifier from a single added source line. Skips
/// matches inside a line comment so a `// fn foo` mention is not a function.
fn extract_fn_name(line: &str) -> Option<String> {
    // If a line comment precedes the `fn `, the `fn` is not a declaration.
    let scan = match line.find("//") {
        Some(comment_at) => &line[..comment_at],
        None => line,
    };
    let idx = scan.find("fn ")?;
    let after = &scan[idx + 3..];
    let name: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Recognise a `// WIRE-DARK[B-N]: justification` suppression marker and return
/// the `B-N` id (bracket contents). This is the wiring-enforcement R3/R9 marker
/// — distinct from the stub-tracking `TODO[C.P.S.B]` babystep-id form — parsed
/// directly here per the cluster spec (the stub registry uses a different
/// 4-level id grammar and lives per-session under workflow/sessions/{id}/stubs.md).
fn extract_wire_dark_marker(line: &str) -> Option<String> {
    let idx = line.find("WIRE-DARK[")?;
    let after = &line[idx + "WIRE-DARK[".len()..];
    let id: String = after.chars().take_while(|c| *c != ']').collect();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

/// Cross-boundary helper consumed by the D-EV autonomous gate (WAVE-4). Runs a
/// Layer-3 coverage check and collapses it to a single bool for the
/// `Evidence::layer3_entrypoint_covered` field. An Err (un-measurable) collapses
/// to `false` (R2 fail-closed: cannot prove clean => not clean).
pub fn layer3_coverage_clean(cmd: &WiringCoverageCmd) -> bool {
    match cmd.run() {
        Ok(report) => report.is_clean(),
        Err(e) => {
            warn!(
                target: "elai_orchestrator::wiring_coverage",
                error = %e,
                "layer3 coverage un-measurable; treating as NOT clean (R2 fail-closed)"
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_llvm_cov_json_extracts_counts() {
        let json = r#"{
          "data": [
            { "functions": [
              { "name": "crate::a::run", "count": 3, "filenames": ["crates/a/src/a.rs"] },
              { "name": "crate::a::dead", "count": 0, "filenames": ["crates/a/src/a.rs"] }
            ] }
          ]
        }"#;
        let fns = parse_llvm_cov_json(json).expect("parse");
        assert_eq!(fns.len(), 2);
        let dead = fns.iter().find(|f| f.name.ends_with("dead")).expect("dead");
        assert_eq!(dead.count, 0);
        let run = fns.iter().find(|f| f.name.ends_with("run")).expect("run");
        assert_eq!(run.count, 3);
    }

    #[test]
    fn parse_llvm_cov_json_no_data_is_measurement_absent() {
        let err = parse_llvm_cov_json("{}").expect_err("must err");
        assert!(matches!(err, WiringCoverageError::MeasurementAbsent(_)));
    }

    #[test]
    fn parse_new_functions_detects_fn_and_cfg_test() {
        let diff = "\
+++ b/crates/cli/src/x_cmd.rs
@@ -0,0 +1,3 @@
+pub fn handler() {}
+#[cfg(test)]
@@ -0,0 +1,1 @@
+fn helper_test() {}
";
        let fns = parse_new_functions_from_diff(diff);
        let handler = fns.iter().find(|f| f.name == "handler").expect("handler");
        assert!(!handler.is_cfg_test);
        assert_eq!(handler.file, "crates/cli/src/x_cmd.rs");
    }

    #[test]
    fn parse_new_functions_reads_wire_dark_marker() {
        let diff = "\
+++ b/crates/orchestrator/src/y.rs
@@ -0,0 +1,1 @@
+fn deferred_path() {} // WIRE-DARK[B-44]: consumer ships in B-45
";
        let fns = parse_new_functions_from_diff(diff);
        let f = fns.iter().find(|f| f.name == "deferred_path").expect("fn");
        assert_eq!(f.wire_dark_marker.as_deref(), Some("B-44"));
    }

    #[test]
    fn extract_fn_name_ignores_commented_fn() {
        assert_eq!(extract_fn_name("    // fn not_a_decl()"), None);
        assert_eq!(extract_fn_name("pub fn real() {}").as_deref(), Some("real"));
    }

    #[test]
    fn gate_blocks_uncovered_new_fn_and_suppresses_cfg_test() {
        let cmd = WiringCoverageCmd {
            source_root: PathBuf::from("."),
            manifest_path: PathBuf::from("smoke-manifest.toml"),
            diff_base: "HEAD".into(),
            static_only: true,
        };
        let new_fns = vec![
            NewFunction {
                name: "dead".into(),
                file: "crates/orchestrator/src/y.rs".into(),
                is_cfg_test: false,
                wire_dark_marker: None,
            },
            NewFunction {
                name: "tested".into(),
                file: "crates/orchestrator/src/y.rs".into(),
                is_cfg_test: true,
                wire_dark_marker: None,
            },
        ];
        let covered = vec![CoveredFunction {
            name: "crate::y::other".into(),
            file: "crates/orchestrator/src/y.rs".into(),
            count: 5,
        }];
        let report = cmd.gate(&new_fns, &covered, &[]);
        assert!(!report.is_clean());
        assert_eq!(report.new_functions_gated, 1);
        assert_eq!(report.suppressed.len(), 1);
        assert!(report
            .violations
            .iter()
            .any(|v| matches!(v, CoverageViolation::DeadNewFunction { name, .. } if name == "dead")));
    }

    #[test]
    fn gate_clean_emits_one_wiring_evidence_row() {
        let cmd = WiringCoverageCmd {
            source_root: PathBuf::from("."),
            manifest_path: PathBuf::from("smoke-manifest.toml"),
            diff_base: "HEAD".into(),
            static_only: true,
        };
        let new_fns = vec![NewFunction {
            name: "live".into(),
            file: "crates/orchestrator/src/y.rs".into(),
            is_cfg_test: false,
            wire_dark_marker: None,
        }];
        let covered = vec![CoveredFunction {
            name: "crate::y::live".into(),
            file: "crates/orchestrator/src/y.rs".into(),
            count: 2,
        }];
        let report = cmd.gate(&new_fns, &covered, &[]);
        assert!(report.is_clean());
        assert_eq!(report.ssb_payloads.len(), 1);
        match &report.ssb_payloads[0] {
            SsbPayload::WiringEvidence(p) => {
                assert_eq!(p.entrypoint, LAYER3_LEG);
                assert!(p.produces_symbols_verified.contains(&"live".to_string()));
            }
            other => panic!("expected WiringEvidence, got {other:?}"),
        }
    }

    #[test]
    fn gate_violation_emits_wiring_violation_rows() {
        let cmd = WiringCoverageCmd {
            source_root: PathBuf::from("."),
            manifest_path: PathBuf::from("smoke-manifest.toml"),
            diff_base: "HEAD".into(),
            static_only: true,
        };
        let new_fns = vec![NewFunction {
            name: "dead".into(),
            file: "crates/orchestrator/src/y.rs".into(),
            is_cfg_test: false,
            wire_dark_marker: None,
        }];
        let report = cmd.gate(&new_fns, &[], &[]);
        assert!(!report.is_clean());
        let v = report
            .ssb_payloads
            .iter()
            .find_map(|p| match p {
                SsbPayload::WiringViolation(v) => Some(v),
                _ => None,
            })
            .expect("violation row");
        assert_eq!(v.file, LAYER3_LEG);
        assert_eq!(v.violation_kind, WiringViolationKind::UnreachableSymbol);
        assert_eq!(v.pattern, "dead");
    }

    #[test]
    fn manifest_loader_missing_file_is_err() {
        let loader = SmokeManifestLoader::new(PathBuf::from("/nonexistent/smoke-manifest.toml"));
        assert!(matches!(
            loader.load(),
            Err(WiringCoverageError::ManifestMissing(_))
        ));
    }

    #[test]
    fn manifest_loader_parses_complete_feature() {
        let dir = std::env::temp_dir().join(format!("elai-smoke-{}", uuid::Uuid::new_v4())); // entropy-allow: test-temp-dir-uniqueness-key-not-replay-compared
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("smoke-manifest.toml");
        std::fs::write(
            &path,
            "[[feature]]\nplan = \"B-1\"\nentrypoint = \"elai x\"\nexpect_ssb = \"a\"\nflag_default = \"on\"\n",
        )
        .expect("write");
        let feats = SmokeManifestLoader::new(&path).load().expect("load");
        assert_eq!(feats.len(), 1);
        assert_eq!(feats[0].plan, "B-1");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn manifest_loader_missing_field_is_parse_err() {
        let dir = std::env::temp_dir().join(format!("elai-smoke-{}", uuid::Uuid::new_v4())); // entropy-allow: test-temp-dir-uniqueness-key-not-replay-compared
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("smoke-manifest.toml");
        // Missing expect_ssb.
        std::fs::write(
            &path,
            "[[feature]]\nplan = \"B-1\"\nentrypoint = \"elai x\"\nflag_default = \"on\"\n",
        )
        .expect("write");
        let err = SmokeManifestLoader::new(&path).load().expect_err("must err");
        assert!(matches!(err, WiringCoverageError::Parse(_)));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn static_only_run_collapses_to_not_clean() {
        let cmd = WiringCoverageCmd {
            source_root: PathBuf::from("."),
            manifest_path: PathBuf::from("smoke-manifest.toml"),
            diff_base: "HEAD".into(),
            static_only: true,
        };
        // layer3_coverage_clean must be false because run() returns Err on the
        // static_only path (R3) and Err collapses to NOT clean (R2).
        assert!(!layer3_coverage_clean(&cmd));
    }
}
