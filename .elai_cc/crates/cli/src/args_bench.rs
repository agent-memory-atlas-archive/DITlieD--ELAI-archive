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
    if sub == "routing-economics" {
        return parse_bench_routing_economics_args(&rest[1..]);
    }
    if sub == "model-refresh" {
        return parse_bench_model_refresh_args(&rest[1..]);
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
    if sub == "context-fill" {
        return parse_bench_context_fill_args(&rest[1..]);
    }
    if sub == "longcot" {
        return parse_bench_longcot_args(&rest[1..]);
    }
    if sub == "positional-probe" {
        return parse_bench_positional_probe_args(&rest[1..]);
    }
    if sub == "kv-tune" {
        return parse_bench_kv_tune_args(&rest[1..]);
    }
    if sub == "qdelta" {
        return parse_bench_qdelta_args(&rest[1..]);
    }
    if sub == "snowballing" {
        return parse_bench_snowballing_args(&rest[1..]);
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
    if sub == "hallucination-delta" {
        return parse_bench_hallucination_delta_args(&rest[1..]);
    }
    if sub == "local-lane-e2e" {
        return parse_bench_local_lane_e2e_args(&rest[1..]);
    }
    if sub == "steering-vs-prompt" {
        return parse_bench_steering_vs_prompt_args(&rest[1..]);
    }
    if sub == "retrieval-adapt" {
        return parse_bench_retrieval_adapt_args(&rest[1..]);
    }
    if sub == "tool-output" {
        return parse_bench_tool_output_args(&rest[1..]);
    }
    if sub == "rrf-ablate" {
        return parse_bench_rrf_ablate_args(&rest[1..]);
    }
    if sub == "perception-faithfulness" {
        return parse_bench_perception_faithfulness_args(&rest[1..]);
    }
    if sub != "prmbench" {
        return Err(format!(
            "unknown `bench` sub-command: '{sub}' (expected 'prmbench' | 'mode-routing' | 'conformal' | 'arc' | 'rag' | 'sycophancy' | 'canary-regression' | 'tamper' | 'seccodeplt' | 'error-amplification' | 'safedelta' | 'case-reuse' | 'routing-economics' | 'model-refresh' | 'gate-tax' | 'staleness' | 'mcp-schema-budget' | 'crane' | 'convergence' | 'context-fill' | 'positional-probe' | 'qdelta' | 'kv-tune' | 'snowballing' | 'claim-scope' | 'governance-probe' | 'sequence-probe' | 'hallucination-delta' | 'local-lane-e2e' | 'longcot' | 'steering-vs-prompt' | 'retrieval-adapt' | 'rrf-ablate' | 'tool-output' | 'perception-faithfulness')"
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

/// G-6: parse `elai bench safedelta [--suites <csv>] [--seed <u64>] [--json]
/// [--endpoint <url>] [--model <id>]`.
///
/// `--suites` accepts a comma-separated list of suite labels. Tier-1 only
/// processes `seccodeplt`; other suites require a live RunPod endpoint and
/// will return a `RunpodPending` error at runtime. If omitted, defaults to
/// `["seccodeplt"]` (the embedded static-gate corpus).
///
/// `--seed` accepts a `u64`. If omitted, defaults to 0 (the default ELAI_SEED
/// path, same as `from_embedded`).
///
/// `--endpoint`/`--model` (P1.a Tier-2 skeleton) opt in to the live-model leg:
/// when `--endpoint` is present, `run_inner` builds a `SidecarRuntime` against
/// that URL (model defaults to the sidecar default when `--model` is absent),
/// runs a fail-loud health check, and issues ONE sample `complete()` call
/// before the static ablation. Absent = byte-for-byte the Tier-1 static path.
fn parse_bench_safedelta_args(rest: &[String]) -> Result<CliAction, String> {
    let mut suites: Vec<String> = Vec::new();
    let mut seed: u64 = 0;
    let mut json = false;
    let mut endpoint: Option<String> = None;
    let mut model: Option<String> = None;
    let mut live = false;
    let mut protocol: Option<String> = None;
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
            "--endpoint" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --endpoint".to_string())?;
                endpoint = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--endpoint=") => {
                endpoint = Some(flag["--endpoint=".len()..].to_string());
                i += 1;
            }
            "--model" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --model".to_string())?;
                model = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--model=") => {
                model = Some(flag["--model=".len()..].to_string());
                i += 1;
            }
            "--live" => {
                live = true;
                i += 1;
            }
            "--protocol" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --protocol".to_string())?;
                protocol = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--protocol=") => {
                let raw = &flag["--protocol=".len()..];
                protocol = Some(raw.to_string());
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
    Ok(CliAction::BenchSafedelta {
        suites,
        seed,
        json,
        endpoint,
        model,
        live,
        protocol,
    })
}

/// PS-3: parse `elai bench hallucination-delta [--suites <csv>] [--seed <u64>]
/// [--json] [--endpoint <url>] [--model <id>]`.
///
/// Mirrors `parse_bench_safedelta_args`. Tier-1 only processes `cgg`; other
/// suites require a live RunPod endpoint and will return a `RunpodPending`
/// error at runtime. `--endpoint` is rejected at runtime in this fixture-tier
/// slice (live CGG ablation is RunPod-gated). If omitted, defaults to
/// `["cgg"]` (the embedded fixture corpus).
fn parse_bench_hallucination_delta_args(rest: &[String]) -> Result<CliAction, String> {
    let mut suites: Vec<String> = Vec::new();
    let mut seed: u64 = 0;
    let mut json = false;
    let mut endpoint: Option<String> = None;
    let mut model: Option<String> = None;
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
            "--endpoint" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --endpoint".to_string())?;
                endpoint = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--endpoint=") => {
                endpoint = Some(flag["--endpoint=".len()..].to_string());
                i += 1;
            }
            "--model" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --model".to_string())?;
                model = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--model=") => {
                model = Some(flag["--model=".len()..].to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench hallucination-delta` argument: {other}"));
            }
        }
    }
    if suites.is_empty() {
        suites.push("cgg".to_string());
    }
    Ok(CliAction::BenchHallucinationDelta {
        suites,
        seed,
        json,
        endpoint,
        model,
    })
}

/// PS-READINESS-E2ERUNNER Slice B — parse `elai bench local-lane-e2e
/// [--endpoint <url>] [--model <id>] [--json]`.
///
/// The ONLY live consumer of the 4B sidecar at http://127.0.0.1:8181. No
/// fixture fallback (R1): a missing/unreachable sidecar is a hard FAIL at
/// runtime, never a synthesized pass. `--endpoint` defaults to
/// http://127.0.0.1:8181; `--model` defaults to qwen3.5-4b-q4_k_m.
fn parse_bench_local_lane_e2e_args(rest: &[String]) -> Result<CliAction, String> {
    let mut json = false;
    let mut endpoint: Option<String> = None;
    let mut model: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => {
                json = true;
                i += 1;
            }
            "--endpoint" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --endpoint".to_string())?;
                endpoint = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--endpoint=") => {
                endpoint = Some(flag["--endpoint=".len()..].to_string());
                i += 1;
            }
            "--model" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --model".to_string())?;
                model = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--model=") => {
                model = Some(flag["--model=".len()..].to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench local-lane-e2e` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchLocalLaneE2e { json, endpoint, model })
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

/// K-2 -- parse `elai bench routing-economics [--seed <u64>] [--json]
/// [--force-cloud-everything]`.
fn parse_bench_routing_economics_args(rest: &[String]) -> Result<CliAction, String> {
    let mut seed = 0_u64;
    let mut json = false;
    let mut force_cloud_everything = false;
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
            "--force-cloud-everything" => {
                force_cloud_everything = true;
                i += 1;
            }
            other => {
                return Err(format!(
                    "unknown `bench routing-economics` argument: {other}"
                ));
            }
        }
    }
    Ok(CliAction::BenchRoutingEconomics {
        seed,
        json,
        force_cloud_everything,
    })
}

/// K-9 -- parse `elai bench model-refresh [--lane <coding|vision|ocr|embedding_reranker>]
/// [--seed <u64>] [--json]`.
fn parse_bench_model_refresh_args(rest: &[String]) -> Result<CliAction, String> {
    let mut lane: Option<String> = None;
    let mut seed = 0_u64;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--lane" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --lane".to_string())?;
                lane = Some(v.clone());
                i += 2;
            }
            f if f.starts_with("--lane=") => {
                lane = Some(f["--lane=".len()..].to_string());
                i += 1;
            }
            "--seed" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = v
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --seed value '{v}': {e}"))?;
                i += 2;
            }
            f if f.starts_with("--seed=") => {
                let raw = &f["--seed=".len()..];
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
                return Err(format!("unknown `bench model-refresh` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchModelRefresh { lane, seed, json })
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

// J-34 leg-1: `elai bench context-fill --suite <PATH> [--seeds <u32>] [--json]`
// Mirrors parse_bench_convergence_args shape. Tier-1 = fixture TOML only (no
// live inference). Emits ContextCalibrationEvent(CurveMeasured) via the
// CalibrationArtifactStore::insert_with_ssb seam (slice-2).
fn parse_bench_context_fill_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
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
                return Err(format!("unknown `bench context-fill` argument: {other}"));
            }
        }
    }
    let suite = suite
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "`bench context-fill` requires --suite <PATH>".to_string())?;
    Ok(crate::args::CliAction::BenchContextFill { suite, seeds, json })
}

// J-107: `elai bench longcot --suite <PATH> [--seeds <u32>] [--json]`
fn parse_bench_longcot_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
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
                return Err(format!("unknown `bench longcot` argument: {other}"));
            }
        }
    }
    let suite = suite
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "`bench longcot` requires --suite <PATH>".to_string())?;
    Ok(crate::args::CliAction::BenchLongcot { suite, seeds, json })
}

// J-34 leg-3: `elai bench positional-probe --suite <PATH> [--seeds <u32>] [--json]`
// Mirrors parse_bench_context_fill_args shape. Tier-1 = fixture TOML only (no
// live inference). Emits ContextCalibrationEvent(CurveMeasured) via the
// CalibrationArtifactStore::insert_with_ssb seam (slice-2).
fn parse_bench_positional_probe_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut suite: Option<String> = None;
    let mut seeds: u32 = 0;
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
                return Err(format!("unknown `bench positional-probe` argument: {other}"));
            }
        }
    }
    let suite = suite
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "`bench positional-probe` requires --suite <PATH>".to_string())?;
    Ok(crate::args::CliAction::BenchPositionalProbe { suite, seeds, json })
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

fn parse_bench_kv_tune_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut dry_run = false; let mut emit_ssb = false; let mut json = false;
    for arg in rest {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--emit-ssb" => emit_ssb = true,
            "--json" => json = true,
            other => return Err(format!("unknown `bench kv-tune` argument: {other}")),
        }
    }
    Ok(crate::args::CliAction::BenchKvTune { dry_run, emit_ssb, json })
}

fn parse_bench_qdelta_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut config: Option<String> = None;
    let mut seed: u64 = 42;
    let mut json: bool = false;
    let mut quality: bool = false;
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
            "--quality" => {
                quality = true;
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
    Ok(crate::args::CliAction::BenchQdelta { config, seed, json, quality })
}

// ---------------------------------------------------------------------------
// C-37: `elai bench snowballing --pool A|B --output <PATH> [...]`
// ---------------------------------------------------------------------------

fn parse_bench_snowballing_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut pool: Option<String> = None;
    let mut n_samples: Option<i64> = None;
    let mut model: Option<String> = None;
    let mut max_trials: Option<i64> = None;
    let mut reflector_mode: Option<String> = None;
    let mut sidecar_url: Option<String> = None;
    let mut output: Option<String> = None;
    let mut python_bin: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--pool" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --pool".to_string())?;
                pool = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--pool=") => {
                pool = Some(flag["--pool=".len()..].to_string());
                i += 1;
            }
            "--n-samples" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --n-samples".to_string())?;
                n_samples = Some(
                    value
                        .parse::<i64>()
                        .map_err(|e| format!("invalid --n-samples value '{value}': {e}"))?,
                );
                i += 2;
            }
            flag if flag.starts_with("--n-samples=") => {
                let raw = &flag["--n-samples=".len()..];
                n_samples = Some(
                    raw.parse::<i64>()
                        .map_err(|e| format!("invalid --n-samples value '{raw}': {e}"))?,
                );
                i += 1;
            }
            "--model" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --model".to_string())?;
                model = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--model=") => {
                model = Some(flag["--model=".len()..].to_string());
                i += 1;
            }
            "--max-trials" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --max-trials".to_string())?;
                max_trials = Some(
                    value
                        .parse::<i64>()
                        .map_err(|e| format!("invalid --max-trials value '{value}': {e}"))?,
                );
                i += 2;
            }
            flag if flag.starts_with("--max-trials=") => {
                let raw = &flag["--max-trials=".len()..];
                max_trials = Some(
                    raw.parse::<i64>()
                        .map_err(|e| format!("invalid --max-trials value '{raw}': {e}"))?,
                );
                i += 1;
            }
            "--reflector-mode" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --reflector-mode".to_string())?;
                reflector_mode = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--reflector-mode=") => {
                reflector_mode = Some(flag["--reflector-mode=".len()..].to_string());
                i += 1;
            }
            "--sidecar-url" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --sidecar-url".to_string())?;
                sidecar_url = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--sidecar-url=") => {
                sidecar_url = Some(flag["--sidecar-url=".len()..].to_string());
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
            "--python-bin" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --python-bin".to_string())?;
                python_bin = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--python-bin=") => {
                python_bin = Some(flag["--python-bin=".len()..].to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench snowballing` argument: {other}"));
            }
        }
    }
    let pool = pool.ok_or_else(|| "`bench snowballing` requires --pool A|B".to_string())?;
    let output = output.ok_or_else(|| "`bench snowballing` requires --output <PATH>".to_string())?;
    Ok(crate::args::CliAction::BenchSnowballing {
        pool,
        n_samples,
        model,
        max_trials,
        reflector_mode,
        sidecar_url,
        output,
        python_bin,
    })
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

// ---------------------------------------------------------------------------
// J-24 MS4: `elai bench steering-vs-prompt --task <fixture> [--seed <u64>] [--json]`
// ---------------------------------------------------------------------------

fn parse_bench_steering_vs_prompt_args(
    rest: &[String],
) -> Result<crate::args::CliAction, String> {
    let mut task: Option<String> = None;
    let mut seed: u64 = 0;
    let mut json: bool = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--task" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --task".to_string())?;
                task = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--task=") => {
                task = Some(flag["--task=".len()..].to_string());
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
                return Err(format!(
                    "unknown `bench steering-vs-prompt` argument: {other}"
                ));
            }
        }
    }
    let task = task.ok_or_else(|| "missing required --task <fixture> argument".to_string())?;
    Ok(crate::args::CliAction::BenchSteeringVsPrompt {
        task: std::path::PathBuf::from(task),
        seed,
        json,
    })
}

// ---------------------------------------------------------------------------
// J-72 slice D: `elai bench retrieval-adapt --gold <path> [--k <N>]
//   [--baseline-ranks <json>] [--adapted-ranks <json>] [--out <report.json>]
//   [--json]`
// ---------------------------------------------------------------------------

/// J-72 slice D — parse `elai bench retrieval-adapt --gold <path> [--k <N>]
/// [--baseline-ranks <json>] [--adapted-ranks <json>] [--out <report.json>]
/// [--json]`. Hand-rolled per the CLAUDE.md "no new clap dep" rule.
/// `--gold` is required; `--k` defaults to 10.
fn parse_bench_retrieval_adapt_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut gold: Option<String> = None;
    let mut k: u32 = 10;
    let mut baseline_ranks: Option<String> = None;
    let mut adapted_ranks: Option<String> = None;
    let mut out: Option<String> = None;
    let mut json: bool = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--gold" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --gold".to_string())?;
                gold = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--gold=") => {
                gold = Some(flag["--gold=".len()..].to_string());
                i += 1;
            }
            "--k" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --k".to_string())?;
                k = value
                    .parse::<u32>()
                    .map_err(|e| format!("invalid --k value '{value}': {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--k=") => {
                let raw = &flag["--k=".len()..];
                k = raw
                    .parse::<u32>()
                    .map_err(|e| format!("invalid --k value '{raw}': {e}"))?;
                i += 1;
            }
            "--baseline-ranks" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --baseline-ranks".to_string())?;
                baseline_ranks = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--baseline-ranks=") => {
                baseline_ranks = Some(flag["--baseline-ranks=".len()..].to_string());
                i += 1;
            }
            "--adapted-ranks" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --adapted-ranks".to_string())?;
                adapted_ranks = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--adapted-ranks=") => {
                adapted_ranks = Some(flag["--adapted-ranks=".len()..].to_string());
                i += 1;
            }
            "--out" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --out".to_string())?;
                out = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--out=") => {
                out = Some(flag["--out=".len()..].to_string());
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench retrieval-adapt` argument: {other}"));
            }
        }
    }
    let gold = gold
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "`bench retrieval-adapt` requires --gold <PATH>".to_string())?;
    if k == 0 {
        return Err("--k must be >= 1".to_string());
    }
    Ok(crate::args::CliAction::BenchRetrievalAdapt {
        gold,
        k,
        baseline_ranks: baseline_ranks.map(std::path::PathBuf::from),
        adapted_ranks: adapted_ranks.map(std::path::PathBuf::from),
        out: out.map(std::path::PathBuf::from),
        json,
    })
}

/// J-75 — parse `elai bench rrf-ablate --gold PATH [--k N]` and optional rank maps.

fn parse_bench_tool_output_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut corpus = None;
    let mut seed: u64 = 1234;
    let mut json = false;
    let mut out = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--corpus" => { corpus = Some(PathBuf::from(rest.get(i + 1).ok_or_else(|| "missing value for --corpus".to_string())?)); i += 2; }
            flag if flag.starts_with("--corpus=") => { corpus = Some(PathBuf::from(&flag[9..])); i += 1; }
            "--seed" => { let v = rest.get(i + 1).ok_or_else(|| "missing value for --seed".to_string())?; seed = v.parse().map_err(|e| format!("invalid --seed {v:?}: {e}"))?; i += 2; }
            flag if flag.starts_with("--seed=") => { seed = flag[7..].parse().map_err(|e| format!("invalid --seed {:?}: {e}", &flag[7..]))?; i += 1; }
            "--out" => { out = Some(PathBuf::from(rest.get(i + 1).ok_or_else(|| "missing value for --out".to_string())?)); i += 2; }
            flag if flag.starts_with("--out=") => { out = Some(PathBuf::from(&flag[6..])); i += 1; }
            "--json" => { json = true; i += 1; }
            other => return Err(format!("unknown `bench tool-output` argument: {other}")),
        }
    }
    Ok(crate::args::CliAction::BenchToolOutput { corpus, seed, json, out })
}

fn parse_bench_rrf_ablate_args(rest: &[String]) -> Result<crate::args::CliAction, String> {
    let mut gold: Option<PathBuf> = None;
    let mut k: u32 = 5;
    let mut full_ranks = None;
    let mut drop_pagerank_ranks = None;
    let mut drop_second_dense_ranks = None;
    let mut nav_only_graph_ranks = None;
    let mut out = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--gold" => { gold = Some(PathBuf::from(rest.get(i + 1).ok_or_else(|| "missing value for --gold".to_string())?)); i += 2; }
            flag if flag.starts_with("--gold=") => { gold = Some(PathBuf::from(&flag[7..])); i += 1; }
            "--k" => { let v = rest.get(i + 1).ok_or_else(|| "missing value for --k".to_string())?; k = v.parse().map_err(|e| format!("invalid --k {v:?}: {e}"))?; i += 2; }
            flag if flag.starts_with("--k=") => { k = flag[4..].parse().map_err(|e| format!("invalid --k {:?}: {e}", &flag[4..]))?; i += 1; }
            "--full-ranks" | "--drop-pagerank-ranks" | "--drop-second-dense-ranks" | "--nav-only-graph-ranks" | "--out" => {
                let v = PathBuf::from(rest.get(i + 1).ok_or_else(|| format!("missing value for {}", rest[i]))?);
                match rest[i].as_str() { "--full-ranks" => full_ranks=Some(v), "--drop-pagerank-ranks" => drop_pagerank_ranks=Some(v), "--drop-second-dense-ranks" => drop_second_dense_ranks=Some(v), "--nav-only-graph-ranks" => nav_only_graph_ranks=Some(v), _ => out=Some(v) }; i += 2;
            }
            flag if flag.starts_with("--full-ranks=") => { full_ranks=Some(PathBuf::from(&flag[13..])); i += 1; }
            flag if flag.starts_with("--drop-pagerank-ranks=") => { drop_pagerank_ranks=Some(PathBuf::from(&flag[22..])); i += 1; }
            flag if flag.starts_with("--drop-second-dense-ranks=") => { drop_second_dense_ranks=Some(PathBuf::from(&flag[26..])); i += 1; }
            flag if flag.starts_with("--nav-only-graph-ranks=") => { nav_only_graph_ranks=Some(PathBuf::from(&flag[23..])); i += 1; }
            flag if flag.starts_with("--out=") => { out=Some(PathBuf::from(&flag[6..])); i += 1; }
            "--json" => { json=true; i += 1; }
            other => return Err(format!("unknown `bench rrf-ablate` argument: {other}")),
        }
    }
    let gold = gold.ok_or_else(|| "`bench rrf-ablate` requires --gold <PATH>".to_string())?;
    if k == 0 { return Err("--k must be >= 1".to_string()); }
    Ok(crate::args::CliAction::BenchRrfAblate { gold, k, full_ranks, drop_pagerank_ranks, drop_second_dense_ranks, nav_only_graph_ranks, out, json })
}

#[cfg(test)]
mod rrf_ablate_parse_tests {
    use super::*;
    #[test]
    fn happy_path_defaults_k() {
        let a = parse_bench_subcommand_args(&["rrf-ablate".into(), "--gold".into(), "g.jsonl".into()]).unwrap();
        match a { crate::args::CliAction::BenchRrfAblate { k, gold, .. } => { assert_eq!(k, 5); assert_eq!(gold, PathBuf::from("g.jsonl")); }, _ => panic!("wrong action") }
    }
    #[test]
    fn missing_gold_errors() { assert!(parse_bench_subcommand_args(&["rrf-ablate".into()]).unwrap_err().contains("requires --gold")); }
}

// ---------------------------------------------------------------------------
// J-77 I11: `elai bench perception-faithfulness [--fixture PATH] [--json]
//   [--enforce]`
// ---------------------------------------------------------------------------

/// J-77 I11 — parse `elai bench perception-faithfulness [--fixture PATH]
/// [--json] [--enforce]`. Hand-rolled per the CLAUDE.md "no new clap dep"
/// rule. `--fixture` defaults to
/// `.elai_cc/data/perception/fixtures/j77_6metric_fixture.json` (resolved
/// at runtime relative to CWD). `--enforce` exits non-zero on metric fail.
fn parse_bench_perception_faithfulness_args(
    rest: &[String],
) -> Result<crate::args::CliAction, String> {
    let mut fixture: Option<String> = None;
    let mut json: bool = false;
    let mut enforce: bool = false;
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
            "--json" => {
                json = true;
                i += 1;
            }
            "--enforce" => {
                enforce = true;
                i += 1;
            }
            other => {
                return Err(format!(
                    "unknown `bench perception-faithfulness` argument: {other}"
                ));
            }
        }
    }
    Ok(crate::args::CliAction::BenchPerceptionFaithfulness {
        fixture: fixture.map(std::path::PathBuf::from).unwrap_or_default(),
        json,
        enforce,
    })
}

#[cfg(test)]
mod retrieval_adapt_parse_tests {
    use super::*;

    #[test]
    fn parse_retrieval_adapt_happy_path() {
        let args: Vec<String> = vec![
            "retrieval-adapt".into(),
            "--gold".into(),
            "gold.jsonl".into(),
            "--k".into(),
            "5".into(),
            "--json".into(),
        ];
        let action = parse_bench_subcommand_args(&args).unwrap();
        match action {
            crate::args::CliAction::BenchRetrievalAdapt {
                gold,
                k,
                baseline_ranks,
                adapted_ranks,
                out,
                json,
            } => {
                assert_eq!(gold, std::path::PathBuf::from("gold.jsonl"));
                assert_eq!(k, 5);
                assert!(baseline_ranks.is_none());
                assert!(adapted_ranks.is_none());
                assert!(out.is_none());
                assert!(json);
            }
            other => panic!("expected BenchRetrievalAdapt, got {other:?}"),
        }
    }

    #[test]
    fn parse_retrieval_adapt_missing_gold_errors() {
        let args: Vec<String> = vec!["retrieval-adapt".into(), "--k".into(), "5".into()];
        let err = parse_bench_subcommand_args(&args).unwrap_err();
        assert!(err.contains("requires --gold"), "err={err}");
    }
}
