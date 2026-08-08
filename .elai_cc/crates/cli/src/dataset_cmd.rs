//! I-30 Slice 1b — `elai dataset export-sft` handler.
//!
//! Thin dispatcher mirroring `rra_cmd.rs`. Drives [`SftExhaust`] over a
//! directory of sealed `.rra` files and prints the report.

use crate::args::{CliOutputFormat, DatasetAction};
use crate::sft_exhaust::{SftExhaust, SftExhaustReport};

/// `elai dataset <sub>` dispatcher invoked by main.rs.
pub(crate) fn run_dataset(
    action: DatasetAction,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    match action {
        DatasetAction::ExportSft { rra_dir, out_dir } => {
            let report = SftExhaust::new(rra_dir, out_dir).run()?;
            render_report(&report, output_format);
            Ok(())
        }
    }
}

fn render_report(report: &SftExhaustReport, output_format: CliOutputFormat) {
    match output_format {
        CliOutputFormat::Json => {
            let out = serde_json::json!({
                "sft_exhaust": {
                    "path_extracted": report.path_extracted,
                    "negative_paired": report.negative_paired,
                    "dataset_sealed": report.dataset_sealed,
                    "skipped_unsealed": report.skipped_unsealed,
                }
            });
            let json = serde_json::to_string_pretty(&out)
                .unwrap_or_else(|e| format!("{{\"error\": \"serializing report: {e}\"}}"));
            println!("{json}");
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("elai dataset export-sft (I-30 Slice 1b)");
            println!("  path_extracted   : {}", report.path_extracted);
            println!("  negative_paired  : {}", report.negative_paired);
            println!("  dataset_sealed   : {}", report.dataset_sealed);
            println!("  skipped_unsealed : {}", report.skipped_unsealed);
        }
    }
}
