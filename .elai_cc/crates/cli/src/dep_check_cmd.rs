//! F-9 / Slice D4 — `elai dep-check <path> [--offline]` CLI handler.
//!
//! Reads the file at `path`, extracts dependency import lines, runs
//! `DepWriteTimeGate.analyze_collect`, prints one BLOCK/WARN line per finding,
//! and emits `WriteTimeSlopsquatBlocked` SSB rows (durably) for each
//! `NotFound`/`TooNew` finding to the canonical session `ssb.db`.
//!
//! Mode selection (W-10/W-5):
//!   * DEFAULT (no flag): `SubprocessRegistryChecker` — the REAL network
//!     detection leg (shells out to `registry_check.py`). This is the gated
//!     live run that actually blocks a 404 crate.
//!   * `--offline`: `StubRegistryChecker` — the deterministic demo / contract
//!     harness. Offline mode is SAFE on real code (only curated
//!     known-hallucinated names Block; everything else is Found, WARN-3) and
//!     exercises the SAME real gate code path as the live leg, so it is the
//!     deterministic proof of the gate's wiring.
//!
//! Exit codes:
//!   0 = all deps clean (no Block findings)
//!   1 = at least one Block finding emitted
//!
//! BLOCK-3 (observable_effect): `main()` is NOT `#[tokio::main]`, so this
//! handler builds its OWN current-thread runtime and opens `ssb.db` INSIDE
//! `block_on` (Unit-C `safety_cmd::write_tool_filter_ssb_rows` pattern —
//! `Ssb::new` spawns the single-writer task and MUST be constructed inside the
//! runtime, else it panics "no reactor running"). The rows are AWAITED (not
//! spawn-detached) so they are durable before the process exits and the 2s
//! observable-effect window can observe them.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::vibesec::dep_writetime_gate::{
    BlockedDepRecord, DepWriteTimeGate, RegistryExistenceChecker, StubRegistryChecker,
    SubprocessRegistryChecker,
};
use elai_orchestrator::hook_dispatcher::HookVerdictKind;
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};

/// CLI entry point.  Called from `main.rs` dispatch arm.
///
/// * `path`     — file path to scan (positional arg from args parser)
/// * `offline`  — when `true` use `StubRegistryChecker` (deterministic demo);
///               otherwise use `SubprocessRegistryChecker` (the real live
///               network leg, the DEFAULT).
///
/// Opens its own `ssb.db` to emit `WriteTimeSlopsquatBlocked` rows durably
/// (BLOCK-3); no SSB handle is passed in by the caller.
pub fn run(path: String, offline: bool) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("dep-check: cannot read '{}': {e}", path))?;

    let checker: Arc<dyn RegistryExistenceChecker + Send + Sync> = if offline {
        Arc::new(StubRegistryChecker)
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        // SubprocessRegistryChecker resolves registry_check.py relative to
        // <project_root>/.elai_cc/checkers/registry/registry_check.py.
        Arc::new(SubprocessRegistryChecker::default_at_root(&cwd))
    };

    // Build the gate WITHOUT an internal SSB sink — the CLI writes rows itself,
    // durably, via block_on (the gate's internal emit is spawn-detached and not
    // durable across an immediate process exit; BLOCK-3).
    let gate = DepWriteTimeGate::new(None, checker);

    let (findings, blocked) = gate.analyze_collect(&source, &path);

    let mut has_block = false;
    for finding in &findings {
        match finding.verdict {
            HookVerdictKind::Block => {
                has_block = true;
                println!("write-time slop block: {}", finding.message);
            }
            HookVerdictKind::Warn => {
                println!("write-time slop warn: {}", finding.message);
            }
            HookVerdictKind::Allow => {}
        }
    }

    if findings.is_empty() {
        println!("dep-check: all deps clean ({})", path);
    }

    // BLOCK-3 observable_effect: write one WriteTimeSlopsquatBlocked row per
    // blocked dep, durably, to the canonical session ssb.db. Best-effort: a
    // DB-open failure is logged but does not change the exit code (the gate
    // verdict already printed).
    if !blocked.is_empty() {
        if let Err(e) = write_slop_ssb_rows(&blocked) {
            eprintln!("dep-check: SSB emit failed (non-fatal): {e}");
        }
    }

    if has_block {
        std::process::exit(1);
    }
    Ok(())
}

/// Open the canonical session `ssb.db` and write each blocked dep as a
/// `WriteTimeSlopsquatBlocked` row, awaited (durable).  Mirrors the Unit-C
/// `safety_cmd::write_tool_filter_ssb_rows` pattern.
fn write_slop_ssb_rows(blocked: &[BlockedDepRecord]) -> anyhow::Result<()> {
    let project_root: PathBuf = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("dep-check: cannot determine cwd: {e}"))?;
    let db_path = sessions_dir_path(&project_root).join("ssb.db");
    let parent = db_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("dep-check: ssb.db has no parent dir"))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| anyhow::anyhow!("dep-check: create sessions dir failed: {e}"))?;

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-dep-check".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("dep-check: build async runtime failed: {e}"))?;

    // Ssb::new spawns the single-writer task and MUST be constructed inside the
    // runtime context. Building it before rt.block_on panics with "there is no
    // reactor running, must be called from the context of a Tokio 1.x runtime".
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock)
            .map_err(|e| anyhow::anyhow!("dep-check: open SSB writer failed: {e}"))?;
        for record in blocked {
            let payload =
                SsbPayload::WriteTimeSlopsquatBlocked(DepWriteTimeGate::build_slop_payload(record));
            ssb.write_payload("cli:dep-check", 0, payload, Vec::new())
                .await
                .map_err(|e| anyhow::anyhow!("dep-check: SSB write failed: {e}"))?;
        }
        Ok::<(), anyhow::Error>(())
    })
}
