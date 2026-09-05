# C-32: Code-Completeness Layer 2 — Skeleton-First Generation + MiniCritic + AcceptAndComplete
Objective: implement SkeletonGenerator (skeleton with BLANK_N placeholders + fill
protocol), MiniCritic between fills (tree-sitter fragment parse + T1 regex subset),
AcceptAndComplete recovery (completeness_ratio >= 0.5), structural retry sequence
(DECOMPOSE -> TEMPLATE -> FEW_SHOT -> BEST_OF_N), BridgeFileTemplate (deterministic
orchestrator-written bridges), completeness_ratio gauge in MonitorWindow, and
MiniCritic verdict rows in ReasoningDrawer.
Estimated effort: 2-3 sessions.
Pre-requisites: C-31 (FilePlan + FileLineTracker stable — SkeletonGenerator consumes
FilePlan.functions list for placeholder count); C-26 (ConstrainedDecodingConfig +
DecodingMode in providers crate — skeleton pass uses FreeText, fill pass uses Grammar
decoding mode per role defaults).
Parallel-safe with: C-21, C-22, C-23, C-24, C-25, C-26, C-27, C-28, C-29, C-30.

## Context

code-completeness-enforcement-design.report.md (2026-05-17, PARTIAL, HIGH confidence)
§MISSING: Layer 2.1 — SkeletonGenerator, BLANK_N, fill orchestrator: no evidence.
MiniCritic (MiniCriticResult, MiniCriticVerdict): no evidence. Layer 2.2 —
BridgeTemplate, LanguageTemplateRegistry: no evidence. Layer 5.1 —
AcceptAndComplete, completeness_ratio, TriageResult: no evidence. Layer 5.2 —
structural retry (decompose/template/few-shot; no instructional): no evidence.

token-efficiency-design.report.md §PARTIAL: structural retry sequence (RetryStrategy
struct, DECOMPOSE->TEMPLATE->FEW_SHOT->SPLIT->BEST_OF_N) absent. BubbleUpPolicy has
only max_retries counts; no temperature_schedule, no futility integration. Note:
C-26 adds RetryStrategy to BubbleUpPolicy — C-32 adds the skeleton-specific retry
arm (structural_retry_sequence) that fires when MiniCritic returns BLOCK on a fill.

MonitorWindow (dashboard-v2) has no completeness_ratio gauge. ReasoningDrawer has no
per-blank MiniCritic verdict rows.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\code-completeness-enforcement-design.report.md
  (§MISSING Layer 2.1, 2.2, 5.1, 5.2 for exact gap boundaries)
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\token-efficiency-design.report.md
  (§PARTIAL structural retry — RetryStrategy shape, BEST_OF_N sequence)
- .elai_cc/crates/orchestrator/src/deterministic_auditor.rs (Layer 4 completeness
  check at :779-852 — completeness_ratio definition to match)
- .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs (:169-184 BubbleUpPolicy
  — structural_retry_sequence arm must chain after C-26 RetryStrategy)
- .elai_cc/crates/plugins/src/tiered_analysis.rs (:200-476 T1 regex statics —
  MiniCritic borrows the same LazyLock<Regex> statics, no duplication)
- .elai_cc/crates/orchestrator/src/task_runner.rs (baby-step dispatch site for
  SkeletonGenerator hook point)
- .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte (gauge panel
  to extend with completeness_ratio row)
- .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (Tauri command pattern for
  new get_completeness_ratio command)
- .elai_cc/crates/orchestrator/src/ssb.rs (SSB action_type registry — all new
  action_types must be registered here)

## Tasks

1. **New file: crates/orchestrator/src/skeleton_generator.rs**
   Types:
   - BlankId: newtype u8. Display: "BLANK_{}", self.0.
   - SkeletonSpec { file_path: PathBuf, language: Language, blanks: Vec<BlankId>,
     skeleton_text: String }. Derive Debug, Clone, Serialize, Deserialize.
   - FillResult { blank_id: BlankId, body: String, mini_critic_verdict:
     MiniCriticVerdict }. Derive Debug, Clone, Serialize, Deserialize.
   - FilledBuffer { spec: SkeletonSpec, fills: Vec<FillResult>,
     completeness_ratio: f32 }. compute_completeness_ratio: fills with verdict
     != BLOCK / blanks.len() clamped [0.0, 1.0]; 0 blanks -> 1.0. No .unwrap().
   - SkeletonGeneratorConfig { max_blanks: u8=32, min_body_lines: u8=3 }.
   - generate(&FilePlan) -> Result<SkeletonSpec, SkeletonError>: function
     signatures intact; bodies -> "// BLANK_{N}". Err if count > max_blanks.
   - fill_blanks(spec, fills: Vec<String>, critic: &MiniCritic)
     -> Result<FilledBuffer, SkeletonError>: runs critic.check per fill;
     collects FillResult. Ok even if some verdicts BLOCK. No disk writes.
   - flush_to_disk(buffer: &FilledBuffer) -> Result<PathBuf, SkeletonError>:
     substitute BLANK_N with accepted body; BLOCK blanks left as
     "// BLANK_N — not filled". Single fs::write. Returns path.
   - SkeletonError (thiserror): TooManyBlanks, FlushFailed, NoFillForBlank.
   - SSB: skeleton_generated (blank_count, language) after generate();
     skeleton_flushed (path, completeness_ratio) after flush_to_disk().
   File: .elai_cc/crates/orchestrator/src/skeleton_generator.rs (ADD)

2. **New file: crates/orchestrator/src/mini_critic.rs**
   Types:
   - MiniCriticVerdict enum: PASS | WARN | BLOCK. Derive Debug, Clone,
     PartialEq, Serialize, Deserialize.
   - MiniCriticResult { blank_id: BlankId, verdict: MiniCriticVerdict,
     reason: String }. Derive Debug, Clone, Serialize, Deserialize.
   - MiniCritic struct { tree_sitter_enabled: bool }.
   - check(blank_id, body: &str, lang: Language) -> MiniCriticResult:
     Three checks; first BLOCK short-circuits:
     (a) T1 regex subset: borrow LazyLock<Regex> statics from tiered_analysis.rs
         (todo!(), unimplemented!(), pass, ..., # TODO, // TODO, FIXME).
         Match -> BLOCK, reason "lazy_pattern:<pattern>".
     (b) Line count: body.lines().count() < 3 -> BLOCK "too_short:<N>_lines".
         Exempt: no '\n' in body.trim() (single-expression body).
     (c) tree-sitter fragment parse (when enabled): wrap in minimal shell per
         language; ERROR node -> BLOCK "parse_error:<kind>"; lang not registered
         -> WARN "ts_unavailable".
     Emit mini_critic_verdict SSB row (blank_id, verdict, reason) per call.
   File: .elai_cc/crates/orchestrator/src/mini_critic.rs (ADD)

3. **New file: crates/orchestrator/src/structural_retry.rs**
   Types:
   - StructuralRetryStage enum: Decompose | Template | FewShot | BestOfN.
     Derive Debug, Clone, PartialEq, Serialize, Deserialize.
   - StructuralRetryPlan { stages: Vec<StructuralRetryStage>,
     current_stage_idx: usize, blank_id: BlankId }. current_stage() ->
     Option<&StructuralRetryStage>. No .unwrap().
   - StructuralRetryOutcome enum: Converged { fill: String } |
     AcceptAndComplete { ratio: f32 } | Exhausted.
   - StructuralRetryConfig { max_best_of_n: u8=3 }.
   - StructuralRetrySequencer { config: StructuralRetryConfig }.
   - run_sequence(plan, blank_id, context: &RetryContext) -> Outcome:
     Stages in order; each does MiniCritic check on result; PASS -> Converged:
     Decompose: split body prompt into 2-3 sub-steps, re-assemble.
     Template: FixTemplateRegistry lookup + slot fill.
     FewShot: few_shot_for_symbol() from DKG prepended to fill prompt.
     BestOfN: generate N candidates; score (compile+2, critic PASS+2,
     lines+1); pick highest. Score>0 -> Converged; else AcceptAndComplete
     if ratio >= 0.5; else Exhausted.
     Emit structural_retry_stage_attempt SSB row per stage. No instructional
     retry text ("try again" forbidden in any stage prompt).
   - RetryContext { project_root: PathBuf, language: Language,
     fn_signature: String, existing_fills: Vec<FillResult> }.
   File: .elai_cc/crates/orchestrator/src/structural_retry.rs (ADD)

4. **New file: crates/orchestrator/src/bridge_template.rs**
   Types:
   - BridgeKind enum: ModRsAggregator | InitPyBarrel | IndexTsBarrel |
     GoPackageDoc. Derive Debug, Clone, PartialEq, Serialize, Deserialize.
   - BridgeFileSpec { kind: BridgeKind, target_path: PathBuf,
     module_names: Vec<String> }. Derive Debug, Clone.
   - BridgeFileTemplate struct { seed: u64 } (ELAI_SEED env var, default 0).
   - generate(spec: &BridgeFileSpec) -> String: deterministic, no LLM call.
     Per kind: ModRsAggregator -> "pub mod {name};\n" sorted lex + header
     comment. InitPyBarrel -> "from .{name} import *\n" sorted. IndexTsBarrel
     -> "export * from './{name}';\n" sorted. GoPackageDoc -> package header.
     Each emits "// Auto-generated by ELAI BridgeFileTemplate. Do not edit."
   - write_bridge(spec: &BridgeFileSpec, output_dir: &Path)
     -> Result<PathBuf, BridgeError>: generate + single fs::write.
     BridgeError: thiserror. Variant: WriteFailed { path, source: io::Error }.
   - Emit bridge_file_generated SSB row (kind, path, module_count).
   File: .elai_cc/crates/orchestrator/src/bridge_template.rs (ADD)

5. **Wire SkeletonGenerator + StructuralRetrySequencer into task_runner.rs**
   At the baby-step LLM call site in task_runner.rs: if BabyStepSpec has a
   FilePlan attached (C-31), invoke SkeletonGenerator::generate() before the
   first LLM call to get SkeletonSpec. Collect fill bodies from LLM (one
   call per BLANK_N). Run MiniCritic::check between fills. On BLOCK:
   invoke StructuralRetrySequencer::run_sequence. On Converged: accept fill.
   On AcceptAndComplete (ratio >= 0.5 and user opt-in flag): accept partial
   buffer and flush. After all fills (or AcceptAndComplete): call
   flush_to_disk. No intermediate disk writes during skeleton/fill loop.
   File: .elai_cc/crates/orchestrator/src/task_runner.rs (MODIFY)

6. **MonitorWindow completeness_ratio gauge**
   Add a "Completeness" row to the live-metrics section of MonitorWindow.svelte.
   Read from new Tauri command get_completeness_ratio() -> f32. Render as
   horizontal gauge bar (0.0 to 1.0) with color: red < 0.5, yellow 0.5..0.9,
   green >= 0.9. Update on SSB skeleton_flushed event via existing SSB event
   subscription pattern in MonitorWindow. No new npm deps.
   File: .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte (MODIFY)

7. **ReasoningDrawer MiniCritic verdict rows**
   In the ReasoningDrawer SSB event table: add a row renderer for
   mini_critic_verdict action_type. Render columns: blank_id, verdict
   (color-coded: PASS=green, WARN=yellow, BLOCK=red), reason. Filter toggle
   "Show MiniCritic" (default on). No new npm deps.
   Locate ReasoningDrawer path via existing dashboard-v2 component layout.
   File: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (MODIFY — add
   get_completeness_ratio Tauri command)

8. **Register new SSB action_types in ssb.rs**
   Add to the action_type allowlist/match in ssb.rs:
   skeleton_generated, skeleton_flushed, mini_critic_verdict,
   structural_retry_stage_attempt, bridge_file_generated.
   File: .elai_cc/crates/orchestrator/src/ssb.rs (MODIFY)

9. **Contract tests for C-32 (TDD: write before implementation)**
   File: crates/contracts/tests/c32_skeleton_minicritc_contract.rs
   Tests (minimum 14):
   - skeleton_spec_blank_count_matches_file_plan: FilePlan with 5 functions ->
     SkeletonSpec.blanks.len() == 5.
   - skeleton_blank_ids_sequential: blanks are BLANK_1..BLANK_5.
   - skeleton_text_contains_all_placeholders: skeleton_text includes
     "// BLANK_1" through "// BLANK_5".
   - flush_single_disk_write_per_file: flush_to_disk on a FilledBuffer does
     exactly one fs::write (mock or tempfile verification).
   - completeness_ratio_all_pass: 4/4 PASS fills -> ratio == 1.0.
   - completeness_ratio_partial: 2/4 PASS, 2/4 BLOCK -> ratio == 0.5.
   - completeness_ratio_zero_blanks: FilledBuffer with 0 blanks -> ratio 1.0.
   - mini_critic_blocks_todo_pattern: body "// TODO implement" -> BLOCK.
   - mini_critic_blocks_short_body: body "x" (1 line, non-expression context)
     -> BLOCK reason "too_short:1_lines".
   - mini_critic_passes_multiline_body: 4-line body with no lazy patterns ->
     PASS verdict.
   - structural_retry_stages_order: StructuralRetryPlan stages are
     [Decompose, Template, FewShot, BestOfN] in that order.
   - bridge_mod_rs_deterministic: two calls to generate(same BridgeFileSpec)
     with ELAI_SEED=42 produce identical output.
   - bridge_mod_rs_sorted_lexicographic: module_names=["z","a","m"] ->
     output lines in order "pub mod a;", "pub mod m;", "pub mod z;".
   - bridge_file_no_llm_call: generate() returns without any network/LLM
     call (assert no HTTP fixture triggered).
   - accept_and_complete_threshold: ratio=0.5 allowed; ratio=0.49 -> Exhausted.
   File: .elai_cc/crates/contracts/tests/c32_skeleton_minicritc_contract.rs (ADD)

## WIRING_CONTRACT

produces_symbols:
  - SkeletonSpec, BlankId, FillResult, FilledBuffer, SkeletonGenerator,
    SkeletonGeneratorConfig, SkeletonError (orchestrator/src/skeleton_generator.rs)
  - MiniCritic, MiniCriticVerdict, MiniCriticResult (orchestrator/src/mini_critic.rs)
  - StructuralRetryStage, StructuralRetryPlan, StructuralRetryOutcome,
    StructuralRetrySequencer, StructuralRetryConfig, RetryContext
    (orchestrator/src/structural_retry.rs)
  - BridgeKind, BridgeFileSpec, BridgeFileTemplate, BridgeError
    (orchestrator/src/bridge_template.rs)
  - SSB action_types: skeleton_generated, skeleton_flushed, mini_critic_verdict,
    structural_retry_stage_attempt, bridge_file_generated
  - Tauri command: get_completeness_ratio -> f32
  - MonitorWindow completeness_ratio gauge
  - ReasoningDrawer mini_critic_verdict row renderer

consumes_symbols:
  - FilePlan.functions (C-31 — drives blank count in SkeletonGenerator::generate)
  - ConstrainedDecodingConfig / DecodingMode (C-26 — fill pass uses Grammar mode)
  - T1 LazyLock<Regex> statics (tiered_analysis.rs — MiniCritic reuses, no copy)
  - FixTemplateRegistry (knowledge/src/fix_templates.rs — Template stage in retry)
  - few_shot_for_symbol() (knowledge/src/fix_templates.rs — FewShot stage)
  - BubbleUpPolicy.retry_strategy (C-26 — structural_retry fires on BLOCK chain)
  - tree_sitter crate (already in workspace — MiniCritic fragment parse)
  - SSB emit infrastructure (orchestrator/src/ssb.rs)

reachable_from:
  - baby-step LLM dispatch in task_runner.rs (FilePlan present -> skeleton path)
  - BridgeFileTemplate called by orchestrator when a new module is added to
    FilePlan (mod.rs aggregator generation)
  - dashboard MonitorWindow skeleton_flushed SSB subscription
  - dashboard ReasoningDrawer mini_critic_verdict SSB row filter

stub_ban:
  - flush_to_disk MUST do exactly one fs::write; no incremental writes during
    skeleton/fill loop.
  - BridgeFileTemplate::generate MUST NOT call any LLM or network endpoint;
    deterministic text assembly only.
  - MiniCritic MUST reuse T1 LazyLock<Regex> statics from tiered_analysis.rs;
    MUST NOT duplicate regex literals.
  - AcceptAndComplete threshold MUST be completeness_ratio >= 0.5; no lower.
  - StructuralRetrySequencer MUST NOT emit "try again" instructional text in
    any retry prompt; structural transforms only.

frontend_pair:
  - MonitorWindow.svelte gauge + get_completeness_ratio Tauri command
  - ReasoningDrawer mini_critic_verdict row renderer (no new Tauri command needed;
    reads from existing SSB event stream)

observable_effect:
  trigger: elai /task --mode cloud "implement a 300-LOC Rust web server with
    4 routes + middleware" (Sonnet 4.6, ELAI_SEED=42)
  expect_ssb: skeleton_generated first; >= 4 mini_critic_verdict rows;
    skeleton_flushed completeness_ratio >= 0.9; zero BLANK_N in file;
    cargo check passes.
  adversarial: TODO-injected fill -> BLOCK -> structural_retry_stage_attempt
    rows; file converges or AcceptAndComplete fires (ratio >= 0.5).
  bridge_test: mod.rs from BridgeFileTemplate (not LLM); ELAI_SEED=42 two
    runs byte-identical.

## DETERMINISTIC_CHECK

```bash
# 1. Core types defined
rg "pub struct SkeletonSpec|pub struct FilledBuffer|pub enum SkeletonError" .elai_cc/crates/orchestrator/src/skeleton_generator.rs
rg "pub enum MiniCriticVerdict|pub struct MiniCritic" .elai_cc/crates/orchestrator/src/mini_critic.rs
rg "pub enum StructuralRetryStage|pub struct StructuralRetryPlan" .elai_cc/crates/orchestrator/src/structural_retry.rs
rg "pub enum BridgeKind|pub struct BridgeFileTemplate" .elai_cc/crates/orchestrator/src/bridge_template.rs

# 2. Single disk write: flush_to_disk has exactly one fs::write
rg "fs::write" .elai_cc/crates/orchestrator/src/skeleton_generator.rs
# Expected: exactly 1 match in flush_to_disk; none elsewhere in file

# 3. MiniCritic no duplicated regex literals
rg "Regex::new" .elai_cc/crates/orchestrator/src/mini_critic.rs
# Expected: zero matches (borrows tiered_analysis statics)

# 4. BridgeFileTemplate no LLM/network call
rg "sidecar|anthropic|reqwest|LlmProvider" .elai_cc/crates/orchestrator/src/bridge_template.rs
# Expected: zero matches

# 5. SSB action_types registered
rg "skeleton_generated|mini_critic_verdict|structural_retry_stage_attempt|bridge_file_generated" .elai_cc/crates/orchestrator/src/ssb.rs

# 6. Contract tests pass
cargo test -p elai-orchestrator --test c32_skeleton_minicritc_contract 2>&1 | grep "test result"
# Expected: test result: ok. 14 passed
```

## ACCEPTANCE

- [ ] generate() -> SkeletonSpec with BLANK_N per function; no LLM call
- [ ] fill_blanks() collects in memory; zero disk writes during fill loop
- [ ] flush_to_disk() exactly one fs::write; BLOCK blanks left as comment
- [ ] MiniCritic checks order: T1 regex -> line count (>=3) -> tree-sitter
- [ ] MiniCritic reuses LazyLock<Regex> from tiered_analysis.rs; zero duplicates
- [ ] MiniCritic emits mini_critic_verdict SSB row per call
- [ ] Structural retry order: Decompose -> Template -> FewShot -> BestOfN;
      no instructional retry text in any stage prompt
- [ ] AcceptAndComplete: ratio >= 0.5 + user opt-in -> accept partial;
      ratio < 0.5 -> Exhausted
- [ ] BridgeFileTemplate::generate is LLM-free; ModRsAggregator sorted lex;
      byte-identical across two runs with ELAI_SEED=42
- [ ] MonitorWindow completeness_ratio gauge updated on skeleton_flushed SSB
- [ ] ReasoningDrawer mini_critic_verdict rows + "Show MiniCritic" toggle
- [ ] All 14 contract tests pass
- [ ] Cloud smoke: skeleton_generated first; >= 4 mini_critic_verdict rows;
      completeness_ratio >= 0.9; zero BLANK_N in file; cargo check passes
- [ ] Adversarial: TODO fill -> BLOCK -> structural_retry rows; convergence
      or AcceptAndComplete
- [ ] No .unwrap() in production paths; cargo check --workspace clean

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/skeleton_generator.rs
- ADD: .elai_cc/crates/orchestrator/src/mini_critic.rs
- ADD: .elai_cc/crates/orchestrator/src/structural_retry.rs
- ADD: .elai_cc/crates/orchestrator/src/bridge_template.rs
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (skeleton hook in
  baby-step dispatch)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (5 new action_types)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte
  (completeness_ratio gauge)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
  (get_completeness_ratio Tauri command)
- LOCATE AND MODIFY: crates/dashboard-v2/src/lib/windows/ReasoningDrawer.svelte
  (mini_critic_verdict row renderer + Show MiniCritic toggle)
- ADD: .elai_cc/crates/contracts/tests/c32_skeleton_minicritc_contract.rs
- ADD: workflow/reasoning/C-32-decisions.md
