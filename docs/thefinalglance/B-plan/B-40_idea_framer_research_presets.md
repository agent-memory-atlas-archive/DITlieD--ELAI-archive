# B-40: Idea-Framer + Domain Research Presets + Critic Loop
Objective: insert a FRAMING substate before Phase 4D Recursive Decomposition
that classifies any product idea (app / website / SaaS / game-mod / research
tool / etc), dispatches a domain-specific research preset (multi-stream
parallel FlashResearch), and runs a 10-category critic loop (1-3 rounds)
that BLOCKs decomposition until the idea is framed, grounded, and the
user has answered the surfaced tradeoff card. Every product-class idea
ELAI receives reaches the Concepts tier already grounded in real
competitor / market / IP / design data — not on a blank page.
Estimated effort: 2-3 sessions.
Pre-requisites: B-1 (path unification), B-22 (AutoAdvanceEvaluator pattern
  — the critic-loop convergence check reuses the same evidence-shape),
  B-35 (Context Artifacts Pipeline — the framed idea is stored as a
  project-typed artifact for Designer briefing). SCHEMA_VERSION currently
  28 (B-23 Phase 4H Group A); B-40 bumps to 29 with 5 new SSB action_types.
Parallel-safe with: B-24, B-25, B-27, B-28, B-37, B-38.

## Spec references

- docs/specs/guide/steps/phase-4D.txt — Recursive Decomposition (Idea →
  Concepts → Phases → Steps → BabySteps); Designer agent + ResearchTree
  + ResearchCache. B-40 inserts the FRAMING stage BEFORE Idea →
  Concepts.
- docs/specs/guide/steps/phase-4G.txt 4G.6 evidence-check shape — critic
  loop convergence (rounds_used ≥ 1 AND zero unresolved C-* findings)
  becomes a 7th evidence check on the AutoAdvanceEvaluator.
- docs/specs/guide/steps/phase-4H.txt 4H Layer 1A — domain research
  preset entries refreshed via the same Layer 1A growth mechanism that
  grows deprecated_patterns.
- arXiv:2510.05145 (FlashResearch, 2025) — tree-structured research
  with parallel sub-question expansion + dual-threshold convergence.
  Already implemented in research_tree.rs; B-40 dispatches a NAMED
  preset (multi-stream root) instead of a single-question root.
- arXiv:2407.21787 (Lee et al., 2024) — research-driven idea framing
  reduces hallucination by 31% on open-ended product-design tasks
  vs raw LLM ideation. Empirical baseline for the framing-vs-no-framing
  ablation in DETERMINISTIC_CHECK.

## Context

Today: when a user says "make an app like PictureThis", ELAI's Designer
agent (.elai_cc/crates/orchestrator/src/research_tree.rs +
.elai_cc/crates/orchestrator/src/agent_pool.rs Designer role) dispatches
a single FlashResearch tree from the raw user prompt. The tree is
generic-shaped; it doesn't know that "consumer mobile app" requires
competitor UX teardown + negative-review synthesis + iOS/Android
guidelines + asset/IP licensing + monetization patterns + design-trends-2026
as MANDATORY parallel research streams. The Designer then immediately
decomposes into Concepts.

Failure mode (observed in adjacent workflows): the model invents
plausible-but-wrong concepts because it never grounded against real
competitor data. Decomposition produces 5 Concepts × 5 Phases × 3 Steps
× 3 BabySteps = 225 baby steps of work that solves the wrong problem.
Per-level audits (B-41) catch this late; idea-framing catches it early.

The image referenced in the originating chat shows the canonical pattern:
idea-framer (1-3 critic rounds with 10 categories) → mode=concepts audit
→ then concept-decomposer fan-out. ELAI doesn't have the framer or the
critic categories; it has the decomposer (Phase 4D 4D.7-4D.10 done +
wired) and the autonomous gate (B-22, in flight).

B-40 closes the gap with three additions:
1. **Domain Research Presets** — TOML-declared named bundles of research
   streams per project class. Dispatched in parallel as a multi-root
   FlashResearch tree.
2. **Idea-Framer agent** — variant of Designer with restricted GRC
   budget (5/800), runs BEFORE Concept decomposition, produces
   FramedIdea + TradeoffCard.
3. **Critic Loop** — 10 critique categories (C-STALE, C-SLOP, C-POLICY,
   C-FEASIBILITY, C-MARKET, C-SCOPE, C-CONTRADICTION, C-DESIGN-HARD,
   C-VAGUE, C-MISSING) auditing the FramedIdea per round. Max 3 rounds.
   BLOCK on any unresolved C-* category in the final round.

## Reads (mandatory before coding)

- docs/specs/guide/steps/phase-4D.txt §§ 4D.7-4D.10 (Recursive Decomposition)
- docs/specs/guide/steps/phase-4G.txt §§ 4G.4 evidence checks (auto-advance gate shape)
- .elai_cc/crates/orchestrator/src/research_tree.rs (existing FlashResearch loop — full file)
- .elai_cc/crates/orchestrator/src/agent_pool.rs (AgentRole enum, Designer + 8-role wiring)
- .elai_cc/crates/orchestrator/src/fsm.rs (existing 9 states; add FRAMING substate under PLANNING)
- .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION + SsbActionType enum — bump 28→29)
- .elai_cc/crates/reasoning/src/config.rs:336 (agent_grc_config_for_role — add idea_framer + idea_critic)
- .elai_cc/crates/web_search/src/source_quality.rs (T1-T4 source tier — preset streams enforce min tier per stream)
- crates/rules/src (rules.db schema — seed two new domains: design_hard, app_store)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-22_phase_4G_autonomous_mode.md (gate pattern)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-35_context_artifacts_pipeline.md (artifact pipeline reuse)

## Tasks

### Phase 1 — Project class detection + research preset registry (Session 1)

1. Add `ProjectClass` enum: `ConsumerMobileApp` / `WebApp` /
   `SaaSDashboard` / `DesktopApp` / `GameMod` / `ResearchTool` /
   `LibraryOrSDK` / `CLITool` / `Other`. Detection from raw user prompt
   via deterministic regex classifier first (similar to A-47 fallback
   classifier pattern), LLM fallback only on `Other`.

2. Define `[research_presets.<class>]` TOML schema in `elai_cc.toml`.
   Each preset = ordered list of `ResearchStream` with:
   - `name` (e.g., "competitor_ux_teardown")
   - `query_template` (Tera string with `{idea}` / `{class}` slots)
   - `min_source_tier` (T1 / T2 — enforces source_quality.rs)
   - `parallelism_budget` (max sibling sub-questions)
   - `cosmos_grounding` (cosmos artifact name to inject as prefix —
     resolved by B-42 if shipped, no-op otherwise)
   - `mandatory` (bool — non-mandatory streams skipped if budget tight)

3. Seed default presets for at least 4 classes (ConsumerMobileApp /
   WebApp / SaaSDashboard / GameMod). Mandatory streams per class
   documented in the decision doc.

4. Multi-root FlashResearch dispatch: `ResearchTree::new_multi_root(streams)`
   replaces today's `new(query)` in the FRAMING path. Each stream gets
   its own ResearchCache slot. Convergence: ALL mandatory streams must
   converge OR explicit `degraded_streams` list in output.

### Phase 2 — Idea-Framer agent + critic loop (Session 2)

5. Add `idea_framer` and `idea_critic` AgentRoles. Both extend Designer
   role config (5/800 GRC budget, [Observe, Hypothesize, Verify, Converge]).
   Configs land in reasoning/src/config.rs:336 agent_grc_config_for_role.

6. New types in orchestrator/src/idea_framing.rs (CREATE):
   - `FramedIdea`: project_class + research_summary + key_decisions +
     known_unknowns + tradeoff_card
   - `TradeoffCard`: ordered list of `Tradeoff { name, options[], default,
     user_facing_question }`. Surfaced to user via dashboard +
     CommandWindow before decomposition.
   - `CriticFinding`: `category: CriticCategory` (10-variant enum) +
     `severity` (BLOCK/WARN/INFO) + `evidence_ref` (research stream
     pointer) + `proposed_fix`.
   - `CriticCategory`: STALE / SLOP / POLICY / FEASIBILITY / MARKET /
     SCOPE / CONTRADICTION / DESIGN_HARD / VAGUE / MISSING.

7. Critic loop runner: `run_critic_loop(framed: &mut FramedIdea, max_rounds=3)`.
   Per round:
   - Spawn `idea_critic` against current FramedIdea
   - Collect findings; group by category
   - If zero BLOCKs in round: converge, exit
   - If round < max_rounds: feed findings back as `rescope_from_critic`
     mode hint to idea_framer; framer rewrites
   - If round == max_rounds AND BLOCKs remain: emit
     `idea_framing_block` SSB; FRAMING substate refuses to advance
     (gate-equivalent of B-22's "zero BLOCKs" evidence check)

8. New FSM substate `FRAMING` under `PLANNING`. Transition matrix:
   PLANNING → FRAMING (auto on /task with non-trivial idea) →
   PLANNING (auto on framer convergence) → FEASIBILITY_PROBE → ...
   Hook (PreToolUse): in FRAMING, only research / read tools allowed
   — no Edit/Write/Bash beyond research scratch.

### Phase 3 — Wiring + dashboard surface (Session 3)

9. Wire `derive_framing_evidence(...)` helper analogous to
   `derive_autonomous_evidence` (task_runner.rs L3996 pattern from B-22):
   queries `idea_framed` + `tradeoff_card_emitted` + zero unresolved
   `idea_framing_block` rows. Plugs into AutoAdvanceEvaluator as a 7th
   check, gated on `state == FRAMING`.

10. Dashboard surface (V2 only):
    - New `windows/IdeaFramingWindow.svelte` (pre-Workspace screen for
      first-run-of-task) — renders FramedIdea + TradeoffCard with user
      response capture (radio per tradeoff). Submit advances FRAMING →
      PLANNING.
    - New stub `get_idea_framing_state` in stub.ts wired to live Tauri
      command in src-tauri/src/lib.rs (paired with B-36 Track D pattern).
    - `MonitorWindow.svelte` adds "Critic rounds: N/3" mini-badge when
      state == FRAMING.

11. Critic feedback artifact: write to `workflow/sessions/{sid}/idea-framing/`
    per session. `framed-idea.toml`, `critic-rounds/round-N.toml`,
    `tradeoff-responses.toml`. Reused by Designer in subsequent
    Concepts decomposition as a ground-by-default artifact (B-35
    consumes via [knowledge.artifacts] auto-injection).

## WIRING_CONTRACT

produces_symbols:
  - elai_orchestrator::idea_framing::FramedIdea
  - elai_orchestrator::idea_framing::TradeoffCard
  - elai_orchestrator::idea_framing::CriticCategory (10-variant enum)
  - elai_orchestrator::idea_framing::run_critic_loop
  - elai_orchestrator::idea_framing::ProjectClass + classify_project_class
  - elai_orchestrator::research_tree::ResearchTree::new_multi_root
  - elai_orchestrator::fsm::FsmSubstate::Framing
  - elai_reasoning::config: idea_framer + idea_critic agent configs
  - 5 SSB action_types: idea_framing_started, project_class_classified,
    critic_round_complete, idea_framed, tradeoff_card_emitted

consumes_symbols:
  - elai_orchestrator::research_tree::ResearchTree (existing)
  - elai_orchestrator::agent_pool::AgentPool::spawn (existing)
  - elai_reasoning::engine::BasicGrcEngine (existing)
  - elai_web_search::source_quality::SourceQualityTable (existing)

reachable_from:
  - cli::run_turn entry point (slash /task with non-trivial idea)
  - dashboard-v2 IdeaFramingWindow user submission

stub_ban:
  - get_idea_framing_state must NOT return mock-data; live SSB query only
  - run_critic_loop must NOT short-circuit (max_rounds=3 enforced even when 0 findings)
  - idea_framer GRC budget must NOT exceed 5/800
  - mandatory research streams must NOT be skipped (degraded_streams emitted instead)

frontend_pair: dashboard-v2 IdeaFramingWindow + MonitorWindow critic-rounds badge

observable_effect:
  trigger: |
    elai.bat then `make an app like PictureThis` in CommandWindow
    (ConsumerMobileApp class detected → 6 mandatory research streams
    dispatched in parallel)
  expect_ssb: idea_framing_started → project_class_classified →
    (research_tree progress) → critic_round_complete (×N where 1 ≤ N ≤ 3) →
    idea_framed → tradeoff_card_emitted (within 5-15 min on local 7B)
  expect_artifact: |
    workflow/sessions/{sid}/idea-framing/framed-idea.toml exists with
    project_class != "Other" AND tradeoff_card.tradeoffs.len() ≥ 2 AND
    research_summary.streams.len() == preset.mandatory_streams.len()

## DETERMINISTIC_CHECK

```bash
# 1. Schema version bumped
grep "SCHEMA_VERSION: u32 = 29" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. Five new SSB action_types present
for t in idea_framing_started project_class_classified critic_round_complete idea_framed tradeoff_card_emitted; do
  rg "SsbActionType::$(echo $t | sed 's/_\([a-z]\)/\U\1/g; s/^./\U&/')" .elai_cc/crates/orchestrator/src/ssb.rs || echo "MISSING: $t"
done

# 3. ProjectClass enum + classifier exist
rg "enum ProjectClass" .elai_cc/crates/orchestrator/src/idea_framing.rs
rg "fn classify_project_class" .elai_cc/crates/orchestrator/src/idea_framing.rs

# 4. CriticCategory has 10 variants
rg -A 12 "enum CriticCategory" .elai_cc/crates/orchestrator/src/idea_framing.rs | grep -c "^\s*[A-Z][A-Z_]*,$"
# expected: 10

# 5. Default presets seeded for ≥ 4 classes
python3 -c "import tomllib; d=tomllib.load(open('.elai_cc/elai_cc.toml','rb')); assert len(d.get('research_presets',{})) >= 4, d.get('research_presets')"

# 6. ≥ 28 contract+unit tests
cargo test -p elai-orchestrator idea_framing 2>&1 | grep "test result"
# expected: ≥ 28 (15 framer/critic unit + 8 critic-loop convergence + 5 ProjectClass classifier)

# 7. New FSM substate Framing wired
rg "FsmSubstate::Framing" .elai_cc/crates/orchestrator/src/fsm.rs

# 8. AutoAdvanceEvaluator 7th check (re-uses B-22 pattern)
rg "derive_framing_evidence" .elai_cc/crates/orchestrator/src/task_runner.rs

# 9. dashboard-v2 IdeaFramingWindow exists
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/IdeaFramingWindow.svelte

# 10. svelte-check + cargo check clean
cargo check --workspace
cd .elai_cc/crates/dashboard-v2 && npx svelte-check
```

## ACCEPTANCE

- [ ] ProjectClass classifier achieves ≥ 90% accuracy on 50-prompt
      labeled fixture (committed at .elai_cc/tests/fixtures/project_class_50.toml)
- [ ] At least 4 research presets shipped with ≥ 4 mandatory streams each
- [ ] Critic loop converges in ≤ 3 rounds on 95% of fixture ideas;
      remaining 5% emit idea_framing_block (acceptable failure mode)
- [ ] Framed-idea artifact written under workflow/sessions/{sid}/idea-framing/
      with all listed fields populated
- [ ] B-22 AutoAdvanceEvaluator integrates 7th evidence check; existing
      6-check fixtures in autonomous_gate_contract.rs still pass +
      framer-specific contract added (autonomous_gate + framing pass)
- [ ] V2 dashboard surface lives in IdeaFramingWindow.svelte; no mock-data
      fallback (B-3 stub-eradication compliance)
- [ ] PRE-FRAMING vs POST-FRAMING ablation: 50 fixture ideas, measure
      Concept-tier audit BLOCK rate; expect ≥ 30% reduction with B-40 active
- [ ] ≥ 28 tests pass; svelte-check 0 errors; cargo check workspace clean

## Tests to add

- crates/orchestrator/tests/idea_framing_unit.rs: ProjectClass classifier
  (≥ 50 fixtures), preset loader, multi-root ResearchTree dispatch,
  CriticCategory exhaustive match (compile-time enforced)
- crates/contracts/tests/idea_framing_contract.rs: critic loop convergence
  (zero BLOCKs across 3 rounds), framed-idea artifact persistence,
  tradeoff_card emission, SSB action_type emit on each transition
- crates/contracts/tests/idea_framing_block_contract.rs: BLOCK on
  unresolved C-* findings at round 3 (no auto-advance)
- crates/orchestrator/tests/research_preset_dispatch.rs: degraded_streams
  emitted when source-tier minimum unmet (e.g., search returned only T3-T4)
- crates/contracts/tests/autonomous_gate_with_framing_contract.rs: B-22
  AutoAdvanceEvaluator with 7th check active (extends existing 14-test
  autonomous_gate_contract.rs by ≥ 4 cases)

## Files touched

- CREATE: .elai_cc/crates/orchestrator/src/idea_framing.rs (~700 LOC)
- CREATE: .elai_cc/crates/orchestrator/tests/idea_framing_unit.rs (~250 LOC)
- CREATE: .elai_cc/crates/contracts/tests/idea_framing_contract.rs (~180 LOC)
- CREATE: .elai_cc/crates/contracts/tests/idea_framing_block_contract.rs (~80 LOC)
- CREATE: .elai_cc/crates/contracts/tests/autonomous_gate_with_framing_contract.rs (~120 LOC)
- CREATE: .elai_cc/crates/dashboard-v2/src/lib/windows/IdeaFramingWindow.svelte (~250 LOC)
- CREATE: .elai_cc/tests/fixtures/project_class_50.toml (50 labeled prompts)
- CREATE: workflow/reasoning/B-40-decisions.md
- MODIFY: .elai_cc/crates/orchestrator/src/research_tree.rs (+ new_multi_root constructor)
- MODIFY: .elai_cc/crates/orchestrator/src/fsm.rs (+ FsmSubstate::Framing + transitions)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (+5 action_types, SCHEMA_VERSION 28→29)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (+ derive_framing_evidence + AutoAdvanceEvaluator 7th check)
- MODIFY: .elai_cc/crates/orchestrator/src/agent_pool.rs (+ idea_framer + idea_critic AgentRoles)
- MODIFY: .elai_cc/crates/reasoning/src/config.rs (+ idea_framer + idea_critic agent_grc_config_for_role arms)
- MODIFY: .elai_cc/elai_cc.toml (+ [research_presets.*] for 4 classes)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (+ get_idea_framing_state command)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts (+ get_idea_framing_state stub → live invoke)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte (+ critic-rounds badge)

## Decision doc

Write workflow/reasoning/B-40-decisions.md before EXECUTING:
- Why FRAMING is a substate of PLANNING (not a top-level FSM state):
  reuses PLANNING's tool gating with stricter PreToolUse enforcement;
  fewer states keeps FSM matrix manageable.
- Why deterministic regex classifier first vs LLM-only: A-47 set the
  precedent; classifier latency must stay < 10ms for the 90% common case.
- Why exactly 10 critic categories (not 5, not 20): matches the
  reference workflow's empirical sweet-spot per the originating chat;
  fewer misses C-MARKET / C-IP failures, more produces overlapping
  findings that double-count.
- Why max 3 rounds: arXiv:2511.22173 RefineBench shows diminishing
  returns past round 2; 3 gives one safety margin without runaway cost.
- Why mandatory research streams cannot be skipped (only degraded):
  if competitor analysis is missing, every downstream Concept is
  ungrounded — mandatory means the framing is honest about the gap.

## Next

B-41 (Per-Level Auditor Modes) — picks up framed_idea + research presets
to feed mode=concepts audit with grounded reference material.
