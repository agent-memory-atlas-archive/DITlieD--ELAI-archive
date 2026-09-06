ELAI brand icon / glyph — gpt_image prompts (2026-06-15)
purpose: distill a small app-icon / favicon glyph mark for ELAI (the open item in DESIGN.md §5 + §14 — "we have the wordmark but no small icon, like Cursor/Claude have"). Run manually in ChatGPT "Create image" (gpt-bridge is banned). Grounded in DESIGN.md §1 (one-teal-signal identity), §3 (palette: monochrome + one teal #2BBBA4 on near-black), §5 (monoline E·L·A·I stroke geometry; must read at 16px, one color), §6 (glyphs carry meaning, never status hues), §10 (bans: no blue/second-accent/gradient/glow/glassmorphism). Follows .claude/rulebook/gpt-image-prompting.md (instruction-following spec, no n param -> use a contact sheet for variations, EXACT text rules, ratio stated in words). Builds on the earlier L2 favicon prompt in gpt-idea-prompts-2026-06-12.md (this one is 10 candidates + the white-monoline + one-teal-point treatment, which is more on-DESIGN than the all-teal L2).

LOCKED FACTS (do not let a render talk you out of them): glyph distilled from the monoline stroke letters E L A I (uniform thin strokes, rounded ends, geometric skeleton); white strokes + exactly ONE small teal point as "the signal" (per §1/§5 — the wordmark is white, teal is the lone signal); near-black tile; flat (no glow/gradient/glassmorphism — glow is brand-HERO-only, never on a flat mark); must stay legible at 16px in one color.

================================================================================
PROMPT A — the 10-candidate contact sheet (primary)
================================================================================
Deliverable: a wide 16:9 landscape brand-icon exploration sheet — ten small square app-icon candidates in a 2-row by 5-column grid on one flat near-black canvas. A logo-exploration contact sheet, not a single hero icon.

Scene: one flat, even, near-black background (#0a0f0e) — no texture, no vignette, no stars, no glow. Ten identical rounded-square tiles in a clean 2x5 grid with generous even spacing; each tile a slightly deeper near-black (#070b0a) with a single 1px hairline border.

Subjects: ten DISTINCT minimal brand glyphs, one per tile, each distilled from a monoline "E L A I" letter system (uniform thin strokes, rounded stroke ends, geometric skeleton letters). Each glyph is drawn in white (#f2f5f4) with exactly ONE small accent in a single muted teal (#2BBBA4) — the "one signal" — and nothing else colored. The ten glyphs:
01 the letter E: three thin horizontal strokes on a vertical, middle bar slightly shorter, a tiny teal dot at the end of the middle bar.
02 the letter A: two slanted strokes meeting at a point with a thin crossbar, the apex a small teal point like a star.
03 the letter I: a single vertical stroke with a teal dot floating above it (a signal point).
04 an E+I monogram: an E sharing its vertical stroke with an I, merged into one compact mark, one teal dot.
05 a four-point constellation: four small white dots (E, L, A, I) joined by hairline white lines into a small angular figure, the single brightest node teal.
06 one bright teal point centered, encircled by a single thin white orbit ring (the lone signal).
07 the letter A as two slanted strokes with no crossbar and a teal dot at the apex, reading like a minimal peak/caret.
08 an "EL" corner mark: the E's verticals and the L forming an inset right-angled bracket like a recessed well, a teal dot in the corner.
09 an upward caret built from the A's peak fused with the I's stroke, suggesting forward motion, teal point at the tip.
10 an ultra-condensed ELAI monogram: the four letters compressed into a tiny 2x2 square stack of monoline strokes, one teal dot.

Composition: top-down flat view, every tile straight-on and identical in size, grid centered with even margins, each glyph centered in its tile with comfortable padding so it reads at very small sizes.

Lighting and mood: flat, even, no shadows, no glow; quiet, minimal, instrument-like.

Style: minimal flat vector icon-design sheet, monoline, geometric, premium and restrained — a designer's app-icon exploration board.

Materials and detail: clean uniform thin strokes, rounded line ends, crisp edges, perfectly flat fills; no gradients, no bevels, no 3D.

Text in image: EXACT, verbatim, no extra characters: labels "01" "02" "03" "04" "05" "06" "07" "08" "09" "10" each appear once, small, in pale gray monospace, beneath their tile. No other text; no letters anywhere except the strokes that form the glyphs.

Constraints: no watermark, no extra text, single muted teal accent only, no blue, no purple, no orange, no second color, no gradients, no glow, no glassmorphism or blur, no drop shadows, flat near-black background only, thin geometric strokes only, every glyph legible at 16px, do not turn the letters into a normal typeface font.

FALLBACK (if 10 tiles + labels render muddy): run the same prompt twice — "five candidates 01-05 in a single row, wide 16:9" then "06-10". Fewer objects per render = crisper strokes + labels (house rule: simplify the scene, there is no quality knob).

VARIANT (monochrome check): re-run Prompt A with "render every glyph in pure white with NO teal accent at all (one color only)" to see the marks as a true one-color favicon, then compare against the white+one-teal version.

================================================================================
PROMPT B — finalize the chosen glyph as a single app icon (after picking from the sheet)
================================================================================
Deliverable: a single square 1:1 app icon, large and centered.
Subjects: [describe the chosen glyph from the sheet, e.g. "the letter A as two thin slanted monoline strokes with a teal dot at the apex"], white (#f2f5f4) thin monoline strokes with exactly one muted teal (#2BBBA4) accent point, on a rounded-square near-black (#0a0f0e) tile with a 1px hairline border.
Composition: centered, generous padding, straight-on.
Lighting and mood: flat, even, no shadow, no glow.
Style: flat vector, monoline, minimal, premium.
Text in image: none.
Constraints: no watermark, no text, single teal accent only, no gradients, no glow, no shadow, must read clearly at 16px.

NEXT AFTER A RENDER LANDS: pick a direction, finalize via Prompt B, then redraw the chosen mark as a hand-authored SVG (the render is a decision artifact, never the shipped asset — same rule as the cook renders) for the favicon + app icon + the .elai_cc/assets identity set; update DESIGN.md §5/§14 (favicon glyph: resolved).
