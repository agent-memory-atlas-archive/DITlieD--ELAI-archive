---
name: see
description: Harness leg-1 image preprocess for the J-1 perception gateway. Triggers when the user is about to attach an image to a message, asks to read or analyze a screenshot / diagram / mockup / photo / figure, says "look at this image", "read this screenshot", "analyze this mockup", "check this figure for the fine print", or invokes /see by name. Runs scripts/preprocess.py locally BEFORE the image is attached so the bytes are conditioned (resized to the target model sweet spot, JPEG q85, dense-UI overlapping tiles, global thumbnail) instead of sending raw pixels. Pillow absent degrades to passthrough plus a one-line stderr warning, never a silent fallthrough. The Rust PerceptionGateway is the runtime leg; /see is the harness leg and they share semantics, not code.
license: internal
compatibility: "Designed for Claude Code. ELAI workflow root. Portable to any harness that attaches images to a vision-capable model message."
metadata:
  version: "1.0.0"
  author: "elai"
user-invocable: true
---

# /see — harness leg-1 image preprocess

`/see` is the HARNESS-side leg of the J-1 perception gateway (docs/thefinalglance/J-plan/J-1_perception_gateway.md). It conditions an image LOCALLY, with stdlib + pillow, BEFORE the image is attached to a claude-code message — so the model receives bytes already sized to its sweet spot instead of raw pixels. The Rust `PreprocessSpec` (crates/orchestrator/src/perception/preprocess.rs) is the RUNTIME leg on the sidecar; the two share SEMANTICS, not code. No FFI, no shared module, no import of `preprocess.rs` from `preprocess.py`.

## WHEN THIS SKILL FIRES

Fire `/see` (set LEG-1 PREPROCESS mode for the rest of the attachment) when the user:
- invokes `/see this image`, `/see` with a path, or `/see` before attaching a screenshot / diagram / mockup / photo / figure.
- says "look at this screenshot and tell me what's wrong", "read this architecture diagram", "analyze this UI mockup", "I'm attaching a photo — what's in it", "check this figure for the fine print".
- is about to attach any image to a message for the model to reason over (drag-drop, paste, or path reference into a harness session).

Does NOT fire on:
- pure-text questions ("what does the SSB single-writer rule mean?") — no image.
- single-file code reads ("read cli/src/args.rs and fix line 88") — no image; that is a normal edit.
- design-judgement asks on a mockup like "improve this UI design" — spec-eval owns the design verdict; /see only conditions the bytes that spec-eval then reasons over.
- tasks already inside another skill's protocol (planning-workflow's own reads, paper-to-plan's source ingestion, incremental-eval's multi-artifact walk) — those skills own the read discipline; /see only fires when an image is the artifact.

## LEG-1 PREPROCESS PROTOCOL

1. Identify the target vision model the message is bound for (default Claude; override per dispatch).
2. Run `scripts/preprocess.py --input IMAGE --target-model claude|gpt|qwen-vl --output OUT.jpg` BEFORE attaching the image.
3. Attach the conditioned output (the resized JPEG q85) to the claude-code message. The thumbnail (STEM.thumb.jpg) and dense-UI tiles (STEM.tiles directory) are emitted alongside for dense-UI reasoning; attach them when fine text or small click targets matter.
4. Cite the conditioning in the answer ("image preprocessed to EDGEpx longest side, JPEG q85, N tiles + 512px thumbnail") so the reasoning trace shows the bytes were conditioned, not raw.

### Sweet-spot table (EXTERNAL figure SRC-3, design rationale ONLY — HD-1/ID-5)

These longest-side edges are EXTERNAL design rationale from `J-HARNESS-GROUNDING-MAP.md` SRC-3 (vlm-vision-research-summary), NOT ELAI-measured numbers. They MUST NOT appear in any user-facing claim or metrics TOML as a measured value. They are pinned as config with a dated source line; re-verify on model swaps.

| target model | sweet-spot longest side | JPEG quality | tile overlap | thumbnail |
|---|---|---|---|---|
| Claude-class  | 1568px | 85 | 0.25 | 512px |
| GPT-class     | 2048px | 85 | 0.25 | 512px |
| Qwen-VL-class | 1280px | 85 | 0.25 | 512px |

Resize is downscale-only (never upscale), aspect ratio preserved. PNG to RGB JPEG q85 is the conditioning step (~60% image-token savings, negligible quality loss — EXTERNAL figure SRC-3, design rationale only). Dense-UI tiling uses 0.25 overlap on each axis plus a 512px global thumbnail so the model has both the fine-grained tiles and the layout overview.

## DEGRADED PASSTHROUGH (JC-4, stub_ban)

If pillow is absent (`ImportError`) OR any preprocess step fails (decode error, resize error, write error), the script forwards the raw input bytes UNCONDITIONED to the output path and emits ONE explicit stderr warning ("pillow not installed; /see degraded to passthrough — install pillow for leg-1 conditioning", or the specific failure). It NEVER silently sends raw bytes while claiming conditioning (plan stub_ban line 102-103; mirrors `PreprocessOutcome::DegradedPassthrough` in preprocess.rs:131-148). The script always exits 0 on the passthrough path so the harness can still attach the raw image; the warning is the audit trail. Pillow is verified before use (JC-4: pillow 12.1.1 confirmed on this machine; the absent path is the acceptance gate, not this machine's state).

## BOUNDARY — what /see is NOT

- /see is the HARNESS leg (local pillow). The Rust `PerceptionGateway` is the RUNTIME leg (sidecar, T2/T5). Same semantics, different language.
- /see does NOT implement: the two-call perception scaffold (leg 3, T2), the hard-limit router (T3), the SSB event (T4), or legs 2/4 sidecar VLM (T5).
- /see FEEDS B-24's output validator and E-36's document leg — it never re-implements either (GD-1, plan line 12).
- /see does NOT add pillow as a Cargo dep, a sidecar dep, or a runtime Rust dep. Pillow is a Python harness-side dep only.

## CITATIONS (cite, do NOT fork — GD-1 / stub_ban)

- J-1 plan: docs/thefinalglance/J-plan/J-1_perception_gateway.md (T6 scope lines 74-77).
- Rust PreprocessSpec: crates/orchestrator/src/perception/preprocess.rs (same sweet-spot consts, same JPEG q85, same tile_overlap, same thumbnail — different language).
- SRC-3 vlm-vision-research-summary (EXTERNAL figures, design rationale only — HD-1/ID-5).
- CLAUDE.md anti-slopsquat (JC-4 pillow verify-before-use).

END /see SKILL
