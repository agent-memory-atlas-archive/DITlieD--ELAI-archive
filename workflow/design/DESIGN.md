ELAI DESIGN.md — the design language
status: v1.5, 2026-06-20 (v1.5 appends the shared feature/interaction layer §17-21 — external-connections guarantor, no-LLM git/review surface, notification model, launcher/hub, VS-Code-grade QOL affordances; functional features shared with TEIKOKU SIGNAL, expressed here in the ELAI teal/glyph language and the drawer-not-panel IA; grounded in a Codex-UI research pass + a VS-Code-QOL research pass, cross-checked against shipped ELAI features. §1-16 unchanged). v1.4, 2026-06-14, cook round 2 CONVERGE output (v1.1 folded in the entry-screen + light-theme decisions, sections 12-13; v1.2 pins the §3 palette to the ledger mockup's real ramp, marks the light VALUES provisional, and reopens the tagline; v1.4 adds §16, the shared app-chrome topbar law — one text wordmark + one top-right control cluster + centered nav, byte-identical and identically positioned across every screen so nothing pops on navigation). Decided by maintainer across three reaction rounds (boards rev1-3, canvas example, manual GPT type/website renders). This file is the GENERAL language: palette, type, logo, state, motion, IA principles. It deliberately specifies NO per-tab content or behavior. Supersedes the aesthetic sections of dashboard-v2/DESIGN.md and every visual prescription in B..K plan files; the canvas decision ledger in workflow/handoff/canvas-design-rethink.md governs canvas mechanics. Reference implementations: workflow/design/mockup/intro.html (LOCKED brand entrance + project-selection boot), workflow/design/mockup/select-terminal-styles.html (project-selection list — LEDGER style chosen), workflow/design/canvas-progress-example-2026-06-12.html, workflow/design/palette-boards-2026-06-12.html (P2 board, blue since removed).

1 IDENTITY

One line: ELAI is a dark instrument with one teal signal — near-pure black, hairline-and-mono typography, color spent only where a decision or a state lives, and radical negative space so the few elements that exist are the focus.

Tagline (UNRESOLVED — needs a rethink; none of the candidates chosen): a slot sits under the wordmark on brand screens and fades in across the exact span of the logo draw. Both "a deterministic development engine, not an assistant" (the README line) and the backronym "Evidence Logic for Autonomous Intelligence" are candidates; neither is locked. Constraints for the rethink: short, hits the trust/evidence wedge, avoids the generic-AI register. See §14.

Theme (locked): dark is the default and the brand's resting state; a full light inversion exists and is toggled on EVERY screen via the sun/moon control (sections 12-13; the light color VALUES are provisional — maintainer retunes them later — but the toggle SCOPE is locked). The instrument flips wholesale because every token is role-named (section 3); the design language is theme-agnostic.

The brand/work split (hard rule):
- BRAND surfaces (boot/project-selection screen, website logo/hero areas): TEAL ON BLACK — near-black --void with the teal radial bloom, sparse 1px dim stars, soft glow on the white logo. Quiet cosmos, felt not seen. This atmosphere exists ONLY here.
- WORK surfaces (canvas graph, panels, chat, every data view): the ONE near-black app background (--bg #0a0f0e, the ledger-mockup ground truth) everywhere work happens. No stars, no glow, no texture, no per-area background shifts. White text and the mono-inversion fail-chip carry the contrast; darker insets (--sink/--void) and the lighter --raised hover give elevation, but the dark steps are small so a hairline border, not the lightness delta, makes an edge read. The work surface is a still instrument.

2 DIALS (design-taste.md, chosen not defaulted)

Work surfaces:  STRUCTURE_VARIANCE 3 · MOTION_INTENSITY 2 · VISUAL_DENSITY 6
Brand surfaces: STRUCTURE_VARIANCE 6 · MOTION_INTENSITY 3 · VISUAL_DENSITY 2
The brand dials buy the near-empty minimal composition the user chose (one focal element, huge margins); the work dials buy legibility-first density.

3 PALETTE (locked; monochrome + ONE teal; blue is BANNED from the ramp)

Role tokens — PINNED to the ledger project-selection mockup's real ramp (workflow/design/mockup/select-terminal-styles.html :root), the ground-truth artifact maintainer reacted to and chose; emit oklch() from these at build, do not hand-convert here:
  --void        #030404   BRAND cosmos black — entrance/hero ONLY (starfield + bloom sit on it); not used inside the app window
  --bg          #0a0f0e   THE app background — every window, chrome bar, and work surface (tabbar, statusbar, flyout, project-selection body, panels, the canvas field); near-black, faint teal-green hint, NEVER blue-black
  --panel       #0e1413   raised panel / card / sub-window lifted one step off --bg (the ledger panelbox)
  --raised      #151c1a   hover / lift (the one lighter direction)
  --sink        #070b0a   deeper inset, sits below --bg
  --row         #0c1211   list / table row background
  --text        #f2f5f4   primary text, strong white
  --read        #c9cecb   secondary reading text — brighter than --dim, for body that must stay legible
  --dim         #79827e   labels / captions (teal-tinted gray, never dead #808080)
  --ghost       #3a423f   disabled / skeleton strokes
  --line        rgba(255,255,255,.17)   hairline border (paired with --line2 at .17 — shipped tune 2026-06-14)
  --line2       rgba(255,255,255,.17)   stronger hairline / divider
  --teal        #2BBBA4   THE accent — interactive, active, focus, links, PASS
  --teal-bright #34C2AA   hover / hot stop (CTAs may burn)
  --teal-deep   #1B7A6E   filled shapes (done-node fill, segments)
  --teal-soft   rgba(43,187,164,.40)   outlines / rings
  --teal-ghost  rgba(43,187,164,.10)   tinted hover washes
  --teal-faint  rgba(43,187,164,.05)   faintest wash / zebra
Rules:
- ONE accent hue. No steel-blue, no purple, no amber, no green, no second accent ever. Every neutral is tinted toward teal (2-4%), never pure gray, never pure #000/#FFF surfaces (text may be near-white).
- Elevation on work surfaces runs in BOTH directions off the single --bg: darker insets (--panel/--void fills for nodes and chrome) and a lighter hover step (--raised). The darker step is intentionally small — on the dark side the hairline border, not the lightness delta, makes the edge read; the white text, the --raised hover, and the inversion-fail carry the visible contrast. No drop-shadow stacks, no glassmorphism.
- Cards and nodes at rest are INSET wells (--panel, sitting below --bg), not floating tiles; they rise to --raised only as transient hover feedback. A card is a recess in the instrument, not a card-on-card stack.
- The ramp above is PINNED to the ledger mockup so DESIGN.md and the built UI never drift; the earlier eyeballed values (#0d1517 work-surface, #090d0c chrome) are superseded by the mockup's --bg #0a0f0e — one app background for chrome and canvas alike. Each dark token is a distinct elevation role (void / sink / bg / row / panel / raised), no near-duplicate values fighting for one job.
- Tokens are ROLE-NAMED so a future light/website section can flip the ramp without touching components (factory pattern).
- Contrast gates at build: >=4.5:1 body, >=3:1 large text / UI strokes, measured over the real background.

4 TYPOGRAPHY (locked voice: hairline display + mono everything else)

  --font-display  an authored HAIRLINE sans (weight 100-200), wide-tracked (.3-.5em), used ONLY for the wordmark voice and rare display moments. This is the stroke-logo's voice as a type system.
  --font-mono     JetBrains Mono — body, labels, data, code, everything else.
Hierarchy (visible jumps, max 2 families / 3 weights):
  display   hairline, large, wide-tracked, near-white or teal (rare, one per screen at most)
  section   9-10px mono ALL-CAPS, .2em tracking, --dim, prefixed by a 5px teal dot (the factory dot-label)
  body      12-14px mono, --text, line-height 1.5-1.6
  caption   9-10px mono, --dim
Build requirement: fonts are BUNDLED via @font-face (variable files). The current "JetBrains Mono with silent system fallback" state is a defect; an unauthored system stack on shipped UI is a BLOCK. Mockups may use fallback stacks with a note.

5 LOGO (locked: the monoline stroke wordmark + the per-stroke draw)

Geometry (canonical, from .elai_cc/assets identity; viewBox -3 -3 236 66; stroke 2.4, round caps, monoline):
  E [0,0-0,60][0,0-34,0][0,30-25,30][0,60-34,60]   L [76,0-76,60][76,60-106,60]
  A [150,60-170,0][170,0-190,60][162.7,38-177.3,38]   I [228,0-228,60]
Animation (ON the logo): each stroke draws in via stroke-dashoffset, 1400ms cubic-bezier(0.3,0,0.2,1), ten strokes left to right — first at ~900ms then +340ms per stroke, the last finishing ~5.3s — staged AFTER the brand composition (starfield, bloom, rings) has faded up. Full entrance staging in section 13. Reduced-motion: drawn instantly.
Presentation: boot-screen feel — near-empty void, a faint teal radial bloom (always on), an animated 3-layer parallax starfield, three hairline orbital rings, the logo as the single focal element; the rings/stars/bloom sit behind it but nothing else moves during the draw (full composition in section 13). Stroke color: WHITE (--text #f2f5f4) everywhere — brand and work chrome alike (locked from the T3 reference render: hairline white letters on near-black). Teal never colors the wordmark; teal stays on dots, accents, and states. Favicon/app-icon glyph distillation still open (candidate: a letter fragment or constellation-point mark; must read at 16px, one color). Chrome wordmark (small, in app topbars): the animated stroke-draw wordmark above is the Intro HERO logo ONLY; in the project-selection + workflow-shell topbars the wordmark is the TEXT lockup "ELAI" (JetBrains Mono, semibold ~13px, .18em tracking, --text on dark / ink on light), a single shared component so the two bars never drift — chosen for legibility at chrome scale. The stroke logo stays the hero/brand mark. See §16.

6 STATE LANGUAGE (locked: shape carries state, color only whispers; palette has NO status colors)

Quinary glyphs, encoded redundantly (glyph + fill/border + position) so they survive a squint and a monochrome display (WCAG 1.4.1):
  done     ✓  teal-deep filled, content RECEDES (opacity ~.8, left teal bar)
  active   ●  teal outline + slow pulse, the only moving thing on the surface
  pending  ○  dashed hairline border, dim text
  warn     ▲  shape only, drawn in --text. No own color. Amber is dead.
  fail     ✕  MONO INVERSION: light chip/card (--text background, --void text). The loudest thing on an all-dark surface. Zero foreign hue.
Doctrine: silence means safe — healthy elements carry ZERO badges; finished clusters collapse into counted ✓ capsules (done reads by absence of detail); alarms are rare, hence loud. Progress is always an explicit fraction on a segmented bar (n/m, factory mission-control style), never a lone percentage.

7 MOTION (motivated or absent)

Durations 120/180/320ms, ease-out cubic-bezier(0.16,1,0.3,1); the logo draw (1400ms staggered) is the single sanctioned hero moment. Allowed motion: state feedback (active pulse, marching dash on the live edge, press states), entry of overlays (flyout/modal slide+fade), the boot logo draw. Banned: ambient/decorative motion on any work surface, bounce/elastic, tweened layout properties (transform/opacity only), scroll-linked animation. prefers-reduced-motion: every animation swaps to its static end state; feedback stays as non-animated highlight.

8 LAYOUT AND IA PRINCIPLES (general; per-tab content is decided elsewhere)

- Primary nav = the four question-tabs: PROGRESS · PROJECT · APP · KNOWLEDGE. Chat lives alongside the canvas as a first-class pane.
- The menu + file explorer is a ZERO-WIDTH flyout: hidden while working, slides left-to-right over the workspace (scrim behind, 220ms), never a persistent rail. Settings opens from it as a POPUP MODAL (semantic z-scale), never a window/tab.
- One thin statusbar (22px). No right rail. No window zoo: surfaces beyond chat/canvas are summoned (drawers/popovers), not parked.
- Radical negative space on brand and low-density surfaces: one focal element, generous void, asymmetric composition — never equal-thirds, never all-centered grids of identical cards, never card-in-card nesting.
- Spacing rhythm on an 8px-ish baseline; internal padding < gap between groups; tight 6px radii (2/4/6/10 scale); hairline borders.
- Data-as-design: the product's own artifacts (node graphs, receipts, gate tables, segmented progress) ARE the imagery, on the website too. No decorative illustration. Marketing sections use the asymmetric dot-label-left / artifact-right rhythm.
- z-index is a named semantic scale (base / chrome / overlay / modal / tooltip), no magic 9999.

9 STATE COVERAGE (a surface is not done until every state is designed)

Every surface designs: empty (focal point + next action, not a bare "no data"), loading (skeleton for structured content, nothing under ~200ms), error (recovery path, obeys the state language — inversion, not red), populated. Dashboard surfaces add: no-data vs not-yet-loaded, stale/cached, disconnected (Tauri/sidecar down), permission/empty-scope. All states keep the one-accent + contrast + type discipline. Worked reference: the project-selection screen (section 13) ships the full set live — populated / single-result / empty / loading-skeleton / error-inversion / offline-warn.

10 BANS (a surviving tell is a BLOCK at audit — design-taste.md enforced)

Blue anywhere in the ramp. A second accent hue. Amber/red/green status colors. System-default-only fonts on shipped UI. Gradient text; purple/teal>purple gradient decoration. Dead grays / pure #000 surfaces / pure #FFF surfaces. Glassmorphism or blur-as-reflex. Stars/glow/texture on work surfaces. Three-identical-cards layouts, all-centered no-focal compositions, divitis, card-in-card-in-card. Bounce/elastic easing, ambient motion on data views, scroll-listener animation, layout-property tweens. Badges on healthy elements. Lone percentages without a fraction.

11 BUILD NOTES (for the eventual implementation, not the mockups)

Emit colors as oklch() in CSS (modern Chromium webview) generated from the hex anchors above; keep role-named custom properties as the only color source in components (no literal hex in component styles). Bundle JetBrains Mono + the chosen hairline display face as variable @font-face files. Replace tokens.css steel/--info family per section 3 (steel demoted out of existence; info = dim text + glyph). The quinary glyphs ship as a tiny shared component, never ad-hoc unicode per view. Verify contrast with the ax-scaffold pixels per visual-ui-loop.md.

12 LIGHT THEME (locked: a full inversion, toggled on every screen)

Dark is the default and the brand's resting state; light is a complete role-token inversion, never a separate design. The toggle is a sun/moon icon present on every screen: the RESTING icon shows the current mode, hover cross-fades to the icon the click switches to (dark = moon at rest -> sun on hover, click goes light; light = sun at rest -> moon on hover, click goes dark). prefers-reduced-motion: instant swap, no rotate/scale.
Light role tokens — PROVISIONAL (maintainer retunes these later "to look better"; the SCOPE is locked — full toggle every screen — the VALUES are not). Currently mirror the ledger mockup's [data-theme="light"] ramp; eyedropper- and contrast-verify whatever lands, same gates as the dark ramp:
  brand light bg    --void  #e9ede9   (the boot/hero cosmos inverts to near-white)
  app-window light  --bg #eef1ef · --panel #ffffff · --raised #f4f6f4 · --sink #e6eae7 · --row #f7f9f7
  text              --text #0b0f0e · --read #2d342f · --dim #646b66 · --ghost #aab1ac
  hairlines         --line rgba(0,0,0,.30) · --line2 rgba(0,0,0,.30)   (shipped tune 2026-06-14; both equal, stronger for legibility on near-white)
  teal (darkened for near-white)  --teal #138a78 · --teal-bright #0f7e6f · --teal-deep #0c6256 · --teal-soft rgba(19,138,120,.40) · --teal-ghost rgba(19,138,120,.12) · --teal-faint rgba(19,138,120,.06)   (brand rings keep the dark-mode --teal-deep #1B7A6E family)
Rules: teal shifts DARKER on light so it stays >=3:1 on near-white; the bloom stays teal (slightly stronger); starfield stars become near-black on light. The state language is theme-independent — mono-inversion fail still inverts relative to its surface (a DARK chip with light text on the light theme), so the loudest-thing-on-the-surface rule holds in both. One accent, one teal, both themes.

13 BRAND & ENTRY SCREENS (locked from intro.html + select-terminal-styles.html)

Two screens precede the workspace and are BRAND-tier chrome (section 1), not work surfaces: the entrance (intro/boot) and project-selection (the session list).
Entrance — a staged cinematic build from pure black, in this exact order, every step motivated not decorative:
  1 the starfield fades up first: a 3-layer parallax canvas (far = many tiny, mid = fewer/bigger, near = few/biggest), every star pure white on dark (near-black on light), each dying in place at alpha~0 and reborn elsewhere on desynchronised timing. The field is a pure function of a SHARED wall clock + per-star hash, so it is identical at any instant on intro and the next screen and continues with NO blink across the navigation. This is the ONE sanctioned ambient motion, brand-only; it never touches a work surface.
  2 the teal bloom fades in (center radial), on its own layer; it is ALWAYS ON (the NO GLOW toggle is retired, 2026-06-14) over the dark vignette + stars.
  3 three concentric orbital rings fade in staggered (~370 / 560 / 750px, hairline --ring teal at .07 / .045 alpha).
  4 the white hairline wordmark draws stroke-by-stroke (section 5 geometry + timing): the single focal element, the rings/stars/bloom behind it, nothing else moving during the draw.
  5 the tagline fades in across the EXACT span of the draw (begins on the first stroke, full opacity as the last finishes).
  6 START fades in after the logo lands (bottom-center, square hairline border, teal underline-draw on hover). On START an exit curtain dissolves to the next screen's exact background colour so the hop between screens has no flash.
Persistent entry chrome: the sun/moon theme toggle (every screen) sits top-right alongside the window controls. The runtime-mode selector LOCAL / CLOUD / HYBRID is NOT on the intro (it lives on the project-selection screen, below). The logo is click-to-replay. prefers-reduced-motion resolves the whole sequence to its static end state (logo drawn, scene present, starfield on a held frame, curtain instant).
Project-selection — a desktop software-window: a 48px topbar (brand wordmark + title at left, the LOCAL/CLOUD/HYBRID mode selector CENTERED, the theme toggle + window controls at right), a scrollable centered content column (<=840px), and a thin bottom statusbar. The mode selector uses the START-button idiom (no box: a center→out teal underline on hover; the active mode keeps the full underline; choosing one calls set_execution_mode). Content is the recent-session list; each row shows ONLY real fields — PROJECT (name) · PATH · UPDATED (last-opened, relative). No invented columns and no index gutter; rows start at the project name. The full state set is part of the screen, not an afterthought, and is the worked reference for section 9: populated, single-result, empty (dashed box + a focal "+ new project" as the next action), loading (pulsing skeleton rows), error (mono-inversion fail-chip plus retry), offline/disconnected (warn-triangle box plus retry connection).
List style LOCKED = LEDGER (chosen over Catalog / Console / Manifest / Instrument, all explored in select-terminal-styles.html). Ledger = an accounting-ledger table held in its own bordered sub-window (a titlebar over a ruled body, lifted a step off the bg): a mono header row with brighter (--read) PROJECT / PATH / UPDATED labels, hairline rules between rows, tabular-aligned UPDATED, and NO index gutter (rows start at the project name); row hover AND select both draw the START-button underline — a teal line under the row, scaleX from the CENTER outwards (the selected row keeps the full line, its name brightening to teal), no row-fill and no receipt stamp (the mono-inversion stamp is retired for row-select, 2026-06-14; mono-inversion stays the §6 fail/error language). The new-project control is a CENTERED square hairline button with the same center→out teal underline-draw. This is the canonical ELAI list idiom: quiet, data-first, one teal accent, motion only as feedback.

14 OPEN ITEMS

- The hairline display FACE is unchosen (the voice is locked, the font file is not): pick via a local @font-face specimen round (candidates must ship a variable hairline weight; pair against JetBrains Mono). The intro mockup renders the display voice with Chakra Petch as a placeholder, not the locked face.
- Favicon/app-icon glyph distillation (prompt L2 in gpt-idea-prompts-2026-06-12.md, or local SVG round).
- TAGLINE unresolved (§1): none of the three candidates chosen. Needs a rethink — short, hits the trust/evidence wedge, avoids the generic-AI register. Revisit before any brand screen ships copy.
- Light-theme color VALUES (§12) are PROVISIONAL — maintainer retunes them later; only the toggle scope is locked. Eyedropper/contrast-verify whatever lands, same gates as dark.
- OKLCH emission values + measured contrast table — produce at implementation, not in this doc.
- INTERACTIVE NAV EFFECT (flagged 2026-06-14): the four question-tabs + nav buttons must use the center->out UNDERLINE idiom (the intro/selector mode-button + START effect — no dot, no box; hover draws the line from center, active keeps the full line), NOT the §4 dot-prefix. The 5px teal dot-label (§4) is for STATIC section labels ONLY; interactive nav = the underline. The shell pass shipped tabs with the dot — reconcile + apply in the shell-polish pass so all screens match.
(Resolved: the near-black ramp — §3 is pinned to the ledger mockup; the #0d1517 / #090d0c / #0a0f0e drift is gone, --bg #0a0f0e is the single app background.)
(Resolved: the website light-section question — ELAI ships a full dark/light toggle on every screen, section 12.)
(Retired 2026-06-14: the NO GLOW toggle is gone — the teal bloom is always on; and the runtime-mode selector moved off the intro onto the project-selection topbar, centered, START-underline style.)

15 USER ACCENT THEMING (planned Settings feature — NOT yet built; this section is the spec for when the Settings room exists)

The accent hue is TEAL by default and that teal IS the brand identity. On WORK surfaces only, the accent is user-themeable from Settings. Because every color is a role-named token (§3) set once on the document root, a single accent control re-skins every screen and panel through the CSS cascade with NO per-component work — the same property that makes the dark/light toggle (§12) a one-line flip. The legacy workspace + ~90 panels must first be migrated onto these role tokens (one-time token-unification pass) before the feature reaches them; the two brand/entry screens already are.

Scope (hard boundary — this is what keeps the feature from diluting the brand):
- BRAND surfaces are TEAL-LOCKED, never user-recolorable: the wordmark/logo, the intro/boot hero (its bloom + white logo stay teal regardless of any user accent), the website, marketing screenshots, the app icon, the default install. The signature teal is the recognition asset.
- WORK surfaces (the app window the user works in) carry the personalization. Default = teal.

The control (ONE hue, disciplined — the §10 single-accent ban still holds; the user chooses WHICH hue, never adds a second):
- A master HUE knob rotates the whole accent family (--teal / --teal-bright / --teal-deep + the derived soft/ghost/faint/ring/bloom) AND the subtle neutral tint, preserving each token's baseline saturation/lightness so the ramp keeps its structure — only the hue moves.
- A VIVIDNESS control scales saturation within clamps. The clamped S/L is what prevents neon slop.
- A MONOCHROME / DISABLE option is part of the package: pure black-and-white, no accent hue at all — the accent collapses to a neutral so the instrument reads graphite-on-black / ink-on-white. State is still carried by shape + mono-inversion (§6), so nothing breaks.
- The chosen accent persists (theme store) and is emitted as the same role tokens, round-tripping through dark/light.

NEVER rotates with the accent (semantic, meaning-bearing color): the data-viz / state palettes — FSM state badges, confidence signals, source-quality tiers, gate verdicts, GRC/orbit palettes. These encode meaning by hue and stay fixed; the accent drives ONLY the --teal family + neutral tint.

Marketing posture (decided): this is a QUIET delight feature discovered in Settings, never a headline. The wedge is trust-evidenced execution; cosmetic customization stays off the marketing page (it competes on the wrong axis). The live tuner lives at workflow/design/mockup/intro.html for tuning the defaults.

16 APP CHROME (the shared topbar law — every screen wears the same chrome)

Every screen — the Intro hero, project-selection, and the workflow shell — wears the SAME topbar chrome so the app reads as ONE program, never three. The chrome is built from SHARED components, never re-implemented per screen; a change is made ONCE and inherited everywhere. THIS HAS REGRESSED REPEATEDLY — the wordmark, the top-right control cluster, AND the back-chevron each drifted (different size/gap/padding/position) the moment their markup was copied into a second screen, and each was fixed the same way: extract ONE shared component with ONE set of coordinates. The rule is therefore ABSOLUTE: any chrome element that appears on more than one screen is EXACTLY ONE component (Wordmark / BrandCluster / TopRightControls), placed with ONE set of coordinates; copying chrome markup into a second screen is itself the bug, even if it "looks the same" at author time — it will drift. Locked rules (origin: settled 2026-06-14 after the three screens read as "different programs" and the top-right buttons popped on navigation — the fix is structural, shared components + one set of coordinates, not per-screen tuning):

- THREE-ZONE TOPBAR: brand wordmark LEFT, primary nav CENTERED, the control cluster RIGHT. The nav is absolutely centered on the bar's true center — the project-selection LOCAL/CLOUD/HYBRID idiom; the workflow shell's four question-tabs (PROGRESS/CODEBASE/FRONTEND/KNOWLEDGE) use the IDENTICAL absolute centering, and the brand on the left + status/controls on the right do not shift it. Bar height 67px on the work + selection surfaces (sized to seat the top-right cluster centered, see below); background --bg; a single --line hairline bottom border spanning the FULL width edge-to-edge.

- ONE TOP-RIGHT CONTROL CLUSTER, byte-identical and IDENTICALLY POSITIONED on all three screens: [ sun/moon theme toggle ] · [ hairline separator ] · [ minimize | restore | close ]. It is a single shared component (TopRightControls). Geometry is fixed and may not vary by screen: the toggle is 33×31, square (no radius), a hairline --line border, --read glyph going teal on hover, performing the §12 sun/moon swap; the window controls are the §6/§10 hairline set (close hovers to MONO-INVERSION, never red); 8px gaps, the separator 1px×18px --line. The whole cluster floats at top:18px, right:20px on EVERY screen — the Intro hero's resting placement, chosen as canonical; the hero uses the same component + coordinates, adding only the scene fade-in. MECHANISM (required): the cluster is VIEWPORT-FIXED on all three screens — position:fixed; top:18px; right:20px; align-items:center — NEVER a flex child of a bordered topbar, whose box-sizing + 1px border would shift it ~1px and make it pop. The project-selection + workflow-shell topbars are 67px tall specifically so this floating cluster seats centered (18px inset top AND bottom); the bar height serves the shared cluster, not the reverse. Same component, same coordinates, same result. A cluster that changes size or position between screens — "popping" on navigation — is a BLOCK.

- THE CHROME WORDMARK IS THE TEXT LOCKUP "ELAI" (§5): mono, semibold ~13px, .18em tracking, --text, top-left. The animated stroke-draw wordmark is the Intro HERO only.

- BACK AFFORDANCE + WORDMARK = ONE SHARED LOCKUP (BrandCluster.svelte): the hairline back-chevron (center→out underline idiom, §13; 26x26, 14px glyph) + the text wordmark are a SINGLE shared component used by BOTH the selector topbar and the workflow shell, at the SAME left inset (16px) and the same internal gap (6px) — so they are pixel-identical in size, spacing and position and cannot drift. The chevron navigates to the previous screen (selection → intro; shell → selection). Page-specific bits (the selector's "· session select" subtitle) sit AFTER the lockup, never inside it.

- INTERACTIVE NAV uses the center→out underline (§13/§14), never the §4 dot — the dot is for STATIC section labels only.

17 EXTERNAL CONNECTIONS (the connection guarantor)

External connections (MCP servers, the remote sidecar, cloud providers, SearXNG) are PROCESS-GLOBAL and PERSISTENT, not re-established per session the way Claude Code does — connect once, available in every session and pane until the user removes or overrides it. ELAI already ships the transport plumbing (MCP bridge C-39, defer-loading H-34, the C-42 authz gateway, canvas system-nodes that bind MCP via E-24 pin / G-4 re-pin); NEW here is the health + lifecycle + surface.
- The guarantor is a single connection registry that reports each connection in the §6 state language: active ● (connected, teal outline + the one slow pulse), pending ○ (connecting / reconnecting, dashed hairline), warn ▲ (degraded — connected but the last call failed or is slow, shape only, no own color), fail ✕ (cannot connect, the MONO INVERSION chip — the loudest thing on the dark surface, zero foreign hue). Silence means safe — a healthy connection carries no badge (§6 doctrine).
- It answers, without opening a session: which MCP / provider is connected, which FAILED, WHEN, and the last error (depth on demand). Auto-reconnect with backoff; a manual reconnect per row.
- Session scope: default = shared across ALL sessions; a per-session override (pin a different endpoint, opt a connection out) is the explicit exception shown on the session, never the default.
- Where it lives (ELAI "no window zoo" §8 — surfaces are summoned, not parked): a CONNECTIONS section in the zero-width flyout + a single connection indicator on the 22px statusbar that goes to the fail ✕ inversion when any connection is down (so it is noticed without opening the flyout). The canvas system MCP-connector nodes read the SAME registry state.
- States (§9): empty (focal "+ add connection"), connecting (skeleton/pending), connected (silent), degraded (warn + retry), failed (mono-inversion + last-error + reconnect). No glow on work surfaces (§10).

18 GIT + REVIEW SURFACE (commit and diffs WITHOUT the model)

From the Codex desktop review pane (verified: stage/revert all, per-file and per-hunk stage/revert, commit with an OPTIONAL auto-message, branch + PR creation — plain buttons, no model call). Deliberate divergence from Codex: its IDE extension has NO accept/reject diff flow (edits land straight in the tree) and its desktop approval diff was reported "too small, ~5 lines, no expand". ELAI does the opposite — an expandable accept/reject review is the default.
- Plain-git actions, no LLM: stage / unstage / revert at file AND hunk grain; commit (message field + an OPTIONAL "suggest message" button, the only model-touching control, never required); create branch; create PR.
- The diff obeys §6 — added / removed carry NO status color (amber/red/green are dead): added = a left --teal-deep bar + brighter text, removed = --dim + a struck bar; reads on shape + position + weight. Click a line -> open it.
- Accept / reject: an agent's edits land in a REVIEW state, not silently in the working tree; accept (stage) or reject (revert) per hunk. Composes the shipped G-5 Reviewer Cockpit + H-52 finding-anchoring (verbatim snippet -> hunk, hallucinated line numbers structurally impossible).
- Lives as a first-class pane alongside the canvas (§8 "chat lives alongside the canvas as a first-class pane"), or summoned as a review drawer.

19 NOTIFICATIONS (the alert model)

ELAI already fires completion / needs-input notifications (NotificationManager + H-36, OSC9 + BEL + OS toast). This specs the model.
- Triggers: task done, needs-approval / needs-input, AND error / fail (Codex covers the first two; we add error).
- Delivery setting (Codex's verified three-way): never / always / only-when-unfocused, per-trigger; an OPTIONAL distinct sound per trigger (Codex users requested sound, it was missing).
- Prominence (against Codex's "too small / easy to miss"): a needs-approval or fail also raises a PERSISTENT mark — a "waiting" badge on the relevant surface + the statusbar going to warn ▲ / fail ✕ — that holds until acted on, not a transient toast only. §6 glyphs, no glow, honors prefers-reduced-motion (§7 — animation swaps to static end state).

20 LAUNCHER / HUB (open-in-X + per-project action buttons)

From Codex desktop's "Open" button, configurable top-of-window action buttons, floating pop-out windows, the codex:// deep links, and CLI<->desktop handoff (all verified). Adapted to the ELAI shell.
- An "open in" menu (in the flyout or the §16 top-right control cluster): open in editor / IDE, open a terminal at the project root, reveal the project in the OS file manager, open the repo on its remote in the browser. Plain shortcuts, no model.
- Per-project action buttons: a small user-configured row of frequent commands (build / test / run / deploy), project-scoped.
- Deep links: an elai:// scheme (new-session / settings / connections / a session id); cross-surface handoff (a CLI session -> the desktop shell) mirroring Codex /app.
- Pop-out: detach the chat or a panel into a floating OS window when the user wants it on top during iteration.

21 INTERACTION AFFORDANCES (the VS-Code-grade QOL layer)

The small expected ergonomics, ranked by the research's "pain if missing"; the top band is table-stakes (absence is resented, not perceived as a feature). All obey §3 / §4 / §7 / §10.
- Table-stakes (ship first): the command palette (already present, keep it keyboard-first); quick-open fuzzy file / session jump (Ctrl/Cmd+P); CLICKABLE paths + links in agent output / chat / terminal (auto-detect URLs + file refs incl. file:line:col, Ctrl/Cmd+click opens at the exact line, highlight the source ref as feedback — never a blind jump); copy path / copy RELATIVE path on every file node + chip; reveal-in-tree (sync the file explorer flyout to the active file) AND reveal in the OS file manager; open-to-the-side / split; a back / forward navigation stack across jumps.
- Media + content: inline image preview; hover-to-preview an image path; render images + links in chat and markdown (the memo: "highlight links, make them clickable, same for images").
- Context menus: every file node answers, without mode-switching — where is this (reveal in tree / OS), open it (here / beside), copy its path (absolute / relative).
- Ambient feedback: status-bar progress for long ops (calm, persistent — fits the one thin 22px statusbar §8, never a blocking spinner); reversible operations (composes shipped H-49 time-travel + H-35 rewind).
- Optional / make-dismissible (the research's polarized items): sticky scroll, minimap — OFF or trivially togglable; do not force context aids.
- Build / Tauri note (research caveat): reveal-in-OS, native file-manager open, and robust terminal path:line:col parsing need a NATIVE bridge + cross-platform path normalization in the Tauri layer, not pure web-view DOM behavior.

22 OPEN ITEMS (feature-layer §17-21)
- Connection registry (§17): the process-global lifecycle owner location + the SSB connection-state rows (reuse McpToolDeferred family or add ConnectionStateChanged) are a build decision.
- Deep links + pop-out (§20): confirm Tauri v2 multi-window + deep-link plugin support the elai:// scheme before committing.
- No-LLM diff encoding (§18): the added / removed shape+weight scheme (no status color) needs a §3 contrast pass.
- QOL v1 scope (§21): table-stakes band is v1; sticky-scroll / minimap / advanced image-diff deferred.
- Whether §17-21 ship in the ELAI teal language or wait for the SIGNAL adoption decision (DESIGN-TEIKOKU §0 open item) is the owner's call.
