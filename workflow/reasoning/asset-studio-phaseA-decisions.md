asset studio, phase A isolated-daemon foundation, decision doc
authored 2026-07-10, overnight autonomous run (owner asleep), controller session 1719a17d

WHAT
Build ONLY the isolated, zero-cost, non-colliding backend foundation of Phase A of TEIKOKU-ASSET-STUDIO-PLAN.md:
- A-1 asset daemon (teikoku-assets.py, FastAPI+uvicorn, 127.0.0.1:8878, own venv) + assets.db (sqlite WAL) + sequential job worker loop + /health /jobs /assets routes + windows start script
- A-2 adapter seam (the one python provider-adapter interface) + ONE deterministic zero-dependency tracer adapter (Pillow, produces a real PNG from prompt+seed, writes manifest + cost=0 row + license CLEAN)

WHERE (decision 4 adopted, low-risk)
Daemon + registry + files under F:\_stuffforgames\assets\ (beside models/tools, big-disk), OUTSIDE both git repos per plan line 136. Daemon code at F:\_stuffforgames\assets\_daemon\. No WSL crossing.

KEY DECISION (grounded, changes the plan)
gpt-bridge is a stdio browser-automation MCP (D:\.apps_workflow\.claude\mcp\gpt-bridge\server.mjs, headed Chrome + ChatGPT pool), reachable as MCP TOOLS from a Claude agent, NOT an HTTP service a standalone daemon can call. So the plan's A-2 "gpt-bridge-image adapter, zero-cost live smoke" is architecturally wrong for a daemon-owned lane: the gpt-bridge image lane must be driven agent-side (enqueue job -> a Claude-side worker harvests), or dropped for fal-FLUX once a fal key exists. Flagged, not papered over. The local-comfy-sdxl tracer needs ComfyUI booted + the GPU, not fired autonomously overnight. Therefore the ONLY live-smokeable tracer tonight is a deterministic local generator, which still proves the entire job lifecycle end to end.

EXPECTED OUTCOME (what verifies)
- uvicorn boots, curl /health = 200 off-agent
- POST /jobs {kind:image, provider:local-tracer, params:{prompt,seed}} -> worker runs -> asset row + real PNG file + manifest.json (RULE-5 fields: adapter, params, seed, output blake3, cost_cents=0, license CLEAN, retrieved_at) + cost row; GET /assets shows it; GET /assets/{id}/file serves the PNG
- contract tests green on the job lifecycle + adapter contract

BLOCKED (needs owner, deferred with exact blocker; NOT attempted)
- A-3 FE slice + A-4/A-5 smoke: edits shared WSL ~/teikoku tree (collides w/ the s-ad2623e9 e2e session) + needs a baked-exe rebuild; owner-eyeball gated per plan
- all Phase C/D-4/F-cloud/G cloud adapters: NO provider keys exist (grep-verified), real paid spend, owner decision 2
- D-1 local TRELLIS / D-2 pod / any GPU smoke: live GPU + pod cost, not fired overnight
- decisions 1 (GLB viewer / N-03), 2 (paid keys), 3 (hunyuan), 5 (audio scope): owner calls
- commit: TEIKOKU standing owner-hold, left uncommitted

MODEL ECONOMY (owner directive this run)
1 opus executor builds the slice; controller verifies off-agent itself; +1 executor only if incomplete; one sonnet auditor at the end, controller double-checks the auditor output. No prompt-engineer, no context-compiler/integration-explorer fleet (owner override of the heavy orchestrate protocol, logged here).
