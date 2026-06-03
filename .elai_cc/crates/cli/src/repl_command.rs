// SP3 pure-move extraction: the REPL slash-command dispatch body of
// `LiveCli::handle_repl_command` lives here. This module is a DESCENDANT of
// `crate::live_cli` (declared via `#[path = "repl_command.rs"] mod repl_command;`
// inside live_cli.rs), so the moved code reaches LiveCli's private methods and
// live_cli.rs's private free functions through `super::` with ZERO visibility
// widening (Rust descendant-module privacy). `handle_repl_command` remains in
// live_cli.rs as a thin shim delegating to `dispatch`; the repl.rs call sites
// are unchanged. This is a behaviour-preserving move only -- no logic change.

#![allow(clippy::too_many_lines)]

use super::*;

// Hoisted out of the inline `SlashCommand::Task` arm so the moved dispatch body
// compiles at module scope. Behaviour is identical to the prior inline type.
struct NoopStepEmitter;
impl elai_orchestrator::web_research_orchestrator::WebResearchStepEmitter
    for NoopStepEmitter
{
    fn emit(
        &self,
        event: elai_orchestrator::web_research_orchestrator::WebResearchStepEvent,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = ()> + Send + '_>,
    > {
        tracing::debug!(
            action = %event.action,
            iteration = event.iteration,
            phi = event.phi,
            psi = event.psi,
            sources_fetched = event.sources_fetched,
            "task: web-research step"
        );
        Box::pin(std::future::ready(()))
    }
}

/// Dispatch a parsed REPL slash command against the live CLI. Moved verbatim
/// from `LiveCli::handle_repl_command`; `cli` is the `&mut LiveCli` receiver.
pub(super) fn dispatch(
    cli: &mut super::LiveCli,
    command: super::SlashCommand,
) -> Result<bool, anyhow::Error> {
        Ok(match command {
            SlashCommand::Help => {
                println!("{}", render_repl_help());
                false
            }
            SlashCommand::Status => {
                cli.print_status();
                false
            }
            SlashCommand::Bughunter { scope } => {
                cli.run_bughunter(scope.as_deref())?;
                false
            }
            SlashCommand::Commit => {
                cli.run_commit(None)?;
                false
            }
            SlashCommand::Pr { context } => {
                cli.run_pr(context.as_deref())?;
                false
            }
            SlashCommand::Issue { context } => {
                cli.run_issue(context.as_deref())?;
                false
            }
            SlashCommand::Ultraplan { task } => {
                cli.run_ultraplan(task.as_deref())?;
                false
            }
            SlashCommand::Teleport { target } => {
                super::LiveCli::run_teleport(target.as_deref())?;
                false
            }
            SlashCommand::DebugToolCall => {
                cli.run_debug_tool_call(None)?;
                false
            }
            SlashCommand::Sandbox => {
                super::LiveCli::print_sandbox_status();
                false
            }
            SlashCommand::Compact => {
                cli.compact()?;
                false
            }
            SlashCommand::Model { model } => cli.set_model(model)?,
            SlashCommand::Permissions { mode } => cli.set_permissions(mode)?,
            SlashCommand::Clear { confirm } => cli.clear_session(confirm)?,
            SlashCommand::Cost => {
                cli.print_cost();
                false
            }
            SlashCommand::Resume { session_path } => cli.resume_session(session_path)?,
            SlashCommand::Continue { snap_id } => cli.continue_from_handoff(snap_id.as_deref())?,
            SlashCommand::Config { section } => {
                super::LiveCli::print_config(section.as_deref())?;
                false
            }
            SlashCommand::Mcp { action, target } => {
                let args = match (action.as_deref(), target.as_deref()) {
                    (None, None) => None,
                    (Some(action), None) => Some(action.to_string()),
                    (Some(action), Some(target)) => Some(format!("{action} {target}")),
                    (None, Some(target)) => Some(target.to_string()),
                };
                super::LiveCli::print_mcp(args.as_deref(), CliOutputFormat::Text)?;
                false
            }
            SlashCommand::Runtime {
                action,
                role,
                target,
            } => {
                super::LiveCli::print_runtime(
                    action.as_deref(),
                    role.as_deref(),
                    target.as_deref(),
                )?;
                false
            }
            SlashCommand::Mucoco {
                action,
                file,
                operator,
                language,
            } => {
                super::LiveCli::print_mucoco(
                    action.as_deref(),
                    file.as_deref(),
                    operator.as_deref(),
                    language.as_deref(),
                )?;
                false
            }
            SlashCommand::Memory => {
                super::LiveCli::print_memory()?;
                false
            }
            SlashCommand::Init => {
                run_init(CliOutputFormat::Text)?;
                false
            }
            SlashCommand::Diff => {
                super::LiveCli::print_diff()?;
                false
            }
            SlashCommand::Version => {
                super::LiveCli::print_version(CliOutputFormat::Text);
                false
            }
            SlashCommand::Export { path, jsonl } => {
                cli.export_session(path.as_deref(), jsonl)?;
                false
            }
            SlashCommand::Session { action, target } => {
                cli.handle_session_command(action.as_deref(), target.as_deref())?
            }
            SlashCommand::Plugins { action, target } => {
                cli.handle_plugins_command(action.as_deref(), target.as_deref())?
            }
            SlashCommand::Agents { args } => {
                super::LiveCli::print_agents(args.as_deref(), CliOutputFormat::Text)?;
                false
            }
            SlashCommand::Skills { args } => {
                super::LiveCli::print_skills(args.as_deref(), CliOutputFormat::Text)?;
                false
            }
            SlashCommand::Doctor => {
                println!("{}", doctor::render_doctor_report()?.render());
                false
            }
            SlashCommand::Profile { path } => {
                let root = path
                    .as_deref()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                match detect_project_profile(&root) {
                    Ok(mut profile) => {
                        populate_tier3(&mut profile, &[]);
                        let output = elai_knowledge::serialize_profile(&profile)
                            .unwrap_or_else(|e| format!("serialization error: {e}"));
                        println!("{output}");
                    }
                    Err(e) => eprintln!("profile detection failed: {e}"),
                }
                false
            }
            SlashCommand::Plan { mode } => {
                match &cli.plan {
                    None => {
                        println!("No plan loaded. Place a plan file at {PLANS_DIR}/<pipeline>.toml and restart.");
                    }
                    Some(plan) => {
                        match mode.as_deref() {
                            None | Some("list") | Some("") => {
                                println!("Plan: {}", plan.pipeline_id);
                                println!("Version: {}", plan.version);
                                println!("Phases ({}):", plan.phases.len());
                                for (id, entry) in &plan.phases {
                                    let deps = if entry.depends_on.is_empty() {
                                        String::new()
                                    } else {
                                        format!(" [deps: {}]", entry.depends_on.join(", "))
                                    };
                                    println!("  {id}: {}{deps}", entry.goal);
                                }
                                if !plan.global_constraints.is_empty() {
                                    println!("Constraints:");
                                    for c in &plan.global_constraints {
                                        println!("  - {c}");
                                    }
                                }
                            }
                            Some(phase_arg) => {
                                // /plan <phase_id> — show contracts
                                match plan.extract_contracts(phase_arg) {
                                    Ok(contracts) => {
                                        println!("Contracts for phase '{phase_arg}':");
                                        println!("  Inputs:");
                                        for c in &contracts.input_contracts {
                                            println!("    {}: {}", c.name, c.type_signature);
                                        }
                                        println!("  Outputs:");
                                        for c in &contracts.output_contracts {
                                            println!("    {}: {}", c.name, c.type_signature);
                                        }
                                    }
                                    Err(e) => eprintln!("Plan error: {e}"),
                                }
                            }
                        }
                    }
                }
                false
            }
            SlashCommand::Login
            | SlashCommand::Logout
            | SlashCommand::Vim
            | SlashCommand::Upgrade
            | SlashCommand::Stats
            | SlashCommand::Share
            | SlashCommand::Feedback
            | SlashCommand::Files
            | SlashCommand::Fast
            | SlashCommand::Exit
            | SlashCommand::Summary
            | SlashCommand::Desktop
            | SlashCommand::Brief
            | SlashCommand::Advisor
            | SlashCommand::Stickers
            | SlashCommand::Insights
            | SlashCommand::Thinkback
            | SlashCommand::ReleaseNotes
            | SlashCommand::SecurityReview
            | SlashCommand::Keybindings
            | SlashCommand::PrivacySettings
            | SlashCommand::Review { .. }
            | SlashCommand::Tasks { .. }
            | SlashCommand::Theme { .. }
            | SlashCommand::Voice { .. }
            | SlashCommand::Usage { .. }
            | SlashCommand::Rename { .. }
            | SlashCommand::Copy { .. }
            | SlashCommand::Hooks { .. }
            | SlashCommand::Context { .. }
            | SlashCommand::Color { .. }
            | SlashCommand::Effort { .. }
            | SlashCommand::Branch { .. }
            | SlashCommand::Rewind { .. }
            | SlashCommand::Ide { .. }
            | SlashCommand::Tag { .. }
            | SlashCommand::OutputStyle { .. }
            | SlashCommand::AddDir { .. } => {
                eprintln!("Command registered but not yet implemented.");
                false
            }
            SlashCommand::Research { question } | SlashCommand::Ask { question } => {
                cli.fsm_transition_sync(elai_orchestrator::FsmState::Researching);
                match question {
                    Some(q) => {
                        // C-30: Oracle answer-level cache (lazy-init).
                        let oracle_cache = cli
                            .oracle_cache
                            .get_or_insert_with(|| {
                                Arc::new(elai_orchestrator::oracle_cache::OracleAnswerCache::new())
                            })
                            .clone();
                        let oracle_project_root = std::env::current_dir()
                            .unwrap_or_else(|_| PathBuf::from("."));
                        let oracle_key = elai_orchestrator::oracle_cache::CacheKey::compute(
                            &q,
                            &oracle_project_root,
                        );
                        // C-15: use injected ClockProvider so oracle-cache TTL
                        // checks are deterministic under ELAI_SEED replay.
                        let now_secs = cli.clock.now_secs();
                        if let Some(cached) = oracle_cache.get(&oracle_key, now_secs) {
                            println!(
                                "Research answer (cached, source_tier={}, {} sources):",
                                elai_orchestrator::oracle_cache::source_tier_str(cached.source_tier),
                                cached.answer.sources_consulted.len()
                            );
                            println!("{}", cached.answer.answer_text);
                            if let Some(ssb) = &cli.ssb {
                                let payload = elai_orchestrator::ssb::SsbPayload::OracleCacheHit(
                                    elai_orchestrator::ssb::OracleCacheHitPayload {
                                        query_hash_hex: oracle_key.query_hash_hex(),
                                        source_tier: elai_orchestrator::oracle_cache::source_tier_str(cached.source_tier).to_string(),
                                        cached_age_secs: cached.age_secs(now_secs),
                                    },
                                );
                                let ssb = ssb.clone();
                                if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                    rt.block_on(async move {
                                        if let Err(e) = ssb.write_payload(elai_orchestrator::task_runner::RUNNER_AGENT_ID, elai_orchestrator::task_runner::RUNNER_SUPERSTEP, payload, vec![]).await {
                                            tracing::warn!(error = %e, "C-30 oracle_cache_hit SSB emit failed");
                                        }
                                    });
                                }
                            }
                            cli.fsm_transition_sync(elai_orchestrator::FsmState::Idle);
                            return Ok(false);
                        }
                        // C-17 (DRIFT-N) Oracle pre-seeding: prefix the
                        // research query with a PROJECT_STARTING_POINTS
                        // block built from detect_entry_points. Closes
                        // B-34 MISSING gap. The injection is best-effort
                        // — if the symbol-graph DB is absent or empty we
                        // proceed with the unaugmented query.
                        let augmented = super::c17_research_oracle_prefix(&q, cli.ssb.as_ref());
                        let query = elai_core::ResearchQuery::new(&augmented);
                        let tier_cfg = match elai_core::parse_source_tier_config(EMPTY_TIER_TOML) {
                            Ok(cfg) => cfg,
                            Err(e) => {
                                eprintln!("Failed to load tier config: {e}");
                                cli.fsm_transition_sync(elai_orchestrator::FsmState::Idle);
                                return Ok(false);
                            }
                        };
                        // Default: hit the sidecar `/search` proxy (SearXNG/Brave-backed).
                        // Dev opt-out: ELAI_STUB_SEARCH=1 swaps in the in-memory stub for
                        // offline research-pipeline smoke tests. Falsy values
                        // (0/false/no/off/empty) explicitly keep the production backend.
                        let use_stub = std::env::var("ELAI_STUB_SEARCH")
                            .map(|v| {
                                let trimmed = v.trim();
                                !trimmed.is_empty()
                                    && !matches!(
                                        trimmed.to_ascii_lowercase().as_str(),
                                        "0" | "false" | "no" | "off"
                                    )
                            })
                            .unwrap_or(false);
                        let backend: Box<dyn elai_orchestrator::SearchBackend> = if use_stub {
                            tracing::info!("research: using StubSearchBackend (ELAI_STUB_SEARCH=1)");
                            Box::new(elai_orchestrator::StubSearchBackend::new())
                        } else {
                            let sidecar_url = std::env::var("ELAI_SIDECAR_URL")
                                .unwrap_or_else(|_| {
                                    elai_orchestrator::DEFAULT_SIDECAR_SEARCH_URL.to_string()
                                });
                            match elai_orchestrator::SearxSearchBackend::new(&sidecar_url) {
                                Ok(b) => {
                                    tracing::info!(
                                        sidecar_url = %sidecar_url,
                                        "research: using SearxSearchBackend"
                                    );
                                    Box::new(b)
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        error = %e,
                                        sidecar_url = %sidecar_url,
                                        "research: SearxSearchBackend construction failed, falling back to stub"
                                    );
                                    Box::new(elai_orchestrator::StubSearchBackend::new())
                                }
                            }
                        };
                        // D3: wire SidecarGroundingClient for LLM-backed Phase F synthesis.
                        // run_research_pipeline is sync (phases A-E use sync SearchBackend::search).
                        // Phase F async bridge is handled inside phase_f_synthesize_blocking
                        // (builds its own current-thread runtime; safe from this sync context).
                        let sidecar_base_url = std::env::var("ANTHROPIC_BASE_URL")
                            .or_else(|_| std::env::var("ELAI_SIDECAR_URL"))
                            .unwrap_or_else(|_| elai_orchestrator::grounding_client::SIDECAR_DEFAULT_HOST.to_string());
                        let grounding = elai_orchestrator::SidecarGroundingClient::new(
                            sidecar_base_url,
                            std::time::Duration::from_secs(elai_orchestrator::grounding_client::GROUNDING_TIMEOUT_SECS),
                        );
                        match elai_orchestrator::run_research_pipeline(&query, &tier_cfg, backend.as_ref(), &grounding) {
                            Ok(answer) => {
                                println!("Research answer ({} sources, {} caveats):",
                                    answer.sources_consulted.len(), answer.caveats.len());
                                println!("{}", answer.answer_text);
                                if !answer.sources_consulted.is_empty() {
                                    println!("\nSources:");
                                    for (i, s) in answer.sources_consulted.iter().enumerate() {
                                        println!("  [{}] {}", i + 1, s.url);
                                    }
                                }
                                if !answer.path.is_empty() {
                                    if let Some(ssb) = &cli.ssb {
                                        use elai_orchestrator::ssb::SsbPayload;
                                        // C-15: OsClock-backed `ts_ms`. Research-path SSB rows are
                                        // observation timestamps; not part of any deterministic test
                                        // fixture, so plain OsClock is sufficient.
                                        let ts_ms = u64::try_from(OsClock.now_ms()).unwrap_or(u64::MAX);
                                        let payload = SsbPayload::ResearchPathChosen(
                                            elai_orchestrator::ResearchPathChosenPayload {
                                                path: answer.path.clone(),
                                                reason: "synthesis".to_string(),
                                                ts_ms,
                                            },
                                        );
                                        let ssb = ssb.clone();
                                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                            rt.block_on(async {
                                                if let Err(e) = ssb.write_payload("research", 0, payload, vec![]).await {
                                                    tracing::warn!(error = %e, "research: failed to emit research_path_chosen SSB row");
                                                }
                                            });
                                        }
                                    }
                                }

                                // C-30: cache the answer using the
                                // confidence -> tier mapping. T4 / Unknown
                                // are dropped by `OracleAnswerCache::insert`
                                // itself. Emit `oracle_cache_miss` regardless
                                // so the dashboard sees one row per pipeline
                                // run that bypassed the cache.
                                let tier = elai_orchestrator::oracle_cache::source_tier_for_confidence(answer.confidence);
                                let cached_entry = elai_orchestrator::oracle_cache::CachedAnswer {
                                    answer: answer.clone(),
                                    cached_at: now_secs,
                                    source_tier: tier,
                                };
                                let inserted = oracle_cache.insert(oracle_key, cached_entry);
                                tracing::debug!(
                                    inserted = inserted,
                                    "C-30 oracle cache: insert"
                                );
                                if let Some(ssb) = &cli.ssb {
                                    let payload = elai_orchestrator::ssb::SsbPayload::OracleCacheMiss(
                                        elai_orchestrator::ssb::OracleCacheMissPayload {
                                            query_hash_hex: oracle_key.query_hash_hex(),
                                            confidence: elai_orchestrator::oracle_cache::confidence_str(answer.confidence).to_string(),
                                        },
                                    );
                                    let ssb = ssb.clone();
                                    if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                        rt.block_on(async move {
                                            if let Err(e) = ssb.write_payload(elai_orchestrator::task_runner::RUNNER_AGENT_ID, elai_orchestrator::task_runner::RUNNER_SUPERSTEP, payload, vec![]).await {
                                                tracing::warn!(error = %e, "C-30 oracle_cache_miss SSB emit failed");
                                            }
                                        });
                                    }
                                }
                            }
                            Err(e) => tracing::error!(error = %e, "research pipeline error"),
                        }
                    }
                    None => eprintln!("Usage: /research <question>"),
                }
                cli.fsm_transition_sync(elai_orchestrator::FsmState::Idle);
                false
            }
            SlashCommand::Explore { target, via_mcp } => {
                match target {
                    Some(t) => {
                        tracing::info!(
                            mode = "explore",
                            query = %t,
                            via_mcp = via_mcp,
                            "routed to explore mode"
                        );
                        if via_mcp {
                            // C-39: per-subcommand --via-mcp surface --
                            // when set, the downstream tool dispatch is
                            // routed through McpClientDispatch. Today the
                            // explore mode prints a placeholder; the
                            // FSM-level wiring lives on `elai run --via-mcp`
                            // (run_subcommand.rs). The flag is preserved
                            // here so future explore-mode wiring can read it.
                            tracing::info!("/explore --via-mcp acknowledged (dispatch path)");
                        }
                        println!("[explore] {t}");
                    }
                    None => eprintln!("Usage: /explore [--via-mcp] <question>"),
                }
                false
            }
            SlashCommand::Task { description, surgical, new_project, via_mcp } => {
                match description {
                    Some(d) => {
                        tracing::info!(
                            mode = "task",
                            surgical = surgical,
                            new_project = new_project,
                            via_mcp = via_mcp,
                            description = %d,
                            "routed to task mode (D2: DefaultWebResearchRunner)"
                        );
                        if via_mcp {
                            // C-39: per-subcommand --via-mcp acknowledgement.
                            // Same scope note as the explore arm above --
                            // the FSM wiring lives on `elai run --via-mcp`.
                            tracing::info!("/task --via-mcp acknowledged (dispatch path)");
                        }

                        // -------------------------------------------------------
                        // C-30: project detection signal scan (first /task only).
                        // -------------------------------------------------------
                        if !cli.project_detection_done {
                            let project_root = std::env::current_dir()
                                .unwrap_or_else(|_| PathBuf::from("."));
                            let indexes_ready = cli
                                .indexing_service
                                .as_ref()
                                .map(|s| s.indexes_exist())
                                .unwrap_or(false);
                            if !indexes_ready {
                                let signals =
                                    super::run_project_detection_signals(&project_root);
                                let any_required = super::c30_any_required_fired(&signals);
                                if let Some(ssb) = &cli.ssb {
                                    super::emit_project_detection_signals(ssb, &signals);
                                }
                                if any_required {
                                    tracing::info!(
                                        signals = signals.len(),
                                        "C-30 fresh-project detected; index build is recommended"
                                    );
                                    eprintln!(
                                        "[C-30] Detected new project. Build knowledge index? (rerun with `/index build` to confirm; proceeding without index for this task.)"
                                    );
                                }
                            }
                            cli.project_detection_done = true;
                        }

                        // -------------------------------------------------------
                        // C-30: surgical branch intercepts BEFORE
                        // DefaultWebResearchRunner. The default ConceptDecomposition
                        // pipeline is skipped and replaced by the SurgicalDecomposer
                        // output. When the DKG is unavailable or returns no
                        // matches, the resulting plan is empty (Simple, 0 LOC)
                        // but still emits the canonical SurgicalPlanCreated row
                        // so the dashboard / auditor observe the surgical-mode
                        // selection.
                        if surgical {
                            let project_root = std::env::current_dir()
                                .unwrap_or_else(|_| PathBuf::from("."));
                            let dkg_db_path_raw = core_dkg_db_path(&project_root);
                            let dkg_db_path = if dkg_db_path_raw.is_file() {
                                Some(dkg_db_path_raw.clone())
                            } else {
                                None
                            };
                            let dkg_querier: Arc<
                                dyn elai_rules::predicate::DkgQuerier + Send + Sync,
                            > = match dkg_db_path
                                .as_ref()
                                .and_then(|p| DkgQuerierAdapter::new(p).ok())
                            {
                                Some(adapter) => Arc::new(adapter),
                                None => Arc::new(EmptyDkgQuerier),
                            };
                            let decomposer = elai_orchestrator::surgical_decomposer::SurgicalDecomposer::new(
                                dkg_querier,
                                dkg_db_path,
                            );
                            let interactive_mode = elai_orchestrator::context_budget::ExecutionMode::Interactive;
                            // `confirmed` is wired from the CLI's --new flag
                            // (best-effort signal that the operator opted in).
                            // Production callers can flip this when the user
                            // answers the confirm prompt above.
                            let confirmed = new_project;
                            let mode_str = "interactive";
                            match decomposer.decompose(&d, interactive_mode, confirmed) {
                                Ok(plan) => {
                                    println!(
                                        "[task --surgical] plan: complexity={} files={} loc={} steps={}",
                                        plan.complexity_class.as_str(),
                                        plan.affected_set.blast_radius_files(),
                                        plan.affected_set.total_loc_estimate,
                                        plan.steps.len(),
                                    );
                                    if let Some(ssb) = &cli.ssb {
                                        let ssb = ssb.clone();
                                        let payload = elai_orchestrator::ssb::SsbPayload::SurgicalPlanCreated(
                                            elai_orchestrator::ssb::SurgicalPlanCreatedPayload {
                                                task_id: plan.task_id.clone(),
                                                complexity_class: plan.complexity_class.as_str().to_string(),
                                                blast_radius_files: plan.affected_set.blast_radius_files(),
                                                total_loc_estimate: plan.affected_set.total_loc_estimate,
                                                step_count: u32::try_from(plan.steps.len()).unwrap_or(u32::MAX),
                                                regression_gate_snapshot_hash: plan.regression_gate.snapshot_hash.clone(),
                                            },
                                        );
                                        if plan.affected_set.total_loc_estimate > elai_orchestrator::surgical_decomposer::SURGICAL_LOC_WARN_THRESHOLD {
                                            let warn_payload = elai_orchestrator::ssb::SsbPayload::SurgicalLocWarning(
                                                elai_orchestrator::ssb::SurgicalLocWarningPayload {
                                                    task_id: plan.task_id.clone(),
                                                    total_loc_estimate: plan.affected_set.total_loc_estimate,
                                                    threshold: elai_orchestrator::surgical_decomposer::SURGICAL_LOC_WARN_THRESHOLD,
                                                    mode: mode_str.to_string(),
                                                },
                                            );
                                            let ssb_warn = ssb.clone();
                                            let task_id_warn = plan.task_id.clone();
                                            if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                                rt.block_on(async move {
                                                    if let Err(e) = ssb_warn.write_payload(elai_orchestrator::task_runner::RUNNER_AGENT_ID, elai_orchestrator::task_runner::RUNNER_SUPERSTEP, warn_payload, vec![]).await {
                                                        tracing::warn!(error = %e, task_id = %task_id_warn, "C-30 surgical_loc_warning SSB emit failed");
                                                    }
                                                });
                                            }
                                        }
                                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                            rt.block_on(async move {
                                                if let Err(e) = ssb.write_payload(elai_orchestrator::task_runner::RUNNER_AGENT_ID, elai_orchestrator::task_runner::RUNNER_SUPERSTEP, payload, vec![]).await {
                                                    tracing::warn!(error = %e, "C-30 surgical_plan_created SSB emit failed");
                                                }
                                            });
                                        }
                                    }
                                    cli.fsm_transition_sync(elai_orchestrator::FsmState::Idle);
                                    return Ok(false);
                                }
                                Err(elai_orchestrator::surgical_decomposer::SurgicalError::ConfirmationRequired { loc, threshold }) => {
                                    eprintln!(
                                        "[task --surgical] blast radius {loc} LOC exceeds {threshold}; rerun with `--new` to confirm."
                                    );
                                    if let Some(ssb) = &cli.ssb {
                                        let payload = elai_orchestrator::ssb::SsbPayload::SurgicalLocWarning(
                                            elai_orchestrator::ssb::SurgicalLocWarningPayload {
                                                task_id: elai_orchestrator::surgical_decomposer::generate_task_id(&d),
                                                total_loc_estimate: loc,
                                                threshold,
                                                mode: "interactive".into(),
                                            },
                                        );
                                        let ssb = ssb.clone();
                                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                            rt.block_on(async move {
                                                let _ = ssb.write_payload(elai_orchestrator::task_runner::RUNNER_AGENT_ID, elai_orchestrator::task_runner::RUNNER_SUPERSTEP, payload, vec![]).await;
                                            });
                                        }
                                    }
                                    cli.fsm_transition_sync(elai_orchestrator::FsmState::Idle);
                                    return Ok(false);
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "C-30 surgical decomposition failed; falling back to default task runner");
                                    eprintln!("[task --surgical] decomposition error: {e}; falling back to default pipeline");
                                }
                            }
                        }

                        // D2: wire DefaultWebResearchRunner — mirror dashboard/src/lib.rs:798-806.
                        // Build a one-shot tokio runtime (same pattern as fsm_transition_sync,
                        // cli/main.rs:3622) to block on the async runner.run() future.
                        let sidecar_url = std::env::var("ELAI_SIDECAR_URL")
                            .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
                            .unwrap_or_else(|_| {
                                elai_orchestrator::DEFAULT_SIDECAR_SEARCH_URL.to_string()
                            });

                        let tier_config = elai_orchestrator::research_tier::research_config_for_tier(
                            elai_core::VramTier::Vram12Gb,
                        );
                        let runner = elai_orchestrator::DefaultWebResearchRunner::new(
                            tier_config,
                            sidecar_url.clone(),
                            cli.clock.clone(),
                        );
                        // F-3p1 / Slice C3 (PRODUCER bootstrap): share the
                        // process-global committed-tool-set slot so the
                        // per-node commit is visible to ENFORCE-A (Fsm) and
                        // ENFORCE-B (CapabilityFlowAdapter), and attach the SSB
                        // writer so commit emits a tool_filter_committed row.
                        let runner = runner
                            .with_committed_tool_set(elai_orchestrator::committed_slot());
                        let runner = match &cli.ssb {
                            Some(ssb) => runner.with_ssb_writer(ssb.clone()),
                            None => runner,
                        };

                        // Build emitter: SSB-backed when SSB is wired, noop otherwise.
                        let emitter: std::sync::Arc<
                            dyn elai_orchestrator::web_research_orchestrator::WebResearchStepEmitter,
                        > = match &cli.ssb {
                            Some(ssb) => std::sync::Arc::new(
                                elai_orchestrator::SsbWebResearchStepEmitter::new(ssb.clone()),
                            ),
                            None => std::sync::Arc::new(NoopStepEmitter),
                        };

                        let rt = match tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            Ok(rt) => rt,
                            Err(e) => {
                                eprintln!("[task] failed to build async runtime: {e}");
                                return Ok(false);
                            }
                        };

                        use elai_orchestrator::WebResearchRunner;
                        match rt.block_on(runner.run(d.clone(), emitter)) {
                            Ok(summary) => {
                                println!(
                                    "[task] research complete: status={} iterations={} sources_used={}",
                                    summary.status, summary.iterations, summary.sources_used
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    error = %e,
                                    sidecar_url = %sidecar_url,
                                    "task: web-research runner failed"
                                );
                                eprintln!("[task] web-research failed: {e}");
                            }
                        }
                    }
                    None => eprintln!("Usage: /task [--surgical|--new] <description>"),
                }
                false
            }
            SlashCommand::Mode { mode } => {
                // B-22 Phase 4G: /mode is a runtime gate-mode switch handled
                // by the dashboard's set_gate_mode IPC. From the live CLI
                // we surface a hint pointing at the dashboard's controls;
                // headless flips happen by setting ELAI_GATE_MODE before
                // bringing up the runner.
                let trimmed = mode.trim();
                if trimmed.is_empty() {
                    eprintln!(
                        "Usage: /mode <interactive|autonomous>  (route the value through the dashboard set_gate_mode IPC)"
                    );
                } else if elai_orchestrator::GateMode::from_str(trimmed).is_none() {
                    eprintln!(
                        "/mode: unknown gate mode `{trimmed}`; expected `interactive` or `autonomous`"
                    );
                } else {
                    eprintln!(
                        "/mode {trimmed}: live-CLI flip is handled by the dashboard. Use the dashboard mode toggle, or set ELAI_GATE_MODE={trimmed} before launching the runner."
                    );
                }
                false
            }
            SlashCommand::Reason { on_off } => {
                // B-39: toggle the Chat REFLECT pipeline. Mirrors /mode: string-typed,
                // validation in handler, zero orchestrator dep.
                let trimmed = on_off.trim();
                if trimmed.eq_ignore_ascii_case("on") {
                    cli.reflect_enabled = true;
                    tracing::info!("REFLECT enabled");
                    eprintln!("[reason] Chat REFLECT pipeline enabled");
                } else if trimmed.eq_ignore_ascii_case("off") {
                    cli.reflect_enabled = false;
                    cli.reflect_pipeline = None;
                    tracing::info!("REFLECT disabled");
                    eprintln!("[reason] Chat REFLECT pipeline disabled");
                } else {
                    tracing::warn!(
                        on_off = %trimmed,
                        "/reason: unrecognised value; expected `on` or `off`"
                    );
                    eprintln!("/reason: unrecognised value `{trimmed}`; expected `on` or `off`");
                }
                false
            }
            SlashCommand::EntryPoints { project_root } => {
                // B-34 BLOCK-4 fix: implement /entry_points handler.
                // Resolves project root from argument or cwd, loads SymbolGraphCache,
                // runs detect_entry_points, and prints one line per entry.
                use elai_knowledge::symbol_graph::{
                    cache::SymbolGraphCache,
                    entry_points::{detect_entry_points, EntryPointReason},
                    sharding::SYMBOL_FILE_LRU_SIZE,
                };
                use elai_core::paths::dkg_db_path as ep_dkg_db_path;

                let root: PathBuf = project_root
                    .as_deref()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let project_id = elai_core::project_id_from_path(&root);
                let db_path = ep_dkg_db_path(&root);

                if !db_path.exists() {
                    eprintln!(
                        "/entry_points: no symbol graph DB found at {}. Run `elai symbol_graph build` first.",
                        db_path.display()
                    );
                } else {
                    match SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE) {
                        Err(e) => eprintln!("/entry_points: cache open failed: {e}"),
                        Ok(mut cache) => {
                            match cache.ensure_reverse_index() {
                                Err(e) => eprintln!("/entry_points: reverse_index load failed: {e}"),
                                Ok(reverse_index) => {
                                    let reverse_index = reverse_index.clone();
                                    let mut all_files: std::collections::BTreeSet<String> =
                                        reverse_index.keys().cloned().collect();
                                    for callers in reverse_index.values() {
                                        all_files.extend(callers.iter().cloned());
                                    }
                                    const EP_CODE_EXT: &[&str] = &[
                                        "ts", "tsx", "js", "jsx", "py", "rb", "go", "rs",
                                        "java", "kt", "php", "c", "cpp", "cs",
                                    ];
                                    let mut payloads = Vec::new();
                                    for file in all_files.iter().filter(|f| {
                                        std::path::Path::new(f.as_str())
                                            .extension()
                                            .and_then(|e| e.to_str())
                                            .map_or(false, |e| EP_CODE_EXT.contains(&e))
                                    }) {
                                        match cache.get_or_load_payload(file) {
                                            Ok(Some(arc)) => payloads.push((*arc).clone()),
                                            Ok(None) | Err(_) => {}
                                        }
                                    }
                                    let entries = detect_entry_points(&reverse_index, &payloads);
                                    if entries.is_empty() {
                                        println!("No entry points detected.");
                                    } else {
                                        println!("Entry points ({}):", entries.len());
                                        for e in &entries {
                                            let reason = match &e.reason {
                                                EntryPointReason::Orphan => "orphan",
                                                EntryPointReason::WellKnownName(_) => "well_known",
                                                EntryPointReason::FrameworkRoute(_) => "framework",
                                                EntryPointReason::Test => "test",
                                            };
                                            let line_str = e.line.map_or_else(|| "?".to_string(), |l| l.to_string());
                                            println!("  {}:{} {} ({})", e.file, line_str, e.name, reason);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                false
            }
            // C-24 — /graph mermaid [--mode file|symbol|decomposition]
            //                       [--max-nodes N] [project-root]
            //
            // Deterministic, zero-LLM: delegates to the same generator the
            // dashboard Tauri command uses (knowledge::visualization::mermaid).
            // `decomposition` mode is NOT reachable here because it needs the
            // dashboard's in-memory DecompositionTreeSnapshot; we surface a
            // clear error pointing users at the Tauri command in that case.
            SlashCommand::GraphMermaid { mode, max_nodes, project_root } => {
                let root: PathBuf = project_root
                    .as_deref()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let project_id = elai_core::project_id_from_path(&root);
                let data_result = match mode.as_str() {
                    "file" => elai_knowledge::visualization::views::build_file_view(
                        &root, &project_id,
                    ),
                    "symbol" => elai_knowledge::visualization::views::build_symbol_view(
                        &root, &project_id,
                    ),
                    "decomposition" => {
                        eprintln!(
                            "/graph mermaid --mode decomposition: not supported via CLI \
                             (requires the dashboard's in-memory DecompositionTreeSnapshot). \
                             Use the dashboard GraphWindow `Export Mermaid` button instead."
                        );
                        return Ok(false);
                    }
                    other => {
                        eprintln!(
                            "/graph mermaid: unknown mode `{other}`; \
                             expected file|symbol|decomposition"
                        );
                        return Ok(false);
                    }
                };
                match data_result {
                    Ok(data) => {
                        let mermaid = elai_knowledge::visualization::mermaid::generate_mermaid(
                            &data, max_nodes,
                        );
                        // Plain stdout — pipes into the deterministic check
                        // script (the spec greps for "^graph " / "^flowchart ").
                        println!("{mermaid}");
                    }
                    Err(e) => eprintln!("/graph mermaid: build failed: {e}"),
                }
                false
            }
            // C-20: `/measure` from the REPL surfaces the same metrics
            // backfill behaviour as the direct-CLI invocation. The handler
            // owns its own tokio runtime via `run_measure_subcommand`, so
            // we can call it from this sync REPL context.
            SlashCommand::Measure { scope, cloud, output_format: _ } => {
                let project_root = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."));
                if let Err(e) = crate::measure::run_measure_subcommand(scope, cloud, project_root) {
                    eprintln!("/measure failed: {e}");
                }
                false
            }
            // C-25: REPL handler for /pipeline. Dispatches to the same
            // synchronous handler used by `elai pipeline …`.
            SlashCommand::Pipeline { action } => {
                let project_root = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."));
                if let Err(e) = crate::pipeline_subcommand::run_pipeline_subcommand(
                    action.as_deref(),
                    &project_root,
                ) {
                    eprintln!("/pipeline failed: {e}");
                }
                false
            }
            // C-27: REPL handler for /wiring audit. Dispatches to the same
            // sync handler used by `elai wiring audit …`. Text output mode
            // by default in the REPL (operators can re-run from the shell
            // for JSON / TOML output).
            SlashCommand::WiringAudit { scope, config_path, contracts_dir } => {
                if let Err(e) = crate::wiring_audit::run(
                    scope,
                    config_path,
                    contracts_dir,
                    crate::CliOutputFormat::Text,
                ) {
                    eprintln!("/wiring audit failed: {e}");
                }
                false
            }
            SlashCommand::Unknown(name) => {
                eprintln!("{}", format_unknown_slash_command(&name));
                false
            }
        })
}
