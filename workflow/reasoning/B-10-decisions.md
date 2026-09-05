B-10 ContextBudget Profile Activation + 3E.5 file_overlap_score
Session s-28972, 2026-04-26
FSM: TESTING_FIRST -> EXECUTING -> AUDITING

DECISION-A — true wiring gap = task_runner.rs:3131 (NOT the 5 dispatch_context_profile:None sites)
  Source: context-package-B-10.md lines 16-25, file:task_runner.rs:1857/3131/6862/6980/7175/7930
  The 5 `dispatch_context_profile: None` field initializations at lines 1857/6862/6980/7175/7930 are
  the GenerationProfileSelector trait field (FU.3 / A-18 batch-fu) — picks FirstAttempt vs RepairRetry
  per dispatch. They are SEPARATE from the ModelCapabilityProfile arg the spec actually wants wired.
  The TRUE production gap is the SINGLE call at task_runner.rs:3131 where build_dispatch_context's
  5th positional arg `profile: Option<&ModelCapabilityProfile>` is hardcoded to None, falling through
  to dispatcher.rs:222 `ContextBudget::compute(raw_max, [0,0,0], 4096)` (the zeroed-stub fallback).
  ACTION: fix only line 3131 (orchestrator) + line 8032 (cli/main.rs REPL). Do NOT touch the 5
  dispatch_context_profile field defaults — out of scope.

DECISION-B — model_name on LlmProvider trait
  Source: file:providers/src/providers/ollama.rs:352-357 (trait surface), file:sidecar_client.rs:63
  (SidecarConfig.model), file:vllm_client.rs:37 (VllmConfig.model), file:ollama.rs:361/599 (model field
  on OllamaProvider/OllamaEmbedder)
  Add `fn model_name(&self) -> &str` with default impl returning "" so external impls keep compiling.
  Override in 4 internal impls: OllamaProvider (returns &self.model), OllamaEmbedder (returns
  &self.model), SidecarProvider (returns &self.config.model), VllmProvider (returns &self.config.model).
  OpenAiCompatClient does NOT impl LlmProvider — it uses the Provider/MessageRequest path; not in
  scope. No new dependencies.

DECISION-C — SSB variants (2 new) + 14-site coordinated edit
  Source: file:ssb.rs:39-70 (CHECK constraint), file:ssb.rs:96-295 (enum), file:ssb.rs:299-376
  (as_sql_str), file:ssb.rs:385-462 (from_str), file:ssb.rs:1569-1653 (SsbPayload), file:ssb.rs:1655-
  1736 (action_type()), file:ssb.rs:2580+ (test_payload helper), file:tier3-drawers-backend.mjs:478-491
  (pin 74 -> 76)
  Add SsbActionType::ContextBudgetComputed and SsbActionType::SsbFilteredByOverlap.
  Payload structs:
    ContextBudgetComputedPayload { tier: String, raw_max: usize, slots: Vec<u64>,
                                   allocations: Vec<usize>, adapt_levels_fired: u8 }
    SsbFilteredByOverlapPayload { entries_total: usize, entries_kept: usize,
                                  overlap_threshold: f32 }
  14-site edit list:
    1. SsbActionType enum (2 new variants) — ssb.rs ~L295 (just before closing brace)
    2. SsbActionType::as_sql_str() match arms (2 new) — ssb.rs L300-376
    3. ssb_action_type_from_str() match arms (2 new) — ssb.rs L385-462
    4. New payload structs (2) — ssb.rs near L719 (after ContextReductionPayload)
    5. SsbPayload enum (2 variants) — ssb.rs L1569-1653
    6. SsbPayload::action_type() (2 arms) — ssb.rs L1658-1735
    7. CHECK constraint string in CREATE_SSB_TABLE — ssb.rs L39-70 (add 2 new strings)
    8. SCHEMA_VERSION 21 -> 22 — ssb.rs L26
    9. test_payload_for_action_type() (2 default constructors) — ssb.rs L2580+
   10. all_test_payloads() / proptest arb_payload() (2 entries each) — ssb.rs ~L3900+/L4232+
   11. Doc comment update on SsbPayload (74 -> 76 variants) — ssb.rs L1556-1568
   12. tier3-drawers-backend.mjs pin 74 -> 76 — tier3 contract test L478-491
   13. New unit tests for variant from_str / as_sql_str roundtrip — ssb.rs tests mod
   14. Decision doc entry (this file) justifying the variant additions
  Rationale: dashboard reads SSB rows for the per-slot budget bar chart and overlap-filter
  transparency; per spec Tasks 3-4 these emit boundaries are required.

DECISION-D — per-tier slot weights: SPEC TABLE wins
  Source: docs/specs/guide/token-efficiency-design.txt §2 (lines 465-543), file:context_budget.rs:38
  (FIXED_SLOT_TELESCOPE_TARGET=1050 universal divergence)
  Spec §2 table is design source-of-truth. Updating constructors to match. Keep
  FIXED_SLOT_TELESCOPE_TARGET=1050 as a /default/ const but use per-tier values in constructors.
  Per-tier truth (source: token-efficiency-design.txt §2 lines 516-526):
    Slot                    48GB-32B (30K)   12GB-7B (20K)   12GB-3B (35K)   8GB (20K)
    1. SystemPrompt         330              330             330             200
    2. Rules                500              400             500             300
    3. Telescope            1500             800             1500            600
    4. RepoMap              3000             1500            3000            1500
    5. CodeContext          10000            8000            14000           8000
    6. SsbEntries           3000             1500            3000            1500
    7. GrcHistory           2000             1000            2000            1000
    8. OutputReserve        512              512             512             512
    Total                   20842            14042           24842           13612
  Constructor changes:
    micro_7b_12gb: fixed_slot_targets [330, 400, 1050] -> [330, 400, 800]; output_reserve 750 -> 512
    micro_3b_12gb: fixed_slot_targets [330, 500, 1050] -> [330, 500, 1500]; output_reserve 750 -> 512
    large_32b_48gb: fixed_slot_targets [330, 500, 1050] -> [330, 500, 1500]; output_reserve 4000 -> 512
    micro_8gb:     fixed_slot_targets [200, 300, 1050] -> [200, 300, 600];  output_reserve 750 -> 512
  Slot weight ratios (dynamic = repo_map+code_context+ssb_entries+grc_history) MUST sum to 1.0;
  use the table's absolute numbers to derive ratios:
    micro_7b_12gb: dyn_total=12000 -> repo 0.125, code 0.667, ssb 0.125, grc 0.083 (matches existing)
    micro_3b_12gb: dyn_total=22000 -> repo 0.136, code 0.637, ssb 0.136, grc 0.091 (matches existing)
    large_32b_48gb: dyn_total=18000 -> repo 0.167, code 0.555, ssb 0.167, grc 0.111 (matches existing)
    micro_8gb:     dyn_total=12000 -> repo 0.125, code 0.667, ssb 0.125, grc 0.083 (matches existing)
  Existing tests on the OLD 750/4000/1050 values WILL fail; updating them to spec values is the
  expected outcome (NOT test rigging — the fixed_slot_targets and output_reserve are now corrected
  to match the design source-of-truth).
  Tests requiring update (verified by grep):
    test_per_tier_output_reserve_micro_750 -> rename + update to 512 across all tiers
    test_per_tier_output_reserve_large_4000 -> remove or merge into 512 universal test
    micro_8gb_profile_has_valid_weights asserts fixed_slot_targets [200, 300, FIXED_SLOT_TELESCOPE_TARGET]
      -> update to [200, 300, 600]
    compute_with_profile_uses_fixed_slot_targets asserts third slot == FIXED_SLOT_TELESCOPE_TARGET
      -> update to 800 (micro_7b_12gb's spec value)

DECISION-E — agent_files source for compute_overlap
  Source: file:agent_pool.rs:59 (AgentHandle.active_files: Vec<PathBuf>),
  file:agent_template.rs:134 (AgentTemplate.active_files: Vec<String>)
  AgentHandle.active_files exists at agent_pool.rs:59 with semantically matching meaning:
  "files relevant to the active agent". The task_runner.rs:2077 site initializes a fresh handle with
  active_files = Vec::new() per task seq.
  DECISION: keep compute_overlap and file_overlap_score as PURE functions in context_budget.rs that
  take agent_files: &[String] explicitly. Callers that have access to AgentHandle (or a BabyStep file
  list) pass it in. For B-10 minimal wiring the production caller will be in dispatcher.rs (or its
  caller) — but since SsbEntries content loading happens later in the assembly pipeline (not yet
  fully wired in task_runner), the helpers are fully tested in isolation and exposed pub(crate) for
  the eventual assembler. The ssb_filtered_by_overlap emit is wired via a thin async helper that
  callers can invoke once they have the (total, kept) counts.

DECISION-F — tier detection caching: bootstrap (preferred per spec)
  Source: spec section "Decision doc" + spec Task 1
  Tier is detected once per TaskRunner construction via:
    fn detect_model_capability_profile(model_name: &str, vram_tier: VramTier) -> ModelCapabilityProfile
  Stored on ResolvedConfig.model_capability_profile: Arc<ModelCapabilityProfile> (always-Some,
  defaulting to micro_7b_12gb if model_name doesn't match — with a tracing::warn).
  Re-detection requires a TaskRunner reconstruction — provider swap is rare and warrants the
  reset. Per-task detection deferred (would need cache invalidation + extra IPC call to /v1/models
  on every task). Documented limitation.
  VRAM tier source: not bootstrapped in B-10 (would couple to detect_vram_tier() probe which is
  separate scope). Default VramTier=Vram12Gb when model_name suggests 7B/3B; Vram48Gb when 32B;
  Vram8Gb when "8gb" in env hint. Spec marks 12GB as primary deployment tier. Future B-task
  may wire detect_vram_tier into the bootstrap.

DECISION-G — file_overlap_score threshold
  Source: spec Task 3 ("e.g. 0.1") + spec Decision doc bullet
  pub(crate) const FILE_OVERLAP_THRESHOLD: f32 = 0.10 in context_budget.rs.
  Rationale: spec suggestion. No measured data yet; conservative low value to avoid filtering
  legitimately related entries on small agent_files sets. Future tuning task may revisit with
  real data on dashboard's overlap distribution.

SCHEMA_VERSION bump: 21 -> 22 (ContextBudgetComputed + SsbFilteredByOverlap added)
  Per Tier-4 precedent (20 -> 21 for ExecutionModeChanged). B-9 SCHEMA_VERSION-not-bumped-after-
  raw_blocks-ALTER gotcha is explicitly NOT propagated here.

Test plan (TDD, written before EXECUTING):
  tests/context_budget_profile.rs (NEW, scoped to elai-orchestrator):
    a. compute_overlap_synthetic_three_entries — 3 entries / 2 agent_files / asserts ranking + filter
    b. file_overlap_score_edge_cases — empty agent / empty entry / exact-match
    c. constructor_micro_7b_12gb_matches_spec_table — fixed_slot_targets=[330,400,800], out=512
    d. constructor_micro_3b_12gb_matches_spec_table — [330,500,1500], 512
    e. constructor_large_32b_48gb_matches_spec_table — [330,500,1500], 512
    f. constructor_micro_8gb_matches_spec_table — [200,300,600], 512
    g. detect_model_capability_profile_qwen_7b_substring — abliterated 7B name -> micro_7b_12gb
    h. detect_model_capability_profile_qwen_32b -> large_32b_48gb
    i. detect_model_capability_profile_unknown_falls_back_with_warn -> micro_7b_12gb
    j. provider_model_name_default_returns_empty (default trait impl)
  tests/contracts/budget_parity.rs (NEW, scoped to elai_cc_contracts):
    k. taskrunner_repl_compute_with_profile_parity — same task input -> same allocations [0..7]

Spec deviations from B-10 plan file:
  None substantive. Spec mentioned "5 profile=None sites" — context-compiler clarified this was a
  misread (DECISION-A). Spec mentioned "compute_overlap wired into context_budget slot allocation"
  but the actual SsbEntries content-loading happens in the assembler outside compute_with_profile;
  helpers are kept pure with documented call-site contract for the eventual assembler integration.

Anti-Stale-Training audit: no new dependencies; no .unwrap() in non-test/non-bootstrap; no std::sync
HashMap in new code (use BTreeMap if any); thiserror for errors; tokio::sync::Mutex if mutation surface.
