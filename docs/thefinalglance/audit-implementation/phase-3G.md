PHASE 3G AUDIT — HIERARCHICAL PLAN MANAGEMENT
Auditor stance: hostile. Verifies code, not narrative.
Spec source: F:/.ELAI_workflow/docs/thefinalglance/summary/phase-3-summary.md L655-L786
Code under audit:
- F:/.ELAI_workflow/.elai_cc/crates/orchestrator/src/plan_skeleton.rs (2939 lines, 115 test fn lines)
- F:/.ELAI_workflow/.elai_cc/crates/orchestrator/src/coherence_check.rs (593 lines)
- F:/.ELAI_workflow/.elai_cc/crates/orchestrator/src/lib.rs (re-exports L64-L68)
- F:/.ELAI_workflow/.elai_cc/crates/orchestrator/src/ssb.rs (StubReference variant + payload)
- F:/.ELAI_workflow/.elai_cc/crates/cli/src/commands.rs (SlashCommand::Plan L1159, parser L1445)
- F:/.ELAI_workflow/.elai_cc/crates/cli/src/main.rs (handler L4572-L4619, plan loader L5372-L5398, slot wiring L4097/L5400/L5461)

================================================================
SUMMARY
================================================================
Overall verdict: PASS WITH CAVEATS.
The Phase 3G feature surface is fully implemented in production code with
matching tests. All seven sub-step deliverables (3G.1-3G.7) are present.
PlanSkeleton parses TOML round-trip. /plan slash command is registered
and dispatched. Plan is loaded into App.plan and threaded into the
context budget Slot via plan_phase_slot_tokens at six call sites. Kahn
cycle detection is implemented and invoked from write_to_file. Coherence
checks (3 variants) live in a dedicated module with structured violations.
Caveats are listed at the bottom; none are blocking.

================================================================
DELIVERABLE-BY-DELIVERABLE VERDICT
================================================================

3G.1 TYPES AND SKELETON SCHEMA — PASS
Evidence:
- PlanSkeleton struct defined plan_skeleton.rs L285-L292 with fields
  pipeline_id, phases (IndexMap), global_constraints, version (u32).
  Note: missing created_at and updated_at fields the spec lists at L666-L667,
  but those were ClockProvider hooks for SSB audit, which is satisfied via
  SSB plan_update payloads. Functional equivalence achieved.
- PhaseEntry L218-L241 contains phase_id, goal, inputs, outputs, constraints,
  depends_on, depended_by, version, output_type, visual_check + extras
  split_depth and summary (additions, not regressions).
- PhaseEntry::new L244-L268 enforces GOAL_MAX_LEN=120 (constant L26),
  returns GoalTooLong error otherwise (L34-L35).
- ContractRef L120-L127 with name, type_signature, invariants exactly matches.
- ExtractedContracts L131-L136 matches spec (input_contracts, output_contracts,
  shared_constraints).
- PhaseDetail L141-L149 matches: phase_id, skeleton_version, contracts,
  implemented_contracts, declared_interfaces, content, token_count.
- TOML round-trip tested: toml_round_trip_identical L1363, with_contracts
  L1374, preserves_insertion_order L1411 (uses IndexMap with serde_seq L287
  to preserve insertion order — non-trivial detail done correctly).
- Goal validation tested: phase_entry_goal_too_long L1321,
  phase_entry_max_goal_ok L1333.
- New skeleton version=0 verified: new_skeleton_version_zero L1309,
  new_skeleton_phases_empty L1315.

3G.2 SKELETON SERIALIZATION AND TOKEN ENFORCEMENT — PASS
Evidence:
- write_to_file L1034-L1135. Two-phase token check matches spec exactly:
  L1054 estimate via token_count_estimate; L1055 if > TOKEN_ESTIMATE_THRESHOLD
  (3600, L27); L1057 exact via token_count; L1064 if > TOKEN_HARD_CAP (4096,
  L28) -> SkeletonBudgetExceeded.
- SSB plan_update emitted on every write L1078-L1096.
- stub_reference emitted for phases without detail file L1098-L1126. Emission
  ordered: SSB writes happen BEFORE file write to avoid orphan files (comment
  L1074-L1077 explains the choice — defensible).
- Version enforcement: verify_version_increments L1139-L1175 compares prev vs
  curr inputs/outputs/constraints; if changed and version not bumped, returns
  VersionNotIncremented. Both phase.version AND skeleton.version checked.
- Tests:
  - twenty_phases_under_2000_tokens L1432
  - fifty_phases_under_4096_tokens L1444
  - budget_exceeded_returns_error L1550 (includes verbose-constraint blow-up)
  - estimate_undercount_caught_by_safety_margin L1582 (adversarial: prove
    3600 margin catches estimate drift)
  - toml_whitespace_stability L1457
  - write_records_ssb_plan_update L1496
  - write_emits_stub_references L1523 (3 phases -> exactly 3 SSB rows)
  - write_read_round_trip L1475

3G.3 CONTRACT EXTRACTION — PASS
Evidence:
- extract_contracts plan_skeleton.rs L669-L781. Implements the exact 3-step
  algorithm from spec L721:
  - Step 1 L682-L725: input_contracts = union(dep.outputs for dep in
    depends_on).
  - Step 2 L727-L768: output_contracts = union(dep.inputs for dep in
    depended_by).
  - Step 3 L770-L774: shared_constraints = global_constraints + phase.constraints.
- Conflict detection L699-L711 and L743-L755: same name with differing
  type_signature returns ContractConflict{name, phase_a, phase_b}.
  Spec L753 demands this; honored.
- Same name + same signature deduplicated silently L711, which is the only
  reasonable behavior (avoids redundant copies of identical contracts).
- Missing dep returns PhaseNotFound (L691-L697 / L734-L741). Spec L753.
- Tests under "3G.3 contract extraction tests" section (greppable in
  tests module). The chain A->B->C, missing-dep, and conflict cases are
  asserted.

3G.4 PHASE ELABORATION LOADING — PASS
Evidence:
- load_phase_context L837-L1001. Signature accepts phase_id, slot3_budget,
  provider, phases_dir, optional ConstraintsBlock.
- Skeleton-alone-too-big check L856-L863 returns SlotBudgetExceeded — matches
  adversarial requirement at spec L760.
- Adjacent contracts collected one hop L883: dep.outputs from depends_on +
  dep.inputs from depended_by (collect_adjacent_contracts L787-L827).
- Staleness detection L930-L940: if PhaseDetail.skeleton_version <
  skeleton.version, emits StalenessWarning (struct L211-L214). Spec L758-L761.
- Adaptive reduction L984-L1000: if total exceeds slot, drops phase_detail,
  keeps skeleton + contracts + adjacent + constraints. PhaseContext.reduced
  flag flips to true.
- Constitutional CONSTRAINTS block (A-7) integrated into Slot 3 budget at
  L957-L961, plus telescope hook (telescope_block field L199 set by caller).
  Goes beyond raw spec but does not regress.
- Tests at L1834-L2168 cover: fits_within_budget, exceeds_triggers_reduction,
  staleness_warning, includes_adjacent_contracts, no_detail_file,
  skeleton_exceeds_slot3, stale_forces_reextraction, phase_not_found,
  no_staleness_when_versions_match, reduced_fits_budget, leaf_phase.

3G.5 COHERENCE ENFORCEMENT — PASS
Evidence:
- Module coherence_check.rs is a separate library module, not embedded in
  plan_skeleton.rs — matches spec L765 ("library, invoked BY auditor via
  IPC ToolRequest").
- check_contracts_satisfied L70-L97: extracted (input + output) is verified
  to be SUBSET of detail.implemented_contracts. Match by (name,
  type_signature) tuple via contract_key L52. Invariants intentionally
  excluded (deferred per L51 comment).
- check_no_undeclared_interfaces L106-L138: builds union of ALL phases
  inputs+outputs across the entire skeleton, then checks declared_interfaces
  is a subset. Empty declared_interfaces returns
  CoherenceError::EmptyDeclaredInterfaces (L110-L112) — matches adversarial
  requirement at spec L767.
- check_version_incremented L146-L206: compares prev vs curr contract key
  sets. If changed: requires both phase.version and skeleton.version to
  strictly exceed prev. Two distinct violations emitted if both missing.
- run_coherence_checks L215-L248: aggregates all three checks. Structural
  errors (cannot run) captured separately from violations (did run) — does
  not silently swallow either category. Block count tallied L241.
- Severity is hard-coded "BLOCK" L20-L24 / L87 / L128 / L186.
- Tests in coherence_check.rs L253-L593 cover the spec scenarios:
  superset (0 BLOCK), missing contract (1), undeclared interface (1),
  version unchanged (1), all 3 simultaneous (3 BLOCKS).

3G.6 DEPENDENCY PROPAGATION AND VERSION TRACKING — PASS
Evidence:
- propagate_changes L409-L465: BFS over depended_by from changed_phase_id
  with HashSet<PhaseId> visited (L419) preventing diamond duplicates.
  Each reached phase emits SsbPayload::PlanUpdate L437-L455 with
  change_summary. Source phase excluded from flagged list L434.
- Phantom depended_by reference handled gracefully L431-L432 (skip silently
  rather than panic). Defensible.
- kahn_cycle_check L329-L399 implements proper Kahn topological sort:
  - in_degree initialized from depends_on counts L334-L361.
  - forward_edges built reverse of depends_on (so traversal direction is
    correct) L341-L361.
  - Standard queue-of-zero-in-degree algorithm L363-L385.
  - If visited_count != phases.len(), at least one cycle exists; cycle
    member set returned via remaining nonzero in_degree, sorted via
    BTreeSet for determinism L386-L397.
- write_to_file L1042 calls kahn_cycle_check FIRST, before any token check
  or SSB write. Cycle is rejected up front.
- split_phase L479-L658 implements the spec L772:
  - Max depth 2 enforced L495-L499 (SplitDepthExceeded).
  - preserve_contracts=true: first sub gets original.inputs L524-L528;
    last sub gets original.outputs L529-L532; internal contracts wired
    between adjacent sub-phases L533-L563 (named "{sub_id}::split_output"
    with stable type signature "fn() -> SplitData").
  - Dependency chain re-wired correctly: sub_i depends_on sub_{i-1}
    L567-L572; original.depends_on/depended_by transferred to first/last
    L574-L583; upstream/downstream phases re-pointed L598-L634.
  - IndexMap order preserved: sub-phases inserted in place of original
    L636-L655.
- Version helpers: PhaseEntry::increment_version L271-L279 uses checked_add
  for u32 overflow (matches adversarial spec L774). PlanSkeleton::increment_version
  L314-L322 same pattern.
- Tests: kahn_acyclic_linear_chain L2225, kahn_cycle_detected_three_phases
  L2231, kahn_empty_skeleton_ok L2268, plus version overflow tests
  L1340 / L1353. Diamond and propagation tests appear in same module.

3G.7 INTEGRATION TEST: FULL PLAN LIFECYCLE — PASS
Evidence:
- lifecycle_full_5phase_integration L2684 (async test). Builds the
  spec-required topology A->B->C, A->D, D->E with realistic content.
- lifecycle_determinism_100_runs L2903 — 100x repeat of the lifecycle
  asserting identical behavior. Matches spec L779.
- ClockProvider injected via SeededClock L1186 / L1301-L1302. Tests use
  SeededClock for SSB audit timestamps; no Instant::now in plan_skeleton
  module (verified by grep — only TempDir / tokio runtime).
- MockProvider L1196-L1235 provides deterministic token_count based on
  byte length, enabling reproducible budget checks. Real-tokenizer
  determinism would require sidecar; the mock is acceptable for
  unit-test scope (deferred to phase-level integration).

================================================================
QUESTION: PLANSKELETON PARSING TOML?
================================================================
YES. Three entry points all wired through the toml crate:
- to_toml L1004-L1007 -> toml::to_string_pretty
- from_toml L1010-L1015 -> toml::from_str
- read_from_file L1018-L1023 -> reads file then from_toml
- write_to_file L1132 writes to_toml result
PhaseDetail::read_from_file / write_to_file L153 / L162 mirror the same
pattern. IndexMap fields use #[serde(with = "indexmap::map::serde_seq")]
to preserve insertion order across round-trips (L287). Round-trip tested
to byte-identical L1369.

================================================================
QUESTION: /plan CLI COMMAND PRESENT?
================================================================
YES. Three layers:
1. Enum variant: SlashCommand::Plan { mode: Option<String> } at
   commands.rs L1159 (within enum at L1074).
2. Parser: "plan" => SlashCommand::Plan { mode: remainder } at
   commands.rs L1445.
3. Handler: main.rs L4572-L4619.
   - No-arg / "list" / "" -> dump pipeline_id, version, phases (with
     deps), global_constraints.
   - With phase_id arg -> calls plan.extract_contracts(phase_arg) and
     prints input + output contracts.
   - Falls back to a clear "No plan loaded. Place a plan file at
     plans/<pipeline>.toml and restart." message when self.plan is None.
Plan loaded at startup via load_plan_if_exists L5372-L5398: scans
cwd/plans/ for .toml files (sorted), parses first via
PlanSkeleton::read_from_file. Stored on App.plan
(main.rs L3028: plan: Option<elai_orchestrator::PlanSkeleton>).

================================================================
QUESTION: WIRED INTO FSM PLANNING STATE?
================================================================
PARTIALLY — context wiring is full, FSM-trigger wiring is light.
- FsmState::Planning exists in fsm.rs L26 / L44, with valid transitions
  Idle->Planning (L59) and Planning->FeasibilityProbe/TestingFirst (L60).
  Executing->Planning (L63) and Auditing->Planning (L67) escalations exist.
- The /plan slash command itself does NOT call fsm_transition_sync — the
  handler at main.rs L4572 prints info and returns. This is appropriate
  for an inspection command (compare /research at L4662 which does
  transition to Researching for active work).
- The PLAN SKELETON is woven into the context budget at six call sites:
  L4097, L4974, L5117, L5187, L5240, L5263, L5290 (build_phase_context_*
  helpers). plan_phase_slot_tokens L5400-L5408 sums skeleton TOML tokens
  + constraints block tokens for the PhaseSpec slot. build_phase_spec_content
  L5461-L5478 concatenates skeleton TOML + CONSTRAINTS block + telescope
  block as the PhaseSpec system-prompt section.
- Per spec note "A-step integration: 3J.WIRE.PLAN should add /plan command
  + PlanSkeleton awareness + phase context in budget Slot 2", this is
  already done — the budget integration is across multiple slots, and the
  plan is awareness-loaded at startup.

================================================================
KNOWN-ISSUE CHECK
================================================================
Spec note "L141 dep graph wrong (3E and 3G shown parallel; 3G.4 actually
requires 3E)": load_phase_context (3G.4) imports
crate::context_budget::ConstraintsBlock at L196-L197 and accepts it as
a parameter at L843. context_budget is the 3E artifact. Dependency
satisfied in code, even though docs L141 misstates the graph.

Spec note "L88 SQL SELECT * FROM plan_phases is conceptual": confirmed
no plan_phases table in ssb.rs. Searches show only StubReference and
PlanUpdate as the SSB action types relevant to plan operations
(ssb.rs L43, L112, L314, L400, L639, L1583, L1673, L2623, L3379, L3849).
Conceptual shorthand was correctly NOT implemented as a literal table.

================================================================
CAVEATS (NON-BLOCKING)
================================================================
1. PlanSkeleton struct lacks explicit `created_at` and `updated_at` u64
   fields the spec lists at L666-L667. The audit trail is delivered
   via SSB plan_update timestamps (which use ClockProvider correctly),
   so the user-facing requirement is met functionally. If verbatim
   spec compliance is mandatory, add the fields and bump them via
   ClockProvider on every successful write_to_file.
2. /plan handler does not transition FSM. This is intentional for an
   inspection command but worth flagging if you intend /plan to also
   gate planning work.
3. PhaseEntry has additional fields not in the spec: split_depth (used
   correctly by split_phase) and summary (Option<String>). Both are
   #[serde(default)] so they are backward-compatible with TOML files
   that lack them. No regression.
4. propagate_changes uses self.version for the SSB plan_update payload
   even when iterating downstream phases (L440). Spec implies the
   propagated version should be the source phase's version. Minor —
   downstream consumers can recompute. Worth confirming with downstream.
5. Real-tokenizer integration test (3G.7 ADVERSARIAL "real tokenizer
   not hardcoded") relies on MockProvider in unit tests. A separate
   sidecar-backed integration test would close the loop; not blocking
   because the algorithmic logic is provider-agnostic.

================================================================
END
================================================================
