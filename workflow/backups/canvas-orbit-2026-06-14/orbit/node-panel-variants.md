# NodePanel V3 — variant content shape

The V3 hybrid NodePanel is composed of three layers:

1. **3D Mandala** (`NodeMandala`) — rings + 4-or-5 compass markers + tendrils
   in world space, integrated with the existing UnrealBloomPass.
2. **Compass mini-cards** (`MiniCard.svelte`) — small DOM cards anchored to
   each marker's projected screen position. Always upright (camera-facing)
   with a small CSS perspective tilt derived from horizontal screen position.
3. **Main workstation panel** (`MainPanel.svelte`) — 480×300px DOM panel
   anchored to a "south of node" projected point. 2×2 quadrant grid with
   IDENTITY / LIVE ACTIVITY / DIFF (or per-variant override) / PROGRESS.

`node-panel-variants.ts` exports `resolveVariant(ctx)` which returns a
`VariantContent` describing every layer for the resolved variant:

```
VariantContent {
  variantKey:    'concept' | 'phase' | 'step' | 'baby' | 'integration-point' | 'research-node' | 'fallback',
  showNwMarker:  boolean,                // 5th compass slot for integration variant
  tierColored:   boolean,                // research-mode: tier-colored markers
  compass:       CompassContent,         // up to 5 mini-cards
  main:          MainPanelContent,       // breadcrumb + 4 quadrants
}
```

SYNTHETIC content is comment-marked in `node-panel-variants.ts` so a
follow-up plumbing pass can replace stub strings (live LLM activity,
diff snippets, files-modified counters) with live SSB telemetry.

## Compass mini-cards

Default slot semantics:
- **N** (north, +z marker)  → `IDENTITY`
- **E** (east, +x marker)   → `LOCATION`
- **S** (south, -z marker)  → `STATUS`
- **W** (west, -x marker)   → `ACTIVITY`
- **NW** (only when `showNwMarker=true`) → `DEPENDENCIES` (integration variant)

Each slot is a `{ tag, lines: [{text, classMod}] }` pair. `classMod`
controls font face / colour: `title` (sans, brighter), `mono` (mono dim),
`mono-bright` (mono teal), `sans-dim`.

## Main workstation panel

Quadrants ordered TL, TR, BL, BR — `kind` controls extra rendering:
- `identity`     — IDENTITY block (title + sub-lines)
- `liveActivity` — LIVE ACTIVITY (3-4 mock LLM action lines)
- `diff`         — DIFF (3-5 line synthetic diff snippet)
- `progress`     — radial progress arc (SVG, stroke-dasharray with
                   `pathLength=100`) + 3-row meta grid
- `phasesList`   — concept variant only: replaces DIFF with phase names
- `dependencies` — integration variant only: replaces DIFF with linked-file list
- `sourcesTier`  — research variant only: replaces DIFF with tier-coloured dots

## Variants

### concept (sun root)
Compass: ONLY IDENTITY (N) + STATUS (S). Main: PHASES list in BL.
```
            [IDENTITY]
                ⊙
[STATUS] ⊙ project ⊙ [LOCATION (none)]
                ⊙
            [ACTIVITY (none)]

  +--------+--------+
  |IDENTITY|ACTIVITY|
  +--------+--------+
  |PHASES  |PROGRESS|
  +--------+--------+
```

### phase
Full mandala (rings + 4 markers). Main: IDENTITY/LIVE/DIFF/PROGRESS,
LIVE ACTIVITY shows the active step and DIFF carries a stub for the
recent files modified.

### step
Same shape as phase. LIVE ACTIVITY = live LLM action line; DIFF = stub
for the function being written.

### baby-step
Same shape as step. Title is the symbol-decl line; LIVE ACTIVITY is the
specific function-currently-being-written; DIFF is the synthetic diff for
that struct/fn.

### integration-point (map mode)
5th NW marker visible. Main: BL replaced with DEPENDENCIES (linked-files
list with overflow). Markers stay teal.

### research-node (research mode)
`tierColored=true` recolours the markers per source-tier (T1=bright,
T2=mid, T3=dim). Main: BL replaced with SOURCES (tier-coloured dots
+ source line list). PROGRESS shows convergence percentage.

## Sketches (ASCII)

### concept
```
[IDENTITY]                          [STATUS]
  ELAI                                ⊙ EXECUTING
  project · concept root              phase 02 · step 04

  +-----+--------------------------+-----+
  | [IDENTITY] ELAI                |[LIVE] ▶ executing  |
  | concept root · 9 phases · 47s  | phase 02 · step 04 |
  +-----+--------------------------+-----+
  | [PHASES] 01 setup · 02 auth ...|[PROGRESS]   ⊙ 22%  |
  |   03 ...                       | phases 9 / steps 47|
  +-----+--------------------------+-----+
```

### phase
```
[IDENTITY] Auth refactor       [LOCATION] ELAI / 9 phases
[STATUS]   ⊙ EXECUTING ▓▓▓▓░░  [ACTIVITY] ▶ writing handler

  +-----+--------------------------+-----+
  | [IDENTITY]                     | [LIVE ACTIVITY]    |
  |  Authentication refactor       |  ▶ active step ... |
  +-----+--------------------------+-----+
  | [DIFF] +pub fn pool_builder()  | [PROGRESS] ⊙ 57%   |
  |        +cap = cap;             |  tokens / elapsed  |
  +-----+--------------------------+-----+
```

### baby-step
```
[IDENTITY] PoolBuilder         [LOCATION] crates/auth/pool.rs:18
[STATUS]   ⊙ EXECUTING 4s      [ACTIVITY] ▶ writing fn pool_builder

  +-----+--------------------------+-----+
  | [IDENTITY] PoolBuilder         | [LIVE ACTIVITY]    |
  |  baby 02/04 · auth refactor    |  ▶ writing fn ...  |
  +-----+--------------------------+-----+
  | [DIFF] +pub struct PoolBuilder | [PROGRESS] ⊙ 55%   |
  +-----+--------------------------+-----+
```

### integration-point (map)
```
[IDENTITY] AuthMiddleware  [LOCATION] router.rs · auth tag
[STATUS]   pagerank 12.4   [ACTIVITY] ↪ 5 callers
                                              [DEPENDENCIES] (NW)
                                                + 4 linked files

  +-----+--------------------------+-----+
  | [IDENTITY] AuthMiddleware      | [LIVE ACTIVITY] (idle) |
  +-----+--------------------------+-----+
  | [DEPENDENCIES] router.rs       | [PROGRESS] ⊙ 62%       |
  |   handler.rs guard.rs ...      |  callers / impls       |
  +-----+--------------------------+-----+
```

### research-node (research)
```
[IDENTITY] "JWT refresh ..." [LOCATION] 3 sources / 2 converged
[STATUS]   ⊙ REFLECT T1      [ACTIVITY] ▶ token rotation OK

  +-----+--------------------------+-----+
  | [IDENTITY] "JWT refresh ..."   | [LIVE ACTIVITY]        |
  +-----+--------------------------+-----+
  | [SOURCES] T1 ● T2 ◐ T3 ○       | [CONVERGENCE] ⊙ 67%    |
  +-----+--------------------------+-----+
```

## Future variants (not yet implemented)
- **Failed-node** — title + error string + outlined-teal "retry" button.
- **Bubble-up** — when an evidence gate fails and bubble-up routes to a
  parent, show source baby + bubble action verb + target tier.
- **Anti-spiral** — same file touched >3 times: warning row + spiral counter.
- **Provider-edge** — for map nodes whose edges include MCP / external
  call edges, show provider name + last latency.
