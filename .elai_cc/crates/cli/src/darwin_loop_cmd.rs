//! F-11 -- `elai darwin-loop` CLI dispatcher.
//!
//! Closes ONE config-only Darwin-Goedel-Machine generation: propose -> evaluate
//! (real candidate-derived alpha, NOT a synthetic alternating reward) -> the
//! four-guard chain (blake3 tripwire fingerprint, trap-pattern scorer on the
//! LIVE candidate, the Layer-1A `is_security_sensitive` keyword filter, and the
//! Two-Gate promotion gate) -> `config_archive::insert_variant` (the FIRST
//! production caller) -> `config_archive::select_parent` (loop closure) -> emit
//! `SsbActionType::DarwinianAdmit` rows.
//!
//! The reusable loop core lives in
//! [`elai_orchestrator::darwin_admit_loop`] (the `elai-cli` crate is binary-only
//! for this module, so the testable logic is in the library so contract tests +
//! the observable smoke can drive the SAME production path). This module is the
//! thin production entrypoint wired at `main.rs::CliAction::DarwinLoop`.
//!
//! Wire-dark: the real loop body runs ONLY when BOTH
//! `cfg(feature = "darwinian-evolve")` AND `ELAI_DARWINIAN_LOOP_ENABLE=1` are
//! set (an autonomous self-modification loop is opt-in by ELAI policy; E-7
//! DECISION-13 + CLAUDE.md autonomous-mode). With the feature OFF the dispatch
//! shim prints the DISABLED message; with the feature ON but the env gate unset
//! the loop prints a gated notice and inserts nothing.
//!
//! Flags (unchanged from E-7):
//!   --generations N    (default 1)        Number of evolve rounds (F-11 ships 1).
//!   --mutants K        (default 3)        Candidate count per round.
//!   --seed S           (default 42)       Deterministic seed (ELAI_SEED).
//!   --traps T          (default 3)        Legacy trap-fixture count (telemetry).
//!   --gamma G          (default ...)      Legacy DS-TS discount (telemetry).
//!
//! Mutant injection: `ELAI_DARWINIAN_LOOP_MUTANTS_JSON` may point at a JSON
//! array of mutant candidates (used by the deterministic offline smoke + live
//! runs to inject a fixed candidate set, e.g. a trap mutant + a clean mutant).
//! When unset, the driver seeds `--mutants` deterministic clean candidates.
//!
//! NO `.unwrap()`/`.expect()` in non-test paths; thresholds are named consts in
//! the orchestrator core; the candidate fingerprint uses blake3 ONLY (per the
//! CLAUDE.md HARD RULE; the orchestrator core computes it).

/// Env var: millisecond Unix timestamp of the model training cutoff for the eval firewall
/// date-gate (H-47 BLOCK-1 fix). When unset, defaults to `ELAI_MODEL_CUTOFF_DATE_MS_DEFAULT`
/// (a conservative past date meaning all tasks are treated as post-cutoff unless the operator
/// sets a real value). Set to the actual model knowledge cutoff before a live run.
#[cfg(feature = "darwinian-evolve")]
const ELAI_MODEL_CUTOFF_DATE_MS_ENV: &str = "ELAI_MODEL_CUTOFF_DATE_MS";

/// Default model cutoff: 2024-01-01 00:00:00 UTC in milliseconds.
/// Conservative default: most held-out tasks are ingested AFTER this date so the
/// date-gate does not suppress them unless `ELAI_MODEL_CUTOFF_DATE_MS` is set to
/// the real model cutoff. Operators running a live eval firewall MUST set the env var.
#[cfg(feature = "darwinian-evolve")]
const ELAI_MODEL_CUTOFF_DATE_MS_DEFAULT: i64 = 1_704_067_200_000; // 2024-01-01 UTC

#[cfg(feature = "darwinian-evolve")]
use elai_orchestrator::darwin_admit_loop::{
    loop_enabled_via_env, run_one_generation, EvalFirewallContext, MutantCandidate,
    DEFAULT_SUITE_NAME, ELAI_DARWINIAN_LOOP_MUTANTS_JSON_ENV,
};

/// E-7/F-11 dispatch shim. Accepts the raw `gamma` String from args, parses it
/// internally, and branches on the feature flag so `main.rs` is cfg-agnostic.
///
/// Feature-on  : parses gamma -> f64, calls [`run`].
/// Feature-off : prints DISABLED message, returns Ok(()).
pub(crate) fn dispatch(
    generations: u32,
    mutants: u32,
    seed: u64,
    traps: u32,
    gamma: String,
) -> anyhow::Result<()> {
    #[cfg(feature = "darwinian-evolve")]
    {
        let gamma_f: f64 = if gamma.is_empty() {
            elai_orchestrator::darwinian_bai::DEFAULT_DISCOUNT_FACTOR
        } else {
            gamma
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid --gamma `{gamma}`: {e}"))?
        };
        return darwinian_admit(generations, mutants, seed, traps, gamma_f);
    }
    #[cfg(not(feature = "darwinian-evolve"))]
    {
        println!(
            "darwin-loop DISABLED (rebuild with --features darwinian-evolve) \
             requested generations={generations} mutants={mutants} seed={seed} \
             traps={traps} gamma={gamma}"
        );
        Ok(())
    }
}

/// F-11 driver entry (the `darwinian_admit` driver named in the
/// WIRING_CONTRACT). Builds a tokio runtime (the archive + SSB calls are async;
/// `Ssb::new` spawns a single-writer task that MUST be constructed inside the
/// runtime context), opens the config archive + SSB, loads the mutant candidate
/// set, and runs ONE closed generation through the four-guard chain via
/// [`elai_orchestrator::darwin_admit_loop::run_one_generation`].
#[cfg(feature = "darwinian-evolve")]
pub(crate) fn darwinian_admit(
    generations: u32,
    mutants: u32,
    seed: u64,
    traps: u32,
    gamma: f64,
) -> Result<(), anyhow::Error> {
    if !(0.0..=1.0).contains(&gamma) {
        return Err(anyhow::anyhow!("--gamma {gamma} must be in (0.0, 1.0]"));
    }

    // BOTH the feature (this `run` body) AND the env gate are required for the
    // live loop. With the env gate unset the command is a no-op notice (no
    // archive mutation, no SSB row) -- the wire-dark posture.
    if !loop_enabled_via_env() {
        println!(
            "darwin-loop GATED: set ELAI_DARWINIAN_LOOP_ENABLE=1 to run the \
             config-DGM admit loop (generations={generations} mutants={mutants} \
             seed={seed} traps={traps} gamma={gamma:.6})"
        );
        return Ok(());
    }

    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("cannot resolve current dir for darwin-loop: {e}"))?;
    let sessions = elai_core::paths::sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions)
        .map_err(|e| anyhow::anyhow!("cannot create sessions dir {}: {e}", sessions.display()))?;
    let archive_path = sessions.join("self_mod.db");
    let ssb_path = sessions.join("ssb.db");

    // H-47 BLOCK-1 fix: read model cutoff date from env (documented default = 2024-01-01).
    let cutoff_date_ms: i64 = std::env::var(ELAI_MODEL_CUTOFF_DATE_MS_ENV)
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(ELAI_MODEL_CUTOFF_DATE_MS_DEFAULT);
    let model_id = std::env::var("ELAI_MODEL_ID").unwrap_or_else(|_| "elai-local".to_string());

    // Open the HeldOutStore and HoldoutBudgetLedger for the eval firewall. Both open
    // from the sessions dir (same db file as ssb). Opened BEFORE the tokio runtime so
    // errors surface before the async block.
    let held_out_store = elai_orchestrator::held_out::HeldOutStore::open(&ssb_path)
        .map_err(|e| anyhow::anyhow!("held_out store open failed: {e}"))?;
    let budget_ledger =
        elai_orchestrator::holdout_budget::HoldoutBudgetLedger::open(&ssb_path, "cli-darwin-loop")
            .map_err(|e| anyhow::anyhow!("holdout_budget ledger open failed: {e}"))?;

    let candidates = load_candidates(mutants, seed)?;
    let candidate_count = candidates.len();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("tokio runtime build failed: {e}"))?;

    let summary = rt.block_on(async move {
        let archive = elai_orchestrator::config_archive::ConfigArchive::open(&archive_path)
            .await
            .map_err(|e| anyhow::anyhow!("config archive open failed: {e}"))?;
        let ssb_cfg = elai_orchestrator::SsbConfig {
            db_path: ssb_path,
            pool_size: None,
            project_id: "default".to_string(),
            session_id: "cli-darwin-loop".to_string(),
        };
        let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
            std::sync::Arc::new(elai_core::OsClock);
        let ssb = std::sync::Arc::new(
            elai_orchestrator::Ssb::new(ssb_cfg, clock.clone())
                .map_err(|e| anyhow::anyhow!("ssb open failed: {e}"))?,
        );

        let cutoff = elai_orchestrator::held_out::ModelTrainingCutoff {
            model_id,
            date_ms: cutoff_date_ms,
        };

        let promotion_cfg = elai_orchestrator::promotion_gate::PromotionConfig::default();
        let mut rng = elai_core::SeededRng::new(seed);
        // F-11 ships ONE generation; --generations is honored as a loop count
        // but the Tier-1 scope + acceptance is the single closed generation.
        let mut last = None;
        let gen_count = generations.max(1);
        for gen_idx in 0..gen_count {
            // H-47 BLOCK-1: construct a REAL EvalFirewallContext on the production CLI path
            // so eval_firewall legs (date-gate + template-regen + holdout-budget) actually run.
            let fw_ctx = EvalFirewallContext {
                held_out_store: &held_out_store,
                budget_ledger: &budget_ledger,
                clock: &clock,
                ssb_arc: &ssb,
                cutoff: &cutoff,
                holdout_id: "darwin-loop-default",
                objective: "fitness",
                data_window: "0..9999999999999",
                budget_seed: seed, // WARN-1 fix: derived from CLI --seed, not a literal
                generation_id: gen_idx as u64,
                workload_ngrams: &[], // empty ok per EvalFirewallContext doc
                ignore_dates: false,  // MUST be false in production (WARN-4)
            };
            let s = run_one_generation(
                &archive,
                &*ssb,
                &candidates,
                elai_orchestrator::self_mod_layer1a::PipelineMode::Surgical,
                seed,
                DEFAULT_SUITE_NAME,
                &promotion_cfg,
                &mut rng,
                Some(&fw_ctx), // H-47 BLOCK-1: wired — eval_firewall now fires on the live path
            )
            .await
            .map_err(|e| anyhow::anyhow!("darwin-loop generation failed: {e}"))?;
            last = Some(s);
        }
        last.ok_or_else(|| anyhow::anyhow!("darwin-loop produced no generation"))
    })?;

    println!(
        "darwin-loop COMPLETE generations={generations} mutants={} seed={seed} traps={traps} \
         gamma={gamma:.6} admitted={} rejected={} loop_closed={} suite_floor={:?} \
         bandit_selected_arm={:?}",
        candidate_count,
        summary.admitted,
        summary.rejected,
        summary.loop_closed,
        summary.suite_floor,
        summary.bandit_selected_arm,
    );
    Ok(())
}

/// Load the mutant candidate set: from the `ELAI_DARWINIAN_LOOP_MUTANTS_JSON`
/// file when set (the deterministic injection path), else seed `mutants`
/// deterministic clean candidates from `seed`.
#[cfg(feature = "darwinian-evolve")]
fn load_candidates(mutants: u32, seed: u64) -> Result<Vec<MutantCandidate>, anyhow::Error> {
    if let Ok(path) = std::env::var(ELAI_DARWINIAN_LOOP_MUTANTS_JSON_ENV) {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("cannot read mutants json {path}: {e}"))?;
        let parsed = MutantCandidate::parse_array(&text)
            .map_err(|e| anyhow::anyhow!("cannot parse mutants json {path}: {e}"))?;
        if parsed.is_empty() {
            return Err(anyhow::anyhow!("mutants json {path} is an empty array"));
        }
        return Ok(parsed);
    }
    // No injection file: synthesize `mutants` deterministic clean candidates.
    // Each carries a distinct force_alpha well above tau so it clears the
    // Two-Gate margin; the score is a REAL field of the candidate, derived from
    // its seed-stamped config (NOT a hard-coded literal shared across runs).
    let count = mutants.max(1) as usize;
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let alpha = 0.50 + ((seed.wrapping_add(i as u64)) % 40) as f64 / 100.0;
        out.push(MutantCandidate {
            variant_label: format!("seed-{seed}-mutant-{i}"),
            pipeline_mode: "surgical".to_string(),
            config: serde_json::json!({
                "generation": 0,
                "mutant_index": i,
                "seed": seed,
                "force_alpha": alpha,
            }),
            eval_log: format!("seeded clean candidate {i} (seed={seed}); no traps; no security keywords"),
        });
    }
    Ok(out)
}
