C-9 — Autonomous Mode Core Infrastructure: Decisions

Session: s-32952
Date: 2026-05-18
Scope: docs/thefinalglance/C-plan/C-9_autonomous_infrastructure.md (T1-T15 + D1-D10 drift items from context-package-C9.md)

DECISION-A — Production wiring chokepoint is apply_task_runner_wiring, not cli/main.rs

Context: C-9 plan §Task 10 named cli/main.rs as the wiring target ("cli/main.rs still passes autonomous_gate: None"). The drift report D1 found this is incorrect — cli/main.rs has no TaskRunnerConfig construction. The two production callers (cli/run_subcommand.rs:268 and dashboard-v2/src-tauri/src/lib.rs:593) both route through the orchestrator-level chokepoint apply_task_runner_wiring(cursor_path, &wiring).

Choice: extended RuntimeWiring with gate_mode: Option<GateMode>. apply_task_runner_wiring inspects the field and constructs Some(Arc<AutonomousGate>) when populated. Both callers opt in. The chokepoint approach satisfies both production paths with one edit.

Rejected: per-caller construction in run_subcommand.rs and dashboard lib.rs separately. Would double the wiring code and break the contract-pinned chokepoint invariant.

DECISION-B — MergeStrategy ships only Sequential; Parallel deferred to C-10

Context: C-9 plan §Task 6 enumerated one variant (Sequential) with a "Parallel deferred until C-10 anti-spiral fan-out is stable" note.

Choice: MergeStrategy enum has a single variant (Sequential) with #[allow(unused)] not applied; the variant is reachable through every constructor. C-10's anti-spiral primitives provide the stagnation signals the gate needs to safely arbitrate parallel fan-out (GoalDriftCanary + StagnationPattern). Until those land, Sequential is the only verifiable merge semantics.

Rejected: shipping Parallel as a stub. Stub-ban (WIRING_CONTRACT.stub_ban) forbids ResearchTree::split_step returning empty for non-empty plans, and a stub Parallel variant would have no validated execution path.

DECISION-C — cli/main.rs wiring passes None for interactive_gate

Context: C-9 plan §Task 10 prescribed AutonomousGate::new(cfg, None) — the second arg is interactive_gate: Option<Arc<dyn InteractiveGate>>.

Choice: keep None at the wiring site for both `elai run` (headless CLI) and dashboard-v2 setup. Headless cloud tasks do not route through InterventionCard, and the dashboard exposes its own InterventionCard via Tauri commands (set_gate_mode, intervention_resolve). When the gate emits RequestHuman in the dashboard wiring, the task_runner's existing run_confirmation_gate handles the modal rendering; the gate's interactive_gate slot would duplicate that path.

Rejected: routing a real InteractiveGate impl in. Would force the orchestrator-side wiring to know about the dashboard's IPC channel, breaking the production decoupling that B-22 established.

DECISION-D — EscalationPolicy is the source of truth; circuit_breaker_limit kept for backward compat

Context: C-9 plan §Task 4 says "keep circuit_breaker_limit for backward compat; max_auto_advances_per_phase mirrors it via Default". The D9 drift item flagged the duplication risk (two sources of truth for the same number).

Choice: EscalationPolicy::default() returns max_auto_advances_per_phase=10. AutonomousGateConfig::default() seeds both circuit_breaker_limit and escalation_policy.max_auto_advances_per_phase from the same EscalationPolicy::default() instance, so they cannot drift at construction. The decide() function still reads circuit_breaker_limit (the legacy hot path) — future C-N refactor should migrate decide() to read escalation_policy.max_auto_advances_per_phase exclusively and deprecate the legacy field. Pinned by `c9_contract_escalation_policy_default_mirrors_legacy_limit`.

Rejected: removing circuit_breaker_limit immediately. Breaking change with no migration path for existing AutonomousGateConfig construction sites (3 test sites + 1 dashboard probe).

DECISION-E — ModelPool::can_fit_llm is a trait default, not a required method

Context: C-9 plan §Task 9 was explicit about the trait-default approach.

Choice: implemented as `fn can_fit_llm(&self, model_size_bytes: u64) -> bool { self.available_vram() >= model_size_bytes }` with default body on the trait. No existing impl (production runtime_model_pool, 16 mocks across orchestrator + contracts + bsp/engine + preload + tests) needs to change. The autonomous_gate consumes it through GateContext.model_pool: Option<Arc<dyn ModelPool>>.

Rejected: making it a required method. Would force 16 impl updates across 12 files and break every contract test that constructs a MockModelPool inline. The default body is the documented intent (phase-4G.txt §4G.5 line 122-124).

DECISION-F — ResearchTree handle threaded into task_runner via TaskRunnerConfig.research_tree

Context: D7 drift item flagged that ResearchTree was not plumbed into task_runner.rs SplitTask dispatch. The original plan acknowledged this with a "fall back to SplitTaskFailed escalation" path.

Choice: added `research_tree: Option<Arc<tokio::sync::RwLock<ResearchTree>>>` to TaskRunnerConfig and ResolvedConfig. emit_autonomous_decision() now takes the handle as a parameter. When Some + ResearchTree::split_step returns empty for a non-empty plan, the dispatch emits EscalationTrigger::SplitTaskFailed AND writes a deterministic TaskSplitPayload so SSB consumers still see the attempted split count. RwLock (not AsyncMutex) because split_step is &self and pure CPU. Pinned by `c9_contract_research_tree_split_step_realises_plan` + `c9_contract_research_tree_split_step_empty_on_empty_plan` + `c9_contract_research_tree_split_step_caps_at_max_depth`.

Rejected: skipping the threading. Acceptance bullet "task_runner SplitTask arm calls split_step" would be hollow per D7. Adding the field today costs one TaskRunnerConfig field; future C-N can populate it from the live ResearchTree instance the orchestrator builds for cosmos-tier research.

DECISION-G — phase_4G.toml [runtime_measurements] populated with placeholders + measured-in: C-20 markers

Context: C-9 plan §Task 13 required populating the 8 runtime measurement keys. Real measurements require a smoke run against cloud Sonnet 4.6 in --mode autonomous which is out of scope for this session (needs sidecar uptime + live cloud auth).

Choice: populated all 8 keys with documented placeholder zeros, each carrying a `# measured-in: C-20` comment marker. The escalation_count_per_session field is populated with the in-session measurable value (3 — counts the C-9 contract tests that assert an Escalate verdict). completion=false marks the section as placeholder-state so downstream tooling can detect and re-run after C-20. Two extra fields (c9_contract_pass_count=15 + c9_unit_test_pass_count=19 + total_pass_count_2026_05_18=101) record the in-session pass counts as ground-truth.

Rejected: leaving the section absent. Acceptance bullet "metrics/phase_4G.toml [runtime_measurements] populated with all 8 keys" requires a section. Inventing synthetic non-zero values would violate Grounded-by-default (no measured source).

DECISION-H — Reuse B-37's [agents.<role>.runtime] stanza; no new [agents.<role>.backend] section

Context: D2 drift item flagged that C-9 plan §Task 11 prescribed [agents.<role>.backend] which collides with B-37's already-shipped [agents.<role>.runtime] backend = "<canonical>" structure (covering 11 RuntimeBackend variants). Operator instruction in the C-9 prompt: "REUSE B-37's [agents.<role>.runtime] stanza with backend = '<canonical>'. Do NOT create a competing [agents.<role>.backend] stanza."

Choice: did NOT add a new BackendStanza struct or [agents.<role>.backend] section. The B-37 reader (runtime_cmd.rs) is the single source of truth for per-role backend selection; the autonomous-mode backend choice (after SubstituteModel) reads from the same B-37 routing map. No runtime_config.rs changes; no elai_cc.toml.example file creation. The C-9 plan bullet for §Task 11 is consciously not satisfied as written; the autonomous-mode wiring consumes B-37's existing config instead.

Rejected: creating a parallel [agents.<role>.backend] table. Would create two separate sources of truth for the same concept (which backend serves which agent role) and confuse the dashboard's RuntimePanel UI.

DECISION-I — SubstituteModel guard: VRAM gate via ctx.model_pool, not on AutonomousGate itself

Context: D8 drift item asked which type carries the ModelPool handle. Three options: AutonomousGate field, AutonomousGateConfig field, GateContext field.

Choice: put it on GateContext (per-call inputs). Keeps the gate stateless w.r.t. provider/runtime concerns — every caller supplies their own pool snapshot via build_dispatch_context. The estimated model sizes live on AutonomousGateConfig.estimated_model_sizes: HashMap<String, u64> because they are config-time data (the gate owner knows fallback chain sizes at construction); the LIVE VRAM number is per-call.

Rejected: AutonomousGate field. Would force the gate to either (a) hold an Arc<dyn ModelPool> for its lifetime (couples the gate to provider lifecycle), or (b) take an Arc parameter on decide() (clutters the signature). The GateContext field is invisible to callers that don't opt in (defaults to None).

DECISION-J — --mode CLI flag on `elai run` (not on the REPL/dashboard binary)

Context: C-10 prompt directed addition of `--mode autonomous|interactive|hybrid` to args.rs. The args.rs parser dispatches multiple subcommands; --mode only makes sense for `run` (the task_runner subcommand) because REPL/Dashboard/Local/Prompt do not construct a TaskRunnerConfig.

Choice: added --mode to parse_run_subcommand_args (the `elai run` parser) and as a field on CliAction::Run. Validation at parse time accepts only autonomous|interactive|hybrid; unknown labels return a typed parse error. The resolved string is plumbed through RunArgs.mode -> run_subcommand.rs -> RuntimeWiring.gate_mode via GateMode::from_cli_args. Hybrid resolves to Interactive at the gate (the gate has 2 modes today); hybrid is enforced at the dispatcher tier going forward.

Rejected: making --mode a top-level flag. Would attach to every subcommand including ones (login, doctor, dep-audit) that have no concept of a gate mode. Scoped to the run subcommand keeps the surface tight.

Implementation summary
- autonomous_gate.rs: +EvidenceVerdict, EvidenceCheck, AutoAdvanceEvaluator, EscalationPolicy, MergeStrategy, SubTask, SplitTaskPlan, FsmTransition. EscalationTrigger gained SubstituteModelFailed + SplitTaskFailed (4 -> 6 variants). AutonomyDecision::SplitTask payload refactored from bare {sub_task_count, depth} to {split_plan: SplitTaskPlan}. AutonomousGateConfig gained escalation_policy + estimated_model_sizes. GateContext gained model_pool: Option<Arc<dyn ModelPool>>. GateMode gained from_cli_args. Decide() consults can_fit_llm + emits SubstituteModelFailed on no-fallback / oversize. Tests: +19 in-module.
- agent_pool.rs: ModelPool trait gained can_fit_llm default method.
- research_tree.rs: ResearchTree gained split_step method (stub-ban compliant — empty on empty/at-cap, never wrong shape).
- task_runner.rs: TaskRunnerConfig + ResolvedConfig gained research_tree field. emit_autonomous_decision threads ResearchTree handle and prefers it over the legacy phase_sizing splitter when wired. SplitTaskFailed escalation row emitted when split_step returns empty for non-empty plan.
- runtime_setup.rs: RuntimeWiring gained gate_mode: Option<GateMode>. apply_task_runner_wiring installs AutonomousGate when Some(_). This is the C-9 wiring chokepoint per DECISION-A.
- args.rs: --mode autonomous|interactive|hybrid added to `elai run` parser + CliAction::Run carries mode: Option<String>.
- run_subcommand.rs: RunArgs.mode plumbed through to wiring.gate_mode via GateMode::from_cli_args.
- dashboard-v2/src-tauri/src/lib.rs: wiring.gate_mode = Some(GateMode::Interactive) so dashboard always installs the gate (users flip to Autonomous at runtime via /mode).
- elai_orchestrator/src/lib.rs: re-exports added for AutoAdvanceEvaluator, EscalationPolicy, EvidenceCheck, EvidenceVerdict, FsmTransition, MergeStrategy, SplitTaskPlan, SubTask. EscalationPolicy from recovery_recipes renamed to RecoveryEscalationPolicy to disambiguate.
- contracts/tests/c9_autonomous_infra_contract.rs: 15 new contract tests.
- contracts/tests/autonomous_gate_contract.rs + phase_4g_extended_contract.rs: added model_pool: None to ctx_at helpers.
- contracts/tests/agent_pool_executor_wiring_contract.rs + 7 other contract tests: added research_tree: None + gate_mode: None to RuntimeWiring/TaskRunnerConfig literals.
- metrics/phase_4G.toml: [runtime_measurements] section added with C-20 markers.

Test counts (2026-05-18)
- elai-orchestrator autonomous_gate:: unit tests:  29 PASS (10 existing + 19 C-9 new).
- elai_cc_contracts c9_autonomous_infra_contract:  15 PASS.
- elai_cc_contracts autonomous_gate_contract:       14 PASS (unchanged).
- elai_cc_contracts phase_4g_extended_contract:     18 PASS (unchanged).
- elai_cc_contracts mode_switch_contract:            7 PASS (unchanged).
- elai_cc_contracts autonomous_gate_with_framing:    7 PASS (unchanged).
- elai_cc_contracts autonomous_gate_with_per_level: 11 PASS (unchanged).
Total: 101 PASS, 0 FAIL.
cargo check --workspace: PASS (zero new errors).

Residuals deferred to C-20 (live smoke test)
- runtime_measurements 8 keys carry placeholder zeros; the 6 latency keys
  require a smoke run with `elai run --mode autonomous` against cloud
  Sonnet 4.6. completion=false until then.
- AutoAdvanceStats dashboard panel populates from live SSB rows once a
  task_runner instance generates gate_decision rows in autonomous mode —
  the contract test asserts the gate field is wired; the live row stream
  is end-to-end-tested in C-20.
