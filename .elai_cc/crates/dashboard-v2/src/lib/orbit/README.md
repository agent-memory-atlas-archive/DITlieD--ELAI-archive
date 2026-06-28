# orbit/ — Deterministic orbit visualizer subsystem

The orbit IS the activity surface for the ACTIVITY window (B-13 V2
rebaseline). It renders the live workflow as a 3D starmap with three
FSM-driven modes. No LLM in any hot path; all positions are seeded from
an LCG for entropy-quarantined determinism.

## Mode resolution

Explicit `mode` prop wins; otherwise derived from `fixture.fsm`:
- `RESEARCHING` -> `research`
- `IDLE` + pagerank nodes present -> `map`
- anything else -> `plan`

## File index

### Core types + pure logic
| File | Role |
|------|------|
| `types.ts` | Orbit-specific types (ZoomState, GrcStep, stubs, fixtures). Layers on `api/types.ts`. |
| `lcg.ts` | Seeded LCG + FNV-1a hash. Entropy-quarantined determinism for positions + idle VFX. |
| `geometry.ts` | Polar/cartesian helpers, Bezier path builder, angle distributors, viewport math. |
| `zoom.ts` | 4-level zoom state machine (concept -> phase -> step -> baby-step) + interpolation. |
| `captions.ts` | Pure deterministic `(FsmState, GrcStep, BubbleUpAction?) -> string` caption table. No LLM. |
| `decompose-converter.ts` | Converts live decomposition tree (plan store) into `OrbitFixture` shape. |
| `contract.ts` | Type-level + pure-function preconditions enforced by `svelte-check`. |

### 3D scene (three.js via StarCanvas)
| File | Role |
|------|------|
| `node-mandala.ts` | "Datastream Mandala" 3D layer — troika-three-text labels, arcs, anchors. ~48K. |
| `node-panel-variants.ts` | Variant resolver for node-panel content (4 quadrants + compass cards). ~30K. |
| `node-panel-variants.md` | Design notes for the V3 NodePanel variant system. |

### Svelte components — wired
| File | Role | Imported by |
|------|------|-------------|
| `OrbitCanvas.svelte` | Universal 3D canvas; dispatches geometry per mode. | `ActivityWindow` |
| `OrbitMode1Plan.svelte` | Plan mode: caption + bubble-up strip + shooting-star overlay. | `OrbitCanvas` |
| `OrbitMode2Map.svelte` | Map mode: badge + symbol count. | `OrbitCanvas` |
| `OrbitMode3Research.svelte` | Research mode: badge + node count. | `OrbitCanvas` |
| `OrbitCaption.svelte` | Plain-language caption below orbit. Calls `captionFor()`. | `OrbitMode1Plan` |
| `BubbleUpStrip.svelte` | Horizontal event chip strip; polls `get_bubble_up_events`. | `OrbitMode1Plan` |
| `OrbitShootingStarOverlay.svelte` | Idle VFX: LCG-seeded shooting star every ~14s. | `OrbitMode1Plan` |
| `OrbitDeepSpace.svelte` | Extended starfield outside main orbit (far zoom). | `OrbitCanvas` |
| `OrbitGrcDots.svelte` | GRC step dots orbiting active node. **Scaffold** — static, awaits history producer. | `OrbitCanvas` (3D scene) |
| `OrbitBudgetRing.svelte` | Context-budget donut (8 slots). **Scaffold** — awaits B-10 producer. | `OrbitCanvas` (3D scene) |
| `OrbitEvidenceGates.svelte` | 7 evidence gates on a ring. **Scaffold** — awaits B-5/B-6 producers. | `OrbitCanvas` (3D scene) |
| `OrbitConcept.svelte` | Single concept as decomposition starmap (SVG). | `OrbitCanvas` (3D scene) |
| `OrbitTendrils.svelte` | Research tendrils extending outside orbit. | `OrbitCanvas` (3D scene) |

### Svelte components — DEAD CODE (unwired)
| File | Status | Note |
|------|--------|------|
| `MainPanel.svelte` | **WIRE-DARK** — no importer. | V3 NodePanel main panel. 370 lines. Never mounted; `OrbitCanvas` renders the mandala via `node-mandala.ts` instead. |
| `MiniCard.svelte` | **WIRE-DARK** — no importer. | V3 NodePanel compass mini-card. 157 lines. Same reason as MainPanel. |

Both files carry `WIRE-DARK` markers at the top. They are retained as
reference for a future DOM-overlay NodePanel pass but are NOT part of
the production render path. Removing them is safe if the V3 DOM-panel
approach is fully abandoned in favor of the 3D mandala.

## Wiring graph (production path)

```
ActivityWindow.svelte
  └─ OrbitCanvas.svelte
       ├─ OrbitMode1Plan.svelte (plan mode)
       │    ├─ OrbitCaption.svelte -> captions.ts (captionFor)
       │    ├─ BubbleUpStrip.svelte -> api/stub.ts (get_bubble_up_events)
       │    └─ OrbitShootingStarOverlay.svelte -> lcg.ts
       ├─ OrbitMode2Map.svelte (map mode)
       ├─ OrbitMode3Research.svelte (research mode)
       ├─ OrbitDeepSpace.svelte
       ├─ node-mandala.ts (3D mandala via StarCanvas)
       ├─ node-panel-variants.ts (variant resolver)
       └─ zoom.ts + geometry.ts (camera math)
```

`contract.ts` is imported by `svelte-check` (type-level only); it
validates the public API surface at compile time but emits no runtime code.

## Known gaps (awaiting upstream producers)

1. **BubbleUpStrip** polls `get_bubble_up_events` from `stub.ts` every
   500ms. The feed is empty until B-7 wires the `bubble_up_event` SSB
   emitter. The component renders an "no bubble-up events" empty state.
2. **Signal layers** (`OrbitGrcDots`, `OrbitBudgetRing`,
   `OrbitEvidenceGates`) are scaffolds with static/placeholder data.
   They await real data from B-5 (evidence gates), B-6 (SSB substrate),
   and B-10 (context budget) producers.
3. **SessionHandoffEntry** (in `api/types.ts`) mirrors the Rust
   `SessionHandoffEntry` shape but has NO frontend consumer. The
   frontend consumes handoff data indirectly via `get_session_branches`
   (which wraps handoffs into `SessionBranch` objects). The type is
   retained as documentation of the canonical Rust shape; remove it
   only if the SettingsWindow is confirmed to never render raw handoff
   rows.

## B-13 acceptance mapping (V2 rebaseline)

The original V1 acceptance criteria (auto-scroll, Jump-to-Latest pill,
inline [preview] entries, filter strip, Ctrl+F search) DO NOT APPLY to
V2 — the orbit has no scrollable text feed. The rebaselined task list:

1. **Orbit captions** — `captions.ts` CAPTION_TABLE covers 14+ entries.
   Verified by `contract.ts` (`_captionAtLeast14`).
2. **Bubble-up event strip** — `BubbleUpStrip.svelte` wired into
   `OrbitMode1Plan.svelte`.
3. **Signal layers** — scaffolds exist; await upstream producers (see
   Known gaps above).
4. **Idle effect** — `OrbitShootingStarOverlay.svelte` fires every ~14s
   via LCG-seeded counter. Wired into `OrbitMode1Plan.svelte`.
5. **Settings schema fix** — moved to B-3; `SessionHandoffEntry` type
   in `api/types.ts` has the canonical shape (see Known gaps #3).
6. **Hover-reveal chrome** — `ActivityWindow.svelte` mode-toggle pill
   uses `opacity: 0` + `.activity-host:hover .mode-toggle { opacity: 1 }`.
   The top-left concept header was removed (duplicated OrbitCaption).
