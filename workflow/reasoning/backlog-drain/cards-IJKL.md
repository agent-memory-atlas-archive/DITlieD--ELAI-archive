# Backlog-drain cards — letters I, J, K, L

Analyst: read-only corpus pass, 2026-07-16. Grounding sources: per-letter SUMMARY_INDEX + INDEX
files (the "does"); workflow/status/plan-dispositions.json (teikoku walk-recon verdicts, authoritative
for the 25 units it covers); direct filesystem existence checks over .elai_cc/crates/**; elai.map
symbol-ref counts; .elai_cc git log. State legend: wired = producer+consumer file present & referenced
from a non-test seam; partial = some slices shipped, some absent; dark = headline symbol/file absent
or uncalled; uncertain = could not settle cheaply (sidecar-python / harness-doc units), settling check
named in the card.

CAVEAT ON "wired": the status feeder overlay reports "I: 54 wired" but that number is drained-from-debt
optimism. Two feeder OPEN rows already contradict it (I-33 dark, I-23 partial). J/K/L were NOT fully
audited by the feeder (J: 1 audited, K: unaudited, L: 1). So J/K/L states below are MY filesystem
grounding, not the feeder's. The dominant J finding: J is a 112-unit PLANNING letter and only ~9-10
units actually shipped code; the J-31-and-later waves are overwhelmingly dark (headline files absent).

===============================================================================
SUPERSESSION_EDGES  (newer SUPERSEDES/EXTENDS/REPLACES older — evidence in parens)
===============================================================================

Cross-letter edges pointing at B..H (highest-value; these are the reversion-risk edges a drain must respect):

- J-20 SUPERSEDES D-5, D-29, H-48 — "NEW file SUPERSEDING the plain-TS slices of D-5/D-29/H-48
  (never-amend rule)" (J-20_selfimprove_bandit_security_gate.md:3,7,13,17,23,30,37). Supersedes the
  MECHANISM (plain Thompson promotion), REUSES the bandit scaffolding. A drain finishing D-5/D-29/H-48
  must NOT re-author their promotion path; J-20 owns it.
- J-21 SUPERSEDES + EXTENDS H-32 (and extends H-31, E-5) — "commit-aware semantic validation ...
  superseding H-32's existence-only check" (J-21:32); "NEW file EXTENDING H-31/H-32/E-5" (J-21:1).
- J-37 EXTENDS/redraws H-31, E-5 contradiction handling — 3-state ConflictAction -> 4-state
  KEEP/STALE/REPLACE/UNKNOWN, folds SupersededByTier into REPLACE (J-37:31,77,91,229). Extends
  contradiction.rs in place.
- J-59 EXTENDS E-5, H-31, H-32 (per-seat memory namespacing over the single-user substrate) (J-59).
- J-66 EXTENDS I-35 — "the compilation algorithm I-35 lacked" (temporal-monitor engine over SSB) (J-66).
- J-67 EXTENDS J-12 — Ligatti typed GateAction over J-12's boolean Block (J-67).
- J-69 EXTENDS I-18 + J-22 — chain-scan + watermark-monotonicity CONSUMER legs over I-18's chain
  producer (J-69; "does NOT author a competing chain").
- J-80 EXTENDS I-2 and MAP-FIXES I-2 PLANNED->PARTIAL (barrier half already built) (J-80:1784-1787).
- J-83 COMPLETES I-11 — BeliefState PARTIAL -> typed reasoning carrier (J-83:1854-1856).
- J-84 EXTENDS H-42 + I-26 (2-axis reasoning-effort classifier over the budget substrate) (J-84).
- J-86 ACTIVATES I-4 + D-2 + I-24 + I-26 (receipt-gated candidate ladder un-darks I-4) (J-86).
- J-50 EXTENDS I-30 (recovery-trajectory chains, not winning-paths-only) (J-50).
- J-58 EXTENDS I-21 (the pre-exec negotiation loop I-21 lacks) (J-58).
- J-90 fuses L5 cross-lang/dynamic edges INTO J-16's cone; usage-weighted reachability over J-56 (J-90).
- J-95 + K-12 CORRECT stale map labels: "I-5/I-11/H-50 are BUILT, the map's PLANNED labels are stale"
  (K-12:322; J-95:2074 "the design is more wired than the 06-23 map claimed W1-W6"). A drain must
  treat I-5/I-11/H-50 as shipped, not re-build them.
- K-19 / H-43 SUPERSEDE the LIVE DGM GAP-1 (`derived_alpha=blake3(config_bytes)` at
  darwin_admit_loop.rs); H-44 supersedes GAP-2 (keyword-grep at trap_tasks.rs)
  (K-19:16,23,110; K-SUMMARY:498-500). These replace live anti-pattern CODE, not a plan file.
- J-91 EXTENDS J-14 (re-executable RRA); K-20 banks the wedge claim J-91 builds (J-91:1987-1995, K-20).
- J-101 EXTENDS E-37 + G-1 + G-7 trifecta (transformation-surviving taint) (J-101).
- J-102 EXTENDS E-43 + E-24 (config-ingress admission gate) (J-102).
- J-99/J-100 WIRE the shipped G3-G6 async primitives + I-1 supervisor + the I-17 watchdog stub
  (action_type_lock.rs:692) (J-99, J-100:2010).
- J-39 REPLACES the fixed budget constant in DefaultPhaseSizingSplitter with J-34's live ceiling
  (J-39:1075, phase_sizing_splitter.rs:150).
- I-23 UNIFIES D-16 + G-7 + conformal_calibration + calibration_bootstrap behind ONE facade so
  tau/coverage cannot drift between consumers (I-SUMMARY:427-440). Supersedes scattered impls with one.
- L-2 GENERALIZES G-7's claim_score from the agent's own output to any backend's claim (L-2:18-22).
- L-3 ACTIVATES D-10/D-11/H-53 grounding methods over arbitrary claims (L-3).
- L-16 REPLACES conversation.rs::compact_session (the prefix-rewriting mutator) with a pure projection;
  recover_stubs -> recompute-with-larger-budget (L-16:73,139; L-SUMMARY:133-139). Re-specs an H-49/E-25
  substrate — highest structural-revert risk in L.
- L-18 RE-POINTS check_lossless_compaction (K-12 R1) at the view and fail-closes it (L-18).
- L-22 / L-24 REPLACE the I-4 N-resample propose leg with a Verbalized-Sampling one-decode propose
  contract (L-22:23,37,49; L-24:257; L-SUMMARY:243). Partial supersession of I-4's propose mechanism only.

Intra-cluster / K-J PROVE-vs-BUILD splits (K "proves/packages" the number, the J-track "builds" the engine;
NOT reversion edges but a drain must not double-build):

- K-11 EXTENDS K-3 (human-viewable evidence render extends the verdict render) (K-11:291-299).
- K-25 EXTENDS K-8 (continuous absorption watchdog makes the one-shot audit standing) (K-25:563).
- K-5 fits UNDER K-4's ceiling (depends K-4) (K-5:200).
- K-9 depends J-9 (model-refresh cadence rides the hardware catalog) (K-9:273).
- K-15 PACKAGES J-83/J-85 (reasoning-budget bench, BENCH-PENDING on the GRC-v3 J-track).
- K-17 PACKAGES J-34 (context-mgmt differentiator, BENCH-PENDING on J-34).
- K-18 PACKAGES the VCG J-track (J-96/J-97); K-20 PACKAGES J-14/J-91.
- K-19 pre-gates the VCG J-track (J-96..J-98) with SF-1..SF-4; K-16 pre-gates J-83..J-86; K-21 pre-gates
  J-87; K-13 pairs with J-101/J-102; K-14 pairs with J-99/J-100. K-native = doc/rule; technical = J.
- J-105 NEW FILE EXTENDING J-21 (never amend) (J-105:2188).
- J-72 EXTENDS J-18 (in-domain retriever adaptation folds into J-18's swap) (J-72).
- J-76 EXTENDS J-34 + H-50 as a NEW file (context-rot calibration) (J-76).
- J-108 EXTENDS J-42 + J-86 + J-38 (multi-hop depth operator), GATED on J-107 (J-108).
- J-62 folds into J-52 LEG 3 (reuses PathRuleInjector; "a third file would be a GD-1 P0", J-0:337).
- J-64 ADDITIVE to the shipped direct-CLI bridges — goose deprecated ITS OWN claude_code for ACP;
  J-64 adopts ACP additively, does NOT remove ELAI's broader direct-CLI coverage (J-64:1624, J-0:355).
- J-31 (system canvas) EXTENDS the shipped H-plan Track-2 canvas; J-60/J-98 add process/memory altitudes.

===============================================================================
CARDS
===============================================================================

------------------------------- I-plan (54 units) -------------------------------
Authored 2026-06-10 (Tracks 12-20). MOSTLY SHIPPED per walk dispositions + file existence. Genuine dark:
I-16, I-25, I-28, I-29, I-30, I-31, I-33. Partial: I-23. RunPod/flywheel tail (I-16/I-30/I-31) blocked
on GPU + the H-43..48 filter.

id: I-1 | does: process-separated Supervisor executes all side effects; generator emits pure Intent->Directive; saga compensation on crash | state: wired (directive.rs present) | supersedes: - | superseded_by: - | residual: contract-test the generator-cant-reach-gate-code invariant; frontend verdict rows | key_files: crates/orchestrator/src/directive.rs | respect: -
id: I-2 | does: CoW/worktree-pooled isolated workspaces for parallel branches; deterministic barrier merge | state: wired (sandbox/branch_workspace.rs; J-80 map-fixed to PARTIAL then extended) | supersedes: - | superseded_by: J-80 (pool-lifecycle half) | residual: pool lifecycle lands in J-80 | key_files: crates/orchestrator/src/sandbox/branch_workspace.rs | respect: J-80 lifecycle layer
id: I-3 | does: harden SSB single-writer throughput + atomic checkpoint saves; measured inserts/sec | state: wired (benchmarks/ssb_throughput_runner.rs) | residual: io_uring linux opt-in experiment | key_files: crates/orchestrator/src/benchmarks/ssb_throughput_runner.rs, ssb.rs | respect: -
id: I-4 | does: evidence-ranked search ladder (best-of-N -> tournament -> PUCT), selects by receipts | state: wired (search/ladder.rs::run_best_of_n; disposition ALREADY-DELIVERED v1) | superseded_by: L-22/L-24 (propose-leg only) | residual: v2 tournament + v3 PUCT tree unshipped; J-86 activates receipt-gated ladder | key_files: crates/orchestrator/src/search/ladder.rs, score.rs | respect: L-22 VS propose contract; J-86 receipt selector
id: I-5 | does: pristine context injector — rebuild from parent chain, hard-exclude failed-sibling tracebacks | state: wired (pristine_path_filter.rs; K-12/J-95 confirm BUILT vs stale map) | residual: vividness-tax A/B tile | key_files: crates/orchestrator/src/pristine_path_filter.rs | respect: do NOT re-build (map label was stale)
id: I-6 | does: immutable PrivilegedAnchor slot appended last carrying TaskSpec/constraints | state: wired (disposition: drained + committed 4a8a43f) | residual: - | key_files: crates/orchestrator/src/context_budget.rs, persona_anchor.rs | respect: -
id: I-7 | does: strict local-first waterfall; condense-then-escalate to cloud on local fail | state: wired (escalation_waterfall.rs / escalation_policy.rs) | residual: cascade trace tile | key_files: crates/orchestrator/src/escalation_waterfall.rs | respect: -
id: I-8 | does: aletheia voluntary-abort tool; dead-end AST-hash pruning; 15s halting bound | state: wired (dead_end_cache.rs; abstention credited via H-39) | residual: - | key_files: crates/orchestrator/src/dead_end_cache.rs | respect: -
id: I-9 | does: error-amplification (alpha_t) telemetry + active annealing when alpha>1 | state: wired (benchmarks/error_amplification_runner.rs + cli bench_error_amplification) | residual: alpha trend tile | key_files: crates/orchestrator/src/benchmarks/error_amplification_runner.rs | respect: -
id: I-10 | does: cache-invariant prefix discipline + TTL tiering + hit-rate hard guard | state: wired (cache_economics.rs) | residual: TTL/economics fields on cost tile; L-17 makes it a freeze-cause SM | key_files: crates/orchestrator/src/cache_economics.rs | respect: L-17 freeze-epoch controller
id: I-11 | does: typed versioned BeliefState replacing raw history on cross-model/tier handoff | state: partial (belief_state.rs present, wired at escalation_waterfall.rs:239; J-83 completes into reasoning carrier) | superseded_by: J-83 (completion) | residual: per-step receipt gate + commit-freeze = J-83 | key_files: crates/orchestrator/src/belief_state.rs | respect: J-83 carrier, K-12 critical_atoms
id: I-12 | does: evidence-gated playbook curator (promote on >=k SSB-verified outcomes) | state: wired/partial (rule_curator.rs) | residual: skills export script; retrieval-time reuse = J-97 | key_files: crates/orchestrator/src/rule_curator.rs | respect: J-97 gated reuse, K-19 SF-gates
id: I-13 | does: write-time memory scan (injection/unicode/secrets) + dual serialization + TraceQuery | state: wired (memory_guard.rs) | residual: FP-bound before default-ON | key_files: crates/orchestrator/src/memory_guard.rs | respect: -
id: I-14 | does: deterministic context compression (LSP slicing + tree-sitter fold) + reversible receipt | state: wired (context_transform.rs) | residual: Tier-2 LLMLingua gated leg | key_files: crates/orchestrator/src/context_transform.rs | respect: -
id: I-15 | does: cache intermediate DKG/research summaries + blake3 exact fast-path | state: wired (oracle_cache.rs) | residual: cache-source badge | key_files: crates/orchestrator/src/oracle_cache.rs | respect: -
id: I-16 | does: sidecar KV pruning + Int4 KV quant EVALUATION vs AWQ baseline | state: dark (disposition: genuine unshipped BUILD, KvTuneEval absent; RunPod-gated) | residual: whole build + `elai bench kv-tune`; needs GPU | key_files: python/elai_inference/server.py (flags) | respect: -
id: I-17 | does: tri-layer verifier stack (L1 hard veto / L2 scorer / L3 adversary) + flake quarantine | state: wired (verifier_stack.rs; disposition delivered, consumed by search ladder) | residual: cross-lang verifier packs Tier-2 | key_files: crates/orchestrator/src/verifier_stack.rs | respect: -
id: I-18 | does: per-row chain_prev_hash + 5-layer stack hash + opt-in Rekor + AI-BOM | state: partial (archive_chain.rs/archive_crypto.rs present; Rekor/AI-BOM legs uncertain) | superseded_by: J-69 (consumer legs) | residual: Rekor push + CycloneDX-ML; settle by grep rra.rs for AI-BOM | key_files: crates/orchestrator/src/archive_chain.rs, rra.rs | respect: J-69 chain-scan gate
id: I-19 | does: symbol-level TDD preflight (src edit requires test-symbol delta) | state: wired (tdd_preflight.rs; disposition delivered) | residual: FP-bound before default-ON | key_files: crates/orchestrator/src/tdd_preflight.rs | respect: -
id: I-20 | does: budgeted mutation scheduler (diff-cone mutants, sampled-CI early stop, survivor directives) | state: wired (mutation_budget.rs) | residual: proptest/Hypothesis parallel lane | key_files: crates/orchestrator/src/mutation_budget.rs | respect: -
id: I-21 | does: sprint contracts + isolated evaluator + bounded Ralph loop | state: wired (sprint_contract.rs; disposition delivered) | superseded_by: J-58 (adds pre-exec negotiation) | residual: pre-exec negotiate-until-agree = J-58 | key_files: crates/orchestrator/src/sprint_contract.rs | respect: J-58 negotiation loop
id: I-22 | does: progress-shaped rewards (delta phi from deterministic observables) + first-error truncation | state: wired (search/score.rs DeltaProgressScorer, 2 prod callers per disposition) | residual: progress sparkline | key_files: crates/orchestrator/src/search/score.rs | respect: -
id: I-23 | does: one conformal-calibration impl, three consumers (D-16 / G-7 / I-4 fork) | state: partial (calibration/mod.rs CalibrationFacade wired; disposition SET_ASIDE — UncertaintyFork absent, live facade-bypass at production_agent_rpc.rs:86) | residual: UncertaintyFork blocked on I-4 PUCT seam (unshipped); calibration_fork_threshold.toml needs an ID-5 measurement run; kill the direct-import bypass | key_files: crates/orchestrator/src/calibration/mod.rs | respect: -
id: I-24 | does: isomorphic AST-preserving canaries into L2 scoring stream; neutralize style-biased scorers | state: wired (canary_injector.rs, caller verifier_stack.rs; disposition shipped) | residual: scorer-health badge | key_files: crates/orchestrator/src/canary_injector.rs | respect: -
id: I-25 | does: ledger-trained small router on own verifier outcomes + cost-shape pre-routing | state: dark (disposition: genuine unshipped BUILD, CostProfiler absent) | residual: whole build + offline trainer | key_files: (planned) escalation_policy.rs consumers | respect: -
id: I-26 | does: dynamic thinking budgets (serial-depth scaling, 85% collapse, plateau stop) | state: wired (reasoning_budget.rs BudgetController, multiple prod callers per disposition) | residual: budget bar tile | key_files: crates/orchestrator/src/reasoning_budget.rs | respect: J-84 sizes it
id: I-27 | does: budgeted-convergence instrument (verified-outcome vs compute Pareto curve) | state: wired (cli bench_convergence.rs) | residual: curve tile | key_files: crates/cli/src/bench_convergence.rs | respect: K-2/K-15/K-18/L-10 COMPOSE this runner, do not fork
id: I-28 | does: DAG credit assignment (backward causal credit + DPO-pair extraction) | state: dark (disposition: DagCreditAssigner absent) | residual: whole build; feeds I-30 | key_files: (planned) dag_credit.rs | respect: -
id: I-29 | does: diffusion draft lane (dLLM fast untrusted drafts behind the veto stack + CRDT merge) | state: dark (disposition: DraftMerger absent; new-dep+network gated) | residual: whole build + user-approved dep | key_files: (planned) draft_merge.rs | respect: -
id: I-30 | does: verifier-ranked SFT exhaust (winning paths + negative pairs -> JSONL, provenance-hashed) | state: dark (disposition: SftExhaust absent; NO crates/training crate exists) | superseded_by: J-50 (recovery-trajectory extension) | residual: whole build; RunPod for the LoRA run | key_files: (planned) crates/cli/src/sft_exhaust.rs | respect: J-50 recovery extractor
id: I-31 | does: GRPO flywheel on execution-grounded rewards (blocking guards H-43/44/28 + I-24) | state: dark (disposition: GrpoHarness absent; no training crate) | residual: whole build; blocked on H-43/44/28 LIVE + RunPod + darwinian allowlist | key_files: (planned) crates/training/grpo_harness.rs | respect: J-20 safe-promotion precondition
id: I-32 | does: structural role scoping — strip persona/narrative on input side per role | state: wired/partial (role_contracts.rs, report_fns; cited live by J-106) | residual: output-side twin = J-106 | key_files: crates/orchestrator/src/role_contracts.rs | respect: J-106 output gate
id: I-33 | does: mechanical decision governance — governed_prompt_decision entry | state: dark (feeder OPEN: governed_prompt_decision has ZERO callers; escalated, blocked on owner seam-design ELAI_E25_PROMPT_HANDLER_ENABLE) | residual: wire into task_runner.rs hook | key_files: crates/orchestrator/src/decision_governance.rs | respect: -
id: I-34 | does: CRANE hybrid grammar pinning + re_validate_output | state: wired (providers/decoding.rs re_validate_output, 3 prod callers; disposition shipped) | superseded_by: J-15 (adds llguidance leg, amendment) | residual: llguidance Rust leg (new dep, user-confirm) = J-15 | key_files: crates/providers/src/decoding.rs | respect: J-15 llguidance amendment
id: I-35 | does: temporal tool-sequence enforcement | state: partial (tool_sequence.rs present) | superseded_by: J-66 (compilation engine) | residual: monitor-synthesis engine = J-66 | key_files: crates/orchestrator/src/tool_sequence.rs | respect: J-66 ssb-monitor
id: I-36 | does: cross-model prompt compilation (CompiledPromptStore) | state: wired (compiled_pipeline.rs; disposition delivered, promote_variant path) | residual: - | key_files: crates/orchestrator/src/compiled_pipeline.rs | respect: J-57 prompt-promotion gate
id: I-37 | does: EARS requirement objects | state: wired (requirements.rs EarsRequirement, callers task_runner/score/eval; disposition shipped) | residual: - | key_files: crates/orchestrator/src/requirements.rs | respect: -
id: I-38 | does: q-delta statistical floor bench | state: wired (cli bench_qdelta.rs::run, QdeltaConfig; disposition shipped) | residual: - | key_files: crates/cli/src/bench_qdelta.rs | respect: K-4 gate-tax shares its corpus/runner
id: I-39 | does: plan-state-from-repo (plan_status/I-39) | state: partial (plan_skeleton.rs present) | residual: verify plan_status verb wired | key_files: crates/orchestrator/src/plan_skeleton.rs | respect: -
id: I-40 | does: trust-layer wrapper / `elai wrap` RRA spec | state: partial/wired (rra.rs + wrap path) | residual: verify wrap CLI arm | key_files: crates/orchestrator/src/rra.rs | respect: L-14 build front-door consumes it
id: I-41 | does: spec-time critique lenses | state: partial (critique/ dir present) | residual: settle by reading critique/ | key_files: crates/orchestrator/src/critique/ | respect: -
id: I-42 | does: critique-only profile | state: partial (profile_selector.rs present) | residual: verify critique-only arm | key_files: crates/orchestrator/src/profile_selector.rs | respect: -
id: I-43 | does: claim-scoper kill gate | state: wired (cli bench_claim_scope, main.rs:1438 CliAction::BenchClaimScope; disposition delivered) | residual: - | key_files: crates/cli/src/bench_claim_scope.rs | respect: K-1 dossier consumes the verdict
id: I-44 | does: zero-retention export mode | state: partial/wired (retention.rs present) | residual: verify export arm | key_files: crates/orchestrator/src/retention.rs | respect: K-6 compliance export composes it
id: I-45 | does: objective-hash sealed receipts | state: partial/wired (result_manifest.rs; cited live by J-89/J-91/L-5) | residual: - | key_files: crates/orchestrator/src/result_manifest.rs | respect: -
id: I-46 | does: bash interceptor redirect (refuse-with-redirect) | state: wired (bash.rs; cited live by J-100/J-103) | residual: widen to Read tool = J-103 | key_files: crates/orchestrator/src/bash.rs | respect: J-103 read-ladder widens it
id: I-47 | does: verifier input hardening (scream-path) | state: wired (prompt_hardening.rs; disposition delivered turn-3) | residual: dropped_row_count accessor wire-dark (the D-030 first-catch) | key_files: crates/tools/src/prompt_hardening.rs, crates/orchestrator/src/prompt_hardening.rs | respect: -
id: I-48 | does: prompts-as-artifacts | state: wired (prompt_artifacts.rs) | residual: - | key_files: crates/orchestrator/src/prompt_artifacts.rs | respect: -
id: I-49 | does: bedrock/azure direct lanes | state: partial (bedrock.rs/azure_openai.rs present; disposition: refine-first, FULL-FORM expansion required, no executable plan) | residual: SUMMARY needs full-form before EXECUTING; J-14 fixture contract | key_files: crates/providers/src/bedrock.rs, azure_openai.rs | respect: -
id: I-50 | does: subscription auth broker | state: partial/wired (auth_broker_emit.rs) | residual: OAuth/device-code lane | key_files: crates/orchestrator/src/auth_broker_emit.rs | respect: J-30/J-64 keep this orthogonal
id: I-51 | does: model capability catalog (static models.toml) | state: partial/wired (tier_matrix.rs/hardware_catalog.rs) | residual: - | key_files: crates/orchestrator/src/tier_matrix.rs | respect: J-9/J-54/K-24 consume; NO dynamic registry (rejected 3x)
id: I-52 | does: checkpoint-rewind tool | state: partial/wired (checkpoint.rs) | residual: verify rewind CLI | key_files: crates/orchestrator/src/checkpoint.rs | respect: -
id: I-53 | does: recipe-runner tool pack | state: partial (recovery_recipes.rs present; NOTE J-100 records "I-53 declined" for cron) | residual: cron/scheduler declined | key_files: crates/orchestrator/src/recovery_recipes.rs | respect: -
id: I-54 | does: structural AST edit tool | state: wired (structural_edit.rs) | residual: - | key_files: crates/orchestrator/src/structural_edit.rs | respect: -

------------------------------- J-plan (112 units) -------------------------------
Authored 2026-06-10..2026-07-10, the grab-bag. GROUNDED FINDING: only ~9-10 units shipped code; the
J-31-and-later waves are overwhelmingly DARK (headline .rs/.py files absent on disk, confirmed by
existence checks + elai.map 0-ref counts for front_door/modality_router). J is NOT J-CLOSED for
authoring but is the least-built letter relative to size.

WIRED / partial (files present):
id: J-1 | does: deterministic image-ingestion gateway + perception scaffold + /see skill | state: wired (perception/ dir present, 344 elai.map refs) | residual: J-8 upgrade pack + J-77 faithfulness routing | key_files: crates/orchestrator/src/perception/ | respect: J-8, J-77 extend it
id: J-2 | does: elai-as-MCP-server control plane (read-only verbs, authz-gated) | state: wired (mcp_stdio.rs + commit 9fbd9c9 "elai mcp-serve") | residual: mutating verbs stay tier-2 defer | key_files: crates/orchestrator/src/mcp_stdio.rs, mcp.rs | respect: -
id: J-9 | does: hardware detection + per-lane model eligibility catalog (DATA-driven) | state: wired (hardware_catalog.rs + commit 9db695c "hardware model catalog") | residual: Tier-2 CLI/panel deferred | key_files: crates/orchestrator/src/hardware_catalog.rs, data/hardware_catalog/model_catalog.toml | respect: K-9 cadence rides it
id: J-13 | does: local-lane token determinism (batch_invariant_ops + seeded Gumbel + seed thread) | state: uncertain (python sidecar leg; runtime_client.rs seed slot exists) — settle: grep server.py for batch_invariant + runtime_client.rs:597 seed | residual: sidecar wrap + RRA thread; remote-gated | key_files: python/elai_inference/server.py, crates/cli/src/runtime_client.rs | respect: -
id: J-19 | does: vLLM-V1 alternative backend (native /v1/messages + AWQ-Marlin + APC) | state: wired (providers/vllm.rs + vllm_client.rs) | residual: swap-out stays user decision; live legs RunPod-gated | key_files: crates/providers/src/vllm.rs, vllm_client.rs | respect: -
id: J-21 | does: commit-aware semantic memory validation + poisoning hardening | state: partial/wired (memory/semantic_validation.rs + citation_validity.rs + contradiction.rs) | supersedes: H-32 (existence-only), extends H-31/E-5 | residual: activation-decay TTL, raw-evidence tier | key_files: crates/memory/src/semantic_validation.rs, citation_validity.rs | respect: -
id: J-28 | does: token ledger — usage unification + per-turn UsageTurnEvent + rollup | state: wired (usage_ledger.rs, 253 elai.map refs + commit) | residual: J-29 surfaces consume it | key_files: crates/orchestrator/src/usage_ledger.rs | respect: K-2/K-24/J-49 consume the meter
id: J-37 | does: STALE staleness adjudication (4-state verdict + propagation-aware + UNKNOWN->abstain) | state: partial (memory/semantic_contradiction.rs + contradiction.rs present) | supersedes: H-31/E-5 contradiction (3->4 state, folds SupersededByTier) | residual: state-schema Ω TOML + retrieval readout gate; HARD-dep on a live memory store consumer | key_files: crates/memory/src/semantic_contradiction.rs, contradiction.rs | respect: -
id: J-53 | does: OpenAI-compat tool-input repair (4 ordered shape repairs at the unmasked seam) | state: wired (providers/tool_input_repair.rs) | residual: - | key_files: crates/providers/src/tool_input_repair.rs | respect: -
id: J-83 | does: GRC-v3 BeliefState reasoning carrier (per-step receipt gate + commit-freeze) | state: partial (belief_state.rs present as I-11; J-83 completion legs uncertain) — settle: grep belief_state.rs for receipt-gate/commit-freeze | supersedes: completes I-11 | residual: per-step receipt gate + commit-freeze if absent | key_files: crates/orchestrator/src/belief_state.rs | respect: K-16 RR-gates
id: J-22 | does: journal payload field-shape lock + writer-isolation contract test | state: partial (action_type_lock.rs present; deny_unknown_fields snapshot uncertain) — settle: grep for deny_unknown_fields | residual: field-shape snapshot test | key_files: crates/orchestrator/src/action_type_lock.rs | respect: -
id: J-96 | does: verified case-graph CORE (mint-on-proof case node + recurrence-hit-rate) | state: partial (benchmarks/case_reuse_runner.rs + case_reuse_fixtures present; CaseNodeMinted uncertain) — settle: grep CaseNodeMinted | residual: typed case node + self-wiring edges if absent | key_files: crates/orchestrator/src/benchmarks/case_reuse_runner.rs | respect: K-18/K-19 pre-gate; J-97 reuse loop

DARK (headline file/module ABSENT on disk — the whole build is residual unless noted):
id: J-3 | does: harness undo + per-session file backups (PreToolUse hook + undo.py) | state: dark (.claude/tools/undo.py absent) | key_files: (planned) .claude/tools/undo.py | respect: -
id: J-4 | does: handoff FILES-TOUCHED machine ledger | state: dark (.claude/tools/handoff_ledger.py absent) | key_files: (planned) handoff_ledger.py | respect: -
id: J-5 | does: litegame harness port (checklist) | state: dark/uncertain (targets D:\.liteGAME_workflow, out of this tree) | key_files: - | respect: -
id: J-6 | does: incremental-eval skill | state: uncertain (skill may exist at .claude/skills/incremental-eval) — settle: ls .claude/skills/incremental-eval; NOTE the incremental-eval skill IS in the loaded skill list, likely wired | key_files: .claude/skills/incremental-eval/ | respect: -
id: J-7 | does: ext-steal intake closeout (skillspector smoke + policy amend + pointer lines) | state: uncertain (doc/log unit) — settle: grep workflow/skill-imports.log | key_files: .claude/rules/skill-import-policy.md | respect: -
id: J-8 | does: VLM perception research upgrade pack (resize table DATA + coord schema-v2 + eval) | state: dark (perception/ exists but data/perception/*.toml + bench_vlm_eval uncertain, likely dark) | key_files: (planned) data/perception/*.toml | respect: J-1 gateway
id: J-10 | does: insight-backprop tree memory (extract/synthesize/inject/provenance) | state: dark (insight/ dir absent) | key_files: (planned) crates/orchestrator/src/insight/ | respect: J-96/J-97 VCG, J-110 CONVOLVE rider
id: J-11 | does: arbor/combench steal-intake closeout (amendment blocks) | state: dark/doc (amendment blocks into I-4/I-2/... ; settle by grep those files) | key_files: workflow/reasoning/J-11-decisions.md | respect: -
id: J-12 | does: runtime intervention harness (4-layer deterministic, LLM-free) | state: dark (intervention/ dir absent) | key_files: (planned) crates/orchestrator/src/intervention/ | respect: J-67 extends to typed GateAction
id: J-14 | does: repro-pack + RRA verify upgrade (tiered verdicts + first-divergence locator) | state: dark/partial (rra.rs present but T0/T1/T2 tiers + export-fixture uncertain) — settle: grep rra.rs:474 for tier verdicts | key_files: crates/orchestrator/src/rra.rs | respect: J-91/K-20 build on it
id: J-15 | does: determinism steal closeout (llguidance amendment into I-34 etc) | state: dark/doc | key_files: workflow/reasoning/J-15-decisions.md | respect: -
id: J-16 | does: blast-radius / anti-regression gate (auditor-owned, USER #1 priority) | state: dark/partial (blast_radius.rs ABSENT; blast_radius_ssb_adapter.rs present suggests partial SSB emission only) — settle: grep ConeComputed | residual: the enforced cone gate | key_files: (planned) crates/orchestrator/src/blast_radius.rs | respect: J-90 fuses cross-lang into it
id: J-17 | does: plan-level compositional safety (pre-dispatch capability aggregation) | state: dark (compositional_policy.rs + sandbox_audit.rs absent) | key_files: (planned) compositional_policy.rs | respect: -
id: J-18 | does: P1 retrieval hardening (CodeRankEmbed swap + LSP channel + agentic-grep) | state: dark/partial (knowledge/pipeline.rs present but lsp_channel absent) — settle: grep CodeRankEmbed/index_version | key_files: crates/knowledge/src/vector/ | respect: J-72 in-domain adapt folds in
id: J-20 | does: non-stationary bandit + security-invariant promotion gate | state: dark/partial (bai.rs/bai_hierarchical.rs/darwinian_bai.rs present; DS-TS/change-point + SecurityInvariantGate uncertain) — settle: grep NonStationaryReset/SecurityInvariantGate | supersedes: D-5/D-29/H-48 plain-TS | residual: DS-TS legs + CWE pre-promotion gate | key_files: crates/orchestrator/src/bai.rs, darwin_admit_loop.rs | respect: D-5/D-29/H-48 — do not re-author their path
id: J-23 | does: multi-language review-gate catalog (SARIF ingest + arch_gate DATA) | state: dark (sarif_ingest/arch_gate absent) | key_files: (planned) | respect: -
id: J-24 | does: non-prompt steering (sidecar /v1/steering) + context-frugal tools | state: uncertain (sidecar python leg) — settle: grep server.py steering | key_files: python/elai_inference/server.py | respect: -
id: J-25 | does: mimocode context+memory mechanics (typed checkpoint template + boundary guard) | state: dark (context_boundary.rs absent) | key_files: (planned) context_boundary.rs | respect: -
id: J-26 | does: sia evaluator ABI + task packs (subprocess evaluator, results.json) | state: dark/uncertain (evaluator_abi module) — settle: grep evaluator_abi | key_files: (planned) | respect: H-43 consumer
id: J-27 | does: harness protected-blocks + 304 webcache + dispatch amendments | state: dark (protect_blocks.py + webfetch_cache.py absent) | key_files: (planned) .claude/tools/protect_blocks.py | respect: -
id: J-28 | (see WIRED section above)
id: J-29 | does: usage window dashboard surface + live mid-stream meter + canvas badge | state: dark/uncertain (UsageWindow.svelte) — settle: ls crates/dashboard-v2/**/UsageWindow.svelte | key_files: (planned) dashboard-v2 UsageWindow.svelte | respect: consumes J-28
id: J-30 | does: provider BYOK key store + custom endpoints + verified configure + turn lock | state: dark (key_store.rs/custom_backends.rs absent) | key_files: (planned) crates/providers/src/key_store.rs | respect: B-37 factory
id: J-31 | does: canvas system-composition mode (nodes=bound units, edges=verified wiring claims) | state: dark (system_canvas.rs absent; canvas_dispatcher/grammar present = base canvas only) | key_files: (planned) system_canvas.rs | respect: shipped H-Track-2 canvas
id: J-32 | does: elai front-door router (chat=ELAI, total-function lane routing) | state: dark (front_door.rs ABSENT, 0 elai.map refs; confirmed by J-111) | residual: the whole router; BLOCKS J-35/J-111 | key_files: (planned) crates/orchestrator/src/front_door.rs | respect: -
id: J-33 | does: MinerU document-parsing lane (wire the dark ParserBackend::MinerU) | state: dark (knowledge doc_parser absent; pdf_ingest.py absent) — NOTE the /pdf skill exists via MinerU separately | key_files: (planned) crates/knowledge DocParserClient | respect: -
id: J-34 | does: empirical context calibration (measured degradation curve replaces 0.85 folklore) | state: dark (context_calibration.rs ABSENT) | residual: the whole bench + profile fields; BLOCKS J-39/J-89/J-90 enforcement, K-17 | key_files: (planned) context_calibration.rs | respect: -
id: J-35 | does: intent-control layer (typed IntentSpec + premise-check + route + decision-card) | state: dark (intent_control.rs absent; intent_classifier.rs present is different) | residual: whole build; depends J-32 | key_files: (planned) intent_control.rs | respect: -
id: J-36 | does: freeze-the-head local prefix-KV reuse on the streaming path | state: uncertain (sidecar python) — settle: grep server.py:2480 streaming lease | key_files: python/elai_inference/server.py | respect: -
id: J-38 | does: agentic retrieval loop (mid-task hybrid_search + SpIDER fusion + sufficiency gate) | state: dark (hybrid_search tool + RetrievalLoopEvent uncertain; knowledge/pipeline.rs present) — settle: grep RetrievalLoopEvent | key_files: crates/knowledge/src/pipeline.rs | respect: J-74/J-108 extend
id: J-39 | does: decomposition node-admission gate (bound-and-escalate sizing vs measured ceiling) | state: dark (admission check absent; phase_sizing_splitter.rs present) — depends J-34 | key_files: crates/orchestrator/src/phase_sizing_splitter.rs | respect: replaces the fixed budget constant
id: J-40 | does: research-modes extractive compression (harness fetches+compresses, model ranks) | state: dark/uncertain (QueryAwareExtractor) — settle: grep web_research_orchestrator | key_files: crates/orchestrator/src/web_research_orchestrator.rs | respect: -
id: J-41 | does: cheap grounding stack (extractive -> small NLI -> closed-set citation) | state: dark/partial (F-12 OnnxNliJudge + H-53 present; ClosedSet decode uncertain) — settle: grep DecodingMode::ClosedSet | key_files: crates/orchestrator/src/citation_prebind.rs | respect: -
id: J-42 | does: within-step distill-reset reasoning (InftyThink sawtooth) | state: dark (checkpoint_and_reset absent; grc.rs present) — settle: grep checkpoint_and_reset | key_files: crates/orchestrator/src/grc.rs | respect: J-108 extends
id: J-43 | does: doctrine-propagation keystone (determinism-doctrine.md + doctrine_floor_check.py) | state: dark (.claude/tools/doctrine_floor_check.py absent) | key_files: (planned) .claude/tools/doctrine_floor_check.py | respect: -
id: J-44 | does: reasoning-method artifact-shape gate + substance audit | state: dark (reasoning_artifact_shape_check.py absent) | key_files: (planned) | respect: -
id: J-45 | does: cross-family advisory calibration (kappa>0.6 + escalation) | state: dark (calibration corpus + dispatch absent) | key_files: (planned) | respect: -
id: J-46 | does: doctrine-debt-rate measurement | state: dark (doctrine_debt_rate.py absent) | key_files: (planned) | respect: -
id: J-47 | does: program falsifier + per-axis discipline (A/B/C experiment) | state: dark | key_files: (planned) | respect: -
id: J-48 | does: local serving tier router (warm/cold/spill outer policy) | state: dark (local_serving.rs ABSENT) | key_files: (planned) local_serving.rs | respect: -
id: J-49 | does: multi-seat admission + per-seat quota/fair-share/spend isolation | state: dark (seat_registry.rs ABSENT) | residual: whole build; depends J-48 | key_files: (planned) seat_registry.rs | respect: J-59 memory isolation, K-24 billing
id: J-50 | does: recovery-trajectory SFT exhaust (failed-then-recovered chains) | state: dark (extends I-30 which is itself dark; no training crate) | key_files: (planned) crates/cli/src/sft_exhaust.rs | respect: I-30
id: J-51 | does: tool-call adherence floor + role gate (per-model semantic adherence) | state: dark (toolcall_adherence.rs ABSENT) | key_files: (planned) toolcall_adherence.rs | respect: -
id: J-52 | does: return-time + per-edit enforcement hooks (SubagentStop evidence gate + LSP block-feed + path-rule inject) | state: dark (subagent_evidence_gate.rs/lsp_daemon_client.rs/path_rule_inject.rs absent; crates/tools only has prompt_hardening.rs) | key_files: (planned) | respect: J-62 folds in here
id: J-54 | does: free-API rotation pool (quota/429/rotation/heal, per-role assignable) | state: dark (providers/runtime/free_pool absent) | key_files: (planned) crates/providers/src/runtime/free_pool/ | respect: K-24 margin lane
id: J-55 | does: model-strength modality router (CodeAct vs constrained tool-call per model) | state: dark (modality_router.rs ABSENT, 0 elai.map refs) | residual: KEYSTONE of win-both; whole build | key_files: (planned) modality_router.rs | respect: E-35 CodeAct
id: J-56 | does: living eval corpus + failure-ingest + CI governance (activate E-54/G-6) | state: dark (eval_corpus.rs absent) | key_files: (planned) crates/orchestrator/src/eval_corpus.rs | respect: L-20 extends
id: J-57 | does: eval-gated prompt-template promotion + rollback (Test/Staging/Prod SM) | state: dark (prompt_promotion.rs ABSENT) | key_files: (planned) prompt_promotion.rs | respect: -
id: J-58 | does: pre-execution sprint-contract negotiation loop | state: dark (NegotiationLoop absent; sprint_contract.rs present as I-21) — settle: grep Renegotiated | key_files: crates/orchestrator/src/sprint_contract.rs | respect: extends I-21
id: J-59 | does: per-seat memory + trust isolation (namespacing + auditor firewall per-seat) | state: dark (seat memory key absent; depends J-49) | key_files: (planned) crates/memory | respect: extends E-5/H-31/H-32
id: J-60 | does: layered observability canvas (PROCESS altitude, journal-derived FSM+agents) | state: dark (process_canvas.rs ABSENT) | key_files: (planned) process_canvas.rs | respect: J-31 system layer
id: J-61 | does: canvas redirect loop (STEER half, journaled intents) | state: dark (canvas_redirect.rs ABSENT) | key_files: (planned) canvas_redirect.rs | respect: J-60
id: J-62 | does: environment-constraint skill cards + pre-write mandatory-read gate | state: dark (dashboard-v2-env skill + path_rule_inject absent) | key_files: (planned) .claude/skills/dashboard-v2-env/ | respect: reuses J-52 PathRuleInjector (do not re-implement)
id: J-63 | does: local-lane Claudeception (keyless local broker + `elai artifact run`) | state: dark (generated_app_broker.rs/artifact_run_cmd.rs absent) | key_files: (planned) generated_app_broker.rs | respect: security review mandatory; L-14 consumes
id: J-64 | does: ACP transport for agent bridges (additive to direct-CLI) | state: dark (providers/runtime/acp_transport.rs absent) | key_files: (planned) acp_transport.rs | respect: do NOT remove direct-CLI bridges
id: J-65 | does: power-of-ten generated-code gate (frozen auditor checker) | state: dark (.claude/tools/power_of_ten_check.py absent) | key_files: (planned) power_of_ten_check.py | respect: -
id: J-66 | does: SSB temporal-monitor engine (ptLTL online + future-time obligations) | state: dark (ssb_monitor/ dir absent) | key_files: (planned) crates/orchestrator/src/ssb_monitor/ | respect: extends I-35; depends J-69
id: J-67 | does: multi-mode gate actions (Block -> Accept/Suppress/Insert/Block) | state: dark (intervention/gate_action.rs absent; J-12 itself dark) | key_files: (planned) | respect: extends J-12
id: J-68 | does: formal verification of ELAI's own core (Kani + SPARK contracts + TLA+) | state: dark (kani harnesses/ElaiFSM.tla absent) — settle: grep kani::proof | key_files: (planned) | respect: -
id: J-69 | does: SSB log-integrity hash chain + watermark monotonicity (consumer legs of I-18) | state: dark/partial (scan_chain/check_watermark uncertain; archive_chain.rs present) — settle: grep scan_chain | key_files: crates/orchestrator/src/archive_chain.rs | respect: extends I-18+J-22
id: J-70 | does: gate-manifest traceability schema + IV&V independence | state: dark (gate-manifest +4 columns + reserved auditor pool absent) — settle: read .claude/gate-manifest.txt columns | key_files: .claude/gate-manifest.txt | respect: -
id: J-71 | does: Cedar policy-as-code SPIKE (time-boxed, may be declined) | state: dark/spike (workflow/reasoning/J-71-cedar-spike.md) | key_files: (planned) | respect: read-only, no wiring
id: J-72 | does: in-domain retriever adaptation (offline fine-tune ELAI's own encoder) | state: dark (adapted encoder + scoring head absent) | key_files: crates/knowledge/src/vector/ | respect: folds into J-18
id: J-73 | does: retrieval-abstention gate (should_retrieve threshold) | state: dark (should_retrieve gate absent; abstention_score.rs present as substrate) — settle: grep should_retrieve | key_files: crates/knowledge/src/pipeline.rs | respect: -
id: J-74 | does: graph-as-retrieval channel + mid-task ego-graph action (wire shipped dataflow edges to RRF) | state: dark (dataflow-RRF fold absent; symbol_graph present) | key_files: crates/knowledge/src/symbol_graph/ | respect: folds into E-40/J-38
id: J-75 | does: RRF ablation harness (measure-then-simplify, prunes nothing) | state: dark (bench rrf-ablate absent) | key_files: (planned) | respect: no removal without user confirm
id: J-76 | does: context-rot calibration + local _compact_history recipe (extends J-34/H-50) | state: dark (depends J-34 which is dark) | key_files: crates/orchestrator/src/context_policy.rs | respect: cloud lane stays masking (HD-2)
id: J-77 | does: perception faithfulness via the trust spine (route J-1/J-8 through H-53/G-1/G-2) | state: dark (citation-prebind over regions + transform ledger absent; depends J-1/J-8) | key_files: crates/orchestrator/src/perception/ | respect: J-1/J-8
id: J-78 | does: context-access spec (declared per-dispatch access_list) | state: dark (access_spec struct absent) | key_files: (planned) | respect: extends I-5
id: J-79 | does: signal-triggered grill interlace (detector->resolver wire) | state: dark (rung selector + return-path consumers absent) | key_files: crates/orchestrator/src/autonomous_gate.rs | respect: grill NEVER auto-run (consent)
id: J-80 | does: worktree pool lifecycle (persistent projection + lease + heal, extends I-2) | state: dark/partial (branch_workspace.rs barrier present; pool-lifecycle layer uncertain; disposition: refine-first, plan_gate FAIL 7/10, undesigned-at-seam) — settle: grep Leased/Reused/Healed | key_files: crates/orchestrator/src/sandbox/branch_workspace.rs | respect: KEEP barrier_merge/escape_check (GD-1 P0)
id: J-81 | does: autonomous supervision hardening (wedge detection + wake-queue + mid-loop cost cap) | state: dark (supervision_wedge.py/wake_queue.py absent; supervision.rs present) — settle: grep E-39 mid-loop caller | key_files: crates/orchestrator/src/supervision.rs | respect: -
id: J-82 | does: deterministic token-efficient emission (TOON/dict-encode/protected-span) | state: dark (token_emit/ dir absent) | key_files: (planned) crates/orchestrator/src/token_emit/ | respect: K-12 must-preserve guard
id: J-87 | does: journal-completeness substrate (mediate/observe/reconcile; the ONE net-new LFG build) | state: dark (JournalCompletenessEvent + eBPF observer absent) | residual: XL green-field; Linux-only observer | key_files: (planned) | respect: pre-gated by K-21
id: J-88 | does: runtime-observed DKG edges (E2) + machine-derived-first audit lane (E3) | state: dark (observed_calls edge kind absent) | key_files: crates/knowledge/src/dkg/ | respect: -
id: J-89 | does: objective + plan-DAG prevention gate (spec bundle + cross-family jury + wiring at plan-DAG time) | state: dark (spec bundle + plan-DAG symmetry absent; wiring_contract_check.py present as substrate) — depends J-34 | key_files: .claude/tools/wiring_contract_check.py | respect: -
id: J-90 | does: cross-lang + dynamic blast cone (L5, USER #1) + usage-weighted reachability + cant-lie x mutation | state: dark (cone composition absent; depends J-88/J-16/J-56/J-34) | key_files: (planned) | respect: fuses into J-16 cone
id: J-91 | does: re-executable RRA (VCR cassette + pinned env; the wedge technical leg) | state: dark (cassette/pinned-env absent; extends J-14) | key_files: crates/orchestrator/src/rra.rs | respect: J-14; K-20 banks the claim
id: J-92 | does: orchestrator-FSM dispatch ordering (promote dispatch to FSM substates + capability tokens) | state: dark (FsmSubstates/EvidenceCheck for dispatch absent; fsm.rs present) | key_files: crates/orchestrator/src/fsm.rs | respect: I-1/J-66/J-68
id: J-93 | does: seven Tier-2 composition wiring fills (MAST/compensation/taint-thru-memory/GRC-citation/drift/rule-labels/plan-derive) | state: dark (7 vertical slices, depends J-87) | key_files: (planned, various) | respect: -
id: J-94 | does: Tier-3 honesty fixes + THIN measure-first (mostly DEFERRED) | state: dark/deferred | key_files: (planned) | respect: discipline is what we DEFER
id: J-95 | does: context-activation wires (wire adaptive_reduce + pristine_path_filter + BeliefState condense) | state: dark/partial (the 3 dark call-site wires; adaptive_reduce + pristine_path_filter.rs exist but wiring uncertain) — settle: grep adaptive_reduce callers | key_files: crates/orchestrator/src/context_budget.rs, pristine_path_filter.rs | respect: corrects stale I-5/I-11/H-50 PLANNED labels (BUILT)
id: J-96 | (see WIRED/partial section above)
id: J-97 | does: VCG gated retrieval-time reuse + episodic->skill->rule ladder (RISKY, default-OFF, LAST) | state: dark (ELAI_CASE_REUSE_ENABLE path absent) | key_files: (planned) | respect: K-19 SF-1..SF-4; NEVER into auditor; depends J-96
id: J-98 | does: canvas MEMORY/CASE altitude (third canvas altitude) | state: dark (memory CanvasMode absent) | key_files: (planned) | respect: depends J-96/J-97/J-61/J-59
id: J-99 | does: time-as-context plane (timestamp slot + agent-time + resume header) | state: dark (TimeContext slot absent; clock.rs present) — settle: grep TimeContext | key_files: crates/orchestrator/src/context_budget.rs | respect: K-14 TZ-1..TZ-4
id: J-100 | does: time-as-control plane (async-tool loop + watchdog promotion) | state: dark/partial (background_task.rs present; T-4 loop + watchdog promotion uncertain) — settle: grep WatchdogTimeout | key_files: crates/orchestrator/src/background_task.rs | respect: K-14 TZ-4; I-53 cron declined
id: J-101 | does: transformation-surviving taint binding + cross-process taint + E-37 perimeter | state: dark/partial (taint_flow_ssb_adapter.rs present; WI-1 taint-window + G-7 session gate hardening uncertain) — settle: grep production_agent_rpc.rs:334 trifecta | key_files: crates/safety, crates/orchestrator/src/taint_flow_ssb_adapter.rs | respect: extends E-37/G-1/G-7
id: J-102 | does: config-ingress admission gate (blake3 load-time manifest over .md instruction surface) | state: dark (ConfigAdmissionEvent + admission manifest absent) | key_files: crates/core/src/config.rs (:205 seam) | respect: extends E-43/E-24
id: J-103 | does: read-access ladder + Read-tool interceptor + savings meter | state: dark (read_ladder.rs ABSENT) | key_files: (planned) read_ladder.rs | respect: widens I-46 to Read tool
id: J-104 | does: live-index overlay-before-base read + idle checkpoint | state: dark (index_overlay.rs ABSENT) | key_files: (planned) index_overlay.rs | respect: extends B-31
id: J-105 | does: counterfactual memory-ROI attribution + curation (extends J-21) | state: dark (roi_attribution.rs absent) | key_files: (planned) crates/memory/src/roi_attribution.rs | respect: NEW file extending J-21, never amend; H-31 firewall
id: J-106 | does: output-side meta-commentary enforcement (op-emission roles only, HD-3-bounded) | state: dark (output_commentary_gate.rs ABSENT) | key_files: (planned) output_commentary_gate.rs | respect: reasoning roles UNTOUCHED (HD-3); reuses I-32 classification
id: J-107 | does: `elai bench longcot` recursive-scaffold-vs-flat measurement arm (GATES J-108) | state: dark (benchmarks/longcot_runner.rs absent) | key_files: (planned) benchmarks/longcot_runner.rs | respect: reuse K-15/I-27 runner, do not fork
id: J-108 | does: multi-hop DEPTH operator (HippoRAG-PPR primary + constrained RetrievalNavigator fallback) | state: dark (search/navigator.rs ABSENT; hippo_rag.rs not found in knowledge/src — the "shipped HippoRAG" premise is unverified) | residual: whole build; depends J-107 | key_files: (planned) crates/orchestrator/src/search/navigator.rs | respect: extends J-42/J-86/J-38
id: J-109 | does: learned-artifact admission hardening (span-dedup + counterfactual replay fitness at write seams) | state: dark (admission_pipeline.rs ABSENT) | key_files: (planned) admission_pipeline.rs | respect: fronts E-5/I-12/J-97; SF-1..SF-4
id: J-110 | does: abstention timeliness (SPL) + abstention-warranted eval supply | state: dark (benchmarks/abstention_runner.rs absent) | key_files: (planned) benchmarks/abstention_runner.rs | respect: extends H-39/J-56/L-20; reuse runner family
id: J-111 | does: voice ingress + spoken readback-confirm (second transducer into J-32) | state: dark (voice/ dir absent; HARD-depends J-32 which is dark) | key_files: (planned) crates/orchestrator/src/voice/ingress.rs | respect: J-32 router; J-112 speaker
id: J-112 | does: spoken render + narration dispatcher (voice output plane) | state: dark (voice/spoken_render.rs absent; notify_dispatch.rs present as seam) | key_files: crates/orchestrator/src/notify_dispatch.rs | respect: ADD target not dispatcher (GD-1); ships before J-111

------------------------------- K-plan (27 units) -------------------------------
Authored 2026-06-12..2026-07-10, the NEWEST-with-L, a SEQUENCING/COMPOSITION/DISTRIBUTION letter. Most K
units are doc/lint/rule ("PROVE/PACKAGE/DISCIPLINE") not code. GROUNDED: the 5 real builds (K-2/K-3/K-4/
K-9/K-18) + K-1 dossier/lint SHIPPED; the DISCIPLINE rule files (K-14/16/19/21/22/25) are DARK (absent
from .claude/rules/ and conditional-routing does not load them); the two net-new capability builds
(K-23 workspace, K-24 billing) are DARK. K is a primary SUPERSESSION SOURCE (K-19/H-43 vs live DGM code).

id: K-1 | does: problem-quantification dossier + claims-policy lint (KD-1 number-before-claim) | state: wired (docs/strategy/ELAI-CLAIMS-POLICY.md + ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md + .claude/tools/claims_policy_lint.py all present) | residual: dossier rows fill as A-1..A-3 numbers land | key_files: docs/strategy/ELAI-CLAIMS-POLICY.md, .claude/tools/claims_policy_lint.py | respect: K-13/17/18/20 extend the lint recogniser
id: K-2 | does: routing-economics bench (hybrid vs cloud-only, $/verified-task) | state: wired (benchmarks/routing_economics_runner.rs + cli bench_routing_economics + commit 2523cf5) | residual: live cost leg gated on J-28 (now present) | key_files: crates/orchestrator/src/benchmarks/routing_economics_runner.rs | respect: -
id: K-3 | does: PR/CI evidence emitter (per-hunk verdicts into GitHub check-runs/comments) | state: wired (github_pr/evidence_emitter.rs + commit ce76e4f "k-3 pr evidence emitter") | residual: - | key_files: crates/orchestrator/src/github_pr/evidence_emitter.rs | respect: K-11 extends to human-viewable evidence
id: K-4 | does: composed gate-tax budget bench (friction vs locked ceiling) | state: wired (benchmarks/gate_tax_runner.rs + gate_tax_audit_leg.rs + cli bench_gate_tax + commit) | residual: - | key_files: crates/orchestrator/src/benchmarks/gate_tax_runner.rs | respect: K-5 fits under its ceiling
id: K-5 | does: risk-tiered gating policy (deterministic GateIntensity table, R8-frozen) | state: partial (gate_intensity.rs present) — settle: grep GateIntensitySelected + gate-manifest for the frozen table hash | residual: verify policy-table hash in frozen manifest | key_files: crates/orchestrator/src/gate_intensity.rs | respect: depends K-4
id: K-6 | does: compliance export (`elai export compliance`, RRA+attestation+mapping doc) | state: dark (no compliance export cmd/doc found; compliance.rs present is different) — settle: grep "export compliance" | key_files: (planned) | respect: composes G-2/G-8/G-9/I-44
id: K-7 | does: appsec/insurance positioning brief (pure prose) | state: dark (no appsec/insurance brief in docs/strategy) | key_files: (planned) docs/strategy/ | respect: K-1 dossier numbers
id: K-8 | does: absorbable-unit audit + provider-check pre-gate (feature-map tags + rule) | state: dark/partial (rule file absent; feature-map tags uncertain) — settle: grep ABSORBED/AT-RISK/DURABLE in ELAI-FEATURE-MAP.md | key_files: (planned) .claude/rules/ | respect: K-25 makes it continuous
id: K-9 | does: quarterly model-refresh cadence (bench wrapper over J-9 catalog) | state: wired (benchmarks/model_refresh_runner.rs + cli bench_model_refresh) | residual: cadence checklist | key_files: crates/orchestrator/src/benchmarks/model_refresh_runner.rs | respect: depends J-9
id: K-11 | does: human-viewable evidence render (## Evidence section into PR + cockpit; extends K-3) | state: partial (evidence_emitter.rs present; --emit-evidence commit mentions per-hunk evidence; evidence_render.rs specifically absent) — settle: grep EvidenceArtifactRendered | key_files: crates/orchestrator/src/github_pr/evidence_emitter.rs | respect: extends K-3
id: K-12 | does: losslessness enforcement (compact.rs WARN->FAIL) + handoff-faithfulness gate (critical_atoms) | state: dark/partial (compact.rs + belief_state.rs present; handoff_faithfulness.rs + CriticalAtom uncertain) — settle: grep HandoffFaithfulnessGated/critical_atoms | key_files: crates/orchestrator/src/compact.rs, belief_state.rs | respect: corrects stale I-5/I-11/H-50 labels; L-18 re-points it
id: K-13 | does: OWASP control-mapping dossier + injection-defense posture proof | state: dark/partial (dossier row maybe in ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md; ELAI-OWASP-CONTROL-MAPPING.md absent) — settle: grep OWASP in docs/strategy | key_files: (planned) docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md | respect: pairs with J-101/J-102
id: K-14 | does: time-awareness determinism doctrine (.claude/rules/time-determinism.md) + differentiator | state: dark (time-determinism.md absent from .claude/rules/) | key_files: (planned) .claude/rules/time-determinism.md | respect: pairs with J-99/J-100
id: K-15 | does: reasoning-budget bench + receipt-grounded-reasoning differentiator | state: dark (bench_reasoning_budget absent; reasoning_budget.rs is I-26) | key_files: (planned) benchmarks/reasoning_budget_runner.rs | respect: BENCH-PENDING on J-83/J-85; compose I-27 runner
id: K-16 | does: reasoning-receipts discipline doctrine (.claude/rules/reasoning-receipts.md) | state: dark (reasoning-receipts.md absent) | key_files: (planned) .claude/rules/reasoning-receipts.md | respect: pre-gates J-83..J-86
id: K-17 | does: context-management measured+regime-aware differentiator (doc+lint) | state: dark/partial (dossier row maybe; no code) — settle: grep context-management in dossier | key_files: docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md | respect: BENCH-PENDING on J-34
id: K-18 | does: case-reuse economics bench + per-customer-compounding differentiator | state: wired (benchmarks/case_reuse_runner.rs + case_reuse_fixtures + cli bench_case_reuse) | residual: reuse-ON economics arm BENCH-PENDING on VCG J-track | key_files: crates/orchestrator/src/benchmarks/case_reuse_runner.rs | respect: K-19 SF-gates; J-96/J-97
id: K-18a | does: PS-5 observe-arm test run (recurrence-hit-rate) | state: partial/wired (2 commits reference k-18a) | residual: - | key_files: crates/orchestrator/src/benchmarks/case_reuse_runner.rs | respect: -
id: K-19 | does: self-improvement/case-reuse fitness doctrine (.claude/rules/self-improvement-fitness.md, SF-1..SF-4) | state: dark (self-improvement-fitness.md absent) | key_files: (planned) .claude/rules/self-improvement-fitness.md | respect: SUPERSEDES live DGM GAP-1/GAP-2 via H-43/H-44; pre-gates VCG J-track
id: K-20 | does: re-executable-RRA differentiator (doc+lint; THE WEDGE) | state: dark/partial (dossier row maybe; no code) | key_files: docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md | respect: BENCH-PENDING on J-91
id: K-21 | does: journal-completeness discipline doctrine (.claude/rules/journal-completeness.md) | state: dark (journal-completeness.md absent) | key_files: (planned) .claude/rules/journal-completeness.md | respect: pre-gates J-87
id: K-22 | does: doctrine-enforcement-coverage ledger (.claude/tools/doctrine_coverage.py + rule) | state: dark (doctrine_coverage.py absent) | key_files: (planned) .claude/tools/doctrine_coverage.py | respect: -
id: K-23 | does: multi-repo/workspace memory (cross-REPO layer; net-new capability, 6 slices) | state: dark (crates/orchestrator/src/workspace/ dir ABSENT) | residual: whole build; ELAI_WORKSPACE_ENABLE | key_files: (planned) crates/orchestrator/src/workspace/ | respect: depends J-18/J-80/J-2; paths.rs unchanged
id: K-24 | does: credit-based seats on OpenRouter rail (billing; net-new capability, 6 slices) | state: dark (crates/orchestrator/src/billing/ dir ABSENT) | residual: whole build; ELAI_BILLING_ENABLE | key_files: (planned) crates/orchestrator/src/billing/ | respect: depends J-28/J-49/J-54; never "unlimited"
id: K-25 | does: continuous absorption watchdog (provider-surface snapshot + absorption_watch.py + rule) | state: dark (.claude/tools/absorption_watch.py absent) | key_files: (planned) .claude/tools/absorption_watch.py | respect: extends K-8
id: K-26 | does: codex-burn campaign (5-day ops+build run to spend expiring credit pools) | state: campaign/ops (not a code unit; wargame plan_gate PASS 10/10) — moves: finish J-28, K-2 build, gate-tax lock | key_files: - | respect: deadline was 2026-07-15
id: K-27 | does: gpt-5.5 SOL isolated executor lane | state: uncertain (referenced in I-23 disposition as a set-aside driver) — settle: read K-27 plan file | key_files: (planned) | respect: -

------------------------------- L-plan (24 units) -------------------------------
Authored 2026-06-27..2026-07-11, the NEWEST letter, points the trust spine OUTWARD (verify ANY model's
CLAIMS). GROUNDED: ALMOST ENTIRELY DARK. The L-T1 claim/ dir is ABSENT (L-1..L-10 all dark); reason/
dir ABSENT (L-9/L-22/L-23/L-24 dark); context_projection.rs ABSENT (L-16..L-19 dark); verification_
intensity.rs ABSENT (L-21). The ONLY present substrate is anti_spiral_extended.rs (L-23 orbit detector,
partial) + claims_policy/maintainability checkers (L-11 depends on solid_proxy_check.py which is ABSENT).
L is ~85% ACTIVATION-of-shipped-substrate on paper but 0% activated in code. L-15's binding revision
supersedes the L-0 W2 ordering.

id: L-1 | does: typed claim extraction substrate (ClaimSet, off-agent) | state: dark (claim/ dir ABSENT; disposition: greenfield BUILD, production seam UNDESIGNED at production_agent_rpc) | residual: whole build + seam design | key_files: (planned) crates/orchestrator/src/claim/ | respect: foundation for L-2..L-8
id: L-2 | does: unified ClaimRisk score + convergence fuse (>=2 predictors) | state: dark (claim/risk.rs absent) | key_files: (planned) crates/orchestrator/src/claim/risk.rs | respect: generalizes G-7 claim_score
id: L-3 | does: evidence round-trip predictor P1 (activate D-10/D-11/H-53 over claims) | state: dark (claim/roundtrip.rs absent) | key_files: (planned) claim/roundtrip.rs | respect: activates D-10/D-11/H-53
id: L-4 | does: semantic-entropy inter-seed predictor P3 (NLI meaning-cluster) | state: dark (claim/semantic_entropy.rs absent) | key_files: (planned) claim/semantic_entropy.rs | respect: F-12 DeterministicNliJudge
id: L-5 | does: counterfactual do-test on model claims (interventional; HG-1) | state: dark (claim/counterfactual.rs absent) | key_files: (planned) claim/counterfactual.rs | respect: drives F-4/C-28 on the claim, do not re-implement
id: L-6 | does: risk-as-curiosity targeted-effort router | state: dark (claim/curiosity_router.rs absent) | key_files: (planned) claim/curiosity_router.rs | respect: J-38/J-103/I-7
id: L-7 | does: hallucination policy gate + abstention integration | state: dark (claim/policy_gate.rs absent) | key_files: (planned) claim/policy_gate.rs | respect: D-16/I-8/B-22 AutonomousGate
id: L-8 | does: working-memory hygiene / prune spurious connections | state: dark (claim/prune.rs absent) | key_files: (planned) claim/prune.rs | respect: F-5/J-21/I-5
id: L-9 | does: research/math deep-think grounding profile (/reason --profile research) | state: dark (reason/ dir absent) | key_files: (planned) crates/orchestrator/src/reason/ | respect: B-39/D-11/E-53/B-42
id: L-10 | does: CGG hallucination-delta bench (whole-plan done-gate) | state: dark (benchmarks/hallucination_delta_runner.rs absent) | key_files: (planned) benchmarks/hallucination_delta_runner.rs | respect: G-6/I-38/I-27 runner shape
id: L-11 | does: output-quality enforcement system (SOLID 4-seam gate + solid_proxy_check.py) | state: dark (.claude/tools/solid_proxy_check.py ABSENT; maintainability_check.py present as sibling) | key_files: (planned) .claude/tools/solid_proxy_check.py | respect: auditor-owned + frozen (OQ-1); C-31/C-32/maintainability_check
id: L-12 | does: skill front-door + self-improving skills (light the dark E-43 door) | state: dark/partial (skill_registry/ + skill_trust.rs present; `elai skill add/improve` + curator wiring uncertain) — settle: grep log_skill_invocation callers | key_files: crates/orchestrator/src/skill_registry/ | respect: E-43/F-6/I-12
id: L-13 | does: user-authorable rules & hooks surface (`elai rule add`) | state: dark (rule CLI + user-rule->hook registration absent; rule_loader.rs present) — settle: grep UserRuleRegistered | key_files: crates/orchestrator/src/rule_loader.rs | respect: rules-engine 3F, cannot weaken mutable=0
id: L-14 | does: one-shot verified builder front-door (`elai build "<idea>"`) | state: dark (elai build verb absent) | key_files: (planned) | respect: PIPELINE FACTORY/E-12/I-40/L-11; depends L-11
id: L-15 | does: receipt-minting repair controller + evidence-first reorder | state: dark (RepairController absent) | supersedes: the L-0 W2 ordering (MEASURED, L-15 R2/R4) | key_files: (planned) | respect: L-3/L-5 land before L-4; D-13 alongside L-4
id: L-16 | does: immutable transcript + pure fold-view context core (KEYSTONE) | state: dark (context_projection.rs ABSENT) | supersedes: REPLACES conversation.rs::compact_session | residual: the projection core + freeze-cause SM; highest structural-revert risk | key_files: (planned) crates/orchestrator/src/context_projection.rs | respect: H-49 log/CAS, 3E budget, H-50 policy, K-12 atoms
id: L-17 | does: freeze-epoch controller (prefix-invariance as recompute-cause SM) | state: dark (FreezeState absent; depends L-16) | key_files: (planned) context_projection.rs | respect: I-10 cache_economics IS its concrete form
id: L-18 | does: projection corollaries (losslessness + time-travel collapse) | state: dark (depends L-16) | key_files: (planned) context_projection.rs + compact.rs | respect: re-points K-12 check_lossless at the view; H-49 undo-tree
id: L-19 | does: ambient page-in budget ladder (concrete H-6 read-back policy) | state: dark (depends L-16/L-17) | key_files: (planned) context_projection.rs | respect: H-6/I-5
id: L-20 | does: commit-mined eval supply + competitor-arm paired harness | state: dark (benchmarks/commit_miner + competitor_arm_runner absent; cli_common spawn present as substrate) | key_files: (planned) benchmarks/competitor_arm_runner.rs | respect: J-56 corpus, agent-CLI bridges (do not re-spawn)
id: L-21 | does: weak-lane verification-intensity policy (table-driven per model tier x role) | state: dark (verification_intensity.rs ABSENT) | key_files: (planned) crates/orchestrator/src/verification_intensity.rs | respect: J-51/J-55/C-32; reviewer advisory-only (B7 decline stands)
id: L-22 | does: claimant-authored falsifiers + demarcation + Verbalized-Sampling propose | state: dark (claim/falsifier.rs + verbalized_sampling.rs absent) | supersedes: I-4 N-resample propose leg (partial) | key_files: (planned) claim/falsifier.rs | respect: L-1/L-2; I-4 propose; J-83 GRC carrier
id: L-23 | does: /reason --profile explore (conjecture-criticism map, local-7B lane, KEYSTONE) | state: dark/partial (anti_spiral_extended.rs PRESENT — the Orbiting detector may be there; reason/explore.rs ABSENT) — settle: grep StagnationPattern::Orbiting | key_files: crates/orchestrator/src/anti_spiral_extended.rs | respect: L-22; anti-spiral; D-13; Dung adjudicator REJECTED
id: L-24 | does: conjecture-memory middle tier (explore lane only) + 3 collab riders | state: dark (reason/conjecture_store.rs absent) | key_files: (planned) crates/orchestrator/src/reason/conjecture_store.rs | respect: J-96..J-98 UNCHANGED (boundary only); L-22/L-23; N-16 evaluator-edge

===============================================================================
DRAIN-RELEVANT NOTES
===============================================================================
1. REVERSION-RISK HOTSPOTS (a drain finishing an OLD unit must not revert these newer owners):
   - L-16 owns context compaction (replaces compact_session). Any drain of E-25/H-49/H-50 compaction
     work must yield to L-16's projection core if L-16 is being built.
   - J-20 owns the promotion path (supersedes D-5/D-29/H-48). J-21/J-37 own memory validation
     (supersede H-31/H-32). J-83 owns the BeliefState carrier (completes I-11).
   - K-19/H-43/H-44 own the DGM fitness signal (replace the LIVE blake3-config-bytes GAP-1 at
     darwin_admit_loop.rs + keyword-grep GAP-2 at trap_tasks.rs). A drain touching darwin_admit_loop.rs
     must not restore the byte-hash fitness.
   - I-23's CalibrationFacade is the single owner of tau/coverage — do not let a drain re-introduce a
     direct conformal_calibration import (the live bypass at production_agent_rpc.rs:86 is already the bug).
2. DOUBLE-BUILD RISK: K-native units (K-15/K-17/K-18/K-20) PACKAGE J-track benches; a drain must not
   build a parallel bench (GD-1 P0). Same for the many "COMPOSE I-27 bench_convergence runner" units.
3. BLOCKING CHAINS still open: J-34 (dark) blocks J-39/J-76/J-89-enforcement/J-90-enforcement + K-17.
   J-32 (dark) blocks J-35/J-111. I-30 (dark, no training crate) blocks I-31/J-50. J-96 blocks J-97/J-98.
   J-48 blocks J-49 blocks J-59; J-49 blocks K-24.
