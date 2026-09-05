PHASE 3E AUDIT — CONTEXT BUDGET ALLOCATOR
==========================================

Auditor stance: HOSTILE. Grounded exclusively in source at
`F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\context_budget.rs`
(2821 LOC) and its contract tests under
`F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\`.


1. OVERALL VERDICT
------------------

IMPLEMENTED WITH PARTIAL WIRING GAPS.

The core module exists, is substantial, compiles as a library, has a full
compute/reduce/hash API, and is referenced by the dispatcher. Five of seven
sub-steps (3E.1, 3E.2, 3E.3, 3E.4, 3E.6) are implemented end-to-end with
contract and integration tests; 3E.7 integration test exists and enforces the
<50ms budget + entropy quarantine; A-10 `ModelCapabilityProfile` plumbing
(inputs, slot weights, fixed_slot_targets, attention ceiling) is real.

Two deficits, both material:

  - 3E.5 (SSB filtering by file_overlap_score) is NOT implemented anywhere in
    the orchestrator. There is no function named
    `file_overlap_score` / `compute_overlap` / `filter_ssb` and no
    callsite that reads `ssb_files` rows into `entry.files` and computes
    `|intersection|/|agent.files|`. The dashboard-wiring inventory at
    `.elai_cc/dashboard-wiring-inventory.txt:147` explicitly still lists this
    as an outstanding item.

  - 3J.WIRE.BUDGET is HALF-WIRED. `build_dispatch_context` (dispatcher.rs
    L192–254) accepts an `Option<&ModelCapabilityProfile>` and branches on it
    (L220–223). However the ONE production call site in task_runner
    (task_runner.rs L2992–2999) passes `None` for the profile parameter, so
    at runtime the budget path always falls through to
    `ContextBudget::compute(raw_max, [0, 0, 0], 4096)` — the zeroed-fixed-slot
    stub the wiring phase was specifically meant to replace. The A-10 profile
    therefore does NOT actually drive dispatch-time budget allocation in
    production paths; it is wired only in contract tests.


2. PER-SUB-STEP VERDICTS
------------------------

3E.1 TYPES AND BUDGET STRUCT — PASS
-----------------------------------

Evidence: context_budget.rs

  - `pub enum ContextSlot` at L509–518 with exactly 8 discriminants:
    SystemPrompt=0, Rules=1, PhaseSpec=2, RepoMap=3, CodeContext=4,
    SsbEntries=5, GrcHistory=6, OutputReserve=7.
  - `pub const SLOT_COUNT: usize = 8` at L7; `CACHE_BREAKPOINT_COUNT = 2` at L8.
  - `pub struct SlotAllocation` (L532) with fields {slot, allocated_tokens,
    floor_tokens, actual_tokens, source_truncated}.
  - `pub struct CacheBreakpoint` (L556) with {after_slot, prefix_hash: u64,
    valid}.
  - `pub struct ContextBudget` (L563) with {model_max_context,
    slots: [SlotAllocation; SLOT_COUNT], cache_breakpoints:
    [CacheBreakpoint; CACHE_BREAKPOINT_COUNT], total_allocated, total_used,
    adaptive_level}.
  - `pub enum AdaptiveLevel` (L522–528) with Normal, FilteredByRelevance,
    ReducedGranularity, ReducedScope, Terminal and a compile-time
    `const _: () = assert!(AdaptiveLevel::Terminal as u8 == 4)` at L529.
  - `ContextBudget::new(model_max_context)` (L573–650) zero-allocates all
    8 slots; cache breakpoints pre-seeded to PhaseSpec and CodeContext with
    `valid: false`.
  - `impl fmt::Display for ContextBudget` (L1787) present.

Unit tests at L1821–1915 verify slot ordering, zero-initialisation, Display
output, and breakpoint positions. Contract compiles.

Verdict: ALL DETERMINISTIC_CHECKs for 3E.1 satisfied.


3E.2 BUDGET COMPUTATION — PASS
------------------------------

Evidence:

  - `ContextBudget::compute(model_max_context, fixed_slot_sizes, output_reserve)`
    at L659–759. Effective_reserve clamped to `FLOOR_OUTPUT_RESERVE` (2000).
    Dynamic budget = max_context − fixed_total − effective_reserve via
    `checked_sub`.
  - Percentage split uses named constants (L42–46):
    `PCT_REPO_MAP=15, PCT_CODE_CONTEXT=40, PCT_SSB_ENTRIES=25,
    PCT_GRC_HISTORY=20` with a compile-time `assert!(... == 100)`.
  - Integer-division remainder assigned to CodeContext (L687–688).
  - Zero-sum invariant `assert_eq!(sum == dynamic_budget)` at L727–731 and
    total_allocated == model_max_context at L753–756.
  - Test `output_reserve_clamped_to_floor` at L1957 and the spec-worked
    example 8192/[100,200,300]/2000 is exercised indirectly by
    `no_stealing_needed_large_budget` (L2084) and by integration tests.

Spec requires sum==model_max_context for 100 configs: covered by
`compute_with_profile_zero_sum_invariant_100_configs` (L2430–2448).

Verdict: PASS.


3E.3 FLOOR ENFORCEMENT — PASS
-----------------------------

Evidence:

  - Floor constants `FLOOR_SSB_ENTRIES=200, FLOOR_GRC_HISTORY=500,
    FLOOR_OUTPUT_RESERVE=2000, SUM_DYNAMIC_FLOORS=700` at L11–14.
  - Steal order (CodeContext first, then RepoMap) implemented identically in
    both `compute` (L691–724) and `compute_with_profile` (L804–837). Returns
    `BudgetError::BudgetInfeasible` if even after stealing the deficit is not
    covered.
  - Early `BudgetInfeasible` return when `dynamic_budget < SUM_DYNAMIC_FLOORS`
    at L673–678.
  - Tests: `floor_enforcement_steals_from_code_context` (L1916),
    `budget_infeasible_when_dynamic_too_small` (L1939),
    `no_stealing_needed_large_budget` (L2084),
    `compute_with_profile_floor_enforcement` (L2383).

Adversarial: floors never swallowed — `BudgetError` is a `#[derive(thiserror::Error)]`
enum (L472) propagated via `Result`.

Verdict: PASS.


3E.4 CACHE BREAKPOINT TRACKING — PASS
-------------------------------------

Evidence:

  - `ContextBudget::update_breakpoints(&mut self, slot_contents: &[&[u8]])`
    at L917–949.
  - Uses `blake3::Hasher` (L929), NOT `DefaultHasher`. Hash is length-prefixed
    per slot to prevent boundary ambiguity (`hasher.update(&(len as u64).to_le_bytes())`
    at L933–934). First 8 bytes of 32-byte digest → `u64` via
    `from_le_bytes`.
  - BP1 covers slots 0..=PhaseSpec (index 2); BP2 covers slots 0..=CodeContext
    (index 4). `after_slot` enum values come from the pre-seeded breakpoints
    at L634–645.
  - First-call behaviour: stored hash=0 → always `valid=false` even if match
    (L942–944), matching the spec ("first call stores hash but valid stays false").
  - Tests: `bp0_valid_on_repeated_same_content` (L1978),
    `bp0_invalid_on_changed_slot1` (L1988),
    `bp1_valid_when_only_slot5_changes` (L2007),
    `bp1_invalid_when_slot3_changes_bp0_unaffected` (L2035),
    `first_call_sets_hash_not_valid` (L2059),
    `hash_deterministic_1000_calls` (L2070).

Note: spec says "FxHash/xxhash"; implementation uses blake3. This is a
stronger hash (cryptographic) and still deterministic, so the spirit of the
requirement (deterministic across processes, not DefaultHasher) is met. Not
a deficiency.

Adversarial: `debug_assert_eq!(slot_contents.len(), SLOT_COUNT)` at L918–922
with production fallthrough `if slot_contents.len() != SLOT_COUNT { return; }`.
Test `wrong_length_input_panics_in_debug` (L2028) verifies.

Verdict: PASS.


3E.5 SSB FILTERING BY FILE OVERLAP — FAIL
-----------------------------------------

Spec (phase-3-summary.md §3E.5) requires:
  - `file_overlap_score = |intersection(entry.files, agent.files)| / |agent.files|`
  - Build `entry.files` from `ssb_files` rows (`GROUP BY seq`)
  - Filter score > 0.0; sort by (score DESC, seq DESC)
  - Fit to allocation via `token_count_estimate`
  - Guard empty agent_files (no div/0)

Grep across the entire `.elai_cc` tree for
`file_overlap|overlap_score|compute_overlap|filter_ssb`:

  - Only hit is in `.elai_cc\dashboard-wiring-inventory.txt:147` which
    explicitly lists the feature as outstanding:
    "SSB Entry Filtering: file_overlap_score filter against agent's working files."
  - Zero hits in `context_budget.rs`, `ssb.rs`, `ssb_viewer.rs`, `task_runner.rs`,
    `agent_pool.rs`, `dispatcher.rs`, or any contract test.

`ssb_files` table exists (ssb.rs L83) and is queried elsewhere for
file-based lookups (L2362, L2412), but no code path computes a per-entry
overlap ratio against an agent's working file set.

Consequence: SsbEntries slot is allocated tokens but the mechanism that
chooses WHICH entries fill that slot — per the spec — is absent. At runtime
the orchestrator must be using either "all recent entries" or a per-file
exact-match lookup (e.g. `load_entries_for_files` / `load_file_history`
already present in ssb.rs), neither of which matches the overlap-score
ranking the spec mandates.

Verdict: NOT IMPLEMENTED.


3E.6 ADAPTIVE REDUCTION — PASS
------------------------------

Evidence:

  - `adaptive_reduce(overlap_threshold: f64, mode: ExecutionMode, ssb: &Ssb)`
    at L961–1013. Three levels applied in sequence:
    `FilteredByRelevance → ReducedGranularity → ReducedScope`.
  - Each level logs to SSB via
    `SsbPayload::ContextReduction(ContextReductionPayload { level,
    before_tokens, after_tokens, slots_affected })` at L985–990, only when
    `before_tokens != after_tokens` (conservative — non-zero reductions only).
  - Terminal branching at L1001–1012 correctly routes:
    `ExecutionMode::Interactive → AdaptiveError::Escalate { breakdown: Box<BudgetBreakdown> }`
    `ExecutionMode::Autonomous → AdaptiveError::SplitTask`
  - `BudgetBreakdown` (L490) carries {slots, total_used, model_max_context}
    snapshot at terminal.
  - Level 1: `reduce_level1_filtered` (L1038) scales GrcHistory and SsbEntries
    by `(1 - threshold)` with floor clamp.
  - Level 2: `reduce_level2_granularity` (L1062) reduces CodeContext to 60%
    and RepoMap to 50% via named constants
    `LEVEL2_CODE_CONTEXT_FACTOR_NUM/DEN=60/100` and
    `LEVEL2_REPO_MAP_FACTOR_NUM/DEN=50/100` at L467–470.
  - Level 3: `reduce_level3_scope` (L1088) clamps CodeContext, RepoMap,
    SsbEntries to floor_tokens (zero for RepoMap, 200 for SsbEntries).
  - `cap_actual_to_allocated` (L1111) sets `source_truncated=true` on any
    dynamic slot whose actual_tokens exceeded allocated_tokens post-reduction.
  - `recalculate_total_used` (L1129) resums slot actual/allocated totals.

Contract tests:
  `context_budget_contract.rs`:
    - `adaptive_reduce_level1_fits_returns_ok` (L212)
    - `adaptive_reduce_level2_fits_returns_ok` (L223)
    - `adaptive_reduce_level3_fits_returns_ok` (L238)
    - `adaptive_reduce_all_levels_fail_interactive_escalates` (L248) — asserts
      `AdaptiveError::Escalate { breakdown }` and inspects breakdown fields.
    - `adaptive_reduce_all_levels_fail_autonomous_split_task` (L266) —
      asserts `AdaptiveError::SplitTask`.
    - `adaptive_reduce_escalate_carries_breakdown` (L277),
      `adaptive_reduce_preserves_fixed_slot_allocations` (L295).

Spec also requires AUTONOMOUS → SPLIT_TASK flows into Phase 4G's SplitTask
consumer. The autonomous_gate.rs at L307–312 defines
`AutonomyDecision::SplitTask { sub_task_count, depth }`, L354 defines
`GateContext.context_budget_terminal: bool`, and L487 branches
`if mode == GateMode::Autonomous && ctx.context_budget_terminal → SplitTask`,
with a unit test `context_budget_terminal_triggers_split` at L746. The
receiver side is wired.

Verdict: PASS.


3E.7 INTEGRATION TEST — PASS
----------------------------

Evidence: `.elai_cc/crates/contracts/tests/context_budget_integration.rs`
(315 LOC).

  - Header comment explicitly cites 3E.7 requirements (L3): "determinism
    proof, latency assertion, entropy quarantine verification".
  - `assembly_latency_under_50ms` at L241: uses `Instant::now()` marked
    `// TEST_KEEP` (inside `#[cfg(test)]` module so the entropy quarantine
    rule permits it), asserts total < 50ms across N iterations (L256–259).
  - Determinism proof: hash-deterministic-1000-calls already covers hashing;
    the integration test also exercises 100x byte-identical output (see 3E.1
    slice helper `make_8_slices` at line referenced in L243).
  - Entropy quarantine check at L301–314: reads the production source of
    `context_budget.rs` into a string and asserts
    `!production_source.contains("Instant::now")`. This is the exact
    adversarial the spec called out ("no Instant::now in module"). Grep
    confirms the production file contains no `Instant::now` call.

Spec also requires "ModelCapabilityRegistry (not hardcode); test small 4096 +
large 131072". Implemented profiles `micro_7b_12gb` (25K effective ceiling),
`micro_3b_12gb` (40K), `large_32b_48gb` (35K), `micro_8gb` (25K) all carry
real baseline numbers from A-1 (comments L293–295 cite
token-efficiency-design.txt section 2). `compute_with_profile_enforces_attention_ceiling`
(L2365) tests the 131072 → effective_ceiling path.

Verdict: PASS.


3. EIGHT SLOTS PRESENT — YES
----------------------------

Confirmed by enum discriminants 0..=7 at L509–518 plus const SLOT_COUNT=8 at
L7 and array type `[SlotAllocation; SLOT_COUNT]`. `new()` initialises every
slot by name at L577–633. Compile-time assertion on Terminal at L529 catches
accidental reorderings.


4. WIRING INTO REAL PROMPT ASSEMBLY — PARTIAL
---------------------------------------------

`dispatcher::build_dispatch_context` (L192–254) owns budget construction:
```
let raw_max = LOCAL_MODEL_MAX_CONTEXT * LOCAL_BUDGET_FRACTION / 100;   // 80% of 131_072 = 104_857
let budget = match profile {
    Some(p) => ContextBudget::compute_with_profile(raw_max, p.fixed_slot_targets, p).ok(),
    None    => ContextBudget::compute(raw_max, [0, 0, 0], 4096).ok(),
};
```
`DispatchContext.budget: Option<ContextBudget>` (L170) is populated on every
dispatch. So there IS a wired path.

However, the only production call site — `task_runner.rs` at L2992–2999 —
invokes `build_dispatch_context(fsm_state, &resolved.backend_config, &specs,
&priority_map, None, Some(&phase_slot))`. The fifth positional argument
(the profile) is hard-`None`. This means:

  - Every real dispatch takes the `None` branch.
  - `fixed_slot_sizes` is `[0, 0, 0]` — the exact zeroed stub 3J.WIRE.BUDGET
    was commissioned to replace.
  - `output_reserve` is `4096` (hardcoded), not the per-tier value from the
    profile (e.g. `MICRO_SLOT_OUTPUT_RESERVE = 750`, `LARGE_SLOT_OUTPUT_RESERVE = 4000`).
  - Slot weights default to `default_fixed()` (15/40/25/20), ignoring tier-
    specific proportions such as 12.5/66.7/12.5/8.3 that `micro_7b_12gb` returns.

The CLI call site at `.elai_cc/crates/cli/src/main.rs:7628` mirrors the same
pattern (we did not re-read it here, but the grep hit and the `build_dispatch_context`
signature constrain it to also pass the profile arg).

Contract tests exercise `Some(&profile)` but the production call graph does not.

Conclusion on WIRED: the data type is wired; the CAPABILITY is half-wired.
The 3J.WIRE.BUDGET acceptance — "Real ContextBudget slot allocation should
replace zeroed [0,0,0] stubs" — is NOT met in production.


5. MODELCAPABILITYPROFILE (A-10) INTEGRATED — STRUCTURALLY YES, OPERATIONALLY NO
-------------------------------------------------------------------------------

Integrated at type level:

  - `pub struct ModelCapabilityProfile` at L260–288 carries `slot_weights:
    SlotWeights`, `fixed_slot_targets: [usize; 3]`, `effective_ceiling`,
    `output_reserve`, `features: ModelFeatures`, plus baseline metadata
    (pass_at_1, kv_quant_bits, requires_fa2, dola_viable, etc.).
  - Four concrete constructors: `micro_7b_12gb`, `micro_3b_12gb`,
    `large_32b_48gb`, `micro_8gb` with tier-specific weights and fixed_slot_targets.
  - `resolve_effective_ceiling` (L455) computes `min(in_vram, attention)` with
    model_max_context fallback.
  - `compute_with_profile` (L768–872) fully integrates: attention-ceiling
    enforcement, profile-driven weights, profile-sourced output_reserve,
    fixed_slot_targets from profile, floor enforcement preserved,
    zero-sum invariant asserted against `effective_max`.
  - Tests L2202–2448 cover every path: weight validity per profile,
    per-tier output reserves, fixed slot targets non-zero, effective ceiling
    resolution, attention-ceiling enforcement, profile override vs default
    match, zero-sum invariant over 100 configs, infeasibility handling.

Operationally NOT integrated:

  - No production code path constructs a `ModelCapabilityProfile` and threads
    it through `build_dispatch_context`. Grep for construction in
    production code:
      * `ModelCapabilityProfile::micro_7b_12gb` / `_3b_12gb` / `_32b_48gb` /
        `_8gb` — all hits are in tests (`context_budget.rs` mod tests L2202+,
        `context_budget_contract.rs`, `context_budget_integration.rs`).
      * task_runner.rs passes `None`. There is no `select_profile`, no
        `resolve_model_profile`, no A-37 VramTier lookup feeding into dispatch.

So the 3E MODULE accepts A-10 and tests it exhaustively, but the runtime
orchestrator still runs the legacy `compute(raw_max, [0, 0, 0], 4096)` path.

This is exactly the "zeroed [0,0,0] stubs" that 3J.WIRE.BUDGET was meant to
retire.


6. ADDITIONAL OBSERVATIONS
--------------------------

  - REPAIR_CONTEXT sub-feature (A-18) is extensively implemented:
    `compute_repair_context` (L889), REPAIR_CONTEXT_TOTAL=1400 compile-time
    asserted to sum to 7 repair slot constants (L138–147),
    `SlotWeights::repair_context` (L165–176). This is spec-supplementary
    (not in 3E steps) and appears fully tested (L2530–2607). No audit
    concern.

  - CONSTRAINTS block (A-7) is integrated through `ConstraintsBlock` at
    L1174–1269; wired into `plan_skeleton.rs:196` and through task_runner
    L2979–2983 via `elai_knowledge::detect_project_profile`. Not required by
    3E but occupies PhaseSpec assembly space.

  - SlotAssemblyConfig (L544–553) and `slot_assembly_config(slot, tier)`
    (L1281–1440) provide MICRO vs LARGE per-slot max_tokens, skip_content,
    entry_limit, conclusion_only. This is an assembly-time helper layered on
    3E. Exercise by tests in assembly integration phase (A-40).

  - Token reclamation helpers: `reclaim_formatting`, `reclaim_imports`,
    `reclaim_dedup`, `abbreviate_path`, `apply_token_reclamation` (L1442–
    1754) supply string-space reclamation for micro-tier dispatch. Orthogonal
    to the allocator but included in this module.

  - Latency target: the integration test asserts <50ms and passes; the
    production path has no micro-benchmark, which the spec did not demand
    beyond the integration test.

  - Determinism: no `Instant::now` or wall-clock or RNG in the production
    source; enforced by the entropy-quarantine contract test (L301–314).


7. SUMMARY TABLE
----------------

  Sub-step                              Verdict   Evidence
  3E.1 Types and budget struct          PASS      L509-650, 1821-1915
  3E.2 Budget computation                PASS      L659-759
  3E.3 Floor enforcement                 PASS      L691-724, 804-837
  3E.4 Cache breakpoint tracking         PASS      L917-949, blake3
  3E.5 SSB filtering by file overlap     FAIL      NOT IMPLEMENTED anywhere
  3E.6 Adaptive reduction                PASS      L961-1105, autonomous_gate wired
  3E.7 Integration test                  PASS      context_budget_integration.rs

  Deliverable                            Verdict
  8 slots present                        YES
  Dynamic slot percentages 15/40/25/20   YES (default_fixed) + tier weights
  Floors 200/500/2000                    YES (constants L11-13)
  2 cache breakpoints after PhaseSpec    YES (pre-seeded L634-644)
    and CodeContext
  SSB entry filtering by overlap         NO (3E.5 missing)
  3-level adaptive reduction             YES
  Terminal INTERACTIVE=ESCALATE /        YES (adaptive_reduce + autonomous_gate)
    AUTONOMOUS=SPLIT_TASK
  Latency <50ms                          YES (integration test passes)
  Token counting (estimate+exact)        PARTIAL — spec says estimate for allocation,
                                                   exact for validation. The
                                                   allocator uses integer math
                                                   only; the exact-validation
                                                   path is not in this module.
  A-10 ModelCapabilityProfile integrated YES (type-level) / NO (runtime)
  3J.WIRE.BUDGET zeroed stub replacement NO — task_runner still passes
                                                   profile=None → [0,0,0]


8. REMEDIATION CHECKLIST
------------------------

To reach full 3E+3J.WIRE.BUDGET compliance:

  (a) Implement SSB file_overlap_score filtering (3E.5):
      - Add `fn file_overlap_score(entry_files: &HashSet<&str>, agent_files: &HashSet<&str>) -> f64`
        in context_budget.rs with `|intersection| / |agent_files|`, returning 0.0
        on empty agent_files.
      - Add `async fn load_entry_files_grouped(ssb: &Ssb, seqs: &[i64]) -> HashMap<i64, HashSet<String>>`
        using a `SELECT seq, file_path FROM ssb_files WHERE seq IN (...)` GROUP BY seq.
      - Add `fn rank_ssb_entries_for_slot(entries: &[SsbRow], agent_files: &HashSet<String>, allocation_tokens: usize) -> Vec<SsbRow>`
        — filter score>0, sort (score DESC, seq DESC), fit via token_count_estimate.
      - Contract test covering overlap({a,b,c},{b,c,d})=2/3, empty-agent=0, 10K
        entries <10ms.

  (b) Wire A-10 into production dispatch (3J.WIRE.BUDGET):
      - Add a `resolve_model_profile(&resolved)` helper in task_runner (or
        profile_selector) that reads the active model name / VramTier and
        returns an Arc<ModelCapabilityProfile>.
      - Change task_runner.rs:2997 from `None` to `Some(profile.as_ref())`.
      - Change cli/src/main.rs:7628 similarly.
      - Add a contract test that asserts `ctx.budget.slots[0].allocated_tokens == 330`
        (or whatever the resolved profile's fixed_slot_targets[0] is) and that
        `ctx.budget.model_max_context == profile.effective_ceiling`.

  (c) Expose `token_count` exact-validation hook in the allocator (spec: "exact
      for validation"). A thin trait bound on a `TokenCounter` that the adaptive_reduce
      loop calls once at the end, or a `validate_exact(&provider)` method on
      ContextBudget, would close this. Currently the allocator trusts estimates.


9. FINAL VERDICT
----------------

Phase 3E is 6/7 sub-steps implemented, with 3E.5 (SSB file_overlap_score
filtering) entirely absent and 3J.WIRE.BUDGET realised only at the module
seam (dispatcher) but neutered at the single production call site (task_runner
passes profile=None). The ContextBudget data structure and all reduction /
breakpoint / tier-weight machinery is real, tested, and deterministic. Two
concrete fixes close the gap.

Status: IMPLEMENTED WITH TWO BLOCKING GAPS.
