//! E-10 / E-39 -- `elai bench-ablation <tile>` dispatcher.
//!
//! E-10 baseline: scaffolded as a printf-stub so the wire-format runs
//! end-to-end in CI without a live multi-agent run.
//!
//! E-39 augmentation (this revision): when `tile == "automix_verifier"`
//! we drive `budget` iterations of [`elai_orchestrator::cost::automix_gate::parse_verifier_response`]
//! against a synthetic deterministic JSON pattern set (continue / retry /
//! escalate / malformed) so the ablation surface is observable in CI
//! without a live sidecar call. The 30-step default in the E-39 plan
//! lands as `--budget 30`. No SsbActionType variants are emitted (those
//! are written by the orchestrator-side gate when the live verifier
//! fires; this CLI exercises the parser surface only).
//!
//! See workflow/reasoning/E-10-decisions.md + E-39-decisions.md.

use elai_orchestrator::cost::automix_gate::{parse_verifier_response, AutomixVerdict};

/// Synthetic verdict patterns used to drive `parse_verifier_response`
/// through every legal branch + the malformed-body branch. Order is
/// deterministic so a 30-iteration loop covers each ~7-8 times.
const PATTERNS: [&str; 4] = [
    r#"{"verdict":"continue","reason":"structurally valid"}"#,
    r#"{"verdict":"retry","reason":"borderline output, retry"}"#,
    r#"{"verdict":"escalate","reason":"requires stronger model"}"#,
    r#"{"verdict":"banana","reason":"malformed verdict label"}"#,
];

pub fn run(tile: String, fixtures: String, budget: u32) -> Result<(), anyhow::Error> {
    if tile == "automix_verifier" {
        return run_automix_verifier_ablation(fixtures, budget);
    }
    // E-10 default tiles (fsm_ablation / role_ablation / rag_challenger /
    // cot_proof / cache_check / grc_blockquote) require an active Ssb +
    // remote sidecar route; live wire is RUNPOD-PENDING.
    println!(
        "bench-ablation tile={tile} fixtures={fixtures} budget={budget} (RUNPOD-PENDING)"
    );
    Ok(())
}

/// E-39 -- drive the AutoMix verifier parser through `budget` synthetic
/// rounds. Counts each verdict + malformed event and reports totals.
fn run_automix_verifier_ablation(fixtures: String, budget: u32) -> Result<(), anyhow::Error> {
    let budget = budget.max(1);
    let mut n_continue: u32 = 0;
    let mut n_retry: u32 = 0;
    let mut n_escalate: u32 = 0;
    let mut n_malformed: u32 = 0;
    for step in 0..budget {
        let body = PATTERNS[(step as usize) % PATTERNS.len()];
        match parse_verifier_response(body) {
            Ok((AutomixVerdict::Continue, _)) => n_continue += 1,
            Ok((AutomixVerdict::Retry, _)) => n_retry += 1,
            Ok((AutomixVerdict::Escalate, _)) => n_escalate += 1,
            Err(_) => n_malformed += 1,
        }
    }
    println!(
        "bench-ablation tile=automix_verifier fixtures={fixtures} budget={budget} continue={n_continue} retry={n_retry} escalate={n_escalate} malformed={n_malformed}"
    );
    // Sanity: every step must be accounted for.
    let total = n_continue + n_retry + n_escalate + n_malformed;
    if total != budget {
        return Err(anyhow::anyhow!(
            "automix_verifier ablation drop: total={total} budget={budget}"
        ));
    }
    Ok(())
}
