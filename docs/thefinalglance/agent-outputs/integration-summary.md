# INTEGRATION/ FOLDER — LOSSLESS AUDIT SUMMARY

Source root: `F:\.ELAI_workflow\docs\specs\integration\`
Files processed: 5 (`empire.txt`, `media-validator.txt`, `pipeline-factory.txt`, `prompt-architecture.txt`, `self-modification.txt`). One non-spec file `desktop.ini` ignored.

---

## 1. empire.txt

Path: `empire.txt`
Purpose: Integration of elai_cc with the existing Empire/Brain/FactoryLine system; deferred D2 deliverable, targeted by Phase 4J.
Source references: `design_document.txt` PART 9 (all subsections). Implementation target: `docs/specs/guide/steps/phase-4J.txt`.

Status markers:
- Explicitly marked `D2 — deferred until D1 proven`.
- Verified 2026-04-08 against actual Empire source.

Empire architecture (as verified from Empire source):
- Engine inheritance hierarchy: `EngineMKI -> EngineMKII -> EngineMKIII`.
  - MKI: IPC dispatch + hooks + buses + HTTP + events (core).
  - MKII: + persistence (SQLite) + module management + webhooks + metrics.
  - MKIII: + WS broadcast + WS client management.
  - Engine selected at build time via `brainctl._write_main()`.
- Two bus layers (both newline-delimited JSON over AF_UNIX SOCK_STREAM):
  - IPC engine socket: `/tmp/brain-engine.sock` (module-to-engine).
  - FactoryLine control socket: `/tmp/brain-factory.sock` (factory-to-brain).
- Hook system: `HookRegistry` — `{agent: [hook]}`, kinds: `"route" | "event"`.
- Bus system: `BusRegistry` — named buses with fnmatch pattern subscriptions.
- Deployment nouns: Brain = deployment unit (`brain.json` manifest). Neuron = module registered to brain.
- FSB binary transport: 12-byte header = `frame_id:u32, part_idx:u16, part_total:u16, payload_len:u32`.

What is broken (must fix in D2, verified 2026-04-08):
- `brainctl <-> brain_lifecycle` circular dependency (lazy imports in both directions).
- `EngineMKII` hard-imports `slc` at module level (`try/except` fallback disables HTTP).
- `unique-factory/types` directory shadows stdlib `types` module.

Previously listed as broken but corrected to NOT broken after source verification 2026-04-08:
- `_send_event()`: `send_event()` in `lib/ipc.py` is fully implemented, not a stub.
- ECS dynamic imports: only fnmatch import in webhook dispatch, not ECS, not tight loop.
- workforce assembly `lambda: None`: confirmed stubs, but dead code (section 6), not broken code.

What is solid (verified 2026-04-08):
- IPC layer (`src/lib/ipc.py`): pure stdlib, tested. `BUF_SIZE=4096`, max `65536` bytes.
  Event format: `{type, agent, channel, payload, timestamp, mac, ip}`.
- FactoryLine bus (`src/core/bus.py`): pure stdlib, tested. Protocol version `"v":1`.
  Command format: `{cmd: register|heartbeat|list|request_update, v, brain, dir, pid, ts}`.
  Response format: `{ok: bool, ...}`.
- `HookRegistry` (`hook_registry.py`): route|event kinds, reserved paths, conflict detection.
- `BusRegistry` (`bus_registry.py`): named buses, fnmatch pattern subscriptions, dedup dispatch.
- Multi-brain isolation tested (separate dbs, hooks, neurons, buses) — 3 test classes.
- FSB binary transport: 12-byte header, 256KB max frame.
- `SmartCatalog`: recursive discovery with unlimited subcategory depth, manifest validation.
- Engine handler types: `REGISTER_HOOK`, `EMIT`, `QUERY_HOOKS`, `SUBSCRIBE`, `UNSUBSCRIBE`, `BUS_CREATE`, `BUS_JOIN`, `BUS_LEAVE`, `BUS_LIST` (MKI). MKII adds persistence/modules. MKIII adds WebSocket broadcast via `slc.SmartApiHub`.

6-step integration plan (ordered by dependency):
1. elai_cc speaks FactoryLine protocol. Rust tokio `UnixStream -> /tmp/brain-factory.sock`. Newline-JSON commands. Zero Empire changes needed.
2. elai_cc registers as a brain. Generate `brain.json` manifest. Register via FactoryLine. Heartbeat monitoring.
3. elai_cc replaces `factory-brain` CLI (`brainctl` is broken due to circular deps). elai_cc orchestrator drives via FactoryLine. No change to engines.
4. elai_cc subscribes to Empire events. IPC `SUBSCRIBE` with patterns `["factory:*"]`. Empire becomes live GRC data source.
5. Empire uses elai_cc capabilities. elai_cc exposes MCP server. factory-brain registers elai_cc as neuron.
6. Fix Empire broken parts using elai_cc. Circular deps, stubs, stdlib shadow — elai_cc generates and validates fixes.

Wire format: start with Option A (newline-JSON) for compatibility. Evaluate FSB/msgpack later.

Networking (design PART 9.6):
- Empire HAS: SSH+rsync deploy, OpenResty proxy, FastAPI HTTP, Docker, systemd.
- Empire LACKS: DNS, SSL/TLS, cloud APIs, CDN, load balancing, public routing.
- Deploy = Tool Auto-Discovery problem. L1 detects hosting need, researches options.

ECS / Workforce (design PART 9.7, verified 2026-04-08):
- `workforce/assembly/runtime/actions.py`: 7 lines, all `lambda: None` placeholders. Dead stubs.
- `brain.json` has no schema validation. Nothing to adopt.
- `PipelineSpec` supersedes `brain.json` (6 layers of enforcement vs zero).
- Previously listed as dead but ALIVE (corrected 2026-04-08):
  - `MetricsStore`: used in `EngineMK0` and re-exported in `EngineMKII`. Not dead.
  - `SeedState`: 7 phases (dormant->optimize), has `hydrate/advance/can_grow`. Actively used.

Self-improvement (design PART 9.8, verified 2026-04-08):
- `MetricsStore`: basic counters, used in EngineMK0/MKII. Functional, no decision logic.
- `SeedState`: 7-phase state machine, functional lifecycle. No self-improvement capability.
- `InternAgent`: does NOT exist in codebase (removed after source verification 2026-04-08).
- elai_cc brings self-improvement TO Empire, not the other way.

Hardcoded paths / sockets / env / schemas in this file:
- `/tmp/brain-engine.sock`
- `/tmp/brain-factory.sock`
- `brain.json`
- `src/lib/ipc.py`, `src/core/bus.py`, `hook_registry.py`, `bus_registry.py`
- `workforce/assembly/runtime/actions.py`
- `unique-factory/types` (path shadow note)

Explicit numbers:
- IPC `BUF_SIZE = 4096`.
- IPC max payload `65536` bytes.
- FactoryLine protocol version `"v": 1`.
- FSB 12-byte header; 256KB max frame.
- 3 test classes for multi-brain isolation.
- 7-phase SeedState; 7-line dead actions file.

Dependencies on existing phases/crates:
- Implementation target phase: `Phase 4J` (`docs/specs/guide/steps/phase-4J.txt`).
- Blocking gate: deferred until D1 proven.
- Engineering: depends on `elai_cc` orchestrator and MCP server capability.

Deterministic checks / acceptance criteria / gates: none explicitly enumerated as invariants in this file; the "must fix" broken list and the 6-step plan are the implicit acceptance contract.

Verbatim quotes (non-obvious audit resolutions):
- "Previously listed but INCORRECT (removed after source verification 2026-04-08): `_send_event()`: `send_event()` in `lib/ipc.py` is fully implemented, not a stub"
- "Previously listed as dead but ALIVE (corrected 2026-04-08): `MetricsStore`: used in EngineMK0 and re-exported in EngineMKII. Not dead."
- "`InternAgent`: does not exist in codebase (removed after source verification 2026-04-08)."
- "elai_cc brings self-improvement TO Empire, not the other way."
- "Wire format: start with Option A (newline-JSON) for compatibility. Evaluate FSB/msgpack later."
- "PipelineSpec supersedes brain.json (6 layers of enforcement vs zero)."

---

## 2. media-validator.txt

Path: `media-validator.txt`
Purpose: Defines the media_validator agent — validates non-code outputs (images, 3D, audio, UI) via deterministic Tier 1 + optional VLM Tier 2, and the Render-Capture-Evaluate visual feedback architecture.
Source reference: `design_document.txt` PART 28 (all subsections).

Agent configuration:
- Role: `media_validator`.
- GRC budget: low (`3/600`).
- Permissions: `FILESYSTEM` (read-only on output files).
- Only loaded when `PipelineSpec` contains media-producing steps.

Tier 2 VLM model selection (two modes):
- MODE A — single image deep analysis:
  - Model: `Qwen3.5-VL-30B-A3B` (MoE), 8-bit precision.
  - Rationale: built-in Thinking Mode (generative CoT); `70-140 tok/s` at 8-bit.
  - Alternative (max intelligence): `Qwen3.5-VL-72B Instruct` at 4-bit EXL2/AWQ. Requires input image downscale.
- MODE B — sequential image flow (multiple smaller images):
  - Model: `InternVL3.5-38B`, 8-bit (or 4-bit for very long sequences).
  - Rationale: 5.5B parameter vision encoder; leaves `12-28GB VRAM` free for KV cache buffer.
  - Engine: MUST use `SGLang` (not vLLM/ollama). RadixAttention caches prior image visual state without re-processing or eviction.

Two-tier validation:
- TIER 1 — DETERMINISTIC (no model, `<100ms`): always runs first.
  - Image: format, resolution, file size, color space, SSIM/LPIPS against reference.
  - 3D model: vertex/face count, manifold check, UV overlap, normals, bounding box. Tools: Blender CLI headless mode.
  - Audio: sample rate, bit depth, duration, silence %, clipping detection.
  - UI screenshot: resolution, SSIM vs mockup, color histogram, OCR text verification.
- TIER 2 — VLM QUALITATIVE (model, `~2-5 seconds`):
  - Only when Tier 1 passes AND qualitative check specified in QualitySpec.
  - VLM output ADVISORY. Tier 1 has veto power.
  - VLM disagrees with Tier 1 -> flag for human review.
  - Audio: CLAP model `~0.8-1.0GB FP32, ~0.4GB FP16`. Use `Xenova/clap-htsat-unfused` ONNX export. Input: raw PCM (avoid mel-spectrogram BLAS variance from FFmpeg decoding).

GPU scheduling:
- Tier 1: CPU only. Zero GPU impact.
- Tier 2 MODE A (Qwen3.5-VL-30B-A3B MoE): active params ~3B (MoE) BUT all 30B stored in VRAM. 8-bit ~30GB. Q4_K_M ~17GB. Does NOT fit alongside other models on 24GB GPU. Swap: unload executor -> load VLM Q4 -> validate -> unload -> reload executor. `~10-15 seconds` at phase boundaries. Phase 0 (RTX 3060 12GB): cannot run, use mock VLM.
- Tier 2 MODE B (InternVL3.5-38B): full 38B model. Requires dedicated GPU. Unload executor/auditor first. SGLang engine manages VRAM via RadixAttention. Swap cost `~30-60 seconds`. Only justified for multi-image validation tasks.

External tools (each registered via Tool Auto-Discovery, pipeline-factory section 7): `ImageMagick`, `Blender CLI`, `FFmpeg`, `Tesseract`, `ComfyUI API`.

ComfyUI integration:
- REST API: `POST /prompt` (submit workflow), `GET /history` (results).
- Executor writes workflow JSON. Tool wrapper sends, polls, downloads.
- Timeout: `600s` for GPU rendering. Then Tier 1 + optional Tier 2 validation.

Render-Capture-Evaluate architecture (visual feedback loop) — section 7:

7.1 Adapter trait (contract):
```
trait MediaRenderAdapter {
    fn can_handle(&self, output_type: &OutputType) -> bool;
    async fn render(&self, artifact: &Path, config: &RenderConfig) -> Result<Vec<CapturedFrame>>;
    fn cleanup(&self);
}
```

Types introduced:
- `CapturedFrame { image_path: PathBuf, viewport: Viewport, metadata: FrameMetadata }`
- `Viewport { width: u32, height: u32, device: Option<String> }`
- `FrameMetadata { capture_time: Duration, adapter_name: String, state_description: String }`
- `RenderConfig` (referenced; accepts optional `Vec<Interaction>` in WebRenderAdapter for state capture).
- `VlmValidationOutput` (structured output: intent match, specific visual issues, severity Critical/Major/Minor).

7.2 Built-in adapters (each implements `MediaRenderAdapter`):
- `WebRenderAdapter`: Playwright headless Chromium/Firefox/WebKit. Process: spawn dev server -> navigate headless browser -> wait for network idle -> capture viewport screenshot(s) at multiple breakpoints (mobile `375px`, tablet `768px`, desktop `1440px`). Returns screenshots as `Vec<CapturedFrame>`. Integration: Playwright MCP (Microsoft) or direct CDP (Chrome DevTools Protocol). State capture also supports post-interaction snapshots via `Vec<Interaction>` in `RenderConfig`.
- `BlenderRenderAdapter`: Blender CLI (`--background --python`). Process: load model -> set camera angles (front/side/top/perspective) -> render. Supports `.obj`, `.fbx`, `.gltf`, `.blend`. Multi-angle: default 4 views (front, right, top, 3/4 perspective).
- `AnimationRenderAdapter`: FFmpeg. Render animation -> extract key frames at configured intervals (every 1s, or at 0%, 25%, 50%, 75%, 100%). Returns key frames with timing metadata. Video: extract frames. GIF/Lottie: render to frames first.
- `UiScreenshotAdapter`: direct file read. Validates image format -> optionally resize -> returns `CapturedFrame`.
- `ComfyUIAdapter`: ComfyUI REST API. `POST /prompt`, poll `/history`, download. Timeout `600s`.

7.3 Render-Capture-Evaluate loop (7 steps, runs INSIDE EXECUTING as sub-loop, NOT a new FSM state):
1. EXECUTE: agent produces output.
2. RENDER: L2 pipeline builder maps `PipelineSpec.output_type -> adapter` via Tool Auto-Discovery.
3. CAPTURE: screenshots/renders collected as `Vec<CapturedFrame>`.
4. TIER 1 EVALUATE: deterministic checks (resolution, format, SSIM vs reference, color histogram).
5. TIER 2 EVALUATE: VLM receives frames + QualitySpec + original intent. Produces `VlmValidationOutput` with severity classification.
6. FEEDBACK: specific, actionable, localized issues fed to executor.
7. ITERATE: executor revises -> go to step 1. Max iterations: configurable default `3`. After max: escalate to INTERACTIVE gate. Convergence check: if Tier 2 issues unchanged between iterations, stop (human review needed).

Quote: "This loop runs INSIDE the EXECUTING phase, between the executor agent's work and the evidence gate check. It is NOT a new FSM state — it is a sub-loop within execution, like the anti-spiral check or the GRC chain."

7.4 Scope universality (PipelineSpec TOML examples):
```
[[phases]]
name = "build_landing_page"
output_type = "web"          # -> WebRenderAdapter
visual_check = true

[[phases]]
name = "generate_character_model"
output_type = "3d_model"     # -> BlenderRenderAdapter
visual_check = true
```
- L2 pipeline builder maps `output_type -> adapter` at plan assembly time.
- If adapter tool unavailable: `visual_check=true but adapter unavailable -> DEGRADED validation (Tier 1 only)`.
- If `output_type` has no adapter: L2 rejects the PipelineSpec step ("cannot validate what you cannot see").
- Adding an adapter: implement trait -> register in adapter registry with `output_type` key -> ensure render tool is in Tool Auto-Discovery.

7.5 Process integration: visual check triggered by `visual_check=true`; runs automatically. For non-visual outputs: `visual_check=false`, loop never activates, zero overhead. Mixed pipelines: per-phase toggling (backend off, frontend on).

7.6 Prior art (grounding):
- Design2Code (`arxiv 2403.03163`): 484 real webpages; CLIP, CW-SSIM, block-matching.
- ScreenCoder (`arxiv 2507.22827`): modular multi-agent (grounding, planning, generation).
- SeeAct (ICML 2024): generalist web agent using screenshots.
- VisualWebArena (`arxiv 2401.13649`): multimodal web agents with Set-of-Mark prompting. VLM still far below human: `16.4% vs 88.7%`.
- Tweag Agentic Coding Handbook: "Visual Feedback Loop".
- Playwright visual regression testing: pixel-diff with configurable threshold.
- Stated gap: "None implement a fully autonomous render-capture-evaluate loop inside a pipeline execution engine. This is novel integration, not novel research — all components exist, the composition is new."

Testable invariants (section 8):
- Tier 1 always before Tier 2.
- Tier 1 fail overrides Tier 2 pass.
- VLM output structured (constrained decoding).
- Only loaded when PipelineSpec has media steps.
- SSIM/LPIPS scores deterministic.
- Render-capture-evaluate loop max iterations enforced (default 3).
- Convergence check stops iteration when issues unchanged.
- Adapter selection deterministic (`output_type -> adapter`, no LLM).
- `visual_check=false -> zero render overhead` (adapter never instantiated).
- Missing adapter tool -> DEGRADED (not FAIL, not silent skip).

Integration dependencies:
- Depends on Tool Auto-Discovery (pipeline-factory.txt section 7).
- Depends on `PipelineSpec.output_type` and per-phase `visual_check` field (see pipeline-factory.txt section 10).
- Loop runs INSIDE EXECUTING phase; integrates with anti-spiral and GRC chain as a peer sub-loop.

Explicit numbers summary (this file):
- GRC budget: 3/600.
- Tier 1 latency: <100ms.
- Tier 2 latency: 2-5s.
- Qwen3.5-VL-30B-A3B: 70-140 tok/s; 8-bit ~30GB, Q4_K_M ~17GB.
- InternVL3.5-38B: 5.5B vision encoder params, frees 12-28GB VRAM for KV cache.
- VLM swap Mode A: 10-15s; Mode B: 30-60s.
- CLAP model: ~0.8-1.0GB FP32, ~0.4GB FP16.
- ComfyUI timeout: 600s.
- Web breakpoints: 375/768/1440 px.
- Default Blender views: 4 angles.
- Animation frame intervals: 0/25/50/75/100%.
- Max render-evaluate iterations default: 3.
- Phase 0 HW: RTX 3060 12GB (cannot run Tier 2 Mode A).

---

## 3. pipeline-factory.txt

Path: `pipeline-factory.txt`
Purpose: The three-layer pipeline factory (Designer / Builder / Executor) that lets elai_cc accept any goal, DESIGN a pipeline, BUILD it, and EXECUTE it. Also defines the Deep Research engine, QualitySpec, model registry, tool auto-discovery, self-extending rules, composition, and templates.
Source references: `design_document.txt` PART 15 (all), PART 23 (Research Agent), PART 25 (Feasibility Probe), PART 26 (Plan Elaboration), PART 29 (QualitySpec).

Three-layer architecture:

- L1 — PIPELINE DESIGNER (LLM-driven, GRC-grounded):
  - Agent role: `Designer` (distinct from Executor — see `orchestrator.txt S14`).
  - Input: human task description + `SystemConstraints` (`model_max_context`, `available_for_work`, `avg_file_tokens`, `avg_test_tokens`).
  - L1 Observe step text: "Context budget: {available_for_work} tokens. Each file averages {avg_file_tokens} tokens. Phase must touch at most `floor(available_for_work * 0.4 / avg_file_tokens)` files."
  - Bootstrap: when knowledge index has `<10 files`, `avg_file_tokens` defaults `150`, `avg_test_tokens` defaults `80` (from `elai_cc.toml [planning.sizing]`).
  - GRC decomposes into sub-questions; all from VERIFIED sources. Missing rule set triggers research-propose-audit-install flow.
  - Output: `PipelineSpec` (TOML/msgpack) + `QualitySpec`.

- L2 — PIPELINE BUILDER (deterministic, no LLM):
  - Input: `PipelineSpec`.
  - Validates components exist, creates FSM states, configures agents, installs rules, estimates resources, generates dependency graph, creates test harness skeleton.
  - Output: `ExecutablePipeline` in `.elai_cc/pipelines/{id}/`.

- L3 — PIPELINE EXECUTOR: existing FSM + agents + rules. Runs INTERACTIVE or AUTONOMOUS.

5-stage Plan Elaboration pipeline:
- Stage 1 — RESEARCH: Research Agent + GRC. Output: structured Research Report. `>=1 BENCHMARK or OFFICIAL_DOCS` grounded alternative per capability. All-TRAINING capabilities flagged HIGH_RISK (human input required in both modes).
- Stage 2 — PLAN ASSEMBLY: L1 Designer + GRC. Output: PipelineSpec with grounded steps. L1 can ONLY reference alternatives from Research Report. L1 receives `SystemConstraints`.
- Stage 3 — STRUCTURAL VALIDATION: L2, pure Rust, deterministic. Tools exist, models in registry, no cycles, resources fit hardware. Kahn's algorithm cycle detection before ExecutablePipeline. Cycle -> `ValidationError::CyclicDependency` with cycle path.
- Stage 3.5 — PHASE SIZING (L2, deterministic, no LLM):
  - Formula:
    ```
    phase_complexity = files_touched * avg_file_tokens
                     + test_count * avg_test_tokens
                     + contract_count * CONTRACT_OVERHEAD   // default 200
    ```
  - `available_for_work` from context budget allocator.
  - Split trigger: `phase_complexity > available_for_work * SPLIT_RATIO` (default `0.6`).
  - Split by contract boundaries (skeleton input/output contracts), then directory proximity.
  - Max split depth: `2` (matches Phase 4G SPLIT_TASK limit).
  - Merge: adjacent phases only, no external dependents between them.
  - `preserve_contracts`: sub-phases inherit parent's input/output contracts.
- Stage 4 — FEASIBILITY PROBE: empirical stub testing of novel steps. `<50` line stubs, max `3` alternatives per step before escalation.

Research Agent (design PART 23):
- Role: `researcher`. Model: cloud preferred or Gemma4 + web search.
- GRC budget: high (`10/1200`).
- Process: decompose -> per-capability structured search -> rank by evidence quality.
- Evidence levels (ranked): `BENCHMARK > OFFICIAL_DOCS > BLOG_POST > TRAINING`.
- Output: structured Research Report (not prose).

Tree-structured research architecture (Deep Research Engine), verified from `arxiv 2508.05668, 2508.12752, 2506.18096` and tree/adaptive per `arxiv 2510.05145` (FlashResearch):

Types:
```
struct ResearchTree {
    plan: ResearchPlan,
    state: SharedResearchState,
    root: ResearchNode,
    breadth_factor: u32,       // default 3
    depth_limit: u32,          // default 5
    max_iterations: u32,       // default 10
    phi_min: f64,              // default 0.7
    psi_min: f64,              // default 0.6
    speculative_depth: u32,    // default 2
}
ResearchNode {
    question: String,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    phi: f64,
    psi: f64,
    status: NodeStatus,        // Pending | Active | Converged | Pruned
}
```

Per-node research loop:
1. FORMULATE (GRC query from sub-question + prior findings).
2. SEARCH (top N, default `10`).
3. EXTRACT (fetch, sampling-based relevance: read first `500 tokens`, score, read full if above threshold).
4. REFLECT (GRC evaluates `phi`, `psi`, contradictions, new sub-questions -> spawn children bounded by `breadth_factor`, information gain).
5. CONVERGE / EXPAND / ITERATE:
   - `phi >= phi_min AND psi >= psi_min` -> converged.
   - `phi < phi_min` -> iterate or spawn children.
   - Contradictions -> disambiguation child node.
   - `depth >= depth_limit` -> stop, report partial w/ confidence.
   - `max_iterations` -> report partial w/ confidence.

Speculative execution (FlashResearch-derived):
- Trigger: `phi > 0.4` suggests likely children.
- Bound: `speculative_depth * breadth_factor` nodes.
- Benefit: "30-50% wall-clock reduction on deep research queries (FlashResearch benchmark)."
- Local models: speculative DISABLED (single GPU, sequential).
- Cloud APIs: ENABLED, rate-limit aware.

Adaptive resource allocation (`breadth_factor`, `depth_limit` by query complexity, assessed by GRC in PLANNING observe step):
- Simple query: breadth=2, depth=3.
- Moderate query: breadth=3, depth=4.
- Complex query: breadth=4, depth=5.

Parallel sub-question processing (via Dynamic Fan-Out, `orchestrator.txt section 8`):
- Root children: parallel (independent sub-questions).
- Speculative children: parallel with parent (cloud API only).
- Local models: sequential within each tree level.
- Cloud APIs: parallel across breadth, rate-limit aware.
- Deferred aggregation node collects all sub-results after tree converges.
- Pruned branches: resources freed immediately.

Source quality tiering (deterministic, no LLM):
- TIER_1 (1.0): arxiv papers, official docs, RFC.
- TIER_2 (0.8): GitHub repos >100 stars, StackOverflow accepted answers.
- TIER_3 (0.6): blog posts from known technical authors, conference talks.
- TIER_4 (0.4): general blog posts, forum discussions.
- TIER_5 (0.2): social media, unverified sources.
- Assignment via domain-based lookup + heuristic rules.
- Configurable per project in `elai_cc.toml [research.trusted_domains]`.
- Ranking: `evidence_level * source_quality_avg` (pure arithmetic).

Structured research output types:
```
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
    sources: Vec<SourceCitation>,      // URL + extracted quote + quality tier
    confidence: f64,                   // source count * quality * agreement
    phi: f64,
    psi: f64,
    depth: u32,
    alternatives: Vec<Alternative>,
    contradictions: Vec<Contradiction>,
}
```

Research progress tracking:
- Every search iteration writes to SSB: `action_type='research_step'`, payload `{ sub_question, iteration, urls_fetched, findings_count, info_gain }`.
- INTERACTIVE mode (default, Phase 3): real-time progress, user approves each phase gate.
- AUTONOMOUS mode (Phase 4G): logged for post-run analysis, gates auto-advance.

QualitySpec (design PART 29):
Types:
- `QualityMetric { name, target, unit, method, phase, hard_fail }`
- `MediaSpec { output_name, format, resolution, ssim_threshold, vlm_check, output_type: OutputType }`
- `OutputType enum { Web, Model3D, Audio, UiScreenshot, Animation, Image, Custom(String) }`
- `PerfSpec { name, target, unit, method, sample_count }`
- L2 validates every metric has automated check method. Unmeasurable -> REJECT.
- Per-phase `visual_check: bool` (default `false`). When true AND phase has MediaSpec, render-capture-evaluate loop activates during EXECUTING.
- L2 validates: `visual_check=true` requires a `MediaRenderAdapter` for the phase's `output_type`. Missing adapter -> `ValidationError` (not silent skip).
- Cross-link: see `media-validator.txt section 7` for full Render-Capture-Evaluate architecture.

Completeness `QualityMetric` examples (proactive thinking, Concern 3):
```
{ name: "image_extraction", target: 100, unit: "%",
  method: "grep_output", phase: "EXECUTING", hard_fail: false }
{ name: "table_extraction", target: 100, unit: "%",
  method: "grep_output", phase: "EXECUTING", hard_fail: false }
{ name: "modality_coverage", target: 100, unit: "%",
  method: "count_modalities", phase: "AUDITING", hard_fail: true }
```
Methods: `grep_output` scans tool results; `count_modalities` checks scope enumeration.

Model Capability Registry (design PART 15.6):
- Per-model: capabilities, `vram_required`, `max_context`, strengths, weaknesses, provider.
- Selection: `score = sum(capability_match) * context_fit * vram_feasibility` — pure numeric, no LLM judgment.
- Extension: L1 queries `ollama /api/tags`, runs capability probe, registers.

External Tool Auto-Discovery (design PART 15.7):
- Three levels: `AVAILABLE` (on PATH), `INSTALLABLE` (package manager), `UNAVAILABLE`.
- Tool wrapper contract: `execute(args) -> ToolResult`, `validate_output() -> bool`, `version() -> SemVer`.
- Discovery: query PATH, package managers, web search. Auditor reviews proposals.

Self-extending rules (design PART 15.2):
- L1 creates new rule categories at runtime.
- Proposed as Rust-evaluable predicates. Auditor validates.
- Rule templates shipped: `SaaS, Trading, Game, Data, Security, ML, Custom`.

Pipeline composition:
- Complex tasks = multiple pipelines.
- Each sub-pipeline has own FSM. Results feed parent via structured handoff.

Pipeline templates:
- Completed pipelines become reusable templates.
- Stored in `.elai_cc/templates/{type}/` as PipelineSpec + outcomes.
- Versioned; improvements propagate. Minimum `3 executions` before ranking applied.
- Template ranking formula (PART 22.2):
  `score = first_try_ratio*0.4 + coverage*0.3 + (1-block_ratio)*0.2 + token_efficiency*0.1`.

Testable invariants (research engine, section 16):
- Every node has bounded iterations (`max_iterations` enforced per node).
- Tree depth bounded by `depth_limit` (default 5).
- Tree breadth bounded by `breadth_factor` per node (default 3).
- Dual-threshold convergence: node converged only when `phi >= phi_min AND psi >= psi_min`.
- Source quality scores deterministic (domain lookup, no LLM).
- Research Report confidence = arithmetic from source data.
- Contradictions surfaced, never silently resolved.
- Total sources tracked (fetched vs used) for efficiency metrics.
- Parallel sub-questions bounded by resource limits (local: sequential).
- Speculative execution disabled when local model (single GPU).
- Speculative execution bounded by `speculative_depth * breadth_factor` nodes.
- Pruned speculative branches free resources immediately.
- Research tree always acyclic (child never references ancestor).
- `speculative_hit_rate` tracked for adaptive tuning.
- L1 Designer receives `SystemConstraints` with `model_max_context`.
- `avg_file_tokens` defaults to 150 when index has <10 files.
- Phase Sizing split trigger: `phase_complexity > available_for_work * 0.6`.
- Phase Sizing max split depth: 2.
- Phase Sizing preserves input/output contracts from skeleton.
- Phase Sizing + SPLIT_TASK do not cascade.
- Cycle detection runs before `ExecutablePipeline` production.
- Cyclic phase dependency -> `ValidationError::CyclicDependency` (no deadlock).
- Designer is distinct `AgentRole` from Executor.

Hardcoded paths, env vars, config keys, TOML sections:
- `.elai_cc/pipelines/{id}/` (L2 output)
- `.elai_cc/templates/{type}/` (template storage)
- `elai_cc.toml [planning.sizing]` (avg_file_tokens, avg_test_tokens defaults)
- `elai_cc.toml [research.trusted_domains]` (per-project overrides)
- `ollama /api/tags` (capability probe endpoint)
- PipelineSpec TOML field: `[[phases]] name, output_type, visual_check`

Explicit numbers (this file):
- Bootstrap defaults: avg_file_tokens=150, avg_test_tokens=80.
- `CONTRACT_OVERHEAD` default=200.
- `SPLIT_RATIO` default=0.6.
- Max split depth=2.
- Feasibility probe stubs `<50` lines, max 3 alternatives.
- Research GRC budget 10/1200.
- breadth_factor=3, depth_limit=5, max_iterations=10.
- phi_min=0.7, psi_min=0.6, speculative_depth=2.
- Speculative trigger threshold phi>0.4.
- Speculative wall-clock reduction: 30-50%.
- Adaptive ranges simple 2/3, moderate 3/4, complex 4/5.
- Source tiers 1.0/0.8/0.6/0.4/0.2.
- L1 observe formula: `floor(available_for_work * 0.4 / avg_file_tokens)`.
- Minimum template executions for ranking=3.
- Template ranking weights: 0.4/0.3/0.2/0.1.

Integration dependencies on existing phases/crates/files:
- `orchestrator.txt S14` (Designer role, deterministic dispatch, dynamic fan-out, context budget allocator).
- `media-validator.txt section 7` (Render-Capture-Evaluate).
- Phase 4G SPLIT_TASK (cited for max split depth=2 alignment).
- Phase 3 (INTERACTIVE mode default).
- Phase 4G (AUTONOMOUS mode).

Status markers:
- L3 labelled "existing FSM + agents + rules".
- All other sections are design; Phase Sizing labelled `(L2, deterministic, no LLM)`.

Verbatim quote (key design):
- "elai_cc is a universal automation engine, not a code assistant."
- "L1 can ONLY reference alternatives from Research Report."
- "No LLM judgment — pure numeric ranking."

---

## 4. prompt-architecture.txt

Path: `prompt-architecture.txt`
Purpose: Explains how elai_cc eliminates prompt-driven orchestration by moving workflow control into Rust code; specifies model prompt composition and what replaces every current `.md` file.
References: `orchestrator.txt`, `rules.txt`, `knowledge.txt`, `providers.txt`.

Problem framing (section 1) — quantified costs of current workflow:
- 4 rule files (~60 lines) every turn.
- 3 skill files (~140 lines) on trigger.
- `CLAUDE.md` (~120 lines) every turn.
- Agent `.md` files (~30-50 lines) per spawn.
- Domain rule files (~100-160 lines) conditional.
- Total: `500-800 tokens` of orchestration instructions per turn, before task context.
- Local models have `8-32k` context -> `2-10%` burned on meta-instructions.

Solution (section 2): "CODE IS THE ORCHESTRATOR". Explicit list of what the model NEVER does (9 bullets) and what it ONLY does (4 bullets). Key negatives:
- Never decides FSM state (orchestrator).
- Never spawns agents (orchestrator).
- Never loads rule files (rules engine queries `rules.db`, injects relevant).
- Never manages sessions or file locks (SSB).
- Never decides context to load (knowledge engine).
- Never calls RAG tools (knowledge engine is internal).
- Never calls LLM provider directly (orchestrator proxies via IPC).
- Never executes tools directly (proxied via IPC).
- No network access except researcher role.
- Never sees enforcement logic, rules DB, harness internals.
- Never sees internal tool parameters (`_rule_context`, `_enforcement_tier`, etc.); tool schemas sanitized (swarm-derived pattern).

Agent Prompt Construction (4 sources, section 3):

SOURCE 1 — ROLE PROMPT (from `elai_cc.toml`, ~50-100 tokens):
TOML keys introduced:
```
[agents.executor]        role_prompt = "..."
[agents.auditor]         role_prompt = "..."
[agents.prompt_engineer] role_prompt = "..."
[agents.context_compiler] role_prompt = "..."
[agents.memory_writer]   role_prompt = "..."
```
Verbatim `executor`: "You write code to pass failing tests. Output: one code block per file, prefixed with file path. Do not explain. Do not add features beyond what tests require."
Verbatim `auditor`: "You are an adversarial code reviewer. Every finding must cite line number and evidence. You must find at least one issue per review."
Verbatim `prompt_engineer`: "Transform the raw task into a structured prompt. Output: XML with instructions, context, task, output_format blocks."
Verbatim `context_compiler`: "Gather relevant code context for the given task. Output: structured context package with blast radius and interfaces."
Verbatim `memory_writer`: "Record changes from this phase into the knowledge base. Output: structured change manifest."

SOURCE 2 — ACTIVE TIER 3 RULES (from `rules.db`, ~100-300 tokens). SQL used:
```
SELECT description FROM rules
WHERE status = 'ACTIVE' AND tier = 3
  AND domain IN (current_domains)
  AND (phase_mask & current_phase_bit) != 0
ORDER BY severity DESC
LIMIT 10
```
Tier 1 and Tier 2 rules are INVISIBLE to the model — enforced by code. Only 5-10 Tier 3 rules injected, not the full 160-line domain rule file.

SOURCE 3 — CODE CONTEXT (knowledge engine, budget-fitted). PageRank repo map + BM25/vector retrieved chunks + cross-encoder reranked. Fitted to budget by context budget allocator (`orchestrator.txt section 6`).

SOURCE 4 — TASK SPEC (plan skeleton + phase detail).

Assembled prompt breakdown (target ~2500-9000 tokens total, mostly code):
- role_prompt ~50-100
- tier_3_rules ~100-300
- code_context ~2000-8000 (budget-fitted)
- task_spec ~200-500
- output_format + constraints ~50-100
- TOTAL INSTRUCTIONS: ~200-500 tokens (vs 500-800 current).

Section 4 — Per-file replacement map:
- 4.1 `session-protocol.md` -> `orchestrator::session_init()`. Steps: SSB query active sessions -> load FSM state from `.elai_cc/sessions/{id}/.fsm-state.json` -> on resume, load plan skeleton, determine phase -> spawn context-compiler -> wait -> spawn prompt-engineer -> present prompt, gate on user approval. "The model is NEVER told to 'spawn' anything. The orchestrator spawns."
- 4.2 `multi-session.md` -> orchestrator SSB + session locks. Automatic `SSB.claim()` on EXECUTING entry; `SSB.check_conflicts()` before any write; auditor `SSB.attribute_errors()` automatically; `SSB.release()` on phase completion. Writes violating locks BLOCKED by PostToolUse hook.
- 4.3 `conditional-routing.md` -> `rules::domain_detection()` Rust function:
  ```
  fn detect_domains(file_paths: &[&str]) -> Vec<Domain> {
      // deterministic file-path-to-domain mapping
      // configured in elai_cc.toml [rules.domain_mapping]
  }
  ```
- 4.4 `output-formatting.md` -> split. (a) "Zero filler..." becomes part of `role_prompt`; (b) "Update memory-bank..." becomes `orchestrator::memorize()` in MEMORIZING phase.
- 4.5 `planning-workflow SKILL.md` -> orchestrator crate FSM engine (states/transitions, evidence gates as Rust functions, `AgentPool`, audit protocol, memorizing, handoff via `handoff.msgpack`).
- 4.6 `prompt-transform SKILL.md` -> 3-line Rust:
  ```
  fn prompt_transform(task: &str, session: &Session) -> TransformedPrompt {
      let context = self.context_compiler.compile(task, session);
      self.prompt_engineer.transform(task, &context)
  }
  ```
- 4.7 `rag-tools SKILL.md` -> `knowledge::KnowledgeEngine` (GONE). No `mcp_search_context`, no `mcp_memorize`, no `mcp_ingest_file`. Orchestrator calls `KnowledgeEngine::query()` directly.
- 4.8 `CLAUDE.md` -> `elai_cc.toml` + hardcoded Rust. Mappings:
  - "FSM gate check" -> orchestrator FSM.
  - "Anti-Slopsquatting" -> `rules.db` Tier 1 GREP rule.
  - "Zero Lazy Code" -> `rules.db` Tier 1 AST rule.
  - "Anti-Spiral" -> `orchestrator::anti_spiral()` Rust function.
  - "GRC tools" -> `reasoning::GrcEngine`, not model-invoked tools.
  - "Context delegation" -> orchestrator prompt assembly.
  - "Output formatting" -> `role_prompt` in `elai_cc.toml`.
  - Config-only remainders: agent model assignments, GRC budget defaults, domain mapping overrides, hardware config.
- 4.9 Agent `.md` files -> `role_prompt` in `elai_cc.toml`.

Enforcement comparison (section 5):
- Current: 4 points of failure; probability `p1*p2*p3*p4 < 1.0`.
- elai_cc Tier 1/2: 0 points of failure; "Probability of enforcement: 1.0 (deterministic code, not probabilistic prompt)". Steps: orchestrator detects domain from file paths (deterministic) -> rules engine loads active Tier 1 predicates (SQLite query) -> model produces output (unconstrained) -> PostToolUse hook evaluates predicates in Rust `<1ms` -> BLOCK -> output rejected, model re-prompted.
- Tier 3: inject 5-10 relevant rules (~100 tokens); auditor reviews; BLOCK findings rejected. Self-discovery: repeated Tier 3 violations -> proposed Tier 1/2 rule (`rules.txt section 6`).

Context budget impact (section 6) — hard numbers:
- Current per-turn: CLAUDE.md `~800`, 4 rule files `~400`, skill file `~600` on trigger, domain rules `~500` conditional, agent md `~300`. Total meta `~1600-2600 per turn`.
- elai_cc per-turn: role_prompt `~80`, Tier 3 rules `~200`, output format `~50`. Total meta `~330 tokens per turn`.
- Savings: `1300-2300 tokens/turn`. 32k context: +4-7% for code. 8k context: +16-29% for code (critical).
- Cognitive isolation (Tier C): formula `context_savings_from_isolation = sum(agent_grc_tokens) / model.max_context`.
  - 8k local: if agents generate 12k tokens internal reasoning -> saves 150% of context window (makes task POSSIBLE).
  - 32k cloud: ~30-40%.
  - Cross-link: `orchestrator.txt section 7A`.

Section 7 — literal sample prompts for EXECUTOR and AUDITOR (quoted in file), confirming: no FSM instructions, no session management, no "spawn context-compiler", no "read tradingRules.md".

Remaining natural-language config (section 8):
- ROLE PROMPTS (50-100 tokens each in `[agents.{role}.role_prompt]`). Editable. Self-modification Layer 2 target.
- TIER 3 RULE DESCRIPTIONS in `rules.db` `description TEXT`. Self-discovery target.
- PLAN SKELETON in `.elai_cc/pipelines/{id}/skeleton.toml`.

Migration path (section 9):
- Phase 3F (rules engine) compiles all `.md` rule files into `rules_seed.db`.
- Phase 4 (workflow integration) replaces skills/rules/CLAUDE.md per the mapping above.
- After Phase 4: zero `.md` files in the orchestration path.

Deterministic Dispatch Protocol (section 10) — full spec at `orchestrator.txt section 14`:
1. User types task in CLI (`/task`, `/config`, `/fix`, etc.).
2. CLI strips prefix, sets `initial_route` in `TaskPacket`.
3. Orchestrator reads FSM state. Selects dispatch table:
   - PLANNING/CONFIGURING: `MetaDispatchTable` (static, hardcoded).
   - All others: `PlanDispatchTable` (dynamic, from `PipelineSpec`).
4. Dispatch table returns `DispatchStage` or `DispatchPipeline` (multi-stage).
5. For each stage: `skip_condition` check -> prompt assembly -> agent spawn -> gate.
6. Prompt assembly from 4 sources. Total meta-instructions: `~330 tokens`.
7. Agent works via IPC. Returns `AgentResponse`.
8. Evidence gate -> FSM transition -> `phase_complete` SSB record -> model swap if needed.

Key types (all defined in `orchestrator.txt S14`): `MetaDispatchTable`, `PlanDispatchTable`, `DispatchPipeline`, `DispatchStage`, `AgentDispatchConfig`, `ToolFilter`, `ArtifactType`, `GateCondition`, `SkipCondition`.

Designer role (NEW): produces PipelineSpec from Research Report. Distinct from Executor. Runs in Stage 2 of PLANNING DispatchPipeline. GRC budget `8/1200`. Full def: `orchestrator.txt S14`.

Testable invariants (section 11):
- Model prompt contains zero orchestration instructions (no FSM, no spawn, no routing).
- Total meta-instruction tokens `<500` per turn (role + Tier 3 + format).
- Tier 1/2 rules never appear in model prompt.
- Only Tier 3 rules for current domain + phase injected.
- Domain detection is deterministic (same file paths -> same domains).
- `role_prompt` round-trips through `elai_cc.toml` (serialize -> deserialize -> identical).
- Prompt assembly completes in `<5ms` (no file I/O, all from memory/SQLite).

Integration dependencies on existing phases/crates:
- `knowledge` crate (`KnowledgeEngine`, PageRank repo map, BM25/vector, cross-encoder rerank).
- `rules` crate (`rules.db`, `domain_detection()`, PostToolUse hook).
- `orchestrator` crate (`session_init`, `prompt_transform`, `memorize`, `AgentPool`, `anti_spiral`, context budget allocator).
- `reasoning` crate (`GrcEngine`).
- `providers` crate (LLM proxy via IPC).
- Phase 3F (rules engine build of `rules_seed.db`).
- Phase 4 (workflow integration — full replacement).

Status markers: section 9 explicitly treats this as a migration plan; Designer role flagged "(NEW)".

Hardcoded paths / config keys: `elai_cc.toml`, `.elai_cc/sessions/{id}/.fsm-state.json`, `.elai_cc/pipelines/{id}/skeleton.toml`, `rules.db`, `rules_seed.db`, `handoff.msgpack`, `[agents.{role}.role_prompt]`, `[rules.domain_mapping]`.

---

## 5. self-modification.txt

Path: `self-modification.txt`
Purpose: Defines a three-layer self-modification system (Data / Config / Structure) for elai_cc — adopts DGM, FunSearch, IC3-Evolve, DarwinNet patterns while keeping the foundation model frozen. Bounds evaluation with staged benchmarks, proof gates, slot restrictions, anti-reward-hacking locks, and a Modification Solidification Index.
Source references: `design_document.txt` PART 22 (self-improvement loop), PART 27 (self-modification). External: FunSearch (DeepMind, Nature 2024), Darwin Godel Machine (`arxiv 2505.22954`), DarwinNet (`arxiv 2604.01236`), IC3-Evolve.

Disclaimer on DGM relationship:
- Verbatim: "elai_cc adopts specific tested mechanisms from DGM (config archive, log-driven proposals, staged evaluation, self-referential alignment) but is architecturally different: elai_cc modifies configs and rules around a frozen FM, DGM modifies agent codebases including tools and strategy."

Core safety property (section 1): "The modifying agent NEVER evaluates its own modifications. Deterministic benchmarks judge. The agent proposes, code decides."

Three-layer architecture (section 2):

LAYER 1 — DATA (automatic, no approval):
- Changes: template rankings, rule trigger counts, pipeline outcome metrics.
- Mechanism: pure arithmetic from `PipelineOutcome`. No LLM.
- Security/safety/architecture rules NEVER auto-adjusted.

LAYER 2 — CONFIG (full pipeline, auditor approval):
- Changes: new rules, GRC budgets, context ratios, model assignments, templates, tool configs.
- Proposal mechanism (DGM log-driven): self-mod agent queries SSB for full pipeline trace including every `reasoning_trace` entry (`orchestrator.txt 7A`). Verbatim: "reasoning traces provide WHERE in the GRC chain reasoning failed, not just THAT a phase failed." Example: "executor's observe step cited a stale grounding source at step 4, causing a cascade of wrong hypotheses -> propose stricter cache invalidation". Heuristic threshold triggers remain as fallback only.
- Config archive (DGM population-based) — SQL schema:
  ```
  config_variants(
    variant_id TEXT PK,
    parent_id  TEXT,
    score      REAL,
    children_count INTEGER,
    config_blob BLOB,
    created_at INTEGER,
    pipeline_outcomes TEXT,
    status TEXT CHECK(status IN ('ACTIVE','ARCHIVED','REVERTED'))
  )
  ```
- Parent selection (DGM sigmoid-weighted, section 2.3):
  ```
  s_i = sigmoid(lambda * (alpha_i - alpha_0))    // lambda=10, alpha_0=0.5
  h_i = 1 / (1 + n_i)                            // n_i = children count
  w_i = s_i * h_i ; p_i = w_i / sum(w_j)         // all variants non-zero probability
  ```
- Cited evidence: "Agent at iteration 4 temporarily dropped performance but spawned lineage exceeding all predecessors. Single-config rollback would have killed that lineage."
- Max archive size: `50 variants` (oldest low-score archived to cold storage).
- Active config: variant with `status='ACTIVE'`. Only one at a time.

Layer 2 process (9 steps):
1. EVALUATE: after every pipeline completion, self-mod agent reads SSB logs. Evaluation cheap (SSB query). Execution bounded: `max 1 self-mod per 3 completed pipelines`.
2. SELECT PARENT (DGM sigmoid-weighted from `config_variants` archive).
3. PROPOSE: log-driven proposal targeting specific config change.
4. PLAN: proposed change as `PipelineSpec` targeting `.elai_cc/` config. Full 4-stage elaboration (research -> plan -> validate -> probe).
5. SANDBOX: copy selected parent config to `.elai_cc/sandbox/{mod_id}/`.
6. BENCHMARK (staged evaluation, DGM-proven):
   - Stage 1: 10 tasks (basic sanity). Fail -> discard immediately.
   - Stage 2: 50 tasks (if Stage 1 passes). Include trigger task.
   - Stage 3: 200 tasks (if `>40%` success on 60-task subset AND ranks top 2 in archive). Threshold per DGM section 2.5. `TestOutcomeReport`: any `pass_to_fail` -> reject. Include 3 SWE-bench rotation instances.
   - Comparison deterministic: subtract two numbers.
7. PROOF GATE (IC3-Evolve pattern, before Stage 2):
   - Run DST reference model with the proposed config change against a FIXED set of `10 regression test traces` (canonical cassettes).
   - Gate checks SOUNDNESS, not behavioral identity:
     - PASS: all evidence gates still trigger correctly, no BLOCK findings bypassed, no test regression (PASS_TO_FAIL), FSM reaches COMPLETE.
     - FAIL: evidence gate bypassed (e.g., UNGROUNDED claim reaches EXECUTING), test regression detected, FSM stuck or crashes.
   - Key distinction: "IC3's gate checks correctness (SAFE/UNSAFE classification matches ground truth). Our gate checks soundness (evidence gates not bypassed, no regressions), not behavioral identity."
   - Evidence citations: "Rounds r093-r094 proposed a clause-cleaning heuristic showing +9 solved instances... However, it failed independent witness validation" (IC3-Evolve section 10.1). "This separates optimization from soundness." (IC3-Evolve section 4)
8. FINAL GATE: better on trigger task AND no SWE-bench regression AND auditor finds no safety regressions AND no evidence gate definitions modified AND proof gate passed. PASS -> INSERT as new variant (`status='ACTIVE'`), demote previous active to `'ARCHIVED'`. FAIL -> insert as variant with low score (stays in archive).
9. CROSS-TRANSFER VALIDATION (optional, post-gate): test winning config with a different FM (e.g., cloud model if primary is local). DGM proved improvements that transfer across models are genuine, not FM-specific. Fail -> flag as FM-specific optimization, keep but mark.

Slot-restricted patches (IC3-Evolve pattern) — enumerated slots:
- Slot 1: GRC budgets (`max_calls`, `max_chars` per role).
- Slot 2: Rule thresholds (FP rate, TP rate, promotion gates).
- Slot 3: Context budget percentages (`repo_map_pct`, `code_context_pct`, `ssb_pct`, `grc_pct`). NOTE: changes to auditor's `code_context_pct` require human approval per Layer 3 (auditor context quality affects alignment loop).
- Slot 4: Model parameters (`temperature`, `top_p` per role).
- Max `1 slot` modified per proposal. Cross-slot changes require explicit `Compass&Jump mode` (higher evidence bar).
- Forbidden: FSM states, IPC protocol, security rules, evidence gates.

Anti-reward-hacking locks (PreToolUse enforced):
- READ-LOCKED during self-mod: metrics code, gate definitions, auditor config, benchmark harness, test suite, `config_variants` table schema. Modifying agent cannot write to these.

LAYER 3 — STRUCTURE (human approval, always):
- Changes: FSM state additions, new agent roles, GRC protocol changes, evidence gate modifications, core pipeline control flow.
- Same process as Layer 2 but human review required in BOTH modes (INTERACTIVE and AUTONOMOUS).

Self-improvement metrics (section 3):

Types:
```
PipelineOutcome {
    template_id, project_id, phases_total, phases_passed_first_try,
    audit_findings_total, audit_findings_block, test_coverage_final,
    time_to_complete_seconds, tokens_consumed_total, human_interventions,
    rules_triggered: Vec<(RuleId, TriggerCount, CatchRate)>,
    test_outcomes: TestOutcomeReport
}
```

`TestOutcomeReport` (SWE-bench-derived 4-category grading):
- `FAIL_TO_PASS`: tests that failed before and pass after (resolution).
- `PASS_TO_PASS`: passed before and still pass (maintenance).
- `FAIL_TO_FAIL`: failed before and still fail (no regression, no fix).
- `PASS_TO_FAIL`: passed before but now fail (REGRESSION).
- Gate: any `PASS_TO_FAIL` -> rejected even if positive `FAIL_TO_PASS`. Stricter than FP rate alone.
- Derived from: SWE-bench harness `grading.py get_eval_tests_report()`.

Modification Solidification Index (MSI, DarwinNet-derived, `arxiv 2604.01236`):
- Definition: `MSI = weighted_moving_avg(delta_score)` over last `N=10` variants (default). `delta_score = variant.score - variant.parent.score`. `MSI in [-1, 1]`.
- Phase classification:
  - `MSI > 0.3`: IMPROVING.
  - `-0.1 < MSI <= 0.3`: STABILIZING.
  - `MSI <= -0.1`: DIVERGING.
- Actions:
  - IMPROVING: maintain 1 per 3 pipelines.
  - STABILIZING: reduce to 1 per 5 pipelines. Log "self-mod entering diminishing returns."
  - DIVERGING: pause self-mod entirely. Alert user. Revert to highest-score variant.
- Storage: `self_mod_history.db` alongside `config_variants`. Computed after every Layer 2 benchmark. Pure arithmetic, no LLM.

Rule weight adjustment:
- SAFETY RULES (Security, Safety, Architecture): NEVER auto-adjusted.
- ADJUSTABLE RULES (Style, Custom): high false positive -> flag for REFINEMENT (not removal).

Self-referential alignment (section 4, DGM-derived):
- Feedback loop: `pipeline runs -> SSB logs accumulate -> self-mod reads logs -> proposes change -> counterfactual rule testing validates causation (rules.txt section 5A) -> config archive captures variant -> next pipeline benefits -> better logs -> better proposals -> tighter loop`.
- Stated limitation: "elai_cc's loop is weaker than DGM's because the FM is frozen. DGM agents modify their own tools and strategies; elai_cc can only modify configs, rules, budgets, and templates around a fixed model. The FM's coding ability doesn't improve, only the orchestration around it does."

Rollback protocol (section 5):
- Every Layer 2 mod: new variant in `config_variants` archive.
- Monitored for `3 pipeline runs`. Auto-revert if metrics degrade `>10%` (status -> `REVERTED`, previous active restored).
- Storage: `.elai_cc/global/self_mod_history.db`.

Resource scheduling (section 6):
- Lowest priority. Idle time only. User task always preempts.
- Evaluation (log reading + proposal): after every pipeline. Cheap.
- Execution (sandbox + benchmark): frequency controlled by MSI (default 1 per 3 pipelines). IMPROVING: 1/3. STABILIZING: 1/5. DIVERGING: paused.
- Staged evaluation reduces cost; Stage 3 (200 tasks) only for top candidates. `~90 min at lowest priority`.

Testable invariants (section 7):
- Self-mod agent cannot write to metrics/gates/auditor-config during pipeline.
- Layer 2 benchmarked against deterministic metrics (no LLM judgment).
- Auto-revert on `>10%` degradation over 3 pipelines (`variant status -> REVERTED`).
- Layer 3 always requires `human_approved=true`.
- Sandbox physically separate from active config.
- Only one variant has `status='ACTIVE'` at any time.
- All variants have non-zero selection probability (DGM sigmoid-weighted).
- Max 50 variants (overflow archives lowest-score).
- Staged evaluation: Stage 1 failure -> no Stage 2 (early termination enforced).
- Log-driven proposals: self-mod reads SSB logs (not heuristic threshold only).
- Cross-transfer validation: FM-specific optimizations flagged (not silently applied).
- Self-referential loop: every self-mod proposal cites specific SSB log entries as evidence.
- MSI computed after every Layer 2 benchmark (pure arithmetic, no LLM).
- MSI DIVERGING pauses self-mod and alerts user (never silently continues).
- MSI STABILIZING reduces frequency to 1 per 5 pipelines (not 1 per 3).
- MSI in `[-1, 1]` range.
- Config variant with any `PASS_TO_FAIL` regressions rejected at gate.
- `TestOutcomeReport` computed for every pipeline (4-category, deterministic).
- Proof gate runs against 10 canonical cassettes before Stage 2 (soundness check).
- Proof gate rejects evidence-gate-bypass but allows different BLOCK/ALLOW distributions.
- Slot restriction: max 1 slot per self-mod proposal.
- Cross-slot changes require Compass&Jump mode (enforced by proposal validator).
- Slot 3 auditor `code_context_pct` changes require human approval (Layer 3 escalation).

Explicit numbers (this file):
- Self-mod execution cadence baseline: 1 per 3 pipelines; STABILIZING 1 per 5.
- Archive max size: 50 variants.
- Stage 1: 10 tasks. Stage 2: 50. Stage 3: 200 (threshold >40% on 60-task subset, rank top 2).
- 3 SWE-bench rotation instances in Stage 3.
- Proof gate: 10 canonical cassettes.
- MSI window N=10; IMPROVING >0.3, STABILIZING -0.1..0.3, DIVERGING <=-0.1.
- Sigmoid params: `lambda=10`, `alpha_0=0.5`.
- Rollback monitor: 3 pipelines; auto-revert `>10%` degradation.
- Stage 3 runtime budget: ~90 min at lowest priority.

Hardcoded paths / schemas / config keys (this file):
- `.elai_cc/sandbox/{mod_id}/`
- `.elai_cc/global/self_mod_history.db`
- Table `config_variants` (schema above).
- Cross-ref: `orchestrator.txt 7A` (reasoning traces), `rules.txt section 5A` (counterfactual rule testing).

Dependencies:
- Phase producing rules.db and counterfactual rule testing (rules crate).
- `orchestrator.txt 7A` reasoning traces.
- SSB for log storage and queries.
- PipelineOutcome / TestOutcomeReport computation harness.
- Auditor for safety regression review.
- FSM reaching COMPLETE state (proof gate prerequisite).

Status markers:
- Layer 1 "automatic, no approval".
- Layer 2 "full pipeline, auditor approval".
- Layer 3 "human approval, always".
- Proof gate, MSI, slot-restriction are stated as mandatory design, not proposals.

Integration/wiring requirements:
- SSB must expose log queries for phase outcomes, audit findings, anti-spiral triggers, context budget reductions, rule trigger counts, reasoning_trace entries.
- PreToolUse hook must enforce anti-reward-hacking read-locks.
- Proposal validator must enforce slot restriction (max 1 slot per proposal, cross-slot requires Compass&Jump).
- Gate executor must run DST reference model against 10 canonical cassettes.
- `self_mod_history.db` must store config_variants + MSI time series.

Verbatim quotes (non-obvious decisions):
- "The modifying agent NEVER evaluates its own modifications. Deterministic benchmarks judge. The agent proposes, code decides."
- "DGM proved: log-driven diagnosis (analyzing evaluation logs, failed tasks, patches, test results) enables targeted self-improvement."
- "reasoning traces provide WHERE in the GRC chain reasoning failed, not just THAT a phase failed."
- "IC3's gate checks correctness... Our gate checks soundness (evidence gates not bypassed, no regressions), not behavioral identity."
- "elai_cc's loop is weaker than DGM's because the FM is frozen."
- "Forbidden: FSM states, IPC protocol, security rules, evidence gates."
- "SAFETY RULES (Security, Safety, Architecture): NEVER auto-adjusted."
