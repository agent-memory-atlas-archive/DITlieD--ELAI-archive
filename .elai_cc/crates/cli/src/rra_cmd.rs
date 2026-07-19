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
use elai_orchestrator::rra::{
    export_fixture, read_rra, verify_artifact_tiered, ReplayVerdict, RunArtifact, TierVerdict,
    TierVerdictLabel, VerifyOutcome,
};
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
    CliOutputFormat, GoldenDemoMode, RerunDemoMode, ReplayAction, ReplayVerifyTier,
    ResultManifestDemoMode, RraAction,
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
        // J-14 — `elai rra export-fixture <bundle> [--adapter <name>]`: export
        // a content-keyed cassette from an RRA bundle. Scrubs auth headers,
        // keys the cassette by `(model, content_blake3, tool_name)`, writes the
        // `.fixture` file, and prints the key fields + path. Exits nonzero on
        // failure (mirrors run_verify's exit contract).
        RraAction::ExportFixture { bundle, adapter } => {
            run_export_fixture(&bundle, &adapter, output_format)
        }
    }
}

/// J-14 — `elai rra export-fixture` handler. Drives
/// [`elai_orchestrator::rra::export_fixture`], prints the key fields + the
/// fixture path on success, and exits nonzero on failure (mirrors
/// `run_verify`'s exit contract). The SSB-ready `ReproPackPayload` is
/// returned by `export_fixture`; this handler logs its fields (an SSB emit
/// helper, if wired in a later slice, would land it as a `ReproPackEvent`
/// row — rra.rs has no `Ssb` handle and the task contract forbids touching
/// ssb/* from this rider).
fn run_export_fixture(
    bundle: &std::path::Path,
    adapter: &str,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let result = export_fixture(bundle, adapter, "", None)
        .map_err(|e| format!("exporting fixture from {}: {e}", bundle.display()))?;
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| format!("serializing export-fixture result as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("elai rra export-fixture (J-14, content-keyed cassette)");
            println!("  bundle         : {}", bundle.display());
            println!("  fixture_path   : {}", result.fixture_path.display());
            println!("  fixture_blake3 : {}", result.cassette.fixture_blake3);
            println!(
                "  key.model      : {}",
                result.cassette.key.model
            );
            println!(
                "  key.content    : {}",
                result.cassette.key.content_blake3
            );
            println!(
                "  key.tool_name  : {}",
                result.cassette.key.tool_name
            );
            println!("  session_id     : {}", result.cassette.session_id);
            println!("  adapter        : {}", result.cassette.adapter);
            println!();
            println!(
                "  ReproPackEvent payload (kind=FixtureExtracted): fixture_path={:?}, fixture_blake3={:?}, session_id={}",
                result.payload.fixture_path, result.payload.fixture_blake3, result.payload.session_id
            );
        }
    }
    Ok(())
}

/// `elai replay <sub>` dispatcher invoked by main.rs.
pub(crate) fn run_replay(
    action: ReplayAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        ReplayAction::Verify { path, current_objective, tier } => {
            run_verify(&path, current_objective.as_deref(), tier, output_format)
        }
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

fn run_verify(
    path: &std::path::Path,
    current_objective: Option<&str>,
    tier: Option<ReplayVerifyTier>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let art = read_rra(path).map_err(|e| format!("reading .rra at {}: {e}", path.display()))?;
    // J-14: default tier is T0 (ActionFaithful) — the G-2 binary verdict.
    // A `--tier t1|t2` request upgrades to the graded path; the effective tier
    // is always <= requested (never a silent upgrade, enforced by
    // `grade_tier` in rra.rs).
    let tier_requested = match tier {
        None | Some(ReplayVerifyTier::T0) => TierVerdictLabel::ActionFaithful,
        Some(ReplayVerifyTier::T1) => TierVerdictLabel::TokenFaithful,
        Some(ReplayVerifyTier::T2) => TierVerdictLabel::ScoreFaithful,
    };
    let outcome = verify_artifact_tiered(&art, current_objective, tier_requested)
        .map_err(|e| format!("verifying .rra at {}: {e}", path.display()))?;
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&outcome)
                .map_err(|e| format!("serializing verify outcome as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            render_outcome_text(path, &art, &outcome, tier_requested);
        }
    }
    // Exit non-zero on any non-Verified binary verdict so CI can gate on it.
    // D-016: ObjectiveUnchecked is NON-fatal (exit 0) — the artifact is
    // cryptographically intact; only the objective-staleness check is skipped
    // because no current_objective was supplied. ObjectiveHashMismatch remains
    // exit 1 (objective was redefined after sealing).
    // J-14: a `MissingDeterminismBasis` tier verdict (tier>T0 requested but
    // the bundle carries no `determinism_applied` block) is NON-fatal at T0
    // (T0 needs no basis) and FATAL at T1/T2 (R2: missing evidence = FAIL).
    let binary_ok = matches!(
        outcome.verdict,
        ReplayVerdict::Verified | ReplayVerdict::ObjectiveUnchecked { .. }
    );
    let tier_ok = match outcome.tier {
        None => true,
        Some(TierVerdict::MissingDeterminismBasis) => tier_requested == TierVerdictLabel::ActionFaithful,
        _ => true,
    };
    if binary_ok && tier_ok {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn render_outcome_text(
    path: &std::path::Path,
    art: &RunArtifact,
    outcome: &VerifyOutcome,
    tier_requested: TierVerdictLabel,
) {
    render_verdict_text(path, &outcome.verdict);
    // J-14: tier block. Printed on every Verified-path outcome (the tier is
    // `None` only when the binary verdict short-circuited, which is already
    // fatal above). The tier line is the human-readable summary; the
    // determinism_applied + downgrade lines follow when present.
    if let Some(tier) = &outcome.tier {
        render_tier_text(tier, tier_requested);
    }
    render_determinism_applied_text(art);
}

fn render_tier_text(tier: &TierVerdict, tier_requested: TierVerdictLabel) {
    let requested_label = tier_label_str(tier_requested);
    match tier {
        TierVerdict::ActionFaithful => {
            println!("  tier           : action_faithful (T0, requested {requested_label})");
        }
        TierVerdict::TokenFaithful => {
            println!("  tier           : token_faithful (T1, requested {requested_label})");
        }
        TierVerdict::ScoreFaithful => {
            println!("  tier           : score_faithful (T2, requested {requested_label})");
        }
        TierVerdict::Downgraded { downgrade } => {
            println!(
                "  tier           : DOWNGRADED {} -> {} (requested {}): {}",
                tier_label_str(downgrade.from),
                tier_label_str(downgrade.to),
                requested_label,
                downgrade.reason
            );
        }
        TierVerdict::MissingDeterminismBasis => {
            println!(
                "  tier           : MISSING-DETERMINISM-BASIS (requested {requested_label} but the bundle carries no determinism_applied block; R2: missing evidence = FAIL)"
            );
        }
    }
}

fn tier_label_str(label: TierVerdictLabel) -> &'static str {
    match label {
        TierVerdictLabel::ActionFaithful => "t0/action_faithful",
        TierVerdictLabel::TokenFaithful => "t1/token_faithful",
        TierVerdictLabel::ScoreFaithful => "t2/score_faithful",
    }
}

fn render_determinism_applied_text(art: &RunArtifact) {
    match &art.determinism_applied {
        None => {
            println!("  determinism    : (none — legacy pre-J-14 or producer did not populate)");
        }
        Some(da) => {
            println!("  determinism    :");
            println!("    seed               : {}", da.seed);
            if let Some(ss) = da.sampling_seed {
                println!("    sampling_seed      : {ss}");
            }
            println!("    hardware_class     : {}", da.hardware_class);
            if let Some(pin) = &da.system_fingerprint_pin {
                println!("    fingerprint_pin    : {pin}");
            }
            if !da.env_controls.is_empty() {
                println!("    env_controls       : {}", da.env_controls.join(", "));
            }
            println!("    fingerprint_blake3 : {}", da.fingerprint_blake3);
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
            first_divergence,
        } => {
            println!(
                "DIVERGENCE {} — first mismatching event seq {} (expected {}, got {})",
                path.display(),
                first_mismatch_event,
                expected_blake3,
                got_blake3
            );
            // J-14: first-divergence locator for T1/T2 mismatch (token-stream
            // position + expected/got token bytes). `None` on T0 action-stream
            // divergence (the `first_mismatch_event` already locates it) and on
            // LEGACY receipts (pre-J-14).
            if let Some(fd) = first_divergence {
                println!(
                    "  first_divergence: position {} (tier {}), expected {}, got {}",
                    fd.position,
                    tier_label_str(fd.tier),
                    hex_encode(&fd.expected),
                    hex_encode(&fd.got)
                );
            }
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

/// J-14: hex-encode a byte slice for the first-divergence locator line. Kept
/// inline (rather than pulling a `hex` crate dep) because the divergence
/// payload is short (a token / score fingerprint) and the CLI already avoids
/// a `hex` dependency.
fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
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
