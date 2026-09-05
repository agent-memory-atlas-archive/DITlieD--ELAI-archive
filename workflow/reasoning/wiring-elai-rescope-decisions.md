WIRING-FIX-ELAI RE-SCOPE DECISIONS (post Phase-0 grounding)
Written 2026-06-02. Grounds: workflow/reasoning/wiring-elai-phase0-grounding.md (grounding workflow wf_1326a305-632),
WIRING-FIX-ELAI-PLAN.md (original plan), WIRING-FIX-ELAI.md (design), WIRING-ENFORCEMENT-RESEARCH-2026-06-02.md.
Verdict: PROCEED_WITH_RESCOPE. User directive: full work, do everything, orchestrate with agents.

SIX CORRECTIONS THE GROUNDING FORCED (design doc misread live code)
1. Type rename: EntrypointSet/EntrypointSetLoader are actually WiringEntrypointSet/WiringEntrypointSetLoader (DRIFT-4,
   wiring_contract.rs:202-270). Every reference must use the Wiring-prefixed names or imports do not compile.
2. Phase 1.2 is a NO-OP: SsbActionType (ssb.rs:4650), FsmSubstate (fsm.rs:125), RouteDecisionEventKind (ssb.rs:24375),
   SkillEventPhase (ssb.rs:21235) ALREADY lack #[non_exhaustive] and are already matched wildcard-free. Replace
   "strip non_exhaustive" with a compile-time exhaustiveness REGRESSION TEST that locks the absence of wildcard arms.
3. DkgReachabilityAuditor is a self-certifying stub: is_reachable (wiring_contract.rs:581) is a real BFS, but the driver
   (wiring_contract.rs:903-914) FABRICATES the graph by adding a direct edge from every entrypoint to every declared
   produces_symbol, so UnreachableSymbol can never fire and W-8 is unenforced. Sound reachability = BUILD-NEW: wire
   elai_knowledge::symbol_graph::SymbolGraphCache into the graph builder + exclude test-file callers (R7).
4. ObservableEffectRunner fail-open is in the DRIVER not the runner: runner uses a manual deadline loop, does Err+emit
   BrokenObservable; the driver drops it with `let _ = runner.run_effect(...)` at wiring_contract.rs:945.
5. CI: .github/workflows/wiring-audit.yml is ABSENT. Live equivalent is the "Wiring contract audit (C-27 T-0)" step in
   .elai_cc/.github/workflows/windows.yml:112-119 (build-failing, but Windows-only + [ci skip]-bypassable). Add a
   cross-platform/Linux leg (R3: drop the opt-in flags).
6. Seed files exist but EMPTY: smoke-manifest.toml (0 features), wire-dark-allowlist.toml (0 rows), and
   [wiring.entrypoints] in elai_cc.toml:106-110 is COMMENTED OUT. Populate them, do not just reference.

THE WORST FAIL-OPEN (autonomous orphan-ship): task_runner.rs:13499 sets test_flipped = test_rows.is_empty() || had_green
=> vacuously TRUE on zero test rows. Reinforced by TestFlippedGate::check (evidence.rs:458) + Evidence::all_passed
(autonomous_gate.rs:355). A cargo-green run with no real evidence auto-advances EXECUTING->AUDITING. This is the bug.

TOOLING (anti-slopsquatting, verified live): cargo-llvm-cov v0.8.7 (JSON flag is --json NOT --format=json),
cargo-machete v0.9.2, knip v6.15.0 (no native fail-on-zero-modules, wrap it), vulture v2.16, tracing-test v0.2.6
(logs_contain does NOT auto-fail), cargo-mutants v27.0.0 (--in-diff exists, no --shard, exits 0 even on survived =>
parse stdout). rustc -D dead_code EXEMPTS pub items (#120079 open) => pub(crate) discipline; -C instrument-coverage real.
CHEAP PATCHES: e45_cross_language_drift_contract.rs:294 asserts 394, live is 482 -> fix to live constant. SCHEMA 205/TV 482.

SUB-PHASE BREAKDOWN (sequencing forced by file coupling)
SP1 fail-close static + cross-lang: wiring_contract.rs (WiringEntrypointSetLoader empty/absent=FAIL :251-267, driver
   skip-on-empty=FAIL :897-901, WiringAuditError ZeroContracts variant :1038), cross_lang_diff.py action-type
   unconditional + wired into CI, e45 394->482 patch, exhaustiveness regression test (replaces no-op 1.2),
   elai_cc.toml uncomment [wiring.entrypoints].
SP2 reachability (BUILD-NEW): wire SymbolGraphCache into DkgReachabilityAuditor graph builder + test-caller exclusion.
   TOUCHES wiring_contract.rs -> sequential after SP1.
SP3 Layer-3 coverage runner (BUILD-NEW): cargo-llvm-cov --json runner, count==0 new-fn=BLOCK, WIRE-DARK stub registry,
   smoke-manifest population + enforcement, knip-zero-module wrapper. Mostly new files -> can design in parallel.
SP4 Layer-4 trace (driver Err propagation wiring_contract.rs:945, db-not-found=FAIL) + auditor-authored trace-assertion
   harness (SimpleSpanProcessor only) + tracing-test mandatory-assert rule. TOUCHES wiring_contract.rs -> after SP2.
SP5 autonomous + freeze: task_runner.rs:13499 vacuous-true=>FAIL, evidence.rs (AntiFabricationGate absent-row=>FAIL,
   DiagnosticProbeGate None=>FAIL, check_transition empty-set=>FAIL), autonomous_gate.rs require Layer3+4 on
   EXECUTING->AUDITING; freeze: locked_keys_registry.toml append (inner) + outer-repo registry/hook (BUILD-NEW),
   wire-dark-allowlist + smoke-manifest population.

COUPLING: wiring_contract.rs in SP1/SP2/SP4 => those three are sequential. evidence.rs/task_runner.rs/autonomous_gate.rs
only in SP5. SP3 + the cross_lang_diff/toml/outer-repo pieces are file-independent. Same crate compiles together =>
NO simultaneous parallel editors on the orchestrator crate; compile-check between waves.

ORCHESTRATION: emulate orchestrate same-project in-band. Controller = main LLM. WAVE-0 = parallel read-only design
workflow (7 agents) emitting exact compile-ready edit specs + module designs + cross-file contracts. Then execute
wave-by-wave: executor agent(s) apply edits (FSM EXECUTING) -> cargo check -> next wave; auditor 1 cycle at end (sole
test runner) -> memory-writer -> single commit (maintainer voice, no AI trailer). Dogfood: the wiring gate must pass its own
WIRING_CONTRACT driven from `elai wiring audit` leaving a real SSB row.

RESIDUAL RISK (carried from research): no sound RTA for Rust+tokio+dyn Trait; SymbolGraphCache reachability is the best
approximation, Layer-3 runtime coverage is the load-bearing backstop. SCIP has no call-graph edges; [wiring.entrypoints]
trait-dispatch enumeration can go stale. Wire-dark has no external benchmark; instrument the rate via an SSB action_type.
