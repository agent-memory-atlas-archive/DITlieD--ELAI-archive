//! C-46 — `elai routing <decisions|pilot|status|export>` handler.
//!
//! Drives the orchestrator's [`elai_orchestrator::routing`] module.
//! Four sub-actions:
//!   - `decisions [--since-ms MS]`: render the most recent
//!     SelectiveDelegationDecision rows from a freshly-run synthetic
//!     pilot harness (no GPU required) so the CLI surface has a
//!     verifiable observable effect even before the live runpod
//!     pilot lands.
//!   - `pilot --task DESC`: run the difficulty classifier +
//!     selective-delegator pilot harness over the 8-base-role pilot
//!     scope and print the chosen `(role, primitive)` pairs.
//!   - `status`: print the resolved RoutingPolicy.
//!   - `export --path PATH`: write the pilot table to a JSON file for
//!     the uno-orchestra-pilot.md artifact.
//!
//! Paper grounding (arXiv:2605.05007, Cui et al 2026, Uno-Orchestra
//! Sections 5 + 6.1): the (model, primitive) action space, the
//! alpha-blend convex combination, the S_max <= 0.10 cost-hack bound,
//! and the direct-answer-cost-zero invariant. The synthetic scoring
//! fixture used by this CLI is a deterministic stand-in for the
//! GPU-billable real pilot; real measurements require the
//! `RUNPOD-PENDING: uno_orchestra_pilot_8roles_qwen3_8b` invocation.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use elai_orchestrator::ipc_types::AgentRole;
use elai_orchestrator::routing::{
    Primitive, RoutingPolicy, RoutingTask, SelectiveDelegator,
    TaskDifficultyClassifier,
};
use serde::Serialize;

use crate::args::{CliOutputFormat, RoutingAction};

/// Top-level dispatcher invoked by main.rs.
pub(crate) fn run(action: RoutingAction, output_format: CliOutputFormat) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("routing_cmd: tokio runtime build failed: {e}"))?;
    runtime.block_on(async {
        match action {
            RoutingAction::Decisions { since_ms } => {
                run_decisions(since_ms, output_format).await
            }
            RoutingAction::Pilot { task_template } => {
                run_pilot(&task_template, output_format).await
            }
            RoutingAction::Status => run_status(output_format),
            RoutingAction::Export { path } => {
                run_export(&PathBuf::from(path), output_format).await
            }
        }
    })
}

/// Default policy used by the CLI surface. Mirrors the
/// `[agent.selective_delegation]` defaults but with `enabled = true`
/// so the synthetic pilot path can demonstrate dispatch. The real
/// production policy comes from the TOML loader.
fn pilot_policy() -> Result<RoutingPolicy, String> {
    RoutingPolicy::new(true, 0.1, 0.10, 8, 8).map_err(|e| format!("pilot policy: {e}"))
}

fn pilot_workers() -> BTreeMap<AgentRole, ()> {
    let roles = [
        AgentRole::Executor,
        AgentRole::Auditor,
        AgentRole::Researcher,
        AgentRole::Designer,
        AgentRole::PromptEngineer,
        AgentRole::ContextCompiler,
        AgentRole::MemoryWriter,
        AgentRole::MediaValidator,
    ];
    roles.iter().map(|r| (*r, ())).collect()
}

/// JSON shape emitted by `decisions` + `export`. Mirrors the SSB
/// `SelectiveDelegationDecisionPayload` field set but renders the
/// dispatched `(role, primitive)` pairs as plaintext for human
/// inspection (the SSB payload only carries hashes).
#[derive(Debug, Serialize)]
struct PilotRow {
    task_id: String,
    difficulty: f32,
    alpha: f32,
    dispatched: Vec<(String, String)>,
    is_lazy_mode: bool,
}

async fn run_decisions(
    _since_ms: Option<u64>,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    // The CLI surface re-runs the synthetic harness so it has data to
    // show even before live SSB rows exist. The `--since-ms` filter
    // becomes meaningful once the production task_runner wires the
    // SelectiveDelegator + SSB emission.
    let policy = pilot_policy()?;
    let delegator = SelectiveDelegator::new(policy);
    let workers = pilot_workers();
    let rows = synthetic_decisions(&delegator, &workers);
    render_rows(&rows, output_format, /* note_synthetic */ true);
    Ok(())
}

async fn run_pilot(task_template: &str, output_format: CliOutputFormat) -> Result<(), String> {
    let policy = pilot_policy()?;
    let delegator = SelectiveDelegator::new(policy);
    let workers = pilot_workers();
    let task = RoutingTask::new_text("cli-pilot", task_template);
    let difficulty = TaskDifficultyClassifier::heuristic_score(&task);
    let dispatched = delegator.select_workers(difficulty, &workers, None);
    let row = PilotRow {
        task_id: task.task_id.clone(),
        difficulty,
        alpha: policy.alpha,
        dispatched: dispatched
            .iter()
            .map(|(r, p)| (role_str(*r), p.as_str().to_string()))
            .collect(),
        is_lazy_mode: dispatched.is_empty() && delegator.is_lazy_mode(difficulty),
    };
    render_rows(&[row], output_format, /* note_synthetic */ true);
    Ok(())
}

fn run_status(output_format: CliOutputFormat) -> Result<(), String> {
    let policy = pilot_policy()?;
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&policy)
                .unwrap_or_else(|_| "{}".into());
            println!("{json}");
        }
        _ => {
            println!("routing policy (NOTE: pilot defaults; production policy comes from elai_cc.toml):");
            println!("  enabled:               {}", policy.enabled);
            println!("  alpha:                 {}", policy.alpha);
            println!("  s_max:                 {}", policy.s_max);
            println!("  max_turns:             {}", policy.max_turns);
            println!("  max_workers_per_query: {}", policy.max_workers_per_query);
        }
    }
    Ok(())
}

async fn run_export(path: &PathBuf, output_format: CliOutputFormat) -> Result<(), String> {
    let policy = pilot_policy()?;
    let delegator = SelectiveDelegator::new(policy);
    let workers = pilot_workers();
    let rows = synthetic_decisions(&delegator, &workers);
    let json = serde_json::to_string_pretty(&rows)
        .map_err(|e| format!("routing export encode failed: {e}"))?;
    fs::write(path, json).map_err(|e| {
        format!("routing export write failed for {}: {e}", path.display())
    })?;
    if matches!(output_format, CliOutputFormat::Json) {
        let summary = serde_json::json!({
            "exported_to": path.display().to_string(),
            "rows": rows.len(),
        });
        println!("{summary}");
    } else {
        println!(
            "routing export wrote {} pilot rows to {}",
            rows.len(),
            path.display()
        );
    }
    Ok(())
}

/// Synthesise a deterministic per-difficulty pilot row sweep. Three
/// representative tasks: trivial (lazy-mode), moderate (subset), hard
/// (full dispatch). Drives the CLI surface so the dashboard +
/// uno-orchestra-pilot.md artifact have shape-correct rows even
/// before the runpod pilot lands.
fn synthetic_decisions(
    delegator: &SelectiveDelegator,
    workers: &BTreeMap<AgentRole, ()>,
) -> Vec<PilotRow> {
    let templates: [(&str, RoutingTask); 3] = [
        (
            "trivial",
            RoutingTask::new_text("synth-trivial", "hi"),
        ),
        (
            "moderate",
            RoutingTask {
                text: "implement a small refactor in the routing module".into(),
                file_count: 2,
                multi_turn: false,
                task_id: "synth-moderate".into(),
            },
        ),
        (
            "hard",
            RoutingTask {
                text: "refactor the orchestrator architecture: investigate why the dispatcher \
                       integrate path fails. ```rust let x = 1; ``` how does it interact with \
                       the auditor design? compare two approaches and evaluate trade-off."
                    .into(),
                file_count: 8,
                multi_turn: true,
                task_id: "synth-hard".into(),
            },
        ),
    ];
    templates
        .iter()
        .map(|(_label, task)| {
            let difficulty = TaskDifficultyClassifier::heuristic_score(task);
            let dispatched = delegator.select_workers(difficulty, workers, None);
            PilotRow {
                task_id: task.task_id.clone(),
                difficulty,
                alpha: delegator.policy.alpha,
                dispatched: dispatched
                    .iter()
                    .map(|(r, p)| (role_str(*r), p.as_str().to_string()))
                    .collect(),
                is_lazy_mode: dispatched.is_empty()
                    && delegator.is_lazy_mode(difficulty),
            }
        })
        .collect()
}

fn role_str(role: AgentRole) -> String {
    serde_json::to_value(role)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| format!("{role:?}").to_ascii_lowercase())
}

fn render_rows(rows: &[PilotRow], output_format: CliOutputFormat, note_synthetic: bool) {
    match output_format {
        CliOutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(rows).unwrap_or_else(|_| "[]".into());
            println!("{json}");
        }
        _ => {
            if note_synthetic {
                println!(
                    "routing decisions (NOTE: synthetic fixture; real decisions via SSB)"
                );
            }
            for row in rows {
                let mode = if row.is_lazy_mode {
                    "lazy-mode (direct answer)"
                } else if row.dispatched.is_empty() {
                    "policy disabled"
                } else {
                    "dispatch"
                };
                println!(
                    "  task={} difficulty={:.3} alpha={:.2} mode={} dispatched={}",
                    row.task_id,
                    row.difficulty,
                    row.alpha,
                    mode,
                    row.dispatched.len(),
                );
                for (role, primitive) in &row.dispatched {
                    println!("    - {role:<20} {primitive}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pilot_policy_passes_paper_invariants() {
        let p = pilot_policy().expect("pilot policy must construct");
        assert!(p.enabled);
        assert!((p.alpha - 0.1).abs() < f32::EPSILON);
        assert!((p.s_max - 0.10).abs() < f32::EPSILON);
    }

    #[test]
    fn pilot_workers_has_eight_base_roles() {
        let w = pilot_workers();
        assert_eq!(w.len(), 8);
        assert!(w.contains_key(&AgentRole::Executor));
        assert!(w.contains_key(&AgentRole::MediaValidator));
    }

    #[test]
    fn synthetic_decisions_yields_three_rows() {
        let policy = pilot_policy().expect("policy");
        let delegator = SelectiveDelegator::new(policy);
        let workers = pilot_workers();
        let rows = synthetic_decisions(&delegator, &workers);
        assert_eq!(rows.len(), 3);
        // The trivial row should be lazy-mode (no dispatch).
        assert!(rows[0].is_lazy_mode || rows[0].dispatched.is_empty());
        // The hard row must dispatch at least one worker.
        assert!(!rows[2].dispatched.is_empty());
    }

    #[test]
    fn role_str_is_snake_case() {
        assert_eq!(role_str(AgentRole::MemoryWriter), "memory_writer");
        assert_eq!(role_str(AgentRole::Executor), "executor");
    }

    #[test]
    fn primitive_as_str_is_snake_case() {
        // Sanity: primitive labels are snake_case for SSB / dashboard.
        assert_eq!(Primitive::ToolCall.as_str(), "tool_call");
        assert_eq!(Primitive::CodeExec.as_str(), "code_exec");
    }
}
