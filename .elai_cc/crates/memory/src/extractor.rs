//! Mem0-style fact extractor (arXiv:2504.19413). The local sidecar
//! produces a JSON envelope `[{"fact_type":..., "key":..., "value":...,
//! "evidence":[...]}, ...]`; this module's responsibility is the deterministic
//! shape — parsing the envelope into [`ExtractedFact`] candidates and the
//! pure-Rust [`extract_facts`] heuristic that runs when no LLM dispatcher
//! is wired (the orchestrator's runtime adapter is the LLM path, kept off
//! limits for this slice).
//!
//! The heuristic is intentionally conservative — `["i always prefer X",
//! "i prefer X over Y", "remember: prefer X"]` patterns become a single
//! `Preference` candidate. It is NOT the production gate — that's reserved
//! for the two-stage defense pipeline (E-5 Task 8) which lands in a
//! follow-up vertical slice.

use serde::{Deserialize, Serialize};

use crate::error::MemoryError;
use crate::fact::FactType;

/// One side of a conversation. Mirrors the chat message shape ELAI's
/// orchestrator already uses — the extractor only needs the role + text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    /// Stable message id (used as an evidence pointer).
    pub message_id: String,
    /// `"user"` or `"assistant"`.
    pub role: String,
    /// UTF-8 text content.
    pub text: String,
}

/// A bounded conversation slice the extractor inspects. Single-turn or
/// multi-turn is fine; the heuristic walks `turns` once and emits all
/// candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Session id (used as an evidence pointer alongside `message_id`).
    pub session_id: String,
    /// One or more turns.
    pub turns: Vec<Turn>,
}

/// A fact candidate emitted by the extractor. The orchestrator's
/// dispatcher (Mem0 ADD/UPDATE/DELETE/NOOP — reserved for follow-up)
/// folds these into [`crate::MemoryFact`] rows via the two-stage gate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedFact {
    /// `fact_type` for the new row.
    pub fact_type: FactType,
    /// Stable, snake_case key (`"prefers_tracing_over_log"`,
    /// `"voice_lowercase_i"`, ...).
    pub key: String,
    /// JSON-shaped payload — the heuristic emits a tiny `{"raw": "<text>",
    /// "matched": "<pattern>"}` so the dashboard can show provenance.
    pub value: serde_json::Value,
    /// Extractor confidence in [0, 1].
    pub confidence: f32,
    /// `[session_id, message_id, ...]` evidence pointers. Non-empty —
    /// empty evidence is rejected upstream by [`crate::MemoryFact::validate`].
    pub evidence: Vec<String>,
}

/// Pure-Rust deterministic heuristic. Walks each user turn and emits one
/// or more [`ExtractedFact`] per regex-like prefix:
///
/// - `"i prefer X"`            -> Preference(key="prefers_X")
/// - `"i always prefer X over Y"` -> Preference(key="prefers_X_over_Y")
/// - `"remember: prefer X"` / `"remember: i prefer X"` -> Preference
/// - `"i hate Y"` / `"i dislike Y"` -> Preference(key="dislikes_Y")
///
/// The function returns `Ok(vec![])` when no pattern matches — empty
/// extractions are normal and not an error.
pub fn extract_facts(conv: &Conversation) -> Result<Vec<ExtractedFact>, MemoryError> {
    if conv.session_id.is_empty() {
        return Err(MemoryError::Invalid("session_id is empty".into()));
    }
    let mut out = Vec::new();
    for turn in &conv.turns {
        if turn.role != "user" {
            continue;
        }
        let evidence = vec![conv.session_id.clone(), turn.message_id.clone()];
        let lower = turn.text.to_ascii_lowercase();

        // Pattern 1: "i prefer X over Y" / "i always prefer X over Y".
        if let Some(pair) = parse_prefer_over(&lower) {
            let (subj, alt) = pair;
            let key = format!("prefers_{}_over_{}", slug(&subj), slug(&alt));
            out.push(ExtractedFact {
                fact_type: FactType::Preference,
                key,
                value: serde_json::json!({
                    "raw": turn.text,
                    "matched": "prefer_over",
                    "subject": subj,
                    "alternative": alt,
                }),
                confidence: 0.85,
                evidence: evidence.clone(),
            });
            continue;
        }

        // Pattern 2: "remember: ..." / "remember that ...".
        if let Some(rest) = strip_prefix_any(&lower, &["remember:", "remember that", "remember,"]) {
            if let Some(pref) = strip_prefix_any(rest, &["i prefer", "prefer"]) {
                let subj = pref.trim();
                if !subj.is_empty() {
                    let key = format!("prefers_{}", slug(subj));
                    out.push(ExtractedFact {
                        fact_type: FactType::Preference,
                        key,
                        value: serde_json::json!({
                            "raw": turn.text,
                            "matched": "remember_prefer",
                            "subject": subj,
                        }),
                        confidence: 0.8,
                        evidence: evidence.clone(),
                    });
                    continue;
                }
            }
        }

        // Pattern 3: "i prefer X" (no "over").
        if let Some(rest) = strip_prefix_any(&lower, &["i always prefer", "i prefer"]) {
            let subj = rest.trim();
            if !subj.is_empty() {
                let key = format!("prefers_{}", slug(subj));
                out.push(ExtractedFact {
                    fact_type: FactType::Preference,
                    key,
                    value: serde_json::json!({
                        "raw": turn.text,
                        "matched": "i_prefer",
                        "subject": subj,
                    }),
                    confidence: 0.7,
                    evidence: evidence.clone(),
                });
                continue;
            }
        }

        // Pattern 4: "i hate Y" / "i dislike Y".
        if let Some(rest) = strip_prefix_any(&lower, &["i hate", "i dislike"]) {
            let subj = rest.trim();
            if !subj.is_empty() {
                let key = format!("dislikes_{}", slug(subj));
                out.push(ExtractedFact {
                    fact_type: FactType::Preference,
                    key,
                    value: serde_json::json!({
                        "raw": turn.text,
                        "matched": "i_dislike",
                        "subject": subj,
                    }),
                    confidence: 0.6,
                    evidence: evidence.clone(),
                });
            }
        }
    }
    Ok(out)
}

fn parse_prefer_over(lower: &str) -> Option<(String, String)> {
    // Strip a leading "i prefer" or "i always prefer".
    let rest = strip_prefix_any(lower, &["i always prefer", "i prefer"])?;
    // Split on " over ".
    let (subj, alt) = rest.split_once(" over ")?;
    let subj = subj.trim();
    let alt = alt.trim();
    if subj.is_empty() || alt.is_empty() {
        return None;
    }
    Some((subj.to_owned(), alt.to_owned()))
}

fn strip_prefix_any<'a>(text: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    for p in prefixes {
        if let Some(rest) = text.strip_prefix(p) {
            return Some(rest.trim_start());
        }
        // Allow leading whitespace before the prefix.
        let trimmed = text.trim_start();
        if let Some(rest) = trimmed.strip_prefix(p) {
            return Some(rest.trim_start());
        }
    }
    None
}

fn slug(s: &str) -> String {
    // Trim trailing punctuation + collapse whitespace into single
    // underscores. Stripped of every non-alphanumeric character.
    let mut out = String::with_capacity(s.len());
    let mut prev_under = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_under = false;
        } else if ch.is_whitespace() {
            if !prev_under && !out.is_empty() {
                out.push('_');
                prev_under = true;
            }
        } else {
            // Strip ',' '.' '!' '?' etc. silently.
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}
