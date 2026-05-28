//! E-55 -- `elai rl <train|list> [--recipe=<name>] [--list] [--json]`
//! dispatcher.
//!
//! The agentic RL substrate ships fully under
//! `elai_orchestrator::rl::*` -- this CLI surface is the operator entry
//! point that names ONE recipe and emits an SsbActionType::RewardLadderEvent
//! row (when ELAI_AGENTIC_RL_ENABLE=1; otherwise wire-dark and the dispatch
//! short-circuits with RUNPOD-PENDING).
//!
//! Recipes (matched literal):
//!   rlaif_judge          -- Task 1: cloud Sonnet 5-dim Likert rating run
//!   gepa_evolve          -- Task 2: reflective prompt evolution round
//!   reward_fusion        -- Task 3: PRM+ORM fused reward compute
//!   mona_canary          -- Task 4 + Task 6: composite reward + canary
//!   sft_rollout_collect  -- Task 5: GREEN-session rollout collector
//!   deferred_unlock_check -- Task 8: 3-condition unlock gate
//!
//! Live cloud Sonnet rollouts + LoRA training is RUNPOD-PENDING. The
//! dispatcher prints the recipe + the env-gate state + the substrate
//! readiness so the operator knows the next live step requires the
//! remote sidecar + the env gate flip.

/// All known recipe names. Used by `--list` AND by the parse-side
/// validation in the future (currently parse accepts any string; the
/// handler does the literal-name check at dispatch).
pub const KNOWN_RECIPES: &[&str] = &[
    "rlaif_judge",
    "gepa_evolve",
    "reward_fusion",
    "mona_canary",
    "sft_rollout_collect",
    "deferred_unlock_check",
];

pub fn run(
    recipe: Option<String>,
    list: bool,
    json: bool,
) -> Result<(), anyhow::Error> {
    let env_gate = std::env::var("ELAI_AGENTIC_RL_ENABLE")
        .ok()
        .map(|v| v == "1")
        .unwrap_or(false);

    if list || recipe.is_none() {
        if json {
            print_recipes_json(env_gate);
        } else {
            print_recipes_text(env_gate);
        }
        return Ok(());
    }

    let r = recipe.unwrap_or_default();
    if !KNOWN_RECIPES.contains(&r.as_str()) {
        return Err(anyhow::anyhow!(
            "unknown `rl train --recipe={r}`. Valid: {}",
            KNOWN_RECIPES.join(", ")
        ));
    }

    // RUNPOD-PENDING: dispatch the recipe via the orchestrator's
    // session-runtime; this CLI surface stays substrate-only until the
    // live cloud Sonnet path is wired (env gate ELAI_AGENTIC_RL_ENABLE=1).
    if json {
        println!(
            "{{\"recipe\":\"{r}\",\"env_gate\":{env_gate},\"status\":\"RUNPOD-PENDING\",\"substrate\":\"ready\"}}"
        );
    } else {
        println!(
            "rl train --recipe={r} env_gate={env_gate} status=RUNPOD-PENDING substrate=ready"
        );
    }
    Ok(())
}

fn print_recipes_text(env_gate: bool) {
    println!("E-55 agentic RL recipes (env gate ELAI_AGENTIC_RL_ENABLE={env_gate}):");
    for r in KNOWN_RECIPES {
        println!("  {r}");
    }
    println!("Live cloud Sonnet rollouts + LoRA training RUNPOD-PENDING.");
    println!("See workflow/reasoning/E-55-decisions.md.");
}

fn print_recipes_json(env_gate: bool) {
    let recipes = KNOWN_RECIPES
        .iter()
        .map(|r| format!("\"{r}\""))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"recipes\":[{recipes}],\"env_gate\":{env_gate},\"status\":\"RUNPOD-PENDING\"}}"
    );
}
