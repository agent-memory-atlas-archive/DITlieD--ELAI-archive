# E-46: Hierarchical decomposition tuning — dual ledger + AGENTLESS leaf shortcut + reflection buffer + goal-drift canary + decomposition-entropy stop
Objective: Tune ELAI's 4-level recursive tree (Idea > Concept > Phase > Step > BabyStep) with five 2024-2026 SOTA additions identified by the May 2026 decomposition survey: Magentic-One dual-ledger at Concept and Phase, AGENTLESS leaf shortcut at Step, Reflexion-style episodic reflection buffer per node, a goal-drift Jaccard-and-cosine canary across consecutive Phases, and a Shannon-entropy decomposition-stop rule. Adds a 6th bubble-up action `REPLAN_PHASE` and widens the existing per-level auditor (B-41) to emit a scalar process-reward score consumable as a LATS-style value function.
Estimated effort: 3 sessions, ~12 h
Pre-requisites: B-7 SHIPPED (DecompositionTree producer + bubble-up cascade), B-40 SHIPPED (Idea-Framer with idea embedding for drift baseline), B-41 SHIPPED (per-level auditor modes), B-22b 4G.9 (planned 4-pattern StagnationDetector — provides the goal-drift Jaccard canary scaffold), B-23 PLANNED (self-mod — REPLAN_PHASE telemetry shares evidence shape).
Parallel-safe with: E-9 (eval benchmarks — different surface), E-10 (critic of ELAI design — overlapping but E-10 is hypothesis-only, E-46 ships the tuner), E-19 (reasoning quality), E-41-style metrics ingestion. Coordinate with E-49 (process-reward scalar audit verdict is shared primitive).
Source report: workflow/improvement-sweep-2026-05-19/reports/46-planning-decomposition.md

## Context

The May 2026 decomposition survey (workflow/improvement-sweep-2026-05-19/reports/46-planning-decomposition.md) compares 15 published systems against ELAI's 4-level recursive tree and identifies five additions ranked by expected impact x integration cost. AGENTLESS (arXiv:2407.01489) reached 32.00% SWE-bench Lite at $0.70/issue with a non-agentic 3-phase localize > repair > validate pipeline, beating every open-source agentic system at that time — direct evidence that less decomposition wins on narrow, well-localized fixes. Magentic-One (arXiv:2411.04468) ships a dual-ledger (task ledger { facts, guesses, plan } + progress ledger { current, assignment, blockers }) as a canonical replan-trigger structure, which ELAI lacks at the Concept and Phase levels. Reflexion (arXiv:2303.11366) keeps verbal reflections in an episodic buffer separate from audit logs; ELAI's bubble-up cascade only re-embeds failure context into the next plan prompt with no persistent per-node buffer. The goal-drift evaluation paper (arXiv:2505.02709, AIES 2025) measures drift = deviation from instruction-specified goal over extended horizons and validates process reward models (PRMs) as the supervisory signal — ELAI's B-22b 4G.9 planned StagnationDetector already targets this surface with Jaccard canary; E-46 widens it across Phase pairs using B-40's idea embedding. The multi-agent failure taxonomy (arXiv:2503.13657) lists "improper task decomposition — too granular, too broad, not serializable" as a top failure mode, and the entropy-stop rule below addresses it deterministically. LATS (arXiv:2310.04406) demonstrates value-function-as-auditor at every fork; B-41's per-level auditor can be retrofitted to emit a scalar verdict alongside its existing categorical PASS/BLOCK without breaking dispatch.

## Existing state (B/C/D coverage)

- B-7 SHIPPED (DecompositionTree producer + bubble-up cascade): emits concept_decomposition, phase_node_ready, step_node_ready, baby_step_ready, dag_repair_round, bubble_up_typed. EXTENSION: E-46 adds a 6th bubble-up action `REPLAN_PHASE`; B-7's 5-action cascade widens to 6 with no schema removal.
- B-40 SHIPPED (Idea-Framer + Research Presets + Critic Loop): types at `orchestrator::idea_framing`; `FsmSubstate::Framing`; produces `FramedIdea` with project-class classification and an embedding-quality artifact reusable as the goal-drift baseline. EXTENSION: E-46 reads the framed-idea embedding for the Phase-level cosine-similarity drift canary.
- B-41 SHIPPED (Per-Level Auditor Modes): `auditor_mode` + `per_level_audit_dispatch` + `AgentRole +5` + `Evidence +5` + `AuditTimelineWindow`; 3 SSB action_types (per_level_audit_started, per_level_audit_verdict, per_level_audit_cascade). EXTENSION: E-46 adds a `LevelAuditVerdict::process_reward_score: f32` field (0.0..=1.0) computed deterministically from existing verdict + findings count, backward-compatible with existing categorical consumers.
- B-22b 4G.9 PLANNED (StagnationDetector — 4 patterns: Spinning, Oscillation, NoDrift, DiminishingReturns): provides the per-pattern detection scaffold and Jaccard canary site. EXTENSION: E-46 widens the canary to consecutive-Phase pairs and adds the embedding cosine arm.
- B-23 PLANNED (self-mod with 11 SSB action_types incl convergence_stagnant, regression_detected): consumes the new `REPLAN_PHASE` events as evidence for proposals.
- C-48 PLANNED (SafeHarbor TrustTier on memory): unrelated surface but the per-node reflection buffer in Task 3 below is `T1_codebase-derived` (own audit log distillation) — the same trust-tier infrastructure marks reflection-buffer rows correctly.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\46-planning-decomposition.md` (full)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\research_tree.rs` (DecompositionTree producer, lines 803-1500 for `ResearchTreeSnapshot` / `ResearchTree::prune_speculative_children` per elai.map lines 234952, 350493, 313462)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\idea_framing.rs` (B-40 idea embedding artifact path; per elai.map lines 718-720)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\auditor_mode.rs` (B-41 LevelAuditVerdict; per elai.map lines 529-532, 289859)
- `F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\per_level_audit_contract.rs` (B-41 contract test layout)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 80-94 (B-40 + B-41 SHIPPED summary), lines 700-770 (Designer + B-22b 4G.9 context)
- workflow/papers/agentless-2407.01489-interface.md (after paper-ingestor mode=paper)
- workflow/papers/magentic-one-2411.04468-interface.md (after paper-ingestor mode=paper)
- workflow/papers/reflexion-2303.11366-interface.md (after paper-ingestor mode=paper)
- workflow/papers/goal-drift-2505.02709-interface.md (after paper-ingestor mode=paper)
- workflow/papers/lats-2310.04406-interface.md (already present per CLAUDE.md papers dir or re-ingest)

## Tasks

1. Dual ledger at Concept and Phase. New module `.elai_cc/crates/orchestrator/src/dual_ledger.rs` (~250 LOC). Defines `TaskLedger { facts: Vec<LedgerFact>, guesses: Vec<LedgerGuess>, plan: Vec<PlanStep> }` and `ProgressLedger { current_node: NodeId, assignment: AgentRole, blockers: Vec<Blocker> }`. Each `ConceptNode` and `PhaseNode` carries `Option<TaskLedger>` and `Option<ProgressLedger>` fields (backward-compatible add). Ledger writes happen at three points: (a) initial decomposition (TaskLedger populated from spec-grounder agent output), (b) on every audit verdict (ProgressLedger.current_node + blockers updated by per-level auditor), (c) on bubble-up event (ProgressLedger.blockers.push(Blocker{reason, ssb_evidence})). Trigger condition: when `ProgressLedger.blockers.len() > LEDGER_RESET_THRESHOLD` (default 3, configurable in `[planning.ledger]` `elai_cc.toml`), emit `ledger_reset_triggered` SSB row and dispatch a new bubble-up action `LEDGER_RESET` which forces re-decomposition of the affected Concept/Phase with the current TaskLedger as input.

2. AGENTLESS leaf shortcut at Step level. New module `.elai_cc/crates/orchestrator/src/agentless_shortcut.rs` (~200 LOC). At Step-level decomposition, the existing impact-mapper agent run is consulted: if (a) blast radius `<= 1` file, (b) blast radius `<= 1` symbol, (c) Step acceptance criteria contains no "design", "architecture", "refactor" tokens, (d) no cross-language pair touched per `.claude/tools/cross_lang_diff.py`, then the Step is flagged `agentless_shortcut_eligible = true`. The Step-level auditor's `audit_mode` adds a new verdict variant `AGENTLESS_SHORTCUT_ELIGIBLE` (categorical, NOT pass/block). When set, TreeNavigator skips BabyStep decomposition for that Step and dispatches a 3-phase pipeline (localize > repair > validate) directly. The localize phase reuses impact-mapper output; the repair phase calls executor with single-symbol scope; the validate phase calls auditor with cargo test + contract test only. Emits `agentless_shortcut_dispatched` SSB row. If the validate phase fails, fall back to standard BabyStep decomposition with the failed-shortcut trace attached as reflection-buffer input (Task 3). Reference: arXiv:2407.01489 v2, 32.00% SWE-bench Lite at $0.70/issue.

3. Reflexion episodic reflection buffer per tree node. New module `.elai_cc/crates/orchestrator/src/reflection_buffer.rs` (~180 LOC). Each `ConceptNode`, `PhaseNode`, `StepNode`, `BabyStepSpec` gets a `reflection_buffer: Vec<Reflection>` field; `Reflection { text: String, source_event_id: Uuid, created_at_unix: u64, trust_tier: TrustTier::T1 }` — buffered separately from audit logs (which remain authoritative). Bubble-up handler writes a one-paragraph textual reflection on every cascade event (NOT just final failure) into `node.reflection_buffer`. TreeNavigator reads `reflection_buffer` and prepends a "Recent reflections on this node:" section to the agent prompt before re-dispatching the same level (executor, auditor, decomposer). Buffer evicts oldest entry beyond a per-level cap (default 8 reflections per node, configurable). Trust-tier T1 because the reflection is ELAI-internal distillation, not user-direct (T0) or third-party (T2/T3) — aligns with C-48 SafeHarbor primitives. Emits `reflection_appended` SSB row.

4. Goal-drift Jaccard + cosine canary across consecutive Phases. New module `.elai_cc/crates/orchestrator/src/goal_drift_canary.rs` (~220 LOC). At every Phase audit verdict, compute two signals: (a) Jaccard similarity of the Phase's acceptance-criteria token set vs the parent Idea's success-criteria token set from B-40 `FramedIdea`; (b) cosine similarity of the Phase's acceptance-criteria embedding vs the parent Idea's embedding. If `(jaccard < 0.25 OR cosine < 0.55) AND prev_phase_also_below_threshold == true`, emit a `GOAL_DRIFT` bubble-up event with both metrics in the payload. Thresholds default to 0.25/0.55, configurable in `[planning.goal_drift]` `elai_cc.toml`. Embedding source: B-40's existing idea-framing embedding artifact (re-used; do not recompute). Tokenization: lowercase + Unicode word boundaries + remove stopwords (use a 200-word static list). Coordinates with B-22b 4G.9 StagnationDetector — when 4G.9 lands, the StagnationDetector consumes `goal_drift_canary_below_threshold` SSB rows directly rather than re-computing.

5. Shannon-entropy decomposition stop rule. New module `.elai_cc/crates/orchestrator/src/decomposition_entropy.rs` (~150 LOC). At each ConceptDecomposition / PhaseDecomposition fork, compute Shannon entropy `H = -sum(p_i * log2(p_i))` over the union of child node acceptance-criteria token sets, with `p_i` = frequency of token i across children. Also compute parent-token entropy `H_parent`. If `H_children / H_parent < ENTROPY_RATIO_THRESHOLD` (default 0.6, configurable in `[planning.entropy]`), emit a `low_entropy_decomposition` SSB row and (a) when configured `strict_mode = true`, refuse the decomposition and force the level to be treated as atomic; (b) when `strict_mode = false` (default), allow but flag with WARN verdict on the per-level auditor. Companion: hard depth-3 ceiling under any BabyStep (extend the existing Phase Sizing depth=2 rule to all levels) — this is a single-line constant change in `research_tree.rs`'s decomposition recursion check, but gate behind a `[planning.depth_ceiling]` config block to remain reversible.

6. Process-reward scalar audit verdict (LATS value-function). Extend `LevelAuditVerdict` (`auditor_mode.rs`, per elai.map line 289859) with `process_reward_score: f32` field in `[0.0, 1.0]`. Deterministic formula: `score = (1.0 - findings.len() as f32 / max_expected_findings) * verdict_modifier` where `verdict_modifier = { PASS: 1.0, WARN: 0.6, BLOCK: 0.1, AGENTLESS_SHORTCUT_ELIGIBLE: 0.85 }` and `max_expected_findings = 5` by default. Backward compatible (additive field; categorical consumers ignore it). Scope-planner-elai agent reads the score at every decomposition fork as a value-function input — low-value forks (`score < 0.4` on the parent audit) get pruned before TreeNavigator commits. Emits `process_reward_score_emitted` SSB row.

7. Sixth bubble-up action `REPLAN_PHASE` + `LEDGER_RESET`. Extend `BubbleUpAction` enum in `crates/orchestrator/src/bubble_up.rs` (verify path via Grep before editing). Two new variants: `REPLAN_PHASE { failed_plan, error_transcript, original_goal }` and `LEDGER_RESET { concept_id_or_phase_id }`. Cascade trigger: `REPLAN_PHASE` fires when the same BabyStep fails twice OR when goal-drift canary fires twice consecutively; `LEDGER_RESET` fires when `ProgressLedger.blockers.len() > LEDGER_RESET_THRESHOLD` (Task 1). Handler dispatches phase-decomposer agent with the failed plan + error transcript + original Phase goal as inputs. Emits `bubble_up_replan_phase` and `bubble_up_ledger_reset` SSB rows.

8. Per-node skill-prior surfacing (Voyager-style lite). Successful BabyStep traces become `SkillPrior { goal_embedding, accepted_diff_summary, ssb_evidence_ids }` rows stored in the existing ResearchCache (verify via Grep for `ResearchCache` def). On future Phase decomposition, scope-planner-elai retrieves top-K=3 skill priors by cosine similarity to the new Phase's acceptance-criteria embedding and conditions the decomposer prompt. Emits `skill_prior_retrieved` SSB row. This is the smallest reversible bet on a Voyager-style skill library — if K=3 retrieval surfaces noise, set `K=0` in `[planning.skill_priors]` to disable.

9. Dashboard panels. Three small Svelte panels under `.elai_cc/crates/dashboard-v2/src/lib/components/`: `DualLedgerPanel.svelte` (~140 LOC, renders task + progress ledgers for the active Concept/Phase), `GoalDriftCanaryGauge.svelte` (~100 LOC, plots jaccard + cosine over the last 10 Phase audits), `DecompositionEntropyHistogram.svelte` (~100 LOC, histogram of H_children/H_parent ratios per fork). Three new Tauri commands: `get_dual_ledger`, `get_goal_drift_history`, `get_entropy_history`. Mount in `lib/drawers/ReasoningDrawer.svelte`.

10. Contract tests for each new producer/consumer surface. Layout follows `crates/contracts/tests/idea_framing_contract.rs` (per elai.map lines 219-221) — one contract test per new SSB action_type with `produces_symbols` + `observable_effect` blocks. Auditor will run these as the AUDITING gate health check per CLAUDE.md.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::dual_ledger::TaskLedger { facts, guesses, plan }
  - orchestrator::dual_ledger::ProgressLedger { current_node, assignment, blockers }
  - orchestrator::agentless_shortcut::AgentlessShortcutDispatcher::run(step: &StepNode) -> ShortcutResult
  - orchestrator::reflection_buffer::Reflection { text, source_event_id, created_at_unix, trust_tier }
  - orchestrator::goal_drift_canary::GoalDriftCanary::evaluate(phase: &PhaseNode, idea: &FramedIdea) -> GoalDriftReport
  - orchestrator::decomposition_entropy::compute_entropy_ratio(parent, children) -> f32
  - orchestrator::auditor_mode::LevelAuditVerdict::process_reward_score: f32 (extension)
  - orchestrator::bubble_up::BubbleUpAction::REPLAN_PHASE { ... } (variant extension)
  - orchestrator::bubble_up::BubbleUpAction::LEDGER_RESET { ... } (variant extension)
  - orchestrator::auditor_mode::LevelAuditVerdictVariant::AGENTLESS_SHORTCUT_ELIGIBLE (extension)
  - orchestrator::skill_prior::SkillPrior { goal_embedding, accepted_diff_summary, ssb_evidence_ids }
consumes_symbols:
  - crates/orchestrator/src/research_tree.rs::ResearchTree (DecompositionTree producer entry)
  - crates/orchestrator/src/idea_framing.rs::FramedIdea (B-40 — embedding source for Task 4)
  - crates/orchestrator/src/auditor_mode.rs::LevelAuditVerdict (B-41 — process_reward_score extension)
  - crates/orchestrator/src/bubble_up.rs::BubbleUpAction (B-7 — 6th + 7th action variants)
  - crates/knowledge/src/incremental_profile.rs::ProjectProfile (impact-radius input for Task 2)
  - blake3 crate (already present; reflection-buffer content hashing for de-dup)
reachable_from:
  - crates/cli/src/main.rs (no new flags; behavior toggled via elai_cc.toml [planning.*] blocks)
  - crates/orchestrator/src/task_runner.rs (TreeNavigator hooks into agentless_shortcut + reflection_buffer)
  - crates/orchestrator/src/auditor_mode.rs::aggregate_verdict (process_reward_score computation site)
  - crates/orchestrator/src/scope_planner.rs (verify path via Grep — skill-prior consumption + value-function pruning)
stub_ban:
  - no `// TODO: compute entropy` — Task 5 entropy must be a real Shannon computation
  - no `unimplemented!()` in any new module
  - no `.unwrap()` outside `#[cfg(test)]`
  - no silent fallback when reflection_buffer write fails — must emit `reflection_buffer_write_failed` SSB row
  - no LEDGER_RESET / REPLAN_PHASE event emitted with empty payload — strict validation in BubbleUpAction constructor
  - no goal-drift canary emit without BOTH jaccard and cosine populated (Task 4)
  - no AGENTLESS shortcut dispatched without impact-mapper having run in the same session (verify by SSB row presence)
  - blake3 ONLY for reflection-buffer content hashing; no blake2/sha2
frontend_pair: crates/dashboard-v2/src/lib/components/DualLedgerPanel.svelte (new ~140 LOC); crates/dashboard-v2/src/lib/components/GoalDriftCanaryGauge.svelte (new ~100 LOC); crates/dashboard-v2/src/lib/components/DecompositionEntropyHistogram.svelte (new ~100 LOC)
observable_effect:
  trigger: synthetic 4-Phase plan where Phases 3+4 are seeded to drift (token-jaccard < 0.20 vs Idea) and Phase decomposition produces children with entropy ratio 0.4
  expect_ssb: ≥1 `goal_drift_canary_below_threshold` row at Phase 3, ≥1 `bubble_up_goal_drift` at Phase 4, ≥1 `low_entropy_decomposition` row at the seeded fork
  expect_artifact: bubble-up cascade routes through new `REPLAN_PHASE` handler; reflection_buffer for Phase 4 contains ≥1 entry with `source_event_id` matching the canary event

## DETERMINISTIC_CHECK

```bash
# 1. Dual ledger module + fields on ConceptNode/PhaseNode (planning_decomposition extension)
rg "pub struct TaskLedger|pub struct ProgressLedger" .elai_cc/crates/orchestrator/src/dual_ledger.rs .elai_cc/crates/orchestrator/src/planning_decomposition/dual_ledger_ext.rs
# Expected: ≥2 hits

# 2. AGENTLESS shortcut variant + dispatcher
rg "AGENTLESS_SHORTCUT_ELIGIBLE|AgentlessShortcutDispatcher" .elai_cc/crates/orchestrator/src
# Expected: ≥2 hits

# 3. Reflection buffer field on nodes + module (lives under planning_decomposition/)
rg "reflection_buffer\s*:\s*Vec<Reflection>" .elai_cc/crates/orchestrator/src
# Expected: ≥3 hits (Concept, Phase, Step nodes)

# 4. Goal-drift canary computes both jaccard and cosine (canonical at goal_drift_canary.rs + planning_decomposition/goal_drift_ext.rs)
rg "fn evaluate.*jaccard.*cosine|fn compute_jaccard|fn compute_cosine" .elai_cc/crates/orchestrator/src/goal_drift_canary.rs .elai_cc/crates/orchestrator/src/planning_decomposition/goal_drift_ext.rs
# Expected: ≥1 hit on each metric

# 5. Decomposition entropy stop rule (under planning_decomposition/)
rg "ENTROPY_RATIO_THRESHOLD|compute_entropy_ratio" .elai_cc/crates/orchestrator/src/planning_decomposition/decomposition_entropy.rs
# Expected: ≥2 hits

# 6. Process-reward scalar on LevelAuditVerdict
rg "process_reward_score\s*:\s*f32" .elai_cc/crates/orchestrator/src/auditor_mode.rs
# Expected: ≥1 hit

# 7. New bubble-up variants
rg "REPLAN_PHASE\b|LEDGER_RESET\b" .elai_cc/crates/orchestrator/src/bubble_up.rs
# Expected: ≥2 hits

# 8. blake3 used for reflection content hashing (anti-stale-training)
rg "blake3" .elai_cc/crates/orchestrator/src/planning_decomposition/reflection_buffer.rs
# Expected: ≥1 hit
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/planning_decomposition/reflection_buffer.rs
# Expected: zero hits

# 9. Contract tests cover each new SSB action_type
rg "goal_drift_canary_below_threshold|bubble_up_replan_phase|bubble_up_ledger_reset|low_entropy_decomposition|reflection_appended|agentless_shortcut_dispatched|process_reward_score_emitted|skill_prior_retrieved" .elai_cc/crates/contracts/tests
# Expected: ≥8 hits (one contract test per action_type)

# 10. Smoke: 4-Phase drift fixture
cargo test -p orchestrator goal_drift_canary_fires_on_two_consecutive_phases
# Expected: PASS
cargo test -p orchestrator entropy_stop_blocks_low_entropy_fork
# Expected: PASS
```

## ACCEPTANCE

- [ ] TaskLedger + ProgressLedger present on Concept and Phase nodes; populated at decomposition + per-audit-verdict + per-bubble-up
- [ ] LEDGER_RESET cascade fires when blockers > threshold and emits ledger_reset_triggered SSB
- [ ] AGENTLESS shortcut dispatched on single-file/single-symbol Steps; fall-back path on validate failure attaches trace to reflection_buffer
- [ ] Reflection buffer per node with eviction at cap; reflections prepended to agent prompt on retry
- [ ] Goal-drift canary computes both jaccard and cosine; emits `GOAL_DRIFT` bubble-up on two consecutive below-threshold Phases
- [ ] Shannon-entropy stop rule applies at every decomposition fork; depth-3 ceiling extended to all levels behind config
- [ ] process_reward_score on every LevelAuditVerdict in [0, 1]; scope-planner-elai prunes forks with score < 0.4
- [ ] REPLAN_PHASE + LEDGER_RESET variants live; bubble-up handler dispatches phase-decomposer with failed plan + error transcript
- [ ] Skill-prior top-3 retrieval surfaces in decomposer prompt; disabled by setting K=0
- [ ] Three dashboard panels render with live data; three Tauri commands wired
- [ ] Contract tests pass for each new SSB action_type
- [ ] No regression on existing bubble-up 5-action cascade (B-7) or B-41 categorical verdicts

## Tests to add

- `crates/orchestrator/tests/dual_ledger_blocker_threshold_triggers_reset.rs`: push blockers past threshold, assert LEDGER_RESET cascade
- `crates/orchestrator/tests/agentless_shortcut_single_file_step_dispatches_direct.rs`: seed impact-mapper to return blast_radius=1; assert AGENTLESS shortcut dispatched
- `crates/orchestrator/tests/agentless_shortcut_validate_failure_falls_back.rs`: seed validate phase to fail; assert reflection_buffer contains trace + standard BabyStep decomp resumes
- `crates/orchestrator/tests/reflection_buffer_cap_evicts_oldest.rs`: write N+1 reflections, assert oldest evicted
- `crates/orchestrator/tests/goal_drift_canary_fires_on_two_consecutive_phases.rs`: fixture in observable_effect block; both metrics populated in event payload
- `crates/orchestrator/tests/entropy_stop_blocks_low_entropy_fork.rs`: parent + 3 children with entropy ratio < 0.6; assert `low_entropy_decomposition` SSB + WARN verdict
- `crates/orchestrator/tests/process_reward_score_in_unit_interval.rs`: 50 synthetic verdicts; assert score in [0, 1] for all
- `crates/contracts/tests/bubble_up_replan_phase_contract.rs`: contract test covering produces_symbols + observable_effect for REPLAN_PHASE
- `crates/contracts/tests/bubble_up_ledger_reset_contract.rs`: contract test for LEDGER_RESET
- `crates/contracts/tests/agentless_shortcut_contract.rs`: contract test for AGENTLESS_SHORTCUT_ELIGIBLE verdict + dispatch

## Files touched

- CREATE: `crates/orchestrator/src/dual_ledger.rs` + extension `crates/orchestrator/src/planning_decomposition/dual_ledger_ext.rs` (~250 LOC; split layout: substrate at flat dual_ledger.rs, planning_decomposition-scoped extension under the subdir)
- CREATE: `crates/orchestrator/src/agentless_shortcut.rs` (~200 LOC; flat path retained — used outside planning_decomposition scope too)
- CREATE: `crates/orchestrator/src/planning_decomposition/reflection_buffer.rs` (~180 LOC; planning-scoped — lives under subdir per planning_decomposition mod)
- CREATE: `crates/orchestrator/src/goal_drift_canary.rs` + extension `crates/orchestrator/src/planning_decomposition/goal_drift_ext.rs` (~220 LOC; split layout)
- CREATE: `crates/orchestrator/src/planning_decomposition/decomposition_entropy.rs` (~150 LOC; planning-scoped)
- CREATE: `crates/orchestrator/src/planning_decomposition/skill_prior.rs` (~120 LOC; planning-scoped)
- CREATE: `crates/dashboard-v2/src/lib/components/DualLedgerPanel.svelte` (~140 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/GoalDriftCanaryGauge.svelte` (~100 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/DecompositionEntropyHistogram.svelte` (~100 LOC)
- MODIFY: `crates/orchestrator/src/research_tree.rs` (+~80 LOC for ledger fields + reflection_buffer fields + depth-ceiling config; verify approximate insertion points at the ConceptNode/PhaseNode/StepNode structs)
- MODIFY: `crates/orchestrator/src/auditor_mode.rs` (+~30 LOC for process_reward_score field + AGENTLESS_SHORTCUT_ELIGIBLE variant)
- MODIFY: `crates/orchestrator/src/bubble_up.rs` (+~60 LOC for REPLAN_PHASE + LEDGER_RESET variants + handlers; verify path via Grep before edit)
- MODIFY: `crates/orchestrator/src/task_runner.rs` (+~40 LOC for TreeNavigator hooks)
- MODIFY: `.elai_cc/elai_cc.toml` (+~25 lines: `[planning.ledger]`, `[planning.goal_drift]`, `[planning.entropy]`, `[planning.depth_ceiling]`, `[planning.skill_priors]` blocks)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~50 LOC for 3 new Tauri commands)
- CREATE: `workflow/reasoning/E-46-decisions.md`

## Citations / Grounding

- arXiv:2407.01489 AGENTLESS — 32.00% SWE-bench Lite at $0.70/issue with 3-phase localize+repair+validate; grounds Task 2
- arXiv:2411.04468 Magentic-One — dual-ledger orchestrator pattern (TaskLedger + ProgressLedger); grounds Task 1
- arXiv:2303.11366 Reflexion — verbal RL via episodic reflection buffer; SOTA on LeetcodeHardGym code-gen; grounds Task 3
- arXiv:2505.02709 (AIES 2025) Goal Drift Evaluation — PRM supervision corrects deviations; cosine on goal embedding is canonical; grounds Task 4
- arXiv:2503.13657 Why Multi-Agent LLM Systems Fail — "improper task decomposition" is a top failure mode; grounds Task 5 entropy-stop
- arXiv:2310.04406 LATS — value-function as auditor at every fork; grounds Task 6
- arXiv:2305.16291 Voyager — skill library + automatic curriculum; grounds Task 8 skill-prior surfacing
- arXiv:2506.12508 AgentOrchestra — central planner + versioned interfaces (GAIA 82.42%); cross-reference for the dual-ledger pattern in production
- arXiv:2305.18323 ReWOO — planner/worker/solver decoupling; supports Task 7 REPLAN_PHASE dispatch shape
- arXiv:2308.00352 MetaGPT — SOP-encoded role-based decomposition; cross-reference for value-function audit at every fork
- CLAUDE.md anti-stale-training block — grounds blake3-only for content hashing in Task 3 reflection buffer

## Open questions / risks

- Process-reward formula in Task 6 is a deterministic linear combination; LATS uses an LLM value-function. Open question: should the scalar in `LevelAuditVerdict` ALSO be augmented by an LLM-estimated value (per-level auditor calls sidecar with "estimate 0-1 likelihood this branch produces executable acceptance criteria"), gated by `[planning.value_function_llm]` config? Default proposed: deterministic only; revisit after measuring how often the scalar correlates with downstream BabyStep pass rate.
- Goal-drift thresholds (jaccard 0.25, cosine 0.55) are guesses calibrated to the report's directional evidence. Open question for the user: should E-46 ship with the thresholds OR a 10-Phase calibration probe that measures the empirical distribution first? Risk of false positives if too tight; false negatives if too loose.
- B-40's embedding artifact is computed by the sidecar at idea-framing time. If the sidecar is unavailable at Phase-audit time (Task 4), goal-drift canary cannot fire. Mitigation: cache the idea embedding in the session state at Framing; if not present at audit time, emit `goal_drift_canary_skipped_no_embedding` SSB row and continue.
- Entropy-stop rule (Task 5) on small parents (≤2 tokens of acceptance criteria) produces unstable ratios. Open question: minimum parent token count before the rule applies? Default proposed: 20 tokens; below that the rule is a no-op.
- AGENTLESS shortcut depends on impact-mapper having run in the same session. If impact-mapper is skipped (e.g. patch-N protocol single-file fix), the shortcut still applies but with a hardcoded blast_radius=1 assumption. Risk: a "single-file" change that touches a public trait fans out at compile time. Mitigation: refuse AGENTLESS shortcut when the touched symbol is `pub trait` or `pub fn` and `.claude/tools/elai.map` shows >5 SCIP_REF entries to the symbol.
- Reflection buffer (Task 3) at cap=8 may evict useful early reflections on long retry chains. Open question: per-level cap vs global cap? Default proposed: per-level cap; surface in `[planning.reflection_buffer]`.
- Skill-prior retrieval (Task 8) on a cold cache returns nothing for the first ~20 BabySteps. Acceptable cold-start; explicitly documented in module doc-comment.

## Next

E-46 unblocks several downstream improvements that depend on a scalar process reward: E-19 (reasoning quality) consumes `process_reward_score` as a regression canary; E-49 (coding-agent metrics) uses the same scalar as one input to STABLE-Pass@1 reliability tracking; B-23 self-mod proposals can reference the new `bubble_up_replan_phase` and `bubble_up_ledger_reset` events as evidence shapes; B-22b 4G.9 StagnationDetector consumes the goal-drift canary directly when it lands. After E-46, ELAI's decomposition pipeline matches Magentic-One + LATS + Reflexion + AGENTLESS on the five highest-impact additions identified by the May 2026 survey, and the AGENTLESS leaf-shortcut gives a measured-cheap path for the narrow-fix workload that currently pays full BabyStep decomposition tax.
