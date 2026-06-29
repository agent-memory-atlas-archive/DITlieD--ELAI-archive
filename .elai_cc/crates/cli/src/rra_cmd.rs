//! G-2 — `elai rra show <path>` + `elai replay --verify <path>` handlers.
//!
//! Drives the orchestrator's [`elai_orchestrator::rra`] module:
//!   * `rra show`  — read + render a Reproducible-Run Artifact (`.rra`)
//!     summary (seed + determinism pins + model pin + honest guarantee level +
//!     per-gate verdict manifest + blake3 root). Read-only; never re-executes.
//!   * `replay --verify` — recompute the blake3 root (tamper check) + re-derive
//!     every event content hash (divergence check), print the verdict, and exit
//!     non-zero on `Divergence` / `Tampered` so CI can gate on it.
//!
//! Honest scope (GD-5): the verdict is action-stream + verdict-hash
//! verify-divergence, NOT bit-identical token replay. A `Verified` means the
//! stream + every present F-gate verdict hash reproduced — it does NOT claim
//! the cloud-lane token stream was re-executed.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::rerun_equality::{
    verify_rerun_digest, MetricDigest, RerunInput, RerunVerdictKind, RunDigests,
    RERUN_EQUALITY_RULE_ID,
};
use elai_orchestrator::rra::{read_rra, verify_rra, ReplayVerdict, RunArtifact};
use elai_orchestrator::result_manifest::{
    verify_result, ReDerivedInputs, ResultEntry, ResultManifest, ResultVerdictKind,
    RESULT_MANIFEST_RULE_ID,
};
use elai_orchestrator::sim_real_parity::{
    verify_replay_golden, EventDigest, EventLog, SimRealInput, SimRealVerdictKind,
    SIM_REAL_PARITY_RULE_ID,
};
use elai_orchestrator::ssb::{Ssb, SsbConfig};

use crate::args::{
    CliOutputFormat, GoldenDemoMode, RerunDemoMode, ReplayAction, ResultManifestDemoMode, RraAction,
};

/// H-23 — session id for the re-run hash-equality fixture session.
const RERUN_SESSION_ID: &str = "elai-rerun-equality-fixture";

/// H-23 — project id for the re-run hash-equality fixture session.
const RERUN_PROJECT_ID: &str = "elai-rerun-equality";

/// H-23 — default declared seed for the Tier-1 fixtures (GD-2 reproducibility).
const RERUN_DEMO_SEED: u64 = 42;

/// H-23 — fixture strategy/experiment path reported on the SSB row.
const RERUN_DEMO_STRATEGY: &str = "demo_experiment.py";

/// H-24 — session id for the sim-to-real parity fixture session.
const PARITY_SESSION_ID: &str = "elai-sim-real-parity-fixture";

/// H-24 — project id for the sim-to-real parity fixture session.
const PARITY_PROJECT_ID: &str = "elai-sim-real-parity";

/// H-24 — default declared seed for the Tier-1 parity fixtures (GD-2).
const PARITY_DEMO_SEED: u64 = 42;

/// H-24 — fixture strategy path reported on the SimRealParity SSB row.
const PARITY_DEMO_STRATEGY: &str = "demo_strategy.py";

/// H-25 — session id for the result-manifest re-derivation fixture session.
const RESULT_MANIFEST_SESSION_ID: &str = "elai-result-manifest-fixture";

/// H-25 — project id for the result-manifest re-derivation fixture session.
const RESULT_MANIFEST_PROJECT_ID: &str = "elai-result-manifest";

/// H-25 — default declared seed for the Tier-1 result-manifest fixtures (GD-2).
const RESULT_MANIFEST_DEMO_SEED: u64 = 42;

/// H-25 — fixture strategy path reported on the ResultManifestVerified SSB row.
const RESULT_MANIFEST_DEMO_STRATEGY: &str = "demo_experiment.py";

/// H-25 — fixture run id bound by the demo manifest.
const RESULT_MANIFEST_DEMO_RUN_ID: &str = "run-demo-001";

/// `elai rra <sub>` dispatcher invoked by main.rs.
pub(crate) fn run_rra(action: RraAction, output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        RraAction::Show { path } => run_show(&path, output_format),
        // I-40 — spec-conformance check (FORK-2: DISTINCT from replay --verify which
        // re-derives content hashes; this checks C1-C6 structural requirements from
        // docs/spec/rra-spec-v1.md). Handler: crate::rra_verify_cmd.
        RraAction::Verify { path } => {
            crate::rra_verify_cmd::run_verify_conformance(path, output_format)
        }
    }
}

/// `elai replay <sub>` dispatcher invoked by main.rs.
pub(crate) fn run_replay(
    action: ReplayAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        ReplayAction::Verify { path } => run_verify(&path, output_format),
        // H-23 — same-seed re-run hash-equality METRIC_GATE (additive; the
        // `Verify { path }` arm above is unchanged).
        ReplayAction::VerifyRerun { demo } => run_verify_rerun(demo, output_format),
        // H-24 — sim-to-real parity gate (Part C; additive; the `Verify` +
        // `VerifyRerun` arms above are unchanged).
        ReplayAction::VerifyGolden { demo } => run_verify_golden(demo, output_format),
        // H-25 — result manifest re-derivation gate (additive; the `Verify` +
        // `VerifyRerun` + `VerifyGolden` arms above are unchanged).
        ReplayAction::VerifyResultManifest { demo } => run_verify_result(demo, output_format),
    }
}

fn run_show(path: &std::path::Path, output_format: CliOutputFormat) -> Result<(), String> {
    let art = read_rra(path).map_err(|e| format!("reading .rra at {}: {e}", path.display()))?;
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&art)
                .map_err(|e| format!("serializing .rra summary as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            render_show_text(&art);
        }
    }
    Ok(())
}

fn render_show_text(art: &RunArtifact) {
    println!("Reproducible-Run Artifact (.rra)");
    println!("  format_version : {}", art.format_version);
    println!("  session_id     : {}", art.session_id);
    println!("  seed           : {}", art.seed);
    println!("  hardware_class : {}", art.hardware_class);
    println!(
        "  fingerprint    : {}",
        art.system_fingerprint_pin.as_deref().unwrap_or("(none)")
    );
    println!(
        "  model_pin      : {}",
        art.model_pin.as_deref().unwrap_or("(none)")
    );
    println!("  guarantee      : {}", art.guarantee_level.as_sql_str());
    println!("  events         : {}", art.events.len());
    println!("  root_blake3    : {}", art.root_blake3);
    println!("  gate manifest  : {} entr(ies)", art.manifest.len());
    for g in &art.manifest {
        println!(
            "    [{:>4}] {:<5} {} @seq {} = {}",
            g.ssb_event_seq, g.gate, g.action_type, g.ssb_event_seq, g.verdict_blake3
        );
    }
    if art.manifest.is_empty() {
        println!("    (no F-gate verdicts present — a valid gate-off state, not a failure)");
    }
    // I-40 Step 5 — render the action-stream event rows so wrapped-run rows
    // (taint_flow + run_artifact_emitted from `elai wrap`) are visible in the
    // existing RRA viewer. Capped to avoid flooding the terminal on a long
    // `elai run` bundle; the full stream is always available via --output json.
    render_event_stream(art);
}

/// I-40 Step 5 — render the action-stream event rows in the text viewer.
///
/// Each event row shows its append-only seq, its SSB action_type wire tag, and
/// its sealed content hash (event_blake3). This makes wrapped-run rows
/// (taint_flow from the untrusted-ingest / blocked-write gates, and the
/// run_artifact_emitted beacon) visible when an operator runs
/// `elai rra show <wrap-fixture.rra>`. The dashboard `RraViewerPanel` already
/// surfaces the run_artifact_emitted beacon via `get_run_artifact` (no session
/// filter), so the frontend_pair is satisfied without a new Svelte component.
///
/// Capped at [`EVENT_STREAM_RENDER_CAP`] rows to bound terminal output; a
/// `... N more` indicator is printed when the stream is truncated. The full
/// event stream is always available via `--output json` (the JSON path
/// serializes the entire `RunArtifact` including every event).
const EVENT_STREAM_RENDER_CAP: usize = 50;

fn render_event_stream(art: &RunArtifact) {
    if art.events.is_empty() {
        return;
    }
    println!("  event stream   :");
    let total = art.events.len();
    let shown = total.min(EVENT_STREAM_RENDER_CAP);
    for ev in &art.events[..shown] {
        println!(
            "    [seq {:>4}] {:<28} {}",
            ev.seq, ev.action_type, ev.event_blake3
        );
    }
    if total > shown {
        println!("    ... {} more (use --output json for the full stream)", total - shown);
    }
}

fn run_verify(path: &std::path::Path, output_format: CliOutputFormat) -> Result<(), String> {
    let verdict =
        verify_rra(path).map_err(|e| format!("verifying .rra at {}: {e}", path.display()))?;
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&verdict)
                .map_err(|e| format!("serializing verdict as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            render_verdict_text(path, &verdict);
        }
    }
    // Exit non-zero on any non-Verified verdict so CI can gate on it. The
    // verdict has already been printed, so a hard exit here is the documented
    // contract (mirrors `elai freeze verify`).
    // D-016: ObjectiveUnchecked is NON-fatal (exit 0) — the artifact is
    // cryptographically intact; only the objective-staleness check is skipped
    // because no current_objective was supplied. ObjectiveHashMismatch remains
    // exit 1 (objective was redefined after sealing).
    match verdict {
        ReplayVerdict::Verified | ReplayVerdict::ObjectiveUnchecked { .. } => Ok(()),
        ReplayVerdict::Divergence { .. }
        | ReplayVerdict::Tampered { .. }
        | ReplayVerdict::ObjectiveHashMismatch { .. } => {
            std::process::exit(1);
        }
    }
}

fn render_verdict_text(path: &std::path::Path, verdict: &ReplayVerdict) {
    match verdict {
        ReplayVerdict::Verified => {
            println!(
                "VERIFIED {} — action stream + every present F-gate verdict hash reproduced",
                path.display()
            );
        }
        ReplayVerdict::Divergence {
            first_mismatch_event,
            expected_blake3,
            got_blake3,
        } => {
            println!(
                "DIVERGENCE {} — first mismatching event seq {} (expected {}, got {})",
                path.display(),
                first_mismatch_event,
                expected_blake3,
                got_blake3
            );
        }
        ReplayVerdict::Tampered {
            expected_root,
            got_root,
        } => {
            println!(
                "TAMPERED {} — bundle root does not recompute (expected {}, got {})",
                path.display(),
                expected_root,
                got_root
            );
        }
        // I-45: objective was redefined after sealing.
        ReplayVerdict::ObjectiveHashMismatch { sealed_hash, current_hash } => {
            println!(
                "OBJECTIVE-MISMATCH {} — sealed objective hash ({sealed_hash}) does not match \
                 current task objective hash ({current_hash}); objective was redefined after sealing",
                path.display()
            );
        }
        // I-45: bundle has an objective hash but no current objective was supplied for comparison.
        // Non-fatal (exit 0, D-016). Instruct the caller how to supply text for a full check.
        ReplayVerdict::ObjectiveUnchecked { sealed_hash } => {
            println!(
                "VERIFIED {} — artifact cryptographically intact. \
                 objective: UNCHECKED (sealed hash {sealed_hash} present but no current \
                 objective text supplied; pass --current-objective <text> to enable staleness check)",
                path.display()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// H-23 — `elai replay --verify --metric-digest <mode>` (re-run hash-equality)
// ---------------------------------------------------------------------------

/// H-23 — build the Tier-1 fixture: two declared-metric-output digest lists for
/// a same-seed re-run. `deterministic` produces two byte-identical lists (Equal
/// twice); `nondeterministic` makes run #2's `sharpe` digest differ (a hidden
/// unseeded RNG => Divergent). The digests are produced via
/// [`MetricDigest::from_bytes`] over deterministic fixture output bytes so the
/// gate's blake3 convention is exercised end-to-end (no live experiment).
fn rerun_demo_inputs(demo: RerunDemoMode) -> (RunDigests, RunDigests) {
    // Run #1's declared outputs (the same for both demos).
    let run_a = RunDigests {
        metrics: vec![
            MetricDigest::from_bytes("sharpe", b"sharpe=1.42;window=0..1;seed=42"),
            MetricDigest::from_bytes("max_drawdown", b"max_drawdown=-0.18;window=0..1;seed=42"),
        ],
    };

    let run_b = match demo {
        // A clean seeded experiment: run #2 hashes byte-identical declared
        // outputs (same bytes -> same blake3 digests) => Equal.
        RerunDemoMode::Deterministic => RunDigests {
            metrics: vec![
                MetricDigest::from_bytes("sharpe", b"sharpe=1.42;window=0..1;seed=42"),
                MetricDigest::from_bytes(
                    "max_drawdown",
                    b"max_drawdown=-0.18;window=0..1;seed=42",
                ),
            ],
        },
        // A hidden unseeded RNG: run #2's `sharpe` output bytes differ (an
        // un-pinned np.random draw perturbed the metric) => Divergent at index 0.
        RerunDemoMode::Nondeterministic => RunDigests {
            metrics: vec![
                MetricDigest::from_bytes(
                    "sharpe",
                    b"sharpe=1.39;window=0..1;rng=0x9e3779b9",
                ),
                MetricDigest::from_bytes(
                    "max_drawdown",
                    b"max_drawdown=-0.18;window=0..1;seed=42",
                ),
            ],
        },
    };

    (run_a, run_b)
}

/// H-23 — run the `elai replay --verify --metric-digest <mode>` gate.
///
/// Opens a real `Ssb`, drives [`verify_rerun_digest`] over the Tier-1 fixture
/// digests, prints the verdict, and exits non-zero on `Divergent` so CI can gate
/// on it (mirrors the `run_verify` exit contract). This is the W-1/W-3 production
/// caller for the re-run hash-equality gate.
fn run_verify_rerun(demo: RerunDemoMode, output_format: CliOutputFormat) -> Result<(), String> {
    let (run_a, run_b) = rerun_demo_inputs(demo);
    let mode_label = match demo {
        RerunDemoMode::Deterministic => "deterministic",
        RerunDemoMode::Nondeterministic => "nondeterministic",
    };

    let project_root =
        std::env::current_dir().map_err(|e| format!("resolving current dir: {e}"))?;
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("creating sessions dir {}: {e}", sessions_dir.display()))?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: RERUN_PROJECT_ID.to_string(),
        session_id: RERUN_SESSION_ID.to_string(),
    };

    let input = RerunInput {
        strategy_path: RERUN_DEMO_STRATEGY.to_string(),
        run_a,
        run_b: Some(run_b),
        // The declared seed is supplied explicitly (the gate fails closed on a
        // missing seed rather than inheriting the silent from_env seed=0).
        seed: Some(RERUN_DEMO_SEED),
    };

    // The gate is async and `Ssb::new` itself spawns its write-loop onto the
    // runtime; build a multi-thread runtime and construct the Ssb + drive the
    // gate INSIDE block_on so the reactor is present (mirrors how the sibling
    // H-17..H-22 CLI gates bridge to async in main.rs). Constructing the Ssb
    // outside the runtime context panics with "no reactor running".
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .map_err(|e| format!("building tokio runtime: {e}"))?;
    let verdict = runtime.block_on(async move {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(Ssb::new(cfg, clock).map_err(|e| format!("ssb init: {e}"))?);
        verify_rerun_digest(&ssb, input)
            .await
            .map_err(|e| format!("re-run hash-equality gate: {e}"))
    })?;

    let verdict_str = match verdict.kind {
        RerunVerdictKind::Equal => "EQUAL (reproducible across same-seed re-runs)",
        RerunVerdictKind::Divergent => "DIVERGENT (re-runs are NOT reproducible)",
    };

    match output_format {
        CliOutputFormat::Json => {
            let out = serde_json::json!({
                "rerun_hash_equality": {
                    "mode": mode_label,
                    "rule_id": RERUN_EQUALITY_RULE_ID,
                    "strategy": RERUN_DEMO_STRATEGY,
                    "verdict": verdict_str,
                    "reason": verdict.reason,
                    "divergent_metric": verdict.divergent_metric,
                    "first_divergence_index": verdict.first_divergence_index,
                    "declared_metric_count": verdict.declared_metric_count,
                    "digest_a": verdict.digest_a,
                    "digest_b": verdict.digest_b,
                    "seed": RERUN_DEMO_SEED,
                    "ssb_action_type": "rerun_hash_equality",
                }
            });
            let json = serde_json::to_string_pretty(&out)
                .map_err(|e| format!("serializing rerun verdict as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("elai replay --verify --metric-digest (H-23, W-1 production entrypoint)");
            println!("  mode:        {mode_label}");
            println!("  rule_id:     {RERUN_EQUALITY_RULE_ID}");
            println!("  strategy:    {RERUN_DEMO_STRATEGY}");
            println!("  seed:        {RERUN_DEMO_SEED}");
            println!();
            println!("  VERDICT:  {verdict_str}  (reason: {})", verdict.reason);
            println!(
                "  digest_a: {}   digest_b: {}   metrics: {}",
                verdict.digest_a, verdict.digest_b, verdict.declared_metric_count
            );
            if verdict.kind == RerunVerdictKind::Divergent && !verdict.divergent_metric.is_empty() {
                println!(
                    "  first divergent metric: '{}' (index {})",
                    verdict.divergent_metric, verdict.first_divergence_index
                );
            }
            println!();
            println!("  evidence: {}", verdict.evidence);
            println!();
            println!("  RerunHashEquality SSB row: see ssb.db action_type='rerun_hash_equality'");
        }
    }

    // Exit non-zero on a Divergent verdict so CI can gate on it (mirrors
    // run_verify's non-Verified exit contract). The verdict + SSB row have
    // already landed, so a hard exit here is the documented contract.
    match verdict.kind {
        RerunVerdictKind::Equal => Ok(()),
        RerunVerdictKind::Divergent => std::process::exit(1),
    }
}

// ---------------------------------------------------------------------------
// H-24 — `elai replay --verify --golden <mode>` (sim-to-real parity, Part C)
// ---------------------------------------------------------------------------

/// H-24 — build the Tier-1 fixture: a frozen golden event log + a seeded replay
/// event log. `parity` produces a replay byte-identical to the golden (Parity
/// PASS); `gap` makes the replay's fill event differ (a different fill model in
/// backtest => ReplayGoldenMismatch). The digests are produced via
/// [`EventDigest::from_bytes`] over deterministic fixture event bytes so the
/// gate's blake3 convention is exercised end-to-end (no live paper-trade feed).
fn golden_demo_inputs(demo: GoldenDemoMode) -> (EventLog, EventLog) {
    // The frozen golden event log (the same for both demos).
    let golden = EventLog {
        events: vec![
            EventDigest::from_bytes("order:AAPL", b"order AAPL 100 @ market;seed=42"),
            EventDigest::from_bytes("fill:AAPL", b"fill AAPL 100 @ 187.34;seed=42"),
        ],
    };

    let replay = match demo {
        // A clean single-entrypoint strategy: the seeded replay reproduces the
        // golden byte-for-byte (same event bytes -> same blake3 digests) => Parity.
        GoldenDemoMode::Parity => EventLog {
            events: vec![
                EventDigest::from_bytes("order:AAPL", b"order AAPL 100 @ market;seed=42"),
                EventDigest::from_bytes("fill:AAPL", b"fill AAPL 100 @ 187.34;seed=42"),
            ],
        },
        // A sim-real divergence: the replay's fill event differs (a different
        // backtest fill model filled at a better price) => ReplayGoldenMismatch
        // at event index 1.
        GoldenDemoMode::Gap => EventLog {
            events: vec![
                EventDigest::from_bytes("order:AAPL", b"order AAPL 100 @ market;seed=42"),
                EventDigest::from_bytes("fill:AAPL", b"fill AAPL 100 @ 186.91;backtest_fill_model"),
            ],
        },
    };

    (golden, replay)
}

/// H-24 — run the `elai replay --verify --golden <mode>` gate.
///
/// Opens a real `Ssb`, drives [`verify_replay_golden`] over the Tier-1 fixture
/// event logs, prints the verdict, and exits non-zero on `ReplayGoldenMismatch`
/// so CI can gate on it (mirrors the `run_verify` / `run_verify_rerun` exit
/// contract). This is the W-1/W-3 production caller for the sim-to-real parity
/// gate (Part C).
fn run_verify_golden(demo: GoldenDemoMode, output_format: CliOutputFormat) -> Result<(), String> {
    let (golden, replay) = golden_demo_inputs(demo);
    let mode_label = match demo {
        GoldenDemoMode::Parity => "parity",
        GoldenDemoMode::Gap => "gap",
    };

    let project_root =
        std::env::current_dir().map_err(|e| format!("resolving current dir: {e}"))?;
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("creating sessions dir {}: {e}", sessions_dir.display()))?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: PARITY_PROJECT_ID.to_string(),
        session_id: PARITY_SESSION_ID.to_string(),
    };

    let input = SimRealInput {
        strategy_path: PARITY_DEMO_STRATEGY.to_string(),
        golden,
        replay: Some(replay),
        seed: PARITY_DEMO_SEED,
    };

    // The gate is async and `Ssb::new` itself spawns its write-loop onto the
    // runtime; build a multi-thread runtime and construct the Ssb + drive the
    // gate INSIDE block_on so the reactor is present (mirrors the H-23 path).
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .map_err(|e| format!("building tokio runtime: {e}"))?;
    let verdict = runtime.block_on(async move {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(Ssb::new(cfg, clock).map_err(|e| format!("ssb init: {e}"))?);
        verify_replay_golden(&ssb, input)
            .await
            .map_err(|e| format!("sim-to-real parity gate: {e}"))
    })?;

    let verdict_str = match verdict.kind {
        SimRealVerdictKind::Parity => "PARITY (replay reproduced the frozen golden)",
        SimRealVerdictKind::ReplayGoldenMismatch => {
            "REPLAY_GOLDEN_MISMATCH (sim-to-real divergence)"
        }
    };

    match output_format {
        CliOutputFormat::Json => {
            let out = serde_json::json!({
                "sim_real_parity": {
                    "mode": mode_label,
                    "rule_id": SIM_REAL_PARITY_RULE_ID,
                    "strategy": PARITY_DEMO_STRATEGY,
                    "verdict": verdict_str,
                    "reason": verdict.reason,
                    "divergent_event": verdict.divergent_event,
                    "first_divergence_index": verdict.first_divergence_index,
                    "declared_event_count": verdict.declared_event_count,
                    "golden_digest": verdict.golden_digest,
                    "replay_digest": verdict.replay_digest,
                    "seed": PARITY_DEMO_SEED,
                    "ssb_action_type": "sim_real_parity",
                }
            });
            let json = serde_json::to_string_pretty(&out)
                .map_err(|e| format!("serializing parity verdict as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("elai replay --verify --golden (H-24, W-1 production entrypoint)");
            println!("  mode:        {mode_label}");
            println!("  rule_id:     {SIM_REAL_PARITY_RULE_ID}");
            println!("  strategy:    {PARITY_DEMO_STRATEGY}");
            println!("  seed:        {PARITY_DEMO_SEED}");
            println!();
            println!("  VERDICT:  {verdict_str}  (reason: {})", verdict.reason);
            println!(
                "  golden_digest: {}   replay_digest: {}   events: {}",
                verdict.golden_digest, verdict.replay_digest, verdict.declared_event_count
            );
            if verdict.kind == SimRealVerdictKind::ReplayGoldenMismatch
                && !verdict.divergent_event.is_empty()
            {
                println!(
                    "  first divergent event: '{}' (index {})",
                    verdict.divergent_event, verdict.first_divergence_index
                );
            }
            println!();
            println!("  evidence: {}", verdict.evidence);
            println!();
            println!("  SimRealParity SSB row: see ssb.db action_type='sim_real_parity'");
        }
    }

    // Exit non-zero on a ReplayGoldenMismatch verdict so CI can gate on it (the
    // verdict + SSB row have already landed, so a hard exit here is the contract).
    match verdict.kind {
        SimRealVerdictKind::Parity => Ok(()),
        SimRealVerdictKind::ReplayGoldenMismatch => std::process::exit(1),
    }
}

/// H-25 — build the Tier-1 fixture manifest + captured outputs for a demo mode.
///
/// The captured output is a single orchestrator-captured blob carrying two labeled
/// floats (`sharpe=1.85`, `maxdd=0.07`); the manifest declares those values + their
/// provenance digests. `Complete` declares the true values (=> ReDerived);
/// `Tampered` HAND-EDITS the sharpe to 2.99 (=> the value does not re-parse =>
/// FabricationDetected); `Incomplete` declares an empty manifest (=> trivial
/// manifest, anti-sandbagging => FabricationDetected).
fn result_manifest_demo_inputs(
    demo: ResultManifestDemoMode,
) -> (ResultManifest, Option<ReDerivedInputs>) {
    // The orchestrator-captured output blob (the off-agent artifact the agent did
    // not author). Both labeled floats live in the same captured blob.
    let captured = b"backtest complete\nsharpe=1.85\nmaxdd=0.07\n".to_vec();
    let captured_blake3 = {
        let full = blake3::hash(&captured).to_hex();
        full.chars().take(16).collect::<String>()
    };
    // The pinned-input provenance digests the manifest binds + the observed
    // re-hashes (they MATCH for Complete/Tampered so the only divergence in Tampered
    // is the hand-edited value, not the provenance).
    const DATA_HASH: &str = "data0000";
    const ENV_HASH: &str = "env00000";
    const COMMIT: &str = "commitabc";

    let entry = |name: &str, value: &str| ResultEntry {
        metric_name: name.to_string(),
        value: value.to_string(),
        captured_output_blake3: captured_blake3.clone(),
        parser_expr: format!("labeled-float:{name}="),
        data_hash: DATA_HASH.to_string(),
        code_commit: COMMIT.to_string(),
        env_lock_hash: ENV_HASH.to_string(),
        n_trials: 100,
        dsr_pvalue: Some(0.01),
        pbo: Some(0.12),
    };

    let rederived = ReDerivedInputs {
        captured_outputs: vec![
            ("sharpe".to_string(), captured.clone()),
            ("maxdd".to_string(), captured.clone()),
        ],
        data_bytes_hash: DATA_HASH.to_string(),
        env_bytes_hash: ENV_HASH.to_string(),
        code_commit_observed: COMMIT.to_string(),
    };

    match demo {
        ResultManifestDemoMode::Complete => (
            ResultManifest {
                strategy_path: RESULT_MANIFEST_DEMO_STRATEGY.to_string(),
                run_id: RESULT_MANIFEST_DEMO_RUN_ID.to_string(),
                elai_seed: Some(RESULT_MANIFEST_DEMO_SEED),
                entries: vec![entry("sharpe", "1.85"), entry("maxdd", "0.07")],
            },
            Some(rederived),
        ),
        ResultManifestDemoMode::Tampered => (
            ResultManifest {
                strategy_path: RESULT_MANIFEST_DEMO_STRATEGY.to_string(),
                run_id: RESULT_MANIFEST_DEMO_RUN_ID.to_string(),
                elai_seed: Some(RESULT_MANIFEST_DEMO_SEED),
                // The declared sharpe was HAND-EDITED to 2.99; the captured output
                // says 1.85 => the number does not re-derive.
                entries: vec![entry("sharpe", "2.99"), entry("maxdd", "0.07")],
            },
            Some(rederived),
        ),
        ResultManifestDemoMode::Incomplete => (
            ResultManifest {
                strategy_path: RESULT_MANIFEST_DEMO_STRATEGY.to_string(),
                run_id: RESULT_MANIFEST_DEMO_RUN_ID.to_string(),
                elai_seed: Some(RESULT_MANIFEST_DEMO_SEED),
                // An empty manifest: a reported number with a 0/empty manifest is
                // the anti-fabrication signature (anti-sandbagging).
                entries: vec![],
            },
            Some(rederived),
        ),
    }
}

/// H-25 — run the `elai replay --verify --result-manifest <mode>` gate.
///
/// Opens a real `Ssb`, drives [`verify_result`] over the Tier-1 fixture manifest +
/// captured outputs, prints the verdict, and exits non-zero on
/// `FabricationDetected` so CI can gate on it (mirrors the `run_verify_golden`
/// exit contract). This is the W-1/W-3 production caller for the result-manifest
/// re-derivation gate.
fn run_verify_result(
    demo: ResultManifestDemoMode,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let (manifest, rederived) = result_manifest_demo_inputs(demo);
    let mode_label = match demo {
        ResultManifestDemoMode::Complete => "complete",
        ResultManifestDemoMode::Tampered => "tampered",
        ResultManifestDemoMode::Incomplete => "incomplete",
    };

    let project_root =
        std::env::current_dir().map_err(|e| format!("resolving current dir: {e}"))?;
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("creating sessions dir {}: {e}", sessions_dir.display()))?;
    let db_path = sessions_dir.join("ssb.db");

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: RESULT_MANIFEST_PROJECT_ID.to_string(),
        session_id: RESULT_MANIFEST_SESSION_ID.to_string(),
    };

    // The gate is async and `Ssb::new` itself spawns its write-loop onto the
    // runtime; build a multi-thread runtime and construct the Ssb + drive the gate
    // INSIDE block_on so the reactor is present (mirrors the H-23/H-24 path).
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .map_err(|e| format!("building tokio runtime: {e}"))?;
    let verdict = runtime.block_on(async move {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(Ssb::new(cfg, clock).map_err(|e| format!("ssb init: {e}"))?);
        verify_result(&ssb, manifest, rederived)
            .await
            .map_err(|e| format!("result manifest re-derivation gate: {e}"))
    })?;

    let verdict_str = match verdict.kind {
        ResultVerdictKind::ReDerived => "RE_DERIVED (every declared number re-derives)",
        ResultVerdictKind::FabricationDetected => {
            "FABRICATION_DETECTED (a reported number did not re-derive)"
        }
    };

    match output_format {
        CliOutputFormat::Json => {
            let out = serde_json::json!({
                "result_manifest_verified": {
                    "mode": mode_label,
                    "rule_id": RESULT_MANIFEST_RULE_ID,
                    "strategy": RESULT_MANIFEST_DEMO_STRATEGY,
                    "run_id": RESULT_MANIFEST_DEMO_RUN_ID,
                    "verdict": verdict_str,
                    "reason": verdict.reason,
                    "offending_metric": verdict.offending_metric,
                    "first_divergence_index": verdict.first_divergence_index,
                    "declared_result_count": verdict.declared_result_count,
                    "manifest_digest": verdict.manifest_digest,
                    "rederived_digest": verdict.rederived_digest,
                    "seed": RESULT_MANIFEST_DEMO_SEED,
                    "ssb_action_type": "result_manifest_verified",
                }
            });
            let json = serde_json::to_string_pretty(&out)
                .map_err(|e| format!("serializing result-manifest verdict as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("elai replay --verify --result-manifest (H-25, W-1 production entrypoint)");
            println!("  mode:        {mode_label}");
            println!("  rule_id:     {RESULT_MANIFEST_RULE_ID}");
            println!("  strategy:    {RESULT_MANIFEST_DEMO_STRATEGY}");
            println!("  run_id:      {RESULT_MANIFEST_DEMO_RUN_ID}");
            println!("  seed:        {RESULT_MANIFEST_DEMO_SEED}");
            println!();
            println!("  VERDICT:  {verdict_str}  (reason: {})", verdict.reason);
            println!(
                "  manifest_digest: {}   rederived_digest: {}   results: {}",
                verdict.manifest_digest, verdict.rederived_digest, verdict.declared_result_count
            );
            if verdict.kind == ResultVerdictKind::FabricationDetected
                && !verdict.offending_metric.is_empty()
            {
                println!(
                    "  first non-re-deriving result: '{}' (index {})",
                    verdict.offending_metric, verdict.first_divergence_index
                );
            }
            println!();
            println!("  evidence: {}", verdict.evidence);
            println!();
            println!(
                "  ResultManifestVerified SSB row: see ssb.db action_type='result_manifest_verified'"
            );
        }
    }

    // Exit non-zero on a FabricationDetected verdict so CI can gate on it (the
    // verdict + SSB row have already landed, so a hard exit here is the contract).
    match verdict.kind {
        ResultVerdictKind::ReDerived => Ok(()),
        ResultVerdictKind::FabricationDetected => std::process::exit(1),
    }
}
