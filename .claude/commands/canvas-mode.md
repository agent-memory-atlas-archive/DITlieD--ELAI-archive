---
description: Set the canvas-workbench mode flag (code|plan|research|design|arch). Mode is a PLAIN DOC FLAG, not an FSM substate.
argument-hint: <code|plan|research|design|arch>
---

CANVAS MODE: $ARGUMENTS

This command sets the active mode on the canvas-workbench doc. Mode is a PLAIN FLAG
on the canvas document, NOT an FSM substate. Do NOT call any mcp_fsm_* tool for this
command. The FSM (IDLE / EXECUTING / AUDITING / ...) is untouched; it is only
read-only-polled elsewhere for the ambient focus ring. Switching mode never
transitions the FSM and never creates a compound state.

Recognized modes (design Section 4):
  code     - CODE-GRAPH (dependency-cruiser depGraph.json > nodes/edges). LIVE in v1.
  plan     - decomposition-tree.md as concept>phase>step>babystep nodes. RESERVED (not built in v1).
  research - tier-tagged researcher reports as source nodes + contradiction edges. RESERVED.
  design   - screenshot-nodes from the capture pipeline + expo-router page-graph. RESERVED.
  arch     - model-authored architecture diagram. RESERVED.

Behavior:
  - If $ARGUMENTS is "code": this is the live mode; the canvas already renders the
    CODE-GRAPH. Confirm the mode is code and tell the user the canvas is at
    http://127.0.0.1:7323.
  - If $ARGUMENTS is one of plan / research / design / arch: the mode is RECOGNIZED
    and RESERVED. Do NOT error. Tell the user the mode is reserved for a later phase
    and that v1 only renders the code mode. The flag is accepted (a future canvas doc
    will namespace nodes by this mode), but no new render path exists yet.
  - If $ARGUMENTS is anything else (or empty): list the five recognized modes and ask
    the user to pick one. Do not guess.

How to open the canvas:
  - The canvas MCP server serves the renderer on http://127.0.0.1:7323 (open it in a
    browser tab or VS Code Simple Browser). The server is the "canvas" entry in
    .mcp.json; it starts with the Claude Code session (stdio MCP) and also serves the
    http page.
  - If the page does not load, the server may not be running. It can be started
    standalone for the http view with:  node .claude/mcp/canvas/server.mjs --serve

How to hand a node to the model (POINT-TO-CONTEXT deixis, design Section 2 FEEDBACK):
  - Click a node (or edge) in the canvas. That writes the tiered selection
    (Focused / Blurry / Peripheral) and fires the MCP resources/list_changed
    notification so Claude Code re-fetches.
  - In your next prompt, reference the selection resource AND add an intent line:
        @canvas:canvas://selection  - explain why this module is a suspected-dead orphan and propose the fix
  - The intent line is mandatory (Mosca 2021): pointing tells the model WHICH thing,
    it does not tell it WHAT to do. The selection resource alone is not a request.
  - Two more resources are available for broader context:
        @canvas:canvas://viewport    Blurry  - ids/types of the on-screen nodes
        @canvas:canvas://periphery   Peripheral - off-screen clusters + counts
  - A node whose provenance is taint:true (web / untrusted source) is delivered as
    DATA-NOT-INSTRUCTIONS: its content is wrapped and labeled untrusted, so it can
    never silently re-enter the prompt as a command. code.* nodes are local + untainted.
