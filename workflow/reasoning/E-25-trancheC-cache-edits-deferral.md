E-25 Tranche C (C.3) -- cache_edits documentation-only deferral
Source session: s-22600 Tranche C
Date: 2026-05-31

DEFERRED: cache_edits producer module.

Rationale (matches s-22300 Tranche-B4 D14 + Tranche-C explorer SUBSYSTEM 3):
- cache_edits is a request-side API parameter on outbound api.anthropic.com calls (Anthropic prompt-cache passthrough). The local sidecar cannot honor it. There is NO server-side cache state the orchestrator can populate today.
- The catalog field ClaudeCodeLadderResult.cache_preserved_bytes (architecture::claude_code_arch.rs:455) is the future fold-point; no producer populates it this tranche.
- Cloud-Sonnet routing path (B-37 PerRoleRuntime cloud lane) is the prerequisite producer site. Once cloud-Sonnet calls return server-side cache stats, those stats fold into ClaudeCodeLadderResult.cache_preserved_bytes via a future patch-N.

Will-resolve-in: future tranche after cloud-Sonnet edit-output capture lands (B-37 cloud lane wiring -> ClaudeCodeLadderResult population). No env gate minted this tranche.

References:
- workflow/reasoning/E-25-tranche-B4-decisions.md D14
- workflow/sessions/s-22600/exploration/trancheC-integration.md SUBSYSTEM 3
- crates/orchestrator/src/architecture/claude_code_arch.rs:455 (cache_preserved_bytes field)
- crates/orchestrator/src/compaction_ladder.rs:30-34 (module-doc cross-ref to cache_edits)
