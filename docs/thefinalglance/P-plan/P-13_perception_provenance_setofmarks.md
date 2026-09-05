# P-13: perception provenance + Set-of-Marks id-indirection

STATUS: PLANNED (WAVE CU-1; EXTENSION of E-14 + J-1/J-8/J-77; PROOF_SPINE: none; execute after the proof spine per A-7)

- Track COMPUTER-USE-STEAL | lever id-indirection-over-coords | grounding STRONG (OmniParser arXiv:2408.00203 measured 70.5->93.8% grounding lift, verified in the eval doc; J-1 anchors PLAN-asserted, executor re-greps W-7) | nature EXTENSION | effort M | WAVE CU-1 | PROOF_SPINE none
- Premise: J-1 conditions bytes + supplies AXTree/DOM text, but the model still has to POINT. Set-of-Marks id-indirection lets the VLM return an integer id off a tagged element list (never coordinates); a per-element source tag is the trust boundary. This is the mechanism behind OmniParser's 93.8% and it works with zero pixel model.

## Context — why this matters
Grounded in workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 6 (OmniParser, arXiv:2408.00203 verified). Measured: captions+IDs lift grounding 70.5->93.8%; ScreenSpot 73.0 vs raw GPT-4V 16.2. Consumed via Set-of-Marks: the VLM returns a Box ID integer, the harness owns geometry. When a DOM/AXTree exists it dominates a pixel model (OmniParser's own paper barely beats GPT-4V+HTML) — which CONFIRMS J-1's AXTree-first doctrine rather than challenging it. The one case J-1's AXTree-first path cannot answer is canvas-rendered UIs (games, WebGL, Figma-class), native apps with dead UIA, streamed screens — exactly the surfaces P-12's "test a game" ask needs.

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- J-1 perception gateway + /see (J-SUMMARY_INDEX.md:105): PerceptionScaffold, counts-from-AXTree/DOM-never-VLM law. EXTENDED here, not duplicated.
- J-8 vlm perception research upgrade (J-SUMMARY_INDEX.md:260): resize table / schema-v2 / EVAL. The upgrade schema this rides.
- J-77 perception faithfulness via trust spine (J-SUMMARY_INDEX.md:1749): a mis-read region is a ghost citation. A mis-sourced element tag is the same failure — this composes.
- E-14 AXTree doctrine (E-SUMMARY_INDEX.md:134): confirmed by OmniParser's own dominance-when-DOM-exists result.
- G-1 taint: the source tag (axtree_exact | ocr_text | vlm_caption | pixel_yolo) IS provenance and slots into the taint/grounding lattice.

## Steal/build (REVISED scope, W-10 tiered)
- SHIP NOW (Tier-1): (S1) id-indirection on the AXTree node list — the scaffold returns a numbered element list, the VLM picks an integer, the harness resolves geometry; (S4) a per-element source tag as a first-class field; (S2) an interactability flag filtering the VLM candidate set.
- DEFER (Tier-2): (S3) advisory-only captions for unnamed canvas/game icons; (S5) asymmetric 80%-containment OCR-into-element label fusion; the Tier-3 pixel/YOLO adapter (fires only on empty/incomplete AXTree — lands with P-12's game surfaces).
- EXCLUDED as vaporware (S8 anti-pattern): a per-crop caption standing in for element identity; counts derived from the VLM (J-1 law — counts stay AXTree-only).

## Reads (mandatory before coding)
- workflow/reasoning/canvas-computer-use-eval-2026-07-11.md (ENTRY 6, steals S1-S8)
- docs/thefinalglance/J-plan/J-1_*.md + J-8_*.md (the scaffold structs to extend), J-77 (faithfulness composition)
- crates/orchestrator/src/perception/ (J-1's scaffold.rs — the extend target)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/perception/scaffold.rs (ElementSource enum {AxtreeExact,OcrText,VlmCaption,PixelYolo} + box_id field on the element)
  - crates/orchestrator/src/perception/set_of_marks.rs (numbered-element-list builder; VLM-picks-integer resolver)
consumes_symbols:
  - crates/orchestrator/src/perception/mod.rs::PerceptionScaffold (J-1; EXTEND, prefer-reuse — net 0..+1 rows)
  - crates/orchestrator/src/taint.rs::TaintSource (G-1; source tag = provenance)
reachable_from:
  - crates/orchestrator/src/perception/mod.rs (the gateway service consumed by E-14 + P-12) [PRODUCTION perception gateway]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO count answered from the VLM leg (J-1 law: counts from AXTree/DOM only)
  - NO element without a source tag; NO vlm_caption/pixel_yolo tag treated as authoritative (axtree_exact is the only authoritative source)
  - NO coordinate returned by the model (the model returns an integer id; the harness owns geometry)
  - NO .unwrap()/.expect() on the perception path; NO blake2/sha2
frontend_pair:
  rides J-1's existing perception tile — a source-tagged element breakdown (no new surface; reuse the tile, W-4).
observable_effect:
  trigger: "elai perception scaffold --fixture tests/fixtures/perception/canvas_empty_axtree.png --json"
  expect_ssb: "perception_scaffold_emitted"
  expect_artifact: "the element list carries integer ids + a source tag per element + the trace shows ZERO VLM count calls; NEGATIVE: a count answered from the VLM leg, or an element missing its source tag, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: running the scaffold on a canvas-rendered UI fixture with an empty AXTree yields an element list where every element has an integer box_id + a source tag; a follow-up "click element 7" resolves by id (harness geometry), never by model-supplied coordinates; the trace shows zero VLM count calls.
ON_FAIL: an element arrives without a source tag, or a count is answered from the VLM (CAUSE: the source tag is optional, or the count path routes through the caption model). COUNTER-MOVE: make ElementSource a required non-optional field on the element struct; assert the count path reads only AXTree/DOM nodes; the negative test (VLM-count / missing-tag) must FAIL.
FORKS: none (the Tier-3 pixel fallback is scoped to P-12; here the AXTree-present path is dominant and settled by OmniParser's own result).
RECON_NEEDED: confirm the exact J-1 PerceptionScaffold element struct + whether an ElementSource-shaped field already exists (avoid a second field). Settling check: `grep -n "struct .*Element" crates/orchestrator/src/perception/scaffold.rs` at edit.
ABORT: if adding the source tag would require a SECOND perception scaffold or a second provenance scale (A-1/N-2 violation), STOP and flag — extend the ONE scaffold + ride N-6's one lattice, never fork.
VERIFY: the load-bearing proof is the off-agent perception_scaffold_emitted row + a trace assertion of zero VLM count calls, driven by the real `elai perception scaffold` entrypoint. Run:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-13_perception_provenance_setofmarks.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-13_perception_provenance_setofmarks.md   # execution done-gate
```
REDTEAM: surviving-attack seed — a caller that trusts a vlm_caption tag as authoritative is defeated by the tag hierarchy (only axtree_exact authorizes an action-critical read; caption/pixel are advisory). Attack that would succeed if unpatched: the model returns coordinates directly bypassing id-indirection — PATCH: the resolver accepts ONLY an integer id and maps it to harness-owned geometry; a coordinate payload is rejected. Deep red-team DEFERRED to execution pre-flight.

## INTEGRATION_SMOKE
Driven jointly with P-12's smoke: the operator lane requests a perception scaffold on a live canvas/game surface (empty AXTree), the VLM picks an element by integer id, the lane actuates by harness geometry, and the trace confirms zero VLM count calls + every element source-tagged. Standalone: the scaffold fixture above. (INTEGRATION_SMOKE cadence shared with P-12 per the CU track's 2-unit R10 rule.)

## Grounding / citations
- Source: workflow/reasoning/canvas-computer-use-eval-2026-07-11.md ENTRY 6 (steals S1-S8; arXiv:2408.00203 verified there, measured 70.5->93.8%).
- ELAI ground truth (PLAN-asserted, executor re-greps W-7): J-1/J-8/J-77, E-14, G-1 per the summary indexes cited.
- CLAUDE.md hard rules: counts-from-AXTree-never-VLM (J-1 law); no second lattice (A-1/N-2); blake3; no .unwrap on prod.

## PRODUCTION_HOOK
crates/orchestrator/src/perception/mod.rs — the PerceptionScaffold gateway service (consumed by E-14 and P-12) is extended in place with the ElementSource field + the set-of-marks builder; no new entrypoint, the existing gateway callers reach the new fields. Wired in this changeset.

## Next
Closes the "which element, and how trustworthy is that read" gap for both browser (E-14) and desktop (P-12) surfaces. Feeds P-12's actuation (id-indirected clicks) and hardens J-77 faithfulness (a source tag is the anti-ghost-citation primitive).

End of P-13.
