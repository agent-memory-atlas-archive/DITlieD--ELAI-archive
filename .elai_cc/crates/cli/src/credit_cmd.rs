//! C-45 — `elai credit <attribute|table|export>` handler.
//!
//! Drives the orchestrator's
//! [`elai_orchestrator::credit_attribution`] module. Three sub-actions:
//!   - `attribute [--role NAME]`: run the contrastive credit-attribution
//!     harness over the eight base `AgentRole` variants (or a single
//!     specified role) and print the resulting credit table.
//!   - `table`: print the most recent in-memory credit table from a
//!     freshly-run synthetic-fixture harness (no GPU required) so the
//!     CLI surface always has a verifiable observable effect even
//!     before the live runpod pilot lands.
//!   - `export --path PATH`: write the in-memory credit table to a
//!     JSON file for downstream tooling (the cantante-pilot.md artifact
//!     consumes this as its input).
//!
//! Paper grounding (arXiv:2605.13295, Zehle 2026, CANTANTE Section 3.2):
//! the contrastive method itself is paper-grounded; the synthetic
//! scoring fixture used by this CLI is a deterministic stand-in for the
//! GPU-billable real pilot. The synthetic fixture's per-role
//! "drop-cost" curve is illustrative ONLY — real credits require the
//! `RUNPOD-PENDING: cantante_pilot_450runs_qwen3_8b` invocation.
//!
//! Exit code conventions:
//!   - `attribute`: 0 on successful run, 1 on harness failure.
//!   - `table`: 0 on successful render.
//!   - `export`: 0 on successful write, 1 on IO failure.

use std::fs;
use std::path::PathBuf;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::credit_attribution::{
    pilot_roles, CreditAttributionHarness, CreditReport, DropPolicy,
};
use elai_orchestrator::ipc_types::AgentRole;

use crate::args::{CliOutputFormat, CreditAction};

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(action: CreditAction, output_format: CliOutputFormat) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("credit_cmd: tokio runtime build failed: {e}"))?;
    runtime.block_on(async {
        match action {
            CreditAction::Attribute { dropped_role } => {
                run_attribute(dropped_role.as_deref(), output_format).await
            }
            CreditAction::Table => run_table(output_format).await,
            CreditAction::Export { path } => run_export(&PathBuf::from(path), output_format).await,
        }
    })
}

/// Resolve a snake_case role name string into the canonical
/// [`AgentRole`] enum (eight-role pilot set only). Returns Err for
/// unknown / out-of-scope names.
fn resolve_role(name: &str) -> Result<AgentRole, String> {
    let snake = name.trim().to_ascii_lowercase();
    let match_role = pilot_roles().iter().copied().find(|role| {
        let role_str = serde_json::to_value(role)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        role_str == snake
    });
    match_role.ok_or_else(|| {
        let valid: Vec<String> = pilot_roles()
            .iter()
            .map(|r| {
                serde_json::to_value(r)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default()
            })
            .collect();
        format!(
            "unknown role `{name}` (C-45 pilot ablates only the 8 base roles: {})",
            valid.join(", ")
        )
    })
}

/// Synthetic scoring fixture for the CLI surface. Deterministic
/// drop-cost curve so the CLI always renders a populated credit
/// table. NOT a real measurement — flagged as such in every output.
async fn synthetic_score(policy: DropPolicy, _i: u32) -> Result<f32, String> {
    match policy.dropped_role {
        None => Ok(0.80),
        // Hand-tuned synthetic curve. Order matches `pilot_roles()`
        // (declaration order in `AgentRole`). Numbers are illustrative
        // ONLY; real credits require the runpod-pending pilot.
        Some(AgentRole::Executor) => Ok(0.10),
        Some(AgentRole::Auditor) => Ok(0.40),
        Some(AgentRole::Researcher) => Ok(0.55),
        Some(AgentRole::Designer) => Ok(0.70),
        Some(AgentRole::PromptEngineer) => Ok(0.65),
        Some(AgentRole::ContextCompiler) => Ok(0.45),
        Some(AgentRole::MemoryWriter) => Ok(0.75),
        Some(AgentRole::MediaValidator) => Ok(0.78),
        // Any role outside the pilot set is treated as baseline (the
        // pilot harness will never pass it; this arm is defensive).
        Some(_) => Ok(0.80),
    }
}

fn now_ms() -> u64 {
    // B-15 Tranche E E.1: ClockProvider via zero-sized OsClock Copy impl.
    // The trait's default `now_ms` returns u128 with `.unwrap_or(0)`
    // saturation absorbed (crates/core/src/clock.rs:59-64); narrow to u64
    // with the same saturating semantic as the prior direct call.
    u64::try_from(OsClock.now_ms()).unwrap_or(u64::MAX)
}

async fn run_attribute(
    dropped_role: Option<&str>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let ts_ms = now_ms();
    let report = if let Some(name) = dropped_role {
        let role = resolve_role(name)?;
        let harness =
            CreditAttributionHarness::new(9, 3, vec![role], ts_ms);
        harness
            .run(synthetic_score)
            .await
            .map_err(|e| format!("credit attribute (single-role) failed: {e}"))?
    } else {
        let harness = CreditAttributionHarness::pilot(ts_ms);
        harness
            .run(synthetic_score)
            .await
            .map_err(|e| format!("credit attribute (pilot) failed: {e}"))?
    };
    render_report(&report, output_format);
    Ok(())
}

async fn run_table(output_format: CliOutputFormat) -> Result<(), String> {
    let harness = CreditAttributionHarness::pilot(now_ms());
    let report = harness
        .run(synthetic_score)
        .await
        .map_err(|e| format!("credit table render failed: {e}"))?;
    render_report(&report, output_format);
    Ok(())
}

async fn run_export(path: &PathBuf, output_format: CliOutputFormat) -> Result<(), String> {
    let harness = CreditAttributionHarness::pilot(now_ms());
    let report = harness
        .run(synthetic_score)
        .await
        .map_err(|e| format!("credit export harness failed: {e}"))?;
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("credit export encode failed: {e}"))?;
    fs::write(path, json).map_err(|e| {
        format!(
            "credit export write failed for {}: {e}",
            path.display()
        )
    })?;
    if matches!(output_format, CliOutputFormat::Json) {
        let summary = serde_json::json!({
            "exported_to": path.display().to_string(),
            "total_roles": report.len(),
            "ts_ms": report.ts_ms,
        });
        println!("{summary}");
    } else {
        println!(
            "credit export wrote {} role rows to {}",
            report.len(),
            path.display()
        );
    }
    Ok(())
}

fn render_report(report: &CreditReport, output_format: CliOutputFormat) {
    match output_format {
        CliOutputFormat::Json => {
            // Pretty-printed JSON for piping into the cantante-pilot.md
            // artifact + future dashboard consumers.
            let json = serde_json::to_string_pretty(report)
                .unwrap_or_else(|_| "{}".into());
            println!("{json}");
        }
        _ => {
            println!("credit attribution table (NOTE: synthetic fixture; real credits via runpod pilot):");
            println!("  ts_ms: {}", report.ts_ms);
            println!("  total roles: {}", report.len());
            // Order rows by the canonical pilot_roles() ordering for
            // stable diffs.
            for role in pilot_roles().iter().copied() {
                if let Some(score) = report.get(role) {
                    let role_str = serde_json::to_value(role)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| format!("{role:?}"));
                    println!("    {role_str:<22} {score:+.4}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_role_accepts_canonical_snake() {
        assert_eq!(resolve_role("executor").expect("executor"), AgentRole::Executor);
        assert_eq!(resolve_role("memory_writer").expect("memory_writer"), AgentRole::MemoryWriter);
    }

    #[test]
    fn resolve_role_rejects_out_of_pilot() {
        // IdeaFramer is intentionally excluded from the C-45 pilot.
        let err = resolve_role("idea_framer").expect_err("out-of-pilot must error");
        assert!(err.contains("idea_framer"));
        assert!(err.contains("8 base roles"));
    }

    #[test]
    fn resolve_role_rejects_unknown() {
        let err = resolve_role("nonexistent_role").expect_err("unknown must error");
        assert!(err.contains("nonexistent_role"));
    }
}
