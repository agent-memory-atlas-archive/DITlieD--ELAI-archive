#!/usr/bin/env python3
"""J-1 perception-gateway measurement fixtures generator (deterministic).

Generates the three measurement fixtures the J-1 perception-gateway contract
tests + T5 live VLM runs consume, plus the consolidated ground-truth JSON the
contract tests assert against:

  1. fine_text_4k.png        -- 3840x2160 PNG with 8pt text unreadable without
                                 preprocessing. The gateway arm answers the
                                 fine-text question; the --raw-image control
                                 arm fails it (R2 negative control).
  2. click_target.png        -- 1920x1080 PNG with a single seeded click target
                                 at a known pixel coordinate. Coordinate error
                                 must be < 2% (normalized 0..=1000 scale).
  3. dense_ui_tiling.png     -- 2560x1440 PNG with dense UI elements (many
                                 buttons/labels). The gateway arm must emit
                                 tiles + a thumbnail in the journaled request.

All generation is DETERMINISTIC (seed=0, fixed patterns, no external images,
no literature figures -- HD-1/ID-5). Pillow is required (same renderer as the
/see skill); if absent the script prints a warning and exits 1 -- these
fixtures cannot be generated without a real rasterizer.

CLI:  python gen_fixtures.py --output-dir <dir>

Outputs (written into <dir>):
  fine_text_4k.png, click_target.png, dense_ui_tiling.png, ground_truth.json
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from typing import Any

try:
    from PIL import Image, ImageDraw, ImageFont  # anti-slopsquat: pillow 12.x verified installed
except ImportError:
    print(
        "WARNING: pillow is not installed; the J-1 perception fixtures cannot be "
        "generated without a real rasterizer. Install pillow (pip install pillow) "
        "and re-run. This is NOT the /see passthrough path.",
        file=sys.stderr,
    )
    sys.exit(1)

# --- deterministic constants (ELAI_SEED=0, GD-2) ---------------------------
SEED = 0
GROUND_TRUTH_SUITE = "j1_perception_gateway"

# Fine-text fixture: the question the gateway arm answers and the raw arm fails.
FINE_TEXT_DIMS = (3840, 2160)
FINE_TEXT_FONT_PT = 8
FINE_TEXT_QUESTION = "What is the serial number shown in the fine print?"
FINE_TEXT_EXPECTED_ANSWER = "ELAI-J1-7Q4Z-2026"

# Click-target fixture: a single target at a known pixel coordinate.
CLICK_TARGET_DIMS = (1920, 1080)
CLICK_TARGET_PIXEL = (1789, 873)  # x, y in pixels
CLICK_TARGET_ERROR_BOUND_PCT = 2.0  # normalized 0..=1000 scale

# Dense-UI tiling fixture: dense grid of buttons/labels.
DENSE_UI_DIMS = (2560, 1440)
# Expected tile grid for the Claude sweet spot (SWEET_SPOT_CLAUDE=1568,
# DEFAULT_TILE_OVERLAP=0.25). Tiling the full 2560x1440 image into
# 1568x1568 tiles with 25% overlap:
#   horizontal: step = 1568*0.75 = 1176; positions 0, 1176 -> 2 cols
#   vertical:   1440 < 1568 -> 1 row
#   total = 2 tiles + 1 thumbnail (DEFAULT_THUMBNAIL_LONGEST_SIDE_PX=512).
DENSE_UI_TILE_SIZE = 1568
DENSE_UI_TILE_OVERLAP = 0.25
DENSE_UI_EXPECTED_TILE_GRID = (2, 1)
DENSE_UI_EXPECTED_TILE_COUNT = 2
DENSE_UI_THUMBNAIL_LONGEST_SIDE_PX = 512

# Normalized coordinate scale used by perception::scaffold::LiteralElement
# (bbox coords are 0..=1000, scaffold.rs:143).
NORMALIZED_SCALE = 1000


def _normalized(pixel_xy: tuple[int, int], dims: tuple[int, int]) -> tuple[int, int]:
    """Map a pixel coordinate to the 0..=1000 normalized scale (scaffold.rs)."""
    nx = round(pixel_xy[0] / dims[0] * NORMALIZED_SCALE)
    ny = round(pixel_xy[1] / dims[1] * NORMALIZED_SCALE)
    return (nx, ny)


def _deterministic_color(stream: str) -> tuple[int, int, int]:
    """Deterministic RGB color from a string (no RNG drift across runs)."""
    h = hashlib.sha256(stream.encode("utf-8")).digest()
    return (h[0], h[1], h[2])


def _draw_fine_text(out_dir: str) -> dict[str, Any]:
    """3840x2160 PNG with 8pt fine text unreadable without preprocessing."""
    img = Image.new("RGB", FINE_TEXT_DIMS, color=(245, 245, 248))
    draw = ImageDraw.Draw(img)
    font = ImageFont.load_default(size=FINE_TEXT_FONT_PT)
    # A block of fine print lines; the serial is the answer the gateway arm
    # must extract and the raw arm must miss.
    lines = [
        "ELAI perception gateway -- fixture fine print (seed=0).",
        "This block is rendered at 8pt on a 3840x2160 canvas; when the image",
        "is downscaled to a model sweet spot (1568/2048/1280 longest side)",
        "the glyphs collapse and the serial below becomes illegible.",
        "Serial: ELAI-J1-7Q4Z-2026",
        "Batch: 2026-06-30-T7   Rev: 1   Channel: gateway-arm-discriminator",
        "Do not read this serial from a downscaled copy; preprocess first.",
    ]
    margin_x, margin_y = 64, 64
    line_height = 14
    text_bboxes: list[dict[str, Any]] = []
    for i, line in enumerate(lines):
        y = margin_y + i * line_height
        draw.text((margin_x, y), line, fill=(20, 20, 30), font=font)
        bbox = draw.textbbox((margin_x, y), line, font=font)
        text_bboxes.append({
            "text": line,
            "bbox_px": [bbox[0], bbox[1], bbox[2], bbox[3]],
            "bbox_normalized": [
                round(bbox[0] / FINE_TEXT_DIMS[0] * NORMALIZED_SCALE),
                round(bbox[1] / FINE_TEXT_DIMS[1] * NORMALIZED_SCALE),
                round(bbox[2] / FINE_TEXT_DIMS[0] * NORMALIZED_SCALE),
                round(bbox[3] / FINE_TEXT_DIMS[1] * NORMALIZED_SCALE),
            ],
        })
    path = os.path.join(out_dir, "fine_text_4k.png")
    img.save(path, "PNG")
    return {
        "image": "fine_text_4k.png",
        "dimensions": list(FINE_TEXT_DIMS),
        "font_pt": FINE_TEXT_FONT_PT,
        "text_lines": text_bboxes,
        "question": FINE_TEXT_QUESTION,
        "expected_answer": FINE_TEXT_EXPECTED_ANSWER,
        "note": (
            "8pt fine text on a 4K canvas. The gateway arm (preprocess + scaffold) "
            "answers the serial question; the --raw-image control arm fails it "
            "(R2 negative control, plan line 116-117)."
        ),
    }


def _draw_click_target(out_dir: str) -> dict[str, Any]:
    """1920x1080 PNG with a single seeded click target at a known coordinate."""
    img = Image.new("RGB", CLICK_TARGET_DIMS, color=(28, 30, 38))
    draw = ImageDraw.Draw(img)
    cx, cy = CLICK_TARGET_PIXEL
    # Draw a distinct target: filled circle + crosshair + label, so a VLM /
    # scaffold can localize it. Deterministic colors from the coordinate.
    ring_color = _deterministic_color(f"ring-{cx}-{cy}")
    cross_color = (240, 240, 245)
    label_font = ImageFont.load_default(size=14)
    # Outer ring (radius 28 px) + inner dot (radius 6 px).
    draw.ellipse([cx - 28, cy - 28, cx + 28, cy + 28], outline=ring_color, width=4)
    draw.ellipse([cx - 6, cy - 6, cx + 6, cy + 6], fill=cross_color)
    # Crosshair extending past the ring.
    draw.line([(cx - 44, cy), (cx + 44, cy)], fill=cross_color, width=2)
    draw.line([(cx, cy - 44), (cx, cy + 44)], fill=cross_color, width=2)
    # Label offset so it does not overlap the target.
    draw.text((cx + 52, cy - 10), "TARGET-01", fill=(235, 235, 240), font=label_font)
    path = os.path.join(out_dir, "click_target.png")
    img.save(path, "PNG")
    norm = _normalized(CLICK_TARGET_PIXEL, CLICK_TARGET_DIMS)
    return {
        "image": "click_target.png",
        "dimensions": list(CLICK_TARGET_DIMS),
        "target": {
            "pixel": list(CLICK_TARGET_PIXEL),
            "normalized": list(norm),
        },
        "error_bound_pct": CLICK_TARGET_ERROR_BOUND_PCT,
        "note": (
            "Single seeded click target at a known pixel + normalized (0..=1000) "
            "coordinate. The gateway arm must localize it within <2% error on the "
            "normalized scale (plan observable_effect line 113)."
        ),
    }


def _draw_dense_ui_tiling(out_dir: str) -> dict[str, Any]:
    """2560x1440 PNG with a dense grid of UI buttons/labels."""
    img = Image.new("RGB", DENSE_UI_DIMS, color=(232, 234, 240))
    draw = ImageDraw.Draw(img)
    font = ImageFont.load_default(size=12)
    # Dense grid of buttons: 16 columns x 9 rows = 144 elements. Each cell
    # gets a filled rectangle + a label, deterministic color from (col,row).
    cols, rows = 16, 9
    cell_w = DENSE_UI_DIMS[0] // cols
    cell_h = DENSE_UI_DIMS[1] // rows
    count = 0
    for r in range(rows):
        for c in range(cols):
            x0 = c * cell_w + 6
            y0 = r * cell_h + 6
            x1 = (c + 1) * cell_w - 6
            y1 = (r + 1) * cell_h - 6
            color = _deterministic_color(f"btn-{c}-{r}")
            draw.rectangle([x0, y0, x1, y1], outline=(60, 62, 70), fill=color, width=2)
            label = f"btn_{c:02d}_{r:02d}"
            draw.text((x0 + 8, y0 + 6), label, fill=(15, 15, 22), font=font)
            count += 1
    path = os.path.join(out_dir, "dense_ui_tiling.png")
    img.save(path, "PNG")
    return {
        "image": "dense_ui_tiling.png",
        "dimensions": list(DENSE_UI_DIMS),
        "ui_element_count": count,
        "tile_size_px": DENSE_UI_TILE_SIZE,
        "tile_overlap": DENSE_UI_TILE_OVERLAP,
        "expected_tile_grid": list(DENSE_UI_EXPECTED_TILE_GRID),
        "expected_tile_count": DENSE_UI_EXPECTED_TILE_COUNT,
        "thumbnail_longest_side_px": DENSE_UI_THUMBNAIL_LONGEST_SIDE_PX,
        "note": (
            "Dense UI grid (16x9 = 144 buttons/labels) on a 2560x1440 canvas. "
            "The gateway arm must emit tiles (expected 2 for Claude sweet spot "
            "1568 with 0.25 overlap) + a 512px thumbnail in the journaled "
            "request (plan observable_effect line 114)."
        ),
    }


def _build_ground_truth(
    fine_text: dict[str, Any],
    click_target: dict[str, Any],
    dense_ui: dict[str, Any],
) -> dict[str, Any]:
    return {
        "suite": GROUND_TRUTH_SUITE,
        "seed": SEED,
        "normalized_coord_scale": NORMALIZED_SCALE,
        "note": (
            "Deterministic reference the J-1 perception-gateway contract tests "
            "assert against. All data is ELAI-generated (seed=0); no literature "
            "figures (HD-1/ID-5). Live-VLM-gated assertions are pending T5."
        ),
        "fixtures": {
            "fine_text_4k": fine_text,
            "click_target": click_target,
            "dense_ui_tiling": dense_ui,
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate J-1 perception-gateway measurement fixtures."
    )
    parser.add_argument(
        "--output-dir",
        default=os.path.dirname(os.path.abspath(__file__)),
        help="Directory to write the fixture PNGs + ground_truth.json.",
    )
    args = parser.parse_args()
    out_dir = os.path.abspath(args.output_dir)
    os.makedirs(out_dir, exist_ok=True)

    fine_text = _draw_fine_text(out_dir)
    click_target = _draw_click_target(out_dir)
    dense_ui = _draw_dense_ui_tiling(out_dir)
    ground_truth = _build_ground_truth(fine_text, click_target, dense_ui)

    gt_path = os.path.join(out_dir, "ground_truth.json")
    with open(gt_path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(json.dumps(ground_truth, indent=2) + "\n")

    print(
        "wrote fine_text_4k.png, click_target.png, dense_ui_tiling.png, "
        f"ground_truth.json to {out_dir} (seed={SEED})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
