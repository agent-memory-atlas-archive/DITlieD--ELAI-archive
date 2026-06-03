// Tranche O.3 (T10): skills/eval/rl/checkpoint parser block extracted from args.rs.
// Hosts parse_skills_subcommand_args, parse_eval_subcommand_args,
// parse_rl_subcommand_args, parse_checkpoint_subcommand_args.

#![allow(dead_code)]

use crate::args::{CliAction, CheckpointSub, CliOutputFormat, SkillsAction};

/// E-43 -- parse `elai skills <sub>` arguments. Three sub-actions:
///   `elai skills list`
///   `elai skills eval [--skill <name>] [--dry-run]`
///   `elai skills publish --target <claude|codex|gemini> [--dry-run]`
pub(crate) fn parse_skills_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| "missing `skills` sub-action; expected: list, eval, publish".to_string())?
        .as_str();
    match sub {
        "list" => Ok(CliAction::SkillsManagement {
            action: SkillsAction::List,
            output_format,
        }),
        "eval" => {
            let mut skill: Option<String> = None;
            let mut dry_run = false;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--skill" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--skill`".to_string())?;
                        skill = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--skill=") => {
                        skill = Some(flag[8..].to_string());
                        i += 1;
                    }
                    "--dry-run" => {
                        dry_run = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `skills eval` argument: {other}"));
                    }
                }
            }
            Ok(CliAction::SkillsManagement {
                action: SkillsAction::Eval { skill, dry_run },
                output_format,
            })
        }
        "publish" => {
            let mut target: Option<String> = None;
            let mut dry_run = false;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--target" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--target`".to_string())?;
                        target = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--target=") => {
                        target = Some(flag[9..].to_string());
                        i += 1;
                    }
                    "--dry-run" => {
                        dry_run = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `skills publish` argument: {other}"));
                    }
                }
            }
            let target = target.ok_or_else(|| {
                "`skills publish` requires --target <claude|codex|gemini>".to_string()
            })?;
            Ok(CliAction::SkillsManagement {
                action: SkillsAction::Publish { target, dry_run },
                output_format,
            })
        }
        // E-22 (SCHEMA_VERSION 170): export action -- writes a normalised
        // SKILL.md to dist/<target>/skills/<name>/SKILL.md and detects
        // drift via blake3.
        "export" => {
            let mut target: Option<String> = None;
            let mut dry_run = false;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--target" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--target`".to_string())?;
                        target = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--target=") => {
                        target = Some(flag[9..].to_string());
                        i += 1;
                    }
                    "--dry-run" => {
                        dry_run = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `skills export` argument: {other}"));
                    }
                }
            }
            // Default target=claude per Anthropic identity-mapping (the
            // SKILL.md spec is identity for Claude; non-Claude targets
            // need explicit --target to flag the format choice).
            let target = target.unwrap_or_else(|| "claude".to_string());
            Ok(CliAction::SkillsManagement {
                action: SkillsAction::Export { target, dry_run },
                output_format,
            })
        }
        // F-6 / Slice A4 (C6c): learned-skill library surface.
        //   elai skills-mgmt library list
        //   elai skills-mgmt library promote <id> [--human-grant]
        //   elai skills-mgmt library show <id>
        //   elai skills-mgmt library tick
        "library" => parse_skills_library_args(&rest[1..], output_format),
        other => Err(format!(
            "unknown `skills` sub-action `{other}`; expected one of: list, eval, publish, export, library"
        )),
    }
}

/// F-6 / Slice A4 (C6c) -- parse `elai skills-mgmt library <sub>` arguments.
/// Four sub-verbs over the learned-skill trust ladder:
///   `library list`                          enumerate the store
///   `library promote <id> [--human-grant]`  climb one rung (S4 needs grant)
///   `library show <id>`                      print one record
///   `library tick`                           run one staleness-demotion sweep
fn parse_skills_library_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `library` sub-verb; expected: list, promote, show, tick".to_string()
        })?
        .as_str();
    match sub {
        "list" => Ok(CliAction::SkillsManagement {
            action: SkillsAction::LibraryList,
            output_format,
        }),
        "tick" => Ok(CliAction::SkillsManagement {
            action: SkillsAction::LibraryTick,
            output_format,
        }),
        "show" => {
            let id = rest
                .get(1)
                .ok_or_else(|| "`library show` requires a skill id".to_string())?
                .clone();
            Ok(CliAction::SkillsManagement {
                action: SkillsAction::LibraryShow { id },
                output_format,
            })
        }
        "promote" => {
            let id = rest
                .get(1)
                .filter(|s| !s.starts_with("--"))
                .ok_or_else(|| "`library promote` requires a skill id".to_string())?
                .clone();
            let mut human_grant = false;
            let mut i = 2;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--human-grant" => {
                        human_grant = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `library promote` argument: {other}"));
                    }
                }
            }
            Ok(CliAction::SkillsManagement {
                action: SkillsAction::LibraryPromote { id, human_grant },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `library` sub-verb `{other}`; expected one of: list, promote, show, tick"
        )),
    }
}

/// E-9 — parse `elai eval <run|list> [--benchmark=NAME] [--harness-vary]
/// [--p-target=0.05] [--list] [--json]`. The sub-verb is `run` (single
/// suite) or `list` (enumerate registry). `--list` flag on `run` also
/// enumerates without dispatching live. Numerical knobs carried as String
/// because CliAction derives Eq; f64 does not.
pub(crate) fn parse_eval_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    if sub != "run" && sub != "list" {
        return Err(format!(
            "unknown `eval` sub-command: '{sub}' (expected 'run' or 'list')"
        ));
    }
    let flag_start = if sub.is_empty() { 0 } else { 1 };
    let mut benchmark: Option<String> = None;
    let mut harness_vary: bool = false;
    let mut p_target: Option<String> = None;
    let mut list: bool = sub == "list";
    let mut json: bool = false;
    let mut i = flag_start;
    while i < rest.len() {
        match rest[i].as_str() {
            "--benchmark" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --benchmark".to_string())?;
                benchmark = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--benchmark=") => {
                benchmark = Some(flag["--benchmark=".len()..].to_string());
                i += 1;
            }
            "--harness-vary" => {
                harness_vary = true;
                i += 1;
            }
            "--p-target" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --p-target".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --p-target {value:?}: {e}"))?;
                p_target = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--p-target=") => {
                let value = &flag["--p-target=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --p-target {value:?}: {e}"))?;
                p_target = Some(value.to_string());
                i += 1;
            }
            "--list" => {
                list = true;
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `eval` argument: {other}"));
            }
        }
    }
    Ok(CliAction::EvalRun {
        benchmark,
        harness_vary,
        p_target,
        list,
        json,
    })
}

/// E-55 -- parse `elai rl <train|list> [--recipe=<name>] [--list] [--json]`.
/// Recipe runner for the agentic RL substrate. Substrate ships fully;
/// cloud Sonnet rollouts + LoRA training are RUNPOD-PENDING and gated by
/// `ELAI_AGENTIC_RL_ENABLE=1`. Valid recipes are listed in
/// `crates/cli/src/rl_train_cmd.rs::KNOWN_RECIPES`.
pub(crate) fn parse_rl_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    if sub != "train" && sub != "list" {
        return Err(format!(
            "unknown `rl` sub-command: '{sub}' (expected 'train' or 'list')"
        ));
    }
    let flag_start = if sub.is_empty() { 0 } else { 1 };
    let mut recipe: Option<String> = None;
    let mut list: bool = sub == "list";
    let mut json: bool = false;
    let mut i = flag_start;
    while i < rest.len() {
        match rest[i].as_str() {
            "--recipe" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --recipe".to_string())?;
                recipe = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--recipe=") => {
                recipe = Some(flag["--recipe=".len()..].to_string());
                i += 1;
            }
            "--list" => {
                list = true;
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `rl` argument: {other}"));
            }
        }
    }
    Ok(CliAction::RlTrain { recipe, list, json })
}

/// E-25 Tranche C (C.4) -- parse `elai checkpoint <list|rewind>`.
/// Mirrors the `lora-gate-check` / `farm-adv-rm` flat-parser style.
pub(crate) fn parse_checkpoint_subcommand_args(
    rest: &[String],
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `checkpoint` sub-action; expected: list, rewind".to_string()
        })?
        .as_str();
    match sub {
        "list" => {
            let mut session_id: Option<String> = None;
            let mut json = false;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--session-id" => {
                        session_id = Some(
                            rest.get(i + 1)
                                .ok_or_else(|| {
                                    "missing value for --session-id".to_string()
                                })?
                                .clone(),
                        );
                        i += 2;
                    }
                    "--json" => {
                        json = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown flag for `checkpoint list`: {other}"
                        ));
                    }
                }
            }
            Ok(CliAction::CheckpointSub {
                sub: CheckpointSub::List { session_id, json },
            })
        }
        "rewind" => {
            let mut steps: Option<u32> = None;
            let mut session_id: Option<String> = None;
            let mut dry_run = false;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--steps" => {
                        let raw = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for --steps".to_string())?;
                        let n: u32 = raw.parse().map_err(|e| {
                            format!("invalid --steps value `{raw}`: {e}")
                        })?;
                        steps = Some(n);
                        i += 2;
                    }
                    "--session-id" => {
                        session_id = Some(
                            rest.get(i + 1)
                                .ok_or_else(|| {
                                    "missing value for --session-id".to_string()
                                })?
                                .clone(),
                        );
                        i += 2;
                    }
                    "--dry-run" => {
                        dry_run = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown flag for `checkpoint rewind`: {other}"
                        ));
                    }
                }
            }
            let steps = steps.ok_or_else(|| {
                "`checkpoint rewind` requires --steps N".to_string()
            })?;
            Ok(CliAction::CheckpointSub {
                sub: CheckpointSub::Rewind {
                    steps,
                    session_id,
                    dry_run,
                },
            })
        }
        other => Err(format!(
            "unknown `checkpoint` sub-action `{other}`; expected: list, rewind"
        )),
    }
}
