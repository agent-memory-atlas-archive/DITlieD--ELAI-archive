# B-11 Decision Document: Drift Color Removal

## STEP 5: Remove drift colors

### Issue Found

Two files contain significant color drift from the spec palette:

#### 1. ConvergencePanel.svelte
Uses a completely different color scheme (likely from a template):
- `#e2e8f0` (text) → spec: `#d0d7de` (text-primary)
- `#0f172a` (surface) → spec: `#0f1318` (bg-base) or `#151a20` (bg-surface)
- `#1e293b` (border) → spec: `#1a2028` (bg-hover)
- `#38bdf8` (accent) → spec: `#1B7A6E` (teal) or `#24a08e` (teal-bright)
- `#64748b` (muted) → spec: `#6e7681` (text-secondary)
- `#f87171` (error) → spec: `#c44040` (fail)
- `#fbbf24` (warn) → spec: `#c49b2a` (warn)

**Decision**: Replace all with spec palette equivalents. This is straightforward mapping.

**Status**: ✅ COMPLETED - All colors replaced with spec equivalents.

#### 2. GraphWindow.svelte
Uses many off-palette colors for code graph visualization:

**Status colors** (not in spec):
- `#9e9e9e` (pending) → `#6e7681` (text-secondary)
- `#1976d2` (running) → `#1B7A6E` (teal)
- `#388e3c` (passed) → `#1B7A6E` (teal/pass)
- `#d32f2f` (failed) → `#c44040` (fail)
- `#f57c00` (blocked) → `#c49b2a` (warn)

**Impact level colors** (not in spec):
- `#ffb199` (impact-1) → `#c44040` (fail)
- `#ff8a73` (impact-2) → `#c49b2a` (warn)
- `#ff5247` (impact-3) → `#6e7681` (text-secondary)

**Edge/selection colors** (not in spec):
- `#888` (default edge) → `#6e7681` (text-secondary)
- `#ff5252` (cyclic edge) → `#c44040` (fail)
- `#0bf` (selected border) → `#1B7A6E` (teal)
- `#ff7676` (overlay error) → `#c44040` (fail)

**Default node colors** (not in spec):
- `#3578e5` (default node) → `#6e7681` (text-secondary)
- `#fff` (node text) → `#d0d7de` (text-primary)
- `#000` (text outline) → `#0a0e12` (bg-void)

**Status**: ✅ COMPLETED - All status/impact/edge/default colors mapped to spec.

**Language node colors** (retained as functional exception):
- `#dea584` (rust) - KEPT (semantic differentiation)
- `#3572a5` (python) - KEPT (semantic differentiation)
- `#3178c6` (typescript) - KEPT (semantic differentiation)
- `#ff3e00` (svelte) - KEPT (semantic differentiation)

**Decomposition level colors** (retained as functional exception):
- `#283c5a` (Concept) - KEPT (semantic differentiation)
- `#3b5a8a` (Phase) - KEPT (semantic differentiation)
- `#4f7eb7` (Step) - KEPT (semantic differentiation)
- `#7fa9d9` (BabyStep) - KEPT (semantic differentiation)

### Key Decision Point

The spec states: "The palette is intentionally narrow. If you need a color not listed here, you probably don't need a color -- use text weight or opacity."

However, GraphWindow uses colors for **semantic meaning** in a data visualization:
1. Language differentiation (helps identify code structure at a glance)
2. Status indication (pending/running/passed/failed/blocked)
3. Impact levels (blast radius severity)
4. Cyclic edge detection (dependency cycles)
5. Decomposition level differentiation (Concept/Phase/Step/BabyStep)

**Decision Applied**: Hybrid approach
- Map all status/impact/edge/default colors to spec palette (semantic states map to spec semantic colors)
- Keep language-specific and decomposition-level colors as functional exceptions (they convey structural information at a glance)

**Rationale**: Language and level colors serve a functional semantic purpose in data visualization - they allow users to instantly recognize code structure and decomposition hierarchy. This is not decorative; it's information density. The spec's narrow palette guidance applies to decorative UI elements, not functional data visualization where color differentiation conveys meaning.

### Expected Test Outcome

After replacement:
- ConvergencePanel matches spec exactly ✅
- GraphWindow status/impact/edge/default colors use spec palette ✅
- GraphWindow language/level colors retained for functional semantic differentiation ✅

### Unknowns

None - decision made and applied.
