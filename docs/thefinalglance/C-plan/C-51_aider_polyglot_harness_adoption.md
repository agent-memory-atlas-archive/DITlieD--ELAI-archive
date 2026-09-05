# C-51: Aider Polyglot Harness Adoption (Test Infrastructure Only)

Objective: bolt Aider's open-source Polyglot benchmark harness (225 Exercism
problems across C++/Go/Java/JS/Python/Rust) onto ELAI's task_runner as a
local-runnable test infrastructure. NO model invocations yet — this patch
ships the harness so C-55 (the actual benchmark run) is a config flip
when RunPod becomes available.

Pre-requisites: Phase 4F code completeness SHIPPED.
Parallel-safe with: C-35..C-50, C-52, C-53.

Reference grounding (verified via prior oracle 2026-05-17):
aider.chat/docs/leaderboards/ — 225-problem Polyglot benchmark. Local entry
example: ollama qwen2.5-coder:32b at 72.9% on edit leaderboard (matching
ELAI's local 32B tier). MIT-licensed, github.com/Aider-AI/aider/benchmark.

## Context

C-55 is the benchmark RUN against the harness. C-51 is the harness ITSELF
landed in the repo, ready to run, but with no model bound yet. This split
is intentional:
1. Harness landing is CPU-only, deterministic, can be code-reviewed and
   merged without GPU access.
2. Benchmark run depends on RunPod / local GPU; C-55 fires when hardware
   is online.
3. By landing C-51 first, the benchmark binding (model adapter shim) is
   ready and tested by the time C-55 fires.

The harness reads tasks, presents per-task scaffolding to the SUT (System
Under Test), evaluates emitted code by running the task's hidden tests in
a sandbox, scores pass/fail. Standard Aider methodology.

## Reads

- aider.chat/docs/leaderboards/ (fresh fetch for current methodology)
- github.com/Aider-AI/aider/benchmark/ (harness source)
- .elai_cc/crates/task_runner/ (existing ELAI test infrastructure)
- .elai_cc/sandbox/ (post-C-11 sandboxed-execution layer)

## Tasks

1. **Fork or vendor Aider's benchmark harness** — MIT license permits.
   Vendor at .elai_cc/external/aider_polyglot/. Pin git SHA in
   .elai_cc/external/manifests.toml so reproducibility is locked.

2. **Polyglot task corpus** — 225 Exercism problems in the harness format.
   Vendor alongside.

3. **ELAI SUT adapter (crates/task_runner/src/aider_polyglot_adapter.rs)** —
   the harness expects a callback: given (problem_dir, prompt), return
   emitted code. The adapter wraps ELAI's task pipeline: spawn
   prompt-engineer → executor → wait → collect emitted file → return.
   No model binding at the adapter level — it consumes whatever runtime
   backend the elai_cc.toml [backends] specifies.

4. **Sandboxed test runner** — leverages C-11 sandbox-completeness layer
   to run hidden tests without leaking to host. Each language family has
   its own runner shim (gcc for C++, go test for Go, mvn for Java,
   jest for JS, pytest for Python, cargo test for Rust).

5. **Scorecard format** — per-task: problem_id, language, time_ms,
   tokens_in, tokens_out, cost_usd, passed (bool). Aggregate scorecard
   matches Aider's format so direct comparison with the leaderboard is
   straightforward.

6. **SSB action types** — polyglot_task_started, polyglot_task_completed,
   polyglot_scorecard_emitted.

7. **Tests first (TDD)**:
   - 4G.X.3: harness loads 225 tasks; non-empty per-language counts
   - 4G.X.4: SUT adapter dispatches a synthetic minimal task and round-trips
   - 4G.X.5: sandboxed test runner executes a known-good Python solution
     and reports pass
   - 4G.X.6: sandboxed test runner detects a known-bad Python solution
     (intentional bug) and reports fail
   - 4G.X.7: scorecard JSON matches Aider's leaderboard format

8. **No benchmark RUN in C-51** — this patch lands infrastructure. The
   `elai benchmark polyglot` CLI subcommand exists but EXITS WITH ERROR
   ("RunPod or local GPU required; C-55 will fire when available") unless
   --dry-run flag is set, in which case it runs 5 sample tasks against a
   mock backend for smoke testing.

9. **Contract tests** — tests/contracts/polyglot_adapter_contract.rs.

## WIRING_CONTRACT

produces_symbols:
  - .elai_cc/external/aider_polyglot/ (vendored harness + 225 tasks)
  - PolyglotAdapter (crates/task_runner/src/aider_polyglot_adapter.rs)
  - SandboxedTestRunner per language (crates/task_runner/src/sandboxes/)
  - elai benchmark polyglot CLI subcommand
  - SSB action_types: polyglot_task_started, polyglot_task_completed,
    polyglot_scorecard_emitted
consumes_symbols:
  - existing prompt-engineer + executor + auditor pipeline
  - C-11 sandbox-completeness layer
  - backends from elai_cc.toml (any registered runtime)
  - SSB event bus
reachable_from:
  - elai benchmark polyglot [--dry-run]
stub_ban:
  - harness vendor MUST include MIT LICENSE file
  - external/manifests.toml MUST pin SHA
  - benchmark CLI without RunPod-or-GPU MUST refuse to run full sweep
  - sandboxed runner MUST honor C-11 isolation; no host leak

## DETERMINISTIC_CHECK

```bash
test -d .elai_cc/external/aider_polyglot
test -f .elai_cc/external/aider_polyglot/LICENSE
grep -q "aider_polyglot" .elai_cc/external/manifests.toml
test -f .elai_cc/crates/task_runner/src/aider_polyglot_adapter.rs
rg "polyglot_task_started|polyglot_task_completed|polyglot_scorecard_emitted" .elai_cc/crates/orchestrator/src
cargo test -p elai_task_runner -- polyglot 2>&1 | grep "test result.*ok"
elai benchmark polyglot --dry-run | grep "5 sample tasks"
```

## ACCEPTANCE

- [ ] Aider Polyglot harness vendored with MIT LICENSE + pinned SHA
- [ ] 225 tasks present, per-language counts visible
- [ ] PolyglotAdapter dispatches synthetic task end-to-end
- [ ] Per-language SandboxedTestRunner shims implemented for all 6 languages
- [ ] Scorecard JSON matches Aider's leaderboard schema (verified by diff
      against an existing leaderboard JSON)
- [ ] All 3 SSB action types emit
- [ ] TDD tests pass
- [ ] CLI subcommand refuses full sweep without GPU; --dry-run works
- [ ] Contract tests pass
- [ ] **NOTE**: this patch does NOT publish a benchmark number; that's C-55.
      Closure here is infrastructure landed + smoke-tested.

## Files

- ADD: .elai_cc/external/aider_polyglot/ (vendored)
- ADD: .elai_cc/external/manifests.toml
- ADD: .elai_cc/crates/task_runner/src/aider_polyglot_adapter.rs
- ADD: .elai_cc/crates/task_runner/src/sandboxes/cpp.rs + go.rs + java.rs + js.rs + python.rs + rust.rs
- ADD: .elai_cc/crates/task_runner/tests/polyglot_tdd.rs
- ADD: .elai_cc/crates/task_runner/tests/contracts/polyglot_adapter_contract.rs
- MODIFY: .elai_cc/crates/cli/src/main.rs (benchmark polyglot subcommand)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (3 new action types)

## Next

C-51 closure means C-55 becomes a config flip + GPU bind, not a code change.
Reduces RunPod time-to-result on the day RunPod arrives.
