# J-77: perception faithfulness via the trust spine — route the J-1/J-8 perception scaffold THROUGH H-53 citation-prebind + G-1 taint + G-2 ledger + the DKG graph shape

- Track J (vision-orchestration wave) | lever CANT-MISREAD | grounding STRONG (deep-research vision report tiled-to-text; FastVLM/InternVL/PaddleOCR-VL/Qwen3-VL/ColPali figures DESIGN-only; the faithfulness primitives are ELAI's own shipped trust spine) | nature EXTENSION (J-1 + J-8) | effort M-L | WAVE VO-1 | depends none (EXTENDS J-1 + J-8; authored as a NEW file per the house never-amend-old-plans rule — the source doc's "fold into J-1/J-8 task additions" becomes a new file that extends them)

- Premise: the vision deep-research report's bottom line IS ELAI's existing perception doctrine, independently arrived at (deterministic-first OCR/layout/detectors own text/coords/counts, small VLM only on the semantic residue, fuse into strict JSON with 0-1000 coords, hand the blind consumer a STRUCTURE not prose). So the report is CONFIRMATION of J-1/J-8's direction. The LEVERAGE point: faithful vision REQUIRES evidence-grounding, per-element provenance, a canonical coordinate frame, and a layout graph — and ELAI ALREADY OWNS all four primitives in OTHER subsystems (H-53 citation pre-binding, G-1 taint Provenanced<T>, G-2 RRA transform/receipt ledger, the DKG graph engine). The improvement is NOT "build a vision system" — it is "route the perception scaffold THROUGH the existing trust spine." Pure GD-1 composition; the highest-value, lowest-mechanism win.

## Steal/build (priority order; cheapest + highest-leverage first)
1. I5 [STRONGEST] CITATION-PREBIND OVER REGIONS: compose H-53/citation_prebind.rs over the perception region set. The deterministic OCR/layout/detector detections ARE the EvidenceAnchorSet; a VLM description referencing a non-existent region fails the closed-set membership check -> D-16 typed refusal / routed to an `unverified_claims` channel. A "ghost region" is a "ghost citation" — structurally impossible by the SAME machinery ELAI shipped for text. ZERO new mechanism.
2. I6 PER-ELEMENT PROVENANCE via G-1 TAINT: add the image-perception path as a 5th G-1 Provenance ingest site (today 4: WebFetch/MCP/file/sandbox). Every atomic item gets a `source` in {ocr, layout, detector, vlm, merged} + confidence; VLM-written fields split into semantic_label / appearance_notes / uncertain_visual_guess. `source` IS the taint label -> the capability_gate treats vlm-sourced fields as lower-trust; the downstream model + auditor see deterministic-grounded vs VLM-guessed.
3. I7 LAYOUT GRAPH FOR THE BLIND MODEL: emit the perception result as a GRAPH in the DKG shape (coordinates, reading order, adjacency, containment, z-order, region descriptors), NOT prose — with a deterministic consistency check (parent boxes contain children, reading-order edges don't cross impossibly, explicit counts == region-list length). The concrete answer to "GLM-5.2 is text-only": the blind model gets a verifiable layout graph, not a blind caption (Qianfan-OCR OCR+LLM baselines score 0.0 on CharXiv vs end-to-end vision — accurate PROSE still fails on 2D structure).
4. I4 CANONICAL COORDINATE FRAME + TRANSFORM LEDGER as an RRA-style artifact: define ONE canonical pixel space at ingest; every resize/crop/tile records its transform RELATIVE to that frame; reproject all boxes back and REJECT any element whose inverse transform is inconsistent (the report's "coordinate drift" failure). Structurally identical to the G-2 RRA receipt ledger — implement it AS an RRA-style transform ledger, not a bespoke thing. Deterministic, verifiable.
5. I3 TYPED MERGE GRAPH (fusion algorithm, GENUINE GAP): neither J-1 nor J-8 specifies HOW per-tile detections fuse. A typed merge graph words->lines->blocks->semantic-regions where only ONE level contributes to COUNTS, merge by IoU + text similarity + class agreement, counts/exact-text ALWAYS from deterministic detections never captions. Deterministic check: no two final regions of the same type exceed an IoU threshold unless their texts meaningfully differ (kills double-counting). TD-3-clean, fits ELAI's "counts are deterministic" law.
6. I8 RESOLVE J-8's OPEN EVAL with the report's candidate set: `elai bench vlm-eval` measures Qwen3-VL-4B (873 OCRBench / 94.9 DocVQA / 83.3 ChartQA) + InternVL3-2B (clean 448-tile path) + PaddleOCR-VL-0.9B (TEDS 0.9195 / RMS-F1 0.844; ~43.7GB server-config, re-measure local) + FastVLM (TTFT 152ms@768) against the canonical E-36 lane; dots.ocr DEMOTED to heavy-lane-only (78.5GB VRAM, ~20min/100pg on a 4090). ColQwen2/ColPali (ViDoRe 86.6/81.3) as the long-doc retrieval adjunct. All anti-slopsquat-resolved (JC-7/JC-8) before any name lands in code.
7. I1/I2 TILE TABLE + OVERLAP: add a tile_px column to J-8's sweet_spots.toml keyed to the DESCRIBER family (448 InternVL / 768-1024 no-tile FastVLM / native-dynamic Qwen2.5/3-VL) DISTINCT from the whole-image resize-sweet-spot; set tile_overlap 12.5% as a BOUNDARY-REPAIR heuristic (never the main strategy) + owner-tile policy (every OCR/detector box owned by the tile containing its CENTER; overlap rescues only truncated boxes). Honest caveat: exact best overlap % is NOT literature-settled — ship 12.5% config, measure on the J-8 fixture.
8. I9 REFUSAL ADDITIONS (extend VR-5): micro-text refusal when DUAL-OCR engines disagree materially (not just low single-engine confidence); chart/table refusal unless a dedicated parser extracted structure; the I5 fabrication check. A dual-OCR path (one modern engine + a second recognizer/doc-VLM verify ONLY on low-confidence spans) is the cheapest stylized/rotated/low-contrast defense.
9. I11 MEASURED 6-METRIC DONE-GATE (replaces J-1's abstract "answers the fine-text question"): >=98% exact-match OCR on legible text above a min font; >=95% region-grounding recall @IoU0.5 for text blocks + major controls; <1% duplicate final regions after fusion; >=90% exact-count for <=10 separated elements; ZERO uncited claims in the final schema (the I5 gate made measurable); blind-LLM QA materially beats an OCR-only baseline on chart/diagram/layout questions. ELAI-MEASURED per-config (SAFEDELTA-style believe-the-number), never the report's numbers.
10. I10 HYGIENE — DROP the unverified "instruction-before-image" assertion: J-1 leg-3 asserts instruction-before-image as settled; the report found NO strong benchmark evidence it universally improves faithfulness. Demote from assertion to "unverified folklore, measure if it matters"; lean on the J-8 VR-2 schema constraint (which the report DOES endorse). grounded-by-default hygiene fix.
EXCLUDED: building a perception system from scratch (J-1/J-8 already specify it; this routes it through the spine); a VLM on visual diffs (B-24 Tier-1 pixel-diff veto stands — visual diffs go to pixel-diff NEVER a VLM, HD-5); counts from captions (TD-3 — counts are deterministic detections); dots.ocr as a ship candidate (heavy-lane only); any describer model name in code before anti-slopsquat resolution (JC-7/JC-8).

## Existing ground truth (grep-confirmed 2026-06-23; re-grep at edit, W-7/W-9)
- PERCEPTION GATEWAY + /SEE [PLANNED(J-1)] preprocess resize/RGB/tiles -> hybrid composer AXTree+image -> 2-call scaffold temp-0 -> small-VLM probe router; visual diffs -> pixel-diff NEVER VLM; +PerceptionScaffoldEvent (feature map 1.6). leg-1 tile_overlap field has no value (I2); leg-3 asserts instruction-before-image (I10).
- VLM PERCEPTION UPGRADE [PLANNED(J-8)] per-model resize table as DATA (sweet_spots.toml + sweet_spot.rs), 0-1000 normalized coords schema-v2, counts-from-AXTree-never-VLM, dots.ocr/Qwen3-VL-Embedding EVAL-not-ship; +VlmPerceptionUpgradeEvent. open describer EVAL is I8; schema-v2 gains the I6 source/confidence fields.
- CITATION PRE-BINDING [PARTIAL] H-53 citation_prebind.rs: freeze ranked EvidenceAnchorSet BEFORE generation, [N] = closed-set membership, out-of-set -> D-16 refusal, ghost citations structurally impossible. I5 composes this over regions.
- TAINT PROPAGATION [WIRED] G-1: Provenance at 4 ingest sites, Provenanced<String>, capability_gate before sinks. I6 adds the perception 5th site.
- REPRODUCIBLE-RUN ARTIFACT (RRA) [WIRED] G-2 transform/receipt ledger. I4 implements the coordinate-transform ledger AS an RRA-style artifact.
- DKG [WIRED] tree-sitter -> SQLite typed-edge graph. I7 emits the perception layout in the DKG graph shape.
- CONFORMAL REFUSAL D-16 [DARK] the typed-refusal target for I5/I9.
- MEDIA VALIDATOR [WIRED] B-24 Tier-1 deterministic veto (the pixel-diff-not-VLM doctrine I5/HD-5 preserve).

## REVISED scope (W-10 tiered)
- SHIP NOW Tier-1 (deterministic, model-free, pure composition): I5 (citation-prebind over regions), I6 (taint provenance source field), I3 (typed merge graph), I4 (canonical-frame transform ledger), I7 (layout graph), I1/I2 (tile table + overlap config), I10 (drop the folklore), I11 (the measured 6-metric gate harness on the J-8 fixture).
- DEFER Tier-2 (sidecar/VLM + RunPod): I8 (`elai bench vlm-eval` live describer measurement — remote-GPU-gated, pod_stop after, anti-slopsquat JC-7/JC-8 before any name in code), I9 dual-OCR verify on low-confidence spans (the second engine/doc-VLM is model-dependent).

## Integration
- crates/orchestrator/src/perception/ (J-1) — the citation-prebind-over-regions wire (compose H-53, ~60 LOC), the canonical-frame transform ledger (compose G-2 RRA, ~120 LOC), the typed merge graph fusion.rs leg (new, deterministic, ~150 LOC), the layout-graph emitter in the DKG shape (~100 LOC).
- crates/knowledge or perception preprocess.rs — the I4 transform ledger + reprojection-consistency check.
- data/perception/sweet_spots.toml + sweet_spot.rs (J-8 task 1) — the tile_px column + tile_overlap 12.5% + owner-tile policy (I1/I2).
- scaffold.rs schema-v2 (J-8 task 2) — the I6 source/confidence fields; G-1 Provenance constructor at the perception ingest site (the 5th site).
- `elai bench vlm-eval` (I8, deferred) + the I11 6-metric fixture gate.
- SSB: PREFER-REUSE J-1 PerceptionScaffoldEvent + J-8 VlmPerceptionUpgradeEvent; add the I6 `source` field + an `unverified_claims`/ghost-region Kind on the existing perception event (net 0..+1 from LIVE ~269/578; re-read at edit, W-9).
- cross_lang pair: the sidecar describer route (server.py) <-> the Rust perception client (cross_lang_diff for I8); frontend_pair: the perception scaffold/region breakdown on the VisionRAGDrawer (E-36) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/perception/ citation-prebind-over-regions (H-53 composed; ghost-region = ghost-citation -> D-16 refusal / unverified_claims channel)
- perception/fusion.rs typed merge graph (words->lines->blocks->regions; one level owns counts; IoU+text+class merge)
- perception canonical-frame transform ledger (RRA-style; reproject + reject inconsistent boxes)
- perception layout-graph emitter (DKG shape: coords/reading-order/adjacency/containment/z-order + consistency check)
- sweet_spots.toml tile_px column + tile_overlap 12.5% + owner-tile policy; scaffold.rs schema-v2 source/confidence fields
- `elai bench vlm-eval` (I8, deferred) + the I11 6-metric done-gate harness
consumes_symbols:
- H-53 citation_prebind.rs (the closed-set membership check; NOT re-authored — composed over regions)
- G-1 Provenance / Provenanced<T> / capability_gate (the 5th ingest site; NOT re-authored)
- G-2 RRA receipt ledger (the transform-ledger shape; NOT re-authored)
- DKG graph engine (the layout-graph shape; NOT re-authored)
- D-16 RefusalDecision (the typed refusal target; NOT re-authored)
- J-1 perception scaffold + J-8 sweet_spot.rs/scaffold.rs (EXTENDED, not duplicated)
reachable_from:
- a real `elai see` / perception query routed through crates/orchestrator/src/perception/  [PRODUCTION]
- `elai bench vlm-eval` (I8) CLI entrypoint  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO VLM-sourced count or exact-text (TD-3 — counts/exact-text are deterministic detections; captions never)
- NO described region without a backing detection (I5 — ghost region = ghost citation -> refusal/unverified, structurally impossible)
- NO VLM on a visual diff (HD-5 / B-24 Tier-1 veto — pixel-diff never a VLM)
- NO instruction-before-image assertion (I10 — demoted to measure-if-it-matters; the VR-2 schema constraint is the kept lever)
- NO describer model name in code before anti-slopsquat resolution (JC-7/JC-8); dots.ocr is heavy-lane-only never a ship candidate
- NO report number as the done-gate (I11 is ELAI-MEASURED per-config; HD-1/ID-5)
- NO new default-OFF ELAI_*_ENABLE without a wire-dark-allowlist row + promotion plan (R9/W-2)
frontend_pair: perception region/scaffold breakdown on VisionRAGDrawer (E-36) (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "a real `elai see` on a dense screenshot/chart with a fabrication-bait region (a control the VLM would hallucinate), run through the perception path with citation-prebind on; AND the I11 6-metric fixture run"
- expect_ssb: "PerceptionScaffoldEvent / VlmPerceptionUpgradeEvent carrying per-element source={ocr|layout|detector|vlm|merged}; a VLM-described region with no backing detection lands in the unverified_claims channel / fires a D-16 refusal"
- expect_artifact: "the final schema has ZERO uncited claims (every element maps to a deterministic detection or is marked unverified); the layout graph passes the containment/reading-order/count-consistency check; the transform ledger reprojects every box consistently. NEGATIVE (fail-closed): a fabricated region is DROPPED/routed-to-unverified (never silently described); a coordinate-drift box FAILS reprojection and is rejected; counts come from the AXTree/detections, the VLM cannot inflate them; the 6-metric gate FAILS the config if duplicate-region >1% or count-accuracy <90%."

## Grounding
- vision deep-research report (C:\Users\user\Downloads\deep-research-report(3).md, tiled-to-text) + the sonnet fugu-research leg — the failure taxonomy (coordinate drift / stylized-text / double-counting / fabrication / blind-prose-on-2D) and the candidate ranking. All model figures DESIGN-rationale only (HD-1/ID-5); ELAI re-measures.
- FastVLM (TTFT 152ms@768, raising resolution essential / AnyRes-tiling not automatically optimal / fewer tiles better) + InternVL (448-tile + use_thumbnail) + PaddleOCR-VL-0.9B (TEDS 0.9195 / RMS-F1 0.844) + Qwen3-VL-4B (873 OCRBench) + dots.ocr (heavy: 78.5GB, ~20min/100pg) + ColQwen2/ColPali (ViDoRe 86.6/81.3) — figures DESIGN-only, anti-slopsquat before any name in code.
- ELAI trust-spine primitives (feature map 1.6): H-53 citation_prebind.rs, G-1 G-2, DKG, D-16, B-24 — all shipped/partial, the four primitives I5/I6/I7/I4 compose. Full synthesis: workflow/reasoning/vision-and-orchestration-improvements-2026-06-23.md PART A.
- LIVE SSB ~269/578 (post-J-66; grep 2026-06-22). re-grep at edit (W-9).

## COMPOSES-WITH
J-1 (the perception gateway this routes through the spine — EXTENDED), J-8 (the resize table / schema-v2 / describer EVAL this fills out — EXTENDED), H-53 (citation pre-binding over regions — composed), G-1 (taint provenance 5th site), G-2 RRA (the transform ledger shape), DKG (the layout-graph shape), D-16 (typed refusal), B-24 (the pixel-diff-not-VLM veto). Restates none — this is GD-1 composition end to end. _grounding/J-PERCEPTION-GROUNDING-MAP.md (the existing J-1/J-8 grounding map) is the per-track anchor; re-grep at edit.

## Open questions / risks
- describer EVAL (I8) is RunPod-gated: the candidate measurement needs GPU + anti-slopsquat resolution of every model name (JC-7/JC-8) before a name lands in code — the deterministic legs (I5/I6/I3/I4/I7/I1/I2/I10/I11-harness) ship first and carry the value without a model decision.
- overlap % is not literature-settled (I2 honest caveat): ship 12.5% config + the owner-tile policy, measure on the J-8 fixture, do not present it as settled.
- the I11 gate needs a labelled fixture: the 6 metrics require a ground-truth fixture (legible-text/region/count/chart labels) — the first checkbox is building that fixture, not the model.

End of J-77.
