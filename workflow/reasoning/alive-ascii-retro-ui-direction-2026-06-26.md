ALIVE ASCII / RETRO-TERMINAL UI DIRECTION (ELAI)
Date 2026-06-26. Author main LLM, grounded run. Status DIRECTION (not a plan). Spike plan sibling = alive-ascii-ui-spike-plan-2026-06-26.md.

PROBLEM
The dashboard look keeps coming out generic. Goal is a UI no one has seen: ELAI as something the user feels close to, not a piece of software. Direction = an ALIVE UI. A character plus everything rendered like an old green terminal (ELIZA 1966 vibe, cool-retro-term palette without the bezel/curvature/nozzles), pseudo-3D with depth so the character feels physically present in the same world as the text, real-time ASCII, UI that reacts (one thing affects another) but text always perfectly readable. Lean off the galaxy/stars theme toward ELIZA + amazing ASCII work.

KEY FINDING (the reason this is shippable, not an art project)
Most of the vision already maps onto ELAI architecture that is planned or wired. This is choosing an art direction over an existing subsystem, not inventing one.
- The stacked-translucent-node-graph-with-depth reference image IS the planned canvas. ELAI canvas has three altitudes drawn as stacked node planes: system-composition (J-31), process/observability (J-60), memory/case (J-98). Nodes bind to plan units / wiring contracts / MCP servers; edges carry verdicts.
- three.js already runs inside Tauri 2.10 + Svelte 5 + Vite 5 in dashboard-v2 (Activity Orbit, WIRED). The orbit is scheduled for retirement (H-16) once canvas-default ships clean. So there is a literal open slot: the alive-ASCII canvas is the art direction for canvas-default already on the roadmap. Not a new window.
- The character is the front-door router (J-32, chat = ELAI not a model). Its ACK -> progress -> RESULT states are the character's states. Mouth moves while text generates = the character animates off the start_chat_stream token stream.
- Every alive interaction the user described is already a journaled op. Character throws something that becomes a node = a CanvasOpApplied SSB row animated as a throw. Character moves nodes = the J-61 redirect verbs REORDER/REBIND animated as grabbing. The animation is the visual interpretation of a real journal row, not invented state.

THE LOAD-BEARING CONSTRAINT (protects the moat)
Canvas doctrine, from the feature map verbatim: the SSB journal is the store, the canvas re-derives byte-identically, no parallel state; a rendered picture is not proof, coverage_count is the backstop. Therefore:
- The character throwing a node must be TRIGGERED BY a CanvasOpApplied row. It cannot invent the node. Node identity, edges, verdicts come from the journal.
- Node layout/physics/animation must be a DETERMINISTIC function of journal + seed (e.g. force-directed layout seeded by row order) so the scene re-derives identically on replay. The moment the character's physics becomes the source of where a node is, the reproducible-op-stream moat is broken.
- Line to hold: journal owns identity/edges/verdicts; the alive layer owns interpretation (layout, depth, character motion), and interpretation is deterministic, not free. Get this right and "alive but never lying" becomes the differentiator instead of a tension.

READABILITY RULE (layer separation)
Content text (what ELAI writes, lists, real data) stays a crisp green-on-black monospace grid, full contrast, axis-aligned, cool-retro-term palette minus bezel/curvature. The ASCII-3D node-world plus character lives behind and around it with parallax depth. The character may reach INTO the text region (point, throw toward a word) but readable glyphs never warp. Depth via parallax layers, never by bending words. This is the Caves of Qud split (alive ASCII world + dead-flat readable panels), with depth added only to the world half.

CHARACTER ANIMATION (grounded)
Do NOT have the LLM draw ASCII. LLMs read ASCII fine but cannot construct 2D-in-1D (spatial blindness, read-write asymmetry). Refs ASCIIBench arXiv:2512.04125; "Why LLMs Suck at ASCII Art" mailitics 2025-01-21; "Learning to Draw ASCII Improves Spatial Reasoning" arXiv:2604.14641. So per-frame glyph authoring by an LLM is out, and so is a video per animation.
AcerolaFX-quality ASCII comes from the SHADER, not from drawing. Architecture: rigged character -> drive parameters at runtime -> ASCII shader produces the look. Two routes:
- Route 1 (3D -> ASCII, highest fidelity): load a pre-rigged VRM avatar via @pixiv/three-vrm. VRM ships free Expression categories: LipSync (visemes), Blink, LookAt, Emotion. Drive mouth from token-stream amplitude (even 3 mouth shapes reads as speech, ref Voice Animator arXiv:1910.08685), eyes look-at mouse. Render through the ASCII shader (Threlte AsciiRenderer, or AcerolaFX-style DoG+Sobel edge-aware, or emilwidlund GPU effect). The face reads as a shaded 3D head in ASCII. More GPU (VRAM competes with local model).
- Route 2 (procedural ASCII, on-brand + cheap): a parametric glyph character where each frame is a function of time/params (ertdfgcvb play.core style). The LLM writes the parametric driver functions (code/math, which it is good at), not glyphs. Mouth = row swap on token amplitude. Lower realism ceiling, but the character is made of the same living glyphs as the terminal it lives in, lowest GPU, maximally "alive code". Likely the most ELIZA-native option.
What you would have to build is modest: the ASCII shader wiring (lift from AcerolaFX/emilwidlund) plus a parameter-driver mapping token-stream / mouse / SSB-row -> rig params. Not a from-scratch 3D pipeline. The spike should try a cheap version of each route to feel them.

BRAND NOTE (ELIZA framing)
The marketing image says ELIZA convinced people it understood, it did not. ELAI substance is the opposite: provable, journaled, a picture is not proof. Wearing the warm 1966-terminal face while the receipts underneath are cold and provable is an honest, sharp story: the terminal that pretended to understand, rebuilt to actually prove what it did. The alive character earns the closeness; the journal keeps it honest.

CONSTRAINTS / RISKS
- Tauri + heavy WebGL is the real risk, not the ASCII. Known WebGL context-lost and Windows-build lag in Tauri (issues 6559, 8020), limited WebGPU (6381). System webview (WebView2) is weaker than native GPU. A full GPU-shader ASCII world competes for the VRAM the local model needs.
- AsciiEffect renders to a DOM char-table: fine for a contained character/ambient panel, heavy at high res for dense UI. GPU-shader variant avoids the table but is the VRAM eater.

DECISION
Build a throwaway vertical-slice spike before any plan, because the three unknowns (does the feeling land, is it readable, does it fit VRAM next to a local model) can only be answered by building one scene. Recommendation: spike the cheap render route (Threlte AsciiRenderer) + try both character routes cheaply; keep dense readable data as plain green-terminal DOM forever; graduate hero/character surfaces to a custom shader only if the feeling lands and VRAM allows. Spike spec = alive-ascii-ui-spike-plan-2026-06-26.md.

REFERENCES (verified this run)
- ertdfgcvb play.core, interactive mouse-reactive textmode engine, github.com/ertdfgcvb/play.core, play.ertdfgcvb.xyz
- textmode.js, dynamic real-time ASCII library (creativeapplications writeup)
- three.js AsciiEffect, threejs.org/examples/webgl_effects_ascii.html
- Threlte AsciiRenderer (Svelte binding), threlte.xyz/docs/reference/extras/ascii-renderer
- emilwidlund/ASCII, GPU three.js ASCII effect, github.com/emilwidlund/ASCII
- AcerolaFX, ReShade HLSL edge-aware ASCII shader (DoG+Sobel+atlas LUT), github.com/GarrettGunnell/AcerolaFX, MIT. ASCILINE, Python/JS video->ASCII streaming, github.com/YusufB5/ASCILINE, MIT+anti-ad. Full notes: workflow/research/ascii-rendering-repos.md
- @pixiv/three-vrm, VRM avatar in three.js with LipSync/Blink/LookAt/Emotion expressions, github.com/pixiv/three-vrm
- cool-retro-term, CRT terminal look, github.com/Swordfish90/cool-retro-term
- LLM ASCII limits: ASCIIBench arXiv:2512.04125; arXiv:2604.14641; mailitics "Why LLMs Suck at ASCII Art"
- procedural lipsync: Voice Animator / real-time live2D lipsync arXiv:1910.08685
- Tauri WebGL risk: github.com/tauri-apps/tauri issues 6559, 8020, 6381
