# J-8: vlm perception research upgrade pack — the concrete tables, schema rules, residues, and eval lines from the 3-file vision corpus

STATUS: PLANNED (WAVE 3+; UPGRADE on J-1's gateway — MUST NOT execute before J-1 is DONE (DC-1);
Tier-1 = rules/schema/data, Tier-2 = the two model EVALs, sidecar/RunPod-gated W-10)

Objective: J-1 ships the perception gateway with ABSTRACT rules ("resize to the target model's
sweet spot", "small-VLM probe") because at authoring the concrete values lived only in the research
corpus. J-8 lands the concrete layer the corpus provides: the per-model resize table with the
native-resolution passthrough rule (VR-1), the normalized 0-1000 coordinate rule as a hard scaffold
JSON-schema constraint (VR-2), the counts-from-AXTree/DOM-never-VLM rule ported from harness law
into the runtime composer (VR-3), multi-image + micro-text honest-residue routing (VR-4/VR-5), the
MoAI design citation (VR-6), the image-cache x I-10 prefix-invariance composition guard (VR-7), an
EVALUATE harness for dots.ocr and Qwen3-VL-Embedding against the canonical E-36 lane (VR-8/VR-9,
JC-7/JC-8), the H-14 component-inventory prompt shape as a library template (VR-10), and the
FastVLM/vision-ladder catalog data handed to J-9 (VR-11/VR-13). Hard boundaries: J-8 NEVER edits
the J-1/E-36/H-14/E-51/B-24 plan files (user hard constraint, DC-1); it never re-implements the
gateway structs (GD-1 — it upgrades J-1's shipped surface); every corpus figure stays [DESIGN]
(HD-1/ID-5) — the done-gate produces ELAI-measured deltas only.

Estimated effort: ~1 session (~6-8 h) Tier-1; +1 gated sub-session for the two EVAL runs
(remote sidecar, batch discipline, pod_stop after).
Pre-requisites: J-1 DONE (the gateway structs + PerceptionScaffoldEvent exist; re-grep exact
paths at edit, W-7). For the EVAL legs: anti-slopsquat resolution of JC-7/JC-8 + healthy sidecar.
Parallel-safe with: J-3/J-4/J-5/J-6/J-7 (disjoint). NOT J-1 (same module), NOT J-9 (J-8 hands
catalog seed data to J-9's vision/OCR lanes; sequence or coordinate the seed file).
Source: the 3-file corpus + condensed summary (VR preamble); _grounding/J-PERCEPTION-GROUNDING-MAP.md.
Doctrine: J-0_INDEX.md.

## Context
The corpus is the post-J-1 refinement layer: J-1's four legs encode the MECHANISMS (preprocess,
hybrid compose, scaffold, route); the corpus supplies the calibration DATA and the failure-mode
boundaries those mechanisms need to stop being placeholders. Three kinds of content land here:
(a) DATA — the resize sweet-spot table, the per-VRAM vision ladder, prompt-shape templates;
(b) RULES — 0-1000 coords in the scaffold schema, counts-from-DOM in the composer, native-res
passthrough in the router, multi-image/micro-text routing with typed refusal;
(c) EVALS — two candidate models (dots.ocr for OCR, Qwen3-VL-Embedding for vision-RAG) measured
against the canonical lane before any adoption. Everything quantitative in the corpus is external
([DESIGN]); the session's own outputs are fixture-measured deltas.

## Existing state (B..J coverage — why this is an UPGRADE, not a build)
- J-1 (PLANNED, WAVE 3): PerceptionGateway / PreprocessSpec / scaffold / router + 
  PerceptionScaffoldEvent. J-8's entire surface hangs off these structs. [J-plan]
- B-24 media_validator (SHIPPED): Tier-1 deterministic veto + Tier-2 VLM advisory (Mode A
  Qwen3.5-VL-30B-A3B / Mode B InternVL3.5-38B SGLang multi-image). VR-4's multi-image escalation
  target. Untouched by J-8.
- E-36 lane (PLANNED; substrate partially in-tree): ColQwen2.5-v0.2 / PaddleOCR-VL named at
  crates/knowledge/src/chunking/image_artifact.rs:45 (HW-13). The EVAL baseline lane; canonical
  until measured otherwise.
- visual-ui-loop.md (harness law): counts-never-visual, describe-before-verdict, 1568px prep —
  already binding for HARNESS image reads; the runtime composer has no equivalent until J-8.
- I-10 (PLANNED): prefix-invariance + hit-rate hard guard — the enforcement layer VR-7 composes
  with; J-8 adds the image-block placement assert, not the guard.
- H-14 / E-51 (PLANNED): consumers of the prompt-shape template (VR-10) and the FastVLM note
  (VR-11) respectively, when they execute. J-8 ships the template/data into ITS OWN files only.
- SSB: LIVE 226/537 (map CANONICAL CONSTANTS); J-1 adds PerceptionScaffoldEvent first.

## Reads (mandatory before coding)
1. _grounding/J-PERCEPTION-GROUNDING-MAP.md (whole map — VR-1..14, JC-7..11, DC-1..3).
2. The 3 corpus files in C:/Users/user/Downloads/ (sections named per VR id) — the tables are
   transcribed into code/data from the ORIGINALS, not the condensed summary (W-3 analog).
3. crates/orchestrator/src/perception/ as shipped by J-1 (re-grep every struct, W-7).
4. crates/knowledge/src/chunking/image_artifact.rs:45 region (the EVAL baseline lane, HW-13).
5. visual-ui-loop.md counts/describe rules (the harness law being ported).
6. CLAUDE.md anti-slopsquat protocol (JC-7/JC-8 resolution before any model name lands in code).

## Tasks (WAVE 3+ — Tier-1 first)
1. Sweet-spot table as DATA (~60 LOC + toml): crates/orchestrator/src/perception/sweet_spot.rs
   reading data/perception/sweet_spots.toml — per-model-family {longest_side_px, native_res:
   bool}. Claude 1568 / GPT-4o 2048 / Qwen-VL 1280 / Gemini native (VR-1), provider-doc re-check
   noted per row. Router consumes: native_res=true -> passthrough, never force-resize. NOT
   hardcoded constants (stub_ban).
2. Scaffold schema v2 (~40 LOC): the call-1 JSON schema gains a REQUIRED coords contract —
   [ymin,xmin,ymax,xmax] normalized 0-1000 (VR-2); a raw-pixel-looking emission (any value >1000
   or schema-tagged px) FAILS validation and re-prompts once, then types a refusal. Instruction-
   before-image stays asserted.
3. Composer counts rule (~50 LOC): a counting-shaped ask (>10 elements) answered from the
   AXTree/DOM leg or a deterministic script; the VLM is never asked to count (VR-3). When no DOM
   leg exists for the image class -> typed refusal (D-16 target), not a guess.
4. Multi-image + micro-text routing (~50 LOC): N>1 evidence images -> tiled-composite (VR-14
   shape) or B-24 Mode B escalation (VR-4); micro-text asks -> tiling then confidence-gated typed
   refusal (VR-5). Both emit their routing Kind; both carry the GD-5 residue note in the payload.
5. Cache-placement assert (~20 LOC): debug_assert the image+scaffold block sits AFTER the last
   cache breakpoint (HD-2; VR-7); composes with I-10's guard when it ships — J-8 adds no guard of
   its own (GD-1).
6. EVAL harness (Tier-2, gated): crates/cli/src/bench_vlm_eval.rs (`elai bench vlm-eval --lane
   ocr|vision-rag`) — fixture corpus vs the canonical lane (HW-13 baseline) vs the candidate
   (dots.ocr / Qwen3-VL-Embedding) AFTER JC-7/JC-8 anti-slopsquat resolution. Output: measured
   accuracy/latency rows in a metrics TOML + EvalScheduled/result SSB rows. ADOPT is a separate
   user-gated decision; the harness only measures (JC-10 analog).
7. Prompt-shape library (~30 LOC + md): perception/prompt_shapes.rs carrying the component-
   inventory template {type, verbatim text, hierarchy, layout cues} (VR-10) + the MoAI citation
   string (VR-6) — exported for H-14's future consumption; H-14's file untouched.
8. Catalog seed handoff: write the VR-13 vision ladder + VR-11 FastVLM row into
   data/perception/vision_ladder.toml for J-9's vision/OCR lane seeding (single shared file;
   coordinate with J-9).
9. SSB: +VlmPerceptionUpgradeEvent (Pattern-2 single-variant; Kind: SchemaApplied /
   ResizeRuleApplied / RouteRuleApplied / EvalScheduled), baseline+1 from LIVE 226/537 — UNLESS
   J-1 shipped a carrier field on PerceptionScaffoldEvent (prefer-reuse, JC-11; decide at edit).
   Full 6-part variant discipline either way.
10. Decision doc: workflow/reasoning/J-8-decisions.md (table values chosen + provider-doc
    re-check results, schema v2 shape, eval candidate resolution status, what stayed
    external-figure-only).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/perception/sweet_spot.rs + data/perception/sweet_spots.toml (VR-1 data)
  - scaffold schema v2 (0-1000 coords contract) in perception/scaffold.rs (VR-2)
  - composer counts/multi-image/micro-text routing rules in perception/router.rs (VR-3/4/5)
  - perception/prompt_shapes.rs (VR-10 template + VR-6 citation)
  - data/perception/vision_ladder.toml (VR-13/VR-11 — J-9 seed)
  - crates/cli/src/bench_vlm_eval.rs (Tier-2 EVAL harness)
  - +VlmPerceptionUpgradeEvent SSB (baseline+1; or the JC-11 prefer-reuse payload extension)
consumes_symbols:
  - J-1 PerceptionGateway / PreprocessSpec / scaffold / router structs (upgrade target — never
    re-implemented; J-8 has no standalone gateway path)
  - B-24 media_validator Mode B entry (VR-4 escalation target — feed, never re-implement)
  - image_artifact.rs:45 ColQwen2.5/PaddleOCR-VL lane (the EVAL baseline, HW-13)
  - D-16 refusal path (typed refusal for counts/micro-text over-limit asks)
  - I-10 cache guard when shipped (the VR-7 enforcement layer; J-8 only places + asserts)
reachable_from:
  - the image-bearing agent-dispatch seam via J-1's gateway (the upgraded rules fire on every
    gateway pass)  [PRODUCTION dispatch path; exact site re-grepped at edit, W-7]
  - `elai bench vlm-eval` (Tier-2 EVAL entrypoint)
stub_ban:
  - NO hardcoded sweet-spot constants (data-driven toml; stub_ban on magic px numbers in code)
  - NO VLM counting call on a counting-shaped ask (the rule IS the deliverable)
  - NO adoption of dots.ocr / Qwen3-VL-Embedding without a measured eval row + user yes (JC-7/8/10)
  - NO corpus figure in any user-facing claim or metrics TOML (measured-only, HD-1)
  - NO edit to J-1/E-36/H-14/E-51/B-24 plan files (DC-1); NO re-implementation of gateway structs
  - NO .unwrap()/.expect() on prod paths; NO new dep without explicit approval; blake3 only
frontend_pair:
  upgrade-rule + eval rows surface in the same drawer tile J-1 chose for perception rows (W-4
  rides J-1's pair; if J-1 justified W-6 absence, J-8 inherits the justification — restate it in
  the decisions doc).
observable_effect:
  trigger: "elai run over J-1's 4K-screenshot fixture with the upgrade pack live; then a
    counting-shaped fixture ask; then `elai bench vlm-eval --lane ocr` (Tier-2, gated)"
  expect_ssb: "VlmPerceptionUpgradeEvent rows — SchemaApplied (scaffold v2 validated 0-1000
    coords), ResizeRuleApplied (table-resolved edge or native passthrough), RouteRuleApplied
    (counting ask routed to DOM leg) — recomputable from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "the journaled call-1 reply validates against schema v2 (all coords 0-1000);
    the counting fixture's answer provably derives from the AXTree leg (zero VLM count call in
    the trace); the native-res fixture (gemini-class target) shows NO resize op in the journal.
    NEGATIVE (R2): a --legacy-scaffold arm emitting raw-pixel coords FAILS schema validation
    (the gate bites); a multi-image fixture forced down the single-image path produces the
    degraded answer the routed arm avoids (proves the route discriminates); an eval run against
    an unresolved candidate name (JC-7 unmet) REFUSES to start."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-8_vlm_perception_research_upgrade.md --run-trigger
rg "sweet_spots.toml" crates/orchestrator/src/perception/     # data-driven table wired, not constants
rg -c "1568|2048|1280" crates/orchestrator/src/perception/*.rs   # MUST be 0 (px values live in toml only)
rg "0-1000|normalized" crates/orchestrator/src/perception/scaffold.rs   # schema v2 contract present
rg -c "\.unwrap\(\)|\.expect\(" crates/orchestrator/src/perception/ crates/cli/src/bench_vlm_eval.rs   # MUST be 0 outside tests
test -f data/perception/vision_ladder.toml                    # J-9 seed handoff exists
test -f workflow/reasoning/J-8-decisions.md
```

## ACCEPTANCE
- [ ] J-1 DONE verified before any J-8 edit (DC-1; the gateway structs exist at the re-grepped paths).
- [ ] Sweet-spot values live in toml; code carries zero magic px numbers; native-res passthrough on.
- [ ] Scaffold schema v2 rejects raw-pixel coords (re-prompt once, then typed refusal).
- [ ] Counting fixture answered from the DOM leg; VLM count calls = 0 in the trace.
- [ ] Multi-image + micro-text fixtures route per VR-4/VR-5 with residue notes in the payload.
- [ ] Cache-placement debug_assert holds on a multi-turn fixture (HD-2).
- [ ] EVAL harness measures-only; no adoption row without user yes; JC-7/JC-8 resolved before
      any candidate name lands in code.
- [ ] No corpus figure appears as an ELAI claim anywhere (HD-1/ID-5).
- [ ] SSB 6-part discipline complete (or the JC-11 prefer-reuse extension); baseline re-read at edit.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-cli (scoped).
- [ ] No new dependency (eval candidates are config rows until separately approved).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the observable_effect trigger PASSes live, not build-green.
- W-2: rules ship default-ON on the gateway path; the EVAL harness is an entrypoint, not a flag.
- W-3: tables transcribed from the 3 ORIGINAL corpus files, not the summary.
- W-4: frontend_pair rides J-1's perception tile (or inherits its W-6 justification).
- W-5/W-10: rules+schema live now; the two model EVALs are the gated Tier-2 checkbox.
- W-7: every J-1 struct path re-grepped at edit; corpus tables re-checked against provider docs.
- W-9: SSB baseline re-read at edit (map says 226/537 as of 2026-06-11).
- GD-1: composes with J-1/B-24/E-36-lane/D-16/I-10; re-implements none.
- GD-2: every emitted row recomputable from the journal under ELAI_SEED.
- GD-5/HD-1: external figures tagged [DESIGN]; residues (VR-4/VR-5) named in-file and in-payload.
- JD-1..4: N/A (no harness leg in J-8 — the /see skill belongs to J-1); justified.

## Tests to add
- crates/contracts/tests/j8_perception_upgrade_contract.rs (~140 LOC):
  (a) sweet_spot table resolves per model family; native_res=true yields passthrough (no resize op);
  (b) scaffold v2 rejects a >1000 coord emission; re-prompt-once then typed refusal;
  (c) a counting-shaped ask dispatches the DOM leg; asserting a VLM count call FAILS the test;
  (d) multi-image fixture routes to composite/Mode-B; the forced single-image arm differs (R2);
  (e) eval harness with an unresolved candidate name (JC-7 unmet) returns a typed refusal;
  (f) corrupt sweet_spots.toml (missing family) degrades to DegradedPassthrough, never panics.

## Files touched
- MODIFY: crates/orchestrator/src/perception/{scaffold,router}.rs (~140 LOC: schema v2 + rules)
- CREATE: crates/orchestrator/src/perception/{sweet_spot,prompt_shapes}.rs (~90 LOC)
- CREATE: data/perception/{sweet_spots,vision_ladder}.toml
- CREATE: crates/cli/src/bench_vlm_eval.rs (~150 LOC, Tier-2)
- MODIFY: ssb.rs + action_type_lock.rs et al (6-part, +1 variant; or JC-11 prefer-reuse)
- CREATE: crates/contracts/tests/j8_perception_upgrade_contract.rs (~140 LOC)
- CREATE: workflow/reasoning/J-8-decisions.md

## Citations / Grounding
- VR-1..VR-14 (_grounding/J-PERCEPTION-GROUNDING-MAP.md) — the corpus extraction with sources;
  all corpus figures [DESIGN], re-verified at execution (provider docs drift).
- arXiv:2403.07508 (MoAI), arXiv:2504.10479 (InternVL3), arXiv:2603.12823 (AVR),
  arXiv:2505.15146 (lmgame-Bench) — web-verified 2026-06-11 (map WEB VERIFICATION block).
- ELAI ground truth: HW-13 (image_artifact.rs:45 baseline lane), J-1's contract (the upgrade
  target), LIVE 226/537.
- CLAUDE.md hard rules invoked: anti-slopsquat (JC-7/JC-8), no-new-deps, blake3-only, HD-1
  measured-only, token discipline.

## Open questions / risks
- JC-7/JC-8: exact dots.ocr / Qwen3-VL-Embedding package+HF names unresolved — EVAL leg blocked
  until anti-slopsquat passes; mitigation: Tier-1 ships independently.
- Sweet-spot drift: provider preprocessing changes with model releases; mitigation: values in
  toml with a per-row re-check date, re-verified at execution.
- J-1's final struct shapes may differ from plan (it is PLANNED, not DONE) — every anchor here is
  re-grepped at edit (W-7); if J-1 ships the JC-11 carrier field, the SSB delta drops to zero.
- Schema-v2 re-prompt loop cost: bounded at one re-prompt then refusal; measured on fixtures
  before any tightening.

## Next
Closes the corpus-to-runtime gap J-1 deliberately left abstract; hands J-9 its vision/OCR lane
seed (vision_ladder.toml); the EVAL rows, once measured, feed the J-9 catalog and any future
B-24 Mode-A/B re-pin decision (user-gated).

End of J-8.
