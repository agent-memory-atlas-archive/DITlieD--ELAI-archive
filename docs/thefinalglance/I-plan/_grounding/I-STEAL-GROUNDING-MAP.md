# I-STEAL-GROUNDING-MAP — Track 20 (COMPETITOR-STEAL PORTS, I-45..I-54)
Grep date 2026-06-10, session s-iplan-track20-2026-06-10. Source design doc: docs/.source/STEAL-LIST.md
(itself a grounded analysis dated 2026-06-10: provider gaps grep-verified against crates/, gajae semantics
MEDIUM from per-repo SUMMARYs). This map re-confirmed the ELAI-side anchors in live source the same day.
Detail files cite SL-n IDs instead of re-asserting facts. elai.map staleness unknown at authoring; every
anchor below was confirmed by direct glob/grep in live source, not the map.

================================================================================
SL-1..SL-11 FINDINGS
================================================================================

SL-1  CANONICAL CONSTANTS (live grep 2026-06-10): SCHEMA_VERSION = 221
      (.elai_cc/crates/orchestrator/src/ssb.rs:2981), TOTAL_VARIANTS = 532
      (.elai_cc/crates/orchestrator/tests/action_type_lock.rs:601). Identical to the I-0_INDEX baseline
      (same-day authoring). NEVER hard-code; re-read BOTH at edit (W-9).

SL-2  BASH VALIDATION EXISTS, REDIRECT ABSENT. crates/tools/src/bash_validation.rs exists (glob-confirmed).
      The STEAL-LIST claim "validates but does not REDIRECT" is STEAL-LIST-grep-asserted; the I-46 executor
      re-greps the file for any rewrite/redirect path before building (W-7). gajae reference:
      docs/.source/gajae-code/packages/coding-agent/src/tools/bash-interceptor.ts + bash-command-fixup.ts +
      bash-allowed-prefixes.ts (SL-10 read rule applies).

SL-3  OAUTH IS ANTHROPIC-ONLY. crates/providers/src/oauth.rs exists (glob-confirmed); STEAL-LIST verified it
      as the Claude-plan OAuth. No other subscription-auth broker exists in crates/providers/ (STEAL-LIST
      grep, HIGH). gajae reference: packages/ai/src/auth-broker/ + auth-gateway/ + auth-storage.ts.

SL-4  CAPABILITY/PRICING DATA IS SCATTERED (A1 confirmed): crates/providers/src/pricing.rs,
      crates/providers/src/vram_tier.rs, crates/core/src/vram_tier.rs (a DUPLICATE name across two crates),
      crates/orchestrator/src/observability/pricing_rollup.rs. No single model-capability catalog. gajae
      reference: packages/ai/src/api-registry.ts + models.json + model-manager.ts.

SL-5  G-2/G-8 SUBSTRATE IS SHIPPED (corrects the stale "rra.rs absent" memory note from G-plan authoring):
      crates/orchestrator/src/rra.rs, crates/orchestrator/src/attestation.rs, crates/cli/src/rra_cmd.rs all
      exist (glob-confirmed 2026-06-10). I-45 is an EXTENSION on live code, not a forward dep. F-4
      stdout_hash is the adjacent precedent (STEAL-LIST E1 note).

SL-6  CHECKPOINT SUBSTRATE IS SHIPPED: crates/orchestrator/src/architecture/edit_checkpoints.rs,
      crates/cli/src/checkpoint_cmd.rs, contracts e25_edit_checkpoints_contract.rs +
      e25_edit_checkpoints_parallel_contract.rs (grep-confirmed). H-plan Track 8 (H-49) plans the full
      CAS/undo-tree ON TOP and is PLANNED, not shipped. I-52 is therefore ACTIVATION-as-agent-tool of the
      E-25 substrate, with an explicit H-49 coordination clause (if H-49 ships first, I-52 becomes the tool
      surface over its store; prefer reuse of its EditHistoryEvent row).

SL-7  TREE-SITTER SUBSTRATE IS SHIPPED: crates/knowledge/Cargo.toml:13-22 pins tree-sitter 0.26 + grammars
      for rust/python/go/typescript/java/c-sharp/ruby/javascript/c. The CLAUDE.md rejection of an ast-grep
      dependency STANDS (STEAL-LIST B3 caveat); I-54 builds structural edit on the existing grammars only.

SL-8  NO BEDROCK / NO AZURE ANYWHERE IN crates/ — STEAL-LIST grep 2026-06-10, HIGH confidence, accepted
      without re-grep (same-day). The CLI-bridge philosophy note stands: ELAI's existing reach is
      runtime/{anthropic_api, claude_code, codex_cli, copilot_cli, gemini_cli, kiro_cli, opencode,
      composer2, genai_adapter, hermes} + local serving + oauth.rs. I-49 adds DIRECT clients only for the
      three protocol shapes no CLI bridge can provide (SigV4+EventStream, Azure Responses, the
      OpenAI-as-Anthropic shim seam).

SL-9  NO RECIPE RUNNER IN crates/tools/src — grep for just/makefile/recipe (case-insensitive) hits only
      file_ops.rs / git_commit.rs / action_classifier.rs incidentally. gajae reference:
      packages/coding-agent/src/tools/recipe/ + archive-reader.ts + json-tree.ts (B9 cherry-picks).

SL-10 GAJAE SEMANTICS ARE MEDIUM CONFIDENCE (STEAL-LIST confidence block): file existence verified, behavior
      from the per-repo SUMMARY. BINDING READ RULE: every Track 20 executor whose session ports gajae logic
      (I-46, I-49, I-50, I-51, I-53) MUST read the named gajae source file(s) BEFORE EXECUTING and re-scope
      the port if the SUMMARY misdescribed them. A port scoped only from the SUMMARY is a W-3 violation.

SL-11 ROACH-PI / LAZYCODEX SEMANTICS ARE MEDIUM CONFIDENCE (same block): techniques read from
      roach-pi-SUMMARY.md / lazycodex-SUMMARY.md, not direct source. Same binding read rule for I-45
      (technique 1) and I-47 (techniques 2+3): read the summary's cited extension source under
      docs/.source/roach-pi/ before EXECUTING.

================================================================================
DESIGN-DOC CORRECTIONS (Track 20)
================================================================================
SLC-1 The STEAL-LIST's B2 framing "lighter than H-49" is corrected to "ACTIVATION of shipped E-25
      edit_checkpoints" (SL-6): the snapshot machinery exists and is test-proven; what is absent is the
      agent-callable tool surface. Do not build a second snapshot store.
SLC-2 A1 "single-catalog shape" must NOT delete the per-VRAM tier logic (vram_tier.rs encodes the measured
      8/12/16/24/48GB plan — memory: project_vram_tier_plan). The catalog WRAPS those sources as one read
      surface; removal/simplification needs explicit user confirmation (CLAUDE.md).
SLC-3 B3 is an IDEA steal only: gajae ships ast-grep-core (vendored); ELAI's implementation uses the SL-7
      grammars. Any dep named "ast-grep" in an I-54 diff is a BLOCK.

================================================================================
CANONICAL CONSTANTS
================================================================================
SCHEMA_VERSION 221 / TOTAL_VARIANTS 532 (SL-1, live 2026-06-10). Track 20 adds at most +6 variants
(I-45/I-46/I-47/I-50/I-52/I-54); I-48/I-49/I-51/I-53 reuse existing rows (GD-1). Bump sites + lockstep
discipline per I-0_INDEX.

================================================================================
WEB VERIFICATION
================================================================================
None required at map time: Track 20 cites no papers and no version-specific external APIs except AWS
SigV4/EventStream and Azure OpenAI Responses, which are deliberately deferred to I-49 authoring-time
verification (the executor WebSearches the current protocol docs + anti-slopsquat-verifies any aws-* crate
candidates BEFORE the port; recorded as a W-10 pre-task in the detail file).

End of I-STEAL-GROUNDING-MAP.
