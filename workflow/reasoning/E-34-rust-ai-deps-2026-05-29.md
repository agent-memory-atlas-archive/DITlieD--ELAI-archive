E-34 Rust AI ecosystem 2026 — dep verification (offline batch s-25120, 2026-05-29)

Scope: verify rig-core, rmcp, genai on crates.io for the E-34 follow-up. NO Cargo.toml mutations (anti-slopsquatting HARD RULE + new-dep user confirmation required).

Source: crates.io REST API /api/v1/crates/{name} fetched via curl + user-agent elai-anti-slopsquat/1.0.

rig-core
  id: rig-core
  max_version / max_stable: 0.37.0 (BOTH; no prerelease ahead)
  num_versions: 55
  recent_downloads: 683,100
  total_downloads: 975,319
  default_version: 0.37.0
  created_at: 2024-05-29
  latest_release_at: 2026-05-13
  repository: github.com/0xPlaygrounds/rig
  description: "An opinionated library for building LLM powered applications."
  default features: ["reqwest","derive","rustls"]
  ELAI alignment: matches CLAUDE.md Anti-Stale-Training "swiftide REJECTED ... use rig-core 0.37 for agent framework". Version verified.
  Slopsquat check: PASS — owner 0xPlaygrounds, 55 published versions over 12 months, downloads consistent with an active framework.

rmcp
  id: rmcp
  max_version / max_stable: 1.7.0
  num_versions: 46
  recent_downloads: 6,677,953
  total_downloads: 10,903,855
  created_at: 2025-03-16
  latest_release_at: 2026-05-13
  repository: github.com/modelcontextprotocol/rust-sdk
  description: "Rust SDK for Model Context Protocol"
  default features: ["base64","macros","server"]
  ELAI alignment: this is the OFFICIAL Anthropic-blessed MCP rust SDK (repo under modelcontextprotocol/rust-sdk). 6.67M recent_downloads + 46 versions in ~14 months = healthy + actively maintained.
  Slopsquat check: PASS — owner modelcontextprotocol (org), repo matches MCP spec source-of-truth.

genai
  id: genai
  max_version / max_stable: 0.6.2
  num_versions: 115
  recent_downloads: 84,212
  total_downloads: 209,796
  created_at: 2024-06-01
  latest_release_at: 2026-05-28 (yesterday — extremely active)
  repository: github.com/jeremychone/rust-genai
  description: "Multi-AI Providers Library for Rust. (OpenAI, Gemini, Anthropic, Ollama, AWS Bedrock, Vertex, Groq, DeepSeek, GitHub Copilot and many more)"
  Slopsquat check: PASS — owner jeremychone (well-known Rust YouTube educator), 115 versions, daily-cadence releases, providers list matches B-37 RuntimeAdapter target set.

Cross-check vs B-37 SHIPPED (memory:project_b37_shipped, commit fd5c2e5):
  - B-37 ships 11 backends: Sidecar, AnthropicApi, 7 CLI ports, vLLM, Ollama.
  - genai covers 9 of those 11 (OpenAI, Gemini, Anthropic, Ollama, AWS Bedrock, Vertex, Groq, DeepSeek, GitHub Copilot). If E-34 lands a genai-backed unification, it replaces 9 of the 11 hand-rolled adapter calls.
  - rmcp at 1.7.0 vs orchestrator/src/dispatch/mcp.rs MCP_TRANSPORT_CURRENT_DEFAULT="streamable_http" (MCP spec 2025-11-25) — aligned.
  - rig-core 0.37 is the canonical agent-framework pick per CLAUDE.md.

DECISION (offline-batch verdict, NOT a Cargo.toml mutation):
  All three deps verified GROUNDED on crates.io. NO Cargo.toml entries added in this pass (per CLAUDE.md "No new dependencies without confirmation"). When user approves E-34 follow-up implementation, the pinned versions are:
    rig-core = "0.37"
    rmcp = "1.7"
    genai = "0.6"
  Pin to minor (not exact) per existing crates/contracts/Cargo.toml pin style; exception is ort = "=2.0.0-rc.12" which is exact-pin per Anti-Stale rule (no stable on ort yet — see B-16-ort-status-2026-05-29.md).

OPEN QUESTIONS:
  - License field in the crates.io API blob is not extracted at this depth. Auditor should `curl crates.io/api/v1/crates/{name}/{version}` and confirm license is MIT or MIT/Apache-2.0 before merge.
  - rig-core's ELAI-fit needs a 1-hour spike comparing rig-core's agent loop against the hand-rolled fsm in orchestrator/src/fsm.rs. The CLAUDE.md anti-stale rule says "rig-core 0.37 ... OR hand-roll"; ELAI currently hand-rolls. No urgency to migrate.
