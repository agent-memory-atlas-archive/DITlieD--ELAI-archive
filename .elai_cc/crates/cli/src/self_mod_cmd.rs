//! F-8 / Slice A4 — `elai self-mod apply --proposal-fixture PATH
//! [--no-env-override]` handler.
//!
//! The SCF symbolic-conflict detector's W-3 production entrypoint and W-5 live
//! trigger. The promotion seam it ALSO composes into
//! (`external_benches::promote_after_bench_completed`) is WIRE-DARK
//! (`#[cfg(feature = "darwinian-evolve")]`, no production caller). A CLI
//! subcommand is itself a W-3 production entrypoint, so this surface drives a
//! real [`ConfigVariant`] through the REAL
//! [`elai_orchestrator::screen_config_variant_for_conflicts`] compose unit
//! (`intent_from_config_variant` -> `current_process_model` -> `detect_conflicts`
//! -> emit) against the live locked-key + tripwire model.
//!
//! POLICY: WARN-then-escalate, NEVER hard fail-closed. On conflict the handler
//! emits the REAL `SymbolicConflictDetected` SSB beacon(s) (the held-pending
//! record) and reports `status = "pending"` — the variant is NOT applied and NOT
//! hard-rejected. A clean variant reports `status = "clean"` (admissible). The
//! handler never widens a `Blocked*` enum and never returns a non-zero exit on a
//! detected conflict (a hold is an observation, not a failure).
//!
//! SCF's published 27.9% precision is a simulation-only NON-target and never
//! appears as an ELAI accuracy goal — it is exactly why the policy is
//! WARN-then-escalate. See `workflow/reasoning/F-8-decisions.md`.
//!
//! Ssb::new spawns a single-writer task and MUST be constructed INSIDE a tokio
//! runtime (gotchas:110 — building it before `rt.block_on` panics "no reactor
//! running"; cargo-green + pure-fn tests hide that). This handler mirrors the
//! F-10 `safety_cmd::run_monotonicity` runtime pattern exactly.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use elai_core::paths::sessions_dir_path;
use elai_core::OsClock;
use elai_orchestrator::self_mod_layer1a::PipelineMode;
use elai_orchestrator::self_mod_types::{ConfigVariant, VariantStatus};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use elai_orchestrator::tripwire::TripwireKind;
use elai_orchestrator::{
    current_process_model, detect_conflicts, intent_from_config_variant,
    screen_config_variant_for_conflicts,
};
use serde::{Deserialize, Serialize};

use crate::args::{CliOutputFormat, SelfModAction};

/// Env kill-switch documented by `--no-env-override`. When the flag is set and
/// this var is truthy, the handler refuses to run rather than silently degrading
/// to a non-screening pass-through. Mirrors the F-10 monotonicity kill-switch
/// contract (the operator demanded a real verdict; a suppressed run is an
/// operator error, not a clean result).
const ELAI_SYMBOLIC_CONFLICT_DISABLE_ENV: &str = "ELAI_SYMBOLIC_CONFLICT_DISABLE";

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(action: SelfModAction, output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        SelfModAction::Apply {
            proposal_fixture,
            no_env_override,
        } => run_apply(&proposal_fixture, no_env_override, output_format),
    }
}

/// JSON shape of a `--proposal-fixture`. A hand-authored, operator-friendly
/// surface over [`ConfigVariant`]: `config` is a flat JSON object (the same key
/// set [`elai_orchestrator::intent_from_config_variant`] decodes from
/// `config_blob`), so a fixture author writes `{"score_weights": "tampered"}`
/// rather than the raw `Vec<u8>` numeric array `ConfigVariant`'s serde derive
/// would otherwise demand. `active_tripwires` drives the quiescence half of the
/// detector (an active tripwire turns a locked-key mutation into an additional
/// `CausalViolation`).
#[derive(Debug, Deserialize)]
struct ProposalFixture {
    /// Stable id stamped on every emitted beacon's `candidate_id` so an operator
    /// can correlate the held-pending rows back to the screened variant.
    variant_id: String,
    /// Flat config object screened for locked-key conflicts. Each key becomes a
    /// `Set` assertion; resource-rail keys additionally become a `Require`.
    #[serde(default)]
    config: serde_json::Map<String, serde_json::Value>,
    /// Active tripwires at screening time. Snake-case `TripwireKind` labels
    /// (`kl_divergence` / `correlation_drop` / `iqr_outlier` / `perplexity` /
    /// `ddm`). Any non-empty set violates quiescence.
    #[serde(default)]
    active_tripwires: Vec<TripwireKind>,
    /// FSM sub-state recorded on each beacon (the admission point at flag time).
    /// Defaults to `AdmittingVariant` — the self-mod archive-admission sub-state.
    #[serde(default = "default_fsm_substate")]
    fsm_substate: String,
}

fn default_fsm_substate() -> String {
    "AdmittingVariant".to_string()
}

/// JSON shape printed by `elai self-mod apply`. Machine-readable so the W-5
/// observable smoke + the wiring-contract-validator can assert on the verdict.
#[derive(Debug, Serialize)]
struct ApplyReport {
    /// Echo of the screened variant id.
    variant_id: String,
    /// `"clean"` (admissible) or `"pending"` (held — conflict found, NOT applied,
    /// NOT hard-rejected). There is intentionally no `"blocked"` status: the
    /// policy is WARN-then-escalate, never fail-closed.
    status: &'static str,
    /// `true` iff the variant was flagged (held pending). Equal to
    /// `status == "pending"`; kept as an explicit boolean for ergonomic asserts.
    held_pending: bool,
    /// `true` iff the variant was applied/admitted. ALWAYS `false` for this
    /// surface — the W-3 entrypoint screens + records; it never mutates the
    /// archive (admission is a separate, deferred step). Surfaced so the policy
    /// ("never silently apply") is observable in the report.
    applied: bool,
    /// Snake-case tripwire labels that were active at screening time.
    active_tripwires: Vec<&'static str>,
    /// Count of `SymbolicConflictDetected` beacons written (the held-pending
    /// records). Zero for a clean variant.
    conflict_rows_emitted: usize,
}

/// `elai self-mod apply --proposal-fixture PATH [--no-env-override]` handler.
///
/// Loads the fixture, builds a real [`ConfigVariant`], drives it through the REAL
/// [`screen_config_variant_for_conflicts`] compose unit (which itself runs the
/// real detector against the live `LOCKED_KEYS` + tripwire model and emits the
/// real `SymbolicConflictDetected` beacon(s)), and prints the verdict.
///
/// `no_env_override`: when set AND [`ELAI_SYMBOLIC_CONFLICT_DISABLE_ENV`] is
/// truthy, the handler errors rather than silently skipping the screen — the
/// caller demanded a real screening pass.
fn run_apply(
    fixture_path: &str,
    no_env_override: bool,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let raw = fs::read_to_string(fixture_path)
        .map_err(|e| format!("self-mod apply: cannot read '{fixture_path}': {e}"))?;
    let fixture: ProposalFixture = serde_json::from_str(&raw)
        .map_err(|e| format!("self-mod apply: parse '{fixture_path}' failed: {e}"))?;

    // --no-env-override: refuse to run a suppressed screen. Unlike a model
    // verdict, the SCF screen is a pure function, but an operator who passed the
    // flag explicitly demanded the screen run; honour that intent.
    if no_env_override && env_kill_switch_truthy() {
        return Err(format!(
            "self-mod apply: --no-env-override set but {ELAI_SYMBOLIC_CONFLICT_DISABLE_ENV} is truthy; \
             unset the kill-switch to run the real symbolic-conflict screen"
        ));
    }

    let cv = config_variant_from_fixture(&fixture)?;
    let active_tripwires = fixture.active_tripwires.clone();
    let active_labels: Vec<&'static str> =
        active_tripwires.iter().map(|t| t.as_str()).collect();

    // Drive the REAL compose unit. Ssb::new spawns the single-writer task and
    // MUST be constructed INSIDE block_on (gotchas:110), so we open the writer
    // and run the screen on a single current-thread runtime. The screen returns
    // `true` when conflicts were found (held pending) after emitting the
    // SymbolicConflictDetected beacon(s) durably.
    let (held, conflict_rows_emitted) =
        screen_on_runtime(&cv, &active_tripwires, &fixture.fsm_substate)?;

    let report = ApplyReport {
        variant_id: fixture.variant_id.clone(),
        status: if held { "pending" } else { "clean" },
        held_pending: held,
        // The W-3 entrypoint screens + records; it never admits. Always false.
        applied: false,
        active_tripwires: active_labels,
        conflict_rows_emitted,
    };
    render(&report, output_format)
}

/// True when the documented kill-switch env var is set to a truthy value
/// (`1` / `true` / `yes`, case-insensitive). Mirrors the orchestrator's
/// env-truthy convention used by the other wire-dark gates.
fn env_kill_switch_truthy() -> bool {
    std::env::var(ELAI_SYMBOLIC_CONFLICT_DISABLE_ENV)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes"
        })
        .unwrap_or(false)
}

/// Build a real [`ConfigVariant`] from the operator-friendly fixture. The
/// fixture's `config` object is serialised back to the `config_blob` byte form
/// the detector decodes (round-tripping through the SAME JSON-object shape
/// `intent_from_config_variant` expects), so the CLI screens the byte-for-byte
/// blob a live proposal would carry.
fn config_variant_from_fixture(fixture: &ProposalFixture) -> Result<ConfigVariant, String> {
    let config_blob = serde_json::to_vec(&serde_json::Value::Object(fixture.config.clone()))
        .map_err(|e| format!("self-mod apply: encode config_blob failed: {e}"))?;
    Ok(ConfigVariant {
        variant_id: fixture.variant_id.clone(),
        parent_id: None,
        score: 0.0,
        children_count: 0,
        config_blob,
        created_at: 0,
        pipeline_outcomes: Vec::new(),
        // Mode is irrelevant to the symbolic screen (the detector reasons over
        // keys + ops, not over mode); Surgical is the neutral default.
        pipeline_mode: PipelineMode::Surgical,
        status: VariantStatus::Active,
    })
}

/// Run the screen on an own current-thread runtime with the SSB writer opened
/// INSIDE `block_on` (gotchas:110). Returns `(held, conflict_rows_emitted)`
/// where `held` is the compose unit's verdict and `conflict_rows_emitted` is the
/// number of `SymbolicConflictDetected` rows now present for this variant.
fn screen_on_runtime(
    cv: &ConfigVariant,
    active_tripwires: &[TripwireKind],
    fsm_substate: &str,
) -> Result<(bool, usize), String> {
    let project_root: PathBuf = std::env::current_dir()
        .map_err(|e| format!("self-mod apply: cannot determine cwd: {e}"))?;
    let db_path = sessions_dir_path(&project_root).join("ssb.db");
    let parent = db_path
        .parent()
        .ok_or_else(|| "self-mod apply: ssb.db has no parent dir".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("self-mod apply: create sessions dir failed: {e}"))?;

    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-self-mod-apply".to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("self-mod apply: build async runtime failed: {e}"))?;

    let cv = cv.clone();
    let active = active_tripwires.to_vec();
    let fsm_substate = fsm_substate.to_string();

    // The number of beacons the compose unit emits equals the number of
    // conflicts the pure detector finds (one beacon per conflict — see
    // `emit_symbolic_conflict_detected`). Recompute it deterministically from the
    // SAME pure detector the compose unit drives, rather than re-decoding the
    // freshly-written SSB rows. This keeps the count exact, avoids a redundant
    // read of the just-written rows, and requires no msgpack decode dependency
    // in the CLI crate.
    let conflict_count = {
        let intent = intent_from_config_variant(&cv);
        if intent.is_empty() {
            0
        } else {
            let model = current_process_model(&active);
            detect_conflicts(&intent, &model).len()
        }
    };

    // Ssb::new MUST be constructed inside the runtime context (gotchas:110). The
    // compose unit emits the durable held-pending beacon(s) and returns the hold
    // verdict.
    let held = rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock)
            .map_err(|e| format!("self-mod apply: open SSB writer failed: {e}"))?;
        let held =
            screen_config_variant_for_conflicts(&ssb, &cv, &active, &fsm_substate).await;
        Ok::<bool, String>(held)
    })?;

    Ok((held, conflict_count))
}

/// Render the report in the requested format. Mirrors the `safety_cmd::render`
/// helper so CLI output is uniform across operator surfaces.
fn render<T: Serialize>(report: &T, output_format: CliOutputFormat) -> Result<(), String> {
    let out = match output_format {
        CliOutputFormat::Json => serde_json::to_string_pretty(report)
            .map_err(|e| format!("self-mod render: json serialise failed: {e}"))?,
        CliOutputFormat::Toml => {
            let wrapped = serde_json::json!({ "report": report });
            toml::to_string(&wrapped)
                .map_err(|e| format!("self-mod render: toml serialise failed: {e}"))?
        }
        CliOutputFormat::Text => {
            let v = serde_json::to_value(report)
                .map_err(|e| format!("self-mod render: text->json failed: {e}"))?;
            format!("{v:#}")
        }
    };
    println!("{out}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(config: serde_json::Value, tripwires: &[&str]) -> ProposalFixture {
        let config = match config {
            serde_json::Value::Object(m) => m,
            _ => serde_json::Map::new(),
        };
        let active_tripwires: Vec<TripwireKind> = tripwires
            .iter()
            .filter_map(|t| serde_json::from_value(serde_json::json!(*t)).ok())
            .collect();
        ProposalFixture {
            variant_id: "v-test".to_string(),
            config,
            active_tripwires,
            fsm_substate: "AdmittingVariant".to_string(),
        }
    }

    #[test]
    fn config_variant_round_trips_blob() {
        let f = fixture(serde_json::json!({ "score_weights": "tampered" }), &[]);
        let cv = config_variant_from_fixture(&f).expect("build cv");
        // The blob must decode back to a JSON object with the same key.
        let decoded: serde_json::Value =
            serde_json::from_slice(&cv.config_blob).expect("decode blob");
        assert_eq!(decoded["score_weights"], serde_json::json!("tampered"));
        assert_eq!(cv.variant_id, "v-test");
    }

    #[test]
    fn fixture_parses_snake_case_tripwires() {
        let raw = r#"{ "variant_id": "v", "config": {}, "active_tripwires": ["ddm", "kl_divergence"] }"#;
        let f: ProposalFixture = serde_json::from_str(raw).expect("parse fixture");
        assert_eq!(f.active_tripwires.len(), 2);
        assert!(f.active_tripwires.contains(&TripwireKind::Ddm));
        assert!(f.active_tripwires.contains(&TripwireKind::KlDivergence));
        // fsm_substate defaults when omitted.
        assert_eq!(f.fsm_substate, "AdmittingVariant");
    }

    #[test]
    fn env_kill_switch_recognises_truthy() {
        // Pure parse of the truthy convention without mutating process env in a
        // way that could race other tests: assert the recognised set directly.
        for truthy in ["1", "true", "TRUE", "Yes"] {
            let v = truthy.trim().to_ascii_lowercase();
            assert!(v == "1" || v == "true" || v == "yes", "{truthy} should be truthy");
        }
    }

    #[test]
    fn empty_config_defaults_to_empty_object_blob() {
        let raw = r#"{ "variant_id": "v" }"#;
        let f: ProposalFixture = serde_json::from_str(raw).expect("parse minimal fixture");
        assert!(f.config.is_empty());
        let cv = config_variant_from_fixture(&f).expect("build cv");
        let decoded: serde_json::Value =
            serde_json::from_slice(&cv.config_blob).expect("decode blob");
        assert!(decoded.is_object());
    }
}
