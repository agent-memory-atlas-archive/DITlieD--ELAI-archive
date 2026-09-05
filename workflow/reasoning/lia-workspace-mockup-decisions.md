# LIA workspace mockup — decisions

## What I will implement
A single self-contained HTML concept showing LIA as a deterministic, evidence-led workspace. Its focal surface is a pannable node graph, with persistent navigation on the left and the front-door chat on the right. Product labels come from `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`; the mockup changes branding only, from ELAI to LIA.

## Design read
Reading this as: a control instrument for a technical operator, late-1990s network-anime editorial language, leaning system over decoration.

- STRUCTURE_VARIANCE: 7 — asymmetrical graph-first composition and vertical editorial labels.
- MOTION_INTENSITY: 3 — only state feedback, pan/zoom, and brief signal motion.
- VISUAL_DENSITY: 8 — a working cockpit with one dominant graph rather than a card grid.

## Approaches considered
1. Chrome-heavy command center: maximizes visible telemetry but demotes the canvas and risks an equal-panel dashboard.
2. Graph-first instrument: makes the evidence chain spatial, keeps chat as the command ingress, and matches the specified left-canvas/right-chat hierarchy.

Decision: use the graph-first instrument.

## Key technical decision
Use semantic HTML, inline SVG, CSS, and dependency-free JavaScript; embed the authored font as data so the final file has no runtime dependencies.

## Palette
Use only exact source swatches: `#000000`, `#004C4C`, `#006666`, `#007E98`, `#008080`, `#02C2CC`, `#66B2B2`, and `#B2D8D8`. Failures use `#B2D8D8` on `#000000` or the inverse—never red or yellow.

## Top failure modes
1. Cyberpunk decoration overwhelms product meaning. Guard: every prominent node maps to a source-described subsystem or evidence artifact.
2. The UI becomes a grid of generic cards. Guard: one dominant graph, open rails, structural lines, and only one inspector surface.
3. State is encoded by color alone. Guard: shape, labels, line style, and position remain legible in monochrome.

## Expected verification outcome
The file opens directly, all navigation/node/chat/state controls work without network access, keyboard focus is visible, reduced-motion is honored, and every CSS color literal belongs to the approved palette.

## Unknowns to verify empirically
- Fit at 1440×900, 1024×768, and a narrow mobile viewport.
- Pointer pan and node selection do not conflict.
- The embedded font renders from a `file://` URL.
