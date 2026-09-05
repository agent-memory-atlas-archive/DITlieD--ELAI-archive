# Planning Decomposition Extras (E-46)

- **ELAI ids:** E-46 / E-006/E006-E46-001..E46-032 / SUMMARY/E-46.F01..F16
- **Spec status (as designed):** DARK (`ELAI_PLANNING_DECOMPOSITION_ENABLE`); feature-map notes goal-drift **slice wired 06-07**; original plan PLANNED
- **Crate / module (spec):** `orchestrator/src/{dual_ledger,agentless_shortcut,goal_drift_canary}.rs` plus `planning_decomposition/{dual_ledger_ext,reflection_buffer,goal_drift_ext,decomposition_entropy,skill_prior}.rs`; extend `research_tree.rs`, `auditor_mode.rs`, `bubble_up.rs`, `task_runner.rs`
- **Purpose of this file:** enough architecture to rebuild Magentic-One dual-ledger, Reflexion buffer, AGENTLESS leaf shortcut, Phase-pair goal-drift canary, Shannon entropy-stop, LATS-style process-reward, REPLAN_PHASE / LEDGER_RESET. Not a status ledger.

## 1. What it is

**Planning decomposition extras** tune ELAI’s 4-level recursive tree (`Idea > Concept > Phase > Step > BabyStep`) with five 2024-2026 additions: a Magentic-One **dual ledger** at Concept and Phase; an **AGENTLESS** 3-phase leaf shortcut at Step; a Reflexion **episodic reflection buffer** per node; a **goal-drift** Jaccard-and-cosine canary across consecutive Phases; a Shannon-**entropy stop** on decomposition forks. It also adds bubble-up actions `REPLAN_PHASE` and `LEDGER_RESET`, and a scalar `process_reward_score` on B-41 `LevelAuditVerdict` for LATS-style prune (`E-plan/E-46_planning_decomposition.md:1-2`, `:8-10`).

## 2. Why it exists

May 2026 survey vs 15 systems: AGENTLESS (2407.01489) hit 32.00% SWE-bench Lite at $0.70/issue with **less** decomposition on narrow fixes; Magentic-One (2411.04468) dual-ledger is the canonical replan trigger ELAI lacks at Concept/Phase; Reflexion (2303.11366) keeps verbal reflections **separate** from audit logs — ELAI’s bubble-up only re-embeds failure into the next prompt; goal-drift (2505.02709) measures deviation from the instruction-specified goal; multi-agent failure taxonomy (2503.13657) lists “improper task decomposition — too granular, too broad, not serializable”; LATS (2310.04406) uses value-function-as-auditor at every fork (`E-plan/E-46_planning_decomposition.md:10`). If extras are missing, narrow fixes pay full BabyStep tax, ledgers do not exist, drift is invisible, and low-information forks proceed. If faked (prompt “don’t over-decompose”), there is no journaled stop.

## 3. How it works (runtime)

1. **Who calls it.** No new CLI flags. `elai_cc.toml` `[planning.*]` toggles. TreeNavigator / `task_runner.rs` hooks; `auditor_mode.rs::aggregate_verdict`; `scope_planner` consumes scores + skill priors (`:78-82`).
2. **Dual ledger (Concept + Phase).** Each `ConceptNode` / `PhaseNode` carries `Option<TaskLedger>` and `Option<ProgressLedger>` (additive). Writes: (a) initial decomposition — TaskLedger from spec-grounder; (b) every audit verdict — ProgressLedger `current_node` + blockers; (c) every bubble-up — `blockers.push(Blocker{reason, ssb_evidence})`. When `ProgressLedger.blockers.len() > LEDGER_RESET_THRESHOLD` (default **3**), emit `ledger_reset_triggered` and dispatch `LEDGER_RESET` — re-decompose the affected Concept/Phase **with the current TaskLedger as input** (`:37`).
3. **AGENTLESS leaf shortcut (Step).** If impact-mapper in the **same session** shows (a) blast radius `≤ 1` file, (b) `≤ 1` symbol, (c) acceptance criteria contain no `"design"|"architecture"|"refactor"` tokens, (d) no cross-language pair (`cross_lang_diff.py`), flag `agentless_shortcut_eligible`. Auditor variant `AGENTLESS_SHORTCUT_ELIGIBLE` (categorical, **not** pass/block). TreeNavigator **skips BabyStep** and runs localize → single-symbol repair → cargo test + contract validate. Validate **fail** → fall back to standard BabyStep with the failed-shortcut trace in the reflection buffer. Emit `agentless_shortcut_dispatched`. Stub-ban: no dispatch without same-session impact-mapper SSB (`:39`, `:90`). Risk-prose “assume radius=1 if mapper skipped” is **superseded** by the stub-ban (exhaustive CON-014).
4. **Reflexion buffer (every node).** `reflection_buffer: Vec<Reflection>` on Concept/Phase/Step/BabyStep. `Reflection { text, source_event_id: Uuid, created_at_unix: u64, trust_tier: T1 }`. **Separate from audit logs** (audit remains authoritative). Every cascade (not only final failure) appends a one-paragraph reflection. On retry, TreeNavigator prepends “Recent reflections on this node:”. Cap default **8** per node (oldest evicts). blake3 content-hash de-dup. Write failure **must** emit `reflection_buffer_write_failed` (no silent fallback). Success: `reflection_appended` (`:41`, `:87`).
5. **Goal-drift canary (consecutive Phases).** At every Phase audit: (a) Jaccard of Phase acceptance-criteria tokens vs parent Idea success-criteria (B-40 `FramedIdea`); (b) cosine of Phase acceptance embedding vs Idea embedding (cached at Framing; do not recompute). Tokenize: lowercase + Unicode word boundaries + 200-word static stoplist. If `(jaccard < 0.25 OR cosine < 0.55) AND prev_phase_also_below` → `GOAL_DRIFT` bubble-up with **both** metrics. Missing embedding: `goal_drift_canary_skipped_no_embedding` and continue (`:43`, `:211`). Coordinates with B-22b/C-10: when 4G.9 lands, StagnationDetector **consumes** these rows rather than recomputing (`:43`). C-10’s canary remains original-task vs subtask Jaccard; this organ is **Phase vs Idea**.
6. **Shannon entropy stop.** At each Concept/Phase decomposition fork (acceptance also says **every** fork — §16): `H = -sum(p_i * log2(p_i))` over the union of child acceptance-criteria tokens (`p_i` = frequency of token i across children); also `H_parent`. If `H_children / H_parent < ENTROPY_RATIO_THRESHOLD` (default **0.6**): emit `low_entropy_decomposition`; `strict_mode=true` → refuse, treat level as atomic; `strict_mode=false` (default) → allow + WARN on per-level auditor. Companion: hard **depth-3** ceiling under any BabyStep, extended to all levels behind `[planning.depth_ceiling]` (`:45`). Tiny parent (`≤2` tokens) unstable; proposed no-op below **20** tokens is **not** in the config schema (AMB-017).
7. **Process-reward scalar (LATS).** `LevelAuditVerdict.process_reward_score: f32 ∈ [0,1]`. Deterministic: `score = (1.0 - findings.len() as f32 / max_expected_findings) * verdict_modifier` with `max_expected_findings = 5` and modifiers `{ PASS: 1.0, WARN: 0.6, BLOCK: 0.1, AGENTLESS_SHORTCUT_ELIGIBLE: 0.85 }`. Additive; categorical consumers ignore it. Scope-planner-elai prunes forks with parent `score < 0.4` before TreeNavigator commits. Emit `process_reward_score_emitted` (`:47`). Default: **no** LLM value-function (`:207-209`).
8. **Bubble-up actions.** `REPLAN_PHASE { failed_plan, error_transcript, original_goal }` fires when the **same BabyStep fails twice** OR goal-drift canary fires **twice consecutively**; handler dispatches phase-decomposer with those three inputs; emit `bubble_up_replan_phase`. `LEDGER_RESET { concept_id_or_phase_id }` fires independently when blockers `> threshold`; nonempty payload required; emit `bubble_up_ledger_reset` (`:49`). Objective text “6th bubble-up action” vs two new variants: **sixth and seventh** (CON-012). Empty payload banned (`:88`).
9. **Skill priors (Voyager-lite).** Successful BabyStep → `SkillPrior { goal_embedding, accepted_diff_summary, ssb_evidence_ids }` in ResearchCache. Future Phase decomposition: top-K=**3** by cosine; `K=0` disables. Emit `skill_prior_retrieved`. Cold cache empty for ~first 20 BabySteps — document, not a bug (`:51`, `:215`).
10. **Dashboard.** `DualLedgerPanel`, `GoalDriftCanaryGauge` (last 10 Phase audits), `DecompositionEntropyHistogram`; Tauri `get_dual_ledger`, `get_goal_drift_history`, `get_entropy_history`; mount in `ReasoningDrawer` (`:53`).

The model may fill ledger **guesses** and reflection **text**. It may **not** skip entropy math, author a shortcut without impact evidence, or emit drift with only one metric.

## 4. Types, enums, objects

Copy from `E-plan/E-46_planning_decomposition.md:37-51`, `:59-70`. Do not invent fields.

```
TaskLedger { facts: Vec<LedgerFact>, guesses: Vec<LedgerGuess>, plan: Vec<PlanStep> }
ProgressLedger { current_node: NodeId, assignment: AgentRole, blockers: Vec<Blocker> }
  Blocker { reason, ssb_evidence }

AgentlessShortcutDispatcher::run(step: &StepNode) -> ShortcutResult
LevelAuditVerdictVariant::AGENTLESS_SHORTCUT_ELIGIBLE

Reflection { text, source_event_id, created_at_unix, trust_tier: T1 }
  cap default 8 / node

GoalDriftCanary::evaluate(phase, idea) -> GoalDriftReport
  jaccard < 0.25 OR cosine < 0.55, two consecutive Phases

fn compute_entropy_ratio(parent, children) -> f32
  H = -Σ p_i log2(p_i); threshold 0.6

LevelAuditVerdict.process_reward_score: f32   // [0,1]
  (1 - findings/5) * modifier

BubbleUpAction::REPLAN_PHASE { failed_plan, error_transcript, original_goal }
BubbleUpAction::LEDGER_RESET { concept_id_or_phase_id }

SkillPrior { goal_embedding, accepted_diff_summary, ssb_evidence_ids }
```

`LedgerFact` / `LedgerGuess` / `PlanStep` / `NodeId` / `Blocker` **field interiors** beyond the shapes above: **UNCERTAIN** (§16).

## 5. Control flow / state machine

**Owner:** TreeNavigator + bubble-up cascade (B-7 5-action cascade **widens**, no schema removal) (`:14`).

```
decompose Concept/Phase
  write TaskLedger
  compute entropy ratio -> maybe low_entropy_decomposition (WARN or atomic)
Step eligible? -> AGENTLESS 3-phase else BabySteps
per audit: update ProgressLedger; process_reward_score; Phase -> goal-drift pair
blockers > 3 -> LEDGER_RESET
same BabyStep fail ×2 OR drift ×2 consecutive -> REPLAN_PHASE
every cascade -> reflection_buffer append (T1, blake3 de-dup)
retry -> prepend reflections
score < 0.4 on parent -> prune fork
```

**Independent triggers:** REPLAN_PHASE and LEDGER_RESET do **not** cascade into each other (exhaustive DEL-034). Summary “cascade first to REPLAN_PHASE, onward to LEDGER_RESET” is **wrong**.

## 6. Grounding, receipts, gates

**Receipts:** SSB evidence ids on blockers and skill priors; impact-mapper row **required** for AGENTLESS; both Jaccard **and** cosine on every drift emit (`:89`); blake3 on reflection text; Idea embedding cached at Framing.

**Never allowed:** fake entropy (`// TODO: compute entropy` banned) (`:84`); drift emit with one metric; shortcut without impact-mapper; silent reflection write failure; LLM process-reward as default (`:207-209`); merging this canary with J-85 overthinking **DRIFT** (different organ) or with C-10 baby-step Jaccard without keeping both.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| LEDGER_RESET_THRESHOLD | 3 (strict `>`) | Concept/Phase |
| AGENTLESS radius | ≤1 file and ≤1 symbol | Step |
| reflection cap | 8 / node (default) | per level |
| drift Jaccard / cosine | 0.25 / 0.55 | two consecutive Phases |
| ENTROPY_RATIO_THRESHOLD | 0.6 | fork |
| entropy strict_mode | false (WARN) | config |
| depth ceiling | 3, all levels, reversible config | tree |
| max_expected_findings | 5 | score |
| prune | parent score `< 0.4` | fork |
| skill-prior K | 3; `0` disables | Phase decomp |
| gauge window | last 10 Phase audits | UI |

Proposed public-trait refuse when SCIP_REF `>5`: risk mitigation, **not** acceptance (`:213`).

## 8. Roles and neighbors

**Consumes:** B-7 `ResearchTree` / bubble-up; B-40 `FramedIdea` embedding; B-41 `LevelAuditVerdict`; `ProjectProfile` impact radius; blake3 (`:71-77`). **Produces:** types in §4. **Frontend:** three new Svelte panels (`:92`).

**C-10 / B-22b:** consume `goal_drift_canary_below_threshold` when 4G.9 lands (`:43`). **E-19 / E-49:** consume `process_reward_score` (`:217-218`). **B-23:** consumes `bubble_up_replan_phase` / `ledger_reset` evidence shapes (`:18`). **C-48:** T1 on reflection rows (`:19`).

**Must not fork:** a second tree; replacing B-7’s five actions; LLM value-function on by default.

## 9. Config / flags / defaults

Feature-map DARK `ELAI_PLANNING_DECOMPOSITION_ENABLE` (`ELAI-FEATURE-MAP.md:619-621`). TOML blocks (`E-plan/E-46_planning_decomposition.md:189`):

- `[planning.ledger]` — reset threshold
- `[planning.goal_drift]` — 0.25 / 0.55
- `[planning.entropy]` — 0.6, `strict_mode`
- `[planning.depth_ceiling]`
- `[planning.skill_priors]` — K

`[planning.reflection_buffer]` is **proposed in risks**, omitted from the five listed blocks (AMB-015). `[planning.value_function_llm]` proposed default off (`:207-209`).

Decision doc: `workflow/reasoning/E-46-decisions.md` (`:191`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `dual_ledger.rs` + `planning_decomposition/dual_ledger_ext.rs` | `E-plan/E-46_planning_decomposition.md:176` |
| `agentless_shortcut.rs` (flat) | `:177` |
| `planning_decomposition/reflection_buffer.rs` | `:178` |
| `goal_drift_canary.rs` + `goal_drift_ext.rs` | `:179` |
| `planning_decomposition/decomposition_entropy.rs` | `:180` |
| `planning_decomposition/skill_prior.rs` | `:181` |
| three Svelte panels | `:182-184` |
| MODIFY `research_tree.rs`, `auditor_mode.rs`, `bubble_up.rs`, `task_runner.rs` | `:186-189` |
| Tauri three commands | `:190` |
| tests `:161-172` | |

Summary path `crates/orchestrator/src/decomposition.rs` is **not** the original file list (E-SUMMARY_INDEX.md:395 vs original Files touched). Rebuild the original split layout.

## 11. Journal / SSB events

Original names (not the summary’s collapsed labels):

| Event | When |
|---|---|
| `ledger_reset_triggered` | blockers `> 3` |
| `bubble_up_ledger_reset` | LEDGER_RESET handler |
| `agentless_shortcut_dispatched` | shortcut run |
| `reflection_appended` / `reflection_buffer_write_failed` | buffer I/O |
| `goal_drift_canary_below_threshold` | first below-threshold Phase |
| `GOAL_DRIFT` / `bubble_up_goal_drift` | two consecutive; **vocabulary UNCERTAIN** (AMB-016) |
| `low_entropy_decomposition` | ratio `< 0.6` |
| `process_reward_score_emitted` | every verdict |
| `bubble_up_replan_phase` | REPLAN_PHASE |
| `skill_prior_retrieved` | K>0 hit |
| `goal_drift_canary_skipped_no_embedding` | missing Idea embedding |

Summary `goal_drift_canary_fired` / `replan_phase` / `ledger_reset` are **not** the original symbols (E-SUMMARY_INDEX.md:395; exhaustive DEL-034). Rebuild originals.

## 12. Tests and observables

Unit/contract (`:161-172`): ledger threshold → LEDGER_RESET; single-file shortcut dispatch; validate-fail fallback + reflection trace; cap evicts oldest; two consecutive Phases fire canary with **both** metrics; entropy ratio `<0.6` → row + WARN; 50 synthetic scores in `[0,1]`; REPLAN_PHASE / LEDGER_RESET / AGENTLESS contracts.

**Observable effect** (`:93-96`): synthetic 4-Phase plan, Phases 3+4 Jaccard `<0.20` vs Idea, entropy ratio 0.4 → `goal_drift_canary_below_threshold` at Phase 3, `bubble_up_goal_drift` at Phase 4, `low_entropy_decomposition` at seeded fork; REPLAN_PHASE handler; Phase-4 buffer ≥1 entry whose `source_event_id` matches the canary. Summary “clean trajectory completes” is **not** a named original test (DEL-035).

Deterministic rg list `:100-144` (ledgers, shortcut, buffers, both drift metrics, real Shannon, score field, both actions, blake3-only, eight event labels).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Dual `TaskLedger` + `ProgressLedger` on Concept and Phase; three write points; `LEDGER_RESET` when blockers `> 3`.
- AGENTLESS 4-predicate shortcut → localize/repair/validate; fail → BabyStep + reflection; no mapper → no shortcut.
- Reflexion buffer per node, T1, cap 8, prepend on retry, blake3 de-dup, fail-loud write.
- Goal-drift **both** Jaccard and cosine; two consecutive Phases; thresholds 0.25 / 0.55.
- Real Shannon `H=-Σ p log2 p`; ratio 0.6; default WARN; optional strict atomic.
- `process_reward_score` formula + modifiers; prune `< 0.4`.
- `REPLAN_PHASE` and `LEDGER_RESET` **independent** payloaded actions.
- Skill-prior K=3 / K=0.
- B-7 five-action cascade does not lose variants.

A “don’t over-decompose” prompt without entropy math, or a single Jaccard canary copied from C-10, is a cousin.

## 14. Non-goals / banned designs

- Fake entropy / `unimplemented!` / prod unwrap (`:84-86`).
- Silent reflection write failure (`:87`).
- Empty REPLAN_PHASE / LEDGER_RESET payload (`:88`).
- Drift emit without both metrics (`:89`).
- AGENTLESS without same-session impact-mapper (`:90`).
- Default-on LLM value-function (`:207-209`).
- Authoring J-85; replacing C-10’s baby-step Jaccard.
- Removing B-7’s existing five bubble-up actions (`:159`).
- blake2/sha2 on reflection (`:91`).

## 15. Source citations

Primary: `docs/thefinalglance/E-plan/E-46_planning_decomposition.md` (full file). Index: `E-plan/E-SUMMARY_INDEX.md:390-396` (summary symbols **wrong** — use original). Feature map: `ELAI-FEATURE-MAP.md:619-621`. Exhaustive: `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md:15950-16013`. B-22b neighbor: `B-plan/B-PLAN-SUMMARY.md:424-431`. C-10 neighbor: `C-plan/C-10_anti_spiral_primitives.md`. Papers: 2407.01489, 2411.04468, 2303.11366, 2505.02709, 2503.13657, 2310.04406, 2305.16291 (`E-plan/E-46_planning_decomposition.md:193-205`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Interiors of `LedgerFact` / `LedgerGuess` / `PlanStep` / `Blocker` (`:37` names vec types only).
- Event vocabulary `goal_drift_canary_below_threshold` vs `GOAL_DRIFT` vs `bubble_up_goal_drift` (AMB-016).
- Entropy at Concept/Phase only vs “every decomposition fork” including Step (CON-013).
- 20-token parent minimum not in schema (AMB-017).
- `[planning.reflection_buffer]` vs five listed blocks (AMB-015).
- Sixth vs seventh bubble-up (CON-012) — rebuild **two** new variants.
- Thresholds 0.25/0.55 are guesses; 10-Phase calibration probe optional (`:210`).
- Whether `ELAI_PLANNING_DECOMPOSITION_ENABLE` gates the 06-07 wired goal-drift slice or only the rest (`ELAI-FEATURE-MAP.md:619-621`).
