# F-6: Learned-skill library: trust ladder + critic-gated acquisition + staleness demotion

Objective: give ELAI a learned-skill library that is ACQUIRED via a critic gate, GOVERNED by an
execution-autonomy trust ladder (SkillTrustTier S1-S4, distinct from the memory provenance TrustTier
T0-T3), and DEMOTED on staleness — wired so a sub-top-tier skill CANNOT run in autonomous mode without
a human gate, and so the library cannot ship wire-dark. This is cluster C6 of the VibeSec/SHARD
steal-and-build synthesis, in its REVISED (decomposed) scope: combine SHARD's T1-T4 trust ladder +
Voyager's critic-gated acquisition loop, the two halves neither system ships alone, on top of ELAI's
already-built TemplateCrystallizer + BestOfN MiniCritic + GraphCodeBERT/Granite reranker retrieval.

Estimated effort: 3-4 sessions (~14-18 h), decomposed C6a -> C6b -> C6c (each gated on the prior).
Tier: 3 (governance + learning loops; sequencing-dependent per F-0 SEQUENCING). All three sub-phases
are in-process / fail-closed (no network, no subprocess, no RunPod at gate time) — F-6 is wholly TIER 1
on the W-10 axis EXCEPT the optional embedding-retrieval recall measurement (see W-10 split below).

Pre-requisites:
  - C-25 (TemplateCrystallizer) — SHIPPED. `crates/orchestrator/src/compiled_pipeline.rs::TemplateCrystallizer`
    (struct compiled_pipeline.rs:292) is the acquisition substrate F-6 extends. CONFIRMED in elai.map.
  - E-43 (skill telemetry) — SHIPPED. `crates/orchestrator/src/skill_telemetry.rs` + `SkillSystemEventPayload`
    + `SsbActionType::SkillSystemEvent` (ssb.rs:9326) is the telemetry precedent F-6 composes with (the new
    SkillLibraryEvent is a DISTINCT action_type, NOT a reuse of skill_system_event). CONFIRMED in elai.map.
  - C-32 BestOfN/MiniCritic — SHIPPED. `crates/orchestrator/src/mini_critic.rs::{MiniCritic, MiniCriticVerdict,
    MiniCriticResult}` + `structural_retry.rs::run_best_of_n` (structural_retry.rs:277) — F-6 REUSES the existing
    verdict, it does NOT add a fresh critic call. CONFIRMED in elai.map.
  - Phase 4G autonomous gate — SHIPPED. `crates/orchestrator/src/autonomous_gate.rs::TreeAutoAdvancePolicy::
    should_auto_advance` (autonomous_gate.rs:178). F-6 wires S<top-tier into this gate. CONFIRMED in elai.map.

Parallel-safe with: F-5 (staleness cascade — F-5 owns memory_facts + DKG concept_stale; F-6 owns a DISTINCT
skill-staleness scalar on SkillTrustTier metadata; no shared column. Coordinate only the decay.rs scheduler-tick
edit window). NOT parallel-safe with any plan touching `compiled_pipeline.rs::check_crystallization` or
`structural_retry.rs::run_best_of_n` (F-6 modifies both).

Source: workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md §4 item C6 (full grounded mechanism +
the REVISED SCOPE decompose C6a/b/c) + §1.5 (Voyager 3.3x non-transferable) + §3 Pillar 3 + §5 TIER 3 + §7
"Do NOT collapse SkillTrustTier into the existing memory TrustTier" + "Do NOT add a Voyager-style acquisition
critic as a fresh LLM call".

## Context

SHARD ships a T1-T4 skill trust ladder (skills.py) and Voyager ships a critic-gated acquisition loop
(curriculum -> generate -> execute -> CRITIC GATE -> retry<=4 -> commit with description-embedding retrieval),
but NEITHER ships both, and neither gates EXECUTION AUTONOMY by trust tier. ELAI already has the substrate that
makes the combination cheap: TemplateCrystallizer crystallizes a CompiledPipeline after N consecutive successes
(compiled_pipeline.rs:335 `check_crystallization` returns `Option<CompiledPipeline>`); BestOfN already runs a
MiniCritic verdict INSIDE structural_retry; the GraphCodeBERT/Granite + reranker retrieval already exists in
`crates/knowledge/src/reranker.rs`. The NEW ELAI capability is the GOVERNANCE wrapper: a learned skill,
acquired only when an already-verified critic verdict approved the artifact, retrieved by the existing reranker,
governed by a trust tier that GATES AUTONOMY (a sub-top-tier skill may not run in autonomous mode without a
human gate), and demoted on staleness.

The synthesis records three corrections that F-6 MUST honor and NOT re-introduce (§1.5 + §7):
  1. Voyager's "3.3x more items" is a Minecraft tech-tree number on a frozen 2024-04 codebase. It is ZERO
     evidence for coding-task skill-reuse. F-6 makes NO reuse-rate claim; the measurable is deferred until >=20
     real acquisitions are logged (§4 C6 "Defer the 70%-reuse measurable claim").
  2. The existing memory TrustTier (T0UserDirect..T3ModelEmitted, memory_trust_contract.rs) is PROVENANCE, not
     execution-autonomy. SkillTrustTier is a DISTINCT Rust enum with DISTINCT SSB wire tags (S1-S4, never T0-T3),
     enforced by an auditor type-barrier contract test so S2 cannot silently coerce to memory T2.
  3. ELAI already runs MiniCritic inside BestOfN. A naive acquisition critic would build a redundant
     TRIPLE-critic. C6b REUSES the verdict already in the SSB row that triggered crystallization — it pins the
     acquisition gate to an already-verified artifact, adding ZERO new LLM calls.

This is a TIER 3 governance/learning-loop session per F-0; it is in-process and fail-closed (the trust ladder and
the staleness scalar are pure computation), so on the W-10 axis F-6 is genuinely fail-closed — the only Tier-2
(measurement-only, never a gate) piece is the optional reranker recall A/B in C6b, which is split out per W-10.

## Existing state (B/C/D/E coverage)

- C-25 TemplateCrystallizer (SHIPPED): `compiled_pipeline.rs:292` struct + `record_execution` (line 318) +
  `check_crystallization<F>(&self, concept_class, plan_skeleton_hash, step_template_provider, now_secs) ->
  Option<CompiledPipeline>` (line 335). Crystallizes on N consecutive successes (CRYSTALLIZATION_THRESHOLD_N).
  CompiledPipeline carries `concept_class`, `plan_skeleton_hash`, `use_count`, `first_try_success_rate`,
  `compilation_eligible`. Constructed at `runtime_setup.rs:316` and held as `Arc<AsyncMutex<TemplateCrystallizer>>`
  in task_runner (task_runner.rs:3019). C6b extends the crystallization gate; C6a/C6c attach metadata to the
  crystallized artifact. CONFIRMED in elai.map.
- C-32 BestOfN MiniCritic (SHIPPED): `mini_critic.rs::{MiniCritic, MiniCriticVerdict (Pass/Warn/Block),
  MiniCriticResult}`, `MiniCriticVerdictPayload` (ssb.rs:15776), `mini_critic_verdict` action_type (ssb.rs:9010).
  `structural_retry.rs::run_best_of_n` (line 277) calls `critic.check(blank_id, body, lang)` and emits the
  verdict SSB row. task_runner.rs:7685 documents the pipeline `Decompose -> Template -> FewShot -> BestOfN ->
  MiniCritic per-blank gate`. C6b consumes this verdict row, NEVER a fresh critic. CONFIRMED in elai.map.
- E-43 skill telemetry (SHIPPED): `skill_telemetry.rs` (PostToolUse helper), `SkillEventPhase` enum
  (ssb.rs:20744), `SkillSystemEventPayload`, `SsbActionType::SkillSystemEvent` (ssb.rs:9326), CLI `skills-mgmt`
  (args.rs:1875 -> `args_skills::parse_skills_subcommand_args`). F-6's SkillLibraryEvent is DISTINCT from
  skill_system_event (E-43 = per-invocation telemetry of HARNESS skills; F-6 = lifecycle of LEARNED skills).
  CONFIRMED in elai.map.
- memory TrustTier (SHIPPED): `crates/memory` TrustTier {T0UserDirect, T1CodebaseDerived, T2WebSourced,
  T3ModelEmitted}, wire tags "T0".."T3", `is_safety_critical_allowed`, `TierCeiling`
  (memory_trust_contract.rs). DISTINCT from SkillTrustTier — the type-barrier contract test (C6a) asserts they
  never cross. CONFIRMED in elai.map.
- Phase 4G autonomous gate (SHIPPED): `autonomous_gate.rs::TreeAutoAdvancePolicy::should_auto_advance(level,
  effective_risk) -> bool` (line 178); `Evidence` struct (line 208); `task_runner.rs:6516`
  `let autonomous_mode = if let Some(gate) = resolved.autonomous_gate.as_ref()`. C6a wires the S-tier check as
  an additional gate clause. CONFIRMED in elai.map.
- Reranker retrieval (SHIPPED): `crates/knowledge/src/reranker.rs` + `pipeline.rs` (GraphCodeBERT/Granite +
  cross-encoder reranker, `RetrievalResult`). C6b retrieves learned skills by description-embedding via this
  existing path — F-6 adds NO new embedding model and NO new dep. CONFIRMED in elai.map.
- SSB Pattern-2 collapse (SHIPPED precedent): E-43 SkillSystemEvent is the 16th SINGLE-VARIANT Pattern-2
  application (ssb.rs:1888); RouteDecisionEvent the 43rd (ssb.rs:23884, 3-arm RouteDecisionEventKind). F-6 uses
  the same collapse: one `SkillLibraryEvent` variant + a 3-arm `SkillLibraryEventKind` discriminator. CONFIRMED.
- Dashboard pattern (SHIPPED): `CriticDecisionPanel.svelte`, `MonitorWindow.svelte`, `DrawerKey` +
  `workspace.svelte.ts::toggleDrawer` (line 374), tauri `commands.rs`. F-6's SkillLibraryPanel mirrors
  CriticDecisionPanel. CONFIRMED in elai.map.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\reasoning\VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md` §4 C6, §1.5, §3 Pillar 3,
  §5 TIER 3, §7 (the four C6 "Do NOT" clauses)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\compiled_pipeline.rs:285-430` (TemplateCrystallizer +
  check_crystallization + CompiledPipeline shape)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\structural_retry.rs:226-320` (run_best_of_n + the
  MiniCriticResult verdict path C6b reuses)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\mini_critic.rs:240-270` (the verdict + the SSB emit shape)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\autonomous_gate.rs:160-220` (TreeAutoAdvancePolicy +
  should_auto_advance + Evidence)
- `F:\.ELAI_workflow\.elai_cc\crates\memory\tests\contracts\memory_trust_contract.rs:39-115` (the memory
  TrustTier wire tags + is_safety_critical_allowed — the type-barrier C6a must NOT collide with)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\skill_telemetry.rs:1-60` (E-43 telemetry shape for the
  distinct-action-type contrast)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\reranker.rs` (retrieval entry; read interface only)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs:15291` (TemplateCrystallizedPayload as the
  Pattern-2 single-variant payload shape to mirror) + ssb.rs:1888 (the E-43 Pattern-2 16th-application note)
- SHARD `00-COMPARISON-shard-vs-priorart.md` Pillar 3 (the T1-T4 ladder + Voyager critic-gate it borrows from)

## Tasks

### C6a — SkillTrustTier governance (ship first; low collision risk; shippable standalone)

1. **Create `crates/orchestrator/src/skill_trust.rs` (~260 LOC).** New module. Exposes:
   - `pub enum SkillTrustTier { S1Candidate, S2Probationary, S3Established, S4Trusted }` with `as_str` returning
     `"S1".."S4"` (DISTINCT wire tags from memory TrustTier's "T0".."T3" — auditor-enforced, Task 5). Derive
     Debug/Clone/Copy/PartialEq/Eq/Serialize/Deserialize, `#[serde(rename_all = "snake_case")]`. NO `From<TrustTier>`
     and NO `Into<TrustTier>` impl in either direction (the type barrier).
   - `pub struct LearnedSkill { skill_id: Uuid, concept_class: String, plan_skeleton_hash: u64, tier:
     SkillTrustTier, invocation_count: u32, failure_count: u32, acquired_at_secs: u64, last_verified_secs: u64,
     description_embedding_ref: Option<String>, critic_verdict_ssb_id: Option<i64> }`. `description_embedding_ref`
     is the reranker retrieval handle (C6b); `critic_verdict_ssb_id` pins the acquisition to the already-verified
     BestOfN row (C6b).
   - `pub fn promote(&mut self, now_secs: u64) -> Option<SkillTrustTier>` implementing the SHARD-style ladder:
     promote by `invocation_count >= TIER_MIN_INVOCATIONS[next]` AND `failure_rate <= TIER_MAX_FAILURE_RATE[next]`
     AND `(now_secs - acquired_at_secs) >= TIER_MIN_AGE_SECS[next]`. S4 (top) is HUMAN-GATED — `promote` returns
     `Some(S4Trusted)` only when an explicit `human_promotion_granted: bool` is set; otherwise the ladder caps at
     S3. All thresholds are NAMED constants (no magic numbers per CLAUDE.md): `const TIER_MIN_INVOCATIONS:
     [u32; 4]`, `const TIER_MAX_FAILURE_RATE: [f32; 4]`, `const TIER_MIN_AGE_SECS: [u64; 4]`.
   - `pub fn permits_autonomous(&self) -> bool { matches!(self.tier, SkillTrustTier::S4Trusted) }` — ONLY S4 may
     run in autonomous mode without a human gate. This is the autonomy barrier C6a wires into Task 3.
   - No `.unwrap()`/`.expect()` on any production path; use `u32::try_from(...).unwrap_or(u32::MAX)` style and
     `Option`/`Result` propagation. blake3 for any skill-content hash (never blake2/sha2).

2. **Add `SkillLibraryEvent` SSB variant + 3-arm Kind (~90 LOC in `crates/orchestrator/src/ssb.rs`).**
   Pattern-2 SINGLE-VARIANT collapse (per E-43 precedent ssb.rs:1888): one `SsbActionType::SkillLibraryEvent`
   + `pub enum SkillLibraryEventKind { Acquired, TrustPromoted, DemotedStale }` (3-arm discriminator) +
   `pub struct SkillLibraryEventPayload { kind: SkillLibraryEventKind, skill_id: String, concept_class: String,
   from_tier: Option<String>, to_tier: Option<String>, critic_verdict_ssb_id: Option<i64>, staleness_score:
   Option<f32>, skill_content_blake3: String }`. Add: the `sql_str` value `"skill_library_event"`, the parse arm
   (ssb.rs ~9300 region, alongside `"skill_system_event" => Ok(SsbActionType::SkillSystemEvent)`), the SQL CHECK
   constraint entry, the round-trip golden fixture, and the action_type_lock.golden row. SCHEMA_VERSION: baseline
   + 1 (the three wire-facing tags skill_acquired / skill_trust_promoted / skill_demoted_stale are emitted as the
   3-arm Kind under the single SkillLibraryEvent variant — that is what the W-9 "3 action types" requirement maps
   to without 3 separate SsbActionType arms). `action_type_lock_golden_matches_runtime` (action_type_lock.rs:2167)
   MUST stay green.

3. **Wire the S-tier autonomy barrier into the autonomous gate (~70 LOC delta in
   `crates/orchestrator/src/autonomous_gate.rs` + the call site in `task_runner.rs`).** Add
   `pub fn skill_permits_autonomous(skill: &crate::skill_trust::LearnedSkill) -> bool` and call it as an
   ADDITIONAL clause where the auto-advance decision is taken (task_runner.rs:6516
   `let autonomous_mode = if let Some(gate) = resolved.autonomous_gate.as_ref()`): when a compiled/learned skill is
   the candidate executor for an autonomous step, `should_auto_advance(...) && skill.permits_autonomous()`. If the
   skill is sub-S4, the step does NOT auto-advance — it routes to the human gate (the existing interactive path),
   and an SSB `SkillLibraryEvent{kind: TrustPromoted? no — emit a gated marker}` is NOT needed here; the BLOCK is
   observable via the existing auto-advance-denied path. This is the W-2 production wiring: the barrier is ON by
   default; a kill-switch `ELAI_SKILL_AUTONOMY_BARRIER_DISABLE=1` may DISABLE it, but default is ENFORCED.

4. **CLI surface in `crates/cli/src/main.rs` + `crates/cli/src/args_skills.rs` (~120 LOC).** Extend the existing
   `skills-mgmt` subcommand (args.rs:1875) with three learned-skill verbs: `elai skills-mgmt library list`
   (prints each LearnedSkill with tier), `elai skills-mgmt library promote <skill_id>` (attempts a ladder
   promotion, emits `SkillLibraryEvent{TrustPromoted}` on success; S4 requires `--human-grant`), and
   `elai skills-mgmt library show <skill_id>`. This is the W-3 production entrypoint + the W-5 observable_effect
   trigger.

5. **Auditor type-barrier contract test (~110 LOC at
   `crates/contracts/tests/skill_trust_type_barrier_contract.rs`).** Assert: (a) `SkillTrustTier::S2Probationary
   .as_str() == "S2"` and is NEVER `"T2"`; (b) there is no `From<SkillTrustTier> for TrustTier` and no reverse
   (compile-fail asserted by a trybuild-style negative OR by an explicit "the conversion does not exist" doc-test
   — verify which the repo already uses); (c) `permits_autonomous()` is true ONLY for S4Trusted; (d) the
   golden-lock round-trip for SkillLibraryEventPayload preserves the Kind discriminator. The auditor MUST run this
   as part of the W-1 gate.

### C6b — critic-gated acquisition REUSING the BestOfN MiniCritic verdict (after C6a)

6. **Extend `TemplateCrystallizer::check_crystallization` with a critic gate that REUSES the existing verdict
   (~130 LOC delta in `crates/orchestrator/src/compiled_pipeline.rs`).** When the N-consecutive-success threshold
   fires and a CompiledPipeline would be produced, BEFORE materializing the LearnedSkill, look up the
   `mini_critic_verdict` SSB row(s) (ssb.rs:9010) emitted by `run_best_of_n` for the supersteps that produced
   those successes (matched by superstep/agent_id), and require the verdict was `MiniCriticVerdict::Pass` (NOT
   Warn, NOT Block) for the crystallizing artifact. This adds ZERO new LLM/critic calls — it READS the verdict
   already in the SSB row (§7 "reuse the BestOfN MiniCritic verdict already in the SSB row"). On gate pass:
   construct a `LearnedSkill` at tier `S1Candidate` with `critic_verdict_ssb_id` set, store it, and emit
   `SkillLibraryEvent{kind: Acquired, critic_verdict_ssb_id: Some(...)}`. On gate fail (verdict was Warn/Block or
   absent): do NOT acquire (return None as today). Add a NEW method
   `pub async fn acquire_learned_skill(&mut self, pipeline: &CompiledPipeline, critic_verdict_ssb_id: i64,
   now_secs: u64) -> Option<LearnedSkill>` so the gate is unit-testable.

7. **Retrieve learned skills via the existing reranker (~80 LOC delta, `crates/orchestrator` calling
   `crates/knowledge/src/reranker.rs`).** At plan-execution start, when a concept_class is about to run, query
   the learned-skill library by description-embedding through the EXISTING GraphCodeBERT/Granite + reranker
   `RetrievalResult` path (no new embedding model, no new dep per F-0 hard rules) and, if a matching S>=S3 skill
   exists, offer its CompiledPipeline as the candidate. Set `description_embedding_ref` on acquisition (Task 6).
   The retrieval itself is in-process over the already-indexed store. (W-10 split: any RECALL-MEASUREMENT A/B over
   this retrieval is Tier-2 measurement-only and is NOT a gate — see W-10 section; the retrieval USE is Tier-1.)

8. **Acquisition contract test (~90 LOC at
   `crates/contracts/tests/skill_critic_gate_contract.rs`).** (a) crystallization with a stored
   `MiniCriticVerdict::Pass` row -> `acquire_learned_skill` returns Some(S1) + emits Acquired; (b) crystallization
   with a `MiniCriticVerdict::Block` row -> returns None, no Acquired row (NO triple-critic; the gate is the reused
   verdict); (c) crystallization with NO matching verdict row -> returns None (fail-closed, never acquire ungated).

### C6c — staleness demotion (after C6a metadata + C6b acquisition exist)

9. **Staleness scalar + demotion on the scheduler tick (~120 LOC in `skill_trust.rs` + a hook on the existing
   background scheduler).** Add `pub fn staleness_score(&self, now_secs: u64) -> f32 { (now_secs -
   self.last_verified_secs) as f32 / SKILL_EXPECTED_LIFETIME_SECS }` (SHARD time-decay shape; NAMED const) and
   `pub fn demote_if_stale(&mut self, now_secs: u64) -> Option<SkillTrustTier>` that drops one tier when
   `staleness_score > SKILL_STALE_THRESHOLD` and emits `SkillLibraryEvent{kind: DemotedStale, staleness_score:
   Some(...)}`. Wire `demote_if_stale` into the SAME background scheduler tick that drives the existing decay
   scheduler (coordinate with F-5's decay.rs edit window — F-6 owns a DISTINCT skill scalar, not memory_facts).
   This is reachable_from a real production entrypoint (the scheduler runs in the live orchestrator), satisfying
   W-3 without a test-only caller; it is ALSO triggerable on-demand via `elai skills-mgmt library tick`
   (CLI, Task 4) for the W-5 deterministic observable_effect.

10. **Frontend pair: SkillLibraryPanel (~280 LOC svelte + ~140 LOC tauri).** New
    `crates/dashboard-v2/src/lib/components/SkillLibraryPanel.svelte` (mirrors CriticDecisionPanel.svelte) showing
    each LearnedSkill's tier (S1-S4), invocation/failure counts, staleness score, and a live feed of
    SkillLibraryEvent rows (Acquired / TrustPromoted / DemotedStale). New tauri command
    `get_skill_library(state) -> CmdResult<Value>` in `crates/dashboard-v2/src-tauri/src/commands.rs` reading the
    learned-skill store + the SkillLibraryEvent SSB rows into a typed DTO. Register the panel in `MonitorWindow.svelte`
    (alongside CriticDecisionPanel) OR add a `DrawerKey 'skill-library'` via `workspace.svelte.ts::toggleDrawer`
    — confirm which surface the dashboard uses at authoring time and use that. `cross_lang_diff.py` MUST pass.

11. **Staleness/demotion contract test (~90 LOC at
    `crates/contracts/tests/skill_staleness_demotion_contract.rs`).** Mock-clock: a skill at S3 whose
    `last_verified_secs` is `SKILL_EXPECTED_LIFETIME_SECS * (SKILL_STALE_THRESHOLD + 0.1)` in the past ->
    `demote_if_stale` drops it to S2 + emits DemotedStale; a fresh skill is untouched; the demotion respects the
    S1 floor (cannot demote below S1).

12. **Decision doc at `workflow/reasoning/F-6-decisions.md`** covering: (a) SkillTrustTier is a DISTINCT enum
    from memory TrustTier with auditor-enforced type barrier (§7); (b) the acquisition critic REUSES the BestOfN
    verdict, adding zero LLM calls (§7); (c) Voyager 3.3x is non-transferable, NO reuse-rate claim is made, the
    measurable is deferred until >=20 acquisitions (§1.5 + §4 C6); (d) S4 is human-gated; sub-S4 cannot autonomous;
    (e) the staleness scalar is DISTINCT from F-5's memory decay (no shared column).

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/skill_trust.rs::SkillTrustTier
  - crates/orchestrator/src/skill_trust.rs::LearnedSkill
  - crates/orchestrator/src/skill_trust.rs::promote
  - crates/orchestrator/src/skill_trust.rs::permits_autonomous
  - crates/orchestrator/src/skill_trust.rs::staleness_score
  - crates/orchestrator/src/skill_trust.rs::demote_if_stale
  - crates/orchestrator/src/task_runner.rs::acquire_learned_skill
  - crates/orchestrator/src/autonomous_gate.rs::skill_permits_autonomous
  - crates/orchestrator/src/ssb.rs::SkillLibraryEventPayload
  - crates/orchestrator/src/ssb.rs::SkillLibraryEventKind
  - crates/orchestrator/src/ssb.rs::SsbActionType::SkillLibraryEvent
  - crates/cli/src/args_skills.rs::parse_skills_library_args
  - crates/dashboard-v2/src/lib/components/SkillsPanel.svelte
  - crates/dashboard-v2/src-tauri/src/commands_f6.rs::get_skill_library
consumes_symbols:
  - crates/orchestrator/src/compiled_pipeline.rs::TemplateCrystallizer (check_crystallization extended; record_execution read)
  - crates/orchestrator/src/compiled_pipeline.rs::CompiledPipeline (concept_class, plan_skeleton_hash, use_count read)
  - crates/orchestrator/src/mini_critic.rs::MiniCriticVerdict (the reused Pass/Warn/Block verdict)
  - crates/orchestrator/src/structural_retry.rs::run_best_of_n (the SSB mini_critic_verdict row it emits)
  - crates/orchestrator/src/autonomous_gate.rs::should_auto_advance (S-tier barrier added as a clause)
  - crates/orchestrator/src/task_runner.rs (autonomous_mode call site at task_runner.rs:6516)
  - crates/knowledge/src/reranker.rs::RetrievalResult (description-embedding retrieval of learned skills)
  - crates/orchestrator/src/args.rs::parse_skills_subcommand_args (skills-mgmt extended)
reachable_from:
  - `elai skills-mgmt library promote <skill_id>` (CLI, args_skills.rs) -> SkillTrustTier::promote -> SkillLibraryEvent{TrustPromoted} row in ssb.db
  - `elai skills-mgmt library tick` (CLI) -> LearnedSkill::demote_if_stale -> SkillLibraryEvent{DemotedStale} row in ssb.db
  - TemplateCrystallizer::check_crystallization (live orchestrator, runtime_setup.rs:316 Arc) fires on N successes -> acquire_learned_skill -> SkillLibraryEvent{Acquired} row in ssb.db
  - task_runner.rs:6516 autonomous_mode decision -> skill_permits_autonomous -> sub-S4 learned skill routed to human gate (auto-advance denied)
  - SkillsPanel.svelte -> get_skill_library tauri command (commands_f6.rs) -> learned-skill store + SkillLibraryEvent rows (dashboard)
stub_ban:
  - "TODO: skill tier later" literal absent
  - "// trust ladder stub" literal absent
  - FORBIDDEN: ELAI_SKILL_LIBRARY_ENABLE default-off as the terminal state. The library + the S-tier autonomy
    barrier are ON by default; only a kill-switch ELAI_SKILL_AUTONOMY_BARRIER_DISABLE=1 may DISABLE the barrier,
    and the observable_effect MUST run with NO env override set (W-2). A default-off terminal state is a BLOCK.
  - FORBIDDEN: any LLM-bypass branch — the S-tier autonomy barrier and the critic gate are compiled-code
    decisions; the model cannot disable the barrier or fabricate a Pass verdict via prompt. No
    `if prompt_says_skip { permit }` branch. No fresh in-band LLM critic call (that would be the triple-critic).
  - No `From<SkillTrustTier> for TrustTier` (the type barrier) — its presence is a BLOCK.
  - No silent acquisition without a reused MiniCriticVerdict::Pass row (fail-closed: absent verdict = no acquire).
frontend_pair:
  svelte: crates/dashboard-v2/src/lib/components/SkillsPanel.svelte (tiers + acquisitions + staleness feed)
  tauri: crates/dashboard-v2/src-tauri/src/commands_f6.rs::get_skill_library
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- skills-mgmt library promote <skill_id> --human-grant ; then cargo run -p elai-cli -- skills-mgmt library tick"
  expect_ssb: "a skill_library_event row appears in ssb.db with kind=trust_promoted (from the promote) AND a second row with kind=demoted_stale OR an acquired row from a prior crystallization — at least one SkillLibraryEvent row written within the same session, NO env override set"
  expect_artifact: "stdout from `skills-mgmt library list` shows the skill at its new tier (regex: 'S[1-4]\\s+.*<skill_id>'); SkillLibraryPanel.svelte renders the tier + the event in the live feed"

## DETERMINISTIC_CHECK

```bash
# W-1 wiring done-gate (the literal core gate line — RUN_TRIGGER must PASS)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/F-plan/F-6_learned_skill_library.md --run-trigger

# Produced symbols exist
test -f .elai_cc/crates/orchestrator/src/skill_trust.rs
rg "enum SkillTrustTier|struct LearnedSkill|fn permits_autonomous|fn demote_if_stale|fn staleness_score" .elai_cc/crates/orchestrator/src/skill_trust.rs
rg "fn acquire_learned_skill" .elai_cc/crates/orchestrator/src/compiled_pipeline.rs
rg "fn skill_permits_autonomous" .elai_cc/crates/orchestrator/src/autonomous_gate.rs

# SSB action type + Pattern-2 Kind + the 3 wire tags
rg "SkillLibraryEvent|SkillLibraryEventKind|SkillLibraryEventPayload" .elai_cc/crates/orchestrator/src/ssb.rs
rg "skill_library_event" .elai_cc/crates/orchestrator/src/ssb.rs
rg "Acquired|TrustPromoted|DemotedStale" .elai_cc/crates/orchestrator/src/ssb.rs
rg "skill_library_event" .elai_cc/crates/orchestrator/tests/action_type_lock.rs

# Type barrier: the forbidden conversion must NOT exist
! rg "From<SkillTrustTier> for TrustTier|impl From<SkillTrustTier>" .elai_cc/crates/

# CLI entrypoint wired
rg "library list|library promote|library tick|library show" .elai_cc/crates/cli/src/args_skills.rs

# Default-ON: no terminal default-off gate (kill-switch only)
! rg "ELAI_SKILL_LIBRARY_ENABLE" .elai_cc/crates/orchestrator/src/skill_trust.rs
rg "ELAI_SKILL_AUTONOMY_BARRIER_DISABLE" .elai_cc/crates/orchestrator/src/autonomous_gate.rs

# Frontend pair + cross-language drift
test -f .elai_cc/crates/dashboard-v2/src/lib/components/SkillLibraryPanel.svelte
rg "get_skill_library" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
python .claude/tools/cross_lang_diff.py

# Compile-only (auditor runs the suite; main LLM never runs tests)
cd .elai_cc && cargo check --workspace
```

## ACCEPTANCE

- [ ] **W-1 wiring done-gate**: `python .claude/tools/wiring_contract_check.py --bn-file
      docs/thefinalglance/F-plan/F-6_learned_skill_library.md --run-trigger` returns PASS (exit 0) — every
      produces_symbol resolves to a DEF/SCIP_DEF, every consumes_symbol has a real CALL/REF, every reachable_from
      anchor is reached by BFS over CALL_RESOLVED, stub_ban finds zero forbidden literals, and the
      observable_effect trigger RAN and emitted the SkillLibraryEvent row + artifact.
- [ ] **W-2 default-ON**: the observable_effect trigger (`skills-mgmt library promote` + `library tick`) runs and
      emits the SkillLibraryEvent row with NO `ELAI_*` env override set; the S-tier autonomy barrier is ENFORCED by
      default; only `ELAI_SKILL_AUTONOMY_BARRIER_DISABLE=1` disables it. There is NO default-off terminal state.
- [ ] **W-6 dashboard E2E**: through the dashboard, open SkillLibraryPanel, trigger a `library promote`, and SEE
      the tier change + the TrustPromoted event render in the live feed (verified via dashboard-smoker:
      svelte-check + cargo check + the panel renders the get_skill_library DTO).
- [ ] C6a: SkillTrustTier S1-S4 with wire tags "S1".."S4"; the auditor type-barrier contract passes (S2 != T2; no
      From<SkillTrustTier> for TrustTier); `permits_autonomous` true ONLY for S4.
- [ ] C6b: `acquire_learned_skill` acquires ONLY when a reused `MiniCriticVerdict::Pass` SSB row exists; a Block/
      absent verdict yields no acquisition (no triple-critic; fail-closed).
- [ ] C6c: mock-clock staleness drops a stale S3 skill to S2 + emits DemotedStale; fresh skills untouched; S1 floor
      respected.
- [ ] SkillLibraryEvent round-trips via the SSB golden + `action_type_lock_golden_matches_runtime` stays green;
      SCHEMA_VERSION = baseline + 1 (named in the WIRING_CONTRACT).
- [ ] `cross_lang_diff.py` PASS (no Rust<->TS drift on the SkillLibraryPanel pair).
- [ ] `cargo check --workspace` GREEN; auditor PASS on `-p elai-orchestrator -p contracts` for the three new
      contract tests.
- [ ] No new external dependency (anti-slopsquatting per CLAUDE.md; F-6 reuses the existing reranker + serde +
      blake3 + uuid). No new dep added without user confirmation.
- [ ] **DEFERRED measurable (NOT a release gate this session)**: the skill-reuse-rate number is NOT claimed (Voyager
      3.3x non-transferable, §1.5); it is deferred until >=20 real acquisitions are logged. This checkbox records
      the deferral, it does not assert a number.

## WIRING DONE-GATE

This section states, clause by clause, how F-6 satisfies W-1..W-10 so the implementer cannot ship it wire-dark.

- **W-1 (no-wire-dark done-gate)**: the DETERMINISTIC_CHECK leads with the literal
  `wiring_contract_check.py --run-trigger` line; the FSM may NOT reach COMPLETE until it returns PASS. cargo-green
  is necessary but NOT sufficient — the observable_effect trigger must actually emit a SkillLibraryEvent row.
- **W-2 (no default-off terminal state)**: the library + S-tier autonomy barrier are ON by default. The ONLY env
  var is a kill-switch (`ELAI_SKILL_AUTONOMY_BARRIER_DISABLE`), default unset = enforced. The stub_ban forbids an
  `ELAI_SKILL_LIBRARY_ENABLE` default-off gate, and the W-2 acceptance verifies observable_effect runs with NO
  override. There is no flip-to-on step needed because there is no default-off staging.
- **W-3 (production entrypoint)**: reachable_from names FOUR real production entrypoints — the `skills-mgmt`
  CLI subcommand (args_skills.rs), the live orchestrator's TemplateCrystallizer (runtime_setup.rs:316 Arc),
  the task_runner autonomous-mode call site (task_runner.rs:6516), and the dashboard tauri command. None is
  test-only.
- **W-4 (frontend pair)**: frontend_pair names SkillLibraryPanel.svelte + the get_skill_library tauri command;
  the SkillLibraryEvent SSB signal is dashboard-surfaceable, so the pair is mandatory and cross_lang_diff must
  pass. Not an internal-only gate — humans SEE tiers and acquisitions, so the W-4 exception does not apply.
- **W-5 (live observable_effect)**: observable_effect.trigger is a runnable CLI command pair (`library promote` +
  `library tick`), expect_ssb is a live `skill_library_event` row, expect_artifact is the `library list` stdout
  regex + the panel render. It is NOT a unit test. The critic-gated ACQUISITION path additionally relies on the
  live crystallizer; that path is exercised by the C6b contract test as a deterministic fixture over the REAL
  `acquire_learned_skill` code (not a mock), and a separate gated checkbox covers a live crystallization run.
- **W-6 (dashboard checkpoint)**: the W-6 ACCEPTANCE item verifies a tier change + event render THROUGH the
  dashboard via dashboard-smoker.
- **W-7 (no hallucinated integration points)**: every consumes_symbol and every MODIFY target was confirmed in
  elai.map at authoring time — TemplateCrystallizer (compiled_pipeline.rs:292), check_crystallization (line 335),
  run_best_of_n (structural_retry.rs:277), MiniCriticVerdict (mini_critic.rs), should_auto_advance
  (autonomous_gate.rs:178), task_runner autonomous_mode (task_runner.rs:6516), memory TrustTier
  (memory_trust_contract.rs), reranker RetrievalResult (reranker.rs), skills-mgmt (args.rs:1875),
  TemplateCrystallizedPayload (ssb.rs:15291), action_type_lock (action_type_lock.rs:2167). None invented.
- **W-8 (produces/consumes symmetry)**: every produced symbol meant to be invoked has an in-plan caller —
  `permits_autonomous` is called by `skill_permits_autonomous` (Task 3); `acquire_learned_skill` is called by the
  extended `check_crystallization` (Task 6); `demote_if_stale` is called by the scheduler tick + `library tick`
  CLI (Task 9); `promote` by `library promote` CLI (Task 4); `get_skill_library` by SkillLibraryPanel (Task 10).
- **W-9 (SSB golden + SCHEMA bump)**: SkillLibraryEvent flows through the action_type golden file; SCHEMA_VERSION
  is named "baseline + 1"; the Pattern-2 SINGLE-VARIANT collapse (one variant + 3-arm SkillLibraryEventKind) is
  used per the E-43 precedent; cross_lang_diff + action_type_lock must be green.
- **W-10 (tiered-gate)**: F-6 has NO network/subprocess/RunPod axis on any GATE path — the trust ladder, the
  reused-verdict critic gate, and the staleness scalar are pure in-process computation (genuinely fail-closed).
  The ONLY Tier-2 piece is the OPTIONAL reranker recall A/B in C6b Task 7, which is MEASUREMENT-ONLY, never a
  gate, and is split out: it does not block the done-gate and does not advertise fail-closed. The reranker USE
  (retrieving a learned skill as a candidate) is Tier-1 in-process over the already-indexed store.

## Tests to add

- `crates/contracts/tests/skill_trust_type_barrier_contract.rs` (~110 LOC) — Task 5: S-tier wire tags distinct
  from T-tier; no From<SkillTrustTier> for TrustTier; permits_autonomous only S4; golden Kind round-trip.
- `crates/contracts/tests/skill_critic_gate_contract.rs` (~90 LOC) — Task 8: acquire only on reused Pass verdict;
  Block/absent verdict -> no acquisition (fail-closed, no triple-critic).
- `crates/contracts/tests/skill_staleness_demotion_contract.rs` (~90 LOC) — Task 11: mock-clock demotion of stale
  S3 -> S2 + DemotedStale emit; fresh untouched; S1 floor.
- Unit tests in skill_trust.rs (#[cfg(test)]) for promote ladder thresholds (S4 human-gated cap) and the named
  constants.

## Files touched

- CREATE: `crates/orchestrator/src/skill_trust.rs` (~260 LOC C6a + ~120 LOC C6c)
- CREATE: `crates/contracts/tests/skill_trust_type_barrier_contract.rs` (~110 LOC)
- CREATE: `crates/contracts/tests/skill_critic_gate_contract.rs` (~90 LOC)
- CREATE: `crates/contracts/tests/skill_staleness_demotion_contract.rs` (~90 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/SkillLibraryPanel.svelte` (~280 LOC)
- CREATE: `workflow/reasoning/F-6-decisions.md` (~120 LOC)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~90 LOC: SkillLibraryEvent variant + 3-arm Kind + payload + sql_str +
  parse arm + CHECK + golden round-trip)
- MODIFY: `crates/orchestrator/src/compiled_pipeline.rs:285-430` (~130 LOC: acquire_learned_skill + critic-gate
  in check_crystallization, reusing the BestOfN verdict)
- MODIFY: `crates/orchestrator/src/autonomous_gate.rs:160-220` (~70 LOC: skill_permits_autonomous + the barrier
  clause)
- MODIFY: `crates/orchestrator/src/task_runner.rs:6500-6530` (~30 LOC: call skill_permits_autonomous at the
  autonomous_mode decision site)
- MODIFY: `crates/orchestrator/src/lib.rs` (~3 LOC: `pub mod skill_trust;` + re-exports)
- MODIFY: `crates/orchestrator/src/runtime_setup.rs:300-330` (~20 LOC: thread the learned-skill store into the
  crystallizer Arc context)
- MODIFY: `crates/cli/src/args_skills.rs` (~120 LOC: library list/promote/show/tick verbs)
- MODIFY: `crates/cli/src/main.rs` (~40 LOC: dispatch the new verbs)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (~140 LOC: get_skill_library)
- MODIFY: `crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte` OR `workspace.svelte.ts` (~30 LOC: register
  SkillLibraryPanel / DrawerKey — confirm surface at authoring time)
- MODIFY: `crates/orchestrator/tests/action_type_lock.rs` golden (~2 LOC: skill_library_event row)
- MODIFY: `crates/knowledge/src/reranker.rs` callers in orchestrator (~80 LOC retrieval wiring; no reranker
  internal change — interface use only)

## Citations / Grounding

- workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md §4 C6 (mechanism + REVISED SCOPE decompose
  C6a/b/c), §1.5 (Voyager 3.3x non-transferable — UNVERIFIED-for-coding, explicitly NOT a target), §3 Pillar 3
  (SHARD T1-T4 ladder + Voyager critic-gate borrow), §5 TIER 3, §7 (the four C6 "Do NOT" clauses). GROUNDED
  (synthesis doc, adversarially verified run wf_b959963a-db6).
- SHARD `workflow/research/shard-prior-art/00-COMPARISON-shard-vs-priorart.md` Pillar 3 — T1-T4 trust ladder
  shipped in skills.py; Voyager critic-gated acquisition loop. GROUNDED (repo fetched 2026-06-01 per synthesis).
- Voyager (the acquisition loop pattern: curriculum -> generate -> execute -> critic gate -> retry<=4 -> commit
  with description-embedding retrieval). The "3.3x more items" number is UNVERIFIED for coding tasks (Minecraft
  tech-tree, frozen 2024-04 codebase) — F-6 makes NO reuse-rate claim (§1.5).
- ELAI ground truth (code-read, CONFIRMED in elai.map this session): TemplateCrystallizer compiled_pipeline.rs:292
  + check_crystallization:335; MiniCritic/MiniCriticVerdict mini_critic.rs + run_best_of_n structural_retry.rs:277;
  TreeAutoAdvancePolicy::should_auto_advance autonomous_gate.rs:178; memory TrustTier memory_trust_contract.rs;
  reranker RetrievalResult reranker.rs; skills-mgmt args.rs:1875; SkillSystemEvent/skill_telemetry.rs;
  TemplateCrystallizedPayload ssb.rs:15291; Pattern-2 16th application ssb.rs:1888; action_type_lock.rs:2167.
- HARD RULES (CLAUDE.md): blake3 for skill-content hash (never blake2/sha2); no `.unwrap()`/`.expect()` on
  production paths; Pattern-2 SINGLE-VARIANT SSB collapse; anti-slopsquatting (no new dep — reuse reranker +
  serde + blake3 + uuid); no new dep without user confirmation.

## Open questions / risks

- The mini_critic_verdict SSB row lookup in C6b (Task 6) must reliably match the supersteps that produced the
  crystallizing successes (by superstep/agent_id). RISK: if the match is loose, an unrelated Pass verdict could
  gate an acquisition. MITIGATION: match on (agent_id, superstep) tuple from the ExecutionOutcome bucket, and the
  C6b contract test asserts a Block verdict for the SAME superstep blocks acquisition. Confirm the
  ExecutionOutcome carries the superstep id at authoring time; if not, thread it through record_execution.
- DrawerKey vs MonitorWindow panel: F-6 names both options; the implementer confirms which surface the dashboard
  uses (CriticDecisionPanel lives in MonitorWindow, suggesting the panel route) and uses exactly one — do not ship
  both. W-4 cross_lang_diff catches a half-wired pair.
- F-5 coordination: F-6's staleness scalar lives on SkillTrustTier metadata; F-5's lives on memory_facts/DKG
  concept_stale. They share the scheduler TICK but NOT a column. Coordinate the decay.rs edit window only; no
  schema overlap. If F-5 ships first, F-6's demote_if_stale hooks the same tick additively.
- Voyager-reuse-rate measurable is DEFERRED (§1.5 + §4 C6): no ELAI baseline exists; do not invent one. The
  measurable lands in a follow-up after >=20 acquisitions + concept_class SSB tagging.
- SCHEMA bump: baseline + 1. If a concurrent F-N also bumps SCHEMA in the same window, reconcile to a single
  monotonic bump per the SCHEMA-LOCK discipline (one bump per shipped unit).
- The S4-human-gate flow needs a real human-grant signal (`--human-grant` CLI flag for now). A dashboard
  approve-button for S4 promotion is a natural follow-up but is NOT in F-6 scope (the CLI flag satisfies W-3/W-5).

## Next

F-5 (cross-subsystem staleness cascade) is the natural composition: F-5's StalenessQuarantineBroadcast and F-6's
SkillLibraryEvent{DemotedStale} both fire on the scheduler tick; a future session can unify them under one
quarantine-broadcast consumer so a quarantined memory fact ALSO demotes any learned skill that cites it. The
deferred skill-reuse-rate measurable (>=20 acquisitions, concept_class SSB tagging) is the F-6 follow-up that
turns the library into a measured number. C6b phase-2 (the reranker recall A/B, Tier-2 measurement) is the
separately-gated measurement that quantifies retrieval quality without ever becoming a gate.
