E-38 MCP transport-version drift probe (offline batch s-25120, 2026-05-29)

Scope: WebFetch modelcontextprotocol.io/specification, compare current spec version to orchestrator's hardcoded MCP_TRANSPORT_CURRENT_DEFAULT constant.

Spec source (WebFetch 2026-05-29):
  modelcontextprotocol.io/specification
  Authoritative TypeScript schema:
    github.com/modelcontextprotocol/specification/blob/main/schema/2025-11-25/schema.ts
  Spec version: 2025-11-25
  Page navigation footer cards link to:
    /specification/2025-11-25/architecture
    /specification/2025-11-25/basic
    /specification/2025-11-25/server
    /specification/2025-11-25/client
  Transports listed in the index: protocol is JSON-RPC 2.0 over a stateful connection. Transport-specific endpoints (stdio / streamable-http / sse) are documented under /specification/2025-11-25/basic.

Orchestrator-side state:
  File: crates/orchestrator/src/dispatch/mcp.rs
  Constants:
    MCP_TRANSPORT_ROADMAP_TARGET = "stateless+tasks+registry-q4-2026+saml-oidc-pkce-q2-2026"
    MCP_TRANSPORT_CURRENT_DEFAULT = "streamable_http"
  Comment grounding: "Aligned with elai_plugins::mcp_bridge::McpTransportMode::default() = StreamableHttp (MCP spec 2025-11-25)."

Drift verdict: NO DRIFT.
  - Spec version on modelcontextprotocol.io: 2025-11-25 (current).
  - Orchestrator's MCP_TRANSPORT_CURRENT_DEFAULT matches spec's recommended transport (streamable_http).
  - Comment in dispatch/mcp.rs already cites "MCP spec 2025-11-25" — matches today's spec page.

E-38 producer-side substrate (per CLAUDE.md E-38 HARD RULE):
  - Posture doc at docs/strategy/ELAI-2026-POSTURE.md SECTION 1(4) cites:
      "Streamable HTTP per spec 2025-11-25"
      modelcontextprotocol.io/development/roadmap (verified May 2026)
  - Contract test crates/contracts/tests/e38_anti_pattern_contracts.rs::no_bespoke_agent_protocol_parallel_to_mcp is in place.
  - SSB row McpTransportVersionLogged emits once per process boot.

Follow-up triggers:
  - If MCP spec bumps to 2026-MM-DD: edit MCP_TRANSPORT_CURRENT_DEFAULT comment + posture doc reference to the new spec date.
  - If `streamable_http` is deprecated in favor of `stateless` (per roadmap target): edit MCP_TRANSPORT_CURRENT_DEFAULT constant + elai_plugins::mcp_bridge::McpTransportMode::default().

RUNPOD MARKER UPDATE: none required. dispatch/mcp.rs:38-41 comment was already accurate; no PENDING marker referenced this probe specifically.

Posture doc staleness gate: docs/strategy/ELAI-2026-POSTURE.md is the canonical reference (90-day staleness check at crates/orchestrator/src/roadmap_signals.rs::check_posture_doc_age). File was current at the time of this probe; no PostureDocStale SSB row would fire.
