B-37 Phase 4M Multi-Runtime Backend Abstraction — HANDOFF
Session: s-18544
Date: 2026-05-16
Status: ABANDONED IN THIS SESSION

Summary
  B-37 was attempted as 4 sub-cycles (A=foundation, B=CLI adapters, C=native HTTP +
  per-role + fallback, D=CLI command + dashboard + migration). After B-37A reported
  20/20 tests passing and B-37B started its CLI adapter implementation, the
  executor was unable to reconcile the on-disk state of providers/src/runtime/.
  Symptoms:
    - providers/src/lib.rs had a `//pub mod runtime; // patch-R1 temp-park` comment
      (the working-tree was modified vs HEAD which has `pub mod runtime;`)
    - PowerShell/ls intermittently showed runtime/ as empty, then later showed
      it populated with 16 .rs files including isolation/ subdir
    - Sub-agent reports of writes succeeded, but verification on disk gave
      conflicting results within seconds
    - The "patch-R1" comment implies an earlier (or concurrent) B-37 attempt
      that temp-parked the module rather than fixing/completing it
    - mcp_session_list reported no other active sessions, but the symptoms
      strongly suggest a concurrent unregistered session editing
      providers/src/runtime/

Current on-disk state (as of session end)
  - .elai_cc/crates/providers/src/lib.rs: contains `pub mod runtime;` (uncommented)
  - .elai_cc/crates/providers/src/runtime/ exists with: mod.rs, types.rs,
    secret.rs, sidecar.rs, anthropic_api.rs, stubs.rs, cli_common.rs,
    claude_code.rs, codex_cli.rs, opencode.rs, hermes.rs, gemini_cli.rs,
    kiro_cli.rs, copilot_cli.rs, isolation/mod.rs, isolation/default_spawn.rs
  - Quality of those files is UNVERIFIED at session close. The B-38 executor
    fixed a mismatched-brace bug in copilot_cli.rs:140 (run_complete) so at
    least one of them had a real build defect. Other files may have similar
    issues. The workspace compiles cleanly enough for B-38's purposes
    (cargo check --workspace clean except for s-10380/B-42's task_runner.rs
    issue which is unrelated).
  - .elai_cc/crates/contracts/tests/runtime_adapter_contract.rs +
    runtime_cli_adapters_contract.rs were deleted by main model during the
    abandonment (they imported elai_providers::runtime symbols that may or
    may not exist as expected). If you want to recover those test files, re-run
    the executor with a clean slate.

Required next session
  1. Identify the OTHER session that has been editing providers/src/runtime/
     (search git reflog, check ~/.claude/projects/ for other Claude Code
     instances, ask the team)
  2. Coordinate or pause that session
  3. Audit the existing providers/src/runtime/ on disk: cargo check -p elai-providers
     --features sidecar --no-default-features. Fix any defects.
  4. Either declare existing B-37A SHIPPED and continue to B-37B/C/D, or
     reset providers/src/runtime/ and start B-37A from scratch following the
     decision record at workflow/reasoning/B-37-decisions.md (still valid).

Artifacts that DID land usefully
  - workflow/papers/ouroboros-runtime-interface.md (paper-ingestor MEDIUM, 420 lines)
  - workflow/reasoning/B-37-decisions.md (decision record; still valid)
  - workflow/reasoning/B-37A-grounding.md (spec-grounding report; still valid)
  - workflow/sessions/s-18544/context-package.md (B-37A briefing)
  - workflow/sessions/s-18544/context-package-B37B.md (B-37B briefing — has
    per-adapter CLI invocation table verbatim from ouroboros + Rust port hints)
  - workflow/sessions/s-18544/blast-radius.md (impact map for B-37A)
  - workflow/sessions/s-18544/B-37A-audit.md (B-37A auditor verdict PASS)

WebSearch findings (verified during this session)
  - `claude` CLI headless: `claude -p "<prompt>" --output-format stream-json
    --verbose --include-partial-messages` for NDJSON streaming output;
    `--input-format stream-json` for bidirectional stdin. Permission mode:
    `--permission-mode {acceptEdits|bypassPermissions|default}`.
  - `copilot` CLI permissions: `--allow-tool 'Kind(arg)'` /
    `--deny-tool`; `--allow-all`; deny precedes allow.

The B-37 work is recoverable. The blocker is operational (concurrent session
coordination), not technical. The design + decisions are sound.
