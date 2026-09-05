steal-goose-acp 2026-06-21

SOURCE
- repo: Block "goose" (github.com/aaif-goose/goose, fork of github.com/block/goose), Apache-2.0, local clone docs/.source/goose-main.
- interface map: workflow/papers/goose-interface.md (1 sonnet paper-ingestor, mode=source; RAG index FAILED on the known reranker/IPC crash, map is on disk).
- exploration: 1 opus explorer (provider auth + onboarding + desktop stack + TOS).
- trigger: user saw goose "log in using my Claude Code account" and asked (a) how, (b) is it against TOS, (c) can ELAI do the same, (d) onboarding + desktop language, (e) what can ELAI steal.

WHAT GOOSE ACTUALLY DOES (grounded, file:line in the interface map)
- Mechanism (a) — RIDE THE VENDOR'S OWN AUTHED CLI: goose spawns the vendor's already-authenticated CLI/adapter as a subprocess and talks a JSON protocol over stdio; it never reads/copies the credential. claude_code.rs spawns `claude --output-format stream-json` (deprecated); claude_acp.rs runs the claude-agent-acp adapter via the shared AcpProvider (acp/provider.rs) speaking the Agent Client Protocol; codex/cursor_agent/gemini_cli/copilot likewise. Both claude paths strip CLAUDECODE to defeat nested-session detection. This is the path that produced "it used my Claude Code account."
- Mechanism (b) — OAUTH-REPLAY: chatgpt_codex.rs runs its OWN PKCE flow with the Codex client-id, caches tokens, and calls chatgpt.com/backend-api/codex directly with a bearer token. The risky one.
- The common abstraction is AcpProvider (the "drive a third-party authenticated agent as a backend" wrapper); 5 goose providers ride it. goose DEPRECATED its direct claude_code.rs in favor of claude-acp.
- Onboarding: metadata-flag-driven configure form (oauth_flow / device_code_flow / api-key + setup_steps); secrets keyring-first (keyring crate) with a GOOSE_DISABLE_KEYRING plaintext fallback; config providers:{name:{enabled,model,configured}}.
- Desktop: Electron + React 19 + TypeScript + Vite + pnpm (electron-forge); renderer <-> the Rust goosed (axum) server over localhost HTTP with an X-Secret-Key handshake; Electron-main spawns goosed.

TOS (calibrated; Anthropic terms fetched live, OpenAI 403 so lower-confidence there)
- Anthropic Consumer Terms §3 (Claude Pro/Claude.ai): no automated/non-human access "except via an Anthropic API Key or where we otherwise explicitly permit it"; §2 no sharing/availing account credentials; + a no-competing-products/resale clause.
- (a) drive the official `claude` CLI: GRAY ZONE leaning non-compliant — the CLI is authorized, but goose-driving it headlessly with --dangerously-skip-permissions + CLAUDECODE stripped is plausibly bot/script access not covered by the API-key carve-out; competing-product clause is a separate exposure. Uncertain whether Anthropic treats it as a breach.
- (b) OAuth-replay (chatgpt_codex): HIGH-CONFIDENCE TOS violation in the Anthropic analogue; likely an OpenAI ToU violation (clause text unverified this session).
- goose ships subscription-reuse as a marketed FEATURE with ZERO disclaimer.
- Transport choice (direct-CLI vs ACP) does NOT change the TOS posture — same gray zone either way. This is a product/legal call for the user, orthogonal to the engineering steal.

CRITICAL CORRECTION (the "gap" collapsed against live code)
- First pass HYPOTHESIZED an un-ledgered "CLI/ACP-wrap" gap. WRONG. Verified in .elai_cc: ELAI ALREADY ships the ride-the-vendor-CLI mechanism, BROADER than goose — crates/providers/src/runtime/{claude_code,codex_cli,copilot_cli,gemini_cli,kiro_cli,opencode}.rs spawn the vendor CLI through cli_common::spawn_isolated_with_event (cli_common.rs:330) under Tier-C isolation, contract-tested (claude_code/codex_cli/copilot_cli_runtime_contract.rs). goose lacks kiro/opencode. ELAI is also MORE hardened (Tier-C + SSB CliSubprocessSpawned telemetry vs goose's bare Command::new + kill_on_drop). I-50's own premise names "claude_code runtime + CLI bridges riding vendor logins" as existing KEEP-list ground truth.
- So there is NO "login with any provider" gap. The ONLY substantive difference: ELAI hand-parses each vendor's --output-format stream-json (claude_code.rs:68 build_argv / :90 handle_event), while goose ADDITIONALLY speaks ACP and deprecated its direct path for it. grep confirms `acp`/`agent-client-protocol` appears NOWHERE in .elai_cc/crates.

STEAL CANDIDATES + VERDICTS (feature-map cited)
1. CLI/ACP-wrap "ride the vendor login" mechanism — ALREADY-SHIPPED (MULTI-RUNTIME BACKEND [WIRED], B-37; the 6 CLI bridges). Broader than goose. No work.
2. ACP TRANSPORT (AcpProvider; speak the Agent Client Protocol to the wrapped agent instead of hand-parsing each vendor's JSON) — NEW(J-64). NOT previously ledgered: the feature-map "ACP proc-macro covered/out-of-focus" line (L2024-28) referred to goose's goose-acp-MACROS code-gen crate, NOT the transport pattern. spec-eval verdict: ACP is the right DIRECTION for this subsystem (one client = all 25+ ACP agents incl ELAI's exact set + a registry; vendor format-churn absorbed by the vendor adapter — the LSP-for-agents unbundling), but the move is ADDITIVE behind a flag (a transport on the SAME cli_common Tier-C spawn seam, direct-CLI kept default), because ACP's thick bidirectional callback surface (fs/permission/tool callbacks) is a liability for ELAI's thin (prompt,config)->(text,usage) RuntimeAdapter and must be jailed deny-by-default. NOT a migration; goose itself keeps both families. dep agent-client-protocol =0.15.0 (Apache-2.0, github.com/agentclientprotocol/rust-sdk, published 2026-06-19, anti-slopsquat-verified via crates.io API 2026-06-21).
3. OAuth-replay (chatgpt_codex direct-token) — REJECT (DO-NOT-COPY): clear TOS-violation pattern, brittle (breaks on client-id/endpoint rotation); ELAI's OAuth lane stays I-50 the safe way.
4. Device-code OAuth arms — ALREADY-REJECTED (ledger: NOT-WORTH-PORTING; OAuth = I-50 AuthProvider only).
5. Onboarding UX (metadata-flag-driven configure form + keyring-first secrets) — ALREADY-PLANNED (J-30 PROVIDER BYOK). No new file; J-64's transport selector rides J-30's per-backend descriptor row.
6. Desktop stack (Electron/React) — REJECT: ELAI's dashboard is Tauri v2 + Svelte 5 (lighter); not a steal, ELAI's choice is better on footprint.

DIVERSITY CHECK
- attempted Grok + GPT cross-model votes on the ACP-vs-direct-CLI fork (spec-eval prescribes it on a hard fork); both gpt-bridge pools POOL_EXHAUSTED this session. Verdict rests on web + code grounding (convergent: goose's own deprecation + the 25-agent registry + the LSP analogy all point the same way), un-cross-checked.

OUTCOME
- NEW plan file: docs/thefinalglance/J-plan/J-64_acp_transport_for_agent_bridges.md (WAVE GOOSE-ACP-1, EXTENSION of B-37, M).
- Indexes updated: J-0_INDEX.md (title 63->64 + "ACP bridge transport" + Date 2026-06-21(b) + THE 64 SESSIONS + the J-64 sessions-table row), J-SUMMARY_INDEX.md (title 61->64 + the J-64 entry under a GOOSE-ACP TRANSPORT separator).
- Feature map updated: ELAI-FEATURE-MAP.md (header 2026-06-21b upsert clause; new AGENT-CLI BRIDGE TRANSPORT entry; goose-ledger rider distinguishing ACP-transport NEW(J-64) from the macro crate + the OAuth-replay DO-NOT-COPY).
- SSB: J-64 adds NO action_type (extends RuntimeEvent::CliSubprocessSpawned; SCHEMA/TV derived from action_type count ssb.rs:1508 stay UNCHANGED).
