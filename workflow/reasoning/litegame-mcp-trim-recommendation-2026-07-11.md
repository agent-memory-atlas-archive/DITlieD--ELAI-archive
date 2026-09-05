litegame MCP trim recommendation (ram-build-efficiency Phase 1 / unit D, 2026-07-11)

STATUS: recommendation only. This document proposes changes to D:\.liteGAME_workflow\.mcp.json. It performs ZERO edits to any D:\ path (hard scope boundary of the ram-build-efficiency plan). The liteGAME owner applies these in a liteGAME session.

WHY liteGAME, not ELAI
Context-package Area 3 (workflow/sessions/s-ram-eff-0710/context-package.md) established that the ELAI root's .mcp.json carries only 5 core/necessary servers (codegraph-elai, local-rag, canvas, gpt-bridge, pod-lifecycle) with no art/unity chaff, so per-project MCP trimming (handoff Fix 5) has near-zero opportunity on ELAI itself. The entire body of the opportunity is in D:\.liteGAME_workflow\.mcp.json, which registers 20 top-level servers, ~10 of which are art/unity-specific and load in every liteGAME session because this config format has no conditional-loading mechanism. Each loaded stdio server is one or more resident processes per session; on a 63.2GB host near commit ceiling, trimming the always-loaded art/unity set is direct RAM headroom.

SERVER GATE LIST (art/unity-specific; gate behind an opt-in profile so they load only in art/unity sessions)
- figma            (bunx)                        design import
- meshy            (bunx)                        3D asset gen
- miro             (official hosted http)        whiteboard read
- miro-write       (bunx community)              whiteboard write
- comfy-pipeline   (miniconda python)            ComfyUI image pipeline
- blender-bridge   (miniconda python)            Blender bridge
- comfyui-cwags    (npx -y comfyui-mcp)          ComfyUI MCP (see bunx flag below)
- unityMCP         (uvx stdio, port 8080 hub)    Unity editor bridge
- unityMCP-runtime (stdio)                        Unity runtime bridge
- ae-mcp-dakkshin  (After Effects bridge)         ALREADY inert (comment: disabled until enabled)
- ae-mcp-ishu86    (After Effects bridge)         ALREADY inert
- ae-mcp-rs        (After Effects bridge)         ALREADY inert
- unrealclaude     (Unreal bridge)                ALREADY inert
- vibeue           (http, 127.0.0.1:8089/mcp)     Unreal/UE bridge

KEEP-ALWAYS (shared/core; do NOT gate)
- local-rag, canvas, gpt-bridge (shared single install), pod-lifecycle (shared), codegraph-zc, codegraph-eo.

The 4 ae-mcp-*/unrealclaude entries are already gated off by comment in the current file; the actionable set is the ~10 live art/unity servers (figma, meshy, miro, miro-write, comfy-pipeline, blender-bridge, comfyui-cwags, unityMCP, unityMCP-runtime, vibeue).

OPT-IN PROFILE MECHANISM (recommendation)
The .mcp.json format has no native per-session conditional loading, so the mechanism must live outside the single always-loaded registry. Two options, preferred first:

1. Split-registry + disabledMcpjsonServers (preferred). Keep the full server set in .mcp.json but add a disabledMcpjsonServers array (Claude Code honors this key in .claude/settings.json / settings.local.json) listing the entire art/unity gate list above by name. Art/unity work is then opted IN per-session by removing a name from the disabled array (or via a small "art profile" settings overlay). Default state = art/unity servers disabled, so a normal liteGAME code session pays for none of them. This is the lowest-risk change: one additive key, fully reversible, no server definitions move.

2. Separate profile registries. Maintain two .mcp.json-equivalents (a lean code profile without the art/unity block, and a full art profile) and select at launch. Higher maintenance (two files drift), only worth it if option 1's per-session toggle proves too manual in practice.

Recommendation: adopt option 1. Populate disabledMcpjsonServers with the 10 live art/unity server names by default; document the opt-in toggle in the liteGAME projectBrief so it does not drift.

comfyui-cwags npx -> bunx FLAG (per .claude/rules/bun-not-node.md)
comfyui-cwags is currently `"command":"npx","args":["-y","comfyui-mcp"]` — the literal triple-process-spawn case bun-not-node.md targets (npx > npm-shim > server). When the liteGAME owner next edits this entry, rewrite it to bun: `"command":"bunx","args":["comfyui-mcp"]` (bunx 1.3.14 is installed and available). This drops the npx wrapper chain to a single bun-hosted process. Per bun-not-node.md's lazy-caveat: do not mass-rewrite the other working entries on sight; convert comfyui-cwags because it is the confirmed npx wrapper and the owner is already opening the config to add disabledMcpjsonServers. If comfyui-mcp genuinely breaks under bunx, capture the actual error first, then fall back to npx and note why in one line — a captured failure is the only valid reason to keep npx.

APPLICATION (owner, liteGAME session — outside this plan's scope)
1. Add disabledMcpjsonServers to D:\.liteGAME_workflow\.claude\settings.local.json listing the 10 live art/unity server names.
2. Rewrite comfyui-cwags command from npx to bunx in D:\.liteGAME_workflow\.mcp.json.
3. Restart the liteGAME session(s) so the disabled set takes effect.
Expected: the art/unity server processes stop spawning in default code sessions; RAM reclaimed per handoff Fix 5. All changes reversible (remove the name from the array to re-enable).
