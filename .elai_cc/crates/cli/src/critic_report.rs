//! E-10 -- `elai critic-report [--output PATH]` dispatcher.
//! Aggregates the 6 ablation tiles into a decision report markdown.
//! Live aggregation queries SSB CriticDecisionEvent rows; RUNPOD-PENDING.

use std::path::PathBuf;

pub fn run(output: Option<String>) -> Result<(), anyhow::Error> {
    let out = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("workflow/reasoning/critic-decisions-pending.md"));
    println!("critic-report output={} (RUNPOD-PENDING)", out.display());
    Ok(())
}
