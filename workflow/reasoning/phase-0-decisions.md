Phase 0 Steps 0.1-0.5 Decision Doc

What: Strip fork-specific code (telemetry, compat-harness, usage pricing, compact, summary_compression, lsp_client, oauth) and restructure 9 current crates + runtime modules into 9 elai_cc crates.

Key decisions:
1. TokenUsage shim: Keep struct definition (4 fields) + Default impl in usage.rs. Strip: format_usd, pricing_for_model, ModelPricing, UsageCostEstimate, UsageTracker. Consumers (conversation.rs, session.rs, main.rs, api/types.rs) updated to use only the struct. Shim replaced by core::TokenUsage in step 0.4a.
2. compact.rs strip: conversation.rs imports CompactionConfig etc. Must stub or remove compact references in conversation.rs simultaneously.
3. summary_compression strip: tools/lib.rs imports compress_summary_text. Must remove that import.
4. telemetry removal order: Remove use-site imports first (api, runtime, conversation.rs), then Cargo.toml deps, then directory.
5. compat-harness removal: Only consumer is rusty-claude-cli/main.rs (extract_manifest, UpstreamPaths). Remove import, then dep, then directory.
6. sandbox.rs extraction: No crate:: imports found (verified grep). Clean leaf extraction to core/.
7. prompt.rs: Audit for hardcoded system prompts. Decision (REPLACE or STRIP) recorded after audit.
8. remote.rs: Evaluate for DEFERRED or STRIP. Contains upstream proxy bootstrap logic.
9. commands/lib.rs: 28+ slash commands. Audit and classify only in step 0.2g. No removal.

Expected outcome: cargo check 0 errors after each sub-step. 9 crates in .elai_cc/crates/. ~3000-4000 LOC removed in strip phase.

Resolved unknowns:
- compact.rs: 30+ consumers in commands and main.rs. DEFERRED to restructure.
- oauth.rs: 20+ call sites in api/anthropic.rs and main.rs. DEFERRED to restructure.
- usage.rs: TokenUsage + pricing used by api/types.rs and main.rs. KEEP, extract to core in 0.4a.
- prompt.rs: FRONTIER_MODEL_NAME="Claude Opus 4.6", CLAUDE.md references, "# Claude instructions" header. REPLACE (structure reusable, content Claude-specific).
- remote.rs: proxy bootstrap + session token. DEFAULT_REMOTE_BASE_URL="https://api.anthropic.com". DEFERRED (keep for restructure, replace constants).
- commands/lib.rs: 48 slash commands total. ~20 STRIP candidates (Login/Logout/Upgrade/Share/Feedback/Desktop/Stickers/Insights/Thinkback/ReleaseNotes/SecurityReview/Keybindings/PrivacySettings/Voice/Color/OutputStyle/Ide/Brief/Advisor/Bughunter/Ultraplan/Teleport), ~28 KEEP.

Deviations from original plan:
- oauth.rs, compact.rs, usage.rs not stripped (blast radius too large for in-place strip)
- telemetry crate absorbed into runtime as module (types needed for API client)
- Total LOC removed: ~3967 (telemetry 526 + compat-harness 357 + mock-anthropic 1157 + lsp_client 747 + summary_compression 300 + mock_parity_harness test 880)
