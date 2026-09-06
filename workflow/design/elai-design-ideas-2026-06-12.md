ELAI design ideas, 2026-06-12 (cook round 2, pre-brief synthesis)
status: PROPOSAL awaiting user reaction to palette renders. reasoning behind every claim: workflow/reasoning/design-rethink-r2-2026-06-12.md. settled constraints honored: teal identity + galaxy-for-brand + flat canvas work surface + 4 question-tabs (handoff ledger). this file becomes workflow/design/elai-design-language.md (the brief) after CONVERGE; that brief seeds the new DESIGN.md.

IDENTITY STATEMENT (the one line)
ELAI is a dark instrument with one teal signal: near-black tinted toward teal, monospace doing the technical texture, the galaxy reserved for brand moments (boot, website hero, chrome edges), and color spent ONLY where a decision or a state lives. Factory.ai supplies the discipline layer, not the skin.

THE THREE PALETTE DIRECTIONS (rendered as brand sheets; user picks/mixes)
P1 ABYSS INSTRUMENT - continuity, disciplined. Base #0a0e12 / surface #151a20 (current cool ramp, kept), teal #1B7A6E with #24a08e bright stop (current, kept), text raised toward #e6ecf1, steel killed as accent (info-state only), semantic amber #C39A3A / red #B04545 muted and rare. The current identity executed with factory restraint.
P2 CARBON SIGNAL - factory-energy translation. Base near-pure black #020202 / surface #101010, neutrals tinted 2-4% toward teal (never dead gray), teal pushed hotter #2BBBA4 / #34C2AA, strong white #f2f5f4 text, accent allowed to be HOT (CTAs, active nodes burn). Highest contrast, most "serious infra product".
P3 DEEP FIELD - galaxy-forward. Base #07090b, brand surfaces carry a faint starfield + low-opacity teal nebula bloom, teal as LUMINANCE (glow halos on active/verified elements, star-point nodes with orbit-line edges), work surfaces stay flat. The most atmospheric, most "ELAI", riskiest for legibility - glow is brand-only, never on data.
All three: ONE accent (teal), no purple, no bright green, pass = teal family, OKLCH emission at build time, role-named tokens that can flip for the website's light sections (factory pattern).

STATE LANGUAGE (replaces warn/fail as palette members - user reaction 2026-06-12: foreign hues break the one-color design)
The palette is FOUR values (void/surface/text/teal). State is carried by SHAPE, color only whispers - WCAG 1.4.1 requires a non-color carrier anyway (verified w3.org/WAI/WCAG21/Understanding/use-of-color.html), and silence-means-safe keeps alarms rare so a quiet alarm stays loud. Quinary glyph set: done check (teal-filled) / active dot (teal outline, pulsing) / pending hollow circle (dim) / warn TRIANGLE in TEXT color (no own hue - amber DIES) / fail X = MONO INVERSION, LOCKED (user pick 2026-06-12): light chip, dark text, reverse-video alarm, ZERO foreign hue anywhere. Rust-whisper option rejected. The palette is fully monochrome + one teal, no exceptions.

TYPE PLAN (font round, after palette lock)
Body/technical stays JetBrains Mono BUT actually bundled via @font-face (today it silently falls back - the no-authored-font tell). Display face = the open question: candidates Geist Sans (factory's, free, proven with a mono companion), Space Grotesk (geometric, pairs with mono), or staying ALL-mono (JetBrains Mono Light at display sizes, the ultra-thin wordmark already proves thin mono display works). Hierarchy: display ultra-light wide-tracked / body mono 14px / caption ALLCAPS mono 10-11px wide-tracked (the factory dot-label voice). Cap: 2 families, 3 weights.

LOGO PLAN - LOCKED (corrected 2026-06-12, third reaction; the constellation-bg lock was a misread)
The logo is the README.md / .elai_cc/assets/logo.svg wordmark: ELAI as a MONOLINE STROKE construction. Canonical geometry (v1 site, viewBox -3 -3 236 66, 10 segments): E = vertical [0,0-0,60] + bars [0,0-34,0][0,30-25,30][0,60-34,60]; L = [76,0-76,60]+[76,60-106,60]; A = [150,60-170,0]+[170,0-190,60]+crossbar[162.7,38-177.3,38]; I = [228,0-228,60]. Stroke 2.4 round-cap teal.
The animation is ON THE LOGO, not behind it: each stroke draws in via stroke-dashoffset, 1400ms cubic-bezier(0.3,0,0.2,1), delay 400+i*380ms, line by line left to right (slow per-stroke per-letter draw). NOT the constellation background (user rejected that port - "the old one was better"). Logo presentation = the project-selector boot-screen feel: near-empty void, faint teal radial glow, few dim stars, logo as the single focal element.
Live in palette-boards-2026-06-12.html (rev 3: animated hero logo per board, click to replay) and canvas-progress-example tab bar. Glyph distillation for favicon still open (prompt L2 in gpt-idea-prompts-2026-06-12.md).

WEBSITE CONCEPT (vibe steal, layout NOT cloned)
Dark-first with the locked palette; the galaxy hero (current v1 hero is directionally right, keep wordmark-over-starfield) but the body sections go factory-disciplined: asymmetric label/content rhythm (left: dot + ALLCAPS mono label + headline + short mono body; right: a REAL product artifact), zero decorative illustration - the artifacts ARE the imagery: the canvas node graph, a receipts panel, a gate-evidence table, the mission progress bar. Light section(s) allowed for contrast (factory industries pattern) using the flipped role tokens. Voice: confident, terse, builder-facing.

DASHBOARD CONCEPT (defaults + reduction on the EXISTING AppShell - not a rebuild)
Layout target (corrected 2026-06-12): the 4 question-tabs ARE the dashboard's primary nav (PROGRESS / PROJECT / APP / KNOWLEDGE in a top tab bar) + CHAT pane alongside. The left menu + file explorer is NOT a persistent rail: it is a zero-width flyout that slides in left-to-right over the workspace on demand (hamburger / hotkey) and gets out of the way while working. Settings opens from inside that flyout as a popup modal. + 22px statusbar. Right rail stays dead - delete it. (Demo: canvas-progress-example-2026-06-12.html menubtn + flyout.)
Settings = modal overlay (z: modal), left section nav, Advanced group swallows the 15+ power inspectors. No longer a window/tab.
First-class surfaces: chat + canvas only. Canvas 4 tabs (PROGRESS home / PROJECT / APP / KNOWLEDGE) absorb plan/graph/activity/monitor as views. Monitor sub-panels -> statusbar popovers + PROGRESS side rail. Audit timeline -> node-history drawer (the "new diff"). Orbit starmap -> boot/brand + PROGRESS ambient visual, not a destination.
Project Intelligence: content stays (Tier1/2/3 + override), chrome reskins (dot+ALLCAPS sections, quiet rows), opens from explorer-rail header + palette. Its save stub needs backend wiring (build scope, not design).
Status chrome everywhere adopts the ternary language: done check / active dot / pending hollow circle, segmented progress bars with explicit fraction (n/m style).

CANVAS CONCEPT (PROGRESS tab = the readiness shape + n8n/comfy graph, feedback amplified)
Left thin column: mission one-liner, overall fraction as a segmented RUNNING bar (factory mission-control style), gate state, last activity. Main: the decomposition tree as a node graph (dagre/ELK owns geometry, local-first 30-40 nodes). Node state legible at a squint, encoded 3x redundant: fill (done = teal-filled), glyph (check/dot/hollow), edge (blocked = red edge + badge). Active node: pulsing outline + worker label + wake-trail on the active edge (reduced-motion: static highlight). Done clusters auto-collapse into a counted capsule - done reads by absence (silence-means-safe extended to geometry). Per-node receipt chips (gate evidence, confidence) make the trust wedge visible. Raw diff stays one click deep in the node-history surface (settled centerpiece).

RENDER QUEUE
this turn: 3 palette brand sheets (P1/P2/P3), each embedding wordmark + chips + UI sliver + mini website/dashboard/canvas thumbnails. 16:9, words-only specs.
after palette lock: font specimen renders (2-3 display candidates) -> style/chrome render -> full-size surface mocks (website hero, dashboard shell, canvas PROGRESS, logo glyph set) anchored with REAL screenshots via the gpt_image images param -> grill CONVERGE -> brief EMIT.

OPEN QUESTIONS FOR THE USER (react to renders, no need to answer in words)
q1 neutral temperature: current cool blue-black (P1) vs near-pure black (P2) vs deepest space (P3)?
q2 teal voice: muted sea-teal (P1) vs hot electric (P2) vs luminous glow (P3)?
q3 galaxy weight on brand surfaces: hint (P1) vs none-but-contrast (P2) vs visible field (P3)?
q4 display face instinct for the font round: a sans companion or all-mono? (T1/T2/T3 prompts in gpt-idea-prompts-2026-06-12.md preview the three voices)
q5 RESOLVED: fail = mono inversion, locked.
