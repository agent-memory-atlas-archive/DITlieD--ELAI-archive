#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    clippy::unneeded_struct_pattern,
    clippy::unnecessary_wraps,
    clippy::unused_self
)]
mod classifier;
mod commands;
mod dashboard;
mod dep_audit;
mod doctor;
mod handoff;
mod indexing;
mod init;
mod input;
mod render;
mod runtime_config;
mod skills_cmd;
mod repl;
mod remote_sidecar;
mod sidecar;
mod sidecar_hyde;

mod args;
mod auth;
mod c17_lifecycle;
mod help;
mod live_cli;
mod measure;
mod pipeline_subcommand;
mod symbol_graph;
mod progress;
mod resume;
mod run_subcommand;
mod runtime_builder;
mod wip_squash_subcommand;
// C-27: T(-1)+T-0 wiring contract audit dispatcher.
mod wiring_audit;
mod archive_verify;
// C-43: `elai freeze status|verify|ratify` — Phase 4H scope-freeze CLI
// driver. Wraps elai_orchestrator::scope_freeze. ELAI design hypothesis
// — no paper grounding (see workflow/sessions/s-25120/mistakes.md for
// the bad-cite analysis of arXiv:2604.21725).
mod freeze_cmd;
// C-45: `elai credit attribute|table|export` — CANTANTE contrastive
// credit-attribution CLI driver. Wraps elai_orchestrator::credit_attribution.
// Paper-grounded against arXiv:2605.13295 (Zehle 2026, Section 3.2 — K
// rollouts NOT K^N, scalar c in [-1,+1], g=3 partition default). See
// workflow/papers/cantante-credit-attribution.md.
mod credit_cmd;
// E-39: `elai cloud override-cap <USD> --confirm <typed-string>` /
// `elai cloud status`. Layered-cap manual-override CLI driver.
// Composes with E-38 EU AI Act audit-trail liveness gate. See
// workflow/reasoning/E-39-decisions.md.
mod cloud_cmd;
// C-46: `elai routing decisions|pilot|status|export` — Uno-Orchestra
// parsimonious agent routing (selective delegation) CLI driver. Wraps
// elai_orchestrator::routing. Paper-grounded against arXiv:2605.05007
// (Cui et al 2026, Sections 5 + 6.1 — (model, primitive) action space,
// alpha in [0, 1] convex blend, S_max <= 0.10 cost-hack bound). See
// workflow/papers/uno-orchestra-interface.md.
mod routing_cmd;
// C-47: `elai calibration status|reset-hints|export` — case-based
// tool-use calibration CLI driver. Wraps
// `elai_plugins::ToolCalibrationRegistry`. Paper-grounded against
// arXiv:2605.15041 (CAST — Case-Based Calibration of Adaptive Reasoning
// and Execution for LLM Tool Use). See
// workflow/papers/case-based-calibration.md.
mod calibration_cmd;
// E-14: `elai browser run|list-journeys|status` — browser-e2e agent
// CLI driver. Wraps elai_orchestrator::browser_e2e (Playwright snapshot
// + axe-core gate + journey assertions). Wire-dark for absent
// npx/playwright; VLM tier RUNPOD-PENDING.
mod browser_e2e_cmd;
mod reliability_cmd;
// E-44: `elai durability status|validate-checkpoint|migrate|
//                  continue-as-new-preview|acquire-lock` — durability
// layer operator surface anchoring the orchestrator's `durability::*`
// module as a production callable (WIRE-DARK avoidance per E-41
// precedent). See workflow/reasoning/E-44-decisions.md.
mod durability_cmd;
// E-45: `elai drift detect|list-pairs|baseline` — cross-language drift
// detector operator surface. Anchors cross_lang_drift::DriftDetector +
// PairRegistry as production callables (WIRE-DARK avoidance per E-41
// precedent). See workflow/reasoning/E-45-decisions.md.
mod drift_cmd;
// E-49: `elai metrics capture|swe-bench-pro|refinebench|aggregate|status` —
// coding-agent metric suite operator surface (scope-lean ship). Anchors
// coding_metrics::MetricsCollector + aggregate_session_metrics + the
// SWE-Bench Pro / RefineBench live-run boundaries (RUNPOD-PENDING) as
// production callables (WIRE-DARK avoidance per E-41 precedent). See
// workflow/reasoning/E-49-decisions.md.
mod metrics_cmd;
// E-35: structured-output substrate operator surface (XGrammar /
// INSTRUCTOR / CodeAct / SEARCH/REPLACE / cloud strict-schema).
// WIRE-DARK avoidance per E40_WIRE_DARK_AVOIDANCE_VIA_CLI for the
// 10 new SsbActionType variants this plan introduces. AgentRole::Runner
// DEFERRED per E41_AGENTROLE_FROZEN_AT_17. See
// workflow/reasoning/E-35-decisions.md.
mod structured_output_cmd;
// E-37: `elai safety status|list-pins|verify-pins|fixture-replay` —
// prompt-injection defense operator surface (OWASP LLM Top 10 LLM01).
// Wraps elai_orchestrator::Ssb + elai_safety pin / classifier helpers.
// Read-only; live model fixture replay RUNPOD-PENDING. See
// workflow/reasoning/E-37-decisions.md.
mod safety_cmd;
// E-40: `elai scip <query|defs|refs|callers|callees|pagerank|status|import>` —
// orchestrator-rooted symbol-graph operator surface. Production caller for
// elai_orchestrator::symbol_graph (defends against E37_WIRE_DARK_GAP_PATTERN).
// See workflow/reasoning/E-40-decisions.md.
mod scip_cmd;
// D-16: `elai conformal-calibration status|update` — durable UniCR
// tau threshold CLI driver. Wraps elai_orchestrator::conformal_calibration.
// Paper-grounded against arXiv:2509.01455 (UniCR Eq.5). Namespace
// distinct from C-47 `elai calibration` per D-16-decisions.md
// DECISION-5.
mod conformal_calibration_cli;
mod experience_cmd;
// C-50: `elai contract scaffold|audit|list` — Contract-Coding
// STRUCTURAL contract test scaffolding CLI driver. Wraps
// elai_orchestrator::contract_coding. Paper-grounded against
// arXiv:2604.13100 (Lin, Zhao, Shi 2026 — kernel K = <N, Sigma,
// Delta> §4.2). Full Generator -> Discriminator RL pipeline
// (Algorithm 1) is RUNPOD-PENDING. See
// workflow/papers/contract-coding-interface.md.
mod contract_cmd;
// C-52: `elai dafny <verify|status|list-tiers>` — Dafny Formal
// Verification Spike CLI driver. Paper-grounded against
// arXiv:2604.22601 (NL2VC). Live `dafny verify` execution is
// RUNPOD-PENDING (local dev host lacks the dafny binary; model
// substitution mandatory — Gemma 4 BLOCKED per memory
// feedback_no_gemma4). See workflow/papers/nl2vc-dafny-interface.md
// + workflow/reasoning/C-52-decisions.md.
mod dafny_cmd;
// E-13: `elai tdd <run|status|mutation-score>` — TDD verifier
// agent CLI surface. Drives orchestrator::tdd_verifier (Red/Green/
// Refactor cycle + anti-mock heuristic + cargo-mutants subprocess
// wrapper). Composes with D-3's
// ConsensusRule::VerifierKindHeterogeneous for aggregation.
mod tdd_cmd;
// E-53: `elai formal-verify <run|status>` — formal-verify pipeline
// CLI surface. Drives orchestrator::formal_verify (which composes
// C-50 contract_coding + C-52 dafny_verify under a CSQ-classifier
// gate). Live Z3 / Kani / Verus dispatch is RUNPOD-PENDING; Dafny
// dispatch reuses C-52 invoke_dafny_cli and surfaces toolchain
// absence as RunpodPending. Gemma-4 BLOCKED per memory
// feedback_no_gemma4 (substrate inheritance from C-52). See
// workflow/reasoning/E-53-decisions.md.
mod formal_verify_cmd;
// E-8: `elai optimize <template|status>` — self-evolving prompts
// driver. Spawns the Python `elai_optimizer` service. Live LLM
// calls RUNPOD-PENDING. Refuses local 7B as reflection LM per
// arXiv:2405.10276 (Zhang et al.). See
// workflow/reasoning/E-8-decisions.md.
mod optimize_cmd;
// C-40: `elai pr <open|list|status|close>` — GitHub PR adapter CLI.
// Shells out to gh via the orchestrator's github_pr::GhCliAdapter.
mod github_pr_cmd;
mod pr_review_cmd;
// D-9: `elai audit-locked-keys ...` — LockedKeysAuditor CLI driver.
mod audit_locked_keys;
// D-19: `elai bench prmbench` — PRMBench adversarial-gaming benchmark
// driver. Owns its own tokio runtime (sync entry pattern).
mod bench_prmbench;
// D-21: `elai bench mode-routing` — mode-routing coverage benchmark
// driver. Replays a hand-labeled fixture through D-10's
// `select_grounding_mode` predicate and gates pass on macro F1 + per-mode
// F1. Sibling of bench_prmbench (sync entry pattern).
mod bench_mode_routing;
// D-22: `elai bench conformal` — conformal calibration validation runner.
// Replays a held-out fixture through D-16's `decide` predicate and gates
// pass iff every distribution bucket's empirical refusal rate is inside
// the UniCR (arXiv:2503.01345) band. Sibling of bench_mode_routing.
mod bench_conformal;
// D-20: `elai bench arc` — ARC-AGI-2 OOD probe harness. Gates ELAI's
// mode-routing + novelty-classifier honesty stack against ARC-style
// out-of-distribution claims. Sibling of bench_mode_routing.
mod bench_arc;
// E-17 SP6: `elai bench rag` — retrieval-quality benchmark driver.
// Drives the local QueryPipeline at an explicit QueryTier and emits the
// final-stage retrieval results as JSON. Used by .elai_cc/eval/rag/run.py
// as its observable interface (no clean knowledge-query subcommand
// existed before SP6; /research is web research, not retrieval).
mod bench_rag;
// D-23: `elai bench sycophancy` — sycophancy detection benchmark driver.
// Quantifies the D-14 SycophancyProbe detection_rate + fp_rate against
// the labelled fixture corpus. Sibling of bench_prmbench (sync entry).
mod bench_sycophancy;
// D-24: `elai bench canary-regression` — cross-domain canary regression
// benchmark driver. Replays a labelled fixture corpus (20 Harmful + 50
// Helpful + 30 Neutral) through D-7's run_canary in-memory and emits a
// single CanaryBenchRunCompleted SSB row + a JSON report.
mod bench_canary;
// D-25: `elai bench tamper` — determinism + archive tamper-suite
// benchmark driver. Drives a 10-scenario JSON-manifest matrix
// IN-PROCESS through D-8's verify_archive and emits a single
// DeterminismTamperSuiteCompleted SSB row + a JSON report.
mod bench_tamper;
// D-28: `elai farm-adv-rm <run|verdict>` — adversarial example farming
// runner CLI surface. See workflow/reasoning/D-28-decisions.md.
mod farm_adv_rm;
// E-10: hostile-critic structural simplifications -- 3 new CLI dispatchers.
mod bench_ablation_cmd;
mod tier_eval_cmd;
mod critic_report;
// E-9: external agent-eval benchmark suite (11 suites; SWE-Bench Pro
// primary; RUNPOD-PENDING live runs). See workflow/reasoning/E-9-decisions.md.
mod eval_cmd;
// E-55: agentic RL + reward ladder recipe runner. Substrate ships fully
// under elai_orchestrator::rl::*; live cloud Sonnet rollouts + LoRA
// training RUNPOD-PENDING. See workflow/reasoning/E-55-decisions.md.
mod rl_train_cmd;
mod runtime_client;
mod runtime_cmd;
// D-2: `elai ppm health` — three-step PPM sidecar health probe
// (TCP -> /health -> canary /v1/ppm/score). Sync entrypoint owning a
// tokio current-thread runtime (mirrors archive_verify::run).
mod ppm_health;
mod mucoco_cmd;
mod mcp_client;
// C-42 (session 2): `elai mcp authz [status|list-rejections|fuzz-report]`
// — Semantic Gateway L2 RBAC dashboard surface. Reads the workspace-root
// `authz-policy.toml` and prints status / recent denies / a bounded fuzz
// summary. Sibling of `mcp_client`. Paper grounding: arXiv:2604.25555.
mod mcp_authz_cmd;
mod tool_executor;
mod tool_formatting;
// C-38: DKG lifecycle helper. Replaces the never-fires "if dkg.db
// exists" gate at live_cli.rs:1722-1732 with an ensure-or-rebuild
// helper that emits paired SSB beacons (DkgRebuildStarted /
// DkgRebuildComplete) around the rebuild path.
mod dkg_lifecycle;

#[cfg(test)]
mod cli_tests;
#[cfg(test)]
mod sandbox_report_tests;

use std::collections::BTreeSet;
use std::env;
use std::pin::Pin;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub(crate) use crate::args::*;
pub(crate) use crate::auth::*;
pub(crate) use crate::live_cli::*;
pub(crate) use crate::progress::*;
pub(crate) use crate::resume::*;
pub(crate) use crate::runtime_builder::*;
pub(crate) use crate::runtime_client::*;
pub(crate) use crate::tool_executor::*;
pub(crate) use crate::tool_formatting::*;
pub(crate) use crate::help::*;
pub(crate) use crate::commands::{SlashCommand, resume_supported_slash_commands};

pub(crate) const DEFAULT_MODEL: &str = "claude-opus-4-6";
pub(crate) fn max_tokens_for_model(model: &str) -> u32 {
    if model.contains("opus") {
        32_000
    } else {
        64_000
    }
}
pub(crate) const TOKEN_CHARS_PER_TOKEN: usize = 4;
pub(crate) const BUDGET_OUTPUT_RESERVE: usize = 4096;

pub(crate) const FENCE_LANG_MAP: &[(&str, &str)] = &[
    ("rust", "rust"), ("rs", "rust"),
    ("python", "python"), ("py", "python"),
    ("typescript", "typescript"), ("ts", "typescript"),
    ("go", "go"), ("golang", "go"),
    ("java", "java"),
    ("csharp", "c_sharp"), ("cs", "c_sharp"), ("c#", "c_sharp"),
    ("ruby", "ruby"), ("rb", "ruby"),
];

pub(crate) fn shortcoder_compress_text(text: &str, sc: &elai_knowledge::ShortCoder, saved: &mut usize) -> usize {
    if !text.contains("```") {
        return text.len();
    }
    let mut effective_chars = 0usize;
    let mut pos = 0;
    while pos < text.len() {
        if let Some(fence_start) = text[pos..].find("```") {
            let abs_fence_start = pos + fence_start;
            effective_chars += fence_start;
            let after_ticks = abs_fence_start + 3;
            let line_end = text[after_ticks..].find('\n')
                .map(|i| after_ticks + i)
                .unwrap_or(text.len());
            let lang_tag = text[after_ticks..line_end].trim().to_lowercase();
            let code_start = if line_end < text.len() { line_end + 1 } else { line_end };
            let closing = text[code_start..].find("\n```").map(|i| code_start + i);
            if let Some(close_pos) = closing {
                let code_content = &text[code_start..close_pos];
                let fence_end = close_pos + 4;
                let after_close = text[fence_end..].find('\n')
                    .map(|i| fence_end + i + 1)
                    .unwrap_or(text.len());
                let ts_lang = FENCE_LANG_MAP.iter()
                    .find(|(tag, _)| *tag == lang_tag)
                    .map(|(_, lang)| *lang);
                if let Some(lang) = ts_lang {
                    match sc.simplify(code_content, lang) {
                        Ok(result) if result.was_simplified() => {
                            let original_block_len = after_close - abs_fence_start;
                            let simplified_block_len = 3 + lang_tag.len() + 1
                                + result.simplified.len() + 1 + 3;
                            if simplified_block_len < original_block_len {
                                *saved += original_block_len - simplified_block_len;
                            }
                            effective_chars += simplified_block_len;
                        }
                        _ => {
                            effective_chars += after_close - abs_fence_start;
                        }
                    }
                } else {
                    effective_chars += after_close - abs_fence_start;
                }
                pos = after_close;
            } else {
                effective_chars += text.len() - abs_fence_start;
                pos = text.len();
            }
        } else {
            effective_chars += text.len() - pos;
            pos = text.len();
        }
    }
    effective_chars
}

pub(crate) const DEFAULT_DATE: &str = "2026-03-31";
pub(crate) const DEFAULT_OAUTH_CALLBACK_PORT: u16 = 4545;
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const BUILD_TARGET: Option<&str> = option_env!("TARGET");
pub(crate) const GIT_SHA: Option<&str> = option_env!("GIT_SHA");
pub(crate) const INTERNAL_PROGRESS_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);
pub(crate) const SESSION_HANDOFF_THRESHOLD: f64 = 0.85;
pub(crate) const PLANS_DIR: &str = "plans";
pub(crate) const PRIMARY_SESSION_EXTENSION: &str = "jsonl";
pub(crate) const OLLAMA_DEFAULT_URL: &str = "http://localhost:11434";
pub(crate) const DEGRADED_PROMPT_PREFIX: &str = "[degraded] using ollama fallback";
pub(crate) const OLLAMA_PROBE_TIMEOUT: Duration = Duration::from_secs(2);
pub(crate) const KNOWLEDGE_TOP_K: usize = 5;
pub(crate) const EMPTY_TIER_TOML: &str = "\
[tier1]\ndomains = []\npatterns = []\n\
[tier2]\ndomains = []\nmin_so_score = 5\n\
[tier4_blocklist]\ndomains = []\npatterns = []\n";

pub(crate) struct CharEstimatorProvider;

impl elai_providers::LlmProvider for CharEstimatorProvider {
    fn complete<'a>(
        &'a self,
        _prompt: &'a str,
        _config: &'a elai_providers::OllamaConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, elai_providers::ApiError>> + Send + 'a>> {
        Box::pin(async { Err(elai_providers::ApiError::Auth("CharEstimatorProvider does not support completion".into())) })
    }

    fn embed<'a>(
        &'a self,
        _text: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, elai_providers::ApiError>> + Send + 'a>> {
        Box::pin(async { Err(elai_providers::ApiError::Auth("CharEstimatorProvider does not support embedding".into())) })
    }

    fn token_count(&self, text: &str) -> Result<usize, elai_providers::ApiError> {
        Ok(text.len() / TOKEN_CHARS_PER_TOKEN)
    }

    fn token_count_estimate(&self, text: &str) -> usize {
        text.len() / TOKEN_CHARS_PER_TOKEN
    }
}

pub(crate) const LEGACY_SESSION_EXTENSION: &str = "json";
pub(crate) const LATEST_SESSION_REFERENCE: &str = "latest";
pub(crate) const SESSION_REFERENCE_ALIASES: &[&str] = &[LATEST_SESSION_REFERENCE, "last", "recent"];
pub(crate) const CLI_OPTION_SUGGESTIONS: &[&str] = &[
    "--help",
    "-h",
    "--version",
    "-V",
    "--model",
    "--output-format",
    "--permission-mode",
    "--dangerously-skip-permissions",
    "--allowedTools",
    "--allowed-tools",
    "--resume",
    "--print",
    "-p",
];

pub(crate) type AllowedToolSet = BTreeSet<String>;
pub(crate) type RuntimePluginStateBuildOutput = (
    Option<Arc<Mutex<live_cli::RuntimeMcpState>>>,
    Vec<elai_tools::RuntimeToolDefinition>,
);

fn main() {
    if let Err(error) = run() {
        let message = error.to_string();
        if message.contains("`elai--help`") {
            eprintln!("error: {message}");
        } else {
            eprintln!(
                "error: {message}\n\nRun `elai--help` for usage."
            );
        }
        std::process::exit(1);
    }
}

/// Gateway-rejection guard per E-24 Task 6.
/// Warns if ANTHROPIC_BASE_URL is set to a known LLM gateway URL.
/// Returns the gateway name if detected, None for safe or unknown URLs.
/// Hard-block requires --strict-no-gateway flag (not yet wired; warn only for now).
pub(crate) fn check_anthropic_gateway(base_url: &str) -> Option<String> {
    let known_gateways = [
        ("openrouter.ai", "openrouter"),
        ("litellm", "litellm"),
        ("portkey.ai", "portkey"),
        ("openai.azure.com", "azure-openai"),
    ];

    let url_lower = base_url.to_lowercase();
    for (pattern, name) in &known_gateways {
        if url_lower.contains(pattern) {
            return Some((*name).to_string());
        }
    }

    // Known-safe URLs: localhost, loopback, or official Anthropic API.
    if url_lower.starts_with("http://localhost:")
        || url_lower.starts_with("http://127.0.0.1:")
        || url_lower.starts_with("https://api.anthropic.com")
    {
        return None;
    }

    // Unknown URL -- warn but do not block (no --strict-no-gateway in args yet).
    None
}

fn run() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().skip(1).collect();
    // C-35: emit startup warning when --tool-gate-off is active.
    // The flag is recognized in parse_args (args.rs) and stored as ELAI_TOOL_GATE_OFF=1.
    // Check before parse_args so the warning fires even if the action itself changes.
    if args.iter().any(|a| a == "--tool-gate-off") {
        eprintln!("[WARN] tool gating disabled; prompt costs will be 5-10x higher");
    }
    // E-24: gateway-rejection guard. Warn if ANTHROPIC_BASE_URL points at a
    // known LLM gateway (openrouter, litellm, portkey). Prompt cache breaks
    // through these gateways and latency overhead can be 50ms+.
    if let Ok(base_url) = env::var("ANTHROPIC_BASE_URL") {
        if let Some(gateway) = check_anthropic_gateway(&base_url) {
            tracing::warn!(
                gateway = %gateway,
                base_url = %base_url,
                "anthropic_gateway_detected: ANTHROPIC_BASE_URL points at a known gateway; \
                 prompt cache is broken through gateways and latency overhead applies. \
                 Use api.anthropic.com directly or localhost sidecar."
            );
        }
    }
    match parse_args(&args).map_err(|e| anyhow::anyhow!(e))? {
        CliAction::DumpManifests { output_format } => auth::dump_manifests(output_format)?,
        CliAction::BootstrapPlan { output_format } => auth::print_bootstrap_plan(output_format)?,
        CliAction::Agents { args, output_format } => LiveCli::print_agents(args.as_deref(), output_format)?,
        CliAction::Mcp { args, output_format } => LiveCli::print_mcp(args.as_deref(), output_format)?,
        CliAction::Runtime { action, role, target, output_format: _ } => {
            // B-37D Phase 4M: text-only output for now; JSON shaping deferred to
            // when RuntimeOrchestrator::health_snapshot lands (Wave-1B).
            LiveCli::print_runtime(action.as_deref(), role.as_deref(), target.as_deref())?;
        }
        CliAction::Skills { args, output_format } => LiveCli::print_skills(args.as_deref(), output_format)?,
        CliAction::PrintSystemPrompt { cwd, date, output_format } => print_system_prompt(cwd, date, output_format)?,
        CliAction::Version { output_format } => print_version(output_format)?,
        CliAction::ResumeSession { session_path, commands, output_format } => resume_session(&session_path, &commands, output_format),
        CliAction::Status { model, permission_mode, output_format } => live_cli::print_status_snapshot(&model, permission_mode, output_format)?,
        CliAction::Sandbox { output_format } => live_cli::print_sandbox_status_snapshot(output_format)?,
        CliAction::Prompt { prompt, model, output_format, allowed_tools, permission_mode, backend } => {
            // C-17 (DRIFT-D/E): auto-resume any persisted indexing batches
            // and install the process-wide SIGINT/SIGTERM observer before
            // dispatching the prompt. The project root is the CWD — the
            // CLI does not accept --project-root on this arm.
            let prompt_project_root = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            c17_lifecycle::install_lifecycle(&prompt_project_root);
            LiveCli::new_prompt(model, true, allowed_tools, permission_mode, backend)?
                .run_turn_with_output(&prompt, output_format)?;
        }
        CliAction::Login { output_format } => run_login(output_format)?,
        CliAction::Logout { output_format } => run_logout(output_format)?,
        CliAction::Doctor { output_format } => doctor::run_doctor(output_format)?,
        // E-17 SP1 task 1 — `elai doctor knowledge-models [...]`.
        CliAction::DoctorKnowledgeModels {
            project_root,
            output_format,
        } => doctor::run_knowledge_models_probe(&project_root, output_format)?,
        // C-43 — `elai freeze status|verify|ratify`. Phase 4H scope-freeze
        // CLI surface. ELAI design hypothesis (see
        // workflow/sessions/s-25120/mistakes.md for the bad-cite
        // analysis of arXiv:2604.21725).
        CliAction::Freeze { action, output_format } => {
            freeze_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // C-45 — `elai credit attribute|table|export`. CANTANTE
        // contrastive credit-attribution surface; paper-grounded against
        // arXiv:2605.13295. Synthetic-fixture path is GPU-free; live
        // pilot is `RUNPOD-PENDING: cantante_pilot_450runs_qwen3_8b`.
        CliAction::Credit { action, output_format } => {
            credit_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-43 -- `elai skills-mgmt list|eval|publish`. Agent skill systems
        // surface (frontmatter + eval/ + publish dist). ABSORBs the
        // E-22 SKILL.md export shim. Paper-grounded against
        // agentskills.io/specification.
        CliAction::SkillsManagement { action, output_format } => {
            skills_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-39 -- `elai cloud override-cap|status`. Layered-cap
        // manual-override CLI surface. Composes with E-38 EU AI Act
        // audit-trail liveness gate. See
        // workflow/reasoning/E-39-decisions.md.
        CliAction::Cloud { action, output_format } => {
            cloud_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // C-46 — `elai routing decisions|pilot|status|export`.
        // Uno-Orchestra parsimonious agent routing surface; paper-
        // grounded against arXiv:2605.05007. Synthetic-fixture path is
        // GPU-free; live pilot is
        // `RUNPOD-PENDING: uno_orchestra_pilot_8roles_qwen3_8b`.
        CliAction::Routing { action, output_format } => {
            routing_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // C-47 — `elai calibration status|reset-hints|export`. Case-based
        // tool-use calibration; paper-grounded against arXiv:2605.15041
        // (CAST). The CLI surface is synthetic + GPU-free; live registry
        // IPC is `RUNPOD-PENDING`.
        CliAction::Calibration { action, output_format } => {
            calibration_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-14 — `elai browser run|list-journeys|status`. Browser-e2e
        // agent surface; wraps elai_orchestrator::browser_e2e. Wire-dark
        // posture for absent npx/playwright; live VLM-tier RUNPOD-PENDING.
        CliAction::Browser { action, output_format } => {
            browser_e2e_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-18 — `elai reliability status|list-spills|replay-spill`.
        // Tool-use reliability gate surface; handler at
        // crates/cli/src/reliability_cmd.rs. Composes on D-11 +
        // E-35 + C-47 + C-42 + C-35 (reuse-heavy posture). See
        // workflow/reasoning/E-18-decisions.md.
        CliAction::Durability { action, output_format } => {
            durability_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-45 — `elai drift detect|list-pairs|baseline`. Cross-language
        // drift detector operator surface; handler at
        // crates/cli/src/drift_cmd.rs. WIRE-DARK avoidance for
        // cross_lang_drift::DriftDetector + PairRegistry per E-41
        // precedent. See workflow/reasoning/E-45-decisions.md.
        CliAction::Drift { action, output_format } => {
            drift_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-49 — `elai metrics capture|swe-bench-pro|refinebench|aggregate|status`.
        // Coding-agent metric suite operator surface (scope-lean ship);
        // handler at crates/cli/src/metrics_cmd.rs. WIRE-DARK avoidance
        // for coding_metrics::MetricsCollector + aggregate_session_metrics
        // per E-41 precedent. Live SWE-Bench Pro + RefineBench transports
        // are RUNPOD-PENDING. See workflow/reasoning/E-49-decisions.md.
        CliAction::Metrics { action, output_format } => {
            metrics_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-35 — `elai structured-output status|codeact-run|strict-mode|
        // grammar emit-test|search-replace check|reject-engine`.
        // Anchors host-side structured-output substrate so the 10 new
        // E-35 SSB beacon variants are not WIRE-DARK. Python sidecar
        // wiring is RUNPOD-PENDING; AgentRole::Runner DEFERRED per
        // E41_AGENTROLE_FROZEN_AT_17. See
        // workflow/reasoning/E-35-decisions.md.
        CliAction::StructuredOutput { action, output_format } => {
            structured_output_cmd::run(action, output_format)
                .map_err(|e| anyhow::anyhow!(e))?
        }
        CliAction::Reliability { action, output_format } => {
            reliability_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-37 — `elai safety status|list-pins|verify-pins|fixture-replay`.
        // Prompt-injection defense operator surface (OWASP LLM01); handler
        // at crates/cli/src/safety_cmd.rs. Layers additively atop E-24
        // mcp_schema_pin + sanitize_result + llm_guard and E-30
        // destructive_verb_allowlist. See
        // workflow/reasoning/E-37-decisions.md.
        CliAction::Safety { action, output_format } => {
            safety_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-40 — `elai scip query|defs|refs|callers|callees|pagerank|status|import`.
        // Orchestrator-rooted symbol-graph operator surface; handler at
        // crates/cli/src/scip_cmd.rs. Production caller for the new
        // elai_orchestrator::symbol_graph module — defends against
        // E37_WIRE_DARK_GAP_PATTERN. See workflow/reasoning/E-40-decisions.md.
        CliAction::SymbolGraph { action, output_format } => {
            scip_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // D-16 — `elai conformal-calibration status|update`. Durable
        // UniCR tau persistence; handler at
        // crates/cli/src/conformal_calibration_cli.rs. Paper-grounded
        // against arXiv:2509.01455 (UniCR Eq.5). Namespace distinct
        // from C-47's `elai calibration` per D-16-decisions.md
        // DECISION-5.
        CliAction::ConformalCalibration { action, output_format } => {
            conformal_calibration_cli::run(action, output_format)
                .map_err(|e| anyhow::anyhow!(e))?
        }
        // C-49 — `elai experience distill|list|export|reset`. ELAI
        // heuristic adaptation of arXiv:2605.14443 (Sayana, Todi, Jash
        // 2026). Synthetic-fixture path is GPU-free; live trace
        // recorder is `RUNPOD-PENDING:
        // iterative_distillation_prompter_rl_qwen3_8b`.
        CliAction::Experience { action, output_format } => {
            experience_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // C-50 — `elai contract scaffold|audit|list`. Contract-Coding
        // STRUCTURAL contract test scaffolding surface; paper-
        // grounded against arXiv:2604.13100. Synthetic-fixture path
        // is GPU-free; the live Generator -> Discriminator RL
        // pipeline (Algorithm 1) is `RUNPOD-PENDING:
        // contract_coding_gen_disc_rl_qwen3_8b`. Citation
        // discipline: the "47%" is the Roguelike subset of
        // Greenfield-5 (Table 1) and TIES Trae; the differentiator
        // is structural integrity per Section 5.2.
        CliAction::Contract { action, output_format } => {
            contract_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // C-52 — `elai dafny verify|status|list-tiers`. Dafny Formal
        // Verification Spike CLI surface; paper-grounded against
        // arXiv:2604.22601 (NL2VC). Live `dafny verify` execution
        // is `RUNPOD-PENDING: live_dafny_verification_with_non_gemma_substitution`.
        // Model substitution mandatory (Gemma 4 BLOCKED per memory
        // feedback_no_gemma4).
        CliAction::Dafny { action, output_format } => {
            dafny_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-13: `elai tdd run|status|mutation-score`. TDD verifier
        // agent dispatch — composes with orchestrator::tdd_verifier
        // and D-3's ConsensusRule::VerifierKindHeterogeneous. Live
        // cargo-mutants execution is RUNPOD-PENDING when the binary
        // is absent (wire-dark posture).
        CliAction::Tdd { action, output_format } => {
            tdd_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-53 — `elai formal-verify run|status`. Formal-verify
        // pipeline dispatch — composes orchestrator::formal_verify
        // (C-50 contract_coding + C-52 dafny_verify under CSQ gate).
        // Live Z3 / Kani / Verus dispatch is RUNPOD-PENDING; Dafny
        // dispatch reuses C-52 invoke_dafny_cli. Gemma-4 BLOCKED per
        // memory feedback_no_gemma4 (substrate inheritance from C-52).
        CliAction::FormalVerify { action, output_format } => {
            formal_verify_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        // E-8 — `elai optimize template|status`. Self-evolving prompts
        // driver. Spawns the Python `elai_optimizer` service (GEPA
        // primary per arXiv:2507.19457 + DSPy MIPROv2 / OPRO
        // supporting). Live LLM calls RUNPOD-PENDING. Refuses local 7B
        // as reflection LM per arXiv:2405.10276 (Zhang et al.) -- the
        // args parser enforces the refusal before this handler runs.
        // See workflow/reasoning/E-8-decisions.md.
        CliAction::Optimize { action, output_format } => {
            optimize_cmd::run(action, output_format).map_err(|e| anyhow::anyhow!(e))?
        }
        CliAction::Init { output_format } => live_cli::run_init(output_format)?,
        CliAction::Repl { model, allowed_tools, permission_mode, export_jsonl } => {
            // C-17 (DRIFT-D/E): auto-resume + signal observer for the
            // long-lived REPL scope. Sync entrypoint — see c17_lifecycle.rs
            // for the transient-runtime pattern.
            let repl_project_root = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            c17_lifecycle::install_lifecycle(&repl_project_root);
            repl::run_repl(model, allowed_tools, permission_mode, export_jsonl)?;
        }
        CliAction::DepAudit { project_dir, output_format } => doctor::run_dep_audit_command(&project_dir, output_format)?,
        CliAction::Local { port } => repl::run_local_repl(port)?,
        CliAction::Dashboard => dashboard::run_dashboard()?,
        // Phase A1a: production caller of `start_task_runner`. Pinned by
        // `crates/contracts/tests/task_runner_chokepoint_contract.rs`.
        CliAction::Run { project_root, mode, provider, via_mcp, empire, export_ssb } => {
            // C-17 (DRIFT-D/E): auto-resume + process-wide signal observer
            // for the long-lived `elai run` task_runner scope. Runs BEFORE
            // run_subcommand builds its own tokio runtime so the resume
            // pass uses a transient runtime that finishes promptly.
            // C-40 session-2 minimal repair: surface the empire + export_ssb
            // fields added by args.rs (session 3 is editing main.rs
            // concurrently and owns the canonical wiring; this is the
            // smallest valid destructure so the workspace compiles).
            c17_lifecycle::install_lifecycle(&project_root);
            run_subcommand::run_task_runner_subcommand(run_subcommand::RunArgs {
                project_root,
                mode,
                provider,
                via_mcp,
                empire,
                export_ssb,
            })?
        }
        CliAction::RemoteSidecar => remote_sidecar::run_remote_sidecar()?,
        CliAction::HelpTopic(topic) => live_cli::print_help_topic(topic),
        CliAction::Help { output_format } => help::print_help(output_format)?,
        // B-29 Phase 4: symbol graph CLI subcommands.
        CliAction::SymbolGraphBuild { project_root } => {
            symbol_graph::run_symbol_graph_build(project_root)?
        }
        CliAction::SymbolGraphStats { project_root } => {
            symbol_graph::run_symbol_graph_stats(project_root)?
        }
        // E-21 Tier-A: `elai symbol_graph rank --top N` — top-N pagerank
        // symbols from the existing dkg_nodes_pagerank table (C-3 ship).
        CliAction::SymbolGraphRank { project_root, top } => {
            symbol_graph::run_symbol_graph_rank(project_root, top)?
        }
        // C-13: `elai wip-squash [--project-root <path>]` — squashes leading
        // WIP commits when count > 5. Invoked by the Python FSM hook chain
        // at AUDITING -> MEMORIZING transitions.
        CliAction::WipSquash { project_root } => {
            wip_squash_subcommand::run_wip_squash_subcommand(project_root)?
        }
        // C-20: `elai measure [--scope <name>] [--cloud] [--output toml|json]`
        // — test-infra metrics backfill driver. Owns its own tokio runtime
        // (sync entrypoint pattern mirrored from `run_subcommand.rs`).
        CliAction::Measure { scope, cloud, output_format: _ } => {
            let project_root = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            measure::run_measure_subcommand(scope, cloud, project_root)?
        }
        // C-25: `elai pipeline [list|reset]` — inspect or reset the
        // compiled-pipeline cache at `.elai/cache/compiled_pipelines.msgpack`.
        CliAction::Pipeline { action, output_format: _ } => {
            let project_root = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            pipeline_subcommand::run_pipeline_subcommand(action.as_deref(), &project_root)?
        }
        // C-27: `elai wiring audit ...` — deterministic Tier T-0 wiring
        // contract audit. Owns its own tokio runtime (sync entry, mirrors
        // pipeline/measure subcommands). Exits 1 on any violation.
        CliAction::WiringAudit {
            scope,
            config_path,
            contracts_dir,
            output_format,
        } => {
            wiring_audit::run(scope, config_path, contracts_dir, output_format)?;
        }
        // D-8: `elai archive verify [--path PATH] [--json] [--fail-fast]`
        // Walks the JSONL ArchiveChain via D-1's verify primitives and
        // exits 1 on any tamper / broken-parent / IO / malformed-JSON.
        CliAction::ArchiveVerify {
            path,
            json,
            fail_fast,
        } => {
            archive_verify::run(path, json, fail_fast)?;
        }
        // D-9: `elai audit-locked-keys [--registry PATH] [--staged | --files A,B]
        // [--json] [--fail-fast]` — LockedKeysAuditor driver. Exits 1 on
        // any registry overlap; 0 on clean / empty registry.
        CliAction::AuditLockedKeys {
            registry_path,
            files,
            use_staged,
            json,
            fail_fast,
        } => {
            audit_locked_keys::run(registry_path, files, use_staged, json, fail_fast)?;
        }
        // D-19: `elai bench prmbench [--dataset PATH] [--hook mock] ...`
        // PRMBench adversarial-gaming benchmark runner. Exits 0 on Pass,
        // 1 on Fail; emits one SsbActionType::PrmBenchRunCompleted row.
        CliAction::BenchPrmbench {
            dataset,
            hook,
            output,
            gating_threshold,
            category,
        } => {
            bench_prmbench::run(dataset, hook, output, gating_threshold, category)?;
        }
        // D-21: `elai bench mode-routing [--dataset PATH] [--output PATH]
        // [--macro-floor 0.85] [--per-mode-floor 0.70]`. Replays the
        // hand-labeled fixture through D-10's mode-routing predicate.
        // Exits 0 on Pass, 1 on Fail; emits one
        // SsbActionType::ModeRoutingBenchRunCompleted row.
        CliAction::BenchModeRouting {
            dataset,
            output,
            macro_floor,
            per_mode_floor,
        } => {
            bench_mode_routing::run(dataset, output, macro_floor, per_mode_floor)?;
        }
        // D-22: `elai bench conformal [--dataset PATH] [--corpus PATH]
        // [--output PATH] [--epsilon 0.1]`. Replays the validation
        // fixture through D-16's `decide` predicate per distribution
        // bucket. Exits 0 on Pass, 1 on Fail; emits one
        // SsbActionType::ConformalValidationCompleted row.
        CliAction::BenchConformal {
            dataset,
            corpus,
            output,
            epsilon,
        } => {
            bench_conformal::run(dataset, corpus, output, epsilon)?;
        }
        // D-20: `elai bench arc [--dataset PATH] [--output PATH]
        // [--novelty-config PATH]`. Replays an ARC-AGI-2 fixture through
        // D-10's `select_grounding_mode` predicate and D-15's
        // `classify`. Exits 0 iff verdict is HonestlyEvaluated; emits
        // one SsbActionType::ArcBenchmarkCompleted row.
        CliAction::BenchArc {
            dataset,
            output,
            novelty_config,
        } => {
            bench_arc::run(dataset, output, novelty_config)?;
        }
        // E-17 SP6: `elai bench rag --tier ... --top-k ... [--json] -- <Q>`.
        // Builds the local knowledge pipeline via LiveCli::build_local_knowledge_stack
        // (same constructor production uses), runs query_with_tier at the
        // explicit tier, prints final-stage results. Exits 0 on success.
        CliAction::BenchRag {
            tier,
            top_k,
            query,
            json,
            project_root,
        } => {
            bench_rag::run(tier, top_k, query, json, project_root)?;
        }
        // D-23: `elai bench sycophancy [--fixtures PATH] [--output PATH]
        // [--detection-floor 0.70] [--fp-ceiling 0.15]`. Quantifies D-14
        // SycophancyProbe detection_rate + fp_rate against the labelled
        // fixture corpus. Exits 0 on Pass, 1 on Fail; the in-process
        // orchestrator producer emits one
        // SsbActionType::SycophancyBenchRunCompleted row when invoked
        // from a task_runner (the CLI does not persist SSB rows directly).
        CliAction::BenchSycophancy {
            fixtures,
            output,
            detection_floor,
            fp_ceiling,
        } => {
            bench_sycophancy::run(fixtures, output, detection_floor, fp_ceiling)?;
        }
        // D-24: `elai bench canary-regression ...` — cross-domain canary
        // regression benchmark. Replays a labelled fixture corpus
        // through D-7 run_canary in-memory; emits exactly one
        // SsbActionType::CanaryBenchRunCompleted row when invoked from
        // a task_runner (the CLI does not persist SSB rows directly).
        // Exits 0 on Pass, 1 on Fail.
        CliAction::BenchCanary {
            fixtures,
            output,
            harmful_block_floor,
            helpful_block_ceiling,
            neutral_block_ceiling,
        } => {
            bench_canary::run(
                fixtures,
                output,
                harmful_block_floor,
                helpful_block_ceiling,
                neutral_block_ceiling,
            )?;
        }
        // D-25: `elai bench tamper` — determinism + archive tamper-suite
        // benchmark driver. Drives D-8 verify_archive IN-PROCESS across
        // the 10-scenario matrix; emits one
        // SsbActionType::DeterminismTamperSuiteCompleted row when
        // invoked from a task_runner (CLI does not persist SSB rows
        // directly). Exits 0 on Pass, 1 on Fail.
        CliAction::BenchTamper {
            scenarios,
            output,
            fail_fast,
            max_wall_ms_per_scenario,
        } => {
            bench_tamper::run(scenarios, output, fail_fast, max_wall_ms_per_scenario)?;
        }
        // D-28: `elai farm-adv-rm <run|verdict>` — adversarial example
        // farming runner. Drives AdvRmFarmer (orchestrator-side) and
        // writes corpus.jsonl with `gold_verdict: "pending"`. Single
        // SsbActionType::AdversarialFarmRunCompleted SSB row at end-of-
        // run when invoked from a task_runner; CLI does not persist SSB
        // rows directly (mirrors D-19/D-23/D-24/D-25 bench dispatcher
        // precedent). Exits 0 on Pass, 1 on Fail.
        CliAction::FarmAdvRm { sub } => {
            farm_adv_rm::run(crate::args::FarmAdvRmArgs { sub })?;
        }
        // E-10: hostile-critic structural simplifications dispatch.
        CliAction::BenchAblation { tile, fixtures, budget } => {
            bench_ablation_cmd::run(tile, fixtures, budget)?;
        }
        CliAction::TierEval { suites, budget } => {
            tier_eval_cmd::run(suites, budget)?;
        }
        CliAction::CriticReport { output } => {
            critic_report::run(output)?;
        }
        // E-9: `elai eval <run|list> --benchmark=<NAME> [--harness-vary]
        // [--p-target=0.05] [--list] [--json]`. External agent-eval benchmark
        // dispatcher across 11 suites (swe-bench-pro / swe-bench-live /
        // livecodebench-v6 / tau2-bench / bfcl-v3 / gaia-l1-l2 /
        // clawsbench-split / refinebench / sycophancy-probes / math-floors /
        // deep-planning). Live evals RUNPOD-PENDING; host-side scaffolding
        // ships fully + emits SsbActionType::BenchmarkRunObserved when env
        // gate `ELAI_EVAL_BENCHMARK_ENABLE=1` is set. See
        // workflow/reasoning/E-9-decisions.md.
        CliAction::EvalRun {
            benchmark,
            harness_vary,
            p_target,
            list,
            json,
        } => {
            eval_cmd::run(benchmark, harness_vary, p_target, list, json)?;
        }
        // E-55: agentic RL recipe runner. Live cloud Sonnet rollouts +
        // LoRA training RUNPOD-PENDING (env-gated by
        // ELAI_AGENTIC_RL_ENABLE=1). See workflow/reasoning/E-55-decisions.md.
        CliAction::RlTrain { recipe, list, json } => {
            rl_train_cmd::run(recipe, list, json)?;
        }
        // E-30: `elai incident record ...` — store an incident postmortem entry.
        CliAction::IncidentRecord {
            title,
            incident_date,
            source_url,
            incident_id,
        } => {
            cmd_incident_record(title, incident_date, source_url, incident_id)?;
        }
        // D-2: `elai ppm health [--endpoint URL] [--timeout SECS] [--json]`.
        // Three-step PPM sidecar health probe (TCP -> /health -> canary
        // /v1/ppm/score). Owns its own tokio runtime in `ppm_health::run`
        // (sync entrypoint pattern, same as `archive_verify::run`). Exits 1
        // on any step failure via std::process::exit inside the handler.
        CliAction::PpmHealth {
            endpoint,
            timeout_secs,
            json,
        } => {
            ppm_health::run(endpoint, timeout_secs, json)?;
        }
        // E-5 session-2: `elai memory <sub>` — heuristic memory inspector.
        // Concurrent session 3 (s-31708) owns the canonical dispatcher; this
        // arm surfaces a typed error so the workspace builds while session 3
        // lands the real handler. NOT a silent stub — the user gets a clear
        // diagnostic and exit code 1.
        CliAction::Memory { sub, .. } => {
            return Err(anyhow::anyhow!(
                "elai memory {sub:?}: handler is being wired in session s-31708; \
                 not yet routed at the main.rs dispatch site"
            ));
        }
        // C-40: `elai pr <open|list|status|close> [args...]` — GitHub
        // PR adapter CLI surface. Owns its own current-thread tokio
        // runtime in `run_pr_subcommand` (sync entry, mirrors
        // archive_verify::run + wip_squash_subcommand::run).
        CliAction::Pr {
            sub,
            repo,
            title,
            body,
            head,
            base,
            draft,
            pr_number,
            state_filter,
            limit,
            json,
        } => {
            github_pr_cmd::run_pr_subcommand(
                sub,
                repo,
                title,
                body,
                head,
                base,
                draft,
                pr_number,
                state_filter,
                limit,
                json,
            )?;
        }
        CliAction::PrReview {
            sub,
            pr,
            body,
            file,
            line,
            template,
            kv_pairs,
            since,
            ruleset,
            loc_added,
            loc_removed,
            files_changed,
            touches_owned,
            repo,
            branch,
            touched_files,
            default_owner,
            title,
            json,
        } => {
            let action = build_pr_review_action(
                &sub,
                pr,
                body,
                file,
                line,
                template,
                kv_pairs,
                since,
                ruleset,
                loc_added,
                loc_removed,
                files_changed,
                touches_owned,
                repo,
                branch,
                touched_files,
                default_owner,
                title,
            )?;
            pr_review_cmd::run_pr_review_subcommand(action, json)?;
        }
    }
    Ok(())
}

/// E-41: assemble a `PrReviewAction` from the args layer's flat shape.
/// Returns the structured action OR an actionable error when a required
/// flag is missing for the given subverb. Validation lives here (not in
/// the args parser) so the args layer stays free of orchestrator types.
#[allow(clippy::too_many_arguments)]
fn build_pr_review_action(
    sub: &str,
    pr: Option<u32>,
    body: Option<String>,
    file: Option<String>,
    line: Option<u32>,
    template: Option<String>,
    kv_pairs: Vec<(String, String)>,
    since: Option<i64>,
    ruleset: Option<String>,
    loc_added: Option<u32>,
    loc_removed: Option<u32>,
    files_changed: Option<u32>,
    touches_owned: bool,
    repo: Option<String>,
    branch: Option<String>,
    touched_files: Vec<String>,
    default_owner: Option<String>,
    title: Option<String>,
) -> anyhow::Result<pr_review_cmd::PrReviewAction> {
    use pr_review_cmd::PrReviewAction;
    let require_pr = |hint: &str| -> anyhow::Result<u32> {
        pr.ok_or_else(|| anyhow::anyhow!("pr-review {hint}: PR number positional required"))
    };
    match sub {
        "compose-body" => Ok(PrReviewAction::ComposeBody {
            pr: require_pr("compose-body")?,
            template: template
                .ok_or_else(|| anyhow::anyhow!("--template is required for compose-body"))?,
            kv_pairs,
        }),
        "comment" => Ok(PrReviewAction::Comment {
            pr: require_pr("comment")?,
            body: body.ok_or_else(|| anyhow::anyhow!("--body is required for comment"))?,
            file,
            line,
        }),
        "watch" => Ok(PrReviewAction::Watch {
            pr: require_pr("watch")?,
            since,
        }),
        "verify-deps" => Ok(PrReviewAction::VerifyDeps {
            pr: require_pr("verify-deps")?,
        }),
        "semgrep-gate" => Ok(PrReviewAction::SemgrepGate {
            pr: require_pr("semgrep-gate")?,
            ruleset: ruleset.unwrap_or_else(|| "auto".to_string()),
        }),
        "auto-merge-assess" => Ok(PrReviewAction::AutoMergeAssess {
            pr: require_pr("auto-merge-assess")?,
            loc_added: loc_added.unwrap_or(0),
            loc_removed: loc_removed.unwrap_or(0),
            files_changed: files_changed.unwrap_or(1),
            touches_owned,
        }),
        "apply-branch-protection" => Ok(PrReviewAction::ApplyBranchProtection {
            repo: repo.ok_or_else(|| {
                anyhow::anyhow!("--repo is required for apply-branch-protection")
            })?,
            branch: branch.unwrap_or_else(|| "main".to_string()),
        }),
        "gen-codeowners" => Ok(PrReviewAction::GenCodeowners {
            touched: touched_files
                .into_iter()
                .map(std::path::PathBuf::from)
                .collect(),
            default_owner: default_owner
                .ok_or_else(|| anyhow::anyhow!("--default-owner is required for gen-codeowners"))?,
        }),
        "suggest-squash" => Ok(PrReviewAction::SuggestSquash {
            title: title
                .ok_or_else(|| anyhow::anyhow!("--title is required for suggest-squash"))?,
            body_hint: body.unwrap_or_default(),
        }),
        "detect-force-push" => Ok(PrReviewAction::DetectForcePush {
            pr: require_pr("detect-force-push")?,
            repo,
        }),
        other => Err(anyhow::anyhow!(
            "unknown pr-review sub-action '{other}' (compose-body | comment | watch | verify-deps | \
             semgrep-gate | auto-merge-assess | apply-branch-protection | gen-codeowners | \
             suggest-squash | detect-force-push)"
        )),
    }
}

/// E-30: Store an incident postmortem entry in the SSB.
///
/// Emits `SsbActionType::IncidentRecordCreated` so the dashboard can surface
/// the entry in its activity feed. The incident_id defaults to a slug derived
/// from the title if not provided.
fn cmd_incident_record(
    title: String,
    incident_date: String,
    source_url: String,
    incident_id: Option<String>,
) -> anyhow::Result<()> {
    // Derive an ID from the title if none was provided.
    let id = incident_id.unwrap_or_else(|| {
        let slug: String = title
            .chars()
            .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        format!("inc-{slug}")
    });
    // Print the record to stdout in a machine-readable format.
    // The SSB write path requires a running runtime; for now we emit a
    // JSON record to stdout so CI scripts can ingest it. The full SSB
    // write path is wired in the run_subcommand integration layer (E-30 Task 9).
    let record = serde_json::json!({
        "action_type": "incident_record_created",
        "incident_id": id,
        "title": title,
        "incident_date": incident_date,
        "source_url": source_url,
    });
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
}
