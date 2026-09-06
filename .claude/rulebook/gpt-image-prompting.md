GPT image prompting (how to prompt ChatGPT's "Create image" tool via the gpt-bridge gpt_image MCP)

Loaded by: any session that calls gpt_image (gpt-bridge MCP) to generate an image — the canvas image-regen loop (.claude/rulebook/canvas-image-regen.md step 2), asset-generation, mockups, any one-shot render. Distilled from docs/researches/gpt-image-prompting-guide-2026.md (OpenAI official image-prompting guide / cookbook / GPT-4o image system card / Model Spec, ran via gpt-bridge Deep Research 2026-06-06). Read that file for the full reasoning + before/after examples; this is the operating rule.

THE MENTAL MODEL (the one thing that matters)
GPT image is an INSTRUCTION-FOLLOWING model, not a keyword-soup diffusion parser. Write a clear visual SPECIFICATION (a design brief / shot list), not a bag of weighted style fragments. This is the opposite of Midjourney/SDXL habits: do NOT write "ultra-realistic, 8k, masterpiece, award-winning, cinematic, trending". Describe the image in plain language as if you were briefing a photographer or designer. It binds object-to-attribute tightly, renders text well, and handles ~10-20 objects with order/count/relations stated in plain words. Grounding: research L16-24, L66.

THE BRIDGE CONSTRAINT BOX (read before prompting; gpt_image now also accepts reference images)
gpt_image({prompt, images}) takes a prompt string AND an optional `images` array (local image file paths) for reference / edit, added per the gpt-bridge update (handoff cont.11, live-verified: gpt_image edits an uploaded image, gpt_chat({prompt, images}) reads images back). There is still NO size, quality, n, model, input_fidelity, or mask parameter. Confirm the exact param shape against the live tool schema at call time (a session with a stale cached schema may still show prompt-only). Consequences, all load-bearing:
- ASPECT RATIO / SIZE: cannot be set as a param. State it IN THE PROMPT in words ("a vertical 3:4 poster", "square 1:1 icon", "wide 16:9 banner", "tall portrait, taller than wide"). After harvest, VERIFY the PNG's real dimensions; ChatGPT may auto-crop or shift composition on its own. Do not assume you got the ratio you asked for.
- QUALITY TIERS: no draft/low vs high param. You always get ChatGPT's single tier. So spend the words on clarity instead; for dense text or fine detail, simplify the scene rather than reaching for a quality knob that isnt there.
- n VARIANTS: no n param. To explore variants, call gpt_image multiple times (each is its own async job/account) — do NOT stuff a dozen competing composition ideas into one prompt.
- REFERENCE IMAGES / EDIT / CONSISTENCY: SUPPORTED now via the `images` param (an array of local image file paths). Pass the source / reference image(s) to EDIT them (change X, preserve the rest), match an existing look, reuse a brand mark faithfully, derive an asset from a mockup, or iterate on a prior render. Verified behaviour: gpt_image recolors while preserving text + background; gpt_chat with images reads them back (vision). You can STILL also describe a reference in words when you have no file. Masks are not exposed; for a surgical change, state the preserve-list in the prompt and pass the image to edit.
- ITERATION: you CAN now iterate by passing the prior render in `images` with "change only X, keep the rest" (image edit). The edit operates on the image you pass, not on hidden prior-turn memory, so restate the preserve-list each call. Without a passed image it is still a fresh "Create image" each call.
- FIRST-PARTY NON-DETERMINISM: ChatGPT applies hidden system messages, tool-side prompt optimization, and safety rewrites. Determinism is lower than the raw API. Build with a clean reusable template + verify the output, not with one magic string. Grounding: research L70-72, L111, L129, L137, L225; gpt-bridge handoff (gpt_image is prompt-only async).

THE ORDERED TEMPLATE (stable order = control; reuse it every time)
Order: deliverable > scene/background > subjects (count + identity) > composition/framing > action + spatial relationships > lighting + mood > style/medium > materials + detail > text-in-image > constraints. For complex work use labeled segments / line breaks, NOT one giant paragraph. Grounding: research L24-57, L62.

  Deliverable: [poster / product hero / editorial portrait / app icon / UI mockup / diagram / sprite / texture]. Include the ASPECT/SIZE in words here since no param.
  Scene: [background, environment, time of day, setting]
  Subjects: [exact number, what each is, distinguishing traits]
  Composition: [wide / close-up / top-down / eye-level / low-angle / centered / left negative space / straight-on]
  Action and relationships: [who does what, where each subject sits, what is in front of what — state back>mid>foreground for busy scenes]
  Lighting and mood: [soft diffuse daylight / golden hour / overcast / high-contrast / cool-toned / restrained saturation]
  Style: [photorealistic photograph / flat vector illustration / watercolor / premium product photography / UI screenshot]
  Materials and detail: [skin pores + wrinkles, fabric weave, brushed metal, film grain, realistic reflections]
  Text in image: [EXACT text in quotes + typography + placement + how many times it appears]
  Constraints: [no logos, no watermark, no extra text, one subject only, preserve layout]

PEOPLE / ANATOMY (cuts warped hands, missing feet, wrong gaze)
Be literally physical: "full body visible, feet included", "looking down at the book, not at the camera", "hands naturally gripping the handlebars". Name body framing, scale, gaze, and every object interaction. If hands/poses come out wrong: REDUCE scene complexity, then describe pose + object contact explicitly. Grounding: research L64, L125.

TEXT INSIDE IMAGES (the #1 fragile area — treat text as a constraint, not a wish)
- Put literal copy in QUOTES and/or ALL CAPS. Prefix with EXACT, verbatim, no extra characters.
- Specify font style, placement, contrast, and HOW MANY TIMES it appears ("appears once only, perfectly legible").
- Spell out brand names / acronyms / uncommon words letter by letter for tricky spelling.
- If text garbles: shorten the copy, give it more physical space, use ONE font family, raise contrast, simplify the scene. Precise text placement is a known model limitation.
- Non-English text is weaker (GPT-ImgEval found Chinese signage errors); budget extra QA for non-English, dense layouts.
Grounding: research L74-97.

PHOTOREALISM (describe a real photo of a real moment)
The word "photorealistic" strongly engages the photoreal mode — use it. Then use photography language for lens/lighting/framing AS A LOOK CUE, not physics ("shot like 35mm film with a 50mm lens" good; "exact T-stop / optical falloff" ignored). Ask for REAL texture: visible pores, fine wrinkles, peach fuzz, slight under-eye darkness, fabric wear, believable reflections, subtle film grain, natural color balance, shallow depth of field. AVOID studio-polish/retouch hype words if you want natural realism. Translucency reads better as physical cues ("warm light bleeding through fingertips", "rim light through the ear edges") than as the tag "subsurface scattering". Grounding: research L99-107.

DIAGRAMS / UI / CHARTS / SLIDES (switch from art-prompt to artifact-spec)
Name the exact deliverable, define canvas + hierarchy, provide the REAL text/data, constrain typography + spacing, forbid tiny text and decorative clutter. Ask for a landscape/wide layout in words when there is small text or labels. Grounding: research L68.

EXCLUSIONS + EDITS (negatives work when they are true invariants)
- Phrase exclusions as invariants / preserve-lists: "no watermark", "no extra text", "no logos", "preserve identity / geometry / layout".
- For a change, say "change only X" + "keep everything else the same", and RESTATE the preserve-list on every re-prompt (no edit-turn memory via the bridge).
- Do NOT spray ALWAYS/NEVER/must/only across every sentence — reserve ONLY/EXACT/verbatim for precise copy, surgical changes, or non-negotiable brand rules. Grounding: research L117-119, L127.

KNOWN FAILURE MODES > FAST FIXES (structural beats poetic)
- garbled text > shorten copy, enlarge text area, one font, restate EXACT/verbatim.
- warped hands / bodies > reduce complexity, describe pose + object contact, "full body, feet visible".
- wrong composition / cropping > state placements in plain words, request the ratio in words, re-prompt from your best description (ChatGPT may auto-crop regardless).
- too saturated / too glossy > "neutral daylight", "restrained saturation", "natural color balance", "no cinematic grading or heavy retouching".
- over-sharpened when you wanted blur > restate the blur as an explicit invariant.
- a complex render can take up to ~2 min (the bridge is async; poll gpt_image_status, retry on hung). Grounding: research L121-125.

THE RULE OF THUMB: maximize the CLARITY-to-length ratio, not length. Minimal, paragraph, JSON-like, and tag prompts all work; long prompts work but are harder to debug. Put the most important instructions early. Grounding: research L131.

CANVAS IMAGE-REGEN TIE-IN
When regenerating a clicked asset: you already Read the current image (canvas-image-regen.md step 1), so DESCRIBE what you saw (subject, palette, framing, style, the role it plays in the screen) as the implicit reference, THEN state the operator's change as "change only X, keep the rest". Encode the asset's needed shape in words (e.g. "square app icon, flat style, transparent-looking flat background" — note true transparency isnt guaranteed via the bridge). Keep the app's visual register / concept-lock if one is active.

COPY-PASTE BASE (fill, delete unused lines, keep the order)
  Deliverable: [what + aspect/size in words]
  Scene: []
  Subjects: [count + identity]
  Composition: []
  Action and relationships: []
  Lighting and mood: []
  Style: []
  Materials and detail: []
  Text in image: EXACT, verbatim, no extra characters: "[copy]" — [font/placement/contrast], appears once only
  Constraints: no watermark, no extra text, no logos, [preserve-list]
