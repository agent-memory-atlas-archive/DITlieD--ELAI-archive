// Tranche O.3 (T10): memory/incident parser block extracted from args.rs.
// Hosts parse_memory_subcommand_args and parse_incident_subcommand_args.

#![allow(dead_code)]

use std::path::PathBuf;

use crate::args::{CliAction, MemorySubcommandKind};

pub(crate) fn parse_memory_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let Some(sub_raw) = rest.first().map(String::as_str) else {
        return Err(
            "memory: expected sub-command (list | inspect | insert | delete)".to_string(),
        );
    };
    let sub = match sub_raw {
        "list" => MemorySubcommandKind::List,
        "inspect" | "show" => MemorySubcommandKind::Inspect,
        "insert" | "add" => MemorySubcommandKind::Insert,
        "delete" | "forget" => MemorySubcommandKind::Delete,
        "pin" => MemorySubcommandKind::Pin,
        "unpin" => MemorySubcommandKind::Unpin,
        "provenance" | "prov" => MemorySubcommandKind::Provenance,
        other => {
            return Err(format!(
                "unknown `memory` sub-command: '{other}' (expected list | inspect | insert | delete | pin | unpin | provenance)"
            ));
        }
    };
    let rest = &rest[1..];
    let mut project_root: Option<PathBuf> = None;
    let mut user_id: Option<String> = None;
    let mut scope: Option<String> = None;
    let mut fact_type: Option<String> = None;
    let mut fact_id: Option<String> = None;
    let mut key: Option<String> = None;
    let mut value_json: Option<String> = None;
    let mut limit: Option<usize> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--project-root" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(v));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag["--project-root=".len()..]));
                i += 1;
            }
            "--user-id" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --user-id".to_string())?;
                user_id = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--user-id=") => {
                user_id = Some(flag["--user-id=".len()..].to_string());
                i += 1;
            }
            "--scope" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --scope".to_string())?;
                scope = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--scope=") => {
                scope = Some(flag["--scope=".len()..].to_string());
                i += 1;
            }
            "--fact-type" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --fact-type".to_string())?;
                fact_type = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--fact-type=") => {
                fact_type = Some(flag["--fact-type=".len()..].to_string());
                i += 1;
            }
            "--fact-id" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --fact-id".to_string())?;
                fact_id = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--fact-id=") => {
                fact_id = Some(flag["--fact-id=".len()..].to_string());
                i += 1;
            }
            "--key" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --key".to_string())?;
                key = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--key=") => {
                key = Some(flag["--key=".len()..].to_string());
                i += 1;
            }
            "--value-json" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --value-json".to_string())?;
                value_json = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--value-json=") => {
                value_json = Some(flag["--value-json=".len()..].to_string());
                i += 1;
            }
            "--limit" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --limit".to_string())?;
                limit = Some(
                    v.parse::<usize>()
                        .map_err(|e| format!("--limit must be a non-negative integer: {e}"))?,
                );
                i += 2;
            }
            flag if flag.starts_with("--limit=") => {
                limit = Some(
                    flag["--limit=".len()..]
                        .parse::<usize>()
                        .map_err(|e| format!("--limit must be a non-negative integer: {e}"))?,
                );
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => return Err(format!("unknown `memory {sub_raw}` argument: {other}")),
        }
    }
    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| format!("cannot determine cwd: {e}"))?,
    };
    let user_id = user_id.unwrap_or_else(|| "default".to_string());
    let limit = limit.unwrap_or(50).max(1).min(500);
    Ok(CliAction::Memory {
        sub,
        project_root,
        user_id,
        scope,
        fact_type,
        fact_id,
        key,
        value_json,
        limit,
        json,
    })
}

/// E-30 — parse `elai incident record [--title T] [--date D] [--url U] [--id I]`.
///
/// Stores an incident postmortem entry in the SSB and emits
/// SsbActionType::IncidentRecordCreated. The `record` sub-verb is mandatory
/// (leaves room for `list` / `search` in future E-plan phases).
pub(crate) fn parse_incident_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let Some(sub) = rest.first().map(String::as_str) else {
        return Err("incident: expected sub-command 'record'".to_string());
    };
    if sub != "record" {
        return Err(format!(
            "unknown `incident` sub-command: '{sub}' (expected 'record')"
        ));
    }
    let rest = &rest[1..];
    let mut title: Option<String> = None;
    let mut incident_date: Option<String> = None;
    let mut source_url: Option<String> = None;
    let mut incident_id: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--title" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --title".to_string())?;
                title = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--title=") => {
                title = Some(flag["--title=".len()..].to_string());
                i += 1;
            }
            "--date" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --date".to_string())?;
                incident_date = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--date=") => {
                incident_date = Some(flag["--date=".len()..].to_string());
                i += 1;
            }
            "--url" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --url".to_string())?;
                source_url = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--url=") => {
                source_url = Some(flag["--url=".len()..].to_string());
                i += 1;
            }
            "--id" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --id".to_string())?;
                incident_id = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--id=") => {
                incident_id = Some(flag["--id=".len()..].to_string());
                i += 1;
            }
            other => return Err(format!("unknown `incident record` argument: {other}")),
        }
    }
    let title = title.ok_or_else(|| "`incident record` requires --title".to_string())?;
    let incident_date =
        incident_date.ok_or_else(|| "`incident record` requires --date".to_string())?;
    let source_url =
        source_url.ok_or_else(|| "`incident record` requires --url".to_string())?;
    Ok(CliAction::IncidentRecord {
        title,
        incident_date,
        source_url,
        incident_id,
    })
}
