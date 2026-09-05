# J-PERCEPTION-GROUNDING-MAP — grounding for J-8 (vlm perception upgrade pack) + J-9 (hardware model catalog)
Grounded 2026-06-11, session s-j8j9-author (main LLM + one prompt-engineer grounding agent).
Method: full read of the 3-file VLM research corpus in C:/Users/user/Downloads/ + the condensed
vlm-vision-research-summary.md, live grep sweep over .elai_cc (system_probe / vram_tier /
context_budget / agent_pool / research_tier / runtime_builder / image_artifact / ssb constants),
odysseus-INDEX.md anchor pull, web verification of every arXiv id cited. Every J-8/J-9 detail-file
claim cites these IDs instead of re-asserting facts (R-a). PLAN-asserted anchors are labelled;
executors re-grep (W-7). NOTE: the hardware family is named HW (not HD) — HD is already the H-plan
delta family (J-0_INDEX doctrine block cites HD-1/HD-2); reusing it would corrupt cross-references.

================================================================================
VR — VLM RESEARCH GROUND TRUTH (the 3-file corpus; sources carried per claim)
================================================================================
Corpus: "Top 10 Vision-Language Models for Image Understanding + Cutting-Edge Architectures
(2025-2026).md", "Making LLM Image Reading Actually Work  Browser Tabs, Designs, Game UI &
More.md", "The Best Mix of Models for Human-Like AI Eyesight (2025-2026).md", all in
C:/Users/user/Downloads/, plus the condensed vlm-vision-research-summary.md. ALL figures below are
EXTERNAL ([DESIGN] per HD-1/ID-5): design rationale, never ELAI claims; every shipped number is
ELAI-measured.

VR-1  resize sweet spots, longest side: Claude 1568px / GPT-4o 2048px (high detail) /
      Qwen-VL-family 1280px / Gemini native up to ~3K. Rule: when the target API supports native
      resolution (Gemini, newer Claude) do NOT force a fixed size — passthrough. Source: "Making
      LLM Image Reading" Part 3 (tianpan.co + reddit-cited measurement). Numbers re-verify against
      current provider docs at execution (they drift with model releases).
VR-2  normalized coordinates: ask for 0-1000-scale [ymin,xmin,ymax,xmax], never raw pixels;
      the cited measurement: coordinate error ~15% -> <2% with proper resize + normalized coords.
      [DESIGN]. J-1's done-gate already demands <2% on the click-target fixture; VR-2 is the
      MECHANISM and goes into the scaffold JSON schema as a hard rule.
VR-3  counting >10 small elements degrades sharply on every model tested; counts must come from
      the AXTree/DOM leg or a deterministic script, never the VLM. Already harness law
      (visual-ui-loop.md "NEVER COUNT VISUALLY"); J-8 ports the rule into the RUNTIME composer.
VR-4  multi-image evidence integration degrades (all 3 docs agree). Mitigations: tiled-composite
      (InternVL dynamic-resolution style, VR-14) or the B-24 Mode B specialist (InternVL3.5-38B
      via SGLang, RadixAttention multi-image KV — phase-index-MASTER 4I block). HONEST RESIDUE:
      routing reduces, does not eliminate, the degradation (GD-5).
VR-5  nested small-text UI (tooltips, micro-labels) stays brittle EVEN WITH tiling. HONEST
      RESIDUE line for J-8: tiling is best-effort; micro-text asks route to tiling THEN typed
      refusal (D-16) when confidence is low, never a guess.
VR-6  MoAI (KAIST, arXiv:2403.07508): verbalizing specialist-CV outputs into the VLM context
      beats any lone VLM — the design rationale for J-1's hybrid composer (AXTree/DOM text
      alongside the image IS the verbalized-specialist pattern). [DESIGN] citation only.
VR-7  image prompt caching: ~90% savings on re-queried identical screenshots [DESIGN].
      Composition note: image blocks live AFTER the last cache breakpoint per HD-2 (J-1 task 2
      already places the scaffold there); I-10's prefix-invariance guard is the enforcement layer
      — a re-queried screenshot must hit the cached prefix, a mutated image must not bust the
      frozen prefix ahead of it.
VR-8  dots.ocr (1.7B): corpus claims it beats GPT-4o and Qwen2-VL-72B on OCR. Source is a vendor
      blog + LinkedIn post (getomni.ai), NOT peer-reviewed — weight [DESIGN], EVALUATE-not-ship
      vs the E-36 lane (ColQwen2.5 / PaddleOCR-VL, HW-13). Exact package/HF name unverified
      (JC-7).
VR-9  Qwen3-VL adds embedding + reranker models for vision RAG (towardsai-cited). EVALUATE line
      vs ColQwen2.5 for the E-36 vision-RAG index. HF model-card names verify at edit (JC-8).
VR-10 design-mockup playbook: structured component-inventory prompt — for each component emit
      {type, verbatim text, hierarchy, layout cues} before any codegen. Ported as a prompt-shape
      TEMPLATE in J-8's library; H-14 (PLANNED) consumes it when it executes — J-8 never edits
      H-14's file.
VR-11 FastVLM (Apple, CVPR 2025, github apple/ml-fastvlm): ~85x faster TTFT at high res,
      on-device. Relevant ONLY to a future E-51 MLX lane; NOT to the remote-3090 sidecar path.
      One catalog/library note, nothing shipped (JC-9).
VR-12 AVR adaptive routing (arXiv:2603.12823): probe small VLM first, escalate on low confidence;
      -78% cost within 2pts of always-large; safety-critical always to the large model. [DESIGN]
      rationale for J-1 leg 4; J-8 supplies the probe-model tier table (VR-13 + HW-13 seed).
VR-13 per-VRAM vision-model ladder from the ranking doc [all DESIGN; re-verify at eval]:
      sub-6B/8GB = Phi-4-Multimodal (MMMU 55.1); single 24GB = Gemma 3 27B QAT (MMMU 64.9) or
      Qwen2.5-VL-7B (basic OCR); 48GB+ = Qwen2.5-VL-32B (~equal to 72B at half compute) /
      Qwen2.5-VL-72B (MMMU 70.2, SOTA SpatialBench) / InternVL3-78B (MMMU 72.2, DocVQA 95.4;
      14B ~GPT-4o on 2x24GB). Cloud = the Anthropic lane (Opus: best document/chart/dense-text,
      200K multi-screenshot). Seeds the J-9 vision lane + the J-1 leg-4 probe choice.
VR-14 InternVL3 (arXiv:2504.10479): native multimodal pretraining; dynamic-resolution tiling
      (overlapping tiles + global thumbnail) is the proven boost for fine text — the J-1 leg-1
      tiling shape.

================================================================================
HW — HARDWARE-DETECTION GROUND TRUTH (.elai_cc live greps 2026-06-11)
================================================================================
HW-1  GpuInfo: crates/orchestrator/src/system_probe.rs:80 — name, vram_total_mb, vram_free_mb.
HW-2  SystemProbeResult: system_probe.rs:127 — gpu, ram_total_mb, ram_free_mb, disk_free_mb.
      (C-22 shipped surface.)
HW-3  classify_vram_tier(total_mb) -> VramTier: system_probe.rs:145.
HW-4  VramTier enum: crates/core/src/vram_tier.rs:6 — Vram8Gb / Vram12Gb / Vram16Gb / Vram24Gb /
      Vram48Gb / Cloud.
HW-5  detect_model_capability_profile: crates/orchestrator/src/context_budget.rs:697 (the B-10
      bootstrap tier detect; the B-10 label itself is PLAN-asserted, the symbol is VERIFIED).
HW-6  ModelCapabilityProfile: context_budget.rs:284 — profiles micro_3b / micro_7b / micro_8gb /
      large_32b / cloud_200k.
HW-7  ModelPool trait + can_fit_llm(model_size_bytes): crates/orchestrator/src/agent_pool.rs:48/62.
HW-8  research_config_for_tier: crates/orchestrator/src/research_tier.rs:50 (C-30's
      [research.vram_tiers]; coding/research lane only — no vision/OCR/embedding lane anywhere).
HW-9  BackendMode + BackendConfig: crates/cli/src/runtime_builder.rs:31 (B-37 RuntimeAdapter seam;
      the "11 backends" count is PLAN-asserted — re-grep at edit).
HW-10 NO HardwareModelCatalog / model-eligibility surface exists (grep 2026-06-11: zero hits for
      catalog/eligibility symbols). All HW-1..HW-9 are INTERNAL routing inputs; nothing user-facing
      lists what the box can run. J-9 is NET-NEW BUILD wrapping them.
HW-11 E-51 MLX/llamacpp symbols: NOT FOUND in .elai_cc — E-51 is PLANNED. J-9's MLX lane coverage
      is PLAN-asserted; the catalog ships lanes for backends that exist, with E-51 lanes as
      data-ready placeholders.
HW-12 odysseus prior art: docs/.source/odysseus-main services/hwfit/fit.py — rank_models:521,
      analyze_model:299, _estimate_speed:73, _quality_score:143, _fit_score:198 (anchors per
      docs/.source/odysseus-INDEX.md:64). Pattern: GPU detect -> model size estimate -> weighted
      quality+speed fit score -> ranked list. J-9 mirrors the STRUCTURE (lane-per-capability,
      fit-scored ranking); the weights are odysseus' own — ELAI derives its own from measured
      fixtures (GD-5).
HW-13 E-36 OCR/vision-RAG lane anchors: crates/knowledge/src/chunking/image_artifact.rs:45 —
      ColQwen2.5-v0.2 / PaddleOCR-VL named in the shipped chunking surface (the E-36 leg's
      substrate is partially in-tree; plan status stays PLANNED — re-grep scope at edit). This is
      the lane J-8's VR-8/VR-9 EVALUATE candidates measure against; canonical until measured.
HW-14 policy shift (user-stated 2026-06-11, carried verbatim): the Gemma 4 and Qwen3-32B
      rejections (metrics/baseline_findings_and_actions.md Part 1) were USER-HARDWARE-SPECIFIC
      (context-extension limits on the user's own box), not universal verdicts. J-9 lifts them to
      TIER-CONDITIONAL ELIGIBILITY at CATALOG level only: the catalog may list them for boxes
      that fit them, with the prior-rejection context carried as an honest-residue note. NO
      routing code, allowlist, or CLAUDE.md default changes until the user separately confirms
      (DC-3).

================================================================================
JC — BINDING CORRECTIONS (extends the JC family from J-HARNESS-GROUNDING-MAP)
================================================================================
JC-7  dots.ocr: no verified pypi/HF package name at authoring. Anti-slopsquat BEFORE any
      reference in code or pyproject: verify the exact HF repo / pypi name + publish date +
      license; until then it exists only as an EVALUATE row in the J-8 eval harness config.
JC-8  Qwen3-VL-Embedding / Qwen3-VL-Reranker: HF model-card names verify at edit; same
      EVALUATE-only status as JC-7.
JC-9  FastVLM is on-device (Apple silicon); irrelevant to the remote-3090 sidecar. It enters the
      J-9 catalog DATA (vision lane, apple-silicon platform tag) and nothing else; no MLX code
      ships before E-51.
JC-10 the J-9 catalog is INFORMATIONAL-ONLY: it emits a structured eligibility list (SSB +
      Tier-2 UI). It never overrides routing, never auto-installs, never flips a model pin.
      Catalog->routing promotion is a separate, user-gated future session.
JC-11 SSB deltas: J-8 +VlmPerceptionUpgradeEvent (Pattern-2 single-variant; Kind: SchemaApplied /
      ResizeRuleApplied / RouteRuleApplied / EvalScheduled), baseline+1 — with the J-2-style
      prefer-reuse alternative: if J-1's PerceptionScaffoldEvent payload ships a carrier field
      (applied_rules), extend THAT instead and add no action_type (decide at edit). J-9
      +HardwareModelCatalogEvent (Pattern-2 single-variant; Kind: CatalogBuilt / TierAssigned /
      EligibilityChecked / LaneSeeded), baseline+1. Both full 6-part variant discipline.

================================================================================
CANONICAL CONSTANTS (LIVE at grep 2026-06-11 — NEVER hard-code; re-read at edit, W-9)
================================================================================
SCHEMA_VERSION = 226 (crates/orchestrator/src/ssb.rs:3028)
TOTAL_VARIANTS = 537 (crates/orchestrator/tests/action_type_lock.rs:641)
The J-HARNESS map's 221/532 (2026-06-10) is already stale one day later — the RT-1 staleness
history holds. Detail files quote "baseline+1 from LIVE 226/537 as of 2026-06-11; re-read at edit".

================================================================================
WEB VERIFICATION (2026-06-11; ids re-verified by executor at execution per house rule)
================================================================================
arXiv:2403.07508 MoAI (KAIST) — confirmed (huggingface.co/papers).
arXiv:2504.10479 InternVL3 — confirmed (arxiv.org).
arXiv:2505.07062 Seed1.5-VL — confirmed (arxiv.org; catalog data only, ByteDance cloud — no ELAI lane).
arXiv:2603.12823 AVR adaptive VLM routing — confirmed (arxiv.org).
arXiv:2505.15146 lmgame-Bench (ICLR 2026; perception scaffold ~0->40% on game tasks) — confirmed.
dots.ocr — vendor blog only (getomni.ai), NOT peer-reviewed; [DESIGN] weight (VR-8, JC-7).
FastVLM — apple/ml-fastvlm github + CVPR 2025 — confirmed (JC-9).

================================================================================
DC — DESIGN-DOC CORRECTIONS (activation-vs-build reality + scope boundaries)
================================================================================
DC-1  J-8 is an UPGRADE session on J-1's gateway: WAVE 3+, MUST NOT execute before J-1 is DONE
      (J-1 owns the gateway structs J-8 upgrades). J-8 carries deltas that touch B-24/E-36/H-14/
      E-51 SURFACES as compose-pointers and eval rows; it NEVER edits those plan files (user hard
      constraint 2026-06-11).
DC-2  J-9 splits W-10-style: Tier-1 SHIP NOW = probe->catalog builder + SSB emit (model-free,
      tests against shipped system_probe fixtures immediately); Tier-2 DEFER = `elai
      hardware-catalog` CLI subcommand + dashboard panel (the frontend_pair lands there).
DC-3  the HW-14 policy shift changes CATALOG DATA only. metrics/baseline_findings_and_actions.md
      stays untouched as the historical record; CLAUDE.md model defaults stay untouched; any
      routing/allowlist change is a separate user-confirmed session.

End of J-PERCEPTION-GROUNDING-MAP.
