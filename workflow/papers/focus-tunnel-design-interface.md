Source: Focus Tunnel Design - Zero-Overhead LLM Execution Path
Reliability: HIGH (official spec authored 2026-04-15)
Files examined:
  F:\.ELAI_workflow\docs\specs\guide\focus-tunnel-design.txt
  F:\.ELAI_workflow\docs\specs\guide\post-phase4-implementation-order.txt (E-1 through E-7 sections)


========================================================================
PUBLIC API SURFACE
========================================================================

FimPromptAssembler
  assemble(file_content: str, start_line: int, end_line: int,
           task_description: str, dependencies: list[str]) -> FimPrompt
  FimPrompt fields: prefix: str, suffix: str, middle_marker: str
  Constraint: len(prefix_tokens) + len(suffix_tokens) + max_gen_tokens <= context_window
  Delimiter tokens: <|fim_prefix|>, <|fim_suffix|>, <|fim_middle|>
  PREFIX content order: file header comment, imports, referenced symbols, lines 1..start_line
  SUFFIX content: lines end_line..EOF or next 50 lines if file is large

ModelCapabilities
  supports_fim: bool  (true: Qwen2.5-Coder, StarCoder, DeepSeek-Coder; false: all cloud models)
  Queried from model metadata at registration time (not runtime detection)

ModeSelector
  select(step_spec: StepSpec, model_caps: ModelCapabilities,
         execution_mode: ExecutionMode) -> GenerationMode
  GenerationMode: FIM | DIFF | STRUCTURED

UnifiedDiffParser
  parse(model_output: str) -> Result<Vec<DiffHunk>, ParseError>
  DiffHunk fields: file_path: str, old_start: int, old_count: int,
                   new_start: int, new_count: int,
                   context_lines: list[str], added_lines: list[str], removed_lines: list[str]
  On total parse failure: returns Err(ParseError::NotUnifiedDiff) -> caller falls back to STRUCTURED

StreamingValidator
  on_token(token: str, position: TokenPosition) -> ValidationSignal
  on_chunk(partial_output: str, token_count: int) -> ValidationSignal
  ValidationSignal: Continue | EarlyTerminate(reason: str) | BlockToken
  Chunk interval: every ~50 tokens

DependencyResolver
  detect_missing(new_code: str, manifest_path: str, language: Language) -> Vec<MissingDep>
  resolve(dep: MissingDep, allowlist: Vec<str>) -> ResolutionDecision
  ResolutionDecision: AutoApprove | QueueForHuman | Substitute(replacement: str)

SpeculativeFileRouter
  route(step: StepSpec) -> RoutingDecision
  RoutingDecision: UseTarget(path: str) | NeedsMicroStep(micro: MicroStep)
  MicroStep: { action: "create_file", path: str, required_type: str, mode: STRUCTURED }
  Max micro-steps per baby step: 2

ContextPreAssembler (pipeline parallelism)
  pre_assemble(next_step: StepSpec) -> PreAssembledContext
  discard(ctx: PreAssembledContext) -> ()
  Pipeline depth: 1 (only one step ahead)
  Adaptive: disable pre-assembly when validation_pass_rate < 0.80


========================================================================
BEHAVIORAL CONTRACTS
========================================================================

FIM mode
  -> model outputs RAW code only (no JSON, no file path, no markdown)
  -> orchestrator inserts output at [start_line, end_line) region in target_file
  -> PostToolUse hooks (rules, AST, linter) validate after insertion
  -> on validation failure: deterministic feedback, retry FIM (not STRUCTURED)
  -> fallback to STRUCTURED only when model.supports_fim == false

DIFF mode
  -> model outputs unified diff starting with "--- a/{file_path}" / "+++ b/{file_path}"
  -> no explanation, no markdown fencing in model output
  -> each hunk wired through tools crate 3-tier edit matching: exact -> whitespace-normalized -> ellipsis-elision
  -> multi-hunk output supported (multiple regions in one pass)
  -> on total parse failure: track fallback_rate in SSB, fall back to STRUCTURED

STRUCTURED mode
  -> existing default (JSON with file_path + content + action_type)
  -> mandatory for: new file creation, cloud execution_mode, FIM/DIFF fallback
  -> cloud models always use STRUCTURED (200K context, overhead negligible)

ModeSelector deterministic rules (evaluated in order, first match wins)
  Rule 1: execution_mode == CLOUD                                    -> STRUCTURED
  Rule 2: step.type == "create_file"                                 -> STRUCTURED
  Rule 3: step.type == "edit_existing" AND step.edit_region.is_some()
          AND model.supports_fim == true                             -> FIM
  Rule 4: step.type == "edit_existing" AND step.changes_count > 1   -> DIFF
  Rule 5: else                                                       -> STRUCTURED

Streaming validation contracts
  Token-level: import/use statements checked against VERSION_GATE deprecated_patterns as they stream
  Token-level: todo!(), pass, unimplemented!(), "// TODO" detected in CODE positions only
    (tree-sitter partial parse required to distinguish code positions from comment/string positions)
  Token-level on detection: STOP generation, inject constraint, re-prompt
  Chunk-level every ~50 tokens: tree-sitter running parse; terminate if unclosed_brace_count grows unbounded
  Chunk-level: terminate if output_token_count > expected_length * 2
  Post-generation: full Tier 1/2 rule evaluation, linter, optional incremental compile (unchanged)

Pipeline parallelism contracts
  Pre-assembly runs DURING step N post-generation validation (rules, AST, linter)
  Pre-assembled context is DISCARDED if step N fails validation (dependency on N output)
  Pre-assembly cost on discard: ~10-50ms (acceptable)
  Pre-assembly benefit on pass: saves 100-500ms per step
  Historical pass rate ~88% makes depth=1 worthwhile
  NOT speculative LLM execution: only context assembly is pipelined

Dependency resolution contracts
  Model NEVER runs cargo add / pip install / npm install
  Model writes code with imports; system detects, resolves, installs
  Detection: diff import statements in new code against manifest
  Substitution check: deprecated_patterns must run before registry query
  Auto-approve conditions (autonomous mode only):
    (a) dep is in elai_cc.toml [dependencies.allowlist]
    (b) dep is already a transitive dependency (making explicit)
  Otherwise: queue for human intervention (blocks execution)

Speculative file routing contracts
  Model NEVER specifies file path in FIM/DIFF mode
  Plan step provides: target_file, edit_region {start_line, end_line}, action, description
  Mismatch detection: AST analysis finds unresolved imports -> check if new module needed
  Micro-step creation: MINOR ReplanRequest, runs in STRUCTURED mode, retries original step
  Guard: max 2 micro-steps per baby step (prevents infinite creation loops)


========================================================================
KEY TYPES AND SHAPES
========================================================================

StepSpec
  target_file: str           (mandatory in FIM/DIFF modes)
  edit_region: Option<{start_line: int, end_line: int}>  (mandatory for FIM)
  action: "edit_existing" | "create_file"
  description: str
  changes_count: int         (number of distinct edit regions)
  preferred_mode: Option<GenerationMode>  (hint only, orchestrator has final say)

PipelineSpec step gains preferred_mode: Option<GenerationMode> field (E-3)

DiffHunk
  file_path: str
  old_start: int, old_count: int
  new_start: int, new_count: int
  context_lines: list[str]
  added_lines: list[str], removed_lines: list[str]

MissingDep
  name: str
  source_language: Language  (Rust | Python | TypeScript)
  imported_from: str         (the import statement that triggered detection)

TokenPosition
  in_code_body: bool         (true = function/method body, not comment or string)
  in_import: bool

ValidationSignal
  Continue
  EarlyTerminate(reason: str)
  BlockToken


========================================================================
PLATFORM CONSTRAINTS
========================================================================

FIM tokens are model-specific: <|fim_prefix|>/<|fim_suffix|>/<|fim_middle|> for Qwen2.5-Coder
  Other model families may use different sentinel tokens — check at registration
Cloud models (Claude, GPT-4) do NOT support FIM — always fallback to STRUCTURED
FIM context constraint: prefix_tokens + suffix_tokens + max_gen must not exceed context_window
  For 3B model at 20K context: if prefix+suffix > ~18K tokens, SUFFIX must be truncated to next 50 lines
Streaming validation requires tree-sitter partial parse — must be fast enough for per-chunk use
  tree-sitter is cited as "fast enough" in spec; verify at ~50-token intervals
Pipeline parallelism: knowledge engine must be queryable concurrently with post-gen validation
  Constraint: no shared mutable state between pre-assembly and validation paths


========================================================================
GOTCHAS
========================================================================

FIM model quality: 3B models may produce worse code in FIM mode than structured mode
  (FIM is a different task than instruction-following)
  Benchmark required: if FIM pass rate is >10% lower than STRUCTURED, disable FIM for that model tier
  Decision is data-driven — do NOT assume FIM is better without measurement

Diff parsing failures: small models produce malformed diffs
  (wrong line numbers, missing context lines, mixed tabs/spaces)
  3-tier edit matching handles fuzzy application; track fallback_rate metric in SSB

Streaming validation false positives: "todo" in a TodoList class name
  Fix: AST-aware detection only — flag todo!/pass/unimplemented! in code body positions
  tree-sitter partial parse on streaming output is MANDATORY for this

Pipeline parallelism wasted work: if validation failure rate spikes, pre-assembled contexts
  are constantly discarded
  Adaptive depth: if validation_pass_rate < 0.80, disable pre-assembly until rate recovers

Approval gate blocks autonomous execution: new deps not in allowlist require human intervention
  Do not silently swallow missing dep — surface immediately as blocked intervention

SUFFIX truncation: spec says "next 50 lines if file is large" — what defines "large" is unspecified
  Implement a token budget check: if suffix would push total over context_window, truncate to 50 lines

Micro-step retry loop: max 2 micro-steps per baby step is a hard guard
  Do not remove this cap — spec explicitly states it to prevent infinite creation loops

Preferred mode is a hint: PipelineSpec step's preferred_mode is advisory
  Orchestrator ModeSelector has FINAL SAY and may override designer hint

Mode metrics MUST be tracked in SSB: mode_distribution (FIM/DIFF/STRUCTURED percentages per task)
  Required by E-3 spec; auditor will check for this


========================================================================
CRITICAL ASSERTIONS (testable)
========================================================================

assert fim_prompt_token_count(prefix, suffix, max_gen) <= model_context_window
assert mode == GenerationMode.STRUCTURED if execution_mode == ExecutionMode.CLOUD
assert mode == GenerationMode.STRUCTURED if step.action == "create_file"
assert mode == GenerationMode.FIM if (step.action == "edit_existing"
    and step.edit_region is not None and model.supports_fim)
assert diff_hunk.old_start > 0 and diff_hunk.new_start > 0
assert len(micro_steps_created_for_step) <= 2
assert validation_signal != EarlyTerminate if lazy_pattern_found_in_comment_or_string
assert pre_assembled_context_discarded if step_n_validation_failed
assert pipeline_depth == 1
assert new_dep_requires_user_confirmation_unless_in_allowlist_or_transitive
assert model_never_specifies_file_path_in_fim_or_diff_mode


========================================================================
COMPLEXITY
========================================================================

FIM prompt assembly: O(file_size) read + O(prefix_tokens + suffix_tokens) copy
  Space: O(context_window) per step

Diff parsing: O(output_length) single-pass
  Space: O(hunks * avg_hunk_size)

Streaming validation: O(1) per token for import check (hash lookup against deprecated_patterns)
  O(partial_output_size) per chunk for tree-sitter parse
  tree-sitter partial parse ~5-20ms per chunk at 50 tokens; acceptable for streaming

Pipeline parallelism: context assembly overlaps with O(validation_time) ~100-500ms
  Wasted work on failure: O(pre_assembly_time) ~10-50ms
  Net benefit: ~88% of steps save 100-500ms

Dependency detection: O(import_count * manifest_entry_count) per step
  Registry query: network-bound; async, non-blocking

Micro-step creation: O(1) per unresolved import detected; capped at 2 per step


========================================================================
IMPLEMENTATION STEPS (E-1 through E-7 from post-phase4-implementation-order.txt)
========================================================================

E-1: FIM prompt assembly + model capability detection
  Crates: providers/ (ModelCapabilities.supports_fim), orchestrator/ (FimPromptAssembler, output handler)
  Scope: ~300-400 LOC
  Prerequisites: Phase 4 complete (providers, tools, orchestrator exist)

E-2: Diff output parser + edit matching integration
  Crates: tools/ (diff parser module), orchestrator/ (diff mode handler)
  Scope: ~250-350 LOC
  Prerequisites: E-1 (generation mode infrastructure)
  Wire: each parsed hunk through tools crate 3-tier edit matching

E-3: Generation mode selection logic
  Crates: orchestrator/ (ModeSelector function), pipeline types (preferred_mode field)
  Scope: ~150-200 LOC
  Prerequisites: E-1 (FIM), E-2 (DIFF)
  Adds: mode_distribution metric to SSB

E-4: Streaming validation framework
  Crates: providers/ (streaming callback hooks), orchestrator/ (validation pipeline integration),
          rules/ (deprecated_patterns lookup)
  Scope: ~350-450 LOC
  Prerequisites: E-3 (mode selection determines when streaming runs)
  Metrics: early_termination_rate, tokens_saved_by_early_term

E-5: Pipeline parallelism for context pre-assembly
  Crates: orchestrator/ (superstep loop, context pre-assembly)
  Scope: ~200-300 LOC
  Prerequisites: E-3 (mode selection working), Phase 3D (BSP supersteps)
  Must respect BSP superstep boundaries

E-6: Automated dependency detection + resolution
  Crates: tools/ (dependency detection), orchestrator/ (resolution pipeline)
  Config: elai_cc.toml [dependencies.allowlist]
  Scope: ~350-450 LOC
  Prerequisites: Phase 4B (tools crate complete)

E-7: Speculative file routing + micro-step creation
  Crates: orchestrator/ (file routing, micro-step creation via MINOR ReplanRequest)
  Scope: ~200-250 LOC
  Prerequisites: E-3 (mode selection), Phase 4D (ReplanRequest exists)


========================================================================
FAILURE MODES
========================================================================

FIM quality degradation on small (3B) models: FIM pass rate >10% below STRUCTURED -> disable FIM for that tier
Malformed diff output from small models: all 3 edit-matching tiers fail -> fall back to STRUCTURED + track fallback_rate
Streaming validation false positives: lazy pattern detected in comment/string -> requires AST-aware detection
Pipeline pre-assembly wasted work: validation_pass_rate < 80% -> disable pre-assembly adaptively
New dep triggers blocked execution: not in allowlist, not transitive -> human intervention required
Micro-step creation loop: more than 2 unresolved imports in single step -> stop at 2, report as minor replan
FIM context overflow: large file where prefix+suffix exceeds context_window -> truncate suffix to 50 lines
Stale diff context lines: model uses wrong line numbers in diff -> edit matching fuzzy tiers handle; still track rate


========================================================================
IMPLEMENTATION PITFALLS (do NOT do this)
========================================================================

DO NOT use FIM mode for cloud execution_mode models (Claude, GPT-4 have no FIM token support)
DO NOT skip the deprecated_patterns check before registry query in dependency resolution
DO NOT allow model to specify file path in FIM or DIFF mode (orchestrator reads from plan only)
DO NOT allow tree-sitter detection of lazy patterns in comment/string positions (false positives)
DO NOT set pipeline depth > 1 (wasted work increases non-linearly; spec hard-limits to 1)
DO NOT remove the 2 micro-step cap per baby step (infinite loop risk)
DO NOT auto-approve new dependencies that are neither in allowlist nor already transitive
DO NOT treat preferred_mode as authoritative (it is a hint; ModeSelector has final say)
DO NOT skip mode_distribution metrics in SSB (required by E-3 spec, auditor checks)
DO NOT run cargo add / pip install from model output (model writes imports, system handles install)
DO NOT disable FIM based on assumption; benchmark actual pass rate first (10% threshold is data gate)
