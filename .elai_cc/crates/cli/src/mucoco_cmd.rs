//! C-36 `/mucoco` slash-command handler.
//!
//! Two actions:
//! - `/mucoco list`                                              — enumerate
//!   the 11 paper-grounded operators from arXiv:2604.19086.
//! - `/mucoco mutate <file> --operator <name> [--language python]` —
//!   read the file, dispatch the named MuCoCo operator via the sidecar
//!   `/mucoco/mutate` endpoint, and print the result (applied bit +
//!   mutated source or skip reason).
//!
//! Sidecar URL is taken from `ANTHROPIC_BASE_URL` (the canonical sidecar
//! address shared by the rest of the CLI) and falls back to the local
//! default `http://127.0.0.1:8100`. The handler does NOT boot the
//! sidecar — health-check is the caller's responsibility (see
//! sidecar-protocol skill / `workflow/remote-setup/REMOTE-SIDECAR.md`).
//!
//! Anti-staleness: native async fn, anyhow for error propagation, no
//! `.unwrap()` / `.expect()` outside test code.

use std::fs;
use std::path::Path;

use elai_orchestrator::{
    dispatch_mucoco_mutation, list_mucoco_operators, MucocoOperator,
};

/// Default sidecar base URL when `ANTHROPIC_BASE_URL` is unset (matches the
/// canonical local default in CLAUDE.md "Remote sidecar (canonical)" block).
const DEFAULT_SIDECAR_BASE_URL: &str = "http://127.0.0.1:8100";

/// Resolve the sidecar base URL from env, defaulting to the local sidecar.
fn resolve_sidecar_base_url() -> String {
    std::env::var("ANTHROPIC_BASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SIDECAR_BASE_URL.to_owned())
}

/// Entry point invoked by `live_cli::print_mucoco`. Returns a printable
/// status block. Errors propagate via anyhow; the caller renders them.
pub async fn handle_mucoco_slash_command(
    action: Option<&str>,
    file: Option<&str>,
    operator: Option<&str>,
    language: Option<&str>,
) -> Result<String, anyhow::Error> {
    match action.unwrap_or("help") {
        "help" => Ok(render_help()),
        "list" => Ok(render_operator_list()),
        "mutate" => {
            let file_path = file.ok_or_else(|| {
                anyhow::anyhow!("missing <file> argument for /mucoco mutate")
            })?;
            let operator_name = operator.ok_or_else(|| {
                anyhow::anyhow!("missing --operator <name> for /mucoco mutate")
            })?;
            let lang = language.unwrap_or("python");
            mutate_file(file_path, operator_name, lang).await
        }
        other => Err(anyhow::anyhow!(
            "unknown /mucoco action {other:?}; use list | mutate | help"
        )),
    }
}

fn render_help() -> String {
    let mut out = String::new();
    out.push_str("MuCoCo Mutation Operator Catalog (C-36, arXiv:2604.19086)\n");
    out.push_str("Subcommands:\n");
    out.push_str("  /mucoco list                                                  Show all 11 operators\n");
    out.push_str("  /mucoco mutate <file> --operator <name> [--language python]   Apply one operator\n");
    out.push_str("  /mucoco help                                                  This message\n");
    out.push_str("\nOperators run inside the Python sidecar (python/elai_inference/mucoco/);\n");
    out.push_str("the sidecar must be reachable (default http://127.0.0.1:8100 or via ANTHROPIC_BASE_URL).\n");
    out
}

fn render_operator_list() -> String {
    let mut out = String::new();
    out.push_str("MuCoCo operators (11, paper Table 9 / Appendix B):\n");
    out.push_str("  Lexical:\n");
    for op in [
        MucocoOperator::RandomMutation,
        MucocoOperator::SequentialMutation,
        MucocoOperator::LiteralFormat,
    ] {
        out.push_str(&format!("    {}\n", op.as_str()));
    }
    out.push_str("  Syntactic:\n");
    for op in [
        MucocoOperator::ForToEnumerate,
        MucocoOperator::ForToWhile,
    ] {
        out.push_str(&format!("    {}\n", op.as_str()));
    }
    out.push_str("  Logical:\n");
    for op in [
        MucocoOperator::DeMorgan,
        MucocoOperator::BooleanLiteral,
        MucocoOperator::CommutativeReorder,
        MucocoOperator::ConstantUnfolding,
        MucocoOperator::ConstantUnfoldingAdd,
        MucocoOperator::ConstantUnfoldingMul,
    ] {
        out.push_str(&format!("    {}\n", op.as_str()));
    }
    // Defensive count assert — if the registry adds an operator we forget to
    // print here, the test surface catches it; the user-facing output still
    // declares the canonical 11.
    debug_assert_eq!(list_mucoco_operators().len(), 11);
    out
}

async fn mutate_file(
    file_path: &str,
    operator_name: &str,
    language: &str,
) -> Result<String, anyhow::Error> {
    let path = Path::new(file_path);
    let source = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {file_path:?}: {e}"))?;
    let operator: MucocoOperator = operator_name
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid --operator {operator_name:?}: {e}"))?;
    let base_url = resolve_sidecar_base_url();
    let result = dispatch_mucoco_mutation(&base_url, &source, language, operator)
        .await
        .map_err(|e| anyhow::anyhow!("sidecar dispatch failed: {e}"))?;
    let mut out = String::new();
    out.push_str(&format!("operator    = {}\n", result.operator));
    out.push_str(&format!("language    = {language}\n"));
    out.push_str(&format!("input_file  = {file_path}\n"));
    out.push_str(&format!("applied     = {}\n", result.applied));
    if result.applied {
        out.push_str("---------- mutated source ----------\n");
        out.push_str(&result.mutated_source);
        if !result.mutated_source.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("------------------------------------\n");
    } else {
        let reason = result
            .skip_reason
            .as_deref()
            .unwrap_or("<missing skip_reason — sidecar protocol drift>");
        out.push_str(&format!("skip_reason = {reason}\n"));
    }
    Ok(out)
}
