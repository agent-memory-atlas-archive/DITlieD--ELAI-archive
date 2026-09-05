# C-44: Composer 2 Runtime Backend Tier (B-37 Extension)

Objective: add Cursor's Composer 2 model as a third runtime backend tier
alongside local Qwen2.5-Coder (NF4+FA2 sidecar) and Sonnet 4.6 (cloud
escalation). Adoption is config-level via B-37 BackendStanza; cost positioning
between local-free and Sonnet-premium.

Pre-requisites: B-37 (Phase 4M Multi-Runtime Backend Abstraction) SHIPPED;
C-39 (CLI MCP surface) SHIPPED (for unified dispatch); confirmed Composer 2
public API access (gating prerequisite).
Parallel-safe with: C-35..C-43, C-45..C-53. Sequence AFTER C-39 (dispatch path
unified) and AFTER C-42 (zero-trust fuzzer covers the new backend's tool
authz).

Paper grounding (arXiv:2603.24477, 2026-03-25, Cursor Research team — verified
abs page 2026-05-17 via prior oracle): "Composer 2 Technical Report" published
SWE-bench Multilingual 73.7, Terminal-Bench 61.7, CursorBench 61.3; pricing
$0.50/M input, $2.50/M output. Built on Kimi K2.5 base.

## Context

ELAI's B-37 abstracted the runtime backend so multiple model families can sit
behind the same FSM dispatch. Currently shipped: local Qwen2.5-Coder
(deterministic, free, latency 30-50 tok/s on consumer GPU) + Sonnet 4.6
(cloud, expensive, 1M context). Composer 2 fills the middle tier: cheaper
than Sonnet by ~6x on input, ~3.6x on output; benchmarked specifically for
agentic coding loops (where Sonnet is generalist and Qwen is small-class).

Open risk per prior oracle 2026-05-17: Composer 2 may not expose public API
outside Cursor IDE. C-44 is gated on API access verification at Task 1.
If no public API, C-44 closure = "documented infeasibility note" rather than
landed integration.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-37_multi_runtime_backend_abstraction.md
- .elai_cc/crates/runtime/src/backends/ (existing local + cloud impls)
- arXiv:2603.24477 (Composer 2 Technical Report)
- cursor.com/blog/composer-2 + cursor.com/docs (API surface documentation, fresh fetch)

## Tasks

1. **API access verification** — WebFetch Cursor docs + attempt unauthenticated
   API ping. If no public API found, write infeasibility note + close C-44
   as "deferred pending Cursor public API."

2. **Source-grounding** — paper-ingestor (mode=paper) against arXiv:2603.24477.
   Capture: prompt-format expectations, system-prompt conventions,
   token-counting quirks. Produces workflow/papers/composer2-interface.md.

3. **BackendStanza entry** — add to elai_cc.toml:
   ```
   [backends.composer2]
   enabled = false  # default off; opt-in per project
   api_key_env = "COMPOSER2_API_KEY"
   model_id = "cursor-composer-2"  # confirmed via API at Task 1
   max_input_tokens = 200000  # confirm via API
   max_output_tokens = 8192
   role_routing = ["executor", "auditor"]  # default roles; user can override
   cost_per_m_input_usd = 0.50
   cost_per_m_output_usd = 2.50
   ```

4. **Composer2Backend impl** — new file crates/runtime/src/backends/composer2.rs
   implementing the Backend trait. async invoke(prompt, params) -> Response.
   Surface API errors as structured BackendError, never panic.

5. **Skill-routing integration (depends on C-41)** — Composer 2 default
   routing per skill (after C-41 SkillRegistry lands):
   - heavy_plan: prefer Sonnet 4.6 (planning quality matters most)
   - focused_edit: prefer Composer 2 (cost-efficient code emission)
   - simple_query: prefer local Qwen 7B
   - benchmark_run: per-config (controlled in C-54)

6. **SSB action types** — backend_composer2_invoked, backend_composer2_error,
   backend_composer2_cost_accrued (carries usd_cents).

7. **Tests first (TDD)**:
   - 4M.X.3: Composer2Backend invokes mocked HTTP endpoint, parses response
   - 4M.X.4: API key absent → structured error (not panic, not silent fallback)
   - 4M.X.5: token-counting matches Composer 2's tokenizer claim (verify
     via API response.usage field)
   - 4M.X.6: cost_accrued SSB row populates correctly per token usage
   - 4M.X.7: backend failover (if Composer 2 returns 503, fallback to
     configured next-tier-backend — typically Sonnet 4.6 or local)

8. **Cost-tracking dashboard panel (dashboard-v2/BackendCostPanel.svelte)** —
   rolling 24h cost per backend, per session. Critical because Composer 2
   adds a non-zero default cost vector to local-first sessions.

9. **Contract tests** — tests/contracts/composer2_contract.rs verifying
   Backend trait conformance.

## WIRING_CONTRACT

produces_symbols:
  - Composer2Backend (crates/runtime/src/backends/composer2.rs)
  - SSB action_types: backend_composer2_invoked, backend_composer2_error,
    backend_composer2_cost_accrued
  - BackendCostPanel.svelte (dashboard-v2)
  - elai_cc.toml [backends.composer2] section
consumes_symbols:
  - Backend trait (crates/runtime/src/backends/mod.rs from B-37)
  - SkillRegistry (C-41) for role routing
  - SSB event bus
  - COMPOSER2_API_KEY env var
reachable_from:
  - elai_cc.toml [backends.composer2.enabled]
  - per-skill routing override
stub_ban:
  - missing API key MUST surface as structured error, never silent fallback
  - cost_accrued SSB MUST emit on every successful invocation
  - failover path MUST emit backend_composer2_error before falling back
frontend_pair: BackendCostPanel.svelte
observable_effect:
  trigger: /task with [backends.composer2.enabled] = true
  expect_ssb: backend_composer2_invoked + backend_composer2_cost_accrued
              per LLM call when routed to Composer 2

## DETERMINISTIC_CHECK

```bash
test -f .elai_cc/crates/runtime/src/backends/composer2.rs
rg "backend_composer2_invoked|backend_composer2_error|backend_composer2_cost_accrued" .elai_cc/crates/orchestrator/src
cargo test -p elai_runtime -- composer2 2>&1 | grep "test result.*ok"
cargo test -p elai_runtime -- composer2_contract 2>&1 | grep "test result.*ok"
grep "composer2" .elai_cc/elai_cc.toml
```

## ACCEPTANCE

- [ ] API access verification result documented (operational OR infeasible)
- [ ] If operational: workflow/papers/composer2-interface.md produced
- [ ] elai_cc.toml [backends.composer2] block validates
- [ ] Composer2Backend implements Backend trait with no placeholder bodies
- [ ] API-key-absent path produces structured error
- [ ] Per-token cost tracking emits SSB row
- [ ] Failover to next-tier on 503/rate-limit
- [ ] All 3 SSB action types emitted with non-null usd_cents on cost
- [ ] TDD tests pass
- [ ] BackendCostPanel renders rolling cost
- [ ] Contract tests pass
- [ ] If infeasible: closure note in workflow/sessions/{id}/composer2-deferral.md
      with API status + revisit-when-trigger

## Files

- ADD: .elai_cc/crates/runtime/src/backends/composer2.rs
- ADD: .elai_cc/crates/runtime/tests/composer2_tdd.rs
- ADD: .elai_cc/crates/runtime/tests/contracts/composer2_contract.rs
- ADD: .elai_cc/crates/dashboard-v2/src/lib/components/BackendCostPanel.svelte
- ADD: workflow/papers/composer2-interface.md (conditional)
- MODIFY: .elai_cc/crates/runtime/src/backends/mod.rs (register Composer2Backend)
- MODIFY: .elai_cc/elai_cc.toml ([backends.composer2] section)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-37_multi_runtime_backend_abstraction.report.md
  — annotate C-44 third-tier extension.

## Next

C-44 closure (or deferred-note) clears the path for C-54 benchmark
publication. Composer 2 baseline becomes a third config in the SWE-bench
matrix: local-7B-only, hybrid Sonnet-Composer2-local, full-cloud.
