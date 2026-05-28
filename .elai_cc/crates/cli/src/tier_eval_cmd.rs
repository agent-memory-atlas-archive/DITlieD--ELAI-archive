//! E-10 -- `elai tier-eval --suites ... --budget N` dispatcher.
//! Per-VRAM tier matrix decision-sheet eval; live runs RUNPOD-PENDING.

pub fn run(suites: Vec<String>, budget: u32) -> Result<(), anyhow::Error> {
    let suite_list = suites.join(",");
    println!(
        "tier-eval suites=[{suite_list}] budget={budget} (RUNPOD-PENDING)"
    );
    Ok(())
}
