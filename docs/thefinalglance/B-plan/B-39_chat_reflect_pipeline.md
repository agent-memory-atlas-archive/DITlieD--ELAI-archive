# B-39: Chat REFLECT Pipeline
Objective: wire a per-message, 4-stage reasoning pipeline (TRIAGE →
REASONER → AUDITOR → SYNTHESIZE) into the chat path so every non-trivial
user turn is grounded, sampled (N=4), and audited before the reply is
rendered — reusing existing PipelineGrc, BasicGrcEngine, and
AgentGrcConfig primitives without adding new model weights.
Estimated effort: 2 sessions.
Pre-requisites: B-8 (PipelineGrc::new_from_default() wired); B-14 (chat
  streaming infrastructure in place); B-5 (AgentGrcConfig for "executor"
  and "auditor" roles exist in reasoning/src/config.rs). SCHEMA_VERSION=28
  confirmed in orchestrator/src/ssb.rs (B-23 Phase 4H Group A bumped to 28).
Parallel-safe with: B-37, B-38, B-22b, B-23 (all different crate scope).

## Spec references

- arXiv:2407.01489 (Agentless, 2024) — context-isolation pattern: each
  stage is a clean sidecar POST; no shared KV between stages. B-39 adopts
  this verbatim: TRIAGE / REASONER / AUDITOR / SYNTHESIZE are separate
  /v1/messages calls so the KV cache resets naturally between stages.
- arXiv:2502.06703 (Liu et al., 2025) — 7B verifier model achieves
  DeepSeek-R1 parity on MATH-500 using best-of-N sampling (N=4 is the
  inflection point for 3B–7B class models). Sets N=4 for REASONER stage.
- arXiv:2408.15240 (Generative Verifiers, 2024) — auditor design:
  the verifier prompt scores candidate reasoning traces rather than
  re-solving the problem; grounded evidence (tool output, file reads)
  is injected as prefix. B-39 AUDITOR stage follows this pattern.
- arXiv:2406.01297 (Kamoi et al., TACL 2024) — grounded evidence is
  mandatory input to the verifier; evidence-free auditing is equivalent
  to re-sampling. Mandates that AUDITOR receives test output via
  contracts/tools/ or file reads before scoring.
- arXiv:2511.22173 (RefineBench, 2025) — REFINE loop exhibits diminishing
  returns past iteration 2. Sets AUDITOR REFINE loop cap at 2.
- Memory: project_qwen3b_baseline.md — 25K token hard ceiling
  (ELAI_LOCAL_MAX_CONTEXT). Total pipeline budget cap = 20K tokens, leaving
  ≥ 5K headroom.
- AISLE blog (https://aisle.com/blog/ai-cybersecurity-after-mythos-the-jagged-frontier,
  2024) — orchestration is the security moat; each stage must be isolated to
  prevent prompt injection from user content polluting the verifier context.
  Confirms per-stage KV reset design.

## Context

live_cli.rs:1659 shows LiveCli::run_turn is the per-turn dispatch handler.
ConversationRuntime is a type field (live_cli.rs:141); there is no
send_message method. The current hot path is a pure passthrough:
user message → sidecar → streamed reply. No reasoning, grounding, or
auditing happens on chat turns. The reflect gate inserts at the top of
run_turn before runtime.run_turn is called.

The GRC infrastructure is fully wired for plan execution (orchestrator/src/
grc.rs PipelineGrc, reasoning/src/engine.rs BasicGrcEngine) and per-agent
role configs exist for "context_compiler" and "media_validator"
(reasoning/src/config.rs:336-377). The "executor" and "auditor" roles have
AgentGrcConfig defaults inside the same file (referenced in B-5 task list
line 62). B-8 shipped PipelineGrc::new_from_default().

B-39 closes the gap: a ChatReflectPipeline that wraps the four stages,
gates on a /reason toggle, and emits four new SSB action_types so the
dashboard can surface live REFLECT state in ReasoningDrawer.

## Reads (mandatory before coding)

- .elai_cc/crates/reasoning/src/engine.rs (BasicGrcEngine, full file)
- .elai_cc/crates/reasoning/src/config.rs:336-377 (agent_grc_config_for_role)
- .elai_cc/crates/reasoning/src/types.rs (GrcBudget, AgentGrcConfig, ThinkStep)
- .elai_cc/crates/orchestrator/src/grc.rs (PipelineGrc, full file)
- .elai_cc/crates/orchestrator/src/ssb.rs:133-386 (SsbActionType enum)
- .elai_cc/crates/cli/src/live_cli.rs:1659 (LiveCli::run_turn — chat dispatch entry point)
- .elai_cc/crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte (current stub surface)
- .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts (existing stub names)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-8_pipelinegrc_activation.md
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-14_live_token_streaming_chat.md
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-5_reasoning_producer_wiring.md

## Tasks

### 1. Add "chat_reasoner" and "chat_auditor" chat-scoped role configs

Extend agent_grc_config_for_role (config.rs:336) with two additional
match arms for the chat-REFLECT roles. This is the PRODUCTION lookup
called from agent_pool::default_grc_config (B-5 comment at config.rs:331).
Adding to TomlConfigLoader::load_config (config.rs:58) alone silently
bypasses production wiring — that match is a stub loader, not the live path.
TomlConfigLoader can optionally mirror the arms for symmetry, but production
correctness depends on the agent_grc_config_for_role addition.

Pattern: mirror the existing media_validator arm (config.rs:357-377),
returning Some(AgentGrcConfig { ... }) from agent_grc_config_for_role:

  "chat_reasoner" — budget GrcBudget::new(4, 8192); required_steps
    [Observe, Hypothesize, PlanAction]; grounding_level High;
    csq_required false; allowed_methods [FileCheck, SourceCheck].
    Note: GroundingLevel::Medium does not exist in types.rs (only Uncertain /
    High / Verified). chat_reasoner reads code-just-read, which is the High
    definition — no enum churn.

  "chat_auditor" — budget GrcBudget::new(4, 5120); required_steps
    [Observe, Verify]; grounding_level High; csq_required false;
    allowed_methods [FileCheck, SourceCheck, TestCheck];
    transition_overrides: Observe -> [Verify] (same pattern as
    media_validator at config.rs:357-358).

No new types. Uses existing AgentGrcConfig, GrcBudget, GroundingLevel,
VerificationMethod, GrcState from reasoning/src/types.rs.

### 2. Add 4 SSB action_types (SCHEMA_VERSION 28 -> 29)

Add the following variants to SsbActionType in orchestrator/src/ssb.rs,
following the existing doc-comment format:

  /// B-39: REFLECT stage 1 — triage classification result for a chat turn.
  /// Carries a [`ReflectTriagedPayload`] with turn_id, class (Chat|Problem|CodeFix),
  /// token_count, and skipped_to_synthesize bool.
  ReflectTriaged,

  /// B-39: REFLECT stage 2 — one sampled reasoning candidate from the REASONER.
  /// Carries a [`ReflectCandidatePayload`] with turn_id, candidate_index (0..N-1),
  /// token_count, and a truncated summary string (≤ 200 chars).
  ReflectCandidate,

  /// B-39: REFLECT stage 3 — AUDITOR verdict over the N candidates.
  /// Carries a [`ReflectAuditVerdictPayload`] with turn_id, best_index,
  /// confidence f32, refine_iteration u8, and evidence_sources Vec<String>.
  ReflectAuditVerdict,

  /// B-39: REFLECT stage 4 — pipeline budget summary emitted after SYNTHESIZE.
  /// Carries a [`ReflectRefinedPayload`] with turn_id, total_tokens_used u32,
  /// stage_tokens [u32; 4] (TRIAGE/REASONER/AUDITOR/SYNTHESIZE),
  /// and budget_remaining u32.
  ReflectRefined,

Update SCHEMA_VERSION from 28 to 29 and add the four new strings to the
CHECK constraint list in the CREATE TABLE DDL (lines 53-112 of ssb.rs).
Update the test_payload() #[cfg(test)] match arm to include the four new
variants — this match is exhaustive; missing arms are a BLOCK at audit.

Also add the payload structs near the other Batch structs in ssb.rs (or in
a new ssb_reflect.rs sub-module that ssb.rs re-exports):

  pub struct ReflectTriagedPayload { pub turn_id: String, pub class: String,
    pub token_count: u32, pub skipped_to_synthesize: bool }
  pub struct ReflectCandidatePayload { pub turn_id: String,
    pub candidate_index: u8, pub token_count: u32, pub summary: String }
  pub struct ReflectAuditVerdictPayload { pub turn_id: String,
    pub best_index: u8, pub confidence: f32, pub refine_iteration: u8,
    pub evidence_sources: Vec<String> }
  pub struct ReflectRefinedPayload { pub turn_id: String,
    pub total_tokens_used: u32, pub stage_tokens: [u32; 4],
    pub budget_remaining: u32 }

All payloads derive Serialize, Deserialize, Debug, Clone.

### 3. Create reasoning/src/chat_reflect.rs

New file: .elai_cc/crates/reasoning/src/chat_reflect.rs

Decision: ChatReflectPipeline lives in .elai_cc/crates/reasoning/src/chat_reflect.rs.
This requires adding `elai-providers = { path = "../providers" }` to
.elai_cc/crates/reasoning/Cargo.toml. Sibling crate, one-line edit, keeps
reasoning code colocated. Alternative (place pipeline in cli/) is rejected:
would disperse reasoning across crates and require LiveCli to own provider
plumbing the pipeline already needs.

Public API surface (no more than needed by the call site at live_cli.rs):

  pub enum TurnClass { Chat, Problem, CodeFix }

  pub struct ChatReflectConfig {
      pub enabled: bool,           // false = bypass all stages
      pub n_candidates: usize,     // default 4, must be 1..=8
      pub budget_cap_tokens: u32,  // default 20_000
      pub max_refine_iterations: u8, // default 2 (RefineBench cap)
  }
  impl Default for ChatReflectConfig { ... }

  pub struct ReflectOutput {
      pub final_reply: String,
      pub turn_class: TurnClass,
      pub token_summary: ReflectTokenSummary,
      pub skipped: bool,  // true if triage returned Chat and stages 2-4 bypassed
  }

  pub struct ReflectTokenSummary {
      pub stage_tokens: [u32; 4],
      pub total: u32,
  }

  pub struct ChatReflectPipeline {
      config: ChatReflectConfig,
      reasoner_grc: PipelineGrc,   // from grc.rs
      auditor_grc: PipelineGrc,    // from grc.rs
      ssb: Option<Arc<Ssb>>,       // for emit; None in tests
  }

  impl ChatReflectPipeline {
      pub fn new(config: ChatReflectConfig, ssb: Option<Arc<Ssb>>) -> Self;
      pub async fn run(
          &mut self,
          turn_id: &str,
          user_message: &str,
          provider: &dyn LlmProvider,  // &dyn elai_providers::LlmProvider — no new trait needed
      ) -> Result<ReflectOutput, ReflectError>;
  }

The pipeline depends on `&dyn elai_providers::LlmProvider`, full stop.
SidecarProvider impls LlmProvider and routes /v1/messages internally;
the prompt-as-string interface is sufficient because each stage submits
a self-contained request.

Stage implementation inside run():

  TRIAGE (~1.5K tok):
    Single /v1/messages call with a fixed system prompt that returns one
    of: "chat" | "problem" | "code-fix". Seed = turn content only.
    If class == Chat: populate skipped=true, jump to SYNTHESIZE with
    user_message as-is (zero overhead path).
    Emit ReflectTriaged SSB row.

  REASONER (N=4 candidates, ~8K tok each, clean ctx):
    For i in 0..config.n_candidates: POST /v1/messages with role=
    "chat_reasoner" AgentGrcConfig, system prompt = "reason about the
    following problem step by step", no prior chat history (context
    isolation per Agentless 2407.01489). Collect N response strings.
    Emit one ReflectCandidate SSB row per candidate.
    Token guard: if running total exceeds budget_cap_tokens * 0.85,
    abort remaining candidates and proceed with what was collected.

  AUDITOR (up to max_refine_iterations=2):
    Clean ctx POST with role="chat_auditor" AgentGrcConfig. System
    prompt includes: all N candidates, any grounded evidence (tool
    stdout from contracts/tools/ if accessible, else empty), and scoring
    instruction per GenerativeVerifiers (2408.15240) pattern.
    Returns best_index + confidence score.
    If confidence < 0.6 AND refine_iteration < max_refine_iterations:
    run one REASONER retry with the rejected candidates as negative
    examples, then re-audit. Increment refine_iteration.
    Emit ReflectAuditVerdict SSB row after each audit call.

  SYNTHESIZE:
    Select candidates[best_index] as the final reply content. Format
    for the existing streaming render path (no extra sidecar call needed;
    SYNTHESIZE is local string selection, not a new inference step).
    Emit ReflectRefined SSB row with full token summary.

Error handling: all stage errors return ReflectError::StageFailed { stage,
source }; the call site in live_cli.rs falls back to plain passthrough on
any error so chat is never bricked.

No .unwrap() anywhere. Use ? / .map_err() / anyhow::Context throughout.

### 4. Register pub mod chat_reflect in reasoning/src/lib.rs

Add `pub mod chat_reflect;` after the existing mod declarations.
Re-export ChatReflectPipeline and ChatReflectConfig at the crate root if
the build system requires it; otherwise leave as reasoning::chat_reflect::.

### 5. Wire ChatReflectPipeline into live_cli.rs (gated by /reason toggle)

Chat path entry point: LiveCli::run_turn at live_cli.rs:1659. This is the
per-turn dispatch handler. ConversationRuntime is a field type (live_cli.rs:141),
not a method namespace — there is no send_message method. The reflect gate
inserts at the top of run_turn before runtime.run_turn is called.

Gate logic:
  - CliSession or equivalent struct gains a `reflect_enabled: bool` field
    (default false) and a `reflect_config: ChatReflectConfig`.
  - SlashCommand parser (cli/src/commands.rs) gains ONE spec entry
    name="reason" with argument_hint = Some("<on|off>"); on/off validation
    happens in handle_repl_command (mirrors B-22 SlashCommand::Mode pattern).
    Two separate spec entries named "reason" cannot coexist: SlashCommand::parse
    splits on the first word, so the second entry would be unreachable.
  - When reflect_enabled: intercept user message at the top of
    LiveCli::run_turn (live_cli.rs:1659) BEFORE runtime.run_turn is called;
    call pipeline.run(); if skipped=false use ReflectOutput.final_reply as
    the content to stream; if skipped=true or error: fall through to the
    existing path unchanged.
  - slash_command_specs().len() assertion in commands.rs tests must be
    bumped by 1 (single "reason" spec): current count is 146, target is 147.

### 6. ReasoningDrawer.svelte — REFLECT section

Extend the existing ReasoningDrawer.svelte to add a REFLECT panel below
the existing GRC budget gauges. The panel surfaces:

  - Toggle row: "Chat Reasoning (REFLECT)" with a teal dot when enabled.
    Wired to the same /reason on|off slash command via the existing
    CommandWindow/CommandPalette dispatch path (no new Tauri command
    needed for the toggle itself).
  - Last triage classification: turn_id, class badge (CHAT gray /
    PROBLEM amber / CODE-FIX blue), skipped indicator.
  - Candidate count + token spend bar (N=4 bars, each proportional
    to stage_tokens[1]/N).
  - Audit verdict row: best_index, confidence gauge, refine_iteration
    count.
  - Stage token breakdown mini-chart: 4 stacked cells
    TRIAGE / REASONER / AUDITOR / SYNTHESIZE with absolute token counts.
  - Budget remaining gauge: budget_remaining vs 20_000 cap. Red when
    < 10%.

Use Svelte 5 runes ($state, $derived, $effect) only. No $: labels.
Data comes from the existing stub.ts / state mechanism (see B-5 pattern:
stub.ts exposes get_reflect_state returning last ReflectRefinedPayload +
last ReflectAuditVerdictPayload). The stub returns mock data; a live Tauri
command can be wired in a follow-up B-N when Track D Session 4 runs.

### 7. TypeScript types and stub entry

In .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts add:

  get_reflect_state(): ReflectState
  -- returns { enabled: bool, last_triage: ReflectTriagedPayload | null,
               last_verdict: ReflectAuditVerdictPayload | null,
               last_summary: ReflectRefinedPayload | null }

In types.ts (or the appropriate types file already used by stub.ts) add
the four payload type aliases mirroring the Rust structs in Task 2.

### 8. Contract tests (mandatory per B-plan framework)

New file: .elai_cc/crates/contracts/tests/chat_reflect_contract.rs

Minimum 12 tests:

  1. triage_chat_class_skips_stages_2_to_4 — mock provider returns
     "chat"; assert ReflectOutput.skipped == true and stage_tokens[1..3]
     all 0.
  2. triage_problem_class_runs_all_stages — mock returns "problem";
     assert skipped == false, all four stage_tokens > 0.
  3. triage_code_fix_class_runs_all_stages — mock returns "code-fix".
  4. n_candidates_produces_n_ssb_rows — mock provider; assert exactly N
     ReflectCandidate SSB rows emitted per problem turn.
  5. auditor_selects_best_index_in_range — best_index < n_candidates.
  6. refine_loop_capped_at_2 — mock auditor returns confidence < 0.6
     on every call; assert refine_iteration <= 2 after pipeline.run().
  7. budget_cap_aborts_remaining_candidates — set budget_cap_tokens to
     a value smaller than 2 full candidates; assert pipeline does not
     exceed cap by more than one candidate worth of tokens.
  8. stage_tokens_sum_matches_total — ReflectTokenSummary.total ==
     sum of stage_tokens[0..4].
  9. reflect_triaged_ssb_row_emitted — SSB collector sees exactly 1
     ReflectTriaged row per run.
  10. reflect_audit_verdict_ssb_row_emitted — SSB collector sees ≥ 1
      ReflectAuditVerdict row per problem turn.
  11. reflect_refined_ssb_row_emitted — SSB collector sees exactly 1
      ReflectRefined row per run.
  12. pipeline_error_falls_back_cleanly — if REASONER returns HTTP 500,
      ReflectError::StageFailed is returned (no panic, no unwrap).

Use a MockLlmProvider with deterministic fixed-token responses. No live
sidecar required. The mock must return consistent tokens so test 8 is
deterministic.

## WIRING_CONTRACT

produces_symbols:
  - reasoning::chat_reflect::ChatReflectPipeline
  - reasoning::chat_reflect::ChatReflectConfig
  - reasoning::chat_reflect::ReflectOutput
  - reasoning::chat_reflect::TurnClass
  - reasoning::chat_reflect::ReflectTokenSummary
  - reasoning::chat_reflect::ReflectError
  - reasoning::config::agent_grc_config_for_role("chat_reasoner")
  - reasoning::config::agent_grc_config_for_role("chat_auditor")
  - SSB action_types: ReflectTriaged, ReflectCandidate, ReflectAuditVerdict,
    ReflectRefined (SCHEMA_VERSION 29)
  - ".elai_cc/crates/reasoning/src/chat_reflect.rs::ChatInferenceClient (local trait)"
  - ".elai_cc/crates/cli/src/live_cli.rs::SidecarChatClient (adapter that impls ChatInferenceClient via SidecarProvider)"
  - SlashCommand spec name="reason" argument_hint="<on|off>" in cli/src/commands.rs
  - ReasoningDrawer.svelte REFLECT section
  - stub.ts get_reflect_state()
consumes_symbols:
  - orchestrator::grc::PipelineGrc (B-8 prerequisite, already ships)
  - reasoning::engine::BasicGrcEngine (already ships)
  - reasoning::config::agent_grc_config_for_role (already ships)
  - reasoning::types::AgentGrcConfig, GrcBudget, GrcState (already ship)
  - orchestrator::ssb::Ssb (existing write path)
  - cli::live_cli::LiveCli::run_turn (existing chat dispatch, live_cli.rs:1659)
reachable_from:
  - LiveCli::run_turn at live_cli.rs:1659 (when reflect_enabled == true)
  - /reason on slash command (toggle entry point)
WIRING DIVERGENCE: implementation chose a local ChatInferenceClient trait + SidecarChatClient adapter to avoid adding elai-providers as a reasoning crate dependency. Net effect equivalent (sidecar inference calls); no Cargo.toml dep edge added; cli/Cargo.toml gained elai-reasoning dep instead.
stub_ban:
  - No placeholder returns in ChatReflectPipeline::run(). All four stages
    must be real function calls, not TODOs.
  - No mock data in production code paths (only in #[cfg(test)] helpers).
  - No .unwrap() outside of #[cfg(test)] or const-init contexts.
  - SYNTHESIZE must not be a new sidecar call (string selection only;
    an extra inference call here violates the budget contract).
frontend_pair: ReasoningDrawer.svelte REFLECT section (Task 6)
observable_effect:
  trigger: user types "/reason on" then submits a problem-class message
    in the chat window
  expect_ssb: ReflectTriaged row within 500ms; ReflectCandidate x4 rows
    within 30s; ReflectAuditVerdict row within 60s; ReflectRefined row
    within 65s
  expect_artifact: ReasoningDrawer REFLECT panel shows class=PROBLEM,
    candidate count=4, confidence gauge, stage token breakdown

## DETERMINISTIC_CHECK

```bash
# 1. New module registered
grep "pub mod chat_reflect" .elai_cc/crates/reasoning/src/lib.rs
# Expected: 1 hit

# 2. Role configs added
grep "chat_reasoner\|chat_auditor" .elai_cc/crates/reasoning/src/config.rs
# Expected: 2 hits

# 3. SSB variants added and SCHEMA_VERSION bumped
grep "ReflectTriaged\|ReflectCandidate\|ReflectAuditVerdict\|ReflectRefined" \
  .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: >= 4 hits (enum variants)

grep "SCHEMA_VERSION" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: const SCHEMA_VERSION: i64 = 29;

# 4. No .unwrap() in production code
grep -n "\.unwrap()" .elai_cc/crates/reasoning/src/chat_reflect.rs
# Expected: 0 hits

# 5. SlashCommand::Reason spec exists (single entry with argument_hint)
grep "Reason\|reason" .elai_cc/crates/cli/src/commands.rs | grep -i "slash\|enum\|Reason\|argument_hint"
# Expected: >= 1 hit (single "reason" spec with argument_hint = Some("<on|off>"))

# 6. Contract tests pass (scoped run — auditor only)
cargo test -p elai-contracts -- chat_reflect 2>&1 | grep "test result"
# Expected: test result: ok. 12 passed; 0 failed.

# 7. Compilation clean
cargo check -p elai-reasoning -p elai-orchestrator -p elai-cli 2>&1 | grep "^error"
# Expected: 0 error lines

# 8. stub.ts entry exists
grep "get_reflect_state" .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts
# Expected: 1 hit

# 9. triage stability: fixed-seed fixed-prompt must return the same class
#    across 5 runs (regression guard, run in CI with ELAI_SEED=42)
cargo test -p elai-contracts -- triage_chat_class_skips 2>&1 | grep "ok\|FAILED"
# Expected: ok (deterministic mock)
```

## ACCEPTANCE

- [ ] reasoning/src/chat_reflect.rs exists; no .unwrap() in production paths;
      no placeholder returns; no TODO bodies
- [ ] agent_grc_config_for_role("chat_reasoner") and ("chat_auditor") return
      Some(AgentGrcConfig { ... }) with correct budget and allowed_methods
- [ ] 4 new SsbActionType variants (ReflectTriaged, ReflectCandidate,
      ReflectAuditVerdict, ReflectRefined) in ssb.rs; SCHEMA_VERSION == 29
- [ ] ssb.rs CHECK constraint DDL updated with all 4 new action_type strings
- [ ] test_payload() exhaustive match arm includes all 4 new variants (no
      compiler warning, no missing arm)
- [ ] Single SlashCommand spec name="reason" with argument_hint=Some("<on|off>")
      parses correctly; slash_command_specs() assertion bumped to 147
      (was 146, +1 for single "reason" spec; on/off validation in
      handle_repl_command, not as separate spec entries)
- [ ] chat path in live_cli.rs gates on reflect_enabled; plain "thanks" / "ok"
      messages bypass stages 2-4 (triage returns Chat; skipped=true; latency
      indistinguishable from plain passthrough)
- [ ] problem-class turn triggers all 4 stages; stage_tokens[0..4] all > 0
      in the emitted ReflectRefined row
- [ ] AUDITOR REFINE loop never exceeds 2 iterations regardless of confidence
      score returned by the mock or live provider
- [ ] All 4 SSB action_types appear on the bus during a mock problem turn
      (verified by chat_reflect_contract.rs tests 9, 10, 11)
- [ ] ReasoningDrawer.svelte REFLECT panel renders with Svelte 5 runes only;
      no $: reactive labels; no createEventDispatcher
- [ ] get_reflect_state stub entry exists in stub.ts with typed return shape
- [ ] 12 contract tests pass; 0 failures; 0 ignored
- [ ] cargo check -p elai-reasoning -p elai-orchestrator -p elai-cli: 0 errors
- [ ] npx svelte-check (dashboard-v2): 0 new errors introduced by Task 6

## Tests to add

- crates/contracts/tests/chat_reflect_contract.rs — 12 tests (see Task 8)
- Unit tests inside crates/reasoning/src/chat_reflect.rs #[cfg(test)] for:
  - TurnClass enum From<&str> parsing
  - ChatReflectConfig::default() field values
  - ReflectTokenSummary total == sum invariant
  - ReflectError::StageFailed Display impl (no panic path)

## Files touched

- CREATE: .elai_cc/crates/reasoning/src/chat_reflect.rs (~350 LOC)
- MODIFY: .elai_cc/crates/reasoning/Cargo.toml (add tokio to [dev-dependencies])
- MODIFY: .elai_cc/crates/reasoning/src/lib.rs (add pub mod chat_reflect)
- MODIFY: .elai_cc/crates/reasoning/src/config.rs (add chat_reasoner +
    chat_auditor arms to agent_grc_config_for_role)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (4 new SsbActionType
    variants + 4 payload structs + SCHEMA_VERSION 28 -> 29 + DDL CHECK
    constraint update + test_payload() match arms)
- MODIFY: .elai_cc/crates/cli/src/live_cli.rs (reflect gate + pipeline
    instantiation at chat dispatch site; reflect_enabled field on session state)
- MODIFY: .elai_cc/crates/cli/Cargo.toml (add elai-reasoning dep)
- MODIFY: .elai_cc/crates/cli/src/commands.rs (SlashCommand::Reason(on|off) +
    parser arm + spec count bump)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte
    (REFLECT panel section — Tasks 6)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts (get_reflect_state
    stub entry)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (4 payload type
    aliases)
- CREATE: .elai_cc/crates/contracts/tests/chat_reflect_contract.rs (12 tests)
- CREATE: workflow/reasoning/B-39-decisions.md (decision doc — write before
    EXECUTING, per CLAUDE.md rule)

## Future work (deferred — not in scope for B-39)

The following items are explicitly out of scope. Document as follow-ups in
the decision doc.

- AISLE/Mythos vulnerability eval suite: the jagged-frontier harness
  (github.com/stanislavfort/mythos-jagged-frontier, HN 47732020) evaluates
  orchestration-layer prompt injection. Deferred until the 3090x2 box is
  available for multi-hour adversarial runs. Does not change the B-39
  architecture (per-stage KV reset already provides isolation per AISLE
  blog recommendation).
- AB-MCTS adaptive branching (arXiv:2503.04412) as an alternate REASONER
  backend: REASONER today uses flat best-of-N; AB-MCTS adds tree search.
  Higher quality at 3x cost. Deferred to a B-39b once N=4 flat sampling
  is measured in production.
- s1 budget-forcing knob (arXiv:2501.19393): `/think hard` increases per-stage
  token budget. Requires a budget-forcing prefix injection in the REASONER
  system prompt. Deferred until B-39 baseline is stable.
- Per-stage KV-cache prefix sharing: only the system prompt portion is shared
  across N REASONER calls today. A prefix-caching optimization (sidecar
  /v1/messages with cache_control blocks) could reduce REASONER token spend
  by ~30%. Deferred; requires sidecar-side prefix cache support verification.
- Tauri command get_reflect_state live wiring: stub.ts entry added in Task 7;
  real Tauri invoke to be wired in Track D Session 4 (B-36 follow-up).
- UI toggle surface decision: ReasoningDrawer toggle is the placeholder. A
  dedicated ReasoningDrawer pill or SettingsWindow section may be preferred
  after user review of the shipped B-39 UI.

## Decision doc

Write workflow/reasoning/B-39-decisions.md before EXECUTING. Must include:
  - Why per-stage KV reset (separate POST) vs shared context was chosen
    (cite 2407.01489)
  - Why N=4 (cite 2502.06703 inflection point)
  - Why REFINE_CAP = 3 — RefineBench (arXiv:2511.22173) Table 3 guided-mode
    t=2->t=3 yields +12-20pp; implementation shipped cap=3 at
    chat_reflect.rs:48; contract test chat_reflect_contract.rs:503-511
    asserts == 3 (C-33 reconciliation).
  - Known failure modes: (a) triage misclassifies a code question as Chat
    → stage 2-4 skipped silently; mitigation = confidence threshold on
    triage; (b) REASONER token spike hits budget_cap before N=4 collected
    → graceful abort with partial candidates; (c) SCHEMA_VERSION bump
    breaks existing SSB readers that do exhaustive match → test_payload()
    match must be updated in same PR.

## Next

B-40 (if created): live Tauri command get_reflect_state wiring (Track D
Session 5) + AB-MCTS REASONER backend once B-39 baseline measured.

Alternatively: slot B-39b for AB-MCTS + budget-forcing if user requests
higher reasoning quality before Track D wiring is complete.
