# MVP Wiring Gaps — Comprehensive Audit

Single-source-of-truth list of every subsystem that is scaffolded but not wired to a production call path in the `.elai_cc/` tree, based on a full static sweep performed on 2026-04-17. The goal is to let you close these gaps in coherent work sessions instead of discovering them one by one.

## Methodology

The sweep combined three passes:

1. **Grep sweep.** Patterns searched in `.elai_cc/crates/**/*.rs` and `.elai_cc/python/**/*.py`: `todo!()`, `unimplemented!()`, raw `panic!(` outside tests, Python `pass`/`raise NotImplementedError`, and the deferral-comment corpus (`"in production"`, `"would happen via"`, `"real implementation"`, `"real LLM"`, `"deferred"`, `"out of.*scope"`, `"not yet"`, `"for now"`, `"proper.*later"`, `"stopgap"`, `"stub"`, `"synthetic"`, `"placeholder"`, `"mock"`). All non-test hits were triaged.
2. **Caller-graph analysis.** For each `pub struct` / `pub fn` of interest, I grepped every construction site and marked it `PRODUCTION`, `TEST-ONLY` (only referenced under `#[cfg(test)]` or in `tests/`), or `UNREFERENCED`. This is how the SidecarProvider, CheckpointManager, RecoveryManager, BspEngine, SupervisionManager, etc. cases were confirmed.
3. **Spec-vs-code delta.** Walked `docs/specs/guide/steps/phase-index-3A-4J.txt` for every phase's deliverables and checked the corresponding identifiers in code (e.g. phase 4F FeasibilityProbe / SystemProbe — absent; phase 4G AutonomousGate — absent; phase 4H self-modification — absent; phase 4I media validator — absent; phase 4J Empire integration — absent).

Triage axes: **Severity** (P0 breaks MVP demo / user-visible functional failure; P1 breaks production quality; P2 cosmetic/dead code), **Confidence** (HIGH = grep-verified + caller analysis; MED = one-signal; LOW = spec says X, absent in code), **Effort** (S ≤1 session; M 2-3 sessions; L 4+ sessions).

Limitations: I did not execute the code, did not compile it, and did not consult test coverage reports. "Test-only caller" means the only `::new(` / `::{` construction-site I could find was under `#[cfg(test)]` or in a `tests/` directory. If a binary wires something via a dynamic name (e.g. reflection, plugin loading), I would have missed it — none are used in this tree, so this is a low-risk assumption. Any "dashboard wires X" claim excludes the Tauri frontend TS code which I did not audit; frontend gaps are out of scope.

## Already fixed (excluded)

- **patch-1.30 (commit 8ce26a0)** — task_runner consumes TaskSubmitted; OllamaProvider `todo!()` resolved; `elai.bat` dashboard default; dashboard runtime (tokio + canonical ssb path); WebValidator `DevServerLauncher` trait + `PreviewAvailable` emission.
- **patch-1.31 (commit b7d1414)** — `ProjectsStoreResolver`, `project_id` unification via `elai_core::project_id` (blake3).
- **Sidecar model path fix** — `python/elai_inference/server.py:2338` and `cli/src/main.rs:2558` (user-applied).
- **Two AgentHandle compile fixes** — `contracts/tests/checkpoint_contract.rs`, `contracts/tests/recovery_contract.rs`.
- **Session 4D.11 (2026-04-17)** — production `SearxSearchBackend` (`orchestrator/src/searx_search_backend.rs`) wraps `SearxClient` and is the default `SearchBackend` at `cli/src/main.rs:4374`. `StubSearchBackend` gated behind `ELAI_STUB_SEARCH=1` dev env var; sidecar URL override via `ELAI_SIDECAR_URL`. Covered by `contracts/tests/searx_search_backend_contract.rs` (5 tests, hand-rolled TCP stub — no new deps).
- **Session 4D.12 (2026-04-17, commit e8e5a03)** — `WebResearchOrchestrator` reachable from `task_runner` via `WebResearchRunner` trait + `DefaultWebResearchRunner` factory. `is_web_research_task` detector routes research-tier TaskSubmitted rows to the orchestrator, emitting per-iteration `WebResearchStep` SSB rows. Dashboard plugs `DefaultWebResearchRunner` (Vram12Gb tier) at setup. Contracts: `task_runner_web_research_contract.rs` (5 tests) + `web_research_orchestrator_contract.rs` (8 tests).
- **Session 4E.15 (2026-04-17)** — `scaffold::apply_template` reachable from `task_runner` via `ScaffoldApplier` trait. `is_web_scaffold_task` heuristic routes "make me a <framework>"-style tasks to `drive_web_scaffold_task`; `detect_scaffold_template` picks among `vite-react`/`vite-svelte`/`astro`/`vanilla`. Dashboard plugs `DashboardScaffoldApplier` at setup (supplies template vars using the scaffold crate's HSL defaults). Contracts: `task_runner_web_scaffold_contract.rs` (5 tests) + orchestrator unit tests (3).
- **Session 4E.13 (2026-04-17)** — `task_runner::drive_task` registers IntegrationSpecs at all four decomposition levels via `register_runner_integration_specs`, fires `run_integration_checkpoint(BabyStep, ...)` after every `nav.record_result`, and detects step/phase/concept boundary crossings via `BabyStepLocation` diff to fire `nav.run_{step,phase,concept}_integration`. `TestOutcome.passed` now derives from `run.success && outcome_passed` instead of the previous hardcoded `true`. 5 in-module unit tests pass (`register_runner_integration_specs_populates_all_four_levels`, `emit_level_transitions_fires_step_phase_concept_on_crossings`, `flush_final_integration_checkpoints_covers_all_three_levels`, `default_integration_spec_has_expected_shape`, `checkpoint_level_tag_maps_each_variant`). Runner still `NoOpIntegrationRunner` pending wire-4A (real baby-step execution) + a follow-up swap to `CommandIntegrationRunner`.
- **Session 4E.14 (2026-04-18)** — `task_runner` now evaluates compiled rules at each baby-step PRE_GATE via a new `RuleEvaluator` (engine + compiled rules bundled behind an `Arc`). `TaskRunnerConfig.rule_evaluator: Option<Arc<RuleEvaluator>>` plugs a `DefaultRuleEngine` + `load_seed_rules_from_dir(data/rules/)` compiled set; dashboard constructs one at setup. Block verdicts emit `Finding { category="rule_violation", severity="error" }` AND flip `TestOutcome.passed=false`; Warn verdicts emit warning Findings without flipping the outcome. Contracts: `task_runner_rule_eval_contract.rs` (5 tests — default-None wiring, phase_mask filter, domain filter + always-included, first-Block short-circuit, `rules_len`). `RuntimePredicates` is retained on the `PipelineBuilder` structural path (FSM-state `allowed_rules` binding at build time); runtime rule eval is the separate `RuleEvaluator` abstraction. `cargo check --workspace --lib --bins --tests` clean.

## Summary

- Total gaps found: **47** curated entries (plus raw grep-dump in Appendix A).
- By severity: **P0 = 9**, **P1 = 24**, **P2 = 14**.
- By effort: **S = 17**, **M = 21**, **L = 9**.
- Proposed session count: **18** (15 from initial audit + 3 promoted from Appendix B resolution on 2026-04-17: `wire-4A` InProcessAgentPool, `wire-antispiral` AntiSpiralDetector, `wire-a12` DeterministicAuditor).
- Estimated total effort: **~32-43 focused dev sessions** for P0+P1 items; P2 items pile onto a cleanup session.
- Appendix B resolution (all 10 items now have HIGH-confidence verdicts): 4 REAL GAP → 3 new sessions above + session #1 existing, 3 INTENTIONAL DESIGN → doc-comment patches folded into `patch-1.35`, 3 DEAD CODE → removal folded into `patch-1.35`.

## Session ordering (dependency DAG)

```
          4D.10 (research LLM)
            |
            v
  +-------- 4E.13 (pipeline integration checkpoint wiring) --+
  |                                                          |
  v                                                          v
4D.11 (web search → SearchBackend)                 patch-1.32 (SidecarProvider wiring)
  |                                                          |
  v                                                          v
4D.12 (WebResearchOrchestrator wiring)              patch-1.33 (steering/dola/TQ request plumbing)
                                                             |
                                                             v
                                                  patch-1.34 (HydeClient prod impl)
                                                             |
  +--------------------------------------------------------- +
  |
  v
4E.14 (rules engine wired to task_runner)
  |
  v
4E.15 (scaffold trigger from baby step)
  |
  v
3C.REDO (CheckpointManager/RecoveryManager wired to task_runner failures)
  |
  v
3D.REDO (BspEngine activation path OR formal deprecation)
  |
  v
A-19.REDO (interrogation tree consumer)
  |
  v
3F.REDO (MistakeBook read/write in task_runner retry loop)
  |
  v
newphase-4F (FeasibilityProbe deliverable)
  |
  v
newphase-4G (AutonomousGate deliverable)
  |
  v
newphase-4H (self-modification — Layer 1A at minimum)
  |
  v
wire-4A    (InProcessAgentPool → task_runner::run_baby_step replaces synthetic TestOutcome surrogate)
  |
  v
wire-antispiral  (AntiSpiralDetector::check_step in GRC loop; emit SpiralDetected SSB row; depends on wire-4A)
  |
  v
wire-a12   (DeterministicAuditor invoked in AUDITING FSM state before LLM auditor agent)
  |
  v
patch-1.35 (dead-code sweep: supervision, DolaDecoder (Appx B #2), compute_vector_via_sidecar (Appx B #3),
            run_research_pipeline + StubSearchBackend + SearchBackend (Appx B #8),
            DefaultMcpBridge behind-feature-flag doc (Appx B #10), IpcGateway, IpcHookDispatcher,
            validated_ssb, stale comments + doc-comments for KnowledgeEngine (Appx B #4) and
            SsbPayload forward-compat variants (Appx B #5))
```

## Sessions

### Session 4D.10 — Research LLM wiring
**Goal:** replace synthetic FlashResearch reports with real sidecar-backed inference and make `research_decompose` async.
**Severity breakdown:** P0 = 1 (this is the blocker the user already identified).
**Effort:** M (2-3 sessions).
**Dependencies:** none.
**Gaps covered:**
1. `research_decompose` returns deterministic synthetic reports — `.elai_cc/crates/orchestrator/src/research_tree.rs:1451-1508` — P0 — HIGH. Called by `task_runner.rs:644`.
2. `build_synthetic_report` is the sole report producer; comment at :1512 marks the IPC substitution point — `.elai_cc/crates/orchestrator/src/research_tree.rs:1510-1566` — P0 — HIGH.
3. `ResearchTree::run` does `node.phi = progress * (phi_min + 0.15)` simulation — `.elai_cc/crates/orchestrator/src/research_tree.rs:791-830` — P1 — HIGH. Comment at :802 acknowledges IPC GroundingRequest substitution.
4. `WebResearchOrchestrator::formulate_query` uses deterministic domain-exclusion instead of LLM reformulation — `.elai_cc/crates/orchestrator/src/web_research_orchestrator.rs:313-345` — P1 — HIGH.
5. Four L0/L1/L2/L3 prompt templates absent — spec at `docs/specs/guide/steps/phase-4D.txt:515-1215`.
6. JSON → `ResearchReport` parser absent — spec at `docs/specs/guide/steps/phase-4D.txt:515-1215`.
7. Sidecar lifecycle owner absent in task_runner path — referenced in comment at `.elai_cc/crates/orchestrator/src/task_runner.rs:17-20`.

**Spec references:** `docs/specs/guide/steps/phase-4D.txt:515-1215`; `docs/specs/guide/steps/phase-index-3A-4J.txt:225-238`.
**Acceptance criteria (testable):**
- [x] `research_decompose` (now async) calls a `GroundingClient` trait; one impl is `SidecarGroundingClient` talking to `/v1/completions`.
- [x] At least 4 prompt templates exist at `.elai_cc/crates/orchestrator/src/research_prompts/{l0_idea,l1_concept,l2_phase,l3_step}.txt` and are referenced by name.
- [x] Parsing an empty/malformed JSON response from sidecar returns a typed error, not a synthetic fallback.
- [x] `build_synthetic_report` is removed from the production module (kept in `#[cfg(test)]` only) OR guarded behind `ExecutionMode::Deterministic` explicitly.
- [x] `task_runner::drive_task` awaits the new async path and still surfaces failures as `Finding` SSB rows.

**Out of scope:** ABMCTS scoring, verbose GG-signal handling, retry policy tuning — separate follow-ups.

---

### Session 4D.11 — Web search production backend — DONE (2026-04-17)
**Goal:** ship a real `SearchBackend` impl so the `run_research_pipeline` CLI command stops returning `NoUsableSources`.
**Severity breakdown:** P0 = 1, P1 = 1.
**Effort:** S.
**Dependencies:** none (can run parallel with 4D.10).
**Gaps covered:**
1. `StubSearchBackend::new()` is the only backend used in production — `.elai_cc/crates/cli/src/main.rs:4374` — P0 — HIGH. Confirmed by grep: `impl SearchBackend for` appears exactly once (`research_pipeline.rs:59`, the stub itself). — RESOLVED: `SearxSearchBackend` is now the default at `cli/src/main.rs:4374`.
2. `SearxClient::new` has zero production callers — `.elai_cc/crates/web_search/src/client.rs:18-28`; callers list: four test files only — P0 — HIGH. — RESOLVED: wrapped by `SearxSearchBackend` in `orchestrator/src/searx_search_backend.rs`.
3. Comment at `.elai_cc/crates/web_search/src/client.rs:16` says "IPC proxy wiring deferred to 4D.17" — P1 — MED. — Still open (IPC proxy for Tier-C isolated agents is the 4D.17 concern; orchestrator-side direct call path is now live).

**Spec references:** `docs/specs/guide/steps/phase-4D.txt`; memory `project_search_architecture.md` (sidecar `/search` + SearXNG Docker auto-start is already built server-side but never called from Rust).
**Acceptance criteria:**
- [x] `SearxSearchBackend` struct implements `SearchBackend`, wraps `SearxClient`. Sync→async bridge via a dedicated current-thread runtime + `std::sync::Mutex<SearxClient>`. Graceful fallback to `StubSearchBackend` at the call site if `SearxSearchBackend::new` returns `Err` (no SearXNG-specific circuit breaker — the sidecar already implements the circuit via its own SearXNG→Brave→empty strategy).
- [x] `cli/src/main.rs:4374` constructs `SearxSearchBackend` by default; `StubSearchBackend` is selected only under `ELAI_STUB_SEARCH=1` dev env var (no new argv flag — kept classifier.rs / commands.rs untouched). `ELAI_SIDECAR_URL` overrides the default `http://localhost:8100` base URL.
- [x] Integration test (hand-rolled `TcpListener` HTTP stub — no new mockito/wiremock dep): `contracts/tests/searx_search_backend_contract.rs` — 5 tests pass (dyn-safety, default-URL construction, stub-backed non-empty result round-trip, max_results truncation, unreachable-host error path).
- [x] Memory note updated — `project_search_architecture.md` records the new wiring and env-var surface.

**Out of scope (unchanged):** Brave fallback in Rust (already handled by the sidecar); source-quality boosting refinements; 4D.17 IPC-proxy path for Tier-C research agents.

**Files touched:**
- NEW: `.elai_cc/crates/orchestrator/src/searx_search_backend.rs` (bridge + 3 unit tests)
- NEW: `.elai_cc/crates/contracts/tests/searx_search_backend_contract.rs` (5 contract tests)
- MOD: `.elai_cc/crates/orchestrator/src/lib.rs` (module + `pub use`)
- MOD: `.elai_cc/crates/cli/src/main.rs` (`SlashCommand::Research` arm — env-var-gated backend selection)

---

### Session 4D.12 — WebResearchOrchestrator wiring — DONE (2026-04-17)
**Goal:** make `WebResearchOrchestrator` reachable from `task_runner` so web-research tasks actually run.
**Severity breakdown:** P0 = 1, P1 = 1.
**Effort:** M.
**Dependencies:** 4D.10, 4D.11.
**Gaps covered:**
1. `WebResearchOrchestrator::new` only has test callers — `.elai_cc/crates/orchestrator/src/web_research_orchestrator.rs:78-91`; `contracts/tests/web_research_orchestrator_contract.rs:70, 146` are the only sites — P0 — HIGH. — RESOLVED: `DefaultWebResearchRunner::build_orchestrator` in `crates/orchestrator/src/task_runner.rs` is the production caller, and the dashboard plugs it in at `crates/dashboard/src/lib.rs`.
2. `WebContentIndex::new(WebAragConfig::default())` only in tests/benchmarks — `.elai_cc/crates/knowledge/src/web_arag.rs:98`; callers: all tests — P1 — HIGH. — RESOLVED: `DefaultWebResearchRunner` constructs a fresh `WebContentIndex` per `run()` call (acceptance criterion 3) so state stays scoped to one task.
3. `SsbPayload::WebResearchStep` emitter exists only in a contract test — `.elai_cc/crates/contracts/tests/web_research_orchestrator_contract.rs:114` — P1 — HIGH. — RESOLVED: `SsbWebResearchStepEmitter` emits `WebResearchStep` rows from `run_node_loop` per iteration (actions: `iterate`, `no_results`, `converged`, `pruned`, `max_iterations`).
4. `ARAG`-for-web chunk-level embedding fallback documented as stub — `.elai_cc/crates/knowledge/src/arag.rs:261, 272` — P2 — MED. — Still open (explicitly listed out of scope below).

**Spec references:** `docs/specs/guide/steps/phase-4D.txt` (Web Research Extension).
**Acceptance criteria:**
- [x] `task_runner::drive_task` detects web-research tasks (via `is_web_research_task` heuristic: "research:", "research ", "find out ", "look up ", "investigate ", "what is/are ", "who is/are ", "when is/was ", "where is ", "how does ", or substrings "web search"/"web research") and routes them through a `WebResearchOrchestrator` instance owned by `ResolvedConfig.web_research`.
- [x] `WebResearchStep` SSB rows are emitted per node iteration (via `WebResearchStepEmitter` plumbed into `run_node_loop`) and visible in the dashboard activity feed (SSB `web_research_step` action type).
- [x] `WebContentIndex` is populated across the session lifetime of one task — `DefaultWebResearchRunner::build_orchestrator()` constructs a fresh `WebContentIndex` on every `run()` call (verified by contract test `web_research_runner_invoked_once_per_task`).
- [x] Cancellation honored: `drive_web_research_task` races `runner.run` against `CancelHandle.wake.notified()` via `tokio::select!`.

**Out of scope:** Dual-vector embeddings (arag.rs:261 mock); catrag overlay for web chunks; contradiction detection; source-tiers.toml loading from disk (runner accepts a TOML string, dashboard currently passes the empty default).

**Files touched:**
- MOD: `.elai_cc/crates/orchestrator/src/web_research_orchestrator.rs` (emitter trait + event + `with_step_emitter` + emission points in `run_node_loop`)
- MOD: `.elai_cc/crates/orchestrator/src/task_runner.rs` (`is_web_research_task`, `WebResearchRunner` trait + `DefaultWebResearchRunner`, `SsbWebResearchStepEmitter`, `TaskRunnerConfig.web_research_runner` field, `drive_web_research_task` with cancel-aware `select!`)
- MOD: `.elai_cc/crates/orchestrator/src/lib.rs` (re-exports for new public API)
- MOD: `.elai_cc/crates/dashboard/src/lib.rs` (plug `DefaultWebResearchRunner` at setup using `Vram12Gb` tier + `DEFAULT_SIDECAR_SEARCH_URL`)
- NEW: `.elai_cc/crates/contracts/tests/task_runner_web_research_contract.rs` (5 contract tests, all pass)

---

### Session 4E.13 — Integration checkpoint activation — DONE (2026-04-17)
**Goal:** actually call `run_integration_checkpoint` at the BabyStep/Step/Phase/Concept boundaries from `task_runner`.
**Severity breakdown:** P0 = 1, P1 = 1.
**Effort:** M.
**Dependencies:** none — foundational for any real baby-step execution.
**Gaps covered:**
1. `run_integration_checkpoint` only called from its own tests — `.elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs:1286`; production callers: zero — P0 — HIGH. — RESOLVED: task_runner now invokes it at BabyStep level after `nav.record_result`.
2. `record_integration_checkpoint` only called from tests — `.elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs:1038` — P1 — HIGH. — Superseded: `run_integration_checkpoint` subsumes the baby-step recording path and is the production code path now.
3. Comment at `.elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs:1765` says "we record checkpoint via a separate call since IntegrationSpec" — P1 — MED. — Still open (P2 now): comment accuracy tracked by patch-1.35.
4. Task-runner loop emits `TestOutcome { passed: true }` unconditionally — `.elai_cc/crates/orchestrator/src/task_runner.rs:717-727`. Comment at :718-720 calls this an MVP surrogate. — PARTIALLY RESOLVED (wiring-only): `TestOutcome.passed` and `PhaseComplete.test_outcomes` now derive from `run_integration_checkpoint` aggregate success AND the recorded `BabyStepResult.outcome`. The formula is on the production path, but both inputs are hardcoded `true` until (a) wire-4A replaces the `BabyStepOutcome::Passed` literal and (b) `NoOpIntegrationRunner` is swapped for `CommandIntegrationRunner` with real `check_command`s. Pre-4E.13 pass/fail was literally unreachable; post-4E.13 it is a single-line change in two separate places.

**Spec references:** `docs/specs/guide/steps/phase-4E.txt`; `phase-index-3A-4J.txt:251-268`.
**Acceptance criteria:**
- [x] After each baby step's `nav.record_result`, `task_runner` calls `run_integration_checkpoint(level=BabyStep, spec, state)` with a real `IntegrationSpec`. Specs are registered on the tree for all four levels via `register_runner_integration_specs` before traversal begins.
- [~] `TestOutcome` payloads reflect real checkpoint pass/fail instead of always `true` — WIRING COMPLETE, SIGNAL PARTIAL. The formula `passed = run.success && outcome_passed` is on the production call path and `PhaseComplete.test_outcomes` is now derived from the per-step `passed` booleans (pre-4E.13 it hardcoded `:passed`). BUT both inputs are still structurally `true` today: `BabyStepOutcome` is hardcoded `Passed` (removed only by wire-4A) and `NoOpIntegrationRunner::run_criterion` always succeeds. So the observable payload is still `passed=true` in every baby step. Full live signal requires wire-4A AND a follow-up swap to `CommandIntegrationRunner` with real `check_command`s — both are separately-tracked blockers.
- [x] Phase-level and concept-level checkpoints fire when the navigator crosses those boundaries. `emit_level_transitions` compares the previous vs. current `BabyStepLocation` each iteration and fires step → phase → concept (bottom-up, matching the spec's bubble-up cascade); `flush_final_integration_checkpoints` closes the last location's boundaries after the loop terminates.

**Out of scope:** Speculative cross-phase checkpoints; cargo-test scheduling (handled by bash tool); swapping `NoOpIntegrationRunner` for `CommandIntegrationRunner` (requires spawn_blocking + scoped command whitelists — separate follow-up).

**Files touched:**
- MOD: `.elai_cc/crates/orchestrator/src/task_runner.rs` (imports for 4E.13 types, `INTEGRATION_CHECKPOINT_FAILED_CATEGORY` / `INTEGRATION_BUBBLE_UP_CATEGORY` consts, `register_runner_integration_specs`, `default_integration_spec`, `emit_level_transitions`, `flush_final_integration_checkpoints`, `emit_checkpoint_outcome`, `emit_integration_bubble_up`, `checkpoint_level_tag`; `drive_task` loop rewired to register specs + track `last_location` + fire per-level checkpoints + emit TestOutcome from checkpoint result)
- Added 5 in-module unit tests (`register_runner_integration_specs_populates_all_four_levels`, `emit_level_transitions_fires_step_phase_concept_on_crossings`, `flush_final_integration_checkpoints_covers_all_three_levels`, `default_integration_spec_has_expected_shape`, `checkpoint_level_tag_maps_each_variant`) — all 5 pass.
- Note: 4 pre-existing task_runner tests (`task_runner_consumes_submitted_task_and_emits_progress`, `..._dedupes_via_cursor_on_restart`, `..._emits_preview_available_for_web_project`, `..._emits_finding_when_launcher_missing_for_web_project`) require a live sidecar at localhost:8100 (broken since 4D.10 commit `4947fb5` hardcoded `SidecarGroundingClient::default()` in `drive_task`). Not caused by 4E.13; separate follow-up to inject a mock `GroundingClient` via `TaskRunnerConfig`.

---

### Session 4E.14 — Rules engine → task_runner — DONE (2026-04-18)
**Goal:** stop using `RuntimePredicates` (always-empty rule set) in task_runner; use the real `DefaultRuleEngine` already loaded in CLI.
**Severity breakdown:** P1 = 2.
**Effort:** M.
**Dependencies:** 4E.13.
**Gaps covered:**
1. `RuntimePredicates::active_rules_for_phase` returns `Vec::new()` — `.elai_cc/crates/orchestrator/src/task_runner.rs:364-377` — P1 — HIGH. — PARTIAL: `RuntimePredicates` is preserved on the `PipelineBuilder` structural-validation path (pipeline DAG checks only; returning `Vec::new()` there is correct for that role and documented accordingly). Runtime rule evaluation is now a separate `RuleEvaluator` abstraction — see below.
2. `DefaultRuleEngine` IS constructed at `cli/src/main.rs:6863`, but the instance is not passed to `task_runner`. Two parallel rule systems exist — P1 — HIGH. — RESOLVED: `task_runner` now accepts `TaskRunnerConfig.rule_evaluator: Option<Arc<RuleEvaluator>>`; dashboard constructs a `DefaultRuleEngine` + compiled seed rules at startup and plugs it in (`crates/dashboard/src/lib.rs`).
3. `rules::self_discovery` conservative defaults comment — `.elai_cc/crates/rules/src/self_discovery.rs:36, 334` — P2 — MED. — Still open (self-discovery proposal loop was always out of scope for 4E.14).

**Spec references:** `docs/specs/guide/steps/phase-3F.txt` (rule engine architecture, PRE_GATE semantics, evaluation order).
**Acceptance criteria:**
- [x] `TaskRunnerConfig` carries an `Arc<dyn RuleEngine>` equivalent. Shape chosen: `Option<Arc<RuleEvaluator>>` where `RuleEvaluator` bundles `Arc<dyn RuleEngine + Send + Sync>` with an `Arc<Vec<(Rule, CompiledPredicate)>>` compiled rule set. The wrapper is required because `CompiledPredicate` is not `Clone` (`SchemaRequired` holds a `jsonschema::Validator`), so sharing by `Arc` is the only allocation-free option.
- [x] `drive_task` evaluates rules at every baby-step PRE_GATE via `evaluate_pre_gate_and_emit_findings`. Each `Block` verdict emits a `Finding { category: "rule_violation", severity: "error" }` row AND flips `TestOutcome.passed` to `false`; each `Warn` verdict emits a `Finding { severity: "warning" }` row without touching the outcome. Action string is `baby_step_pre_gate`; evaluator uses `FsmPhaseBit::EXECUTING` and derives `file_domains` from `is_web_project` (`["web"]` when web, empty otherwise — engineering/security are always-included by `rule_engine::get_active_rules` semantics).
- [x] Seeded rules reach the dashboard. The dashboard constructs `DefaultRuleEngine` + calls the new `load_seed_rules_from_dir` helper (lives in `.elai_cc/crates/orchestrator/src/rule_loader.rs`, shared with the CLI's loader in spirit — `CARGO_MANIFEST_DIR/../../data/rules` resolves to the same on-disk TOML files `engineering.toml`/`web.toml`). At runtime, a violation on a task triggers `Finding` rows the existing dashboard activity feed already surfaces.
- **Not done (documented decision):** full deletion of `RuntimePredicates`. The structural `PredicateCompiler` role and the runtime `RuleEngine` role are intentionally separated — `PipelineBuilder::generate_states` uses `PredicateCompiler` to bind FSM-state `allowed_rules` at pipeline-build time; runtime evaluation happens via `RuleEvaluator` at baby-step PRE_GATE. Merging the two would require changing `PipelineBuilder` signatures (out of 4E.14 scope). Logged as a follow-up in `patch-1.35` (dead-code sweep) territory.

**Out of scope (unchanged):** Self-discovery proposal loop; interrogation-tree traversal (covered in A-19.REDO); mapping pipeline phase-name strings ("DESIGN"/"EXECUTING"/…) to `FsmPhaseBit` bitmasks for `RuntimePredicates::active_rules_for_phase` (future `PipelineBuilder` consolidation work).

**Files touched:**
- NEW: `.elai_cc/crates/orchestrator/src/rule_loader.rs` — `load_seed_rules_from_dir(rules_dir, engine)` returns `Vec<(Rule, CompiledPredicate)>` from TOML seeds; silently skips unreadable/unparsable/uncompilable rules with `tracing::warn`.
- NEW: `.elai_cc/crates/contracts/tests/task_runner_rule_eval_contract.rs` — 5 contract tests covering: `new_minimal` default, phase_mask filter, domain filter with always-included override, first-Block short-circuit, `rules_len` observability.
- MOD: `.elai_cc/crates/orchestrator/src/task_runner.rs` — add `RuleEvaluator` + `evaluate_at_pre_gate`; add `RULE_VIOLATION_CATEGORY`, `BABY_STEP_PRE_GATE_ACTION` constants; wire `rule_evaluator: Option<Arc<RuleEvaluator>>` through `TaskRunnerConfig` + `ResolvedConfig`; insert `evaluate_pre_gate_and_emit_findings` call in the baby-step loop before `emit_test_outcome`; AND Block verdicts into `passed`. Three internal test `TaskRunnerConfig { ... }` literals updated with `rule_evaluator: None`.
- MOD: `.elai_cc/crates/orchestrator/src/lib.rs` — `pub mod rule_loader`; re-export `RuleEvaluator`, `RULE_VIOLATION_CATEGORY`, `BABY_STEP_PRE_GATE_ACTION`, `load_seed_rules_from_dir`.
- MOD: `.elai_cc/crates/dashboard/Cargo.toml` — add `elai-rules` and `elai-knowledge` path deps.
- MOD: `.elai_cc/crates/dashboard/src/lib.rs` — import `RuleEvaluator`, `DefaultRuleEngine`, `EmptyDkgQuerier`, `TreeSitterAstEvaluator`, `load_seed_rules_from_dir`; add `default_seed_rules_dir()`; construct `RuleEvaluator` and pass to `TaskRunnerConfig.rule_evaluator` at dashboard setup.
- MOD: `.elai_cc/crates/contracts/tests/task_runner_web_research_contract.rs` — add `scaffold_applier: None` (pre-existing 4E.15 gap) + `rule_evaluator: None` to all 3 `TaskRunnerConfig {..}` literals.
- MOD: `.elai_cc/crates/contracts/tests/task_runner_web_scaffold_contract.rs` — add `rule_evaluator: None` to all 3 `TaskRunnerConfig {..}` literals.
- MOD: `.elai_cc/crates/dashboard/tests/integration_dashboard_state.rs` — add `task_runner_cancel` field (pre-existing patch-1.30 test drift, fixed incidentally to keep `cargo check --workspace --tests` green).

**Tests:**
- `cargo test -p elai_cc_contracts --test task_runner_rule_eval_contract`: 9/9 passed (post-audit: +Escalate surface + 3 loader tests).
- `cargo test -p elai_cc_contracts --test task_runner_web_research_contract --test task_runner_web_scaffold_contract --test task_runner_rule_eval_contract`: 19/19 passed.
- `cargo check --workspace --lib --bins --tests`: clean (only pre-existing `unused_import` warnings in bm25_contract / bsp_engine_contract / plugins_hooks / pipeline_tree_navigator; `dead_code` warning on `WebResearchOrchestrator.clock`).

**Audit fixes (adversarial-auditor, 2 BLOCK + 3 WARN resolved before commit):**
- BLOCK: engine errors in `evaluate_at_pre_gate` previously mapped to `VerdictResult::Allow` (silent). Now map to `VerdictResult::Warn` AND log via `tracing::warn!` so the emitter surfaces a warning-severity Finding.
- BLOCK: `VerdictResult::Escalate(target)` at PRE_GATE was silently dropped. Now emits a distinct `Finding { severity: "escalate", category: "rule_violation" }` carrying the escalation target string. Contract test `evaluate_at_pre_gate_surfaces_escalate_verdicts` asserts the verdict survives the filter path.
- WARN: local duplicate `RULE_ALWAYS_INCLUDED_DOMAINS` removed; `rule_engine::ALWAYS_INCLUDED_DOMAINS` promoted to `pub const` so there's a single source of truth.
- WARN: dashboard `default_seed_rules_dir()` now honours `ELAI_RULES_DIR` env var before falling back to `CARGO_MANIFEST_DIR` (dev path). Production Tauri bundles can set the env var to point at a shipped resource dir.
- WARN: 3 new contract tests cover `load_seed_rules_from_dir` happy path, missing-directory fallback, and unparseable-file skip-but-keep-siblings behaviour.

**Follow-ups (not done by this session):**
- `patch-1.35` / future consolidation: collapse `RuntimePredicates` into `RuleEvaluator` once `PipelineBuilder::generate_states` accepts a richer abstraction (phase-name → `FsmPhaseBit` mapping + rule-id listing).
- Promote the CLI's `load_seed_rules` (`cli/src/main.rs:6286`) to the shared `load_seed_rules_from_dir` helper so both binaries share one source of truth.
- `drive_task` PRE_GATE `source` currently is the natural-language task text — limits what `GrepPattern`/`AstPattern` rules catch. Real source text lands when `InProcessAgentPool` is wired into the runner (wire-4A).

---

### Session 4E.15 — Scaffold trigger from baby step — DONE (2026-04-17)
**Goal:** wire `scaffold::apply_template` into baby-step execution for web-scaffold baby steps. Dashboard command alone is insufficient.
**Severity breakdown:** P1 = 1.
**Effort:** S.
**Dependencies:** 4E.13.
**Gaps covered:**
1. `scaffold::apply_template` only called from `dashboard::commands::create_project_from_template` — `.elai_cc/crates/dashboard/src/commands.rs:147`. Zero callers from `task_runner` — P1 — HIGH. — RESOLVED: `task_runner::drive_web_scaffold_task` routes scaffold-intent tasks through a pluggable `ScaffoldApplier` trait; the dashboard's production impl (`DashboardScaffoldApplier`, `crates/dashboard/src/lib.rs`) wraps `scaffold::apply_template` and supplies the seven template variables using the scaffold crate's shipped HSL defaults.
2. `task_runner` detects web projects via `project_dir_is_web` and launches a dev server but never scaffolds — `.elai_cc/crates/orchestrator/src/task_runner.rs:629, 737`. — RESOLVED: `is_web_scaffold_task` heuristic matches "make me / create / build me / bootstrap / new <framework>" prefixes AND a web keyword; the dev-server integration flow ("Scaffold a Vite ... and serve it") stays on the default pipeline by design (the "scaffold a" phrasing is deliberately excluded from the prefix list).

**Spec references:** `docs/specs/guide/steps/phase-WEB*.txt` (absent — WEB work landed as WEB.1/.2/.3; `phase-4K.txt:363-368` + the scaffold crate source stand in).
**Acceptance criteria:**
- [x] Baby step with tool = `scaffold_apply` routes through `scaffold::apply_template` via task_runner (not dashboard). Detector + router live at `task_runner::is_web_scaffold_task` / `drive_web_scaffold_task`; dashboard injects `DashboardScaffoldApplier` at setup.
- [x] Integration test: submit a "make me a React app" task → scaffold flow emits `PhaseComplete` with template=`vite-react`. See `contracts/tests/task_runner_web_scaffold_contract.rs` (5 tests: heuristic, template picker, missing-applier Finding, error-path Finding, happy-path round-trip) + orchestrator unit tests (`is_web_scaffold_task_matches_documented_triggers`, `detect_scaffold_template_picks_framework_or_vanilla`, `task_runner_routes_web_scaffold_task_through_applier`).

**Out of scope (unchanged):** `apply_template()` non-root variant compile-time CARGO_MANIFEST_DIR path portability (WARN noted in `workflow/audit-scratch/test_web1_scaffold_invariants.py:121-132`). Template variable overrides from user input — the dashboard applier uses fixed defaults (project_name="New Project", author="elai", scaffold crate's HSL constants); per-task variable extraction is a follow-up.

**Files touched:**
- MOD: `.elai_cc/crates/orchestrator/src/task_runner.rs` (web-scaffold routing + 3 unit tests)
- MOD: `.elai_cc/crates/orchestrator/src/lib.rs` (re-exports for `ScaffoldApplier`, `is_web_scaffold_task`, template constants)
- MOD: `.elai_cc/crates/dashboard/src/lib.rs` (`DashboardScaffoldApplier` production impl + wiring)
- NEW: `.elai_cc/crates/contracts/tests/task_runner_web_scaffold_contract.rs` (5 contract tests)

---

### Session 3C.REDO — Checkpoint / Recovery wiring
**Goal:** make `CheckpointManager` and `RecoveryManager` fire on real failures in task_runner.
**Severity breakdown:** P1 = 2.
**Effort:** M.
**Dependencies:** 4E.13 (needs real test outcomes to recover from).
**Gaps covered:**
1. `CheckpointManager::new` has ZERO production callers — `.elai_cc/crates/orchestrator/src/checkpoint.rs:115-119`. All 31 construction sites are under `#[cfg(test)]` or in `tests/`. Confirmed by grep — P1 — HIGH.
2. `RecoveryManager::new` has ZERO production callers — `.elai_cc/crates/orchestrator/src/recovery.rs:319`. All construction sites are in `#[cfg(test)]` modules or `tests/recovery_contract.rs` — P1 — HIGH.
3. Comment in `.elai_cc/crates/orchestrator/src/recovery.rs:23`: `STUB(3C.2): heartbeat-based death detection not yet implemented.`
4. Comment at `.elai_cc/crates/orchestrator/src/recovery.rs:515`: `STUB(3C.2): Timeout treated same as Crash` — P2 — MED.

**Spec references:** `docs/specs/guide/steps/phase-3C.txt`.
**Acceptance criteria:**
- [ ] `ResolvedConfig` owns a `CheckpointManager` and writes a `Checkpoint` SSB row at each baby step boundary.
- [ ] On `BabyStepOutcome::Failed` (not yet emitted by task_runner — needs 4E.13), `RecoveryManager::recover_agent` fires with the matching recipe.
- [ ] Integration test kills a baby-step agent → checkpoint visible → recovery runs → task resumes.

**Out of scope:** heartbeat-based death detection (separate recovery.rs:23 STUB); `attempt_recovery` wiring for non-BabyStep scenarios.

---

### Session 3D.REDO — BSP engine: activate 
**Goal:**  route task_runner through `BspEngine` for parallel execution
**Severity breakdown:** P1 = 1, P2 = 2.
**Effort:** L (activate)
**Dependencies:** 3C.REDO.
**Gaps covered:**
1. `BspEngine::new` has ZERO production callers — `.elai_cc/crates/orchestrator/src/bsp/engine.rs`. All 20+ construction sites are in `#[cfg(test)]` or `tests/` — P1 — HIGH.
2. `task_runner` executes baby steps sequentially (no parallelism) — `.elai_cc/crates/orchestrator/src/task_runner.rs:691-803` — P1 — HIGH.
3. Dead-BSP-payload-types: `DeathEvent`, `Checkpoint`, `ModelSwap`, `SupervisionExhausted`, `WorkloadProfile` all emitted ONLY inside `bsp/engine.rs` → dead code cascade — P2 — MED.

**Spec references:** `docs/specs/guide/steps/phase-3D.txt`; `phase-index-3A-4J.txt` mentions BSP parallel execution.
**Acceptance criteria:**
- [ ] EITHER: `task_runner` constructs `BspEngine` for phases with `is_parallel = true` and executes via `execute_tasks_with_timeout`.
- [ ] OR: `pub use bsp::engine::*` removed from `lib.rs`; module moved under `#[cfg(feature = "bsp-experimental")]` with explicit inactive note.

**Out of scope:** Full workload-profiling instrumentation; deferred-phase re-wiring (already works inside BSP, but the engine itself isn't called).

---

### Session A-19.REDO — InterrogationTree consumer
**Goal:** make seeded interrogation trees actually used for failure-driven reasoning.
**Severity breakdown:** P1 = 1, P2 = 1.
**Effort:** M.
**Dependencies:** 4E.14.
**Gaps covered:**
1. `TreeTraverser::new` and `traverse_with_answers` have ZERO production callers — `.elai_cc/crates/reasoning/src/traversal.rs:30, 124`; usage limited to module tests — P1 — HIGH.
2. `seed_interrogation_trees` fires at schema init — `.elai_cc/crates/rules/src/schema.rs:465` — good. But `load_tree_by_id` / `load_all_trees` / `save_interrogation_tree` / `delete_tree` have ZERO production callers — `.elai_cc/crates/rules/src/interrogation_store.rs:62, 22, 86, 112` — P1 — HIGH.
3. Re-confirms the prior A-series audit: A-19 interrogation tree storage is seeded but never queried at runtime.

**Spec references:** `reasoning-architecture-design.txt` LAYER 3; prior A-19 audit.
**Acceptance criteria:**
- [ ] On baby-step failure, `task_runner` (or rules engine) looks up the matching `InterrogationTree` by `failure_taxonomy_category` and drives a short traversal to produce a fix-template suggestion attached to the Finding.
- [ ] `TreeTraverser` used from at least one non-test call site.
- [ ] Seed-tree count > 0 contributes to a metric visible in dashboard.

**Out of scope:** Building new trees for uncovered failure categories; LLM-assisted tree synthesis.

---

### Session 3F.REDO — MistakeBook read/write in retry loop
**Goal:** have task_runner query MistakeBook at PRE_GATE and insert entries on failure.
**Severity breakdown:** P1 = 1.
**Effort:** S.
**Dependencies:** 4E.13 (need real failure outcomes to record).
**Gaps covered:**
1. `mistake_book::query_by_baby_step`, `insert`, `clear_for_baby_step` are defined and tested — `.elai_cc/crates/orchestrator/src/mistake_book.rs:31, 65, 105` — but callers outside the same file: ZERO — P1 — HIGH.
2. Context-budget module references `MistakeBook constraints (150 tokens)` — `.elai_cc/crates/orchestrator/src/context_budget.rs:153, 883` — reserved slot but unfilled.
3. `MistakeEntryPayload` + `SsbActionType::MistakeEntry` exist but no production emitter.

**Spec references:** `docs/specs/guide/steps/phase-A-20.txt` (Code-A1 arXiv:2603.15611).
**Acceptance criteria:**
- [ ] Task_runner calls `query_by_baby_step` before each attempt and injects results into the agent's context slot (the 150-token budget already reserved).
- [ ] On retry after failure, a `MistakeEntry` row is inserted via `insert_mistake`.
- [ ] On success, `clear_for_baby_step` tombstones prior entries.

**Out of scope:** Micro-reflection generation (works but is a CLI-invoked path, not task_runner).

---

### Session patch-1.32 — SidecarProvider as default Local backend
**Goal:** make the production `Local` BackendMode actually route to SidecarProvider.
**Severity breakdown:** P0 = 1.
**Effort:** S.
**Dependencies:** none.
**Gaps covered:**
1. `SidecarProvider::new` constructed only at `.elai_cc/crates/providers/src/sidecar_client.rs:396` under `#[cfg(test)]`. Grep confirmed: zero production callers of `.complete()` on SidecarProvider — P0 — HIGH.
2. Same pattern for `VllmProvider::new` — all four sites are test-only (`.elai_cc/crates/providers/src/vllm_client.rs:655, 668, 700, 768`) — P1 — HIGH.
3. `build_dispatch_context` has zero production callers — `.elai_cc/crates/orchestrator/src/dispatcher.rs:135`; only test sites at :222, 247, 273 — P1 — HIGH. Means `BackendMode::Local` routing is inert.

**Spec references:** `docs/specs/guide/steps/phase-3J.txt` (LOCAL backend); `phase-index-3A-4J.txt:310-319`.
**Acceptance criteria:**
- [ ] CLI `--local` flag constructs and dispatches through `SidecarProvider`.
- [ ] `build_dispatch_context` called before every LLM dispatch in `conversation.rs`.
- [ ] Integration smoke test: `elai --local "hello"` produces a non-synthetic response.

**Out of scope:** VLLM backend (can stay test-only until a real vLLM server is part of the demo).

---

### Session patch-1.33 — Sidecar steering/DoLa/TurboQuant request plumbing
**Goal:** send the `dola_enabled`, `turboquant_bits`, `steering_*` config fields to the sidecar — they are currently local-only.
**Severity breakdown:** P1 = 1.
**Effort:** S.
**Dependencies:** patch-1.32.
**Gaps covered:**
1. `SidecarProvider::complete_impl` sends only `prompt, max_tokens, temperature` — `.elai_cc/crates/providers/src/sidecar_client.rs:234-239`. The `dola_enabled / turboquant_bits / steering_enabled / steering_coefficient / dola_alpha / etc_threshold` fields on `SidecarConfig` (defined lines 65-98) are never serialized into the request body.
2. `compute_vector_via_sidecar` defined but has ZERO callers — `.elai_cc/crates/providers/src/layer_config/steering.rs:501, 563` — P1 — HIGH. Steering is fully dead.
3. `DolaDecoder::new` only in test block — `.elai_cc/crates/providers/src/layer_config/dola.rs:490` — P2 — MED. (The sidecar does DoLa internally, so Rust-side DolaDecoder may be redundant; confirm design intent.)

**Spec references:** `docs/specs/guide/steps/phase-3J.txt` steering; memory `project_dola_tq_findings.md`.
**Acceptance criteria:**
- [ ] `complete_impl` body JSON includes the seven config fields.
- [ ] Sidecar Python side reads and applies them (server.py already supports — verify contract).
- [ ] `compute_vector_via_sidecar` called from at least one agent-setup path (e.g. first-time steering vector computation per role).

**Out of scope:** DolaDecoder removal if redundant — file a separate decision note.

---

### Session patch-1.34 — HydeClient production implementation
**Goal:** give the HyDE stage a real client so hypothetical-document rewriting works.
**Severity breakdown:** P1 = 1.
**Effort:** S.
**Dependencies:** patch-1.32.
**Gaps covered:**
1. All `HydeClient` impls are `MockHydeClient`, `FailingHydeClient`, `EmptyHydeClient` — all in test module — `.elai_cc/crates/knowledge/src/hyde.rs:204, 230, 239`. Zero production impls — P1 — HIGH.
2. `HydeStage::new` constructed only in tests — same file line 87.

**Spec references:** knowledge pipeline design.
**Acceptance criteria:**
- [ ] `SidecarHydeClient: HydeClient` implements `generate` via `/v1/completions` with short timeout.
- [ ] `QueryPipeline` accepts an optional `HydeStage`; CLI `new_local` wires one when sidecar is healthy.

**Out of scope:** hypothetical-code generation (separate mode); HyDE quality evaluation.

---

###  — Feasibility probe deliverable
**Goal:** implement the FEASIBILITY_PROBE phase: `SystemProbe`, `FeasibilityReport`, `BabyStepFeasibilityReport`.
**Severity breakdown:** P1 = 1, P2 = 1.
**Effort:** L.
**Dependencies:** 4E.14.
**Gaps covered:**
1. `SystemProbe`, `FeasibilityReport`, `probe_system` — ZERO matches in `.elai_cc/crates/` — P1 — LOW. (Only match is a string in `tool_filter.rs:477` test name.)
2. `FsmState::FeasibilityProbe` exists (ref: `tool_filter.rs:479-485` tests) but no state machine transitions into it — P1 — MED.
3. Spec explicitly calls out this phase — `docs/specs/guide/steps/phase-4F.txt`; `phase-index-3A-4J.txt:271-289`.

**Spec references:** `docs/specs/guide/steps/phase-4F.txt`; `phase-index-3A-4J.txt:271-289`.
**Acceptance criteria:**
- [ ] New module `.elai_cc/crates/orchestrator/src/feasibility.rs` with `SystemProbe` (GPU via `nvml-wrapper`, RAM/disk via `sysinfo`).
- [ ] `FeasibilityReport`, `BabyStepFeasibilityReport`, `BabyStepStubResult`, `BabyStepEscalation` types.
- [ ] Novel-step-stub branch-and-explore harness; dual timeout (25s graceful / 30s hard).
- [ ] Pipeline-builder gate: probe → proceed-or-escalate before executing a concept.

**Out of scope:** NVML discovery resolution (already stubbed at u64::MAX in agent_pool); cross-platform GPU probe parity.

---

### Session newphase-4G — AutonomousGate deliverable
**Goal:** implement the AUTONOMOUS mode gating on top of the existing interactive gate system.
**Severity breakdown:** P1 = 2.
**Effort:** L.
**Dependencies:** newphase-4F.
**Gaps covered:**
1. `AutonomousGate` — ZERO matches in `.elai_cc/` — P1 — LOW.
2. `CliInteractiveGate` exists at `.elai_cc/crates/cli/src/main.rs:7012` but no autonomous variant.
3. Spec lists circuit breaker at max 10 auto-advances + 50 auto baby steps per concept (`phase-index-3A-4J.txt:297, 307`) — no corresponding counter anywhere.
4. `SPLIT_TASK` / `SUBSTITUTE_MODEL` protocols (spec :298-299) — grep returns nothing.

**Spec references:** `docs/specs/guide/steps/phase-4G.txt`; `phase-index-3A-4J.txt:290-319`.
**Acceptance criteria:**
- [ ] `AutonomousGate` wraps `CliInteractiveGate`; six evidence checks from spec.
- [ ] `/mode autonomous|interactive` CLI command; per-session, not global.
- [ ] Circuit-breaker counters stored on `RecoveryContext` (extend existing) and emit `Finding` on trip.
- [ ] Integration test: submit a 3-baby-step task → runs autonomously to PhaseComplete without prompts.

**Out of scope:** TreeAutoAdvancePolicy extensions (spec 4G.7-4G.8 — separate session).

---

### Session newphase-4H — Self-modification Layer 1A
**Goal:** ship the minimum viable self-modification: automatic data-table growth (deprecated_patterns, FailureTaxonomy, lazy-code patterns) with FPR validation. Defer Layer 2/3.
**Severity breakdown:** P1 = 1.
**Effort:** L.
**Dependencies:** newphase-4G.
**Gaps covered:**
1. `DarwinNet`, `GenePool`, `DGM`, `PipelineOutcome`, `self_modification` — ZERO matches (one false-positive for `test_self_modification_locks` in sandbox tests) — P1 — LOW.
2. `AllowedConfigKey` enum absent.
3. `MSI` (Modification Solidification Index) absent.
4. Self-mod safety-layer (dispatch + IPC + filesystem + git) absent at the dedicated module level.

**Spec references:** `docs/specs/guide/steps/phase-4H.txt`; `phase-index-3A-4J.txt:321-353`.
**Acceptance criteria:**
- [ ] `AllowedConfigKey` enum with two initial values: `DeprecatedPattern`, `LazyCodeRegex`.
- [ ] Layer 1A loop reads `PipelineOutcome` SSB rows nightly and proposes additions; human approves via `InterventionRequest`.
- [ ] Auto-revert on >10% degradation over 3 pipelines.

**Out of scope:** Layer 2 staged eval (10/50/200); Layer 3 structural changes; DarwinNet Gene Pool (paper-derived, needs paper-ingestor first).

---

### Session patch-1.35 — Dead-code sweep / scaffolding alignment
**Goal:** formally mark genuinely-scaffolded systems as WIP, remove stale comments, fix test-only `pub` visibility bleed.
**Severity breakdown:** P2 = 13 (all cosmetic but collectively confusing).
**Effort:** S.
**Dependencies:** none (runs after all feature sessions).
**Gaps covered (curated, not exhaustive):**
1. Stale comment at `.elai_cc/crates/orchestrator/src/ssb.rs:580`: "pickup by the orchestrator is deferred" — patch-1.30 fixed this — P2 — HIGH.
2. Stale comment at `.elai_cc/crates/dashboard/src/commands.rs:238-239`: `INTERVENTION_WRITE_DEFERRED` const — 4K.5 note says "lands in 4K.3" which already happened — P2 — HIGH.
3. Stale comment at `.elai_cc/crates/knowledge/src/vector/mod.rs:93`: "current stub" — `embed` is now real (ort_session.rs:128) — P2 — HIGH.
4. Stale DKG "todo!()" comments in tests — `.elai_cc/crates/knowledge/tests/dkg_tests.rs:62, 109, 145, 324, 344, 367, 400, 420, 437, 457` — build_graph is implemented (`src/dkg/mod.rs:148`); comments misleading — P2 — HIGH.
5. Stale CatRAG "todo!()" test labels — `.elai_cc/crates/knowledge/tests/catrag_tests.rs:44, 56, 70` — `compute_overlay` is implemented (`src/catrag.rs:253`) — P2 — HIGH.
6. `AntiSpiralDetector::new` only in tests — `.elai_cc/crates/orchestrator/src/anti_spiral.rs:226`, `tests/adversarial_state_space.rs:49` — P2 — HIGH. Decide: wire or feature-gate.
7. `SupervisionManager::new` only in tests — `.elai_cc/crates/orchestrator/src/supervision.rs:254, 376, 466, 527`, `contracts/tests/supervision_contract.rs:22, 47, 71` — P2 — HIGH.
8. `DeterministicAuditor::new` only in tests — `.elai_cc/crates/orchestrator/src/deterministic_auditor.rs` lines 1182-1429, `contracts/tests/deterministic_auditor_a41_contract.rs:56, 85, 116` — P2 — HIGH. The auditor agent runs in a sub-shell, so this Rust struct is orphaned.
9. `ValidatedSsb::new` has zero callers anywhere — `.elai_cc/crates/orchestrator/src/validated_ssb.rs:15-20` — P2 — HIGH.
10. `IpcHookDispatcher::new` only in tests — `.elai_cc/crates/plugins/src/ipc_hooks.rs:483` (test), `contracts/tests/plugins_hooks.rs:56, 61, 66, 465`, `plugins_adversarial.rs:69, 196, 218` — P2 — HIGH.
11. `DefaultMcpBridge` + `McpConnectionManager` gated behind `mcp` feature, which is NOT in `default`: `.elai_cc/crates/plugins/Cargo.toml:30-31`. All code is dead in prod builds — P2 — HIGH. Decide: enable-by-default or remove.
12. `IpcGateway::new` only in tests — `.elai_cc/crates/orchestrator/tests/agent_isolation.rs:53-54`, `tests/ipc_simulation.rs:28` — P2 — HIGH.
13. `SsbPayload::SpiralDetected`, `::Claim`, `::KnowledgeDegradation`, `::IsolationMetric`, `::BranchMerge`, `::ModelSwap`, `::SupervisionExhausted` — defined with deserializers and contract variants but no production emitter. Grep confirms: only the enum-to-payload dispatch in `ssb.rs` and test proptest constructors — P2 — HIGH.
14. `InterventionRequest` SSB rows written ONLY by tests — `contracts/tests/dashboard_4k3_4k4_4k5_contract.rs:221`; `ssb_viewer.rs:808, 840` are test helpers. No production path emits them — P2 — HIGH.

**Spec references:** N/A.
**Acceptance criteria:**
- [ ] Each stale comment either rewritten to reflect current state OR removed.
- [ ] Each "test-only pub struct" either wired to a real caller OR moved under `#[cfg(test)]` OR marked with `#[doc(hidden)] pub` and a decision note.
- [ ] `mcp` feature is either added to `default` or removed entirely (decision-doc required — `no new dependencies without confirmation` rule).

**Out of scope:** Actually deleting types (requires user confirmation per rules); rewriting tests that reference removed types.

---

### Session patch-1.36 — Sidecar ETC search trigger read-back wiring
**Goal:** use `last_search_trigger` / `last_gg_signals` from the sidecar response in the reasoning loop.
**Severity breakdown:** P2 = 1.
**Effort:** S.
**Dependencies:** patch-1.32, 4D.11.
**Gaps covered:**
1. `SidecarProvider::last_search_trigger` and `last_gg_signals` exist and are populated — `.elai_cc/crates/providers/src/sidecar_client.rs:287-302` — but grepped callers: ZERO outside the module — P2 — MED. AB-MCTS branch scoring (comment at :296) is not implemented.

**Spec references:** sidecar GG intrinsic verification design.
**Acceptance criteria:**
- [ ] After each LLM dispatch, conversation layer reads `last_search_trigger` and, if present, injects a WebSearch baby step.
- [ ] `last_gg_signals` recorded into an SSB `Finding` or Claim row.

**Out of scope:** AB-MCTS full branch scoring (separate design).

---

## Appendix A — Raw findings (for verification)

### A.1 Production-path stubs (file:line)

```
.elai_cc/crates/orchestrator/src/research_tree.rs:1466  // Synthetic report for the idea (real LLM call would happen via IPC)
.elai_cc/crates/orchestrator/src/research_tree.rs:1511  // Deterministic synthetic report for testing / non-LLM mode.
.elai_cc/crates/orchestrator/src/research_tree.rs:1512  // In production, this is replaced by IPC GroundingRequest.
.elai_cc/crates/orchestrator/src/research_tree.rs:589   // PipelineSpec stub — filled by Designer agent (out of 4D scope)
.elai_cc/crates/orchestrator/src/research_tree.rs:729-731 // no LLM calls — real inference happens via IPC GroundingRequest proxy (out of 4D scope)
.elai_cc/crates/orchestrator/src/research_tree.rs:802   // In production, these values come from IPC GroundingRequest responses.
.elai_cc/crates/orchestrator/src/task_runner.rs:17-20   // research_tree.rs:1451 ... substitution point for a real LLM call.
.elai_cc/crates/orchestrator/src/task_runner.rs:87-89   // RUNNER_SYNTHETIC_DURATION_MS + "Real dispatch is a follow-up blocker"
.elai_cc/crates/orchestrator/src/task_runner.rs:347-348 // RuntimeToolLookup "load-bearing design, not a stub"  (OK — annotated)
.elai_cc/crates/orchestrator/src/task_runner.rs:362-377 // RuntimePredicates "MVP doesn't wire policy evaluation"
.elai_cc/crates/orchestrator/src/task_runner.rs:379-388 // RuntimeModelPool "Reports u64::MAX"
.elai_cc/crates/orchestrator/src/task_runner.rs:718-720 // "MVP execution surrogate. Real test runs land when we wire agent_pool::InProcessAgentPool"
.elai_cc/crates/orchestrator/src/web_research_orchestrator.rs:313-345  // "In production, this calls the sidecar LLM via InferRequest IPC. For now, deterministic"
.elai_cc/crates/orchestrator/src/research_pipeline.rs:255 // Stub confidence based on tier. Real implementation would use LLM output.
.elai_cc/crates/orchestrator/src/research_pipeline.rs:554 // Re-synthesis retries are deferred to the LLM-backed implementation
.elai_cc/crates/orchestrator/src/recovery.rs:23        // STUB(3C.2): heartbeat-based death detection not yet implemented.
.elai_cc/crates/orchestrator/src/recovery.rs:515       // STUB(3C.2): Timeout treated same as Crash
.elai_cc/crates/orchestrator/src/context_budget.rs:1703-1712  // STUB: tree-sitter not added to orchestrator due to dep cycle risk with knowledge crate. Wire after dep audit (see A-42-T2).
.elai_cc/crates/orchestrator/src/output_postprocessor.rs:37   // A-42-T2: stub — tree-sitter not wired yet (dep cycle risk with knowledge crate).
.elai_cc/crates/orchestrator/src/output_postprocessor.rs:167  // Stage 4: output formatting restoration (A-42-T2, stub).
.elai_cc/crates/orchestrator/src/coherence_check.rs:51  // Invariants are intentionally excluded (deferred from 3G.3).
.elai_cc/crates/orchestrator/src/gate.rs:122          // deferred to Phase 4G.
.elai_cc/crates/orchestrator/src/ssb.rs:580            // pickup by the orchestrator is deferred (stale — fixed in patch-1.30)
.elai_cc/crates/orchestrator/src/preload.rs:132       // "provider load deferred to first request" (design, not stub)
.elai_cc/crates/web_search/src/client.rs:16           // IPC proxy wiring deferred to 4D.17
.elai_cc/crates/providers/src/vllm_client.rs:463-467  // All backends currently return u64::MAX — real NVML deferred
.elai_cc/crates/providers/src/client.rs:38            // "Ollama provider not yet implemented" (may be stale — check)
.elai_cc/crates/knowledge/src/arag.rs:261             // In production this would use real dual-vector embeddings; for now we use chunk-level
.elai_cc/crates/knowledge/src/arag.rs:272             // In production: dot_product(query_embedding, chunk_embedding).
.elai_cc/crates/knowledge/src/tier3.rs:6              // LLM path deferred to sidecar integration.
.elai_cc/crates/knowledge/src/tier3.rs:414            // template fallback (~40 tokens). LLM-assisted path is deferred
.elai_cc/crates/knowledge/src/project_profile.rs:440  // out of scope for A-32
.elai_cc/crates/knowledge/src/project_profile.rs:1517-1523 // Deferred from CLI wiring in A-32 — LiveCli does not yet have a consent prompt.
.elai_cc/crates/knowledge/src/vector/mod.rs:93        // (or the current stub) — stale; embed IS real
.elai_cc/crates/knowledge/src/pipeline.rs:433         // `let _overlay = self.catrag.compute_overlay(...)?;` — result DISCARDED
.elai_cc/crates/knowledge/src/pipeline.rs:625         // canary: vector module unavailable (embed stub or model missing)
.elai_cc/crates/reasoning/src/verification.rs:331-338 // Stub: network verification not yet implemented.
.elai_cc/crates/reasoning/src/engine.rs:664           // Full semantic contradiction detection deferred to phase 4F.
.elai_cc/crates/reasoning/src/types.rs:366            // caller provides directly (TimeSource deferred)
.elai_cc/crates/reasoning/src/config.rs:56            // role definitions are hardcoded (deferred config-file support)
.elai_cc/crates/rules/src/rule_engine.rs:562          // (false, "out of scope".into())
.elai_cc/crates/rules/src/self_discovery.rs:36        // Conservative default for pipeline count (per-pattern tracking not yet implemented).
.elai_cc/crates/rules/src/self_discovery.rs:334       // tracking is not yet implemented.
.elai_cc/crates/plugins/src/mcp_bridge.rs:156         // Return sanitized result (for now, just clone - can add more sophisticated sanitization)
.elai_cc/crates/plugins/src/mcp_bridge.rs:164-165     // Placeholder implementation - actual connection logic would go here; For now, we simulate a successful connection
.elai_cc/crates/plugins/src/mcp_bridge.rs:173-176     // Placeholder - would query actual MCP server; Ok(vec![])
.elai_cc/crates/plugins/src/mcp_bridge.rs:183-186     // call_tool — only sanitizes, does not dispatch
.elai_cc/crates/plugins/src/tiered_analysis.rs:745    // Deep pattern checks (magic numbers, nesting) are Rust/Python-only for now.
.elai_cc/crates/cli/src/main.rs:4358                  // "Command registered but not yet implemented."
.elai_cc/crates/cli/src/dep_audit.rs:360-361          // Ecosystem::Maven => skip for now; Nuget => skip for now
.elai_cc/crates/tools/src/routing_rules.rs:38         // Config (serde-ready; not yet wired to elai_cc.toml loader)
.elai_cc/crates/dashboard/src/commands.rs:236-239     // INTERVENTION_WRITE_DEFERRED (stale — already wired in 4K.5)
```

### A.2 Test-only `pub` constructors (production callers = 0)

```
SidecarProvider                       .elai_cc/crates/providers/src/sidecar_client.rs:175   (test ctor at :396)
VllmProvider                          .elai_cc/crates/providers/src/vllm_client.rs           (test ctors at :655,668,700,768)
CheckpointManager                     .elai_cc/crates/orchestrator/src/checkpoint.rs:115     (31 test-only ctors)
RecoveryManager                       .elai_cc/crates/orchestrator/src/recovery.rs           (all ctors in tests)
BspEngine                             .elai_cc/crates/orchestrator/src/bsp/engine.rs         (20+ test-only ctors)
SupervisionManager                    .elai_cc/crates/orchestrator/src/supervision.rs:105    (test-only)
AntiSpiralDetector                    .elai_cc/crates/orchestrator/src/anti_spiral.rs        (test-only)
DeterministicAuditor                  .elai_cc/crates/orchestrator/src/deterministic_auditor.rs:362 (test-only)
ValidatedSsb                          .elai_cc/crates/orchestrator/src/validated_ssb.rs:15   (NO callers at all)
IpcHookDispatcher                     .elai_cc/crates/plugins/src/ipc_hooks.rs               (test-only)
DefaultMcpBridge                      .elai_cc/crates/plugins/src/mcp_bridge.rs:81           (test-only + feature-gated)
McpConnectionManager                  .elai_cc/crates/plugins/src/mcp_lifecycle.rs:87        (test-only + feature-gated)
HeartbeatMonitor                      .elai_cc/crates/plugins/src/mcp_lifecycle.rs            (test-only + feature-gated)
IpcGateway                            .elai_cc/crates/orchestrator/src/ipc_gateway.rs         (test-only)
WebResearchOrchestrator               .elai_cc/crates/orchestrator/src/web_research_orchestrator.rs:78 (test-only)
WebContentIndex                       .elai_cc/crates/knowledge/src/web_arag.rs:98           (test-only)
SearxClient                           .elai_cc/crates/web_search/src/client.rs:18            (test-only)
HydeStage                             .elai_cc/crates/knowledge/src/hyde.rs:78               (test-only)
MistakeBook {insert,query,clear_for}  .elai_cc/crates/orchestrator/src/mistake_book.rs       (test-only use-sites for each fn)
TreeTraverser / traverse_with_answers .elai_cc/crates/reasoning/src/traversal.rs:30,124      (test-only)
{load,save,delete}_interrogation_tree .elai_cc/crates/rules/src/interrogation_store.rs:22,62,86,112 (test-only)
StubSearchBackend is the ONLY         .elai_cc/crates/orchestrator/src/research_pipeline.rs:40  SearchBackend impl
DolaDecoder                           .elai_cc/crates/providers/src/layer_config/dola.rs:490 (test-only)
compute_vector_via_sidecar            .elai_cc/crates/providers/src/layer_config/steering.rs:501 (zero callers anywhere)
build_dispatch_context                .elai_cc/crates/orchestrator/src/dispatcher.rs:135     (test-only)
run_integration_checkpoint            .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs:1286 (test-only)
record_integration_checkpoint         .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs:1038 (test-only)
attempt_recovery / recipe_for         .elai_cc/crates/orchestrator/src/recovery_recipes.rs   (test-only)
scaffold::apply_template              (sole prod caller: dashboard/commands.rs:147 — NOT task_runner)
SsbPayload::Claim                     no production emitter (emitter only in deserialize dispatch)
SsbPayload::ModelSwap                 no production emitter
SsbPayload::KnowledgeDegradation      no production emitter
SsbPayload::IsolationMetric           no production emitter
SsbPayload::BranchMerge               no production emitter
SsbPayload::SpiralDetected            no production emitter
SsbPayload::SupervisionExhausted      no production emitter outside BSP (dead)
SsbPayload::InterventionRequest       emitter only in ssb_viewer tests
SsbPayload::WebResearchStep           emitter only in contract test
```

### A.3 Feature-flag dead code (default = off)

```
.elai_cc/crates/plugins/Cargo.toml:29-31     default = []
                                              mcp = []
  → DefaultMcpBridge, McpConnectionManager, HeartbeatMonitor, RetryPolicy: all dead in default builds.
  → .elai_cc/crates/plugins/src/lib.rs:28-31 re-exports are #[cfg(feature = "mcp")].
```

### A.4 Spec-vs-code deltas by phase

```
Phase 1I — KnowledgeEngine trait        defined (knowledge/src/lib.rs:244)  IMPLEMENTATIONS: 0
Phase 3C — CheckpointManager/Recovery   defined             PRODUCTION CALLERS: 0
Phase 3D — BspEngine parallel exec      defined             PRODUCTION CALLERS: 0
Phase 3F — Rules in baby-step loop      DefaultRuleEngine exists in CLI, NOT in task_runner
Phase 3F — MistakeBook query loop       defined (orchestrator/src/mistake_book.rs)  PRODUCTION CALLERS: 0
Phase 3G/3J — AutonomousGate            absent
Phase 4D — Research LLM path            synthetic only (Session 4D.10)
Phase 4D — SearxClient usage            WIRED via SearxSearchBackend (Session 4D.11 DONE 2026-04-17)
Phase 4D — WebResearchOrchestrator      WIRED via DefaultWebResearchRunner (Session 4D.12 DONE 2026-04-17)
Phase 4E — Integration checkpoints      WIRED into task_runner (Session 4E.13 DONE 2026-04-17) at BabyStep/Step/Phase/Concept boundaries; runner = NoOpIntegrationRunner until CommandIntegrationRunner swap
Phase 4E — Baby-step feasibility probe  absent
Phase 4F — SystemProbe / FeasibilityReport  absent (ZERO grep hits)
Phase 4G — AutonomousGate               absent
Phase 4G — Circuit breaker counters     absent
Phase 4G — SPLIT_TASK / SUBSTITUTE_MODEL  absent
Phase 4H — Self-modification (DGM)      absent
Phase 4H — AllowedConfigKey             absent
Phase 4H — MSI                          absent
Phase 4I — Media validator              absent
Phase 4J — Empire integration           absent (conditional phase — OK to skip but note)
A-19   — InterrogationTree traversal    defined, zero production callers
A-20   — MistakeBook integration        see Phase 3F above
A-42-T2 — tree-sitter pretty-print     explicit stub (output_postprocessor.rs:37)
```

## Appendix B — Items I was UNSURE about

| Verdicts | Count |
|---|---|
| REAL GAP → new session | 4 |
| INTENTIONAL DESIGN → doc-comment | 3 |
| DEAD CODE → removal session | 3 |
| NEEDS USER DECISION | 0 |

All 10 items resolved after grep/module-doc investigation. Previously-ambiguous items #3, #6, #7 resolved as follows (see entries below for full evidence):
- #3 Steering → DEAD CODE (zero dashboard-crate references; post-MVP roadmap per memory note `project_vram_tier_plan.md` "2x3090 coming for 32B+TQ testing")
- #6 AntiSpiralDetector → REAL GAP (module-doc declares per-agent runtime detection; defense-in-depth with Claude-Code `mcp_think` budget)
- #7 DeterministicAuditor → REAL GAP (module-doc declares "runs 5 deterministic checks BEFORE the LLM auditor agent"; A-12 scheduled item with prereqs A-4/A-6 already landed)

Things that *look* like gaps but might be intentional MVP scoping or my grep missing a dynamic caller. Please confirm:

1. **`elai_cc_agent` binary spawn.** `agent_pool::spawn` is implemented at `.elai_cc/crates/orchestrator/src/agent_pool.rs:270-291`, but `InProcessAgentPool` itself has only test callers. Is `agent_pool::spawn` hit at runtime from CLI, just not with a test-traced call? My grep found no `InProcessAgentPool::new` in any non-test file.
   **Verdict:** REAL GAP
   **Confidence:** HIGH
   **Evidence:**
   - `.elai_cc/crates/orchestrator/src/task_runner.rs:717-720` — in-source comment "Real test runs land when we wire `agent_pool::InProcessAgentPool` into the runner — a separate blocker."
   - Grep confirms `InProcessAgentPool::new` appears only in `tests/`, `src/agent_pool.rs` tests mod, `src/bsp/engine.rs` tests mod, and `contracts/tests/*` — zero non-test callers in `crates/cli/` or any binary entrypoint.
   - `.elai_cc/crates/cli/src/main.rs` grep for `agent_pool|AgentPool|launch_agent_process|spawn_agent` returns zero matches.
   - Phase spec `docs/specs/guide/steps/phase-4A.txt:60,89,264,321` mandates the `elai_cc_agent` binary + orchestrator-spawned connection sequence (listen-before-spawn, SO_PEERCRED, etc.) — load-bearing for MVP agent isolation.
   **Recommended action:** "Add Session wire-4A — wire `InProcessAgentPool` + `launch_agent_process` into `task_runner::run_baby_step` replacing the synthetic `emit_test_outcome` surrogate. Insert before any session that depends on real agent execution."

2. **`DolaDecoder` Rust-side vs sidecar DoLa.** The sidecar does DoLa internally (`python/elai_inference/server.py:642 DoLaSteeringProcessor`). Is the Rust `DolaDecoder` intentional dead code (server authority) or a duplicated implementation? Decision-doc needed before patch-1.33 removes it.
   **Verdict:** DEAD CODE
   **Confidence:** HIGH
   **Evidence:**
   - `.elai_cc/crates/providers/src/layer_config/dola.rs:252` — `DolaDecoder` struct; all callers outside the file are `crates/contracts/tests/dola_contract.rs` + `metrics/dola_e2b_eval.py` (a Python re-implementation, not a Rust caller).
   - `.elai_cc/python/elai_inference/server.py:642` — `DoLaSteeringProcessor` is the production path; sidecar is single source of authority (per CLAUDE.md project-path note "Sidecar does DoLa + TurboQuant").
   - `docs/specs/guide/steps/` grep for `DoLa|dola` returns 0 matches — no phase spec requires a Rust-side decoder.
   - `.elai_cc/Cargo.toml:26` `publish = false` applies workspace-wide — removal is safe, no downstream crates depend on `DolaDecoder` as a public API.
   **Recommended action:** "Remove `DolaDecoder`, `DolaConfig`, `DolaOutput`, `DolaError`, `ModelSizeStatus` + `dola_contract.rs` in Session patch-1.33. Zero external consumers (verified: only in-file tests + Python re-implementation in `metrics/`, which is a scratch eval, not a consumer of Rust types)."

3. **`compute_vector_via_sidecar` zero callers.** Could be an API surfaced for a dashboard UI "compute steering" button that doesn't exist yet. Confirm if steering is truly MVP-scope or deferred.
   **Verdict:** DEAD CODE
   **Confidence:** HIGH
   **Evidence:**
   - `.elai_cc/crates/providers/src/layer_config/steering.rs:501` — `compute_vector_via_sidecar` defined; zero Rust callers outside the file.
   - Grep `steering|Steering|ActAdd|compute_steering|SteeringVector` across `.elai_cc/crates/dashboard/` = **0 matches**. Dashboard has no steering UI, no Tauri command exposes steering.
   - `.elai_cc/benches/model_capability/run_benchmark.py:1128,1173` + `metrics/compute_steering_vector.py` both POST directly to `/v1/compute_steering` (sidecar endpoint), bypassing the Rust wrapper.
   - `docs/specs/guide/steps/` grep for `steering|SteeringVector|ActAdd` returns 0 matches — no phase spec schedules steering into MVP.
   - Memory note `project_vram_tier_plan.md` places steering behind "2x3090 coming for 32B+TQ testing" — explicitly post-MVP roadmap.
   **Recommended action:** "Remove `compute_vector_via_sidecar` + `SteeringConfig` Rust API in Session patch-dead-steering. Python scripts in `benches/` + `metrics/` already hit the sidecar endpoint directly, so deletion is zero-impact. When steering ships post-MVP (new GPU tier), add the Rust wrapper back with a live dashboard control."

4. **`KnowledgeEngine` trait with zero impls.** `populate_tier3`, `ShortCoder`, `DkgEngine`, `QueryPipeline` are all used — but the `KnowledgeEngine` trait itself has no implementor. Trait may be aspirational; the CLI wires sub-components directly. Is the trait meant to stay as a design marker or be deleted?
   **Verdict:** INTENTIONAL DESIGN
   **Confidence:** HIGH
   **Evidence:**
   - `.elai_cc/crates/knowledge/src/lib.rs:242-250` — in-source doc says "Top-level knowledge engine trait. Implementations aggregate DKG, vector, and BM25 sub-engines. **Full impl deferred to Phase 1I**."
   - Grep `impl.*KnowledgeEngine` + `dyn KnowledgeEngine` = 0 matches → no trait objects, not used via indirection.
   - Referenced as a design marker in sibling traits: `crates/orchestrator/src/evidence.rs:137` + `gate.rs:118` say "consistent with KnowledgeEngine/GrcEngine trait pattern" — it is the canonical shape for future-agglomerator types.
   **Recommended action:** "Add a module-doc at `crates/knowledge/src/lib.rs:242` saying: \"Trait is a scheduled facade for Phase 1I; sub-engines (DkgEngine, ShortCoder, populate_tier3) are wired directly by CLI in MVP. Do not remove — used as architectural template by EvidenceGate/GrcEngine.\" No code change."

5. **`SsbPayload` variants with no emitter.** Claim, ModelSwap, KnowledgeDegradation, IsolationMetric, BranchMerge, SpiralDetected — are these intentional future-proofing (schema stable) or leftover from abandoned features? Deleting them is schema-breaking.
   **Verdict:** INTENTIONAL DESIGN
   **Confidence:** HIGH
   **Evidence:**
   - `.elai_cc/crates/orchestrator/src/ssb.rs:597-630` — `SsbPayload` enumerates all 31 action types; file-level doc "each variant maps 1:1 to an [`SsbActionType`] (31 total)".
   - `ssb.rs:1531-1597,2439-2549,2785-2850` — every "unused" variant has `fn parse_payload` decode arm + proptest generator → schema round-trips enforced as invariant even without emitters.
   - `.elai_cc/crates/dashboard/src-frontend/src/DiffViewer.svelte:13` — frontend typing is `payload_json: Record<string, unknown>` (generic); no variant-name coupling, BUT the `action_type` string discriminant (already stable) is consumed.
   - Removing a variant is an `rmp_serde` schema break (old SSB files on disk would fail to decode) — confirmed by the decode tests at `ssb_viewer.rs:220,268`.
   **Recommended action:** "Add a module-doc at `crates/orchestrator/src/ssb.rs:591` saying: \"All 31 variants are schema-committed. Variants without current emitters (Claim, ModelSwap, KnowledgeDegradation, IsolationMetric, BranchMerge, SpiralDetected) are forward-compat for phases 3F/4G/4H; do not remove — on-disk SSB files and the dashboard `payload_json` pipeline depend on the discriminant set being stable.\" No code change."

6. **`AntiSpiralDetector` wired via a non-obvious path?** Memory notes enforce anti-spiral rules at Claude-Code level via `mcp_think`. Is the Rust `AntiSpiralDetector` supposed to ever be called at orchestrator runtime, or is it Claude-Code-layer only by design?
   **Verdict:** REAL GAP (defense-in-depth intended)
   **Confidence:** HIGH
   **Evidence:**
   - `.elai_cc/crates/orchestrator/src/anti_spiral.rs:1-4` — module doc: *"Anti-spiral / hot-loop detection for GRC agents. Monitors repeated (step_type, topic, content_hash) tuples per agent. Triggers escalation after SPIRAL_THRESHOLD identical steps, then HardStop if the agent ignores the escalation and repeats again."* Unambiguously per-agent runtime detection at orchestrator layer.
   - `.elai_cc/crates/orchestrator/src/anti_spiral.rs:17-30` — SPIRAL_THRESHOLD=3, RING_SIZE=4, ESCALATION_MESSAGE constants defined. The `SpiralResult::EscalationSignalNeeded` / `HardStop` signals are the interface the task_runner should consume.
   - CLAUDE.md's anti-spiral rule at Claude-Code layer is a SEPARATE concern (file-edit counter). Orchestrator-side detection watches GRC step streams — different data, different decision surface. Both layers firing is defense-in-depth by design.
   - `SsbPayload::SpiralDetected` variant exists (ssb.rs:619) as the emission channel; needs wiring to `AntiSpiralDetector::check_step` from task_runner's GRC loop.
   **Recommended action:** "Add Session wire-antispiral — task_runner's reasoning loop calls `AntiSpiralDetector::check_step` per GRC step. On `EscalationSignalNeeded`, inject the `ESCALATION_MESSAGE` into the agent's next turn. On `HardStop`, emit `SsbPayload::SpiralDetected` and terminate the task with a Finding. ~1 session. Depends on Session wire-4A (real agent execution) since anti-spiral on synthetic baby steps is meaningless."

7. **`DeterministicAuditor`.** The auditor runs as a sub-agent shelled out via Python/Claude Code per the workflow. The Rust `DeterministicAuditor` struct may be an unused library API. Remove or wire?
   **Verdict:** REAL GAP (pre-LLM-auditor layer)
   **Confidence:** HIGH
   **Evidence:**
   - `.elai_cc/crates/orchestrator/src/deterministic_auditor.rs:1-14` — module doc: *"Deterministic-first auditor: runs 5 deterministic checks BEFORE the LLM auditor agent. Language-parameterized via ProjectProfile. Spec: anti-staleness-enforcement.txt §3.3, code-completeness-enforcement-design.txt Layer 4. Step: A-12. Prerequisites: A-6 (VERSION_GATE), A-4 (ProjectProfile)."* This is a preprocessing layer, NOT a replacement for the Claude-Code auditor.
   - Checks enumerated: (1) Build check via `ProjectProfile.commands.build`, (2) VERSION_GATE (deprecated_patterns), (3) FILE_SIZE_CENSUS, (4) LAZY_CODE_SCAN, (5) COMPLETENESS + MODULE_COHERENCE. Early-exit on first BLOCK. Only after all pass does the LLM auditor run.
   - Prerequisites already landed: A-4 (ProjectProfile) per patch-1.26 + A-6 (VERSION_GATE) per multiple A-series commits. A-12 itself is the scheduled wiring step.
   - `crates/contracts/tests/deterministic_auditor_a41_contract.rs` exists as the acceptance surface — A-41 is the "auditor completions" contract (per earlier audit).
   - CLI has no `elai audit` subcommand today — only `dep-audit` (dependency audit, different concept at `crates/cli/src/main.rs:1180`). The deterministic auditor is meant to run inline during the AUDITING FSM state, not as a standalone CLI.
   **Recommended action:** "Add Session wire-a12 — invoke `DeterministicAuditor::run_checks(&profile, &changed_files)` from the FSM's AUDITING state transition handler. Emit findings as `SsbPayload::Finding` rows with severity Block/Warn. Early-exit the AUDITING→MEMORIZING transition on any Block. Depends on FSM transition hook in orchestrator (already exists via `mcp_fsm_agent_transition`). ~1-2 sessions."

8. **`run_research_pipeline` CLI path.** This is ONE code path that uses `StubSearchBackend`. It's plausibly a deprecated CLI command overtaken by the web_research_orchestrator flow. Confirm which is canonical.
   **Verdict:** DEAD CODE
   **Confidence:** HIGH
   **Evidence:**
   - Git log `.elai_cc`: `ea82fd7` (A-45+A-43 "Oracle research pipeline") → `345fad5` (A-47 `/research` CLI) → later `bd73051` (4D.17-4D.18 "WebResearchOrchestrator + end-to-end benchmark"). `run_research_pipeline` is A-43 era; `WebResearchOrchestrator` is the 4D.17 superseder.
   - `.elai_cc/crates/cli/src/main.rs:4374-4382` — the CLI `/research` command still hardcodes `StubSearchBackend::new()` with `EMPTY_TIER_TOML`, so production "research" returns only caveats; the real search path is `web_research_orchestrator`.
   - `.elai_cc/crates/web_search/src/extractor.rs:25` — "Signature locked for 4D.17 WebResearchOrchestrator consumption" confirms 4D.17 is the canonical consumer.
   - Zero non-test callers of `WebResearchOrchestrator::new` outside contract tests → BOTH paths are un-wired in CLI, BUT `WebResearchOrchestrator` is the spec-blessed replacement (phase-4D.17).
   **Recommended action:** "Remove `run_research_pipeline`, `StubSearchBackend`, `phase_a_formulate_queries`..`phase_g_compose_answer`, `SearchBackend`, `SearchResult`, `research_pipeline.rs` + a47_research_cli_contract test + the `/research` CLI branch at `crates/cli/src/main.rs:4374-4382`, in Session patch-dead-research. Zero external consumers (verified: only test file + CLI branch that feeds StubSearchBackend). Replacement path is `WebResearchOrchestrator` (which is separately flagged as un-wired elsewhere in this doc — that is the REAL GAP)."

9. **`output_postprocessor` A-42-T2 stub.** The tree-sitter dep-cycle note (context_budget.rs:1703-1712, output_postprocessor.rs:37) says "wire after dep audit." Is that audit done? If yes, this is a quick win.
   **Verdict:** REAL GAP
   **Confidence:** HIGH
   **Evidence:**
   - `.elai_cc/crates/orchestrator/Cargo.toml:11-33` — orchestrator ALREADY depends on `elai-knowledge` (line 11) AND `tree-sitter = "0.26"` + `tree-sitter-rust = "0.24"` + `tree-sitter-python = "0.25"` (lines 31-33). The dep cycle has been resolved; the stub's precondition is satisfied.
   - `.elai_cc/crates/orchestrator/src/context_budget.rs:1705-1708` `restore_output_formatting` still returns `code.to_owned()` — identity function. `output_postprocessor.rs:174` still calls the stub.
   - `.elai_cc/crates/contracts/tests/context_budget_contract.rs:511-515` `contract_restore_output_formatting_is_identity_stub` — contract currently CODIFIES identity; wiring real tree-sitter pretty-print is a contract change.
   - `CLAUDE.md` Zero Lazy Code rule: "no pass bodies, TODO, placeholder returns" — the stub violates this hard rule.
   **Recommended action:** "Add Session patch-tree-sitter-format — replace stub `restore_output_formatting` with a `tree-sitter` pretty-print using the `Tree::root_node().utf8_text()` walk + indent-by-depth. Update the identity contract to a formatting-invariance contract. Insert before any session that relies on deterministic LLM output formatting. ~60-min scope."

10. **`mcp` cargo feature.** Default = `[]` means MCP bridge code is dead in production builds. But the workflow heavily uses MCP (mcp_fsm_*, mcp_think, etc.). Are those routed through a different path (the Python MCP client at Claude Code level)? If yes, the Rust `DefaultMcpBridge` is orphaned; decide: remove or wire.
    **Verdict:** INTENTIONAL DESIGN
    **Confidence:** HIGH
    **Evidence:**
    - `.elai_cc/crates/plugins/Cargo.toml:29-31` — `[features] default = [] mcp = []`. No crate in the workspace sets `features = ["mcp"]` (grep returns 0 matches) → `DefaultMcpBridge` + `McpConnectionManager` + `HeartbeatMonitor` never compile into the default production build.
    - `CLAUDE.md` "GRC tools: mcp_think(…) …" — every `mcp_*` call the workflow makes is served by the Python/Node MCP server spawned by Claude Code, not by Rust. Memory note `project_search_architecture.md` confirms sidecar architecture.
    - `.elai_cc/crates/plugins/src/mcp_bridge.rs:81,162` + `src/mcp_lifecycle.rs:87` — `DefaultMcpBridge` is a library abstraction for FUTURE in-process MCP server hosting (e.g. if the dashboard embeds its own MCP endpoint for Tauri → Rust tool calls), but that's not MVP.
    - `.elai_cc/Cargo.toml:26` `publish = false` → dead-feature code isn't exposed externally, so leaving it behind a feature flag is zero-cost.
    **Recommended action:** "Add a module-doc at `crates/plugins/src/mcp_bridge.rs:1` saying: \"This crate is behind the `mcp` feature (default=off). The MVP workflow routes all `mcp_*` tool calls through the Claude-Code-hosted Python MCP server; this Rust bridge is scaffolding for a future in-process MCP host (e.g. dashboard Tauri → Rust tool invocation). Do not remove without confirming the dashboard roadmap does not require in-process MCP.\" No code change."


Other things worth knowing now (from Appendix)
Sidecar HTTP body ignores dola/turboquant/steering config — sidecar_client.rs:234-239 only sends prompt, max_tokens, temperature. Server side does DoLa + TurboQuant but the client never tells it when. All seven config fields you tuned are dead on the wire.
CheckpointManager / RecoveryManager / BspEngine all zero prod callers — verified on CheckpointManager. Task failures in task_runner just emit a Finding and continue, no checkpoint/recovery.
mcp cargo feature not in default — all DefaultMcpBridge, McpConnectionManager, HeartbeatMonitor code compiles out of production builds.
Phases 4F, 4G, 4H, 4I grep-empty — FeasibilityProbe, AutonomousGate, self-modification, media validator. The commit history may call them "done" but code isn't there.
run_integration_checkpoint only test-exercised — 4E deliverable exists but never fires; task_runner emits TestOutcome{passed:true} unconditionally.