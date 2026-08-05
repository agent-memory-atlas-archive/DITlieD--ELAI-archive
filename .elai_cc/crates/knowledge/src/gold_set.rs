//! J-72 slice C — Tier-1 gold-set assembler pure types + merge/write.
//!
//! Assembles query/positive/hard-negative triples from three optional sources
//! (RRA bundles, issue/PR/edit history JSONL, eval goldset TOML seed queries)
//! into a single JSONL gold set. This module is pure std + serde + toml; it
//! does NOT depend on `elai_orchestrator` (the CLI layer owns RRA loading).

use std::path::Path;

use serde::{Deserialize, Serialize};

/// One query / positive / hard-negative triple for retrieval adaptation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoldTriple {
    pub id: String,
    pub query: String,
    pub positive: String,
    pub hard_negatives: Vec<String>,
    pub source: String,
}

/// Summary counts for a built gold set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoldSetReport {
    pub n_triples: usize,
    pub n_from_rra: usize,
    pub n_from_history: usize,
    pub n_from_seed: usize,
    pub out_path: String,
}

/// Error type for gold-set assembly.
#[derive(Debug, thiserror::Error)]
pub enum GoldSetError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML parse error: {0}")]
    Toml(String),
    #[error("gold-set error: {0}")]
    Other(String),
}

/// Inputs to [`build_gold_set`]: the three optional triple sources + the
/// output path + the cross-session hard-negative cap.
#[derive(Debug, Clone, Default)]
pub struct GoldSetParts {
    pub rra_triples: Vec<GoldTriple>,
    pub history_triples: Vec<GoldTriple>,
    pub seed_triples: Vec<GoldTriple>,
    pub out_path: String,
    pub max_neg: usize,
}

/// Parse a history JSONL string into triples.
///
/// Each line is a JSON object with at least `query` + `positive`; optional
/// `hard_negatives` (array), `id`, `source`. Blank lines are skipped.
pub fn parse_history_jsonl(text: &str) -> Result<Vec<GoldTriple>, GoldSetError> {
    let mut out = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(trimmed).map_err(|e| {
            GoldSetError::Other(format!("history line {}: invalid JSON: {e}", idx + 1))
        })?;
        let obj = v.as_object().ok_or_else(|| {
            GoldSetError::Other(format!("history line {}: not a JSON object", idx + 1))
        })?;
        let query = obj
            .get("query")
            .and_then(|x| x.as_str())
            .ok_or_else(|| {
                GoldSetError::Other(format!(
                    "history line {}: missing required string field 'query'",
                    idx + 1
                ))
            })?;
        let positive = obj
            .get("positive")
            .and_then(|x| x.as_str())
            .ok_or_else(|| {
                GoldSetError::Other(format!(
                    "history line {}: missing required string field 'positive'",
                    idx + 1
                ))
            })?;
        let hard_negatives: Vec<String> = obj
            .get("hard_negatives")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let id = obj
            .get("id")
            .and_then(|x| x.as_str())
            .map(String::from)
            .unwrap_or_else(|| format!("history:{idx}"));
        let source = obj
            .get("source")
            .and_then(|x| x.as_str())
            .map(String::from)
            .unwrap_or_else(|| "history".to_string());
        out.push(GoldTriple {
            id,
            query: query.to_string(),
            positive: positive.to_string(),
            hard_negatives,
            source,
        });
    }
    Ok(out)
}

/// Minimal TOML shape for seed queries: `[[query]]` with `id`, `text`,
/// `expected_paths` (array of strings).
#[derive(Debug, Deserialize)]
struct SeedFile {
    #[serde(default)]
    query: Vec<SeedQuery>,
}

#[derive(Debug, Deserialize)]
struct SeedQuery {
    id: String,
    text: String,
    #[serde(default)]
    expected_paths: Vec<String>,
}

/// Parse a seed queries TOML string (the `eval/rag/queries.toml` shape)
/// into triples. `query=text`, `positive=first expected_paths entry`,
/// `hard_negatives=rest of expected_paths` (distractors), `source=seed:{id}`.
pub fn parse_seed_queries_toml(text: &str) -> Result<Vec<GoldTriple>, GoldSetError> {
    let parsed: SeedFile = toml::from_str(text)
        .map_err(|e| GoldSetError::Toml(format!("seed queries.toml: {e}")))?;
    let mut out = Vec::new();
    for q in parsed.query {
        let positive = q
            .expected_paths
            .first()
            .cloned()
            .unwrap_or_default();
        let hard_negatives: Vec<String> = q
            .expected_paths
            .iter()
            .skip(1)
            .cloned()
            .collect();
        out.push(GoldTriple {
            id: q.id.clone(),
            query: q.text,
            positive,
            hard_negatives,
            source: format!("seed:{}", q.id),
        });
    }
    Ok(out)
}

/// For each triple with empty `hard_negatives`, fill up to `max_neg` from
/// other triples' positives (deterministic order by id).
pub fn assign_cross_session_hard_negatives(triples: &mut [GoldTriple], max_neg: usize) {
    if triples.len() < 2 || max_neg == 0 {
        return;
    }
    // Collect (id, positive) pairs sorted by id for deterministic order.
    let mut pool: Vec<(String, String)> = triples
        .iter()
        .map(|t| (t.id.clone(), t.positive.clone()))
        .collect();
    pool.sort_by(|a, b| a.0.cmp(&b.0));
    for t in triples.iter_mut() {
        if !t.hard_negatives.is_empty() {
            continue;
        }
        let mut filled = 0usize;
        for (other_id, other_positive) in &pool {
            if filled >= max_neg {
                break;
            }
            if other_id == &t.id {
                continue;
            }
            if other_positive.is_empty() || *other_positive == t.positive {
                continue;
            }
            if t.hard_negatives.iter().any(|h| h == other_positive) {
                continue;
            }
            t.hard_negatives.push(other_positive.clone());
            filled += 1;
        }
    }
}

/// Write triples as JSONL (one `GoldTriple` per line) to `path`.
pub fn write_gold_jsonl(path: &Path, triples: &[GoldTriple]) -> Result<(), GoldSetError> {
    let mut buf = String::new();
    for t in triples {
        let line = serde_json::to_string(t)?;
        buf.push_str(&line);
        buf.push('\n');
    }
    std::fs::write(path, buf)?;
    Ok(())
}

/// Build a gold set from the three optional sources, assign cross-session
/// hard negatives, write JSONL to `parts.out_path`, and return the report.
pub fn build_gold_set(
    parts: GoldSetParts,
) -> Result<(Vec<GoldTriple>, GoldSetReport), GoldSetError> {
    let n_from_rra = parts.rra_triples.len();
    let n_from_history = parts.history_triples.len();
    let n_from_seed = parts.seed_triples.len();
    let mut triples = Vec::with_capacity(n_from_rra + n_from_history + n_from_seed);
    triples.extend(parts.rra_triples);
    triples.extend(parts.history_triples);
    triples.extend(parts.seed_triples);
    assign_cross_session_hard_negatives(&mut triples, parts.max_neg);
    let out_path = Path::new(&parts.out_path);
    write_gold_jsonl(out_path, &triples)?;
    let n_triples = triples.len();
    Ok((
        triples,
        GoldSetReport {
            n_triples,
            n_from_rra,
            n_from_history,
            n_from_seed,
            out_path: parts.out_path,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_jsonl_roundtrip_2_lines() {
        let text = r#"{"id":"h1","query":"q1","positive":"p1","hard_negatives":["x"],"source":"hist"}
{"id":"h2","query":"q2","positive":"p2","source":"hist2"}"#;
        let triples = parse_history_jsonl(text).unwrap();
        assert_eq!(triples.len(), 2);
        assert_eq!(triples[0].id, "h1");
        assert_eq!(triples[0].query, "q1");
        assert_eq!(triples[0].positive, "p1");
        assert_eq!(triples[0].hard_negatives, vec!["x".to_string()]);
        assert_eq!(triples[0].source, "hist");
        assert_eq!(triples[1].id, "h2");
        assert!(triples[1].hard_negatives.is_empty());
    }

    #[test]
    fn seed_queries_toml_maps_expected_paths() {
        let text = r#"
[[query]]
id = "q-01"
text = "RuntimeAdapter::stream_step"
expected_paths = ["crates/providers/src/runtime/types.rs", "crates/providers/src/runtime/ollama.rs"]
"#;
        let triples = parse_seed_queries_toml(text).unwrap();
        assert_eq!(triples.len(), 1);
        let t = &triples[0];
        assert_eq!(t.id, "q-01");
        assert_eq!(t.query, "RuntimeAdapter::stream_step");
        assert_eq!(t.positive, "crates/providers/src/runtime/types.rs");
        assert_eq!(t.hard_negatives, vec!["crates/providers/src/runtime/ollama.rs".to_string()]);
        assert_eq!(t.source, "seed:q-01");
    }

    #[test]
    fn cross_session_hard_neg_fills_when_empty() {
        let mut triples = vec![
            GoldTriple { id: "a".into(), query: "qa".into(), positive: "pa".into(), hard_negatives: vec![], source: "s".into() },
            GoldTriple { id: "b".into(), query: "qb".into(), positive: "pb".into(), hard_negatives: vec![], source: "s".into() },
            GoldTriple { id: "c".into(), query: "qc".into(), positive: "pc".into(), hard_negatives: vec![], source: "s".into() },
        ];
        assign_cross_session_hard_negatives(&mut triples, 2);
        assert_eq!(triples[0].hard_negatives.len(), 2);
        assert!(triples[0].hard_negatives.contains(&"pb".to_string()));
        assert!(triples[0].hard_negatives.contains(&"pc".to_string()));
        assert_eq!(triples[1].hard_negatives.len(), 2);
        assert!(triples[1].hard_negatives.contains(&"pa".to_string()));
        assert!(triples[1].hard_negatives.contains(&"pc".to_string()));
    }

    #[test]
    fn write_gold_jsonl_creates_file_with_n_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("gold.jsonl");
        let triples = vec![
            GoldTriple { id: "x".into(), query: "qx".into(), positive: "px".into(), hard_negatives: vec![], source: "t".into() },
            GoldTriple { id: "y".into(), query: "qy".into(), positive: "py".into(), hard_negatives: vec![], source: "t".into() },
        ];
        write_gold_jsonl(&path, &triples).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content.lines().count(), 2);
        let first: serde_json::Value = serde_json::from_str(content.lines().next().unwrap()).unwrap();
        assert_eq!(first["id"], "x");
    }

    #[test]
    fn malformed_history_line_errors() {
        let text = r#"{"id":"h1","query":"q1"}
{"id":"h2","query":"q2","positive":"p2"}"#;
        let err = parse_history_jsonl(text).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing required string field 'positive'"), "msg={msg}");
    }
}
