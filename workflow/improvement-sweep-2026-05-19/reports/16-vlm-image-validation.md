# VLM image validation for ELAI

## TL;DR
- Phase 4I's two-tier design (deterministic Tier 1 + VLM Tier 2 advisory) is the right shape; the literature converges on "symbolic-first, VLM-as-tiebreaker" because VLM-as-judge is provably sycophantic, position-biased, and verbosity-biased (Shi et al. IJCNLP 2025; Zhao et al. 2025; arXiv 2408.11261).
- Swap Qwen3.5-VL-30B for **Qwen3-VL-32B dense** or **Qwen3-VL-30B-A3B MoE** (Nov 2025 release, MathVista 85.8%, MMMU-Pro 69.3%, beats GPT-5 on visual math) and add **InternVL3.5-38B** as the second-opinion judge for disagreement gating (arXiv 2511.21631, arXiv 2508.18265).
- Add an **ARIA AXTree snapshot adapter** (Playwright `page.ariaSnapshot()` v1.59+) to Tier 1 — this is structural truth for web UIs and catches ~40% of WCAG issues axe-core misses on context-dependent violations; it is far cheaper than VLM-judge and not sycophantic.
- Adopt the **RRVF / vision-guided iterative refinement** pattern (arXiv 2604.05839, arXiv 2507.20766) as the loop topology: render → VLM critic with structured-JSON feedback (XGrammar) → executor edit → re-render, cap at 3-5 iterations with LPIPS-delta convergence detector.
- Add **anti-sycophancy guards**: query-neutralization + contrastive-decoding (arXiv 2408.11261) on Tier 2 prompts, and a swap-order pairwise eval mode (position-bias mitigation from Shi et al.) when comparing two render attempts.

## Key sources
URL or arxiv | what | takeaway
--- | --- | ---
arXiv:2408.11261 | Sycophancy in VLMs: systematic analysis + mitigation | Qwen-VL, CogVLM, InternVL, LLaVA, mPLUG all sycophantic; contrastive decoding mitigates without retraining
arXiv:2509.21979 | Medical-VLM sycophancy (EchoBench, MM-SY) | First sycophancy benchmark; VLMs prioritize user prompt over visual data when tone is assertive
arXiv:2406.07791 | "Judging the Judges" position bias (AACL-IJCNLP 2025) | 15 judges, ~150K instances; position bias not random; swap-order accuracy shifts >10% in pairwise code judging
arXiv:2511.21631 | Qwen3-VL Technical Report (Nov 2025) | Dense 2/4/8/32B + MoE 30B-A3B/235B-A22B; MMMU-Pro 69.3, MathVista 85.8, MathVision 74.6; 256K context
arXiv:2508.18265 | InternVL3.5 | 8B/38B/241B-A28B; +16% reasoning vs InternVL3; 4.05x inference speedup; Visual Resolution Router
arXiv:2403.03163 | Design2Code (NAACL 2025) | 484 real webpages; Block-match + Text + Position + Color + CLIP metrics; "models lag in recalling visual elements and generating correct layouts"
arXiv:2604.05839 | Vision-Guided Iterative Refinement for Frontend Codegen | RCE loop with VLM critic providing structured feedback; iteration cap + convergence detection
arXiv:2507.20766 | RRVF: Visual RL with Reasoning + Rendering + Visual Feedback | MLLM reasons → renders code → iteratively refines on visual discrepancy
arXiv:2411.15100 | XGrammar (MLSys 2025) | 100x speedup over FSM; <40us/token JSON; default backend for vLLM/SGLang/TensorRT-LLM
arXiv:2409.13711 | WebQuest benchmark | Multimodal QA on web page sequences; single/multi-screen + nav-trace categories
arXiv:2401.10935 | SeeClick + ScreenSpot | GUI grounding pre-training; 600 screenshots/1200 instructions across iOS/Android/macOS/Windows/web
arXiv:2410.07073 | Pixtral 12B | DocVQA 90.7, ChartQA 81.8, VQAv2 78.6; beats Qwen2-VL-7B and Llama-3.2-11B
arXiv:2511.12675 | "Appreciate the View" — NVS evaluation framework | PSNR/SSIM need target, FID ignores consistency, Met3r breaks at large viewpoint changes
arXiv:2407.16124 | Fréchet Video Motion Distance | FVD has content-bias, prioritizes per-frame over temporal; fails human preference on flicker/motion
playwright.dev/docs/aria-snapshots | ARIA snapshots in Playwright | `locator.ariaSnapshot()` (v1.49+), `page.ariaSnapshot()` (v1.59+); YAML structural tree
deque.com/axe-core | axe-core 4.8 coverage | Only ~57% of WCAG detectable automatically (zero false positives); 75% time reduction
percy.io 2025 update | Visual Review Agent (Percy AI) | Replaces pixel highlighting with smart bounding boxes around meaningful changes — pixel-diff alone is being abandoned
mlc-ai/xgrammar (GitHub) | Default structured-gen backend | vLLM/SGLang/TensorRT-LLM as of Mar 2026

## VLM judge calibration

**Sycophancy** (arXiv 2408.11261, arXiv 2509.21979): VLMs including Qwen-VL, CogVLM, InternVL, LLaVA, mPLUG agree with leading or deceptive user prompts and override correct visual evidence. The MM-SY benchmark across 10 task families (activity recognition, attribute, color, counting, object presence, object recognition, positional reasoning, scene, sport, utility affordance) shows consistent susceptibility. The user tone matters: assertive prompting collapses visual grounding.

**Position bias** (arXiv 2406.07791, AACL-IJCNLP 2025): 15-judge / ~150K-instance study finds position bias is structural, not noise. Repetition stability, position consistency, and preference fairness all suffer. In pairwise code judging, simply swapping option order shifts accuracy >10 percentage points. Bias is quality-gap-dependent (close calls flip more).

**Verbosity / recency bias** (Adaline 2025 blog, emergentmind LLM-as-judge): judges prefer verbose, fluent, formal outputs regardless of substance — an RLHF artifact. A strong recency bias systematically favors "new" over "old" responses in pairwise comparison.

**Language-over-vision bias** (VLM-DeflectionBench, arXiv 2604.12033): models handle visual distractors fine when positives are textual, but collapse when textual distractors interfere with visual positives — misleading text overrides correct visual evidence.

**Inter-rater reliability**: direct VLM-as-judge on images reaches ICC=0.718 (good) vs caption-based at ICC=0.339 (poor) (MDPI 2025 visual narrative coherence study). VLM judge correlates ρ≈0.77-0.90 with human experts on rubric-driven tasks (PMC 12926494). Solid signal, but not a substitute for symbolic ground truth.

## VLM landscape (May 2026)

Model | Params | Best benchmark score | VRAM (approx) | Constrained-decoding compat | Local-feasible | Citation
--- | --- | --- | --- | --- | --- | ---
Qwen3-VL-2B | 2B dense | MMMU mid-tier | ~1.5GB Q4 | XGrammar via vLLM/SGLang | yes | arXiv:2511.21631
Qwen3-VL-4B | 4B dense | strong for size | ~2.5GB Q4 | XGrammar | yes | arXiv:2511.21631
Qwen3-VL-8B | 8B dense | MMMU strong | ~5GB Q4 | XGrammar | yes (12GB) | arXiv:2511.21631
Qwen3-VL-32B | 32B dense | MMMU-Pro ~69, MathVista 85.8 | ~18GB Q4 | XGrammar | borderline 24GB GPU | arXiv:2511.21631
Qwen3-VL-30B-A3B | 30B MoE, 3B active | near-32B at lower compute | ~17GB Q4 | XGrammar | yes 24GB | arXiv:2511.21631
Qwen3-VL-235B-A22B | 235B MoE, 22B active | flagship; MathVision 74.6 beats GPT-5 65.8 | ~120GB+ | XGrammar | no | arXiv:2511.21631
InternVL3.5-8B | 8B | competitive open-source | ~5GB Q4 | XGrammar via SGLang | yes | arXiv:2508.18265
InternVL3.5-38B | 38B | reasoning-focused | ~22GB Q4 | XGrammar | tight on 24GB | arXiv:2508.18265
InternVL3.5-241B-A28B | 241B MoE, 28B active | SOTA open-source overall | ~140GB+ | XGrammar | no | arXiv:2508.18265
Pixtral 12B | 12B | DocVQA 90.7, ChartQA 81.8 | ~7GB Q4 | Outlines compatible | yes | arXiv:2410.07073
Llama-3.2-11B-Vision | 11B | beaten by Pixtral-12B and Qwen2-VL-7B | ~7GB Q4 | XGrammar | yes | bentoml.com 2026
Llama-3.2-90B-Vision | 90B | beaten by Pixtral 12B on multimodal | ~50GB Q4 | XGrammar | no | bentoml.com 2026
GLM-4.5V | mid-tier | occasional repetition issues noted | ~mid | basic | yes | anotherwrapper 2026

ELAI's current "Qwen3.5-VL-30B-A3B ~17GB Q4" matches Qwen3-VL-30B-A3B; the literature confirms it as the sweet spot for single 24GB GPU. The "Mode B InternVL3.5-38B ~38GB 8-bit" pick is also sound but Q4 cuts VRAM ~half.

## Visual regression strategy comparison

Strategy | Cost | Strengths | Weaknesses | Citation
--- | --- | --- | --- | ---
Pixel-diff (Percy, Chromatic, BackstopJS, Lost Pixel) | <50ms | exact reproduction catch, deterministic | catastrophic on antialias, font-render variance, network-flake; Percy 2025 explicitly pivoted to AI bounding-box highlights to suppress noise | percy.io 2025, lost-pixel.com
SSIM | <100ms CPU | luminance+contrast+structure; widely understood | weak on textures and complex distortions; misses perceptual layout shifts | eureka.patsnap.com 2025
LPIPS | ~100-300ms CPU | VGG-feature perceptual similarity; aligns with human judgment on natural images | trained on natural images not UIs; needs ONNX export; not for OCR-grade text | unifiedimagetools.com 2025
DISTS | ~200ms CPU | tolerates texture resampling; SSIM+texture hybrid | similar UI-domain mismatch as LPIPS | eureka.patsnap.com 2025
axe-tree / Playwright AXTree (`page.ariaSnapshot()` YAML) | ~100ms | structural truth, semantic, role/name/value; deterministic; cheap | only for web UIs (no Blender/FFmpeg coverage); doesn't catch visual aesthetics | playwright.dev v1.59
axe-core / Pa11y WCAG | ~200ms | catches ~57% WCAG with zero FPs (4.8) | misses context-dependent (alt-text accuracy, chart long-descs); ~30-40% of issues need humans | deque.com, leadwithskills 2025
VLM-judge (Tier 2) | 2-5s | semantic understanding, "does this look like a TODO app", catches things no symbolic check sees | sycophancy, position/verbosity bias, hallucinations, costly | arXiv 2408.11261, 2406.07791

**Verdict**: pixel-diff is being abandoned even by commercial vendors (Percy 2025 Visual Review Agent draws bounding boxes around "meaningful" changes); SSIM is good as a flake-suppressor under tight thresholds; LPIPS is best perceptual signal but train-domain mismatch on UIs; **axe-tree + AXTree-YAML is the underused gem** for web UIs and ELAI's current Tier 1 does not appear to include it.

## Best techniques for Media Validator

Ranked.

1. **AXTree-YAML structural snapshot as a Tier 1 web adapter** | grounding: Playwright v1.59 `page.ariaSnapshot()` returns deterministic YAML accessibility tree; semantic role+name+value tree comparison catches structural regressions pixel-diff misses | integration: new `Tier1::AXTreeAdapter` for the web MediaRenderAdapter, deterministic <100ms, fits the Tier 1 budget.
2. **Anti-sycophancy prompt protocol for Tier 2** | grounding: arXiv 2408.11261 — query neutralization (strip "this is the X you asked for", "does this match"), contrastive decoding between neutral and leading prompts | integration: rewrite Tier 2 VLM prompts in Media Validator to be neutral-framing only; never tell the VLM what the executor was supposed to render.
3. **Swap-order pairwise eval** for two render attempts | grounding: arXiv 2406.07791 — position bias shifts accuracy >10pp in pairwise code judging | integration: when Veto logic compares two candidates (e.g. before/after fix), evaluate both orderings and require agreement; disagreement → HumanReview.
4. **XGrammar JSON-schema constrained decoding** for VlmValidationOutput | grounding: arXiv 2411.15100 + mlc-ai/xgrammar — <40us/token overhead, 100x faster than Outlines FSM, default backend of vLLM/SGLang/TensorRT-LLM as of Mar 2026 | integration: ensure Tier 2 inference goes through SGLang (which ELAI uses) with XGrammar backend; current code likely already on this path but verify schema registration.
5. **RCE loop with LPIPS-delta convergence + iteration cap** | grounding: arXiv 2604.05839, arXiv 2507.20766 — iterative refinement with VLM critic; converge when delta drops below threshold | integration: extend Render-Capture-Evaluate into a true loop in MediaRenderAdapter; max 3-5 iters; convergence = LPIPS(prev, curr) < 0.02 or VLM-critic returns "no actionable diffs".
6. **Second-opinion judge gating** for HumanReview reduction | grounding: arXiv 2406.07791 shows judge disagreement is a strong signal of edge cases; InternVL3.5 and Qwen3-VL agree highly on confident cases | integration: when Tier 2 (Qwen3-VL) is uncertain (low logprob on JSON verdict), spawn InternVL3.5 as Mode B and require both agree before passing; reduces HumanReview backlog without dropping safety.
7. **Design2Code rubric metrics** for web UI codegen output | grounding: arXiv 2403.03163 Block-match + Text + Position + Color + CLIP | integration: implement as a deterministic Tier 1 plugin when the executor task was "build a webpage"; cheaper than VLM-judge for layout fidelity.
8. **ScreenSpot/SeeClick-style GUI grounding check** when the spec is "click X" | grounding: arXiv 2401.10935 — ScreenSpot is the gold standard for UI element localization | integration: a Tier 2 sub-mode that asks the VLM "where is the Submit button?" with a bounding box constrained-JSON output; verify it matches the rendered DOM via Playwright.
9. **Drop FVD-only animation eval** | grounding: arXiv 2407.16124 — FVD has content bias, prioritizes per-frame over temporal coherence | integration: for the FFmpeg-animation adapter, add VBench-style multi-dimension scoring or pair FVD with CLIP-T + a flicker detector.
10. **Pixtral 12B as a backup Tier 2** | grounding: arXiv 2410.07073 — DocVQA 90.7, ChartQA 81.8, beats Qwen2-VL-7B/Llama-3.2-11B on instruction following | integration: keep behind a feature flag; useful when Qwen3-VL hits its context limit or for OCR-heavy renders.

## When NOT to use VLM

Symbolic alternative > VLM-judge in these cases:

- **CSS layout assertion**: parse the rendered CSS computed-style + DOM box model; Playwright `evaluate()` returns numeric positions; deterministic, no sycophancy. (playwright.dev, AXTree docs)
- **Color contrast / WCAG-AA**: axe-core does this with zero FPs; VLM hallucinates contrast ratios. (deque.com)
- **Element existence / role / label**: AXTree YAML snapshot equality is faster, cheaper, and not biased. (playwright.dev v1.59)
- **Text content match**: Tesseract OCR + string compare; never ask a VLM to "verify the title says X". (ELAI's current Tier 1 already does this — keep it.)
- **3D vertex/topology**: Blender CLI mesh export + numeric assertion; never ask a VLM to count vertices.
- **Audio sample rate / channel count**: FFmpeg `-show_streams`; deterministic.
- **Pixel-exact reproduction** (e.g. screenshot golden file): pHash or SSIM with tight threshold; VLM is overkill and noisier.
- **Pairwise "which is better"**: at minimum require swap-order agreement; even then, prefer symbolic deltas (LPIPS-delta sign + rubric score sign). (arXiv 2406.07791)

## Anti-patterns

- **Single-shot VLM verdict without grounding** — arXiv 2408.11261 shows the user prompt overrides visual evidence; never let the executor tell the validator what to expect.
- **Pixel-exact diff as primary gate** — Percy 2025 abandoned this for AI-highlight regions; flake from antialias/font/network kills CI. (percy.io blog 2025)
- **VLM-judge for pairwise code/UI comparison without swap-order** — arXiv 2406.07791: >10pp shift just from order.
- **Caption-then-judge** (caption an image, then judge the caption) — ICC drops from 0.718 to 0.339; always judge images directly. (MDPI 2025)
- **GPU-EP for "deterministic" VLM inference** — non-determinism in CUDA reductions; ELAI's CPU EP choice is correct, but quantization (NF4/AWQ) also introduces calibration-dependent drift. Pin model+quant config + tokenizer hash. (onnxruntime.ai docs)
- **FVD-only video eval** — content-biased, ignores flicker/motion. (arXiv 2407.16124)
- **Trusting axe-core as full WCAG** — only ~57% coverage; the remaining 43% (alt-text accuracy, chart long-descriptions) are exactly the things VLM can help with. (deque.com 2025)
- **No iteration cap on RCE loop** — convergence detection is mandatory; arXiv 2604.05839 explicitly motivates max-iter to avoid infinite drift.
- **Mocking the VLM judge in CI tests against fixtures** — if the judge is part of the gate, drift between fixture-VLM and prod-VLM produces false-green builds.

## Open questions

1. **Sycophancy under neutral framing**: does Qwen3-VL-30B-A3B with query-neutralization (arXiv 2408.11261) drop sycophancy rate below 5% on MM-SY, or do we still need contrastive decoding? Measurable on ELAI's 200-image internal eval set with leading vs neutral prompts.
2. **Inter-judge agreement (Qwen3-VL vs InternVL3.5)** on ELAI's RealWorldQA-style dashboard screenshots: target ICC ≥ 0.7 to retire HumanReview escalation on agreement.
3. **LPIPS-delta convergence threshold** for the RCE loop: at what delta (0.01? 0.02? 0.05?) does additional iteration stop improving Design2Code Block-match score? Sweep on 50 representative renders.
4. **AXTree-YAML snapshot stability** across runs: how much drift do deterministic-mode Playwright runs produce on the same React component? Need <1% YAML-diff rate to make it a reliable Tier 1 gate.
5. **Tier 1 vs Tier 2 cost-Pareto**: at what failure-rate threshold does it pay to escalate from AXTree+axe to Qwen3-VL? Should be measurable as a ROC curve on ELAI's existing veto-log.
