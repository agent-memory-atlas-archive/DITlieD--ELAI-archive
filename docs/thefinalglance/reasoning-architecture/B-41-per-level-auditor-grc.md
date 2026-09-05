# Per-Level Auditor GRC Modes (B-41)

- **ELAI ids:** B-41 / B41-001..B41-032
- **Spec status (as designed):** PARTIAL — types/events specified; TreeNavigator 5 insertion points historically deferred (`ELAI-FEATURE-MAP.md:766-770`, `B-PLAN-SUMMARY.md:884-915`)
- **Crate / module (spec):** `crates/orchestrator/src/auditor_mode.rs`; parametric `AgentRole::Auditor(AuditorMode)`; `reasoning/src/config.rs` six GRC arms; `.claude/agents/auditor.md` MODE DISPATCH; `pipeline_tree_navigator.rs` insertion points; `AuditTimelineWindow.svelte`
- **Purpose of this file:** enough architecture to rebuild the 6-mode per-tier auditor with **tiered GRC budgets**, shared 7-domain rule cluster, one-respawn BLOCK policy, and per-level auto-advance checks. Not a status ledger.

## 1. What it is

**Per-level auditor modes** replace the single end-of-implementation auditor with six modes that audit each `DecompositionTree` tier **immediately after that tier is produced**: `Concepts`, `Phases`, `Steps`, `BabySteps`, `BabyStepDetail`, `Plan` (`B-41_per_level_auditor_modes.md:1-10`, `ELAI-SYSTEMS-OVERVIEW.md:195-197`). Every mode loads the **same** rule cluster in fixed order but **scopes findings to its tier**. BLOCK at tier N respawns the level-N decomposer **ONCE** (no re-audit of the same output), then cascades to N+1 / parent (`:7-8`, `:73-80`). `Visual` is reserved under `cfg(feature="visual_audit")` (`:98-100`). GRC budgets: Concepts **6/1000**, Phases **5/800**, Steps **4/700**, BabySteps **4/700**, BabyStepDetail **3/600**, Plan **10/1200** (`:115-121`, B41-010).

## 2. Why it exists

The stock auditor (FSM AUDITING, sub-states READ_CONTEXT / RUN_TESTS / REVIEW_CODE / WRITE_FINDINGS) runs **once** at end-of-implementation and cannot tell a bad Concept from a misplaced semicolon (`B-41_per_level_auditor_modes.md:42-49`). Phase 4E `BubbleUpHandler` retries on **test** failure (BabyStep=3, Step=2, Phase=1, Concept=1, Escalate), not on auditor verdict, and no auditor runs at intermediate tiers (`:51-54`, `:24-27`). If this organ is missing, 50 wasted baby steps follow a rotten Concept. If it is faked (one global auditor with a “mode=” prompt and Plan’s 10/1200 for every tier), cheap line-level checks starve and Concept checks under-budget. If BLOCK retries the **same** auditor on the **same** output, compute is wasted (`:75-78`).

## 3. How it works (runtime)

1. **Who.** `TreeNavigator` on tier completion — **five** insertion points (`:123-135`, `:204-206`, B41-011). BabySteps audit sits between navigator cycle **step 6 acceptance** and **step 7 integration** (`:87`, B41-011). `mode=Plan` remains the existing end-of-task cycle (`:70-71`).
2. **Dispatch.** `AuditorMode` argument on `.claude/agents/auditor.md` **MODE DISPATCH** section **before** the FSM SUB-STATE block (`:101-110`). Parametric `AgentRole::Auditor(AuditorMode)` — not six role variants (`:112-114`, `:337-342`). Missing mode must **not** default to Plan (`:209`).
3. **Rule cluster (deterministic, every mode, fixed order)** (`:102-106`, `:211`, B41-002):
   ```
   anti-stale → coding-standards → design-hard → app-store →
   source-tiers → slopsquatting → risk-levels
   ```
   Missing domain **fails the test**, does not silent-skip (`:283-284`).
4. **Sub-states.** Modes 1–4 (Concepts…BabySteps) skip `RUN_TESTS` (no executable code yet). Reduced flow: `READ_CONTEXT → REVIEW → WRITE_FINDINGS` (`:109-110`, B41-009). BabyStepDetail / Plan may use tests as appropriate to their tier.
5. **GRC install.** `agent_grc_config_for_role` six arms with budgets in §7 (`:114-121`). Model generates findings; engine still enforces 2A/2C `think` (Plan mode retains anti-rubber-stamp FINDING — neighbor 2C).
6. **BLOCK policy (deterministic, distinct from BubbleUp)** (`:73-80`, `:123-135`, B41-012, B41-014):
   - BLOCK → inject findings as `rescope_from_audit` hint → respawn **level-N decomposer exactly once** → `respawned_once = true`.
   - Do **not** re-run the same auditor on the same decomposer output (objective/context).
   - If the **next** tier’s audit also BLOCKs, parent tier is the real fault → `cascade_to_parent` + `BubbleUpHandler` (Concepts second-fail → Idea / B-40 FRAMING) (`:128-130`).
   - Re-audit **after respawn** is described in the observable sequence (second `per_level_audit_started` on Concepts) (`:221-227`). Spec contradicts itself on “no re-audit” vs second audit of **new** output (B41-013) — see §16. Rebuild: **one respawn per node**; audit the **new** decomposer output once; never a third.
7. **`LevelAuditVerdict`** persisted as SSB `per_level_audit_verdict` (`:137-149`).
8. **AutoAdvanceEvaluator.** Keep B-22’s 6 checks; **split** `zero_blocks_remaining` into `concept_zero_blocks | phase_zero_blocks | step_zero_blocks | babystep_zero_blocks | plan_zero_blocks`. Each queries verdicts by mode and requires `cascade_to_parent == false` (`:150-154`, B41-016). Ambiguity ALLOW does not satisfy these (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B-001/X-006).
9. **Concepts ground truth.** Consume B-40 `FramedIdea`. Un-framed task → **WARN** + coarse-rule fallback (`:286-287`).
10. **Artifact.** `workflow/sessions/{sid}/audits/{mode}/{node_id}.md` — full findings + respawn payload (`:180-183`). In-memory-only verdicts forbidden (`:212`).
11. **Commit / fail-closed.** ALLOW advances the navigator; BLOCK without remaining respawn cascades; missing rule domain fails closed; mode always explicit.

## 4. Types, enums, objects

Copy from `B-41_per_level_auditor_modes.md:98-148`, `:115-121`.

```
enum AuditorMode {
  Concepts,
  Phases,
  Steps,
  BabySteps,
  BabyStepDetail,
  Plan,
  // Visual reserved: cfg(feature="visual_audit")
}

pub struct LevelAuditVerdict {
    pub mode: AuditorMode,
    pub tier_node_id: NodeId,     // DecompositionTree
    pub findings: Vec<Finding>,
    pub verdict: Verdict,         // ALLOW | BLOCK
    pub respawned_once: bool,     // true after the one allowed respawn path
    pub cascade_to_parent: bool,  // respawned_once && still BLOCK
}

dispatch_level_audit(...)
respawn_with_rescope_hint(...)

AgentRole::Auditor(AuditorMode)

# GRC arms (calls/chars)
Concepts        6 / 1000     // semantic, expensive
Phases          5 / 800
Steps           4 / 700
BabySteps       4 / 700
BabyStepDetail  3 / 600      // line-level, fast
Plan           10 / 1200     // existing end-of-task

# BabyStepSpec 8 blocks (mode=BabySteps completeness)
GROUNDING | PRE_GATE | TASK | TDD | DETERMINISTIC_CHECK |
ACCEPTANCE | INTEGRATION | EXIT_CRITERIA
```

**Per-mode checks** (`:56-71`, B41-003..B41-008):

| Mode | When | Looks for | Ignores (other tiers) |
|---|---|---|---|
| Concepts | after ~5 Designer concepts | market fit, IP, scope, contradictions vs `FramedIdea` | line-level code |
| Phases | after each Concept→Phases | ordering, contract sufficiency, dependency cycles | baby-step completeness |
| Steps | after each Phase→Steps | granularity, test coverage of step boundaries | store-guideline prose |
| BabySteps | after each Step→BabySteps | 8-block completeness | implementation unwraps |
| BabyStepDetail | after batch **≤4** parallel authored baby steps | lazy-code, unwrap, hardcoded-secret, API-version-stale | concept market fit |
| Plan | end-of-task | integration issues no per-tier audit sees | (retained global) |

BabyStepDetail batches **because** children share the Step `pre_gate` and findings correlate (`:346-348`).

**Rule seed domains** (SHADOW, Phase 3F lifecycle) (`:156-171`):

| domain | ~count | examples |
|---|---|---|
| `design_hard` | ~25 | contrast, spacing, WCAG AA palette, density, font-stack |
| `app_store` | ~15 | private API ban, ATS, deprecated permissions, ratings |
| `source_tiers` | ~10 | financial=T1, design=T2–T3 OK, marketing=T2+ |
| `slopsquatting` | ~30 patterns | A-6 `deprecated_patterns` format; hallucinated packages |
| `risk_levels` | ~12 | concept pivot=HIGH, babystep refactor=LOW |

## 5. Control flow / state machine

**Owner:** `TreeNavigator` + `dispatch_level_audit`. LLM does not pick the mode.

```
tier N produced
  → per_level_audit_started(mode)
  → Auditor(mode) GRC think under tier budget
  → LevelAuditVerdict
       ALLOW → next tier / next node
       BLOCK && !respawned_once → rescope_from_audit → respawn decomposer once
       BLOCK && respawned_once → cascade_to_parent → BubbleUpHandler
```

**Insertion map** (`:123-135`):

- After Concepts: `Auditor(Concepts)`; second Concepts fail at Phases → cascade to Idea/FRAMING.
- Same pattern Phases / Steps / BabySteps → corresponding decomposer.
- After BabyStep batch (≤4 parallel, existing BubbleUp policy): `Auditor(BabyStepDetail)`; BLOCK respawns babystep-author once.

**Forced:** `respawned_once` cap 1 per node (`:210`). Rule order not configurable (`:352-354` — entropy quarantine: same seed + input = identical findings).

Auditor agent sub-state machine is **not** a second GRC; GRC `think` still owns Observe/Verify/Converge inside REVIEW/WRITE_FINDINGS.

## 6. Grounding, receipts, gates

Findings must be tier-scoped; a Concepts auditor citing an unwrap is out of scope (deferred to BabyStepDetail). Concepts without `FramedIdea` WARNs — does not invent market evidence (`:286-287`). Cosmos briefings (B-42) are fuel, not required for the organ to exist; tolerate missing fresh Cosmos (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B-001/X-008).

**Never allowed as a confidence signal:** Plan-mode 10/1200 used as a quality score; re-running the same auditor to “get a better vibe”; defaulting unknown tier to Plan; in-memory verdict without SSB; skipping rule domains.

Per-level auto-advance checks are **receipts** over `per_level_audit_verdict` rows, not LLM “all clear”.

## 7. Budgets, limits, ceilings

| Mode | GRC calls/chars | Notes |
|---|---|---|
| Concepts | **6/1000** | semantic |
| Phases | **5/800** | |
| Steps | **4/700** | |
| BabySteps | **4/700** | |
| BabyStepDetail | **3/600** | line-level |
| Plan | **10/1200** | existing auditor |

| Knob | Value |
|---|---|
| BLOCK respawns | **1** per node |
| BabyStepDetail batch | ≤ **4** parallel |
| BubbleUp retries (neighbor, **not** this policy) | BabyStep=3, Step=2, Phase=1, Concept=1, Escalate (`phase-4E.txt` 4E.10 as cited `:24-27`) |
| AutoAdvance new checks | **5** (plus original 6) |
| tests | ≥ 30 |
| ablation | ≥ 40% drop in BabyStep-tier BLOCKs on 30-task fixture |
| SCHEMA | designed +1 (28→29 or 29→30); shipped 45 — §16 |

Plan 10/1200 matches 2C auditor (`02C-per-agent-grc-config.md` §7). Do not use 2C auditor 10/1200 for Concepts.

## 8. Roles and neighbors

**Consumes:** `DecompositionTree`, `TreeNavigator`, `BubbleUpHandler`, `RuleEngine`, B-22 AutoAdvanceEvaluator, B-40 `FramedIdea` (`:196-202`).

**Produces:** `AuditorMode`, `LevelAuditVerdict`, `dispatch_level_audit`, `respawn_with_rescope_hint`, 6 GRC arms, 5 rule seeds, 3 SSB types, AuditTimelineWindow, Monitor per-mode `audit X/Y` badges (`:185-194`, `:173-178`).

**Must not fork:** six separate `AgentRole` variants (`:337-342`); using BubbleUp’s 3-retry as auditor retry; a second rule-load order.

**B-39:** text-only substitute turns never hit this auditor (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B-001/X-010).

## 9. Config / flags / defaults

- `cfg(feature="visual_audit")` for Visual only (`:100`). Visual ≠ B-43 one-line install (identifier collision, B41-031).
- Rule cluster order **not** user-configurable (`:352-354`).
- Seeds SHADOW until 3F promotion (`:157-158`).
- `get_per_level_verdicts(task_id)` live, no mock (`:178`, `:288-290`).
- Decision doc `workflow/reasoning/B-41-decisions.md` (`:337-356`).

## 10. Implementation map

| Symbol | Spec |
|---|---|
| `orchestrator/src/auditor_mode.rs` (~600 LOC) | `:313` |
| `.claude/agents/auditor.md` MODE DISPATCH + ~80 LOC/mode prompts | `:101-108`, `:327` |
| `agent_pool.rs` `Auditor(AuditorMode)` | `:328` |
| `reasoning/src/config.rs` 6 arms | `:329` |
| `pipeline_tree_navigator.rs` 5 points | `:330` |
| `task_runner.rs` 5 evidence checks + respawn | `:331` |
| `ssb.rs` +3 types | `:332` |
| `AuditTimelineWindow.svelte` | `:320` |
| `rules/seed/{design_hard,app_store,source_tiers,slopsquatting,risk_levels}.toml` | `:321-326` |

Feature map later notes C-12 navigator wiring 5 emit sites in `task_runner` (`ELAI-FEATURE-MAP.md:768-770`) vs B-0 “navigator-spawn deferred” (`B-PLAN-SUMMARY.md:914-915`) — architecture still requires the five **semantic** insertion points.

## 11. Journal / SSB events

| action_type | role |
|---|---|
| `per_level_audit_started` | mode (+ respawned_once in observable replay `:226`) |
| `per_level_audit_verdict` | full `LevelAuditVerdict` |
| `per_level_audit_cascade` | parent cascade |

Canonical observable (`:217-231`): bad concept “use blockchain for plant ID” → Concepts BLOCK with C-FEASIBILITY or C-MARKET → Designer `rescope_from_audit` → started(respawned_once=true) → ALLOW → Phases. Artifact `audits/Concepts/concept-2.md`.

## 12. Tests and observables

Deterministic (`:233-272`): SCHEMA bump grep; 3 action strings; AuditorMode 6 active + Visual cfg (7 with cfg); `## MODE DISPATCH`; 5 seed TOMLs; `respawned_once`; 5 evaluator names; ≥30 tests; AuditTimelineWindow; cargo + svelte-check.

Suites (`:296-310`): `auditor_mode_dispatch.rs` (6 modes through navigator); `per_level_audit_contract.rs` (one respawn, cascade, SSB); `rule_cluster_load_order.rs` (anti-stale first, missing domain throws); `autonomous_gate_with_per_level_contract.rs`; `audit_artifact_persistence.md` shape.

Acceptance also: existing 14-test `autonomous_gate_contract.rs` still passes; B-40 WARN fallback; no mock UI; 40% ablation (`:274-294`).

## 13. Rebuild contract

Another harness has **this** organ iff:

- Six modes + cfg Visual; parametric `Auditor(mode)`; no default-to-Plan.
- GRC budgets **6/1000, 5/800, 4/700, 4/700, 3/600, 10/1200** exactly.
- Shared 7-domain cluster in that order; missing domain fails.
- Five navigator insertion points including BabySteps between acceptance and integration; Plan retained at end.
- BLOCK → one `rescope_from_audit` respawn per node → cascade if still BLOCK; not BubbleUp 3/2/1/1.
- `LevelAuditVerdict` + three SSB types + disk markdown.
- Five per-level zero-BLOCK checks **added** to B-22’s six.
- Concepts consumes `FramedIdea` or WARNs.
- BabyStepDetail batches ≤4.

A single end auditor with a mode string in the prompt and one 10/1200 budget is a cousin.

## 14. Non-goals / banned designs

- Replacing BubbleUp test-retry policy with auditor retries (`:73-80`).
- Re-auditing the **same** decomposer output (`:75-78`).
- Configurable rule order (`:352-354`).
- Implementing Visual here (B-43-as-visual-audit; depends on B-24 VLM + B-42; identifier collides with one-line-install B-43) (`:355-356`, B41-031).
- Mock verdicts (`:288-290`).
- Skipping RUN_TESTS skip on modes 1–4 (would invent tests for concepts).

## 15. Source citations

Primary: `docs/thefinalglance/B-plan/B-41_per_level_auditor_modes.md` (full file). Index: `B-PLAN-SUMMARY.md:884-918`, `:993`. Feature map: `ELAI-FEATURE-MAP.md:766-770`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:195-197`. Exhaustive: B41-001..B41-032. Neighbors: B-22, B-40, phase-4D 4D.7–4D.10, phase-4E 4E.10, phase-4G 4G.4, `02C-per-agent-grc-config.md`. Paper: B-41 cites arXiv:2406.01297; B-0 substitutes arXiv:2404.03602 (B41-030).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- **Re-audit after respawn** (B41-013): “no re-audit” vs observable second Concepts audit of **new** output vs `respawned_once` “after second audit”. Rebuild one respawn + audit the new tree node once.
- SCHEMA 28→29 / 29→30 vs shipped 45 (`B41-029`).
- Navigator-spawn deferred vs C-12 five emit sites (`B-PLAN-SUMMARY.md:914-915` vs `ELAI-FEATURE-MAP.md:768-770`). Organ **requires** the five insertion semantics.
- `Finding` / `Verdict` / `NodeId` field-level types (named, not defined in B-41).
- Per-mode GRC `required_steps` / `allowed_methods` / FINDING for non-Plan modes (only budgets specified).
- Kamoi citation 2406.01297 vs 2404.03602 (B41-030) — do not pick; both recorded.
- Visual vs B-43 one-line-install name collision (B41-031).
- Whether `plan_zero_blocks` is the old global check renamed or an additional Plan-mode row.
- Exact MODE DISPATCH prompt text (~80 LOC/mode) — specified as existing in auditor.md, not reproduced here.
