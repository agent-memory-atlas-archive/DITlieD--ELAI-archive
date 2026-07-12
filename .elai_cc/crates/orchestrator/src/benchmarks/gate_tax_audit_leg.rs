//! K-4 SP2 — MOVE 4b: the per-audit checker-timing leg (RECON_NEEDED-5
//! settled), split into its own sibling module so `gate_tax_runner.rs` stays
//! under the MR-1 1000-line boundary (maintainability-review.md).
//!
//! Times the 5 offline auditor checkers named in the plan's Reads §7 via
//! `std::process::Command` wall-clock, median over [`AUDIT_CHECKER_REPS`]
//! reps each, summed into `audit_gate_tax_ms`. MEMOIZED per-process
//! ([`AUDIT_GATE_TAX_CACHE`]): the per-audit tax is a machine/checker-set
//! property, not a per-seed per-call quantity. Computing it once per process
//! and reusing it preserves GD-2 same-seed byte-identity for the
//! `RunnerPresent` path in `gate_tax_runner.rs` (two independent live
//! wall-clock measurements of 15 subprocess spawns would never bit-match —
//! the same physical-reality tension already named for the coding leg's
//! `Measured` path, see `workflow/reasoning/K-4-decisions.md`'s GD-2 scope
//! clarification) while still reporting a REAL wall-clock number once per
//! `elai bench gate-tax` process invocation (the actual CLI use case: one
//! process per run).
//!
//! No `.unwrap()` / `.expect()` outside `#[cfg(test)]`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{mpsc, OnceLock};
use std::time::{Duration, Instant};

use tracing::warn;

use super::gate_tax_runner::median_ms;

/// Reps per checker. Subprocess spawn (~50-300ms cold Python interpreter
/// startup) is orders of magnitude costlier than the in-process
/// `ROUTE_B_MICROBENCH_REPS` fn calls in `gate_tax_runner.rs`, so this stays
/// a small K — enough to de-noise a single cold-start rep (ADV-2) without
/// making every `elai bench gate-tax` invocation (or every `cargo test` run
/// that touches this module, since the cache is process-wide) pay tens of
/// seconds. `pub(crate)` so `gate_tax_runner.rs`'s residue note can cite the
/// same number rather than duplicating the literal.
pub(crate) const AUDIT_CHECKER_REPS: usize = 3;

/// Interpreter binary for every audit-checker subprocess. Mirrors the
/// existing `Command::new("python")` precedent
/// (crates/cli/src/plan_status_cmd.rs::run_wiring_check).
const AUDIT_CHECKER_PYTHON_BIN: &str = "python";

/// Per-subprocess-rep timeout: a hung/slow checker degrades that ONE checker
/// to "unavailable" (contributes 0.0, named in residue) rather than hanging
/// the whole bench run.
const AUDIT_CHECKER_TIMEOUT_SECS: u64 = 30;

/// Fixture inputs (Reads §7): real, stable, already-shipped files chosen so
/// every checker does REAL representative work (real git subprocess calls,
/// real file I/O, real regex scanning) without depending on there being a
/// live in-progress changeset. Never live-changeset-derived (a fixture path
/// must stay available on any dev box, K-18a precedent).
const AUDIT_RS_FIXTURE_REL: &str = ".elai_cc/crates/orchestrator/src/benchmarks/gate_tax_runner.rs";
const AUDIT_PLAN_FIXTURE_REL: &str = "docs/thefinalglance/K-plan/K-4_composed_gate_tax_budget.md";
const AUDIT_SVELTE_FIXTURE_REL: &str = ".elai_cc/crates/dashboard-v2/src/lib/ui/Card.svelte";

/// One offline auditor checker's fixture invocation.
struct AuditCheckerInvocation {
    checker_id: &'static str,
    /// Relative to WORKFLOW_ROOT (the dir carrying `.claude/tools`).
    script_rel: &'static str,
    args: Vec<String>,
}

/// Memoized result of the per-audit checker-timing leg (see the module doc
/// above for why this is process-wide memoized, not re-timed every call).
/// Fields are `pub(crate)` so `gate_tax_runner.rs`'s `build_residue_note`
/// can read them without a getter-wrapper layer.
pub(crate) struct AuditGateTaxResult {
    pub(crate) total_ms: f64,
    /// checker_ids that could not be spawned/timed at all (script missing,
    /// interpreter missing, or a rep timed out) — GD-5 named residue, never
    /// silently dropped from the sum (they simply contribute 0.0).
    pub(crate) unavailable: Vec<String>,
}

static AUDIT_GATE_TAX_CACHE: OnceLock<AuditGateTaxResult> = OnceLock::new();

/// Walk up from `cwd` looking for the dir carrying `.claude/tools` (the
/// WORKFLOW_ROOT every offline checker script resolves against internally,
/// e.g. `production_caller_check.py`'s
/// `WORKFLOW_ROOT = Path(__file__).resolve().parents[2]`). `elai bench
/// gate-tax` may be invoked from `.elai_cc` (this crate's typical cwd) or
/// from the workflow root itself — an ancestor walk covers both, mirroring
/// the precedent at crates/tools/src/lib.rs::resolve_skill_path.
fn resolve_workflow_root(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors()
        .find(|a| a.join(".claude").join("tools").is_dir())
        .map(Path::to_path_buf)
}

fn build_audit_checker_invocations(workflow_root: &Path) -> Vec<AuditCheckerInvocation> {
    let rs_fixture = workflow_root.join(AUDIT_RS_FIXTURE_REL).display().to_string();
    let plan_fixture = workflow_root.join(AUDIT_PLAN_FIXTURE_REL).display().to_string();
    let svelte_fixture = workflow_root.join(AUDIT_SVELTE_FIXTURE_REL).display().to_string();
    vec![
        AuditCheckerInvocation {
            checker_id: "production_caller_check",
            script_rel: ".claude/tools/production_caller_check.py",
            args: vec!["--files".to_string(), rs_fixture.clone(), "--json".to_string()],
        },
        AuditCheckerInvocation {
            checker_id: "maintainability_check",
            script_rel: ".claude/tools/maintainability_check.py",
            args: vec!["--files".to_string(), rs_fixture, "--json".to_string()],
        },
        AuditCheckerInvocation {
            checker_id: "wiring_contract_check",
            script_rel: ".claude/tools/wiring_contract_check.py",
            args: vec!["--bn-file".to_string(), plan_fixture.clone(), "--json".to_string()],
        },
        AuditCheckerInvocation {
            checker_id: "plan_gate_check",
            script_rel: ".claude/tools/plan_gate_check.py",
            args: vec!["--file".to_string(), plan_fixture, "--json".to_string()],
        },
        AuditCheckerInvocation {
            checker_id: "taste_check",
            script_rel: ".claude/tools/taste_check.py",
            args: vec!["--files".to_string(), svelte_fixture, "--json".to_string()],
        },
    ]
}

/// Spawn `python <script> <args>` and wait up to `AUDIT_CHECKER_TIMEOUT_SECS`.
/// Mirrors the bounded-wait idiom at
/// crates/cli/src/plan_status_cmd.rs::run_wiring_check (thread + mpsc
/// `recv_timeout`, since `Child::wait` has no built-in deadline). The exit
/// code is intentionally IGNORED — MOVE 4b measures wall-clock cost, not
/// verdict; a checker legitimately exits nonzero against a fixture with
/// findings to report, and that is still real per-audit wall-clock.
fn run_checker_once(script_abs: &Path, args: &[String]) -> Result<(), String> {
    let mut child = Command::new(AUDIT_CHECKER_PYTHON_BIN)
        .arg(script_abs)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let (tx, rx) = mpsc::channel::<std::io::Result<std::process::ExitStatus>>();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait());
    });
    match rx.recv_timeout(Duration::from_secs(AUDIT_CHECKER_TIMEOUT_SECS)) {
        Ok(Ok(_status)) => Ok(()),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_timeout) => Err(format!("checker timed out after {AUDIT_CHECKER_TIMEOUT_SECS}s")),
    }
}

/// Median wall-clock (ms) of one checker's subprocess invocation over
/// `AUDIT_CHECKER_REPS` reps. `None` when the checker script is missing or
/// any rep could not be spawned/timed at all (the whole checker is then
/// named in `AuditGateTaxResult::unavailable`, never silently averaged over
/// fewer reps).
fn time_audit_checker(script_abs: &Path, args: &[String]) -> Option<f64> {
    if !script_abs.is_file() {
        return None;
    }
    let mut samples = Vec::with_capacity(AUDIT_CHECKER_REPS);
    for _ in 0..AUDIT_CHECKER_REPS {
        let start = Instant::now(); // entropy-allow: monotonic start-marker for audit-checker subprocess latency telemetry only; a seeded clock would not measure real execution cost
        if let Err(e) = run_checker_once(script_abs, args) {
            warn!(script = %script_abs.display(), error = %e, "gate-tax: audit checker rep failed; marking unavailable");
            return None;
        }
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    Some(median_ms(samples))
}

/// Best-effort compute of the full per-audit leg. Any resolution failure
/// (cwd, WORKFLOW_ROOT) degrades to a single named "all-checkers" residue
/// entry and `total_ms = 0.0` — never a hard error (ABORT-2's "missing
/// evidence degrades gracefully" posture, applied to the audit leg).
fn compute_audit_gate_tax_result() -> AuditGateTaxResult {
    let cwd = match std::env::current_dir() {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "gate-tax: could not determine cwd for audit-checker leg");
            return AuditGateTaxResult {
                total_ms: 0.0,
                unavailable: vec!["all-checkers (cwd unresolved)".to_string()],
            };
        }
    };
    let workflow_root = match resolve_workflow_root(&cwd) {
        Some(r) => r,
        None => {
            warn!(cwd = %cwd.display(), "gate-tax: could not resolve WORKFLOW_ROOT for audit-checker leg");
            return AuditGateTaxResult {
                total_ms: 0.0,
                unavailable: vec!["all-checkers (WORKFLOW_ROOT unresolved)".to_string()],
            };
        }
    };
    let invocations = build_audit_checker_invocations(&workflow_root);
    let mut total_ms = 0.0f64;
    let mut unavailable = Vec::new();
    for inv in &invocations {
        let script_abs = workflow_root.join(inv.script_rel);
        match time_audit_checker(&script_abs, &inv.args) {
            Some(median) => total_ms += median,
            None => unavailable.push(inv.checker_id.to_string()),
        }
    }
    AuditGateTaxResult { total_ms, unavailable }
}

/// Memoized accessor — see the module doc for why this is process-wide
/// memoized rather than re-timed on every `run_gate_tax` call. `pub(crate)`
/// so `gate_tax_runner.rs::run_gate_tax` can call it.
pub(crate) fn audit_gate_tax() -> &'static AuditGateTaxResult {
    AUDIT_GATE_TAX_CACHE.get_or_init(compute_audit_gate_tax_result)
}
