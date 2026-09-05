D2 decisions: CLI /task handler — planning mode L1 designer wiring

What files touched:
- crates/cli/src/main.rs (lines 4145-4159, SlashCommand::Task handler)

Decision: Option A (un-stub the /task slash command)
Reason: matches A-47 spec, smaller diff, REPL-invocation is sufficient for MVP.
Option B (/task as top-level subcommand) deferred per task scope.

Approach:
- Cannot call drive_web_research_task directly (private fn, requires private ResolvedConfig).
- Cannot call start_task_runner and submit TaskSubmitted SSB row (races, complex lifecycle).
- Correct approach: call DefaultWebResearchRunner::run() directly from the handler.
  - All required types are pub-exported from elai_orchestrator.
  - WebResearchStepEmitter lives in elai_orchestrator::web_research_orchestrator (pub mod).
  - SsbWebResearchStepEmitter is pub-exported; use it when self.ssb is Some.
  - When self.ssb is None, define a local NoopWebResearchStepEmitter inline.
- Async runtime: one-shot tokio::runtime::Builder::new_current_thread().enable_all()
  mirroring the fsm_transition_sync pattern (line 3622).
- VRAM tier: Vram12Gb (dashboard default at lib.rs:788).
- Sidecar URL: read ELAI_SIDECAR_URL env var, fallback to DEFAULT_SIDECAR_SEARCH_URL.
- Clock: Arc::new(elai_core::clock::OsClock).
- Summary output: print iterations, status, sources_used to stdout.
- Error handling: graceful eprintln, no panic, no unwrap in prod path.

Expected test outcome:
- cargo test -p elai-cli: D1 baseline 180 passing (no regressions; no new tests for D2 since integration test is D5).
- cargo check -p elai-cli + --workspace: clean.

Unknowns:
- Live sidecar at 8100 may or may not have SearXNG wired; runner will return Err if search fails.
  The handler prints the error gracefully (no panic).
- No CancelHandle: the one-shot run has no cancellation; acceptable for MVP per spec.
