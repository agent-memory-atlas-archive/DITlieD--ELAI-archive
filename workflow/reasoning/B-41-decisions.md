B-41 Per-Level Auditor Modes — Decisions
========================================

Session: s-10380
FSM: EXECUTING
Status: Slice 1 (foundational types) SHIPPED 2026-05-16; Slices 2-3 deferred (see CONTINUATION at end).

DECISION-AGENTROLE: 5 unit variants vs parametric Auditor(AuditorMode)
---------------------------------------------------------------------
Chose 5 unit variants (AuditorConcepts / AuditorPhases / AuditorSteps /
AuditorBabySteps / AuditorBabyStepDetail) over `Auditor(AuditorMode)`.
Rationale:
- Parametric form requires AuditorMode to be Copy (achievable: 6 unit
  variants), but more importantly widens every existing pattern arm from
  `AgentRole::Auditor` to `AgentRole::Auditor(_)` across 22 referencing
  files and the contract test suite. Unit variants keep existing arms
  untouched; only exhaustive `match role { ... }` arms need the 5 new lines.
- `#[derive(Copy)]` on AgentRole stays trivially valid.
- Plan-mode keeps using the pre-existing `AgentRole::Auditor` — zero
  back-compat risk for callers that spawn the Plan auditor.
- Mapping from AuditorMode -> AgentRole lives in
  `auditor_mode::AuditorMode::map_to_agent_role` (canonical site).

DECISION-BLOCK-POLICY: one respawn vs BubbleUp 3/2/1/1
------------------------------------------------------
B-41 audit respawn policy is 1/1/1/1/1 (one respawn per tier per node), NOT
the 3/2/1/1 BubbleUp retry thresholds from phase-4E.txt §4E.10. Re-running
the same auditor on the same decomposer output yields the same verdict by
construction — repeated audit cycles burn budget for no information gain.
Second BLOCK at any tier cascades up via the existing
`emit_integration_bubble_up` path (sibling category
`AUDITOR_BUBBLE_UP_CATEGORY = "auditor_bubble_up"`).
Monotonic guard: `auditor_mode::mark_respawned` returns
`AuditorModeError::DoubleRespawn` if `respawned_once` is already true.

DECISION-BABYSTEP-DETAIL-BATCH: audit batch, not each
-----------------------------------------------------
Mode BabyStepDetail audits the BabyStep BATCH (≤4 parallel per existing
BubbleUp policy) as a single pass, not each BabyStep individually. Per-
babystep line audit would multiply auditor budget by batch size with
minimal precision gain (line-level findings cluster by file, not by
BabyStep). The verdict is keyed on the parent BabyStep tier id of the
batch — TierNodeId::BabyStep(BabyStepId).

DECISION-PLAN-MODE: keep existing single auditor
-------------------------------------------------
Mode Plan is the pre-B-41 single-cycle integration auditor. Retained for
two reasons:
1. Back-compat: existing callers that spawn `AgentRole::Auditor` see no
   behaviour change.
2. Per-tier audits catch tier-local issues; integration-level issues
   (cross-tier contract drift, end-to-end test failures) still need a
   single pass over the full plan. Plan-mode covers that gap.

DECISION-RULE-CLUSTER-ORDER: fixed not configurable
---------------------------------------------------
Rule cluster load order is HARD-CODED:
anti-stale -> coding-standards -> design-hard -> app-store -> source-tiers
-> slopsquatting -> risk-levels.
Configurable order would let entropy leak into deterministic-audit results
(same input + same auditor agent could yield different verdicts on
different sessions). Determinism > flexibility for audit.

DECISION-VISUAL-CFG-GATED: deferred to B-43
-------------------------------------------
`AuditorMode::Visual` exists in the source but is gated under
`#[cfg(feature = "visual_audit")]`. The feature is declared in
`crates/orchestrator/Cargo.toml` and defaults OFF. Without the flag the
variant is absent at compile time, so no match-arm obligation propagates
to downstream code. B-43 will (a) flip the default ON, (b) add the
AuditorVisual AgentRole variant, (c) wire dashboard rendering capture.

DECISION-IPC-VERSION: keep 2
----------------------------
Same rationale as B-40 DECISION-IPC: adding string variants to a
`#[serde(rename_all="snake_case")]` enum is a soft-compat change. The
IpcMessageType variant table is unchanged (still 16 tags); MAX_FRAME_SIZE
and FRAME_HEADER_SIZE are unchanged; orchestrator + agent binaries ship
together so older agents never receive an `auditor_concepts` dispatch.
IPC_PROTOCOL_VERSION stays at 2.

DECISION-SCHEMA-DELTA: 44 -> 45 (pending Slice 2)
-------------------------------------------------
Plan text said 28->29 (or 29->30 if B-40 ships first); actual working
tree is 44 (B-40 took 41, B-34 took 42, B-38 took 43, B-42 took 44).
B-41 bumps 44->45 atomically with the 3 new SsbActionType variants
(per_level_audit_started / _verdict / _cascade) and the canonical 8-site
SSB bijection lockstep. Implementation deferred to Slice 2 — see
CONTINUATION.

DECISION-PAPER: cite arXiv:2404.03602, not 2406.01297
-----------------------------------------------------
The B-41 plan + paper-ingestor note cite arXiv:2406.01297 (Kamoi TACL 2024)
as justification for "per-level scoping improves recall AND precision".
The paper does NOT make that exact claim. Closer empirical citation is
arXiv:2404.03602 (Kamoi COLM 2024, same author family) which shows prompt
scope changes recall by ~17-37pp without consistent direction (i.e. scope
matters but the sign depends on the task). Per-level scoping in B-41 is
therefore a DESIGN HYPOTHESIS to be validated by verdict telemetry post-
ship, not an empirically-grounded claim. Citation updated in
auditor_mode.rs module docstring.

DECISION-CASCADE: cascade to BabyStep tier, no new DecompositionLevel
---------------------------------------------------------------------
Mode BabyStepDetail cascades to the BabyStep tier (treats the batch as
belonging to its BabyStep parent) rather than introducing a 5th
DecompositionLevel::BabyStepDetail variant. Rationale:
- DecompositionLevel is consumed by `web_validation_tiers_for_level` and
  `NavigatorTree::integration_specs` keyed on `(DecompositionLevel, u64)`.
  Adding a 5th variant ripples through every keyed lookup with no
  corresponding tier structure (there is no node id distinct from
  BabyStepId for "the batch this BabyStep is in").
- BabyStepDetail is a line-level lens on the same node, not a separate
  tier. Cascading to BabyStep matches the conceptual relationship.
- TierNodeId in auditor_mode.rs intentionally has no BabyStepDetail
  variant — the typed sum stays bijective with DecompositionLevel.

DECISION-RULES-SEED-LOADER: forward dependency (Slice 3)
--------------------------------------------------------
`.elai_cc/crates/rules/seed/` does not exist yet. The 5 domain TOMLs
(design_hard, app_store, source_tiers, slopsquatting, risk_levels)
ship as the SPEC of what each rule cluster contains, but until a loader
in `rules/src/seed_loader.rs` (or extension to `project_rule_seeder.rs`)
actually reads `[seed_dir]/*.toml` into `rules.db`, the TOMLs are
inert. This is documented as forward-dep for a follow-up patch; the
per-level auditor dispatch in Slice 3 will load the rule clusters via
the existing project_rule_seeder path, with the seed_dir loader as a
separate add-on.

CONTINUATION (Slices 2-3, deferred)
-----------------------------------
Slice 1 SHIPPED in this session (cargo check workspace green, 8 unit tests
green in `auditor_mode::tests`). Slices 2-3 are NOT in this commit:

Slice 2 — SSB triplet + Evidence vacuous-default (estimated ~600 LOC):
  - ssb.rs: SsbActionType +3 variants atomic-8-tuple (enum, CHECK,
    as_sql_str, from_str, SsbPayload, action_type, payload structs,
    test_payload reconstructor), SCHEMA_VERSION 44->45.
  - autonomous_gate.rs Evidence struct: +5 fields (concept_zero_blocks,
    phase_zero_blocks, step_zero_blocks, babystep_zero_blocks,
    plan_zero_blocks), all `#[serde(default = "default_true_*")]`,
    extended `all_passed()` AND `summary()`.
  - Contract tests: `autonomous_gate_with_per_level_contract.rs` (new) +
    confirm 14 + 11 existing tests still pass UNCHANGED (vacuous default
    pattern from B-40).
  - 5 derive helpers in task_runner.rs querying per_level_audit_verdict
    rows. Wire at Evidence construction site.

Slice 3 — TreeNavigator dispatch + dashboard + agent.md + TOMLs (~1500 LOC):
  - pipeline_tree_navigator.rs: 5 audit insertion points (after Designer,
    after Concept-decomposer, after Phase-decomposer, after Step-decomposer,
    after BabyStep batch). on_tier_complete hook on TreeNavigator trait.
  - task_runner.rs: AUDITOR_BUBBLE_UP_CATEGORY wiring + 5 evidence-source
    helpers.
  - .claude/agents/auditor.md: APPEND MODE DISPATCH section (~480 LOC)
    with 6 per-mode REVIEW prompts.
  - dashboard-v2 AuditTimelineWindow.svelte + MonitorWindow.svelte badges
    + get_per_level_verdicts Tauri command + types.ts + stub.ts (LIVE
    invoke, no mock).
  - rules/seed/ directory + 5 TOMLs.
  - 4 remaining contract tests (per_level_audit_contract,
    rule_cluster_load_order, audit_artifact_persistence, dashboard wiring).

Justification for the slicing: full B-41 plan = ~3000+ LOC across 8+ crates.
A single-session execution that touches all of it risks the anti-spiral
pattern (CLAUDE.md HARD RULE) and produces an unreviewable diff. Slice 1
delivers a compile-verifiable, test-covered foundation that downstream
slices can build on without re-litigating type design.
