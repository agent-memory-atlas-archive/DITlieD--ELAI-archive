orchestrator crate-split boundary proposal (ram-build-efficiency Phase 6, unit H, DOC-ONLY)

Author: session ram-P6 (2026-07-11). Status: PROPOSAL. No product file was edited producing this doc; the split it describes is a future letter-plan.
Target of proposal: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs (24,418 lines on disk 2026-07-11; the plan header's "23,284" is stale drift, the boundary math below uses the measured 24,418).
Grounding: structure grep of task_runner.rs (RECON_NEEDED R6, settled) + scoped seam reads (header 1-60, drive_task 6116-6236) + the lib.rs facade blocks (lib.rs:1176 `pub mod task_runner;`, lib.rs:1502-1505, lib.rs:2407-2438, lib.rs:2462) + the recorded ssb.rs precedent (ssb.rs is a 1632-line root shell over an ssb/ dir whose payloads.rs measures 20,044 lines TODAY).

WHY THIS IS A PROPOSAL, NOT A SPLIT
task_runner.rs is the single largest hand-editable source file in the crate and a first-order cargo build-cost + RAM driver (rust-analyzer holds the whole 24k-line module in one analysis unit; every touch reparses it). MR-1 (maintainability, 1000-line boundary) is violated 24x over. But a mechanical "move it into a directory" reproduces the ssb.rs outcome verbatim (see anti-ssb clause b): ssb.rs WAS split into an ssb/ submodule dir and the god-content simply relocated into ssb/payloads.rs, which is now 20,044 lines — a file LARGER than task_runner.rs, i.e. the split reduced zero real complexity. This doc specifies boundaries that actually keep every target near or under 1000 lines, names the ONE seam (drive_task) that cannot get there by file-move alone, and preserves the pub-use facade so cli / contracts / dashboard-v2 consumers never change an import.

--------------------------------------------------------------------------------
(a) LAYOUT: task_runner/ submodule dir, task_runner.rs kept as the crate-root file (mirror the ssb.rs rule)

The proven, facade-preserving mechanism already lives in this crate at ssb.rs:45-53:
    // ssb.rs (crate-root file, 1632 lines) declares:
    mod schema; mod action_type; mod payloads; mod store;   // ssb.rs:45-48
    pub use schema::*; pub use action_type::*;               // ssb.rs:50-53
    pub use payloads::*; pub use store::*;
Every symbol defined in ssb/payloads.rs remains reachable to the outside world as `crate::ssb::Foo` / `elai_orchestrator::Foo`, because the glob re-export flattens the sub-files back up to the module root. Compilation, external paths, and the pub-use facade all survive the move; ONLY the on-disk file each symbol lives in changes.

Apply the identical shape to task_runner:
    - RENAME task_runner.rs -> task_runner/mod.rs OR (ssb.rs style, preferred for path-stability of the crate-root file) KEEP task_runner.rs and add `#[path]`-free `mod` declarations pointing at task_runner/<sub>.rs siblings. Rust resolves `mod foo;` from `task_runner.rs` to either `task_runner/foo.rs` (sibling dir named after the file's stem) — this is the ssb.rs arrangement exactly (ssb.rs + ssb/ dir coexist). So task_runner.rs STAYS as the crate-root file and gains only:
        mod constants; mod cursor; mod web_launch; mod runtime_adapters;
        mod baby_step_executor; mod fixer; mod web_research; mod web_scaffold;
        mod config; mod resolved_config; mod post_execute_hooks; mod runner_loop;
        mod decomposition; mod drive_task; mod dispatch; mod emit;
        mod derive_evidence; mod gate_audit; mod integration_checkpoints;
        pub use constants::*; pub use cursor::*; pub use web_launch::*; ...  // one glob per submodule
        #[cfg(test)] mod tests;
    - task_runner.rs shrinks to: the module-level `//!` doc (lines 1-32), the shared `use` block (lines 34-~120), the `mod`/`pub use` registry above. Projected ~250 lines.

WHAT SURVIVES UNTOUCHED (the invariants section e enforces per-boundary):
    - lib.rs:1176 `pub mod task_runner;` — unchanged (still points at the file/dir).
    - lib.rs:1502-1505, lib.rs:2407-2438, lib.rs:2462 — the three `pub use task_runner::{...}` blocks re-export ~80 symbols (start_task_runner, drive-path helpers, TaskRunnerConfig, all the DEFAULT_* / *_CATEGORY consts, derive_*_zero_blocks, runtime_model_pool, ...). Every one stays reachable as `task_runner::Symbol` via the per-submodule glob re-exports, so ALL 151 pub-use blocks in lib.rs and every downstream `elai_orchestrator::Foo` import compile unchanged.
    - 80 contract-test files under crates/contracts/tests reference task_runner symbols; the 526-test contract suite (constraint thread, section e) sees an identical public surface.

--------------------------------------------------------------------------------
(b) ANTI-SSB CLAUSE (blocking on the future executor): NO target file over 1000 lines

The recorded failure is concrete and lives in this same crate: ssb.rs was "split" into ssb/ and ssb/payloads.rs is 20,044 lines RIGHT NOW (409KB action_type.rs, 359KB schema.rs, 425KB tests.rs alongside it). That "split" moved the monolith one directory deeper and satisfied zero maintainability goal. THIS PROPOSAL IS REFUSED AT EXECUTION if any produced target file exceeds 1000 lines. The seam table below is built to that constraint: every standalone target is projected under ~1000 lines by grouping on COHESION (one theme per file), and the two regions that arithmetic alone cannot get under 1000 — drive_task and the config struct — are split by state/trait extraction (drive_task, PHASE-2-RISK) or by a clean type boundary (config vs resolved_config), never by dumping the remainder into one catch-all sub-file. A future letter-plan executor that produces a >1000-line task_runner/*.rs has reproduced the ssb failure and must re-decompose, not ship.

--------------------------------------------------------------------------------
(c) ssb/payloads.rs STRETCH-TARGET NOTE (out of scope here, flagged so it is never forgotten)

ssb/payloads.rs (20,044 lines) is the crate's actual largest file and the same class of defect this proposal fixes for task_runner. It is OUT OF SCOPE for the task_runner split (different module, different consumers) but is the obvious next boundary-proposal target: it holds the SsbPayload variant bodies and their serde impls, which decompose cleanly by payload-family (one file per feature-era batch: e-24/*, b-41/*, d-*/*, h-*/*) behind the existing `pub use payloads::*` glob at ssb.rs:52 — the identical facade-preserving mechanism. Recommend a sibling unit in the same future letter-plan. Do NOT bundle it into the task_runner split changeset (blast radius + review load); a shared letter-plan with two independent phases is the right grain.

--------------------------------------------------------------------------------
(d) lib.rs FLAT 337-DECL REGISTRY: grouping scheme that preserves the pub-use facade

lib.rs is 2489 lines: 337 flat `mod` / `pub mod` declarations and 151 `pub use` blocks, an unstructured god-facade that is itself a scan-cost and merge-conflict driver (though not a compile-unit-size driver the way task_runner.rs is — each `mod` is its own file). The facade is a HARD CONTRACT: cli, contracts (80 files), and dashboard-v2 program against `elai_orchestrator::Foo` paths, so the pub-use lines are load-bearing and CANNOT be reordered into a shape that changes any resolved path.

Proposal (lowest-risk, facade-verbatim):
    1. DO NOT touch any `pub use` line's content or the symbols it names — the facade is frozen. Grouping is presentation-only.
    2. Introduce REGION grouping over the 337 `mod` decls via section banners + alphabetical-within-theme ordering (runner/orchestration, ssb+telemetry, agent-pool+dispatch, config+runtime-setup, grounding+reasoning, web+validation, ipc, util). This is a comment-and-reorder pass; every `mod foo;` stays a top-level item so no path changes.
    3. OPTIONAL stronger form (defer, higher risk): collapse a cohesive cluster behind a facade sub-module (e.g. `pub mod runner { pub use crate::task_runner::*; pub use crate::runtime_setup::*; }`) ONLY if paired with KEEPING the existing flat `pub use task_runner::{...}` lines verbatim as compatibility shims — the facade is the contract, the internal grouping is additive. Reject any variant that requires a consumer to change an import; that is a breaking change masquerading as cleanup.
    Net: lib.rs stays ~2489 lines (grouping does not delete code) but becomes scannable; the real size win is task_runner.rs, not lib.rs. Section (d) is a readability/merge-conflict fix, explicitly NOT a build-cost fix — do not oversell it.

--------------------------------------------------------------------------------
(e) CONSTRAINT THREAD (hard invariants asserted on EVERY boundary in the seam table)

Every proposed boundary must hold all three or it is rejected at execution:
    1. crates/contracts 526-test suite: the public symbol set exported via `task_runner::*` is UNCHANGED (same names, same signatures, same module path). 80 contract files name task_runner symbols; a boundary that renames, re-scopes (pub -> pub(crate)), or drops any re-exported symbol breaks the contract suite and is a BLOCK. The glob re-export per submodule (section a) is what guarantees this — verify post-split by `cargo check -p elai_cc_contracts --tests` (note: real package name is elai_cc_contracts, NOT elai-contracts, per the P3 finding).
    2. elai.map cached FILE:LINE refs: elai.map carries 32,702 lines mentioning task_runner (DEF/CALL/USE rows with FILE:LINE). The MODULE PATH (`task_runner::Foo`) is stable across the split, so SCIP_REF canonical paths survive; but the FILE:LINE of every moved DEF row changes. A boundary is not "done" until elai.map is regenerated (`python .claude/tools/codebase_mapper.py .elai_cc elai_graph.json --compact --map elai.map --scip-dir ...`). Treat elai.map regen as a mandatory post-split step in the executor plan, not an afterthought — stale FILE:LINE refs silently mis-route impact analysis.
    3. facade re-export symmetry: the three lib.rs `pub use task_runner::{...}` blocks (1502-1505, 2407-2438, 2462) enumerate the exact symbols that must stay reachable. A boundary that moves a symbol into a submodule WITHOUT a `pub use <sub>::*` (or explicit re-export) at the task_runner root fails compilation of lib.rs. The walking-skeleton check per boundary: after each submodule extraction, `cargo check -p elai_orchestrator` must stay green before the next extraction (one-move-at-a-time, not a big-bang rewrite).

--------------------------------------------------------------------------------
(f) SEQUENCING SKETCH for a future letter-plan execution (vertical, one-move-at-a-time, green between each)

The split is a mechanical refactor with a large blast radius; sequence it so `cargo check -p elai_orchestrator` (and the contract suite at the end) is green after every step, never a single 24k-line big-bang:
    Step 0  RECON: regenerate elai.map fresh (baseline), snapshot the current `task_runner::*` public symbol list (`cargo public-api` or grep the three lib.rs pub-use blocks) as the frozen contract to diff against at the end.
    Step 1  Create task_runner/ dir; move the LEAF clusters first (no drive_task dependents): constants, cursor, web_launch, runtime_adapters, fixer, web_scaffold, web_research. Add `mod`+`pub use *` per file in task_runner.rs. `cargo check` green after each.
    Step 2  Move config -> config.rs + resolved_config.rs (type-boundary split of the 2622-4024 block). Green.
    Step 3  Move the dispatch/ cluster (verifiers, tripwire, claim_checker, verdict_reward, operationalization, novelty_calibration) into task_runner/dispatch/ with its own mod.rs re-exporting up. Green.
    Step 4  Move the emit/ + derive_evidence + gate_audit + integration_checkpoints clusters. Green.
    Step 5  Move post_execute_hooks, decomposition, runner_loop. Green.
    Step 6  drive_task (THE risk step, do LAST when everything it calls is already relocated): first move the fn verbatim into task_runner/drive_task/mod.rs (one 5960-line file — TEMPORARILY over 1000, explicitly marked WIRE-DARK/MR-1-waiver for this step only), THEN internally decompose it (see PHASE-2-RISK below) into drive_task/{startup_emits, routing, navigate_baby_steps, finalize}.rs each under 1000. Green + contract suite after the internal decomposition.
    Step 7  Split #[cfg(test)] mod tests (lines 21050-24418, ~3368 lines) into task_runner/tests/ per-theme files (each test cluster already maps to the fn it exercises). ssb/tests.rs (425KB, one file) is the ANTI-pattern to avoid here too.
    Step 8  Regenerate elai.map; diff the frozen public-symbol list from Step 0 (must be identical); run `cargo check -p elai_cc_contracts --tests`. Any symbol delta = a facade break to fix before commit.
    Commit grain: one commit per step is defensible for reviewability, but per commit-cadence house rule the whole refactor is ONE unit of work = ONE commit at the end unless the executor plan explicitly authorizes step-commits.

--------------------------------------------------------------------------------
SEAM TABLE (region -> line-range -> proposed target file -> projected lines)

Coverage: the table assigns EVERY line-range of task_runner.rs (1-24,418) to a target, i.e. ~100% region coverage. Projected line counts are line-range arithmetic from the settled structure grep (RECON_NEEDED R6), not a full read of every body; the two projections that exceed 1000 by arithmetic (drive_task, config-struct) carry an explicit decomposition strategy rather than a single oversized target. Anchor line numbers below are verbatim from the structure grep and are the auditor's spot-check set.

  #  | line-range      | target file (under task_runner/)      | theme                                                  | proj. lines
 ----|-----------------|---------------------------------------|--------------------------------------------------------|------------
  R0 | 1-200           | task_runner.rs (root shell)           | module doc + shared `use` + mod/pub-use registry       | ~250
  R1 | 201-380         | constants.rs                          | pub consts (DEFAULT_*, *_CATEGORY, MAX_*), env-flag fns | ~200
     |                 |   + 6101-6115, 16849, 17027           |   (now_unix_ms, plan_approval_gate_enabled, e44_saga,  |
     |                 |                                       |    refusal_sla_tick_enabled, measurement_batch_delay)  |
  R2 | 381-505         | cursor.rs                             | Cursor load/save, TaskRunnerError enum, CancelHandle   | ~150
  R3 | 506-672         | web_launch.rs                         | LaunchedPreview, WebValidatorLauncher, DevServer*,     | ~400
     |   +16370-16460  |                                       |   ProjectDirResolver, project_dir_is_web, preview emit |
  R4 | 673-795         | runtime_adapters.rs                   | RuntimeToolLookup/Predicates/ModelPool, RuleEvaluator  | ~250
  R5 | 796-935,        | baby_step_executor.rs                 | BabyStepExecutionCtx/Result, Synthetic + AgentPool     | ~950  (NEAR)
     |  1304-2066      |                                       |   executors + impls, SpiralTracker                     |
  R6 | 935-1304        | fixer.rs                              | lighthouse report, FixerValidator, fix_and_rerun_...   | ~370
  R7 | 2066-2450,      | web_research.rs                       | is_web_research_task, WebResearch*, drive_web_research | ~500
     |  5257-5373      |                                       |                                                        |
  R8 | 2450-2622       | web_scaffold.rs                       | is_web_scaffold_task, detect_scaffold_template,        | ~170
     |                 |                                       |   ScaffoldApplier, drive_web_scaffold_task             |
  R9 | 2622-3663       | config.rs + resolved_config.rs        | TaskRunnerConfig struct+builder | ResolvedConfig+From   | ~700 + ~700
 R10 | 4024-4908       | post_execute_hooks.rs                 | crystallizer/skill/experience/cost-router/focus-tunnel | ~880
 R11 | 4908-5468       | runner_loop.rs                        | start/run_task_runner, replay_backlog, handle_task_... | ~450
     |  (excl 5257)    |                                       |   emit_fsm_transition, emit_dag_checkpoint             |
 R12 | 5468-6101       | decomposition.rs                      | strip/serialize decomposition, parallel-phases, dkg    | ~630
 R13 | 6116-12078      | drive_task/ (dir)                     | THE core per-task orchestration loop                   | ~5960  **PHASE-2-RISK**
     |                 |   startup_emits.rs / routing.rs /     |   -> state-struct extraction into 4-6 sub-files,       |   (each <1000
     |                 |   navigate_baby_steps.rs / finalize.rs|      each under 1000; see PHASE-2-RISK note            |    projected)
 R14 | 12078-12673     | dispatch/verifiers.rs + tripwire.rs   | verifier adapters, tripwire/bai monitors + dispatch    | ~130 + ~600
 R15 | 12673-13072     | dispatch/claim_checker.rs             | claim routing + checker result/gap emitters, runtime   | ~380
 R16 | 13072-13803     | dispatch/verdict_reward.rs            | tri-layer verdict, progress reward, search ladder,     | ~730
     |                 |                                       |   escalation waterfall, sprint contract                |
 R17 | 13803-14509     | dispatch/operationalization.rs        | constructive checkers, operationalization runtime,     | ~700
     |                 |                                       |   adversarial pass, sycophancy probe                   |
 R18 | 14509-15497     | dispatch/novelty_calibration.rs       | novelty, calibration emitters, conformal corpus/refusal| ~980  (NEAR)
     |                 |                                       |   claim-confidence, observable-effect                  |
 R19 | 15497-15843     | emit/csq_reasoning.rs                 | CsqSeverityLabel, csq/cosmos/reasoning emit            | ~350
 R20 | 15843-16331     | emit/checkpoint_recovery.rs           | baby-step checkpoint/recovery, per-level audit hook,   | ~490
     |  (excl 16370)   |                                       |   spiral hard-stop                                     |
 R21 | 16331-17027     | emit/tool_dispatch.rs                 | test-outcome/tool/validation/commit emits, validate/   | ~930  (NEAR)
     |                 |                                       |   file-read/classify dispatch, e44 saga                |
 R22 | 17027-17358     | emit/findings_canvas.rs               | emit_finding(+audit_step), canvas activity, refusal-   | ~330
     |                 |                                       |   sla tick, skill-staleness loop, git-commit dispatch  |
 R23 | 17358-17774     | derive_evidence.rs                    | derive_autonomous/*_zero_blocks/replan/security/       | ~420
     |                 |                                       |   layer3/layer4/framing, persist_wiring_coverage       |
 R24 | 17774-18490     | emit/snapshots_startup.rs             | autonomous-decision, provider/search/vram/system probe | ~720
     |                 |                                       |   warm-start calibration, held-out sample              |
 R25 | 18490-18815     | emit/project_class.rs                 | resolve_project_class, framework/empire/checklist once | ~330
     |                 |                                       |   vram_tier_label, feasibility_label, margin_bp, blake3|
 R26 | 18815-19606     | emit/feasibility_mistakes.rs          | feasibility probes, memory-note/mistake-book, confirm  | ~790
     |                 |                                       |   gate, repair telescope, interrogation metric         |
 R27 | 19606-20372     | gate_audit.rs                         | classify_failure, BlockSource, interrogation fix,      | ~770
     |                 |                                       |   pre/post-gate audit, impact summary, project context |
 R28 | 20372-21050     | integration_checkpoints.rs            | navigator specs, level transitions, checkpoint flush,  | ~680
     |                 |                                       |   file-read budget guard, bubble-up, small helpers     |
 R29 | 21050-24418     | tests/ (dir, per-theme files)         | #[cfg(test)] mod tests                                  | ~3368  (split
     |                 |                                       |   -> per-fn-cluster test files, each <1000             |    like R13)

Max projected STANDALONE target (excluding the two decomposed regions R13/R29): ~980 lines (R18 dispatch/novelty_calibration), i.e. under the 1000 MR-1 boundary. R9 config is kept under 1000 by the config/resolved_config type-boundary split. R13 (drive_task) and R29 (tests) are the only regions whose single-file form exceeds 1000; both carry a decomposition strategy that lands every produced file under 1000 (drive_task via state-struct extraction, tests via per-cluster files), so the whole-file MR-1 target is achievable — with drive_task as the one genuine engineering risk, not a mechanical move.

--------------------------------------------------------------------------------
PHASE-2-RISK (the one seam that does not decompose by file-move alone)

drive_task (async fn, lines 6116-12078, ~5960 lines = 24% of the file) is a single sequential pipeline: a run of one-shot startup emits (emit_vram_tier_once, emit_system_probe_once, maybe_warm_start_calibration_once, maybe_sample_held_out_once, maybe_emit_framework_profile_once, maybe_emit_empire_connected_once, maybe_run_production_checklist_once, maybe_build_project_index — all read at 6127-6190), then scaffold/research routing branches (6197-6206), then project-dir resolution and the baby-step navigation loop with its ~30 dispatch_*/emit_* hooks inline. It CANNOT reach <1000 lines by moving it to a file — it IS one function. The mandated decomposition (ssb-failure-avoidance fallback): extract a `DriveTaskState` struct owning the shared refs (`writer: &Arc<Ssb>`, `seq`, `task`, `resolved`, `cancel`) plus the mutable per-task flags (e.g. `preview_launched_for_task`), and break the body into stage methods across drive_task/{startup_emits, routing, navigate_baby_steps, finalize}.rs, each under 1000. This is trait/state extraction, NOT a file-move, and its target sizes are PROJECTED (the 5960-line body was not read in full this session — only the 6116-6236 head) — so a future executor MUST read the full body, confirm the stage boundaries have no shared-mutable-state entanglement that blocks the split, and treat any stage that stays >1000 as a further-decomposition item, never a shipped oversized file. This is the single highest-effort, highest-risk step of the whole split and is sequenced LAST (Step 6) so every callee it dispatches is already relocated first.

Secondary (lower-risk) note: R29 tests (~3368 lines) must not become one task_runner/tests.rs — the ssb/tests.rs 425KB single file is that exact anti-pattern in-crate. Split per test-cluster (the test module already groups by the fn under test).
