//! E-10 -- `elai bench-ablation <tile>` dispatcher.
//! See workflow/reasoning/E-10-decisions.md.
//! Calls into the orchestrator critic module set; live wiring is
//! RUNPOD-PENDING -- this dispatcher prints the chosen tile + budget
//! and exits 0 so the wire-format runs end-to-end in CI.

pub fn run(tile: String, fixtures: String, budget: u32) -> Result<(), anyhow::Error> {
    // RUNPOD-PENDING: live invocation of orchestrator::fsm_ablation /
    // role_ablation / rag_challenger / cot_proof / cache_check /
    // grc_blockquote requires an active Ssb + remote sidecar route.
    // The dispatcher echoes the parsed inputs to stdout so the CLI
    // surface is exercised in tests without a live multi-agent run.
    println!(
        "bench-ablation tile={tile} fixtures={fixtures} budget={budget} (RUNPOD-PENDING)"
    );
    Ok(())
}
