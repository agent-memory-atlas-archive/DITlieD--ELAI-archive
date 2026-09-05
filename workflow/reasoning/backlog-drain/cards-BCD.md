# Backlog-drain state + supersession map — letters B, C, D

Analyst scope: B (43 units), C (59 units), D (30 units) = 132 units.
Grounding basis: full read of B-PLAN-SUMMARY, C-SUMMARY_INDEX, D-SUMMARY_INDEX, ALL-PLANS-MASTER-SUMMARY; headline-symbol greps in `.claude/tools/elai.map` (DEF + production-caller presence).
Generated: 2026-07-16.

CRITICAL FRAMING (read before draining): the per-letter summaries are STALE. C-SUMMARY marks all C-1..C-53 "PLANNED" and D-SUMMARY marks all D-1..D-27 "PLANNED", but the codebase (git log through K/L, 2026-07) shows the overwhelming majority of C and D are SHIPPED + WIRED with production callers and `cNN_*` / `dNN_*` contract tests. The map confirms it: `crates/orchestrator/src/{grounding_mode,novelty,adversary,ppm_client,held_out,bai,mutation_testing,intent_pair,metamorphic_testing,skeleton_generator,mini_critic,compiled_pipeline,checker_registry,observable_effect,contract_coding}.rs`, `crates/plugins/src/{authz,mcp/gate,tool_calibration/case_base}.rs`, `crates/memory/` (whole crate), `crates/agent/src/dropper.rs`, `crates/providers/src/{decoding,runtime/composer2}.rs`, `crates/cli/src/{archive_verify,github_pr_cmd,mcp_authz_cmd,routing_cmd,experience_cmd}.rs` all exist and are called from non-test entrypoints. Treat "state" fields below as the ground truth over the summaries. The residual work is mostly RunPod-gated live benchmark runs, a few genuinely-dark producers, and dashboard surfaces.

STATE LEGEND: wired = producer+consumer present, called from non-test production entrypoint. partial = core shipped, named deferred/gated pieces remain. dark = headline symbols absent or only test callers. uncertain = cheap grep could not settle (settling check named).

=================================================================
SUPERSESSION_EDGES (newer supersedes/closes/extends older)
=================================================================

Two edge kinds. CLOSES = the newer unit implements/completes what the older only specified (respect the newer impl when finishing the old). SUPERSEDES/REPLACES = the newer unit redesigns the older's approach (do NOT re-ship the old approach). EXTENDS = additive; old stays valid, new layers on.

--- Within-scope: C CLOSES B (source: C-SUMMARY_INDEX §4 "B-N -> Closed by", grounded — this is the authoritative closure map) ---
- C-18 CLOSES B-1 (contract test), C-8 CLOSES B-1 (pagerank/AragOrchestrator runtime)
- C-18 CLOSES B-2 (fsm_phase_transition_contract)
- C-3,C-8,C-23 CLOSE B-3 (stub eradication)
- C-1,C-3,C-8 CLOSE B-4 (knowledge producer)
- C-1,C-3,C-12 CLOSE B-5 (reasoning producer)
- C-2,C-4 CLOSE B-6 (orchestration producer)
- C-2,C-4,C-8 CLOSE B-7 (pipeline factory producer)
- C-1,C-3 CLOSE B-8 (PipelineGrc)
- C-23 CLOSES B-9 (SessionDag)
- C-4 CLOSES B-10 (ContextBudget)
- C-6,C-7 CLOSE B-11,B-12; C-2,C-7 CLOSE B-13; C-7 CLOSES B-14 (dashboard)
- C-14,C-15 CLOSE B-15 (phase-0 debt)
- C-21 CLOSES B-16 (anti-stale; also E-34 amends the ort rule B-16 enforced)
- C-2,C-8 CLOSE B-17 (vacuous gates); B-38 EXTENDS B-17 (continuous ambiguity score)
- C-11 CLOSES B-18 (Tier-C sandbox)
- C-13 CLOSES B-19,B-20 (IPC hook + tool registry swap)
- C-22 CLOSES B-21 (feasibility probe)
- C-9,C-10 CLOSE B-22 (autonomous mode + anti-spiral)
- C-21 CLOSES B-23 (self-mod automation, partial — see cross-letter DGM edges)
- C-7,C-8 CLOSE B-24 (media validator surfacing)
- C-34 CLOSES B-25 (Empire — both DEFERRED, external block)
- C-4,C-7 CLOSE B-26 (dashboard completion)
- C-5 CLOSES B-27 (cloud mode)
- C-4 CLOSES B-28 (focus tunnel surfacing)
- C-24 CLOSES B-29 (symbol impact)
- C-17 CLOSES B-30,B-31,B-34 (multi-lang DKG, resumable indexing, entry-points)
- C-8,C-18 CLOSE B-35 (context artifacts)
- C-4,C-18 CLOSE B-36; C-4 CLOSES B-37; C-2,C-8 CLOSE B-38
- C-33 CLOSES B-39; C-12,C-33 CLOSE B-40; C-12 CLOSES B-41; C-2,C-16 CLOSE B-42

--- Within-scope: D EXTENDS/REPLACES C (source: D-SUMMARY_INDEX "Cross-plan link (C -> D)", grounded) ---
- D-1 EXTENDS C-21 (typed reasoning-template archive over self-mod automation)
- D-4 REPLACES C-27 single-signal DeterminismDrift with 5 orthogonal tripwires — "replaces C-27 single-signal detection"
- D-9 EXTENDS C-27 StubBanAuditor with frozen-surface access-control predicates
- D-11 consumes/EXTENDS C-24 symbol_impact + C-30 surgical DKG tooling
- D-12 EXTENDS C-31 FILE_PLAN (adds block-9 MEASURABLE_PROCEDURE)
- D-13 EXTENDS C-12 per-level auditor REVIEW with cross-family Adversary
- D-18 EXTENDS C-22 FeasibilityProber with typed ObservableEffect schema
- D-26,D-27 EXTEND C-7 dashboard-UX with D-plan drawers (MutationArchive, ModeRouting)
- D-5 REPLACES the MSI-sigmoid promotion of B-23/C-21 self-mod with Beta-posterior Thompson BAI ("replaces MSI sigmoid Boltzmann")

--- Cross-letter: E/F/G/H/I/J supersede or absorb B/C/D (source: ALL-PLANS-MASTER-SUMMARY letter digests + auto-memory; each edge carries the phrase it was read from). These are the edges a drainer MUST respect so an old B/C/D unit does not revert newer shipped work. ---
- E-9 SUPERSEDES C-54 (and reframes C-55..C-59): "drop SWE-bench Verified, adopt SWE-Live+Pro+tau2". C-54 SWE-bench-Verified flagship is the canonical verify-then-skip candidate (it is also DARK — never built). Also E-49 (STABLE-Pass@1 + $/CTS metrics) reframes the benchmark reporting of C-54..C-59.
- E-17 SUPERSEDES the B-4 RAG activation approach: "THE BIG ONE — stage ONNX models, activate code_vec, Contextual Retrieval + Jina-rerank, fixes DEEP<STANDARD MRR". A drainer finishing B-4/C-1 knowledge work must use the E-17 retrieval design, not B-4's original RRF-only.
- E-27 SUPERSEDES C-53 NF4 path: "NF4->AWQ+Marlin 4.4x + FP8 KV". C-53 is a passive upstream-watch; E-27 is the actual quant upgrade.
- E-32 SUPERSEDES the B-27/D-2(B-plan cloud) context-window sizing: "25K->32K native Qwen".
- E-34 AMENDS/REPLACES the B-16 ort rule: "amends CLAUDE.md ort rule — was wrong" (rc-track pin, not stable-only). B-16 as originally written is wrong; respect E-34.
- E-39 SUPERSEDES C-5 + B-27 cloud spend ceiling: "HARD CEILING: blocks not warns, closes $47K wallet-loop". C-5's warn_at_percent is softened; E-39 blocks.
- E-5 ABSORBS C-48 SafeHarbor: the C-48 TrustTier (T0/T1/T2/T3) now lives in `crates/memory/src/trust.rs` (the E-5 Mem0 memory crate), wired across the whole memory crate. C-48 as a standalone "SafeHarbor" unit is effectively shipped-inside-E-5; do not re-author it in crates/plugins.
- E-45 SUPERSEDES the ad-hoc SCHEMA_VERSION bump discipline used across ALL B/C/D units: "ts-rs snapshot + action_type golden lock". Any B/C/D unit that bumps SCHEMA must now go through the E-45 action_type golden lock (the git log shows `action_type_lock` goldens actively maintained).
- F-4 SUPERSEDES/HARDENS C-27+C-28 test-integrity: "add stdout_hash to TestOutcomePayload; auditor cannot emit passed=true without matching hash". The C-27/C-28 wiring/mutation gates are subsumed by F's enforced done-gate (`wiring_contract_check --run-trigger`, not cargo-green).
- E-52 EXTENDS C-27/C-28 auditor with LSP-as-verifier (rust-analyzer headless pre-AUDITING).
- H-43..H-48 SUPERSEDE the B-23 DGM placeholder filters: "replaces derived_alpha=blake3(bytes) with CodeRM pass-rate over frozen tests" + behavioral trap detection. B-23's L2 config-eval and DarwinNet fitness are the exact two filters H-Track7 replaces; do NOT ship B-23's original scoring.
- D-5 + H-48 wire the BAI bandit onto the real reward, superseding B-23/C-21 MSI.
- H-49 (reversibility) delivers B-9 SessionDag's deferred undo/branch items via CAS time-travel; respect H-49's undo-tree over re-implementing B-9 branch merge.
- I-51 SUPERSEDES the model-metadata scatter touched by B-27/C-5/C-44 (Composer2 pricing): "unified model capability catalog — one models.toml over pricing.rs/vram_tier.rs scatter". J-9 hardware catalog is the eligibility layer above it.
- I-39 SUPERSEDES hand-maintained plan status itself: "`elai plan status` derives from journal — kills hand-maintained drift". This is meta-relevant to the whole drain: prefer journal-derived status over the stale summaries.
- G-2 RRA + G-6 SAFEDELTA reframe the "prove it with a cloud-Sonnet ACCEPTANCE test" smoke that every C unit specifies — the acceptance proof is now the reproducible-action-stream (RRA), not raw tokens (GD-5).

VERIFY-THEN-SKIP CANDIDATES (dark/deferred old unit already superseded by shipped-or-canonical newer work): C-54 (dark; E-9 drops SWE-bench Verified), C-51 (dark; polyglot harness — RunPod-gated, E-9/E-49 reframe), C-53 (passive watch; E-27 is the real quant path), B-25/C-34 (Empire; external block, no consumer), B-16 (superseded by E-34), B-23 original DGM scoring (superseded by D-5/H-43..48).

=================================================================
B-PLAN CARDS (B-1 .. B-42)
=================================================================

## B-1 path unification (.elai -> .elai_cc)
does: promote ELAI_DATA_DIR_NAME=".elai_cc" + path helpers to core/src/paths.rs; unblocks PageRank/CatRAG/A-RAG at runtime.
state: wired (SHIPPED s-59964, commit-verified; paths.rs is the map's path-constant source of truth).
supersedes[]: — | superseded_by[]: —
residual: none.
key_files: crates/core/src/paths.rs
respect[]: paths.rs is now depended on by every crate; never reintroduce ".elai/" literals.

## B-2 FSM phase_transition wiring
does: emit "phase_transition" SSB inside Fsm::transition_with_metrics; activates StatusBar badge + orbit halo.
state: wired (SHIPPED s-59964; C-18 added fsm_phase_transition_contract).
supersedes[]: — | superseded_by[]: CLOSED by C-18 (test)
residual: none.
key_files: crates/orchestrator/src/fsm.rs, task_runner.rs
respect[]: single consolidated "phase_transition" action_type (do not re-split emit_fsm_transition).

## B-3 stub command eradication
does: classify 30 dead UI elements IMPLEMENT/DELETE/RENAME; +ts-rs.
state: wired (SHIPPED s-59964; CLASS-A live-wire done by B-36/C-3/C-8/C-23).
supersedes[]: — | superseded_by[]: CLOSED by C-3,C-8,C-23
residual: none material.
key_files: crates/dashboard-v2/src/lib/api/stub.ts, commands.rs
respect[]: dashboard-v2 is the live surface (V1 dashboard parked in .bkp_old).

## B-4 producer wiring — knowledge
does: stage 3 ONNX models, SearchMode cascade, code_vec RRF, QueryTrace, circuit breakers; fix DEEP<STANDARD MRR.
state: partial->wired (SHIPPED s-59964; code_vec was ELAI_CODE_VEC=1 opt-in). Retrieval design later superseded.
supersedes[]: — | superseded_by[]: SUPERSEDED by E-17 (Contextual Retrieval + Jina-rerank, the canonical RAG activation); CLOSED by C-1,C-3,C-8
residual: adopt E-17 retrieval stack; confirm code_vec default-on (remove opt-in). settle: grep ELAI_CODE_VEC in elai.map.
key_files: crates/knowledge/src/{pipeline,reranker,vector,bm25}.rs
respect[]: E-17 RAG design and C-1 SSB emission; do not revert to BM25-only RRF.

## B-5 producer wiring — reasoning
does: replace synthetic emit_reasoning with real PipelineGrc values; 4-5 GRC SSB types.
state: wired (SHIPPED s-59964; per-agent budget was a TODO, closed by C-1/C-12).
supersedes[]: — | superseded_by[]: CLOSED by C-1,C-3,C-12
residual: none material.
key_files: crates/orchestrator/src/task_runner.rs, reasoning/engine.rs
respect[]: C-1 GrcStepEmitted/GroundingSweepResult emission.

## B-6 producer wiring — orchestration
does: EvidenceGate panel, AgentsPanel, SupervisionManager wiring, 9 SSB types, SCHEMA 25->26.
state: wired (SHIPPED s-59964).
supersedes[]: — | superseded_by[]: CLOSED by C-2,C-4
residual: none.
key_files: crates/orchestrator/src/{evidence,task_runner,agent_pool}.rs
respect[]: E-45 action_type golden lock for any further SSB additions.

## B-7 producer wiring — pipeline factory (4D+4E)
does: DecompositionTree producer, DAG repair events, research-tree per-stage events, 14 SSB types.
state: wired (SHIPPED via B-7b WebResearchStepEmitter).
supersedes[]: — | superseded_by[]: CLOSED by C-2,C-4,C-8
residual: none material.
key_files: crates/orchestrator/src/{web_research_orchestrator,task_runner}.rs
respect[]: —

## B-8 PipelineGrc activation in hot paths
does: wire ground_plan/verify_claim/generate_csq/check_inline_feasibility into FSM transitions.
state: wired (SHIPPED s-59964 via with_pipeline_grc()).
supersedes[]: — | superseded_by[]: CLOSED by C-1,C-3
residual: none.
key_files: crates/orchestrator/src/task_runner.rs, reasoning/engine.rs
respect[]: —

## B-9 SessionDag activation
does: stub_ssb_seq population, create_handoff at 85%, /continue load_handoff, branch UI, msgpack BLOBs.
state: wired (SHIPPED s-59964; dag-write-in-dashboard deferred, closed by C-23).
supersedes[]: — | superseded_by[]: CLOSED by C-23; undo/branch time-travel EXTENDED/superseded by H-49 (CAS reversibility)
residual: dashboard dag-write (C-23) — verify shipped. Prefer H-49 undo-tree for new reversibility work.
key_files: crates/orchestrator/src/session_dag.rs
respect[]: H-49 CAS store + undo-tree; do not re-implement branch merge from scratch.

## B-10 ContextBudget profile + file_overlap_score
does: stop passing profile=None at 5 sites; detect model tier; compute_overlap SSB filter.
state: wired (SHIPPED s-59964).
supersedes[]: — | superseded_by[]: CLOSED by C-4; context policy later refined by H-50 (regime-aware) + E-32 (32K window)
residual: adopt H-50 regime-aware ctx for local-vs-cloud opposite policies.
key_files: crates/orchestrator/src/task_runner.rs, context_budget.rs
respect[]: E-32 32K ceiling; H-50 window-minimization for local lane.

## B-11 design system restoration
does: verify V2 tokens vs dashboard-design; token/type/animation audit.
state: wired (largely DONE in V2; C-6 aligned tokens.css).
supersedes[]: — | superseded_by[]: CLOSED by C-6,C-7
residual: none material (a full UI rework is queued post-backend per auto-memory, out of B scope).
key_files: crates/dashboard-v2/src/lib/tokens.css
respect[]: C-6 token values (teal #1B7A6E on #0f1318).

## B-12 shell vs workspace decision
does: Workspace.svelte as canonical shell; Tier-0 Shell retired.
state: wired (DONE in V2).
supersedes[]: — | superseded_by[]: CLOSED by C-6,C-7
residual: none.
key_files: crates/dashboard-v2/src/lib/screens/Workspace.svelte
respect[]: —

## B-13 activity feed restoration (orbit)
does: ActivityWindow IS the 3-mode orbit visualizer (PLAN/MAP/RESEARCH); text feed retired.
state: wired (DONE s-59964 re-scoped).
supersedes[]: REPLACES the V1 text ActivityFeed | superseded_by[]: CLOSED by C-2,C-7; orbit itself slated for retirement by H-16 (canvas replaces orbit)
residual: H-Track2 canvas-workbench is designed to REPLACE the orbit (H-16 retires it). Do not invest new orbit features.
key_files: crates/dashboard-v2/src/lib/orbit/*, windows/ActivityWindow.svelte
respect[]: H-10..H-16 canvas direction; the orbit is end-of-life.

## B-14 live token streaming chat
does: Tauri stream_chat SSE, intent classifier, card variants, Stop/cancel.
state: wired (SHIPPED s-59964).
supersedes[]: — | superseded_by[]: CLOSED by C-7
residual: none.
key_files: crates/dashboard-v2/src-tauri/src/lib.rs, src/lib/windows/CommandWindow.svelte
respect[]: —

## B-15 phase-0 debt burndown
does: main.rs 10570->252 LOC decomp; delete compact/oauth/prompt.rs; HashMap ban; ClockProvider injection; entropy quarantine.
state: wired (main.rs decomp DONE 2026-05-08; the rest CLOSED by C-14 dead-code strip + C-15 ClockProvider/FxHashMap).
supersedes[]: — | superseded_by[]: CLOSED by C-14,C-15
residual: none material (C-14/C-15 shipped the deletes + entropy injection).
key_files: crates/cli/src/*, crates/core/src/clock.rs
respect[]: C-15 ClockProvider/entropy-allow convention; FxHashMap over HashMap.

## B-16 anti-stale rule enforcement
does: ort rc-track pin, Box<dyn Error>->thiserror, .unwrap removal, log->tracing, magic-number ban, CI lint.
state: partial (CLOSED by C-21 anti-stale automation; ort rule AMENDED by E-34).
supersedes[]: — | superseded_by[]: SUPERSEDED by E-34 (ort rule "was wrong" — rc-track not stable-only); CLOSED by C-21
residual: verify CI anti-stale lint active (check_anti_stale.sh in windows.yml per C-21).
key_files: crates/*/Cargo.toml (ort pin), .github/workflows/windows.yml
respect[]: E-34 ort=2.0.0-rc.x rule; NEVER enforce "stable-only".

## B-17 ground the vacuous gates
does: per-gate liveness regression suite + GatesPanel; meta-step over B-2/4/6/8.
state: wired (CLOSED by C-2/C-8 SSB emission making gates live); B-38 adds continuous score.
supersedes[]: — | superseded_by[]: CLOSED by C-2,C-8; EXTENDED by B-38
residual: confirm tests/contracts/gate_liveness.rs exists. settle: grep gate_liveness in elai.map.
key_files: crates/contracts/tests/*, dashboard GatesPanel
respect[]: —

## B-18 Tier-C sandbox activation
does: Linux clone3+seccomp allowlist+rlimits+hidepid; Windows AppContainer; IPC accept loop; real /proc NSpid tests.
state: partial (CLOSED by C-11 sandbox-completeness; SandboxConfig present in core/config.rs). Windows AppContainer + full seccomp likely partial.
supersedes[]: — | superseded_by[]: CLOSED by C-11 (SAFETY-CRITICAL)
residual: verify Windows AppContainer path + real NSpid test (C-11 deliverables); confirm no FAKE isolation tests remain.
key_files: crates/orchestrator/src/sandbox.rs, ipc_accept.rs, crates/core/src/config.rs
respect[]: C-11 rlimit/seccomp ordering; do not weaken isolation. G-3 runtime blast-radius gates layer on top.

## B-19 hook system unification
does: keep IpcHookDispatcher, delete InProcessHookDispatcher; migrate hook sites to IPC; DAIRA 2->15 markers; ruff PostToolUse.
state: wired (CLOSED by C-13; AdaptiveTool emits 10 DAIRA markers; IpcHookDispatcher wired).
supersedes[]: REPLACES InProcessHookDispatcher | superseded_by[]: CLOSED by C-13
residual: none material.
key_files: crates/tools/src/adaptive_tool.rs, orchestrator IPC hook path
respect[]: no InProcessHookDispatcher in production (C-13 hard rule).

## B-20 tool registry unification
does: keep IpcToolRegistry, delete InProcessToolDispatcher; FSM filtering to IPC; failure taxonomy 13->9; WIP squash.
state: wired (CLOSED by C-13; IpcToolRegistry in crates/tools/src/ipc_registry.rs, wired via adaptive_tool.rs).
supersedes[]: REPLACES InProcessToolDispatcher | superseded_by[]: CLOSED by C-13
residual: none material.
key_files: crates/tools/src/ipc_registry.rs, adaptive_tool.rs
respect[]: no InProcessToolDispatcher in production (C-13 hard rule).

## B-21 phase-4F feasibility probe
does: SystemProbe (GPU/RAM/disk), novel-step detection, Tier-C stub harness, FeasibilityReport, skip-iff-zero-novel.
state: wired (CLOSED by C-22; SystemProbeResult + DefaultFeasibilityProber wired in cli/main.rs).
supersedes[]: — | superseded_by[]: CLOSED by C-22; EXTENDED by D-18 (typed ObservableEffect)
residual: D-18 typed ObservableEffect layered on feasibility (shipped — observable_effect.rs).
key_files: crates/orchestrator/src/feasibility_probe.rs, observable_effect.rs
respect[]: C-22 name reconciliation (system_probe/feasibility_report/novel_step_detected/stub_run); D-18 effect schema.

## B-22 phase-4G autonomous mode
does: AutonomousGate wraps InteractiveGate, 6 evidence checks, 4 escalation triggers, SPLIT_TASK, SUBSTITUTE_MODEL, /mode + /model.
state: wired (SHIPPED s-26716 + patch-4G.1; B-22b ouroboros anti-spiral shipped via C-10).
supersedes[]: — | superseded_by[]: CLOSED by C-9,C-10
residual: open follow-ups (/mode CLI live-wiring, orbit per-decision viz) — mostly cosmetic.
key_files: crates/orchestrator/src/{autonomous_gate,task_runner}.rs, anti_spiral_extended.rs
respect[]: C-9 AutoAdvanceEvaluator + C-10 StagnationDetector/GoalDriftCanary (both wired into agent_pool + task_runner).

## B-23 phase-4H self-modification (DGM/CEL/DarwinNet)
does: L1 data-tables, L2 config proposals (DGM sigmoid parent-select, staged eval 10/50/200), L3 human-only; MSI decay; auto-revert.
state: partial/dark-gated (C-21 shipped L1A automation + ParameterKey enum; the DGM loop ships INERT behind ELAI_DARWINIAN_LOOP_ENABLE per F-11). L2/L3 scoring superseded.
supersedes[]: — | superseded_by[]: L2 config-eval + fitness SUPERSEDED by H-43..H-48 (execution-grounded CodeRM fitness replaces derived_alpha=blake3(bytes)); MSI sigmoid REPLACED by D-5 BAI Thompson; CLOSED(partial) by C-21
residual: the Darwinian loop is wire-dark by design (F-11 closes ONE config-only generation behind the kill-switch). Do NOT ship B-23's original blake3-bytes fitness or MSI sigmoid — use H-43/H-44 + D-5.
key_files: crates/orchestrator/src/{bai,self_mod_bridge}.rs, rl/*, cli/darwin_loop_cmd.rs
respect[]: ELAI_DARWINIAN_LOOP_ENABLE kill-switch; H-Track7 fitness/trap filters; D-5 BAI promotion; AllowedConfigKey->ParameterKey closed enum (C-21).

## B-24 phase-4I media validator
does: Tier1 deterministic (ImageMagick/FFmpeg/Tesseract/Blender/ONNX SSIM), Tier2 VLM advisory, veto logic, RCE loop.
state: wired (SHIPPED s-59964; 71 lib + 21 RCE + 19 dashboard tests).
supersedes[]: — | superseded_by[]: CLOSED by C-7,C-8; reused by J-1 perception gateway
residual: none material.
key_files: crates/orchestrator/src/media_validator*, dashboard MediaValidationPanel
respect[]: J-1 /see perception gateway reuses media_validator; keep Tier1-veto semantics.

## B-25 phase-4J Empire integration
does: FactoryLine protocol client, Brain registration, ElaiNeuron MCP, repair pipeline.
state: dark (DEFERRED — external Empire codebase access blocker).
supersedes[]: — | superseded_by[]: CLOSED-as-DEFERRED by C-34
residual: blocked on external Empire access; no production consumer. VERIFY-THEN-SKIP for the drain.
key_files: (none shipped) crates/empire/ planned
respect[]: do not build until elai_cc.toml [empire.codebase_path] confirmed.

## B-26 dashboard completion (4K.6-4K.13)
does: MonitorWindow, ExploreWindow, ReviewWindow, notifications, webhook firing, keyboard shortcuts.
state: wired (SHIPPED s-59964 majority; 6 polish tasks — webhook POST, responsive breakpoints).
supersedes[]: — | superseded_by[]: CLOSED by C-4,C-7
residual: 6 polish tasks (webhook firing, <1000px hamburger, phase-dot real progression). Cosmetic.
key_files: crates/dashboard-v2/src/lib/windows/*
respect[]: D1 decision (localStorage-only window state, no WindowState SSB).

## B-27 cloud mode (D-1..D-4)
does: LOCAL/CLOUD/HYBRID per-task modes, cloud config + spend ceiling, privacy redaction, BYOE.
state: wired (CLOSED by C-5; RouteBlocked + estimate_task_cost_usd shipped). Ceiling semantics superseded.
supersedes[]: — | superseded_by[]: SUPERSEDED by E-39 (hard ceiling BLOCKS not warns); CLOSED by C-5; context sizing by E-32
residual: adopt E-39 blocking ceiling over C-5 warn_at_percent; G-9 write-time PII redaction over B-27 redaction.
key_files: crates/orchestrator/src/cloud*.rs, crates/cli cloud path
respect[]: E-39 hard-ceiling (closes $47K wallet-loop); I-51 model catalog for pricing.

## B-28 focus tunnel (FIM/DIFF/STRUCTURED)
does: deterministic mode selection, streaming validation, pipeline parallelism, auto dependency resolution.
state: partial (focus_tunnel module exists per C-15 entropy-injection list; extent of FIM/DIFF wiring uncertain).
supersedes[]: — | superseded_by[]: CLOSED(surface) by C-4; output-token stack SUPERSEDED/EXTENDED by H-3 (DIFF/FIM emission + streaming early-abort)
residual: settle FIM/DIFF activation depth. settle: grep "focus_tunnel" + "supports_fim" in elai.map. Prefer H-3 output-token stack.
key_files: crates/orchestrator/src/focus_tunnel/*
respect[]: H-3 DIFF/FIM emission design.

## B-29 symbol-level impact API
does: 4 agent tools (impact_radius/call_flow/symbol_context/list) over sharded SymbolGraphCache; schema v3.
state: wired (CLOSED by C-24; symbol_impact tools in build_tool_catalog + incremental symbol-graph update).
supersedes[]: — | superseded_by[]: CLOSED by C-24; consumed by D-11 checker registry
residual: none material.
key_files: crates/knowledge/src/symbol_graph*, dispatcher symbol_* arms
respect[]: C-24 dispatcher tool entries; codegraph-elai MCP is the live twin.

## B-29.5 symbol graph scale benchmark
does: benchmarks vs Linux/Chromium/VSCode/Postgres fixtures; p99<=500ms, RSS<=2GB.
state: partial/uncertain (metrics TOML backfilled by C-20 phase_b29_5_scale.toml with tier runs; live multi-hour indexing may be incomplete).
supersedes[]: — | superseded_by[]: metrics CLOSED by C-20
residual: verify full 4-fixture scale run wall-clock completed. settle: read .elai_cc/metrics/phase_b29_5_scale.toml for non-placeholder values.
key_files: crates/knowledge/benches/*, metrics/phase_b29_5_scale.toml
respect[]: —

## B-30 multi-language DKG hardening (closes A-35)
does: 13+ tree-sitter grammars with per-grammar isolation, multi-module resolvers, Svelte/Vue composite re-parse.
state: wired (CLOSED by C-17; KotlinGrammar impl LanguageGrammar confirmed; ext_to_lang_id registration).
supersedes[]: closes A-35 | superseded_by[]: CLOSED by C-17
residual: verify full 13-grammar coverage (Scala/PHP/Swift). settle: grep "impl LanguageGrammar" in elai.map.
key_files: crates/knowledge/src/dkg/mod.rs (grammars), language extractors
respect[]: tree-sitter retained (no ast-grep swap, per CLAUDE.md + I-54).

## B-31 resumable indexing + cross-process lock
does: per-batch checkpoint (blake3 hashes), IndexLock, auto_resume_indexed_projects, graceful_shutdown.
state: wired (CLOSED by C-17; auto_resume + FileWatcherService + signal handlers wired into cli/main.rs).
supersedes[]: — | superseded_by[]: CLOSED by C-17
residual: none material.
key_files: crates/knowledge/src/{indexing_service,watcher_service}.rs
respect[]: C-17 LockScope::Index/Watch; B-42 cosmos shares LockScope::Index.

## B-31.5 resumable indexing scale benchmark
does: 5-tier kill-resume identity + cross-process race + graceful shutdown under load.
state: partial/uncertain (C-17 populated phase_b31_5_tier_*.toml with Linux v6.7 + VSCode runs; full 5-tier may be incomplete).
supersedes[]: — | superseded_by[]: metrics CLOSED by C-17,C-20
residual: verify Tier B/C/D identity + race tests ran. settle: read metrics/phase_b31_5_tier_*.toml.
key_files: scripts/scale_test/*, metrics/phase_b31_5_*.toml
respect[]: —

## B-32 minified file detection + char cap
does: avg_line_length>500 -> char chunker; universal MAX_CHUNK_CHARS=2000 cap; UTF-8 safe boundaries.
state: wired (CLOSED by C-18 chunking_minified test; MinifiedFileChunked SSB).
supersedes[]: — | superseded_by[]: CLOSED by C-18
residual: none.
key_files: crates/knowledge/src/chunking*
respect[]: —

## B-33 interactive graph viewer (Mermaid + cytoscape)
does: File/Symbol/Decomposition modes, cytoscape+dagre, Mermaid export, blast-radius overlay.
state: partial (CLOSED-partial by C-24; ExportGraphMermaid dispatch + GraphWindow selectors shipped; cytoscape npm dep needs user confirmation).
supersedes[]: — | superseded_by[]: CLOSED by C-24; overlaps H-Track2 canvas code-graph (H-10)
residual: cytoscape dep + full GraphWindow interactivity; H-10 canvas code-graph may absorb this.
key_files: crates/dashboard-v2/src/lib/windows/GraphWindow.svelte, dispatcher ExportGraphMermaid
respect[]: H-10 canvas code-graph direction; Mermaid export is zero-LLM deterministic.

## B-34 entry-point detection
does: 4-heuristic union (Orphan/WellKnownName/FrameworkRoute/Test) over SymbolGraphCache; auto-inject for Designer.
state: wired (CLOSED by C-17; detect_entry_points + EntryPointsDetected SSB + /research PROJECT_STARTING_POINTS seed).
supersedes[]: — | superseded_by[]: CLOSED by C-17
residual: none.
key_files: crates/knowledge/src/entry_points*, /research seeding
respect[]: —

## B-35 context artifacts pipeline
does: [[knowledge.artifacts]] blocks, per-project context_* vector index, GroundingMethod::ArtifactCheck (7th).
state: wired (CLOSED by C-8 index_all_artifacts + C-18 grounding_artifact_check; ArtifactStage in QueryPipeline).
supersedes[]: — | superseded_by[]: CLOSED by C-8,C-18; EXTENDED by B-42 cosmos (8th method, global tier)
residual: none material.
key_files: crates/knowledge/src/artifact*, QueryPipeline ArtifactStage
respect[]: B-42 CosmosStage sits after ArtifactStage; both STANDARD+DEEP only.

## B-36 dashboard-v2 src-tauri wiring
does: 13 D-cmds + 15 net-new Tauri surfaces (sidecar_health, get_gg_signals, get_dkg_stats, etc.).
state: wired (SHIPPED s-59964 + followup; ~637 LOC across 4 files).
supersedes[]: — | superseded_by[]: CLOSED by C-4,C-18
residual: none.
key_files: crates/dashboard-v2/src-tauri/src/{lib,commands}.rs
respect[]: never boot local sidecar (remote-only); never back-port V1 commands.rs verbatim.

## B-37 phase-4M multi-runtime backend
does: RuntimeAdapter trait, 11 backends (Sidecar/AnthropicApi/7 CLI ports/Ollama/Vllm), PerRoleRuntime, fallback chain.
state: wired (SHIPPED commit fd5c2e5; live cloud Sonnet 4.6 PROVEN).
supersedes[]: — | superseded_by[]: CLOSED by C-4; EXTENDED by C-44 (Composer2 backend), I-49 (Bedrock/Azure lanes), I-51 (model catalog)
residual: none material; new backends layer on (Composer2 shipped).
key_files: crates/providers/src/runtime/*, orchestrator RuntimeOrchestrator
respect[]: Tier-C isolation mandatory for CLI subprocesses; I-51 unified model catalog for capability metadata.

## B-38 ambiguity-score continuous metric
does: extends B-17 GroundingSweepGate with continuous ambiguity=1-Σclarity*weight, 4-dim floors, streak counter.
state: partial (AmbiguityScorer wired into task_runner pre-gate per C-2/C-8; floor/streak depth uncertain).
supersedes[]: EXTENDS B-17 | superseded_by[]: CLOSED by C-2,C-8
residual: verify 4-dimension floors + 2-snapshot streak. settle: grep "AmbiguityScorer" in elai.map.
key_files: crates/orchestrator/src/ambiguity*, task_runner pre-gate
respect[]: —

## B-39 chat REFLECT pipeline
does: per-message TRIAGE->REASONER(N=4)->AUDITOR(REFINE cap)->SYNTHESIZE; /reason on|off; SCHEMA 28->29.
state: wired (SHIPPED s-30584 + B-39C answer substitution s-34648).
supersedes[]: — | superseded_by[]: CLOSED by C-33 (REFINE_CAP 2->3 reconciliation)
residual: deferred B-39B/D/E/F (AISLE eval needs 3090, progress UI, system-hint mode) — non-blocking.
key_files: crates/orchestrator/src/chat_reflect.rs, cli/live_cli.rs
respect[]: C-33 REFINE_CAP=3 (RefineBench Table 3).

## B-40 idea-framer + research presets + critic loop
does: FRAMING substate, ProjectClass classifier, [research_presets] TOML, 10 critic categories, 3-round critic loop.
state: wired (SHIPPED; SCHEMA 41; 79/79 PASS; FsmSubstate::Framing).
supersedes[]: — | superseded_by[]: CLOSED by C-12,C-33
residual: none material.
key_files: crates/orchestrator/src/idea_framing*, IdeaFramingWindow.svelte
respect[]: C-12 forwards FramedIdea to Concepts auditor.

## B-41 per-level auditor modes
does: 6-mode auditor (Concepts/Phases/Steps/BabySteps/BabyStepDetail/Plan), BLOCK->respawn-once, 5 rule domains.
state: wired (SHIPPED; SCHEMA 45; 80 GREEN; navigator-spawn deferred, closed by C-12).
supersedes[]: — | superseded_by[]: CLOSED by C-12 (navigator wiring); EXTENDED by D-13 (cross-family Adversary in REVIEW)
residual: none material.
key_files: crates/orchestrator/src/{auditor_mode,per_level_audit_dispatch}.rs
respect[]: D-13 Adversary role extends the REVIEW modes.

## B-42 cosmos artifact tier
does: global ~/.elai/cosmos/ tier, CosmosWorker.tick, GroundingMethod::CosmosCheck (8th), 12 default artifacts, briefing compiler.
state: wired (SHIPPED; SCHEMA 44; cosmos_worker.rs ~700 LOC; spawn_loop dispatcher deferred, closed by C-2/C-16).
supersedes[]: EXTENDS B-35 | superseded_by[]: CLOSED by C-2,C-16 (CosmosStage wired into pipeline)
residual: none material.
key_files: crates/orchestrator/src/cosmos_worker.rs, crates/knowledge/src/cosmos_stage.rs
respect[]: C-16 CosmosStage placement (after ArtifactStage, STANDARD+DEEP); shared LockScope::Index with B-31.

=================================================================
C-PLAN CARDS (C-1 .. C-59)
=================================================================
NOTE: C-SUMMARY marks ALL of these "PLANNED" — that is STALE. Grep confirms C-1..C-53 are overwhelmingly SHIPPED+WIRED (cNN_* contract tests + production callers). Residuals are mostly RunPod live runs and a couple dark benchmark harnesses.

## C-1 SSB emission: knowledge + reasoning producers
does: 7 SSB variants (GrcStepEmitted/GroundingSweepResult/CsqEvaluated/etc.); thread real Ssb into HydeStage/VectorCache/reranker; +1 SCHEMA.
state: wired (SSB variants + emission sites shipped; closes B-4/B-5/B-8 producers).
supersedes[]: — | superseded_by[]: EXTENDED by D-plan reasoning-template archive (D-1)
residual: none material.
key_files: crates/orchestrator/src/{ssb,task_runner,reasoning/engine}.rs, knowledge/{hyde,vector/cache,reranker}.rs
respect[]: no NoopSsbNotifier in producer paths (hard rule after C-1/C-8); E-45 action_type golden lock.

## C-2 SSB emission: orchestration + tree + cosmos + ambiguity + anti-spiral
does: 12 SSB variants (tree nodes, dag_repair, stagnation, goal-drift, watchdog); emit cosmos rows; wire AmbiguityScorer.
state: wired (closes B-6/B-7/B-13/B-38/B-42/4G).
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/orchestrator/src/{ssb,task_runner,anti_spiral_extended,cosmos_worker}.rs
respect[]: E-45 golden lock.

## C-3 Tauri commands: knowledge/reasoning surface
does: replace 7 viewer stubs; 8 Tauri commands (get_pagerank_top/get_freshness_state/etc.).
state: wired (dashboard commands live per B-36 AppState pattern).
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/dashboard-v2/src-tauri/src/commands.rs, ssb_viewer.rs
respect[]: no zero/mock Tauri returns (hard rule after C-3/C-4).

## C-4 Tauri commands: orchestration + cloud spend
does: 10 Tauri commands (get_phase_detail/get_executable_pipeline/list_projects/etc.); RuntimePanel real invoke.
state: wired.
supersedes[]: — | superseded_by[]: cloud-spend surface reframed by E-39 (hard ceiling)
residual: none material.
key_files: crates/dashboard-v2/src-tauri/src/commands.rs
respect[]: E-39 blocking ceiling.

## C-5 cloud mode hardening
does: hard ceiling (DailySpendCapExceeded), warn_at_percent, --mode flag, HYBRID escalation, RouteBlocked SSB, BYOE, estimate_task_cost_usd.
state: wired (closes B-27; RouteBlocked confirmed in map).
supersedes[]: — | superseded_by[]: SUPERSEDED by E-39 (blocks-not-warns hard ceiling); pricing by I-51 model catalog
residual: warn_at_percent softness superseded by E-39; migrate pricing to I-51 models.toml.
key_files: crates/orchestrator/src/cloud*.rs, cli cloud path
respect[]: E-39 ceiling; I-51 catalog; G-5 RRA cloud-lane = action-stream not tokens.

## C-6 dashboard design token alignment
does: tokens.css alignment (bg-void/teal/text), 11px->10px sweep, sidebar 64->48px, Stop button + Ctrl+Enter.
state: wired (closes B-11/B-12/B-14).
supersedes[]: — | superseded_by[]: a full UI rework queued post-backend (auto-memory), out of C scope
residual: none material.
key_files: crates/dashboard-v2/src/lib/tokens.css, 7 Svelte files
respect[]: design-taste.md token doctrine for any new UI.

## C-7 dashboard UX completeness
does: ExploreWindow, closedTabs stack + RecoveryModal, review accept/reject-all, per-hunk controls, responsive breakpoints.
state: wired (closes B-11..14/B-26).
supersedes[]: — | superseded_by[]: EXTENDED by D-26/D-27 (D-plan drawers)
residual: none material.
key_files: crates/dashboard-v2/src/lib/windows/*, base.css
respect[]: —

## C-8 viewer stub eradication: remaining producers
does: wire AmbiguityScorer + CosmosWorker.tick + index_all_artifacts + real ONNX dot-product + real latencies; CosmosStage insert.
state: wired (closes B-3/B-4/B-5/B-7/B-38/B-42).
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/orchestrator/src/task_runner.rs, knowledge/{pipeline,arag}.rs
respect[]: no latency_ms=0 stubs; real Instant::elapsed.

## C-9 autonomous mode core infrastructure
does: AutoAdvanceEvaluator/EvidenceCheck/EscalationPolicy, SplitTaskPlan, TreeAutoAdvancePolicy wiring, [agents.{role}.backend] stanza.
state: wired (closes 4G/B-22).
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/orchestrator/src/{autonomous_gate,task_runner}.rs
respect[]: C-10 anti-spiral extends the decide() path.

## C-10 phase-4G.9 anti-spiral primitives
does: anti_spiral_extended.rs — StagnationPattern(4), ThinkingPersona(5), GoalDriftCanary(Jaccard), GenerationWatchdog(blake3 fingerprint).
state: wired (GoalDriftCanary confirmed wired into agent_pool.rs + task_runner.rs:7606; 5 persona agent files).
supersedes[]: REPLACES lone "3x same step" rule | superseded_by[]: —
residual: none material.
key_files: crates/orchestrator/src/{anti_spiral_extended,goal_drift_canary}.rs
respect[]: —

## C-11 sandbox completeness (Tier C)
does: PR_CAPBSET_DROP + setrlimit + hidepid ordering; wire IpcHookDispatcher; real NSpid tests; Windows AppContainer SID test.
state: wired/partial (SandboxConfig in core/config.rs; closes B-18; Windows AppContainer + full seccomp depth to verify).
supersedes[]: — | superseded_by[]: G-3 deterministic blast-radius gates layer on top
residual: verify Windows AppContainer + real /proc NSpid (SAFETY-CRITICAL — user review).
key_files: crates/orchestrator/src/sandbox.rs, crates/core/src/config.rs
respect[]: never weaken isolation; G-3 FS/egress gates.

## C-12 per-level auditor navigator wiring
does: wire per_level_audit_dispatch emit_started + record_verdict at 5 tier points; CascadeOutcome; live get_per_level_verdicts.
state: wired (closes B-41; live SSB via B-36 AppState pattern).
supersedes[]: — | superseded_by[]: EXTENDED by D-13 (cross-family Adversary in REVIEW)
residual: none material.
key_files: crates/orchestrator/src/per_level_audit_dispatch.rs, AuditTimelineWindow.svelte
respect[]: D-13 Adversary.

## C-13 IPC dispatcher production swap
does: replace InProcessToolDispatcher with IpcToolRegistry; build_production_hook_dispatcher; AdaptiveTool<T> (10 DAIRA markers); squash_wip_commits.
state: wired (closes B-19/B-20; IpcToolRegistry + adaptive_tool.rs confirmed).
supersedes[]: REPLACES InProcess dispatchers | superseded_by[]: —
residual: none material.
key_files: crates/tools/src/{ipc_registry,adaptive_tool}.rs, orchestrator RuntimeWiring
respect[]: no InProcess dispatcher in production.

## C-14 phase-0 debt part 1: dead code strip (~1959 LOC)
does: delete compact.rs/oauth.rs/prompt.rs; strip usage.rs banned symbols; migration_manifest [[command]].
state: wired (closes B-15; deletes verified — compact/oauth/prompt gone).
supersedes[]: DELETES dead modules | superseded_by[]: —
residual: none.
key_files: crates/cli/src/*, core/src/usage.rs, migration_manifest.toml
respect[]: don't reintroduce deleted symbols (greps must stay zero).

## C-15 phase-0 debt part 2: ClockProvider + HashMap ban + entropy quarantine
does: instant_now() on ClockProvider; HashMap->FxHashMap (10 files); ClockProvider injection (~68 sites); entropy-allow annotation; re-enable CI.
state: wired (closes B-15/B-16; entropy quarantine CI active).
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/core/src/clock.rs, check_entropy_quarantine.sh, .github/workflows/windows.yml
respect[]: ClockProvider over SystemTime::now; entropy-allow convention; blake3 keyed_hash.

## C-16 cosmos pipeline integration
does: CosmosStage + with_cosmos_stage builder; CosmosWorker Ssb emission; tokio tick loop + IndexLock; trigger_cosmos_refresh.
state: wired (CosmosStage confirmed in knowledge/cosmos_stage.rs + pipeline.rs + cli/knowledge_stack.rs).
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/knowledge/src/cosmos_stage.rs, orchestrator/cosmos_worker.rs
respect[]: shared LockScope::Index; STANDARD+DEEP only (FAST skips).

## C-17 knowledge indexing resilience
does: KotlinGrammar; auto_resume_indexed_projects + SIGINT/SIGTERM; FileWatcherService; Heading/Paragraph chunks; real ONNX dot-product; entry-point pre-seeding.
state: wired (KotlinGrammar impl LanguageGrammar confirmed; closes B-30/B-31/B-31.5/B-34).
supersedes[]: — | superseded_by[]: —
residual: verify full grammar set + Tier B/C/D scale metrics.
key_files: crates/knowledge/src/dkg/mod.rs, watcher_service.rs
respect[]: LockScope::Watch; tree-sitter retained.

## C-18 test infra backfill: contract tests
does: >=10 contract/integration test files (fsm_phase_transition, symbol_query_dispatch, chunking_minified, graph_export, grounding_artifact, dashboard D-contracts).
state: wired (pure test infra; cNN_* tests populate crates/contracts/tests).
supersedes[]: — | superseded_by[]: —
residual: none.
key_files: crates/contracts/tests/*
respect[]: —

## C-19 test infra backfill: adversarial + Criterion
does: 15 phase-4A adversarial IPC tests; tiered_analysis_bench; adversarial_research; feasibility_adversarial A1-A8; B-9 named tests.
state: wired.
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/orchestrator/tests/*, plugins/benches/*
respect[]: —

## C-20 test infra: metrics TOML backfill + measurement suite
does: `elai measure` subcommand; ELAI_MEASURE_MODE loop; phase_4B/4C/4F/4G/4M metrics; scale TOMLs.
state: wired (elai measure shipped; real measurements, unmeasured_reason annotations).
supersedes[]: — | superseded_by[]: benchmark reporting reframed by E-49 metrics + G-2 RRA
residual: verify RunPod-gated scale metrics non-placeholder. settle: read metrics/last_measure_run.toml.
key_files: crates/cli/src/measure*.rs, metrics/phase_*.toml
respect[]: E-49 STABLE-Pass@1/$-per-CTS metric shape.

## C-21 self-modification + anti-staleness automation
does: check_anti_stale CI; pre-commit LOCKED_KEYS hook; ParameterKey closed enum (12 variants); MistakeBook recording; steering vector refresh.
state: wired (ParameterKey confirmed; closes B-16/B-23 automation half).
supersedes[]: REPLACES AllowedConfigKey::Parameter(String) | superseded_by[]: EXTENDED by D-1 (typed template archive); D-9 (L3 frozen surfaces); LOCKED_KEYS extended by H-28
residual: DGM loop stays wire-dark (ELAI_DARWINIAN_LOOP_ENABLE).
key_files: crates/orchestrator/src/self_mod_bridge.rs, ParameterKey enum, install_pre_commit_hook.sh
respect[]: closed ParameterKey enum; D-9 frozen-surface predicates; H-28 harness-in-frozen-set.

## C-22 FeasibilityProber completeness
does: SystemProbeReport->SystemProbeResult rename; GpuInfo.uuid; ResourceDelta capture; DefaultFeasibilityProber wired; FeasibilityPanel.
state: wired (closes B-21; SystemProbeResult confirmed).
supersedes[]: — | superseded_by[]: EXTENDED by D-18 (typed ObservableEffect)
residual: none material.
key_files: crates/orchestrator/src/feasibility_probe.rs
respect[]: no kill_on_drop(true) (explicit child.kill().await); D-18 effect schema.

## C-23 SessionDag branch UI
does: /session merge|close arms; real fork/merge/close Tauri commands; BranchTree.svelte live IPC; /continue load_handoff; parent_chain reverse fix.
state: wired (closes B-9/B-3; BranchTree in dashboard).
supersedes[]: — | superseded_by[]: reversibility EXTENDED by H-49 (CAS undo-tree)
residual: none material.
key_files: crates/orchestrator/src/session_dag.rs, BranchTree.svelte
respect[]: H-49 undo-tree for new time-travel.

## C-24 symbol impact dashboard surface
does: 4 symbol_* ToolDefinitions in build_tool_catalog; incremental symbol-graph update; KnowledgeDrawer IMPACT panel; GraphMermaid.
state: wired (closes B-29/B-33; symbol tools self-selectable by agent LLM).
supersedes[]: — | superseded_by[]: consumed by D-11 checker registry (code/api checkers); overlaps H-10 canvas code-graph
residual: none material.
key_files: crates/orchestrator/src/dispatcher.rs (symbol arms), GraphWindow.svelte
respect[]: D-11 consumes these; H-10 canvas.

## C-25 compiled pipeline + template crystallizer
does: CompiledPipeline/TemplateCrystallizer (3-success threshold)/CompiledExecutor/store (msgpack, 7d TTL); BackgroundTaskManager; CumulativeTokenTracker.
state: wired (CompiledPipeline wired into pipeline_builder.rs try-template-first path).
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/orchestrator/src/{compiled_pipeline,pipeline_builder}.rs
respect[]: —

## C-26 constrained decoding + retry/futility
does: decoding.rs (DecodingMode enum, ConstrainedDecodingConfig, PER_ROLE_DECODING_DEFAULTS); RetryStrategy; futility detection; 3 EBNF grammars.
state: wired (DecodingMode/ConstrainedDecodingConfig wired into sidecar_client + runtime_orchestrator + decision_governance).
supersedes[]: — | superseded_by[]: re-evaluated by C-37 (structure-snowballing); EXTENDED by I-34 (CRANE hybrid + grammar pinning)
residual: apply C-37 decoding-default re-eval; I-34 blake3 grammar pinning.
key_files: crates/providers/src/decoding.rs, orchestrator grammars/
respect[]: C-37 pilot decision; I-34 grammar pin manifest (GrammarPinManifest present in map).

## C-27 testing-trust T(-1) determinism + T-0 wiring contract
does: SCHEMA 46->47; determinism.rs (DriftComparator, HardwareClass, blake3 keyed_hash); wiring_contract.rs (StubBanAuditor, DkgReachabilityAuditor, ObservableEffectRunner); `elai wiring audit`.
state: wired (WiringContract/StubBanAuditor/DeterminismDrift confirmed; wiring_audit.rs CLI).
supersedes[]: — | superseded_by[]: single-signal drift REPLACED by D-4 (5 tripwires); StubBanAuditor EXTENDED by D-9; test-integrity HARDENED by F-4 (stdout_hash)
residual: adopt D-4 tripwire stack over single DeterminismDrift; F-4 stdout_hash.
key_files: crates/orchestrator/src/{determinism,wiring_contract}.rs, cli/wiring_audit.rs
respect[]: D-4 replaces single-signal; the wiring gate is now the enforced done-gate (F-plan W-1).

## C-28 testing-trust T-1 mutation testing
does: mutation_testing.rs (MutantOutcome, RustMutationRunner cargo-mutants, PythonMutationRunner mutmut); EquivalentMutantCache; ValidationTier::Mutation; blocks AUDITING.
state: wired (MutantOutcome wired into deterministic_audit_gate + mutation_budget).
supersedes[]: — | superseded_by[]: EXTENDED by I-20 (budgeted mutation scheduler on diff data-flow cone)
residual: adopt I-20 budgeted scheduler for cost.
key_files: crates/orchestrator/src/{mutation_testing,mutation_budget}.rs
respect[]: I-20 diff-cone sampling; F-4 test-integrity.

## C-29 testing-trust T-2 IntentPair + T-3 metamorphic
does: intent_pair.rs (IntentPair, min_pairs=3 PLANNING gate); metamorphic_testing.rs (Identity/Inverse/Permutation relations); MrViolated.
state: wired (IntentPair wired into requirements.rs + dashboard commands.rs; MetamorphicRelation in metamorphic_testing.rs + metamorphic_pbt).
supersedes[]: — | superseded_by[]: EXTENDED by C-36 (MUCOCO 9 operators); I-37 (EARS requirement objects)
residual: C-36 operator catalog; I-37 EARS binding.
key_files: crates/orchestrator/src/{intent_pair,metamorphic_testing,requirements}.rs
respect[]: PLANNING->TESTING_FIRST blocked by intent_pairs<3.

## C-30 surgical mode + oracle mode
does: surgical_decomposer.rs (DKG blast-radius, ComplexityClass, RegressionGate); oracle_cache.rs (OracleAnswerCache TTL by tier); 4 DKG ToolDefinitions.
state: wired (OracleAnswerCache wired into cli/repl_command.rs::persisted_default; SurgicalDecomposer present).
supersedes[]: — | superseded_by[]: OracleAnswerCache persistence EXTENDED by H-8; consumed by D-11
residual: none material.
key_files: crates/orchestrator/src/{surgical_decomposer,oracle_cache}.rs
respect[]: H-8 persisted oracle cache; D-11 consumes DKG tools.

## C-31 code completeness L1: FilePlan + FileLineTracker
does: file_plan.rs (FunctionSpec/FileSpec/FilePlan, MAX_FUNCTIONS=20); FILE_PLAN 9th BabyStepSpec field; FileLineTracker TOCTOU guard; MistakeRecorder.
state: wired (FilePlan wired into framework_profile/adapter + skeleton_generator + plugins/tiered_analysis).
supersedes[]: — | superseded_by[]: EXTENDED by D-12 (block-9 MEASURABLE_PROCEDURE)
residual: none material.
key_files: crates/core/src/file_plan.rs, orchestrator TreeNavigatorState
respect[]: D-12 adds procedure block.

## C-32 code completeness L2: skeleton-first + MiniCritic
does: skeleton_generator.rs (BLANK_N fill); mini_critic.rs (PASS/WARN/BLOCK, borrows T1 regex statics); structural_retry.rs (Decompose/Template/FewShot/BestOfN); bridge_template.rs.
state: wired (SkeletonGenerator + MiniCritic wired into task_runner.rs:10255 + c32_dispatch + structural_retry).
supersedes[]: — | superseded_by[]: — (contract-coding C-50 layers ContractViolationCheck into MiniCritic)
residual: none material.
key_files: crates/orchestrator/src/{skeleton_generator,mini_critic,structural_retry,bridge_template,c32_dispatch}.rs
respect[]: AcceptAndComplete ratio>=0.5; C-50 contract checks in MiniCritic.

## C-33 documentation + phase map updates
does: creates local-only phase specs; FailureCategory +3 variants; REFINE_CAP 2->3; B-21 SSB name reconcile; B-0_INDEX status update.
state: wired (failure_taxonomy variants shipped; docs local-only, gitignored).
supersedes[]: — | superseded_by[]: plan-status doc drift SUPERSEDED by I-39 (`elai plan status` from journal)
residual: prefer I-39 journal-derived status over hand-maintained docs.
key_files: crates/orchestrator/src/failure_taxonomy.rs, docs/specs (local)
respect[]: I-39 kills hand-maintained plan drift.

## C-34 phase-4J Empire integration
does: FactoryLine transport/client, BrainManifest, ElaiNeuron MCP, repair pipeline.
state: dark (DEFERRED — external Empire access; no production consumer).
supersedes[]: — | superseded_by[]: — (mirrors B-25 defer)
residual: blocked externally. VERIFY-THEN-SKIP.
key_files: (none) crates/empire/ planned
respect[]: don't build until external prerequisite met.

## C-35 tool attention dynamic gating + lazy MCP
does: ToolGate/GatedCatalog/LazySchemaCache; per-FSM-state gate manifest; tool_gated SSB.
state: wired (GatedCatalog/ToolGate wired into mcp_bridge.rs; DEF_STRUCT confirmed).
supersedes[]: — | superseded_by[]: EXTENDED by H-4 (Tool Search Tool on OAuth path); H-34 (MCP defer-loading)
residual: H-4 advanced-tool-use beta.
key_files: crates/plugins/src/mcp/gate.rs, mcp_bridge.rs
respect[]: H-4/H-34 tool-search; overlapping surface with C-42 (do not run both blind).

## C-36 MUCOCO mutation-operator catalog
does: 9 MetamorphicRelation variants behind mucoco_operators feature; per-language applicability matrix.
state: wired/partial (extends C-29 metamorphic; feature-gated).
supersedes[]: EXTENDS C-29 | superseded_by[]: —
residual: verify mucoco_operators feature wired to a production caller (may be behind flag). settle: grep "mucoco" in elai.map.
key_files: crates/orchestrator/src/metamorphic_testing.rs
respect[]: —

## C-37 PER_ROLE_DECODING_DEFAULTS re-evaluation
does: structure_snowballing_harness; decoding-default migration; possible LightlyConstrained mode.
state: partial/uncertain (pilot RunPod-gated; re-evaluates C-26).
supersedes[]: RE-EVALUATES C-26 | superseded_by[]: —
residual: RunPod pilot decision matrix. settle: grep "structure_snowballing" / "LightlyConstrained".
key_files: crates/providers/src/decoding.rs
respect[]: anti-rubber-stamp FINDING regression under any decoding mode.

## C-38 PageRank production contract + Aider spec fix
does: pagerank_production_contract.rs; hub fixture; DETAILED_SUMMARY §10 correction (symbol-level not file-level).
state: wired (contract asserts compute_repo_map on production path).
supersedes[]: corrects stale Aider spec claim | superseded_by[]: —
residual: none material.
key_files: crates/knowledge/src/pagerank/*, contract test
respect[]: —

## C-39 MCP bridge CLI surface (OpenHands parity)
does: MCPClient (cli/mcp_client.rs); McpClientDispatch via Dispatch trait; --via-mcp flag; CI parity job.
state: wired/partial (mcp bridge shipped; verify --via-mcp co-process launch).
supersedes[]: — | superseded_by[]: EXTENDED by J-2 (elai-as-MCP-server control plane)
residual: J-2 is the fuller MCP control plane (mcp-serve shipped per git log 9db695c).
key_files: crates/cli/src/mcp_client.rs
respect[]: J-2 mcp-serve read-only control plane.

## C-40 GitHub PR adapter (Devin parity)
does: crates github_pr (GhCliAdapter + RestApiAdapter, PrBodyComposer with honesty section, ReviewCommentWatcher); --to-pr/--watch-reviews flags.
state: wired (heavily — GhCliAdapter in cli/github_pr_cmd + dashboard commands_c40 + orchestrator/github_pr/{auto_merge_guard,comment_watcher,semgrep_gate,package_verify,review}). Expanded well beyond original plan.
supersedes[]: — | superseded_by[]: EXTENDED by later PR-evidence work (git log ce76e4f k-3 PR evidence emitter over github_pr adapter)
residual: none material; K-3 layers evidence rendering on top.
key_files: crates/orchestrator/src/github_pr/*, cli/github_pr_cmd.rs
respect[]: K-3 PR-evidence emitter; auto_merge_guard/semgrep_gate additions.

## C-41 skills-driven workflow layer
does: skills/ + registry.toml; SkillRegistry (trigram+keyword); 9 initial skills; SkillsPanel.
state: wired/partial (SkillRegistry — verify production dispatch; overlaps harness .claude/skills).
supersedes[]: — | superseded_by[]: —
residual: verify SkillRegistry has a production caller (may be advisory). settle: grep "SkillRegistry" production caller in elai.map.
key_files: crates/orchestrator/src/skill_registry*, skills/registry.toml
respect[]: —

## C-42 semantic gateway zero-trust MCP fuzzing
does: AuthzPolicy + #[authz] attribute (DENY-ALL default); crates mcp_fuzzer; FuzzReport; AuthzPanel; CI fuzzer job.
state: wired (AuthzPolicy in plugins/authz/{policy,checker,compose,fuzz} + cli/mcp_authz_cmd + dashboard commands_c42).
supersedes[]: — | superseded_by[]: EXTENDED by J-2 (mcp-serve authz), G-4 (live rug-pull re-pin)
residual: none material.
key_files: crates/plugins/src/authz/*, cli/mcp_authz_cmd.rs
respect[]: DENY-ALL default for unlabeled tools; G-4 schema-pin.

## C-43 phase-4H scope-freeze ratification
does: phase-4h-scope-policy.md; #[allow(self_mod_experimental)] guardrail; ablation_harness.rs (proves negative delta).
state: wired/partial (ablation harness + policy doc; cites AEL "mechanisms beyond memory+reflection degrade").
supersedes[]: FREEZES B-23/4H scope | superseded_by[]: reinforced by H-Track7 (DGM-verification)
residual: none material.
key_files: crates/orchestrator/src/ablation_harness.rs
respect[]: AEL scope-freeze; do not add self-mod mechanisms beyond memory+reflection without ablation proof.

## C-44 Composer2 runtime backend tier
does: Composer2Backend impl RuntimeAdapter (SWE-bench Multilingual 73.7, $0.50/$2.50 per M); skill-routing.
state: wired (composer2.rs in crates/providers/src/runtime; c44 contract test; RuntimeBackend::all includes composer2). GATED on API access.
supersedes[]: EXTENDS B-37 | superseded_by[]: pricing metadata by I-51 (models.toml)
residual: verify live API access; migrate pricing to I-51 catalog.
key_files: crates/providers/src/runtime/composer2.rs
respect[]: I-51 unified model capability catalog.

## C-45 CANTANTE contrastive credit attribution
does: AgentRoleDropper (per-role fallback, no panic); credit_attribution_harness; per-role credit table.
state: wired (AgentRoleDropper in crates/agent/src/dropper.rs, wired via agent/main.rs::from_env; credit_attribution_harness). Pilot RunPod-deferred.
supersedes[]: — | superseded_by[]: feeds I-28 (DAG credit assignment / DPO pairs)
residual: 50-task pilot post-RunPod for the actual credit numbers.
key_files: crates/agent/src/dropper.rs, tests/credit_attribution_harness.rs
respect[]: I-28 DAG credit assignment.

## C-46 Uno-Orchestra parsimonious routing
does: TaskDifficultyClassifier (trigram+centrality); SelectiveDelegator (tier policy); selective_delegation SSB.
state: wired (SelectiveDelegator in orchestrator/routing/selective + cli/routing_cmd + dashboard commands).
supersedes[]: — | superseded_by[]: EXTENDED by I-25 (ledger-trained router on own verifier outcomes); K-2 routing economics
residual: I-25 trains a router on ELAI's own outcomes (git log shows routing-economics bench).
key_files: crates/orchestrator/src/routing/selective.rs, cli/routing_cmd.rs
respect[]: I-25 ledger-trained router; K-2 PS-4 routing economics ($/verified-task).

## C-47 case-based calibration tool-use telemetry
does: sled-backed CaseBase (LRU 100/tool); CaseRetriever (Jaccard+Levenshtein); hint injection.
state: wired (case_base.rs in crates/plugins/src/tool_calibration; c47 contract test).
supersedes[]: — | superseded_by[]: —
residual: 50-session replay for the >=10% repeat-error reduction number.
key_files: crates/plugins/src/tool_calibration/case_base.rs
respect[]: —

## C-48 SafeHarbor hierarchical memory guardrail
does: TrustTier (T0 user / T1 codebase / T2 web / T3 model); MemoryView::T0_T1_only filter; provenance propagation.
state: wired (TrustTier lives in crates/memory/src/trust.rs, wired across whole memory crate + cli/memory_cmd).
supersedes[]: — | superseded_by[]: ABSORBED BY E-5 (crates/memory Mem0); consumed by H-31/H-32 (bi-temporal ledger, AUDITOR-OBJECTIVITY firewall)
residual: none — C-48 shipped inside the E-5 memory crate. Do NOT re-author in crates/plugins.
key_files: crates/memory/src/{trust,view,store}.rs
respect[]: E-5 memory crate ownership; H-31 auditor gets ZERO memory injection.

## C-49 iterative distillation for prompt-engineer
does: ExperienceCorpus (.elai/experience/{skill}.md); Distiller (5-gram dedup); avoid-pattern from mistakes.md; ~500 token cap.
state: wired (ExperienceCorpus in cli/experience_cmd.rs + c49 contract test).
supersedes[]: — | superseded_by[]: EXTENDED by I-12 (evidence-gated playbook curator, promote on SSB-verified outcomes)
residual: I-12 promotion gating.
key_files: crates/cli/src/experience_cmd.rs
respect[]: I-12 playbook curator.

## C-50 contract-coding test scaffolding
does: ContractSpec (pre/post-conditions); ContractEmitter post-FILE_PLAN pre-skeleton; ContractViolationCheck in MiniCritic; greenfield template.
state: wired (contract_coding/{emitter,discriminator,spec,scaffolder}.rs in orchestrator; c50 contract test).
supersedes[]: — | superseded_by[]: EXTENDED by I-37 (EARS requirement objects); I-19 (TDD enforcer preflight)
residual: Greenfield-5 pilot for the >=40% functional number.
key_files: crates/orchestrator/src/contract_coding/*
respect[]: I-37 EARS; contracts feed MiniCritic (C-32).

## C-51 Aider polyglot harness adoption
does: vendored aider_polyglot (225 tasks); PolyglotAdapter; SandboxedTestRunner per language; elai benchmark polyglot.
state: dark (no polyglot/PolyglotAdapter symbols in map — harness not built; RunPod/GPU-gated).
supersedes[]: — | superseded_by[]: reframed by E-9 (drop SWE-bench Verified for SWE-Live/Pro/tau2) + E-49 metrics
residual: RunPod-gated; VERIFY-THEN-SKIP against E-9/E-49 benchmark direction before building.
key_files: (none) .elai_cc/external/aider_polyglot/ planned
respect[]: E-9 benchmark reframing; C-55 depends on this.

## C-52 Dafny formal verification spike (research-grade)
does: 3 spike Dafny specs; VerificationHarness; OR infeasibility note.
state: uncertain/dark (spike; likely a doc + optional CI, verify).
supersedes[]: — | superseded_by[]: E-53 posture (Z3/Kani/Verus in CSQ-Critical; Lean HARD-AVOID) reframes formal-verification direction
residual: settle: grep "dafny" in elai.map / check dafny-spike-infeasibility.md.
key_files: (spike) crates dafny specs
respect[]: E-53 formal-verification posture (Dafny is a spike, not the chosen path).

## C-53 NF4 kernel watch + in-context counter-test
does: upstream-watch.md (bitsandbytes); cargo bench nf4_e2e; --baseline-single-prompt flag; counter-test pilot.
state: partial (passive watch + bench harness; counter-test RunPod-gated).
supersedes[]: — | superseded_by[]: SUPERSEDED by E-27 (NF4->AWQ+Marlin 4.4x + FP8 KV is the real quant upgrade); E-28 (FA2 mandatory)
residual: E-27 is the quant path; C-53 is passive. VERIFY-THEN-SKIP the counter-test if E-27 shipped.
key_files: .elai_cc/docs/upstream-watch.md, nf4 bench
respect[]: E-27 quant direction; E-28 FA2.

## C-54 SWE-bench Verified run, 3 configs (FLAGSHIP)
does: 500-task corpus; 3 config TOMLs; SwebenchHarness; scorecards + methodology + honesty statement.
state: dark (no SwebenchHarness/swe_bench symbols in map — never built; RunPod-required).
supersedes[]: — | superseded_by[]: SUPERSEDED by E-9 ("drop SWE-bench Verified, adopt SWE-Live+Pro+tau2"); E-49 metrics; K-2/PS-4 routing-economics is the shipped measurement direction
residual: PRIMARY VERIFY-THEN-SKIP — E-9 explicitly drops SWE-bench Verified. Do not build unless the drain owner overrides E-9.
key_files: (none) benchmarks/swe_bench/ planned
respect[]: E-9 benchmark reframing; PS-4 proof-spine ($/verified-task).

## C-55 Aider polyglot leaderboard submission
does: 2 config TOMLs; scorecards; public DRAFT PR against Aider-AI/aider.
state: dark (depends on C-51 which is dark; RunPod-required).
supersedes[]: — | superseded_by[]: reframed by E-9/E-49
residual: blocked on C-51 + RunPod. VERIFY-THEN-SKIP.
key_files: (none)
respect[]: E-9/E-49; C-40 PR adapter (shipped) for the submission mechanism.

## C-56 codebase-memory DKG 31-repo head-to-head
does: 31 vendored repos; AnswerQualityScorer (Sonnet judge); hub-detection queries; per-repo metrics.
state: dark (RunPod-required benchmark; no runner symbols found).
supersedes[]: — | superseded_by[]: — (measures B-29/B-30/C-38 DKG which ARE shipped)
residual: RunPod-gated. The thing being measured (DKG) is wired; only the benchmark run is missing.
key_files: (none) CodebaseMemoryAdapter planned
respect[]: —

## C-57 OpenHands index + SWE-Cycle + SWE Atlas combo
does: 3 harness adapters; 3 scorecards; combined methodology.
state: dark (RunPod-required; depends C-54).
supersedes[]: — | superseded_by[]: reframed by E-9
residual: blocked on C-54 + RunPod. VERIFY-THEN-SKIP.
key_files: (none)
respect[]: E-9.

## C-58 AgentForge 5-role + RRK reranker head-to-head
does: vendored AgentForge; AgentForgeAdapter; rrk_reranker feature + 8B loader; latency benchmark.
state: dark (RunPod-required; no symbols found).
supersedes[]: — | superseded_by[]: RRK adoption decision config-gated; E-27/E-28 inference direction
residual: RunPod-gated. RRK is a maybe-adopt; default unchanged.
key_files: (none)
respect[]: —

## C-59 cross-config benchmark master report (CAPSTONE)
does: MasterReportAggregator; external_baselines.toml; machine-regenerated master report with honesty section.
state: dark (aggregates C-54..C-58 which are dark; RunPod-required).
supersedes[]: — | superseded_by[]: reframed by E-9/E-49 + G-2 RRA + K-2 routing-economics (the shipped measurement)
residual: entirely gated on the dark benchmark chain. VERIFY-THEN-SKIP; the shipped measurement path is K-2/PS-4.
key_files: (none)
respect[]: E-9/E-49; PS-4 proof-spine number.

=================================================================
D-PLAN CARDS (D-1 .. D-30)
=================================================================
NOTE: D-SUMMARY marks ALL "PLANNED" — STALE. Grep confirms D-1..D-27 substrate is overwhelmingly SHIPPED+WIRED (dNN_* contract tests + production callers in orchestrator/benchmarks + cli). D-28..D-30 genuinely deferred.

## D-1 ReasoningTemplate typed schema + blake3 archive
does: ReasoningTemplate record + ArchiveEntry + ARCHIVE_ENTRY_ADDED/RETIRED SSB; migrate in-prompt templates to typed records. [KEYSTONE]
state: wired/partial (archive substrate present — see D-8 archive_verify shipped; TemplateId used by BAI/gepa). Verify template migration depth.
supersedes[]: EXTENDS C-21 | superseded_by[]: feeds D-5/D-8; template evolution EXTENDED by I-31 GRPO/gepa (rl/gepa_template_evolution.rs present)
residual: verify in-prompt->typed template migration count. settle: grep "ReasoningTemplate" DEF + callers.
key_files: crates/orchestrator/src/{reasoning_template,archive}*.rs, bai.rs (TemplateId)
respect[]: D-8 crypto archive; D-9 L3 frozen surfaces; blake3 hash chain.

## D-2 PPM sidecar (Qwen2.5-Math-PRM-7B)
does: second sidecar at port 8101 for step-level scoring; PpmClient trait; dual-GPU placement.
state: wired (PpmClient in orchestrator/ppm_client.rs, wired via http_sidecar_ppm_hook.rs + prmbench_runner). Live model run needs the 8101 sidecar/RunPod.
supersedes[]: — | superseded_by[]: —
residual: live PPM inference needs the second sidecar booted (hardware-gated); code path wired.
key_files: crates/orchestrator/src/{ppm_client,http_sidecar_ppm_hook}.rs
respect[]: heterogeneous verifier (different family from policy SLM); never run two model instances locally.

## D-3 VerificationResult aggregator + consensus
does: VerificationResult {deterministic,ppm,intrinsic,cross_context}; promotion blocked unless >=2 verifiers concur.
state: wired/partial (consumes D-2 PPM + D-13 Adversary; verify consensus_rule production caller).
supersedes[]: — | superseded_by[]: —
residual: settle: grep "VerificationResult" + consensus in elai.map for production promotion caller.
key_files: crates/orchestrator/src/verification_result*.rs
respect[]: >=2 verifier concurrence rule.

## D-4 tripwire stack (KL+Δρ+IQR+perplexity+DDM)
does: 5 orthogonal tripwires + auto-revert on >=2 concurrent fires; TRIPWIRE_FIRED SSB.
state: wired/partial (replaces C-27 single-signal; verify all 5 tripwires + revert wired).
supersedes[]: REPLACES C-27 single-signal DeterminismDrift | superseded_by[]: —
residual: settle: grep "Tripwire"/"tripwire" in elai.map for the 5-signal impl + revert caller.
key_files: crates/orchestrator/src/tripwire*.rs
respect[]: auto-revert target is D-1 archive; do not fall back to "10% over 3 pipelines".

## D-5 BAI promotion: Beta-posterior Thompson sampling
does: ArmPosterior {pass_count,fail_count} + Thompson sampling + Successive Rejects/LUCB; replaces MSI sigmoid.
state: wired (ArmPosterior in orchestrator/bai.rs, used by rl/gepa_template_evolution + bai_hierarchical; e55 reward-ladder contract). Promotion path is in the DGM/RL flywheel (kill-switch-gated).
supersedes[]: REPLACES MSI sigmoid (B-23/C-21) | superseded_by[]: wired onto real reward by H-48; hierarchical variant is D-29 (deferred)
residual: full promotion loop is behind ELAI_DARWINIAN_LOOP_ENABLE (wire-dark by design, F-11).
key_files: crates/orchestrator/src/{bai,bai_hierarchical}.rs, rl/*
respect[]: H-48 wires BAI on real reward; D-29 hierarchical deferred; kill-switch.

## D-6 time-windowed held-out task store + 9-gram filter
does: held-out store .elai_cc/held-out/{date}/; 9-gram overlap filter; auto-retire >6mo uniformly-passing; HELD_OUT_REFRESH SSB.
state: wired (HeldOutStore in orchestrator/held_out.rs, wired via task_runner.rs:18406 + eval_firewall.rs + cli/darwin_loop_cmd.rs).
supersedes[]: — | superseded_by[]: 9-gram EXTENDED to 13-gram by E-54; eval firewall by H-47
residual: adopt E-54 13-gram + per-family shards.
key_files: crates/orchestrator/src/{held_out,eval_firewall}.rs
respect[]: E-54 13-gram; H-47 date-gate + holdout-budget.

## D-7 cross-domain transfer canary (5 families)
does: 5 orthogonal families; promotion blocked if any non-target regresses >5%; CrossDomainRegression SSB.
state: wired/partial (verify canary production caller; reused by G-10 model-drift canary).
supersedes[]: — | superseded_by[]: reused by G-10 (model-drift regression canary)
residual: settle: grep "canary"/"CrossDomain" for production promotion gate.
key_files: crates/orchestrator/src/cross_domain_canary*.rs
respect[]: G-10 reuses this canary.

## D-8 cryptographic archive + `elai archive verify`
does: blake3 hash chain on mutation candidates; append-only ArchiveEntry; `elai archive verify` CLI + tamper test.
state: wired (archive_verify.rs in crates/cli/src; confirmed in map with tests).
supersedes[]: — | superseded_by[]: uPIP chain EXTENDED by I-18 (sigstore + AI-BOM per-row chain_prev_hash)
residual: I-18 sigstore/Rekor layer.
key_files: crates/cli/src/archive_verify.rs
respect[]: blake3 (never sha256); I-18 chain extension.

## D-9 L3 expansion + frozen-surface access-control
does: LockedKeysAuditor + LOCKED_KEYS_TOUCH SSB; enumerate L3-locked surfaces (thresholds, scoring, archive, verifier_required).
state: wired/partial (extends C-27 StubBanAuditor + C-21 pre-commit; verify LockedKeysAuditor production caller).
supersedes[]: EXTENDS C-27 StubBanAuditor | superseded_by[]: harness-in-frozen-set EXTENDED by H-28; gate-freeze manifest (wiring-enforcement R8)
residual: settle: grep "LockedKeysAuditor" for production caller.
key_files: crates/orchestrator/src/locked_keys*.rs, pre-commit hook
respect[]: H-28 reward-tampering gate + frozen-set; gate-manifest.txt.

## D-10 mode-routing predicate select_grounding_mode
does: GroundingMode enum + select_grounding_mode (A/B/C/D/E); LLM has no input; MODE_SELECTED SSB.
state: wired (select_grounding_mode + GroundingMode in orchestrator/grounding_mode.rs, wired into agent_pool.rs + benchmarks/mode_routing_runner + arc_agi2_runner).
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/orchestrator/src/grounding_mode.rs
respect[]: routing is orchestrator-side, L3-frozen (D-9); LLM never selects.

## D-11 constructive-checker registry
does: .elai_cc/checkers/{math,code,text,graph,api,perf}/; 6 checkers (sympy/z3/compile/registry-check/regex); CheckerResult; Mode A dispatch.
state: wired (CheckerRegistry in orchestrator/checker_registry.rs, wired into agent_pool.rs + baby_step.rs; consumed by i4/i17/i24 tri-layer).
supersedes[]: consumes C-24/C-30 DKG tools | superseded_by[]: feeds I-17 tri-layer eval (L1 HardVeto)
residual: none material.
key_files: crates/orchestrator/src/checker_registry.rs, .elai_cc/checkers/
respect[]: I-17 tri-layer (L1 HardVeto supreme); D-12 procedures reference checkers.

## D-12 OperationalizationCheck baby-step predicate
does: MEASURABLE_PROCEDURE field on BabyStepSpec block 7; claim-emitting steps without procedure BLOCKED at AUDITING.
state: wired/partial (operationalization_contract.rs test present using CheckerRegistry; verify AUDITING-gate production caller).
supersedes[]: EXTENDS C-31 FILE_PLAN | superseded_by[]: —
residual: settle: grep "OperationalizationCheck"/"MEASURABLE_PROCEDURE" for the AUDITING gate.
key_files: crates/orchestrator/src/operationalization*.rs
respect[]: procedure references a D-11 checker.

## D-13 adversary role + cross-family
does: AgentRole::Adversary with mandatory cross-model (different family); CounterEvidenceSchema; falls to Mode E if no alt-family, never self-critique.
state: wired (Adversary trait in orchestrator/adversary.rs; CloudAdversary impl in cloud_adversary.rs; adversary subagent exists).
supersedes[]: EXTENDS C-12 per-level auditor REVIEW | superseded_by[]: cross-family verifier reused by I-17 L3, E-6 MAST adversary
residual: none material.
key_files: crates/orchestrator/src/{adversary,cloud_adversary}.rs, .claude/agents/adversary.md
respect[]: NEVER same-model self-critique; D-3 consensus consumes the verdict.

## D-14 sycophancy probe (inverted-framing)
does: re-run user-aligned claims with framing inverted; divergence -> SYCOPHANCY_FLAG + human quarantine.
state: wired/partial (uses D-13 cross-family infra; verify SycophancyProbe production caller + benchmark D-23).
supersedes[]: — | superseded_by[]: —
residual: settle: grep " sycophancy"/"SycophancyProbe" for production caller.
key_files: crates/orchestrator/src/sycophancy*.rs
respect[]: D-23 detection benchmark gates it.

## D-15 discovery-vs-search labeling (NoveltyClass)
does: NoveltyClass {Search,DistantSearch,DiscoveryCandidate}; DiscoveryCandidate -> human review.
state: wired (NoveltyClass + classify in orchestrator/novelty.rs, wired into benchmarks/arc_agi2_runner + novelty_integration).
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/orchestrator/src/novelty.rs
respect[]: DiscoveryCandidate never auto-published (human queue).

## D-16 conformal refusal calibration (UniCR)
does: UniCR conformal risk control on IDK outputs; calibration set .elai_cc/calibration/; τ nightly L3-only; REFUSAL_DECISION SSB.
state: wired/partial (calibration facade shipped per git log 081722b i-23; verify UniCR τ-updater + Mode-E refusal caller). D-30 bootstrap corpus deferred.
supersedes[]: — | superseded_by[]: unified conformal by I-23 (one impl, 3 consumers — git log confirms calibration facade)
residual: I-23 unified conformal_calibration facade is the canonical impl; D-30 calibration corpus deferred.
key_files: crates/orchestrator/src/{conformal,calibration}*.rs
respect[]: I-23 single conformal impl; H-53/D-16 refusal for out-of-set citations.

## D-17 honest-refusal SLA monitoring
does: refusal_rate = refusals/total_claims; auto-flag <5% (confabulating) or >50% (misconfigured).
state: wired/partial (consumes D-16 refusal events; verify SLA monitor caller).
supersedes[]: — | superseded_by[]: composes with H-39 abstention-aware scoring
residual: settle: grep "refusal_rate"/"honest_refusal" for the session-metric caller.
key_files: crates/orchestrator/src/refusal_sla*.rs
respect[]: H-39 abstention scoring (correct=+1/abstain=0/confident-wrong=-t).

## D-18 ObservableEffect typed (Mode D upgrade)
does: typed ObservableEffect {trigger,expected_observation,success_predicate,sandbox_tier}; >=2 per claim; GOODHART_RISK_FLAG.
state: wired (ObservableEffect in orchestrator/observable_effect.rs + consumed by wiring_contract.rs; d18 contract + integration test).
supersedes[]: EXTENDS C-22 FeasibilityProber | superseded_by[]: feeds the wiring-enforcement observable_effect gate
residual: none material.
key_files: crates/orchestrator/src/observable_effect.rs
respect[]: >=2 effects per claim (Goodhart guard); ties into wiring_contract.

## D-19 PRMBench adversarial gaming benchmark
does: PRMBench fixtures (Deception Resistance + Circular Logic); run PPM on adversarial; gate PPM promotion on >=baseline.
state: wired/partial (prmbench_runner.rs in orchestrator/benchmarks, uses PpmClient). Live run needs PPM sidecar + fixtures/RunPod.
supersedes[]: — | superseded_by[]: —
residual: live PRMBench run (hardware-gated); runner present.
key_files: crates/orchestrator/src/benchmarks/prmbench_runner.rs
respect[]: gates D-2 PPM promotion.

## D-20 ARC-AGI-2 OOD probe
does: ARC-AGI-2 subset under cloud Sonnet with mode-routing on; solve-rate + refusal-rate.
state: wired/partial (arc_agi2_runner.rs in benchmarks + bench_arc_smoke test; uses GroundingMode + NoveltyClass). Live run cloud-gated.
supersedes[]: — | superseded_by[]: —
residual: live ARC run (cloud cost); runner present.
key_files: crates/orchestrator/src/benchmarks/arc_agi2_runner.rs, cli/tests/bench_arc_smoke.rs
respect[]: no local 7B for ARC (cloud Sonnet); DiscoveryCandidate not auto-published.

## D-21 mode-routing coverage benchmark
does: 200 synthetic hand-labeled claims; select_grounding_mode F1 + confusion matrix.
state: wired (mode_routing_runner.rs in benchmarks; gates D-10; F1 harness present).
supersedes[]: — | superseded_by[]: —
residual: none material (deterministic, runnable).
key_files: crates/orchestrator/src/benchmarks/mode_routing_runner.rs
respect[]: F1>=0.85 across 5 modes gate.

## D-22 conformal calibration validation under shift
does: 3 held-out distributions; UniCR empirical refusal within ε=0.1.
state: wired/partial (depends D-16 UniCR + D-6 held-out; verify runner).
supersedes[]: — | superseded_by[]: —
residual: settle: grep conformal validation runner; needs D-16 τ landed.
key_files: crates/orchestrator/src/benchmarks/conformal_validation*.rs
respect[]: I-23 unified conformal.

## D-23 sycophancy detection benchmark
does: Sharma probes; probe-on/off; detection rate + FP rate.
state: wired/partial (gates D-14; verify runner present).
supersedes[]: — | superseded_by[]: —
residual: settle: grep sycophancy benchmark runner.
key_files: crates/orchestrator/src/benchmarks/sycophancy*.rs
respect[]: gates D-14 (>=70% detection).

## D-24 cross-domain canary regression suite
does: 100 synthetic mutations (50 helpful/30 neutral/20 harmful); canary blocks 20 harmful + <=5 helpful.
state: wired/partial (gates D-7; verify runner).
supersedes[]: — | superseded_by[]: —
residual: settle: grep canary regression runner.
key_files: crates/orchestrator/src/benchmarks/canary_regression*.rs
respect[]: —

## D-25 determinism + archive tamper suite
does: 10 tamper scenarios each detected by `elai archive verify` <5s.
state: wired (archive_verify.rs has tamper tests — c27_mismatch_with_entry_index etc. in map).
supersedes[]: — | superseded_by[]: —
residual: none material.
key_files: crates/cli/src/archive_verify.rs (tests)
respect[]: D-8 archive; C-27 DeterminismDrift.

## D-26 ReasoningTemplate + archive dashboard
does: MutationArchivePanel + LineageGraphView (cytoscape); ReasoningDrawer PPM/consensus/tripwire; get_archive_recent commands.
state: uncertain/partial (dashboard surface for D-1..D-8; verify panels mounted).
supersedes[]: EXTENDS C-7 | superseded_by[]: —
residual: settle: grep "MutationArchivePanel"/"get_archive_recent" in dashboard-v2.
key_files: crates/dashboard-v2/src/lib/*, src-tauri commands
respect[]: —

## D-27 mode-routing + refusal-SLA dashboard
does: ModeRoutingPanel + RefusalRateGauge + NoveltyBreakdown + SycophancyFlagStrip; get_mode_breakdown commands.
state: uncertain/partial (dashboard surface for D-10..D-17; verify panels mounted).
supersedes[]: EXTENDS C-7 | superseded_by[]: —
residual: settle: grep "ModeRoutingPanel"/"get_mode_breakdown" in dashboard-v2.
key_files: crates/dashboard-v2/src/lib/*, src-tauri commands
respect[]: —

## D-28 adversarial example farming for PPM (DEFERRED)
does: iteratively generate OOD high-proxy-reward low-quality examples; retrain PPM to immunize.
state: dark (DEFERRED — needs D-5 stable + D-19 baseline + RunPod retraining).
supersedes[]: — | superseded_by[]: —
residual: blocked on >=20 D-5 promotion cycles + RunPod. Skip until unblocked.
key_files: (none)
respect[]: —

## D-29 hierarchical BAI for non-IID rewards (DEFERRED, research)
does: hierarchical Bayesian model over pipeline outcomes (non-IID); infeasibility note is a valid outcome.
state: partial/dark (bai_hierarchical.rs + d29_flat_vs_hierarchical_baseline example EXIST in map; but research-grade, RunPod Gibbs sampler deferred).
supersedes[]: EXTENDS D-5 flat BAI | superseded_by[]: —
residual: baseline harness present; full hierarchical run deferred (RunPod). Partial scaffolding shipped.
key_files: crates/orchestrator/src/bai_hierarchical.rs, examples/d29_flat_vs_hierarchical_baseline.rs
respect[]: D-5 flat baseline for comparison.

## D-30 conformal calibration bootstrap corpus (DEFERRED)
does: bootstrap N=500 calibration tuples from generic corpus, refine per-project.
state: dark (DEFERRED — needs D-16 + external calibration corpus).
supersedes[]: — | superseded_by[]: I-23 unified conformal is the consuming impl
residual: blocked on HuggingFace calibration dataset identification. Skip until unblocked.
key_files: (none)
respect[]: I-23 conformal facade consumes the calibration.

=================================================================
END cards-BCD.md
=================================================================
