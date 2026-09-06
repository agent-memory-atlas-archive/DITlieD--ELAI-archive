System Health Audit Checklist
Date created: 2026-04-14
Purpose: session to run through every major subsystem, fix compile/test issues

STATUS SNAPSHOT (as of 2026-04-14):

PASSING (no action needed unless regressions):
  elai-core         47 pass, 0 fail
  elai-providers   156 pass, 0 fail (1 ignored)
  elai-reasoning    52 pass, 0 fail
  elai-knowledge   390 pass, 0 fail

ISSUES TO FIX (ordered by blast radius):

1. elai-plugins — COMPILE FAIL
   error[E0762]: unterminated character literal
   Blocks: elai-tools, elai-orchestrator, elai-cli test builds (dep chain)
   Action: find the bad literal, fix it. Likely a raw string or backslash issue.

2. elai-rules — COMPILE FAIL (test build)
   GrepPattern struct changed (added language, file_context fields) but
   3 call sites not updated: contract_tests.rs, orchestrator match arm.
   Blocks: elai-orchestrator test build via dep chain.
   Action: update all GrepPattern construction sites to include new fields.

3. elai-orchestrator — BLOCKED by #1 and #2
   SSB tests: FIXED (patch-1.27, 26/26 pass now).
   Once plugins+rules compile, verify full orchestrator test suite.

4. elai-cli — BLOCKED by #1
   Once elai-plugins compiles, verify cli tests.

5. elai-tools — 1 FAILING TEST
   glob_and_grep_tools_cover_success_and_errors
   Glob tool returns success JSON instead of error for invalid glob pattern.
   Action: fix glob error handling or update test expectation.

6. elai_cc_contracts — 1 FAILING TEST
   arag_contract: query_pipeline returns KnowledgeError but contract
   expects Box<dyn Error>. Type mismatch after interface change.
   Action: update contract to match current KnowledgeError type.

7. Python sidecar (python/elai_inference/) — ZERO TESTS
   pyproject.toml declares pytest but no test files exist.
   Action: write basic smoke tests (server startup, /v1/completions endpoint).

8. Benchmarks (benches/model_capability/) — measurement harness only
   Not a test suite. Low priority.

SUBSYSTEM CHECKLIST (verify during audit session):
  [ ] elai-core: cargo test -p elai-core
  [ ] elai-providers: cargo test -p elai-providers
  [ ] elai-reasoning: cargo test -p elai-reasoning
  [ ] elai-knowledge: cargo test -p elai-knowledge
  [ ] elai-rules: cargo test -p elai-rules (fix GrepPattern first)
  [ ] elai-plugins: cargo test -p elai-plugins (fix literal first)
  [ ] elai-orchestrator: cargo test -p elai-orchestrator
  [ ] elai-tools: cargo test -p elai-tools
  [ ] elai-cli: cargo test -p elai-cli
  [ ] contracts: cargo test -p elai_cc_contracts
  [ ] cargo check --workspace (full workspace clean)
  [ ] cargo test --workspace (full green)
