# B-41: Per-Level Auditor Modes (Concepts/Phases/Steps/BabySteps/BabyStepDetail/Plan)
Objective: replace ELAI's single-mode auditor with a 6-mode per-level
auditor that audits each tier of the DecompositionTree (Concepts, Phases,
Steps, BabySteps, BabyStepDetail batches, final Plan) immediately after
that tier is produced. Each mode loads the same rule cluster (anti-stale +
coding-standards + design-hard + app-store + source-tiers + slopsquatting +
risk-levels) but scopes findings to its tier only. BLOCK at tier N respawns
the level-N decomposer ONCE (no re-audit), then cascades to N+1. Catches
bad concepts before 50 wasted babysteps; catches bad babysteps before
they consume executor cycles.
Estimated effort: 2-3 sessions.
Pre-requisites: B-22 (AutoAdvanceEvaluator pattern — per-level verdicts feed
  the "zero BLOCKs" evidence check at appropriate granularity), B-40
  (idea-framer provides grounded reference material for mode=concepts +
  mode=phases audits — without it the auditor compares decomposition
  against nothing). SCHEMA_VERSION currently 28 (29 if B-40 ships first);
  B-41 bumps by +1 with 3 new SSB action_types.
Parallel-safe with: B-24, B-25, B-27, B-28, B-37, B-38, B-42.

## Spec references

- docs/specs/guide/steps/phase-4D.txt 4D.7-4D.10 — DecompositionTree
  (Concept → Phase → Step → BabyStep, 4 tiers).
- docs/specs/guide/steps/phase-4E.txt 4E.10 — BubbleUpHandler retry
  thresholds: BabyStep=3, Step=2, Phase=1, Concept=1, Escalate. B-41
  auditor BLOCK at tier N respawns the level-N decomposer exactly
  once (NOT three retries of the same auditor — different policy).
- docs/specs/guide/steps/phase-4G.txt 4G.4 — 6 evidence checks; B-41
  splits the existing "zero BLOCKs" check into per-level checks
  (concept_zero_blocks, phase_zero_blocks, step_zero_blocks,
  babystep_zero_blocks, plan_zero_blocks). Each gates auto-advance at
  its tier.
- arXiv:2406.01297 (Kamoi et al., TACL 2024) — verifier prompt
  scopes determine recall/precision tradeoff. Per-level scoping
  improves both vs single global auditor.
- The originating-chat reference image (10 critique categories ×
  per-level audits) — B-41 implements the per-level dimension; B-40
  implements the critique categories.

## Context

Today: ELAI ships a single .claude/agents/auditor.md (FSM AUDITING state,
4 sub-states: READ_CONTEXT / RUN_TESTS / REVIEW_CODE / WRITE_FINDINGS).
It runs ONCE at end-of-implementation, audits everything, returns one
verdict. It can't tell whether a BLOCK comes from a flawed Concept-level
choice or a single misplaced semicolon at BabyStep level. When the
Concept was bad, the auditor sees 135 babysteps of downstream rot and
either (a) BLOCKs everything (cascade unrecoverable), or (b) misses the
root cause and approves shippable nonsense.

Phase 4E's BubbleUpHandler (DONE per audit) already cascades retry up
the tree, but it triggers on test/integration failure, not on auditor
verdict. There's no auditor verdict at intermediate tiers because no
auditor runs there.

Per-level audits = 5 inserted auditor cycles, one per tier:
- mode=concepts: after Designer produces Concepts (5 typical) — checks
  market fit, IP boundaries, scope, contradictions vs framed idea.
- mode=phases: after each Concept is decomposed into Phases — checks
  phase ordering, contract sufficiency, dependency cycles.
- mode=steps: after each Phase is decomposed into Steps — checks step
  granularity, test coverage of step boundaries.
- mode=babysteps: after each Step is decomposed into BabySteps —
  checks BabyStepSpec 8-block completeness (GROUNDING / PRE_GATE /
  TASK / TDD / DETERMINISTIC_CHECK / ACCEPTANCE / INTEGRATION /
  EXIT_CRITERIA).
- mode=babystep-detail: AFTER a batch of BabySteps is authored (≤4
  parallel per BubbleUp policy) — line-level lazy-code / unwrap /
  hardcoded-secret / API-version-stale checks.
- mode=plan (existing single auditor cycle, retained): final
  end-of-task pass that nothing else covers.

BLOCK policy per the originating-chat note: BLOCK at tier N respawns
level-N decomposer ONCE, no re-audit (cascades caught by next tier).
This differs from BubbleUpHandler's 3/2/1/1 RETRY counts because
auditor BLOCKs are deterministic findings — re-running the SAME auditor
on the SAME decomposer output is pure waste. One respawn with the
auditor findings injected as a `rescope_from_audit` hint, then continue.
If the next tier's audit ALSO BLOCKs, the parent tier is the real fault
and bubble-up cascades there.

## Reads (mandatory before coding)

- .claude/agents/auditor.md (current single-mode auditor — full file)
- .elai_cc/crates/orchestrator/src/agent_pool.rs (AgentRole enum — add Auditor variants per mode)
- .elai_cc/crates/orchestrator/src/research_tree.rs (DecompositionTree types)
- .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs (TreeNavigator + 8-step BabyStep cycle — INSERT mode=babysteps audit between step 6 acceptance and step 7 integration)
- .elai_cc/crates/orchestrator/src/task_runner.rs (BubbleUpHandler integration; AutoAdvanceEvaluator from B-22 evidence checks)
- .elai_cc/crates/rules/src (rules.db — seed rules for new domains: design_hard, app_store, source_tiers, slopsquatting, risk_levels — domains not yet in seed)
- .elai_cc/crates/orchestrator/src/ssb.rs (add 3 new SSB action_types + bump SCHEMA_VERSION)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-22_phase_4G_autonomous_mode.md (gate evaluator pattern)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-40_idea_framer_research_presets.md (framed-idea ground truth for mode=concepts)

## Tasks

### Phase 1 — AuditorMode dispatch + agent definition (Session 1)

1. Add `AuditorMode` enum: `Concepts` / `Phases` / `Steps` / `BabySteps`
   / `BabyStepDetail` / `Plan`. Plus `Visual` reserved variant (held for
   B-43 if approved — feature-gated under cfg(feature="visual_audit")).

2. Modify .claude/agents/auditor.md to dispatch by `mode={Mode}` argument:
   - Add MODE DISPATCH section before existing FSM SUB-STATE block.
   - Each mode loads the same rule cluster (load order: anti-stale →
     coding-standards → design-hard → app-store → source-tiers →
     slopsquatting → risk-levels) BUT scopes findings to its tier only.
   - Mode-specific prompt sections (~80 LOC per mode) describe what to
     check at that tier and what to ignore (deferred to other tiers).
   - Sub-state machine (READ_CONTEXT → REVIEW → WRITE_FINDINGS)
     unchanged; RUN_TESTS skipped for modes 1-4 (no executable code yet).

3. Add 6 AgentRole variants in agent_pool.rs (or single
   `Auditor(AuditorMode)` parametric variant — recommend the parametric
   form, simpler to wire). agent_grc_config_for_role in
   reasoning/src/config.rs gets 6 match arms with tiered budgets:
   - Concepts: 6/1000 (semantic, expensive checks)
   - Phases: 5/800
   - Steps: 4/700
   - BabySteps: 4/700
   - BabyStepDetail: 3/600 (line-level, fast)
   - Plan: 10/1200 (existing, retained)

### Phase 2 — Per-tier dispatch + BLOCK policy (Session 2)

4. TreeNavigator integration (orchestrator/src/pipeline_tree_navigator.rs):
   - After Concepts produced: spawn `Auditor(Concepts)`. On BLOCK:
     respawn Designer once with `rescope_from_audit` hint payload
     containing critique findings; do NOT re-audit. If second
     Concepts also produces BLOCK at Phases tier, BubbleUpHandler
     cascades to Idea (FRAMING substate from B-40).
   - Same pattern at Phases / Steps / BabySteps tiers, dispatching
     to the corresponding decomposer agent.
   - After BabyStep batch authored (≤ 4 parallel per existing
     BubbleUpHandler policy): spawn `Auditor(BabyStepDetail)`.
     BLOCK respawns the babystep-author once.

5. New `LevelAuditVerdict` type:
   ```rust
   pub struct LevelAuditVerdict {
       pub mode: AuditorMode,
       pub tier_node_id: NodeId,        // points into DecompositionTree
       pub findings: Vec<Finding>,
       pub verdict: Verdict,            // ALLOW / BLOCK
       pub respawned_once: bool,        // BLOCK policy: true after second audit on same node
       pub cascade_to_parent: bool,     // true when respawned_once && still BLOCK
   }
   ```
   Persisted to SSB as `per_level_audit_verdict` action_type.

6. AutoAdvanceEvaluator (B-22) extension: split the existing
   `zero_blocks_remaining` check into 5 per-level checks. Each check
   queries `per_level_audit_verdict` rows by `mode` and verifies
   `cascade_to_parent == false` for the current tier. Existing 6 checks
   stay; B-22's contracts are extended (not replaced).

### Phase 3 — Rule cluster seed + dashboard surface (Session 3)

7. Seed rules.db with 5 new domains (each ~10-30 rules at SHADOW status
   per Phase 3F lifecycle):
   - `design_hard`: typography contrast, spacing scale, palette
     accessibility (WCAG AA), component density, font-stack hygiene
   - `app_store`: iOS/Android submission gates (private API ban, ATS
     compliance, deprecated permissions, content rating triggers)
   - `source_tiers`: enforces min source tier per claim category
     (financial=T1, design=T2-T3 OK, marketing=T2+)
   - `slopsquatting`: extends existing deprecated_patterns with
     hallucination-prone API/package patterns
   - `risk_levels`: per-tier risk classification (concept-level
     pivot=HIGH, babystep refactor=LOW)
   Seed entries documented in .elai_cc/crates/rules/seed/ — one TOML
   file per domain.

8. Dashboard surface (V2):
   - `windows/AuditTimelineWindow.svelte` (NEW): timeline of per-level
     verdicts for current task, color-coded by mode, cascade arrows
     drawn when `cascade_to_parent == true`.
   - `MonitorWindow.svelte` adds per-mode "audit X/Y" badges.
   - New stub `get_per_level_verdicts(task_id)` → live Tauri command.

9. Auditor verdict artifact: write to
   `workflow/sessions/{sid}/audits/{mode}/{node_id}.md` per verdict.
   Captures full findings list + respawn payload for later analysis +
   self-mod (B-23 Layer 1A) feedback fuel.

## WIRING_CONTRACT

produces_symbols:
  - elai_orchestrator::auditor_mode::AuditorMode (6-variant enum + Visual reserved)
  - elai_orchestrator::auditor_mode::LevelAuditVerdict
  - elai_orchestrator::auditor_mode::dispatch_level_audit
  - elai_orchestrator::auditor_mode::respawn_with_rescope_hint
  - elai_reasoning::config: 6 mode-specific agent_grc_config_for_role arms
  - rules.db seed for 5 new domains (design_hard, app_store, source_tiers, slopsquatting, risk_levels)
  - 3 SSB action_types: per_level_audit_started, per_level_audit_verdict, per_level_audit_cascade

consumes_symbols:
  - elai_orchestrator::research_tree::DecompositionTree (existing)
  - elai_orchestrator::pipeline_tree_navigator::TreeNavigator (existing — modified)
  - elai_orchestrator::task_runner::BubbleUpHandler (existing)
  - elai_rules::engine::RuleEngine (existing — consumes new domain seeds)
  - B-22 AutoAdvanceEvaluator (extended with 5 per-level checks)
  - B-40 FramedIdea (input to mode=concepts auditor as ground truth)

reachable_from:
  - TreeNavigator dispatch on tier completion (5 insertion points)
  - dashboard-v2 AuditTimelineWindow live consumer

stub_ban:
  - mode dispatch must NOT default to Plan (each tier must be explicit)
  - BLOCK respawn must NOT exceed 1 per node (respawned_once enforced)
  - rule cluster load order must NOT skip any of the 7 (anti-stale, coding-standards, design-hard, app-store, source-tiers, slopsquatting, risk-levels)
  - per-level verdicts must NOT bypass SSB persistence (in-memory only forbidden)

frontend_pair: dashboard-v2 AuditTimelineWindow + MonitorWindow per-mode badges

observable_effect:
  trigger: |
    elai.bat task that produces a 5-Concept decomposition. Inject a
    deliberately bad concept (e.g., "use blockchain for plant ID") in
    the second slot via test fixture.
  expect_ssb: |
    per_level_audit_started(mode=Concepts) →
    per_level_audit_verdict(mode=Concepts, verdict=BLOCK,
                            findings includes C-FEASIBILITY/C-MARKET) →
    Designer respawn (rescope_from_audit) →
    per_level_audit_started(mode=Concepts, respawned_once=true) →
    per_level_audit_verdict(mode=Concepts, verdict=ALLOW) →
    proceed to mode=phases.
  expect_artifact: |
    workflow/sessions/{sid}/audits/Concepts/concept-2.md exists with
    BLOCK verdict and ≥ 1 finding tagged C-FEASIBILITY or C-MARKET.

## DETERMINISTIC_CHECK

```bash
# 1. Schema bumped (28→29 standalone, or 29→30 if B-40 shipped first)
rg "SCHEMA_VERSION: u32 =" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. Three new SSB action_types
for t in per_level_audit_started per_level_audit_verdict per_level_audit_cascade; do
  rg -q "$t" .elai_cc/crates/orchestrator/src/ssb.rs || echo "MISSING: $t"
done

# 3. AuditorMode 6-variant enum (Plan + 5 per-level + Visual reserved feature-gated)
rg -A 8 "enum AuditorMode" .elai_cc/crates/orchestrator/src/auditor_mode.rs | grep -c "^\s*[A-Z][a-zA-Z]*,\?$"
# expected: 6 active + 1 reserved = 7 with Visual under cfg

# 4. .claude/agents/auditor.md has MODE DISPATCH section
rg "^## MODE DISPATCH" .claude/agents/auditor.md

# 5. 5 new rule domains seeded
ls .elai_cc/crates/rules/seed/ | grep -E "(design_hard|app_store|source_tiers|slopsquatting|risk_levels)\.toml" | wc -l
# expected: 5

# 6. respawn-once policy enforced (no double-respawn)
rg "respawned_once" .elai_cc/crates/orchestrator/src/auditor_mode.rs

# 7. AutoAdvanceEvaluator has 5 per-level checks (in addition to B-22's existing 6)
rg "concept_zero_blocks|phase_zero_blocks|step_zero_blocks|babystep_zero_blocks|plan_zero_blocks" .elai_cc/crates/orchestrator/src/task_runner.rs | wc -l
# expected: ≥ 5

# 8. Tests pass
cargo test -p elai-orchestrator auditor_mode 2>&1 | grep "test result"
# expected: ≥ 30 tests (6 mode dispatch + 5 respawn-once + 5 cascade + 6 rule cluster load + 8 SSB persistence)

# 9. AuditTimelineWindow.svelte exists
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/AuditTimelineWindow.svelte

# 10. cargo check + svelte-check clean
cargo check --workspace
cd .elai_cc/crates/dashboard-v2 && npx svelte-check
```

## ACCEPTANCE

- [ ] 6 AuditorMode variants implemented; Visual reserved under cfg
      feature gate
- [ ] All 5 modes (Concepts/Phases/Steps/BabySteps/BabyStepDetail) wire
      into TreeNavigator dispatch points; mode=Plan unchanged
- [ ] BLOCK respawns level-N decomposer exactly ONCE; second BLOCK
      cascades via BubbleUpHandler
- [ ] Rule cluster (7 domains) loaded in fixed order on every mode;
      missing-domain test FAILS as expected
- [ ] AutoAdvanceEvaluator (B-22) integrates 5 per-level checks;
      existing 14-test autonomous_gate_contract.rs still passes
- [ ] B-40 FramedIdea consumed as ground-truth input to mode=Concepts;
      audit on un-framed task emits WARN + falls back to coarse rules
- [ ] Per-level verdicts persisted under workflow/sessions/{sid}/audits/{mode}/
- [ ] dashboard V2 AuditTimelineWindow renders verdicts + cascade arrows;
      no mock-data fallback
- [ ] Ablation: 30-task fixture, measure BabyStep-tier BLOCK rate with
      vs without per-level audits — expect ≥ 40% reduction in
      BabyStep-tier BLOCKs (caught earlier at higher tiers)
- [ ] ≥ 30 tests pass; svelte-check 0 errors; cargo check workspace clean

## Tests to add

- crates/orchestrator/tests/auditor_mode_dispatch.rs: each of 6 modes
  routes correctly through TreeNavigator
- crates/contracts/tests/per_level_audit_contract.rs: BLOCK respawn
  policy (exactly one respawn per node), cascade behavior on second
  BLOCK, SSB persistence
- crates/orchestrator/tests/rule_cluster_load_order.rs: anti-stale
  loaded first, all 7 domains present, missing domain throws not
  silent-skips
- crates/contracts/tests/autonomous_gate_with_per_level_contract.rs:
  B-22 AutoAdvanceEvaluator integration with 5 new evidence checks
- crates/orchestrator/tests/audit_artifact_persistence.rs:
  workflow/sessions/{sid}/audits/{mode}/ structure correct, verdict
  Markdown matches expected format

## Files touched

- CREATE: .elai_cc/crates/orchestrator/src/auditor_mode.rs (~600 LOC)
- CREATE: .elai_cc/crates/orchestrator/tests/auditor_mode_dispatch.rs (~250 LOC)
- CREATE: .elai_cc/crates/contracts/tests/per_level_audit_contract.rs (~200 LOC)
- CREATE: .elai_cc/crates/orchestrator/tests/rule_cluster_load_order.rs (~120 LOC)
- CREATE: .elai_cc/crates/contracts/tests/autonomous_gate_with_per_level_contract.rs (~150 LOC)
- CREATE: .elai_cc/crates/orchestrator/tests/audit_artifact_persistence.rs (~120 LOC)
- CREATE: .elai_cc/crates/dashboard-v2/src/lib/windows/AuditTimelineWindow.svelte (~280 LOC)
- CREATE: .elai_cc/crates/rules/seed/design_hard.toml (~25 rules)
- CREATE: .elai_cc/crates/rules/seed/app_store.toml (~15 rules)
- CREATE: .elai_cc/crates/rules/seed/source_tiers.toml (~10 rules)
- CREATE: .elai_cc/crates/rules/seed/slopsquatting.toml (~30 patterns; reuses A-6 deprecated_patterns format)
- CREATE: .elai_cc/crates/rules/seed/risk_levels.toml (~12 rules)
- CREATE: workflow/reasoning/B-41-decisions.md
- MODIFY: .claude/agents/auditor.md (+ MODE DISPATCH section, mode-specific REVIEW prompts)
- MODIFY: .elai_cc/crates/orchestrator/src/agent_pool.rs (parametric Auditor(AuditorMode))
- MODIFY: .elai_cc/crates/reasoning/src/config.rs (6 mode-specific agent_grc_config_for_role arms)
- MODIFY: .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs (5 audit insertion points)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (5 per-level evidence checks; respawn-once dispatch)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (+3 action_types, schema bump)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (+ get_per_level_verdicts command)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts (+ live invoke for verdicts)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte (+ per-mode badges)

## Decision doc

Write workflow/reasoning/B-41-decisions.md before EXECUTING:
- Why parametric `Auditor(AuditorMode)` over 6 separate AgentRole variants:
  reduces dispatch table size, GRC config delta is the only per-mode
  difference, simpler trait impls.
- Why BLOCK respawns ONCE (not 3 like BubbleUpHandler retries): retrying
  the same auditor on the same decomposer output is wasted compute —
  if respawn fixes nothing, the parent tier is the real fault, cascade.
- Why mode=babystep-detail audits a BATCH (≤4 babysteps) not each
  individually: batches share the same Step parent's pre_gate, so line-
  level findings often correlate; one audit cycle catches them all.
- Why mode=Plan stays as the existing single auditor: backward
  compatibility, plus mode=Plan catches integration-level issues no
  per-tier audit can see.
- Why rule cluster load order is fixed (not configurable): ensures
  deterministic audits across runs (entropy quarantine principle —
  same seed + same input = identical findings).
- Why Visual is reserved (cfg-gated) and not implemented here: B-43 is
  the proper home (depends on B-24 Tier-2 VLM + B-42 cosmos + this B-41).

## Next

B-42 (Cosmos Tier on B-35) — provides the design-trends + competitor-
landscape global artifacts that mode=concepts and mode=phases audits
ground against.
