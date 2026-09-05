# J-9: hardware detection + per-lane model eligibility catalog — scan the user's box, show what it can run

STATUS: PLANNED (WAVE 2; Tier-1 SHIP NOW = probe->catalog builder + SSB emit (model-free);
Tier-2 DEFER = `elai hardware-catalog` CLI + dashboard panel (DC-2); BUILD wrapping shipped probes)

Objective: ELAI ships the entire hardware-sensing substrate — GpuInfo/SystemProbeResult
(system_probe.rs:80/127, HW-1/HW-2), classify_vram_tier (:145, HW-3), VramTier (vram_tier.rs:6,
HW-4), ModelCapabilityProfile + detect (context_budget.rs:284/697, HW-5/HW-6), can_fit_llm
(agent_pool.rs:62, HW-7), the C-30 research tiers (HW-8) — but every one of them is an INTERNAL
routing input; nothing user-facing answers "what can THIS box run, per capability lane?" (HW-10,
net-new confirmed). J-9 builds HardwareModelCatalog: probe the user's GPU/RAM/disk/platform once,
resolve a data-driven catalog into per-lane eligibility lists (coding LLM / vision VLM / OCR /
embedding+reranker) with per-quantization VRAM-fit estimates, emit the result as journaled SSB
rows. POLICY SHIFT carried (HW-14, user-stated 2026-06-11): the Gemma 4 / Qwen3-32B rejections
were user-hardware-specific, not universal — the catalog lists them TIER-CONDITIONALLY for boxes
that fit them, with the prior-rejection context as an honest-residue note. CATALOG DATA ONLY:
no routing override, no auto-install, no model-pin flip, no CLAUDE.md/allowlist change (JC-10,
DC-3). Prior art: odysseus services/hwfit/fit.py (HW-12) — mirror the lane/fit-score STRUCTURE,
derive ELAI's own weights from measured fixtures, never copy odysseus' (GD-5).

Estimated effort: ~1 session (~6-8 h) Tier-1; Tier-2 surface ~0.5 session when scheduled.
Pre-requisites: none beyond shipped substrate (HW-1..HW-9; re-grep at edit, W-7). The vision/OCR
lane seed arrives from J-8's data/perception/vision_ladder.toml when J-8 ships — J-9 ships first
with the coding/embedding lanes live and the vision lane seeded from the same map data (VR-13)
marked [DESIGN; J-8 eval pending].
Parallel-safe with: J-2..J-7 (disjoint). Coordinate with J-8 on vision_ladder.toml ownership.
Source: HW family + VR-13/VR-11 (_grounding/J-PERCEPTION-GROUNDING-MAP.md); odysseus-INDEX.md:64.
Doctrine: J-0_INDEX.md.

## Context
Why this matters: ELAI's model policy history is full of box-specific verdicts frozen into prose
(Gemma 4 rejected, Qwen3-32B rejected, 7B canonical at 12-24GB) that are correct for the author's
RTX 3090 12GB and wrong as universal statements. A user on a 48GB box, an 8GB laptop, or Apple
silicon needs DIFFERENT answers, derived from THEIR hardware. The odysseus pattern (HW-12):
detect -> estimate model footprint -> fit-score -> ranked per-capability list. ELAI already owns
the detect leg (HW-1..HW-4) and the footprint primitive (HW-7 can_fit_llm); what is missing is
the catalog data model, the per-lane resolution, and the journaled emission. The catalog is the
INFORMATIONAL substrate a later user-gated session can promote into routing; J-9 deliberately
stops at "show what fits" (JC-10).

## Existing state (B..J coverage — why this is BUILD-on-shipped)
- HW-1..HW-4: GpuInfo / SystemProbeResult / classify_vram_tier / VramTier — SHIPPED (C-22 line).
  The probe leg is done; J-9 adds zero probing code.
- HW-5/HW-6: detect_model_capability_profile + ModelCapabilityProfile (micro_3b/micro_7b/
  micro_8gb/large_32b/cloud_200k) — SHIPPED (B-10 label PLAN-asserted; symbol verified). The
  coding-lane capability shapes the catalog's coding rows reuse.
- HW-7: ModelPool::can_fit_llm(model_size_bytes) — SHIPPED. The fit predicate the catalog calls
  per (model, quantization) row.
- HW-8: research_config_for_tier (C-30) — SHIPPED, coding/research lane only. Proof a per-tier
  config pattern exists; proof no vision/OCR/embedding lane does (HW-10).
- HW-9: BackendMode/BackendConfig (B-37 seam) — the backend column on catalog rows (a model is
  eligible AND has a backend that serves it; E-51 backends are placeholders, HW-11).
- E-31 tier matrix (docs/strategy/E-31-tier-matrix.md): the strategy-doc coding ladder the
  catalog's coding lane transcribes into DATA.
- HW-12: odysseus hwfit (rank_models:521 / analyze_model:299 / _estimate_speed:73 /
  _quality_score:143 / _fit_score:198) — structural prior art, read-only.
- HW-14: the policy shift this session encodes at catalog level.
- SSB: LIVE 226/537; no catalog action_type exists.

## Reads (mandatory before coding)
1. _grounding/J-PERCEPTION-GROUNDING-MAP.md (HW family + VR-13/VR-11 + JC-9..11 + DC-2/DC-3).
2. crates/orchestrator/src/system_probe.rs (HW-1..HW-3 regions) + crates/core/src/vram_tier.rs.
3. crates/orchestrator/src/context_budget.rs:284/697 region (profile shapes to reuse).
4. crates/orchestrator/src/agent_pool.rs:48-70 (can_fit_llm contract).
5. docs/.source/odysseus-main/services/hwfit/fit.py:73-560 (the structural pattern; read-only,
   GD-1 pointer — never a dep, never copied weights).
6. docs/strategy/E-31-tier-matrix.md (coding-lane ladder to transcribe).
7. metrics/baseline_findings_and_actions.md Part 1 (the rejection context the residue notes cite).

## Tasks (WAVE 2 — Tier-1)
1. Catalog data model (~80 LOC): crates/orchestrator/src/hardware_catalog.rs —
   ModelCatalogEntry {lane (Coding|Vision|Ocr|Embedding), model_id, backend: BackendMode,
   footprint: Vec<{quant, min_vram_mb}>, platform_tags, source_tag ([MEASURED]|[DESIGN]),
   residue_note: Option<String>}, CatalogLane, HardwareModelCatalog {probe, lanes}. Entries are
   DATA rows deserialized from toml — zero model constants in code (stub_ban).
2. Seed catalog (~1 toml): data/hardware_catalog/model_catalog.toml —
   coding lane from the E-31 matrix (Qwen2.5-Coder 3B/7B/32B per tier, cloud lane);
   vision lane from VR-13 (Phi-4-MM sub-6B / Qwen2.5-VL-7B / Gemma-3-27B / Qwen2.5-VL-32B/72B /
   InternVL3-14B/78B / FastVLM apple-silicon tag per JC-9) all [DESIGN; J-8 eval pending];
   OCR lane (PaddleOCR-VL canonical per HW-13; dots.ocr row gated on JC-7);
   embedding lane (GraphCodeBERT/Granite shipped; Qwen3-VL-Embedding gated on JC-8).
   Gemma-4 + Qwen3-32B rows carry tier-conditional eligibility + the HW-14 residue note verbatim
   ("rejected 2026-05 on a 12GB box for context-extension limits; tier-conditional since
   2026-06-11; unmeasured on this hardware").
3. Resolution (~100 LOC): HardwareModelCatalog::build(probe: &SystemProbeResult) -> resolved
   lanes — classify_vram_tier (HW-3) for the tier, can_fit_llm (HW-7) per footprint row,
   platform_tags vs detected platform, backend availability vs HW-9 (absent backend = listed
   with eligible=false, reason=backend_missing — honest, not hidden). Fit-score = the odysseus
   STRUCTURE (fit x footprint headroom) with ELAI-derived weights documented in the decisions
   doc (HW-12, GD-5).
4. SSB (~6-part): +HardwareModelCatalogEvent (Pattern-2 single-variant; Kind: CatalogBuilt /
   TierAssigned / EligibilityChecked / LaneSeeded), baseline+1 from LIVE 226/537, re-read at
   edit (W-9). Emit CatalogBuilt at the startup probe site (the existing SystemProbe call path —
   re-grep the exact site at edit, W-7).
5. Tier-2 (DEFER, registered): `elai hardware-catalog [--lane X] [--json]` CLI subcommand +
   HardwareModelCatalogPanel dashboard tile (the frontend_pair lands here; deferred per DC-2
   with the catalog builder live first).
6. Decision doc: workflow/reasoning/J-9-decisions.md (lane seeds chosen + source tags, fit-score
   weights + derivation, the HW-14 residue wording, the Tier-2 defer record).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/hardware_catalog.rs (ModelCatalogEntry, CatalogLane,
    HardwareModelCatalog::build, fit scoring)
  - data/hardware_catalog/model_catalog.toml (the seed catalog — DATA, user-editable)
  - +HardwareModelCatalogEvent SSB (baseline+1)
  - [Tier-2, deferred] `elai hardware-catalog` subcommand + HardwareModelCatalogPanel
consumes_symbols:
  - system_probe.rs::SystemProbeResult/GpuInfo/classify_vram_tier (HW-1..3 — never re-probed)
  - vram_tier.rs::VramTier (HW-4)
  - context_budget.rs::ModelCapabilityProfile (HW-6 — coding-lane shapes reused)
  - agent_pool.rs::ModelPool::can_fit_llm (HW-7 — the fit predicate)
  - runtime_builder.rs::BackendMode (HW-9 — the backend column)
  - data/perception/vision_ladder.toml when J-8 ships (the vision-lane refresh source)
reachable_from:
  - the startup SystemProbe call path (CatalogBuilt emitted where the probe already runs)
    [PRODUCTION startup path; exact site re-grepped at edit, W-7]
  - [Tier-2] `elai hardware-catalog` CLI entrypoint
stub_ban:
  - NO re-implementation of GpuInfo / SystemProbeResult / classify_vram_tier / VramTier /
    can_fit_llm (consume only, GD-1)
  - NO model names/footprints as code constants (toml data only)
  - NO routing override, auto-install, model-pin flip, or allowlist/CLAUDE.md mutation from the
    catalog (JC-10/DC-3 — informational only)
  - NO hidden ineligibility (backend-missing/VRAM-miss rows listed with reason, never dropped)
  - NO external perf figure presented as measured ([DESIGN] tags survive into the emitted rows)
  - NO .unwrap()/.expect() on prod paths; blake3 only; no new deps
frontend_pair:
  HardwareModelCatalogPanel (Tier-2, DEFERRED per DC-2 — recorded here + in the decisions doc;
  the Tier-1 surface is the journaled SSB rows; W-6 justification: the panel ships with the CLI
  subcommand in the registered Tier-2 checkbox, not silently dropped).
observable_effect:
  trigger: "elai run (startup probe) on this box; then HardwareModelCatalog::build over the five
    fixture probes (8/12/16/24/48GB) in the contract test; [Tier-2] elai hardware-catalog"
  expect_ssb: "one CatalogBuilt row carrying the detected tier + per-lane eligible counts +
    fit-scored ordering, recomputable from the journal under ELAI_SEED (GD-2); TierAssigned /
    EligibilityChecked rows per lane resolution"
  expect_artifact: "on the author's 12GB box the coding lane resolves micro_7b-class entries
    eligible and large_32b ineligible with reason=vram; the Gemma-4 row appears ONLY on >=24GB
    fixture probes WITH its residue note intact. NEGATIVE (R2): a mocked zero-VRAM probe yields
    cloud-lane-only eligibility plus an honest-residue note (never an empty/silent catalog); a
    catalog entry claiming eligibility above the fixture's VRAM FAILS the contract test; grep
    proves zero routing-path symbols consume the catalog (JC-10 — informational-only is
    structurally enforced)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-9_hardware_detection_model_catalog.md --run-trigger
rg "model_catalog.toml" crates/orchestrator/src/hardware_catalog.rs   # data-driven, wired
rg -c "Qwen|Gemma|InternVL|Phi" crates/orchestrator/src/hardware_catalog.rs   # MUST be 0 (names live in toml only)
rg "classify_vram_tier|can_fit_llm" crates/orchestrator/src/hardware_catalog.rs   # consumes shipped probes
rg -c "\.unwrap\(\)|\.expect\(" crates/orchestrator/src/hardware_catalog.rs   # MUST be 0 outside tests
rg -L "hardware_catalog" crates/orchestrator/src/cloud_mode.rs crates/providers/src/runtime/escalation_policy.rs || true   # routing does NOT consume the catalog (JC-10)
test -f data/hardware_catalog/model_catalog.toml
test -f workflow/reasoning/J-9-decisions.md
```

## ACCEPTANCE
- [ ] Catalog builds from the live probe at startup; CatalogBuilt row lands (JD-1 analog:
      off-agent journal artifact, not stdout).
- [ ] All five fixture probes (8/12/16/24/48GB) resolve per-lane lists matching the seed data;
      zero-VRAM probe degrades to cloud-only + residue note.
- [ ] Gemma-4 / Qwen3-32B rows are tier-conditional WITH the HW-14 residue note verbatim;
      metrics/baseline_findings_and_actions.md and CLAUDE.md untouched (DC-3).
- [ ] Catalog is informational-only: no routing symbol consumes it (grep-proven); no install or
      pin mutation paths exist.
- [ ] Ineligible rows carry reasons (vram | backend_missing | platform); nothing silently dropped.
- [ ] [DESIGN] source tags survive into emitted rows; no external figure presented as measured.
- [ ] SSB 6-part discipline complete; baseline re-read at edit (map: 226/537 as of 2026-06-11).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator (scoped).
- [ ] No new dependency; odysseus consulted read-only (no import, no copied weights).
- [ ] Tier-2 CLI/panel registered as a deferred checkbox in the decisions doc (DC-2), not dropped.

## WIRING DONE-GATE (clause by clause)
- W-1: done = CatalogBuilt landing off the REAL startup probe path + the fixture matrix passing,
  not build-green.
- W-2: Tier-1 ships default-ON on the startup path (no enable flag); Tier-2 is a registered
  deferred checkbox, not a dark flag.
- W-4/W-6: frontend_pair deferred WITH the Tier-2 surface per DC-2 — justified, recorded.
- W-5/W-10: live startup trigger now; the GPU-zoo breadth (multi-GPU, exotic cards) is the
  honest untested residue, not a gated checkbox (see risks).
- W-7: HW anchors re-grepped at edit; the startup emit site located by grep, never assumed.
- W-9: SSB baseline re-read at edit.
- GD-1: consumes HW-1..HW-9; re-implements none (stub_ban enforced by the deterministic check).
- GD-2: catalog resolution + fit ordering recompute byte-identically from probe fixtures under
  ELAI_SEED.
- GD-5: VRAM footprints are quantization-dependent ESTIMATES; [DESIGN] rows say so; the HW-14
  residue note ships verbatim; fit-score weights are ELAI-derived and documented.
- JD-1..4: N/A (no harness leg); justified.

## Tests to add
- crates/contracts/tests/j9_hardware_catalog_contract.rs (~160 LOC):
  (a) five fixture probes resolve the expected per-lane eligibility (golden toml fixtures);
  (b) zero-VRAM probe -> cloud-only + residue note (never empty, never panic);
  (c) an entry whose min_vram_mb exceeds the fixture VRAM asserting eligible=true FAILS;
  (d) Gemma-4 row absent at 12GB fixture, present at 48GB fixture WITH residue note;
  (e) backend_missing rows listed with reason (E-51 placeholder lanes);
  (f) corrupt/missing model_catalog.toml degrades to probe-only CatalogBuilt with
      lanes_unavailable reason (fail-visible, never silent);
  (g) routing-isolation: the catalog type is not referenced from cloud_mode/escalation_policy
      (compile-time module-boundary assert or grep-based contract).

## Files touched
- CREATE: crates/orchestrator/src/hardware_catalog.rs (~180 LOC)
- CREATE: data/hardware_catalog/model_catalog.toml (seed lanes)
- MODIFY: the startup probe call site (+~10 LOC emit; exact file re-grepped at edit, W-7)
- MODIFY: ssb.rs + action_type_lock.rs et al (6-part, +1 variant)
- CREATE: crates/contracts/tests/j9_hardware_catalog_contract.rs (~160 LOC)
- CREATE: workflow/reasoning/J-9-decisions.md
- [Tier-2, deferred] crates/cli subcommand + dashboard panel + cross_lang_diff pass

## Citations / Grounding
- ELAI ground truth: HW-1..HW-13 (_grounding/J-PERCEPTION-GROUNDING-MAP.md; live greps
  2026-06-11); LIVE 226/537.
- HW-14 policy shift: user statement 2026-06-11, carried verbatim; DC-3 bounds it to catalog data.
- odysseus prior art: services/hwfit/fit.py:73/143/198/299/521 via odysseus-INDEX.md:64
  (read-only structural reference, GD-1 pointer).
- VR-13/VR-11 vision ladder ([DESIGN] seeds; J-8 eval refreshes).
- E-31 tier matrix (docs/strategy/E-31-tier-matrix.md) — the coding-lane transcription source.
- CLAUDE.md hard rules invoked: no-new-deps, blake3-only, anti-slopsquat (every catalog model_id
  is data + tagged, never an import), HARDWARE-HEAVY (the catalog never boots anything).

## Open questions / risks
- Multi-GPU and CPU-offload boxes: untested residue — the catalog reads the primary GpuInfo;
  named in the emitted residue note (GD-5). Extend when such a fixture exists.
- VRAM footprint estimates vary by quantization/runtime (NF4 vs AWQ vs GGUF); rows carry
  per-quant footprints but real headroom depends on KV cache + context — the estimate is
  labelled an estimate.
- The startup emit site: probe call locations may be plural (CLI vs dashboard boot); decide the
  single canonical emit at edit (W-7) — duplicate CatalogBuilt rows per boot are a FAIL.
- vision_ladder.toml ownership with J-8 (J-8 task 8 writes it; J-9 reads it): if J-9 ships
  first, the vision lane seeds directly from VR-13 and J-8's file becomes the refresh source.

## Next
Closes the "what can this box run" gap with journaled, data-driven eligibility; the Tier-2
CLI/panel makes it user-visible; a later user-gated session may promote catalog verdicts into
routing defaults (explicitly out of scope here, JC-10/DC-3) — and the J-8 eval rows upgrade the
vision/OCR lanes from [DESIGN] to [MEASURED].

End of J-9.
