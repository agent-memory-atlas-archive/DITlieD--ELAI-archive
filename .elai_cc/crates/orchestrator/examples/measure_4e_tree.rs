// Phase 4E.12 measurement harness: exercises the DecompositionTree +
// DepthFirstNavigator end-to-end against a synthetic ConceptDecomposition and
// (optionally) the running sidecar, then appends a [decomp_tree] section to
// .elai_cc/metrics/phase_4E.toml.
//
// I/O convention: this is a dev-time CLI binary, not library code. Human-
// readable summary lines go to stdout (println!), fallback / warning lines go
// to stderr (eprintln!). The CLAUDE.md tracing-only rule targets library code;
// pulling tracing-subscriber into orchestrator's deps just to silence this
// example would require a dependency bump and still produce stderr output.
//
// Usage:
//   cargo run --example measure_4e_tree -p elai-orchestrator
//   ELAI_SIDECAR_URL=http://127.0.0.1:8100 cargo run --example measure_4e_tree -p elai-orchestrator
//   ELAI_METRICS_PATH=metrics/phase_4E.toml cargo run --example measure_4e_tree ...
//
// Sidecar contract (server.py /v1/messages):
//   request  : {"model": "...", "max_tokens": 1, "messages": [{"role":"user", "content":"..."}]}
//   response : {"usage": {"input_tokens": N, "output_tokens": M}, ...}
//
// If the sidecar is unreachable (connect refused / timeout), this binary logs
// a warning and falls back to a chars/4 token estimate. Exit code is 0 for
// normal completion (including fallback), non-zero only on navigator / TOML
// errors.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use elai_orchestrator::pipeline_tree_navigator::{
    BabyStepOutcome, BabyStepResult, BubbleUpAction, BubbleUpPolicy, DecompositionLevel,
    DepthFirstNavigator, IntegrationRunResult, NavigatorTree, NoOpIntegrationRunner, TreeNavigator,
    TreeNavigatorState,
};
use elai_orchestrator::research_tree::{
    BabyStepGrounding, BabyStepId, BabyStepSpec, ConceptDecomposition, ConceptId, ConceptNode,
    EvidenceLevel, ExecutionMode, ExitCriterion, IntegrationSpec, PhaseId, PhaseNode,
    SourceCitation, SourceQuality, StepId, StepNode,
};
use serde::Serialize;

// --- Named constants (no magic numbers) -------------------------------------

const PHASES_PER_CONCEPT: u64 = 3;
const STEPS_PER_PHASE: u64 = 3;
const BABY_STEPS_PER_STEP: u64 = 3;
const TOTAL_BABY_STEPS: u64 = PHASES_PER_CONCEPT * STEPS_PER_PHASE * BABY_STEPS_PER_STEP;
/// Fraction of baby steps that FAIL once before eventually passing. Drives the
/// bubble_up_rate metric above zero deterministically.
const SEEDED_FAIL_MODULO: u64 = 5;
/// Fraction of baby steps that resolve a cache hit (ResearchCache stub). Drives
/// cache_effectiveness above zero deterministically.
const SEEDED_CACHE_HIT_MODULO: u64 = 3;
/// FSM cycles per baby step (TESTING_FIRST -> EXECUTING -> AUDITING = 3).
const FSM_CYCLES_PER_BABY_STEP: u64 = 3;
const SIDECAR_DEFAULT_URL: &str = "http://127.0.0.1:8100";
const SIDECAR_PATH_MESSAGES: &str = "/v1/messages";
const SIDECAR_CONNECT_TIMEOUT_SECS: u64 = 5;
const SIDECAR_READ_TIMEOUT_SECS: u64 = 30;
const SIDECAR_MAX_TOKENS: u64 = 1;
/// Chars-per-token ratio used for fallback estimation when sidecar is down.
const FALLBACK_CHARS_PER_TOKEN: usize = 4;
const METRICS_PATH_DEFAULT: &str = "metrics/phase_4E.toml";
const METRICS_PATH_ENV: &str = "ELAI_METRICS_PATH";
const SIDECAR_URL_ENV: &str = "ELAI_SIDECAR_URL";
const CONTEXT_TARGET_TOKENS: u64 = 2000;

// --- Errors -----------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
enum MeasureError {
    #[error("toml parse: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("toml serialize: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("navigator: {0}")]
    Navigator(String),
}

// --- Metrics collection -----------------------------------------------------

#[derive(Debug, Default, Clone, Serialize)]
struct DecompTreeMetrics {
    baby_steps_per_idea_avg: f64,
    baby_steps_per_idea_median: f64,
    baby_steps_per_idea_max: u64,
    fsm_cycles_per_idea: u64,
    bubble_up_rate: f64,
    integration_pass_rate: f64,
    traversal_overhead_ratio: f64,
    context_p50: u64,
    context_p95: u64,
    total_execution_time_per_idea_secs: f64,
    cache_effectiveness: f64,
    samples: u64,
    sidecar_reachable: bool,
    generated_at_unix_ms: u64,
    ideas_measured: u64,
    tree_phases: u64,
    tree_steps_per_phase: u64,
    tree_baby_steps_per_step: u64,
}

// --- Synthetic tree construction --------------------------------------------

fn make_baby_step(id: u64, concept_name: &str, phase_name: &str, step_name: &str) -> BabyStepSpec {
    BabyStepSpec {
        id: BabyStepId(id),
        task: format!(
            "Implement baby step {id} for {concept_name}/{phase_name}/{step_name}: \
             wire the new module into the parent and add one assertion."
        ),
        estimated_tokens: 500,
        grounding: BabyStepGrounding {
            concept_name: concept_name.to_string(),
            phase_name: phase_name.to_string(),
            step_name: step_name.to_string(),
            top_citations: vec![SourceCitation {
                url: "https://doc.rust-lang.org/book".to_string(),
                title: "Rust Book".to_string(),
                quality: SourceQuality::Tier1,
                evidence_level: EvidenceLevel::Verified,
                excerpt: "Rust ownership model guarantees memory safety.".to_string(),
            }],
            compressed_tokens: 200,
        },
        tdd: None,
        acceptance: None,
        preferred_mode: None,
        target_file: None,
        edit_region: None,
    }
}

fn make_step(id: u64, concept: &str, phase: &str, baby_start_id: u64) -> StepNode {
    let name = format!("step_{id}");
    let baby_steps = (0..BABY_STEPS_PER_STEP)
        .map(|i| make_baby_step(baby_start_id + i, concept, phase, &name))
        .collect();
    StepNode {
        id: StepId(id),
        description: format!("{name}: {concept} / {phase}"),
        baby_steps,
        evidence_level: EvidenceLevel::MultiSource,
    }
}

fn make_phase(id: u64, concept: &str, step_start_id: u64, baby_start_id: u64) -> PhaseNode {
    let name = format!("phase_{id}");
    let mut steps = Vec::with_capacity(usize::try_from(STEPS_PER_PHASE).unwrap_or(3));
    let mut bs_cursor = baby_start_id;
    for i in 0..STEPS_PER_PHASE {
        steps.push(make_step(step_start_id + i, concept, &name, bs_cursor));
        bs_cursor += BABY_STEPS_PER_STEP;
    }
    PhaseNode {
        id: PhaseId(id),
        name,
        steps,
        depends_on: Vec::new(),
        research: None,
        is_parallel: false,
    }
}

fn build_synthetic_tree() -> NavigatorTree {
    let concept_name = "synthetic_concept";
    let mut phases = Vec::with_capacity(usize::try_from(PHASES_PER_CONCEPT).unwrap_or(3));
    let mut step_cursor = 1u64;
    let mut bs_cursor = 1u64;
    for p in 0..PHASES_PER_CONCEPT {
        phases.push(make_phase(p + 1, concept_name, step_cursor, bs_cursor));
        step_cursor += STEPS_PER_PHASE;
        bs_cursor += STEPS_PER_PHASE * BABY_STEPS_PER_STEP;
    }

    let concept = ConceptNode {
        id: ConceptId(1),
        name: concept_name.to_string(),
        description: "Synthetic concept for 4E.12 measurement.".to_string(),
        phases,
        research: None,
    };
    let decomp = ConceptDecomposition {
        idea: "Measure the decomposition tree end-to-end".to_string(),
        concepts: vec![concept],
        mode: ExecutionMode::Autonomous,
        total_research_calls: 0,
    };
    let mut tree = NavigatorTree::new(decomp, BubbleUpPolicy::default());

    // Register integration specs at all three aggregate levels so the run
    // exercises the 4E.11 execution path end-to-end.
    let step_ids: Vec<StepId> = tree
        .location_index
        .values()
        .map(|loc| loc.step_id)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for sid in &step_ids {
        tree.register_integration_spec(
            DecompositionLevel::Step,
            sid.0,
            synthetic_spec("step_module", "cargo check -p elai-orchestrator"),
        );
    }
    let phase_ids: Vec<PhaseId> = tree
        .location_index
        .values()
        .map(|loc| loc.phase_id)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for pid in &phase_ids {
        tree.register_integration_spec(
            DecompositionLevel::Phase,
            pid.0,
            synthetic_spec(
                "phase_integration",
                "cargo test -p elai-orchestrator --lib -- pipeline_tree_navigator",
            ),
        );
    }
    tree.register_integration_spec(
        DecompositionLevel::Concept,
        1,
        synthetic_spec("concept_e2e", "cargo check --workspace"),
    );
    tree
}

fn synthetic_spec(wiring: &str, command: &str) -> IntegrationSpec {
    IntegrationSpec {
        wiring_target: wiring.to_string(),
        callers: vec!["measure_4e_tree".to_string()],
        exit_criteria: vec![ExitCriterion {
            description: format!("{wiring} wired"),
            check_command: Some(command.to_string()),
        }],
    }
}

// --- Seeded outcome generator (deterministic) -------------------------------

fn seeded_outcome(step_id: u64, attempt: u32) -> BabyStepOutcome {
    if attempt == 0 && step_id % SEEDED_FAIL_MODULO == 0 {
        BabyStepOutcome::Failed {
            reason: format!("seeded transient failure on first attempt for bs {step_id}"),
        }
    } else {
        BabyStepOutcome::Passed
    }
}

fn seeded_cache_hit(step_id: u64) -> bool {
    step_id % SEEDED_CACHE_HIT_MODULO == 0
}

// --- Sidecar client ---------------------------------------------------------

struct SidecarClient {
    base_url: String,
    http: reqwest::Client,
}

impl SidecarClient {
    fn new(base_url: String) -> Result<Self, MeasureError> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(SIDECAR_CONNECT_TIMEOUT_SECS))
            .timeout(Duration::from_secs(SIDECAR_READ_TIMEOUT_SECS))
            .build()
            .map_err(|e| MeasureError::Navigator(format!("reqwest build: {e}")))?;
        Ok(Self { base_url, http })
    }

    async fn count_prompt_tokens(&self, prompt: &str) -> Result<u64, reqwest::Error> {
        let url = format!("{}{}", self.base_url, SIDECAR_PATH_MESSAGES);
        let body = serde_json::json!({
            "model": "elai-local",
            "max_tokens": SIDECAR_MAX_TOKENS,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });
        let resp = self.http.post(&url).json(&body).send().await?;
        let json: serde_json::Value = resp.json().await?;
        let tokens = json
            .get("usage")
            .and_then(|u| u.get("input_tokens"))
            .and_then(|t| t.as_u64())
            .unwrap_or_else(|| estimate_tokens(prompt));
        Ok(tokens)
    }
}

fn estimate_tokens(text: &str) -> u64 {
    let chars = text.chars().count();
    (chars / FALLBACK_CHARS_PER_TOKEN) as u64
}

fn build_baby_step_prompt(spec: &BabyStepSpec) -> String {
    format!(
        "You are implementing baby step {bs_id}.\n\n\
         Concept: {concept}\n\
         Phase: {phase}\n\
         Step: {step}\n\n\
         Task:\n{task}\n\n\
         Grounding (top citation):\n  - {citation_title} (quality: {quality:?}) — {excerpt}\n\n\
         Budget: {tokens} tokens.\n\n\
         Return the code diff. Do not include commentary.",
        bs_id = spec.id.0,
        concept = spec.grounding.concept_name,
        phase = spec.grounding.phase_name,
        step = spec.grounding.step_name,
        task = spec.task,
        citation_title = spec
            .grounding
            .top_citations
            .first()
            .map(|c| c.title.as_str())
            .unwrap_or("(none)"),
        quality = spec
            .grounding
            .top_citations
            .first()
            .map(|c| c.quality)
            .unwrap_or(SourceQuality::Tier5),
        excerpt = spec
            .grounding
            .top_citations
            .first()
            .map(|c| c.excerpt.as_str())
            .unwrap_or(""),
        tokens = spec.estimated_tokens,
    )
}

// --- Measurement driver -----------------------------------------------------

struct RunOutcome {
    baby_step_count: u64,
    bubble_ups_triggered: u64,
    integration_results: Vec<IntegrationRunResult>,
    context_tokens: Vec<u64>,
    navigator_wall_ns: u128,
    agent_wall_ns: u128,
    cache_hits: u64,
    sidecar_reachable: bool,
}

async fn run_one_iteration(
    tree: &NavigatorTree,
    sidecar: &SidecarClient,
    sidecar_reachable_input: bool,
) -> Result<RunOutcome, MeasureError> {
    let nav = DepthFirstNavigator::from_tree(tree);
    let mut state = TreeNavigatorState::new();
    let runner = NoOpIntegrationRunner;

    let mut navigator_ns: u128 = 0;
    let mut agent_ns: u128 = 0;
    let mut bubble_ups = 0u64;
    let mut integration_results: Vec<IntegrationRunResult> = Vec::new();
    let mut context_tokens: Vec<u64> = Vec::new();
    let mut cache_hits: u64 = 0;
    let mut sidecar_reachable = sidecar_reachable_input;

    // Track already-failed-once ids so seeded_outcome(attempt=1) returns Pass.
    let mut retried: std::collections::BTreeSet<BabyStepId> = std::collections::BTreeSet::new();
    let mut last_completed_step: Option<StepId> = None;
    let mut last_completed_phase: Option<PhaseId> = None;

    loop {
        let nav_t0 = Instant::now();
        let next = nav.next_baby_step(&state);
        navigator_ns += nav_t0.elapsed().as_nanos();
        let bs_id = match next {
            Some(id) => id,
            None => break,
        };

        let spec = tree
            .find_baby_step(bs_id)
            .ok_or_else(|| MeasureError::Navigator(format!("baby step {} missing", bs_id.0)))?
            .clone();

        // Agent work: build prompt + count tokens via sidecar (falls back to estimate).
        let agent_t0 = Instant::now();
        let prompt = build_baby_step_prompt(&spec);
        let tokens = if sidecar_reachable {
            match sidecar.count_prompt_tokens(&prompt).await {
                Ok(n) => n,
                Err(e) => {
                    eprintln!(
                        "[measure_4e_tree] WARN sidecar call failed ({e}); \
                         falling back for remaining baby steps"
                    );
                    sidecar_reachable = false;
                    estimate_tokens(&prompt)
                }
            }
        } else {
            estimate_tokens(&prompt)
        };
        context_tokens.push(tokens);
        if seeded_cache_hit(bs_id.0) {
            cache_hits += 1;
        }
        agent_ns += agent_t0.elapsed().as_nanos();

        // Record outcome with seeded attempt pattern.
        let attempt = if retried.contains(&bs_id) { 1 } else { 0 };
        let outcome = seeded_outcome(bs_id.0, attempt);
        let is_fail = matches!(outcome, BabyStepOutcome::Failed { .. });
        let result = BabyStepResult {
            baby_step_id: bs_id,
            outcome,
            lines_modified: 40,
            retry_count: attempt,
        };
        let nav_t1 = Instant::now();
        let action = nav.record_result(result, &mut state, tree);
        navigator_ns += nav_t1.elapsed().as_nanos();
        if action.is_some() {
            bubble_ups += 1;
        }
        if is_fail {
            retried.insert(bs_id);
        }

        // After each baby step, check if its parent step is now terminal and
        // fire the step-level integration. Then phase, then concept.
        let loc = match tree.location_index.get(&bs_id) {
            Some(l) => l.clone(),
            None => continue,
        };

        let nav_t2 = Instant::now();
        if nav.step_is_terminal(loc.step_id, &state, tree)
            && last_completed_step != Some(loc.step_id)
        {
            let (ir, action) =
                nav.run_step_integration(loc.step_id, &runner, &mut state, tree);
            if let Some(res) = ir {
                integration_results.push(res);
            }
            if matches!(action, Some(BubbleUpAction::RedecomposeStep { .. })) {
                bubble_ups += 1;
            }
            last_completed_step = Some(loc.step_id);
        }
        if nav.phase_is_terminal(loc.phase_id, &state, tree)
            && last_completed_phase != Some(loc.phase_id)
        {
            let (ir, action) =
                nav.run_phase_integration(loc.phase_id, &runner, &mut state, tree);
            if let Some(res) = ir {
                integration_results.push(res);
            }
            if matches!(action, Some(BubbleUpAction::ReplanPhase { .. })) {
                bubble_ups += 1;
            }
            last_completed_phase = Some(loc.phase_id);
        }
        navigator_ns += nav_t2.elapsed().as_nanos();
    }

    // Concept-level integration once all phases are terminal.
    if nav.concept_is_terminal(ConceptId(1), &state, tree) {
        let nav_t3 = Instant::now();
        let (ir, _action) =
            nav.run_concept_integration(ConceptId(1), &runner, &mut state, tree);
        if let Some(res) = ir {
            integration_results.push(res);
        }
        navigator_ns += nav_t3.elapsed().as_nanos();
    }

    let baby_step_count = tree.traversal_order.len() as u64;
    Ok(RunOutcome {
        baby_step_count,
        bubble_ups_triggered: bubble_ups,
        integration_results,
        context_tokens,
        navigator_wall_ns: navigator_ns,
        agent_wall_ns: agent_ns,
        cache_hits,
        sidecar_reachable,
    })
}

fn percentile(sorted: &[u64], pct: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * pct).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn assemble_metrics(
    outcome: &RunOutcome,
    wall_secs: f64,
    sidecar_reachable: bool,
) -> DecompTreeMetrics {
    let mut sorted_tokens = outcome.context_tokens.clone();
    sorted_tokens.sort_unstable();

    let integration_pass_count = outcome
        .integration_results
        .iter()
        .filter(|r| r.success)
        .count() as f64;
    let integration_total = outcome.integration_results.len().max(1) as f64;
    let integration_pass_rate = integration_pass_count / integration_total;

    let total_ns = (outcome.navigator_wall_ns + outcome.agent_wall_ns).max(1) as f64;
    let traversal_overhead_ratio = outcome.navigator_wall_ns as f64 / total_ns;

    let baby_steps = outcome.baby_step_count.max(1) as f64;
    let bubble_up_rate = outcome.bubble_ups_triggered as f64 / baby_steps;
    let cache_effectiveness = outcome.cache_hits as f64 / baby_steps;

    DecompTreeMetrics {
        baby_steps_per_idea_avg: outcome.baby_step_count as f64,
        baby_steps_per_idea_median: outcome.baby_step_count as f64,
        baby_steps_per_idea_max: outcome.baby_step_count,
        fsm_cycles_per_idea: outcome.baby_step_count * FSM_CYCLES_PER_BABY_STEP,
        bubble_up_rate,
        integration_pass_rate,
        traversal_overhead_ratio,
        context_p50: percentile(&sorted_tokens, 0.50),
        context_p95: percentile(&sorted_tokens, 0.95),
        total_execution_time_per_idea_secs: wall_secs,
        cache_effectiveness,
        samples: outcome.context_tokens.len() as u64,
        sidecar_reachable,
        generated_at_unix_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0),
        ideas_measured: 1,
        tree_phases: PHASES_PER_CONCEPT,
        tree_steps_per_phase: STEPS_PER_PHASE,
        tree_baby_steps_per_step: BABY_STEPS_PER_STEP,
    }
}

// --- TOML append (preserves existing sections) ------------------------------

fn append_metrics_to_toml(path: &Path, metrics: &DecompTreeMetrics) -> Result<(), MeasureError> {
    let existing = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(MeasureError::Io(e)),
    };

    let mut doc: toml::Table = if existing.trim().is_empty() {
        toml::Table::new()
    } else {
        toml::from_str(&existing)?
    };

    let section = toml::Value::try_from(metrics)?;
    doc.insert("decomp_tree".to_string(), section);

    let serialized = toml::to_string_pretty(&doc)?;
    std::fs::write(path, serialized)?;
    Ok(())
}

// --- Entry point ------------------------------------------------------------

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), MeasureError> {
    let sidecar_url =
        std::env::var(SIDECAR_URL_ENV).unwrap_or_else(|_| SIDECAR_DEFAULT_URL.to_string());
    let metrics_path: PathBuf = std::env::var(METRICS_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(METRICS_PATH_DEFAULT));

    let client = SidecarClient::new(sidecar_url.clone())?;
    let sidecar_reachable = probe_sidecar(&client).await;
    if sidecar_reachable {
        eprintln!("[measure_4e_tree] sidecar reachable at {sidecar_url}");
    } else {
        eprintln!(
            "[measure_4e_tree] WARN sidecar unreachable at {sidecar_url} — \
             using chars/{FALLBACK_CHARS_PER_TOKEN} fallback for token counts"
        );
    }

    let tree = build_synthetic_tree();
    if tree.traversal_order.len() as u64 != TOTAL_BABY_STEPS {
        return Err(MeasureError::Navigator(format!(
            "expected {} baby steps in synthetic tree, got {}",
            TOTAL_BABY_STEPS,
            tree.traversal_order.len()
        )));
    }

    let wall_t0 = Instant::now();
    let outcome = run_one_iteration(&tree, &client, sidecar_reachable).await?;
    let wall_secs = wall_t0.elapsed().as_secs_f64();

    // The sidecar may have become unreachable mid-run; reflect that in output.
    let effective_reachable = outcome.sidecar_reachable;
    let metrics = assemble_metrics(&outcome, wall_secs, effective_reachable);
    append_metrics_to_toml(&metrics_path, &metrics)?;

    print_summary(&metrics, &sidecar_url, &metrics_path);
    diagnostics(&outcome);
    Ok(())
}

async fn probe_sidecar(client: &SidecarClient) -> bool {
    let url = format!("{}/health", client.base_url);
    // /health is a GET returning 200 when the engine is loaded. A short timeout
    // wraps the connect+read so an unreachable sidecar fails fast.
    match tokio::time::timeout(
        Duration::from_secs(SIDECAR_CONNECT_TIMEOUT_SECS),
        client.http.get(&url).send(),
    )
    .await
    {
        Ok(Ok(resp)) => resp.status().is_success(),
        Ok(Err(_)) | Err(_) => false,
    }
}

fn print_summary(metrics: &DecompTreeMetrics, sidecar_url: &str, metrics_path: &Path) {
    println!("=== 4E.12 decomp-tree measurement ===");
    println!("sidecar_url          : {sidecar_url}");
    println!("sidecar_reachable    : {}", metrics.sidecar_reachable);
    println!("baby_steps_per_idea  : {}", metrics.baby_steps_per_idea_avg);
    println!("fsm_cycles_per_idea  : {}", metrics.fsm_cycles_per_idea);
    println!("bubble_up_rate       : {:.3}", metrics.bubble_up_rate);
    println!(
        "integration_pass     : {:.3}",
        metrics.integration_pass_rate
    );
    println!(
        "traversal_overhead   : {:.3}",
        metrics.traversal_overhead_ratio
    );
    println!(
        "context p50 / p95    : {} / {} (target <{})",
        metrics.context_p50, metrics.context_p95, CONTEXT_TARGET_TOKENS
    );
    println!(
        "total_time_per_idea  : {:.3}s",
        metrics.total_execution_time_per_idea_secs
    );
    println!(
        "cache_effectiveness  : {:.3}",
        metrics.cache_effectiveness
    );
    println!("metrics written to   : {}", metrics_path.display());
}

fn diagnostics(outcome: &RunOutcome) {
    let step_cp = outcome
        .integration_results
        .iter()
        .filter(|r| matches!(r.level, DecompositionLevel::Step))
        .count();
    let phase_cp = outcome
        .integration_results
        .iter()
        .filter(|r| matches!(r.level, DecompositionLevel::Phase))
        .count();
    let concept_cp = outcome
        .integration_results
        .iter()
        .filter(|r| matches!(r.level, DecompositionLevel::Concept))
        .count();
    println!(
        "integration checkpoints (step/phase/concept): {step_cp} / {phase_cp} / {concept_cp}"
    );
    println!("bubble_ups_triggered  : {}", outcome.bubble_ups_triggered);
    println!("context_samples       : {}", outcome.context_tokens.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_tree_has_expected_baby_step_count() {
        let tree = build_synthetic_tree();
        assert_eq!(tree.traversal_order.len() as u64, TOTAL_BABY_STEPS);
        let step_ids: std::collections::BTreeSet<StepId> = tree
            .location_index
            .values()
            .map(|loc| loc.step_id)
            .collect();
        assert_eq!(step_ids.len() as u64, PHASES_PER_CONCEPT * STEPS_PER_PHASE);
    }

    #[test]
    fn synthetic_tree_registers_specs_at_all_aggregate_levels() {
        let tree = build_synthetic_tree();
        let step_specs = tree
            .integration_specs
            .keys()
            .filter(|(l, _)| *l == DecompositionLevel::Step)
            .count();
        let phase_specs = tree
            .integration_specs
            .keys()
            .filter(|(l, _)| *l == DecompositionLevel::Phase)
            .count();
        let concept_specs = tree
            .integration_specs
            .keys()
            .filter(|(l, _)| *l == DecompositionLevel::Concept)
            .count();
        assert_eq!(step_specs as u64, PHASES_PER_CONCEPT * STEPS_PER_PHASE);
        assert_eq!(phase_specs as u64, PHASES_PER_CONCEPT);
        assert_eq!(concept_specs, 1);
    }

    #[test]
    fn percentile_returns_bounds() {
        let v = vec![10u64, 20, 30, 40, 50];
        assert_eq!(percentile(&v, 0.0), 10);
        assert_eq!(percentile(&v, 0.5), 30);
        assert_eq!(percentile(&v, 1.0), 50);
    }

    #[test]
    fn fallback_token_estimate_matches_ratio() {
        let s = "a".repeat(FALLBACK_CHARS_PER_TOKEN * 25);
        assert_eq!(estimate_tokens(&s), 25);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_one_iteration_without_sidecar_completes() {
        let tree = build_synthetic_tree();
        let client = SidecarClient::new("http://127.0.0.1:1".to_string())
            .expect("client construction");
        let outcome = run_one_iteration(&tree, &client, false).await.expect("run");
        assert_eq!(outcome.baby_step_count, TOTAL_BABY_STEPS);
        // context_tokens grows by one per execution, and seeded_outcome retries
        // baby steps whose id is divisible by SEEDED_FAIL_MODULO. So we expect
        // at least TOTAL_BABY_STEPS samples and strictly more than that when
        // retries fired.
        assert!(
            outcome.context_tokens.len() as u64 >= TOTAL_BABY_STEPS,
            "expected >= {TOTAL_BABY_STEPS} context samples, got {}",
            outcome.context_tokens.len()
        );
        let expected_retries = (1..=TOTAL_BABY_STEPS)
            .filter(|id| id % SEEDED_FAIL_MODULO == 0)
            .count() as u64;
        assert_eq!(
            outcome.context_tokens.len() as u64,
            TOTAL_BABY_STEPS + expected_retries
        );
        assert!(!outcome.sidecar_reachable);
    }

    #[test]
    fn assemble_metrics_populates_required_keys() {
        let tree = build_synthetic_tree();
        let outcome = RunOutcome {
            baby_step_count: tree.traversal_order.len() as u64,
            bubble_ups_triggered: 2,
            integration_results: Vec::new(),
            context_tokens: (1..=20).collect(),
            navigator_wall_ns: 1000,
            agent_wall_ns: 9000,
            cache_hits: 3,
            sidecar_reachable: false,
        };
        let m = assemble_metrics(&outcome, 1.5, false);
        assert!(m.bubble_up_rate > 0.0);
        assert!(m.context_p50 >= 1 && m.context_p50 <= 20);
        assert!(m.context_p95 >= m.context_p50);
        assert_eq!(m.fsm_cycles_per_idea, m.baby_steps_per_idea_max * FSM_CYCLES_PER_BABY_STEP);
    }
}
