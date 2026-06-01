// Tranche O.3 (T10): symbol-analysis parser block extracted from args.rs.
// Hosts parse_symbol_graph_args, parse_symbol_impact_args,
// parse_symbol_context_args, parse_entry_points_args.
//
// parse_output_format_value STAYS in args.rs (pub(crate)) and is called via
// crate::args::parse_output_format_value.  CliAction enum variants and
// CliOutputFormat stay in args.rs.

#![allow(dead_code)]

use std::path::PathBuf;

use crate::args::{CliAction, CliOutputFormat};

pub(crate) fn parse_symbol_graph_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    let flag_rest = if rest.is_empty() { &rest[..] } else { &rest[1..] };

    let mut project_root: Option<PathBuf> = None;
    // E-21 Tier-A: `rank --top N` accepts an additional --top flag (default 20).
    let mut top: u32 = 0;
    let mut i = 0;
    while i < flag_rest.len() {
        match flag_rest[i].as_str() {
            "--project-root" => {
                let value = flag_rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag[15..]));
                i += 1;
            }
            "--top" => {
                let value = flag_rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --top".to_string())?;
                top = value
                    .parse::<u32>()
                    .map_err(|e| format!("--top must be a non-negative integer: {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--top=") => {
                top = flag[6..]
                    .parse::<u32>()
                    .map_err(|e| format!("--top must be a non-negative integer: {e}"))?;
                i += 1;
            }
            other => {
                return Err(format!("unknown `symbol_graph` argument: {other}"));
            }
        }
    }

    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| {
            format!("`symbol_graph` cannot determine cwd for --project-root default: {e}")
        })?,
    };

    match sub {
        "build" => Ok(CliAction::SymbolGraphBuild { project_root }),
        "stats" => Ok(CliAction::SymbolGraphStats { project_root }),
        "rank" => Ok(CliAction::SymbolGraphRank { project_root, top }),
        other => Err(format!(
            "unknown `symbol_graph` sub-command: '{other}' (expected 'build', 'stats', or 'rank')"
        )),
    }
}

/// B-29 Phase 5b — parse `elai symbol_impact --target <X> [--depth N]
/// [--project-root PATH] [--output-format text|json]`. Thin shell wrapper
/// around `compute_impact_radius` for fork-exec callers.
pub(crate) fn parse_symbol_impact_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut project_root: Option<PathBuf> = None;
    let mut target: Option<String> = None;
    let mut depth: u32 = 2;
    let mut output_format_override: Option<CliOutputFormat> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--project-root" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag[15..]));
                i += 1;
            }
            "--target" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --target".to_string())?;
                target = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--target=") => {
                target = Some(flag[9..].to_string());
                i += 1;
            }
            "--depth" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --depth".to_string())?;
                depth = value
                    .parse::<u32>()
                    .map_err(|e| format!("--depth must be a non-negative integer: {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--depth=") => {
                depth = flag[8..]
                    .parse::<u32>()
                    .map_err(|e| format!("--depth must be a non-negative integer: {e}"))?;
                i += 1;
            }
            "--output-format" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output-format".to_string())?;
                output_format_override =
                    Some(crate::args::parse_output_format_value(value)?);
                i += 2;
            }
            flag if flag.starts_with("--output-format=") => {
                output_format_override =
                    Some(crate::args::parse_output_format_value(&flag[16..])?);
                i += 1;
            }
            "--json" => {
                output_format_override = Some(CliOutputFormat::Json);
                i += 1;
            }
            other => {
                return Err(format!("unknown `symbol_impact` argument: {other}"));
            }
        }
    }
    let target = target.ok_or_else(|| "missing required --target".to_string())?;
    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| {
            format!("`symbol_impact` cannot determine cwd for --project-root default: {e}")
        })?,
    };
    Ok(CliAction::SymbolImpact {
        project_root,
        target,
        depth,
        output_format: output_format_override.unwrap_or(output_format),
    })
}

/// B-29 Phase 5b — parse `elai symbol_context --name <X> [--file PATH]
/// [--project-root PATH] [--output-format text|json]`. Thin shell wrapper
/// around `compute_symbol_context` for fork-exec callers.
pub(crate) fn parse_symbol_context_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut project_root: Option<PathBuf> = None;
    let mut name: Option<String> = None;
    let mut file: Option<String> = None;
    let mut output_format_override: Option<CliOutputFormat> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--project-root" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag[15..]));
                i += 1;
            }
            "--name" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --name".to_string())?;
                name = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--name=") => {
                name = Some(flag[7..].to_string());
                i += 1;
            }
            "--file" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --file".to_string())?;
                file = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--file=") => {
                file = Some(flag[7..].to_string());
                i += 1;
            }
            "--output-format" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output-format".to_string())?;
                output_format_override =
                    Some(crate::args::parse_output_format_value(value)?);
                i += 2;
            }
            flag if flag.starts_with("--output-format=") => {
                output_format_override =
                    Some(crate::args::parse_output_format_value(&flag[16..])?);
                i += 1;
            }
            "--json" => {
                output_format_override = Some(CliOutputFormat::Json);
                i += 1;
            }
            other => {
                return Err(format!("unknown `symbol_context` argument: {other}"));
            }
        }
    }
    let name = name.ok_or_else(|| "missing required --name".to_string())?;
    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| {
            format!("`symbol_context` cannot determine cwd for --project-root default: {e}")
        })?,
    };
    Ok(CliAction::SymbolContext {
        project_root,
        name,
        file,
        output_format: output_format_override.unwrap_or(output_format),
    })
}

/// B-34 Phase 5b — parse `elai entry_points [--project-root PATH]
/// [--output-format text|json] [--limit N]`. Thin shell wrapper around
/// `detect_entry_points` for fork-exec callers. Mirrors the
/// TOOL_ENTRY_POINTS dispatcher in production_agent_rpc.rs (orphan +
/// well-known + framework heuristics, tests excluded by default).
pub(crate) fn parse_entry_points_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut project_root: Option<PathBuf> = None;
    let mut limit: usize = 100;
    let mut output_format_override: Option<CliOutputFormat> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--project-root" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag[15..]));
                i += 1;
            }
            "--limit" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --limit".to_string())?;
                limit = value
                    .parse::<usize>()
                    .map_err(|e| format!("--limit must be a non-negative integer: {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--limit=") => {
                limit = flag[8..]
                    .parse::<usize>()
                    .map_err(|e| format!("--limit must be a non-negative integer: {e}"))?;
                i += 1;
            }
            "--output-format" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output-format".to_string())?;
                output_format_override =
                    Some(crate::args::parse_output_format_value(value)?);
                i += 2;
            }
            flag if flag.starts_with("--output-format=") => {
                output_format_override =
                    Some(crate::args::parse_output_format_value(&flag[16..])?);
                i += 1;
            }
            "--json" => {
                output_format_override = Some(CliOutputFormat::Json);
                i += 1;
            }
            other => {
                return Err(format!("unknown `entry_points` argument: {other}"));
            }
        }
    }
    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| {
            format!("`entry_points` cannot determine cwd for --project-root default: {e}")
        })?,
    };
    Ok(CliAction::EntryPoints {
        project_root,
        limit,
        output_format: output_format_override.unwrap_or(output_format),
    })
}
