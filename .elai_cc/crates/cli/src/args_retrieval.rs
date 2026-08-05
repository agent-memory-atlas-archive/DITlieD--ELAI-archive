// J-72 slice C — `elai retrieval gold-build` arg parser.
// Grammar:
//   elai retrieval gold-build --out <path>
//       [--rra-dir <dir>] [--history <path>] [--seed-queries <path>]
//       [--max-hard-neg <N>] [--json]

#![allow(dead_code)]

use std::path::PathBuf;

use crate::args::CliAction;

pub(crate) fn parse_retrieval_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let Some(sub) = rest.first().map(String::as_str) else {
        return Err("retrieval: expected sub-command 'gold-build'".to_string());
    };
    if sub != "gold-build" {
        return Err(format!(
            "unknown `retrieval` sub-command: '{sub}' (expected 'gold-build')"
        ));
    }
    let rest = &rest[1..];
    let mut out: Option<PathBuf> = None;
    let mut rra_dir: Option<PathBuf> = None;
    let mut history: Option<PathBuf> = None;
    let mut seed_queries: Option<PathBuf> = None;
    let mut max_hard_neg: Option<usize> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--out" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --out".to_string())?;
                out = Some(PathBuf::from(v));
                i += 2;
            }
            flag if flag.starts_with("--out=") => {
                out = Some(PathBuf::from(&flag["--out=".len()..]));
                i += 1;
            }
            "--rra-dir" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --rra-dir".to_string())?;
                rra_dir = Some(PathBuf::from(v));
                i += 2;
            }
            flag if flag.starts_with("--rra-dir=") => {
                rra_dir = Some(PathBuf::from(&flag["--rra-dir=".len()..]));
                i += 1;
            }
            "--history" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --history".to_string())?;
                history = Some(PathBuf::from(v));
                i += 2;
            }
            flag if flag.starts_with("--history=") => {
                history = Some(PathBuf::from(&flag["--history=".len()..]));
                i += 1;
            }
            "--seed-queries" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --seed-queries".to_string())?;
                seed_queries = Some(PathBuf::from(v));
                i += 2;
            }
            flag if flag.starts_with("--seed-queries=") => {
                seed_queries = Some(PathBuf::from(&flag["--seed-queries=".len()..]));
                i += 1;
            }
            "--max-hard-neg" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --max-hard-neg".to_string())?;
                max_hard_neg = Some(
                    v.parse::<usize>()
                        .map_err(|e| format!("--max-hard-neg must be a non-negative integer: {e}"))?,
                );
                i += 2;
            }
            flag if flag.starts_with("--max-hard-neg=") => {
                max_hard_neg = Some(
                    flag["--max-hard-neg=".len()..]
                        .parse::<usize>()
                        .map_err(|e| format!("--max-hard-neg must be a non-negative integer: {e}"))?,
                );
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => return Err(format!("unknown `retrieval gold-build` argument: {other}")),
        }
    }
    let out = out.ok_or_else(|| "`retrieval gold-build` requires --out <path>".to_string())?;
    if rra_dir.is_none() && history.is_none() && seed_queries.is_none() {
        return Err(
            "`retrieval gold-build` requires at least one of --rra-dir, --history, --seed-queries"
                .to_string(),
        );
    }
    Ok(CliAction::RetrievalGoldBuild {
        out,
        rra_dir,
        history,
        seed_queries,
        max_hard_neg: max_hard_neg.unwrap_or(3),
        json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_happy_path() {
        let rest = args(&[
            "gold-build",
            "--out",
            "gold.jsonl",
            "--history",
            "h.jsonl",
            "--max-hard-neg",
            "5",
            "--json",
        ]);
        let action = parse_retrieval_subcommand_args(&rest).unwrap();
        match action {
            CliAction::RetrievalGoldBuild {
                out,
                history,
                max_hard_neg,
                json,
                ..
            } => {
                assert_eq!(out, PathBuf::from("gold.jsonl"));
                assert_eq!(history, Some(PathBuf::from("h.jsonl")));
                assert_eq!(max_hard_neg, 5);
                assert!(json);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn missing_out_errors() {
        let rest = args(&["gold-build", "--history", "h.jsonl"]);
        let err = parse_retrieval_subcommand_args(&rest).unwrap_err();
        assert!(err.contains("requires --out"), "err={err}");
    }

    #[test]
    fn unknown_sub_errors() {
        let rest = args(&["frobnicate"]);
        let err = parse_retrieval_subcommand_args(&rest).unwrap_err();
        assert!(err.contains("expected 'gold-build'"), "err={err}");
    }
}
