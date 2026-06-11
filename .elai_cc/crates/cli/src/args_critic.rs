// Tranche O.3 (T10): critic/eval-loop parser block extracted from args.rs.
// Hosts parse_bench_ablation_args, parse_darwin_loop_args, parse_tier_eval_args,
// parse_critic_report_args, parse_lora_gate_check_args.
//
// These do NOT collide with args_bench.rs parse_bench_* names: args_bench
// handles the `bench` sub-family (prmbench, mode-routing, conformal, arc,
// rag, sycophancy, canary, tamper); args_critic handles the E-7/E-10/E-47
// standalone critic/eval-loop surfaces.

#![allow(dead_code)]

use crate::args::CliAction;

pub(crate) fn parse_bench_ablation_args(rest: &[String]) -> Result<CliAction, String> {
    let tile = rest.first().cloned().unwrap_or_else(|| "fsm".to_string());
    let mut fixtures: String = "fixtures/critic_30.toml".to_string();
    let mut budget: u32 = 5000;
    let mut i = 1;
    while i < rest.len() {
        match rest[i].as_str() {
            "--fixtures" => {
                fixtures = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --fixtures".to_string())?
                    .clone();
                i += 2;
            }
            "--budget" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --budget".to_string())?;
                budget = v.parse().map_err(|e| format!("invalid --budget: {e}"))?;
                i += 2;
            }
            other => return Err(format!("unknown flag for `bench-ablation`: {other}")),
        }
    }
    Ok(CliAction::BenchAblation { tile, fixtures, budget })
}

/// E-7 — `elai darwin-loop` argument parser. All flags optional with
/// defaults matching `darwinian_bai::DEFAULT_DISCOUNT_FACTOR` and the
/// plan task 8 trap_tasks fixture count guidance.
pub(crate) fn parse_darwin_loop_args(rest: &[String]) -> Result<CliAction, String> {
    let mut generations: u32 = 1;
    let mut mutants: u32 = 3;
    let mut seed: u64 = 42;
    let mut traps: u32 = 3;
    let mut gamma: String = String::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--generations" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --generations".to_string())?;
                generations = v.parse().map_err(|e| format!("invalid --generations: {e}"))?;
                i += 2;
            }
            "--mutants" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --mutants".to_string())?;
                mutants = v.parse().map_err(|e| format!("invalid --mutants: {e}"))?;
                i += 2;
            }
            "--seed" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = v.parse().map_err(|e| format!("invalid --seed: {e}"))?;
                i += 2;
            }
            "--traps" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --traps".to_string())?;
                traps = v.parse().map_err(|e| format!("invalid --traps: {e}"))?;
                i += 2;
            }
            "--gamma" => {
                gamma = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --gamma".to_string())?
                    .clone();
                i += 2;
            }
            other => return Err(format!("unknown flag for `darwin-loop`: {other}")),
        }
    }
    Ok(CliAction::DarwinLoop { generations, mutants, seed, traps, gamma })
}

pub(crate) fn parse_tier_eval_args(rest: &[String]) -> Result<CliAction, String> {
    let mut suites: Vec<String> = Vec::new();
    let mut budget: u32 = 100_000;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--suites" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --suites".to_string())?;
                suites = v
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                i += 2;
            }
            "--budget" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --budget".to_string())?;
                budget = v.parse().map_err(|e| format!("invalid --budget: {e}"))?;
                i += 2;
            }
            other => return Err(format!("unknown flag for `tier-eval`: {other}")),
        }
    }
    Ok(CliAction::TierEval { suites, budget })
}

/// E-6 — parse `elai mast report [--session <id>] [--limit N] [--json]`.
/// `rest` is the args AFTER the `mast` token. The only subcommand is
/// `report`; it may be omitted (bare `elai mast` == `elai mast report`).
pub(crate) fn parse_mast_args(rest: &[String]) -> Result<CliAction, String> {
    // Skip an optional leading "report" subcommand token.
    let args: &[String] = match rest.first().map(String::as_str) {
        Some("report") => &rest[1..],
        _ => rest,
    };
    let mut session: Option<String> = None;
    let mut limit: Option<usize> = None;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--session" => {
                session = Some(
                    args.get(i + 1)
                        .ok_or_else(|| "missing value for --session".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--limit" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --limit".to_string())?;
                limit = Some(v.parse().map_err(|e| format!("invalid --limit: {e}"))?);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => return Err(format!("unknown flag for `mast`: {other}")),
        }
    }
    Ok(CliAction::Mast { session, limit, json })
}

pub(crate) fn parse_critic_report_args(rest: &[String]) -> Result<CliAction, String> {
    let mut output: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--output" => {
                output = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --output".to_string())?
                        .clone(),
                );
                i += 2;
            }
            other => return Err(format!("unknown flag for `critic-report`: {other}")),
        }
    }
    Ok(CliAction::CriticReport { output })
}

/// E-47 -- parse `elai lora-gate-check [--config PATH] [--json]`.
/// Loads the `[finetune.lora]` block from the resolved config and
/// evaluates the triple-AND ship-gate.
pub(crate) fn parse_lora_gate_check_args(rest: &[String]) -> Result<CliAction, String> {
    let mut config: Option<String> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--config" => {
                config = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --config".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => return Err(format!("unknown flag for `lora-gate-check`: {other}")),
        }
    }
    Ok(CliAction::LoraGateCheck { config, json })
}

#[cfg(test)]
mod tests {
    use crate::args::{parse_args, CliAction};

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    /// E-6 ON-PATH dispatch: the real top-level `parse_args` resolves
    /// `mast report --session <id> --limit N --json` to `CliAction::Mast`
    /// with the parsed fields. This drives the SAME entrypoint chain
    /// (`parse_args` -> `"mast"` arm -> `parse_mast_args`) that `elai mast`
    /// uses in production, so the dispatch is proven reachable, not just
    /// the parser in isolation.
    #[test]
    fn mast_report_dispatches_through_parse_args() {
        let action = parse_args(&argv(&[
            "mast", "report", "--session", "s-1", "--limit", "50", "--json",
        ]))
        .expect("mast report must parse");
        assert_eq!(
            action,
            CliAction::Mast {
                session: Some("s-1".to_string()),
                limit: Some(50),
                json: true,
            }
        );
    }

    /// Bare `elai mast` (no subcommand, no flags) defaults to a Mast report
    /// over all sessions.
    #[test]
    fn bare_mast_defaults_to_report_all() {
        let action = parse_args(&argv(&["mast"])).expect("bare mast must parse");
        assert_eq!(
            action,
            CliAction::Mast {
                session: None,
                limit: None,
                json: false,
            }
        );
    }
}
