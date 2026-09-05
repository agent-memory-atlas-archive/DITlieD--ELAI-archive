# DEEP-J-006 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L960-L975`, heading `### DEEP-J-006`; the next chunk starts at L977.
- Declared chunk: family J, **10 files**, **1,223 logical lines**, **149,753 bytes**, one auxiliary/grounding + nine numbered originals; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all ten listed sources were read completely, first through last logical line. No original assigned to another deep chunk was opened. `10-J-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact for the relevant J-8/J-9/J-11/J-13/J-15/J-17/J-77/J-83/J-98/J-102/J-111 blocks plus composed-workflow and inventory lines; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `PGM:Lx-Ly`, `J-11:Lx-Ly`, `J-13:Lx-Ly`, `J-15:Lx-Ly`, `J-17:Lx-Ly`, `J-77:Lx-Ly`, `J-83:Lx-Ly`, `J-98:Lx-Ly`, `J-102:Lx-Ly`, and `J-111:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| auxiliary/grounding/fixture | 170 | 13,623 | ffeab5ab6d32a1b22f820d822b27928e2c32b258b43fb61cfcb3c76c0c2d2e15 | docs/thefinalglance/J-plan/_grounding/J-PERCEPTION-GROUNDING-MAP.md | exact |
| numbered/index original | 252 | 19,833 | a92b5a3aa97e0927aaa41e6b6f116cb53eb4471cc2398e5bf39894aae4b4d899 | docs/thefinalglance/J-plan/J-11_arbor_steal_intake_closeout.md | exact |
| numbered/index original | 69 | 8,181 | b06877268151b696e3991341488089503919973cb300d17229c18ec148028f2c | docs/thefinalglance/J-plan/J-13_local_lane_token_determinism.md | exact |
| numbered/index original | 58 | 7,173 | f764dbb3f13b7c4137f3c451aa5a53f41274f904294c76286fe9018a99fce4e3 | docs/thefinalglance/J-plan/J-15_determinism_steal_amendments.md | exact |
| numbered/index original | 73 | 10,771 | 8bc686eaa2a1a829b97a8fad03d30e2268b644c37b14468cf1c4794a8c961b3c | docs/thefinalglance/J-plan/J-17_plan_level_compositional_safety.md | exact |
| numbered/index original | 90 | 16,227 | d64fc105d2f13d4661258375d150a47e683d53dc626e623c8894f5524b857287 | docs/thefinalglance/J-plan/J-77_perception_faithfulness_trust_spine.md | exact |
| numbered/index original | 121 | 19,114 | 523bc77737116f97b33c7d9c9bcc4977b88106a44d86024be430bc0373f42dd9 | docs/thefinalglance/J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md | exact |
| numbered/index original | 114 | 16,451 | 0069c60ec5bab56eeca9887b39ae43ba4066af67390c8cbd7a73d2a9006117c5 | docs/thefinalglance/J-plan/J-98_canvas_case_memory_altitude.md | exact |
| numbered/index original | 127 | 22,540 | 37b04877d6e92c0afc41543fe046249a24e35ac54b47b679e96a2f752d9de940 | docs/thefinalglance/J-plan/J-102_config_ingress_admission_gate.md | exact |
| numbered/index original | 149 | 15,840 | 900bfa925f6c7711cada1160227e74d0975effedd4bc90f4c5b1d0fbaf056743 | docs/thefinalglance/J-plan/J-111_voice_ingress_spoken_confirm.md | exact |
| **Aggregate** | **1,223** | **149,753** | — | **10 exact paths** | **exact** |

## Per-file atomic ledgers

### PGM — J-PERCEPTION-GROUNDING-MAP (J-8 + J-9 grounding)

Source: `docs/thefinalglance/J-plan/_grounding/J-PERCEPTION-GROUNDING-MAP.md` (170/170 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| PGM-E001 | E/STATUS | Map grounds J-8 (VLM perception upgrade) and J-9 (hardware model catalog), authored 2026-06-11 by session s-j8j9-author. | PGM:L1-L2 |
| PGM-E002 | E/EVIDENCE | Method: full read of 3-file VLM research corpus in Downloads + condensed summary, live greps over `.elai_cc`, odysseus-INDEX pull, and web verification of every arXiv id. | PGM:L2-L6 |
| PGM-E003 | E/DATA | All corpus figures are EXTERNAL `[DESIGN]` per HD-1/ID-5: design rationale only; every shipped number is ELAI-measured. | PGM:L17-L19 |
| PGM-E004 | E/CFG | VR-1 resize sweet spots (longest side): Claude 1568px / GPT-4o 2048px high-detail / Qwen-VL-family 1280px / Gemini native up to ~3K; when target API supports native resolution do NOT force a fixed size. | PGM:L21-L25 |
| PGM-E005 | E/DATA | VR-2 normalized coordinates: 0-1000-scale `[ymin,xmin,ymax,xmax]`, never raw pixels; cited error ~15% -> <2% with proper resize; J-1 done-gate already demands <2% on click-target fixture. | PGM:L26-L29 |
| PGM-E006 | E/LIMIT | VR-3: counting >10 small elements degrades on every model; counts must come from AXTree/DOM or a deterministic script, never the VLM; already harness law; J-8 ports into runtime composer. | PGM:L30-L32 |
| PGM-E007 | E/FLOW | VR-4 multi-image evidence integration degrades; mitigations: tiled-composite (InternVL style, VR-14) or B-24 Mode B specialist (InternVL3.5-38B via SGLang/RadixAttention). | PGM:L33-L36 |
| PGM-E008 | E/LIMIT | VR-4 honest residue: routing reduces, does not eliminate, multi-image degradation (GD-5). | PGM:L35-L36 |
| PGM-E009 | E/FAIL | VR-5 nested small-text UI stays brittle even with tiling; micro-text routes to tiling THEN typed refusal (D-16) when confidence is low, never a guess. | PGM:L37-L39 |
| PGM-E010 | E/EVIDENCE | VR-6 MoAI (arXiv:2403.07508): verbalizing specialist-CV outputs into VLM context beats lone VLM; design rationale for J-1 hybrid composer (AXTree/DOM text + image). | PGM:L40-L42 |
| PGM-E011 | E/OBS | VR-7 image prompt caching: ~90% savings on re-queried identical screenshots [DESIGN]; image blocks live AFTER last cache breakpoint per HD-2; I-10 prefix-invariance is the enforcement layer. | PGM:L43-L47 |
| PGM-E012 | E/DEFER | VR-8 dots.ocr (1.7B): vendor-blog claim only, NOT peer-reviewed; weight [DESIGN], EVALUATE-not-ship vs E-36 lane (ColQwen2.5 / PaddleOCR-VL); package/HF name unverified (JC-7). | PGM:L48-L51 |
| PGM-E013 | E/DEFER | VR-9 Qwen3-VL embedding + reranker models for vision RAG: EVALUATE vs ColQwen2.5 for E-36; HF names verify at edit (JC-8). | PGM:L52-L54 |
| PGM-E014 | E/SURFACE | VR-10 design-mockup playbook: structured component-inventory prompt emits `{type, verbatim text, hierarchy, layout cues}` before codegen; ported as J-8 library TEMPLATE; H-14 (PLANNED) consumes it; J-8 never edits H-14. | PGM:L55-L57 |
| PGM-E015 | E/DEFER | VR-11 FastVLM (Apple, CVPR 2025): ~85x faster TTFT at high res, on-device; relevant ONLY to future E-51 MLX lane, NOT remote-3090 sidecar; catalog note only, nothing shipped (JC-9). | PGM:L58-L60 |
| PGM-E016 | E/FLOW | VR-12 AVR adaptive routing (arXiv:2603.12823): probe small VLM first, escalate on low confidence; -78% cost within 2pts of always-large; safety-critical always to large model [DESIGN] for J-1 leg 4. | PGM:L61-L63 |
| PGM-E017 | E/CFG | VR-13 per-VRAM vision ladder [DESIGN]: sub-6B/8GB Phi-4-Multimodal; single 24GB Gemma 3 27B QAT or Qwen2.5-VL-7B; 48GB+ Qwen2.5-VL-32B/72B or InternVL3-78B; Cloud Anthropic Opus for document/chart/dense-text 200K multi-screenshot. Seeds J-9 vision lane + J-1 leg-4 probe. | PGM:L64-L69 |
| PGM-E018 | E/CAP | VR-14 InternVL3 (arXiv:2504.10479): native multimodal pretraining; dynamic-resolution tiling (overlapping tiles + global thumbnail) is the proven boost for fine text — J-1 leg-1 tiling shape. | PGM:L70-L72 |
| PGM-E019 | E/DATA | HW-1 `GpuInfo` at `system_probe.rs:80`: name, vram_total_mb, vram_free_mb. | PGM:L77-L77 |
| PGM-E020 | E/DATA | HW-2 `SystemProbeResult` at `system_probe.rs:127`: gpu, ram_total_mb, ram_free_mb, disk_free_mb (C-22 shipped). | PGM:L78-L79 |
| PGM-E021 | E/CAP | HW-3 `classify_vram_tier(total_mb) -> VramTier` at `system_probe.rs:145`. | PGM:L80-L80 |
| PGM-E022 | E/DATA | HW-4 `VramTier` enum at `vram_tier.rs:6`: Vram8Gb / Vram12Gb / Vram16Gb / Vram24Gb / Vram48Gb / Cloud. | PGM:L81-L82 |
| PGM-E023 | E/CAP | HW-5 `detect_model_capability_profile` at `context_budget.rs:697` (B-10 bootstrap; B-10 label PLAN-asserted, symbol VERIFIED). | PGM:L83-L84 |
| PGM-E024 | E/DATA | HW-6 `ModelCapabilityProfile` at `context_budget.rs:284`: micro_3b / micro_7b / micro_8gb / large_32b / cloud_200k. | PGM:L85-L86 |
| PGM-E025 | E/CAP | HW-7 `ModelPool` trait + `can_fit_llm(model_size_bytes)` at `agent_pool.rs:48/62`. | PGM:L87-L87 |
| PGM-E026 | E/CFG | HW-8 `research_config_for_tier` at `research_tier.rs:50` (`[research.vram_tiers]`); coding/research lane only — no vision/OCR/embedding lane anywhere. | PGM:L88-L89 |
| PGM-E027 | E/CAP | HW-9 `BackendMode` + `BackendConfig` at `runtime_builder.rs:31` (B-37 RuntimeAdapter); "11 backends" count PLAN-asserted. | PGM:L90-L91 |
| PGM-E028 | E/STATUS | HW-10: NO HardwareModelCatalog / model-eligibility surface exists (grep 2026-06-11 zero hits); HW-1..HW-9 are INTERNAL routing inputs; J-9 is NET-NEW BUILD wrapping them. | PGM:L92-L94 |
| PGM-E029 | E/DEP | HW-11 E-51 MLX/llamacpp symbols NOT FOUND; J-9 MLX lane coverage PLAN-asserted as data-ready placeholders. | PGM:L95-L97 |
| PGM-E030 | E/EVIDENCE | HW-12 odysseus prior art `services/hwfit/fit.py`: rank_models:521, analyze_model:299, _estimate_speed:73, _quality_score:143, _fit_score:198; J-9 mirrors STRUCTURE (lane-per-capability, fit-scored ranking); weights are ELAI-measured (GD-5). | PGM:L98-L103 |
| PGM-E031 | E/DEP | HW-13 E-36 OCR/vision-RAG anchors: `image_artifact.rs:45` names ColQwen2.5-v0.2 / PaddleOCR-VL; substrate partially in-tree; plan status stays PLANNED; canonical until measured for VR-8/VR-9 EVALUATE. | PGM:L104-L107 |
| PGM-E032 | E/CFG | HW-14 policy shift (user-stated 2026-06-11): Gemma 4 and Qwen3-32B rejections were USER-HARDWARE-SPECIFIC; J-9 lifts them to TIER-CONDITIONAL ELIGIBILITY at CATALOG level only with honest-residue note. | PGM:L108-L114 |
| PGM-E033 | E/LIMIT | HW-14 / DC-3: NO routing code, allowlist, or CLAUDE.md default changes until user separately confirms; `metrics/baseline_findings_and_actions.md` stays historical record. | PGM:L112-L114, L166-L168 |
| PGM-E034 | E/SEC | JC-7: dots.ocr has no verified pypi/HF package name; anti-slopsquat BEFORE any code/pyproject reference; until then EVALUATE row only in J-8 eval harness config. | PGM:L119-L121 |
| PGM-E035 | E/SEC | JC-8: Qwen3-VL-Embedding / Qwen3-VL-Reranker HF names verify at edit; same EVALUATE-only status. | PGM:L122-L123 |
| PGM-E036 | E/LIMIT | JC-9: FastVLM enters J-9 catalog DATA (vision lane, apple-silicon platform tag) only; no MLX code ships before E-51. | PGM:L124-L126 |
| PGM-E037 | E/SEC | JC-10: J-9 catalog is INFORMATIONAL-ONLY: emits structured eligibility list (SSB + Tier-2 UI); never overrides routing, never auto-installs, never flips a model pin; Catalog->routing promotion is separate user-gated session. | PGM:L127-L129 |
| PGM-E038 | E/EVENT | JC-11: J-8 +`VlmPerceptionUpgradeEvent` Pattern-2 single-variant Kind SchemaApplied / ResizeRuleApplied / RouteRuleApplied / EvalScheduled, baseline+1; prefer-reuse alternative: if J-1 PerceptionScaffoldEvent ships carrier field `applied_rules`, extend THAT instead (net 0). | PGM:L130-L133 |
| PGM-E039 | E/EVENT | JC-11: J-9 +`HardwareModelCatalogEvent` Pattern-2 single-variant Kind CatalogBuilt / TierAssigned / EligibilityChecked / LaneSeeded, baseline+1; both full 6-part variant discipline. | PGM:L133-L135 |
| PGM-E040 | E/DATA | Live constants 2026-06-11: SCHEMA_VERSION=226 (`ssb.rs:3028`), TOTAL_VARIANTS=537 (`action_type_lock.rs:641`); J-HARNESS map 221/532 already stale; detail files quote "baseline+1 from LIVE 226/537; re-read at edit". | PGM:L140-L143 |
| PGM-E041 | E/EVIDENCE | Web-verified 2026-06-11: arXiv 2403.07508 MoAI, 2504.10479 InternVL3, 2505.07062 Seed1.5-VL (catalog only, no ELAI lane), 2603.12823 AVR, 2505.15146 lmgame-Bench; dots.ocr vendor only; FastVLM apple/ml-fastvlm confirmed. | PGM:L148-L154 |
| PGM-E042 | E/DEP | DC-1: J-8 is UPGRADE on J-1 gateway; WAVE 3+; MUST NOT execute before J-1 DONE; carries compose-pointers/eval rows to B-24/E-36/H-14/E-51 surfaces; NEVER edits those plan files (user hard constraint 2026-06-11). | PGM:L159-L162 |
| PGM-E043 | E/DEFER | DC-2: J-9 splits W-10-style: Tier-1 SHIP NOW = probe->catalog builder + SSB emit (model-free, tests against system_probe fixtures); Tier-2 DEFER = `elai hardware-catalog` CLI + dashboard panel. | PGM:L163-L165 |
| PGM-E044 | E/LIMIT | Hardware family named HW (not HD); HD is already H-plan delta family; reusing would corrupt cross-references. | PGM:L7-L9 |
| PGM-A001 | A/CFG | Exact package/HF names for dots.ocr and Qwen3-VL embedding/reranker remain unverified at map authoring. | PGM:L48-L54, L119-L123 |
| PGM-A002 | A/EVENT | Whether J-8 adds VlmPerceptionUpgradeEvent or extends PerceptionScaffoldEvent.applied_rules is deferred to edit-time prefer-reuse decision. | PGM:L130-L133 |

Category coverage: capabilities/surfaces ✓ (catalog CLI/panel deferred Tier-2; mockup template; no standalone privacy policy); workflows/states ✓; schemas/data ✓; flags/defaults/config ✓; integrations/events ✓; security/permissions ✓ (informational catalog, anti-slopsquat); failures/refusal ✓; determinism ✓ via measured-only numbers; observability/evidence ✓; tests/done gates ✓ (via J-1/J-8/J-9 consumers); constraints/non-goals ✓; dependencies/status/deferred ✓. Retry/idempotency APIs absent from this map.

### J-11 — arbor / combench / denovoswe steal intake closeout

Source: `docs/thefinalglance/J-plan/J-11_arbor_steal_intake_closeout.md` (252/252 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J11-E001 | E/STATUS | J-11 is PLANNED WAVE 2 EXTENSION discharging steal-list S-2/S-3/S-4/S-5/S-7/S-8 as amendments into existing I/H/D plan files; zero SSB, zero runtime code; direct analog of J-7. | J-11:L1-L4 |
| J11-E002 | E/CAP | Objective: one gated documentation unit; each target gains source-file pointers + verified mechanics + two Arbor FAIL-OPENS ELAI must refuse; hard boundary: INSERTS only, never re-authors, never writes runtime code (W-3). | J-11:L6-L13 |
| J11-E003 | E/STATUS | Effort ~1 session (4-6 h); ~6 amendment insertions; no code, no SSB, no build; pre-requisites none; parallel-safe with all J sessions (disjoint plan-file targets). | J-11:L15-L17 |
| J11-E004 | E/DEP | Targets: I-2, I-4, I-17, I-25, I-30, D-6, D-11, H-43, H-47, H-50; none currently cite Arbor/ComBench/DeNovoSWE sources; statuses STALE-by-construction in summary indexes (SRC-4). | J-11:L31-L35 |
| J11-E005 | E/FLOW | S-2 held-out merge-gate into I-4: gate itself re-runs held-out eval (B_test) on candidate branch in fresh DETACHED worktree off-agent; executor only ever sees B_dev; LLM-reported scores logged but NEVER trusted. | J-11:L67-L72 |
| J11-E006 | E/FAIL | No improvement vs trunk/baseline (metric_direction-aware) => reject; merge --no-ff; conflict => abort+restore; protected_paths fnmatch guard; required_outputs checked (git_ops.py:282-520). | J-11:L70-L72 |
| J11-E007 | E/SEC | DO-NOT-COPY F/O-1 (git_ops.py:413): eval_cmd_test unset => Arbor falls back to LLM-reported score; ELAI: absent held-out cmd = BLOCK (R1). | J-11:L72-L74 |
| J11-E008 | E/SEC | DO-NOT-COPY F/O-2: Arbor eval commands live in MUTABLE tree.meta; ELAI: eval cmds in FROZEN gate-manifest, never agent-writable state (R8). | J-11:L75-L76 |
| J11-E009 | E/DET | Score extraction: Arbor LLM fallback `_parse_eval_score:62`; ELAI: deterministic parser ONLY (TD-3, LLM never the gate). | J-11:L77-L78 |
| J11-E010 | E/DEP | GD-1 prior-art pointers: H-43 (Arbor existence proof of convergence), D-6 (B_dev/B_test = temporal split), I-17 (L1 veto supremacy). | J-11:L79-L81 |
| J11-E011 | E/STATE | S-3 worktree lifecycle into I-2 + I-4 (5 mechanics): (1) branch start = CURRENT trunk HEAD; (2) finalize-commit filters generated artifacts; (3) worktree removed but BRANCH PRESERVED; (4) merge-eval in separate detached worktree pool; (5) stale force-removed, collision => timestamp suffix. | J-11:L82-L89 |
| J11-E012 | E/LIMIT | Arbor has NO pooling and is Unix-only; ELAI I-2 warm pooled Windows-safe worktrees (IC-9) are an IMPROVEMENT — keep them. | J-11:L87-L89 |
| J11-E013 | E/LIMIT | S-4 prompt content: DO-NOT-PORT persistent-ReAct, 500-turn loop, premature-stop nudges, LLM-chosen node selection (M-1); ELAI FSM + deterministic leaf selection replaces them. | J-11:L90-L93 |
| J11-E014 | E/SURFACE | DO-PORT as advisory prompt text WITH source cite: prompts.py:396 SELECT => I-4 ranker tie-break; prompts.py:442 Evaluation Discipline => executor/eval prompt; idea_drafting.md IDEATE => babystep-author/scope-planner CLOUD lane ONLY; executor/prompts.py UNDERSTAND..REPORT => steal deltas only. | J-11:L93-L98 |
| J11-E015 | E/DEP | do-not-re-propose-same-hidden-assumption wording already carried by J-10 CONSTRAINTS BLOCK — cross-reference, do not duplicate. | J-11:L98-L100 |
| J11-E016 | E/CAP | S-5 token-efficiency into H-50: Arbor ContextManager layers 1-3 (truncate-15K / snip-old-tool-results / drop-redundant-reads of SAME file) as prior art; L3 redundant-READ dedup is distinct from H-6 turn-masking. | J-11:L101-L104 |
| J11-E017 | E/LIMIT | Layer-4 (20K LLM-summarize) EXPLICITLY REJECTED at 32K local ceiling; H-51 topic-digest + I-14 deterministic compression replace it. | J-11:L104-L106 |
| J11-E018 | E/CAP | S-7 ComBench checker contract into D-11: verifier = self-contained script, payload on stdin, prints "True" on pass; answer-format in PROMPT not harness; extraction via exactly-one `<construct>` regex; proof and construction parsed INDEPENDENTLY; per-component result dict kept separate; 4-string judge rubric N in {0,1,6,7}, parse failure = error never guessed score. | J-11:L109-L117 |
| J11-E019 | E/TEST | NET-NEW register-time gate: at checker REGISTRATION run checker against reference fixture; register FAILS CLOSED (ComBench check_ref_construction.py analog). | J-11:L117-L120 |
| J11-E020 | E/LIMIT | DO-NOT-COPY: reliability_guard "NOT a security sandbox" (ELAI uses Tier-C); stdout=="True" string protocol (ELAI: JSON CheckerResult); ast.literal_eval-only payloads (ELAI: JSON+schema); no checker versioning (ELAI: blake3 content-address). | J-11:L120-L123 |
| J11-E021 | E/STATE | EXPLICIT DEMOTION (M-2: ComBench averages, never demotes): D-11 must implement demotion — construction-fail demotes advisory-judge proof score monotonically, final<=raw; generalizes to any prose+checkable-witness gate (HD-5/VD-1). | J-11:L124-L128 |
| J11-E022 | E/CFG | S-8a I-30: adaptive difficulty-banded thresholds bands [0,.2)/.9 [.2,.4)/.85 [.4,.6)/.8 [.6,.8)/.7 [.8,1]/.6; no global pass/fail on SFT exhaust; PARTIAL on hard task worth more than PASS on trivial. | J-11:L129-L133 |
| J11-E023 | E/DATA | S-8b I-25+I-30: difficulty fusion d = w_s*struct + w_g*judge1 + w_q*judge2; weights grid-searched on simplex; ELAI calibrates on OWN rollouts; structural signal needs domain substitute (SCIP function count / cross-lang bindings / SSB variants). | J-11:L134-L138 |
| J11-E024 | E/SEC | S-8c H-47+D-6 leakage: static command-blocking alone insufficient; two-layer = block + LLM trace audit; bypass modes: git clone of origin, pip install/download via pypi fuzzy match, curl/wget of source, git reflog + loose-object recovery (re-init .git entirely). | J-11:L139-L144 |
| J11-E025 | E/DEFER | S-8d DCR doc loop DECLINED (offline-only, no session); M-3 caveats: SFT-only, 131K context => cloud-lane-only for trajectory training, loss-masking unablated. | J-11:L145-L147 |
| J11-E026 | E/LIMIT | S-6 parallel HTR subsystem explicitly not built; WHAT ARBOR LACKS positioning is not a forced write unless a strategy doc is touched. | J-11:L22-L23, L148-L152, L243-L244 |
| J11-E027 | E/DATA | Decision doc: `workflow/reasoning/J-11-decisions.md` (targets amended, verified status, two fail-opens, demotion composition, declines). | J-11:L153-L154, L161-L161 |
| J11-E028 | E/SURFACE | frontend_pair N/A — plan-file amendments; justified (no runtime surface). | J-11:L177-L178 |
| J11-E029 | E/TEST | Acceptance: every amendment names real source file:line (W-3); both fail-opens as DO-NOT-COPY into I-4 mapped to R1/R8; D-11 register-time gate + demotion; I-30 banding + partial-over-trivial + structural substitute; H-47 four leakage modes + two-layer; D-6 .git re-init; no runtime/SSB/re-author; external figures design-rationale only (HD-1/ID-5). | J-11:L200-L213 |
| J11-E030 | E/TEST | DETERMINISTIC_CHECK greps are the test (JD-1); negative arm: uncited amendment or fail-open-as-pattern fails review. | J-11:L188-L218 |
| J11-E031 | E/INT | I-4 carries three separately-anchored blocks (merge-gate / worktree / prompt-content) so each is independently grep-checkable. | J-11:L241-L242 |
| J11-E032 | E/IDEM | Target-file status drift: verify each at edit; skip+note any target already carrying the amendment (idempotent). | J-11:L238-L240 |
| J11-I001 | I/DEP | Consuming sessions (I-4 merge, D-11 checker, I-30 SFT exhaust, H-47 firewall) are the production readers of amendments; J-11 itself has no runtime reachability. | J-11:L166-L169, L246-L250 |
| J11-A001 | A/STATUS | Whether each target already carries the amendment is unresolved until edit-time git log + FINISH master index. | J-11:L238-L240 |

Category coverage: capabilities/actions ✓ (doc amendments); surfaces N/A justified; workflows/states ✓; schemas/data ✓; config/defaults ✓; integrations/events (zero SSB) ✓; security/permissions ✓; failures/demotion ✓; determinism ✓; observability via grep evidence ✓; tests/acceptance ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No privacy/retry/concurrency runtime API.

### J-13 — local-lane token determinism

Source: `docs/thefinalglance/J-plan/J-13_local_lane_token_determinism.md` (69/69 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J13-E001 | E/STATUS | J-13 is BUILD, lever PROVABLE, grounding STRONG, effort M, WAVE 3 (fixture legs now; live GPU legs remote-sidecar/RunPod-gated per W-10); depends none. | J-13:L1-L3 |
| J13-E002 | E/CAP | Sidecar: vendor/pin batch_invariant_ops alpha v0.1.0 (pin exact commit; install from local clone, never registry guess) + wrap generate path in set_batch_invariant_mode ONCE, never nested (destroy-on-inner-exit bug); Triton warm-up at startup; assert bf16 log_softmax path. | J-13:L7-L8 |
| J13-E003 | E/CAP | Sidecar: SeededGumbelProcessor as LogitsProcessor in existing chain (DoLaSteeringProcessor:786 / ETCLogitsProcessor:990 precedent) implementing multinomial_with_seed; active only when request carries a seed. | J-13:L9-L9 |
| J13-E004 | E/FLOW | Rust: thread sampling_seed from ELAI_SEED through request struct (flip `runtime_client.rs:597` from None) and RECORD it in RRA bundle so replay re-issues the same seed. | J-13:L10-L10, L15-L18 |
| J13-E005 | E/CFG | E-26 SGLang adapter det-mode requirements (--enable-deterministic-inference + per-request sampling_seed + FlashInfer/FA3/Triton backend constraint) written as capability check, not prose. | J-13:L11-L11 |
| J13-E006 | E/CFG | Determinism mode is kill-switch-shaped (R-f): ON for `elai run --emit-rra` local-lane by default ONLY after GD-4 overhead bound measured (~20% matmul reported upstream; 3090 Triton fallback unmeasured); until then opt-in via run config + wire-dark-allowlist row + promotion plan. | J-13:L12-L12 |
| J13-E007 | E/LIMIT | EXCLUDED: any attention-kernel patch; LayerCast/TBIK (TP=1 single 3090 — rejected); determinism CLAIM on cloud lane (provider-side, impossible). | J-13:L13-L13 |
| J13-E008 | E/LIMIT | HONEST RESIDUE: attention (SDPA/FA2) is NOT covered by ops library; full cross-batch token identity needs greedy or batch_size=1 at attention; ships as measured opt-in mode, never blanket guarantee. | J-13:L4-L5 |
| J13-E009 | E/STATUS | Live ground truth 2026-06-11: `runtime_client.rs:596-597 stream:true, seed:None`; server.py LogitsProcessor chain; rra.rs verify_rra:474 / verify_artifact:481; sidecar REMOTE 3090 only; python 3.10+ required. | J-13:L15-L20 |
| J13-E010 | E/INT | Integration: `python/elai_inference/server.py` (~120 LOC) + vendored batch_invariant_ops; `runtime_client.rs` (~20 LOC); `rra.rs` (~30 LOC seed field + replay). | J-13:L22-L23 |
| J13-E011 | E/EVENT | +LocalDeterminismEvent Pattern-2 single-variant Kind BatchInvariantEnabled / SeededSamplingApplied / DeterminismProbeRun / DeterminismDivergence; baseline+1 from LIVE 229/540; full 6-part discipline. | J-13:L24-L24 |
| J13-E012 | E/SURFACE | frontend_pair: determinism status line on existing runtime/cloud panel (decide at edit; W-4) or W-6-justified absence. | J-13:L25-L25, L46-L46 |
| J13-E013 | E/INT | cross_lang pair: sidecar route change + rust client change ship together (cross_lang_diff.py). | J-13:L26-L26 |
| J13-E014 | E/INT | Production reachability: `elai run --local` -> runtime_client.rs:596 -> sidecar /v1/messages; test/contracts callers never satisfy R7. | J-13:L37-L39 |
| J13-E015 | E/LIMIT | Stub bans: NO nested set_batch_invariant_mode; NO determinism claim while mode off or attention residue applies; NO registry-resolved install; NO new default-OFF ELAI_*_ENABLE without allowlist+promotion; NO cloud-lane determinism claims. | J-13:L40-L45 |
| J13-E016 | E/TEST | Trigger: two consecutive `elai run --local --emit-rra` with determinism mode on and same ELAI_SEED; expect BatchInvariantEnabled + SeededSamplingApplied + DeterminismProbeRun with identical blake3 over token streams; both RRA bundles same sampling_seed; `elai replay --verify` matches. | J-13:L47-L50 |
| J13-E017 | E/FAIL | NEGATIVE: mode off -> capability echo says so and DeterminismProbeRun records non-identical streams as DeterminismDivergence (measured, not hidden); seed-less request with mode on FAILS contract test. | J-13:L50-L50 |
| J13-E018 | E/OBS | Capability echo must state exactly what is covered. | J-13:L41-L42 |
| J13-E019 | E/DEP | Composes G-2 RRA, G-10 model-drift canary, E-26 SGLang, E-2/E-28 sidecar perf, H-50 context policy. | J-13:L58-L59 |
| J13-A001 | A/OBS | 3090 Triton-fallback overhead UNMEASURED; GD-4 bound (>=3 real runs) decides default-ON vs opt-in. | J-13:L61-L62 |
| J13-A002 | A/TEST | NF4 interplay verdict is source-reasoned, not yet probe-confirmed; startup probe is first live checkbox. | J-13:L63-L63 |

Category coverage: capabilities/CLI/surfaces ✓; workflows ✓; data/seed field ✓; flags/config ✓; events ✓; security (anti-slopsquat pin) ✓; failures/divergence ✓; determinism/idempotency ✓; observability ✓; tests/acceptance ✓; constraints ✓; deps/deferred ✓. No privacy policy; concurrency limited to single wrap rule.

### J-15 — determinism steal closeout amendments

Source: `docs/thefinalglance/J-plan/J-15_determinism_steal_amendments.md` (58/58 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J15-E001 | E/STATUS | J-15 is EXTENSION, lever AUDITABLE, effort S, WAVE 2; zero SSB, zero runtime code; J-11 analog for 50-repo determinism triage. | J-15:L1-L5 |
| J15-E002 | E/CAP | Amend I-34: ADD llguidance crates.io v1.7.6 (MIT, 525K dl, anti-slopsquat verified 2026-06-11) as orchestrator-side grammar compile+validate+blake3-pin engine via ParserFactory at plan time; sidecar XGrammar UNCHANGED for inference masking (Option A). | J-15:L8-L8 |
| J15-E003 | E/LIMIT | Option B full Rust-side masking REJECTED (requires sidecar inference-loop redesign); compiled parser state NOT serializable — recompile from spec each boot; xgrammar-rs rejected (C++ binding, 5.6K dl). | J-15:L8-L8 |
| J15-E004 | E/SEC | NEW DEP: requires explicit user confirmation at I-34 execution (house rule); amendment records verification snapshot so executor re-verifies. | J-15:L8-L8, L53-L53 |
| J15-E005 | E/EVENT | Amend E-31/H-7: recorded-escalation-hops idiom — every attempted+failed backend hop journals its own row, not just final RouteDecisionEvent; prefer-reuse existing RouteDecisionEvent kinds. | J-15:L9-L9 |
| J15-E006 | E/CAP | Amend J-2: (a) script-step-with-exit-code-routing + LLM-free set-step as BSP node shapes if J-2 grows workflow verbs; (b) MCP tool-config DELEGATION to host runtime (never embed MCP server config in elai workflow definitions); intentusnet transparent-proxy shape note for recording leg. | J-15:L10-L10 |
| J15-E007 | E/RETRY | Amend D-11: Logic-LLM solver-error-feedback loop — on checker FAIL, solver error fed back for ONE bounded reformulation pass before claim UNGROUNDED; checker verdict stays load-bearing (TD-3). | J-15:L11-L11 |
| J15-E008 | E/STATE | Amend H-49: (a) event_id-anchored fork points (never positions); (b) mutate-then-replay affordance as adversarial-testing rider (pairs with J-14 fixture format). | J-15:L12-L12 |
| J15-E009 | E/DATA | Amend I-49: fixtures come from J-14 `rra export-fixture` format; matching key (model + message-content-blake3 + tool_name), NEVER positional. | J-15:L13-L13 |
| J15-E010 | E/SEC | DO-NOT-COPY exhibits: culpa stores auth headers verbatim in session JSON (J-14 scrub is counter-design); culpa/evalcraft positional stub-matching is brittle; Arbor-class fail-opens already recorded by J-11. | J-15:L14-L14 |
| J15-E011 | E/DATA | REJECTED/ALREADY-TRIED ledger upserts (~40 triage rejections) into feature map Section 4 so no future steal session re-litigates; includes empty eth-sri/domino, LayerCast/TBIK, Anandb71/arbor name-collision (not ledger Arbor). | J-15:L15-L15 |
| J15-E012 | E/DATA | Artifacts: amendment blocks + `workflow/reasoning/J-15-decisions.md` + feature-map ledger upserts; NO new action_type. | J-15:L21-L24, L27-L29 |
| J15-E013 | E/SURFACE | frontend_pair N/A — doc unit; justified. | J-15:L24-L24 |
| J15-E014 | E/TEST | Grep each target for amendment marker; each block source-cited; I-34 must carry dep-confirmation flag; uncited block or missing new-dep flag FAILS review. | J-15:L39-L41 |
| J15-E015 | E/LIMIT | Stub bans: NO uncited amendment; NO re-authoring target scope; NO presenting DO-NOT-COPY as pattern to copy. | J-15:L35-L38 |
| J15-A001 | A/DEP | E-31 vs H-7 as hop-recording home: pick the one whose executor lands first; note in J-15-decisions.md. | J-15:L51-L52 |
| J15-A002 | A/SEC | I-34 llguidance dep stays PENDING-USER-CONFIRMATION until that session executes. | J-15:L53-L53 |

Category coverage: capabilities via amendments ✓; surfaces N/A; workflows/states ✓; data/schema ✓; config/deps ✓; security/DO-NOT-COPY ✓; failures/retry ✓; determinism via pins/fixtures ✓; tests/review gates ✓; constraints ✓; deps/deferred ✓. Zero runtime events.

### J-17 — plan-level compositional safety

Source: `docs/thefinalglance/J-plan/J-17_plan_level_compositional_safety.md` (73/73 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J17-E001 | E/STATUS | J-17 is BUILD, lever CANT-DERAIL, grounding STRONG (SIF + SAFEFLOW + SandboxEscapeBench VERIFIED), effort M, WAVE 2; depends G-1 taint substrate (composes, does not rebuild). | J-17:L1-L3 |
| J17-E002 | E/CAP | Compositional policy check at concepts->phases expansion: assemble subtask DAG and evaluate policy predicate over WHOLE set (union of declared subtask effects / WIRING_CONTRACT produces/consumes + capability requests); reuses rules-engine COMPOSITE (3F) + G-1 taint graph. | J-17:L9-L10, L17-L19 |
| J17-E003 | E/STATE | Deterministic verdict ALLOW / FRAGMENTATION-FLAGGED / BLOCK. | J-17:L10-L10 |
| J17-E004 | E/FLOW | Check runs at PLAN plane BEFORE any subtask dispatches (SIF lesson: catching at execution is too late). | J-17:L11-L11 |
| J17-E005 | E/CFG | OBSERVE-ONLY first (GD-4 / JD-3 analog): flag fragmentation with offending subtask subset, do not block, until FP rate bounded on >=3 real plans; then BLOCK behind wire-dark-allowlist row. | J-17:L11-L11, L45-L45 |
| J17-E006 | E/CAP | Sandbox config-audit leg (model-free): deterministic auditor over remote sidecar container/sandbox config (Docker socket exposure, privileged flags, writable host mounts, cap set); known-bad config = surfaced finding. | J-17:L12-L12 |
| J17-E007 | E/DEFER | gVisor note Priority 2, NOT built here: document runsc as drop-in for Linux remote sidecar; Firecracker Q3-2026 and WASM/WASI ~2027 recorded, not authored. | J-17:L13-L13 |
| J17-E008 | E/LIMIT | EXCLUDED: full SAFEFLOW/IFC-everywhere port; any sandbox REPLACEMENT of Tier-C OS-process isolation; SIF 71% as a target. | J-17:L14-L14 |
| J17-E009 | E/EVIDENCE | SIF 71%-violation / 0%-FP-with-plan-IFC is N=14 enterprise scenarios — mechanism on POC sample, not production benchmark; done-gate produces ELAI-measured detection, never quotes 71%. | J-17:L7-L7 |
| J17-E010 | E/INT | Modules: `compositional_policy.rs` (~180 LOC: aggregate_subtask_capabilities + check_goal_policy); `sandbox_audit.rs` (~120 LOC: audit_sidecar_config). | J-17:L23-L25, L30-L33 |
| J17-E011 | E/EVENT | +CompositionalPolicyEvent Pattern-2 Kind PlanGraphChecked / FragmentationFlagged / SandboxConfigAudited / CompositionBlocked; baseline+1 from LIVE 231/542; prefer-reuse if G-1 capability-gate row already carries composition verdict (net 0). | J-17:L26-L26 |
| J17-E012 | E/SURFACE | frontend_pair: fragmentation finding on existing EvidenceGate / compliance panel (decide at edit; W-4) or W-6-justified absence. | J-17:L27-L27, L48-L48 |
| J17-E013 | E/INT | Production reachability: PLANNING -> phase-dispatch on real `elai` plan/orchestrate (pre-dispatch); sidecar boot/health for config-audit; test callers never satisfy R7. | J-17:L38-L41 |
| J17-E014 | E/LIMIT | Stub bans: NO per-step-only check; NO LLM judgment of "is this plan malicious"; NO BLOCK default until FP bounded; NO SAFEFLOW-scale rewrite; NO sandbox REPLACEMENT. | J-17:L42-L47 |
| J17-E015 | E/TEST | Trigger: plan whose subtasks individually benign but jointly request denied capability (A secret-read + B egress + C write combined = exfiltration); expect PlanGraphChecked + FragmentationFlagged naming {A,B,C} BEFORE A dispatches; SandboxConfigAudited emits sidecar verdict. | J-17:L49-L52 |
| J17-E016 | E/FAIL | NEGATIVE: jointly benign plan passes with CompositionCleared; DAG that cannot fully resolve FAILS-LOUD never silently clears; known-bad Docker-socket mount raises finding not silence. | J-17:L52-L52 |
| J17-E017 | E/DEP | Composes G-1 taint, rules-engine 3F, E-38 compliance pre-gate (sibling), Tier-C 4A / B-18 sandbox (inspect, not replace), J-16 blast-radius (per-EDIT sibling). | J-17:L62-L63 |
| J17-E018 | E/FLOW | Subtask added dynamically (SPLIT_TASK, 4G) after concepts->phases must re-trigger aggregation — wire check to re-run on DAG mutation, not once. | J-17:L67-L67 |
| J17-E019 | E/LIMIT | Sandbox-audit targets LINUX remote sidecar; Windows-primary host out of scope (no container there). | J-17:L68-L68 |
| J17-I001 | I/LIMIT | Check is necessary-not-sufficient; runtime G-1 taint remains the backstop (R5) when aggregation over/under-approximates. | J-17:L66-L66 |
| J17-A001 | A/DET | Union of declared capabilities may OVER-approximate (flag benign) or UNDER-approximate (runtime-only capability); precision unresolved until observe-only FP measured. | J-17:L66-L66 |

Category coverage: capabilities ✓; surfaces (panel) ✓; workflows/states ✓; data/DAG aggregation ✓; flags/observe-only ✓; events ✓; security/sandbox ✓; failures/fail-loud ✓; determinism ✓; observability ✓; tests ✓; constraints/deps ✓. Privacy not a standalone rule; no formal retry API beyond re-trigger on DAG mutation.

### J-77 — perception faithfulness via the trust spine

Source: `docs/thefinalglance/J-plan/J-77_perception_faithfulness_trust_spine.md` (90/90 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J77-E001 | E/STATUS | J-77 is EXTENSION (J-1 + J-8), lever CANT-MISREAD, effort M-L, WAVE VO-1; depends none; authored as NEW file per never-amend-old-plans rule. | J-77:L1-L3 |
| J77-E002 | E/CAP | I5 STRONGEST: compose H-53 citation_prebind over perception region set; deterministic detections ARE EvidenceAnchorSet; VLM description of non-existent region fails closed-set membership -> D-16 refusal / `unverified_claims` channel. | J-77:L7-L8 |
| J77-E003 | E/DATA | I6: add image-perception as 5th G-1 Provenance ingest site (today 4: WebFetch/MCP/file/sandbox); every atomic item gets `source` in {ocr, layout, detector, vlm, merged} + confidence. | J-77:L9-L9 |
| J77-E004 | E/DATA | VLM-written fields split into semantic_label / appearance_notes / uncertain_visual_guess; `source` IS taint label -> capability_gate treats vlm-sourced fields as lower-trust. | J-77:L9-L9 |
| J77-E005 | E/DATA | I7: emit perception result as GRAPH in DKG shape (coordinates, reading order, adjacency, containment, z-order, region descriptors), NOT prose; deterministic consistency check (parent contains children, reading-order edges don't cross impossibly, explicit counts == region-list length). | J-77:L10-L10 |
| J77-E006 | E/DATA | I4: ONE canonical pixel space at ingest; every resize/crop/tile records transform RELATIVE to that frame; reproject all boxes; REJECT any element whose inverse transform is inconsistent; implement AS RRA-style transform ledger (G-2 shape). | J-77:L11-L11 |
| J77-E007 | E/CAP | I3 typed merge graph (GENUINE GAP): words->lines->blocks->semantic-regions; only ONE level contributes to COUNTS; merge by IoU + text similarity + class agreement; counts/exact-text ALWAYS from deterministic detections never captions. | J-77:L12-L12 |
| J77-E008 | E/DET | Deterministic check: no two final regions of same type exceed IoU threshold unless texts meaningfully differ (kills double-counting). | J-77:L12-L12 |
| J77-E009 | E/SURFACE | I8: `elai bench vlm-eval` measures Qwen3-VL-4B + InternVL3-2B + PaddleOCR-VL-0.9B + FastVLM against E-36; dots.ocr DEMOTED to heavy-lane-only; ColQwen2/ColPali as long-doc retrieval adjunct; all anti-slopsquat-resolved (JC-7/JC-8) before any name in code. | J-77:L13-L13 |
| J77-E010 | E/CFG | I1/I2: tile_px column in sweet_spots.toml keyed to DESCRIBER family (448 InternVL / 768-1024 no-tile FastVLM / native-dynamic Qwen); tile_overlap 12.5% as BOUNDARY-REPAIR heuristic never main strategy; owner-tile policy (box owned by tile containing its CENTER). | J-77:L14-L14 |
| J77-E011 | E/FAIL | I9 refusal additions: micro-text refusal when DUAL-OCR engines disagree materially; chart/table refusal unless dedicated parser extracted structure; I5 fabrication check. | J-77:L15-L15 |
| J77-E012 | E/TEST | I11 measured 6-metric done-gate: >=98% exact-match OCR on legible text above min font; >=95% region-grounding recall @IoU0.5; <1% duplicate final regions; >=90% exact-count for <=10 separated elements; ZERO uncited claims; blind-LLM QA materially beats OCR-only baseline on chart/diagram/layout. | J-77:L16-L16 |
| J77-E013 | E/LIMIT | I10: DROP unverified "instruction-before-image" assertion from J-1 leg-3 to "unverified folklore, measure if it matters"; lean on J-8 VR-2 schema constraint. | J-77:L17-L17 |
| J77-E014 | E/LIMIT | EXCLUDED: building perception from scratch; VLM on visual diffs (B-24 Tier-1 pixel-diff veto); counts from captions; dots.ocr as ship candidate; any describer name in code before anti-slopsquat. | J-77:L18-L18 |
| J77-E015 | E/DEFER | Tier-2 DEFER: I8 live describer measurement (remote-GPU-gated, pod_stop after); I9 dual-OCR verify on low-confidence spans. | J-77:L30-L32 |
| J77-E016 | E/CAP | Tier-1 SHIP NOW: I5, I6, I3, I4, I7, I1/I2, I10, I11 harness (deterministic, model-free, pure composition). | J-77:L30-L31 |
| J77-E017 | E/INT | Modules under `crates/orchestrator/src/perception/`: citation-prebind wire (~60 LOC), transform ledger (~120 LOC), fusion.rs (~150 LOC), layout-graph emitter (~100 LOC); sweet_spots.toml + scaffold.rs schema-v2 source/confidence fields. | J-77:L34-L39 |
| J77-E018 | E/EVENT | PREFER-REUSE J-1 PerceptionScaffoldEvent + J-8 VlmPerceptionUpgradeEvent; add I6 `source` field + `unverified_claims`/ghost-region Kind (net 0..+1 from LIVE ~269/578). | J-77:L40-L40 |
| J77-E019 | E/SURFACE | frontend_pair: perception scaffold/region breakdown on VisionRAGDrawer (E-36) or W-6-justified absence; cross_lang pair for I8 sidecar describer route. | J-77:L41-L41, L70-L70 |
| J77-E020 | E/INT | Production: real `elai see` / perception query through perception/; `elai bench vlm-eval` (I8) CLI; test callers never R7. | J-77:L58-L61 |
| J77-E021 | E/LIMIT | Stub bans: NO VLM-sourced count/exact-text; NO described region without backing detection; NO VLM on visual diff; NO instruction-before-image assertion; NO report number as done-gate; NO new default-OFF flag without allowlist. | J-77:L62-L69 |
| J77-E022 | E/TEST | Trigger: `elai see` on dense screenshot/chart with fabrication-bait region + I11 6-metric fixture; expect per-element source field; fabricated region in unverified_claims / D-16; ZERO uncited claims; layout graph consistency; transform ledger consistent; VLM cannot inflate counts; gate FAILS if duplicate-region >1% or count-accuracy <90%. | J-77:L71-L74 |
| J77-E023 | E/DEP | Composes J-1, J-8, H-53, G-1, G-2 RRA, DKG, D-16, B-24; `_grounding/J-PERCEPTION-GROUNDING-MAP.md` is per-track anchor. | J-77:L82-L83 |
| J77-E024 | E/EVIDENCE | All model figures (FastVLM TTFT 152ms@768, PaddleOCR TEDS 0.9195, Qwen3-VL-4B 873 OCRBench, dots.ocr 78.5GB, ColQwen2/ColPali ViDoRe) DESIGN-only; ELAI re-measures. | J-77:L76-L78 |
| J77-A001 | A/CFG | Exact best tile_overlap % is NOT literature-settled; ship 12.5% config, measure on J-8 fixture. | J-77:L14-L14, L87-L87 |
| J77-A002 | A/TEST | I11 gate needs labelled ground-truth fixture first (legible-text/region/count/chart labels); first checkbox is building fixture, not the model. | J-77:L88-L88 |

Category coverage: capabilities/CLI ✓ (`elai see`, `elai bench vlm-eval`); UI surfaces ✓ (VisionRAGDrawer); workflows/states ✓; schemas/data/lifecycle ✓; config/tile table ✓; events ✓; security/taint ✓; failures/refusal ✓; determinism ✓; observability/evidence ✓; tests/6-metric gate ✓; constraints/deps/deferred ✓. Privacy not standalone.

### J-83 — GRC-v3 BeliefState reasoning-state carrier

Source: `docs/thefinalglance/J-plan/J-83_grcv3_beliefstate_reasoning_carrier.md` (121/121 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J83-E001 | E/STATUS | J-83 is PLANNED WAVE GRC-1, M-L, EXTENSION(I-11); KEYSTONE of GRC-v3 wave; completes condense-on-escalate seam into unification carrier. | J-83:L1-L3 |
| J83-E002 | E/CAP | Typed BeliefState carrier: unit of reasoning that crosses model/tier/unload boundary as CLAIMS+RECEIPTS, never raw transcript. | J-83:L5-L5 |
| J83-E003 | E/DATA | CV-1 ENTRY SHAPE: `{claim, grounding_receipt: Option<ReceiptRef>, confidence_bit: read from H-39 AbstentionScored (correct=+1 / abstain=0 / confident-wrong=-t/(1-t)), provenance: DKG key + source SSB seq}`; model emits claim text; engine fills receipt + bit; model NEVER writes the bit (TD-3). | J-83:L15-L15 |
| J83-E004 | E/FLOW | CV-2 PER-STEP RECEIPT GATE: step COMMITTED to carried prefix iff domain receipt PASSES (D-10 routes validator: F-4 / D-11 / J-1 / H-trading); failing/missing receipt -> entry ABSTAINS, more depth, or escalates (I-7) — NEVER committed on chain's own say-so. | J-83:L16-L16 |
| J83-E005 | E/STATE | CV-3 COMMIT-AND-FREEZE: committing receipt-confirmed entry FREEZES it; generator may not re-open; simultaneously grounding + compression (committed foldable, prose lossy, receipt lossless) + overthinking defense (GA-3/J-85). | J-83:L17-L17 |
| J83-E006 | E/FLOW | CV-4 BOUNDARY TRANSPORT: on escalate (I-7) carrier is condensed-then-handed (typed entries, not raw history); on local unload (H-50) carrier is unloaded and selectively reloaded; receipts full fidelity; prose rederivable. | J-83:L18-L18 |
| J83-E007 | E/INT | CV-5 DOMAIN ROUTER: D-10 select_grounding_mode picks validator per claim; carrier identical across domains; only closing receipt differs. | J-83:L19-L19 |
| J83-E008 | E/DEP | Pre-requisites (re-grep W-7): I-11 belief_state.rs (BeliefState::new / emit_condensed_receipt, escalation_waterfall.rs:239); H-39 abstention_score.rs; I-7 cascade contract; H-50 context_policy.rs + live_cli.rs:50; D-10 select_grounding_mode; SessionDag branch/merge. | J-83:L7-L7, L21-L29 |
| J83-E009 | E/CAP | Tasks: extend belief_state.rs typed Entry + ordered carrier (~120 LOC); per-step receipt gate at GRC Verify (~80 LOC); commit-and-freeze (~40 LOC); thread through I-7 condense-on-escalate (~50 LOC); J-83-decisions.md. | J-83:L38-L44 |
| J83-E010 | E/INT | Production reachability: GRC Verify step -> per-step receipt gate; escalation_waterfall.rs:239 carrier crosses model/tier boundary; test callers never R7. | J-83:L54-L57 |
| J83-E011 | E/LIMIT | Stub bans: NO model-set confidence bit; NO commit on chain's own say-so; NO raw-transcript handoff; NO re-implementation of I-11/H-50/I-7/D-10; NO LLM re-summarization compaction (HD-2); NO ciphers; NO .unwrap/.expect on prod; blake3 not sha2 for provenance. | J-83:L58-L62 |
| J83-E012 | E/SURFACE | frontend_pair: existing context/trust telemetry panels (ContextStatePanel / GRC history) read committed/abstained counts via SSB poll; decide exact tile at edit (W-4); per-entry receipt drawer is later UX; justified N/A if no new tile. | J-83:L63-L64 |
| J83-E013 | E/EVENT | prefer-reuse BeliefStateEvent (+Kind: EntryCommitted / EntryAbstained / CarrierHandedOnEscalate) — net 0..+1 from LIVE 268/577. | J-83:L67-L67 |
| J83-E014 | E/TEST | Trigger: multi-turn task that escalates local->cloud mid-reasoning with strict validator and step whose receipt FAILS; after escalation cloud receives TYPED carrier; failed-receipt step ABSTAINED not committed; frozen rewrite dropped+logged. | J-83:L65-L68 |
| J83-E015 | E/TEST | Contract j83_beliefstate_carrier_contract.rs (~80 LOC): (a) PASS commits; (b) FAIL/absent ABSTAINS; (c) model cannot set bit; (d) freeze rewrite dropped; (e) escalation typed not raw; (f) corrupt/empty receipt degrades to ABSTAIN fail-closed (R1). | J-83:L97-L98 |
| J83-E016 | E/CFG | W-2: no new feature flag (carrier on live reasoning path); if gated, default-ON behind *_DISABLE kill-switch with allowlist row. | J-83:L90-L90 |
| J83-E017 | E/DET | GD-2: committed/abstained verdicts recomputable from SSB stream; RD-1: abstention bit read from D-16/H-39; IC-6: receipts gate commit, never a vote. | J-83:L95-L95 |
| J83-E018 | E/DEP | Unlocks J-84 (classifier sizing carrier), J-85 (overthinking guard on frozen entries), J-86 (receipt-gated candidate branches); K-15 bench measures win; K-16 doctrine binds receipts-not-traces. | J-83:L118-L119 |
| J83-A001 | A/DATA | Exact ReceiptRef shape (pointer to SSB row vs inlined verdict): prefer pointer (GD-2, keeps carrier small); decide at authoring against F-4/D-11/J-1 verdict row shapes. | J-83:L113-L114 |
| J83-A002 | A/DEP | R-1 seam risk: ship J-83 with I-7 consumer + multi-turn smoke FIRST (vertical slice), never carrier alone. | J-83:L115-L115 |
| J83-C001 | C/STATUS | DC-1: I-11 is PARTIAL not absent; temptation to treat "done already" is wrong — condense seam exists; typed-entry + per-step gate + commit-freeze do not; wrong seam = STOP + handoff, not silent re-implement. | J-83:L116-L116 |

Category coverage: capabilities ✓; surfaces (panels) ✓; workflows/states/lifecycle ✓; schemas/entry shape ✓; config/kill-switch ✓; events ✓; security/TD-3 ✓; failures/abstain ✓; determinism/receipts ✓; observability ✓; tests/acceptance ✓; constraints/deps ✓. Privacy not standalone; concurrency via freeze immutability.

### J-98 — canvas MEMORY/CASE altitude

Source: `docs/thefinalglance/J-plan/J-98_canvas_case_memory_altitude.md` (114/114 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J98-E001 | E/STATUS | J-98 is PLANNED WAVE 3 = VCG Slice 3 visibility; BUILD-on-shipped; new canvas ALTITUDE; depends J-96 (case store+edges), J-97 (reuse/promotion), shipped H-13/H-15 canvas, J-61 redirect verbs, J-59 per-seat isolation. | J-98:L1-L5 |
| J98-E002 | E/SURFACE | MODE: add `'memory'` to CanvasMode union (`canvas.svelte.ts:39` alongside code/plan/research/arch/design/system/process) + CanvasModeBar entry + `'memory'` op-namespace on InProcessCanvasDispatcher (`canvas_dispatcher.rs:101`); `.claude/commands/canvas-mode.md` flag gains `'memory'`. | J-98:L11-L11 |
| J98-E003 | E/SURFACE | CaseAltitudeNode renders J-96 case: trigger_signature (symptom + taxonomy class), blast_radius_cone, fix_delta summary, proof_receipt (F-4 hash + RRA root badge), 1-3 sentence insight; standing (episodic/skill/rule) from J-97 promotion state. | J-98:L12-L12 |
| J98-E004 | E/STATE | Nodes persist ONLY as J-96/J-97 SSB rows; canvas re-derives byte-identically (GD-2); node with no proof receipt CANNOT render (minting requires F-4 proof); DRAFT/grey impossible — every node is proof-born. | J-98:L12-L12, L38-L38 |
| J98-E005 | E/SURFACE | EDGE renders J-96 deterministic edges (cone-overlap / co-occurrence / same-taxonomy / supersession) with checker-derived weight + row id on edge (J-31 verdict-on-edge); LLM sets no edge/weight (TD-3); supersession dims older case (lineage shown, hard-filtered at reuse SF-4). | J-98:L13-L13 |
| J98-E006 | E/STATE | PROMOTION: node visibly transitions episodic -> skill (after J-97 k held-out verified reuses) -> rule (human-gated, B-23 L3). | J-98:L14-L14 |
| J98-E007 | E/SURFACE | STEER via J-61 closed verb set only: mark-superseded, propose-promotion-for-human-review, pin/unpin; never free-form canvas edit; "promote to rule" is PROPOSAL to human (B-23 L3), not apply; build/reuse through NORMAL FSM (anti-n8n). | J-98:L15-L15, L41-L41 |
| J98-E008 | E/FLOW | Sessions-as-nodes: sessions are J-60 projection; case-nodes are durable RESIDUE once fix PROVEN; process node that produced proven fix wires to its J-96 case node. | J-98:L16-L16 |
| J98-E009 | E/SEC | Per-user adaptation: namespace case altitude per user/project via J-59 PER-SEAT MEMORY + TRUST ISOLATION (UsageTurnEvent.identity) over E-5; poisoned/contradictory case cannot leak across seats; SF-3 auditor firewall holds per-seat-AND-role. | J-98:L18-L19, L42-L42 |
| J98-E010 | E/EVENT | NO new action_type — pure canvas ops PREFER-REUSE CanvasOpApplied; case data is J-96 rows re-derived; net 0 new SSB action_types. | J-98:L25-L26, L43-L43 |
| J98-E011 | E/INT | Production: live dashboard switched to `'memory'` mode renders case altitude from journal; J-61 steer on case node journals + re-renders; test callers never satisfy render reachability R7. | J-98:L56-L59 |
| J98-E012 | E/LIMIT | Stub bans: NO parallel state file; NO LLM-authored node/edge/weight; NO node without proof receipt; NO free-form mutation; NO cross-seat leakage; NO VCG into auditor/critic/adversary (SF-3); NO canvas-side runtime. | J-98:L60-L66 |
| J98-E013 | E/SURFACE | frontend_pair: the MEMORY mode surface itself (cross_lang_diff + svelte-check in-unit). | J-98:L67-L67 |
| J98-E014 | E/TEST | Trigger: >=2 minted J-96 cases (one superseding) + one J-97-promoted-to-skill; switch to memory mode; issue J-61 mark-superseded; expect re-derive under ELAI_SEED (GD-2); supersession dims older; skill standing shows; steer re-renders. | J-98:L68-L71 |
| J98-E015 | E/FAIL | NEGATIVE: node from executor self-report (no proof) FAILS; free-form edit rejected; cross-seat case not visible; promote-to-rule auto-apply FAILS; altitude injected into auditor context FAILS SF-3. | J-98:L71-L71 |
| J98-E016 | E/TEST | Contract j98_case_altitude_contract.rs (~120 LOC) + dashboard svelte-check: GD-2 re-derive; proof-born only; supersession SF-4; free-form rejected; human-gate; cross-seat; SF-3 firewall. | J-98:L91-L92 |
| J98-E017 | E/DEP | Sequence: J-96 CLI/journal value FIRST, J-98 canvas LAST (visibility layer, not engine); answers run §8 "Canvas vs CLI first". | J-98:L104-L104, L109-L109 |
| J98-A001 | A/SURFACE | Mode name: 'memory' vs 'case' vs 'vcg' — pick consistent with J-31/J-60 at edit; 'memory' reads best as third altitude. | J-98:L107-L107 |
| J98-A002 | A/SURFACE | Mature graph scale: render must cluster by taxonomy/cone, not dump 500 nodes; reuse existing canvas clustering; never visually count past ~10 (visual-ui-loop); counts from journal. | J-98:L108-L108 |

Category coverage: user-facing surfaces exhaustive ✓ (mode, bar, nodes, edges, promotion lights, steer verbs, commands flag); workflows/states ✓; schemas via re-derive ✓; events net-0 ✓; security/isolation ✓; failures ✓; determinism GD-2 ✓; tests ✓; constraints/deps ✓. No privacy policy beyond seat isolation; no retry API.

### J-102 — config-ingress admission gate

Source: `docs/thefinalglance/J-plan/J-102_config_ingress_admission_gate.md` (127/127 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J102-E001 | E/STATUS | J-102 is PLANNED WAVE 1; technical config-admission half K-13 OUT-OF-K recorded not authored; EXTENSION on E-43/E-24/J-52 (new-plan-files-only); effort L (WI-6c keystone); +ConfigAdmissionEvent net 0..+1. | J-102:L1-L5 |
| J102-E002 | E/SEC | Lever NO-OUT-OF-BAND-CONFIG: everything a user can ADD or MODIFY that ends up in model hands passes ELAI safety check at first-touch + after any modification. | J-102:L5-L5, L9-L10 |
| J102-E003 | E/FLOW | Architectural key: enforce at LOAD time, not ADD time; content-hash-keyed admission manifest (blake3 TOFU on first-touch, re-scan on hash change) FAIL-CLOSED QUARANTINES any artifact lacking matching ALLOWED row so un-admitted/modified artifact NEVER reaches context or dispatcher. | J-102:L11-L12 |
| J102-E004 | E/SURFACE | Mediated `elai mcp add` / `elai skill add` UX is convenience only (WI-6d follow-on); load-time hash gate is the enforcement. | J-102:L12-L12, L43-L43 |
| J102-E005 | E/STATUS | MCP current: STRONG identity via pins.rs McpServerPin{exact_version,expected_pubkey_blake3,auth_required} + McpPinPolicy{enforce_pin_list,refuse_version_float,refuse_unauthenticated}; GAP = agentlint-style CONTENT checks absent (WI-6a). | J-102:L14-L15 |
| J102-E006 | E/STATUS | Skills: SOFT gate E-43 skill-import-policy + skillspector; PreToolUse HARD enforcement DEFERRED today (WI-6b target). | J-102:L16-L16 |
| J102-E007 | E/STATUS | Rules/agents/CLAUDE.md/AGENTS.md: NOTHING scans new/modified instruction .md before context load — THE GAP keystone (WI-6c). | J-102:L17-L17 |
| J102-E008 | E/LIMIT | agentlint: STEAL MCP check-RULE-SET as DATA only; DO NOT port pip dep; reuse F-1 secret-GREP + VibeGate AST; no new dependency (anti-slopsquat). | J-102:L18-L19, L64-L64 |
| J102-E009 | E/CAP | WI-6c: blake3-keyed admission manifest over `.claude/rules/*.md` + `.claude/agents/*.md` + CLAUDE.md + AGENTS.md + `.claude/skills/*/SKILL.md` at LOAD into context-budget Rules/PhaseSpec slot (ConfigLoader config.rs:205) AND at agent dispatch. | J-102:L40-L40 |
| J102-E010 | E/FAIL | If artifact NEW or hash CHANGED since last admission: run scanner and FAIL-CLOSED QUARANTINE (never enters context/dispatch). | J-102:L40-L40 |
| J102-E011 | E/CAP | Scanner DETERMINISTIC legs: secrets via F-1 GREP; dangerous-verb/capability-escalation directive patterns; known injection phrases "ignore previous"/"forget instructions"; zero-width/homoglyph/encoded-payload detection; frontmatter/schema validation. | J-102:L40-L40 |
| J102-E012 | E/CAP | ADVISORY semantic leg: D-13 cross-family critic "is this directive trying to escalate" (TD-3, NOT the gate). | J-102:L40-L40 |
| J102-E013 | E/INT | NATURAL HOOK = J-52 PathRuleInjector (REUSE, GD-1); also ConfigLoader + Rules-slot fill + agent dispatch. | J-102:L40-L40, L53-L53 |
| J102-E014 | E/CFG | OBSERVE-ONLY first (GD-4): emit would-quarantine set, measure FP on clean config, before defaulting quarantine ON; default-ON flag gets wire-dark-allowlist row (W-2/R9). | J-102:L40-L40, L67-L67 |
| J102-E015 | E/CAP | WI-6a: fold dangerous-FS-path-arg + hardcoded-secret-in-env (F-1) + approval-field-presence + risk-score INTO existing pins.rs / mcp_pin_cmd admission path. | J-102:L41-L41 |
| J102-E016 | E/CAP | WI-6b: promote E-43 from SOFT to HARD PreToolUse refusal — skill with no workflow/skill-imports.log ALLOWED row matching CURRENT content-hash is REFUSED at reference/load. | J-102:L42-L42 |
| J102-E017 | E/EVENT | +ConfigAdmissionEvent Pattern-2: Admitted / Quarantined / ReadmissionRequired; prefer-reuse skill_system_event / mcp pin rows where possible (net 0..+1 from ~268/577). | J-102:L30-L30, L51-L51 |
| J102-E018 | E/SURFACE | frontend_pair: ConfigAdmissionEvent surfaces on SSB safety timeline + quarantine banner; decide config-admission tile at edit (W-4) or W-6-justify (security gate, no required new UI). | J-102:L70-L71 |
| J102-E019 | E/INT | Production: live `elai run` loading new/modified instruction .md quarantines before Rules slot; live `elai mcp pin` / MCP load with dangerous FS-path or hardcoded key refused; unscanned skill HARD PreToolUse blocks; test callers never R7. | J-102:L58-L62 |
| J102-E020 | E/LIMIT | Stub bans: NO agentlint pip dep; NO ADD-time-only enforcement; NO semantic-injection as THE GATE; NO default-ON quarantine before FP measured; NO missing-admission-row => pass (R1 FAIL-CLOSED); NO fork of E-43 / J-52. | J-102:L63-L69 |
| J102-E021 | E/TEST | Trigger: malicious set (i) rules/x.md with ignore-all-gates + zero-width exfil; (ii) mcp.json with /etc FS-path + hardcoded key; (iii) unscanned skill; PLUS clean config FP guard + modify-already-admitted case. | J-102:L72-L73 |
| J102-E022 | E/OBS | Expect Quarantined per malicious + Admitted per clean + ReadmissionRequired on modified-already-admitted; malicious never enter context; clean loads unchanged; modify flips to QUARANTINE until re-admitted. | J-102:L74-L75 |
| J102-E023 | E/FAIL | NEGATIVE: gate-OFF arm lets obfuscated .md into Rules slot (proves load-bearing); benign edit re-admits cleanly (no FP). | J-102:L75-L75 |
| J102-E024 | E/TEST | Contract j102_config_admission_contract.rs (~140 LOC) arms a-g: quarantine obfuscated rules; refuse dangerous mcp; refuse unscanned skill; clean FP guard; on-modify re-scan; missing row FAIL-CLOSED; gate-OFF negative. | J-102:L101-L102 |
| J102-E025 | E/DEP | J-101 is sibling DATA-path hardening; J-102 is CONFIG-path; K-13 is K-native PROVE+PACKAGE half. | J-102:L116-L116 |
| J102-E026 | E/EVIDENCE | Threat rubric: OWASP LLM03 Supply Chain / LLM04 Data & Model Poisoning / LLM01 Prompt Injection + Agentic ASI04 / ASI06. | J-102:L5-L5 |
| J102-A001 | A/SEC | settings.json hook-registration admission: recommend YES (poisoned hook COMMAND is a sink); DECISION at authoring. | J-102:L120-L120 |
| J102-A002 | A/DEP | J-52 PathRuleInjector is PLANNED — soft-dependency; seam may need J-52 first OR gate wires directly at ConfigLoader. | J-102:L121-L121 |
| J102-A003 | A/CFG | FP risk of over-strict scanner blocking legitimate rule edits; mitigation observe-only + clean-config guard + clear re-admission path. | J-102:L122-L122 |

Category coverage: CLI verbs ✓ (`elai mcp pin`, skill load, future mcp/skill add); UI surfaces ✓ (safety timeline, quarantine banner, optional tile); workflows/load path ✓; schemas/manifest ✓; flags/observe-only ✓; events ✓; security/admission exhaustive ✓; failures/quarantine ✓; determinism/hash ✓; observability ✓; tests ✓; constraints/deps ✓. Privacy via secrets detection; no formal concurrency beyond hash re-scan.

### J-111 — voice ingress + spoken readback-confirm

Source: `docs/thefinalglance/J-plan/J-111_voice_ingress_spoken_confirm.md` (149/149 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J111-E001 | E/STATUS | J-111 is PLANNED WAVE VOICE-2, M-L, BUILD-on-planned; depends J-32 HARD (ships AFTER front_door.rs exists); J-112 ships FIRST on shipped substrate; PROOF_SPINE none (product-surface/voice-operator, user-acked 2026-07-10, V-10). | J-111:L1-L4 |
| J111-E002 | E/CAP | Voice input plane is local ASR transducer whose output lands on J-32 FrontDoorRouter exactly like typed chat; NO second router (V-2 — hand-rolled voice command grammar BANNED). | J-111:L6-L11 |
| J111-E003 | E/LIMIT | NO model in T0/T1 command path (ASR + deterministic matchers end to end); NO always-on open mic (V-9); HIGH-risk lanes NEVER auto-execute from voice. | J-111:L11-L12 |
| J111-E004 | E/DEP | GD-1 do NOT re-implement: J-32 lanes/chip/RiskLevel/GateMode, J-35 IntentSpec card (readback is that card spoken, V-5), J-1 perception-gateway doctrine (audio is second modality). | J-111:L12-L14 |
| J111-E005 | E/DEP | HARD dep: front_door.rs::FrontDoorRouter + IntentRouted SHIPPED (J-32 — ABSENT 2026-07-10 = hard gate); soft: J-35 IntentSpec; J-112 spoken_render.rs speaks readback; notify_dispatch.rs:57 is speaker seam. | J-111:L18-L21, L37-L41 |
| J111-E006 | E/CAP | python/elai_voice/ ingress daemon (~250 LOC NEW): local mic capture, push-to-talk gate (open-mic mode exists but default-OFF, V-9), VAD, streaming local ASR (Moonshine v2 primary / faster-whisper fallback, V-7 — anti-slopsquat + USER CONFIRM deps at edit). | J-111:L52-L52 |
| J111-E007 | E/DATA | Daemon emits `{transcript, asr_confidence, audio_blake3, asr_model_pin, ts}` over localhost to CLI host; audio bytes never leave machine; bytes NOT stored (hash only). | J-111:L52-L52 |
| J111-E008 | E/CAP | voice/ingress.rs (~120 LOC NEW): typed VoiceUtterance + taint=Untrusted (G-1); journals +VoiceUtteranceIngested BEFORE routing (replay input = transcript + hash + model pin, GD-2); then FrontDoorRouter::route(transcript) — SAME call typed chat makes. | J-111:L53-L53 |
| J111-E009 | E/EVENT | IntentRouted gains source=voice discriminator (reuse, no new row); +VoiceUtteranceIngested is the new action type (V-6 re-derive at edit). | J-111:L53-L53, L130-L130 |
| J111-E010 | E/CAP | Session addressing (~60 LOC): deterministic addressee matcher BEFORE lane routing over call-sign vocabulary — phonetically distinct aliases assigned at session spawn (collision-checked); bare numerals accepted but ALWAYS read back on mutating lanes. | J-111:L54-L54 |
| J111-E011 | E/FAIL | Unresolved/ambiguous addressee -> spoken CLARIFY (never a guess; J-32 fail-closed-to-asking posture). | J-111:L54-L54 |
| J111-E012 | E/SURFACE | Spoken readback-confirm (~100 LOC): for MEDIUM+ lanes, build readback from ROUTED intent (lane + addressee + arg_spans; J-35 card spoken) — NOT raw transcript — speak via J-112, await explicit confirm token. | J-111:L55-L55 |
| J111-E013 | E/FAIL | asr_confidence below floor -> CLARIFY regardless of lane. | J-111:L55-L55 |
| J111-E014 | E/SEC | HIGH risk: readback + confirm necessary but NOT sufficient — voice HIGH always ALSO requires existing non-voice confirmation surface (chip/dashboard). | J-111:L55-L55 |
| J111-E015 | E/EVENT | Confirm/abort decisions journal as same labeled tuples J-32 chip emits (D-16 calibration feed). | J-111:L55-L55 |
| J111-E016 | E/SURFACE | CLI: `elai voice` subcommand (~80 LOC) — daemon host + production entrypoint (spawn daemon, own socket, wire ingress). | J-111:L56-L56 |
| J111-E017 | E/CFG | Env gate ELAI_VOICE_ENABLE default-OFF (hardware + new deps; type=experiment wire-dark-allowlist.toml row with owner/expiry/promotion_plan_id=J-111/smoke, R9). | J-111:L56-L56 |
| J111-E018 | E/TEST | INTEGRATION_SMOKE with J-112 (R10): `elai voice` recorded fixture end-to-end: ingress row -> IntentRouted(source=voice) -> readback -> confirm -> dispatch rows IDENTICAL to typed /fix sequence (GD-2 diff zero) -> completion -> J-112 narration row. | J-111:L57-L57 |
| J111-E019 | E/TEST | HARD input: mis-addressed utterance ("session thirty" where 38 exists) must land in CLARIFY, zero dispatch. | J-111:L57-L57 |
| J111-E020 | E/INT | Production: `elai voice` -> daemon -> ingress.rs -> FrontDoorRouter::route; test callers never R7. | J-111:L86-L88 |
| J111-E021 | E/LIMIT | Stub bans: NO second router/bespoke grammar; NO model in T0/T1; NO raw-audio persistence; NO readback from raw transcript; NO mutating dispatch without journaled confirm; NO HIGH voice-only; NO thread_rng in spoken variance (seeded EntropySource only); NO .unwrap on prod; NO always-on mic default. | J-111:L89-L93 |
| J111-E022 | E/SURFACE | frontend_pair: dashboard chat shows voice-sourced messages with source=voice badge + readback/confirm state (reuse J-32 ACK-card surface; Tauri read of same rows); justified minimal — voice UI is audio. | J-111:L94-L96 |
| J111-E023 | E/TEST | Trigger: `ELAI_VOICE_ENABLE=1 elai voice --fixture tests/fixtures/voice/fix_sum_utterance.wav` then live-mic arm manually. | J-111:L97-L98 |
| J111-E024 | E/OBS | Expect VoiceUtteranceIngested (audio_blake3 + asr_model_pin) -> IntentRouted{source=voice} -> confirm tuple -> IDENTICAL dispatch sequence as typed /fix. | J-111:L99-L99 |
| J111-E025 | E/FAIL | NEGATIVE must FAIL: mutating dispatch with no confirm row; readback containing transcript text absent from routed args; mis-addressed fixture producing any dispatch. | J-111:L100-L100 |
| J111-E026 | E/TEST | Contract j111_voice_ingress_contract.rs (~120 LOC): (a) ingress before route; (b) readback from routed fields + corrupted-transcript fixture; (c) mis-address CLARIFY; (d) no-confirm mutating impossible; (e) HIGH voice-only impossible; (f) confirm GD-2 vs typed /fix; (g) corrupt/empty audio -> CLARIFY never guess (R1). | J-111:L122-L123 |
| J111-E027 | E/SEC | Redteam residues accepted (GD-5): cooperative attacker with mic access during push-to-talk on LOW lane; LOW lanes non-mutating by J-32 ladder. | J-111:L73-L73 |
| J111-E028 | E/DEFER | Streaming-ASR partials: route only on end-of-utterance (VAD close) in v1; partial-transcript routing DEFERRED optimization. | J-111:L142-L142 |
| J111-E029 | E/FAIL | FORK: J-32 unshipped -> STOP, BLOCKED handoff (R-4); do NOT hand-roll a router (V-2). FORK: ASR dep fails anti-slopsquat or user declines -> fall back to OS/system dictation path (daemon accepts text over same socket, ASR fields provider=external). | J-111:L67-L68 |
| J111-E030 | E/LIMIT | ABORT: any path where mutating dispatch fires with no confirm row; any storage of raw audio bytes; any second command grammar outside J-32 lanes. | J-111:L71-L71 |
| J111-A001 | A/CFG | Confirm-token grammar ("confirm"/"go"/Esc-word) must be T1 deterministic matcher; decide token set at edit; part of frozen lane vocabulary, never model-classified. | J-111:L140-L140 |
| J111-A002 | A/DEP | Call-sign assignment collides with dashboard session naming — reuse same registry if one ships before this; do not fork naming. | J-111:L141-L141 |
| J111-A003 | A/CAP | Exact FrontDoorRouter::route signature + IntentRouted payload fields unknowable until J-32 lands (V-2); re-grep at edit W-7. | J-111:L69-L69 |
| J111-A004 | A/DEP | Moonshine v2 python package name/version/license live at edit (V-7 directional; anti-slopsquat protocol). | J-111:L70-L70 |

Category coverage: CLI verb `elai voice` ✓; dashboard badge/confirm state ✓; notifications via J-112 ✓; workflows/states ✓; schemas/VoiceUtterance ✓; flags ELAI_VOICE_ENABLE ✓; events ✓; security/taint/push-to-talk ✓; privacy (no audio persistence) ✓; failures/CLARIFY ✓; determinism/replay ✓; observability ✓; tests/smoke ✓; constraints/deps/deferred ✓.

## Cross-file interaction records

| Interaction ID | Class | Interaction | Atom references |
|---|---|---|---|
| IX-001 | E | PGM grounds J-8/J-9 figures and JC corrections that J-77 consumes (VR-2 coords, VR-5 micro-text, JC-7/JC-8 anti-slopsquat, E-36 lane, dots.ocr heavy-only). | PGM-E005, PGM-E009, PGM-E012–E013, PGM-E031, PGM-E034–E035; J77-E009–E010, J77-E013, J77-E023 |
| IX-002 | E | J-77 routes J-1/J-8 perception scaffold through H-53/G-1/G-2/DKG trust spine; PGM is named per-track anchor. | J77-E001–E006, J77-E023; PGM-E001 |
| IX-003 | E | J-11 S-3 worktree lifecycle amends I-2 (also reflected in I-2 deep discovery J-11 amendment atoms). | J11-E011–E012 |
| IX-004 | E | J-11 S-2 held-out merge-gate + DO-NOT-COPY fail-opens target I-4; score parser TD-3 shared with D-11 demotion doctrine. | J11-E005–E009, J11-E021 |
| IX-005 | E | J-15 is the determinism-triage twin of J-11 (doc amendments, zero runtime/SSB); both produce decisions.md and source-cited blocks. | J11-E001, J15-E001, J15-E012 |
| IX-006 | E | J-15 I-34 llguidance Option A leaves XGrammar on sidecar; J-13 seeds local-lane determinism that J-14 grades. | J15-E002–E003; J13-E001–E004 |
| IX-007 | E | J-13 records sampling_seed into G-2 RRA; J-15 binds I-49 fixtures to J-14 export-fixture format using content blake3 keys. | J13-E004; J15-E009 |
| IX-008 | E | J-17 per-PLAN compositional check is sibling of J-16 per-EDIT blast-radius; both observe-only first then block behind allowlist. | J17-E001–E005, J17-E017 |
| IX-009 | E | J-17 aggregates G-1 per-subtask capabilities; J-77 adds perception as 5th G-1 ingest; J-111 taints voice as Untrusted G-1. | J17-E002; J77-E003; J111-E008 |
| IX-010 | E | J-83 completes I-11 BeliefState; J-85 consumes commit-freeze; J-86 layers receipt-gated candidates; J-84 sizes effort into the carrier. | J83-E001–E005, J83-E018 |
| IX-011 | E | J-83 confidence bit is READ from H-39 (RD-1); D-10 routes domain validators including J-1 pixel-diff for visual claims. | J83-E003–E004, J83-E007 |
| IX-012 | E | J-98 is third canvas altitude after J-31 system + J-60 process; re-derives J-96/J-97; steers via J-61; namespaces via J-59. | J98-E001–E010 |
| IX-013 | E | VCG sequence J-96 mint -> J-97 reuse (risky default-off) -> J-98 visibility last. | J98-E017 |
| IX-014 | E | J-102 config-path admission pairs with J-101 data-path hardening; K-13 owns PROVE+PACKAGE mapping of both. | J102-E001, J102-E025 |
| IX-015 | E | J-102 reuses J-52 PathRuleInjector, E-24 pins, E-43 skill policy, F-1 VibeGate, D-13 advisory critic. | J102-E005–E006, J102-E011–E016 |
| IX-016 | E | J-111 hard-depends J-32 FrontDoorRouter; consumes J-112 for TTS readback; reuses J-35 IntentSpec spoken form. | J111-E002–E005, J111-E012 |
| IX-017 | E | J-111 + J-112 form voice loop: speak command, hear readback, confirm, harness narrates from journal. | J111-E018, J111-E001 |
| IX-018 | E | Multiple sources prefer-reuse existing SSB families and cite live baselines that drift (226/537, 229/540, 231/542, ~268-269/577-578); all require W-9 re-read at edit. | PGM-E040; J13-E011; J17-E011; J77-E018; J83-E013; J98-E010; J102-E017 |
| IX-019 | E | Anti-slopsquat + user-confirmation gates recur: PGM JC-7/8 dots.ocr/Qwen3-VL; J-13 batch_invariant_ops pin; J-15 llguidance; J-77 describer names; J-111 Moonshine/faster-whisper. | PGM-E034–E035; J13-E002; J15-E004; J77-E009; J111-E006, J111-A004 |
| IX-020 | E | Observe-only-then-block / wire-dark-allowlist pattern: J-13 determinism default, J-17 compositional BLOCK, J-102 quarantine default-ON, J-111 ELAI_VOICE_ENABLE. | J13-E006; J17-E005; J102-E014; J111-E017 |
| IX-021 | E | Dashboard surfaces cluster: VisionRAGDrawer (J-77), runtime panel determinism (J-13), EvidenceGate/compliance (J-17), ContextStatePanel/GRC history (J-83), memory canvas mode (J-98), safety timeline/quarantine banner (J-102), voice badge/ACK-card (J-111). | J13-E012; J17-E012; J77-E019; J83-E012; J98-E002; J102-E018; J111-E022 |
| IX-022 | E | CLI surface cluster: `elai run --local/--emit-rra`, `elai replay --verify`, `elai see`, `elai bench vlm-eval`, `elai mcp pin`, `elai voice`, deferred `elai hardware-catalog` / `elai mcp add` / `elai skill add`. | PGM-E043; J13-E016; J77-E009, J77-E020; J102-E004, J102-E019; J111-E016 |
| IX-023 | I | Doc-only units (J-11, J-15) create no production symbols but gate correctness of later build sessions through W-3 source-cited amendments. | J11-E001, J11-E028; J15-E001, J15-E013 |
| IX-024 | E | Composed summary workflows: voice enters J-32 after J-111 taint row; perception chain J-1->J-8->J-77; config admission J-102 before writes; GRC spine J-83->J-84->J-85->J-86. | 10-J composed map; J111-E008; J77-E001; J102-E009; J83-E018 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | ambiguity | PGM package names for dots.ocr / Qwen3-VL embedding/reranker unverified; J-8 prefer-reuse event choice open. | PGM-A001–A002 |
| AC-002 | ambiguity | J-11 target status drift and whether amendments already present. | J11-A001 |
| AC-003 | ambiguity | J-13 3090 overhead and NF4 live probe unmeasured. | J13-A001–A002 |
| AC-004 | ambiguity | J-15 E-31 vs H-7 hop home; I-34 dep pending user confirmation. | J15-A001–A002 |
| AC-005 | ambiguity | J-17 capability-union over/under-approximation precision. | J17-A001 |
| AC-006 | ambiguity | J-77 tile_overlap 12.5% unproven; I11 fixture labels not yet built. | J77-A001–A002 |
| AC-007 | ambiguity/conflict | J-83 ReceiptRef shape; vertical-slice sequencing; DC-1 partial-vs-done temptation. | J83-A001–A002, J83-C001 |
| AC-008 | ambiguity | J-98 mode name and clustering scale for large case graphs. | J98-A001–A002 |
| AC-009 | ambiguity | J-102 settings.json admission decision; J-52 soft-dependency; FP risk. | J102-A001–A003 |
| AC-010 | ambiguity | J-111 confirm-token set; call-sign registry ownership; FrontDoorRouter signature; Moonshine package pin. | J111-A001–A004 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/10-J-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### PGM deltas (map has no dedicated summary heading; compare via J-8/J-9 + inventory)

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-PGM-001 | **ADD** exact 3-file VLM corpus titles + Downloads path method; summary flags this as ambiguous. | 10-J:L1553 | PGM-E002 |
| SD-PGM-002 | **ADD** full VR-1..VR-14 table (resize numbers, MoAI/AVR arXiv ids, per-VRAM ladder models, InternVL tiling). | 10-J:L106-L116 (compressed) | PGM-E004–E018 |
| SD-PGM-003 | **ADD** complete HW-1..HW-14 live anchors (file:line) and odysseus fit.py structure. | 10-J:L118-L127 | PGM-E019–E033 |
| SD-PGM-004 | **ADD** JC-7..JC-11 corrections including prefer-reuse PerceptionScaffoldEvent alternative and live 226/537 baseline. | partial | PGM-E034–E040 |
| SD-PGM-005 | **ADD** DC-1/DC-2/DC-3 sequencing and informational-only catalog enforcement. | 10-J:L124-L127 | PGM-E037, PGM-E042–E043 |
| SD-PGM-006 | **ADD** HW-not-HD naming rule and web verification set including Seed1.5-VL catalog-only. | none | PGM-E041, PGM-E044 |

### J-11 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J11-001 | **ADD** full target list (I-2/I-4/I-17/I-25/I-30/D-6/D-11/H-43/H-47/H-50) and three separately-anchored I-4 blocks. | 10-J:L140-L149 | J11-E004, J11-E031 |
| SD-J11-002 | **ADD** exact Arbor source file:line map, protected_paths/required_outputs, merge --no-ff, and metric_direction-aware reject. | partial | J11-E005–E006 |
| SD-J11-003 | **ADD** complete ComBench checker clauses, 4-string rubric N∈{0,1,6,7}, and final<=raw demotion formula. | partial | J11-E018–E021 |
| SD-J11-004 | **ADD** full difficulty band table, fusion equation, four leakage bypass modes, and S-8d decline. | partial | J11-E022–E025 |
| SD-J11-005 | **ADD** DETERMINISTIC_CHECK grep suite, idempotent skip, and WHAT ARBOR LACKS non-forced write. | none | J11-E026, J11-E030, J11-E032 |

### J-13 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J13-001 | **ADD** exact anchors runtime_client.rs:596-597, server.py:786/990, rra.rs:474, LIVE 229/540, and cross_lang pair requirement. | 10-J:L162-L170 | J13-E009–E013 |
| SD-J13-002 | **ADD** full Kind list BatchInvariantEnabled/SeededSamplingApplied/DeterminismProbeRun/DeterminismDivergence and seed-less FAIL arm. | partial | J13-E011, J13-E017 |
| SD-J13-003 | **ADD** E-26 capability-check requirement and nesting destroy bug as contract-tested ban. | partial | J13-E002, J13-E005, J13-E015 |
| SD-J13-004 | **FLAG** unmeasured 3090 Triton overhead and NF4 probe gap (summary notes residue but not these open probes). | 10-J:L167-L169 | J13-A001–A002 |

### J-15 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J15-001 | **ADD** llguidance v1.7.6 MIT/525K snapshot, non-serializable recompile, xgrammar-rs rejection details. | 10-J:L184-L193 | J15-E002–E004 |
| SD-J15-002 | **ADD** J-2 conductor patterns detail and intentusnet transparent-proxy note. | partial | J15-E006 |
| SD-J15-003 | **ADD** culpa DO-NOT-COPY exhibits and ~40 rejection clusters including Anandb71/arbor name-collision. | partial | J15-E010–E011 |
| SD-J15-004 | **ADD** E-31 vs H-7 open choice and PENDING-USER-CONFIRMATION for I-34 dep. | none | J15-A001–A002 |

### J-17 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J17-001 | **ADD** exact verdict triad ALLOW/FRAGMENTATION-FLAGGED/BLOCK and CompositionCleared negative arm. | 10-J:L206-L213 | J17-E003, J17-E016 |
| SD-J17-002 | **ADD** module paths, symbol names, LIVE 231/542, prefer-reuse option, and EvidenceGate/compliance panel. | partial | J17-E010–E012 |
| SD-J17-003 | **ADD** exfiltration fixture shape (A/B/C), DAG re-trigger on SPLIT_TASK, and Linux-sidecar-only audit boundary. | partial | J17-E015, J17-E018–E019 |
| SD-J17-004 | **ADD** full grounding paper set (SIF/SAFEFLOW/MAScope/PRUDENTIA/SandboxEscapeBench/SDOF/SEMAP) and honest N=14 sample note. | none | J17-E009 |
| SD-J17-005 | **FLAG** aggregation over/under-approximation as open precision risk. | none | J17-A001 |

### J-77 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J77-001 | **ADD** full I1–I11 priority numbering, Tier-1 vs Tier-2 split, and dual-OCR Tier-2 deferral. | 10-J:L816-L826 | J77-E002–E016 |
| SD-J77-002 | **ADD** exact 6-metric thresholds (>=98% OCR, >=95% recall@0.5, <1% dups, >=90% count, zero uncited, blind-LLM beats OCR-only). | partial (summary has only >1% / <90% fail) | J77-E012, J77-E022 |
| SD-J77-003 | **ADD** VLM field split semantic_label/appearance_notes/uncertain_visual_guess and source enum. | partial | J77-E003–E004 |
| SD-J77-004 | **ADD** owner-tile center policy, VisionRAGDrawer surface, LIVE ~269/578 prefer-reuse, and fixture-first risk. | none | J77-E010, J77-E018–E019, J77-A002 |
| SD-J77-005 | **ADD** DESIGN model figures retained only as rationale (OCRBench/TEDS/ViDoRe/VRAM). | partial | J77-E024 |

### J-83 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J83-001 | **ADD** CV-1..CV-5 formal carrier semantics and confidence formula correct=+1/abstain=0/confident-wrong=-t/(1-t). | 10-J:L876-L884 | J83-E003–E007 |
| SD-J83-002 | **ADD** exact LOC task split, LIVE 268/577 Kind list, kill-switch W-2 rule, and contract arms a-f. | partial | J83-E009, J83-E013, J83-E015–E016 |
| SD-J83-003 | **ADD** ContextStatePanel/GRC history frontend_pair and later per-entry receipt drawer deferral. | partial | J83-E012 |
| SD-J83-004 | **ADD** ReceiptRef open choice, vertical-slice R-1 sequencing, and DC-1 partial-vs-done conflict. | none | J83-A001–A002, J83-C001 |
| SD-J83-005 | **ADD** unlocks J-84/J-85/J-86/K-15/K-16 and external paper rationale set. | 10-J:L1274 | J83-E018 |

### J-98 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J98-001 | **ADD** exact CanvasMode union members, canvas.svelte.ts:39 / canvas_dispatcher.rs:101 anchors, and canvas-mode.md flag. | 10-J:L1074-L1083 | J98-E002 |
| SD-J98-002 | **ADD** full CaseAltitudeNode field list (trigger_signature, cone, fix_delta, proof badge, insight, standing). | partial | J98-E003 |
| SD-J98-003 | **ADD** four edge kinds, J-61 verb subset, net-0 CanvasOpApplied policy, and SF-3/SF-4 firewall arms. | partial | J98-E005, J98-E007, J98-E010, J98-E015 |
| SD-J98-004 | **ADD** contract arms a-g, clustering/count visual-ui-loop risk, and CLI-first sequencing rationale. | partial | J98-E016–E017, J98-A001–A002 |

### J-102 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J102-001 | **ADD** WI-6a/b/c/d labeling, LOAD-not-ADD architecture, and mediated-add-as-convenience-only rule. | 10-J:L1001-L1010 | J102-E003–E004, J102-E009 |
| SD-J102-002 | **ADD** complete instruction-surface path glob, deterministic scanner leg list, and D-13 advisory split. | partial | J102-E009–E012 |
| SD-J102-003 | **ADD** current-state MCP/Skills/Rules split (STRONG/SOFT/NOTHING), agentlint DATA-only steal, and live pin symbols. | none | J102-E005–E008 |
| SD-J102-004 | **ADD** safety timeline + quarantine banner surfaces, contract arms a-g, and settings.json open decision. | partial (no UI detail) | J102-E018, J102-E024, J102-A001 |
| SD-J102-005 | **ADD** J-52 soft-dependency and OWASP threat rubric mapping. | none | J102-A002, J102-E026 |

### J-111 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J111-001 | **ADD** exact daemon emit schema, hash-only audio policy, and VoiceUtteranceIngested-before-route ordering. | 10-J:L1188-L1199 | J111-E006–E009 |
| SD-J111-002 | **ADD** call-sign assignment/collision rules, bare-numeral always-readback on mutating lanes, and HIGH dual-surface requirement. | partial | J111-E010, J111-E014 |
| SD-J111-003 | **ADD** ELAI_VOICE_ENABLE allowlist row fields, fixture path, wave smoke with J-112, and OS-dictation fallback fork. | partial | J111-E017–E019, J111-E029 |
| SD-J111-004 | **ADD** wargame MOVE/ON_FAIL structure, redteam residue, entropy quarantine, and deferred partial-ASR routing. | none | J111-E027–E028, J111-E021 |
| SD-J111-005 | **ADD** open confirm-token grammar, call-sign registry ownership, and J-32 signature recon needs. | none | J111-A001–A004 |
| SD-J111-006 | **ADD** dashboard source=voice badge + readback/confirm state on ACK-card surface. | partial | J111-E022 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **10/10 files**, **1,223/1,223 logical lines**, **149,753/149,753 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **268 unique atoms** = **244 explicit**, **002 inferred**, **021 ambiguous**, **001 conflicting/corrective**.
- Per-file atom counts: PGM **046**; J-11 **034**; J-13 **021**; J-15 **017**; J-17 **021**; J-77 **026**; J-83 **021**; J-98 **019**; J-102 **029**; J-111 **034**. Sum **268**.
- Relational interaction records: **24** (`IX-001`–`IX-024`). Summary-delta records: **049** (`SD-*`). Ambiguity/conflict register: **10** (`AC-001`–`AC-010`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `PGM/J11/J13/J15/J17/J77/J83/J98/J102/J111-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them. User-facing surfaces captured as own atoms: VR-10 mockup template; deferred `elai hardware-catalog` + HardwareModelCatalogPanel; LocalDeterminism runtime-panel status; EvidenceGate/compliance fragmentation finding; `elai see` / `elai bench vlm-eval` / VisionRAGDrawer; ContextStatePanel/GRC history; CanvasMode `'memory'` + CanvasModeBar + CaseAltitudeNode/Edge + canvas-mode.md flag + J-61 verbs; ConfigAdmissionEvent safety timeline + quarantine banner; `elai voice` CLI + source=voice dashboard badge + readback/confirm state + push-to-talk; deferred `elai mcp add` / `elai skill add`.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the ten exact DEEP-J-006 manifest rows. The only orientation artifact consulted was `10-J-SUMMARY_INDEX.discovery.md`.
- Output metrics after finalization: `output_lines=0549`; `output_bytes=087171`.
- Canonical normalized output SHA-256: 458b01f4b5ff0a0fa7f6c881313a3151499c142fec678135483642daa2d8ab98
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
