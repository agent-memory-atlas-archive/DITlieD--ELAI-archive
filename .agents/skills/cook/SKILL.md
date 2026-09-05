---
name: cook
description: Design cooking — the co-creation front-end that DISCOVERS a design instead of assuming one exists. USER-INVOKED ONLY — triggers when the user types /cook, /cook <surface>, says "cook the design", "let's cook this", "help me figure out the look", "i don't know what this should look like", "design this with me", or "what should X look like". Runs FRAME (ground the surface + pull real references) > DIVERGE (generate 2-3 genuinely distinct directions as real gpt_image renders the user reacts to, because design is discovered by reacting to artifacts not answering questions) > CONVERGE (grill's engine to pin the three dials, the locked accent, type hierarchy, focal layout, and the full state set) > EMIT a design brief that design-taste's pre-flight + visual-ui-loop build against. Surface-agnostic (dashboard UI, a feature's UX shape, branding, a CLI's interaction design). Read-only on code, any FSM state, never auto-fires, never transitions the FSM. The AI brings directions the user did not ask for; the human's aesthetic call is final.
license: MIT
compatibility: "Designed for Codex. Works on Codex with explicit $cook invocation. Requires the gpt-bridge MCP for the DIVERGE render phase."
metadata:
  version: "1.0.0"
  author: "elai"
---

Cook is design discovery, not design specification. The premise: neither side knows what the thing should look like yet, and that is normal — design is DISCOVERED through reacting to concrete artifacts, not specified up front. Cook's job is to help the user think: bring real, distinct directions to react to, narrow toward the one that resonates, and hand off a design brief a build can run against.

Why this exists: design-taste.md (the anti-slop quality bar) opens with "declare the design read before any visual work" — it ASSUMES the read already exists and only judges whether the output is generic. visual-ui-loop.md is the build-time critique loop. Neither helps you ARRIVE at the read when you and the AI both start blank. Cook is the missing front-end: it produces the design read + dials + brief that design-taste judges and visual-ui-loop builds. Flow: cook (discover the read) -> design-taste (taste bar) -> visual-ui-loop (build + screenshot-critique).

What makes cook NOT grill: grill is a convergence engine (recommended-answer-first, resolve-the-known-fork, codebase-already-settles-it). Early design is DIVERGENT first — forcing convergence before the user has reacted to real options railroads them into the AI's first idea, which is the opposite of discovery. Cook diverges first (generate + react), THEN borrows grill's convergence engine to pin the decisions once a direction is chosen.


HARD CONSTRAINT — USER-INVOKED ONLY

NEVER auto-fire cook off a keyword or the shape of a task. It runs ONLY when the user explicitly asks to cook/design something with you in the current message. An unprompted design interrogation (or worse, an unprompted burst of image generation) is friction and burns the rate-limited gpt-bridge pool. If a task looks design-shaped and underspecified, OFFER it in one line ("want to cook the look for this before i build it ?") and wait.


WHEN TO INVOKE

- user types `/cook` or `/cook <surface>`
- user says "cook the design", "let's cook this", "help me figure out the look", "design this with me", "i don't know what this should look like", "what should X look like"
- user hands a visual/UX surface with no clear direction and asks to discover it together

When NOT to invoke:
- the design is already decided (a mock, a brief, or a clear reference exists) — go to design-taste's pre-flight + build, no cooking
- the user wants ONE exact image rendered — that is a direct gpt_image call, not a discovery loop
- a pure fact lookup ("what's the contrast ratio rule") — that is oracle / design-taste, not cook
- the user gave a concrete build command — do not stall them; confirm the one genuine direction if any and proceed


THE METHOD (FRAME > DIVERGE > CONVERGE > EMIT)

1 FRAME (silent reasoning + grounding, before any render). Through mcp_think (GRC is mandatory here; built-in thinking is disabled), establish: the surface and its operator task, who it is for, the hard constraints (existing accent/tokens, brand, platform, the Tauri-webview stack if it is the dashboard), and the design space — which axes can actually vary (structure, color, motion, density, voice). Then GROUND THE DIRECTIONS: do not generate from the model's defaults (that is where AI-slop is born — design-taste.md names the same handful of tells every untrained model reaches for). Pull real prior-art / references first: WebSearch the pattern, name 2-3 real design languages or products whose look fits the read, optionally an oracle call. Each DIVERGE direction will be anchored to one of these named references, not a vibe. If it is a dashboard surface, read what already exists (codegraph-elai / dashboardRules.md component vocabulary + the locked accent) so a direction does not fight the system.

2 DIVERGE (real gpt_image renders — the core of discovery). Generate 2-3 GENUINELY DISTINCT directions (not three variations of one safe idea — distinct structure/color/voice, each anchored to a different named reference from FRAME). Render each as a real image, because a layout/feel is decided by SEEING it, not reading a description.
- Construct every prompt per .Codex/rulebook/gpt-image-prompting.md: a written VISUAL SPEC not keyword soup, the aspect ratio + style folded into the prompt text (gpt_image is PROMPT-ONLY — no size/quality/n params), reference looks described IN WORDS, exact text-in-image rules if the mock carries labels.
- gpt_image is ASYNC: gpt_image returns {job_id, account}; poll gpt_image_status until done; gpt_image_result harvests the saved file. Re-prompt with the FULL spec to iterate (gpt_image_retry), never a terse "make it better".
- Present the renders and let the user REACT. Reaction is the signal — "this one, but calmer", "the second's color, the first's layout", "none, too corporate" all move the design forward. The AI BRINGS A DIRECTION THE USER DID NOT ASK FOR in the set (the generative, help-them-think beat) — at least one option should surprise, not just safely interpolate the brief.
- Iterate: fold the reaction into a new spec, re-render, narrow. Do not converge yet.

3 CONVERGE (grill's engine, once a direction resonates). Now run the grill method (see .Codex/skills/grill/SKILL.md — one question at a time via AskUserQuestion, recommended answer first and grounded, pin overloaded terms, surface contradictions, decide edge cases) to pin exactly what design-taste.md needs to judge and visual-ui-loop needs to build:
- the THREE DIALS with justified values (STRUCTURE_VARIANCE / MOTION_INTENSITY / VISUAL_DENSITY, design-taste.md) — chosen from the resonant direction, not defaulted.
- the ONE locked accent in OKLCH + the neutral-tint, the type hierarchy (display/body/caption steps + an authored web font, not system-default-only), the focal element + reading order.
- the FULL STATE SET this surface can enter (empty / loading / error / populated + dashboard's no-data / stale / disconnected / permission) — the design's edge cases; a direction is not chosen until its non-happy states are designed, not just the hero shot.
- pin overloaded DESIGN terms on sight ("clean" = minimal-chrome or high-contrast or airy-spacing ? "modern" = which named language ?). Surface contradictions flatly (a chosen look that fights the operator task, or breaks the locked accent / the contrast gate).

4 EMIT the design brief (below), persist it, and hand off.


OUTPUT (the design brief — handed to design-taste pre-flight + visual-ui-loop / dashboardRules)

Persist to workflow/design/{surface-slug}.md (create the dir on first use) AND show it:

```
DESIGN BRIEF — {surface}

DESIGN READ (one line, design-taste.md form)
- reading this as: {surface} for {who}, {vibe} language, leaning {system/aesthetic}

DIALS (design-taste.md, justified)
- STRUCTURE_VARIANCE {n}: {why}
- MOTION_INTENSITY  {n}: {why}
- VISUAL_DENSITY    {n}: {why}

DIRECTION (chosen, with the render that decided it)
- anchor reference: {named language/product}
- chosen render: {path to the gpt_image_result file}
- in one line: {what this direction is}

PALETTE / TYPE
- accent (locked, OKLCH): {value} ; neutrals tinted toward it
- type: {display / body / caption + the authored font}

LAYOUT
- focal element: {what is seen first} ; reading order: {2nd, 3rd, never}

STATE SET (every state this surface enters, each designed)
- empty / loading / error / populated [+ no-data / stale / disconnected / permission]

GLOSSARY (design terms pinned this session)
- {term} = {the one thing it means here}

OPEN-FOR-GROUNDING (verify before build)
- {claim} > {oracle / WebSearch / paper-ingestor}

NEXT
- dashboard surface > design-taste.md pre-flight, then build via visual-ui-loop.md (renders are the baseline, NOT the final pixels).
- non-dashboard surface > this brief is the spec; build against it directly.
```

The DESIGN READ + DIALS + PALETTE/TYPE + STATE SET are the spec design-taste.md assumes exists. The chosen render is a DECISION BASELINE, never the shipped pixels — visual-ui-loop.md owns the real screenshot-vs-render judgement at build time.


COMPOSES (the design asset set cook orchestrates — it invents none of these)
- grill (.Codex/skills/grill/SKILL.md) — the CONVERGE engine (one-question interrogation, term-pinning, contradiction-surfacing).
- design-taste.md (.Codex/rulebook/) — the dials, OKLCH/one-accent/contrast doctrine, anti-slop bans, and the pre-flight the brief must satisfy. Cook PRODUCES the read this file judges.
- dashboardRules.md (workflow/memory-bank/) — component/state doctrine, loaded when the surface IS the Svelte dashboard (assemble from existing components, do not freestyle).
- visual-ui-loop.md (.Codex/rules/) — the build-time screenshot-critique loop the brief feeds.
- gpt-image-prompting.md (.Codex/rulebook/) — how to construct the DIVERGE render prompts.
- gpt-bridge gpt_image / gpt_image_status / gpt_image_result / gpt_image_retry — the renders. CAVEATS: prompt-only, async, browser-subscription + rate-limit-fragile, WEAK grounding (one vote, a discovery aid — never a grounding source and never on a critical path).


CONSTRAINTS
- read-only on code. Cook never writes a project file (only the design-brief under workflow/design/), never transitions the FSM, never session-claims, never dispatches an executor.
- diverge BEFORE converge. Do not run grill's convergence on a design the user has not yet reacted to a real render of — premature convergence railroads the user into the AI's first idea and defeats discovery.
- the render is a BASELINE to decide from, not the final look. Real pixels are visual-ui-loop.md's job; do not treat a gpt_image mock as shipped UI.
- GRC-bound: the FRAME reasoning + the design space come from actual mcp_think calls with grounding, never a hand-written blockquote.
- grounded divergence: directions are anchored to named real references (WebSearch'd), not the model's defaults — that is how cook avoids generating the slop design-taste.md would later flag.
- the human's aesthetic call is final. design-taste.md notes the measured LLM self-leniency bias; cook proposes and narrows, the user decides the look. Do not self-score a direction as "good" and move on.
- ANTI-SPIRAL: if after ~3 render rounds the user cannot react toward anything, the gap is foundational (audience / purpose / the surface itself is unclear), not a styling detail. STOP rendering, surface the foundational question, do not keep burning the rate-limited image pool (anti-spiral skill + token-discipline.md).


END COOK SKILL
