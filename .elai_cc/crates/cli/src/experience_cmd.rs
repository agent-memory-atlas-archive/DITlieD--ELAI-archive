//! C-49 — `elai experience <distill|list|export|reset>` handler.
//!
//! Drives the orchestrator's [`elai_orchestrator::experience`] module.
//! Four sub-actions:
//!   - `distill [--skill NAME]`: re-run the heuristic distiller over
//!     a synthetic in-memory corpus and print the emitted fragments.
//!   - `list`: list every skill with a non-empty trace bin.
//!   - `export --path PATH`: write the per-skill distilled fragments
//!     to a JSON file.
//!   - `reset`: clear the synthetic corpus.
//!
//! Paper grounding caveats (arXiv:2605.14443, Sayana, Todi, Jash 2026):
//!   - This CLI surface ships the HEURISTIC counting-based distiller
//!     only. The RL-trained prompter (Algorithm 1 — GRPO + KL anchor +
//!     feedback model M_FB) is RUNPOD-PENDING.
//!   - The synthetic corpus is per-process in-memory; nothing persists
//!     to disk in this session. A persistent on-disk store would
//!     require wiring a path through the project root, which the C-49
//!     plan defers to follow-on work.
//!   - The paper's headline numbers (Dyck 63 -> 91, Web of Lies 52 ->
//!     90, etc.) are PER-TABLE-ROW and task-specific. Do not cite
//!     "55 -> 90" or "74 -> 91" abstract aggregates as ELAI expected
//!     outcomes.
//!   - Cross-model transfer is poor (paper Section 5.5 Table 4); the
//!     per-skill binning here keeps fragments scoped to one
//!     `skill_name` so the transfer pitfall is structurally avoided.

use std::fs;
use std::path::PathBuf;

use elai_orchestrator::experience::{
    distill_fragments, lookup_distilled_fragments, ExperienceCorpus, PromptFragment, SuccessTrace,
};
use serde::Serialize;

use crate::args::{CliOutputFormat, ExperienceAction};

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(action: ExperienceAction, output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        ExperienceAction::Distill { skill } => run_distill(skill.as_deref(), output_format),
        ExperienceAction::List => run_list(output_format),
        ExperienceAction::Export { path } => run_export(&PathBuf::from(path), output_format),
        ExperienceAction::Reset => run_reset(output_format),
    }
}

/// JSON shape emitted by `distill` + `export`. Mirrors the SSB
/// `PromptFragmentSelectedPayload` field set but renders raw fragment
/// text for human inspection (the SSB payload carries only hashes).
#[derive(Debug, Serialize)]
struct DistilledRow {
    skill: String,
    text: String,
    freq: u32,
    success_rate: f32,
    ts_ms_last_used: u64,
}

/// Build a small synthetic corpus the CLI can demonstrate against
/// without depending on a live ELAI session. Five traces under
/// `planning_workflow`, five under `coding-workflow`, with two
/// commonly-occurring fragments per skill so the distiller has
/// something to rank.
fn synthetic_corpus() -> ExperienceCorpus {
    let mut c = ExperienceCorpus::new();
    let skills = [
        (
            "planning_workflow",
            vec![
                "decompose the task into concept > phase > step > baby step",
                "verify wiring contracts before EXECUTING",
            ],
        ),
        (
            "coding-workflow",
            vec![
                "grep call sites before changing any function signature",
                "use ? for Result propagation, no unwrap in library code",
            ],
        ),
    ];
    let mut ts = 1_700_000_000_000u64;
    for (skill, fragments_text) in skills.iter() {
        for _ in 0..5 {
            ts += 1_000;
            let frags: Vec<PromptFragment> = fragments_text
                .iter()
                .map(|t| PromptFragment::new((*t).to_string(), 1, 1.0, ts))
                .collect();
            let trace = SuccessTrace::new(
                (*skill).into(),
                0xAAAA,
                0xBBBB,
                true,
                ts,
                frags,
            );
            c.record_trace(skill, trace).map_err(|e| {
                eprintln!("synthetic_corpus: record_trace failed: {e}");
            }).ok();
        }
    }
    c
}

fn run_distill(skill: Option<&str>, output_format: CliOutputFormat) -> Result<(), String> {
    let corpus = synthetic_corpus();
    let rows = match skill {
        Some(s) => {
            let frags = lookup_distilled_fragments(s, &corpus);
            project_rows(s, &frags)
        }
        None => corpus
            .skills()
            .flat_map(|(s, _)| {
                let frags = distill_fragments(&corpus, s, 2, 0.0);
                project_rows(s, &frags)
            })
            .collect(),
    };
    render_rows(&rows, output_format);
    Ok(())
}

fn run_list(output_format: CliOutputFormat) -> Result<(), String> {
    let corpus = synthetic_corpus();
    let rows: Vec<SkillRow> = corpus
        .skills()
        .map(|(s, n)| SkillRow {
            skill: s.to_string(),
            trace_count: n,
        })
        .collect();
    match output_format {
        CliOutputFormat::Json => {
            let body = serde_json::to_string_pretty(&rows)
                .map_err(|e| format!("experience list json encode: {e}"))?;
            println!("{body}");
        }
        _ => {
            if rows.is_empty() {
                println!("(no skills in synthetic corpus)");
            } else {
                for r in &rows {
                    println!("{:32} {} traces", r.skill, r.trace_count);
                }
            }
        }
    }
    Ok(())
}

fn run_export(path: &PathBuf, output_format: CliOutputFormat) -> Result<(), String> {
    let corpus = synthetic_corpus();
    let rows: Vec<DistilledRow> = corpus
        .skills()
        .flat_map(|(s, _)| {
            let frags = distill_fragments(&corpus, s, 2, 0.0);
            project_rows(s, &frags)
        })
        .collect();
    let body = serde_json::to_string_pretty(&rows)
        .map_err(|e| format!("experience export json encode: {e}"))?;
    fs::write(path, body).map_err(|e| format!("experience export write {}: {e}", path.display()))?;
    if matches!(output_format, CliOutputFormat::Text) {
        println!("wrote {} rows to {}", rows.len(), path.display());
    }
    Ok(())
}

fn run_reset(output_format: CliOutputFormat) -> Result<(), String> {
    // Synthetic corpus is per-invocation; "reset" is a no-op in this
    // CLI surface (no persistent store yet — see module header for the
    // RUNPOD-PENDING follow-on). Emit a clear message so the operator
    // does not misinterpret silence.
    if matches!(output_format, CliOutputFormat::Text) {
        println!(
            "experience reset: synthetic CLI corpus is per-invocation; \
             nothing to clear (persistent store is RUNPOD-PENDING)"
        );
    } else {
        println!("{{\"reset\":\"noop_synthetic_corpus_is_per_invocation\"}}");
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct SkillRow {
    skill: String,
    trace_count: usize,
}

fn project_rows(skill: &str, fragments: &[PromptFragment]) -> Vec<DistilledRow> {
    fragments
        .iter()
        .map(|f| DistilledRow {
            skill: skill.to_string(),
            text: f.text.clone(),
            freq: f.freq_used,
            success_rate: f.success_rate,
            ts_ms_last_used: f.ts_ms_last_used,
        })
        .collect()
}

fn render_rows(rows: &[DistilledRow], output_format: CliOutputFormat) {
    match output_format {
        CliOutputFormat::Json => {
            let body = serde_json::to_string_pretty(rows)
                .unwrap_or_else(|_| "[]".to_string());
            println!("{body}");
        }
        _ => {
            if rows.is_empty() {
                println!("(no distilled fragments produced)");
            } else {
                for r in rows {
                    println!(
                        "[{}] freq={} success={:.2} ts={} | {}",
                        r.skill, r.freq, r.success_rate, r.ts_ms_last_used, r.text
                    );
                }
            }
        }
    }
}
