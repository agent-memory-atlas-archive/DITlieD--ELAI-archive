INTEGRATION SPECS — LOSSLESS SUMMARY
Source folder: F:\.ELAI_workflow\docs\specs\integration\
Target project root: F:\.ELAI_workflow\.elai_cc\
Files covered: prompt-architecture.txt, self-modification.txt, media-validator.txt, pipeline-factory.txt, empire.txt


================================================================================
FILE 1 — prompt-architecture.txt
================================================================================

1. PURPOSE
Eliminates prompt-driven orchestration by moving workflow control into Rust code; the FM becomes a pure worker that receives a structured prompt, produces output, and gets validated by deterministic code. Replaces all .md rule/skill/agent files with code + SQLite + tiny role_prompts in elai_cc.toml.

2. KEY TYPES & STRUCTS
Roles (in [agents.{role}.role_prompt]): executor, auditor, prompt_engineer, context_compiler, memory_writer.
Designer (NEW): produces PipelineSpec from Research Report; distinct from Executor; runs in Stage 2 of PLANNING DispatchPipeline; GRC 8/1200.
Dispatch types (defined in orchestrator.txt S14, referenced here):
  MetaDispatchTable — static, hardcoded; used in PLANNING/CONFIGURING.
  PlanDispatchTable — dynamic, from PipelineSpec; used in all other states.
  DispatchPipeline — multi-stage.
  DispatchStage — { skip_condition, prompt assembly, agent spawn, gate }.
  AgentDispatchConfig.
  ToolFilter.
  ArtifactType.
  GateCondition.
  SkipCondition.
TaskPacket field: initial_route (set by CLI from prefix /task, /config, /fix, etc.).
TransformedPrompt — return type of orchestrator::prompt_transform(task, session).

Prompt assembly sources (4):
  SOURCE 1 — ROLE PROMPT: 50-100 tokens, from elai_cc.toml [agents.{role}.role_prompt]
  SOURCE 2 — ACTIVE TIER 3 RULES: 100-300 tokens, SELECT from rules.db
  SOURCE 3 — CODE CONTEXT: 2000-8000 tokens (budget-fitted), from KnowledgeEngine
  SOURCE 4 — TASK SPEC: 200-500 tokens, from plan skeleton + phase detail
  Output format + constraints: 50-100 tokens

Token budgets per turn:
  Total meta-instructions assembled: ~200-500 tokens (target ~330)
  Total prompt: ~2500-9000 tokens (mostly code context)
  Hard ceiling: < 500 meta-instruction tokens per turn.

3. PROTOCOLS / WIRE FORMATS
Tier 3 rules SQL query (verbatim):
  SELECT description FROM rules
  WHERE status = 'ACTIVE' AND tier = 3
    AND domain IN (current_domains)
    AND (phase_mask & current_phase_bit) != 0
  ORDER BY severity DESC
  LIMIT 10
Tier 3 injection format:
  "Domain constraints (mandatory):
   1. <description>. [<rule_id>]
   2. ..."

Example role_prompts (verbatim TOML):
  [agents.executor]
  role_prompt = """You write code to pass failing tests.
  Output: one code block per file, prefixed with file path.
  Do not explain. Do not add features beyond what tests require."""

  [agents.auditor]
  role_prompt = """You are an adversarial code reviewer.
  Every finding must cite line number and evidence.
  You must find at least one issue per review."""

  [agents.prompt_engineer]
  role_prompt = """Transform the raw task into a structured prompt.
  Output: XML with instructions, context, task, output_format blocks."""

  [agents.context_compiler]
  role_prompt = """Gather relevant code context for the given task.
  Output: structured context package with blast radius and interfaces."""

  [agents.memory_writer]
  role_prompt = """Record changes from this phase into the knowledge base.
  Output: structured change manifest."""

Auditor output JSON (verbatim):
  { findings: [{severity, description, evidence, rule_id}] }

What the model NEVER does (full list, verbatim):
  - Decides FSM transitions
  - Spawns agents
  - Loads rule files
  - Manages sessions or file locks
  - Decides what context to load
  - Calls RAG tools
  - Calls the LLM provider directly
  - Executes tools directly
  - Accesses the network (except researcher role)
  - Sees enforcement logic, rules database, or harness internals
  - Sees internal tool parameters (_rule_context, _enforcement_tier, etc.) — sanitized

What the model ONLY does:
  - Receives structured prompt
  - Produces output in constrained format
  - Gets validated by deterministic code
  - Runs in isolated OS process

4. STATE MACHINES / LIFECYCLES
End-to-end deterministic dispatch flow (Section 10):
  1. User types task in CLI (/task, /config, /fix, etc.)
  2. CLI strips prefix, sets initial_route in TaskPacket.
  3. Orchestrator reads FSM state. Selects dispatch table:
     - PLANNING/CONFIGURING: MetaDispatchTable (static, hardcoded).
     - All others: PlanDispatchTable (dynamic, from PipelineSpec).
  4. Dispatch table returns DispatchStage or DispatchPipeline (multi-stage).
  5. For each stage: skip_condition check -> prompt assembly -> agent spawn -> gate.
  6. Prompt assembly from 4 sources (role_prompt + tier_3_rules + code_context + task_spec); ~330 tokens meta.
  7. Agent works via IPC. Returns AgentResponse.
  8. Evidence gate -> FSM transition -> phase_complete SSB record -> model swap if needed.

Enforcement comparison:
  Tier 1/2 enforcement (0 points of failure):
    1. Orchestrator detects domain from file paths (deterministic)
    2. Rules engine loads active Tier 1 predicates (SQLite query)
    3. Model produces output (unconstrained)
    4. PostToolUse hook evaluates predicates on output (Rust, <1ms)
    5. BLOCK verdict -> output rejected, model re-prompted
  Tier 3 enforcement:
    1. Orchestrator injects 5-10 relevant Tier 3 rules into prompt (~100 tokens)
    2. Model produces output
    3. Auditor agent reviews output against Tier 3 rules
    4. BLOCK findings -> output rejected
    Self-discovery: repeated Tier 3 violations -> proposed Tier 1/2 rule (rules.txt section 6)

5. CONFIG SCHEMA EXCERPTS
elai_cc.toml retains:
  - Agent model assignments
  - GRC budget defaults
  - Domain mapping overrides (used by rules::detect_domains)
  - Hardware config
  - [rules.domain_mapping] (file-path-to-Domain)
  - [agents.{role}.role_prompt] strings
Plan skeleton: stored in .elai_cc/pipelines/{id}/skeleton.toml.
Tier 3 rule descriptions: stored in rules.db as description TEXT.

6. INTEGRATION POINTS (.md files replaced)
  4.1 session-protocol.md   -> orchestrator::session_init()
  4.2 multi-session.md      -> orchestrator::SSB + session locks (auto SSB.claim/check_conflicts/attribute_errors/release; PostToolUse hook BLOCKS lock-violating writes)
  4.3 conditional-routing.md-> rules::domain_detection() — fn detect_domains(file_paths: &[&str]) -> Vec<Domain>
  4.4 output-formatting.md  -> (a) into role_prompt; (b) orchestrator::memorize() in MEMORIZING
  4.5 planning-workflow SKILL.md -> orchestrator FSM engine (entire orchestrator crate)
  4.6 prompt-transform SKILL.md  -> orchestrator::prompt_transform(task, session) — 3-line Rust fn
  4.7 rag-tools SKILL.md         -> knowledge::KnowledgeEngine (internal; mcp_search_context/mcp_memorize/mcp_ingest_file ALL GONE)
  4.8 CLAUDE.md                  -> elai_cc.toml + hardcoded Rust:
        FSM gate check        -> orchestrator FSM
        Anti-Slopsquatting    -> rules.db Tier 1 GREP rule
        Zero Lazy Code        -> rules.db Tier 1 AST rule
        Anti-Spiral           -> orchestrator::anti_spiral() Rust fn
        GRC tools             -> reasoning::GrcEngine (not model-invoked tools)
        Context delegation    -> orchestrator prompt assembly
        Output formatting     -> role_prompt
  4.9 Agent .md files            -> [agents.{role}.role_prompt] in elai_cc.toml

Cognitive isolation metric (cross-ref orchestrator.txt §7A):
  context_savings_from_isolation = sum(agent_grc_tokens) / model.max_context
  On 8k local model: 12k internal reasoning -> isolation saves 150% of context window (makes task possible).
  On 32k cloud model: 30-40% savings.

7. PHASE DEPENDENCIES
  Phase 3F: rules engine compiles all .md rule files into rules_seed.db (PREREQUISITE).
  Phase 4: workflow integration replaces all .md routes per Section 9 list.
  After Phase 4: zero .md files remain in orchestration path.

8. RISKS / OPEN QUESTIONS
- Migration assumes rules_seed.db successfully captures all current .md rules at Tier 1/2/3 granularity.
- "_rule_context, _enforcement_tier" sanitization spec is referenced ("swarm-derived pattern") but not defined in this file.
- Cognitive isolation metric defined cross-spec; requires orchestrator.txt §7A.

9. TESTABLE INVARIANTS
- Model prompt contains zero orchestration instructions (no FSM, no spawn, no routing)
- Total meta-instruction tokens < 500 per turn
- Tier 1/2 rules never appear in model prompt
- Only Tier 3 rules for current domain + phase injected
- Domain detection deterministic
- role_prompt round-trips through elai_cc.toml
- Prompt assembly < 5ms (no file I/O, all from memory/SQLite)


================================================================================
FILE 2 — self-modification.txt
================================================================================

1. PURPOSE
A 3-layer self-improvement system where the modifying agent NEVER evaluates its own modifications — deterministic benchmarks judge. Adopts DGM (Darwin Godel Machine, arxiv 2505.22954) population-based archive + log-driven proposals + staged eval + self-referential alignment, applied to configs/rules around a frozen FM (not modifying the FM itself).

2. KEY TYPES & STRUCTS
PipelineOutcome { template_id, project_id, phases_total, phases_passed_first_try,
  audit_findings_total, audit_findings_block, test_coverage_final,
  time_to_complete_seconds, tokens_consumed_total, human_interventions,
  rules_triggered: Vec<(RuleId, TriggerCount, CatchRate)>,
  test_outcomes: TestOutcomeReport }

TestOutcomeReport (4-category SWE-bench-derived grading):
  FAIL_TO_PASS — failed before, pass after (resolution)
  PASS_TO_PASS — passed before, still pass (maintenance)
  FAIL_TO_FAIL — failed before, still fail (no regression, no fix)
  PASS_TO_FAIL — passed before, now fail (REGRESSION)
  Source: SWE-bench harness grading.py get_eval_tests_report().
  Gate rule: any PASS_TO_FAIL -> reject variant even if positive FAIL_TO_PASS.

config_variants table schema (verbatim):
  config_variants(
    variant_id TEXT PK,
    parent_id TEXT,
    score REAL,
    children_count INTEGER,
    config_blob BLOB,
    created_at INTEGER,
    pipeline_outcomes TEXT,
    status TEXT CHECK(status IN ('ACTIVE','ARCHIVED','REVERTED'))
  )

DGM sigmoid-weighted parent selection (DGM §2.3):
  s_i = sigmoid(lambda * (alpha_i - alpha_0))   [performance; lambda=10, alpha_0=0.5]
  h_i = 1 / (1 + n_i)                            [exploration bonus; n_i = children count]
  w_i = s_i * h_i;  p_i = w_i / sum(w_j)         [all variants non-zero probability]

Modification Solidification Index (MSI, DarwinNet-derived, arxiv 2604.01236):
  MSI = weighted_moving_avg(delta_score) over last N variants (default N=10)
  delta_score = variant.score - variant.parent.score
  Range: [-1, 1]
  Phase classification + actions:
    MSI > 0.3            IMPROVING   -> maintain 1 self-mod per 3 pipelines
    -0.1 < MSI <= 0.3    STABILIZING -> reduce to 1 per 5 pipelines; log "diminishing returns"
    MSI <= -0.1          DIVERGING   -> pause self-mod, alert user, revert to highest-score variant
  Stored in self_mod_history.db. Computed after every Layer 2 benchmark. Pure arithmetic.

Slot-restricted patches (IC3-Evolve pattern):
  Slot 1: GRC budgets (max_calls, max_chars per role)
  Slot 2: Rule thresholds (FP rate, TP rate, promotion gates)
  Slot 3: Context budget percentages (repo_map_pct, code_context_pct, ssb_pct, grc_pct)
          Note: auditor's code_context_pct change requires human approval (Layer 3).
  Slot 4: Model parameters (temperature, top_p per role)
  Max 1 slot per proposal. Cross-slot requires Compass&Jump mode.
  Forbidden: FSM states, IPC protocol, security rules, evidence gates.

Anti-reward-hacking READ-LOCKED (PreToolUse enforced) during self-mod:
  metrics code, gate definitions, auditor config, benchmark harness, test suite, config_variants table schema.

3. PROTOCOLS / WIRE FORMATS
Layer-2 process (9 steps):
  1. EVALUATE: after every pipeline, self-mod agent reads SSB logs (cheap; max 1 self-mod per 3 completed pipelines)
  2. SELECT PARENT: DGM sigmoid-weighted from config_variants archive
  3. PROPOSE: log-driven proposal targeting specific config change
  4. PLAN: PipelineSpec targeting .elai_cc/ config (full 4-stage research->plan->validate->probe)
  5. SANDBOX: copy parent config to .elai_cc/sandbox/{mod_id}/
  6. BENCHMARK staged:
       Stage 1: 10 tasks (sanity) — fail -> discard
       Stage 2: 50 tasks (if Stage 1 passes; include trigger task)
       Stage 3: 200 tasks (if >40% success on 60-task subset AND ranks top 2 in archive)
                (Threshold per DGM §2.5; any PASS_TO_FAIL -> reject; include 3 SWE-bench rotation instances)
  7. PROOF GATE (IC3-Evolve, before Stage 2):
       Run DST reference model with proposed config against 10 FIXED canonical regression cassettes.
       SOUNDNESS check (not behavioral identity):
         PASS: all evidence gates trigger correctly, no BLOCK findings bypassed, no PASS_TO_FAIL, FSM reaches COMPLETE.
         FAIL: evidence gate bypassed (e.g. UNGROUNDED claim reaches EXECUTING), regression detected, FSM stuck/crashes.
       Distinct from IC3 (correctness): we check soundness; differing BLOCK/ALLOW distributions are valid.
  8. FINAL GATE: ALL of:
       better on trigger task AND no SWE-bench regression AND auditor finds no safety regressions
       AND no evidence gate definitions modified AND proof gate passed
       PASS -> INSERT new variant (status='ACTIVE'), demote previous active to 'ARCHIVED'
       FAIL -> insert as variant with low score (kept in archive for future branching)
  9. CROSS-TRANSFER VALIDATION (post-gate, optional):
       Re-test winning config with a different FM (e.g. cloud if primary local).
       Fail -> flag as FM-specific optimization, keep but mark.

4. STATE MACHINES / LIFECYCLES
3-layer architecture:
  LAYER 1 — DATA (automatic, no approval): template rankings, rule trigger counts, pipeline outcome metrics. Pure arithmetic from PipelineOutcome. NO LLM. Security/safety/architecture rules NEVER auto-adjusted.
  LAYER 2 — CONFIG (full pipeline, auditor approval): new rules, GRC budgets, context ratios, model assignments, templates, tool configs. Process = 9 steps above.
  LAYER 3 — STRUCTURE (human approval, always): FSM state additions, new agent roles, GRC protocol changes, evidence gate modifications, core pipeline control flow.

Rollback protocol:
  Every Layer 2 mod: new variant in config_variants archive.
  Monitored for 3 pipeline runs.
  Auto-revert if metrics degrade > 10% (variant status set to 'REVERTED', previous active variant restored).
  Storage: .elai_cc/global/self_mod_history.db

Resource scheduling:
  Lowest priority. Idle time only. User task always preempts.
  Evaluation cadence: after every pipeline (cheap SSB query).
  Execution cadence: gated by MSI (default 1/3; STABILIZING 1/5; DIVERGING paused).
  Staged eval: Stage 1 cheap, Stage 3 (200 tasks) ~90 min at lowest priority.

5. CONFIG SCHEMA EXCERPTS
Storage paths:
  .elai_cc/global/self_mod_history.db (config_variants + MSI history)
  .elai_cc/sandbox/{mod_id}/ (per-mod sandbox copy of parent config)
Archive size: max 50 variants (oldest low-score variants archived to cold storage).
Active config: exactly one variant with status='ACTIVE' at any time.

Rule weight adjustment:
  SAFETY RULES (Security, Safety, Architecture): NEVER auto-adjusted.
  ADJUSTABLE RULES (Style, Custom): high false positive -> flag for REFINEMENT (not removal).

6. INTEGRATION POINTS
- SSB (orchestrator): provides full pipeline trace, every phase outcome, audit findings, anti-spiral triggers, context budget reductions, rule trigger counts, reasoning_trace entries (orchestrator.txt §7A).
- rules engine: Tier 1/2/3 rule database; counterfactual rule testing referenced (rules.txt §5A).
- pipeline-factory L2: builds sandbox PipelineSpec for the mod itself.
- auditor agent: reviews proposed change for safety regressions.
- DST reference model: required for proof gate.
- 10 canonical regression cassettes: required input artifact (location not specified in this file).

7. PHASE DEPENDENCIES
Implicitly requires:
  - Rules engine (Phase 3F)
  - Orchestrator FSM with reasoning_trace (orchestrator.txt §7A)
  - SSB with full trace logging
  - Auditor + benchmark harness
  - PreToolUse READ-LOCK enforcement
  - DST reference model
  - 10 regression cassettes
Source attribution: design_document.txt PART 22 (self-improvement loop), PART 27 (self-modification).

8. RISKS / OPEN QUESTIONS
- Location of 10 canonical regression cassettes is not specified.
- "Compass&Jump mode" referenced for cross-slot changes but not defined in this file.
- "DST reference model" referenced but not specified.
- "counterfactual rule testing (rules.txt §5A)" — requires rules.txt to define semantics.
- "highest-score variant" restoration during DIVERGING — selection criteria not detailed.
- arxiv 2604.01236 cited for DarwinNet/PSI is a future date relative to general training data; verify citation when grounding.
- Limitation acknowledged: FM is frozen, so loop is weaker than DGM (no FM coding-ability improvement).

9. TESTABLE INVARIANTS
- Self-mod agent cannot write to metrics/gates/auditor-config during pipeline
- Layer 2 benchmarked against deterministic metrics (no LLM judgment)
- Auto-revert on >10% degradation over 3 pipelines (status -> REVERTED)
- Layer 3 always requires human_approved=true
- Sandbox physically separate from active config
- Config archive: only one ACTIVE at a time
- Config archive: all variants non-zero selection probability (sigmoid-weighted)
- Config archive: max 50 variants (overflow archives lowest-score)
- Staged evaluation: Stage 1 failure -> no Stage 2 (early termination enforced)
- Log-driven proposals: agent reads SSB logs (not heuristic threshold only)
- Cross-transfer validation: FM-specific optimizations flagged
- Self-referential loop: every proposal cites specific SSB log entries as evidence
- MSI computed after every Layer 2 benchmark (pure arithmetic)
- MSI DIVERGING pauses self-mod and alerts user
- MSI STABILIZING reduces frequency to 1 per 5 pipelines
- MSI in [-1, 1] range
- Config variant with any PASS_TO_FAIL rejected at gate
- TestOutcomeReport computed for every pipeline (4-category, deterministic)
- Proof gate runs against 10 canonical cassettes before Stage 2
- Proof gate rejects evidence-gate-bypass; allows different BLOCK/ALLOW distributions
- Slot restriction: max 1 slot per proposal
- Cross-slot requires Compass&Jump mode (proposal validator enforced)
- Slot 3 auditor code_context_pct -> Layer 3 escalation


================================================================================
FILE 3 — media-validator.txt
================================================================================

1. PURPOSE
A specialized agent that validates non-code outputs (images, 3D models, audio, UI screenshots). Loaded only when PipelineSpec contains media-producing steps. Implements a two-tier (deterministic + VLM) validator and a Render-Capture-Evaluate visual feedback loop that gives the pipeline "eyes."

2. KEY TYPES & STRUCTS
Agent: role=media_validator; GRC budget low (3/600); Permissions=FILESYSTEM (read-only on output files).

Tier-2 VLM models (two modes):
  MODE A — Single image deep analysis:
    Primary: Qwen3.5-VL-30B-A3B (MoE), 8-bit; built-in Thinking Mode CoT. 70-140 tok/s; 8-bit detail fidelity.
    Alternative max-intelligence: Qwen3.5-VL-72B Instruct, 4-bit EXL2/AWQ. Requires input downscale.
  MODE B — Sequential image flow:
    Model: InternVL3.5-38B, 8-bit (or 4-bit for very long sequences). 5.5B-param vision encoder.
    Engine: MUST use SGLang (not vLLM/ollama) — RadixAttention caches prior image visual state.

Audio Tier-2 model: CLAP (Xenova/clap-htsat-unfused ONNX export — no official ONNX for fused). ~0.8-1.0 GB FP32, ~0.4 GB FP16. Input MUST be raw PCM (avoid mel-spectrogram BLAS variance from FFmpeg).

Render abstraction trait (verbatim):
  trait MediaRenderAdapter {
      fn can_handle(&self, output_type: &OutputType) -> bool;
      async fn render(&self, artifact: &Path, config: &RenderConfig) -> Result<Vec<CapturedFrame>>;
      fn cleanup(&self);
  }
  CapturedFrame { image_path: PathBuf, viewport: Viewport, metadata: FrameMetadata }
  Viewport { width: u32, height: u32, device: Option<String> }
  FrameMetadata { capture_time: Duration, adapter_name: String, state_description: String }

Built-in adapters:
  WebRenderAdapter — Playwright (headless Chromium/Firefox/WebKit); breakpoints: mobile 375px, tablet 768px, desktop 1440px; integration via Playwright MCP (Microsoft) or direct CDP; can capture post-interaction via Vec<Interaction> in RenderConfig.
  BlenderRenderAdapter — Blender CLI (--background --python); supports .obj, .fbx, .gltf, .blend; default 4 angles (front, right, top, 3/4 perspective).
  AnimationRenderAdapter — FFmpeg; key frames at intervals (e.g. every 1s, or 0/25/50/75/100% of duration); for video extract frames; for GIF/Lottie render to frames first.
  UiScreenshotAdapter — direct file read; validate format, optionally resize.
  ComfyUIAdapter — ComfyUI REST API (POST /prompt, poll /history, download); timeout 600s.

VLM output: VlmValidationOutput (structured; constrained decoding) with: intent match boolean, list of issues, severity per issue {Critical, Major, Minor}.

3. PROTOCOLS / WIRE FORMATS
ComfyUI REST API:
  POST /prompt — submit workflow JSON
  GET  /history — fetch results
  Timeout: 600s for GPU rendering
  Then Tier 1 + optional Tier 2 validation.

External tools used (each registered via Tool Auto-Discovery from pipeline-factory.txt §7):
  ImageMagick, Blender CLI, FFmpeg, Tesseract, ComfyUI API.

PipelineSpec phase fields (verbatim TOML):
  [[phases]]
  name = "build_landing_page"
  output_type = "web"         # -> WebRenderAdapter
  visual_check = true

  [[phases]]
  name = "generate_character_model"
  output_type = "3d_model"    # -> BlenderRenderAdapter
  visual_check = true

4. STATE MACHINES / LIFECYCLES
Two-tier validation:
  TIER 1 — DETERMINISTIC (always first; no model; <100ms; CPU only):
    Image: format, resolution, file size, color space, SSIM/LPIPS vs reference.
    3D model: vertex/face count, manifold check, UV overlap, normals, bounding box (Blender CLI headless).
    Audio: sample rate, bit depth, duration, silence %, clipping detection.
    UI screenshot: resolution, SSIM vs mockup, color histogram, OCR text verification.
  TIER 2 — VLM QUALITATIVE (only when Tier 1 passes AND QualitySpec specifies; ~2-5s):
    VLM ADVISORY only; Tier 1 has VETO power.
    VLM disagreement with Tier 1 -> flag for human review.

GPU scheduling:
  Tier 1: CPU only.
  Tier 2 MODE A (Qwen3.5-VL-30B-A3B MoE): all 30B params in VRAM (active ~3B). 8-bit ~30GB; Q4_K_M ~17GB. Does not fit alongside other models on 24GB GPU.
    Swap: unload executor -> load VLM Q4 -> validate -> unload -> reload executor. ~10-15s at phase boundaries.
    Phase 0 (RTX 3060 12GB): cannot run -> use mock VLM.
  Tier 2 MODE B (InternVL3.5-38B): full 38B; dedicated GPU; unload executor/auditor first; SGLang manages VRAM; swap cost ~30-60s.

Render-Capture-Evaluate loop (sub-loop inside EXECUTING; NOT a new FSM state):
  1. EXECUTE — agent produces output
  2. RENDER — MediaRenderAdapter renders output to image(s)
       Adapter selection: L2 pipeline builder maps PipelineSpec.output_type -> adapter via Tool Auto-Discovery (pipeline-factory.txt §7)
  3. CAPTURE — Vec<CapturedFrame>
  4. TIER 1 EVALUATE — deterministic checks
  5. TIER 2 EVALUATE — VLM with frames + QualitySpec + original intent; produces structured VlmValidationOutput
  6. FEEDBACK — structured issues to executor as correction signal (specific, actionable, localized; NOT "looks bad")
  7. ITERATE — executor revises -> step 1
       Max iterations: configurable (default 3)
       After max: escalate to INTERACTIVE gate
       Convergence: if Tier 2 issues unchanged between iterations -> stop (human review)

Adapter degradation rules (L2 plan assembly):
  visual_check=true AND adapter exists AND tool unavailable -> DEGRADED validation (Tier 1 only)
  output_type has NO adapter -> L2 REJECTS PipelineSpec step (cannot validate what cannot be seen)

5. CONFIG SCHEMA EXCERPTS
Per-phase visual_check field defaults to false; non-visual outputs incur zero render overhead.
Mixed pipelines: visual_check is per-phase (backend phases off, frontend on).

OutputType enum (cross-ref pipeline-factory §10): Web, Model3D, Audio, UiScreenshot, Animation, Image, Custom(String).

6. INTEGRATION POINTS
- pipeline-factory.txt §7 (Tool Auto-Discovery) — registers all render tools.
- pipeline-factory.txt §10 (QualitySpec.MediaSpec) — defines MediaSpec, OutputType, visual_check.
- orchestrator FSM EXECUTING phase — hosts the Render-Capture-Evaluate sub-loop.
- INTERACTIVE gate — escalation point after max iterations.
- L2 builder — maps output_type -> adapter, validates adapter availability.

7. PHASE DEPENDENCIES
Implementation deferred until PipelineSpec contains media steps.
Phase 0 (RTX 3060 12GB) constraint: MODE A cannot run; mock VLM required.
Source: design_document.txt PART 28 (all subsections).

Prior-art citations (grounding sources):
  Design2Code (arxiv 2403.03163) — visual-to-code benchmark, CLIP/CW-SSIM/block-matching, 484 webpages.
  ScreenCoder (arxiv 2507.22827) — modular multi-agent visual-to-code.
  SeeAct (ICML 2024) — generalist web agent using screenshots.
  VisualWebArena (arxiv 2401.13649) — multimodal web-agent benchmark; Set-of-Mark prompting; VLM 16.4% vs human 88.7%.
  Tweag Agentic Coding Handbook — "Visual Feedback Loop" workflow.
  Playwright visual regression testing — pixel-diff with threshold, cross-browser.
Gap: existing systems are human-in-the-loop or benchmark-only; none implement fully autonomous render-capture-evaluate inside a pipeline engine. Novel integration, not novel research.

8. RISKS / OPEN QUESTIONS
- Phase 0 hardware (RTX 3060 12GB) cannot host Tier-2 VLM; mock VLM is acknowledged but not specified.
- Cross-spec type OutputType is defined in pipeline-factory.txt §10 — must stay consistent across both files.
- ComfyUIAdapter overlaps with ComfyUI integration mentioned in §6; only one registration path should win.
- Convergence detection ("issues unchanged") requires structural diff of VlmValidationOutput across iterations (not specified here).

9. TESTABLE INVARIANTS
- Tier 1 always before Tier 2
- Tier 1 fail overrides Tier 2 pass
- VLM output structured (constrained decoding)
- Only loaded when PipelineSpec has media steps
- SSIM/LPIPS scores deterministic
- Render-capture-evaluate loop max iterations enforced (default 3)
- Convergence check stops iteration when issues unchanged
- Adapter selection deterministic (output_type -> adapter, no LLM)
- visual_check=false -> zero render overhead (adapter never instantiated)
- Missing adapter tool -> DEGRADED (not FAIL, not silent skip)


================================================================================
FILE 4 — pipeline-factory.txt
================================================================================

1. PURPOSE
elai_cc as a universal automation engine: user provides ANY goal; system DESIGNS the pipeline (L1 LLM Designer), BUILDS it (L2 deterministic Rust), EXECUTES it (L3 existing FSM/agents/rules). LLM handles creative work; everything else is deterministic infrastructure.

2. KEY TYPES & STRUCTS
3-layer architecture:
  L1 — PIPELINE DESIGNER (LLM-driven, GRC-grounded):
    Agent role: Designer (distinct from Executor; see orchestrator.txt §14).
    Input: human task description + SystemConstraints (model_max_context, available_for_work, avg_file_tokens, avg_test_tokens).
    L1 Observe step injection: "Context budget: {available_for_work} tokens. Each file averages {avg_file_tokens} tokens. Phase must touch at most {floor(available_for_work * 0.4 / avg_file_tokens)} files."
    Bootstrap defaults (when knowledge index has <10 files):
      avg_file_tokens = 150
      avg_test_tokens = 80
      Source: elai_cc.toml [planning.sizing]
    Output: PipelineSpec (TOML/msgpack) + QualitySpec.
  L2 — PIPELINE BUILDER (deterministic, no LLM):
    Validates components, creates FSM states, configures agents, installs rules, estimates resources, generates dependency graph, creates test harness skeleton.
    Output: ExecutablePipeline in .elai_cc/pipelines/{id}/
  L3 — PIPELINE EXECUTOR (existing FSM + agents + rules): runs INTERACTIVE or AUTONOMOUS.

ResearchTree:
  struct ResearchTree {
      plan: ResearchPlan,
      state: SharedResearchState,
      root: ResearchNode,
      breadth_factor: u32,          // default: 3
      depth_limit: u32,             // default: 5
      max_iterations: u32,          // per-node, default: 10
      phi_min: f64,                 // goal satisfaction threshold, default: 0.7
      psi_min: f64,                 // quality threshold, default: 0.6
      speculative_depth: u32,       // default: 2
  }
  ResearchNode {
      question: String,
      parent: Option<NodeId>,
      children: Vec<NodeId>,
      phi: f64,                     // [0,1]
      psi: f64,                     // [0,1]
      status: NodeStatus,           // Pending | Active | Converged | Pruned
  }

ResearchReport:
  ResearchReport {
      query: String,
      tree: ResearchTreeSnapshot,
      sub_questions: Vec<SubQuestionResult>,
      total_sources_fetched: usize,
      total_sources_used: usize,
      total_iterations: usize,
      total_nodes_explored: usize,
      total_nodes_pruned: usize,
      speculative_hit_rate: f64,
      convergence_status: Vec<(String, ConvergenceStatus)>,
  }
  SubQuestionResult {
      question: String,
      answer: String,
      sources: Vec<SourceCitation>,    // URL + extracted quote + quality tier
      confidence: f64,                 // source count * quality * agreement
      phi: f64,
      psi: f64,
      depth: u32,
      alternatives: Vec<Alternative>,
      contradictions: Vec<Contradiction>,  // surfaced, never silently resolved
  }

QualitySpec types:
  QualityMetric { name, target, unit, method, phase, hard_fail }
  MediaSpec { output_name, format, resolution, ssim_threshold, vlm_check, output_type: OutputType }
  OutputType enum { Web, Model3D, Audio, UiScreenshot, Animation, Image, Custom(String) }
  PerfSpec { name, target, unit, method, sample_count }
  Per-phase visual_check: bool (default false). True AND phase has MediaSpec -> Render-Capture-Evaluate activates during EXECUTING.
    L2 validates: visual_check=true requires a MediaRenderAdapter for phase output_type. Missing -> ValidationError.

Completeness QualityMetric examples (verbatim):
  { name: "image_extraction", target: 100, unit: "%",
    method: "grep_output", phase: "EXECUTING", hard_fail: false }
  { name: "table_extraction", target: 100, unit: "%",
    method: "grep_output", phase: "EXECUTING", hard_fail: false }
  { name: "modality_coverage", target: 100, unit: "%",
    method: "count_modalities", phase: "AUDITING", hard_fail: true }
Method semantics:
  grep_output — scans tool results
  count_modalities — checks scope enumeration

ValidationError variants referenced:
  ValidationError::CyclicDependency (with cycle path)

Source quality tiering (deterministic, domain-based lookup):
  TIER_1 (1.0): arxiv, official docs, RFC
  TIER_2 (0.8): GitHub repos >100 stars, StackOverflow accepted answers
  TIER_3 (0.6): blog posts from known technical authors, conference talks
  TIER_4 (0.4): general blog posts, forum discussions
  TIER_5 (0.2): social media, unverified
  Configurable in elai_cc.toml [research.trusted_domains]
  Ranking: evidence_level * source_quality_avg (pure arithmetic)

Evidence levels: BENCHMARK > OFFICIAL_DOCS > BLOG_POST > TRAINING.

Model Capability Registry (PART 15.6): per-model {capabilities, vram_required, max_context, strengths, weaknesses, provider}. Selection: score = sum(capability_match) * context_fit * vram_feasibility. NO LLM judgment. Extension: L1 queries ollama /api/tags, runs capability probe, registers.

External Tool Auto-Discovery (PART 15.7):
  Three states: AVAILABLE (on PATH), INSTALLABLE (package manager), UNAVAILABLE.
  Tool wrapper trait: execute(args)->ToolResult, validate_output()->bool, version()->SemVer.
  Auto-discovery sources: PATH, package managers, web search. Auditor reviews proposals.

Self-Extending Rules (PART 15.2): L1 creates new rule categories at runtime; proposed as Rust-evaluable predicates; auditor validates. Templates shipped: SaaS, Trading, Game, Data, Security, ML, Custom.

3. PROTOCOLS / WIRE FORMATS
Per-node research loop:
  1. FORMULATE — GRC generates search query
  2. SEARCH — web search returns top N (default 10)
  3. EXTRACT — fetch content, sampling-based filter (read first 500 tokens, score, read full if above threshold)
  4. REFLECT — GRC evaluates: phi, psi, contradictions, new sub-questions (bounded by breadth_factor), info gain
  5. CONVERGE/EXPAND/ITERATE:
     phi >= phi_min AND psi >= psi_min -> converged
     phi < phi_min -> iterate or spawn children
     contradictions -> spawn disambiguation child
     depth >= depth_limit -> stop, partial with confidence
     max_iterations -> partial with confidence

Speculative execution (FlashResearch-derived):
  When phi > 0.4, begin researching top speculative_depth children before parent converges.
  Pruned if parent changes direction.
  Cost bound: speculative_depth * breadth_factor.
  Benefit: 30-50% wall-clock reduction (FlashResearch benchmark).
  Local models: DISABLED (single GPU, sequential only).
  Cloud APIs: ENABLED (rate-limit aware).

Adaptive resource allocation:
  Simple query (single concept):   breadth=2, depth=3
  Moderate query (comparison):     breadth=3, depth=4
  Complex query (novel domain):    breadth=4, depth=5
  Complexity assessed by GRC during PLANNING (observe step).

Parallel sub-question processing:
  - Root children: parallel (independent sub-questions)
  - Speculative children: parallel with parent (cloud only)
  - Local models: sequential within each tree level
  - Cloud APIs: parallel across breadth, rate-limit aware
  - Deferred aggregation node collects all sub-results
  - Pruned branches free resources immediately
  Cross-ref: orchestrator.txt §8 Dynamic Fan-Out.

Research progress to SSB:
  action_type='research_step'
  payload: { sub_question, iteration, urls_fetched, findings_count, info_gain }

INTERACTIVE mode (default, Phase 3): user sees real-time progress; approves each phase gate.
AUTONOMOUS mode (Phase 4G): logged for post-run analysis; gates auto-advance.

Pipeline templates ranking (PART 22.2):
  score = first_try_ratio*0.4 + coverage*0.3 + (1-block_ratio)*0.2 + token_efficiency*0.1
  Storage: .elai_cc/templates/{type}/ (PipelineSpec + outcomes; versioned)
  Minimum 3 executions before ranking applied.

4. STATE MACHINES / LIFECYCLES
Plan Elaboration Pipeline (5 stages):
  Stage 1 — RESEARCH: Research Agent + GRC. Output: Research Report. >= 1 BENCHMARK or OFFICIAL_DOCS-grounded alternative per capability. All-TRAINING capabilities flagged HIGH_RISK -> human input in both modes.
  Stage 2 — PLAN ASSEMBLY: L1 Designer + GRC. Output: PipelineSpec with grounded steps. L1 can ONLY reference alternatives from Research Report. Receives SystemConstraints with model context budget + file size averages.
  Stage 3 — STRUCTURAL VALIDATION: L2, pure Rust. Tools exist, models in registry, no cycles, resources fit hardware. Includes Kahn's algorithm cycle detection BEFORE producing ExecutablePipeline. Cyclic -> ValidationError::CyclicDependency with cycle path.
  Stage 3.5 — PHASE SIZING (L2, deterministic, no LLM):
    phase_complexity = files_touched * avg_file_tokens
                     + test_count * avg_test_tokens
                     + contract_count * CONTRACT_OVERHEAD (default 200)
    available_for_work computed from context budget allocator formula.
    SPLIT_RATIO = 0.6
    Split trigger: phase_complexity > available_for_work * SPLIT_RATIO
    Split priority: contract boundaries (skeleton input/output contracts) -> directory proximity.
    Max split depth: 2 (matches Phase 4G SPLIT_TASK limit).
    Merge: adjacent phases only, no external dependents between them.
    preserve_contracts: sub-phases inherit parent's input/output contracts.
  Stage 4 — FEASIBILITY PROBE: empirical stub testing of novel steps. <50 line stubs. Max 3 alternatives per step before escalation.

Research Agent (PART 23):
  Role: researcher.
  Model: cloud preferred or Gemma4 + web search.
  GRC budget: high (10/1200).
  Process: decompose -> per-capability structured search -> rank by evidence quality.
  Output: structured Research Report (not prose).

Pipeline composition: complex tasks = multiple pipelines; each sub-pipeline has own FSM; results feed parent via structured handoff.

5. CONFIG SCHEMA EXCERPTS
elai_cc.toml sections referenced:
  [planning.sizing]            — bootstrap defaults: avg_file_tokens=150, avg_test_tokens=80
  [research.trusted_domains]   — overrides for source quality tiering
PipelineSpec storage: .elai_cc/pipelines/{id}/ (ExecutablePipeline)
Templates: .elai_cc/templates/{type}/ (PipelineSpec + outcomes; versioned)

Verbatim phase TOML (also appears in media-validator.txt §7.4):
  [[phases]]
  name = "build_landing_page"
  output_type = "web"
  visual_check = true

  [[phases]]
  name = "generate_character_model"
  output_type = "3d_model"
  visual_check = true

6. INTEGRATION POINTS
- orchestrator.txt §14 — Designer agent role definition; dispatch tables.
- orchestrator.txt §8 — Dynamic Fan-Out for parallel sub-questions.
- media-validator.txt §7 — MediaRenderAdapter consumed by L2 mapping (output_type -> adapter).
- rules engine — Self-Extending Rules path (auditor validates predicate proposals).
- SSB — research_step entries for INTERACTIVE progress and AUTONOMOUS post-analysis.
- knowledge engine — provides avg_file_tokens basis (when index has >=10 files).
- Phase 4G — SPLIT_TASK limit reused (depth 2).

7. PHASE DEPENDENCIES
Source: design_document.txt PART 15 (all), PART 23 (Research Agent), PART 25 (Feasibility Probe), PART 26 (Plan Elaboration), PART 29 (QualitySpec).
INTERACTIVE mode default in Phase 3.
AUTONOMOUS mode introduced in Phase 4G (also defines SPLIT_TASK limit reused by Phase Sizing).
Templates require minimum 3 executions before ranking applied (operational gating).
Cross-spec arxiv refs: 2508.05668, 2508.12752, 2506.18096 (deep research patterns), 2510.05145 (FlashResearch).

8. RISKS / OPEN QUESTIONS
- Several arxiv IDs (2508.05668, 2508.12752, 2510.05145) are post-cutoff; verify when grounding.
- "context budget allocator formula" referenced for available_for_work but not defined in this file.
- "ConvergenceStatus", "ResearchPlan", "SharedResearchState", "Alternative", "Contradiction", "SourceCitation", "ResearchTreeSnapshot", "NodeStatus" are referenced as types but only partially defined here.
- Verbatim phase TOML duplication with media-validator §7.4 — single source of truth needed.
- "Designer" role appears in both this file and prompt-architecture.txt §10; both reference orchestrator.txt §14.

9. TESTABLE INVARIANTS (research engine)
- Every node has bounded iterations (max_iterations enforced per node)
- Tree depth bounded by depth_limit (default 5, never exceeded)
- Tree breadth bounded by breadth_factor per node (default 3)
- Dual-threshold convergence: phi >= phi_min AND psi >= psi_min
- Source quality scores deterministic (domain lookup, no LLM)
- Research Report confidence = arithmetic from source data
- Contradictions surfaced, never silently resolved
- Total sources tracked (fetched vs used) for efficiency metrics
- Parallel sub-questions bounded by resource limits (local: sequential)
- Speculative execution disabled when local model
- Speculative execution bounded by speculative_depth * breadth_factor nodes
- Pruned speculative branches free resources immediately
- Research tree always acyclic (child never references ancestor)
- speculative_hit_rate tracked for adaptive tuning
- L1 Designer receives SystemConstraints with model_max_context
- avg_file_tokens defaults to 150 when index has <10 files
- Phase Sizing split trigger: phase_complexity > available_for_work * 0.6
- Phase Sizing max split depth: 2
- Phase Sizing preserves input/output contracts from skeleton
- Phase Sizing + SPLIT_TASK do not cascade
- Cycle detection runs before ExecutablePipeline production
- Cyclic phase dependency -> ValidationError::CyclicDependency (no deadlock)
- Designer is distinct AgentRole from Executor


================================================================================
FILE 5 — empire.txt
================================================================================

1. PURPOSE
Integrates the pre-existing "Empire" platform (D2 — deferred until D1 proven) with elai_cc. Empire provides IPC, hooks, buses, multi-brain isolation, FSB transport. elai_cc plays brain role and drives Empire via the existing FactoryLine protocol.

2. KEY TYPES & STRUCTS
Engine hierarchy (inheritance):
  EngineMKI   — IPC dispatch + hooks + buses + HTTP + events (core)
  EngineMKII  — +persistence (SQLite) + module management + webhooks + metrics
  EngineMKIII — +WS broadcast + WS client management
  Engine selected at build time via brainctl._write_main().

Two bus layers (both newline-delimited JSON over AF_UNIX SOCK_STREAM):
  IPC engine socket:        /tmp/brain-engine.sock   (module-to-engine)
  FactoryLine control sock: /tmp/brain-factory.sock  (factory-to-brain)

Hook system: HookRegistry — {agent: [hook]}, kinds: "route" | "event".
Bus system:  BusRegistry  — named buses with fnmatch pattern subscriptions.
Brain  = deployment unit (brain.json manifest).
Neuron = module registered to brain.
FSB binary transport: 12-byte header (frame_id:u32, part_idx:u16, part_total:u16, payload_len:u32). Max frame 256 KB.

Verified solid (leverage in D2):
  IPC layer (src/lib/ipc.py): pure stdlib, BUF_SIZE=4096, max 65536 bytes.
    Event format: {type, agent, channel, payload, timestamp, mac, ip}
  FactoryLine bus (src/core/bus.py): pure stdlib. Protocol version "v":1.
    Command format: {cmd: register|heartbeat|list|request_update, v, brain, dir, pid, ts}
    Response format: {ok: bool, ...}
  HookRegistry (hook_registry.py): route|event kinds, reserved paths, conflict detection.
  BusRegistry (bus_registry.py): named buses, fnmatch pattern subscriptions, dedup dispatch.
  Multi-brain isolation: tested (separate dbs/hooks/neurons/buses; 3 test classes).
  FSB binary transport: 12-byte header, 256KB max frame.
  SmartCatalog: recursive discovery, unlimited subcategory depth, manifest validation.
  Engine handler types (EngineMKI): REGISTER_HOOK, EMIT, QUERY_HOOKS, SUBSCRIBE,
    UNSUBSCRIBE, BUS_CREATE, BUS_JOIN, BUS_LEAVE, BUS_LIST.
    EngineMKII: + persistence/modules.
    EngineMKIII: + WebSocket broadcast via slc.SmartApiHub.

Verified broken (must fix in D2; verified 2026-04-08):
  - brainctl <-> brain_lifecycle circular dependency (lazy imports both directions)
  - EngineMKII hard-imports slc at module level (try/except fallback disables HTTP)
  - unique-factory/types directory shadows stdlib types module

Previously listed but INCORRECT (removed after source verification 2026-04-08):
  - _send_event(): send_event() in lib/ipc.py is fully implemented, not a stub
  - ECS dynamic imports: only fnmatch import in webhook dispatch, not ECS, not tight loop
  - workforce assembly lambda: None: confirmed stubs but dead code (see §6), not broken code

ECS / Workforce status (verified 2026-04-08):
  workforce/assembly/runtime/actions.py: 7 lines, all `lambda: None` placeholders. Dead stubs.
  brain.json has no schema validation. Nothing to adopt.
  PipelineSpec supersedes brain.json (6 layers of enforcement vs zero).

Previously listed dead but ALIVE (corrected 2026-04-08):
  MetricsStore: used in EngineMK0 and re-exported in EngineMKII. Not dead.
  SeedState: 7 phases (dormant->optimize), has hydrate/advance/can_grow. Actively used.

Self-improvement status (verified 2026-04-08):
  MetricsStore: basic counters; functional but no decision logic.
  SeedState: 7-phase state machine; functional lifecycle; no self-improvement capability.
  InternAgent: does not exist in codebase (removed after source verification).
  Direction: elai_cc brings self-improvement TO Empire, not the other way.

3. PROTOCOLS / WIRE FORMATS
Wire format choice: start with Option A (newline-JSON) for compatibility. Evaluate FSB/msgpack later.
FactoryLine command set: register | heartbeat | list | request_update.
FactoryLine command shape: {cmd, v, brain, dir, pid, ts}; response {ok: bool, ...}; protocol "v":1.
IPC engine event shape: {type, agent, channel, payload, timestamp, mac, ip}; BUF_SIZE 4096; max 65536 bytes.
FSB header (12 bytes): frame_id:u32, part_idx:u16, part_total:u16, payload_len:u32; max 256 KB frame.
Hook kinds: "route" | "event".
Subscription patterns: fnmatch (e.g. ["factory:*"]).

4. STATE MACHINES / LIFECYCLES
6-step integration plan (ordered by dependency):
  Step 1: elai_cc speaks FactoryLine protocol — Rust tokio UnixStream -> /tmp/brain-factory.sock; newline-JSON. Zero Empire changes.
  Step 2: elai_cc registers as a brain — generate brain.json manifest; register via FactoryLine; heartbeat monitoring.
  Step 3: elai_cc replaces factory-brain CLI — brainctl is broken (circular deps); orchestrator drives via FactoryLine; no engine changes.
  Step 4: elai_cc subscribes to Empire events — IPC SUBSCRIBE patterns ["factory:*"]; Empire becomes live GRC data source.
  Step 5: Empire uses elai_cc capabilities — elai_cc exposes MCP server; factory-brain registers elai_cc as neuron.
  Step 6: Fix Empire broken parts using elai_cc — circular deps, stubs, stdlib shadow generated and validated by elai_cc.

5. CONFIG SCHEMA EXCERPTS
brain.json — manifest for a brain (no schema validation in Empire; treat as opaque payload).
PipelineSpec supersedes brain.json semantically.

6. INTEGRATION POINTS
- orchestrator (Rust tokio): UnixStream client for /tmp/brain-factory.sock and /tmp/brain-engine.sock.
- knowledge engine: subscribes to factory:* events for live GRC data.
- MCP server (elai_cc-exposed): consumed by factory-brain when registering elai_cc as neuron.
- auditor + executor: drive fixes for circular deps, stubs, stdlib shadow during Step 6.
- Tool Auto-Discovery (pipeline-factory.txt §7): handles deploy options (SSH+rsync, OpenResty proxy, FastAPI HTTP, Docker, systemd are PRESENT in Empire; DNS, SSL/TLS, cloud APIs, CDN, load balancing, public routing are MISSING).

7. PHASE DEPENDENCIES
Implementation track: D2 — deferred until D1 proven.
Implementation phase: 4J (docs/specs/guide/steps/phase-4J.txt).
Source: design_document.txt PART 9 (all subsections); networking PART 9.6; ECS/workforce PART 9.7; self-improvement PART 9.8.
Verification snapshot date: 2026-04-08.

8. RISKS / OPEN QUESTIONS
- brainctl is broken at the time of integration; Step 3 must produce a CLI replacement before deployment workflows depend on it.
- EngineMKII module-level slc import may need patching alongside Step 3 to keep HTTP alive.
- unique-factory/types stdlib shadowing must be resolved early — risk of import-order surprises in any Python sub-process Empire spawns.
- FSB transport path is reserved as a future optimization (not adopted in initial integration).
- brain.json lacks schema validation; until PipelineSpec fully supersedes, manifest typos will silently mis-register brains.


================================================================================
INTEGRATION CROSS-MAP
================================================================================

Mapping each integration spec to the 9-crate architecture and phase map:

prompt-architecture.txt
  Crates touched: orchestrator (FSM, dispatch, prompt assembly, anti_spiral, AgentPool, session_init,
    SSB, prompt_transform, fan_out), rules (domain_detection, rules.db Tier 1/2/3),
    knowledge (KnowledgeEngine internal, retrieval+rerank), reasoning (GrcEngine), config (elai_cc.toml).
  Phase map:
    Phase 3F  — rules engine compiles all .md rule files into rules_seed.db
    Phase 4   — workflow integration (replaces all .md routes; planning-workflow, prompt-transform,
                rag-tools, session-protocol, multi-session, conditional-routing, output-formatting, CLAUDE.md)
  Existing files touched: .claude/rules/*, .claude/skills/*, CLAUDE.md, agent .md files (collapsed into
    [agents.{role}.role_prompt]), .elai_cc/sessions/{id}/.fsm-state.json (state load).

self-modification.txt
  Crates touched: orchestrator (SSB log access, reasoning_trace, anti-spiral signal),
    rules (Tier 1/2/3 management, counterfactual rule testing per rules.txt §5A),
    pipeline-factory (mod proposed as PipelineSpec; full 4-stage elaboration),
    auditor (safety-regression review), benchmark harness, sandbox manager,
    self_mod (variant archive, MSI computation, slot validator, proof gate driver),
    config (elai_cc.toml + sandbox copies).
  Phase map:
    Pre-req: orchestrator §7A (reasoning_trace), rules.txt §5A (counterfactual testing), Phase 3F (rules engine).
    Direct implementer: PART 22 + PART 27 of design_document.txt (no single phase number cited;
      depends on 4G AUTONOMOUS mode for unattended runs).
  Existing files touched: .elai_cc/global/self_mod_history.db, .elai_cc/sandbox/{mod_id}/,
    config_variants table; PreToolUse hook (READ-LOCK list).

media-validator.txt
  Crates touched: agents (media_validator role), pipeline-factory L2 (adapter mapping),
    orchestrator (Render-Capture-Evaluate sub-loop in EXECUTING; INTERACTIVE escalation),
    tools (Playwright MCP/CDP, Blender CLI, FFmpeg, ImageMagick, Tesseract, ComfyUI REST),
    model registry (Qwen3.5-VL-30B-A3B, Qwen3.5-VL-72B, InternVL3.5-38B, CLAP),
    constrained decoding (VlmValidationOutput).
  Phase map:
    Activated only when PipelineSpec contains media-producing steps.
    Phase 0 (RTX 3060 12GB): mock VLM only (MODE A cannot run).
    Source: PART 28 (all subsections).
  Existing files touched: PipelineSpec phase entries (output_type, visual_check),
    QualitySpec.MediaSpec, Tool Auto-Discovery registry.

pipeline-factory.txt
  Crates touched: pipeline-factory (L1 Designer + L2 Builder), orchestrator (L3 Executor, dispatch §14,
    Dynamic Fan-Out §8), research engine (ResearchTree, speculative execution, source tiering),
    knowledge (avg_file_tokens basis, repo map), rules (Self-Extending Rules; auditor validation),
    model registry (Capability Registry), tools (External Tool Auto-Discovery), SSB (research_step events),
    templates (.elai_cc/templates/{type}/), validation (Kahn cycle detection).
  Phase map:
    Phase 3 — INTERACTIVE mode default (per-phase user approval).
    Phase 4G — AUTONOMOUS mode + SPLIT_TASK depth limit reused by Phase Sizing.
    Source: PART 15, PART 23, PART 25, PART 26, PART 29.
  Existing files touched: .elai_cc/pipelines/{id}/skeleton.toml (per prompt-architecture §8),
    .elai_cc/pipelines/{id}/ (ExecutablePipeline output), .elai_cc/templates/{type}/,
    elai_cc.toml [planning.sizing] and [research.trusted_domains].

empire.txt
  Crates touched: orchestrator (tokio UnixStream client; brain registration; event subscription),
    MCP server (exposes elai_cc as neuron), knowledge (live factory:* GRC data ingest),
    auditor + executor (Step 6 fixes for circular deps, stub elimination, stdlib shadow removal),
    pipeline-factory Tool Auto-Discovery (deployment gap: DNS/SSL/cloud/CDN/LB/public-routing).
  Phase map:
    Track: D2 — deferred until D1 proven.
    Implementation phase: 4J (docs/specs/guide/steps/phase-4J.txt).
    Source: PART 9 (all subsections); networking PART 9.6; ECS/workforce PART 9.7; self-improvement PART 9.8.
  Existing files touched (Empire side, NOT .elai_cc/):
    src/lib/ipc.py (read-only contract)
    src/core/bus.py (read-only contract)
    hook_registry.py, bus_registry.py (read-only contract)
    brainctl._write_main() (replaced)
    EngineMKII slc import (patched)
    unique-factory/types (renamed to remove stdlib shadow)
    workforce/assembly/runtime/actions.py (left dead)
  Wire transport at start: newline-JSON; FSB/msgpack reserved for later evaluation.

Cross-spec OutputType single source of truth:
  pipeline-factory.txt §10 owns:
    OutputType enum { Web, Model3D, Audio, UiScreenshot, Animation, Image, Custom(String) }
  media-validator.txt consumes via PipelineSpec.output_type and adapter registry.

Cross-spec Designer role single source of truth:
  orchestrator.txt §14 owns the canonical AgentDispatchConfig for Designer.
  prompt-architecture.txt §10 and pipeline-factory.txt §2 both reference it.

Cross-spec context budget formula:
  available_for_work definition referenced in pipeline-factory.txt §3 ("context budget allocator formula")
  but the formula itself lives in orchestrator.txt §6 (per pipeline-factory comment).

Cross-spec arxiv reference list (gathered for grounding pass):
  2505.22954 — Darwin Godel Machine (self-modification)
  2604.01236 — DarwinNet PSI (cited for MSI in self-modification.txt) [FUTURE-DATED — verify]
  2403.03163 — Design2Code
  2507.22827 — ScreenCoder
  2401.13649 — VisualWebArena
  2508.05668, 2508.12752, 2506.18096 — Deep research patterns (pipeline-factory.txt) [POST-CUTOFF — verify]
  2510.05145 — FlashResearch (pipeline-factory.txt) [POST-CUTOFF — verify]
  IC3-Evolve sections 4, 10.1 (proof gate inspiration) — paper ID not given in self-modification.txt.
  FunSearch (DeepMind, Nature 2024) — referenced in self-modification.txt header.
