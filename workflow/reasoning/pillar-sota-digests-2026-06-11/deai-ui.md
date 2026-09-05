DIGEST: deai-ui.md
SOURCE: eeffa5e2-ca9a-4ac5-9f4b-d81985d92092.txt

TOPIC: De-AI-ifying LLM-generated interfaces — why LLM UI looks generic ("AI slop"), the recurring failure patterns, and the evidence-backed process to remove the fingerprint. Not pillar-numbered; a practice/process report. Citations are turn-style (citeturnNNviewMM), not URLs/arXiv — preserved verbatim where given; most named tools have no URL in this report.

REPORT VERDICTS (adopt/drop style — the report's prescriptions):
- ADOPT: define a design system (brand tokens) BEFORE the first generated screen — palette, semantic surfaces, typography roles, spacing scale, radii, shadows, component recipes. Most reliable anti-slop move. Best for dashboards/authenticated products/multi-screen apps; less for one-off campaign pages.
- ADOPT: prompt from references not adjectives — anchor with real apps/style-guides/layout crops/explicit constraints; shrink search space. "Build a beautiful UI" is a bad brief.
- ADOPT: generate screens/components atomically (one screen or one component class at a time); treat model as fast junior not design director; nail one screen then propagate the system.
- ADOPT: separate structure from style (structure-first, style-second two-pass) — Smashing experiment: direct component-library integration least effective, style JSON better, structure-then-style best.
- ADOPT: prompt format = brief-then-delta (first prompt = goal/context/elements/behaviors/constraints/done; follow-ups = what changes + what stays). Use precise language ("add 16px between these buttons") not "make it nicer".
- ADOPT: human polish loop on screenshots not just code — screenshot -> critique -> fix; paste screenshot to Claude "this looks off, specifically X, fix it" more effective than describing from memory.
- ADOPT: bake accessibility + interaction into the stack — use Radix / React Aria / Headless UI audited primitives, not model-invented div widgets ("deepest solution"). Adds minutes per component vs 45-90 min fixing later.
- ADOPT: context-preserving tool stacks keeping design context attached source->generation->review.
- DROP/AVOID (anti-patterns to ban in prompts): purple/blue hero gradients, glass cards, Inter-everywhere small body, pill buttons, soft default shadows, nested cards, big rounded icons above headings, identical three-up feature grids, centered body copy, untouched Tailwind/shadcn defaults.
- CORE THESIS: "do not let the model design; let it implement a design direction that you already constrained." UI quality is a context-and-constraints problem, not a find-the-magic-model problem.

CANDIDATES (named tools/systems/benchmarks/papers; most no-url in this report):
UI-Bench | no-url (citeturn29view5) | Evaluates 10 text-to-app tools, 30 prompts, 4,000+ expert judgments; moves field past demo-looks toward expert-rated visual quality | benchmark for UI quality
v0 (Vercel) | no-url (citeturn21view0/21view1) | Defaults to shadcn/ui, "specifically trained on default implementations" of shadcn components; supports custom utility classes/CSS variables/registries | the de-slop path = pass tokens/registry
Figma Make | no-url (citeturn33view0/33view4/33view6) | Can ingest production React design system package; point-and-prompt edits on selected elements; preserve existing frame structure | context-preserving generation
Figma MCP | no-url (citeturn22view0/22view1) | Passes variables/components/layout data into generation (Cursor/Claude/VS Code) | context-preserving stack leg
Google Stitch | no-url (citeturn23view0/23view2) | Supports DESIGN.md export/import of design rules across projects; real-time critique + live iteration + exports | design-system portability + critique loop
Tailwind | no-url (citeturn21view5/21view6/21view7) | Default theme variables are design tokens, a starting point not a finished brand; if undefined model has no system to follow | token drift source
shadcn/ui | no-url (citeturn17view0/21view7) | Default Tailwind/shadcn apps look the same; problem is untouched DEFAULTS (font/palette/spacing/proportions) | default-token lock-in source
Radix | no-url (citeturn26view2/34search0) | Encodes ARIA/keyboard nav/focus management/interactions out of the box; v0 inherits a11y advantages from shadcn+Radix | accessibility-into-stack
React Aria | no-url (citeturn34search1) | Encodes ARIA patterns/keyboard nav/focus/interaction by default | accessibility-into-stack
Headless UI | no-url (citeturn34search2) | Audited interactive primitives | accessibility-into-stack
Mobbin | no-url (citeturn24view0/24view1) | Vetted design references; now offers MCP/API entry so references usable directly inside AI workflows | reference-first workflow
CrowdGenUI | no-url (citeturn29view0/29view2) | Unguided UI generation tends toward generic solutions missing task context + user preferences; preference-guided does better | grounds template-stack/weak-hierarchy diagnosis
AlignUI | no-url (citeturn29view1) | Preference-guided methods outperform unguided generic output | grounds preference-guidance need
DCGen | no-url (citeturn28view0/28view1) | Motivating study on GPT-4o: three screenshot-to-code failure classes = element omission, distortion, misarrangement; segment-then-refine | grounds whole-page-inconsistency
ComUICoder | no-url (citeturn29view6/29view7) | On long/complex websites MLLMs struggle: fragmented segmentation, redundant component generation, frequent UI inconsistencies | grounds long-page failures + segmentation fix
VisRefiner | no-url (citeturn28view1) | Comparing rendered output vs target design and learning from visual differences materially improves layout fidelity | grounds screenshot-critique loop
PromptInfuser | no-url (citeturn30view0) | Coupling prompt and UI design helped designers identify incompatibilities earlier | grounds scoped-collaborator over one-shot
Figma Code Connect | no-url (citeturn22view4) | Hand the model the coded system directly instead of asking it to invent | context-preserving stack leg
WCAG (2.1 AA) | no-url (citeturn10search1/10search7/11search10) | Requires visible focus indicators + proper status/error messages; audit criteria for focus/status/error | accessibility audit oracle
Cursor (w/ Figma/MCP) | no-url (citeturn21view4/22view1) | Chat-based builder for brief-then-delta refinement | workflow target

ATTACK-THE-DESIGN (the report's strongest criticisms of unconstrained LLM UI):
- Template-stack layout: models converge on the same skeleton (centered hero, three-card grid, testimonial strip, CTA footer); the statistically-safest page, ignores brand/journey/information-architecture.
- Weak hierarchy / no primary action: page looks "clean" but everything equally weighted; model covers, emphasizes nothing; hierarchy needs user-specific prioritization the prompt didn't give ("gulf of execution and evaluation").
- Center-everything syndrome: overuses centered headings/body/symmetrical stacks; kills scannability/comparison/operational clarity; centering masks IA uncertainty.
- Default-token lock-in: page inherits toolkit personality not product; the one failure tied directly to tool architecture (v0 trained on shadcn defaults).
- Token drift: spacing/radii/shadows/font-sizes close-but-inconsistent (12 here, 15 there, 20 elsewhere); reads amateur; model invents reasonable local numbers without a project scale.
- Components render but don't feel designed: tidy but inert; uncanny valley; models weak at taste/restraint/editorial decisions even with design-system materials.
- Whole-page inconsistency on long/dense screens: element omission, distortion, misarrangement, redundant components as pages grow.
- Happy-path-only state design: no empty/loading/error/validation/status-for-AT states; demo-ready product-unready; prompts describe ideal screen so model designs ideal screen.
- Hover polish without real interaction system: hover styling present but focus invisible, expanded states unannounced, visual-only changes; collapses on touch/AT.
- Accessibility theater: DOM semantics/keyboard/labels/live-feedback inadequate; "a painting of a door" not a real button; non-semantic <div> toggle; semantic HTML/ARIA underrepresented in public code + <div onClick> token-cheaper.
- AI aesthetic fingerprint: forensic markers — purple/violet gradients, glass-on-white, Inter everywhere, pill buttons, soft shadows, nested cards, big rounded icons over headings, identical three-up grids, generic "premium startup" motion.
- Root cause: UI quality depends on constraints-with-taste; most prompts supply neither; visual plausibility outruns semantic/UX correctness; tool defaults intensify; workflow mismatch (AI used as one-shot art director vs scoped collaborator).

NUMBERS (claimed source, unverified):
- UI-Bench: 10 text-to-app tools, 30 prompts, 4,000+ expert judgments [citeturn29view5].
- Frontend Masters: AI-generated sidebar accessibility tree reports role=generic, name=none, focusable=false despite polished hover [citeturn26view0].
- Accessibility constraints+testing add a few minutes per component vs 45-90 minutes fixing after generation [citeturn26view6].
- AWM-style five-token file / locked spacing scale "snaps into rhythm" (qualitative before/after) [citeturn19view4/19view6].
(Note: this report is largely qualitative — few hard benchmark percentages; the load-bearing payload is the named failure patterns + tools + the smell-test/process checklists.)

CHECKLISTS PROVIDED (for reference, not expanded): "LLM UI smell test" (12 symptom items) + "Anti-AI-slop UI process" (9 workflow steps) + 3 concrete prompt templates (dashboard with banned-trope list; reference-anchored landing page; React-Aria/Radix data-table with WCAG audit step).

REPORT'S OWN LIMITATIONS: research base young; better at measuring structure/fidelity/preference-alignment than brand distinctiveness/"taste"; UI-Bench newer than code-correctness benchmarks; source asymmetry (official docs/papers strong for WHY tools behave; practitioner blogs/Reddit strong for WHAT outputs feel like) — treated first group stronger for causal claims, second for recurring symptoms.
