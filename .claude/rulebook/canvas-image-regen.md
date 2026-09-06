Canvas image regeneration loop (click an image > regenerate via ChatGPT > preview > approve > writeback)

Loaded by: the main LLM when the operator clicks an image element in the canvas-workbench and asks to regenerate / replace / restyle it via AI, OR when a canvas://edit-request of tier image-regen is pasted. Requires BOTH the canvas MCP and the gpt-bridge MCP to be connected.

ROLE BOUNDARY (load-bearing)
The canvas server NEVER calls gpt-bridge and NEVER generates an image. The main LLM is the sole orchestrator and the only component that can SEE the clicked image. Canvas resolves + packages the click and serves the preview file; gpt-bridge generates a PNG to disk; the main LLM is the glue and the eyes. The two MCPs share only the filesystem and the main LLM, never each other.

WHEN IT TRIGGERS
- operator clicks an image in the live canvas (kind=image) and states an intent ("regenerate this warmer", "make it a mountain at sunrise", "swap for a flat-style icon"), OR
- operator hits "Regenerate with AI" in the canvas ImageEditor, which composes an edit-request of tier image-regen and copies the @canvas:canvas://edit-request mention to paste in chat.

THE STRUCTURED PACKAGE (canvas://edit-request, tier image-regen)
Carries: file, ast_range {start,end}, oid, instanceIndex, scope:"one", element (with element.src AND element.images[] when the clicked element holds more than one image), requested_change.image_regen {prompt, from}, the server-enriched source_asset_path, writeback_mode, writeback_note, all_images[] (every image inside the clicked element — regenerate/account for ALL of them, not just the first), and screen_snapshot_path (a whole-screen PNG of this route).

MULTI-IMAGE + WHOLE-SCREEN CONTEXT (mandatory)
- all_images[]: when the operator hands over a BLOCK that contains several images (a row of cards, a gallery), every image src is here. Do not regenerate only the first — handle each image the operator means, or ask which ones if ambiguous.
- screen_snapshot_path: the cached whole-screen screenshot of the current route. READ IT (Read tool, vision) before composing any generation prompt, so the new image is consistent with the WHOLE screen (overall composition, palette, lighting, the other on-screen elements), not just its folder siblings. If the path is null/missing the snapshot was not captured yet — trigger one by reloading the screen in the canvas, or proceed with the sibling sweep alone and say so.

writeback_mode decision table (set by lib/imageResolve.mjs, trust it):
- uri                         dynamic source={{uri:'...'}} string literal. Persist by editing the literal.
- require-single             static require()/import asset referenced at EXACTLY one confirmed site. Safe to overwrite the asset bytes in place.
- require-shared            same asset required at >1 site. Overwriting changes every usage > do NOT overwrite; add a NEW asset + edit this one source site's require path, or ask the operator.
- require-single-unconfirmed  scan was capped or the asset is referenced via a Metro/tsconfig alias the resolver could not ground. Treat as require-shared for safety (do NOT blind-overwrite); read writeback_note, confirm the real ref count by grepping the app yourself before any overwrite.
- null                       no static literal resolvable (hashed/remote/Metro-only src). Resolve the asset yourself from the source file, or treat as uri if it is a remote image.

THE LOOP
0. CONTEXT SWEEP (MANDATORY, before touching the clicked image). TWO reads:
   a. Glob the sibling assets in the same folder as the clicked image (e.g. if the asset is assets/onboarding/skin-type/combination.png, glob assets/onboarding/skin-type/*.png). Read ALL siblings visually (Read tool, vision).
   b. Read the whole-screen screenshot at screen_snapshot_path (Read tool, vision) so you see the image IN CONTEXT of the full screen it lives on.
   This is non-negotiable: you must understand both the sibling set AND the whole screen so your prompt matches their composition, crop, angle, framing, lighting, palette, and style exactly. A generated image that doesn't match its siblings AND the surrounding screen is a failure even if it looks good in isolation. If the package carries all_images[] (a multi-image block), account for every image, not just the clicked one.
1. RESOLVE + READ (the "look at the image" step). From the package take source_asset_path. If non-null, Read it directly (vision). If null: open the component file at ast_range, find the require()/import/uri literal, resolve it to a disk path (data: > decode to temp; remote https > download to temp), then Read it. You must actually SEE the current image before prompting.
2. COMPOSE THE PROMPT. Load .claude/rulebook/gpt-image-prompting.md and follow it. Build a gpt_image prompt grounded in BOTH what you saw (the clicked image AND its siblings from step 0), the operator's intent, and the sibling style. Explicitly describe: the crop/framing/angle that matches the siblings, the lighting that matches, the background that matches. The prompt must produce an image that looks like it belongs in the same set. Use "change only X, keep everything else matching the other cards in this set" framing. gpt_image takes ONLY a prompt string (no size/count param) so encode any aspect-ratio / dimensions / style constraints AS TEXT in the prompt, and describe what you saw in words as the implicit reference (no image upload). Use the ordered template; phrase the change as "change only X, keep the rest". Keep the app's visual register (per design-rules.md / concept-lock if active).
3. GENERATE (best-effort, never block). gpt_image(prompt) > {job_id, account}. Poll gpt_image_status(job_id) until state="done". If status returns hung:true (past ~150s), call gpt_image_retry(job_id) and poll the NEW job_id. On POOL_EXHAUSTED, stop and tell the operator plainly that every account is unavailable (rate-limited / expired ~2026-09-04) — do not fake a result. Then gpt_image_result(job_id) > harvests PNG(s) to disk by absolute path.
4. PREVIEW (default: preview-then-approve). canvas_preview_image({ file_path: <harvested PNG abs path>, oid, instanceIndex }) > registers it under /gen-preview/<id> and live-swaps the rendered image in the running app via the overlay (visual only, no source change). Returns {ok, preview_id, preview_url}. Show the operator; if they want changes, refine the prompt and go to step 2. file_path must be a real image (png/jpg/jpeg/webp/gif, <15MB) or the tool rejects it.
5. APPROVE + WRITEBACK (only after the operator approves). Persist per writeback_mode:
   - require-single: copy the generated PNG over the existing asset bytes at source_asset_path (same path, Metro fast-refreshes, no source edit, no oid drift). Verify the overwrite then confirm.
   - uri: save the PNG into the app's asset dir, then edit the uri literal (canvas_writeback handles length-changing edits with oid-remap) to point at the new local asset (or a hosted url if that is the app's pattern).
   - require-shared / require-single-unconfirmed: do NOT overwrite in place. Add a new asset file, edit ONLY this source site's require/import path to it, or surface the shared-usage fact and let the operator decide. Never silently mutate an asset used elsewhere.
6. CONFIRM. State what changed (file + asset path + mode), and that the live canvas now reflects it.

FAILURE / HONESTY
- gpt-bridge is a ToS-bypass pool: single-IP rate limits, hung renders, accounts expire ~2026-09-04. The gen leg is best-effort. A failed generation must NEVER trigger a writeback (writeback happens only after a real PNG lands and the operator approves). Report gen failures plainly.
- A preview is reversible (visual only). A writeback is the commit. Keep that boundary.
- If source_asset_path is under the app root only (the server enforces containment, rejecting traversal) — do not try to read outside it; if you need an out-of-root image, fetch/copy it in explicitly.

ARTIFACTS (built 2026-06-06, .claude/mcp/canvas)
- MCP tool: canvas_preview_image({file_path, oid, instanceIndex}) > {ok, preview_id, preview_url}
- HTTP route: GET /gen-preview/<id> (image/png|jpeg|webp|gif, registry-lookup only, CSPRNG ids)
- edit-request tier: image-regen (requested_change.image_regen{prompt,from} + enriched source_asset_path/writeback_mode/writeback_note)
- resolver: lib/imageResolve.mjs (pure, never throws)
- panel: ElementInfoPanel.jsx ImageEditor regen-prompt + regen-button (helper spa/src/lib/imageRegenRequest.mjs)
- tests: imageResolve-smoke, gen-preview-smoke, preview-tool-smoke, edit-request-image-regen-smoke, panel-regen-smoke
Load it via canvas RECONNECT (server changes) + canvas tab RELOAD (overlay/SPA), and a gpt-bridge /mcp reconnect so canvas_preview_image surfaces.
