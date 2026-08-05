//! J-72 slice C — `elai retrieval gold-build` command runner.
//!
//! Assembles query/positive/hard-negative triples from:
//! 1. sealed RRA bundles (`*.rra` under `--rra-dir`) via
//!    `elai_orchestrator::rra::read_rra`
//! 2. optional issue/PR/edit history JSONL (`--history <path>`)
//! 3. optional seed from eval goldset TOML (`--seed-queries <path>`)
//!
//! Output: JSONL gold set to `--out <path>` + optional `--json` summary.

use std::path::{Path, PathBuf};

use elai_knowledge::gold_set::{
    build_gold_set, parse_history_jsonl, parse_seed_queries_toml, GoldSetParts, GoldTriple,
};
use elai_orchestrator::rra::read_rra;

/// Run the `retrieval gold-build` subcommand.
pub(crate) fn run_retrieval_gold_build(
    out: PathBuf,
    rra_dir: Option<PathBuf>,
    history: Option<PathBuf>,
    seed_queries: Option<PathBuf>,
    max_hard_neg: usize,
    json: bool,
) -> anyhow::Result<()> {
    let mut rra_triples = Vec::new();
    if let Some(dir) = &rra_dir {
        rra_triples = load_rra_triples(dir)?;
    }

    let mut history_triples = Vec::new();
    if let Some(path) = &history {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("--history read failed at {}: {e}", path.display()))?;
        history_triples = parse_history_jsonl(&text)
            .map_err(|e| anyhow::anyhow!("--history parse failed: {e}"))?;
    }

    let mut seed_triples = Vec::new();
    if let Some(path) = &seed_queries {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("--seed-queries read failed at {}: {e}", path.display()))?;
        seed_triples = parse_seed_queries_toml(&text)
            .map_err(|e| anyhow::anyhow!("--seed-queries parse failed: {e}"))?;
    }

    if rra_triples.is_empty() && history_triples.is_empty() && seed_triples.is_empty() {
        anyhow::bail!(
            "retrieval gold-build: no triples produced from any source (rra_dir={:?}, history={:?}, seed={:?})",
            rra_dir, history, seed_queries
        );
    }

    let parts = GoldSetParts {
        rra_triples,
        history_triples,
        seed_triples,
        out_path: out.to_string_lossy().to_string(),
        max_neg: max_hard_neg,
    };
    let (triples, report) = build_gold_set(parts)
        .map_err(|e| anyhow::anyhow!("retrieval gold-build: build failed: {e}"))?;

    if json {
        let s = serde_json::to_string_pretty(&report)
            .map_err(|e| anyhow::anyhow!("retrieval gold-build: report serialize: {e}"))?;
        println!("{s}");
    } else {
        println!(
            "retrieval gold-build: {} triples (rra={}, history={}, seed={}) -> {}",
            report.n_triples,
            report.n_from_rra,
            report.n_from_history,
            report.n_from_seed,
            report.out_path,
        );
    }
    let _ = triples;
    Ok(())
}

/// Recursively walk `dir` for `*.rra` files and build one `GoldTriple` per
/// readable bundle. Unreadable RRAs emit a stderr warning and are skipped.
fn load_rra_triples(dir: &Path) -> anyhow::Result<Vec<GoldTriple>> {
    let mut paths = Vec::new();
    collect_rra_files(dir, &mut paths);
    paths.sort();
    let mut triples = Vec::new();
    let mut had_any = false;
    for path in &paths {
        match read_rra(&path) {
            Ok(artifact) => {
                had_any = true;
                let triple = build_triple_from_rra(&path, &artifact);
                triples.push(triple);
            }
            Err(e) => {
                eprintln!(
                    "retrieval gold-build: WARN skipping unreadable RRA {}: {e}",
                    path.display()
                );
            }
        }
    }
    if !had_any {
        // If the dir had .rra files but none were readable, surface that.
        if !paths.is_empty() {
            anyhow::bail!(
                "retrieval gold-build: all {} .rra files under {} were unreadable",
                paths.len(),
                dir.display()
            );
        }
    }
    Ok(triples)
}

fn collect_rra_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rra_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rra") {
            out.push(path);
        }
    }
}

/// Build a best-effort `GoldTriple` from an RRA artifact.
///
/// - id: `rra:{session_id}`
/// - query: best-effort UTF-8 scan of early event payloads for a
///   task/objective-looking string; fallback `session {session_id}` + model_pin
/// - positive: join up to 8 unique file-path-like tokens found in payloads
///   (contains `/` and ends with `.rs`/`.py`/`.ts`/`.md`/`.toml`); if none,
///   use `session_id` as a positive placeholder with a source note
/// - hard_negatives: empty (filled by `assign_cross_session_hard_negatives`)
/// - source: `rra:{file_name}`
fn build_triple_from_rra(path: &Path, artifact: &elai_orchestrator::rra::RunArtifact) -> GoldTriple {
    let session_id = &artifact.session_id;
    let id = format!("rra:{session_id}");
    let source = format!(
        "rra:{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown")
    );

    // Collect UTF-8-ish payload strings + file-path-like tokens.
    let mut path_tokens: Vec<String> = Vec::new();
    let mut query_candidate: Option<String> = None;
    for ev in &artifact.events {
        if path_tokens.len() >= 8 && query_candidate.is_some() {
            break;
        }
        // Best-effort UTF-8 scan (lossy).
        let text = String::from_utf8_lossy(&ev.payload);
        if query_candidate.is_none() {
            // Look for objective/task-looking lines in early events.
            for line in text.lines() {
                let l = line.trim();
                if l.is_empty() {
                    continue;
                }
                // Heuristic: a line that looks like a task/objective description.
                if (l.contains("objective") || l.contains("task") || l.contains("goal"))
                    && l.len() > 10
                    && l.len() < 500
                {
                    query_candidate = Some(l.to_string());
                    break;
                }
            }
        }
        if path_tokens.len() < 8 {
            extract_path_tokens(&text, &mut path_tokens);
        }
    }

    let query = query_candidate.unwrap_or_else(|| {
        if let Some(mp) = &artifact.model_pin {
            format!("session {session_id} (model: {mp})")
        } else {
            format!("session {session_id}")
        }
    });

    let positive = if path_tokens.is_empty() {
        format!("session:{session_id} (no path tokens found)")
    } else {
        path_tokens.join(" | ")
    };

    GoldTriple {
        id,
        query,
        positive,
        hard_negatives: Vec::new(),
        source,
    }
}

/// Extract file-path-like tokens from a text blob: substrings that contain
/// `/` and end with a known source extension. Dedupes into `out`.
fn extract_path_tokens(text: &str, out: &mut Vec<String>) {
    const EXTS: &[&str] = &[".rs", ".py", ".ts", ".md", ".toml"];
    for token in text.split_whitespace() {
        if out.len() >= 8 {
            return;
        }
        if !token.contains('/') {
            continue;
        }
        // Strip trailing punctuation.
        let cleaned = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '_' && c != '-');
        if EXTS.iter().any(|ext| cleaned.ends_with(ext)) {
            if !out.iter().any(|existing| existing == cleaned) {
                out.push(cleaned.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn run_with_history_jsonl_produces_file() {
        let dir = tempfile::tempdir().unwrap();
        let hist = dir.path().join("h.jsonl");
        let mut f = std::fs::File::create(&hist).unwrap();
        writeln!(f, r#"{{"id":"h1","query":"q1","positive":"p1","source":"hist"}}"#).unwrap();
        writeln!(f, r#"{{"id":"h2","query":"q2","positive":"p2","source":"hist"}}"#).unwrap();
        let out = dir.path().join("gold.jsonl");
        run_retrieval_gold_build(out.clone(), None, Some(hist), None, 3, false).unwrap();
        assert!(out.is_file());
        let content = std::fs::read_to_string(&out).unwrap();
        let n = content.lines().filter(|l| !l.is_empty()).count();
        assert!(n >= 1, "expected >=1 lines, got {n}");
    }

    #[test]
    fn run_with_seed_toml_produces_file() {
        let dir = tempfile::tempdir().unwrap();
        let seed = dir.path().join("queries.toml");
        std::fs::write(
            &seed,
            r#"
[[query]]
id = "q-01"
text = "RuntimeAdapter::stream_step"
expected_paths = ["crates/providers/src/runtime/types.rs"]
"#,
        )
        .unwrap();
        let out = dir.path().join("gold.jsonl");
        run_retrieval_gold_build(out.clone(), None, None, Some(seed), 3, false).unwrap();
        assert!(out.is_file());
        let content = std::fs::read_to_string(&out).unwrap();
        let n = content.lines().filter(|l| !l.is_empty()).count();
        assert!(n >= 1, "expected >=1 lines, got {n}");
    }

    #[test]
    fn run_with_all_sources_empty_errors() {
        let dir = tempfile::tempdir().unwrap();
        let empty_rra = dir.path().join("empty_rra");
        std::fs::create_dir_all(&empty_rra).unwrap();
        let out = dir.path().join("gold.jsonl");
        let res = run_retrieval_gold_build(out, Some(empty_rra), None, None, 3, false);
        assert!(res.is_err(), "expected error when no triples produced");
    }
}
