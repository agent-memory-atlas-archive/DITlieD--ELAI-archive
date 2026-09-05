Phase 0 Grounding Log
Date: 2026-04-06
Pass: 4 (BLOCK fixes from adversarial audit)

SPEC VERIFICATION
  docs/specs/architecture-overview.txt        Sections 1-4 | EXISTS, VERIFIED. LOC updated to 61,317.
  docs/specs/crates/core.txt                  Section 3    | EXISTS, UPDATED (added structural strip details, sandbox types)
  docs/specs/crates/core.txt                  Section 4    | EXISTS, UPDATED (valid Rust trait syntax, RuntimeContext, exception sites)
  docs/specs/crates/providers.txt             Section 2    | EXISTS, VERIFIED
  docs/specs/crates/providers.txt             Section 7    | EXISTS, UPDATED (dual-trait period, api/ LOC, sse.rs dup, tools->api dep)
  docs/specs/decisions/base-fork-comparison.txt             | EXISTS, UPDATED (staleness notes added)
  SPEC_GAP: none
  SPEC_CONFLICT: resolved — all specs now reference correct LOC numbers

BLOCKS FIXED (from adversarial audit)
  B1 WORKSPACE PATH: Added DESIGN DECISION section defining migration .elai_cc/rust/ -> .elai_cc/.
     Steps 0.1-0.3 explicitly use cd .elai_cc/rust. Step 0.4a creates new root. Steps 0.5+ use .elai_cc/.
  B2 CIRCULAR DEP: SandboxConfig/FilesystemIsolationMode moved to core/ in MODULE MAPPING.
     sandbox.rs has zero internal deps (only std+serde). Added to core.txt section 4.
     session_control.rs: identity -> core/, state -> orchestrator/ (split documented in 0.4a).
  B3 TOKENUSAGE STRUCTURAL: Added DESIGN DECISION for TokenUsage handling.
     Explicit remediation sub-steps in 0.2c: minimal shim, Session update, strip billing types.
  B4 ENTROPY EXCEPTIONS: Listed all exception sites. CI script excludes core/src/clock.rs and
     core/src/entropy.rs by path. DETERMINISTIC_CHECK updated to verify hits only in exception files.
     Updated core.txt section 4 with measured scope (60/31) and key files with hit counts.
  B5 CARGO SYNTAX: All cargo test commands now use -p elai_cc_contracts --test <name> syntax.
     No brace expansion. PRE_GATE checks are valid shell commands.
  B6 CONTRACTS CRATE: Created crates/contracts/ as test-only crate with dev-deps on all 9 crates.
     Step 0.6 explicitly creates it. All subsequent steps use cargo test -p elai_cc_contracts.
  B7 SLASH COMMANDS: Added step 0.2g — classify all 28+ commands as KEEP/STRIP.
     Step 0.4c strips claw-code commands during cli/ absorption. No conflicting commands.
  B8 API LOC: Fixed 1907 -> 4377 in MODULE MAPPING. Verified by find+wc on api/src/*.rs.

WARNS FIXED
  W1 STEP 0.4 SPLIT: Split into 0.4a (leaf: core/rules/providers), 0.4b (mid: tools/plugins/skeletons),
     0.4c (top: orchestrator/cli/cleanup). Each is one session.
  W2 CYCLIC CHECK: Removed cargo tree --edges grep "cyclic" (doesn't work). Rely on cargo check.
  W3 MANIFEST PATH: Specified .elai_cc/migration_manifest.toml (workspace-relative, survives restructure).
  W4 MSGPACK CHECK: Added round-trip test + python msgpack verification in 0.7b.
  W5 COMPAT-HARNESS: Moved to STRIP in step 0.2b (dead TypeScript bridge code).
  W6 TRAIT SYNTAX: Fixed to valid Rust: &self on ClockProvider, &mut self on EntropySource, typed params.
  W7 PROVIDER TRAIT: Added DESIGN DECISION documenting dual-trait period. Old marked #[deprecated].
  W10 TOOLS->API DEP: Added R24 risk. Step 0.4a explicitly handles tools/api import update.

SPEC FILES UPDATED
  docs/specs/crates/core.txt: section 3 (strip list expanded), section 4 (trait syntax, RuntimeContext,
    exception sites, sandbox types, measured scope)
  docs/specs/crates/providers.txt: section 7 (api/ LOC, dual-trait, sse.rs dup, tools dep, entropy sites)
  docs/specs/architecture-overview.txt: section 2 (LOC 59449->61317, tests 666->684, staleness note)
  docs/specs/decisions/base-fork-comparison.txt: staleness annotations on all LOC figures
  docs/specs/guide/implementation-guide.txt: phase 0 description updated for new step structure
