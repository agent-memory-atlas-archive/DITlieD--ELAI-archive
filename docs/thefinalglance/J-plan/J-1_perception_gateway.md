# J-1: perception gateway + /see skill — condition every inbound image before any model sees it

STATUS: PLANNED (WAVE 3; legs 1+3 Tier-1 now, legs 2/4 sidecar/VLM-gated W-10; BUILD — the only
user idea with no lane anywhere in B..I)

Objective: most production VLM failures are silent-downscale INPUT failures, not model failures —
yet ELAI ships zero inbound-image conditioning: B-24/E-16 validate OUTPUT media, E-36 ingests
documents, H-13 captures canvas screenshots into the validator (grounding-map SRC-1/SRC-3; novelty
re-verified against all of B..I in the pre-read log). J-1 ships a PerceptionGateway every
screenshot/diagram/mockup passes before ANY model reasons over it, plus a two-call perception
scaffold, plus a thin harness /see skill so the workflow's own image reads benefit. Hard boundary:
the gateway FEEDS B-24's validator and E-36's document leg — it never re-implements either (GD-1).
All external figures (30-60% preprocess recovery, ~0->40% scaffold, -78% router cost, ~60% JPEG
token savings) are design rationale ONLY; every shipped number is ELAI-measured (HD-1/ID-5).

Estimated effort: ~2 sessions (~12-16 h): legs 1+3 = M (model-free + cheap), legs 2/4 = M-L
(sidecar/VLM, RunPod batch discipline, pod_stop after).
Pre-requisites: none for legs 1+3. Legs 2/4: remote sidecar healthy (sidecar-protocol 3-step
check); B-24 Tier2 VLM path confirmed live at edit.
Parallel-safe with: J-3/J-4/J-6/J-7 (disjoint surfaces). NOT J-2 (shared cli dispatch seam).
Source: SRC-1 N-1 + SRC-3 (vlm-vision-research-summary.md). Doctrine: J-0_INDEX.md.

## Context
The research summary's mechanism chain: (1) silent provider-side downscale destroys fine text —
resizing to the TARGET model's sweet spot (Claude 1568px / GPT 2048 / Qwen-VL 1280 longest side)
before send recovers most of it; (2) PNG->RGB JPEG q85 cuts image tokens ~60% with negligible
quality loss; (3) dense UIs need overlapping tiles + a global thumbnail; (4) a literal-elements
description pass (temp 0, normalized 0-1000 coords, instruction BEFORE image) followed by the
actual reasoning call measurably unlocks hard visual tasks; (5) probe-small-escalate routing cuts
cost sharply at ~equal accuracy; (6) visual DIFFS belong to pixel-diff/Grounded-SAM-2 — never a
VLM (B-24's Tier-1 veto already encodes this); counting >10 elements and micro-text get tiling or
honest refusal. Figures external [SRC-3]; scope caveats carried in J-0 CARRIED WEB LANDMINES.

## Existing state (B..I coverage — why this is BUILD)
- B-24 media_validator: Tier1 deterministic checks + Tier2 VLM advisory + RCE loop — OUTPUT
  validation. SHIPPED (s-59964). The gateway is its INPUT-side sibling and feeds it.
- E-16: AXTree + anti-sycophancy add-ons on B-24 — still output-side.
- E-36: GroundingMethod::VisualArtifactCheck for documents (ColQwen/PaddleOCR) — the document
  route target, not an inbound conditioner.
- E-14: AXTree-first doctrine for browser E2E — the hybrid leg independently confirms it.
- H-13: canvas_screenshot -> Tauri capture -> media_validator Tier2 — a future gateway consumer.
- Harness: no /see skill (HG-2); image reads attach raw bytes today.
- SSB: LIVE 221/532 (RT-1); no perception action_type exists.

## Reads (mandatory before coding)
1. _grounding/J-HARNESS-GROUNDING-MAP.md (RT-1, JC-3, JC-4, SRC-3).
2. SRC-1 N-1 + SRC-3 in full (the leg definitions + the hard-limit routing table).
3. crates/orchestrator/src/media_validator/ (the Tier1 veto + Tier2 entry points to feed — re-grep
   exact paths, W-7).
4. The E-36 leg's dispatch site (route target) + H-13's canvas_screenshot site (consumer).
5. python/elai_inference/ server entry (where legs 2/4's heavy CV/VLM land — remote sidecar only,
   HARDWARE-HEAVY rule).
6. CLAUDE.md anti-slopsquat protocol (pillow for the harness leg, JC-4).

## Tasks (WAVE 3 — legs 1+3 first)
1. PerceptionGateway skeleton (~150 LOC): crates/orchestrator/src/perception/{mod,preprocess,
   scaffold,router}.rs. PreprocessSpec {target_model -> sweet-spot edge, jpeg_q85, tile_overlap,
   thumbnail}. Pure-code leg 1 (image ops via the already-present image toolchain on the sidecar
   for heavy work; the Rust side orchestrates + records). NOT a new image-processing dep in Rust
   without approval.
2. Perception scaffold (leg 3, ~120 LOC): call 1 = describe-literal-elements to structured JSON
   (temp 0, 0-1000 coords, instruction-before-image), call 2 = the task model reasons over
   description+image. Scaffold fires ONLY on image-bearing dispatches; injected AFTER the last
   cache breakpoint (HD-2 — never busts the byte-stable prefix).
3. Hard-limit router table (~60 LOC): visual-diff -> pixel-diff path (B-24 Tier-1 veto reused);
   counting>10 / micro-text -> tiling or typed refusal (D-16 refusal target, never a guess).
4. SSB: +PerceptionScaffoldEvent Pattern-2 single-variant (Kind: Preprocessed / ScaffoldEmitted /
   RoutedToSpecialist / DegradedPassthrough), baseline+1 from LIVE 221/532 — full 6-part variant
   discipline (the wire-remediation 91-error lesson), re-read consts at edit (W-9).
5. Legs 2/4 (separate sub-session, sidecar-gated): hybrid composer (UI -> AXTree/DOM alongside;
   document -> E-36 leg; OCR-heavy -> specialist) + probe-small-escalate router + image prompt
   caching on re-queried screenshots. Deterministic fixture harness ships NOW; live VLM runs are a
   separately-gated checkbox (W-10).
6. Harness /see skill (~1 SKILL.md + ~80-LOC stdlib+pillow script): runs leg-1 preprocess locally
   before attaching any image to a claude-code message. JC-4: pillow verified at first use;
   absent -> passthrough + one-line warning. JC-5: frontmatter passes agentskills_validator.py +
   eval jsonl pair.
7. Measurement fixtures: 4K-screenshot fine-text fixture + seeded click-target fixture (coordinate
   ground truth) + a dense-UI tiling fixture. Record ELAI deltas (raw vs gateway arms) in a
   metrics TOML.
8. Decision doc: workflow/reasoning/J-1-decisions.md (sweet-spot table chosen, scaffold prompt,
   router thresholds, what stayed external-figure-only).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/perception/ (PerceptionGateway, PreprocessSpec, ScaffoldResult, the
    hard-limit router)
  - python sidecar perception endpoints (legs 2/4; remote-only)
  - .claude/skills/see/ (SKILL.md + preprocess script + eval pair)
  - +PerceptionScaffoldEvent SSB (baseline+1)
consumes_symbols:
  - media_validator Tier1 pixel-diff veto + Tier2 VLM entry (B-24 — feed, never re-implement)
  - the E-36 document-vision leg (route target)
  - D-16 refusal path (typed refusal for over-limit asks)
  - H-13 canvas_screenshot (downstream consumer; no change required here)
reachable_from:
  - the image-bearing agent-dispatch seam (every dispatch whose content carries an image)
    [PRODUCTION dispatch path; exact site re-grepped at edit, W-7]
  - .claude/skills/see trigger on harness image reads  [HARNESS leg, JD-1 evidence]
stub_ban:
  - NO VLM call on the visual-diff path (Tier-1 veto is law)
  - NO silent downscale fallthrough (a preprocess failure emits DegradedPassthrough, never
    silently sends the raw image while claiming conditioning)
  - NO literature figure in any user-facing claim or metrics TOML (measured-only, HD-1)
  - NO .unwrap()/.expect() on prod paths; NO new Rust image dep without approval
frontend_pair:
  perception rows surfaced in an existing drawer tile (decide exact drawer at edit; W-4 satisfied
  or W-6-justified in the decisions doc).
observable_effect:
  trigger: "elai run over the 4K-screenshot fixture task (gateway ON) vs --raw-image control arm;
    plus: invoke /see on a local image read in a harness session"
  expect_ssb: "PerceptionScaffoldEvent rows (Preprocessed + ScaffoldEmitted) recomputable from the
    journal under ELAI_SEED (GD-2)"
  expect_artifact: "the gateway arm answers the fine-text question the control arm fails;
    coordinate error <2% on the click-target fixture; the dense-UI fixture shows tiles+thumbnail
    in the journaled request. NEGATIVE: --raw-image arm fails the fine-text question (proves the
    metric discriminates, R2); a visual-diff fixture routed to a VLM = FAIL."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-1_perception_gateway.md --run-trigger
rg "PerceptionScaffoldEvent" .elai_cc/crates/orchestrator/src/ssb.rs           # variant exists (all 6 parts)
rg -c "unwrap\(\)" .elai_cc/crates/orchestrator/src/perception/                # MUST be 0
python .claude/tools/agentskills_validator.py .claude/skills/see/SKILL.md      # exit 0
test -f workflow/reasoning/J-1-decisions.md
```

## ACCEPTANCE
- [ ] legs 1+3 ship model-free; legs 2/4 fixture harness ships now, live VLM runs separately
      gated (W-10); pod stopped after any GPU run.
- [ ] scaffold injection placed after the last cache breakpoint; HD-2 byte-stability assert holds.
- [ ] all shipped numbers ELAI-measured per config; external figures appear only as cited
      rationale (HD-1/ID-5).
- [ ] SSB 6-part discipline complete; baseline re-read at edit (W-9); cross_lang_diff PASS if the
      frontend pair lands.
- [ ] /see degrades to passthrough without pillow (JC-4); validator + eval pair pass (JC-5).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator (+contracts).
- [ ] no new dependency without explicit approval (pillow is harness-side, verify-before-use).

## Tests to add
- crates/contracts/tests/j1_perception_gateway_contract.rs (~120 LOC): (a) preprocess spec maps
  each target model to its sweet spot; (b) visual-diff input routes to pixel-diff, VLM call count
  0; (c) over-limit counting ask returns typed refusal; (d) scaffold event lands with correct Kind
  on the fixture; (e) DegradedPassthrough on a corrupt image (corrupt-input case); (f) broken arm:
  --raw-image control documented failing the fine-text fixture.

## Files touched
- CREATE: crates/orchestrator/src/perception/{mod,preprocess,scaffold,router}.rs (~350 LOC)
- MODIFY: ssb.rs + action_type_lock.rs(.golden) (+1 variant, 6 parts)
- MODIFY: the image-bearing dispatch seam (gateway insertion; exact site at edit)
- CREATE: sidecar perception endpoints (legs 2/4 sub-session)
- CREATE: .claude/skills/see/{SKILL.md, scripts/preprocess.py, eval/positive.jsonl, eval/negative.jsonl}
- CREATE: workflow/reasoning/J-1-decisions.md

## Citations / Grounding
- SRC-3 vlm-vision-research-summary.md (mechanisms + figures; EXTERNAL, scope-caveated).
- ELAI ground truth: RT-1 (LIVE 221/532), B-24/E-36/E-16/H-13 shipped status per pre-read log;
  JC-3/JC-4/JC-5.
- Hard rules invoked: HARDWARE-HEAVY sidecar, anti-slopsquat (pillow), HD-1/HD-2, W-9/W-10, GD-1/2.

## Open questions / risks
- exact dispatch-seam insertion point (one seam vs per-agent): decide at edit with a re-grep
  (W-7); prefer the single choke-point the scaffold can gate.
- scaffold cost on image-heavy sessions: bounded by leg-4 router + caching; measure before
  default-ON for call-2-always (GD-4).
- sidecar VRAM for specialist legs: remote 3090 only; never local (house rule).
- sweet-spot table drift as providers change: pin per-model values in config with a dated source
  line; re-verify on model swaps (G-10 canary is the runtime sibling).

## Next
Closes U7 (the LLM doesnt SEE properly) with the only genuinely-new perception lane in the corpus;
feeds B-24/H-13 with conditioned inputs; the /see skill gives the harness the same floor. After
J-1, E-14/E-16's AXTree doctrine and the canvas screenshot path all consume better pixels.

End of J-1.
