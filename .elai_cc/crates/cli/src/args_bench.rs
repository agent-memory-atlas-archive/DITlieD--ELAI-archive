// B-15 Tranche O.2 (orchestrate s-22600): bench parser block extraction.
// Hosts the eight `parse_bench_*_args` functions + `validate_bool_arg`
// helper carved out of crates/cli/src/args.rs L3358-L4081 (~722 LOC).
//
// CliAction enum is UNCHANGED — all eight Bench* variants stay in
// crates/cli/src/args.rs L568-L674; only the parser functions relocate.
// Single call site change at args.rs L2018 dispatches via
// `args_bench::parse_bench_subcommand_args(...)`.
//
// Visibility: only `parse_bench_subcommand_args` needs `pub(crate)` since
// it is the dispatch entrypoint. The seven sibling parsers and
// `validate_bool_arg` collapse to module-private — they were `pub(crate)`
// only to be accessible within args.rs itself, which is no longer needed.

#![allow(dead_code)]

use std::path::PathBuf;

use crate::args::CliAction;

/// D-19 — parse `elai bench prmbench [--dataset PATH] [--hook mock|http://host:port]
/// [--output PATH] [--gating-threshold 0.30] [--category NAME]`.
///
/// Only one sub-verb is supported (`prmbench`). The flag grammar mirrors
/// `archive verify`: long-form `--flag VALUE` and `--flag=VALUE` both
/// accepted. Validation of the values (URL parsing for `--hook`, range
/// check on `--gating-threshold`) lives in the bench_prmbench handler
/// boundary so this parser stays free of orchestrator type deps.
pub(crate) fn parse_bench_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    if sub == "mode-routing" {
        return parse_bench_mode_routing_args(&rest[1..]);
    }
    if sub == "conformal" {
        return parse_bench_conformal_args(&rest[1..]);
    }
    if sub == "arc" {
        return parse_bench_arc_args(&rest[1..]);
    }
    if sub == "rag" {
        return parse_bench_rag_args(&rest[1..]);
    }
    if sub == "sycophancy" {
        return parse_bench_sycophancy_args(&rest[1..]);
    }
    if sub == "canary-regression" {
        return parse_bench_canary_args(&rest[1..]);
    }
    if sub == "tamper" {
        return parse_bench_tamper_args(&rest[1..]);
    }
    if sub == "seccodeplt" {
        return parse_bench_seccodeplt_args(&rest[1..]);
    }
    if sub == "error-amplification" {
        return parse_bench_error_amplification_args(&rest[1..]);
    }
    if sub == "safedelta" {
        return parse_bench_safedelta_args(&rest[1..]);
    }
    if sub == "case-reuse" {
        return parse_bench_case_reuse_args(&rest[1..]);
    }
    if sub == "gate-tax" {
        return parse_bench_gate_tax_args(&rest[1..]);
    }
    if sub == "staleness" {
        return parse_bench_staleness_args(&rest[1..]);
    }
    if sub == "mcp-schema-budget" {
        return parse_bench_mcp_schema_budget_args(&rest[1..]);
    }
    if sub == "crane" {
        return parse_bench_crane_args(&rest[1..]);
    }
    if sub == "convergence" {
        return parse_bench_convergence_args(&rest[1..]);
    }
    if sub == "qdelta" {
        return parse_bench_qdelta_args(&rest[1..]);
    }
    if sub == "claim-scope" {
        return parse_bench_claim_scope_args(&rest[1..]);
    }
    if sub == "governance-probe" {
        return parse_bench_governance_probe_args(&rest[1..]);
    }
    if sub == "sequence-probe" {
        return parse_bench_sequence_probe_args(&rest[1..]);
    }
    if sub != "prmbench" {
        return Err(format!(
            "unknown `bench` sub-command: '{sub}' (expected 'prmbench' | 'mode-routing' | 'conformal' | 'arc' | 'rag' | 'sycophancy' | 'canary-regression' | 'tamper' | 'seccodeplt' | 'error-amplification' | 'safedelta' | 'case-reuse' | 'gate-tax' | 'staleness' | 'mcp-schema-budget' | 'crane' | 'convergence' | 'qdelta' | 'claim-scope' | 'governance-probe' | 'sequence-probe')"
        ));
    }
    let mut dataset: Option<String> = None;
    let mut hook: String = "mock".to_string();
    let mut output: Option<String> = None;
    let mut gating_threshold: Option<String> = None;
    let mut category: Option<String> = None;
    let mut i = 1;
    while i < rest.len() {
        match rest[i].as_str() {
            "--dataset" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --dataset".to_string())?;
                dataset = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--dataset=") => {
                dataset = Some(flag["--dataset=".len()..].to_string());
                i += 1;
            }
            "--hook" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --hook".to_string())?;
                hook = value.clone();
                i += 2;
            }
            flag if flag.starts_with("--hook=") => {
                hook = flag["--hook=".len()..].to_string();
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--gating-threshold" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --gating-threshold".to_string())?;
                // Validate parseability up front so a malformed flag fails
                // at the parser boundary instead of mid-run.
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --gating-threshold {value:?}: {e}"))?;
                gating_threshold = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--gating-threshold=") => {
                let value = &flag["--gating-threshold=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --gating-threshold {value:?}: {e}"))?;
                gating_threshold = Some(value.to_string());
                i += 1;
            }
            "--category" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --category".to_string())?;
                category = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--category=") => {
                category = Some(flag["--category=".len()..].to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench prmbench` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchPrmbench {
        dataset,
        hook,
        output,
        gating_threshold,
        category,
    })
}

/// D-21 — parse `elai bench mode-routing [--dataset PATH] [--output PATH]
/// [--macro-floor 0.85] [--per-mode-floor 0.70]`. Mirrors the prmbench
/// parser; hand-rolled per the CLAUDE.md "no new clap dep" rule.
/// Validation of the floor values lives at the handler boundary.
fn parse_bench_mode_routing_args(rest: &[String]) -> Result<CliAction, String> {
    let mut dataset: Option<String> = None;
    let mut output: Option<String> = None;
    let mut macro_floor: Option<String> = None;
    let mut per_mode_floor: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--dataset" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --dataset".to_string())?;
                dataset = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--dataset=") => {
                dataset = Some(flag["--dataset=".len()..].to_string());
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--macro-floor" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --macro-floor".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --macro-floor {value:?}: {e}"))?;
                macro_floor = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--macro-floor=") => {
                let value = &flag["--macro-floor=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --macro-floor {value:?}: {e}"))?;
                macro_floor = Some(value.to_string());
                i += 1;
            }
            "--per-mode-floor" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --per-mode-floor".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --per-mode-floor {value:?}: {e}"))?;
                per_mode_floor = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--per-mode-floor=") => {
                let value = &flag["--per-mode-floor=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --per-mode-floor {value:?}: {e}"))?;
                per_mode_floor = Some(value.to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench mode-routing` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchModeRouting {
        dataset,
        output,
        macro_floor,
        per_mode_floor,
    })
}

/// D-22 — parse `elai bench conformal [--dataset PATH] [--corpus PATH]
/// [--output PATH] [--epsilon 0.1]`. Mirrors the mode-routing parser;
/// hand-rolled per the CLAUDE.md "no new clap dep" rule. Validation of
/// the epsilon value lives at the handler boundary.
fn parse_bench_conformal_args(rest: &[String]) -> Result<CliAction, String> {
    let mut dataset: Option<String> = None;
    let mut corpus: Option<String> = None;
    let mut output: Option<String> = None;
    let mut epsilon: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--dataset" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --dataset".to_string())?;
                dataset = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--dataset=") => {
                dataset = Some(flag["--dataset=".len()..].to_string());
                i += 1;
            }
            "--corpus" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --corpus".to_string())?;
                corpus = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--corpus=") => {
                corpus = Some(flag["--corpus=".len()..].to_string());
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--epsilon" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --epsilon".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --epsilon {value:?}: {e}"))?;
                epsilon = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--epsilon=") => {
                let value = &flag["--epsilon=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --epsilon {value:?}: {e}"))?;
                epsilon = Some(value.to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench conformal` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchConformal {
        dataset,
        corpus,
        output,
        epsilon,
    })
}

/// D-20 — parse `elai bench arc [--dataset PATH] [--output PATH]
/// [--novelty-config PATH]`. Hand-rolled per the CLAUDE.md "no new clap
/// dep" rule.
fn parse_bench_arc_args(rest: &[String]) -> Result<CliAction, String> {
    let mut dataset: Option<String> = None;
    let mut output: Option<String> = None;
    let mut novelty_config: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--dataset" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --dataset".to_string())?;
                dataset = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--dataset=") => {
                dataset = Some(flag["--dataset=".len()..].to_string());
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--novelty-config" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --novelty-config".to_string())?;
                novelty_config = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--novelty-config=") => {
                novelty_config = Some(flag["--novelty-config=".len()..].to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench arc` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchArc {
        dataset,
        output,
        novelty_config,
    })
}

/// E-17 SP6 — parse `elai bench rag --tier <STANDARD|DEEP> --top-k <N>
/// [--project-root PATH] [--json] -- <QUERY...>`. Hand-rolled per the
/// CLAUDE.md "no new clap dep" rule. The query is taken from positional
/// arguments after the `--` sentinel (joined with spaces); flags before
/// `--` are parsed as named options. `--tier` is required; everything
/// else has a default (top-k=10, json=false, project-root=cwd).
fn parse_bench_rag_args(rest: &[String]) -> Result<CliAction, String> {
    let mut tier: Option<String> = None;
    let mut top_k: u32 = 10;
    let mut project_root: Option<PathBuf> = None;
    let mut json: bool = false;
    let mut query_parts: Vec<String> = Vec::new();
    let mut i = 0;
    let mut saw_sentinel = false;
    while i < rest.len() {
        let arg = rest[i].as_str();
        if saw_sentinel {
            query_parts.push(rest[i].clone());
            i += 1;
            continue;
        }
        match arg {
            "--" => {
                saw_sentinel = true;
                i += 1;
            }
            "--tier" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --tier".to_string())?;
                tier = Some(value.to_uppercase());
                i += 2;
            }
            flag if flag.starts_with("--tier=") => {
                tier = Some(flag["--tier=".len()..].to_uppercase());
                i += 1;
            }
            "--top-k" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --top-k".to_string())?;
                top_k = value
                    .parse()
                    .map_err(|e| format!("invalid --top-k {value:?}: {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--top-k=") => {
                let value = &flag["--top-k=".len()..];
                top_k = value
                    .parse()
                    .map_err(|e| format!("invalid --top-k {value:?}: {e}"))?;
                i += 1;
            }
            "--project-root" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag["--project-root=".len()..]));
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                // Pre-sentinel positional: treat as the start of an
                // unsentineled query. Permissive grammar: `elai bench rag
                // --tier STANDARD --top-k 5 my query here` is equivalent
                // to `... -- my query here` once we hit a non-flag arg.
                if other.starts_with("--") {
                    return Err(format!("unknown `bench rag` argument: {other}"));
                }
                saw_sentinel = true;
                query_parts.push(rest[i].clone());
                i += 1;
            }
        }
    }
    let tier_value = tier
        .ok_or_else(|| "missing required --tier (STANDARD or DEEP)".to_string())?;
    if tier_value != "STANDARD" && tier_value != "DEEP" && tier_value != "FAST" {
        return Err(format!(
            "invalid --tier {tier_value:?} (expected STANDARD | DEEP | FAST)"
        ));
    }
    let query = query_parts.join(" ").trim().to_string();
    if query.is_empty() {
        return Err("missing query (pass after `--` or as trailing positional args)".to_string());
    }
    if top_k == 0 {
        return Err("--top-k must be >= 1".to_string());
    }
    let resolved_project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir()
            .map_err(|e| format!("cannot determine cwd for --project-root: {e}"))?,
    };
    Ok(CliAction::BenchRag {
        tier: tier_value,
        top_k,
        query,
        json,
        project_root: resolved_project_root,
    })
}

/// D-23 — parse `elai bench sycophancy [--fixtures PATH] [--output PATH]
/// [--detection-floor 0.70] [--fp-ceiling 0.15]`. Mirrors the prmbench
/// parser; hand-rolled per the CLAUDE.md "no new clap dep" rule.
/// Validation of the floor / ceiling values lives at the handler boundary.
fn parse_bench_sycophancy_args(rest: &[String]) -> Result<CliAction, String> {
    let mut fixtures: Option<String> = None;
    let mut output: Option<String> = None;
    let mut detection_floor: Option<String> = None;
    let mut fp_ceiling: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--fixtures" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --fixtures".to_string())?;
                fixtures = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--fixtures=") => {
                fixtures = Some(flag["--fixtures=".len()..].to_string());
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--detection-floor" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --detection-floor".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --detection-floor {value:?}: {e}"))?;
                detection_floor = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--detection-floor=") => {
                let value = &flag["--detection-floor=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --detection-floor {value:?}: {e}"))?;
                detection_floor = Some(value.to_string());
                i += 1;
            }
            "--fp-ceiling" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --fp-ceiling".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --fp-ceiling {value:?}: {e}"))?;
                fp_ceiling = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--fp-ceiling=") => {
                let value = &flag["--fp-ceiling=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --fp-ceiling {value:?}: {e}"))?;
                fp_ceiling = Some(value.to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench sycophancy` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchSycophancy {
        fixtures,
        output,
        detection_floor,
        fp_ceiling,
    })
}

/// D-24 — parse `elai bench canary-regression [--fixtures PATH] [--output
/// PATH] [--harmful-block-floor N] [--helpful-block-ceiling N]
/// [--neutral-block-ceiling N]`. Mirrors the sycophancy parser; hand-
/// rolled per the CLAUDE.md "no new clap dep" rule. Threshold values
/// are validated as u32 here so a stray `-1` or `foo` is caught at
/// parse time rather than during the runner's gate evaluation.
fn parse_bench_canary_args(rest: &[String]) -> Result<CliAction, String> {
    let mut fixtures: Option<String> = None;
    let mut output: Option<String> = None;
    let mut harmful_block_floor: Option<String> = None;
    let mut helpful_block_ceiling: Option<String> = None;
    let mut neutral_block_ceiling: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--fixtures" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --fixtures".to_string())?;
                fixtures = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--fixtures=") => {
                fixtures = Some(flag["--fixtures=".len()..].to_string());
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--harmful-block-floor" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --harmful-block-floor".to_string())?;
                let _: u32 = value
                    .parse()
                    .map_err(|e| format!("invalid --harmful-block-floor {value:?}: {e}"))?;
                harmful_block_floor = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--harmful-block-floor=") => {
                let value = &flag["--harmful-block-floor=".len()..];
                let _: u32 = value
                    .parse()
                    .map_err(|e| format!("invalid --harmful-block-floor {value:?}: {e}"))?;
                harmful_block_floor = Some(value.to_string());
                i += 1;
            }
            "--helpful-block-ceiling" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --helpful-block-ceiling".to_string())?;
                let _: u32 = value
                    .parse()
                    .map_err(|e| format!("invalid --helpful-block-ceiling {value:?}: {e}"))?;
                helpful_block_ceiling = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--helpful-block-ceiling=") => {
                let value = &flag["--helpful-block-ceiling=".len()..];
                let _: u32 = value
                    .parse()
                    .map_err(|e| format!("invalid --helpful-block-ceiling {value:?}: {e}"))?;
                helpful_block_ceiling = Some(value.to_string());
                i += 1;
            }
            "--neutral-block-ceiling" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --neutral-block-ceiling".to_string())?;
                let _: u32 = value
                    .parse()
                    .map_err(|e| format!("invalid --neutral-block-ceiling {value:?}: {e}"))?;
                neutral_block_ceiling = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--neutral-block-ceiling=") => {
                let value = &flag["--neutral-block-ceiling=".len()..];
                let _: u32 = value
                    .parse()
                    .map_err(|e| format!("invalid --neutral-block-ceiling {value:?}: {e}"))?;
                neutral_block_ceiling = Some(value.to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench canary-regression` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchCanary {
        fixtures,
        output,
        harmful_block_floor,
        helpful_block_ceiling,
        neutral_block_ceiling,
    })
}

/// D-25 — parse `elai bench tamper [--scenarios PATH] [--output PATH]
/// [--fail-fast=<true|false>] [--max-wall-ms-per-scenario N]`. Mirrors
/// the canary parser; hand-rolled per the CLAUDE.md "no new clap dep"
/// rule. Threshold values are validated as u64 here so a stray `-1` or
/// `foo` is caught at parse time rather than during the runner's gate
/// evaluation.
fn parse_bench_tamper_args(rest: &[String]) -> Result<CliAction, String> {
    let mut scenarios: Option<String> = None;
    let mut output: Option<String> = None;
    let mut fail_fast: Option<String> = None;
    let mut max_wall_ms_per_scenario: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--scenarios" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --scenarios".to_string())?;
                scenarios = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--scenarios=") => {
                scenarios = Some(flag["--scenarios=".len()..].to_string());
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--fail-fast" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --fail-fast".to_string())?;
                validate_bool_arg("--fail-fast", value)?;
                fail_fast = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--fail-fast=") => {
                let value = &flag["--fail-fast=".len()..];
                validate_bool_arg("--fail-fast", value)?;
                fail_fast = Some(value.to_string());
                i += 1;
            }
            "--max-wall-ms-per-scenario" => {
                let value = rest.get(i + 1).ok_or_else(|| {
                    "missing value for --max-wall-ms-per-scenario".to_string()
                })?;
                let _: u64 = value.parse().map_err(|e| {
                    format!("invalid --max-wall-ms-per-scenario {value:?}: {e}")
                })?;
                max_wall_ms_per_scenario = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--max-wall-ms-per-scenario=") => {
                let value = &flag["--max-wall-ms-per-scenario=".len()..];
                let _: u64 = value.parse().map_err(|e| {
                    format!("invalid --max-wall-ms-per-scenario {value:?}: {e}")
                })?;
                max_wall_ms_per_scenario = Some(value.to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench tamper` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchTamper {
        scenarios,
        output,
        fail_fast,
        max_wall_ms_per_scenario,
    })
}

/// F-2b — parse `elai bench seccodeplt [--json]`. The corpus is embedded at
/// compile time + the report dir is fixed (`<cwd>/benchmarks/seccodeplt/`), so
/// the only flag is `--json` (full report to stdout). No `--suite` flag (the
/// bench_cmd pattern has none). Hand-rolled per the CLAUDE.md "no new clap dep"
/// rule.
fn parse_bench_seccodeplt_args(rest: &[String]) -> Result<CliAction, String> {
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench seccodeplt` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchSeccodeplt { json })
}

/// I-9 — parse `elai bench error-amplification [--fixture PATH] [--seed <u64>]
/// [--json] [--no-repair]`. The fixture is embedded at compile time (default:
/// `pipeline_v1.json`); `--fixture` overrides it with a path. `--seed` pins the
/// perturbation seed (default: the `ELAI_SEED` path). `--no-repair` runs the R2
/// negative control. Hand-rolled per the CLAUDE.md "no new clap dep" rule.
fn parse_bench_error_amplification_args(rest: &[String]) -> Result<CliAction, String> {
    let mut fixture: Option<String> = None;
    let mut seed: Option<u64> = None;
    let mut json = false;
    let mut no_repair = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--fixture" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --fixture".to_string())?;
                fixture = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--fixture=") => {
                fixture = Some(flag["--fixture=".len()..].to_string());
                i += 1;
            }
            "--seed" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(
                    value
                        .parse::<u64>()
                        .map_err(|e| format!("invalid --seed value '{value}': {e}"))?,
                );
                i += 2;
            }
            flag if flag.starts_with("--seed=") => {
                let raw = &flag["--seed=".len()..];
                seed = Some(
                    raw.parse::<u64>()
                        .map_err(|e| format!("invalid --seed value '{raw}': {e}"))?,
                );
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            "--no-repair" => {
                no_repair = true;
                i += 1;
            }
            other => {
                return Err(format!(
                    "unknown `bench error-amplification` argument: {other}"
                ));
            }
        }
    }
    Ok(CliAction::BenchErrorAmplification {
        fixture,
        seed,
        json,
        no_repair,
    })
}

/// H-30 — parse `elai bench staleness [--json] [--broken-retrieval]`. The
/// fixture corpus is embedded at compile time + the report dir is fixed
/// (`<cwd>/benchmarks/staleness/`), so the flags are `--json` (full report to
/// stdout) and `--broken-retrieval` (the R2 negative-control arm). Hand-rolled
/// per the CLAUDE.md "no new clap dep" rule.
fn parse_bench_staleness_args(rest: &[String]) -> Result<CliAction, String> {
    let mut json = false;
    let mut broken_retrieval = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => {
                json = true;
                i += 1;
            }
            "--broken-retrieval" => {
                broken_retrieval = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench staleness` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchStaleness {
        json,
        broken_retrieval,
    })
}

/// G-6: parse `elai bench safedelta [--suites <csv>] [--seed <u64>] [--json]`.
///
/// `--suites` accepts a comma-separated list of suite labels. Tier-1 only
/// processes `seccodeplt`; other suites require a live RunPod endpoint and
/// will return a `RunpodPending` error at runtime. If omitted, defaults to
/// `["seccodeplt"]` (the embedded static-gate corpus).
///
/// `--seed` accepts a `u64`. If omitted, defaults to 0 (the default ELAI_SEED
/// path, same as `from_embedded`).
fn parse_bench_safedelta_args(rest: &[String]) -> Result<CliAction, String> {
    let mut suites: Vec<String> = Vec::new();
    let mut seed: u64 = 0;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--suites" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --suites".to_string())?;
                suites = value.split(',').map(|s| s.trim().to_string()).collect();
                i += 2;
            }
            flag if flag.starts_with("--suites=") => {
                let raw = &flag["--suites=".len()..];
                suites = raw.split(',').map(|s| s.trim().to_string()).collect();
                i += 1;
            }
            "--seed" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = value
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{value}': {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--seed=") => {
                let raw = &flag["--seed=".len()..];
                seed = raw
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{raw}': {e}"))?;
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench safedelta` argument: {other}"));
            }
        }
    }
    // Default to the embedded seccodeplt corpus when no suites specified.
    if suites.is_empty() {
        suites.push("seccodeplt".to_string());
    }
    Ok(CliAction::BenchSafedelta { suites, seed, json })
}

/// K-18a — parse `elai bench case-reuse [--suite fixture] [--seed <u64>] [--json]`.
///
/// Mirrors the `parse_bench_safedelta_args` grammar. Only the `fixture` suite is
/// runnable today (the real-journal `journal` suite is F-2 deferred, rejected at
/// the handler boundary). Default suite is `fixture`; default seed is 0.
fn parse_bench_case_reuse_args(rest: &[String]) -> Result<CliAction, String> {
    let mut suite: Option<String> = None;
    let mut seed: u64 = 0;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--suite" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --suite".to_string())?;
                suite = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--suite=") => {
                suite = Some(flag["--suite=".len()..].to_string());
                i += 1;
            }
            "--seed" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = value
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{value}': {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--seed=") => {
                let raw = &flag["--seed=".len()..];
                seed = raw
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{raw}': {e}"))?;
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench case-reuse` argument: {other}"));
            }
        }
    }
    let suite = suite.unwrap_or_else(|| "fixture".to_string());
    Ok(CliAction::BenchCaseReuse { suite, seed, json })
}

/// K-4 -- parse `elai bench gate-tax [--seed <u64>] [--json] [--enforce-ceiling]`.
///
/// No `--suite` flag (gate-tax has no fixture/journal suite toggle; the
/// runner itself decides Measured vs RunnerPresent from the real journal).
fn parse_bench_gate_tax_args(rest: &[String]) -> Result<CliAction, String> {
    let mut seed: u64 = 0;
    let mut json = false;
    let mut enforce_ceiling = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--seed" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = value
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{value}': {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--seed=") => {
                let raw = &flag["--seed=".len()..];
                seed = raw
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{raw}': {e}"))?;
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            "--enforce-ceiling" => {
                enforce_ceiling = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench gate-tax` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchGateTax { seed, json, enforce_ceiling })
}

/// H-34 -- parse `elai bench mcp-schema-budget [--json]`.
///
/// The fixture corpus is embedded at compile time and the report dir is fixed
/// (`<cwd>/benchmarks/mcp_schema_budget/`), so the only flag is `--json`
/// (full report to stdout). Hand-rolled per the CLAUDE.md "no new clap dep" rule.
fn parse_bench_mcp_schema_budget_args(rest: &[String]) -> Result<CliAction, String> {
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!(
                    "unknown `bench mcp-schema-budget` argument: {other}"
                ));
            }
        }
    }
    Ok(CliAction::BenchMcpSchemaBudget { json })
}

/// I-34 — parse `elai bench crane [--crane-fixture PATH] [--seed <u64>]`.
///
/// Deterministic CRANE grammar-pin gate fixture harness. No live inference runs
/// (W-10 PENDING checkbox). `--crane-fixture` is the path to a `.ebnf` fixture
/// file used for the blake3 pin verification arms; if omitted the harness uses
/// a built-in synthetic fixture. `--seed` is passed through to the harness for
/// reproducible test ordering; defaults to 42. Hand-rolled per the CLAUDE.md
/// "no new clap dep" rule.
fn parse_bench_crane_args(rest: &[String]) -> Result<CliAction, String> {
    let mut crane_fixture: Option<String> = None;
    let mut seed: Option<u64> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--crane-fixture" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --crane-fixture".to_string())?;
                crane_fixture = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--crane-fixture=") => {
                crane_fixture = Some(flag["--crane-fixture=".len()..].to_string());
                i += 1;
            }
            "--seed" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(
                    value
                        .parse::<u64>()
                        .map_err(|e| format!("invalid --seed value '{value}': {e}"))?,
                );
                i += 2;
            }
            flag if flag.starts_with("--seed=") => {
                let raw = &flag["--seed=".len()..];
                seed = Some(
                    raw.parse::<u64>()
                        .map_err(|e| format!("invalid --seed value '{raw}': {e}"))?,
                );
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench crane` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchCrane {
        crane_fixture,
        seed,
    })
}

// ---------------------------------------------------------------------------
// I-27: `elai bench convergence --suite <PATH> [--seeds <u32>] [--json]`
// ---------------------------------------------------------------------------

/// I-27 — parse `elai bench convergence --suite <PATH> [--seeds <u32>] [--json]`.
///
/// `--suite` (required): path to the convergence fixture TOML.
/// `--seeds` (optional): override seed_count from config (u32, must be >= 5).
/// `--json`: print the full beacon JSON to stdout after the summary line.
/// Hand-rolled per the CLAUDE.md "no new clap dep" rule.
fn parse_bench_convergence_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut suite: Option<String> = None;
    let mut seeds: u32 = 0; // 0 = use config's seed_count
    let mut json: bool = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--suite" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --suite".to_string())?;
                suite = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--suite=") => {
                suite = Some(flag["--suite=".len()..].to_string());
                i += 1;
            }
            "--seeds" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seeds".to_string())?;
                seeds = value
                    .parse::<u32>()
                    .map_err(|e| format!("invalid --seeds value '{value}': {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--seeds=") => {
                let raw = &flag["--seeds=".len()..];
                seeds = raw
                    .parse::<u32>()
                    .map_err(|e| format!("invalid --seeds value '{raw}': {e}"))?;
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench convergence` argument: {other}"));
            }
        }
    }
    let suite = suite
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "`bench convergence` requires --suite <PATH>".to_string())?;
    Ok(crate::args::CliAction::BenchConvergence { suite, seeds, json })
}

fn validate_bool_arg(name: &str, value: &str) -> Result<(), String> {
    match value {
        "true" | "false" | "1" | "0" | "TRUE" | "FALSE" | "True" | "False" => Ok(()),
        other => Err(format!(
            "invalid {name} {other:?}: expected 'true'|'false'|'1'|'0'"
        )),
    }
}

// ---------------------------------------------------------------------------
// I-38: `elai bench qdelta --config <PATH> [--seed <u64>] [--json]`
// ---------------------------------------------------------------------------

fn parse_bench_qdelta_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut config: Option<String> = None;
    let mut seed: u64 = 42;
    let mut json: bool = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--config" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --config".to_string())?;
                config = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--config=") => {
                config = Some(flag["--config=".len()..].to_string());
                i += 1;
            }
            "--seed" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = value
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{value}': {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--seed=") => {
                let raw = &flag["--seed=".len()..];
                seed = raw
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{raw}': {e}"))?;
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench qdelta` argument: {other}"));
            }
        }
    }
    let config = config
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "`bench qdelta` requires --config <PATH>".to_string())?;
    Ok(crate::args::CliAction::BenchQdelta { config, seed, json })
}

// ---------------------------------------------------------------------------
// I-43: `elai bench claim-scope [--suite <SUITE>] [--seed <u64>] [--json]`
// ---------------------------------------------------------------------------

/// I-43 — parse `elai bench claim-scope [--suite <SUITE>] [--seed <u64>] [--json]`.
///
/// `--suite` accepts a fixture suite label (default: "seccodeplt_claim_scope_fixture").
/// `--seed` accepts a `u64` seed (default: 42).
/// `--json` prints the full report JSON to stdout after the summary line.
/// Hand-rolled per the CLAUDE.md "no new clap dep" rule.
fn parse_bench_claim_scope_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut suite: Option<String> = None;
    let mut seed: u64 = 42;
    let mut json: bool = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--suite" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --suite".to_string())?;
                suite = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--suite=") => {
                suite = Some(flag["--suite=".len()..].to_string());
                i += 1;
            }
            "--seed" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = value
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{value}': {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--seed=") => {
                let raw = &flag["--seed=".len()..];
                seed = raw
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{raw}': {e}"))?;
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench claim-scope` argument: {other}"));
            }
        }
    }
    Ok(crate::args::CliAction::BenchClaimScope { suite, seed, json })
}

// ---------------------------------------------------------------------------
// I-35: `elai bench sequence-probe [--seed <u64>]`
// ---------------------------------------------------------------------------

/// Parse args for `elai bench sequence-probe`.
///
/// Hand-rolled per the CLAUDE.md "no new clap dep" rule. Only `--seed` is
/// accepted; defaults to 42.
fn parse_bench_sequence_probe_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut seed: u64 = 42;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--seed" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = value
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{value}': {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--seed=") => {
                let raw = &flag["--seed=".len()..];
                seed = raw
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{raw}': {e}"))?;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench sequence-probe` argument: {other}"));
            }
        }
    }
    Ok(crate::args::CliAction::BenchSequenceProbe { seed })
}

// ---------------------------------------------------------------------------
// I-33: `elai bench governance-probe [--seed <u64>]`
// ---------------------------------------------------------------------------

/// Parse args for `elai bench governance-probe`.
///
/// Hand-rolled per the CLAUDE.md "no new clap dep" rule. Only `--seed` is
/// accepted; defaults to 42.
fn parse_bench_governance_probe_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut seed: u64 = 42;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--seed" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = value
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{value}': {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--seed=") => {
                let raw = &flag["--seed=".len()..];
                seed = raw
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{raw}': {e}"))?;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench governance-probe` argument: {other}"));
            }
        }
    }
    Ok(crate::args::CliAction::BenchGovernanceProbe { seed })
}
