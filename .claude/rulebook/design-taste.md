Design taste (anti-slop) — the look layer for the dashboard (HARD RULE for design-shaped UI work)

Loaded by: any task where the dashboard output is JUDGED ON HOW IT LOOKS, not just whether it renders — Svelte component visual design, the orbit/graph views, theming, or any "make this look good / less generic / more polished" request. Pairs with visual-ui-loop.md (the screenshot-compare LOOP that scores against this taste), dashboardRules.md (component/state doctrine — written in React terms, real stack is Svelte 5 + Vite + Tauri v2), and the Dashboard/frontend trigger in conditional-routing.md.

Why this exists

visual-ui-loop.md scores the dashboard but its critique step says "derive the checklist from the actual target" and ships no library of what good-vs-slop looks like, so the checklist gets filled from vibes. An LLM trained on a million near-identical SaaS dashboards defaults to the same handful of tells every time, and a "does this look right" self-review rubber-stamps them. This file is the missing taste layer: the named AI-default tells, the moves that read as intentional, and a pre-flight every surface ticks before done. Stack: the dashboard runs in a Tauri v2 webview (modern Chromium), so this is the WEB translation — real CSS, real oklch(), real prefers-reduced-motion, not a cut-down subset.

THE ONE LINE
Before building a dashboard surface, declare a one-line design read and the three dial values, reason in order (task > hierarchy > states > components > build > critique), then prove zero anti-slop tells survive a pre-flight before calling it done. Generic-looking output is a defect at the same severity as a broken layout.

THE DESIGN READ (declare before ANY visual work, one line, blocks building)
If the read does not exist yet because the look is genuinely undecided (no mock, brief, or reference, "design this with me"), do not invent one here off the model's defaults: cook (.claude/skills/cook/SKILL.md) is the discovery front-end that produces this read (diverge on real renders, then converge), offer it first (cook is user-invoked only, never auto-fired), then return to this pre-flight to judge what it emits.
State, in one sentence: "reading this as: [surface] for [who], [vibe] language, leaning [system/aesthetic]." Then REASON IN ORDER, never jump to markup: the operator task on THIS surface > information hierarchy (what is seen first, second, never) > the full STATE SET this surface can enter > map each piece to an existing Svelte component > only then build > then critique. Skipping to build is how the AI-default layout and missing states slip in.

THE THREE DIALS (set explicitly, reason from the read, never silently default; values 1-10)
STRUCTURE_VARIANCE (1 centered/symmetric/safe ... 10 asymmetric/editorial). A data dashboard trends low-mid (legibility first); a landing/marketing view can go high.
MOTION_INTENSITY (1 static ... 10 choreographed). A live-data view stays low (motion must never fight a reading task); transitions and onboarding can rise.
VISUAL_DENSITY (1 airy ... 10 packed cockpit). A graph/table view is dense by nature; an empty or status view is airy. Baseline 3 / 2 / 6 for a working dashboard, tuned per surface. The values are CHOSEN and visible, so a reviewer can argue with them.

COLOR
- Reason and emit in OKLCH. The Tauri webview is modern Chromium, so oklch() works in CSS directly — use it (perceptually even ramps), do not hand-roll hex steps that drift in lightness.
- One accent hue, locked across the surface. A second "accent" is almost always slop; tints/shades of the one accent are fine, a competing hue needs a real reason. Reserve any second strong color for a semantic state (success/warn/error), never decoration.
- Tint neutrals toward the accent. Dead pure-gray (#808080-ish) panels/text and pure #000/#FFF read as untextured and AI-default; even 2-4% accent in the neutral ramp reads as authored. Layer dark surfaces by tonal elevation, not by stacking shadows.
- Contrast is a hard gate: >=4.5:1 body, >=3:1 large text / meaningful UI strokes/icons, measured over the ACTUAL background (a value over a busy graph, not over a flat swatch). The ax-scaffold + ui-diff path in visual-ui-loop.md gives you the real pixels to check against.

TYPOGRAPHY
- The bare system-default stack as the ONLY choice is the local "Inter everywhere" tell. Ship an authored web font (@font-face / variable font) that matches the read; a variable font is one file, multi-axis, dynamic-type friendly.
- Pair on CONTRAST not similarity; build a real hierarchy (display / body / caption with visible size+weight jumps), not three near-identical sizes. Cap weights at ~3 per surface, families at ~2.
- Body 14-16px floor on a content surface, unitless line-height 1.4-1.6 body / 1.1-1.3 headings. Design in grayscale first: if the hierarchy fails without color, color will not save it.

LAYOUT AND COMPOSITION
- Vary spacing to make rhythm on an 8px-ish baseline; internal padding < the gap between groups. Uniform equal gaps everywhere is the flat-template tell.
- Kill the equal-thirds reflex: three identical cards in a row, identical panels in a grid, everything centered with no focal point are the layout AI-defaults. Give the surface a focal element and a deliberate reading order sized to STRUCTURE_VARIANCE.
- Avoid div-soup card-in-card-in-card nesting; flatten or earn each layer. Avoid divitis (a wrapper per style); use semantic elements + utility/scoped classes.
- Semantic z-index scale (named layers: base / chrome / overlay / modal / tooltip), never an arbitrary 9999 to win a stacking fight you did not diagnose.

MOTION
- Motion must be MOTIVATED: hierarchy, feedback, state change, storytelling. "It looked cool" is not a reason; unmotivated motion on a data view harms the reading task and is a defect.
- Ease out, do not bounce (exponential/quart ease-out reads expensive; elastic/bounce reads like a toy unless the read wants playful). Routine durations 0.15-0.3s, larger transitions to ~0.4s, hero moments once.
- Respect prefers-reduced-motion (CSS has it natively): swap transform-heavy moves for crossfades, kill auto-loops/parallax. Essential feedback (a press state, a loading pulse) stays.
- Animate transform/opacity, never width/height/top/left/margin in a tween (they relayout every frame). No window.addEventListener('scroll') for scroll-linked motion — use IntersectionObserver or CSS scroll-driven animations.

STATE COVERAGE (a surface is not done until every state it can enter is designed)
- Never ship only the happy path. Design: empty (no data yet), loading (skeleton for structured content, spinner only for short opaque waits, show nothing under ~200ms), error (failed load/action with a recovery path), and populated/success.
- Dashboard surfaces add: no-data vs not-yet-loaded (different), stale/cached, disconnected (socket/IPC down — Tauri backend unreachable), and permission/empty-scope. Enumerate which apply in the design read's state-set step.
- Each non-happy state still obeys the rest of this file: an error panel keeps the one-accent + contrast + authored-font discipline; an empty state has a focal point and a clear next action, not a bare "no data".

ASSEMBLE FROM EXISTING COMPONENTS, DON'T INVENT
- Emit against the dashboard's existing Svelte component vocabulary and tokens, not freeform markup re-invented per view. Assembling pre-built components holds the system together; freeform is where token drift and one-off colors creep in. Read dashboardRules.md for the component/state doctrine.
- NO COMPONENT YET is a finding, not a license to freestyle: scope the component + its states first (research the pattern if unsure via gpt-bridge / oracle), author it into the component set, then build against it. Capture it so the next view assembles instead of re-inventing.

ANTI-SLOP BANS (a surviving tell is a BLOCK at audit)
- System-default font as the only typeface on shipped UI; gradient text; the purple>blue (or teal>purple) gradient reflex as default decoration.
- Gray text on a colored/data fill failing the contrast gate; pure #000/#FFF/dead #808080 palette with no tint.
- Three identical cards in a row as the default content layout; everything centered with no focal hierarchy; card-in-card-in-card as the only depth cue; divitis.
- Glassmorphism/heavy blur as a reflex with no reason in the read; two competing accent hues; theme inverted mid-surface without reason.
- Bounce/elastic as the default transition; unmotivated ambient motion on a data view; scroll listeners driving animation; tweening layout properties.

THE PRE-FLIGHT (run before declaring any surface done; every line must hold)
This seeds the visual-ui-loop.md critique checklist — each line becomes a schema-constrained pass|fail|unknown with a real evidence region (from a ui-diff hotspot or ax-scaffold.json), unknown counts as fail. The pre-flight is an EARLY FILTER, not the final word: an LLM scoring its own UI has a measured leniency / self-favoring bias, so a clean pre-flight gates the work forward but maintainer judges the aesthetic call and gives the go.
  design read declared + reasoned in order (task > hierarchy > states > components > build > critique)
  three dials set with values justified from the read
  color in OKLCH, one locked accent, neutrals tinted not dead-gray
  contrast measured over the real background: >=4.5:1 body, >=3:1 large/icon (use ax-scaffold pixels)
  authored web font in place (not system-default-only), hierarchy has visible display/body/caption steps
  layout has a focal point and spacing rhythm, not equal-thirds / all-centered / nested-cards / divitis
  z-index is a named semantic scale, no magic 9999
  motion motivated, eases out not bounce, animates transform/opacity not layout, respects prefers-reduced-motion, no scroll listeners
  every state this surface can enter is designed (empty / loading / error / populated + no-data, stale, disconnected, permission)
  built by assembling existing components; if none existed, one was scoped and authored first
  every ANTI-SLOP BAN line checked and absent
If any line is unknown or fail, the surface is not done — it is generic. Fix the one highest-impact item, then re-run the visual-ui-loop (one fix per loop, two clean passes to stop).

THE TASTE RUBRIC (explicit yes/no, per surface — floor tier + brand tier)
The pre-flight above is prose; this is its yes/no form so "looks fine" cannot pass and so the two tiers stay separate. The split is deliberate (grounded in the Taste Labs anti-slop method, workflow/reasoning/taste-labs-anti-slop-research-2026-07-02.md): raise the FLOOR first (kill the crude tells — this is what stops slop), THEN personalize with the BRAND tier (the defended signature — this is what makes it ELAI and not a clean generic dev tool). De-slop that only runs the floor tier produces exactly that clean generic result; the brand tier is the taste the floor tier alone strips out. Score each line yes/no/unknown; unknown = no. Run taste_check.py (.claude/tools/taste_check.py) FIRST as the deterministic leg — a measured NO from the checker outranks any vibes-based yes.

FLOOR TIER (universal anti-slop; every ELAI surface, work and brand alike; a NO on any line is a BLOCK — this is the raise-the-floor pass):
  F1  an authored web font is bundled (@font-face / var(--font-*)), NOT a system-default stack as the only face
  F2  exactly ONE accent hue, locked across the surface; no second decorative hue (semantic state colors exempt)
  F3  neutrals tinted toward the accent (2-4%); no dead #808080, no pure #000 / pure #FFF surface
  F4  contrast measured over the REAL background: >=4.5:1 body, >=3:1 large text / meaningful UI strokes-icons
  F5  a real type hierarchy (visible display/body/caption jumps), <=2 families / <=3 weights
  F6  a focal element + spacing rhythm; NOT equal-thirds, NOT all-centered-no-focal, NOT card-in-card, NOT divitis
  F7  no gradient-as-decoration, no glassmorphism/blur-reflex, no unprompted glow/bloom/neon; elevation = a lightness step + a 1px line
  F8  motion motivated, eases OUT not bounce, animates transform/opacity not layout props, respects prefers-reduced-motion, no scroll-listener animation
  F9  every state designed (empty = focal next action / loading = skeleton / error = recovery, never color-only / populated; live views add disconnected + stale)
  F10 assembled from existing components (a missing one is scoped + authored, not freestyled); color comes from role tokens, no literal hex in components
  F11 z-index is a named semantic scale, no magic 9999

BRAND TIER (ELAI identity; judged only AFTER the floor passes; a NO here means the surface is clean but GENERIC — not ELAI — which on a brand/entry surface is a BLOCK and on a work surface is a WARN unless the design file scopes the signature there. This is the personalize pass the pure de-slop dropped):
  Locked brand facts (true regardless of the signature reconciliation):
  B1  the one accent IS ELAI teal (--teal #2BBBA4 dark / #138a78 light); never blue, purple, indigo, amber, red, or green in the ramp
  B2  near-black app ground --bg #0a0f0e, teal-tinted, never blue-black
  B3  fail/error is the MONO-INVERSION chip (light chip, dark text, the loudest thing on the dark field), never red
  B4  status reads by SHAPE + weight + position first, color only whispers (survives a squint / a monochrome display, WCAG 1.4.1)
  B5  chrome is shared components with ONE set of coordinates (Wordmark / BrandCluster / TopRightControls); copying chrome markup into a second screen is itself the bug
  B6  role-named tokens are the only color source, so a light flip / an accent re-theme is a one-line change at the root
  Signature slot (the 1-2 DEFENDED taste moves — the thing that is ELAI and not a generic instrument; fill from the reconciled design file once chosen. Until chosen, a brand surface scores UNKNOWN here = fail):
  S1  [SIGNATURE-1: __________] present and applied as THE idiom on this surface, not a one-off
  S2  [SIGNATURE-2: __________] present where the design file scopes it
  (candidate pool pending the signature decision: mono-as-instrument typography, the boot stroke-by-stroke logo draw, the center-out underline idiom, the four question-tab IA, the teal dot-label. Whatever is NOT chosen is not a slop tell — it is simply not a signature; do not re-flag it as slop.)

Verdict: a surface has TASTE only when the floor tier is all-yes AND the brand tier is all-yes for its class (brand/entry surfaces need B1-B6 + the signatures; work surfaces need B1-B6 + any signature the design file scopes onto them). Floor-all-yes with brand-tier NO is the de-slopped-but-tasteless state this rubric exists to catch.

SELF-CHECK before calling a surface intentional
1 design read + dials declared, reasoned in order, not defaulted?
2 one accent, tinted neutrals, contrast measured over the real background?
3 authored web font with a real hierarchy?
4 focal point + spacing rhythm, not equal-thirds/centered/nested/divitis?
5 every motion motivated, eased-out, transform-driven, reduced-motion-respected, no scroll listeners?
6 every state designed, not just the happy path?
7 assembled from existing components (or one scoped + authored when none existed)?
8 every anti-slop ban absent at audit?
If any answer is no, it looks AI-default, which is a defect, not a finish. The pre-flight is my filter; the final aesthetic go is maintainer's, not my own self-score.

Grounding: synthesized 2026-06-11 from three portable anti-slop design skills (pbakaus/impeccable Apache-2.0, Leonxlnx/taste-skill MIT, heygen-com/hyperframes house-style + visual-styles) and two LLM-UI-design research syntheses maintainer provided — registry-based assembly over describe-the-rules (Design System-Compliant UI Generation, ACM CHI 2026), state coverage as a done-gate (NN/g heuristics), product/task-first ordering, and the LLM-as-judge leniency bias that keeps the human as final aesthetic authority (MLLM-as-UI-Judge). Translated to ELAI's stack: web CSS (real oklch(), prefers-reduced-motion, @font-face) for the Svelte 5 + Vite + Tauri v2 dashboard, not the Unity-USS variant that lives in the liteGAME root. Cross-ref dashboardRules.md (components/state) and visual-ui-loop.md (the loop + ax-scaffold contrast pixels). Web-stack tooling from the source reports (Figma MCP, shadcn/Radix/Tailwind, v0) deliberately NOT ported; the native analogs (dashboardRules.md, the ax-scaffold perception scaffold, visual-ui-loop as the critic loop) already exist.
