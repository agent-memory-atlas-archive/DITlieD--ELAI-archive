PHASE 4E.11-4E.12 DECISION DOC
Session: s-1311948
Date: 2026-04-17

SCOPE
4E.11: integration checkpoint EXECUTION at 4 levels (baby_step/step/phase/concept).
4E.12: measure_4e_tree.rs binary + append [decomp_tree] metrics to phase_4E.toml via real sidecar.
Deferred (user directive): FileLineTracker, skeleton-first generation, fill protocol, mini-critic, phase sizing, bridge file templates.

KEY DECISIONS

D1 Integration checkpoint execution model
Approach: add IntegrationRunner trait (object-safe) with run_checkpoint(level, spec) -> Result<IntegrationCheckpoint>.
Default impl: CommandIntegrationRunner that executes ExitCriterion.check_command via std::process::Command with per-criterion timeout (30s default, named const).
Inject via DepthFirstNavigator::with_runner(Arc<dyn IntegrationRunner>).
Reason: keeps tests deterministic (mock runner) AND permits real-command execution in measure binary.
Expected: 3 new tests per level (step/phase/concept) = 9 tests minimum.

D2 When checkpoints fire
Baby: after each baby step result recorded, if spec has non-empty exit_criteria AND !deferred.
Step: when all baby steps in step are terminal (Pass/Skipped/FailBubbleUp) AND StepNode has integration spec.
Phase: when all steps in phase are terminal AND PhaseNode has integration spec.
Concept: when all phases in concept are terminal AND concept has integration spec.
Gate via NavigatorTree helpers (all_baby_steps_terminal_in_step/phase/concept).

D3 BubbleUp on integration fail
Fail at level L -> BubbleUpAction per BubbleUpPolicy at L (existing policy).
Record BubbleUpEvent with level=L. No new variants of BubbleUpAction (HARD LOCK: 6 only).

D4 SSB wiring
Integration result -> ResearchStep payload with project_id scoping.
Defer actual SsbWriter call behind feature flag in state; emit IntegrationCheckpoint + BubbleUpEvent in state log; add optional SsbWriter sink to TreeNavigatorState (Arc<SsbWriter>, None by default).
Reason: unit tests must not need Postgres; real measure binary can supply writer.

D5 Sidecar call strategy (4E.12)
Endpoint: /v1/messages (sidecar speaks Anthropic protocol per MEMORY.md).
Port: ELAI_SIDECAR_URL env var, default http://127.0.0.1:8100 (grep'd server.py SIDECAR_PORT_DEFAULT=8100).
Timeout: 5s connect + 30s read.
Fallback: if unreachable (connect refused OR timeout) -> byte/4 estimate, log tracing::warn!, continue.
No new dep: reqwest already in workspace (verified before use).

D6 Metrics aggregation
Synthetic tree: 3 phases x 3 steps x 3 baby steps = 27 baby steps per idea.
Run DepthFirstNavigator end-to-end 5 iterations (for p50/p95 distribution) with mock results driven by seeded RNG.
Collect: baby_steps_per_idea avg/median/max; fsm_cycles_per_idea (count of TESTING_FIRST->EXECUTING->AUDITING transitions per idea); bubble_up_rate; integration_pass_rate; traversal_overhead (navigator-only wall time / total); context_p50/p95 (per-baby-step prompt tokens via sidecar); total_execution_time_per_idea; cache_effectiveness (hit ratio from ResearchCache stub).

D7 TOML append strategy
Parse existing phase_4E.toml with toml crate; if [decomp_tree] absent, append; if present, UPDATE in-place (idempotent).
Preserve existing sections verbatim via re-serialize.
Deterministic check: grep "baby_steps_per_idea|bubble_up_rate|context_p50" >= 3.

UNKNOWNS
- Whether sidecar is running at measure time — fallback covers this.
- Whether PhaseNode/ConceptNode have IntegrationSpec fields — grep before use (CONTEXT_PACKAGE_FIELD_NAMES_DRIFT gotcha).
- Whether BabyStepSpec.integration is Option<IntegrationSpec> — grep before use.

EXPECTED TEST OUTCOME
cargo test -p elai-orchestrator --lib -- pipeline_tree_navigator: 42+ passing (33 existing + 9 new minimum).
cargo check --workspace: clean.
Measure binary: runs to completion, phase_4E.toml gains [decomp_tree] section, deterministic grep >= 3 matches.

RISKS
R1 Field name drift: mitigated by grep pre-check (D5 unknown).
R2 Sidecar down: mitigated by fallback (D5).
R3 New dep pull: reqwest verified already present.
