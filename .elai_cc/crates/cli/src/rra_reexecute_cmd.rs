//! J-91 — `elai rra re-execute` handler (split from rra_cmd.rs for MR-1).

use elai_orchestrator::rra::{pin_environment, re_execute_rra, RunArtifact};
use elai_orchestrator::rra_export::FixtureCassette;

use crate::args::CliOutputFormat;

/// J-91 — `elai rra re-execute --cassette <path> [--tier T0|T1|T2]` handler.
///
/// Reads a `.fixture` cassette, decodes the bundled `RunArtifact`, derives a
/// `PinnedEnv` from it (no external env pins threaded yet), drives
/// [`re_execute_rra`], prints the verdict, and exits non-zero when
/// `!verdict.ok`. This is the production caller for the J-91 cassette-only
/// re-execution path.
pub fn run_re_execute(
    cassette: &std::path::Path,
    tier: &str,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let cassette_bytes = std::fs::read(cassette)
        .map_err(|e| format!("reading cassette at {}: {e}", cassette.display()))?;
    let cassette_decoded: FixtureCassette = rmp_serde::from_slice(&cassette_bytes)
        .map_err(|e| format!("decoding cassette at {}: {e}", cassette.display()))?;
    let art: RunArtifact = rmp_serde::from_slice(&cassette_decoded.bundle_bytes)
        .map_err(|e| format!("decoding RunArtifact from cassette: {e}"))?;

    let pinned = pin_environment(&art, None, None, &[]);

    let verdict = re_execute_rra(cassette, &pinned, tier, None)
        .map_err(|e| format!("re-executing cassette at {}: {e}", cassette.display()))?;

    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&verdict)
                .map_err(|e| format!("serializing re-execute verdict as json: {e}"))?;
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("elai rra re-execute (J-91, cassette-only re-execution)");
            println!("  cassette         : {}", cassette.display());
            println!("  tier_requested   : {}", verdict.tier_requested);
            println!("  tier_effective   : {}", verdict.tier_effective);
            println!("  ok               : {}", verdict.ok);
            println!("  cassette_only    : {}", verdict.cassette_only);
            println!("  cassette_blake3  : {}", verdict.cassette_blake3);
            if let Some(matched) = verdict.objective_seal_match {
                println!("  objective_seal   : {}", matched);
            }
            if !verdict.downgrades.is_empty() {
                println!("  downgrades       : {}", verdict.downgrades.join(", "));
            }
            if !verdict.notes.is_empty() {
                println!("  notes            : {}", verdict.notes.join(", "));
            }
        }
    }

    if verdict.ok {
        Ok(())
    } else {
        std::process::exit(1);
    }
}
