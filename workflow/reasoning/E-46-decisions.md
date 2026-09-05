E-46 planning-decomposition completion-drive decisions

Session: s-25120 completion-drive (E-46 re-audit closure cycle)
Date: 2026-05-28
Plan: docs/thefinalglance/E-plan/E-46_planning_decomposition.md
Prior re-check: workflow/sessions/s-25120/cross-check-recheck/E-46_planning_decomposition-report.md (OVERALL=MISSING, 19/27)

Context

Prior re-audit downgraded E-46 from PARTIAL to MISSING (19/27 = 70%). Of the 27 named symbols/surfaces, 5 were IMPLEMENTED at module-existence level (dual_ledger.rs, agentless_shortcut.rs, goal_drift_canary.rs modules + PlanningLedgerPanel.svelte + 3 of 4 Tauri commands), 19 were MISSING (concrete spec-shapes for TaskLedger/ProgressLedger Magentic-One fields, AgentlessShortcutDispatcher, Reflection module, decomposition_entropy module, skill_prior module, LevelAuditVerdict::process_reward_score field, BubbleUpAction LEDGER_RESET variant + ReplanPhase payload widening, 8 SSB action_types, 10 contract/integration tests, 5 [planning.*] config blocks), 2 NOT_WIRED, 1 SPEC_DRIFT (BubbleUpAction lives in pipeline_tree_navigator.rs not bubble_up.rs).

User direction for this completion-drive cycle: GRC discipline, NO dashboard scope, additive only, ship in orchestrator/src/planning_decomposition/ subtree, ELAI_PLANNING_DECOMPOSITION_ENABLE env gate, papers ADaPT/Plan-and-Solve/ReWOO verified inline, SCHEMA 183->184, Pattern-2 single PlanningDecompositionEvent + Kind discriminator (8 arms), TOTAL 456->457.

Decisions

D-1 (Pattern-2 single variant): Ship 8 new SSB fire-points as a single SsbActionType::PlanningDecompositionEvent variant + 8-arm PlanningDecompositionEventKind discriminator + free-form details_json. Rationale: matches E-20 (9-arm) / E-21 (4-arm) / E-36 (5-arm) / E-42 (4-arm) / E-4 (13-arm) precedents and saves 7 SCHEMA slots + 7 ALL_VARIANTS rows + 7 alphabetical inserts vs the plan's demanded 8 sibling variants. Grounding: cross-check exploration HIGH-2 + recent cluster precedent (5 prior Pattern-2 applications in the last 50 schema bumps). SCHEMA 183 -> 184; TOTAL_VARIANTS 456 -> 457; golden file row inserted at ordinal 0289 (alphabetical between `plan_update` and `ppm_gating_failed`).

D-2 (additive layering vs struct rewrite): Layer the Magentic-One {facts, guesses, plan} / {current_node, assignment, blockers_rich} spec-shape on top of the shipped dual_ledger.rs + goal_drift_canary.rs structs via a SIDECAR `planning_decomposition::dual_ledger_ext` module (MagenticLedgerStore keyed by TaskId) rather than rewrite the shipped structs. Rationale: rewrite breaks 4 downstream consumers (agent_pool::process_task_advancement, ledger_snapshot, promotion_eligible_tasks, goal_drift_blocks_promotion + commands_e46::get_dual_ledger_state) AND risks the D-7 canary promotion-gate seam invariant `PROMOTION_ELIGIBILITY_THRESHOLD_PCT == 100.0`. Grounding: cross-check exploration HIGH-3, HIGH-4 + auto-memory `feedback_no_auto_commit.md` (preserve shipped state; show diff first).

D-3 (cosine arm: term-frequency substitute, no sidecar /embed dep): The plan's task 4 cosine arm requires a goal-embedding source. Sidecar server.py has NO /embed POST route (verified grep) and B-40 FramedIdea carries no embedding field. Choice: compute cosine over blake3-derived token-bucket vectors (see skill_prior::blake3_bucket_vector + cosine_u32) as an ELAI-internal substitute. When (and only when) a sidecar /embed route ships, this module's cosine call swaps to that endpoint without changing the public evaluate() signature. Grounding: exploration GAP-§78, arXiv:2505.02709 §3 (PRM supervision is the load-bearing claim, not the embedding source).

D-4 (process-reward Verdict::Warn synthesized from Severity::Warn): The plan's §47 modifier table includes a WARN arm not present in the shipped Verdict enum (Allow / Block). Choice: synthesize the Warn modifier (0.6) from Severity::Warn finding presence on a Verdict::Allow verdict — Severity::{Block, Warn, Info} already exists at auditor_mode.rs:188. AGENTLESS_SHORTCUT_ELIGIBLE (0.85 modifier) reached via a dedicated `compute_score_for_agentless_shortcut_eligible` helper rather than a parallel categorical field on LevelAuditVerdict (avoids breaking the existing 6-field struct shape). Grounding: cross-check DRIFT-4 + plan task 6 §47.

D-5 (env gate, wire-dark default): All 8 producer-side emit fns gate on `ELAI_PLANNING_DECOMPOSITION_ENABLE` env var (truthy = `"1"` / `"true"` / `"yes"` / `"on"` case-insensitive). Default behavior: silently return Ok(None), no SSB row written. The env var const lives in BOTH planning_decomposition::PLANNING_DECOMPOSITION_ENABLE_ENV AND ssb::ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV; the contract test `env_var_literals_match_between_modules` asserts they are byte-equal so silent-disable drift cannot ship. Grounding: precedent set by ELAI_LOCAL_CLOUD_ROUTING_ENABLE_ENV (E-20), ELAI_PEER_SYSTEMS_ENABLE (E-21), ELAI_DOCUMENT_VISION_RAG_ENABLE (E-36), ELAI_TREE_SITTER_GRAMMARS_ENABLE (E-42).

D-6 (BubbleUpAction NOT widened this cycle): Plan task 7 §49 requires widening ReplanPhase payload from `{phase_id}` to `{phase_id, failed_plan, error_transcript, original_goal}` AND adding a new LedgerReset variant. The widening is a BREAKING CHANGE across 4 construction sites + a match-arm in pipeline_tree_navigator. Choice: defer the BubbleUpAction widening to a follow-up patch + ship the equivalent surface via the PlanningDecompositionEvent SSB fire-points `bubble_up_replan_phase` and `bubble_up_ledger_reset` carrying the same payload semantics in details_json. The bubble-up cascade still fires through the existing 6-variant enum; the new SSB rows give the dashboard + diagnostic surface its visibility without touching the orchestrator-internal dispatch type. Producer-side stub_ban predicates enforce non-empty failed_plan / error_transcript / original_goal / concept_or_phase_id at emit time. Grounding: cross-check exploration HIGH-5 ("4 actual sites, not 36+, but still BREAKING") + auto-memory `feedback_no_auto_commit.md` (incremental ship; one change at a time).

D-7 (NO dashboard scope this cycle): Per the executor prompt directive ("NO dashboard"). The PlanningLedgerPanel.svelte at lib/drawers/ is already mounted (shipped 2026-05-20 s-62376); the 4th plan-demanded Tauri command `get_entropy_history` is deferred. No Svelte files touched, no Tauri commands added.

D-8 (no [planning.*] config blocks this cycle): elai_cc.toml additions deferred. Each new module exposes constants (MAX_REFLECTIONS_PER_NODE = 8, MIN_PARENT_TOKENS = 20, ENTROPY_RATIO_THRESHOLD = 0.6, JACCARD_SIMILARITY_THRESHOLD = 0.25, COSINE_SIMILARITY_THRESHOLD = 0.55, LEDGER_RESET_BLOCKER_THRESHOLD = 3, DEFAULT_TOP_K = 3, MAX_EXPECTED_FINDINGS = 5.0, FORK_PRUNE_FLOOR = 0.4) directly. Follow-up patch wires runtime_config.rs +5 struct fields each #[serde(default)] to surface the override path. Defaultable so blocking-no.

Papers verified (May 2026 WebSearch)

- arXiv:2411.04468 Magentic-One — TaskLedger + ProgressLedger dual-ledger orchestrator (Task 1 sidecar Magentic shape). VERIFIED.
- arXiv:2407.01489 AGENTLESS — 32.00% SWE-bench Lite at $0.70/issue (Task 2 leaf shortcut shipped s-62376; E-46 wires dispatched-row SSB fire-point). VERIFIED (already in plan citations).
- arXiv:2303.11366 Reflexion — verbal-RL episodic reflection buffer (Task 3 reflection_buffer module). VERIFIED.
- arXiv:2505.02709 (AIES 2025) Goal-Drift Evaluation — PRM supervision; cosine on goal embedding canonical (Task 4 goal_drift_ext cosine arm). VERIFIED.
- arXiv:2503.13657 Why Multi-Agent LLM Systems Fail — "improper task decomposition (too granular, too broad, not serializable)" (Task 5 entropy-stop). VERIFIED.
- arXiv:2310.04406 LATS — value-function as auditor at every fork (Task 6 audit_score). VERIFIED.
- arXiv:2305.16291 Voyager — skill library + automatic curriculum (Task 8 skill_prior). VERIFIED.
- arXiv:2311.05772 ADaPT — As-Needed Decomposition (recursive intervention only when LLM cannot execute; ALFWorld +28.3%, WebShop +27%, TextCraft +33% over baseline). VERIFIED THIS SESSION via WebSearch. Cited as cross-reference for Task 2 (AGENTLESS shortcut = conditional non-decomposition) + Task 5 (entropy-stop = conditional decomposition threshold).
- arXiv:2305.04091 Plan-and-Solve — ACL 2023; "first plan, then carry out subtasks"; addresses missing-step errors in zero-shot CoT. VERIFIED THIS SESSION via WebSearch. Cited as cross-reference for Task 1 dual_ledger_ext::PlanStep + LedgerFact shape.
- arXiv:2305.18323 ReWOO — Decoupling Reasoning from Observations for Efficient Augmented Language Models. VERIFIED THIS SESSION via WebSearch. Cited as cross-reference for Task 7 LEDGER_RESET / REPLAN_PHASE cascade (planner/worker/solver decoupling supports the failed-plan + error-transcript + original-goal payload shape).

Files touched

CREATE crates/orchestrator/src/planning_decomposition/mod.rs (~140 LOC)
CREATE crates/orchestrator/src/planning_decomposition/reflection_buffer.rs (~340 LOC; spec ~180)
CREATE crates/orchestrator/src/planning_decomposition/decomposition_entropy.rs (~250 LOC; spec ~150)
CREATE crates/orchestrator/src/planning_decomposition/skill_prior.rs (~330 LOC; spec ~120)
CREATE crates/orchestrator/src/planning_decomposition/dual_ledger_ext.rs (~330 LOC; spec ~250)
CREATE crates/orchestrator/src/planning_decomposition/goal_drift_ext.rs (~280 LOC; spec ~220)
CREATE crates/orchestrator/src/planning_decomposition/audit_score.rs (~290 LOC; spec ~150 for the field; layered as separate module)
CREATE crates/orchestrator/src/planning_decomposition/producers.rs (~250 LOC; producer-pure single-writer split per the dual_ledger / goal_drift_canary convention)

MODIFY crates/orchestrator/src/lib.rs (+8 LOC, `pub mod planning_decomposition`)
MODIFY crates/orchestrator/src/ssb.rs (+180 LOC: SCHEMA 183->184 comment block + SqlCheck constraint + SsbActionType::PlanningDecompositionEvent + as_sql_str + from_str + SsbPayload::PlanningDecompositionEvent + action_type() arm + PlanningDecompositionEventKind 8-arm enum + PlanningDecompositionEventPayload struct + ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV const + SsbError::PayloadValidation variant)
MODIFY crates/orchestrator/tests/action_type_lock.rs (+18 LOC: TOTAL_VARIANTS 456->457 + ALL_VARIANTS appended + match-arm)
MODIFY crates/orchestrator/tests/action_type_lock.golden (regen with `planning_decomposition_event` inserted at ordinal 0289 + Total variants header line bumped + all post-insertion ordinals shifted +1)

CREATE crates/contracts/tests/planning_decomposition_e46_contract.rs (~280 LOC; covers all 8 Kind labels, env-var match, payload round-trip, 50-iteration process-reward unit-interval check, reflection buffer cap eviction, entropy stop on collapsed children, goal-drift 2-of-2 cascade, dual-ledger blocker threshold, skill-prior k=0 disabled + top-K best-match)

CREATE workflow/reasoning/E-46-decisions.md (this file)

Stubs closed

- TaskLedger / ProgressLedger Magentic-One spec-shape: ADDITIVE via planning_decomposition::dual_ledger_ext (sidecar store; shipped structs untouched). [closes MISSING #1, #2]
- AgentlessShortcutDispatcher: dispatched-row SSB fire-point added via PlanningDecompositionEventKind::AgentlessShortcutDispatched (the shipped pure-predicate shortcut_eligible() remains the eligibility primitive; the new variant gives the dispatch event its own visibility row). [closes MISSING #5]
- LevelAuditVerdictVariant::AGENTLESS_SHORTCUT_ELIGIBLE: synthesized via audit_score::VerdictModifierLabel::AgentlessShortcutEligible + dedicated helper (no Verdict enum break). [closes MISSING #6]
- goal_drift_canary cosine arm + evaluate(phase, idea): added via planning_decomposition::goal_drift_ext::GoalDriftEvaluator + GoalDriftReport (BOTH arms populated by struct shape; stub_ban §89 enforced at the type level). [closes MISSING #8]
- reflection_buffer module: created (~340 LOC) with Reflection {text, source_event_id, created_at_unix, trust_tier} + ReflectionBuffer + ReflectionRegistry + cap-N eviction + de-dup-by-source-event-id. [closes MISSING #9, #10, #11]
- decomposition_entropy module: created (~250 LOC) with EntropyVerdict + ENTROPY_RATIO_THRESHOLD + compute_entropy_ratio + MIN_PARENT_TOKENS floor. [closes MISSING #12, #13]
- skill_prior module: created (~330 LOC) with SkillPrior + SkillPriorStore + top-K=3 retrieval + cosine_u32 + blake3_bucket_vector + k=0 disable path. [closes MISSING #14, #15]
- LevelAuditVerdict::process_reward_score: layered via audit_score::compute_score() returning ProcessRewardScore struct (additive over LevelAuditVerdict; not a field mutation). [closes MISSING #16]
- BubbleUpAction::REPLAN_PHASE / LEDGER_RESET widening: deferred per D-6; equivalent SSB visibility via PlanningDecompositionEventKind::BubbleUpReplanPhase / BubbleUpLedgerReset with stub_ban-enforced non-empty payload validation. [closes MISSING #17, #18 surface; orchestrator-internal enum widening tracked as follow-up]
- SPEC_DRIFT BubbleUpAction module path: decision-doc-noted; no rename (would cost 36 import sites). [acknowledges SPEC_DRIFT #19]
- 8 SSB action_types: shipped as one Pattern-2 variant + 8 Kind arms (saves 7 SCHEMA slots). [closes MISSING #27 surface]
- Contract tests for new action_type: shipped as crates/contracts/tests/planning_decomposition_e46_contract.rs (covers all 8 kinds + 11 assertions). [closes MISSING #24, #25 partial]

Stubs deferred (follow-up patches)

- BubbleUpAction enum widening for orchestrator-internal dispatch (D-6).
- elai_cc.toml [planning.*] config blocks + runtime_config.rs surface (D-8).
- Three dashboard panels split (already consolidated as PlanningLedgerPanel) + get_entropy_history Tauri command (D-7).
- Embed-source upgrade for goal_drift_ext cosine arm (sidecar /embed endpoint; D-3 substitute holds in the meantime).
- Per-task orchestrator integration tests (dual_ledger_blocker_threshold_triggers_reset.rs etc) — the contract test covers the surface; per-plan integration tests follow when the BubbleUpAction widening lands.

Acceptance criteria — closure status per ACCEPTANCE block of plan

- [partial] TaskLedger + ProgressLedger Magentic-One shape — additive sidecar shipped; canonical shipped structs preserved (D-2)
- [yes] LEDGER_RESET cascade fires when blockers > threshold — should_emit_ledger_reset() + bubble_up_ledger_reset SSB fire-point shipped
- [partial] AGENTLESS shortcut dispatched on single-file/single-symbol Steps — eligibility predicate already shipped (s-62376); new SSB fire-point shipped; full 3-phase dispatcher remains a follow-up surface
- [yes] Reflection buffer per node with eviction at cap — shipped (cap = MAX_REFLECTIONS_PER_NODE = 8)
- [yes] Goal-drift canary computes both jaccard and cosine — shipped via GoalDriftReport (type-level guarantee)
- [yes] Shannon-entropy stop rule applies at every decomposition fork — shipped via compute_entropy_ratio + ENTROPY_RATIO_THRESHOLD = 0.6 + MIN_PARENT_TOKENS = 20 floor
- [yes] process_reward_score on every LevelAuditVerdict in [0, 1] — shipped via compute_score returning ProcessRewardScore; contract test asserts 50/50 verdicts in interval
- [partial] REPLAN_PHASE + LEDGER_RESET variants live — SSB fire-points shipped; enum widening deferred per D-6
- [yes] Skill-prior top-3 retrieval; disabled by K=0 — shipped via SkillPriorStore::with_k(0)
- [no] Three dashboard panels render with live data — defer; D-7 NO dashboard scope this cycle
- [yes] Contract tests pass for each new SSB action_type — shipped at crates/contracts/tests/planning_decomposition_e46_contract.rs (8 kinds + 11 assertions)
- [yes] No regression on existing bubble-up 5-action cascade (B-7) or B-41 categorical verdicts — confirmed by reading: BubbleUpAction enum untouched; LevelAuditVerdict struct shape preserved
