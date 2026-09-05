# C-42: Semantic Gateway Zero-Trust Fuzzing for MCP Bridge

Objective: stand up a deterministic fuzzer against the Phase 4K MCP bridge
that finds 100% of unauthorized-state transitions in the synthetic-fixture
suite, per arXiv:2604.25555. Catches MCP-surface authz/validation bugs
BEFORE C-40 (GitHub PR adapter) and C-44 (Composer 2 backend tier) start
landing new tool invocations on the bridge.

Pre-requisites: Phase 4K MCP bridge SHIPPED; C-35 (Tool Attention dynamic
gating) SHIPPED; C-39 (CLI MCP surface) SHIPPED.
Parallel-safe with: C-36, C-37, C-38, C-41, C-43..C-53 (orthogonal). NOT
parallel-safe with C-35 (overlapping plugins/src/mcp/ surface).

Paper grounding (arXiv:2604.25555, 2026-05-01, verified abs page 2026-05-17):
"From CRUD to Autonomous Agents: Formal Validation and Zero-Trust Security
for Semantic Gateways in AI-Native Enterprise Systems." Measured claim
(per verifier 2026-05-17): "100% unauthorized-state discovery" against the
paper's deterministic fuzzer on synthetic semantic-gateway fixtures.
Reference implementation released alongside paper.

## Context

The MCP bridge is the most-attacked external surface in ELAI:
- Other agentic frameworks plug into it
- IDE extensions invoke it
- C-39 brings CLI as a third consumer
- Future C-44 brings Composer 2 routing as a fourth
- C-40 brings GitHub PR review-comment routing as a fifth

Each consumer can request any registered tool. Without zero-trust validation
at the bridge, a misbehaving consumer (or an attacker-controlled prompt
emitted by an LLM) can invoke tools the FSM never authorized. ELAI's
current trust model is "the dashboard is trusted; nothing else talks to the
bridge." C-39 and C-40 break that assumption.

C-42 lands a deterministic fuzzer that:
1. Enumerates all (FSM state × tool × argument shape) tuples reachable from
   a malicious-consumer perspective.
2. For each tuple, asserts the bridge either AUTHORIZES (per C-35 gate +
   per-tool authz policy) or REJECTS with structured error.
3. Reports any UNAUTHORIZED-AUTHORIZED outcome as a finding.

Paper claims 100% discovery on synthetic fixtures; the practical target for
ELAI is 100% on ELAI's own synthetic fixture set + zero false-negatives on
the post-fix re-run.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-35_tool_attention_dynamic_gating.md
- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-39_mcp_bridge_cli_surface.md
- .elai_cc/crates/plugins/src/mcp/server.rs (current authz path, if any)
- workflow/papers/semantic-gateway-fuzzer.md (paper-ingestor + mode=source, Task 1)

## Tasks

1. **Source-grounding** — paper-ingestor (mode=paper) AND (mode=source)
   against arXiv:2604.25555 + the reference fuzzer implementation.
   Produces workflow/papers/semantic-gateway-fuzzer.md.

2. **AuthzPolicy struct + per-tool authz declarations** — every registered
   tool gets a `#[authz(states = "PLANNING,EXECUTING", actors = "fsm,user")]`
   attribute (or equivalent toml manifest entry). AuthzPolicy::check
   (tool_name, fsm_state, actor) -> Authorized | Rejected{reason}.

3. **fuzzer crate (NEW, crates/mcp_fuzzer/)** — feature="mcp_fuzzer",
   integration-test-only (NOT shipped in production binary). Exports
   FuzzCorpus, FuzzRunner, FuzzReport. Enumerates tuples deterministically
   (seeded RNG for reproducibility).

4. **Synthetic-fixture corpus** — under .elai_cc/crates/mcp_fuzzer/corpus/:
   - Every registered tool × every FSM state × 5 representative arg shapes
     (valid, oversized, undersized, malformed, escape-attempting).
   - At least 200 tuples total to ensure coverage is non-trivial.

5. **Fuzzer runner** — bind to a localhost MCP bridge instance, invoke each
   tuple, record (authorized, expected_authorized, response_shape) triple.
   FuzzReport summarizes findings:
   - Total tuples
   - Authorized-when-should-reject (CRITICAL findings)
   - Rejected-when-should-authorize (LATENT BUGS, P1)
   - Response-shape violations (BLOCK if any tool returns non-structured
     error on rejection)

6. **CI integration** — new CI job runs the fuzzer at every PR. Findings
   table posted as PR comment. Any CRITICAL finding blocks merge.

7. **SSB action types** — mcp_authz_check, mcp_authz_reject, fuzzer_finding.
   First two emit at runtime; fuzzer_finding emits when the integration test
   surfaces a violation.

8. **Tests first (TDD)**:
   - 4K.X.3: FuzzCorpus generates deterministic tuple set from seed
   - 4K.X.4: FuzzRunner invocation round-trips through real MCP bridge
   - 4K.X.5: AuthzPolicy::check honors per-state-per-tool declarations
   - 4K.X.6: KNOWN BUG fixture (one deliberate misconfiguration) is found
     by the fuzzer (regression test for the fuzzer itself)
   - 4K.X.7: Re-run after fix produces zero CRITICAL findings

9. **Per-tool authz audit** — sweep all currently-registered tools and add
   #[authz(...)] attribute. Tools without an explicit authz declaration
   default to DENY-ALL (fail-closed); registration emits a warning to SSB
   on startup ("tool X registered with no authz declaration; defaulting to
   DENY-ALL — declare explicit policy in source").

10. **Dashboard panel (dashboard-v2/AuthzPanel.svelte, PLANNED)** —
    real-time mcp_authz_check + mcp_authz_reject rate, click-through to
    rejection reason histogram.

11. **Contract tests** — tests/contracts/mcp_authz_contract.rs verifies
    AuthzPolicy interface shape + DENY-ALL default behavior.

## WIRING_CONTRACT

produces_symbols:
  - AuthzPolicy, AuthzResult (plugins/src/mcp/authz.rs)
  - FuzzCorpus, FuzzRunner, FuzzReport (crates/mcp_fuzzer/)
  - SSB action_types: mcp_authz_check, mcp_authz_reject, fuzzer_finding
  - AuthzPanel.svelte (dashboard-v2, PLANNED)
  - CI job (fuzzer integration run)
consumes_symbols:
  - MCP bridge endpoint (plugins/src/mcp/server.rs)
  - ToolGate from C-35
  - SSB event bus
reachable_from:
  - every MCP tool invocation (runtime authz check)
  - CI fuzzer job (offline)
stub_ban:
  - any tool registered without explicit #[authz] declaration MUST default
    DENY-ALL — registration WITHOUT failure but WITH structured warning
  - AuthzPolicy::check MUST emit mcp_authz_check SSB row before returning;
    silent authz is banned
  - rejection SSB MUST carry structured reason; "unauthorized" alone is
    insufficient (need tool_name + fsm_state + actor + denied_reason)
frontend_pair: AuthzPanel.svelte
observable_effect:
  trigger: any MCP tool call (production or fuzzer)
  expect_ssb: mcp_authz_check row before tool runs; mcp_authz_reject row
              if denied
  expect_artifact: CI fuzzer report JSON in test artifacts; zero CRITICAL
                   findings

## DETERMINISTIC_CHECK

```bash
# 1. AuthzPolicy module
test -f .elai_cc/crates/plugins/src/mcp/authz.rs

# 2. Fuzzer crate
test -d .elai_cc/crates/mcp_fuzzer

# 3. Corpus has at least 200 tuples
ls .elai_cc/crates/mcp_fuzzer/corpus | wc -l | awk '$1 >= 200 {exit 0} {exit 1}'

# 4. SSB action types
rg "mcp_authz_check|mcp_authz_reject|fuzzer_finding" .elai_cc/crates/orchestrator/src

# 5. All tools have #[authz] attribute
rg "register_tool\!|fn register" .elai_cc/crates/plugins/src
# Cross-check: every match should have a sibling #[authz(...)] within 5 lines

# 6. TDD tests pass
cargo test --features mcp_fuzzer -p elai_mcp_fuzzer 2>&1 | grep "test result.*ok"

# 7. CI fuzzer job present
grep -r "mcp_fuzzer" .github/workflows/

# 8. Contract test passes
cargo test -p elai_plugins -- mcp_authz_contract 2>&1 | grep "test result.*ok"
```

## ACCEPTANCE

- [ ] paper-ingestor produces workflow/papers/semantic-gateway-fuzzer.md
- [ ] AuthzPolicy + AuthzResult implemented with no placeholder bodies
- [ ] Every registered tool has #[authz] declaration OR defaults DENY-ALL
      with startup warning
- [ ] Fuzzer crate compiles only with feature="mcp_fuzzer"
- [ ] Corpus has ≥200 deterministic tuples
- [ ] FuzzReport surfaces CRITICAL / P1 / shape-violation findings
- [ ] All 3 SSB action types emitted with structured reasons
- [ ] CI fuzzer job runs at every PR
- [ ] CRITICAL findings block PR merge
- [ ] TDD tests 4K.X.3..4K.X.7 written and passing BEFORE wiring
- [ ] Known-bug regression test (4K.X.6) finds the deliberate misconfiguration
- [ ] Post-fix re-run produces zero CRITICAL findings
- [ ] AuthzPanel.svelte renders authz check/reject rate
- [ ] Contract tests pass

## Files

- ADD: .elai_cc/crates/plugins/src/mcp/authz.rs
- ADD: .elai_cc/crates/mcp_fuzzer/Cargo.toml
- ADD: .elai_cc/crates/mcp_fuzzer/src/lib.rs
- ADD: .elai_cc/crates/mcp_fuzzer/src/corpus.rs
- ADD: .elai_cc/crates/mcp_fuzzer/src/runner.rs
- ADD: .elai_cc/crates/mcp_fuzzer/src/report.rs
- ADD: .elai_cc/crates/mcp_fuzzer/corpus/* (200+ tuples)
- ADD: .elai_cc/crates/mcp_fuzzer/tests/fuzzer_tdd.rs
- ADD: .elai_cc/crates/plugins/tests/contracts/mcp_authz_contract.rs
- ADD: .elai_cc/crates/dashboard-v2/src/lib/components/AuthzPanel.svelte (PLANNED)
- ADD: workflow/papers/semantic-gateway-fuzzer.md
- ADD: .github/workflows/mcp-fuzzer.yml
- MODIFY: .elai_cc/Cargo.toml (mcp_fuzzer workspace member)
- MODIFY: .elai_cc/crates/plugins/src/mcp/server.rs (wire AuthzPolicy)
- MODIFY: every tool registration site to carry #[authz] attribute
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\steps\phase-4K.report.md
  — annotate C-42 zero-trust arrival.

## Next

C-42 closure means the MCP bridge surface is hardened BEFORE C-40 and C-44
land third- and fourth-party consumers. If the fuzzer surfaces CRITICAL
findings, those become individual P0 bug tickets and gate C-40/C-44 progress
until resolved. Do not merge C-40 (GitHub PR adapter) until the fuzzer
reports zero CRITICAL on the synthetic corpus.
