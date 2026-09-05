ALIVE ASCII / RETRO-TERMINAL UI SPIKE PLAN (ELAI)
Date 2026-06-26. For a separate implementation session. Status SPIKE (throwaway, measure-the-feeling, NOT production). Direction doc = alive-ascii-retro-ui-direction-2026-06-26.md (read it first; it carries the architecture mapping, the journal-truth constraint, and the grounded references).

GOAL
Answer three unknowns by building ONE scene, not by reasoning: (1) does the alive ASCII / green-terminal feeling land, (2) does it stay readable, (3) does it fit fps + VRAM next to a running local model in the REAL Tauri webview. Output is a feel verdict for maintainer, not shippable code.

SCOPE / ISOLATION (decision, flag if you disagree)
This is a throwaway. Do NOT edit production dashboard-v2 in place. Create an isolated spike app at F:\.ELAI_workflow\workflow\spikes\ascii-ui\ (gitignored under workflow/). Copy the MINIMAL three.js + Svelte 5 + Vite + Tauri setup from .elai_cc/crates/dashboard-v2 (the Activity Orbit scene is the closest working reference for three.js-in-Tauri). The point of copying real ELAI parts is to reproduce the real webview/GPU conditions, not to wire into the live engine. No SSB schema changes, no new action types, no production callers. The wiring-enforcement gates do not apply to a throwaway spike, BUT the journal-as-truth doctrine still shapes the design (see CONSTRAINT below) so the spike does not teach a wrong pattern.
If isolation turns out to block the realism of the test (e.g. you genuinely need a live CanvasOpApplied stream), STOP and hand back the question rather than mutating production.

RUNTIME TARGET
Tauri 2.10 + Svelte 5 runes + Vite 5 + three.js, matching dashboard-v2. Run the BUILT Tauri window (WebView2 on Windows), not just the browser dev server. The browser will lie about performance; the known Tauri WebGL fragility (tauri issues 6559/8020/6381) only shows in the built webview.

DEPENDENCIES (verify each on the registry before adding, anti-slopsquat)
- @threlte/extras for the AsciiRenderer (Svelte binding to three.js AsciiEffect). Cheap render route.
- @pixiv/three-vrm IF doing character Route 1 (VRM rig). Skip if doing Route 2 only.
- Use bun for the spike (bun install / bun run), not npm, per .claude/rules/bun-not-node.md.

BUILD STEPS (vertical slice)
1. Green-on-black retro terminal frame. cool-retro-term palette (phosphor green on near-black), monospace, scanline + subtle bloom via CSS/shader, but NO bezel/curvature/CRT-housing. A readable content panel (plain DOM monospace grid) on one side proving text stays crisp.
2. One canvas altitude as an ASCII-converted 3D object. Render a small node graph (10-20 nodes, a few edges, stacked on 2-3 depth planes like the reference image) through the AsciiRenderer. Parallax/orbit so depth reads. Layout = deterministic function of a fixed seed + node order (NOT physics-as-truth; see CONSTRAINT).
3. A character that moves its mouth on a streaming-token signal. Fake the token stream with a timer emitting characters; drive a mouth-open parameter from the emission cadence/amplitude. Eyes look-at the mouse. Do BOTH character routes cheaply enough to compare:
   - Route 1: a pre-rigged VRM via three-vrm, drive LipSync + LookAt + Blink expressions, render through the same ASCII shader.
   - Route 2: a procedural ASCII face (parametric glyph functions, ertdfgcvb play.core style), mouth = row swap on amplitude. No 3D model.
4. One alive interaction wired to an event. On a simulated CanvasOpApplied event (fake it on a keypress/timer), the character throws a glyph that arcs and lands as a new node in the graph. The node's existence is triggered BY the event, never invented by the throw animation.

CONSTRAINT THE SPIKE MUST RESPECT (so it does not teach a wrong pattern)
Even faked, model the data flow correctly: an event (stand-in for an SSB row) creates the node; the animation only INTERPRETS it. Node identity/edges come from the event log; node position/motion is a deterministic function of that log + a fixed seed. Do not let the character's physics decide whether a node exists or where it canonically lives. This mirrors the real doctrine (journal is the store, canvas re-derives byte-identically, a picture is not proof) so a follow-on production session can lift the pattern, not just the pixels.

MEASURE (the actual deliverable)
- fps of the ASCII scene at idle and during the throw interaction, in the BUILT Tauri window.
- GPU VRAM used by the webview, measured WITH a local model loaded/running concurrently (the binding constraint). Note delta vs model-only.
- Readability check: can you read the content panel comfortably while the world animates behind it? yes/no + a screenshot.
- Feel: a 20-30s screen capture for maintainer to react to. Both character routes side by side if possible.

SUCCESS CRITERIA
Not "is it done." The spike succeeds if it produces: a clear fps/VRAM number in the real Tauri webview, a readable/not-readable verdict, and a feel capture good enough for a go/no-go on committing to the direction. A captured failure (e.g. WebGL context-lost in the built window, or VRAM blows the local-model budget) is a SUCCESSFUL spike outcome, recorded, not a thing to fight.

DO NOT
- Do not edit .elai_cc production dashboard-v2 or any engine crate.
- Do not add SSB action types, change SCHEMA_VERSION, or touch the journal.
- Do not have an LLM hand-draw ASCII frames (it is architecturally bad at producing ASCII; drive a rig/shader or parametric functions instead).
- Do not make a video per animation.
- Do not raise any pixel-diff threshold to hide font/AA churn; pin the capture instead (deviceScaleFactor 1, font-render-hinting none) per visual-ui-loop.md if you screenshot for comparison.

HANDBACK
Write findings (the numbers, the verdict, the capture path) to workflow/reasoning/alive-ascii-ui-spike-findings-2026-06-26.md and ping maintainer. If blocked on a material fork, append a handoff entry per .claude/rules/handoff-protocol.md and return BLOCKED rather than guessing.
