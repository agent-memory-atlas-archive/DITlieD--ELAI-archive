E-24 MCP Bridge Hardening -- Decision Record

What: Harden MCP bridge with schema-hash pinning, sanitization, token-budget enforcement,
Streamable HTTP transport, LLM-guard hook, and gateway-rejection guard.

Key decisions:

1. blake3 for schema pinning (HARD RULE: never blake2/sha2 per CLAUDE.md anti-stale-training).

2. chars/4 heuristic for token estimation -- no new tokenizer dep. tiktoken-rs anti-slopsquatting
   check would require cargo search verification; chars/4 is accurate enough for the drop-tail
   use case and adds zero deps.

3. Fail-closed as LLM-guard default -- when sidecar unavailable, block all MCP calls.
   Alternative was fail-open (allow). Fail-closed chosen because ELAI's threat model
   (OWASP MCP03:2025, 92% exploit rate) warrants erring on the side of caution.
   User can override with --no-llm-guard.

4. Gateway-rejection guard warns, does not hard-block by default. Hard-block requires
   --strict-no-gateway. Rationale: team may have legitimate proxy setups; warning surfaces
   the risk without breaking workflows.

5. SSE-legacy transport kept behind opt-in config flag (transport: sse_legacy). 90-day
   deprecation window recommended before removal per plan open questions.

6. Pin storage: .elai_cc/.elai/mcp/pins/{encoded_url}.toml (TOML for human readability
   and git-diffable pin files). Alternative was SQLite in the SSB -- rejected because
   pins need to be inspectable/auditable by the user without a database tool.

7. LLM-guard prompt is deterministic and does NOT include full chat history -- only tool
   name + sanitized params. This prevents the guard itself from being a prompt-injection
   vector.

8. pre_mcp_call_decision is gated behind #[cfg(feature = "mcp")] to avoid reqwest dep
   on non-mcp builds. LlmGuardConfig and GuardDecision types are always compiled so
   contract tests can reference them without the mcp feature.

9. regex crate used for sanitize_description (verified present in Cargo.lock 1.12.3,
   already in plugins [dependencies] as regex.workspace = true). No new dep added.

10. tempfile added to plugins [dev-dependencies] for mcp_schema_pin unit tests.
    Already in Cargo.lock (3.27.0) -- no new network fetch needed.

Expected test outcomes:
- mcp_schema_pin_contract: postmark_style_rug_pull_blocked PASS
- mcp_sanitization_line_jumping_contract: all 5 pattern contracts PASS
- mcp_budget_enforcement_contract: 20-tool fixture drops tail PASS
- mcp_transport_streamable_http_contract: StreamableHttp is default PASS
- llm_guard_decision_contract: fail-closed default PASS
- gateway_rejection_startup_warning: gateway types identified PASS
- fuld_mcp_bridge_contract: 3 new E-24 regression contracts PASS

Unknowns at E-24 ship:
- C-42 (Zero-Trust MCP Fuzzing) will fuzz this surface -- ordering confirmed: E-24 first.
- A2A protocol deferred: no confirmed ELAI consumer that speaks A2A.
- 90-day SSE-legacy deprecation timeline starts from E-24 ship date.
